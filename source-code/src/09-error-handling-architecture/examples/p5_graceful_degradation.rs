//! Pattern 5: Recoverable vs Unrecoverable Errors
//! Example: Graceful Degradation with Fallbacks
//!
//! Run with: cargo run --example p5_graceful_degradation

struct User {
    id: u64,
    name: String,
}

impl User {
    fn unknown(id: u64) -> Self {
        User {
            id,
            name: "Unknown".to_string(),
        }
    }
}

/// Simulated cache lookup (might fail).
fn fetch_from_cache(id: u64) -> Result<User, &'static str> {
    // Simulate cache miss
    Err("cache miss")
}

/// Simulated database lookup (might fail).
fn fetch_from_db(id: u64) -> Result<User, &'static str> {
    // Simulate success for even IDs, failure for odd
    if id % 2 == 0 {
        Ok(User {
            id,
            name: format!("User{}", id),
        })
    } else {
        Err("database timeout")
    }
}

/// Get user with graceful degradation.
fn get_user_with_fallback(id: u64) -> User {
    // Try cache first
    match fetch_from_cache(id) {
        Ok(user) => {
            println!("  [CACHE HIT] User {} from cache", id);
            return user;
        }
        Err(e) => {
            println!("  [CACHE MISS] {}, trying database...", e);
        }
    }

    // Fall back to database
    match fetch_from_db(id) {
        Ok(user) => {
            println!("  [DB HIT] User {} from database", id);
            user
        }
        Err(e) => {
            println!("  [DB FAIL] {}, using default", e);
            User::unknown(id)
        }
    }
}

/// Configuration with defaults.
fn get_config_value(key: &str) -> String {
    // Try environment variable
    if let Ok(value) = std::env::var(key) {
        println!("  [ENV] {} = {}", key, value);
        return value;
    }

    // Fall back to default
    let default = match key {
        "PORT" => "8080",
        "HOST" => "127.0.0.1",
        "LOG_LEVEL" => "info",
        _ => "default",
    };
    println!("  [DEFAULT] {} = {}", key, default);
    default.to_string()
}

/// Read file with fallback content.
fn read_or_create_default(path: &str, default: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            println!("  [FILE] Read from {}", path);
            content
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("  [CREATE] {} not found, creating with default", path);
            let _ = std::fs::write(path, default);
            default.to_string()
        }
        Err(e) => {
            println!("  [ERROR] Failed to read {}: {}, using default", path, e);
            default.to_string()
        }
    }
}

fn main() {
    println!("=== Graceful Degradation ===\n");

    // User lookup with cache -> DB -> default
    println!("=== User Lookup Chain ===");
    for id in [2, 3, 4, 5] {
        let user = get_user_with_fallback(id);
        println!("  Result: {} (name: {})\n", user.id, user.name);
    }

    // Config with environment -> default
    println!("=== Config with Defaults ===");
    let port = get_config_value("PORT");
    let host = get_config_value("HOST");
    let log_level = get_config_value("LOG_LEVEL");
    let custom = get_config_value("CUSTOM_KEY");
    println!("  Final config: {}:{} log={} custom={}\n", host, port, log_level, custom);

    // File with creation fallback
    println!("=== File with Creation ===");
    let test_path = "/tmp/graceful_test.txt";
    let _ = std::fs::remove_file(test_path); // Ensure it doesn't exist

    let content1 = read_or_create_default(test_path, "default content");
    println!("  First read: '{}'\n", content1.trim());

    let content2 = read_or_create_default(test_path, "default content");
    println!("  Second read: '{}'", content2.trim());

    // Cleanup
    let _ = std::fs::remove_file(test_path);

    println!("\n=== Degradation Patterns ===");
    println!("1. Cache -> Database -> Default");
    println!("2. Primary -> Secondary -> Tertiary service");
    println!("3. Environment -> Config file -> Hardcoded default");
    println!("4. Fresh data -> Stale cache -> Error message");

    println!("\n=== Implementation ===");
    println!("match primary() {{");
    println!("    Ok(v) => v,");
    println!("    Err(_) => match secondary() {{");
    println!("        Ok(v) => v,");
    println!("        Err(_) => default(),");
    println!("    }}");
    println!("}}");
    println!();
    println!("Or with unwrap_or_else:");
    println!("primary().unwrap_or_else(|_| default())");

    println!("\n=== Key Points ===");
    println!("1. Log each fallback for observability");
    println!("2. Don't hide errors - degrade visibly");
    println!("3. Service stays up even when dependencies fail");
    println!("4. Users get partial functionality vs nothing");
}
