//! Pattern 3: Chunking and Windowing
//! Example: Parallel Processing with Chunks
//!
//! Run with: cargo run --example p3_parallel_chunks

use rayon::prelude::*;
use std::thread;

fn main() {
    println!("=== Parallel Processing with Chunks ===\n");

    // Manual parallel sum with threads
    println!("=== Manual Thread-Based Parallel Sum ===\n");

    fn parallel_sum_manual(data: &[i64]) -> i64 {
        let num_threads = num_cpus();
        let chunk_size = (data.len() + num_threads - 1) / num_threads;

        let handles: Vec<_> = data.chunks(chunk_size)
            .map(|chunk| {
                let chunk = chunk.to_vec();
                thread::spawn(move || chunk.iter().sum::<i64>())
            })
            .collect();

        handles.into_iter()
            .map(|h| h.join().unwrap())
            .sum()
    }

    let data: Vec<i64> = (0..1_000_000).collect();

    let start = std::time::Instant::now();
    let sum_sequential: i64 = data.iter().sum();
    let seq_time = start.elapsed();

    let start = std::time::Instant::now();
    let sum_parallel = parallel_sum_manual(&data);
    let par_time = start.elapsed();

    println!("Summing 1M elements:");
    println!("  Sequential: {} ({:?})", sum_sequential, seq_time);
    println!("  Parallel:   {} ({:?})", sum_parallel, par_time);

    // Rayon parallel chunks
    println!("\n=== Rayon Parallel Chunks ===\n");

    fn parallel_transform(data: &mut [f32]) {
        data.par_chunks_mut(1024)
            .for_each(|chunk| {
                for value in chunk {
                    *value = value.sqrt();
                }
            });
    }

    let mut data: Vec<f32> = (1..100_001).map(|i| i as f32).collect();

    let start = std::time::Instant::now();
    parallel_transform(&mut data);
    let transform_time = start.elapsed();

    println!("Parallel sqrt on 100K elements: {:?}", transform_time);
    println!("Sample results: [{:.2}, {:.2}, {:.2}, ..., {:.2}]",
        data[0], data[1], data[2], data[data.len()-1]);

    // Parallel map-reduce
    println!("\n=== Parallel Map-Reduce ===\n");

    let data: Vec<i64> = (0..1_000_000).collect();

    let start = std::time::Instant::now();
    let sum_of_squares: i64 = data.par_iter()
        .map(|&x| x * x)
        .sum();
    let rayon_time = start.elapsed();

    let start = std::time::Instant::now();
    let sum_of_squares_seq: i64 = data.iter()
        .map(|&x| x * x)
        .sum();
    let seq_time = start.elapsed();

    println!("Sum of squares (1M elements):");
    println!("  Sequential: {} ({:?})", sum_of_squares_seq, seq_time);
    println!("  Rayon:      {} ({:?})", sum_of_squares, rayon_time);

    // Parallel filtering
    println!("\n=== Parallel Filtering ===\n");

    let data: Vec<i32> = (0..1_000_000).collect();

    let start = std::time::Instant::now();
    let evens: Vec<i32> = data.par_iter()
        .filter(|&&x| x % 2 == 0)
        .copied()
        .collect();
    let par_time = start.elapsed();

    let start = std::time::Instant::now();
    let evens_seq: Vec<i32> = data.iter()
        .filter(|&&x| x % 2 == 0)
        .copied()
        .collect();
    let seq_time = start.elapsed();

    println!("Filtering even numbers from 1M:");
    println!("  Sequential: {} items ({:?})", evens_seq.len(), seq_time);
    println!("  Parallel:   {} items ({:?})", evens.len(), par_time);

    // Chunk size matters
    println!("\n=== Chunk Size Impact ===\n");

    let data: Vec<i32> = (0..1_000_000).collect();

    for chunk_size in [100, 1000, 10000, 100000] {
        let start = std::time::Instant::now();
        let _sum: i64 = data.par_chunks(chunk_size)
            .map(|chunk| chunk.iter().map(|&x| x as i64).sum::<i64>())
            .sum();
        let time = start.elapsed();
        println!("  Chunk size {:>6}: {:?}", chunk_size, time);
    }

    println!("\n=== Key Points ===");
    println!("1. par_chunks_mut enables parallel in-place transformation");
    println!("2. Rayon handles thread pool and work stealing");
    println!("3. Chunk size affects performance (too small = overhead, too large = poor balance)");
    println!("4. Use rayon::par_iter() for simple parallel operations");
    println!("5. Manual threading for fine-grained control");
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
