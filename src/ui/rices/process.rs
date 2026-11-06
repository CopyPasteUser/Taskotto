use super::*;
use egui::{Align2, FontId, Rect, StrokeKind};

#[derive(Clone, Copy)]
enum Column {
    Name,
    Pid,
    Cpu,
    Memory,
    Io,
    User,
}

impl Column {
    fn sort(self) -> Option<SortBy> {
        match self {
            Self::Name => Some(SortBy::Name),
            Self::Pid => Some(SortBy::Pid),
            Self::Cpu => Some(SortBy::Cpu),
            Self::Memory => Some(SortBy::Memory),
            Self::Io => Some(SortBy::Io),
            Self::User => None,
        }
    }
}

struct TableColumns {
    columns: Vec<(Column, f32)>,
    width: f32,
}

impl TableColumns {
    fn new(available: f32, settings: &Settings) -> Self {
        let available = available.max(390.0);
        let xp = settings.theme == Theme::XpLuna;
        // XP uses a fixed-width report-view header. The list itself fills the
        // window, but stretching "Image Name" across a large monitor would
        // look unlike the classic Win32 control.
        let width = if xp { available.min(800.0) } else { available };
        let min_io_width = if xp { 840.0 } else { 875.0 };
        let min_user_width = if xp { 700.0 } else { 1040.0 };
        let mut columns = vec![
            (Column::Pid, if xp { 65.0 } else { 75.0 }),
            (Column::Cpu, if xp { 62.0 } else { 104.0 }),
            (Column::Memory, if xp { 113.0 } else { 126.0 }),
        ];
        if settings.show_io_column && available >= min_io_width {
            columns.push((Column::Io, if xp { 115.0 } else { 133.0 }));
        }
        if settings.show_user_column && available >= min_user_width {
            columns.push((Column::User, if xp { 120.0 } else { 128.0 }));
        }
        let other: f32 = columns.iter().map(|(_, w)| w).sum();
        columns.insert(0, (Column::Name, (width - other).max(140.0)));
        Self { columns, width }
    }

    fn cells(&self, rect: Rect) -> impl Iterator<Item = (Column, Rect)> + '_ {
        let mut x = rect.left();
        self.columns.iter().map(move |&(column, width)| {
            let cell =
                Rect::from_min_size(egui::pos2(x, rect.top()), egui::vec2(width, rect.height()));
            x += width;
            (column, cell)
        })
    }
}

