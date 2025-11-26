## Project 1: Regular Expression Engine with Pattern Matching

### Problem Statement

Build a regular expression engine that:
- Parses regex patterns into an Abstract Syntax Tree (AST)
- Compiles patterns into a state machine representation
- Executes matches using exhaustive pattern matching
- Supports core regex features: literals, wildcards (.), character classes ([a-z]), quantifiers (*, +, ?), alternation (|), groups ()
- Implements backtracking for complex patterns
- Uses enum-driven architecture for regex operators
- Demonstrates deep destructuring for AST traversal
- Provides match extraction with capture groups
- Optimizes with pattern guards and range matching

The engine must showcase Rust's pattern matching for implementing a complete domain-specific language processor.

### Why It Matters

Regular expressions are fundamental to:
- **Text Processing**: Search, replace, validation in editors and tools
- **Compilers**: Lexical analysis (tokenization)
- **Data Validation**: Email, phone, URL validation
- **Log Processing**: Extract structured data from logs
- **Network Security**: Pattern matching in intrusion detection

Pattern matching is ideal for regex engines because:
- Enums naturally represent regex operators
- Match exhaustiveness ensures all patterns handled
- Guards enable optimization (e.g., literal string fast path)
- Destructuring simplifies AST traversal
- State machines map directly to Rust enums

### Use Cases

1. **Text Editors**: Find/replace with regex patterns
2. **Form Validation**: Email, password strength, input sanitization
3. **Log Analyzers**: Extract timestamp, level, message from logs
4. **Web Scrapers**: Extract structured data from HTML
5. **Compilers**: Tokenize source code
6. **Network Tools**: Grep-like search in packet data
7. **Data ETL**: Transform text data with pattern-based rules

### Solution Outline

**Core AST Structure:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(char),
    Wildcard,                              // .
    CharClass { chars: Vec<char>, negated: bool },  // [abc] or [^abc]
    Range { start: char, end: char },      // [a-z]
    Sequence(Vec<Regex>),                  // abc
    Alternation(Vec<Regex>),               // a|b|c
    Repeat { expr: Box<Regex>, min: usize, max: Option<usize> },  // a*, a+, a?
    Group { expr: Box<Regex>, capturing: bool },  // (a) or (?:a)
    Anchor { kind: AnchorKind },           // ^ or $
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorKind {
    Start,   // ^
    End,     // $
}
```

**Pattern Matching for Execution:**
```rust
impl Regex {
    pub fn matches(&self, input: &str) -> bool {
        match self {
            Regex::Literal(ch) => input.chars().next() == Some(*ch),
            Regex::Wildcard => input.chars().next().is_some(),
            Regex::CharClass { chars, negated } => {
                match input.chars().next() {
                    Some(ch) if *negated => !chars.contains(&ch),
                    Some(ch) => chars.contains(&ch),
                    None => false,
                }
            }
            Regex::Sequence(exprs) => self.match_sequence(input, exprs),
            Regex::Alternation(exprs) => {
                exprs.iter().any(|e| e.matches(input))
            }
            // ... other patterns
        }
    }
}
```

**Key Patterns to Demonstrate:**
- **Exhaustive Matching**: Every regex operator must be handled
- **Guards with Bindings**: `x @ 'a'..='z' if expensive(x)`
- **Nested Destructuring**: Extract from deeply nested AST
- **Or-Patterns**: Handle multiple similar cases
- **Range Patterns**: Character class matching

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_literal_match() {
    let regex = Regex::parse("hello").unwrap();
    assert!(regex.is_match("hello world"));
    assert!(!regex.is_match("goodbye"));
}

#[test]
fn test_quantifiers() {
    let regex = Regex::parse("a*b+c?").unwrap();
    assert!(regex.is_match("aaabbbbc"));
    assert!(regex.is_match("bc"));
    assert!(!regex.is_match("ac"));
}

#[test]
fn test_capture_groups() {
    let regex = Regex::parse(r"(\d+)-(\d+)-(\d+)").unwrap();
    let captures = regex.captures("2024-01-15").unwrap();
    assert_eq!(captures.get(1), Some("2024"));
    assert_eq!(captures.get(2), Some("01"));
    assert_eq!(captures.get(3), Some("15"));
}
```

**Property-Based Testing:**
```rust
#[quickcheck]
fn prop_literal_always_matches_itself(s: String) -> bool {
    let regex = Regex::Literal(s);
    regex.is_match(&s)
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Literal and Wildcard Matching

**Goal:** Implement simple string matching with literals and wildcards.

**What to implement:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Wildcard,  // .
    Sequence(Vec<Regex>),
}

impl Regex {
    pub fn is_match(&self, input: &str) -> bool {
        self.match_at(input, 0).is_some()
    }

    fn match_at(&self, input: &str, pos: usize) -> Option<usize> {
        match self {
            Regex::Literal(s) => {
                if input[pos..].starts_with(s) {
                    Some(pos + s.len())
                } else {
                    None
                }
            }

            Regex::Wildcard => {
                if pos < input.len() {
                    // Match any single character
                    let next_pos = pos + input[pos..].chars().next()?.len_utf8();
                    Some(next_pos)
                } else {
                    None
                }
            }

            Regex::Sequence(exprs) => {
                let mut current_pos = pos;
                for expr in exprs {
                    current_pos = expr.match_at(input, current_pos)?;
                }
                Some(current_pos)
            }
        }
    }

    pub fn parse(pattern: &str) -> Result<Self, ParseError> {
        let mut exprs = Vec::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            let expr = match ch {
                '.' => Regex::Wildcard,
                c => Regex::Literal(c.to_string()),
            };
            exprs.push(expr);
        }

        if exprs.len() == 1 {
            Ok(exprs.into_iter().next().unwrap())
        } else {
            Ok(Regex::Sequence(exprs))
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidSyntax(String),
}
```

