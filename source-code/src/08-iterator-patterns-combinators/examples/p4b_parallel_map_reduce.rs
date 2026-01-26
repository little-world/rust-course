//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Map-Reduce Pattern
//!
//! Run with: cargo run --example p4b_parallel_map_reduce

use rayon::prelude::*;
use std::collections::HashMap;

/// Parallel word frequency count using map-reduce pattern.
/// fold builds per-thread maps, reduce merges them.
fn parallel_map_reduce(data: &[String]) -> HashMap<String, usize> {
    data.par_iter()
        .fold(
            || HashMap::new(), // Identity: empty HashMap per thread
            |mut map, line| {
                // Map phase: count words in this line
                for word in line.split_whitespace() {
                    *map.entry(word.to_string()).or_insert(0) += 1;
                }
                map
            },
        )
        .reduce(
            || HashMap::new(), // Identity for reduce
            |mut a, b| {
                // Reduce phase: merge HashMaps
                for (key, count) in b {
                    *a.entry(key).or_insert(0) += count;
                }
                a
            },
        )
}

/// Generic map-reduce with custom types.
/// Simplified: map produces R, reduce combines two R values.
fn map_reduce<T, R, MapFn, ReduceFn>(
    data: &[T],
    identity: impl Fn() -> R + Sync + Send,
    map_fn: MapFn,
    reduce_fn: ReduceFn,
) -> R
where
    T: Sync,
    R: Send,
    MapFn: Fn(&T) -> R + Sync,
    ReduceFn: Fn(R, R) -> R + Sync + Send,
{
    data.par_iter()
        .map(|item| map_fn(item))
        .reduce(&identity, |a, b| reduce_fn(a, b))
}

/// Character frequency using map-reduce.
fn char_frequency_mapreduce(texts: &[String]) -> HashMap<char, usize> {
    texts
        .par_iter()
        .fold(
            HashMap::new,
            |mut map, text| {
                for c in text.chars() {
                    if c.is_alphabetic() {
                        *map.entry(c.to_ascii_lowercase()).or_insert(0) += 1;
                    }
                }
                map
            },
        )
        .reduce(
            HashMap::new,
            |mut a, b| {
                for (k, v) in b {
                    *a.entry(k).or_insert(0) += v;
                }
                a
            },
        )
}

fn main() {
    println!("=== Parallel Map-Reduce Pattern ===\n");

    // Word frequency example
    let lines: Vec<String> = vec![
        "hello world hello".into(),
        "world of rust".into(),
        "rust is awesome".into(),
        "hello rust world".into(),
    ];

    println!("Lines:");
    for line in &lines {
        println!("  '{}'", line);
    }

    let word_counts = parallel_map_reduce(&lines);
    println!("\nWord frequencies:");
    let mut sorted: Vec<_> = word_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (word, count) in sorted {
        println!("  '{}': {}", word, count);
    }

    // Character frequency
    println!("\n=== Character Frequency ===");
    let texts: Vec<String> = vec![
        "The quick brown fox".into(),
        "jumps over the lazy dog".into(),
    ];
    let char_freq = char_frequency_mapreduce(&texts);
    let mut sorted_chars: Vec<_> = char_freq.iter().collect();
    sorted_chars.sort_by(|a, b| b.1.cmp(a.1));
    println!("Character frequencies (top 10):");
    for (c, count) in sorted_chars.iter().take(10) {
        println!("  '{}': {}", c, count);
    }

    // Generic map-reduce example
    println!("\n=== Generic Map-Reduce ===");
    let numbers: Vec<i32> = (1..=100).collect();

    // Sum of squares using map-reduce
    let sum_of_squares: i64 = map_reduce(
        &numbers,
        || 0i64,
        |&x| (x as i64) * (x as i64),
        |acc, sq| acc + sq,
    );
    println!("Sum of squares (1..=100): {}", sum_of_squares);

    println!("\n=== How Map-Reduce Works ===");
    println!("1. FOLD phase (parallel):");
    println!("   - Each thread gets identity value");
    println!("   - Processes its portion of data");
    println!("   - Builds local partial result");
    println!("");
    println!("2. REDUCE phase (tree reduction):");
    println!("   - Combine partial results from threads");
    println!("   - Tree structure: log(n) depth");
    println!("   - Final single result");

    println!("\n=== Why fold + reduce? ===");
    println!("Separate phases allow:");
    println!("  - Different identity for each phase");
    println!("  - Optimized local accumulation (fold)");
    println!("  - Efficient tree-based combination (reduce)");

    println!("\n=== Key Points ===");
    println!("1. fold() creates local accumulators per thread");
    println!("2. reduce() merges all local results");
    println!("3. No locks needed - each thread works independently");
    println!("4. Classic pattern for parallel aggregation");
}
