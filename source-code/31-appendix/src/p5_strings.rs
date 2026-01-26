// Pattern 5: String and Text Processing
// Demonstrates String, &str, parsing, formatting, character iteration, regex.

use regex::Regex;
use std::fmt;

// ============================================================================
// Example: String vs &str
// ============================================================================

fn string_vs_str() {
    // &str: Borrowed string slice
    let literal: &str = "Hello, world!";
    println!("String literal (&str): {}", literal);

    // String: Owned, growable
    let mut owned = String::from("Hello");
    let owned2 = "Hello".to_string();
    let owned3 = String::new();

    println!("String::from: {}", owned);
    println!("to_string(): {}", owned2);
    println!("String::new(): '{}'", owned3);

    // Converting between
    let s = String::from("hello");
    let slice: &str = &s;
    let slice2: &str = s.as_str();
    println!("String to &str: {}, {}", slice, slice2);

    let owned_again: String = slice.to_string();
    println!("&str to String: {}", owned_again);

    // Mutating String
    owned.push_str(", world!");
    println!("After push_str: {}", owned);
}

// ============================================================================
// Example: String Creation and Manipulation
// ============================================================================

fn string_manipulation() {
    // Creating strings
    let s1 = String::new();
    let s2 = String::from("hello");
    let s3 = "hello".to_string();
    let s4 = String::with_capacity(100);

    println!("new: '{}', from: '{}', to_string: '{}', with_capacity: len={}",
             s1, s2, s3, s4.len());

    // Appending text
    let mut s = String::from("Hello");
    s.push_str(", world");
    s.push('!');
    println!("After push_str and push: {}", s);

    // Concatenation with +
    let s1 = String::from("Hello");
    let s2 = String::from(" world");
    let s3 = s1 + &s2; // s1 is moved!
    println!("Concatenated with +: {}", s3);

    // Concatenation with format! (doesn't take ownership)
    let s1 = String::from("Hello");
    let s2 = String::from(" world");
    let s3 = format!("{}{}", s1, s2);
    let s4 = format!("{s1}{s2}");
    println!("format!: {}, {}", s3, s4);

    // Inserting and removing
    let mut s = String::from("Hello world");
    s.insert(5, ',');
    s.insert_str(6, " beautiful");
    println!("After insert: {}", s);

    s.truncate(5);
    println!("After truncate(5): {}", s);

    // Replacing text
    let s = "I like cats";
    let s2 = s.replace("cats", "dogs");
    println!("replace: {} -> {}", s, s2);

    let s = "aaabbbccc";
    let s2 = s.replacen("a", "x", 2);
    println!("replacen (first 2): {} -> {}", s, s2);
}

// ============================================================================
// Example: String Inspection and Searching
// ============================================================================

fn string_searching() {
    let text = "Hello, world!";

    // Basic properties
    println!("len (bytes): {}", text.len());
    println!("is_empty: {}", text.is_empty());

    // Checking contents
    println!("starts_with 'Hello': {}", text.starts_with("Hello"));
    println!("ends_with '!': {}", text.ends_with("!"));
    println!("contains 'world': {}", text.contains("world"));

    // Finding patterns
    println!("find 'world': {:?}", text.find("world"));
    println!("find 'w': {:?}", text.find('w'));
    println!("rfind 'o': {:?}", text.rfind('o'));

    // Splitting
    let parts: Vec<&str> = "a,b,c,d".split(',').collect();
    println!("split by ',': {:?}", parts);

    let parts: Vec<&str> = "a::b::c".split("::").collect();
    println!("split by '::': {:?}", parts);

    let parts: Vec<&str> = "  a  b  c  ".split_whitespace().collect();
    println!("split_whitespace: {:?}", parts);

    if let Some((left, right)) = "key=value".split_once('=') {
        println!("split_once: '{}', '{}'", left, right);
    }

    let lines: Vec<&str> = "line1\nline2\nline3".lines().collect();
    println!("lines: {:?}", lines);

    // Trimming whitespace
    let trimmed = "  hello  ".trim();
    let left = "  hello  ".trim_start();
    let right = "  hello  ".trim_end();
    println!("trim: '{}', trim_start: '{}', trim_end: '{}'", trimmed, left, right);

    let custom = "###hello###".trim_matches('#');
    println!("trim_matches '#': {}", custom);
}

// ============================================================================
// Example: Character Iteration
// ============================================================================

