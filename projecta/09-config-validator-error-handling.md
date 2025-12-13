
# Configuration Validator with Rich Error Context

### Problem Statement

Build a configuration file validator that parses TOML/JSON configuration files and validates them against a schema with comprehensive error reporting. The validator should collect ALL validation errors (not just the first), provide actionable error messages with suggestions, track error locations (line/column), and help users fix configuration problems quickly.

Your validator should support:
- Parsing configuration files (TOML or JSON)
- Validating against a schema (required fields, type constraints, value ranges)
- Collecting multiple errors in a single validation pass
- Reporting errors with file location, field path, actual vs expected values
- Suggesting fixes for common mistakes (typos, missing required fields)
- Distinguishing between parsing errors and validation errors

Example config validation:
```toml
[database]
host = "localhost"
port = "invalid"  # Should be number
max_connections = 1000  # Exceeds maximum of 500

[server]
# Missing required field: address
timeout = -5  # Should be positive
```

### Why It Matters

Configuration errors are among the most frustrating bugs in production systems. Poor error messages lead to trial-and-error debugging, wasting developer time. Good error handling in configuration validation catches all problems before deployment, provides actionable feedback, suggests corrections, and prevents cascading failures from misconfiguration.

This pattern applies to any validation system: API request validation, command-line argument parsing, data import validation, and compiler error reporting.

---

## Key Concepts Explained

This project demonstrates Rust's error handling patterns for building user-friendly validation systems.

### 1. thiserror for Ergonomic Error Types

Derive error trait implementations automatically:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Failed to parse at line {line}: {message}")]
    ParseError { line: usize, message: String },

    #[error("Missing field: {field}")]
    MissingField { field: String },
}

// Automatically implements:
// - Display (using #[error] messages)
// - Error trait
// - From conversions
```

**vs Manual implementation**:
```rust
// ❌ Tedious manual Display
impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::ParseError { line, message } =>
                write!(f, "Failed to parse at line {}: {}", line, message),
            Self::MissingField { field } =>
                write!(f, "Missing field: {}", field),
        }
    }
}

impl Error for ConfigError {}
```

**Benefit**: Less boilerplate, consistent error messages.

### 2. Structured Error Context

Include actionable information in errors:

```rust
#[derive(Error, Debug)]
enum ConfigError {
    #[error("Invalid type for '{field}' at line {line}: expected {expected}, got {actual}")]
    InvalidType {
        field: String,
        expected: String,
        actual: String,
        line: usize,
    },
}
```

**Good error**:
```
Invalid type for 'port' at line 5: expected integer, got string
```

**Bad error**:
```
Type error
```

**Context includes**:
- **What** went wrong (type mismatch)
- **Where** it happened (line 5, field 'port')
- **Expected** value (integer)
- **Actual** value (string)

### 3. Location Tracking

Track file position for precise error reporting:

```rust
struct Location {
    line: usize,
    column: usize,
}

#[derive(Error, Debug)]
enum ConfigError {
    #[error("Parse error at line {line}, col {col}")]
    ParseError { line: usize, col: usize, message: String },
}
```

**Why it matters**: Users can jump directly to problem in editor.

**Example**:
```
Error: Parse error at line 15, column 23: unexpected token ','
```

### 4. Error Conversion with From Trait

Convert library errors to domain errors:

```rust
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError {
            line: err.line(),
            col: err.column(),
            message: err.to_string(),
        }
    }
}

// Now can use ? operator
fn parse(s: &str) -> Result<Value, ConfigError> {
    let value: Value = serde_json::from_str(s)?;  // Auto-converts!
    Ok(value)
}
```

**Benefit**: `?` operator works across error types.

### 5. Error Accumulation (Not Fail-Fast)

Collect all errors, not just the first:

```rust
struct ValidationErrors {
    errors: Vec<ConfigError>,
}

impl ValidationErrors {
    fn add(&mut self, error: ConfigError) {
        self.errors.push(error);
    }

    fn into_result<T>(self, value: T) -> Result<T, Vec<ConfigError>> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self.errors)
        }
    }
}
```

**Usage**:
```rust
fn validate(config: &Config) -> Result<(), Vec<ConfigError>> {
    let mut errors = ValidationErrors::new();

    if config.port == 0 {
        errors.add(ConfigError::InvalidValue { ... });
    }

    if config.host.is_empty() {
        errors.add(ConfigError::MissingField { ... });
    }

    errors.into_result(())  // Returns all errors at once
}
```

**vs Fail-fast**:
```rust
// ❌ User must fix errors one at a time
fn validate_fail_fast(config: &Config) -> Result<(), ConfigError> {
    if config.port == 0 {
        return Err(ConfigError::InvalidValue { ... });  // Stops here
    }

    if config.host.is_empty() {
        return Err(ConfigError::MissingField { ... });  // Never reached
    }

    Ok(())
}
```

### 6. Suggestions in Errors

Provide hints for fixing problems:

```rust
#[derive(Error, Debug)]
enum ConfigError {
    #[error("Missing field '{field}' in [{section}]{}", suggestion_text(.suggestion))]
    MissingField {
        section: String,
        field: String,
        suggestion: Option<String>,
    },
}

fn suggestion_text(opt: &Option<String>) -> String {
    match opt {
        Some(s) => format!("\n  Hint: Did you mean '{}'?", s),
        None => String::new(),
    }
}
```

**Example output**:
```
Missing field 'host' in [database]
  Hint: Did you mean 'hostname'?
