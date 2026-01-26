//! Pattern 3: Advanced Iterator Composition
//! Example: Scan for Cumulative Operations
//!
//! Run with: cargo run --example p3_scan

/// Compute cumulative sum (prefix sums) using scan.
fn cumulative_sum(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .scan(0, |state, &x| {
            *state += x;
            Some(*state)
        })
        .collect()
}

/// Compute running maximum.
fn running_max(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .scan(i32::MIN, |max, &x| {
            *max = (*max).max(x);
            Some(*max)
        })
        .collect()
}

/// Track running average as (sum, count, average).
fn running_average(numbers: &[f64]) -> Vec<f64> {
    numbers
        .iter()
        .scan((0.0_f64, 0_usize), |state, &x| {
            state.0 += x;
            state.1 += 1;
            Some(state.0 / state.1 as f64)
        })
        .collect()
}

/// Exponential moving average.
fn exponential_moving_average(numbers: &[f64], alpha: f64) -> Vec<f64> {
    let mut first = true;
    numbers
        .iter()
        .scan(0.0, |ema, &x| {
            if first {
                first = false;
                *ema = x;
            } else {
                *ema = alpha * x + (1.0 - alpha) * *ema;
            }
            Some(*ema)
        })
        .collect()
}

/// Detect changes: output only when value differs from previous.
fn detect_changes<T: Clone + PartialEq>(items: &[T]) -> Vec<(usize, T)> {
    items
        .iter()
        .enumerate()
        .scan(None, |prev, (i, item)| {
            let result = if prev.as_ref() != Some(item) {
                Some(Some((i, item.clone())))
            } else {
                Some(None) // Still iterating, but no change
            };
            *prev = Some(item.clone());
            result
        })
        .flatten()
        .collect()
}

fn main() {
    println!("=== Scan for Cumulative Operations ===\n");

    // Usage: compute running totals (prefix sums)
    let sums = cumulative_sum(&[1, 2, 3, 4]);
    println!("cumulative_sum([1, 2, 3, 4]) = {:?}", sums);
    // [1, 3, 6, 10]
    assert_eq!(sums, vec![1, 3, 6, 10]);

    println!("\n=== Running Maximum ===");
    let maxes = running_max(&[3, 1, 4, 1, 5, 9, 2, 6]);
    println!("running_max([3,1,4,1,5,9,2,6]) = {:?}", maxes);
    // [3, 3, 4, 4, 5, 9, 9, 9]

    println!("\n=== Running Average ===");
    let avgs = running_average(&[10.0, 20.0, 30.0, 40.0]);
    println!("running_average([10, 20, 30, 40]) = {:?}", avgs);
    // [10.0, 15.0, 20.0, 25.0]

    println!("\n=== Exponential Moving Average ===");
    let data = vec![10.0, 12.0, 11.0, 14.0, 15.0, 13.0, 16.0];
    let ema = exponential_moving_average(&data, 0.3);
    println!("Data: {:?}", data);
    println!("EMA (alpha=0.3): {:?}", ema);

    println!("\n=== Difference Between scan and fold ===");
    let numbers = [1, 2, 3, 4, 5];

    // fold produces ONE final value
    let sum: i32 = numbers.iter().fold(0, |acc, &x| acc + x);
    println!("fold: {} (single final value)", sum);

    // scan produces MANY intermediate values
    let running: Vec<i32> = numbers.iter()
        .scan(0, |acc, &x| { *acc += x; Some(*acc) })
        .collect();
    println!("scan: {:?} (all intermediate values)", running);

    println!("\n=== Detect Changes ===");
    let signal = ['a', 'a', 'b', 'b', 'b', 'c', 'a', 'a'];
    let changes = detect_changes(&signal);
    println!("Signal: {:?}", signal);
    println!("Changes: {:?}", changes);
    // [(0, 'a'), (2, 'b'), (5, 'c'), (6, 'a')]

    println!("\n=== Key Points ===");
    println!("1. scan threads mutable state through iteration");
    println!("2. Unlike fold, scan yields intermediate results");
    println!("3. Perfect for running totals, prefix sums, moving averages");
    println!("4. State can be any type (numbers, tuples, etc.)");
}
