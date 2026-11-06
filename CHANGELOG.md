# Changelog

Taskotto follows semantic versioning; hardware-dependent stable-release gates are explicit.

## [1.0.0-rc.6] — 2026-11-06

- Windows 10 and XP Luna task-manager views receive a closer historical layout: slimmer title/menu/navigation chrome, table-first process pages, refined tabs, and a search field opened only by Ctrl+F or the View menu.
- XP Luna uses its white full-height process-list surface, era-appropriate column headings and row heights, the all-users checkbox and grouped CPU/RAM/swap/thread statistics.
- Windows 10 process heatmaps and focused performance history retain real Linux values with a subtle tinted history fill and clearer legend labels.
- Fixed `/proc/PID/stat` thread-count parsing (field 20 was read as field 19); a regression test confirms correct values.
- Refreshed four real, headless egui process/performance screenshots using live Linux data. No Windows assets, fake data or native-GUI validation claims.
- Rust 1.88 formatting, 41 automated tests (including the ignored screenshot generator) and warning-free Clippy passed locally; real hardware acceptance is still outstanding.
- Follow-up CI compatibility fix: explicitly type seven egui stroke widths as `f32` for newer Rust's strict floating-point inference.

## [1.0.0-rc.5] — 2026-11-06

- End-to-end Linux tests exercise real child-process pause/resume/terminate/kill, stale PID start-time rejection, CLI JSON output and invalid argument handling.
- User-local desktop install now replaces the launcher and icon atomically, including a regression against following symlinked output files.
- Automated installation/reinstallation smoke test verifies real desktop and SVG files under an isolated XDG data directory.
- CI checks the declared Rust 1.88 minimum, warning-free documentation and revision-pinned upstream actions. Tag releases recheck formatting/lints, construct deterministically timestamped archives, and smoke-test the archive and checksum before publishing.
- No scope creep: final X11/Wayland compatibility, native visual accuracy, GUI performance and packaging sign-off still require real machines.

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
