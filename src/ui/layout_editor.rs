//! Session-only editor for the same strictly validated RicePack layout schema.
//! Edits never alter disk, monitoring rights or a watched RicePack file.
use super::*;
use crate::layout::{
    Gauge, LayoutSpec, MAX_PAGES, MAX_WIDGETS_PER_PAGE, Metric, Plot, ProcessColumn, Screen,
    ToolbarAction, Widget,
};

pub(super) fn starter() -> LayoutSpec {
    LayoutSpec {
        compact_below: Some(680),
        pages: vec![Screen {
            id: "overview".into(),
            title: "Overview".into(),
            widgets: vec![
                Widget::Metric {
                    label: "CPU load".into(),
                    binding: Metric::CpuPercent,
                },
                Widget::ProcessList {
                    columns: vec![ProcessColumn::Name, ProcessColumn::Cpu, ProcessColumn::Pid],
                },
            ],
            compact_widgets: Vec::new(),
        }],
    }
}

pub(super) fn draw(app: &mut TaskottoApp, ui: &mut egui::Ui, xp: bool) {
    preference_section(ui, "Screen layout designer", xp);
    ui.label(
        RichText::new("Compose pages from safe, live-data widgets. A draft is only applied when valid; export the appearance above to save a new RicePack file.")
            .small()
            .weak(),
    );
    if app.ricepack_library.editor_layout.is_none() {
        if ui.button("Open layout designer").clicked() {
            app.ricepack_library.editor_layout =
                Some(app.settings.custom_layout.clone().unwrap_or_else(starter));
            app.ricepack_library.editor_page = 0;
            app.ricepack_library.editor_compact = false;
            app.ricepack_library.editor_error = None;
        }
        return;
    }

    // Work on a disconnected draft, not the monitor's active appearance.
    // Invalid partial text edits remain local to the designer until Apply.
    let mut draft = app
        .ricepack_library
        .editor_layout
        .take()
        .expect("editor open");
    let mut page = app.ricepack_library.editor_page.min(draft.pages.len() - 1);
    let mut compact = app.ricepack_library.editor_compact;
    let mut error = app.ricepack_library.editor_error.take();
    let mut apply = false;
    let mut close = false;

    ui.horizontal_wrapped(|ui| {
        if ui.button("Reset draft from active layout").clicked() {
            draft = app.settings.custom_layout.clone().unwrap_or_else(starter);
            page = 0;
            error = None;
        }
        if ui.button("Discard draft").clicked() {
            close = true;
        }
        if ui.button("Apply to this session").clicked() {
            apply = true;
        }
    });
    let mut breakpoint = draft.compact_below.unwrap_or(680);
    if ui
        .add(egui::Slider::new(&mut breakpoint, 480..=1024).text("Compact below (inner px)"))
        .changed()
    {
        draft.compact_below = Some(breakpoint);
    }
    ui.horizontal_wrapped(|ui| {
        egui::ComboBox::from_id_salt("screen-designer-pages")
            .selected_text(&draft.pages[page].title)
            .show_ui(ui, |ui| {
                for (index, screen) in draft.pages.iter().enumerate() {
                    ui.selectable_value(&mut page, index, &screen.title);
                }
            });
        if ui
            .add_enabled(
                draft.pages.len() < MAX_PAGES,
                egui::Button::new("Add screen"),
            )
            .clicked()
        {
            let index = draft.pages.len() + 1;
            draft.pages.push(Screen {
                id: format!("screen-{index}"),
                title: format!("Screen {index}"),
                widgets: vec![Widget::Heading {
                    text: format!("Screen {index}"),
                }],
                compact_widgets: vec![],
            });
            page = draft.pages.len() - 1;
        }
        if ui
            .add_enabled(draft.pages.len() > 1, egui::Button::new("Remove screen"))
            .clicked()
        {
            draft.pages.remove(page);
            page = page.min(draft.pages.len() - 1);
        }
    });
    let screen = &mut draft.pages[page];
    ui.horizontal_wrapped(|ui| {
        ui.label("ID");
        ui.add(egui::TextEdit::singleline(&mut screen.id).desired_width(120.0));
        ui.label("Title");
        ui.add(egui::TextEdit::singleline(&mut screen.title).desired_width(210.0));
    });
    ui.horizontal_wrapped(|ui| {
        ui.radio_value(&mut compact, false, "Standard / large");
        ui.radio_value(&mut compact, true, "Compact fallback");
        if compact
            && screen.compact_widgets.is_empty()
            && ui
                .button("Copy standard widgets as starting point")
                .clicked()
        {
            screen.compact_widgets.clone_from(&screen.widgets);
        }
    });
    let widgets = if compact {
        &mut screen.compact_widgets
    } else {
        &mut screen.widgets
    };
    if widgets.is_empty() {
        ui.label("No compact override. The standard widgets will remain visible at all widths.");
    }
    let count = widgets.len();
    let mut edit = None;
    for (index, widget) in widgets.iter_mut().enumerate() {
        let operation = ui
            .push_id((page, compact, index), |ui| {
                widget_controls(ui, widget, index, count)
            })
            .inner;
        if operation.is_some() {
            edit = operation;
        }
    }
    match edit {
        Some(WidgetEdit::Up(index)) => widgets.swap(index, index - 1),
        Some(WidgetEdit::Down(index)) => widgets.swap(index, index + 1),
        Some(WidgetEdit::Remove(index)) => {
            widgets.remove(index);
        }
        None => {}
    }
    ui.horizontal_wrapped(|ui| {
        if widgets.len() < MAX_WIDGETS_PER_PAGE {
            for (name, widget) in presets() {
                if ui.small_button(format!("+ {name}")).clicked() {
                    widgets.push(widget);
                }
            }
        } else {
            ui.label("Widget limit reached (16 per screen).");
        }
    });

    if apply {
        match draft.validate() {
            Ok(()) => {
                app.settings.custom_layout = Some(draft.clone());
                app.notification = Some(
                    "Valid screen layout applied for this session. Export current appearance to save it as a new RicePack.".into(),
                );
                error = None;
            }
            Err(reason) => error = Some(format!("Layout not applied: {reason}")),
        }
    }
    if let Some(message) = error.as_deref() {
        ui.colored_label(ui.visuals().error_fg_color, message);
    }
    if !close {
        app.ricepack_library.editor_layout = Some(draft);
    }
    app.ricepack_library.editor_page = page;
    app.ricepack_library.editor_compact = compact;
    app.ricepack_library.editor_error = if close { None } else { error };
}

