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

To regenerate all 14 images on Linux:

```bash
TASKOTTO_SCREENSHOT_DIR=assets/screenshots \
  cargo test --locked generate_headless_ui_screenshots -- --ignored --nocapture
```

The command samples real system activity for about four seconds and writes:

1. `assets/screenshots/taskotto-win10-processes.png` — horizontal seven-tab navigation and process heatmap.
2. `assets/screenshots/taskotto-win10-vertical.png` — custom optional vertical navigation (not the historical Windows 10 default).
3. `assets/screenshots/taskotto-win10-performance.png` — CPU performance and metric navigator.
4. `assets/screenshots/taskotto-win10-app-history.png` — current Linux process-lifetime CPU accounting, not persistent Windows app history.
5. `assets/screenshots/taskotto-win10-startup.png` — XDG autostart entries and their configured state.
6. `assets/screenshots/taskotto-win10-services.png` — asynchronous systemd user service status or explicit environment error/loading state.
7. `assets/screenshots/taskotto-win10-users.png` — per-UID process grouping and resource totals.
8. `assets/screenshots/taskotto-win10-details.png` — ungrouped Linux process details table.
9. `assets/screenshots/taskotto-xp-luna-applications.png` — current-user process list; **not** a compositor window enumeration.
10. `assets/screenshots/taskotto-xp-luna-new-task.png` — functional user-command launch dialog.
11. `assets/screenshots/taskotto-xp-luna-processes.png` — classic process view.
12. `assets/screenshots/taskotto-xp-luna-performance.png` — CPU/memory history.
13. `assets/screenshots/taskotto-xp-luna-networking.png` — non-loopback receive/send trends.
14. `assets/screenshots/taskotto-xp-luna-details.png` — process properties using live Linux fields.

The old dark, Classic 98 and Seven gallery captures were removed in the RC7
focused fidelity pass. Their original styles remain available in Taskotto.

To regenerate only one rice's screenshots (and preserve the others), set
`TASKOTTO_SCREENSHOT_FILTER=win10` or `TASKOTTO_SCREENSHOT_FILTER=xp-luna`
before the same command.

Captures are pinned to 1280 × 800 pixels. Re-running on another machine will
change displayed processes, readings and potentially the image hashes. Inspect
the resulting images before committing them; do not imply they are native
screenshots. Capture real X11, KDE Wayland and labwc windows as part of the
[remaining hardware acceptance](QA.md) before a stable 1.0 release.

The sidebar capture is an explicitly **custom** layout. XDG and systemd
screens display live local configuration and availability; they are not
Windows-native startup impact estimates or Windows service manager output.
