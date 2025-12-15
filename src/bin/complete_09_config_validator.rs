use colored::Colorize;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

// =============================================================================
// Milestone 1: Rich error types with context
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Failed to parse config file at line {line}, column {col}: {message}")]
    ParseError { line: usize, col: usize, message: String },

    #[error("Missing required field: '{field}' in section [{section}]")]
    MissingField {
        section: String,
        field: String,
        suggestion: Option<String>,
    },

    #[error("Invalid type for field '{field}': expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
        location: Location,
    },

    #[error("Invalid value for field '{field}': {reason}")]
    InvalidValue {
        field: String,
        value: String,
        reason: String,
        location: Location,
    },

    #[error("Value {value} for field '{field}' is out of range (min: {min}, max: {max})")]
    OutOfRange {
        field: String,
        value: i64,
        min: i64,
        max: i64,
    },
}

impl ConfigError {
    pub fn parse_error(line: usize, col: usize, message: impl Into<String>) -> Self {
        Self::ParseError {
            line,
            col,
            message: message.into(),
        }
    }

    pub fn missing_field(
        section: impl Into<String>,
        field: impl Into<String>,
        suggestion: Option<String>,
    ) -> Self {
        Self::MissingField {
            section: section.into(),
            field: field.into(),
            suggestion,
        }
    }

    pub fn invalid_type(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        location: Location,
    ) -> Self {
        Self::InvalidType {
            field: field.into(),
            expected: expected.into(),
            actual: actual.into(),
            location,
        }
    }
}

// =============================================================================
// Milestone 2: Parsing with preserved error context
// =============================================================================

pub struct ConfigParser;

impl ConfigParser {
    pub fn parse_json(content: &str) -> Result<Value, ConfigError> {
        serde_json::from_str(content).map_err(ConfigError::from)
    }

    pub fn parse_toml(content: &str) -> Result<Value, ConfigError> {
        let toml_value: toml::Value = toml::from_str(content).map_err(ConfigError::from)?;
        serde_json::to_value(toml_value)
            .map_err(|err| ConfigError::parse_error(0, 0, format!("Failed to convert TOML: {err}")))
    }

    pub fn parse_file(path: &Path) -> Result<Value, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|err| ConfigError::parse_error(0, 0, format!("Failed to read {}: {err}", path.display())))?;

        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase());

        match format.as_deref() {
            Some("json") => Self::parse_json(&content),
            Some("toml") => Self::parse_toml(&content),
            _ => {
                let trimmed = content.trim_start();
                if trimmed.starts_with('{') || trimmed.starts_with('[') {
                    Self::parse_json(&content)
                } else {
                    Self::parse_toml(&content)
                }
            }
        }
    }
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

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::parse_error(0, 0, err.to_string())
    }
}

// =============================================================================
// Milestone 3: Validation with error accumulation
// =============================================================================

#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<ConfigError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
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

    pub fn count(&self) -> usize {
        self.errors.len()
    }
}

pub struct Validator;

impl Validator {
    pub fn validate_config(config: &Value) -> Result<(), Vec<ConfigError>> {
        let mut errors = ValidationErrors::new();
        Self::validate_database_section(config, &mut errors);
        Self::validate_server_section(config, &mut errors);
        errors.into_result(())
    }

    fn validate_database_section(config: &Value, errors: &mut ValidationErrors) {
        let Some(db) = config.get("database").and_then(|v| v.as_object()) else {
            errors.add(ConfigError::missing_field("root", "database", None));
            return;
        };

        match db.get("host") {
            Some(Value::String(_)) => {}
            Some(other) => errors.add(ConfigError::invalid_type(
                "database.host",
                "string",
                value_type_name(other).to_string(),
                Location { line: 0, column: 0 },
            )),
            None => errors.add(ConfigError::missing_field("database", "host", None)),
        }

        match db.get("port") {
            Some(Value::Number(num)) => {
                if let Some(value) = num.as_i64() {
                    if !(1..=65535).contains(&value) {
                        errors.add(ConfigError::OutOfRange {
                            field: "database.port".to_string(),
                            value,
                            min: 1,
                            max: 65535,
                        });
                    }
                } else {
                    errors.add(ConfigError::invalid_type(
                        "database.port",
                        "integer",
                        "non-integer number",
                        Location { line: 0, column: 0 },
                    ));
                }
            }
            Some(other) => errors.add(ConfigError::invalid_type(
                "database.port",
                "integer",
                value_type_name(other).to_string(),
                Location { line: 0, column: 0 },
            )),
            None => errors.add(ConfigError::missing_field("database", "port", None)),
        }

        if let Some(value) = db.get("max_connections") {
            match value.as_i64() {
                Some(n) if (1..=500).contains(&n) => {}
                Some(n) => errors.add(ConfigError::OutOfRange {
                    field: "database.max_connections".to_string(),
                    value: n,
                    min: 1,
                    max: 500,
                }),
                None => errors.add(ConfigError::invalid_type(
                    "database.max_connections",
                    "integer",
                    value_type_name(value).to_string(),
                    Location { line: 0, column: 0 },
                )),
            }
        }
    }

