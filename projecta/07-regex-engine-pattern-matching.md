# Project 1: Regular Expression Engine with Pattern Matching

## Learning Objectives

By completing this project, you will:

- Master **exhaustive pattern matching** on complex enum types
- Use **pattern guards** for validation and optimization
- Apply **range patterns** for character matching (`'a'..='z'`)
- Leverage **slice patterns** for sequence matching
- Practice **deep destructuring** with nested Box patterns
- Use **or-patterns** to combine multiple cases
- Apply `matches!` macro for concise checks
- Use **let-else** patterns for error handling
- Understand **recursive pattern matching** for AST traversal
- Build a complete regex engine demonstrating all pattern matching features

## Problem Statement

Build a regular expression engine that:
- Parses regex patterns into an Abstract Syntax Tree (AST)
- Evaluates patterns using Rust's pattern matching
- Supports literals, wildcards (.), character classes ([a-z]), quantifiers (*, +, ?, {n,m})
- Implements alternation (|), capture groups, and anchors (^, $, \b)
- Uses backtracking for non-greedy matching
- Optimizes patterns through pattern analysis
- Demonstrates ALL Rust pattern matching features


## Milestone 1: Basic Literal and Wildcard Matching

**Goal:** Build the foundation with literals, wildcards, and simple sequences.

**Concepts:**
- Exhaustive enum matching
- Recursive pattern matching
- Basic string traversal

### Implementation Steps

#### Step 1.1: Define Core AST Types

```rust
// TODO: Define the Regex enum representing all regex constructs
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    // TODO: Add Literal variant for exact string matching
 
    // TODO: Add Char variant for single character matching

    // TODO: Add Wildcard variant for '.' (any character)

    // TODO: Add Sequence variant for concatenation

    // TODO: Add Empty variant for empty pattern
}

// TODO: Implement Display for readable pattern output
impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Use exhaustive pattern matching to format each variant
        // Pseudocode:
        // match self {
        //     Literal(s) => write the string
        //     Char(c) => write the character
        //     Wildcard => write "."
        //     Sequence(exprs) => loop through and write each expression
        //     Empty => write nothing
        // }
        todo!()
    }
}
```

#### Step 1.2: Implement Basic Matching Engine

```rust
// TODO: Implement the matching engine
impl Regex {
    /// Match the regex against the input string at the current position
    pub fn is_match(&self, text: &str) -> bool {
        // TODO: Try matching at every position in the text
        // Pseudocode:
        // for each position i from 0 to text.len():
        //     if match_at(text, i) returns Some:
        //         return true
        // return false
        todo!()
    }

    /// Attempt to match at a specific position, returns number of chars consumed
    fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        // TODO: Use exhaustive pattern matching on self
        // Pseudocode:
        // match self {
        //     Literal(s) =>
        //         if text[pos..] starts with s:
        //             return Some(s.len())
        //         else:
        //             return None
        //
        //     Char(expected) =>
        //         get next char from text[pos..]
        //         if it equals expected:
        //             return Some(char byte length)
        //         else:
        //             return None
        //
        //     Wildcard =>
        //         get next char from text[pos..]
        //         if exists:
        //             return Some(char byte length)
        //         else:
        //             return None
        //
        //     Sequence(exprs) =>
        //         current_pos = pos
        //         for each expr in exprs:
        //             consumed = expr.match_at(text, current_pos)
        //             if consumed is None:
        //                 return None
        //             current_pos += consumed
        //         return Some(current_pos - pos)
        //
        //     Empty => return Some(0)
        // }
        todo!()
    }

    /// Find the position and length of the first match
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        // TODO: Return position and length of match
        // Pseudocode:
        // for each position i from 0 to text.len():
        //     if match_at(text, i) returns Some(len):
        //         return Some((i, len))
        // return None
        todo!()
    }
}
```

#### Step 1.3: Simple Parser

