# Taskotto QA — real hardware acceptance

Run this checklist before a stable 1.0 tag. Record distro, kernel, display server, compositor, CPU, and actual measurements. A green CI run is not a substitute for this list.

## Automated checks (no desktop session needed)

The Rust integration tests launch a disposable child, exercise SIGSTOP,
SIGCONT, SIGTERM and SIGKILL through the real pidfd API, and verify stale
start-tick rejection without killing that child. Additional tests exercise
the executable's JSON snapshot, invalid CLI options, two successive desktop
installs into a temporary XDG home, and symlink-safe atomic launcher updates.

```bash
cargo test --locked --all-targets
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
```

GitHub CI additionally checks the declared Rust 1.88 MSRV and documentation
warnings. The release-tag workflow verifies the archive checksum, extracts the
archive and invokes its binary and local desktop installer before publishing.
**These tests do not replace the interactive checks below.**

## Desktop behavior

- [ ] Windows 98: open `taskotto --theme windows98`, test the compact 650×450 (and minimum 490×340) Close Program view, gray 3D controls, user-process selection and End Task confirmation, New Task launch, Refresh, Close, Tools → all advanced views, and return to Close Program. Verify RiceBowl/native frame is the **only** top-level window decoration and keyboard navigation/focus work.
- [ ] XP Luna: verify RiceBowl/the WM provides the **only** blue Luna title chrome and top-right min/max/close buttons; Taskotto starts immediately with its cream File/Options/View/Help menu, raised tabs, classic CPU/RAM history, process sorting and status strip. Test keyboard focus and minimum resize.
- [ ] Windows 7: verify RiceBowl/the WM provides the **only** Aero-like title chrome and top-right controls; Taskotto starts with its actual menu and six historical tabs. Test report sorting/filtering, per-core CPU histories, blue memory graph, process confirmation, service unavailable/available states and keyboard selection at 650×420 and large size.
- [ ] Windows 10: verify RiceBowl/the WM provides the **only** native title bar and window controls. Inspect Taskotto's light File/Options/View/Help menu and horizontal tabs, process grouping, CPU/memory heatmap, sorting, context menus and CPU/Memory/Disk/Network performance navigation.
- [ ] macOS Activity Monitor: verify five horizontal CPU/Memory/Energy/Disk/Network segments, toolbar View filters, search, process selection, double-click Info, context-menu actions and continuously updated bottom history. Switch to optional Sidebar and back; resize to 650×420 and 1280×800. Note that Energy is a CPU proxy and Network is global-only by design.
- [ ] All seven dedicated rices plus System: verify exactly one native window frame, its correct RiceBowl-owned border/button colors, no duplicate title strips in Taskotto, and functional in-client menus and toolbars. Inspect a disposable process, use available safe signal actions after confirmation; verify no unintended actions or stale PID targets.
- [ ] Capture and review a native Taskotto window screenshot on X11, KDE Wayland and labwc. The [headless README images](SCREENSHOTS.md) do not certify display-server rendering.
- [ ] Linux Mint / X11: launch via `.desktop` menu, resize, minimize, close, reopen.
- [ ] Fedora KDE / Wayland: repeat and test clipboard/focus.
- [ ] labwc / Wayland: launch without XFCE/KDE services; confirm no missing runtime dependencies.
- [ ] At 650×420, at Win98 minimum 490×340, and at a large desktop resolution, all controls remain reachable.
- [ ] Tab through controls and use Ctrl+F, F5, Escape.
- [ ] With a live process list, press Ctrl+Shift+Space (and use Preferences → Process table) to freeze ordering. Verify values still change, a new child stays under its parent in tree mode, and unfreeze immediately restores the configured sort. Restart Taskotto to confirm freeze is not persisted. Check the generic **Freeze order** button as well.
- [ ] In generic Details or Preferences → Process follow, select a disposable process and click **Follow selected process**. Verify real CPU/RSS trends, stop/clear/retarget, window resize and keyboard navigation. Exit the process: no invented zero samples; after three missed samples tracking ends and never attaches to a reused PID. Restart: history is gone.
- [ ] Switch all 8 profiles, density, custom accent/surface/text, font scale, spacing, corners, table columns/row height and sidebar/top-tabs; reopen and verify persistence.
- [ ] Launch with `RICEBOWL_RICE=windows7 taskotto` and explicit `--theme xp-luna` override.
- [ ] Validate `examples/ricepacks/sea-glass.toml` with `--check-ricepack`, then run `--ricepack` on a real desktop. Verify base theme, palette/spacing/navigation and normal process actions; restart without the pack and ensure saved appearance is unchanged.
- [ ] Launch with `RICEBOWL_TASKOTTO_RICEPACK=/absolute/path/taskotto-rice.toml` and confirm the desktop's RiceBowl rice remains optional. Test malformed, oversized and inaccessible packs (clear error, no GUI crash), and confirm an explicit `--theme` overrides only the pack's base theme.
- [ ] With a pack active, change refresh interval and save. Check that monitoring options persist while temporary RicePack colors/layout do not; appearance edits made during that session remain temporary.
- [ ] Load `examples/ricepacks/diagnostic-studio.toml` on X11, KDE Wayland and labwc. Switch its two generated tabs; compare CPU, memory, network and disk values with `/proc`, confirm compact/wide layouts and that Process List selection followed by End Process still asks for confirmation. Exit without the pack and confirm the original rice returned. Reject a manually added `kind = "script"` widget via `--check-ricepack`.
- [ ] In Diagnostic Studio, test the generated **Refresh now** button, toggle **Search processes**, type a process filter and verify matching rows; check the **End selected process** button is disabled until a safe process is selected and still requires PID-safe confirmation. Verify memory gauge percentages against `free` and status strips against Linux sampling; the first frame without a sample must show a waiting state, not zero.
- [ ] Verify custom toolbar, gauge and status strip at 490×340, 650×420 and 1280×800, with keyboard-only navigation. Change custom component colors and check text/background contrast. Reject duplicate toolbar actions and unknown command-like actions with `--check-ricepack`.
- [ ] Confirm the launcher override does not replace the saved theme after changing another setting.
- [ ] Resize Diagnostic Studio through the configured 720 px **content-area** breakpoint. Confirm Overview and I/O switch between their distinct verified-metric widget sets, keyboard selection and confirmation remain usable, and old RicePacks without `compact_widgets` do not change appearance. Try malformed `compact_below = 300` and an over-16 compact widget list; CLI validation must reject both.
- [ ] Open Preferences → Screen layout designer. Create, rename, reorder and delete screens; add each of the seven component templates and alter typed bindings, column sets and toolbar actions. Copy the standard widgets to a compact fallback, edit the latter and verify the correct list appears on narrow windows. A temporarily invalid draft (empty title, duplicate ID or empty action set) must report an error and **must not** replace the current Processes layout. Discard and reset must preserve the previous layout. Apply valid changes, navigate to Processes to inspect, export to a new file, and use `--check-ricepack`/reimport to verify round-trip. The current config must remain unchanged.
- [ ] In both the full Diagnostic Studio and a pack whose only widget is a process list, press `Ctrl+F`, type and clear a query, and close the search control. Confirm the text field takes keyboard focus and no hidden filter remains. Check `--check-ricepack` reports page/compact counts for v1, rejects a future format version with an actionable error and never auto-migrates an incompatible pack.
- [ ] Test malformed `config.toml`: error shown, defaults active; original saved as `config.invalid-*.toml` after editing preferences.

