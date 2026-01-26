//! Pattern 1: Custom Error Enums for Libraries
//! Example: #[non_exhaustive] for Library Stability
//!
//! Run with: cargo run --example p1_non_exhaustive

use thiserror::Error;

/// Non-exhaustive error enum allows adding variants without breaking changes.
/// Users of this library must include a wildcard `_` arm in match statements.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("the network request failed")]
    NetworkError,
    #[error("the request timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    #[error("authentication failed")]
    AuthError,
    // New variants can be added in future versions without breaking
    // downstream code, because callers must have a wildcard arm.
}

/// Simulates an API call that might fail in various ways.
fn make_api_call(scenario: &str) -> Result<String, ApiError> {
    match scenario {
        "network" => Err(ApiError::NetworkError),
        "timeout" => Err(ApiError::Timeout { timeout_ms: 5000 }),
        "auth" => Err(ApiError::AuthError),
        _ => Ok("Success!".to_string()),
    }
}

/// Handle API errors with a wildcard for future-proofing.
fn handle_error(err: ApiError) {
    match err {
        ApiError::Timeout { timeout_ms } => {
            println!("  -> Retrying after {}ms timeout...", timeout_ms);
        }
        ApiError::NetworkError => {
            println!("  -> Check network connection");
        }
        ApiError::AuthError => {
            println!("  -> Please re-authenticate");
        }
        // Required due to #[non_exhaustive] - handles future variants
        _ => {
            println!("  -> Unknown error: {}", err);
        }
    }
}

fn main() {
    println!("=== #[non_exhaustive] Error Enums ===\n");

    println!("Testing different error scenarios:\n");

    let scenarios = vec!["network", "timeout", "auth", "success"];

    for scenario in scenarios {
        println!("Scenario '{}':", scenario);
        match make_api_call(scenario) {
            Ok(msg) => println!("  -> {}", msg),
            Err(e) => handle_error(e),
        }
        println!();
    }

    println!("=== Why #[non_exhaustive]? ===");
    println!("1. Allows adding new error variants in future library versions");
    println!("2. Forces callers to have a wildcard `_` match arm");
    println!("3. Adding variants is NOT a breaking change");
    println!("4. Downstream code continues to compile and work");

    println!("\n=== Example: Future-Proof Match ===");
    println!("match err {{");
    println!("    ApiError::Timeout {{ .. }} => retry(),");
    println!("    ApiError::NetworkError => check_connection(),");
    println!("    ApiError::AuthError => reauthenticate(),");
    println!("    _ => log_unknown_error(err),  // REQUIRED for non_exhaustive");
    println!("}}");
}
