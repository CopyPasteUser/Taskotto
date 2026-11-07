//! Explicit, bounded exports of the current GUI session's sampler-visible
//! system metrics; procfs/namespace visibility can vary in containers.
//! samples. No PIDs, process names, commands, retained files or worker daemon.
use super::History;
use serde::Serialize;
use std::{
    fmt::Write as _,
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};

#[derive(Serialize)]
struct Export {
    format_version: u8,
    scope: &'static str,
    nominal_interval_ms: u64,
    sample_count: usize,
    samples: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    timestamp_unix_ms: Option<u64>,
    cpu_percent: Option<f32>,
    ram_used_percent: Option<f32>,
    network_rx_bytes_per_sec: Option<f32>,
    network_tx_bytes_per_sec: Option<f32>,
    disk_read_bytes_per_sec: Option<f32>,
    disk_write_bytes_per_sec: Option<f32>,
    psi_cpu_some_avg10_percent: Option<f32>,
    psi_memory_some_avg10_percent: Option<f32>,
    psi_memory_full_avg10_percent: Option<f32>,
    psi_io_some_avg10_percent: Option<f32>,
    psi_io_full_avg10_percent: Option<f32>,
}

fn valid(value: f32) -> Option<f32> {
    value.is_finite().then_some(value)
}

fn rows(history: &History, interval_ms: u64) -> io::Result<Export> {
    let timestamps = &history.sampled_at_unix_ms;
    let count = timestamps.len();
    if count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No session samples collected yet",
        ));
    }
    if count > 120
        || [
            history.cpu.len(),
            history.memory.len(),
            history.received.len(),
            history.sent.len(),
            history.disk_read.len(),
            history.disk_write.len(),
            history.psi_ticks.len(),
        ]
        .iter()
        .any(|length| *length != count)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Session sampler history is not aligned",
        ));
    }
    let samples = (0..count)
        .map(|index| {
            let psi = history.psi_ticks[index];
            Row {
                timestamp_unix_ms: timestamps[index],
                cpu_percent: valid(history.cpu[index]),
                ram_used_percent: valid(history.memory[index]),
                network_rx_bytes_per_sec: valid(history.received[index]),
                network_tx_bytes_per_sec: valid(history.sent[index]),
                disk_read_bytes_per_sec: valid(history.disk_read[index]),
                disk_write_bytes_per_sec: valid(history.disk_write[index]),
                psi_cpu_some_avg10_percent: psi[0].and_then(valid),
                psi_memory_some_avg10_percent: psi[1].and_then(valid),
                psi_memory_full_avg10_percent: psi[2].and_then(valid),
                psi_io_some_avg10_percent: psi[3].and_then(valid),
                psi_io_full_avg10_percent: psi[4].and_then(valid),
            }
        })
        .collect();
    Ok(Export {
        format_version: 1,
        scope: "linux-sampler-visible-system",
        nominal_interval_ms: interval_ms,
        sample_count: count,
        samples,
    })
}

fn cell<T: ToString>(value: Option<T>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn render_csv(export: &Export) -> Vec<u8> {
    let mut csv = String::from(
        "format_version,scope,nominal_interval_ms,timestamp_unix_ms,cpu_percent,ram_used_percent,network_rx_bytes_per_sec,network_tx_bytes_per_sec,disk_read_bytes_per_sec,disk_write_bytes_per_sec,psi_cpu_some_avg10_percent,psi_memory_some_avg10_percent,psi_memory_full_avg10_percent,psi_io_some_avg10_percent,psi_io_full_avg10_percent\n",
    );
    for sample in &export.samples {
        // All columns are fixed labels or numerical values. No user-supplied
        // strings enter CSV cells; no formula/quoting ambiguity is possible.
        writeln!(
            &mut csv,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            export.format_version,
            export.scope,
            export.nominal_interval_ms,
            cell(sample.timestamp_unix_ms),
            cell(sample.cpu_percent),
            cell(sample.ram_used_percent),
            cell(sample.network_rx_bytes_per_sec),
            cell(sample.network_tx_bytes_per_sec),
            cell(sample.disk_read_bytes_per_sec),
            cell(sample.disk_write_bytes_per_sec),
            cell(sample.psi_cpu_some_avg10_percent),
            cell(sample.psi_memory_some_avg10_percent),
            cell(sample.psi_memory_full_avg10_percent),
            cell(sample.psi_io_some_avg10_percent),
            cell(sample.psi_io_full_avg10_percent),
        )
        .expect("writing to a String cannot fail");
    }
    csv.into_bytes()
}

pub(super) fn write_new(
    history: &History,
    path: &Path,
    nominal_interval_ms: u64,
) -> io::Result<usize> {
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Enter an absolute destination path",
        ));
    }
    let export = rows(history, nominal_interval_ms)?;
    let contents = match path.extension().and_then(|ext| ext.to_str()) {
        Some("csv") => render_csv(&export),
        Some("json") => serde_json::to_vec_pretty(&export).map_err(io::Error::other)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Destination must end in .csv or .json",
            ));
        }
    };
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    if let Err(error) = file.write_all(&contents).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    Ok(export.sample_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn test_history() -> History {
        History {
            sampled_at_unix_ms: VecDeque::from([Some(1_700_000_000_000), Some(1_700_000_001_000)]),
            cpu: VecDeque::from([15.5, 17.0]),
            memory: VecDeque::from([50.0, 51.0]),
            received: VecDeque::from([100.0, 200.0]),
            sent: VecDeque::from([300.0, 400.0]),
            disk_read: VecDeque::from([500.0, 600.0]),
            disk_write: VecDeque::from([700.0, 800.0]),
            psi_ticks: VecDeque::from([[Some(5.0), None, None, None, None], [None; 5]]),
            ..History::default()
        }
    }

    #[test]
    fn csv_and_json_preserve_sample_alignment_and_missing_values() {
        let data = rows(&test_history(), 1000).unwrap();
        let csv = String::from_utf8(render_csv(&data)).unwrap();
        assert_eq!(csv.lines().count(), 3);
        assert!(csv.contains("linux-sampler-visible-system,1000,1700000000000,15.5,50"));
        assert!(csv.lines().last().unwrap().ends_with(",,,,,"));
        let json: serde_json::Value = serde_json::to_value(&data).unwrap();
        assert_eq!(json["sample_count"], 2);
        assert_eq!(json["samples"][0]["psi_cpu_some_avg10_percent"], 5.0);
        assert!(json["samples"][1]["psi_cpu_some_avg10_percent"].is_null());
    }

    #[test]
    fn unaligned_or_empty_data_is_not_exported() {
        assert!(rows(&History::default(), 1000).is_err());
        let mut history = test_history();
        history.psi_ticks.pop_back();
        assert!(rows(&history, 1000).is_err());
    }

    #[test]
    fn explicit_new_file_prevents_overwrite_and_requires_supported_format() {
        let path = std::env::temp_dir().join(format!(
            "taskotto-test-export-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let count = write_new(&test_history(), &path, 1000).unwrap();
        assert_eq!(count, 2);
        assert!(write_new(&test_history(), &path, 1000).is_err());
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("linux-sampler-visible-system"));
        assert!(!raw.contains("process_name"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        std::fs::remove_file(&path).unwrap();
        assert!(write_new(&test_history(), Path::new("relative.csv"), 1000).is_err());
        assert!(write_new(&test_history(), Path::new("/tmp/unsupported.txt"), 1000).is_err());
    }
}