**Check/Test:**
- Test `is_match("hello", "hello world")` returns true
- Test wildcard: `is_match("h.llo", "hello")` returns true
- Test sequence: `is_match("a.c", "abc")` returns true
- Test non-match: `is_match("hello", "world")` returns false

**Why this isn't enough:**
Only supports the most basic patterns. No quantifiers (* + ?), no character classes [a-z], no alternation (a|b), no groups. The pattern matching is trivial—we're not demonstrating guards, ranges, or complex destructuring. We need to add more regex features to showcase advanced pattern matching techniques.

---

### Step 2: Add Character Classes and Range Patterns

**Goal:** Implement character classes with range matching and pattern guards.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),  // Single char for efficiency
    Wildcard,
    CharClass {
        ranges: Vec<CharRange>,
        chars: Vec<char>,
        negated: bool,
    },
    Sequence(Vec<Regex>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharRange {
    start: char,
    end: char,
}

impl CharRange {
    fn contains(&self, ch: char) -> bool {
        ch >= self.start && ch <= self.end
    }
}

impl Regex {
    fn match_at(&self, input: &str, pos: usize) -> Option<usize> {
        let chars: Vec<char> = input.chars().collect();

        if pos >= chars.len() {
            return None;
        }

        match self {
            Regex::Char(expected) => {
                if chars[pos] == *expected {
                    Some(pos + 1)
                } else {
                    None
                }
            }

            Regex::Wildcard => Some(pos + 1),

            Regex::CharClass { ranges, chars: class_chars, negated } => {
                let ch = chars[pos];

                // Check if char matches any range
                let in_range = ranges.iter().any(|r| r.contains(ch));
                // Check if char is in explicit char list
                let in_chars = class_chars.contains(&ch);

                let matches = in_range || in_chars;

                // Apply negation if needed
                if *negated != matches {
                    Some(pos + 1)
                } else {
                    None
                }
            }

            Regex::Sequence(exprs) => {
                let mut current_pos = pos;
                for expr in exprs {
                    current_pos = expr.match_at(input, current_pos)?;
                }
                Some(current_pos)
            }

            _ => None,
        }
    }
}

// Enhanced parser
fn parse_char_class(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Regex, ParseError> {
    let mut ranges = Vec::new();
    let mut class_chars = Vec::new();
    let negated = chars.peek() == Some(&'^');

    if negated {
        chars.next();
    }

    while let Some(&ch) = chars.peek() {
        if ch == ']' {
            chars.next();
            break;
        }

        let start = chars.next().unwrap();

        // Check for range
        if chars.peek() == Some(&'-') {
            chars.next(); // consume '-'

            let end = chars.next()
                .ok_or_else(|| ParseError::InvalidSyntax("Incomplete range".into()))?;

            if end == ']' {
                // Special case: [a-] means 'a' or '-'
                class_chars.push(start);
                class_chars.push('-');
                break;
            }

            ranges.push(CharRange { start, end });
        } else {
            class_chars.push(start);
        }
    }

    Ok(Regex::CharClass {
        ranges,
        chars: class_chars,
        negated,
    })
}
```

**Pattern matching showcase:**
```rust
// Using range patterns with guards
fn matches_digit(ch: char) -> bool {
    match ch {
        '0'..='9' => true,
        _ => false,
    }
}

// Using pattern guards
fn matches_alphanum(ch: char) -> bool {
    match ch {
        c @ 'a'..='z' | c @ 'A'..='Z' | c @ '0'..='9' => true,
        _ => false,
    }
}

// Complex pattern matching for character classes
impl CharClass {
    fn matches(&self, ch: char) -> bool {
        match (ch, &self.ranges, &self.chars, self.negated) {
            // Fast path: common ranges
            (c @ '0'..='9', _, _, false) if self.is_digit_class() => true,
            (c @ 'a'..='z', _, _, false) if self.is_lower_class() => true,

            // General case
            (ch, ranges, chars, negated) => {
                let in_class = ranges.iter().any(|r| r.contains(ch))
                    || chars.contains(&ch);
                *negated != in_class
            }
        }
    }
}
```

**Check/Test:**
- Test `[a-z]` matches lowercase letters
- Test `[^0-9]` matches non-digits
- Test `[a-zA-Z0-9]` matches alphanumeric
- Test negated classes work correctly
- Test range patterns and guards compile

**Why this isn't enough:**
We have character classes but no quantifiers. Patterns like `a*`, `a+`, `a{2,5}` are essential for real regex. Quantifiers introduce backtracking complexity—we need to handle multiple possible match lengths. This is where pattern matching with state tracking becomes crucial. We also don't have alternation (|) or groups yet.

---

### Step 3: Add Quantifiers with Backtracking

**Goal:** Implement `*`, `+`, `?`, `{n,m}` quantifiers using exhaustive pattern matching for different cases.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    CharClass { ranges: Vec<CharRange>, chars: Vec<char>, negated: bool },
    Sequence(Vec<Regex>),
    Repeat {
        expr: Box<Regex>,
        min: usize,
        max: Option<usize>,
    },
}

impl Regex {
    fn match_at(&self, input: &str, pos: usize) -> Option<usize> {
        let chars: Vec<char> = input.chars().collect();

        match self {
            // ... previous cases ...

            Regex::Repeat { expr, min, max } => {
                self.match_repeat(input, pos, expr, *min, *max)
            }

            Regex::Sequence(exprs) => {
                let mut current_pos = pos;
                for expr in exprs {
                    current_pos = expr.match_at(input, current_pos)?;
                }
                Some(current_pos)
            }
        }
    }

    fn match_repeat(
        &self,
        input: &str,
        pos: usize,
        expr: &Regex,
        min: usize,
        max: Option<usize>,
    ) -> Option<usize> {
        // Greedy matching with backtracking
        let mut matches = Vec::new();
        let mut current_pos = pos;

        // Match as many as possible (greedy)
        loop {
            match expr.match_at(input, current_pos) {
                Some(new_pos) if Some(matches.len()) < max || max.is_none() => {
                    matches.push(new_pos);
                    current_pos = new_pos;
                }
                _ => break,
            }
        }

        // Backtrack if we matched more than minimum
        if matches.len() >= min {
            // Try from most matches to minimum
            for &end_pos in matches.iter().rev() {
                return Some(end_pos);
            }

            // If we have at least min matches, return position after min matches
            if matches.len() >= min {
                return Some(matches.get(min - 1).copied().unwrap_or(pos));
            }
        }

        None
    }

    // Helper to create common quantifiers
    pub fn zero_or_more(expr: Regex) -> Self {
        Regex::Repeat {
            expr: Box::new(expr),
            min: 0,
            max: None,
        }
    }

    pub fn one_or_more(expr: Regex) -> Self {
        Regex::Repeat {
            expr: Box::new(expr),
            min: 1,
            max: None,
        }
    }

    pub fn optional(expr: Regex) -> Self {
        Regex::Repeat {
            expr: Box::new(expr),
            min: 0,
            max: Some(1),
        }
    }
}

// Enhanced parser for quantifiers
impl Regex {
    pub fn parse(pattern: &str) -> Result<Self, ParseError> {
        let mut chars = pattern.chars().peekable();
        let exprs = Self::parse_sequence(&mut chars)?;

        Ok(if exprs.len() == 1 {
            exprs.into_iter().next().unwrap()
        } else {
            Regex::Sequence(exprs)
        })
    }

    fn parse_sequence(
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> Result<Vec<Regex>, ParseError> {
        let mut exprs = Vec::new();

        while let Some(&ch) = chars.peek() {
            let mut expr = match ch {
                '.' => {
                    chars.next();
                    Regex::Wildcard
                }
                '[' => {
                    chars.next();
                    parse_char_class(chars)?
                }
                '(' => {
                    chars.next();
                    Self::parse_group(chars)?
                }
                ')' | '|' => break,
                c => {
                    chars.next();
                    Regex::Char(c)
                }
            };

            // Check for quantifiers
            if let Some(&q) = chars.peek() {
                expr = match q {
                    '*' => {
                        chars.next();
                        Regex::zero_or_more(expr)
                    }
                    '+' => {
                        chars.next();
                        Regex::one_or_more(expr)
                    }
                    '?' => {
                        chars.next();
                        Regex::optional(expr)
                    }
                    '{' => {
                        chars.next();
                        Self::parse_counted_repeat(chars, expr)?
                    }
                    _ => expr,
                };
            }

            exprs.push(expr);
        }

        Ok(exprs)
    }

    fn parse_counted_repeat(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        expr: Regex,
    ) -> Result<Regex, ParseError> {
        let mut num_str = String::new();

        while let Some(&ch) = chars.peek() {
            if ch.is_numeric() {
                num_str.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        let min = num_str.parse::<usize>()
            .map_err(|_| ParseError::InvalidSyntax("Invalid repeat count".into()))?;

        let (min, max) = match chars.peek() {
            Some(&',') => {
                chars.next();
                let mut max_str = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_numeric() {
                        max_str.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let max = if max_str.is_empty() {
                    None
                } else {
                    Some(max_str.parse::<usize>()
                        .map_err(|_| ParseError::InvalidSyntax("Invalid repeat count".into()))?)
                };

                (min, max)
            }
            _ => (min, Some(min)),
        };

        if chars.next() != Some('}') {
            return Err(ParseError::InvalidSyntax("Expected '}'".into()));
        }

        Ok(Regex::Repeat {
            expr: Box::new(expr),
            min,
            max,
        })
    }
}
```

**Pattern matching for quantifiers:**
```rust
// Exhaustive pattern matching for quantifier types
fn optimize_quantifier(repeat: &Regex) -> Regex {
    match repeat {
        // * (zero or more) - most common, optimize
        Regex::Repeat { min: 0, max: None, expr } => {
            match expr.as_ref() {
                Regex::Char(c) => {
                    // Optimized char repetition
                    Regex::Repeat {
                        expr: Box::new(Regex::Char(*c)),
                        min: 0,
                        max: None,
                    }
                }
                _ => repeat.clone(),
            }
        }

        // + (one or more)
        Regex::Repeat { min: 1, max: None, .. } => repeat.clone(),

        // ? (optional)
        Regex::Repeat { min: 0, max: Some(1), .. } => repeat.clone(),

        // {n} (exactly n)
        Regex::Repeat { min, max: Some(max), .. } if min == max => repeat.clone(),

        // {n,m} (between n and m)
        Regex::Repeat { .. } => repeat.clone(),

        _ => repeat.clone(),
    }
}
```

**Check/Test:**
- Test `a*` matches "", "a", "aaa"
- Test `a+` matches "a", "aaa" but not ""
- Test `a?` matches "", "a"
- Test `a{2,4}` matches "aa", "aaa", "aaaa"
- Test greedy matching: `a*a` on "aaa" should match all
- Test backtracking when needed

**Why this isn't enough:**
Quantifiers work but our backtracking is naive—exponential time on pathological cases like `a*a*a*b` matching "aaaaaaaaac". Real regex engines use memoization or NFA/DFA compilation. We also lack alternation (|), capture groups, and anchors (^ $). These features will showcase more advanced pattern matching patterns like nested destructuring and or-patterns.

---

### Step 4: Add Alternation and Groups with Deep Destructuring

**Goal:** Implement alternation (a|b) and capture groups, demonstrating nested destructuring.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    CharClass { ranges: Vec<CharRange>, chars: Vec<char>, negated: bool },
    Sequence(Vec<Regex>),
    Repeat { expr: Box<Regex>, min: usize, max: Option<usize> },
    Alternation(Vec<Regex>),  // a|b|c
    Group { expr: Box<Regex>, id: Option<usize> },  // (a) with optional capture ID
}

#[derive(Debug, Clone)]
pub struct Match<'a> {
    pub full_match: &'a str,
    pub captures: Vec<Option<&'a str>>,
}

