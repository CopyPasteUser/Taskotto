//! ChromeOS-inspired task manager. Only the client area is painted: the window
//! manager (or RiceBowl) owns window decorations and their buttons.
//!
//! The original Chrome task manager reports per-task network activity. Linux
//! procfs does not provide that measurement, so the analogous optional column
//! here is explicitly *disk I/O*, never a made-up network figure.
use super::*;
use egui::{Align2, FontId, Frame, Margin, Rect, StrokeKind};

const MUTED: Color32 = Color32::from_rgb(94, 105, 121);
const LINE: Color32 = Color32::from_rgb(223, 227, 234);

struct Palette {
    accent: Color32,
    surface: Color32,
    paper: Color32,
    ink: Color32,
    muted: Color32,
    line: Color32,
    selected: Color32,
    hover: Color32,
}

impl Palette {
    fn new(app: &TaskottoApp) -> Self {
        let style = ThemeStyle::from_settings(&app.settings);
        let paper = if style.dark {
            mix(style.surface, Color32::WHITE, 0.07)
        } else {
            mix(style.surface, Color32::WHITE, 0.70)
        };
        let ink = style.text;
        let muted = if style.dark {
            mix(ink, paper, 0.28)
        } else {
            MUTED
        };
        Self {
            accent: style.accent,
            surface: style.surface,
            paper,
            ink,
            muted,
            line: if style.dark {
                mix(paper, ink, 0.23)
            } else {
                LINE
            },
            selected: mix(paper, style.accent, if style.dark { 0.24 } else { 0.14 }),
            hover: mix(paper, style.accent, if style.dark { 0.13 } else { 0.055 }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Column {
    Task,
    Memory,
    Cpu,
    Disk,
    Pid,
    User,
}

impl Column {
    fn title(self) -> &'static str {
        match self {
            Self::Task => "Task",
            Self::Memory => "Memory footprint",
            Self::Cpu => "CPU",
            Self::Disk => "Disk I/O",
            Self::Pid => "Process ID",
            Self::User => "User",
        }
    }

    fn sort(self) -> Option<SortBy> {
        Some(match self {
            Self::Task => SortBy::Name,
            Self::Memory => SortBy::Memory,
            Self::Cpu => SortBy::Cpu,
            Self::Disk => SortBy::Io,
            Self::Pid => SortBy::Pid,
            Self::User => return None,
        })
    }
}

struct Columns {
    cells: Vec<(Column, f32)>,
    width: f32,
}

impl Columns {
    fn new(width: f32, settings: &Settings) -> Self {
        let width = width.max(430.0);
        let mut cells = vec![
            (Column::Memory, 148.0),
            (Column::Cpu, 82.0),
            (Column::Pid, 100.0),
        ];
        if settings.show_io_column && width >= 780.0 {
            cells.insert(2, (Column::Disk, 116.0));
        }
        if settings.show_user_column && width >= 1070.0 {
            cells.push((Column::User, 124.0));
        }
        let other = cells.iter().map(|(_, w)| w).sum::<f32>();
        cells.insert(0, (Column::Task, (width - other).max(100.0)));
        Self { cells, width }
    }

    fn rects(&self, row: Rect) -> impl Iterator<Item = (Column, Rect)> + '_ {
        let mut x = row.left();
        self.cells.iter().map(move |&(column, width)| {
            let cell =
                Rect::from_min_size(egui::pos2(x, row.top()), egui::vec2(width, row.height()));
            x += width;
            (column, cell)
        })
    }
}

/// The top strip is an application toolbar, not a substitute titlebar.
pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let colors = Palette::new(app);
    egui::TopBottomPanel::top("chromeos-toolbar")
        .frame(
            Frame::NONE
                .fill(colors.paper)
                .inner_margin(Margin::symmetric(18, 11)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let current = app.page;
                if current != Page::Processes
                    && ui
                        .add(egui::Button::new("‹  Tasks").fill(colors.surface))
                        .on_hover_text("Return to the task list")
                        .clicked()
                {
                    app.page = Page::Processes;
                }
                ui.label(
                    RichText::new(if current == Page::Processes {
                        "Task manager"
                    } else {
                        "Taskotto tools"
                    })
                    .color(colors.ink)
                    .size(17.0)
                    .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("More", |ui| tools_menu(app, ui))
                        .response
                        .on_hover_text("More tools and options");
                    if ui.button("↻").on_hover_text("Refresh now (F5)").clicked() {
                        let _ = app.worker.commands.send(Command::Refresh);
                    }
                    if app.page == Page::Processes {
                        let search_width = (ui.available_width() - 8.0).clamp(140.0, 310.0);
                        let search = ui.add_sized(
                            [search_width, 31.0],
                            egui::TextEdit::singleline(&mut app.query)
                                .hint_text("Search tasks")
                                .desired_width(search_width),
                        );
                        if app.focus_search {
                            search.request_focus();
                            app.focus_search = false;
                        }
                    }
                });
            });
        });
}

