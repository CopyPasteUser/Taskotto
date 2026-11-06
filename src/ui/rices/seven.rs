//! Original, asset-free recreation of the Windows 7-era Task Manager client UI.
//! Historical chrome and report layout; every measurement/action remains Linux-native.
use super::*;
use egui::{Align2, FontId, Frame, Margin, Rect, StrokeKind};
use std::{collections::BTreeMap, collections::VecDeque};

const INK: Color32 = Color32::from_rgb(35, 45, 57);
const BORDER: Color32 = Color32::from_rgb(145, 162, 178);
const PALE: Color32 = Color32::from_rgb(237, 245, 250);
const GRID: Color32 = Color32::from_rgb(24, 84, 31);
const GRID_BLUE: Color32 = Color32::from_rgb(24, 74, 106);
const BLACK: Color32 = Color32::from_rgb(2, 13, 4);
const GREEN: Color32 = Color32::from_rgb(61, 237, 78);
const BLUE: Color32 = Color32::from_rgb(64, 182, 246);

const TABS: [(Page, &str); 6] = [
    (Page::Applications, "Applications"),
    (Page::Processes, "Processes"),
    (Page::Services, "Services"),
    (Page::Performance, "Performance"),
    (Page::Networking, "Networking"),
    (Page::Users, "Users"),
];

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let accent = accent(app);
    egui::TopBottomPanel::top("seven-aero-title")
        .frame(Frame::NONE)
        .show(ctx, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 38.0), Sense::hover());
            // An original drawn glass-like Aero title, not a copied bitmap.
            for line in 0..38 {
                let t = line as f32 / 37.0;
                let top = if t < 0.5 {
                    blend(
                        Color32::from_rgb(203, 232, 249),
                        Color32::from_rgb(149, 200, 228),
                        t * 2.0,
                    )
                } else {
                    blend(
                        Color32::from_rgb(149, 200, 228),
                        Color32::from_rgb(194, 224, 242),
                        (t - 0.5) * 2.0,
                    )
                };
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        rect.min + egui::vec2(0.0, line as f32),
                        egui::vec2(rect.width(), 1.1),
                    ),
                    0.0,
                    blend(top, accent, 0.13),
                );
            }
            ui.painter().line_segment(
                [
                    rect.left_top() + egui::vec2(0.0, 1.0),
                    rect.right_top() + egui::vec2(0.0, 1.0),
                ],
                Stroke::new(1.0_f32, Color32::WHITE),
            );
            ui.painter().line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                Stroke::new(1.0_f32, Color32::from_rgb(104, 151, 181)),
            );
            let badge =
                Rect::from_min_size(rect.min + egui::vec2(11.0, 9.0), egui::vec2(19.0, 19.0));
            ui.painter()
                .rect_filled(badge, 3.0, Color32::from_rgb(239, 250, 253));
            ui.painter().rect_stroke(
                badge,
                3.0,
                Stroke::new(1.0_f32, Color32::from_rgb(91, 142, 168)),
                StrokeKind::Inside,
            );
            for (i, level) in [8.0, 12.0, 6.0].iter().enumerate() {
                ui.painter().rect_filled(
                    Rect::from_min_size(
                        egui::pos2(
                            badge.left() + 4.0 + 4.0 * i as f32,
                            badge.bottom() - 3.0 - level,
                        ),
                        egui::vec2(2.0, *level),
                    ),
                    0.0,
                    Color32::from_rgb(50, 145, 185),
                );
            }
            // The light outline is important on the bright Aero gradient.
            ui.painter().text(
                rect.min + egui::vec2(37.0, 19.0),
                Align2::LEFT_CENTER,
                "Taskotto Task Manager",
                FontId::proportional(14.0),
                Color32::WHITE,
            );
            ui.painter().text(
                rect.min + egui::vec2(36.0, 18.0),
                Align2::LEFT_CENTER,
                "Taskotto Task Manager",
                FontId::proportional(14.0),
                INK,
            );
        });
    chrome::menu_bar(app, ctx);
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("seven-era-tabs")
        .frame(Frame::NONE.fill(PALE).inner_margin(Margin::symmetric(9, 3)))
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let width = (ui.available_width() / 6.0 - 2.0).clamp(66.0, 115.0);
                for (page, title) in TABS {
                    let selected =
                        app.page == page || (page == Page::Processes && app.page == Page::Details);
                    let response = ui.add_sized(
                        [width, 27.0],
                        egui::Button::new(RichText::new(title).size(12.5).color(INK))
                            .corner_radius(egui::CornerRadius {
                                nw: 3,
                                ne: 3,
                                sw: 0,
                                se: 0,
                            })
                            .fill(if selected {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(223, 234, 242)
                            })
                            .stroke(Stroke::new(1.0_f32, BORDER)),
                    );
                    if selected {
                        let rect = response.rect;
                        ui.painter().line_segment(
                            [
                                rect.left_bottom() + egui::vec2(1.0, 0.0),
                                rect.right_bottom() + egui::vec2(-1.0, 0.0),
                            ],
                            Stroke::new(2.0_f32, Color32::WHITE),
                        );
                    }
                    if response.clicked() {
                        app.page = page;
                    }
                }
            });
        });
}

