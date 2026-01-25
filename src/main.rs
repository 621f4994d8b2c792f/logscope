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
    if let Some(ref keyword) = args.keyword {
        entries.retain(|entry| entry.message.contains(keyword));
    }

    if let Some(ref from) = args.from {
        entries.retain(|entry| entry.timestamp >= *from);
    }

    if let Some(ref to) = args.to {
        entries.retain(|entry| entry.timestamp <= *to);
    }

    entries
}
