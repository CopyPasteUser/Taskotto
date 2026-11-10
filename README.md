<div align="center">

# 🍚 Taskotto

**A Linux task manager that fits your desktop.**

Built in **Rust** for X11 and Wayland. Seven OS-inspired interfaces, custom RicePacks, and optional [RiceBowl](https://github.com/CopyPasteUser/Ricebowl) integration. No root daemon or desktop-shell dependency.

[**Download v1.0.0**](https://github.com/CopyPasteUser/Taskotto/releases/tag/v1.0.0) · [Screenshots](SCREENSHOT_GALLERY.md) · [RicePack guide](RICEPACK_FORMAT.md)

</div>

| Frutiger Aero | Windows 10 |
| :---: | :---: |
| [![Frutiger Aero process view](assets/screenshots/taskotto-frutiger-aero-processes.png)](assets/screenshots/taskotto-frutiger-aero-processes.png) | [![Windows 10 process view](assets/screenshots/taskotto-win10-processes.png)](assets/screenshots/taskotto-win10-processes.png) |

[Explore all 52 real-data captures](SCREENSHOT_GALLERY.md), including Windows 98, XP Luna, Windows 7, ChromeOS, macOS and custom dashboards.

## What you get

- **Manage processes:** CPU, RAM and I/O, search, sorting, tree view and safe process actions with PID-reuse protection.
- **Monitor Linux:** per-core CPU, memory, disk, network, pressure (PSI), and available GPU/battery sensors.
- **Inspect details:** cgroups, on-demand PSS, bounded history and CSV/JSON exports.
- **Make it yours:** seven built-in looks, TOML RicePacks, layout editor and live theme reload.
- **Keep it efficient:** adjustable refresh, optional low-overhead mode, bounded queues and no background service.

**Status:** `v1.0.0` is the stable release (9 November 2026). The `main` branch includes newer maintenance and performance improvements. Cross-desktop hardware checks and measured GUI memory benchmarks remain [documented follow-ups](QA.md).

## Install

Requires Linux, **Rust 1.88+**, and an X11 or Wayland session with OpenGL. To install the latest source version:

```bash
git clone https://github.com/CopyPasteUser/Taskotto.git
cd Taskotto
bash install.sh
```

`install.sh` installs Taskotto for your user and adds its application-menu entry and icon. To update later, run from your Taskotto directory:

```bash
git pull --ff-only
bash install.sh
```

## Run and customize

```bash
taskotto --theme frutiger-aero
taskotto --snapshot --json
taskotto --check-ricepack examples/ricepacks/sea-glass.toml
taskotto --ricepack examples/ricepacks/sea-glass.toml
```

Run directly from source with `cargo run --release`, or select a RiceBowl-inspired style using `RICEBOWL_RICE=windows7 taskotto`. RicePacks are data-only and session overrides do not overwrite your saved theme. Process actions require confirmation and your normal Linux permissions.

## Develop

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
```

[Performance & benchmarks](PERFORMANCE.md) · [Hardware QA](QA.md) · [Roadmap](ROADMAP.md) · [Release notes](RELEASE_1_0_0.md) · [Changelog](CHANGELOG.md) · [MIT License](LICENSE)
