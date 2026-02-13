use chrono::NaiveDateTime;
use regex::Regex;

use crate::parser::{LogEntry, LogLevel};

pub struct FilterConfig {
    pub keyword: Option<String>,
    pub keyword_regex: Option<Regex>,
    pub from: Option<NaiveDateTime>,
    pub to: Option<NaiveDateTime>,
    pub min_level: Option<u8>,
    pub source: Option<String>,
}

impl FilterConfig {
    pub fn new() -> Self {
        Self {
            keyword: None,
            keyword_regex: None,
            from: None,
            to: None,
            min_level: None,
            source: None,
        }
    }

    pub fn with_keyword(mut self, kw: String) -> Self {
        if let Ok(re) = Regex::new(&format!("(?i){}", regex::escape(&kw))) {
            self.keyword_regex = Some(re);
        }
        self.keyword = Some(kw);
        self
    }

    pub fn with_time_range(mut self, from: Option<NaiveDateTime>, to: Option<NaiveDateTime>) -> Self {
        self.from = from;
        self.to = to;
        self
    }

    pub fn with_min_level(mut self, level: &LogLevel) -> Self {
        self.min_level = Some(level.severity());
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.keyword.is_none()
            && self.from.is_none()
            && self.to.is_none()
            && self.min_level.is_none()
            && self.source.is_none()
    }
}

pub fn apply(entries: Vec<LogEntry>, config: &FilterConfig) -> Vec<LogEntry> {
    if config.is_empty() {
        return entries;
    }

    entries
        .into_iter()
        .filter(|entry| matches_all(entry, config))
        .collect()
}

fn matches_all(entry: &LogEntry, config: &FilterConfig) -> bool {
    if let Some(re) = &config.keyword_regex {
        if !re.is_match(&entry.message) {
            return false;
        }
    }

    if let Some(from) = &config.from {
        if entry.timestamp < *from {
            return false;
        }
    }

    if let Some(to) = &config.to {
        if entry.timestamp > *to {
            return false;
        }
    }

    if let Some(min_sev) = config.min_level {
        if entry.level.severity() < min_sev {
            return false;
        }
    }

    if let Some(src) = &config.source {
        match &entry.source {
            Some(s) => {
                if !s.to_lowercase().contains(&src.to_lowercase()) {
                    return false;
                }
            }
            None => return false,
        }
    }

    true
}
