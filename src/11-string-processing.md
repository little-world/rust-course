# String Processing

Strings in Rust are more complex than in many languages due to UTF-8 encoding guarantees, ownership semantics, and platform interoperability requirements. `String` is a heap-allocated, growable UTF-8 string, while `&str` is an immutable view into UTF-8 data. Understanding the type system, encoding rules, and zero-copy techniques is essential for writing efficient text processing code.

 The key insight is that string type selection, allocation strategy, and understanding UTF-8's variable-length encoding dramatically impact both correctness and performance.



## Pattern 1: String Type Selection

**Problem**: Rust offers multiple string types (String, &str, Cow, OsString, Path), and choosing the wrong one leads to unnecessary allocations, API inflexibility, or platform compatibility issues. Developers face the ownership vs borrowing decision at every string boundary.

**Solution**: Use `String` for owned, growable UTF-8 strings when you need to build or modify text. Use `&str` for borrowed string slices—the most flexible function parameter type.

**Why It Matters**: Type choice determines allocation patterns and API ergonomics. Functions taking `&str` work with both `String` and `&str` due to deref coercion.

**Use Cases**: `String` for building strings dynamically, returning from functions, storing in collections. `&str` for function parameters, parsing, zero-copy operations. `Cow<str>` for normalization, escaping, sanitization where input is often already valid. `OsString` for file paths, environment variables, FFI with OS APIs. `Path` for cross-platform path manipulation with extension extraction, parent directory operations.

### Example: String - Owned and Growable

`String` is a heap-allocated, growable UTF-8 string that owns its data. Use it when you need to build strings dynamically or return strings from functions.

```rust
fn string_example() {
    let mut s = String::from("Hello");
    s.push_str(", World!");
    println!("{}", s);

    // Use when:
    // - Need to own the string
    // - Building strings dynamically
    // - Returning strings from functions
}
```

### Example: &str - Borrowed String Slice

`&str` is an immutable view into UTF-8 data without ownership. It's the most flexible type for function parameters since `String` automatically derefs to `&str`.

```rust
fn str_slice_example(s: &str) {
    println!("Length: {}", s.len());

    // Use when:
    // - Read-only access needed
    // - Function parameters (most flexible)
    // - String literals
}
```

### Example: Cow - Clone on Write

`Cow<str>` enables conditional allocation: borrow when possible, allocate only when modification is needed. This eliminates unnecessary allocations in common cases.

```rust
use std::borrow::Cow;

fn cow_example<'a>(data: &'a str, uppercase: bool) -> Cow<'a, str> {
    if uppercase {
        Cow::Owned(data.to_uppercase())  // Allocates
    } else {
        Cow::Borrowed(data)  // No allocation
    }
}
// Usage: conditionally allocate based on transformation needs
let s = cow_example("hello", false); // Cow::Borrowed, no allocation
let s = cow_example("hello", true);  // Cow::Owned("HELLO")
```

### Example: OsString/OsStr - Platform-Native Strings

Operating systems don't guarantee UTF-8. `OsString` and `OsStr` handle platform-specific encodings (UTF-16 on Windows, bytes on Unix) safely.

```rust
use std::ffi::{OsStr, OsString};

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
```

### Example: Path/PathBuf - Cross-Platform File Paths

`Path` and `PathBuf` wrap `OsStr`/`OsString` with path-specific operations and handle platform differences in path separators automatically.

```rust
use std::path::{Path, PathBuf};

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
```

### Example: Type Conversions

Understanding how to convert between string types is essential for working effectively with Rust's string ecosystem.

