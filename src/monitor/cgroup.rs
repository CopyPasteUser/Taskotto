//! Selected-process, read-only cgroup v2 inspection. Counters describe the
//! cgroup (including other members/descendants), not just the chosen PID.
use super::{PressureMetric, psi::parse_pressure, start_ticks};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::Instant,
};

#[derive(Clone, Debug)]
pub(crate) struct CgroupReport {
    pub path: String,
    /// Cgroupfs inode disambiguates deletion/recreation at the same path.
    pub inode: u64,
    pub collected_at: Instant,
    /// Bounded one-level neighborhood from the same read-only cgroupfs view.
    pub neighborhood: CgroupNeighborhood,
    /// Names inferred strictly from kernel cgroup path segments; no runtime or
    /// systemd API is queried and these are not verified app/container IDs.
    pub associations: Vec<AssociationHint>,
    pub pressure: CgroupPressure,
    pub cpu_usage_usec: Option<u64>,
    pub cpu_throttled_usec: Option<u64>,
    pub cpu_throttled_events: Option<u64>,
    pub memory_bytes: Option<u64>,
    pub memory_limit: MemoryLimit,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct CgroupRateComparison {
    pub elapsed_seconds: f64,
    /// CPU time relative to one logical core: >100% is possible.
    pub cpu_core_percent: Option<f64>,
    pub io_read_bytes_per_second: Option<f64>,
    pub io_write_bytes_per_second: Option<f64>,
}

/// Compare only two explicit inspections of the same kernel cgroup inode.
/// Counter reset, insufficient elapsed time or a missing source stays unknown.
pub(crate) fn compare_rates(
    older: &CgroupReport,
    newer: &CgroupReport,
) -> Option<CgroupRateComparison> {
    if older.path != newer.path || older.inode != newer.inode {
        return None;
    }
    let elapsed = newer
        .collected_at
        .checked_duration_since(older.collected_at)?
        .as_secs_f64();
    if elapsed < 0.25 {
        return None;
    }
    let cpu_usage = counter_rate(older.cpu_usage_usec, newer.cpu_usage_usec, elapsed);
    Some(CgroupRateComparison {
        elapsed_seconds: elapsed,
        cpu_core_percent: cpu_usage.map(|value| value * 100.0 / 1_000_000.0),
        io_read_bytes_per_second: counter_rate(older.io_read_bytes, newer.io_read_bytes, elapsed),
        io_write_bytes_per_second: counter_rate(
            older.io_write_bytes,
            newer.io_write_bytes,
            elapsed,
        ),
    })
}

fn counter_rate(previous: Option<u64>, current: Option<u64>, seconds: f64) -> Option<f64> {
    let delta = current?.checked_sub(previous?)?;
    (seconds.is_finite() && seconds >= 0.25).then_some(delta as f64 / seconds)
}

#[derive(Clone, Debug)]
pub(crate) struct CgroupNeighbor {
    pub path: String,
    pub cpu_usage_usec: Option<u64>,
    pub memory_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CgroupNeighborhood {
    pub parent: Option<CgroupNeighbor>,
    /// None means directory enumeration failed, not that the group is empty.
    pub children: Option<Vec<CgroupNeighbor>>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct CgroupPressure {
    pub cpu_some: Option<PressureMetric>,
    pub cpu_full: Option<PressureMetric>,
    pub memory_some: Option<PressureMetric>,
    pub memory_full: Option<PressureMetric>,
    pub io_some: Option<PressureMetric>,
    pub io_full: Option<PressureMetric>,
}

impl CgroupPressure {
    pub fn is_empty(&self) -> bool {
        self.cpu_some.is_none()
            && self.cpu_full.is_none()
            && self.memory_some.is_none()
            && self.memory_full.is_none()
            && self.io_some.is_none()
            && self.io_full.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AssociationHint {
    pub category: &'static str,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MemoryLimit {
    Bytes(u64),
    Unlimited,
    Unavailable,
}

pub(crate) fn inspect(pid: i32, original_start: u64) -> Result<CgroupReport, String> {
    inspect_under(pid, original_start, Path::new("/sys/fs/cgroup"))
}

fn inspect_under(pid: i32, original_start: u64, root: &Path) -> Result<CgroupReport, String> {
    if start_ticks(pid).map_err(|err| err.to_string())? != original_start {
        return Err("Selected PID identity changed before cgroup inspection".into());
    }
    if !root.join("cgroup.controllers").exists() {
        return Err("cgroup v2 hierarchy unavailable or not mounted here".into());
    }
    let member = fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .map_err(|err| format!("Cannot read cgroup membership: {err}"))?;
    let path = parse_unified_path(&member).ok_or("No valid unified cgroup v2 membership")?;
    let target = safe_cgroup_path(root, &path).ok_or("Invalid cgroup path")?;
    // cgroupfs paths are kernel-managed; ensure a path cannot traverse a
    // different mount, even in unusual nested namespace configurations.
    let canonical_root = fs::canonicalize(root).map_err(|err| err.to_string())?;
    let canonical_target =
        fs::canonicalize(&target).map_err(|err| format!("Cgroup unavailable: {err}"))?;
    if !canonical_target.starts_with(&canonical_root) {
        return Err("Cgroup path escapes mounted hierarchy".into());
    }
    use std::os::unix::fs::MetadataExt;
    let inode = fs::metadata(&canonical_target)
        .map_err(|err| format!("Cannot identify cgroup inode: {err}"))?
        .ino();
    let cpu = fs::read_to_string(target.join("cpu.stat")).unwrap_or_default();
    let memory_bytes = read_number(&target.join("memory.current"));
    let memory_limit = match fs::read_to_string(target.join("memory.max")) {
        Ok(value) if value.trim() == "max" => MemoryLimit::Unlimited,
        Ok(value) => value
            .trim()
            .parse::<u64>()
            .map(MemoryLimit::Bytes)
            .unwrap_or(MemoryLimit::Unavailable),
        Err(_) => MemoryLimit::Unavailable,
    };
    let io_stat = fs::read_to_string(target.join("io.stat")).ok();
    let pressure = read_cgroup_pressure(&target);
    let neighborhood = read_neighborhood(&canonical_root, &canonical_target);
    let (io_read_bytes, io_write_bytes) = io_stat
        .as_deref()
        .and_then(parse_io_stat)
        .map_or((None, None), |(read, write)| (Some(read), Some(write)));
    if start_ticks(pid).map_err(|err| err.to_string())? != original_start {
        return Err("Selected PID identity changed during cgroup inspection".into());
    }
    let latest_membership = fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .map_err(|err| format!("Cannot recheck cgroup membership: {err}"))?;
    if parse_unified_path(&latest_membership).as_deref() != Some(path.as_str()) {
        return Err("Selected process moved to another cgroup during inspection".into());
    }
    if fs::metadata(&canonical_target)
        .map_err(|err| format!("Cgroup changed during inspection: {err}"))?
        .ino()
        != inode
    {
        return Err("Cgroup was replaced during inspection".into());
    }
    Ok(CgroupReport {
        associations: associations_from_path(&path),
        inode,
        collected_at: Instant::now(),
        neighborhood,
        pressure,
        path,
        cpu_usage_usec: parse_cpu_stat(&cpu, "usage_usec"),
        cpu_throttled_usec: parse_cpu_stat(&cpu, "throttled_usec"),
        cpu_throttled_events: parse_cpu_stat(&cpu, "nr_throttled"),
        memory_bytes,
        memory_limit,
        io_read_bytes,
        io_write_bytes,
    })
}

/// Inspect only the direct parent and at most 16 of the immediate children.
/// Reading 128 entries is the upper bound, even in huge containerized trees;
/// these are cumulative group counters, not disjoint per-process totals.
fn read_neighborhood(root: &Path, target: &Path) -> CgroupNeighborhood {
    const SCAN_LIMIT: usize = 128;
    const CHILD_LIMIT: usize = 16;
    let parent = (target != root)
        .then(|| target.parent())
        .flatten()
        .filter(|path| path.starts_with(root))
        .map(|path| neighbor_from_path(root, path));
    let Ok(entries) = fs::read_dir(target) else {
        return CgroupNeighborhood {
            parent,
            ..CgroupNeighborhood::default()
        };
    };
    let mut children = Vec::new();
    let mut truncated = false;
    for (index, item) in entries.enumerate() {
        if index == SCAN_LIMIT {
            truncated = true;
            break;
        }
        let Ok(item) = item else {
            truncated = true;
            continue;
        };
        if !item.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let Ok(child) = fs::canonicalize(item.path()) else {
            truncated = true;
            continue;
        };
        if child.starts_with(root) && child.parent() == Some(target) {
            children.push(neighbor_from_path(root, &child));
        }
    }
    children.sort_by(|a, b| a.path.cmp(&b.path));
    if children.len() > CHILD_LIMIT {
        children.truncate(CHILD_LIMIT);
        truncated = true;
    }
    CgroupNeighborhood {
        parent,
        children: Some(children),
        truncated,
    }
}

fn neighbor_from_path(root: &Path, group: &Path) -> CgroupNeighbor {
    let path = group.strip_prefix(root).unwrap_or(group).to_string_lossy();
    let path = format!("/{path}").trim_end_matches('/').to_owned();
    CgroupNeighbor {
        path: if path.is_empty() { "/".into() } else { path },
        cpu_usage_usec: fs::read_to_string(group.join("cpu.stat"))
            .ok()
            .and_then(|text| parse_cpu_stat(&text, "usage_usec")),
        memory_bytes: read_number(&group.join("memory.current")),
    }
}

fn read_cgroup_pressure(target: &Path) -> CgroupPressure {
    // Newer kernels allow disabling PSI accounting for an individual cgroup.
    // Existing files may still be readable but their zeros no longer mean
    // "no stalls". Suppress them instead of claiming a healthy workload.
    if fs::read_to_string(target.join("cgroup.pressure"))
        .ok()
        .as_deref()
        .is_some_and(pressure_accounting_disabled)
    {
        return CgroupPressure::default();
    }
    let cpu = fs::read_to_string(target.join("cpu.pressure")).ok();
    let memory = fs::read_to_string(target.join("memory.pressure")).ok();
    let io = fs::read_to_string(target.join("io.pressure")).ok();
    parse_cgroup_pressure(cpu.as_deref(), memory.as_deref(), io.as_deref())
}

fn pressure_accounting_disabled(value: &str) -> bool {
    value.trim() == "0"
}

fn parse_cgroup_pressure(
    cpu: Option<&str>,
    memory: Option<&str>,
    io: Option<&str>,
) -> CgroupPressure {
    CgroupPressure {
        cpu_some: cpu.and_then(|v| parse_pressure(v, "some")),
        cpu_full: cpu.and_then(|v| parse_pressure(v, "full")),
        memory_some: memory.and_then(|v| parse_pressure(v, "some")),
        memory_full: memory.and_then(|v| parse_pressure(v, "full")),
        io_some: io.and_then(|v| parse_pressure(v, "some")),
        io_full: io.and_then(|v| parse_pressure(v, "full")),
    }
}

fn associations_from_path(path: &str) -> Vec<AssociationHint> {
    let mut hints = Vec::new();
    // The only trusted fact is the exact kernel membership path. Unit and
    // runtime ownership remain unverified without talking to those services.
    for component in path.split('/').filter(|part| !part.is_empty()) {
        if component.len() > 160 || component.chars().any(char::is_control) {
            continue;
        }
        if let Some(id) = component
            .strip_prefix("docker-")
            .and_then(|v| v.strip_suffix(".scope"))
            && id.len() == 64
            && id.bytes().all(|b| b.is_ascii_hexdigit())
        {
            hints.push(AssociationHint {
                category: "Docker-like cgroup scope",
                detail: format!("ID prefix {} (unverified)", &id[..12]),
            });
        } else if let Some(id) = component
            .strip_prefix("libpod-")
            .and_then(|v| v.strip_suffix(".scope"))
            && id.len() == 64
            && id.bytes().all(|b| b.is_ascii_hexdigit())
        {
            hints.push(AssociationHint {
                category: "Podman-like cgroup scope",
                detail: format!("ID prefix {} (unverified)", &id[..12]),
            });
        } else if component.starts_with("app-flatpak-") && component.ends_with(".scope") {
            hints.push(AssociationHint {
                category: "Flatpak-like cgroup scope",
                detail: component.to_owned(),
            });
        } else if component.ends_with(".service") && component.len() > ".service".len() {
            hints.push(AssociationHint {
                category: "systemd service-like path segment",
                detail: component.to_owned(),
            });
        } else if component.ends_with(".scope") && component.len() > ".scope".len() {
            hints.push(AssociationHint {
                category: "systemd scope-like path segment",
                detail: component.to_owned(),
            });
        }
    }
    // Deeply nested cgroups may have many .scope segments. Keep the summary
    // compact while retaining the full authoritative path above it.
    hints
        .into_iter()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn parse_unified_path(text: &str) -> Option<String> {
    let path = text.lines().find_map(|line| line.strip_prefix("0::"))?;
    if path.starts_with('/') && !path.contains('\0') && path.len() <= 4_096 {
        Some(path.to_string())
    } else {
        None
    }
}

fn safe_cgroup_path(root: &Path, cgroup: &str) -> Option<PathBuf> {
    let mut result = root.to_path_buf();
    for part in Path::new(cgroup).components() {
        match part {
            Component::RootDir => {}
            Component::Normal(name) => result.push(name),
            _ => return None,
        }
    }
    Some(result)
}

fn read_number(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

fn parse_cpu_stat(text: &str, field: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let (name, value) = line.split_once(' ')?;
        (name == field)
            .then(|| value.trim().parse::<u64>().ok())
            .flatten()
    })
}

fn parse_io_stat(text: &str) -> Option<(u64, u64)> {
    let mut read = 0_u64;
    let mut written = 0_u64;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut tokens = line.split_whitespace();
        let device = tokens.next()?;
        if !device.contains(':') {
            return None;
        }
        let mut has_read = false;
        let mut has_write = false;
        for token in tokens {
            if let Some(value) = token.strip_prefix("rbytes=") {
                read = read.checked_add(value.parse::<u64>().ok()?)?;
                has_read = true;
            } else if let Some(value) = token.strip_prefix("wbytes=") {
                written = written.checked_add(value.parse::<u64>().ok()?)?;
                has_write = true;
            }
        }
        if !has_read || !has_write {
            return None;
        }
    }
    Some((read, written))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_only_unified_paths_and_rejects_traversal() {
        assert_eq!(
            parse_unified_path("2:cpu:/legacy\n0::/user.slice/test\n"),
            Some("/user.slice/test".into())
        );
        assert_eq!(parse_unified_path("2:cpu:/legacy"), None);
        assert!(safe_cgroup_path(Path::new("/sys/fs/cgroup"), "/../../etc").is_none());
        assert_eq!(
            safe_cgroup_path(Path::new("/sys/fs/cgroup"), "/user.slice/a").unwrap(),
            Path::new("/sys/fs/cgroup/user.slice/a")
        );
    }

    #[test]
    fn accounts_cpu_and_io_without_inventing_missing_fields() {
        assert_eq!(
            parse_cpu_stat("usage_usec 520\nnr_throttled 4\n", "usage_usec"),
            Some(520)
        );
        assert_eq!(parse_cpu_stat("usage_usec 520\n", "throttled_usec"), None);
        assert_eq!(
            parse_io_stat("8:0 rbytes=80 wbytes=4 rios=3\n8:16 rbytes=100 wbytes=6\n"),
            Some((180, 10))
        );
        assert_eq!(parse_io_stat("8:0 rbytes=n/a wbytes=4"), None);
        assert_eq!(parse_io_stat("8:0 rios=1 wios=2"), None);
    }

    #[test]
    fn current_process_inspection_is_safe_when_cgroups_unavailable() {
        let pid = std::process::id() as i32;
        let start = start_ticks(pid).expect("own process start ticks");
        assert!(inspect(pid, start + 1).is_err());
        if let Ok(result) = inspect(pid, start) {
            assert!(result.path.starts_with('/'));
            if let Some(cpu) = result.pressure.cpu_some {
                assert!(cpu.avg10_percent <= 100.0);
            }
        }
    }

    #[test]
    fn identifies_only_naming_hints_and_never_claims_runtime_verification() {
        let docker_id = "a".repeat(64);
        let podman_id = "b".repeat(64);
        let path = format!(
            "/system.slice/sshd.service/docker-{docker_id}.scope/libpod-{podman_id}.scope/app-flatpak-org.example.App-42.scope"
        );
        let hints = associations_from_path(&path);
        assert_eq!(hints.len(), 4);
        assert_eq!(hints[0].category, "systemd service-like path segment");
        assert_eq!(hints[1].category, "Docker-like cgroup scope");
        assert_eq!(hints[1].detail, "ID prefix aaaaaaaaaaaa (unverified)");
        assert_eq!(hints[2].category, "Podman-like cgroup scope");
        assert_eq!(hints[3].category, "Flatpak-like cgroup scope");
    }

    #[test]
    fn arbitrary_names_and_incorrect_ids_do_not_make_container_claims() {
        assert!(associations_from_path("/user.slice/app.slice/myprogram").is_empty());
        let hints = associations_from_path("/user.slice/docker-NOT-A-CONTAINER.scope/fake.service");
        assert_eq!(hints.len(), 2);
        assert_eq!(hints[0].category, "systemd scope-like path segment");
        assert_eq!(hints[1].category, "systemd service-like path segment");
        assert!(associations_from_path("/a\0bad.service").is_empty());
    }

    #[test]
    fn parses_cgroup_specific_psi_without_mixing_system_scope_or_missing_sources() {
        let cpu = "some avg10=9.50 avg60=6.00 avg300=4.00 total=90000\nfull avg10=1.25 avg60=1.00 avg300=0.50 total=23000\n";
        let memory = "some avg10=3.00 avg60=2.00 avg300=1.00 total=500\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
        let report = parse_cgroup_pressure(Some(cpu), Some(memory), None);
        assert_eq!(report.cpu_some.unwrap().avg10_percent, 9.5);
        assert_eq!(report.cpu_full.unwrap().total_stall_usec, 23000);
        assert_eq!(report.memory_some.unwrap().avg60_percent, 2.0);
        assert_eq!(report.memory_full.unwrap().total_stall_usec, 0);
        assert!(report.io_some.is_none());
        assert!(report.io_full.is_none());
    }

    #[test]
    fn malformed_or_absent_cgroup_pressure_is_unavailable_not_zero() {
        assert!(parse_cgroup_pressure(None, None, None).is_empty());
        let report = parse_cgroup_pressure(
            Some("some avg10=NaN avg60=0 avg300=0 total=0"),
            None,
            Some("full avg10=2 avg60=1 avg300=0 total=100"),
        );
        assert!(report.cpu_some.is_none());
        assert!(report.cpu_full.is_none());
        assert!(report.io_some.is_none());
        assert_eq!(report.io_full.unwrap().avg10_percent, 2.0);
        assert!(pressure_accounting_disabled("0\n"));
        assert!(!pressure_accounting_disabled("1\n"));
    }

    #[test]
    fn neighboring_cgroups_are_bounded_sorted_and_not_summed() {
        let root = std::env::temp_dir().join(format!(
            "taskotto-cgroup-neighbors-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let selected = root.join("selected.scope");
        fs::create_dir_all(&selected).unwrap();
        fs::write(root.join("cpu.stat"), "usage_usec 500\n").unwrap();
        fs::write(root.join("memory.current"), "8192\n").unwrap();
        for i in 0..20 {
            let child = selected.join(format!("child-{i:02}.scope"));
            fs::create_dir_all(&child).unwrap();
            fs::write(child.join("cpu.stat"), format!("usage_usec {}\n", 100 + i)).unwrap();
            fs::write(child.join("memory.current"), "4096\n").unwrap();
        }
        let group = read_neighborhood(&root, &selected);
        assert_eq!(group.parent.as_ref().unwrap().path, "/");
        assert_eq!(group.parent.as_ref().unwrap().cpu_usage_usec, Some(500));
        assert_eq!(group.parent.as_ref().unwrap().memory_bytes, Some(8192));
        let children = group.children.unwrap();
        assert_eq!(children.len(), 16);
        assert!(group.truncated);
        assert!(children.windows(2).all(|w| w[0].path <= w[1].path));
        assert!(
            children
                .iter()
                .all(|child| child.memory_bytes == Some(4096))
        );
        assert!(children.iter().all(|child| child.cpu_usage_usec.is_some()));
        let missing = read_neighborhood(&root, &root.join("not-found"));
        assert!(missing.children.is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cgroup_rates_require_matching_inode_elapsed_time_and_monotonic_counters() {
        use std::time::Duration;
        let earlier = CgroupReport {
            path: "/user.slice/sample.scope".into(),
            inode: 7,
            collected_at: Instant::now(),
            neighborhood: CgroupNeighborhood::default(),
            associations: Vec::new(),
            pressure: CgroupPressure::default(),
            cpu_usage_usec: Some(1_000_000),
            cpu_throttled_usec: None,
            cpu_throttled_events: None,
            memory_bytes: None,
            memory_limit: MemoryLimit::Unavailable,
            io_read_bytes: Some(10_000),
            io_write_bytes: None,
        };
        let mut later = earlier.clone();
        later.collected_at += Duration::from_secs(2);
        later.cpu_usage_usec = Some(1_400_000);
        later.io_read_bytes = Some(18_192);
        let rates = compare_rates(&earlier, &later).unwrap();
        assert_eq!(rates.elapsed_seconds, 2.0);
        assert_eq!(rates.cpu_core_percent, Some(20.0));
        assert_eq!(rates.io_read_bytes_per_second, Some(4_096.0));
        assert!(rates.io_write_bytes_per_second.is_none());
        later.io_read_bytes = Some(5_000);
        assert!(
            compare_rates(&earlier, &later)
                .unwrap()
                .io_read_bytes_per_second
                .is_none()
        );
        later.inode = 8; // path was reused after cgroup deletion
        assert!(compare_rates(&earlier, &later).is_none());
        later.inode = 7;
        later.path = "/user.slice/new.scope".into();
        assert!(compare_rates(&earlier, &later).is_none());
        later.path = earlier.path.clone();
        later.collected_at = earlier.collected_at + Duration::from_millis(100);
        assert!(compare_rates(&earlier, &later).is_none());
        assert!(counter_rate(Some(100), Some(90), 2.0).is_none());
        assert!(counter_rate(Some(100), None, 2.0).is_none());
    }
}
