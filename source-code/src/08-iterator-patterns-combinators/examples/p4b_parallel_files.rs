//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel File Processing
//!
//! Run with: cargo run --example p4b_parallel_files

use rayon::prelude::*;
use std::fs::File;
use std::io::Write;

/// Count lines in multiple files concurrently.
fn parallel_process_files(paths: &[String]) -> Vec<usize> {
    paths
        .par_iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .map(|content| content.lines().count())
                .unwrap_or(0)
        })
        .collect()
}

/// Parallel word count across files.
fn parallel_word_count_files(paths: &[String]) -> Vec<(String, usize)> {
    paths
        .par_iter()
        .map(|path| {
            let count = std::fs::read_to_string(path)
                .map(|content| content.split_whitespace().count())
                .unwrap_or(0);
            (path.clone(), count)
        })
        .collect()
}

/// Find files containing a pattern.
fn parallel_grep_files(paths: &[String], pattern: &str) -> Vec<String> {
    paths
        .par_iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .map(|content| content.contains(pattern))
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

/// Compute file sizes in parallel.
fn parallel_file_sizes(paths: &[String]) -> Vec<(String, u64)> {
    paths
        .par_iter()
        .filter_map(|path| {
            std::fs::metadata(path)
                .ok()
                .map(|meta| (path.clone(), meta.len()))
        })
        .collect()
}

fn main() -> std::io::Result<()> {
    println!("=== Parallel File Processing ===\n");

    // Create test files
    let test_dir = "/tmp/rayon_file_test";
    std::fs::create_dir_all(test_dir)?;

    let file_paths: Vec<String> = (1..=5)
        .map(|i| format!("{}/file{}.txt", test_dir, i))
        .collect();

    // Create files with varying content
    for (i, path) in file_paths.iter().enumerate() {
        let mut f = File::create(path)?;
        for j in 0..(i + 1) * 10 {
            writeln!(f, "Line {} in file {}: hello world Rust programming", j, i + 1)?;
            if i == 2 {
                writeln!(f, "This file contains ERROR marker")?;
            }
        }
    }
    println!("Created {} test files in {}", file_paths.len(), test_dir);

    // Usage: count lines in multiple files concurrently
    println!("\n=== Parallel Line Count ===");
    let line_counts = parallel_process_files(&file_paths);
    for (path, count) in file_paths.iter().zip(line_counts.iter()) {
        println!("  {}: {} lines", path, count);
    }

    // Word count
    println!("\n=== Parallel Word Count ===");
    let word_counts = parallel_word_count_files(&file_paths);
    for (path, count) in &word_counts {
        println!("  {}: {} words", path, count);
    }

    // Total words using parallel reduction
    let total_words: usize = word_counts.par_iter().map(|(_, c)| c).sum();
    println!("Total words across all files: {}", total_words);

    // Grep files
    println!("\n=== Parallel Grep (files containing 'ERROR') ===");
    let matching = parallel_grep_files(&file_paths, "ERROR");
    println!("Files containing 'ERROR': {:?}", matching);

    // File sizes
    println!("\n=== Parallel File Sizes ===");
    let sizes = parallel_file_sizes(&file_paths);
    for (path, size) in &sizes {
        println!("  {}: {} bytes", path, size);
    }

    let total_size: u64 = sizes.par_iter().map(|(_, s)| s).sum();
    println!("Total size: {} bytes", total_size);

    // Cleanup
    std::fs::remove_dir_all(test_dir)?;
    println!("\nCleaned up test files");

    println!("\n=== Why Parallel File I/O? ===");
    println!("1. Overlap I/O wait times - while one thread waits, others work");
    println!("2. Better utilization of SSD parallelism");
    println!("3. Simple code - par_iter handles the complexity");

    println!("\n=== Key Points ===");
    println!("1. par_iter() on paths distributes file reads");
    println!("2. Each file processed independently");
    println!("3. Results collected in order (preserves mapping)");
    println!("4. Good for I/O-bound workloads with many files");

    Ok(())
}
