use super::*;
use egui::{Align2, FontId, Rect, StrokeKind};

pub(super) fn page(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    if app.snapshot.is_none() {
        ui.spinner();
        ui.label("Reading system performance…");
        return;
    }
    if is_xp(app) {
        xp_performance(app, ui);
    } else {
        modern_performance(app, ui);
    }
}

fn modern_performance(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let wide = ui.available_width() > 800.0;
    ui.add_space(7.0);
    ui.heading(RichText::new("Performance").size(30.0));
    ui.add_space(10.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        if wide {
            let room = ui.available_width();
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(224.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        for focus in ALL_FOCUS {
                            metric_tile(app, ui, focus, 212.0, 72.0);
                        }
                    },
                );
                ui.separator();
                ui.allocate_ui_with_layout(
                    egui::vec2((room - 254.0).max(250.0), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| focused_chart(app, ui),
                );
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                for focus in ALL_FOCUS {
                    metric_tile(app, ui, focus, 139.0, 57.0);
                }
            });
            ui.add_space(12.0);
            focused_chart(app, ui);
        }
    });
}

const ALL_FOCUS: [PerformanceFocus; 4] = [
    PerformanceFocus::Cpu,
    PerformanceFocus::Memory,
    PerformanceFocus::Disk,
    PerformanceFocus::Network,
];

fn metric_data(
    app: &TaskottoApp,
    focus: PerformanceFocus,
) -> (&'static str, String, Color32, &VecDeque<f32>, f32) {
    let snapshot = app.snapshot.as_ref().expect("checked by page");
    match focus {
        PerformanceFocus::Cpu => (
            "CPU",
            format!(
                "{:.0}%  ·  {} cores",
                snapshot.cpu_percent, snapshot.logical_cpus
            ),
            Color32::from_rgb(45, 145, 193),
            &app.history.cpu,
            100.0,
        ),
        PerformanceFocus::Memory => (
            "Memory",
            format!(
                "{} / {}",
                format::bytes(snapshot.memory.used()),
                format::bytes(snapshot.memory.total)
            ),
            Color32::from_rgb(136, 84, 174),
            &app.history.memory,
            100.0,
        ),
        PerformanceFocus::Disk => (
            "Disk",
            format!("Read {}", format::rate(snapshot.rates.disk_read_per_second)),
            Color32::from_rgb(60, 152, 91),
            &app.history.disk_read,
            graph_max(&app.history.disk_read, &app.history.disk_write),
        ),
        PerformanceFocus::Network => (
            "Network",
            format!("Rx {}", format::rate(snapshot.rates.received_per_second)),
            Color32::from_rgb(182, 117, 39),
            &app.history.received,
            graph_max(&app.history.received, &app.history.sent),
        ),
    }
}

fn metric_tile(
    app: &mut TaskottoApp,
    ui: &mut egui::Ui,
    focus: PerformanceFocus,
    width: f32,
    height: f32,
) {
    let (title, detail, color, values, scale) = metric_data(app, focus);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
    let selected = app.performance_focus == focus;
    ui.painter().rect_filled(
        rect,
        0.0,
        if selected {
            Color32::from_rgb(234, 244, 251)
        } else {
            Color32::WHITE
        },
    );
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(
            if selected { 2.0_f32 } else { 1.0_f32 },
            if selected {
                color
            } else {
                Color32::from_rgb(219, 219, 219)
            },
        ),
        StrokeKind::Inside,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(3.0),
            0.0,
            Stroke::new(2.0_f32, color),
            StrokeKind::Inside,
        );
    }
    let spark_width = if width > 160.0 { 78.0 } else { 43.0 };
    let graph_rect = Rect::from_min_size(
        egui::pos2(rect.right() - spark_width - 7.0, rect.top() + 13.0),
        egui::vec2(spark_width, height - 24.0),
    );
    ui.painter().rect_stroke(
        graph_rect,
        0.0,
        Stroke::new(1.0_f32, color),
        StrokeKind::Inside,
    );
    plot_lines(ui, graph_rect.shrink(2.0), &[(values, color)], scale, false);
    ui.painter()
        .with_clip_rect(Rect::from_min_max(
            rect.left_top(),
            egui::pos2(graph_rect.left() - 3.0, rect.bottom()),
        ))
        .text(
            rect.min + egui::vec2(10.0, 20.0),
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(16.0),
            Color32::from_rgb(37, 37, 37),
        );
    ui.painter()
        .with_clip_rect(Rect::from_min_max(
            rect.left_top(),
            egui::pos2(graph_rect.left() - 3.0, rect.bottom()),
        ))
        .text(
            rect.min + egui::vec2(10.0, height - 20.0),
            Align2::LEFT_CENTER,
            detail,
            FontId::proportional(11.5),
            Color32::from_rgb(91, 91, 91),
        );
    if response.clicked() {
        app.performance_focus = focus;
    }
    response.on_hover_text("Show detailed live history");
}

