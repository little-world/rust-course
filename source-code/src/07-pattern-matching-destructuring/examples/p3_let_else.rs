//! Pattern 3: if let, while let, and let-else
//! Example: let-else for Early Returns
//!
//! Run with: cargo run --example p3_let_else

#[derive(Debug)]
struct Claims {
    user_id: u64,
    role: String,
}

#[derive(Debug)]
enum AuthError {
    MissingAuth,
    InvalidToken,
    Unauthorized,
}

struct Request {
    authorization: Option<String>,
}

fn parse_token(auth: &str) -> Result<Claims, &'static str> {
    if auth.starts_with("Bearer ") && auth.len() > 10 {
        Ok(Claims {
            user_id: 42,
            role: "user".to_string(),
        })
    } else {
        Err("Invalid token")
    }
}

// Using let-else for guard clauses
fn get_user_id(request: &Request) -> Result<u64, AuthError> {
    // let-else: bind if pattern matches, else return/break/continue/panic
    let Some(auth_header) = &request.authorization else {
        return Err(AuthError::MissingAuth);
    };

    let Ok(claims) = parse_token(auth_header) else {
        return Err(AuthError::InvalidToken);
    };

    // After the let-else blocks, we know auth_header and claims are valid
    println!("Authenticated user: {}", claims.user_id);
    Ok(claims.user_id)
}

// Compare: same logic with nested if-let (harder to read)
fn get_user_id_nested(request: &Request) -> Result<u64, AuthError> {
    if let Some(auth_header) = &request.authorization {
        if let Ok(claims) = parse_token(auth_header) {
            println!("Authenticated user: {}", claims.user_id);
            return Ok(claims.user_id);
        } else {
            return Err(AuthError::InvalidToken);
        }
    } else {
        return Err(AuthError::MissingAuth);
    }
}

// let-else with more complex patterns
fn process_pair(pair: Option<(i32, i32)>) -> i32 {
    let Some((a, b)) = pair else {
        println!("No pair provided, using default");
        return 0;
    };

    // a and b are now in scope
    println!("Processing pair: ({}, {})", a, b);
    a + b
}

// let-else in loops
fn find_first_positive(numbers: &[i32]) -> Option<i32> {
    for &n in numbers {
        let positive @ 1.. = n else {
            continue; // let-else can use continue in loops
        };
        return Some(positive);
    }
    None
}

// let-else must diverge (return, break, continue, or panic)
fn must_have_value(opt: Option<i32>) -> i32 {
    let Some(value) = opt else {
        panic!("Value was required but not provided!");
    };
    value
}

fn main() {
    println!("=== let-else for Guard Clauses ===");
    // Usage: extract user ID with early returns for errors
    let valid_req = Request {
        authorization: Some("Bearer abc123xyz".to_string()),
    };
    let invalid_req = Request {
        authorization: Some("Bad".to_string()),
    };
    let no_auth_req = Request {
        authorization: None,
    };

    println!("Valid request: {:?}", get_user_id(&valid_req));
    println!("Invalid token: {:?}", get_user_id(&invalid_req));
    println!("No auth: {:?}", get_user_id(&no_auth_req));

    println!("\n=== Nested if-let (for comparison) ===");
    println!("Valid request: {:?}", get_user_id_nested(&valid_req));

    println!("\n=== let-else with Tuples ===");
    let sum = process_pair(Some((3, 5)));
    println!("Sum: {}", sum);
    let default = process_pair(None);
    println!("Default: {}", default);

    println!("\n=== let-else in Loops ===");
    let numbers = [-3, -1, 0, 2, 5, -4];
    if let Some(n) = find_first_positive(&numbers) {
        println!("First positive: {}", n);
    }

    println!("\n=== let-else Syntax ===");
    println!("let pattern = expression else {{");
    println!("    // must diverge: return, break, continue, or panic");
    println!("}};");
    println!("// pattern variables are now in scope");

    println!("\n=== Benefits of let-else ===");
    println!("1. Flat control flow (no nesting)");
    println!("2. Early returns at function start");
    println!("3. Variables bound in pattern available after");
    println!("4. Clearly separates success path from error handling");

    println!("\n=== let-else vs if-let ===");
    println!("if-let: when you need else block to do more work");
    println!("let-else: when else should immediately diverge");
    println!("          (cleaner for guard clauses)");
}