```

**Benefit**: Users know how to fix, not just what's wrong.

### 7. Pattern Matching on Errors

Handle different error types appropriately:

```rust
match result {
    Ok(config) => run_app(config),
    Err(errors) => {
        for error in errors {
            match error {
                ConfigError::ParseError { line, col, .. } => {
                    eprintln!("Syntax error at {}:{}", line, col);
                    std::process::exit(1);  // Fatal
                }
                ConfigError::MissingField { suggestion, .. } => {
                    eprintln!("Config incomplete: {}", error);
                    if let Some(hint) = suggestion {
                        eprintln!("  Hint: {}", hint);
                    }
                    // Continue showing other errors
                }
                _ => eprintln!("Error: {}", error),
            }
        }
    }
}
```

**Benefit**: Different error severities handled differently.

### 8. Error Display vs Debug

Two representations for different audiences:

```rust
#[derive(Error, Debug)]
#[error("Invalid port: {port}")]
struct PortError {
    port: u16,
}

// Display (user-facing)
println!("{}", error);
// Output: Invalid port: 70000

// Debug (developer-facing)
println!("{:?}", error);
// Output: PortError { port: 70000 }
```

**When to use**:
- **Display** (`{}`): End users, logs
- **Debug** (`{:?}`): Developers, diagnostics

### 9. Result Type for Error Propagation

Explicit error handling with Result:

```rust
fn validate_port(port: u16) -> Result<(), ConfigError> {
    if port == 0 || port > 65535 {
        return Err(ConfigError::OutOfRange { ... });
    }
    Ok(())
}

fn validate_config(cfg: &Config) -> Result<(), ConfigError> {
    validate_port(cfg.port)?;  // Propagates error
    validate_host(&cfg.host)?;
    Ok(())
}
```

**vs Exceptions** (other languages):
```java
// Can throw but not declared - runtime surprise!
void validate(Config cfg) {
    if (cfg.port == 0) throw new InvalidPort();
}
```

**Benefit**: Compiler forces error handling, no surprises.

### 10. Custom Error Types per Domain

Domain-specific errors vs generic errors:

```rust
// ✅ Domain-specific - actionable
enum ConfigError {
    MissingField { field: String },
    OutOfRange { min: i64, max: i64 },
}

// ❌ Generic - vague
enum Error {
    Generic(String),
}
```

**Benefit**: Type system enforces proper error handling for each case.

---

## Connection to This Project

Here's how each milestone applies these concepts to build production-grade configuration validation.

### Milestone 1: Basic Error Type with Context

**Concepts applied**:
- **thiserror derive**: Automatic Display and Error trait implementation
- **Structured errors**: Each variant has rich context fields
- **Location tracking**: Line and column numbers in errors
- **Suggestions**: Optional hints for fixing problems

**Why this matters**: Foundation of good error messages.

**Comparison**:

| Error Type | Information | User Experience |
|------------|-------------|-----------------|
| String | "error" | ❌ Vague, no context |
| Generic enum | "ParseError" | ❌ What line? What's wrong? |
| Rich context | "Parse error at line 5, col 10: unexpected ','" | ✅ Actionable, precise |

**Real-world impact**:
```rust
// ❌ Bad: Generic error
Err("Invalid config")
// User: What's invalid? Where? How do I fix it?

// ✅ Good: Rich context
Err(ConfigError::InvalidType {
    field: "port",
    expected: "integer",
    actual: "string",
    location: Location { line: 15, column: 10 }
})
// User: Ah, line 15, change port from string to integer!
```

---

### Milestone 2: Parse Configuration with Error Context

**Concepts applied**:
- **Error conversion**: `From<serde_json::Error>` for ConfigError
- **? operator**: Propagate errors with automatic conversion
- **Location preservation**: Extract line/column from parser errors
- **Format detection**: Auto-detect JSON vs TOML

**Why this matters**: Preserve location info through error transformations.

**Error flow**:
```rust
fn parse(s: &str) -> Result<Value, ConfigError> {
    serde_json::from_str(s)?  // serde_json::Error converted to ConfigError
}

// Conversion preserves location:
impl From<serde_json::Error> for ConfigError {
    fn from(e: serde_json::Error) -> Self {
        ConfigError::ParseError {
            line: e.line(),       // Preserved!
            col: e.column(),      // Preserved!
            message: e.to_string(),
        }
    }
}
```

**Without conversion**:
```rust
// ❌ Loses location info
fn parse(s: &str) -> Result<Value, ConfigError> {
    match serde_json::from_str(s) {
        Ok(v) => Ok(v),
        Err(_) => Err(ConfigError::ParseError {
            line: 0,  // Lost!
            col: 0,   // Lost!
            message: "parse failed".into(),
        })
    }
}
```

**Performance**: Parsing 1000 config files with errors:
- Without location: Debug time ~10 minutes (search for problem)
- With location: Debug time ~10 seconds (jump to line)
- **60× faster debugging**

---

### Milestone 3: Collect Multiple Errors

**Concepts applied**:
- **Error accumulation**: `ValidationErrors` collects all errors
- **Continue on error**: Don't stop at first failure
- **Result with Vec**: `Result<T, Vec<ConfigError>>`

**Why this matters**: One validation pass finds all problems.

**Comparison**:

| Approach | Iterations Needed | User Experience |
|----------|------------------|-----------------|
| Fail-fast | 10 (one per error) | ❌ Frustrating cycle |
| Collect-all | 1 (shows all errors) | ✅ Fix all at once |

**Example**: Config with 5 errors

**Fail-fast**:
```
Run 1: Error - missing 'host'
[Fix host]
Run 2: Error - invalid 'port' type
[Fix port]
Run 3: Error - 'timeout' negative
[Fix timeout]
Run 4: Error - 'max_connections' out of range
[Fix max_connections]
Run 5: Error - missing 'address'
[Fix address]
Run 6: Success! ✓

Time: 5 minutes (6 validation cycles)
```

**Collect-all**:
```
Run 1: 5 Errors:
  - Missing 'host'
  - Invalid 'port' type
  - 'timeout' negative
  - 'max_connections' out of range
  - Missing 'address'
