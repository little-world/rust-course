//! Pattern 1: Custom Iterators and IntoIterator
//! Example: An Infinite Iterator
//!
//! Run with: cargo run --example p1_fibonacci

/// An infinite iterator that produces Fibonacci numbers.
/// Uses checked arithmetic to gracefully handle overflow.
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { current: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current;
        // Use `checked_add` to handle overflow gracefully.
        let new_next = self.current.checked_add(self.next)?;
        self.current = self.next;
        self.next = new_next;
        Some(result)
    }
}

fn main() {
    println!("=== Infinite Iterator: Fibonacci ===\n");

    // Usage: take the first 10 Fibonacci numbers
    let fib = Fibonacci { current: 0, next: 1 };
    let fibs: Vec<_> = fib.take(10).collect();
    assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    println!("First 10 Fibonacci numbers: {:?}", fibs);

    // Using the constructor
    println!("\nFirst 20 Fibonacci numbers:");
    for (i, n) in Fibonacci::new().take(20).enumerate() {
        print!("F({})={} ", i, n);
    }
    println!();

    // The iterator is lazy - no computation until needed
    println!("\n=== Lazy Evaluation Demo ===");
    let mut fib = Fibonacci::new();
    println!("Created iterator, no computation yet");
    println!("First call to next(): {:?}", fib.next());
    println!("Second call to next(): {:?}", fib.next());
    println!("Third call to next(): {:?}", fib.next());

    // Chaining with adapters
    println!("\n=== Filtering Fibonacci ===");
    let even_fibs: Vec<_> = Fibonacci::new()
        .take(20)
        .filter(|&n| n % 2 == 0)
        .collect();
    println!("Even Fibonacci numbers (first 20): {:?}", even_fibs);

    // Sum of first N Fibonacci
    let sum: u64 = Fibonacci::new().take(10).sum();
    println!("\nSum of first 10 Fibonacci: {}", sum);

    // How many Fibonacci numbers fit in u64?
    println!("\n=== Overflow Handling ===");
    let count = Fibonacci::new().count();
    println!("Number of Fibonacci numbers that fit in u64: {}", count);

    // Show the last few before overflow
    println!("\nLast 5 Fibonacci numbers before overflow:");
    let all_fibs: Vec<_> = Fibonacci::new().collect();
    for n in all_fibs.iter().rev().take(5).rev() {
        println!("  {}", n);
    }

    println!("\n=== Key Points ===");
    println!("1. Infinite iterators are possible because iteration is lazy");
    println!("2. Use .take(n) to limit infinite sequences");
    println!("3. Use checked_add to gracefully stop at overflow");
    println!("4. Works with all standard adapters");
}
