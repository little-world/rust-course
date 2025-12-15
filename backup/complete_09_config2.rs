use colored::*;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

//==============================================================================
// Part 1: Error Types
//==============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Parse error at line {line}, column {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Missing required field '{field}' in section [{section}]")]
    MissingField {
        section: String,
        field: String,
        suggestion: Option<String>,
    },

    #[error("Invalid type for '{field}': expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
        location: Location,
    },

    #[error("Invalid value for '{field}': {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
        location: Location,
    },

    #[error("Value {value} for '{field}' out of range ({min}..{max})")]
    OutOfRange {
        field: String,
        value: i64,
        min: i64,
        max: i64,
    },
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError {
            line: err.line(),
            col: err.column(),
            message: err.to_string(),
        }
    }
}

//==============================================================================
// Part 2: Parser
//==============================================================================

pub struct ConfigParser;

impl ConfigParser {
    pub fn parse_json(content: &str) -> Result<Value, ConfigError> {
        serde_json::from_str(content).map_err(ConfigError::from)
    }

    pub fn parse_file(path: &Path) -> Result<Value, ConfigError> {
        let content = fs::read_to_string(path).map_err(|e| ConfigError::ParseError {
            line: 0,
            col: 0,
            message: format!("Failed to read file: {}", e),
        })?;

        Self::parse_json(&content)
    }
}

//==============================================================================
// Part 3: Validation
//==============================================================================

#[derive(Default)]
pub struct ValidationErrors {
    errors: Vec<ConfigError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ConfigError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> Result<T, Vec<ConfigError>> {
        if self.has_errors() {
            Err(self.errors)
        } else {
            Ok(value)
        }
    }
}

//==============================================================================
// Part 4: Suggestions
//==============================================================================

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

pub fn find_similar_field(typo: &str, valid_fields: &[&str]) -> Option<String> {
    let mut best_match = None;
    let mut best_distance = usize::MAX;
    const MAX_DISTANCE: usize = 2;

    for &field in valid_fields {
        let distance = levenshtein_distance(typo, field);
        if distance < best_distance && distance <= MAX_DISTANCE {
            best_distance = distance;
            best_match = Some(field.to_string());
        }
    }

    best_match
}

//==============================================================================
// Part 5: Schema Validation
//==============================================================================

type ValidatorFn = Box<dyn Fn(&Value) -> Result<(), String> + Send + Sync>;

pub struct FieldSchema {
    path: String,
    required: bool,
    validators: Vec<ValidatorFn>,
}

pub struct Schema {
    fields: HashMap<String, FieldSchema>,
}

impl Schema {
    pub fn validate(&self, config: &Value) -> Result<(), Vec<ConfigError>> {
        let mut errors = ValidationErrors::new();

        for (path, field_schema) in &self.fields {
            let value = Self::get_nested_value(config, path);

            if field_schema.required && value.is_none() {
                errors.add(ConfigError::MissingField {
                    section: path.split('.').next().unwrap_or("").to_string(),
                    field: path.clone(),
                    suggestion: None,
                });
                continue;
            }

            if let Some(value) = value {
                for validator in &field_schema.validators {
                    if let Err(reason) = validator(value) {
                        errors.add(ConfigError::InvalidValue {
                            field: path.clone(),
                            value: format!("{}", value),
                            reason,
                            location: Location { line: 0, column: 0 },
                        });
                    }
                }
            }
        }

        errors.into_result(())
    }

    fn get_nested_value<'a>(config: &'a Value, path: &str) -> Option<&'a Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = config;

        for part in parts {
            current = current.get(part)?;
        }

        Some(current)
    }
}

pub struct SchemaBuilder {
    fields: Vec<FieldSchema>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        SchemaBuilder { fields: Vec::new() }
    }

    pub fn required_field(
        mut self,
        path: &str,
        validator: impl Fn(&Value) -> Result<(), String> + 'static + Send + Sync,
    ) -> Self {
        self.fields.push(FieldSchema {
            path: path.to_string(),
            required: true,
            validators: vec![Box::new(validator)],
        });
        self
    }

    pub fn build(self) -> Schema {
        Schema {
            fields: self.fields.into_iter().map(|f| (f.path.clone(), f)).collect(),
        }
    }
}

//==============================================================================
// Part 6: Formatted Output
//==============================================================================

pub struct ErrorFormatter;

impl ErrorFormatter {
    pub fn format_errors(errors: &[ConfigError], source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let mut output = String::new();

        output.push_str(&format!("\n{}\n", "Configuration Validation Errors:".red().bold()));
        output.push_str(&format!("{}\n\n", "=".repeat(60)));

        for (i, error) in errors.iter().enumerate() {
            output.push_str(&format!("{}. {}\n\n", i + 1, error));
        }

        output.push_str(&format!(
            "{} {} error{}\n",
            "Summary:".bold(),
            errors.len(),
            if errors.len() == 1 { "" } else { "s" }
        ));

        output
    }
}

//==============================================================================
// Example Usage
//==============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Configuration Validator Examples ===\n");

    // Example 1: Parse and validate
    println!("Example 1: Basic Validation");
    {
        let config_str = r#"
{
  "database": {
    "host": "localhost",
    "port": 5432,
    "max_connections": 100
  },
  "server": {
    "address": "0.0.0.0:8080",
    "timeout": 30
  }
}
"#;

        let schema = SchemaBuilder::new()
            .required_field("database.host", |v| {
                v.as_str().ok_or("must be string")?;
                Ok(())
            })
            .required_field("database.port", |v| {
                let port = v.as_i64().ok_or("must be integer")?;
                if port < 1 || port > 65535 {
                    return Err("must be between 1 and 65535".to_string());
                }
                Ok(())
            })
            .build();

        let config = ConfigParser::parse_json(config_str)?;
        match schema.validate(&config) {
            Ok(()) => println!("✓ Configuration is valid!"),
            Err(errors) => println!("{}", ErrorFormatter::format_errors(&errors, config_str)),
        }
    }
    println!();

    // Example 2: Multiple errors
    println!("Example 2: Multiple Validation Errors");
    {
        let config_str = r#"
{
  "database": {
    "port": "invalid",
    "max_connections": 1000
  }
}
"#;

        let schema = SchemaBuilder::new()
            .required_field("database.host", |v| {
                v.as_str().ok_or("must be string")?;
                Ok(())
            })
            .required_field("database.port", |v| {
                v.as_i64().ok_or("must be integer")?;
                Ok(())
            })
            .required_field("database.max_connections", |v| {
                let n = v.as_i64().ok_or("must be integer")?;
                if n > 500 {
                    return Err("must be <= 500".to_string());
                }
                Ok(())
            })
            .build();

        let config = ConfigParser::parse_json(config_str)?;
        match schema.validate(&config) {
            Ok(()) => println!("✓ Valid"),
            Err(errors) => {
                println!("{}", ErrorFormatter::format_errors(&errors, config_str));
                println!("Found {} errors", errors.len());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_validation_pipeline() {
        let config = r#"{"database": {"host": "localhost", "port": 5432}}"#;

        let schema = SchemaBuilder::new()
            .required_field("database.host", |v| {
                v.as_str().ok_or("must be string")?;
                Ok(())
            })
            .required_field("database.port", |v| {
                v.as_i64().ok_or("must be integer")?;
                Ok(())
            })
            .build();

        let parsed = ConfigParser::parse_json(config).unwrap();
        let result = schema.validate(&parsed);

        assert!(result.is_ok());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_find_similar_field() {
        let fields = &["timeout", "retries", "max_connections"];
        assert_eq!(find_similar_field("timout", fields), Some("timeout".to_string()));
    }
}
