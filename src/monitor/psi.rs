//! Linux pressure stall information (PSI) via the kernel's read-only
//! /proc/pressure interface. Percentages measure stalled task time, not
//! resource utilization; the same strict parser is reused for cgroup v2 PSI.
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize)]
pub struct PressureMetric {
    pub avg10_percent: f32,
    pub avg60_percent: f32,
    pub avg300_percent: f32,
    pub total_stall_usec: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct PressureSnapshot {
    pub cpu_some: Option<PressureMetric>,
    pub memory_some: Option<PressureMetric>,
    pub memory_full: Option<PressureMetric>,
    pub io_some: Option<PressureMetric>,
    pub io_full: Option<PressureMetric>,
}

impl PressureSnapshot {
    pub fn is_empty(&self) -> bool {
        self.cpu_some.is_none()
            && self.memory_some.is_none()
            && self.memory_full.is_none()
            && self.io_some.is_none()
            && self.io_full.is_none()
    }
}

pub(super) fn sample() -> PressureSnapshot {
    sample_from(Path::new("/proc/pressure"))
}

fn sample_from(root: &Path) -> PressureSnapshot {
    let cpu = fs::read_to_string(root.join("cpu")).ok();
    let memory = fs::read_to_string(root.join("memory")).ok();
    let io = fs::read_to_string(root.join("io")).ok();
    PressureSnapshot {
        cpu_some: cpu
            .as_deref()
            .and_then(|value| parse_pressure(value, "some")),
        memory_some: memory
            .as_deref()
            .and_then(|value| parse_pressure(value, "some")),
        memory_full: memory
            .as_deref()
            .and_then(|value| parse_pressure(value, "full")),
        io_some: io
            .as_deref()
            .and_then(|value| parse_pressure(value, "some")),
        io_full: io
            .as_deref()
            .and_then(|value| parse_pressure(value, "full")),
    }
}

pub(super) fn parse_pressure(text: &str, level: &str) -> Option<PressureMetric> {
    let row = text
        .lines()
        .find(|line| line.split_whitespace().next() == Some(level))?;
    let mut avg10 = None;
    let mut avg60 = None;
    let mut avg300 = None;
    let mut total = None;
    for field in row.split_whitespace().skip(1) {
        let (key, value) = field.split_once('=')?;
        match key {
            "avg10" if avg10.is_none() => avg10 = valid_percent(value),
            "avg60" if avg60.is_none() => avg60 = valid_percent(value),
            "avg300" if avg300.is_none() => avg300 = valid_percent(value),
            "total" if total.is_none() => total = value.parse::<u64>().ok(),
            _ => return None,
        }
    }
    Some(PressureMetric {
        avg10_percent: avg10?,
        avg60_percent: avg60?,
        avg300_percent: avg300?,
        total_stall_usec: total?,
    })
}

fn valid_percent(text: &str) -> Option<f32> {
    let number = text.parse::<f32>().ok()?;
    (number.is_finite() && (0.0..=100.0).contains(&number)).then_some(number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_kernel_percentages_and_cumulative_microseconds() {
        let text = "some avg10=3.50 avg60=1.25 avg300=0.50 total=123456\nfull avg10=0.10 avg60=0.20 avg300=0.30 total=150\n";
        let result = parse_pressure(text, "some").unwrap();
        assert_eq!(result.avg10_percent, 3.5);
        assert_eq!(result.avg60_percent, 1.25);
        assert_eq!(result.total_stall_usec, 123456);
        assert_eq!(parse_pressure(text, "full").unwrap().total_stall_usec, 150);
    }

    #[test]
    fn refuses_missing_invalid_and_nonfinite_pressure_data() {
        for value in ["NaN", "inf", "-0.1", "101", "abc"] {
            assert!(
                parse_pressure(
                    &format!("some avg10={value} avg60=1 avg300=1 total=1"),
                    "some"
                )
                .is_none()
            );
        }
        assert!(parse_pressure("some avg10=1 avg60=1 total=1", "some").is_none());
        assert!(parse_pressure("some avg10=1 avg10=2 avg60=1 avg300=1 total=1", "some").is_none());
        assert!(parse_pressure("some avg10=1 avg60=1 avg300=1 total=-1", "some").is_none());
        assert!(sample_from(Path::new("/dev/taskotto-nonexistent-pressure")).is_empty());
    }

    #[test]
    fn actual_proc_psi_is_optional_not_assumed() {
        let measured = sample();
        if let Some(cpu) = measured.cpu_some {
            assert!(cpu.avg10_percent <= 100.0);
        }
    }
}
