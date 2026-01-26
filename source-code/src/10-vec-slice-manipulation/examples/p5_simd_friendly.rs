//! Pattern 5: SIMD Operations
//! Example: SIMD-Friendly Chunking
//!
//! Run with: cargo run --example p5_simd_friendly

fn main() {
    println!("=== SIMD-Friendly Chunking ===\n");

    // Manual SIMD-friendly chunking (stable Rust)
    println!("=== Chunked Sum (SIMD-Friendly) ===\n");

    fn sum_bytes(data: &[u8]) -> u64 {
        let mut chunks = data.chunks_exact(8);
        let mut sum = 0u64;

        // Process 8 bytes at a time
        for chunk in chunks.by_ref() {
            for &byte in chunk {
                sum += byte as u64;
            }
        }

        // Handle remainder
        for &byte in chunks.remainder() {
            sum += byte as u64;
        }

        sum
    }

    let data: Vec<u8> = (0..100).collect();
    let sum = sum_bytes(&data);
    let expected: u64 = (0..100u64).sum();

    println!("Sum of 0..100: {} (expected {})", sum, expected);

    // SIMD reduction with multiple accumulators
    println!("\n=== Multiple Accumulator Pattern ===\n");

    fn sum_f32_vectorized(data: &[f32]) -> f32 {
        let mut chunks = data.chunks_exact(8);
        let mut sums = [0.0f32; 8];

        for chunk in chunks.by_ref() {
            for (i, &value) in chunk.iter().enumerate() {
                sums[i] += value;
            }
        }

        let chunk_sum: f32 = sums.iter().sum();
        let remainder_sum: f32 = chunks.remainder().iter().sum();

        chunk_sum + remainder_sum
    }

    let data: Vec<f32> = (1..=1000).map(|i| i as f32).collect();
    let sum = sum_f32_vectorized(&data);
    let expected: f32 = (1..=1000).sum::<i32>() as f32;

    println!("Sum of 1..=1000: {} (expected {})", sum, expected);

    // Performance comparison
    println!("\n=== Performance: Simple vs Vectorized ===\n");

    let data: Vec<f32> = (0..1_000_000).map(|i| i as f32 * 0.001).collect();

    // Simple sum
    let start = std::time::Instant::now();
    let sum1: f32 = data.iter().sum();
    let simple_time = start.elapsed();

    // Vectorized sum
    let start = std::time::Instant::now();
    let sum2 = sum_f32_vectorized(&data);
    let vec_time = start.elapsed();

    println!("Summing 1M f32 values:");
    println!("  Simple iter sum: {} ({:?})", sum1, simple_time);
    println!("  Vectorized sum:  {} ({:?})", sum2, vec_time);

    // Dot product with chunking
    println!("\n=== Dot Product with Chunking ===\n");

    fn dot_product_chunks(a: &[f32], b: &[f32]) -> f32 {
        let mut a_chunks = a.chunks_exact(4);
        let mut b_chunks = b.chunks_exact(4);

        let mut sum = 0.0;

        // Process 4 elements at a time
        for (a_chunk, b_chunk) in a_chunks.by_ref().zip(b_chunks.by_ref()) {
            for i in 0..4 {
                sum += a_chunk[i] * b_chunk[i];
            }
        }

        // Handle remainder
        for (a_val, b_val) in a_chunks.remainder().iter().zip(b_chunks.remainder()) {
            sum += a_val * b_val;
        }

        sum
    }

    let a = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
    let b = vec![2.0f32, 3.0, 4.0, 5.0, 6.0];

    let dot = dot_product_chunks(&a, &b);
    let expected = 1.0*2.0 + 2.0*3.0 + 3.0*4.0 + 4.0*5.0 + 5.0*6.0;

    println!("a = {:?}", a);
    println!("b = {:?}", b);
    println!("Dot product: {} (expected {})", dot, expected);

    // Chunked search
    println!("\n=== Chunked Byte Search ===\n");

    fn find_byte_chunked(haystack: &[u8], needle: u8) -> Option<usize> {
        let mut chunks = haystack.chunks_exact(16);
        let mut offset = 0;

        for chunk in chunks.by_ref() {
            for (j, &byte) in chunk.iter().enumerate() {
                if byte == needle {
                    return Some(offset + j);
                }
            }
            offset += 16;
        }

        chunks.remainder().iter()
            .position(|&b| b == needle)
            .map(|pos| offset + pos)
    }

    let haystack = b"Hello, World! This is a test string for searching.";
    let needle = b'W';

    match find_byte_chunked(haystack, needle) {
        Some(idx) => println!("Found '{}' at index {}",
            needle as char, idx),
        None => println!("'{}' not found", needle as char),
    }

    // Aligned data structures
    println!("\n=== Aligned Data for SIMD ===\n");

    #[repr(align(32))]
    struct AlignedBuffer([f32; 8]);

    fn process_aligned(data: &[AlignedBuffer]) -> f32 {
        data.iter()
            .flat_map(|buf| buf.0.iter())
            .sum()
    }

    let buffers = vec![
        AlignedBuffer([1.0; 8]),
        AlignedBuffer([2.0; 8]),
        AlignedBuffer([3.0; 8]),
    ];

    let sum = process_aligned(&buffers);
    println!("Sum of {} aligned buffers: {}", buffers.len(), sum);
    println!("Alignment of AlignedBuffer: {} bytes",
        std::mem::align_of::<AlignedBuffer>());

    println!("\n=== Key Points ===");
    println!("1. Use chunks_exact for uniform processing");
    println!("2. Multiple accumulators exploit instruction pipelining");
    println!("3. Process 4/8/16 elements at a time (SIMD lane widths)");
    println!("4. Handle remainder separately");
    println!("5. #[repr(align(N))] for aligned data structures");
}
