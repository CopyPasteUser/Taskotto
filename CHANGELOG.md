# Changelog

Taskotto follows semantic versioning; hardware-dependent stable-release gates are explicit.

## [1.0.0] — 2026-11-09

- Published the first stable Taskotto release after the project owner approved it following several days of real-world use. This approval does not constitute a completed per-desktop test matrix or measured GUI resource baseline; the outstanding evidence is tracked in `ACCEPTANCE_1_0.md` and `PERFORMANCE.md`.
- Documents the existing RiceProfile v1 session selection contract and includes user-local desktop launcher/icon installation fixes delivered after RC56, plus the following sampler improvements.

- Lowered `/proc` polling work by caching stable per-process status/command metadata for at most five seconds, invalidated on PID lifetime or process-name change and pruned when a PID disappears. Live CPU/RSS/I/O counters and all process actions remain unaffected; mutable metadata can be displayed with a bounded five-second delay.
- Automatically reduce sampling frequency while the GUI is minimized (minimum five seconds), then restore the configured interval upon unminimizing. The existing low-overhead setting and on-demand rendering remain intact.
- Added metadata lifecycle, minimized-cadence and optional sampler CPU benchmark coverage; recorded measured limitations in `PERFORMANCE.md`. Actual Mint/X11 CPU and RAM improvements still require hardware verification.

## [1.0.0-rc.56] — 2026-11-07

- Added opt-in **per-user startup RicePack** from Preferences → RicePack library: a validated absolute `.toml` path is persisted independently of session-only appearance overrides, and can be cleared with one click. This makes user-authored P2 dashboards reusable after restart without a resident service or executable RicePack content.
- Explicit `--ricepack`, `RICEBOWL_TASKOTTO_RICEPACK` and `TASKOTTO_RICEPACK` still override the saved default in that order; a broken explicit pack remains an error, but a missing/invalid saved pack falls back to normal saved appearance with an actionable UI warning. The normal session-only reload/watcher is retained.
- Added startup precedence, broken-file recovery, path-bound and legacy preference tests. No auto-save of process/system history, no privilege change, and no stable-1.0 hardware sign-off is implied.

## [1.0.0-rc.55] — 2026-11-07

- Added three strictly typed RicePack v1 time-series comparisons: CPU versus RAM on one 0–100% scale, and RX/TX or disk read/write on one dynamically chosen bytes/s scale. Existing single-series plots and the stable pack schema remain compatible; missing samples are not synthesized. The custom screen shows a readable legend and its shared y-axis range without adding a polling thread.
- Added a valid Comparison Studio pack with responsive fallback, headless egui screenshot and strict round-trip/unit tests. Extended the Windows 10 App history and Windows 10/XP process-owner panels to honor optional table/hover/border RicePack colors with legible foreground while keeping the original era-specific fallback.
- Native comparison, live color/contrast and resource-usage acceptance on Mint/X11, KDE/Wayland and RiceBowl/labwc remain open. Driver-specific GPU telemetry and privileged scheduling controls remain separate post-foundation tracks, not silently counted as stable release gates.

## [1.0.0-rc.54] — 2026-11-07

- Added explicit two-snapshot cgroup CPU usage and block I/O rate comparison on re-inspection of the same selected process, using monotonic actual elapsed time, cgroupfs inode and membership path identity. No continuous sampler, service, root permissions or extra background task is introduced.
- Rejects cgroup replacement/recreation, paths changed by migration, zero/short intervals, missing controllers and counter resets instead of inventing negative or zero rates. CPU use is expressed as a percentage of one core (multi-core values may exceed 100%). Tests verify identity, elapsed time, reset/missing data and real units.

## [1.0.0-rc.53] — 2026-11-07

- Extended the explicitly requested, asynchronous read-only cgroup v2 inspection with a bounded nearby-hierarchy view: the selected group's direct parent and up to 16 immediate child cgroups, showing their own cumulative CPU use and memory current when readable. The scan stops after 128 entries and notes truncation; unavailable enumeration never appears as an empty hierarchy.
- No background recursive scans or synthetic service ownership. The UI warns that ancestor/descendant counters overlap and cannot be summed. Fixture tests verify deterministic sorting, bounds, real missing-data handling and selected-process identity protection remains in place.

## [1.0.0-rc.52] — 2026-11-07

- Added safe, idempotent `--uninstall-desktop` for removing only Taskotto-managed user-local launcher/icon files; refuses modified or symlinked files before removing either, preserves other apps and never deletes the Taskotto executable or preferences. Re-running `--install-desktop` repairs a stale launch path.
- Release workflow verifies the determinism of its own sorted/timestamp-normalized archive by producing a second identical package; it still checks SHA256 integrity and now smoke-tests install and uninstall. This does not claim independently reproduced Rust binary bytes on dissimilar build hosts.
- Added Rust file-integrity/symlink tests and CLI integration coverage for uninstall/idempotence/invalid flag combinations.

