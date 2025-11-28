# Project 1: Regular Expression Engine with Pattern Matching

## Problem Statement

Build a **regular expression engine** that parses regex patterns into an Abstract Syntax Tree (AST), executes matches using exhaustive pattern matching, and demonstrates Rust's pattern matching system through a complete domain-specific language processor.

**Core Features**:
- Parse regex patterns into AST
- Execute matches using pattern matching
- Support: literals, wildcards (.), character classes ([a-z]), quantifiers (*, +, ?), alternation (|), groups ()
- Implement backtracking for complex patterns
- Extract captures from groups
- Optimize with pattern guards and range matching

**Use Cases**:
- Text editors (find/replace with regex)
- Form validation (email, phone, URL)
- Log analyzers (extract structured data)
- Web scrapers (extract from HTML)
- Compilers (tokenization/lexing)
- Network tools (grep-like packet search)

## Why It Matters

Regular expressions demonstrate **enum-driven architecture** and **exhaustive pattern matching**:

**Pattern Matching Benefits**:
- **Exhaustiveness**: Compiler ensures all regex operators handled
- **Guards**: Enable optimization (literal string fast path)
- **Destructuring**: Simplifies AST traversal
- **State Machines**: Map directly to Rust enums
- **Type Safety**: Invalid patterns caught at compile time

**Real-World Impact**:
- Text processing is fundamental to software
- Regex engines power editors, validators, parsers
- Understanding regex internals aids debugging
- Pattern matching is ideal for DSL implementation

---

## Milestone 1: Basic Literal and Wildcard Matching

**Goal**: Implement simple string matching with literals and wildcards.

**Concepts**:
- Enum definition for regex AST
- Basic pattern matching with `match`
- Position-based matching
- Sequence handling

**Implementation Steps**:

1. **Define the `Regex` enum**:
   - `Literal(String)` for literal text
   - `Wildcard` for `.` (match any character)
   - `Sequence(Vec<Regex>)` for concatenation

2. **Implement `match_at` method**:
   - Takes input and position
   - Returns `Option<usize>` (new position after match)
   - Uses pattern matching on enum variants

3. **Implement `is_match` method**:
   - Wrapper that tries matching from position 0
   - Returns boolean success

4. **Implement basic parser**:
   - Parse pattern string into `Regex` AST
   - Handle `.` and regular characters

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    // TODO: Add Literal variant holding String
    // TODO: Add Wildcard variant (unit variant)
    // TODO: Add Sequence variant holding Vec<Regex>
}

#[derive(Debug)]
pub enum ParseError {
    InvalidSyntax(String),
}

impl Regex {
    pub fn is_match(&self, input: &str) -> bool {
        // TODO: Try matching from position 0
        // Hint: Call match_at(input, 0) and check if Some
        todo!()
    }

    fn match_at(&self, input: &str, pos: usize) -> Option<usize> {
        // TODO: Pattern match on self
        match self {
            // TODO: Regex::Literal(s) case
            // Check if input[pos..] starts with s
            // Return Some(pos + s.len()) if match, None otherwise

            // TODO: Regex::Wildcard case
            // Match any single character
            // Return Some(pos + 1) if pos < input.len(), None otherwise
            // Hint: Handle UTF-8 with input[pos..].chars().next()

            // TODO: Regex::Sequence(exprs) case
            // Match each expression in sequence
            // Thread position through: pos → expr1 → pos1 → expr2 → pos2 → ...
            // Return final position if all match, None if any fails

            _ => None,
        }
    }

