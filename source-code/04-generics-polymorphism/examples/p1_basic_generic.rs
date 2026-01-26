//! Pattern 1: Type-Safe Generic Functions
//! Example: Basic Generic Functions with Type Inference
//!
//! Run with: cargo run --example p1_basic_generic

use std::fmt::Display;

// Basic generic function - compiler infers T from usage
fn identity<T>(x: T) -> T {
    x
}

// Generic function with trait bound
fn largest<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut largest = list.first()?;
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    Some(largest)
}

// Multiple trait bounds with +
fn print_sorted<T: Ord + Display>(mut items: Vec<T>) {
    items.sort();
    for item in items {
        println!("{}", item);
    }
}

fn main() {
    println!("=== Basic Generic Function ===");
    // Usage: Compiler infers T from argument; works with any type.
    let x = identity(42); // T = i32
    let s = identity("hello"); // T = &str
    let v = identity(vec![1, 2, 3]); // T = Vec<i32>
    println!("identity(42) = {}", x);
    println!("identity(\"hello\") = {}", s);
    println!("identity(vec![1, 2, 3]) = {:?}", v);

    println!("\n=== Generic with Trait Bound ===");
    // Usage: Works on any slice of comparable items.
    let max_int = largest(&[34, 50, 25, 100, 65]);
    let max_str = largest(&["apple", "zebra", "mango"]);
    println!("largest of [34, 50, 25, 100, 65]: {:?}", max_int);
    println!("largest of [\"apple\", \"zebra\", \"mango\"]: {:?}", max_str);

    println!("\n=== Multiple Trait Bounds ===");
    // Usage: Bounds enable sorting (Ord) and printing (Display).
    println!("Sorted numbers:");
    print_sorted(vec![3, 1, 4, 1, 5]);

    println!("\nSorted strings:");
    print_sorted(vec!["banana", "apple", "cherry"]);
}
