//! Pattern 2: Work Partitioning Strategies
//!
//! Run with: cargo run --bin p2_work_partitioning

use rayon::prelude::*;
use std::time::Instant;

fn chunk_size_comparison() {
    let data: Vec<i32> = (0..1_000_000).collect();

    // Default chunking (Rayon decides)
    let start = Instant::now();
    let sum1: i32 = data.par_iter().sum();
    let default_time = start.elapsed();

    // Custom chunk size (too small - more overhead)
    let start = Instant::now();
    let sum2: i32 = data.par_chunks(100).map(|chunk| chunk.iter().sum::<i32>()).sum();
    let small_chunk_time = start.elapsed();

    // Custom chunk size (balanced)
    let start = Instant::now();
    let sum3: i32 = data.par_chunks(10_000).map(|chunk| chunk.iter().sum::<i32>()).sum();
    let balanced_chunk_time = start.elapsed();

    println!("Default: {:?}", default_time);
    println!("Small chunks (100): {:?}", small_chunk_time);
    println!("Balanced chunks (10k): {:?}", balanced_chunk_time);

    assert_eq!(sum1, sum2);
    assert_eq!(sum2, sum3);
}

fn work_splitting_strategies() {
    let data: Vec<i32> = (0..100_000).collect();

    // Strategy 1: Equal splits (good for uniform work)
    let chunk_size = data.len() / rayon::current_num_threads();
    let result1: Vec<i32> = data
        .par_chunks(chunk_size.max(1))
        .flat_map(|chunk| chunk.par_iter().map(|&x| x * x))
        .collect();

    // Strategy 2: Adaptive (good for non-uniform work)
    let result2: Vec<i32> = data.par_iter().map(|&x| x * x).collect();

    assert_eq!(result1.len(), result2.len());
    println!("Equal splits: {} items", result1.len());
    println!("Adaptive: {} items", result2.len());
}

struct Matrix {
    data: Vec<f64>,
    rows: usize,
    cols: usize,
}

impl Matrix {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            data: vec![0.0; rows * cols],
            rows,
            cols,
        }
    }

    fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }

    // Parallel matrix multiplication with blocking for cache efficiency
    fn multiply_blocked(&self, other: &Matrix, block_size: usize) -> Matrix {
        assert_eq!(self.cols, other.rows);

        let mut result = Matrix::new(self.rows, other.cols);

        // Partition work by output blocks
        let row_blocks: Vec<usize> = (0..self.rows).step_by(block_size).collect();
        let col_blocks: Vec<usize> = (0..other.cols).step_by(block_size).collect();

        row_blocks.par_iter().for_each(|&row_start| {
            for &col_start in &col_blocks {
                // Process block
                let row_end = (row_start + block_size).min(self.rows);
                let col_end = (col_start + block_size).min(other.cols);

                for i in row_start..row_end {
                    for j in col_start..col_end {
                        let mut sum = 0.0;
                        for k in 0..self.cols {
                            sum += self.get(i, k) * other.get(k, j);
                        }
                        unsafe {
                            let ptr = result.data.as_ptr() as *mut f64;
                            *ptr.add(i * result.cols + j) = sum;
                        }
                    }
                }
            }
        });

        result
    }
}

fn dynamic_load_balancing() {
    // Simulate irregular workload
    let work_items: Vec<usize> = (0..1000).map(|i| i % 100).collect();

    let start = Instant::now();

    // Rayon automatically balances work through work stealing
    let results: Vec<usize> = work_items
        .par_iter()
        .map(|&work| {
            // Simulate variable work
            let mut sum = 0;
            for _ in 0..work {
                sum += 1;
            }
            sum
        })
        .collect();

    println!("Dynamic balancing: {:?}", start.elapsed());
    println!("Total work: {}", results.iter().sum::<usize>());
}

fn grain_size_tuning() {
    let data: Vec<i32> = (0..1_000_000).collect();

    for grain_size in [100, 1_000, 10_000, 100_000] {
        let start = Instant::now();

        let _sum: i32 = data
            .par_chunks(grain_size)
            .map(|chunk| chunk.iter().sum::<i32>())
            .sum();

        println!("Grain size {}: {:?}", grain_size, start.elapsed());
    }
}

fn main() {
    println!("=== Chunk Size Comparison ===\n");
    chunk_size_comparison();

    println!("\n=== Work Splitting Strategies ===\n");
    work_splitting_strategies();

    println!("\n=== Matrix Multiplication with Blocking ===\n");
    let a = Matrix::new(100, 100);
    let b = Matrix::new(100, 100);
    let start = Instant::now();
    let _result = a.multiply_blocked(&b, 32);
    println!("Matrix multiply (100x100): {:?}", start.elapsed());

    println!("\n=== Dynamic Load Balancing ===\n");
    dynamic_load_balancing();

    println!("\n=== Grain Size Tuning ===\n");
    grain_size_tuning();
}