enum Entry {
    Section(&'static str, usize),
    Process(ProcessRow),
}

fn grouped_rows(app: &TaskottoApp, rows: Vec<ProcessRow>) -> Vec<Entry> {
    if app.page == Page::Details || is_xp(app) || app.settings.tree_view {
        return rows.into_iter().map(Entry::Process).collect();
    }
    // Windows 10-style sectioning with truthful Linux semantics. Grouping by
    // owner is reliable; pretending to know which processes own GUI windows is not.
    let (owned, rest): (Vec<_>, Vec<_>) = rows
        .into_iter()
        .partition(|row| row.process.user_id == app.my_uid);
    let mut entries = Vec::with_capacity(owned.len() + rest.len() + 2);
    if !owned.is_empty() {
        entries.push(Entry::Section("User processes", owned.len()));
        entries.extend(owned.into_iter().map(Entry::Process));
    }
    if !rest.is_empty() {
        entries.push(Entry::Section("Other processes", rest.len()));
        entries.extend(rest.into_iter().map(Entry::Process));
    }
    entries
}

pub(super) fn page(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let xp = is_xp(app);
    // Windows 10's Processes tab and XP's report view begin directly with
    // the column headers. The large heading belongs to Performance, not here.
    // Both historical task managers show their table immediately. Keep the
    // extra Linux search available with Ctrl+F / View rather than inserting a
    // permanent toolbar that changes the silhouette of either application.
    if app.focus_search || !app.query.is_empty() {
        ui.horizontal(|ui| {
            let search = ui.add_sized(
                [ui.available_width().min(300.0), 24.0],
                egui::TextEdit::singleline(&mut app.query).hint_text("Filter processes (Ctrl+F)"),
            );
            if app.focus_search {
                search.request_focus();
                app.focus_search = false;
            }
            if ui.small_button("Clear ×").clicked() {
                app.query.clear();
            }
        });
    }
    ui.add_space(if xp { 2.0 } else { 5.0 });
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Loading Linux process information…");
        return;
    }
    let rows = app.processes();
    if rows.is_empty() {
        egui::Frame::new()
            .fill(Color32::WHITE)
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(173, 173, 173)))
            .show(ui, |ui| {
                ui.set_min_height((ui.available_height() - 12.0).max(100.0));
                ui.add_space(32.0);
                ui.vertical_centered(|ui| {
                    ui.label("No matching processes");
                    if ui.button("Clear filter").clicked() {
                        app.query.clear();
                    }
                });
            });
        return;
    }
    let footer_height = if xp { 32.0 } else { 0.0 };
    let table_height = (ui.available_height() - footer_height - 6.0).max(100.0);
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(
            1.0_f32,
            if xp {
                Color32::from_rgb(128, 128, 128)
            } else {
                Color32::from_rgb(218, 218, 218)
            },
        ))
        .show(ui, |ui| {
            ui.set_min_height(table_height);
            let layout = TableColumns::new(ui.available_width() - 14.0, &app.settings);
            header(app, ui, &layout);
            let entries = grouped_rows(app, rows);
            let row_height = app
                .settings
                .table_row_height
                .unwrap_or(if xp { 21.0 } else { 33.0 });
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height((table_height - if xp { 25.0 } else { 66.0 }).max(60.0))
                .show_rows(ui, row_height, entries.len(), |ui, range| {
                    for index in range {
                        match &entries[index] {
                            Entry::Section(label, count) => {
                                section(ui, &layout, label, *count, row_height)
                            }
                            Entry::Process(row) => {
                                process_row(app, ui, &layout, row, index, row_height)
                            }
                        }
                    }
                });
        });
    if xp {
        ui.add_space(4.0);
        let mut all_users = !app.settings.only_my_processes;
        if ui
            .checkbox(&mut all_users, "Show processes from all users")
            .changed()
        {
            app.settings.only_my_processes = !all_users;
            app.save_preferences(ui.ctx());
        }
    }
}

fn header(app: &mut TaskottoApp, ui: &mut egui::Ui, layout: &TableColumns) {
    let xp = is_xp(app);
    let details = app.page == Page::Details;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(layout.width, if xp || details { 23.0 } else { 65.0 }),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, 0.0, Color32::WHITE);
    for (column, cell) in layout.cells(rect) {
        let painter = ui.painter();
        if xp {
            painter.rect_stroke(
                cell,
                0.0,
                Stroke::new(1.0_f32, Color32::from_rgb(169, 167, 153)),
                StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    cell.left_top() + egui::vec2(1.0, 1.0),
                    cell.right_top() + egui::vec2(-1.0, 1.0),
                ],
                Stroke::new(1.0_f32, Color32::WHITE),
            );
        } else {
            painter.line_segment(
                [cell.right_top(), cell.right_bottom()],
                Stroke::new(1.0_f32, Color32::from_rgb(235, 235, 235)),
            );
            painter.line_segment(
                [cell.left_bottom(), cell.right_bottom()],
                Stroke::new(1.0_f32, Color32::from_rgb(216, 216, 216)),
            );
        }
        let label = match column {
            Column::Name => {
                if xp {
                    "Image Name"
                } else {
                    "Name"
                }
            }
            Column::Pid => "PID",
            Column::Cpu => "CPU",
            Column::Memory => {
                if xp {
                    "Mem Usage"
                } else {
                    "Memory"
                }
            }
            Column::Io => {
                if xp {
                    "I/O Bytes"
                } else {
                    "Disk I/O"
                }
            }
            Column::User => {
                if xp {
                    "User Name"
                } else {
                    "User"
                }
            }
        };
        if !xp && !details && matches!(column, Column::Cpu | Column::Memory) {
            let percent = match column {
                Column::Cpu => app.snapshot.as_ref().map_or(0.0, |s| s.cpu_percent),
                _ => app
                    .snapshot
                    .as_ref()
                    .map_or(0.0, |s| s.memory.used_fraction() * 100.0),
            };
            painter.rect_filled(
                cell.shrink2(egui::vec2(1.0, 1.0)),
                0.0,
                Color32::from_rgb(228, 242, 251),
            );
            painter.text(
                egui::pos2(cell.center().x, cell.top() + 24.0),
                Align2::CENTER_CENTER,
                format!("{percent:.0}%"),
                FontId::proportional(22.0),
                Color32::from_rgb(17, 86, 142),
            );
        }
        painter.text(
            egui::pos2(
                cell.left() + if xp { 7.0 } else { 10.0 },
                cell.bottom() - if xp { 15.0 } else { 12.0 },
            ),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(if xp || details { 12.0 } else { 13.0 }),
            Color32::from_rgb(40, 40, 40),
        );
        if let Some(sort) = column.sort() {
            let response = ui.interact(cell, ui.id().with(("rice-sort", label)), Sense::click());
            if response.has_focus() {
                ui.painter().rect_stroke(
                    cell.shrink(2.0),
                    0.0,
                    Stroke::new(2.0_f32, accent(app)),
                    StrokeKind::Inside,
                );
            }
            if response.clicked() {
                if app.settings.sort_by == sort {
                    app.settings.descending = !app.settings.descending;
                } else {
                    app.settings.sort_by = sort;
                    app.settings.descending = !matches!(sort, SortBy::Name | SortBy::Pid);
                }
                app.save_preferences(ui.ctx());
            }
            response.on_hover_text(format!("Sort by {label}"));
            if app.settings.sort_by == sort {
                painter.text(
                    egui::pos2(
                        cell.right() - 12.0,
                        cell.bottom() - if xp { 15.0 } else { 12.0 },
                    ),
                    Align2::CENTER_CENTER,
                    if app.settings.descending { "v" } else { "^" },
                    FontId::monospace(10.0),
                    Color32::from_rgb(70, 70, 70),
                );
            }
        }
    }
}