pub(super) fn statusbar(app: &TaskottoApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("seven-status")
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(232, 240, 245))
                .inner_margin(Margin::symmetric(10, 5)),
        )
        .show(ctx, |ui| {
            ui.painter().line_segment(
                [ui.max_rect().left_top(), ui.max_rect().right_top()],
                Stroke::new(1.0_f32, BORDER),
            );
            if let Some(s) = &app.snapshot {
                ui.horizontal(|ui| {
                    ui.label(format!("Processes: {}", s.processes.len()));
                    ui.separator();
                    ui.label(format!("CPU Usage: {:.0}%", s.cpu_percent));
                    ui.separator();
                    ui.label(format!(
                        "Physical Memory: {:.0}%",
                        s.memory.used_fraction() * 100.0
                    ));
                });
            } else {
                ui.label("Reading live Linux system status…");
            }
        });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReportColumn {
    Name,
    User,
    Cpu,
    Memory,
    Description,
    Pid,
    Io,
}

impl ReportColumn {
    fn title(self) -> &'static str {
        match self {
            Self::Name => "Image Name",
            Self::User => "User Name",
            Self::Cpu => "CPU",
            Self::Memory => "Memory (Working Set)",
            Self::Description => "Description",
            Self::Pid => "PID",
            Self::Io => "I/O rate",
        }
    }
    fn sort(self) -> Option<SortBy> {
        match self {
            Self::Name => Some(SortBy::Name),
            Self::Cpu => Some(SortBy::Cpu),
            Self::Memory => Some(SortBy::Memory),
            Self::Pid => Some(SortBy::Pid),
            Self::Io => Some(SortBy::Io),
            Self::User | Self::Description => None,
        }
    }
}

fn process_columns(width: f32, settings: &Settings) -> Vec<(ReportColumn, f32)> {
    let mut fixed = vec![
        (ReportColumn::Cpu, if width < 600.0 { 50.0 } else { 55.0 }),
        (
            ReportColumn::Memory,
            if width < 600.0 { 115.0 } else { 150.0 },
        ),
    ];
    if width >= 600.0 && settings.show_user_column {
        fixed.insert(0, (ReportColumn::User, 125.0));
    }
    if width >= 960.0 {
        fixed.push((ReportColumn::Pid, 65.0));
    }
    if width >= 1110.0 && settings.show_io_column {
        fixed.push((ReportColumn::Io, 108.0));
    }
    let rest = (width - fixed.iter().map(|(_, w)| w).sum::<f32>()).max(260.0);
    let name = (rest * 0.48).max(125.0);
    let mut result = vec![(ReportColumn::Name, name)];
    result.extend(fixed);
    result.push((ReportColumn::Description, rest - name));
    result
}

