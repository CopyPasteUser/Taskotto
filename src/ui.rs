use crate::{
    actions::{SignalAction, send_signal},
    config::{Density, Settings, SortBy, Theme, is_hex_color},
    format,
    monitor::{Monitor, Process, Snapshot},
};
use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Processes,
    Performance,
    Details,
    Preferences,
}

enum Command {
    Refresh,
    SetPeriod(u64),
    Shutdown,
}

struct Worker {
    commands: Sender<Command>,
    snapshots: Receiver<io::Result<Snapshot>>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn start(interval: u64, ctx: egui::Context) -> Self {
        let (commands, rx) = mpsc::channel();
        let (tx, snapshots) = mpsc::channel();
        let thread = thread::spawn(move || {
            let mut monitor = Monitor::default();
            let mut interval = interval;
            loop {
                let result = monitor.sample();
                if tx.send(result).is_err() {
                    break;
                }
                ctx.request_repaint();
                loop {
                    match rx.recv_timeout(Duration::from_millis(interval)) {
                        Ok(Command::Refresh) | Err(mpsc::RecvTimeoutError::Timeout) => break,
                        Ok(Command::SetPeriod(ms)) => interval = ms,
                        Ok(Command::Shutdown) | Err(mpsc::RecvTimeoutError::Disconnected) => return,
                    }
                }
            }
        });
        Self {
            commands,
            snapshots,
            thread: Some(thread),
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Default)]
struct History {
    cpu: VecDeque<f32>,
    memory: VecDeque<f32>,
    received: VecDeque<f32>,
    sent: VecDeque<f32>,
}

impl History {
    fn record(&mut self, snapshot: &Snapshot) {
        for (queue, value) in [
            (&mut self.cpu, snapshot.cpu_percent),
            (&mut self.memory, snapshot.memory.used_fraction() * 100.0),
            (
                &mut self.received,
                snapshot.rates.received_per_second as f32,
            ),
            (&mut self.sent, snapshot.rates.sent_per_second as f32),
        ] {
            if queue.len() == 120 {
                queue.pop_front();
            }
            queue.push_back(value);
        }
    }
}

struct PendingAction {
    target: Process,
    action: SignalAction,
}

#[derive(Clone)]
struct ProcessRow {
    process: Process,
    depth: usize,
}

pub struct TaskottoApp {
    settings: Settings,
    worker: Worker,
    snapshot: Option<Snapshot>,
    history: History,
    page: Page,
    query: String,
    focus_search: bool,
    selected: Option<(i32, u64)>,
    pending: Option<PendingAction>,
    notification: Option<String>,
    theme_accent_input: String,
    my_uid: u32,
    users: HashMap<u32, String>,
}

impl TaskottoApp {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        settings: Settings,
        warning: Option<String>,
    ) -> Self {
        apply_theme(&context.egui_ctx, &settings);
        let accent_input = settings.accent_hex.clone().unwrap_or_default();
        Self {
            worker: Worker::start(settings.refresh_ms, context.egui_ctx.clone()),
            settings,
            snapshot: None,
            history: History::default(),
            page: Page::Processes,
            query: String::new(),
            focus_search: false,
            selected: None,
            pending: None,
            notification: warning,
            theme_accent_input: accent_input,
            my_uid: unsafe { libc::geteuid() },
            users: load_users(),
        }
    }

    fn accept_snapshots(&mut self) {
        while let Ok(result) = self.worker.snapshots.try_recv() {
            match result {
                Ok(snapshot) => {
                    self.history.record(&snapshot);
                    self.snapshot = Some(snapshot);
                }
                Err(err) => self.notification = Some(format!("Monitoring error: {err}")),
            }
        }
    }

    fn save_preferences(&mut self, ctx: &egui::Context) {
        self.settings.normalize();
        apply_theme(ctx, &self.settings);
        let _ = self
            .worker
            .commands
            .send(Command::SetPeriod(self.settings.refresh_ms));
        if let Err(err) = self.settings.save() {
            self.notification = Some(format!("Could not save preferences: {err}"));
        }
    }

    fn selected_process(&self) -> Option<&Process> {
        let (pid, start) = self.selected?;
        self.snapshot
            .as_ref()?
            .processes
            .iter()
            .find(|p| p.pid == pid && p.start_ticks == start)
    }