## [1.0.0-rc.51] — 2026-11-07

- Added an optional persisted low-overhead profile in Preferences: the existing worker samples at no faster than every 3000 ms and egui motion transitions are disabled, without disabling any monitoring tab or process action. Switching off restores the user's configured 500–10000 ms interval, which is never overwritten by the profile.
- Session and followed-process CSV/JSON exports now state the effective nominal sample interval. Added legacy-config, serialization, effective-interval and animation restoration tests. This is an explicit tradeoff, not an unmeasured CPU/RAM savings claim.
- Update-speed menus in the Windows-like, macOS and ChromeOS rices explicitly indicate the active 3-second floor, so their configured interval choices are not mistaken for the effective cadence.

## [1.0.0-rc.50] — 2026-11-07

- Added deliberately triggered, private CSV/JSON export of the currently followed process's bounded CPU/RSS history. Includes numeric PID+start-tick identity, observed GUI-receipt timestamps, MiB/percentage units, scope and nominal interval; excludes process name and command, refuses existing files, requires an absolute path and writes owner-only permissions.
- Reused RC45's safe new-file writer rather than duplicating file logic. Tests cover JSON/CSV fields, missing timestamps, alignment, sensitive-name omission, overwrite refusal and file mode. Process history stays in RAM until user chooses export.
- Regenerated the real headless process-follow screenshot to include the export control and its privacy explanation.

## [1.0.0-rc.49] — 2026-11-07

- Added genuine cgroup-local CPU, memory and I/O PSI to the existing selected-PID, asynchronous read-only cgroup v2 inspector. Reuses Taskotto's strictly validated kernel PSI parser for SOME/FULL 10/60/300-second stalled-task percentages and cumulative microseconds; cgroup CPU FULL is supported when present, unlike meaningless system CPU FULL.
- Missing/unsupported cgroup files and explicitly disabled `cgroup.pressure` accounting remain unavailable; namespaced cgroup data is never presented as host pressure. The expanded report is boxed in the GUI state to avoid inefficient enum layout. Synthetic parsing, missing-source and non-finite cases tested; native load comparison remains open.

## [1.0.0-rc.48] — 2026-11-07

- Follow-up accuracy fix: timed process-follow charts now place samples at their actual horizontal timestamp offsets. Missing/late sampler ticks preserve proportional horizontal distances instead of evenly spaced points; All samples still shows collection order. Added a deterministic gap-position test.
- Added optional elapsed 30-second and 2-minute views (or entire session retention) to the opt-in per-PID CPU/RSS follow panel, based on GUI-receipt timestamps rather than assuming equally spaced samples. CPU/RSS peaks reflect the selected window, and missing timestamps cannot masquerade as recent evidence.
- At narrow widths the two charts stack instead of squeezing into unreadable columns. Existing PID+start-tick identity verification, three-missing-sample expiry, 120-observation memory bound and no-persistence defaults remain intact. Deterministic time-boundary/gap tests added.
- Regenerated the actual `taskotto-system-process-follow.png` gallery screenshot with the three functional time-window controls visible, and documented targeted screenshot reproduction.

## [1.0.0-rc.47] — 2026-11-07

- Audited Frutiger Aero's remaining high-visibility navigation and hero panel hardcoded surfaces: horizontal tabs and opt-in sidebar use semantic control color, hero/layered backdrop and heading/subheading use table and readable-on-table colors, chart outlines honor the border token.
- Historical vector lens, glossy bubbles and ring meters remain intentionally rice-specific rather than genericizing the Y2K UI. Default unmodified presets preserve their palette; custom RicePacks gain reliable contrast in prominent areas. Native comparison remains pending.
- Regenerated all eight Frutiger Aero screenshots through the real headless egui renderer, including the dark Aero Lagoon process and performance views; kept the README's complete 51-capture gallery intact.

## [1.0.0-rc.46] — 2026-11-07

- Added strictly read-only, bounded cgroup v2 membership naming hints for systemd-like services/scopes, Flatpak-like app scopes, and Docker/Podman-style 64-hex unit IDs. Only the cgroup path is authoritative; display explicitly refuses to claim verified service ownership, API linkage or PID-container attribution.
- Synthetic path tests cover valid naming conventions, malformed IDs, controls and absent paths. No runtime sockets, root privileges, daemon or process mutations.

## [1.0.0-rc.45] — 2026-11-07

- Added explicit, no-overwrite, user-only-permission CSV and JSON exports of up to 120 session-only sampler-visible Linux system samples from every rice's Preferences. GUI-receipt timestamped rows include CPU, RAM, network, disk, nullable per-tick PSI, export schema version, observable scope and configured (nominal) interval.
- Unlike the visual PSI history, export storage preserves absent ticks as missing values; diagnostics still use only eight actual consecutive readings. Export contains no process identity information and never runs unattended. Tested format alignment, absence, file ownership mode, overwrite refusal and malformed history.

