# logscope

A lightweight CLI tool for parsing and analyzing log files with detailed statistics and insights.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)


## Features

logscope helps you make sense of your log files by providing:

- Parse standard log formats with timestamps and severity levels
- Filter logs by keywords and time ranges
- Generate statistical summaries by log level
- Identify top occurring keywords and patterns
- Calculate activity time spans
- Export analysis results in clean, readable tables

## Installation

Clone the repository and build from source:

```bash
git clone https://github.com/621f4994d8b2c792f/logscope.git
cd logscope
cargo build --release
```

The binary will be available at `target/release/logscope`.

## Usage

Basic usage to analyze a log file:

```bash
logscope analyze path/to/your/file.log
```

Filter by keyword:

```bash
logscope analyze path/to/your/file.log --keyword "database"
```

Filter by time range:

```bash
logscope analyze path/to/your/file.log --from "2026-01-15 21:00:00" --to "2026-01-15 22:00:00"
```

Show help:

```bash
logscope --help
```

## Example Output

When you run logscope on a log file, you'll get a comprehensive analysis like this:

```
=== Log Analysis Report ===

File: examples/sample.log
Total Lines: 1247
Time Range: 2026-01-15 08:12:45 to 2026-01-15 23:58:12

Log Level Distribution:
  INFO:  892 (71.5%)
  WARN:  201 (16.1%)
  ERROR: 154 (12.4%)

Top Keywords:
  1. "database" - 89 occurrences
  2. "connection" - 67 occurrences
  3. "timeout" - 45 occurrences
  4. "request" - 38 occurrences
  5. "failed" - 29 occurrences
```

## Data Analysis Table

| Metric | Value | Percentage |
|--------|-------|------------|
| Total Lines | 1247 | 100% |
| INFO Level | 892 | 71.5% |
| WARN Level | 201 | 16.1% |
| ERROR Level | 154 | 12.4% |
| Unique Keywords | 347 | - |
| Time Span | 15h 45m 27s | - |

## Project Structure

```
logscope/
├── Cargo.toml           # Project dependencies and metadata
├── README.md            # This file
├── src/
│   ├── main.rs          # Entry point and CLI initialization
│   ├── cli.rs           # Command-line argument parsing
│   ├── parser.rs        # Log file parsing logic
│   ├── analyzer.rs      # Statistical analysis engine
│   └── report.rs        # Report generation and formatting
└── examples/
    └── sample.log       # Sample log file for testing
```

## Design Decisions

This project was built with simplicity and extensibility in mind. Instead of using heavy parsing libraries, we implemented a straightforward regex-based parser that handles common log formats. The analyzer works with in-memory data structures, which keeps things fast for typical log file sizes. Each module has a clear responsibility: parsing extracts data, analyzing computes statistics, and reporting formats output.

The CLI uses clap for argument parsing because it provides a great balance between functionality and ease of use. We opted for a modular architecture so you can easily swap out components or add new analysis features without touching the core parsing logic.

## Future Improvements

There's plenty of room to grow:

- Support for more log formats (JSON, syslog, custom patterns)
- Real-time log monitoring with file watching
- Export results to JSON, CSV, or HTML
- Parallel processing for large files
- Visualization with charts and graphs
- Pattern detection and anomaly highlighting
- Configuration file support for custom rules

## License

MIT License - feel free to use this in your own projects.

---

Built by [621f4994d8b2c792f](https://github.com/621f4994d8b2c792f)
