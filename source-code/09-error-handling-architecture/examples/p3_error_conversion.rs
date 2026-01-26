//! Pattern 3: Error Propagation Strategies
//! Example: Error Type Conversion with #[from]
//!
//! Run with: cargo run --example p3_error_conversion

use thiserror::Error;

/// Custom parse error for our application.
#[derive(Error, Debug, PartialEq)]
pub enum ParseError {
    #[error("the input was empty")]
    EmptyInput,
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

fn parse_number(input: &str) -> Result<i32, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    input
        .trim()
        .parse()
        .map_err(|_| ParseError::InvalidFormat(input.to_string()))
}

/// Application error that wraps multiple error types.
/// #[from] enables automatic conversion with ?.
#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Config error: {message}")]
    Config { message: String },
}

/// Process a file containing a number.
/// The ? operator automatically converts errors via #[from].
fn process_file(path: &str) -> Result<i32, AppError> {
    // std::io::Error is auto-converted to AppError::Io
    let content = std::fs::read_to_string(path)?;

    // ParseError is auto-converted to AppError::Parse
    let number = parse_number(&content)?;

    if number < 0 {
        return Err(AppError::Config {
            message: "Number must be positive".into(),
        });
    }

    Ok(number * 2)
}

/// Demonstrate manual conversion without #[from].
fn process_file_manual(path: &str) -> Result<i32, AppError> {
    let content = std::fs::read_to_string(path).map_err(AppError::Io)?;

    let number = parse_number(&content).map_err(AppError::Parse)?;

    Ok(number * 2)
}

fn main() {
    println!("=== Error Type Conversion with #[from] ===\n");

    // Create test files
    let valid_file = "/tmp/valid_number.txt";
    let invalid_file = "/tmp/invalid_number.txt";
    let negative_file = "/tmp/negative_number.txt";

    std::fs::write(valid_file, "42").unwrap();
    std::fs::write(invalid_file, "not a number").unwrap();
    std::fs::write(negative_file, "-10").unwrap();

    // Test different scenarios
    println!("Testing process_file():\n");

    let test_cases = vec![
        ("Valid file", valid_file),
        ("Invalid content", invalid_file),
        ("Negative number", negative_file),
        ("Missing file", "/tmp/nonexistent.txt"),
    ];

    for (name, path) in test_cases {
        print!("  {}: ", name);
        match process_file(path) {
            Ok(n) => println!("Success -> {}", n),
            Err(AppError::Io(e)) => println!("IO Error -> {}", e),
            Err(AppError::Parse(e)) => println!("Parse Error -> {}", e),
            Err(AppError::Config { message }) => println!("Config Error -> {}", message),
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(valid_file);
    let _ = std::fs::remove_file(invalid_file);
    let _ = std::fs::remove_file(negative_file);

    println!("\n=== How #[from] Works ===");
    println!("  #[derive(Error)]");
    println!("  enum AppError {{");
    println!("      #[error(\"IO error\")]");
    println!("      Io(#[from] std::io::Error),  // <-- Generates From impl");
    println!("  }}");
    println!();
    println!("  // This From impl is auto-generated:");
    println!("  impl From<std::io::Error> for AppError {{");
    println!("      fn from(err: std::io::Error) -> Self {{");
    println!("          AppError::Io(err)");
    println!("      }}");
    println!("  }}");

    println!("\n=== With vs Without #[from] ===");
    println!("With #[from]:");
    println!("  let content = std::fs::read_to_string(path)?;");
    println!();
    println!("Without #[from] (manual):");
    println!("  let content = std::fs::read_to_string(path).map_err(AppError::Io)?;");

    println!("\n=== Key Points ===");
    println!("1. #[from] generates From<SourceError> for your error type");
    println!("2. ? calls .into() which uses the From impl");
    println!("3. Mix different error types in one function seamlessly");
    println!("4. Pattern match on variants to handle each error type differently");
}
