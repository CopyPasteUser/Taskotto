use crate::{
    actions::{SignalAction, send_signal},
    config::{Density, LayoutMode, Settings, SortBy, Theme, is_hex_color},
    format,
    monitor::{Monitor, Process, Snapshot},
    ricepack::{self, LibraryEntry, RicePack},
    theme::ThemeStyle,
};
use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender, TryRecvError, TrySendError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Page {
    Applications,
    Processes,
    Performance,
    AppHistory,
    Startup,
    Networking,
    Users,
    Details,
    Services,
    MacEnergy,
    MacDisk,
    Preferences,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PerformanceFocus {
    #[default]
    Cpu,
    Memory,
    Disk,
    Network,
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
        // A minimized window may stop consuming frames. Never retain an
        // unlimited number of full process snapshots in that case.
        let (tx, snapshots) = mpsc::sync_channel(1);
        let thread = thread::spawn(move || {
            let mut monitor = Monitor::default();
            let mut interval = interval;
            loop {
                let result = monitor.sample();
                match tx.try_send(result) {
                    Ok(()) => ctx.request_repaint(),
                    Err(TrySendError::Full(_)) => {}
                    Err(TrySendError::Disconnected(_)) => break,
                }
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
    cores: Vec<VecDeque<f32>>,
    memory: VecDeque<f32>,
    received: VecDeque<f32>,
    sent: VecDeque<f32>,
    disk_read: VecDeque<f32>,
    disk_write: VecDeque<f32>,
    process_follow: Option<ProcessFollow>,
    pss_inspection: Option<PssInspection>,
    cgroup_inspection: Option<CgroupInspection>,
}

struct CgroupInspection {
    target: (i32, u64),
    state: CgroupState,
}

enum CgroupState {
    Loading(Receiver<Result<crate::monitor::cgroup::CgroupReport, String>>),
    Ready(Result<crate::monitor::cgroup::CgroupReport, String>),
}

struct PssInspection {
    target: (i32, u64),
    state: PssState,
}

enum PssState {
    Loading(Receiver<Result<crate::monitor::pss::TreePss, String>>),
    Ready(Result<crate::monitor::pss::TreePss, String>),
}

/// Explicitly followed process only; avoids keeping an unbounded global PID
/// database and never mistakes a recycled PID for the original process.
struct ProcessFollow {
    pid: i32,
    start_ticks: u64,
    name: String,
    cpu: VecDeque<f32>,
    rss_mib: VecDeque<f32>,
    missed: usize,
    active: bool,
}

impl ProcessFollow {
    const LIMIT: usize = 120;

    fn new(process: &Process) -> Self {
        Self {
            pid: process.pid,
            start_ticks: process.start_ticks,
            name: process.name.clone(),
            cpu: VecDeque::new(),
            rss_mib: VecDeque::new(),
            missed: 0,
            active: true,
        }
    }

    fn record(&mut self, snapshot: &Snapshot) {
        if !self.active {
            return;
        }
        let Some(process) = snapshot
            .processes
            .iter()
            .find(|p| p.pid == self.pid && p.start_ticks == self.start_ticks)
        else {
            self.missed = self.missed.saturating_add(1);
            if self.missed >= 3 {
                self.active = false;
            }
            return;
        };
        self.missed = 0;
        for (history, value) in [
            (&mut self.cpu, process.cpu_percent),
            (
                &mut self.rss_mib,
                process.resident_bytes as f32 / (1024.0 * 1024.0),
            ),
        ] {
            if history.len() == Self::LIMIT {
                history.pop_front();
            }
            history.push_back(value);
        }
    }
}

impl History {
    fn record(&mut self, snapshot: &Snapshot) {
        if let Some(follow) = &mut self.process_follow {
            follow.record(snapshot);
        }
        self.cores
            .resize_with(snapshot.cores_percent.len(), VecDeque::new);
        self.cores.truncate(snapshot.cores_percent.len());
        for (history, &load) in self.cores.iter_mut().zip(&snapshot.cores_percent) {
            if history.len() == 120 {
                history.pop_front();
            }
            history.push_back(load);
        }
        for (queue, value) in [
            (&mut self.cpu, snapshot.cpu_percent),
            (&mut self.memory, snapshot.memory.used_fraction() * 100.0),
            (
                &mut self.received,
                snapshot.rates.received_per_second as f32,
            ),
            (&mut self.sent, snapshot.rates.sent_per_second as f32),
            (
                &mut self.disk_read,
                snapshot.rates.disk_read_per_second as f32,
            ),
            (
                &mut self.disk_write,
                snapshot.rates.disk_write_per_second as f32,
            ),
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

#[derive(Clone)]
struct RicePackWatcher {
    path: PathBuf,
    applied: RicePack,
    forced_theme: Option<Theme>,
    next_check: Instant,
    last_error: Option<String>,
}

#[derive(Default)]
struct RicePackLibraryUi {
    source_path: String,
    export_path: String,
    entries: Vec<LibraryEntry>,
    loaded: bool,
    preview: Option<RicePackPreview>,
    editor_layout: Option<crate::layout::LayoutSpec>,
    editor_page: usize,
    editor_compact: bool,
    editor_error: Option<String>,
}

struct RicePackPreview {
    settings: Settings,
    watcher: Option<RicePackWatcher>,
    path: PathBuf,
    pack: RicePack,
}

pub struct TaskottoApp {
    settings: Settings,
    worker: Worker,
    snapshot: Option<Snapshot>,
    history: History,
    page: Page,
    performance_focus: PerformanceFocus,
    query: String,
    focus_search: bool,
    /// Session-only order. PID and start tick together prevent PID-reuse aliasing.
    frozen_sort_order: Option<HashMap<(i32, u64), usize>>,
    selected: Option<(i32, u64)>,
    pending: Option<PendingAction>,
    notification: Option<String>,
    theme_accent_input: String,
    theme_surface_input: String,
    theme_text_input: String,
    my_uid: u32,
    users: HashMap<u32, String>,
    system_pages: rices::SystemPages,
    new_task: NewTaskDialog,
    mac_info_open: bool,
    ricepack_watcher: Option<RicePackWatcher>,
    ricepack_library: RicePackLibraryUi,
}

#[derive(Default)]
struct NewTaskDialog {
    open: bool,
    command: String,
    error: Option<String>,
}

fn launch_user_task(command: &str) -> io::Result<()> {
    if command.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Enter a command first",
        ));
    }
    // Explicitly user-entered shell input, never an untrusted .desktop Exec
    // string or a synthesized command. Inherits only normal user permissions.
    let mut child = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    // A dropped Child is not reaped on Unix; wait without blocking the GUI.
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

impl TaskottoApp {
    pub fn new(
        context: &eframe::CreationContext<'_>,
        settings: Settings,
        warning: Option<String>,
    ) -> Self {
        apply_theme(&context.egui_ctx, &settings);
        let accent_input = settings.accent_hex.clone().unwrap_or_default();
        let surface_input = settings.surface_hex.clone().unwrap_or_default();
        let text_input = settings.text_hex.clone().unwrap_or_default();
        let initial_page = if settings.theme == Theme::Windows98 {
            Page::Applications
        } else {
            Page::Processes
        };
        Self {
            worker: Worker::start(settings.refresh_ms, context.egui_ctx.clone()),
            settings,
            snapshot: None,
            history: History::default(),
            page: initial_page,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: warning,
            theme_accent_input: accent_input,
            theme_surface_input: surface_input,
            theme_text_input: text_input,
            my_uid: unsafe { libc::geteuid() },
            users: load_users(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        }
    }

    /// Watch only an explicitly launched RicePack. Regular Taskotto sessions
    /// retain their sampler-driven, timer-free idle behavior.
    pub fn watch_ricepack(
        mut self,
        path: PathBuf,
        applied: RicePack,
        forced_theme: Option<Theme>,
    ) -> Self {
        self.ricepack_watcher = Some(RicePackWatcher {
            path,
            applied,
            forced_theme,
            next_check: Instant::now() + Duration::from_secs(2),
            last_error: None,
        });
        self
    }

    fn maybe_reload_ricepack(&mut self, ctx: &egui::Context) {
        let Some(watcher) = self.ricepack_watcher.as_mut() else {
            return;
        };
        let now = Instant::now();
        if now < watcher.next_check {
            ctx.request_repaint_after(watcher.next_check.duration_since(now));
            return;
        }
        watcher.next_check = now + Duration::from_secs(2);
        ctx.request_repaint_after(Duration::from_secs(2));
        let pack = match RicePack::load(&watcher.path) {
            Ok(pack) => pack,
            Err(err) => {
                let message =
                    format!("RicePack reload failed (keeping the last valid design): {err}");
                if watcher.last_error.as_deref() != Some(message.as_str()) {
                    self.notification = Some(message.clone());
                    watcher.last_error = Some(message);
                }
                return;
            }
        };
        watcher.last_error = None;
        if pack == watcher.applied {
            return;
        }
        let old_theme = self.settings.theme;
        self.settings.reset_ricepack_appearance();
        pack.apply(&mut self.settings);
        if let Some(theme) = watcher.forced_theme {
            self.settings.theme = theme;
        }
        self.theme_accent_input = self.settings.accent_hex.clone().unwrap_or_default();
        self.theme_surface_input = self.settings.surface_hex.clone().unwrap_or_default();
        self.theme_text_input = self.settings.text_hex.clone().unwrap_or_default();
        if self.settings.theme != old_theme {
            if self.settings.theme == Theme::Windows98 {
                self.page = Page::Applications;
            } else if self.page == Page::Applications {
                self.page = Page::Processes;
            }
        }
        apply_theme(ctx, &self.settings);
        self.notification = Some(format!("Reloaded RicePack: {}", pack.name));
        watcher.applied = pack;
    }

    fn sync_ricepack_color_inputs(&mut self) {
        self.theme_accent_input = self.settings.accent_hex.clone().unwrap_or_default();
        self.theme_surface_input = self.settings.surface_hex.clone().unwrap_or_default();
        self.theme_text_input = self.settings.text_hex.clone().unwrap_or_default();
    }

    fn refresh_ricepack_library(&mut self) {
        let result = ricepack::library_dir().and_then(|dir| ricepack::discover_in(&dir));
        match result {
            Ok(entries) => {
                self.ricepack_library.entries = entries;
                self.ricepack_library.loaded = true;
            }
            Err(error) => {
                self.notification = Some(format!("Cannot scan RicePack library: {error}"))
            }
        }
    }

    /// A preview is reversible even if the user originally started Taskotto
    /// with an active RicePack. The original file watcher is paused during it.
    fn preview_ricepack(&mut self, ctx: &egui::Context) {
        let path = PathBuf::from(self.ricepack_library.source_path.trim());
        let pack = match RicePack::load(&path) {
            Ok(pack) => pack,
            Err(error) => {
                self.notification = Some(format!("Cannot preview RicePack: {error}"));
                return;
            }
        };
        self.restore_ricepack_preview(ctx);
        self.ricepack_library.preview = Some(RicePackPreview {
            settings: self.settings.clone(),
            watcher: self.ricepack_watcher.take(),
            path,
            pack: pack.clone(),
        });
        self.settings.reset_ricepack_appearance();
        self.settings.launch_ricepack_original = None;
        self.settings.launch_ricepack_name = None;
        pack.apply(&mut self.settings);
        self.sync_page_for_theme();
        self.sync_ricepack_color_inputs();
        apply_theme(ctx, &self.settings);
        self.notification = Some(format!(
            "Previewing {} — choose Revert or Keep for session",
            pack.name
        ));
    }

    fn restore_ricepack_preview(&mut self, ctx: &egui::Context) {
        let Some(previous) = self.ricepack_library.preview.take() else {
            return;
        };
        // Monitoring preferences may have been changed and saved while the
        // visual preview was active. Preserve them when restoring its palette.
        let current = &self.settings;
        let mut settings = previous.settings;
        settings.refresh_ms = current.refresh_ms;
        settings.sort_by = current.sort_by;
        settings.descending = current.descending;
        settings.tree_view = current.tree_view;
        settings.only_my_processes = current.only_my_processes;
        settings.show_kernel_threads = current.show_kernel_threads;
        settings.show_io_column = current.show_io_column;
        settings.show_user_column = current.show_user_column;
        self.settings = settings;
        self.ricepack_watcher = previous.watcher;
        self.sync_page_for_theme();
        self.sync_ricepack_color_inputs();
        apply_theme(ctx, &self.settings);
        self.notification = Some("RicePack preview reverted; previous appearance restored".into());
    }

    fn keep_ricepack_preview(&mut self) {
        let Some(preview) = self.ricepack_library.preview.take() else {
            return;
        };
        let forced_theme = preview.watcher.and_then(|watcher| watcher.forced_theme);
        if let Some(theme) = forced_theme {
            self.settings.theme = theme;
        }
        self.ricepack_watcher = Some(RicePackWatcher {
            path: preview.path,
            applied: preview.pack,
            forced_theme,
            next_check: Instant::now() + Duration::from_secs(2),
            last_error: None,
        });
        self.notification = Some("RicePack active for this session; saved design unchanged".into());
    }

    fn sync_page_for_theme(&mut self) {
        if self.settings.theme == Theme::Windows98 {
            self.page = Page::Applications;
        } else if self.page == Page::Applications {
            self.page = Page::Processes;
        }
    }

    fn ricepack_library_controls(&mut self, ui: &mut egui::Ui, xp: bool) {
        if !self.ricepack_library.loaded {
            self.refresh_ricepack_library();
        }
        preference_section(ui, "RicePack library", xp);
        if let Ok(path) = ricepack::library_dir() {
            ui.label(
                RichText::new(format!("Local designs: {}", path.display()))
                    .small()
                    .weak(),
            );
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("Refresh library").clicked() {
                self.refresh_ricepack_library();
            }
            ui.label(format!("{} designs", self.ricepack_library.entries.len()));
        });
        egui::ComboBox::from_id_salt("ricepack-library")
            .selected_text(
                self.ricepack_library
                    .entries
                    .iter()
                    .find(|entry| entry.path.to_string_lossy() == self.ricepack_library.source_path)
                    .map(|entry| entry.name.as_str())
                    .unwrap_or("Choose a local RicePack…"),
            )
            .show_ui(ui, |ui| {
                for entry in &self.ricepack_library.entries {
                    let label = if entry.error.is_some() {
                        format!("⚠ {} (invalid)", entry.name)
                    } else {
                        entry.name.clone()
                    };
                    if ui.selectable_label(false, label).clicked() {
                        self.ricepack_library.source_path = entry.path.display().to_string();
                    }
                }
            });
        ui.horizontal_wrapped(|ui| {
            ui.label("Pack file");
            ui.add(
                egui::TextEdit::singleline(&mut self.ricepack_library.source_path)
                    .desired_width(380.0)
                    .hint_text("/absolute/path/to/design.toml"),
            );
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Preview design").clicked() {
                self.preview_ricepack(ui.ctx());
            }
            if ui.button("Import to library").clicked() {
                let source = PathBuf::from(self.ricepack_library.source_path.trim());
                let result = RicePack::load(&source).and_then(|pack| {
                    let directory = ricepack::library_dir()?;
                    ricepack::import_into(&pack, &directory)
                });
                match result {
                    Ok(path) => {
                        self.ricepack_library.source_path = path.display().to_string();
                        self.refresh_ricepack_library();
                        self.notification = Some(format!("Imported design: {}", path.display()));
                    }
                    Err(error) => {
                        self.notification = Some(format!("RicePack import failed: {error}"))
                    }
                }
            }
        });
        if let Some(preview) = &self.ricepack_library.preview {
            ui.label(
                RichText::new(format!(
                    "Preview: {} · changes are temporary",
                    preview.pack.name
                ))
                .strong(),
            );
            ui.horizontal(|ui| {
                if ui.button("Keep for session").clicked() {
                    self.keep_ricepack_preview();
                    apply_theme(ui.ctx(), &self.settings);
                }
                if ui.button("Revert preview").clicked() {
                    self.restore_ricepack_preview(ui.ctx());
                }
            });
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.label("Export to");
            ui.add(
                egui::TextEdit::singleline(&mut self.ricepack_library.export_path)
                    .desired_width(320.0)
                    .hint_text("/absolute/path/to/new-design.toml"),
            );
            if ui.button("Export current appearance").clicked() {
                let path = PathBuf::from(self.ricepack_library.export_path.trim());
                match RicePack::from_settings(&self.settings).write_new(&path) {
                    Ok(()) => {
                        self.notification = Some(format!("Exported design: {}", path.display()))
                    }
                    Err(error) => {
                        self.notification = Some(format!("RicePack export failed: {error}"))
                    }
                }
            }
        });
        ui.add_space(12.0);
        layout_editor::draw(self, ui, xp);
        ui.label(RichText::new("Preview changes only this window. Import and export never overwrite files. RicePacks cannot run commands or change monitoring permissions.").small().weak());
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
        if let Some(inspection) = &mut self.history.pss_inspection {
            let result = if let PssState::Loading(rx) = &inspection.state {
                match rx.try_recv() {
                    Ok(result) => Some(result),
                    Err(TryRecvError::Disconnected) => {
                        Some(Err("PSS inspection worker exited unexpectedly".into()))
                    }
                    Err(TryRecvError::Empty) => None,
                }
            } else {
                None
            };
            if let Some(result) = result {
                inspection.state = PssState::Ready(result);
            }
        }
        if let Some(inspection) = &mut self.history.cgroup_inspection {
            let result = if let CgroupState::Loading(rx) = &inspection.state {
                match rx.try_recv() {
                    Ok(result) => Some(result),
                    Err(TryRecvError::Disconnected) => {
                        Some(Err("Cgroup inspection worker exited unexpectedly".into()))
                    }
                    Err(TryRecvError::Empty) => None,
                }
            } else {
                None
            };
            if let Some(result) = result {
                inspection.state = CgroupState::Ready(result);
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
            if !can_signal_process(&process) {
                self.notification = Some("Taskotto refuses to signal PID 1 or itself".to_owned());
                return;
            }
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
            .filter(|p| self.process_matches_filters(p, &query))
            .cloned()
            .collect();
        matches.sort_by(|a, b| self.compare_process_order(a, b));
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

    fn process_matches_filters(&self, process: &Process, lowercase_query: &str) -> bool {
        (self.settings.show_kernel_threads || !is_kernel_thread(process))
            && (!self.settings.only_my_processes || process.user_id == self.my_uid)
            && (lowercase_query.is_empty()
                || process.name.to_lowercase().contains(lowercase_query)
                || process.command.to_lowercase().contains(lowercase_query)
                || process.pid.to_string().contains(lowercase_query)
                || self
                    .users
                    .get(&process.user_id)
                    .is_some_and(|name| name.to_lowercase().contains(lowercase_query)))
    }

    fn sort_pids(&self, processes: Vec<&Process>) -> Vec<i32> {
        let mut rows = processes;
        rows.sort_by(|a, b| self.compare_process_order(a, b));
        rows.into_iter().map(|p| p.pid).collect()
    }

    fn compare_process_order(&self, a: &Process, b: &Process) -> std::cmp::Ordering {
        compare_process_order(
            a,
            b,
            self.settings.sort_by,
            self.settings.descending,
            self.frozen_sort_order.as_ref(),
        )
    }

    fn set_sort_frozen(&mut self, frozen: bool) {
        if !frozen {
            self.frozen_sort_order = None;
        } else if self.frozen_sort_order.is_none() {
            if self.snapshot.is_none() {
                self.notification =
                    Some("Wait for the first process sample to freeze order".into());
                return;
            }
            let order = self
                .processes()
                .iter()
                .enumerate()
                .map(|(index, row)| ((row.process.pid, row.process.start_ticks), index))
                .collect();
            self.frozen_sort_order = Some(order);
        }
    }

    fn topbar(&mut self, ctx: &egui::Context) {
        if rices::supports(self.settings.theme) {
            rices::topbar(self, ctx);
            return;
        }
        egui::TopBottomPanel::top("topbar").show(ctx, |ui| {
            ui.add_space(9.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Taskotto").strong().size(21.0));
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
        if self.settings.theme == Theme::Chromeos {
            // ChromeOS presents one primary task table. Additional navigation
            // is opt-in, with all views available from the toolbar menu.
            match self.settings.layout_mode {
                LayoutMode::Auto => {}
                LayoutMode::TopTabs => rices::navigation(self, ctx),
                LayoutMode::Sidebar => rices::vertical_navigation(self, ctx),
            }
            return;
        }
        if self.settings.theme == Theme::FrutigerAero
            && self.settings.layout_mode.resolve(self.settings.theme) == LayoutMode::Sidebar
        {
            rices::vertical_navigation(self, ctx);
            return;
        }
        if self.settings.theme == Theme::Macos
            && self.settings.layout_mode.resolve(self.settings.theme) == LayoutMode::Sidebar
        {
            rices::vertical_navigation(self, ctx);
            return;
        }
        if self.settings.theme == Theme::Windows10
            && self.settings.layout_mode.resolve(self.settings.theme) == LayoutMode::Sidebar
        {
            rices::vertical_navigation(self, ctx);
            return;
        }
        if rices::supports(self.settings.theme)
            && self.settings.layout_mode.resolve(self.settings.theme) == LayoutMode::TopTabs
        {
            rices::navigation(self, ctx);
            return;
        }
        if self.settings.layout_mode.resolve(self.settings.theme) == LayoutMode::TopTabs {
            egui::TopBottomPanel::top("navigation-tabs").show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (page, label) in [
                        (Page::Applications, "Applications"),
                        (Page::Processes, "Processes"),
                        (Page::Performance, "Performance"),
                        (Page::Networking, "Network"),
                        (Page::Users, "Users"),
                        (Page::Details, "Details"),
                        (Page::Preferences, "Preferences"),
                    ] {
                        if ui.selectable_label(self.page == page, label).clicked() {
                            self.page = page;
                        }
                    }
                });
            });
            return;
        }
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(156.0)
            .show(ctx, |ui| {
                ui.add_space(16.0);
                ui.heading("WORKSPACE");
                ui.add_space(11.0);
                for (page, label) in [
                    (Page::Applications, "Applications"),
                    (Page::Processes, "Processes"),
                    (Page::Performance, "Performance"),
                    (Page::Networking, "Network"),
                    (Page::Users, "Users"),
                    (Page::Details, "Details"),
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
        if self.settings.custom_layout.is_some() {
            rices::declarative_processes(self, ui);
            return;
        }
        if rices::supports(self.settings.theme) {
            rices::processes(self, ui);
            return;
        }
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
                    "DESC"
                } else {
                    "ASC"
                })
                .on_hover_text("Reverse sort")
                .clicked()
            {
                self.settings.descending = !self.settings.descending;
                self.save_preferences(ui.ctx());
            }
            if ui
                .selectable_label(self.frozen_sort_order.is_some(), "Freeze order")
                .on_hover_text("Keep the current visible order as CPU and memory values change; Ctrl+Shift+Space")
                .clicked()
            {
                self.set_sort_frozen(self.frozen_sort_order.is_none());
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
        let default_height = if self.settings.density == Density::Compact {
            26.0
        } else {
            34.0
        };
        let height = self.settings.table_row_height.unwrap_or(default_height);
        let show_io = self.settings.show_io_column && ui.available_width() >= 780.0;
        let show_user = self.settings.show_user_column && ui.available_width() >= 560.0;
        let reserved = 58.0
            + 68.0
            + 91.0
            + if show_io { 104.0 } else { 0.0 }
            + if show_user { 110.0 } else { 0.0 }
            + 28.0;
        let label_width = (ui.available_width() - reserved).max(135.0);
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
            if show_io {
                ui.add_sized(
                    [104.0, 21.0],
                    egui::Label::new(RichText::new("I/O RATE").strong().small()),
                );
            }
            if show_user {
                ui.label(RichText::new("USER").strong().small());
            }
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
                        if show_io {
                            ui.add_sized(
                                [104.0, height],
                                egui::Label::new(if p.io_accessible {
                                    format::rate(process_io_rate(p))
                                } else {
                                    "Unavailable".to_owned()
                                }),
                            );
                        }
                        if show_user {
                            ui.label(
                                self.users
                                    .get(&p.user_id)
                                    .cloned()
                                    .unwrap_or_else(|| p.user_id.to_string()),
                            );
                        }
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
                if ui
                    .add_enabled(
                        self.selected_process().is_some_and(can_signal_process),
                        egui::Button::new("Terminate…"),
                    )
                    .clicked()
                {
                    self.queue_action(SignalAction::Terminate);
                }
            } else {
                ui.label(RichText::new("Select a process to inspect or manage it").weak());
            }
        });
    }

    fn performance_page(&mut self, ui: &mut egui::Ui) {
        if rices::supports(self.settings.theme) {
            rices::performance(self, ui);
            return;
        }
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
                        ui.label(format!("CPU {index} · {usage:.0}%"));
                        ui.add_sized([ui.available_width().clamp(80.0, 160.0), 13.0], egui::ProgressBar::new(usage / 100.0));
                    });
                    if (index + 1) % 4 == 0 { ui.end_row(); }
                }
            });
            ui.add_space(18.0);
            ui.heading("I/O throughput");
            ui.columns(2, |columns| {
                columns[0].label(RichText::new("Network (all interfaces except loopback)").strong());
                columns[0].label(format!("Rx  {}", format::rate(snapshot.rates.received_per_second)));
                columns[0].label(format!("Tx  {}", format::rate(snapshot.rates.sent_per_second)));
                let network_max = graph_max(&self.history.received, &self.history.sent);
                chart_lines(&mut columns[0], &[
                    (&self.history.received, accent),
                    (&self.history.sent, Color32::from_rgb(82, 173, 155)),
                ], network_max);
                columns[0].label(RichText::new(format!("Rx Accent  ·  Tx Green  ·  scale {}", format::rate(network_max as u64))).small().weak());
                columns[1].label(RichText::new("Block devices").strong());
                columns[1].label(format!("Read  {}", format::rate(snapshot.rates.disk_read_per_second)));
                columns[1].label(format!("Write  {}", format::rate(snapshot.rates.disk_write_per_second)));
                let disk_max = graph_max(&self.history.disk_read, &self.history.disk_write);
                chart_lines(&mut columns[1], &[
                    (&self.history.disk_read, accent),
                    (&self.history.disk_write, Color32::from_rgb(82, 173, 155)),
                ], disk_max);
                columns[1].label(RichText::new(format!("Read accent  ·  Write green  ·  scale {}", format::rate(disk_max as u64))).small().weak());
            });
            ui.add_space(12.0);
            ui.label(RichText::new("The first sample has no rate baseline. Network totals can include virtual interfaces; disk data represents block-layer traffic, not filesystem-only I/O.").small().weak());
            ui.add_space(18.0);
            self.gpu_telemetry_ui(ui);
            ui.add_space(18.0);
            self.power_telemetry_ui(ui);
            ui.add_space(18.0);
            self.pressure_telemetry_ui(ui);
        });
    }

    fn details_page(&mut self, ui: &mut egui::Ui) {
        if self.settings.theme == Theme::Windows10 {
            // Modern 10 uses a dense, ungrouped process report for Details.
            rices::processes(self, ui);
            return;
        }
        if self.settings.theme == Theme::XpLuna {
            rices::xp_details(self, ui);
            return;
        }
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
        self.process_follow_controls(ui);
        self.pss_inspection_controls(ui);
        self.cgroup_inspection_controls(ui);
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
                    ("Nice (leader)", process.nice.to_string()),
                    (
                        "CPUs allowed (leader)",
                        process
                            .cpus_allowed_list
                            .clone()
                            .unwrap_or_else(|| "Unavailable".into()),
                    ),
                    (
                        "Started (local)",
                        format::local_timestamp(process.start_unix_seconds),
                    ),
                    ("Runtime", format::process_age(process.elapsed_seconds)),
                    ("CPU", format!("{:.1}%", process.cpu_percent)),
                    ("Resident memory", format::bytes(process.resident_bytes)),
                    ("Virtual memory", format::bytes(process.virtual_bytes)),
                    ("Threads", process.threads.to_string()),
                    (
                        "Disk read (cumulative)",
                        format_process_io(process.io_accessible, process.read_bytes, false),
                    ),
                    (
                        "Disk read / second",
                        format_process_io(process.io_accessible, process.read_per_second, true),
                    ),
                    (
                        "Disk written (cumulative)",
                        format_process_io(process.io_accessible, process.written_bytes, false),
                    ),
                    (
                        "Disk written / second",
                        format_process_io(process.io_accessible, process.written_per_second, true),
                    ),
                ] {
                    ui.label(RichText::new(name).weak());
                    ui.label(value);
                    ui.end_row();
                }
            });
        if let Some(totals) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.tree_summary(process.pid, process.start_ticks))
        {
            ui.add_space(13.0);
            ui.label(RichText::new("OBSERVED PROCESS TREE").strong());
            ui.label(format!(
                "{} observed processes · combined CPU {:.1}% · sum of RSS {}",
                totals.process_count,
                totals.cpu_percent,
                format::bytes(totals.summed_rss_bytes),
            ));
            ui.label(RichText::new("Includes the selected PID and visible descendants only. RSS may count shared pages more than once; this is not unique physical memory.").small().weak());
        }
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
                if ui
                    .add_enabled(
                        can_signal_process(&process),
                        egui::Button::new(action.label()),
                    )
                    .clicked()
                {
                    self.queue_action(action);
                }
            }
        });
    }

    fn preferences_page(&mut self, ui: &mut egui::Ui) {
        if rices::supports(self.settings.theme) {
            let xp = self.settings.theme == Theme::XpLuna;
            egui::Frame::new()
                .fill(if self.settings.theme == Theme::Windows98 {
                    Color32::from_rgb(192, 192, 192)
                } else if xp {
                    Color32::from_rgb(246, 244, 231)
                } else {
                    Color32::WHITE
                })
                .stroke(Stroke::new(
                    1.0_f32,
                    if self.settings.theme == Theme::Windows98 {
                        Color32::from_rgb(128, 128, 128)
                    } else if xp {
                        Color32::from_rgb(158, 156, 138)
                    } else {
                        Color32::from_rgb(218, 218, 218)
                    },
                ))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| self.preferences_contents(ui));
        } else {
            self.preferences_contents(ui);
        }
    }

    fn preferences_contents(&mut self, ui: &mut egui::Ui) {
        let xp = matches!(self.settings.theme, Theme::XpLuna | Theme::Windows98);
        ui.label(
            RichText::new(if xp {
                "Taskotto Properties"
            } else if self.settings.theme == Theme::Windows10 {
                "Taskotto Settings"
            } else if self.settings.theme == Theme::Windows7 {
                "Taskotto Options"
            } else if self.settings.theme == Theme::Macos {
                "Taskotto Preferences"
            } else {
                "Preferences"
            })
            .size(if xp { 17.0 } else { 24.0 })
            .strong(),
        );
        ui.label(
            RichText::new("Taskotto appearance and monitoring controls · no background service")
                .small()
                .weak(),
        );
        if let Some(name) = &self.settings.launch_ricepack_name {
            ui.label(
                RichText::new(format!(
                    "RicePack active: {name} · appearance overrides are session-only; other settings can still be saved"
                ))
                .small(),
            );
            if let Some(watcher) = &self.ricepack_watcher {
                ui.label(
                    RichText::new(format!(
                        "Watching {} for changes (about every 2 seconds)",
                        watcher.path.display()
                    ))
                    .small()
                    .weak(),
                );
            }
        }
        ui.add_space(if xp { 8.0 } else { 14.0 });
        let mut changed = false;
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.ricepack_library_controls(ui, xp);
            ui.add_space(18.0);
            preference_section(ui, "Appearance", xp);
            ui.horizontal(|ui| {
                ui.label("Rice profile");
                egui::ComboBox::from_id_salt("theme-choice")
                    .selected_text(self.settings.theme.label())
                    .show_ui(ui, |ui| {
                        for theme in Theme::ALL {
                            if ui.selectable_value(&mut self.settings.theme, theme, theme.label()).changed() {
                                self.settings.launch_theme_original = None;
                                if self.settings.theme == Theme::Windows98 {
                                    self.page = Page::Applications;
                                } else if self.page == Page::Applications {
                                    self.page = Page::Processes;
                                }
                                changed = true;
                            }
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Density");
                changed |= ui.radio_value(&mut self.settings.density, Density::Compact, "Compact").changed();
                changed |= ui.radio_value(&mut self.settings.density, Density::Comfortable, "Comfortable").changed();
            });
            ui.horizontal(|ui| {
                ui.label("Navigation");
                egui::ComboBox::from_id_salt("layout-choice")
                    .selected_text(match self.settings.layout_mode {
                        LayoutMode::Auto => "Automatic",
                        LayoutMode::Sidebar => "Sidebar",
                        LayoutMode::TopTabs => "Top tabs",
                    })
                    .show_ui(ui, |ui| {
                        changed |= ui.selectable_value(&mut self.settings.layout_mode, LayoutMode::Auto, "Automatic per profile").changed();
                        changed |= ui.selectable_value(&mut self.settings.layout_mode, LayoutMode::Sidebar, "Sidebar").changed();
                        changed |= ui.selectable_value(&mut self.settings.layout_mode, LayoutMode::TopTabs, "Top tabs").changed();
                    });
            });
            changed |= ui.add(egui::Slider::new(&mut self.settings.font_scale, 0.8..=1.5).text("Font scale")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.settings.ui_spacing, 3.0..=16.0).text("Widget spacing")).changed();
            changed |= ui.add(egui::Slider::new(&mut self.settings.chart_scale, 0.75..=1.5).text("Chart size")).changed();
            let mut radius = self.settings.corner_radius.unwrap_or_else(|| ThemeStyle::from_settings(&self.settings).rounding);
            if ui.add(egui::Slider::new(&mut radius, 0..=16).text("Corner radius")).changed() {
                self.settings.corner_radius = Some(radius);
                changed = true;
            }
            if self.settings.corner_radius.is_some() && ui.small_button("Use profile's default corners").clicked() {
                self.settings.corner_radius = None;
                changed = true;
            }
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
            for (label, input, destination) in [
                ("Surface color (optional #RRGGBB)", &mut self.theme_surface_input, &mut self.settings.surface_hex),
                ("Text color (optional #RRGGBB)", &mut self.theme_text_input, &mut self.settings.text_hex),
            ] {
                ui.horizontal(|ui| {
                    ui.label(label);
                    let response = ui.add(egui::TextEdit::singleline(input).desired_width(130.0));
                    if response.lost_focus() {
                        if input.is_empty() || is_hex_color(input) {
                            *destination = if input.is_empty() { None } else { Some(input.clone()) };
                            changed = true;
                        } else {
                            self.notification = Some(format!("{label} must be a #RRGGBB color"));
                        }
                    }
                });
            }
            ui.label(RichText::new("Low-contrast text colors automatically fall back to a readable color.").small().weak());
            ui.add_space(18.0);
            preference_section(ui, "Component colors", xp);
            ui.label(RichText::new("Optional overrides for controls, borders, tables and selection. Reset each color to keep the rice's original design.").small().weak());
            let defaults = ui.visuals().clone();
            let components = &mut self.settings.components;
            changed |= component_color_picker(ui, "Controls", &mut components.control, defaults.widgets.inactive.bg_fill);
            changed |= component_color_picker(ui, "Hover", &mut components.hover, defaults.widgets.hovered.bg_fill);
            changed |= component_color_picker(ui, "Borders", &mut components.border, defaults.widgets.inactive.bg_stroke.color);
            changed |= component_color_picker(ui, "Table background", &mut components.table, defaults.extreme_bg_color);
            changed |= component_color_picker(ui, "Selection", &mut components.selection, defaults.selection.bg_fill);
            ui.add_space(18.0);
            preference_section(ui, "Process table", xp);
            let mut frozen = self.frozen_sort_order.is_some();
            if ui.checkbox(&mut frozen, "Freeze process order (this session)").on_hover_text("Ctrl+Shift+Space. Newly observed processes appear at the end; restarting resets the order.").changed() {
                self.set_sort_frozen(frozen);
            }
            changed |= ui.checkbox(&mut self.settings.show_io_column, "Show I/O rate (wide windows)").changed();
            changed |= ui.checkbox(&mut self.settings.show_user_column, "Show user (wide windows)").changed();
            let mut row_height = self.settings.table_row_height.unwrap_or(if self.settings.density == Density::Compact { 26.0 } else { 34.0 });
            if ui.add(egui::Slider::new(&mut row_height, 22.0..=48.0).text("Row height")).changed() {
                self.settings.table_row_height = Some(row_height);
                changed = true;
            }
            if self.settings.table_row_height.is_some() && ui.small_button("Use default row height").clicked() {
                self.settings.table_row_height = None;
                changed = true;
            }
            ui.add_space(18.0);
            preference_section(ui, "Process follow · this session", xp);
            self.process_follow_controls(ui);
            ui.label(RichText::new("Only a deliberately followed process is recorded. Its PID and start time must both match; samples are bounded to 120, and tracking stops after three missed samples. No file is written.").small().weak());
            ui.add_space(18.0);
            preference_section(ui, "Proportional memory · on demand", xp);
            self.pss_inspection_controls(ui);
            ui.add_space(18.0);
            preference_section(ui, "Linux cgroup v2 · on demand", xp);
            self.cgroup_inspection_controls(ui);
            ui.add_space(18.0);
            preference_section(ui, "GPU sensors · read only", xp);
            self.gpu_telemetry_ui(ui);
            ui.add_space(18.0);
            preference_section(ui, "Battery and external power · read only", xp);
            self.power_telemetry_ui(ui);
            ui.add_space(18.0);
            preference_section(ui, "Linux pressure stalls · PSI", xp);
            self.pressure_telemetry_ui(ui);
            ui.add_space(18.0);
            preference_section(ui, "Monitoring", xp);
            changed |= ui.add(egui::Slider::new(&mut self.settings.refresh_ms, 500..=10_000).text("Refresh interval (ms)")).changed();
            changed |= ui.checkbox(&mut self.settings.only_my_processes, "Show only my processes").changed();
            changed |= ui.checkbox(&mut self.settings.show_kernel_threads, "Show kernel threads").changed();
            ui.add_space(12.0);
            ui.label(RichText::new("The UI samples only while the application is open. Longer intervals reduce idle CPU use.").small().weak());
            ui.add_space(18.0);
            if ui.button("Reset to defaults").clicked() {
                self.ricepack_library.preview = None;
                self.ricepack_library.editor_layout = None;
                self.ricepack_library.editor_error = None;
                self.ricepack_watcher = None;
                self.settings = Settings::default();
                self.theme_accent_input.clear();
                self.theme_surface_input.clear();
                self.theme_text_input.clear();
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

    fn pressure_telemetry_ui(&self, ui: &mut egui::Ui) {
        let Some(snapshot) = &self.snapshot else {
            ui.label(RichText::new("Pressure stalls: waiting for Linux system sample…").weak());
            return;
        };
        let pressure = &snapshot.pressure;
        if pressure.is_empty() {
            ui.label(
                RichText::new("Kernel PSI counters unavailable in /proc/pressure.")
                    .small()
                    .weak(),
            );
            return;
        }
        ui.label(
            RichText::new("Share of elapsed time that tasks waited for a resource")
                .small()
                .weak(),
        );
        let rows = [
            ("CPU · some stalled", pressure.cpu_some.as_ref()),
            ("Memory · some stalled", pressure.memory_some.as_ref()),
            ("Memory · full stalled", pressure.memory_full.as_ref()),
            ("I/O · some stalled", pressure.io_some.as_ref()),
            ("I/O · full stalled", pressure.io_full.as_ref()),
        ];
        if ui.available_width() < 700.0 {
            for (name, metric) in rows {
                ui.label(RichText::new(name).strong());
                if let Some(metric) = metric {
                    ui.label(format!(
                        "10s {:.2}% · 60s {:.2}% · 300s {:.2}%",
                        metric.avg10_percent, metric.avg60_percent, metric.avg300_percent
                    ));
                    ui.label(
                        RichText::new(format!(
                            "Total {:.1}s",
                            metric.total_stall_usec as f64 / 1_000_000.0
                        ))
                        .small()
                        .weak(),
                    );
                } else {
                    ui.label("Unavailable");
                }
                ui.add_space(4.0);
            }
        } else {
            egui::Grid::new("linux-psi-pressure")
                .num_columns(3)
                .spacing([20.0, 7.0])
                .show(ui, |ui| {
                    for (name, metric) in rows {
                        ui.label(RichText::new(name).weak());
                        if let Some(metric) = metric {
                            ui.label(format!(
                                "10s {:.2}% · 60s {:.2}% · 300s {:.2}%",
                                metric.avg10_percent, metric.avg60_percent, metric.avg300_percent
                            ));
                            ui.label(format!(
                                "Total {:.1}s",
                                metric.total_stall_usec as f64 / 1_000_000.0
                            ));
                        } else {
                            ui.label("Unavailable");
                            ui.label("—");
                        }
                        ui.end_row();
                    }
                });
        }
        ui.label(RichText::new("PSI is stalled-task time, not CPU usage, RAM usage, disk throughput or a diagnosis on its own. System-level CPU FULL is deliberately omitted; totals are cumulative since kernel start and may have different scopes in containers.").small().weak());
    }

    fn power_telemetry_ui(&self, ui: &mut egui::Ui) {
        let Some(snapshot) = &self.snapshot else {
            ui.label(RichText::new("Power sensors: waiting for Linux system sample…").weak());
            return;
        };
        let power = &snapshot.power;
        if power.is_empty() {
            ui.label(
                RichText::new("No power_supply battery or adapter sensors available.")
                    .small()
                    .weak(),
            );
            return;
        }
        if let Some(online) = power.external_power_online {
            ui.label(if online {
                "External power: connected"
            } else {
                "External power: disconnected"
            });
        }
        for battery in &power.batteries {
            ui.label(RichText::new(format!("Battery · {}", battery.name)).strong());
            let unknown = || "Unavailable".to_owned();
            let rows = [
                ("Status", battery.status.clone().unwrap_or_else(unknown)),
                (
                    "Charge",
                    battery
                        .capacity_percent
                        .map(|value| format!("{value}%"))
                        .unwrap_or_else(unknown),
                ),
                (
                    "Stored energy",
                    match (battery.energy_now_wh, battery.energy_full_wh) {
                        (Some(now), Some(full)) => format!("{now:.1} / {full:.1} Wh"),
                        (Some(now), None) => format!("{now:.1} Wh (full capacity unavailable)"),
                        _ => unknown(),
                    },
                ),
                (
                    "Power rate",
                    battery
                        .power_watts
                        .map(|value| format!("{value:.1} W"))
                        .unwrap_or_else(unknown),
                ),
            ];
            egui::Grid::new(("battery-power", &battery.name))
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    for (name, value) in rows {
                        ui.label(RichText::new(name).weak());
                        ui.label(value);
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
        }
        ui.label(RichText::new("Linux energy fields use Wh, current capacity uses %. Charge-based sensors use different units and are never mislabeled as Wh. Missing hardware readings are not zero; no synthetic battery runtime is calculated.").small().weak());
    }

    fn gpu_telemetry_ui(&self, ui: &mut egui::Ui) {
        let Some(snapshot) = &self.snapshot else {
            ui.label(RichText::new("GPU sensors: waiting for Linux system sample…").weak());
            return;
        };
        if snapshot.gpus.is_empty() {
            ui.label(
                RichText::new(
                    "No DRM GPU device found in /sys/class/drm (or sysfs is unavailable).",
                )
                .small()
                .weak(),
            );
            return;
        }
        for gpu in &snapshot.gpus {
            ui.label(
                RichText::new(format!(
                    "{} · {} · {}",
                    gpu.card,
                    gpu.vendor,
                    gpu.driver.as_deref().unwrap_or("driver unavailable")
                ))
                .strong(),
            );
            let unknown = || "Unavailable (driver capability)".to_string();
            let fields = [
                (
                    "GPU busy",
                    gpu.utilization_percent
                        .map(|value| format!("{value}%"))
                        .unwrap_or_else(unknown),
                ),
                (
                    "VRAM used / total",
                    match (gpu.vram_used_bytes, gpu.vram_total_bytes) {
                        (Some(used), Some(total)) => {
                            format!("{} / {}", format::bytes(used), format::bytes(total))
                        }
                        _ => unknown(),
                    },
                ),
                (
                    "Temperature",
                    gpu.temperature_celsius
                        .map(|value| format!("{value:.1} °C"))
                        .unwrap_or_else(unknown),
                ),
                (
                    "Power",
                    gpu.power_watts
                        .map(|value| format!("{value:.1} W"))
                        .unwrap_or_else(unknown),
                ),
                (
                    "Core clock",
                    gpu.core_clock_mhz
                        .map(|value| format!("{value:.0} MHz"))
                        .unwrap_or_else(unknown),
                ),
            ];
            egui::Grid::new(("gpu-telemetry", &gpu.card))
                .num_columns(2)
                .spacing([20.0, 6.0])
                .show(ui, |ui| {
                    for (name, value) in fields {
                        ui.label(RichText::new(name).weak());
                        ui.label(value);
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
        }
        ui.label(RichText::new("AMDGPU exposes busy/VRAM and optional hwmon values; Intel/NVIDIA utilization requires a supported driver backend. Missing fields are not zero. Some AMD APU power sensors include CPU power.").small().weak());
    }

    fn cgroup_inspection_controls(&mut self, ui: &mut egui::Ui) {
        let selected = self.selected_process().map(|p| (p.pid, p.start_ticks));
        let loading = self
            .history
            .cgroup_inspection
            .as_ref()
            .is_some_and(|inspection| matches!(inspection.state, CgroupState::Loading(_)));
        if ui.add_enabled(selected.is_some() && !loading, egui::Button::new("Inspect cgroup v2"))
            .on_hover_text("Read selected PID's cgroup v2 resource counters once, asynchronously; no process/container control")
            .clicked()
            && let Some(target) = selected
        {
            let (tx, rx) = mpsc::channel();
            let ctx = ui.ctx().clone();
            thread::spawn(move || {
                let result = crate::monitor::cgroup::inspect(target.0, target.1);
                let _ = tx.send(result);
                ctx.request_repaint();
            });
            self.history.cgroup_inspection = Some(CgroupInspection {
                target,
                state: CgroupState::Loading(rx),
            });
        }
        let Some(inspection) = &self.history.cgroup_inspection else {
            return;
        };
        if selected != Some(inspection.target) {
            ui.label(
                RichText::new(format!(
                    "Last inspection was for PID {} (different from selection)",
                    inspection.target.0
                ))
                .small()
                .weak(),
            );
        }
        match &inspection.state {
            CgroupState::Loading(_) => {
                ui.spinner();
                ui.label("Reading cgroup v2 counters…");
            }
            CgroupState::Ready(Err(error)) => {
                ui.label(
                    RichText::new(format!("Cgroup unavailable: {error}"))
                        .small()
                        .weak(),
                );
            }
            CgroupState::Ready(Ok(report)) => {
                ui.label(
                    RichText::new(format!("Cgroup: {}", report.path))
                        .small()
                        .strong(),
                );
                let available_bytes = |value: Option<u64>| {
                    value
                        .map(format::bytes)
                        .unwrap_or_else(|| "Unavailable".into())
                };
                let available_usec = |value: Option<u64>| {
                    value
                        .map(|us| format!("{:.2} s", us as f64 / 1_000_000.0))
                        .unwrap_or_else(|| "Unavailable".into())
                };
                let limit = match report.memory_limit {
                    crate::monitor::cgroup::MemoryLimit::Bytes(bytes) => format::bytes(bytes),
                    crate::monitor::cgroup::MemoryLimit::Unlimited => "No cgroup limit".into(),
                    crate::monitor::cgroup::MemoryLimit::Unavailable => "Unavailable".into(),
                };
                let rows = [
                    (
                        "CPU usage (cumulative)",
                        available_usec(report.cpu_usage_usec),
                    ),
                    (
                        "CPU throttle time",
                        available_usec(report.cpu_throttled_usec),
                    ),
                    (
                        "CPU throttle events",
                        report
                            .cpu_throttled_events
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "Unavailable".into()),
                    ),
                    ("Memory current", available_bytes(report.memory_bytes)),
                    ("Memory max", limit),
                    (
                        "I/O read (cumulative)",
                        available_bytes(report.io_read_bytes),
                    ),
                    (
                        "I/O written (cumulative)",
                        available_bytes(report.io_write_bytes),
                    ),
                ];
                egui::Grid::new("cgroup-inspection-fields")
                    .num_columns(2)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        for (name, value) in rows {
                            ui.label(RichText::new(name).weak());
                            ui.label(value);
                            ui.end_row();
                        }
                    });
                ui.label(RichText::new("These are cumulative counters for this Linux cgroup, potentially including other processes and nested cgroups. They are not per-PID or container-API values; collection is a non-atomic snapshot.").small().weak());
            }
        }
    }

    fn pss_inspection_controls(&mut self, ui: &mut egui::Ui) {
        let selected = self.selected_process().map(|p| (p.pid, p.start_ticks));
        let loading = self
            .history
            .pss_inspection
            .as_ref()
            .is_some_and(|inspection| matches!(inspection.state, PssState::Loading(_)));
        if ui
            .add_enabled(selected.is_some() && !loading, egui::Button::new("Measure PSS now"))
            .on_hover_text("Read selected process and observed descendants from /proc/*/smaps_rollup on a background worker. No persistent sampling.")
            .clicked()
            && let (Some(target), Some(snapshot)) = (selected, self.snapshot.clone())
        {
            let (tx, rx) = mpsc::channel();
            let ctx = ui.ctx().clone();
            thread::spawn(move || {
                let result = crate::monitor::pss::inspect_tree(&snapshot, target);
                let _ = tx.send(result);
                ctx.request_repaint();
            });
            self.history.pss_inspection = Some(PssInspection {
                target,
                state: PssState::Loading(rx),
            });
        }
        let Some(inspection) = &self.history.pss_inspection else {
            return;
        };
        if selected != Some(inspection.target) {
            ui.label(
                RichText::new(format!(
                    "Last PSS inspection: PID {} (different from current selection)",
                    inspection.target.0
                ))
                .small()
                .weak(),
            );
        }
        match &inspection.state {
            PssState::Loading(_) => {
                ui.spinner();
                ui.label("Reading proportional memory asynchronously…");
            }
            PssState::Ready(Ok(report)) => {
                if report.complete() {
                    ui.label(format!(
                        "Observed tree PSS: {} across {} processes",
                        format::bytes(report.proportional_bytes),
                        report.observed
                    ));
                } else {
                    ui.label(format!("Incomplete PSS: {} from {} of {} observed processes; missing values unavailable (not zero)", format::bytes(report.proportional_bytes), report.readable, report.observed));
                }
                ui.label(RichText::new("PSS is each process's proportional share of its resident pages, not unique RAM owned by this tree. Memory maps may change during collection; permission restrictions and exiting PIDs can make results incomplete.").small().weak());
            }
            PssState::Ready(Err(error)) => {
                ui.label(
                    RichText::new(format!("PSS unavailable: {error}"))
                        .small()
                        .weak(),
                );
            }
        }
    }

    fn process_follow_controls(&mut self, ui: &mut egui::Ui) {
        let selected = self.selected_process().cloned();
        let following_selected = self.history.process_follow.as_ref().is_some_and(|follow| {
            follow.active
                && selected.as_ref().is_some_and(|process| {
                    (process.pid, process.start_ticks) == (follow.pid, follow.start_ticks)
                })
        });
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    selected.is_some(),
                    egui::Button::new(if following_selected {
                        "Stop following"
                    } else {
                        "Follow selected process"
                    }),
                )
                .on_hover_text(
                    "Remember up to 120 live CPU and RSS samples for this exact Linux process",
                )
                .clicked()
            {
                self.history.process_follow = if following_selected {
                    None
                } else {
                    selected.as_ref().map(ProcessFollow::new)
                };
            }
            if self.history.process_follow.is_some() && ui.small_button("Clear history").clicked() {
                self.history.process_follow = None;
            }
        });
        let Some(follow) = &self.history.process_follow else {
            return;
        };
        let state = if !follow.active {
            "Ended: 3 missed samples (no automatic PID reattach)"
        } else if follow.missed > 0 {
            "Temporarily unavailable: waiting for a matching sample"
        } else {
            "Following live process"
        };
        ui.label(
            RichText::new(format!("{} · PID {} · {}", follow.name, follow.pid, state)).small(),
        );
        if follow.cpu.is_empty() {
            ui.label(RichText::new("Waiting for the next sample…").small().weak());
            return;
        }
        let accent = accent_color(&self.settings);
        let cpu_peak = graph_max(&follow.cpu, &VecDeque::new());
        let mem_peak = graph_max(&follow.rss_mib, &VecDeque::new());
        let cpu_max = cpu_peak.max(100.0);
        let mem_max = mem_peak.max(1.0);
        ui.columns(2, |columns| {
            columns[0].label(format!(
                "CPU · latest {:.1}% · peak {:.1}%",
                follow.cpu.back().unwrap_or(&0.0),
                cpu_peak
            ));
            chart(&mut columns[0], &follow.cpu, cpu_max, accent);
            columns[1].label(format!(
                "RSS · latest {:.1} MiB · peak {:.1} MiB",
                follow.rss_mib.back().unwrap_or(&0.0),
                mem_peak
            ));
            chart(
                &mut columns[1],
                &follow.rss_mib,
                mem_max,
                Color32::from_rgb(82, 173, 155),
            );
        });
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

    fn new_task_dialog(&mut self, ctx: &egui::Context) {
        if !self.new_task.open {
            return;
        }
        let mut open = true;
        let mut run = false;
        egui::Window::new("Create New Task")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(if self.settings.theme == Theme::Windows98 {
                        Color32::from_rgb(192, 192, 192)
                    } else if self.settings.theme == Theme::XpLuna {
                        Color32::from_rgb(236, 233, 216)
                    } else {
                        Color32::WHITE
                    })
                    .stroke(Stroke::new(
                        2.0_f32,
                        if self.settings.theme == Theme::Windows98 {
                            Color32::from_rgb(128, 128, 128)
                        } else if self.settings.theme == Theme::XpLuna {
                            Color32::from_rgb(51, 94, 169)
                        } else {
                            Color32::from_rgb(168, 168, 168)
                        },
                    )),
            )
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_min_width(340.0);
                ui.label("Type the name of a program or command to open.");
                ui.add_space(7.0);
                ui.horizontal(|ui| {
                    ui.label("Open:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_task.command).desired_width(280.0),
                    );
                    if response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter))
                    {
                        run = true;
                    }
                });
                if let Some(message) = &self.new_task.error {
                    ui.colored_label(Color32::from_rgb(156, 65, 24), message);
                }
                ui.label(
                    RichText::new("Runs as your current Linux user; no administrator elevation.")
                        .small()
                        .weak(),
                );
                ui.horizontal(|ui| {
                    if ui.button("Run").clicked() {
                        run = true;
                    }
                    if ui.button("Cancel").clicked() {
                        self.new_task.open = false;
                    }
                });
            });
        if run {
            let command = self.new_task.command.trim();
            match launch_user_task(command) {
                Ok(()) => {
                    self.new_task.error = None;
                    self.new_task.open = false;
                }
                Err(err) => self.new_task.error = Some(format!("Could not start task: {err}")),
            }
        } else {
            self.new_task.open &= open;
        }
    }

    fn draw_contents(&mut self, ctx: &egui::Context) {
        self.topbar(ctx);
        self.nav(ctx);
        if rices::supports(self.settings.theme) {
            rices::statusbar(self, ctx);
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(if rices::supports(self.settings.theme) {
                3.0
            } else {
                14.0
            });
            if self.settings.theme == Theme::Windows98
                && self.settings.layout_mode.resolve(self.settings.theme) == LayoutMode::TopTabs
                && self.page != Page::Applications
            {
                rices::classic98_navigation(self, ui);
            }
            if let Some(message) = self.notification.clone() {
                ui.horizontal_wrapped(|ui| {
                    let warning_color = if rices::supports(self.settings.theme) {
                        Color32::from_rgb(155, 69, 16)
                    } else {
                        Color32::from_rgb(226, 147, 74)
                    };
                    ui.colored_label(warning_color, message);
                    if ui.small_button("Dismiss ×").clicked() {
                        self.notification = None;
                    }
                });
                ui.separator();
            }
            match self.page {
                Page::Applications => rices::applications(self, ui),
                Page::Processes => self.processes_page(ui),
                Page::Performance => self.performance_page(ui),
                Page::AppHistory => rices::app_history(self, ui),
                Page::Startup => rices::startup(self, ui),
                Page::Networking => rices::networking(self, ui),
                Page::Users => rices::users(self, ui),
                Page::Details if self.settings.theme == Theme::Windows7 => self.processes_page(ui),
                Page::Details => self.details_page(ui),
                Page::Services => rices::services(self, ui),
                Page::MacEnergy => rices::mac_energy(self, ui),
                Page::MacDisk => rices::mac_disk(self, ui),
                Page::Preferences => self.preferences_page(ui),
            }
        });
        if self.settings.theme == Theme::Windows7 && self.page == Page::Details {
            rices::seven_properties_dialog(self, ctx);
        }
        if self.settings.theme == Theme::Macos && self.mac_info_open {
            rices::mac_info_dialog(self, ctx);
        }
        self.confirmation_dialog(ctx);
        self.new_task_dialog(ctx);
    }
}