    fn queue_action(&mut self, action: SignalAction) {
        if let Some(process) = self.selected_process().cloned() {
            self.pending = Some(PendingAction {
                target: process,
                action,
            });
        }
    }

    fn processes(&self) -> Vec<ProcessRow> {
        let Some(snapshot) = &self.snapshot else {
            return Vec::new();
        };
        let query = self.query.to_lowercase();
        let mut matches: Vec<Process> = snapshot
            .processes
            .iter()
            .filter(|p| {
                (self.settings.show_kernel_threads || !is_kernel_thread(p))
                    && (!self.settings.only_my_processes || p.user_id == self.my_uid)
                    && (query.is_empty()
                        || p.name.to_lowercase().contains(&query)
                        || p.command.to_lowercase().contains(&query)
                        || p.pid.to_string().contains(&query)
                        || self
                            .users
                            .get(&p.user_id)
                            .is_some_and(|s| s.to_lowercase().contains(&query)))
            })
            .cloned()
            .collect();
        matches.sort_by(|a, b| {
            let order = match self.settings.sort_by {
                SortBy::Cpu => a.cpu_percent.total_cmp(&b.cpu_percent),
                SortBy::Memory => a.resident_bytes.cmp(&b.resident_bytes),
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Pid => a.pid.cmp(&b.pid),
            };
            let order = if self.settings.descending {
                order.reverse()
            } else {
                order
            };
            order.then_with(|| a.pid.cmp(&b.pid))
        });
        if !self.settings.tree_view {
            return matches
                .into_iter()
                .map(|process| ProcessRow { process, depth: 0 })
                .collect();
        }
        // Preserve sorting among siblings, then visit every child exactly once.
        let pids = matches.iter().map(|p| p.pid).collect::<HashSet<_>>();
        let mut children: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut by_pid = HashMap::new();
        for process in matches {
            if process.pid != process.parent_pid && pids.contains(&process.parent_pid) {
                children
                    .entry(process.parent_pid)
                    .or_default()
                    .push(process.pid);
            }
            by_pid.insert(process.pid, process);
        }
        let roots = by_pid
            .keys()
            .copied()
            .filter(|pid| {
                !by_pid
                    .get(pid)
                    .is_some_and(|p| p.pid != p.parent_pid && by_pid.contains_key(&p.parent_pid))
            })
            .collect::<HashSet<_>>();
        let ordered = self.sort_pids(by_pid.values().collect());
        let mut visited = HashSet::new();
        let mut result = Vec::with_capacity(by_pid.len());
        for pid in ordered
            .iter()
            .copied()
            .filter(|pid| roots.contains(pid))
            .chain(ordered.iter().copied())
        {
            append_tree(pid, 0, &by_pid, &children, &mut visited, &mut result);
        }
        result
    }

    fn sort_pids(&self, processes: Vec<&Process>) -> Vec<i32> {
        let mut rows = processes;
        rows.sort_by(|a, b| {
            let cmp = match self.settings.sort_by {
                SortBy::Cpu => a.cpu_percent.total_cmp(&b.cpu_percent),
                SortBy::Memory => a.resident_bytes.cmp(&b.resident_bytes),
                SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                SortBy::Pid => a.pid.cmp(&b.pid),
            };
            (if self.settings.descending {
                cmp.reverse()
            } else {
                cmp
            })
            .then(a.pid.cmp(&b.pid))
        });
        rows.into_iter().map(|p| p.pid).collect()
    }

