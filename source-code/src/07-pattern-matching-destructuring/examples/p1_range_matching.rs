//! Pattern 1: Advanced Match Patterns
//! Example: Range Matching
//!
//! Run with: cargo run --example p1_range_matching

fn classify_temperature(temp: i32) -> &'static str {
    match temp {
        i32::MIN..=-40 => "extreme cold",
        -39..=-20 => "very cold",
        -19..=0 => "cold",
        1..=15 => "cool",
        16..=25 => "comfortable",
        26..=35 => "warm",
        36..=45 => "hot",
        46..=i32::MAX => "extreme heat",
    }
}

fn classify_http_status(status: u16) -> &'static str {
    match status {
        100..=199 => "Informational",
        200..=299 => "Success",
        300..=399 => "Redirection",
        400..=499 => "Client Error",
        500..=599 => "Server Error",
        _ => "Unknown",
    }
}

fn classify_character(ch: char) -> &'static str {
    match ch {
        'a'..='z' => "lowercase letter",
        'A'..='Z' => "uppercase letter",
        '0'..='9' => "digit",
        ' ' | '\t' | '\n' => "whitespace",
        '!' | '?' | '.' | ',' => "punctuation",
        _ => "other",
    }
}

fn main() {
    println!("=== Range Matching: Temperature ===");
    // Usage: classify temperatures into human-readable categories
    let temps = [-50, -30, -10, 5, 20, 30, 40, 60];
    for temp in temps {
        println!("  {}°C => {}", temp, classify_temperature(temp));
    }

    assert_eq!(classify_temperature(20), "comfortable");
    assert_eq!(classify_temperature(-50), "extreme cold");

    println!("\n=== Range Matching: HTTP Status ===");
    let statuses = [100, 200, 301, 404, 500, 999];
    for status in statuses {
        println!("  {} => {}", status, classify_http_status(status));
    }

    println!("\n=== Range Matching: Characters ===");
    let chars = ['a', 'Z', '5', ' ', '!', '@'];
    for ch in chars {
        println!("  '{}' => {}", ch, classify_character(ch));
    }

    println!("\n=== Why Range Patterns? ===");
    println!("Without range patterns, classify_temperature would need:");
    println!("  if temp <= -40 {{ ... }}");
    println!("  else if temp <= -20 {{ ... }}");
    println!("  else if temp <= 0 {{ ... }}");
    println!("  // ... 8+ conditions!");
    println!("\nRange patterns make this concise and exhaustive.");

    println!("\n=== Key Syntax ===");
    println!("  a..=b  => inclusive range [a, b]");
    println!("  a..b   => exclusive range [a, b) (not allowed in match yet)");
    println!("  i32::MIN..=x => from minimum to x");
    println!("  x..=i32::MAX => from x to maximum");
}
