---
version: alpha
name: Taskotto
description: "An honest Linux task manager with independent, era-specific desktop interfaces and safe generated RicePack palettes"
colors:
  system-surface: "#192433"
  system-text: "#E7EDF7"
  system-accent: "#82B4FF"
  aero-surface: "#E2F3EE"
  aero-text: "#184A4F"
  aero-accent: "#259572"
typography:
  body:
    fontFamily: "system-ui, sans-serif"
    fontSize: "14px"
  data:
    fontFamily: "system-ui, sans-serif"
    fontSize: "12px"
omitted:
  - section: rounded
    reason: "The eight historical rice profiles intentionally have different corner shapes; src/theme.rs owns each runtime value."
  - section: spacing
    reason: "User-defined density and spacing are normalized dynamically by Settings and ThemeStyle, not a fixed design-document scale."
---

# Taskotto Design System

## Overview

Taskotto's north star is a **genuine Linux process manager that feels native to
the chosen desktop era**: compact Win98 lists, XP Luna, Windows 7 Aero,
Windows 10, macOS Activity Monitor, ChromeOS, aquatic Frutiger Aero, and an
independent system view. These are deliberately distinct product skins, not a
single neutral dashboard with swapped accent colors. All imagery must be
original; the active window manager/RiceBowl draws the outer window frame.

Audience: Linux desktop users choosing a lightweight on-demand system tool and
creators/AI workflows building safe, data-only themes. Platform: Rust/egui on
Linux X11 and Wayland. Product language is currently English; no unverified
localization support is promised. Primary jobs: search and inspect real
processes, review honest performance data and perform explicitly confirmed
process actions. Memorability belongs to the chosen rice; restraint belongs to
functional tables and safety dialogs. Do not homogenize all profiles into
generic cards, fabricate metrics, or borrow proprietary OS artwork.

**Token ownership:** `src/config.rs` and `src/theme.rs` are canonical, with
versioned user/AI tokens described in `RICEPACK_FORMAT.md`. This document
mirrors established values; it does not generate Rust constants. The six
frontmatter colors are exact `ThemeStyle::from_settings` system/Frutiger
defaults. Runtime custom component tokens (`control`, `hover`, `border`,
`table`, `selection`) override fallback colors only when present.

## Colors

Colors have two jobs: express the selected desktop era and maintain readable
system data. The system baseline uses navy `#192433`, pale text `#E7EDF7` and
blue `#82B4FF`. The Frutiger Aero client uses mint `#E2F3EE`, ink `#184A4F`
and green `#259572` plus original aqua, sky and leaf highlights. Other rice
palettes remain in `ThemeStyle::from_settings` and their dedicated modules;
they are intentionally not forced onto the Aero palette. Generated colors
have contrast fallbacks; custom selected rows require readable foreground.

## Typography

egui's runtime proportional font is canonical. Normal data/control size is
approximately 14px (13px for Classic 98); dense secondary data uses smaller
sizes. Numeric alignment and truncation must remain legible at narrow window
widths. `font_scale` is user-adjustable from 0.8 to 1.5, and no downloaded
font or proprietary font is required.

## Layout

The process table is virtualized. Real pages adapt to compact minimum windows
and wide desktops; optional columns give way to process name, PID and core
metrics. Density, row height, navigation and spacing are user-owned tokens.
`chart_scale` changes chart height without changing each rice's relative
proportions; 1.0 preserves the historical defaults.
Pills in Frutiger Aero are intentionally unlike the square Classic 98 controls.
Do not impose a fixed web-style viewport model on egui.

## Elevation & Depth

Flat historical panels use precise borders and inset states. Frutiger Aero's
original translucent bubbles and charts add decorative depth without making
tabular data ambiguous. No extra shadow/titlebar should mimic the compositor.

## Shapes

Corner radius is preset-specific (`0` for Classic 98/XP/Windows 10, `9` for
Frutiger Aero defaults) and may be overridden for shared widgets by a validated
RicePack. Hand-drawn illustration shapes stay recognizable as the original
rice's visual identity.

## Components

`ThemeStyle::apply` owns egui control/hover/border/selection visuals.
`src/ui/rices/` owns each custom client. Shared `ComponentPaint` in
`src/theme.rs` resolves optional RicePack tokens; Frutiger Aero custom
navigation/process/chart surfaces, macOS Activity Monitor's custom tabs/process
report, ChromeOS's adaptive table palette, Windows 7's classic tab/process
reports, the shared XP Luna/Windows 10 table and navigation widgets, and
Windows 98's beveled buttons and classic Close Program list use it with
era-specific fallbacks. Secondary-page statistics, data grids, process Info
and performance selectors adopt the same background/border colors and
contrast-aware text while keeping distinctive era-specific artwork.
Menu and status strips are native client UI rather than duplicate top-level
titlebars. Their optional control/table fills and status borders derive from
the same semantic tokens; dark global surfaces must not make fixed light
historical strips unreadable.
`readable_text` provides contrast-aware
foreground for user-selected colors. Loading/empty/error states describe the
real `/proc` state. Keyboard focus is visible, destructive actions require
confirmation, and unsupported permissions are reported as such.

Optional `layout.pages` is a typed, bounded, data-only screen description.
It reuses the real Linux sampler and confirmation-protected process actions;
it cannot render proprietary assets, invoke commands or introduce fake
measurements. Original clients remain the default unless a pack asks for
named widgets. A future in-app editor must serialize this exact contract.
The toolbar vocabulary is fixed to refresh/search/confirmed termination;
gauges and status strips only bind real, explicitly named host metrics.
P2 screen variants may supply a bounded `compact_widgets` list at an explicit
content-width breakpoint. It selects an alternative **validated** composition
on resize; legacy layouts and all original historical rices retain the
normal widget list when no compact alternative exists.

The interface is live and honest: charts use measured bounded history;
labels distinguish system-wide traffic from per-process information. Motion
is limited to genuine loading and sampling status; the application must not
force a continuous idle repaint.

## Do's and Don'ts

- Do keep each historical preset's silhouette and functional information hierarchy.
- Do use optional semantic component tokens for generated themes, with safe fallbacks.
- Do validate small/narrow and wide desktop layouts with genuine egui captures.
- Don't duplicate the compositor's window title, border or close/minimize buttons.
- Don't invent Linux statistics, imply unsupported process actions, or add always-on agents.
