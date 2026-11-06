//! Asset-free macOS Activity Monitor-inspired client. The Linux sampler is
//! authoritative: unavailable macOS-only metrics are labeled, never simulated.
use super::*;
use egui::{Align2, FontId, Frame, Margin, StrokeKind};
use std::collections::VecDeque;

const WHITE: Color32 = Color32::WHITE;
const INK: Color32 = Color32::from_rgb(37, 43, 51);
const MUTED: Color32 = Color32::from_rgb(96, 105, 119);
const BORDER: Color32 = Color32::from_rgb(204, 210, 219);
const TOOLBAR: Color32 = Color32::from_rgb(238, 241, 246);
const BLUE: Color32 = Color32::from_rgb(48, 126, 218);
const GREEN: Color32 = Color32::from_rgb(45, 170, 115);
const ORANGE: Color32 = Color32::from_rgb(224, 151, 57);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Focus {
    Cpu,
    Memory,
    Energy,
    Disk,
    Network,
}

const TABS: [(Page, &str, Focus); 5] = [
    (Page::Processes, "CPU", Focus::Cpu),
    (Page::Performance, "Memory", Focus::Memory),
    (Page::MacEnergy, "Energy", Focus::Energy),
    (Page::MacDisk, "Disk", Focus::Disk),
    (Page::Networking, "Network", Focus::Network),
];

fn focus(page: Page) -> Focus {
    TABS.iter()
        .find(|(p, _, _)| *p == page)
        .map_or(Focus::Cpu, |(_, _, f)| *f)
}

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("mac-activity-toolbar")
        .frame(
            Frame::NONE
                .fill(TOOLBAR)
                .inner_margin(Margin::symmetric(12, 9)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // This is a toolbar, not another app caption. The compositor
                // renders the app icon/title and the window controls.
                if ui
                    .add_enabled(
                        app.selected_process().is_some_and(can_signal_process),
                        egui::Button::new(
                            RichText::new("Stop").color(Color32::from_rgb(163, 53, 57)),
                        ),
                    )
                    .on_hover_text("Quit the selected process (confirmation required)")
                    .clicked()
                {
                    app.queue_action(SignalAction::Terminate);
                }
                if ui
                    .add_enabled(app.selected_process().is_some(), egui::Button::new("Info"))
                    .on_hover_text("Inspect selected process")
                    .clicked()
                {
                    app.mac_info_open = true;
                }
                ui.menu_button("View", |ui| {
                    let mut all = !app.settings.only_my_processes;
                    if ui.checkbox(&mut all, "All Processes").changed() {
                        app.settings.only_my_processes = !all;
                        app.save_preferences(ctx);
                    }
                    if ui
                        .checkbox(&mut app.settings.tree_view, "All Processes, Hierarchically")
                        .changed()
                    {
                        app.save_preferences(ctx);
                    }
                    if ui
                        .checkbox(&mut app.settings.show_kernel_threads, "Show kernel threads")
                        .changed()
                    {
                        app.save_preferences(ctx);
                    }
                    ui.menu_button("Update Frequency", |ui| {
                        for (interval, name) in [
                            (500, "Very often"),
                            (1500, "Often"),
                            (5000, "Normal"),
                            (10000, "Less often"),
                        ] {
                            if ui
                                .selectable_label(app.settings.refresh_ms == interval, name)
                                .clicked()
                            {
                                app.settings.refresh_ms = interval;
                                app.save_preferences(ctx);
                                ui.close();
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Preferences…").clicked() {
                        app.page = Page::Preferences;
                        ui.close();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let search_width = ui.available_width().clamp(120.0, 260.0);
                    let response = ui.add_sized(
                        [search_width, 26.0],
                        egui::TextEdit::singleline(&mut app.query).hint_text("Search processes"),
                    );
                    if app.focus_search {
                        response.request_focus();
                        app.focus_search = false;
                    }
                });
            });
        });
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("mac-activity-segments")
        .frame(
            Frame::NONE
                .fill(TOOLBAR)
                .inner_margin(Margin::symmetric(12, 5)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                let width = (ui.available_width() / 5.0 - 1.0).clamp(72.0, 126.0);
                let total = width * TABS.len() as f32;
                ui.add_space((ui.available_width() - total).max(0.0) / 2.0);
                for (page, name, _) in TABS {
                    let active = app.page == page;
                    let button = egui::Button::new(RichText::new(name).size(12.5).color(INK))
                        .corner_radius(0)
                        .fill(if active {
                            WHITE
                        } else {
                            Color32::from_rgb(226, 230, 236)
                        })
                        .stroke(Stroke::new(1.0_f32, BORDER));
                    let response = ui.add_sized([width, 30.0], button);
                    if active {
                        ui.painter().line_segment(
                            [response.rect.left_bottom(), response.rect.right_bottom()],
                            Stroke::new(2.0_f32, BLUE),
                        );
                    }
                    if response.clicked() {
                        app.page = page;
                    }
                }
            });
        });
}

