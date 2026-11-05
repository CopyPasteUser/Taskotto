# Taskotto 1.0.0-rc.1 — release candidate

**Date:** 2026-11-05 · **Platform:** Linux x86_64 (build tested), X11/Wayland UI code

First executable Taskotto release candidate, written entirely in Rust.

## Implemented

- Live process monitor with search, sorts, tree mode, filters, user and detailed I/O data.
- Safe interactive SIGTERM, SIGKILL, SIGSTOP and SIGCONT actions via `pidfd`, with an explicit confirmation step and start-time check.
- Total/per-core CPU, RAM, swap, network and disk activity with bounded live graphs.
- Eight original visual presets, density and accent controls, persisted TOML settings and optional RiceBowl theme hand-off.
- XDG desktop file/icon, user-local `--install-desktop`, CLI snapshots and JSON output.
- Non-blocking sampler, no root service and no idle GUI render loop.

## Validation performed in the development environment

- Rust stable 1.99.1, `cargo fmt`, warning-free `cargo clippy`, all automated tests.
- Headless process actions on disposable children (pause, resume, terminate, and stale selection rejection).
- Production-optimized x86_64 Linux build; binary approximately **11 MiB** in this environment.
- Manual `--snapshot` and JSON sanity checks against current `/proc` values.

## Known limitations / still required for 1.0 stable

- No GPU activity view, per-container accounting or automatic privilege escalation; these are deliberate scope exclusions.
- Virtual network interfaces can double-count traffic. Disk numbers are block-layer I/O.
- `pidfd` actions need Linux 5.3+; monitoring itself supports older kernels.
- GUI screenshots, keyboard/visual acceptance and power/RAM measurements on Mint X11, KDE Wayland and labwc are pending (see [QA.md](QA.md)).
- The GitHub 1.0.0 tag/release should be created only after those gates pass.
