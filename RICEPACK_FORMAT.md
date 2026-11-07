# Taskotto RicePack format v1

RicePacks are portable **data-only** TOML design presets for Taskotto.
RiceBowl, a user or an AI generator can create them. A pack does not contain
code, shell commands, downloaded assets or monitoring permissions.
This format covers validated appearance tokens and an optional, strictly
bounded set of **data-bound custom screens**. Entirely arbitrary components,
artwork and programmatic layouts remain on the
[expansion roadmap](EXPANSION_ROADMAP.md).

Example: [Sea Glass](examples/ricepacks/sea-glass.toml)

```toml
format_version = 1
name = "Sea Glass"
base_theme = "frutiger-aero"

[appearance]
accent = "#159DA4"
surface = "#E8FBF7"
text = "#143940"
density = "comfortable"
navigation = "top-tabs"
font_scale = 1.05
spacing = 9.0
chart_scale = 1.2
corner_radius = 10
row_height = 34.0

[components]
control = "#D4F2EC"
hover = "#B6EAE0"
border = "#80C8BE"
table = "#F7FFFD"
selection = "#148E94"
```

### Custom screens (P2 widgets, RC30)

If `[layout]` is absent, all original rice screens remain unchanged. When it
is present, the active RicePack replaces only the **Processes client page**
with its data-defined mini-dashboard; the existing Preferences and other
functional pages remain available. This is an appearance/session override,
not a change to sampling rights or saved preferences.

```toml
[[layout.pages]]
id = "overview"
title = "Overview"

[[layout.pages.widgets]]
kind = "metric"
label = "CPU load"
binding = "cpu-percent"

[[layout.pages.widgets]]
kind = "plot"
label = "Recent CPU activity"
binding = "cpu"

[[layout.pages.widgets]]
kind = "process-list"
columns = ["name", "cpu", "memory", "pid"]
```

- `layout.pages`: 1–4 tabs. Each `id` is a unique lowercase ASCII slug
  (1–24 characters: `a-z`, `0-9`, `-`); `title` is 1–40 printable characters.
- Optional `layout.compact_below` is a **content area** breakpoint of 480–1024
  logical pixels (default: 680), measured after Taskotto's own navigation.
  Each page may include up to 16 optional `[[layout.pages.compact_widgets]]`
  with the exact same validated kinds and bindings as its normal `widgets`.
  Below the breakpoint Taskotto renders the compact set; if omitted, the
  normal widgets remain visible at every width, preserving legacy packs.
- Each tab has 1–16 widgets. `heading` accepts `text` (1–120 printable
  characters), `metric` and `plot` accept `label` (1–48) and a typed `binding`.
- `metric` bindings: `cpu-percent`, `memory-used`, `memory-total`,
  `memory-percent`, `processes`, `uptime`, `network-received`, `network-sent`,
  `disk-read`, `disk-write`.
- `plot` bindings: `cpu`, `memory`, `network-received`, `network-sent`,
  `disk-read`, `disk-write`, `cpu-memory`, `network-traffic`, `disk-traffic`
  (observed session samples, no invented counters). The three comparison
  bindings overlay two independently measured series on **one** y-axis:
  CPU/RAM both use 0–100%, while network receive/send and disk read/write
  each share the maximum observed byte-per-second scale. Lines are labeled
  and no byte rate is compared against a percentage.
- `gauge` accepts a `label` (1–48) and a percentage `binding` of either
  `cpu-percent` or `memory-percent`. It reads live Linux samples and displays
  an explicit waiting state until data is available.
- `status-strip` accepts 1–5 distinct named `metric` bindings, displayed
  compactly with the same live values as metric tiles.
- `toolbar` accepts 1–3 distinct `actions`: `refresh` (request another
  sample), `focus-search` (show and focus the process search), and
  `end-process` (open the existing PID-safe termination confirmation for the
  currently selected process). These are predefined application actions;
  arbitrary commands, scripts, and network calls cannot be expressed.
- `process-list` accepts 1–5 unique columns chosen from `name`, `pid`, `cpu`,
  `memory`, `threads` (including required clickable `name`). Rows are
  virtualized. Selection and End Process follow
  the same PID/start-tick safety check and confirmation flow as built-in rices.

All other fields, kinds, bindings, scripts and shell hooks are rejected.
Generated screens retain keyboard process search via `Ctrl+F`. If a page has
no search-enabled toolbar, Taskotto opens a temporary focused search control
instead; dismissing either search clears its filter.
Try the complete two-tab [Diagnostic Studio](examples/ricepacks/diagnostic-studio.toml)
or the live [Comparison Studio](examples/ricepacks/comparison-studio.toml),
which overlays related history samples and retains an independently usable
process page plus a compact fallback:

