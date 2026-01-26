//! Pattern 3: Struct Memory and Update Patterns
//! Example: Struct Update Syntax
//!
//! Run with: cargo run --example p3_struct_update

#[derive(Debug, Clone, PartialEq)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
}

fn main() {
    let base = Config {
        host: "localhost".to_string(),
        port: 8080,
        timeout_ms: 5000,
    };

    // Create updated config using struct update syntax
    // Note: we clone base to preserve it
    let updated = Config {
        port: 9090,
        ..base.clone()
    };

    // Usage: updated has new port, other fields copied from base.
    assert_eq!(updated.port, 9090);
    assert_eq!(updated.host, "localhost"); // Cloned from base
    assert_eq!(base.port, 8080); // Original unchanged

    println!("Base config: {:?}", base);
    println!("Updated config: {:?}", updated);

    // Multiple variations from same base
    let dev_config = Config {
        host: "127.0.0.1".to_string(),
        ..base.clone()
    };

    let prod_config = Config {
        host: "prod.example.com".to_string(),
        port: 443,
        timeout_ms: 30000,
        // No ..base needed when all fields specified
    };

    println!("\nDev config: {:?}", dev_config);
    println!("Prod config: {:?}", prod_config);

    // Without clone, non-Copy fields are moved:
    let base2 = Config {
        host: "test".to_string(),
        port: 80,
        timeout_ms: 1000,
    };
    let moved = Config {
        port: 81,
        ..base2 // host (String) is moved, not copied
    };
    println!("\nMoved config: {:?}", moved);
    // println!("{:?}", base2.host); // Error: value moved
    // But Copy fields are still accessible:
    println!("base2.port still accessible: {}", base2.port);
}
