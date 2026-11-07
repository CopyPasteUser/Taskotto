//! Safe, data-bound client screens assembled from validated RicePack widgets.
//! No scripts, external assets, custom commands or hidden privileges.
use super::*;
use crate::{
    layout::{Gauge, Metric, Plot, ProcessColumn, ToolbarAction, Widget},
    theme::ComponentPaint,
};
use egui::{Align2, FontId, Frame, Margin, StrokeKind};

pub(in crate::ui) fn render(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    // The schema limits this clone to four short screens and 64 small widgets.
    // It keeps renderer borrows independent from PID-safe selection/actions.
    let Some(layout) = app.settings.custom_layout.clone() else {
        return;
    };
    let paint = ComponentPaint::from_settings(&app.settings);
    let id = egui::Id::new("taskotto-declarative-screen");
    let mut current = ui
        .ctx()
        .data(|data| data.get_temp::<usize>(id).unwrap_or(0));
    current = current.min(layout.pages.len() - 1);
    let base = ui.visuals().panel_fill;

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        for (index, screen) in layout.pages.iter().enumerate() {
            let selected = current == index;
            let fill = if selected {
                paint.selection(base)
            } else {
                paint.control(base)
            };
            let label_ink = if selected {
                paint.on_selection(base, ui.visuals().text_color())
            } else {
                paint.on_control(base, ui.visuals().text_color())
            };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(&screen.title).color(label_ink))
                        .fill(fill)
                        .selected(selected),
                )
                .clicked()
            {
                current = index;
                ui.ctx().data_mut(|data| data.insert_temp(id, index));
            }
        }
    });
    ui.add_space(8.0);
    ui.heading(&layout.pages[current].title);
    ui.add_space(7.0);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // The optional compact scene is a validated, explicitly-authored
            // fallback. The original scene remains the default at wide sizes.
            let widgets = layout.pages[current].widgets_for_width(
                ui.available_width(),
                layout.compact_below.unwrap_or(680) as f32,
            );
            let has_search_action = widgets.iter().any(|widget| match widget {
                Widget::Toolbar { actions } => actions.contains(&ToolbarAction::FocusSearch),
                _ => false,
            });
            if !has_search_action {
                fallback_search(app, ui);
            }
            let end_in_toolbar = widgets.iter().any(|widget| match widget {
                Widget::Toolbar { actions } => actions.contains(&ToolbarAction::EndProcess),
                _ => false,
            });
            let mut index = 0;
            while index < widgets.len() {
                if matches!(widgets[index], Widget::Metric { .. }) {
                    let end = (index..widgets.len())
                        .take_while(|&at| matches!(widgets[at], Widget::Metric { .. }))
                        .take(4)
                        .last()
                        .map_or(index + 1, |last| last + 1);
                    let tile_width =
                        (ui.available_width() / (end - index) as f32 - 14.0).clamp(135.0, 245.0);
                    ui.horizontal_wrapped(|ui| {
                        for (at, widget) in widgets.iter().enumerate().take(end).skip(index) {
                            if let Widget::Metric { label, binding } = widget {
                                ui.push_id(at, |ui| {
                                    metric_tile(app, ui, label, *binding, paint, tile_width);
                                });
                            }
                        }
                    });
                    index = end;
                } else {
                    ui.push_id(index, |ui| match &widgets[index] {
                        Widget::Heading { text } => {
                            ui.label(egui::RichText::new(text).size(15.0).strong());
                        }
                        Widget::Plot { label, binding } => {
                            ui.label(egui::RichText::new(label).strong());
                            plot(app, ui, *binding, paint);
                        }
                        Widget::Gauge { label, binding } => {
                            gauge(app, ui, label, *binding, paint);
                        }
                        Widget::StatusStrip { bindings } => {
                            status_strip(app, ui, bindings, paint);
                        }
                        Widget::Toolbar { actions } => {
                            toolbar(app, ui, actions, paint);
                        }
                        Widget::ProcessList { columns } => {
                            process_list(app, ui, columns, paint, !end_in_toolbar);
                        }
                        Widget::Metric { .. } => {}
                    });
                    index += 1;
                }
                ui.add_space(10.0);
            }
        });
}

fn metric_tile(
    app: &TaskottoApp,
    ui: &mut egui::Ui,
    label: &str,
    binding: Metric,
    paint: ComponentPaint,
    width: f32,
) {
    let base = ui.visuals().panel_fill;
    let ink = paint.on_table(ui.visuals().text_color());
    Frame::new()
        .fill(paint.table(base))
        .stroke(egui::Stroke::new(
            1.0_f32,
            paint.border(ui.visuals().widgets.noninteractive.bg_stroke.color),
        ))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(width - 28.0);
            if paint.table.is_some() {
                ui.visuals_mut().override_text_color = Some(ink);
            }
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(label).small());
                ui.label(
                    egui::RichText::new(metric_value(app.snapshot.as_ref(), binding))
                        .size(22.0)
                        .strong(),
                );
            });
        });
}