    fn topbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.add_space(9.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("◉  Taskotto").strong().size(21.0));
                ui.add_space(14.0);
                if let Some(snapshot) = &self.snapshot {
                    let cpu = snapshot.cpu_percent / 100.0;
                    ui.label("CPU");
                    ui.add_sized([105.0, 15.0], egui::ProgressBar::new(cpu).show_percentage());
                    ui.add_space(8.0);
                    ui.label("RAM");
                    ui.add_sized(
                        [105.0, 15.0],
                        egui::ProgressBar::new(snapshot.memory.used_fraction()).show_percentage(),
                    );
                } else {
                    ui.spinner();
                    ui.label("Reading /proc…");
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button("↻ Refresh")
                        .on_hover_text("Refresh now (F5)")
                        .clicked()
                    {
                        let _ = self.worker.commands.send(Command::Refresh);
                    }
                });
            });
            ui.add_space(8.0);
        });
    }

    fn nav(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(156.0)
            .show(ctx, |ui| {
                ui.add_space(16.0);
                ui.heading("WORKSPACE");
                ui.add_space(11.0);
                for (page, label) in [
                    (Page::Processes, "▦  Processes"),
                    (Page::Performance, "◴  Performance"),
                    (Page::Details, "▤  Details"),
                    (Page::Preferences, "⚙  Preferences"),
                ] {
                    if ui
                        .add_sized(
                            [144.0, 35.0],
                            egui::Button::new(label).selected(self.page == page),
                        )
                        .clicked()
                    {
                        self.page = page;
                    }
                    ui.add_space(3.0);
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        RichText::new(format!(
                            "v{} · {}",
                            env!("CARGO_PKG_VERSION"),
                            self.settings.theme.label()
                        ))
                        .small()
                        .weak(),
                    );
                    ui.label(RichText::new("Linux · X11 / Wayland").small().weak());
                });
            });
    }

    fn processes_page(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Processes");
            if let Some(snapshot) = &self.snapshot {
                ui.label(RichText::new(format!("{} running", snapshot.processes.len())).weak());
            }
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let search = ui.add_sized(
                [ui.available_width().min(320.0), 30.0],
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("Search name, command, PID, user…"),
            );
            if self.focus_search {
                search.request_focus();
                self.focus_search = false;
            }
            egui::ComboBox::from_id_salt("sort")
                .selected_text(format!("Sort: {}", self.settings.sort_by.label()))
                .show_ui(ui, |ui| {
                    for sort in SortBy::ALL {
                        if ui
                            .selectable_value(&mut self.settings.sort_by, sort, sort.label())
                            .changed()
                        {
                            self.save_preferences(ui.ctx());
                        }
                    }
                });
            if ui
                .button(if self.settings.descending {
                    "↓"
                } else {
                    "↑"
                })
                .on_hover_text("Reverse sort")
                .clicked()
            {
                self.settings.descending = !self.settings.descending;
                self.save_preferences(ui.ctx());
            }
            if ui
                .selectable_label(self.settings.tree_view, "Tree")
                .clicked()
            {
                self.settings.tree_view = !self.settings.tree_view;
                self.save_preferences(ui.ctx());
            }
        });
        ui.add_space(8.0);
        let rows = self.processes();
        if self.snapshot.is_none() {
            ui.spinner();
            ui.label("Loading process information…");
            return;
        }
        if rows.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.heading("No matching processes");
                ui.label("Try another search or change the filters in Preferences.");
                if ui.button("Clear search").clicked() {
                    self.query.clear();
                }
            });
            return;
        }
        let height = if self.settings.density == Density::Compact {
            26.0
        } else {
            34.0
        };
        let label_width = (ui.available_width() - 355.0).max(145.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [label_width, 21.0],
                egui::Label::new(RichText::new("PROCESS").strong().small()),
            );
            ui.add_sized(
                [58.0, 21.0],
                egui::Label::new(RichText::new("PID").strong().small()),
            );
            ui.add_sized(
                [68.0, 21.0],
                egui::Label::new(RichText::new("CPU %").strong().small()),
            );
            ui.add_sized(
                [91.0, 21.0],
                egui::Label::new(RichText::new("MEMORY").strong().small()),
            );
            ui.label(RichText::new("USER").strong().small());
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show_rows(ui, height, rows.len(), |ui, range| {
                for row in &rows[range] {
                    let p = &row.process;
                    let selected = self.selected == Some((p.pid, p.start_ticks));
                    ui.horizontal(|ui| {
                        let prefix = if row.depth == 0 {
                            String::new()
                        } else {
                            format!("{}↳ ", "  ".repeat(row.depth.min(10)))
                        };
                        let label = format!("{prefix}{}", p.name);
                        let button = egui::Button::new(label).selected(selected);
                        if ui
                            .add_sized([label_width, height - 3.0], button)
                            .on_hover_text(&p.command)
                            .clicked()
                        {
                            self.selected = Some((p.pid, p.start_ticks));
                        }
                        ui.add_sized([58.0, height], egui::Label::new(p.pid.to_string()));
                        ui.add_sized(
                            [68.0, height],
                            egui::Label::new(format!("{:.1}", p.cpu_percent)),
                        );
                        ui.add_sized(
                            [91.0, height],
                            egui::Label::new(format::bytes(p.resident_bytes)),
                        );
                        ui.label(
                            self.users
                                .get(&p.user_id)
                                .cloned()
                                .unwrap_or_else(|| p.user_id.to_string()),
                        );
                    });
                }
            });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{} shown", rows.len())).weak());
            if self.selected_process().is_some() {
                if ui.button("Details").clicked() {
                    self.page = Page::Details;
                }
                if ui.button("Terminate…").clicked() {
                    self.queue_action(SignalAction::Terminate);
                }
            } else {
                ui.label(RichText::new("Select a process to inspect or manage it").weak());
            }
        });
    }

    fn performance_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Performance");
        ui.label(RichText::new("Live system activity · last 120 samples").weak());
        ui.add_space(12.0);
        let Some(snapshot) = &self.snapshot else {
            ui.spinner();
            return;
        };
        let accent = accent_color(&self.settings);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.columns(2, |columns| {
                columns[0].heading(format!("CPU  {:.1}%", snapshot.cpu_percent));
                chart(&mut columns[0], &self.history.cpu, 100.0, accent);
                columns[0].label(format!("{} logical CPUs · Uptime {}", snapshot.logical_cpus, format::duration(snapshot.uptime_seconds)));
                columns[1].heading(format!("Memory  {:.1}%", snapshot.memory.used_fraction() * 100.0));
                chart(&mut columns[1], &self.history.memory, 100.0, Color32::from_rgb(82, 173, 155));
                columns[1].label(format!("{} used / {} total", format::bytes(snapshot.memory.used()), format::bytes(snapshot.memory.total)));
                columns[1].label(format!("Swap: {} / {}", format::bytes(snapshot.memory.swap_total.saturating_sub(snapshot.memory.swap_free)), format::bytes(snapshot.memory.swap_total)));
            });
            ui.add_space(20.0);
            ui.heading("CPU cores");
            egui::Grid::new("cores").num_columns(4).spacing([15.0, 7.0]).show(ui, |ui| {
                for (index, usage) in snapshot.cores_percent.iter().enumerate() {
                    ui.vertical(|ui| {
                        ui.label(format!("CPU {} · {:.0}%", index, usage));
                        ui.add_sized([ui.available_width().clamp(80.0, 160.0), 13.0], egui::ProgressBar::new(usage / 100.0));
                    });
                    if (index + 1) % 4 == 0 { ui.end_row(); }
                }
            });
            ui.add_space(18.0);
            ui.heading("I/O throughput");
            ui.columns(2, |columns| {
                columns[0].label(RichText::new("Network (all interfaces except loopback)").strong());
                columns[0].label(format!("↓  {}", format::rate(snapshot.rates.received_per_second)));
                columns[0].label(format!("↑  {}", format::rate(snapshot.rates.sent_per_second)));
                columns[1].label(RichText::new("Block devices").strong());
                columns[1].label(format!("Read  {}", format::rate(snapshot.rates.disk_read_per_second)));
                columns[1].label(format!("Write  {}", format::rate(snapshot.rates.disk_write_per_second)));
            });
            ui.add_space(12.0);
            ui.label(RichText::new("The first sample has no rate baseline. Network totals can include virtual interfaces; disk data represents block-layer traffic, not filesystem-only I/O.").small().weak());
        });
    }

    fn details_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Process details");
        ui.add_space(10.0);
        let Some(process) = self.selected_process().cloned() else {
            ui.label("Select a process from the Processes page to view its details.");
            if ui.button("Go to Processes").clicked() {
                self.page = Page::Processes;
            }
            return;
        };
        ui.heading(&process.name);
        ui.label(RichText::new(&process.command).weak());
        ui.add_space(8.0);
        egui::Grid::new("process-details")
            .num_columns(2)
            .spacing([30.0, 9.0])
            .show(ui, |ui| {
                for (name, value) in [
                    ("PID", process.pid.to_string()),
                    ("Parent PID", process.parent_pid.to_string()),
                    (
                        "User",
                        self.users
                            .get(&process.user_id)
                            .cloned()
                            .unwrap_or_else(|| process.user_id.to_string()),
                    ),
                    ("State", state_name(process.state).to_owned()),
                    ("CPU", format!("{:.1}%", process.cpu_percent)),
                    ("Resident memory", format::bytes(process.resident_bytes)),
                    ("Virtual memory", format::bytes(process.virtual_bytes)),
                    ("Threads", process.threads.to_string()),
                    ("Disk read (cumulative)", format::bytes(process.read_bytes)),
                    (
                        "Disk written (cumulative)",
                        format::bytes(process.written_bytes),
                    ),
                ] {
                    ui.label(RichText::new(name).weak());
                    ui.label(value);
                    ui.end_row();
                }
            });
        ui.add_space(18.0);
        ui.label(RichText::new("PROCESS ACTIONS").strong());
        ui.label(
            RichText::new(
                "Signals are sent as your current user. Root-owned processes may reject them.",
            )
            .small()
            .weak(),
        );
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for action in [
                SignalAction::Terminate,
                SignalAction::Stop,
                SignalAction::Continue,
                SignalAction::ForceKill,
            ] {
                if ui.button(action.label()).clicked() {
                    self.queue_action(action);
                }
            }
        });
    }

    fn preferences_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("Preferences");
        ui.label(
            RichText::new("Local to Taskotto; no root service or session dependencies").weak(),
        );
        ui.add_space(16.0);
        let mut changed = false;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Appearance");
            ui.horizontal(|ui| {
                ui.label("Rice profile");
                egui::ComboBox::from_id_salt("theme-choice")
                    .selected_text(self.settings.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            changed |= ui.selectable_value(&mut self.settings.theme, theme, theme.label()).changed();
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Density");
                changed |= ui.radio_value(&mut self.settings.density, Density::Compact, "Compact").changed();
                changed |= ui.radio_value(&mut self.settings.density, Density::Comfortable, "Comfortable").changed();
            });
            ui.horizontal(|ui| {
                ui.label("Accent color (optional #RRGGBB)");
                let response = ui.add(egui::TextEdit::singleline(&mut self.theme_accent_input).desired_width(130.0));
                if response.lost_focus() {
                    if self.theme_accent_input.is_empty() || is_hex_color(&self.theme_accent_input) {
                        self.settings.accent_hex = if self.theme_accent_input.is_empty() { None } else { Some(self.theme_accent_input.clone()) };
                        changed = true;
                    } else {
                        self.notification = Some("Accent must be a #RRGGBB color, e.g. #4D8CFF".to_owned());
                    }
                }
            });
            ui.add_space(18.0);
            ui.heading("Monitoring");
            changed |= ui.add(egui::Slider::new(&mut self.settings.refresh_ms, 500..=10_000).text("Refresh interval (ms)")).changed();
            changed |= ui.checkbox(&mut self.settings.only_my_processes, "Show only my processes").changed();
            changed |= ui.checkbox(&mut self.settings.show_kernel_threads, "Show kernel threads").changed();
            ui.add_space(12.0);
            ui.label(RichText::new("The UI samples only while the application is open. Longer intervals reduce idle CPU use.").small().weak());
            ui.add_space(18.0);
            if ui.button("Reset to defaults").clicked() {
                self.settings = Settings::default();
                self.theme_accent_input.clear();
                changed = true;
            }
            ui.add_space(10.0);
            if let Ok(path) = crate::config::config_file() {
                ui.label(RichText::new(format!("Configuration: {}", path.display())).small().weak());
            }
        });
        if changed {
            self.save_preferences(ui.ctx());
        }
    }

    fn confirmation_dialog(&mut self, ctx: &egui::Context) {
        let Some(pending) = &self.pending else { return };
        let mut cancel = false;
        let mut confirm = false;
        egui::Window::new("Confirm process action")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(350.0);
                ui.label(format!(
                    "{} — PID {}",
                    pending.target.name, pending.target.pid
                ));
                ui.label(format!("Action: {}", pending.action.label()));
                ui.add_space(8.0);
                if pending.action == SignalAction::ForceKill {
                    ui.colored_label(
                        Color32::from_rgb(220, 96, 86),
                        "SIGKILL prevents the process from cleaning up or saving data.",
                    );
                } else if pending.action == SignalAction::Terminate {
                    ui.label("SIGTERM gives the application a chance to exit gracefully.");
                }
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    if ui.button("Confirm action").clicked() {
                        confirm = true;
                    }
                });
            });
        if confirm {
            if let Some(pending) = self.pending.take() {
                match send_signal(&pending.target, pending.action) {
                    Ok(()) => {
                        self.notification = Some(format!(
                            "{} sent to PID {}",
                            pending.action.label(),
                            pending.target.pid
                        ))
                    }
                    Err(err) => {
                        self.notification = Some(format!(
                            "Action failed for PID {}: {err}",
                            pending.target.pid
                        ))
                    }
                }
                let _ = self.worker.commands.send(Command::Refresh);
            }
        } else if cancel || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.pending = None;
        }
    }
}

