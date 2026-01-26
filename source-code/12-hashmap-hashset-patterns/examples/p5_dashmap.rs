//! Pattern 5: Concurrent HashMaps
//! DashMap for High-Concurrency Scenarios
//!
//! Run with: cargo run --example p5_dashmap

use dashmap::DashMap;
use rayon::prelude::*;
use std::sync::Arc;

fn main() {
    println!("=== Concurrent HashMaps with DashMap ===\n");

    concurrent_request_counter();

    // Concurrent read/write
    println!("\n=== Concurrent Read/Write ===\n");
    concurrent_read_write();

    println!("\n=== Key Points ===");
    println!("1. DashMap is sharded internally (many small locks)");
    println!("2. Scales almost linearly with CPU cores");
    println!("3. Entry API is thread-safe");
    println!("4. Much better than Arc<Mutex<HashMap>>");
}

fn concurrent_request_counter() {
    let counters: Arc<DashMap<String, usize>> = Arc::new(DashMap::new());

    // Simulate 1000 concurrent requests to different endpoints.
    (0..1000).into_par_iter().for_each(|i| {
        let endpoint = format!("/endpoint_{}", i % 10);
        // DashMap's entry API is similar to HashMap's and is thread-safe.
        *counters.entry(endpoint).or_insert(0) += 1;
    });

    println!("Simulated 1000 requests across 10 endpoints:");
    println!("Request counts per endpoint:");

    // Sort for consistent output
    let mut entries: Vec<_> = counters.iter()
        .map(|r| (r.key().clone(), *r.value()))
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (endpoint, count) in entries {
        println!("  {}: {} requests", endpoint, count);
    }
}

fn concurrent_read_write() {
    let cache: Arc<DashMap<i32, String>> = Arc::new(DashMap::new());

    // Writer threads
    let cache_write = cache.clone();
    let writer = std::thread::spawn(move || {
        for i in 0..100 {
            cache_write.insert(i, format!("value_{}", i));
        }
    });

    // Reader threads (can run concurrently with writer)
    let cache_read = cache.clone();
    let reader = std::thread::spawn(move || {
        let mut found = 0;
        for i in 0..100 {
            if cache_read.contains_key(&i) {
                found += 1;
            }
        }
        found
    });

    writer.join().unwrap();
    let found = reader.join().unwrap();

    println!("Writer inserted 100 items");
    println!("Reader found {} items (concurrent access)", found);
    println!("Final cache size: {}", cache.len());

    // Demonstrate concurrent modification
    println!("\nConcurrent modification example:");
    let counters: Arc<DashMap<&str, i32>> = Arc::new(DashMap::new());
    counters.insert("counter", 0);

    // 10 threads each incrementing 1000 times
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let counters = counters.clone();
            std::thread::spawn(move || {
                for _ in 0..1000 {
                    *counters.entry("counter").or_insert(0) += 1;
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("10 threads x 1000 increments = {}",
             counters.get("counter").map(|r| *r).unwrap_or(0));
}
