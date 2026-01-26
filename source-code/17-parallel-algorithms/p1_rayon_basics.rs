//! Pattern 1: Rayon Basics - Parallel Iterator Examples
//!
//! Run with: cargo run --bin p1_rayon_basics

use rayon::prelude::*;
use std::time::Instant;

fn parallel_map_example() {
    let numbers: Vec<i64> = (0..1_000_000).collect();

    // Sequential
    let start = Instant::now();
    let sequential: Vec<i64> = numbers.iter().map(|&x| x * x).collect();
    let seq_time = start.elapsed();

    // Parallel
    let start = Instant::now();
    let parallel: Vec<i64> = numbers.par_iter().map(|&x| x * x).collect();
    let par_time = start.elapsed();

    println!("Sequential: {:?}", seq_time);
    println!("Parallel: {:?}", par_time);
    println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    assert_eq!(sequential, parallel);
}

fn iterator_variants() {
    let mut data = vec![1, 2, 3, 4, 5];

    // Immutable parallel iteration
    let sum: i32 = data.par_iter().sum();
    println!("Sum: {}", sum);

    // Mutable parallel iteration
    data.par_iter_mut().for_each(|x| *x *= 2);
    println!("Doubled: {:?}", data);

    // Consuming parallel iteration
    let owned_data = vec![1, 2, 3, 4, 5];
    let squares: Vec<i32> = owned_data.into_par_iter().map(|x| x * x).collect();
    println!("Squares: {:?}", squares);
}

fn parallel_filter_map() {
    let numbers: Vec<i32> = (0..10_000).collect();

    let result: Vec<i32> = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * x)
        .collect();

    println!("Filtered and squared {} numbers", result.len());
}

fn parallel_flat_map() {
    let ranges: Vec<std::ops::Range<i32>> = vec![0..10, 10..20, 20..30];

    let flattened: Vec<i32> = ranges
        .into_par_iter()
        .flat_map(|range| range.into_par_iter())
        .collect();

    println!("Flattened {} items", flattened.len());
}

fn main() {
    println!("=== Parallel Map Example ===\n");
    parallel_map_example();

    println!("\n=== Iterator Variants ===\n");
    iterator_variants();

    println!("\n=== Parallel Filter and Map ===\n");
    parallel_filter_map();

    println!("\n=== Parallel Flat Map ===\n");
    parallel_flat_map();
}