## [1.0.0-rc.44] — 2026-11-07

- Added evidence-first, explicitly tentative CPU/memory/I/O pressure investigation hints. They use thresholded 10s/60s PSI and at least six above-threshold observations among eight consecutive sampler ticks; missing readings cannot masquerade as sustained pressure.
- Hints describe measured evidence and a safe human follow-up, never assert causal attribution or trigger process actions. Synthetic tests cover thresholds, low sample counts, data gaps, non-finite values and mixed resources. Native controlled-workload validation remains a P6 gate.

## [1.0.0-rc.43] — 2026-11-07

- Extended Linux PSI telemetry into history-backed CPU SOME, memory SOME/FULL and I/O SOME/FULL trend plots using the existing sampler and chart primitives. The five rolling series retain at most 120 measured observations each and do not invent missing samples or imply separate independent stall events.
- System Performance and all rice Preferences display these as source-specific trend charts with labels and honest auto-scales, including compact-window support. Each point is a kernel-produced rolling 10-second stall-time percentage collected at Taskotto's interval, not a diagnosis from a single CPU load number.
- Added deterministic regressions for bounded PSI history and missing-reading continuity. Systemic slowdown hypotheses still require controlled workload/physical acceptance before any certainty claim.

## [1.0.0-rc.42] — 2026-11-07

- Began P6 kernel-backed PSI telemetry. The existing monitor worker samples `/proc/pressure/{cpu,memory,io}`; System Performance and every rice's Preferences report CPU SOME, memory SOME/FULL and I/O SOME/FULL stalled-task percentages over 10, 60 and 300 seconds, and cumulative stall time.
- Strictly validate kernel fields, reject missing, duplicate, nonfinite, negative or above-100% values. Do not fabricate zero when unsupported, mistake pressure for utilization, or display system-wide CPU FULL as a meaningful saturation signal. Compact layouts stack metrics below 700 px available width.
- Added deterministic kernel-format tests and optional real procfs probe. Rule-based diagnosis, thresholds and live workload acceptance remain P6 tasks; no single PSI sample is presented as a proven slowdown cause.

## [1.0.0-rc.41] — 2026-11-07

- Started P5 with an explicitly requested read-only cgroup v2 inspector for the selected process. It reads kernel membership `0::/path`, CPU use and throttling, current/maximum memory (including unlimited) and summed block I/O byte counters in a bounded background task; normal sampling performance remains unchanged.
- Defends against unsafe cgroup traversal and namespace mount escapes and checks PID start ticks and membership both before and after reading. Missing controllers, permissions, moved or exited processes yield truthful unavailable/error states; cgroup-wide results are never falsely attributed to one PID or an invented container identity.
- Added strict path, cgroup CPU/I/O parsing and current-process tests. User-systemd, real Podman/Docker associations, Flatpak labels and physical container acceptance remain open.

## [1.0.0-rc.40] — 2026-11-07

- Added read-only battery and external-power sampling through the standard Linux `power_supply` sysfs class inside the existing worker loop. System Performance and every rice's Preferences show reported charge percentage, charging state, actual Wh, W and external-source availability when present.
- Precisely distinguish `energy_now/full` microWh from `charge_now/full` microAh; the latter can calculate a percentage but is **never** displayed as Wh. Reject inconsistent or out-of-range capacity/energy readings rather than showing synthetic zeroes or fabricated run times.
- Added fake-sysfs tests for Wh/W conversion, separate charge/energy fallback, invalid percentages and missing hardware. Real ThinkPad battery, dual-battery and AC hot-plug tests remain part of physical acceptance.

## [1.0.0-rc.39] — 2026-11-07

- Began P4 read-only GPU telemetry with small DRM/sysfs probes inside the existing monitor sampling cycle. AMDGPU busy percentage and VRAM usage/total use documented kernel counters; accessible hwmon temperature, power and core clock and Intel GT clock values use their actual units.
- All DRM cards with accessible PCI vendor IDs are inventoried (AMD, Intel, NVIDIA or Other). System Performance and all rice Preferences display genuine available sensors and explicit driver-specific **Unavailable** states, with no fake Intel/NVIDIA utilization or per-process GPU attribution. No vendor SDK, subprocess polling or extra dependency is required.
- Added deterministic mock-sysfs tests for supported AMD values, units and PCI connector filtering, plus Intel/NVIDIA missing-capability behavior. Hardware validation and full Intel/NVIDIA sensor backends remain open.

## [1.0.0-rc.38] — 2026-11-07

