# Screenshot provenance and reproduction

The README images are captured from **Taskotto itself**: an isolated Rust test
executes its real egui navigation, process page and performance page, then
rasterizes the resulting egui triangle meshes and font atlas to PNG. Process
names and resource values come from live Linux `/proc` samples; there are no
hand-drawn components, synthetic process entries or proprietary OS graphics.

The renderer is **headless and test-only** (the `png` crate is a dev-dependency).
It is not a native GPU/desktop screenshot: there is no X11/Wayland window frame,
and software antialiasing may differ slightly from the production renderer.
The restricted capture environment currently exposes few processes and very
little I/O; this is why some screenshots look sparse. They are not evidence of
hardware compatibility or GUI performance.

To regenerate the three images on Linux:

```bash
TASKOTTO_SCREENSHOT_DIR=assets/screenshots \
  cargo test --locked generate_headless_ui_screenshots -- --ignored --nocapture
```

The command samples real system activity for about four seconds and writes:

1. `assets/screenshots/taskotto-seven-performance.png` — Windows 7-inspired performance view.
2. `assets/screenshots/taskotto-processes-dark.png` — dark profile process list.
3. `assets/screenshots/taskotto-classic98.png` — Classic 98 process list.

Captures are pinned to 1280 × 800 pixels. Re-running on another machine will
change displayed processes, readings and potentially the image hashes. Inspect
the resulting images before committing them; do not imply they are native
screenshots. Capture real X11, KDE Wayland and labwc windows as part of the
[remaining hardware acceptance](QA.md) before a stable 1.0 release.
