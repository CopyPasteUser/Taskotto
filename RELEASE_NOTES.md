# Taskotto 1.0.0-rc.8 — release candidate

**Date:** 2026-11-06 · **Platform:** Linux x86_64 development/CI build; X11/Wayland GUI hardware acceptance pending.

Rust-only Linux process manager and system monitor. This is a **prerelease**, not a stable 1.0 certification.

## Changes since rc.7

- Windows 10's historical navigation has seven horizontal tabs rather than five. New App history, Startup, and Services pages operate on live Linux data, without inventing Windows APIs or fake startup impact estimates: current-process CPU lifetime counters, XDG autostart desktop files and a nonblocking systemd user service query.
- The vertical Windows-10-like sidebar requested as an alternate layout is available via Preferences → Navigation → Sidebar. It is explicitly custom, because the original Windows 10 Task Manager used horizontal tabs (the later Windows 11 task manager introduced a sidebar).
- XP Luna Applications adds a functional New Task/Run dialog; the menu offers the same command launcher. It runs the user's explicit input with the same Linux permissions and no background service or privilege escalation. Switch To remains disabled because Wayland/X11 window-to-PID activation cannot be mapped portably.
- New parsing tests cover XDG override precedence, desktop file visibility and systemd unit rows. Extra layout checks cover narrow and wide horizontal/vertical pages.
- Fourteen genuine headless screenshot scenes demonstrate the added functionality, including a vertical sidebar and New Task modal, with accurate provenance and the existing honest native-UI caveat.
- **Important:** 95% pixel-fidelity cannot be empirically certified without side-by-side native desktop reference captures and actual user-hardware walkthroughs.

## Earlier changes (rc.6 → rc.7)

- XP Luna now exposes Applications, Processes, Performance, Networking and Users, mirroring the historical tab structure. Applications is a clearly labeled **current-user process** approximation, not a graphical window list: portable Wayland procfs data cannot prove window ownership.
- The XP Networking tab draws genuine receive and send history for non-loopback interfaces. Users groups actual Linux processes by UID, and selected-process Properties uses classic segmented rows and existing confirmation-protected signals.
- Windows 10 adds a per-user expandable process summary and a separate ungrouped Details process table with existing sort/inspect/action behavior.
- Improved XP Applications left alignment/selection and truncated oversized Properties command previews to preserve layout.
- README now leads with two representative current screenshots, offering seven additional XP/Windows 10 captures in an expandable gallery. All nine come from genuine Taskotto egui rendering with live procfs values. Removed the obsolete generic dark, Classic 98, and Seven images from the repository; those rice themes remain available.
- At rc.7 the App history, Startup and Services tabs were still absent; RC8 adds explicitly labeled Linux equivalents. Windows-native window/process categories still do not map directly to Linux, and native X11/Wayland comparison remains a release gate.

## Earlier changes (rc.5 → rc.6)

- Both Taskotto legacy task-manager rices were refined against historical XP Luna and Windows 10 UI patterns: slimmer window/menu/tab chrome, default table-first process display and era-specific colors, sizing and structure.
- XP now has a white full-height process pane, classic `Image Name`/`Mem Usage` headings, compact rows and grouped CPU, memory, swap and thread statistics; the all-users toggle remains functional.
- Windows 10 has cleaner process grouping and CPU/memory heatmaps, and the performance history line has a translucent area fill.
- The optional Linux search is exposed on Ctrl+F and View → Find rather than always occupying space; action menus, safety confirmations and theme overrides remain functional.
- Corrected a genuine `/proc/PID/stat` bug where the process's nice value was displayed as its number of threads. Added a failing-then-passing regression assertion.
- Replaced four real egui/process snapshots in the README and retained provenance; hardware acceptance and pixel fidelity are not claimed.
- Declared egui stroke widths explicitly as `f32` after newer GitHub CI Rust caught inference fallback that the Rust 1.88 compatibility build still permitted.

## Earlier changes (rc.4 → rc.5)

- Tests now launch real disposable Linux processes and verify all four pidfd signal actions, stale start-tick rejection and safe cleanup.
- End-to-end executable tests check JSON snapshots, malformed CLI options, XDG desktop installation and repeat installation.
- Desktop launcher and icon are atomically replaced, including an automated symlink-target safety regression.
- CI checks Rust 1.88 compatibility and warning-free documentation and pins upstream GitHub Actions to verified revision hashes.
- Release archives use stable ordering/ownership/commit timestamps and undergo checksum verification, extraction, CLI smoke test and user-local desktop install before GitHub publication.
- Source functionality and the Windows 10 / XP Luna rice appearance were unchanged from rc.4; screenshots were regenerated from rc.5's real egui renderer before the rc.6 fidelity pass.
- RC5 locally stripped Linux x86_64 binary: **10,892,944 bytes** (`stat -c %s target/release/taskotto`). This is executable file size, not runtime RSS.

