# Taskotto performance measurements

Numbers below were observed, not estimated. They are **headless CLI samples**, not desktop GUI benchmarks. They must not be compared directly to `btop`, `xfce4-taskmanager` or GNOME/KDE monitors.

## Development-container baseline — 2026-11-05

- OS: Linux kernel 6.18.57, container without X11/Wayland display server.
- Rust: `rustc 1.99.1`; `cargo build --locked --release` with repository release profile.
- Binary: **10,651,280 bytes** (about 10.16 MiB; file size is not memory usage).
- `taskotto --snapshot` warms two `/proc` samples with a deliberate **250 ms sleep**. Three consecutive warm-cache wall times from Bash `time -p`: **0.26 / 0.25 / 0.25 seconds**. This chiefly measures the deliberate delay, not GUI startup.
- Three sampled headless CLI RSS readings around 50 ms after process start: **2,728 / 2,668 / 2,664 KiB** (`ps -o rss=`). These are point samples, not peak RSS, not the egui window process and not a reliable memory ceiling.
- A prior uncached invocation took 4.80 seconds amid development-container I/O activity; cold-start performance was **not** reproducibly established. Do not infer a general cold-start guarantee from the warm results.

Reproduce with a locally built binary:

```bash
stat -c '%s' target/release/taskotto
for run in 1 2 3; do time -p ./target/release/taskotto --snapshot >/dev/null; done
for run in 1 2 3; do
  ./target/release/taskotto --snapshot >/dev/null & taskotto_pid=$!
  sleep 0.05
  ps -p "$taskotto_pid" -o rss=,%cpu=,etimes=
  wait "$taskotto_pid"
done
```

## Required GUI baseline before stable 1.0

No graphical session was available in the development environment. Startup, idle CPU, peak/steady RSS, process-list scaling and extended soak behavior are **unverified** on real desktop hardware. Record three reproducible GUI runs per machine at 500, 1500 and 10000 ms sampling intervals, plus kernel, load, compositor and RAM. Use [QA.md](QA.md) for test steps; do not replace that acceptance with the headless numbers above.