fn header_cell(ui: &mut egui::Ui, label: &str, width: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 24.0), Sense::click());
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_rgb(238, 244, 248));
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0_f32, Color32::WHITE),
    );
    ui.painter().line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0_f32, BORDER),
    );
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, BORDER),
    );
    ui.painter().with_clip_rect(rect.shrink(3.0)).text(
        rect.left_center() + egui::vec2(6.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.5),
        INK,
    );
    response
}

fn row_cell(
    ui: &mut egui::Ui,
    value: &str,
    width: f32,
    shade: Color32,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 22.0), Sense::click());
    let fill = if selected {
        Color32::from_rgb(64, 124, 188)
    } else if response.hovered() {
        Color32::from_rgb(224, 237, 247)
    } else {
        shade
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    ui.painter().with_clip_rect(rect.shrink(3.0)).text(
        rect.left_center() + egui::vec2(6.0, 0.0),
        Align2::LEFT_CENTER,
        value,
        FontId::proportional(12.0),
        if selected { Color32::WHITE } else { INK },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            0.0,
            Stroke::new(1.0_f32, accent_from_ui(ui)),
            StrokeKind::Inside,
        );
    }
    response
}

fn accent_from_ui(ui: &egui::Ui) -> Color32 {
    ui.visuals().selection.bg_fill
}

pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    find_strip(app, ui);
    let rows = app.processes();
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading Linux processes…");
        return;
    }
    let footer = 59.0;
    let height = (ui.available_height() - footer).max(88.0);
    report_frame(ui, height, |ui| {
        let width = (ui.available_width() - 14.0).max(430.0);
        let cols = process_columns(width, &app.settings);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for &(column, width) in &cols {
                let title = if app.settings.sort_by == column.sort().unwrap_or(SortBy::Name)
                    && column.sort().is_some()
                {
                    if app.settings.descending {
                        format!("{} v", column.title())
                    } else {
                        format!("{} ^", column.title())
                    }
                } else {
                    column.title().to_owned()
                };
                if header_cell(ui, &title, width)
                    .on_hover_text("Click to sort")
                    .clicked()
                    && let Some(sort) = column.sort()
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
            ui.add_space(20.0);
            ui.label("No matching Linux processes.");
            if ui.button("Clear filter").clicked() {
                app.query.clear();
            }
            return;
        }
        let row_height = app.settings.table_row_height.unwrap_or(22.0).max(22.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((height - 26.0).max(48.0))
            .show_rows(ui, row_height, rows.len(), |ui, range| {
                for index in range {
                    let p = &rows[index].process;
                    let selected = app.selected == Some((p.pid, p.start_ticks));
                    let shade = if index % 2 == 0 {
                        Color32::WHITE
                    } else {
                        Color32::from_rgb(251, 253, 254)
                    };
                    let user = app
                        .users
                        .get(&p.user_id)
                        .cloned()
                        .unwrap_or_else(|| p.user_id.to_string());
                    let mut response = None;
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for &(column, width) in &cols {
                            let value = match column {
                                ReportColumn::Name => p.name.clone(),
                                ReportColumn::User => user.clone(),
                                ReportColumn::Cpu => format!("{:.0}", p.cpu_percent),
                                ReportColumn::Memory => format::bytes(p.resident_bytes),
                                ReportColumn::Description => p.command.clone(),
                                ReportColumn::Pid => p.pid.to_string(),
                                ReportColumn::Io => {
                                    if p.io_accessible {
                                        format::rate(
                                            p.read_per_second.saturating_add(p.written_per_second),
                                        )
                                    } else {
                                        "N/A".into()
                                    }
                                }
                            };
                            let cell = row_cell(ui, &value, width, shade, selected);
                            if cell.clicked()
                                || cell.double_clicked()
                                || column == ReportColumn::Name
                            {
                                response = Some(cell);
                            }
                        }
                    });
                    if let Some(response) = response {
                        let response = response.on_hover_text(&p.command);
                        if response.clicked() || response.double_clicked() {
                            app.selected = Some((p.pid, p.start_ticks));
                        }
                        if response.double_clicked() {
                            app.page = Page::Details;
                        }
                        response.context_menu(|ui| {
                            if ui.button("Properties…").clicked() {
                                app.selected = Some((p.pid, p.start_ticks));
                                app.page = Page::Details;
                                ui.close();
                            }
                            if ui
                                .add_enabled(
                                    can_signal_process(p),
                                    egui::Button::new("End Process…"),
                                )
                                .clicked()
                            {
                                app.selected = Some((p.pid, p.start_ticks));
                                app.queue_action(SignalAction::Terminate);
                                ui.close();
                            }
                            for (label, signal) in [
                                ("Suspend…", SignalAction::Stop),
                                ("Resume…", SignalAction::Continue),
                                ("Force end process…", SignalAction::ForceKill),
                            ] {
                                if ui
                                    .add_enabled(can_signal_process(p), egui::Button::new(label))
                                    .clicked()
                                {
                                    app.selected = Some((p.pid, p.start_ticks));
                                    app.queue_action(signal);
                                    ui.close();
                                }
                            }
                        });
                    }
                }
            });
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let mut all = !app.settings.only_my_processes;
        if ui
            .checkbox(&mut all, "Show processes from all users")
            .changed()
        {
            app.settings.only_my_processes = !all;
            app.save_preferences(ui.ctx());
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    app.selected_process().is_some_and(can_signal_process),
                    egui::Button::new("End Process"),
                )
                .clicked()
            {
                app.queue_action(SignalAction::Terminate);
            }
        });
    });
    ui.label(RichText::new("Description shows the Linux command line; memory is RSS (not Windows private working set).").size(10.0).weak());
}

