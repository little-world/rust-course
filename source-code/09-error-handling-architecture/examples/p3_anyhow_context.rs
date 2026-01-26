//! Pattern 3: Error Propagation Strategies
//! Example: Rich Context with anyhow
//!
//! Run with: cargo run --example p3_anyhow_context

use anyhow::{bail, Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Config {
    name: String,
    port: u16,
    debug: bool,
}

/// Load and parse a JSON config file.
/// Each ? adds context explaining what operation failed.
fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path))?;

    let config: Config = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON in: {}", path))?;

    validate_config(&config).context("Config validation failed")?;

    Ok(config)
}

/// Validate config values.
fn validate_config(config: &Config) -> Result<()> {
    if config.name.is_empty() {
        bail!("Config 'name' cannot be empty");
    }
    if config.port == 0 {
        bail!("Config 'port' cannot be 0");
    }
    if config.port < 1024 && !config.debug {
        bail!(
            "Port {} requires debug mode (privileged ports need debug=true)",
            config.port
        );
    }
    Ok(())
}

/// Process config with lazy context (allocates only on error).
fn process_config(path: &str) -> Result<String> {
    let config = load_config(path)?;
    Ok(format!(
        "Server '{}' configured on port {}",
        config.name, config.port
    ))
}

fn main() {
    println!("=== Rich Context with anyhow ===\n");

    // Create valid config
    let valid_path = "/tmp/valid_config.json";
    std::fs::write(
        valid_path,
        r#"{"name": "myserver", "port": 8080, "debug": false}"#,
    )
    .unwrap();

    // Create invalid JSON
    let invalid_json_path = "/tmp/invalid_json.json";
    std::fs::write(invalid_json_path, "not valid json {").unwrap();

    // Create config with validation error
    let invalid_config_path = "/tmp/invalid_config.json";
    std::fs::write(
        invalid_config_path,
        r#"{"name": "", "port": 8080, "debug": false}"#,
    )
    .unwrap();

    // Test cases
    println!("=== Test Cases ===\n");

    let test_cases = vec![
        ("Valid config", valid_path),
        ("Missing file", "/tmp/nonexistent.json"),
        ("Invalid JSON", invalid_json_path),
        ("Validation error", invalid_config_path),
    ];

    for (name, path) in test_cases {
        println!("{}:", name);
        match process_config(path) {
            Ok(msg) => println!("  Success: {}", msg),
            Err(e) => {
                println!("  Error: {}", e);
                // Print error chain
                let mut source = e.source();
                while let Some(cause) = source {
                    println!("  Caused by: {}", cause);
                    source = cause.source();
                }
            }
        }
        println!();
    }

    // Cleanup
    let _ = std::fs::remove_file(valid_path);
    let _ = std::fs::remove_file(invalid_json_path);
    let _ = std::fs::remove_file(invalid_config_path);

    println!("=== context() vs with_context() ===");
    println!("context(\"static string\"):");
    println!("  - Allocates string immediately");
    println!("  - Use for constant messages");
    println!();
    println!("with_context(|| format!(\"...\", path)):");
    println!("  - Closure called only on error");
    println!("  - Use when message needs runtime data");
    println!();
    println!("bail!(\"message\"):");
    println!("  - Creates error and returns immediately");
    println!("  - Useful for validation failures");

    println!("\n=== Error Chain Output ===");
    println!("Error: Failed to read config file: /tmp/missing.json");
    println!("Caused by: No such file or directory (os error 2)");
    println!();
    println!("The chain shows exactly what went wrong at each level!");

    println!("\n=== Key Points ===");
    println!("1. with_context() adds context lazily (only on error)");
    println!("2. Error chain preserved - can iterate through causes");
    println!("3. bail!() for immediate error return");
    println!("4. anyhow::Result is Result<T, anyhow::Error>");
}
