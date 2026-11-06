//! Original Frutiger Aero / Y2K-inspired task-manager client area.
//! No OS artwork, simulated titlebar, window controls, fake counters, or
//! background polling beyond Taskotto's existing Linux procfs worker.
use super::*;
use crate::theme::{parse_hex, readable_text};
use egui::{Align2, FontId, Frame, Margin, Pos2, Rect, StrokeKind};

const INK: Color32 = Color32::from_rgb(24, 74, 79);
const MUTED: Color32 = Color32::from_rgb(66, 115, 122);
const SKY: Color32 = Color32::from_rgb(42, 169, 220);
const LEAF: Color32 = Color32::from_rgb(61, 174, 104);
const AQUA: Color32 = Color32::from_rgb(26, 177, 168);
const BORDER: Color32 = Color32::from_rgb(157, 212, 211);

// The era-specific artwork stays original, but interactive client surfaces
// consume the same optional RicePack v1 semantic tokens as egui widgets.
// All None values preserve the existing built-in Frutiger Aero rendering.
#[derive(Clone, Copy, Debug, Default)]
struct AeroPalette {
    control: Option<Color32>,
    hover: Option<Color32>,
    border: Option<Color32>,
    table: Option<Color32>,
    selection: Option<Color32>,
}

impl AeroPalette {
    fn from_settings(settings: &Settings) -> Self {
        let components = &settings.components;
        Self {
            control: components.control.as_deref().and_then(parse_hex),
            hover: components.hover.as_deref().and_then(parse_hex),
            border: components.border.as_deref().and_then(parse_hex),
            table: components.table.as_deref().and_then(parse_hex),
            selection: components.selection.as_deref().and_then(parse_hex),
        }
    }

    fn control(self, fallback: Color32) -> Color32 {
        self.control.unwrap_or(fallback)
    }
    fn hover(self, fallback: Color32) -> Color32 {
        self.hover.unwrap_or(fallback)
    }
    fn border(self) -> Color32 {
        self.border.unwrap_or(BORDER)
    }
    fn table(self, fallback: Color32) -> Color32 {
        self.table.unwrap_or(fallback)
    }
    fn selection(self, fallback: Color32) -> Color32 {
        self.selection.unwrap_or(fallback)
    }
    fn on_table(self) -> Color32 {
        self.table.map(readable_text).unwrap_or(INK)
    }
    fn on_selection(self, fallback: Color32) -> Color32 {
        if self.selection.is_some() {
            readable_text(self.selection(fallback))
        } else {
            Color32::WHITE
        }
    }
    fn on_control(self, fallback: Color32) -> Color32 {
        if self.control.is_some() {
            readable_text(self.control(fallback))
        } else {
            INK
        }
    }
}

const PAGES: [(Page, &str); 7] = [
    (Page::Processes, "Processes"),
    (Page::Performance, "Performance"),
    (Page::Networking, "Network"),
    (Page::Users, "Users"),
    (Page::Services, "Services"),
    (Page::Details, "Details"),
    (Page::Preferences, "Settings"),
];

pub(super) fn topbar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let accent = accent(app);
    let palette = AeroPalette::from_settings(&app.settings);
    egui::TopBottomPanel::top("frutiger-aero-masthead")
        .frame(Frame::NONE)
        .show(ctx, |ui| {
            paint_sky(ui, ui.max_rect(), accent);
            ui.add_space(13.0);
            ui.horizontal(|ui| {
                ui.add_space(17.0);
                let (orb, _) = ui.allocate_exact_size(egui::vec2(49.0, 49.0), Sense::hover());
                paint_orb(ui, orb.center(), 22.0, AQUA);
                // These labels describe the application, not the native
                // window caption (owned by the desktop compositor).
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("taskotto")
                            .strong()
                            .size(27.0)
                            .color(Color32::from_rgb(13, 85, 98)),
                    );
                    ui.label(
                        RichText::new("your system, in its element")
                            .size(12.0)
                            .color(INK),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(17.0);
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("↻  Refresh")
                                    .strong()
                                    .color(palette.on_control(Color32::from_rgb(235, 255, 253))),
                            )
                            .fill(palette.control(Color32::from_rgb(235, 255, 253)))
                            .stroke(Stroke::new(1.0_f32, palette.border())),
                        )
                        .on_hover_text("Read fresh Linux /proc data (F5)")
                        .clicked()
                    {
                        let _ = app.worker.commands.send(Command::Refresh);
                    }
                    if ui.available_width() > 235.0 {
                        if let Some(snapshot) = &app.snapshot {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(format!("CPU  {:.0}%", snapshot.cpu_percent))
                                        .strong()
                                        .color(INK),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "RAM  {:.0}%",
                                        snapshot.memory.used_fraction() * 100.0
                                    ))
                                    .color(MUTED),
                                );
                            });
                        } else {
                            ui.spinner();
                        }
                    }
                });
            });
            ui.add_space(15.0);
        });
}

