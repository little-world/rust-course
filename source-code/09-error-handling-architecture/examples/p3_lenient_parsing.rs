//! Pattern 3: Error Propagation Strategies
//! Example: Lenient Parsing - Log Failures, Keep Successes
//!
//! Run with: cargo run --example p3_lenient_parsing

use thiserror::Error;

#[derive(Error, Debug)]
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

/// Parse all valid values, logging failures.
/// Useful for data pipelines where partial results are acceptable.
fn parse_all_lenient(lines: Vec<&str>) -> Vec<i32> {
    lines
        .into_iter()
        .filter_map(|line| {
            parse_number(line)
                .map_err(|e| eprintln!("  Warning: skipping '{}': {}", line, e))
                .ok()
        })
        .collect()
}

/// Parse with detailed logging and statistics.
fn parse_with_stats(lines: Vec<&str>) -> (Vec<i32>, usize, usize) {
    let mut successes = Vec::new();
    let mut success_count = 0;
    let mut failure_count = 0;

    for line in lines {
        match parse_number(line) {
            Ok(n) => {
                successes.push(n);
                success_count += 1;
            }
            Err(e) => {
                eprintln!("  [SKIP] '{}': {}", line, e);
                failure_count += 1;
            }
        }
    }

    (successes, success_count, failure_count)
}

/// Batch processing with error threshold.
/// Aborts if error rate exceeds threshold.
fn parse_with_threshold(lines: Vec<&str>, max_error_rate: f64) -> Result<Vec<i32>, String> {
    let total = lines.len();
    let mut successes = Vec::new();
    let mut failures = 0;

    for line in lines {
        match parse_number(line) {
            Ok(n) => successes.push(n),
            Err(_) => {
                failures += 1;
                let error_rate = failures as f64 / total as f64;
                if error_rate > max_error_rate {
                    return Err(format!(
                        "Error rate {:.1}% exceeds threshold {:.1}%",
                        error_rate * 100.0,
                        max_error_rate * 100.0
                    ));
                }
            }
        }
    }

    Ok(successes)
}

fn main() {
    println!("=== Lenient Parsing ===\n");

    let inputs = vec!["10", "20", "bad", "30", "", "forty", "50"];
    println!("Input: {:?}\n", inputs);

    // Simple lenient parsing
    println!("=== Simple Lenient Mode ===");
    let results = parse_all_lenient(inputs.clone());
    println!("  Parsed: {:?}\n", results);

    // With statistics
    println!("=== With Statistics ===");
    let (results, ok, err) = parse_with_stats(inputs.clone());
    println!("  Parsed: {:?}", results);
    println!("  Success: {}, Failures: {}", ok, err);
    println!(
        "  Success rate: {:.1}%\n",
        (ok as f64 / (ok + err) as f64) * 100.0
    );

    // With error threshold
    println!("=== With Error Threshold ===");

    // Low threshold - should fail
    println!("  Threshold 10%:");
    match parse_with_threshold(inputs.clone(), 0.10) {
        Ok(nums) => println!("    Success: {:?}", nums),
        Err(e) => println!("    Aborted: {}", e),
    }

    // High threshold - should succeed
    println!("  Threshold 50%:");
    match parse_with_threshold(inputs.clone(), 0.50) {
        Ok(nums) => println!("    Success: {:?}", nums),
        Err(e) => println!("    Aborted: {}", e),
    }

    println!("\n=== Use Cases ===");
    println!("1. Log file parsing (skip malformed lines)");
    println!("2. Web scraping (handle missing fields)");
    println!("3. Data migration (report but continue)");
    println!("4. ETL pipelines (threshold-based abort)");

    println!("\n=== Pattern: filter_map with logging ===");
    println!("  lines.filter_map(|line| {{");
    println!("      parse(line)");
    println!("          .map_err(|e| log::warn!(\"skip: {{}}\", e))");
    println!("          .ok()");
    println!("  }})");

    println!("\n=== Key Points ===");
    println!("1. .ok() converts Result<T, E> to Option<T>");
    println!("2. filter_map keeps only Some values");
    println!("3. map_err runs side effects before .ok() discards error");
    println!("4. Threshold-based abort for quality control");
}
