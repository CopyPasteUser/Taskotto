//! Explicit, bounded proportional-memory inspection. Never part of the
//! regular monitor sampler: smaps_rollup walks process memory maps and can
//! be expensive or denied by procfs permissions.
use super::{Process, Snapshot, start_ticks};
use std::{
    collections::{HashMap, HashSet},
    fs, io,
};

const MAX_PROCESSES: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TreePss {
    pub observed: usize,
    pub readable: usize,
    pub proportional_bytes: u64,
}

impl TreePss {
    pub fn complete(&self) -> bool {
        self.observed == self.readable
    }
}

pub(crate) fn inspect_tree(snapshot: &Snapshot, root: (i32, u64)) -> Result<TreePss, String> {
    let by_pid: HashMap<i32, &Process> = snapshot.processes.iter().map(|p| (p.pid, p)).collect();
    if by_pid.get(&root.0).is_none_or(|p| p.start_ticks != root.1) {
        return Err("Selected process identity is absent from the sampled tree".into());
    }
    let mut children: HashMap<i32, Vec<i32>> = HashMap::new();
    for p in &snapshot.processes {
        if p.pid != p.parent_pid {
            children.entry(p.parent_pid).or_default().push(p.pid);
        }
    }
    let mut stack = vec![root.0];
    let mut visited = HashSet::new();
    let mut ids = Vec::new();
    while let Some(pid) = stack.pop() {
        if !visited.insert(pid) {
            continue;
        }
        if let Some(process) = by_pid.get(&pid) {
            ids.push((process.pid, process.start_ticks));
            if ids.len() > MAX_PROCESSES {
                return Err(format!(
                    "Tree is larger than the {MAX_PROCESSES}-process inspection limit"
                ));
            }
            if let Some(next) = children.get(&pid) {
                stack.extend(next.iter().copied());
            }
        }
    }
    let mut result = TreePss {
        observed: ids.len(),
        readable: 0,
        proportional_bytes: 0,
    };
    for (pid, birth) in ids {
        // Procfs may deny smaps_rollup, a child may exit, or its PID may be
        // reused while the background worker is reading. Never turn that into
        // a made-up zero: mark the report incomplete.
        if let Ok(bytes) = read_verified_process_pss(pid, birth) {
            result.readable += 1;
            result.proportional_bytes = result.proportional_bytes.saturating_add(bytes);
        }
    }
    Ok(result)
}

fn read_verified_process_pss(pid: i32, original_start_ticks: u64) -> io::Result<u64> {
    if start_ticks(pid)? != original_start_ticks {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "PID identity changed",
        ));
    }
    let text = fs::read_to_string(format!("/proc/{pid}/smaps_rollup"))?;
    let pss = parse_rollup_pss(&text).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "No valid Pss entry in smaps_rollup",
        )
    })?;
    if start_ticks(pid)? != original_start_ticks {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "PID identity changed",
        ));
    }
    Ok(pss)
}

fn parse_rollup_pss(text: &str) -> Option<u64> {
    let value = text.lines().find_map(|line| line.strip_prefix("Pss:"))?;
    let mut parts = value.split_whitespace();
    let number = parts.next()?.parse::<u64>().ok()?;
    if parts.next()? != "kB" || parts.next().is_some() {
        return None;
    }
    number.checked_mul(1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pss_only_from_exact_rollup_field_and_units() {
        assert_eq!(
            parse_rollup_pss("Rss: 9 kB\nPss: 17 kB\nPss_Anon: 5 kB\n"),
            Some(17 * 1024)
        );
        assert_eq!(parse_rollup_pss("Pss_Anon: 8 kB"), None);
        assert_eq!(parse_rollup_pss("Pss: 12 KiB"), None);
        assert_eq!(parse_rollup_pss("Pss: 18446744073709551615 kB"), None);
    }

    #[test]
    fn process_rollup_uses_real_start_tick_and_permissions() {
        let pid = std::process::id() as i32;
        let start = start_ticks(pid).expect("current proc stat");
        // Not every kernel/user namespace allows reading smaps_rollup. A
        // permission-denied result is legitimate, not a test failure.
        if let Ok(value) = read_verified_process_pss(pid, start) {
            assert!(value > 0);
        }
        assert!(read_verified_process_pss(pid, start.saturating_add(1)).is_err());
    }

    #[test]
    fn tree_inspection_checks_identity_and_caps_procfs_work() {
        let root = Process {
            pid: 400,
            parent_pid: 1,
            start_ticks: 321,
            ..Process::default()
        };
        let mut processes = vec![root];
        for child in 401..=529 {
            processes.push(Process {
                pid: child,
                parent_pid: 400,
                start_ticks: 200,
                ..Process::default()
            });
        }
        let snapshot = Snapshot {
            processes,
            cpu_percent: 0.0,
            cores_percent: vec![],
            memory: Default::default(),
            rates: Default::default(),
            uptime_seconds: 0.0,
            logical_cpus: 1,
            gpus: vec![],
            power: Default::default(),
        };
        assert!(
            inspect_tree(&snapshot, (400, 322))
                .unwrap_err()
                .contains("identity")
        );
        assert!(
            inspect_tree(&snapshot, (400, 321))
                .unwrap_err()
                .contains("128")
        );
    }

    #[test]
    fn single_current_process_is_reported_without_faking_unavailable_pss() {
        let pid = std::process::id() as i32;
        let start = start_ticks(pid).unwrap();
        let snapshot = Snapshot {
            processes: vec![Process {
                pid,
                start_ticks: start,
                ..Process::default()
            }],
            cpu_percent: 0.0,
            cores_percent: vec![],
            memory: Default::default(),
            rates: Default::default(),
            uptime_seconds: 0.0,
            logical_cpus: 1,
            gpus: vec![],
            power: Default::default(),
        };
        let result = inspect_tree(&snapshot, (pid, start)).unwrap();
        assert_eq!(result.observed, 1);
        if result.complete() {
            assert!(result.proportional_bytes > 0);
        } else {
            assert_eq!(result.readable, 0);
            assert_eq!(result.proportional_bytes, 0);
        }
    }
}
