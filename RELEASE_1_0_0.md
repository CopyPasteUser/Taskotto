# Taskotto 1.0.0 — first stable release

**Released:** 9 November 2026 · **Platform:** Linux x86_64, X11 or Wayland

Taskotto is an independent Rust process manager and live Linux system monitor
with seven distinct OS-inspired client interfaces, a generic System view and
safe, data-only RicePacks. It works without RiceBowl, a privileged daemon or
an always-on background service.

## Highlights

- Live processes, searchable and sortable tables, process trees, per-process
  CPU/RAM/I/O details and confirmation-backed, PID-identity-safe signals.
- CPU, memory, disk, network and Linux PSI monitoring; available GPU/battery
  sensors and read-only cgroup inspection with explicit missing-value states.
- Windows 98, XP Luna, Windows 7, Windows 10, macOS Activity Monitor,
  ChromeOS and Frutiger Aero client layouts. The active window manager owns
  native title bars and window controls.
- Validated v1 RicePacks with preview, live file reload, declarative screens,
  screen editor, shared-scale comparisons and an optional personal startup
  dashboard. RiceBowl can pass an explicit design or select a compatible
  RiceProfile session using the documented environment contract.
- Optional low-overhead mode, bounded session history and deliberate private
  CSV/JSON exports. No historical process logs are saved automatically.
- User-local application-menu installer with repair and safe uninstall.
  The last pre-1.0 development commits also fixed the Mint menu icon and
  reduced repeated `/proc` metadata reads and minimized-window polling.

## Install

Download `taskotto-v1.0.0-linux-x86_64.tar.gz` and
`taskotto-v1.0.0-SHA256SUMS.txt` from this GitHub release. Verify the checksum
in the directory containing both files:

```bash
sha256sum -c taskotto-v1.0.0-SHA256SUMS.txt
tar -xzf taskotto-v1.0.0-linux-x86_64.tar.gz
cd taskotto
bash install.sh
```

The archive contains an Ubuntu 24.04-built x86_64 executable and the local
installer. Other Linux architectures should build from source with Rust 1.88+:

```bash
git clone https://github.com/CopyPasteUser/Taskotto.git
cd Taskotto
cargo install --path . --locked
taskotto --install-desktop
```

Try `taskotto --theme xp-luna`, `taskotto --snapshot --json` or
`taskotto --check-ricepack examples/ricepacks/comparison-studio.toml`.

## Acceptance and known boundaries

The project owner approved the 1.0 release after several days of ordinary use
on 9 November 2026. Detailed results for the separate Mint/X11, KDE/Wayland
and RiceBowl/labwc test matrix and three-run GUI startup/CPU/RSS benchmarks
were **not supplied**; no numerical GUI-performance or pixel-match guarantee
is asserted. The test matrix remains available in
[`ACCEPTANCE_1_0.md`](ACCEPTANCE_1_0.md) and [`PERFORMANCE.md`](PERFORMANCE.md).

Linux process data is subject to the caller's permissions and kernel APIs.
GPU capabilities vary by driver; in particular full Intel/NVIDIA utilization
is not guaranteed. macOS-style Energy is labeled as a CPU proxy, not measured
watts, and global network throughput must not be read as per-process traffic.
Some historical looks are artistic Linux adaptations rather than exact copies
of proprietary desktop software. Further compositor-specific and advanced
management features remain on the expansion roadmap.

**Integrity:** GitHub Actions builds the tagged commit, verifies the release
archive twice byte-for-byte, checks SHA256, exercises the binary and its
desktop install/uninstall flow, then uploads the archive and checksum.