```rust
// TODO: Implement basic parser for literals and wildcards
pub fn parse_simple(pattern: &str) -> Result<Regex, ParseError> {
    // TODO: Handle empty pattern
    // Pseudocode:
    // if pattern is empty:
    //     return Ok(Regex::Empty)
    //
    // exprs = empty vector
    // for each character ch in pattern:
    //     match ch:
    //         '.' => push Regex::Wildcard to exprs
    //         c => push Regex::Char(c) to exprs
    //
    // Optimize single element sequences:
    // match exprs.as_slice():
    //     [] => return Ok(Regex::Empty)
    //     [single] => return Ok(single.clone())
    //     _ => return Ok(Regex::Sequence(exprs))
    todo!()
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedChar(char),
    UnexpectedEnd,
    InvalidRange,
    InvalidQuantifier,
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_matching() {
        let regex = Regex::Literal("hello".to_string());
        assert!(regex.is_match("hello world"));
        assert!(regex.is_match("say hello there"));
        assert!(!regex.is_match("HELLO"));
    }

    #[test]
    fn test_wildcard_matching() {
        let regex = parse_simple("h.llo").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(!regex.is_match("hllo"));
    }

    #[test]
    fn test_sequence_matching() {
        let regex = Regex::Sequence(vec![
            Regex::Char('a'),
            Regex::Wildcard,
            Regex::Char('c'),
        ]);
        assert!(regex.is_match("abc"));
        assert!(regex.is_match("axc"));
        assert!(!regex.is_match("ac"));
    }

    #[test]
    fn test_find_position() {
        let regex = parse_simple("lo").unwrap();
        assert_eq!(regex.find("hello"), Some((3, 2)));
        assert_eq!(regex.find("world"), Some((3, 2)));
        assert_eq!(regex.find("hi"), None);
    }
}
```

### Check Your Understanding

1. Why is exhaustive pattern matching important for regex engines?
2. How does the `match_at` function use recursion for Sequence matching?
3. What would happen if we forgot to handle a variant in the Display impl?
4. How would you extend this to support case-insensitive matching?

---

## Milestone 2: Character Classes and Range Patterns

**Goal:** Add character classes ([a-z], [0-9]) using range patterns.

**Concepts:**
- Range patterns (`'a'..='z'`)
- Pattern guards for validation
- Negated character classes

### Implementation Steps

#### Step 2.1: Add CharClass to AST

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    Sequence(Vec<Regex>),
    Empty,

    // TODO: Add character class variant
    // Hint: CharClass {
    //     ranges:          // e.g., ('a', 'z'), ('0', '9')
    //     chars:           // Individual characters
    //     negated:         // true for [^...], false for [...]
    // }
}

impl Regex {
    // TODO: Add helper to create character classes
    pub fn char_class(ranges: Vec<(char, char)>, chars: Vec<char>, negated: bool) -> Self {
        // Pseudocode:
        // return CharClass with the provided ranges, chars, and negated flag
        todo!()
    }

    // TODO: Helper for common classes
    pub fn digit() -> Self {
        // Pseudocode:
        // return CharClass with range ('0', '9'), empty chars, not negated
        todo!()
    }

    pub fn word_char() -> Self {
        // Pseudocode:
        // return CharClass with ranges [('a', 'z'), ('A', 'Z'), ('0', '9')]
        // and chars ['_'], not negated
        todo!()
    }

