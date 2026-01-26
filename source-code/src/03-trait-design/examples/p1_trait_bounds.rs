//! Pattern 1: Trait Inheritance and Bounds
//! Example: Trait Bounds in Generic Functions
//!
//! Run with: cargo run --example p1_trait_bounds

use std::fmt::{Debug, Display};

// Simple bound
fn print_item<T: Display>(item: T) {
    println!("{}", item);
}

// Multiple bounds
fn process<T: Clone + Debug>(item: T) {
    let copy = item.clone();
    println!("Processing: {:?}", copy);
}

// Where clause for readability
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: Debug + Clone,
    U: Display + Default,
{
    format!("{:?} and {}", t, u)
}

// Where clause in impl block
struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    fn new(value: T) -> Self {
        Container { value }
    }
}

impl<T: Display> Container<T> {
    fn display(&self) {
        println!("Container holds: {}", self.value);
    }
}

impl<T: Debug> Container<T> {
    fn debug(&self) {
        println!("Container debug: {:?}", self.value);
    }
}

fn main() {
    // Usage: Bounds specify required capabilities for generic parameters.
    println!("=== Simple bound (Display) ===");
    print_item("hello");
    print_item(42);

    println!("\n=== Multiple bounds (Clone + Debug) ===");
    process(vec![1, 2, 3]);
    process("test".to_string());

    println!("\n=== Where clause ===");
    let result = complex_function(vec![1, 2], String::new());
    println!("Result: {}", result);

    println!("\n=== Conditional methods on Container ===");
    let c1 = Container::new(42);
    c1.display(); // Works: i32 is Display
    c1.debug(); // Works: i32 is Debug

    let c2 = Container::new(vec![1, 2, 3]);
    // c2.display(); // Won't compile: Vec<i32> is not Display
    c2.debug(); // Works: Vec<i32> is Debug
}
