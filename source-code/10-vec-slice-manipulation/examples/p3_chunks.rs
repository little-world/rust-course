//! Pattern 3: Chunking and Windowing
//! Example: Fixed-Size Chunking
//!
//! Run with: cargo run --example p3_chunks

fn main() {
    println!("=== Fixed-Size Chunking ===\n");

    // Basic chunks
    let data: Vec<u8> = (0..20).collect();
    println!("Data (20 bytes): {:?}", data);

    println!("\nChunks of 8:");
    for (i, chunk) in data.chunks(8).enumerate() {
        println!("  Chunk {}: {:?} (len={})", i, chunk, chunk.len());
    }

    // chunks vs chunks_exact
    println!("\n=== chunks vs chunks_exact ===\n");

    println!("chunks(8) - last chunk may be smaller:");
    for chunk in data.chunks(8) {
        println!("  {:?}", chunk);
    }

    println!("\nchunks_exact(8) - only full chunks:");
    let mut iter = data.chunks_exact(8);
    for chunk in iter.by_ref() {
        println!("  {:?}", chunk);
    }
    println!("  Remainder: {:?}", iter.remainder());

    // Mutable chunks
    println!("\n=== Mutable Chunks for In-Place Transformation ===\n");

    fn normalize_batches(data: &mut [f64], batch_size: usize) {
        for chunk in data.chunks_mut(batch_size) {
            let sum: f64 = chunk.iter().sum();
            let mean = sum / chunk.len() as f64;

            for value in chunk {
                *value -= mean;
            }
        }
    }

    let mut values: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    println!("Original: {:?}", values);

    normalize_batches(&mut values, 4);
    println!("After normalizing in batches of 4: {:?}", values);
    println!("(Each batch now has mean ~0)");

    // Batch processing pattern
    println!("\n=== Batch Processing Pattern ===\n");

    #[derive(Debug)]
    struct ProcessedBatch {
        batch_id: usize,
        sum: i32,
        count: usize,
    }

    fn process_batch(batch: &[i32]) -> ProcessedBatch {
        ProcessedBatch {
            batch_id: 0, // Will be set by caller
            sum: batch.iter().sum(),
            count: batch.len(),
        }
    }

    fn process_in_batches(data: &[i32], batch_size: usize) -> Vec<ProcessedBatch> {
        data.chunks(batch_size)
            .enumerate()
            .map(|(i, chunk)| {
                let mut result = process_batch(chunk);
                result.batch_id = i;
                result
            })
            .collect()
    }

    let data: Vec<i32> = (1..=100).collect();
    let batches = process_in_batches(&data, 20);

    println!("Processing {} items in batches of 20:", data.len());
    for batch in &batches {
        println!("  {:?}", batch);
    }

    // Reverse chunking
    println!("\n=== Reverse Chunking (rchunks) ===\n");

    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("Data: {:?}", data);

    println!("rchunks(3) - from end:");
    for (i, chunk) in data.rchunks(3).enumerate() {
        println!("  Chunk {}: {:?}", i, chunk);
    }

    // Splitting into N equal parts
    println!("\n=== Splitting into N Equal Parts ===\n");

    fn split_into_n_parts(data: &[u8], n: usize) -> Vec<&[u8]> {
        let chunk_size = (data.len() + n - 1) / n; // Ceiling division
        data.chunks(chunk_size).collect()
    }

    let data: Vec<u8> = (0..100).collect();
    let parts = split_into_n_parts(&data, 3);

    println!("Splitting 100 elements into 3 parts:");
    for (i, part) in parts.iter().enumerate() {
        println!("  Part {}: {} elements", i, part.len());
    }

    println!("\n=== Key Points ===");
    println!("1. chunks(n) creates non-overlapping segments");
    println!("2. Last chunk may be smaller (use chunks_exact for uniform size)");
    println!("3. chunks_mut enables in-place transformation");
    println!("4. rchunks processes from the end");
    println!("5. Perfect for batch processing and parallel work distribution");
}
