# Taskotto 1.0.0-rc.55 — release candidate

## RC55 — Comparable trends and historical palette consistency

- Data-only v1 RicePacks and the native Screen layout designer now support three additional paired history plots. CPU/RAM share a 0–100% y-axis; network receive/send and disk read/write share one actual observed bytes-per-second axis. Each pair has a visible legend and scale, uses the already bounded session history, and requires no new background polling or permission.
- Bundled Comparison Studio has a responsive compact alternative and existing safe process tools; the new headless screenshot is genuine Taskotto egui output with live Linux observations. Windows 10 App history and XP/Windows 10 process-owner surfaces also consume validated optional table/hover/border colors while preserving original fallbacks.
- v1 RicePack schema remains backwards-compatible and executable content remains impossible. Tests verify new bindings, strict validation, round trips and chart scaling. **Not stable 1.0:** all native X11/Wayland/labwc display, accessibility, lifetime, driver, installation and GUI resource-use gates still require measured acceptance on the user's devices.

## RC54 — Re-inspect to compare cgroup rates

- Repeating a selected-process cgroup v2 inspection compares two observed kernel snapshots (CPU used relative to one core; block I/O read/write KiB/s) over their actual monotonic interval. The second reading must belong to the exact same cgroupfs inode and membership path, with the same PID/start identity guarded by the inspector.
- Counter reset, a disappeared controller, a different/recreated group and less than 250 ms between observations yield an unavailable result, never a misleading rate. No daemon or periodic cgroup sampling was added.
- Added unit tests for actual units, positive counter deltas, missing counters, reset, inode/path replacement and too-short intervals. Native cgroup workload acceptance is still pending.


## RC53 — Bounded parent/child cgroup context

- The selected-PID cgroup inspector now offers a collapsible snapshot of the direct parent and up to 16 immediately nested cgroups. Their kernel-provided cumulative CPU and current-memory values are shown separately, with no extra daemon or privileged API. Enumeration checks at most 128 entries and marks truncated or unavailable views explicitly.
- Parent/child counters can overlap and must not be summed. The complete selected-process PID/start and membership revalidation from RC41 remains mandatory. A one-time neighborhood snapshot is not a continuous cgroup history or verified container/service inventory.
- Added hermetic cgroupfs fixtures for nested groups, bounded scanning, ordering, missing groups and unavailable metrics. Native multi-container acceptance remains open.


## RC52 — User-local desktop installation recovery

- Added an explicit `taskotto --uninstall-desktop` command. Only entries matching Taskotto's own launcher template and exact icon are deleted; changed files and symlinks are rejected and no other app files, preferences or binary are touched. Reinstall with `--install-desktop` repairs the menu path after a binary move.
- Release packaging adds a second-byte-identical deterministic archive check and a CLI install/uninstall smoke test, in addition to the existing SHA256 integrity metadata. Cross-machine compiled binary reproducibility remains unverified.
- Unit tests cover normal, repeated, customized and symlinked removals; CLI tests cover standalone invocation and rejecting mixed commands. Native menu verification remains required for stable 1.0.


## RC51 — Opt-in low-overhead monitoring

- Preferences → Monitoring adds an optional low-overhead mode. When enabled, the existing sampler runs at a minimum 3-second interval and native UI style animations are removed. Every view, safety confirmation and process action remains available; CPU/RAM savings are not claimed without hardware measurement.
- The saved preferred refresh period is not changed. Switching the mode off immediately restores it. Both session and followed-process exports use the actual nominal monitoring interval. Old configurations keep normal behavior.
- Added regression tests for the legacy default, persistence, effective period and returning from reduced animation to normal styling. Native measured idle-power/CPU acceptance is still outstanding.


## RC50 — Optional private followed-process history export

- The process-follow control in Details and Preferences now offers a separate explicit CSV/JSON export to a new absolute file with Unix `0600` permissions. Only the selected follow's numeric PID, start-tick identity, recorded timestamp, CPU percentage and RSS MiB enter the file; the process name, command, other processes and unrelated host histories are not included.
- Every row is labeled with schema version, process scope and nominal interval. Missing timestamps remain missing, no existing file is overwritten and data never persists without a deliberate click. Export continues to refer to the followed PID after changing the selection; the display identifies that target.
- Added pure formatting, missing-data, identity, no-overwrite and permissions tests. Native user interaction/permission acceptance remains open.

