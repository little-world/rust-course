//! Pattern 3: Advanced Iterator Composition
//! Example: take_while and skip_while for Prefix/Suffix Operations
//!
//! Run with: cargo run --example p3_take_skip_while

/// Split lines at empty line into header and body.
fn extract_header_body(lines: &[String]) -> (Vec<&String>, Vec<&String>) {
    let header: Vec<_> = lines.iter()
        .take_while(|line| !line.is_empty())
        .collect();
    let body: Vec<_> = lines
        .iter()
        .skip_while(|line| !line.is_empty())
        .skip(1) // Skip the empty line itself
        .collect();
    (header, body)
}

/// Extract leading digits from a string.
fn extract_leading_digits(s: &str) -> String {
    s.chars()
        .take_while(|c| c.is_ascii_digit())
        .collect()
}

/// Skip leading whitespace.
fn skip_leading_whitespace(s: &str) -> String {
    s.chars()
        .skip_while(|c| c.is_whitespace())
        .collect()
}

/// Extract comment lines at the start of a file.
fn extract_leading_comments<'a>(lines: &[&'a str]) -> Vec<&'a str> {
    lines.iter()
        .take_while(|line| line.starts_with('#') || line.starts_with("//"))
        .copied()
        .collect()
}

fn main() {
    println!("=== take_while and skip_while ===\n");

    // Usage: split text at empty line into header and body
    let lines: Vec<String> = vec![
        "Content-Type: text/html".into(),
        "Content-Length: 1234".into(),
        "".into(),
        "<html>".into(),
        "<body>Hello</body>".into(),
        "</html>".into(),
    ];

    let (header, body) = extract_header_body(&lines);
    println!("HTTP-like message parsing:");
    println!("Header lines:");
    for line in &header {
        println!("  {}", line);
    }
    println!("Body lines:");
    for line in &body {
        println!("  {}", line);
    }

    println!("\n=== Extract Leading Digits ===");
    let tests = ["123abc456", "42", "nodigits", "007bond"];
    for s in tests {
        let digits = extract_leading_digits(s);
        println!("  '{}' -> '{}'", s, digits);
    }

    println!("\n=== Skip Leading Whitespace ===");
    let padded = "   hello world   ";
    println!("Original: '{}'", padded);
    println!("Trimmed left: '{}'", skip_leading_whitespace(padded));

    println!("\n=== Extract Leading Comments ===");
    let source_lines = [
        "# Configuration file",
        "# Author: Alice",
        "// Also a comment",
        "",
        "key = value",
        "# Not a header comment",
    ];
    let comments = extract_leading_comments(&source_lines);
    println!("Source file leading comments:");
    for comment in comments {
        println!("  {}", comment);
    }

    println!("\n=== Understanding the Behavior ===");
    let numbers = [1, 2, 3, 10, 11, 12, 1, 2];
    println!("Numbers: {:?}", numbers);

    let taken: Vec<_> = numbers.iter().take_while(|&&x| x < 10).collect();
    println!("take_while(x < 10): {:?}", taken);
    // [1, 2, 3] - stops at first >= 10

    let skipped: Vec<_> = numbers.iter().skip_while(|&&x| x < 10).collect();
    println!("skip_while(x < 10): {:?}", skipped);
    // [10, 11, 12, 1, 2] - starts at first >= 10, includes ALL after

    println!("\n=== Important: They Don't Filter! ===");
    println!("take_while STOPS at first false, doesn't filter");
    println!("skip_while STARTS at first false, includes everything after");
    println!("");
    println!("For filtering, use .filter() instead");

    println!("\n=== Key Points ===");
    println!("1. take_while yields until predicate becomes false, then stops");
    println!("2. skip_while discards until predicate becomes false, then yields all");
    println!("3. Neither one filters - they split at a boundary");
    println!("4. Useful for parsing headers/bodies, comments, etc.");
}
