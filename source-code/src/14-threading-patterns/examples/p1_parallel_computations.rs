//! Pattern 1: Thread Spawn and Join Patterns
//! Parallel Computations with Multiple Threads
//!
//! Run with: cargo run --example p1_parallel_computations

use std::thread;

fn parallel_computations() {
    let numbers = vec![1, 2, 3, 4, 5];

    // Clone data for the first thread.
    let numbers_clone1 = numbers.clone();
    let sum_handle = thread::spawn(move || {
        numbers_clone1.iter().sum::<i32>()
    });

    // Clone data for the second thread.
    let numbers_clone2 = numbers.clone();
    let product_handle = thread::spawn(move || {
        numbers_clone2.iter().product::<i32>()
    });

    // Wait for both threads to complete and collect their results.
    let sum = sum_handle.join().unwrap();
    let product = product_handle.join().unwrap();

    println!("Original data: {:?}", numbers);
    println!("Parallel Sum: {}, Parallel Product: {}", sum, product);
}

fn main() {
    println!("=== Parallel Computations with Multiple Threads ===\n");
    parallel_computations();

    println!("\n=== Key Points ===");
    println!("1. Clone data to give each thread its own copy");
    println!("2. Threads run in parallel on different CPU cores");
    println!("3. Collect results by joining all handles");
}
