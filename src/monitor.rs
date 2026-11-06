//! Unprivileged, on-demand Linux /proc sampler. No daemon and no shell commands.
use serde::Serialize;
use std::{
    collections::HashMap,
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

#[derive(Default)]
pub struct Monitor {
    previous: Option<Sample>,
}

impl Monitor {
    pub fn sample(&mut self) -> io::Result<Snapshot> {
        let at = Instant::now();
        let cpu = read_cpu()?;
        let memory = read_memory()?;
        let uptime_seconds = read_uptime().unwrap_or_default();
        let io = read_io();
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.max(4096) as u64;
        let mut processes = read_processes(page_size)?;
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

fn read_processes(page_size: u64) -> io::Result<Vec<Process>> {
    let entries = fs::read_dir("/proc")?;
    Ok(entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_string_lossy().parse::<i32>().ok())
        .filter_map(|pid| read_process(pid, page_size).ok())
        .collect())
}

fn read_process(pid: i32, page_size: u64) -> io::Result<Process> {
    let directory = Path::new("/proc").join(pid.to_string());
    let stat = fs::read_to_string(directory.join("stat"))?;
    let mut process = parse_process_stat(pid, &stat, page_size)?;
    let status = fs::read_to_string(directory.join("status")).unwrap_or_default();
    process.user_id = status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|s| s.split_whitespace().next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(u32::MAX);
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
    }
}
