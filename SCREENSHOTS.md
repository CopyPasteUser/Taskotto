# Screenshot provenance and reproduction

The README images are captured from **Taskotto itself**: an isolated Rust test
executes its real egui navigation and rice-specific pages, then
rasterizes the resulting egui triangle meshes and font atlas to PNG. Process
names and resource values come from live Linux `/proc` samples; there are no
hand-drawn components, synthetic process entries or proprietary OS graphics.

The renderer is **headless and test-only** (the `png` crate is a dev-dependency).
It is not a native GPU/desktop screenshot: there is no X11/Wayland window frame,
and software antialiasing may differ slightly from the production renderer.
The restricted capture environment currently exposes few processes and very
little I/O; this is why some screenshots look sparse. They are not evidence of
hardware compatibility or GUI performance.

To regenerate all nine images on Linux:

```bash
TASKOTTO_SCREENSHOT_DIR=assets/screenshots \
  cargo test --locked generate_headless_ui_screenshots -- --ignored --nocapture
```

The command samples real system activity for about four seconds and writes:

1. `assets/screenshots/taskotto-win10-processes.png` — Windows 10-inspired process grouping and heatmap.
2. `assets/screenshots/taskotto-win10-performance.png` — Windows 10-inspired CPU performance with metric navigation.
3. `assets/screenshots/taskotto-win10-users.png` — real per-UID process grouping and resource totals.
4. `assets/screenshots/taskotto-win10-details.png` — ungrouped Linux process list in a details-table style.
5. `assets/screenshots/taskotto-xp-luna-applications.png` — current-user process list; **not** a compositor window enumeration.
6. `assets/screenshots/taskotto-xp-luna-processes.png` — XP Luna-inspired process view.
7. `assets/screenshots/taskotto-xp-luna-performance.png` — XP Luna-inspired CPU/memory history.
8. `assets/screenshots/taskotto-xp-luna-networking.png` — actual non-loopback receive/send trends.
9. `assets/screenshots/taskotto-xp-luna-details.png` — selected process properties with Linux-native fields.

The old dark, Classic 98 and Seven gallery captures were removed in this
focused fidelity pass. Their original styles remain available in Taskotto.

To regenerate only one rice's screenshots (and preserve the others), set
`TASKOTTO_SCREENSHOT_FILTER=win10` or `TASKOTTO_SCREENSHOT_FILTER=xp-luna`
before the same command.

Captures are pinned to 1280 × 800 pixels. Re-running on another machine will
change displayed processes, readings and potentially the image hashes. Inspect
the resulting images before committing them; do not imply they are native
screenshots. Capture real X11, KDE Wayland and labwc windows as part of the
[remaining hardware acceptance](QA.md) before a stable 1.0 release.
