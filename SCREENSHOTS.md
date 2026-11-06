# Screenshot provenance and reproduction

[**View all 38 screenshots in the expandable README gallery**](README.md#screenshots).
The gallery groups every image by rice, including the supplementary pages
and alternate layouts; click any preview to see its full-resolution PNG.

The README images are captured from **Taskotto itself**: an isolated Rust test
executes its real egui navigation, modal and rice-specific pages, then
rasterizes the resulting egui triangle meshes and font atlas to PNG. Process
names and resource values come from live Linux `/proc` samples; autostart and
service information comes from real XDG files and an optional systemd user
query. The test briefly starts and reaps a real unprivileged `sleep` process so
the sparse CI environment has an actionable row to show. There are no hand-drawn
components, synthetic process entries or proprietary OS graphics.

The renderer is **headless and test-only** (the `png` crate is a dev-dependency).
It is not a native GPU/desktop screenshot: there is no X11/Wayland window frame,
and software antialiasing may differ slightly from the production renderer.
The rc.13 captures show the app content **without** any extra XP/Seven/Ten
title strip: RiceBowl or the currently active window manager supplies the
real outer caption, title-bar buttons, border colors and focus styling.
The restricted capture environment currently exposes few processes and very
little I/O; this is why some screenshots look sparse. They are not evidence of
hardware compatibility or GUI performance.

To regenerate all 38 images on Linux:

```bash
TASKOTTO_SCREENSHOT_DIR=assets/screenshots \
  cargo test --locked generate_headless_ui_screenshots -- --ignored --nocapture
```

The command samples real system activity and writes:

### ChromeOS Task manager (primary task table and supplementary tools)

- `assets/screenshots/taskotto-chromeos-tasks.png` — compact ChromeOS-inspired Linux process list with live memory RSS, CPU, PID and a confirmation-protected End process action (disabled until selection); not a ChromeOS or Chromium native window.
- `assets/screenshots/taskotto-chromeos-performance.png` — supplementary Linux CPU, RAM, disk and uptime metrics, reachable via the toolbar menu. It does not invent proprietary ChromeOS counters.

### Frutiger Aero (original Y2K design)

- `assets/screenshots/taskotto-frutiger-aero-processes.png` — live Linux process report, active aqua pill navigation and search/sort/tree/actions.
- `assets/screenshots/taskotto-frutiger-aero-sidebar.png` — optional custom vertical navigation with the same process report.
- `assets/screenshots/taskotto-frutiger-aero-performance.png` — four live CPU, RAM, disk and network graphs plus actual memory/CPU gauges.
- `assets/screenshots/taskotto-frutiger-aero-network.png` — measured aggregate non-loopback RX/TX with explicit scope caveat.
- `assets/screenshots/taskotto-frutiger-aero-ricepack-editor.png` — actual Taskotto Preferences screen showing the bundled Sea Glass RicePack in reversible session-only preview; no desktop frame or external file chooser is implied.

### Windows 98 (Close Program and optional advanced tools)

- `assets/screenshots/taskotto-win98-close-program.png` — authentic compact dialog layout, white inset list of current-user Linux processes, 3D controls and confirmation-backed End Task.
- `assets/screenshots/taskotto-win98-new-task.png` — user-entered New Task command dialog; no elevation or background service.
- `assets/screenshots/taskotto-win98-processes.png` — optional classic process report with real PID/CPU/RSS data, search, sorting and safe actions.
- `assets/screenshots/taskotto-win98-performance.png` — real Linux CPU/memory history in classic high-contrast graphs.

### macOS Activity Monitor (five views and process information)

- `assets/screenshots/taskotto-macos-cpu.png` — sortable CPU report, process actions, search and genuine CPU load history.
- `assets/screenshots/taskotto-macos-memory.png` — RSS/virtual-memory process report, aggregate Linux physical and swap usage; not Apple's private memory or Memory Pressure algorithm.
- `assets/screenshots/taskotto-macos-energy.png` — per-process CPU activity as an **explicitly labeled proxy**, because macOS Energy Impact / App Nap are not available from portable Linux procfs.
- `assets/screenshots/taskotto-macos-disk.png` — permission-sensitive real `/proc/PID/io` process bytes and aggregate block-device rates.
- `assets/screenshots/taskotto-macos-network.png` — real non-loopback system-wide traffic history; no imaginary per-process counters.
- `assets/screenshots/taskotto-macos-info.png` — modal process information from the currently selected actual PID.

### Windows 7 (six historical tabs)

- `assets/screenshots/taskotto-win7-applications.png` — current-user process approximation, not compositor window enumeration.
- `assets/screenshots/taskotto-win7-processes.png` — Aero-era chrome, six tabs, compact sortable report and PID-safe process actions.
- `assets/screenshots/taskotto-win7-services.png` — real read-only systemd **user** units or explicit unavailable/error state (as in restricted CI).
- `assets/screenshots/taskotto-win7-performance.png` — original green CPU and blue memory histories with Linux CPU-core samples and memory statistics.
- `assets/screenshots/taskotto-win7-networking.png` — real non-loopback receive/send trends, bytes/second not Windows link utilization.
- `assets/screenshots/taskotto-win7-users.png` — per-UID process owner resource summary, not a session enumeration.
- `assets/screenshots/taskotto-win7-properties.png` — selected-process inspection in a floating properties dialog, not an invented seventh tab.

### Windows 10 (seven historical tabs, plus custom sidebar)

- `assets/screenshots/taskotto-win10-processes.png` — horizontal seven-tab navigation and process heatmap.
- `assets/screenshots/taskotto-win10-vertical.png` — custom optional vertical navigation (not the historical Windows 10 default).
- `assets/screenshots/taskotto-win10-performance.png` — CPU performance and metric navigator.
- `assets/screenshots/taskotto-win10-app-history.png` — current Linux process-lifetime CPU accounting, not persistent Windows app history.
- `assets/screenshots/taskotto-win10-startup.png` — XDG autostart entries and their configured state.
- `assets/screenshots/taskotto-win10-services.png` — asynchronous systemd user service status or explicit environment error/loading state.
- `assets/screenshots/taskotto-win10-users.png` — per-UID process grouping and resource totals.
- `assets/screenshots/taskotto-win10-details.png` — ungrouped Linux process details table.

### XP Luna (five historical tabs)

- `assets/screenshots/taskotto-xp-luna-applications.png` — current-user process list; **not** a compositor window enumeration.
- `assets/screenshots/taskotto-xp-luna-new-task.png` — functional user-command launch dialog.
- `assets/screenshots/taskotto-xp-luna-processes.png` — classic process view.
- `assets/screenshots/taskotto-xp-luna-performance.png` — CPU/memory history.
- `assets/screenshots/taskotto-xp-luna-networking.png` — non-loopback receive/send trends.
- `assets/screenshots/taskotto-xp-luna-details.png` — process properties using live Linux fields.

The obsolete generic dark/Classic 98 gallery captures were removed in RC7;
RC11 added four independently rendered Classic 98 views; RC12 adds two ChromeOS and four Frutiger Aero views.

To regenerate only one rice's screenshots (and preserve the others), set
`TASKOTTO_SCREENSHOT_FILTER=win7`, `TASKOTTO_SCREENSHOT_FILTER=win10`,
`TASKOTTO_SCREENSHOT_FILTER=xp-luna`, `TASKOTTO_SCREENSHOT_FILTER=win98`,
`TASKOTTO_SCREENSHOT_FILTER=chromeos`, `TASKOTTO_SCREENSHOT_FILTER=macos`
or `TASKOTTO_SCREENSHOT_FILTER=frutiger-aero`
before the same command.

Classic 98 captures are rendered at the actual 650 × 450 default viewport;
other captures remain 1280 × 800 pixels. Re-running on another machine will
change displayed processes, readings and potentially the image hashes. Inspect
the resulting images before committing them; do not imply they are native
screenshots. Capture real X11, KDE Wayland and labwc windows as part of the
[remaining hardware acceptance](QA.md) before a stable 1.0 release.

The sidebar capture is an explicitly **custom** layout. XDG and systemd
screens display live local configuration and availability; they are not
Windows-native startup impact estimates or Windows service manager output.
