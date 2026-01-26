//! Pattern 3: Error Propagation Strategies
//! Example: Recovering from Specific Errors
//!
//! Run with: cargo run --example p3_recover_specific

use std::io::ErrorKind;

/// Read file or return default for missing files.
/// Other IO errors are propagated.
fn read_or_default(path: &str) -> Result<String, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            println!("  [INFO] File not found, using default");
            Ok("default value".to_string())
        }
        Err(e) => Err(e), // Propagate other errors
    }
}

/// Read config with multiple fallback paths.
fn read_config_with_fallbacks(paths: &[&str]) -> Result<String, std::io::Error> {
    for path in paths {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                println!("  [INFO] Loaded from: {}", path);
                return Ok(content);
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                println!("  [INFO] Not found: {}, trying next...", path);
                continue;
            }
            Err(e) => return Err(e), // Propagate permission errors, etc.
        }
    }

    // All paths failed
    Err(std::io::Error::new(
        ErrorKind::NotFound,
        format!("None of the config paths exist: {:?}", paths),
    ))
}

/// Try parsing, fall back to default on parse errors.
fn parse_or_default(input: &str, default: i32) -> i32 {
    input.trim().parse().unwrap_or_else(|_| {
        println!("  [INFO] Parse failed, using default: {}", default);
        default
    })
}

/// Try operation, retry on timeout.
fn retry_on_timeout<F, T>(mut f: F, max_retries: usize) -> Result<T, std::io::Error>
where
    F: FnMut() -> Result<T, std::io::Error>,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match f() {
            Ok(value) => return Ok(value),
            Err(e) if e.kind() == ErrorKind::TimedOut && attempts < max_retries => {
                println!("  [RETRY] Attempt {} timed out, retrying...", attempts);
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}

fn main() {
    println!("=== Recovering from Specific Errors ===\n");

    // Create a test file
    let existing_file = "/tmp/existing_config.txt";
    std::fs::write(existing_file, "existing content").unwrap();

    // Test read_or_default
    println!("=== read_or_default ===");

    println!("\nReading existing file:");
    match read_or_default(existing_file) {
        Ok(content) => println!("  Content: '{}'", content.trim()),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\nReading missing file:");
    match read_or_default("/tmp/missing.txt") {
        Ok(content) => println!("  Content: '{}'", content),
        Err(e) => println!("  Error: {}", e),
    }

    // Test fallback paths
    println!("\n=== Config with Fallbacks ===");
    let fallback_file = "/tmp/fallback_config.txt";
    std::fs::write(fallback_file, "fallback content").unwrap();

    let paths = &[
        "/tmp/primary.txt",   // doesn't exist
        "/tmp/secondary.txt", // doesn't exist
        fallback_file,        // exists
    ];

    match read_config_with_fallbacks(paths) {
        Ok(content) => println!("  Loaded: '{}'", content.trim()),
        Err(e) => println!("  Error: {}", e),
    }

    // Test parse_or_default
    println!("\n=== parse_or_default ===");
    println!("  parse_or_default(\"42\", 0) = {}", parse_or_default("42", 0));
    println!(
        "  parse_or_default(\"bad\", 0) = {}",
        parse_or_default("bad", 0)
    );
    println!(
        "  parse_or_default(\"\", 100) = {}",
        parse_or_default("", 100)
    );

    // Cleanup
    let _ = std::fs::remove_file(existing_file);
    let _ = std::fs::remove_file(fallback_file);

    println!("\n=== Match Guard Pattern ===");
    println!("  match result {{");
    println!("      Ok(v) => Ok(v),");
    println!("      Err(e) if e.kind() == NotFound => Ok(default),");
    println!("      Err(e) => Err(e),  // Propagate other errors");
    println!("  }}");

    println!("\n=== Common Recovery Patterns ===");
    println!("1. NotFound -> use default value");
    println!("2. TimedOut -> retry with backoff");
    println!("3. PermissionDenied -> try alternate path");
    println!("4. WouldBlock -> retry later (non-blocking IO)");
    println!("5. ConnectionRefused -> use cached value");

    println!("\n=== Key Points ===");
    println!("1. Match guards (if condition) for specific error handling");
    println!("2. Use ErrorKind for portable error matching");
    println!("3. Recover from expected errors, propagate unexpected ones");
    println!("4. unwrap_or_else() for recovery with side effects");
}
