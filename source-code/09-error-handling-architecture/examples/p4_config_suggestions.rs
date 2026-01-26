//! Pattern 4: Custom Error Types with Context
//! Example: Errors with Actionable Suggestions
//!
//! Run with: cargo run --example p4_config_suggestions

use thiserror::Error;

/// Configuration error with helpful suggestions.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing field: {field}\n  Hint: {suggestion}")]
    MissingField { field: String, suggestion: String },

    #[error("Invalid value for '{field}': {value}\n  Expected: {expected}")]
    InvalidValue {
        field: String,
        value: String,
        expected: String,
    },

    #[error("Type mismatch for '{field}': expected {expected}, got {actual}")]
    TypeMismatch {
        field: String,
        expected: String,
        actual: String,
    },
}

impl ConfigError {
    /// Create a missing field error with context-aware suggestion.
    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        let suggestion = match field.as_str() {
            "database_url" => "Add DATABASE_URL to your .env file or set it as an environment variable".into(),
            "api_key" => "Set API_KEY environment variable or add it to config.toml".into(),
            "port" => "Add 'port = 8080' to your config file".into(),
            "host" => "Add 'host = \"127.0.0.1\"' to your config file".into(),
            _ => format!("Add '{}' to your configuration", field),
        };
        ConfigError::MissingField { field, suggestion }
    }

    /// Create an invalid value error with expected format.
    pub fn invalid_value(field: &str, value: &str, expected: &str) -> Self {
        ConfigError::InvalidValue {
            field: field.into(),
            value: value.into(),
            expected: expected.into(),
        }
    }
}

/// Simulated config validation.
fn validate_config(config: &std::collections::HashMap<String, String>) -> Result<(), ConfigError> {
    // Check required fields
    if !config.contains_key("database_url") {
        return Err(ConfigError::missing_field("database_url"));
    }

    // Validate port
    if let Some(port) = config.get("port") {
        if port.parse::<u16>().is_err() {
            return Err(ConfigError::invalid_value("port", port, "a number between 1-65535"));
        }
    }

    // Validate log_level
    if let Some(level) = config.get("log_level") {
        let valid = ["debug", "info", "warn", "error"];
        if !valid.contains(&level.as_str()) {
            return Err(ConfigError::invalid_value(
                "log_level",
                level,
                "one of: debug, info, warn, error",
            ));
        }
    }

    Ok(())
}

fn main() {
    println!("=== Errors with Actionable Suggestions ===\n");

    use std::collections::HashMap;

    // Missing required field
    println!("=== Missing Field ===");
    let empty_config: HashMap<String, String> = HashMap::new();
    if let Err(e) = validate_config(&empty_config) {
        println!("{}\n", e);
    }

    // Invalid port value
    println!("=== Invalid Value ===");
    let mut bad_port: HashMap<String, String> = HashMap::new();
    bad_port.insert("database_url".into(), "postgres://...".into());
    bad_port.insert("port".into(), "not_a_number".into());
    if let Err(e) = validate_config(&bad_port) {
        println!("{}\n", e);
    }

    // Invalid log level
    println!("=== Invalid Enum Value ===");
    let mut bad_level: HashMap<String, String> = HashMap::new();
    bad_level.insert("database_url".into(), "postgres://...".into());
    bad_level.insert("log_level".into(), "verbose".into());
    if let Err(e) = validate_config(&bad_level) {
        println!("{}\n", e);
    }

    // Valid config
    println!("=== Valid Config ===");
    let mut valid: HashMap<String, String> = HashMap::new();
    valid.insert("database_url".into(), "postgres://localhost/db".into());
    valid.insert("port".into(), "8080".into());
    valid.insert("log_level".into(), "info".into());
    match validate_config(&valid) {
        Ok(_) => println!("Config is valid!\n"),
        Err(e) => println!("Error: {}\n", e),
    }

    // Show different missing field suggestions
    println!("=== Context-Aware Suggestions ===");
    let fields = ["database_url", "api_key", "port", "host", "custom_field"];
    for field in fields {
        let err = ConfigError::missing_field(field);
        println!("{}\n", err);
    }

    println!("=== Key Points ===");
    println!("1. Suggestions tell users HOW to fix the problem");
    println!("2. Include expected formats/values in error message");
    println!("3. Context-aware hints based on field name");
    println!("4. Makes errors actionable, not just informative");
}
