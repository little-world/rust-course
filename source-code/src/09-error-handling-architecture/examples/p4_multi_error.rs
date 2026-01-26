//! Pattern 4: Custom Error Types with Context
//! Example: Error Aggregation (MultiError)
//!
//! Run with: cargo run --example p4_multi_error

use std::fmt;
use thiserror::Error;

/// Aggregates multiple errors for batch validation.
#[derive(Error, Debug)]
#[error("Multiple errors occurred ({} total)", errors.len())]
pub struct MultiError {
    errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

impl MultiError {
    pub fn new() -> Self {
        MultiError { errors: Vec::new() }
    }

    pub fn add(&mut self, error: impl std::error::Error + Send + Sync + 'static) {
        self.errors.push(Box::new(error));
    }

    pub fn add_boxed(&mut self, error: Box<dyn std::error::Error + Send + Sync>) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Convert to Result - Ok if no errors, Err otherwise.
    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }

    /// Display all errors with numbering.
    pub fn display_all(&self) -> String {
        let mut lines = vec![format!("{} error(s) occurred:", self.errors.len())];
        for (i, err) in self.errors.iter().enumerate() {
            lines.push(format!("  {}. {}", i + 1, err));
        }
        lines.join("\n")
    }

    /// Iterate over all errors.
    pub fn iter(&self) -> impl Iterator<Item = &(dyn std::error::Error + Send + Sync)> {
        self.errors.iter().map(|e| e.as_ref())
    }
}

impl Default for MultiError {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple validation error for demonstration.
#[derive(Error, Debug)]
#[error("{field}: {message}")]
struct ValidationError {
    field: String,
    message: String,
}

impl ValidationError {
    fn new(field: &str, message: &str) -> Self {
        ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Validate a user record, collecting all errors.
fn validate_user(name: &str, email: &str, age: i32) -> Result<(), MultiError> {
    let mut errors = MultiError::new();

    if name.is_empty() {
        errors.add(ValidationError::new("name", "cannot be empty"));
    } else if name.len() < 2 {
        errors.add(ValidationError::new("name", "must be at least 2 characters"));
    }

    if email.is_empty() {
        errors.add(ValidationError::new("email", "cannot be empty"));
    } else if !email.contains('@') {
        errors.add(ValidationError::new("email", "must contain @"));
    }

    if age < 0 {
        errors.add(ValidationError::new("age", "cannot be negative"));
    } else if age > 150 {
        errors.add(ValidationError::new("age", "must be realistic (< 150)"));
    }

    errors.into_result(())
}

/// Batch validate multiple records.
fn validate_batch(records: Vec<(&str, &str, i32)>) -> (usize, usize, MultiError) {
    let mut success_count = 0;
    let mut all_errors = MultiError::new();

    for (i, (name, email, age)) in records.iter().enumerate() {
        match validate_user(name, email, *age) {
            Ok(_) => success_count += 1,
            Err(e) => {
                for err in e.errors {
                    all_errors.add_boxed(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Record {}: {}", i + 1, err),
                    )));
                }
            }
        }
    }

    (success_count, records.len() - success_count, all_errors)
}

fn main() {
    println!("=== Error Aggregation (MultiError) ===\n");

    // Valid user
    println!("=== Valid User ===");
    match validate_user("Alice", "alice@example.com", 30) {
        Ok(_) => println!("  Validation passed!\n"),
        Err(e) => println!("{}\n", e.display_all()),
    }

    // Single error
    println!("=== Single Error ===");
    match validate_user("Alice", "invalid-email", 30) {
        Ok(_) => println!("  Validation passed!"),
        Err(e) => println!("{}\n", e.display_all()),
    }

    // Multiple errors
    println!("=== Multiple Errors ===");
    match validate_user("", "bad", -5) {
        Ok(_) => println!("  Validation passed!"),
        Err(e) => println!("{}\n", e.display_all()),
    }

    // Batch validation
    println!("=== Batch Validation ===");
    let records = vec![
        ("Alice", "alice@example.com", 30),  // valid
        ("", "bob@example.com", 25),          // invalid name
        ("Carol", "invalid", 200),            // invalid email and age
        ("Dave", "dave@example.com", 40),     // valid
    ];

    let (success, failure, errors) = validate_batch(records);
    println!("  Results: {} valid, {} invalid", success, failure);
    if !errors.is_empty() {
        println!("\n{}", errors.display_all());
    }

    println!("\n=== Use Cases ===");
    println!("1. Form validation - show all field errors at once");
    println!("2. Data import - report all invalid records");
    println!("3. Config validation - check all settings");
    println!("4. Batch API requests - aggregate failures");

    println!("\n=== Key Points ===");
    println!("1. into_result() converts empty MultiError to Ok");
    println!("2. Collect all errors before returning");
    println!("3. Better UX than failing on first error");
    println!("4. display_all() formats numbered list");
}