impl eframe::App for TaskottoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.accept_snapshots();
        ctx.input(|input| {
            if input.key_pressed(egui::Key::F5) {
                let _ = self.worker.commands.send(Command::Refresh);
            }
            if input.modifiers.command && input.key_pressed(egui::Key::F) {
                self.page = Page::Processes;
                self.focus_search = true;
            }
            if input.key_pressed(egui::Key::Escape) && self.pending.is_none() {
                self.query.clear();
            }
        });
        self.topbar(ctx);
        self.nav(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(14.0);
            if let Some(message) = self.notification.clone() {
                ui.horizontal_wrapped(|ui| {
                    ui.colored_label(Color32::from_rgb(226, 147, 74), message);
                    if ui.small_button("Dismiss ×").clicked() {
                        self.notification = None;
                    }
                });
                ui.separator();
            }
            match self.page {
                Page::Processes => self.processes_page(ui),
                Page::Performance => self.performance_page(ui),
                Page::Details => self.details_page(ui),
                Page::Preferences => self.preferences_page(ui),
            }
        });
        self.confirmation_dialog(ctx);
        // The sampler wakes the GUI when new data exists. No idle render timer.
    }
}

fn append_tree(
    pid: i32,
    depth: usize,
    processes: &HashMap<i32, Process>,
    children: &HashMap<i32, Vec<i32>>,
    visited: &mut HashSet<i32>,
    result: &mut Vec<ProcessRow>,
) {
    if !visited.insert(pid) {
        return;
    }
    if let Some(process) = processes.get(&pid) {
        result.push(ProcessRow {
            process: process.clone(),
            depth,
        });
        if let Some(child_ids) = children.get(&pid) {
            for child in child_ids {
                append_tree(*child, depth + 1, processes, children, visited, result);
            }
        }
    }
}

