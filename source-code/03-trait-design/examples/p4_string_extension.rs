//! Pattern 4: Extension Traits
//! Example: Extending Standard String Types
//!
//! Run with: cargo run --example p4_string_extension

// Extension for all string-like types
trait StringExt {
    fn truncate_to(&self, max_len: usize) -> String;
    fn remove_whitespace(&self) -> String;
    fn is_blank(&self) -> bool;
    fn word_count(&self) -> usize;
}

impl<T: AsRef<str>> StringExt for T {
    fn truncate_to(&self, max_len: usize) -> String {
        let s = self.as_ref();
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }

    fn remove_whitespace(&self) -> String {
        self.as_ref().chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn is_blank(&self) -> bool {
        self.as_ref().trim().is_empty()
    }

    fn word_count(&self) -> usize {
        self.as_ref().split_whitespace().count()
    }
}

// Extension for case conversion with more options
trait CaseExt {
    fn to_title_case(&self) -> String;
    fn to_snake_case(&self) -> String;
}

impl<T: AsRef<str>> CaseExt for T {
    fn to_title_case(&self) -> String {
        self.as_ref()
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().chain(chars.map(|c| c.to_ascii_lowercase())).collect()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn to_snake_case(&self) -> String {
        let s = self.as_ref();
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        }
        result
    }
}

fn main() {
    // Usage: Extension adds truncate_to() to all string-like types.
    println!("=== truncate_to() ===");
    let s = "This is a long string that needs truncation";
    let truncated = s.truncate_to(20);
    println!("Original: {}", s);
    println!("Truncated: {}", truncated);

    // Works with String too
    let owned = String::from("Another long string for testing");
    println!("Owned truncated: {}", owned.truncate_to(15));

    println!("\n=== remove_whitespace() ===");
    let spaced = "  hello   world  ";
    println!("'{}' -> '{}'", spaced, spaced.remove_whitespace());

    println!("\n=== is_blank() ===");
    println!("'   ' is blank: {}", "   ".is_blank());
    println!("'hi' is blank: {}", "hi".is_blank());
    println!("'' is blank: {}", "".is_blank());

    println!("\n=== word_count() ===");
    let text = "The quick brown fox jumps over the lazy dog";
    println!("'{}' has {} words", text, text.word_count());

    println!("\n=== to_title_case() ===");
    let lower = "hello world from rust";
    println!("'{}' -> '{}'", lower, lower.to_title_case());

    println!("\n=== to_snake_case() ===");
    let camel = "MyVariableName";
    println!("'{}' -> '{}'", camel, camel.to_snake_case());

    let pascal = "GetUserById";
    println!("'{}' -> '{}'", pascal, pascal.to_snake_case());
}
