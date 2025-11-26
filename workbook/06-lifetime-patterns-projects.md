# Chapter 6: Lifetime Patterns - Programming Projects

## Project 1: Zero-Copy Parser with Lifetime Management

### Problem Statement

Build a high-performance text parser that:
- Parses input without allocating (zero-copy approach)
- Returns string slices that borrow from the original input
- Supports nested parsing contexts with multiple lifetime relationships
- Implements streaming iteration over parsed tokens
- Handles complex lifetime relationships (parser outliving input, tokens tied to parser)
- Provides both owned and borrowed parsing modes
- Uses lifetime elision where possible, explicit lifetimes where necessary
- Achieves O(1) token extraction (no copying, just slicing)

The parser must demonstrate proper lifetime management while maintaining ergonomics and performance.

### Why It Matters

Zero-copy parsing is critical for performance:
- **Web Servers**: Parsing HTTP requests without allocation
- **Compilers**: Tokenizing source code efficiently
- **Databases**: Query parsing without string copies
- **Log Processing**: Extracting fields from millions of log lines
- **Protocol Handlers**: Network protocol parsing at wire speed

Improper lifetime management leads to:
- Unnecessary allocations (copying when borrowing would work)
- Dangling pointers (returning references to dropped data)
- Rigid APIs (can't express flexible lifetime relationships)
- Performance degradation (allocation pressure on hot paths)

### Use Cases

1. **HTTP Request Parsing**: Extract headers, body without copying
2. **CSV/TSV Parsers**: Tokenize fields as string slices
3. **Configuration Files**: Parse INI/TOML without allocation
4. **Log Analysis**: Extract timestamp, level, message from log lines
5. **JSON Streaming**: Parse large JSON without loading entire document
6. **Command-Line Parsers**: Split arguments as borrowed slices
7. **Network Protocols**: Parse packet headers as slices

### Solution Outline

**Core Structure:**
```rust
// Parser borrows input for lifetime 'a
pub struct Parser<'a> {
    input: &'a str,
    position: usize,
}

// Tokens borrow from parser (and transitively from input)
pub struct Token<'a> {
    kind: TokenKind,
    text: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self { /* ... */ }

    pub fn next_token(&mut self) -> Option<Token<'a>> { /* ... */ }

    pub fn peek(&self) -> Option<&'a str> { /* ... */ }
}

// Streaming iterator that yields borrowed tokens
pub struct TokenIterator<'a> {
    parser: Parser<'a>,
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_token()
    }
}
```

**Key Lifetime Patterns:**
- **Lifetime Elision**: `fn peek(&self) -> Option<&str>` inferred as `&'a str`
- **Explicit Lifetimes**: When returning borrowed data from multiple sources
- **Lifetime Bounds**: Ensuring generic parsers have correct constraints
- **Multiple Lifetimes**: Parser context with independent lifetime scopes

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_zero_copy_parsing() {
    let input = "hello world foo bar";
    let parser = Parser::new(input);

    let tokens: Vec<_> = parser.into_iter().collect();

    // Verify tokens are slices of original input
    assert_eq!(tokens[0].text.as_ptr(), input.as_ptr());
    assert_eq!(tokens[1].text.as_ptr(), unsafe { input.as_ptr().add(6) });
}

#[test]
fn test_lifetime_relationships() {
    let input = String::from("test data");
    let parser = Parser::new(&input);

    let token = parser.next_token().unwrap();

    // Token borrows from input
    drop(parser);
    // token still valid - borrows from input, not parser
    assert_eq!(token.text, "test");
}
```

**Benchmark Tests:**
```rust
#[bench]
fn bench_zero_copy_vs_owned(b: &mut Bencher) {
    let input = "word1 word2 word3 ..."; // Large input

    b.iter(|| {
        let parser = Parser::new(input);
        let count: usize = parser.into_iter().map(|t| t.text.len()).sum();
        black_box(count);
    });
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Parser with Owned Strings

**Goal:** Create a working parser that allocates for each token.

**What to implement:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Word,
    Number,
    Whitespace,
    Punctuation,
    EndOfFile,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,  // Owned!
    pub position: usize,
}

pub struct Parser {
    input: String,     // Owned!
    position: usize,
}

impl Parser {
    pub fn new(input: String) -> Self {
        Parser {
            input,
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return None;
        }

        let start = self.position;
        let ch = self.current_char()?;

        let kind = if ch.is_alphabetic() {
            self.consume_while(|c| c.is_alphanumeric());
            TokenKind::Word
        } else if ch.is_numeric() {
            self.consume_while(|c| c.is_numeric());
            TokenKind::Number
        } else {
            self.position += ch.len_utf8();
            TokenKind::Punctuation
        };

        let text = self.input[start..self.position].to_string(); // Allocation!

        Some(Token {
            kind,
            text,
            position: start,
        })
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.position += ch.len_utf8();
        }
    }

    fn consume_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(ch) = self.current_char() {
            if !predicate(ch) {
                break;
            }
            self.position += ch.len_utf8();
        }
    }
}
```

**Check/Test:**
- Test parsing "hello world 123"
- Test different token types
- Verify positions are correct
- Test UTF-8 handling (emoji, accented characters)

**Why this isn't enough:**
Every token allocates a new `String`, creating memory pressure. For a 1MB log file with 100k tokens, we're allocating 100k strings—each requiring heap allocation, growing, copying. The allocator becomes a bottleneck. Profiling shows ~60% of CPU time in malloc/free. We're also forcing owned input, preventing zero-copy from memory-mapped files or network buffers. We need borrowing.

---

### Step 2: Add Lifetimes for Zero-Copy Parsing

**Goal:** Use lifetimes to borrow from input instead of allocating.

**What to improve:**
```rust
// Token now borrows with lifetime 'a
#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,  // Borrowed!
    pub position: usize,
}

// Parser borrows input with lifetime 'a
pub struct Parser<'a> {
    input: &'a str,     // Borrowed!
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            input,
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return None;
        }

        let start = self.position;
        let ch = self.current_char()?;

        let kind = if ch.is_alphabetic() {
            self.consume_while(|c| c.is_alphanumeric());
            TokenKind::Word
        } else if ch.is_numeric() {
            self.consume_while(|c| c.is_numeric());
            TokenKind::Number
        } else {
            self.position += ch.len_utf8();
            TokenKind::Punctuation
        };

        // Just slice - no allocation!
        let text = &self.input[start..self.position];

        Some(Token {
            kind,
            text,
            position: start,
        })
    }

    // Lifetime elision: inferred as &'a self -> Option<&'a str>
    pub fn peek(&self) -> Option<&str> {
        if self.position >= self.input.len() {
            None
        } else {
            Some(&self.input[self.position..])
        }
    }

    // Explicit lifetime when needed
    pub fn remaining(&self) -> &'a str {
        &self.input[self.position..]
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.position += ch.len_utf8();
        }
    }

    fn consume_while<F>(&mut self, predicate: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(ch) = self.current_char() {
            if !predicate(ch) {
                break;
            }
            self.position += ch.len_utf8();
        }
    }
}
```

**Check/Test:**
- Verify tokens are slices (check pointer addresses)
- Test that tokens borrow from input (not parser)
- Test lifetime: input must outlive tokens
- Benchmark: should be 10-100x faster than owned version
- Test cannot use token after input is dropped (compile error)

**Why this isn't enough:**
While we've eliminated allocations, the API is still basic. Real parsers need context—tracking line numbers, handling errors with position info, supporting backtracking. We also can't iterate over tokens ergonomically. The `next_token()` approach requires manual looping. We need iterator support and richer context, but these add lifetime complexity.

---

### Step 3: Add Iterator Support and Parse Context

**Goal:** Implement `Iterator` for ergonomic token consumption and add parsing context.

**What to improve:**

**1. Token iterator:**
```rust
pub struct TokenIterator<'a> {
    parser: Parser<'a>,
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.next_token()
    }
}

impl<'a> IntoIterator for Parser<'a> {
    type Item = Token<'a>;
    type IntoIter = TokenIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator { parser: self }
    }
}
```

**2. Parse context with lifetime:**
```rust
pub struct ParseContext<'a> {
    pub source: &'a str,
    pub filename: Option<&'a str>,
    pub line_offsets: Vec<usize>,
}

impl<'a> ParseContext<'a> {
    pub fn new(source: &'a str, filename: Option<&'a str>) -> Self {
        let line_offsets = Self::compute_line_offsets(source);
        ParseContext {
            source,
            filename,
            line_offsets,
        }
    }

    fn compute_line_offsets(source: &str) -> Vec<usize> {
        std::iter::once(0)
            .chain(source.match_indices('\n').map(|(pos, _)| pos + 1))
            .collect()
    }

    pub fn line_col(&self, position: usize) -> (usize, usize) {
        let line = self.line_offsets
            .binary_search(&position)
            .unwrap_or_else(|x| x.saturating_sub(1));

        let col = position - self.line_offsets[line];
        (line + 1, col + 1)
    }

    pub fn get_line(&self, position: usize) -> &'a str {
        let (line_num, _) = self.line_col(position);
        let start = self.line_offsets[line_num - 1];
        let end = self.line_offsets
            .get(line_num)
            .copied()
            .unwrap_or(self.source.len());

        &self.source[start..end.saturating_sub(1)]
    }
}

