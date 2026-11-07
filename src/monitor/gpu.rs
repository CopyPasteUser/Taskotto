//! Read-only DRM/sysfs GPU telemetry. This deliberately does not claim
//! universal GPU utilization: AMDGPU sysfs provides busy/VRAM counters,
//! while Intel/NVIDIA devices are inventoried with explicitly absent values
//! until their supported driver-specific backends are implemented.
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize)]
pub struct Gpu {
    pub card: String,
    pub vendor: String,
    pub driver: Option<String>,
    pub utilization_percent: Option<u8>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f32>,
    pub power_watts: Option<f32>,
    pub core_clock_mhz: Option<f32>,
}

pub(super) fn sample() -> Vec<Gpu> {
    sample_from(Path::new("/sys/class/drm"))
}

fn sample_from(directory: &Path) -> Vec<Gpu> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut gpus = entries
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.strip_prefix("card").is_some_and(|digits| {
                !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
            }) {
                return None;
            }
            read_card(&entry.path(), &name)
        })
        .collect::<Vec<_>>();
    gpus.sort_by(|a, b| a.card.cmp(&b.card));
    gpus
}

fn read_card(path: &Path, card: &str) -> Option<Gpu> {
    let device = path.join("device");
    let vendor_id = fs::read_to_string(device.join("vendor")).ok()?;
    let vendor = match vendor_id.trim().to_ascii_lowercase().as_str() {
        "0x1002" => "AMD",
        "0x8086" => "Intel",
        "0x10de" => "NVIDIA",
        _ => "Other",
    }
    .to_string();
    let driver = fs::read_link(device.join("driver"))
        .ok()
        .and_then(|link| {
            link.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .or_else(|| {
            fs::read_to_string(device.join("uevent"))
                .ok()?
                .lines()
                .find_map(|line| line.strip_prefix("DRIVER=").map(str::to_owned))
        });
    let amdgpu = vendor == "AMD" && driver.as_deref() == Some("amdgpu");
    let hwmon = fs::read_dir(device.join("hwmon"))
        .ok()
        .and_then(|entries| {
            entries
                .flatten()
                .find(|entry| entry.file_name().to_string_lossy().starts_with("hwmon"))
        })
        .map(|entry| entry.path());
    let read_sensor = |field: &str| {
        hwmon
            .as_ref()
            .and_then(|hwmon| read_number(&hwmon.join(field)))
    };
    let utilization_percent = if amdgpu {
        read_number(&device.join("gpu_busy_percent"))
            .filter(|value| *value <= 100)
            .map(|value| value as u8)
    } else {
        None
    };
    let (vram_total_bytes, vram_used_bytes) = if amdgpu {
        let total = read_number(&device.join("mem_info_vram_total")).filter(|n| *n > 0);
        let used = read_number(&device.join("mem_info_vram_used"))
            .filter(|used| total.is_some_and(|total| *used <= total));
        (total, used)
    } else {
        (None, None)
    };
    let temperature_celsius = read_sensor("temp1_input")
        .filter(|value| *value <= 150_000)
        .map(|value| value as f32 / 1_000.0);
    let power_watts = read_sensor("power1_average")
        .filter(|value| *value <= 1_000_000_000)
        .map(|value| value as f32 / 1_000_000.0);
    // AMDGPU hwmon frequency is in Hz; Intel i915 exposes a separate MHz
    // current-frequency sensor when supported by its GT sysfs ABI.
    let core_clock_mhz = if amdgpu {
        read_sensor("freq1_input")
            .filter(|value| *value <= 10_000_000_000)
            .map(|value| value as f32 / 1_000_000.0)
    } else if vendor == "Intel" {
        read_number(&path.join("gt/gt0/rps_act_freq_mhz"))
            .filter(|value| *value <= 10_000)
            .map(|value| value as f32)
    } else {
        None
    };
    Some(Gpu {
        card: card.into(),
        vendor,
        driver,
        utilization_percent,
        vram_used_bytes,
        vram_total_bytes,
        temperature_celsius,
        power_watts,
        core_clock_mhz,
    })
}

fn read_number(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_drm_directory_is_a_valid_empty_inventory() {
        assert!(sample_from(Path::new("/dev/taskotto-nonexistent-drm")).is_empty());
    }

    #[test]
    fn parses_realistic_amdgpu_sysfs_units_and_rejects_connectors() {
        let directory =
            std::env::temp_dir().join(format!("taskotto-gpu-test-{}", std::process::id()));
        let device = directory.join("card4/device");
        let hwmon = device.join("hwmon/hwmon0");
        fs::create_dir_all(&hwmon).unwrap();
        let fixtures = [
            (device.join("vendor"), "0x1002"),
            (device.join("uevent"), "DRIVER=amdgpu\n"),
            (device.join("gpu_busy_percent"), "63"),
            (device.join("mem_info_vram_total"), "8589934592"),
            (device.join("mem_info_vram_used"), "1073741824"),
            (hwmon.join("temp1_input"), "57000"),
            (hwmon.join("power1_average"), "42000000"),
            (hwmon.join("freq1_input"), "1500000000"),
        ];
        for (path, value) in fixtures {
            fs::write(path, value).unwrap();
        }
        fs::create_dir_all(directory.join("card4-DP-1")).unwrap();
        let list = sample_from(&directory);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].utilization_percent, Some(63));
        assert_eq!(list[0].vram_used_bytes, Some(1_073_741_824));
        assert_eq!(list[0].temperature_celsius, Some(57.0));
        assert_eq!(list[0].power_watts, Some(42.0));
        assert_eq!(list[0].core_clock_mhz, Some(1500.0));
        fs::write(device.join("gpu_busy_percent"), "180").unwrap();
        fs::write(device.join("mem_info_vram_used"), "99999999999").unwrap();
        let invalid = sample_from(&directory);
        assert_eq!(invalid[0].utilization_percent, None);
        assert_eq!(invalid[0].vram_used_bytes, None);
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn intel_and_nvidia_do_not_invent_gpu_busy_counters() {
        let directory =
            std::env::temp_dir().join(format!("taskotto-gpu-capability-{}", std::process::id()));
        for (card, vendor) in [("card0", "0x8086"), ("card2", "0x10de")] {
            let device = directory.join(card).join("device");
            fs::create_dir_all(&device).unwrap();
            fs::write(device.join("vendor"), vendor).unwrap();
            fs::write(device.join("gpu_busy_percent"), "85").unwrap();
        }
        let gpus = sample_from(&directory);
        assert_eq!(gpus.len(), 2);
        assert!(gpus.iter().all(|gpu| gpu.utilization_percent.is_none()));
        fs::remove_dir_all(&directory).unwrap();
    }
}