- Added one-shot asynchronous **PSS memory inspection** for the selected Linux process and at most 127 observed descendants using `/proc/PID/smaps_rollup`. Taskotto's usual fast sampler remains unaffected, with no daemon or root service. Accessible via generic Details or Preferences, including historical rices.
- Rejects stale root identity and any tree above 128 observed processes before memory-map reads; revalidates each PID's start time around its own read. Missing privileges, unsupported kernels, disappearing processes or unreadable files yield an explicitly **incomplete** proportional-memory report instead of a bogus zero. Results reflect non-atomic procfs observations, not exclusively owned physical memory.
- Added strict parser/overflow tests, actual-proc best-effort tests, stale-PID and size-cap tests. P3's privileged controls, compositor window grouping and native memory-accounting QA remain open.

## [1.0.0-rc.37] — 2026-11-07

- Added explicit **Follow selected process** in generic Details and Preferences; at most one exact PID+start-time identity is tracked per open window, storing a maximum of 120 real CPU and RSS samples entirely in memory. No idle sampler, daemon, file write or extra permission is involved.
- Missing process samples are not replaced by fake zeros; three consecutive misses end tracking without reattaching to reused PIDs. The user can clear ended history or follow a new selection. History charts report actual last and peak values, not chart-scale maxima.
- New true-egui screenshot `taskotto-system-process-follow.png` captures a real disposable Linux child and the bounded graphs. Regression tests cover bounds, expiry and PID reuse. This completes a first session-history/follow slice of P3, not its advanced privileged controls, PSS attribution or compositor-specific grouping.

## [1.0.0-rc.36] — 2026-11-07

- Added **session-only process sort freeze**: the visible row order stays stable while CPU/RAM/I/O measurements update. Toggle with Ctrl+Shift+Space in all eight profiles, the generic process toolbar or Preferences → Process table. Settings and saved RicePacks remain unchanged.
- New processes append to the frozen list in deterministic PID order. Entries are keyed by PID **and start ticks** so kernel PID reuse cannot inherit a defunct process's position. Hierarchical mode still places descendants beneath observed parents; normal sorting returns immediately when unfrozen.
- Added regression coverage for live metric flips, stale PID reuse, new processes and tree hierarchy. Physical keyboard-focus and window-manager verification remains outstanding.

## [1.0.0-rc.35] — 2026-11-07

- Added low-overhead read-only scheduling telemetry sourced from the process leader's `/proc/PID/stat` signed `nice` value and `/proc/PID/status` `Cpus_allowed_list`. No new syscall, background sampler, privilege or driver dependency is required.
- Generic Details, XP Luna Properties and macOS Info now expose the observed **leader** Nice level and allowed CPU range alongside existing truthful CPU/RAM and descendant metrics. Other threads can differ; missing affinity remains `Unavailable` rather than invented all-core access.
- Regression tests cover negative priorities, absent/oversized affinity text and actual current-process fields. Mutating nice, affinity or I/O priority remains pending safe race-free targeting, explicit user confirmation and Linux permission testing.

## [1.0.0-rc.34] — 2026-11-07

- Started P3 read-only process tree aggregation: for any selected process, a single live Linux snapshot now yields the number of **distinct observed PIDs** in the descendant tree, their summed per-process CPU percentages, and their summed resident RSS. The traversal guards cycles, duplicate PID entries, stale root start ticks and byte-count overflow.
- Added the observed-tree summary to the standalone process Details view, XP Luna properties and macOS-style Info. All three explicitly explain that totals include only visible processes and **summed RSS may count shared memory more than once**, so it is not unique physical usage or proportional set size (PSS).
- Added deterministic malformed-tree/PID-reuse tests and updated native acceptance checks. P3 process priority/affinity safety, PSS-aware shared-memory accounting, app/window grouping and follow/history remain open.

## [1.0.0-rc.33] — 2026-11-07

- Fixed `Ctrl+F` in declarative TOML screens: generated toolbars now open and focus the actual process search; packs without any search toolbar get a keyboard-opened, dismissible fallback field. Closing a search also clears hidden filters instead of leaving an invisible query applied.
- RicePack validation now diagnoses missing, malformed, future and legacy `format_version` before parsing unsupported new widget shapes. The CLI reports verified page/widget/compact capabilities after `--check-ricepack`, explicitly without unsafe permissions or unrequested automatic migration.
- Added headless keyboard-focus regressions for generated screens with and without search toolbars, and tests that decode **all 50** bundled PNG screenshots, verify capture sizes and reject blank/corrupt images. Full native screen-reader, keyboard and display-server acceptance is still pending.

## [1.0.0-rc.32] — 2026-11-07

- Added a functional, session-only **Screen layout designer** in Preferences. Users can start with a safe default, edit up to four pages, name and reorder widgets, choose verified metric/plot/gauge bindings and select predefined status, toolbar and process columns. The seven predefined widget templates need no scripting or compiling.
- Separate standard and compact compositions can be edited; draft changes are validated together and **never applied** to the active client while invalid. Valid layouts can be applied to the running monitor and explicitly exported through the existing non-overwriting RicePack export flow. Discard and reset are available; no implicit config/file updates occur.
- Added a genuine egui layout-editor screenshot, editor schema test and accessibility-friendly Move up/Move down buttons. Desktop keyboard/physical acceptance and P2 migration diagnostics remain open.