fn find_strip(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.focus_search || !app.query.is_empty() {
        ui.horizontal(|ui| {
            let search = ui.add_sized(
                [ui.available_width().min(320.0), 23.0],
                egui::TextEdit::singleline(&mut app.query).hint_text("Find a process (Ctrl+F)"),
            );
            if app.focus_search {
                search.request_focus();
                app.focus_search = false;
            }
            if ui.small_button("Clear").clicked() {
                app.query.clear();
            }
        });
    }
}

fn report_frame(ui: &mut egui::Ui, height: f32, contents: impl FnOnce(&mut egui::Ui)) {
    let width = ui.available_width();
    Frame::NONE
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, BORDER))
        .inner_margin(Margin::same(1))
        .show(ui, |ui| {
            ui.set_min_height(height);
            ui.set_min_width(width - 2.0);
            contents(ui);
        });
}

pub(super) fn applications(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading current-user processes…");
        return;
    }
    let rows: Vec<_> = app
        .processes()
        .into_iter()
        .filter(|r| r.process.user_id == app.my_uid)
        .collect();
    let height = (ui.available_height() - 69.0).max(100.0);
    report_frame(ui, height, |ui| {
        let report_width = ui.available_width();
        let task_width = report_width * 0.71;
        let status_width = report_width * 0.28;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            header_cell(ui, "Task", task_width);
            header_cell(ui, "Status", status_width);
        });
        if rows.is_empty() {
            ui.label("No current-user Linux processes match this view.");
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height((height - 27.0).max(55.0))
            .show_rows(ui, 24.0, rows.len(), |ui, range| {
                for i in range {
                    let p = &rows[i].process;
                    let selected = app.selected == Some((p.pid, p.start_ticks));
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let action = row_cell(ui, &p.name, task_width, Color32::WHITE, selected);
                        row_cell(
                            ui,
                            state_name(p.state),
                            status_width,
                            Color32::WHITE,
                            selected,
                        );
                        if action.clicked() {
                            app.selected = Some((p.pid, p.start_ticks));
                        }
                        if action.double_clicked() {
                            app.selected = Some((p.pid, p.start_ticks));
                            app.page = Page::Details;
                        }
                    });
                }
            });
    });
    ui.add_space(6.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("New Task…").clicked() {
            app.new_task.open = true;
            app.new_task.error = None;
        }
        ui.add_enabled(false, egui::Button::new("Switch To"))
            .on_disabled_hover_text("Window activation requires compositor-specific support.");
        if ui
            .add_enabled(
                app.selected_process().is_some_and(can_signal_process),
                egui::Button::new("End Task"),
            )
            .clicked()
        {
            app.queue_action(SignalAction::Terminate);
        }
    });
    ui.label(
        RichText::new("Current-user Linux processes, not a Wayland/X11 graphical window list.")
            .size(10.0)
            .weak(),
    );
}

