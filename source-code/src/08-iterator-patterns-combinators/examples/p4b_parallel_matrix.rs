//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Matrix Operations (Nested Iteration with flat_map)
//!
//! Run with: cargo run --example p4b_parallel_matrix

use rayon::prelude::*;
use std::time::Instant;

/// Parallel matrix multiplication.
/// Precompute columns for cache-friendly access.
fn parallel_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if a.is_empty() || b.is_empty() || a[0].len() != b.len() {
        return vec![];
    }

    // Transpose b for better cache locality: b_cols[j] = column j of b
    let b_cols: Vec<Vec<f64>> = (0..b[0].len())
        .map(|col| b.iter().map(|row| row[col]).collect())
        .collect();

    // Each row of result computed in parallel
    a.par_iter()
        .map(|row| {
            b_cols
                .iter()
                .map(|col| row.iter().zip(col).map(|(a, b)| a * b).sum())
                .collect()
        })
        .collect()
}

/// Sequential matrix multiply for comparison.
fn sequential_matrix_multiply(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if a.is_empty() || b.is_empty() || a[0].len() != b.len() {
        return vec![];
    }

    let b_cols: Vec<Vec<f64>> = (0..b[0].len())
        .map(|col| b.iter().map(|row| row[col]).collect())
        .collect();

    a.iter()
        .map(|row| {
            b_cols
                .iter()
                .map(|col| row.iter().zip(col).map(|(a, b)| a * b).sum())
                .collect()
        })
        .collect()
}

/// Parallel element-wise operations.
fn parallel_element_wise<F>(matrix: &[Vec<f64>], op: F) -> Vec<Vec<f64>>
where
    F: Fn(f64) -> f64 + Sync,
{
    matrix
        .par_iter()
        .map(|row| row.iter().map(|&x| op(x)).collect())
        .collect()
}

/// Parallel matrix addition.
fn parallel_matrix_add(a: &[Vec<f64>], b: &[Vec<f64>]) -> Vec<Vec<f64>> {
    a.par_iter()
        .zip(b.par_iter())
        .map(|(row_a, row_b)| {
            row_a.iter().zip(row_b.iter()).map(|(a, b)| a + b).collect()
        })
        .collect()
}

fn print_matrix(name: &str, m: &[Vec<f64>]) {
    println!("{}:", name);
    for row in m {
        println!("  {:?}", row);
    }
}

fn main() {
    println!("=== Parallel Matrix Operations ===\n");

    // Small example
    let a = vec![
        vec![1.0, 2.0],
        vec![3.0, 4.0],
    ];
    let b = vec![
        vec![5.0, 6.0],
        vec![7.0, 8.0],
    ];

    print_matrix("A", &a);
    print_matrix("B", &b);

    let c = parallel_matrix_multiply(&a, &b);
    print_matrix("A × B", &c);
    // [1*5+2*7, 1*6+2*8] = [19, 22]
    // [3*5+4*7, 3*6+4*8] = [43, 50]

    // Element-wise operations
    println!("\n=== Element-wise Operations ===");
    let squared = parallel_element_wise(&a, |x| x * x);
    print_matrix("A² (element-wise)", &squared);

    // Matrix addition
    println!("\n=== Matrix Addition ===");
    let sum = parallel_matrix_add(&a, &b);
    print_matrix("A + B", &sum);

    // Performance comparison
    println!("\n=== Performance Comparison ===");
    let size = 200;
    let large_a: Vec<Vec<f64>> = (0..size)
        .map(|i| (0..size).map(|j| (i * size + j) as f64).collect())
        .collect();
    let large_b: Vec<Vec<f64>> = (0..size)
        .map(|i| (0..size).map(|j| ((i + j) % 100) as f64).collect())
        .collect();

    let start = Instant::now();
    let _seq_result = sequential_matrix_multiply(&large_a, &large_b);
    let seq_time = start.elapsed();
    println!("Sequential {}x{} multiply: {:?}", size, size, seq_time);

    let start = Instant::now();
    let _par_result = parallel_matrix_multiply(&large_a, &large_b);
    let par_time = start.elapsed();
    println!("Parallel {}x{} multiply: {:?}", size, size, par_time);

    println!("Speedup: {:.2}x", seq_time.as_secs_f64() / par_time.as_secs_f64());

    println!("\n=== Why Transpose for Multiplication? ===");
    println!("Without transpose: access b[k][j] - jumps across rows");
    println!("With transpose:    access b_cols[j][k] - sequential in memory");
    println!("");
    println!("Better cache locality = faster execution!");

    println!("\n=== Key Points ===");
    println!("1. Transpose columns for cache-friendly dot products");
    println!("2. par_iter() on rows parallelizes by row");
    println!("3. Inner loops can be sequential (work per row)");
    println!("4. Significant speedup for large matrices");
}
