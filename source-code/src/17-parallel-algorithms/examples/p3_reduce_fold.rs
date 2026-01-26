//! Pattern 3: Parallel Reduce and Fold
//!
//! Run with: cargo run --bin p3_reduce_fold

use rayon::prelude::*;
use std::collections::HashMap;

fn simple_reductions() {
    let numbers: Vec<i64> = (1..=1_000_000).collect();

    // Sum
    let sum: i64 = numbers.par_iter().sum();
    println!("Sum: {}", sum);

    // Min/Max
    let min = numbers.par_iter().min().unwrap();
    let max = numbers.par_iter().max().unwrap();
    println!("Min: {}, Max: {}", min, max);

    // Product (be careful of overflow!)
    let small_numbers: Vec<i64> = (1..=10).collect();
    let product: i64 = small_numbers.par_iter().product();
    println!("Product: {}", product);
}

fn custom_reduce() {
    let numbers: Vec<i32> = (1..=100).collect();

    // Custom reduction: concatenate all numbers
    let concatenated = numbers
        .par_iter()
        .map(|n| n.to_string())
        .reduce(|| String::new(), |a, b| format!("{},{}", a, b));

    println!("Concatenated (first 50 chars): {}", &concatenated[..50.min(concatenated.len())]);

    // Find element closest to target
    let target = 42;
    let closest = numbers
        .par_iter()
        .reduce(
            || &numbers[0],
            |a, b| {
                if (a - target).abs() < (b - target).abs() {
                    a
                } else {
                    b
                }
            },
        );

    println!("Closest to {}: {}", target, closest);
}

fn fold_vs_reduce() {
    let numbers: Vec<i32> = (1..=1000).collect();

    // fold: provide identity and combine function
    let sum_fold = numbers.par_iter().fold(
        || 0, // Identity function
        |acc, &x| acc + x, // Fold function
    ).sum::<i32>(); // Reduce the folded results

    // reduce: simpler but less flexible
    let sum_reduce = numbers.par_iter().sum::<i32>();

    assert_eq!(sum_fold, sum_reduce);
    println!("Sum: {}", sum_fold);
}

fn fold_with_accumulator() {
    let numbers: Vec<i32> = (1..=100).collect();

    // Collect statistics in one pass
    #[derive(Default)]
    struct Stats {
        count: usize,
        sum: i64,
        min: i32,
        max: i32,
    }

    let stats = numbers
        .par_iter()
        .fold(
            || Stats {
                count: 0,
                sum: 0,
                min: i32::MAX,
                max: i32::MIN,
            },
            |mut acc, &x| {
                acc.count += 1;
                acc.sum += x as i64;
                acc.min = acc.min.min(x);
                acc.max = acc.max.max(x);
                acc
            },
        )
        .reduce(
            || Stats::default(),
            |a, b| Stats {
                count: a.count + b.count,
                sum: a.sum + b.sum,
                min: a.min.min(b.min),
                max: a.max.max(b.max),
            },
        );

    println!("Count: {}", stats.count);
    println!("Average: {:.2}", stats.sum as f64 / stats.count as f64);
    println!("Min: {}, Max: {}", stats.min, stats.max);
}

fn parallel_histogram(data: Vec<i32>) -> HashMap<i32, usize> {
    data.par_iter()
        .fold(
            || HashMap::new(),
            |mut map, &value| {
                *map.entry(value).or_insert(0) += 1;
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (key, count) in b {
                    *a.entry(key).or_insert(0) += count;
                }
                a
            },
        )
}

fn word_frequency_parallel(text: String) -> HashMap<String, usize> {
    text.par_split_whitespace()
        .fold(
            || HashMap::new(),
            |mut map, word| {
                *map.entry(word.to_lowercase()).or_insert(0) += 1;
                map
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a, b| {
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            },
        )
}

fn parallel_variance(numbers: &[f64]) -> (f64, f64) {
    // Two-pass algorithm (more numerically stable)
    let mean = numbers.par_iter().sum::<f64>() / numbers.len() as f64;

    let variance = numbers
        .par_iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>()
        / numbers.len() as f64;

    (mean, variance)
}

fn parallel_merge_reduce(mut chunks: Vec<Vec<i32>>) -> Vec<i32> {
    while chunks.len() > 1 {
        chunks = chunks
            .par_chunks(2)
            .map(|pair| {
                if pair.len() == 2 {
                    merge(&pair[0], &pair[1])
                } else {
                    pair[0].clone()
                }
            })
            .collect();
    }

    chunks.into_iter().next().unwrap_or_default()
}

fn merge(a: &[i32], b: &[i32]) -> Vec<i32> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            result.push(a[i]);
            i += 1;
        } else {
            result.push(b[j]);
            j += 1;
        }
    }

    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    result
}

fn main() {
    println!("=== Simple Reductions ===\n");
    simple_reductions();

    println!("\n=== Custom Reduce ===\n");
    custom_reduce();

    println!("\n=== Fold vs Reduce ===\n");
    fold_vs_reduce();

    println!("\n=== Fold with Accumulator ===\n");
    fold_with_accumulator();

    println!("\n=== Parallel Histogram ===\n");
    let data: Vec<i32> = (0..10000).map(|i| i % 100).collect();
    let histogram = parallel_histogram(data);
    println!("Histogram buckets: {}", histogram.len());
    println!("Bucket 50: {}", histogram.get(&50).unwrap_or(&0));

    println!("\n=== Word Frequency ===\n");
    let text = "the quick brown fox jumps over the lazy dog the fox".to_string();
    let freq = word_frequency_parallel(text);
    for (word, count) in freq.iter().take(5) {
        println!("{}: {}", word, count);
    }

    println!("\n=== Parallel Variance ===\n");
    let numbers: Vec<f64> = (1..=100).map(|x| x as f64).collect();
    let (mean, variance) = parallel_variance(&numbers);
    println!("Mean: {:.2}, Variance: {:.2}, StdDev: {:.2}", mean, variance, variance.sqrt());

    println!("\n=== Parallel Merge Reduce ===\n");
    let chunks: Vec<Vec<i32>> = vec![
        vec![1, 4, 7],
        vec![2, 5, 8],
        vec![3, 6, 9],
        vec![0, 10, 11],
    ];
    let merged = parallel_merge_reduce(chunks);
    println!("Merged: {:?}", merged);
}