    pub fn whitespace() -> Self {
        // Pseudocode:
        // return CharClass with empty ranges and chars [' ', '\t', '\n', '\r']
        // not negated
        todo!()
    }
}
```

#### Step 2.2: Implement CharClass Matching with Range Patterns

```rust
impl Regex {
    fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        match self {
            // ... existing cases ...

            // TODO: Implement character class matching with range patterns
            Regex::CharClass { ranges, chars, negated } => {
                // Pseudocode:
                // get the next char from text[pos..]
                // if no char exists, return None
                //
                // Check if character matches any range or individual char:
                // matches = false
                // for each (start, end) in ranges:
                //     if char is in range start..=end:
                //         matches = true
                // if chars contains the character:
                //     matches = true
                //
                // Apply negation:
                // result = if negated then !matches else matches
                //
                // if result:
                //     return Some(char byte length)
                // else:
                //     return None
                todo!()
            }

            _ => todo!()
        }
    }
}
```

#### Step 2.3: Parse Character Classes

```rust
// TODO: Implement character class parser
fn parse_char_class(chars: &[char], pos: &mut usize) -> Result<Regex, ParseError> {
    // Pseudocode:
    // increment pos to skip '['
    //
    // Check for negation:
    // negated = false
    // if chars[*pos] == '^':
    //     negated = true
    //     increment pos
    //
    // ranges = empty vector
    // class_chars = empty vector
    //
    // while pos < chars.len() and chars[*pos] != ']':
    //     start = chars[*pos]
    //     increment pos
    //
    //     Check for range (a-z):
    //     if chars[*pos] == '-' and chars[*pos + 1] exists and != ']':
    //         increment pos to skip '-'
    //         end = chars[*pos]
    //         increment pos
    //
    //         Validate range:
    //         if start > end:
    //             return Err(ParseError::InvalidRange)
    //
    //         add (start, end) to ranges
    //     else:
    //         add start to class_chars
    //
    // Ensure we found closing ']':
    // if pos >= chars.len():
    //     return Err(ParseError::UnexpectedEnd)
    //
    // increment pos to skip ']'
    //
    // return Ok(Regex::CharClass { ranges, chars: class_chars, negated })
    todo!()
}
```

#### Step 2.4: Display CharClass

```rust
impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing cases ...

            // TODO: Format character classes
            Regex::CharClass { ranges, chars, negated } => {
                // Pseudocode:
                // write '['
                // if negated, write '^'
                // for each (start, end) in ranges:
                //     write "{start}-{end}"
                // for each ch in chars:
                //     write ch
                // write ']'
                todo!()
            }

            _ => todo!()
        }
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_char_class_ranges() {
    let regex = Regex::char_class(vec![('a', 'z')], vec![], false);
    assert!(regex.is_match("hello"));
    assert!(!regex.is_match("HELLO"));
    assert!(!regex.is_match("123"));
}

#[test]
fn test_char_class_multiple_ranges() {
    let regex = Regex::char_class(
        vec![('a', 'z'), ('A', 'Z'), ('0', '9')],
        vec![],
        false,
    );
    assert!(regex.is_match("a"));
    assert!(regex.is_match("Z"));
    assert!(regex.is_match("5"));
    assert!(!regex.is_match("!"));
}

#[test]
fn test_negated_char_class() {
    let regex = Regex::char_class(vec![('a', 'z')], vec![], true);
    assert!(!regex.is_match("hello"));
    assert!(regex.is_match("HELLO"));
    assert!(regex.is_match("123"));
}

#[test]
fn test_range_pattern_matching() {
    // Test that our range matching works correctly
    let ch = 'm';
    let in_range = matches!(ch, 'a'..='z');
    assert!(in_range);

    let ch2 = 'M';
    let not_in_range = matches!(ch2, 'a'..='z');
    assert!(!not_in_range);
}
```

### Check Your Understanding

1. How do range patterns (`'a'..='z'`) improve code readability over manual comparisons?
2. Why is negation handled differently from normal character class matching?
3. What happens if we create an invalid range like `[z-a]`?
4. How would you optimize character class matching for large ranges?

---

## Milestone 3: Quantifiers with Backtracking

**Goal:** Add *, +, ?, {n,m} quantifiers with backtracking.

**Concepts:**
- Recursive backtracking
- Pattern guards for validation
- Deep destructuring of quantifier bounds

### Implementation Steps

#### Step 3.1: Add Quantifier Variants

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    Sequence(Vec<Regex>),
    Empty,
    CharClass { ranges: Vec<(char, char)>, chars: Vec<char>, negated: bool },

    // TODO: Add quantifier variant
    // Hint: Repeat {
    //     expr: Box<Regex>,
    //     min: usize,          // Minimum repetitions
    //     max: Option<usize>,  // None means unlimited (*)
    // }
}

impl Regex {
    // TODO: Add helper constructors
    pub fn zero_or_more(expr: Regex) -> Self {
        // Pseudocode: return Repeat with min=0, max=None
        todo!()
    }

    pub fn one_or_more(expr: Regex) -> Self {
        // Pseudocode: return Repeat with min=1, max=None
        todo!()
    }

    pub fn optional(expr: Regex) -> Self {
        // Pseudocode: return Repeat with min=0, max=Some(1)
        todo!()
    }

    pub fn exactly(expr: Regex, n: usize) -> Self {
        // Pseudocode: return Repeat with min=n, max=Some(n)
        todo!()
    }

    pub fn between(expr: Regex, min: usize, max: usize) -> Self {
        // Pseudocode: return Repeat with min=min, max=Some(max)
        todo!()
    }
}
```