fn metric_value(snapshot: Option<&Snapshot>, binding: Metric) -> String {
    let Some(snapshot) = snapshot else {
        return "Reading Linux data…".into();
    };
    match binding {
        Metric::CpuPercent => format!("{:.1}%", snapshot.cpu_percent),
        Metric::MemoryUsed => format::bytes(snapshot.memory.used()),
        Metric::MemoryTotal => format::bytes(snapshot.memory.total),
        Metric::MemoryPercent => format!("{:.1}%", snapshot.memory.used_fraction() * 100.0),
        Metric::Processes => snapshot.processes.len().to_string(),
        Metric::Uptime => format::duration(snapshot.uptime_seconds),
        Metric::NetworkReceived => format::rate(snapshot.rates.received_per_second),
        Metric::NetworkSent => format::rate(snapshot.rates.sent_per_second),
        Metric::DiskRead => format::rate(snapshot.rates.disk_read_per_second),
        Metric::DiskWrite => format::rate(snapshot.rates.disk_write_per_second),
    }
}

fn gauge(app: &TaskottoApp, ui: &mut egui::Ui, label: &str, binding: Gauge, paint: ComponentPaint) {
    let actual = gauge_percent(app.snapshot.as_ref(), binding);
    ui.label(egui::RichText::new(label).strong());
    if let Some(percent) = actual {
        let progress = (percent / 100.0).clamp(0.0, 1.0);
        let fill = paint.selection(ThemeStyle::from_settings(&app.settings).accent);
        ui.add(
            egui::ProgressBar::new(progress)
                .fill(fill)
                .desired_width(ui.available_width().clamp(145.0, 540.0))
                .text(format!("{percent:.1}%")),
        );
    } else {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Waiting for measured CPU/memory data…");
        });
    }
}

fn gauge_percent(snapshot: Option<&Snapshot>, binding: Gauge) -> Option<f32> {
    snapshot.map(|snapshot| match binding {
        Gauge::CpuPercent => snapshot.cpu_percent,
        Gauge::MemoryPercent => snapshot.memory.used_fraction() * 100.0,
    })
}

fn status_strip(app: &TaskottoApp, ui: &mut egui::Ui, bindings: &[Metric], paint: ComponentPaint) {
    Frame::new()
        .fill(paint.table(ui.visuals().faint_bg_color))
        .stroke(egui::Stroke::new(
            1.0_f32,
            paint.border(ui.visuals().widgets.noninteractive.bg_stroke.color),
        ))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            if paint.table.is_some() {
                ui.visuals_mut().override_text_color =
                    Some(paint.on_table(ui.visuals().text_color()));
            }
            ui.horizontal_wrapped(|ui| {
                for (index, binding) in bindings.iter().enumerate() {
                    if index > 0 {
                        ui.separator();
                    }
                    ui.label(format!(
                        "{}: {}",
                        binding.label(),
                        metric_value(app.snapshot.as_ref(), *binding)
                    ));
                }
            });
        });
}

fn toolbar(
    app: &mut TaskottoApp,
    ui: &mut egui::Ui,
    actions: &[ToolbarAction],
    paint: ComponentPaint,
) {
    let id = egui::Id::new("taskotto-declarative-toolbar-search");
    let mut search_open = ui
        .ctx()
        .data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    let mut focus_now = app.focus_search && actions.contains(&ToolbarAction::FocusSearch);
    if focus_now {
        app.focus_search = false;
        search_open = true;
        ui.ctx().data_mut(|data| data.insert_temp(id, true));
    }
    Frame::new()
        .fill(paint.control(ui.visuals().faint_bg_color))
        .inner_margin(Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for action in actions {
                    match action {
                        ToolbarAction::Refresh => {
                            if ui.button("Refresh now").clicked()
                                && app.worker.commands.send(Command::Refresh).is_err()
                            {
                                app.notification =
                                    Some("Could not request a monitor refresh".into());
                            }
                        }
                        ToolbarAction::FocusSearch => {
                            if ui.button("Search processes").clicked() {
                                search_open = !search_open;
                                focus_now = search_open;
                                if !search_open {
                                    app.query.clear();
                                }
                                ui.ctx().data_mut(|data| data.insert_temp(id, search_open));
                            }
                        }
                        ToolbarAction::EndProcess => {
                            if ui
                                .add_enabled(
                                    app.selected_process().is_some_and(can_signal_process),
                                    egui::Button::new("End selected process…"),
                                )
                                .on_hover_text("Requires PID-safe confirmation")
                                .clicked()
                            {
                                app.queue_action(SignalAction::Terminate);
                            }
                        }
                    }
                }
            });
            if search_open && actions.contains(&ToolbarAction::FocusSearch) {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.query)
                        .hint_text("Filter name, command, PID or user…")
                        .desired_width(ui.available_width().clamp(160.0, 360.0)),
                );
                if focus_now {
                    response.request_focus();
                }
            }
        });
}

