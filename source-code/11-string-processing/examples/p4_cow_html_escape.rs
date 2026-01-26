//! Pattern 4: Cow for Conditional Allocation
//! HTML Escaping with Fast-Path Check
//!
//! Run with: cargo run --example p4_cow_html_escape

use std::borrow::Cow;

fn main() {
    println!("=== HTML Escaping with Cow ===\n");

    let test_cases = [
        "Hello World",           // Safe - no escaping needed
        "Hello <World>",         // Needs escaping
        "Price: $100",           // Safe
        "5 > 3 && 3 < 5",        // Multiple escapes
        "Say \"Hello\"",         // Quote escaping
        "Tom & Jerry",           // Ampersand
        "It's fine",             // Single quote
    ];

    for text in test_cases {
        let escaped = escape_html(text);
        let variant = match &escaped {
            Cow::Borrowed(_) => "Borrowed",
            Cow::Owned(_) => "Owned",
        };
        println!("Input:  '{}'\nOutput: '{}' ({})\n", text, escaped, variant);
    }

    // Demonstrate performance benefit
    println!("=== Performance Benefit ===\n");

    let safe_texts: Vec<&str> = (0..1000).map(|_| "Hello World").collect();
    let unsafe_texts: Vec<&str> = (0..1000).map(|_| "Hello <World>").collect();

    // Count allocations
    let safe_allocations = safe_texts.iter()
        .filter(|s| matches!(escape_html(s), Cow::Owned(_)))
        .count();

    let unsafe_allocations = unsafe_texts.iter()
        .filter(|s| matches!(escape_html(s), Cow::Owned(_)))
        .count();

    println!("Safe texts (1000): {} allocations", safe_allocations);
    println!("Unsafe texts (1000): {} allocations", unsafe_allocations);
    println!("\nCow saved {} allocations!", 1000 - safe_allocations);

    println!("\n=== Key Points ===");
    println!("1. Fast-path check: s.contains(&['<', '>', '&', '\"', '\\''][..])");
    println!("2. Most strings don't need escaping - Cow avoids allocation");
    println!("3. Pre-allocate with extra capacity for escapes");
    println!("4. Web servers: 90%+ inputs safe = 90%+ allocation savings");
}

fn escape_html(s: &str) -> Cow<str> {
    if !s.contains(&['<', '>', '&', '"', '\''][..]) {
        return Cow::Borrowed(s);
    }

    let mut escaped = String::with_capacity(s.len() + 20);

    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(c),
        }
    }

    Cow::Owned(escaped)
}
