# Taskotto 1.0.0-rc.4 — release candidate

**Date:** 2026-11-06 · **Platform:** Linux x86_64 development/CI build; X11/Wayland GUI hardware acceptance pending.

Rust-only Linux process manager and system monitor. This is a **prerelease**, not a stable 1.0 certification.

## Changes since rc.3

- Dedicated XP Luna task-manager UI: original blue title gradient, raised beige tabs, white process table, persistent status strip and classic green-on-black CPU/RAM usage history.
- Dedicated Windows 10-inspired UI: white task-manager chrome, menus and tabs, CPU/memory table heatmap, truthful user/system process groups and a four-way CPU/memory/disk/network performance navigator.
- Menus, row/column sorting, process context actions and End Process/End task use existing real Taskotto features; all signals still require confirmation and pidfd safety checks.
- Four new inspected headless screenshots using authentic egui commands and actual procfs samples, plus focused color/overflow/layout regressions.
- Local stripped x86_64 Linux executable: **10,892,048 bytes** (`stat -c %s target/release/taskotto`, RC4 build). This is file size, not idle RAM.
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

## Validation performed in the development environment

- Rust stable 1.99.1, `cargo fmt`, warning-free `cargo clippy`, and automated unit/integration tests (run again against this exact revision before release).
- Headless process actions on disposable children (pause, resume, terminate, and stale selection rejection).
- Production-optimized x86_64 Linux binary measured **10,651,280 bytes** in the development environment. Headless CLI timing/RSS samples, methods and caveats are in [PERFORMANCE.md](PERFORMANCE.md); these are not GUI benchmarks.
- PR #3 and its `main` merge passed GitHub Rust CI (2026-11-05; 32 automated tests locally); the rc.3 checks are separately reported when complete. No stable release tag was created.

## Known limitations / still required for 1.0 stable

- No GPU activity view, per-container accounting or automatic privilege escalation; these are deliberate scope exclusions.
- Virtual network interfaces can double-count traffic. Disk numbers are block-layer I/O.
- `pidfd` actions need Linux 5.3+; monitoring itself supports older kernels.
- **Native desktop screenshots**, keyboard/visual acceptance, installation and power/RAM measurements on Mint X11, KDE Wayland and labwc are pending (see [QA.md](QA.md)). Headless egui renders and container metrics are not substitutes.
- The GitHub 1.0.0 tag/release should be created only after those gates pass.
