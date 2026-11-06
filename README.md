<div align="center">

# 🍚 Taskotto

**The Linux task manager, served your way.**

A customizable process manager and system monitor written in **Rust**. Runs alone or alongside [RiceBowl](https://github.com/CopyPasteUser/Ricebowl) on **X11 and Wayland**.

![License](https://img.shields.io/badge/license-MIT-blue) ![Rust](https://img.shields.io/badge/language-Rust-orange) ![Status](https://img.shields.io/badge/status-1.0%20RC13-yellow)

</div>

> **Status:** `1.0.0-rc.13` — seven dedicated interfaces: Frutiger Aero, ChromeOS, Windows 98, XP Luna, Windows 7, Windows 10 and macOS Activity Monitor. **All eight profiles leave the window frame, title and window buttons to RiceBowl/the active window manager**; Taskotto draws only its content, menus and functional toolbars. Native GUI and resource-use acceptance are still outstanding. See [roadmap](ROADMAP.md) and [QA checklist](QA.md). This is **not yet stable 1.0**.

## Screenshots

| Frutiger Aero · Live processes | ChromeOS · Task manager |
| :---: | :---: |
| [![Frutiger Aero Taskotto running real Linux processes](assets/screenshots/taskotto-frutiger-aero-processes.png)](assets/screenshots/taskotto-frutiger-aero-processes.png) | [![ChromeOS-inspired task manager with real Linux processes](assets/screenshots/taskotto-chromeos-tasks.png)](assets/screenshots/taskotto-chromeos-tasks.png) |
| **Windows 98 · Close Program** | **XP Luna · Classic performance** |
| [![Windows 98 Close Program with real Linux processes](assets/screenshots/taskotto-win98-close-program.png)](assets/screenshots/taskotto-win98-close-program.png) | [![XP Luna Taskotto performance](assets/screenshots/taskotto-xp-luna-performance.png)](assets/screenshots/taskotto-xp-luna-performance.png) |
| **Windows 7 · Aero-era report** | **Windows 10 · Modern processes** |
| [![Windows 7 Taskotto process report](assets/screenshots/taskotto-win7-processes.png)](assets/screenshots/taskotto-win7-processes.png) | [![Windows 10 Taskotto processes](assets/screenshots/taskotto-win10-processes.png)](assets/screenshots/taskotto-win10-processes.png) |
| **macOS · Activity Monitor (CPU)** | **Frutiger Aero · Live performance** |
| [![macOS-inspired Activity Monitor CPU process list](assets/screenshots/taskotto-macos-cpu.png)](assets/screenshots/taskotto-macos-cpu.png) | [![Frutiger Aero CPU and memory performance dashboard](assets/screenshots/taskotto-frutiger-aero-performance.png)](assets/screenshots/taskotto-frutiger-aero-performance.png) |

<details>
<summary>More genuine renders: Aero, ChromeOS, Windows 98, macOS, Windows 7, Windows 10 and XP Luna</summary>

![ChromeOS-inspired supplementary performance view](assets/screenshots/taskotto-chromeos-performance.png)

![Windows 98 advanced process report](assets/screenshots/taskotto-win98-processes.png)

![Frutiger Aero network receive/send history](assets/screenshots/taskotto-frutiger-aero-network.png)

![Frutiger Aero alternative navigation sidebar](assets/screenshots/taskotto-frutiger-aero-sidebar.png)

![Windows 98 New Task user-command dialog](assets/screenshots/taskotto-win98-new-task.png)

![Windows 98 classic CPU and memory activity](assets/screenshots/taskotto-win98-performance.png)

![macOS-inspired Activity Monitor Memory view](assets/screenshots/taskotto-macos-memory.png)

![macOS-inspired Activity Monitor Energy view with honest Linux CPU proxy](assets/screenshots/taskotto-macos-energy.png)

![macOS-inspired Activity Monitor Disk activity](assets/screenshots/taskotto-macos-disk.png)

![macOS-inspired Activity Monitor aggregate Network activity](assets/screenshots/taskotto-macos-network.png)

![macOS-inspired Activity Monitor selected process information](assets/screenshots/taskotto-macos-info.png)

![Windows 7 historical CPU and memory usage layout](assets/screenshots/taskotto-win7-performance.png)

![Windows 7 Networking with measured Linux receive/send rates](assets/screenshots/taskotto-win7-networking.png)

![Windows 10-inspired performance page](assets/screenshots/taskotto-win10-performance.png)

![Optional Windows 10-inspired vertical navigation](assets/screenshots/taskotto-win10-vertical.png)

![Windows 10-inspired Linux startup entries](assets/screenshots/taskotto-win10-startup.png)

![XP Luna-inspired applications-style page](assets/screenshots/taskotto-xp-luna-applications.png)

![XP Luna-inspired New Task dialog](assets/screenshots/taskotto-xp-luna-new-task.png)

</details>

*All images are refreshed rc.13 Taskotto egui renders using live Linux `/proc`; the capture environment exposes few processes. These client-only renders intentionally omit RiceBowl's native window frame. [All 37 captures and reproduction details](SCREENSHOTS.md) · [Fidelity boundaries](RICE_FIDELITY.md). Native X11/Wayland verification remains pending.*

## What works

- **Processes:** searchable live list, CPU/RAM/I/O/PID/name sort, tree, user filter, process start/runtime and per-process I/O rates.
- **Safe actions:** terminate, force kill, pause and resume with confirmation and PID-reuse protection via Linux `pidfd`.
- **Performance:** CPU/core load, RAM/swap, network RX/TX and disk read/write rates, bounded comparative rolling graphs.
- **Riceable:** 8 profiles; Frutiger Aero (original luminous Y2K task manager), ChromeOS (compact task table), Windows 98 **Close Program**, XP Luna (5 tabs), Windows 7 (6 tabs), Windows 10 (7 tabs), and macOS Activity Monitor (5 views). Custom navigation, colors, fonts, density and persistent TOML preferences remain supported.
- **One window decoration:** Taskotto requests WM-managed window decorations for every rice. XP Luna, Windows 7 and Windows 10 no longer draw fake title strips or app captions; their real menus and tabs remain intact. The macOS toolbar no longer repeats the native app caption. Native border colors/buttons are configured in RiceBowl/the compositor, not Taskotto.
- **Frutiger Aero:** original vector aqua/glass masthead, bubble meters, seven navigable destinations, searchable/sortable process garden, selected-PID actions and four live performance charts. Both pill tabs and a customizable sidebar use the existing sampler; RiceBowl/the compositor owns native decorations.
- **Classic 98:** compact, gray 3D controls and a white running-program list showing actual *user-owned Linux processes*. End Task uses the same safe confirmation flow; New Task, Refresh, Close and advanced Processes/Performance/Network/Users/Options all work. The compositor owns external window decoration; Taskotto does not draw a second title bar.
- **macOS-style Activity Monitor:** searchable, sortable process reports with alternating rows, context actions, process Info and a live bottom history panel. Energy shows CPU activity as a clearly labeled proxy, not invented watts; Network reports genuine machine-wide traffic without false per-PID attribution.
- **ChromeOS-style Task manager:** compact sortable Task/Memory/CPU/Process ID table, responsive optional *disk* I/O and user columns, always-on search, all/my-tasks filter, process detail actions and confirmation-protected End process. The menu also exposes real Linux performance, network, startup and service tools. ChromeOS per-process network traffic is **not** fabricated from system-wide counters; RiceBowl handles window decoration.
- **Desktop features:** user-local XDG autostart list (read-only), live user systemd services (read-only if available), process-lifetime CPU accounting, and an XP-style **New Task** command dialog running with ordinary user permissions.
- **Independent:** no root service, no resident daemon, no RiceBowl or desktop-shell dependency.
- **Headless:** `--snapshot --json` is available without a graphical session.

<details>
<summary>Available Rice themes</summary>

Taskotto dark, Classic 98, XP Luna, Windows 7, Windows 10, Frutiger Aero, ChromeOS and macOS-inspired. All assets are original; no proprietary OS icons are included.

</details>

## Build and run

Requires Linux, Rust 1.88+ and a working X11 or Wayland session with OpenGL support.

```bash
git clone https://github.com/CopyPasteUser/Taskotto.git
cd Taskotto
cargo run --release
```

The first compilation may take a while because of the GUI toolkit. Later starts do not recompile.

### Install for your user

```bash
cargo install --path . --locked
taskotto --install-desktop
```

The installer creates a user-local application entry with the executable's absolute path, so the menu does not depend on your graphical session's `PATH`. Then launch **Taskotto** from the applications menu.
The launcher and icon are replaced atomically on reinstall, so interrupted writes do not truncate the previous files.

## Controls

| Action | Shortcut / command |
| --- | --- |
| Search processes | `Ctrl+F` |
| Refresh immediately | `F5` |
| Clear search / dismiss confirmation | `Esc` |
| Show system snapshot | `taskotto --snapshot` |
| JSON snapshot | `taskotto --snapshot --json` |
| Choose a rice | `taskotto --theme frutiger-aero` (or `chromeos`, `windows98`, `xp-luna`, `windows7`, `windows10`, `macos`) |
| Reduce polling | `taskotto --refresh-ms 5000` |

Select a process to see **Details** and signal actions. A confirmation dialog appears before any signal. Taskotto uses only the permissions of the current user and refuses signals to PID 1 and itself.

### RiceBowl integration

RiceBowl can pass its current rice when launching Taskotto:

```bash
RICEBOWL_RICE=windows7 taskotto
```

The `--theme` flag overrides this environment variable. Launcher-selected themes are **session-only**: other preference changes do not overwrite the saved theme. Preferences live at `~/.config/taskotto/config.toml` or the XDG config location. No external service or theme assets are needed.

### Custom design tokens

The Preferences page offers color, typography, density, spacing, corner and navigation controls. RiceBowl or another theme generator can also write these settings as TOML (other preferences can be omitted):

```toml
theme = "windows7"
layout_mode = "top-tabs" # auto, sidebar, top-tabs
density = "compact"
accent_hex = "#2179BC"
surface_hex = "#E8F2FA"
text_hex = "#26394F"
font_scale = 1.05
ui_spacing = 7.0
corner_radius = 5
show_io_column = true
show_user_column = true
table_row_height = 30.0
```

Unreadable text colors automatically fall back to a contrasting color. Invalid settings are reported; if a malformed config is replaced, its original contents are backed up in the same directory.

## Development

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --release
```

- [Roadmap to 1.0](ROADMAP.md) — milestones, acceptance gates and deferred enhancements.
- [Hardware QA checklist](QA.md) — X11/Wayland, process safety and measured resource use.
- [Performance measurements](PERFORMANCE.md) — reproducible headless baseline and GUI acceptance limits.
- [Screenshot provenance](SCREENSHOTS.md) — reproducible live-data renders and their limitations.
- [Frutiger Aero, ChromeOS, Windows 98, XP Luna, Windows 7, Windows 10 and macOS rice details](RICE_FIDELITY.md) — layout, interaction and fidelity boundaries.
- [Release candidate notes](RELEASE_NOTES.md) — implemented scope, verified checks and known limitations.
- [Contributing](CONTRIBUTING.md) · [Security policy](SECURITY.md) · [Changelog](CHANGELOG.md)
- [RiceField](https://github.com/CopyPasteUser/Ricefield) — matching standalone Rust file manager.

**Limitations:** Linux only. The first sample has no historical CPU/I/O baseline. Per-process I/O depends on `/proc/<PID>/io` permissions; restricted entries report unavailable. Disk rates are block-device totals, and network rates sum interfaces excluding loopback (virtual interfaces can double-count traffic). Kernel 5.3+ is required for safe process signals; monitoring works on older kernels. GPU monitoring and container-specific views are future extensions.

MIT licensed. © 2026 CopyPasteUser.
