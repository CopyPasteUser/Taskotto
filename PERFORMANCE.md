# Taskotto performance measurements

Numbers below were observed, not estimated. They are **headless CLI samples**, not desktop GUI benchmarks. They must not be compared directly to `btop`, `xfce4-taskmanager` or GNOME/KDE monitors.

## Sampler CPU optimization — 2026-11-08

- Root cause: each periodic Linux process scan re-opened `/proc/PID/status` and `/proc/PID/cmdline` for **every** PID, even when the process identity and metadata were unchanged. The sampler already uses a background thread and egui already wakes on samples; adding another UI redraw timer would increase idle work.
- A per-`Monitor` bounded 5-second metadata cache now amortizes these reads. It requires a matching PID **and** `/proc/PID/stat` start tick **and** process name. Disappearing PIDs are pruned every scan. CPU, RSS, state, threads and I/O still come from live `/proc` counters on **every** tick. UID/affinity/command changes within an otherwise unchanged process can appear with up to 5 seconds' delay; Linux independently enforces permissions for process actions.
- Native GUI windows reported as minimized by egui now sample at **at least 5 seconds** instead of the configured interval, without modifying saved settings. Restoring the window resumes its normal interval. Platforms that do not expose minimized state keep the original cadence.
- Reproducible sampler microbenchmark: `cargo test --locked --test sampler_performance -- --ignored --nocapture` (300 back-to-back process samples, debug build, development container). Before: **0.435 ms CPU/sample**. After: **0.356 ms CPU/sample** (single-run, about 18% lower in this best-case hot-cache microbenchmark). This is **not** a claim of 18% lower overall application CPU, nor a prediction for 1.5-second cadence on Mint. The cache has extra per-process metadata allocations; GUI RAM changes were not measured.
- Mint/ThinkPad GUI validation remains necessary: compare idle CPU (visible/minimized), RSS/PSS, process command/UID refresh after up to 5 seconds, PID replacement, and sustained process activity with the same theme, compositor and sampling rate.

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

## GUI benchmark follow-up after owner-approved 1.0

No graphical session was available in the original development environment. The owner approved stable 1.0 on 2026-11-09 following several days of use, but detailed startup, idle CPU, peak/steady RSS, process-list scaling and extended soak measurements were **not submitted**. Record three reproducible GUI runs per machine at 500, 1500 and 10000 ms sampling intervals, plus kernel, load, compositor and RAM. Use [QA.md](QA.md) for test steps; do not replace that evidence with the headless numbers above.