fn section(ui: &mut egui::Ui, layout: &TableColumns, label: &str, count: usize, height: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(layout.width, height), Sense::hover());
    ui.painter()
        .rect_filled(rect, 0.0, Color32::from_rgb(245, 247, 250));
    ui.painter().text(
        rect.left_center() + egui::vec2(11.0, 0.0),
        Align2::LEFT_CENTER,
        format!("{label} ({count})"),
        FontId::proportional(14.0),
        Color32::from_rgb(48, 64, 80),
    );
}

fn process_row(
    app: &mut TaskottoApp,
    ui: &mut egui::Ui,
    layout: &TableColumns,
    row: &ProcessRow,
    index: usize,
    height: f32,
) {
    let p = &row.process;
    let xp = is_xp(app);
    let selected = app.selected == Some((p.pid, p.start_ticks));
    let (rect, response) = ui.allocate_exact_size(egui::vec2(layout.width, height), Sense::click());
    let background = if selected {
        if xp {
            Color32::from_rgb(49, 106, 195)
        } else {
            Color32::from_rgb(203, 230, 247)
        }
    } else if xp || index.is_multiple_of(2) {
        Color32::WHITE
    } else {
        Color32::from_rgb(250, 250, 250)
    };
    ui.painter().rect_filled(
        rect,
        0.0,
        if response.hovered() && !selected {
            if xp {
                Color32::from_rgb(228, 237, 249)
            } else {
                Color32::from_rgb(234, 247, 253)
            }
        } else {
            background
        },
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            0.0,
            Stroke::new(2.0_f32, accent(app)),
            StrokeKind::Inside,
        );
    }
    let text_color = if xp && selected {
        Color32::WHITE
    } else {
        Color32::from_rgb(37, 37, 37)
    };
    let font = FontId::proportional(if xp { 12.0 } else { 13.5 });
    let memory_used = app.snapshot.as_ref().map_or(1, |s| s.memory.used()).max(1);
    for (column, cell) in layout.cells(rect) {
        if !xp && matches!(column, Column::Name) {
            // Original generic app mark: Linux process identities do not
            // provide a Windows executable icon or imply a desktop window.
            let icon = Rect::from_min_size(
                egui::pos2(cell.left() + 9.0, cell.center().y - 7.0),
                egui::vec2(14.0, 14.0),
            );
            ui.painter().rect_filled(icon, 1.0, accent(app));
            ui.painter()
                .rect_filled(icon.shrink(4.0), 0.0, Color32::WHITE);
        }
        if !selected && !xp && matches!(column, Column::Cpu | Column::Memory) {
            let intensity = match column {
                Column::Cpu => (p.cpu_percent / 100.0).clamp(0.0, 1.0),
                _ => (p.resident_bytes as f32 / memory_used as f32).clamp(0.0, 1.0),
            };
            ui.painter().rect_filled(
                cell.shrink2(egui::vec2(1.0, 0.0)),
                0.0,
                heat_color(intensity, response.hovered()),
            );
        }
        if xp {
            ui.painter().line_segment(
                [cell.right_top(), cell.right_bottom()],
                Stroke::new(0.5_f32, Color32::from_rgb(225, 225, 225)),
            );
        }
        let value = match column {
            Column::Name => {
                let prefix = if row.depth > 0 {
                    format!("{}> ", "  ".repeat(row.depth.min(9)))
                } else {
                    String::new()
                };
                format!("{prefix}{}", p.name)
            }
            Column::Pid => p.pid.to_string(),
            Column::Cpu => {
                if xp {
                    format!("{:.0}", p.cpu_percent)
                } else {
                    format!("{:.1}%", p.cpu_percent)
                }
            }
            Column::Memory => {
                if xp {
                    format!("{} K", p.resident_bytes.div_ceil(1024))
                } else {
                    format::bytes(p.resident_bytes)
                }
            }
            Column::Io => {
                if p.io_accessible {
                    format::rate(process_io_rate(p))
                } else {
                    "N/A".to_owned()
                }
            }
            Column::User => app
                .users
                .get(&p.user_id)
                .cloned()
                .unwrap_or_else(|| p.user_id.to_string()),
        };
        ui.painter().with_clip_rect(cell.shrink(3.0)).text(
            egui::pos2(
                cell.left()
                    + if xp {
                        7.0
                    } else if matches!(column, Column::Name) {
                        30.0
                    } else {
                        10.0
                    },
                cell.center().y,
            ),
            Align2::LEFT_CENTER,
            value,
            font.clone(),
            text_color,
        );
    }
    if response.clicked() {
        app.selected = Some((p.pid, p.start_ticks));
    }
    if response.double_clicked() {
        app.selected = Some((p.pid, p.start_ticks));
        app.page = Page::Details;
    }
    response.on_hover_text(&p.command).context_menu(|ui| {
        if ui.button("View process details").clicked() {
            app.selected = Some((p.pid, p.start_ticks));
            app.page = Page::Details;
            ui.close();
        }
        ui.separator();
        for (label, action) in [
            ("End process…", SignalAction::Terminate),
            ("Force kill…", SignalAction::ForceKill),
            ("Suspend process…", SignalAction::Stop),
            ("Resume process…", SignalAction::Continue),
        ] {
            if ui
                .add_enabled(can_signal_process(p), egui::Button::new(label))
                .clicked()
            {
                app.selected = Some((p.pid, p.start_ticks));
                app.queue_action(action);
                ui.close();
            }
        }
    });
}

