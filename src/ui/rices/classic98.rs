//! Windows 98's compact Close Program client, without a simulated title bar.
//! RiceBowl/the window manager owns the external frame and caption buttons.
//! We cannot enumerate Wayland/X11 application windows from portable procfs,
//! so this list explicitly shows current-user processes instead.
use super::*;
use crate::theme::{ComponentPaint, readable_text};
use egui::{Align, Align2, FontId, Frame, Margin, StrokeKind};

const FACE: Color32 = Color32::from_rgb(192, 192, 192);
const WHITE: Color32 = Color32::WHITE;
const SHADOW: Color32 = Color32::from_rgb(128, 128, 128);
const BLACK: Color32 = Color32::BLACK;
const NAVY: Color32 = Color32::from_rgb(0, 0, 128);

// The Windows 98 dialog is intentionally the first screen, not an extra
// "Applications" tab in a modern task manager.
const TOOLS: [(Page, &str); 6] = [
    (Page::Applications, "Close Program"),
    (Page::Processes, "Processes"),
    (Page::Performance, "Performance"),
    (Page::Networking, "Network"),
    (Page::Users, "Users"),
    (Page::Preferences, "Options"),
];

pub(super) fn navigation(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.page == Page::Applications {
        return;
    }
    let paint = ComponentPaint::from_settings(&app.settings);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 3.0;
        let width = ((ui.available_width() - 15.0) / TOOLS.len() as f32).clamp(74.0, 99.0);
        for (page, label) in TOOLS {
            let label = if page == Page::Applications && width < 90.0 {
                "Close"
            } else {
                label
            };
            if bevel_button(ui, label, app.page == page, [width, 26.0], paint).clicked() {
                app.page = page;
            }
        }
    });
    ui.add_space(3.0);
    bevel_line(ui, paint);
    ui.add_space(6.0);
}

pub(super) fn statusbar(app: &TaskottoApp, ctx: &egui::Context) {
    if app.page == Page::Applications {
        return;
    }
    let paint = ComponentPaint::from_settings(&app.settings);
    egui::TopBottomPanel::bottom("classic98-status")
        .frame(Frame::NONE.fill(FACE).inner_margin(Margin::symmetric(9, 5)))
        .show(ctx, |ui| {
            bevel_line(ui, paint);
            ui.horizontal_wrapped(|ui| {
                if let Some(snapshot) = &app.snapshot {
                    ui.label(format!("Processes: {}", snapshot.processes.len()));
                    ui.separator();
                    ui.label(format!("CPU: {:.0}%", snapshot.cpu_percent));
                    ui.separator();
                    ui.label(format!(
                        "Memory: {:.0}%",
                        snapshot.memory.used_fraction() * 100.0
                    ));
                } else {
                    ui.label("Reading Linux system information...");
                }
            });
        });
}

fn user_processes(app: &TaskottoApp) -> Vec<ProcessRow> {
    let mut rows = app
        .processes()
        .into_iter()
        .filter(|row| close_program_visible(&row.process, app.my_uid))
        .collect::<Vec<_>>();
    // Win98's simple list is alphabetical regardless of the advanced report's
    // sort preference, so the two surfaces never fight over shared settings.
    rows.sort_by(|a, b| {
        a.process
            .name
            .to_ascii_lowercase()
            .cmp(&b.process.name.to_ascii_lowercase())
            .then(a.process.pid.cmp(&b.process.pid))
    });
    rows
}

fn close_program_visible(process: &Process, uid: u32) -> bool {
    process.user_id == uid && !is_kernel_thread(process)
}

fn visible_selected(app: &TaskottoApp, rows: &[ProcessRow]) -> bool {
    let Some(selected) = app.selected else {
        return false;
    };
    rows.iter().any(|row| {
        (row.process.pid, row.process.start_ticks) == selected && can_signal_process(&row.process)
    })
}

