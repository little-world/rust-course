// Pattern 2: Zero-Copy Deserialization
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

// Zero-copy: borrows from the input string
// Lifetimes ensure the struct can't outlive the input
#[derive(Deserialize, Debug)]
struct BorrowedData<'a> {
    #[serde(borrow)]
    name: &'a str,

    #[serde(borrow)]
    description: &'a str,

    count: u32, // Primitive types are always copied
}

fn zero_copy_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Zero-Copy Demo ===\n");

    let json = r#"{"name": "Product", "description": "A great product", "count": 42}"#;

    // No string allocation happens here!
    // name and description borrow from the `json` string
    let data: BorrowedData = serde_json::from_str(json)?;

    println!("Name: {}", data.name);
    println!("Description: {}", data.description);
    println!("Count: {}", data.count);

    // Demonstrate that borrowed data points to original string
    let name_ptr = data.name.as_ptr();
    let json_ptr = json.as_ptr();
    println!("\nName pointer: {:?}", name_ptr);
    println!("JSON pointer: {:?}", json_ptr);
    println!("Name borrows from JSON: {}", name_ptr > json_ptr);

    Ok(())
}

// Cow (Clone on Write) for flexible ownership
#[derive(Deserialize, Serialize, Debug)]
struct FlexibleData<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,

    #[serde(borrow)]
    description: Cow<'a, str>,
}

fn cow_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Cow (Clone on Write) Demo ===\n");

    // Simple string - will borrow (zero-copy)
    let json_simple = r#"{"name": "Product", "description": "Simple description"}"#;
    let data_simple: FlexibleData = serde_json::from_str(json_simple)?;
    println!("Simple case:");
    println!("  Name: {:?}", data_simple.name);
    println!("  Is borrowed: {}", matches!(data_simple.name, Cow::Borrowed(_)));

    // String with escapes - will own (needs allocation for unescaping)
    let json_escaped = r#"{"name": "Product \"Special\"", "description": "Line1\nLine2"}"#;
    let data_escaped: FlexibleData = serde_json::from_str(json_escaped)?;
    println!("\nEscaped case:");
    println!("  Name: {:?}", data_escaped.name);
    println!("  Is borrowed: {}", matches!(data_escaped.name, Cow::Borrowed(_)));

    Ok(())
}

// Zero-copy with binary data using serde_bytes
#[derive(Serialize, Deserialize, Debug)]
struct BinaryData {
    #[serde(with = "serde_bytes")]
    data: Vec<u8>,
}

fn serde_bytes_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== serde_bytes Demo ===\n");

    let data = BinaryData {
        data: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0xFF],
    };

    // With serde_bytes, binary data is more efficiently encoded
    let json = serde_json::to_string(&data)?;
    println!("JSON with serde_bytes: {}", json);

    // Compare with bincode (much more compact)
    let bincode_data = bincode::serialize(&data)?;
    println!("Bincode size: {} bytes", bincode_data.len());
    println!("Bincode data: {:?}", bincode_data);

    Ok(())
}

// Zero-copy with bincode
#[derive(Serialize, Deserialize, Debug)]
struct Record<'a> {
    id: u64,
    #[serde(borrow)]
    name: &'a str,
    #[serde(borrow, with = "serde_bytes")]
    data: &'a [u8],
}

fn bincode_zero_copy_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Bincode Zero-Copy Demo ===\n");

    // For owned data, we need to create it first
    let name = "Test Record";
    let payload = [1u8, 2, 3, 4, 5];

    let record = Record {
        id: 123,
        name,
        data: &payload,
    };

    // Serialize to bytes
    let encoded = bincode::serialize(&record)?;
    println!("Encoded size: {} bytes", encoded.len());
    println!("Encoded: {:?}", encoded);

    // For zero-copy deserialization, we need owned serialized data
    #[derive(Serialize, Deserialize, Debug)]
    struct OwnedRecord {
        id: u64,
        name: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    }

    let owned_record = OwnedRecord {
        id: 456,
        name: "Owned".to_string(),
        data: vec![10, 20, 30],
    };

    let encoded = bincode::serialize(&owned_record)?;
    let decoded: OwnedRecord = bincode::deserialize(&encoded)?;
    println!("\nDecoded: {:?}", decoded);

    Ok(())
}

// Borrowed vs Owned comparison
#[derive(Deserialize, Debug)]
struct OwnedPerson {
    name: String,
    email: String,
    bio: String,
}

#[derive(Deserialize, Debug)]
struct BorrowedPerson<'a> {
    #[serde(borrow)]
    name: &'a str,
    #[serde(borrow)]
    email: &'a str,
    #[serde(borrow)]
    bio: &'a str,
}

fn borrowed_vs_owned_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Borrowed vs Owned Comparison ===\n");

    // Large JSON with repeated parsing scenario
    let json = r#"{
        "name": "Alice Johnson",
        "email": "alice.johnson@example.com",
        "bio": "Software engineer with 10 years of experience in distributed systems and cloud computing."
    }"#;

    // Owned: allocates 3 Strings
    let owned: OwnedPerson = serde_json::from_str(json)?;
    println!("Owned version allocates Strings:");
    println!("  name: {} bytes", owned.name.len());
    println!("  email: {} bytes", owned.email.len());
    println!("  bio: {} bytes", owned.bio.len());

    // Borrowed: zero allocation for strings
    let borrowed: BorrowedPerson = serde_json::from_str(json)?;
    println!("\nBorrowed version uses references:");
    println!("  name: {} bytes (borrowed)", borrowed.name.len());
    println!("  email: {} bytes (borrowed)", borrowed.email.len());
    println!("  bio: {} bytes (borrowed)", borrowed.bio.len());

    println!("\nFor high-throughput parsing, borrowed can be 10x faster!");

    Ok(())
}

// Processing multiple records with zero-copy
fn batch_processing_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Batch Processing Demo ===\n");

    #[derive(Deserialize, Debug)]
    struct LogEntry<'a> {
        #[serde(borrow)]
        level: &'a str,
        #[serde(borrow)]
        message: &'a str,
        timestamp: u64,
    }

    // Simulated log lines (JSON Lines format)
    let log_data = r#"{"level": "INFO", "message": "Server started", "timestamp": 1000}
{"level": "DEBUG", "message": "Processing request", "timestamp": 1001}
{"level": "WARN", "message": "High memory usage", "timestamp": 1002}
{"level": "ERROR", "message": "Connection failed", "timestamp": 1003}
{"level": "INFO", "message": "Request completed", "timestamp": 1004}"#;

    println!("Processing log entries with zero-copy:");
    let mut error_count = 0;
    let mut warn_count = 0;

    for line in log_data.lines() {
        let entry: LogEntry = serde_json::from_str(line)?;

        match entry.level {
            "ERROR" => {
                error_count += 1;
                println!("  [ERROR] {}", entry.message);
            }
            "WARN" => {
                warn_count += 1;
                println!("  [WARN] {}", entry.message);
            }
            _ => {}
        }
    }

    println!("\nSummary: {} errors, {} warnings", error_count, warn_count);
    println!("All string comparisons used borrowed references (no allocation)");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Zero-Copy Deserialization Demo ===\n");

    zero_copy_demo()?;
    cow_demo()?;
    serde_bytes_demo()?;
    bincode_zero_copy_demo()?;
    borrowed_vs_owned_demo()?;
    batch_processing_demo()?;

    println!("\nZero-copy demo completed");
    Ok(())
}