fn usage_meter(ui: &mut egui::Ui, title: &str, value: f32, color: Color32, height: f32) {
    ui.label(RichText::new(title).size(12.0).strong());
    let (rect, _) = ui.allocate_exact_size(egui::vec2(112.0, height), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, BLACK);
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0_f32, BORDER), StrokeKind::Inside);
    let fill_height = (rect.height() - 8.0) * (value / 100.0).clamp(0.0, 1.0);
    if fill_height > 0.0 {
        ui.painter().rect_filled(
            Rect::from_min_max(
                egui::pos2(rect.left() + 5.0, rect.bottom() - 4.0 - fill_height),
                egui::pos2(rect.right() - 5.0, rect.bottom() - 4.0),
            ),
            0.0,
            color,
        );
    }
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        format!("{value:.0}%"),
        FontId::proportional(18.0),
        Color32::WHITE,
    );
}

fn history_plot(
    ui: &mut egui::Ui,
    title: &str,
    series: &[(&VecDeque<f32>, Color32)],
    max: f32,
    height: f32,
) {
    ui.label(RichText::new(title).size(12.0).strong());
    let grid = if series.first().is_some_and(|(_, color)| *color == BLUE) {
        GRID_BLUE
    } else {
        GRID
    };
    let rect = performance::graph_rect(ui, height, BLACK, grid);
    performance::plot_lines(ui, rect.shrink(2.0), series, max, true);
}

pub(super) fn performance(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        ui.label("Reading system performance…");
        return;
    };
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| usage_meter(ui, "CPU Usage", snapshot.cpu_percent, GREEN, 135.0));
            ui.add_space(10.0);
            ui.vertical(|ui| {
                let visible = if ui.available_width() < 750.0 { 2 } else { 4 };
                let cores = app.history.cores.len().min(visible);
                if cores == 0 {
                    history_plot(ui, "CPU Usage History", &[(&app.history.cpu, GREEN)], 100.0, 135.0);
                } else {
                    ui.label(RichText::new("CPU Usage History").size(12.0).strong());
                    ui.columns(cores, |columns| {
                        for (index, column) in columns.iter_mut().enumerate() {
                            let rect = performance::graph_rect(column, 135.0, BLACK, GRID);
                            performance::plot_lines(column, rect.shrink(2.0), &[(&app.history.cores[index], GREEN)], 100.0, true);
                        }
                    });
                    if app.history.cores.len() > cores {
                        ui.label(RichText::new(format!("First {cores} of {} logical CPU histories", app.history.cores.len())).size(10.0).weak());
                    }
                }
            });
        });
        ui.add_space(13.0);
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| usage_meter(ui, "Memory", snapshot.memory.used_fraction() * 100.0, BLUE, 129.0));
            ui.add_space(10.0);
            ui.vertical(|ui| history_plot(ui, "Physical Memory Usage History", &[(&app.history.memory, BLUE)], 100.0, 129.0));
        });
        ui.add_space(18.0);
        ui.columns(3, |columns| {
            stats_box(&mut columns[0], "Physical Memory (MB)", &[
                ("Total", (snapshot.memory.total / 1_048_576).to_string()),
                ("Available", (snapshot.memory.available / 1_048_576).to_string()),
                ("In Use", (snapshot.memory.used() / 1_048_576).to_string()),
            ]);
            stats_box(&mut columns[1], "System", &[
                ("Processes", snapshot.processes.len().to_string()),
                ("Threads", snapshot.processes.iter().map(|p| p.threads as u64).sum::<u64>().to_string()),
                ("Up Time", format::duration(snapshot.uptime_seconds)),
            ]);
            stats_box(&mut columns[2], "Swap (MB)", &[
                ("Total", (snapshot.memory.swap_total / 1_048_576).to_string()),
                ("Available", (snapshot.memory.swap_free / 1_048_576).to_string()),
                ("In Use", ((snapshot.memory.swap_total - snapshot.memory.swap_free) / 1_048_576).to_string()),
            ]);
        });
        ui.add_space(9.0);
        ui.label(RichText::new("Performance data from Linux /proc. Memory uses available-page accounting; no Windows commit/kernel counters are invented.").small().weak());
    });
}

