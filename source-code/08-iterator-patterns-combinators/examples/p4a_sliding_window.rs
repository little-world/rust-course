//! Pattern 4a: Streaming Algorithms
//! Example: Sliding Window Statistics
//!
//! Run with: cargo run --example p4a_sliding_window

use std::collections::VecDeque;

/// A reusable sliding window that tracks the latest N values.
struct SlidingWindow<T> {
    window: VecDeque<T>,
    capacity: usize,
}

impl<T> SlidingWindow<T> {
    fn new(capacity: usize) -> Self {
        SlidingWindow {
            window: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a value, returning the removed value if window was full.
    fn push(&mut self, value: T) -> Option<T> {
        if self.window.len() == self.capacity {
            let removed = self.window.pop_front();
            self.window.push_back(value);
            removed
        } else {
            self.window.push_back(value);
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.window.iter()
    }

    fn len(&self) -> usize {
        self.window.len()
    }

    fn is_full(&self) -> bool {
        self.window.len() == self.capacity
    }
}

/// Compute sliding window sums efficiently using incremental updates.
fn sliding_window_sum(
    numbers: impl Iterator<Item = i32>,
    window_size: usize,
) -> impl Iterator<Item = i32> {
    let mut window = SlidingWindow::new(window_size);
    let mut sum = 0;

    numbers.filter_map(move |num| {
        if let Some(old) = window.push(num) {
            // Window was full, update sum incrementally
            sum = sum - old + num;
            Some(sum)
        } else {
            // Window not yet full
            sum += num;
            if window.is_full() {
                Some(sum)
            } else {
                None
            }
        }
    })
}

/// Sliding window average.
fn sliding_window_avg(
    numbers: impl Iterator<Item = f64>,
    window_size: usize,
) -> impl Iterator<Item = f64> {
    let mut window = SlidingWindow::new(window_size);
    let mut sum = 0.0;
    let ws = window_size as f64;

    numbers.filter_map(move |num| {
        if let Some(old) = window.push(num) {
            sum = sum - old + num;
            Some(sum / ws)
        } else {
            sum += num;
            if window.is_full() {
                Some(sum / ws)
            } else {
                None
            }
        }
    })
}

/// Sliding window maximum (naive O(n*k) version for clarity).
fn sliding_window_max(
    numbers: impl Iterator<Item = i32>,
    window_size: usize,
) -> impl Iterator<Item = i32> {
    let mut window = SlidingWindow::new(window_size);

    numbers.filter_map(move |num| {
        window.push(num);
        if window.is_full() {
            window.iter().copied().max()
        } else {
            None
        }
    })
}

fn main() {
    println!("=== Sliding Window Statistics ===\n");

    // Sliding window sum
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Numbers: {:?}", numbers);

    let sums: Vec<_> = sliding_window_sum(numbers.iter().copied(), 3).collect();
    println!("Window size 3 sums: {:?}", sums);
    // [6, 9, 12, 15, 18, 21, 24, 27]
    // [1+2+3, 2+3+4, 3+4+5, ...]

    println!("\n=== How Incremental Update Works ===");
    println!("Window [1,2,3] -> sum = 6");
    println!("Add 4, remove 1: sum = 6 - 1 + 4 = 9");
    println!("Add 5, remove 2: sum = 9 - 2 + 5 = 12");
    println!("...");
    println!("O(1) per update instead of O(window_size)!");

    println!("\n=== Sliding Window Average ===");
    let data: Vec<f64> = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0];
    let avgs: Vec<_> = sliding_window_avg(data.iter().copied(), 3).collect();
    println!("Data: {:?}", [10.0, 20.0, 30.0, 40.0, 50.0, 60.0]);
    println!("Window size 3 averages: {:?}", avgs);
    // [20.0, 30.0, 40.0, 50.0]

    println!("\n=== Sliding Window Maximum ===");
    let values = vec![1, 3, -1, -3, 5, 3, 6, 7];
    let maxes: Vec<_> = sliding_window_max(values.iter().copied(), 3).collect();
    println!("Values: {:?}", [1, 3, -1, -3, 5, 3, 6, 7]);
    println!("Window size 3 maximums: {:?}", maxes);
    // [3, 3, 5, 5, 6, 7]

    println!("\n=== Using the SlidingWindow Struct ===");
    let mut window: SlidingWindow<char> = SlidingWindow::new(3);
    for c in ['a', 'b', 'c', 'd', 'e'] {
        let removed = window.push(c);
        let contents: Vec<_> = window.iter().collect();
        println!(
            "Push '{}': removed {:?}, window = {:?}",
            c, removed, contents
        );
    }

    println!("\n=== Memory Efficiency ===");
    println!("SlidingWindow stores only {} elements", 3);
    println!("Can process infinite streams with constant memory");

    println!("\n=== Key Points ===");
    println!("1. VecDeque provides O(1) push/pop at both ends");
    println!("2. Incremental updates give O(1) amortized per element");
    println!("3. Return removed value for caller to update aggregates");
    println!("4. Memory is O(window_size), not O(stream_length)");
}
