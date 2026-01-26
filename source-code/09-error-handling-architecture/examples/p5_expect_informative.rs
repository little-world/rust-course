//! Pattern 5: Recoverable vs Unrecoverable Errors
//! Example: Informative Expect Messages
//!
//! Run with: cargo run --example p5_expect_informative

fn main() {
    println!("=== Informative Expect Messages ===\n");

    // Good: Informative expect message
    println!("=== Good Expect Messages ===");

    let config_content = "port=8080\nhost=localhost";
    let port_line = config_content
        .lines()
        .find(|l| l.starts_with("port="))
        .expect("config must contain 'port=' line - check config.toml");
    println!("  Found port line: {}", port_line);

    // Demonstrate the difference
    println!("\n=== Message Quality Comparison ===\n");

    // BAD: Unhelpful message
    println!("BAD (uninformative):");
    println!("  .unwrap()");
    println!("  -> panic: called `Option::unwrap()` on a `None` value");
    println!();
    println!("  .expect(\"failed\")");
    println!("  -> panic: failed");
    println!();

    // GOOD: Helpful message
    println!("GOOD (informative):");
    println!("  .expect(\"DATABASE_URL not set - add to .env file\")");
    println!("  -> panic: DATABASE_URL not set - add to .env file");
    println!();
    println!("  .expect(\"config.toml must exist in current directory\")");
    println!("  -> panic: config.toml must exist in current directory");

    // Pattern: Startup validation
    println!("\n=== Startup Validation Pattern ===");

    fn simulate_startup() {
        // In real code, these would read from environment/files
        let db_url = Some("postgres://localhost/mydb");
        let api_key = Some("secret-key-123");

        let _db = db_url.expect("DATABASE_URL must be set in environment");
        let _key = api_key.expect("API_KEY must be set in environment");

        println!("  Startup validation passed!");
    }

    simulate_startup();

    // Pattern: Known-good parse
    println!("\n=== Known-Good Parse Pattern ===");

    let default_port: u16 = "8080"
        .parse()
        .expect("BUG: hardcoded port '8080' must parse");
    println!("  Default port: {}", default_port);

    let default_timeout: u64 = "30"
        .parse()
        .expect("BUG: hardcoded timeout '30' must parse");
    println!("  Default timeout: {}s", default_timeout);

    // Pattern: Internal invariant
    println!("\n=== Internal Invariant Pattern ===");

    fn get_first_word(text: &str) -> &str {
        // After checking is_empty, split always yields at least one item
        if text.is_empty() {
            return "";
        }
        text.split_whitespace()
            .next()
            .expect("BUG: non-empty string must have at least one word")
    }

    println!("  First word of 'hello world': '{}'", get_first_word("hello world"));
    println!("  First word of '': '{}'", get_first_word(""));

    println!("\n=== Expect Message Guidelines ===");
    println!("1. Explain WHAT was expected, not just that it failed");
    println!("2. Include HOW TO FIX if possible");
    println!("3. Prefix with 'BUG:' for internal invariants");
    println!("4. Mention the source (env var name, file path, etc.)");

    println!("\n=== Examples ===");
    println!("Environment:");
    println!("  .expect(\"DATABASE_URL must be set in .env or environment\")");
    println!();
    println!("Configuration:");
    println!("  .expect(\"config.toml must contain [server] section\")");
    println!();
    println!("Internal bug:");
    println!("  .expect(\"BUG: Vec guaranteed non-empty at this point\")");
    println!();
    println!("Initialization:");
    println!("  .expect(\"Logger must be initialized before use\")");
}
