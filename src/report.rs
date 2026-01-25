use crate::analyzer::LogAnalysis;
use crate::parser::LogLevel;

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn new() -> Self {
        ReportGenerator
    }

    pub fn generate_report(&self, file_path: &str, analysis: &LogAnalysis) {
        println!("=== Log Analysis Report ===\n");
        println!("File: {}", file_path);
        println!("Total Lines: {}\n", analysis.total_lines);

        if let Some((start, end)) = &analysis.time_range {
            println!("Time Range: {} to {}\n", start, end);
        }

        self.print_level_distribution(analysis);
        self.print_top_keywords(analysis);
    }

    fn print_level_distribution(&self, analysis: &LogAnalysis) {
        println!("Log Level Distribution:");

        let levels = [LogLevel::Info, LogLevel::Warn, LogLevel::Error];

        for level in &levels {
            if let Some(&count) = analysis.level_counts.get(level) {
                let percentage = (count as f64 / analysis.total_lines as f64) * 100.0;
                println!(
                    "  {}: {} ({:.1}%)",
                    level.as_str(),
                    count,
                    percentage
                );
            }
        }

        println!();
    }

    fn print_top_keywords(&self, analysis: &LogAnalysis) {
        if analysis.top_keywords.is_empty() {
            return;
        }

        println!("Top Keywords:");

        for (index, (keyword, count)) in analysis.top_keywords.iter().enumerate() {
            println!("  {}. \"{}\" - {} occurrences", index + 1, keyword, count);
        }

        println!();
    }
}
