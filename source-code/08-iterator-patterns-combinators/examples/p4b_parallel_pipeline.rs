//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Pipeline with Multiple Stages
//!
//! Run with: cargo run --example p4b_parallel_pipeline

use rayon::prelude::*;

/// Process data through multiple parallel stages.
fn parallel_pipeline(data: &[i32]) -> Vec<i32> {
    data.par_iter()
        .map(|&x| x * 2)      // Stage 1: multiply
        .filter(|&x| x > 100) // Stage 2: filter
        .map(|x| x / 3)       // Stage 3: divide
        .collect()
}

/// Multi-stage text processing pipeline.
fn text_processing_pipeline(texts: &[String]) -> Vec<String> {
    texts
        .par_iter()
        .map(|s| s.to_lowercase())           // Stage 1: normalize case
        .map(|s| s.trim().to_string())       // Stage 2: trim whitespace
        .filter(|s| !s.is_empty())           // Stage 3: remove empty
        .filter(|s| s.len() > 3)             // Stage 4: minimum length
        .map(|s| s.replace("  ", " "))       // Stage 5: normalize spaces
        .collect()
}

/// Pipeline with side effects (logging).
fn pipeline_with_logging(data: &[i32]) -> Vec<i32> {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let processed = AtomicUsize::new(0);
    let filtered = AtomicUsize::new(0);

    let result: Vec<i32> = data
        .par_iter()
        .inspect(|_| {
            processed.fetch_add(1, Ordering::Relaxed);
        })
        .map(|&x| x * x)
        .filter(|&x| {
            let pass = x > 50;
            if pass {
                filtered.fetch_add(1, Ordering::Relaxed);
            }
            pass
        })
        .collect();

    println!("Processed: {}, Passed filter: {}",
             processed.load(Ordering::Relaxed),
             filtered.load(Ordering::Relaxed));

    result
}

/// Complex pipeline with multiple data types.
fn complex_pipeline(numbers: &[i32]) -> (Vec<String>, i32) {
    // Stage 1: Transform to strings for positive, filter negatives
    let strings: Vec<String> = numbers
        .par_iter()
        .filter(|&&x| x > 0)
        .map(|&x| format!("num_{}", x))
        .collect();

    // Stage 2: Sum of all absolute values
    let sum: i32 = numbers
        .par_iter()
        .map(|&x| x.abs())
        .sum();

    (strings, sum)
}

fn main() {
    println!("=== Parallel Pipeline with Multiple Stages ===\n");

    // Usage: chain map/filter/map operations in parallel
    let data: Vec<i32> = (1..=100).collect();
    let result = parallel_pipeline(&data);
    println!("Pipeline: multiply by 2 -> filter > 100 -> divide by 3");
    println!("Input: 1..=100");
    println!("Output ({} elements): {:?}...", result.len(), &result[..5.min(result.len())]);

    // Verify: 51*2=102 > 100, 102/3=34
    println!("\nExample: 51 -> 102 -> (passes) -> 34");

    println!("\n=== Text Processing Pipeline ===");
    let texts: Vec<String> = vec![
        "  Hello World  ".into(),
        "".into(),
        "   ".into(),
        "RUST  PROGRAMMING".into(),
        "hi".into(), // Too short
        "parallel  iteration".into(),
    ];
    println!("Input texts:");
    for t in &texts {
        println!("  '{}'", t);
    }

    let processed = text_processing_pipeline(&texts);
    println!("Processed:");
    for t in &processed {
        println!("  '{}'", t);
    }

    println!("\n=== Pipeline with Logging ===");
    let nums: Vec<i32> = (1..=20).collect();
    let result2 = pipeline_with_logging(&nums);
    println!("Results > 50: {:?}", result2);

    println!("\n=== Complex Multi-Result Pipeline ===");
    let mixed = vec![-5, 3, -2, 7, 1, -1, 4, 0, 6];
    println!("Input: {:?}", mixed);
    let (strings, sum) = complex_pipeline(&mixed);
    println!("Positive as strings: {:?}", strings);
    println!("Sum of absolute values: {}", sum);

    println!("\n=== How Rayon Pipelines Work ===");
    println!("Unlike sequential iteration:");
    println!("  Sequential: element 1 through ALL stages, then element 2...");
    println!("");
    println!("Rayon pipelines:");
    println!("  - Split data into chunks");
    println!("  - Each chunk flows through all stages on one thread");
    println!("  - Chunks processed in parallel");
    println!("  - Operations fused - no intermediate collections");

    println!("\n=== Pipeline Optimization ===");
    println!("1. Filter early - reduce work for later stages");
    println!("2. Expensive operations late - after filtering");
    println!("3. inspect() for debugging without changing data");
    println!("4. Use atomic counters for parallel-safe logging");

    println!("\n=== Key Points ===");
    println!("1. Chain map/filter/etc. on par_iter()");
    println!("2. Stages execute on same thread per chunk");
    println!("3. No intermediate allocations between stages");
    println!("4. Rayon fuses operations like sequential iterators");
}
