//! Windows 98 Close Program-inspired client area. The actual window frame,
//! caption and caption buttons belong to RiceBowl / the window manager.
//! A real Windows 98 Close Program list was of desktop tasks; /proc cannot
//! enumerate compositor windows, so the Linux approximation is labeled.
use super::*;
use egui::{Align2, FontId, Frame, Margin, Rect, StrokeKind};

const FACE: Color32 = Color32::from_rgb(192, 192, 192);
const LIGHT: Color32 = Color32::WHITE;
const SHADOW: Color32 = Color32::from_rgb(128, 128, 128);
const DARK: Color32 = Color32::from_rgb(64, 64, 64);
const INK: Color32 = Color32::BLACK;
const NAVY: Color32 = Color32::from_rgb(0, 0, 128);
const GREEN: Color32 = Color32::from_rgb(0, 187, 0);

const TOOLS: &[(Page, &str)] = &[
    (Page::Processes, "Close Program"),
    (Page::Applications, "Process Report"),
    (Page::Performance, "System Monitor"),
    (Page::Networking, "Network Activity"),
    (Page::Users, "Process Owners"),
    (Page::Details, "Process Information"),
    (Page::Preferences, "Preferences"),
];

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    // Close Program had no in-client navigation or titlebar. Advanced Taskotto
    // pages get a back link, not another imitated OS caption.
    if app.page != Page::Processes {
        egui::TopBottomPanel::top("classic98-client-navigation")
            .frame(Frame::NONE.fill(FACE).inner_margin(Margin::symmetric(9, 5)))
            .show(ctx, |ui| {
                if flat_button(ui, "< Close Program", true, 119.0).clicked() {
                    app.page = Page::Processes;
                }
            });
    }
}

pub(super) fn page(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    match app.page {
        Page::Processes => close_program(app, ui),
        Page::Applications => report(app, ui),
        Page::Performance => system_monitor(app, ui),
        Page::Networking => network(app, ui),
        Page::Users => owners(app, ui),
        Page::Details => information(app, ui),
        Page::Preferences => app.preferences_page(ui),
        _ => close_program(app, ui),
    }
}

fn flat_button(ui: &mut egui::Ui, label: &str, enabled: bool, width: f32) -> egui::Response {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(label).size(12.0).color(INK))
            .fill(FACE)
            .stroke(Stroke::new(1.0_f32, DARK))
            .corner_radius(0)
            .min_size(egui::vec2(width, 27.0)),
    );
    if !response.is_pointer_button_down_on() {
        let rect = response.rect.shrink(1.0);
        ui.painter().line_segment(
            [rect.left_bottom(), rect.left_top()],
            Stroke::new(1.0_f32, LIGHT),
        );
        ui.painter().line_segment(
            [rect.left_top(), rect.right_top()],
            Stroke::new(1.0_f32, LIGHT),
        );
    }
    response
}

fn tools_menu(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.menu_button("Tools", |ui| {
        for &(page, label) in TOOLS {
            if ui.selectable_label(app.page == page, label).clicked() {
                app.page = page;
                ui.close();
            }
        }
        ui.separator();
        if ui.button("New Task (Run…)").clicked() {
            app.new_task.open = true;
            app.new_task.error = None;
            ui.close();
        }
        if ui.button("Refresh    F5").clicked() {
            let _ = app.worker.commands.send(Command::Refresh);
            ui.close();
        }
        let mut show_all = !app.settings.only_my_processes;
        if ui
            .checkbox(&mut show_all, "Include other users in report")
            .changed()
        {
            app.settings.only_my_processes = !show_all;
            app.save_preferences(ui.ctx());
        }
    });
}

fn active_owned(rows: Vec<ProcessRow>, uid: u32) -> Vec<ProcessRow> {
    rows.into_iter()
        .filter(|r| r.process.user_id == uid)
        .collect()
}

