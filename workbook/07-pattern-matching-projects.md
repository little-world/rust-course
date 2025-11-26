# Chapter 7: Pattern Matching & Destructuring - Programming Projects

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

## Project 2: Network Packet Inspector with Binary Pattern Matching

### Problem Statement

Build a network packet analyzer that:
- Parses binary network protocols (Ethernet, IP, TCP, UDP, HTTP)
- Uses pattern matching to destructure packet headers
- Implements protocol-aware filtering rules
- Supports deep packet inspection with nested destructuring
- Provides firewall rule engine using enum-driven architecture
- Extracts payload data with byte slice patterns
- Demonstrates range matching for port numbers and IP addresses
- Handles protocol variants (IPv4 vs IPv6, TCP vs UDP)

The tool must showcase pattern matching on binary data and protocol layers.

### Why It Matters

Network analysis is essential for:
- **Security**: Firewalls, intrusion detection, malware analysis
- **Debugging**: Protocol tracing, performance analysis
- **Monitoring**: Traffic analysis, bandwidth monitoring
- **Compliance**: Data loss prevention, audit logging

Pattern matching excels for packet parsing because:
- Protocol headers map directly to struct destructuring
- Enums represent protocol types naturally
- Match exhaustiveness ensures all protocols handled
- Range patterns perfect for port/address filtering
- Guards enable complex filtering rules

### Use Cases

1. **Firewalls**: Filter packets by IP, port, protocol
2. **IDS/IPS**: Detect malicious patterns in traffic
3. **Packet Capture**: tcpdump/Wireshark functionality
4. **Load Balancers**: Route packets by content
5. **VPN/Proxy**: Inspect and modify packets
6. **Network Monitoring**: Track bandwidth, connections
7. **Protocol Testing**: Validate protocol implementations

### Solution Outline

**Core Protocol Enums:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum EtherType {
    IPv4,
    IPv6,
    ARP,
    Other(u16),
}

