# String Operations Cookbook

## Type System Overview

```rust
// Core types
String              // Vec<u8> wrapper, owned, growable, UTF-8 validated
&str               // Fat pointer: (*const u8, usize), borrowed, UTF-8
Cow<'a, str>       // Borrowed | Owned, lazy cloning
Box<str>           // Sized str on heap, no capacity overhead
Arc<str>/Rc<str>   // Shared ownership, zero-copy clone

// System interop
OsString/&OsStr    // Platform strings, no UTF-8 guarantee
PathBuf/&Path      // Typed wrappers over OsString/OsStr
CString/&CStr      // Null-terminated, FFI-safe
```

## Pattern: Zero-Copy String Processing

```rust
use std::borrow::Cow;

// Return borrowed when possible, owned only when necessary
fn normalize_path(path: &str) -> Cow<str> {
    if path.starts_with("~/") {
        Cow::Owned(path.replacen("~", "/home/user", 1))
    } else {
        Cow::Borrowed(path)
    }
}

// Chain transformations, delay allocation
fn process<'a>(s: &'a str) -> Cow<'a, str> {
    let mut result = Cow::Borrowed(s);

    if s.contains('\t') {
        result = Cow::Owned(result.replace('\t', "    "));
    }
    if result.len() > 100 {
        result.to_mut().truncate(100);
    }

    result
}
```

## Pattern: Builder with Capacity Pre-allocation

```rust
// Avoid repeated reallocations
fn build_sql_query(table: &str, filters: &[(&str, &str)]) -> String {
    let capacity = 50 + table.len() + filters.len() * 30;
    let mut query = String::with_capacity(capacity);

    query.push_str("SELECT * FROM ");
    query.push_str(table);

    if !filters.is_empty() {
        query.push_str(" WHERE ");
        for (i, (col, val)) in filters.iter().enumerate() {
            if i > 0 { query.push_str(" AND "); }
            query.push_str(col);
            query.push_str(" = '");
            query.push_str(val);
            query.push('\'');
        }
    }

    query
}
```

## Pattern: String Interning

```rust
use std::sync::Arc;
use std::collections::HashMap;

struct StringPool {
    pool: HashMap<Arc<str>, ()>,
}

impl StringPool {
    fn intern(&mut self, s: &str) -> Arc<str> {
        self.pool.keys()
            .find(|k| k.as_ref() == s)
            .cloned()
            .unwrap_or_else(|| {
                let arc: Arc<str> = Arc::from(s);
                self.pool.insert(arc.clone(), ());
                arc
            })
    }
}

// Use case: AST nodes, tokens, identifiers
struct Token {
    kind: TokenKind,
    text: Arc<str>,  // Shared across many tokens
}
```

## Pattern: Efficient String Joining

```rust
// Iterator-based joining (no intermediate Vec)
fn join_iter<'a, I>(iter: I, sep: &str) -> String
where
    I: Iterator<Item = &'a str>,
{
    let mut iter = iter.peekable();
    let mut result = String::new();

    while let Some(s) = iter.next() {
        result.push_str(s);
        if iter.peek().is_some() {
            result.push_str(sep);
        }
    }

    result
}

// Or use standard library
vec!["a", "b", "c"].join(",")
```

## Pattern: Safe UTF-8 Slicing

```rust
// Byte-based indexing can panic on invalid boundaries
fn safe_slice(s: &str, start: usize, end: usize) -> Option<&str> {
    if s.is_char_boundary(start) && s.is_char_boundary(end) {
        Some(&s[start..end])
    } else {
        None
    }
}

// Grapheme-aware truncation (requires unicode-segmentation crate)
use unicode_segmentation::UnicodeSegmentation;

fn truncate_graphemes(s: &str, max: usize) -> String {
    s.graphemes(true)
        .take(max)
        .collect()
}
```

## Pattern: Lazy String Transformations

```rust
// Chain iterators, avoid intermediate allocations
fn transform(s: &str) -> String {
    s.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

// Or with custom iterator
struct TrimLines<'a> {
    iter: std::str::Lines<'a>,
}

impl<'a> Iterator for TrimLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.find(|line| !line.trim().is_empty())
            .map(|line| line.trim())
    }
}
```

