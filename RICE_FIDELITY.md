# Windows 98 / macOS / Windows 7 / Windows 10 / XP Luna task-manager rice fidelity

Taskotto is Linux system software. The goal is **recognizable task-manager
interaction and visual hierarchy**, not a falsely branded Windows program.
Only original vector-style drawing and standard egui controls are used.

## References and design direction

- Windows 98's original **Close Program** dialog is illustrated in
  [GUIdebook's historical screenshots](https://guidebookgallery.org/screenshots/tasks/)
  and documented in the [Microsoft Knowledge Base archive (KB222518)](https://www.betaarchive.com/wiki/index.php/Microsoft_KB_Archive/222518).

- Historical Windows 10 Task Manager processes/navigation examples:
  [BleepingComputer Task Manager guide](https://www.bleepingcomputer.com/tutorials/close-a-program-using-task-manager/).
- Historical Windows XP task-manager processes example:
  [San José State University CS149](https://www.cs.sjsu.edu/faculty/pearce/cs149/multiproc.htm).
- Windows 7 historical processes and Services screenshots in the
  [Windows 7 Resource Kit](https://ptgmedia.pearsoncmg.com/images/9780735627000/samplepages/9780735627000.pdf)
  (published Microsoft Press material) and
  [Windows 7 configuration study guide](https://ugvet.ug.edu.ge/storage/books/June2021/TvtRE8oiIEpI6VnL9jM8.pdf).
- Functional Windows task-manager concepts:
  [Microsoft Learn](https://learn.microsoft.com/en-us/troubleshoot/windows-server/support-tools/support-tools-task-manager).
- Apple's [Activity Monitor process overview](https://support.apple.com/guide/activity-monitor/view-information-about-processes-actmntr1001/mac),
  [CPU](https://support.apple.com/guide/activity-monitor/view-cpu-activity-actmntr43452/mac),
  [memory](https://support.apple.com/guide/activity-monitor/view-memory-usage-actmntr1004/mac),
  [disk](https://support.apple.com/guide/activity-monitor/view-disk-activity-actmntr1005/mac), and
  [energy](https://support.apple.com/guide/activity-monitor/view-energy-consumption-actmntr43697/mac) views.

These are design references; their screenshots, icons, product logos, textures
and fonts are **not** copied or bundled.

## Window-decoration ownership contract (all profiles)

- **RiceBowl / the X11 or Wayland compositor** owns the actual top-level window
  title, border, window-shadow and theme-colored titlebar with the native
  minimize, maximize and close controls. Taskotto uses the ordinary decorated
  eframe/winit viewport; it does **not** draw extra in-client caption strips.
- **Taskotto** owns only the task manager's interior: menus, navigation,
  toolbar actions, reports, status displays and graphs. The earlier blue XP,
  gradient Seven and white Win10 imitation titlebars were removed. The macOS
  view retains Stop/Info/View/Search as genuine tools, not a duplicated title.
- In-client modal dialogs (`egui::Window` for properties/confirmation) remain
  local pop-ups with their own dismiss action; they are *not* independent
  top-level OS windows and cannot be decorated by RiceBowl.
- Headless PNGs show *only* client pixels. Real screenshots and frame colors
  depend on the user's RiceBowl compositor, X11/Wayland theme and window rules.

## Windows 98 Close Program

- Authentic Windows 98 uses the small **Close Program** task list from
  Ctrl+Alt+Delete, **not** the Windows NT/XP multi-tab Task Manager. Taskotto's
  Classic 98 default therefore has no fake historical tabs or caption strip.
- Compact default 650 × 510 client, classic #C0C0C0 dialog surface, inset white
  list, original-drawn neutral task icons, navy selection and beveled buttons.
  **End Task** is confirmation-protected through the existing Linux pidfd path,
  **Cancel** closes Taskotto and **Shut Down** is explicitly disabled so a
  monitoring tool never unexpectedly powers off the Linux machine.
- The original warning about pressing Ctrl+Alt+Del twice would be false on
  Linux and is intentionally replaced with an honest unsaved-data warning.
  `/proc` cannot enumerate GUI windows across X11/Wayland: the list represents
  current-user **Linux processes**, labeled as such, not inferred applications.
- A clearly nonhistorical **Tools** menu preserves full utility: an advanced
  sortable process report, live green-on-black System Monitor with Linux CPU
  and RAM histories, honest aggregate Network and UID-owner summaries, and
  individual selected-process information. No empty decorative UI controls.
- Keyboard search (Ctrl+F) and manual refresh (F5) still work; the custom
  classic list retains pointer selection, accessible focus and process actions.

## macOS Activity Monitor

- Dedicated light, neutral Activity Monitor-inspired UI rather than the generic
  Taskotto theme: a compact content toolbar with **Stop**, **Info**, functional
  **View** menu and a persistent **Search processes** field. Window title and
  app icon belong to the compositor and are not repeated inside the client.
- Five familiar segments in their original order: **CPU, Memory, Energy, Disk,
  Network**, displayed as horizontal tabs by default. Explicitly choosing a
  sidebar is a Taskotto customization, not a claim about Apple's default.
- Sortable, alternating-row process tables with clickable column headers,
  live filtering, safe PID-aware Stop/Quit/Force Quit actions, details on
  double-click or context menu, and process inspection in a separate sheet.
- Responsive CPU columns include process name, % CPU, threads and PID;
  available space adds CPU Time and user. Memory uses real resident RSS and
  virtual memory (not macOS app/private or compressed memory). The Disk table
  shows actual permission-limited process read/write counters and I/O rate
  columns at larger widths, displaying **N/A** when `/proc/PID/io` is denied.
- The bottom panel keeps historical **live rolling activity** and summaries
  for the selected tab; labels identify Linux-native units/scopes.
- **Energy Impact is not available** in portable Linux `/proc`. The Energy tab
  explicitly reports sampled per-process %CPU as an *activity proxy*, not
  macOS energy scores, watts, battery effects or App Nap state.
- **Per-process network traffic is unavailable** from the baseline Linux
  sampler. The Network tab deliberately shows true non-loopback aggregate
  receive/send rates without inventing per-PID values. Virtual interfaces may
  double-count traffic.
- Screen proportions, system title bar, control rendering, typography and
  proprietary macOS OS APIs differ; do not claim 1:1 or 95% pixel parity
  without paired native reference and X11/Wayland verification.

## Windows 7

- Pale blue Aero-era **client surface**, beveled rectangular tabs and classic
  status strip for process count, CPU usage and physical memory. The old
  simulated blue titlebar was removed; RiceBowl decorates the real frame.
- The original six-tab order: **Applications, Processes, Services, Performance,
  Networking, Users**. Each tab has a Linux-backed screen rather than a
  nonfunctional visual placeholder.
- Processes use a dense white report with classic `Image Name`, `User Name`,
  `CPU`, `Memory (Working Set)` and `Description` columns. PID and I/O appear
  at sufficiently wide widths. The actual Description cell contains the
  Linux command line, not imagined Windows file-description metadata; memory
  is explicitly identified as Linux resident set size (RSS), not private
  working set. Header sorting, optional search, all-user filter and
  confirm-protected End Process actions work.
- Historical-style CPU usage gauge, real per-core CPU usage history (up to
  four visible at a time, limited to two at narrower widths), blue physical
  memory gauge/history, physical memory, swap, process/thread/uptime groups.
  Core histories have individually bounded queues and adjust to core count.
- Applications are truthfully labeled current-user *Linux processes*, not
  desktop windows; New Task launches a user-entered command. **Switch To**
  remains disabled until compositor-specific activation is supportable.
- Services shows asynchronously queried read-only `systemd --user` units,
  or a visible unavailable/error state. Windows service PIDs, group control
  and administrator privilege are not invented.
- Networking uses real non-loopback receive/send byte-rate history with an
  honest aggregate-interface caveat. Users lists actual process owners by UID,
  rather than claiming those UIDs are active graphical desktop sessions.
- Details/Properties opens as a floating sheet over the Processes tab, not a
  seventh historical tab; it retains real Linux fields and protected signals.
  Palette and border treatment follow Seven rather than Luna.

## XP Luna

- Cream menu strip; RiceBowl draws the blue Luna titlebar and window controls.
- Raised tabs, a white list surface filling the
  process pane, and XP-era `Image Name`, `Mem Usage` and `User Name` headers.
- The optional Taskotto process filter appears on Ctrl+F or View → Find,
  keeping the default view close to the period's table-first design.
- File, Options, View, Help menus with functional refresh, filtering, tree,
  column, preferences and close actions.
- Square, beveled tabs and clickable, sortable white process-table headers.
- Selection, double-click for Details and right-click signal actions with
  the normal confirmation flow.
- Fixed bottom status strip: process count, CPU, memory and End Process.
- Classic high-contrast green-on-black CPU and memory history with usage gauges
  and accurately labeled Total, Thread, Memory and Swap summary groups.
- The historical five-tab structure: Applications, Processes, Performance,
  Networking and Users. Applications lists current-user **processes**, explicitly
  not application windows (which need compositor-specific window-to-PID mapping).
- Networking displays real aggregate non-loopback RX/TX rate history on a
  high-contrast graph without inventing link speeds or adapter identities.
- Users groups Linux processes by UID; selected-process Properties uses compact
  period-style sections with live identification, CPU, memory, and I/O fields.
- XP Applications now has a working **New Task** dialog through the File menu
  and bottom action row. It launches explicitly entered commands as the current
  Linux user; Windows-style **Switch To** remains disabled until a compositor
  integration can reliably identify and activate the selected window.

## Windows 10

- Light, restrained task-manager chrome, thin dividers, flat active tabs and
  familiar menu hierarchy.
- Table-first Processes page; optional find field does not occupy default
  screen space. The performance graph now uses a soft fill under the live line.
- Dense process rows, accurate aggregate CPU/memory header figures,
  proportional blue resource-cell heatmaps and sortable column headers.
- Actual Linux owner-based grouping into **User processes** and **Other
  processes**. Taskotto does *not* claim every Linux process is a GUI app.
- Performance view with live CPU, Memory, Disk and Network sections,
  miniature trend graphs, a larger focused graph and core counters.
- Responsive columns, accessible focus rings and a fixed, functional End task
  button. No empty tab or decorative control that purports to work.
- The Users tab groups processes by Linux UID with expandable rows, actual
  per-owner CPU and RSS sums, and an inspect action. The Details tab shows an
  ungrouped list of live processes instead of reusing a generic properties pane.
- The familiar **seven horizontal tabs** now include App history, Startup and
  Services. The implementations are clearly labeled Linux equivalents:
  process-lifetime CPU totals for running processes; XDG desktop-session
  autostart entries; and asynchronously fetched systemd **user** service status.
  Startup and Services are read-only; genuine Windows app-history persistence,
  startup-impact estimates and Windows service controls are not claimed.
- **Sidebar** is a separate customizable vertical variant, available through
  Settings → Navigation. This satisfies an alternate layout preference but is
  deliberately **not** presented as Windows 10's historical default.

## Intentional differences and verification

- The compositor/window manager owns the **native window frame** on X11 and
  Wayland. Taskotto never overlays it with a second painted caption/titlebar.
- Windows 98 exposes its untabbed Close Program dialog; Windows 10 exposes
  seven data-backed tabs, Windows 7 six and XP five. The original Windows APIs and historic App history accounting
  do not directly exist on Linux. Settings and selected-process Properties
  remain accessible through menus and process actions.
- Linux cannot provide Windows process categories, application-history
  accounting or window ownership across all Wayland compositors from procfs
  alone. We label substitutes instead of silently misrepresenting them.
- Fonts use Taskotto/egui's redistributable default glyphs. Exact proprietary
  Windows text rasterization, asset sets and system dialogs are out of scope.
- Historical controls are approximated, not pixel-identical; egui font metrics,
  system window decorations, OS-only categories, and unavailable Windows APIs
  prevent a literal 1:1 match. This is a fidelity target, **not a certification**.
- Profile style is configurable; custom palettes and the explicit layout
  override remain supported, so a user can depart from the historical look.
- Headless screenshot renders prove actual egui layout/paint commands and real
  procfs values, **not native desktop output**. [QA.md](QA.md) retains the Mint
  X11, KDE Wayland and labwc visual/keyboard/performance sign-off gates.
- The README gallery and provenance describe 32 current 98/Mac/Seven/XP/Windows 10
  captures; old generic-theme renders were removed, while all eight rice
  presets remain available.
