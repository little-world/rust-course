# Cookbook: String Processing

> Advanced string manipulation, parsing, and text processing algorithms

## String Foundations

Rust distinguishes between `String` (an owned, heap-allocated, growable UTF-8 buffer) and `&str` (a borrowed slice into UTF-8 data). This design enforces memory safety and prevents common bugs like buffer overflows and use-after-free errors. Every `String` guarantees valid UTF-8 encoding, making operations like indexing by byte position potentially unsafe—Rust requires explicit handling of character boundaries through iterators or methods like `.chars()`. Internally, `String` is a `Vec<u8>` with UTF-8 validation, storing three words: a pointer to heap data, length (bytes currently used), and capacity (bytes allocated). String slices (`&str`) are "fat pointers" containing a pointer and length, enabling zero-copy views into existing data. This architecture means string operations fall into three categories: byte-level (fast but requires UTF-8 awareness), character-level (safe but slower due to variable-width encoding), and grapheme-level (most correct for user-perceived characters, requires external crates). The ownership model forces explicit decisions about allocation—methods ending in `_mut` or returning `String` signal potential allocation, while those returning `&str` guarantee zero-copy operations.

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

Rust's string handling system is more nuanced than many languages, offering multiple string types designed for different use cases. Understanding these types is crucial for writing efficient, safe code. The primary distinction revolves around **ownership** (whether the type owns its data or merely borrows it) and **encoding guarantees** (whether UTF-8 validity is enforced).

The type system prevents common string-related bugs at compile time: buffer overflows, invalid UTF-8 sequences, and use-after-free errors. However, this safety comes with the requirement to understand which type to use when.

### Pattern 1: Choosing the Right String Type

**Problem**: Rust offers multiple string types, and choosing the wrong one leads to unnecessary allocations, API inflexibility, or platform compatibility issues.

**Solution**: Select types based on ownership needs, encoding requirements, and platform interoperability.

**When to Use This Pattern**: API design, cross-platform code, performance-critical string handling, FFI boundaries.


```rust
use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

//=====================================================
// String: Owned, heap-allocated, growable UTF-8 string
//=====================================================
fn string_example() {
    let mut s = String::from("Hello");
    s.push_str(", World!");
    println!("{}", s);

    // Use when:
    // - Need to own the string
    // - Building strings dynamically
    // - Returning strings from functions
}

//==============================================
// &str: Borrowed string slice, doesn't own data
//==============================================
fn str_slice_example(s: &str) {
    println!("Length: {}", s.len());

    // Use when:
    // - Read-only access needed
    // - Function parameters (most flexible)
    // - String literals
}

//=================================================================
// Cow (Clone on Write): Borrows when possible, owns when necessary
//=================================================================
fn cow_example<'a>(data: &'a str, uppercase: bool) -> Cow<'a, str> {
    if uppercase {
        Cow::Owned(data.to_uppercase())  // Allocates
    } else {
        Cow::Borrowed(data)  // No allocation
    }
}

//===========================================================
// OsString/OsStr: Platform-native strings (may not be UTF-8)
//===========================================================
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

//================================
// Path/PathBuf: File system paths
//================================
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

**Algorithm & Design Rationale**:

The string type hierarchy reflects a fundamental trade-off in systems programming:

1. **String vs &str**: This is the owned/borrowed dichotomy. `String` is heap-allocated and growable (similar to `Vec<u8>` but with UTF-8 guarantees). `&str` is a "view" into UTF-8 data stored elsewhere. Functions taking `&str` are maximally flexible since `String` automatically derefs to `&str`, but the reverse isn't true.

2. **Cow (Clone on Write)**: This is a performance optimization that delays allocation. The type `Cow<'a, str>` is an enum: either `Borrowed(&'a str)` or `Owned(String)`. When a function might or might not need to modify its input, `Cow` allows returning the borrowed input when no changes are needed, avoiding allocation entirely.

3. **OsString/OsStr**: Operating systems don't guarantee UTF-8 encoding. Windows uses UTF-16, Unix systems use arbitrary byte sequences. `OsString` handles these platform differences while preserving the owned/borrowed distinction. Use these at OS boundaries: file paths, environment variables, command-line arguments.

4. **Path/PathBuf**: These wrap `OsStr`/`OsString` with path-specific operations (extension extraction, joining, parent directory). They understand path separators are platform-dependent (`/` vs `\`).

**Memory Layout**:
- `String`: 3 words (pointer, length, capacity)
- `&str`: 2 words (pointer, length)
- `Cow<str>`: 4 words (discriminant + either 2-word `&str` or 3-word `String`)

**Key Concepts**:
- `String` owns data, `&str` borrows
- `Cow` optimizes by borrowing when possible, allocating only when necessary
- `OsString` handles platform-specific encodings (UTF-16 on Windows, bytes on Unix)
- `Path` provides platform-independent path operations with correct separator handling

---

### Pattern 2: String Builder Pattern

**Problem**: Concatenating strings in a loop or building complex strings from parts can cause multiple allocations. Using `s = s + "text"` or `s.push_str()` repeatedly may reallocate memory multiple times.

**Solution**: Pre-allocate capacity and use a builder with method chaining for ergonomic, efficient construction.

**Why This Matters**: String concatenation creates a new String on each operation. In a loop with N iterations, naive concatenation triggers O(N²) character copies as each intermediate string is reallocated and copied. Pre-allocation reduces this to O(N).

**Use Case**: Template rendering, HTML generation, log formatting, SQL query building.


```rust
//==============================================
// Pattern: Builder with capacity pre-allocation
//==============================================
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