#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        dst_mac: [u8; 6],
        src_mac: [u8; 6],
        ethertype: EtherType,
        payload: Box<Packet>,
    },
    IPv4 {
        version: u8,
        header_len: u8,
        src_ip: [u8; 4],
        dst_ip: [u8; 4],
        protocol: IpProtocol,
        payload: Box<Packet>,
    },
    TCP {
        src_port: u16,
        dst_port: u16,
        seq: u32,
        ack: u32,
        flags: TcpFlags,
        payload: Vec<u8>,
    },
    UDP {
        src_port: u16,
        dst_port: u16,
        payload: Vec<u8>,
    },
    HTTP {
        method: HttpMethod,
        path: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    },
    Raw(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpProtocol {
    TCP,
    UDP,
    ICMP,
    Other(u8),
}
```

**Pattern Matching for Filtering:**
```rust
impl Packet {
    pub fn matches_filter(&self, filter: &PacketFilter) -> bool {
        match (self, filter) {
            // Match specific ports with range patterns
            (
                Packet::TCP { src_port, dst_port, .. },
                PacketFilter::Port(p @ 1..=1023) // Well-known ports
            ) => *src_port == *p || *dst_port == *p,

            // Deep destructuring for nested protocols
            (
                Packet::Ethernet {
                    payload: box Packet::IPv4 {
                        src_ip,
                        protocol: IpProtocol::TCP,
                        payload: box Packet::TCP { dst_port, .. },
                        ..
                    },
                    ..
                },
                PacketFilter::HttpTraffic
            ) if *dst_port == 80 || *dst_port == 443 => true,

            // Exhaustive protocol matching
            (packet, filter) => self.deep_match(packet, filter),
        }
    }
}
```

**Testing Hints:**
```rust
#[test]
fn test_tcp_packet_parsing() {
    let raw = &[/* TCP packet bytes */];
    let packet = Packet::parse(raw).unwrap();

    match packet {
        Packet::TCP { src_port: 80, dst_port, .. } => {
            assert!(dst_port > 1024);
        }
        _ => panic!("Expected TCP packet"),
    }
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Parse Ethernet Frames with Byte Slice Patterns

**Goal:** Parse Ethernet layer using slice destructuring.

**What to implement:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MacAddress([u8; 6]);

#[derive(Debug, Clone, PartialEq)]
pub enum EtherType {
    IPv4,     // 0x0800
    IPv6,     // 0x86DD
    ARP,      // 0x0806
    Unknown(u16),
}

#[derive(Debug, Clone)]
pub struct EthernetFrame {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub ethertype: EtherType,
    pub payload: Vec<u8>,
}

impl EthernetFrame {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Pattern match on slice length and destructure
        match data {
            // Minimum Ethernet frame: 14 bytes header + payload
            [dst @ .., src @ .., eth_type @ .., payload @ ..]
                if dst.len() == 6 && src.len() == 6 && eth_type.len() == 2 =>
            {
                let dst_mac = MacAddress([
                    dst[0], dst[1], dst[2], dst[3], dst[4], dst[5]
                ]);

                let src_mac = MacAddress([
                    src[0], src[1], src[2], src[3], src[4], src[5]
                ]);

                let ethertype_value = u16::from_be_bytes([eth_type[0], eth_type[1]]);

                let ethertype = match ethertype_value {
                    0x0800 => EtherType::IPv4,
                    0x86DD => EtherType::IPv6,
                    0x0806 => EtherType::ARP,
                    other => EtherType::Unknown(other),
                };

                Ok(EthernetFrame {
                    dst_mac,
                    src_mac,
                    ethertype,
                    payload: payload.to_vec(),
                })
            }

            // Better approach: explicit indexing
            data if data.len() >= 14 => {
                let dst_mac = MacAddress([
                    data[0], data[1], data[2], data[3], data[4], data[5]
                ]);

                let src_mac = MacAddress([
                    data[6], data[7], data[8], data[9], data[10], data[11]
                ]);

                let ethertype_value = u16::from_be_bytes([data[12], data[13]]);

                let ethertype = match ethertype_value {
                    0x0800 => EtherType::IPv4,
                    0x86DD => EtherType::IPv6,
                    0x0806 => EtherType::ARP,
                    other => EtherType::Unknown(other),
                };

                Ok(EthernetFrame {
                    dst_mac,
                    src_mac,
                    ethertype,
                    payload: data[14..].to_vec(),
                })
            }

            _ => Err(ParseError::TooShort {
                expected: 14,
                found: data.len(),
            }),
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    TooShort { expected: usize, found: usize },
    InvalidProtocol(u8),
    Malformed(String),
}
```

**Check/Test:**
- Test parsing valid Ethernet frame
- Test different ethertypes
- Test too-short buffer returns error
- Test MAC address extraction

**Why this isn't enough:**
Only parses Ethernet layer. Real packet analysis needs IP, TCP, UDP layers. Pattern matching is basic—we're not demonstrating guards, nested destructuring, or complex filtering. We need to parse the protocol stack recursively and showcase deep pattern matching.

---

### Step 2: Add IPv4 Parsing with Nested Destructuring

**Goal:** Parse IP layer and demonstrate nested protocol destructuring.

**What to improve:**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Address([a, b, c, d])
    }

    pub fn is_private(&self) -> bool {
        match self.0 {
            [10, _, _, _] => true,                    // 10.0.0.0/8
            [172, b, _, _] if (16..=31).contains(&b) => true,  // 172.16.0.0/12
            [192, 168, _, _] => true,                 // 192.168.0.0/16
            _ => false,
        }
    }

    pub fn is_loopback(&self) -> bool {
        matches!(self.0, [127, _, _, _])
    }

    pub fn is_multicast(&self) -> bool {
        matches!(self.0, [a, _, _, _] if (224..=239).contains(&a))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpProtocol {
    ICMP,   // 1
    TCP,    // 6
    UDP,    // 17
    Unknown(u8),
}

#[derive(Debug, Clone)]
pub struct Ipv4Packet {
    pub version: u8,
    pub header_length: u8,
    pub total_length: u16,
    pub ttl: u8,
    pub protocol: IpProtocol,
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub payload: Vec<u8>,
}

impl Ipv4Packet {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        match data {
            // IPv4 minimum header: 20 bytes
            [version_ihl, _tos, total_len @ .., _id @ .., _flags @ .., ttl, protocol, _checksum @ .., src @ .., dst @ .., rest @ ..]
                if data.len() >= 20 =>
            {
                let version = (version_ihl >> 4) & 0x0F;
                let header_length = (version_ihl & 0x0F) * 4;

                if version != 4 {
                    return Err(ParseError::InvalidProtocol(version));
                }

                // Better approach with explicit slicing
                let total_length = u16::from_be_bytes([data[2], data[3]]);
                let ttl = data[8];
                let protocol_num = data[9];

                let protocol = match protocol_num {
                    1 => IpProtocol::ICMP,
                    6 => IpProtocol::TCP,
                    17 => IpProtocol::UDP,
                    other => IpProtocol::Unknown(other),
                };

                let src_ip = Ipv4Address([data[12], data[13], data[14], data[15]]);
                let dst_ip = Ipv4Address([data[16], data[17], data[18], data[19]]);

                let header_len = header_length as usize;
                let payload = if data.len() > header_len {
                    data[header_len..].to_vec()
                } else {
                    vec![]
                };

                Ok(Ipv4Packet {
                    version,
                    header_length,
                    total_length,
                    ttl,
                    protocol,
                    src_ip,
                    dst_ip,
                    payload,
                })
            }

            _ => Err(ParseError::TooShort {
                expected: 20,
                found: data.len(),
            }),
        }
    }
}

// Layered packet representation
#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        frame: EthernetFrame,
        inner: Option<Box<Packet>>,
    },
    IPv4 {
        packet: Ipv4Packet,
        inner: Option<Box<Packet>>,
    },
    Raw(Vec<u8>),
}

impl Packet {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let ethernet = EthernetFrame::parse(data)?;

        let inner = match ethernet.ethertype {
            EtherType::IPv4 => {
                let ipv4 = Ipv4Packet::parse(&ethernet.payload)?;
                Some(Box::new(Packet::IPv4 {
                    packet: ipv4,
                    inner: None,
                }))
            }
            _ => None,
        };

        Ok(Packet::Ethernet {
            frame: ethernet,
            inner,
        })
    }

    // Deep destructuring to extract information
    pub fn extract_ips(&self) -> Option<(Ipv4Address, Ipv4Address)> {
        match self {
            // Nested destructuring
            Packet::Ethernet {
                inner: Some(box Packet::IPv4 { packet: Ipv4Packet { src_ip, dst_ip, .. }, .. }),
                ..
            } => Some((*src_ip, *dst_ip)),

            Packet::IPv4 { packet: Ipv4Packet { src_ip, dst_ip, .. }, .. } => {
                Some((*src_ip, *dst_ip))
            }

            _ => None,
        }
    }
}
```

**Pattern matching for IP classification:**
```rust
fn classify_traffic(packet: &Packet) -> TrafficType {
    match packet {
        // Local traffic
        Packet::IPv4 {
            packet: Ipv4Packet {
                src_ip: src @ Ipv4Address([10, _, _, _]),
                dst_ip: dst @ Ipv4Address([10, _, _, _]),
                ..
            },
            ..
        } => TrafficType::LocalPrivate,

        // Internet-bound traffic
        Packet::IPv4 {
            packet: Ipv4Packet {
                src_ip,
                dst_ip,
                ..
            },
            ..
        } if src_ip.is_private() && !dst_ip.is_private() => TrafficType::Outbound,

        // Multicast
        Packet::IPv4 {
            packet: Ipv4Packet {
                dst_ip,
                ..
            },
            ..
        } if dst_ip.is_multicast() => TrafficType::Multicast,

        _ => TrafficType::Other,
    }
}

#[derive(Debug, PartialEq)]
enum TrafficType {
    LocalPrivate,
    Outbound,
    Inbound,
    Multicast,
    Other,
}
```

**Check/Test:**
- Test IPv4 parsing
- Test private IP detection
- Test nested packet parsing (Ethernet → IPv4)
- Test deep destructuring for IP extraction
- Test traffic classification

**Why this isn't enough:**
We parse IP but not transport layer (TCP/UDP). Can't inspect port numbers or flags. Pattern matching for filtering is limited—we need complex filter rules. Real packet inspection requires TCP/UDP parsing with state tracking (connections, sessions).

---

### Step 3: Add TCP/UDP Parsing and Port Range Matching

**Goal:** Parse transport layer and demonstrate range patterns for port filtering.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub struct TcpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub flags: TcpFlags,
    pub window_size: u16,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
}

impl TcpFlags {
    fn from_byte(byte: u8) -> Self {
        TcpFlags {
            fin: (byte & 0x01) != 0,
            syn: (byte & 0x02) != 0,
            rst: (byte & 0x04) != 0,
            psh: (byte & 0x08) != 0,
            ack: (byte & 0x10) != 0,
            urg: (byte & 0x20) != 0,
        }
    }
}

impl TcpPacket {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 20 {
            return Err(ParseError::TooShort {
                expected: 20,
                found: data.len(),
            });
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let seq_num = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let ack_num = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        let data_offset = (data[12] >> 4) * 4;
        let flags = TcpFlags::from_byte(data[13]);
        let window_size = u16::from_be_bytes([data[14], data[15]]);

        let header_len = data_offset as usize;
        let payload = if data.len() > header_len {
            data[header_len..].to_vec()
        } else {
            vec![]
        };

        Ok(TcpPacket {
            src_port,
            dst_port,
            seq_num,
            ack_num,
            flags,
            window_size,
            payload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct UdpPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
    pub payload: Vec<u8>,
}

impl UdpPacket {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        if data.len() < 8 {
            return Err(ParseError::TooShort {
                expected: 8,
                found: data.len(),
            });
        }

        let src_port = u16::from_be_bytes([data[0], data[1]]);
        let dst_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);

        let payload = data[8..].to_vec();

        Ok(UdpPacket {
            src_port,
            dst_port,
            length,
            payload,
        })
    }
}

// Enhanced packet enum
#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        frame: EthernetFrame,
        inner: Option<Box<Packet>>,
    },
    IPv4 {
        packet: Ipv4Packet,
        inner: Option<Box<Packet>>,
    },
    TCP(TcpPacket),
    UDP(UdpPacket),
    Raw(Vec<u8>),
}

// Pattern matching for port ranges
fn classify_port(port: u16) -> PortClass {
    match port {
        0 => PortClass::Reserved,
        1..=1023 => PortClass::WellKnown,
        1024..=49151 => PortClass::Registered,
        49152..=65535 => PortClass::Dynamic,
    }
}

#[derive(Debug, PartialEq)]
enum PortClass {
    Reserved,
    WellKnown,
    Registered,
    Dynamic,
}

// Service detection with range patterns
fn detect_service(packet: &Packet) -> Option<Service> {
    match packet {
        Packet::TCP(TcpPacket { dst_port: 80 | 8080 | 8000, .. }) => Some(Service::Http),
        Packet::TCP(TcpPacket { dst_port: 443 | 8443, .. }) => Some(Service::Https),
        Packet::TCP(TcpPacket { dst_port: 22, .. }) => Some(Service::SSH),
        Packet::TCP(TcpPacket { dst_port: 21 | 20, .. }) => Some(Service::FTP),
        Packet::TCP(TcpPacket { dst_port: 25, .. }) => Some(Service::SMTP),
        Packet::TCP(TcpPacket { dst_port: p @ 3306 | p @ 5432, .. }) => Some(Service::Database),
        Packet::UDP(UdpPacket { dst_port: 53, .. }) => Some(Service::DNS),
        Packet::UDP(UdpPacket { dst_port: 67 | 68, .. }) => Some(Service::DHCP),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
enum Service {
    Http,
    Https,
    SSH,
    FTP,
    SMTP,
    Database,
    DNS,
    DHCP,
}
```

**Check/Test:**
- Test TCP parsing with flags
- Test UDP parsing
- Test port classification
- Test service detection with or-patterns
- Test range patterns compile correctly

**Why this isn't enough:**
We parse packets but have no filtering engine. Real firewalls need complex rule matching with multiple criteria. We also don't handle HTTP payload inspection. The pattern matching showcases ranges and or-patterns but not guards, if-let chains, or exhaustive rule engines. Let's build a complete firewall rule system.

---

### Step 4: Add Firewall Rule Engine with Guards and Complex Patterns

**Goal:** Implement a firewall rule engine demonstrating guards, exhaustive matching, and complex filters.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum FirewallRule {
    AllowAll,
    DenyAll,
    AllowPort { port: u16 },
    DenyPort { port: u16 },
    AllowPortRange { start: u16, end: u16 },
    AllowIp { ip: Ipv4Address },
    DenyIp { ip: Ipv4Address },
    AllowSubnet { network: Ipv4Address, mask: u8 },
    AllowService(Service),
    DenyService(Service),
    Complex {
        src_ip: Option<Ipv4Address>,
        dst_ip: Option<Ipv4Address>,
        src_port: Option<u16>,
        dst_port: Option<u16>,
        protocol: Option<IpProtocol>,
        action: Action,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Allow,
    Deny,
    Log,
    LogAndAllow,
    LogAndDeny,
}

#[derive(Debug)]
pub struct Firewall {
    rules: Vec<FirewallRule>,
    default_action: Action,
}

impl Firewall {
    pub fn new(default_action: Action) -> Self {
        Firewall {
            rules: Vec::new(),
            default_action,
        }
    }

    pub fn add_rule(&mut self, rule: FirewallRule) {
        self.rules.push(rule);
    }

    pub fn check_packet(&self, packet: &Packet) -> Action {
        // Try each rule in order
        for rule in &self.rules {
            if let Some(action) = self.match_rule(rule, packet) {
                return action;
            }
        }

        self.default_action
    }

    fn match_rule(&self, rule: &FirewallRule, packet: &Packet) -> Option<Action> {
        match (rule, packet) {
            // Simple allow/deny all
            (FirewallRule::AllowAll, _) => Some(Action::Allow),
            (FirewallRule::DenyAll, _) => Some(Action::Deny),

            // Port-based rules with exhaustive protocol matching
            (FirewallRule::AllowPort { port }, Packet::TCP(tcp))
                if tcp.src_port == *port || tcp.dst_port == *port =>
            {
                Some(Action::Allow)
            }

            (FirewallRule::AllowPort { port }, Packet::UDP(udp))
                if udp.src_port == *port || udp.dst_port == *port =>
            {
                Some(Action::Allow)
            }

            // Port range with guards
            (
                FirewallRule::AllowPortRange { start, end },
                Packet::TCP(TcpPacket { dst_port, .. })
            ) if (*start..=*end).contains(dst_port) => Some(Action::Allow),

            (
                FirewallRule::AllowPortRange { start, end },
                Packet::UDP(UdpPacket { dst_port, .. })
            ) if (*start..=*end).contains(dst_port) => Some(Action::Allow),

            // Deep destructuring for nested packets with guards
            (
                FirewallRule::AllowIp { ip },
                Packet::Ethernet {
                    inner: Some(box Packet::IPv4 { packet, .. }),
                    ..
                }
            ) if packet.src_ip == *ip || packet.dst_ip == *ip => Some(Action::Allow),

            (
                FirewallRule::AllowIp { ip },
                Packet::IPv4 { packet, .. }
            ) if packet.src_ip == *ip || packet.dst_ip == *ip => Some(Action::Allow),

            // Subnet matching with guards
            (
                FirewallRule::AllowSubnet { network, mask },
                Packet::IPv4 { packet, .. }
            ) if Self::in_subnet(&packet.dst_ip, network, *mask) => Some(Action::Allow),

            // Service-based rules
            (FirewallRule::AllowService(service), packet)
                if detect_service(packet) == Some(*service) =>
            {
                Some(Action::Allow)
            }

            (FirewallRule::DenyService(service), packet)
                if detect_service(packet) == Some(*service) =>
            {
                Some(Action::Deny)
            }

            // Complex rule with multiple criteria
            (
                FirewallRule::Complex {
                    src_ip,
                    dst_ip,
                    src_port,
                    dst_port,
                    protocol,
                    action,
                },
                packet,
            ) => {
                // Extract packet details
                let packet_info = PacketInfo::extract(packet)?;

                // Check all criteria with if-let chains
                let src_ip_matches = src_ip
                    .map(|ip| packet_info.src_ip == Some(ip))
                    .unwrap_or(true);

                let dst_ip_matches = dst_ip
                    .map(|ip| packet_info.dst_ip == Some(ip))
                    .unwrap_or(true);

                let src_port_matches = src_port
                    .map(|port| packet_info.src_port == Some(port))
                    .unwrap_or(true);

                let dst_port_matches = dst_port
                    .map(|port| packet_info.dst_port == Some(port))
                    .unwrap_or(true);

                let protocol_matches = protocol
                    .map(|proto| packet_info.protocol == Some(proto))
                    .unwrap_or(true);

                if src_ip_matches
                    && dst_ip_matches
                    && src_port_matches
                    && dst_port_matches
                    && protocol_matches
                {
                    Some(*action)
                } else {
                    None
                }
            }

            // No match
            _ => None,
        }
    }

    fn in_subnet(ip: &Ipv4Address, network: &Ipv4Address, mask: u8) -> bool {
        let ip_bits = u32::from_be_bytes(ip.0);
        let net_bits = u32::from_be_bytes(network.0);
        let mask_bits = !0u32 << (32 - mask);

        (ip_bits & mask_bits) == (net_bits & mask_bits)
    }
}

// Helper to extract packet info
#[derive(Debug)]
struct PacketInfo {
    src_ip: Option<Ipv4Address>,
    dst_ip: Option<Ipv4Address>,
    src_port: Option<u16>,
    dst_port: Option<u16>,
    protocol: Option<IpProtocol>,
}

impl PacketInfo {
    fn extract(packet: &Packet) -> Option<Self> {
        match packet {
            // Deep destructuring for full packet info
            Packet::Ethernet {
                inner: Some(box Packet::IPv4 {
                    packet: ipv4,
                    inner: Some(box Packet::TCP(tcp)),
                }),
                ..
            } => Some(PacketInfo {
                src_ip: Some(ipv4.src_ip),
                dst_ip: Some(ipv4.dst_ip),
                src_port: Some(tcp.src_port),
                dst_port: Some(tcp.dst_port),
                protocol: Some(IpProtocol::TCP),
            }),

            Packet::Ethernet {
                inner: Some(box Packet::IPv4 {
                    packet: ipv4,
                    inner: Some(box Packet::UDP(udp)),
                }),
                ..
            } => Some(PacketInfo {
                src_ip: Some(ipv4.src_ip),
                dst_ip: Some(ipv4.dst_ip),
                src_port: Some(udp.src_port),
                dst_port: Some(udp.dst_port),
                protocol: Some(IpProtocol::UDP),
            }),

            Packet::IPv4 { packet, .. } => Some(PacketInfo {
                src_ip: Some(packet.src_ip),
                dst_ip: Some(packet.dst_ip),
                src_port: None,
                dst_port: None,
                protocol: Some(packet.protocol),
            }),

            Packet::TCP(tcp) => Some(PacketInfo {
                src_ip: None,
                dst_ip: None,
                src_port: Some(tcp.src_port),
                dst_port: Some(tcp.dst_port),
                protocol: Some(IpProtocol::TCP),
            }),

            Packet::UDP(udp) => Some(PacketInfo {
                src_ip: None,
                dst_ip: None,
                src_port: Some(udp.src_port),
                dst_port: Some(udp.dst_port),
                protocol: Some(IpProtocol::UDP),
            }),

            _ => None,
        }
    }
}
```

**Pattern matching utilities:**
```rust
// Using matches! macro for quick checks
pub fn is_tcp_syn(packet: &Packet) -> bool {
    matches!(
        packet,
        Packet::TCP(TcpPacket { flags: TcpFlags { syn: true, ack: false, .. }, .. })
    )
}

pub fn is_tcp_ack(packet: &Packet) -> bool {
    matches!(
        packet,
        Packet::TCP(TcpPacket { flags: TcpFlags { ack: true, .. }, .. })
    )
}

// Exhaustive pattern matching for TCP flags
fn classify_tcp_packet(flags: &TcpFlags) -> TcpPacketType {
    match (flags.syn, flags.ack, flags.fin, flags.rst) {
        (true, false, false, false) => TcpPacketType::Syn,
        (true, true, false, false) => TcpPacketType::SynAck,
        (false, true, false, false) => TcpPacketType::Ack,
        (false, true, true, false) => TcpPacketType::FinAck,
        (false, false, false, true) => TcpPacketType::Rst,
        (false, true, false, true) => TcpPacketType::RstAck,
        _ => TcpPacketType::Other,
    }
}

#[derive(Debug, PartialEq)]
enum TcpPacketType {
    Syn,
    SynAck,
    Ack,
    FinAck,
    Rst,
    RstAck,
    Other,
}
```

**Check/Test:**
- Test firewall allows/denies based on ports
- Test subnet matching works correctly
- Test complex rules with multiple criteria
- Test deep destructuring extracts all packet info
- Test exhaustive TCP flag matching
- Benchmark: rule evaluation performance

**Why this isn't enough:**
Firewall rules work but we don't inspect payload content. Real IDS/IPS systems need deep packet inspection—analyzing HTTP headers, extracting strings from payloads, detecting malicious patterns. We also don't handle stateful inspection (tracking TCP connections). Let's add HTTP parsing and payload analysis.

---

### Step 5: Deep Packet Inspection with HTTP Parsing and Pattern Guards

**Goal:** Parse HTTP over TCP and demonstrate payload inspection with pattern guards.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn parse(data: &[u8]) -> Result<Self, ParseError> {
        // Convert to string for parsing
        let text = String::from_utf8_lossy(data);

        // Split headers and body
        let parts: Vec<&str> = text.splitn(2, "\r\n\r\n").collect();

        let header_section = parts[0];
        let body = parts.get(1).map(|s| s.as_bytes().to_vec()).unwrap_or_default();

        // Parse request line and headers
        let mut lines = header_section.lines();

        let Some(request_line) = lines.next() else {
            return Err(ParseError::Malformed("Missing request line".into()));
        };

        // Parse request line with pattern matching
        let request_parts: Vec<&str> = request_line.split_whitespace().collect();

        let (method_str, path, version) = match request_parts.as_slice() {
            [method, path, version] => (*method, *path, *version),
            _ => {
                return Err(ParseError::Malformed(
                    "Invalid request line format".into(),
                ))
            }
        };

        let method = match method_str {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "HEAD" => HttpMethod::HEAD,
            "OPTIONS" => HttpMethod::OPTIONS,
            "PATCH" => HttpMethod::PATCH,
            other => HttpMethod::Other(other.to_string()),
        };

        // Parse headers
        let mut headers = Vec::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(HttpRequest {
            method,
            path: path.to_string(),
            version: version.to_string(),
            headers,
            body,
        })
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

// Enhanced packet enum with HTTP
#[derive(Debug, Clone)]
pub enum Packet {
    Ethernet {
        frame: EthernetFrame,
        inner: Option<Box<Packet>>,
    },
    IPv4 {
        packet: Ipv4Packet,
        inner: Option<Box<Packet>>,
    },
    TCP(TcpPacket),
    UDP(UdpPacket),
    Http(HttpRequest),
    Raw(Vec<u8>),
}

impl Packet {
    pub fn parse_full(data: &[u8]) -> Result<Self, ParseError> {
        let ethernet = EthernetFrame::parse(data)?;

        let inner = match ethernet.ethertype {
            EtherType::IPv4 => {
                let ipv4 = Ipv4Packet::parse(&ethernet.payload)?;

                let transport = match ipv4.protocol {
                    IpProtocol::TCP => {
                        let tcp = TcpPacket::parse(&ipv4.payload)?;

                        // Try to parse HTTP if it's HTTP ports
                        match tcp.dst_port {
                            80 | 8080 | 8000 if !tcp.payload.is_empty() => {
                                if let Ok(http) = HttpRequest::parse(&tcp.payload) {
                                    Packet::Http(http)
                                } else {
                                    Packet::TCP(tcp)
                                }
                            }
                            _ => Packet::TCP(tcp),
                        }
                    }

                    IpProtocol::UDP => Packet::UDP(UdpPacket::parse(&ipv4.payload)?),

                    _ => Packet::Raw(ipv4.payload.clone()),
                };

                Some(Box::new(Packet::IPv4 {
                    packet: ipv4,
                    inner: Some(Box::new(transport)),
                }))
            }
            _ => None,
        };

        Ok(Packet::Ethernet {
            frame: ethernet,
            inner,
        })
    }
}

// Deep packet inspection with pattern matching
pub fn inspect_http_request(packet: &Packet) -> Option<HttpInspectionReport> {
    match packet {
        // Deep destructuring through all layers
        Packet::Ethernet {
            inner: Some(box Packet::IPv4 {
                packet: ipv4,
                inner: Some(box Packet::Http(http)),
            }),
            ..
        } => {
            let mut report = HttpInspectionReport {
                src_ip: ipv4.src_ip,
                dst_ip: ipv4.dst_ip,
                method: http.method.clone(),
                path: http.path.clone(),
                user_agent: http.get_header("User-Agent").map(String::from),
                content_type: http.get_header("Content-Type").map(String::from),
                threats: Vec::new(),
            };

            // Detect threats using pattern guards
            report.threats.extend(detect_threats(http));

            Some(report)
        }

        _ => None,
    }
}

#[derive(Debug)]
pub struct HttpInspectionReport {
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub method: HttpMethod,
    pub path: String,
    pub user_agent: Option<String>,
    pub content_type: Option<String>,
    pub threats: Vec<ThreatType>,
}

#[derive(Debug, PartialEq)]
pub enum ThreatType {
    SqlInjection,
    XssAttempt,
    PathTraversal,
    SuspiciousUserAgent,
    LargePayload,
    SuspiciousHeader,
}

// Threat detection with pattern matching and guards
fn detect_threats(http: &HttpRequest) -> Vec<ThreatType> {
    let mut threats = Vec::new();

    // SQL injection patterns
    let path_lower = http.path.to_lowercase();
    if path_lower.contains("' or ")
        || path_lower.contains("1=1")
        || path_lower.contains("union select")
        || path_lower.contains("drop table")
    {
        threats.push(ThreatType::SqlInjection);
    }

    // XSS patterns
    if path_lower.contains("<script")
        || path_lower.contains("javascript:")
        || path_lower.contains("onerror=")
    {
        threats.push(ThreatType::XssAttempt);
    }

    // Path traversal
    if http.path.contains("../") || http.path.contains("..\\") {
        threats.push(ThreatType::PathTraversal);
    }

    // User-Agent analysis with pattern matching
    if let Some(ua) = http.get_header("User-Agent") {
        match ua {
            // Suspicious tools
            ua if ua.contains("sqlmap")
                || ua.contains("nikto")
                || ua.contains("nmap")
                || ua.contains("masscan") =>
            {
                threats.push(ThreatType::SuspiciousUserAgent);
            }
            _ => {}
        }
    }

    // Large payload
    if http.body.len() > 1_000_000 {
        threats.push(ThreatType::LargePayload);
    }

    // Suspicious headers
    for (name, value) in &http.headers {
        match name.to_lowercase().as_str() {
            "x-forwarded-for" if value.split(',').count() > 10 => {
                threats.push(ThreatType::SuspiciousHeader);
            }
            "referer" if value.len() > 1000 => {
                threats.push(ThreatType::SuspiciousHeader);
            }
            _ => {}
        }
    }

    threats
}

// Pattern matching for HTTP analysis
pub fn classify_http_request(http: &HttpRequest) -> HttpRequestClass {
    match (&http.method, http.path.as_str(), http.get_header("Content-Type")) {
        // API requests
        (HttpMethod::GET, path, _) if path.starts_with("/api/") => {
            HttpRequestClass::ApiGet
        }

        (HttpMethod::POST, path, Some(ct))
            if path.starts_with("/api/") && ct.contains("json") =>
        {
            HttpRequestClass::ApiPost
        }

        // Form submission
        (HttpMethod::POST, _, Some(ct))
            if ct.contains("application/x-www-form-urlencoded") =>
        {
            HttpRequestClass::FormSubmission
        }

        // File upload
        (HttpMethod::POST | HttpMethod::PUT, _, Some(ct))
            if ct.contains("multipart/form-data") =>
        {
            HttpRequestClass::FileUpload
        }

        // Static resources
        (HttpMethod::GET, path, _)
            if path.ends_with(".css")
                || path.ends_with(".js")
                || path.ends_with(".png")
                || path.ends_with(".jpg") =>
        {
            HttpRequestClass::StaticResource
        }

        // Page request
        (HttpMethod::GET, _, _) => HttpRequestClass::PageRequest,

        _ => HttpRequestClass::Other,
    }
}

#[derive(Debug, PartialEq)]
enum HttpRequestClass {
    ApiGet,
    ApiPost,
    FormSubmission,
    FileUpload,
    StaticResource,
    PageRequest,
    Other,
}
```

**If-let chains for validation:**
```rust
// Use if-let chains to validate HTTP requests
fn validate_http_request(packet: &Packet) -> Result<(), ValidationError> {
    // Extract HTTP request using if-let chain
    if let Packet::Ethernet {
        inner: Some(box Packet::IPv4 {
            inner: Some(box Packet::Http(http)),
            ..
        }),
        ..
    } = packet
    {
        // Validate method
        if !matches!(http.method, HttpMethod::GET | HttpMethod::POST | HttpMethod::PUT | HttpMethod::DELETE) {
            return Err(ValidationError::InvalidMethod);
        }

        // Validate path
        if http.path.is_empty() || !http.path.starts_with('/') {
            return Err(ValidationError::InvalidPath);
        }

        // Validate headers
        if http.get_header("Host").is_none() {
            return Err(ValidationError::MissingHostHeader);
        }

        Ok(())
    } else {
        Err(ValidationError::NotHttpPacket)
    }
}

#[derive(Debug)]
enum ValidationError {
    InvalidMethod,
    InvalidPath,
    MissingHostHeader,
    NotHttpPacket,
}
```

**Check/Test:**
- Test HTTP parsing from TCP payload
- Test threat detection identifies SQL injection, XSS
- Test deep packet inspection through all layers
- Test HTTP request classification
- Test if-let chains for validation
- Verify pattern guards work correctly

**Why this isn't enough:**
We inspect individual packets but don't track connections or sessions. Real packet analyzers maintain state—tracking TCP handshakes, HTTP sessions, request/response pairs. Performance is also not optimal for high-throughput scenarios. Let's add connection tracking and optimize with memoization and parallel processing.

---

### Step 6: Connection Tracking, Statistics, and Performance Optimization

**Goal:** Add stateful packet inspection, connection tracking, statistics, and performance optimizations.

**What to improve:**
```rust
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

// Connection tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionKey {
    pub src_ip: Ipv4Address,
    pub dst_ip: Ipv4Address,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: IpProtocol,
}

impl ConnectionKey {
    fn from_packet(packet: &Packet) -> Option<Self> {
        let info = PacketInfo::extract(packet)?;

        Some(ConnectionKey {
            src_ip: info.src_ip?,
            dst_ip: info.dst_ip?,
            src_port: info.src_port?,
            dst_port: info.dst_port?,
            protocol: info.protocol?,
        })
    }

    // Bidirectional key (for tracking both directions)
    fn canonical(&self) -> Self {
        if self.src_ip.0 < self.dst_ip.0
            || (self.src_ip == self.dst_ip && self.src_port < self.dst_port)
        {
            *self
        } else {
            ConnectionKey {
                src_ip: self.dst_ip,
                dst_ip: self.src_ip,
                src_port: self.dst_port,
                dst_port: self.src_port,
                protocol: self.protocol,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub key: ConnectionKey,
    pub state: ConnectionState,
    pub packets: usize,
    pub bytes: usize,
    pub start_time: Instant,
    pub last_seen: Instant,
    pub http_requests: Vec<HttpRequest>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    TcpSynSent,
    TcpSynReceived,
    TcpEstablished,
    TcpFinWait,
    TcpClosed,
    UdpActive,
    Unknown,
}

pub struct PacketAnalyzer {
    connections: HashMap<ConnectionKey, Connection>,
    statistics: Statistics,
}

impl PacketAnalyzer {
    pub fn new() -> Self {
        PacketAnalyzer {
            connections: HashMap::new(),
            statistics: Statistics::default(),
        }
    }

    pub fn process_packet(&mut self, packet: &Packet) {
        self.statistics.total_packets += 1;

        // Update statistics based on packet type
        self.update_statistics(packet);

        // Track connection if applicable
        if let Some(key) = ConnectionKey::from_packet(packet) {
            self.track_connection(key.canonical(), packet);
        }
    }

    fn track_connection(&mut self, key: ConnectionKey, packet: &Packet) {
        let conn = self.connections.entry(key).or_insert_with(|| Connection {
            key,
            state: ConnectionState::Unknown,
            packets: 0,
            bytes: 0,
            start_time: Instant::now(),
            last_seen: Instant::now(),
            http_requests: Vec::new(),
        });

        conn.packets += 1;
        conn.last_seen = Instant::now();

        // Update connection state using pattern matching
        self.update_connection_state(conn, packet);

        // Extract HTTP if present
        if let Some(http) = self.extract_http(packet) {
            conn.http_requests.push(http);
        }
    }

    fn update_connection_state(&mut self, conn: &mut Connection, packet: &Packet) {
        match (packet, &conn.state) {
            // TCP state machine
            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { syn: true, ack: false, .. },
                    ..
                }),
                ConnectionState::Unknown,
            ) => {
                conn.state = ConnectionState::TcpSynSent;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { syn: true, ack: true, .. },
                    ..
                }),
                ConnectionState::TcpSynSent,
            ) => {
                conn.state = ConnectionState::TcpSynReceived;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { ack: true, syn: false, fin: false, .. },
                    ..
                }),
                ConnectionState::TcpSynReceived,
            ) => {
                conn.state = ConnectionState::TcpEstablished;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { fin: true, .. },
                    ..
                }),
                ConnectionState::TcpEstablished,
            ) => {
                conn.state = ConnectionState::TcpFinWait;
            }

            (
                Packet::TCP(TcpPacket {
                    flags: TcpFlags { rst: true, .. },
                    ..
                }),
                _,
            ) => {
                conn.state = ConnectionState::TcpClosed;
            }

            // UDP connections
            (Packet::UDP(_), _) => {
                conn.state = ConnectionState::UdpActive;
            }

            _ => {}
        }
    }

    fn extract_http(&self, packet: &Packet) -> Option<HttpRequest> {
        match packet {
            Packet::Ethernet {
                inner: Some(box Packet::IPv4 {
                    inner: Some(box Packet::Http(http)),
                    ..
                }),
                ..
            } => Some(http.clone()),

            _ => None,
        }
    }

    fn update_statistics(&mut self, packet: &Packet) {
        match packet {
            Packet::Ethernet { .. } => {
                self.statistics.ethernet_packets += 1;
            }

            Packet::IPv4 { packet, .. } => {
                self.statistics.ipv4_packets += 1;

                match packet.protocol {
                    IpProtocol::TCP => self.statistics.tcp_packets += 1,
                    IpProtocol::UDP => self.statistics.udp_packets += 1,
                    IpProtocol::ICMP => self.statistics.icmp_packets += 1,
                    _ => {}
                }
            }

            Packet::TCP(_) => self.statistics.tcp_packets += 1,
            Packet::UDP(_) => self.statistics.udp_packets += 1,
            Packet::Http(_) => self.statistics.http_requests += 1,
            Packet::Raw(_) => self.statistics.raw_packets += 1,
        }
    }

    pub fn get_active_connections(&self) -> Vec<&Connection> {
        let now = Instant::now();

        self.connections
            .values()
            .filter(|conn| {
                // Active if seen in last 60 seconds
                now.duration_since(conn.last_seen) < Duration::from_secs(60)
                    && !matches!(conn.state, ConnectionState::TcpClosed)
            })
            .collect()
    }

    pub fn cleanup_old_connections(&mut self, max_age: Duration) {
        let now = Instant::now();

        self.connections.retain(|_, conn| {
            now.duration_since(conn.last_seen) < max_age
                || !matches!(conn.state, ConnectionState::TcpClosed)
        });
    }
}

