//! Pattern 1: VecDeque and Ring Buffers
//! Deque-Based Sliding Window Maximum
//!
//! Run with: cargo run --example p1_sliding_window_max

use std::collections::VecDeque;

struct SlidingWindowMax {
    deque: VecDeque<(usize, i32)>, // (index, value)
    window_size: usize,
}

impl SlidingWindowMax {
    fn new(window_size: usize) -> Self {
        Self {
            deque: VecDeque::new(),
            window_size,
        }
    }

    fn add(&mut self, index: usize, value: i32) -> Option<i32> {
        // Remove elements outside window
        while let Some(&(idx, _)) = self.deque.front() {
            if idx + self.window_size <= index {
                self.deque.pop_front();
            } else {
                break;
            }
        }

        // Remove elements smaller than current
        while let Some(&(_, val)) = self.deque.back() {
            if val <= value {
                self.deque.pop_back();
            } else {
                break;
            }
        }

        self.deque.push_back((index, value));

        // Return max if window is full
        if index >= self.window_size - 1 {
            self.deque.front().map(|(_, val)| *val)
        } else {
            None
        }
    }

    fn max_in_windows(arr: &[i32], k: usize) -> Vec<i32> {
        let mut solver = Self::new(k);
        let mut result = Vec::new();

        for (i, &val) in arr.iter().enumerate() {
            if let Some(max) = solver.add(i, val) {
                result.push(max);
            }
        }

        result
    }
}

// Real-world application: Stock price analysis
struct StockAnalyzer {
    prices: Vec<f64>,
}

impl StockAnalyzer {
    fn new(prices: Vec<f64>) -> Self {
        Self { prices }
    }

    fn resistance_levels(&self, window_size: usize) -> Vec<f64> {
        self.sliding_max(window_size)
    }

    fn support_levels(&self, window_size: usize) -> Vec<f64> {
        self.sliding_min(window_size)
    }

    fn sliding_max(&self, window_size: usize) -> Vec<f64> {
        let mut deque = VecDeque::new();
        let mut result = Vec::new();

        for (i, &price) in self.prices.iter().enumerate() {
            // Remove old elements
            while let Some(&idx) = deque.front() {
                if idx + window_size <= i {
                    deque.pop_front();
                } else {
                    break;
                }
            }

            // Maintain decreasing order
            while let Some(&idx) = deque.back() {
                if self.prices[idx] <= price {
                    deque.pop_back();
                } else {
                    break;
                }
            }

            deque.push_back(i);

            if i >= window_size - 1 {
                result.push(self.prices[*deque.front().unwrap()]);
            }
        }

        result
    }

    fn sliding_min(&self, window_size: usize) -> Vec<f64> {
        let mut deque = VecDeque::new();
        let mut result = Vec::new();

        for (i, &price) in self.prices.iter().enumerate() {
            while let Some(&idx) = deque.front() {
                if idx + window_size <= i {
                    deque.pop_front();
                } else {
                    break;
                }
            }

            // Maintain increasing order (opposite of max)
            while let Some(&idx) = deque.back() {
                if self.prices[idx] >= price {
                    deque.pop_back();
                } else {
                    break;
                }
            }

            deque.push_back(i);

            if i >= window_size - 1 {
                result.push(self.prices[*deque.front().unwrap()]);
            }
        }

        result
    }

    fn volatility(&self, window_size: usize) -> Vec<f64> {
        let max_values = self.sliding_max(window_size);
        let min_values = self.sliding_min(window_size);

        max_values
            .iter()
            .zip(min_values.iter())
            .map(|(max, min)| max - min)
            .collect()
    }
}

fn main() {
    println!("=== Sliding Window Maximum ===\n");

    let arr = vec![1, 3, -1, -3, 5, 3, 6, 7];
    let k = 3;

    let result = SlidingWindowMax::max_in_windows(&arr, k);
    println!("Array: {:?}", arr);
    println!("Window size: {}", k);
    println!("Maximums: {:?}", result);

    println!("\n=== Stock Analysis ===\n");

    let prices = vec![
        100.0, 102.0, 101.0, 105.0, 103.0,
        108.0, 107.0, 110.0, 109.0, 112.0,
    ];

    let analyzer = StockAnalyzer::new(prices.clone());

    println!("Prices: {:?}", prices);
    println!("\nResistance (5d): {:?}", analyzer.resistance_levels(5));
    println!("Support (5d): {:?}", analyzer.support_levels(5));
    println!("Volatility (5d): {:?}", analyzer.volatility(5));

    println!("\n=== Key Points ===");
    println!("1. O(n) time complexity for entire array");
    println!("2. Each element added/removed at most once");
    println!("3. Deque maintains decreasing order for max");
    println!("4. Space: O(k) bounded by window size");
}
