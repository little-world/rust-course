# Cookbook: String Processing

> Advanced string manipulation, parsing, and text processing algorithms

## Table of Contents

1. [String Type Overview](#string-type-overview)
2. [Zero-Copy String Operations](#zero-copy-string-operations)
3. [UTF-8 Handling and Validation](#utf-8-handling-and-validation)
4. [String Parsing State Machines](#string-parsing-state-machines)
5. [Text Editor Data Structures](#text-editor-data-structures)
6. [Pattern Matching Algorithms](#pattern-matching-algorithms)
7. [String Interning](#string-interning)
8. [Unicode Operations](#unicode-operations)
9. [Performance Optimizations](#performance-optimizations)
10. [Quick Reference](#quick-reference)

---

## String Type Overview

### Recipe 1: Choosing the Right String Type

**Problem**: Understand when to use each string type.

**Use Case**: API design, cross-platform code, efficient string handling.

**Types**: String, &str, Cow, OsString, Path

```rust
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

// String: Owned, heap-allocated, growable UTF-8 string
fn string_example() {
    let mut s = String::from("Hello");
    s.push_str(", World!");
    println!("{}", s);

    // Use when:
    // - Need to own the string
    // - Building strings dynamically
    // - Returning strings from functions
}

// &str: Borrowed string slice, doesn't own data
fn str_slice_example(s: &str) {
    println!("Length: {}", s.len());

    // Use when:
    // - Read-only access needed
    // - Function parameters (most flexible)
    // - String literals
}

// Cow (Clone on Write): Borrows when possible, owns when necessary
fn cow_example<'a>(data: &'a str, uppercase: bool) -> Cow<'a, str> {
    if uppercase {
        Cow::Owned(data.to_uppercase())  // Allocates
    } else {
        Cow::Borrowed(data)  // No allocation
    }
}

// OsString/OsStr: Platform-native strings (may not be UTF-8)
fn os_string_example() {
    use std::env;

    for (key, value) in env::vars_os() {
        println!("{:?} = {:?}", key, value);
    }

    // Use when:
    // - Dealing with file system
    // - Environment variables
    // - FFI with OS APIs
}

// Path/PathBuf: File system paths
fn path_example() {
    let path = Path::new("/tmp/foo.txt");

    println!("Extension: {:?}", path.extension());
    println!("Parent: {:?}", path.parent());
    println!("File name: {:?}", path.file_name());

    // Building paths
    let mut path_buf = PathBuf::from("/tmp");
    path_buf.push("subdir");
    path_buf.push("file.txt");

    // Use when:
    // - Working with file paths
    // - Cross-platform path manipulation
}

fn main() {
    // Demonstrate type conversions
    let string = String::from("Hello");
    let str_slice: &str = &string;  // String -> &str (deref coercion)
    let cow: Cow<str> = Cow::Borrowed(str_slice);

    // String from &str
    let owned: String = str_slice.to_string();

    // Path conversions
    let path = Path::new("file.txt");
    let os_str: &OsStr = path.as_os_str();

    println!("Type conversions demonstrated");
}
```

**Key Concepts**:
- `String` owns data, `&str` borrows
- `Cow` optimizes by borrowing when possible
- `OsString` handles platform-specific encodings
- `Path` provides platform-independent path operations

---

### Recipe 2: String Builder Pattern

**Problem**: Efficiently build strings from multiple parts.

**Use Case**: Template rendering, HTML generation, log formatting.

**Pattern**: Builder with capacity pre-allocation

```rust
struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    fn new() -> Self {
        StringBuilder {
            buffer: String::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        StringBuilder {
            buffer: String::with_capacity(capacity),
        }
    }

    fn append(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self
    }

    fn append_line(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self.buffer.push('\n');
        self
    }

    fn append_fmt(&mut self, args: std::fmt::Arguments) -> &mut Self {
        use std::fmt::Write;
        let _ = write!(&mut self.buffer, "{}", args);
        self
    }

    fn build(self) -> String {
        self.buffer
    }

    fn as_str(&self) -> &str {
        &self.buffer
    }
}

// HTML builder example
struct HtmlBuilder {
    builder: StringBuilder,
    indent: usize,
}

impl HtmlBuilder {
    fn new() -> Self {
        HtmlBuilder {
            builder: StringBuilder::with_capacity(1024),
            indent: 0,
        }
    }

    fn open_tag(&mut self, tag: &str) -> &mut Self {
        self.write_indent();
        self.builder.append("<").append(tag).append(">\n");
        self.indent += 2;
        self
    }

    fn close_tag(&mut self, tag: &str) -> &mut Self {
        self.indent -= 2;
        self.write_indent();
        self.builder.append("</").append(tag).append(">\n");
        self
    }

    fn content(&mut self, text: &str) -> &mut Self {
        self.write_indent();
        self.builder.append(text).append("\n");
        self
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.builder.append(" ");
        }
    }

    fn build(self) -> String {
        self.builder.build()
    }
}

fn main() {
    // Simple string building
    let mut sb = StringBuilder::with_capacity(100);
    sb.append("Hello")
      .append(", ")
      .append("World")
      .append("!");
    println!("{}", sb.as_str());

    // HTML building
    let mut html = HtmlBuilder::new();
    html.open_tag("html")
        .open_tag("body")
        .open_tag("h1")
        .content("Welcome")
        .close_tag("h1")
        .open_tag("p")
        .content("This is a paragraph")
        .close_tag("p")
        .close_tag("body")
        .close_tag("html");

    println!("{}", html.build());
}
```

**Key Patterns**:
- Pre-allocate capacity when size is known
- Method chaining for fluent API
- Consume builder with `build()` method

---

## Zero-Copy String Operations

### Recipe 3: String Slicing and Splitting

**Problem**: Process strings without allocating new memory.

**Use Case**: Log parsing, CSV processing, token extraction.

**Technique**: Borrowing slices instead of creating owned strings

```rust
// Zero-copy line parser
struct LineParser<'a> {
    data: &'a str,
}

impl<'a> LineParser<'a> {
    fn new(data: &'a str) -> Self {
        LineParser { data }
    }

    // Returns iterator over lines without allocation
    fn lines(&self) -> impl Iterator<Item = &'a str> {
        self.data.lines()
    }

    // Split by delimiter without allocation
    fn split(&self, delimiter: &str) -> impl Iterator<Item = &'a str> {
        self.data.split(delimiter)
    }

    // Extract field by index
    fn field(&self, line: &'a str, index: usize) -> Option<&'a str> {
        line.split(',').nth(index)
    }
}

// CSV parser with zero allocations during parsing
struct CsvParser<'a> {
    data: &'a str,
}

impl<'a> CsvParser<'a> {
    fn new(data: &'a str) -> Self {
        CsvParser { data }
    }

    fn parse(&self) -> Vec<Vec<&'a str>> {
        self.data
            .lines()
            .map(|line| line.split(',').map(|field| field.trim()).collect())
            .collect()
    }

    // Process without intermediate allocations
    fn process<F>(&self, mut f: F)
    where
        F: FnMut(&[&str]),
    {
        for line in self.data.lines() {
            let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
            f(&fields);
        }
    }
}

// String view with bounds checking
struct StringView<'a> {
    data: &'a str,
    start: usize,
    len: usize,
}

impl<'a> StringView<'a> {
    fn new(data: &'a str, start: usize, len: usize) -> Option<Self> {
        if start + len <= data.len() && data.is_char_boundary(start) {
            if start + len == data.len() || data.is_char_boundary(start + len) {
                return Some(StringView { data, start, len });
            }
        }
        None
    }

    fn as_str(&self) -> &'a str {
        &self.data[self.start..self.start + self.len]
    }

    fn slice(&self, start: usize, len: usize) -> Option<StringView<'a>> {
        if start + len <= self.len {
            StringView::new(self.data, self.start + start, len)
        } else {
            None
        }
    }
}

fn main() {
    let data = "name,age,city\nAlice,30,NYC\nBob,25,LA";

    let parser = CsvParser::new(data);
    let rows = parser.parse();

    for row in &rows {
        println!("{:?}", row);
    }

    // Zero-copy processing
    parser.process(|fields| {
        if fields.len() >= 2 {
            println!("Name: {}, Age: {}", fields[0], fields[1]);
        }
    });

    // String view example
    let text = "Hello, World!";
    if let Some(view) = StringView::new(text, 0, 5) {
        println!("View: {}", view.as_str());
    }
}
```

**Key Techniques**:
- Return iterators instead of vectors
- Use string slices (`&str`) as return types
- Leverage `split()`, `lines()` for zero-copy splitting
- Check `is_char_boundary()` before slicing

---

### Recipe 4: Cow for Conditional Allocation

**Problem**: Avoid allocating when input doesn't need modification.

**Use Case**: String normalization, case conversion, escaping.

**Algorithm**: Clone-on-write optimization

```rust
use std::borrow::Cow;

// Normalize whitespace: collapse multiple spaces to one
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

// Escape HTML entities only if needed
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

// Remove prefix/suffix without allocation if not present
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

// Case normalization
fn to_lowercase_if_needed(s: &str) -> Cow<str> {
    if s.chars().all(|c| !c.is_uppercase()) {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_lowercase())
    }
}

fn main() {
    // Whitespace normalization
    let s1 = "hello world";
    let s2 = "hello  world";  // Extra space

    println!("s1: {:?}", normalize_whitespace(s1));  // Borrowed
    println!("s2: {:?}", normalize_whitespace(s2));  // Owned

    // HTML escaping
    let safe = "Hello World";
    let unsafe_text = "Hello <b>World</b>";

    println!("Safe: {:?}", escape_html(safe));      // Borrowed
    println!("Unsafe: {:?}", escape_html(unsafe_text));  // Owned

    // Prefix/suffix stripping
    println!("{:?}", strip_affixes("hello", "", ""));        // Borrowed
    println!("{:?}", strip_affixes("[hello]", "[", "]"));   // Owned

    // Case normalization
    println!("{:?}", to_lowercase_if_needed("hello"));     // Borrowed
    println!("{:?}", to_lowercase_if_needed("Hello"));     // Owned
}
```

**Key Patterns**:
- Check if modification needed before allocating
- Return `Cow::Borrowed` for no-op transformations
- Return `Cow::Owned` only when changes made
- Pre-allocate capacity for owned variants

---

## UTF-8 Handling and Validation

### Recipe 5: UTF-8 Validation and Repair

**Problem**: Handle potentially invalid UTF-8 data.

**Use Case**: Network protocols, file parsing, legacy data.

**Algorithm**: UTF-8 validation and replacement

```rust
// Validate UTF-8 and replace invalid sequences
fn validate_utf8_lossy(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

// Strict validation
fn validate_utf8_strict(data: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(data)
}

// Custom UTF-8 validator with error positions
struct Utf8Validator<'a> {
    data: &'a [u8],
}

impl<'a> Utf8Validator<'a> {
    fn new(data: &'a [u8]) -> Self {
        Utf8Validator { data }
    }

    fn validate(&self) -> Result<&'a str, Utf8Error> {
        let mut pos = 0;

        while pos < self.data.len() {
            match self.decode_char(pos) {
                Ok((_, next_pos)) => pos = next_pos,
                Err(error_pos) => {
                    return Err(Utf8Error {
                        valid_up_to: error_pos,
                        error_len: self.error_length(error_pos),
                    });
                }
            }
        }

        unsafe { Ok(std::str::from_utf8_unchecked(self.data)) }
    }

    fn decode_char(&self, pos: usize) -> Result<(char, usize), usize> {
        if pos >= self.data.len() {
            return Err(pos);
        }

        let first = self.data[pos];

        // 1-byte sequence (ASCII)
        if first < 0x80 {
            return Ok((first as char, pos + 1));
        }

        // 2-byte sequence
        if first & 0xE0 == 0xC0 {
            if pos + 1 >= self.data.len() {
                return Err(pos);
            }
            let second = self.data[pos + 1];
            if second & 0xC0 != 0x80 {
                return Err(pos);
            }
            let ch = ((first as u32 & 0x1F) << 6) | (second as u32 & 0x3F);
            if ch < 0x80 {
                return Err(pos);  // Overlong encoding
            }
            return Ok((char::from_u32(ch).ok_or(pos)?, pos + 2));
        }

        // 3-byte sequence
        if first & 0xF0 == 0xE0 {
            if pos + 2 >= self.data.len() {
                return Err(pos);
            }
            let second = self.data[pos + 1];
            let third = self.data[pos + 2];
            if second & 0xC0 != 0x80 || third & 0xC0 != 0x80 {
                return Err(pos);
            }
            let ch = ((first as u32 & 0x0F) << 12)
                | ((second as u32 & 0x3F) << 6)
                | (third as u32 & 0x3F);
            if ch < 0x800 {
                return Err(pos);  // Overlong encoding
            }
            return Ok((char::from_u32(ch).ok_or(pos)?, pos + 3));
        }

        // 4-byte sequence
        if first & 0xF8 == 0xF0 {
            if pos + 3 >= self.data.len() {
                return Err(pos);
            }
            let bytes = &self.data[pos..pos + 4];
            if bytes[1] & 0xC0 != 0x80
                || bytes[2] & 0xC0 != 0x80
                || bytes[3] & 0xC0 != 0x80
            {
                return Err(pos);
            }
            let ch = ((first as u32 & 0x07) << 18)
                | ((bytes[1] as u32 & 0x3F) << 12)
                | ((bytes[2] as u32 & 0x3F) << 6)
                | (bytes[3] as u32 & 0x3F);
            if ch < 0x10000 || ch > 0x10FFFF {
                return Err(pos);  // Overlong or out of range
            }
            return Ok((char::from_u32(ch).ok_or(pos)?, pos + 4));
        }

        Err(pos)
    }

    fn error_length(&self, pos: usize) -> Option<usize> {
        if pos >= self.data.len() {
            return None;
        }

        let first = self.data[pos];
        if first < 0x80 {
            Some(1)
        } else if first & 0xE0 == 0xC0 {
            Some(2)
        } else if first & 0xF0 == 0xE0 {
            Some(3)
        } else if first & 0xF8 == 0xF0 {
            Some(4)
        } else {
            Some(1)
        }
    }
}

#[derive(Debug)]
struct Utf8Error {
    valid_up_to: usize,
    error_len: Option<usize>,
}

fn main() {
    // Valid UTF-8
    let valid = "Hello, 世界!".as_bytes();
    let validator = Utf8Validator::new(valid);
    assert!(validator.validate().is_ok());

    // Invalid UTF-8
    let invalid = &[0xFF, 0xFE, 0xFD];
    let validator = Utf8Validator::new(invalid);
    match validator.validate() {
        Ok(_) => println!("Valid"),
        Err(e) => println!("Invalid UTF-8 at position {}", e.valid_up_to),
    }

    // Lossy conversion
    let lossy = validate_utf8_lossy(invalid);
    println!("Lossy: {}", lossy);
}
```

**Key Concepts**:
- UTF-8 byte sequence patterns
- Overlong encoding detection
- Surrogate and out-of-range detection
- Lossy vs strict validation

---

### Recipe 6: Character and Grapheme Iteration

**Problem**: Correctly iterate over user-perceived characters.

**Use Case**: Text editing, display width calculation, emoji handling.

**Concept**: Grapheme clusters vs code points vs bytes

```rust
use unicode_segmentation::UnicodeSegmentation;

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
        println!("{}: '{}' (U+{:04X})", i, ch, ch as u32);
    }

    // Grapheme cluster iteration (requires unicode-segmentation crate)
    println!("\nGrapheme clusters:");
    for (i, grapheme) in s.graphemes(true).enumerate() {
        println!("{}: '{}'", i, grapheme);
    }

    println!("\nChar count: {}", s.chars().count());
    println!("Grapheme count: {}", s.graphemes(true).count());
}

// Calculate display width (East Asian Width)
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

// Safe string truncation at character boundary
fn truncate_at_char(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

// Truncate at grapheme boundary
fn truncate_at_grapheme(s: &str, max_graphemes: usize) -> &str {
    s.graphemes(true)
        .take(max_graphemes)
        .collect::<String>()
        .as_str()
}

// Reverse string preserving grapheme clusters
fn reverse_graphemes(s: &str) -> String {
    s.graphemes(true).rev().collect()
}

fn main() {
    // Simple ASCII
    analyze_string("Hello");
    println!("\n{}", "=".repeat(50));

    // Multi-byte UTF-8
    analyze_string("Héllo");
    println!("\n{}", "=".repeat(50));

    // Emoji with modifier
    analyze_string("👨‍👩‍👧‍👦");  // Family emoji
    println!("\n{}", "=".repeat(50));

    // Korean text
    let korean = "안녕하세요";
    println!("Korean: {}", korean);
    println!("Display width: {}", display_width(korean));

    // Truncation
    let text = "Hello, 世界! 👋";
    println!("\nOriginal: {}", text);
    println!("Truncated (5 chars): {}", truncate_at_char(text, 5));

    // Reversal
    let text = "café";
    println!("\nOriginal: {}", text);
    println!("Reversed: {}", reverse_graphemes(text));
}
```

**Key Concepts**:
- Bytes vs characters vs graphemes
- Grapheme clusters for user-perceived characters
- East Asian Width for display
- Safe truncation at boundaries

---

## String Parsing State Machines

### Recipe 7: Lexer with State Machine

**Problem**: Tokenize source code or structured text.

**Use Case**: Compilers, syntax highlighting, linters.

**Algorithm**: Finite state machine lexer

```rust
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Identifier(String),
    Number(f64),
    String(String),
    Operator(String),
    Keyword(String),
    Whitespace,
    Comment(String),
    Invalid(char),
}

#[derive(Debug, PartialEq)]
enum LexerState {
    Start,
    InIdentifier,
    InNumber,
    InString,
    InComment,
    InOperator,
}

struct Lexer {
    input: Vec<char>,
    pos: usize,
    state: LexerState,
    current_token: String,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
            state: LexerState::Start,
            current_token: String::new(),
        }
    }

    fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while self.pos < self.input.len() {
            if let Some(token) = self.next_token() {
                if !matches!(token, Token::Whitespace) {
                    tokens.push(token);
                }
            }
        }

        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        let ch = self.current_char()?;

        match self.state {
            LexerState::Start => self.handle_start(ch),
            LexerState::InIdentifier => self.handle_identifier(ch),
            LexerState::InNumber => self.handle_number(ch),
            LexerState::InString => self.handle_string(ch),
            LexerState::InComment => self.handle_comment(ch),
            LexerState::InOperator => self.handle_operator(ch),
        }
    }

    fn handle_start(&mut self, ch: char) -> Option<Token> {
        match ch {
            c if c.is_whitespace() => {
                self.pos += 1;
                Some(Token::Whitespace)
            }
            c if c.is_alphabetic() || c == '_' => {
                self.state = LexerState::InIdentifier;
                self.current_token.push(c);
                self.pos += 1;
                None
            }
            c if c.is_numeric() => {
                self.state = LexerState::InNumber;
                self.current_token.push(c);
                self.pos += 1;
                None
            }
            '"' => {
                self.state = LexerState::InString;
                self.pos += 1;
                None
            }
            '/' if self.peek() == Some('/') => {
                self.state = LexerState::InComment;
                self.pos += 2;  // Skip //
                None
            }
            c if "+-*/<>=!&|".contains(c) => {
                self.state = LexerState::InOperator;
                self.current_token.push(c);
                self.pos += 1;
                None
            }
            c => {
                self.pos += 1;
                Some(Token::Invalid(c))
            }
        }
    }

    fn handle_identifier(&mut self, ch: char) -> Option<Token> {
        if ch.is_alphanumeric() || ch == '_' {
            self.current_token.push(ch);
            self.pos += 1;
            None
        } else {
            let token = self.finish_identifier();
            self.state = LexerState::Start;
            Some(token)
        }
    }

    fn handle_number(&mut self, ch: char) -> Option<Token> {
        if ch.is_numeric() || ch == '.' {
            self.current_token.push(ch);
            self.pos += 1;
            None
        } else {
            let token = Token::Number(
                self.current_token.parse().unwrap_or(0.0)
            );
            self.current_token.clear();
            self.state = LexerState::Start;
            Some(token)
        }
    }

    fn handle_string(&mut self, ch: char) -> Option<Token> {
        if ch == '"' {
            let token = Token::String(self.current_token.clone());
            self.current_token.clear();
            self.state = LexerState::Start;
            self.pos += 1;
            Some(token)
        } else {
            self.current_token.push(ch);
            self.pos += 1;
            None
        }
    }

    fn handle_comment(&mut self, ch: char) -> Option<Token> {
        if ch == '\n' {
            let token = Token::Comment(self.current_token.clone());
            self.current_token.clear();
            self.state = LexerState::Start;
            Some(token)
        } else {
            self.current_token.push(ch);
            self.pos += 1;
            None
        }
    }

    fn handle_operator(&mut self, ch: char) -> Option<Token> {
        // Multi-char operators: ==, !=, <=, >=, &&, ||
        let two_char = format!("{}{}", self.current_token, ch);
        if matches!(two_char.as_str(), "==" | "!=" | "<=" | ">=" | "&&" | "||") {
            self.current_token = two_char;
            self.pos += 1;
            let token = Token::Operator(self.current_token.clone());
            self.current_token.clear();
            self.state = LexerState::Start;
            Some(token)
        } else {
            let token = Token::Operator(self.current_token.clone());
            self.current_token.clear();
            self.state = LexerState::Start;
            Some(token)
        }
    }

    fn finish_identifier(&mut self) -> Token {
        let keywords = ["if", "else", "while", "for", "return", "fn", "let"];

        let token = if keywords.contains(&self.current_token.as_str()) {
            Token::Keyword(self.current_token.clone())
        } else {
            Token::Identifier(self.current_token.clone())
        };

        self.current_token.clear();
        token
    }

    fn current_char(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos + 1).copied()
    }
}

fn main() {
    let code = r#"
        fn main() {
            let x = 42;
            if x == 42 {
                return x + 10;
            }
        }
        // This is a comment
    "#;

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize();

    for token in tokens {
        println!("{:?}", token);
    }
}
```

**Key Patterns**:
- State machine with explicit states
- Lookahead with `peek()`
- Multi-character token recognition
- Keyword vs identifier discrimination

---

### Recipe 8: URL Parser State Machine

**Problem**: Parse URLs into components.

**Use Case**: Web scraping, URL validation, link processing.

**Algorithm**: RFC 3986 URL parsing

```rust
#[derive(Debug, PartialEq)]
struct Url {
    scheme: String,
    authority: Option<String>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

#[derive(Debug)]
enum ParseState {
    Scheme,
    AfterScheme,
    Authority,
    Path,
    Query,
    Fragment,
}

struct UrlParser {
    input: Vec<char>,
    pos: usize,
    state: ParseState,
}

impl UrlParser {
    fn new(url: &str) -> Self {
        UrlParser {
            input: url.chars().collect(),
            pos: 0,
            state: ParseState::Scheme,
        }
    }

    fn parse(&mut self) -> Result<Url, String> {
        let mut scheme = String::new();
        let mut authority = None;
        let mut path = String::new();
        let mut query = None;
        let mut fragment = None;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];

            match self.state {
                ParseState::Scheme => {
                    if ch == ':' {
                        if scheme.is_empty() {
                            return Err("Empty scheme".to_string());
                        }
                        self.state = ParseState::AfterScheme;
                        self.pos += 1;
                    } else if ch.is_alphanumeric() || ch == '+' || ch == '-' || ch == '.' {
                        scheme.push(ch);
                        self.pos += 1;
                    } else {
                        return Err(format!("Invalid scheme character: {}", ch));
                    }
                }

                ParseState::AfterScheme => {
                    if self.pos + 1 < self.input.len()
                        && self.input[self.pos] == '/'
                        && self.input[self.pos + 1] == '/'
                    {
                        self.state = ParseState::Authority;
                        self.pos += 2;
                    } else {
                        self.state = ParseState::Path;
                    }
                }

                ParseState::Authority => {
                    if ch == '/' {
                        self.state = ParseState::Path;
                    } else if ch == '?' {
                        self.state = ParseState::Query;
                        self.pos += 1;
                    } else if ch == '#' {
                        self.state = ParseState::Fragment;
                        self.pos += 1;
                    } else {
                        if authority.is_none() {
                            authority = Some(String::new());
                        }
                        authority.as_mut().unwrap().push(ch);
                        self.pos += 1;
                    }
                }

                ParseState::Path => {
                    if ch == '?' {
                        self.state = ParseState::Query;
                        self.pos += 1;
                    } else if ch == '#' {
                        self.state = ParseState::Fragment;
                        self.pos += 1;
                    } else {
                        path.push(ch);
                        self.pos += 1;
                    }
                }

                ParseState::Query => {
                    if ch == '#' {
                        self.state = ParseState::Fragment;
                        self.pos += 1;
                    } else {
                        if query.is_none() {
                            query = Some(String::new());
                        }
                        query.as_mut().unwrap().push(ch);
                        self.pos += 1;
                    }
                }

                ParseState::Fragment => {
                    if fragment.is_none() {
                        fragment = Some(String::new());
                    }
                    fragment.as_mut().unwrap().push(ch);
                    self.pos += 1;
                }
            }
        }

        Ok(Url {
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }
}

fn main() {
    let urls = [
        "https://example.com/path/to/page?key=value#section",
        "http://user:pass@host:8080/path",
        "file:///home/user/file.txt",
        "mailto:user@example.com",
    ];

    for url_str in &urls {
        let mut parser = UrlParser::new(url_str);
        match parser.parse() {
            Ok(url) => {
                println!("\nURL: {}", url_str);
                println!("  Scheme: {}", url.scheme);
                println!("  Authority: {:?}", url.authority);
                println!("  Path: {}", url.path);
                println!("  Query: {:?}", url.query);
                println!("  Fragment: {:?}", url.fragment);
            }
            Err(e) => println!("Parse error: {}", e),
        }
    }
}
```

**Key Patterns**:
- State transitions on delimiter characters
- Lookahead for multi-character delimiters
- Optional components with `Option<String>`

---

## Text Editor Data Structures

### Recipe 9: Gap Buffer Implementation

**Problem**: Efficient text insertion/deletion at cursor position.

**Use Case**: Text editors, in-memory document editing.

**Data Structure**: Gap buffer

```rust
struct GapBuffer {
    buffer: Vec<char>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    fn new() -> Self {
        GapBuffer::with_capacity(64)
    }

    fn with_capacity(capacity: usize) -> Self {
        GapBuffer {
            buffer: vec!['\0'; capacity],
            gap_start: 0,
            gap_end: capacity,
        }
    }

    fn from_str(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        let capacity = (len * 2).max(64);

        let mut buffer = vec!['\0'; capacity];
        buffer[..len].copy_from_slice(&chars);

        GapBuffer {
            buffer,
            gap_start: len,
            gap_end: capacity,
        }
    }

    // Insert character at cursor (gap_start)
    fn insert(&mut self, ch: char) {
        if self.gap_start == self.gap_end {
            self.grow();
        }

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;
    }

    // Delete character before cursor
    fn delete_backward(&mut self) -> Option<char> {
        if self.gap_start == 0 {
            return None;
        }

        self.gap_start -= 1;
        Some(self.buffer[self.gap_start])
    }

    // Delete character after cursor
    fn delete_forward(&mut self) -> Option<char> {
        if self.gap_end == self.buffer.len() {
            return None;
        }

        let ch = self.buffer[self.gap_end];
        self.gap_end += 1;
        Some(ch)
    }

    // Move cursor left
    fn move_left(&mut self) {
        if self.gap_start > 0 {
            self.gap_start -= 1;
            self.gap_end -= 1;
            self.buffer[self.gap_end] = self.buffer[self.gap_start];
        }
    }

    // Move cursor right
    fn move_right(&mut self) {
        if self.gap_end < self.buffer.len() {
            self.buffer[self.gap_start] = self.buffer[self.gap_end];
            self.gap_start += 1;
            self.gap_end += 1;
        }
    }

    // Move cursor to position
    fn move_to(&mut self, pos: usize) {
        let current_pos = self.gap_start;

        if pos < current_pos {
            for _ in 0..(current_pos - pos) {
                self.move_left();
            }
        } else if pos > current_pos {
            for _ in 0..(pos - current_pos) {
                self.move_right();
            }
        }
    }

    fn grow(&mut self) {
        let new_capacity = self.buffer.len() * 2;
        let additional = new_capacity - self.buffer.len();

        // Extend buffer
        self.buffer.resize(new_capacity, '\0');

        // Move content after gap to end
        let content_after_gap = self.buffer.len() - self.gap_end - additional;
        for i in (0..content_after_gap).rev() {
            self.buffer[new_capacity - 1 - i] = self.buffer[self.gap_end + i];
        }

        self.gap_end = new_capacity - content_after_gap;
    }

    fn to_string(&self) -> String {
        let mut result = String::new();

        for i in 0..self.gap_start {
            result.push(self.buffer[i]);
        }

        for i in self.gap_end..self.buffer.len() {
            result.push(self.buffer[i]);
        }

        result
    }

    fn len(&self) -> usize {
        self.gap_start + (self.buffer.len() - self.gap_end)
    }

    fn cursor_position(&self) -> usize {
        self.gap_start
    }
}

fn main() {
    let mut gb = GapBuffer::from_str("Hello World");

    println!("Initial: {}", gb.to_string());
    println!("Cursor at: {}", gb.cursor_position());

    // Move to position 5 (before "World")
    gb.move_to(6);
    gb.delete_backward();  // Delete space
    gb.insert(',');
    gb.insert(' ');

    println!("After edit: {}", gb.to_string());

    // Insert at beginning
    gb.move_to(0);
    gb.insert('>');
    gb.insert(' ');

    println!("Final: {}", gb.to_string());
}
```

**Key Concepts**:
- Gap buffer for O(1) insertion/deletion at cursor
- Gap grows when needed
- Move operations shift gap position
- Efficient for localized edits

---

### Recipe 10: Rope Data Structure

**Problem**: Efficient large document editing with undo/redo.

**Use Case**: Text editors for large files, collaborative editing.

**Data Structure**: Rope (tree of strings)

```rust
#[derive(Clone)]
enum Rope {
    Leaf(String),
    Branch {
        left: Box<Rope>,
        right: Box<Rope>,
        length: usize,  // Total length of left subtree
    },
}

impl Rope {
    fn from_str(s: &str) -> Self {
        Rope::Leaf(s.to_string())
    }

    fn concat(left: Rope, right: Rope) -> Self {
        let length = left.len();
        Rope::Branch {
            left: Box::new(left),
            right: Box::new(right),
            length,
        }
    }

    fn len(&self) -> usize {
        match self {
            Rope::Leaf(s) => s.len(),
            Rope::Branch { length, right, .. } => length + right.len(),
        }
    }

    // Insert string at position
    fn insert(&mut self, pos: usize, text: &str) {
        let (left, right) = self.split(pos);
        *self = Rope::concat(Rope::concat(left, Rope::from_str(text)), right);
    }

    // Delete range
    fn delete(&mut self, start: usize, end: usize) {
        let (left, rest) = self.split(start);
        let (_, right) = rest.split(end - start);
        *self = Rope::concat(left, right);
    }

    // Split rope at position
    fn split(self, pos: usize) -> (Rope, Rope) {
        match self {
            Rope::Leaf(s) => {
                if pos >= s.len() {
                    (Rope::Leaf(s), Rope::Leaf(String::new()))
                } else if pos == 0 {
                    (Rope::Leaf(String::new()), Rope::Leaf(s))
                } else {
                    let (left, right) = s.split_at(pos);
                    (Rope::Leaf(left.to_string()), Rope::Leaf(right.to_string()))
                }
            }
            Rope::Branch { left, right, length } => {
                if pos < length {
                    let (ll, lr) = left.split(pos);
                    (ll, Rope::concat(lr, *right))
                } else if pos == length {
                    (*left, *right)
                } else {
                    let (rl, rr) = right.split(pos - length);
                    (Rope::concat(*left, rl), rr)
                }
            }
        }
    }

    // Get character at position
    fn char_at(&self, pos: usize) -> Option<char> {
        match self {
            Rope::Leaf(s) => s.chars().nth(pos),
            Rope::Branch { left, right, length } => {
                if pos < *length {
                    left.char_at(pos)
                } else {
                    right.char_at(pos - length)
                }
            }
        }
    }

    // Convert to string
    fn to_string(&self) -> String {
        match self {
            Rope::Leaf(s) => s.clone(),
            Rope::Branch { left, right, .. } => {
                format!("{}{}", left.to_string(), right.to_string())
            }
        }
    }

    // Rebalance tree if needed
    fn rebalance(self) -> Self {
        // Simplified rebalancing: collect all leaves and rebuild
        let text = self.to_string();
        if text.len() < 100 {
            return Rope::Leaf(text);
        }

        let mid = text.len() / 2;
        let (left, right) = text.split_at(mid);
        Rope::concat(Rope::Leaf(left.to_string()), Rope::Leaf(right.to_string()))
    }
}

fn main() {
    let mut rope = Rope::from_str("Hello World");
    println!("Initial: {}", rope.to_string());

    // Insert at position 5
    rope.insert(5, ", Beautiful");
    println!("After insert: {}", rope.to_string());

    // Delete range
    rope.delete(5, 16);  // Remove ", Beautiful"
    println!("After delete: {}", rope.to_string());

    // Character access
    if let Some(ch) = rope.char_at(0) {
        println!("First char: {}", ch);
    }

    println!("Length: {}", rope.len());
}
```

**Key Concepts**:
- Binary tree of string fragments
- O(log n) insertion and deletion
- Structural sharing for undo/redo
- Rebalancing for performance

---

## Pattern Matching Algorithms

### Recipe 11: Knuth-Morris-Pratt (KMP) String Search

**Problem**: Find all occurrences of pattern in text.

**Use Case**: Text search, log analysis, DNA sequence matching.

**Algorithm**: KMP with failure function

```rust
struct KmpMatcher {
    pattern: Vec<char>,
    failure: Vec<usize>,
}

impl KmpMatcher {
    fn new(pattern: &str) -> Self {
        let pattern: Vec<char> = pattern.chars().collect();
        let failure = Self::compute_failure(&pattern);

        KmpMatcher { pattern, failure }
    }

    fn compute_failure(pattern: &[char]) -> Vec<usize> {
        let mut failure = vec![0; pattern.len()];
        let mut j = 0;

        for i in 1..pattern.len() {
            while j > 0 && pattern[i] != pattern[j] {
                j = failure[j - 1];
            }

            if pattern[i] == pattern[j] {
                j += 1;
            }

            failure[i] = j;
        }

        failure
    }

    fn find_all(&self, text: &str) -> Vec<usize> {
        let text: Vec<char> = text.chars().collect();
        let mut matches = Vec::new();
        let mut j = 0;

        for (i, &ch) in text.iter().enumerate() {
            while j > 0 && ch != self.pattern[j] {
                j = self.failure[j - 1];
            }

            if ch == self.pattern[j] {
                j += 1;
            }

            if j == self.pattern.len() {
                matches.push(i + 1 - j);
                j = self.failure[j - 1];
            }
        }

        matches
    }

    fn contains(&self, text: &str) -> bool {
        !self.find_all(text).is_empty()
    }
}

fn main() {
    let matcher = KmpMatcher::new("ABABC");
    let text = "ABABDABACDABABCABAB";

    let matches = matcher.find_all(text);
    println!("Pattern found at positions: {:?}", matches);

    for pos in matches {
        println!("  Position {}: {}", pos, &text[pos..pos + 5]);
    }
}
```

**Key Concepts**:
- Failure function for efficient backtracking
- O(n + m) time complexity
- No backtracking in text

---

### Recipe 12: Boyer-Moore String Search

**Problem**: Fast string search with preprocessing.

**Use Case**: Large text search, virus scanning, plagiarism detection.

**Algorithm**: Boyer-Moore with bad character rule

```rust
use std::collections::HashMap;

struct BoyerMoore {
    pattern: Vec<char>,
    bad_char: HashMap<char, usize>,
}

impl BoyerMoore {
    fn new(pattern: &str) -> Self {
        let pattern: Vec<char> = pattern.chars().collect();
        let bad_char = Self::build_bad_char_table(&pattern);

        BoyerMoore { pattern, bad_char }
    }

    fn build_bad_char_table(pattern: &[char]) -> HashMap<char, usize> {
        let mut table = HashMap::new();

        for (i, &ch) in pattern.iter().enumerate() {
            table.insert(ch, i);
        }

        table
    }

    fn find_all(&self, text: &str) -> Vec<usize> {
        let text: Vec<char> = text.chars().collect();
        let mut matches = Vec::new();
        let m = self.pattern.len();
        let n = text.len();

        if m > n {
            return matches;
        }

        let mut s = 0;  // Shift of pattern relative to text

        while s <= n - m {
            let mut j = m;

            // Scan from right to left
            while j > 0 && self.pattern[j - 1] == text[s + j - 1] {
                j -= 1;
            }

            if j == 0 {
                // Match found
                matches.push(s);

                // Shift pattern
                if s + m < n {
                    let next_char = text[s + m];
                    s += m - self.bad_char.get(&next_char).unwrap_or(&0);
                } else {
                    s += 1;
                }
            } else {
                // Mismatch: use bad character rule
                let bad_char = text[s + j - 1];
                let shift = if let Some(&pos) = self.bad_char.get(&bad_char) {
                    if pos < j - 1 {
                        j - 1 - pos
                    } else {
                        1
                    }
                } else {
                    j
                };

                s += shift;
            }
        }

        matches
    }
}

fn main() {
    let bm = BoyerMoore::new("EXAMPLE");
    let text = "THIS IS A SIMPLE EXAMPLE FOR EXAMPLE MATCHING";

    let matches = bm.find_all(text);
    println!("Matches at: {:?}", matches);

    for pos in matches {
        println!("  {}", &text[pos..pos + 7]);
    }
}
```

**Key Concepts**:
- Right-to-left scanning
- Bad character rule for large shifts
- Efficient for long patterns

---

## String Interning

### Recipe 13: String Interning Pool

**Problem**: Reduce memory usage for repeated strings.

**Use Case**: Symbol tables, configuration keys, tag/label systems.

**Data Structure**: String interning with hash map

```rust
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct InternedString(Arc<str>);

impl InternedString {
    fn as_str(&self) -> &str {
        &self.0
    }
}

struct StringInterner {
    map: HashMap<Arc<str>, InternedString>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            map: HashMap::new(),
        }
    }

    fn intern(&mut self, s: &str) -> InternedString {
        if let Some(interned) = self.map.get(s) {
            return interned.clone();
        }

        let arc: Arc<str> = Arc::from(s);
        let interned = InternedString(arc.clone());
        self.map.insert(arc, interned.clone());
        interned
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn memory_usage(&self) -> usize {
        self.map.iter()
            .map(|(k, _)| k.len())
            .sum()
    }
}

fn main() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("world");
    let s3 = interner.intern("hello");  // Returns same Arc

    println!("s1 == s3: {}", s1 == s3);  // true
    println!("Unique strings: {}", interner.len());

    // Demonstrate memory savings
    let tags = vec!["rust", "programming", "rust", "tutorial", "rust"];
    let interned_tags: Vec<_> = tags.iter()
        .map(|&s| interner.intern(s))
        .collect();

    println!("Tags: {} unique", interner.len());
    println!("Memory: {} bytes", interner.memory_usage());
}
```

**Key Patterns**:
- Arc for shared ownership
- HashMap for deduplication
- Cheap cloning with reference counting

---

## Quick Reference

### String Types

| Type | Owned | UTF-8 | Use Case |
|------|-------|-------|----------|
| `String` | Yes | Yes | Dynamic, growable strings |
| `&str` | No | Yes | String slices, literals |
| `Cow<str>` | Sometimes | Yes | Clone-on-write optimization |
| `OsString` | Yes | No | Platform strings, file paths |
| `Path` | No | No | File system paths |

### Common Operations

```rust
// Concatenation
let s = format!("{}{}", s1, s2);
let s = [s1, s2].concat();
let s = [s1, s2].join("");

// Splitting
for part in s.split(',') { }
for line in s.lines() { }
let parts: Vec<_> = s.split_whitespace().collect();

// Trimming
let s = s.trim();
let s = s.trim_start();
let s = s.trim_end();

// Case conversion
let s = s.to_lowercase();
let s = s.to_uppercase();

// Searching
if s.contains("pattern") { }
if let Some(pos) = s.find("pattern") { }
```

### Performance Tips

1. **Pre-allocate**: Use `String::with_capacity()` when size known
2. **Avoid cloning**: Use `&str` parameters instead of `String`
3. **Use Cow**: For conditional modifications
4. **Intern strings**: For repeated strings
5. **Chars not bytes**: Use `char_indices()` for UTF-8 safety

---

## Summary

String processing in Rust requires understanding:

1. **Type System**: String, &str, Cow, OsString, Path
2. **Zero-Copy**: Borrowing and slicing without allocation
3. **UTF-8**: Proper handling of multi-byte characters
4. **State Machines**: Efficient parsing and lexing
5. **Data Structures**: Gap buffers and ropes for editing
6. **Algorithms**: KMP, Boyer-Moore for pattern matching
7. **Optimization**: Interning for memory efficiency

**Key Principles**:
- UTF-8 is default, always check boundaries
- Borrow when possible, own when necessary
- Use appropriate data structures for use case
- Profile before optimizing
