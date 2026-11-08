//! Unprivileged, on-demand Linux /proc sampler. No daemon and no shell commands.
use serde::Serialize;
pub(crate) mod cgroup;
mod gpu;
mod power;
mod psi;
pub(crate) mod pss;
pub use gpu::Gpu;
pub use power::PowerSnapshot;
pub use psi::{PressureMetric, PressureSnapshot};
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default, Serialize)]
pub struct Process {
    pub pid: i32,
    pub parent_pid: i32,
    pub user_id: u32,
    pub name: String,
    pub command: String,
    pub state: char,
    /// /proc/PID/stat field 19: scheduler nice value for the group leader.
    pub nice: i32,
    /// /proc/PID/status leader affinity; other threads may differ.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpus_allowed_list: Option<String>,
    pub threads: u32,
    pub resident_bytes: u64,
    pub virtual_bytes: u64,
    pub cpu_percent: f32,
    pub read_bytes: u64,
    pub written_bytes: u64,
    pub io_accessible: bool,
    pub read_per_second: u64,
    pub written_per_second: u64,
    pub start_ticks: u64,
    pub start_unix_seconds: u64,
    pub elapsed_seconds: f64,
    #[serde(skip)]
    pub(crate) total_ticks: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Memory {
    pub total: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_free: u64,
}

impl Memory {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }
    pub fn used_fraction(&self) -> f32 {
        fraction(self.used(), self.total)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Rates {
    pub received_per_second: u64,
    pub sent_per_second: u64,
    pub disk_read_per_second: u64,
    pub disk_write_per_second: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub cpu_percent: f32,
    pub cores_percent: Vec<f32>,
    pub memory: Memory,
    pub rates: Rates,
    pub uptime_seconds: f64,
    pub processes: Vec<Process>,
    pub logical_cpus: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gpus: Vec<Gpu>,
    #[serde(skip_serializing_if = "PowerSnapshot::is_empty")]
    pub power: PowerSnapshot,
    #[serde(skip_serializing_if = "PressureSnapshot::is_empty")]
    pub pressure: PressureSnapshot,
}

/// Process-inclusive descendant totals from one observed /proc sample.
/// Each PID contributes once even if malformed/cyclic ancestry is presented.
/// RSS is **summed**, not unique physical memory: shared pages can overlap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeSummary {
    pub process_count: usize,
    pub cpu_percent: f64,
    pub summed_rss_bytes: u64,
}

impl Snapshot {
    pub fn tree_summary(&self, root_pid: i32, root_start_ticks: u64) -> Option<TreeSummary> {
        let processes = self
            .processes
            .iter()
            .map(|process| (process.pid, process))
            .collect::<HashMap<_, _>>();
        if processes.get(&root_pid)?.start_ticks != root_start_ticks {
            return None;
        }
        let mut children: HashMap<i32, Vec<i32>> = HashMap::new();
        for process in &self.processes {
            if process.pid != process.parent_pid {
                children
                    .entry(process.parent_pid)
                    .or_default()
                    .push(process.pid);
            }
        }
        let mut stack = vec![root_pid];
        let mut visited = HashSet::new();
        let mut cpu_percent = 0.0_f64;
        let mut summed_rss_bytes = 0_u64;
        while let Some(pid) = stack.pop() {
            if !visited.insert(pid) {
                continue;
            }
            let Some(process) = processes.get(&pid) else {
                continue;
            };
            cpu_percent += f64::from(process.cpu_percent.max(0.0));
            summed_rss_bytes = summed_rss_bytes.saturating_add(process.resident_bytes);
            if let Some(offspring) = children.get(&pid) {
                stack.extend(offspring.iter().copied());
            }
        }
        Some(TreeSummary {
            process_count: visited.len(),
            cpu_percent,
            summed_rss_bytes,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct CpuTicks {
    total: u64,
    idle: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct IoTotals {
    rx: u64,
    tx: u64,
    read: u64,
    write: u64,
}

struct Sample {
    at: Instant,
    cpu: Vec<CpuTicks>,
    io: IoTotals,
    processes: HashMap<i32, ProcessCounters>,
}

#[derive(Clone, Copy)]
struct ProcessCounters {
    start: u64,
    cpu: u64,
    read: u64,
    written: u64,
}

/// Process identity and relatively stable metadata. `/proc/PID/stat` and
/// `/proc/PID/io` remain live on every sample; only status and command-line
/// reads are amortized across sampler ticks.
struct ProcessMetadata {
    start_ticks: u64,
    name: String,
    user_id: u32,
    cpus_allowed_list: Option<String>,
    command: String,
    refreshed_at: Instant,
    seen_at: Instant,
}

impl ProcessMetadata {
    const MAX_AGE: Duration = Duration::from_secs(5);

    fn valid_for(&self, process: &Process, now: Instant) -> bool {
        self.start_ticks == process.start_ticks
            && self.name == process.name
            && now.saturating_duration_since(self.refreshed_at) < Self::MAX_AGE
    }
}

#[derive(Default)]
pub struct Monitor {
    previous: Option<Sample>,
    metadata: HashMap<i32, ProcessMetadata>,
}

impl Monitor {
    pub fn sample(&mut self) -> io::Result<Snapshot> {
        let at = Instant::now();
        let cpu = read_cpu()?;
        let memory = read_memory()?;
        let uptime_seconds = read_uptime().unwrap_or_default();
        let io = read_io();
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.max(4096) as u64;
        let mut processes = read_processes(page_size, at, &mut self.metadata)?;
        let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as u64;
        let unix_now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for process in &mut processes {
            (process.start_unix_seconds, process.elapsed_seconds) = process_times(
                process.start_ticks,
                ticks_per_second,
                uptime_seconds,
                unix_now,
            );
        }
        let logical_cpus = cpu.len().saturating_sub(1).max(1);
        let previous = self.previous.as_ref();
        let elapsed = previous
            .map_or(Duration::ZERO, |old| at.saturating_duration_since(old.at))
            .as_secs_f64();

        let cpu_percent = previous
            .and_then(|old| old.cpu.first().zip(cpu.first()))
            .map_or(0.0, |(old, new)| cpu_usage(*old, *new));
        let cores_percent = cpu
            .iter()
            .skip(1)
            .enumerate()
            .map(|(index, current)| {
                previous
                    .and_then(|old| old.cpu.get(index + 1))
                    .map_or(0.0, |old| cpu_usage(*old, *current))
            })
            .collect();

        let total_delta = previous
            .and_then(|old| old.cpu.first().zip(cpu.first()))
            .map_or(0, |(old, now)| now.total.saturating_sub(old.total));
        for process in &mut processes {
            if let Some(old) = previous
                .and_then(|old| old.processes.get(&process.pid))
                .copied()
                && old.start == process.start_ticks
            {
                if total_delta > 0 {
                    let delta = process.total_ticks.saturating_sub(old.cpu);
                    process.cpu_percent =
                        ((delta as f64 / total_delta as f64) * 100.0 * logical_cpus as f64)
                            .clamp(0.0, logical_cpus as f64 * 100.0) as f32;
                }
                if process.io_accessible {
                    (process.read_per_second, process.written_per_second) =
                        process_rates(process, old, elapsed);
                }
            }
        }

        let rates = if let Some(old) = previous.filter(|_| elapsed > 0.0) {
            Rates {
                received_per_second: per_second(io.rx, old.io.rx, elapsed),
                sent_per_second: per_second(io.tx, old.io.tx, elapsed),
                disk_read_per_second: per_second(io.read, old.io.read, elapsed),
                disk_write_per_second: per_second(io.write, old.io.write, elapsed),
            }
        } else {
            Rates::default()
        };
        self.previous = Some(Sample {
            at,
            cpu,
            io,
            processes: processes
                .iter()
                .map(|p| {
                    (
                        p.pid,
                        ProcessCounters {
                            start: p.start_ticks,
                            cpu: p.total_ticks,
                            read: p.read_bytes,
                            written: p.written_bytes,
                        },
                    )
                })
                .collect(),
        });
        Ok(Snapshot {
            cpu_percent,
            cores_percent,
            memory,
            rates,
            uptime_seconds,
            processes,
            logical_cpus,
            gpus: gpu::sample(),
            power: power::sample(),
            pressure: psi::sample(),
        })
    }
}

fn fraction(part: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64).clamp(0.0, 1.0) as f32
    }
}

fn process_times(start_ticks: u64, hz: u64, uptime_seconds: f64, unix_now: u64) -> (u64, f64) {
    if hz == 0 || uptime_seconds <= 0.0 {
        return (0, 0.0);
    }
    let elapsed = (uptime_seconds - start_ticks as f64 / hz as f64).max(0.0);
    (unix_now.saturating_sub(elapsed as u64), elapsed)
}

fn process_rates(now: &Process, old: ProcessCounters, elapsed: f64) -> (u64, u64) {
    if old.start != now.start_ticks || elapsed <= 0.0 {
        return (0, 0);
    }
    (
        per_second(now.read_bytes, old.read, elapsed),
        per_second(now.written_bytes, old.written, elapsed),
    )
}

fn per_second(now: u64, previous: u64, elapsed: f64) -> u64 {
    (now.saturating_sub(previous) as f64 / elapsed).min(u64::MAX as f64) as u64
}

fn cpu_usage(before: CpuTicks, after: CpuTicks) -> f32 {
    let total = after.total.saturating_sub(before.total);
    if total == 0 {
        return 0.0;
    }
    let idle = after.idle.saturating_sub(before.idle);
    (100.0 * (1.0 - fraction(idle.min(total), total))).clamp(0.0, 100.0)
}

fn read_cpu() -> io::Result<Vec<CpuTicks>> {
    let text = fs::read_to_string("/proc/stat")?;
    let mut cpus = Vec::new();
    for line in text.lines().take_while(|line| line.starts_with("cpu")) {
        let mut numbers = line.split_whitespace().skip(1).map(str::parse::<u64>);
        let values = (0..10)
            .map(|_| numbers.next().and_then(Result::ok).unwrap_or_default())
            .collect::<Vec<_>>();
        // Guest ticks are already included in user/nice and must not be counted twice.
        let total = values[..8].iter().sum();
        let idle = values[3].saturating_add(values[4]);
        cpus.push(CpuTicks { total, idle });
    }
    if cpus.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "no CPU counters",
        ));
    }
    Ok(cpus)
}

fn read_memory() -> io::Result<Memory> {
    let text = fs::read_to_string("/proc/meminfo")?;
    let mut values = HashMap::new();
    for line in text.lines() {
        if let Some((key, rest)) = line.split_once(':')
            && let Some(kib) = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<u64>().ok())
        {
            values.insert(key, kib.saturating_mul(1024));
        }
    }
    let get = |key| values.get(key).copied().unwrap_or_default();
    Ok(Memory {
        total: get("MemTotal"),
        available: get("MemAvailable"),
        swap_total: get("SwapTotal"),
        swap_free: get("SwapFree"),
    })
}

