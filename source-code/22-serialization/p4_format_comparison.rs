// Pattern 4: Binary vs Text Formats
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Product {
    id: u64,
    name: String,
    price: f64,
    in_stock: bool,
}

// JSON format demo
fn json_format_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== JSON Format Demo ===\n");

    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    // Compact JSON
    let json_compact = serde_json::to_string(&product)?;
    println!("Compact JSON ({} bytes):", json_compact.len());
    println!("{}\n", json_compact);

    // Pretty JSON
    let json_pretty = serde_json::to_string_pretty(&product)?;
    println!("Pretty JSON ({} bytes):", json_pretty.len());
    println!("{}", json_pretty);

    // Deserialize
    let deserialized: Product = serde_json::from_str(&json_compact)?;
    println!("\nDeserialized: {:?}", deserialized);

    Ok(())
}

// Bincode format demo (Rust-to-Rust, most compact)
fn bincode_format_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Bincode Format Demo ===\n");

    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    let encoded = bincode::serialize(&product)?;
    println!("Bincode ({} bytes):", encoded.len());
    println!("Raw bytes: {:?}", &encoded[..std::cmp::min(50, encoded.len())]);

    let decoded: Product = bincode::deserialize(&encoded)?;
    println!("\nDeserialized: {:?}", decoded);

    Ok(())
}

// MessagePack format demo (cross-language binary)
fn messagepack_format_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== MessagePack Format Demo ===\n");

    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    let encoded = rmp_serde::to_vec(&product)?;
    println!("MessagePack ({} bytes):", encoded.len());
    println!("Raw bytes: {:?}", encoded);

    let decoded: Product = rmp_serde::from_slice(&encoded)?;
    println!("\nDeserialized: {:?}", decoded);

    Ok(())
}

// CBOR format demo (IoT/embedded)
fn cbor_format_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== CBOR Format Demo ===\n");

    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    let encoded = serde_cbor::to_vec(&product)?;
    println!("CBOR ({} bytes):", encoded.len());
    println!("Raw bytes: {:?}", encoded);

    let decoded: Product = serde_cbor::from_slice(&encoded)?;
    println!("\nDeserialized: {:?}", decoded);

    Ok(())
}

// YAML format demo (human-readable config)
fn yaml_format_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== YAML Format Demo ===\n");

    let product = Product {
        id: 12345,
        name: "Widget".to_string(),
        price: 29.99,
        in_stock: true,
    };

    let yaml = serde_yaml::to_string(&product)?;
    println!("YAML ({} bytes):", yaml.len());
    println!("{}", yaml);

    let deserialized: Product = serde_yaml::from_str(&yaml)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}

// TOML format demo (config files)
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    database: DatabaseConfig,
    server: ServerConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: u32,
}

fn toml_format_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== TOML Format Demo ===\n");

    let config = Config {
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "admin".to_string(),
        },
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
        },
    };

    let toml_str = toml::to_string_pretty(&config)?;
    println!("TOML ({} bytes):", toml_str.len());
    println!("{}", toml_str);

    let deserialized: Config = toml::from_str(&toml_str)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}

// Format comparison
#[derive(Serialize, Deserialize, Debug, Clone)]
struct BenchmarkData {
    id: u64,
    name: String,
    values: Vec<f64>,
    metadata: HashMap<String, String>,
}