fn is_kernel_thread(process: &Process) -> bool {
    process.pid == 2 || process.parent_pid == 2
}

fn state_name(state: char) -> &'static str {
    match state {
        'R' => "Running",
        'S' => "Sleeping",
        'D' => "Disk sleep",
        'T' | 't' => "Stopped",
        'Z' => "Zombie",
        'I' => "Idle",
        'X' | 'x' => "Dead",
        _ => "Other",
    }
}

fn load_users() -> HashMap<u32, String> {
    let mut users = HashMap::new();
    if let Ok(passwd) = std::fs::read_to_string("/etc/passwd") {
        for line in passwd.lines() {
            let columns = line.split(':').collect::<Vec<_>>();
            if columns.len() >= 3
                && let Ok(uid) = columns[2].parse()
            {
                users.insert(uid, columns[0].to_owned());
            }
        }
    }
    users
}

fn chart(ui: &mut egui::Ui, values: &VecDeque<f32>, max_value: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
    if values.len() < 2 {
        return;
    }
    let points = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            egui::pos2(
                rect.left() + rect.width() * index as f32 / 119.0,
                rect.bottom() - rect.height() * (*value / max_value).clamp(0.0, 1.0),
            )
        })
        .collect::<Vec<_>>();
    ui.painter()
        .add(egui::Shape::line(points, Stroke::new(2.0_f32, color)));
}

