//! Pattern 1: Capacity Management
//! Example: Pre-allocate When Size is Known
//!
//! Run with: cargo run --example p1_preallocate

fn main() {
    println!("=== Pre-allocation with with_capacity ===\n");

    // Demonstrate allocation difference
    let n = 100_000;

    // Without pre-allocation (triggers multiple reallocations)
    let start = std::time::Instant::now();
    let mut vec_no_prealloc: Vec<i32> = Vec::new();
    for i in 0..n {
        vec_no_prealloc.push(i);
    }
    let no_prealloc_time = start.elapsed();

    // With pre-allocation (no reallocations)
    let start = std::time::Instant::now();
    let mut vec_prealloc: Vec<i32> = Vec::with_capacity(n as usize);
    for i in 0..n {
        vec_prealloc.push(i);
    }
    let prealloc_time = start.elapsed();

    println!("Building {} elements:", n);
    println!("  Without pre-allocation: {:?}", no_prealloc_time);
    println!("  With pre-allocation:    {:?}", prealloc_time);

    // Track allocations during growth
    println!("\n=== Tracking Allocations During Growth ===\n");

    let mut allocations = 0;
    let mut vec: Vec<i32> = Vec::new();
    let mut prev_cap = vec.capacity();

    for i in 0..20 {
        vec.push(i);
        if vec.capacity() != prev_cap {
            allocations += 1;
            println!(
                "  Reallocation {}: len={}, capacity {} -> {}",
                allocations,
                vec.len(),
                prev_cap,
                vec.capacity()
            );
            prev_cap = vec.capacity();
        }
    }

    println!("\nTotal reallocations for 20 pushes: {}", allocations);

    // Pre-allocated version
    println!("\n=== Pre-allocated Version ===");
    let mut vec: Vec<i32> = Vec::with_capacity(20);
    let initial_cap = vec.capacity();
    for i in 0..20 {
        vec.push(i);
    }
    println!("  Initial capacity: {}", initial_cap);
    println!("  Final capacity: {}", vec.capacity());
    println!("  Reallocations: 0");

    // Batch processing example
    println!("\n=== Batch Processing Pattern ===\n");

    #[derive(Debug, Clone)]
    struct Item {
        id: usize,
        value: i32,
    }

    #[derive(Debug)]
    struct ProcessedItem {
        id: usize,
        result: i32,
    }

    fn process(item: &Item) -> ProcessedItem {
        ProcessedItem {
            id: item.id,
            result: item.value * 2,
        }
    }

    fn process_batch(items: &[Item]) -> Vec<ProcessedItem> {
        // Pre-allocate to avoid reallocations
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            results.push(process(item));
        }
        results
    }

    let items: Vec<Item> = (0..5)
        .map(|i| Item { id: i, value: i as i32 * 10 })
        .collect();

    let results = process_batch(&items);
    println!("Input items: {:?}", items);
    println!("Processed results: {:?}", results);

    println!("\n=== Key Points ===");
    println!("1. with_capacity(n) pre-allocates space for n elements");
    println!("2. Avoids O(log n) reallocations during construction");
    println!("3. Essential for performance-critical vector building");
    println!("4. Use when size is known or can be estimated");
}
