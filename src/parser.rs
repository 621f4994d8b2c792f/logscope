use chrono::NaiveDateTime;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: NaiveDateTime,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Unknown,
}

impl LogLevel {
    fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "INFO" => LogLevel::Info,
            "WARN" | "WARNING" => LogLevel::Warn,
            "ERROR" | "ERR" => LogLevel::Error,
            _ => LogLevel::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Unknown => "UNKNOWN",
        }
    }
}

pub struct LogParser {
    pattern: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        let pattern = Regex::new(r"\[(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})\] (\w+) (.+)")
            .expect("Invalid regex pattern");

        LogParser { pattern }
    }

    pub fn parse_file(&self, file_path: &str) -> Result<Vec<LogEntry>, std::io::Error> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Some(entry) = self.parse_line(&line) {
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    fn parse_line(&self, line: &str) -> Option<LogEntry> {
        let captures = self.pattern.captures(line)?;

        let timestamp_str = captures.get(1)?.as_str();
        let timestamp = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S").ok()?;

        let level_str = captures.get(2)?.as_str();
        let level = LogLevel::from_str(level_str);

        let message = captures.get(3)?.as_str().to_string();

        Some(LogEntry {
            timestamp,
            level,
            message,
        })
    }
}