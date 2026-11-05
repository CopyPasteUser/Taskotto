use std::{env, process::ExitCode, thread, time::Duration};
use taskotto::{
    config::{Settings, Theme, configured_theme},
    desktop, format,
    monitor::Monitor,
    ui::TaskottoApp,
};

const HELP: &str = "Taskotto — Linux process manager and system monitor

USAGE
  taskotto [--theme NAME] [--refresh-ms N]
  taskotto --snapshot [--json]
  taskotto --install-desktop
  taskotto --help | --version

OPTIONS
  --theme NAME       system, windows98, xp-luna, windows7, windows10,
                     frutiger-aero, chromeos, macos
  --refresh-ms N     Sampling period in milliseconds (500–10000)
  --snapshot         Print a one-time unprivileged system snapshot and exit
  --install-desktop  Add a user-local application menu entry and icon
  --json             Emit machine-readable JSON with --snapshot

RICEBOWL
  Export RICEBOWL_RICE=<theme> when launching Taskotto to select a profile.
  An explicit --theme always takes precedence. No RiceBowl installation is needed.
";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("taskotto: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut snapshot = false;
    let mut json = false;
    let mut install_desktop = false;
    let mut forced_theme = None;
    let mut refresh_ms = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                print!("{HELP}");
                return Ok(());
            }
            "--version" | "-V" => {
                println!("Taskotto {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--snapshot" => snapshot = true,
            "--install-desktop" => install_desktop = true,
            "--json" => json = true,
            "--theme" => {
                let value = args.next().ok_or("--theme requires a name")?;
                forced_theme =
                    Some(Theme::parse(&value).ok_or_else(|| format!("unknown theme: {value}"))?);
            }
            "--refresh-ms" => {
                let raw = args.next().ok_or("--refresh-ms requires a number")?;
                let ms = raw.parse::<u64>().map_err(|_| "invalid refresh period")?;
                if !(500..=10_000).contains(&ms) {
                    return Err("refresh period must be between 500 and 10000 ms".to_string());
                }
                refresh_ms = Some(ms);
            }
            _ => return Err(format!("unknown argument: {argument}. Try --help")),
        }
    }
    if json && !snapshot {
        return Err("--json requires --snapshot".to_string());
    }
    if install_desktop {
        if snapshot || json {
            return Err("--install-desktop cannot be used with --snapshot or --json".to_owned());
        }
        let path = desktop::install_for_current_user().map_err(|err| err.to_string())?;
        println!("Taskotto application entry installed: {}", path.display());
        return Ok(());
    }
    if snapshot {
        let mut monitor = Monitor::default();
        monitor.sample().map_err(|err| err.to_string())?;
        thread::sleep(Duration::from_millis(250));
        let current = monitor.sample().map_err(|err| err.to_string())?;
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&current).map_err(|err| err.to_string())?
            );
        } else {
            println!(
                "CPU: {:.1}% ({} logical cores)",
                current.cpu_percent, current.logical_cpus
            );
            println!(
                "RAM: {} / {}",
                format::bytes(current.memory.used()),
                format::bytes(current.memory.total)
            );
            println!("Uptime: {}", format::duration(current.uptime_seconds));
            println!("Processes: {}", current.processes.len());
        }
        return Ok(());
    }

    let mut warning = None;
    let mut settings = match Settings::load() {
        Ok(settings) => settings,
        Err(err) => {
            warning = Some(format!(
                "Could not load preferences: {err}. Defaults are active."
            ));
            Settings::default()
        }
    };
    if let Some(theme) = forced_theme.or_else(configured_theme) {
        settings.launch_theme_original = Some(settings.theme);
        settings.theme = theme;
    }
    if let Some(ms) = refresh_ms {
        settings.refresh_ms = ms;
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([650.0, 420.0])
            .with_app_id("io.github.copypasteuser.taskotto"),
        ..Default::default()
    };
    eframe::run_native(
        "Taskotto",
        options,
        Box::new(move |context| Ok(Box::new(TaskottoApp::new(context, settings, warning)))),
    )
    .map_err(|err| format!("GUI startup failed: {err}"))
}
