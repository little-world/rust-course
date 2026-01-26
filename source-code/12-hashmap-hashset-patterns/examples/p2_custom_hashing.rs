//! Pattern 2: Custom Hashing and Equality
//! Case-Insensitive String Keys
//!
//! Run with: cargo run --example p2_custom_hashing

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

fn main() {
    println!("=== Custom Hashing and Equality ===\n");

    case_insensitive_headers();

    println!("\n=== Key Points ===");
    println!("1. If a == b, then hash(a) must == hash(b)");
    println!("2. Hash lowercase for case-insensitive hashing");
    println!("3. Use eq_ignore_ascii_case for comparison");
    println!("4. Perfect for HTTP headers, config keys, usernames");
}

#[derive(Debug, Eq)]
struct CaseInsensitiveString(String);

impl Hash for CaseInsensitiveString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash lowercase so "A" and "a" have the same hash.
        for byte in self.0.bytes() {
            byte.to_ascii_lowercase().hash(state);
        }
    }
}

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        // Compare the strings case-insensitively.
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

fn case_insensitive_headers() {
    let mut headers = HashMap::new();

    headers.insert(
        CaseInsensitiveString("Content-Type".to_string()),
        "application/json",
    );
    headers.insert(
        CaseInsensitiveString("X-Request-ID".to_string()),
        "12345",
    );
    headers.insert(
        CaseInsensitiveString("Authorization".to_string()),
        "Bearer token123",
    );

    println!("Inserted headers:");
    println!("  Content-Type: application/json");
    println!("  X-Request-ID: 12345");
    println!("  Authorization: Bearer token123");

    // Lookup is case-insensitive
    println!("\nCase-insensitive lookups:");

    let test_keys = [
        "content-type",
        "CONTENT-TYPE",
        "Content-Type",
        "x-request-id",
        "AUTHORIZATION",
    ];

    for key in test_keys {
        let lookup_key = CaseInsensitiveString(key.to_string());
        match headers.get(&lookup_key) {
            Some(value) => println!("  '{}' -> '{}'", key, value),
            None => println!("  '{}' -> not found", key),
        }
    }

    // Verify specific lookup
    let key = CaseInsensitiveString("content-type".to_string());
    assert_eq!(headers.get(&key), Some(&"application/json"));
    println!("\nAssertion passed: 'content-type' finds 'Content-Type' entry");
}
