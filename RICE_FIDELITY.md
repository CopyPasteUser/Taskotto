# Windows 10 / XP Luna task-manager rice fidelity

Taskotto is Linux system software. The goal is **recognizable task-manager
interaction and visual hierarchy**, not a falsely branded Windows program.
Only original vector-style drawing and standard egui controls are used.

## References and design direction

- Historical Windows 10 Task Manager processes/navigation examples:
  [BleepingComputer Task Manager guide](https://www.bleepingcomputer.com/tutorials/close-a-program-using-task-manager/).
- Historical Windows XP task-manager processes example:
  [San José State University CS149](https://www.cs.sjsu.edu/faculty/pearce/cs149/multiproc.htm).
- Functional Windows task-manager concepts:
  [Microsoft Learn](https://learn.microsoft.com/en-us/troubleshoot/windows-server/support-tools/support-tools-task-manager).

These are design references; their screenshots, icons, product logos, textures
and fonts are **not** copied or bundled.

## XP Luna

- Blue vertical-gradient, original app-mark and cream menu strip.
- File, Options, View, Help menus with functional refresh, filtering, tree,
  column, preferences and close actions.
- Square, beveled tabs and clickable, sortable white process-table headers.
- Selection, double-click for Details and right-click signal actions with
  the normal confirmation flow.
- Fixed bottom status strip: process count, CPU, memory and End Process.
- Classic high-contrast green-on-black CPU and memory history with usage gauges
  and a hardware summary.

## Windows 10

- Light, restrained task-manager chrome, thin dividers, flat active tabs and
  familiar menu hierarchy.
- Dense process rows, accurate aggregate CPU/memory header figures,
  proportional blue resource-cell heatmaps and sortable column headers.
- Actual Linux owner-based grouping into **User processes** and **Other
  processes**. Taskotto does *not* claim every Linux process is a GUI app.
- Performance view with live CPU, Memory, Disk and Network sections,
  miniature trend graphs, a larger focused graph and core counters.
- Responsive columns, accessible focus rings and a fixed, functional End task
  button. No empty tab or decorative control that purports to work.

## Intentional differences and verification

- The compositor/window manager owns the **native window frame** on X11 and
  Wayland. The in-client bars are styled headers, not replacement decorations
  or simulated OS window buttons.
- Taskotto exposes four real pages (Processes, Performance, Details, Settings).
  Windows-only task-manager tabs such as App history/Startup/Services are not
  invented with empty content; future capabilities must be genuinely backed by
  Linux data and behavior.
- Fonts use Taskotto/egui's redistributable default glyphs. Exact proprietary
  Windows text rasterization, asset sets and system dialogs are out of scope.
- Profile style is configurable; custom palettes and the explicit layout
  override remain supported, so a user can depart from the historical look.
- Headless screenshot renders prove actual egui layout/paint commands and real
  procfs values, **not native desktop output**. [QA.md](QA.md) retains the Mint
  X11, KDE Wayland and labwc visual/keyboard/performance sign-off gates.
