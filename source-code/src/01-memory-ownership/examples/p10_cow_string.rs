// Pattern 10: Conditional String Processing with Cow
use std::borrow::Cow;

fn normalize_whitespace(text: &str) -> Cow<'_, str> {
    if text.contains("  ") || text.contains('\t') {
        Cow::Owned(text.replace("  ", " ").replace('\t', " "))
    } else {
        Cow::Borrowed(text)
    }
}

fn main() {
    let clean = normalize_whitespace("hello world");      // Borrowed
    let fixed = normalize_whitespace("hello  world");     // Owned

    println!("Clean: {} (borrowed: {})", clean, matches!(clean, Cow::Borrowed(_)));
    println!("Fixed: {} (borrowed: {})", fixed, matches!(fixed, Cow::Borrowed(_)));
    println!("Cow string example completed");
}