fn read_uptime() -> io::Result<f64> {
    fs::read_to_string("/proc/uptime")?
        .split_whitespace()
        .next()
        .and_then(|text| text.parse::<f64>().ok())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid uptime"))
}

fn read_io() -> IoTotals {
    let mut totals = IoTotals::default();
    if let Ok(text) = fs::read_to_string("/proc/net/dev") {
        for line in text.lines().skip(2) {
            let Some((interface, values)) = line.split_once(':') else {
                continue;
            };
            if interface.trim() == "lo" {
                continue;
            }
            let values = values
                .split_whitespace()
                .filter_map(|s| s.parse::<u64>().ok())
                .collect::<Vec<_>>();
            if values.len() >= 16 {
                totals.rx = totals.rx.saturating_add(values[0]);
                totals.tx = totals.tx.saturating_add(values[8]);
            }
        }
    }
    if let Ok(text) = fs::read_to_string("/proc/diskstats") {
        for line in text.lines() {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            if fields.len() < 10 || !Path::new("/sys/block").join(fields[2]).exists() {
                continue;
            }
            // Linux diskstats sectors are 512-byte units, independent of physical block size.
            let sectors_read = fields[5].parse::<u64>().unwrap_or_default();
            let sectors_written = fields[9].parse::<u64>().unwrap_or_default();
            totals.read = totals.read.saturating_add(sectors_read.saturating_mul(512));
            totals.write = totals
                .write
                .saturating_add(sectors_written.saturating_mul(512));
        }
    }
    totals
}

