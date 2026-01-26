use clap::Parser;
use std::process;

mod analyzer;
mod cli;
mod parser;
mod report;

use analyzer::LogAnalyzer;
use cli::Cli;
use parser::LogParser;
use report::ReportGenerator;

fn main() {
    let args = Cli::parse();

    let parser = LogParser::new();
    let entries = match parser.parse_file(&args.file_path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Error parsing log file: {}", e);
            process::exit(1);
        }
    };

    let filtered_entries = apply_filters(entries, &args);

    let analyzer = LogAnalyzer::new(filtered_entries);
    let analysis = analyzer.analyze();

    let generator = ReportGenerator::new();
    generator.generate_report(&args.file_path, &analysis);
}

fn apply_filters(mut entries: Vec<parser::LogEntry>, args: &Cli) -> Vec<parser::LogEntry> {
    let has_keyword = args.keyword.is_some();
    let has_from = args.from.is_some();
    let has_to = args.to.is_some();

    if !has_keyword && !has_from && !has_to {
        return entries;
    }

    entries.retain(|entry| {
        if let Some(ref keyword) = args.keyword {
            if !entry.message.contains(keyword) {
                return false;
            }
        }

        if let Some(ref from) = args.from {
            if entry.timestamp < *from {
                return false;
            }
        }

        if let Some(ref to) = args.to {
            if entry.timestamp > *to {
                return false;
            }
        }

        true
    });

    entries
}