impl Regex {
    pub fn find<'a>(&self, input: &'a str) -> Option<Match<'a>> {
        let chars: Vec<char> = input.chars().collect();
        let mut captures = Vec::new();

        for start in 0..=chars.len() {
            captures.clear();
            if let Some(end) = self.match_with_captures(input, start, &mut captures) {
                let matched_text = &input[start..end];
                return Some(Match {
                    full_match: matched_text,
                    captures: captures.clone(),
                });
            }
        }

        None
    }

    fn match_with_captures(
        &self,
        input: &str,
        pos: usize,
        captures: &mut Vec<Option<&str>>,
    ) -> Option<usize> {
        match self {
            Regex::Char(expected) => {
                let chars: Vec<char> = input.chars().collect();
                if pos < chars.len() && chars[pos] == *expected {
                    Some(pos + 1)
                } else {
                    None
                }
            }

            Regex::Alternation(alternatives) => {
                // Try each alternative (or-pattern in action)
                for alt in alternatives {
                    if let Some(end) = alt.match_with_captures(input, pos, captures) {
                        return Some(end);
                    }
                }
                None
            }

            Regex::Group { expr, id } => {
                let start_pos = pos;
                let end_pos = expr.match_with_captures(input, pos, captures)?;

                // Record capture if this is a capturing group
                if let Some(capture_id) = id {
                    while captures.len() <= *capture_id {
                        captures.push(None);
                    }
                    captures[*capture_id] = Some(&input[start_pos..end_pos]);
                }

                Some(end_pos)
            }

            Regex::Sequence(exprs) => {
                let mut current_pos = pos;
                for expr in exprs {
                    current_pos = expr.match_with_captures(input, current_pos, captures)?;
                }
                Some(current_pos)
            }

            Regex::Repeat { expr, min, max } => {
                let mut count = 0;
                let mut current_pos = pos;

                // Match minimum required
                for _ in 0..*min {
                    current_pos = expr.match_with_captures(input, current_pos, captures)?;
                    count += 1;
                }

                // Match up to maximum (greedy)
                while max.map_or(true, |m| count < m) {
                    match expr.match_with_captures(input, current_pos, captures) {
                        Some(new_pos) => {
                            current_pos = new_pos;
                            count += 1;
                        }
                        None => break,
                    }
                }

                Some(current_pos)
            }

            // ... other cases
            _ => None,
        }
    }
}