fn close_program(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let available = ui.available_width();
    let width = (available - 12.0).clamp(260.0, 615.0);
    // Compute from the main client area before entering nested horizontal UI:
    // a nested auto's available_height can be only its first-row hint.
    let list_height = (ui.available_height() - 160.0).clamp(105.0, 340.0);
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_space(((available - width) / 2.0).max(0.0));
        ui.vertical(|ui| {
            ui.set_max_width(width);
            ui.label(
                RichText::new("To close a program, select it and click End Task.")
                    .size(12.0)
                    .color(INK),
            );
            ui.label(
                RichText::new("Only processes belonging to your Linux user are shown.")
                    .size(11.0)
                    .color(DARK),
            );
            ui.add_space(12.0);
            if app.focus_search || !app.query.is_empty() {
                let response = ui.add_sized(
                    [width, 24.0],
                    egui::TextEdit::singleline(&mut app.query).hint_text("Find task (Ctrl+F)"),
                );
                if app.focus_search {
                    response.request_focus();
                    app.focus_search = false;
                }
            }
            let rows = active_owned(app.processes(), app.my_uid);
            Frame::NONE
                .fill(LIGHT)
                .stroke(Stroke::new(1.0_f32, SHADOW))
                .inner_margin(Margin::same(2))
                .show(ui, |ui| {
                    ui.set_min_width(width - 6.0);
                    ui.set_min_height(list_height);
                    if app.snapshot.is_none() {
                        ui.spinner();
                        ui.label("Reading running tasks…");
                    } else if rows.is_empty() {
                        ui.label("No matching user processes.");
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(list_height - 5.0)
                            .auto_shrink([false, false])
                            .show_rows(ui, 23.0, rows.len(), |ui, range| {
                                for index in range {
                                    let process = &rows[index].process;
                                    task_entry(app, ui, process);
                                }
                            });
                    }
                });
            ui.add_space(9.0);
            ui.horizontal(|ui| {
                let can_end = app.selected_process().is_some_and(can_signal_process);
                if flat_button(ui, "End Task", can_end, 94.0).clicked() {
                    app.queue_action(SignalAction::Terminate);
                }
                flat_button(ui, "Shut Down", false, 94.0).on_disabled_hover_text(
                    "Use your desktop's session menu; Taskotto never powers off the host implicitly.",
                );
                if flat_button(ui, "Cancel", true, 94.0).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                tools_menu(app, ui);
            });
            ui.add_space(9.0);
            ui.label(
                RichText::new("Ending a Linux process may discard unsaved data. Confirm each action.")
                    .size(10.5)
                    .color(DARK),
            );
        });
    });
}

fn task_entry(app: &mut TaskottoApp, ui: &mut egui::Ui, process: &Process) {
    let selected = app.selected == Some((process.pid, process.start_ticks));
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(190.0), 22.0),
        Sense::click(),
    );
    if selected || response.hovered() {
        ui.painter().rect_filled(
            rect,
            0.0,
            if selected {
                NAVY
            } else {
                Color32::from_rgb(225, 225, 245)
            },
        );
    }
    let icon = Rect::from_min_size(
        rect.left_top() + egui::vec2(4.0, 4.0),
        egui::vec2(13.0, 13.0),
    );
    ui.painter().rect_filled(icon, 0.0, FACE);
    ui.painter()
        .rect_stroke(icon, 0.0, Stroke::new(1.0_f32, DARK), StrokeKind::Inside);
    ui.painter()
        .rect_filled(icon.shrink(3.0), 0.0, Color32::from_rgb(0, 128, 128));
    ui.painter().with_clip_rect(rect.shrink(3.0)).text(
        rect.left_center() + egui::vec2(23.0, 0.0),
        Align2::LEFT_CENTER,
        &process.name,
        FontId::proportional(12.0),
        if selected { LIGHT } else { INK },
    );
    if response.clicked() || response.double_clicked() {
        app.selected = Some((process.pid, process.start_ticks));
    }
    if response.double_clicked() {
        app.page = Page::Details;
    }
    response.on_hover_text(&process.command).context_menu(|ui| {
        if ui.button("Process Information").clicked() {
            app.selected = Some((process.pid, process.start_ticks));
            app.page = Page::Details;
            ui.close();
        }
        if ui
            .add_enabled(can_signal_process(process), egui::Button::new("End Task…"))
            .clicked()
        {
            app.selected = Some((process.pid, process.start_ticks));
            app.queue_action(SignalAction::Terminate);
            ui.close();
        }
    });
}

