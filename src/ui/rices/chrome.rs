use super::*;
use crate::theme::{ComponentPaint, readable_text};
use egui::{Align2, FontId, Frame, Margin, Rect};

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    // Render the application menu only: RiceBowl/the window manager owns
    // the caption, border and window control buttons for every rice.
    menu_bar(app, ctx);
}

pub(super) fn menu_bar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let xp = is_xp(app);
    let paint = ComponentPaint::from_settings(&app.settings);
    let surface = if xp {
        Color32::from_rgb(236, 233, 216)
    } else {
        Color32::WHITE
    };
    egui::TopBottomPanel::top("rice-menu-strip")
        .frame(
            Frame::NONE
                .fill(paint.control(surface))
                .inner_margin(Margin::symmetric(if xp { 6 } else { 8 }, 2)),
        )
        .show(ctx, |ui| {
            if paint.control.is_some() || ThemeStyle::from_settings(&app.settings).dark {
                ui.visuals_mut().override_text_color =
                    Some(paint.on_control(surface, Color32::BLACK));
            }
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = if xp { 5.0 } else { 10.0 };
                ui.menu_button("File", |ui| {
                    if ui.button("New Task (Run…)").clicked() {
                        app.new_task.open = true;
                        app.new_task.error = None;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Refresh now   F5").clicked() {
                        let _ = app.worker.commands.send(Command::Refresh);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Close Taskotto").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });
                ui.menu_button("Options", |ui| {
                    let mut all_users = !app.settings.only_my_processes;
                    if ui
                        .checkbox(&mut all_users, "Show processes from all users")
                        .changed()
                    {
                        app.settings.only_my_processes = !all_users;
                        app.save_preferences(ctx);
                    }
                    if ui
                        .checkbox(&mut app.settings.show_kernel_threads, "Show kernel threads")
                        .changed()
                    {
                        app.save_preferences(ctx);
                    }
                    ui.menu_button("Update speed", |ui| {
                        for (ms, label) in [
                            (500, "High (0.5 s)"),
                            (1_500, "Normal (1.5 s)"),
                            (5_000, "Low (5 s)"),
                            (10_000, "Very low (10 s)"),
                        ] {
                            if ui
                                .selectable_label(app.settings.refresh_ms == ms, label)
                                .clicked()
                            {
                                app.settings.refresh_ms = ms;
                                app.save_preferences(ctx);
                                ui.close();
                            }
                        }
                    });
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Find processes…    Ctrl+F").clicked() {
                        app.page = Page::Processes;
                        app.focus_search = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui
                        .checkbox(&mut app.settings.tree_view, "Process tree")
                        .changed()
                    {
                        app.save_preferences(ctx);
                    }
                    if ui
                        .checkbox(&mut app.settings.show_io_column, "I/O column")
                        .changed()
                    {
                        app.save_preferences(ctx);
                    }
                    if ui
                        .checkbox(&mut app.settings.show_user_column, "User column")
                        .changed()
                    {
                        app.save_preferences(ctx);
                    }
                    ui.separator();
                    if ui.button("Customize appearance…").clicked() {
                        app.page = Page::Preferences;
                        ui.close();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About Taskotto").clicked() {
                        app.notification = Some(format!(
                            "Taskotto {} · Linux task manager · built with Rust",
                            env!("CARGO_PKG_VERSION")
                        ));
                        ui.close();
                    }
                });
            });
        });
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    let xp = is_xp(app);
    let paint = ComponentPaint::from_settings(&app.settings);
    let surface = if xp {
        Color32::from_rgb(236, 233, 216)
    } else {
        Color32::WHITE
    };
    egui::TopBottomPanel::top("rice-navigation-tabs")
        .frame(
            Frame::NONE
                .fill(paint.control(surface))
                .inner_margin(Margin::symmetric(8, if xp { 1 } else { 2 })),
        )
        .show(ctx, |ui| {
            if paint.control.is_some() || ThemeStyle::from_settings(&app.settings).dark {
                ui.visuals_mut().override_text_color =
                    Some(paint.on_control(surface, Color32::BLACK));
            }
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = if xp { 2.0 } else { 0.0 };
                let tabs: &[(Page, &str)] = if xp {
                    &[
                        (Page::Applications, "Applications"),
                        (Page::Processes, "Processes"),
                        (Page::Performance, "Performance"),
                        (Page::Networking, "Networking"),
                        (Page::Users, "Users"),
                    ]
                } else {
                    &[
                        (Page::Processes, "Processes"),
                        (Page::Performance, "Performance"),
                        (Page::AppHistory, "App history"),
                        (Page::Startup, "Startup"),
                        (Page::Users, "Users"),
                        (Page::Details, "Details"),
                        (Page::Services, "Services"),
                    ]
                };
                // Keep the seven Windows 10 tabs on one line down to the
                // supported minimum window width, rather than unexpectedly
                // wrapping Services onto a second row.
                let modern_tab_width = (ui.available_width() / tabs.len() as f32).clamp(74.0, 98.0);
                for &(page, label) in tabs {
                    let selected = app.page == page;
                    let title = if selected && paint.selection.is_some() {
                        RichText::new(label)
                            .color(paint.on_selection(Color32::WHITE, Color32::BLACK))
                    } else if selected
                        && (paint.control.is_some()
                            || ThemeStyle::from_settings(&app.settings).dark)
                    {
                        RichText::new(label).color(Color32::BLACK)
                    } else if !selected && paint.control.is_some() {
                        RichText::new(label).color(paint.on_control(surface, Color32::BLACK))
                    } else {
                        RichText::new(label)
                    };
                    let button = if xp {
                        egui::Button::new(title)
                            .fill(if selected {
                                paint.selection(Color32::WHITE)
                            } else {
                                paint.control(surface)
                            })
                            .stroke(Stroke::new(
                                1.0_f32,
                                paint.border(Color32::from_rgb(135, 134, 121)),
                            ))
                            .corner_radius(0)
                    } else {
                        egui::Button::new(title)
                            .fill(if selected {
                                paint.selection(Color32::from_rgb(242, 249, 254))
                            } else {
                                paint.control(Color32::WHITE)
                            })
                            .stroke(Stroke::NONE)
                            .corner_radius(0)
                    };
                    let response = ui.add_sized(
                        [
                            if xp { 95.0 } else { modern_tab_width },
                            if xp { 26.0 } else { 30.0 },
                        ],
                        button,
                    );
                    if xp && selected {
                        // A raised selected tab joins the cream body below.
                        let r = response.rect;
                        ui.painter().line_segment(
                            [
                                r.left_top() + egui::vec2(1.0, 1.0),
                                r.right_top() + egui::vec2(-1.0, 1.0),
                            ],
                            Stroke::new(1.0_f32, paint.selection(Color32::WHITE)),
                        );
                        ui.painter().line_segment(
                            [
                                r.left_bottom() + egui::vec2(2.0, -1.0),
                                r.right_bottom() + egui::vec2(-2.0, -1.0),
                            ],
                            Stroke::new(2.0_f32, paint.selection(Color32::WHITE)),
                        );
                    }
                    if !xp && selected {
                        let r = response.rect;
                        ui.painter().rect_filled(
                            Rect::from_min_max(
                                egui::pos2(r.left(), r.bottom() - 2.5),
                                r.right_bottom(),
                            ),
                            0.0,
                            accent(app),
                        );
                    }
                    if response.clicked() {
                        app.page = page;
                    }
                }
            });
            subtle_separator(
                ui,
                paint.border(if xp {
                    Color32::from_rgb(134, 132, 117)
                } else {
                    Color32::from_rgb(221, 221, 221)
                }),
            );
        });
}