    pub fn parse(pattern: &str) -> Result<Self, ParseError> {
        // TODO: Parse pattern into Regex AST
        let mut exprs = Vec::new();
        let mut chars = pattern.chars().peekable();

        while let Some(ch) = chars.next() {
            let expr = match ch {
                // TODO: Handle '.' → Wildcard
                // TODO: Handle other chars → Literal
                _ => todo!(),
            };
            exprs.push(expr);
        }

        // TODO: Return single expr if len == 1, otherwise Sequence
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
    fn test_literal_match() {
        let regex = Regex::parse("hello").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hello world"));
        assert!(!regex.is_match("hell"));
        assert!(!regex.is_match("goodbye"));
    }

    #[test]
    fn test_wildcard() {
        let regex = Regex::parse("h.llo").unwrap();
        assert!(regex.is_match("hello"));
        assert!(regex.is_match("hallo"));
        assert!(regex.is_match("h@llo"));
        assert!(!regex.is_match("hllo"));
    }

    #[test]
    fn test_sequence() {
        let regex = Regex::parse("a.c").unwrap();
        assert!(regex.is_match("abc"));
        assert!(regex.is_match("a1c"));
        assert!(!regex.is_match("ac"));
        assert!(!regex.is_match("abbc"));
    }

    #[test]
    fn test_empty_pattern() {
        let regex = Regex::parse("").unwrap();
        assert!(regex.is_match(""));
        assert!(regex.is_match("anything"));
    }
}
```

**Check Your Understanding**:
1. Why does `match_at` return `Option<usize>` instead of `bool`?
2. How does the `Sequence` variant enable matching multiple patterns?
3. What happens if we try to match beyond the input length?

---

## Milestone 2: Character Classes and Range Patterns

**Goal**: Implement character classes `[a-z]` with range matching and pattern guards.

**Concepts**:
- Range patterns in match expressions
- Pattern guards with `if`
- Negated character classes `[^0-9]`
- Multiple ranges in one class

**Implementation Steps**:

1. **Extend `Regex` enum**:
   - Add `Char(char)` for single characters
   - Add `CharClass { ranges, chars, negated }`

2. **Define `CharRange` struct**:
   - Fields: `start: char`, `end: char`
   - Method: `contains(&self, ch: char) -> bool`

3. **Implement character class matching**:
   - Check if character is in ranges
   - Check if character is in explicit list
   - Apply negation if needed

4. **Parse character classes**:
   - Handle `[a-z]`, `[abc]`, `[^0-9]`
   - Parse ranges vs individual characters

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
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
        // TODO: Check if ch is in range [start, end]
        ch >= self.start && ch <= self.end
    }
}

impl Regex {
    fn match_at(&self, input: &str, pos: usize) -> Option<usize> {
        let chars: Vec<char> = input.chars().collect();

        if pos >= chars.len() {
            return match self {
                Regex::Sequence(exprs) if exprs.is_empty() => Some(pos),
                _ => None,
            };
        }

        match self {
            Regex::Char(expected) => {
                // TODO: Match single character
                // Return Some(pos + 1) if chars[pos] == *expected
                todo!()
            }

            Regex::Wildcard => {
                // TODO: Match any character
                Some(pos + 1)
            }

            Regex::CharClass { ranges, chars: class_chars, negated } => {
                // TODO: Get character at position
                let ch = chars[pos];

                // TODO: Check if in ranges
                let in_range = ranges.iter().any(|r| r.contains(ch));

                // TODO: Check if in explicit chars
                let in_chars = class_chars.contains(&ch);

                // TODO: Combine with OR
                let matches = in_range || in_chars;

                // TODO: Apply negation: XOR with negated flag
                if *negated != matches {
                    Some(pos + 1)
                } else {
                    None
                }
            }

            Regex::Sequence(exprs) => {
                // TODO: Same as before
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

// Helper to demonstrate range patterns
fn classify_char(ch: char) -> &'static str {
    match ch {
        // TODO: Use range patterns
        '0'..='9' => "digit",
        'a'..='z' => "lowercase",
        'A'..='Z' => "uppercase",
        ' ' | '\t' | '\n' => "whitespace",
        _ => "other",
    }
}

// Parse character class helper
fn parse_char_class(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Regex, ParseError> {
    let mut ranges = Vec::new();
    let mut class_chars = Vec::new();

    // TODO: Check for negation (^)
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

        // TODO: Check for range (-)
        if chars.peek() == Some(&'-') {
            chars.next(); // consume '-'

            let end = chars.next()
                .ok_or_else(|| ParseError::InvalidSyntax("Incomplete range".into()))?;

            if end == ']' {
                // [a-] means 'a' or '-'
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

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_range() {
        let regex = Regex::parse("[a-z]").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("m"));
        assert!(regex.is_match("z"));
        assert!(!regex.is_match("A"));
        assert!(!regex.is_match("1"));
    }

    #[test]
    fn test_char_list() {
        let regex = Regex::parse("[abc]").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("b"));
        assert!(regex.is_match("c"));
        assert!(!regex.is_match("d"));
    }

    #[test]
    fn test_negated_class() {
        let regex = Regex::parse("[^0-9]").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("Z"));
        assert!(!regex.is_match("0"));
        assert!(!regex.is_match("5"));
    }

    #[test]
    fn test_multiple_ranges() {
        let regex = Regex::parse("[a-zA-Z0-9]").unwrap();
        assert!(regex.is_match("a"));
        assert!(regex.is_match("Z"));
        assert!(regex.is_match("5"));
        assert!(!regex.is_match("@"));
    }

    #[test]
    fn test_range_patterns() {
        assert_eq!(classify_char('5'), "digit");
        assert_eq!(classify_char('a'), "lowercase");
        assert_eq!(classify_char('Z'), "uppercase");
        assert_eq!(classify_char(' '), "whitespace");
        assert_eq!(classify_char('@'), "other");
    }
}
```

**Check Your Understanding**:
1. How does the `contains` method use range comparison for characters?
2. Why do we use `negated != matches` instead of `if negated { !matches } else { matches }`?
3. What are the advantages of using range patterns (`'a'..='z'`) in match expressions?

---

## Milestone 3: Quantifiers with Backtracking

**Goal**: Implement `*`, `+`, `?`, `{n,m}` quantifiers using pattern matching for different repetition cases.

**Concepts**:
- Greedy matching
- Backtracking algorithm
- Pattern guards for quantifier types
- Recursive matching

**Implementation Steps**:

1. **Add `Repeat` variant**:
   - Fields: `expr: Box<Regex>`, `min: usize`, `max: Option<usize>`

2. **Implement greedy matching**:
   - Match as many repetitions as possible
   - Backtrack if later patterns fail

3. **Add helper constructors**:
   - `zero_or_more` for `*`
   - `one_or_more` for `+`
   - `optional` for `?`

4. **Parse quantifiers**:
   - Attach to previous expression
   - Handle `{n,m}` syntax

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    CharClass {
        ranges: Vec<CharRange>,
        chars: Vec<char>,
        negated: bool,
    },
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
                // TODO: Match minimum required repetitions
                let mut count = 0;
                let mut current_pos = pos;

