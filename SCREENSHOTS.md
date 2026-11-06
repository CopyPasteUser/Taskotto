# Screenshot provenance and reproduction

The README images are captured from **Taskotto itself**: an isolated Rust test
executes its real egui navigation, modal and rice-specific pages, then
rasterizes the resulting egui triangle meshes and font atlas to PNG. Process
names and resource values come from live Linux `/proc` samples; autostart and
service information comes from real XDG files and an optional systemd user
query. There are no hand-drawn components, synthetic process entries or
proprietary OS graphics.

The renderer is **headless and test-only** (the `png` crate is a dev-dependency).
It is not a native GPU/desktop screenshot: there is no X11/Wayland window frame,
and software antialiasing may differ slightly from the production renderer.
The restricted capture environment currently exposes few processes and very
little I/O; this is why some screenshots look sparse. They are not evidence of
hardware compatibility or GUI performance.

To regenerate all 32 images on Linux:

```bash
TASKOTTO_SCREENSHOT_DIR=assets/screenshots \
  cargo test --locked generate_headless_ui_screenshots -- --ignored --nocapture
```

The command samples real system activity for about four seconds and writes:

### Windows 98 (Close Program and explicitly modern Linux extensions)

- `assets/screenshots/taskotto-win98-close-program.png` — compact 650 × 510 dialog-like client, actual current user's Linux processes and safe disabled host shutdown, without fake caption buttons.
- `assets/screenshots/taskotto-win98-process-report.png` — advanced sortable real Linux process table, accessible via Tools; not a historical Windows 98 tab.
- `assets/screenshots/taskotto-win98-system-monitor.png` — classic green-on-black CPU and turquoise memory history with Linux readings.
- `assets/screenshots/taskotto-win98-network.png` — measured system-wide network usage, not original Windows 98 per-interface metadata.
- `assets/screenshots/taskotto-win98-information.png` — permission-sensitive live selected process details, no forged app/window ownership.

### macOS Activity Monitor (five views and process information)

- `assets/screenshots/taskotto-macos-cpu.png` — sortable CPU report, process actions, search and genuine CPU load history.
- `assets/screenshots/taskotto-macos-memory.png` — RSS/virtual-memory process report, aggregate Linux physical and swap usage; not Apple's private memory or Memory Pressure algorithm.
- `assets/screenshots/taskotto-macos-energy.png` — per-process CPU activity as an **explicitly labeled proxy**, because macOS Energy Impact / App Nap are not available from portable Linux procfs.
- `assets/screenshots/taskotto-macos-disk.png` — permission-sensitive real `/proc/PID/io` process bytes and aggregate block-device rates.
- `assets/screenshots/taskotto-macos-network.png` — real non-loopback system-wide traffic history; no imaginary per-process counters.
- `assets/screenshots/taskotto-macos-info.png` — modal process information from the currently selected actual PID.

### Windows 7 (six historical tabs)

- `assets/screenshots/taskotto-win7-applications.png` — current-user process approximation, not compositor window enumeration.
- `assets/screenshots/taskotto-win7-processes.png` — Aero-era client controls, six tabs, compact sortable report and PID-safe process actions; the titlebar is compositor-owned.
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

The old dark, Classic 98 and Seven gallery captures were removed in the RC7
focused fidelity pass. Their original styles remain available in Taskotto.

To regenerate only one rice's screenshots (and preserve the others), set
`TASKOTTO_SCREENSHOT_FILTER=win7`, `TASKOTTO_SCREENSHOT_FILTER=win10`,
`TASKOTTO_SCREENSHOT_FILTER=xp-luna`, `TASKOTTO_SCREENSHOT_FILTER=macos` or
`TASKOTTO_SCREENSHOT_FILTER=win98`
before the same command.

Most captures are pinned to 1280 × 800 pixels; Windows 98 uses its compact
default client dimensions, 650 × 510 pixels. Re-running on another machine will
change displayed processes, readings and potentially the image hashes. Inspect
the resulting images before committing them; do not imply they are native
screenshots. Capture real X11, KDE Wayland and labwc windows as part of the
[remaining hardware acceptance](QA.md) before a stable 1.0 release.

Top-level native titlebars, accent-colored frames and caption buttons are
deliberately **not** painted by Taskotto. They are owned by the active window
manager/compositor and the RiceBowl rice. These headless captures only include
the egui client area. The older XP, Seven, Win10 and macOS scenes have been
recaptured after removing earlier fake in-client title treatments.

The sidebar capture is an explicitly **custom** layout. XDG and systemd
screens display live local configuration and availability; they are not
Windows-native startup impact estimates or Windows service manager output.