#[derive(Debug, Default)]
pub struct Statistics {
    pub total_packets: usize,
    pub ethernet_packets: usize,
    pub ipv4_packets: usize,
    pub tcp_packets: usize,
    pub udp_packets: usize,
    pub icmp_packets: usize,
    pub http_requests: usize,
    pub raw_packets: usize,
}

impl Statistics {
    pub fn report(&self) -> String {
        format!(
            "Total: {}, Ethernet: {}, IPv4: {}, TCP: {}, UDP: {}, HTTP: {}",
            self.total_packets,
            self.ethernet_packets,
            self.ipv4_packets,
            self.tcp_packets,
            self.udp_packets,
            self.http_requests
        )
    }
}
```

**Advanced pattern matching for analysis:**
```rust
// Exhaustive pattern matching for connection analysis
pub fn analyze_connection(conn: &Connection) -> ConnectionAnalysis {
    match (&conn.state, conn.packets, conn.http_requests.len()) {
        // Suspicious: many packets but no established connection
        (ConnectionState::TcpSynSent, p, _) if p > 100 => {
            ConnectionAnalysis::SynFlood
        }

        // Port scan: SYN sent but never established
        (ConnectionState::TcpSynSent, p, _) if p < 5 => {
            ConnectionAnalysis::PossiblePortScan
        }

        // Normal HTTP connection
        (ConnectionState::TcpEstablished, _, http_count) if http_count > 0 => {
            ConnectionAnalysis::NormalHttp
        }

        // Long-lived connection with many packets
        (ConnectionState::TcpEstablished, p, _) if p > 10000 => {
            ConnectionAnalysis::LongLived
        }

        // UDP without response
        (ConnectionState::UdpActive, p, _) if p < 3 => {
            ConnectionAnalysis::UdpQuery
        }

        // Closed properly
        (ConnectionState::TcpClosed, _, _) => {
            ConnectionAnalysis::Closed
        }

        _ => ConnectionAnalysis::Normal,
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectionAnalysis {
    Normal,
    NormalHttp,
    LongLived,
    SynFlood,
    PossiblePortScan,
    UdpQuery,
    Closed,
}

// While-let for processing packet stream
pub fn process_packet_stream<I>(analyzer: &mut PacketAnalyzer, mut packets: I)
where
    I: Iterator<Item = Packet>,
{
    while let Some(packet) = packets.next() {
        analyzer.process_packet(&packet);

        // Periodic cleanup
        if analyzer.statistics.total_packets % 1000 == 0 {
            analyzer.cleanup_old_connections(Duration::from_secs(300));
        }
    }
}

// Let-else for extracting connection info
pub fn get_connection_duration(conn: &Connection) -> Result<Duration, String> {
    let Some(duration) = conn.last_seen.checked_duration_since(conn.start_time) else {
        return Err("Invalid time range".into());
    };

    Ok(duration)
}
```

**Complete example:**
```rust
fn main() {
    let mut analyzer = PacketAnalyzer::new();
    let mut firewall = Firewall::new(Action::Allow);

    // Add firewall rules
    firewall.add_rule(FirewallRule::DenyPort { port: 23 }); // Block telnet
    firewall.add_rule(FirewallRule::AllowPortRange {
        start: 80,
        end: 443,
    }); // Allow HTTP/HTTPS

    // Process packets
    let packets = vec![/* ... packet data ... */];

    for packet_data in packets {
        let Ok(packet) = Packet::parse_full(&packet_data) else {
            continue;
        };

        // Check firewall
        let action = firewall.check_packet(&packet);

        match action {
            Action::Allow | Action::LogAndAllow => {
                analyzer.process_packet(&packet);
            }
            Action::Deny | Action::LogAndDeny => {
                eprintln!("Blocked packet: {:?}", packet);
                continue;
            }
            Action::Log => {
                println!("Logged packet: {:?}", packet);
            }
        }

        // Inspect HTTP if present
        if let Some(report) = inspect_http_request(&packet) {
            if !report.threats.is_empty() {
                eprintln!("Threats detected: {:?}", report.threats);
            }
        }
    }

    // Print statistics
    println!("{}", analyzer.statistics.report());

    // Analyze connections
    for conn in analyzer.get_active_connections() {
        let analysis = analyze_connection(conn);
        if !matches!(analysis, ConnectionAnalysis::Normal | ConnectionAnalysis::NormalHttp) {
            println!("Suspicious connection: {:?} - {:?}", conn.key, analysis);
        }
    }
}
```

**Check/Test:**
- Test connection tracking maintains state correctly
- Test TCP state machine transitions
- Test connection cleanup removes old entries
- Test statistics accumulate correctly
- Test pattern matching on connection state
- Test while-let processes packet streams
- Benchmark: throughput with connection tracking

**What this achieves:**
A complete network packet inspector demonstrating:
- **Exhaustive Pattern Matching**: All packet types and states handled
- **Deep Destructuring**: Extract data through protocol layers
- **Range Patterns**: Port and IP filtering
- **Guards**: Complex firewall rules
- **If-Let Chains**: HTTP validation
- **While-Let**: Stream processing
- **Let-Else**: Error handling
- **Matches! Macro**: Quick checks
- **Enum-Driven Architecture**: Protocol representation

**Extensions to explore:**
- IPv6 support
- More protocols (DNS, DHCP, SMTP, FTP)
- PCAP file format reading/writing
- TLS/SSL inspection
- Regex-based payload matching
- Distributed packet capture
- Real-time visualization
- Machine learning for anomaly detection

---

## Project 3: Business Rule Engine with Enum-Driven Architecture

### Problem Statement

Build a business rule engine that:
- Represents business rules as enums and pattern matching
- Evaluates complex conditional logic using exhaustive matching
- Supports rule composition and chaining
- Implements rule priorities and conflict resolution
- Provides rule validation and analysis
- Demonstrates all pattern matching features (guards, ranges, destructuring, if-let chains)
- Handles temporal rules (time-based, date-based conditions)
- Supports dynamic rule loading and hot-reload

The engine must showcase enum-driven design where business logic is encoded in types and pattern matching ensures correctness.

### Why It Matters

Business rule engines are critical for:
- **E-commerce**: Pricing, discounts, promotions, shipping rules
- **Finance**: Credit scoring, fraud detection, compliance
- **Insurance**: Policy underwriting, claims processing
- **Healthcare**: Treatment protocols, billing rules
- **Workflow**: Approval processes, routing, escalation

Pattern matching excels for rule engines because:
- Rules map naturally to enum variants
- Exhaustiveness ensures all cases handled
- Guards enable complex conditions
- Refactoring rules is safer (compiler catches missing cases)
- Rule composition through nested enums

### Use Cases

1. **E-commerce Promotions**: "Buy 2 get 1 free", "10% off orders over $100"
2. **Credit Approval**: Multi-factor decision trees for loan approval
3. **Insurance Pricing**: Risk-based premium calculation
4. **Fraud Detection**: Score transactions based on patterns
5. **Workflow Routing**: Route tasks based on properties
6. **Tax Calculation**: Complex tax rules with jurisdictions
7. **Access Control**: Role-based permission systems

### Solution Outline

**Core Rule Types:**
```rust
#[derive(Debug, Clone)]
pub enum Rule {
    // Simple conditions
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    In { field: String, values: Vec<Value> },
    Between { field: String, min: Value, max: Value },

    // String matching
    StartsWith { field: String, prefix: String },
    EndsWith { field: String, suffix: String },
    Contains { field: String, substring: String },
    Matches { field: String, pattern: String },

    // Logical operations
    And(Vec<Rule>),
    Or(Vec<Rule>),
    Not(Box<Rule>),

    // Temporal rules
    TimeRange { start: Time, end: Time },
    DateRange { start: Date, end: Date },
    DayOfWeek(Vec<DayOfWeek>),

    // Complex rules
    Custom { name: String, evaluator: fn(&Context) -> bool },
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
}

#[derive(Debug, Clone)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<(Rule, Action)>,
    pub default_action: Action,
}

#[derive(Debug, Clone)]
pub enum Action {
    ApplyDiscount(f64),
    SetPrice(f64),
    Approve,
    Reject { reason: String },
    Flag { severity: Severity },
    Route { destination: String },
    Multiple(Vec<Action>),
}
```

**Pattern Matching for Evaluation:**
```rust
impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            // Simple comparisons with pattern guards
            Rule::Equals { field, value } => {
                match (context.get(field), value) {
                    (Some(ctx_val), expected) if ctx_val == expected => true,
                    _ => false,
                }
            }

            // Range patterns
            Rule::Between { field, min, max } => {
                match context.get(field) {
                    Some(Value::Int(n)) if matches!(min, Value::Int(min_val))
                        && matches!(max, Value::Int(max_val))
                        && (*min_val..=*max_val).contains(n) => true,
                    _ => false,
                }
            }

            // Logical operations with exhaustive matching
            Rule::And(rules) => rules.iter().all(|r| r.evaluate(context)),
            Rule::Or(rules) => rules.iter().any(|r| r.evaluate(context)),
            Rule::Not(rule) => !rule.evaluate(context),

            // ... other cases
        }
    }
}
```

**Testing Hints:**
```rust
#[test]
fn test_discount_rule() {
    let rule = Rule::And(vec![
        Rule::GreaterThan {
            field: "total".into(),
            value: Value::Float(100.0)
        },
        Rule::Equals {
            field: "customer_tier".into(),
            value: Value::String("gold".into())
        },
    ]);

    let mut context = Context::new();
    context.set("total", Value::Float(150.0));
    context.set("customer_tier", Value::String("gold".into()));

    assert!(rule.evaluate(&context));
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Rule Types and Simple Evaluation

**Goal:** Implement fundamental rule types with basic pattern matching.

**What to implement:**
```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl Value {
    fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Rule {
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    And(Vec<Rule>),
    Or(Vec<Rule>),
}

#[derive(Debug)]
pub struct Context {
    values: HashMap<String, Value>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            values: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.values.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            Rule::Equals { field, value } => {
                match context.get(field) {
                    Some(ctx_value) if ctx_value == value => true,
                    _ => false,
                }
            }

            Rule::GreaterThan { field, value } => {
                match (context.get(field), value) {
                    // Integer comparison
                    (Some(Value::Int(ctx)), Value::Int(expected)) => ctx > expected,

                    // Float comparison
                    (Some(ctx_val), expected_val) => {
                        match (ctx_val.as_float(), expected_val.as_float()) {
                            (Some(ctx_f), Some(exp_f)) => ctx_f > exp_f,
                            _ => false,
                        }
                    }
                }
            }

            Rule::LessThan { field, value } => {
                match (context.get(field), value) {
                    (Some(Value::Int(ctx)), Value::Int(expected)) => ctx < expected,

                    (Some(ctx_val), expected_val) => {
                        match (ctx_val.as_float(), expected_val.as_float()) {
                            (Some(ctx_f), Some(exp_f)) => ctx_f < exp_f,
                            _ => false,
                        }
                    }
                }
            }

            Rule::And(rules) => {
                rules.iter().all(|rule| rule.evaluate(context))
            }

            Rule::Or(rules) => {
                rules.iter().any(|rule| rule.evaluate(context))
            }
        }
    }
}

// Simple action system
#[derive(Debug, Clone)]
pub enum Action {
    Approve,
    Reject,
    ApplyDiscount(f64),
}

#[derive(Debug)]
pub struct RuleEngine {
    rules: Vec<(Rule, Action)>,
    default_action: Action,
}

impl RuleEngine {
    pub fn new(default_action: Action) -> Self {
        RuleEngine {
            rules: Vec::new(),
            default_action,
        }
    }

    pub fn add_rule(&mut self, rule: Rule, action: Action) {
        self.rules.push((rule, action));
    }

    pub fn evaluate(&self, context: &Context) -> Action {
        for (rule, action) in &self.rules {
            if rule.evaluate(context) {
                return action.clone();
            }
        }

        self.default_action.clone()
    }
}
```

**Check/Test:**
- Test simple equality checks
- Test numeric comparisons (greater than, less than)
- Test AND/OR logic
- Test rule evaluation order
- Test missing fields return false

**Why this isn't enough:**
Only supports basic comparisons. Real business rules need range checks, string matching, temporal logic, priorities, and complex nested conditions. The pattern matching is straightforward—we're not showcasing guards, range patterns, or deep destructuring. We need more rule types and sophisticated matching.

---

### Step 2: Add Range Patterns, String Matching, and Negation

**Goal:** Expand rule types and demonstrate range patterns and guards.

**What to improve:**
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
}

