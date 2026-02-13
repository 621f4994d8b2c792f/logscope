use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

use crate::parser::{LogEntry, LogLevel};
use crate::stats::{self, Stats};

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "with", "from", "that", "this", "have", "has",
    "been", "was", "were", "are", "will", "would", "could", "should",
    "not", "but", "can", "into", "its", "just", "when", "then", "also",
    "than", "more", "some", "over", "such", "after", "before", "while",
];

#[derive(Debug, Serialize)]
pub struct KeywordEntry {
    pub word: String,
    pub count: usize,
    pub error_ratio: f64,
}

#[derive(Debug, Serialize)]
pub struct LogAnalysis {
    pub stats: Stats,
    pub level_counts: HashMap<String, usize>,
    pub top_keywords: Vec<KeywordEntry>,
    pub anomaly_score: f64,
    pub unparsed_lines: usize,
}

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    unparsed_lines: usize,
}

impl LogAnalyzer {
    pub fn new(entries: Vec<LogEntry>, unparsed_lines: usize) -> Self {
        Self { entries, unparsed_lines }
    }

    pub fn analyze(self, top_n: usize) -> LogAnalysis {
        let stats = stats::compute(&self.entries);
        let level_counts = count_by_level(&self.entries);
        let top_keywords = extract_keywords(&self.entries, top_n);
        let anomaly_score = compute_anomaly_score(&stats, &level_counts);

        LogAnalysis {
            stats,
            level_counts,
            top_keywords,
            anomaly_score,
            unparsed_lines: self.unparsed_lines,
        }
    }
}

fn count_by_level(entries: &[LogEntry]) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        *counts.entry(entry.level.as_str().to_string()).or_insert(0) += 1;
    }
    counts
}

fn extract_keywords(entries: &[LogEntry], limit: usize) -> Vec<KeywordEntry> {
    // parallel word count per level
    let (total_counts, error_counts): (HashMap<String, usize>, HashMap<String, usize>) = entries
        .par_iter()
        .map(|entry| {
            let mut total: HashMap<String, usize> = HashMap::new();
            let mut errors: HashMap<String, usize> = HashMap::new();
            let is_error = matches!(entry.level, LogLevel::Error | LogLevel::Fatal);

            for word in entry.message.split_whitespace() {
                let clean = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();

                if clean.len() < 3 || STOPWORDS.contains(&clean.as_str()) {
                    continue;
                }

                *total.entry(clean.clone()).or_insert(0) += 1;
                if is_error {
                    *errors.entry(clean).or_insert(0) += 1;
                }
            }

            (total, errors)
        })
        .reduce(
            || (HashMap::new(), HashMap::new()),
            |(mut acc_t, mut acc_e), (t, e)| {
                for (k, v) in t {
                    *acc_t.entry(k).or_insert(0) += v;
                }
                for (k, v) in e {
                    *acc_e.entry(k).or_insert(0) += v;
                }
                (acc_t, acc_e)
            },
        );

    let mut result: Vec<KeywordEntry> = total_counts
        .into_iter()
        .map(|(word, count)| {
            let err_count = *error_counts.get(&word).unwrap_or(&0);
            let error_ratio = if count > 0 {
                err_count as f64 / count as f64
            } else {
                0.0
            };
            KeywordEntry { word, count, error_ratio }
        })
        .collect();

    result.sort_unstable_by(|a, b| b.count.cmp(&a.count).then(b.error_ratio.partial_cmp(&a.error_ratio).unwrap()));
    result.truncate(limit);
    result
}

fn compute_anomaly_score(stats: &Stats, level_counts: &HashMap<String, usize>) -> f64 {
    let mut score = 0.0_f64;

    // error rate weight
    score += stats.error_rate * 0.4;

    // burst penalty
    score += stats.error_bursts.len() as f64 * 5.0;

    // fatal presence
    if *level_counts.get("FATAL").unwrap_or(&0) > 0 {
        score += 20.0;
    }

    // MTBF: shorter = worse
    if let Some(mtbf) = stats.mtbf_seconds {
        if mtbf < 60.0 {
            score += 15.0;
        } else if mtbf < 300.0 {
            score += 8.0;
        }
    }

    score.min(100.0)
}