[Fix all 5 issues]
Run 2: Success! ✓

Time: 30 seconds (1 validation cycle)
```

**10× faster feedback loop**.

**Implementation pattern**:
```rust
fn validate(cfg: &Config) -> Result<(), Vec<ConfigError>> {
    let mut errors = ValidationErrors::new();

    // Check all fields, accumulate errors
    if cfg.host.is_empty() {
        errors.add(ConfigError::MissingField { ... });
    }

    if cfg.port == 0 {
        errors.add(ConfigError::InvalidValue { ... });
    }

    if cfg.timeout < 0 {
        errors.add(ConfigError::InvalidValue { ... });
    }

    // Return all errors or success
    errors.into_result(())
}
```

---

### Project-Wide Benefits

**Concrete comparisons** - Validating config with 10 errors:

| Metric | Generic Errors | Structured Errors | Rich Context + Accumulation | Improvement |
|--------|---------------|-------------------|----------------------------|-------------|
| Error message quality | "error" | "Parse failed" | "Parse error at line 5, col 10: unexpected ','" | **Actionable** |
| Validation cycles | 10 | 10 | 1 | **10× faster** |
| Time to fix | ~10 min | ~5 min | ~30 sec | **20× faster** |
| Location info | No | No | Yes (line/col) | **Direct navigation** |
| Suggestions | No | No | Yes ("Did you mean 'host'?") | **Self-service fixes** |

**Real-world validation**:
- **Rust compiler**: Collects all errors, provides suggestions (same pattern!)
- **ESLint**: Shows all linting errors at once
- **TypeScript**: Type errors with locations and hints
- **Kubernetes**: Config validation with detailed error messages

**Production requirements met**:
- ✅ Actionable errors (what, where, how to fix)
- ✅ Location tracking (line/column for editor navigation)
- ✅ Error accumulation (fix all problems at once)
- ✅ Suggestions (hints for common mistakes)
- ✅ Type safety (compiler ensures all error cases handled)
- ✅ Ergonomic (thiserror reduces boilerplate)

**Developer experience impact**:
- **Before**: "Config invalid" → Guess and check → Frustration
- **After**: See all problems → Fix all → Done → Delight

This project teaches patterns used in production tools (compilers, linters, validators) that process millions of files daily with excellent error reporting.

---

### Milestone 1: Basic Error Type with Context

**Goal**: Define a comprehensive error type for configuration validation using thiserror.

**What to implement**:
- Define `ConfigError` enum with variants for different failure modes
- Use `thiserror` to derive `Display` and `Error` traits
- Include context in each variant (field name, location, expected/actual values)
- Implement custom display messages that are user-friendly

**Architecture**:
- Enums: `ConfigError`
- Structs: `Location`
- Fields (Location): `line: usize`, `column: usize`
- Functions:
  - `ConfigError` variants with contextual fields
  - Automatic `Display` implementation from thiserror

---

**Starter Code**:

```rust
use thiserror::Error;

/// Location in configuration file
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,                   // Line number (1-indexed)       
    pub column: usize,                 // Column number (1-indexed)   
}