#[derive(Debug, Clone)]
pub enum Rule {
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    Between { field: String, min: Value, max: Value },
    In { field: String, values: Vec<Value> },

    // String matching
    StartsWith { field: String, prefix: String },
    EndsWith { field: String, suffix: String },
    Contains { field: String, substring: String },

    // Logical
    And(Vec<Rule>),
    Or(Vec<Rule>),
    Not(Box<Rule>),
}

impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            // Existing cases...

            // Range patterns with guards
            Rule::Between { field, min, max } => {
                let Some(value) = context.get(field) else {
                    return false;
                };

                match (value, min, max) {
                    // Integer ranges
                    (Value::Int(n), Value::Int(min_val), Value::Int(max_val))
                        if (*min_val..=*max_val).contains(n) => true,

                    // Float ranges
                    (Value::Float(n), Value::Float(min_val), Value::Float(max_val))
                        if n >= min_val && n <= max_val => true,

                    // Mixed numeric types
                    (val, min_val, max_val) => {
                        match (val.as_float(), min_val.as_float(), max_val.as_float()) {
                            (Some(n), Some(min_f), Some(max_f)) if n >= min_f && n <= max_f => true,
                            _ => false,
                        }
                    }
                }
            }

            // In-list checking with or-patterns
            Rule::In { field, values } => {
                match context.get(field) {
                    Some(ctx_value) => values.contains(ctx_value),
                    None => false,
                }
            }

            // String matching patterns
            Rule::StartsWith { field, prefix } => {
                match context.get(field) {
                    Some(Value::String(s)) if s.starts_with(prefix) => true,
                    _ => false,
                }
            }

            Rule::EndsWith { field, suffix } => {
                match context.get(field) {
                    Some(Value::String(s)) if s.ends_with(suffix) => true,
                    _ => false,
                }
            }

            Rule::Contains { field, substring } => {
                match context.get(field) {
                    Some(Value::String(s)) if s.contains(substring) => true,
                    _ => false,
                }
            }

            // Negation
            Rule::Not(rule) => !rule.evaluate(context),

            // Logical operations (existing)
            Rule::And(rules) => rules.iter().all(|r| r.evaluate(context)),
            Rule::Or(rules) => rules.iter().any(|r| r.evaluate(context)),

            _ => false,
        }
    }
}

