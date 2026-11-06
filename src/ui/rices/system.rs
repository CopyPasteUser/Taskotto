//! Windows 10-style management tabs backed by Linux information. None of
//! these views claims to expose Windows APIs or historical Windows counters.
use super::*;
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command as SystemCommand,
    sync::mpsc::{self, Receiver},
    thread,
};

#[derive(Debug, Clone)]
struct StartupEntry {
    name: String,
    command: String,
    source: String,
    disabled: bool,
}

#[derive(Debug, Clone)]
pub(super) struct ServiceEntry {
    pub(super) unit: String,
    pub(super) active: String,
    pub(super) state: String,
    pub(super) description: String,
}

#[derive(Default)]
pub(in crate::ui) struct SystemPages {
    startup: Option<(Vec<StartupEntry>, Vec<String>)>,
    pub(super) services: Option<Result<Vec<ServiceEntry>, String>>,
    pub(super) services_loading: Option<Receiver<Result<Vec<ServiceEntry>, String>>>,
}

pub(super) fn app_history(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.heading(RichText::new("App history").size(24.0));
    ui.label(
        RichText::new(
            "Linux process lifetime · currently running processes, not Windows app history",
        )
        .small()
        .weak(),
    );
    ui.add_space(12.0);
    let Some(snapshot) = &app.snapshot else {
        ui.spinner();
        return;
    };
    let mut processes: Vec<_> = snapshot
        .processes
        .iter()
        .filter(|process| process.user_id == app.my_uid)
        .collect();
    processes.sort_unstable_by_key(|p| std::cmp::Reverse(p.total_ticks));
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64;
    ui.label(
        "CPU time is accumulated since each process started; exited processes are not retained.",
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.add_sized([270.0, 26.0], egui::Label::new("Name"));
        ui.add_sized([84.0, 26.0], egui::Label::new("PID"));
        ui.add_sized([122.0, 26.0], egui::Label::new("CPU time"));
        ui.label("Memory");
    });
    ui.separator();
    if processes.is_empty() {
        ui.label("No running processes belong to the current user.");
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show_rows(ui, 28.0, processes.len(), |ui, range| {
            for index in range {
                let p = processes[index];
                ui.horizontal(|ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(egui::vec2(270.0, 24.0), Sense::click());
                    if response.hovered() {
                        ui.painter()
                            .rect_filled(rect, 0.0, Color32::from_rgb(226, 241, 252));
                    }
                    ui.painter().with_clip_rect(rect.shrink(3.0)).text(
                        rect.left_center() + egui::vec2(9.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        &p.name,
                        egui::FontId::proportional(13.0),
                        Color32::from_rgb(35, 35, 35),
                    );
                    if response.on_hover_text(&p.command).clicked() {
                        app.selected = Some((p.pid, p.start_ticks));
                        app.page = Page::Details;
                    }
                    ui.add_sized([84.0, 24.0], egui::Label::new(p.pid.to_string()));
                    ui.add_sized(
                        [122.0, 24.0],
                        egui::Label::new(format_cpu_time(p.total_ticks, ticks_per_second)),
                    );
                    ui.label(format::bytes(p.resident_bytes));
                });
            }
        });
}

fn format_cpu_time(ticks: u64, ticks_per_second: f64) -> String {
    let seconds = (ticks as f64 / ticks_per_second.max(1.0)) as u64;
    format!(
        "{:02}:{:02}:{:02}",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

pub(super) fn startup(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Startup").size(24.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Refresh").clicked() {
                app.system_pages.startup = None;
            }
        });
    });
    ui.label(
        RichText::new("XDG desktop-session autostart · read-only · not boot services")
            .small()
            .weak(),
    );
    ui.add_space(10.0);
    let (entries, warnings) = app
        .system_pages
        .startup
        .get_or_insert_with(|| load_autostart(&xdg_autostart_paths()));
    for warning in warnings.iter().take(2) {
        ui.colored_label(Color32::from_rgb(150, 75, 27), warning);
    }
    if entries.is_empty() {
        ui.label("No XDG autostart desktop files were found in the configured directories.");
        return;
    }
    egui::ScrollArea::both().show(ui, |ui| {
        egui::Grid::new("win10-startup-table")
            .num_columns(4)
            .striped(true)
            .min_col_width(98.0)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                for title in ["Name", "Status", "Source", "Command"] {
                    ui.label(RichText::new(title).strong());
                }
                ui.end_row();
                for entry in entries {
                    ui.label(&entry.name);
                    ui.label(if entry.disabled {
                        "Disabled"
                    } else {
                        "Configured"
                    });
                    ui.label(&entry.source);
                    ui.label(RichText::new(truncated(&entry.command, 72)).small())
                        .on_hover_text(&entry.command);
                    ui.end_row();
                }
            });
    });
    ui.label(RichText::new("Configured does not guarantee the current desktop starts an entry (OnlyShowIn, TryExec and desktop policy can restrict it).").small().weak());
}

