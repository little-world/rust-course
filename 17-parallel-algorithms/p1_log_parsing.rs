//! Pattern 1: Log Parsing with Rayon
//!
//! Run with: cargo run --bin p1_log_parsing

use rayon::prelude::*;

#[derive(Debug)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

fn parse_logs_parallel(lines: Vec<String>) -> Vec<LogEntry> {
    lines
        .into_par_iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                Some(LogEntry {
                    timestamp: parts[0].parse().ok()?,
                    level: parts[1].to_string(),
                    message: parts[2].to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn main() {
    println!("=== Log Parsing ===\n");

    // Generate sample log lines
    let lines: Vec<String> = (0..10000)
        .map(|i| format!("{}|{}|Message number {}", i, if i % 2 == 0 { "INFO" } else { "ERROR" }, i))
        .collect();

    let start = std::time::Instant::now();
    let entries = parse_logs_parallel(lines);
    println!("Parsed {} log entries in {:?}", entries.len(), start.elapsed());

    // Show first few entries
    for entry in entries.iter().take(3) {
        println!("{:?}", entry);
    }
}
