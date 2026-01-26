//! Pattern 5: SIMD Parallelism
//!
//! Run with: cargo run --bin p5_simd

use rayon::prelude::*;
use std::time::Instant;

fn simd_friendly_sum(data: &[f32]) -> f32 {
    // Process 4 elements at a time (compiler can auto-vectorize)
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut sums = [0.0f32; 4];

    for chunk in chunks {
        sums[0] += chunk[0];
        sums[1] += chunk[1];
        sums[2] += chunk[2];
        sums[3] += chunk[3];
    }

    let chunk_sum: f32 = sums.iter().sum();
    let remainder_sum: f32 = remainder.iter().sum();

    chunk_sum + remainder_sum
}

fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), result.len());

    // Compiler can auto-vectorize this
    for i in 0..a.len() {
        result[i] = a[i] + b[i];
    }
}

fn vector_add_parallel(a: &[f32], b: &[f32]) -> Vec<f32> {
    // Combine SIMD and thread parallelism
    a.par_iter()
        .zip(b.par_iter())
        .map(|(&x, &y)| x + y)
        .collect()
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| x * y)
        .sum()
}

fn dot_product_parallel(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    a.par_iter()
        .zip(b.par_iter())
        .map(|(&x, &y)| x * y)
        .sum()
}

fn matrix_multiply_simd(a: &[f32], b: &[f32], result: &mut [f32], n: usize) {
    // Matrix dimensions: n x n
    assert_eq!(a.len(), n * n);
    assert_eq!(b.len(), n * n);
    assert_eq!(result.len(), n * n);

    for i in 0..n {
        for j in 0..n {
            let mut sum = 0.0;

            // Inner loop is SIMD-friendly
            for k in 0..n {
                sum += a[i * n + k] * b[k * n + j];
            }

            result[i * n + j] = sum;
        }
    }
}

fn matrix_multiply_parallel_simd(a: &[f32], b: &[f32], n: usize) -> Vec<f32> {
    let result = vec![0.0f32; n * n];

    // Use interior mutability pattern for parallel writes
    let result_ptr = result.as_ptr() as usize;

    (0..n).into_par_iter().for_each(|i| {
        for j in 0..n {
            let mut sum = 0.0;

            // This loop can be auto-vectorized
            for k in 0..n {
                sum += a[i * n + k] * b[k * n + j];
            }

            unsafe {
                let ptr = result_ptr as *mut f32;
                *ptr.add(i * n + j) = sum;
            }
        }
    });

    result
}

fn blocked_matrix_multiply(a: &[f32], b: &[f32], result: &mut [f32], n: usize, block_size: usize) {
    for i_block in (0..n).step_by(block_size) {
        for j_block in (0..n).step_by(block_size) {
            for k_block in (0..n).step_by(block_size) {
                // Process block
                let i_end = (i_block + block_size).min(n);
                let j_end = (j_block + block_size).min(n);
                let k_end = (k_block + block_size).min(n);

                for i in i_block..i_end {
                    for j in j_block..j_end {
                        let mut sum = result[i * n + j];

                        // Inner loop is SIMD-friendly
                        for k in k_block..k_end {
                            sum += a[i * n + k] * b[k * n + j];
                        }

                        result[i * n + j] = sum;
                    }
                }
            }
        }
    }
}

fn convolve_2d(image: &[f32], kernel: &[f32], width: usize, height: usize, kernel_size: usize) -> Vec<f32> {
    let offset = kernel_size / 2;
    let result = vec![0.0f32; width * height];
    let result_ptr = result.as_ptr() as usize;

    (offset..height - offset).into_par_iter().for_each(|y| {
        for x in offset..width - offset {
            let mut sum = 0.0;

            // Convolution kernel (SIMD-friendly inner loops)
            for ky in 0..kernel_size {
                for kx in 0..kernel_size {
                    let img_y = y + ky - offset;
                    let img_x = x + kx - offset;
                    let img_idx = img_y * width + img_x;
                    let kernel_idx = ky * kernel_size + kx;

                    sum += image[img_idx] * kernel[kernel_idx];
                }
            }

            unsafe {
                let ptr = result_ptr as *mut f32;
                *ptr.add(y * width + x) = sum;
            }
        }
    });

    result
}

fn parallel_sum_simd(data: &[f32]) -> f32 {
    // Split into chunks for parallel processing
    data.par_chunks(1024)
        .map(|chunk| {
            // Each chunk can be SIMD-vectorized
            chunk.iter().sum::<f32>()
        })
        .sum()
}

fn auto_vectorize_examples() {
    let data: Vec<f32> = (0..10000).map(|x| x as f32).collect();

    // Good: Simple map (auto-vectorizes)
    let doubled: Vec<f32> = data.iter().map(|&x| x * 2.0).collect();

    // Good: Zip and map (auto-vectorizes)
    let summed: Vec<f32> = data
        .iter()
        .zip(data.iter())
        .map(|(&a, &b)| a + b)
        .collect();

    // Good: Chunks with fold (auto-vectorizes)
    let chunk_sums: Vec<f32> = data
        .chunks(4)
        .map(|chunk| chunk.iter().sum())
        .collect();

    println!("Doubled first 5: {:?}", &doubled[..5]);
    println!("Summed first 5: {:?}", &summed[..5]);
    println!("Chunk sums first 5: {:?}", &chunk_sums[..5]);
}