/// Only a single pane and action column are present by default, mirroring the
/// actual Close Program dialog rather than the later multi-tab Task Manager.
pub(super) fn close_program(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let paint = ComponentPaint::from_settings(&app.settings);
    ui.add_space(8.0);
    let content_width = ui.available_width().min(620.0);
    let left_padding = ((ui.available_width() - content_width) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(left_padding);
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, ui.available_height()),
            egui::Layout::top_down(Align::Min),
            |ui| {
                ui.label(
                    RichText::new("Select a program and click End Task to close it.").color(BLACK),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new("Shows Linux processes owned by you, not desktop windows.")
                        .small()
                        .color(Color32::from_rgb(60, 60, 60)),
                );
                ui.add_space(10.0);
                ui.label(RichText::new("Running programs:").color(BLACK));
                ui.add_space(3.0);

                let rows = user_processes(app);
                let row_height = 23.0 * app.settings.font_scale.max(1.0);
                let width = ui.available_width();
                // At minimum window size the list must remain scrollable and
                // the close/action buttons stay reachable.
                let list_height = (ui.available_height() - 45.0).clamp(130.0, 315.0);
                let actions_width = if width < 510.0 { 108.0 } else { 116.0 };
                ui.horizontal_top(|ui| {
                    let list_width = (width - actions_width - 12.0).max(200.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(list_width, list_height),
                        egui::Layout::top_down(Align::Min),
                        |ui| {
                            inset_list(ui, list_height, paint, |ui| {
                                if app.snapshot.is_none() {
                                    ui.spinner();
                                    ui.colored_label(paint.on_table(BLACK), "Reading /proc...");
                                } else if rows.is_empty() {
                                    ui.colored_label(
                                        paint.on_table(BLACK),
                                        "No matching user processes.",
                                    );
                                } else {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .max_height(list_height - 9.0)
                                        .show_rows(ui, row_height, rows.len(), |ui, range| {
                                            for index in range {
                                                process_list_row(
                                                    app,
                                                    ui,
                                                    &rows[index],
                                                    row_height,
                                                    paint,
                                                );
                                            }
                                        });
                                }
                            });
                        },
                    );
                    ui.add_space(7.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 8.0;
                        if bevel_button_enabled(
                            ui,
                            "End Task",
                            visible_selected(app, &rows),
                            [actions_width, 27.0],
                            paint,
                        )
                        .on_hover_text("Send SIGTERM after PID-safe confirmation")
                        .clicked()
                        {
                            app.queue_action(SignalAction::Terminate);
                        }
                        if bevel_button(ui, "New Task...", false, [actions_width, 27.0], paint)
                            .clicked()
                        {
                            app.new_task.open = true;
                            app.new_task.error = None;
                        }
                        if bevel_button(ui, "Refresh", false, [actions_width, 27.0], paint)
                            .clicked()
                        {
                            let _ = app.worker.commands.send(Command::Refresh);
                        }
                        if bevel_button(ui, "Close", false, [actions_width, 27.0], paint).clicked()
                        {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        ui.add_space(3.0);
                        ui.menu_button("Tools >>", |ui| {
                            for (page, label) in TOOLS.into_iter().skip(1) {
                                if ui.button(label).clicked() {
                                    app.page = page;
                                    ui.close();
                                }
                            }
                            ui.separator();
                            if ui.button("About Taskotto").clicked() {
                                app.notification = Some(format!(
                                    "Taskotto {} · Windows 98-inspired Linux process manager",
                                    env!("CARGO_PKG_VERSION")
                                ));
                                ui.close();
                            }
                        });
                    });
                });
                ui.add_space(9.0);
                let count = rows.len();
                ui.label(
                    RichText::new(format!(
                        "{count} running user processes  ·  Ctrl+F: advanced search  ·  F5: refresh"
                    ))
                    .small()
                    .color(Color32::from_rgb(55, 55, 55)),
                );
            },
        );
    });
}