pub(super) fn navigation(app: &mut TaskottoApp, ctx: &egui::Context) {
    let accent = accent(app);
    let palette = AeroPalette::from_settings(&app.settings);
    egui::TopBottomPanel::top("frutiger-aero-tabs")
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(214, 242, 235))
                .inner_margin(Margin::symmetric(12, 7)),
        )
        .show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                let available = ui.available_width();
                let tab_width = if available >= 790.0 { 111.0 } else { 80.0 };
                for (page, label) in PAGES {
                    let active = app.page == page;
                    let response = ui.add_sized(
                        [tab_width, 32.0],
                        egui::Button::new(
                            RichText::new(label)
                                .size(if available >= 790.0 { 13.5 } else { 12.0 })
                                .strong()
                                .color(if active {
                                    palette.on_selection(accent)
                                } else {
                                    palette.on_control(Color32::from_rgb(244, 255, 251))
                                }),
                        )
                        .corner_radius(15.0)
                        .fill(if active {
                            palette.selection(accent)
                        } else {
                            palette.control(Color32::from_rgb(244, 255, 251))
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if active {
                                palette.selection(accent)
                            } else {
                                palette.border()
                            },
                        )),
                    );
                    if !active && response.hovered() {
                        ui.painter().rect_stroke(
                            response.rect.shrink(1.0),
                            14.0,
                            Stroke::new(1.0_f32, palette.hover(SKY)),
                            StrokeKind::Inside,
                        );
                    }
                    if response.clicked() {
                        app.page = page;
                    }
                }
            });
        });
}

pub(super) fn sidebar(app: &mut TaskottoApp, ctx: &egui::Context) {
    let accent = accent(app);
    let palette = AeroPalette::from_settings(&app.settings);
    egui::SidePanel::left("frutiger-aero-side-tabs")
        .resizable(false)
        .exact_width(172.0)
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(217, 242, 235))
                .inner_margin(11.0),
        )
        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("EXPLORE").strong().color(MUTED).size(11.0));
            ui.add_space(8.0);
            for (page, label) in PAGES {
                let active = app.page == page;
                if ui
                    .add_sized(
                        [148.0, 37.0],
                        egui::Button::new(
                            RichText::new(label)
                                .color(if active {
                                    palette.on_selection(accent)
                                } else {
                                    palette.on_control(Color32::from_rgb(245, 255, 250))
                                })
                                .strong(),
                        )
                        .corner_radius(14.0)
                        .fill(if active {
                            palette.selection(accent)
                        } else {
                            palette.control(Color32::from_rgb(245, 255, 250))
                        }),
                    )
                    .clicked()
                {
                    app.page = page;
                }
                ui.add_space(5.0);
            }
        });
}

pub(super) fn statusbar(app: &TaskottoApp, ctx: &egui::Context) {
    let palette = AeroPalette::from_settings(&app.settings);
    egui::TopBottomPanel::bottom("frutiger-aero-status")
        .frame(
            Frame::NONE
                .fill(Color32::from_rgb(229, 249, 244))
                .inner_margin(Margin::symmetric(14, 7)),
        )
        .show(ctx, |ui| {
            ui.painter().line_segment(
                [ui.max_rect().left_top(), ui.max_rect().right_top()],
                Stroke::new(1.0_f32, palette.border()),
            );
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Live").strong().color(LEAF));
                if let Some(s) = &app.snapshot {
                    ui.label(format!("{} running processes", s.processes.len()));
                    ui.separator();
                    ui.label(format!("CPU {:.0}%", s.cpu_percent));
                    ui.separator();
                    ui.label(format!("Memory {:.0}%", s.memory.used_fraction() * 100.0));
                } else {
                    ui.label("Gathering live system information…");
                }
                if ui.available_width() > 130.0 {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Linux /proc · live").small().color(MUTED));
                    });
                }
            });
        });
}