                for _ in 0..*min {
                    current_pos = expr.match_at(input, current_pos)?;
                    count += 1;
                }

                // TODO: Match up to maximum (greedy)
                while max.map_or(true, |m| count < m) {
                    match expr.match_at(input, current_pos) {
                        Some(new_pos) => {
                            current_pos = new_pos;
                            count += 1;
                        }
                        None => break,
                    }
                }

                Some(current_pos)
            }

            _ => None,
        }
    }

    // Helper constructors
    pub fn zero_or_more(expr: Regex) -> Self {
        // TODO: Create Repeat with min=0, max=None
        Regex::Repeat {
            expr: Box::new(expr),
            min: 0,
            max: None,
        }
    }

    pub fn one_or_more(expr: Regex) -> Self {
        // TODO: Create Repeat with min=1, max=None
        todo!()
    }

    pub fn optional(expr: Regex) -> Self {
        // TODO: Create Repeat with min=0, max=Some(1)
        todo!()
    }
}

// Pattern matching for optimization
fn optimize_quantifier(repeat: &Regex) -> String {
    match repeat {
        // TODO: Match specific quantifier types with guards
        Regex::Repeat { min: 0, max: None, .. } => "zero or more (*)".to_string(),
        Regex::Repeat { min: 1, max: None, .. } => "one or more (+)".to_string(),
        Regex::Repeat { min: 0, max: Some(1), .. } => "optional (?)".to_string(),
        Regex::Repeat { min, max: Some(max_val), .. } if min == max_val => {
            format!("exactly {}", min)
        }
        Regex::Repeat { min, max, .. } => {
            let max_str = max.map_or("∞".to_string(), |m| m.to_string());
            format!("{} to {}", min, max_str)
        }
        _ => "not a quantifier".to_string(),
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_or_more() {
        let regex = Regex::zero_or_more(Regex::Char('a'));
        assert!(regex.is_match(""));
        assert!(regex.is_match("a"));
        assert!(regex.is_match("aaa"));
    }

    #[test]
    fn test_one_or_more() {
        let regex = Regex::one_or_more(Regex::Char('a'));
        assert!(!regex.is_match(""));
        assert!(regex.is_match("a"));
        assert!(regex.is_match("aaaa"));
    }

    #[test]
    fn test_optional() {
        let regex = Regex::optional(Regex::Char('a'));
        assert!(regex.is_match(""));
        assert!(regex.is_match("a"));
        // Optional matches 0 or 1, so "aa" matches first 'a' only
    }

    #[test]
    fn test_greedy_matching() {
        // a*a should match "aaa" completely
        let regex = Regex::Sequence(vec![
            Regex::zero_or_more(Regex::Char('a')),
            Regex::Char('a'),
        ]);
        assert!(regex.is_match("aaa"));
        assert!(!regex.is_match(""));
    }

    #[test]
    fn test_quantifier_description() {
        let star = Regex::zero_or_more(Regex::Char('a'));
        assert_eq!(optimize_quantifier(&star), "zero or more (*)");

        let plus = Regex::one_or_more(Regex::Char('a'));
        assert_eq!(optimize_quantifier(&plus), "one or more (+)");

        let opt = Regex::optional(Regex::Char('a'));
        assert_eq!(optimize_quantifier(&opt), "optional (?)");
    }
}
```

**Check Your Understanding**:
1. Why is greedy matching important for `*` and `+` quantifiers?
2. How would you implement lazy (non-greedy) quantifiers?
3. What's the time complexity of matching `a*a*a*b` against `"aaa...aac"`?

---

## Milestone 4: Alternation and Capture Groups

**Goal**: Implement alternation `a|b` and capture groups `(a)`, demonstrating nested destructuring.

**Concepts**:
- Or-patterns for alternatives
- Deep destructuring of nested AST
- Capture extraction with lifetimes
- Non-capturing groups `(?:a)`

**Implementation Steps**:

1. **Add `Alternation` and `Group` variants**:
   - `Alternation(Vec<Regex>)` for `a|b|c`
   - `Group { expr: Box<Regex>, id: Option<usize> }`

2. **Implement capture tracking**:
   - Create `Match` struct with captures
   - Thread captures through matching

3. **Parse groups and alternation**:
   - Handle `(...)` and `|`
   - Assign capture IDs sequentially

4. **Extract deep nested captures**:
   - Use recursive destructuring
   - Handle `((a)(b))` patterns

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    CharClass {
        ranges: Vec<CharRange>,
        chars: Vec<char>,
        negated: bool,
    },
    Sequence(Vec<Regex>),
    Repeat {
        expr: Box<Regex>,
        min: usize,
        max: Option<usize>,
    },
    Alternation(Vec<Regex>),
    Group {
        expr: Box<Regex>,
        id: Option<usize>,
    },
}

#[derive(Debug, Clone)]
pub struct Match<'a> {
    pub full_match: &'a str,
    pub captures: Vec<Option<&'a str>>,
}

impl Regex {
    pub fn find<'a>(&self, input: &'a str) -> Option<Match<'a>> {
        for start in 0..=input.len() {
            let mut captures = Vec::new();
            if let Some(end) = self.match_with_captures(input, start, &mut captures) {
                return Some(Match {
                    full_match: &input[start..end],
                    captures,
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
        let chars: Vec<char> = input.chars().collect();

        match self {
            // TODO: Handle Alternation
            Regex::Alternation(alternatives) => {
                // Try each alternative in order
                for alt in alternatives {
                    if let Some(end) = alt.match_with_captures(input, pos, captures) {
                        return Some(end);
                    }
                }
                None
            }

            // TODO: Handle Group
            Regex::Group { expr, id } => {
                let start_pos = pos;
                let end_pos = expr.match_with_captures(input, pos, captures)?;

                // Record capture if this is a capturing group
                if let Some(capture_id) = id {
                    // Ensure captures vec is large enough
                    while captures.len() <= *capture_id {
                        captures.push(None);
                    }
                    captures[*capture_id] = Some(&input[start_pos..end_pos]);
                }

                Some(end_pos)
            }

            // ... other cases (reuse match_at logic)
            _ => {
                // Convert to simple match_at call
                // (In real code, merge both methods or call match_at)
                None
            }
        }
    }
}

// Deep destructuring example
fn extract_all_capture_ids(regex: &Regex) -> Vec<usize> {
    match regex {
        // TODO: Nested destructuring with box patterns
        Regex::Group { id: Some(id), expr } => {
            let mut ids = vec![*id];
            ids.extend(extract_all_capture_ids(expr));
            ids
        }

        // TODO: Sequence destructuring
        Regex::Sequence(exprs) => {
            exprs.iter()
                .flat_map(|e| extract_all_capture_ids(e))
                .collect()
        }

        // TODO: Alternation
        Regex::Alternation(alts) => {
            alts.iter()
                .flat_map(|a| extract_all_capture_ids(a))
                .collect()
        }

        // TODO: Repeat
        Regex::Repeat { expr, .. } => extract_all_capture_ids(expr),

        _ => Vec::new(),
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alternation() {
        let regex = Regex::Alternation(vec![
            Regex::Char('a'),
            Regex::Char('b'),
            Regex::Char('c'),
        ]);

        assert!(regex.find("a").is_some());
        assert!(regex.find("b").is_some());
        assert!(regex.find("c").is_some());
        assert!(regex.find("d").is_none());
    }

    #[test]
    fn test_capture_single_group() {
        let regex = Regex::Group {
            expr: Box::new(Regex::Literal("hello".to_string())),
            id: Some(0),
        };

        let m = regex.find("hello").unwrap();
        assert_eq!(m.captures.get(0), Some(&Some("hello")));
    }

    #[test]
    fn test_multiple_captures() {
        // (a)(b)(c)
        let regex = Regex::Sequence(vec![
            Regex::Group {
                expr: Box::new(Regex::Char('a')),
                id: Some(0),
            },
            Regex::Group {
                expr: Box::new(Regex::Char('b')),
                id: Some(1),
            },
            Regex::Group {
                expr: Box::new(Regex::Char('c')),
                id: Some(2),
            },
        ]);

        let m = regex.find("abc").unwrap();
        assert_eq!(m.captures[0], Some("a"));
        assert_eq!(m.captures[1], Some("b"));
        assert_eq!(m.captures[2], Some("c"));
    }

    #[test]
    fn test_non_capturing_group() {
        let regex = Regex::Group {
            expr: Box::new(Regex::Char('a')),
            id: None,  // Non-capturing
        };

        let m = regex.find("a").unwrap();
        assert!(m.captures.is_empty());
    }

    #[test]
    fn test_extract_capture_ids() {
        let regex = Regex::Sequence(vec![
            Regex::Group {
                expr: Box::new(Regex::Char('a')),
                id: Some(0),
            },
            Regex::Group {
                expr: Box::new(Regex::Char('b')),
                id: Some(1),
            },
        ]);

        let ids = extract_all_capture_ids(&regex);
        assert_eq!(ids, vec![0, 1]);
    }
}
```

**Check Your Understanding**:
1. Why do we use `Option<&'a str>` for captures instead of `String`?
2. How does deep destructuring help extract nested capture groups?
3. What's the difference between capturing and non-capturing groups?

---

## Milestone 5: Anchors, Optimization, and Comprehensive Pattern Matching

**Goal**: Implement anchors `^` `$`, optimize with pattern guards, and demonstrate all pattern matching features.

**Concepts**:
- Anchor matching (start/end positions)
- If-let chains for optimization
- Let-else for early returns
- Matches! macro for pattern testing
- Exhaustive pattern coverage

**Implementation Steps**:

1. **Add `Anchor` variant**:
   - Types: `Start` (^), `End` ($), `WordBound` (\b)
   - Match at specific positions only

2. **Implement optimization**:
   - Use if-let chains to detect patterns
   - Flatten nested sequences
   - Merge adjacent literals

3. **Add comprehensive analysis**:
   - Complexity reporting
   - Pattern description
   - Validation

4. **Demonstrate all match features**:
   - Guards, ranges, destructuring
   - Or-patterns, slice patterns
   - matches! macro

**Starter Code**:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Literal(String),
    Char(char),
    Wildcard,
    CharClass {
        ranges: Vec<CharRange>,
        chars: Vec<char>,
        negated: bool,
    },
    Sequence(Vec<Regex>),
    Repeat {
        expr: Box<Regex>,
        min: usize,
        max: Option<usize>,
    },
    Alternation(Vec<Regex>),
    Group {
        expr: Box<Regex>,
        id: Option<usize>,
    },
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
            // TODO: Handle anchors
            Regex::Anchor(AnchorKind::Start) if pos == 0 => Some(pos),
            Regex::Anchor(AnchorKind::End) if pos == input.len() => Some(pos),
            Regex::Anchor(AnchorKind::WordBound) => {
                if is_word_boundary(input, pos) {
                    Some(pos)
                } else {
                    None
                }
            }
            Regex::Anchor(_) => None,

            // ... other cases
            _ => None,
        }
    }
}