impl eframe::App for TaskottoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.accept_snapshots();
        self.maybe_reload_ricepack(ctx);
        ctx.input(|input| {
            if input.key_pressed(egui::Key::F5) {
                let _ = self.worker.commands.send(Command::Refresh);
            }
            if input.modifiers.command && input.key_pressed(egui::Key::F) {
                if self.settings.theme != Theme::Macos {
                    self.page = Page::Processes;
                }
                self.focus_search = true;
            }
            if input.modifiers.command
                && input.modifiers.shift
                && input.key_pressed(egui::Key::Space)
            {
                self.set_sort_frozen(self.frozen_sort_order.is_none());
            }
            if input.key_pressed(egui::Key::Escape) && self.pending.is_none() {
                if self.settings.theme == Theme::Macos && self.mac_info_open {
                    self.mac_info_open = false;
                } else if self.settings.theme == Theme::Windows7 && self.page == Page::Details {
                    self.page = Page::Processes;
                } else {
                    self.query.clear();
                }
            }
        });
        self.draw_contents(ctx);
        // The sampler wakes the GUI. Only live RicePack sessions use a 2s check.
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

fn can_signal_process(process: &Process) -> bool {
    process.pid > 1 && process.pid != std::process::id() as i32
}

fn process_io_rate(process: &Process) -> u64 {
    process
        .read_per_second
        .saturating_add(process.written_per_second)
}

