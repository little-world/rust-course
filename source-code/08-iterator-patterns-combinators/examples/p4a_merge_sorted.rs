//! Pattern 4a: Streaming Algorithms
//! Example: Streaming Merge of Sorted Iterators
//!
//! Run with: cargo run --example p4a_merge_sorted

/// Merge two sorted iterators into a single sorted stream.
/// Uses from_fn with buffered values for comparison.
fn merge_sorted<T: Ord>(
    mut a: impl Iterator<Item = T>,
    mut b: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let mut a_next = a.next();
    let mut b_next = b.next();

    std::iter::from_fn(move || match (&a_next, &b_next) {
        (Some(a_val), Some(b_val)) => {
            if a_val <= b_val {
                let result = a_next.take();
                a_next = a.next();
                result
            } else {
                let result = b_next.take();
                b_next = b.next();
                result
            }
        }
        (Some(_), None) => {
            let result = a_next.take();
            a_next = a.next();
            result
        }
        (None, Some(_)) => {
            let result = b_next.take();
            b_next = b.next();
            result
        }
        (None, None) => None,
    })
}

/// K-way merge using a heap (for merging many sorted sequences).
fn k_way_merge<T: Ord>(iterators: Vec<impl Iterator<Item = T>>) -> impl Iterator<Item = T> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    // Create peekable iterators and track which ones have values
    let mut iters: Vec<_> = iterators
        .into_iter()
        .map(|i| i.peekable())
        .collect();

    // Initialize heap with (value, iterator_index)
    let mut heap: BinaryHeap<Reverse<(T, usize)>> = BinaryHeap::new();

    // Load initial values
    for (idx, iter) in iters.iter_mut().enumerate() {
        if let Some(val) = iter.next() {
            heap.push(Reverse((val, idx)));
        }
    }

    std::iter::from_fn(move || {
        let Reverse((val, idx)) = heap.pop()?;

        // Refill from the same iterator
        if let Some(next_val) = iters[idx].next() {
            heap.push(Reverse((next_val, idx)));
        }

        Some(val)
    })
}

fn main() {
    println!("=== Streaming Merge of Sorted Iterators ===\n");

    // Two-way merge
    let a = vec![1, 3, 5, 7, 9];
    let b = vec![2, 4, 6, 8, 10];
    println!("List A: {:?}", a);
    println!("List B: {:?}", b);

    let merged: Vec<_> = merge_sorted(a.into_iter(), b.into_iter()).collect();
    println!("Merged: {:?}", merged);
    assert_eq!(merged, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    println!("\n=== Unequal Length Merge ===");
    let short = vec![2, 5];
    let long = vec![1, 3, 4, 6, 7, 8];
    println!("Short: {:?}", short);
    println!("Long: {:?}", long);

    let merged2: Vec<_> = merge_sorted(short.into_iter(), long.into_iter()).collect();
    println!("Merged: {:?}", merged2);

    println!("\n=== K-Way Merge ===");
    let lists = vec![
        vec![1, 5, 9],
        vec![2, 6, 10],
        vec![3, 7, 11],
        vec![4, 8, 12],
    ];
    println!("Lists:");
    for (i, list) in lists.iter().enumerate() {
        println!("  {}: {:?}", i, list);
    }

    let iters: Vec<_> = lists.into_iter().map(|v| v.into_iter()).collect();
    let k_merged: Vec<_> = k_way_merge(iters).collect();
    println!("K-way merged: {:?}", k_merged);

    println!("\n=== How Two-Way Merge Works ===");
    println!("1. Buffer one value from each iterator");
    println!("2. Compare buffered values");
    println!("3. Yield smaller, refill that buffer");
    println!("4. Repeat until both exhausted");

    println!("\n=== How K-Way Merge Works ===");
    println!("1. Use min-heap to track smallest from each iterator");
    println!("2. Pop minimum from heap, yield it");
    println!("3. Refill heap from same iterator that yielded min");
    println!("4. Repeat until heap empty");

    println!("\n=== Complexity ===");
    println!("Two-way merge: O(n + m) where n, m are list lengths");
    println!("K-way merge: O(N log k) where N = total elements, k = number of lists");

    println!("\n=== Use Cases ===");
    println!("1. External merge sort (merging sorted file chunks)");
    println!("2. Merging log files by timestamp");
    println!("3. Combining sorted database results");
    println!("4. Time-series data fusion");

    println!("\n=== Key Points ===");
    println!("1. Buffer head of each iterator for comparison");
    println!("2. O(n + m) time, O(1) extra space for two-way");
    println!("3. Heap gives O(log k) comparison for k-way");
    println!("4. Input must be sorted for correct output");
}
