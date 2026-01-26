//! Pattern 3: Error Propagation Strategies
//! Example: Fallible Iterator Processing
//!
//! Run with: cargo run --example p3_fallible_iterator

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("empty input")]
    EmptyInput,
    #[error("invalid format: '{0}'")]
    InvalidFormat(String),
}

fn parse_number(input: &str) -> Result<i32, ParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    trimmed
        .parse()
        .map_err(|_| ParseError::InvalidFormat(trimmed.to_string()))
}

/// Collect all results, failing on first error.
/// Type annotation on collect() drives this behavior.
fn parse_all_strict(lines: Vec<&str>) -> Result<Vec<i32>, ParseError> {
    lines.into_iter().map(parse_number).collect()
}

/// Collect only successes, discarding errors.
/// Uses filter_map with .ok() to convert Result to Option.
fn parse_all_lenient(lines: Vec<&str>) -> Vec<i32> {
    lines
        .into_iter()
        .filter_map(|line| parse_number(line).ok())
        .collect()
}

/// Collect successes and log errors.
fn parse_all_logging(lines: Vec<&str>) -> Vec<i32> {
    lines
        .into_iter()
        .filter_map(|line| {
            parse_number(line)
                .map_err(|e| eprintln!("  Warning: skipping '{}': {}", line, e))
                .ok()
        })
        .collect()
}

/// Partition into successes and failures.
fn parse_all_partitioned(lines: Vec<&str>) -> (Vec<i32>, Vec<ParseError>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for line in lines {
        match parse_number(line) {
            Ok(n) => successes.push(n),
            Err(e) => failures.push(e),
        }
    }

    (successes, failures)
}

fn main() {
    println!("=== Fallible Iterator Processing ===\n");

    let inputs = vec!["1", "2", "bad", "4", "", "5"];
    println!("Input: {:?}\n", inputs);

    // Strict mode: fail on first error
    println!("=== Strict Mode (fail fast) ===");
    match parse_all_strict(inputs.clone()) {
        Ok(nums) => println!("  Success: {:?}", nums),
        Err(e) => println!("  Failed at first error: {}", e),
    }

    // Lenient mode: keep only successes
    println!("\n=== Lenient Mode (ignore errors) ===");
    let nums = parse_all_lenient(inputs.clone());
    println!("  Parsed: {:?}", nums);

    // Logging mode: log errors but continue
    println!("\n=== Logging Mode (log and continue) ===");
    let nums = parse_all_logging(inputs.clone());
    println!("  Parsed: {:?}", nums);

    // Partitioned: separate successes and failures
    println!("\n=== Partitioned Mode (collect both) ===");
    let (successes, failures) = parse_all_partitioned(inputs.clone());
    println!("  Successes: {:?}", successes);
    println!("  Failures: {:?}", failures);

    // Using itertools partition_map (shown as manual version)
    println!("\n=== Summary Stats ===");
    let total = inputs.len();
    let (ok, err) = parse_all_partitioned(inputs);
    println!("  Total: {}", total);
    println!("  Successes: {}", ok.len());
    println!("  Failures: {}", err.len());
    println!("  Success rate: {:.1}%", (ok.len() as f64 / total as f64) * 100.0);

    println!("\n=== Key Patterns ===");
    println!("Fail fast:  iter.map(f).collect::<Result<Vec<_>, _>>()");
    println!("Keep OK:    iter.filter_map(|x| f(x).ok()).collect()");
    println!("Log errors: iter.filter_map(|x| f(x).map_err(log).ok()).collect()");
    println!("Partition:  Manual loop or itertools::partition_map");

    println!("\n=== When to Use Each ===");
    println!("Fail fast:  All items must succeed (transactions, migrations)");
    println!("Keep OK:    Partial results acceptable (web scraping, logs)");
    println!("Log errors: Need visibility into failures");
    println!("Partition:  Report all errors at once (validation)");
}