// Rule builder for convenience
impl Rule {
    pub fn field_equals(field: &str, value: Value) -> Self {
        Rule::Equals {
            field: field.into(),
            value,
        }
    }

    pub fn field_between(field: &str, min: Value, max: Value) -> Self {
        Rule::Between {
            field: field.into(),
            min,
            max,
        }
    }

    pub fn field_in(field: &str, values: Vec<Value>) -> Self {
        Rule::In {
            field: field.into(),
            values,
        }
    }

    pub fn and(rules: Vec<Rule>) -> Self {
        Rule::And(rules)
    }

    pub fn or(rules: Vec<Rule>) -> Self {
        Rule::Or(rules)
    }

    pub fn not(rule: Rule) -> Self {
        Rule::Not(Box::new(rule))
    }
}
```

**Pattern matching for rule analysis:**
```rust
// Analyze rule complexity
pub fn count_conditions(rule: &Rule) -> usize {
    match rule {
        // Leaf conditions
        Rule::Equals { .. }
        | Rule::GreaterThan { .. }
        | Rule::LessThan { .. }
        | Rule::Between { .. }
        | Rule::In { .. }
        | Rule::StartsWith { .. }
        | Rule::EndsWith { .. }
        | Rule::Contains { .. } => 1,

        // Compound conditions
        Rule::And(rules) | Rule::Or(rules) => {
            rules.iter().map(count_conditions).sum()
        }

        Rule::Not(rule) => count_conditions(rule),
    }
}

