//! # Error Handling Architecture
//!
//! This crate contains runnable examples demonstrating error handling patterns in Rust.
//!
//! ## Patterns Covered
//!
//! 1. **Custom Error Enums** - Type-safe errors for libraries
//! 2. **Structured Errors** - Rich error types with context
//! 3. **Error Propagation** - Using `?`, `map_err`, and `context`
//! 4. **Errors with Context** - Location tracking, suggestions, aggregation
//! 5. **Recoverable vs Unrecoverable** - When to use Result vs panic
//! 6. **Async Error Handling** - Timeouts, retries, circuit breakers
//! 7. **Anti-patterns** - Common mistakes and how to avoid them
//!
//! ## Running Examples
//!
//! ```bash
//! # Pattern 1: Custom Error Enums
//! cargo run --example p1_parse_error
//! cargo run --example p1_non_exhaustive
//!
//! # Pattern 2: Structured Errors
//! cargo run --example p2_http_error
//! cargo run --example p2_error_builder
//!
//! # Pattern 3: Error Propagation
//! cargo run --example p3_basic_propagation
//! cargo run --example p3_error_conversion
//! cargo run --example p3_anyhow_context
//! cargo run --example p3_retry_logic
//!
//! # Pattern 4: Errors with Context
//! cargo run --example p4_location_tracking
//! cargo run --example p4_config_suggestions
//! cargo run --example p4_multi_error
//!
//! # Pattern 5: Recoverable vs Unrecoverable
//! cargo run --example p5_panic_vs_result
//! cargo run --example p5_graceful_degradation
//! cargo run --example p5_catch_unwind
//!
//! # Pattern 6: Async Error Handling
//! cargo run --example p6_async_propagation
//! cargo run --example p6_timeout
//! cargo run --example p6_circuit_breaker
//!
//! # Pattern 7: Anti-patterns
//! cargo run --example p7_anti_patterns
//! ```
//!
//! ## Key Dependencies
//!
//! - `thiserror` - Derive macro for custom error types
//! - `anyhow` - Flexible error handling for applications
//! - `tokio` - Async runtime for async examples
//! - `futures` - Stream processing utilities

pub mod common {
    use thiserror::Error;

    /// Common parse error type used across examples
    #[derive(Error, Debug, PartialEq)]
    pub enum ParseError {
        #[error("the input was empty")]
        EmptyInput,
        #[error("the input format was invalid")]
        InvalidFormat,
        #[error("the number was too large")]
        NumberTooLarge,
    }

    /// Parse a number from string input
    pub fn parse_number(input: &str) -> Result<i32, ParseError> {
        if input.is_empty() {
            return Err(ParseError::EmptyInput);
        }
        input.trim().parse().map_err(|_| ParseError::InvalidFormat)
    }
}
