use chrono::NaiveDateTime;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
    Unknown,
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DEBUG" | "DBG" | "TRACE" => Self::Debug,
            "INFO" | "INFORMATION" => Self::Info,
            "WARN" | "WARNING" => Self::Warn,
            "ERROR" | "ERR" => Self::Error,
            "FATAL" | "CRITICAL" | "CRIT" => Self::Fatal,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Fatal => "FATAL",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub fn severity(&self) -> u8 {
        match self {
            Self::Debug => 0,
            Self::Info => 1,
            Self::Warn => 2,
            Self::Error => 3,
            Self::Fatal => 4,
            Self::Unknown => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: NaiveDateTime,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub line_number: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogFormat {
    Bracket,   // [2026-01-01 12:00:00] LEVEL message
    Syslog,    // Jan  1 12:00:00 host process[pid]: message
    Json,      // {"timestamp":"...","level":"...","message":"..."}
    Apache,    // 127.0.0.1 - - [01/Jan/2026:12:00:00 +0000] "GET / HTTP/1.1" 200 1234
    Auto,
}

pub struct LogParser {
    format: LogFormat,
    bracket_re: Regex,
    syslog_re: Regex,
    apache_re: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        Self::with_format(LogFormat::Auto)
    }

    pub fn with_format(format: LogFormat) -> Self {
        Self {
            format,
            bracket_re: Regex::new(
                r"^\[(\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2})\]\s+(\w+)\s+(.+)$",
            )
            .unwrap(),
            syslog_re: Regex::new(
                r"^(\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2})\s+\S+\s+(\S+?)(?:\[\d+\])?:\s+(.+)$",
            )
            .unwrap(),
            apache_re: Regex::new(
                r#"^\S+\s+\S+\s+\S+\s+\[([^\]]+)\]\s+"[^"]*"\s+(\d{3})\s+\S+"#,
            )
            .unwrap(),
        }
    }

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<LogEntry>, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let lines: Vec<(usize, String)> = reader
            .lines()
            .enumerate()
            .filter_map(|(i, l)| l.ok().map(|s| (i + 1, s)))
            .collect();

        let entries: Vec<LogEntry> = lines
            .par_iter()
            .filter_map(|(line_num, line)| self.parse_line(line, *line_num))
            .collect();

        let mut sorted = entries;
        sorted.sort_unstable_by_key(|e| e.timestamp);

        Ok(sorted)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        match self.format {
            LogFormat::Bracket => self.parse_bracket(line, line_number),
            LogFormat::Syslog => self.parse_syslog(line, line_number),
            LogFormat::Json => self.parse_json(line, line_number),
            LogFormat::Apache => self.parse_apache(line, line_number),
            LogFormat::Auto => self
                .parse_bracket(line, line_number)
                .or_else(|| self.parse_json(line, line_number))
                .or_else(|| self.parse_apache(line, line_number))
                .or_else(|| self.parse_syslog(line, line_number)),
        }
    }

    fn parse_bracket(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = self.bracket_re.captures(line)?;
        let ts_str = caps.get(1)?.as_str().replace('T', " ");
        let timestamp = NaiveDateTime::parse_from_str(&ts_str, "%Y-%m-%d %H:%M:%S").ok()?;
        let level = LogLevel::from_str(caps.get(2)?.as_str());
        let message = caps.get(3)?.as_str().to_string();

        Some(LogEntry { timestamp, level, message, source: None, line_number })
    }

    fn parse_json(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let v: serde_json::Value = serde_json::from_str(line).ok()?;
        let obj = v.as_object()?;

        let ts_str = obj.get("timestamp")
            .or_else(|| obj.get("time"))
            .or_else(|| obj.get("@timestamp"))
            .and_then(|v| v.as_str())?;

        let timestamp = NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%dT%H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S"))
            .ok()?;

        let level_str = obj.get("level")
            .or_else(|| obj.get("severity"))
            .or_else(|| obj.get("lvl"))
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        let message = obj.get("message")
            .or_else(|| obj.get("msg"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let source = obj.get("logger")
            .or_else(|| obj.get("source"))
            .or_else(|| obj.get("service"))
            .and_then(|v| v.as_str())
            .map(String::from);

        Some(LogEntry {
            timestamp,
            level: LogLevel::from_str(level_str),
            message,
            source,
            line_number,
        })
    }

    fn parse_apache(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = self.apache_re.captures(line)?;
        let ts_str = caps.get(1)?.as_str();
        let timestamp = NaiveDateTime::parse_from_str(ts_str, "%d/%b/%Y:%H:%M:%S %z")
            .or_else(|_| NaiveDateTime::parse_from_str(ts_str, "%d/%b/%Y:%H:%M:%S +0000"))
            .ok()?;

        let status: u16 = caps.get(2)?.as_str().parse().ok()?;
        let level = match status {
            200..=399 => LogLevel::Info,
            400..=499 => LogLevel::Warn,
            500..=599 => LogLevel::Error,
            _ => LogLevel::Unknown,
        };

        Some(LogEntry {
            timestamp,
            level,
            message: line.to_string(),
            source: Some("apache".into()),
            line_number,
        })
    }

    fn parse_syslog(&self, line: &str, line_number: usize) -> Option<LogEntry> {
        let caps = self.syslog_re.captures(line)?;
        let ts_str = caps.get(1)?.as_str();

        let current_year = chrono::Local::now().format("%Y").to_string();
        let full_ts = format!("{} {}", current_year, ts_str);

        let timestamp = NaiveDateTime::parse_from_str(&full_ts, "%Y %b %e %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&full_ts, "%Y %b %d %H:%M:%S"))
            .ok()?;

        let source = Some(caps.get(2)?.as_str().to_string());
        let message = caps.get(3)?.as_str().to_string();

        let level = if message.to_lowercase().contains("error") || message.to_lowercase().contains("fail") {
            LogLevel::Error
        } else if message.to_lowercase().contains("warn") {
            LogLevel::Warn
        } else {
            LogLevel::Info
        };

        Some(LogEntry { timestamp, level, message, source, line_number })
    }
}

impl LogParser {
    pub fn parse_file_counted(
        &self,
        file_path: &str,
    ) -> Result<(Vec<LogEntry>, usize), std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);

        let lines: Vec<(usize, String)> = reader
            .lines()
            .enumerate()
            .filter_map(|(i, l)| l.ok().map(|s| (i + 1, s)))
            .collect();

        let total_non_empty = lines
            .iter()
            .filter(|(_, l)| !l.trim().is_empty())
            .count();

        let entries: Vec<LogEntry> = lines
            .par_iter()
            .filter_map(|(num, line)| self.parse_line(line, *num))
            .collect();

        let mut sorted = entries;
        sorted.sort_unstable_by_key(|e| e.timestamp);

        let unparsed = total_non_empty.saturating_sub(sorted.len());

        Ok((sorted, unparsed))
    }
}
