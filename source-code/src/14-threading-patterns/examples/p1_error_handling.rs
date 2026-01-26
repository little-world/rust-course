//! Pattern 1: Thread Spawn and Join Patterns
//! Handling Errors and Panics in Threads
//!
//! Run with: cargo run --example p1_error_handling

use std::thread;

fn thread_with_error_handling() {
    let handle = thread::spawn(|| {
        // Simulate a computation that might fail.
        let value = 42;
        if value > 0 {
            Ok(value)
        } else {
            Err("Computation failed in thread!")
        }
    });

    match handle.join() {
        Ok(Ok(value)) => {
            println!("Thread completed with value: {}", value)
        }
        Ok(Err(e)) => {
            println!("Thread returned error: {}", e)
        }
        Err(_) => println!("Thread panicked!"),
    }
}

fn main() {
    println!("=== Handling Errors and Panics in Threads ===\n");
    thread_with_error_handling();

    println!("\n=== Key Points ===");
    println!("1. join() returns Result<T, Box<dyn Any + Send>>");
    println!("2. Err from join() means the thread panicked");
    println!("3. Thread's own Result creates nested Result handling");
}