fn chunked_operations(data: &[f32]) -> Vec<f32> {
    let mut result = Vec::with_capacity(data.len());

    // Process in SIMD-width chunks
    const SIMD_WIDTH: usize = 8; // Typical for AVX

    for chunk in data.chunks(SIMD_WIDTH) {
        for &value in chunk {
            result.push(value * 2.0 + 1.0);
        }
    }

    result
}

// Bad for SIMD: Array of Structs
#[derive(Copy, Clone)]
struct PointAoS {
    x: f32,
    y: f32,
    z: f32,
}

fn process_aos(points: &[PointAoS]) -> Vec<f32> {
    // Poor vectorization: scattered access
    points.iter().map(|p| p.x + p.y + p.z).collect()
}

// Good for SIMD: Struct of Arrays
struct PointsSoA {
    x: Vec<f32>,
    y: Vec<f32>,
    z: Vec<f32>,
}

impl PointsSoA {
    fn process(&self) -> Vec<f32> {
        // Good vectorization: contiguous access
        self.x
            .iter()
            .zip(self.y.iter())
            .zip(self.z.iter())
            .map(|((&x, &y), &z)| x + y + z)
            .collect()
    }
}

fn benchmark_vectorization() {
    let data: Vec<f32> = (0..10_000_000).map(|x| x as f32).collect();

    // Version 1: Simple loop
    let start = Instant::now();
    let mut result1 = Vec::with_capacity(data.len());
    for &x in &data {
        result1.push(x * 2.0 + 1.0);
    }
    let time1 = start.elapsed();

    // Version 2: Iterator (likely vectorized)
    let start = Instant::now();
    let result2: Vec<f32> = data.iter().map(|&x| x * 2.0 + 1.0).collect();
    let time2 = start.elapsed();

    // Version 3: Parallel + potential vectorization
    let start = Instant::now();
    let result3: Vec<f32> = data.par_iter().map(|&x| x * 2.0 + 1.0).collect();
    let time3 = start.elapsed();

    println!("Simple loop: {:?}", time1);
    println!("Iterator: {:?}", time2);
    println!("Parallel: {:?}", time3);
    println!("Speedup (iter vs loop): {:.2}x", time1.as_secs_f64() / time2.as_secs_f64());
    println!("Speedup (parallel vs iter): {:.2}x", time2.as_secs_f64() / time3.as_secs_f64());
}

fn main() {
    println!("=== SIMD-friendly Sum ===\n");
    let data: Vec<f32> = (0..1_000_000).map(|x| x as f32).collect();
    let sum = simd_friendly_sum(&data);
    println!("Sum: {}", sum);

    println!("\n=== Vector Add ===\n");
    let a: Vec<f32> = (0..1000).map(|x| x as f32).collect();
    let b: Vec<f32> = (0..1000).map(|x| (x * 2) as f32).collect();
    let result = vector_add_parallel(&a, &b);
    println!("First 5 results: {:?}", &result[..5]);

    println!("\n=== Dot Product ===\n");
    let result = dot_product(&a, &b);
    println!("Dot product: {}", result);
    let result_par = dot_product_parallel(&a, &b);
    println!("Dot product (parallel): {}", result_par);

    println!("\n=== Matrix Multiply ===\n");
    let n = 128;
    let a: Vec<f32> = (0..n * n).map(|x| x as f32 * 0.001).collect();
    let b: Vec<f32> = (0..n * n).map(|x| (x * 2) as f32 * 0.001).collect();

    let start = Instant::now();
    let result = matrix_multiply_parallel_simd(&a, &b, n);
    println!("Matrix multiply ({}x{}): {:?}", n, n, start.elapsed());
    println!("Result checksum: {}", result.iter().sum::<f32>());

    println!("\n=== Image Convolution ===\n");
    let width = 256;
    let height = 256;
    let image: Vec<f32> = (0..width * height).map(|x| (x % 256) as f32).collect();
    let kernel = vec![
        1.0/9.0, 1.0/9.0, 1.0/9.0,
        1.0/9.0, 1.0/9.0, 1.0/9.0,
        1.0/9.0, 1.0/9.0, 1.0/9.0,
    ];
    let start = Instant::now();
    let result = convolve_2d(&image, &kernel, width, height, 3);
    println!("Convolution ({}x{}): {:?}", width, height, start.elapsed());

    println!("\n=== Parallel Sum SIMD ===\n");
    let sum = parallel_sum_simd(&data);
    println!("Parallel SIMD sum: {}", sum);

    println!("\n=== Auto-vectorize Examples ===\n");
    auto_vectorize_examples();

    println!("\n=== SoA vs AoS ===\n");
    let points_soa = PointsSoA {
        x: (0..10000).map(|i| i as f32).collect(),
        y: (0..10000).map(|i| (i * 2) as f32).collect(),
        z: (0..10000).map(|i| (i * 3) as f32).collect(),
    };
    let start = Instant::now();
    let sums = points_soa.process();
    println!("SoA processing: {:?}", start.elapsed());
    println!("First 5 sums: {:?}", &sums[..5]);

    println!("\n=== Benchmark Vectorization ===\n");
    benchmark_vectorization();
}