## [1.0.0-rc.31] — 2026-11-07

- Added backwards-compatible, validated responsive RicePack screen variants: an optional 480–1024 px content-area breakpoint (`layout.compact_below`) and up to 16 alternative `compact_widgets` per page, chosen automatically as the Taskotto window resizes. Old packs continue rendering exactly their original widgets.
- Diagnostic Studio now has original compact overview and traffic compositions with fewer columns, controls and metrics, while preserving accurate Linux data and the existing PID-confirmed process actions. The real egui gallery gains a third, 720 px wide compact capture and strict bound/round-trip tests.
- P2 in-app drag/drop layout editing and explicit visual/accessibility migration checks remain open; hardware acceptance for a stable 1.0 is still pending.

## [1.0.0-rc.30] — 2026-11-07

- Completed the initial P2 safe widget vocabulary: generated TOML screens can now compose real CPU/memory percentage gauges, read-only aggregate status strips and a toolbar with only `refresh`, `focus-search` and confirmation-backed `end-process` actions, in addition to the existing live metrics, plots, tabs and process lists.
- Extended strict schema validation for missing/duplicate/unknown bindings and toolbar actions; unavailable gauge values remain explicitly unmeasured instead of becoming fake zeroes. Search focuses its input, process actions retain the same PID-safe confirmation and duplicate End Process buttons are suppressed when the generated toolbar supplies one.
- Updated the two genuinely rendered Diagnostic Studio gallery scenes and CLI/format/QA documentation. The P2 responsive capability contract, interactive editor and migration/visual-focus testing remain open.

## [1.0.0-rc.29] — 2026-11-07

- Began P2 with a strictly bounded, data-only declarative screen layout in RicePack TOML: up to four named screens with 16 widgets each, typed headings, live metrics, historical plots and virtualized real-process lists. Screen tabs and safe process selection/confirmation run through existing unprivileged Linux code.
- Added `Diagnostic Studio`, a distinct two-screen original RicePack that can be checked, exported, live-reloaded and rendered without a bespoke Rust rice module. Invalid bindings, unknown widget kinds, duplicate columns, repeated screen ids, excessive sizes and script-like fields fail validation.
- Existing packs still render their original client pages. Declarative toolbars, gauges, layout editor, responsive capabilities and the rest of P2 remain open, as do real X11/Wayland/labwc acceptance gates.

## [1.0.0-rc.28] — 2026-11-07

- Extended validated RicePack control/table/border colors to the Windows 98, XP Luna, Windows 7/10, macOS and Aero status/menu/toolbar strips; preserved original fallback palette and native compositor-owned window frames. ChromeOS retains its existing dynamic dark/light strip palette.
- Adjusted contrast for custom dark backgrounds on legacy light strips and when only a dark global surface is configured. The macOS bottom timeline and network charts now honor table/border tokens; network labels switch to light text for a dark chosen surface.
- Added compact/desktop layout regressions for custom dark bars, no-component dark surfaces and all historical clients. P1 intentional illustration/internal per-widget palette audit and P0 real-device QA remain pending.

## [1.0.0-rc.27] — 2026-11-07

- Continued data-only RicePack component-color propagation into historical secondary pages: Windows 7 Users/Services reports, performance statistic boxes and process properties; XP properties, performance groups and networking summary; Windows 10 focused chart and metric selector; macOS process inspector; Frutiger Aero network activity chips.
- Matched user-supplied dark table backgrounds with legible light foreground, applied semantic border and selection colors, and retained each rice's original appearance when overrides are absent. Added narrow/wide live-page rendering regression runs with a dark generated palette.
- Left intentional artwork, platform-specific chrome and untested native display-server acceptance untouched. Remaining secondary surface audit and declarative screen schema stay open on the roadmap.

## [1.0.0-rc.26] — 2026-11-07

- Added bounded performance chart-size customization to RicePack v1 and Preferences while preserving every built-in rice's default dimensions.
- Extended export, validation, transient preview, live reload and config rollback to chart scaling, with invalid-input and legacy-pack regression tests.
- Secondary-page palette audit and stable 1.0 hardware release gates remain open.

## [1.0.0-rc.25] — 2026-11-06

- Connected Classic 98's original beveled action/navigation buttons, focus outlines, sunken list border, selected/hovered Close Program entries and optional advanced process report to the shared validated RicePack component-color contract. Defaults keep the original grayscale Win98 feel; real window controls remain the window manager's responsibility.
- Replaced legacy fixed-fill list buttons with focused/clickable egui cells that can reflect actual pointer hover and safely retain PID-aware selection/process actions; dark list and hover fills get readable foreground.
- Added the original Violet Windows 98 RicePack and a genuine Close Program client screenshot (46 gallery captures), with palette regression coverage. Metric/chart sizing and further secondary-page token extraction remain on P1; native hardware acceptance remains outstanding.