// Parser for alternation and groups
impl Regex {
    fn parse_sequence(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        capture_counter: &mut usize,
    ) -> Result<Vec<Regex>, ParseError> {
        let mut alternatives = vec![Vec::new()];

        while let Some(&ch) = chars.peek() {
            match ch {
                ')' => break,

                '|' => {
                    chars.next();
                    alternatives.push(Vec::new());
                }

                '(' => {
                    chars.next();
                    let group = Self::parse_group(chars, capture_counter)?;
                    alternatives.last_mut().unwrap().push(group);
                }

                _ => {
                    let expr = Self::parse_atom(chars)?;
                    let expr = Self::parse_quantifier(chars, expr)?;
                    alternatives.last_mut().unwrap().push(expr);
                }
            }
        }

        // Build result
        if alternatives.len() == 1 {
            Ok(alternatives.into_iter().next().unwrap())
        } else {
            let alt_seqs: Vec<Regex> = alternatives
                .into_iter()
                .map(|exprs| {
                    if exprs.len() == 1 {
                        exprs.into_iter().next().unwrap()
                    } else {
                        Regex::Sequence(exprs)
                    }
                })
                .collect();

            Ok(vec![Regex::Alternation(alt_seqs)])
        }
    }

    fn parse_group(
        chars: &mut std::iter::Peekable<std::str::Chars>,
        capture_counter: &mut usize,
    ) -> Result<Regex, ParseError> {
        // Check for non-capturing group (?:...)
        let capturing = if chars.peek() == Some(&'?') {
            chars.next();
            if chars.next() != Some(':') {
                return Err(ParseError::InvalidSyntax("Expected ':' after '?'".into()));
            }
            false
        } else {
            true
        };

        let capture_id = if capturing {
            let id = *capture_counter;
            *capture_counter += 1;
            Some(id)
        } else {
            None
        };

        let exprs = Self::parse_sequence(chars, capture_counter)?;

        if chars.next() != Some(')') {
            return Err(ParseError::InvalidSyntax("Expected ')'".into()));
        }

        let expr = if exprs.len() == 1 {
            exprs.into_iter().next().unwrap()
        } else {
            Regex::Sequence(exprs)
        };

        Ok(Regex::Group {
            expr: Box::new(expr),
            id: capture_id,
        })
    }
}
```

**Deep destructuring examples:**
```rust
// Pattern matching with deep destructuring
fn extract_captures(regex: &Regex) -> Vec<usize> {
    match regex {
        // Nested destructuring
        Regex::Group { expr: box Regex::Sequence(exprs), id: Some(id) } => {
            let mut ids = vec![*id];
            for expr in exprs {
                ids.extend(extract_captures(expr));
            }
            ids
        }

        // Destructure alternation with nested groups
        Regex::Alternation(alts) => {
            alts.iter().flat_map(|alt| extract_captures(alt)).collect()
        }

        // Destructure repeat with group inside
        Regex::Repeat { expr: box Regex::Group { id: Some(id), .. }, .. } => {
            vec![*id]
        }

        // Match and destructure sequence
        Regex::Sequence(exprs) => {
            exprs.iter().flat_map(|e| extract_captures(e)).collect()
        }

        _ => Vec::new(),
    }
}

