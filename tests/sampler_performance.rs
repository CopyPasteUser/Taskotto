//! Reproducible opt-in sampler CPU benchmark. The GUI needs a separate desktop test.

use std::time::Instant;
use taskotto::monitor::Monitor;

fn process_cpu_time() -> f64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let status = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut time) };
    assert_eq!(status, 0, "could not read process CPU clock");
    time.tv_sec as f64 + time.tv_nsec as f64 * 1e-9
}

#[test]
#[ignore = "opt-in performance measurement: cargo test --test sampler_performance -- --ignored --nocapture"]
fn benchmark_proc_sampler_cpu_time() {
    const SAMPLES: usize = 300;
    let mut monitor = Monitor::default();
    monitor.sample().expect("initial /proc sample");
    let start_cpu = process_cpu_time();
    let start_wall = Instant::now();
    for _ in 0..SAMPLES {
        monitor.sample().expect("repeat /proc sample");
    }
    let elapsed_cpu = (process_cpu_time() - start_cpu) * 1000.0;
    let elapsed_wall = start_wall.elapsed().as_secs_f64() * 1000.0;
    println!(
        "sampler: {SAMPLES} iterations, {:.3} ms CPU/sample, {:.3} ms wall/sample",
        elapsed_cpu / SAMPLES as f64,
        elapsed_wall / SAMPLES as f64
    );
}
