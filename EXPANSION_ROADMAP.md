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
  - [x] RC20: Frutiger Aero custom tabs/sidebar, list rows, hover/selection,
    table and performance graph containers consume semantic component colors.
  - [x] RC21: macOS Activity Monitor tabs/sidebar, table headers, hover/selected
    rows use shared semantic component tokens with high-contrast foreground.
  - [x] RC22: ChromeOS's existing adaptive palette consumes the shared table,
    hover, border, control and selection tokens with honest row contrast.
  - [x] RC23: Windows 7's top tabs, sortable Processes/Applications reports,
    hover/selection, frame borders and empty states use shared component colors.
  - [x] RC24: shared XP Luna and Windows 10 navigation and process report,
    XP Applications, and Win10 optional sidebar consume semantic tokens;
    custom tables preserve readable Linux process metrics and heatmap signal.
  - [x] RC25: Windows 98 Close Program, optional advanced process rows and
    selected/hovered lists, raised/sunken beveled controls and borders honor
    shared semantic component colors without fake titlebar decoration.
  - [x] RC26: validated `appearance.chart_scale` adjusts native performance
    chart heights in the original rices without Rust changes.
  - [x] RC27: Windows 7 Users/Services, performance statistics and Properties;
    XP properties/networking/statistics; Win10 performance selection/chart;
    macOS process Info and Aero network chips honor semantic table/border
    colors with contrast-safe text and responsive layout regressions.
  - [x] RC28: historical macOS, Aero, Classic 98 and Windows 7/10/XP menu,
    navigation and status strip backgrounds consume the control/table palette;
    macOS timeline and network plots consume table/border tokens, with readable
    text even when a generated pack only chooses a dark global surface.
  - [x] RC47: Aero's secondary horizontal navigation, opt-in sidebar and
    process/performance hero are now palette-aware with contrast-safe text;
    chart plot outlines honor the border token. The original Y2K lens and
    meter artwork stays intentionally distinct when no override is supplied.
  - [x] RC55: Windows 10 App history text/hover rows and Windows 10/XP owner
    summary surfaces honor optional table/hover/border colors. Historical
    graph series colors remain intentional resource identity cues.
  - [ ] Audit remaining intentional artwork, complex per-widget charts and
    secondary controls for hardcoded palette tokens without homogenizing rices.

**Gate:** an AI-generated pack can be checked, previewed and applied without
Rust changes, while the legacy presets render and behave identically.

## P2 — Declarative screens (only after the P1 token contract is stable)

- [x] Typed and bounded layout schema for lists, tabs, status strips,
  toolbars, gauges and plots with named data bindings.
  - [x] RC29 foundation: strictly typed 1–4 screens with 1–16 widgets per
    screen; named live Linux metrics, session history plots, original headings
    and virtualized PID-safe process lists. Malformed/oversized/unknown fields
    fail CLI validation. An AI-authored TOML pack can build and switch a new
    two-screen functional client without theme-specific Rust changes.
  - [x] RC30: bounded, data-bound status strips, CPU/memory gauges and a
    toolbar limited to refresh/search/confirmed process termination. Unknown,
    duplicate or script-like actions are rejected, and missing samples are
    honestly unavailable rather than displayed as fake zeroes.
- [x] Responsive screen capabilities and fallbacks for compact and large screens:
  RC31 adds an opt-in validated breakpoint and per-screen alternative widget
  composition; the legacy layout renders unchanged when no fallback is defined.
- [x] User-driven layout editor and reusable component presets: RC32 adds a
  session-only draft with validated apply, bounded screen/widget edits,
  binding controls, explicit compact alternatives, accessible reorder buttons,
  and seven safe templates; export uses the existing non-overwriting RicePack path.
- [ ] Visual regression captures, accessibility/focus checks and pack
  migration diagnostics for schema upgrades.
  - [x] RC33: decode/verify all 50 genuine egui image captures, test generated
    Ctrl+F focus with and without a search toolbar, and report legacy/future
    pack version incompatibilities before parsing optional widget fields.
  - [ ] Native keyboard traversal, screen-reader and screen-size acceptance on
    Mint/X11, KDE/Wayland and RiceBowl/labwc; compare captures after refinement.

**Gate:** build a new, visually distinct functional monitor in a RicePack
with **no new Rust UI module**. No untrusted scripts or arbitrary shell hooks.

## P3 — Advanced process controls

- [ ] CPU affinity, `nice` priority and I/O priority where Linux permissions
  permit, with clear errors and explicit consent for mutations.
  - [x] RC35: display actual leader Nice priority and permitted CPU list from
    existing procfs sampling, with explicit thread scope and unavailable states.
  - [ ] Implement any mutation only after an auditable PID-identity/race
    strategy, explicit confirmation and permitted/unprivileged error tests.