fn compare_process_order(
    a: &Process,
    b: &Process,
    sort_by: SortBy,
    descending: bool,
    frozen: Option<&HashMap<(i32, u64), usize>>,
) -> std::cmp::Ordering {
    if let Some(order) = frozen {
        // Unknown/new processes go to the end in deterministic PID order;
        // reused PIDs never inherit a stale process identity's position.
        let a_rank = order.get(&(a.pid, a.start_ticks));
        let b_rank = order.get(&(b.pid, b.start_ticks));
        return match (a_rank, b_rank) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.pid.cmp(&b.pid),
        };
    }
    let order = match sort_by {
        SortBy::Cpu => a.cpu_percent.total_cmp(&b.cpu_percent),
        SortBy::Memory => a.resident_bytes.cmp(&b.resident_bytes),
        SortBy::Io => process_io_rate(a).cmp(&process_io_rate(b)),
        SortBy::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        SortBy::Pid => a.pid.cmp(&b.pid),
    };
    (if descending { order.reverse() } else { order }).then(a.pid.cmp(&b.pid))
}

fn format_process_io(accessible: bool, value: u64, per_second: bool) -> String {
    if !accessible {
        "Unavailable (permissions)".to_string()
    } else if per_second {
        format::rate(value)
    } else {
        format::bytes(value)
    }
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
    chart_lines(ui, &[(values, color)], max_value);
}