// Extract field dependencies
pub fn get_required_fields(rule: &Rule) -> Vec<String> {
    match rule {
        Rule::Equals { field, .. }
        | Rule::GreaterThan { field, .. }
        | Rule::LessThan { field, .. }
        | Rule::Between { field, .. }
        | Rule::In { field, .. }
        | Rule::StartsWith { field, .. }
        | Rule::EndsWith { field, .. }
        | Rule::Contains { field, .. } => vec![field.clone()],

        Rule::And(rules) | Rule::Or(rules) => {
            rules.iter()
                .flat_map(get_required_fields)
                .collect()
        }

        Rule::Not(rule) => get_required_fields(rule),
    }
}
```

**Check/Test:**
- Test range checking with Between
- Test In with multiple values
- Test string prefix/suffix/contains
- Test NOT negation
- Test nested AND/OR combinations
- Test rule analysis functions

**Why this isn't enough:**
Rules work but no priority system. What if multiple rules match? Real engines need priority, conflict resolution, and rule ordering. We also don't have temporal rules (date/time-based), custom predicates, or rule validation. The actions are too simple—no parameterized actions or action chaining.

---

### Step 3: Add Rule Priorities, Actions, and Conflict Resolution

**Goal:** Implement rule priorities and rich action types with pattern matching.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum Action {
    Approve,
    Reject { reason: String },
    ApplyDiscount { percent: f64 },
    SetPrice { amount: f64 },
    AddBonus { points: i64 },
    Flag { severity: Severity, message: String },
    Route { destination: String },
    Log { level: LogLevel, message: String },
    Multiple(Vec<Action>),
    Conditional { condition: Rule, then_action: Box<Action>, else_action: Box<Action> },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct RuleWithPriority {
    pub rule: Rule,
    pub action: Action,
    pub priority: i32,  // Higher = more important
    pub name: String,
}

#[derive(Debug)]
pub struct RuleEngine {
    rules: Vec<RuleWithPriority>,
    default_action: Action,
    conflict_strategy: ConflictStrategy,
}

#[derive(Debug, Clone)]
pub enum ConflictStrategy {
    FirstMatch,      // Return first matching rule
    HighestPriority, // Return highest priority match
    AllMatches,      // Execute all matching rules
    MostSpecific,    // Return most specific (most conditions)
}

impl RuleEngine {
    pub fn new(default_action: Action, strategy: ConflictStrategy) -> Self {
        RuleEngine {
            rules: Vec::new(),
            default_action,
            conflict_strategy: strategy,
        }
    }

    pub fn add_rule(&mut self, rule: Rule, action: Action, priority: i32, name: String) {
        self.rules.push(RuleWithPriority {
            rule,
            action,
            priority,
            name,
        });

        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn evaluate(&self, context: &Context) -> EvaluationResult {
        let matches: Vec<&RuleWithPriority> = self.rules
            .iter()
            .filter(|r| r.rule.evaluate(context))
            .collect();

        if matches.is_empty() {
            return EvaluationResult {
                action: self.default_action.clone(),
                matched_rules: vec![],
            };
        }

        // Apply conflict resolution strategy
        let selected_rules = match self.conflict_strategy {
            ConflictStrategy::FirstMatch => {
                vec![matches[0]]
            }

            ConflictStrategy::HighestPriority => {
                vec![matches[0]]  // Already sorted by priority
            }

            ConflictStrategy::AllMatches => {
                matches
            }

            ConflictStrategy::MostSpecific => {
                // Find rule with most conditions
                let max_conditions = matches
                    .iter()
                    .map(|r| count_conditions(&r.rule))
                    .max()
                    .unwrap_or(0);

                matches
                    .into_iter()
                    .filter(|r| count_conditions(&r.rule) == max_conditions)
                    .take(1)
                    .collect()
            }
        };

        // Combine actions
        let action = self.combine_actions(
            selected_rules.iter().map(|r| &r.action).collect()
        );

        EvaluationResult {
            action,
            matched_rules: selected_rules.iter().map(|r| r.name.clone()).collect(),
        }
    }

    fn combine_actions(&self, actions: Vec<&Action>) -> Action {
        if actions.is_empty() {
            return self.default_action.clone();
        }

        if actions.len() == 1 {
            return actions[0].clone();
        }

        Action::Multiple(actions.iter().map(|a| (*a).clone()).collect())
    }
}

#[derive(Debug)]
pub struct EvaluationResult {
    pub action: Action,
    pub matched_rules: Vec<String>,
}

// Execute actions with pattern matching
pub fn execute_action(action: &Action, context: &mut Context) -> ActionResult {
    match action {
        Action::Approve => {
            context.set("status", Value::String("approved".into()));
            ActionResult::Success
        }

        Action::Reject { reason } => {
            context.set("status", Value::String("rejected".into()));
            context.set("reason", Value::String(reason.clone()));
            ActionResult::Success
        }

        Action::ApplyDiscount { percent } if *percent > 0.0 && *percent <= 100.0 => {
            if let Some(Value::Float(price)) = context.get("price") {
                let discounted = price * (1.0 - percent / 100.0);
                context.set("final_price", Value::Float(discounted));
                ActionResult::Success
            } else {
                ActionResult::Error("Price field not found".into())
            }
        }

        Action::SetPrice { amount } if *amount >= 0.0 => {
            context.set("final_price", Value::Float(*amount));
            ActionResult::Success
        }

        Action::Flag { severity, message } => {
            context.set("flagged", Value::Bool(true));
            context.set("flag_severity", Value::String(format!("{:?}", severity)));
            context.set("flag_message", Value::String(message.clone()));
            ActionResult::Success
        }

        Action::Multiple(actions) => {
            for action in actions {
                if let ActionResult::Error(e) = execute_action(action, context) {
                    return ActionResult::Error(e);
                }
            }
            ActionResult::Success
        }

        Action::Conditional { condition, then_action, else_action } => {
            if condition.evaluate(context) {
                execute_action(then_action, context)
            } else {
                execute_action(else_action, context)
            }
        }

        _ => ActionResult::Error("Invalid action parameters".into()),
    }
}

#[derive(Debug)]
pub enum ActionResult {
    Success,
    Error(String),
}
```

**Pattern matching for action analysis:**
```rust
// Classify actions by type
pub fn classify_action(action: &Action) -> ActionType {
    match action {
        Action::Approve | Action::Reject { .. } => ActionType::Decision,

        Action::ApplyDiscount { .. } | Action::SetPrice { .. } => ActionType::Pricing,

        Action::Flag { severity, .. } => match severity {
            Severity::Critical | Severity::High => ActionType::Alert,
            _ => ActionType::Warning,
        },

        Action::Route { .. } => ActionType::Routing,

        Action::Log { level, .. } => match level {
            LogLevel::Error => ActionType::Alert,
            _ => ActionType::Informational,
        },

        Action::Multiple(actions) => {
            // Classify by highest severity action
            actions
                .iter()
                .map(classify_action)
                .max_by_key(|t| action_type_priority(t))
                .unwrap_or(ActionType::Other)
        }

        _ => ActionType::Other,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ActionType {
    Decision,
    Pricing,
    Alert,
    Warning,
    Routing,
    Informational,
    Other,
}

fn action_type_priority(action_type: &ActionType) -> u8 {
    match action_type {
        ActionType::Decision => 5,
        ActionType::Alert => 4,
        ActionType::Pricing => 3,
        ActionType::Warning => 2,
        ActionType::Routing => 2,
        ActionType::Informational => 1,
        ActionType::Other => 0,
    }
}
```

**Check/Test:**
- Test priority ordering
- Test conflict resolution strategies
- Test action execution
- Test multiple action combination
- Test conditional actions
- Test action classification

**Why this isn't enough:**
Rules and actions work but no temporal logic. Business rules often depend on time/date (weekends, holidays, time ranges). We also don't validate rules before execution or provide debugging/explanation. Real engines need rule validation, temporal support, and explainability.

---

### Step 4: Add Temporal Rules and Time-Based Pattern Matching

**Goal:** Implement time and date-based rules with pattern matching for temporal logic.

**What to improve:**
```rust
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq)]
pub struct DateTime {
    pub year: u16,
    pub month: u8,   // 1-12
    pub day: u8,     // 1-31
    pub hour: u8,    // 0-23
    pub minute: u8,  // 0-59
}

impl DateTime {
    pub fn now() -> Self {
        // Simplified; use chrono in production
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        let secs = duration.as_secs();

        // Basic conversion (simplified)
        let days = secs / 86400;
        let remaining = secs % 86400;
        let hour = (remaining / 3600) as u8;
        let minute = ((remaining % 3600) / 60) as u8;

        // Simplified date calculation
        DateTime {
            year: 2024,
            month: 1,
            day: 1,
            hour,
            minute,
        }
    }

    pub fn day_of_week(&self) -> DayOfWeek {
        // Simplified Zeller's congruence
        let mut m = self.month as i32;
        let mut y = self.year as i32;

        if m < 3 {
            m += 12;
            y -= 1;
        }

        let k = y % 100;
        let j = y / 100;
        let h = (self.day as i32 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 - 2 * j) % 7;

        match (h + 6) % 7 {
            0 => DayOfWeek::Monday,
            1 => DayOfWeek::Tuesday,
            2 => DayOfWeek::Wednesday,
            3 => DayOfWeek::Thursday,
            4 => DayOfWeek::Friday,
            5 => DayOfWeek::Saturday,
            _ => DayOfWeek::Sunday,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl DayOfWeek {
    pub fn is_weekend(&self) -> bool {
        matches!(self, DayOfWeek::Saturday | DayOfWeek::Sunday)
    }

    pub fn is_weekday(&self) -> bool {
        !self.is_weekend()
    }
}

#[derive(Debug, Clone)]
pub enum Rule {
    // Previous variants...
    Equals { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    LessThan { field: String, value: Value },
    Between { field: String, min: Value, max: Value },
    In { field: String, values: Vec<Value> },
    StartsWith { field: String, prefix: String },
    EndsWith { field: String, suffix: String },
    Contains { field: String, substring: String },
    And(Vec<Rule>),
    Or(Vec<Rule>),
    Not(Box<Rule>),

    // Temporal rules
    TimeRange { start_hour: u8, end_hour: u8 },
    DateRange { start: DateTime, end: DateTime },
    DayOfWeekRule(Vec<DayOfWeek>),
    IsWeekend,
    IsWeekday,
    HourBetween { start: u8, end: u8 },
    AfterDate { date: DateTime },
    BeforeDate { date: DateTime },
}

impl Rule {
    pub fn evaluate(&self, context: &Context) -> bool {
        match self {
            // Previous cases...

            // Temporal pattern matching
            Rule::TimeRange { start_hour, end_hour } => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => {
                        let hour = dt.hour;
                        if start_hour <= end_hour {
                            hour >= *start_hour && hour < *end_hour
                        } else {
                            // Wraps around midnight
                            hour >= *start_hour || hour < *end_hour
                        }
                    }
                    _ => false,
                }
            }

            Rule::DateRange { start, end } => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => {
                        dt >= start && dt <= end
                    }
                    _ => false,
                }
            }

            Rule::DayOfWeekRule(days) => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => {
                        let day = dt.day_of_week();
                        days.contains(&day)
                    }
                    _ => false,
                }
            }

            Rule::IsWeekend => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => {
                        dt.day_of_week().is_weekend()
                    }
                    _ => false,
                }
            }

            Rule::IsWeekday => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => {
                        dt.day_of_week().is_weekday()
                    }
                    _ => false,
                }
            }

            Rule::HourBetween { start, end } => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => {
                        matches!(dt.hour, h if h >= *start && h < *end)
                    }
                    _ => false,
                }
            }

            Rule::AfterDate { date } => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => dt > date,
                    _ => false,
                }
            }

            Rule::BeforeDate { date } => {
                match context.get("current_time") {
                    Some(Value::DateTime(dt)) => dt < date,
                    _ => false,
                }
            }

            // Existing rules...
            _ => false,
        }
    }
}

// Add DateTime variant to Value enum
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    DateTime(DateTime),
}

// Implement ordering for DateTime
impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.year.cmp(&other.year) {
            std::cmp::Ordering::Equal => {}
            ord => return Some(ord),
        }
        match self.month.cmp(&other.month) {
            std::cmp::Ordering::Equal => {}
            ord => return Some(ord),
        }
        match self.day.cmp(&other.day) {
            std::cmp::Ordering::Equal => {}
            ord => return Some(ord),
        }
        match self.hour.cmp(&other.hour) {
            std::cmp::Ordering::Equal => {}
            ord => return Some(ord),
        }
        Some(self.minute.cmp(&other.minute))
    }
}
```

**Example: Business hours and weekend pricing:**
```rust
// Weekend discount rule
let weekend_discount = RuleWithPriority {
    name: "Weekend 20% discount".into(),
    priority: 100,
    rule: Rule::And(vec![
        Rule::IsWeekend,
        Rule::GreaterThan {
            field: "total".into(),
            value: Value::Float(50.0),
        },
    ]),
    action: Action::ApplyDiscount { percent: 20.0 },
};

// Business hours support rule
let business_hours_support = RuleWithPriority {
    name: "Business hours support".into(),
    priority: 80,
    rule: Rule::And(vec![
        Rule::IsWeekday,
        Rule::HourBetween { start: 9, end: 17 },
    ]),
    action: Action::Route {
        destination: "priority_support".into(),
    },
};

// Holiday pricing (Christmas week)
let christmas_pricing = RuleWithPriority {
    name: "Christmas week premium".into(),
    priority: 120,
    rule: Rule::DateRange {
        start: DateTime {
            year: 2024,
            month: 12,
            day: 20,
            hour: 0,
            minute: 0,
        },
        end: DateTime {
            year: 2024,
            month: 12,
            day: 27,
            hour: 23,
            minute: 59,
        },
    },
    action: Action::ApplyDiscount { percent: 15.0 },
};
```

