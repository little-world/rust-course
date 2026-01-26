//! Pattern 4: Alternative Map Implementations
//! IndexMap for Insertion Order Preservation
//!
//! Run with: cargo run --example p4_indexmap

use indexmap::IndexMap;

fn main() {
    println!("=== IndexMap for Insertion Order ===\n");

    ordered_json();

    // Compare with standard HashMap
    println!("\n=== Comparison with HashMap ===\n");
    compare_iteration_order();

    println!("\n=== Key Points ===");
    println!("1. IndexMap preserves insertion order");
    println!("2. Drop-in replacement for HashMap");
    println!("3. Perfect for ordered JSON/config files");
    println!("4. Also supports by-index access");
}

fn ordered_json() {
    let mut user_data: IndexMap<&str, String> = IndexMap::new();

    // The insertion order is preserved.
    user_data.insert("id", "123".to_string());
    user_data.insert("name", "Alice".to_string());
    user_data.insert("email", "alice@example.com".to_string());
    user_data.insert("role", "admin".to_string());

    // When serialized (e.g., to JSON), the fields will appear in the order they were inserted.
    println!("User data (insertion order preserved):");
    for (key, value) in &user_data {
        println!("  \"{}\": \"{}\"", key, value);
    }

    // Access by index
    println!("\nAccess by index:");
    if let Some((key, value)) = user_data.get_index(0) {
        println!("  First entry: {} = {}", key, value);
    }
    if let Some((key, value)) = user_data.get_index(2) {
        println!("  Third entry: {} = {}", key, value);
    }
}

fn compare_iteration_order() {
    use std::collections::HashMap;

    // Standard HashMap - order is NOT preserved
    let mut hashmap: HashMap<i32, &str> = HashMap::new();
    for i in 1..=5 {
        hashmap.insert(i, "value");
    }

    print!("HashMap iteration order: ");
    for (k, _) in &hashmap {
        print!("{} ", k);
    }
    println!("(unpredictable)");

    // IndexMap - order IS preserved
    let mut indexmap: IndexMap<i32, &str> = IndexMap::new();
    for i in 1..=5 {
        indexmap.insert(i, "value");
    }

    print!("IndexMap iteration order: ");
    for (k, _) in &indexmap {
        print!("{} ", k);
    }
    println!("(always 1 2 3 4 5)");
}
