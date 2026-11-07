# Frutiger Aero / Windows 98 / XP Luna / Windows 7 / Windows 10 / ChromeOS / macOS rice fidelity

Taskotto is Linux system software. The goal is **recognizable task-manager
interaction and visual hierarchy**, not a falsely branded Windows program.
Only original vector-style drawing and standard egui controls are used.

## References and design direction

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
- Google's [Chromebook Task Manager shortcut and guidance](https://support.google.com/chromebook/answer/6309225?hl=en) and [Chrome process management](https://support.google.com/chrome/answer/2391819?hl=en) for table-first process inspection and End process behavior.

These are design references; their screenshots, icons, product logos, textures
and fonts are **not** copied or bundled.

## ChromeOS Task manager

- A dedicated, minimalist browser-style client area, without duplicating the compositor/RiceBowl window frame, close buttons or a Google logo. The standard **Auto** layout is a single task table; optional sidebar/top tabs are explicit Taskotto preferences.
- Persistent search, **All tasks / My tasks** filter, sortable Task / Memory footprint / CPU / Process ID headers, and responsive optional Disk I/O and User columns. Task names and ownership come from Linux `/proc`, not an imagined mapping to browser tabs, extensions or Android applications.
- Per-process memory means Linux **resident set size (RSS)**, not Chrome's proprietary private memory accounting. Chrome's per-process Network figure is unavailable from portable Linux `/proc`, so the optional column is transparently *Disk I/O*; missing permission is shown as N/A instead of zero. The supplementary Network tool shows correctly labeled **system-wide** RX/TX activity.
- Selection, hover, focus and context actions work; double-click shows process details. The blue End process button is unavailable until a permitted process is selected, then runs through the existing confirmation and pidfd safety gate.
- An overflow menu keeps Performance, Network, Users, Details, Applications, App history, Startup, Services, filters, refresh rate, New Task and settings accessible without crowding the primary pane.
- Appearance is a ChromeOS-inspired interpretation rather than a pixel-perfect verified release. The default egui font and Linux compositor differ; native visuals and keyboard testing remain necessary before claiming fidelity.

## Frutiger Aero (original Y2K-inspired design)

- Dedicated, original aqua/sky/leaf client surface with a vector bubble activity mark, glass-like layered gradients, meter rings and rounded navigation. The reference is a broad *Frutiger Aero* design language rather than a branded Windows or Mac Task Manager; no OS-specific wallpaper, artwork or proprietary fonts are bundled.
- Seven destinations (Processes, Performance, Network, Users, Services, Details, Settings) operate on Taskotto's existing functions. The default is wide pill-shaped horizontal tabs; opting for a themed sidebar is an explicit RiceBowl/taskotto customization.
- Process Garden offers actual Linux PID/name/command/owner, CPU/RSS and permission-dependent I/O readings; live search, sorting, child-process tree mode, process selection, details and terminate-with-confirmation all use the same protected signal mechanism as other rices. Column density responds to viewport width.
- System Vitals features four genuine rolling charts (CPU, memory, disk read/write, network receive/send) and circular percent meters; displays do not invent CPU, GPU or battery readings. Network is explicitly non-loopback **system-wide** traffic, which can double-count virtual interfaces.
- The masthead is **application content**, not simulated X11/Wayland title-bar buttons, borders or caption. Native decorations belong to RiceBowl/the compositor. Users/Services/Details/Settings reuse validated Linux functionality; fully custom Aero detail page designs can be iterated without sacrificing those controls.
- Headless screenshots demonstrate the egui surfaces but not native input latency, compositor styling, accessibility tree or exact monitor color. Physical Linux Mint/Fedora and labwc checks remain in release QA.

## Windows 98 Close Program

- The original compact **Close Program** hierarchy is the first screen: short instruction, sunken white running-program list, square gray bevel buttons, dark-blue selection, and a small status line. There is no fabricated Windows title bar, close button or frame in the Taskotto client: RiceBowl/the native window manager owns window decoration.
- The list alphabetizes current-user processes and excludes kernel threads. Linux `/proc` does not give portable compositor-independent window ownership, so the UI identifies these accurately as **processes**, not a list of GUI windows or a 'Not Responding' detection mechanism.
- **End Task** operates only on a currently visible, still-matching process, with the existing PID start-time/pidfd verification, explicit confirmation, no elevated privileges, and accurate error reporting. **New Task** launches only a command the user enters; **Refresh** and **Close** work immediately.
- **Tools >>** opens optional period-styled Processes, Performance, Network, Users and Options. The advanced process report provides sorting, search, PID/CPU/RSS details and confirmed End Process; graph values are actual CPU/memory history. The historical primary dialog remains uncluttered.
- Default viewport is 650×450 (minimum 490×340) for the compact dialog; advanced pages adapt to resizing. Colors/fonts and other RiceBowl tokens remain configurable. No proprietary graphics, system libraries or historical Windows APIs are included.
- The result is a faithful design *direction*, not verified 1:1 historical pixels. Native screenshots, system font comparisons and keyboard/accessibility validation remain in QA.

## macOS Activity Monitor

- Dedicated light, neutral Activity Monitor-inspired UI rather than the generic
  Taskotto theme: an original activity-wave mark, compact toolbar with **Stop**,
  **Info**, functional **View** menu and a persistent **Search processes** field.
- The Linux process Info inspector may additionally show an explicitly
  labeled *observed descendant tree* summary. Summed RSS can overlap shared
  pages and is not a claim about macOS private/unique memory accounting.
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

- Pale blue Aero-era **client** surface, functional menu, beveled rectangular
  tabs and a classic status strip for process count, CPU usage and physical
  memory. The outer Aero title treatment comes from RiceBowl/the compositor,
  never from an in-client imitation.
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

- Cream menu strip, raised tabs, a white list surface filling the process
  pane, and XP-era `Image Name`, `Mem Usage` and `User Name` headers.
  RiceBowl owns the blue Luna title gradient and window buttons; Taskotto
  deliberately does not duplicate that 32 px native title area.
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

- For all eight themes, Taskotto explicitly requests a **compositor/window-manager
  owned native frame** on X11 and Wayland. XP, Seven and Ten have *only*
  application menus and tabs, never an extra title strip. ChromeOS and macOS
  retain functional toolbars; Frutiger Aero retains a distinct application
  masthead. No in-client top-level window-control buttons are painted.
- Windows 10 exposes seven data-backed tabs; Windows 7 exposes six; XP
  exposes five. The original Windows APIs and historic App history accounting
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
- The README gallery and provenance describe all 37 current client-only
  captures across seven dedicated visual styles; old generic renders were
  removed, while all eight rice presets remain available.