pub(super) fn vertical_navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    egui::SidePanel::left("mac-custom-sidebar")
        .resizable(false)
        .exact_width(147.0)
        .frame(
            Frame::NONE
                .fill(TOOLBAR)
                .inner_margin(Margin::symmetric(9, 12)),
        )
        .show(ctx, |ui| {
            ui.label(RichText::new("ACTIVITY").small().strong().color(MUTED));
            ui.add_space(7.0);
            for (page, name, _) in TABS {
                let active = app.page == page;
                if ui
                    .add_sized([128.0, 33.0], egui::Button::new(name).selected(active))
                    .clicked()
                {
                    app.page = page;
                }
                ui.add_space(2.0);
            }
            ui.separator();
            if ui.button("Preferences…").clicked() {
                app.page = Page::Preferences;
            }
        });
}

pub(super) fn statusbar(app: &TaskottoApp, ctx: &egui::Context) {
    let current = focus(app.page);
    egui::TopBottomPanel::bottom("mac-activity-summary")
        .resizable(false)
        .exact_height(133.0)
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(246, 247, 250))
                .inner_margin(Margin::symmetric(14, 11)),
        )
        .show(ctx, |ui| {
            ui.painter().line_segment(
                [ui.max_rect().left_top(), ui.max_rect().right_top()],
                Stroke::new(1.0_f32, BORDER),
            );
            ui.horizontal_top(|ui| {
                let available = ui.available_width();
                let chart_width = (available * 0.33).clamp(155.0, 350.0);
                ui.allocate_ui_with_layout(
                    egui::vec2(chart_width, 103.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let label = match current {
                            Focus::Cpu => "CPU LOAD",
                            Focus::Memory => "MEMORY USED",
                            Focus::Energy => "CPU ACTIVITY (ENERGY PROXY)",
                            Focus::Disk => "DISK ACTIVITY",
                            Focus::Network => "NETWORK ACTIVITY",
                        };
                        ui.label(RichText::new(label).size(10.0).color(MUTED).strong());
                        let series: Vec<(&VecDeque<f32>, Color32)> = match current {
                            Focus::Cpu | Focus::Energy => vec![(&app.history.cpu, BLUE)],
                            Focus::Memory => vec![(&app.history.memory, GREEN)],
                            Focus::Disk => vec![
                                (&app.history.disk_read, BLUE),
                                (&app.history.disk_write, ORANGE),
                            ],
                            Focus::Network => {
                                vec![(&app.history.received, BLUE), (&app.history.sent, ORANGE)]
                            }
                        };
                        let max = if matches!(current, Focus::Cpu | Focus::Memory | Focus::Energy) {
                            100.0
                        } else {
                            super::super::graph_max(series[0].0, series[1].0)
                        };
                        sparkline(ui, &series, max, chart_width - 2.0, 60.0);
                    },
                );
                ui.add_space(17.0);
                ui.separator();
                ui.add_space(14.0);
                if let Some(s) = &app.snapshot {
                    let stats = match current {
                        Focus::Cpu => vec![
                            ("CPU used", format!("{:.1}%", s.cpu_percent)),
                            ("Idle", format!("{:.1}%", 100.0 - s.cpu_percent.min(100.0))),
                            (
                                "Processes / threads",
                                format!(
                                    "{} / {}",
                                    s.processes.len(),
                                    s.processes.iter().map(|p| p.threads as u64).sum::<u64>()
                                ),
                            ),
                        ],
                        Focus::Memory => vec![
                            ("Physical Memory", format::bytes(s.memory.total)),
                            ("Memory Used", format::bytes(s.memory.used())),
                            (
                                "Swap Used",
                                format::bytes(
                                    s.memory.swap_total.saturating_sub(s.memory.swap_free),
                                ),
                            ),
                        ],
                        Focus::Energy => vec![
                            ("System CPU activity", format!("{:.1}%", s.cpu_percent)),
                            ("Energy Impact", "Not exposed by /proc".to_owned()),
                            ("Active processes", s.processes.len().to_string()),
                        ],
                        Focus::Disk => vec![
                            ("Data read/sec", format::rate(s.rates.disk_read_per_second)),
                            (
                                "Data written/sec",
                                format::rate(s.rates.disk_write_per_second),
                            ),
                            ("Scope", "Block devices".to_owned()),
                        ],
                        Focus::Network => vec![
                            (
                                "Data received/sec",
                                format::rate(s.rates.received_per_second),
                            ),
                            ("Data sent/sec", format::rate(s.rates.sent_per_second)),
                            ("Scope", "Non-loopback aggregate".to_owned()),
                        ],
                    };
                    egui::Grid::new("mac-summary-stats")
                        .num_columns(2)
                        .spacing([15.0, 8.0])
                        .show(ui, |ui| {
                            for (title, value) in stats {
                                ui.label(RichText::new(title).size(11.0).color(MUTED));
                                ui.label(RichText::new(value).size(11.0).strong().color(INK));
                                ui.end_row();
                            }
                        });
                } else {
                    ui.spinner();
                    ui.label("Reading Linux activity…");
                }
            });
        });
}