//======================
// Pattern: HTML builder 
//======================
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

**Algorithm Analysis**:

The builder pattern's efficiency comes from capacity pre-allocation. When you call `String::with_capacity(n)`, Rust allocates a buffer of size `n` immediately. Subsequent `push_str()` calls are O(1) operations that copy bytes into the pre-allocated buffer—no reallocation needed until capacity is exhausted.

Without pre-allocation, `String` uses an exponential growth strategy (doubling capacity when full). While this gives amortized O(1) insertion, it still wastes memory and CPU on copying during growth. If you know the approximate final size, pre-allocation eliminates this overhead.

**Performance Characteristics**:
- **With capacity**: O(N) total time for N bytes inserted
- **Without capacity**: O(N) amortized, but with up to log(N) reallocations
- **Memory**: Pre-allocated buffer may have unused capacity; this is acceptable for temporary builders

**Key Patterns**:
- Pre-allocate capacity when size is known (or can be estimated)
- Method chaining (`&mut self` returns) for fluent API
- Consume builder with `build(self)` to transfer ownership without copying

---

## Zero-Copy String Operations

The most efficient memory operation is **no operation at all**. Zero-copy techniques avoid allocating new strings by working with borrowed slices (`&str`) that point into existing data. This is critical for performance-sensitive code: allocation is expensive, and copying large strings can dominate runtime.

Rust's ownership system makes zero-copy safe. The borrow checker ensures slices can't outlive the data they point to, preventing use-after-free bugs that plague C programs doing similar optimization.

### Pattern 3: String Slicing and Splitting

**Problem**: Parsing structured text (CSV, logs, configuration files) by creating owned strings for each field wastes memory and CPU time.

**Solution**: Use iterators that return `&str` slices into the original string. Methods like `split()`, `lines()`, and `split_whitespace()` return iterators over borrowed slices.

**Why This Matters**: Parsing a 10MB log file by allocating strings for each line and field can cause thousands of allocations. Zero-copy parsing borrows from the original buffer, reducing memory usage by orders of magnitude and eliminating allocation overhead.

**Use Case**: Log parsing, CSV processing, configuration file parsing, text analysis on large files.

