//! Pattern 5: Recoverable vs Unrecoverable Errors
//! Example: When to Panic vs Return Result
//!
//! Run with: cargo run --example p5_panic_vs_result

struct User {
    id: u64,
    name: String,
}

/// PANIC: Index access panics on out-of-bounds (caller bug).
fn get_user_by_index(users: &[User], index: usize) -> &User {
    &users[index] // Panics if index >= users.len()
}

/// RESULT: Find returns None for missing items (expected case).
fn find_user_by_id(users: &[User], id: u64) -> Option<&User> {
    users.iter().find(|u| u.id == id)
}

/// PANIC: Unwrap on "impossible" error.
fn parse_hardcoded_port() -> u16 {
    "8080".parse().unwrap() // Known to succeed
}

/// RESULT: Parse user input returns Result.
fn parse_user_port(input: &str) -> Result<u16, std::num::ParseIntError> {
    input.trim().parse()
}

/// PANIC: Assert for programming invariants.
fn calculate_average(values: &[f64]) -> f64 {
    assert!(!values.is_empty(), "Cannot calculate average of empty slice");
    values.iter().sum::<f64>() / values.len() as f64
}

/// RESULT: Empty input is valid, return None.
fn calculate_average_safe(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        None
    } else {
        Some(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn main() {
    println!("=== Panic vs Result ===\n");

    let users = vec![
        User { id: 1, name: "Alice".into() },
        User { id: 2, name: "Bob".into() },
    ];

    // Index access (panics on invalid)
    println!("=== Index Access (panics on invalid) ===");
    let user = get_user_by_index(&users, 0);
    println!("  users[0] = {}", user.name);
    // get_user_by_index(&users, 99); // Would panic!

    // Find by ID (returns Option)
    println!("\n=== Find by ID (returns Option) ===");
    match find_user_by_id(&users, 1) {
        Some(u) => println!("  Found user 1: {}", u.name),
        None => println!("  User 1 not found"),
    }
    match find_user_by_id(&users, 99) {
        Some(u) => println!("  Found user 99: {}", u.name),
        None => println!("  User 99 not found (expected)"),
    }

    // Hardcoded parse (safe to unwrap)
    println!("\n=== Hardcoded Parse (safe to unwrap) ===");
    let port = parse_hardcoded_port();
    println!("  Default port: {}", port);

    // User input parse (returns Result)
    println!("\n=== User Input Parse (returns Result) ===");
    for input in &["3000", "abc", ""] {
        match parse_user_port(input) {
            Ok(p) => println!("  '{}' -> port {}", input, p),
            Err(e) => println!("  '{}' -> error: {}", input, e),
        }
    }

    // Average with assertion
    println!("\n=== Average with Assertion ===");
    let values = vec![1.0, 2.0, 3.0];
    println!("  Average of {:?}: {}", values, calculate_average(&values));
    // calculate_average(&[]); // Would panic!

    // Safe average
    println!("\n=== Safe Average (returns Option) ===");
    println!("  Average of [1,2,3]: {:?}", calculate_average_safe(&[1.0, 2.0, 3.0]));
    println!("  Average of []: {:?}", calculate_average_safe(&[]));

    println!("\n=== Decision Guide ===");
    println!("Use PANIC when:");
    println!("  - Caller violated API contract (index out of bounds)");
    println!("  - Invariant broken (impossible state reached)");
    println!("  - Parse of hardcoded/known-good value");
    println!("  - Test code (unwrap is fine)");
    println!();
    println!("Use RESULT when:");
    println!("  - Failure is expected (file not found, user not found)");
    println!("  - External input (user input, network data)");
    println!("  - Library code (let caller decide)");
    println!("  - Recoverable error (retry, fallback)");
    println!();
    println!("Use OPTION when:");
    println!("  - Absence is valid (no user with ID)");
    println!("  - No error context needed");
    println!("  - Lookup/search operations");

    println!("\n=== Key Points ===");
    println!("1. Panic = bug in caller, Result = expected failure");
    println!("2. Libraries should prefer Result");
    println!("3. Applications can panic on startup for missing config");
    println!("4. Tests can use unwrap() liberally");
}
