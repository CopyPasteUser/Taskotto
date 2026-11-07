<div align="center">

# 🍚 Taskotto

**The Linux task manager, served your way.**

A customizable process manager and system monitor written in **Rust**. Runs alone or alongside [RiceBowl](https://github.com/CopyPasteUser/Ricebowl) on **X11 and Wayland**.

![License](https://img.shields.io/badge/license-MIT-blue) ![Rust](https://img.shields.io/badge/language-Rust-orange) ![Status](https://img.shields.io/badge/status-1.0%20RC52-yellow)

</div>

> **Status:** `1.0.0-rc.52` — seven dedicated interfaces: Frutiger Aero, ChromeOS, Windows 98, XP Luna, Windows 7, Windows 10 and macOS Activity Monitor, plus optional data-defined client screens. **All eight profiles leave the window frame, title and window buttons to RiceBowl/the active window manager**; Taskotto draws only its content, menus and functional toolbars. Native GUI and resource-use acceptance are still outstanding. See [1.0 roadmap](ROADMAP.md), [future development roadmap](EXPANSION_ROADMAP.md) and [QA checklist](QA.md). This is **not yet stable 1.0**.

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
<summary><strong>All screenshots (51)</strong> — expand to view every rice and every captured screen</summary>

### Frutiger Aero

| Frutiger Aero | Frutiger Aero |
| :---: | :---: |
| **Processes** | **Performance** |
| [![Frutiger Aero: Processes](assets/screenshots/taskotto-frutiger-aero-processes.png)](assets/screenshots/taskotto-frutiger-aero-processes.png) | [![Frutiger Aero: Performance](assets/screenshots/taskotto-frutiger-aero-performance.png)](assets/screenshots/taskotto-frutiger-aero-performance.png) |
| **Network** | **Sidebar layout** |
| [![Frutiger Aero: Network](assets/screenshots/taskotto-frutiger-aero-network.png)](assets/screenshots/taskotto-frutiger-aero-network.png) | [![Frutiger Aero: Sidebar layout](assets/screenshots/taskotto-frutiger-aero-sidebar.png)](assets/screenshots/taskotto-frutiger-aero-sidebar.png) |
| **RicePack live preview and editor** | **Aero Lagoon · custom component tokens** |
| [![Frutiger Aero: RicePack preview with reversible appearance controls](assets/screenshots/taskotto-frutiger-aero-ricepack-editor.png)](assets/screenshots/taskotto-frutiger-aero-ricepack-editor.png) | [![Frutiger Aero: Aero Lagoon RicePack applies control, hover, border, table and selection colors to custom widgets](assets/screenshots/taskotto-frutiger-aero-lagoon-ricepack.png)](assets/screenshots/taskotto-frutiger-aero-lagoon-ricepack.png) |
| **Screen layout designer · typed widget editor** | |
| [![Frutiger Aero: functional screen layout designer with widget labels, process data bindings and validated session-only application](assets/screenshots/taskotto-frutiger-aero-layout-editor.png)](assets/screenshots/taskotto-frutiger-aero-layout-editor.png) | |
| **Aero Lagoon · custom performance charts** | |
| [![Frutiger Aero: Aero Lagoon RicePack recolors live performance charts](assets/screenshots/taskotto-frutiger-aero-lagoon-performance.png)](assets/screenshots/taskotto-frutiger-aero-lagoon-performance.png) | |

### ChromeOS

| ChromeOS | ChromeOS |
| :---: | :---: |
| **Task list** | **Performance** |
| [![ChromeOS: Task list](assets/screenshots/taskotto-chromeos-tasks.png)](assets/screenshots/taskotto-chromeos-tasks.png) | [![ChromeOS: Performance](assets/screenshots/taskotto-chromeos-performance.png)](assets/screenshots/taskotto-chromeos-performance.png) |
| **Nocturne RicePack · dark task table** | |
| [![ChromeOS: Nocturne RicePack with dark process table, custom selection and semantic borders](assets/screenshots/taskotto-chromeos-nocturne-ricepack.png)](assets/screenshots/taskotto-chromeos-nocturne-ricepack.png) | |

### Windows 98

| Windows 98 | Windows 98 |
| :---: | :---: |
| **Close Program** | **Processes** |
| [![Windows 98: Close Program](assets/screenshots/taskotto-win98-close-program.png)](assets/screenshots/taskotto-win98-close-program.png) | [![Windows 98: Processes](assets/screenshots/taskotto-win98-processes.png)](assets/screenshots/taskotto-win98-processes.png) |
| **Performance** | **New Task** |
| [![Windows 98: Performance](assets/screenshots/taskotto-win98-performance.png)](assets/screenshots/taskotto-win98-performance.png) | [![Windows 98: New Task](assets/screenshots/taskotto-win98-new-task.png)](assets/screenshots/taskotto-win98-new-task.png) |
| **Violet RicePack: 3D controls and dark list** | |
| [![Windows 98: Violet RicePack with a dark Close Program list, violet selected process and matching beveled buttons](assets/screenshots/taskotto-win98-violet-ricepack.png)](assets/screenshots/taskotto-win98-violet-ricepack.png) | |

### Windows XP Luna

| Windows XP Luna | Windows XP Luna |
| :---: | :---: |
| **Applications** | **Processes** |
| [![Windows XP Luna: Applications](assets/screenshots/taskotto-xp-luna-applications.png)](assets/screenshots/taskotto-xp-luna-applications.png) | [![Windows XP Luna: Processes](assets/screenshots/taskotto-xp-luna-processes.png)](assets/screenshots/taskotto-xp-luna-processes.png) |
| **Performance** | **Networking** |
| [![Windows XP Luna: Performance](assets/screenshots/taskotto-xp-luna-performance.png)](assets/screenshots/taskotto-xp-luna-performance.png) | [![Windows XP Luna: Networking](assets/screenshots/taskotto-xp-luna-networking.png)](assets/screenshots/taskotto-xp-luna-networking.png) |
| **Details** | **New Task** |
| [![Windows XP Luna: Details](assets/screenshots/taskotto-xp-luna-details.png)](assets/screenshots/taskotto-xp-luna-details.png) | [![Windows XP Luna: New Task](assets/screenshots/taskotto-xp-luna-new-task.png)](assets/screenshots/taskotto-xp-luna-new-task.png) |
| **Midnight RicePack: custom report** | |
| [![XP Luna: Midnight RicePack with a dark process table](assets/screenshots/taskotto-xp-midnight-ricepack.png)](assets/screenshots/taskotto-xp-midnight-ricepack.png) | |

### Windows 7

| Windows 7 | Windows 7 |
| :---: | :---: |
| **Applications** | **Processes** |
| [![Windows 7: Applications](assets/screenshots/taskotto-win7-applications.png)](assets/screenshots/taskotto-win7-applications.png) | [![Windows 7: Processes](assets/screenshots/taskotto-win7-processes.png)](assets/screenshots/taskotto-win7-processes.png) |
| **Services** | **Performance** |
| [![Windows 7: Services](assets/screenshots/taskotto-win7-services.png)](assets/screenshots/taskotto-win7-services.png) | [![Windows 7: Performance](assets/screenshots/taskotto-win7-performance.png)](assets/screenshots/taskotto-win7-performance.png) |
| **Networking** | **Users** |
| [![Windows 7: Networking](assets/screenshots/taskotto-win7-networking.png)](assets/screenshots/taskotto-win7-networking.png) | [![Windows 7: Users](assets/screenshots/taskotto-win7-users.png)](assets/screenshots/taskotto-win7-users.png) |
| **Process properties** | **Dusk RicePack: semantic colors** |
| [![Windows 7: Process properties](assets/screenshots/taskotto-win7-properties.png)](assets/screenshots/taskotto-win7-properties.png) | [![Windows 7: Dusk RicePack with custom controls and dark task table](assets/screenshots/taskotto-win7-dusk-ricepack.png)](assets/screenshots/taskotto-win7-dusk-ricepack.png) |

### Windows 10

| Windows 10 | Windows 10 |
| :---: | :---: |
| **Processes** | **Performance** |
| [![Windows 10: Processes](assets/screenshots/taskotto-win10-processes.png)](assets/screenshots/taskotto-win10-processes.png) | [![Windows 10: Performance](assets/screenshots/taskotto-win10-performance.png)](assets/screenshots/taskotto-win10-performance.png) |
| **App history** | **Startup** |
| [![Windows 10: App history](assets/screenshots/taskotto-win10-app-history.png)](assets/screenshots/taskotto-win10-app-history.png) | [![Windows 10: Startup](assets/screenshots/taskotto-win10-startup.png)](assets/screenshots/taskotto-win10-startup.png) |
| **Users** | **Details** |
| [![Windows 10: Users](assets/screenshots/taskotto-win10-users.png)](assets/screenshots/taskotto-win10-users.png) | [![Windows 10: Details](assets/screenshots/taskotto-win10-details.png)](assets/screenshots/taskotto-win10-details.png) |
| **Services** | **Vertical navigation** |
| [![Windows 10: Services](assets/screenshots/taskotto-win10-services.png)](assets/screenshots/taskotto-win10-services.png) | [![Windows 10: Vertical navigation](assets/screenshots/taskotto-win10-vertical.png)](assets/screenshots/taskotto-win10-vertical.png) |
| **Indigo RicePack: custom heatmap** | |
| [![Windows 10: Indigo RicePack dark process table and accurate CPU/memory heatmap](assets/screenshots/taskotto-win10-indigo-ricepack.png)](assets/screenshots/taskotto-win10-indigo-ricepack.png) | |

### macOS Activity Monitor

| macOS Activity Monitor | macOS Activity Monitor |
| :---: | :---: |
| **CPU** | **Memory** |
| [![macOS Activity Monitor: CPU](assets/screenshots/taskotto-macos-cpu.png)](assets/screenshots/taskotto-macos-cpu.png) | [![macOS Activity Monitor: Memory](assets/screenshots/taskotto-macos-memory.png)](assets/screenshots/taskotto-macos-memory.png) |
| **Energy** | **Disk** |
| [![macOS Activity Monitor: Energy](assets/screenshots/taskotto-macos-energy.png)](assets/screenshots/taskotto-macos-energy.png) | [![macOS Activity Monitor: Disk](assets/screenshots/taskotto-macos-disk.png)](assets/screenshots/taskotto-macos-disk.png) |
| **Network** | **Process info** |
| [![macOS Activity Monitor: Network](assets/screenshots/taskotto-macos-network.png)](assets/screenshots/taskotto-macos-network.png) | [![macOS Activity Monitor: Process info](assets/screenshots/taskotto-macos-info.png)](assets/screenshots/taskotto-macos-info.png) |
| **Nightshift RicePack · custom process table** | |
| [![macOS Activity Monitor: Nightshift custom RicePack recolors process table and selection](assets/screenshots/taskotto-macos-nightshift-ricepack.png)](assets/screenshots/taskotto-macos-nightshift-ricepack.png) | |

### Declarative screens (original TOML-generated layout)

| **Diagnostic Studio · Overview** | **Diagnostic Studio · I/O and network** |
| [![Diagnostic Studio RicePack: real Linux CPU metrics, history and process table from a bounded data-only layout](assets/screenshots/taskotto-diagnostic-studio-ricepack.png)](assets/screenshots/taskotto-diagnostic-studio-ricepack.png) | [![Diagnostic Studio RicePack: network and disk metrics, history and Linux process list in a second generated screen](assets/screenshots/taskotto-diagnostic-studio-traffic.png)](assets/screenshots/taskotto-diagnostic-studio-traffic.png) |
| **Diagnostic Studio · compact adaptive layout** | |
| [![Diagnostic Studio RicePack: 720px narrow window activates CPU/RAM widget fallback, compact process table and status bar](assets/screenshots/taskotto-diagnostic-studio-compact.png)](assets/screenshots/taskotto-diagnostic-studio-compact.png) | |

### System monitor · process follow

| **Process inspector · session CPU/RSS history** | |
| :---: | :---: |
| [![Taskotto system process inspector follows a real Linux sleep PID, with bounded CPU and RSS sample graphs](assets/screenshots/taskotto-system-process-follow.png)](assets/screenshots/taskotto-system-process-follow.png) | |

</details>

*All 51 images above are headless Taskotto egui renders using live Linux `/proc`; the capture environment exposes few processes. The editor, Aero Lagoon, Nightshift, Nocturne, Dusk, Midnight, Indigo, Violet and Diagnostic Studio captures are actual Taskotto renders (the designer is an in-session draft; the others use bundled RicePacks). These client-only renders intentionally omit RiceBowl's native window frame. [Capture provenance and reproduction](SCREENSHOTS.md) · [Fidelity boundaries](RICE_FIDELITY.md). Native X11/Wayland verification remains pending.*

## What works

- **Processes:** searchable live list, CPU/RAM/I/O/PID/name sort, session-only sort freeze (`Ctrl+Shift+Space`), tree, user filter, process start/runtime and per-process I/O rates. Opt-in **Follow selected process** charts record up to 120 CPU/RSS samples without PID-reuse mistakes; switch between the last **30 seconds, 2 minutes or all observations** using real GUI-receipt timestamps. At narrow widths charts stack vertically; no extra daemon or automatically saved process log. An explicit **Save CSV / JSON** action exports the currently followed PID, start ticks and samples to a new owner-only file (no command/name, no overwrite). Open generic Details or Preferences → Process follow. Details/Properties/Info summarize observed descendant CPU and **summed RSS** (shared pages may overlap), plus read-only leader Nice/CPU-affinity metadata (other threads may differ). An explicit **Measure PSS now** action in generic Details/Preferences calculates proportional memory for at most 128 observed tree processes on a background worker; missing procfs permissions show an incomplete report, never fake zero.
- **Safe actions:** terminate, force kill, pause and resume with confirmation and PID-reuse protection via Linux `pidfd`.
- **Performance:** CPU/core load, RAM/swap, network RX/TX and disk read/write rates, bounded comparative rolling graphs. Linux **PSI** shows measured CPU, memory and I/O task-stall percentages over 10/60/300 seconds, bounded PSI trend charts and conservative, evidence-labeled investigation hints in System Performance and all rice Preferences. Thresholds are display heuristics, not kernel limits or proven causes.
- **Optional low-overhead mode:** Preferences → Monitoring can use a minimum 3-second sampling interval and no style animations on resource-sensitive desktops. All functions remain available, the original configured interval is restored when disabled, and no unmeasured power-saving claims are made.
- **Session export:** In Preferences, explicitly export up to 120 Linux sampler-visible system samples as a new `.csv` or `.json` file. Exports include GUI-receipt Unix timestamps, nominal interval and units; container visibility may vary. No file is written by default; no existing destination is overwritten, and no process names, commands or PIDs are included. Missing PSI readings remain missing.
- **Cgroup inspection:** The opt-in, read-only selected-process inspector displays cgroup v2 CPU/memory/I/O counters and genuine cgroup-local **PSI** (including meaningful cgroup CPU FULL where available), strictly separated from host-level pressure. It also highlights likely systemd/Docker/Podman/Flatpak naming segments; these are *unverified naming hints*, not confirmed runtime identities. No privileged service or process action is needed.
- **GPU sensors (P4 foundation):** lightweight DRM/sysfs discovery of AMD, Intel and NVIDIA devices; real **AMDGPU utilization and VRAM counters**, and optional available hardware temperature, power and core-clock readings. Visible in the System Performance view and every rice's Preferences. Missing driver capabilities show **Unavailable**, not fabricated zeroes; full Intel/NVIDIA utilization backends and physical GPU acceptance remain open.
- **Battery and power:** Linux `power_supply` telemetry for available laptop batteries and external adapters: charging state, percentage, actual stored energy (Wh) and reported W. Wh and charge in µAh are never confused; no imaginary runtime estimate. Displayed alongside GPU sensors in System Performance and all rice Preferences.
- **Cgroup inspector:** explicit read-only Linux **cgroup v2** inspection for the selected process in generic Details or Preferences: membership path, cumulative group CPU use/throttling, memory usage/limit and read/write I/O where exposed. Values belong to the cgroup, **not the selected PID alone**; stale identities, missing controllers and migrations are checked.
- **Riceable:** 8 profiles; Frutiger Aero (original luminous Y2K task manager), ChromeOS (compact task table), Windows 98 **Close Program**, XP Luna (5 tabs), Windows 7 (6 tabs), Windows 10 (7 tabs), and macOS Activity Monitor (5 views). Custom navigation, colors, fonts, density and persistent TOML preferences remain supported.
- **RicePacks (v1 + declarative screens):** Export your current appearance with `--export-ricepack`, modify its TOML, validate and apply it using `--check-ricepack` / `--ricepack` without rebuilding Rust. Active RicePack sessions automatically reload valid file edits about every 2 seconds. Preferences → RicePack library discovers local designs, previews them with a one-click revert, imports them without overwriting, and exports the current appearance to a new file. The **Screen layout designer** builds and reorders bounded, data-bound screens from seven safe widget presets, validates a detached draft and applies it temporarily; optional compact compositions adapt on resize. Keyboard search (`Ctrl+F`) works with or without a generated toolbar; CLI checks describe compatibility without silently upgrading schemas. See [RicePack format](RICEPACK_FORMAT.md) and [Diagnostic Studio](examples/ricepacks/diagnostic-studio.toml).
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
To repair a stale launcher after moving Taskotto, run `taskotto --install-desktop` again from the current binary. To remove only Taskotto's user-local menu entry and icon, use `taskotto --uninstall-desktop`. Removal refuses customized files and symlinks; it leaves your binary, preferences and unrelated desktop entries untouched.

## Controls

| Action | Shortcut / command |
| --- | --- |
| Search processes | `Ctrl+F` |
| Refresh immediately | `F5` |
| Clear search / dismiss confirmation | `Esc` |
| Show system snapshot | `taskotto --snapshot` |
| JSON snapshot | `taskotto --snapshot --json` |
| Choose a rice | `taskotto --theme frutiger-aero` (or `chromeos`, `windows98`, `xp-luna`, `windows7`, `windows10`, `macos`) |
| Validate a RicePack | `taskotto --check-ricepack examples/ricepacks/sea-glass.toml` |
| Export saved appearance | `taskotto --export-ricepack my-rice.toml` (new file only) |
| Apply a RicePack | `taskotto --ricepack examples/ricepacks/sea-glass.toml` |
| Reduce polling | `taskotto --refresh-ms 5000` |

Select a process to see **Details** and signal actions. A confirmation dialog appears before any signal. Taskotto uses only the permissions of the current user and refuses signals to PID 1 and itself.

### RiceBowl integration

RiceBowl can pass its current rice when launching Taskotto:

```bash
RICEBOWL_RICE=windows7 taskotto
```

The `--theme` flag overrides this environment variable. Launcher-selected themes are **session-only**: other preference changes do not overwrite the saved theme. Preferences live at `~/.config/taskotto/config.toml` or the XDG config location. No external service or theme assets are needed.

To supply a versioned Taskotto design, set `RICEBOWL_TASKOTTO_RICEPACK=/absolute/path/taskotto-rice.toml` or use `--ricepack PATH`. RicePacks never execute code or overwrite saved appearance preferences; the v1 [schema and example](RICEPACK_FORMAT.md) describe the supported design tokens.

RiceBowl can export a compatible, validated Taskotto RicePack directly from its desktop recipe: `ricebowl de taskotto-export --recipe classic --output /absolute/path/taskotto-rice.toml`. Then run `taskotto --ricepack /absolute/path/taskotto-rice.toml`. The [cross-repository v1 fixture](tests/fixtures/ricebowl-classic-v1.toml) is tested by Taskotto's parser and CLI; this bridge is explicitly opt-in and does not make RiceBowl a dependency.

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