```toml
[layout]
compact_below = 720

[[layout.pages]]
id = "overview"
title = "Overview"

[[layout.pages.widgets]]
kind = "process-list"
columns = ["name", "cpu", "memory", "pid"]

[[layout.pages.compact_widgets]]
kind = "process-list"
columns = ["name", "pid"]
```

The snippet illustrates the alternative list, not a complete RicePack. When
the content area is narrow, it shows fewer verified Linux process columns
without losing selection/confirmation. No CSS/scripts or runtime permissions
are introduced. At wide sizes the original process table is retained.

```bash
taskotto --check-ricepack examples/ricepacks/diagnostic-studio.toml
taskotto --ricepack examples/ricepacks/diagnostic-studio.toml
```

`--check-ricepack` also reports the validated screen count, normal/compact
widget counts and breakpoint. If an AI produces a missing, outdated or future
`format_version`, Taskotto rejects it with an explicit compatibility/migration
diagnostic before attempting to interpret unsupported widget extensions.
There is no automatic or silent migration; a compatible generator must
re-export the pack in the supported v1 contract.

## Use

```bash
taskotto --check-ricepack examples/ricepacks/sea-glass.toml
taskotto --ricepack examples/ricepacks/sea-glass.toml
```

To create a starting point from your **saved** Taskotto appearance, without
opening the GUI:

```bash
taskotto --export-ricepack my-rice.toml
# Edit the exported TOML manually or with an AI, then:
taskotto --check-ricepack my-rice.toml
taskotto --ricepack my-rice.toml
```

### In-app library and preview (RC17)

Open **Preferences → RicePack library**. The local library lives in
`$XDG_CONFIG_HOME/taskotto/ricepacks/` (normally
`~/.config/taskotto/ricepacks/`). Taskotto discovers up to 128 `.toml`
files without scanning subdirectories, reports invalid files, and excludes
symlinks. Click **Refresh library** after adding files yourself, or select
a design in the list. You can also enter a RicePack file path directly.

- **Preview design** temporarily switches Taskotto's current appearance;
  **Revert preview** restores the previous appearance and active file watcher.
- **Keep for session** applies the preview for this run and resumes live
  reloads. Neither preview nor keep silently changes saved appearance.
- **Use this RicePack at startup** validates an absolute `.toml` file path
  and saves only that path in the current user's Taskotto preferences. Next
  GUI launch loads the selected dashboard/palette as a still-transient pack;
  its colors and screen layout never overwrite the underlying saved profile.
  **Clear startup RicePack** removes that association immediately. Missing or
  malformed pinned files show a recoverable GUI warning and fall back to the
  saved rice; explicit `--ricepack` or pack environment variables override
  the saved default and remain strict errors when invalid.
- **Import to library** validates a source file and creates a distinct new
  file. Repeated imports get numbered suffixes; existing files are untouched.
- **Export current appearance** writes the visible settings to a new file.
  You choose the destination; an existing file is never overwritten.

The Component colors section includes optional swatches for button/control,
hover, border, table-background and selection colors. Each field can be reset
to the original rice preset; older packs without `[components]` keep exactly
their existing behavior. Control/selection text uses automatic contrast
fallback; a table fill that would make inherited text unreadable is ignored.

This is an interactive editor for the **v1 supported appearance tokens**, not
arbitrary scripts or visual assets. **Screen layout designer (RC32)** appears
below the library controls in Preferences. It can compose and reorder the
schema's seven safe widget kinds, edit captions, typed data sources, action
sets and process columns, and maintain independent standard/compact widget
lists for up to four pages. The active design is not altered while the draft
is incomplete or invalid. **Apply to this session** previews the validated
layout in the Processes view without writing any files; use the existing
**Export current appearance** to create a new, non-overwriting portable TOML
file. **Discard draft** and **Reset draft from active layout** preserve the
currently running design. The separate RiceBowl desktop RicePack
JSON/ZIP formats cannot be loaded as Taskotto TOML; RiceBowl must export a
`taskotto-rice.toml` and pass its absolute path through the environment.