- [ ] Aggregate descendant-tree CPU and memory statistics without double
  counting; show collection scope accurately.
  - [x] RC34: count each observed descendant PID once, prevent cycles and PID
    reuse at the selected root, sum actual per-process CPU/RSS with saturation
    and label limited visibility and shared-RSS page overlap explicitly.
  - [x] RC38: explicit asynchronous `/proc/PID/smaps_rollup` PSS inspection
    across up to 128 observed descendants, PID+start-tick verified, reporting
    partial permission coverage instead of inventing unique RSS.
  - [ ] Real-hardware PSS-vs-shared-memory verification, with and without
    procfs permissions and during rapidly exiting descendant processes.
- [ ] Optional compositor-specific app/window grouping with real identifiers,
  fallback to process lists when window information is unavailable.
- [ ] Session history, process-follow and sort-freeze controls.
  - [x] RC36: session-only sort freeze, keyboard toggle, PID+start-time identity,
    stable new-process insertion, and tree-parent preservation.
  - [ ] Time-bounded process history and follow-selection with explicit expiry.
  - [x] RC37: opt-in process-follow with 120-sample CPU/RSS trends in the
    generic inspector and Preferences, expiry after repeated missing samples,
    and PID+start-time isolation; no persisted data.
  - [x] RC48: session-only, timestamp-filtered 30-second and 2-minute
    process-follow views or all retained observations, with correct peaks,
    missing-timestamp exclusion and narrow-screen stacked graphs; sampling
    retention remains 120 observations and no new daemon is started.
  - [x] RC50: explicit private CSV/JSON export of the opted-in followed PID,
    start ticks, timestamps, CPU and RSS; new-file-only, `0600`, no commands
    or names, no background persistence. RC45's separate host-aggregate export
    still excludes process identities.
  - [ ] Native lifetime/follow/export acceptance remains open.

**Gate:** no actions on PID reuse, no hidden root service, accurate
distinction between processes, GUI windows and logical applications.

## P4 — GPU and hardware telemetry

- [ ] Optional AMD, Intel and NVIDIA backends with capability detection.
  - [x] RC39: safe DRM device discovery and native AMDGPU busy/VRAM sysfs
    counters, plus optional hwmon temperature/power/frequency and Intel GT
    frequency with truthful per-device capability gaps.
  - [ ] Intel/NVIDIA utilization and additional supported driver-specific
    metrics, with real hardware and permission tests.
- [ ] Show available GPU utilization, memory, clocks, temperatures and power.
  - [x] RC39: display all supported sampled AMDGPU values in System
    Performance and every rice's Preferences; missing values remain unknown.
- [ ] Per-process GPU use only where the corresponding driver exposes
  trustworthy process attribution.
- [ ] Battery/power information where available; distinguish unavailable
  metrics from zero.
  - [x] RC40: battery/adapter discovery via Linux power_supply, with actual
    status, capacity %, energy Wh, power W and no mixed electrical units.
  - [ ] Cross-vendor real laptop, dual-battery and AC hot-plug acceptance.

**Gate:** feature and overhead tests across at least one AMD and one
non-AMD machine. No mandatory vendor SDK for the baseline binary.

## P5 — Container, cgroup and service monitoring

- [ ] Read cgroup v2 CPU, memory and I/O counters with correct aggregation.
  - [x] RC41: explicit selected-process cgroup v2 inspector for actual
    membership, cumulative group CPU/throttling, memory current/max and
    aggregated per-device I/O, with PID/membership revalidation.
  - [x] RC49: read-on-demand cgroup-local `cpu.pressure`, `memory.pressure`
    and `io.pressure` SOME/FULL with kernel parser, accurate 10/60/300-second
    stall percentages and cumulative microseconds; absent sources unavailable,
    never host/cgroup scope conflation. Live workload QA still open.
  - [x] RC53: bounded direct-parent/16-child cgroup v2 neighborhood in the
    on-demand inspector, at most 128 directory entries, explicit truncation
    and separate cumulative CPU/memory values. Not a recursive tree or rate.
  - [x] RC54: on-demand two-snapshot rate comparison for the same selected PID,
    cgroupfs inode and hierarchy path; real monotonic CPU and I/O counter
    deltas, with reset/missing/too-fast samples unavailable. Not a daemon.
  - [ ] Multi-cgroup and nested-hierarchy overview, dynamic group sample
    rates, container host/namespace accounting and native stress QA.
- [ ] Show user systemd units and optional Podman/Docker associations.
  - [x] RC46: selected-process cgroup inspection now displays **unverified**
    systemd .service/.scope segments and strict Docker/Podman 64-hex scope
    ID prefixes from kernel-reported membership, plus Flatpak-style scopes.
    Real systemd and runtime API resolution remain open and optional.
- [ ] Recognize Flatpak/containerized processes where kernel metadata allows.
  - [x] RC46: bounded, read-only cgroup path hints; no API-based app identity
    or runtime guarantee, no hidden process/container controller.
- [ ] Start read-only; consider safe service actions only behind explicit
  permission and confirmation boundaries.