pub(super) fn processes(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let accent = accent(app);
    let palette = AeroPalette::from_settings(&app.settings);
    let cpu = app.snapshot.as_ref().map(|s| s.cpu_percent);
    let memory = app
        .snapshot
        .as_ref()
        .map(|s| s.memory.used_fraction() * 100.0);
    hero(
        ui,
        "PROCESS GARDEN",
        "Every little thing that's running.",
        cpu,
        memory,
        accent,
        palette,
    );
    ui.add_space(13.0);
    egui::Frame::new()
        .fill(palette.table(Color32::from_rgb(247, 255, 253)))
        .stroke(Stroke::new(1.0_f32, palette.border()))
        .corner_radius(17.0)
        .inner_margin(egui::Margin::symmetric(15, 11))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                let search = ui.add_sized(
                    [ui.available_width().clamp(160.0, 290.0), 29.0],
                    egui::TextEdit::singleline(&mut app.query)
                        .hint_text("Search process, user or PID…"),
                );
                if app.focus_search {
                    search.request_focus();
                    app.focus_search = false;
                }
                egui::ComboBox::from_id_salt("aero-sort")
                    .selected_text(format!("Sort: {}", app.settings.sort_by.label()))
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
                        "DESC"
                    } else {
                        "ASC"
                    })
                    .on_hover_text("Reverse sort order")
                    .clicked()
                {
                    app.settings.descending = !app.settings.descending;
                    app.save_preferences(ui.ctx());
                }
                if ui
                    .selectable_label(
                        app.settings.tree_view,
                        RichText::new("Tree").color(palette.on_table()),
                    )
                    .on_hover_text("Group child processes under parents")
                    .clicked()
                {
                    app.settings.tree_view = !app.settings.tree_view;
                    app.save_preferences(ui.ctx());
                }
            });
            ui.add_space(11.0);
            if app.snapshot.is_none() {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Reading processes from Linux /proc…");
                });
                return;
            }
            let rows = app.processes();
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!("{} processes in view", rows.len()))
                        .strong()
                        .color(palette.on_table()),
                );
                ui.add_space(10.0);
                if let Some(p) = app.selected_process() {
                    ui.label(
                        RichText::new(format!("Selected: {} · PID {}", p.name, p.pid))
                            .color(palette.on_table()),
                    );
                    if ui.button("Inspect ↗").clicked() {
                        app.page = Page::Details;
                    }
                    if ui
                        .add_enabled(
                            app.selected_process().is_some_and(can_signal_process),
                            egui::Button::new("End task…")
                                .fill(palette.control(Color32::from_rgb(245, 251, 247))),
                        )
                        .clicked()
                    {
                        app.queue_action(SignalAction::Terminate);
                    }
                } else {
                    ui.label(
                        RichText::new("Select a row to inspect or manage it")
                            .color(palette.on_table()),
                    );
                }
            });
            ui.add_space(7.0);
            if rows.is_empty() {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new("No matching processes")
                            .strong()
                            .color(palette.on_table()),
                    );
                    ui.label(
                        RichText::new("Try a different name, PID or filter.")
                            .color(palette.on_table()),
                    );
                    if ui.button("Clear search").clicked() {
                        app.query.clear();
                    }
                });
                return;
            }
            let width = ui.available_width();
            let show_user = app.settings.show_user_column && width >= 740.0;
            let show_io = app.settings.show_io_column && width >= 920.0;
            let pid_width = 54.0;
            let cpu_width = 74.0;
            let ram_width = 104.0;
            let user_width = if show_user { 103.0 } else { 0.0 };
            let io_width = if show_io { 105.0 } else { 0.0 };
            let name_width =
                (width - pid_width - cpu_width - ram_width - user_width - io_width - 30.0)
                    .max(130.0);
            ui.separator();
            ui.horizontal(|ui| {
                head(ui, "PROCESS", name_width, palette);
                head(ui, "PID", pid_width, palette);
                head(ui, "CPU", cpu_width, palette);
                head(ui, "MEMORY", ram_width, palette);
                if show_io {
                    head(ui, "I/O", io_width, palette);
                }
                if show_user {
                    head(ui, "USER", user_width, palette);
                }
            });
            ui.separator();
            let height = app.settings.table_row_height.unwrap_or(
                if app.settings.density == Density::Compact {
                    27.0
                } else {
                    35.0
                },
            );
            egui::ScrollArea::vertical()
                .id_salt("aero-process-list")
                .auto_shrink([false, false])
                .show_rows(ui, height, rows.len(), |ui, range| {
                    for (offset, row) in rows[range.clone()].iter().enumerate() {
                        let p = &row.process;
                        let selected = app.selected == Some((p.pid, p.start_ticks));
                        let index = range.start + offset;
                        let fill = if selected {
                            palette.selection(Color32::from_rgb(205, 239, 229))
                        } else if index % 2 == 0 {
                            palette.table(Color32::from_rgb(241, 253, 249))
                        } else {
                            palette.table(Color32::WHITE)
                        };
                        Frame::NONE.fill(fill).corner_radius(7.0).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let prefix = if row.depth == 0 {
                                    String::new()
                                } else {
                                    format!("{}↳ ", "  ".repeat(row.depth.min(5)))
                                };
                                let (name_rect, response) = ui.allocate_exact_size(
                                    egui::vec2(name_width, height - 3.0),
                                    Sense::click(),
                                );
                                if selected || response.hovered() {
                                    ui.painter().rect_filled(
                                        name_rect,
                                        6.0,
                                        if selected {
                                            palette.selection(Color32::from_rgb(182, 231, 216))
                                        } else {
                                            palette.hover(Color32::from_rgb(220, 246, 239))
                                        },
                                    );
                                }
                                if response.has_focus() {
                                    ui.painter().rect_stroke(
                                        name_rect.shrink(2.0),
                                        6.0,
                                        Stroke::new(2.0_f32, accent),
                                        StrokeKind::Inside,
                                    );
                                }
                                let full_name = format!("{prefix}{}", p.name);
                                response.widget_info(|| {
                                    egui::WidgetInfo::selected(
                                        egui::WidgetType::SelectableLabel,
                                        true,
                                        selected,
                                        &full_name,
                                    )
                                });
                                let max_chars = ((name_width - 23.0) / 7.5).max(6.0) as usize;
                                let name = if full_name.chars().count() > max_chars {
                                    format!(
                                        "{}…",
                                        full_name.chars().take(max_chars - 1).collect::<String>()
                                    )
                                } else {
                                    full_name
                                };
                                ui.painter().text(
                                    name_rect.left_center() + egui::vec2(11.0, 0.0),
                                    Align2::LEFT_CENTER,
                                    name,
                                    FontId::proportional(13.0),
                                    if selected && palette.selection.is_some() {
                                        palette.on_selection(accent)
                                    } else if response.hovered() && palette.hover.is_some() {
                                        readable_text(palette.hover(Color32::WHITE))
                                    } else {
                                        palette.on_table()
                                    },
                                );
                                if response.on_hover_text(&p.command).clicked() {
                                    app.selected = Some((p.pid, p.start_ticks));
                                }
                                let row_text = if selected && palette.selection.is_some() {
                                    palette.on_selection(accent)
                                } else {
                                    palette.on_table()
                                };
                                cell(ui, p.pid.to_string(), pid_width, row_text);
                                cell(ui, format!("{:.1}%", p.cpu_percent), cpu_width, row_text);
                                cell(ui, format::bytes(p.resident_bytes), ram_width, row_text);
                                if show_io {
                                    cell(
                                        ui,
                                        if p.io_accessible {
                                            format::rate(process_io_rate(p))
                                        } else {
                                            "Unavailable".to_owned()
                                        },
                                        io_width,
                                        row_text,
                                    );
                                }
                                if show_user {
                                    cell(
                                        ui,
                                        app.users
                                            .get(&p.user_id)
                                            .cloned()
                                            .unwrap_or_else(|| p.user_id.to_string()),
                                        user_width,
                                        row_text,
                                    );
                                }
                            });
                        });
                    }
                });
        });
}