While Taskotto stays open with a RicePack, edited files are checked about every
**two seconds**. Only fully valid, changed packs are applied; a missing,
partial or malformed file leaves the last valid appearance running and shows a
recoverable warning. When the file becomes valid again, its new design is
applied, including removal of now-absent overrides (which fall back to the
user's original saved appearance). An explicit `--theme` continues to win.
Ordinary launches without a RicePack have no extra reload timer.

Export reads the ordinary Taskotto config (not a temporary active session
override), writes only supported appearance tokens and never overwrites an
existing file. It preserves the original config and refuses malformed
preferences instead of silently exporting defaults. The destination directory
must already exist; on Linux the new RicePack file is owner-readable/writable.
Process and monitoring preferences are intentionally **not** part of the pack.

For RiceBowl session integration:

```bash
RICEBOWL_TASKOTTO_RICEPACK=/absolute/path/taskotto-rice.toml taskotto
```

With a RiceBowl installation supporting the v1 bridge, export a concrete
Taskotto design from an existing desktop recipe without writing to Taskotto's
preferences:

```bash
ricebowl de taskotto-export --recipe classic --output /absolute/path/taskotto-rice.toml
taskotto --check-ricepack /absolute/path/taskotto-rice.toml
taskotto --ricepack /absolute/path/taskotto-rice.toml
```

RiceBowl's version-controlled
[`taskotto-classic-v1.toml`](https://github.com/CopyPasteUser/Ricebowl/blob/main/tests/fixtures/taskotto-classic-v1.toml)
and Taskotto's [matching integration fixture](tests/fixtures/ricebowl-classic-v1.toml)
are compared as part of cross-repository development. The Taskotto integration
test validates the fixture through the production parser and headless CLI.
RiceBowl's JSON/ZIP desktop pack is never fed directly into this TOML parser.

`TASKOTTO_RICEPACK` is also supported for non-RiceBowl launchers. Selection
order: explicit `--ricepack` > `RICEBOWL_TASKOTTO_RICEPACK` >
`TASKOTTO_RICEPACK`. An explicit `--theme` can override the pack's
`base_theme`; otherwise a selected pack overrides `RICEBOWL_RICE`.
Without a pack, behavior is unchanged.

The pack's appearance is **session-only**. Saving other preferences, including
the monitoring refresh interval, does not copy pack colors or layout into
`~/.config/taskotto/config.toml`. Appearance adjustments made while a pack
is active are also transient. Quit and reopen without the pack to restore
the user's saved design.

## Contract

| Field | Required | Allowed values |
| --- | --- | --- |
| `format_version` | Yes | Integer `1` |
| `name` | Yes | 1–64 printable Unicode characters |
| `base_theme` | Yes | `system`, `windows98`, `xp-luna`, `windows7`, `windows10`, `frutiger-aero`, `chromeos`, `macos` |
| `appearance.accent` | No | `#RRGGBB` |
| `appearance.surface` | No | `#RRGGBB` |
| `appearance.text` | No | `#RRGGBB` |
| `appearance.density` | No | `compact`, `comfortable` |
| `appearance.navigation` | No | `auto`, `sidebar`, `top-tabs` |
| `appearance.font_scale` | No | 0.8–1.5 |
| `appearance.spacing` | No | 3–16 |
| `appearance.chart_scale` | No | 0.75–1.5; multiplies performance chart heights |
| `appearance.corner_radius` | No | 0–16 |
| `appearance.row_height` | No | 22–48 |
| `components.control` | No | `#RRGGBB` default control fill |
| `components.hover` | No | `#RRGGBB` hover fill |
| `components.border` | No | `#RRGGBB` control/widget border |
| `components.table` | No | `#RRGGBB` shared table/content fill |
| `components.selection` | No | `#RRGGBB` selection fill (contrast-safe text) |
| `layout.pages` | No | 1–4 strictly typed screens with at most 16 supported widgets each |

Unknown fields, unrecognized versions, invalid colors, NaN/out-of-range
values and files larger than **64 KiB** are rejected with a descriptive error.
Contrast fallback still applies to requested foreground colors.
No network request, plugin, parser evaluation or shell execution is involved.

The outer window frame is managed by RiceBowl/the active compositor;
RicePacks cannot paint duplicate titlebars or alter window permissions.

## Roadmap boundary

Component tokens apply to shared egui widget visuals; some historical rice
drawings still contain intentionally hardcoded era-specific colors. Frutiger
Aero's custom navigation, process list and chart panels consume these
tokens; macOS Activity Monitor, ChromeOS tables, Windows 7 classic reports,
XP Luna/Windows 10 task tables and navigation, and Classic 98's beveled
controls and process lists use the shared parser. Windows 7 Users/Services,
XP networking/performance, Windows 10 performance selector/chart, macOS
process inspector and Aero network cards also accept semantic table/border
tokens. The original window-manager-owned frame is never recolored by
Taskotto. Historical menu and status strips also consume optional control or
table tokens while preserving their normal era-specific defaults. Try
[Aero Lagoon](examples/ricepacks/aero-lagoon.toml),
[Nightshift](examples/ricepacks/macos-nightshift.toml),
[Nocturne](examples/ricepacks/chromeos-nocturne.toml),
[Dusk](examples/ricepacks/win7-dusk.toml),
[Midnight](examples/ricepacks/xp-midnight.toml),
[Indigo](examples/ricepacks/win10-indigo.toml) or
[Violet](examples/ricepacks/win98-violet.toml) for contrasting original
designs without Rust edits. Complete per-widget
token extraction across the remaining rices remains on P1. This contract does not yet support
custom icons, fonts, free-form widget placement or arbitrary user-defined data
bindings; screen data can only use the explicit, verifiable bindings above.
Those require a separate bounded
layout schema, migration tests and visual QA under P1/P2, not unchecked TOML
keys pretending to implement functionality.
