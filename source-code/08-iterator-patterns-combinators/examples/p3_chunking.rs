//! Pattern 3: Advanced Iterator Composition
//! Example: Chunking with Stateful Iteration
//!
//! Run with: cargo run --example p3_chunking

/// Process items in chunks using std::iter::from_fn.
/// The closure captures mutable state and drains chunk_size elements each call.
fn process_in_chunks<T>(
    items: Vec<T>,
    chunk_size: usize,
) -> impl Iterator<Item = Vec<T>> {
    let mut items = items;
    std::iter::from_fn(move || {
        if items.is_empty() {
            None
        } else {
            let drain_end = chunk_size.min(items.len());
            Some(items.drain(..drain_end).collect())
        }
    })
}

/// Alternative: chunk by predicate (start new chunk when predicate is true)
fn chunk_by<T, F>(items: Vec<T>, mut should_split: F) -> impl Iterator<Item = Vec<T>>
where
    F: FnMut(&T) -> bool,
{
    let mut items = items.into_iter().peekable();

    std::iter::from_fn(move || {
        if items.peek().is_none() {
            return None;
        }

        let mut chunk = vec![items.next().unwrap()];
        while let Some(item) = items.peek() {
            if should_split(item) {
                break;
            }
            chunk.push(items.next().unwrap());
        }
        Some(chunk)
    })
}

fn main() {
    println!("=== Chunking with Stateful Iteration ===\n");

    // Usage: process items in batches of specified size
    println!("Processing [1,2,3,4,5,6,7] in chunks of 2:");
    for batch in process_in_chunks(vec![1, 2, 3, 4, 5, 6, 7], 2) {
        println!("  Batch: {:?}", batch);
    }

    println!("\nProcessing [1,2,3,4,5] in chunks of 3:");
    for batch in process_in_chunks(vec![1, 2, 3, 4, 5], 3) {
        println!("  Batch: {:?}", batch);
    }

    println!("\n=== Using from_fn for Stateful Generation ===");
    // Generate squares until exceeding a limit
    let mut n = 1;
    let squares: Vec<_> = std::iter::from_fn(|| {
        let sq = n * n;
        if sq > 100 {
            None
        } else {
            n += 1;
            Some(sq)
        }
    })
    .collect();
    println!("Squares up to 100: {:?}", squares);

    println!("\n=== Chunk by Predicate ===");
    // Split at numbers divisible by 5
    let numbers = vec![1, 2, 3, 5, 6, 7, 10, 11, 12, 15, 16];
    println!("Numbers: {:?}", numbers);
    println!("Chunking at multiples of 5:");
    for chunk in chunk_by(numbers, |&x| x % 5 == 0) {
        println!("  Chunk: {:?}", chunk);
    }

    // Split at uppercase letters (simulating paragraph breaks)
    println!("\n=== Text Chunking Example ===");
    let words = vec!["hello", "world", "Next", "paragraph", "here", "Another", "section"];
    println!("Words: {:?}", words);
    println!("Chunking at capitalized words:");
    for chunk in chunk_by(words, |s| s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)) {
        println!("  Chunk: {:?}", chunk);
    }

    println!("\n=== Key Points ===");
    println!("1. from_fn creates iterator from stateful closure");
    println!("2. Closure owns its state (via move)");
    println!("3. Return None to signal end of iteration");
    println!("4. Perfect for custom chunking/batching logic");
}