fn chart_lines(ui: &mut egui::Ui, series: &[(&VecDeque<f32>, Color32)], max_value: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, 6.0, ui.visuals().faint_bg_color);
    for line in [0.25, 0.5, 0.75] {
        let y = rect.bottom() - rect.height() * line;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(0.5_f32, ui.visuals().widgets.noninteractive.bg_stroke.color),
        );
    }
    for &(values, color) in series {
        if values.len() >= 2 {
            ui.painter().add(egui::Shape::line(
                chart_points(values, rect, max_value),
                Stroke::new(2.0_f32, color),
            ));
        }
    }
}

fn chart_points(values: &VecDeque<f32>, rect: egui::Rect, max_value: f32) -> Vec<egui::Pos2> {
    let denominator = (values.len().saturating_sub(1)).max(1) as f32;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            egui::pos2(
                rect.left() + rect.width() * index as f32 / denominator,
                rect.bottom() - rect.height() * (*value / max_value).clamp(0.0, 1.0),
            )
        })
        .collect()
}

fn graph_max(first: &VecDeque<f32>, second: &VecDeque<f32>) -> f32 {
    first
        .iter()
        .chain(second.iter())
        .copied()
        .filter(|value| value.is_finite())
        .fold(1.0, f32::max)
}

fn preference_section(ui: &mut egui::Ui, title: &str, xp: bool) {
    if xp {
        ui.label(RichText::new(title).size(13.0).strong());
        ui.separator();
    } else {
        ui.label(RichText::new(title).size(18.0).strong());
        ui.add_space(3.0);
    }
}

