# Taskotto UX Contract

This is a native Rust/egui process manager, not a browser CRUD application.
Apply browser-only patterns only where their intent fits the native toolkit.

## Product and authoritative sources

| Concern | Source | Meaning |
| --- | --- | --- |
| Linux process actions and permissions | `src/actions.rs`, `ROADMAP.md`, `QA.md` | Confirm signals; guard PID reuse, PID 1, self and permission failures |
| Sampling and information accuracy | `src/monitor.rs`, `ROADMAP.md` | Display actual `/proc` values and explicit unavailable states |
| Appearance and transient overrides | `src/config.rs`, `src/ricepack.rs`, `RICEPACK_FORMAT.md` | Strict data-only tokens; never persist session pack appearance by accident |
| Visual identity and renderer | `DESIGN.md`, `src/theme.rs`, `src/ui/rices/` | Each rice owns its client look; the WM owns window chrome |

Reviewed: 2026-11-06. UI language: English. Intended platforms: Linux X11
and Wayland. Accessibility goal: readable contrast, visible keyboard focus,
functional labels and real-device WCAG-equivalent checks where applicable.

## Canonical UI ownership

| Capability | Owner | Variants | Verification |
| --- | --- | --- | --- |
| Process table selection and scroll | `src/ui.rs` / `src/ui/rices/` egui widgets | Theme-specific rows, same `(PID, start_ticks)` identity | Rust UI layout and action tests |
| Search, sort and filters | `TaskottoApp` state, shared monitoring data | Era-specific controls | Rust UI layout and filter tests |
| Form widgets, menus and focus | `egui` native toolkit | Styles from `ThemeStyle` | Headless frame tests, hardware QA |
| Process action confirmation | `TaskottoApp` + `src/actions.rs` | Terminate, kill, pause, resume | Unit tests + disposable process tests |
| Status/errors | `TaskottoApp` and corresponding rice view | Loading, empty, invalid pack, inaccessible metrics | Unit/integration plus GUI acceptance |

No date picker, billing form, web toast provider, browser scrollbar CSS or
server pagination exists; do not invent equivalents to fill a template.

## Main interactions

| Operation | Trigger | Pending | Success | Failure/recovery |
| --- | --- | --- | --- | --- |
| Process refresh | F5 / Refresh | Keep old data and show refresh state | Update snapshot and tables | Report `/proc` error, retain UI |
| Search/filter/sort | Search or table control | Local state update | Show matching processes | Empty state with clear search |
| Inspect | Select a process/Details | None | Show actual process information | Show vanished or inaccessible process clearly |
| Signal | Chosen process + action | Explicit confirmation | Report resulting action | PID reuse, permission, no longer running: reject safely |
| RicePack preview | Library/CLI pack selection | Validate up front | Session-only appearance | Invalid pack retains last good appearance |
| Pinned personal dashboard | Choose a valid file → Use at startup | Validate absolute TOML and save user-only path | Next GUI run applies the transient layout; explicit launch overrides win | Missing saved file opens regular rice with actionable warning; clear or repair the path |
| Export design | Explicit path | Write a new file atomically | Re-importable TOML | Never overwrite an existing destination |

## Stability and verification

All controls must remain keyboard-operable and keep accessible focus feedback.
Data tables never silently substitute a system-wide metric for a per-process
metric. User-facing action errors identify what failed and whether retrying
or changing permissions may help. Compact windows may hide *optional* columns,
never remove the process identity or signal confirmation path. The native
window manager owns external decorations.

Before merging UI changes, run `cargo fmt --check`, Clippy, all tests, release
build, generated egui screenshot checks and a regression check of neighboring
rices. Real Mint/X11, Fedora KDE/Wayland and RiceBowl/labwc acceptance remains
open until user hardware or dedicated desktop infrastructure supplies evidence.