fn fallback_search(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    // Ctrl+F must work even in an AI-authored screen with no toolbar widget.
    let id = egui::Id::new("taskotto-declarative-fallback-search");
    let mut open = ui
        .ctx()
        .data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    let focus_now = app.focus_search;
    if focus_now {
        app.focus_search = false;
        open = true;
        ui.ctx().data_mut(|data| data.insert_temp(id, true));
    }
    if open {
        ui.horizontal(|ui| {
            let response = ui.add(
                egui::TextEdit::singleline(&mut app.query)
                    .hint_text("Filter Linux processes…")
                    .desired_width(ui.available_width().clamp(145.0, 340.0)),
            );
            if focus_now {
                response.request_focus();
            }
            if ui.small_button("Close search").clicked() {
                app.query.clear();
                ui.ctx().data_mut(|data| data.insert_temp(id, false));
            }
        });
        ui.add_space(8.0);
    }
}

fn plot(app: &TaskottoApp, ui: &mut egui::Ui, binding: Plot, paint: ComponentPaint) {
    let cpu = Color32::from_rgb(45, 169, 115);
    let memory = Color32::from_rgb(64, 138, 222);
    let inbound = Color32::from_rgb(44, 139, 221);
    let outbound = Color32::from_rgb(229, 147, 64);
    let (primary, secondary, fixed_max, unit): (_, Option<_>, _, _) = match binding {
        Plot::Cpu => ((&app.history.cpu, cpu, "CPU"), None, 100.0, "%"),
        Plot::Memory => ((&app.history.memory, memory, "Memory"), None, 100.0, "%"),
        Plot::NetworkReceived => (
            (&app.history.received, inbound, "Receive"),
            None,
            0.0,
            "B/s",
        ),
        Plot::NetworkSent => ((&app.history.sent, outbound, "Send"), None, 0.0, "B/s"),
        Plot::DiskRead => ((&app.history.disk_read, inbound, "Read"), None, 0.0, "B/s"),
        Plot::DiskWrite => (
            (&app.history.disk_write, outbound, "Write"),
            None,
            0.0,
            "B/s",
        ),
        Plot::CpuMemory => (
            (&app.history.cpu, cpu, "CPU"),
            Some((&app.history.memory, memory, "RAM")),
            100.0,
            "%",
        ),
        Plot::NetworkTraffic => (
            (&app.history.received, inbound, "Receive"),
            Some((&app.history.sent, outbound, "Send")),
            0.0,
            "B/s",
        ),
        Plot::DiskTraffic => (
            (&app.history.disk_read, inbound, "Read"),
            Some((&app.history.disk_write, outbound, "Write")),
            0.0,
            "B/s",
        ),
    };
    let height = 142.0 * app.settings.chart_scale;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(100.0), height),
        Sense::hover(),
    );
    ui.painter()
        .rect_filled(rect, 5.0, paint.table(ui.visuals().faint_bg_color));
    ui.painter().rect_stroke(
        rect,
        5.0,
        egui::Stroke::new(
            1.0_f32,
            paint.border(ui.visuals().widgets.noninteractive.bg_stroke.color),
        ),
        StrokeKind::Inside,
    );
    let maximum = plot_maximum(primary.0, secondary.map(|line| line.0), fixed_max);
    if primary.0.len() < 2 && secondary.is_none_or(|line| line.0.len() < 2) {
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            "Waiting for samples…",
            FontId::proportional(12.0),
            paint.on_table(ui.visuals().text_color()),
        );
    }
    for (series, color, _) in [Some(primary), secondary].into_iter().flatten() {
        if series.len() >= 2 {
            ui.painter().add(egui::Shape::line(
                super::super::chart_points(series, rect.shrink(7.0), maximum),
                egui::Stroke::new(2.0_f32, color),
            ));
        }
    }
    if let Some(second) = secondary {
        ui.horizontal_wrapped(|ui| {
            for (_, color, label) in [primary, second] {
                ui.colored_label(color, label);
            }
            ui.label(if unit == "%" {
                "Common scale: 0–100%".to_owned()
            } else {
                format!("Common scale: 0–{}", format::rate(maximum as u64))
            });
        });
    }
}

fn plot_maximum(first: &VecDeque<f32>, second: Option<&VecDeque<f32>>, fixed_max: f32) -> f32 {
    if fixed_max > 0.0 {
        return fixed_max;
    }
    first
        .iter()
        .chain(second.into_iter().flatten())
        .copied()
        .filter(|point| point.is_finite() && *point >= 0.0)
        .fold(1.0_f32, f32::max)
        * 1.2
}