fn tools_menu(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.set_min_width(210.0);
    for (page, name) in [
        (Page::Processes, "Tasks"),
        (Page::Performance, "Performance"),
        (Page::Networking, "Network activity"),
        (Page::Users, "Users"),
        (Page::Details, "Process details"),
        (Page::Applications, "Applications"),
        (Page::AppHistory, "App history"),
        (Page::Startup, "Startup"),
        (Page::Services, "Services"),
    ] {
        if ui.selectable_label(app.page == page, name).clicked() {
            app.page = page;
            ui.close();
        }
    }
    ui.separator();
    let mut all = !app.settings.only_my_processes;
    if ui.checkbox(&mut all, "All users' processes").changed() {
        app.settings.only_my_processes = !all;
        app.save_preferences(ui.ctx());
    }
    if ui
        .checkbox(&mut app.settings.show_kernel_threads, "Kernel threads")
        .changed()
    {
        app.save_preferences(ui.ctx());
    }
    if ui
        .checkbox(&mut app.settings.show_io_column, "Disk I/O column")
        .changed()
    {
        app.save_preferences(ui.ctx());
    }
    if ui
        .checkbox(&mut app.settings.show_user_column, "User column")
        .changed()
    {
        app.save_preferences(ui.ctx());
    }
    if ui
        .checkbox(&mut app.settings.tree_view, "Process tree")
        .changed()
    {
        app.save_preferences(ui.ctx());
    }
    ui.menu_button("Update interval", |ui| {
        for (ms, label) in [
            (500, "Fast · 0.5 s"),
            (1_500, "Normal · 1.5 s"),
            (5_000, "Slow · 5 s"),
            (10_000, "Very slow · 10 s"),
        ] {
            if ui
                .selectable_label(app.settings.refresh_ms == ms, label)
                .clicked()
            {
                app.settings.refresh_ms = ms;
                app.save_preferences(ui.ctx());
                ui.close();
            }
        }
    });
    ui.separator();
    if ui.button("New task…").clicked() {
        app.new_task.open = true;
        app.new_task.error = None;
        ui.close();
    }
    if ui.button("Settings…").clicked() {
        app.page = Page::Preferences;
        ui.close();
    }
}

/// Advanced navigation is opt-in through the existing layout preference.
/// The standard ChromeOS view remains a single task table with a tools menu.
pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    let colors = Palette::new(app);
    egui::TopBottomPanel::top("chromeos-extra-tabs")
        .frame(
            Frame::NONE
                .fill(colors.paper)
                .inner_margin(Margin::symmetric(16, 3)),
        )
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                for (page, label) in [
                    (Page::Processes, "Tasks"),
                    (Page::Performance, "Performance"),
                    (Page::Networking, "Network"),
                    (Page::Users, "Users"),
                    (Page::Preferences, "Settings"),
                ] {
                    if ui
                        .add(egui::Button::new(label).selected(app.page == page))
                        .clicked()
                    {
                        app.page = page;
                    }
                }
            });
        });
}

pub(super) fn vertical_navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    let colors = Palette::new(app);
    egui::SidePanel::left("chromeos-extra-sidebar")
        .exact_width(164.0)
        .resizable(false)
        .frame(
            Frame::NONE
                .fill(colors.paper)
                .inner_margin(Margin::symmetric(8, 12)),
        )
        .show(ctx, |ui| {
            for (page, label) in [
                (Page::Processes, "Tasks"),
                (Page::Performance, "Performance"),
                (Page::Networking, "Network"),
                (Page::Users, "Users"),
                (Page::Details, "Details"),
                (Page::Preferences, "Settings"),
            ] {
                if ui
                    .add_sized(
                        [ui.available_width(), 36.0],
                        egui::Button::new(label).selected(app.page == page),
                    )
                    .clicked()
                {
                    app.page = page;
                }
            }
        });
}

