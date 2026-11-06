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
```

## Use

```bash
taskotto --check-ricepack examples/ricepacks/sea-glass.toml
taskotto --ricepack examples/ricepacks/sea-glass.toml
```

For RiceBowl session integration:

```bash
RICEBOWL_TASKOTTO_RICEPACK=/absolute/path/taskotto-rice.toml taskotto
```

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

Unknown fields, unrecognized versions, invalid colors, NaN/out-of-range
values and files larger than **64 KiB** are rejected with a descriptive error.
Contrast fallback still applies to requested foreground colors.
No network request, plugin, parser evaluation or shell execution is involved.

The outer window frame is managed by RiceBowl/the active compositor;
RicePacks cannot paint duplicate titlebars or alter window permissions.

## Roadmap boundary

This contract does not yet support custom icons, fonts, free-form widget
placement or user-defined data bindings. Those require a separate bounded
layout schema, migration tests and visual QA under P1/P2, not unchecked TOML
keys pretending to implement functionality.
