use chrono::NaiveDateTime;
use clap::Parser;

#[derive(Parser)]
#[command(name = "logscope")]
#[command(about = "A lightweight CLI tool for parsing and analyzing log files", long_about = None)]
pub struct Cli {
    #[arg(help = "Path to the log file to analyze")]
    pub file_path: String,

    #[arg(short, long, help = "Filter logs by keyword")]
    pub keyword: Option<String>,

    #[arg(long, help = "Filter logs from this timestamp (format: YYYY-MM-DD HH:MM:SS)", value_parser = parse_datetime)]
    pub from: Option<NaiveDateTime>,

    #[arg(long, help = "Filter logs until this timestamp (format: YYYY-MM-DD HH:MM:SS)", value_parser = parse_datetime)]
    pub to: Option<NaiveDateTime>,
}

fn parse_datetime(s: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| format!("Invalid datetime format: {}", e))
}