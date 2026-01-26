//! Pattern 1: Capacity Management
//! Example: Track Amortized Growth
//!
//! Run with: cargo run --example p1_growable_buffer

/// A wrapper that tracks allocation behavior during vector growth.
struct GrowableBuffer<T> {
    data: Vec<T>,
    allocations: usize,
}

impl<T> GrowableBuffer<T> {
    fn new() -> Self {
        GrowableBuffer {
            data: Vec::new(),
            allocations: 0,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        GrowableBuffer {
            data: Vec::with_capacity(capacity),
            allocations: if capacity > 0 { 1 } else { 0 },
        }
    }

    fn push(&mut self, value: T) {
        let old_cap = self.data.capacity();
        self.data.push(value);
        let new_cap = self.data.capacity();

        if new_cap > old_cap {
            self.allocations += 1;
        }
    }

    fn stats(&self) -> (usize, usize, usize) {
        (self.data.len(), self.data.capacity(), self.allocations)
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn allocations(&self) -> usize {
        self.allocations
    }
}

fn main() {
    println!("=== Tracking Vector Growth ===\n");

    // Without pre-allocation
    println!("=== Growth Without Pre-allocation ===\n");
    let mut buffer: GrowableBuffer<i32> = GrowableBuffer::new();

    let checkpoints = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

    for &target in &checkpoints {
        while buffer.len() < target {
            buffer.push(buffer.len() as i32);
        }
        let (len, cap, allocs) = buffer.stats();
        println!(
            "  At {} elements: capacity={:4}, allocations={}",
            len, cap, allocs
        );
    }

    // With pre-allocation
    println!("\n=== Growth With Pre-allocation ===\n");
    let mut buffer: GrowableBuffer<i32> = GrowableBuffer::with_capacity(1024);

    for &target in &checkpoints {
        while buffer.len() < target {
            buffer.push(buffer.len() as i32);
        }
        let (len, cap, allocs) = buffer.stats();
        println!(
            "  At {} elements: capacity={:4}, allocations={}",
            len, cap, allocs
        );
    }

    // Demonstrate growth pattern
    println!("\n=== Vector Growth Pattern ===\n");
    println!("Rust vectors typically double capacity when full:");

    let mut vec: Vec<i32> = Vec::new();
    let mut growth_log = Vec::new();

    for i in 0..100 {
        let old_cap = vec.capacity();
        vec.push(i);
        let new_cap = vec.capacity();

        if new_cap != old_cap {
            growth_log.push((i + 1, old_cap, new_cap));
        }
    }

    println!("  {:>6} {:>12} {:>12}", "Items", "Old Cap", "New Cap");
    for (items, old, new) in growth_log {
        println!("  {:>6} {:>12} {:>12}", items, old, new);
    }

    // Calculate total copies
    println!("\n=== Amortized Cost Analysis ===\n");
    let n = 1000;
    let mut vec: Vec<i32> = Vec::new();
    let mut total_copies = 0;

    for i in 0..n {
        let old_len = vec.len();
        let old_cap = vec.capacity();
        vec.push(i);

        if vec.capacity() != old_cap {
            // All existing elements were copied
            total_copies += old_len;
        }
    }

    println!("Building {} elements without pre-allocation:", n);
    println!("  Total element copies: {}", total_copies);
    println!("  Average copies per element: {:.2}", total_copies as f64 / n as f64);
    println!("\nWith pre-allocation: 0 copies!");

    println!("\n=== Key Points ===");
    println!("1. Track allocations to identify optimization opportunities");
    println!("2. Without pre-allocation: O(log n) allocations, O(n) total copies");
    println!("3. With pre-allocation: 1 allocation, 0 copies during growth");
    println!("4. Use this diagnostic pattern to find hot paths");
}