// Enhanced parser with context
pub struct ContextualParser<'a> {
    context: &'a ParseContext<'a>,
    position: usize,
}

impl<'a> ContextualParser<'a> {
    pub fn new(context: &'a ParseContext<'a>) -> Self {
        ContextualParser {
            context,
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        // Same tokenization logic but using context.source
        // Returns tokens that borrow from context.source
        todo!()
    }

    pub fn current_location(&self) -> Location<'a> {
        let (line, col) = self.context.line_col(self.position);
        Location {
            line,
            col,
            source_line: self.context.get_line(self.position),
            filename: self.context.filename,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Location<'a> {
    pub line: usize,
    pub col: usize,
    pub source_line: &'a str,
    pub filename: Option<&'a str>,
}
```

**3. Error handling with borrowed context:**
```rust
#[derive(Debug, Clone)]
pub struct ParseError<'a> {
    pub message: String,
    pub location: Location<'a>,
}

impl<'a> ParseError<'a> {
    pub fn new(message: String, location: Location<'a>) -> Self {
        ParseError { message, location }
    }

    pub fn format(&self) -> String {
        format!(
            "{}:{}:{}: {}\n  {}\n  {}^",
            self.location.filename.unwrap_or("<input>"),
            self.location.line,
            self.location.col,
            self.message,
            self.location.source_line,
            " ".repeat(self.location.col.saturating_sub(1))
        )
    }
}
```

**Usage:**
```rust
let source = "hello world\n123 456\nfoo bar";
let context = ParseContext::new(source, Some("test.txt"));
let parser = ContextualParser::new(&context);

for token in parser {
    println!("{:?} at line {}", token.text, token.location.line);
}
```

**Check/Test:**
- Test iterator collects all tokens
- Test line/column calculation
- Test error formatting with context
- Verify all borrowed data ties back to original input
- Test that context outlives parser and tokens

**Why this isn't enough:**
We have iteration and context, but no support for complex parsing patterns. Real parsers need:
- **Lookahead**: Peek ahead without consuming
- **Backtracking**: Try one parse, rewind if it fails
- **Nested contexts**: Parse expressions within statements
- **Multiple independent lifetimes**: Parser state vs input data

These patterns require more sophisticated lifetime management—multiple lifetime parameters and careful bounds.

---

### Step 4: Add Backtracking and Multiple Lifetime Parameters

**Goal:** Support parser combinators with backtracking using multiple lifetimes.

**What to improve:**

**1. Parser state with checkpoint:**
```rust
#[derive(Debug, Clone, Copy)]
pub struct Checkpoint {
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            position: self.position,
        }
    }

    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.position = checkpoint.position;
    }

    pub fn try_parse<F, T>(&mut self, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> Option<T>,
    {
        let checkpoint = self.checkpoint();
        match f(self) {
            Some(result) => Some(result),
            None => {
                self.restore(checkpoint);
                None
            }
        }
    }
}
```

**2. Combinator with multiple lifetimes:**
```rust
// Parser with two independent lifetimes:
// 'input: lifetime of the input being parsed
// 'parser: lifetime of the parser itself
pub struct CombinatorParser<'input, 'parser> {
    input: &'input str,
    position: usize,
    // Parser can hold references to external state
    keywords: &'parser [&'static str],
}

impl<'input, 'parser> CombinatorParser<'input, 'parser> {
    pub fn new(input: &'input str, keywords: &'parser [&'static str]) -> Self {
        CombinatorParser {
            input,
            position: 0,
            keywords,
        }
    }

    // Returns token tied to 'input lifetime, not 'parser
    pub fn next_token(&mut self) -> Option<Token<'input>> {
        // Parse token from self.input
        todo!()
    }

    // Parse keyword - returns borrowed from 'input
    pub fn parse_keyword(&mut self) -> Option<&'input str> {
        for keyword in self.keywords {
            if self.input[self.position..].starts_with(keyword) {
                let start = self.position;
                self.position += keyword.len();
                return Some(&self.input[start..self.position]);
            }
        }
        None
    }

    // Peek uses 'input lifetime (from elision rule)
    pub fn peek(&self) -> Option<&str> {
        if self.position < self.input.len() {
            Some(&self.input[self.position..])
        } else {
            None
        }
    }
}
```

**3. Parser combinators:**
```rust
// Higher-order function accepting closure with lifetime constraints
pub fn many<'a, F, T>(parser: &mut Parser<'a>, mut f: F) -> Vec<T>
where
    F: FnMut(&mut Parser<'a>) -> Option<T>,
{
    let mut results = Vec::new();
    while let Some(item) = f(parser) {
        results.push(item);
    }
    results
}

pub fn separated<'a, F, S, T>(
    parser: &mut Parser<'a>,
    mut item_parser: F,
    mut sep_parser: S,
) -> Vec<T>
where
    F: FnMut(&mut Parser<'a>) -> Option<T>,
    S: FnMut(&mut Parser<'a>) -> Option<()>,
{
    let mut results = Vec::new();

    if let Some(first) = item_parser(parser) {
        results.push(first);

        while sep_parser(parser).is_some() {
            if let Some(item) = item_parser(parser) {
                results.push(item);
            } else {
                break;
            }
        }
    }

    results
}

pub fn optional<'a, F, T>(parser: &mut Parser<'a>, f: F) -> Option<T>
where
    F: FnOnce(&mut Parser<'a>) -> Option<T>,
{
    parser.try_parse(f)
}
```

**4. Complex example with lifetime bounds:**
```rust
// Parse structure with lifetime bounds
pub struct Parsed<'input> {
    pub tokens: Vec<Token<'input>>,
    pub source: &'input str,
}

// Generic parse function with lifetime bounds
pub fn parse_with_context<'input, 'ctx, F, T>(
    input: &'input str,
    context: &'ctx ParseContext<'ctx>,
    parser_fn: F,
) -> Result<T, ParseError<'input>>
where
    F: FnOnce(&mut Parser<'input>) -> Option<T>,
{
    let mut parser = Parser::new(input);
    parser_fn(&mut parser)
        .ok_or_else(|| ParseError {
            message: "Parse failed".to_string(),
            location: Location {
                line: 1,
                col: parser.position,
                source_line: input,
                filename: context.filename,
            },
        })
}
```

**Usage:**
```rust
let input = "if foo then bar else baz";
let keywords = ["if", "then", "else"];
let mut parser = CombinatorParser::new(input, &keywords);

// Parse with backtracking
let result = parser.try_parse(|p| {
    let keyword = p.parse_keyword()?;
    let condition = p.next_token()?;
    Some((keyword, condition))
});

// Use combinators
let tokens = many(&mut parser, |p| p.next_token());
```

**Check/Test:**
- Test backtracking restores position correctly
- Test combinators with various inputs
- Test multiple lifetime parameters compile correctly
- Test that tokens borrow from input, not parser state
- Verify keyword list can have different lifetime than input

**Why this isn't enough:**
We have backtracking and combinators, but performance suffers from repeated parsing attempts. When parsing complex grammars with deep nesting, backtracking can cause exponential blowup. We also don't have memoization (remembering parse results at positions). For production parsers, we need optimizations like:
- Memoization (packrat parsing)
- Lookahead optimization
- Error recovery instead of backtracking

Let's add these while maintaining lifetime correctness.

---

### Step 5: Add Memoization and Streaming Iterator

**Goal:** Implement memoization for performance and streaming iteration for memory efficiency.

**What to improve:**

**1. Memoized parser with lifetime-correct cache:**
```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Cache key: (rule ID, position)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    rule: usize,
    position: usize,
}

// Cached result with borrowed data
#[derive(Debug, Clone)]
enum CacheEntry<'a, T> {
    Success {
        result: T,
        new_position: usize,
    },
    Failure,
}

pub struct MemoizedParser<'input> {
    input: &'input str,
    position: usize,
    // Cache stores parse results
    cache: HashMap<CacheKey, Box<dyn std::any::Any>>,
}

impl<'input> MemoizedParser<'input> {
    pub fn new(input: &'input str) -> Self {
        MemoizedParser {
            input,
            position: 0,
            cache: HashMap::new(),
        }
    }

    pub fn memoized<F, T>(&mut self, rule_id: usize, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> Option<T>,
        T: Clone + 'static,
    {
        let key = CacheKey {
            rule: rule_id,
            position: self.position,
        };

        // Check cache
        if let Some(cached) = self.cache.get(&key) {
            if let Some(entry) = cached.downcast_ref::<CacheEntry<'input, T>>() {
                return match entry {
                    CacheEntry::Success { result, new_position } => {
                        self.position = *new_position;
                        Some(result.clone())
                    }
                    CacheEntry::Failure => None,
                };
            }
        }

        // Not in cache, parse
        let start_position = self.position;
        let result = f(self);

        // Cache result
        let entry: Box<dyn std::any::Any> = match &result {
            Some(value) => Box::new(CacheEntry::Success {
                result: value.clone(),
                new_position: self.position,
            }),
            None => {
                self.position = start_position; // Restore on failure
                Box::new(CacheEntry::<T>::Failure)
            }
        };

        self.cache.insert(key, entry);
        result
    }
}
```

**2. Streaming iterator (lending iterator):**
```rust
// Streaming iterator where items borrow from the iterator
pub trait StreamingIterator {
    type Item<'a> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

// Window iterator that yields overlapping slices
pub struct WindowIterator<'a> {
    input: &'a str,
    window_size: usize,
    position: usize,
}

