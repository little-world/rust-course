//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Sort
//!
//! Run with: cargo run --example p4b_parallel_sort

use rayon::prelude::*;
use std::time::Instant;

/// Sort a vector in parallel using Rayon.
fn parallel_sort(mut data: Vec<i32>) -> Vec<i32> {
    data.par_sort_unstable();
    data
}

/// Sequential sort for comparison.
fn sequential_sort(mut data: Vec<i32>) -> Vec<i32> {
    data.sort_unstable();
    data
}

fn main() {
    println!("=== Parallel Sort with Rayon ===\n");

    // Usage: sort a vector using multiple cores
    let data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
    let sorted = parallel_sort(data);
    println!("Sorted: {:?}", sorted);

    // Performance comparison with larger data
    println!("\n=== Performance Comparison ===");

    // Generate random data
    let mut rng_state = 12345u64;
    let random_data: Vec<i32> = (0..5_000_000)
        .map(|_| {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            (rng_state >> 16) as i32
        })
        .collect();

    // Sequential sort
    let seq_data = random_data.clone();
    let start = Instant::now();
    let seq_sorted = sequential_sort(seq_data);
    let seq_time = start.elapsed();
    println!("Sequential sort of 5M elements: {:?}", seq_time);

    // Parallel sort
    let par_data = random_data.clone();
    let start = Instant::now();
    let par_sorted = parallel_sort(par_data);
    let par_time = start.elapsed();
    println!("Parallel sort of 5M elements: {:?}", par_time);

    // Verify results match
    assert_eq!(seq_sorted, par_sorted);
    println!("\nSpeedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    // Other parallel sort variants
    println!("\n=== Sort Variants ===");

    // Stable sort (preserves order of equal elements)
    let mut data1 = vec![(1, 'a'), (2, 'b'), (1, 'c'), (2, 'd')];
    data1.par_sort_by_key(|&(k, _)| k);
    println!("Stable sort by key: {:?}", data1);

    // Custom comparator
    let mut data2 = vec![3, 1, 4, 1, 5, 9, 2, 6];
    data2.par_sort_by(|a, b| b.cmp(a)); // Descending
    println!("Descending sort: {:?}", data2);

    // Sort strings by length
    let mut words = vec!["banana", "apple", "kiwi", "strawberry", "fig"];
    words.par_sort_by_key(|s| s.len());
    println!("Sort by length: {:?}", words);

    println!("\n=== Stable vs Unstable ===");
    println!("par_sort()          - stable, preserves equal element order");
    println!("par_sort_unstable() - faster, may reorder equal elements");
    println!("");
    println!("For primitives, unstable is usually better.");
    println!("For complex types where order matters, use stable.");

    println!("\n=== Key Points ===");
    println!("1. par_sort_unstable() is fastest for most cases");
    println!("2. par_sort_by_key() for custom key extraction");
    println!("3. par_sort_by() for full custom comparison");
    println!("4. 3-4x speedup typical on modern multi-core CPUs");
}