pub(super) fn performance(app: &TaskottoApp, ui: &mut egui::Ui) {
    let accent = accent(app);
    let palette = AeroPalette::from_settings(&app.settings);
    let Some(s) = &app.snapshot else {
        ui.spinner();
        ui.label("Waiting for live CPU and memory samples…");
        return;
    };
    hero(
        ui,
        "SYSTEM VITALS",
        "A breath of fresh data, every second.",
        Some(s.cpu_percent),
        Some(s.memory.used_fraction() * 100.0),
        accent,
        palette,
    );
    ui.add_space(15.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        let wide = ui.available_width() >= 810.0;
        let memory = format!(
            "{} of {}",
            format::bytes(s.memory.used()),
            format::bytes(s.memory.total)
        );
        let panels = [
            ChartPanel {
                title: "CPU ACTIVITY",
                detail: format!("{:.1}% · {} logical cores", s.cpu_percent, s.logical_cpus),
                primary: &app.history.cpu,
                secondary: None,
                max: 100.0,
                colors: [SKY, accent],
            },
            ChartPanel {
                title: "PHYSICAL MEMORY",
                detail: memory,
                primary: &app.history.memory,
                secondary: None,
                max: 100.0,
                colors: [LEAF, accent],
            },
            ChartPanel {
                title: "DISK ACTIVITY",
                detail: format!(
                    "Read {}  ·  Write {}",
                    format::rate(s.rates.disk_read_per_second),
                    format::rate(s.rates.disk_write_per_second)
                ),
                primary: &app.history.disk_read,
                secondary: Some(&app.history.disk_write),
                max: graph_max(&app.history.disk_read, &app.history.disk_write),
                colors: [AQUA, LEAF],
            },
            ChartPanel {
                title: "NETWORK FLOW",
                detail: format!(
                    "Receive {}  ·  Send {}",
                    format::rate(s.rates.received_per_second),
                    format::rate(s.rates.sent_per_second)
                ),
                primary: &app.history.received,
                secondary: Some(&app.history.sent),
                max: graph_max(&app.history.received, &app.history.sent),
                colors: [SKY, LEAF],
            },
        ];
        if wide {
            ui.columns(2, |columns| {
                for (index, panel) in panels.iter().enumerate() {
                    chart_panel(&mut columns[index % 2], panel, 175.0, palette);
                    if index < 2 {
                        columns[index % 2].add_space(12.0);
                    }
                }
            });
        } else {
            for panel in &panels {
                chart_panel(ui, panel, 156.0, palette);
                ui.add_space(12.0);
            }
        }
        ui.add_space(12.0);
        ui.label(
            RichText::new(format!(
                "Live Linux measurements · uptime {} · {} graph samples retained",
                format::duration(s.uptime_seconds),
                app.history.cpu.len()
            ))
            .small()
            .color(MUTED),
        );
    });
}