## [1.0.0-rc.24] — 2026-11-06

- Unified optional RicePack semantic colors across XP Luna/Windows 10 top tabs and shared process reports, XP Applications and the custom Windows 10 sidebar. The original designs remain unchanged when overrides are absent; table, borders, selection, control and hover fills and foreground adapt without scripts or proprietary artwork.
- Preserved Windows 10's truthful per-process CPU/memory heatmaps on dark custom tables by blending observed utilization into the user-defined row palette; selected/hovered cells keep readable contrast.
- Shipped original Midnight (XP Luna) and Indigo (Windows 10) packs with two live-procfs headless screenshots (45 gallery images) and regression tests for both token palettes, report geometry and bounded heatmap blending. Windows 98 bevel/size extraction and native hardware QA remain open.

## [1.0.0-rc.23] — 2026-11-06

- Wired the Windows 7-inspired client tabs, sortable Applications/Processes tables, hover and selection fills, border strokes and empty-table messages to shared, validated RicePack component colors. Missing tokens keep the original Aero-era colors and layouts.
- Added original `win7-dusk.toml` and a genuine egui screenshot of the selected Linux process in a dark custom table (43 screenshots), plus a default/custom palette regression test.
- Explicitly retained the standard palette for Windows 7 Services/Users plain egui grids; their user-defined table styling remains for later work. No native window decoration or proprietary assets were added, and real X11/Wayland/labwc QA remains open.

## [1.0.0-rc.22] — 2026-11-06

- Connected ChromeOS's existing light/dark adaptive theme to shared RicePack component tokens for control, hover, border, table and selection. Dynamic table text uses contrast-aware colors while toolbar, metric panels and Linux statistics preserve their existing semantics.
- Added a true headless egui process-list capture for the original Nocturne pack (42 screenshots total), with regression coverage for default theme preservation and customized semantic colors.
- Continued P1 renderer extraction without new runtime dependencies or proprietary OS assets; Windows-inspired renderers and real-device hardware QA remain outstanding.

## [1.0.0-rc.21] — 2026-11-06

- Extracted a shared, no-dependency `ComponentPaint` resolver for validated semantic RicePack colors, retaining Frutiger Aero's previous output and avoiding per-rice duplicate parsers.
- Wired macOS Activity Monitor's custom tab/sidebar controls, table headers, process rows, hover/selection and borders to the same component tokens. Historical defaults remain intact when fields are absent; dark table and selected rows get readable contrast.
- Added the original Nightshift RicePack and a genuine egui screenshot (41 total), with default/custom regression tests. Native hardware acceptance and the remaining renderer migration remain outstanding.

## [1.0.0-rc.20] — 2026-11-06

- Extended existing validated RicePack v1 component tokens into Frutiger Aero's individually drawn client: topbar control, navigation/hover/selected tabs, process table rows and focus, and performance chart containers now honor optional control/hover/border/table/selection colors without new dependencies or scripts.
- Added contrast-aware foreground for customized dark tables/controls and selection, preserving all historical defaults when no token is supplied. The original aquamarine vector decoration, chart data and window-manager-owned frame remain unchanged.
- Published two new headless egui screenshots of the included Aero Lagoon example (40 captures) and tests for default and custom semantic colors. Recorded the stable design and native interaction contracts; other custom rice widgets and physical desktop QA remain open.

## [1.0.0-rc.19] — 2026-11-06

- Added an independently runnable, versioned contract fixture generated by RiceBowl's `de taskotto-export` bridge. A Rust integration test validates the actual exported v1 TOML, applies its appearance tokens and verifies process-monitoring preferences remain unchanged.
- Extended headless `--check-ricepack` CLI coverage to include RiceBowl's fixture, without making RiceBowl or Python a Taskotto runtime/test dependency. Documented the opt-in recipe export roundtrip.
- P0 native Mint/X11, KDE/Wayland and RiceBowl/labwc QA remains required for stable 1.0; custom rice component-token extraction remains on P1.

## [1.0.0-rc.18] — 2026-11-06

- Added five optional, strictly validated semantic component colors to RicePack v1: control, hover, border, table and selection. Existing packs remain compatible; no code/scripts/assets or monitoring rights can be supplied by a pack.
- Extended session-only config backups, file reload and CLI export roundtrips to include semantic tokens; new color swatches in Preferences support a per-token return to built-in values.
- Applied component overrides after rice-specific default egui styles, added readable control/selection text and rejected visually unsafe table fills. Re-rendered the genuine Sea Glass in-app preview and added parser/reload/token-style regression coverage.
- More fine-grained custom rice drawings still need token extraction; cross-repository RiceBowl contract and physical GUI acceptance are not claimed complete.

## [1.0.0-rc.17] — 2026-11-06