## Pattern: Type-Safe String Wrappers

```rust
// Newtype pattern for semantic clarity
struct UserId(String);
struct Email(String);

impl UserId {
    fn new(id: String) -> Result<Self, &'static str> {
        if id.len() < 3 {
            Err("User ID too short")
        } else {
            Ok(UserId(id))
        }
    }
}

impl Email {
    fn new(email: String) -> Result<Self, &'static str> {
        if email.contains('@') {
            Ok(Email(email))
        } else {
            Err("Invalid email")
        }
    }
}
```

## Pattern: String Parsing State Machine

```rust
enum State { Start, InQuote, InEscape, Done }

fn parse_quoted_string(input: &str) -> Result<String, &'static str> {
    let mut state = State::Start;
    let mut result = String::new();
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        state = match (state, ch) {
            (State::Start, '"') => State::InQuote,
            (State::Start, _) => return Err("Expected quote"),

            (State::InQuote, '\\') => State::InEscape,
            (State::InQuote, '"') => State::Done,
            (State::InQuote, c) => { result.push(c); State::InQuote }

            (State::InEscape, 'n') => { result.push('\n'); State::InQuote }
            (State::InEscape, 't') => { result.push('\t'); State::InQuote }
            (State::InEscape, c) => { result.push(c); State::InQuote }

            (State::Done, _) => return Err("Unexpected character after quote"),
        };
    }

    match state {
        State::Done => Ok(result),
        _ => Err("Unterminated string"),
    }
}
```

## Pattern: Rope Data Structure (Large Text)

```rust
// For editors, large documents where frequent insertion/deletion occurs
enum Rope {
    Leaf(String),
    Node { left: Box<Rope>, right: Box<Rope>, len: usize },
}

impl Rope {
    fn len(&self) -> usize {
        match self {
            Rope::Leaf(s) => s.len(),
            Rope::Node { len, .. } => *len,
        }
    }

    fn concat(left: Rope, right: Rope) -> Self {
        let len = left.len() + right.len();
        Rope::Node { left: Box::new(left), right: Box::new(right), len }
    }

    fn split_at(self, pos: usize) -> (Rope, Rope) {
        match self {
            Rope::Leaf(s) => {
                let (l, r) = s.split_at(pos);
                (Rope::Leaf(l.to_string()), Rope::Leaf(r.to_string()))
            }
            Rope::Node { left, right, .. } => {
                let left_len = left.len();
                if pos < left_len {
                    let (l, r) = left.split_at(pos);
                    (l, Rope::concat(r, *right))
                } else {
                    let (l, r) = right.split_at(pos - left_len);
                    (Rope::concat(*left, l), r)
                }
            }
        }
    }
}
```

## Pattern: Format String Optimization

```rust
// Avoid format! when unnecessary
let s = format!("{}", x);          // ❌ Allocates
let s = x.to_string();             // ✓ More direct

// Reuse buffer
let mut buf = String::new();
for i in 0..1000 {
    use std::fmt::Write;
    buf.clear();
    write!(&mut buf, "Item {}", i).unwrap();
    process(&buf);
}

// Or use format_args! for zero-allocation formatting
fn log(args: std::fmt::Arguments) {
    println!("{}", args);
}
log(format_args!("Value: {}", x));
```

## Pattern: String Validation Pipeline

```rust
struct Validator<'a> {
    value: &'a str,
}

impl<'a> Validator<'a> {
    fn new(value: &'a str) -> Self { Self { value } }

    fn min_length(self, min: usize) -> Result<Self, &'static str> {
        if self.value.len() >= min { Ok(self) } else { Err("Too short") }
    }

    fn max_length(self, max: usize) -> Result<Self, &'static str> {
        if self.value.len() <= max { Ok(self) } else { Err("Too long") }
    }

    fn matches_pattern(self, pattern: &str) -> Result<Self, &'static str> {
        if self.value.contains(pattern) { Ok(self) } else { Err("Pattern not found") }
    }

    fn validate(self) -> &'a str { self.value }
}

// Usage
let email = Validator::new(input)
    .min_length(5)?
    .max_length(100)?
    .matches_pattern("@")?
    .validate();
```