**Pattern matching for temporal analysis:**
```rust
// Check if rule has temporal dependencies
pub fn is_temporal_rule(rule: &Rule) -> bool {
    match rule {
        Rule::TimeRange { .. }
        | Rule::DateRange { .. }
        | Rule::DayOfWeekRule(_)
        | Rule::IsWeekend
        | Rule::IsWeekday
        | Rule::HourBetween { .. }
        | Rule::AfterDate { .. }
        | Rule::BeforeDate { .. } => true,

        Rule::And(rules) | Rule::Or(rules) => {
            rules.iter().any(is_temporal_rule)
        }

        Rule::Not(rule) => is_temporal_rule(rule),

        _ => false,
    }
}

// Get time window when rule is active
pub fn get_active_window(rule: &Rule) -> Option<TimeWindow> {
    match rule {
        Rule::TimeRange { start_hour, end_hour } => {
            Some(TimeWindow::HourRange {
                start: *start_hour,
                end: *end_hour,
            })
        }

        Rule::DateRange { start, end } => {
            Some(TimeWindow::DateRange {
                start: start.clone(),
                end: end.clone(),
            })
        }

        Rule::DayOfWeekRule(days) => {
            Some(TimeWindow::DaysOfWeek(days.clone()))
        }

        _ => None,
    }
}

#[derive(Debug)]
pub enum TimeWindow {
    HourRange { start: u8, end: u8 },
    DateRange { start: DateTime, end: DateTime },
    DaysOfWeek(Vec<DayOfWeek>),
}
```

**Check/Test:**
- Test weekend/weekday detection
- Test time range checking (including midnight wraparound)
- Test date range comparison
- Test day of week matching
- Test temporal rule combinations with business rules
- Test that non-temporal rules still work

**Why this isn't enough:**
Temporal rules work but we can't validate rules before deployment or explain why a rule matched/didn't match. Real engines need to validate that rules make sense (no conflicts, proper field references, valid ranges) and provide explanations for debugging and compliance.

---

### Step 5: Add Rule Validation and Explainability

**Goal:** Implement rule validation and generate explanations for rule matching.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidFieldReference { field: String },
    InvalidValueType { expected: String, got: String },
    InvalidRange { min: Value, max: Value },
    ConflictingRules { rule1: String, rule2: String },
    EmptyCompoundRule { rule_type: String },
    CircularDependency { rule_chain: Vec<String> },
    InvalidTimeRange { start: u8, end: u8 },
}

// Rule validator
pub struct RuleValidator {
    valid_fields: Vec<String>,
    field_types: HashMap<String, ValueType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueType {
    Integer,
    Float,
    String,
    Boolean,
    DateTime,
}

impl RuleValidator {
    pub fn new() -> Self {
        RuleValidator {
            valid_fields: Vec::new(),
            field_types: HashMap::new(),
        }
    }

    pub fn register_field(&mut self, name: String, value_type: ValueType) {
        self.valid_fields.push(name.clone());
        self.field_types.insert(name, value_type);
    }