fn format_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Format Size Comparison ===\n");

    let data = BenchmarkData {
        id: 12345,
        name: "Test Data with a longer name".to_string(),
        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        metadata: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), "value1".to_string());
            map.insert("key2".to_string(), "value2".to_string());
            map.insert("key3".to_string(), "value3".to_string());
            map
        },
    };

    // Compare sizes
    let json = serde_json::to_string(&data)?;
    let json_pretty = serde_json::to_string_pretty(&data)?;
    let bincode_data = bincode::serialize(&data)?;
    let msgpack = rmp_serde::to_vec(&data)?;
    let cbor = serde_cbor::to_vec(&data)?;
    let yaml = serde_yaml::to_string(&data)?;

    println!("Format comparison for same data:");
    println!("  JSON (compact):  {:4} bytes", json.len());
    println!("  JSON (pretty):   {:4} bytes", json_pretty.len());
    println!("  Bincode:         {:4} bytes", bincode_data.len());
    println!("  MessagePack:     {:4} bytes", msgpack.len());
    println!("  CBOR:            {:4} bytes", cbor.len());
    println!("  YAML:            {:4} bytes", yaml.len());

    println!("\nSize relative to JSON compact:");
    let json_size = json.len() as f64;
    println!("  JSON (compact):  100.0%");
    println!("  JSON (pretty):   {:5.1}%", json_pretty.len() as f64 / json_size * 100.0);
    println!("  Bincode:         {:5.1}%", bincode_data.len() as f64 / json_size * 100.0);
    println!("  MessagePack:     {:5.1}%", msgpack.len() as f64 / json_size * 100.0);
    println!("  CBOR:            {:5.1}%", cbor.len() as f64 / json_size * 100.0);
    println!("  YAML:            {:5.1}%", yaml.len() as f64 / json_size * 100.0);

    Ok(())
}

// Format selection guide
fn format_selection_guide() {
    println!("\n=== Format Selection Guide ===\n");

    println!("| Format      | Use Case                    | Pros                  | Cons                |");
    println!("|-------------|-----------------------------|-----------------------|---------------------|");
    println!("| JSON        | REST APIs, web, debugging   | Universal, readable   | Verbose, slower     |");
    println!("| Bincode     | Rust IPC, caching           | Smallest, fastest     | Rust-only           |");
    println!("| MessagePack | Cross-lang RPC, gaming      | Compact, fast, interop| Not readable        |");
    println!("| CBOR        | IoT, embedded systems       | Compact, self-desc    | Less common         |");
    println!("| YAML        | Complex configs             | Very readable         | Complex spec, slow  |");
    println!("| TOML        | Simple configs              | Simple, clear         | Limited nesting     |");
}

// Practical usage scenarios
fn practical_scenarios_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Practical Usage Scenarios ===\n");

    #[derive(Serialize, Deserialize, Debug)]
    struct UserSession {
        user_id: u64,
        username: String,
        roles: Vec<String>,
        expires_at: u64,
    }

    let session = UserSession {
        user_id: 42,
        username: "alice".to_string(),
        roles: vec!["user".to_string(), "admin".to_string()],
        expires_at: 1735689600,
    };

    // Scenario 1: REST API response
    println!("1. REST API (JSON):");
    let json = serde_json::to_string_pretty(&session)?;
    println!("{}\n", json);

    // Scenario 2: Redis cache (bincode for speed)
    println!("2. Redis cache (Bincode):");
    let bincode_data = bincode::serialize(&session)?;
    println!("   {} bytes (vs {} JSON bytes)\n", bincode_data.len(), json.len());

    // Scenario 3: Cross-language microservice (MessagePack)
    println!("3. Cross-language RPC (MessagePack):");
    let msgpack = rmp_serde::to_vec(&session)?;
    println!("   {} bytes, compatible with Python, Go, etc.\n", msgpack.len());

    // Scenario 4: Debug logging (JSON pretty)
    println!("4. Debug logging (JSON pretty):");
    println!("   log::debug!(\"Session: {{}}\", serde_json::to_string_pretty(&session)?);");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Binary vs Text Formats Demo ===\n");

    json_format_demo()?;
    bincode_format_demo()?;
    messagepack_format_demo()?;
    cbor_format_demo()?;
    yaml_format_demo()?;
    toml_format_demo()?;
    format_comparison()?;
    format_selection_guide();
    practical_scenarios_demo()?;

    println!("\nFormat comparison demo completed");
    Ok(())
}
