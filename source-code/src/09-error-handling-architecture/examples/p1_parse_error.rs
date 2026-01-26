//! Pattern 1: Custom Error Enums for Libraries
//! Example: Simple Custom Error Enum with thiserror
//!
//! Run with: cargo run --example p1_parse_error

use std::io;
use std::num::ParseIntError;
use thiserror::Error;

/// Custom error enum for parsing operations.
/// Each variant represents a distinct failure mode.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("the input was empty")]
    EmptyInput,
    #[error("the input format was invalid")]
    InvalidFormat,
    #[error("the number was too large")]
    NumberTooLarge,
}

/// Parse a number from a string, returning a typed error.
fn parse_number(input: &str) -> Result<i32, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    input.trim().parse().map_err(|_| ParseError::InvalidFormat)
}

/// Error enum that wraps underlying errors with #[from].
/// This enables automatic conversion with the ? operator.
#[derive(Error, Debug)]
pub enum DataError {
    #[error("failed to read data: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse number: {0}")]
    Parse(#[from] ParseIntError),
}

/// Load and parse a number from a file.
/// The ? operator automatically converts errors via #[from].
fn load_and_parse_number(path: &str) -> Result<i32, DataError> {
    let content = std::fs::read_to_string(path)?; // io::Error -> DataError
    let number = content.trim().parse()?;         // ParseIntError -> DataError
    Ok(number)
}

fn main() {
    println!("=== Custom Error Enums ===\n");

    // Demonstrate ParseError variants
    println!("Testing parse_number():");

    match parse_number("") {
        Err(ParseError::EmptyInput) => println!("  Empty input: handled correctly"),
        _ => println!("  Unexpected result"),
    }

    match parse_number("not a number") {
        Err(ParseError::InvalidFormat) => println!("  Invalid format: handled correctly"),
        _ => println!("  Unexpected result"),
    }

    match parse_number("42") {
        Ok(n) => println!("  Parsed '42' -> {}", n),
        Err(e) => println!("  Error: {}", e),
    }

    // Pattern matching on specific error variants
    println!("\n=== Pattern Matching on Errors ===");
    let inputs = vec!["", "abc", "123", "99999999999999"];

    for input in inputs {
        match parse_number(input) {
            Ok(n) => println!("  '{}' -> {}", input, n),
            Err(ParseError::EmptyInput) => println!("  '{}' -> Error: input was empty", input),
            Err(ParseError::InvalidFormat) => println!("  '{}' -> Error: invalid format", input),
            Err(ParseError::NumberTooLarge) => println!("  '{}' -> Error: number too large", input),
        }
    }

    // Demonstrate DataError with file operations
    println!("\n=== DataError with File Operations ===");
    match load_and_parse_number("nonexistent.txt") {
        Ok(n) => println!("  Loaded: {}", n),
        Err(DataError::Io(e)) => println!("  IO error: {}", e),
        Err(DataError::Parse(e)) => println!("  Parse error: {}", e),
    }

    println!("\n=== Key Points ===");
    println!("1. #[derive(Error)] from thiserror generates Display and Error traits");
    println!("2. #[error(\"...\") ] defines the error message format");
    println!("3. #[from] enables automatic conversion with ? operator");
    println!("4. Callers can match on specific variants for different handling");
}
