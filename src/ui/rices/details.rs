//! Native-feeling XP process properties; uses real Linux process fields and
//! the same PID-reuse-safe signal confirmation as the regular inspector.
use super::*;

pub(super) fn xp_page(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    let Some(p) = app.selected_process().cloned() else {
        ui.label("Select a process on the Processes tab to inspect its properties.");
        if ui.button("Go to Processes").clicked() {
            app.page = Page::Processes;
        }
        return;
    };
    ui.label(
        RichText::new(format!("{} Properties", p.name))
            .size(15.0)
            .strong(),
    );
    ui.add_space(8.0);
    let seven = app.settings.theme == Theme::Windows7;
    egui::ScrollArea::vertical().show(ui, |ui| {
        classic_group(
            ui,
            seven,
            "Identification",
            &[
                ("Image Name", p.name.clone()),
                // Keep the properties grid bounded even for very long command
                // lines (some launchers carry thousands of bytes of arguments).
                ("Command", short_command(&p.command)),
                ("PID", p.pid.to_string()),
                ("Parent PID", p.parent_pid.to_string()),
                (
                    "User Name",
                    app.users
                        .get(&p.user_id)
                        .cloned()
                        .unwrap_or_else(|| p.user_id.to_string()),
                ),
                ("Status", state_name(p.state).to_owned()),
            ],
        );
        ui.add_space(9.0);
        classic_group(
            ui,
            seven,
            "Performance",
            &[
                ("CPU Usage", format!("{:.1}%", p.cpu_percent)),
                ("Mem Usage", format::bytes(p.resident_bytes)),
                ("VM Size", format::bytes(p.virtual_bytes)),
                ("Threads", p.threads.to_string()),
                ("Start Time", format::local_timestamp(p.start_unix_seconds)),
                ("Running Time", format::process_age(p.elapsed_seconds)),
            ],
        );
        ui.add_space(9.0);
        classic_group(
            ui,
            seven,
            "I/O Counters",
            &[
                (
                    "Read bytes",
                    format_process_io(p.io_accessible, p.read_bytes, false),
                ),
                (
                    "Write bytes",
                    format_process_io(p.io_accessible, p.written_bytes, false),
                ),
                (
                    "Read speed",
                    format_process_io(p.io_accessible, p.read_per_second, true),
                ),
                (
                    "Write speed",
                    format_process_io(p.io_accessible, p.written_per_second, true),
                ),
            ],
        );
        ui.add_space(10.0);
        ui.label(RichText::new("Process actions").size(12.0).strong());
        ui.horizontal_wrapped(|ui| {
            for (label, action) in [
                ("End Process…", SignalAction::Terminate),
                ("Suspend…", SignalAction::Stop),
                ("Resume…", SignalAction::Continue),
                ("Force End…", SignalAction::ForceKill),
            ] {
                if ui
                    .add_enabled(can_signal_process(&p), egui::Button::new(label))
                    .clicked()
                {
                    app.queue_action(action);
                }
            }
        });
        ui.label(
            RichText::new("Actions use your Linux permissions and require confirmation.")
                .small()
                .weak(),
        );
    });
}

fn short_command(value: &str) -> String {
    const MAX_CHARS: usize = 74;
    if value.chars().count() <= MAX_CHARS {
        return value.to_owned();
    }
    let mut result: String = value.chars().take(MAX_CHARS - 1).collect();
    result.push('…');
    result
}

fn classic_group(ui: &mut egui::Ui, seven: bool, title: &str, rows: &[(&str, String)]) {
    ui.label(RichText::new(title).size(12.0).strong());
    egui::Frame::new()
        .fill(if seven {
            Color32::WHITE
        } else {
            Color32::from_rgb(250, 249, 241)
        })
        .stroke(Stroke::new(
            1.0_f32,
            if seven {
                Color32::from_rgb(145, 162, 178)
            } else {
                Color32::from_rgb(156, 154, 137)
            },
        ))
        .inner_margin(egui::Margin::same(9))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width().max(320.0) - 20.0);
            egui::Grid::new(("xp-properties", title))
                .num_columns(2)
                .spacing([26.0, 7.0])
                .show(ui, |ui| {
                    for (label, value) in rows {
                        ui.label(RichText::new(*label).size(12.0));
                        ui.label(RichText::new(value).size(12.0));
                        ui.end_row();
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::short_command;

    #[test]
    fn command_preview_handles_long_unicode_values() {
        let long = "🐧".repeat(100);
        let short = short_command(&long);
        assert_eq!(short.chars().count(), 74);
        assert!(short.ends_with('…'));
        assert_eq!(short_command("cargo test"), "cargo test");
    }
}
