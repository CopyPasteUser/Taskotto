# Taskotto 1.0 — owner hardware sign-off

The implementation and automated CI are **not** equivalent to a stable 1.0
approval. Use this short pass/fail sheet on actual Linux desktops. For detailed
edge cases and comparisons see [QA.md](QA.md). Record failures as reproducible
issues; do not sign off until P0/P1 defects are fixed and retested.

## Prepare each machine

Record distribution, kernel (`uname -r`), desktop/compositor, display server
(`echo "$XDG_SESSION_TYPE"`), CPU, GPU, available RAM and tested Taskotto
commit (`taskotto --version`, `git rev-parse HEAD`). Do not run destructive
process actions on important applications: use a disposable child only.

From a fresh checkout of the tested release candidate:

```bash
cargo test --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo build --locked --release
./target/release/taskotto --snapshot --json
./target/release/taskotto --check-ricepack examples/ricepacks/comparison-studio.toml
```

## Functional pass/fail checklist

- [ ] **Mint/X11:** launch via native window and app menu, minimize, restore,
  resize, use clipboard and keyboard shortcuts, close and reopen.
- [ ] **Fedora KDE/Wayland:** repeat launch, focus, resize, clipboard and
  keyboard navigation; record any visual/rendering differences.
- [ ] **RiceBowl/labwc/Wayland:** launch without extra XFCE/KDE processes and
  confirm the compositor draws exactly one native title/window-control frame.
- [ ] **All eight profiles:** select each rice, inspect a real process and
  performance view, change density/fonts/navigation and restart. All controls
  remain reachable at 490×340 where supported, 650×420 and 1280×800.
- [ ] **Visual fidelity:** capture native-resolution XP Luna, Windows 7,
  Windows 10, Windows 98, macOS, ChromeOS and Aero screenshots; compare menus,
  tabs, metrics, fonts, density, interaction and colors to legal references.
  Do not call designs "95%" or "1:1" without an assessed comparison.
- [ ] **Process safety:** compare live CPU/RSS/I/O and process hierarchy to
  Linux tools; confirm/abort SIGSTOP, SIGCONT, SIGTERM, SIGKILL on disposable
  children. Verify permission denial, self/PID 1 blocks and PID reuse checks.
- [ ] **Telemetry:** verify system/core CPU, RAM, network, disk, PSI and
  available GPU/battery/cgroup readings against host `/proc` and `/sys`;
  unavailable sources must say unavailable, never invent values.
- [ ] **RicePack:** open Diagnostic and Comparison Studio; inspect real units,
  legends, compact fallbacks and functional actions. Export/import, preview,
  revert and pin a personal default. Rename its file and ensure launch falls
  back safely; CLI/RiceBowl explicit packs still take precedence.
- [ ] **Persistence/export:** check only intended XDG settings persist; verify
  owner-only CSV/JSON exports, timestamp/sample bounds and no overwrite;
  intentionally malformed settings must recover without losing the original.
- [ ] **Installer:** install/reinstall/uninstall the user-local desktop entry,
  confirm app-menu launch and safe handling of edited launcher/icon files.
- [ ] **Soak:** monitor 15 minutes of short-lived processes and 10 minutes
  minimized. Watch for crashes, UI stalls, stale selections, growing memory,
  leftover worker threads/processes or GPU rendering errors.
- [ ] **Performance:** after 60 seconds warm-up measure *three runs per
  sampling interval* at 500, 1500 and 10000 ms on a slower X11 machine and a
  Wayland machine. Record RSS, idle CPU, GUI startup and compare with another
  local system monitor. See the commands in [QA.md](QA.md).

## Evidence log

Fill a row for every desktop environment. Link or attach representative
native screenshots and logs to the release issue/PR. Replace `TBD` rather
than checking off a test you did not run.

| Environment | Version/commit | Functional result | Median RSS / idle CPU / startup | Issues/screenshots |
| --- | --- | --- | --- | --- |
| Mint / X11 | TBD | TBD | TBD | TBD |
| Fedora KDE / Wayland | TBD | TBD | TBD | TBD |
| RiceBowl / labwc | TBD | TBD | TBD | TBD |

**Owner sign-off:** name/date TBD. **Release engineer:** verify sign-off,
address all release-blocking defects, update actual measurements and limitations
in release notes, then bump to `1.0.0` and publish the stable tag. Until that
sequence is complete, the repository remains a release candidate.
