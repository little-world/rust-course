//! Pattern 3: Trait Objects and Dynamic Dispatch
//! Example: Dynamic Dispatch with Trait Objects
//!
//! Run with: cargo run --example p3_dynamic_dispatch

use std::fmt::Display;

// Dynamic dispatch - single function handles all types at runtime
fn process(item: &dyn Display) {
    println!("{}", item);
}

// Can also use Box<dyn Trait> for owned values
fn process_owned(item: Box<dyn Display>) {
    println!("Owned: {}", item);
}

fn main() {
    // Usage: One function handles all types; enables heterogeneous collections.
    println!("=== Dynamic Dispatch Demo ===\n");

    let num: i32 = 42;
    let text: &str = "hello";
    let float: f64 = 3.14;

    println!("Calling process() with references:");
    process(&num);
    process(&text);
    process(&float);

    // Heterogeneous collection - impossible with static dispatch!
    println!("\n=== Heterogeneous Collection ===");
    let items: Vec<&dyn Display> = vec![&num, &text, &float];

    for (i, item) in items.iter().enumerate() {
        println!("Item {}: {}", i, item);
    }

    // With owned values
    println!("\n=== Boxed Trait Objects ===");
    let owned_items: Vec<Box<dyn Display>> = vec![
        Box::new(100),
        Box::new("boxed string"),
        Box::new(2.718),
    ];

    for item in &owned_items {
        println!("Boxed: {}", item);
    }

    // Pass to function taking owned trait object
    process_owned(Box::new("owned and moved"));

    println!("\n=== Key Points ===");
    println!("- Single function handles all types via vtable");
    println!("- Smaller binary size");
    println!("- Slight runtime cost (~2-3ns per call)");
    println!("- Enables heterogeneous collections!");
}
