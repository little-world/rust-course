//! Pattern 4: Cow for Conditional Allocation
//! Normalize Whitespace and Strip Affixes
//!
//! Run with: cargo run --example p4_cow_whitespace

use std::borrow::Cow;

fn main() {
    println!("=== Cow for Conditional Allocation ===\n");

    // Whitespace normalization
    println!("=== Whitespace Normalization ===\n");

    let s1 = "hello world";      // No normalization needed
    let s2 = "hello  world";     // Multiple spaces
    let s3 = "hello   world  !"; // Multiple groups of spaces

    println!("Input: '{}' -> {:?}", s1, normalize_whitespace(s1));
    println!("Input: '{}' -> {:?}", s2, normalize_whitespace(s2));
    println!("Input: '{}' -> {:?}", s3, normalize_whitespace(s3));

    // Strip prefix/suffix
    println!("\n=== Strip Prefix/Suffix ===\n");

    let texts = [
        ("hello", "", ""),           // No changes
        ("[hello]", "[", "]"),       // Strip both
        ("[hello", "[", "]"),        // Strip prefix only
        ("hello]", "[", "]"),        // Strip suffix only
    ];

    for (text, prefix, suffix) in texts {
        let result = strip_affixes(text, prefix, suffix);
        println!("strip_affixes('{}', '{}', '{}') -> {:?}", text, prefix, suffix, result);
    }

    // Case normalization
    println!("\n=== Case Normalization ===\n");

    let words = ["hello", "Hello", "HELLO", "HeLLo"];

    for word in words {
        let result = to_lowercase_if_needed(word);
        println!("to_lowercase_if_needed('{}') -> {:?}", word, result);
    }

    println!("\n=== Key Points ===");
    println!("1. Cow::Borrowed when no modification needed");
    println!("2. Cow::Owned when modification required");
    println!("3. Fast-path check avoids allocation in common case");
    println!("4. O(N) scan is cheaper than unnecessary allocation");
}

fn normalize_whitespace(s: &str) -> Cow<str> {
    let mut prev_was_space = false;
    let mut needs_normalization = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if prev_was_space {
                needs_normalization = true;
                break;
            }
            prev_was_space = true;
        } else {
            prev_was_space = false;
        }
    }

    if !needs_normalization {
        return Cow::Borrowed(s);
    }

    // Build normalized string
    let mut result = String::with_capacity(s.len());
    let mut prev_was_space = false;

    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(c);
            prev_was_space = false;
        }
    }

    Cow::Owned(result)
}

fn strip_affixes<'a>(s: &'a str, prefix: &str, suffix: &str) -> Cow<'a, str> {
    let mut start = 0;
    let mut end = s.len();

    if s.starts_with(prefix) {
        start = prefix.len();
    }

    if s.ends_with(suffix) {
        end = end.saturating_sub(suffix.len());
    }

    if start == 0 && end == s.len() {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s[start..end].to_string())
    }
}

fn to_lowercase_if_needed(s: &str) -> Cow<str> {
    if s.chars().all(|c| !c.is_uppercase()) {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_lowercase())
    }
}