pub(super) fn networking(app: &TaskottoApp, ui: &mut egui::Ui) {
    let Some(s) = &app.snapshot else {
        ui.spinner();
        ui.label("Reading real network counters…");
        return;
    };
    let accent = accent(app);
    let palette = AeroPalette::from_settings(&app.settings);
    hero(
        ui,
        "CONNECTED WORLD",
        "Follow the flow of information.",
        None,
        None,
        accent,
        palette,
    );
    ui.add_space(14.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        chart_panel(ui, &ChartPanel {
            title: "NETWORK ACTIVITY · ALL NON-LOOPBACK INTERFACES",
            detail: "Measured bytes per second · receive (blue) / send (green)".to_owned(),
            primary: &app.history.received,
            secondary: Some(&app.history.sent),
            max: graph_max(&app.history.received, &app.history.sent),
            colors: [SKY, LEAF],
        }, 257.0, palette);
        ui.add_space(15.0);
        ui.horizontal_wrapped(|ui| {
            info_chip(ui, "RECEIVING", &format::rate(s.rates.received_per_second), SKY);
            info_chip(ui, "SENDING", &format::rate(s.rates.sent_per_second), LEAF);
            info_chip(ui, "SAMPLES", &app.history.received.len().to_string(), AQUA);
        });
        ui.add_space(17.0);
        ui.label(RichText::new("System-wide traffic, not per-process usage. Virtual interfaces may cause double-counting; connection state and link speed are not inferred.").small().color(MUTED));
    });
}