// Pattern guards for optimization
fn can_optimize(regex: &Regex) -> bool {
    match regex {
        // Simple patterns that can be optimized
        Regex::Sequence(exprs) if exprs.iter().all(|e| matches!(e, Regex::Char(_))) => true,

        // Repeated character class
        Regex::Repeat { expr: box Regex::CharClass { .. }, .. } => true,

        // Alternation of literals
        Regex::Alternation(alts) if alts.iter().all(|a| matches!(a, Regex::Literal(_))) => true,

        _ => false,
    }
}
```

**Check/Test:**
- Test `a|b|c` matches any of "a", "b", "c"
- Test `(a)(b)(c)` captures all three groups
- Test nested groups: `((a)(b))` captures correctly
- Test non-capturing: `(?:a|b)c` matches but doesn't capture a or b
- Test complex: `(a|b)*c` with captures
- Verify deep destructuring extracts all capture IDs

**Why this isn't enough:**
We have most regex features but the engine is still slow. No memoization means patterns like `(a|b)*c` on large inputs backtrack exponentially. We also don't have anchors (^ $), lookahead/lookbehind, or word boundaries (\b). The implementation doesn't demonstrate if-let chains or let-else patterns yet. Let's add optimizations and more advanced pattern matching features.

---

### Step 5: Add Anchors and Optimization with Pattern Matching

**Goal:** Implement anchors, optimize with if-let chains, and add memoization.

**What to improve:**

**1. Add anchors:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    // ... existing variants ...
    Anchor(AnchorKind),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnchorKind {
    Start,      // ^
    End,        // $
    WordBound,  // \b
}

impl Regex {
    fn match_with_captures(
        &self,
        input: &str,
        pos: usize,
        captures: &mut Vec<Option<&str>>,
    ) -> Option<usize> {
        match self {
            Regex::Anchor(anchor_kind) => {
                match anchor_kind {
                    AnchorKind::Start if pos == 0 => Some(pos),
                    AnchorKind::End if pos == input.len() => Some(pos),
                    AnchorKind::WordBound => {
                        let at_word_boundary = self.is_word_boundary(input, pos);
                        if at_word_boundary {
                            Some(pos)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }

            // ... existing cases with if-let chains for optimization
            Regex::Sequence(exprs) => {
                // Optimize common patterns with if-let chains
                if let [Regex::Anchor(AnchorKind::Start), rest @ .., Regex::Anchor(AnchorKind::End)] = exprs.as_slice()
                    && pos == 0
                {
                    // Full string match - optimize by requiring exact length
                    let mut current_pos = pos;
                    for expr in rest {
                        current_pos = expr.match_with_captures(input, current_pos, captures)?;
                    }

                    if current_pos == input.len() {
                        return Some(current_pos);
                    } else {
                        return None;
                    }
                }

                // General case
                let mut current_pos = pos;
                for expr in exprs {
                    current_pos = expr.match_with_captures(input, current_pos, captures)?;
                }
                Some(current_pos)
            }

            _ => { /* ... existing cases ... */ }
        }
    }

    fn is_word_boundary(&self, input: &str, pos: usize) -> bool {
        let chars: Vec<char> = input.chars().collect();

        let before_is_word = if pos > 0 {
            chars[pos - 1].is_alphanumeric() || chars[pos - 1] == '_'
        } else {
            false
        };

        let after_is_word = if pos < chars.len() {
            chars[pos].is_alphanumeric() || chars[pos] == '_'
        } else {
            false
        };

        before_is_word != after_is_word
    }
}
```

