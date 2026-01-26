//! Pattern 4: Extension Traits
//! Example: Ergonomic Error Handling Extensions
//!
//! Run with: cargo run --example p4_result_extension

use std::fmt::Display;

// Extension for logging errors
trait ResultLogExt<T, E> {
    fn log_err(self, context: &str) -> Self;
}

impl<T, E: Display> ResultLogExt<T, E> for Result<T, E> {
    fn log_err(self, context: &str) -> Self {
        if let Err(ref e) = self {
            eprintln!("[ERROR] {}: {}", context, e);
        }
        self
    }
}

// Extension for adding context to errors
trait ResultContextExt<T, E> {
    fn with_context(self, msg: &str) -> Result<T, String>;
}

impl<T, E: Display> ResultContextExt<T, E> for Result<T, E> {
    fn with_context(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{}: {}", msg, e))
    }
}

// Extension for providing default on error with logging
trait ResultDefaultExt<T> {
    fn or_default_log(self, default: T, context: &str) -> T;
}

impl<T, E: Display> ResultDefaultExt<T> for Result<T, E> {
    fn or_default_log(self, default: T, context: &str) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[WARN] {}: {} - using default", context, e);
                default
            }
        }
    }
}

// Extension for Option
trait OptionContextExt<T> {
    fn ok_or_context(self, msg: &str) -> Result<T, String>;
}

impl<T> OptionContextExt<T> for Option<T> {
    fn ok_or_context(self, msg: &str) -> Result<T, String> {
        self.ok_or_else(|| msg.to_string())
    }
}

// Simulated operations that can fail
fn parse_number(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse()
}

fn find_user(id: i32) -> Option<String> {
    match id {
        1 => Some("Alice".to_string()),
        2 => Some("Bob".to_string()),
        _ => None,
    }
}

fn main() {
    println!("=== log_err() ===");
    let result1: Result<i32, _> = parse_number("42");
    let _ = result1.log_err("parsing input"); // No error, no log

    let result2: Result<i32, _> = parse_number("not a number");
    let _ = result2.log_err("parsing input"); // Logs error

    println!("\n=== with_context() ===");
    let result: Result<i32, _> = parse_number("bad")
        .with_context("Failed to parse user input");
    println!("Contextualized error: {:?}", result);

    // Chain contexts
    let chained: Result<i32, String> = parse_number("bad")
        .with_context("parsing age")
        .map_err(|e| format!("User registration failed: {}", e));
    println!("Chained error: {:?}", chained);

    println!("\n=== or_default_log() ===");
    let value1 = parse_number("42").or_default_log(0, "parsing value");
    println!("Parsed value: {}", value1);

    let value2 = parse_number("bad").or_default_log(0, "parsing value");
    println!("Default value: {}", value2);

    println!("\n=== Option with context ===");
    let user1 = find_user(1).ok_or_context("User not found");
    println!("User 1: {:?}", user1);

    let user3 = find_user(999).ok_or_context("User 999 not found");
    println!("User 999: {:?}", user3);

    println!("\n=== Combining extensions ===");
    let final_result = parse_number("bad")
        .log_err("initial parse")
        .with_context("processing input")
        .map(|n| n * 2);
    println!("Final: {:?}", final_result);
}