## Earlier changes (rc.3 → rc.4)

- Dedicated XP Luna task-manager UI: original blue title gradient, raised beige tabs, white process table, persistent status strip and classic green-on-black CPU/RAM usage history.
- Dedicated Windows 10-inspired UI: white task-manager chrome, menus and tabs, CPU/memory table heatmap, truthful user/system process groups and a four-way CPU/memory/disk/network performance navigator.
- Menus, row/column sorting, process context actions and End Process/End task use existing real Taskotto features; all signals still require confirmation and pidfd safety checks.
- Four new inspected headless screenshots using authentic egui commands and actual procfs samples, plus focused color/overflow/layout regressions.
- The rc.4 stripped x86_64 Linux executable measured **10,892,048 bytes** (`stat -c %s target/release/taskotto`). File size is not idle RAM.
- Window decorations remain under the Linux window manager/compositor; custom UI does not ship Windows logos, task-manager binaries or proprietary artwork. See [RICE_FIDELITY.md](RICE_FIDELITY.md).

## Earlier changes (rc.2 → rc.3)

- Network receive/transmit and block read/write now share comparable, dual-color charts with gridlines and scaling labels; short histories span the full chart width.
- Removed navigation and sort-label symbols that were rendered as missing-font boxes with the built-in egui font.
- Added three inspected, reproducible screenshots painted from Taskotto's actual egui commands and live procfs data. They use a test-only software rasterizer, not a running desktop session; process visibility is limited by the capture environment. See [SCREENSHOTS.md](SCREENSHOTS.md).
- The locally built, stripped x86_64 Linux rc.3 executable measured **10,652,944 bytes** (`stat -c %s target/release/taskotto`); this is file size, not runtime RSS.

## Earlier changes (rc.1 → rc.2)

- Process start time, elapsed runtime and per-process read/write throughput, with PID-reuse-aware rate baselines.
- Rolling network and disk I/O graphs alongside CPU and memory history.
- One-slot sampler queue, avoiding a growing backlog while the GUI is minimized.
- Data-driven theme tokens for optional surface/text/accent colors, fonts, spacing, corners, responsive process-table columns/row height and layout (sidebar/top tabs).
- Generated-color contrast fallback, brightness-aware styling and session-only RiceBowl/CLI theme overrides.
- Restricted per-process I/O counters show an explicit unavailable state rather than a misleading zero; an unreadable `/proc` root returns a monitoring error.
- Atomic private settings writes with malformed-config backup and additional regression tests.
- Eight-profile headless layout checks, contributor/security policies and Dependabot configuration.

## Implemented

- Live process monitor with search, sorts, tree mode, filters, user and detailed I/O data.
- Safe interactive SIGTERM, SIGKILL, SIGSTOP and SIGCONT actions via `pidfd`, with an explicit confirmation step and start-time check.
- Total/per-core CPU, RAM, swap, network and disk activity with bounded live graphs.
- Eight original visual presets, style/layout controls, persisted TOML settings and optional RiceBowl theme hand-off.
- XDG desktop file/icon, user-local `--install-desktop`, CLI snapshots and JSON output.
- Non-blocking sampler, no root service and no idle GUI render loop.

## Validation and release gates

- `cargo fmt --all -- --check`, warning-free `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked --all-targets`, and `cargo build --locked --release` are the mandatory baseline; record exact results after each RC merge.
- Rust 1.88 check, `cargo doc` with warnings denied, subprocess signal and actual CLI/desktop E2E tests are part of CI.
- Headless CLI timing/RSS samples, methods and caveats are in [PERFORMANCE.md](PERFORMANCE.md); **they are not GUI benchmarks**.
- The release workflow builds a checked archive and publishes only a version-matched tag, with a SHA256 verification and extracted-binary smoke test.
- The local archive packaging rehearsal produced identical SHA256 checksums for two archives built from the same binary/files. Full binary reproducibility across toolchains is not claimed.

## Known limitations / still required for 1.0 stable

- No GPU activity view, per-container accounting or automatic privilege escalation; these are deliberate scope exclusions.
- Virtual network interfaces can double-count traffic. Disk numbers are block-layer I/O.
- `pidfd` actions need Linux 5.3+; monitoring itself supports older kernels.
- **Native desktop screenshots**, keyboard/visual acceptance, installation and power/RAM measurements on Mint X11, KDE Wayland and labwc are pending (see [QA.md](QA.md)). Headless egui renders and container metrics are not substitutes.
- The GitHub 1.0.0 tag/release should be created only after those gates pass.
