//! Pattern 3: Error Propagation Strategies
//! Example: Partitioning Results into Successes and Failures
//!
//! Run with: cargo run --example p3_partition_results

use itertools::{Either, Itertools};

fn parse_number(input: &str) -> Result<i32, String> {
    input
        .trim()
        .parse()
        .map_err(|_| format!("invalid: '{}'", input))
}

/// Partition results using itertools.
fn partition_with_itertools(inputs: Vec<&str>) -> (Vec<i32>, Vec<String>) {
    inputs
        .into_iter()
        .map(parse_number)
        .partition_map(|r| match r {
            Ok(v) => Either::Left(v),
            Err(e) => Either::Right(e),
        })
}

/// Manual partition without external crate.
fn partition_manual(inputs: Vec<&str>) -> (Vec<i32>, Vec<String>) {
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for input in inputs {
        match parse_number(input) {
            Ok(n) => successes.push(n),
            Err(e) => failures.push(e),
        }
    }

    (successes, failures)
}

fn main() {
    println!("=== Partitioning Results ===\n");

    let inputs = vec!["1", "two", "3", "four", "5", "", "7"];
    println!("Input: {:?}\n", inputs);

    // Using itertools
    println!("=== With itertools::partition_map ===");
    let (ok, err) = partition_with_itertools(inputs.clone());
    println!("  Successes: {:?}", ok);
    println!("  Failures: {:?}", err);

    // Manual implementation
    println!("\n=== Manual Implementation ===");
    let (ok, err) = partition_manual(inputs.clone());
    println!("  Successes: {:?}", ok);
    println!("  Failures: {:?}", err);

    // Report all errors at once
    println!("\n=== Validation Report ===");
    let (values, errors) = partition_manual(inputs);
    if errors.is_empty() {
        println!("  All {} values parsed successfully", values.len());
    } else {
        println!("  Parsed {} values, {} errors:", values.len(), errors.len());
        for (i, err) in errors.iter().enumerate() {
            println!("    {}. {}", i + 1, err);
        }
    }

    println!("\n=== Pattern: partition_map ===");
    println!("  results.partition_map(|r| match r {{");
    println!("      Ok(v) => Either::Left(v),");
    println!("      Err(e) => Either::Right(e),");
    println!("  }})");

    println!("\n=== Use Cases ===");
    println!("1. Batch validation - report all errors at once");
    println!("2. Data migration - process valid, log invalid");
    println!("3. Form validation - show all field errors");
    println!("4. Import jobs - success count vs failure count");

    println!("\n=== Key Points ===");
    println!("1. partition_map separates successes and failures in one pass");
    println!("2. Either::Left collects Ok values, Either::Right collects Err");
    println!("3. Better UX than failing on first error");
    println!("4. Manual version works without dependencies");
}