**2. Add memoization:**
```rust
use std::collections::HashMap;

pub struct RegexMatcher<'a> {
    regex: &'a Regex,
    input: &'a str,
    memo: HashMap<(usize, usize), Option<usize>>,  // (regex_id, pos) -> end_pos
}

impl<'a> RegexMatcher<'a> {
    pub fn new(regex: &'a Regex, input: &'a str) -> Self {
        RegexMatcher {
            regex,
            input,
            memo: HashMap::new(),
        }
    }

    pub fn find(&mut self) -> Option<Match<'a>> {
        for start in 0..=self.input.len() {
            self.memo.clear();
            let mut captures = Vec::new();

            if let Some(end) = self.match_memo(self.regex, start, 0, &mut captures) {
                return Some(Match {
                    full_match: &self.input[start..end],
                    captures,
                });
            }
        }

        None
    }

    fn match_memo(
        &mut self,
        regex: &'a Regex,
        pos: usize,
        regex_id: usize,
        captures: &mut Vec<Option<&'a str>>,
    ) -> Option<usize> {
        // Check memo
        if let Some(&result) = self.memo.get(&(regex_id, pos)) {
            return result;
        }

        let result = match regex {
            Regex::Sequence(exprs) => {
                let mut current_pos = pos;
                let mut id = regex_id;

                for expr in exprs {
                    current_pos = self.match_memo(expr, current_pos, id, captures)?;
                    id += 1;
                }

                Some(current_pos)
            }

            Regex::Alternation(alts) => {
                for (i, alt) in alts.iter().enumerate() {
                    if let Some(end) = self.match_memo(alt, pos, regex_id + i, captures) {
                        return Some(end);
                    }
                }
                None
            }

            // ... other cases
            _ => regex.match_with_captures(self.input, pos, captures),
        };

        // Memoize
        self.memo.insert((regex_id, pos), result);
        result
    }
}
```

**3. Use if-let chains and let-else:**
```rust
// If-let chains for validation
impl Regex {
    pub fn parse(pattern: &str) -> Result<Self, ParseError> {
        let mut chars = pattern.chars().peekable();
        let mut capture_counter = 0;

        let exprs = Self::parse_sequence(&mut chars, &mut capture_counter)?;

        // Let-else for early validation
        let Some(regex) = Self::simplify_sequence(exprs) else {
            return Err(ParseError::InvalidSyntax("Empty pattern".into()));
        };

        Ok(regex)
    }

    fn simplify_sequence(exprs: Vec<Regex>) -> Option<Regex> {
        // If-let chain for optimization
        if let [single] = exprs.as_slice()
            && !matches!(single, Regex::Sequence(_))
        {
            return Some(single.clone());
        }

        if let [] = exprs.as_slice() {
            return None;
        }

        Some(Regex::Sequence(exprs))
    }
}

// Let-else for early returns
fn validate_and_execute(regex: &Regex, input: &str) -> Result<bool, ValidationError> {
    // Validate regex structure
    let Some(capture_count) = count_captures(regex) else {
        return Err(ValidationError::InvalidRegex);
    };

    // Validate input
    let Some(first_char) = input.chars().next() else {
        return Err(ValidationError::EmptyInput);
    };

    // Execute
    Ok(regex.is_match(input))
}
```

**4. While-let for iteration:**
```rust
// While-let for consuming backtrack stack
fn match_with_backtrack(
    regex: &Regex,
    input: &str,
    mut pos: usize,
) -> Option<usize> {
    let mut backtrack_stack: Vec<(usize, Vec<Regex>)> = vec![(pos, vec![regex.clone()])];

    while let Some((current_pos, remaining)) = backtrack_stack.pop() {
        if remaining.is_empty() {
            return Some(current_pos);
        }

        let (first, rest) = remaining.split_first()?;

        match first {
            Regex::Char(ch) => {
                if let Some(input_ch) = input.chars().nth(current_pos)
                    && input_ch == *ch
                {
                    backtrack_stack.push((current_pos + 1, rest.to_vec()));
                }
            }

            Regex::Alternation(alts) => {
                // Add all alternatives to backtrack stack
                for alt in alts.iter().rev() {
                    let mut new_remaining = vec![alt.clone()];
                    new_remaining.extend_from_slice(rest);
                    backtrack_stack.push((current_pos, new_remaining));
                }
            }

            _ => {
                // ... handle other cases
            }
        }
    }

    None
}
```

**Check/Test:**
- Test `^hello` matches only at start
- Test `world$` matches only at end
- Test `^hello world$` matches exact string
- Test `\bhello\b` matches word boundary
- Test memoization improves performance
- Test if-let chains compile
- Benchmark: with vs without memoization

**Why this isn't enough:**
The engine works well but lacks error reporting. When a regex doesn't match, users don't know why. Production regex engines provide detailed error messages and match debugging. We also don't demonstrate matches! macro or advanced enum-driven patterns. Let's add comprehensive error handling and pattern matching utilities.

---

### Step 6: Add Error Reporting and Regex Analysis with Exhaustive Patterns

**Goal:** Rich error reporting, regex analysis, and showcase all pattern matching features.

**What to improve:**