impl<'a> WindowIterator<'a> {
    pub fn new(input: &'a str, window_size: usize) -> Self {
        WindowIterator {
            input,
            window_size,
            position: 0,
        }
    }
}

impl<'a> StreamingIterator for WindowIterator<'a> {
    type Item<'b> = &'b str where Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position + self.window_size <= self.input.len() {
            let window = &self.input[self.position..self.position + self.window_size];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}

// Streaming token iterator
pub struct StreamingTokenIterator<'input> {
    parser: Parser<'input>,
    // Can't use buffer: Vec<Token<'input>> because Token borrows from parser
    // Instead, reparse on each next() call
}

impl<'input> StreamingIterator for StreamingTokenIterator<'input> {
    type Item<'a> = Token<'a> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.parser.next_token()
    }
}
```

**3. Efficient lookahead with lifetime constraints:**
```rust
pub struct LookaheadParser<'input> {
    input: &'input str,
    position: usize,
    // Lookahead buffer: pre-parsed tokens
    lookahead_buffer: Vec<Token<'input>>,
    lookahead_position: usize,
}

impl<'input> LookaheadParser<'input> {
    pub fn new(input: &'input str) -> Self {
        LookaheadParser {
            input,
            position: 0,
            lookahead_buffer: Vec::new(),
            lookahead_position: 0,
        }
    }

    pub fn peek_nth(&mut self, n: usize) -> Option<&Token<'input>> {
        // Fill buffer up to n+1 tokens
        while self.lookahead_buffer.len() <= n {
            if let Some(token) = self.parse_next_token() {
                self.lookahead_buffer.push(token);
            } else {
                break;
            }
        }

        self.lookahead_buffer.get(n)
    }

    pub fn consume(&mut self) -> Option<Token<'input>> {
        if self.lookahead_position < self.lookahead_buffer.len() {
            let token = self.lookahead_buffer[self.lookahead_position].clone();
            self.lookahead_position += 1;
            Some(token)
        } else {
            self.parse_next_token()
        }
    }

    fn parse_next_token(&mut self) -> Option<Token<'input>> {
        // Token parsing logic
        todo!()
    }
}
```

**Check/Test:**
- Test memoization prevents reparsing
- Benchmark: memoized parser faster on complex grammars
- Test streaming iterator doesn't hold all tokens in memory
- Test lookahead buffer with various peek depths
- Verify cache entries maintain correct lifetimes

**Why this isn't enough:**
We have performance optimizations, but error handling is still basic. Production parsers need:
- Rich error messages with expected vs actual
- Error recovery (continue parsing after errors)
- Multiple error reporting (don't stop at first error)
- Suggestion generation ("did you mean X?")

These features require careful lifetime management to store error context without copying. Let's add comprehensive error handling.

---

### Step 6: Add Rich Error Handling with Lifetime-Correct Error Context

**Goal:** Implement production-grade error reporting with borrowed error context.

**What to improve:**

**1. Rich error types:**
```rust
#[derive(Debug, Clone)]
pub enum ErrorKind {
    UnexpectedToken,
    UnexpectedEof,
    InvalidSyntax,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Span<'a> {
    pub start: usize,
    pub end: usize,
    pub source: &'a str,
}

impl<'a> Span<'a> {
    pub fn new(start: usize, end: usize, source: &'a str) -> Self {
        Span { start, end, source }
    }

    pub fn text(&self) -> &'a str {
        &self.source[self.start..self.end]
    }

    pub fn merge(&self, other: &Span<'a>) -> Span<'a> {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            source: self.source,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expectation<'a> {
    pub expected: Vec<&'static str>,
    pub found: &'a str,
    pub span: Span<'a>,
}

#[derive(Debug, Clone)]
pub struct RichError<'input> {
    pub kind: ErrorKind,
    pub span: Span<'input>,
    pub message: String,
    pub expectation: Option<Expectation<'input>>,
    pub context: Vec<&'input str>,
    pub suggestions: Vec<String>,
}

impl<'input> RichError<'input> {
    pub fn format(&self, context: &ParseContext<'input>) -> String {
        let (line, col) = context.line_col(self.span.start);
        let source_line = context.get_line(self.span.start);

        let mut output = format!(
            "Error at {}:{}: {}\n",
            line,
            col,
            self.message
        );

        // Show source line with highlighting
        output.push_str(&format!("  {}\n", source_line));

        // Underline error span
        let underline_start = col - 1;
        let underline_len = (self.span.end - self.span.start).max(1);
        output.push_str(&format!(
            "  {}{}\n",
            " ".repeat(underline_start),
            "^".repeat(underline_len)
        ));

        // Show expectation
        if let Some(ref exp) = self.expectation {
            output.push_str(&format!(
                "  Expected one of: {}\n",
                exp.expected.join(", ")
            ));
            output.push_str(&format!("  Found: '{}'\n", exp.found));
        }

        // Show suggestions
        if !self.suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  - {}\n", suggestion));
            }
        }

        // Show parsing context
        if !self.context.is_empty() {
            output.push_str("\nContext:\n");
            for ctx in &self.context {
                output.push_str(&format!("  in {}\n", ctx));
            }
        }

        output
    }
}
```

**2. Error recovery parser:**
```rust
pub struct RecoveringParser<'input> {
    input: &'input str,
    position: usize,
    errors: Vec<RichError<'input>>,
    context_stack: Vec<&'static str>,
}

impl<'input> RecoveringParser<'input> {
    pub fn new(input: &'input str) -> Self {
        RecoveringParser {
            input,
            position: 0,
            errors: Vec::new(),
            context_stack: Vec::new(),
        }
    }