fn sparkline(
    ui: &mut egui::Ui,
    series: &[(&VecDeque<f32>, Color32)],
    max: f32,
    width: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width.max(115.0), height), Sense::hover());
    ui.painter().rect_filled(rect, 3.0, WHITE);
    ui.painter()
        .rect_stroke(rect, 3.0, Stroke::new(1.0_f32, BORDER), StrokeKind::Inside);
    for level in [0.25, 0.5, 0.75] {
        let y = rect.bottom() - rect.height() * level;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(0.5_f32, Color32::from_rgb(232, 235, 241)),
        );
    }
    for &(data, color) in series {
        if data.len() >= 2 {
            let points = super::super::chart_points(data, rect.shrink(2.0), max.max(1.0));
            ui.painter()
                .add(egui::Shape::line(points, Stroke::new(1.8_f32, color)));
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Column {
    Name,
    Cpu,
    CpuTime,
    Threads,
    Memory,
    Virtual,
    Pid,
    User,
    Read,
    Written,
    ReadRate,
    WriteRate,
}

impl Column {
    fn name(self) -> &'static str {
        match self {
            Self::Name => "Process Name",
            Self::Cpu => "% CPU",
            Self::CpuTime => "CPU Time",
            Self::Threads => "Threads",
            Self::Memory => "Memory",
            Self::Virtual => "Virtual Mem",
            Self::Pid => "PID",
            Self::User => "User",
            Self::Read => "Bytes Read",
            Self::Written => "Bytes Written",
            Self::ReadRate => "Read/sec",
            Self::WriteRate => "Write/sec",
        }
    }
    fn sort(self) -> Option<SortBy> {
        match self {
            Self::Name => Some(SortBy::Name),
            Self::Cpu => Some(SortBy::Cpu),
            Self::Memory => Some(SortBy::Memory),
            Self::Pid => Some(SortBy::Pid),
            Self::Read | Self::Written | Self::ReadRate | Self::WriteRate => Some(SortBy::Io),
            _ => None,
        }
    }
}

fn columns(focus: Focus, width: f32, settings: &Settings) -> Vec<(Column, f32)> {
    let mut fixed: Vec<(Column, f32)> = match focus {
        Focus::Cpu => vec![
            (Column::Cpu, 76.0),
            (Column::Threads, 75.0),
            (Column::Pid, 68.0),
        ],
        Focus::Memory => vec![
            (Column::Memory, 105.0),
            (Column::Threads, 75.0),
            (Column::Pid, 68.0),
        ],
        Focus::Energy => vec![
            (Column::Cpu, 96.0),
            (Column::Threads, 75.0),
            (Column::Pid, 68.0),
        ],
        Focus::Disk if width < 560.0 => vec![
            (Column::Read, 80.0),
            (Column::Written, 80.0),
            (Column::Pid, 64.0),
        ],
        Focus::Disk => vec![
            (Column::Read, 110.0),
            (Column::Written, 110.0),
            (Column::Pid, 68.0),
        ],
        Focus::Network => unreachable!("network has an aggregate view"),
    };
    if width >= 780.0 && focus == Focus::Cpu {
        fixed.insert(2, (Column::CpuTime, 105.0));
    }
    if width >= 830.0 && focus == Focus::Memory {
        fixed.insert(2, (Column::Virtual, 110.0));
    }
    if width >= 920.0 && focus == Focus::Disk {
        fixed.insert(2, (Column::ReadRate, 108.0));
        fixed.insert(3, (Column::WriteRate, 108.0));
    }
    if width >= 1000.0 && settings.show_user_column && focus != Focus::Disk {
        fixed.push((Column::User, 120.0));
    }
    let rest = (width - fixed.iter().map(|(_, w)| w).sum::<f32>()).max(160.0);
    let mut result = vec![(Column::Name, rest)];
    result.extend(fixed);
    result
}

fn cell(
    ui: &mut egui::Ui,
    text: &str,
    width: f32,
    row: usize,
    selected: bool,
    height: f32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    let background = if selected {
        Color32::from_rgb(47, 119, 212)
    } else if response.hovered() {
        Color32::from_rgb(231, 240, 253)
    } else if row.is_multiple_of(2) {
        WHITE
    } else {
        Color32::from_rgb(247, 248, 250)
    };
    ui.painter().rect_filled(rect, 0.0, background);
    ui.painter().with_clip_rect(rect.shrink(4.0)).text(
        rect.left_center() + egui::vec2(9.0, 0.0),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(12.3),
        if selected { WHITE } else { INK },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            0.0,
            Stroke::new(1.3_f32, if selected { WHITE } else { BLUE }),
            StrokeKind::Inside,
        );
    }
    response
}

