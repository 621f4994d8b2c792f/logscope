use serde_json;
use std::fs::File;
use std::io::{BufWriter, Write};

use crate::analyzer::LogAnalysis;
use crate::parser::LogEntry;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

pub fn export_analysis(
    analysis: &LogAnalysis,
    entries: &[LogEntry],
    format: ExportFormat,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match format {
        ExportFormat::Json => export_json(analysis, entries, output_path),
        ExportFormat::Csv => export_csv(entries, output_path),
    }
}

fn export_json(
    analysis: &LogAnalysis,
    _entries: &[LogEntry],
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, analysis)?;
    Ok(())
}

fn export_csv(
    entries: &[LogEntry],
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "timestamp,level,source,message")?;

    for entry in entries {
        let source = entry.source.as_deref().unwrap_or("");
        let msg = entry.message.replace('"', "\"\"");
        writeln!(
            writer,
            "{},{},{},\"{}\"",
            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
            entry.level.as_str(),
            source,
            msg,
        )?;
    }

    Ok(())
}
