//! Pattern 1: Capacity Management
//! Example: Reuse Vectors to Avoid Allocation
//!
//! Run with: cargo run --example p1_reuse_buffer

fn main() {
    println!("=== Reuse Vectors with clear() ===\n");

    // Demonstrate that clear() retains capacity
    let mut buffer: Vec<i32> = Vec::with_capacity(100);
    buffer.extend(0..50);

    println!("Before clear:");
    println!("  len: {}, capacity: {}", buffer.len(), buffer.capacity());

    buffer.clear();

    println!("After clear:");
    println!("  len: {}, capacity: {}", buffer.len(), buffer.capacity());
    println!("  Capacity retained!");

    // Batch processing pattern
    println!("\n=== Batch Processing with Buffer Reuse ===\n");

    #[derive(Debug, Clone)]
    struct Batch {
        items: Vec<i32>,
    }

    fn process_item(item: &i32) -> i32 {
        item * 2
    }

    fn batch_processor(batches: &[Batch]) -> Vec<Vec<i32>> {
        let mut buffer = Vec::with_capacity(1000);
        let mut all_results = Vec::with_capacity(batches.len());

        for (i, batch) in batches.iter().enumerate() {
            let cap_before = buffer.capacity();
            buffer.clear(); // Retains capacity

            for item in &batch.items {
                buffer.push(process_item(item));
            }

            println!(
                "  Batch {}: processed {} items, buffer capacity {} -> {}",
                i,
                batch.items.len(),
                cap_before,
                buffer.capacity()
            );

            // Clone only the used portion
            all_results.push(buffer.clone());
        }

        all_results
    }

    let batches = vec![
        Batch { items: (0..100).collect() },
        Batch { items: (0..200).collect() },
        Batch { items: (0..50).collect() },
        Batch { items: (0..300).collect() },
    ];

    let results = batch_processor(&batches);
    println!("\nProcessed {} batches", results.len());
    println!("Total items: {}", results.iter().map(|r| r.len()).sum::<usize>());

    // Compare with fresh allocation each time
    println!("\n=== Comparison: Reuse vs Fresh Allocation ===\n");

    let iterations = 1000;
    let batch_size = 100;

    // With buffer reuse
    let start = std::time::Instant::now();
    let mut buffer: Vec<i32> = Vec::with_capacity(batch_size);
    for _ in 0..iterations {
        buffer.clear();
        buffer.extend(0..batch_size as i32);
    }
    let reuse_time = start.elapsed();

    // Without buffer reuse
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let buffer: Vec<i32> = (0..batch_size as i32).collect();
        drop(buffer);
    }
    let fresh_time = start.elapsed();

    println!("{} iterations with batch size {}:", iterations, batch_size);
    println!("  Buffer reuse:      {:?}", reuse_time);
    println!("  Fresh allocation:  {:?}", fresh_time);

    println!("\n=== Key Points ===");
    println!("1. clear() sets length to 0 but retains allocated capacity");
    println!("2. Reusing buffers eliminates allocation overhead in loops");
    println!("3. First iteration allocates, subsequent iterations are free");
    println!("4. Use clone() to copy only used portion when needed");
}
