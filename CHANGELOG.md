# Changelog

Taskotto follows semantic versioning; hardware-dependent stable-release gates are explicit.

## [1.0.0-rc.4] — 2026-11-06

- Separate Windows 10-inspired and XP Luna-inspired task-manager screens with original Chrome, functional menus and native-feeling tab navigation.
- Windows 10 light process heatmap, responsive sortable columns, truthful Linux process grouping and switchable CPU/memory/disk/network performance focus.
- XP Luna blue gradient, cream controls, raised tab/table styling, status bar and dark-green performance history.
- Shared functional context menus for process details, termination, force kill, pause and resume using the existing pidfd-protected confirmation flow.
- Four actual egui screenshots with live procfs measurements, additional narrow/wide layout and color-token regression tests.
- No proprietary Windows graphics; native compositor window frames and hardware acceptance remain open.

## [1.0.0-rc.3] — 2026-11-06

- README gallery with genuine headless egui-rendered screenshots and reproducible, documented live-procfs capture workflow.
- Shared scale and color distinction for simultaneous network RX/TX and block read/write graphs, with gridlines and full-width early sample history.
- Readable text navigation and sort controls in place of missing default-font glyphs.
- Real-desktop screenshot, performance and interaction sign-off remains pending; stable 1.0 is not claimed.

## [1.0.0-rc.2] — 2026-11-05

- Process start/runtime and per-process I/O rates; network and disk history charts.
- Bounded sampler queue during minimized/unresponsive GUI periods.
- Data-driven visual tokens, navigation/table layouts and readable-color fallback.
- Session-only RiceBowl/CLI themes without overwriting saved preferences.
- Atomic settings persistence with backup of malformed configuration.
- Expanded automated tests and release security/contributor materials.

## [1.0.0-rc.1] — 2026-11-05

- First Rust GUI candidate: live processes, safe pidfd actions, system resources, eight theme presets, headless snapshots, desktop integration and CI.

Stable `1.0.0` follows X11/Wayland, GUI performance and packaging sign-off.
