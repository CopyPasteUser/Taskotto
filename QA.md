# Taskotto QA — real hardware acceptance

Run this checklist before a stable 1.0 tag. Record distro, kernel, display server, compositor, CPU, and actual measurements. A green CI run is not a substitute for this list.

## Desktop behavior

- [ ] XP Luna: verify blue chrome, raised tabs, File/Options/View/Help menus, classic CPU/RAM history, process sorting and status strip; use keyboard focus and resize to minimum.
- [ ] Windows 10: verify light tabs, process grouping, CPU/memory heatmap, sorting, context menus, and switching CPU/Memory/Disk/Network in the Performance sidebar.
- [ ] Both rices: right-click a disposable process, open Details, Pause/Resume and End task after confirmation; verify no unintended actions or stale PID targets.
- [ ] Capture and review a native Taskotto window screenshot on X11, KDE Wayland and labwc. The [headless README images](SCREENSHOTS.md) do not certify display-server rendering.
- [ ] Linux Mint / X11: launch via `.desktop` menu, resize, minimize, close, reopen.
- [ ] Fedora KDE / Wayland: repeat and test clipboard/focus.
- [ ] labwc / Wayland: launch without XFCE/KDE services; confirm no missing runtime dependencies.
- [ ] At 650×420 and at a large desktop resolution, all controls remain reachable.
- [ ] Tab through controls and use Ctrl+F, F5, Escape.
- [ ] Switch all 8 profiles, density, custom accent/surface/text, font scale, spacing, corners, table columns/row height and sidebar/top-tabs; reopen and verify persistence.
- [ ] Launch with `RICEBOWL_RICE=windows7 taskotto` and explicit `--theme xp-luna` override.
- [ ] Confirm the launcher override does not replace the saved theme after changing another setting.
- [ ] Test malformed `config.toml`: error shown, defaults active; original saved as `config.invalid-*.toml` after editing preferences.

## Process data and actions

- [ ] Compare process CPU/RSS, host RAM, per-core CPU, uptime with `top`/`free`/`uptime`.
- [ ] Compare network and disk rates during a known I/O operation.
- [ ] Compare process start/runtime and read/write rates against `/proc/<PID>/stat` and `/proc/<PID>/io` for permitted processes.
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