- Added a local RicePack library browser with bounded non-recursive discovery, explicit invalid-pack errors and safe, uniquely named import without overwriting designs.
- Preferences now offers live, reversible RicePack preview, Keep for session with file watching, and no-overwrite export of the current appearance. Original packs and monitoring preferences survive preview rollback.
- Added 38th genuine egui/process-data capture for the Sea Glass preview editor; expanded gallery, workflow documentation, roadmap and state regression tests. RiceBowl's JSON desktop manifest is explicitly not treated as Taskotto TOML.
- Real Mint/X11, KDE/Wayland, labwc acceptance and stable 1.0 certification remain outstanding.

## [1.0.0-rc.16] — 2026-11-06

- Added optional **live RicePack file reload** to GUI sessions launched with a design. Valid changes to a RicePack's theme/tokens appear about two seconds after saving, without restarting Taskotto.
- Retains the last valid appearance through incomplete writes, malformed TOML and missing files. Removed overrides fall back to the saved user design, explicitly selected `--theme` remains authoritative, and monitoring preferences persist unchanged.
- RicePack-free sessions retain the previous timer-free GUI behavior. Added regression coverage for invalid→valid transitions, design reset, navigation changes and diagnostics; updated RicePack documentation and roadmap.

## [1.0.0-rc.15] — 2026-11-06

- Added `--export-ricepack PATH` to convert the saved Taskotto appearance into a validated, editable RicePack v1 design. This makes a concrete roundtrip for manual/AI-assisted customization: export → edit → check → apply.
- Export works headlessly, reads only saved preferences, preserves monitoring settings, never replaces an existing RicePack file, and uses owner-only file permissions on Unix. Added parser roundtrip and binary integration tests for successful export, validation, config preservation, and overwrite protection.
- Updated the RicePack format guide and expansion roadmap. Live RicePack editing, previews and arbitrary component layouts remain future milestones; stable 1.0 still requires real hardware acceptance.

## [1.0.0-rc.14] — 2026-11-06

- Published an [expansion roadmap](EXPANSION_ROADMAP.md) covering 1.0 hardware gates and eight subsequent milestones: RicePack contracts, declarative UI, advanced process actions, GPU monitoring, cgroups/containers, pressure diagnosis, dashboards/exports and measured performance.
- Implemented the first RicePack v1 contract: a bounded, strictly validated data-only TOML design with a selectable built-in base rice, palette, navigation, density and sizing. New `--ricepack PATH`, `--check-ricepack PATH` and `RICEBOWL_TASKOTTO_RICEPACK` entry points.
- Packs apply only to the current Taskotto session; saved appearance remains intact when other preferences change. Added a versioned format guide, original sample RicePack, parser/rejection/config-persistence tests and a real CLI integration test. No new dependencies, scripts or unverified compositor window controls.

## [1.0.0-rc.13] — 2026-11-06

- Fixed duplicate top-level window chrome across the eight rice presets: the XP Luna, Windows 7 and Windows 10 client no longer draw an extra application title strip. Native title, borders and min/max/close buttons belong to RiceBowl/the active window manager.
- Explicitly enabled window-manager decorations in the native viewport. Removed the redundant app-icon/title group from the macOS toolbar; retained real menu items, buttons, search, navigation, the ChromeOS toolbar and Frutiger Aero's content masthead.
- Added a historical-client menu-height regression test. Refreshed all 37 actual headless egui/procfs screenshots; the compositor's native frame is not present in these reproducible client captures. Physical labwc, X11 and KDE Wayland verification is still required for the stable release.

## [1.0.0-rc.12] — 2026-11-06

- Promoted ChromeOS from a generic color preset to a dedicated, table-first task-manager client with search, owner filters, sorting, keyboard focus, Linux-backed memory/CPU/PID/disk I/O, selection, process inspection and confirmation-protected End process.
- Added a compact ChromeOS overflow menu for the remaining Linux tools and optional custom navigation, plus responsive supplemental CPU/memory/disk/network views. ChromeOS-only private memory and per-PID network counters are not invented.
- Added a standalone **Frutiger Aero** task-manager UI: original glass/sky gradient, aqua/leaf vector graphics, luminous tab navigation (and optional sidebar), process table and clear system-health presentation. No copied OS assets, extra native window borders or title buttons.
- Processes use genuine Linux procfs entries with live search, persistent sorting, tree mode, selection, inspection and PID-safe confirmed termination. CPU/memory/disk/network pages show original meter/chart components backed by the existing bounded real-data histories. Network labeling explicitly identifies system-wide non-loopback traffic, not invented per-process rates.
- Added six genuine live-data egui captures (two ChromeOS, four Aero), narrow/desktop and layout regression coverage, and docs. Native compositor, interaction and performance checks remain open for stable 1.0.

## [1.0.0-rc.11] — 2026-11-06