#[cfg(test)]
mod comparisons_tests {
    use super::*;

    #[test]
    fn compare_plots_share_a_scale_and_ignore_invalid_samples() {
        let received = VecDeque::from([8.0, 16.0, 4.0]);
        let sent = VecDeque::from([0.0, 40.0]);
        assert_eq!(plot_maximum(&received, Some(&sent), 0.0), 48.0);
        assert_eq!(plot_maximum(&received, Some(&sent), 100.0), 100.0);
        assert_eq!(plot_maximum(&VecDeque::from([f32::NAN]), None, 0.0), 1.2);
    }
}

fn process_list(
    app: &mut TaskottoApp,
    ui: &mut egui::Ui,
    columns: &[ProcessColumn],
    paint: ComponentPaint,
    show_end_action: bool,
) {
    let rows = app.processes();
    let width = ui.available_width();
    let fixed = columns
        .iter()
        .filter(|col| **col != ProcessColumn::Name)
        .count() as f32
        * 90.0;
    let name_width = (width - fixed - 22.0).max(110.0);
    let row_height = app
        .settings
        .table_row_height
        .unwrap_or(28.0)
        .clamp(22.0, 48.0);
    ui.horizontal(|ui| {
        for column in columns {
            ui.add_sized(
                [
                    if *column == ProcessColumn::Name {
                        name_width
                    } else {
                        90.0
                    },
                    20.0,
                ],
                egui::Label::new(egui::RichText::new(column.label()).strong().small()),
            );
        }
    });
    Frame::new()
        .fill(paint.table(ui.visuals().faint_bg_color))
        .stroke(egui::Stroke::new(
            1.0_f32,
            paint.border(ui.visuals().widgets.noninteractive.bg_stroke.color),
        ))
        .show(ui, |ui| {
            if paint.table.is_some() {
                ui.visuals_mut().override_text_color =
                    Some(paint.on_table(ui.visuals().text_color()));
            }
            egui::ScrollArea::vertical().max_height(290.0).show_rows(
                ui,
                row_height,
                rows.len(),
                |ui, range| {
                    for index in range {
                        let process = &rows[index].process;
                        let selected = app.selected == Some((process.pid, process.start_ticks));
                        ui.horizontal(|ui| {
                            for column in columns {
                                let value = match column {
                                    ProcessColumn::Name => process.name.clone(),
                                    ProcessColumn::Pid => process.pid.to_string(),
                                    ProcessColumn::Cpu => format!("{:.1}%", process.cpu_percent),
                                    ProcessColumn::Memory => format::bytes(process.resident_bytes),
                                    ProcessColumn::Threads => process.threads.to_string(),
                                };
                                let size = [
                                    if *column == ProcessColumn::Name {
                                        name_width
                                    } else {
                                        90.0
                                    },
                                    row_height - 1.0,
                                ];
                                if *column == ProcessColumn::Name {
                                    if ui
                                        .add_sized(
                                            size,
                                            egui::Button::new(value).selected(selected),
                                        )
                                        .on_hover_text(&process.command)
                                        .clicked()
                                    {
                                        app.selected = Some((process.pid, process.start_ticks));
                                    }
                                } else {
                                    ui.add_sized(size, egui::Label::new(value));
                                }
                            }
                        });
                    }
                },
            );
        });
    ui.horizontal(|ui| {
        let selected = app.selected_process().is_some_and(can_signal_process);
        if show_end_action
            && ui
                .add_enabled(selected, egui::Button::new("End selected process…"))
                .on_hover_text("Uses the usual PID-safe Linux confirmation dialog")
                .clicked()
        {
            app.queue_action(SignalAction::Terminate);
        }
        ui.label(format!("{} matching processes", rows.len()));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_do_not_fabricate_unavailable_samples() {
        assert_eq!(
            metric_value(None, Metric::CpuPercent),
            "Reading Linux data…"
        );
        assert_eq!(gauge_percent(None, Gauge::CpuPercent), None);
        let sample = Snapshot {
            cpu_percent: 25.0,
            cores_percent: vec![],
            memory: crate::monitor::Memory {
                total: 4_000,
                available: 1_000,
                swap_total: 0,
                swap_free: 0,
            },
            rates: Default::default(),
            uptime_seconds: 60.0,
            processes: Vec::new(),
            logical_cpus: 1,
            gpus: vec![],
            power: Default::default(),
            pressure: Default::default(),
        };
        assert_eq!(metric_value(Some(&sample), Metric::CpuPercent), "25.0%");
        assert_eq!(metric_value(Some(&sample), Metric::Processes), "0");
        assert_eq!(
            gauge_percent(Some(&sample), Gauge::MemoryPercent),
            Some(75.0)
        );
    }
}