    pub fn with_context<F, T>(&mut self, ctx_name: &'static str, f: F) -> Option<T>
    where
        F: FnOnce(&mut Self) -> Option<T>,
    {
        self.context_stack.push(ctx_name);
        let result = f(self);
        self.context_stack.pop();
        result
    }

    pub fn expect(&mut self, expected: &[&'static str]) -> Option<Token<'input>> {
        let checkpoint = self.position;
        let token = self.next_token();

        match token {
            Some(t) if self.token_matches(&t, expected) => Some(t),
            Some(t) => {
                // Wrong token - record error and try to recover
                self.errors.push(RichError {
                    kind: ErrorKind::UnexpectedToken,
                    span: Span::new(t.position, t.position + t.text.len(), self.input),
                    message: format!("Unexpected token '{}'", t.text),
                    expectation: Some(Expectation {
                        expected: expected.to_vec(),
                        found: t.text,
                        span: Span::new(t.position, t.position + t.text.len(), self.input),
                    }),
                    context: self.context_stack.clone(),
                    suggestions: self.generate_suggestions(expected, t.text),
                });

                // Try to recover
                self.recover_to_sync_point();
                None
            }
            None => {
                self.errors.push(RichError {
                    kind: ErrorKind::UnexpectedEof,
                    span: Span::new(checkpoint, checkpoint, self.input),
                    message: "Unexpected end of input".to_string(),
                    expectation: Some(Expectation {
                        expected: expected.to_vec(),
                        found: "<EOF>",
                        span: Span::new(checkpoint, checkpoint, self.input),
                    }),
                    context: self.context_stack.clone(),
                    suggestions: vec![],
                });
                None
            }
        }
    }

    fn token_matches(&self, token: &Token<'input>, expected: &[&'static str]) -> bool {
        expected.iter().any(|exp| token.text == *exp)
    }

    fn generate_suggestions(&self, expected: &[&'static str], found: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Levenshtein distance for typo suggestions
        for exp in expected {
            if self.levenshtein_distance(found, exp) <= 2 {
                suggestions.push(format!("Did you mean '{}'?", exp));
            }
        }

        suggestions
    }

    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,
                        matrix[i + 1][j] + 1,
                    ),
                    matrix[i][j] + cost,
                );
            }
        }

        matrix[len1][len2]
    }

    fn recover_to_sync_point(&mut self) {
        // Skip tokens until we find a synchronization point
        // (e.g., semicolon, closing brace, etc.)
        while let Some(token) = self.next_token() {
            if matches!(token.kind, TokenKind::Punctuation) &&
               (token.text == ";" || token.text == "}" || token.text == "\n") {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'input>> {
        // Token parsing logic
        todo!()
    }

    pub fn finish(self) -> Result<(), Vec<RichError<'input>>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}
```

**3. Error reporting with lifetime-correct formatting:**
```rust
pub struct ErrorReporter<'input> {
    context: &'input ParseContext<'input>,
}

impl<'input> ErrorReporter<'input> {
    pub fn new(context: &'input ParseContext<'input>) -> Self {
        ErrorReporter { context }
    }

    pub fn report_errors(&self, errors: &[RichError<'input>]) {
        eprintln!("Found {} error(s):", errors.len());
        eprintln!();

        for (i, error) in errors.iter().enumerate() {
            eprintln!("Error {}:", i + 1);
            eprintln!("{}", error.format(self.context));
            eprintln!();
        }
    }

    pub fn format_error(&self, error: &RichError<'input>) -> String {
        error.format(self.context)
    }
}
```

**Complete usage example:**
```rust
let source = r#"
    function greet(name) {
        print "Hello, " + nane;  // Typo: nane instead of name
    }
"#;

let context = ParseContext::new(source, Some("example.js"));
let mut parser = RecoveringParser::new(source);

// Parse with error recovery
let result = parser.with_context("function", |p| {
    p.expect(&["function"])?;
    let name = p.expect(&["identifier"])?;
    p.expect(&["("])?;
    // ... parse function body
    Some(())
});

// Report all errors
let reporter = ErrorReporter::new(&context);
match parser.finish() {
    Ok(()) => println!("Parse successful!"),
    Err(errors) => reporter.report_errors(&errors),
}

/* Output:
Error at 3:28: Unexpected token 'nane'
      print "Hello, " + nane;
                           ^^^^
  Expected one of: name, identifier
  Found: 'nane'

Suggestions:
  - Did you mean 'name'?

Context:
  in function
*/
```

**Check/Test:**
- Test error recovery continues parsing after errors
- Test multiple errors reported
- Test suggestion generation for typos
- Test context stack shows nested parse contexts
- Test all error messages borrow from original input
- Verify error spans highlight correct source regions
- Test Levenshtein distance for fuzzy matching

**What this achieves:**
A production-quality parser with:
- **Zero-copy**: No allocations for tokens, all slices
- **Performance**: Memoization prevents exponential blowup
- **Rich Errors**: Detailed error messages with suggestions
- **Error Recovery**: Continues parsing to find multiple errors
- **Lifetime Correctness**: All borrows from original input
- **Ergonomic API**: Iterator support, combinators
- **Flexible**: Multiple lifetime parameters for complex scenarios

**Extensions to explore:**
- Syntax highlighting with borrowed spans
- Auto-formatting preserving original whitespace
- Incremental reparsing (re-parse only changed portions)
- Parallel parsing of independent sections
- AST construction with arena allocation

---

## Project 2: Async Task Scheduler with Pin

### Problem Statement

Build an asynchronous task scheduler that:
- Supports async/await futures with self-referential state
- Uses `Pin` to safely handle futures that reference their own data
- Implements a custom executor for cooperative multitasking
- Handles wake notifications and task scheduling
- Supports futures that hold references across await points
- Implements higher-ranked trait bounds for flexible async closures
- Demonstrates why Pin is necessary for async Rust
- Provides both heap-pinned (Box<Pin>) and stack-pinned futures

The scheduler must safely handle the self-referential nature of async state machines.

### Why It Matters

Understanding Pin is essential for async Rust:
- **Async Runtimes**: Tokio, async-std rely on Pin
- **Self-Referential Futures**: Futures hold pointers to their own stack frames
- **Zero-Copy Async**: Avoid allocations while preserving safety
- **Custom Executors**: Building specialized async runtimes
- **Generator Desugaring**: How async/await translates to state machines

Without Pin:
- Can't safely implement async/await
- Self-referential structs unsound (moving breaks pointers)
- No way to guarantee futures won't move
- Async would require boxing everything (performance cost)

### Use Cases

1. **Custom Async Runtime**: Application-specific task scheduling
2. **Embedded Systems**: Async without heap allocation
3. **Game Loop**: Cooperative multitasking for game logic
4. **State Machines**: Explicit async state management
5. **Protocol Implementations**: Self-referential parser state
6. **Streaming**: Async iterators with borrowed state
7. **Resource Management**: Async RAII with self-references

### Solution Outline

**Core Structure:**
```rust
use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

// Simple future that's self-referential
struct SelfRefFuture {
    data: String,
    // Reference to data (would be invalid if moved)
    data_ptr: *const String,
    _pin: PhantomPinned,
}

impl Future for SelfRefFuture {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safe because we're pinned
        unsafe {
            let data_ref = &*self.data_ptr;
            Poll::Ready(data_ref.as_str())
        }
    }
}

// Executor that schedules tasks
struct Executor {
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push(Box::pin(future));
    }

    fn run(&mut self) {
        // Poll all tasks until complete
    }
}
```

**Key Pin Patterns:**
- **Box::pin()**: Heap-pin for dynamic dispatch
- **Pin::new_unchecked()**: Unsafe pinning for static guarantees
- **Pin projection**: Safely access fields of pinned structs
- **PhantomPinned**: Marker to prevent Unpin auto-impl

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_pinned_future() {
    let future = Box::pin(SelfRefFuture::new("test"));
    // Verify future can't be moved
    // Verify polling works
}

#[test]
fn test_executor_runs_tasks() {
    let mut executor = Executor::new();
    let mut counter = 0;

    executor.spawn(async {
        counter += 1;
    });

    executor.run();
    assert_eq!(counter, 1);
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Synchronous Task Queue

**Goal:** Build a basic task scheduler without async.

**What to implement:**
```rust
type Task = Box<dyn FnOnce()>;

pub struct SyncScheduler {
    tasks: Vec<Task>,
}

impl SyncScheduler {
    pub fn new() -> Self {
        SyncScheduler {
            tasks: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, task: F)
    where
        F: FnOnce() + 'static,
    {
        self.tasks.push(Box::new(task));
    }

    pub fn run(&mut self) {
        while let Some(task) = self.tasks.pop() {
            task();
        }
    }
}
```

**Check/Test:**
- Test spawning and running tasks
- Test tasks execute in order
- Test nested task spawning

**Why this isn't enough:**
No support for waiting/blocking. Tasks run to completion immediately. Can't handle I/O operations efficiently. No cooperative multitasking—once a task starts, it blocks everything else. We need async for non-blocking operations and yielding control.

---

### Step 2: Add Future Trait and Simple Async Execution

**Goal:** Implement basic `Future` trait and polling.

**What to improve:**
```rust
use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
use std::future::Future;
use std::pin::Pin;

// Simple future that completes immediately
struct ReadyFuture<T> {
    value: Option<T>,
}

impl<T> Future for ReadyFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.value.take().unwrap())
    }
}

// Future that yields once before completing
struct YieldOnceFuture<T> {
    value: Option<T>,
    yielded: bool,
}

impl<T> Future for YieldOnceFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.yielded {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(self.value.take().unwrap())
        }
    }
}

// Simple executor
pub struct SimpleExecutor {
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            tasks: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push(Box::pin(future));
    }

    pub fn run(&mut self) {
        let waker = create_noop_waker();
        let mut context = Context::from_waker(&waker);

        while !self.tasks.is_empty() {
            self.tasks.retain_mut(|task| {
                match task.as_mut().poll(&mut context) {
                    Poll::Ready(()) => false,  // Remove completed
                    Poll::Pending => true,      // Keep pending
                }
            });
        }
    }
}

fn create_noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        raw_waker()
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    fn raw_waker() -> RawWaker {
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
        )
    }

    unsafe { Waker::from_raw(raw_waker()) }
}
```

**Check/Test:**
- Test ReadyFuture completes immediately
- Test YieldOnceFuture yields then completes
- Test executor runs all tasks
- Test multiple concurrent tasks

**Why this isn't enough:**
All futures are heap-allocated with `Box::pin()`. No demonstration of *why* Pin is needed—our futures don't actually have self-references yet. The executor is naive (busy-loops on pending tasks). We need to show the actual problem Pin solves: self-referential futures.

---

### Step 3: Demonstrate Self-Referential Problem and Pin Solution

**Goal:** Show why moving self-referential structs is unsound, then solve with Pin.

**What to improve:**

**1. The problem (won't compile):**
```rust
// This CANNOT be implemented safely!
struct SelfReferential {
    data: String,
    reference: &'??? String,  // Can't reference self.data
}

// Even this doesn't work:
struct Attempted<'a> {
    data: String,
    reference: &'a String,
}

impl<'a> Attempted<'a> {
    fn new(s: String) -> Self {
        Attempted {
            data: s,
            reference: &s,  // ERROR: can't borrow s after moving
        }
    }
}
```

**2. Raw pointer approach (unsafe but shows the issue):**
```rust
struct UnsafeSelfRef {
    data: String,
    data_ptr: *const String,  // Raw pointer to data
}

impl UnsafeSelfRef {
    fn new(s: String) -> Self {
        let mut this = UnsafeSelfRef {
            data: s,
            data_ptr: std::ptr::null(),
        };

        // Set pointer to our own data
        this.data_ptr = &this.data as *const String;
        this
    }

    fn get_ref(&self) -> &str {
        unsafe { &*self.data_ptr }
    }
}

fn demonstrate_problem() {
    let s = UnsafeSelfRef::new(String::from("hello"));
    println!("{}", s.get_ref());  // OK

    // Move the struct!
    let s2 = s;

    // BUG: s2.data_ptr still points to OLD location
    // println!("{}", s2.get_ref());  // Use-after-move!
}
```

**3. Pin solution:**
```rust
use std::marker::PhantomPinned;
use std::pin::Pin;

struct PinnedSelfRef {
    data: String,
    data_ptr: *const String,
    _pin: PhantomPinned,  // Opts out of Unpin
}

impl PinnedSelfRef {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(PinnedSelfRef {
            data: s,
            data_ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // Safe: boxed is pinned, won't move
        let data_ptr: *const String = &boxed.data;

        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).data_ptr = data_ptr;
        }

        boxed
    }

    fn get_ref(self: Pin<&Self>) -> &str {
        // Safe: we're pinned, pointer is valid
        unsafe { &*self.data_ptr }
    }
}