/// A user-requested vertical navigation option. Windows 10's original task
/// manager uses horizontal tabs; this is deliberately opt-in through the
/// existing Sidebar navigation setting, not called the historical default.
pub(super) fn vertical_navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    let paint = ComponentPaint::from_settings(&app.settings);
    egui::SidePanel::left("rice-win10-vertical-navigation")
        .resizable(false)
        .exact_width(188.0)
        .frame(
            Frame::NONE
                .fill(paint.control(Color32::from_rgb(248, 249, 250)))
                .inner_margin(Margin::symmetric(8, 12)),
        )
        .show(ctx, |ui| {
            if paint.control.is_some() || ThemeStyle::from_settings(&app.settings).dark {
                ui.visuals_mut().override_text_color =
                    Some(paint.on_control(Color32::WHITE, Color32::BLACK));
            }
            ui.label(
                RichText::new("TASKOTTO")
                    .size(12.0)
                    .strong()
                    .color(paint.on_control(Color32::WHITE, Color32::from_rgb(82, 90, 98))),
            );
            ui.add_space(10.0);
            for (page, title) in [
                (Page::Processes, "Processes"),
                (Page::Performance, "Performance"),
                (Page::AppHistory, "App history"),
                (Page::Startup, "Startup"),
                (Page::Users, "Users"),
                (Page::Details, "Details"),
                (Page::Services, "Services"),
            ] {
                let selected = app.page == page;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(ui.available_width(), 38.0), Sense::click());
                if selected || response.hovered() {
                    let fill = if selected {
                        paint.selection(Color32::from_rgb(222, 236, 246))
                    } else {
                        paint.hover(Color32::from_rgb(236, 240, 243))
                    };
                    ui.painter().rect_filled(rect, 1.0, fill);
                }
                if selected {
                    ui.painter().rect_filled(
                        Rect::from_min_size(rect.min, egui::vec2(3.0, rect.height())),
                        0.0,
                        accent(app),
                    );
                }
                ui.painter().text(
                    rect.left_center() + egui::vec2(14.0, 0.0),
                    Align2::LEFT_CENTER,
                    title,
                    FontId::proportional(14.0),
                    if selected && paint.selection.is_some() {
                        readable_text(paint.selection(Color32::from_rgb(222, 236, 246)))
                    } else if response.hovered() && paint.hover.is_some() {
                        readable_text(paint.hover(Color32::from_rgb(236, 240, 243)))
                    } else {
                        paint.on_control(Color32::WHITE, Color32::from_rgb(30, 35, 43))
                    },
                );
                if response.has_focus() {
                    ui.painter().rect_stroke(
                        rect.shrink(2.0),
                        0.0,
                        Stroke::new(2.0_f32, accent(app)),
                        egui::StrokeKind::Inside,
                    );
                }
                if response.clicked() {
                    app.page = page;
                }
                ui.add_space(2.0);
            }
            ui.separator();
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("Taskotto Settings"),
                )
                .clicked()
            {
                app.page = Page::Preferences;
            }
            ui.label(RichText::new("Custom vertical layout").size(10.0).weak());
        });
}