**1. Comprehensive error types:**
```rust
#[derive(Debug, Clone)]
pub enum MatchError<'a> {
    NoMatch {
        input: &'a str,
        attempted_at: usize,
        reason: NoMatchReason,
    },
    PartialMatch {
        matched: &'a str,
        remaining: &'a str,
        expected: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum NoMatchReason {
    AnchorMismatch { expected: AnchorKind, at: usize },
    CharMismatch { expected: char, found: char, at: usize },
    CharClassMismatch { expected: String, found: char, at: usize },
    QuantifierFailed { min: usize, found: usize, at: usize },
    AlternativesFailed { alternatives: Vec<String> },
}

impl Regex {
    pub fn find_with_error<'a>(&self, input: &'a str) -> Result<Match<'a>, MatchError<'a>> {
        for start in 0..=input.len() {
            let mut captures = Vec::new();
            let mut error = None;

            match self.match_with_diagnosis(input, start, &mut captures, &mut error) {
                Some(end) => {
                    return Ok(Match {
                        full_match: &input[start..end],
                        captures,
                    });
                }
                None if start == 0 => {
                    if let Some(err) = error {
                        return Err(err);
                    }
                }
                _ => {}
            }
        }

        Err(MatchError::NoMatch {
            input,
            attempted_at: 0,
            reason: NoMatchReason::AlternativesFailed {
                alternatives: vec![self.to_string()],
            },
        })
    }

    fn match_with_diagnosis<'a>(
        &self,
        input: &'a str,
        pos: usize,
        captures: &mut Vec<Option<&'a str>>,
        error: &mut Option<MatchError<'a>>,
    ) -> Option<usize> {
        match self {
            Regex::Char(expected) => {
                let chars: Vec<char> = input.chars().collect();

                if pos >= chars.len() {
                    *error = Some(MatchError::NoMatch {
                        input,
                        attempted_at: pos,
                        reason: NoMatchReason::CharMismatch {
                            expected: *expected,
                            found: '\0',
                            at: pos,
                        },
                    });
                    return None;
                }

                if chars[pos] != *expected {
                    *error = Some(MatchError::NoMatch {
                        input,
                        attempted_at: pos,
                        reason: NoMatchReason::CharMismatch {
                            expected: *expected,
                            found: chars[pos],
                            at: pos,
                        },
                    });
                    return None;
                }

                Some(pos + 1)
            }

            // ... other cases with detailed error tracking
            _ => None,
        }
    }
}
```

**2. Regex analysis with exhaustive patterns:**
```rust
// Analyze regex complexity
pub fn analyze_complexity(regex: &Regex) -> ComplexityReport {
    match regex {
        // Literals and char classes are O(1)
        Regex::Char(_) | Regex::Literal(_) | Regex::CharClass { .. } => {
            ComplexityReport {
                time_complexity: "O(1)".into(),
                backtracking: false,
                capture_groups: 0,
            }
        }

        // Anchors are O(1)
        Regex::Anchor(_) => ComplexityReport {
            time_complexity: "O(1)".into(),
            backtracking: false,
            capture_groups: 0,
        },

        // Sequences combine complexities
        Regex::Sequence(exprs) => {
            let reports: Vec<_> = exprs.iter().map(analyze_complexity).collect();
            ComplexityReport {
                time_complexity: combine_complexities(&reports),
                backtracking: reports.iter().any(|r| r.backtracking),
                capture_groups: reports.iter().map(|r| r.capture_groups).sum(),
            }
        }

        // Alternation can cause backtracking
        Regex::Alternation(alts) => {
            let reports: Vec<_> = alts.iter().map(analyze_complexity).collect();
            ComplexityReport {
                time_complexity: format!("O({} alternatives)", alts.len()),
                backtracking: true,
                capture_groups: reports.iter().map(|r| r.capture_groups).max().unwrap_or(0),
            }
        }

        // Quantifiers can be exponential
        Regex::Repeat { expr, min, max } => {
            let inner = analyze_complexity(expr);
            let complexity = match (min, max) {
                (0, None) | (1, None) => "O(n) to O(n²)".into(),
                (m, Some(n)) if m == n => format!("O({})", n),
                _ => "O(n) to O(2ⁿ)".into(),
            };

            ComplexityReport {
                time_complexity: complexity,
                backtracking: true,
                capture_groups: inner.capture_groups,
            }
        }

        // Groups add captures
        Regex::Group { expr, id } => {
            let mut report = analyze_complexity(expr);
            if id.is_some() {
                report.capture_groups += 1;
            }
            report
        }

        Regex::Wildcard => ComplexityReport {
            time_complexity: "O(1)".into(),
            backtracking: false,
            capture_groups: 0,
        },
    }
}

#[derive(Debug)]
pub struct ComplexityReport {
    pub time_complexity: String,
    pub backtracking: bool,
    pub capture_groups: usize,
}
```