```rust
use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::Path;

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

## Pattern 2: String Builder Pattern

**Problem**: Concatenating strings in loops creates O(N²) complexity—each `s = s + "text"` allocates a new string and copies all previous content. Building HTML, SQL queries, or templates with repeated `push_str()` causes multiple reallocations when capacity is exceeded.

**Solution**: Pre-allocate capacity with `String::with_capacity(n)` when approximate size is known. Implement builder pattern with `&mut self` returns for method chaining.

**Why This Matters**: Pre-allocation eliminates reallocations—O(N) total time vs O(N) amortized with up to log(N) reallocations. A 10KB HTML document built with 100 appends: pre-allocated uses one 10KB buffer, non-pre-allocated copies ~20KB total (due to exponential growth). Method chaining creates fluent APIs that are both ergonomic and efficient. Builder pattern separates construction complexity from final immutable result.

**Use Cases**: HTML/XML generation (builders with tag methods, indentation), SQL query construction (type-safe builder preventing injection), log formatting (structured message building), template rendering (placeholder substitution), configuration file generation, protocol message assembly.


### Example: Basic StringBuilder with Capacity Pre-allocation

A simple string builder that pre-allocates capacity and provides method chaining for fluent APIs. The `build()` method consumes the builder to transfer ownership without copying.

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
// Usage: build strings with method chaining
let s = StringBuilder::with_capacity(100)
    .append("Hello").append(" World").build();
```

### Example: Domain-Specific HTML Builder

An HTML builder wraps StringBuilder with domain-specific methods for tag handling and automatic indentation, making HTML generation both safe and ergonomic.

```rust
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
// Usage: fluent HTML generation with automatic indentation
let html = HtmlBuilder::new()
    .open_tag("div").content("Hello").close_tag("div").build();
```

### Example: Using the Builders

Demonstrating the fluent API style enabled by method chaining with `&mut self` returns.

```rust
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

## Pattern 3: Zero-Copy String Operations

**Problem**: Parsing structured text (CSV, logs, configs) by creating owned `String` for each field wastes memory and CPU. A 10MB log file with 100K lines allocating strings for each line = 100K allocations + copying 10MB of data.

**Solution**: Use iterator methods that return `&str` slices borrowing from original data: `split()`, `lines()`, `split_whitespace()`. Pass slices to processing functions instead of collecting into vectors.

**Why It Matters**: Zero-copy parsing eliminates allocations entirely—10MB file processing uses 10MB (the file buffer) instead of 20MB (file + owned strings). String slicing is O(1)—just creating a fat pointer (2 words: pointer + length).

**Use Cases**: CSV/TSV parsing (split by delimiter, process fields), log analysis (pattern matching on lines), configuration file parsing (key=value splitting), protocol parsing (header extraction), text search (finding substrings without copying), streaming data processing (process-then-discard pattern).

### Example: Zero-Copy Line Parser

A parser that returns iterators over string slices without allocating. Each operation borrows from the original data, making parsing extremely efficient.

```rust
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
// Usage: zero-copy iteration over lines
let parser = LineParser::new("a,b,c\n1,2,3");
for line in parser.lines() { println!("{}", line); }
```

### Example: Zero-Allocation CSV Parser

A CSV parser that returns slices into the original data rather than allocating strings for each field. Ideal for streaming or one-pass processing.

```rust
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
// Usage: process CSV without intermediate allocations
let csv = CsvParser::new("a,b,c\n1,2,3");
csv.process(|fields| println!("{:?}", fields));
```

### Example: String View with UTF-8 Boundary Checking

A safe string view that validates UTF-8 character boundaries before slicing, preventing panics from invalid slices.

```rust
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
```

### Example: Using Zero-Copy Parsers

Demonstrating how to use zero-copy parsing techniques for efficient text processing without unnecessary allocations.

```rust
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

## Pattern 4: Cow for Conditional Allocation

**Problem**: Functions that sometimes modify input (escape HTML entities, normalize whitespace, strip prefixes) and sometimes don't face an allocation dilemma. Always allocating wastes memory when input is already valid.

**Solution**: Return `Cow<str>` which is either `Borrowed(&str)` or `Owned(String)`. Implement two-phase algorithm: fast-path check if modification needed (scan for special characters, extra whitespace, etc), then return `Cow::Borrowed(input)` for no-op case.

**Why This Matters**: The fast-path check (O(N) scan) is faster than allocation + copy. For already-normalized input (common in production), `Cow` returns immediately with zero allocation. Web server escaping HTML: if 90% of inputs have no special characters, `Cow` eliminates 90% of allocations. URL normalization processing millions of requests: `Cow` saves GB of allocations. Even worst case (modification needed) matches always-allocating performance while best case is 10-100x faster.

