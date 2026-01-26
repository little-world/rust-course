//! Pattern 2: Attribute Macro with Parameters
//!
//! Demonstrates parsing custom arguments passed to attribute macros.
//! The log prefix is configurable per-function.

use my_macros::log;

#[log("[INFO]")]
fn process_data() {
    println!("  Processing data...");
}

#[log("[DEBUG]")]
fn validate_input() -> bool {
    println!("  Validating input...");
    true
}

#[log("[TRACE]")]
fn inner_computation() -> i32 {
    println!("  Computing...");
    42
}

fn main() {
    println!("=== Log Attribute Macro Demo ===\n");

    process_data();
    println!();

    let valid = validate_input();
    println!("Validation result: {}\n", valid);

    let result = inner_computation();
    println!("Computation result: {}", result);
}
