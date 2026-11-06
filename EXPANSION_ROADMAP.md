# Taskotto — expansion roadmap

Taskotto stays an **on-demand, unprivileged, Rust-only Linux process manager**.
RiceBowl is an optional theme/launcher integration, never a mandatory runtime
dependency. The goal is a genuine system monitor whose UI can be generated
and remixed through versioned, safe RicePacks.

This plan extends [the 1.0 release gates](ROADMAP.md), rather than silently
considering them complete. Release numbering beyond the current RC is a plan,
not a promise or schedule.

## Non-negotiable quality gates

Every milestone retains functional GUI and headless CLI, process-signaling
safety, X11/Wayland fallback, readable errors and working built-in rices.
No proprietary OS artwork, fake per-process network data, resident daemon,
silent privilege escalation or scripts inside RicePacks. Every change needs
automated tests, documentation and a green CI build.

## P0 — Complete the stable 1.0 hardware acceptance

- [x] Full Linux sampler, PID-safe actions, eight presets, headless scenes,
  release pipeline and installation path available in RC13.
- [ ] Test a real client on Mint/X11, KDE/Wayland and RiceBowl/labwc.
- [ ] Measure GUI startup, idle CPU and RSS at several polling intervals;
  test heavy process churn and several-hour soak behavior.
- [ ] Verify native RiceBowl window decorations, accessibility, keyboard,
  responsive layouts and real software-install/reinstall paths.
- [ ] Correct any P0/P1 issues, then perform an explicitly signed-off 1.0 tag.

**Gate:** actual measurements and [QA.md](QA.md) evidence; successful
headless screenshots alone cannot certify a 1.0 desktop release.
Development on later milestones may proceed while user-hardware QA is pending.

## P1 — RicePack contract and generator-friendly foundation

- [x] Specify versioned, data-only `taskotto-rice.toml` with validated
  base theme, colors, density, navigation and sizing tokens.
- [x] Allow `--ricepack FILE`, `RICEBOWL_TASKOTTO_RICEPACK` and a
  headless `--check-ricepack FILE` validation command.
- [x] Cap pack input size, reject malformed/unknown/unsupported fields,
  preserve existing user config and avoid any executable pack content.
- [x] Ship an original demonstration pack, tests and [format specification](RICEPACK_FORMAT.md).
- [x] Export saved appearance via `--export-ricepack FILE` to a new, safe,
  re-importable and strictly validated TOML file, without GUI launch or config writes.
- [x] Automatically reload changed valid RicePack files during GUI sessions,
  with an explicit warning and last-known-good fallback for broken edits.
- [x] Contract tests with a RiceBowl-created pack and a versioned integration
  fixture supplied by that repository.
- [x] Add pack discovery, in-app visual preview and non-destructive
  import/export from the Taskotto UI.
- [x] Extend the v1 design with backwards-compatible semantic colors for
  controls, hover, borders, tables and selection. Expose them in Preferences,
  validate them strictly and preserve them across preview/revert/export.
- [ ] Separate all palette/size tokens from hardcoded rice widget styles;
  make component overrides available without recompilation.

**Gate:** an AI-generated pack can be checked, previewed and applied without
Rust changes, while the legacy presets render and behave identically.

## P2 — Declarative screens (only after the P1 token contract is stable)

- [ ] Typed and bounded layout schema for lists, tabs, status strips,
  toolbars, gauges and plots with named data bindings.
- [ ] Theme capabilities and fallbacks for compact and large screens.
- [ ] User-driven layout editor and reusable component presets.
- [ ] Visual regression captures, accessibility/focus checks and pack
  migration diagnostics for schema upgrades.

**Gate:** build a new, visually distinct functional monitor in a RicePack
with **no new Rust UI module**. No untrusted scripts or arbitrary shell hooks.

## P3 — Advanced process controls

- [ ] CPU affinity, `nice` priority and I/O priority where Linux permissions
  permit, with clear errors and explicit consent for mutations.
- [ ] Aggregate descendant-tree CPU and memory statistics without double
  counting; show collection scope accurately.
- [ ] Optional compositor-specific app/window grouping with real identifiers,
  fallback to process lists when window information is unavailable.
- [ ] Session history, process-follow and sort-freeze controls.

**Gate:** no actions on PID reuse, no hidden root service, accurate
distinction between processes, GUI windows and logical applications.

## P4 — GPU and hardware telemetry

- [ ] Optional AMD, Intel and NVIDIA backends with capability detection.
- [ ] Show available GPU utilization, memory, clocks, temperatures and power.
- [ ] Per-process GPU use only where the corresponding driver exposes
  trustworthy process attribution.
- [ ] Battery/power information where available; distinguish unavailable
  metrics from zero.

**Gate:** feature and overhead tests across at least one AMD and one
non-AMD machine. No mandatory vendor SDK for the baseline binary.

## P5 — Container, cgroup and service monitoring

- [ ] Read cgroup v2 CPU, memory and I/O counters with correct aggregation.
- [ ] Show user systemd units and optional Podman/Docker associations.
- [ ] Recognize Flatpak/containerized processes where kernel metadata allows.
- [ ] Start read-only; consider safe service actions only behind explicit
  permission and confirmation boundaries.

**Gate:** host/container metrics are not mixed, no root daemon or false
attribution, graceful behavior when cgroups or container APIs are unavailable.

## P6 — Explain system slowdowns

- [ ] Use Linux Pressure Stall Information (CPU, memory, I/O) and actual
  historical samples to identify likely bottlenecks.
- [ ] Expose process/cgroup throttling, memory pressure and I/O congestion.
- [ ] Evidence-first diagnostic explanations; clearly label hypotheses.
- [ ] No claim of measuring the cause of a slowdown from a single percentage.

**Gate:** synthetic and live workloads demonstrate correct source data,
units, threshold behavior and no false certainty.

## P7 — Composable dashboards and exports

- [ ] Build per-user dashboards from the P2 component schema.
- [ ] Session-only history by default; opt-in bounded retention and export.
- [ ] CSV/JSON export with explicit timestamp, scope and collection interval.
- [ ] Time-series comparisons without a mandatory always-on daemon.

**Gate:** export/import/retention tests, low-memory history bounds and clear
permission/ownership rules.

## P8 — Performance, packaging and stabilization

- [ ] Measure GUI RSS, CPU, latency and process-list scaling on low-end and
  modern desktops before optimizing.
- [ ] Profile and optimize only measured hot paths (sampling, rendering,
  allocations and polling frequency).
- [ ] Optional reduced-animation/low-overhead profile, preserving functions.
- [ ] Reproducible packages, integrity metadata and installation recovery.

**Gate:** benchmarks accompany optimization claims; release builds remain
lean, offline-capable and independent from RiceBowl.

## Work order

1. **Delivered (RC14–19):** P1 format/loader/CLI, secure reference pack,
   headless export/check/re-import, live file reload and in-app library,
   session-only preview/revert/import/export, semantic widget tokens and a
   cross-repository, versioned RiceBowl export fixture.
2. **Next:** P1 component-level UI token extraction. RiceBowl's separate JSON
   desktop pack still is not Taskotto TOML; the explicit RiceBowl exporter is
   the validated conversion boundary.
3. **Then:** P2 data-bound layout schema; P3 real process controls; P4/P5
   optional telemetry backends; P6 diagnosis; P7 dashboards.
4. **Throughout:** P0 device acceptance and P8 measured resource costs.

The stable 1.0 release remains gated by physical desktop verification, even
when implementation work on the future feature track continues.
