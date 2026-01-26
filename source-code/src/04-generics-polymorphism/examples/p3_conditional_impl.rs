//! Pattern 3: Trait Bounds and Constraints
//! Example: Conditional Method Implementation
//!
//! Run with: cargo run --example p3_conditional_impl

use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq, Clone)]
struct Wrapper<T>(T);

// Base impl - available for all T
impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper(value)
    }

    fn into_inner(self) -> T {
        self.0
    }
}

// Only for Display types
impl<T: Display> Wrapper<T> {
    fn display(&self) {
        println!("Display: {}", self.0);
    }

    fn formatted(&self) -> String {
        format!("[{}]", self.0)
    }
}

// Only for Clone types
impl<T: Clone> Wrapper<T> {
    fn duplicate(&self) -> Self {
        Wrapper(self.0.clone())
    }
}

// Only for Debug types
impl<T: Debug> Wrapper<T> {
    fn debug_print(&self) {
        println!("Debug: {:?}", self.0);
    }
}

// Only for numeric types (via trait bounds)
impl<T: std::ops::Add<Output = T> + Copy> Wrapper<T> {
    fn add_to(&self, other: T) -> T {
        self.0 + other
    }
}

// Only for Default types
impl<T: Default> Wrapper<T> {
    fn reset(&mut self) {
        self.0 = T::default();
    }
}

// A type that only implements Debug, not Display
#[derive(Debug, Clone)]
struct DebugOnly {
    value: i32,
}

fn main() {
    println!("=== Base Methods (all types) ===");
    let w = Wrapper::new(42);
    println!("Created Wrapper(42)");

    let inner = Wrapper::new("extract me").into_inner();
    println!("into_inner() = \"{}\"", inner);

    println!("\n=== Display Methods ===");
    // Usage: Methods appear only when inner type has required traits.
    let w = Wrapper::new(42);
    w.display(); // Works because i32: Display
    println!("formatted() = {}", w.formatted());

    let ws = Wrapper::new("hello");
    ws.display();
    println!("formatted() = {}", ws.formatted());

    println!("\n=== Clone Methods ===");
    let w = Wrapper::new(vec![1, 2, 3]);
    let dup = w.duplicate();
    println!("Original: {:?}", w);
    println!("duplicate(): {:?}", dup);

    println!("\n=== Debug Methods ===");
    let w = Wrapper::new(DebugOnly { value: 100 });
    w.debug_print(); // Works because DebugOnly: Debug
    // w.display(); // Would NOT compile - DebugOnly doesn't implement Display

    println!("\n=== Numeric Methods ===");
    let w = Wrapper::new(10);
    let sum = w.add_to(5);
    println!("Wrapper(10).add_to(5) = {}", sum);

    let wf = Wrapper::new(2.5);
    let sumf = wf.add_to(1.5);
    println!("Wrapper(2.5).add_to(1.5) = {}", sumf);

    println!("\n=== Default Methods ===");
    let mut w = Wrapper::new(100);
    println!("Before reset: {:?}", w);
    w.reset();
    println!("After reset: {:?}", w);

    let mut ws = Wrapper::new(String::from("hello"));
    println!("Before reset: {:?}", ws);
    ws.reset();
    println!("After reset: {:?}", ws);
}
