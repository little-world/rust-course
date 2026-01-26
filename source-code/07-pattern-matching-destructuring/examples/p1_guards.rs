//! Pattern 1: Advanced Match Patterns
//! Example: Guards for Complex Conditions
//!
//! Run with: cargo run --example p1_guards

#[derive(Debug)]
struct Response {
    status: u16,
    body: String,
}

#[derive(Debug)]
struct Error {
    code: u16,
    message: String,
}

fn process_request(status: u16, body: &str) -> Result<Response, Error> {
    match (status, body.len()) {
        // Guard checks additional condition after pattern matches
        (200, len) if len > 0 => Ok(Response {
            status: 200,
            body: body.to_string(),
        }),
        // 200 with empty body is an error
        (200, _) => Err(Error {
            code: 200,
            message: "Empty response body".to_string(),
        }),
        // Bind status with @ while matching range
        (s @ 400..=499, _) => Err(Error {
            code: s,
            message: format!("Client error: {}", s),
        }),
        (s @ 500..=599, _) => Err(Error {
            code: s,
            message: format!("Server error: {}", s),
        }),
        // Catch-all for other status codes
        (s, _) => Err(Error {
            code: s,
            message: format!("Unexpected status: {}", s),
        }),
    }
}

fn classify_number(n: i32) -> &'static str {
    match n {
        x if x < 0 => "negative",
        0 => "zero",
        x if x % 2 == 0 => "positive even",
        _ => "positive odd",
    }
}

fn describe_pair(pair: (i32, i32)) -> &'static str {
    match pair {
        (x, y) if x == y => "equal",
        (x, y) if x + y == 0 => "opposites",
        (x, y) if x > y => "first is larger",
        (x, y) if x < y => "second is larger",
        _ => "unknown", // Actually unreachable, but needed for exhaustiveness
    }
}

fn main() {
    println!("=== Guards for HTTP Response Processing ===");
    // Usage: match HTTP status with body length using guards
    let cases = [
        (200, "data"),
        (200, ""),
        (404, "not found"),
        (500, "internal error"),
        (301, "redirect"),
    ];

    for (status, body) in cases {
        let result = process_request(status, body);
        println!("  ({}, \"{}\") => {:?}", status, body, result);
    }

    println!("\n=== Guards for Number Classification ===");
    let numbers = [-5, 0, 4, 7];
    for n in numbers {
        println!("  {} => {}", n, classify_number(n));
    }

    println!("\n=== Guards for Tuple Matching ===");
    let pairs = [(5, 5), (3, -3), (10, 5), (2, 8)];
    for pair in pairs {
        println!("  {:?} => {}", pair, describe_pair(pair));
    }

    println!("\n=== Guard Syntax ===");
    println!("  pattern if condition => {{ ... }}");
    println!("\nGuards are checked AFTER the pattern matches.");
    println!("Variables bound by the pattern can be used in the guard.");

    println!("\n=== Guards vs Nested Patterns ===");
    println!("Guards are needed when:");
    println!("  - Conditions involve computed values (x % 2 == 0)");
    println!("  - Conditions compare multiple bound variables (x == y)");
    println!("  - Conditions involve function calls");
}
