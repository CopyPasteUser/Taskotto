//! XP Applications-style report. Linux and especially Wayland do not expose a
//! compositor-independent map from processes to open graphical windows; show
//! real user-owned processes and communicate that limitation prominently.
use super::*;

fn owned_rows(rows: Vec<ProcessRow>, uid: u32) -> Vec<ProcessRow> {
    rows.into_iter()
        .filter(|row| row.process.user_id == uid)
        .collect()
}

pub(super) fn page(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.label(
        RichText::new("Applications · current user's Linux processes (not a window list)")
            .small()
            .weak(),
    );
    ui.add_space(4.0);
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading processes…");
        return;
    }
    let rows = owned_rows(app.processes(), app.my_uid);
    let frame_height = (ui.available_height() - 48.0).max(100.0);
    egui::Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(138, 137, 124)))
        .show(ui, |ui| {
            ui.set_min_height(frame_height);
            let (header, _) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), 24.0), Sense::hover());
            ui.painter()
                .rect_filled(header, 0.0, Color32::from_rgb(245, 244, 234));
            ui.painter().text(
                header.left_center() + egui::vec2(9.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "Task",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(35, 35, 35),
            );
            ui.painter().text(
                egui::pos2(header.left() + (header.width() * 0.62), header.center().y),
                egui::Align2::LEFT_CENTER,
                "Status",
                egui::FontId::proportional(12.0),
                Color32::from_rgb(35, 35, 35),
            );
            ui.separator();
            if rows.is_empty() {
                ui.label("No current-user processes match the filter.");
            } else {
                egui::ScrollArea::vertical()
                    .max_height((frame_height - 29.0).max(55.0))
                    .auto_shrink([false, false])
                    .show_rows(ui, 25.0, rows.len(), |ui, range| {
                        for index in range {
                            let p = &rows[index].process;
                            let selected = app.selected == Some((p.pid, p.start_ticks));
                            ui.horizontal(|ui| {
                                let width = (ui.available_width() * 0.60).max(160.0);
                                let (rect, response) =
                                    ui.allocate_exact_size(egui::vec2(width, 23.0), Sense::click());
                                let background = if selected {
                                    Color32::from_rgb(49, 95, 162)
                                } else if response.hovered() {
                                    Color32::from_rgb(226, 233, 244)
                                } else {
                                    Color32::WHITE
                                };
                                ui.painter().rect_filled(rect, 0.0, background);
                                ui.painter().text(
                                    rect.left_center() + egui::vec2(8.0, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    &p.name,
                                    egui::FontId::proportional(13.0),
                                    if selected {
                                        Color32::WHITE
                                    } else {
                                        Color32::BLACK
                                    },
                                );
                                if response.clicked() {
                                    app.selected = Some((p.pid, p.start_ticks));
                                }
                                if response.double_clicked() {
                                    app.selected = Some((p.pid, p.start_ticks));
                                    app.page = Page::Details;
                                }
                                ui.label(state_name(p.state));
                            });
                        }
                    });
            }
        });
    ui.add_space(5.0);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.button("New Task…").clicked() {
            app.new_task.open = true;
            app.new_task.error = None;
        }
        ui.add_enabled(false, egui::Button::new("Switch To"))
            .on_disabled_hover_text("Window activation needs a compositor-specific integration.");
        if ui
            .add_enabled(
                app.selected_process().is_some(),
                egui::Button::new("Properties…"),
            )
            .clicked()
        {
            app.page = Page::Details;
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applications_filter_never_claims_other_users_processes() {
        let rows = [1000, 1001, 1000]
            .into_iter()
            .enumerate()
            .map(|(index, uid)| ProcessRow {
                process: Process {
                    pid: index as i32 + 10,
                    user_id: uid,
                    ..Default::default()
                },
                depth: 0,
            })
            .collect();
        assert_eq!(owned_rows(rows, 1000).len(), 2);
    }
}
