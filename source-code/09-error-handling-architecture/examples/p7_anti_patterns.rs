//! Pattern 7: Error Handling Anti-Patterns
//! Example: Common Mistakes and How to Fix Them
//!
//! Run with: cargo run --example p7_anti_patterns

use anyhow::{Context, Result};
use thiserror::Error;

// =============================================================================
// ANTI-PATTERN 1: Swallowing Errors
// =============================================================================

mod swallowing {
    // BAD: Error is silently ignored
    pub fn bad_error_handling(path: &str) {
        let _ = std::fs::read_to_string(path); // Error ignored!
    }

    // GOOD: Log or propagate the error
    pub fn good_error_handling(path: &str) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }
}

// =============================================================================
// ANTI-PATTERN 2: Using unwrap() in Library Code
// =============================================================================

mod unwrap_abuse {
    use super::*;

    #[derive(Debug)]
    pub struct Config {
        pub port: u16,
    }

    // BAD: Panics on error - caller can't handle it
    pub fn parse_config_bad(path: &str) -> Config {
        let content = std::fs::read_to_string(path).unwrap(); // Panic!
        Config {
            port: content.trim().parse().unwrap(), // Panic!
        }
    }

    // GOOD: Return Result, let caller decide
    pub fn parse_config_good(path: &str) -> Result<Config> {
        let content = std::fs::read_to_string(path).context("Reading config")?;
        let port = content.trim().parse().context("Parsing port")?;
        Ok(Config { port })
    }
}

// =============================================================================
// ANTI-PATTERN 3: Generic Error Messages
// =============================================================================

mod generic_errors {
    use super::*;

    // BAD: Unhelpful error message
    pub fn load_data_bad() -> Result<Vec<u8>> {
        std::fs::read("data.bin").map_err(|e| anyhow::anyhow!("Error: {}", e))
    }

    // GOOD: Specific, actionable message
    pub fn load_data_good() -> Result<Vec<u8>> {
        std::fs::read("data.bin").context("Failed to read data.bin - check file exists and permissions")
    }
}

// =============================================================================
// ANTI-PATTERN 4: Catching and Re-Panicking
// =============================================================================

mod repanic {
    use super::*;

    fn risky_operation() -> Result<()> {
        Ok(())
    }

    // BAD: Catches error just to panic
    pub fn bad_panic_handling() {
        if let Err(e) = risky_operation() {
            panic!("Operation failed: {}", e); // Why not just use ?
        }
    }

    // GOOD: Propagate errors normally
    pub fn good_error_handling() -> Result<()> {
        risky_operation()?;
        Ok(())
    }
}

// =============================================================================
// ANTI-PATTERN 5: Over-Broad Error Types
// =============================================================================

mod broad_errors {
    use super::*;

    // BAD: Caller doesn't know what errors to expect
    pub fn overly_generic() -> Result<i32, Box<dyn std::error::Error>> {
        Ok(42)
    }

    // GOOD: Specific error types
    #[derive(Error, Debug)]
    pub enum ParseError {
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),
        #[error("Parse error: {0}")]
        Parse(#[from] std::num::ParseIntError),
    }

    pub fn specific_errors() -> Result<i32, ParseError> {
        Ok(42)
    }
}

// =============================================================================
// ANTI-PATTERN 6: Ignoring Error Context
// =============================================================================

mod no_context {
    use super::*;

    // BAD: No context about what operation failed
    pub fn process_file_bad(path: &str) -> Result<i32> {
        let content = std::fs::read_to_string(path)?; // Which file?
        let num: i32 = content.trim().parse()?;       // What was the input?
        Ok(num)
    }

    // GOOD: Context at each step
    pub fn process_file_good(path: &str) -> Result<i32> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path))?;
        let num: i32 = content.trim().parse()
            .with_context(|| format!("Failed to parse '{}' as number from {}", content.trim(), path))?;
        Ok(num)
    }
}

fn main() {
    println!("=== Error Handling Anti-Patterns ===\n");

    // 1. Swallowing errors
    println!("=== Anti-Pattern 1: Swallowing Errors ===");
    println!("BAD:  let _ = operation();  // Error ignored!");
    println!("GOOD: let result = operation()?;  // Propagate");
    println!();

    // 2. unwrap in libraries
    println!("=== Anti-Pattern 2: unwrap() in Libraries ===");
    println!("BAD:  content.parse().unwrap()  // Panics!");
    println!("GOOD: content.parse()?  // Returns Result");
    println!();

    // 3. Generic messages
    println!("=== Anti-Pattern 3: Generic Error Messages ===");
    println!("BAD:  Err(anyhow!(\"Error: {{}}\", e))");
    println!("GOOD: .context(\"Failed to read config.toml\")");
    println!();

    // 4. Catch and re-panic
    println!("=== Anti-Pattern 4: Catching and Re-Panicking ===");
    println!("BAD:  if let Err(e) = op() {{ panic!(e); }}");
    println!("GOOD: op()?;");
    println!();

    // 5. Over-broad errors
    println!("=== Anti-Pattern 5: Over-Broad Error Types ===");
    println!("BAD:  Result<T, Box<dyn Error>>  // What errors?");
    println!("GOOD: Result<T, MyError>  // Specific variants");
    println!();

    // 6. No context
    println!("=== Anti-Pattern 6: Missing Context ===");
    println!("BAD:  fs::read_to_string(path)?  // Which path?");
    println!("GOOD: fs::read_to_string(path).context(format!(\"...\", path))?");
    println!();

    // Demonstrate good vs bad
    println!("=== Demonstration ===\n");

    println!("Calling parse_config_good(\"/nonexistent\"):");
    match unwrap_abuse::parse_config_good("/nonexistent") {
        Ok(c) => println!("  Config: {:?}", c),
        Err(e) => {
            println!("  Error chain:");
            for cause in e.chain() {
                println!("    - {}", cause);
            }
        }
    }

    println!("\nCalling process_file_good(\"/nonexistent\"):");
    match no_context::process_file_good("/nonexistent") {
        Ok(n) => println!("  Number: {}", n),
        Err(e) => {
            println!("  Error chain:");
            for cause in e.chain() {
                println!("    - {}", cause);
            }
        }
    }

    println!("\n=== Summary: Do's and Don'ts ===\n");
    println!("DON'T:");
    println!("  - Use let _ = to ignore errors");
    println!("  - Use unwrap() in library code");
    println!("  - Write generic error messages");
    println!("  - Catch errors just to panic");
    println!("  - Use Box<dyn Error> everywhere");
    println!("  - Skip error context");
    println!();
    println!("DO:");
    println!("  - Return Result and let caller decide");
    println!("  - Use ? for propagation");
    println!("  - Add context with .context()/.with_context()");
    println!("  - Define specific error types for libraries");
    println!("  - Use anyhow for applications");
    println!("  - Include file paths, values, operation names");
}
