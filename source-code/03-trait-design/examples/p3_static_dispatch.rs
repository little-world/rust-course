//! Pattern 3: Trait Objects and Dynamic Dispatch
//! Example: Static Dispatch (Monomorphization)
//!
//! Run with: cargo run --example p3_static_dispatch

use std::fmt::Display;

// Generic function - static dispatch
fn process<T: Display>(item: T) {
    println!("{}", item);
}

// The compiler generates specialized versions:
// fn process_i32(item: i32) { println!("{}", item); }
// fn process_str(item: &str) { println!("{}", item); }
// fn process_string(item: String) { println!("{}", item); }

// Another example with multiple bounds
fn describe<T: Display + Clone>(item: T) -> String {
    let copy = item.clone();
    format!("Value: {}, Copy: {}", item, copy)
}

// Generic struct with trait bound
struct Printer<T: Display> {
    value: T,
}

impl<T: Display> Printer<T> {
    fn new(value: T) -> Self {
        Printer { value }
    }

    fn print(&self) {
        println!("Printer: {}", self.value);
    }
}

fn main() {
    // Usage: Compiler generates specialized function for each type.
    println!("=== Static Dispatch Demo ===\n");

    println!("Calling process() with different types:");
    process(42i32);   // Generates process::<i32>
    process("hello"); // Generates process::<&str>
    process(3.14f64); // Generates process::<f64>
    process(String::from("world")); // Generates process::<String>

    println!("\n=== With Multiple Bounds ===");
    println!("{}", describe(42));
    println!("{}", describe("test"));

    println!("\n=== Generic Struct ===");
    let p1 = Printer::new(100);
    let p2 = Printer::new("static dispatch");
    let p3 = Printer::new(2.718);

    p1.print(); // Uses Printer::<i32>::print
    p2.print(); // Uses Printer::<&str>::print
    p3.print(); // Uses Printer::<f64>::print

    println!("\n=== Key Points ===");
    println!("- Each type gets its own optimized function copy");
    println!("- No runtime overhead (no vtable lookup)");
    println!("- Larger binary size (code bloat)");
    println!("- Cannot create heterogeneous collections");
}