## Pattern: Streaming String Processing

```rust
use std::io::{BufRead, BufReader};
use std::fs::File;

// Process large files line-by-line without loading into memory
fn count_pattern(path: &str, pattern: &str) -> std::io::Result<usize> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines()
        .filter_map(Result::ok)
        .filter(|line| line.contains(pattern))
        .count())
}
```

## Pattern: Efficient String Comparison

```rust
// Use byte comparison for ASCII
fn ascii_eq_ignore_case(a: &str, b: &str) -> bool {
    a.len() == b.len() &&
    a.bytes()
        .zip(b.bytes())
        .all(|(a, b)| a.eq_ignore_ascii_case(&b))
}

// Use memchr for single character search
use memchr::memchr;

fn find_delimiter(s: &str, delim: u8) -> Option<usize> {
    memchr(delim, s.as_bytes())
}
```

## Pattern: String Deduplication

```rust
use std::collections::HashSet;

fn deduplicate_lines(input: &str) -> String {
    let mut seen = HashSet::new();
    input.lines()
        .filter(|line| seen.insert(*line))
        .collect::<Vec<_>>()
        .join("\n")
}

// With order preservation
fn deduplicate_preserving_order(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items.into_iter()
        .filter(|s| seen.insert(s.clone()))
        .collect()
}
```

## Pattern: Path Manipulation

```rust
use std::path::{Path, PathBuf};

// Canonical path operations
fn normalize_path(path: &Path) -> PathBuf {
    path.components()
        .fold(PathBuf::new(), |mut acc, component| {
            match component {
                std::path::Component::ParentDir => { acc.pop(); }
                std::path::Component::CurDir => {}
                _ => acc.push(component),
            }
            acc
        })
}

// Safe filename extraction
fn get_filename(path: &Path) -> Option<&str> {
    path.file_name()
        .and_then(|os| os.to_str())
}
```

## Pattern: C FFI String Handling

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

// Rust → C
fn rust_to_c(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// C → Rust (unsafe, requires null-terminated pointer)
unsafe fn c_to_rust<'a>(ptr: *const c_char) -> &'a str {
    CStr::from_ptr(ptr).to_str().unwrap()
}

// Free C string created by rust_to_c
unsafe fn free_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}
```

## Performance Cheatsheet

```rust
// ❌ Slow: repeated allocation
let mut s = String::new();
for i in 0..1000 {
    s = format!("{}{}", s, i);  // Allocates each time
}

// ✓ Fast: single buffer
let mut s = String::with_capacity(4000);
for i in 0..1000 {
    use std::fmt::Write;
    write!(&mut s, "{}", i).unwrap();
}

// ❌ Slow: unnecessary cloning
fn process(s: String) { /* ... */ }

// ✓ Fast: borrow when possible
fn process(s: &str) { /* ... */ }

// ❌ Slow: O(n) repeated starts_with
if s.starts_with("http://") || s.starts_with("https://") { /* ... */ }

// ✓ Fast: single scan
if s.len() >= 7 && &s[..7] == "http://" ||
   s.len() >= 8 && &s[..8] == "https://" { /* ... */ }

// Or use aho-corasick for multiple patterns
```

## Type Conversion Reference

```rust
// String ↔ &str
let s = String::from("hello");
let slice: &str = &s;
let owned: String = slice.to_string();

// String ↔ Vec<u8>
let bytes: Vec<u8> = s.into_bytes();
let s: String = String::from_utf8(bytes).unwrap();

// &str ↔ &[u8]
let bytes: &[u8] = s.as_bytes();
let s: &str = std::str::from_utf8(bytes).unwrap();

// String → Box<str> (shrink to fit)
let boxed: Box<str> = s.into_boxed_str();

// String → Arc<str> (shared ownership)
let arc: Arc<str> = Arc::from(s);

// OsString ↔ String (lossy)
use std::ffi::OsString;
let os = OsString::from("path");
let s = os.to_string_lossy().into_owned();

// PathBuf ↔ String (lossy)
use std::path::PathBuf;
let path = PathBuf::from("/tmp");
let s = path.to_string_lossy().into_owned();
```