fn focused_chart(app: &TaskottoApp, ui: &mut egui::Ui) {
    let snapshot = app.snapshot.as_ref().expect("checked by page");
    let focus = app.performance_focus;
    let (title, summary, color, first, scale) = metric_data(app, focus);
    ui.horizontal(|ui| {
        ui.heading(RichText::new(title).size(31.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(summary).size(13.0).weak());
        });
    });
    ui.add_space(7.0);
    let subtitle = match focus {
        PerformanceFocus::Cpu => "CPU utilization · percentage over time",
        PerformanceFocus::Memory => "Physical memory use · percentage over time",
        PerformanceFocus::Disk => "Block device throughput · bytes per second",
        PerformanceFocus::Network => "Received and transmitted throughput · bytes per second",
    };
    ui.label(RichText::new(subtitle).small().weak());
    let rect = graph_rect(
        ui,
        260.0,
        Color32::from_rgb(252, 254, 255),
        Color32::from_rgb(197, 222, 237),
    );
    let second = match focus {
        PerformanceFocus::Disk => Some(&app.history.disk_write),
        PerformanceFocus::Network => Some(&app.history.sent),
        _ => None,
    };
    if let Some(second) = second {
        plot_lines(
            ui,
            rect.shrink(2.0),
            &[(first, color), (second, Color32::from_rgb(76, 169, 150))],
            scale,
            false,
        );
    } else {
        plot_lines(ui, rect.shrink(2.0), &[(first, color)], scale, false);
    }
    ui.add_space(15.0);
    match focus {
        PerformanceFocus::Cpu => {
            ui.horizontal_wrapped(|ui| {
                metric(ui, "Utilization", format!("{:.1}%", snapshot.cpu_percent));
                metric(ui, "Logical processors", snapshot.logical_cpus.to_string());
                metric(ui, "Processes", snapshot.processes.len().to_string());
                metric(ui, "Up time", format::duration(snapshot.uptime_seconds));
            });
            ui.add_space(20.0);
            ui.label("Logical processor utilization");
            let available = ui.available_width();
            let columns = ((available / 145.0).floor() as usize).clamp(1, 6);
            let bar_width =
                ((available - (columns - 1) as f32 * 12.0) / columns as f32).clamp(82.0, 130.0);
            egui::Grid::new("win10-logical-cores")
                .num_columns(columns)
                .spacing([12.0, 10.0])
                .show(ui, |ui| {
                    for (index, usage) in snapshot.cores_percent.iter().enumerate() {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(format!("CPU {index}   {:.0}%", usage)).small());
                            ui.add_sized(
                                [bar_width, 13.0],
                                egui::ProgressBar::new((usage / 100.0).clamp(0.0, 1.0)),
                            );
                        });
                        if (index + 1).is_multiple_of(columns) {
                            ui.end_row();
                        }
                    }
                });
        }
        PerformanceFocus::Memory => {
            ui.horizontal_wrapped(|ui| {
                metric(ui, "In use", format::bytes(snapshot.memory.used()));
                metric(ui, "Available", format::bytes(snapshot.memory.available));
                metric(
                    ui,
                    "Installed / visible",
                    format::bytes(snapshot.memory.total),
                );
                metric(
                    ui,
                    "Swap",
                    format!(
                        "{} / {}",
                        format::bytes(
                            snapshot
                                .memory
                                .swap_total
                                .saturating_sub(snapshot.memory.swap_free)
                        ),
                        format::bytes(snapshot.memory.swap_total)
                    ),
                );
            });
        }
        PerformanceFocus::Disk => {
            ui.horizontal_wrapped(|ui| {
                metric(
                    ui,
                    "Read speed",
                    format::rate(snapshot.rates.disk_read_per_second),
                );
                metric(
                    ui,
                    "Write speed",
                    format::rate(snapshot.rates.disk_write_per_second),
                );
                metric(ui, "Scale", format::rate(scale as u64));
            });
            ui.label(RichText::new("Blue-green curves: read and write. Block-layer totals, not per-filesystem I/O.").small().weak());
        }
        PerformanceFocus::Network => {
            ui.horizontal_wrapped(|ui| {
                metric(
                    ui,
                    "Receive",
                    format::rate(snapshot.rates.received_per_second),
                );
                metric(ui, "Send", format::rate(snapshot.rates.sent_per_second));
                metric(ui, "Scale", format::rate(scale as u64));
            });
            ui.label(RichText::new("Blue-green curves: receive and send. Virtual interfaces can double-count traffic.").small().weak());
        }
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: String) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).size(12.0).weak());
        ui.label(RichText::new(value).size(20.0).strong());
    });
    ui.add_space(24.0);
}

