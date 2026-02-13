use colored::Colorize;

use crate::analyzer::LogAnalysis;
use crate::parser::LogLevel;

pub struct ReportGenerator {
    color: bool,
}

impl ReportGenerator {
    pub fn new(color: bool) -> Self {
        Self { color }
    }

    pub fn generate(&self, file_path: &str, analysis: &LogAnalysis, show_heatmap: bool) {
        self.print_header(file_path, analysis);
        self.print_level_distribution(analysis);
        self.print_stats(analysis);
        self.print_top_keywords(analysis);

        if !analysis.stats.error_bursts.is_empty() {
            self.print_bursts(analysis);
        }

        if show_heatmap {
            self.print_heatmap(analysis);
        }

        self.print_anomaly_score(analysis);
    }

    fn print_header(&self, file_path: &str, analysis: &LogAnalysis) {
        let title = "logscope — Log Analysis Report";
        if self.color {
            println!("\n{}", title.bold().cyan());
        } else {
            println!("\n{}", title);
        }
        println!("{}", "─".repeat(50));

        println!("File    : {}", file_path);
        println!("Entries : {}", analysis.stats.total);

        if analysis.unparsed_lines > 0 {
            let msg = format!("Skipped : {} unparsed lines", analysis.unparsed_lines);
            if self.color {
                println!("{}", msg.yellow());
            } else {
                println!("{}", msg);
            }
        }

        if let Some(ref t) = analysis.stats.time {
            println!("Range   : {} → {}", t.start, t.end);
            println!("Span    : {}", t.span_human);
        }

        println!("Rate    : {:.1} entries/min\n", analysis.stats.rate_per_minute);
    }

    fn print_level_distribution(&self, analysis: &LogAnalysis) {
        println!("{}", "Log Level Distribution");
        println!("{}", "─".repeat(30));

        let levels = [
            LogLevel::Fatal,
            LogLevel::Error,
            LogLevel::Warn,
            LogLevel::Info,
            LogLevel::Debug,
        ];

        for level in &levels {
            let key = level.as_str();
            let count = *analysis.level_counts.get(key).unwrap_or(&0);
            if count == 0 {
                continue;
            }

            let pct = count as f64 / analysis.stats.total as f64 * 100.0;
            let bar_len = (pct / 2.0) as usize;
            let bar = "█".repeat(bar_len);

            let label = format!("  {:<7} {:>5}  ({:5.1}%)  {}", key, count, pct, bar);

            if self.color {
                let colored = match level {
                    LogLevel::Fatal => label.red().bold().to_string(),
                    LogLevel::Error => label.red().to_string(),
                    LogLevel::Warn => label.yellow().to_string(),
                    LogLevel::Info => label.green().to_string(),
                    LogLevel::Debug => label.dimmed().to_string(),
                    LogLevel::Unknown => label,
                };
                println!("{}", colored);
            } else {
                println!("{}", label);
            }
        }

        println!();
    }

    fn print_stats(&self, analysis: &LogAnalysis) {
        println!("{}", "Statistics");
        println!("{}", "─".repeat(30));
        println!("  Error rate  : {:.1}%", analysis.stats.error_rate);

        if let Some(mtbf) = analysis.stats.mtbf_seconds {
            let formatted = format_duration(mtbf as i64);
            println!("  MTBF errors : {}", formatted);
        }

        if let Some(peak) = analysis.stats.peak_hour {
            println!("  Peak hour   : {:02}:00 – {:02}:59", peak, peak);
        }

        println!();
    }

    fn print_top_keywords(&self, analysis: &LogAnalysis) {
        if analysis.top_keywords.is_empty() {
            return;
        }

        println!("{}", "Top Keywords");
        println!("{}", "─".repeat(30));

        for (i, kw) in analysis.top_keywords.iter().enumerate() {
            let ratio_bar = if kw.error_ratio > 0.0 {
                format!("  [{:.0}% in errors]", kw.error_ratio * 100.0)
            } else {
                String::new()
            };

            let line = format!(
                "  {:>2}. {:>15}  ×{:<6}{}",
                i + 1,
                kw.word,
                kw.count,
                ratio_bar,
            );

            if self.color && kw.error_ratio > 0.5 {
                println!("{}", line.red());
            } else {
                println!("{}", line);
            }
        }

        println!();
    }

    fn print_bursts(&self, analysis: &LogAnalysis) {
        let header = format!("Error Bursts Detected ({})", analysis.stats.error_bursts.len());
        if self.color {
            println!("{}", header.red().bold());
        } else {
            println!("{}", header);
        }
        println!("{}", "─".repeat(30));

        for burst in &analysis.stats.error_bursts {
            println!("  {} — {} errors in 60s", burst.window_start, burst.count);
        }

        println!();
    }

    fn print_heatmap(&self, analysis: &LogAnalysis) {
        println!("{}", "Hourly Activity Heatmap");
        println!("{}", "─".repeat(50));

        let max = *analysis.stats.hourly_counts.iter().max().unwrap_or(&1).max(&1);

        for hour in 0..24_usize {
            let count = analysis.stats.hourly_counts[hour];
            let bar_len = count * 40 / max;
            let bar = "▪".repeat(bar_len);
            println!("  {:02}h │{:<40}│ {}", hour, bar, count);
        }

        println!();
    }

    fn print_anomaly_score(&self, analysis: &LogAnalysis) {
        let score = analysis.anomaly_score;
        let label = match score as u32 {
            0..=20 => "Healthy",
            21..=50 => "Moderate",
            51..=79 => "Elevated",
            _ => "Critical",
        };

        let line = format!("Anomaly Score: {:.1} / 100  [{}]", score, label);

        println!("{}", "─".repeat(50));
        if self.color {
            let colored = match score as u32 {
                0..=20 => line.green().bold().to_string(),
                21..=50 => line.yellow().bold().to_string(),
                _ => line.red().bold().to_string(),
            };
            println!("{}\n", colored);
        } else {
            println!("{}\n", line);
        }
    }
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
