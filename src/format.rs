pub fn bytes(value: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut number = value as f64;
    let mut unit = 0;
    while number >= 1024.0 && unit < UNITS.len() - 1 {
        number /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{value} B")
    } else {
        format!("{number:.1} {}", UNITS[unit])
    }
}

pub fn rate(value: u64) -> String {
    format!("{}/s", bytes(value))
}

pub fn duration(seconds: f64) -> String {
    let seconds = seconds.max(0.0) as u64;
    let days = seconds / 86_400;
    let hours = (seconds / 3_600) % 24;
    let minutes = (seconds / 60) % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else {
        format!("{hours}h {minutes}m")
    }
}

pub fn process_age(seconds: f64) -> String {
    if seconds < 60.0 {
        format!("{}s", seconds.max(0.0) as u64)
    } else {
        duration(seconds)
    }
}

pub fn local_timestamp(seconds: u64) -> String {
    if seconds == 0 {
        return "Unknown".to_string();
    }
    let Ok(value) = libc::time_t::try_from(seconds) else {
        return "Unknown".to_string();
    };
    // SAFETY: libc::tm is an output-only C struct. localtime_r receives
    // valid pointers to stack-allocated values and writes the complete tm.
    let mut broken_down: libc::tm = unsafe { std::mem::zeroed() };
    let pointer = unsafe { libc::localtime_r(&value, &mut broken_down) };
    if pointer.is_null() {
        return "Unknown".to_string();
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        broken_down.tm_year + 1900,
        broken_down.tm_mon + 1,
        broken_down.tm_mday,
        broken_down.tm_hour,
        broken_down.tm_min,
        broken_down.tm_sec
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_binary_units() {
        assert_eq!(bytes(0), "0 B");
        assert_eq!(bytes(1024), "1.0 KiB");
        assert_eq!(rate(1_048_576), "1.0 MiB/s");
    }

    #[test]
    fn formats_uptime() {
        assert_eq!(duration(90_000.0), "1d 1h 0m");
        assert_eq!(duration(-5.0), "0h 0m");
    }

    #[test]
    fn formats_process_start_and_short_duration() {
        assert_eq!(process_age(12.5), "12s");
        assert_eq!(process_age(-1.0), "0s");
        assert_eq!(local_timestamp(0), "Unknown");
        assert!(local_timestamp(1_700_000_000).starts_with("2023-"));
    }
}
