//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Chunked Processing
//!
//! Run with: cargo run --example p4b_parallel_chunks

use rayon::prelude::*;

/// Process data in chunks, with each chunk handled by a separate thread.
fn parallel_chunk_processing(data: &[u8], chunk_size: usize) -> Vec<u32> {
    data.par_chunks(chunk_size)
        .map(|chunk| chunk.iter().map(|&b| b as u32).sum())
        .collect()
}

/// Parallel checksum calculation per chunk.
fn parallel_checksums(data: &[u8], chunk_size: usize) -> Vec<u8> {
    data.par_chunks(chunk_size)
        .map(|chunk| chunk.iter().fold(0u8, |acc, &b| acc.wrapping_add(b)))
        .collect()
}

/// Find maximum in each chunk.
fn parallel_chunk_max(data: &[i32], chunk_size: usize) -> Vec<i32> {
    data.par_chunks(chunk_size)
        .map(|chunk| *chunk.iter().max().unwrap_or(&i32::MIN))
        .collect()
}

/// Process mutable chunks in parallel.
fn parallel_mutate_chunks(data: &mut [i32], chunk_size: usize) {
    data.par_chunks_mut(chunk_size)
        .for_each(|chunk| {
            for x in chunk.iter_mut() {
                *x *= 2;
            }
        });
}

fn main() {
    println!("=== Parallel Chunked Processing ===\n");

    // Usage: process data in chunks across multiple threads
    let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sums = parallel_chunk_processing(&data, 3);
    println!("Data: {:?}", data);
    println!("Chunk size 3, sums: {:?}", sums);
    // [6, 15, 24, 10] (chunks: [1,2,3], [4,5,6], [7,8,9], [10])

    println!("\n=== Why Chunks? ===");
    println!("1. Better cache locality - each thread works on contiguous memory");
    println!("2. Reduced synchronization overhead");
    println!("3. More predictable work distribution");

    println!("\n=== Parallel Checksums ===");
    let data2: Vec<u8> = (0..20).collect();
    let checksums = parallel_checksums(&data2, 5);
    println!("Data: 0..20");
    println!("Chunk size 5, checksums: {:?}", checksums);

    println!("\n=== Parallel Chunk Maximum ===");
    let data3 = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7];
    let maxes = parallel_chunk_max(&data3, 4);
    println!("Data: {:?}", data3);
    println!("Chunk size 4, max per chunk: {:?}", maxes);

    println!("\n=== Mutable Chunks ===");
    let mut data4 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Before: {:?}", data4);
    parallel_mutate_chunks(&mut data4, 3);
    println!("After (doubled): {:?}", data4);

    println!("\n=== Exact Chunks ===");
    let data5: Vec<i32> = (1..=12).collect();
    println!("Data: {:?}", data5);

    // par_chunks may have a smaller final chunk
    let regular: Vec<Vec<i32>> = data5.par_chunks(5)
        .map(|c| c.to_vec())
        .collect();
    println!("par_chunks(5): {:?}", regular);
    // [[1,2,3,4,5], [6,7,8,9,10], [11,12]]

    // par_chunks_exact ignores remainder
    let exact: Vec<Vec<i32>> = data5.par_chunks_exact(5)
        .map(|c| c.to_vec())
        .collect();
    println!("par_chunks_exact(5): {:?}", exact);
    // [[1,2,3,4,5], [6,7,8,9,10]]

    println!("\n=== Choosing Chunk Size ===");
    println!("Too small: More overhead, poor cache usage");
    println!("Too large: Poor parallelism");
    println!("");
    println!("Rule of thumb: chunk_size = data.len() / (num_cpus * 4)");
    println!("Rayon usually handles this automatically!");

    println!("\n=== Key Points ===");
    println!("1. par_chunks() for immutable parallel processing");
    println!("2. par_chunks_mut() for in-place modification");
    println!("3. Better cache locality than interleaved access");
    println!("4. Each chunk is processed by one thread");
}