fn column_value(column: Column, process: &Process, app: &TaskottoApp) -> String {
    match column {
        Column::Name => process.name.clone(),
        Column::Cpu => format!("{:.1}", process.cpu_percent),
        Column::CpuTime => {
            let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as u64;
            let time = process.total_ticks / hz;
            format!("{}:{:02}:{:02}", time / 3600, time / 60 % 60, time % 60)
        }
        Column::Threads => process.threads.to_string(),
        Column::Memory => format::bytes(process.resident_bytes),
        Column::Virtual => format::bytes(process.virtual_bytes),
        Column::Pid => process.pid.to_string(),
        Column::User => app
            .users
            .get(&process.user_id)
            .cloned()
            .unwrap_or_else(|| process.user_id.to_string()),
        Column::Read => {
            if process.io_accessible {
                format::bytes(process.read_bytes)
            } else {
                "N/A".into()
            }
        }
        Column::Written => {
            if process.io_accessible {
                format::bytes(process.written_bytes)
            } else {
                "N/A".into()
            }
        }
        Column::ReadRate => {
            if process.io_accessible {
                format::rate(process.read_per_second)
            } else {
                "N/A".into()
            }
        }
        Column::WriteRate => {
            if process.io_accessible {
                format::rate(process.written_per_second)
            } else {
                "N/A".into()
            }
        }
    }
}

