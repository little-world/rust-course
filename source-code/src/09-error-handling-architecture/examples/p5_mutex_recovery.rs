//! Pattern 5: Recoverable vs Unrecoverable Errors
//! Example: Mutex Poisoning Recovery
//!
//! Run with: cargo run --example p5_mutex_recovery

use std::sync::{Arc, Mutex};
use std::thread;

/// Update counter, recovering from poisoned mutex.
fn update_counter(counter: &Mutex<i32>) -> Result<i32, String> {
    match counter.lock() {
        Ok(mut c) => {
            *c += 1;
            Ok(*c)
        }
        Err(poisoned) => {
            eprintln!("  [WARN] Mutex was poisoned, recovering...");
            // Get inner value even though mutex is poisoned
            let mut c = poisoned.into_inner();
            *c += 1;
            Ok(*c)
        }
    }
}

/// Strict version that refuses to use poisoned data.
fn update_counter_strict(counter: &Mutex<i32>) -> Result<i32, String> {
    match counter.lock() {
        Ok(mut c) => {
            *c += 1;
            Ok(*c)
        }
        Err(_) => Err("Mutex poisoned - data may be corrupted".into()),
    }
}

fn main() {
    println!("=== Mutex Poisoning Recovery ===\n");

    // Normal operation
    println!("=== Normal Operation ===");
    let counter = Arc::new(Mutex::new(0));

    for i in 1..=3 {
        match update_counter(&counter) {
            Ok(v) => println!("  Update {}: counter = {}", i, v),
            Err(e) => println!("  Update {}: error = {}", i, e),
        }
    }

    // Demonstrate poisoning
    println!("\n=== Mutex Poisoning Demo ===");
    let poison_counter = Arc::new(Mutex::new(0));
    let poison_clone = Arc::clone(&poison_counter);

    // Spawn thread that will panic while holding the lock
    let handle = thread::spawn(move || {
        let _lock = poison_clone.lock().unwrap();
        println!("  Thread: acquired lock, about to panic...");
        panic!("Intentional panic while holding lock!");
    });

    // Wait for thread to panic
    let _ = handle.join();
    println!("  Main: thread panicked, mutex is now poisoned");

    // Try to use poisoned mutex
    println!("\n=== Recovering from Poison ===");
    match update_counter(&poison_counter) {
        Ok(v) => println!("  Recovered! counter = {}", v),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n=== Strict (No Recovery) ===");
    match update_counter_strict(&poison_counter) {
        Ok(v) => println!("  Success: counter = {}", v),
        Err(e) => println!("  Refused: {}", e),
    }

    println!("\n=== When to Recover ===");
    println!("DO recover when:");
    println!("  - Data is immutable or easily validated");
    println!("  - Operation is idempotent");
    println!("  - Counter/statistics (increment is safe)");
    println!("  - Cache (can rebuild)");
    println!();
    println!("DON'T recover when:");
    println!("  - Data may be in inconsistent state");
    println!("  - Operation has side effects");
    println!("  - Financial/critical data");
    println!("  - Complex multi-field updates");

    println!("\n=== Recovery Pattern ===");
    println!("match mutex.lock() {{");
    println!("    Ok(guard) => use_data(guard),");
    println!("    Err(poisoned) => {{");
    println!("        log::warn!(\"Mutex poisoned, recovering\");");
    println!("        let guard = poisoned.into_inner();");
    println!("        // Validate or reset data if needed");
    println!("        use_data(guard)");
    println!("    }}");
    println!("}}");

    println!("\n=== Key Points ===");
    println!("1. Mutex is poisoned when a thread panics holding it");
    println!("2. into_inner() accesses data despite poison");
    println!("3. Recovery is a deliberate choice - not always safe");
    println!("4. Consider if panic could have left data inconsistent");
}
