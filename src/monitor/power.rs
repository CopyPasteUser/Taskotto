//! Read-only Linux power_supply class. Energy is µWh, charge is µAh;
//! never combine these quantities to invent physical energy or runtime.
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Clone, Debug, Default, Serialize)]
pub struct PowerSnapshot {
    pub batteries: Vec<Battery>,
    pub external_power_online: Option<bool>,
}

impl PowerSnapshot {
    pub fn is_empty(&self) -> bool {
        self.batteries.is_empty() && self.external_power_online.is_none()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Battery {
    pub name: String,
    pub status: Option<String>,
    pub capacity_percent: Option<u8>,
    pub energy_now_wh: Option<f32>,
    pub energy_full_wh: Option<f32>,
    pub power_watts: Option<f32>,
}

pub(super) fn sample() -> PowerSnapshot {
    sample_from(Path::new("/sys/class/power_supply"))
}

fn sample_from(directory: &Path) -> PowerSnapshot {
    let mut result = PowerSnapshot::default();
    let Ok(entries) = fs::read_dir(directory) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let kind = fs::read_to_string(path.join("type")).unwrap_or_default();
        if kind.trim() == "Battery" {
            if result.batteries.len() < 4 {
                result
                    .batteries
                    .push(read_battery(&path, &entry.file_name().to_string_lossy()));
            }
        } else if let Some(online) = read_number(&path.join("online"))
            && online <= 1
        {
            // Several adapters can coexist; any active one counts.
            result.external_power_online =
                Some(result.external_power_online.unwrap_or(false) || online == 1);
        }
    }
    result.batteries.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

fn read_battery(path: &Path, name: &str) -> Battery {
    let energy_now = read_number(&path.join("energy_now"));
    let energy_full = read_number(&path.join("energy_full")).filter(|value| *value > 0);
    let charge_now = read_number(&path.join("charge_now"));
    let charge_full = read_number(&path.join("charge_full")).filter(|value| *value > 0);
    let capacity_percent = read_number(&path.join("capacity"))
        .filter(|value| *value <= 100)
        .map(|value| value as u8)
        .or_else(|| {
            let (now, full) = match (energy_now, energy_full) {
                (Some(now), Some(full)) => (now, full),
                _ => (charge_now?, charge_full?),
            };
            if now > full {
                return None;
            }
            Some(((now as u128 * 100 + full as u128 / 2) / full as u128).min(100) as u8)
        });
    let status = fs::read_to_string(path.join("status"))
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| {
            matches!(
                s.as_str(),
                "Charging" | "Discharging" | "Full" | "Not charging" | "Unknown"
            )
        });
    Battery {
        name: name.to_owned(),
        status,
        capacity_percent,
        energy_now_wh: energy_now
            .filter(|n| *n <= 1_000_000_000 && energy_full.is_none_or(|full| *n <= full))
            .map(|n| n as f32 / 1_000_000.0),
        energy_full_wh: energy_full
            .filter(|n| *n <= 1_000_000_000)
            .map(|n| n as f32 / 1_000_000.0),
        power_watts: read_number(&path.join("power_now"))
            .filter(|n| *n <= 1_000_000_000)
            .map(|n| n as f32 / 1_000_000.0),
    }
}

fn read_number(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_power_supply_is_not_a_fake_battery() {
        assert!(sample_from(Path::new("/dev/taskotto-nonexistent-power")).is_empty());
    }

    #[test]
    fn energy_and_charge_are_never_mixed() {
        let directory =
            std::env::temp_dir().join(format!("taskotto-power-test-{}", std::process::id()));
        let battery = directory.join("BAT1");
        let ac = directory.join("AC0");
        fs::create_dir_all(&battery).unwrap();
        fs::create_dir_all(&ac).unwrap();
        for (path, value) in [
            (battery.join("type"), "Battery"),
            (battery.join("status"), "Discharging"),
            (battery.join("energy_now"), "20000000"),
            (battery.join("energy_full"), "50000000"),
            (battery.join("charge_now"), "9999999"),
            (battery.join("charge_full"), "10000000"),
            (battery.join("power_now"), "8000000"),
            (ac.join("type"), "Mains"),
            (ac.join("online"), "0"),
        ] {
            fs::write(path, value).unwrap();
        }
        let battery_readings = sample_from(&directory);
        assert_eq!(battery_readings.external_power_online, Some(false));
        assert_eq!(battery_readings.batteries.len(), 1);
        assert_eq!(battery_readings.batteries[0].capacity_percent, Some(40));
        assert_eq!(battery_readings.batteries[0].energy_now_wh, Some(20.0));
        assert_eq!(battery_readings.batteries[0].power_watts, Some(8.0));
        fs::remove_file(battery.join("energy_now")).unwrap();
        fs::remove_file(battery.join("energy_full")).unwrap();
        assert_eq!(
            sample_from(&directory).batteries[0].capacity_percent,
            Some(100)
        );
        assert!(sample_from(&directory).batteries[0].energy_now_wh.is_none());
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn invalid_percent_and_energy_avoid_unreal_battery_values() {
        let directory =
            std::env::temp_dir().join(format!("taskotto-power-invalid-{}", std::process::id()));
        let battery = directory.join("BAT0");
        fs::create_dir_all(&battery).unwrap();
        fs::write(battery.join("type"), "Battery").unwrap();
        fs::write(battery.join("capacity"), "201").unwrap();
        fs::write(battery.join("energy_now"), "40000000").unwrap();
        fs::write(battery.join("energy_full"), "20000000").unwrap();
        let result = sample_from(&directory);
        assert_eq!(result.batteries[0].capacity_percent, None);
        assert_eq!(result.batteries[0].energy_now_wh, None);
        fs::remove_dir_all(&directory).unwrap();
    }
}