    pub fn validate(&self, rule: &Rule) -> Result<(), ValidationError> {
        match rule {
            // Validate field references
            Rule::Equals { field, value }
            | Rule::GreaterThan { field, value }
            | Rule::LessThan { field, value } => {
                self.validate_field_exists(field)?;
                self.validate_value_type_matches(field, value)?;
                Ok(())
            }

            Rule::Between { field, min, max } => {
                self.validate_field_exists(field)?;
                self.validate_value_type_matches(field, min)?;
                self.validate_value_type_matches(field, max)?;

                // Validate range makes sense
                if !self.is_valid_range(min, max) {
                    return Err(ValidationError::InvalidRange {
                        min: min.clone(),
                        max: max.clone(),
                    });
                }
                Ok(())
            }

            Rule::In { field, values } => {
                self.validate_field_exists(field)?;
                if values.is_empty() {
                    return Err(ValidationError::EmptyCompoundRule {
                        rule_type: "In".into(),
                    });
                }
                for value in values {
                    self.validate_value_type_matches(field, value)?;
                }
                Ok(())
            }

            // String rules
            Rule::StartsWith { field, .. }
            | Rule::EndsWith { field, .. }
            | Rule::Contains { field, .. } => {
                self.validate_field_exists(field)?;
                self.validate_field_type(field, ValueType::String)?;
                Ok(())
            }

            // Temporal rules validation
            Rule::TimeRange { start_hour, end_hour }
            | Rule::HourBetween { start: start_hour, end: end_hour } => {
                if *start_hour >= 24 || *end_hour > 24 {
                    return Err(ValidationError::InvalidTimeRange {
                        start: *start_hour,
                        end: *end_hour,
                    });
                }
                Ok(())
            }

            // Compound rules
            Rule::And(rules) | Rule::Or(rules) => {
                if rules.is_empty() {
                    return Err(ValidationError::EmptyCompoundRule {
                        rule_type: if matches!(self, Rule::And(_)) { "And" } else { "Or" }.into(),
                    });
                }
                for rule in rules {
                    self.validate(rule)?;
                }
                Ok(())
            }

            Rule::Not(rule) => self.validate(rule),

            // Temporal rules always valid
            Rule::IsWeekend | Rule::IsWeekday => Ok(()),

            Rule::DateRange { start, end } => {
                if start > end {
                    return Err(ValidationError::InvalidRange {
                        min: Value::DateTime(start.clone()),
                        max: Value::DateTime(end.clone()),
                    });
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn validate_field_exists(&self, field: &str) -> Result<(), ValidationError> {
        if !self.valid_fields.contains(&field.to_string()) {
            Err(ValidationError::InvalidFieldReference {
                field: field.into(),
            })
        } else {
            Ok(())
        }
    }

    fn validate_field_type(&self, field: &str, expected: ValueType) -> Result<(), ValidationError> {
        match self.field_types.get(field) {
            Some(actual) if *actual == expected => Ok(()),
            Some(actual) => Err(ValidationError::InvalidValueType {
                expected: format!("{:?}", expected),
                got: format!("{:?}", actual),
            }),
            None => Ok(()), // Field type unknown, allow it
        }
    }

    fn validate_value_type_matches(&self, field: &str, value: &Value) -> Result<(), ValidationError> {
        let value_type = match value {
            Value::Int(_) => ValueType::Integer,
            Value::Float(_) => ValueType::Float,
            Value::String(_) => ValueType::String,
            Value::Bool(_) => ValueType::Boolean,
            Value::DateTime(_) => ValueType::DateTime,
            _ => return Ok(()),
        };

        if let Some(expected_type) = self.field_types.get(field) {
            // Allow Int to Float promotion
            if *expected_type == ValueType::Float && value_type == ValueType::Integer {
                return Ok(());
            }

            if *expected_type != value_type {
                return Err(ValidationError::InvalidValueType {
                    expected: format!("{:?}", expected_type),
                    got: format!("{:?}", value_type),
                });
            }
        }

        Ok(())
    }

    fn is_valid_range(&self, min: &Value, max: &Value) -> bool {
        match (min, max) {
            (Value::Int(min_val), Value::Int(max_val)) => min_val <= max_val,
            (Value::Float(min_val), Value::Float(max_val)) => min_val <= max_val,
            (Value::DateTime(min_dt), Value::DateTime(max_dt)) => min_dt <= max_dt,
            _ => true, // Can't compare, assume valid
        }
    }
}

// Explanation system
#[derive(Debug)]
pub struct RuleExplanation {
    pub rule_name: String,
    pub matched: bool,
    pub reason: String,
    pub sub_explanations: Vec<RuleExplanation>,
}

impl Rule {
    pub fn explain(&self, context: &Context, rule_name: &str) -> RuleExplanation {
        let matched = self.evaluate(context);
        let reason = self.generate_explanation(context, matched);

        let sub_explanations = match self {
            Rule::And(rules) | Rule::Or(rules) => {
                rules
                    .iter()
                    .enumerate()
                    .map(|(i, r)| r.explain(context, &format!("{}_sub_{}", rule_name, i)))
                    .collect()
            }
            Rule::Not(rule) => {
                vec![rule.explain(context, &format!("{}_negated", rule_name))]
            }
            _ => vec![],
        };

        RuleExplanation {
            rule_name: rule_name.to_string(),
            matched,
            reason,
            sub_explanations,
        }
    }

    fn generate_explanation(&self, context: &Context, matched: bool) -> String {
        match self {
            Rule::Equals { field, value } => {
                let actual = context.get(field)
                    .map(|v| format!("{:?}", v))
                    .unwrap_or_else(|| "not set".to_string());

                if matched {
                    format!("Field '{}' equals {:?} (actual: {})", field, value, actual)
                } else {
                    format!("Field '{}' does not equal {:?} (actual: {})", field, value, actual)
                }
            }

            Rule::GreaterThan { field, value } => {
                let actual = context.get(field)
                    .map(|v| format!("{:?}", v))
                    .unwrap_or_else(|| "not set".to_string());

                if matched {
                    format!("Field '{}' > {:?} (actual: {})", field, value, actual)
                } else {
                    format!("Field '{}' not > {:?} (actual: {})", field, value, actual)
                }
            }

            Rule::Between { field, min, max } => {
                let actual = context.get(field)
                    .map(|v| format!("{:?}", v))
                    .unwrap_or_else(|| "not set".to_string());

                if matched {
                    format!("Field '{}' between {:?} and {:?} (actual: {})", field, min, max, actual)
                } else {
                    format!("Field '{}' not between {:?} and {:?} (actual: {})", field, min, max, actual)
                }
            }

            Rule::IsWeekend => {
                if matched {
                    "Current day is weekend (Saturday or Sunday)".into()
                } else {
                    "Current day is not weekend".into()
                }
            }

            Rule::TimeRange { start_hour, end_hour } => {
                if matched {
                    format!("Current time is between {}:00 and {}:00", start_hour, end_hour)
                } else {
                    format!("Current time is not between {}:00 and {}:00", start_hour, end_hour)
                }
            }

            Rule::And(rules) => {
                if matched {
                    format!("All {} conditions matched", rules.len())
                } else {
                    format!("Not all {} conditions matched", rules.len())
                }
            }

            Rule::Or(rules) => {
                if matched {
                    format!("At least one of {} conditions matched", rules.len())
                } else {
                    format!("None of {} conditions matched", rules.len())
                }
            }

            Rule::Not(rule) => {
                if matched {
                    "Negated condition is true".into()
                } else {
                    "Negated condition is false".into()
                }
            }

            _ => format!("Rule evaluated to {}", matched),
        }
    }
}

// Pretty print explanation
impl RuleExplanation {
    pub fn print(&self, indent: usize) {
        let prefix = "  ".repeat(indent);
        let status = if self.matched { "✓" } else { "✗" };

        println!("{}{} {} - {}", prefix, status, self.rule_name, self.reason);

        for sub in &self.sub_explanations {
            sub.print(indent + 1);
        }
    }
}
```

**Example: Validation and explanation:**
```rust
// Set up validator
let mut validator = RuleValidator::new();
validator.register_field("total".into(), ValueType::Float);
validator.register_field("customer_tier".into(), ValueType::String);
validator.register_field("order_count".into(), ValueType::Integer);

// Create rule
let rule = Rule::And(vec![
    Rule::GreaterThan {
        field: "total".into(),
        value: Value::Float(100.0),
    },
    Rule::Equals {
        field: "customer_tier".into(),
        value: Value::String("gold".into()),
    },
    Rule::Or(vec![
        Rule::IsWeekend,
        Rule::GreaterThan {
            field: "order_count".into(),
            value: Value::Int(10),
        },
    ]),
]);

// Validate
match validator.validate(&rule) {
    Ok(()) => println!("Rule is valid"),
    Err(e) => println!("Validation error: {:?}", e),
}

// Explain evaluation
let mut context = Context::new();
context.set("total", Value::Float(150.0));
context.set("customer_tier", Value::String("silver".into()));
context.set("order_count", Value::Int(12));

let explanation = rule.explain(&context, "gold_weekend_discount");
explanation.print(0);

// Output:
// ✗ gold_weekend_discount - Not all 3 conditions matched
//   ✓ gold_weekend_discount_sub_0 - Field 'total' > Float(100.0) (actual: Float(150.0))
//   ✗ gold_weekend_discount_sub_1 - Field 'customer_tier' does not equal String("gold") (actual: String("silver"))
//   ✓ gold_weekend_discount_sub_2 - At least one of 2 conditions matched
//     ✗ gold_weekend_discount_sub_2_sub_0 - Current day is not weekend
//     ✓ gold_weekend_discount_sub_2_sub_1 - Field 'order_count' > Int(10) (actual: Int(12))
```

**Check/Test:**
- Test validation catches invalid field references
- Test validation catches type mismatches
- Test validation catches invalid ranges
- Test explanation for matched rules
- Test explanation for failed rules
- Test explanation for compound rules (And/Or/Not)
- Test explanation includes all sub-rules

**Why this isn't enough:**
We can validate and explain rules, but everything is hardcoded. Real engines need to load rules dynamically from configuration files (JSON/YAML), support rule hot-reload, and optimize evaluation for large rule sets.

---

### Step 6: Dynamic Rule Loading and Performance Optimization

**Goal:** Implement dynamic rule loading from configuration and optimize for performance.

**What to improve:**
```rust
use serde::{Deserialize, Serialize};
use std::fs;

// Serializable rule representation
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleConfig {
    Equals {
        field: String,
        value: ValueConfig,
    },
    GreaterThan {
        field: String,
        value: ValueConfig,
    },
    LessThan {
        field: String,
        value: ValueConfig,
    },
    Between {
        field: String,
        min: ValueConfig,
        max: ValueConfig,
    },
    In {
        field: String,
        values: Vec<ValueConfig>,
    },
    StartsWith {
        field: String,
        prefix: String,
    },
    Contains {
        field: String,
        substring: String,
    },
    And {
        rules: Vec<RuleConfig>,
    },
    Or {
        rules: Vec<RuleConfig>,
    },
    Not {
        rule: Box<RuleConfig>,
    },
    TimeRange {
        start_hour: u8,
        end_hour: u8,
    },
    IsWeekend,
    IsWeekday,
    HourBetween {
        start: u8,
        end: u8,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ValueConfig {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

impl From<ValueConfig> for Value {
    fn from(config: ValueConfig) -> Self {
        match config {
            ValueConfig::Int(n) => Value::Int(n),
            ValueConfig::Float(f) => Value::Float(f),
            ValueConfig::String(s) => Value::String(s),
            ValueConfig::Bool(b) => Value::Bool(b),
        }
    }
}

impl RuleConfig {
    pub fn to_rule(&self) -> Rule {
        match self {
            RuleConfig::Equals { field, value } => Rule::Equals {
                field: field.clone(),
                value: value.clone().into(),
            },

            RuleConfig::GreaterThan { field, value } => Rule::GreaterThan {
                field: field.clone(),
                value: value.clone().into(),
            },

            RuleConfig::Between { field, min, max } => Rule::Between {
                field: field.clone(),
                min: min.clone().into(),
                max: max.clone().into(),
            },

            RuleConfig::In { field, values } => Rule::In {
                field: field.clone(),
                values: values.iter().map(|v| v.clone().into()).collect(),
            },

            RuleConfig::And { rules } => Rule::And(
                rules.iter().map(|r| r.to_rule()).collect()
            ),

            RuleConfig::Or { rules } => Rule::Or(
                rules.iter().map(|r| r.to_rule()).collect()
            ),

            RuleConfig::Not { rule } => Rule::Not(
                Box::new(rule.to_rule())
            ),

            RuleConfig::StartsWith { field, prefix } => Rule::StartsWith {
                field: field.clone(),
                prefix: prefix.clone(),
            },

            RuleConfig::IsWeekend => Rule::IsWeekend,
            RuleConfig::IsWeekday => Rule::IsWeekday,

            RuleConfig::TimeRange { start_hour, end_hour } => Rule::TimeRange {
                start_hour: *start_hour,
                end_hour: *end_hour,
            },

            RuleConfig::HourBetween { start, end } => Rule::HourBetween {
                start: *start,
                end: *end,
            },

            _ => panic!("Unimplemented rule type"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleSetConfig {
    pub name: String,
    pub rules: Vec<RuleEntryConfig>,
    pub default_action: ActionConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleEntryConfig {
    pub name: String,
    pub priority: i32,
    pub rule: RuleConfig,
    pub action: ActionConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionConfig {
    Approve,
    Reject { reason: String },
    ApplyDiscount { percent: f64 },
    SetPrice { amount: f64 },
    Flag { severity: String, message: String },
    Route { destination: String },
}

impl ActionConfig {
    pub fn to_action(&self) -> Action {
        match self {
            ActionConfig::Approve => Action::Approve,
            ActionConfig::Reject { reason } => Action::Reject {
                reason: reason.clone(),
            },
            ActionConfig::ApplyDiscount { percent } => Action::ApplyDiscount {
                percent: *percent,
            },
            ActionConfig::SetPrice { amount } => Action::SetPrice {
                amount: *amount,
            },
            ActionConfig::Flag { severity, message } => {
                let sev = match severity.as_str() {
                    "low" => Severity::Low,
                    "medium" => Severity::Medium,
                    "high" => Severity::High,
                    "critical" => Severity::Critical,
                    _ => Severity::Low,
                };
                Action::Flag {
                    severity: sev,
                    message: message.clone(),
                }
            }
            ActionConfig::Route { destination } => Action::Route {
                destination: destination.clone(),
            },
        }
    }
}

// Rule loader
pub struct RuleLoader {
    validator: RuleValidator,
}

impl RuleLoader {
    pub fn new(validator: RuleValidator) -> Self {
        RuleLoader { validator }
    }

    pub fn load_from_file(&self, path: &str) -> Result<RuleEngine, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        self.load_from_json(&content)
    }

    pub fn load_from_json(&self, json: &str) -> Result<RuleEngine, Box<dyn std::error::Error>> {
        let config: RuleSetConfig = serde_json::from_str(json)?;
        self.build_engine(config)
    }

    fn build_engine(&self, config: RuleSetConfig) -> Result<RuleEngine, Box<dyn std::error::Error>> {
        let mut engine = RuleEngine::new(
            config.default_action.to_action(),
            ConflictStrategy::HighestPriority,
        );

        for entry in config.rules {
            let rule = entry.rule.to_rule();

            // Validate rule before adding
            if let Err(e) = self.validator.validate(&rule) {
                return Err(format!("Validation error in rule '{}': {:?}", entry.name, e).into());
            }

            engine.add_rule(
                rule,
                entry.action.to_action(),
                entry.priority,
                entry.name,
            );
        }

        Ok(engine)
    }
}

// Performance: Rule caching
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct CachedRuleEngine {
    engine: RuleEngine,
    cache: Arc<Mutex<HashMap<String, EvaluationResult>>>,
    cache_enabled: bool,
}

impl CachedRuleEngine {
    pub fn new(engine: RuleEngine) -> Self {
        CachedRuleEngine {
            engine,
            cache: Arc::new(Mutex::new(HashMap::new())),
            cache_enabled: true,
        }
    }

    pub fn evaluate(&self, context: &Context) -> EvaluationResult {
        if !self.cache_enabled {
            return self.engine.evaluate(context);
        }

        // Create cache key from context
        let cache_key = self.context_to_key(context);

        // Check cache
        {
            let cache = self.cache.lock().unwrap();
            if let Some(result) = cache.get(&cache_key) {
                return result.clone();
            }
        }

        // Evaluate and cache
        let result = self.engine.evaluate(context);

        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(cache_key, result.clone());
        }

        result
    }

    fn context_to_key(&self, context: &Context) -> String {
        // Simple hash of context values
        // In production, use a proper hash function
        format!("{:?}", context)
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

// Performance: Rule indexing for fast field lookups
pub struct IndexedRuleEngine {
    rules_by_field: HashMap<String, Vec<usize>>,
    all_rules: Vec<RuleWithPriority>,
    default_action: Action,
}

impl IndexedRuleEngine {
    pub fn new(engine: RuleEngine) -> Self {
        let mut rules_by_field: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, rule_with_priority) in engine.rules.iter().enumerate() {
            let fields = get_required_fields(&rule_with_priority.rule);
            for field in fields {
                rules_by_field
                    .entry(field)
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
        }

        IndexedRuleEngine {
            rules_by_field,
            all_rules: engine.rules,
            default_action: engine.default_action,
        }
    }

    pub fn evaluate(&self, context: &Context) -> EvaluationResult {
        // Only evaluate rules that reference fields present in context
        let relevant_indices: Vec<usize> = context
            .values
            .keys()
            .filter_map(|field| self.rules_by_field.get(field))
            .flatten()
            .copied()
            .collect();

        let mut matched = Vec::new();

        for idx in relevant_indices {
            if let Some(rule) = self.all_rules.get(idx) {
                if rule.rule.evaluate(context) {
                    matched.push(rule);
                }
            }
        }

        if matched.is_empty() {
            EvaluationResult {
                action: self.default_action.clone(),
                matched_rules: vec![],
            }
        } else {
            // Return highest priority
            let best = matched.iter().max_by_key(|r| r.priority).unwrap();
            EvaluationResult {
                action: best.action.clone(),
                matched_rules: vec![best.name.clone()],
            }
        }
    }
}
```

**Example JSON configuration:**
```json
{
  "name": "E-commerce Pricing Rules",
  "default_action": {
    "type": "approve"
  },
  "rules": [
    {
      "name": "weekend_big_discount",
      "priority": 100,
      "rule": {
        "type": "and",
        "rules": [
          { "type": "is_weekend" },
          {
            "type": "greater_than",
            "field": "total",
            "value": 100.0
          }
        ]
      },
      "action": {
        "type": "apply_discount",
        "percent": 25.0
      }
    },
    {
      "name": "gold_customer_discount",
      "priority": 80,
      "rule": {
        "type": "and",
        "rules": [
          {
            "type": "equals",
            "field": "customer_tier",
            "value": "gold"
          },
          {
            "type": "greater_than",
            "field": "total",
            "value": 50.0
          }
        ]
      },
      "action": {
        "type": "apply_discount",
        "percent": 15.0
      }
    },
    {
      "name": "business_hours_routing",
      "priority": 90,
      "rule": {
        "type": "and",
        "rules": [
          { "type": "is_weekday" },
          {
            "type": "hour_between",
            "start": 9,
            "end": 17
          }
        ]
      },
      "action": {
        "type": "route",
        "destination": "priority_queue"
      }
    }
  ]
}
```

**Usage example:**
```rust
// Load rules from file
let mut validator = RuleValidator::new();
validator.register_field("total".into(), ValueType::Float);
validator.register_field("customer_tier".into(), ValueType::String);

let loader = RuleLoader::new(validator);
let engine = loader.load_from_file("rules/pricing.json")?;

// Wrap in cached engine for performance
let cached_engine = CachedRuleEngine::new(engine);

// Use it
let mut context = Context::new();
context.set("total", Value::Float(150.0));
context.set("customer_tier", Value::String("gold".into()));
context.set("current_time", Value::DateTime(DateTime::now()));

let result = cached_engine.evaluate(&context);
println!("Action: {:?}", result.action);
println!("Matched rules: {:?}", result.matched_rules);
```

**Check/Test:**
- Test loading rules from JSON
- Test validation catches errors in loaded rules
- Test caching improves performance
- Test indexed engine with many rules
- Test hot-reload (load rules without restart)
- Test serialization/deserialization roundtrip
- Test performance with 1000+ rules

**Complete Solution:**
You now have a production-ready business rule engine with:
- ✅ Exhaustive pattern matching for type safety
- ✅ Rich rule types (comparison, ranges, strings, temporal)
- ✅ Logical composition (AND/OR/NOT)
- ✅ Priority system and conflict resolution
- ✅ Temporal logic (time, date, day of week)
- ✅ Rule validation before execution
- ✅ Explainability for debugging
- ✅ Dynamic rule loading from JSON
- ✅ Performance optimizations (caching, indexing)
- ✅ Extensible action system

This demonstrates all key pattern matching features:
- Guards (`if` conditions in match arms)
- Range patterns
- Destructuring (field extraction)
- Exhaustiveness checking
- Or-patterns for multiple cases
- Nested matching for complex logic

---