**3. Pattern matching utilities with matches! macro:**
```rust
// Utility functions using exhaustive patterns
pub fn is_simple_literal(regex: &Regex) -> bool {
    matches!(regex, Regex::Literal(_) | Regex::Char(_))
}

pub fn has_captures(regex: &Regex) -> bool {
    match regex {
        Regex::Group { id: Some(_), .. } => true,
        Regex::Sequence(exprs) | Regex::Alternation(exprs) => {
            exprs.iter().any(has_captures)
        }
        Regex::Repeat { expr, .. } | Regex::Group { expr, .. } => {
            has_captures(expr)
        }
        _ => false,
    }
}

pub fn optimize(regex: Regex) -> Regex {
    match regex {
        // Flatten nested sequences
        Regex::Sequence(exprs) => {
            let flattened: Vec<Regex> = exprs
                .into_iter()
                .flat_map(|e| match optimize(e) {
                    Regex::Sequence(inner) => inner,
                    other => vec![other],
                })
                .collect();

            match flattened.len() {
                0 => Regex::Sequence(vec![]),
                1 => flattened.into_iter().next().unwrap(),
                _ => Regex::Sequence(flattened),
            }
        }

        // Merge adjacent literals
        Regex::Sequence(exprs) if exprs.iter().all(|e| matches!(e, Regex::Char(_))) => {
            let s: String = exprs
                .iter()
                .filter_map(|e| match e {
                    Regex::Char(c) => Some(*c),
                    _ => None,
                })
                .collect();
            Regex::Literal(s)
        }

        // Simplify alternations with single branch
        Regex::Alternation(alts) if alts.len() == 1 => {
            optimize(alts.into_iter().next().unwrap())
        }

        // Remove empty sequences
        Regex::Sequence(exprs) if exprs.is_empty() => {
            Regex::Sequence(vec![])
        }

        // Recursively optimize
        Regex::Repeat { expr, min, max } => {
            Regex::Repeat {
                expr: Box::new(optimize(*expr)),
                min,
                max,
            }
        }

        Regex::Group { expr, id } => {
            Regex::Group {
                expr: Box::new(optimize(*expr)),
                id,
            }
        }

        Regex::Alternation(alts) => {
            Regex::Alternation(alts.into_iter().map(optimize).collect())
        }

        // Keep others as-is
        other => other,
    }
}
```

**4. Complete example with all patterns:**
```rust
pub fn validate_email(email: &str) -> Result<EmailParts, ValidationError> {
    // Simplified email regex: (.+)@(.+)\.(.+)
    let regex = Regex::parse(r"(.+)@(.+)\.(.+)").unwrap();

    let Some(m) = regex.find(email) else {
        return Err(ValidationError::InvalidFormat);
    };

    let [local, domain, tld] = match m.captures.as_slice() {
        [Some(l), Some(d), Some(t)] => [l, d, t],
        _ => return Err(ValidationError::MissingParts),
    };

    Ok(EmailParts {
        local: local.to_string(),
        domain: domain.to_string(),
        tld: tld.to_string(),
    })
}

// Showcase all pattern matching features
pub fn describe_regex(regex: &Regex) -> String {
    match regex {
        // Simple patterns
        Regex::Literal(s) => format!("literal '{}'", s),
        Regex::Char(c) => format!("char '{}'", c),
        Regex::Wildcard => "any character".into(),

        // Character class with destructuring
        Regex::CharClass { ranges, chars, negated: false } => {
            format!("one of: {:?} or ranges {:?}", chars, ranges)
        }
        Regex::CharClass { negated: true, .. } => {
            "not in character class".into()
        }

        // Quantifiers with pattern guards
        Regex::Repeat { min: 0, max: None, expr } => {
            format!("zero or more of ({})", describe_regex(expr))
        }
        Regex::Repeat { min: 1, max: None, expr } => {
            format!("one or more of ({})", describe_regex(expr))
        }
        Regex::Repeat { min: 0, max: Some(1), expr } => {
            format!("optionally ({})", describe_regex(expr))
        }
        Regex::Repeat { min, max: Some(max), expr } if min == max => {
            format!("exactly {} of ({})", min, describe_regex(expr))
        }
        Regex::Repeat { min, max, expr } => {
            let max_str = max.map_or("∞".to_string(), |m| m.to_string());
            format!("{} to {} of ({})", min, max_str, describe_regex(expr))
        }

        // Groups
        Regex::Group { expr, id: Some(id) } => {
            format!("capture group {} ({})", id, describe_regex(expr))
        }
        Regex::Group { expr, id: None } => {
            format!("non-capturing group ({})", describe_regex(expr))
        }

        // Alternation
        Regex::Alternation(alts) => {
            let descriptions: Vec<_> = alts.iter()
                .map(|a| describe_regex(a))
                .collect();
            format!("one of: {}", descriptions.join(" | "))
        }

        // Sequence with slice patterns
        Regex::Sequence(exprs) => match exprs.as_slice() {
            [] => "empty".into(),
            [single] => describe_regex(single),
            [first, rest @ ..] => {
                let first_desc = describe_regex(first);
                let rest_desc: Vec<_> = rest.iter()
                    .map(|e| describe_regex(e))
                    .collect();
                format!("{} then {}", first_desc, rest_desc.join(" then "))
            }
        },

        // Anchors
        Regex::Anchor(AnchorKind::Start) => "start of string".into(),
        Regex::Anchor(AnchorKind::End) => "end of string".into(),
        Regex::Anchor(AnchorKind::WordBound) => "word boundary".into(),
    }
}
```

**Check/Test:**
- Test error messages are helpful
- Test complexity analysis identifies backtracking
- Test optimization merges adjacent literals
- Test describe_regex produces readable output
- Test all pattern matching features compile
- Test matches! macro usage
- Verify exhaustiveness catches all cases

**What this achieves:**
A complete regex engine demonstrating:
- **Exhaustive Pattern Matching**: All enum variants handled
- **Guards and Ranges**: Quantifier optimization
- **Deep Destructuring**: AST traversal and analysis
- **If-Let Chains**: Validation sequences
- **Let-Else**: Early returns
- **While-Let**: Backtracking iteration
- **Matches! Macro**: Pattern testing utilities
- **Enum-Driven Architecture**: Regex operators as enums

**Extensions to explore:**
- Compile to DFA for O(n) matching
- Unicode support (character classes, case-insensitive)
- Backreferences (\1, \2)
- Lookahead and lookbehind
- Named capture groups
- Regex debugging/visualization

---