fn xdg_autostart_paths() -> Vec<(PathBuf, &'static str)> {
    let user = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    let mut paths = Vec::new();
    if let Some(user) = user {
        paths.push((user.join("autostart"), "User"));
    }
    let dirs = env::var("XDG_CONFIG_DIRS")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/etc/xdg".into());
    for path in dirs.split(':').map(Path::new).filter(|p| p.is_absolute()) {
        paths.push((path.join("autostart"), "System"));
    }
    paths
}

fn load_autostart(paths: &[(PathBuf, &'static str)]) -> (Vec<StartupEntry>, Vec<String>) {
    let mut seen = HashSet::new();
    let mut entries = Vec::new();
    let mut warnings = Vec::new();
    for (directory, source) in paths {
        let files = match fs::read_dir(directory) {
            Ok(files) => files,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => {
                warnings.push(format!("Cannot read {}: {err}", directory.display()));
                continue;
            }
        };
        for file in files.take(2048) {
            let Ok(file) = file else { continue };
            let path = file.path();
            let Some(id) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !id.ends_with(".desktop") || !seen.insert(id.to_owned()) {
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(text) => {
                    if let Some(mut entry) = parse_autostart(&text, id) {
                        entry.source = (*source).to_owned();
                        entries.push(entry);
                    }
                }
                Err(err) => warnings.push(format!("Cannot read {}: {err}", path.display())),
            }
        }
    }
    entries.sort_unstable_by_key(|item| item.name.to_lowercase());
    (entries, warnings)
}

fn parse_autostart(text: &str, fallback_name: &str) -> Option<StartupEntry> {
    let mut in_desktop_entry = false;
    let mut name = None;
    let mut command = String::new();
    let mut hidden = false;
    let mut enabled = true;
    for line in text.lines().map(str::trim) {
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "Name" => name = Some(value.trim().to_owned()),
            "Exec" => command = value.trim().to_owned(),
            "Hidden" => hidden = value.trim().eq_ignore_ascii_case("true"),
            "X-GNOME-Autostart-enabled" => enabled = !value.trim().eq_ignore_ascii_case("false"),
            _ => {}
        }
    }
    if !text.lines().any(|line| line.trim() == "[Desktop Entry]") {
        return None;
    }
    Some(StartupEntry {
        name: name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| fallback_name.to_owned()),
        command,
        source: String::new(),
        disabled: hidden || !enabled,
    })
}

fn truncated(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_owned();
    }
    let mut preview = input
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    preview.push('…');
    preview
}

pub(super) fn services(app: &mut TaskottoApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("Services").size(24.0));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Refresh").clicked() {
                app.system_pages.services = None;
                app.system_pages.services_loading = None;
            }
        });
    });
    ui.label(
        RichText::new("Your systemd user services · read-only, not Windows services")
            .small()
            .weak(),
    );
    ui.add_space(10.0);
    poll_services(app, ui.ctx());
    let Some(result) = &app.system_pages.services else {
        ui.spinner();
        ui.label("Reading user services…");
        return;
    };
    let Ok(entries) = result else {
        ui.colored_label(
            Color32::from_rgb(155, 66, 26),
            "systemd user service manager unavailable",
        );
        ui.label(result.as_ref().unwrap_err());
        return;
    };
    if entries.is_empty() {
        ui.label("No user services are loaded in the current systemd manager.");
        return;
    }
    egui::ScrollArea::both().show(ui, |ui| {
        egui::Grid::new("win10-services-table")
            .num_columns(4)
            .striped(true)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                for label in ["Name", "Status", "State", "Description"] {
                    ui.label(RichText::new(label).strong());
                }
                ui.end_row();
                for entry in entries {
                    ui.label(&entry.unit);
                    ui.label(&entry.active);
                    ui.label(&entry.state);
                    ui.label(truncated(&entry.description, 80));
                    ui.end_row();
                }
            });
    });
}