fn xp_performance(app: &TaskottoApp, ui: &mut egui::Ui) {
    let snapshot = app.snapshot.as_ref().expect("checked by page");
    ui.label(RichText::new("Performance").size(14.0).strong());
    ui.add_space(7.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        xp_graph(
            ui,
            "CPU Usage",
            snapshot.cpu_percent,
            &app.history.cpu,
            100.0,
        );
        ui.add_space(12.0);
        xp_graph(
            ui,
            "Physical Memory Usage",
            snapshot.memory.used_fraction() * 100.0,
            &app.history.memory,
            100.0,
        );
        ui.add_space(14.0);
        egui::Frame::group(ui.style())
            .fill(Color32::from_rgb(245, 243, 230))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    metric(ui, "Processes", snapshot.processes.len().to_string());
                    metric(ui, "CPU", format!("{:.1}%", snapshot.cpu_percent));
                    metric(
                        ui,
                        "Physical Memory",
                        format!(
                            "{} / {}",
                            format::bytes(snapshot.memory.used()),
                            format::bytes(snapshot.memory.total)
                        ),
                    );
                    metric(
                        ui,
                        "Swap",
                        format!(
                            "{} / {}",
                            format::bytes(
                                snapshot
                                    .memory
                                    .swap_total
                                    .saturating_sub(snapshot.memory.swap_free)
                            ),
                            format::bytes(snapshot.memory.swap_total)
                        ),
                    );
                    metric(ui, "Up time", format::duration(snapshot.uptime_seconds));
                });
            });
    });
}

fn xp_graph(ui: &mut egui::Ui, title: &str, fraction: f32, history: &VecDeque<f32>, scale: f32) {
    ui.label(RichText::new(title).size(13.0).strong());
    ui.horizontal_top(|ui| {
        let (bar, _) = ui.allocate_exact_size(egui::vec2(96.0, 154.0), Sense::hover());
        let frame = Color32::from_rgb(106, 105, 98);
        ui.painter()
            .rect_filled(bar, 0.0, Color32::from_rgb(2, 12, 5));
        ui.painter()
            .rect_stroke(bar, 0.0, Stroke::new(2.0_f32, frame), StrokeKind::Inside);
        let filled = bar.height() * (fraction / 100.0).clamp(0.0, 1.0);
        if filled > 5.0 {
            ui.painter().rect_filled(
                Rect::from_min_max(
                    egui::pos2(bar.left() + 5.0, bar.bottom() - filled),
                    egui::pos2(bar.right() - 5.0, bar.bottom() - 5.0),
                ),
                0.0,
                Color32::from_rgb(57, 212, 64),
            );
        }
        ui.painter().text(
            bar.center(),
            Align2::CENTER_CENTER,
            format!("{fraction:.0}%"),
            FontId::proportional(18.0),
            Color32::from_rgb(160, 255, 160),
        );
        ui.vertical(|ui| {
            ui.label(
                RichText::new(format!("{title} History"))
                    .size(12.0)
                    .strong(),
            );
            let rect = graph_rect(
                ui,
                133.0,
                Color32::from_rgb(0, 15, 3),
                Color32::from_rgb(21, 92, 27),
            );
            plot_lines(
                ui,
                rect.shrink(2.0),
                &[(history, Color32::from_rgb(62, 236, 65))],
                scale,
                true,
            );
        });
    });
}

fn graph_rect(ui: &mut egui::Ui, height: f32, fill: Color32, grid: Color32) -> Rect {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(175.0), height),
        Sense::hover(),
    );
    ui.painter().rect_filled(rect, 0.0, fill);
    for index in 1..=12 {
        let x = rect.left() + rect.width() * index as f32 / 12.0;
        ui.painter().line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            Stroke::new(0.65_f32, grid),
        );
    }
    for index in 1..=6 {
        let y = rect.top() + rect.height() * index as f32 / 6.0;
        ui.painter().line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(0.65_f32, grid),
        );
    }
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0_f32, grid), StrokeKind::Inside);
    rect
}

fn plot_lines(
    ui: &mut egui::Ui,
    rect: Rect,
    series: &[(&VecDeque<f32>, Color32)],
    max: f32,
    _classic: bool,
) {
    for &(history, color) in series {
        if history.len() >= 2 {
            ui.painter().add(egui::Shape::line(
                chart_points(history, rect, max.max(1.0)),
                Stroke::new(1.7_f32, color),
            ));
        }
    }
}
