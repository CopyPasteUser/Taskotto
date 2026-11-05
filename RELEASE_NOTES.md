# Taskotto 1.0.0-rc.2 — release candidate

**Date:** 2026-11-05 · **Platform:** Linux x86_64 development/CI build; X11/Wayland GUI hardware acceptance pending.

Rust-only Linux process manager and system monitor. This is a **prerelease**, not a stable 1.0 certification.

## Changes since rc.1

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
- Production-optimized x86_64 Linux build and manual `--snapshot`/JSON sanity checks are repeatable via CI; previous RC binary was approximately **11 MiB** in this environment.

## Known limitations / still required for 1.0 stable

- No GPU activity view, per-container accounting or automatic privilege escalation; these are deliberate scope exclusions.
- Virtual network interfaces can double-count traffic. Disk numbers are block-layer I/O.
- `pidfd` actions need Linux 5.3+; monitoring itself supports older kernels.
- GUI screenshots, keyboard/visual acceptance, installation and power/RAM measurements on Mint X11, KDE Wayland and labwc are pending (see [QA.md](QA.md)). Headless/container metrics are not a substitute.
- The GitHub 1.0.0 tag/release should be created only after those gates pass.