**Use Cases**: HTML escaping (only allocate if <>&"' present), whitespace normalization (only if multiple spaces found), case conversion (only if mixed case), prefix/suffix stripping (only if present), path canonicalization, validation with optional sanitization.

### Example: Normalize Whitespace with Cow

A two-phase algorithm: first check if normalization is needed, then only allocate if multiple consecutive spaces are found.

```rust
use std::borrow::Cow;

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
// Usage: normalize whitespace, allocating only if needed
let s1 = normalize_whitespace("hello world");   // Cow::Borrowed
let s2 = normalize_whitespace("hello  world");  // Cow::Owned("hello world")
```

### Example: Conditional HTML Escaping

Check if any special characters exist before allocating. Most strings don't need escaping, so this fast-path check saves allocations.

```rust
use std::borrow::Cow;

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
// Usage: escape HTML, skipping allocation when safe
let safe = escape_html("hello");        // Cow::Borrowed
let esc = escape_html("<script>");      // Cow::Owned("&lt;script&gt;")
```

### Example: Strip Prefix/Suffix Without Allocation

Only allocate a new string if prefix or suffix actually exists. Otherwise return the original slice.

```rust
use std::borrow::Cow;

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
```

### Example: Conditional Case Normalization

Check if the string is already lowercase before allocating for conversion. Avoids unnecessary work for already-normalized input.

```rust
use std::borrow::Cow;

fn to_lowercase_if_needed(s: &str) -> Cow<str> {
    if s.chars().all(|c| !c.is_uppercase()) {
        Cow::Borrowed(s)
    } else {
        Cow::Owned(s.to_lowercase())
    }
}
```

### Example: Using Cow Functions

Demonstrating how Cow eliminates allocations when input is already in the desired format.

```rust
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

## Pattern 5: UTF-8 Validation and Repair

**Problem**: UTF-8 is variable-length (1-4 bytes per character), but external data sources don't guarantee validity. Files may have encoding mismatches (Latin-1 labeled as UTF-8), network data can be corrupted, FFI receives arbitrary bytes.

**Solution**: Validate external data with `str::from_utf8()` for strict errors or `String::from_utf8_lossy()` to replace invalid sequences with �. Implement custom validators understanding UTF-8 structure (first byte determines length, continuation bytes match `10xxxxxx`, detect overlong encodings, reject surrogates).

**Why It Matters**: Invalid UTF-8 crashes programs or creates security vulnerabilities (overlong encodings bypass filters). A byte index mid-character panics: "hello"[1..4] is fine, "hëllo"[1..4] panics (ë is 2 bytes).

**Use Cases**: File I/O from unknown encodings, network protocol parsing, FFI with C strings, data recovery tools, text editors (proper cursor movement), terminal emulators (width calculation), web servers (sanitizing input), internationalization (proper truncation/reversal).

### Example: Lossy UTF-8 Validation

For handling potentially invalid data, `from_utf8_lossy` replaces invalid byte sequences with the replacement character (�). This is useful for recovery and logging.

```rust
fn validate_utf8_lossy(data: &[u8]) -> String {
    String::from_utf8_lossy(data).into_owned()
}
// Usage: convert bytes to string, replacing invalid UTF-8
let s = validate_utf8_lossy(&[0x48, 0x65, 0xFF, 0x6C]); // "He�l"
```

### Example: Strict UTF-8 Validation

When you need to know if data is valid UTF-8, use `from_utf8` which returns an error for invalid sequences.

```rust
fn validate_utf8_strict(data: &[u8]) -> Result<&str, std::str::Utf8Error> {
    std::str::from_utf8(data)
}
// Usage: validate UTF-8, returning error on invalid bytes
let valid = validate_utf8_strict(b"hello");   // Ok("hello")
let invalid = validate_utf8_strict(&[0xFF]); // Err(Utf8Error)
```

### Example: Custom UTF-8 Validator

A comprehensive UTF-8 validator that detects overlong encodings and provides detailed error positions. This demonstrates the complete UTF-8 validation algorithm.

```rust
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
            let ch = ((first as u32 & 0x1F) << 6)
                | (second as u32 & 0x3F);
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

```

### Example: Using UTF-8 Validators

Demonstrating validation strategies for different scenarios: strict validation, lossy conversion, and detailed error reporting.

```rust
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


## Pattern 6: Character and Grapheme Iteration

**Problem**: What users perceive as "one character" isn't what Rust's `.chars()` sees. Emoji like "👨‍👩‍👧‍👦" (family) are multiple code points joined with Zero-Width Joiners.

**Solution**: Understand three iteration levels: bytes (`.bytes()`), characters (`.chars()`), graphemes (`unicode-segmentation::graphemes()`). Use bytes only for serialization/protocols.

**Why It Matters**: Byte iteration sees 4 bytes for "👋", char iteration sees 1 code point, grapheme iteration sees 1 user-perceived character. But "👨‍👩‍👧‍👦" is 7 code points (4 emoji + 3 ZWJ), 1 grapheme.

**Use Cases**: Text editors (cursor movement across graphemes), terminal output (width calculation for alignment), string truncation (safe at grapheme boundaries), text reversal (preserve composed characters), search/replace (whole grapheme), length display (user-perceived count not bytes), internationalization.

### Example: Three Levels of String Iteration

Demonstrating the difference between bytes, characters (code points), and grapheme clusters—each serves different purposes.

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
```

### Example: Display Width Calculation

Implements East Asian Width rules for terminal alignment. CJK characters and fullwidth forms occupy two columns.

```rust
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
```

### Example: Safe String Truncation

Truncating at character boundaries prevents corrupting multi-byte UTF-8 sequences. Uses `char_indices()` to find safe cut points.

```rust
fn truncate_at_char(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
```

### Example: Grapheme-Aware Truncation

For user-facing text, truncate at grapheme cluster boundaries to avoid breaking composed characters or emoji sequences.

```rust
use unicode_segmentation::UnicodeSegmentation;

fn truncate_at_grapheme(s: &str, max_graphemes: usize) -> String {
    s.graphemes(true)
        .take(max_graphemes)
        .collect()
}
```

### Example: Reversing While Preserving Graphemes

String reversal that keeps grapheme clusters intact—essential for text editors and international text processing.

```rust
use unicode_segmentation::UnicodeSegmentation;

fn reverse_graphemes(s: &str) -> String {
    s.graphemes(true).rev().collect()
}
```

### Example: Using Character Iteration Techniques

Demonstrating different iteration levels on ASCII, multi-byte characters, complex emoji, and CJK text.

```rust
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


## Pattern 7: String Parsing State Machines

**Problem**: Parsing structured text (source code, protocols, markup) with simple string methods leads to complex nested loops and fragile conditional logic. Tracking whether you're "inside a string" or "in a comment" requires multiple boolean flags.

**Solution**: Model the parser as a finite state machine with explicit states (Start, InString, InComment, InNumber, etc.). Each state handles specific characters and transitions to new states.

**Why It Matters**: State machines make parsing logic explicit and declarative—you can visualize the automaton on paper. Adding a new token type means adding a state and transitions, not hunting through conditionals.

**Use Cases**: Lexers for programming languages (keywords, operators, literals, comments), protocol parsers (HTTP headers, binary formats), markup languages (Markdown, HTML fragments), configuration file parsers (TOML, INI), CSV/TSV with escaping, syntax highlighting (real-time tokenization), code completion (incomplete token recovery), template languages (text + interpolation), log parsing (structured formats).

### Examples

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
        let ops = ["==", "!=", "<=", ">=", "&&", "||"];
        if ops.contains(&two_char.as_str()) {
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
        let keywords = [
            "if", "else", "while", "for", "return", "fn", "let"
        ];

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



## Pattern 8: URL Parser State Machine

**Problem**: Parsing URLs with regex or simple string splits is fragile and error-prone. URLs have complex structure (scheme, authority, path, query, fragment) with context-dependent delimiters—":" after scheme vs in authority, "//" for authority vs "/" for path, "?" for query, "#" for fragment.

**Solution**: Use a state machine with explicit states (Scheme, AfterScheme, Authority, Path, Query, Fragment) that transitions based on delimiter characters. Parse in one pass, accumulating characters into buffers.

**Why It Matters**: State machine parsing is faster (single pass), more maintainable (explicit transitions), and RFC-compliant (handles all edge cases). Adding support for new URL schemes or components means adding states/transitions, not rewriting regex.

**Use Cases**: Web frameworks (route parsing, request URL decomposition), HTTP clients (validating and normalizing URLs), web scraping (extracting links, resolving relative URLs), URL shorteners (parsing and validating input), API gateways (routing based on URL structure), browser implementations (address bar parsing), link checkers (validating URL format), sitemap generators, OAuth redirect URI validation.

### Examples

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
                    } else if ch.is_alphanumeric()
                        || ch == '+' || ch == '-' || ch == '.' {
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


## Pattern 9: Gap Buffer Implementation
**Problem**: Text editors need fast insertion/deletion at cursor. `Vec<char>` requires O(N) time for insert—all characters after cursor shift.

**Solution**: Use gap buffer for single-cursor editors: maintain "gap" at cursor position, insertion fills gap O(1), cursor movement slides gap O(distance). When gap full, grow buffer (double size).

**Why It Matters**: Gap buffer: typing is O(1) amortized, but moving cursor across document is O(N) worst case. Rope: all operations O(log N), handles GB files, multiple cursors efficiently, undo/redo via structural sharing (no copying).

**Use Cases**: Gap buffer for simple single-user editors with localized editing, command-line text input, undo buffers for small documents. Rope for modern editors (VS Code, Sublime), large log file viewers, collaborative editing (multiple users), version control diffs, syntax highlighting (parsing unchanged after local edit), mobile text editors (memory constrained).

### Examples

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



### Rope Data Structure

**Problem**: Gap buffers struggle with: - Large documents (multi-megabyte files cause O(N) cursor movement) - Multiple cursors (each needs its own gap) - Undo/redo (copying entire buffer is expensive)

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
            Rope::Branch { length, right, .. } => {
                length + right.len()
            }
        }
    }

    // Insert string at position
    fn insert(&mut self, pos: usize, text: &str) {
        let current = std::mem::replace(self, Rope::Leaf(String::new()));
        let (left, right) = current.split(pos);
        let inner = Rope::concat(left, Rope::from_str(text));
        *self = Rope::concat(inner, right);
    }

    // Delete range
    fn delete(&mut self, start: usize, end: usize) {
        let current = std::mem::replace(self, Rope::Leaf(String::new()));
        let (left, rest) = current.split(start);
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
                    (Rope::Leaf(left.to_string()),
                     Rope::Leaf(right.to_string()))
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
        Rope::concat(
            Rope::Leaf(left.to_string()),
            Rope::Leaf(right.to_string()))
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

## Pattern 10: Knuth-Morris-Pratt (KMP) String Search

**Problem**: Naive string search is O(NM) where N = text length, M = pattern length. Searching 1GB log file for 100-byte pattern = 10^9 * 100 comparisons worst case.

**Solution**: Use KMP (Knuth-Morris-Pratt) for guaranteed O(N+M) with no text backtracking via preprocessed "failure function". Use Boyer-Moore for practical O(N/M) best case by scanning pattern right-to-left and skipping sections.

**Why It Matters**: KMP guarantees linear time—1GB file with 1KB pattern: naive = 10^12 ops worst case, KMP = 10^9 ops always. Boyer-Moore often 3-5x faster than KMP in practice (especially long patterns, large alphabets)—grep and text editors use it.

**Use Cases**: Text search in editors (Boyer-Moore for interactive search), genomic analysis (KMP for ATCG sequences), log filtering (pattern matching millions of lines), compiler lexical analysis (token recognition), intrusion detection (packet payload scanning), plagiarism detection (document comparison), virus scanning.

### Examples 

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

## Pattern 11: Boyer-Moore String Search

**Problem**: KMP scans every text character left-to-right, no skipping possible. Can we do better?

**Solution**: Scan pattern right-to-left (most discriminating character first). Build "bad character table" mapping each character to its rightmost position in pattern.

**Why This Matters**: Best case O(N/M)—searching for "PATTERN" (7 chars) in text where "N" never appears skips 7 positions per comparison, examining only N/7 characters! Average case much faster than KMP for long patterns and large alphabets. English text search: Boyer-Moore 3-5x faster than KMP. DNA search (4-letter alphabet): KMP competitive. This is why grep, Vim, and text editors use Boyer-Moore variants.

**Use Cases**: Interactive text search in editors (fast visual feedback), grep/ripgrep tools (searching codebases), plagiarism detection (comparing documents), bioinformatics (protein sequence search has 20-letter alphabet), large document search (legal discovery, log analysis), web page search.

### Examples 

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
                    let skip = self.bad_char.get(&next_char).unwrap_or(&0);
                    s += m - skip;
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

## Pattern 12: String Interning

**Problem**: Programs with repeated strings waste memory—a web server with 10K sessions storing "logged_in" user state = 10K copies of same string. Compilers store identifiers thousands of times.

**Solution**: Implement string interning pool using `HashMap<Arc<str>, Arc<str>>` to deduplicate. When interning a string: check if already in pool, return existing `Arc` (cheap clone); if not in pool, insert new `Arc` and return it.

**Why It Matters**: Memory: 1M instances of "error" = 5MB as separate strings, 5 bytes + overhead as interned. String comparison: O(N) for string equality, O(1) for Arc pointer equality.

**Use Cases**: Compiler symbol tables (variable names, type names repeated in AST), configuration systems (keys like "database.host" repeated), logging (level strings "ERROR"/"INFO" everywhere), web frameworks (route paths, session keys), game engines (asset tags, entity names), network protocols (header names, status codes), i18n (translation keys).

### Examples

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




### String Types

| Type | Owned | UTF-8 | Use Case |
|------|-------|-------|----------|
| `String` | Yes | Yes | Dynamic, growable strings |
| `&str` | No | Yes | String slices, literals |
| `Cow<str>` | Sometimes | Yes | Clone-on-write optimization |
| `OsString` | Yes | No | Platform strings, file paths |
| `Path` | No | No | File system paths |


### Performance Tips

1. **Pre-allocate**: Use `String::with_capacity()` when size known
2. **Avoid cloning**: Use `&str` parameters instead of `String`
3. **Use Cow**: For conditional modifications
4. **Intern strings**: For repeated strings
5. **Chars not bytes**: Use `char_indices()` for UTF-8 safety


### Summary

1. **String Type Selection**: String (owned), &str (borrowed), Cow (conditional), OsString (platform), Path (filesystem)
2. **Builder Patterns**: Pre-allocate capacity, method chaining, domain-specific builders for HTML/SQL
3. **Zero-Copy Operations**: Iterator methods returning &str slices, Cow for conditional allocation
4. **UTF-8 Handling**: Validation, character boundaries, grapheme clusters for display
5. **Text Editor Data Structures**: Gap buffers O(1) cursor insert, ropes O(log N) everywhere
6. **Pattern Matching**: KMP O(N+M) guaranteed, Boyer-Moore O(N/M) best case
7. **String Interning**: Arc-based deduplication, O(1) comparison, memory savings

**Key Takeaways**:
- Type choice determines allocation: &str for parameters, String for owned, Cow for conditional
- Pre-allocation is O(N) vs O(N) amortized with log(N) reallocations
- Zero-copy parsing: 10MB file = 10MB memory (not 20MB with owned strings)
- UTF-8 has three levels: bytes < characters < graphemes (use appropriate level)
- Gap buffer for simple editors, rope for production features (multi-cursor, undo, large files)
- KMP for guaranteed linear time, Boyer-Moore 3-5x faster in practice
- Intern repeated strings: 1M copies of "error" = 5 bytes + overhead vs 5MB

**Performance Guidelines**:
- String building: pre-allocate capacity when size known or estimable
- Parsing: use zero-copy iterators (split, lines), collect only when necessary
- UTF-8: use char_indices() for boundaries, graphemes for display, bytes for protocols
- Text editing: gap buffer < 1MB, rope > 1MB or multiple cursors
- Search: Boyer-Moore for interactive (large alphabet), KMP for guaranteed (small alphabet)
- Comparison: intern if comparing same strings repeatedly


**Safety Notes**:
- Never index strings with byte positions without checking char boundaries
- Validate UTF-8 from external sources (from_utf8, from_utf8_lossy)
- Use grapheme clusters for user-facing operations (truncation, reversal, width)
- Cow scans input twice (check + build) but faster than always allocating
- Arc clones are cheap (refcount increment) but not free—benchmark if critical
