//! Pattern 3: Error Propagation Strategies
//! Example: Manual Error Mapping with map_err
//!
//! Run with: cargo run --example p3_map_err

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("failed to read '{path}': {source}")]
    ReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write '{path}': {source}")]
    WriteFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid content in '{path}': {reason}")]
    InvalidContent { path: String, reason: String },
}

/// Read file with context added via map_err.
fn read_config(path: &str) -> Result<String, FileError> {
    std::fs::read_to_string(path).map_err(|e| FileError::ReadFailed {
        path: path.to_string(),
        source: e,
    })
}

/// Write file with context added via map_err.
fn write_config(path: &str, content: &str) -> Result<(), FileError> {
    std::fs::write(path, content).map_err(|e| FileError::WriteFailed {
        path: path.to_string(),
        source: e,
    })
}

/// Parse and validate config content.
fn parse_config(path: &str, content: &str) -> Result<i32, FileError> {
    content.trim().parse().map_err(|_| FileError::InvalidContent {
        path: path.to_string(),
        reason: format!("expected number, got '{}'", content.trim()),
    })
}

/// Complete workflow using map_err for context.
fn load_and_validate_config(path: &str) -> Result<i32, FileError> {
    let content = read_config(path)?;
    let value = parse_config(path, &content)?;
    Ok(value)
}

fn main() {
    println!("=== Manual Error Mapping with map_err ===\n");

    // Create a test config file
    let config_path = "/tmp/config_value.txt";
    std::fs::write(config_path, "42").unwrap();

    // Successful read
    println!("Reading valid config:");
    match load_and_validate_config(config_path) {
        Ok(value) => println!("  Config value: {}", value),
        Err(e) => println!("  Error: {}", e),
    }

    // Missing file - error includes path
    println!("\nReading missing config:");
    match load_and_validate_config("/tmp/missing_config.txt") {
        Ok(value) => println!("  Config value: {}", value),
        Err(e) => println!("  Error: {}", e),
    }

    // Invalid content - error includes path and reason
    let invalid_path = "/tmp/invalid_config.txt";
    std::fs::write(invalid_path, "not a number").unwrap();

    println!("\nReading invalid config:");
    match load_and_validate_config(invalid_path) {
        Ok(value) => println!("  Config value: {}", value),
        Err(e) => println!("  Error: {}", e),
    }

    // Show error chain
    println!("\n=== Error Chain ===");
    let result = read_config("/nonexistent/path/config.txt");
    if let Err(e) = result {
        println!("Error: {}", e);
        if let Some(source) = std::error::Error::source(&e) {
            println!("Caused by: {}", source);
        }
    }

    // Cleanup
    let _ = std::fs::remove_file(config_path);
    let _ = std::fs::remove_file(invalid_path);

    println!("\n=== map_err Pattern ===");
    println!("  std::fs::read_to_string(path)");
    println!("      .map_err(|e| FileError::ReadFailed {{");
    println!("          path: path.to_string(),");
    println!("          source: e,");
    println!("      }})?");

    println!("\n=== When to Use map_err ===");
    println!("1. When you need to add context (file path, operation name)");
    println!("2. When #[from] isn't specific enough");
    println!("3. When the same error type could come from multiple places");
    println!("4. When you want to enrich errors with domain information");

    println!("\n=== Key Points ===");
    println!("1. map_err transforms the error before propagation");
    println!("2. Original error preserved via #[source] for error chain");
    println!("3. Path context makes errors actionable");
    println!("4. Combine with ? for clean propagation after mapping");
}