fn read_processes(
    page_size: u64,
    at: Instant,
    metadata: &mut HashMap<i32, ProcessMetadata>,
) -> io::Result<Vec<Process>> {
    let entries = fs::read_dir("/proc")?;
    let processes = entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_string_lossy().parse::<i32>().ok())
        .filter_map(|pid| read_process(pid, page_size, at, metadata).ok())
        .collect();
    // A PID that disappeared must not keep its metadata indefinitely. A
    // replacement process with the same PID is also checked by start_ticks.
    metadata.retain(|_, cached| cached.seen_at == at);
    Ok(processes)
}

fn read_process(
    pid: i32,
    page_size: u64,
    at: Instant,
    metadata: &mut HashMap<i32, ProcessMetadata>,
) -> io::Result<Process> {
    let directory = Path::new("/proc").join(pid.to_string());
    let stat = fs::read_to_string(directory.join("stat"))?;
    let mut process = parse_process_stat(pid, &stat, page_size)?;
    if let Some(cached) = metadata.get_mut(&pid)
        && cached.valid_for(&process, at)
    {
        process.user_id = cached.user_id;
        process.cpus_allowed_list = cached.cpus_allowed_list.clone();
        process.command.clone_from(&cached.command);
        cached.seen_at = at;
    } else {
        let status = fs::read_to_string(directory.join("status")).unwrap_or_default();
        process.user_id = status
            .lines()
            .find_map(|line| line.strip_prefix("Uid:"))
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(u32::MAX);
        process.cpus_allowed_list = allowed_cpus(&status);
        if let Ok(command) = fs::read(directory.join("cmdline")) {
            let display = command
                .split(|byte| *byte == 0)
                .filter(|part| !part.is_empty())
                .map(String::from_utf8_lossy)
                .collect::<Vec<_>>()
                .join(" ");
            if !display.is_empty() {
                process.command = display;
            }
        }
        metadata.insert(
            pid,
            ProcessMetadata {
                start_ticks: process.start_ticks,
                name: process.name.clone(),
                user_id: process.user_id,
                cpus_allowed_list: process.cpus_allowed_list.clone(),
                command: process.command.clone(),
                refreshed_at: at,
                seen_at: at,
            },
        );
    }
    if let Ok(io) = fs::read_to_string(directory.join("io")) {
        process.io_accessible = true;
        for line in io.lines() {
            if let Some(raw) = line.strip_prefix("read_bytes:") {
                process.read_bytes = raw.trim().parse().unwrap_or_default();
            }
            if let Some(raw) = line.strip_prefix("write_bytes:") {
                process.written_bytes = raw.trim().parse().unwrap_or_default();
            }
        }
    }
    Ok(process)
}

