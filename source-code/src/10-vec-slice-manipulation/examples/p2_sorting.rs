//! Pattern 2: Slice Algorithms
//! Example: Custom Sorting with Comparators
//!
//! Run with: cargo run --example p2_sorting

use std::cmp::Ordering;

#[derive(Debug, Clone)]
struct Task {
    id: usize,
    priority: u8,
    timestamp: u64,
    name: String,
}

fn main() {
    println!("=== Custom Sorting with Comparators ===\n");

    let mut tasks = vec![
        Task { id: 1, priority: 2, timestamp: 100, name: "Task A".into() },
        Task { id: 2, priority: 1, timestamp: 50, name: "Task B".into() },
        Task { id: 3, priority: 2, timestamp: 75, name: "Task C".into() },
        Task { id: 4, priority: 3, timestamp: 200, name: "Task D".into() },
        Task { id: 5, priority: 1, timestamp: 150, name: "Task E".into() },
    ];

    println!("Original tasks:");
    for task in &tasks {
        println!("  {:?}", task);
    }

    // Sort by single field
    tasks.sort_by_key(|t| t.priority);
    println!("\nSorted by priority (ascending):");
    for task in &tasks {
        println!("  priority={}: {}", task.priority, task.name);
    }

    // Sort by priority descending
    tasks.sort_by_key(|t| std::cmp::Reverse(t.priority));
    println!("\nSorted by priority (descending):");
    for task in &tasks {
        println!("  priority={}: {}", task.priority, task.name);
    }

    // Multi-level sort
    fn sort_by_priority(tasks: &mut [Task]) {
        tasks.sort_by(|a, b| {
            // Sort by priority descending, then by timestamp ascending
            b.priority.cmp(&a.priority)
                .then_with(|| a.timestamp.cmp(&b.timestamp))
        });
    }

    sort_by_priority(&mut tasks);
    println!("\nSorted by priority (desc), then timestamp (asc):");
    for task in &tasks {
        println!("  priority={}, timestamp={}: {}", task.priority, task.timestamp, task.name);
    }

    // Stable vs unstable sort
    println!("\n=== Stable vs Unstable Sort ===\n");

    let mut data: Vec<f64> = vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
    println!("Original: {:?}", data);

    // Stable sort (preserves relative order of equal elements)
    let mut stable = data.clone();
    stable.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("After sort (stable): {:?}", stable);

    // Unstable sort (faster, doesn't preserve order)
    let mut unstable = data.clone();
    unstable.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    println!("After sort_unstable: {:?}", unstable);

    // Performance comparison
    println!("\n=== Performance: Stable vs Unstable ===\n");

    let n = 100_000;
    let mut data1: Vec<i32> = (0..n).rev().collect();
    let mut data2 = data1.clone();

    let start = std::time::Instant::now();
    data1.sort();
    let stable_time = start.elapsed();

    let start = std::time::Instant::now();
    data2.sort_unstable();
    let unstable_time = start.elapsed();

    println!("Sorting {} elements (reversed):", n);
    println!("  sort (stable):   {:?}", stable_time);
    println!("  sort_unstable:   {:?}", unstable_time);

    // Sorting floats (handling NaN)
    println!("\n=== Sorting Floats (Handling NaN) ===\n");

    fn sort_large_dataset(data: &mut [f64]) {
        // sort_unstable is faster for primitive types
        data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    }

    let mut floats = vec![3.14, 1.0, f64::NAN, 2.71, f64::INFINITY, -1.0];
    println!("Before: {:?}", floats);

    sort_large_dataset(&mut floats);
    println!("After:  {:?}", floats);

    // Partial sort (sort_by with is_sorted)
    println!("\n=== Checking If Already Sorted ===\n");

    let sorted = vec![1, 2, 3, 4, 5];
    let unsorted = vec![1, 3, 2, 4, 5];

    println!("{:?} is sorted: {}", sorted, sorted.is_sorted());
    println!("{:?} is sorted: {}", unsorted, unsorted.is_sorted());

    println!("\n=== Key Points ===");
    println!("1. sort_by_key for simple single-field sorts");
    println!("2. sort_by with then_with for multi-level sorts");
    println!("3. sort_unstable is faster when order doesn't matter");
    println!("4. Use partial_cmp for floats (handles NaN)");
    println!("5. is_sorted() checks without modifying");
}
