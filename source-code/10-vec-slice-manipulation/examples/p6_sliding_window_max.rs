//! Pattern 6: Advanced Slice Patterns
//! Example: Sliding Window Maximum with Deque
//!
//! Run with: cargo run --example p6_sliding_window_max

use std::collections::VecDeque;

fn main() {
    println!("=== Sliding Window Maximum with Deque ===\n");

    // O(n) sliding window maximum using monotonic deque
    fn sliding_window_max(values: &[i32], win_size: usize) -> Vec<i32> {
        let mut result = Vec::new();
        let mut deque: VecDeque<usize> = VecDeque::new();

        for (i, &value) in values.iter().enumerate() {
            // Remove elements outside window
            while deque.front().map_or(false, |&idx| i >= win_size && idx <= i - win_size) {
                deque.pop_front();
            }

            // Remove smaller elements from back (they can never be maximum)
            while deque.back().map_or(false, |&idx| values[idx] < value) {
                deque.pop_back();
            }

            deque.push_back(i);

            // Window is complete
            if i >= win_size - 1 {
                result.push(values[*deque.front().unwrap()]);
            }
        }

        result
    }

    let values = vec![1, 3, -1, -3, 5, 3, 6, 7];
    let win_size = 3;

    println!("Values: {:?}", values);
    println!("Window size: {}", win_size);

    let maxes = sliding_window_max(&values, win_size);
    println!("\nSliding windows and their maximums:");
    for (i, &max) in maxes.iter().enumerate() {
        let window: Vec<_> = values[i..i+win_size].to_vec();
        println!("  Window {:?} -> max = {}", window, max);
    }

    // Sliding window minimum
    println!("\n=== Sliding Window Minimum ===\n");

    fn sliding_window_min(values: &[i32], win_size: usize) -> Vec<i32> {
        let mut result = Vec::new();
        let mut deque: VecDeque<usize> = VecDeque::new();

        for (i, &value) in values.iter().enumerate() {
            while deque.front().map_or(false, |&idx| i >= win_size && idx <= i - win_size) {
                deque.pop_front();
            }

            // For minimum, remove LARGER elements
            while deque.back().map_or(false, |&idx| values[idx] > value) {
                deque.pop_back();
            }

            deque.push_back(i);

            if i >= win_size - 1 {
                result.push(values[*deque.front().unwrap()]);
            }
        }

        result
    }

    let mins = sliding_window_min(&values, win_size);
    println!("Sliding window minimums: {:?}", mins);

    // Naive O(n*k) comparison
    println!("\n=== Complexity Comparison ===\n");

    fn sliding_window_max_naive(values: &[i32], win_size: usize) -> Vec<i32> {
        values.windows(win_size)
            .map(|window| *window.iter().max().unwrap())
            .collect()
    }

    // Generate large dataset
    let large_values: Vec<i32> = (0..100_000)
        .map(|i| ((i * 17) % 1000) as i32)
        .collect();
    let win_size = 100;

    // Naive approach
    let start = std::time::Instant::now();
    let _maxes_naive = sliding_window_max_naive(&large_values, win_size);
    let naive_time = start.elapsed();

    // Optimized approach
    let start = std::time::Instant::now();
    let _maxes_opt = sliding_window_max(&large_values, win_size);
    let opt_time = start.elapsed();

    println!("Processing {} elements with window size {}:", large_values.len(), win_size);
    println!("  Naive O(n*k): {:?}", naive_time);
    println!("  Deque O(n):   {:?}", opt_time);
    println!("  Speedup:      {:.1}x", naive_time.as_secs_f64() / opt_time.as_secs_f64());

    // Generic sliding window statistic
    println!("\n=== Generic Sliding Window Statistics ===\n");

    fn sliding_window_stat<T, F, R>(
        values: &[T],
        win_size: usize,
        stat_fn: F,
    ) -> Vec<R>
    where
        T: Clone,
        F: Fn(&[T]) -> R,
    {
        values.windows(win_size)
            .map(|window| stat_fn(window))
            .collect()
    }

    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    println!("Values: {:?}", values);

    let means: Vec<f64> = sliding_window_stat(&values, 3, |w| {
        w.iter().sum::<f64>() / w.len() as f64
    });
    println!("3-window means: {:?}", means);

    let ranges: Vec<f64> = sliding_window_stat(&values, 3, |w| {
        let (min, max) = w.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &x| {
            (min.min(x), max.max(x))
        });
        max - min
    });
    println!("3-window ranges: {:?}", ranges);

    println!("\n=== Key Points ===");
    println!("1. Monotonic deque maintains candidates in order");
    println!("2. O(n) total - each element enters/exits deque once");
    println!("3. Front always has window maximum/minimum");
    println!("4. Remove elements outside window from front");
    println!("5. Remove dominated elements from back");
}
