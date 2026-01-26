//! Pattern 1: Custom Iterators and IntoIterator
//! Example: Implementing IntoIterator
//!
//! Run with: cargo run --example p1_ring_buffer

/// A simple ring buffer that wraps a Vec.
/// We implement IntoIterator to enable for-loop syntax.
struct RingBuffer<T> {
    data: Vec<T>,
}

impl<T> RingBuffer<T> {
    fn new() -> Self {
        RingBuffer { data: Vec::new() }
    }

    fn push(&mut self, item: T) {
        self.data.push(item);
    }
}

// For `for item in my_buffer` (consumes the buffer)
impl<T> IntoIterator for RingBuffer<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// For `for item in &my_buffer` (borrows the buffer)
impl<'a, T> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

// For `for item in &mut my_buffer` (mutably borrows)
impl<'a, T> IntoIterator for &'a mut RingBuffer<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

fn main() {
    println!("=== IntoIterator for Custom Collections ===\n");

    // Create and populate a buffer
    let mut buffer = RingBuffer::new();
    buffer.push(1);
    buffer.push(2);
    buffer.push(3);

    // Borrow iteration: for item in &buffer
    println!("Iterating by reference (&buffer):");
    for item in &buffer {
        println!("  {}", item);
    }

    // Mutable iteration: for item in &mut buffer
    println!("\nMutating items (&mut buffer):");
    for item in &mut buffer {
        *item *= 10;
    }

    // Verify mutation
    println!("After mutation:");
    for item in &buffer {
        println!("  {}", item);
    }

    // Consuming iteration: for item in buffer
    println!("\nConsuming iteration (into_iter):");
    for item in buffer {
        println!("  {}", item);
    }
    // buffer is now moved, can't be used

    println!("\n=== The Three Forms of IntoIterator ===");
    println!("1. impl IntoIterator for T        -> for item in collection (consumes)");
    println!("2. impl IntoIterator for &T       -> for item in &collection (borrows)");
    println!("3. impl IntoIterator for &mut T   -> for item in &mut collection (mut borrow)");

    println!("\n=== Usage with Standard Library ===");
    let buffer2 = RingBuffer { data: vec![1, 2, 3] };
    // Works with any IntoIterator-compatible function
    let sum: i32 = buffer2.into_iter().sum();
    println!("Sum of buffer: {}", sum);
}
