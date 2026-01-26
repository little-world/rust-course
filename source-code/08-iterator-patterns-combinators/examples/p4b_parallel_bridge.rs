//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Bridge (Converting Sequential to Parallel)
//!
//! Run with: cargo run --example p4b_parallel_bridge

use rayon::prelude::*;
use std::sync::mpsc::channel;

/// Demonstrate par_bridge with a channel.
fn parallel_bridge_example() {
    let (sender, receiver) = channel();

    // Spawn producer thread
    std::thread::spawn(move || {
        for i in 0..100 {
            sender.send(i).unwrap();
        }
    });

    // Use par_bridge to process channel items in parallel
    let sum: i32 = receiver
        .into_iter()
        .par_bridge()
        .map(|x| x * x)
        .sum();

    println!("Sum of squares (via channel): {}", sum);
}

/// Bridge a custom iterator to parallel.
fn custom_iterator_bridge() {
    // A custom generator that doesn't implement ParallelIterator
    struct MyGenerator {
        current: u32,
        max: u32,
    }

    impl Iterator for MyGenerator {
        type Item = u32;
        fn next(&mut self) -> Option<u32> {
            if self.current < self.max {
                let val = self.current;
                self.current += 1;
                Some(val)
            } else {
                None
            }
        }
    }

    let gen = MyGenerator { current: 0, max: 50 };

    // Convert to parallel with par_bridge
    let result: u32 = gen
        .par_bridge()
        .map(|x| x * 2)
        .sum();

    println!("Doubled sum from custom iterator: {}", result);
}

/// Bridge from file lines (inherently sequential source)
fn file_lines_bridge() {
    use std::io::BufRead;

    let data = "line one\nline two\nline three\nline four\nline five";
    let reader = std::io::Cursor::new(data);

    // lines() returns a sequential iterator
    let word_count: usize = reader
        .lines()
        .filter_map(Result::ok)
        .par_bridge()
        .map(|line| line.split_whitespace().count())
        .sum();

    println!("Word count (bridged from lines): {}", word_count);
}

fn main() {
    println!("=== Parallel Bridge ===\n");

    println!("=== Channel to Parallel ===");
    parallel_bridge_example();

    println!("\n=== Custom Iterator to Parallel ===");
    custom_iterator_bridge();

    println!("\n=== File Lines to Parallel ===");
    file_lines_bridge();

    println!("\n=== When to Use par_bridge ===");
    println!("1. Channel receivers (mpsc, crossbeam)");
    println!("2. Custom iterators that don't implement ParallelIterator");
    println!("3. External data sources (network streams, etc.)");
    println!("4. Any Iterator that you want to parallelize");

    println!("\n=== How par_bridge Works ===");
    println!("1. Consumes items from the sequential iterator");
    println!("2. Buffers them internally");
    println!("3. Distributes work to Rayon's thread pool");
    println!("4. Workers steal from the buffer as needed");

    println!("\n=== Performance Considerations ===");
    println!("par_bridge adds overhead:");
    println!("  - Synchronization for the input buffer");
    println!("  - Less efficient work distribution than native par_iter");
    println!("");
    println!("Best when:");
    println!("  - Per-item work is significant (hides overhead)");
    println!("  - Source is inherently sequential");
    println!("  - Alternative would be collecting everything first");

    println!("\n=== Comparison ===");
    let data: Vec<i32> = (0..1000).collect();

    // Native parallel (best performance)
    let _r1: i32 = data.par_iter().map(|&x| x * x).sum();

    // Bridge (works but slower)
    let _r2: i32 = data.iter().par_bridge().map(|&x| x * x).sum();

    println!("For Vec/slice: prefer par_iter() over iter().par_bridge()");
    println!("Use par_bridge() only for inherently sequential sources");

    println!("\n=== Key Points ===");
    println!("1. par_bridge() converts any Iterator to parallel");
    println!("2. Adds synchronization overhead");
    println!("3. Best for inherently sequential sources");
    println!("4. Prefer native par_iter() when available");
}
