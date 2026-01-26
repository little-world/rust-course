//! Pattern 3: Error Propagation Strategies
//! Example: Basic Error Propagation with ?
//!
//! Run with: cargo run --example p3_basic_propagation

use std::fs::File;
use std::io::{self, Read, Write};

/// Read username from a file using ? operator.
/// The ? unwraps Ok values and early-returns Err values.
fn read_username(path: &str) -> Result<String, io::Error> {
    let content = std::fs::read_to_string(path)?; // Early return on error
    let username = content.trim().to_string();
    Ok(username)
}

/// Read and process a file with multiple ? operators.
/// Each ? can fail independently, errors bubble up.
fn read_and_count_lines(path: &str) -> Result<usize, io::Error> {
    let content = std::fs::read_to_string(path)?;
    let count = content.lines().count();
    Ok(count)
}

/// Demonstrate verbose error handling without ?.
fn read_username_verbose(path: &str) -> Result<String, io::Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(e), // What ? does automatically
    };
    let username = content.trim().to_string();
    Ok(username)
}

/// Chain multiple operations with ?.
fn copy_file(src: &str, dst: &str) -> Result<u64, io::Error> {
    let mut source = File::open(src)?;
    let mut dest = File::create(dst)?;
    let mut buffer = Vec::new();
    let bytes = source.read_to_end(&mut buffer)?;
    dest.write_all(&buffer)?;
    Ok(bytes as u64)
}

fn main() -> Result<(), io::Error> {
    println!("=== Basic Error Propagation with ? ===\n");

    // Create a test file
    let test_file = "/tmp/test_user.txt";
    std::fs::write(test_file, "alice\n")?;
    println!("Created test file: {}", test_file);

    // Demonstrate successful read
    match read_username(test_file) {
        Ok(name) => println!("Read username: '{}'", name),
        Err(e) => println!("Error: {}", e),
    }

    // Demonstrate error on missing file
    println!("\nReading nonexistent file:");
    match read_username("/tmp/nonexistent.txt") {
        Ok(name) => println!("Username: {}", name),
        Err(e) => println!("Error (expected): {}", e),
    }

    // Count lines
    std::fs::write(test_file, "line1\nline2\nline3\n")?;
    match read_and_count_lines(test_file) {
        Ok(count) => println!("\nLine count: {}", count),
        Err(e) => println!("Error: {}", e),
    }

    // Demonstrate file copy
    let src_file = "/tmp/source.txt";
    let dst_file = "/tmp/dest.txt";
    std::fs::write(src_file, "Hello, World!")?;

    match copy_file(src_file, dst_file) {
        Ok(bytes) => println!("\nCopied {} bytes from {} to {}", bytes, src_file, dst_file),
        Err(e) => println!("Copy failed: {}", e),
    }

    // Cleanup
    let _ = std::fs::remove_file(test_file);
    let _ = std::fs::remove_file(src_file);
    let _ = std::fs::remove_file(dst_file);

    println!("\n=== The ? Operator ===");
    println!("  let content = std::fs::read_to_string(path)?;");
    println!();
    println!("Is equivalent to:");
    println!();
    println!("  let content = match std::fs::read_to_string(path) {{");
    println!("      Ok(c) => c,");
    println!("      Err(e) => return Err(e.into()),");
    println!("  }};");

    println!("\n=== Key Points ===");
    println!("1. ? unwraps Ok and early-returns Err");
    println!("2. ? calls Into::into() for automatic error conversion");
    println!("3. Chain multiple ? for clean, linear code");
    println!("4. main() can return Result for simple error handling");

    Ok(())
}