fn process_list_row(
    app: &mut TaskottoApp,
    ui: &mut egui::Ui,
    row: &ProcessRow,
    height: f32,
    paint: ComponentPaint,
) {
    let p = &row.process;
    let selected = app.selected == Some((p.pid, p.start_ticks));
    let width = ui.available_width();
    let response = process_name_cell(
        ui,
        &p.name,
        width,
        height - 1.0,
        selected,
        12.5 * app.settings.font_scale,
        paint,
    );
    if response.clicked() {
        app.selected = Some((p.pid, p.start_ticks));
    }
    if response.double_clicked() {
        app.selected = Some((p.pid, p.start_ticks));
        app.page = Page::Details;
    }
    response.on_hover_text(format!(
        "PID {}  ·  {}  ·  double-click for details",
        p.pid,
        state_name(p.state)
    ));
}

/// Classic Win9x report cell: painted locally so a RicePack hover token can
/// affect an actual pointer-hover state without changing native window chrome.
fn process_name_cell(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    height: f32,
    selected: bool,
    font_size: f32,
    paint: ComponentPaint,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    let fill = if selected {
        paint.selection(ui.visuals().selection.bg_fill)
    } else if response.hovered() {
        paint.hover(WHITE)
    } else {
        paint.table(WHITE)
    };
    let text = if (selected && paint.selection.is_some())
        || (!selected && (paint.table.is_some() || paint.hover.is_some()))
    {
        readable_text(fill)
    } else if selected {
        ui.visuals().selection.stroke.color
    } else {
        BLACK
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    ui.painter().with_clip_rect(rect.shrink(2.0)).text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(font_size),
        text,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.5),
            0.0,
            Stroke::new(1.0_f32, paint.selection(NAVY)),
            StrokeKind::Inside,
        );
    }
    response
}

