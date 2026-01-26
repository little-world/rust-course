//! Pattern 3: Advanced Iterator Composition
//! Example: Complex Filtering and Transformation Pipeline
//!
//! Run with: cargo run --example p3_log_analysis

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: u64,
    level: String,
    message: String,
}

/// Analyze logs by filtering and counting by level.
/// Multiple filters chain without creating intermediate vectors.
fn analyze_logs(logs: &[LogEntry]) -> HashMap<String, usize> {
    logs.iter()
        .filter(|e| e.level == "ERROR" || e.level == "WARN")
        .filter(|entry| entry.timestamp > 1_000_000)
        .map(|entry| &entry.level)
        .fold(HashMap::new(), |mut map, level| {
            *map.entry(level.clone()).or_insert(0) += 1;
            map
        })
}

/// Extract error messages within a time range.
fn extract_errors(logs: &[LogEntry], start: u64, end: u64) -> Vec<&str> {
    logs.iter()
        .filter(|e| e.level == "ERROR")
        .filter(|e| e.timestamp >= start && e.timestamp <= end)
        .map(|e| e.message.as_str())
        .collect()
}

/// Count logs per minute (assuming timestamps in seconds).
fn logs_per_minute(logs: &[LogEntry]) -> HashMap<u64, usize> {
    logs.iter()
        .map(|e| e.timestamp / 60)
        .fold(HashMap::new(), |mut map, minute| {
            *map.entry(minute).or_insert(0) += 1;
            map
        })
}

fn main() {
    println!("=== Complex Filtering and Transformation Pipeline ===\n");

    // Create sample log data
    let logs = vec![
        LogEntry { timestamp: 1_000_001, level: "INFO".into(), message: "Starting up".into() },
        LogEntry { timestamp: 1_000_002, level: "ERROR".into(), message: "Connection failed".into() },
        LogEntry { timestamp: 1_000_003, level: "WARN".into(), message: "Retrying".into() },
        LogEntry { timestamp: 1_000_004, level: "ERROR".into(), message: "Database timeout".into() },
        LogEntry { timestamp: 1_000_005, level: "INFO".into(), message: "Connected".into() },
        LogEntry { timestamp: 999_999, level: "ERROR".into(), message: "Old error".into() },
        LogEntry { timestamp: 1_000_060, level: "WARN".into(), message: "Memory low".into() },
        LogEntry { timestamp: 1_000_061, level: "ERROR".into(), message: "Crash".into() },
    ];

    // Usage: filter logs by level and count occurrences
    let counts = analyze_logs(&logs);
    println!("Log counts (WARN/ERROR after timestamp 1_000_000):");
    for (level, count) in &counts {
        println!("  {}: {}", level, count);
    }

    println!("\n=== Extract Errors in Time Range ===");
    let errors = extract_errors(&logs, 1_000_000, 1_000_010);
    println!("Errors between timestamps 1_000_000 and 1_000_010:");
    for msg in errors {
        println!("  - {}", msg);
    }

    println!("\n=== Logs Per Minute ===");
    let per_minute = logs_per_minute(&logs);
    let mut minutes: Vec<_> = per_minute.iter().collect();
    minutes.sort_by_key(|&(m, _)| m);
    for (minute, count) in minutes {
        println!("  Minute {}: {} logs", minute, count);
    }

    println!("\n=== Pipeline Demonstration ===");
    println!("The pipeline:");
    println!("  logs.iter()");
    println!("    .filter(|e| e.level == 'ERROR' || e.level == 'WARN')");
    println!("    .filter(|e| e.timestamp > 1_000_000)");
    println!("    .map(|e| &e.level)");
    println!("    .fold(...accumulate into HashMap...)");
    println!("\nAll steps execute lazily in a single pass!");

    println!("\n=== Key Points ===");
    println!("1. Multiple filters chain efficiently");
    println!("2. fold accumulates into HashMap in single pass");
    println!("3. No intermediate vectors are created");
    println!("4. Each element flows through entire pipeline before next");
}