```rust
//==============================
// Pattern:Zero-copy line parser
//==============================
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

//=========================================================
// Pattern: CSV parser with zero allocations during parsing
//=========================================================
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

//==========================================
// Pattern: String view with bounds checking
//==========================================
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

**Algorithm Insights**:

String slicing in Rust is implemented as a fat pointer: a pointer to the start byte plus a length. Creating a slice is O(1)—just two integer values. No copying occurs.

The `split()` iterator maintains state (current position) and advances through the string on each `next()` call, returning slices between delimiters. This is lazy: if you only take the first 3 items from `split()`, it never scans past the 3rd delimiter.

**Critical Safety Issue**: UTF-8 characters can be 1-4 bytes. Slicing in the middle of a multi-byte character would create invalid UTF-8. Rust prevents this at compile-time by making slicing operations panic if the index isn't a character boundary. Use `is_char_boundary()` to check before slicing, or use `char_indices()` which returns valid indices.

**Performance**:
- Creating a slice: O(1)
- Iterating with `split()`: O(N) where N is string length
- Memory: Zero allocations (slices borrow from original)

**Key Techniques**:
- Return iterators instead of vectors to avoid collecting until necessary
- Use string slices (`&str`) as return types for maximum flexibility
- Leverage `split()`, `lines()`, `split_whitespace()` for zero-copy splitting
- Always check `is_char_boundary()` before manual slicing with byte indices

---

### Pattern 4: Cow for Conditional Allocation

**Problem**: Functions that sometimes need to modify input and sometimes don't face a dilemma: always allocate (wasteful), or return different types (awkward API).

**Solution**: Return `Cow<str>` (Clone-On-Write). Check if modification is needed first. If not, return `Cow::Borrowed`. If yes, allocate and return `Cow::Owned`.

**Why This Matters**: Consider a function that normalizes whitespace—if the input has no extra whitespace, why allocate? `Cow` lets you return the original string. For a web server processing millions of URLs, this can eliminate gigabytes of allocations for already-normalized inputs.

**Use Case**: Data normalization, HTML escaping, path canonicalization, validation with sanitization.

```rust
use std::borrow::Cow;

//===============================================================
// Pattern: Normalize whitespace: collapse multiple spaces to one
//===============================================================
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

//=============================================
// Pattern: Escape HTML entities only if needed
//=============================================
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

//================================================================
// Pattern: Remove prefix/suffix without allocation if not present
//================================================================
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

//============================
// Pattern: Case normalization
//============================
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

**Algorithm Strategy**:

The `Cow` pattern implements a two-phase algorithm:

1. **Scan Phase**: Iterate through the input to detect if modification is needed. This is a read-only O(N) pass. If no modification is required, return `Cow::Borrowed(input)` immediately—zero allocation.

2. **Build Phase**: If modification is needed, allocate a new `String`, perform transformations, and return `Cow::Owned(result)`.

The key insight: the scan phase cost (O(N)) is dominated by the allocation and copy cost we're trying to avoid. Even though we potentially traverse the string twice (once to check, once to modify), this is faster than always allocating.

**Trade-off Analysis**:
- **Best case** (no modification needed): O(N) scan, zero allocation
- **Worst case** (modification needed): O(N) scan + O(N) build = O(N) total, one allocation
- **Always allocating**: O(N) build, one allocation (even when unnecessary)

For high-frequency operations where the input is often already in the desired form, `Cow` provides significant savings.

**Key Patterns**:
- Implement fast-path check before allocating (e.g., `contains()` check for escaping)
- Return `Cow::Borrowed` for no-op transformations
- Return `Cow::Owned` only when changes made
- Pre-allocate capacity for owned variants when size is known (`s.len() + overhead`)

---

## UTF-8 Handling and Validation

UTF-8 is a variable-length encoding where characters take 1-4 bytes. Rust's `String` and `&str` guarantee valid UTF-8, but you often need to handle potentially invalid data from external sources (network, files, FFI).

Understanding UTF-8's structure is critical for implementing custom validators and for safe string manipulation.

**UTF-8 Encoding Rules**:
```
1-byte (ASCII):      0xxxxxxx                    (U+0000 to U+007F)
2-byte:              110xxxxx 10xxxxxx           (U+0080 to U+07FF)
3-byte:              1110xxxx 10xxxxxx 10xxxxxx  (U+0800 to U+FFFF)
4-byte:  11110xxx 10xxxxxx 10xxxxxx 10xxxxxx     (U+10000 to U+10FFFF)
```

**Validation Requirements**:
1. First byte determines sequence length
2. Continuation bytes must match `10xxxxxx` pattern
3. Shortest encoding required (no overlong sequences)
4. No encoding of surrogates (U+D800 to U+DFFF)
5. Maximum code point is U+10FFFF

### Pattern 5: UTF-8 Validation and Repair

**Problem**: Reading text from untrusted sources (files, network, legacy systems) may yield invalid UTF-8 byte sequences, causing panics in Rust's string APIs.

**Solution**: Validate UTF-8 explicitly. Use `from_utf8()` for strict validation, or `from_utf8_lossy()` to replace invalid sequences with U+FFFD (�).

