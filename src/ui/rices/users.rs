//! Windows 10-style Users report backed by Linux UID and live /proc samples.
//! A UID is not the same thing as a signed-in graphical session; the heading
//! and summary deliberately avoid claiming that equivalence.
use super::*;
use crate::theme::ComponentPaint;
use std::collections::BTreeMap;

fn by_uid(rows: Vec<ProcessRow>) -> BTreeMap<u32, Vec<ProcessRow>> {
    let mut groups = BTreeMap::new();
    for row in rows {
        groups
            .entry(row.process.user_id)
            .or_insert_with(Vec::new)
            .push(row);
    }
    groups
}

pub(super) fn page(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let xp = is_xp(app);
    let paint = ComponentPaint::from_settings(&app.settings);
    ui.add_space(if xp { 2.0 } else { 5.0 });
    ui.label(
        RichText::new(if xp { "Process owners" } else { "Users" })
            .size(if xp { 14.0 } else { 27.0 })
            .strong(),
    );
    ui.label(
        RichText::new("Live Linux processes by owner · not a list of graphical login sessions")
            .small()
            .weak(),
    );
    ui.add_space(10.0);
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading process owners…");
        return;
    }

    let groups = by_uid(app.processes());
    if groups.is_empty() {
        ui.label("No matching process owners.");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (uid, rows) in &groups {
                let name = app.users.get(uid).cloned().unwrap_or_else(|| uid.to_string());
                let cpu: f32 = rows.iter().map(|row| row.process.cpu_percent).sum();
                let memory: u64 = rows.iter().fold(0_u64, |sum, row| {
                    sum.saturating_add(row.process.resident_bytes)
                });
                let label = format!("{name}   ·   {} processes   ·   CPU {cpu:.1}%   ·   RSS {}", rows.len(), format::bytes(memory));
                egui::Frame::new()
                    .fill(paint.table(if xp { Color32::from_rgb(250, 249, 241) } else { Color32::WHITE }))
                    .stroke(Stroke::new(1.0_f32, paint.border(if xp { Color32::from_rgb(157, 156, 141) } else { Color32::from_rgb(220, 220, 220) })))
                    .inner_margin(egui::Margin::same(7))
                    .show(ui, |ui| {
                        if paint.table.is_some() {
                            ui.visuals_mut().override_text_color = Some(paint.on_table(Color32::BLACK));
                        }
                        egui::CollapsingHeader::new(label)
                            .id_salt(("taskotto-user", uid))
                            .default_open(*uid == app.my_uid)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add_sized([235.0, 22.0], egui::Label::new(RichText::new("Process").strong()));
                                    ui.add_sized([65.0, 22.0], egui::Label::new(RichText::new("PID").strong()));
                                    ui.add_sized([90.0, 22.0], egui::Label::new(RichText::new("CPU").strong()));
                                    ui.label(RichText::new("Memory (RSS)").strong());
                                });
                                ui.separator();
                                egui::ScrollArea::vertical()
                                    .max_height(260.0)
                                    .show_rows(ui, 26.0, rows.len(), |ui, range| {
                                        for index in range {
                                            let p = &rows[index].process;
                                            let selected = app.selected == Some((p.pid, p.start_ticks));
                                            ui.horizontal(|ui| {
                                                let default_ink = if selected {
                                                    ui.visuals().selection.stroke.color
                                                } else {
                                                    ui.visuals().widgets.inactive.fg_stroke.color
                                                };
                                                let button_ink = if selected {
                                                    paint.on_selection(ui.visuals().selection.bg_fill, default_ink)
                                                } else {
                                                    paint.on_control(ui.visuals().widgets.inactive.bg_fill, default_ink)
                                                };
                                                if ui.add_sized([235.0, 24.0], egui::Button::new(RichText::new(&p.name).color(button_ink)).selected(selected)).clicked() {
                                                    app.selected = Some((p.pid, p.start_ticks));
                                                }
                                                ui.add_sized([65.0, 24.0], egui::Label::new(p.pid.to_string()));
                                                ui.add_sized([90.0, 24.0], egui::Label::new(format!("{:.1}%", p.cpu_percent)));
                                                ui.label(format::bytes(p.resident_bytes));
                                                if ui.small_button("Inspect").clicked() {
                                                    app.selected = Some((p.pid, p.start_ticks));
                                                    app.page = Page::Details;
                                                }
                                            });
                                        }
                                    });
                            });
                    });
                ui.add_space(5.0);
            }
            ui.label(RichText::new("Resident totals sum per-process RSS and may count shared mapped pages more than once.").small().weak());
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping_uses_actual_uid_not_process_names() {
        let rows = [
            (1000, 11, "worker"),
            (1001, 12, "worker"),
            (1000, 13, "helper"),
        ]
        .into_iter()
        .map(|(uid, pid, name)| ProcessRow {
            process: Process {
                user_id: uid,
                pid,
                name: name.to_owned(),
                ..Default::default()
            },
            depth: 0,
        })
        .collect();
        let groups = by_uid(rows);
        assert_eq!(groups.get(&1000).unwrap().len(), 2);
        assert_eq!(groups.get(&1001).unwrap().len(), 1);
    }
}