fn presets() -> [(&'static str, Widget); 7] {
    [
        (
            "Heading",
            Widget::Heading {
                text: "Section".into(),
            },
        ),
        (
            "Metric",
            Widget::Metric {
                label: "CPU".into(),
                binding: Metric::CpuPercent,
            },
        ),
        (
            "Graph",
            Widget::Plot {
                label: "CPU history".into(),
                binding: Plot::Cpu,
            },
        ),
        (
            "Gauge",
            Widget::Gauge {
                label: "Memory".into(),
                binding: Gauge::MemoryPercent,
            },
        ),
        (
            "Status",
            Widget::StatusStrip {
                bindings: vec![Metric::Uptime],
            },
        ),
        (
            "Toolbar",
            Widget::Toolbar {
                actions: vec![ToolbarAction::Refresh],
            },
        ),
        (
            "Processes",
            Widget::ProcessList {
                columns: vec![ProcessColumn::Name, ProcessColumn::Pid],
            },
        ),
    ]
}

enum WidgetEdit {
    Up(usize),
    Down(usize),
    Remove(usize),
}

fn widget_controls(
    ui: &mut egui::Ui,
    widget: &mut Widget,
    index: usize,
    count: usize,
) -> Option<WidgetEdit> {
    let mut operation = None;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{}. {}", index + 1, kind_label(widget))).strong());
            if index > 0 && ui.small_button("Move up").clicked() {
                operation = Some(WidgetEdit::Up(index));
            }
            if index + 1 < count && ui.small_button("Move down").clicked() {
                operation = Some(WidgetEdit::Down(index));
            }
            if ui.small_button("Remove ×").clicked() {
                operation = Some(WidgetEdit::Remove(index));
            }
        });
        match widget {
            Widget::Heading { text } => {
                ui.add(egui::TextEdit::singleline(text).desired_width(300.0));
            }
            Widget::Metric { label, binding } => {
                caption(ui, label);
                metric_choice(ui, binding);
            }
            Widget::Plot { label, binding } => {
                caption(ui, label);
                egui::ComboBox::from_id_salt("plot-source")
                    .selected_text(format!("{:?}", binding))
                    .show_ui(ui, |ui| {
                        for (value, name) in [
                            (Plot::Cpu, "CPU"),
                            (Plot::Memory, "Memory"),
                            (Plot::NetworkReceived, "Network receive"),
                            (Plot::NetworkSent, "Network send"),
                            (Plot::DiskRead, "Disk read"),
                            (Plot::DiskWrite, "Disk write"),
                        ] {
                            ui.selectable_value(binding, value, name);
                        }
                    });
            }
            Widget::Gauge { label, binding } => {
                caption(ui, label);
                egui::ComboBox::from_id_salt("gauge-source")
                    .selected_text(match binding {
                        Gauge::CpuPercent => "CPU",
                        Gauge::MemoryPercent => "RAM",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(binding, Gauge::CpuPercent, "CPU load");
                        ui.selectable_value(binding, Gauge::MemoryPercent, "Memory used");
                    });
            }
            Widget::StatusStrip { bindings } => {
                ui.label("Metrics (1–5)");
                ui.horizontal_wrapped(|ui| {
                    for value in all_metrics() {
                        let mut selected = bindings.contains(&value);
                        if ui.checkbox(&mut selected, value.label()).changed() {
                            if selected {
                                bindings.push(value);
                            } else {
                                bindings.retain(|item| *item != value);
                            }
                        }
                    }
                });
            }
            Widget::Toolbar { actions } => {
                ui.label("Built-in actions only (1–3)");
                ui.horizontal_wrapped(|ui| {
                    for (value, name) in [
                        (ToolbarAction::Refresh, "Refresh"),
                        (ToolbarAction::FocusSearch, "Search"),
                        (ToolbarAction::EndProcess, "Confirm end process"),
                    ] {
                        let mut selected = actions.contains(&value);
                        if ui.checkbox(&mut selected, name).changed() {
                            if selected {
                                actions.push(value);
                            } else {
                                actions.retain(|item| *item != value);
                            }
                        }
                    }
                });
            }
            Widget::ProcessList { columns } => {
                ui.label("Process columns; Name is required for safe selection");
                ui.horizontal_wrapped(|ui| {
                    let mut required = true;
                    ui.add_enabled(false, egui::Checkbox::new(&mut required, "Name"));
                    for (value, name) in [
                        (ProcessColumn::Pid, "PID"),
                        (ProcessColumn::Cpu, "CPU"),
                        (ProcessColumn::Memory, "Memory"),
                        (ProcessColumn::Threads, "Threads"),
                    ] {
                        let mut selected = columns.contains(&value);
                        if ui.checkbox(&mut selected, name).changed() {
                            if selected {
                                columns.push(value);
                            } else {
                                columns.retain(|item| *item != value);
                            }
                        }
                    }
                });
            }
        }
    });
    operation
}

