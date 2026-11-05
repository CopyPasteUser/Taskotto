# Taskotto — Roadmap to 1.0

Taskotto is a **standalone, on-demand Rust system monitor and process manager** for Linux. It must function on X11 and Wayland, independently of RiceBowl, while accepting optional RiceBowl styling. No root daemon, mandatory background service or compositor dependency.

## Acceptance rules for each milestone

- All changes remain in Rust; resources (`.desktop`, SVG, TOML) and workflow YAML are not application code.
- Unit/integration tests, `cargo fmt`, `cargo clippy`, and a release build pass in CI.
- Any advertised feature is implemented and discoverable, not a placeholder.
- Destructive actions have confirmation, permissions errors, and PID-reuse protection.
- Linux procfs errors, disappearing processes and incomplete permissions never panic the UI.
- Maintain useful fallback behavior on both X11 and Wayland; no desktop-specific services are required.
- Update README and release notes whenever behavior changes. Never publish performance numbers without measurements.

## 0.1 — Foundation

- [x] Rust crate with library and GUI binary; versioned lockfile.
- [x] Unprivileged `/proc` sampler with per-process data, host memory and CPU.
- [x] Strict process-stat parsing, including names with parentheses.
- [x] Command-line `--snapshot` and `--json` for headless diagnostics.
- [x] Initial unit tests, release profile, and CI workflow.

**Gate:** `cargo test`, `cargo run -- --snapshot`, `--snapshot --json` return truthful data on Linux.

## 0.2 — Process management

- [x] Search, CPU/RAM/PID/name sorting, live process list.
- [x] Process details, parent PID, state, thread count, command and I/O.
- [x] Graceful terminate, force kill, pause and resume with confirmation.
- [x] Linux `pidfd` signals plus process start-time validation; no PID reuse race in the final signal.
- [x] Permission and process-exit errors shown without application crashes.
- [x] Virtualized process rows for large systems.

**Gate:** signal action tests on a disposable child process, plus manually verify denial for a protected process. No blanket `sudo` operation.

## 0.3 — Performance view

- [x] Total CPU and per-core usage, memory and swap.
- [x] Rolling CPU/memory graphs and update interval.
- [x] Network RX/TX and block I/O rates, with correct units and caveats.
- [x] Non-blocking sampler worker, rate baselines and bounded history.

**Gate:** compare readings with `top`, `free`, `/proc/stat`, `/proc/net/dev` and `/proc/diskstats`. Document double-counting risks with virtual interfaces.

## 0.4 — Ricing foundation

- [x] Independent theme presets for Classic 98, Luna, Seven, Modern 10, Aero, Chrome and Mac.
- [x] Compact and comfortable density.
- [x] Optional accent hex override and persisted TOML settings.
- [x] Theme selection via `--theme` and opt-in RiceBowl environment contract.

**Gate:** no Microsoft, Google or Apple logos/assets; demonstrate theme and density switching without restart.

## 0.5 — Usability and reliability

- [x] Process tree and user/kernel-thread filters.
- [x] Search focus with Ctrl+F, manual refresh with F5.
- [x] Empty, error and confirmation states.
- [x] No automatic elevated permissions, no actions on PID 1 or Taskotto itself.
- [x] Show read/save failures rather than silently overwriting invalid preferences.
- [ ] Hardware UX acceptance on Mint (X11), Fedora KDE (Wayland), and labwc. *Needs the user's actual devices.*

**Gate:** keyboard navigation, readable labels, graph scaling and selection remain usable when process list changes.

## 0.6 — Desktop integration

- [x] Freedesktop `.desktop` entry and scalable icon.
- [x] Explicit install instructions; no mandatory RiceBowl dependency.
- [x] One-command, user-local desktop entry installer in Rust (`--install-desktop`).
- [x] CI build/test on Linux and headless smoke test.
- [x] Tag-triggered release workflow with tested Linux archive and SHA256 checksum.
- [ ] Real RiceBowl session launcher passing `RICEBOWL_RICE`. *Requires coordinated RiceBowl integration.*

**Gate:** launches from desktop menu after installation, and on both X11 and Wayland.

## 0.7 — Robustness and process safety

- [x] Avoid acting on a stale selection if PID has been reused.
- [x] Restrict data access to current user's kernel permissions.
- [x] Handle rapidly exiting PIDs, malformed stat records and reset counters.
- [x] Ensure worker thread stops when the window exits.
- [ ] Multi-hour soak test under rapid process creation/deletion on real hardware.

**Gate:** never signal the wrong process, never panic during process churn.

## 0.8 — Efficiency

- [x] Virtualized process table; no full UI repaint loop while idle.
- [x] Configurable sample interval (500ms–10s), no resident process when closed.
- [x] Bounded history and no external monitoring service.
- [ ] Measure cold start, idle CPU and RSS on low-end and modern machines; record reproducible results.

**Gate:** collect measured metrics, compare to `gnome-system-monitor`, `xfce4-taskmanager` and `btop` where available; avoid invented numbers.

## 0.9 — Release candidate

- [x] Buildable `1.0.0-rc.1` candidate with a headless layout smoke test and integration tests.
- [ ] X11/Wayland screenshot and interaction walkthrough.
- [ ] Complete hardware test checklist (`QA.md`) with recorded results.
- [ ] Packaging/desktop integration acceptance.
- [ ] CI green on the release commit; review warnings, errors, license/dependencies.

**Gate:** no P0/P1 defects, all regression tests passing and no misleading feature claims.

## 1.0 — Stable release

- [ ] Complete the hardware acceptance checklist on representative desktops.
- [ ] Record reference performance and limitations in release notes.
- [ ] Create a signed-off GitHub 1.0.0 tag/release with a reproducible Linux build.
- [ ] Mark 1.0 only after all required gates pass.

### After 1.0 (not blockers)

Optional per-GPU utilization (driver-specific), container/cgroup view, separate process inspector, more RiceBowl-specific window chrome, and profile-dependent layout modes. These must not burden the baseline runtime.

### Why this architecture?

`monitor.rs` owns all Linux system reading; `actions.rs` handles narrowly scoped signals; `config.rs` owns persistent preferences; `ui.rs` only coordinates the display. RiceBowl remains an **optional launcher/theme source**, not a runtime dependency. A dedicated monitor service is unnecessary because Taskotto only runs when the user opens it.
