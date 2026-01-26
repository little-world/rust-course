//! Pattern 4: Custom Error Types with Context
//! Example: Error with Location Tracking
//!
//! Run with: cargo run --example p4_location_tracking

use thiserror::Error;

/// Parse error with precise location information.
#[derive(Error, Debug)]
#[error("Parse error at line {line}, column {column}: {message}")]
pub struct ParseErrorWithLocation {
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub snippet: Option<String>,
}

impl ParseErrorWithLocation {
    pub fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        ParseErrorWithLocation {
            line,
            column,
            message: message.into(),
            snippet: None,
        }
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Format error with visual pointer to the error location.
    pub fn display_with_pointer(&self) -> String {
        let mut output = format!(
            "error: {}\n  --> line {}:{}\n",
            self.message, self.line, self.column
        );

        if let Some(snippet) = &self.snippet {
            output.push_str(&format!("   |\n{:3} | {}\n   | ", self.line, snippet));
            output.push_str(&" ".repeat(self.column.saturating_sub(1)));
            output.push_str("^ here\n");
        }

        output
    }
}

/// Simple tokenizer that tracks position.
fn tokenize(input: &str) -> Result<Vec<String>, ParseErrorWithLocation> {
    let mut tokens = Vec::new();
    let mut line = 1;
    let mut column = 1;

    for ch in input.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                tokens.push(ch.to_string());
            }
            ' ' | '\t' => {
                // Skip whitespace
            }
            '\n' => {
                line += 1;
                column = 0; // Will be incremented below
            }
            '@' | '#' | '$' => {
                let snippet = input.lines().nth(line - 1).unwrap_or("").to_string();
                return Err(
                    ParseErrorWithLocation::new(line, column, format!("unexpected character '{}'", ch))
                        .with_snippet(snippet),
                );
            }
            _ => {
                // Allow other characters
            }
        }
        column += 1;
    }

    Ok(tokens)
}

fn main() {
    println!("=== Error with Location Tracking ===\n");

    // Valid input
    let valid = "hello world 123";
    println!("Tokenizing: '{}'", valid);
    match tokenize(valid) {
        Ok(tokens) => println!("  Tokens: {:?}\n", tokens),
        Err(e) => println!("{}", e.display_with_pointer()),
    }

    // Invalid input with error location
    let invalid = "hello @world";
    println!("Tokenizing: '{}'", invalid);
    match tokenize(invalid) {
        Ok(tokens) => println!("  Tokens: {:?}", tokens),
        Err(e) => println!("{}", e.display_with_pointer()),
    }

    // Multiline with error
    let multiline = "line one\nline #two\nline three";
    println!("Tokenizing multiline:");
    println!("  '{}'", multiline.replace('\n', "\\n"));
    match tokenize(multiline) {
        Ok(tokens) => println!("  Tokens: {:?}", tokens),
        Err(e) => println!("{}", e.display_with_pointer()),
    }

    // Direct error construction
    println!("\n=== Error Display Formats ===\n");
    let err = ParseErrorWithLocation::new(10, 15, "expected '}', found EOF")
        .with_snippet("fn main() { let x = 1;");

    println!("Standard Display:");
    println!("  {}\n", err);

    println!("With Visual Pointer:");
    println!("{}", err.display_with_pointer());

    println!("=== Key Points ===");
    println!("1. Track line and column during parsing");
    println!("2. Include snippet of offending source");
    println!("3. Visual pointer shows exact error location");
    println!("4. Similar to rustc and other compiler errors");
}
