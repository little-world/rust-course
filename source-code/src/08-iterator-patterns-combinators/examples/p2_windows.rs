//! Pattern 2: Zero-Allocation Iteration
//! Example: Using `windows` for Sliding Window
//!
//! Run with: cargo run --example p2_windows

/// Compute the moving average using sliding windows.
/// Each window is a view into the original data, no copying.
fn moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
    data.windows(window_size)
        .map(|w| w.iter().sum::<f64>() / window_size as f64)
        .collect()
}

/// Detect trends by comparing consecutive windows.
fn detect_trends(data: &[f64], window_size: usize) -> Vec<&'static str> {
    data.windows(window_size)
        .map(|w| w.iter().sum::<f64>() / window_size as f64)
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| {
            if pair[1] > pair[0] {
                "up"
            } else if pair[1] < pair[0] {
                "down"
            } else {
                "flat"
            }
        })
        .collect()
}

/// Find the maximum sum of any window.
fn max_window_sum(data: &[i32], window_size: usize) -> Option<i32> {
    data.windows(window_size)
        .map(|w| w.iter().sum())
        .max()
}

fn main() {
    println!("=== Sliding Windows for Zero-Cost Views ===\n");

    // Usage: compute moving average over sliding windows
    let data = [1.0, 2.0, 3.0, 4.0, 5.0];
    let avgs = moving_average(&data, 2);
    println!("Data: {:?}", data);
    println!("Moving average (window=2): {:?}", avgs);
    // [1.5, 2.5, 3.5, 4.5]

    let avgs3 = moving_average(&data, 3);
    println!("Moving average (window=3): {:?}", avgs3);
    // [2.0, 3.0, 4.0]

    println!("\n=== Understanding windows() ===");
    let numbers = [1, 2, 3, 4, 5];
    println!("Original: {:?}", numbers);
    println!("windows(2):");
    for window in numbers.windows(2) {
        println!("  {:?}", window);
    }
    println!("windows(3):");
    for window in numbers.windows(3) {
        println!("  {:?}", window);
    }

    println!("\n=== Trend Detection ===");
    let stock_prices = [100.0, 102.0, 101.0, 105.0, 108.0, 107.0, 110.0];
    let trends = detect_trends(&stock_prices, 2);
    println!("Prices: {:?}", stock_prices);
    println!("Trends: {:?}", trends);

    println!("\n=== Maximum Subarray Sum (fixed window) ===");
    let values = [1, -2, 3, 4, -1, 2, 1, -5, 4];
    println!("Values: {:?}", values);
    for window_size in 1..=4 {
        let max = max_window_sum(&values, window_size);
        println!("Max sum (window={}): {:?}", window_size, max);
    }

    println!("\n=== Consecutive Differences ===");
    let sequence = [10, 15, 13, 20, 25, 22];
    let diffs: Vec<i32> = sequence
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    println!("Sequence: {:?}", sequence);
    println!("Differences: {:?}", diffs);

    println!("\n=== Key Points ===");
    println!("1. windows(n) yields overlapping slices (views, not copies)");
    println!("2. Perfect for moving averages, convolutions, diff sequences");
    println!("3. Zero allocation - each window borrows from original data");
    println!("4. Number of windows = len - window_size + 1");
}
