//! Pattern 1: Thread Spawn and Join Patterns
//! Spawning a Thread with Owned Data
//!
//! Run with: cargo run --example p1_spawn_join

use std::thread;

fn spawn_with_owned_data() {
    let data = vec![1, 2, 3, 4, 5];

    // The 'move' keyword transfers ownership of 'data' to the new thread.
    let handle = thread::spawn(move || {
        let sum: i32 = data.iter().sum();
        println!("Sum calculated by thread: {}", sum);
        sum // The thread returns the sum.
    });

    // The join() method waits for the thread to finish and returns a Result.
    let result = handle.join().unwrap();
    println!("Result received from thread: {}", result);

    // This would fail to compile, as 'data' has been moved:
    // println!("Data in main thread: {:?}", data);
}

fn main() {
    println!("=== Spawning a Thread with Owned Data ===\n");
    spawn_with_owned_data();

    println!("\n=== Key Points ===");
    println!("1. 'move' closure transfers ownership to the thread");
    println!("2. join() waits for thread completion and returns Result");
    println!("3. Thread can return a value through join()");
}
