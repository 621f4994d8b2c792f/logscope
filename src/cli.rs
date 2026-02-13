use chrono::NaiveDateTime;
use clap::Parser;

#[derive(Parser)]
#[command(name = "logscope")]
#[command(version = "0.2.0")]
#[command(about = "Parse and analyze log files with detailed statistics")]
pub struct Cli {
    #[arg(help = "Path to the log file")]
    pub file_path: String,

    #[arg(short, long, help = "Filter by keyword (supports regex)")]
    pub keyword: Option<String>,

    #[arg(long, value_parser = parse_datetime, help = "Start time (YYYY-MM-DD HH:MM:SS)")]
    pub from: Option<NaiveDateTime>,

    #[arg(long, value_parser = parse_datetime, help = "End time (YYYY-MM-DD HH:MM:SS)")]
    pub to: Option<NaiveDateTime>,

    #[arg(long, help = "Minimum log level (debug/info/warn/error/fatal)")]
    pub level: Option<String>,

    #[arg(long, help = "Filter by source/logger name")]
    pub source: Option<String>,

    #[arg(long, default_value = "10", help = "Number of top keywords to show")]
    pub top: usize,

    #[arg(long, help = "Force log format (bracket/json/apache/syslog)")]
    pub format: Option<String>,

    #[arg(long, help = "Export results: json or csv")]
    pub output_format: Option<String>,

    #[arg(long, help = "Output file path for export")]
    pub output: Option<String>,

    #[arg(long, help = "Disable colored output")]
    pub no_color: bool,

    #[arg(long, help = "Show hourly activity heatmap")]
    pub heatmap: bool,
}

fn parse_datetime(s: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("Invalid datetime: {}", e))
}
