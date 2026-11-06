# Changelog

Taskotto follows semantic versioning; hardware-dependent stable-release gates are explicit.

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
