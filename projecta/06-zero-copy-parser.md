# Project 1: Zero-Copy Text Parser with Lifetime-Aware Tokens

## Problem Statement

Build a **zero-copy text parser** that borrows from the input without allocating, using Rust's lifetime system to ensure tokens remain valid. The parser demonstrates how lifetimes prevent use-after-free bugs while achieving performance 10-100x faster than allocation-based parsers.

**Use Cases**:
- Compiler lexers and tokenizers (parsing source code)
- JSON/XML parsers with minimal allocation
- Log file analyzers processing large files
- Configuration file parsers (INI, TOML, etc.)
- Network protocol parsers (HTTP headers, DNS packets)
- CSV/TSV data processors

## Why It Matters

Zero-copy parsing leverages **Rust's lifetime system** for correctness and performance:

**Memory Efficiency**:
- **Allocating parser**: Each token creates a `String` (heap allocation + copy)
- **Zero-copy parser**: Tokens are `&str` slices (just pointer + length, no allocation)
- **Example**: Parsing 1M tokens → 1M allocations vs 0 allocations

**Performance Impact**:
- **String allocation**: ~50-100ns per allocation + copy overhead
- **Slice borrowing**: ~1-2ns (just pointer arithmetic)
- **10-100x faster** for large inputs (benchmarks on real parsers)
- **Cache friendly**: No heap fragmentation, better locality

**Lifetime Safety**:
- **Without lifetimes** (C/C++): Tokens can outlive input → use-after-free bugs
- **With lifetimes**: Compiler guarantees tokens can't outlive input at compile time
- **Zero runtime cost**: Lifetime checks are compile-time only, erased after verification

**Real-World Examples**:
- `serde_json` uses zero-copy for string values (10x faster deserialization)
- `nom` parser combinator library: zero-copy by default
- `logos` lexer generator: zero-allocation token streams
- Rust compiler's lexer: borrows from source text

---

## Milestone 1: Basic Parser with Input Lifetime

**Goal**: Implement `Parser<'input>` that borrows from input text and tokenizes whitespace-separated words.

**Concepts**:
- Struct with lifetime parameter: `Parser<'a>`
- Borrowing input with `&'input str`
- Method lifetime elision (implicit `&self` lifetime)
- Returning borrowed slices with correct lifetime
- Position tracking without copying data

**Implementation Steps**:

1. **Define `Parser<'input>` struct**:
   - Field: `input: &'input str` (borrowed input, lives for 'input)
   - Field: `position: usize` (current parsing position)
   - Lifetime parameter ensures tokens can't outlive input

2. **Implement `new(input: &'input str) -> Self`**:
   - Store input reference
   - Initialize position to 0
   - Return parser (no allocation)

3. **Implement `peek(&self) -> Option<&'input str>`**:
   - Skip whitespace from current position
   - Find next word boundary (space, tab, newline)
   - Return `&input[start..end]` slice (borrowed, not copied)
   - Don't modify position (peek doesn't consume)

4. **Implement `advance(&mut self) -> Option<&'input str>`**:
   - Call `peek()` to get next token
   - If Some, update position to end of token
   - Return the token
   - If None (end of input), return None

5. **Implement `remaining(&self) -> &'input str`**:
   - Return `&input[position..]` (rest of unparsed input)
   - Lifetime automatically tied to 'input

**Starter Code**:

