//! Pattern 4: Custom Error Types with Context
//! Example: Generic Context Wrapper
//!
//! Run with: cargo run --example p4_error_context

use std::fmt;

/// Generic wrapper that adds context to any error type.
pub struct ErrorContext<E> {
    error: E,
    context: Vec<String>,
}

impl<E: std::error::Error> ErrorContext<E> {
    pub fn new(error: E) -> Self {
        ErrorContext {
            error,
            context: Vec::new(),
        }
    }

    pub fn add_context(mut self, ctx: impl Into<String>) -> Self {
        self.context.push(ctx.into());
        self
    }

    pub fn inner(&self) -> &E {
        &self.error
    }
}

impl<E: fmt::Display> fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        for ctx in &self.context {
            write!(f, "\n  Context: {}", ctx)?;
        }
        Ok(())
    }
}

impl<E: fmt::Debug> fmt::Debug for ErrorContext<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorContext")
            .field("error", &self.error)
            .field("context", &self.context)
            .finish()
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ErrorContext<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Extension trait for adding context to any Result.
trait ResultExt<T, E> {
    fn with_ctx(self, ctx: impl Into<String>) -> Result<T, ErrorContext<E>>;
}

impl<T, E: std::error::Error> ResultExt<T, E> for Result<T, E> {
    fn with_ctx(self, ctx: impl Into<String>) -> Result<T, ErrorContext<E>> {
        self.map_err(|e| ErrorContext::new(e).add_context(ctx))
    }
}

fn read_file(path: &str) -> Result<String, ErrorContext<std::io::Error>> {
    std::fs::read_to_string(path)
        .map_err(|e| ErrorContext::new(e))
        .map_err(|e| e.add_context(format!("reading {}", path)))
}

fn parse_number(content: &str, path: &str) -> Result<i32, ErrorContext<std::num::ParseIntError>> {
    content
        .trim()
        .parse()
        .map_err(|e| ErrorContext::new(e))
        .map_err(|e| e.add_context(format!("parsing number from {}", path)))
}

fn main() {
    println!("=== Generic Context Wrapper ===\n");

    // Demonstrate with file read error
    println!("=== File Read Error ===");
    let result = read_file("/nonexistent/path.txt");
    match result {
        Ok(content) => println!("Content: {}", content),
        Err(e) => {
            println!("Error: {}", e);
            println!("\nError chain:");
            let mut source: Option<&dyn std::error::Error> = Some(&e);
            while let Some(err) = source {
                println!("  - {}", err);
                source = err.source();
            }
        }
    }

    // Create a test file for parse error
    let test_file = "/tmp/test_number.txt";
    std::fs::write(test_file, "not a number").unwrap();

    println!("\n=== Parse Error ===");
    let content = std::fs::read_to_string(test_file).unwrap();
    let result = parse_number(&content, test_file);
    match result {
        Ok(n) => println!("Parsed: {}", n),
        Err(e) => println!("Error: {}", e),
    }

    // Cleanup
    let _ = std::fs::remove_file(test_file);

    // Using extension trait
    println!("\n=== Extension Trait Usage ===");
    let result: Result<String, _> = std::fs::read_to_string("/missing.txt")
        .with_ctx("loading configuration");

    if let Err(e) = result {
        println!("Error: {}", e);
    }

    println!("\n=== Pattern Benefits ===");
    println!("1. Works with any error type implementing std::error::Error");
    println!("2. Context accumulated as error propagates");
    println!("3. Original error preserved via source()");
    println!("4. Extension trait makes it ergonomic to use");

    println!("\n=== Key Points ===");
    println!("1. Generic over E: std::error::Error");
    println!("2. Implements Display, Debug, and Error traits");
    println!("3. source() returns original error for chain");
    println!("4. Extension trait adds .with_ctx() to all Results");
}
