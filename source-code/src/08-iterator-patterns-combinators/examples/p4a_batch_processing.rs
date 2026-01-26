//! Pattern 4a: Streaming Algorithms
//! Example: Buffered Batch Processing
//!
//! Run with: cargo run --example p4a_batch_processing

/// Process items in batches, calling callback for each full batch.
/// Uses std::mem::replace to efficiently swap buffers.
fn process_in_batches<T, F>(
    iter: impl Iterator<Item = T>,
    batch_size: usize,
    mut process_batch: F,
) where
    F: FnMut(Vec<T>),
{
    let mut batch = Vec::with_capacity(batch_size);

    for item in iter {
        batch.push(item);
        if batch.len() == batch_size {
            process_batch(std::mem::replace(
                &mut batch,
                Vec::with_capacity(batch_size),
            ));
        }
    }

    // Process any remaining items
    if !batch.is_empty() {
        process_batch(batch);
    }
}

/// Iterator adapter that yields batches.
fn batched<T>(iter: impl Iterator<Item = T>, batch_size: usize) -> impl Iterator<Item = Vec<T>> {
    let mut iter = iter.peekable();

    std::iter::from_fn(move || {
        if iter.peek().is_none() {
            return None;
        }

        let batch: Vec<T> = iter.by_ref().take(batch_size).collect();
        Some(batch)
    })
}

/// Batch with timeout - yields batch when full OR after timeout.
/// (Simplified version without actual async/timeout for demonstration)
fn batch_with_min_size<T>(
    iter: impl Iterator<Item = T>,
    max_batch_size: usize,
    min_batch_size: usize,
) -> impl Iterator<Item = Vec<T>> {
    let mut iter = iter.peekable();

    std::iter::from_fn(move || {
        if iter.peek().is_none() {
            return None;
        }

        // Collect up to max_batch_size
        let batch: Vec<T> = iter.by_ref().take(max_batch_size).collect();

        // Only yield if we have at least min_batch_size (or stream ended)
        if batch.len() >= min_batch_size || iter.peek().is_none() {
            Some(batch)
        } else {
            // In real code, you might wait for more items
            Some(batch)
        }
    })
}

fn main() {
    println!("=== Buffered Batch Processing ===\n");

    // Using process_in_batches with callback
    println!("=== Callback-based Batching ===");
    let items: Vec<i32> = (1..=10).collect();
    println!("Items: {:?}", items);
    println!("Processing in batches of 3:");

    process_in_batches(items.into_iter(), 3, |batch| {
        println!("  Batch: {:?} (sum: {})", batch, batch.iter().sum::<i32>());
    });

    // Using iterator adapter
    println!("\n=== Iterator-based Batching ===");
    let items2: Vec<i32> = (1..=11).collect();
    println!("Items: {:?}", items2);
    println!("Batched (size 4):");

    for batch in batched(items2.into_iter(), 4) {
        println!("  {:?}", batch);
    }

    // Batch processing with strings
    println!("\n=== String Batch Processing ===");
    let words = vec!["apple", "banana", "cherry", "date", "elderberry", "fig", "grape"];
    println!("Words: {:?}", words);

    let batched_words: Vec<Vec<_>> = batched(words.into_iter(), 2).collect();
    println!("Batched (size 2):");
    for (i, batch) in batched_words.iter().enumerate() {
        println!("  Batch {}: {:?}", i, batch);
    }

    // Practical example: database inserts
    println!("\n=== Practical Example: Simulated DB Inserts ===");
    let records: Vec<_> = (1..=15)
        .map(|i| format!("record_{}", i))
        .collect();

    println!("Inserting {} records in batches of 5:", records.len());
    process_in_batches(records.into_iter(), 5, |batch| {
        println!("  INSERT INTO table VALUES ({:?})", batch);
    });

    // The mem::replace trick
    println!("\n=== The std::mem::replace Trick ===");
    println!("Instead of:");
    println!("  let old = batch.clone();");
    println!("  batch.clear();");
    println!("");
    println!("We use:");
    println!("  let old = std::mem::replace(&mut batch, Vec::new());");
    println!("");
    println!("This moves the old vec out and puts a new empty one in.");
    println!("Zero copying, zero cloning!");

    println!("\n=== Why Batch? ===");
    println!("1. Reduce per-item overhead (network, disk I/O)");
    println!("2. Better throughput for database operations");
    println!("3. Amortize fixed costs across multiple items");
    println!("4. Control memory usage (bounded batch size)");

    println!("\n=== Key Points ===");
    println!("1. process_in_batches for callback-based processing");
    println!("2. batched() iterator adapter for lazy batching");
    println!("3. std::mem::replace swaps without allocation");
    println!("4. Final partial batch must be handled");
}
