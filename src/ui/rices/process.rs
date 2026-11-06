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
    fn new(available: f32, app: &TaskottoApp) -> Self {
        let width = available.max(440.0);
        let xp = is_xp(app);
        let mut columns = vec![
            (Column::Pid, if xp { 80.0 } else { 78.0 }),
            (Column::Cpu, if xp { 95.0 } else { 106.0 }),
            (Column::Memory, 128.0),
        ];
        if app.settings.show_io_column && width >= 875.0 {
            columns.push((Column::Io, 133.0));
        }
        if app.settings.show_user_column && width >= 1040.0 {
            columns.push((Column::User, 128.0));
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
    if is_xp(app) || app.settings.tree_view {
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
    if xp {
        ui.label(RichText::new("Processes").size(13.0).strong());
    } else {
        ui.add_space(5.0);
        ui.heading(RichText::new("Processes").size(30.0));
        ui.add_space(8.0);
    }
    ui.horizontal_wrapped(|ui| {
        let search = ui.add_sized(
            [
                ui.available_width().min(if xp { 250.0 } else { 300.0 }),
                26.0,
            ],
            egui::TextEdit::singleline(&mut app.query).hint_text("Search processes…"),
        );
        if app.focus_search {
            search.request_focus();
            app.focus_search = false;
        }
        if ui.button("Refresh").clicked() {
            let _ = app.worker.commands.send(Command::Refresh);
        }
        if ui
            .selectable_label(app.settings.tree_view, "Process tree")
            .clicked()
        {
            app.settings.tree_view = !app.settings.tree_view;
            app.save_preferences(ui.ctx());
        }
        if !xp {
            ui.label(
                RichText::new("Click a column to sort · right-click for actions")
                    .small()
                    .weak(),
            );
        }
    });
    ui.add_space(if xp { 5.0 } else { 12.0 });
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Loading Linux process information…");
        return;
    }
    let rows = app.processes();
    if rows.is_empty() {
        ui.add_space(32.0);
        ui.vertical_centered(|ui| {
            ui.heading("No matching processes");
            if ui.button("Clear search").clicked() {
                app.query.clear();
            }
        });
        return;
    }
    let layout = TableColumns::new(ui.available_width() - 16.0, app);
    header(app, ui, &layout);
    let entries = grouped_rows(app, rows);
    let row_height = app
        .settings
        .table_row_height
        .unwrap_or(if xp { 27.0 } else { 37.0 });
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, row_height, entries.len(), |ui, range| {
            for index in range {
                match &entries[index] {
                    Entry::Section(label, count) => section(ui, &layout, label, *count, row_height),
                    Entry::Process(row) => process_row(app, ui, &layout, row, index, row_height),
                }
            }
        });
}

fn header(app: &mut TaskottoApp, ui: &mut egui::Ui, layout: &TableColumns) {
    let xp = is_xp(app);
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(layout.width, if xp { 30.0 } else { 69.0 }),
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
            Column::Name => "Name",
            Column::Pid => "PID",
            Column::Cpu => "CPU",
            Column::Memory => "Memory",
            Column::Io => "Disk I/O",
            Column::User => "User",
        };
        if !xp && matches!(column, Column::Cpu | Column::Memory) {
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
            FontId::proportional(if xp { 12.0 } else { 13.0 }),
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
            Column::Cpu => format!("{:.1}%", p.cpu_percent),
            Column::Memory => format::bytes(p.resident_bytes),
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
            egui::pos2(cell.left() + if xp { 7.0 } else { 10.0 }, cell.center().y),
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
}
