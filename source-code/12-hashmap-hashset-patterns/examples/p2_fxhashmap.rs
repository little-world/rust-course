//! Pattern 2: Custom Hashing and Equality
//! Faster Hashing with FxHashMap
//!
//! Run with: cargo run --example p2_fxhashmap

use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("=== FxHashMap vs Standard HashMap ===\n");

    benchmark_fxhashmap();

    println!("\n=== Key Points ===");
    println!("1. FxHashMap uses non-cryptographic hash (faster)");
    println!("2. Only use for trusted keys (not user input)");
    println!("3. 2-3x faster for integer keys");
    println!("4. Used by rustc compiler internally");
}

fn benchmark_fxhashmap() {
    const SIZE: usize = 1_000_000;

    // Benchmark FxHashMap insertion
    let start = Instant::now();
    let mut fx_map: FxHashMap<usize, usize> = FxHashMap::default();
    for i in 0..SIZE {
        fx_map.insert(i, i);
    }
    let fx_insert_time = start.elapsed();

    // Benchmark standard HashMap insertion
    let start = Instant::now();
    let mut std_map: HashMap<usize, usize> = HashMap::new();
    for i in 0..SIZE {
        std_map.insert(i, i);
    }
    let std_insert_time = start.elapsed();

    println!("Insertion of {} items:", SIZE);
    println!("  FxHashMap:       {:?}", fx_insert_time);
    println!("  Standard HashMap: {:?}", std_insert_time);
    println!("  Speedup:          {:.2}x",
             std_insert_time.as_secs_f64() / fx_insert_time.as_secs_f64());

    // Benchmark lookups
    let start = Instant::now();
    for i in 0..SIZE {
        let _ = fx_map.get(&i);
    }
    let fx_lookup_time = start.elapsed();

    let start = Instant::now();
    for i in 0..SIZE {
        let _ = std_map.get(&i);
    }
    let std_lookup_time = start.elapsed();

    println!("\nLookup of {} items:", SIZE);
    println!("  FxHashMap:       {:?}", fx_lookup_time);
    println!("  Standard HashMap: {:?}", std_lookup_time);
    println!("  Speedup:          {:.2}x",
             std_lookup_time.as_secs_f64() / fx_lookup_time.as_secs_f64());
}
