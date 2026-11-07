use std::{env, path::PathBuf, process::ExitCode, thread, time::Duration};
use taskotto::{
    config::{Settings, Theme, configured_theme},
    desktop, format,
    monitor::Monitor,
    ricepack::RicePack,
    ui::TaskottoApp,
};

const HELP: &str = "Taskotto — Linux process manager and system monitor

USAGE
  taskotto [--theme NAME] [--ricepack PATH] [--refresh-ms N]
  taskotto --snapshot [--json]
  taskotto --check-ricepack PATH
  taskotto --export-ricepack PATH
  taskotto --install-desktop
  taskotto --uninstall-desktop
  taskotto --help | --version

OPTIONS
  --theme NAME       system, windows98, xp-luna, windows7, windows10,
                     frutiger-aero, chromeos, macos
  --refresh-ms N     Sampling period in milliseconds (500–10000)
  --ricepack PATH    Apply a session-only Taskotto RicePack TOML design
  --check-ricepack PATH   Validate a RicePack without opening a GUI
  --export-ricepack PATH  Save current design to a new RicePack TOML file
  --snapshot         Print a one-time unprivileged system snapshot and exit
  --install-desktop  Add a user-local application menu entry and icon
  --uninstall-desktop  Remove only Taskotto-managed user-local menu files
  --json             Emit machine-readable JSON with --snapshot

RICEBOWL
  Export RICEBOWL_RICE=<theme> when launching Taskotto to select a profile.
  RICEBOWL_TASKOTTO_RICEPACK=<path> supplies an optional Taskotto design.
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
    let mut uninstall_desktop = false;
    let mut forced_theme = None;
    let mut refresh_ms = None;
    let mut ricepack_path: Option<PathBuf> = None;
    let mut check_ricepack_path: Option<PathBuf> = None;
    let mut export_ricepack_path: Option<PathBuf> = None;
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
            "--ricepack" => {
                ricepack_path = Some(PathBuf::from(
                    args.next().ok_or("--ricepack requires a file path")?,
                ));
            }
            "--check-ricepack" => {
                check_ricepack_path = Some(PathBuf::from(
                    args.next().ok_or("--check-ricepack requires a file path")?,
                ));
            }
            "--export-ricepack" => {
                export_ricepack_path = Some(PathBuf::from(
                    args.next()
                        .ok_or("--export-ricepack requires a file path")?,
                ));
            }
            "--install-desktop" => install_desktop = true,
            "--uninstall-desktop" => uninstall_desktop = true,
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
    if let Some(path) = export_ricepack_path {
        if snapshot
            || json
            || install_desktop
            || uninstall_desktop
            || forced_theme.is_some()
            || ricepack_path.is_some()
            || check_ricepack_path.is_some()
            || refresh_ms.is_some()
        {
            return Err("--export-ricepack must be used on its own".to_owned());
        }
        let settings =
            Settings::load().map_err(|err| format!("cannot load saved preferences: {err}"))?;
        let pack = RicePack::from_settings(&settings);
        pack.write_new(&path)
            .map_err(|err| format!("cannot export RicePack {}: {err}", path.display()))?;
        println!("Exported RicePack: {} ({})", pack.name, path.display());
        return Ok(());
    }
    if let Some(path) = check_ricepack_path {
        if snapshot
            || json
            || install_desktop
            || uninstall_desktop
            || forced_theme.is_some()
            || ricepack_path.is_some()
            || refresh_ms.is_some()
        {
            return Err("--check-ricepack must be used on its own".to_owned());
        }
        let pack = RicePack::load(&path)
            .map_err(|err| format!("invalid RicePack {}: {err}", path.display()))?;
        println!(
            "Valid RicePack: {} (format {}, base {})",
            pack.name,
            pack.format_version,
            pack.base_theme.label()
        );
        if let Some(layout) = &pack.layout {
            let normal = layout
                .pages
                .iter()
                .map(|page| page.widgets.len())
                .sum::<usize>();
            let compact = layout
                .pages
                .iter()
                .map(|page| page.compact_widgets.len())
                .sum::<usize>();
            println!(
                "Validated layout: {} pages, {} normal widgets, {} compact alternatives; compact breakpoint {} logical px; no scripts or additional permissions",
                layout.pages.len(),
                normal,
                compact,
                layout.compact_below.unwrap_or(680)
            );
        }
        return Ok(());
    }
    if ricepack_path.is_some() && (snapshot || install_desktop || uninstall_desktop) {
        return Err("--ricepack requires GUI mode".to_owned());
    }
    if install_desktop || uninstall_desktop {
        if snapshot
            || json
            || install_desktop == uninstall_desktop
            || forced_theme.is_some()
            || refresh_ms.is_some()
        {
            return Err("--install-desktop and --uninstall-desktop must be used alone".to_owned());
        }
        if uninstall_desktop {
            let (launcher, icon) =
                desktop::uninstall_for_current_user().map_err(|err| err.to_string())?;
            println!("Taskotto managed menu files removed: launcher={launcher}, icon={icon}");
            return Ok(());
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
    let original_theme = settings.theme;
    let pack_path = ricepack_path
        .or_else(|| env::var_os("RICEBOWL_TASKOTTO_RICEPACK").map(PathBuf::from))
        .or_else(|| env::var_os("TASKOTTO_RICEPACK").map(PathBuf::from));
    let mut active_pack = None;
    if let Some(path) = pack_path {
        let pack = RicePack::load(&path)
            .map_err(|err| format!("invalid RicePack {}: {err}", path.display()))?;
        pack.apply(&mut settings);
        active_pack = Some((path, pack));
    }
    let selected_theme = forced_theme.or_else(|| {
        if settings.launch_ricepack_name.is_some() {
            None // A pack's base theme takes precedence over the RiceBowl rice.
        } else {
            configured_theme()
        }
    });
    if let Some(theme) = selected_theme {
        settings.launch_theme_original = Some(original_theme);
        settings.theme = theme;
    }
    if let Some(ms) = refresh_ms {
        settings.refresh_ms = ms;
    }
    let classic98 = settings.theme == Theme::Windows98;
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            // Request native decorations for every theme. RiceBowl/labwc (or
            // the active window manager) owns the frame and window buttons.
            .with_decorations(true)
            .with_inner_size(if classic98 {
                [650.0, 450.0]
            } else {
                [1100.0, 720.0]
            })
            .with_min_inner_size(if classic98 {
                [490.0, 340.0]
            } else {
                [650.0, 420.0]
            })
            .with_app_id("io.github.copypasteuser.taskotto"),
        ..Default::default()
    };
    eframe::run_native(
        "Taskotto",
        options,
        Box::new(move |context| {
            let app = TaskottoApp::new(context, settings, warning);
            let app = if let Some((path, pack)) = active_pack {
                app.watch_ricepack(path, pack, forced_theme)
            } else {
                app
            };
            Ok(Box::new(app))
        }),
    )
    .map_err(|err| format!("GUI startup failed: {err}"))
}