**Why This Matters**: Invalid UTF-8 can indicate corrupted data, encoding mismatches (Latin-1 vs UTF-8), or malicious input. Proper handling prevents crashes and security issues.

**Use Case**: File I/O, network protocols, FFI with C libraries, data recovery.

```rust
//======================================================
// Pattern: Validate UTF-8 and replace invalid sequences
//======================================================
fn validate_utf8_lossy(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}

//============================
// Pattern:: Strict validation
//============================
fn validate_utf8_strict(data: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(data)
}

//=====================================================
// Pattern: Custom UTF-8 validator with error positions
//=====================================================
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

**Algorithm Walkthrough**:

The validator implements a state machine that processes bytes sequentially:

1. **First Byte Analysis**: Examine the bit pattern to determine character length:
   - `0xxxxxxx`: 1-byte ASCII (U+0000-U+007F)
   - `110xxxxx`: 2-byte sequence expected
   - `1110xxxx`: 3-byte sequence expected
   - `11110xxx`: 4-byte sequence expected
   - Any other pattern is invalid

2. **Continuation Byte Validation**: Each subsequent byte must match `10xxxxxx`. The validator checks this with `byte & 0xC0 == 0x80`.

3. **Code Point Reconstruction**: Extract bits from each byte and combine:
   - For 2-byte: `((first & 0x1F) << 6) | (second & 0x3F)`
   - For 3-byte: `((first & 0x0F) << 12) | ((second & 0x3F) << 6) | (third & 0x3F)`
   - For 4-byte: `((first & 0x07) << 18) | ... `

4. **Overlong Detection**: Each range has a minimum code point:
   - 2-byte sequences must encode U+0080 or higher (if lower, use 1-byte)
   - 3-byte sequences must encode U+0800 or higher
   - 4-byte sequences must encode U+10000 or higher
   This prevents security attacks that use overlong encodings to bypass filters.

5. **Range Validation**: Reject surrogate pairs (U+D800-U+DFFF) and values > U+10FFFF.

**Performance**:
- Time: O(N) single pass through bytes
- Space: O(1) constant memory for state
- Early termination on first error

**Key Concepts**:
- UTF-8 is self-synchronizing: you can find character boundaries by scanning for non-`10xxxxxx` bytes
- Overlong encoding detection prevents security exploits
- Surrogate detection (U+D800-U+DFFF) rejects UTF-16 artifacts
- Lossy conversion (`from_utf8_lossy`) replaces invalid sequences with U+FFFD (�)
- Strict validation (`from_utf8`) returns `Err` with the error position

---

### Pattern 6: Character and Grapheme Iteration

**Problem**: What looks like "one character" to users may be multiple Unicode code points. Emoji like "👨‍👩‍👧‍👦" (family) or "é" (e + combining accent) break naive iteration.

**Solution**: Understand three levels of iteration:
- **Bytes**: Raw UTF-8 bytes (1-4 per code point)
- **Characters**: Unicode code points (use `.chars()`)
- **Graphemes**: User-perceived characters (use `unicode-segmentation` crate)

**Why This Matters**: Truncating at the wrong boundary creates corrupted text. Displaying text requires grapheme-aware width calculation for proper alignment. String reversal must preserve grapheme clusters.

**Use Case**: Text rendering, terminal output, string truncation, text editing, internationalization.

```rust
use unicode_segmentation::UnicodeSegmentation;
//====================
// Pattern: Iterations
//====================
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

//====================================================
// Pattern: Calculate display width (East Asian Width)
//====================================================
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