```rust
pub struct Parser<'input> {
    // TODO: Add field for input reference
    // Hint: &'input str

    // TODO: Add field for current position
    // Hint: usize
}

impl<'input> Parser<'input> {
    pub fn new(input: &'input str) -> Self {
        // TODO: Create parser with input and position 0
        todo!()
    }

    pub fn peek(&self) -> Option<&'input str> {
        // TODO: Skip whitespace from self.position
        // Hint: Use input[position..].chars() and skip_while(|c| c.is_whitespace())

        // TODO: Find end of word (next whitespace or end of input)
        // Hint: Use find(|c| c.is_whitespace())

        // TODO: Return slice &input[start..end]
        // If no word found, return None

        todo!()
    }

    pub fn advance(&mut self) -> Option<&'input str> {
        // TODO: Call peek() to get next token

        // TODO: If Some(token), update self.position to end of token
        // Hint: Calculate new position from token's byte range

        // TODO: Return the token

        todo!()
    }

    pub fn remaining(&self) -> &'input str {
        // TODO: Return unparsed portion of input
        // Hint: &self.input[self.position..]
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        // TODO: Check if remaining input is empty after skipping whitespace
        self.remaining().trim().is_empty()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_new() {
        let input = "hello world";
        let parser = Parser::new(input);
        assert_eq!(parser.remaining(), "hello world");
    }

    #[test]
    fn test_peek_does_not_consume() {
        let input = "foo bar baz";
        let parser = Parser::new(input);

        assert_eq!(parser.peek(), Some("foo"));
        assert_eq!(parser.peek(), Some("foo")); // Still "foo"
        assert_eq!(parser.remaining(), "foo bar baz"); // Unchanged
    }

    #[test]
    fn test_advance_consumes() {
        let mut parser = Parser::new("one two three");

        assert_eq!(parser.advance(), Some("one"));
        assert_eq!(parser.advance(), Some("two"));
        assert_eq!(parser.advance(), Some("three"));
        assert_eq!(parser.advance(), None);
    }

    #[test]
    fn test_remaining() {
        let mut parser = Parser::new("alpha beta gamma");

        parser.advance(); // Consume "alpha"
        assert_eq!(parser.remaining().trim(), "beta gamma");

        parser.advance(); // Consume "beta"
        assert_eq!(parser.remaining().trim(), "gamma");
    }

    #[test]
    fn test_whitespace_handling() {
        let mut parser = Parser::new("  spaced   out  ");

        assert_eq!(parser.advance(), Some("spaced"));
        assert_eq!(parser.advance(), Some("out"));
        assert_eq!(parser.advance(), None);
    }

    #[test]
    fn test_lifetime_bounds() {
        // This should compile: token lifetime tied to input
        let input = String::from("test data");
        let mut parser = Parser::new(&input);
        let token = parser.advance().unwrap();

        // token is valid as long as input is valid
        assert_eq!(token, "test");

        // This should NOT compile (uncomment to verify):
        // drop(input);
        // println!("{}", token); // ERROR: token can't outlive input
    }
}
```

**Check Your Understanding**:
1. Why does `Parser<'input>` need a lifetime parameter?
2. How does the compiler know that tokens returned by `advance()` have lifetime `'input`?
3. What would happen if we returned `String` instead of `&str` from `advance()`?

---

## Milestone 2: Token Enum with Multiple Lifetime Variants

**Goal**: Define a `Token<'a>` enum representing different token types, all borrowing from input.

**Concepts**:
- Enum with lifetime parameter
- Multiple variants with borrowed data
- Pattern matching on lifetime-aware enums
- Lifetime elision in enum methods
- Zero-cost token representation (no heap allocation)

**Implementation Steps**:

1. **Define `Token<'a>` enum**:
   - Variant: `Identifier(&'a str)` for variable names, keywords
   - Variant: `Number(&'a str)` for numeric literals (stored as slice, parsed later)
   - Variant: `String(&'a str)` for string literals (without quotes)
   - Variant: `Symbol(char)` for single-character operators (+, -, etc.)
   - Variant: `Eof` for end of input

2. **Implement `Token::as_str(&self) -> Option<&'a str>`**:
   - Return the string slice for variants that have one
   - Return None for Symbol and Eof

3. **Implement `Token::is_keyword(&self, word: &str) -> bool`**:
   - Check if token is an Identifier matching a specific keyword
   - Example: `token.is_keyword("if")`, `token.is_keyword("fn")`