/// Period-appropriate compact process report, deliberately separate from the
/// Windows 98 Close Program dialog and backed by the shared Linux monitor.
pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let paint = ComponentPaint::from_settings(&app.settings);
    ui.label(RichText::new("Processes (Linux)").size(15.0).strong());
    ui.add_space(5.0);
    ui.horizontal_wrapped(|ui| {
        let search = ui.add_sized(
            [ui.available_width().min(225.0), 24.0],
            egui::TextEdit::singleline(&mut app.query).hint_text("Find name or PID (Ctrl+F)"),
        );
        if app.focus_search {
            search.request_focus();
            app.focus_search = false;
        }
        if ui.small_button("Clear").clicked() {
            app.query.clear();
        }
        egui::ComboBox::from_id_salt("classic98-sort")
            .selected_text(app.settings.sort_by.label())
            .show_ui(ui, |ui| {
                for sort in SortBy::ALL {
                    if ui
                        .selectable_value(&mut app.settings.sort_by, sort, sort.label())
                        .changed()
                    {
                        app.save_preferences(ui.ctx());
                    }
                }
            });
        if ui
            .button(if app.settings.descending {
                "↓"
            } else {
                "↑"
            })
            .clicked()
        {
            app.settings.descending = !app.settings.descending;
            app.save_preferences(ui.ctx());
        }
    });
    ui.add_space(7.0);
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading /proc...");
        return;
    }
    let rows = app.processes();
    let row_height = 23.0 * app.settings.font_scale.max(1.0);
    let height = ui.available_height().max(100.0) - 43.0;
    inset_list(ui, height, paint, |ui| {
        let w = ui.available_width();
        let name_width = (w - 225.0 - 3.0 * ui.spacing().item_spacing.x).max(150.0);
        ui.horizontal(|ui| {
            for (label, width, sort) in [
                ("Image Name", name_width, SortBy::Name),
                ("PID", 54.0, SortBy::Pid),
                ("CPU", 55.0, SortBy::Cpu),
                ("Memory (RSS)", 116.0, SortBy::Memory),
            ] {
                if ui
                    .add_sized([width, 25.0], classic_button(label, width, false, paint))
                    .clicked()
                {
                    if app.settings.sort_by == sort {
                        app.settings.descending = !app.settings.descending;
                    } else {
                        app.settings.sort_by = sort;
                        app.settings.descending = sort != SortBy::Name;
                    }
                    app.save_preferences(ui.ctx());
                }
            }
        });
        bevel_line(ui, paint);
        if rows.is_empty() {
            ui.colored_label(
                paint.on_table(BLACK),
                "No matching processes. Try clearing your search.",
            );
        } else {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show_rows(ui, row_height, rows.len(), |ui, range| {
                    for index in range {
                        let p = &rows[index].process;
                        let selected = app.selected == Some((p.pid, p.start_ticks));
                        ui.horizontal(|ui| {
                            let response = process_name_cell(
                                ui,
                                &p.name,
                                name_width,
                                row_height - 1.0,
                                selected,
                                12.0 * app.settings.font_scale,
                                paint,
                            );
                            if response.clicked() {
                                app.selected = Some((p.pid, p.start_ticks));
                            }
                            if response.double_clicked() {
                                app.selected = Some((p.pid, p.start_ticks));
                                app.page = Page::Details;
                            }
                            for (width, value) in [
                                (54.0, p.pid.to_string()),
                                (55.0, format!("{:.1}%", p.cpu_percent)),
                                (116.0, format::bytes(p.resident_bytes)),
                            ] {
                                ui.add_sized(
                                    [width, 21.0],
                                    egui::Label::new(
                                        RichText::new(value).color(paint.on_table(BLACK)),
                                    ),
                                );
                            }
                        });
                    }
                });
        }
    });
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        if bevel_button_enabled(
            ui,
            "End Process",
            app.selected_process().is_some_and(can_signal_process),
            [115.0, 27.0],
            paint,
        )
        .clicked()
        {
            app.queue_action(SignalAction::Terminate);
        }
        if bevel_button_enabled(
            ui,
            "Details...",
            app.selected_process().is_some(),
            [98.0, 27.0],
            paint,
        )
        .clicked()
        {
            app.page = Page::Details;
        }
    });
}

fn classic_button(
    label: &str,
    width: f32,
    selected: bool,
    paint: ComponentPaint,
) -> egui::Button<'_> {
    egui::Button::new(RichText::new(label).color(if selected {
        paint.on_selection(FACE, BLACK)
    } else {
        paint.on_control(FACE, BLACK)
    }))
    .fill(if selected {
        paint.selection(FACE)
    } else {
        paint.control(FACE)
    })
    .stroke(Stroke::new(1.0_f32, paint.border(SHADOW)))
    .corner_radius(0)
    .min_size(egui::vec2(width, 27.0))
}

fn bevel_button(
    ui: &mut egui::Ui,
    label: &str,
    selected: bool,
    size: [f32; 2],
    paint: ComponentPaint,
) -> egui::Response {
    let response = ui.add_sized(
        size,
        classic_button(label, size[0], selected, paint).selected(selected),
    );
    paint_button_bevel(ui, &response, selected, paint);
    response
}

fn bevel_button_enabled(
    ui: &mut egui::Ui,
    label: &str,
    enabled: bool,
    size: [f32; 2],
    paint: ComponentPaint,
) -> egui::Response {
    let response = ui.add_enabled(enabled, classic_button(label, size[0], false, paint));
    paint_button_bevel(ui, &response, false, paint);
    response
}