fn allowed_cpus(status: &str) -> Option<String> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Cpus_allowed_list:"))
        .map(str::trim)
        .filter(|mask| !mask.is_empty() && mask.len() <= 128)
        .map(str::to_owned)
}

pub fn start_ticks(pid: i32) -> io::Result<u64> {
    let contents = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    Ok(parse_process_stat(pid, &contents, 4096)?.start_ticks)
}

fn parse_process_stat(pid: i32, stat: &str, page_size: u64) -> io::Result<Process> {
    let left = stat.find('(').ok_or_else(invalid_stat)?;
    let right = stat.rfind(')').ok_or_else(invalid_stat)?;
    if right <= left {
        return Err(invalid_stat());
    }
    let name = stat[left + 1..right].to_owned();
    let fields = stat[right + 1..].split_whitespace().collect::<Vec<_>>();
    let field = |index: usize| -> io::Result<u64> {
        fields
            .get(index)
            .and_then(|s| s.parse().ok())
            .ok_or_else(invalid_stat)
    };
    let signed_field = |index: usize| -> io::Result<i32> {
        fields
            .get(index)
            .and_then(|s| s.parse().ok())
            .ok_or_else(invalid_stat)
    };
    if fields.len() < 22 {
        return Err(invalid_stat());
    }
    Ok(Process {
        pid,
        parent_pid: signed_field(1)?,
        user_id: u32::MAX,
        command: name.clone(),
        name,
        state: fields[0].chars().next().unwrap_or('?'),
        // Nice is signed and may be -20..19. This is the leader's value,
        // not a promise that all threads share a scheduling priority.
        nice: signed_field(16)?,
        cpus_allowed_list: None,
        // /proc/PID/stat field 20 (num_threads): offset 17 from field 3.
        threads: field(17)?.min(u32::MAX as u64) as u32,
        resident_bytes: field(21)?.saturating_mul(page_size),
        virtual_bytes: field(20)?,
        cpu_percent: 0.0,
        read_bytes: 0,
        written_bytes: 0,
        io_accessible: false,
        read_per_second: 0,
        written_per_second: 0,
        start_ticks: field(19)?,
        start_unix_seconds: 0,
        elapsed_seconds: 0.0,
        total_ticks: field(11)?.saturating_add(field(12)?),
    })
}