#### Step 3.2: Implement Greedy Matching with Backtracking

```rust
impl Regex {
    fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        match self {
            // ... existing cases ...

            // TODO: Implement quantifier matching with backtracking
            Regex::Repeat { expr, min, max } => {
                // Pseudocode:
                // First, match minimum required times:
                // current_pos = pos
                // for i in 0..*min:
                //     consumed = expr.match_at(text, current_pos)
                //     if consumed is None:
                //         return None (failed to meet minimum)
                //     current_pos += consumed
                //
                // Greedily match as many as possible:
                // matches = empty vector
                // loop:
                //     Check if we've hit the maximum:
                //     if max is Some(max_count) and matches.len() >= max_count:
                //         break
                //
                //     Try to match one more:
                //     consumed = expr.match_at(text, current_pos)
                //     if consumed is Some:
                //         add consumed to matches
                //         current_pos += consumed
                //     else:
                //         break
                //
                // return Some(current_pos - pos)
                todo!()
            }

            _ => todo!()
        }
    }
}
```

#### Step 3.3: Add Pattern Guards for Quantifier Validation

```rust
// TODO: Validate quantifier bounds
pub fn validate_quantifier(min: usize, max: Option<usize>) -> Result<(), String> {
    // Pseudocode:
    // match (min, max):
    //     (m, Some(mx)) if m <= mx => Ok(())
    //     (_, None) => Ok(()) // unlimited max is valid
    //     (m, Some(mx)) if m > mx => Err("min > max")
    //     _ => unreachable
    todo!()
}

// TODO: Describe quantifier using pattern matching
pub fn describe_quantifier(repeat: &Regex) -> String {
    // Pseudocode:
    // match repeat:
    //     Repeat { min: 0, max: None, .. } => "zero or more (*)"
    //     Repeat { min: 1, max: None, .. } => "one or more (+)"
    //     Repeat { min: 0, max: Some(1), .. } => "optional (?)"
    //     Repeat { min, max: Some(max_val), .. } if min == max_val => "exactly {min}"
    //     Repeat { min, max: Some(max_val), .. } => "between {min} and {max_val}"
    //     Repeat { min, max: None, .. } => "at least {min}"
    //     _ => "not a quantifier"
    todo!()
}
```

#### Step 3.4: Display Quantifiers

```rust
impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // ... existing cases ...

            // TODO: Format quantifiers using pattern matching
            Regex::Repeat { expr, min, max } => {
                // Pseudocode:
                // write expr
                // match (min, max):
                //     (0, None) => write "*"
                //     (1, None) => write "+"
                //     (0, Some(1)) => write "?"
                //     (m, Some(mx)) if m == mx => write "{{{m}}}"
                //     (m, Some(mx)) => write "{{{m},{mx}}}"
                //     (m, None) => write "{{{m},}}"
                todo!()
            }

            _ => todo!()
        }
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_zero_or_more() {
    let regex = Regex::zero_or_more(Regex::Char('a'));
    assert!(regex.is_match(""));
    assert!(regex.is_match("a"));
    assert!(regex.is_match("aaaa"));
    assert!(regex.is_match("baaaa")); // Matches at position 1
}

#[test]
fn test_one_or_more() {
    let regex = Regex::one_or_more(Regex::Char('a'));
    assert!(!regex.is_match(""));
    assert!(regex.is_match("a"));
    assert!(regex.is_match("aaaa"));
}

#[test]
fn test_exact_count() {
    let regex = Regex::exactly(Regex::Char('a'), 3);
    assert!(!regex.is_match("aa"));
    assert!(regex.is_match("aaa"));
    assert!(regex.is_match("aaaa")); // Matches first 3
}
```

### Check Your Understanding

1. Why does greedy matching try to consume as many characters as possible?
2. How would you implement non-greedy (lazy) matching?
3. What role do pattern guards play in quantifier validation?
4. How does backtracking help when greedy matching fails?