pub(super) fn process_table(app: &mut TaskottoApp, ui: &mut egui::Ui, focus: Focus) {
    let Some(_) = &app.snapshot else {
        ui.spinner();
        ui.label("Reading live Linux processes…");
        return;
    };
    let rows = app.processes();
    let note = match focus {
        Focus::Cpu => {
            "Linux process CPU is sampled from /proc; CPU Time is accumulated process time."
        }
        Focus::Memory => {
            "Memory is resident set size (RSS), not macOS compressed or private app memory."
        }
        Focus::Energy => {
            "Energy Impact and App Nap have no portable Linux equivalent. % CPU shows actual activity, not watts."
        }
        Focus::Disk => {
            "Per-process disk data is permission-limited /proc/PID/io; N/A means unavailable."
        }
        Focus::Network => unreachable!(),
    };
    let height = (ui.available_height() - 26.0).max(60.0);
    Frame::NONE
        .fill(WHITE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .show(ui, |ui| {
            ui.set_min_height(height);
            let width = (ui.available_width() - 13.0).max(395.0);
            let specs = columns(focus, width, &app.settings);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for &(kind, width) in &specs {
                    let title = if kind.sort().is_some_and(|s| s == app.settings.sort_by) {
                        format!(
                            "{} {}",
                            kind.name(),
                            if app.settings.descending { "v" } else { "^" }
                        )
                    } else {
                        kind.name().to_owned()
                    };
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(width, 27.0), Sense::click());
                    ui.painter()
                        .rect_filled(rect, 0.0, Color32::from_rgb(243, 245, 248));
                    ui.painter().line_segment(
                        [rect.left_bottom(), rect.right_bottom()],
                        Stroke::new(1.0_f32, BORDER),
                    );
                    ui.painter().line_segment(
                        [rect.right_top(), rect.right_bottom()],
                        Stroke::new(0.7_f32, BORDER),
                    );
                    ui.painter().with_clip_rect(rect.shrink(4.0)).text(
                        rect.left_center() + egui::vec2(9.0, 0.0),
                        Align2::LEFT_CENTER,
                        title,
                        FontId::proportional(11.5),
                        MUTED,
                    );
                    if response.clicked()
                        && let Some(sort) = kind.sort()
                    {
                        if app.settings.sort_by == sort {
                            app.settings.descending = !app.settings.descending;
                        } else {
                            app.settings.sort_by = sort;
                            app.settings.descending = false;
                        }
                        app.save_preferences(ui.ctx());
                    }
                }
            });
            if rows.is_empty() {
                ui.add_space(34.0);
                ui.vertical_centered(|ui| {
                    ui.label("No processes match the current search or filter.");
                    if ui.button("Clear search").clicked() {
                        app.query.clear();
                    }
                });
                return;
            }
            let row_h = app.settings.table_row_height.unwrap_or(27.0).max(22.0);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height((height - 29.0).max(42.0))
                .show_rows(ui, row_h, rows.len(), |ui, range| {
                    for index in range {
                        let process = &rows[index].process;
                        let selected = app.selected == Some((process.pid, process.start_ticks));
                        let mut response = None;
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            for &(kind, width) in &specs {
                                let value = column_value(kind, process, app);
                                let cell_response = cell(ui, &value, width, index, selected, row_h);
                                if kind == Column::Name
                                    || cell_response.clicked()
                                    || cell_response.double_clicked()
                                    || cell_response.secondary_clicked()
                                {
                                    response = Some(cell_response);
                                }
                            }
                        });
                        if let Some(response) = response {
                            let response = response.on_hover_text(&process.command);
                            if response.clicked() || response.double_clicked() {
                                app.selected = Some((process.pid, process.start_ticks));
                            }
                            if response.double_clicked() {
                                app.mac_info_open = true;
                            }
                            response.context_menu(|ui| {
                                if ui.button("Get Info").clicked() {
                                    app.selected = Some((process.pid, process.start_ticks));
                                    app.mac_info_open = true;
                                    ui.close();
                                }
                                for (label, action) in [
                                    ("Quit Process…", SignalAction::Terminate),
                                    ("Force Quit…", SignalAction::ForceKill),
                                    ("Pause Process…", SignalAction::Stop),
                                    ("Resume Process…", SignalAction::Continue),
                                ] {
                                    if ui
                                        .add_enabled(
                                            can_signal_process(process),
                                            egui::Button::new(label),
                                        )
                                        .clicked()
                                    {
                                        app.selected = Some((process.pid, process.start_ticks));
                                        app.queue_action(action);
                                        ui.close();
                                    }
                                }
                            });
                        }
                    }
                });
        });
    ui.label(RichText::new(note).size(10.5).color(MUTED));
}

pub(super) fn network_page(app: &TaskottoApp, ui: &mut egui::Ui) {
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        ui.label("Reading network activity…");
        return;
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label(RichText::new("Network Activity").size(17.0).strong().color(INK));
        ui.label(RichText::new("Linux-wide non-loopback interfaces · true rate samples, not per-process networking").small().color(MUTED));
        ui.add_space(11.0);
        let max = super::super::graph_max(&app.history.received, &app.history.sent);
        sparkline(ui, &[(&app.history.received, BLUE), (&app.history.sent, ORANGE)],
            max, ui.available_width() - 5.0, ui.available_height().clamp(125.0, 310.0) * 0.62);
        ui.add_space(16.0);
        egui::Grid::new("mac-network-details").num_columns(2).spacing([32.0, 12.0]).show(ui, |ui| {
            for (label, value) in [
                ("Data received/sec", format::rate(snapshot.rates.received_per_second)),
                ("Data sent/sec", format::rate(snapshot.rates.sent_per_second)),
                ("Sample history", app.history.received.len().to_string()),
                ("Interfaces", "Aggregate, excluding loopback".to_owned()),
            ] {
                ui.label(RichText::new(label).color(MUTED)); ui.label(value); ui.end_row();
            }
        });
        ui.add_space(16.0);
        ui.label(RichText::new("Taskotto does not attribute aggregate /proc/net/dev counters to individual PIDs. Virtual adapters may double-count traffic.").small().color(MUTED));
    });
}