- Gave Classic 98 a standalone Close Program client rather than the generic Taskotto theme: compact gray 3D controls, a sunken white program list, dark-blue selection and no duplicate RiceBowl-managed external window decorations.
- The list transparently shows actual current-user Linux processes (not invented Wayland/X11 graphical windows); End Task requires the established PID-safe confirmation. New Task, Refresh and Close work, while Tools reveals optional Processes, Performance, Network, Users and Options.
- Added a searchable/sortable classic PID/CPU/RSS report, 98-style color/typography tokens, small default viewport, four genuine Linux/egui captures, and additional layout/filter regression coverage. Native UI verification remains required for stable 1.0.

## [1.0.0-rc.10] — 2026-11-06

- Gave macOS its own Activity Monitor-inspired UI: original-drawn toolbar, process Stop/Info/View controls, always-available search, familiar five horizontal CPU/Memory/Energy/Disk/Network segments, compact sortable alternating-row process report and process-inspection dialog.
- Reused the authoritative Linux monitor, bounded CPU/memory/disk/network history and PID-reuse-protected confirmation actions. Energy reports measured CPU activity as an openly labeled *proxy*, not invented Apple Energy Impact; Network displays real system-wide traffic without invented per-process attribution.
- Added a macOS-specific light color palette, optional custom sidebar, narrow/wide responsive columns and five-view/two-layout process/info regression coverage.
- Added six live-Linux-procfs egui screenshots for the five tabs and process information. Refreshed the compact README gallery and screenshot/fidelity/QA notes (27 headless scenes total). Pixel-level 95% fidelity, native window visuals, performance and hardware acceptance are still unverified.

## [1.0.0-rc.9] — 2026-11-06

- Added a separate, era-specific Windows 7-style task manager: original-drawn Aero chrome and six horizontal classic tabs (Applications, Processes, Services, Performance, Networking, Users).
- Implemented a period-appropriate sortable white process report with classic-style headers, status strip, functional all-user toggle, menu search, and protected context/end actions; row widths respond to narrow windows.
- Added bounded per-core CPU histories and Windows 7-style green CPU/blue memory charts, Linux physical-memory/swap/system statistics, real aggregate RX/TX history, per-UID Users table, and read-only asynchronous user service rows or explicit error states.
- Reused the real New Task launcher for Applications, shared Linux process safety and live procfs data. No invented graphical window enumeration, Windows-only service PIDs, resource counters or platform icons.
- Captured seven new genuine egui screenshots (including a floating selected-process properties sheet), now 21 total; condensed the README gallery into three representative historical rices and updated provenance/fidelity/QA documentation. Native X11/Wayland and reference-image comparison are still required before claiming 95% similarity.

## [1.0.0-rc.8] — 2026-11-06

- Windows 10 task-manager rice now has all seven historical top tabs (Processes, Performance, App history, Startup, Users, Details, Services). The original uses horizontal tabs; a separate, clearly identified vertical sidebar can be selected in Preferences → Navigation.
- Implemented truthful Linux-backed substitutes: live process-lifetime CPU time (not permanent Windows app history), XDG autostart desktop entries with user-over-system override precedence, and asynchronous read-only systemd user service status with explicit failure states.
- XP Luna's File menu and Applications page now offer a working New Task dialog that executes explicitly entered shell input under current user permissions. The unsupported cross-compositor Switch To action is visibly unavailable.
- Polished XP Applications action-button placement, Windows 10 current-process history table alignment, read-only service/startup states, and sidebar active/focus visuals.
- Extended reproducible headless egui gallery to fourteen screenshots, with the custom vertical variant and an XP New Task dialog. Added XDG specification, CPU-time and systemd parsing tests and narrow/wide navigation regression coverage.
- Native X11/Wayland and reference-image pixel comparisons are still required before any quantifiable historical accuracy claim or stable 1.0 release.

## [1.0.0-rc.7] — 2026-11-06

- XP Luna now uses all five historically recognizable tabs. The new Applications screen explicitly lists the current user's Linux processes rather than pretending to know open Wayland windows; Networking graphs live non-loopback RX/TX traffic and Users groups real processes by UID.
- Added XP-style selected-process Properties with live PID, owner, memory, threads, timing and I/O fields, plus existing safe/confirmed process signal actions.
- Windows 10's Users tab expands per-owner process rows with CPU/RSS summaries; Details is an ungrouped live process table with sorting and the existing process actions.
- Redesigned the XP Applications row selection to align names left and bounded command previews in Properties to prevent very long arguments stretching the layout.
- Rebuilt the README gallery around nine fresh XP/Windows 10 screenshots generated from actual egui paint commands and live procfs samples. Removed three obsolete dark/Classic 98/Seven PNGs from the focused gallery and source tree; other rice presets remain functional.
- Native X11/Wayland acceptance and pixel-for-pixel historical verification remain open. No proprietary OS imagery is included.

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
