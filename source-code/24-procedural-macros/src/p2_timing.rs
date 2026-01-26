//! Pattern 2: Attribute Macro (Timing)
//!
//! Wraps functions with timing instrumentation without modifying their body.
//! Demonstrates non-invasive cross-cutting concerns like performance monitoring.

use my_macros::timing;
use std::thread;
use std::time::Duration;

#[timing]
fn slow_function() {
    thread::sleep(Duration::from_millis(100));
}

#[timing]
fn compute_sum(n: u64) -> u64 {
    (1..=n).sum()
}

#[timing]
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

fn main() {
    println!("=== Timing Attribute Macro Demo ===\n");

    println!("Calling slow_function()...");
    slow_function();

    println!("\nCalling compute_sum(1_000_000)...");
    let sum = compute_sum(1_000_000);
    println!("Result: {}", sum);

    println!("\nCalling fibonacci(50)...");
    let fib = fibonacci(50);
    println!("Result: {}", fib);
}