4. **Update Parser to return `Token<'input>`**:
   - Modify `advance()` to classify tokens
   - Detect numbers (starts with digit)
   - Detect strings (between quotes)
   - Detect symbols (single char operators)
   - Default to Identifier for words

**Starter Code**:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    Identifier(&'a str),
    Number(&'a str),
    String(&'a str),
    Symbol(char),
    Eof,
}

impl<'a> Token<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        // TODO: Return &str for Identifier, Number, String
        // Return None for Symbol and Eof

        match self {
            Token::Identifier(s) | Token::Number(s) | Token::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_keyword(&self, word: &str) -> bool {
        // TODO: Check if self is Identifier matching word
        matches!(self, Token::Identifier(s) if *s == word)
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::Eof)
    }
}

impl<'input> Parser<'input> {
    pub fn next_token(&mut self) -> Token<'input> {
        // TODO: Skip whitespace

        // TODO: Check if at end of input
        // Return Token::Eof

        // TODO: Get remaining input
        let remaining = self.remaining();

        // TODO: Handle string literals (starts with ")
        // Find closing quote, extract content, update position
        // Return Token::String(content)

        // TODO: Handle numbers (starts with digit)
        // Find end of number, extract slice, update position
        // Return Token::Number(slice)

        // TODO: Handle single-char symbols
        // Check for: + - * / ( ) { } [ ] , ; = < > ! &  |
        // Update position by 1, return Token::Symbol(char)

        // TODO: Handle identifiers (alphanumeric + underscore)
        // Find end of identifier, extract slice, update position
        // Return Token::Identifier(slice)

        todo!()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_identifier() {
        let mut parser = Parser::new("foo bar_123");

        assert_eq!(parser.next_token(), Token::Identifier("foo"));
        assert_eq!(parser.next_token(), Token::Identifier("bar_123"));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_token_number() {
        let mut parser = Parser::new("42 3.14 0xFF");

        assert_eq!(parser.next_token(), Token::Number("42"));
        assert_eq!(parser.next_token(), Token::Number("3.14"));
        assert_eq!(parser.next_token(), Token::Number("0xFF"));
    }

    #[test]
    fn test_token_string() {
        let mut parser = Parser::new(r#""hello" "world""#);

        assert_eq!(parser.next_token(), Token::String("hello"));
        assert_eq!(parser.next_token(), Token::String("world"));
    }

    #[test]
    fn test_token_symbols() {
        let mut parser = Parser::new("+ - * / ( )");

        assert_eq!(parser.next_token(), Token::Symbol('+'));
        assert_eq!(parser.next_token(), Token::Symbol('-'));
        assert_eq!(parser.next_token(), Token::Symbol('*'));
        assert_eq!(parser.next_token(), Token::Symbol('/'));
        assert_eq!(parser.next_token(), Token::Symbol('('));
        assert_eq!(parser.next_token(), Token::Symbol(')'));
    }

    #[test]
    fn test_mixed_tokens() {
        let mut parser = Parser::new(r#"let x = 42 + "test""#);

        assert_eq!(parser.next_token(), Token::Identifier("let"));
        assert_eq!(parser.next_token(), Token::Identifier("x"));
        assert_eq!(parser.next_token(), Token::Symbol('='));
        assert_eq!(parser.next_token(), Token::Number("42"));
        assert_eq!(parser.next_token(), Token::Symbol('+'));
        assert_eq!(parser.next_token(), Token::String("test"));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_is_keyword() {
        let token = Token::Identifier("if");
        assert!(token.is_keyword("if"));
        assert!(!token.is_keyword("else"));

        let token = Token::Number("42");
        assert!(!token.is_keyword("if"));
    }

    #[test]
    fn test_token_lifetime_tied_to_input() {
        let input = String::from("test");
        let mut parser = Parser::new(&input);
        let token = parser.next_token();

        // Token is valid as long as input is valid
        assert_eq!(token, Token::Identifier("test"));

        // This should NOT compile (uncomment to verify):
        // drop(input);
        // println!("{:?}", token); // ERROR: token can't outlive input
    }
}
```

**Check Your Understanding**:
1. Why does `Token<'a>` have a lifetime parameter when `Symbol(char)` doesn't borrow?
2. How does the lifetime `'a` in `Token<'a>` relate to `'input` in `Parser<'input>`?
3. What's the memory layout difference between `Vec<Token<'a>>` and `Vec<String>`?

---

## Milestone 3: Iterator Implementation with Lifetime Bounds

**Goal**: Implement `Iterator` for `Parser<'input>` to enable idiomatic Rust iteration over tokens.

**Concepts**:
- Iterator trait with lifetime-aware `Item` type
- Lifetime bounds in trait implementations
- Lifetime elision in iterator methods
- Consuming vs borrowing iteration
- IntoIterator for owned parsers

**Implementation Steps**:

1. **Implement `Iterator` for `Parser<'input>`**:
   - `type Item = Token<'input>`
   - `fn next(&mut self) -> Option<Self::Item>`
   - Call `next_token()`, return None on Eof

2. **Implement `IntoIterator` for `Parser<'input>`**:
   - `type Item = Token<'input>`
   - `type IntoIter = Self`
   - `fn into_iter(self) -> Self::IntoIter`

3. **Add iterator adapter methods**:
   - `filter_keywords(&mut self) -> impl Iterator<Item = &'input str>`
   - `filter_identifiers(&mut self) -> impl Iterator<Item = &'input str>`
   - Demonstrate iterator chaining with zero-copy tokens

4. **Handle lifetime inference**:
   - Compiler infers lifetime bounds automatically
   - No explicit `where 'input: 'a` needed (modern Rust)

**Starter Code**:

```rust
impl<'input> Iterator for Parser<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Call next_token()
        // If Eof, return None
        // Otherwise, return Some(token)

        let token = self.next_token();
        if token.is_eof() {
            None
        } else {
            Some(token)
        }
    }
}

impl<'input> IntoIterator for Parser<'input> {
    type Item = Token<'input>;
    type IntoIter = Self;

    fn into_iter(self) -> Self::IntoIter {
        self
    }
}

impl<'input> Parser<'input> {
    // Collect all identifiers
    pub fn identifiers(&mut self) -> Vec<&'input str> {
        // TODO: Filter for Token::Identifier, extract &str
        self.filter_map(|token| match token {
            Token::Identifier(s) => Some(s),
            _ => None,
        })
        .collect()
    }

    // Check if keyword exists in input
    pub fn has_keyword(&mut self, keyword: &str) -> bool {
        // TODO: Use any() to check if any token is the keyword
        self.any(|token| token.is_keyword(keyword))
    }

    // Count tokens of a specific type
    pub fn count_numbers(&mut self) -> usize {
        // TODO: Filter for Token::Number, count them
        self.filter(|token| matches!(token, Token::Number(_)))
            .count()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator() {
        let parser = Parser::new("a b c");
        let tokens: Vec<Token> = parser.into_iter().collect();

        assert_eq!(tokens.len(), 3);
        assert!(tokens.iter().all(|t| matches!(t, Token::Identifier(_))));
    }

    #[test]
    fn test_iterator_chaining() {
        let parser = Parser::new("let x = 42");
        let identifiers: Vec<&str> = parser
            .into_iter()
            .filter_map(|token| match token {
                Token::Identifier(s) => Some(s),
                _ => None,
            })
            .collect();

        assert_eq!(identifiers, vec!["let", "x"]);
    }

    #[test]
    fn test_identifiers_method() {
        let mut parser = Parser::new("fn foo(x: i32) -> bool");
        let ids = parser.identifiers();

        assert_eq!(ids, vec!["fn", "foo", "x", "i32", "bool"]);
    }

    #[test]
    fn test_has_keyword() {
        let mut parser = Parser::new("if x > 0 { return true }");

        assert!(parser.has_keyword("if"));

        // Parser consumed, need new one
        let mut parser = Parser::new("let x = 42");
        assert!(!parser.has_keyword("if"));
        assert!(parser.has_keyword("let"));
    }

    #[test]
    fn test_count_numbers() {
        let mut parser = Parser::new("1 + 2 * 3 - 4");
        assert_eq!(parser.count_numbers(), 4);
    }

    #[test]
    fn test_iterator_zero_copy() {
        let input = String::from("foo bar baz");
        let parser = Parser::new(&input);

        // Collect tokens (no allocation for token content)
        let tokens: Vec<Token> = parser.into_iter().collect();

        // All identifiers should point into original input
        for token in tokens {
            if let Token::Identifier(s) = token {
                // Verify s is a slice of input (same pointer range)
                let input_ptr = input.as_ptr() as usize;
                let s_ptr = s.as_ptr() as usize;
                assert!(s_ptr >= input_ptr);
                assert!(s_ptr < input_ptr + input.len());
            }
        }
    }
}
```

**Check Your Understanding**:
1. Why can `Parser` implement `Iterator` even though it mutates state?
2. How does the lifetime `'input` flow through the iterator chain?
3. What's the difference between `into_iter()` consuming the parser vs `&mut self` methods?

---

## Milestone 4: Multiple Lifetime Parameters for Context

**Goal**: Add parser context with separate lifetime from input, demonstrating multiple lifetime parameters.

**Concepts**:
- Multiple lifetime parameters: `<'input, 'context>`
- Lifetime bounds: `'context: 'input` (context outlives input)
- Returning references with different lifetimes
- Method signatures with multiple lifetimes
- Lifetime parameter ordering and relationships

**Implementation Steps**:

1. **Define `ParserContext<'ctx>` struct**:
   - Stores configuration: keywords list, operators, etc.
   - Lifetime `'ctx` for borrowed keyword slice: `&'ctx [&'ctx str]`

2. **Update `Parser<'input, 'ctx>` with two lifetimes**:
   - `'input` for input text
   - `'ctx` for context reference
   - Field: `context: &'ctx ParserContext<'ctx>`

3. **Implement methods returning references with different lifetimes**:
   - `get_input_slice() -> &'input str` (returns from input)
   - `get_keywords() -> &'ctx [&'ctx str]` (returns from context)
   - `is_reserved_keyword() -> bool` (uses both lifetimes)

4. **Add lifetime bounds where needed**:
   - Modern Rust infers most bounds automatically
   - Explicit bounds only for complex relationships

**Starter Code**:

```rust
pub struct ParserContext<'ctx> {
    keywords: &'ctx [&'ctx str],
    operators: &'ctx [char],
}

impl<'ctx> ParserContext<'ctx> {
    pub fn new(keywords: &'ctx [&'ctx str], operators: &'ctx [char]) -> Self {
        Self { keywords, operators }
    }

    pub fn is_keyword(&self, word: &str) -> bool {
        self.keywords.contains(&word)
    }

    pub fn is_operator(&self, ch: char) -> bool {
        self.operators.contains(&ch)
    }
}

pub struct Parser<'input, 'ctx> {
    input: &'input str,
    position: usize,
    context: &'ctx ParserContext<'ctx>,
}

impl<'input, 'ctx> Parser<'input, 'ctx> {
    pub fn new(input: &'input str, context: &'ctx ParserContext<'ctx>) -> Self {
        Self {
            input,
            position: 0,
            context,
        }
    }

    // Returns reference with 'input lifetime
    pub fn remaining(&self) -> &'input str {
        &self.input[self.position..]
    }

    // Returns reference with 'ctx lifetime
    pub fn get_keywords(&self) -> &'ctx [&'ctx str] {
        self.context.keywords
    }

    pub fn next_token(&mut self) -> Token<'input> {
        // TODO: Similar to before, but use context.is_operator()
        // to classify symbols

        // Skip whitespace
        while self.position < self.input.len()
            && self.input[self.position..].chars().next().unwrap().is_whitespace()
        {
            self.position += 1;
        }

        if self.position >= self.input.len() {
            return Token::Eof;
        }

        let remaining = &self.input[self.position..];
        let first_char = remaining.chars().next().unwrap();

        // TODO: Check if first_char is operator using context
        if self.context.is_operator(first_char) {
            self.position += first_char.len_utf8();
            return Token::Symbol(first_char);
        }

        // TODO: Handle numbers, strings, identifiers (as before)
        todo!()
    }

    // Method using both lifetimes
    pub fn next_keyword_token(&mut self) -> Option<&'input str> {
        // TODO: Find next token that is a keyword (check against context)
        loop {
            match self.next_token() {
                Token::Identifier(s) if self.context.is_keyword(s) => return Some(s),
                Token::Eof => return None,
                _ => continue,
            }
        }
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_with_context() {
        let keywords = &["if", "else", "fn", "let"];
        let operators = &['+', '-', '*', '/'];
        let context = ParserContext::new(keywords, operators);

        let input = "if x + 1";
        let mut parser = Parser::new(input, &context);

        assert_eq!(parser.next_token(), Token::Identifier("if"));
        assert_eq!(parser.next_token(), Token::Identifier("x"));
        assert_eq!(parser.next_token(), Token::Symbol('+'));
        assert_eq!(parser.next_token(), Token::Number("1"));
    }

    #[test]
    fn test_keyword_detection() {
        let keywords = &["if", "else", "while"];
        let context = ParserContext::new(keywords, &[]);

        assert!(context.is_keyword("if"));
        assert!(!context.is_keyword("foo"));
    }

    #[test]
    fn test_next_keyword_token() {
        let keywords = &["if", "fn"];
        let operators = &['+'];
        let context = ParserContext::new(keywords, operators);

        let mut parser = Parser::new("foo if bar fn baz", &context);

        assert_eq!(parser.next_keyword_token(), Some("if"));
        assert_eq!(parser.next_keyword_token(), Some("fn"));
        assert_eq!(parser.next_keyword_token(), None);
    }

    #[test]
    fn test_multiple_lifetime_bounds() {
        let keywords = vec!["let".to_string(), "mut".to_string()];
        let keyword_refs: Vec<&str> = keywords.iter().map(|s| s.as_str()).collect();

        let context = ParserContext::new(&keyword_refs, &[]);
        let input = String::from("let mut x = 42");

        let mut parser = Parser::new(&input, &context);

        // Both input and context must outlive parser
        let token = parser.next_token();
        assert_eq!(token, Token::Identifier("let"));

        // This demonstrates two independent lifetimes
        let kw = parser.get_keywords();
        assert_eq!(kw.len(), 2);
    }

    #[test]
    fn test_lifetime_independence() {
        // Context lives longer than input
        let keywords = &["fn"];
        let context = ParserContext::new(keywords, &[]);

        {
            let input = String::from("fn foo");
            let mut parser = Parser::new(&input, &context);

            assert_eq!(parser.next_keyword_token(), Some("fn"));
            // input dropped here
        }

        // context still valid here
        assert!(context.is_keyword("fn"));
    }
}
```

**Check Your Understanding**:
1. Why do we need two separate lifetime parameters `'input` and `'ctx`?
2. Can `'input` outlive `'ctx`, or vice versa, or are they independent?
3. How would you add a lifetime bound `'ctx: 'input` and what would it mean?

---

## Milestone 5: Performance Comparison and Iterator Combinators

**Goal**: Benchmark zero-copy vs allocating parser, and demonstrate complex iterator chains with lifetime-aware tokens.

**Concepts**:
- Performance measurement of zero-copy vs String allocation
- Iterator combinators preserving lifetimes
- Collecting into lifetime-aware structures
- Demonstrating real-world parser patterns
- Memory profiling and allocation tracking

**Implementation Steps**:

1. **Create allocating parser for comparison**:
   - `AllocatingParser` that returns `Vec<String>` (owned tokens)
   - Compare performance: zero-copy vs allocating

2. **Implement complex iterator chains**:
   - Filter keywords, map to string slices, collect
   - Group consecutive tokens by type
   - Fold tokens into structured AST nodes

3. **Add benchmarking code**:
   - Parse 10,000 tokens with both parsers
   - Measure time and allocations
   - Demonstrate 10-100x speedup

4. **Demonstrate practical patterns**:
   - Expression parser using token stream
   - CSV parser with zero-copy fields
   - Log file analyzer

**Starter Code**:

```rust
// Allocating parser for comparison
pub struct AllocatingParser {
    input: String,
    position: usize,
}

impl AllocatingParser {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    pub fn next_token(&mut self) -> Option<String> {
        // TODO: Same logic as Parser, but return String (allocated)
        // This performs String::from() for each token

        todo!()
    }
}

// Performance comparison
pub fn benchmark_parsers(input: &str, iterations: usize) {
    use std::time::Instant;

    // Zero-copy parser
    let start = Instant::now();
    for _ in 0..iterations {
        let parser = Parser::new(input, &ParserContext::new(&[], &[]));
        let _tokens: Vec<Token> = parser.into_iter().collect();
    }
    let zero_copy_time = start.elapsed();

    // Allocating parser
    let start = Instant::now();
    for _ in 0..iterations {
        let mut parser = AllocatingParser::new(input.to_string());
        let mut tokens = Vec::new();
        while let Some(token) = parser.next_token() {
            tokens.push(token);
        }
    }
    let allocating_time = start.elapsed();

    println!("Zero-copy: {:?}", zero_copy_time);
    println!("Allocating: {:?}", allocating_time);
    println!(
        "Speedup: {:.2}x",
        allocating_time.as_secs_f64() / zero_copy_time.as_secs_f64()
    );
}

// Complex iterator patterns
impl<'input, 'ctx> Parser<'input, 'ctx> {
    // Collect all identifier/number pairs
    pub fn extract_assignments(&mut self) -> Vec<(&'input str, &'input str)> {
        // TODO: Find patterns like "name = value"
        // Return Vec of (identifier, number) tuples

        let mut result = Vec::new();
        let tokens: Vec<Token> = self.into_iter().collect();

        for window in tokens.windows(3) {
            if let [Token::Identifier(name), Token::Symbol('='), Token::Number(value)] = window {
                result.push((*name, *value));
            }
        }

        result
    }

    // Group consecutive identifiers
    pub fn group_identifiers(&mut self) -> Vec<Vec<&'input str>> {
        // TODO: Group consecutive Token::Identifier into Vec<Vec<&str>>

        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        for token in self {
            match token {
                Token::Identifier(s) => current_group.push(s),
                _ => {
                    if !current_group.is_empty() {
                        groups.push(current_group);
                        current_group = Vec::new();
                    }
                }
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        groups
    }
}

// Example: CSV parser
pub fn parse_csv_line<'a>(line: &'a str) -> Vec<&'a str> {
    // TODO: Split by comma, return slices (zero-copy)
    line.split(',').map(|s| s.trim()).collect()
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_assignments() {
        let context = ParserContext::new(&[], &['=']);
        let mut parser = Parser::new("x = 42 y = 100", &context);

        let assignments = parser.extract_assignments();
        assert_eq!(assignments, vec![("x", "42"), ("y", "100")]);
    }

    #[test]
    fn test_group_identifiers() {
        let context = ParserContext::new(&[], &['+']);
        let mut parser = Parser::new("foo bar + baz qux", &context);

        let groups = parser.group_identifiers();
        assert_eq!(groups, vec![vec!["foo", "bar"], vec!["baz", "qux"]]);
    }

    #[test]
    fn test_csv_zero_copy() {
        let line = "alice,bob,charlie";
        let fields = parse_csv_line(line);

        assert_eq!(fields, vec!["alice", "bob", "charlie"]);

        // Verify zero-copy: fields point into original line
        for field in fields {
            let line_ptr = line.as_ptr() as usize;
            let field_ptr = field.as_ptr() as usize;
            assert!(field_ptr >= line_ptr);
            assert!(field_ptr < line_ptr + line.len());
        }
    }

    #[test]
    fn test_benchmark() {
        // Large input with many tokens
        let input = (0..1000)
            .map(|i| format!("token{} ", i))
            .collect::<String>();

        // Run benchmark (just verify it doesn't crash)
        benchmark_parsers(&input, 100);
    }

    #[test]
    fn test_memory_efficiency() {
        use std::mem::size_of_val;

        let input = "test data here";
        let parser = Parser::new(input, &ParserContext::new(&[], &[]));
        let tokens: Vec<Token> = parser.into_iter().collect();

        // Each &str token is 2 * usize (pointer + length)
        // String would be 3 * usize (pointer + length + capacity) + heap allocation

        let token_size: usize = tokens.iter().map(|t| size_of_val(t)).sum();

        // Zero-copy: only stack allocation for Vec + token metadata
        println!("Token vector size: {} bytes", token_size);
        println!("Number of tokens: {}", tokens.len());
        println!("Avg bytes per token: {}", token_size / tokens.len());
    }
}
```

**Check Your Understanding**:
1. Why is zero-copy parsing 10-100x faster than allocating parsers?
2. How do iterator combinators preserve lifetime information through the chain?
3. When would you choose an allocating parser over a zero-copy parser?

---

## Summary

You've built a **complete zero-copy text parser** with:

1. **Basic Parser with Input Lifetime** - `Parser<'input>` borrowing from input
2. **Token Enum with Lifetime Variants** - `Token<'a>` with zero allocation
3. **Iterator Implementation** - Idiomatic Rust iteration over tokens
4. **Multiple Lifetime Parameters** - `Parser<'input, 'ctx>` for context
5. **Performance Comparison** - Demonstrating 10-100x speedup

**Key Patterns Learned**:
- **Struct lifetimes**: `Parser<'input>` ensures tokens can't outlive input
- **Lifetime elision**: Compiler infers lifetimes in most method signatures
- **Multiple lifetimes**: `'input` and `'ctx` for independent borrowed data
- **Lifetime bounds**: Implicit bounds from usage (modern Rust)
- **Iterator with lifetimes**: `Item = Token<'input>` preserves zero-copy
- **Zero-cost abstraction**: Lifetimes erased after compilation (no runtime cost)

**Performance Characteristics**:
- **Zero-copy**: 1-2ns per token (pointer arithmetic only)
- **Allocating**: 50-100ns per token (malloc + memcpy)
- **10-100x speedup** on real-world parsing workloads
- **Memory efficiency**: Vec of &str vs Vec of String (no heap fragmentation)
- **Cache friendly**: All tokens reference contiguous input buffer

**Real-World Applications**:
- Compiler lexers (Rust compiler, JavaScript engines)
- JSON/XML parsers (`serde_json` uses zero-copy)
- Log file analyzers (grep-like tools)
- Network protocol parsers (HTTP, DNS)
- CSV/data processors (streaming large files)

**Next Steps**:
- Add error recovery and position tracking for diagnostics
- Implement full expression parser (recursive descent)
- Support Unicode and multi-byte characters properly
- Add macro/comment handling (nested structures)
- Benchmark against real parsers (nom, pest, logos)