---

## Milestone 4: Alternation and Capture Groups

**Goal:** Add | alternation and () capture groups with deep destructuring.

**Concepts:**
- Deep destructuring with Box patterns
- Or-patterns for combining cases
- Capture group tracking

### Implementation Steps

#### Step 4.1: Add Alternation and Group Variants

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    Sequence(Vec<Regex>),
    Empty,
    CharClass { ranges: Vec<(char, char)>, chars: Vec<char>, negated: bool },
    Repeat { expr: Box<Regex>, min: usize, max: Option<usize> },

    // TODO: Add alternation (a|b|c)
    // Hint: Alternation(Vec<Regex>)

    // TODO: Add capture groups
    // Hint: Group {
    //     expr: Box<Regex>,
    //     id: usize,  // Group number for captures
    // }
}

impl Regex {
    pub fn alt(options: Vec<Regex>) -> Self {
        // Pseudocode: return Alternation(options)
        todo!()
    }

    pub fn group(expr: Regex, id: usize) -> Self {
        // Pseudocode: return Group with boxed expr and id
        todo!()
    }
}
```

#### Step 4.2: Implement Alternation Matching

```rust
impl Regex {
    fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        match self {
            // ... existing cases ...

            // TODO: Try each alternative until one succeeds
            Regex::Alternation(alts) => {
                // Pseudocode:
                // for each alt in alts:
                //     consumed = alt.match_at(text, pos)
                //     if consumed is Some:
                //         return consumed
                // return None
                todo!()
            }

            // TODO: Groups are transparent for basic matching
            Regex::Group { expr, .. } => {
                // Pseudocode: return expr.match_at(text, pos)
                todo!()
            }

            _ => todo!()
        }
    }
}
```

#### Step 4.3: Add Capture Extraction with Deep Destructuring

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Captures {
    pub full_match: Match,
    pub groups: Vec<Option<Match>>,
}

impl Regex {
    // TODO: Match with capture extraction
    pub fn captures(&self, text: &str) -> Option<Captures> {
        // Pseudocode:
        // for each position i from 0 to text.len():
        //     groups = empty vector
        //     len = self.match_at_with_captures(text, i, &mut groups)
        //     if len is Some:
        //         return Some(Captures {
        //             full_match: Match with i, i+len, and text slice
        //             groups: groups
        //         })
        // return None
        todo!()
    }

    fn match_at_with_captures(
        &self,
        text: &str,
        pos: usize,
        captures: &mut Vec<Option<Match>>,
    ) -> Option<usize> {
        // Pseudocode:
        // match self:
        //     Group { expr, id } =>
        //         Ensure captures vec is large enough for id
        //         start = pos
        //         consumed = expr.match_at_with_captures(text, pos, captures)
        //         if consumed is Some:
        //             end = pos + consumed
        //             Store Match in captures[id]
        //             return consumed
        //         return None
        //
        //     Sequence(exprs) =>
        //         current_pos = pos
        //         for each expr:
        //             consumed = expr.match_at_with_captures(text, current_pos, captures)
        //             if consumed is None: return None
        //             current_pos += consumed
        //         return Some(current_pos - pos)
        //
        //     ... handle other variants similarly ...
        todo!()
    }
}
```

#### Step 4.4: Pattern Matching for Analysis

```rust
// TODO: Analyze regex complexity using deep destructuring
pub fn count_groups(regex: &Regex) -> usize {
    // Pseudocode:
    // match regex:
    //     Group { expr, .. } => 1 + count_groups(expr)
    //     Sequence(exprs) => sum of count_groups for all exprs
    //     Alternation(alts) => sum of count_groups for all alts
    //     Repeat { expr: box inner, .. } => count_groups(inner)  // deep destructuring
    //     _ => 0 (leaf nodes have no groups)
    todo!()
}

// TODO: Check if regex contains alternation
pub fn has_alternation(regex: &Regex) -> bool {
    // Pseudocode:
    // match regex:
    //     Alternation(_) => true
    //     Sequence(exprs) => any expr has_alternation
    //     Group { expr: box inner, .. } => has_alternation(inner)
    //     Repeat { expr: box inner, .. } => has_alternation(inner)
    //     _ => false
    todo!()
}
```