pub(super) fn statusbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let colors = Palette::new(app);
    let count = app.snapshot.as_ref().map_or(0, |s| s.processes.len());
    let query = app.query.to_lowercase();
    let available = app.selected_process().is_some_and(|process| {
        can_signal_process(process) && app.process_matches_filters(process, &query)
    });
    egui::TopBottomPanel::bottom("chromeos-statusbar")
        .frame(
            Frame::NONE
                .fill(colors.paper)
                .inner_margin(Margin::symmetric(18, 9)),
        )
        .show(ctx, |ui| {
            let line = ui.max_rect();
            ui.painter().line_segment(
                [line.left_top(), line.right_top()],
                Stroke::new(1.0_f32, colors.line),
            );
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{count} Linux processes"))
                        .color(colors.muted)
                        .size(12.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if app.page == Page::Processes
                        && ui
                            .add_enabled(
                                available,
                                egui::Button::new(RichText::new("End process").color(
                                    if available {
                                        crate::theme::readable_text(colors.accent)
                                    } else {
                                        colors.muted
                                    },
                                ))
                                .fill(if available {
                                    colors.accent
                                } else {
                                    mix(colors.paper, colors.surface, 0.6)
                                })
                                .corner_radius(17),
                            )
                            .on_hover_text("End the selected Linux process (asks for confirmation)")
                            .clicked()
                    {
                        app.queue_action(SignalAction::Terminate);
                    }
                });
            });
        });
}

pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let colors = Palette::new(app);
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        ui.add_space(15.0);
        for (mine, title) in [(false, "All tasks"), (true, "My tasks")] {
            let active = app.settings.only_my_processes == mine;
            let response = ui.add(
                egui::Button::new(RichText::new(title).color(if active {
                    colors.ink
                } else {
                    colors.muted
                }))
                .fill(if active {
                    colors.selected
                } else {
                    colors.paper
                })
                .stroke(if active {
                    Stroke::new(1.0_f32, colors.accent)
                } else {
                    Stroke::NONE
                })
                .corner_radius(16),
            );
            if response.clicked() {
                app.settings.only_my_processes = mine;
                app.save_preferences(ui.ctx());
            }
        }
    });
    ui.add_space(6.0);
    if app.snapshot.is_none() {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.spinner();
            ui.label("Reading Linux processes…");
        });
        return;
    }
    let rows = app.processes();
    let row_height =
        app.settings
            .table_row_height
            .unwrap_or(if app.settings.density == Density::Compact {
                27.0
            } else {
                34.0
            });
    let content_height = ui.available_height().max(140.0);
    Frame::NONE
        .fill(colors.paper)
        .inner_margin(Margin::symmetric(12, 0))
        .show(ui, |ui| {
            ui.set_min_height(content_height);
            let columns = Columns::new(ui.available_width() - 4.0, &app.settings);
            header(app, ui, &colors, &columns);
            if rows.is_empty() {
                ui.add_space(48.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No matching tasks")
                            .color(colors.ink)
                            .strong(),
                    );
                    ui.label(
                        RichText::new("Try a different search or filter.").color(colors.muted),
                    );
                    if !app.query.is_empty() && ui.button("Clear search").clicked() {
                        app.query.clear();
                    }
                });
                return;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height((content_height - 40.0).max(80.0))
                .show_rows(ui, row_height, rows.len(), |ui, visible| {
                    for (index, row) in rows[visible.clone()].iter().enumerate() {
                        row_ui(
                            app,
                            ui,
                            &colors,
                            &columns,
                            row,
                            visible.start + index,
                            row_height,
                        );
                    }
                });
        });
}