**Gate:** host/container metrics are not mixed, no root daemon or false
attribution, graceful behavior when cgroups or container APIs are unavailable.

## P6 — Explain system slowdowns

- [ ] Use Linux Pressure Stall Information (CPU, memory, I/O) and actual
  historical samples to identify likely bottlenecks.
  - [x] RC42: truthful system-level CPU SOME and memory/I/O SOME/FULL PSI,
    10/60/300 s averages and cumulative stall time, with
    compact UI and no false utilization or unsupported field defaults.
  - [ ] Correlate PSI trends with process/cgroup pressure and actual resource
    history before emitting bounded, evidence-based bottleneck hypotheses.
  - [x] RC43: per-source bounded 120-point PSI trends for CPU SOME,
    memory SOME/FULL and I/O SOME/FULL; no fabricated gap zeros or false
    causal attribution.
- [ ] Expose process/cgroup throttling, memory pressure and I/O congestion.
  - [x] RC41 and RC49: selected-cgroup cumulative CPU throttling, memory
    current/max, I/O bytes and cgroup-local PSI stalls on demand. Real-time
    rates, per-process attribution and controlled-load QA remain open.
- [ ] Evidence-first diagnostic explanations; clearly label hypotheses.
  - [x] RC44: non-actionable CPU, memory and I/O investigation hints require
    repeated PSI readings from eight consecutive monitor samples plus a
    supporting 60-second average. Missing ticks cannot imply continuity;
    explain observed numbers and suggested *human* checks, never a root cause.
    Native workload/false-positive acceptance remains outstanding.
- [ ] No claim of measuring the cause of a slowdown from a single percentage.

**Gate:** synthetic and live workloads demonstrate correct source data,
units, threshold behavior and no false certainty.

## P7 — Composable dashboards and exports

- [x] Build per-user dashboards from the P2 component schema.
  - [x] RC56: opt-in persisted path to a strictly validated user-owned
    P2 RicePack makes personal multi-page dashboards available on restart,
    with explicit precedence, in-app clearing and recoverable missing-file
    fallback. Layout editing and importing are provided by the RC32 editor
    and RC17 library; individual samples remain session-only.
- [ ] Session-only history by default; opt-in bounded retention and export.
  - [x] RC45: session RAM history stays capped at 120 system ticks, with
    explicit non-overwriting CSV/JSON export from every rice's Preferences;
    no process identities or implicit retention.
- [ ] CSV/JSON export with explicit timestamp, scope and collection interval.
  - [x] RC45: host-aggregate export includes per-observation Unix milliseconds,
    nominal polling interval and measured CPU, RAM, network, disk and PSI;
    absent pressure sources remain null/blank, not zero.
- [x] Time-series comparisons without a mandatory always-on daemon.
  - [x] RC55: RicePack v1 declarative screens can graph real CPU/RAM %,
    network RX/TX B/s or disk read/write B/s together on a shared scale,
    using only the existing capped in-memory sampler. The new pack has
    a compact fallback and normal process actions; native QA remains open.

**Gate:** export/import/retention tests, low-memory history bounds and clear
permission/ownership rules.

## P8 — Performance, packaging and stabilization

- [ ] Measure GUI RSS, CPU, latency and process-list scaling on low-end and
  modern desktops before optimizing.
- [ ] Profile and optimize only measured hot paths (sampling, rendering,
  allocations and polling frequency).
- [x] RC51: optional reduced-animation/low-overhead profile, preserving all
  functionality and the preferred sampling interval; opt-in minimum 3-second
  worker cycle, no new daemon. Measured impact still requires physical-device QA.
- [x] RC52: deterministic artifact packaging test, existing SHA256 release
  integrity metadata, user-local reinstall repair and safe `--uninstall-desktop`
  recovery. Cross-host Rust binary reproducibility and native launcher QA are
  still release acceptance work, not claimed results.

**Gate:** benchmarks accompany optimization claims; release builds remain
lean, offline-capable and independent from RiceBowl.

## Work order

1. **Delivered (RC14–33):** P1's validated RicePack format, live reload,
   in-app preview/export and semantic component tokens; P2's typed functional
   screens, real Linux bindings, seven safe widget kinds, responsive compact
   layouts and session-only visual designer. All 50 genuine egui captures are
   integrity-checked. RiceBowl's separate JSON desktop pack still requires
   its explicitly tested TOML exporter bridge.
2. **Next:** Finish the P1 intentional historical artwork/palette audit and
   the P2 native accessibility, visual regression and keyboard QA on actual
   Linux desktop sessions.
3. **Then:** P3 advanced process controls; P4 GPU/hardware telemetry; P5
   cgroups/services; P6 evidence-based diagnosis; P7 dashboards/exports.
4. **Throughout:** P0 physical-device acceptance and P8 measured resource
   costs, packaging recovery and stability gates.

The stable 1.0 release remains gated by physical desktop verification, even
when implementation work on the future feature track continues.