fn heat_color(intensity: f32, hovered: bool) -> Color32 {
    let t = intensity.max(0.08).clamp(0.0, 1.0);
    let base = if hovered {
        (216.0, 237.0, 251.0)
    } else {
        (229.0, 243.0, 251.0)
    };
    Color32::from_rgb(
        (base.0 - t * 54.0) as u8,
        (base.1 - t * 40.0) as u8,
        (base.2 - t * 18.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_scale_is_bounded_for_outlier_percentages() {
        assert_eq!(heat_color(-20.0, false), heat_color(0.0, false));
        assert_eq!(heat_color(200.0, false), heat_color(1.0, false));
    }

    #[test]
    fn xp_report_columns_stay_compact_without_disabling_optional_io() {
        let settings = Settings {
            theme: Theme::XpLuna,
            show_io_column: true,
            show_user_column: true,
            ..Settings::default()
        };
        let wide = TableColumns::new(1200.0, &settings);
        assert_eq!(wide.width, 800.0);
        assert!(
            wide.columns
                .iter()
                .any(|(column, _)| matches!(column, Column::Io))
        );
        assert!(
            wide.columns
                .iter()
                .any(|(column, _)| matches!(column, Column::User))
        );
        let narrow = TableColumns::new(650.0, &settings);
        assert!(
            narrow
                .columns
                .iter()
                .all(|(column, _)| !matches!(column, Column::Io))
        );
    }
}
