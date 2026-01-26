//! Pattern 4: Shared State with Locks
//! Shared Counter with Arc<Mutex<T>>
//!
//! Run with: cargo run --example p4_shared_counter

use std::sync::{Arc, Mutex};
use std::thread;

fn shared_counter() {
    // Arc<Mutex<T>> is the standard way to share mutable state.
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        // Clone the Arc to give each thread a reference to the Mutex.
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // lock() acquires the mutex, blocking until it's available.
            // The returned "lock guard" provides access to the data.
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
            // The lock is automatically released when 'num' goes out of scope.
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}

fn main() {
    println!("=== Shared Counter with Arc<Mutex<T>> ===\n");
    shared_counter();

    println!("\n=== Key Points ===");
    println!("1. Arc allows shared ownership across threads");
    println!("2. Mutex provides interior mutability with locking");
    println!("3. Lock guard auto-releases when it goes out of scope");
    println!("4. lock() blocks; try_lock() returns immediately");
}