fn stats_box(ui: &mut egui::Ui, name: &str, items: &[(&str, String)]) {
    ui.label(RichText::new(name).size(12.0).strong());
    Frame::NONE
        .fill(Color32::from_rgb(247, 250, 252))
        .stroke(Stroke::new(1.0_f32, BORDER))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(120.0) - 16.0);
            for (label, value) in items {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(*label).size(11.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(value).size(11.0));
                    });
                });
            }
        });
}

pub(super) fn networking(app: &TaskottoApp, ui: &mut egui::Ui) {
    let Some(s) = &app.snapshot else {
        ui.spinner();
        ui.label("Reading network rates…");
        return;
    };
    ui.label(
        RichText::new("Network Utilization History")
            .size(13.0)
            .strong(),
    );
    ui.add_space(7.0);
    let scale = super::super::graph_max(&app.history.received, &app.history.sent);
    let height = ui.available_height().clamp(200.0, 440.0) * 0.65;
    let rect = performance::graph_rect(ui, height, BLACK, GRID);
    performance::plot_lines(
        ui,
        rect.shrink(2.0),
        &[
            (&app.history.received, GREEN),
            (&app.history.sent, Color32::from_rgb(245, 226, 84)),
        ],
        scale,
        true,
    );
    ui.add_space(12.0);
    egui::Grid::new("seven-network-readings")
        .num_columns(2)
        .spacing([25.0, 7.0])
        .show(ui, |ui| {
            for (label, value) in [
                ("Receive", format::rate(s.rates.received_per_second)),
                ("Send", format::rate(s.rates.sent_per_second)),
                ("Graph scale", format::rate(scale as u64)),
                ("Source", "Non-loopback interfaces (aggregate)".into()),
            ] {
                ui.label(RichText::new(label).strong());
                ui.label(value);
                ui.end_row();
            }
        });
    ui.add_space(9.0);
    ui.label(RichText::new("Bytes/second, not an invented link utilization %. Virtual interfaces may double-count traffic.").small().weak());
}

pub(super) fn users(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading Linux users…");
        return;
    }
    let mut groups: BTreeMap<u32, (usize, f32, u64)> = BTreeMap::new();
    for row in app.processes() {
        let value = groups.entry(row.process.user_id).or_default();
        value.0 += 1;
        value.1 += row.process.cpu_percent;
        value.2 = value.2.saturating_add(row.process.resident_bytes);
    }
    let height = (ui.available_height() - 45.0).max(96.0);
    report_frame(ui, height, |ui| {
        egui::Grid::new("seven-user-header")
            .num_columns(4)
            .spacing([20.0, 7.0])
            .show(ui, |ui| {
                for heading in ["User Name", "Processes", "CPU", "Memory (RSS)"] {
                    ui.label(RichText::new(heading).strong());
                }
                ui.end_row();
                for (uid, (count, cpu, rss)) in groups {
                    ui.label(
                        app.users
                            .get(&uid)
                            .cloned()
                            .unwrap_or_else(|| uid.to_string()),
                    );
                    ui.label(count.to_string());
                    ui.label(format!("{cpu:.1}%"));
                    ui.label(format::bytes(rss));
                    ui.end_row();
                }
            });
    });
    ui.add_space(5.0);
    ui.label(RichText::new("Linux process owners (UIDs), not verified Windows-style desktop login sessions. RSS can include shared pages.").size(10.0).weak());
}

