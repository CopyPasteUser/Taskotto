# Taskotto 1.0.0-rc.28 — release candidate

**Date:** 2026-11-07 · **Platform:** Linux x86_64 development/CI build; X11/Wayland GUI hardware acceptance pending.

Rust-only Linux process manager and system monitor. This is a **prerelease**, not a stable 1.0 certification.

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