fn invalid_stat() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid /proc/PID/stat")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_process_names_containing_parentheses() {
        // Fields 3–24 after the parenthesized name, including all required indices.
        let stat = "412 (a weird ) ( name) S 1 0 0 0 0 0 0 0 0 0 42 8 0 0 20 0 3 0 1000 8192 12";
        let p = parse_process_stat(412, stat, 4096).unwrap();
        assert_eq!(p.name, "a weird ) ( name");
        assert_eq!(p.parent_pid, 1);
        assert_eq!(p.total_ticks, 50);
        assert_eq!(p.start_ticks, 1000);
        assert_eq!(p.resident_bytes, 49_152);
        assert_eq!(p.threads, 3);
        assert_eq!(p.nice, 0);
        let negative = stat.replace(" 20 0 3 0 1000", " 20 -5 3 0 1000");
        assert_eq!(parse_process_stat(412, &negative, 4096).unwrap().nice, -5);
    }

    #[test]
    fn cpu_calculations_ignore_counter_reset() {
        let old = CpuTicks {
            total: 100,
            idle: 20,
        };
        let new = CpuTicks {
            total: 200,
            idle: 50,
        };
        assert!((cpu_usage(old, new) - 70.0).abs() < 0.001);
        assert_eq!(cpu_usage(old, old), 0.0);
        assert_eq!(cpu_usage(new, old), 0.0);
        assert_eq!(per_second(12, 20, 0.5), 0);
        assert_eq!(fraction(1, 0), 0.0);
    }

    #[test]
    fn calculates_process_start_and_runtime() {
        assert_eq!(process_times(30_000, 100, 400.0, 1_000), (900, 100.0));
        assert_eq!(process_times(50_000, 100, 400.0, 1_000), (1_000, 0.0));
        assert_eq!(process_times(100, 0, 400.0, 1_000), (0, 0.0));
    }

    #[test]
    fn per_process_rates_reject_pid_reuse_and_counter_reset() {
        let old = ProcessCounters {
            start: 10,
            cpu: 100,
            read: 10_000,
            written: 7_000,
        };
        let process = Process {
            start_ticks: 10,
            read_bytes: 12_000,
            written_bytes: 6_000,
            ..Process::default()
        };
        assert_eq!(process_rates(&process, old, 2.0), (1_000, 0));
        assert_eq!(
            process_rates(
                &Process {
                    start_ticks: 20,
                    ..process
                },
                old,
                2.0
            ),
            (0, 0)
        );
    }

    #[test]
    fn actual_proc_snapshot_is_readable() {
        let snapshot = Monitor::default().sample().unwrap();
        assert!(snapshot.logical_cpus > 0);
        assert!(snapshot.memory.total > 0);
        assert!(
            snapshot
                .processes
                .iter()
                .any(|p| p.pid == std::process::id() as i32)
        );
        let own = snapshot
            .processes
            .iter()
            .find(|p| p.pid == std::process::id() as i32)
            .unwrap();
        assert!((-20..=19).contains(&own.nice));
        assert!(own.cpus_allowed_list.is_some());
    }

    #[test]
    fn metadata_cache_checks_identity_name_and_freshness() {
        let now = Instant::now();
        let cached = ProcessMetadata {
            start_ticks: 77,
            name: "original".into(),
            user_id: 1000,
            cpus_allowed_list: Some("0-3".into()),
            command: "original --flag".into(),
            refreshed_at: now,
            seen_at: now,
        };
        let mut process = Process {
            pid: 123,
            start_ticks: 77,
            name: "original".into(),
            ..Process::default()
        };
        assert!(cached.valid_for(&process, now + Duration::from_secs(4)));
        assert!(!cached.valid_for(&process, now + ProcessMetadata::MAX_AGE));
        process.start_ticks += 1; // PID reuse must not reuse metadata.
        assert!(!cached.valid_for(&process, now));
        process.start_ticks = 77;
        process.name = "exec-replacement".into(); // Same PID, exec'd program.
        assert!(!cached.valid_for(&process, now));
    }

    #[test]
    fn live_proc_metadata_is_reused_and_refreshed() {
        let mut monitor = Monitor::default();
        let pid = std::process::id() as i32;
        let initial = monitor.sample().unwrap();
        let process = initial.processes.iter().find(|p| p.pid == pid).unwrap();
        let cached_at = monitor.metadata.get(&pid).unwrap().refreshed_at;
        let updated = monitor.sample().unwrap();
        let next = updated.processes.iter().find(|p| p.pid == pid).unwrap();
        assert_eq!(process.user_id, next.user_id);
        assert_eq!(process.command, next.command);
        assert_eq!(monitor.metadata.get(&pid).unwrap().refreshed_at, cached_at);

        // Expiration triggers a fresh read, while the PID identity stays valid.
        monitor.metadata.get_mut(&pid).unwrap().refreshed_at = cached_at - ProcessMetadata::MAX_AGE;
        monitor.sample().unwrap();
        assert!(monitor.metadata.get(&pid).unwrap().refreshed_at > cached_at);

        // A cached PID with another lifetime must be invalidated immediately.
        monitor.metadata.get_mut(&pid).unwrap().start_ticks = 0;
        monitor.sample().unwrap();
        assert_eq!(
            monitor.metadata.get(&pid).unwrap().start_ticks,
            process.start_ticks
        );
    }

    #[test]
    fn proc_status_affinity_is_available_only_when_explicitly_reported() {
        assert_eq!(
            allowed_cpus("Uid:\t1000\nCpus_allowed_list:\t0-3,5\n"),
            Some("0-3,5".into())
        );
        assert_eq!(allowed_cpus("Uid:\t1000\n"), None);
        assert_eq!(allowed_cpus("Cpus_allowed_list:\t\n"), None);
        assert_eq!(
            allowed_cpus(&format!("Cpus_allowed_list:\t{}", "1".repeat(129))),
            None
        );
    }

    #[test]
    fn descendant_totals_count_each_snapshot_pid_once_and_reject_stale_roots() {
        let proc = |pid, parent_pid, start_ticks, cpu_percent, resident_bytes| Process {
            pid,
            parent_pid,
            start_ticks,
            cpu_percent,
            resident_bytes,
            ..Process::default()
        };
        let snapshot = Snapshot {
            cpu_percent: 20.0,
            cores_percent: vec![],
            memory: Memory::default(),
            rates: Rates::default(),
            uptime_seconds: 1.0,
            processes: vec![
                proc(10, 2, 7, 12.0, 100),
                proc(11, 10, 8, 2.0, 200),
                proc(12, 10, 9, 3.0, u64::MAX),
                proc(13, 11, 10, 1.0, 300),
                proc(14, 99, 11, 80.0, 500),
                proc(11, 10, 8, 2.0, 200),
            ],
            logical_cpus: 2,
            gpus: vec![],
            power: Default::default(),
            pressure: Default::default(),
        };
        let summary = snapshot.tree_summary(10, 7).unwrap();
        assert_eq!(summary.process_count, 4);
        assert_eq!(summary.cpu_percent, 18.0);
        assert_eq!(summary.summed_rss_bytes, u64::MAX); // saturating
        assert_eq!(snapshot.tree_summary(10, 6), None); // PID reuse
        assert_eq!(snapshot.tree_summary(99, 1), None); // missing root
        assert_eq!(snapshot.tree_summary(14, 11).unwrap().process_count, 1);
        let mut cycle = snapshot;
        cycle.processes[0].parent_pid = 13;
        assert_eq!(cycle.tree_summary(10, 7).unwrap().process_count, 4);
    }
}