### Checkpoint Tests

```rust
#[test]
fn test_alternation() {
    let regex = Regex::alt(vec![
        Regex::Literal("cat".to_string()),
        Regex::Literal("dog".to_string()),
        Regex::Literal("bird".to_string()),
    ]);

    assert!(regex.is_match("cat"));
    assert!(regex.is_match("dog"));
    assert!(regex.is_match("bird"));
    assert!(!regex.is_match("fish"));
}

#[test]
fn test_capture_groups() {
    // (a+)(b+)
    let regex = Regex::Sequence(vec![
        Regex::group(Regex::one_or_more(Regex::Char('a')), 0),
        Regex::group(Regex::one_or_more(Regex::Char('b')), 1),
    ]);

    let caps = regex.captures("aaabbb").unwrap();
    assert_eq!(caps.full_match.text, "aaabbb");
    assert_eq!(caps.groups[0].as_ref().unwrap().text, "aaa");
    assert_eq!(caps.groups[1].as_ref().unwrap().text, "bbb");
}
```

### Check Your Understanding

1. How does deep destructuring with `box` patterns simplify nested regex analysis?
2. Why do we clone captures when trying alternation branches?
3. What would happen if we forgot to handle captures in the Repeat variant?
4. How would you implement backreferences (e.g., `\1` to match first capture again)?

---

## Milestone 5: Anchors, Optimization, and Comprehensive Pattern Matching

**Goal:** Add anchors (^, $, \b) and optimize using pattern analysis.

**Concepts:**
- Pattern guards for anchor validation
- Matches! macro for quick checks
- Let-else for error handling
- Comprehensive optimization

### Implementation Steps

#### Step 5.1: Add Anchor Variants

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    Sequence(Vec<Regex>),
    Empty,
    CharClass { ranges: Vec<(char, char)>, chars: Vec<char>, negated: bool },
    Repeat { expr: Box<Regex>, min: usize, max: Option<usize> },
    Alternation(Vec<Regex>),
    Group { expr: Box<Regex>, id: usize },

    // TODO: Add anchors
    // Hint: Anchor(AnchorKind)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorKind {
    StartOfLine,   // ^
    EndOfLine,     // $
    WordBoundary,  // \b
}
```

#### Step 5.2: Implement Anchor Matching

```rust
impl Regex {
    fn match_at(&self, text: &str, pos: usize) -> Option<usize> {
        match self {
            // ... existing cases ...

            // TODO: Implement anchor matching with guards
            Regex::Anchor(kind) => {
                // Pseudocode:
                // match kind:
                //     StartOfLine if pos == 0 => Some(0)
                //     EndOfLine if pos == text.len() => Some(0)
                //     WordBoundary =>
                //         if is_word_boundary(text, pos):
                //             Some(0)
                //         else:
                //             None
                //     _ => None (anchor not satisfied at this position)
                todo!()
            }

            _ => todo!()
        }
    }

    // TODO: Helper to check word boundaries
    fn is_word_boundary(text: &str, pos: usize) -> bool {
        // Pseudocode:
        // before = if pos > 0: last char before pos else None
        // after = char at pos
        //
        // before_is_word = matches!(before, Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
        // after_is_word = matches!(after, Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
        //
        // Boundary if exactly one side is a word character:
        // return before_is_word != after_is_word
        todo!()
    }
}
```

#### Step 5.3: Optimize Patterns Using Pattern Matching

```rust
// TODO: Optimize regex by simplifying patterns
pub fn optimize(regex: Regex) -> Regex {
    // Pseudocode:
    // match regex:
    //     Sequence(exprs) if exprs is empty => Regex::Empty
    //
    //     Sequence(exprs) match exprs.as_slice():
    //         [] => Regex::Empty
    //         [single] => optimize(single.clone())
    //         _ => Sequence of optimized exprs
    //
    //     Alternation(alts) match alts.as_slice():
    //         [] => Regex::Empty
    //         [single] => optimize(single.clone())
    //         _ => Alternation of optimized alts
    //
    //     Repeat { expr, min, max } =>
    //         optimized_expr = optimize(expr)
    //         match (min, max):
    //             (0, Some(0)) => Regex::Empty  // {0,0} is empty
    //             (1, Some(1)) => optimized_expr  // {1,1} is just the expression
    //             _ => Repeat with optimized_expr
    //
    //     Group { expr, id } => Group with optimized expr
    //
    //     other => other (leaf nodes don't need optimization)
    todo!()
}

