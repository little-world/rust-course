//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Basic Parallel Iteration
//!
//! Run with: cargo run --example p4b_parallel_basics

use rayon::prelude::*;
use std::time::Instant;

/// Compute sum of squares in parallel.
/// Just change .iter() to .par_iter()!
fn parallel_sum_of_squares(numbers: &[i64]) -> i64 {
    numbers
        .par_iter() // The only change needed for parallelization!
        .map(|&x| x * x)
        .sum()
}

/// Sequential version for comparison.
fn sequential_sum_of_squares(numbers: &[i64]) -> i64 {
    numbers
        .iter()
        .map(|&x| x * x)
        .sum()
}

fn main() {
    println!("=== Basic Parallel Iteration with Rayon ===\n");

    // Usage: compute sum of squares in parallel
    let small_data: Vec<i64> = (1..=10).collect();
    let result = parallel_sum_of_squares(&small_data);
    println!("Sum of squares of 1..=10: {}", result);
    // 1 + 4 + 9 + 16 + 25 + 36 + 49 + 64 + 81 + 100 = 385

    // Performance comparison
    println!("\n=== Performance Comparison ===");
    // Use 1 million elements (10M would overflow i64 with sum of squares)
    let large_data: Vec<i64> = (1..=1_000_000).collect();

    let start = Instant::now();
    let seq_result = sequential_sum_of_squares(&large_data);
    let seq_time = start.elapsed();
    println!("Sequential: {} in {:?}", seq_result, seq_time);

    let start = Instant::now();
    let par_result = parallel_sum_of_squares(&large_data);
    let par_time = start.elapsed();
    println!("Parallel:   {} in {:?}", par_result, par_time);

    assert_eq!(seq_result, par_result);
    println!("\nSpeedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    // Different parallel operations
    println!("\n=== Various Parallel Operations ===");

    // Parallel map and collect
    let squares: Vec<i64> = (1..=5).into_par_iter().map(|x| x * x).collect();
    println!("Parallel squares: {:?}", squares);

    // Parallel filter
    let evens: Vec<i64> = (1..=10).into_par_iter().filter(|&x| x % 2 == 0).collect();
    println!("Parallel evens: {:?}", evens);

    // Parallel any/all
    let data = vec![2, 4, 6, 8, 10];
    let all_even = data.par_iter().all(|&x| x % 2 == 0);
    println!("All even? {}", all_even);

    let has_big = data.par_iter().any(|&x| x > 5);
    println!("Any > 5? {}", has_big);

    // Parallel count
    let big_count = (1..=1000).into_par_iter().filter(|&x| x > 500).count();
    println!("Count > 500 in 1..=1000: {}", big_count);

    println!("\n=== The Magic of Rayon ===");
    println!(".iter()     -> sequential iteration");
    println!(".par_iter() -> parallel iteration");
    println!("");
    println!("That's it! Rayon handles:");
    println!("  - Thread pool management");
    println!("  - Work distribution");
    println!("  - Work stealing for load balancing");
    println!("  - Result aggregation");

    println!("\n=== Key Points ===");
    println!("1. Drop-in replacement: .iter() -> .par_iter()");
    println!("2. Automatic thread pool and work stealing");
    println!("3. Same iterator API (map, filter, fold, etc.)");
    println!("4. Near-linear speedup for data-parallel tasks");
}