fn is_word_boundary(input: &str, pos: usize) -> bool {
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

// Optimization with pattern matching
pub fn optimize(regex: Regex) -> Regex {
    match regex {
        // TODO: Flatten nested sequences
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

        // TODO: Merge adjacent Char into Literal
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

        // TODO: Simplify single-branch alternation
        Regex::Alternation(mut alts) if alts.len() == 1 => {
            optimize(alts.pop().unwrap())
        }

        // TODO: Recursively optimize nested expressions
        Regex::Repeat { expr, min, max } => Regex::Repeat {
            expr: Box::new(optimize(*expr)),
            min,
            max,
        },

        Regex::Group { expr, id } => Regex::Group {
            expr: Box::new(optimize(*expr)),
            id,
        },

        Regex::Alternation(alts) => {
            Regex::Alternation(alts.into_iter().map(optimize).collect())
        }

        other => other,
    }
}

// Pattern analysis with matches! macro
pub fn is_simple_literal(regex: &Regex) -> bool {
    matches!(regex, Regex::Literal(_) | Regex::Char(_))
}

pub fn has_backtracking(regex: &Regex) -> bool {
    match regex {
        Regex::Repeat { .. } => true,
        Regex::Alternation(_) => true,
        Regex::Sequence(exprs) => exprs.iter().any(has_backtracking),
        Regex::Group { expr, .. } => has_backtracking(expr),
        _ => false,
    }
}

// Comprehensive description with all pattern features
pub fn describe(regex: &Regex) -> String {
    match regex {
        Regex::Literal(s) => format!("literal '{}'", s),
        Regex::Char(c) => format!("char '{}'", c),
        Regex::Wildcard => "any character".to_string(),

        // Pattern guards for specific quantifiers
        Regex::Repeat { min: 0, max: None, expr } => {
            format!("zero or more of ({})", describe(expr))
        }
        Regex::Repeat { min: 1, max: None, expr } => {
            format!("one or more of ({})", describe(expr))
        }
        Regex::Repeat { min: 0, max: Some(1), expr } => {
            format!("optionally ({})", describe(expr))
        }
        Regex::Repeat { min, max: Some(max_val), expr } if min == max_val => {
            format!("exactly {} of ({})", min, describe(expr))
        }
        Regex::Repeat { min, max, expr } => {
            let max_str = max.map_or("∞".to_string(), |m| m.to_string());
            format!("{} to {} of ({})", min, max_str, describe(expr))
        }

        // Slice patterns for sequences
        Regex::Sequence(exprs) => match exprs.as_slice() {
            [] => "empty".to_string(),
            [single] => describe(single),
            [first, rest @ ..] => {
                let first_desc = describe(first);
                let rest_desc: Vec<_> = rest.iter().map(|e| describe(e)).collect();
                format!("{} then {}", first_desc, rest_desc.join(" then "))
            }
        },

        Regex::Alternation(alts) => {
            let descriptions: Vec<_> = alts.iter().map(|a| describe(a)).collect();
            format!("one of: {}", descriptions.join(" | "))
        }

        Regex::Group { expr, id: Some(id) } => {
            format!("capture group {} ({})", id, describe(expr))
        }
        Regex::Group { expr, id: None } => {
            format!("non-capturing group ({})", describe(expr))
        }

        Regex::CharClass { negated: true, .. } => {
            "not in character class".to_string()
        }
        Regex::CharClass { ranges, chars, .. } => {
            format!("one of: {:?} or ranges {:?}", chars, ranges)
        }

        Regex::Anchor(AnchorKind::Start) => "start of string".to_string(),
        Regex::Anchor(AnchorKind::End) => "end of string".to_string(),
        Regex::Anchor(AnchorKind::WordBound) => "word boundary".to_string(),
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_start() {
        let regex = Regex::Sequence(vec![
            Regex::Anchor(AnchorKind::Start),
            Regex::Literal("hello".to_string()),
        ]);

        assert!(regex.find("hello").is_some());
        assert!(regex.find("hello world").is_some());
        assert!(regex.find("say hello").is_none());
    }

    #[test]
    fn test_anchor_end() {
        let regex = Regex::Sequence(vec![
            Regex::Literal("world".to_string()),
            Regex::Anchor(AnchorKind::End),
        ]);

        assert!(regex.find("world").is_some());
        assert!(regex.find("hello world").is_some());
        assert!(regex.find("world!").is_none());
    }

    #[test]
    fn test_exact_match() {
        let regex = Regex::Sequence(vec![
            Regex::Anchor(AnchorKind::Start),
            Regex::Literal("test".to_string()),
            Regex::Anchor(AnchorKind::End),
        ]);

        assert!(regex.find("test").is_some());
        assert!(regex.find("test ").is_none());
        assert!(regex.find(" test").is_none());
    }

    #[test]
    fn test_optimization() {
        // Nested sequences should flatten
        let regex = Regex::Sequence(vec![
            Regex::Sequence(vec![
                Regex::Char('a'),
                Regex::Char('b'),
            ]),
            Regex::Char('c'),
        ]);

        let optimized = optimize(regex);
        // Should become Literal("abc")
        assert!(matches!(optimized, Regex::Literal(s) if s == "abc"));
    }

    #[test]
    fn test_is_simple_literal() {
        assert!(is_simple_literal(&Regex::Literal("test".to_string())));
        assert!(is_simple_literal(&Regex::Char('a')));
        assert!(!is_simple_literal(&Regex::Wildcard));
    }

    #[test]
    fn test_has_backtracking() {
        assert!(has_backtracking(&Regex::zero_or_more(Regex::Char('a'))));
        assert!(has_backtracking(&Regex::Alternation(vec![])));
        assert!(!has_backtracking(&Regex::Char('a')));
    }

    #[test]
    fn test_describe() {
        let regex = Regex::zero_or_more(Regex::Char('a'));
        assert_eq!(describe(&regex), "zero or more of (char 'a')");

        let regex = Regex::Alternation(vec![
            Regex::Char('a'),
            Regex::Char('b'),
        ]);
        assert_eq!(describe(&regex), "one of: char 'a' | char 'b'");
    }
}
```

**Benchmarking**:

```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_simple_literal() {
        let regex = Regex::Literal("hello".to_string());
        let input = "hello world ".repeat(1000);

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = regex.find(&input);
        }
        println!("Simple literal: {:?}", start.elapsed());
    }

    #[test]
    fn bench_quantifier() {
        let regex = Regex::Sequence(vec![
            Regex::zero_or_more(Regex::Char('a')),
            Regex::Char('b'),
        ]);
        let input = "aaaaaaaaab".repeat(100);

        let start = Instant::now();
        for _ in 0..100 {
            let _ = regex.find(&input);
        }
        println!("Quantifier: {:?}", start.elapsed());
    }

    #[test]
    fn bench_alternation() {
        let regex = Regex::Alternation(vec![
            Regex::Literal("foo".to_string()),
            Regex::Literal("bar".to_string()),
            Regex::Literal("baz".to_string()),
        ]);
        let input = "baz ".repeat(1000);

        let start = Instant::now();
        for _ in 0..1000 {
            let _ = regex.find(&input);
        }
        println!("Alternation: {:?}", start.elapsed());
    }
}
```

**Check Your Understanding**:
1. How do anchors differ from other regex patterns in matching?
2. Why is optimization important for nested sequences?
3. When would you use `matches!` vs a full `match` expression?

---

## Summary

You've built a **complete regular expression engine** demonstrating:

**Pattern Matching Features**:
- ✅ **Exhaustive matching** - All enum variants handled
- ✅ **Range patterns** - `'a'..='z'` for character classes
- ✅ **Pattern guards** - `if` conditions in match arms
- ✅ **Deep destructuring** - Nested AST traversal
- ✅ **Or-patterns** - `Literal(_) | Char(_)`
- ✅ **Slice patterns** - `[first, rest @ ..]`
- ✅ **matches! macro** - Pattern testing utility
- ✅ **If-let chains** - Optimization sequences
- ✅ **Let-else** - Early returns

**Regex Features**:
- Literals, wildcards, character classes
- Quantifiers (*, +, ?, {n,m})
- Alternation (|)
- Capture groups
- Anchors (^, $, \b)
- Greedy matching with backtracking

**Real-World Applications**:
- Text editors (find/replace)
- Form validation
- Log parsing
- Web scraping
- Compiler lexing

**Next Steps**:
- Compile to NFA/DFA for O(n) matching
- Add Unicode support
- Implement backreferences
- Add lookahead/lookbehind
- Create visual debugger