    fn validate_server_section(config: &Value, errors: &mut ValidationErrors) {
        let Some(server) = config.get("server").and_then(|v| v.as_object()) else {
            errors.add(ConfigError::missing_field("root", "server", None));
            return;
        };

        match server.get("address") {
            Some(Value::String(_)) => {}
            Some(other) => errors.add(ConfigError::invalid_type(
                "server.address",
                "string",
                value_type_name(other).to_string(),
                Location { line: 0, column: 0 },
            )),
            None => errors.add(ConfigError::missing_field("server", "address", None)),
        }

        match server.get("timeout") {
            Some(value) => match value.as_i64() {
                Some(n) if n > 0 => {}
                Some(n) => errors.add(ConfigError::InvalidValue {
                    field: "server.timeout".to_string(),
                    value: n.to_string(),
                    reason: "timeout must be positive".to_string(),
                    location: Location { line: 0, column: 0 },
                }),
                None => errors.add(ConfigError::invalid_type(
                    "server.timeout",
                    "integer",
                    value_type_name(value).to_string(),
                    Location { line: 0, column: 0 },
                )),
            },
            None => errors.add(ConfigError::missing_field("server", "timeout", None)),
        }
    }

    pub fn validate_database_with_suggestions(config: &Value) -> Result<(), ConfigError> {
        const VALID_FIELDS: &[&str] = &["host", "port", "username", "password", "max_connections"];
        validate_field_exists(config, "database", "host", VALID_FIELDS)?;
        validate_field_exists(config, "database", "port", VALID_FIELDS)?;
        Ok(())
    }
}

// =============================================================================
// Milestone 4: Suggestions and typo recovery
// =============================================================================

pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut matrix = vec![vec![0; b_chars.len() + 1]; a_chars.len() + 1];

    for i in 0..=a_chars.len() {
        matrix[i][0] = i;
    }
    for j in 0..=b_chars.len() {
        matrix[0][j] = j;
    }

    for i in 1..=a_chars.len() {
        for j in 1..=b_chars.len() {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_chars.len()][b_chars.len()]
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

pub fn validate_field_exists(
    config: &Value,
    section: &str,
    field: &str,
    valid_fields: &[&str],
) -> Result<(), ConfigError> {
    let Some(section_obj) = config.get(section).and_then(|v| v.as_object()) else {
        return Err(ConfigError::missing_field(section, field, None));
    };

    if section_obj.contains_key(field) {
        Ok(())
    } else {
        let suggestion = find_similar_field(field, valid_fields);
        Err(ConfigError::missing_field(section, field, suggestion))
    }
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) => {
            if n.is_i64() {
                "integer"
            } else if n.is_u64() {
                "integer"
            } else {
                "float"
            }
        }
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

// =============================================================================
// Milestone 5: Schema validation with builder pattern
// =============================================================================

type ValidatorFn = Box<dyn Fn(&Value) -> Result<(), String> + Send + Sync>;

#[derive(Default)]
pub struct FieldSchema {
    path: String,
    required: bool,
    validators: Vec<ValidatorFn>,
}

impl FieldSchema {
    pub fn new(path: impl Into<String>, required: bool) -> Self {
        Self {
            path: path.into(),
            required,
            validators: Vec::new(),
        }
    }

    pub fn add_validator(&mut self, validator: ValidatorFn) {
        self.validators.push(validator);
    }

    pub fn validate(&self, value: &Value) -> Result<(), ConfigError> {
        for validator in &self.validators {
            if let Err(reason) = validator(value) {
                return Err(ConfigError::InvalidValue {
                    field: self.path.clone(),
                    value: value.to_string(),
                    reason,
                    location: Location { line: 0, column: 0 },
                });
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Schema {
    fields: HashMap<String, FieldSchema>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn add_field(&mut self, field: FieldSchema) {
        self.fields.insert(field.path.clone(), field);
    }

    pub fn validate(&self, config: &Value) -> Result<(), Vec<ConfigError>> {
        let mut errors = ValidationErrors::new();

        for field in self.fields.values() {
            match get_nested_value(config, &field.path) {
                Some(value) => {
                    if let Err(err) = field.validate(value) {
                        errors.add(err);
                    }
                }
                None if field.required => errors.add(ConfigError::missing_field(
                    parent_section(&field.path),
                    &field.path,
                    None,
                )),
                None => {}
            }
        }

        errors.into_result(())
    }
}

fn get_nested_value<'a>(config: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = config;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn parent_section(path: &str) -> String {
    path.split('.').next().unwrap_or(path).to_string()
}

pub struct SchemaBuilder {
    fields: Vec<FieldSchema>,
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self { fields: Vec::new() }
    }

    pub fn required_field(
        mut self,
        path: &str,
        validator: impl Fn(&Value) -> Result<(), String> + 'static + Send + Sync,
    ) -> Self {
        let mut field = FieldSchema::new(path, true);
        field.add_validator(Box::new(validator));
        self.fields.push(field);
        self
    }

    pub fn optional_field(
        mut self,
        path: &str,
        validator: impl Fn(&Value) -> Result<(), String> + 'static + Send + Sync,
    ) -> Self {
        let mut field = FieldSchema::new(path, false);
        field.add_validator(Box::new(validator));
        self.fields.push(field);
        self
    }

    pub fn build(self) -> Schema {
        let mut schema = Schema::new();
        for field in self.fields {
            schema.add_field(field);
        }
        schema
    }
}

pub mod validators {
    use super::*;

    pub fn is_string() -> impl Fn(&Value) -> Result<(), String> {
        |value| {
            if value.as_str().is_some() {
                Ok(())
            } else {
                Err("must be string".to_string())
            }
        }
    }

    pub fn is_integer() -> impl Fn(&Value) -> Result<(), String> {
        |value| {
            if value.as_i64().is_some() {
                Ok(())
            } else {
                Err("must be integer".to_string())
            }
        }
    }

    pub fn in_range(min: i64, max: i64) -> impl Fn(&Value) -> Result<(), String> {
        move |value| {
            let Some(n) = value.as_i64() else {
                return Err("must be integer".to_string());
            };
            if (min..=max).contains(&n) {
                Ok(())
            } else {
                Err(format!("must be between {min} and {max}"))
            }
        }
    }

    pub fn matches_pattern(pattern: &str) -> impl Fn(&Value) -> Result<(), String> {
        let regex = Regex::new(pattern).expect("invalid regex pattern");
        move |value| {
            let Some(text) = value.as_str() else {
                return Err("must be string".to_string());
            };
            if regex.is_match(text) {
                Ok(())
            } else {
                Err("does not match expected pattern".to_string())
            }
        }
    }
}

// =============================================================================
// Milestone 6: Formatted error output
// =============================================================================

pub struct ErrorFormatter;

impl ErrorFormatter {
    pub fn format_errors(errors: &[ConfigError], source: &str) -> String {
        if errors.is_empty() {
            return "Configuration valid".green().to_string();
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut output = String::new();
        output.push_str(&format!(
            "{}\n{}\n\n",
            "Configuration Validation Errors".bold().red(),
            "=".repeat(60)
        ));

        for (idx, error) in errors.iter().enumerate() {
            output.push_str(&Self::format_single_error(error, idx + 1, &lines));
            output.push('\n');
        }

        output.push_str(&Self::format_summary(errors));
        output
    }

    pub fn format_single_error(error: &ConfigError, number: usize, lines: &[&str]) -> String {
        let mut output = String::new();
        output.push_str(&format!("{}. {}\n", number, error));

        match error {
            ConfigError::ParseError { line, col, .. } => {
                output.push_str(&Self::format_context(*line, *col, lines));
            }
            ConfigError::InvalidType { location, .. }
            | ConfigError::InvalidValue { location, .. } => {
                if location.line > 0 {
                    output.push_str(&Self::format_context(location.line, location.column, lines));
                }
            }
            ConfigError::MissingField { suggestion, .. } => {
                if let Some(s) = suggestion {
                    output.push_str(&format!("   Hint: Did you mean '{}'?\n", s));
                }
            }
            ConfigError::OutOfRange { .. } => {}
        }

        output
    }

    pub fn format_context(line: usize, col: usize, lines: &[&str]) -> String {
        if line == 0 || line - 1 >= lines.len() {
            return String::new();
        }

        let mut output = String::new();
        if line > 1 {
            output.push_str(&format!(
                "   {} | {}\n",
                format!("{:3}", line - 1).blue(),
                lines[line - 2]
            ));
        }

        output.push_str(&format!(
            " > {} | {}\n",
            format!("{:3}", line).yellow(),
            lines[line - 1]
        ));

        let pointer_col = col.saturating_sub(1);
        let caret_line = format!(
            "     {}{}\n",
            " ".repeat(pointer_col),
            "^^^".green()
        );
        output.push_str(&caret_line);

        if line < lines.len() {
            output.push_str(&format!(
                "   {} | {}\n",
                format!("{:3}", line + 1).blue(),
                lines[line]
            ));
        }

        output
    }

    pub fn format_summary(errors: &[ConfigError]) -> String {
        let count = errors.len();
        let plural = if count == 1 { "" } else { "s" };
        format!("Summary: {count} error{plural} found\n")
    }

    pub fn should_use_colors() -> bool {
        std::env::var("NO_COLOR").is_err()
    }
}

pub fn format_validation_result(result: Result<(), Vec<ConfigError>>, source: &str) -> String {
    match result {
        Ok(()) => "Configuration valid".green().to_string(),
        Err(errors) => ErrorFormatter::format_errors(&errors, source),
    }
}

// =============================================================================
// Example usage
// =============================================================================

fn main() {
    let sample = r#"{
  "database": {
    "host": "localhost",
    "port": 5432,
    "max_connections": 200
  },
  "server": {
    "address": "0.0.0.0:8080",
    "timeout": 30
  }
}"#;

    let config = ConfigParser::parse_json(sample).expect("valid sample config");
    match Validator::validate_config(&config) {
        Ok(()) => println!("{}", "✓ Configuration valid".green()),
        Err(errors) => println!("{}", ErrorFormatter::format_errors(&errors, sample)),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Write;
    use std::path::Path;
    use tempfile::NamedTempFile;
    #[test]
    fn test_parse_error_display() {
        let error = ConfigError::ParseError {
            line: 10,
            col: 5,
            message: "unexpected token".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("line 10"));
        assert!(display.contains("column 5"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_missing_field_with_suggestion() {
        let error = ConfigError::MissingField {
            section: "database".to_string(),
            field: "port".to_string(),
            suggestion: Some("host".to_string()),
        };
        let display = format!("{}", error);
        assert!(display.contains("database"));
        assert!(display.contains("port"));
    }

    #[test]
    fn test_invalid_type_error() {
        let error = ConfigError::InvalidType {
            field: "timeout".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
            location: Location { line: 5, column: 10 },
        };
        let display = format!("{}", error);
        assert!(display.contains("timeout"));
        assert!(display.contains("expected integer"));
        assert!(display.contains("got string"));
    }

    #[test]
    fn test_out_of_range_error() {
        let error = ConfigError::OutOfRange {
            field: "max_connections".to_string(),
            value: 1000,
            min: 1,
            max: 500,
        };
        let display = format!("{}", error);
        assert!(display.contains("1000"));
        assert!(display.contains("min: 1"));
        assert!(display.contains("max: 500"));
    }

    #[test]
    fn test_error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ConfigError>();
        assert_sync::<ConfigError>();
    }

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"database": {"host": "localhost", "port": 5432}}"#;
        let result = ConfigParser::parse_json(json);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"database": {"host": "localhost", "port": 5432}"#;
        let result = ConfigParser::parse_json(json);
        assert!(matches!(result, Err(ConfigError::ParseError { .. })));
    }

    #[test]
    fn test_parse_valid_toml() {
        let toml = "[database]\nhost = \"localhost\"\nport = 5432\n";
        assert!(ConfigParser::parse_toml(toml).is_ok());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = "[database]\nhost = \"localhost\nport = 5432\n";
        assert!(matches!(
            ConfigParser::parse_toml(toml),
            Err(ConfigError::ParseError { .. })
        ));
    }

    #[test]
    fn test_parse_file_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"key": "value"}}"#).unwrap();
        assert!(ConfigParser::parse_file(file.path()).is_ok());
    }

    #[test]
    fn test_parse_file_not_found() {
        let result = ConfigParser::parse_file(Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_errors_accumulation() {
        let mut errors = ValidationErrors::new();
        assert!(!errors.has_errors());
        assert_eq!(errors.count(), 0);

        errors.add(ConfigError::missing_field("db", "host", None));
        assert!(errors.has_errors());
        assert_eq!(errors.count(), 1);

        errors.add(ConfigError::OutOfRange {
            field: "port".to_string(),
            value: 70000,
            min: 1,
            max: 65535,
        });
        assert_eq!(errors.count(), 2);
    }

    #[test]
    fn test_validation_errors_into_result_failure() {
        let mut errors = ValidationErrors::new();
        errors.add(ConfigError::missing_field("db", "host", None));
        let result = errors.into_result(());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_with_multiple_errors() {
        let config = json!({
            "database": {
                "port": "invalid",
                "max_connections": 1000
            },
            "server": {
                "timeout": -5
            }
        });
        let result = Validator::validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().len() >= 3);
    }

    #[test]
    fn test_validate_valid_config() {
        let config = json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "max_connections": 100
            },
            "server": {
                "address": "0.0.0.0:8080",
                "timeout": 30
            }
        });
        assert!(Validator::validate_config(&config).is_ok());
    }

    #[test]
    fn test_continue_validation_after_error() {
        let config = json!({
            "database": {
                "port": "bad",
                "max_connections": -1
            }
        });
        let errors = Validator::validate_config(&config).unwrap_err();
        assert!(errors.len() >= 3);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("sitting", "kitten"), 3);
    }

    #[test]
    fn test_find_similar_field_typo() {
        let valid = &["timeout", "retries", "max_connections"];
        assert_eq!(
            find_similar_field("timout", valid),
            Some("timeout".to_string())
        );
    }

    #[test]
    fn test_validate_field_exists_with_suggestion() {
        let config = json!({
            "database": {
                "hst": "localhost"
            }
        });
        let result = validate_field_exists(&config, "database", "host", &["host", "port"]);
        assert!(matches!(result, Err(ConfigError::MissingField { .. })));
    }

    #[test]
    fn test_field_schema_validation() {
        let mut field = FieldSchema::new("port", true);
        field.add_validator(Box::new(validators::is_integer()));
        field.add_validator(Box::new(validators::in_range(1, 10)));
        assert!(field.validate(&json!(5)).is_ok());
        assert!(field.validate(&json!(20)).is_err());
    }

    #[test]
    fn test_schema_builder_and_validation() {
        let schema = SchemaBuilder::new()
            .required_field("database.host", validators::is_string())
            .required_field("database.port", validators::in_range(1, 65535))
            .optional_field("database.timeout", validators::is_integer())
            .build();

        let config = json!({"database": {"host": "localhost", "port": 8080}});
        assert!(schema.validate(&config).is_ok());

        let bad = json!({"database": {"host": 42, "port": 70000}});
        assert!(schema.validate(&bad).is_err());
    }

    #[test]
    fn test_pattern_validator() {
        let schema = SchemaBuilder::new()
            .required_field("email", validators::matches_pattern(r"^[^@]+@[^@]+\.[^@]+$"))
            .build();
        assert!(schema.validate(&json!({"email": "user@example.com"})).is_ok());
        assert!(schema.validate(&json!({"email": "not-an-email"})).is_err());
    }

    #[test]
    fn test_error_formatter() {
        let source = "{\n  \"key\": invalid\n}";
        let lines: Vec<&str> = source.lines().collect();
        let error = ConfigError::ParseError {
            line: 2,
            col: 10,
            message: "unexpected token".to_string(),
        };
        let formatted = ErrorFormatter::format_single_error(&error, 1, &lines);
        assert!(formatted.contains("unexpected token"));
        assert!(formatted.contains("2"));
    }

    #[test]
    fn test_format_validation_result_success() {
        let formatted = format_validation_result(Ok(()), "");
        assert!(formatted.contains("valid"));
    }

    #[test]
    fn test_format_validation_result_errors() {
        let errors = vec![ConfigError::missing_field("db", "host", None)];
        let formatted = format_validation_result(Err(errors), "");
        assert!(formatted.contains("Missing required field"));
    }

    #[test]
    fn test_context_formatting() {
        let source = "line1\nline2\nline3\nline4";
        let lines: Vec<&str> = source.lines().collect();
        let context = ErrorFormatter::format_context(3, 2, &lines);
        assert!(context.contains("line2"));
        assert!(context.contains("line3"));
        assert!(context.contains("line4"));
    }

    #[test]
    fn test_no_color_environment() {
        std::env::set_var("NO_COLOR", "1");
        assert!(!ErrorFormatter::should_use_colors());
        std::env::remove_var("NO_COLOR");
        assert!(ErrorFormatter::should_use_colors());
    }

    #[test]
    fn test_summary_pluralization() {
        let one = vec![ConfigError::missing_field("db", "host", None)];
        assert!(ErrorFormatter::format_summary(&one).contains("1 error"));

        let two = vec![
            ConfigError::missing_field("db", "host", None),
            ConfigError::missing_field("db", "port", None),
        ];
        assert!(ErrorFormatter::format_summary(&two).contains("2 errors"));
    }
}
