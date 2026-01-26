//! Pattern 6: Advanced Lifetime Patterns
//! Example: Streaming Iterators with GAT
//!
//! Run with: cargo run --example p6_streaming_iterator

// Streaming iterator trait using Generic Associated Types (GAT).
// Each yielded item borrows from the iterator, unlike std::iter::Iterator.
trait StreamingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// Window iterator: yields overlapping slices
struct WindowIter<'data> {
    data: &'data [i32],
    window_size: usize,
    position: usize,
}

impl<'data> WindowIter<'data> {
    fn new(data: &'data [i32], window_size: usize) -> Self {
        WindowIter {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<'data> StreamingIterator for WindowIter<'data> {
    // The yielded slice borrows from the iterator
    type Item<'a> = &'a [i32] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position + self.window_size <= self.data.len() {
            let start = self.position;
            let end = self.position + self.window_size;
            let window = &self.data[start..end];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}

// Line iterator: yields lines from a buffer
struct LineIter<'data> {
    data: &'data str,
    position: usize,
}

impl<'data> LineIter<'data> {
    fn new(data: &'data str) -> Self {
        LineIter { data, position: 0 }
    }
}

impl<'data> StreamingIterator for LineIter<'data> {
    type Item<'a> = &'a str where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position >= self.data.len() {
            return None;
        }

        let remaining = &self.data[self.position..];
        let line_end = remaining.find('\n').unwrap_or(remaining.len());
        let line = &remaining[..line_end];

        self.position += line_end + 1; // Skip the newline
        Some(line)
    }
}

// Chunk iterator: yields non-overlapping chunks
struct ChunkIter<'data, T> {
    data: &'data [T],
    chunk_size: usize,
    position: usize,
}

impl<'data, T> ChunkIter<'data, T> {
    fn new(data: &'data [T], chunk_size: usize) -> Self {
        ChunkIter {
            data,
            chunk_size,
            position: 0,
        }
    }
}

impl<'data, T> StreamingIterator for ChunkIter<'data, T> {
    type Item<'a> = &'a [T] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position >= self.data.len() {
            return None;
        }

        let start = self.position;
        let end = (self.position + self.chunk_size).min(self.data.len());
        let chunk = &self.data[start..end];
        self.position = end;
        Some(chunk)
    }
}

// Helper function to consume a streaming iterator
fn consume_streaming<I: StreamingIterator>(mut iter: I)
where
    for<'a> I::Item<'a>: std::fmt::Debug,
{
    while let Some(item) = iter.next() {
        println!("  {:?}", item);
    }
}

fn main() {
    println!("=== Streaming Iterators with GAT ===\n");

    println!("=== Window Iterator ===");
    // Usage: GAT enables iterator that yields refs tied to iterator's lifetime.
    let data = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut iter = WindowIter::new(&data, 3);

    println!("Sliding windows of size 3 over {:?}:", data);
    while let Some(window) = iter.next() {
        println!("  {:?}", window);
    }

    println!("\n=== Line Iterator ===");
    let text = "Hello\nWorld\nFrom\nRust";
    let mut lines = LineIter::new(text);

    println!("Lines from text:");
    while let Some(line) = lines.next() {
        println!("  '{}'", line);
    }

    println!("\n=== Chunk Iterator ===");
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut chunks = ChunkIter::new(&numbers, 3);

    println!("Chunks of size 3 from {:?}:", numbers);
    while let Some(chunk) = chunks.next() {
        println!("  {:?}", chunk);
    }

    println!("\n=== Why Streaming Iterators? ===");
    println!("Standard Iterator:");
    println!("  - Items are owned or have independent lifetimes");
    println!("  - Can collect into Vec, store items, etc.");
    println!("  - Example: vec.iter() yields &T with vec's lifetime");

    println!("\nStreaming Iterator (GAT):");
    println!("  - Items borrow from the iterator itself");
    println!("  - Each item only valid until next() called again");
    println!("  - Enables zero-copy iteration over computed views");

    println!("\n=== Memory Efficiency ===");
    // Streaming iterators don't allocate for each item
    let large_data: Vec<i32> = (0..1000).collect();
    let mut window_iter = WindowIter::new(&large_data, 100);

    let mut count = 0;
    while let Some(_window) = window_iter.next() {
        count += 1;
    }
    println!("Processed {} windows without any allocation!", count);

    println!("\n=== GAT Syntax Explained ===");
    println!("trait StreamingIterator {{");
    println!("    type Item<'a> where Self: 'a;  // GAT with lifetime");
    println!("    fn next(&mut self) -> Option<Self::Item<'_>>;");
    println!("}}");
    println!("\nThe 'a in Item<'a> is tied to each call of next()");
    println!("Self: 'a ensures the iterator outlives the yielded item");

    println!("\n=== Use Cases ===");
    println!("- Sliding windows over data");
    println!("- Parsing without copying");
    println!("- Database cursor iteration");
    println!("- Streaming decompression");
    println!("- Memory-mapped file iteration");
}
