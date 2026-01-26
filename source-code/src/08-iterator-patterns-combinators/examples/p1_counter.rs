//! Pattern 1: Custom Iterators and IntoIterator
//! Example: A Basic Custom Iterator
//!
//! Run with: cargo run --example p1_counter

/// A simple counter that iterates from `current` to `max` (exclusive).
/// This demonstrates the simplest form of a custom iterator.
struct Counter {
    current: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Self {
        Counter { current: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            let result = self.current;
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

fn main() {
    println!("=== Basic Custom Iterator: Counter ===\n");

    // Usage: create a counter and sum its values
    let sum: u32 = Counter { current: 0, max: 5 }.sum();
    assert_eq!(sum, 10); // 0 + 1 + 2 + 3 + 4
    println!("Sum of Counter(0..5): {}", sum);

    // Using the constructor
    let counter = Counter::new(10);
    println!("\nCounter(0..10) values:");
    for value in counter {
        print!("{} ", value);
    }
    println!();

    // Chaining with standard adapters
    println!("\n=== Chaining with Adapters ===");
    let filtered: Vec<u32> = Counter::new(10)
        .filter(|&x| x % 2 == 0)
        .collect();
    println!("Even numbers from Counter(0..10): {:?}", filtered);

    let mapped: Vec<u32> = Counter::new(5)
        .map(|x| x * x)
        .collect();
    println!("Squares from Counter(0..5): {:?}", mapped);

    // Taking elements
    let first_three: Vec<u32> = Counter::new(100)
        .take(3)
        .collect();
    println!("First 3 from Counter(0..100): {:?}", first_three);

    println!("\n=== Key Points ===");
    println!("1. Implement Iterator trait with type Item and next() method");
    println!("2. Return Some(value) to yield, None to signal end");
    println!("3. Iterator state is mutable (self.current advances)");
    println!("4. Works with all standard adapters (map, filter, take, etc.)");
}