pub(super) fn services(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("User services · systemd").size(11.0).weak());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Refresh").clicked() {
                app.system_pages.services = None;
                app.system_pages.services_loading = None;
            }
        });
    });
    system::poll_services(app, ui.ctx());
    let height = (ui.available_height() - 51.0).max(95.0);
    report_frame(ui, height, |ui| {
        let Some(result) = &app.system_pages.services else {
            ui.spinner();
            ui.label("Reading systemd user service units…");
            return;
        };
        let Ok(entries) = result else {
            ui.colored_label(Color32::from_rgb(155, 66, 26), "systemd --user unavailable");
            ui.add(egui::Label::new(result.as_ref().unwrap_err()).wrap());
            return;
        };
        if entries.is_empty() {
            ui.label("No user service units loaded.");
            return;
        }
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("seven-services-table")
                .striped(true)
                .num_columns(4)
                .spacing([22.0, 7.0])
                .show(ui, |ui| {
                    for title in ["Name", "Status", "Unit state", "Description"] {
                        ui.label(RichText::new(title).strong());
                    }
                    ui.end_row();
                    for entry in entries {
                        ui.label(&entry.unit);
                        ui.label(&entry.active);
                        ui.label(&entry.state);
                        ui.label(&entry.description);
                        ui.end_row();
                    }
                });
        });
    });
    ui.add_space(5.0);
    ui.label(RichText::new("Read-only Linux user units; no invented Windows service PID, group or service-control privileges.").size(10.0).weak());
}

pub(super) fn properties_dialog(app: &mut TaskottoApp, ctx: &egui::Context) {
    // A classic floating properties sheet rather than a fictitious seventh tab.
    // The existing Linux inspector and PID-safe actions live in this dialog.
    let mut open = true;
    egui::Window::new("Process Properties")
        .id(egui::Id::new("seven-process-properties"))
        .open(&mut open)
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(520.0)
        .default_height(465.0)
        .resizable(true)
        .frame(
            Frame::window(&ctx.style())
                .fill(Color32::from_rgb(237, 245, 250))
                .stroke(Stroke::new(1.0_f32, BORDER)),
        )
        .show(ctx, |ui| details::xp_page(app, ui));
    if !open {
        app.page = Page::Processes;
    }
}

fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let channel = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8;
    Color32::from_rgb(
        channel(a.r(), b.r()),
        channel(a.g(), b.g()),
        channel(a.b(), b.b()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_has_historical_six_tabs_in_order() {
        assert_eq!(
            TABS.map(|(_, label)| label),
            [
                "Applications",
                "Processes",
                "Services",
                "Performance",
                "Networking",
                "Users"
            ]
        );
    }

    #[test]
    fn seven_columns_fit_compact_and_wide_views() {
        let settings = Settings::default();
        for width in [430.0_f32, 630.0, 1000.0, 1260.0] {
            let cols = process_columns(width, &settings);
            assert_eq!(cols[0].0, ReportColumn::Name);
            assert_eq!(cols[cols.len() - 1].0, ReportColumn::Description);
            assert!(cols.iter().all(|(_, w)| *w >= 50.0));
            let actual = cols.iter().map(|(_, w)| w).sum::<f32>();
            assert!((actual - width).abs() < 0.01, "width {width}: got {actual}");
        }
    }

    #[test]
    fn color_interpolation_endpoints_are_exact() {
        assert_eq!(blend(PALE, INK, 0.0), PALE);
        assert_eq!(blend(PALE, INK, 1.0), INK);
    }
}
