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

- [x] Search, CPU/RAM/I/O/PID/name sorting, live process list.
- [x] Process details, parent PID, state, thread count, command, start/runtime, I/O totals and rates.
- [x] Graceful terminate, force kill, pause and resume with confirmation.
- [x] Linux `pidfd` signals plus process start-time validation; no PID reuse race in the final signal.
- [x] Permission and process-exit errors shown without application crashes.
- [x] Virtualized process rows for large systems.

**Gate:** signal action tests on a disposable child process, plus manually verify denial for a protected process. No blanket `sudo` operation.

## 0.3 — Performance view

- [x] Total CPU and per-core usage, memory and swap.
- [x] Rolling CPU/memory/network/disk graphs and update interval.
- [x] Network RX/TX and block I/O rates, with correct units and caveats.
- [x] Non-blocking sampler worker, rate baselines, bounded history and one-slot snapshot queue.

**Gate:** compare readings with `top`, `free`, `/proc/stat`, `/proc/net/dev` and `/proc/diskstats`. Document double-counting risks with virtual interfaces.

## 0.4 — Ricing foundation

- [x] Independent theme presets for Classic 98, Luna, Seven, Modern 10, Aero, Chrome and Mac.
- [x] Compact and comfortable density.
- [x] Data-driven colors/typography/spacing/corners, responsive table columns/row heights, layout variants and persisted TOML settings.
- [x] Theme selection via `--theme` and opt-in RiceBowl environment contract.
- [x] Session-only launcher overrides and readable-contrast fallback for custom text colors.

**Gate:** no Microsoft, Google or Apple logos/assets; demonstrate theme and density switching without restart.

## 0.5 — Usability and reliability

- [x] Process tree and user/kernel-thread filters.
- [x] Search focus with Ctrl+F, manual refresh with F5.
- [x] Empty, error and confirmation states.
- [x] No automatic elevated permissions, no actions on PID 1 or Taskotto itself.
- [x] Show read/save failures and back up malformed preferences before replacing the file.
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
- [x] Document reproducible measured headless CLI binary size, timing and point-sampled RSS in `PERFORMANCE.md` (not GUI-equivalent).
- [ ] Measure **GUI** cold start, idle CPU and RSS on low-end and modern machines. *Requires desktop sessions.*

**Gate:** collect measured metrics, compare to `gnome-system-monitor`, `xfce4-taskmanager` and `btop` where available; avoid invented numbers.

## 0.9 — Release candidate

- [x] Buildable `1.0.0-rc.6` candidate with all-profile small/large headless layout and integration tests.
- [x] Contributor/security policies, dependency-update configuration and CI checks.
- [x] Reproducible egui-rendered headless screenshots with real procfs measurements, documented provenance and visual inspection.
- [x] Consistent network/disk comparison charts (both directions on one scale), early-history width regression coverage and readable navigation labels.
- [x] Deep, independent XP Luna and Windows 10 task-manager layouts with functional menus, controls, history views, accessible focus and four additional genuine headless screenshots.
- [x] RC6 visual fidelity pass: classic white XP process pane and statistic groups, compact legacy chrome, hidden-until-requested search, softer Windows 10 history graph fill; fixed `/proc` thread count.
- [x] Linux end-to-end child-signal test of all four signals, PID start-time mismatch and failure handling.
- [x] Real CLI JSON/error handling and isolated XDG desktop install/reinstall integration tests.
- [x] Atomic launcher/icon replacement, including regression against symlink-target overwrites.
- [x] Pinned upstream CI actions, Rust 1.88 MSRV checks and documentation build checks.
- [x] Release workflow includes deterministically timestamped archives, checksum verification, extraction and installer smoke checks before upload.
- [ ] Native X11/Wayland screenshots and interaction walkthrough. *The headless captures do not satisfy this hardware gate.*
- [ ] Complete hardware test checklist (`QA.md`) with recorded results.
- [ ] Packaging/desktop integration acceptance.
- [ ] CI green on the rc.5 merge and prerelease commit; review warnings, errors, license/dependencies.

**Gate:** no P0/P1 defects, all regression tests passing and no misleading feature claims.

## 1.0 — Stable release

- [ ] Complete the hardware acceptance checklist on representative desktops.
- [ ] Record reference performance and limitations in release notes.
- [ ] Create a signed-off GitHub 1.0.0 tag/release with a reproducible Linux build.
- [ ] Mark 1.0 only after all required gates pass.

### After 1.0 (not blockers)

Optional per-GPU utilization (driver-specific), container/cgroup view, separate process inspector and more RiceBowl-specific window chrome. These must not burden the baseline runtime.

## Remaining 1.0 release gate

1. **Automated:** exact RC commit passes formatting, Clippy, tests, release build, headless JSON and process-signal integration checks in CI.
2. **GUI:** launch, resize, navigate, persist profiles and signal disposable processes on Mint/X11, KDE/Wayland and labwc/Wayland.
3. **Efficiency:** measure three GUI runs of RSS, idle CPU and startup at 500/1500/10000 ms; record kernel, system load and hardware.
4. **Packaging:** verify installed `.desktop` entry and release binary on representative sessions.
5. **Stable:** only after these gates pass, bump to `1.0.0` and create the stable release; until then maintain an accurately labelled prerelease.

### Why this architecture?

`monitor.rs` owns all Linux system reading; `actions.rs` handles narrowly scoped signals; `config.rs` owns persistent preferences; `ui.rs` only coordinates the display. RiceBowl remains an **optional launcher/theme source**, not a runtime dependency. A dedicated monitor service is unnecessary because Taskotto only runs when the user opens it.
