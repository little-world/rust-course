//! Pattern 2: Thread Pools and Work Stealing
//! Data Parallelism with Rayon
//!
//! Run with: cargo run --example p2_rayon_basics

use rayon::prelude::*;

fn parallel_map_reduce_with_rayon() {
    let numbers: Vec<i32> = (1..=1_000_000).collect();

    // Parallel sum
    let sum: i32 = numbers.par_iter().sum();
    println!("Parallel Sum (Rayon): {}", sum);

    // Parallel map
    let squares: Vec<i32> = numbers
        .par_iter()
        .map(|&x| x * x)
        .collect();
    println!("First 5 squares: {:?}", &squares[..5]);

    // Parallel filter and count
    let even_count = numbers
        .par_iter()
        .filter(|&&x| x % 2 == 0)
        .count();
    println!("Number of even numbers: {}", even_count);
}

fn main() {
    println!("=== Data Parallelism with Rayon ===\n");
    parallel_map_reduce_with_rayon();

    println!("\n=== Key Points ===");
    println!("1. par_iter() converts sequential iterator to parallel");
    println!("2. Work-stealing scheduler distributes work across cores");
    println!("3. Same API as standard iterators");
    println!("4. Ideal for CPU-bound data processing");
}
