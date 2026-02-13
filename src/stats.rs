use chrono::Timelike;
use serde::Serialize;

use crate::parser::{LogEntry, LogLevel};

#[derive(Debug, Serialize)]
pub struct TimeStats {
    pub start: String,
    pub end: String,
    pub span_seconds: i64,
    pub span_human: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorBurst {
    pub window_start: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct Stats {
    pub total: usize,
    pub time: Option<TimeStats>,
    pub rate_per_minute: f64,
    pub peak_hour: Option<u32>,
    pub hourly_counts: [usize; 24],
    pub error_rate: f64,
    pub error_bursts: Vec<ErrorBurst>,
    pub mtbf_seconds: Option<f64>,
}

pub fn compute(entries: &[LogEntry]) -> Stats {
    let total = entries.len();

    if total == 0 {
        return Stats {
            total: 0,
            time: None,
            rate_per_minute: 0.0,
            peak_hour: None,
            hourly_counts: [0; 24],
            error_rate: 0.0,
            error_bursts: vec![],
            mtbf_seconds: None,
        };
    }

    let first = &entries[0].timestamp;
    let last = &entries[total - 1].timestamp;
    let span_seconds = (*last - *first).num_seconds().max(1);

    let time = Some(TimeStats {
        start: first.format("%Y-%m-%d %H:%M:%S").to_string(),
        end: last.format("%Y-%m-%d %H:%M:%S").to_string(),
        span_seconds,
        span_human: format_duration(span_seconds),
    });

    let rate_per_minute = total as f64 / (span_seconds as f64 / 60.0);

    let mut hourly_counts = [0usize; 24];
    for entry in entries {
        hourly_counts[entry.timestamp.hour() as usize] += 1;
    }

    let peak_hour = hourly_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &c)| c)
        .map(|(h, _)| h as u32);

    let error_count = entries
        .iter()
        .filter(|e| matches!(e.level, LogLevel::Error | LogLevel::Fatal))
        .count();
    let error_rate = error_count as f64 / total as f64 * 100.0;

    let error_bursts = detect_bursts(entries);
    let mtbf_seconds = compute_mtbf(entries, span_seconds);

    Stats {
        total,
        time,
        rate_per_minute,
        peak_hour,
        hourly_counts,
        error_rate,
        error_bursts,
        mtbf_seconds,
    }
}

fn detect_bursts(entries: &[LogEntry]) -> Vec<ErrorBurst> {
    // sliding 60-second window, burst threshold = 3 errors
    const WINDOW_SECS: i64 = 60;
    const BURST_THRESHOLD: usize = 3;

    let mut bursts = Vec::new();
    let errors: Vec<&LogEntry> = entries
        .iter()
        .filter(|e| matches!(e.level, LogLevel::Error | LogLevel::Fatal))
        .collect();

    let mut i = 0;
    while i < errors.len() {
        let window_end = errors[i].timestamp + chrono::Duration::seconds(WINDOW_SECS);
        let count = errors[i..]
            .iter()
            .take_while(|e| e.timestamp <= window_end)
            .count();

        if count >= BURST_THRESHOLD {
            bursts.push(ErrorBurst {
                window_start: errors[i].timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                count,
            });
            i += count;
        } else {
            i += 1;
        }
    }

    bursts
}

fn compute_mtbf(entries: &[LogEntry], span_seconds: i64) -> Option<f64> {
    let error_count = entries
        .iter()
        .filter(|e| matches!(e.level, LogLevel::Error | LogLevel::Fatal))
        .count();

    if error_count < 2 {
        return None;
    }

    Some(span_seconds as f64 / (error_count - 1) as f64)
}

fn format_duration(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;

    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