fn demonstrate_solution() {
    let pinned = PinnedSelfRef::new(String::from("hello"));
    println!("{}", pinned.as_ref().get_ref());

    // Cannot move out of Pin!
    // let moved = *pinned;  // ERROR: cannot move out of Pin
}
```

**4. Future that's actually self-referential:**
```rust
struct JoinFuture<F1, F2>
where
    F1: Future,
    F2: Future,
{
    future1: F1,
    future2: F2,
    // Store reference to future1's output across polls
    future1_output: Option<F1::Output>,
    _pin: PhantomPinned,
}

impl<F1, F2> Future for JoinFuture<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is simplified - real version needs unsafe projection
        todo!("Requires pin projection")
    }
}
```

**Check/Test:**
- Test UnsafeSelfRef shows the bug
- Test PinnedSelfRef prevents moving
- Verify PhantomPinned makes type !Unpin
- Test cannot extract value from Pin

**Why this isn't enough:**
We've shown Pin solves self-references, but accessing fields of pinned structs is cumbersome. We used unsafe everywhere. Real futures need "pin projection"—safely accessing fields while maintaining pin guarantees. Also, no actual useful executor yet—just demonstrations.

---

### Step 4: Implement Pin Projection and Useful Futures

**Goal:** Use pin-project for safe field access and build useful combinators.

**What to improve:**

**1. Pin projection (using pin-project crate):**
```rust
use pin_project::pin_project;

#[pin_project]
struct Join<F1, F2> {
    #[pin]
    future1: F1,
    #[pin]
    future2: F2,
    state: JoinState,
}

enum JoinState {
    BothPending,
    FirstComplete,
    SecondComplete,
}

impl<F1, F2> Future for Join<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();  // Pin projection magic

        match this.state {
            JoinState::BothPending => {
                match this.future1.poll(cx) {
                    Poll::Ready(output1) => {
                        *this.state = JoinState::FirstComplete;
                        // Store output1 somehow...
                    }
                    Poll::Pending => {}
                }

                match this.future2.poll(cx) {
                    Poll::Ready(output2) => {
                        *this.state = JoinState::SecondComplete;
                        // Store output2 somehow...
                    }
                    Poll::Pending => {}
                }

                Poll::Pending
            }
            _ => todo!(),
        }
    }
}
```

**2. Useful future combinators:**
```rust
// Map combinator
#[pin_project]
struct Map<F, G> {
    #[pin]
    future: F,
    mapper: Option<G>,
}

impl<F, G, T> Future for Map<F, G>
where
    F: Future,
    G: FnOnce(F::Output) -> T,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.future.poll(cx) {
            Poll::Ready(output) => {
                let mapper = this.mapper.take().unwrap();
                Poll::Ready(mapper(output))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Then combinator (sequential composition)
#[pin_project(project = ThenProj)]
enum Then<F1, F2> {
    First {
        #[pin]
        future1: F1,
        future2_factory: Option<F2Factory>,
    },
    Second {
        #[pin]
        future2: F2,
    },
}

// AndThen combinator
// Select combinator (first to complete)
// Timeout combinator
```

**3. Stack pinning (no allocation):**
```rust
use std::pin::pin;

fn stack_pin_example() {
    let future = async {
        println!("Hello from future!");
    };

    // Pin on stack (Rust 1.68+)
    let mut pinned = pin!(future);

    let waker = create_noop_waker();
    let mut context = Context::from_waker(&waker);

    match pinned.as_mut().poll(&mut context) {
        Poll::Ready(()) => println!("Done!"),
        Poll::Pending => println!("Not yet"),
    }
}
```

**Check/Test:**
- Test pin projection allows safe field access
- Test map combinator transforms outputs
- Test then combinator sequences futures
- Test stack pinning works without allocation
- Benchmark stack-pinned vs heap-pinned

**Why this isn't enough:**
Combinators are useful but our executor is still naive. A real executor needs:
- Wake mechanism (don't poll unless notified)
- Task queue (fair scheduling)
- Thread pool (parallel execution)
- Timers and I/O readiness

Let's build a proper executor.

---

### Step 5: Build Executor with Wake Support and Task Queue

**Goal:** Implement a real executor with proper wake notifications.

**What to improve:**

**1. Task structure with waker:**
```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::task::{Context, Poll, Waker};

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: Arc<Executor>,
}

impl Task {
    fn poll(self: &Arc<Self>) {
        let waker = self.create_waker();
        let mut context = Context::from_waker(&waker);

        let mut future = self.future.lock().unwrap();

        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {
                // Task complete
            }
            Poll::Pending => {
                // Task yielded, will be woken later
            }
        }
    }

    fn create_waker(self: &Arc<Self>) -> Waker {
        // Create waker that reschedules this task
        Arc::clone(self).into_waker()
    }
}

// Implement Waker interface
fn task_into_waker(task: Arc<Task>) -> Waker {
    unsafe fn clone_raw(ptr: *const ()) -> RawWaker {
        let task = Arc::from_raw(ptr as *const Task);
        let cloned = Arc::clone(&task);
        std::mem::forget(task);
        RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
    }

    unsafe fn wake_raw(ptr: *const ()) {
        let task = Arc::from_raw(ptr as *const Task);
        task.executor.schedule(Arc::clone(&task));
    }

    unsafe fn wake_by_ref_raw(ptr: *const ()) {
        let task = Arc::from_raw(ptr as *const Task);
        task.executor.schedule(Arc::clone(&task));
        std::mem::forget(task);
    }

    unsafe fn drop_raw(ptr: *const ()) {
        drop(Arc::from_raw(ptr as *const Task));
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(
        clone_raw,
        wake_raw,
        wake_by_ref_raw,
        drop_raw,
    );

    unsafe {
        Waker::from_raw(RawWaker::new(
            Arc::into_raw(task) as *const (),
            &VTABLE,
        ))
    }
}
```

**2. Executor with task queue:**
```rust
pub struct Executor {
    queue: Mutex<VecDeque<Arc<Task>>>,
}

impl Executor {
    pub fn new() -> Arc<Self> {
        Arc::new(Executor {
            queue: Mutex::new(VecDeque::new()),
        })
    }

    pub fn spawn<F>(self: &Arc<Self>, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: Arc::clone(self),
        });

        self.schedule(task);
    }

    fn schedule(&self, task: Arc<Task>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(task);
    }

    pub fn run(&self) {
        loop {
            let task = {
                let mut queue = self.queue.lock().unwrap();
                queue.pop_front()
            };

            match task {
                Some(task) => task.poll(),
                None => break,  // No more tasks
            }
        }
    }
}
```

**3. Timer future (demonstrates wake mechanism):**
```rust
use std::time::{Duration, Instant};
use std::thread;

struct TimerFuture {
    deadline: Instant,
    waker_sent: bool,
}

impl TimerFuture {
    fn new(duration: Duration) -> Self {
        TimerFuture {
            deadline: Instant::now() + duration,
            waker_sent: false,
        }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }

        if !self.waker_sent {
            let waker = cx.waker().clone();
            let deadline = self.deadline;

            // Spawn thread to wake us
            thread::spawn(move || {
                let now = Instant::now();
                if now < deadline {
                    thread::sleep(deadline - now);
                }
                waker.wake();
            });

            self.waker_sent = true;
        }

        Poll::Pending
    }
}
```

**Usage:**
```rust
let executor = Executor::new();

executor.spawn(async {
    println!("Starting task 1");
    TimerFuture::new(Duration::from_secs(1)).await;
    println!("Task 1 after 1 second");
});

executor.spawn(async {
    println!("Starting task 2");
    TimerFuture::new(Duration::from_millis(500)).await;
    println!("Task 2 after 500ms");
});

executor.run();
```

**Check/Test:**
- Test tasks wake correctly after timer
- Test multiple concurrent tasks
- Test wake from different threads
- Verify fair scheduling (all tasks make progress)
- Test executor stops when no tasks remain

**Why this isn't enough:**
Single-threaded executor is a bottleneck. Need work-stealing thread pool for parallelism. Also no I/O support (files, network). Real executors integrate with OS event loops (epoll, kqueue, IOCP). Let's add multi-threading and I/O.

---

### Step 6: Add Multi-Threading and Higher-Ranked Trait Bounds

**Goal:** Implement work-stealing thread pool and demonstrate HRTB with async closures.

**What to improve:**

**1. Work-stealing executor:**
```rust
use crossbeam::deque::{Injector, Stealer, Worker};
use std::thread;

pub struct WorkStealingExecutor {
    global_queue: Arc<Injector<Arc<Task>>>,
    stealers: Vec<Stealer<Arc<Task>>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl WorkStealingExecutor {
    pub fn new(num_threads: usize) -> Arc<Self> {
        let global_queue = Arc::new(Injector::new());
        let mut local_queues = Vec::new();
        let mut stealers = Vec::new();

        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            local_queues.push(worker);
        }

        let executor = Arc::new(WorkStealingExecutor {
            global_queue,
            stealers,
            workers: Vec::new(),
        });

        // Start worker threads
        // ...

        executor
    }

