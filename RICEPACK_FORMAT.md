# Taskotto RicePack format v1

RicePacks are portable **data-only** TOML design presets for Taskotto.
RiceBowl, a user or an AI generator can create them. A pack does not contain
code, shell commands, downloaded assets or monitoring permissions.
This first format covers **shared** tokens; theme-specific hardcoded widget
art and entirely new layouts remain on the
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
corner_radius = 10
row_height = 34.0

[components]
control = "#D4F2EC"
hover = "#B6EAE0"
border = "#80C8BE"
table = "#F7FFFD"
selection = "#148E94"
```

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
arbitrary UI widget placement. The separate RiceBowl desktop RicePack
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
| `appearance.corner_radius` | No | 0–16 |
| `appearance.row_height` | No | 22–48 |
| `components.control` | No | `#RRGGBB` default control fill |
| `components.hover` | No | `#RRGGBB` hover fill |
| `components.border` | No | `#RRGGBB` control/widget border |
| `components.table` | No | `#RRGGBB` shared table/content fill |
| `components.selection` | No | `#RRGGBB` selection fill (contrast-safe text) |

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
tokens, and macOS Activity Monitor and ChromeOS task tables use the shared
parser. Try [Aero Lagoon](examples/ricepacks/aero-lagoon.toml),
[Nightshift](examples/ricepacks/macos-nightshift.toml) or
[Nocturne](examples/ricepacks/chromeos-nocturne.toml) for contrasting original
designs without Rust edits. Complete per-widget
token extraction across the remaining rices remains on P1. This contract does not yet support
custom icons, fonts, free-form widget placement or user-defined data bindings.
Those require a separate bounded
layout schema, migration tests and visual QA under P1/P2, not unchecked TOML
keys pretending to implement functionality.
