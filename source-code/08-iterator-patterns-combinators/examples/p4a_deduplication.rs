//! Pattern 4a: Streaming Algorithms
//! Example: Streaming Deduplication
//!
//! Run with: cargo run --example p4a_deduplication

use std::collections::HashSet;

/// Deduplicate a stream lazily using a HashSet to track seen values.
/// Memory grows only with unique elements, not total elements.
fn deduplicate_stream<T>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut seen = HashSet::new();
    iter.filter(move |item| seen.insert(item.clone()))
}

/// Deduplicate by a key function (like SQL DISTINCT ON).
fn deduplicate_by_key<T, K, F>(iter: impl Iterator<Item = T>, key_fn: F) -> impl Iterator<Item = T>
where
    K: Eq + std::hash::Hash,
    F: Fn(&T) -> K,
{
    let mut seen = HashSet::new();
    iter.filter(move |item| seen.insert(key_fn(item)))
}

/// Keep only first N occurrences of each value.
fn limit_occurrences<T>(
    iter: impl Iterator<Item = T>,
    max_count: usize,
) -> impl Iterator<Item = T>
where
    T: Eq + std::hash::Hash + Clone,
{
    let mut counts: std::collections::HashMap<T, usize> = std::collections::HashMap::new();
    iter.filter(move |item| {
        let count = counts.entry(item.clone()).or_insert(0);
        if *count < max_count {
            *count += 1;
            true
        } else {
            false
        }
    })
}

/// Consecutive deduplication (like Unix `uniq`).
fn consecutive_dedup<T: Clone + PartialEq>(
    iter: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let mut prev: Option<T> = None;
    iter.filter(move |item| {
        let dominated = prev.as_ref() == Some(item);
        if !dominated {
            prev = Some(item.clone());
        }
        !dominated
    })
}

fn main() {
    println!("=== Streaming Deduplication ===\n");

    // Usage: remove duplicates while streaming
    let unique: Vec<_> = deduplicate_stream([1, 2, 1, 3, 2, 4, 1, 5].into_iter()).collect();
    println!("deduplicate([1, 2, 1, 3, 2, 4, 1, 5]) = {:?}", unique);
    // [1, 2, 3, 4, 5]

    println!("\n=== How It Works ===");
    let mut seen = HashSet::new();
    for x in [1, 2, 1, 3, 2] {
        let is_new = seen.insert(x);
        println!("  {} -> is_new: {} (seen: {:?})", x, is_new, seen);
    }

    println!("\n=== Deduplicate Strings ===");
    let words = vec!["apple", "banana", "apple", "cherry", "banana", "date"];
    let unique_words: Vec<_> = deduplicate_stream(words.into_iter()).collect();
    println!("Unique words: {:?}", unique_words);

    println!("\n=== Deduplicate by Key ===");
    #[derive(Debug, Clone)]
    struct Person {
        name: String,
        age: u32,
    }

    let people = vec![
        Person { name: "Alice".into(), age: 30 },
        Person { name: "Bob".into(), age: 25 },
        Person { name: "Alice".into(), age: 31 }, // Same name, different age
        Person { name: "Carol".into(), age: 35 },
    ];

    // Keep first person with each name
    let unique_names: Vec<_> = deduplicate_by_key(people.into_iter(), |p| p.name.clone()).collect();
    println!("Unique by name:");
    for p in &unique_names {
        println!("  {:?}", p);
    }

    println!("\n=== Limit Occurrences ===");
    let repeated = vec![1, 1, 1, 2, 2, 2, 2, 3, 3, 1, 1, 2];
    let limited: Vec<_> = limit_occurrences(repeated.into_iter(), 2).collect();
    println!("Max 2 occurrences of each: {:?}", limited);
    // [1, 1, 2, 2, 3, 3]

    println!("\n=== Consecutive Dedup (like `uniq`) ===");
    let signal = vec!['a', 'a', 'b', 'b', 'b', 'a', 'a', 'c'];
    let compressed: Vec<_> = consecutive_dedup(signal.into_iter()).collect();
    println!("Consecutive dedup ['a','a','b','b','b','a','a','c']: {:?}", compressed);
    // ['a', 'b', 'a', 'c']

    println!("\n=== Memory Characteristics ===");
    println!("Full dedup: O(unique_elements) memory");
    println!("Consecutive dedup: O(1) memory");
    println!("");
    println!("For 1M elements with 1K unique:");
    println!("  - Full dedup: ~40KB (1K * 40 bytes per entry)");
    println!("  - Consecutive: ~constant");

    println!("\n=== Key Points ===");
    println!("1. HashSet.insert() returns true if item was new");
    println!("2. Memory grows with unique elements, not total");
    println!("3. Consecutive dedup is O(1) memory");
    println!("4. Can dedupe by key for complex types");
}