fn component_color_picker(
    ui: &mut egui::Ui,
    label: &str,
    destination: &mut Option<String>,
    fallback: Color32,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.add_sized([112.0, 20.0], egui::Label::new(label));
        let mut color = destination
            .as_deref()
            .and_then(crate::theme::parse_hex)
            .unwrap_or(fallback);
        if ui.color_edit_button_srgba(&mut color).changed() {
            *destination = Some(format!(
                "#{:02X}{:02X}{:02X}",
                color.r(),
                color.g(),
                color.b()
            ));
            changed = true;
        }
        if ui
            .add_enabled(destination.is_some(), egui::Button::new("Use preset"))
            .clicked()
        {
            *destination = None;
            changed = true;
        }
    });
    changed
}

fn accent_color(settings: &Settings) -> Color32 {
    ThemeStyle::from_settings(settings).accent
}

fn apply_theme(ctx: &egui::Context, settings: &Settings) {
    ThemeStyle::from_settings(settings).apply(ctx, settings);
}

#[cfg(test)]
mod captures;
mod layout_editor;
mod rices;

#[cfg(test)]
mod tests {
    use super::*;

    fn dark_component_tokens() -> crate::config::ComponentColors {
        crate::config::ComponentColors {
            control: Some("#304761".to_owned()),
            hover: Some("#416387".to_owned()),
            border: Some("#63809E".to_owned()),
            table: Some("#182639".to_owned()),
            selection: Some("#DDEAF8".to_owned()),
        }
    }