fn character_iteration() {
    let text = "Hello 世界";

    // Iterate over chars
    print!("chars: ");
    for c in text.chars() {
        print!("{} ", c);
    }
    println!();

    // Iterate over bytes
    print!("bytes: ");
    for b in text.bytes() {
        print!("{} ", b);
    }
    println!();

    // Get char at position (expensive!)
    let third_char = text.chars().nth(2);
    println!("Third char: {:?}", third_char);

    // Count characters vs bytes
    let char_count = text.chars().count();
    let byte_count = text.len();
    println!("Char count: {}, Byte count: {}", char_count, byte_count);

    // Character predicates
    let all_alpha = "hello".chars().all(|c| c.is_alphabetic());
    let has_digit = "hello123".chars().any(|c| c.is_numeric());
    println!("all alphabetic: {}, has digit: {}", all_alpha, has_digit);
}

// ============================================================================
// Example: String Slicing (Use with Caution!)
// ============================================================================

fn string_slicing() {
    let s = "Hello, 世界";

    // Slicing at valid UTF-8 boundaries
    let slice = &s[0..5];
    println!("Slice [0..5]: {}", slice);

    // Safe slicing with get
    let safe = s.get(0..5);
    let bad = s.get(0..8);
    println!("get(0..5): {:?}", safe);
    println!("get(0..8) (invalid): {:?}", bad);

    // Finding character boundaries
    if s.is_char_boundary(5) {
        println!("Index 5 is a valid char boundary");
    }
}

// ============================================================================
// Example: Parsing and Formatting
// ============================================================================

struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

fn parsing_and_formatting() {
    // Parsing from strings
    let num: i32 = "42".parse().unwrap();
    let float: f64 = "3.14".parse().unwrap();
    let boolean: bool = "true".parse().unwrap();
    println!("Parsed: int={}, float={}, bool={}", num, float, boolean);

    let bad_parse: Result<i32, _> = "not a number".parse();
    println!("Bad parse: {:?}", bad_parse);

    // Formatting with format!
    let name = "Alice";
    let age = 30;
    let msg = format!("Name: {}, Age: {}", name, age);
    let msg2 = format!("Name: {name}, Age: {age}");
    println!("{}", msg);
    println!("{}", msg2);

    // Format specifiers
    println!("Right align: '{:>10}'", "right");
    println!("Left align:  '{:<10}'", "left");
    println!("Center:      '{:^10}'", "center");
    println!("Zero pad:    '{:0>5}'", "42");

    println!("2 decimals: {:.2}", 3.14159);
    println!("Scientific: {:e}", 1000.0);
    println!("Hex:        {:#x}", 255);
    println!("Binary:     {:#b}", 10);

    // Custom Display
    let p = Point { x: 10, y: 20 };
    println!("Point: {}", p);
}

// ============================================================================
// Example: Case Conversion
// ============================================================================

fn case_conversion() {
    let s = "Hello, World!";

    println!("lowercase: {}", s.to_lowercase());
    println!("uppercase: {}", s.to_uppercase());

    // Unicode-aware
    let german = "Straße";
    println!("German uppercase: {} -> {}", german, german.to_uppercase());
}

// ============================================================================
// Example: Regular Expressions (regex crate)
// ============================================================================

fn regex_examples() {
    // Creating and matching
    let re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
    let is_match = re.is_match("2024-01-15");
    println!("Date pattern match: {}", is_match);

    // Capturing groups
    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    let text = "Date: 2024-01-15";

    if let Some(caps) = re.captures(text) {
        let year = &caps[1];
        let month = &caps[2];
        let day = &caps[3];
        println!("Captured: year={}, month={}, day={}", year, month, day);
    }

    // Finding all matches
    let re = Regex::new(r"\d+").unwrap();
    print!("All numbers: ");
    for mat in re.find_iter("Numbers: 42, 100, 7") {
        print!("{} ", mat.as_str());
    }
    println!();

    // Replacing text
    let re = Regex::new(r"\d+").unwrap();
    let result = re.replace_all("Id: 123, Code: 456", "XXX");
    println!("Replaced: {}", result);
}

// ============================================================================
// Example: Common String Patterns
// ============================================================================