/// Extra Linux tooling stays functional without introducing ChromeOS-only
/// metrics or changing the single-table default experience.
pub(super) fn performance(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let colors = Palette::new(app);
    page_heading(
        ui,
        &colors,
        "System performance",
        "Live Linux CPU, memory and disk activity",
    );
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        ui.label("Sampling system activity…");
        return;
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.columns(2, |columns| {
            metric_panel(
                &mut columns[0],
                &colors,
                "CPU",
                &format!("{:.1}%", snapshot.cpu_percent),
                |ui| {
                    chart(ui, &app.history.cpu, 100.0, colors.accent);
                    ui.label(format!("{} logical CPUs", snapshot.logical_cpus));
                },
            );
            metric_panel(
                &mut columns[1],
                &colors,
                "Memory",
                &format!("{:.1}%", snapshot.memory.used_fraction() * 100.0),
                |ui| {
                    chart(ui, &app.history.memory, 100.0, colors.accent);
                    ui.label(format!(
                        "{} of {} used",
                        format::bytes(snapshot.memory.used()),
                        format::bytes(snapshot.memory.total)
                    ));
                },
            );
        });
        ui.add_space(12.0);
        ui.columns(2, |columns| {
            metric_panel(
                &mut columns[0],
                &colors,
                "Disk read",
                &format::rate(snapshot.rates.disk_read_per_second),
                |ui| {
                    chart_lines(
                        ui,
                        &[
                            (&app.history.disk_read, colors.accent),
                            (&app.history.disk_write, Color32::from_rgb(48, 157, 123)),
                        ],
                        graph_max(&app.history.disk_read, &app.history.disk_write),
                    );
                    ui.label("Blue: read · Green: write · bytes/s");
                },
            );
            metric_panel(
                &mut columns[1],
                &colors,
                "Uptime",
                &format::duration(snapshot.uptime_seconds),
                |ui| {
                    ui.label(format!(
                        "Swap: {} / {}",
                        format::bytes(
                            snapshot
                                .memory
                                .swap_total
                                .saturating_sub(snapshot.memory.swap_free)
                        ),
                        format::bytes(snapshot.memory.swap_total)
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        "Values come from Linux procfs; no simulated ChromeOS hardware counters.",
                    );
                },
            );
        });
    });
}

pub(super) fn networking(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let colors = Palette::new(app);
    page_heading(
        ui,
        &colors,
        "Network activity",
        "Combined system traffic excluding loopback · not individual task traffic",
    );
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        ui.label("Sampling network activity…");
        return;
    };
    metric_panel(ui, &colors, "Live traffic", "Receive / Send", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!(
                "Received  {}",
                format::rate(snapshot.rates.received_per_second)
            ));
            ui.separator();
            ui.label(format!(
                "Sent  {}",
                format::rate(snapshot.rates.sent_per_second)
            ));
        });
        chart_lines(
            ui,
            &[
                (&app.history.received, colors.accent),
                (&app.history.sent, Color32::from_rgb(48, 157, 123)),
            ],
            graph_max(&app.history.received, &app.history.sent),
        );
        ui.label(
            "Blue: received · Green: sent · bytes/s · Virtual interfaces can double-count traffic.",
        );
    });
}

fn page_heading(ui: &mut egui::Ui, colors: &Palette, title: &str, subtitle: &str) {
    ui.add_space(13.0);
    ui.label(RichText::new(title).size(22.0).strong().color(colors.ink));
    ui.label(RichText::new(subtitle).size(12.0).color(colors.muted));
    ui.add_space(18.0);
}

fn metric_panel(
    ui: &mut egui::Ui,
    colors: &Palette,
    name: &str,
    value: &str,
    content: impl FnOnce(&mut egui::Ui),
) {
    Frame::NONE
        .fill(colors.paper)
        .stroke(Stroke::new(1.0_f32, colors.line))
        .corner_radius(12.0)
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(RichText::new(name).size(12.0).color(colors.muted));
            ui.label(RichText::new(value).size(24.0).strong().color(colors.ink));
            ui.add_space(8.0);
            content(ui);
        });
}

fn header(app: &mut TaskottoApp, ui: &mut egui::Ui, colors: &Palette, columns: &Columns) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(columns.width, 37.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, colors.paper);
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, colors.line),
    );
    for (column, cell) in columns.rects(rect) {
        let label = if column.sort() == Some(app.settings.sort_by) {
            format!(
                "{} {}",
                column.title(),
                if app.settings.descending { "v" } else { "^" }
            )
        } else {
            column.title().to_owned()
        };
        paint_cell(
            ui,
            cell,
            &label,
            colors.muted,
            app.settings.font_scale * 12.0,
            false,
        );
        if let Some(sort) = column.sort() {
            let response = ui.interact(
                cell,
                ui.id().with(("chrome-sort", column.title())),
                Sense::click(),
            );
            if response.has_focus() {
                ui.painter().rect_stroke(
                    cell.shrink(2.0),
                    3.0,
                    Stroke::new(2.0_f32, colors.accent),
                    StrokeKind::Inside,
                );
            }
            if response.clicked() {
                if app.settings.sort_by == sort {
                    app.settings.descending = !app.settings.descending;
                } else {
                    app.settings.sort_by = sort;
                    app.settings.descending =
                        matches!(sort, SortBy::Cpu | SortBy::Memory | SortBy::Io);
                }
                app.save_preferences(ui.ctx());
            }
            response.on_hover_text(format!("Sort by {}", column.title()));
        }
    }
}

