//! Pattern 4a: Streaming Algorithms
//! Example: Top-K Elements without Sorting
//!
//! Run with: cargo run --example p4a_top_k

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Find top k elements from an iterator without sorting all elements.
/// Uses a min-heap of size k for O(n log k) complexity.
fn top_k<T: Ord>(iter: impl Iterator<Item = T>, k: usize) -> Vec<T> {
    let mut heap = BinaryHeap::new();

    for item in iter {
        if heap.len() < k {
            heap.push(Reverse(item));
        } else if let Some(&Reverse(ref min)) = heap.peek() {
            if &item > min {
                heap.pop();
                heap.push(Reverse(item));
            }
        }
    }

    heap.into_iter().map(|Reverse(x)| x).collect()
}

/// Find bottom k elements (smallest).
fn bottom_k<T: Ord>(iter: impl Iterator<Item = T>, k: usize) -> Vec<T> {
    let mut heap = BinaryHeap::new();

    for item in iter {
        if heap.len() < k {
            heap.push(item);
        } else if let Some(max) = heap.peek() {
            if &item < max {
                heap.pop();
                heap.push(item);
            }
        }
    }

    heap.into_iter().collect()
}

/// Find top k with a key function.
/// Note: T needs Ord for BinaryHeap to work with (K, T) tuples.
fn top_k_by_key<T, K, F>(iter: impl Iterator<Item = T>, k: usize, key_fn: F) -> Vec<T>
where
    K: Ord,
    T: Ord,
    F: Fn(&T) -> K,
{
    let mut heap: BinaryHeap<Reverse<(K, T)>> = BinaryHeap::new();

    for item in iter {
        let key = key_fn(&item);
        if heap.len() < k {
            heap.push(Reverse((key, item)));
        } else if let Some(Reverse((ref min_key, _))) = heap.peek() {
            if &key > min_key {
                heap.pop();
                heap.push(Reverse((key, item)));
            }
        }
    }

    heap.into_iter().map(|Reverse((_, item))| item).collect()
}

fn main() {
    println!("=== Top-K Elements without Sorting ===\n");

    // Usage: find top 3 elements without sorting
    let numbers = [5, 1, 9, 3, 7, 2, 8, 6, 4];
    let top3 = top_k(numbers.into_iter(), 3);
    println!("Numbers: {:?}", [5, 1, 9, 3, 7, 2, 8, 6, 4]);
    println!("Top 3: {:?}", top3);
    // Contains 7, 8, 9 (order may vary)

    println!("\n=== Large Dataset Simulation ===");
    let large_data: Vec<i32> = (1..=100_000).collect();
    let top5 = top_k(large_data.iter().copied(), 5);
    println!("Top 5 from 1..=100_000: {:?}", top5);
    // [99996, 99997, 99998, 99999, 100000] (order may vary)

    println!("\n=== Bottom K (Smallest) ===");
    let numbers = [5, 1, 9, 3, 7, 2, 8, 6, 4];
    let bottom3 = bottom_k(numbers.into_iter(), 3);
    println!("Numbers: {:?}", [5, 1, 9, 3, 7, 2, 8, 6, 4]);
    println!("Bottom 3: {:?}", bottom3);
    // Contains 1, 2, 3 (order may vary)

    println!("\n=== Top K by Key Function ===");
    let words = vec!["apple", "banana", "kiwi", "strawberry", "fig", "grape"];
    let longest3 = top_k_by_key(words.iter(), 3, |s| s.len());
    println!("Words: {:?}", ["apple", "banana", "kiwi", "strawberry", "fig", "grape"]);
    println!("Longest 3 by length: {:?}", longest3);

    println!("\n=== Complexity Analysis ===");
    println!("Finding top k from n elements:");
    println!("  - Naive (sort all): O(n log n)");
    println!("  - Min-heap approach: O(n log k)");
    println!("");
    println!("When k << n, this is much faster!");
    println!("Example: top 10 from 1 million");
    println!("  - Sort: 1M * 20 = 20M operations");
    println!("  - Heap:  1M * 3 =  3M operations");

    println!("\n=== How the Min-Heap Works ===");
    println!("1. Use BinaryHeap with Reverse for min-heap");
    println!("2. Keep only k elements in heap");
    println!("3. For each new element:");
    println!("   - If heap has < k elements, push");
    println!("   - Else if new > heap minimum, replace");
    println!("4. Final heap contains top k");

    println!("\n=== Memory Efficiency ===");
    println!("Only stores k elements, not entire dataset");
    println!("For top 10 from 1GB of data: ~40 bytes heap, not 1GB!");

    println!("\n=== Key Points ===");
    println!("1. O(n log k) beats O(n log n) sorting");
    println!("2. O(k) memory instead of O(n)");
    println!("3. Works with infinite/streaming data");
    println!("4. Reverse wrapper converts max-heap to min-heap");
}