fn report(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.heading("Process Report");
    ui.label(
        RichText::new("Advanced Taskotto view · real Linux processes, not desktop windows").small(),
    );
    if app.focus_search || !app.query.is_empty() {
        let search = ui.text_edit_singleline(&mut app.query);
        if app.focus_search {
            search.request_focus();
            app.focus_search = false;
        }
    }
    ui.add_space(10.0);
    let rows = app.processes();
    let height = (ui.available_height() - 60.0).max(90.0);
    Frame::NONE
        .fill(LIGHT)
        .stroke(Stroke::new(1.0_f32, SHADOW))
        .show(ui, |ui| {
            ui.set_min_height(height);
            let width = ui.available_width().max(390.0);
            let fixed = 78.0 + 90.0 + 74.0;
            let specs = [
                ("Name", (width - fixed).max(135.0), SortBy::Name),
                ("PID", 78.0, SortBy::Pid),
                ("Memory", 90.0, SortBy::Memory),
                ("CPU %", 74.0, SortBy::Cpu),
            ];
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for &(title, size, sort) in &specs {
                    if ui
                        .add_sized([size, 24.0], egui::Button::new(title).corner_radius(0))
                        .clicked()
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
                ui.label("No processes match the current filter.");
            } else {
                egui::ScrollArea::vertical()
                    .max_height(height - 26.0)
                    .show_rows(ui, 24.0, rows.len(), |ui, range| {
                        for index in range {
                            let p = &rows[index].process;
                            let selected = app.selected == Some((p.pid, p.start_ticks));
                            let values = [
                                p.name.clone(),
                                p.pid.to_string(),
                                format::bytes(p.resident_bytes),
                                format!("{:.1}", p.cpu_percent),
                            ];
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                for (text, &(_, size, _)) in values.iter().zip(&specs) {
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(size, 23.0),
                                        Sense::click(),
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        0.0,
                                        if selected {
                                            NAVY
                                        } else if index % 2 == 0 {
                                            LIGHT
                                        } else {
                                            Color32::from_rgb(245, 245, 245)
                                        },
                                    );
                                    ui.painter().with_clip_rect(rect.shrink(4.0)).text(
                                        rect.left_center() + egui::vec2(6.0, 0.0),
                                        Align2::LEFT_CENTER,
                                        text,
                                        FontId::proportional(12.0),
                                        if selected { LIGHT } else { INK },
                                    );
                                    if response.clicked() || response.double_clicked() {
                                        app.selected = Some((p.pid, p.start_ticks));
                                    }
                                    if response.double_clicked() {
                                        app.page = Page::Details;
                                    }
                                    response.context_menu(|ui| {
                                        if ui.button("Information…").clicked() {
                                            app.selected = Some((p.pid, p.start_ticks));
                                            app.page = Page::Details;
                                            ui.close();
                                        }
                                        if ui
                                            .add_enabled(
                                                can_signal_process(p),
                                                egui::Button::new("End Task…"),
                                            )
                                            .clicked()
                                        {
                                            app.selected = Some((p.pid, p.start_ticks));
                                            app.queue_action(SignalAction::Terminate);
                                            ui.close();
                                        }
                                    });
                                }
                            });
                        }
                    });
            }
        });
}

fn classic_graph(
    ui: &mut egui::Ui,
    series: &[(&std::collections::VecDeque<f32>, Color32)],
    max: f32,
    height: f32,
) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(180.0), height),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
    for fraction in [0.25_f32, 0.5, 0.75] {
        let y = rect.bottom() - rect.height() * fraction;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(0.7_f32, Color32::from_rgb(0, 60, 0)),
        );
    }
    for &(data, color) in series {
        if data.len() > 1 {
            let points = super::super::chart_points(data, rect.shrink(2.0), max.max(1.0));
            ui.painter()
                .add(egui::Shape::line(points, Stroke::new(1.5_f32, color)));
        }
    }
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(2.0_f32, SHADOW), StrokeKind::Inside);
}

fn system_monitor(app: &TaskottoApp, ui: &mut egui::Ui) {
    let compact = ui.available_height() < 550.0;
    ui.heading("System Monitor");
    ui.label(RichText::new("Live Linux measurements · classic Windows 98-inspired graphs").small());
    ui.add_space(12.0);
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        ui.label("Waiting for system samples…");
        return;
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        Frame::NONE
            .fill(FACE)
            .stroke(Stroke::new(1.0_f32, SHADOW))
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.label(RichText::new("Processor usage").strong());
                classic_graph(
                    ui,
                    &[(&app.history.cpu, GREEN)],
                    100.0,
                    if compact { 110.0 } else { 135.0 },
                );
                ui.add_space(12.0);
                ui.label(RichText::new("Physical memory used").strong());
                classic_graph(
                    ui,
                    &[(&app.history.memory, Color32::from_rgb(50, 220, 220))],
                    100.0,
                    if compact { 76.0 } else { 96.0 },
                );
                ui.add_space(11.0);
                egui::Grid::new("98-system-stats")
                    .num_columns(2)
                    .spacing([24.0, 7.0])
                    .show(ui, |ui| {
                        for (label, value) in [
                            ("CPU", format!("{:.1}%", snapshot.cpu_percent)),
                            ("Physical memory", format::bytes(snapshot.memory.total)),
                            ("Memory used", format::bytes(snapshot.memory.used())),
                            ("Processes", snapshot.processes.len().to_string()),
                        ] {
                            ui.label(label);
                            ui.label(value);
                            ui.end_row();
                        }
                    });
            });
    });
}

