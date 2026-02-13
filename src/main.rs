use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::process;
use std::time::Duration;

mod analyzer;
mod cli;
mod export;
mod filter;
mod parser;
mod report;
mod stats;

use analyzer::LogAnalyzer;
use cli::Cli;
use export::{export_analysis, ExportFormat};
use filter::FilterConfig;
use parser::{LogFormat, LogParser, LogLevel};
use report::ReportGenerator;

fn main() {
    let args = Cli::parse();

    if args.no_color {
        colored::control::set_override(false);
    }

    let format = resolve_format(args.format.as_deref());
    let parser = LogParser::with_format(format);

    let spinner = build_spinner("Parsing log file…");

    let (entries, unparsed) = match parser.parse_file_counted(&args.file_path) {
        Ok(result) => result,
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    spinner.finish_and_clear();

    let filter_cfg = build_filter(&args);
    let filtered = filter::apply(entries.clone(), &filter_cfg);

    if filtered.is_empty() {
        eprintln!("No entries matched the given filters.");
        process::exit(0);
    }

    let analyzer = LogAnalyzer::new(filtered, unparsed);
    let analysis = analyzer.analyze(args.top);

    let reporter = ReportGenerator::new(!args.no_color);
    reporter.generate(&args.file_path, &analysis, args.heatmap);

    if let (Some(fmt_str), Some(out_path)) = (&args.output_format, &args.output) {
        match ExportFormat::from_str(fmt_str) {
            Some(fmt) => {
                match export_analysis(&analysis, &entries, fmt, out_path) {
                    Ok(()) => println!("Exported to {}", out_path),
                    Err(e) => eprintln!("Export error: {}", e),
                }
            }
            None => eprintln!("Unknown export format: {}", fmt_str),
        }
    }
}

fn resolve_format(s: Option<&str>) -> LogFormat {
    match s {
        Some("bracket") => LogFormat::Bracket,
        Some("json") => LogFormat::Json,
        Some("apache") => LogFormat::Apache,
        Some("syslog") => LogFormat::Syslog,
        _ => LogFormat::Auto,
    }
}

fn build_filter(args: &Cli) -> FilterConfig {
    let mut cfg = FilterConfig::new();

    if let Some(ref kw) = args.keyword {
        cfg = cfg.with_keyword(kw.clone());
    }

    cfg = cfg.with_time_range(args.from, args.to);

    if let Some(ref level_str) = args.level {
        let level = LogLevel::from_str(level_str);
        cfg = cfg.with_min_level(&level);
    }

    if let Some(ref src) = args.source {
        cfg = cfg.with_source(src.clone());
    }

    cfg
}

fn build_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}
