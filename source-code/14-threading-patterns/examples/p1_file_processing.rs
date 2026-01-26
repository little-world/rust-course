//! Pattern 1: Thread Spawn and Join Patterns
//! Parallel File Processing
//!
//! Run with: cargo run --example p1_file_processing

use std::thread;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct ProcessResult {
    path: PathBuf,
    line_count: usize,
    word_count: usize,
    byte_count: usize,
    error: Option<String>,
}

fn process_file(path: &PathBuf) -> ProcessResult {
    match fs::read_to_string(path) {
        Ok(content) => ProcessResult {
            path: path.clone(),
            line_count: content.lines().count(),
            word_count: content.split_whitespace().count(),
            byte_count: content.len(),
            error: None,
        },
        Err(e) => ProcessResult {
            path: path.clone(),
            line_count: 0,
            word_count: 0,
            byte_count: 0,
            error: Some(e.to_string()),
        },
    }
}

fn process_files_parallel(paths: Vec<PathBuf>) -> Vec<ProcessResult> {
    let handles: Vec<_> = paths
        .into_iter()
        .map(|path| {
            thread::spawn(move || {
                process_file(&path)
            })
        })
        .collect();

    handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect()
}

fn main() {
    println!("=== Parallel File Processing ===\n");

    // Process some files in parallel
    let paths: Vec<PathBuf> = vec![
        PathBuf::from("Cargo.toml"),
        PathBuf::from("src/main.rs"),
        PathBuf::from("nonexistent.txt"),
    ];

    let results = process_files_parallel(paths);
    for result in results {
        match result.error {
            Some(e) => println!("{}: Error - {}", result.path.display(), e),
            None => println!(
                "{}: {} lines, {} words, {} bytes",
                result.path.display(),
                result.line_count,
                result.word_count,
                result.byte_count
            ),
        }
    }

    println!("\n=== Key Points ===");
    println!("1. Each file processed in its own thread");
    println!("2. Results collected after all threads complete");
    println!("3. Errors handled gracefully per file");
}
