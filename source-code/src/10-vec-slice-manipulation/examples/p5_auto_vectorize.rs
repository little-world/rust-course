//! Pattern 5: SIMD Operations
//! Example: Auto-Vectorization
//!
//! Run with: cargo run --example p5_auto_vectorize
//!
//! For best results, compile with:
//! RUSTFLAGS="-C target-cpu=native" cargo run --release --example p5_auto_vectorize

fn main() {
    println!("=== Auto-Vectorization ===\n");
    println!("Note: For best performance, compile with:");
    println!("  RUSTFLAGS=\"-C target-cpu=native\" cargo run --release\n");

    // Simple loop that auto-vectorizes
    println!("=== Simple Scaling (Auto-Vectorizable) ===\n");

    fn scale_values(data: &mut [f32], scale: f32) {
        // Compiler can auto-vectorize this loop
        for value in data {
            *value *= scale;
        }
    }

    let mut data: Vec<f32> = (0..16).map(|i| i as f32).collect();
    println!("Before: {:?}", data);

    scale_values(&mut data, 2.0);
    println!("After scaling by 2.0: {:?}", data);

    // Vector addition (auto-vectorizes well)
    println!("\n=== Vector Addition ===\n");

    fn add_vectors(a: &[f32], b: &[f32], result: &mut [f32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());

        for i in 0..a.len() {
            result[i] = a[i] + b[i];
        }
    }

    let a: Vec<f32> = (0..8).map(|i| i as f32).collect();
    let b: Vec<f32> = (0..8).map(|i| (i * 2) as f32).collect();
    let mut result = vec![0.0f32; 8];

    add_vectors(&a, &b, &mut result);
    println!("a:      {:?}", a);
    println!("b:      {:?}", b);
    println!("a + b:  {:?}", result);

    // Element-wise operations
    println!("\n=== Element-wise Operations ===\n");

    fn apply_elementwise(data: &mut [f32]) {
        for value in data {
            *value = value.sqrt() * 2.0 + 1.0;
        }
    }

    let mut data: Vec<f32> = vec![1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0];
    println!("Before: {:?}", data);

    apply_elementwise(&mut data);
    println!("After sqrt(x)*2+1: {:?}", data);

    // Performance comparison
    println!("\n=== Performance Test ===\n");

    let n = 10_000_000;
    let mut data1: Vec<f32> = (0..n).map(|i| i as f32 * 0.001).collect();
    let mut data2 = data1.clone();

    // Simple multiplication (vectorizable)
    let start = std::time::Instant::now();
    for value in &mut data1 {
        *value *= 2.0;
    }
    let mult_time = start.elapsed();

    // More complex operation (may not vectorize)
    let start = std::time::Instant::now();
    for value in &mut data2 {
        *value = if *value > 0.5 { *value * 2.0 } else { *value };
    }
    let branch_time = start.elapsed();

    println!("Processing {} elements:", n);
    println!("  Simple multiply:  {:?}", mult_time);
    println!("  With branching:   {:?}", branch_time);
    println!("\n(Branching often prevents auto-vectorization)");

    // Branchless alternative
    println!("\n=== Branchless Operations ===\n");

    fn conditional_scale_branchy(data: &mut [f32], threshold: f32) {
        for value in data {
            if *value > threshold {
                *value *= 2.0;
            }
        }
    }

    fn conditional_scale_branchless(data: &mut [f32], threshold: f32) {
        for value in data {
            // Branchless: multiplier is 2.0 if > threshold, else 1.0
            let multiplier = 1.0 + (*value > threshold) as i32 as f32;
            *value *= multiplier;
        }
    }

    let mut data1: Vec<f32> = (0..1_000_000).map(|i| (i % 100) as f32 / 100.0).collect();
    let mut data2 = data1.clone();

    let start = std::time::Instant::now();
    conditional_scale_branchy(&mut data1, 0.5);
    let branchy_time = start.elapsed();

    let start = std::time::Instant::now();
    conditional_scale_branchless(&mut data2, 0.5);
    let branchless_time = start.elapsed();

    println!("Conditional scaling of 1M elements:");
    println!("  With branches:    {:?}", branchy_time);
    println!("  Branchless:       {:?}", branchless_time);

    // Reduction operations
    println!("\n=== Reduction Operations ===\n");

    fn sum_simple(data: &[f32]) -> f32 {
        let mut sum = 0.0;
        for &value in data {
            sum += value;
        }
        sum
    }

    fn sum_iterator(data: &[f32]) -> f32 {
        data.iter().sum()
    }

    let data: Vec<f32> = (0..1_000_000).map(|i| i as f32 * 0.001).collect();

    let start = std::time::Instant::now();
    let sum1 = sum_simple(&data);
    let simple_time = start.elapsed();

    let start = std::time::Instant::now();
    let sum2 = sum_iterator(&data);
    let iter_time = start.elapsed();

    println!("Summing 1M elements:");
    println!("  Manual loop: {} ({:?})", sum1, simple_time);
    println!("  Iterator:    {} ({:?})", sum2, iter_time);

    println!("\n=== Tips for Auto-Vectorization ===");
    println!("1. Use simple loops without branches");
    println!("2. Avoid data dependencies between iterations");
    println!("3. Use fixed-size arrays when possible");
    println!("4. Compile with -C target-cpu=native");
    println!("5. Use --release mode for optimizations");
    println!("6. Check assembly with cargo-show-asm");
}