fn hero(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    cpu: Option<f32>,
    memory: Option<f32>,
    accent: Color32,
    palette: AeroPalette,
) {
    let width = ui.available_width();
    let height = if width < 710.0 { 102.0 } else { 131.0 };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 19.0, Color32::from_rgb(224, 248, 244));
    painter.rect_filled(
        Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.center().y)),
        egui::CornerRadius {
            nw: 19,
            ne: 19,
            sw: 0,
            se: 0,
        },
        Color32::from_rgb(241, 255, 254),
    );
    painter.rect_stroke(
        rect,
        19.0,
        Stroke::new(1.0_f32, palette.border()),
        StrokeKind::Inside,
    );
    let center = Pos2::new(rect.right() - 87.0, rect.center().y);
    // Translucent lens, offset highlights, and repeated bubbles are drawn
    // from vector primitives. No copied wallpaper or trademarked OS assets.
    painter.circle_filled(
        center,
        66.0,
        Color32::from_rgba_unmultiplied(59, 180, 202, 21),
    );
    painter.circle_stroke(
        center,
        67.0,
        Stroke::new(2.0_f32, Color32::from_rgb(180, 231, 226)),
    );
    for (offset, radius) in [(-37.0, 7.0), (-9.0, 4.0), (39.0, 11.0)] {
        painter.circle_stroke(
            center + egui::vec2(offset, -37.0 + offset.abs() / 4.0),
            radius,
            Stroke::new(1.3_f32, Color32::from_rgb(126, 207, 209)),
        );
    }
    painter.text(
        rect.min + egui::vec2(22.0, if width < 710.0 { 29.0 } else { 40.0 }),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(if width < 710.0 { 17.0 } else { 21.0 }),
        INK,
    );
    painter.text(
        rect.min + egui::vec2(22.0, if width < 710.0 { 55.0 } else { 71.0 }),
        Align2::LEFT_CENTER,
        subtitle,
        FontId::proportional(13.0),
        MUTED,
    );
    painter.line_segment(
        [
            rect.min + egui::vec2(22.0, height - 17.0),
            rect.min + egui::vec2(106.0, height - 17.0),
        ],
        Stroke::new(3.0_f32, accent),
    );
    if width >= 710.0 {
        if let Some(cpu) = cpu {
            meter(
                ui,
                Pos2::new(rect.right() - 192.0, rect.center().y),
                37.0,
                cpu,
                SKY,
                "CPU",
            );
        }
        if let Some(memory) = memory {
            meter(
                ui,
                Pos2::new(rect.right() - 89.0, rect.center().y),
                37.0,
                memory,
                LEAF,
                "RAM",
            );
        }
    }
}

struct ChartPanel<'a> {
    title: &'static str,
    detail: String,
    primary: &'a VecDeque<f32>,
    secondary: Option<&'a VecDeque<f32>>,
    max: f32,
    colors: [Color32; 2],
}