    fn worker_loop(
        local: Worker<Arc<Task>>,
        global: Arc<Injector<Arc<Task>>>,
        stealers: Vec<Stealer<Arc<Task>>>,
    ) {
        loop {
            // Try local queue first
            let task = local.pop()
                .or_else(|| {
                    // Try stealing from global
                    global.steal_batch_and_pop(&local).success()
                })
                .or_else(|| {
                    // Try stealing from other workers
                    stealers.iter()
                        .map(|s| s.steal())
                        .find_map(|s| s.success())
                });

            match task {
                Some(task) => task.poll(),
                None => {
                    thread::yield_now();
                }
            }
        }
    }
}
```

**2. Higher-ranked trait bounds with async:**
```rust
// HRTB: closure that works with any lifetime
pub fn with_async_context<F, Fut>(f: F)
where
    F: for<'a> FnOnce(&'a Context) -> Fut,
    Fut: Future<Output = ()>,
{
    // f can be called with Context of any lifetime
}

// Async function that accepts HRTB closure
pub async fn process_items<F, Fut>(items: Vec<String>, processor: F)
where
    F: for<'a> Fn(&'a str) -> Fut,
    Fut: Future<Output = ()>,
{
    for item in &items {
        processor(item).await;
    }
}

// Usage
async {
    let items = vec!["a".to_string(), "b".to_string()];

    process_items(items, |s| async move {
        println!("Processing: {}", s);
        TimerFuture::new(Duration::from_millis(100)).await;
    }).await;
}
```

**3. Async channel for communication:**
```rust
use std::sync::mpsc::{channel, Sender, Receiver};