pub(super) fn statusbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let xp = is_xp(app);
    let paint = ComponentPaint::from_settings(&app.settings);
    let color = if xp {
        Color32::from_rgb(236, 233, 216)
    } else {
        Color32::WHITE
    };
    let count = app.snapshot.as_ref().map_or(0, |s| s.processes.len());
    let cpu = app.snapshot.as_ref().map_or(0.0, |s| s.cpu_percent);
    let memory = app
        .snapshot
        .as_ref()
        .map_or(0.0, |s| s.memory.used_fraction() * 100.0);
    let can_act = app.selected_process().is_some_and(can_signal_process);
    let process_action_page = matches!(app.page, Page::Processes | Page::Details | Page::Users);
    egui::TopBottomPanel::bottom("rice-statusbar")
        .frame(
            Frame::NONE
                .fill(paint.control(color))
                .inner_margin(Margin::symmetric(12, if xp { 5 } else { 8 })),
        )
        .show(ctx, |ui| {
            if paint.control.is_some() || ThemeStyle::from_settings(&app.settings).dark {
                ui.visuals_mut().override_text_color =
                    Some(paint.on_control(color, Color32::BLACK));
            }
            subtle_separator(
                ui,
                paint.border(if xp {
                    Color32::from_rgb(163, 159, 144)
                } else {
                    Color32::from_rgb(221, 221, 221)
                }),
            );
            ui.horizontal(|ui| {
                if xp {
                    ui.label(format!("Processes: {count}"));
                    ui.separator();
                    ui.label(format!("CPU Usage: {cpu:.0}%"));
                    ui.separator();
                    ui.label(format!("Memory Usage: {memory:.0}%"));
                } else {
                    ui.label(
                        RichText::new(format!(
                            "{count} processes   ·   CPU {cpu:.0}%   ·   Memory {memory:.0}%"
                        ))
                        .small()
                        .weak(),
                    );
                }
                if process_action_page {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                can_act,
                                egui::Button::new(if xp { "End Process" } else { "End task" }),
                            )
                            .clicked()
                        {
                            app.queue_action(SignalAction::Terminate);
                        }
                    });
                }
            });
        });
}