fn chart_panel(ui: &mut egui::Ui, chart: &ChartPanel<'_>, chart_height: f32, palette: AeroPalette) {
    Frame::new()
        .fill(palette.table(Color32::from_rgb(248, 255, 254)))
        .stroke(Stroke::new(1.0_f32, palette.border()))
        .corner_radius(15.0)
        .inner_margin(Margin::same(13))
        .show(ui, |ui| {
            ui.set_min_width((ui.available_width() - 5.0).max(190.0));
            ui.label(
                RichText::new(chart.title)
                    .strong()
                    .size(13.0)
                    .color(palette.on_table()),
            );
            ui.label(
                RichText::new(&chart.detail)
                    .size(12.0)
                    .color(if palette.table.is_some() {
                        palette.on_table()
                    } else {
                        MUTED
                    }),
            );
            ui.add_space(7.0);
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width().max(1.0), chart_height),
                Sense::hover(),
            );
            let painter = ui.painter();
            painter.rect_filled(rect, 9.0, palette.table(Color32::from_rgb(232, 251, 249)));
            painter.rect_filled(
                Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.center().y)),
                egui::CornerRadius {
                    nw: 9,
                    ne: 9,
                    sw: 0,
                    se: 0,
                },
                palette.table(Color32::from_rgb(244, 255, 255)),
            );
            for i in 1..4 {
                let y = rect.top() + rect.height() * i as f32 / 4.0;
                painter.line_segment(
                    [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                    Stroke::new(
                        1.0_f32,
                        palette.border.unwrap_or(Color32::from_rgb(204, 231, 227)),
                    ),
                );
            }
            for i in 1..6 {
                let x = rect.left() + rect.width() * i as f32 / 6.0;
                painter.line_segment(
                    [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                    Stroke::new(
                        1.0_f32,
                        palette.border.unwrap_or(Color32::from_rgb(220, 242, 238)),
                    ),
                );
            }
            plot(
                ui,
                rect.shrink(8.0),
                chart.primary,
                chart.max,
                chart.colors[0],
            );
            if let Some(other) = chart.secondary {
                plot(ui, rect.shrink(8.0), other, chart.max, chart.colors[1]);
            }
            painter.rect_stroke(rect, 9.0, Stroke::new(1.0_f32, BORDER), StrokeKind::Inside);
        });
}

fn plot(ui: &egui::Ui, rect: Rect, history: &VecDeque<f32>, max: f32, color: Color32) {
    if history.is_empty() || rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }
    let denominator = max.max(1.0);
    // Keep the window fixed at 120 samples, so new data advances rightward.
    let points: Vec<Pos2> = history
        .iter()
        .enumerate()
        .map(|(i, value)| {
            Pos2::new(
                rect.right() - (history.len().saturating_sub(i + 1) as f32 / 119.0) * rect.width(),
                rect.bottom() - (value.max(0.0) / denominator).clamp(0.0, 1.0) * rect.height(),
            )
        })
        .collect();
    if points.len() > 1 {
        ui.painter()
            .add(egui::Shape::line(points, Stroke::new(2.1_f32, color)));
    } else {
        ui.painter().circle_filled(points[0], 2.5, color);
    }
}

fn paint_sky(ui: &egui::Ui, rect: Rect, accent: Color32) {
    // At most 90 painter primitives per frame; no textures or animations.
    let steps = 90;
    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let base = if t < 0.46 {
            blend(
                Color32::from_rgb(197, 239, 252),
                Color32::from_rgb(137, 212, 230),
                t / 0.46,
            )
        } else {
            blend(
                Color32::from_rgb(137, 212, 230),
                Color32::from_rgb(211, 247, 231),
                (t - 0.46) / 0.54,
            )
        };
        let color = blend(base, accent, 0.12);
        ui.painter().rect_filled(
            Rect::from_min_size(
                Pos2::new(rect.left(), rect.top() + rect.height() * t),
                egui::vec2(rect.width(), rect.height() / steps as f32 + 1.0),
            ),
            0.0,
            color,
        );
    }
    ui.painter().line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(1.0_f32, Color32::from_rgb(87, 171, 178)),
    );
}

fn paint_orb(ui: &egui::Ui, center: Pos2, radius: f32, color: Color32) {
    let p = ui.painter();
    p.circle_filled(center, radius, Color32::from_rgb(246, 255, 255));
    p.circle_filled(center, radius - 3.0, color);
    p.circle_filled(
        center - egui::vec2(6.0, 9.0),
        radius * 0.53,
        Color32::from_rgb(162, 246, 251),
    );
    p.circle_stroke(
        center,
        radius,
        Stroke::new(1.5_f32, Color32::from_rgb(65, 146, 157)),
    );
    for (i, magnitude) in [11.0, 18.0, 9.0, 22.0].iter().enumerate() {
        let x = center.x - 12.0 + i as f32 * 8.0;
        p.line_segment(
            [
                Pos2::new(x, center.y + 11.0),
                Pos2::new(x, center.y + 11.0 - magnitude),
            ],
            Stroke::new(3.0_f32, Color32::WHITE),
        );
    }
}