fn row_ui(
    app: &mut TaskottoApp,
    ui: &mut egui::Ui,
    colors: &Palette,
    columns: &Columns,
    row: &ProcessRow,
    index: usize,
    height: f32,
) {
    let process = &row.process;
    let selected = app.selected == Some((process.pid, process.start_ticks));
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(columns.width, height), Sense::click());
    ui.painter().rect_filled(
        rect,
        0.0,
        if selected {
            colors.selected
        } else if response.hovered() {
            colors.hover
        } else if index.is_multiple_of(2) {
            colors.paper
        } else {
            mix(colors.paper, colors.surface, 0.12)
        },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            3.0,
            Stroke::new(2.0_f32, colors.accent),
            StrokeKind::Inside,
        );
    }
    for (column, cell) in columns.rects(rect) {
        let content = match column {
            Column::Task if row.depth > 0 => {
                format!("{}↳ {}", "  ".repeat(row.depth.min(6)), process.name)
            }
            Column::Task => process.name.clone(),
            Column::Memory => format::bytes(process.resident_bytes),
            Column::Cpu => format!("{:.1}%", process.cpu_percent),
            Column::Disk if !process.io_accessible => "N/A".to_owned(),
            Column::Disk => format::rate(process_io_rate(process)),
            Column::Pid => process.pid.to_string(),
            Column::User => app
                .users
                .get(&process.user_id)
                .cloned()
                .unwrap_or_else(|| process.user_id.to_string()),
        };
        paint_cell(
            ui,
            cell,
            &content,
            if column == Column::Task {
                colors.ink
            } else {
                colors.muted
            },
            app.settings.font_scale * 13.0,
            column == Column::Task,
        );
    }
    if response.clicked() {
        app.selected = Some((process.pid, process.start_ticks));
    }
    if response.double_clicked() {
        app.selected = Some((process.pid, process.start_ticks));
        app.page = Page::Details;
    }
    response.on_hover_text(&process.command).context_menu(|ui| {
        if ui.button("Process details").clicked() {
            app.selected = Some((process.pid, process.start_ticks));
            app.page = Page::Details;
            ui.close();
        }
        if ui
            .add_enabled(
                can_signal_process(process),
                egui::Button::new("End process…"),
            )
            .clicked()
        {
            app.selected = Some((process.pid, process.start_ticks));
            app.queue_action(SignalAction::Terminate);
            ui.close();
        }
    });
}

fn paint_cell(ui: &egui::Ui, rect: Rect, content: &str, color: Color32, size: f32, strong: bool) {
    let painter = ui
        .painter()
        .with_clip_rect(rect.shrink2(egui::vec2(7.0, 1.0)));
    painter.text(
        rect.left_center() + egui::vec2(9.0, 0.0),
        Align2::LEFT_CENTER,
        content,
        if strong {
            FontId::proportional(size + 0.2)
        } else {
            FontId::proportional(size)
        },
        color,
    );
}

fn mix(a: Color32, b: Color32, amount: f32) -> Color32 {
    let t = amount.clamp(0.0, 1.0);
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
    fn columns_fit_small_and_large_windows_without_fabricated_network() {
        let settings = Settings {
            theme: Theme::Chromeos,
            ..Settings::default()
        };
        for width in [430.0, 590.0, 760.0, 780.0, 1280.0] {
            let cols = Columns::new(width, &settings);
            assert!(cols.cells[0].1 >= 100.0);
            assert!(cols.cells.iter().map(|(_, w)| w).sum::<f32>() <= width + 0.1);
            assert!(
                cols.cells
                    .iter()
                    .all(|(column, _)| column.title() != "Network")
            );
            assert_eq!(
                cols.cells.iter().any(|(c, _)| *c == Column::Disk),
                width >= 780.0
            );
        }
    }
}
