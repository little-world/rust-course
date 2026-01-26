//! Pattern 6: Character and Grapheme Iteration
//! Three Levels of String Iteration
//!
//! Run with: cargo run --example p6_grapheme_iteration

use unicode_segmentation::UnicodeSegmentation;

fn main() {
    println!("=== Character and Grapheme Iteration ===\n");

    // Simple ASCII
    println!("=== ASCII String ===\n");
    analyze_string("Hello");

    // Multi-byte UTF-8
    println!("\n=== Multi-byte UTF-8 ===\n");
    analyze_string("Héllo");

    // Emoji with modifier
    println!("\n=== Complex Emoji ===\n");
    analyze_string("👨‍👩‍👧‍👦");  // Family emoji (7 code points, 1 grapheme)

    // Display width
    println!("\n=== Display Width ===\n");

    let texts = ["Hello", "你好", "👋", "café"];
    for text in texts {
        println!("'{}': {} chars, {} display width",
                 text, text.chars().count(), display_width(text));
    }

    // Safe truncation
    println!("\n=== Safe Truncation ===\n");

    let text = "Hello, 世界! 👋";
    println!("Original: '{}'", text);
    println!("Truncate at 5 chars: '{}'", truncate_at_char(text, 5));
    println!("Truncate at 10 chars: '{}'", truncate_at_char(text, 10));

    // Grapheme-aware truncation
    println!("\n=== Grapheme-Aware Truncation ===\n");

    let emoji_text = "Hello 👨‍👩‍👧‍👦 World";
    println!("Original: '{}'", emoji_text);
    println!("Truncate at 7 graphemes: '{}'", truncate_at_grapheme(emoji_text, 7));

    // Reversal
    println!("\n=== String Reversal ===\n");

    let texts = ["Hello", "café", "👋🏻"];
    for text in texts {
        println!("Original: '{}' -> Reversed: '{}'", text, reverse_graphemes(text));
    }

    println!("\n=== Key Points ===");
    println!("1. Bytes < Characters < Graphemes (increasing abstraction)");
    println!("2. Use graphemes for user-perceived characters");
    println!("3. Use char_indices() for safe byte-index iteration");
    println!("4. Display width differs from byte/char count for CJK");
}

fn analyze_string(s: &str) {
    println!("String: {:?}", s);
    println!("Byte length: {}", s.len());

    // Byte iteration
    println!("\nBytes:");
    for (i, byte) in s.bytes().enumerate() {
        print!("{:02X} ", byte);
        if (i + 1) % 8 == 0 {
            println!();
        }
    }
    println!();

    // Character (code point) iteration
    println!("\nCharacters (code points):");
    for (i, ch) in s.chars().enumerate() {
        println!("  {}: '{}' (U+{:04X})", i, ch, ch as u32);
    }

    // Grapheme cluster iteration
    println!("\nGrapheme clusters:");
    for (i, grapheme) in s.graphemes(true).enumerate() {
        println!("  {}: '{}'", i, grapheme);
    }

    println!("\nChar count: {}", s.chars().count());
    println!("Grapheme count: {}", s.graphemes(true).count());
}

fn display_width(s: &str) -> usize {
    s.chars().map(|c| {
        let cp = c as u32;
        // Simplified: full-width chars count as 2
        if (0x1100..=0x115F).contains(&cp)     // Hangul Jamo
            || (0x2E80..=0x9FFF).contains(&cp)  // CJK
            || (0xAC00..=0xD7AF).contains(&cp)  // Hangul Syllables
            || (0xFF00..=0xFF60).contains(&cp)  // Fullwidth Forms
        {
            2
        } else {
            1
        }
    }).sum()
}

fn truncate_at_char(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

fn truncate_at_grapheme(s: &str, max_graphemes: usize) -> String {
    s.graphemes(true)
        .take(max_graphemes)
        .collect()
}

fn reverse_graphemes(s: &str) -> String {
    s.graphemes(true).rev().collect()
}