fn meter(ui: &egui::Ui, center: Pos2, radius: f32, percent: f32, color: Color32, label: &str) {
    let p = ui.painter();
    p.circle_filled(center, radius + 7.0, Color32::from_rgb(245, 255, 252));
    p.circle_stroke(
        center,
        radius,
        Stroke::new(7.0_f32, Color32::from_rgb(206, 230, 225)),
    );
    let end = (percent / 100.0).clamp(0.0, 1.0);
    let mut arc = Vec::with_capacity(65);
    for step in 0..=64 {
        let t = step as f32 / 64.0;
        if t > end {
            break;
        }
        let angle = std::f32::consts::TAU * (t - 0.25);
        arc.push(center + egui::vec2(angle.cos(), angle.sin()) * radius);
    }
    if arc.len() > 1 {
        p.add(egui::Shape::line(arc, Stroke::new(7.0_f32, color)));
    }
    p.text(
        center + egui::vec2(0.0, -6.0),
        Align2::CENTER_CENTER,
        format!("{percent:.0}%"),
        FontId::proportional(17.0),
        INK,
    );
    p.text(
        center + egui::vec2(0.0, 13.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(11.0),
        MUTED,
    );
}

fn info_chip(ui: &mut egui::Ui, title: &str, value: &str, color: Color32) {
    Frame::new()
        .fill(Color32::from_rgb(244, 255, 252))
        .stroke(Stroke::new(1.0_f32, BORDER))
        .corner_radius(13.0)
        .inner_margin(Margin::symmetric(17, 11))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(color).size(11.0));
            ui.label(RichText::new(value).strong().size(20.0).color(INK));
        });
}

fn head(ui: &mut egui::Ui, label: &str, width: f32, palette: AeroPalette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 21.0), Sense::hover());
    ui.painter().text(
        rect.left_center() + egui::vec2(if label == "PROCESS" { 11.0 } else { 0.0 }, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        if palette.table.is_some() {
            palette.on_table()
        } else {
            MUTED
        },
    );
}

fn cell(ui: &mut egui::Ui, label: String, width: f32, ink: Color32) {
    ui.add_sized(
        [width, 28.0],
        egui::Label::new(RichText::new(label).color(ink).size(12.5)),
    );
}

fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 * (1.0 - t) + b.r() as f32 * t) as u8,
        (a.g() as f32 * (1.0 - t) + b.g() as f32 * t) as u8,
        (a.b() as f32 * (1.0 - t) + b.b() as f32 * t) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_tokens_preserve_default_and_recolor_custom_aero_surfaces() {
        let base = AeroPalette::from_settings(&Settings::default());
        assert_eq!(base.control(Color32::WHITE), Color32::WHITE);
        assert_eq!(base.hover(SKY), SKY);
        assert_eq!(base.border(), BORDER);
        assert_eq!(base.table(Color32::WHITE), Color32::WHITE);
        assert_eq!(base.selection(AQUA), AQUA);
        assert_eq!(base.on_table(), INK);
        assert_eq!(base.on_selection(AQUA), Color32::WHITE);

        let mut settings = Settings::default();
        settings.components.control = Some("#223344".into());
        settings.components.hover = Some("#334455".into());
        settings.components.border = Some("#445566".into());
        settings.components.table = Some("#192A40".into());
        settings.components.selection = Some("#F1F2F3".into());
        let custom = AeroPalette::from_settings(&settings);
        assert_eq!(
            custom.control(Color32::WHITE),
            Color32::from_rgb(34, 51, 68)
        );
        assert_eq!(custom.hover(SKY), Color32::from_rgb(51, 68, 85));
        assert_eq!(custom.border(), Color32::from_rgb(68, 85, 102));
        assert_eq!(custom.on_control(Color32::WHITE), Color32::WHITE);
        assert_eq!(custom.on_table(), Color32::WHITE);
        assert_eq!(custom.on_selection(AQUA), Color32::BLACK);
    }
}