fn common_patterns() {
    // Joining strings
    let words = vec!["Hello", "world"];
    let sentence = words.join(" ");
    println!("join: {}", sentence);

    let numbers = vec![1, 2, 3];
    let csv = numbers
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    println!("CSV: {}", csv);

    // Repeating strings
    let repeated = "abc".repeat(3);
    println!("repeat: {}", repeated);

    // Escaping
    let with_newlines = "Line 1\nLine 2";
    let escaped = with_newlines.escape_default().to_string();
    println!("escaped: {}", escaped);

    // Building strings efficiently
    let mut s = String::with_capacity(50);
    for i in 0..5 {
        s.push_str(&i.to_string());
        s.push(' ');
    }
    println!("Built: {}", s);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_creation() {
        let s1 = String::from("hello");
        let s2 = "hello".to_string();
        let s3: String = "hello".into();

        assert_eq!(s1, s2);
        assert_eq!(s2, s3);
    }

    #[test]
    fn test_string_manipulation() {
        let mut s = String::from("Hello");
        s.push_str(", world");
        s.push('!');
        assert_eq!(s, "Hello, world!");
    }

    #[test]
    fn test_string_replace() {
        let s = "Hello, Hello!";
        assert_eq!(s.replace("Hello", "Hi"), "Hi, Hi!");
        assert_eq!(s.replacen("Hello", "Hi", 1), "Hi, Hello!");
    }

    #[test]
    fn test_string_split() {
        let parts: Vec<&str> = "a,b,c".split(',').collect();
        assert_eq!(parts, vec!["a", "b", "c"]);

        let (left, right) = "key=value".split_once('=').unwrap();
        assert_eq!(left, "key");
        assert_eq!(right, "value");
    }

    #[test]
    fn test_string_trim() {
        assert_eq!("  hello  ".trim(), "hello");
        assert_eq!("  hello  ".trim_start(), "hello  ");
        assert_eq!("  hello  ".trim_end(), "  hello");
        assert_eq!("##hello##".trim_matches('#'), "hello");
    }

    #[test]
    fn test_char_count_vs_len() {
        let s = "Hello 世界";
        assert_eq!(s.chars().count(), 8);
        assert_eq!(s.len(), 12); // Bytes, not chars
    }

    #[test]
    fn test_string_parsing() {
        let num: i32 = "42".parse().unwrap();
        assert_eq!(num, 42);

        let result: Result<i32, _> = "not a number".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_format_specifiers() {
        assert_eq!(format!("{:>5}", "ab"), "   ab");
        assert_eq!(format!("{:<5}", "ab"), "ab   ");
        assert_eq!(format!("{:.2}", 3.14159), "3.14");
        assert_eq!(format!("{:#x}", 255), "0xff");
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!("Hello".to_lowercase(), "hello");
        assert_eq!("Hello".to_uppercase(), "HELLO");
    }

    #[test]
    fn test_regex_match() {
        let re = Regex::new(r"\d+").unwrap();
        assert!(re.is_match("abc123"));
        assert!(!re.is_match("abc"));
    }

    #[test]
    fn test_regex_capture() {
        let re = Regex::new(r"(\w+)@(\w+)\.(\w+)").unwrap();
        let caps = re.captures("user@example.com").unwrap();
        assert_eq!(&caps[1], "user");
        assert_eq!(&caps[2], "example");
        assert_eq!(&caps[3], "com");
    }

    #[test]
    fn test_regex_replace() {
        let re = Regex::new(r"\d+").unwrap();
        let result = re.replace_all("a1b2c3", "X");
        assert_eq!(result, "aXbXcX");
    }

    #[test]
    fn test_join() {
        let words = vec!["a", "b", "c"];
        assert_eq!(words.join(","), "a,b,c");
    }

    #[test]
    fn test_repeat() {
        assert_eq!("ab".repeat(3), "ababab");
    }

    #[test]
    fn test_safe_slicing() {
        let s = "Hello, 世界";
        assert_eq!(s.get(0..5), Some("Hello"));
        assert_eq!(s.get(0..8), None); // Invalid boundary
    }
}

fn main() {
    println!("Pattern 5: String and Text Processing");
    println!("======================================\n");

    println!("=== String vs &str ===");
    string_vs_str();
    println!();

    println!("=== String Manipulation ===");
    string_manipulation();
    println!();

    println!("=== String Searching ===");
    string_searching();
    println!();

    println!("=== Character Iteration ===");
    character_iteration();
    println!();

    println!("=== String Slicing ===");
    string_slicing();
    println!();

    println!("=== Parsing and Formatting ===");
    parsing_and_formatting();
    println!();

    println!("=== Case Conversion ===");
    case_conversion();
    println!();

    println!("=== Regular Expressions ===");
    regex_examples();
    println!();

    println!("=== Common Patterns ===");
    common_patterns();
}