// TODO: Check if regex is anchored at start
pub fn is_anchored_at_start(regex: &Regex) -> bool {
    // Pseudocode:
    // match regex:
    //     Anchor(StartOfLine) => true
    //     Sequence(exprs) =>
    //         first element is Anchor(StartOfLine)
    //     Group { expr, .. } => is_anchored_at_start(expr)
    //     _ => false
    todo!()
}

// TODO: Check if regex matches only fixed strings (no wildcards/quantifiers)
pub fn is_literal_only(regex: &Regex) -> bool {
    // Pseudocode:
    // match regex:
    //     Literal(_) | Char(_) | Empty => true
    //     Sequence(exprs) => all exprs are is_literal_only
    //     Group { expr, .. } => is_literal_only(expr)
    //     Wildcard | CharClass { .. } | Repeat { .. } | Alternation(_) | Anchor(_) => false
    todo!()
}
```

#### Step 5.4: Let-Else for Error Handling

```rust
// TODO: Extract literal string from regex using let-else
pub fn extract_literal(regex: &Regex) -> Result<String, &'static str> {
    // Pseudocode:
    // let Regex::Literal(s) = regex else {
    //     return Err("Not a literal pattern");
    // };
    // Ok(s.clone())
    todo!()
}

// TODO: Extract group ID using let-else
pub fn extract_group_id(regex: &Regex) -> Result<usize, &'static str> {
    // Pseudocode:
    // let Regex::Group { id, .. } = regex else {
    //     return Err("Not a group");
    // };
    // Ok(*id)
    todo!()
}

// TODO: Get quantifier bounds using let-else
pub fn get_quantifier_bounds(regex: &Regex) -> Result<(usize, Option<usize>), &'static str> {
    // Pseudocode:
    // let Regex::Repeat { min, max, .. } = regex else {
    //     return Err("Not a quantifier");
    // };
    // Ok((*min, *max))
    todo!()
}
```

#### Step 5.5: Comprehensive Display Implementation

```rust
impl std::fmt::Display for Regex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Exhaustive pattern matching for all variants
        // Pseudocode:
        // match self:
        //     Literal(s) => write s
        //     Char(c) => write c
        //     Wildcard => write "."
        //     CharClass { ... } => write [...] with ranges and chars
        //     Sequence(exprs) => write each expr
        //     Repeat { expr, min, max } =>
        //         Add parentheses if expr is complex
        //         Write expr
        //         Write quantifier suffix (*, +, ?, {n,m})
        //     Alternation(alts) => write alts separated by |
        //     Group { expr, .. } => write (expr)
        //     Anchor(kind) =>
        //         StartOfLine => write "^"
        //         EndOfLine => write "$"
        //         WordBoundary => write "\\b"
        //     Empty => write nothing
        todo!()
    }
}
```

### Checkpoint Tests

```rust
#[test]
fn test_start_anchor() {
    let regex = Regex::Sequence(vec![
        Regex::Anchor(AnchorKind::StartOfLine),
        Regex::Literal("hello".to_string()),
    ]);

    assert!(regex.is_match("hello world"));
    assert!(!regex.is_match("say hello"));
}

#[test]
fn test_word_boundary() {
    let regex = Regex::Sequence(vec![
        Regex::Anchor(AnchorKind::WordBoundary),
        Regex::Literal("word".to_string()),
        Regex::Anchor(AnchorKind::WordBoundary),
    ]);

    assert!(regex.is_match("a word here"));
    assert!(regex.is_match("word"));
    assert!(!regex.is_match("sword"));
    assert!(!regex.is_match("words"));
}