pub(super) fn info_dialog(app: &mut TaskottoApp, ctx: &egui::Context) {
    let mut open = app.mac_info_open;
    egui::Window::new("Process Information")
        .id(egui::Id::new("mac-process-information"))
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(485.0)
        .resizable(true)
        .open(&mut open)
        .frame(
            Frame::window(&ctx.style())
                .fill(Color32::from_rgb(248, 249, 251))
                .stroke(Stroke::new(1.0_f32, BORDER))
                .inner_margin(Margin::same(13)),
        )
        .show(ctx, |ui| {
            let Some(process) = app.selected_process().cloned() else {
                ui.label("The selected process has exited or is no longer available.");
                return;
            };
            ui.label(RichText::new(&process.name).size(20.0).strong().color(INK));
            let command_preview: String = process.command.chars().take(95).collect();
            let command_preview = if process.command.chars().nth(95).is_some() {
                format!("{command_preview}…")
            } else {
                command_preview
            };
            ui.label(RichText::new(command_preview).small().color(MUTED))
                .on_hover_text(&process.command);
            ui.separator();
            egui::Grid::new("mac-info-actual-fields")
                .num_columns(2)
                .spacing([28.0, 9.0])
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Process ID", process.pid.to_string()),
                        ("Parent PID", process.parent_pid.to_string()),
                        (
                            "User",
                            app.users
                                .get(&process.user_id)
                                .cloned()
                                .unwrap_or_else(|| process.user_id.to_string()),
                        ),
                        ("Status", state_name(process.state).to_owned()),
                        ("CPU", format!("{:.1}%", process.cpu_percent)),
                        ("Real Memory (RSS)", format::bytes(process.resident_bytes)),
                        ("Virtual Memory", format::bytes(process.virtual_bytes)),
                        ("Threads", process.threads.to_string()),
                        ("Running Time", format::process_age(process.elapsed_seconds)),
                        (
                            "Read bytes",
                            if process.io_accessible {
                                format::bytes(process.read_bytes)
                            } else {
                                "Unavailable".into()
                            },
                        ),
                        (
                            "Written bytes",
                            if process.io_accessible {
                                format::bytes(process.written_bytes)
                            } else {
                                "Unavailable".into()
                            },
                        ),
                    ] {
                        ui.label(RichText::new(label).color(MUTED));
                        ui.label(value);
                        ui.end_row();
                    }
                });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_signal_process(&process), egui::Button::new("Quit…"))
                    .clicked()
                {
                    app.queue_action(SignalAction::Terminate);
                }
                if ui
                    .add_enabled(
                        can_signal_process(&process),
                        egui::Button::new("Force Quit…"),
                    )
                    .clicked()
                {
                    app.queue_action(SignalAction::ForceKill);
                }
            });
            ui.label(
                RichText::new("Linux signals require confirmation; no automatic elevation.")
                    .small()
                    .color(MUTED),
            );
        });
    app.mac_info_open = open;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn activity_monitor_has_five_historical_sections() {
        assert_eq!(
            TABS.map(|(_, title, _)| title),
            ["CPU", "Memory", "Energy", "Disk", "Network"]
        );
    }

    #[test]
    fn responsive_report_columns_preserve_name_and_fit() {
        let settings = Settings::default();
        for section in [Focus::Cpu, Focus::Memory, Focus::Energy, Focus::Disk] {
            for width in [420.0, 650.0, 850.0, 1230.0] {
                let cells = columns(section, width, &settings);
                assert_eq!(cells[0].0, Column::Name);
                assert!(cells.iter().all(|(_, size)| *size >= 60.0));
                let total: f32 = cells.iter().map(|(_, size)| size).sum();
                assert!(
                    (total - width).abs() < 0.01,
                    "section {section:?}, width {width}: {total}"
                );
            }
        }
    }

    #[test]
    fn missing_io_is_not_reported_as_zero() {
        let process = Process {
            io_accessible: false,
            ..Default::default()
        };
        let ctx = egui::Context::default();
        let app = TaskottoApp {
            settings: Settings::default(),
            worker: Worker::start(10_000, ctx),
            snapshot: None,
            history: History::default(),
            page: Page::Processes,
            performance_focus: PerformanceFocus::Cpu,
            query: String::new(),
            focus_search: false,
            selected: None,
            pending: None,
            notification: None,
            theme_accent_input: String::new(),
            theme_surface_input: String::new(),
            theme_text_input: String::new(),
            my_uid: 1000,
            users: Default::default(),
            system_pages: SystemPages::default(),
            new_task: NewTaskDialog::default(),
            mac_info_open: false,
        };
        assert_eq!(column_value(Column::ReadRate, &process, &app), "N/A");
    }
}
