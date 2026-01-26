// Pattern 1: Field Attributes
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Metadata {
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    // Rename: use "username" in JSON, but "name" in Rust code
    #[serde(rename = "username")]
    name: String,

    // Skip serializing if None (reduces JSON size)
    // Field is serialized as "middleName": "value" only when Some
    #[serde(skip_serializing_if = "Option::is_none")]
    middle_name: Option<String>,

    // Provide default value when deserializing if field is missing
    // If JSON doesn't include "age", this becomes 0
    #[serde(default)]
    age: u32,

    // Skip this field entirely (never serialize or deserialize)
    // Useful for runtime-only data or secrets
    #[serde(skip)]
    password_hash: String,

    // Accept multiple names during deserialization
    // Deserializes from "mail", "e-mail", or "email"
    #[serde(alias = "mail", alias = "e-mail")]
    email: String,

    // Flatten: merge nested struct's fields into parent
    // Instead of "metadata": {"created_at": ...}
    // you get "created_at": ... at the top level
    #[serde(flatten)]
    metadata: Metadata,
}

fn field_attributes_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Field Attributes Demo ===\n");

    let user = User {
        name: "Bob".to_string(),
        middle_name: None, // Won't appear in JSON
        age: 25,
        password_hash: "secret".to_string(), // Never serialized
        email: "bob@example.com".to_string(),
        metadata: Metadata {
            created_at: Some("2024-01-01".to_string()),
            updated_at: None, // Won't appear in JSON
        },
    };

    let json = serde_json::to_string_pretty(&user)?;
    println!("Serialized User:\n{}", json);
    // Note: "username" (not "name"), no "middle_name", no "password_hash"
    // "created_at" is at top level due to flatten

    // Deserialize with alias
    println!("\n--- Deserializing with alias ---\n");
    let json_with_alias = r#"{
        "username": "Alice",
        "mail": "alice@example.com"
    }"#;

    let user: User = serde_json::from_str(json_with_alias)?;
    println!("Deserialized from 'mail' alias: {:?}", user);
    println!("  email field: {}", user.email);

    Ok(())
}

// Demonstrate skip_serializing_if with custom functions
#[derive(Serialize, Deserialize, Debug)]
struct OptimizedData {
    name: String,

    // Skip if empty
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    tags: Vec<String>,

    // Skip if zero
    #[serde(skip_serializing_if = "is_zero", default)]
    count: u32,

    // Skip if default value
    #[serde(skip_serializing_if = "String::is_empty", default)]
    description: String,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

fn skip_serializing_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Skip Serializing If Demo ===\n");

    let data_minimal = OptimizedData {
        name: "Item".to_string(),
        tags: vec![],
        count: 0,
        description: String::new(),
    };

    let data_full = OptimizedData {
        name: "Item".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        count: 42,
        description: "A description".to_string(),
    };

    println!("Minimal data JSON:");
    println!("{}", serde_json::to_string_pretty(&data_minimal)?);
    // Only "name" appears

    println!("\nFull data JSON:");
    println!("{}", serde_json::to_string_pretty(&data_full)?);
    // All fields appear

    Ok(())
}

// Demonstrate rename_all with different cases
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CamelCaseStruct {
    first_name: String,
    last_name: String,
    email_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingSnakeCase {
    first_name: String,
    last_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct KebabCase {
    first_name: String,
    last_name: String,
}

fn rename_all_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Rename All Demo ===\n");

    let camel = CamelCaseStruct {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email_address: "john@example.com".to_string(),
    };
    println!("camelCase: {}", serde_json::to_string(&camel)?);

    let screaming = ScreamingSnakeCase {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
    };
    println!("SCREAMING_SNAKE_CASE: {}", serde_json::to_string(&screaming)?);

    let kebab = KebabCase {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
    };
    println!("kebab-case: {}", serde_json::to_string(&kebab)?);

    Ok(())
}

// Demonstrate default with custom function
fn default_port() -> u16 {
    8080
}

fn default_timeout() -> u64 {
    30
}

#[derive(Serialize, Deserialize, Debug)]
struct ServerConfig {
    host: String,

    #[serde(default = "default_port")]
    port: u16,

    #[serde(default = "default_timeout")]
    timeout_secs: u64,
}

fn default_functions_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Default Functions Demo ===\n");

    let minimal_json = r#"{"host": "localhost"}"#;
    let config: ServerConfig = serde_json::from_str(minimal_json)?;

    println!("Config from minimal JSON: {:?}", config);
    println!("  port (defaulted): {}", config.port);
    println!("  timeout_secs (defaulted): {}", config.timeout_secs);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Field Attributes Demo ===\n");

    field_attributes_demo()?;
    skip_serializing_demo()?;
    rename_all_demo()?;
    default_functions_demo()?;

    println!("\nField attributes demo completed");
    Ok(())
}