//======================================================
// Pattern: Safe string truncation at character boundary
//======================================================
fn truncate_at_char(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

//=======================================
// Pattern: Truncate at grapheme boundary
//=======================================
fn truncate_at_grapheme(s: &str, max_graphemes: usize) -> &str {
    s.graphemes(true)
        .take(max_graphemes)
        .collect::<String>()
        .as_str()
}

//=====================================================
// Pattern: Reverse string preserving grapheme clusters
//=====================================================
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

**Understanding the Three Levels**:

1. **Bytes** (`s.bytes()`): Raw UTF-8 bytes. "é" encoded as U+00E9 is two bytes: `0xC3 0xA9`. This is the lowest level—useful for serialization and binary protocols, but meaningless for text processing.

2. **Characters** (`s.chars()`): Unicode code points. "é" can be:
   - Single code point: U+00E9 (precomposed)
   - Two code points: U+0065 (e) + U+0301 (combining acute accent)
   Both representations are valid and display identically. `chars()` yields one or two items depending on the representation.

3. **Graphemes** (`s.graphemes(true)`): User-perceived characters following Unicode segmentation rules. "é" is always one grapheme, regardless of internal representation. Complex emoji like "👨‍👩‍👧‍👦" are composed of base emoji joined with Zero Width Joiners (ZWJ), but form a single grapheme.

**East Asian Width**: Characters have display widths in terminal applications. ASCII is width 1, but CJK (Chinese/Japanese/Korean) characters and emoji are width 2. The `display_width()` function estimates this using Unicode ranges.

**Truncation Safety**: Never truncate using byte indices unless you verify boundaries. Use `char_indices()` which returns `(byte_index, char)` pairs where byte_index is guaranteed to be a valid UTF-8 boundary.

**Key Concepts**:
- Bytes < Characters < Graphemes (increasingly high-level abstractions)
- Grapheme clusters preserve user-perceived text units
- East Asian Width (UAX #11) affects terminal display calculations
- Always truncate at grapheme or character boundaries, never bytes
- Use `char_indices()` for safe byte-index iteration

---

## String Parsing State Machines

### Pattern 7: Lexer with State Machine
 Tokenize source code or structured text.
 Using Finite state machine lexer

```rust
//=========================
// Pattern Lexer (complete)
//=========================
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

### Pattern 8: URL Parser State Machine
 Parse URLs into components.
Using RFC 3986 URL parsing

```rust
//===============================
// Pattern: URL Parser (complete)
//===============================
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

### Pattern 9: Gap Buffer Implementation

**Problem**: Text editors need fast insertion/deletion at the cursor position. Using `Vec<char>` requires O(N) time to insert/delete because all characters after the cursor must shift.

**Solution**: Gap buffer—maintain a "gap" (empty space) at the cursor position. Insertion fills the gap (O(1)), and cursor movement slides the gap by copying characters across it.

**Why This Matters**: Text editors process thousands of keystrokes per second. O(N) insertion would make typing lag noticeably. Gap buffers provide O(1) insertion at the cursor and O(distance) cursor movement—optimal for the common case of sequential editing.

**Use Case**: Text editors (Emacs, early versions), simple document editing, undo buffers.

```rust
//======================
// Gap Buffer (complete)
//======================
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

**Algorithm Explanation**:

A gap buffer is a vector with three regions:
```
[prefix | gap | suffix]
 ↑       ↑     ↑       ↑
 0    gap_start gap_end buffer.len()
```

- **Prefix**: Characters before cursor (`buffer[0..gap_start]`)
- **Gap**: Empty space (`buffer[gap_start..gap_end]`)
- **Suffix**: Characters after cursor (`buffer[gap_end..]`)

The cursor is conceptually at `gap_start`. Operations:

**1. Insert at Cursor**: O(1)
```rust
buffer[gap_start] = ch;
gap_start += 1;  // Gap shrinks from left
```

**2. Delete Backward**: O(1)
```rust
gap_start -= 1;  // Gap grows to the left
// Character at buffer[gap_start] is now in the gap (deleted)
```

**3. Delete Forward**: O(1)
```rust
gap_end += 1;  // Gap grows to the right
// Character at buffer[gap_end-1] is now in the gap (deleted)
```

**4. Move Cursor Left**: O(1)
```rust
gap_start -= 1;
gap_end -= 1;
buffer[gap_end] = buffer[gap_start];  // Move char across gap
```
The gap "slides" left by moving one character from prefix to suffix.

**5. Move Cursor Right**: O(1)
```rust
buffer[gap_start] = buffer[gap_end];  // Move char across gap
gap_start += 1;
gap_end += 1;
```

**6. Move to Position**: O(distance)
Repeatedly move left or right. For large jumps, this can be optimized by bulk copying.

**7. Gap Full**: When `gap_start == gap_end`, grow the buffer:
- Allocate new buffer (typically double size)
- Copy prefix to start
- Copy suffix to end
- Gap is now in the middle

**Memory Layout Example**:

Initial: "Hello" with gap size 4 at position 5:
```
[H][e][l][l][o][_][_][_][_]
                ↑gap_start=5  ↑gap_end=9
```

After inserting " World":
```
[H][e][l][l][o][ ][W][o][r]
                          ↑gap_start=9, gap_end=9 (gap full!)
```

**Performance Characteristics**:
- **Insert/delete at cursor**: O(1) amortized
- **Cursor movement**: O(distance moved)
- **Random access**: O(N) worst case (if need to move gap)
- **Memory**: O(N + gap_size)

**Trade-offs**:
- **Pros**: Extremely fast for sequential editing (typing, deleting)
- **Cons**: Multiple cursor positions require multiple gaps (use Rope instead)
- **Cons**: Moving cursor far requires O(N) operations

Gap buffers excel when edits are localized around a single cursor—exactly the pattern of human typing!

**Key Concepts**:
- Gap buffer provides O(1) insertion/deletion at cursor position
- Gap grows when full (exponential reallocation)
- Cursor movement slides gap by copying characters across it: O(distance)
- Efficient for localized, sequential edits (the common case in text editing)
- Simple implementation compared to more complex structures like ropes

---

### Pattern 10: Rope Data Structure

**Problem**: Gap buffers struggle with:
- Large documents (multi-megabyte files cause O(N) cursor movement)
- Multiple cursors (each needs its own gap)
- Undo/redo (copying entire buffer is expensive)

**Solution**: Rope—a binary tree where leaves contain string fragments. Operations split/concatenate tree nodes, achieving O(log N) insertion/deletion anywhere in the document.

**Why This Matters**: For documents > 1MB or collaborative editing with multiple cursors, ropes outperform gap buffers dramatically. The tree structure enables structural sharing for efficient undo/redo without copying the entire document.

**Use Case**: Modern text editors (Xi, Visual Studio Code internals), large log file viewers, collaborative editing systems, version control systems.

```rust
//================
// Rope (complete)
//================
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

**Algorithm Explanation**:

A rope is a binary tree where:
- **Leaf nodes**: Contain actual string data
- **Branch nodes**: Have left/right children and store the total length of the left subtree

Example rope for "Hello, World!":
```
        Branch(6)
        /        \
    Leaf("Hello")  Leaf(", World!")
```

The number in `Branch(6)` is the length of the left subtree (6 characters in "Hello").

**Core Operations**:

**1. Indexing** (`char_at(pos)`): O(log N)
```rust
if pos < left.len() {
    search left subtree
} else {
    search right subtree at (pos - left.len())
}
```

Navigate down the tree by comparing position with left subtree length. This is O(log N) with balanced trees (O(height) in general).

**2. Concatenation** (`concat(rope1, rope2)`): O(1)
```rust
Branch {
    left: rope1,
    right: rope2,
    length: rope1.len()
}
```

Simply create a new branch node—no copying of string data! This is why ropes excel: combining two documents is instant.

**3. Splitting** (`split(pos)`): O(log N)

Splitting at position `pos` creates two ropes. Navigate to the leaf containing `pos`:
- Split that leaf into two strings
- Reassemble ancestors, putting left parts in one rope and right parts in another

Example: Splitting "Hello, World!" at position 7:
```
Original:
        Branch(6)
        /        \
    Leaf("Hello")  Leaf(", World!")

Split at 7 (after "Hello, "):
Left rope:
        Branch(6)
        /        \
    Leaf("Hello")  Leaf(", ")

Right rope:
    Leaf("World!")
```

**4. Insertion** (`insert(pos, text)`): O(log N)
```rust
let (left, right) = rope.split(pos);
rope = concat(concat(left, from_str(text)), right);
```

This is the key insight: insertion is just split + concatenate! We create new tree nodes but reuse existing string leaves. The old tree still exists (structural sharing for undo).

**5. Deletion** (`delete(start, end)`): O(log N)
```rust
let (left, rest) = rope.split(start);
let (_, right) = rest.split(end - start);
rope = concat(left, right);
```

**Tree Rebalancing**:

Without rebalancing, a rope can degenerate into a linked list (height O(N)). Rebalancing maintains O(log N) height. Simple strategy:
- If leaf gets too large (>1KB), split it
- If tree gets too deep, rebuild by collecting all text and recreating a balanced tree
- Production implementations use more sophisticated techniques (red-black trees, weight-balanced trees)

**Structural Sharing for Undo**:

Old rope versions can coexist with new versions:
```rust
let v1 = rope.clone();          // Cheap: just Arc/Rc clone
rope.insert(100, "text");       // Creates new nodes, reuses old leaves
// v1 still valid, shares leaves with new rope
```

Undo is just reverting to the old rope reference—no need to "unapply" operations.

**Performance Characteristics**:
- **Insert/Delete**: O(log N) anywhere in document
- **Index access**: O(log N)
- **Concatenation**: O(1)
- **Split**: O(log N)
- **Iteration**: O(N) but with good cache locality if leaves are large
- **Memory**: O(N) for text + O(N/leaf_size) for tree nodes

**Trade-offs**:

| Operation | Gap Buffer | Rope |
|-----------|------------|------|
| Insert at cursor | O(1) | O(log N) |
| Insert random position | O(N) | O(log N) |
| Cursor movement | O(distance) | N/A (no cursor) |
| Large files | Poor | Excellent |
| Multiple cursors | Poor | Excellent |
| Undo/redo | O(N) copy | O(1) structural sharing |
| Memory overhead | Low | Moderate |
| Implementation complexity | Simple | Complex |

**When to Use**:
- **Gap Buffer**: Small documents, single cursor, simple implementation
- **Rope**: Large documents, multiple cursors, undo/redo, collaborative editing

**Key Concepts**:
- Rope is a binary tree of string fragments (leaves) and metadata (branches)
- All operations are O(log N) by navigating the tree
- Concatenation is O(1)—just create a branch node
- Structural sharing enables O(1) undo/redo without copying
- Rebalancing maintains O(log N) height for guaranteed performance
- Trades constant-time cursor insertion for logarithmic-time random access

---

## Pattern Matching Algorithms

String pattern matching is fundamental to text processing: searching source code, filtering logs, finding substrings in documents. Naive algorithms are O(NM) where N is text length and M is pattern length. Advanced algorithms reduce this to O(N+M) by preprocessing the pattern.

### Pattern 11: Knuth-Morris-Pratt (KMP) String Search

**Problem**: Naive string search backtracks in the text when a mismatch occurs, leading to O(NM) complexity. For large texts or patterns, this is unacceptable.

**Solution**: Preprocess the pattern to build a "failure function" that tells us where to resume matching after a mismatch, eliminating backtracking in the text.

**Why This Matters**: KMP guarantees O(N+M) performance. When searching genomic sequences (billions of bases) or log files (gigabytes), the difference between O(NM) and O(N+M) is dramatic.

**Use Case**: Text search, DNA sequence matching, compiler lexical analysis, intrusion detection.

```rust
//===========================================================
// Pattern: Knuth-Morris-Pratt (KMP) String Search (complete)
//===========================================================
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

**Algorithm Explanation**:

KMP's brilliance lies in its **failure function** (also called "partial match table" or "prefix function"). This preprocessed array tells us: "If we've matched `j` characters and then mismatch, how many characters from the pattern's start also appear at the current position?"

**Failure Function Construction** (`compute_failure`):

For pattern "ABABC":
```
Pattern:  A B A B C
Index:    0 1 2 3 4
Failure:  0 0 1 2 0
```

Explanation:
- `failure[0] = 0`: No proper prefix/suffix for single character
- `failure[1] = 0`: "AB" has no matching prefix/suffix
- `failure[2] = 1`: "ABA" has "A" as both prefix and suffix (length 1)
- `failure[3] = 2`: "ABAB" has "AB" as both prefix and suffix (length 2)
- `failure[4] = 0`: "ABABC" has no matching prefix/suffix

The algorithm builds this by:
1. Comparing `pattern[i]` with `pattern[j]` where `j` represents the length of the current matching prefix
2. If they match, increment `j` (extending the match)
3. If they don't match and `j > 0`, set `j = failure[j-1]` (try a shorter prefix)
4. Store the result: `failure[i] = j`

**Search Algorithm** (`find_all`):

When searching, we maintain `j` = number of pattern characters matched. On a mismatch:
- **Naive approach**: Reset `j` to 0 and continue
- **KMP approach**: Set `j = failure[j-1]`, utilizing overlap information

Example: Searching for "ABABC" in "ABABDABABC":
```
Text:    A B A B D A B A B C
Pattern: A B A B C
                ↑ mismatch at position 4

Instead of restarting from position 0, we know:
- We matched "ABAB" (4 chars)
- failure[3] = 2, meaning "AB" at start matches "AB" at position 2
- Resume matching from pattern[2], not pattern[0]

This eliminates redundant comparisons!
```

**Complexity Analysis**:
- **Preprocessing**: O(M) to build failure function
- **Search**: O(N) for text scan (never backtracks!)
- **Total**: O(N + M)
- **Space**: O(M) for failure array

The key insight: each text character is examined at most once. The `j` variable may decrease (via failure function), but the text index `i` only advances forward.

**Key Concepts**:
- Failure function encodes prefix/suffix overlap information
- No backtracking in text—each character examined once
- O(N + M) time complexity (optimal for string matching)
- Works well when pattern has internal repetition

---

### Pattern 12: Boyer-Moore String Search

**Problem**: KMP scans left-to-right and examines every text character. Can we do better by skipping sections of text?

**Solution**: Boyer-Moore scans the pattern **right-to-left** and uses heuristics to skip large portions of the text when mismatches occur.

**Why This Matters**: Boyer-Moore is often 3-5x faster than KMP in practice, especially for long patterns and large alphabets (like English text). It's the basis for `grep` and many text editors' search functions.

**Use Case**: Text editors, grep/search tools, bioinformatics, large document processing.

```rust
use std::collections::HashMap;

//====================================
// Boyer-Moore string search algorithm
//====================================
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

**Algorithm Explanation**:

Boyer-Moore's key insight: **scan the pattern right-to-left**. When you find a mismatch, you often have information that allows skipping multiple positions in the text.

**Bad Character Heuristic**:

The "bad character rule" says: when we mismatch at text character `c`, we can shift the pattern to align the rightmost occurrence of `c` in the pattern with the mismatched text position.

Example: Searching for "EXAMPLE" in text "...SIMPLE...":
```
Text:    S I M P L E
Pattern: E X A M P L E
         ↑ mismatch

The mismatched character is 'S'. Looking at pattern "EXAMPLE":
- 'S' doesn't appear in the pattern at all
- We can shift the entire pattern past 'S'
- This skips 7 positions in one step!

If 'S' appeared in the pattern, we'd align it:
Text:    S I M P L E
Pattern:     E X A M P L E (shifted to align any 'S')
```

**Bad Character Table Construction**:

For each character in the pattern, store its rightmost position:
```
Pattern: EXAMPLE
Table: E→6, L→5, P→4, M→3, A→2, X→1
```

When we mismatch on character `c` at pattern position `j`:
- If `c` is in the table at position `pos` and `pos < j`: shift `j - pos` positions
- If `c` is not in the table: shift `j` positions (entire pattern)
- If `pos >= j`: shift 1 position (to avoid negative shift)

**Right-to-Left Scanning**:

Scanning right-to-left is crucial. If the last character of the pattern doesn't match, we immediately know the pattern can't match at this position. The rightmost character is the most "discriminating"—if it's rare in the text, we skip many positions.

**Complexity Analysis**:
- **Best case**: O(N/M) — when last pattern character never matches, we skip M positions each time
- **Average case**: O(N) — significantly faster than KMP in practice
- **Worst case**: O(NM) — pathological patterns like "AAA...A" in text "AAA...A"
- **Preprocessing**: O(M + alphabet_size)

The full Boyer-Moore includes both "bad character" and "good suffix" heuristics. Our implementation uses bad character only, which is simpler but still very effective.

**Why It's Fast in Practice**:
- Long patterns benefit more (larger skips)
- Large alphabets (English: 26 letters) have better discrimination
- Real text isn't pathological—mismatches are common and allow big skips

**Key Concepts**:
- Right-to-left scanning maximizes information from mismatches
- Bad character rule enables large forward jumps
- Sublinear average-case performance (O(N/M) best case)
- Optimal for long patterns and large alphabets
- Trade-off: slightly more complex than KMP, but faster in practice

---

## String Interning

### Pattern 13: String Interning Pool
Reduce memory usage for repeated strings.
Using String interning with hash map

```rust
use std::collections::HashMap;
use std::sync::Arc;
//============================
// String Interning (complete)
//============================

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
