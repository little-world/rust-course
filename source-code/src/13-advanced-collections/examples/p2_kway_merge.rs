//! Pattern 2: BinaryHeap and Priority Queues
//! K-way Merge and Median Tracking
//!
//! Run with: cargo run --example p2_kway_merge

use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};

//======================================
// K-way merge: merge k sorted iterators
//======================================
struct KWayMerge<T> {
    heap: BinaryHeap<MergeItem<T>>,
}

struct MergeItem<T> {
    value: T,
    source_id: usize,
}

impl<T: Ord> Ord for MergeItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap behavior
        other.value.cmp(&self.value)
    }
}

impl<T: Ord> PartialOrd for MergeItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Eq for MergeItem<T> {}

impl<T: Ord> PartialEq for MergeItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Ord + Clone> KWayMerge<T> {
    fn merge(lists: Vec<Vec<T>>) -> Vec<T> {
        let mut heap = BinaryHeap::new();
        let mut iters: Vec<_> = lists
            .into_iter()
            .map(|v| v.into_iter())
            .collect();

        // Initialize heap with first element from each list
        for (id, iter) in iters.iter_mut().enumerate() {
            if let Some(value) = iter.next() {
                heap.push(MergeItem {
                    value,
                    source_id: id,
                });
            }
        }

        let mut result = Vec::new();

        while let Some(item) = heap.pop() {
            result.push(item.value);

            // Get next element from same source
            if let Some(value) = iters[item.source_id].next() {
                heap.push(MergeItem {
                    value,
                    source_id: item.source_id,
                });
            }
        }

        result
    }
}

//=======================================
// Running median tracker using two heaps
//=======================================
struct MedianTracker {
    lower_half: BinaryHeap<i32>,              // max-heap
    upper_half: BinaryHeap<Reverse<i32>>,     // min-heap
}

impl MedianTracker {
    fn new() -> Self {
        Self {
            lower_half: BinaryHeap::new(),
            upper_half: BinaryHeap::new(),
        }
    }

    fn add(&mut self, num: i32) {
        // Add to appropriate heap
        let add_to_lower = self.lower_half.is_empty()
            || num <= *self.lower_half.peek().unwrap();
        if add_to_lower {
            self.lower_half.push(num);
        } else {
            self.upper_half.push(Reverse(num));
        }

        // Rebalance: ensure size difference <= 1
        if self.lower_half.len() > self.upper_half.len() + 1 {
            if let Some(val) = self.lower_half.pop() {
                self.upper_half.push(Reverse(val));
            }
        } else if self.upper_half.len() > self.lower_half.len() {
            if let Some(Reverse(val)) = self.upper_half.pop() {
                self.lower_half.push(val);
            }
        }
    }

    fn median(&self) -> Option<f64> {
        if self.lower_half.is_empty() && self.upper_half.is_empty() {
            return None;
        }

        if self.lower_half.len() > self.upper_half.len() {
            Some(*self.lower_half.peek().unwrap() as f64)
        } else if self.upper_half.len() > self.lower_half.len() {
            Some(self.upper_half.peek().unwrap().0 as f64)
        } else {
            let lower = *self.lower_half.peek().unwrap() as f64;
            let upper = self.upper_half.peek().unwrap().0 as f64;
            Some((lower + upper) / 2.0)
        }
    }

    fn count(&self) -> usize {
        self.lower_half.len() + self.upper_half.len()
    }
}

//==========================================
// Real-world: External sort for large files
//==========================================
struct ExternalSorter {
    chunk_size: usize,
}

impl ExternalSorter {
    fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    fn sort(&self, data: Vec<i32>) -> Vec<i32> {
        // Phase 1: Sort chunks
        let chunks: Vec<Vec<i32>> = data
            .chunks(self.chunk_size)
            .map(|chunk| {
                let mut sorted = chunk.to_vec();
                sorted.sort();
                sorted
            })
            .collect();

        // Phase 2: K-way merge
        KWayMerge::merge(chunks)
    }
}

fn main() {
    println!("=== K-Way Merge ===\n");

    let lists = vec![
        vec![1, 4, 7, 10],
        vec![2, 5, 8, 11],
        vec![3, 6, 9, 12],
    ];

    let merged = KWayMerge::merge(lists.clone());
    println!("Input lists: {:?}", lists);
    println!("Merged: {:?}", merged);

    println!("\n=== Running Median ===\n");

    let mut tracker = MedianTracker::new();

    for num in [5, 15, 1, 3, 8, 7, 9, 2] {
        tracker.add(num);
        println!("Added {}: median = {:.1}", num, tracker.median().unwrap());
    }

    println!("\n=== External Sort ===\n");

    let data: Vec<i32> = (0..20).rev().collect();
    println!("Unsorted: {:?}", data);

    let sorter = ExternalSorter::new(5);
    let sorted = sorter.sort(data);
    println!("Sorted: {:?}", sorted);

    println!("\n=== Key Points ===");
    println!("1. K-way merge: O(n log k) time complexity");
    println!("2. Median tracker: O(log n) per insertion");
    println!("3. Two heaps maintain balance for O(1) median");
    println!("4. External sort for data larger than memory");
}
