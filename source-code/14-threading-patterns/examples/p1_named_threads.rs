//! Pattern 1: Thread Spawn and Join Patterns
//! Naming Threads for Better Debugging
//!
//! Run with: cargo run --example p1_named_threads

use std::thread;
use std::time::Duration;

fn named_threads() {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::Builder::new()
                .name(format!("worker-{}", i))
                .spawn(move || {
                    let name = thread::current().name()
                        .unwrap_or("unnamed").to_string();
                    println!("Thread '{}' starting", name);
                    thread::sleep(Duration::from_millis(100));
                    println!("Thread '{}' finished", name);
                    i * 2
                })
                .unwrap()
        })
        .collect();

    let results: Vec<i32> = handles
        .into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    println!("Results from named threads: {:?}", results);
}

fn main() {
    println!("=== Naming Threads for Better Debugging ===\n");
    named_threads();

    println!("\n=== Key Points ===");
    println!("1. thread::Builder allows customization (name, stack size)");
    println!("2. Thread names appear in panic messages and profilers");
    println!("3. thread::current().name() retrieves the current thread's name");
}