struct AsyncChannel<T> {
    sender: Sender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> AsyncChannel<T> {
    fn new() -> Self {
        let (sender, receiver) = channel();
        AsyncChannel {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    fn send(&self, value: T) -> Result<(), T> {
        self.sender.send(value).map_err(|e| e.0)
    }

    async fn recv(&self) -> Option<T> {
        // Poll receiver, park if empty, wake when data arrives
        todo!()
    }
}
```

**4. Complete example:**
```rust
#[tokio::main]  // Or our custom executor
async fn main() {
    let executor = WorkStealingExecutor::new(4);

    executor.spawn(async {
        println!("Parallel task 1");
        TimerFuture::new(Duration::from_secs(1)).await;
        println!("Task 1 done");
    });

    executor.spawn(async {
        println!("Parallel task 2");
        TimerFuture::new(Duration::from_secs(1)).await;
        println!("Task 2 done");
    });

    // With HRTB
    let items = vec!["item1".to_string(), "item2".to_string()];
    executor.spawn(async move {
        process_items(items, |item| async move {
            println!("Processing: {}", item);
        }).await;
    });

    executor.run();
}
```

**Check/Test:**
- Test work-stealing load balances across threads
- Test HRTB closures with various lifetimes
- Test concurrent task execution
- Benchmark: multi-threaded vs single-threaded
- Test channel communication between tasks
- Verify Pin safety maintained across threads

**What this achieves:**
A production-ready async executor:
- **Pin-Safe**: Properly handles self-referential futures
- **Multi-Threaded**: Work-stealing for parallelism
- **Wake Mechanism**: Efficient task scheduling
- **HRTB Support**: Flexible async closures
- **Zero-Cost**: Pin is compile-time only
- **Practical**: Can run real async workloads

**Extensions to explore:**
- I/O integration (async file/network)
- Task priorities and deadlines
- Async cancellation/timeout
- Async streams (async iterators)
- Integration with existing runtimes (Tokio, async-std)

---

## Project 3: Arena Allocator with Lifetime Variance

### Problem Statement

Build a type-safe arena allocator that:
- Allocates objects with tied lifetimes (all objects live as long as arena)
- Supports efficient bulk allocation and deallocation
- Demonstrates lifetime variance (covariance, invariance)
- Provides typed and untyped arena variants
- Implements safe iteration over allocated objects
- Supports interior mutability with lifetime safety
- Uses phantom data to control variance
- Enables self-referential object graphs within arena

The allocator must leverage Rust's variance rules for maximum flexibility while maintaining safety.

### Why It Matters

Arena allocators are critical for performance:
- **Compilers**: AST nodes allocated in arena
- **Game Engines**: Frame-by-frame entity allocation
- **Parsers**: Parse tree nodes in arena
- **Database Systems**: Query plan nodes
- **Graphics**: Scene graph allocation

Understanding variance is essential:
- **Lifetime Flexibility**: Longer lifetimes usable where shorter expected
- **Soundness**: Invariance prevents lifetime bugs with mutation
- **API Design**: Choose correct variance for custom pointer types
- **Generic Collections**: Understand why `Vec<T>` covariant, `Cell<T>` invariant

### Use Cases

1. **AST Construction**: Parse tree with arena-allocated nodes
2. **Graph Algorithms**: Nodes/edges in arena
3. **Game Objects**: Entities, components in frame arena
4. **String Interning**: Deduplicated strings with arena lifetime
5. **Temporary Allocations**: Batch allocations for request handling
6. **Bump Allocator**: Fast linear allocation for short-lived objects
7. **Object Pools**: Reusable typed object allocation

### Solution Outline

**Core Structure:**
```rust
use std::cell::UnsafeCell;
use std::marker::PhantomData;

pub struct Arena<'a> {
    chunks: Vec<Chunk>,
    _marker: PhantomData<&'a ()>,  // Covariant over 'a
}

struct Chunk {
    data: Vec<u8>,
    offset: usize,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self { /* ... */ }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T { /* ... */ }

    pub fn alloc_slice<T>(&'a self, len: usize) -> &'a mut [T] { /* ... */ }
}

// Typed arena (invariant for safety)
pub struct TypedArena<T> {
    chunks: Vec<Vec<T>>,
    current: UnsafeCell<Vec<T>>,
}

impl<T> TypedArena<T> {
    pub fn alloc(&self, value: T) -> &mut T { /* ... */ }

    pub fn alloc_iter<I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T> { /* ... */ }
}
```

**Variance Patterns:**
- **Covariant Arena**: `PhantomData<&'a ()>` makes `Arena<'static>` usable as `Arena<'short>`
- **Invariant TypedArena**: Interior mutability requires invariance
- **Lifetime Bounds**: Objects in arena can reference each other

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_arena_allocation() {
    let arena = Arena::new();
    let x = arena.alloc(42);
    let y = arena.alloc(100);

    assert_eq!(*x, 42);
    assert_eq!(*y, 100);
}

#[test]
fn test_variance() {
    fn takes_short_arena(arena: &Arena<'_>) {
        let _ = arena.alloc(42);
    }

    let long_arena: Arena<'static> = Arena::new();
    takes_short_arena(&long_arena);  // OK: covariant
}

#[test]
fn test_self_referential_graph() {
    struct Node<'a> {
        value: i32,
        next: Option<&'a Node<'a>>,
    }

    let arena = Arena::new();
    let node1 = arena.alloc(Node { value: 1, next: None });
    let node2 = arena.alloc(Node { value: 2, next: Some(node1) });

    assert_eq!(node2.next.unwrap().value, 1);
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Bump Allocator with Unsafe

**Goal:** Implement simple arena using unsafe pointer arithmetic.

**What to implement:**
```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

pub struct BumpAllocator {
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).unwrap();
        let buffer = unsafe { alloc(layout) };

        BumpAllocator {
            buffer,
            capacity,
            offset: 0,
        }
    }

    pub fn alloc<T>(&mut self, value: T) -> *mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Align offset
        let padding = (align - (self.offset % align)) % align;
        self.offset += padding;

        if self.offset + size > self.capacity {
            panic!("Arena out of memory");
        }

        let ptr = unsafe { self.buffer.add(self.offset) as *mut T };
        self.offset += size;

        unsafe {
            ptr.write(value);
        }

        ptr
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        // Note: doesn't call destructors!
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 8).unwrap();
        unsafe {
            dealloc(self.buffer, layout);
        }
    }
}
```

**Check/Test:**
- Test allocation of various types
- Test alignment is correct
- Test out-of-memory panic
- Test reset reuses buffer
- Memory leak test (valgrind/miri)

**Why this isn't enough:**
Returns raw pointers—no lifetime safety! Can use-after-free if arena dropped while pointers exist. No Drop called for allocated objects—leaks resources. Type-unsafe (just `*mut T`). No bulk operations. We need lifetimes to tie allocations to arena lifetime.

---

### Step 2: Add Lifetimes and Safe References

**Goal:** Use lifetimes to prevent use-after-free.

**What to improve:**
```rust
use std::cell::Cell;

pub struct Arena<'a> {
    buffer: Vec<u8>,
    offset: Cell<usize>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self {
        Arena::with_capacity(4096)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Arena {
            buffer: Vec::with_capacity(capacity),
            offset: Cell::new(0),
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let mut offset = self.offset.get();

        // Align
        let padding = (align - (offset % align)) % align;
        offset += padding;

        let new_offset = offset + size;

        // Grow buffer if needed
        if new_offset > self.buffer.capacity() {
            panic!("Arena out of memory");
        }

        unsafe {
            // Write value at aligned offset
            let ptr = self.buffer.as_ptr().add(offset) as *mut T;
            ptr.write(value);

            self.offset.set(new_offset);

            &mut *ptr
        }
    }

    pub fn alloc_slice<T: Copy>(&'a self, slice: &[T]) -> &'a mut [T] {
        let len = slice.len();
        let size = std::mem::size_of::<T>() * len;
        let align = std::mem::align_of::<T>();

        let mut offset = self.offset.get();
        let padding = (align - (offset % align)) % align;
        offset += padding;

        let new_offset = offset + size;

        if new_offset > self.buffer.capacity() {
            panic!("Arena out of memory");
        }

        unsafe {
            let ptr = self.buffer.as_ptr().add(offset) as *mut T;

            // Copy slice data
            ptr.copy_from_nonoverlapping(slice.as_ptr(), len);

            self.offset.set(new_offset);

            std::slice::from_raw_parts_mut(ptr, len)
        }
    }
}
```

**Check/Test:**
- Test references tied to arena lifetime
- Test cannot use reference after arena dropped (compile error)
- Test multiple allocations share arena lifetime
- Test slice allocation

**Why this isn't enough:**
Growing buffer invalidates all previous pointers! When we grow `Vec`, it reallocates, moving data to a new address. All existing `&'a mut T` now point to freed memory. We need a chunked approach where each chunk has a stable address. Also, no Drop support—allocated objects don't destruct.

---

### Step 3: Implement Chunked Arena and Drop Support

**Goal:** Fix pointer invalidation and support destructors.

**What to improve:**
```rust
use std::cell::{Cell, RefCell};
use std::ptr::NonNull;

struct Chunk {
    data: Vec<u8>,
    offset: usize,
}

impl Chunk {
    fn new(capacity: usize) -> Self {
        Chunk {
            data: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.data.capacity() - self.offset
    }

    fn alloc<T>(&mut self, value: T) -> NonNull<T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let padding = (align - (self.offset % align)) % align;
        self.offset += padding;

        assert!(self.offset + size <= self.data.capacity());

        unsafe {
            let ptr = self.data.as_mut_ptr().add(self.offset) as *mut T;
            ptr.write(value);
            self.offset += size;

            NonNull::new_unchecked(ptr)
        }
    }
}

pub struct Arena<'a> {
    chunks: RefCell<Vec<Chunk>>,
    chunk_size: usize,
    // Track allocated objects for Drop
    destructors: RefCell<Vec<Box<dyn FnOnce()>>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self {
        Arena::with_chunk_size(4096)
    }

    pub fn with_chunk_size(chunk_size: usize) -> Self {
        let mut chunks = Vec::new();
        chunks.push(Chunk::new(chunk_size));

        Arena {
            chunks: RefCell::new(chunks),
            chunk_size,
            destructors: RefCell::new(Vec::new()),
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        let size = std::mem::size_of::<T>();

        // Find or create chunk with enough space
        let mut chunks = self.chunks.borrow_mut();

        let current_chunk = chunks.last_mut().unwrap();
        if current_chunk.remaining() < size + std::mem::align_of::<T>() {
            // Need new chunk
            let new_size = self.chunk_size.max(size * 2);
            chunks.push(Chunk::new(new_size));
        }

        let chunk = chunks.last_mut().unwrap();
        let ptr = chunk.alloc(value);

        // Register destructor if T needs drop
        if std::mem::needs_drop::<T>() {
            let destructor = Box::new(move || unsafe {
                ptr::drop_in_place(ptr.as_ptr());
            });
            self.destructors.borrow_mut().push(destructor);
        }

        unsafe { &mut *ptr.as_ptr() }
    }

    pub fn alloc_many<T, I>(&'a self, iter: I) -> &'a mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let len = iter.len();

        if len == 0 {
            return &mut [];
        }

        let size = std::mem::size_of::<T>() * len;
        let align = std::mem::align_of::<T>();

        let mut chunks = self.chunks.borrow_mut();

        // Ensure current chunk has space
        if chunks.last().unwrap().remaining() < size + align {
            chunks.push(Chunk::new(self.chunk_size.max(size * 2)));
        }

        let chunk = chunks.last_mut().unwrap();

        // Allocate space
        let padding = (align - (chunk.offset % align)) % align;
        chunk.offset += padding;

        let ptr = unsafe {
            chunk.data.as_mut_ptr().add(chunk.offset) as *mut T
        };

        chunk.offset += size;

        // Write items
        for (i, item) in iter.enumerate() {
            unsafe {
                ptr.add(i).write(item);
            }

            if std::mem::needs_drop::<T>() {
                let item_ptr = unsafe { ptr.add(i) };
                let destructor = Box::new(move || unsafe {
                    ptr::drop_in_place(item_ptr);
                });
                self.destructors.borrow_mut().push(destructor);
            }
        }

        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<'a> Drop for Arena<'a> {
    fn drop(&mut self) {
        // Run all destructors
        let destructors = std::mem::take(&mut *self.destructors.borrow_mut());
        for destructor in destructors {
            destructor();
        }
    }
}
```

**Check/Test:**
- Test chunked allocation doesn't invalidate pointers
- Test Drop is called for allocated objects
- Test allocating many items at once
- Test large allocations get dedicated chunks
- Memory safety with Miri

**Why this isn't enough:**
Interior mutability (`RefCell`) makes the arena invariant over its lifetime `'a`. This prevents useful variance patterns. Also, the Drop tracking is inefficient—storing a closure per object is heavyweight. We should use a typed arena for better performance when all objects are the same type. Let's add variance control.

---

### Step 4: Add Variance Control and Typed Arena

**Goal:** Control variance with PhantomData and create typed arena variant.

**What to improve:**

**1. Covariant arena (immutable allocations):**
```rust
// Covariant over 'a - can use Arena<'long> where Arena<'short> expected
pub struct CovariantArena<'a> {
    chunks: UnsafeCell<Vec<Chunk>>,
    chunk_size: usize,
    _marker: PhantomData<&'a ()>,  // Covariant!
}

impl<'a> CovariantArena<'a> {
    pub fn alloc<T>(&'a self, value: T) -> &'a T {
        // Returns immutable reference for covariance
        unsafe {
            let chunks = &mut *self.chunks.get();
            // ... allocation logic ...
            &*ptr
        }
    }
}

// Can use with variance:
fn use_arena(arena: &CovariantArena<'_>) {
    let _ = arena.alloc(42);
}

let static_arena: CovariantArena<'static> = CovariantArena::new();
use_arena(&static_arena);  // OK: 'static coerces to shorter lifetime
```

**2. Typed arena (no Drop tracking overhead):**
```rust
pub struct TypedArena<T> {
    chunks: RefCell<Vec<Vec<T>>>,
    current: RefCell<Vec<T>>,
}

impl<T> TypedArena<T> {
    pub fn new() -> Self {
        TypedArena {
            chunks: RefCell::new(Vec::new()),
            current: RefCell::new(Vec::with_capacity(64)),
        }
    }

    pub fn alloc(&self, value: T) -> &mut T {
        let mut current = self.current.borrow_mut();

        // Move full chunk to chunks list
        if current.len() == current.capacity() {
            let new_current = Vec::with_capacity(current.capacity() * 2);
            let old_current = std::mem::replace(&mut *current, new_current);
            self.chunks.borrow_mut().push(old_current);
        }

        current.push(value);

        // Safe: reference is valid as long as TypedArena exists
        unsafe {
            let ptr = current.last_mut().unwrap() as *mut T;
            &mut *ptr
        }
    }

    pub fn alloc_many<I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
    {
        let mut current = self.current.borrow_mut();

        let start = current.len();
        current.extend(iter);
        let end = current.len();

        unsafe {
            let ptr = current.as_mut_ptr().add(start);
            std::slice::from_raw_parts_mut(ptr, end - start)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        // Iterate over all allocated objects
        self.chunks.borrow().iter()
            .flat_map(|chunk| chunk.iter())
            .chain(self.current.borrow().iter())
    }
}

// Drop automatically called for all T when TypedArena drops
```

**3. Demonstrate variance:**
```rust
#[test]
fn test_covariance() {
    fn accepts_short<'a>(arena: &CovariantArena<'a>, data: &'a str) {
        let _ = arena.alloc(data);
    }

    let long_arena: CovariantArena<'static> = CovariantArena::new();
    let static_str: &'static str = "hello";

    // 'static coerces to shorter lifetime
    accepts_short(&long_arena, static_str);  // OK!
}

#[test]
fn test_typed_arena_variance() {
    // TypedArena<&'a str> is covariant over 'a
    fn process_strings<'a>(arena: &TypedArena<&'a str>) {
        let _ = arena.alloc("short-lived");
    }

    let arena: TypedArena<&'static str> = TypedArena::new();
    process_strings(&arena);  // OK: covariant
}
```

**Check/Test:**
- Test variance allows lifetime coercion
- Test typed arena performance vs untyped
- Test iterator over allocated objects
- Test Drop called for all objects in TypedArena
- Benchmark: typed vs untyped arena

**Why this isn't enough:**
Can't have self-referential objects yet. What if we want graph nodes in the arena that reference each other? Current API doesn't support two-phase initialization. Also no string interning (deduplicate identical strings). Let's add those features.

---

### Step 5: Add Self-Referential Support and String Interning

**Goal:** Support object graphs with interior references.

**What to improve:**

**1. Two-phase initialization for self-references:**
```rust
impl<'a> Arena<'a> {
    pub fn alloc_uninit<T>(&'a self) -> &'a mut std::mem::MaybeUninit<T> {
        // Allocate uninitialized memory
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        unsafe {
            let chunks = &mut *self.chunks.get();
            // ... allocate space ...
            &mut *(ptr as *mut std::mem::MaybeUninit<T>)
        }
    }

    pub fn alloc_with<T, F>(&'a self, f: F) -> &'a mut T
    where
        F: FnOnce(&'a Arena<'a>) -> T,
    {
        // Call f with arena reference, allows creating self-refs
        let value = f(self);
        self.alloc(value)
    }
}

// Usage: Self-referential graph
#[test]
fn test_self_referential_graph() {
    struct Node<'a> {
        value: i32,
        neighbors: Vec<&'a Node<'a>>,
    }

    let arena = Arena::new();

    let node1 = arena.alloc(Node {
        value: 1,
        neighbors: Vec::new(),
    });

    let node2 = arena.alloc_with(|arena| {
        Node {
            value: 2,
            neighbors: vec![node1],  // Reference to node1!
        }
    });

    let node3 = arena.alloc_with(|arena| {
        Node {
            value: 3,
            neighbors: vec![node1, node2],
        }
    });

    assert_eq!(node3.neighbors.len(), 2);
    assert_eq!(node3.neighbors[0].value, 1);
    assert_eq!(node3.neighbors[1].value, 2);
}
```

**2. String interning:**
```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Wrapper that hashes by content
struct InternedStr<'a>(&'a str);

impl<'a> Hash for InternedStr<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> PartialEq for InternedStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> Eq for InternedStr<'a> {}

pub struct StringInterner<'a> {
    arena: Arena<'a>,
    map: RefCell<HashMap<InternedStr<'a>, &'a str>>,
}

impl<'a> StringInterner<'a> {
    pub fn new() -> Self {
        StringInterner {
            arena: Arena::new(),
            map: RefCell::new(HashMap::new()),
        }
    }

    pub fn intern(&'a self, s: &str) -> &'a str {
        let mut map = self.map.borrow_mut();

        // Check if already interned
        if let Some(&interned) = map.get(&InternedStr(s)) {
            return interned;
        }

        // Allocate new string in arena
        let bytes = self.arena.alloc_slice(s.as_bytes());
        let interned = unsafe {
            std::str::from_utf8_unchecked(bytes)
        };

        map.insert(InternedStr(interned), interned);
        interned
    }

    pub fn get(&self, s: &str) -> Option<&'a str> {
        self.map.borrow().get(&InternedStr(s)).copied()
    }
}

#[test]
fn test_string_interning() {
    let interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");
    let s3 = interner.intern("world");

    // Same string, same pointer
    assert!(std::ptr::eq(s1, s2));
    assert!(!std::ptr::eq(s1, s3));
}
```

**3. Iteration and collection:**
```rust
impl<T> TypedArena<T> {
    pub fn into_vec(self) -> Vec<T> {
        let mut result = Vec::new();

        let chunks = self.chunks.into_inner();
        for chunk in chunks {
            result.extend(chunk);
        }

        result.extend(self.current.into_inner());
        result
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.chunks.get_mut().iter_mut()
            .flat_map(|chunk| chunk.iter_mut())
            .chain(self.current.get_mut().iter_mut())
    }
}
```

**Check/Test:**
- Test self-referential graph allocation
- Test string interning deduplicates strings
- Test iteration over arena objects
- Test complex graph structures (trees, DAGs)
- Verify pointer equality for interned strings

**Why this isn't enough:**
No support for parallel allocation (thread-safe arena). Also, the iteration borrows the entire arena mutably—can't allocate while iterating. Real-world use cases like parallel parsing need thread-local arenas. Let's add thread safety.

---

### Step 6: Add Thread-Safe Scoped Arena and Parallel Allocation

**Goal:** Support concurrent allocation with scoped lifetimes.

**What to improve:**

**1. Thread-safe arena:**
```rust
use std::sync::{Arc, Mutex};

pub struct ThreadSafeArena<'a> {
    chunks: Arc<Mutex<Vec<Chunk>>>,
    chunk_size: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ThreadSafeArena<'a> {
    pub fn new() -> Self {
        let chunks = vec![Chunk::new(4096)];
        ThreadSafeArena {
            chunks: Arc::new(Mutex::new(chunks)),
            chunk_size: 4096,
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        let mut chunks = self.chunks.lock().unwrap();

        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Find/create chunk with space
        if chunks.last().unwrap().remaining() < size + align {
            chunks.push(Chunk::new(self.chunk_size));
        }

        let chunk = chunks.last_mut().unwrap();
        let ptr = chunk.alloc(value);

        unsafe { &mut *ptr.as_ptr() }
    }
}

unsafe impl<'a> Send for ThreadSafeArena<'a> {}
unsafe impl<'a> Sync for ThreadSafeArena<'a> {}
```

**2. Scoped arena with crossbeam:**
```rust
use crossbeam::thread;

pub fn scoped_arena<F, R>(f: F) -> R
where
    F: for<'scope> FnOnce(&'scope Arena<'scope>) -> R,
{
    let arena = Arena::new();
    f(&arena)
    // arena dropped here, all references invalid
}

#[test]
fn test_scoped_arena() {
    let result = scoped_arena(|arena| {
        let x = arena.alloc(42);
        let y = arena.alloc(100);
        *x + *y
    });

    assert_eq!(result, 142);
    // Cannot use x or y here - lifetime ended
}
```

**3. Thread-local arena pool:**
```rust
use std::cell::RefCell;

thread_local! {
    static ARENA: RefCell<Option<TypedArena<u8>>> = RefCell::new(None);
}

pub fn with_thread_arena<F, R>(f: F) -> R
where
    F: FnOnce(&TypedArena<u8>) -> R,
{
    ARENA.with(|cell| {
        let mut arena_opt = cell.borrow_mut();

        if arena_opt.is_none() {
            *arena_opt = Some(TypedArena::new());
        }

        let arena = arena_opt.as_ref().unwrap();
        f(arena)
    })
}
```

**4. Parallel parsing example:**
```rust
#[test]
fn test_parallel_parsing() {
    use rayon::prelude::*;

    let inputs = vec![
        "line 1 data",
        "line 2 data",
        "line 3 data",
    ];

    let results: Vec<_> = inputs.par_iter()
        .map(|input| {
            // Each thread gets its own arena
            let arena = Arena::new();
            parse_line(&arena, input)
        })
        .collect();

    assert_eq!(results.len(), 3);
}

fn parse_line<'a>(arena: &'a Arena<'a>, input: &str) -> Vec<&'a str> {
    input.split_whitespace()
        .map(|word| {
            let bytes = arena.alloc_slice(word.as_bytes());
            unsafe { std::str::from_utf8_unchecked(bytes) }
        })
        .collect()
}
```

**5. Complete variance demonstration:**
```rust
// Demonstrate all variance types
use std::cell::Cell;

// Covariant: &'a T
fn covariant_example() {
    let long: &'static str = "long";
    let short: &str = long;  // OK
}

// Invariant: Cell<&'a T>
fn invariant_example() {
    let long: Cell<&'static str> = Cell::new("long");
    // let short: Cell<&str> = long;  // ERROR: invariant
}

// Arena variance
fn arena_variance() {
    fn use_arena<'a>(arena: &Arena<'a>) {
        let _ = arena.alloc(42);
    }

    let static_arena: Arena<'static> = Arena::new();
    use_arena(&static_arena);  // OK: covariant over 'a
}

// PhantomData variance control
struct CovariantWrapper<'a, T> {
    _marker: PhantomData<&'a T>,
}

struct InvariantWrapper<'a, T> {
    _marker: PhantomData<Cell<&'a T>>,
}
```

**Check/Test:**
- Test thread-safe arena from multiple threads
- Test scoped arena lifetime enforcement
- Test parallel parsing with thread-local arenas
- Verify variance rules with compile tests
- Benchmark: parallel vs sequential allocation
- Test cannot leak references outside scoped lifetime

**What this achieves:**
A production-ready arena allocator:
- **Lifetime Safety**: References tied to arena lifetime
- **Performance**: O(1) allocation, bulk deallocation
- **Variance**: Covariant for flexibility
- **Thread Safety**: Concurrent allocation support
- **Self-References**: Object graphs within arena
- **String Interning**: Deduplication for strings
- **Scoped Lifetimes**: Safe temporary allocation

**Extensions to explore:**
- Reset/clear arena for reuse
- Statistics (bytes allocated, objects count)
- Alignment guarantees for SIMD types
- Integration with allocator API
- Stack-like pop/restore checkpoints

---

## Summary

These three projects teach essential lifetime patterns in Rust:

1. **Zero-Copy Parser**: Lifetime elision, explicit lifetimes, multiple lifetime parameters, borrowed data management—patterns for high-performance text processing.

2. **Async Scheduler with Pin**: Self-referential structs, Pin safety, HRTB for closures, async/await internals—understanding what makes async Rust work.

3. **Arena Allocator**: Variance, lifetime bounds, interior mutability, scoped lifetimes—advanced memory management with compile-time safety.

All three emphasize:
- **Zero-Cost Abstractions**: Lifetimes are compile-time only
- **Safety**: Prevent use-after-free at compile time
- **Flexibility**: Variance enables ergonomic APIs
- **Performance**: Lifetimes enable zero-copy patterns

Students will understand how Rust's lifetime system enables both safety and performance, preventing entire classes of bugs that plague C/C++ while maintaining zero runtime overhead.
