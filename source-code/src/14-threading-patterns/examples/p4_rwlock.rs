//! Pattern 4: Shared State with Locks
//! Read-Heavy Workloads with RwLock
//!
//! Run with: cargo run --example p4_rwlock

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn rwlock_for_read_heavy_data() {
    let config = Arc::new(RwLock::new("initial_config".to_string()));
    let mut handles = vec![];

    // Spawn multiple reader threads.
    for i in 0..5 {
        let config_clone = Arc::clone(&config);
        let handle = thread::spawn(move || {
            // read() acquires a read lock. Multiple threads can hold a read lock.
            let cfg = config_clone.read().unwrap();
            println!("Reader {}: Current config is '{}'", i, *cfg);
        });
        handles.push(handle);
    }

    // Wait a moment, then spawn a writer thread.
    thread::sleep(Duration::from_millis(10));
    let config_clone = Arc::clone(&config);
    let writer_handle = thread::spawn(move || {
        // write() acquires a write lock. This will wait until all read locks are released.
        // No new readers can acquire a lock while the writer is waiting.
        let mut cfg = config_clone.write().unwrap();
        *cfg = "updated_config".to_string();
        println!("Writer: Updated config.");
    });
    handles.push(writer_handle);

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Final config: {}", *config.read().unwrap());
}

fn main() {
    println!("=== Read-Heavy Workloads with RwLock ===\n");
    rwlock_for_read_heavy_data();

    println!("\n=== Key Points ===");
    println!("1. RwLock allows multiple readers OR one writer");
    println!("2. Better than Mutex when reads >> writes");
    println!("3. Writers have priority (readers starve writers otherwise)");
    println!("4. Use Mutex if reads and writes are equally frequent");
}