## RC49 — Real cgroup v2 pressure readings

- The explicitly requested, asynchronously loaded selected-process cgroup report now includes real **cgroup-local** `cpu.pressure`, `memory.pressure` and `io.pressure` SOME/FULL rows, presenting kernel 10/60/300-second rolling stalled-time percentages and cumulative total seconds. Cgroup-level CPU FULL is included when supported; unlike the system-wide CPU FULL field, it has a meaningful workload-specific interpretation. See [Linux PSI documentation](https://docs.kernel.org/accounting/psi.html).
- Validates metrics with the existing strict PSI parser; absent controllers, denied files and explicitly disabled `cgroup.pressure` accounting are shown as unavailable. The cgroup scope is always labeled and never attributed to one PID or mixed with system-wide metrics. No new polling loop, daemon, root privilege or mutable service API.
- Added deterministic data-source and missing/NaN parsing tests and kept the PID/start-tick/membership revalidation. Host/container comparison and controlled load acceptance remain open.

## RC48 — Accurate, selectable process-follow time windows

## RC48 — Accurate, selectable process-follow time windows

- Opt-in process-follow CPU/RSS charts now offer the actual elapsed last **30 seconds**, **2 minutes**, or **all retained observations**. They use each observed sample's GUI-receipt Unix timestamp, not fabricated evenly spaced time or a fixed samples-per-second assumption; window peaks update to match only displayed samples.
- Missing timestamps and sampler gaps never become zero. Charts stack below 640 px; the PID and start-tick identity gate, three-miss process expiry, 120-sample RAM bound and no persisted process data remain unchanged.
- Added regression tests for inclusive time boundaries, missing readings, window filtering and retention. The real-device lifetime/accessibility QA gate still needs physical verification.
- Updated the process-follow README gallery screenshot through the real Taskotto headless renderer, and documented the existing one-file screenshot filter for faster visual QA.

## RC47 — Frutiger Aero visual polish for custom RicePacks

## RC47 — Frutiger Aero visual polish for custom RicePacks

- The Frutiger Aero horizontal navigation, opt-in sidebar, prominent layered hero background, title/subtitle text and performance plot strokes now use the same validated semantic component palette as its tables and action buttons.
- Dark/generated RicePacks receive readable hero and sidebar text; the original aqua/sky, translucent vector lens and circular CPU/RAM indicators remain visually unique to Aero. Default fallback colors are preserved, without adding window-frame elements (RiceBowl/compositor still own them).
- Re-rendered all eight Aero screenshots (including both Aero Lagoon captures) from current Taskotto UI paint output, preserving the 51-image README gallery and capture provenance.
- This addresses an additional P1 hardcoded-surface seam; remaining historical artwork/complex widget audits and native fidelity remain open. Screenshots are genuine Taskotto egui captures, not reconstructed static mockups.

## RC46 — Read-only service and container naming hints

## RC46 — Read-only service and container naming hints

- The existing explicitly requested cgroup inspector now highlights plausible systemd .service/.scope paths, strict Docker/Podman container-ID scope conventions (12-character prefix only) and Flatpak-style scope names. It derives these solely from the selected PID's kernel-verified cgroup v2 membership and never contacts a systemd, Podman or Docker API.
- All associations are **unverified naming hints**, not proof that a specific service or container owns the process. No service/container controls, elevated privileges or background discovery. Parser regression tests cover false positives, control bytes and path ambiguity; native runtime comparisons are still pending.

## RC45 — Explicit session history export

## RC45 — Explicit session history export

- Added a user-initiated CSV/JSON export control to the common Preferences surface in every rice. It records at most 120 measured **Linux host aggregate** samples in RAM; never saves automatically, and creates a new file with `0600` permissions and no overwrite of an existing destination.
- Rows provide GUI-receipt Unix timestamps in milliseconds (not exact kernel read times), CPU/RAM percentages, network and block I/O bytes/s, nullable CPU/memory/I/O PSI 10-second stalled-time percentages, format version, sampler-visible scope and **nominal** configured interval. Real sampling can drift and a container's system view may be restricted; no process identities are exported and absent PSI is never converted to zero.
- Added deterministic export schema, missing-value, alignment, unsupported-path, no-overwrite and permission regressions. Native live sampling/export acceptance, editable dashboards, historical comparisons and persisted history remain future P7 work.

## RC44 — Evidence-labeled Linux slowdown investigation

## RC44 — Evidence-labeled Linux slowdown investigation

- System Performance and all rice Preferences now show tentative CPU contention, memory stalls or I/O stall hints only when kernel-reported PSI is sufficiently sustained. The display requires at least six of eight consecutive sampler readings over chosen presentation thresholds and supporting 60-second rolling evidence. Missing ticks break the evidence chain.
- Each hint cites its measured 10/60-second numbers and points to non-destructive follow-up checks (process list, cgroups, memory availability, block I/O). It never attributes a cause to a particular task, container or disk; thresholds are **heuristics**, not kernel definitions or performance guarantees.
- Added deterministic gap, uncertainty, resource-mix and insufficient-data tests. Real controlled-workload acceptance and cross-source attribution are **not completed**.

## RC43 — Linux pressure history

**Date:** 2026-11-07 · **Platform:** Linux x86_64 development/CI build; X11/Wayland GUI hardware acceptance pending.

Rust-only Linux process manager and system monitor. This is a **prerelease**, not a stable 1.0 certification.

## Changes since rc.42

- Added bounded real PSI history for CPU SOME, memory SOME/FULL and I/O SOME/FULL. System Performance and all rice Preferences draw live trend charts alongside the existing 10/60/300-second source readings, with a compact single-column presentation on narrow windows.
- Retains at most 120 **valid observations per PSI source** in memory. Kernel PSI values are already overlapping rolling 10-second averages; these graphs visualize their changes at the configured Taskotto sampler interval, not independent events or proven bottleneck causes. Missing samples never become fabricated zeros; they do not claim uniform elapsed time across collection gaps.
- Extended regression tests for the 120-sample bound and absent PSI readings, with the same no-root/no-new-dependency data flow. Multi-workload correlation, explicit diagnostic hypotheses and native acceptance remain open.

## Changes since rc.41

- Began P6 pressure diagnostics with genuine Linux PSI readings from `/proc/pressure/cpu`, `memory` and `io`. The monitor captures CPU SOME, memory SOME/FULL and I/O SOME/FULL stalls as distinct 10s, 60s and 300s percentages plus cumulative stall microseconds.
- These metrics now appear in System Performance and all rice Preferences, with a compact layout for narrow screens. PSI measures **stalled-task time**, not CPU load, memory occupation or disk activity. System-level CPU FULL is excluded; missing kernels and invalid/NaN/out-of-range readings remain unavailable rather than zero.
- Added strict parser and best-effort Linux PSI tests. Automated slowdown diagnoses and hardware workload/false-positive acceptance remain future P6 work; Taskotto does not claim a cause from a single pressure reading.

## Changes since rc.40

- Started P5 with an on-demand, asynchronous **cgroup v2 inspector** for the selected Linux PID. It reads true unified cgroup membership, cumulative `cpu.stat` usage/throttling, `memory.current`, `memory.max` (including unlimited) and aggregates device-level `io.stat` read/write bytes.
- Revalidates the PID's original start tick and membership after reading, rejects unsafe/escaping cgroup paths, and distinguishes unavailable controller data from actual zero. Results explicitly describe **cgroup-wide** counters, not per-process or container API metrics; unsupported cgroup v1/namespace mounts show an error, not a crash.
- Added pure path/traversal/CPU/I/O parsing tests and a best-effort own-process probe. Available through generic Details and every rice's Preferences, without privilege escalation, background daemon or process mutations. Native cgroup/container tests remain outstanding.

## Changes since rc.39

- Completed the first P4 power-supply slice: Linux battery inventory and external-adapter status from `/sys/class/power_supply`, including genuine battery percentage, charging status, energy in Wh and current power in W when the driver provides them.
- Preserved the kernel's distinction between `energy_*` (micro-watt-hours) and `charge_*` (micro-amp-hours); fallback charge percentages never masquerade as physical energy or invented remaining runtime. Absent battery/adapter sensors produce truthful unavailable states.
- Added deterministic synthetic laptop/adapter, missing-sysfs, unit-conversion and malformed-sensor tests. System Performance and every rice's Preferences display the data without an extra daemon, privilege or dependency. Laptop and multi-battery acceptance remains pending.

## Changes since rc.38

- Began P4 with optional, unprivileged GPU discovery from Linux DRM sysfs. AMDGPU drivers expose real `gpu_busy_percent` and VRAM usage/total bytes where available; hwmon and GT capabilities supply temperature, average power and core clock in their documented units. No mandatory SDK, root service or new dependency.
- All detected AMD, Intel and NVIDIA DRM cards appear in the System Performance page and Preferences of every rice, but **missing** driver metrics explicitly say unavailable. Intel/NVIDIA utilization and per-process GPU usage are not invented; unsupported kernels/no `/sys/class/drm` produce an honest empty state.
- Deterministic synthetic-sysfs tests verify units, VRAM bounds, matching DRM cards (not output connectors), AMD capability detection and Intel/NVIDIA unavailable states. Physical AMD/Intel/NVIDIA acceptance and sampling overhead benchmarks are pending.

## Changes since rc.37

- Added opt-in **Measure PSS now** for a selected Linux process and its visible descendants. A bounded background worker reads kernel `/proc/PID/smaps_rollup` proportional set sizes; the normal on-demand sampler does not acquire expensive memory-map reads.
- The worker processes at most 128 distinct observed PIDs and confirms each PID's original start ticks before and after reading. Inaccessible/exited entries are counted as **unavailable**; the UI distinguishes complete from partial PSS, never reports partial sums as a complete unique-memory total and discloses that maps may change during collection.
- Unit tests cover strict kernel PSS field parsing, overflow/units, stale PID rejection, tree-size cap and optional real-process reading without demanding elevated permissions. X11/Wayland/labwc and shared-page physical QA remain pending.

## Changes since rc.36

- Added an **opt-in, strictly session-only process follow** control in generic Details and Preferences. It graphs up to 120 genuine per-process CPU and resident-memory samples with no background process service or disk writes.
- Samples must match the **PID and original process start ticks**. A temporarily unavailable process does not generate invented zero samples; after three consecutive misses the follow ends and its bounded history remains visible until cleared. Tracking never silently resumes for a recycled PID.
- Added a genuine headless screenshot of the generic inspector watching an actual disposable Linux process, plus lifecycle/limit regression tests. Hardware keyboard and desktop acceptance remain pending.

## Changes since rc.35

- Process sorting can now be temporarily **frozen** while live CPU, memory and I/O data keep updating. Use **Ctrl+Shift+Space** in any rice, the plain process table's **Freeze order** button, or **Preferences → Process table**. The preference is deliberately session-only; restarting resumes normal sorting.
- New processes append deterministically; a recycled numeric PID cannot take the previous process's slot because the freeze records PID **and process start ticks**. Tree mode still keeps descendants under their actual observed parent, and unfreezing resumes configured sorting immediately.
- Added regression coverage for live metric changes, PID reuse, newly appearing processes and tree hierarchy. No background sampler, process mutation or persistent settings schema was added.

## Changes since rc.34

- Read-only Linux scheduling diagnostics now show `nice` and permitted CPU-list masks of each process's **leader thread** in generic Details, XP Luna Properties and macOS Info. Values come from already sampled procfs metadata and do not grant any additional permission.
- Unavailable affinity is explicitly marked unavailable; negative Nice values are parsed correctly. These are not process-wide guarantees when other threads have separate affinity/priority, and privilege-sensitive controls remain unimplemented pending safe PID targeting.
- Updated genuine historical Info/Properties screenshots and added parser/real-proc tests.

## Changes since rc.33

- Added observed Linux process-tree totals to generic Details, XP Properties and macOS Info: unique PID count, summed per-process CPU and summed RSS, using a single snapshot, safe start-tick matching and cycle guards.
- The UI explains the exact scope: only observed descendants; RSS is not unique physical memory and can overlap because of shared pages. This is the first read-only P3 step, not completion of P3's future PSS-aware accounting or privilege-sensitive actions.
- Added duplicate/cycle/stale PID and overflow tests; refreshed historical Details/Info captures and roadmap/QA documentation.

## Changes since rc.32

- `Ctrl+F` reliably opens and focuses a real process search input in TOML-generated screens; a minimal no-toolbar pack gets an accessible keyboard-driven fallback. Hiding search no longer leaves invisible filtering.
- `--check-ricepack` now reports the fully validated page/widget/compact capabilities. Incompatible, missing and future schema versions get explicit no-auto-migration diagnostics instead of being masked by unrecognized experimental fields.
- Automated coverage checks keyboard search focus and decodes every screenshot in the 50-image gallery, verifying dimensions and nonblank renders. Physical Linux display-server, assistive technology and focus-order acceptance remains open.

## Changes since rc.31

- Preferences now contains a real screen layout designer built on the same strictly validated RicePack v1 schema. Create/edit up to four pages, change all supported widget labels and data bindings, reorder and remove widgets, add one of seven safe widget templates, and maintain a separate narrow-screen variant.
- Apply only validated drafts to the current session; discard invalid work or reset the draft without overwriting any files. Existing **Export current appearance** writes a new, re-importable TOML RicePack via the existing no-clobber path.
- New screenshot shows the designer's true egui layout in Taskotto's Frutiger Aero client. Additional manual UI/focus and cross-desktop acceptance are required before stable 1.0.

## Changes since rc.30

- Generated TOML screen layouts support an optional validated compact breakpoint and explicit per-screen compact widget fallback. Taskotto switches scenes as the available content width changes; older RicePacks behave exactly as before.
- Diagnostic Studio demonstrates functional small-screen layouts for Overview and I/O without multiplying source-code UI modules. Added a genuine 720×800 compact capture and schema/round-trip checks; the gallery now includes 49 captures.
- The graphical layout editor, deeper visual/keyboard QA and native desktop acceptance remain outstanding.

## Changes since rc.29

- Finished the first bounded screen-component palette in P2: generated TOML layouts now include percentage gauges (actual CPU/RAM), status strips (real named Linux metrics) and a restricted functional toolbar for refresh, process search and PID-safe confirmed termination.
- Reject unknown, duplicated or oversized toolbar/status values; missing samples show a loading state. Search opens with focus and separate process lists avoid duplicating the toolbar's End Process control.
- Refreshed both Diagnostic Studio screenshots, schema tests and docs. In-app layout editing, theme capability fallbacks and physical desktop QA are still outstanding.

## Changes since rc.28

- Added the first P2 declarative client-screen renderer and safe typed TOML schema. A RicePack can now supply 1–4 screen tabs with headings, named real Linux metric bindings, measured history plots and virtualized process lists with selected columns. PID+start-tick process actions still use the existing explicit confirmation flow.
- Bundled the original two-screen `Diagnostic Studio` data-only layout and automatic validation; preview/revert/export/live reload carry the layout only in the active RicePack session. Existing original rice layouts remain unchanged when the optional `[layout]` is absent.
- Tests cover strict schema limits, unknown/script-like fields, round-trip/no preference leakage, CLI validation and real-snapshot rendering of both new screens at compact and wide widths. The P2 editor, additional widget types and native GUI hardware acceptance remain open.

## Changes since rc.27

- Historical menu, navigation, status and toolbar strips in XP Luna, Windows 7/10, Classic 98, macOS and Frutiger Aero use validated RicePack control/table and border colors, with contrast-appropriate foreground. Default rices retain original strips and drawing; the window frame remains the compositor's responsibility.
- macOS live summary and network sparkline chart surfaces adopt table/border tokens. Dark custom global surfaces no longer force white text on fixed light strips where local foreground is needed.
- Regression rendering now checks dark custom statusbars at compact and desktop widths and an all-theme case with only dark global surface/text customization. Native display-server acceptance and further declarative layout work remain open.

## Changes since rc.26

- Extended validated semantic RicePack table/border/selection colors into Windows 7 Users/Services, performance statistics and properties inspector; XP properties, performance and networking; Windows 10 performance selectors and focused chart; the macOS Info dialog; and Frutiger Aero network metric chips.
- Added contrast-aware text where dark generated table backgrounds cover historical light panels, preserving every original built-in palette in the absence of overrides. Automated layout regressions now exercise dark component tokens on compact and large historical secondary pages.
- The remaining widget-level palette audit, future declarative screen layout API and required native desktop hardware acceptance remain open. No stable 1.0 certification is implied.

## Changes since rc.25

- Added strict session-safe `appearance.chart_scale` (0.75–1.5) and a Preferences slider. AI-authored packs can change genuine chart heights in XP Luna, Windows 7/10, Frutiger Aero, macOS and shared views; built-in sizes are unchanged without overrides.
- Export/import, preview/revert, live reload and persistent settings preserve chart scaling. Invalid/nonfinite pack values are rejected; ordinary preferences normalize them. Process permissions and sampling behavior are unchanged.
- Secondary-page palette extraction and the P2 layout schema remain open, as do real Mint/X11, KDE/Wayland and RiceBowl/labwc QA gates. Existing screenshots deliberately show original chart sizes.

## Changes since rc.24

- Classic 98's real Close Program process list, optional advanced report, selected/hovered rows, beveled buttons, sunken borders and focus outlines now accept the shared RicePack semantic colors. Built-in original grayscale defaults and unprivileged PID-safe actions remain intact.
- The included Violet pack and genuine headless egui screenshot bring the gallery to 46. Custom dark list fills get readable text, while RiceBowl/the window manager still owns native window decorations.
- All seven custom rice clients now support component colors in their primary process/task UI. P1 sizing and remaining secondary pages, P2+ features and real Mint/X11, KDE/Wayland, RiceBowl/labwc acceptance remain open; no stable release is certified.

## Changes since rc.23

- XP Luna and Windows 10 now share validated semantic RicePack colors in their custom top tabs and process tables. XP Applications and Windows 10's optional vertical navigation also honor component tokens without altering their historical no-override appearance.
- On customized dark table fills, the real per-process Windows 10 CPU/memory utilization heatmap remains visible, with per-cell contrast-aware text; Linux telemetry, sorting and process actions are unchanged.
- Added the original `xp-midnight.toml` and `win10-indigo.toml` examples with two real headless egui screenshots (45 gallery captures) and token/heatmap geometry tests. Classic 98 widget/size customization and native Mint/X11, Fedora/Wayland, RiceBowl/labwc acceptance still remain open.

## Changes since rc.22

- Windows 7 custom task-manager navigation and Applications/Processes tables now honor the shared RicePack control, table, hover, selection and border colors. Without overrides, historical layouts and palette remain unchanged.
- Added an original Dusk design pack and a live-egui screenshot of the dark selected process table (43 gallery captures). The Services/Users plain grids continue using their existing readable background.
- Windows 98, XP Luna and Windows 10 component extraction, advanced layouts and native desktop hardware acceptance remain open; stable 1.0 is not certified.

## Changes since rc.21

- ChromeOS's custom task list now honors semantic RicePack colors for table, hover, selection, control and border, using the shared v1 color parser. Selected and dark table rows get readable foreground without changing default light/dark profiles.
- Bundled a new `chromeos-nocturne.toml` example with a genuine headless egui capture, bringing the complete screenshot gallery to 42.
- Other Windows-inspired rice clients and advanced user-defined layouts remain on the roadmap; native Mint/X11, KDE/Wayland and RiceBowl/labwc hardware testing still blocks a stable 1.0 tag.

## Changes since rc.20

- Shared one validated `ComponentPaint` parser between Frutiger Aero and macOS Activity Monitor's custom client widgets, keeping their historical no-override appearance unchanged.
- Activity Monitor now honors RicePack component tokens on navigation, table headers, hover/selection and process rows with contrast-safe text. The original Nightshift example and authentic egui screenshot bring the total gallery to 41.
- P1 still includes the remaining rice widget styles and richer sizing/layout foundations; stable 1.0 remains gated on native Mint/X11, KDE/Wayland and RiceBowl/labwc QA.

## Changes since rc.19

- Frutiger Aero custom-drawn navigation, process tables and chart surfaces now accept RicePack v1 semantic component colors rather than bypassing shared token settings. Missing tokens retain historical built-in colors.
- Included the `examples/ricepacks/aero-lagoon.toml` palette with bright controls and a contrasting dark process table; added genuine egui/headless process and performance captures (40 screenshots total) and default-versus-custom token tests.
- Documented visual and native interaction owners in `DESIGN.md` and `UX-CONTRACT.md`. No new dependency or permission is required; remaining rices still need equivalent token extraction, and native hardware QA remains a blocker for stable 1.0.

## Changes since rc.18

- Verified the first RiceBowl-to-Taskotto bridge end-to-end using RiceBowl's versioned `de taskotto-export` fixture: the generated TOML is parsed in a standalone Rust integration test, and Taskotto's production `--check-ricepack` CLI accepts it without a GUI or RiceBowl installation.
- Documented manual opt-in export/launch commands, preserving all user preferences and keeping desktop recipe JSON/ZIP separate from Taskotto's strictly checked TOML design contract.
- The pack affects appearance only. Native X11/Wayland/labwc acceptance, GUI resource measurements and broader custom widget tokens still remain open.

## Changes since rc.17

- RicePack v1 now supports optional semantic component colors (`[components]`): control, hover, border, table and selection. Legacy packs still parse and render unchanged; malformed or unknown fields are rejected.
- Preview, import, export, live reload and settings persistence now carry these colors. Preferences offers interactive color swatches and per-token preset resets. Selection text automatically follows the chosen fill contrast.
- Refreshed the original Sea Glass example and reproducible Taskotto designer screenshot, with parser and theme-rendering regression tests. Some individually drawn rice widgets still need extraction before fully free-form visual composition.

## Changes since rc.16

- Added an in-app RicePack library to Preferences, including bounded local TOML discovery, invalid-file feedback, path-based import and overwrite-safe export. No additional GUI toolkit or runtime dependency is needed.
- Added reversible live preview: users can temporarily apply a design, restore the previous appearance and watcher, or keep a valid design for the current session with live reload. Monitoring settings retain their persistence behavior.
- Added an authentic egui-rendered Sea Glass preview screenshot (38 total), unit tests for import/overwrite, preview rollback and preserved CLI theme precedence. RiceBowl's distinct desktop pack format still needs a versioned interop fixture.
- The stable 1.0 release remains gated by physical X11/Wayland/labwc QA, GUI resource measurements and signed-off packaging.

## Changes since rc.15

- GUI sessions started with `--ricepack` or a RicePack environment variable automatically check for edited pack files about every two seconds. Valid design changes appear without restarting the app.
- Incomplete or invalid edits keep the last valid design visible and produce a nonfatal warning, then recover when the file becomes valid again. A removed override falls back to the user's prior saved appearance, and `--theme` precedence is retained.
- No additional idle polling is added when Taskotto is started without a RicePack. Existing historical client screens, screenshots and compositor-owned window decorations remain unchanged.

## Changes since rc.14

- Implemented `taskotto --export-ricepack my-rice.toml`: export the saved theme and supported design tokens to a new RicePack v1 TOML file, then edit, validate and reload it. This is the first complete, data-only customization roundtrip suitable for AI-generated edits.
- Export refuses overwriting existing files, preserves user preferences, rejects invalid saved configs, and runs without a display. Added regression coverage and updated the [RicePack guide](RICEPACK_FORMAT.md).
- Native desktop acceptance, live in-app pack previews and arbitrary AI-generated component layout schemas remain pending.

## Changes since rc.13

- Added an explicit future [expansion roadmap](EXPANSION_ROADMAP.md) with stable-1.0 hardware gates followed by phased work on a complete declarative RicePack UI, process affinity/priorities, GPU and container metrics, system pressure diagnosis, dashboards and measured performance.
- Introduced the first **Taskotto RicePack v1** format: unprivileged, declarative TOML files with a valid base theme and optional shared colors, density, navigation, typography and spacing values. The current client UI remains Rust-rendered; per-component layouts and live editor are planned, not claimed.
- `--ricepack FILE` and `RICEBOWL_TASKOTTO_RICEPACK` apply a pack for one GUI session; `--check-ricepack FILE` validates it without a display. Unknown fields, invalid input and files over 64 KiB are rejected. The existing appearance preferences are restored for persistence even when monitoring settings are saved during a pack session.
- Added the sample Sea Glass pack, the versioned [format guide](RICEPACK_FORMAT.md), parser/CLI/config persistence regression tests and kept all existing rice behavior and the 37 screenshots. Native hardware QA remains pending.

## Earlier changes since rc.12 (rc.13)

- Removed the in-client simulated XP Luna, Windows 7 Aero and Windows 10 title strips, icons and duplicate captions. These were not native window decoration, so they caused duplicate app bars when RiceBowl/labwc supplied the real window frame. File/Options/View/Help menus and actual navigation stay intact.
- Explicitly request native decorations on the eframe window for **every** rice; window caption, border, button arrangement and colors remain the responsibility of RiceBowl/the user's window manager. The macOS activity toolbar no longer repeats the app caption; ChromeOS and Frutiger Aero retain real application toolbars/masthead content, and Windows 98 remains a compact client without a fake window frame.
- Added regression coverage for the compact historical menu bars and regenerated all 37 real egui/procfs client screenshots. Headless screenshots do **not** include the compositor-provided outer window decoration; native verification remains an open acceptance gate.

## Earlier changes since rc.11 (rc.12)

- ChromeOS also graduates from a color preset to a dedicated table-first task-manager UI: responsive task/memory/CPU/PID/disk I/O report with live search, sorting, user filters, selection, confirmation-backed End process and an overflow menu leading to real Linux tools. Chrome-only per-process network/private-memory figures are deliberately not fabricated.
- Frutiger Aero is now a fully independent seventh-destination client UI rather than a color preset: original aqua/sky gradient, bubble-mark masthead, polished pill navigation or optional sidebar, glossy system-health gauges, live process report and four live charts.
- Search, persistent sorting, tree mode, process inspection and confirmed End Task remain backed by existing real Linux procfs/PID-safe signaling. Physical memory, aggregate CPU, block-device I/O and non-loopback RX/TX display measured values; network figures are never attributed to individual PIDs.
- Six reproducible live-egui screenshots (37 total: two ChromeOS, four Aero), 650×420/1280×800 and responsive navigation layout tests, zero proprietary assets; native X11/Wayland hardware QA remains pending.

## Earlier changes since rc.10 (rc.11)

- Classic 98 now opens an independent, compact Windows 98-style **Close Program** view, not a modern sidebar with a gray palette. Raised/sunken controls, white list and classic blue selection are original drawings; there is no extra in-client title bar or window buttons because RiceBowl/native WM owns external decoration.
- The running-program list is explicitly current-user Linux processes, not fabricated GUI-window detection. End Task uses safe PID-aware signal confirmation, New Task runs user-entered commands without elevation, and Refresh/Close work. The optional Tools menu exposes searchable/sortable Processes, real CPU/memory Performance, Networking, Users and Options.
- Four new real egui/procfs renders (31 total) plus headless narrow/desktop layout coverage; native display-server and side-by-side historical pixel comparisons remain for hardware acceptance.

## Earlier changes since rc.9 (rc.10)

- The macOS preset now has an independent, light Activity Monitor-inspired layout: original mini activity-mark, Stop/Info/View toolbar, live search, five historical horizontal segments (CPU, Memory, Energy, Disk, Network), sortable dense report, per-process context menu and an actual selected-process Info dialog.
- Context-specific bottom statistics and rolling CPU/memory/disk/network charts reflect live Linux samples. Memory columns show resident/virtual usage; Disk exposes permission-checked `/proc/PID/io` with explicit N/A; Energy reports genuine process CPU as an explicitly labeled activity proxy since the proprietary macOS Energy Impact and App Nap measurements cannot be derived from procfs. Network remains system-wide, not fabricated per PID.
- Added custom-sidebar support, column-fit regression tests for narrow and wide windows, five-page/two-layout process+info headless UI tests and six genuine procfs-driven screenshots. The compact README now leads with four dedicated presets and documents all 27 focused captures.
- Native X11/Wayland testing and paired visual comparisons against a licensed macOS installation remain open; **95% or 1:1 pixel accuracy is not claimed**.

## Earlier changes since rc.8 (rc.9)

- Windows 7 now has its own task-manager UI, rather than inheriting the generic Taskotto sidebar/cards: a bright, original Aero-like header, classic File/Options/View/Help menu, six period-correct horizontal tabs, thin raised borders and a system status strip.
- Implemented the six usable Windows 7-style tabs: Applications (current-user process approximation and working New Task), Processes (compact sortable report, column selection and safe process actions), Services (nonblocking, read-only systemd user units with explicit errors), Performance (per-logical-core history plus CPU/RAM usage and live memory/swap/thread statistics), Networking (real aggregate RX/TX rates), and Users (real UID group summaries).
- New per-core sample history is bounded to 120 readings per core and handles CPU-core-count changes. No per-core history is fabricated from an aggregate sample.
- Seven actual headless egui scene captures (six historical pages and the floating process properties sheet) and a refreshed compact three-rice README gallery bring screenshot provenance to 21 images. Screens with systemd permission failures display those genuine errors in the capture environment.
- As with the other two rices, the goal is a recognizable, precise historical layout, but **95% pixel similarity is not certified** without paired reference captures on the target desktop. Linux process command lines and RSS are not Windows file-description metadata or private working set.

## Earlier changes since rc.7 (rc.8)

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