/// Comprehensive error type for configuration validation
///
/// Enum:
/// - ConfigError: All possible validation errors
///
/// Variants:
/// - ParseError: Syntax errors in config file
/// - MissingField: Required field not found
/// - InvalidType: Field has wrong type
/// - InvalidValue: Field value doesn't meet constraints
/// - OutOfRange: Numeric value outside allowed range
///
/// Role: Provide rich error context for debugging
#[derive(Error, Debug, Clone)]
pub enum ConfigError {
    #[error("Failed to parse config file at line {line}, column {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    #[error("Missing required field: '{field}' in section [{section}]")]
    MissingField {
        section: String,
        field: String,
        suggestion: Option<String>,  // Suggest similar field names
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
    /// Create parse error
    /// Role: Construct parse error with location
    pub fn parse_error(line: usize, col: usize, message: impl Into<String>) -> Self {
        todo!("Create ParseError variant")
    }

    /// Create missing field error with optional suggestion
    /// Role: Report missing required field
    pub fn missing_field(
        section: impl Into<String>,
        field: impl Into<String>,
        suggestion: Option<String>,
    ) -> Self {
        todo!("Create MissingField variant")
    }

    /// Create type mismatch error
    /// Role: Report type validation failure
    pub fn invalid_type(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        location: Location,
    ) -> Self {
        todo!("Create InvalidType variant")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(format!("{}", error).contains("database"));
        assert!(format!("{}", error).contains("port"));
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
}
```

---

### Milestone 2: Parse Configuration with Error Context

**Goal**: Parse TOML/JSON files and preserve location information for error reporting.

**Why the previous milestone is not enough**: Having error types is great, but we need to actually parse files and capture where errors occur. Without location tracking, users see "parse failed" instead of "parse failed at line 15, column 8".

**What's the improvement**: Preserving location information transforms generic error messages into actionable feedback. Users can immediately jump to the problematic line in their editor. This reduces debugging time from minutes to seconds.

**Architecture**:
- Structs: `ConfigParser`
- Functions:
  - `parse_json(content: &str) -> Result<Value, ConfigError>` - Parse JSON with locations
  - `parse_toml(content: &str) -> Result<Value, ConfigError>` - Parse TOML with locations
  - Conversion from parser errors to ConfigError

---

**Starter Code**:

```rust
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Configuration file parser with error context
///
/// Structs:
/// - ConfigParser: Main parser interface
///
/// Functions:
/// - parse_json() - Parse JSON with location tracking
/// - parse_toml() - Parse TOML with location tracking
/// - parse_file() - Detect format and parse
pub struct ConfigParser;

impl ConfigParser {
    /// Parse JSON configuration
    /// Role: Parse JSON and convert errors to ConfigError
    pub fn parse_json(content: &str) -> Result<Value, ConfigError> {
        todo!("Parse JSON, preserve location on error")
    }

    /// Parse TOML configuration
    /// Role: Parse TOML and convert errors to ConfigError
    pub fn parse_toml(content: &str) -> Result<Value, ConfigError> {
        todo!("Parse TOML, preserve location on error")
    }

    /// Parse configuration file (auto-detect format)
    /// Role: Read file and parse based on extension
    pub fn parse_file(path: &Path) -> Result<Value, ConfigError> {
        todo!("Read file, detect format, parse")
    }
}

/// Convert serde_json errors to ConfigError
impl From<serde_json::Error> for ConfigError {
    /// Role: Preserve line/column information from JSON parser
    fn from(err: serde_json::Error) -> Self {
      // TODO: Extract line/column from serde_json error
    }
}

/// Convert toml errors to ConfigError
impl From<toml::de::Error> for ConfigError {
    /// Role: Preserve location information from TOML parser
    fn from(err: toml::de::Error) -> Self {
        todo!("Extract line/column from TOML error")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_json() {
        let json = r#"{"database": {"host": "localhost", "port": 5432}}"#;
        let result = ConfigParser::parse_json(json);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config["database"]["host"], "localhost");
        assert_eq!(config["database"]["port"], 5432);
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"database": {"host": "localhost", "port": 5432}"#; // Missing closing brace
        let result = ConfigParser::parse_json(json);

        assert!(result.is_err());
        let error = result.unwrap_err();

        match error {
            ConfigError::ParseError { line, col, message } => {
                assert!(line > 0);
                assert!(message.contains("EOF") || message.contains("brace"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_parse_valid_toml() {
        let toml = r#"
[database]
host = "localhost"
port = 5432
"#;
        let result = ConfigParser::parse_toml(toml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let toml = r#"
[database]
host = "localhost
port = 5432
"#; // Unterminated string
        let result = ConfigParser::parse_toml(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_file_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"key": "value"}}"#).unwrap();

        let result = ConfigParser::parse_file(file.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_file_not_found() {
        let result = ConfigParser::parse_file(Path::new("/nonexistent/config.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_error_conversion_preserves_location() {
        let json = "{\n  \"key\": invalid\n}";
        let result = ConfigParser::parse_json(json);

        match result {
            Err(ConfigError::ParseError { line, .. }) => {
                assert_eq!(line, 2); // Error on second line
            }
            _ => panic!("Expected ParseError with line 2"),
        }
    }
}
```

---

### Milestone 3: Collect Multiple Errors (Don't Fail Fast)

**Goal**: Validate entire configuration and collect ALL errors, not just the first one.

**Why the previous milestone is not enough**: Failing on the first error creates a frustrating cycle: fix one error, run again, find next error, repeat. For a config with 10 errors, this means 10 iterations.

**What's the improvement**: Collecting errors enables a "fix all at once" workflow. Instead of 10 validation cycles, users get all errors in one run. This is 10x faster feedback for complex configurations. This is the difference between "annoying" and "delightful" developer experience.

**Architecture**:
- Structs: `ValidationErrors`, `Validator`
- Functions:
  - `ValidationErrors::new()` - Create error collector
  - `add(&mut self, error: ConfigError)` - Add error to collection
  - `into_result<T>(self, value: T) -> Result<T, Vec<ConfigError>>` - Convert to result
  - `validate_all(config: &Value) -> Result<(), Vec<ConfigError>>` - Validate and collect

---

**Starter Code**:

```rust
/// Error collector for validation
///
/// Structs:
/// - ValidationErrors: Accumulates errors during validation
///
/// Fields:
/// - errors: Vec<ConfigError> - Collected errors
///
/// Functions:
/// - new() - Create empty collector
/// - add() - Add error to collection
/// - has_errors() - Check if any errors collected
/// - into_result() - Convert to Result
#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<ConfigError>,
}

impl ValidationErrors {
    /// Create new error collector
    /// Role: Initialize empty error list
    pub fn new() -> Self {
        todo!("Initialize empty Vec")
    }

    /// Add error to collection
    /// Role: Accumulate validation errors
    pub fn add(&mut self, error: ConfigError) {
        todo!("Push error to Vec")
    }

    /// Check if any errors were collected
    /// Role: Test for error presence
    pub fn has_errors(&self) -> bool {
        todo!("Check if Vec is not empty")
    }

    /// Convert to Result, returning all errors or success value
    /// Role: Transform collector into Result
    pub fn into_result<T>(self, value: T) -> Result<T, Vec<ConfigError>> {
        todo!("Return Err(errors) if has_errors, else Ok(value)")
    }

    /// Get number of errors
    /// Role: Query error count
    pub fn count(&self) -> usize {
        // TODO: return count
    }
}

/// Configuration validator
///
/// Structs:
/// - Validator: Validation orchestrator
///
/// Functions:
/// - validate_config() - Validate entire config
/// - validate_section() - Validate config section
pub struct Validator;

impl Validator {
    /// Validate entire configuration, collecting all errors
    /// Role: Orchestrate validation and collect errors
    pub fn validate_config(config: &Value) -> Result<(), Vec<ConfigError>> {
        let mut errors = ValidationErrors::new();

        // Validate database section
        if let Err(e) = Self::validate_database_section(config) {
            errors.add(e);
        }

        // Validate server section
        if let Err(e) = Self::validate_server_section(config) {
            errors.add(e);
        }

        // Continue validating other sections...

        errors.into_result(())
    }

    /// Validate database section
    /// Role: Check database-specific requirements
    fn validate_database_section(config: &Value) -> Result<(), ConfigError> {
        todo!("Validate database fields")
    }

    /// Validate server section
    /// Role: Check server-specific requirements
    fn validate_server_section(config: &Value) -> Result<(), ConfigError> {
        todo!("Validate server fields")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_errors_accumulation() {
        let mut errors = ValidationErrors::new();
        assert!(!errors.has_errors());
        assert_eq!(errors.count(), 0);

        errors.add(ConfigError::MissingField {
            section: "db".to_string(),
            field: "host".to_string(),
            suggestion: None,
        });

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
    fn test_validation_errors_into_result_success() {
        let errors = ValidationErrors::new();
        let result = errors.into_result(42);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_validation_errors_into_result_failure() {
        let mut errors = ValidationErrors::new();
        errors.add(ConfigError::MissingField {
            section: "db".to_string(),
            field: "host".to_string(),
            suggestion: None,
        });

        let result = errors.into_result(());

        assert!(result.is_err());
        let err_vec = result.unwrap_err();
        assert_eq!(err_vec.len(), 1);
    }

    #[test]
    fn test_validate_config_with_multiple_errors() {
        let config = json!({
            "database": {
                "port": "invalid",  // Should be number
                "max_connections": 1000  // Out of range
            },
            "server": {
                "timeout": -5  // Should be positive
            }
        });

        let result = Validator::validate_config(&config);

        assert!(result.is_err());
        let errors = result.unwrap_err();

        // Should collect multiple errors
        assert!(errors.len() >= 2);
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

        let result = Validator::validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_validation_after_error() {
        // Ensure validation doesn't stop at first error
        let config = json!({
            "database": {
                // Missing host
                "port": "bad",  // Invalid type
                "max_connections": -1  // Invalid value
            }
        });

        let result = Validator::validate_config(&config);
        let errors = result.unwrap_err();

        // Should find all 3 errors
        assert_eq!(errors.len(), 3);
    }
}
```

---

### Milestone 4: Add Suggestions for Common Mistakes

**Goal**: Enhance error messages with actionable suggestions using string similarity.

**Why the previous milestone is not enough**: Reporting errors is good, but users still need to figure out how to fix them. "Field 'timout' not found" requires the user to scan documentation or code to find the correct spelling.

**What's the improvement**: Suggestions reduce cognitive load dramatically. Instead of users searching documentation, the validator tells them exactly what to do: "Did you mean 'timeout'?". This is especially valuable for large configuration schemas with dozens of fields.

**Optimization focus**: Developer experience through intelligent error messages.

**Architecture**:
- Functions:
  - `find_similar_field(typo: &str, valid: &[&str]) -> Option<String>` - Find similar field names
  - `levenshtein_distance(a: &str, b: &str) -> usize` - Compute edit distance
  - Enhanced validation with suggestions

---

**Starter Code**:

```rust
/// String similarity utilities
///
/// Functions:
/// - levenshtein_distance() - Compute edit distance between strings
/// - find_similar_field() - Find most similar valid field name
/// - find_similar_value() - Suggest similar valid values

/// Compute Levenshtein distance between two strings
/// Role: Calculate minimum edits needed to transform a into b
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    todo!("Implement Levenshtein distance algorithm")
}

/// Find field name most similar to typo
/// Role: Suggest corrections for typos
///
/// Returns: Best match if within edit distance threshold
pub fn find_similar_field(typo: &str, valid_fields: &[&str]) -> Option<String> {
    let mut best_match = None;
    let mut best_distance = usize::MAX;
    const MAX_DISTANCE: usize = 2; // Maximum 2 edits

    for &field in valid_fields {
        let distance = levenshtein_distance(typo, field);

        if distance < best_distance && distance <= MAX_DISTANCE {
            best_distance = distance;
            best_match = Some(field.to_string());
        }
    }

    best_match
}

/// Validate field exists in section
/// Role: Check field presence and suggest similar names
pub fn validate_field_exists(
    config: &Value,
    section: &str,
    field: &str,
    valid_fields: &[&str],
) -> Result<(), ConfigError> {
    let section_obj = config.get(section)
        .and_then(|v| v.as_object())
        .ok_or_else(|| ConfigError::MissingField {
            section: section.to_string(),
            field: section.to_string(),
            suggestion: None,
        })?;

    if section_obj.contains_key(field) {
        Ok(())
    } else {
        let suggestion = find_similar_field(field, valid_fields);
        Err(ConfigError::MissingField {
            section: section.to_string(),
            field: field.to_string(),
            suggestion,
        })
    }
}

/// Enhanced validator with suggestions
impl Validator {
    /// Validate database section with suggestions
    /// Role: Provide helpful error messages with corrections
    pub fn validate_database_with_suggestions(config: &Value) -> Result<(), ConfigError> {
        const VALID_FIELDS: &[&str] = &["host", "port", "username", "password", "max_connections"];

        // Check for required fields with suggestions
        validate_field_exists(config, "database", "host", VALID_FIELDS)?;
        validate_field_exists(config, "database", "port", VALID_FIELDS)?;

        // Additional validation...

        Ok(())
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "abd"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("sitting", "kitten"), 3);
    }

    #[test]
    fn test_find_similar_field_exact_match() {
        let valid = &["timeout", "retries", "max_connections"];
        let result = find_similar_field("timeout", valid);
        assert_eq!(result, Some("timeout".to_string()));
    }

    #[test]
    fn test_find_similar_field_typo() {
        let valid = &["timeout", "retries", "max_connections"];

        // One character off
        assert_eq!(find_similar_field("timout", valid), Some("timeout".to_string()));
        assert_eq!(find_similar_field("retrys", valid), Some("retries".to_string()));

        // Two characters off
        assert_eq!(find_similar_field("timeot", valid), Some("timeout".to_string()));
    }

    #[test]
    fn test_find_similar_field_no_match() {
        let valid = &["timeout", "retries"];

        // Too different (>2 edits)
        assert_eq!(find_similar_field("completely_different", valid), None);
    }

    #[test]
    fn test_validate_field_exists_success() {
        let config = json!({
            "database": {
                "host": "localhost",
                "port": 5432
            }
        });

        let result = validate_field_exists(
            &config,
            "database",
            "host",
            &["host", "port", "username"]
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_field_exists_with_suggestion() {
        let config = json!({
            "database": {
                "hst": "localhost"  // Typo: should be "host"
            }
        });

        let result = validate_field_exists(
            &config,
            "database",
            "host",
            &["host", "port", "username"]
        );

        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::MissingField { field, suggestion, .. } => {
                assert_eq!(field, "host");
                // Should suggest something (likely "hst" is too different, but port might match)
            }
            _ => panic!("Expected MissingField error"),
        }
    }

    #[test]
    fn test_error_message_includes_suggestion() {
        let error = ConfigError::MissingField {
            section: "server".to_string(),
            field: "timout".to_string(),
            suggestion: Some("timeout".to_string()),
        };

        let display = format!("{}", error);
        assert!(display.contains("timout"));
        // Note: thiserror doesn't automatically include suggestion in display
        // We'll add custom formatting in next milestone
    }

    #[test]
    fn test_multiple_typos_get_suggestions() {
        let config = json!({
            "database": {
                "hst": "localhost",   // typo
                "prt": 5432,          // typo
                "usrname": "admin"    // typo
            }
        });

        let mut errors = ValidationErrors::new();

        // Validate all fields
        if let Err(e) = validate_field_exists(&config, "database", "host", &["host", "port", "username"]) {
            errors.add(e);
        }
        if let Err(e) = validate_field_exists(&config, "database", "port", &["host", "port", "username"]) {
            errors.add(e);
        }
        if let Err(e) = validate_field_exists(&config, "database", "username", &["host", "port", "username"]) {
            errors.add(e);
        }

        // Should collect all typos
        assert_eq!(errors.count(), 3);
    }
}
```

---

### Milestone 5: Type-Safe Schema Validation with Builder Pattern

**Goal**: Create a fluent API for defining validation schemas that can be reused and composed.

**Why the previous milestone is not enough**: Hardcoding validation logic for each field is brittle and verbose. Every new field requires code changes. A schema-driven approach separates "what to validate" from "how to validate".

**What's the improvement**: Schema-based validation is declarative and maintainable. Adding a new field becomes configuration, not code. Schemas can be serialized, versioned, shared across teams, and even loaded from external files. This is essential for systems where non-programmers define validation rules.

**Optimization focus**: Maintainability and extensibility through declarative schemas.

**Architecture**:
- Structs: `Schema`, `FieldSchema`, `SchemaBuilder`, `FieldValidator`
- Traits: `Validator` trait for custom validators
- Functions:
  - `SchemaBuilder::new()` - Create schema builder
  - `required_field()`, `optional_field()` - Define fields
  - `with_type()`, `with_range()`, `with_pattern()` - Add constraints
  - `build()` - Construct schema
  - `Schema::validate()` - Validate config against schema

---

**Starter Code**:

```rust
use std::collections::HashMap;
use regex::Regex;

/// Schema for configuration validation
///
/// Structs:
/// - Schema: Complete validation schema
/// - FieldSchema: Single field validation rules
/// - SchemaBuilder: Fluent builder for schemas
///
/// Functions:
/// - Schema::validate() - Validate config against schema
/// - SchemaBuilder methods for defining fields and constraints

/// Field validator function type
type ValidatorFn = Box<dyn Fn(&Value) -> Result<(), String> + Send + Sync>;

/// Schema for a single field
#[derive(Default)]
pub struct FieldSchema {
    path: String,                            // Field path (e.g., "database.port")      
    required: bool,                          // Whether field is mandatory            
    validators: Vec<ValidatorFn>,            // Validation functions    
}

impl FieldSchema {
    /// Create new field schema
    /// Role: Initialize field definition
    pub fn new(path: impl Into<String>, required: bool) -> Self {
        todo!("Initialize FieldSchema")
    }

    /// Add validator function
    /// Role: Attach validation rule
    pub fn add_validator(&mut self, validator: ValidatorFn) {    
      todo!("add role")
    }
  

    /// Validate value against all rules
    /// Role: Apply all validators to value
    pub fn validate(&self, value: &Value) -> Result<(), ConfigError> {
        todo!("Run all validators, collect errors")
    }
}

/// Complete validation schema
///
/// Struct:
/// - Schema: Collection of field validations
///
/// Fields:
/// - fields: HashMap<String, FieldSchema> - Field validators by path
#[derive(Default)]
pub struct Schema {
    fields: HashMap<String, FieldSchema>,
}

impl Schema {
    /// Create empty schema
    /// Role: Initialize schema
    pub fn new() -> Self {
      todo!("new Schema")
    }

    /// Add field to schema
    /// Role: Register field validation
    pub fn add_field(&mut self, field: FieldSchema) {
        todo!("insert")
    }

    /// Validate configuration against schema
    /// Role: Check all fields and collect errors
    pub fn validate(&self, config: &Value) -> Result<(), Vec<ConfigError>> {
        todo!("Validate all fields, collect errors")
    }
}

/// Fluent builder for schemas
///
/// Struct:
/// - SchemaBuilder: Fluent API for building schemas
///
/// Methods:
/// - required_field() - Add required field
/// - optional_field() - Add optional field
/// - build() - Construct final schema
pub struct SchemaBuilder {
    fields: Vec<FieldSchema>,
}

impl SchemaBuilder {
    /// Create new builder
    /// Role: Initialize empty builder
    pub fn new() -> Self {
        todo!("Initialize empty builder")
    }

    /// Add required field
    /// Role: Define mandatory field with validator
    pub fn required_field(
        mut self,
        path: &str,
        validator: impl Fn(&Value) -> Result<(), String> + 'static + Send + Sync,
    ) -> Self {
        todo!("add required field to schema")        
    }

    /// Add optional field
    /// Role: Define optional field with validator
    pub fn optional_field(
        mut self,
        path: &str,
        validator: impl Fn(&Value) -> Result<(), String> + 'static + Send + Sync,
    ) -> Self {
        todo!("add optional field to schema")          
    }

    /// Build final schema
    /// Role: Construct immutable schema
    pub fn build(self) -> Schema {
         todo!("add fields to schema")
    }
}

/// Common validators
///
/// Functions for common validation patterns
pub mod validators {
    use super::*;

    /// Validate value is string
    pub fn is_string() -> impl Fn(&Value) -> Result<(), String> {
       todo!("type check for string")
    }

    /// Validate value is integer
    pub fn is_integer() -> impl Fn(&Value) -> Result<(), String> {
       todo!("type check for integer")  
    }

    /// Validate integer is in range
    /// Role: Range constraint for numbers
    pub fn in_range(min: i64, max: i64) -> impl Fn(&Value) -> Result<(), String> {
        todo!("Range constraint for numbers")
    }

    /// Validate string matches regex
    pub fn matches_pattern(pattern: &str) -> impl Fn(&Value) -> Result<(), String> {
        todo!("Pattern matching for strings")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use super::validators::*;

    #[test]
    fn test_field_schema_validation_success() {
        let mut field = FieldSchema::new("port", true);
        field.add_validator(Box::new(is_integer()));
        field.add_validator(Box::new(in_range(1, 65535)));

        let value = json!(8080);
        assert!(field.validate(&value).is_ok());
    }

    #[test]
    fn test_field_schema_validation_failure() {
        let mut field = FieldSchema::new("port", true);
        field.add_validator(Box::new(is_integer()));
        field.add_validator(Box::new(in_range(1, 65535)));

        let value = json!(70000); // Out of range
        assert!(field.validate(&value).is_err());
    }

    #[test]
    fn test_schema_builder_fluent_api() {
        let schema = SchemaBuilder::new()
            .required_field("database.host", is_string())
            .required_field("database.port", |v| {
                let port = v.as_i64().ok_or("must be integer")?;
                if port < 1 || port > 65535 {
                    return Err("port out of range".to_string());
                }
                Ok(())
            })
            .optional_field("database.timeout", is_integer())
            .build();

        // Schema should have 3 fields
        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_schema_validates_required_fields() {
        let schema = SchemaBuilder::new()
            .required_field("host", is_string())
            .required_field("port", is_integer())
            .build();

        // Missing required fields
        let config = json!({});
        let result = schema.validate(&config);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // At least 2 missing fields
    }

    #[test]
    fn test_schema_validates_types() {
        let schema = SchemaBuilder::new()
            .required_field("database.port", is_integer())
            .build();

        // Wrong type
        let config = json!({
            "database": {
                "port": "not_a_number"
            }
        });

        let result = schema.validate(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_schema_validates_ranges() {
        let schema = SchemaBuilder::new()
            .required_field("database.port", in_range(1, 65535))
            .build();

        // Out of range
        let config = json!({
            "database": {
                "port": 70000
            }
        });

        assert!(schema.validate(&config).is_err());

        // In range
        let config = json!({
            "database": {
                "port": 8080
            }
        });

        assert!(schema.validate(&config).is_ok());
    }

    #[test]
    fn test_optional_fields_can_be_missing() {
        let schema = SchemaBuilder::new()
            .required_field("host", is_string())
            .optional_field("timeout", is_integer())
            .build();

        // Missing optional field should be OK
        let config = json!({
            "host": "localhost"
        });

        assert!(schema.validate(&config).is_ok());
    }

    #[test]
    fn test_pattern_validator() {
        let schema = SchemaBuilder::new()
            .required_field("email", matches_pattern(r"^[^@]+@[^@]+\.[^@]+$"))
            .build();

        let valid = json!({"email": "user@example.com"});
        assert!(schema.validate(&valid).is_ok());

        let invalid = json!({"email": "not-an-email"});
        assert!(schema.validate(&invalid).is_err());
    }

    #[test]
    fn test_multiple_validators_on_field() {
        let schema = SchemaBuilder::new()
            .required_field("password", |v| {
                let s = v.as_str().ok_or("must be string")?;

                if s.len() < 8 {
                    return Err("must be at least 8 characters".to_string());
                }
                if !s.chars().any(|c| c.is_numeric()) {
                    return Err("must contain at least one number".to_string());
                }

                Ok(())
            })
            .build();

        assert!(schema.validate(&json!({"password": "pass123456"})).is_ok());
        assert!(schema.validate(&json!({"password": "short"})).is_err());
        assert!(schema.validate(&json!({"password": "nonumbers"})).is_err());
    }
}
```

---

### Milestone 6: Formatted Error Output with Color and Context

**Goal**: Pretty-print validation errors with colors, file context snippets, and helpful formatting.

**Why the previous milestone is not enough**: A list of error structs is machine-readable but not user-friendly. Developers need visual, scannable output that guides them to fixes quickly.

**What's the improvement**: Visual formatting with colors and context makes errors instantly understandable. Instead of scanning through text, errors jump out visually. Context snippets show exactly where the problem is. This transforms error messages from "technical output" to "helpful guidance".

**Optimization focus**: Developer experience through visual presentation.

**Architecture**:
- Structs: `ErrorFormatter`, `FormattedOutput`
- Functions:
  - `format_errors(errors: &[ConfigError], source: &str) -> String` - Pretty-print errors
  - `format_single_error()` - Format one error with context
  - `extract_context_lines()` - Get lines around error location
  - `colorize_output()` - Add ANSI color codes

---

**Starter Code**:

```rust
use colored::*;

/// Error formatter with colors and context
///
/// Structs:
/// - ErrorFormatter: Pretty-prints validation errors
///
/// Functions:
/// - format_errors() - Format all errors with colors
/// - format_single_error() - Format one error with context
/// - extract_context() - Get source lines around error

pub struct ErrorFormatter;

impl ErrorFormatter {
    /// Format all validation errors
    pub fn format_errors(errors: &[ConfigError], file_content: &str) -> String {
       todo!("Create user-friendly error output")
    }

    /// Format a single error with context
    fn format_single_error(error: &ConfigError, number: usize, lines: &[&str]) -> String {
        let mut output = String::new();

        output.push_str(&format!("{}. ", number));

        match error {
           todo!("Show errors with source lines and annotations");
        }

        output
    }

    /// Format context lines with annotation
    /// Role: Show source code with error pointer
    fn format_context(line: usize, col: usize, lines: &[&str]) -> String {
        let mut output = String::new();

        // Show line before (if exists)
        if line > 1 {
            output.push_str(&format!(
                "   {} | {}\n",
                format!("{:3}", line - 1).blue(),
                lines[line - 2]
            ));
        }

        // Show error line
        todo!();

        // Show pointer to error location
        todo!();

        // Show line after (if exists)
        todo!();

        output
    }

    /// Format summary footer
    fn format_summary(errors: &[ConfigError]) -> String {
        todo!("Show total error count and next steps");
    }

    /// Check if colors should be disabled
    pub fn should_use_colors() -> bool {
        todo!("Respect NO_COLOR environment variable")
    }
}

/// Format validation result for display
pub fn format_validation_result(result: Result<(), Vec<ConfigError>>, source: &str) -> String {
    match result {
      todo!("Convert Result to formatted string");
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_parse_error() {
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
        assert!(formatted.contains("invalid"));
    }

    #[test]
    fn test_format_missing_field_with_suggestion() {
        let error = ConfigError::MissingField {
            section: "database".to_string(),
            field: "hst".to_string(),
            suggestion: Some("host".to_string()),
        };

        let formatted = ErrorFormatter::format_single_error(&error, 1, &[]);

        assert!(formatted.contains("Missing required field"));
        assert!(formatted.contains("database"));
        assert!(formatted.contains("hst"));
        assert!(formatted.contains("Did you mean 'host'?"));
    }

    #[test]
    fn test_format_multiple_errors() {
        let source = r#"
{
  "database": {
    "port": "invalid",
    "max_connections": 1000
  }
}
"#;

        let errors = vec![
            ConfigError::InvalidType {
                field: "database.port".to_string(),
                expected: "integer".to_string(),
                actual: "string".to_string(),
                location: Location { line: 3, column: 12 },
            },
            ConfigError::OutOfRange {
                field: "database.max_connections".to_string(),
                value: 1000,
                min: 1,
                max: 500,
            },
        ];

        let formatted = ErrorFormatter::format_errors(&errors, source);

        assert!(formatted.contains("Configuration Validation Errors"));
        assert!(formatted.contains("1."));
        assert!(formatted.contains("2."));
        assert!(formatted.contains("2 errors found"));
    }

    #[test]
    fn test_format_validation_result_success() {
        let result: Result<(), Vec<ConfigError>> = Ok(());
        let formatted = format_validation_result(result, "");

        assert!(formatted.contains("valid"));
    }

    #[test]
    fn test_format_validation_result_errors() {
        let errors = vec![
            ConfigError::MissingField {
                section: "db".to_string(),
                field: "host".to_string(),
                suggestion: None,
            },
        ];

        let result: Result<(), Vec<ConfigError>> = Err(errors);
        let formatted = format_validation_result(result, "");

        assert!(formatted.contains("error"));
        assert!(formatted.contains("Missing required field"));
    }

    #[test]
    fn test_context_formatting() {
        let source = "line1\nline2\nline3\nline4\nline5";
        let lines: Vec<&str> = source.lines().collect();

        let context = ErrorFormatter::format_context(3, 2, &lines);

        // Should show line 2, 3 (error), and 4
        assert!(context.contains("line2"));
        assert!(context.contains("line3"));
        assert!(context.contains("line4"));
        assert!(context.contains("^^^")); // Pointer
    }

    #[test]
    fn test_no_color_environment() {
        std::env::set_var("NO_COLOR", "1");
        assert!(!ErrorFormatter::should_use_colors());

        std::env::remove_var("NO_COLOR");
        assert!(ErrorFormatter::should_use_colors());
    }

    #[test]
    fn test_summary_plural() {
        let one_error = vec![ConfigError::MissingField {
            section: "s".to_string(),
            field: "f".to_string(),
            suggestion: None,
        }];

        let summary = ErrorFormatter::format_summary(&one_error);
        assert!(summary.contains("1 error"));
        assert!(!summary.contains("errors"));

        let two_errors = vec![one_error[0].clone(), one_error[0].clone()];
        let summary = ErrorFormatter::format_summary(&two_errors);
        assert!(summary.contains("2 errors"));
    }
}
```

---


### Complete Working Example

Here's a complete, production-ready configuration validator:

```rust
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
```

This complete example demonstrates:
- **Part 1**: Rich error types with thiserror
- **Part 2**: Parsing with location preservation
- **Part 3**: Error collection without fail-fast
- **Part 4**: Typo suggestions with Levenshtein distance
- **Part 5**: Schema-based validation with builder pattern
- **Part 6**: Beautiful error formatting with colors
- **Examples**: Real-world validation workflows
- **Tests**: Comprehensive validation of all components

The implementation shows how proper error handling transforms configuration validation from a frustrating chore into a helpful, guided experience.
