//! Pattern 1: Thread Spawn and Join Patterns
//! Borrowing Stack Data with Scoped Threads
//!
//! Run with: cargo run --example p1_scoped_threads

use std::thread;

fn scoped_threads_for_borrowing() {
    let mut data = vec![1, 2, 3, 4, 5];

    // 'thread::scope' creates a scope for spawning threads.
    // The scope guarantees that all threads within it will join before it exits.
    thread::scope(|s| {
        // This thread borrows 'data' immutably.
        s.spawn(|| {
            let sum: i32 = data.iter().sum();
            println!("Scoped thread sees sum: {}", sum);
        });

        // This thread also borrows 'data' immutably.
        s.spawn(|| {
            let product: i32 = data.iter().product();
            println!("Scoped thread sees product: {}", product);
        });
    }); // The scope blocks here until all spawned threads complete.

    // After the scope, we can mutate 'data' again.
    data.push(6);
    println!("After scope, data is: {:?}", data);
}

fn main() {
    println!("=== Borrowing Stack Data with Scoped Threads ===\n");
    scoped_threads_for_borrowing();

    println!("\n=== Key Points ===");
    println!("1. thread::scope allows borrowing non-'static data");
    println!("2. All threads must complete before scope exits");
    println!("3. No need for Arc or cloning for read-only access");
    println!("4. Stabilized in Rust 1.63");
}