pub(super) fn poll_services(app: &mut TaskottoApp, ctx: &egui::Context) {
    if app.system_pages.services.is_none() && app.system_pages.services_loading.is_none() {
        let (tx, rx) = mpsc::channel();
        let context = ctx.clone();
        thread::spawn(move || {
            let result = query_user_services();
            let _ = tx.send(result);
            context.request_repaint();
        });
        app.system_pages.services_loading = Some(rx);
    }
    if let Some(rx) = &app.system_pages.services_loading {
        match rx.try_recv() {
            Ok(data) => {
                app.system_pages.services = Some(data);
                app.system_pages.services_loading = None;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                app.system_pages.services = Some(Err("Service query stopped unexpectedly".into()));
                app.system_pages.services_loading = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
}

fn query_user_services() -> Result<Vec<ServiceEntry>, String> {
    let output = SystemCommand::new("systemctl")
        .args([
            "--user",
            "--all",
            "--type=service",
            "--plain",
            "--no-legend",
            "--no-pager",
            "list-units",
        ])
        .output()
        .map_err(|err| format!("Cannot execute systemctl: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(truncated(stderr.trim(), 200));
    }
    Ok(parse_service_units(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

fn parse_service_units(text: &str) -> Vec<ServiceEntry> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim().trim_start_matches('●').trim();
            let mut fields = line.split_whitespace();
            let unit = fields.next()?;
            if !unit.ends_with(".service") {
                return None;
            }
            let _load = fields.next()?;
            let active = fields.next()?;
            let state = fields.next()?;
            Some(ServiceEntry {
                unit: unit.to_owned(),
                active: active.to_owned(),
                state: state.to_owned(),
                description: fields.collect::<Vec<_>>().join(" "),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_desktop_section_and_disabled_state() {
        let text = "[Other]\nName=Wrong\n[Desktop Entry]\nName=My Tool\nExec=tool --flag\nHidden=true\n[Other]\nExec=wrong";
        let entry = parse_autostart(text, "fallback.desktop").unwrap();
        assert_eq!(entry.name, "My Tool");
        assert_eq!(entry.command, "tool --flag");
        assert!(entry.disabled);
        assert!(parse_autostart("Name=No section", "missing.desktop").is_none());
    }

    #[test]
    fn user_autostart_override_takes_precedence_over_system_file() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let base = std::env::temp_dir().join(format!(
            "taskotto-autostart-test-{}-{suffix}",
            std::process::id()
        ));
        let user = base.join("user");
        let system = base.join("system");
        fs::create_dir_all(&user).unwrap();
        fs::create_dir_all(&system).unwrap();
        fs::write(
            system.join("example.desktop"),
            "[Desktop Entry]\nName=System App\nExec=system-app\n",
        )
        .unwrap();
        fs::write(
            user.join("example.desktop"),
            "[Desktop Entry]\nName=Disabled App\nHidden=true\n",
        )
        .unwrap();
        let (entries, warnings) = load_autostart(&[(user, "User"), (system, "System")]);
        assert!(warnings.is_empty());
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Disabled App");
        assert!(entries[0].disabled);
        assert_eq!(entries[0].source, "User");
        fs::remove_dir_all(base).unwrap();
    }

    #[test]
    fn parses_systemd_user_service_rows_without_summary() {
        let rows = parse_service_units(
            "pipewire.service loaded active running Multimedia Service\n2 loaded units listed.\n● failed.service loaded failed failed A service",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].description, "Multimedia Service");
        assert_eq!(rows[1].active, "failed");
    }

    #[test]
    fn cpu_time_is_a_process_lifetime_not_periodic_rate() {
        assert_eq!(format_cpu_time(6150, 100.0), "00:01:01");
    }
}
