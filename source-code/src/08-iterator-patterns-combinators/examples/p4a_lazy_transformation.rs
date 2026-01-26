//! Pattern 4a: Streaming Algorithms
//! Example: Lazy Transformation Chain
//!
//! Run with: cargo run --example p4a_lazy_transformation

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

/// Process a large file lazily: read, normalize, and count word frequencies.
/// Only one line is in memory at any moment.
fn process_large_file(path: &str) -> std::io::Result<Vec<(String, usize)>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let result = reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_lowercase())
        .flat_map(|line| {
            line.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .fold(HashMap::new(), |mut map, word| {
            *map.entry(word).or_insert(0) += 1;
            map
        })
        .into_iter()
        .collect();

    Ok(result)
}

/// Pipeline that transforms, filters, and aggregates in one pass.
/// Simplified version that returns collected result to avoid complex lifetime issues.
fn transform_pipeline<T, U, R, M, F, A>(
    iter: impl Iterator<Item = T>,
    transform: M,
    filter: F,
    aggregate: A,
) -> R
where
    M: Fn(T) -> U,
    F: Fn(&U) -> bool,
    A: FnOnce(Vec<U>) -> R,
{
    let transformed: Vec<U> = iter
        .map(transform)
        .filter(|x| filter(x))
        .collect();

    aggregate(transformed)
}

fn main() -> std::io::Result<()> {
    println!("=== Lazy Transformation Chain ===\n");

    // Create a test file
    let test_file = "/tmp/lazy_transform_test.txt";
    {
        let mut f = File::create(test_file)?;
        writeln!(f, "Hello World hello")?;
        writeln!(f, "Rust is great")?;
        writeln!(f, "Hello Rust")?;
        writeln!(f, "")?; // Empty line (will be filtered)
        writeln!(f, "World of Rust programming")?;
        writeln!(f, "Great RUST great")?;
    }
    println!("Test file created with content:");
    let content = std::fs::read_to_string(test_file)?;
    for line in content.lines() {
        println!("  '{}'", line);
    }

    // Process the file
    println!("\n=== Word Frequencies ===");
    let mut frequencies = process_large_file(test_file)?;
    frequencies.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

    for (word, count) in frequencies.iter().take(10) {
        println!("  '{}': {}", word, count);
    }

    // Demonstrate the pipeline concept
    println!("\n=== Pipeline Breakdown ===");
    println!("reader.lines()         // Stream lines lazily");
    println!("  .filter_map(Ok)      // Skip I/O errors");
    println!("  .filter(!empty)      // Skip empty lines");
    println!("  .map(lowercase)      // Normalize case");
    println!("  .flat_map(split)     // Split into words");
    println!("  .fold(HashMap)       // Count frequencies");
    println!("");
    println!("All in ONE PASS through the file!");

    // Generic transform pipeline
    println!("\n=== Generic Transform Pipeline ===");
    let numbers: Vec<i32> = (1..=10).collect();
    println!("Input: {:?}", numbers);

    let result = transform_pipeline(
        numbers.into_iter(),
        |x| x * x,           // Transform: square
        |&x| x > 10,         // Filter: > 10
        |iter| iter.into_iter().sum::<i32>(), // Aggregate: sum
    );
    println!("Square -> Filter(>10) -> Sum = {}", result);
    // 16 + 25 + 36 + 49 + 64 + 81 + 100 = 371

    // Cleanup
    std::fs::remove_file(test_file)?;

    println!("\n=== Why This Matters ===");
    println!("1. Memory: Only ONE line in memory at a time");
    println!("2. Speed: Single pass through data");
    println!("3. Composable: Each stage is independent");
    println!("4. Flexible: Easy to add/remove/reorder stages");

    println!("\n=== The Lazy Evaluation Model ===");
    println!("No work happens until the final consuming operation!");
    println!("");
    println!("Creating the chain: O(1) - just builds wrapper types");
    println!("Consuming (fold/collect): O(n) - actual work");

    println!("\n=== Key Points ===");
    println!("1. Chain adapters for complex transformations");
    println!("2. flat_map splits nested structures");
    println!("3. fold aggregates into any type");
    println!("4. One pass = one read through the file");

    Ok(())
}
