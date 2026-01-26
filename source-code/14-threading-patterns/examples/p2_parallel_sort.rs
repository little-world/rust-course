//! Pattern 2: Thread Pools and Work Stealing
//! Parallel Sorting with Rayon
//!
//! Run with: cargo run --example p2_parallel_sort

use rayon::prelude::*;
use std::time::Instant;

fn parallel_sorting_with_rayon() {
    let mut data: Vec<i32> = (0..1_000_000).rev().collect();
    println!("First 10 elements (before sort): {:?}", &data[..10]);

    let start = Instant::now();
    // Parallel sort is much faster for large collections.
    data.par_sort();
    let parallel_time = start.elapsed();

    println!("First 10 elements (after sort): {:?}", &data[..10]);
    println!("Parallel sort time: {:?}", parallel_time);

    // Compare with sequential sort
    let mut data2: Vec<i32> = (0..1_000_000).rev().collect();
    let start = Instant::now();
    data2.sort();
    let sequential_time = start.elapsed();

    println!("Sequential sort time: {:?}", sequential_time);
    println!(
        "Speedup: {:.2}x",
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );
}

fn main() {
    println!("=== Parallel Sorting with Rayon ===\n");
    parallel_sorting_with_rayon();

    println!("\n=== Key Points ===");
    println!("1. par_sort() uses parallel merge sort");
    println!("2. Significant speedup for large datasets");
    println!("3. Also available: par_sort_by, par_sort_unstable");
}