fn caption(ui: &mut egui::Ui, value: &mut String) {
    ui.add(egui::TextEdit::singleline(value).desired_width(245.0));
}

fn metric_choice(ui: &mut egui::Ui, binding: &mut Metric) {
    egui::ComboBox::from_id_salt("metric-source")
        .selected_text(binding.label())
        .show_ui(ui, |ui| {
            for value in all_metrics() {
                ui.selectable_value(binding, value, value.label());
            }
        });
}

fn all_metrics() -> [Metric; 10] {
    [
        Metric::CpuPercent,
        Metric::MemoryUsed,
        Metric::MemoryTotal,
        Metric::MemoryPercent,
        Metric::Processes,
        Metric::Uptime,
        Metric::NetworkReceived,
        Metric::NetworkSent,
        Metric::DiskRead,
        Metric::DiskWrite,
    ]
}

fn kind_label(widget: &Widget) -> &'static str {
    match widget {
        Widget::Heading { .. } => "Heading",
        Widget::Metric { .. } => "Metric",
        Widget::Plot { .. } => "History graph",
        Widget::Gauge { .. } => "Gauge",
        Widget::StatusStrip { .. } => "Status strip",
        Widget::Toolbar { .. } => "Toolbar",
        Widget::ProcessList { .. } => "Process list",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starter_and_widget_presets_are_valid_data_only_layouts() {
        let mut layout = starter();
        assert!(layout.validate().is_ok());
        for (_, widget) in presets() {
            layout.pages[0].widgets.push(widget);
        }
        assert!(layout.validate().is_ok());
        assert_eq!(presets().len(), 7);
        assert!(
            layout.pages[0]
                .widgets
                .iter()
                .all(|widget| !kind_label(widget).is_empty())
        );
    }
}
