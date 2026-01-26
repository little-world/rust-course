//! Pattern 5: Blanket Implementations
//! Example: Blanket Impl for All Types Implementing a Trait
//!
//! Run with: cargo run --example p5_blanket_impl

use std::fmt::Display;

// Blanket impl: Printable for any Display type
trait Printable {
    fn print(&self);
    fn print_with_prefix(&self, prefix: &str);
}

impl<T: Display> Printable for T {
    fn print(&self) {
        println!("{}", self);
    }

    fn print_with_prefix(&self, prefix: &str) {
        println!("{}: {}", prefix, self);
    }
}

// Another blanket impl example: Describable
use std::fmt::Debug;

trait Describable {
    fn describe(&self) -> String;
}

impl<T: Debug> Describable for T {
    fn describe(&self) -> String {
        format!("Debug representation: {:?}", self)
    }
}

// Blanket impl with multiple bounds
trait Inspectable {
    fn inspect(&self);
}

impl<T: Debug + Display> Inspectable for T {
    fn inspect(&self) {
        println!("Display: {}", self);
        println!("Debug: {:?}", self);
    }
}

fn main() {
    println!("=== Printable (blanket impl for Display) ===");
    // Usage: Any Display type automatically gets print() method.
    42.print();
    "hello".print();
    3.14.print();

    42.print_with_prefix("Number");
    "hello".print_with_prefix("String");

    println!("\n=== Describable (blanket impl for Debug) ===");
    let v = vec![1, 2, 3];
    println!("{}", v.describe());

    let tuple = (1, "two", 3.0);
    println!("{}", tuple.describe());

    #[derive(Debug)]
    struct Point {
        x: i32,
        y: i32,
    }
    let p = Point { x: 10, y: 20 };
    println!("{}", p.describe());

    println!("\n=== Inspectable (blanket impl for Debug + Display) ===");
    42.inspect();
    println!();
    "world".inspect();

    println!("\n=== How Blanket Impls Work ===");
    println!("impl<T: Display> Printable for T {{ ... }}");
    println!("  - Applies to ALL types implementing Display");
    println!("  - i32, &str, String, f64, etc. all get Printable");
    println!("  - Future types implementing Display will also get it");

    println!("\n=== Standard Library Examples ===");
    println!("impl<T: Display> ToString for T {{ ... }}");
    println!("  - Any Display type gets .to_string()");
    let s = 42.to_string();
    println!("  - 42.to_string() = \"{}\"", s);
}
