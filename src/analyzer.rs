use crate::parser::{LogEntry, LogLevel};
use std::collections::HashMap;

pub struct LogAnalysis {
    pub total_lines: usize,
    pub level_counts: HashMap<LogLevel, usize>,
    pub top_keywords: Vec<(String, usize)>,
    pub time_range: Option<(String, String)>,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new(entries: Vec<LogEntry>) -> Self {
        LogAnalyzer { entries }
    }

    pub fn analyze(&self) -> LogAnalysis {
        let total_lines = self.entries.len();
        let level_counts = self.count_by_level();
        let top_keywords = self.extract_top_keywords(5);
        let time_range = self.calculate_time_range();

        LogAnalysis {
            total_lines,
            level_counts,
            top_keywords,
            time_range,
        }
    }

    fn count_by_level(&self) -> HashMap<LogLevel, usize> {
        let mut counts = HashMap::new();

        for entry in &self.entries {
            *counts.entry(entry.level.clone()).or_insert(0) += 1;
        }

        counts
    }

    fn extract_top_keywords(&self, limit: usize) -> Vec<(String, usize)> {
        let stopwords = self.build_stopwords();
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        for entry in &self.entries {
            let words: Vec<&str> = entry.message.split_whitespace().collect();

            for word in words {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();

                if clean_word.len() > 3 && !stopwords.contains(clean_word.as_str()) {
                    *word_counts.entry(clean_word).or_insert(0) += 1;
                }
            }
        }

        let mut sorted: Vec<_> = word_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(limit);

        sorted
    }

    fn build_stopwords(&self) -> Vec<&str> {
        vec![
            "the", "and", "for", "with", "from", "that", "this", "have", "has",
            "been", "was", "were", "are", "will", "would", "could", "should",
        ]
    }

    fn calculate_time_range(&self) -> Option<(String, String)> {
        if self.entries.is_empty() {
            return None;
        }

        let first = &self.entries[0].timestamp;
        let last = &self.entries[self.entries.len() - 1].timestamp;

        Some((
            first.format("%Y-%m-%d %H:%M:%S").to_string(),
            last.format("%Y-%m-%d %H:%M:%S").to_string(),
        ))
    }
}