fn paint_button_bevel(
    ui: &mut egui::Ui,
    response: &egui::Response,
    selected: bool,
    paint: ComponentPaint,
) {
    let rect = response.rect;
    let pressed = response.is_pointer_button_down_on() || selected;
    let upper = if pressed { paint.border(SHADOW) } else { WHITE };
    let lower = if pressed { WHITE } else { paint.border(BLACK) };
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0_f32, upper),
    );
    ui.painter().line_segment(
        [rect.left_top(), rect.left_bottom()],
        Stroke::new(1.0_f32, upper),
    );
    ui.painter().line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0_f32, lower),
    );
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, lower),
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(3.0),
            0.0,
            Stroke::new(1.0_f32, paint.selection(NAVY)),
            StrokeKind::Inside,
        );
    }
}

fn bevel_line(ui: &mut egui::Ui, paint: ComponentPaint) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 2.0), Sense::hover());
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0_f32, paint.border(SHADOW)),
    );
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, WHITE),
    );
}

fn inset_list(
    ui: &mut egui::Ui,
    height: f32,
    paint: ComponentPaint,
    content: impl FnOnce(&mut egui::Ui),
) {
    let frame = Frame::NONE
        .fill(paint.table(WHITE))
        .stroke(Stroke::new(1.0_f32, paint.border(SHADOW)))
        .inner_margin(Margin::same(3));
    let shown = frame.show(ui, |ui| {
        ui.set_min_height((height - 8.0).max(72.0));
        content(ui);
    });
    let rect = shown.response.rect;
    // The sunken top/left edge and raised bottom/right edge are recognizable
    // Win9x client-control details, not a replacement window border.
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        Stroke::new(1.0_f32, paint.border(SHADOW)),
    );
    ui.painter().line_segment(
        [rect.left_top(), rect.left_bottom()],
        Stroke::new(1.0_f32, paint.border(SHADOW)),
    );
    ui.painter().line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(1.0_f32, WHITE),
    );
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, WHITE),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classic98_only_lists_current_user_non_kernel_tasks() {
        let sample = vec![
            Process {
                pid: 31,
                user_id: 1000,
                parent_pid: 1,
                ..Default::default()
            },
            Process {
                pid: 32,
                user_id: 0,
                parent_pid: 1,
                ..Default::default()
            },
            Process {
                pid: 33,
                user_id: 1000,
                parent_pid: 2,
                ..Default::default()
            },
        ];
        let visible = sample
            .into_iter()
            .filter(|p| close_program_visible(p, 1000))
            .collect::<Vec<_>>();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].pid, 31);
    }

    #[test]
    fn classic98_has_accessible_advanced_pages_and_a_return_path() {
        assert_eq!(TOOLS[0].0, Page::Applications);
        assert!(TOOLS.iter().any(|(page, _)| *page == Page::Processes));
        assert!(TOOLS.iter().any(|(page, _)| *page == Page::Performance));
        assert!(TOOLS.iter().any(|(page, _)| *page == Page::Preferences));
    }

    #[test]
    fn classic98_component_colors_keep_default_bevel_and_allow_a_dark_list() {
        let original = ComponentPaint::from_settings(&Settings::default());
        assert_eq!(original.control(FACE), FACE);
        assert_eq!(original.border(SHADOW), SHADOW);
        assert_eq!(original.table(WHITE), WHITE);
        assert_eq!(original.selection(NAVY), NAVY);
        assert_eq!(original.on_table(BLACK), BLACK);

        let mut settings = Settings::default();
        crate::ricepack::RicePack::from_toml(include_str!(
            "../../../examples/ricepacks/win98-violet.toml"
        ))
        .expect("Violet example must be valid")
        .apply(&mut settings);
        assert_eq!(settings.theme, Theme::Windows98);
        let paint = ComponentPaint::from_settings(&settings);
        assert_ne!(paint.control(FACE), FACE);
        assert_ne!(paint.border(SHADOW), SHADOW);
        assert_ne!(paint.hover(WHITE), WHITE);
        assert_ne!(paint.selection(NAVY), NAVY);
        assert_eq!(paint.on_table(BLACK), WHITE);
        assert_eq!(paint.on_selection(NAVY, BLACK), WHITE);
    }
}
