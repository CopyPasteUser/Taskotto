//! Selected-process, read-only cgroup v2 inspection. Counters describe the
//! cgroup (including other members/descendants), not just the chosen PID.
use super::start_ticks;
use std::{
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug)]
pub(crate) struct CgroupReport {
    pub path: String,
    /// Names inferred strictly from kernel cgroup path segments; no runtime or
    /// systemd API is queried and these are not verified app/container IDs.
    pub associations: Vec<AssociationHint>,
    pub cpu_usage_usec: Option<u64>,
    pub cpu_throttled_usec: Option<u64>,
    pub cpu_throttled_events: Option<u64>,
    pub memory_bytes: Option<u64>,
    pub memory_limit: MemoryLimit,
    pub io_read_bytes: Option<u64>,
    pub io_write_bytes: Option<u64>,
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
    Ok(CgroupReport {
        associations: associations_from_path(&path),
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
}
