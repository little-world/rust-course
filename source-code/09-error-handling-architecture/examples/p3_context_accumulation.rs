//! Pattern 3: Error Propagation Strategies
//! Example: Context Accumulation with with_context
//!
//! Run with: cargo run --example p3_context_accumulation

use anyhow::{bail, Context, Result};

fn load_config(path: &str) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path))
}

fn parse_port(config: &str) -> Result<u16> {
    config
        .lines()
        .find(|l| l.starts_with("port="))
        .ok_or_else(|| anyhow::anyhow!("Missing 'port' in config"))?
        .split('=')
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Invalid port format"))?
        .trim()
        .parse()
        .context("Port must be a valid number")
}

fn validate_port(port: u16) -> Result<u16> {
    if port == 0 {
        bail!("Port cannot be 0");
    }
    if port < 1024 {
        bail!("Port {} is privileged (< 1024)", port);
    }
    Ok(port)
}

fn initialize_server(path: &str) -> Result<u16> {
    let config = load_config(path).context("Loading configuration")?;
    let port = parse_port(&config).context("Parsing port number")?;
    let valid_port = validate_port(port).context("Validating port")?;
    Ok(valid_port)
}

fn main() {
    println!("=== Context Accumulation ===\n");

    // Create test configs
    let valid_config = "/tmp/valid_server.conf";
    std::fs::write(valid_config, "name=myserver\nport=8080\n").unwrap();

    let invalid_port_config = "/tmp/invalid_port.conf";
    std::fs::write(invalid_port_config, "port=abc\n").unwrap();

    let privileged_port_config = "/tmp/privileged_port.conf";
    std::fs::write(privileged_port_config, "port=80\n").unwrap();

    let test_cases = vec![
        ("Valid config", valid_config),
        ("Missing file", "/tmp/nonexistent.conf"),
        ("Invalid port value", invalid_port_config),
        ("Privileged port", privileged_port_config),
    ];

    for (name, path) in test_cases {
        println!("{}:", name);
        match initialize_server(path) {
            Ok(port) => println!("  Success: server on port {}", port),
            Err(e) => {
                println!("  Error chain:");
                println!("    {}", e);
                for cause in e.chain().skip(1) {
                    println!("    Caused by: {}", cause);
                }
            }
        }
        println!();
    }

    // Cleanup
    let _ = std::fs::remove_file(valid_config);
    let _ = std::fs::remove_file(invalid_port_config);
    let _ = std::fs::remove_file(privileged_port_config);

    println!("=== Error Chain Benefits ===");
    println!("Error: Loading configuration");
    println!("Caused by: Failed to read config: /tmp/missing.conf");
    println!("Caused by: No such file or directory (os error 2)");
    println!();
    println!("Each level adds context about what operation failed!");

    println!("\n=== Key Points ===");
    println!("1. with_context() adds lazy context (closure only called on error)");
    println!("2. context() adds static context (string allocated immediately)");
    println!("3. e.chain() iterates through all causes");
    println!("4. bail!() creates error and returns immediately");
}
