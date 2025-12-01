### String Cheat Sheet
```rust
// ===== CREATING STRINGS =====
// Empty string
let s = String::new();                              // Empty String
let s = String::default();                          // Default (empty)
let s = "".to_string();                             // From string literal

// From string literal
let s = String::from("hello");                      // String::from
let s = "hello".to_string();                        // to_string()
let s = "hello".to_owned();                         // to_owned()

// With capacity
let mut s = String::with_capacity(10);              // Pre-allocate

// From char
let s = String::from('x');                          // From single char
let s = 'x'.to_string();

// From bytes
let bytes = vec![72, 101, 108, 108, 111];
let s = String::from_utf8(bytes).unwrap();          // From Vec<u8>
let s = String::from_utf8_lossy(&bytes);           // Lossy conversion (Cow)
let s = unsafe { String::from_utf8_unchecked(bytes) }; // Unsafe (no validation)

// From iterator
let s: String = ['h', 'e', 'l', 'l', 'o'].iter().collect();
let s: String = "hello".chars().collect();

// Format macro
let s = format!("Hello, {}!", "world");             // Format string
let s = format!("{} + {} = {}", 1, 2, 3);          // With multiple values

// ===== STRING CAPACITY =====
let mut s = String::with_capacity(10);
s.len()                                             // Length in bytes (not chars!)
s.capacity()                                        // Allocated capacity
s.is_empty()                                        // Check if empty

s.reserve(20)                                       // Reserve additional space
s.reserve_exact(20)                                 // Reserve exact space
s.shrink_to_fit()                                   // Reduce capacity to len
s.shrink_to(5)                                      // Shrink to at least n capacity

// ===== APPENDING =====
let mut s = String::from("hello");

s.push('!')                                         // Append char
s.push_str(" world")                                // Append &str
s += " more"                                        // += operator (same as push_str)

s.extend(['a', 'b', 'c'])                          // Extend from chars
s.extend(" text".chars())                          // Extend from iterator

s.insert(0, 'H')                                    // Insert char at byte index
s.insert_str(5, " there")                          // Insert &str at byte index

// ===== CONCATENATION =====
let s1 = String::from("Hello");
let s2 = String::from(" world");

let s3 = s1 + &s2;                                  // s1 moved, s2 borrowed
// println!("{}", s1);                              // ERROR: s1 moved

let s1 = String::from("Hello");
let s3 = s1.clone() + " world";                    // Clone to avoid move

let s = format!("{} {}", "Hello", "world");        // Format (no move)
let s = ["Hello", " ", "world"].concat();          // Concat slice
let s = ["Hello", " ", "world"].join("");          // Join with separator

// ===== REMOVING =====
let mut s = String::from("hello world");

s.pop()                                             // Remove last char: Option<char>
s.remove(0)                                         // Remove char at byte index: char
s.truncate(5)                                       // Keep first n bytes
s.clear()                                           // Remove all chars

// Drain - remove range and return iterator
let drained: String = s.drain(0..5).collect();     // Remove range
let all: String = s.drain(..).collect();           // Remove all (empties string)

// Replace
s.replace_range(0..5, "bye")                       // Replace range with &str

// Retain
s.retain(|c| c != 'o')                             // Keep chars matching predicate

// ===== ACCESSING CHARACTERS =====
let s = String::from("hello");

// Cannot index: s[0]                               // ERROR: strings not indexable

s.chars()                                           // Iterator over chars
s.chars().nth(0)                                    // Get nth char: Option<char>
s.chars().next()                                    // First char: Option<char>
s.chars().last()                                    // Last char: Option<char>

// Byte access
s.bytes()                                           // Iterator over bytes
s.as_bytes()                                        // &[u8] slice
s.as_bytes()[0]                                     // Index bytes (not chars!)

// ===== SLICING =====
let s = String::from("hello");

&s[..]                                              // Full slice: &str
&s[0..2]                                            // Byte range: "he"
&s[..3]                                             // First 3 bytes: "hel"
&s[2..]                                             // From byte 2: "llo"

// Be careful with multi-byte chars!
let s = "hello 世界";
// &s[0..7]                                         // May panic if cuts char boundary

s.get(0..5)                                         // Safe slice: Option<&str>
s.get(0..100)                                       // Returns None if out of bounds

// ===== ITERATION =====
let s = String::from("hello");

// Iterate over chars
for c in s.chars() {
    println!("{}", c);
}

// Iterate over bytes
for b in s.bytes() {
    println!("{}", b);
}

// Iterate with char indices
for (i, c) in s.char_indices() {
    println!("{}: {}", i, c);                      // i is byte index
}

// Iterate over lines
let s = "line1\nline2\nline3";
for line in s.lines() {
    println!("{}", line);
}

// Split whitespace
for word in s.split_whitespace() {
    println!("{}", word);
}

// ===== SEARCHING =====
let s = String::from("hello world");

s.contains("world")                                 // Check if contains substring
s.starts_with("hello")                             // Check prefix
s.ends_with("world")                               // Check suffix

s.find("world")                                     // Find first: Option<usize> (byte index)
s.rfind("o")                                        // Find last: Option<usize>

// Find with predicate
s.find(|c: char| c.is_numeric())                   // First matching char

// Match indices
s.match_indices("l")                                // Iterator over (index, match)
s.rmatch_indices("l")                              // From right

// ===== SPLITTING =====
let s = "hello,world,rust";

s.split(',')                                        // Split by char: Iterator<&str>
s.split(',').collect::<Vec<_>>()                   // Collect splits

s.splitn(2, ',')                                    // Split n times
s.rsplitn(2, ',')                                   // Split n times from right
s.split_terminator(',')                            // Split, ignore trailing empty
s.rsplit(',')                                       // Split from right

// Split with predicate
s.split(|c: char| c == ',' || c == ';')            // Split by multiple delimiters

// Split whitespace
s.split_whitespace()                                // Split by any whitespace
s.split_ascii_whitespace()                         // Split by ASCII whitespace

// Lines
let s = "line1\nline2\r\nline3";
s.lines()                                           // Iterator over lines
s.split('\n')                                       // Split by newline (keeps \r)

// ===== REPLACING =====
let s = String::from("hello world");

s.replace("world", "rust")                          // Replace all: String
s.replacen("l", "L", 2)                            // Replace first n: String
s.replace('o', "0")                                // Replace char

// Replace with closure (requires regex or manual)
s.chars()
    .map(|c| if c == 'o' { '0' } else { c })
    .collect::<String>()

// ===== TRIMMING =====
let s = "  hello world  \n";

s.trim()                                            // Trim both ends: &str
s.trim_start()                                      // Trim start
s.trim_end()                                        // Trim end
s.trim_left()                                       // Deprecated: use trim_start
s.trim_right()                                      // Deprecated: use trim_end

// Trim specific chars
s.trim_matches(' ')                                 // Trim specific char
s.trim_matches(&[' ', '\n'][..])                   // Trim multiple chars
s.trim_start_matches("hello")                      // Trim prefix
s.trim_end_matches("world")                        // Trim suffix

// Strip prefix/suffix (returns Option)
s.strip_prefix("hello")                             // Remove prefix: Option<&str>
s.strip_suffix("world")                            // Remove suffix: Option<&str>

// ===== CASE CONVERSION =====
let s = String::from("Hello World");

s.to_lowercase()                                    // Convert to lowercase: String
s.to_uppercase()                                    // Convert to uppercase: String
s.to_ascii_lowercase()                             // ASCII lowercase: String
s.to_ascii_uppercase()                             // ASCII uppercase: String

s.make_ascii_lowercase()                           // Mutate to ASCII lowercase
s.make_ascii_uppercase()                           // Mutate to ASCII uppercase

// ===== CHECKING PROPERTIES =====
let s = "hello123";

s.is_empty()                                        // Check if empty
s.is_ascii()                                        // Check if all ASCII
s.chars().all(|c| c.is_numeric())                 // Check if all numeric
s.chars().all(|c| c.is_alphabetic())              // Check if all alphabetic
s.chars().all(|c| c.is_alphanumeric())            // Check if all alphanumeric
s.chars().all(|c| c.is_lowercase())               // Check if all lowercase
s.chars().all(|c| c.is_uppercase())               // Check if all uppercase
s.chars().all(|c| c.is_whitespace())              // Check if all whitespace

// ===== PARSING =====
let s = "42";

s.parse::<i32>()                                    // Parse to i32: Result<i32, _>
s.parse::<i32>().unwrap()                          // Unwrap result
s.parse::<f64>()                                    // Parse to f64

"true".parse::<bool>()                             // Parse bool
"[1,2,3]".parse::<Vec<i32>>()                     // Parse complex types (with serde)

// ===== REPEATING =====
let s = "ha";
s.repeat(3)                                         // "hahaha"

// ===== PADDING =====
// Left pad (requires external crate or manual)
format!("{:>10}", "hello")                          // Right-align in 10 chars
format!("{:<10}", "hello")                          // Left-align in 10 chars
format!("{:^10}", "hello")                          // Center in 10 chars
format!("{:0>5}", "42")                            // Pad with zeros: "00042"

// ===== ESCAPING =====
let s = r"C:\Users\name";                          // Raw string (no escapes)
let s = r#"He said "hello""#;                      // Raw string with quotes
let s = "Line 1\nLine 2";                          // With newline
let s = "Tab\there";                               // With tab

// Escape HTML/URL (requires external crate)

// ===== MULTI-LINE STRINGS =====
let s = "line 1
line 2
line 3";

let s = "line 1\n\
        line 2\n\
        line 3";                                    // Backslash continues line

let s = r"
    line 1
    line 2
";                                                  // Raw multi-line

// ===== COMPARING =====
let s1 = String::from("hello");
let s2 = String::from("hello");

s1 == s2                                            // Equality
s1 != s2                                            // Inequality
s1 < s2                                             // Lexicographic comparison
s1.cmp(&s2)                                         // Ordering

s1.eq_ignore_ascii_case(&s2)                       // Case-insensitive compare (ASCII)

// ===== STRING VS &str =====
let s: String = String::from("hello");             // Owned String
let slice: &str = "hello";                         // String slice

let slice: &str = &s;                              // Borrow String as &str
let slice: &str = &s[..];                          // Explicit slice
let slice: &str = s.as_str();                      // as_str()

let owned: String = slice.to_string();             // &str to String
let owned: String = slice.to_owned();              // to_owned()
let owned: String = String::from(slice);           // String::from

// ===== RAW PARTS =====
let mut s = String::from("hello");

s.as_ptr()                                          // Get raw pointer: *const u8
s.as_mut_ptr()                                      // Get mutable raw pointer: *mut u8
s.as_bytes()                                        // Get bytes: &[u8]
s.as_bytes_mut()                                    // ERROR: String not as_bytes_mut
unsafe { s.as_mut_vec() }                          // Get Vec<u8> (unsafe)

// Construct from raw parts (unsafe)
let bytes = vec![72, 101, 108, 108, 111];
let s = unsafe { String::from_utf8_unchecked(bytes) };

// Into bytes
let bytes = s.into_bytes();                        // Consume String, get Vec<u8>

// ===== COMMON PATTERNS =====
// Pattern 1: Build string from parts
let mut s = String::new();
s.push_str("Hello");
s.push(' ');
s.push_str("world");

// Pattern 2: Collect from iterator
let s: String = vec!["hello", " ", "world"]
    .into_iter()
    .collect();

// Pattern 3: Filter characters
let s = "hello123"
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect::<String>();

// Pattern 4: Reverse string
let reversed: String = "hello".chars().rev().collect();

// Pattern 5: Remove whitespace
let no_space: String = "h e l l o"
    .chars()
    .filter(|c| !c.is_whitespace())
    .collect();

// Pattern 6: Split and process
let parts: Vec<i32> = "1,2,3,4"
    .split(',')
    .filter_map(|s| s.parse().ok())
    .collect();

// Pattern 7: Join with separator
let words = vec!["hello", "world", "rust"];
let joined = words.join(" ");                      // "hello world rust"

// Pattern 8: Title case (simple)
let title: String = "hello world"
    .split_whitespace()
    .map(|word| {
        let mut chars = word.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() 
                + chars.as_str(),
        }
    })
    .collect::<Vec<_>>()
    .join(" ");

// Pattern 9: Check for substring ignoring case
let contains = "Hello World"
    .to_lowercase()
    .contains(&"world".to_lowercase());

// Pattern 10: Truncate with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// Pattern 11: Remove duplicates
let unique: String = "hello"
    .chars()
    .fold(String::new(), |mut acc, c| {
        if !acc.contains(c) {
            acc.push(c);
        }
        acc
    });

// Pattern 12: Word count
let word_count = "hello world rust".split_whitespace().count();

// Pattern 13: Palindrome check
let is_palindrome = |s: &str| {
    let clean: String = s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect();
    clean == clean.chars().rev().collect::<String>()
};

// Pattern 14: Extract numbers
let numbers: Vec<i32> = "abc123def456"
    .chars()
    .filter(|c| c.is_numeric())
    .collect::<String>()
    .chars()
    .map(|c| c.to_digit(10).unwrap() as i32)
    .collect();

// Pattern 15: Safe substring
fn safe_substring(s: &str, start: usize, end: usize) -> &str {
    let indices: Vec<_> = s.char_indices().map(|(i, _)| i).collect();
    let start_byte = indices.get(start).copied().unwrap_or(s.len());
    let end_byte = indices.get(end).copied().unwrap_or(s.len());
    &s[start_byte..end_byte]
}

// Pattern 16: Indent lines
fn indent(s: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    s.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}

// Pattern 17: Remove ANSI codes (simple)
fn strip_ansi(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .collect()
}
```