fn network(app: &TaskottoApp, ui: &mut egui::Ui) {
    ui.heading("Network Activity");
    ui.label(RichText::new("Aggregate Linux network interfaces (excluding loopback)").small());
    ui.add_space(11.0);
    if let Some(snapshot) = &app.snapshot {
        let max = super::super::graph_max(&app.history.received, &app.history.sent);
        classic_graph(
            ui,
            &[
                (&app.history.received, GREEN),
                (&app.history.sent, Color32::YELLOW),
            ],
            max,
            180.0,
        );
        ui.add_space(15.0);
        ui.label(format!(
            "Received: {}",
            format::rate(snapshot.rates.received_per_second)
        ));
        ui.label(format!(
            "Sent: {}",
            format::rate(snapshot.rates.sent_per_second)
        ));
        ui.label(
            RichText::new("Virtual network interfaces may count traffic more than once.").small(),
        );
    } else {
        ui.spinner();
    }
}

fn owners(app: &TaskottoApp, ui: &mut egui::Ui) {
    ui.heading("Process Owners");
    ui.label(RichText::new("UID owners of running Linux processes, not desktop sessions").small());
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        return;
    };
    let mut totals = std::collections::BTreeMap::<u32, (usize, u64)>::new();
    for p in &snapshot.processes {
        let (count, memory) = totals.entry(p.user_id).or_default();
        *count += 1;
        *memory += p.resident_bytes;
    }
    ui.add_space(10.0);
    Frame::NONE
        .fill(LIGHT)
        .stroke(Stroke::new(1.0_f32, SHADOW))
        .show(ui, |ui| {
            egui::Grid::new("98-owner-stats")
                .num_columns(3)
                .spacing([27.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("User");
                    ui.strong("Processes");
                    ui.strong("Total RSS");
                    ui.end_row();
                    for (uid, (count, memory)) in totals {
                        ui.label(
                            app.users
                                .get(&uid)
                                .cloned()
                                .unwrap_or_else(|| uid.to_string()),
                        );
                        ui.label(count.to_string());
                        ui.label(format::bytes(memory));
                        ui.end_row();
                    }
                });
        });
}

fn information(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.heading("Process Information");
    let Some(p) = app.selected_process().cloned() else {
        ui.label("Select a process in Close Program or Process Report first.");
        return;
    };
    Frame::NONE
        .fill(LIGHT)
        .stroke(Stroke::new(1.0_f32, SHADOW))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            egui::Grid::new("98-process-info")
                .num_columns(2)
                .spacing([18.0, 9.0])
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Name", p.name.clone()),
                        ("PID", p.pid.to_string()),
                        ("Parent PID", p.parent_pid.to_string()),
                        (
                            "User",
                            app.users
                                .get(&p.user_id)
                                .cloned()
                                .unwrap_or_else(|| p.user_id.to_string()),
                        ),
                        ("State", state_name(p.state).to_owned()),
                        ("CPU", format!("{:.1}%", p.cpu_percent)),
                        ("RAM (RSS)", format::bytes(p.resident_bytes)),
                        ("Threads", p.threads.to_string()),
                    ] {
                        ui.label(label);
                        ui.label(value);
                        ui.end_row();
                    }
                });
        });
    ui.add_space(10.0);
    if flat_button(ui, "End Task…", can_signal_process(&p), 100.0).clicked() {
        app.queue_action(SignalAction::Terminate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win98_uses_dialog_first_no_historical_tabs() {
        assert_eq!(TOOLS[0], (Page::Processes, "Close Program"));
        assert!(TOOLS.iter().all(|(_, label)| !label.is_empty()));
    }

    #[test]
    fn close_program_only_lists_the_users_real_processes() {
        let rows = [1000, 1001, 1000]
            .into_iter()
            .enumerate()
            .map(|(i, uid)| ProcessRow {
                process: Process {
                    pid: i as i32 + 10,
                    user_id: uid,
                    ..Process::default()
                },
                depth: 0,
            })
            .collect();
        let owned = active_owned(rows, 1000);
        assert_eq!(owned.len(), 2);
        assert!(owned.iter().all(|row| row.process.user_id == 1000));
    }
}