## Process data and actions

- [ ] Compare process CPU/RSS, host RAM, per-core CPU, uptime with `top`/`free`/`uptime`.
- [ ] Compare network and disk rates during a known I/O operation.
- [ ] On an AMDGPU system, compare Taskotto Preferences → GPU sensors (and System Performance) with `/sys/class/drm/card*/device/gpu_busy_percent`, `mem_info_vram_used`, `mem_info_vram_total` and device-scoped hwmon sensors. Test zero/idle load, gaming load, unplugged sensors and driver permission errors. On Intel/NVIDIA, verify missing utilization explicitly says unavailable rather than inventing a value; confirm no connector (e.g. card0-HDMI-A-1) appears as a separate GPU. Record actual monitoring overhead.
- [ ] On a laptop, compare System Performance / Preferences → Battery and external power with `/sys/class/power_supply/BAT*/capacity`, `status`, `energy_now`, `energy_full`, `power_now`, and AC/USB `online`. Test unplugged, charging and full states, optional BAT0+BAT1, missing sysfs fields and desktop without a battery; verify no Wh is inferred from charge in µAh and no runtime is invented.
- [ ] Compare process start/runtime and read/write rates against `/proc/<PID>/stat` and `/proc/<PID>/io` for permitted processes.
- [ ] Select a child with shared libraries, click **Measure PSS now** in generic Details or Preferences and compare proportional tree bytes against `/proc/<PID>/smaps_rollup` (`Pss:` in kB) for each available descendant. Confirm no UI blocking; audit inaccessible children, exiting/reused PIDs, 128-process limit and partial-result wording. Do not interpret PSS as uniquely owned RAM or an atomic snapshot.
- [ ] Launch a parent process with multiple child/grandchild processes. Select its real PID and compare the Details/XP Properties/macOS Info observed-tree process count, CPU sum and **summed** RSS with one snapshot. Verify exiting children disappear on refresh, PID reuse cannot select a stale root, cycles/duplicate entries do not inflate counts, and the UI never claims shared RSS is unique physical memory or PSS.
- [ ] Compare selected process leader Nice and `Cpus_allowed_list` shown in Details/XP/macOS Info with `/proc/<PID>/stat` (field 19, signed) and `/proc/<PID>/status`. Try an affinity-restricted disposable child and a process with several threads; verify the UI explicitly says **leader** and does not imply all threads share the same mask. Missing data must be unavailable, and Taskotto must not offer unverified privileged scheduler mutations.
- [ ] Search by name, command, PID and user; sort CPU/RAM/I/O/PID/name and toggle tree mode.
- [ ] Launch a disposable `sleep 1000` process; pause, resume and terminate it after confirmation.
- [ ] Use the Force kill dialog only on a disposable test process; check warning text.
- [ ] Attempt a signal to an inaccessible process; UI reports permission denied.
- [ ] Confirm PID 1 and Taskotto itself cannot be signaled.
- [ ] Watch processes rapidly start/exit for at least 15 minutes; no crashes or stale selection actions.

## Performance and resource use

Measure after a 60-second warm-up, then with monitoring stopped. Capture 3 runs each:

```bash
ps -C taskotto -o pid,rss,%cpu,etime,cmd
/usr/bin/time -v taskotto --snapshot
```

- [ ] RSS, average idle CPU and startup latency on a low-end dual-core machine.
- [ ] Same on a modern Wayland desktop.
- [ ] Repeat at 500ms, 1500ms and 10000ms refresh periods.
- [ ] Verify no taskotto worker remains after closing the window.
- [ ] Minimize for 10 minutes, restore and verify fresh readings without a growing snapshot backlog.
- [ ] Record results, kernel, display server, and system load in release notes.

## Sign-off

Mark passed steps with dates and actual outputs; never mark checked based only on code review or CI.
