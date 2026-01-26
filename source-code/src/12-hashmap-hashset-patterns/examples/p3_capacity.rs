//! Pattern 3: Capacity and Memory Management
//! Pre-allocation and Shrinking
//!
//! Run with: cargo run --example p3_capacity

use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("=== Capacity and Memory Management ===\n");

    // Batch processing with capacity
    println!("=== Pre-allocation for Batch Processing ===\n");
    batch_processing_with_capacity();

    // Shrinking to fit
    println!("\n=== Shrinking to Reclaim Memory ===\n");
    shrinking_to_fit();

    println!("\n=== Key Points ===");
    println!("1. with_capacity() avoids resize operations");
    println!("2. Reserve capacity before batch insertions");
    println!("3. shrink_to_fit() reclaims unused memory");
    println!("4. Pre-allocation can be 3-10x faster");
}

fn batch_processing_with_capacity() {
    const BATCH_SIZE: usize = 500_000;

    // Without pre-allocation
    let start = Instant::now();
    let mut map1: HashMap<usize, usize> = HashMap::new();
    for i in 0..BATCH_SIZE {
        map1.insert(i, i);
    }
    let time_without = start.elapsed();

    // With pre-allocation
    let start = Instant::now();
    let mut map2: HashMap<usize, usize> = HashMap::with_capacity(BATCH_SIZE);
    for i in 0..BATCH_SIZE {
        map2.insert(i, i);
    }
    let time_with = start.elapsed();

    println!("Inserting {} items:", BATCH_SIZE);
    println!("  Without pre-allocation: {:?}", time_without);
    println!("  With pre-allocation:    {:?}", time_with);
    println!("  Speedup:                {:.2}x",
             time_without.as_secs_f64() / time_with.as_secs_f64());

    // Using reserve for batch additions
    println!("\n=== Using reserve() ===\n");

    let mut map3: HashMap<usize, usize> = HashMap::new();
    map3.insert(0, 0);
    println!("Initial capacity: {}", map3.capacity());

    // Reserve space for 1000 more items
    map3.reserve(1000);
    println!("After reserve(1000): {}", map3.capacity());
}

fn shrinking_to_fit() {
    let mut map: HashMap<usize, usize> = HashMap::with_capacity(1000);
    println!("Initial capacity: {}", map.capacity());

    for i in 0..100 {
        map.insert(i, i);
    }
    println!("After 100 insertions:");
    println!("  Length: {}", map.len());
    println!("  Capacity: {}", map.capacity());

    // Shrink the map to reclaim the unused capacity.
    map.shrink_to_fit();
    println!("After shrink_to_fit():");
    println!("  Length: {}", map.len());
    println!("  Capacity: {}", map.capacity());

    // Memory savings
    let wasted_before = 1000 - 100;
    let wasted_after = map.capacity() as i32 - 100;
    println!("\nMemory saved: ~{} unused slots reclaimed",
             wasted_before - wasted_after as usize);
}
