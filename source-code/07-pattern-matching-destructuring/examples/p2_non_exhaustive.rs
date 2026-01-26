//! Pattern 2: Exhaustiveness and Match Ergonomics
//! Example: The #[non_exhaustive] Attribute
//!
//! Run with: cargo run --example p2_non_exhaustive

// In a real library, this would be `pub enum` in a separate crate.
// #[non_exhaustive] tells external users they must use a wildcard.
// Within the defining crate, the enum is still exhaustive.
#[non_exhaustive]
#[derive(Debug)]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    // Future: Http3 could be added without breaking external code
}

// In the SAME crate, we can match exhaustively (no wildcard needed)
fn version_string_internal(version: &HttpVersion) -> &'static str {
    match version {
        HttpVersion::Http10 => "HTTP/1.0",
        HttpVersion::Http11 => "HTTP/1.1",
        HttpVersion::Http2 => "HTTP/2",
        // No wildcard needed in the defining crate!
    }
}

// But library USERS would need to write it like this:
fn version_string_external(version: &HttpVersion) -> &'static str {
    match version {
        HttpVersion::Http10 => "HTTP/1.0",
        HttpVersion::Http11 => "HTTP/1.1",
        HttpVersion::Http2 => "HTTP/2",
        // External users MUST include this wildcard.
        // If library adds Http3, their code won't break.
        _ => "Unknown HTTP version",
    }
}

// Structs can also be non_exhaustive
#[non_exhaustive]
#[derive(Debug)]
pub struct Config {
    pub timeout: u32,
    pub retries: u32,
    // Future fields could be added
}

impl Config {
    // Users can't construct Config directly with struct literal
    // They must use this constructor
    pub fn new(timeout: u32, retries: u32) -> Self {
        Config { timeout, retries }
    }
}

// non_exhaustive on variants
#[derive(Debug)]
pub enum Error {
    #[non_exhaustive]
    Network { code: i32 },
    #[non_exhaustive]
    Parse { message: String },
}

fn describe_error(err: &Error) -> String {
    match err {
        // Must use `..` because the variant is non_exhaustive
        Error::Network { code, .. } => format!("Network error: {}", code),
        Error::Parse { message, .. } => format!("Parse error: {}", message),
    }
}

fn main() {
    println!("=== #[non_exhaustive] Enum ===");
    let versions = [
        HttpVersion::Http10,
        HttpVersion::Http11,
        HttpVersion::Http2,
    ];

    for version in &versions {
        println!(
            "  {:?} => internal: {}, external: {}",
            version,
            version_string_internal(version),
            version_string_external(version)
        );
    }

    println!("\n=== #[non_exhaustive] Struct ===");
    // Can't do: let config = Config { timeout: 30, retries: 3 };
    // Must use constructor:
    let config = Config::new(30, 3);
    println!("  Config: {:?}", config);
    println!("  (Can only be constructed via Config::new())");

    println!("\n=== #[non_exhaustive] on Variants ===");
    let errors = [
        Error::Network { code: 404 },
        Error::Parse {
            message: "invalid syntax".to_string(),
        },
    ];

    for err in &errors {
        println!("  {} ", describe_error(err));
    }

    println!("\n=== When to Use #[non_exhaustive] ===");
    println!("Use on PUBLIC library types when:");
    println!("  1. You might add enum variants in the future");
    println!("  2. You might add struct fields in the future");
    println!("  3. You want to avoid SemVer breaking changes");

    println!("\n=== Key Behaviors ===");
    println!("For ENUMS:");
    println!("  - External: Must use wildcard in match");
    println!("  - Internal: Can match exhaustively");
    println!();
    println!("For STRUCTS:");
    println!("  - External: Can't use struct literal syntax");
    println!("  - External: Must use `..` in destructuring");
    println!("  - Internal: Full access");
    println!();
    println!("For VARIANTS:");
    println!("  - Must use `..` when destructuring");
}