fn accent_color(settings: &Settings) -> Color32 {
    if let Some(hex) = settings
        .accent_hex
        .as_deref()
        .filter(|color| is_hex_color(color))
        && let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[1..3], 16),
            u8::from_str_radix(&hex[3..5], 16),
            u8::from_str_radix(&hex[5..7], 16),
        )
    {
        return Color32::from_rgb(r, g, b);
    }
    match settings.theme {
        Theme::System => Color32::from_rgb(130, 180, 255),
        Theme::Windows98 => Color32::from_rgb(0, 128, 128),
        Theme::XpLuna => Color32::from_rgb(48, 105, 215),
        Theme::Windows7 => Color32::from_rgb(43, 138, 195),
        Theme::Windows10 => Color32::from_rgb(0, 120, 215),
        Theme::FrutigerAero => Color32::from_rgb(37, 149, 114),
        Theme::Chromeos => Color32::from_rgb(68, 125, 220),
        Theme::Macos => Color32::from_rgb(24, 118, 238),
    }
}

fn apply_theme(ctx: &egui::Context, settings: &Settings) {
    let light = matches!(
        settings.theme,
        Theme::Windows98
            | Theme::XpLuna
            | Theme::Windows7
            | Theme::FrutigerAero
            | Theme::Chromeos
            | Theme::Macos
    );
    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    let accent = accent_color(settings);
    visuals.selection.bg_fill = accent;
    visuals.hyperlink_color = accent;
    visuals.widgets.active.bg_fill = accent;
    if settings.theme == Theme::Windows98 {
        visuals.panel_fill = Color32::from_rgb(196, 196, 196);
        visuals.window_fill = visuals.panel_fill;
        visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
    }
    if settings.theme == Theme::XpLuna {
        visuals.panel_fill = Color32::from_rgb(237, 235, 222);
    }
    if settings.theme == Theme::FrutigerAero {
        visuals.panel_fill = Color32::from_rgb(226, 243, 238);
    }
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = if settings.density == Density::Compact {
        Vec2::new(6.0, 3.0)
    } else {
        Vec2::new(8.0, 6.0)
    };
    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_user_data_does_not_crash() {
        assert_eq!(state_name('Z'), "Zombie");
        assert_eq!(state_name('?'), "Other");
    }

    #[test]
    fn history_stays_bounded() {
        let mut history = History::default();
        let snapshot = Snapshot {
            cpu_percent: 33.0,
            cores_percent: vec![],
            memory: Default::default(),
            rates: Default::default(),
            uptime_seconds: 0.0,
            processes: vec![],
            logical_cpus: 1,
        };
        for _ in 0..200 {
            history.record(&snapshot);
        }
        assert_eq!(history.cpu.len(), 120);
    }

    #[test]
    fn tree_walker_handles_parent_cycles_without_duplicate_rows() {
        let mut processes = HashMap::new();
        processes.insert(
            11,
            Process {
                pid: 11,
                parent_pid: 12,
                ..Process::default()
            },
        );
        processes.insert(
            12,
            Process {
                pid: 12,
                parent_pid: 11,
                ..Process::default()
            },
        );
        let children = HashMap::from([(11, vec![12]), (12, vec![11])]);
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        append_tree(11, 0, &processes, &children, &mut visited, &mut result);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].depth, 0);
        assert_eq!(result[1].depth, 1);
    }

    #[test]
    fn theme_application_supports_light_and_dark() {
        let ctx = egui::Context::default();
        let mut settings = Settings::default();
        apply_theme(&ctx, &settings);
        assert!(ctx.style().visuals.dark_mode);
        settings.theme = Theme::Windows98;
        apply_theme(&ctx, &settings);
        assert!(!ctx.style().visuals.dark_mode);
    }

    #[test]
    fn pages_can_layout_without_a_display_server() {
        let ctx = egui::Context::default();
        let settings = Settings::default();
        let mut app = TaskottoApp {
            worker: Worker::start(settings.refresh_ms, ctx.clone()),
            settings,
            snapshot: Some(Snapshot {
                cpu_percent: 12.5,
                cores_percent: vec![25.0, 0.0],
                memory: crate::monitor::Memory {
                    total: 1024,
                    available: 512,
                    swap_total: 0,
                    swap_free: 0,
                },
                rates: Default::default(),
                uptime_seconds: 100.0,
                processes: vec![Process {
                    pid: 456,
                    name: "test-app".into(),
                    command: "test-app --debug".into(),
                    resident_bytes: 4096,
                    ..Process::default()
                }],
                logical_cpus: 2,
            }),
            history: History::default(),
            page: Page::Processes,
            query: String::new(),
            focus_search: false,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
        };
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                Vec2::new(1100.0, 720.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                app.processes_page(ui);
                app.performance_page(ui);
                app.details_page(ui);
                app.preferences_page(ui);
            });
        });
    }
}