    #[test]
    fn new_task_requires_explicit_command_and_uses_regular_user_launcher() {
        assert!(launch_user_task("   ").is_err());
        assert!(launch_user_task("/bin/true").is_ok());
    }

    #[test]
    fn missing_user_data_does_not_crash() {
        assert_eq!(state_name('Z'), "Zombie");
        assert_eq!(state_name('?'), "Other");
        assert_eq!(
            format_process_io(false, 0, true),
            "Unavailable (permissions)"
        );
    }

    #[test]
    fn protected_pids_are_not_actionable_in_rice_controls() {
        assert!(!can_signal_process(&Process {
            pid: 1,
            ..Process::default()
        }));
        assert!(!can_signal_process(&Process {
            pid: std::process::id() as i32,
            ..Process::default()
        }));
        assert!(can_signal_process(&Process {
            pid: i32::MAX,
            ..Process::default()
        }));
    }

    #[test]
    fn frozen_process_order_survives_live_metrics_pid_reuse_and_unfreeze() {
        let ctx = egui::Context::default();
        let mut app = TaskottoApp {
            settings: Settings::default(),
            worker: Worker::start(10_000, ctx),
            snapshot: Some(Snapshot {
                processes: vec![
                    Process {
                        pid: 42,
                        start_ticks: 100,
                        name: "first".into(),
                        cpu_percent: 80.0,
                        ..Process::default()
                    },
                    Process {
                        pid: 7,
                        start_ticks: 200,
                        name: "second".into(),
                        cpu_percent: 40.0,
                        ..Process::default()
                    },
                ],
                cpu_percent: 0.0,
                cores_percent: vec![],
                memory: Default::default(),
                rates: Default::default(),
                uptime_seconds: 0.0,
                logical_cpus: 1,
                gpus: vec![],
                power: Default::default(),
                pressure: Default::default(),
            }),
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        };
        let pids = |app: &TaskottoApp| {
            app.processes()
                .iter()
                .map(|row| (row.process.pid, row.process.start_ticks))
                .collect::<Vec<_>>()
        };
        assert_eq!(pids(&app), [(42, 100), (7, 200)]);
        app.set_sort_frozen(true);
        assert_eq!(app.frozen_sort_order.as_ref().map(HashMap::len), Some(2));

        let snapshot = app.snapshot.as_mut().unwrap();
        snapshot.processes[0].cpu_percent = 1.0;
        snapshot.processes[1].cpu_percent = 99.0;
        // PID 42 has exited; a different process acquired the same numeric PID.
        snapshot.processes[0].start_ticks = 300;
        snapshot.processes.push(Process {
            pid: 200,
            start_ticks: 400,
            cpu_percent: 55.0,
            ..Process::default()
        });
        assert_eq!(pids(&app), [(7, 200), (42, 300), (200, 400)]);
        app.set_sort_frozen(false);
        assert_eq!(pids(&app), [(7, 200), (200, 400), (42, 300)]);

        app.settings.tree_view = true;
        app.set_sort_frozen(true);
        app.snapshot.as_mut().unwrap().processes[0].parent_pid = 7;
        // Tree view still preserves the hierarchy; it never sacrifices it
        // just to reproduce a flat frozen row index.
        let rows = app.processes();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows.iter().map(|row| row.process.pid).collect::<Vec<_>>(),
            [7, 42, 200]
        );
        assert_eq!(rows[1].depth, 1);
    }

    #[test]
    fn explicit_process_follow_is_bounded_and_never_reattaches_reused_pid() {
        let process = Process {
            pid: 77,
            start_ticks: 300,
            name: "watched".into(),
            cpu_percent: 12.5,
            resident_bytes: 4 * 1024 * 1024,
            ..Process::default()
        };
        let mut history = History::default();
        let mut snapshot = Snapshot {
            processes: vec![process.clone()],
            cpu_percent: 0.0,
            cores_percent: vec![],
            memory: Default::default(),
            rates: Default::default(),
            uptime_seconds: 0.0,
            logical_cpus: 1,
            gpus: vec![],
            power: Default::default(),
            pressure: Default::default(),
        };
        history.process_follow = Some(ProcessFollow::new(&process));
        for _ in 0..200 {
            history.record(&snapshot);
        }
        let trail = history.process_follow.as_ref().unwrap();
        assert_eq!(trail.cpu.len(), ProcessFollow::LIMIT);
        assert_eq!(trail.rss_mib.len(), ProcessFollow::LIMIT);
        assert_eq!(trail.cpu.back(), Some(&12.5));
        assert_eq!(trail.rss_mib.back(), Some(&4.0));

        // Reused numeric PID with a different start tick must not be recorded.
        snapshot.processes[0].start_ticks = 301;
        snapshot.processes[0].cpu_percent = 99.0;
        for _ in 0..3 {
            history.record(&snapshot);
        }
        let trail = history.process_follow.as_ref().unwrap();
        assert!(!trail.active);
        assert_eq!(trail.cpu.back(), Some(&12.5));
        snapshot.processes[0] = process;
        history.record(&snapshot);
        assert!(!history.process_follow.as_ref().unwrap().active);
        assert_eq!(history.process_follow.as_ref().unwrap().cpu.len(), 120);
    }

    #[test]
    fn history_stays_bounded() {
        let mut history = History::default();
        let mut snapshot = Snapshot {
            cpu_percent: 33.0,
            cores_percent: vec![18.0, 55.0],
            memory: Default::default(),
            rates: Default::default(),
            uptime_seconds: 0.0,
            processes: vec![],
            logical_cpus: 1,
            gpus: vec![],
            power: Default::default(),
            pressure: Default::default(),
        };
        for _ in 0..200 {
            history.record(&snapshot);
        }
        assert_eq!(history.cpu.len(), 120);
        assert_eq!(history.disk_read.len(), 120);
        assert_eq!(history.cores.len(), 2);
        assert_eq!(history.cores[0].len(), 120);
        assert_eq!(history.cores[1].back().copied(), Some(55.0));
        snapshot.cores_percent = vec![10.0];
        history.record(&snapshot);
        assert_eq!(history.cores.len(), 1);
        assert_eq!(history.cores[0].len(), 120);
    }

    #[test]
    fn throughput_scale_handles_idle_and_nonfinite_samples() {
        assert_eq!(graph_max(&VecDeque::new(), &VecDeque::new()), 1.0);
        assert_eq!(
            graph_max(&VecDeque::from([f32::NAN, 3.0]), &VecDeque::from([5.0])),
            5.0
        );
    }

    #[test]
    fn chart_history_uses_full_width_even_before_120_samples() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, Vec2::new(320.0, 120.0));
        let values = VecDeque::from([0.0, 50.0, 100.0]);
        let points = chart_points(&values, rect, 100.0);
        assert_eq!(points.first().unwrap().x, rect.left());
        assert_eq!(points.last().unwrap().x, rect.right());
        assert_eq!(points.last().unwrap().y, rect.top());
    }

    #[test]
    fn worker_does_not_accumulate_unbounded_pending_snapshots() {
        let worker = Worker::start(500, egui::Context::default());
        // An occluded/minimized GUI may not poll for an arbitrarily long time.
        std::thread::sleep(Duration::from_millis(1_250));
        assert!(worker.snapshots.try_iter().count() <= 1);
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
                gpus: vec![],
                power: Default::default(),
                pressure: Default::default(),
            }),
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
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

    #[test]
    fn live_ricepack_reload_keeps_last_valid_pack_and_restores_missing_tokens() {
        use std::time::{SystemTime, UNIX_EPOCH};
        let path = std::env::temp_dir().join(format!(
            "taskotto-live-ricepack-{}-{}.toml",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let initial =
            RicePack::from_toml(include_str!("../examples/ricepacks/sea-glass.toml")).unwrap();
        std::fs::write(&path, include_str!("../examples/ricepacks/sea-glass.toml")).unwrap();
        let ctx = egui::Context::default();
        let mut saved = Settings {
            theme: Theme::Windows7,
            accent_hex: Some("#456789".to_owned()),
            refresh_ms: 4_000,
            ..Settings::default()
        };
        initial.apply(&mut saved);
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings: saved,
            snapshot: None,
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: Some(RicePackWatcher {
                path: path.clone(),
                applied: initial,
                forced_theme: None,
                next_check: Instant::now(),
                last_error: None,
            }),
            ricepack_library: RicePackLibraryUi::default(),
        };

        std::fs::write(&path, "format_version = 1\n[broken\n").unwrap();
        app.maybe_reload_ricepack(&ctx);
        assert_eq!(app.settings.theme, Theme::FrutigerAero);
        assert_eq!(app.settings.accent_hex.as_deref(), Some("#159DA4"));
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("reload failed")
        );

        std::fs::write(
            &path,
            "format_version = 1\nname = 'New Look'\nbase_theme = 'windows98'\n",
        )
        .unwrap();
        app.ricepack_watcher.as_mut().unwrap().next_check = Instant::now();
        app.maybe_reload_ricepack(&ctx);
        assert_eq!(app.settings.theme, Theme::Windows98);
        assert_eq!(app.settings.accent_hex.as_deref(), Some("#456789"));
        assert_eq!(app.settings.refresh_ms, 4_000);
        assert_eq!(app.page, Page::Applications);
        assert_eq!(
            app.settings.launch_ricepack_name.as_deref(),
            Some("New Look")
        );
        assert!(
            app.notification
                .as_deref()
                .unwrap()
                .contains("Reloaded RicePack")
        );

        app.ricepack_watcher.as_mut().unwrap().forced_theme = Some(Theme::Windows10);
        app.ricepack_watcher.as_mut().unwrap().next_check = Instant::now();
        std::fs::write(
            &path,
            "format_version = 1\nname = 'Another Look'\nbase_theme = 'macos'\n",
        )
        .unwrap();
        app.maybe_reload_ricepack(&ctx);
        assert_eq!(app.settings.theme, Theme::Windows10);
        assert_eq!(app.page, Page::Processes);

        app.ricepack_library.source_path = path.display().to_string();
        app.preview_ricepack(&ctx);
        assert_eq!(app.settings.theme, Theme::Macos);
        assert!(app.ricepack_watcher.is_none());
        assert!(app.ricepack_library.preview.is_some());
        app.settings.refresh_ms = 5_000;
        app.restore_ricepack_preview(&ctx);
        assert_eq!(app.settings.theme, Theme::Windows10);
        assert_eq!(app.settings.refresh_ms, 5_000);
        assert_eq!(app.ricepack_watcher.as_ref().unwrap().path, path);

        app.preview_ricepack(&ctx);
        app.keep_ricepack_preview();
        assert_eq!(app.settings.theme, Theme::Windows10); // Existing CLI override wins.
        assert_eq!(
            app.ricepack_watcher.as_ref().unwrap().applied.name,
            "Another Look"
        );
        assert!(app.ricepack_library.preview.is_none());

        app.settings.reset_ricepack_appearance();
        assert_eq!(app.settings.theme, Theme::Windows7);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn all_rice_profiles_layout_at_minimum_and_desktop_sizes() {
        let ctx = egui::Context::default();
        let mut settings = Settings::default();
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings: settings.clone(),
            snapshot: None,
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        };
        for theme in Theme::ALL {
            for dimensions in [(650.0, 420.0), (1440.0, 900.0)] {
                settings.theme = theme;
                app.settings = settings.clone();
                apply_theme(&ctx, &app.settings);
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(dimensions.0, dimensions.1),
                    )),
                    ..Default::default()
                };
                let _ = ctx.run(input, |ctx| {
                    app.draw_contents(ctx);
                });
            }
        }

        // A dark AI-authored surface can be used without optional per-widget
        // colors; light legacy menu/status bars still need readable captions.
        settings.surface_hex = Some("#101B2B".into());
        settings.text_hex = Some("#FFFFFF".into());
        for theme in Theme::ALL {
            settings.theme = theme;
            app.settings = settings.clone();
            apply_theme(&ctx, &app.settings);
            for (width, height) in [(650.0, 420.0), (1280.0, 800.0)] {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw_contents(ctx),
                );
                assert!(
                    !output.shapes.is_empty(),
                    "dark surface {theme:?} did not paint"
                );
            }
        }
    }

    #[test]
    fn historical_windows_rices_do_not_draw_duplicate_window_captions() {
        let ctx = egui::Context::default();
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings: Settings::default(),
            snapshot: None,
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        };
        for theme in [Theme::XpLuna, Theme::Windows7, Theme::Windows10] {
            app.settings.theme = theme;
            apply_theme(&ctx, &app.settings);
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(1100.0, 720.0),
                )),
                ..Default::default()
            };
            let mut menu_bottom = 0.0;
            let _ = ctx.run(input, |ctx| {
                rices::topbar(&mut app, ctx);
                menu_bottom = ctx.available_rect().top();
            });
            assert!(
                menu_bottom < 33.0,
                "{theme:?} consumed {menu_bottom}px above its menu: +                 the window caption must come from RiceBowl/the WM"
            );
        }
    }

    #[test]
    fn frutiger_aero_live_pages_render_at_two_sizes_and_both_navigations() {
        let ctx = egui::Context::default();
        let snapshot = Snapshot {
            cpu_percent: 32.0,
            cores_percent: vec![32.0, 8.0],
            memory: crate::monitor::Memory {
                total: 8 * 1024 * 1024 * 1024,
                available: 4 * 1024 * 1024 * 1024,
                swap_total: 0,
                swap_free: 0,
            },
            rates: Default::default(),
            uptime_seconds: 1_500.0,
            processes: vec![Process {
                pid: 5678,
                name: "sample-service".into(),
                command: "sample-service --foreground".into(),
                user_id: 1000,
                resident_bytes: 20 * 1024 * 1024,
                cpu_percent: 12.0,
                start_ticks: 77,
                ..Process::default()
            }],
            logical_cpus: 2,
            gpus: vec![],
            power: Default::default(),
            pressure: Default::default(),
        };
        let mut history = History::default();
        for _ in 0..4 {
            history.record(&snapshot);
        }
        let settings = Settings {
            theme: Theme::FrutigerAero,
            ..Settings::default()
        };
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings,
            snapshot: Some(snapshot),
            history,
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: Some((5678, 77)),
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        };
        assert_eq!(app.processes().len(), 1);
        app.query = "not-there".into();
        assert!(app.processes().is_empty());
        app.query.clear();
        for layout in [LayoutMode::TopTabs, LayoutMode::Sidebar] {
            app.settings.layout_mode = layout;
            apply_theme(&ctx, &app.settings);
            for page in [
                Page::Processes,
                Page::Performance,
                Page::Networking,
                Page::Users,
                Page::Services,
                Page::Details,
                Page::Preferences,
            ] {
                app.page = page;
                for dimensions in [(650.0, 420.0), (1280.0, 800.0)] {
                    let input = egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(dimensions.0, dimensions.1),
                        )),
                        ..Default::default()
                    };
                    let _ = ctx.run(input, |ctx| app.draw_contents(ctx));
                }
            }
        }
        app.settings.components = dark_component_tokens();
        apply_theme(&ctx, &app.settings);
        for page in [Page::Processes, Page::Performance, Page::Networking] {
            app.page = page;
            for dimensions in [(650.0, 420.0), (1280.0, 800.0)] {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(dimensions.0, dimensions.1),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw_contents(ctx),
                );
                assert!(
                    !output.shapes.is_empty(),
                    "dark Aero {page:?} did not paint"
                );
            }
        }
        assert!(app.pending.is_none());
    }

    #[test]
    fn macos_five_views_and_process_info_layout_at_two_sizes() {
        let ctx = egui::Context::default();
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings: Settings {
                theme: Theme::Macos,
                ..Settings::default()
            },
            snapshot: Some(Snapshot {
                cpu_percent: 24.0,
                cores_percent: vec![24.0, 5.0],
                memory: crate::monitor::Memory {
                    total: 8 * 1024 * 1024 * 1024,
                    available: 4 * 1024 * 1024 * 1024,
                    swap_total: 0,
                    swap_free: 0,
                },
                rates: Default::default(),
                uptime_seconds: 100.0,
                processes: vec![Process {
                    pid: 3300,
                    parent_pid: 1,
                    name: "example-worker".into(),
                    command: "/usr/bin/example-worker --test".into(),
                    resident_bytes: 90 * 1024 * 1024,
                    cpu_percent: 12.0,
                    ..Process::default()
                }],
                logical_cpus: 2,
                gpus: vec![],
                power: Default::default(),
                pressure: Default::default(),
            }),
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        };
        for layout in [LayoutMode::TopTabs, LayoutMode::Sidebar] {
            app.settings.layout_mode = layout;
            apply_theme(&ctx, &app.settings);
            for page in [
                Page::Processes,
                Page::Performance,
                Page::MacEnergy,
                Page::MacDisk,
                Page::Networking,
            ] {
                app.page = page;
                app.selected = Some((3300, 0));
                app.mac_info_open = page == Page::Processes;
                for (width, height) in [(650.0, 420.0), (1280.0, 800.0)] {
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                Vec2::new(width, height),
                            )),
                            ..Default::default()
                        },
                        |ctx| app.draw_contents(ctx),
                    );
                    assert!(
                        !output.shapes.is_empty(),
                        "missing macOS {page:?} {layout:?} paint commands"
                    );
                }
            }
        }

        // The secondary Activity Monitor panes and process inspector must
        // remain paintable with a generator-supplied dark table surface.
        app.settings.components = dark_component_tokens();
        apply_theme(&ctx, &app.settings);
        for page in [Page::Processes, Page::Performance, Page::Networking] {
            app.page = page;
            app.mac_info_open = page == Page::Processes;
            app.selected = Some((3300, 0));
            for (width, height) in [(650.0, 420.0), (1280.0, 800.0)] {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw_contents(ctx),
                );
                assert!(
                    !output.shapes.is_empty(),
                    "dark macOS {page:?} did not paint"
                );
            }
        }
    }

    #[test]
    fn windows_rices_real_data_pages_layout_at_minimum_and_desktop_sizes() {
        let ctx = egui::Context::default();
        let settings = Settings::default();
        let mut app = TaskottoApp {
            worker: Worker::start(10_000, ctx.clone()),
            settings,
            snapshot: Some(Snapshot {
                cpu_percent: 23.0,
                cores_percent: (0..24).map(|index| index as f32).collect(),
                memory: crate::monitor::Memory {
                    total: 16 * 1024 * 1024 * 1024,
                    available: 9 * 1024 * 1024 * 1024,
                    swap_total: 0,
                    swap_free: 0,
                },
                rates: Default::default(),
                uptime_seconds: 10_000.0,
                processes: vec![Process {
                    pid: 3300,
                    parent_pid: 1,
                    name: "example-worker".into(),
                    command: "/usr/bin/example-worker --test".into(),
                    resident_bytes: 90 * 1024 * 1024,
                    cpu_percent: 23.0,
                    ..Process::default()
                }],
                logical_cpus: 24,
                gpus: vec![],
                power: Default::default(),
                pressure: Default::default(),
            }),
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            frozen_sort_order: None,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: HashMap::new(),
            system_pages: rices::SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
            ricepack_watcher: None,
            ricepack_library: RicePackLibraryUi::default(),
        };
        for theme in [
            Theme::Windows98,
            Theme::XpLuna,
            Theme::Windows7,
            Theme::Windows10,
            Theme::Chromeos,
        ] {
            for &navigation in if theme == Theme::Chromeos {
                &[LayoutMode::Auto, LayoutMode::TopTabs, LayoutMode::Sidebar][..]
            } else if matches!(theme, Theme::Windows98 | Theme::Windows10) {
                &[LayoutMode::TopTabs, LayoutMode::Sidebar][..]
            } else {
                &[LayoutMode::TopTabs][..]
            } {
                app.settings.theme = theme;
                app.settings.layout_mode = navigation;
                apply_theme(&ctx, &app.settings);
                for page in [
                    Page::Applications,
                    Page::Processes,
                    Page::Performance,
                    Page::AppHistory,
                    Page::Startup,
                    Page::Networking,
                    Page::Users,
                    Page::Details,
                    Page::Services,
                    Page::Preferences,
                ] {
                    app.page = page;
                    app.selected = if page == Page::Details {
                        Some((3300, 0))
                    } else {
                        None
                    };
                    for focus in [
                        PerformanceFocus::Cpu,
                        PerformanceFocus::Memory,
                        PerformanceFocus::Disk,
                        PerformanceFocus::Network,
                    ] {
                        app.performance_focus = focus;
                        let sizes: &[(f32, f32)] = if theme == Theme::Windows98 {
                            &[(490.0, 340.0), (650.0, 450.0), (1280.0, 800.0)]
                        } else {
                            &[(650.0, 420.0), (1280.0, 800.0)]
                        };
                        for &(width, height) in sizes {
                            let output = ctx.run(
                                egui::RawInput {
                                    screen_rect: Some(egui::Rect::from_min_size(
                                        egui::Pos2::ZERO,
                                        Vec2::new(width, height),
                                    )),
                                    ..Default::default()
                                },
                                |ctx| app.draw_contents(ctx),
                            );
                            assert!(
                                !output.shapes.is_empty(),
                                "missing {theme:?} {page:?} paint commands"
                            );
                        }
                    }
                }
            }
        }

        // A completely new two-screen monitor is data-driven from TOML and
        // paints real procfs statistics without adding a theme-specific UI.
        let generated =
            RicePack::from_toml(include_str!("../examples/ricepacks/diagnostic-studio.toml"))
                .expect("validated declarative example");
        generated.apply(&mut app.settings);
        app.page = Page::Processes;
        apply_theme(&ctx, &app.settings);
        for screen in 0..2 {
            ctx.data_mut(|data| {
                data.insert_temp(egui::Id::new("taskotto-declarative-screen"), screen)
            });
            for (width, height) in [(650.0, 420.0), (1280.0, 800.0)] {
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(width, height),
                        )),
                        ..Default::default()
                    },
                    |ctx| app.draw_contents(ctx),
                );
                assert!(
                    !output.shapes.is_empty(),
                    "declarative screen {screen} did not paint"
                );
            }
        }

        // Keyboard Ctrl+F must focus a genuine process search field on both
        // authored toolbars and lean AI layouts without a toolbar widget.
        for with_toolbar in [true, false] {
            let mut pack = generated.clone();
            if !with_toolbar {
                let layout = pack.layout.as_mut().unwrap();
                for page in &mut layout.pages {
                    page.widgets
                        .retain(|widget| !matches!(widget, crate::layout::Widget::Toolbar { .. }));
                    page.compact_widgets
                        .retain(|widget| !matches!(widget, crate::layout::Widget::Toolbar { .. }));
                }
            }
            pack.apply(&mut app.settings);
            app.focus_search = true;
            ctx.data_mut(|data| {
                data.insert_temp(egui::Id::new("taskotto-declarative-screen"), 0_usize)
            });
            let _ = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(1280.0, 800.0),
                    )),
                    ..Default::default()
                },
                |ctx| app.draw_contents(ctx),
            );
            assert!(!app.focus_search, "Ctrl+F ignored in generated layout");
            assert!(
                ctx.memory(|memory| memory.focused().is_some()),
                "Ctrl+F did not focus a text input with_toolbar={with_toolbar}"
            );
        }

        // Secondary reports, performance metric tiles and the Windows 7
        // properties sheet should work with custom RicePack component colors.
        app.settings.components = dark_component_tokens();
        for theme in [
            Theme::Windows98,
            Theme::XpLuna,
            Theme::Windows7,
            Theme::Windows10,
        ] {
            app.settings.theme = theme;
            apply_theme(&ctx, &app.settings);
            for page in [
                Page::Performance,
                Page::Networking,
                Page::Users,
                Page::Services,
                Page::Details,
            ] {
                app.page = page;
                app.selected = Some((3300, 0));
                for (width, height) in [(650.0, 420.0), (1280.0, 800.0)] {
                    let output = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                Vec2::new(width, height),
                            )),
                            ..Default::default()
                        },
                        |ctx| app.draw_contents(ctx),
                    );
                    assert!(
                        !output.shapes.is_empty(),
                        "dark {theme:?} {page:?} did not paint"
                    );
                }
            }
        }
    }
}
