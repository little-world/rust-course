//! Pattern 5: Synchronization Primitives
//! Barrier for Phased Computation
//!
//! Run with: cargo run --example p5_barrier

use std::sync::{Arc, Barrier};
use std::thread;

fn barrier_for_phased_work() {
    let num_threads = 4;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for id in 0..num_threads {
        let barrier_clone = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            println!("Thread {}: Performing phase 1", id);
            // ... do some work ...
            barrier_clone.wait(); // All threads wait here.

            println!("Thread {}: Phase 1 done. Starting phase 2.", id);
            // ... do some work ...
            barrier_clone.wait(); // Wait again.

            println!("Thread {}: Phase 2 done.", id);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn main() {
    println!("=== Barrier for Phased Computation ===\n");
    barrier_for_phased_work();

    println!("\n=== Key Points ===");
    println!("1. Barrier synchronizes N threads at a point");
    println!("2. All threads must call wait() before any can proceed");
    println!("3. Barrier can be reused (threads can wait multiple times)");
    println!("4. Useful for parallel algorithms with phases");
}
