//! Pattern 4a: Streaming Algorithms
//! Example: Line-by-line File Processing
//!
//! Run with: cargo run --example p4a_file_processing

use std::fs::File;
use std::io::{BufRead, BufReader, Write};

/// Count lines matching a pattern without loading the whole file.
/// Memory usage stays constant regardless of file size.
fn count_lines_matching(path: &str, pattern: &str) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| line.contains(pattern))
        .count())
}

/// Extract all lines containing a pattern.
fn grep_file(path: &str, pattern: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| line.contains(pattern))
        .collect())
}

/// Count words in a file by streaming.
fn count_words(path: &str) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .map(|line| line.split_whitespace().count())
        .sum())
}

/// Find longest line in a file.
fn longest_line(path: &str) -> std::io::Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .max_by_key(|line| line.len()))
}

fn main() -> std::io::Result<()> {
    println!("=== Line-by-line File Processing ===\n");

    // Create a sample file for demonstration
    let test_file = "/tmp/iterator_test.txt";
    {
        let mut f = File::create(test_file)?;
        writeln!(f, "Hello world")?;
        writeln!(f, "This is a test file")?;
        writeln!(f, "With multiple lines")?;
        writeln!(f, "Some contain ERROR markers")?;
        writeln!(f, "Others contain WARNING signs")?;
        writeln!(f, "And some are just normal")?;
        writeln!(f, "ERROR: Something went wrong")?;
        writeln!(f, "INFO: All is well")?;
        writeln!(f, "This is a much longer line that has many more words than the others")?;
    }
    println!("Created test file: {}", test_file);

    // Count matching lines
    println!("\n=== Count Lines Matching Pattern ===");
    let error_count = count_lines_matching(test_file, "ERROR")?;
    println!("Lines containing 'ERROR': {}", error_count);

    let test_count = count_lines_matching(test_file, "test")?;
    println!("Lines containing 'test': {}", test_count);

    // Grep-like functionality
    println!("\n=== Grep-like Line Extraction ===");
    let error_lines = grep_file(test_file, "ERROR")?;
    println!("Lines with 'ERROR':");
    for line in &error_lines {
        println!("  {}", line);
    }

    // Word count
    println!("\n=== Word Count ===");
    let words = count_words(test_file)?;
    println!("Total words in file: {}", words);

    // Longest line
    println!("\n=== Longest Line ===");
    if let Some(longest) = longest_line(test_file)? {
        println!("Longest line ({} chars):", longest.len());
        println!("  {}", longest);
    }

    // Demonstrate streaming behavior
    println!("\n=== Memory-Efficient Streaming ===");
    println!("Key insight: BufReader.lines() is lazy!");
    println!("- Each line is read on demand");
    println!("- Memory usage is O(max_line_length), not O(file_size)");
    println!("- Can process files larger than RAM");

    // Cleanup
    std::fs::remove_file(test_file)?;
    println!("\nCleaned up test file");

    println!("\n=== Key Points ===");
    println!("1. BufReader::lines() streams lazily");
    println!("2. filter_map(Result::ok) handles I/O errors gracefully");
    println!("3. Memory stays constant regardless of file size");
    println!("4. Perfect for grep, wc, and similar utilities");

    Ok(())
}