#[test]
fn test_optimization() {
    let regex = Regex::Sequence(vec![Regex::Char('a')]);
    let optimized = optimize(regex);
    assert_eq!(optimized, Regex::Char('a'));
}
```

### Check Your Understanding

1. How do pattern guards help validate anchor positions?
2. Why is the `matches!` macro useful for character classification?
3. How does let-else improve error handling compared to if-let?
4. What optimizations could you add for common patterns like `a*a+` → `a+`?

---

## Complete Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        // Simplified: [a-z]+@[a-z]+\.[a-z]+
        let regex = Regex::Sequence(vec![
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
            Regex::Char('@'),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
            Regex::Char('.'),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
        ]);

        assert!(regex.is_match("user@example.com"));
        assert!(regex.is_match("test@test.org"));
        assert!(!regex.is_match("invalid"));
        assert!(!regex.is_match("@example.com"));
    }

    #[test]
    fn test_phone_number() {
        // \d{3}-\d{3}-\d{4}
        let regex = Regex::Sequence(vec![
            Regex::exactly(Regex::digit(), 3),
            Regex::Char('-'),
            Regex::exactly(Regex::digit(), 3),
            Regex::Char('-'),
            Regex::exactly(Regex::digit(), 4),
        ]);

        assert!(regex.is_match("123-456-7890"));
        assert!(!regex.is_match("1234567890"));
        assert!(!regex.is_match("123-45-6789"));
    }

    #[test]
    fn test_url_matching() {
        // https?://[a-z]+\.[a-z]+
        let regex = Regex::Sequence(vec![
            Regex::Literal("http".to_string()),
            Regex::optional(Regex::Char('s')),
            Regex::Literal("://".to_string()),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
            Regex::Char('.'),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
        ]);

        assert!(regex.is_match("http://example.com"));
        assert!(regex.is_match("https://test.org"));
        assert!(!regex.is_match("ftp://example.com"));
    }
}
```

## Benchmarks

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_simple_literal() {
        let regex = Regex::Literal("test".to_string());
        let text = "this is a test string with test repeated";

        let start = Instant::now();
        for _ in 0..100_000 {
            regex.is_match(text);
        }
        let elapsed = start.elapsed();
        println!("Simple literal: {:?}", elapsed);
    }

    #[test]
    fn bench_complex_pattern() {
        let regex = Regex::Sequence(vec![
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
            Regex::Char('@'),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
        ]);
        let text = "contact user@example.com for info";

        let start = Instant::now();
        for _ in 0..10_000 {
            regex.is_match(text);
        }
        let elapsed = start.elapsed();
        println!("Complex pattern: {:?}", elapsed);
    }
}
```

## Extensions and Challenges

1. **Non-Greedy Matching**: Implement `*?`, `+?`, `??` lazy quantifiers
2. **Backreferences**: Support `\1`, `\2` to match previous captures
3. **Lookahead/Lookbehind**: Implement `(?=...)` and `(?<=...)`
4. **Named Groups**: Support `(?P<name>...)` for named captures
5. **Unicode Support**: Handle Unicode character classes (`\p{L}`)
6. **Optimization**: Implement NFA/DFA compilation for faster matching
7. **Parser**: Build a complete regex parser from strings
8. **Replace**: Implement `replace()` and `replace_all()` functions

## Pattern Matching Features Demonstrated

✅ **Exhaustive Matching**: All Regex variants handled
✅ **Pattern Guards**: Validation in anchors and quantifiers
✅ **Range Patterns**: Character classes with `'a'..='z'`
✅ **Slice Patterns**: Sequence optimization with `match exprs.as_slice()`
✅ **Deep Destructuring**: `box` patterns for nested expressions
✅ **Or-Patterns**: `Regex::Literal(_) | Regex::Char(_)`
✅ **Matches! Macro**: Quick character checks
✅ **Let-Else**: Error handling in extract functions
✅ **If-Let Chains**: Capture extraction logic
✅ **Tuple Matching**: Quantifier bounds `(min, max)`

## Real-World Applications

- **Web Frameworks**: URL routing and parameter extraction
- **Text Editors**: Search and replace functionality
- **Compilers**: Lexical analysis and tokenization
- **Validation**: Form input validation
- **Log Analysis**: Extract structured data from logs
- **Data Extraction**: Web scraping and parsing

This project demonstrates how Rust's pattern matching system naturally models the structure of regular expressions, making the code both safe (exhaustive checking) and maintainable (clear structure).
