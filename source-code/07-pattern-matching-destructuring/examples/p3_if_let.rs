//! Pattern 3: if let, while let, and let-else
//! Example: if let and if let Chains
//!
//! Run with: cargo run --example p3_if_let

#[derive(Debug)]
struct Claims {
    user_id: u64,
    role: String,
}

struct Token;

struct Request {
    authorization: Option<String>,
    method: String,
}

fn parse_token(auth: &str) -> Result<Token, &'static str> {
    if auth.starts_with("Bearer ") {
        Ok(Token)
    } else {
        Err("Invalid token format")
    }
}

fn validate_token(_token: &Token) -> Result<Claims, &'static str> {
    // Simplified validation
    Ok(Claims {
        user_id: 42,
        role: "user".to_string(),
    })
}

// Without if-let chains: nested if-let (pyramid of doom)
fn handle_request_nested(req: &Request) -> Option<Claims> {
    if let Some(auth) = &req.authorization {
        if let Ok(token) = parse_token(auth) {
            if let Ok(claims) = validate_token(&token) {
                println!("Nested: Authenticated user {}", claims.user_id);
                return Some(claims);
            }
        }
    }
    println!("Nested: Authentication failed");
    None
}

// With if-let chains (Rust 1.79+): much flatter!
fn handle_request_chained(req: &Request) -> Option<Claims> {
    if let Some(auth) = &req.authorization
        && let Ok(token) = parse_token(auth)
        && let Ok(claims) = validate_token(&token)
    {
        println!("Chained: Authenticated user {}", claims.user_id);
        return Some(claims);
    }
    println!("Chained: Authentication failed");
    None
}

// Combining if-let with boolean conditions
fn handle_admin_request(req: &Request) -> Option<Claims> {
    if let Some(auth) = &req.authorization
        && let Ok(token) = parse_token(auth)
        && let Ok(claims) = validate_token(&token)
        && claims.role == "admin" // Regular boolean condition!
    {
        println!("Admin access granted to user {}", claims.user_id);
        return Some(claims);
    }
    println!("Admin access denied");
    None
}

// Simple if-let for single pattern
fn process_value(opt: Option<i32>) {
    if let Some(n) = opt {
        println!("Got value: {}", n);
    } else {
        println!("No value");
    }
}

// if-let with else-if-let
fn classify_result(res: Result<i32, String>) {
    if let Ok(n) = res {
        if n > 0 {
            println!("Positive: {}", n);
        } else {
            println!("Non-positive: {}", n);
        }
    } else if let Err(e) = res {
        println!("Error: {}", e);
    }
}

fn main() {
    println!("=== Simple if-let ===");
    process_value(Some(42));
    process_value(None);

    println!("\n=== if-let with else-if-let ===");
    classify_result(Ok(10));
    classify_result(Ok(-5));
    classify_result(Err("something went wrong".to_string()));

    println!("\n=== Nested if-let (Old Style) ===");
    let valid_req = Request {
        authorization: Some("Bearer token123".to_string()),
        method: "GET".to_string(),
    };
    handle_request_nested(&valid_req);

    let invalid_req = Request {
        authorization: Some("InvalidToken".to_string()),
        method: "GET".to_string(),
    };
    handle_request_nested(&invalid_req);

    let no_auth_req = Request {
        authorization: None,
        method: "GET".to_string(),
    };
    handle_request_nested(&no_auth_req);

    println!("\n=== if-let Chains (Modern Style) ===");
    handle_request_chained(&valid_req);
    handle_request_chained(&invalid_req);
    handle_request_chained(&no_auth_req);

    println!("\n=== if-let Chains with Boolean Conditions ===");
    handle_admin_request(&valid_req);

    println!("\n=== if-let Chain Syntax ===");
    println!("if let pattern1 = expr1");
    println!("    && let pattern2 = expr2");
    println!("    && boolean_condition");
    println!("{{");
    println!("    // all patterns matched and conditions true");
    println!("}}");

    println!("\n=== When to Use if-let ===");
    println!("1. Single pattern to match (simpler than full match)");
    println!("2. Chained validations (flatter than nested if-let)");
    println!("3. Combining pattern matches with boolean conditions");
    println!("4. Early returns from complex validation logic");
}
