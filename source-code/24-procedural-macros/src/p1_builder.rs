//! Pattern 1: Derive Macro with Attributes (Builder Pattern)
//!
//! Generates a complete builder pattern implementation automatically.
//! Creates FooBuilder struct with Option<T> fields, fluent setters, and build() method.

use my_macros::Builder;

#[derive(Debug, Builder)]
struct User {
    username: String,
    email: String,
    age: u32,
}

#[derive(Debug, Builder)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

fn main() {
    println!("=== Builder Pattern Derive Demo ===\n");

    // Build a User with the generated builder
    let user = User::builder()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build()
        .unwrap();

    println!("Built user: {:?}", user);

    // Build a Config
    let config = Config::builder()
        .host("localhost".to_string())
        .port(8080)
        .debug(true)
        .build()
        .unwrap();

    println!("Built config: {:?}", config);

    // Demonstrate error when field is missing
    let result = User::builder()
        .username("bob".to_string())
        .build();

    println!("\nMissing fields error: {:?}", result.err());
}
