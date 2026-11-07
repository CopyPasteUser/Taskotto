//! Conservative, explainable hypotheses from *multiple* real PSI samples.
//! These are hints for human investigation, never automated diagnoses or
//! process-control triggers. Thresholds are display filters, not Linux limits.
use super::History;
use crate::monitor::{PressureMetric, PressureSnapshot};

pub(super) struct PressureHint {
    pub label: &'static str,
    pub evidence: String,
    pub next_step: &'static str,
}

fn sustained(history: &History, index: usize, metric: &PressureMetric, minimum: f32) -> bool {
    metric.avg10_percent.is_finite()
        && metric.avg60_percent.is_finite()
        && metric.avg10_percent >= minimum
        && metric.avg60_percent >= minimum / 2.0
        && history.psi_ticks.len() >= 8
        && history
            .psi_ticks
            .iter()
            .rev()
            .take(8)
            .all(|tick| tick[index].is_some_and(|value| value.is_finite()))
        && history
            .psi_ticks
            .iter()
            .rev()
            .take(8)
            .filter_map(|tick| tick[index])
            .filter(|value| value.is_finite() && *value >= minimum)
            .count()
            >= 6
}

pub(super) fn analyze(
    history: &History,
    current: &PressureSnapshot,
    host_cpu_percent: f32,
) -> Vec<PressureHint> {
    let mut hints = Vec::new();
    if let Some(cpu) = current.cpu_some.as_ref()
        && sustained(history, 0, cpu, 10.0)
    {
        let busy = if host_cpu_percent.is_finite() && host_cpu_percent >= 85.0 {
            format!(" Host CPU busy: {host_cpu_percent:.1}% (separate utilization signal).")
        } else {
            String::new()
        };
        hints.push(PressureHint {
            label: "Possible sustained CPU contention",
            evidence: format!("CPU SOME stalls: {:.1}% (10s), {:.1}% (60s), >=10% in at least 6 of 8 recent readings.{busy}", cpu.avg10_percent, cpu.avg60_percent),
            next_step: "Inspect the CPU process list and any relevant cgroup CPU throttling; high PSI alone does not identify the offending process.",
        });
    }
    let memory = current
        .memory_full
        .as_ref()
        .filter(|metric| sustained(history, 2, metric, 1.0))
        .map(|metric| (metric, true))
        .or_else(|| {
            current
                .memory_some
                .as_ref()
                .filter(|metric| sustained(history, 1, metric, 5.0))
                .map(|metric| (metric, false))
        });
    if let Some((metric, full)) = memory {
        hints.push(PressureHint {
            label: if full { "Possible sustained full memory stalls" } else { "Possible sustained memory contention" },
            evidence: format!("Memory {} stalls: {:.1}% (10s), {:.1}% (60s), persistent in at least 6 of 8 recent readings.", if full { "FULL" } else { "SOME" }, metric.avg10_percent, metric.avg60_percent),
            next_step: "Compare RAM/available memory and the relevant cgroup memory limit; PSI cannot prove which process or allocation caused it.",
        });
    }
    let io = current
        .io_full
        .as_ref()
        .filter(|metric| sustained(history, 4, metric, 2.0))
        .map(|metric| (metric, true))
        .or_else(|| {
            current
                .io_some
                .as_ref()
                .filter(|metric| sustained(history, 3, metric, 10.0))
                .map(|metric| (metric, false))
        });
    if let Some((metric, full)) = io {
        hints.push(PressureHint {
            label: if full { "Possible sustained full I/O stalls" } else { "Possible sustained I/O contention" },
            evidence: format!("I/O {} stalls: {:.1}% (10s), {:.1}% (60s), persistent in at least 6 of 8 recent readings.", if full { "FULL" } else { "SOME" }, metric.avg10_percent, metric.avg60_percent),
            next_step: "Inspect block device throughput and waiting processes; pressure does not identify a particular disk or application.",
        });
    }
    hints
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(percent: f32) -> PressureMetric {
        PressureMetric {
            avg10_percent: percent,
            avg60_percent: percent,
            avg300_percent: percent,
            total_stall_usec: 1_000_000,
        }
    }

    #[test]
    fn no_hypothesis_from_one_sample_or_missing_psi() {
        let mut history = History::default();
        history
            .psi_ticks
            .push_back([Some(99.0), None, None, None, None]);
        let current = PressureSnapshot {
            cpu_some: Some(sample(99.0)),
            ..PressureSnapshot::default()
        };
        assert!(analyze(&history, &current, 95.0).is_empty());
    }

    #[test]
    fn requires_repeated_evidence_and_never_invents_root_cause() {
        let mut history = History {
            psi_ticks: std::collections::VecDeque::from(
                [[Some(20.0), None, Some(2.0), None, Some(3.0)]; 8],
            ),
            ..History::default()
        };
        let current = PressureSnapshot {
            cpu_some: Some(sample(20.0)),
            memory_full: Some(sample(2.0)),
            io_full: Some(sample(3.0)),
            ..PressureSnapshot::default()
        };
        let hints = analyze(&history, &current, 92.0);
        assert_eq!(hints.len(), 3);
        assert!(hints[0].evidence.contains("Host CPU busy"));
        assert!(hints.iter().all(|hint| hint.label.starts_with("Possible")));
        for (index, value) in [1.0, 2.0, 3.0, 4.0, 5.0, 7.0, 20.0, 20.0]
            .into_iter()
            .enumerate()
        {
            history.psi_ticks[index][0] = Some(value);
        }
        assert_eq!(analyze(&history, &current, 92.0).len(), 2);
    }

    #[test]
    fn no_hypotheses_with_missing_or_nonfinite_data() {
        let history = History {
            psi_ticks: std::collections::VecDeque::from(
                [[None, Some(f32::NAN), None, None, None]; 8],
            ),
            ..History::default()
        };
        let current = PressureSnapshot {
            memory_some: Some(sample(20.0)),
            ..PressureSnapshot::default()
        };
        assert!(analyze(&history, &current, 90.0).is_empty());
    }

    #[test]
    fn missing_sampler_ticks_cannot_be_interpreted_as_sustained_contention() {
        let history = History {
            psi_ticks: std::collections::VecDeque::from([
                [Some(20.0), None, None, None, None],
                [Some(20.0), None, None, None, None],
                [Some(20.0), None, None, None, None],
                [None; 5],
                [None; 5],
                [None; 5],
                [Some(20.0), None, None, None, None],
                [Some(20.0), None, None, None, None],
            ]),
            ..History::default()
        };
        let current = PressureSnapshot {
            cpu_some: Some(sample(20.0)),
            ..PressureSnapshot::default()
        };
        assert!(analyze(&history, &current, 95.0).is_empty());
    }

    #[test]
    fn old_pressure_outside_recent_eight_ticks_does_not_trigger_hint() {
        let mut history = History {
            psi_ticks: std::collections::VecDeque::from(
                [[Some(30.0), None, None, None, None]; 112],
            ),
            ..History::default()
        };
        history
            .psi_ticks
            .extend([[Some(0.0), None, None, None, None]; 8]);
        let current = PressureSnapshot {
            cpu_some: Some(sample(30.0)),
            ..PressureSnapshot::default()
        };
        assert!(analyze(&history, &current, 95.0).is_empty());
    }
}
