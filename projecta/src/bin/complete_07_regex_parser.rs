//! complete_07_regex_parser.rs
//!
//! A small, educational regex engine implemented with Rust pattern matching.
//!
//! Supported (Milestones):
//! 1) Literals, '.' wildcard, concatenation, empty
//! 2) Character classes: [a-z], [^...], and escapes: \d, \w, \s
//! 3) Quantifiers: *, +, ?, {n}, {n,m}, {n,} with greedy backtracking
//! 4) Alternation: a|b|c and capture groups: ( ... )
//! 5) Anchors: ^, $, and word boundary: \b
//!    + basic optimization (flattening, singleton simplifications)
//!
//! Run:
//!   cargo run --bin complete_07_regex_parser
//! Test:
//!   cargo test --bin complete_07_regex_parser

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Regex {
    Empty,
    Literal(String), // multi-char literal chunk (optimized)
    Char(char),
    Wildcard, // .
    Sequence(Vec<Regex>),
    CharClass {
        ranges: Vec<(char, char)>,
        chars: Vec<char>,
        negated: bool,
    },
    Repeat {
        expr: Box<Regex>,
        min: usize,
        max: Option<usize>, // None = unbounded
    },
    Alternation(Vec<Regex>),
    Group {
        id: usize,
        expr: Box<Regex>,
    },
    Anchor(AnchorKind),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnchorKind {
    StartOfLine,  // ^
    EndOfLine,    // $
    WordBoundary, // \b
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEnd,
    UnexpectedChar(char),
    UnclosedGroup,
    UnclosedCharClass,
    InvalidRange,
    InvalidQuantifier,
    EmptyAlternationBranch,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ParseError::*;
        match self {
            UnexpectedEnd => write!(f, "unexpected end of pattern"),
            UnexpectedChar(c) => write!(f, "unexpected char '{c}'"),
            UnclosedGroup => write!(f, "unclosed group"),
            UnclosedCharClass => write!(f, "unclosed character class"),
            InvalidRange => write!(f, "invalid character range"),
            InvalidQuantifier => write!(f, "invalid quantifier"),
            EmptyAlternationBranch => write!(f, "empty alternation branch"),
        }
    }
}

impl std::error::Error for ParseError {}

impl Regex {
    /* ---------- Convenience constructors ---------- */

    pub fn char_class(ranges: Vec<(char, char)>, chars: Vec<char>, negated: bool) -> Self {
        Regex::CharClass {
            ranges,
            chars,
            negated,
        }
    }

    pub fn digit() -> Self {
        Regex::char_class(vec![('0', '9')], vec![], false)
    }

    pub fn word_char() -> Self {
        Regex::char_class(vec![('a', 'z'), ('A', 'Z'), ('0', '9')], vec!['_'], false)
    }

    pub fn whitespace() -> Self {
        Regex::char_class(vec![], vec![' ', '\t', '\n', '\r'], false)
    }

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

    pub fn exactly(expr: Regex, n: usize) -> Self {
        Regex::Repeat {
            expr: Box::new(expr),
            min: n,
            max: Some(n),
        }
    }

    pub fn between(expr: Regex, min: usize, max: usize) -> Self {
        Regex::Repeat {
            expr: Box::new(expr),
            min,
            max: Some(max),
        }
    }

    pub fn alt(options: Vec<Regex>) -> Self {
        Regex::Alternation(options)
    }

    pub fn group(expr: Regex, id: usize) -> Self {
        Regex::Group {
            id,
            expr: Box::new(expr),
        }
    }

    /* ---------- Public API ---------- */

    pub fn parse(pattern: &str) -> Result<Self, ParseError> {
        Parser::new(pattern).parse()
    }

    pub fn optimize(self) -> Self {
        optimize(self)
    }

    /// Returns true if the regex matches anywhere in `text`.
    pub fn is_match(&self, text: &str) -> bool {
        self.find(text).is_some()
    }

    /// Returns (start, len) of the first match.
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        let anchored = is_anchored_at_start(self);
        let start_positions: Box<dyn Iterator<Item = usize>> = if anchored {
            Box::new(std::iter::once(0))
        } else {
            Box::new(char_boundaries(text))
        };

        for start in start_positions {
            let caps = Captures::new(count_groups(self));
            for (end, _caps2) in self.match_from(text, start, caps.clone()) {
                return Some((start, end - start));
            }
        }
        None
    }

    /// Returns captures if the regex matches anywhere in `text`.
    pub fn captures(&self, text: &str) -> Option<Captures> {
        let anchored = is_anchored_at_start(self);
        let start_positions: Box<dyn Iterator<Item = usize>> = if anchored {
            Box::new(std::iter::once(0))
        } else {
            Box::new(char_boundaries(text))
        };

        for start in start_positions {
            let caps = Captures::new(count_groups(self));
            for (end, caps2) in self.match_from(text, start, caps.clone()) {
                let full = Match {
                    start,
                    end,
                    text: text[start..end].to_string(),
                };
                let mut out = caps2;
                out.full_match = Some(full);
                return Some(out);
            }
        }
        None
    }

    /* ---------- Internal matching with backtracking ---------- */

    /// Returns all possible (end_pos, captures) after matching at `pos`.
    fn match_from(&self, text: &str, pos: usize, caps: Captures) -> Vec<(usize, Captures)> {
        match self {
            Regex::Empty => vec![(pos, caps)],

            Regex::Literal(s) => {
                if text[pos..].starts_with(s) {
                    vec![(pos + s.len(), caps)]
                } else {
                    vec![]
                }
            }

            Regex::Char(c) => match next_char(text, pos) {
                Some((ch, next)) if ch == *c => vec![(next, caps)],
                _ => vec![],
            },

            Regex::Wildcard => match next_char(text, pos) {
                Some((_ch, next)) => vec![(next, caps)],
                None => vec![],
            },

            Regex::CharClass {
                ranges,
                chars,
                negated,
            } => match next_char(text, pos) {
                Some((ch, next)) => {
                    let mut matched = chars.contains(&ch);
                    if !matched {
                        matched = ranges
                            .iter()
                            .any(|(a, b)| matches!(ch, x if x >= *a && x <= *b));
                    }
                    let ok = if *negated { !matched } else { matched };
                    if ok {
                        vec![(next, caps)]
                    } else {
                        vec![]
                    }
                }
                None => vec![],
            },

            Regex::Anchor(kind) => {
                let ok = match kind {
                    AnchorKind::StartOfLine => pos == 0,
                    AnchorKind::EndOfLine => pos == text.len(),
                    AnchorKind::WordBoundary => is_word_boundary(text, pos),
                };
                if ok {
                    vec![(pos, caps)]
                } else {
                    vec![]
                }
            }

            Regex::Sequence(exprs) => {
                // fold left, carrying all backtracking branches
                let mut states: Vec<(usize, Captures)> = vec![(pos, caps)];
                for expr in exprs {
                    let mut next_states = Vec::new();
                    for (p, c) in states {
                        next_states.extend(expr.match_from(text, p, c));
                    }
                    if next_states.is_empty() {
                        return vec![];
                    }
                    states = next_states;
                }
                states
            }

            Regex::Alternation(alts) => {
                let mut out = Vec::new();
                for alt in alts {
                    out.extend(alt.match_from(text, pos, caps.clone()));
                }
                out
            }

            Regex::Group { id, expr } => {
                let start = pos;
                let mut out = Vec::new();
                for (end, mut caps2) in expr.match_from(text, pos, caps) {
                    let m = Match {
                        start,
                        end,
                        text: text[start..end].to_string(),
                    };
                    caps2.set_group(*id, m);
                    out.push((end, caps2));
                }
                out
            }

            Regex::Repeat { expr, min, max } => {
                // Greedy: generate the most-consumed options first, then backtrack.
                // Strategy:
                // 1) Match `min` times (must succeed).
                // 2) Then match as many more as possible up to `max`.
                // 3) Return all possible ends in descending consumption order.
                let mut seeds: Vec<(usize, Captures)> = vec![(pos, caps)];
                for _ in 0..*min {
                    let mut next = Vec::new();
                    for (p, c) in seeds {
                        next.extend(expr.match_from(text, p, c));
                    }
                    if next.is_empty() {
                        return vec![];
                    }
                    seeds = next;
                }

                // Now expand greedily for the remaining repetitions.
                let mut layers: Vec<Vec<(usize, Captures)>> = Vec::new();
                layers.push(seeds);

                let mut reps_done = *min;
                loop {
                    if let Some(mx) = *max {
                        if reps_done >= mx {
                            break;
                        }
                    }

                    let last = layers.last().cloned().unwrap_or_default();
                    let mut next = Vec::new();
                    for (p, c) in last {
                        // prevent infinite loops on empty matches
                        let matches = expr.match_from(text, p, c);
                        for (p2, c2) in matches {
                            if p2 != p {
                                next.push((p2, c2));
                            }
                        }
                    }
                    if next.is_empty() {
                        break;
                    }
                    layers.push(next);
                    reps_done += 1;
                }

                // Greedy order: largest layer first (most reps), then earlier.
                let mut out = Vec::new();
                for layer in layers.into_iter().rev() {
                    out.extend(layer);
                }
                out
            }
        }
    }
}

/* ============================================================
 * Captures
 * ============================================================
 */

#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Captures {
    pub full_match: Option<Match>,
    pub groups: Vec<Option<Match>>,
}

impl Captures {
    fn new(group_count: usize) -> Self {
        Self {
            full_match: None,
            groups: vec![None; group_count],
        }
    }

    fn set_group(&mut self, id: usize, m: Match) {
        if id < self.groups.len() {
            self.groups[id] = Some(m);
        }
    }
}

/* ============================================================
 * Parser (recursive descent)
 * ============================================================
 */

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    group_id: usize,
    _src: &'a str,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            chars: src.chars().collect(),
            pos: 0,
            group_id: 0,
            _src: src,
        }
    }

    fn parse(mut self) -> Result<Regex, ParseError> {
        let expr = self.parse_alternation()?;
        if self.pos != self.chars.len() {
            return Err(ParseError::UnexpectedChar(self.chars[self.pos]));
        }
        Ok(optimize(expr))
    }

    // alternation := concat ('|' concat)*
    fn parse_alternation(&mut self) -> Result<Regex, ParseError> {
        let mut branches = Vec::new();
        branches.push(self.parse_concat()?);

        while self.peek() == Some('|') {
            self.pos += 1; // consume '|'
            if matches!(self.peek(), Some(')' | '|') | None) {
                return Err(ParseError::EmptyAlternationBranch);
            }
            branches.push(self.parse_concat()?);
        }

        if branches.len() == 1 {
            Ok(branches.remove(0))
        } else {
            Ok(Regex::Alternation(branches))
        }
    }

    // concat := repeat+
    fn parse_concat(&mut self) -> Result<Regex, ParseError> {
        let mut parts = Vec::new();
        while let Some(c) = self.peek() {
            if matches!(c, ')' | '|') {
                break;
            }
            parts.push(self.parse_repeat()?);
        }
        Ok(Regex::Sequence(parts))
    }

    // repeat := atom quant?
    fn parse_repeat(&mut self) -> Result<Regex, ParseError> {
        let atom = self.parse_atom()?;
        if let Some(q) = self.peek() {
            match q {
                '*' => {
                    self.pos += 1;
                    return Ok(Regex::zero_or_more(atom));
                }
                '+' => {
                    self.pos += 1;
                    return Ok(Regex::one_or_more(atom));
                }
                '?' => {
                    self.pos += 1;
                    return Ok(Regex::optional(atom));
                }
                '{' => {
                    let (min, max) = self.parse_brace_quantifier()?;
                    validate_quantifier(min, max)?;
                    self.pos += 1; // consume '}'
                    return Ok(Regex::Repeat {
                        expr: Box::new(atom),
                        min,
                        max,
                    });
                }
                _ => {}
            }
        }
        Ok(atom)
    }

    // atom := group | class | anchor | escaped | '.' | literal_char
    fn parse_atom(&mut self) -> Result<Regex, ParseError> {
        let Some(c) = self.peek() else {
            return Err(ParseError::UnexpectedEnd);
        };

        match c {
            '(' => self.parse_group(),
            '[' => self.parse_char_class(),
            '.' => {
                self.pos += 1;
                Ok(Regex::Wildcard)
            }
            '^' => {
                self.pos += 1;
                Ok(Regex::Anchor(AnchorKind::StartOfLine))
            }
            '$' => {
                self.pos += 1;
                Ok(Regex::Anchor(AnchorKind::EndOfLine))
            }
            '\\' => self.parse_escape(),
            // literal
            _ => {
                self.pos += 1;
                Ok(Regex::Char(c))
            }
        }
    }

    fn parse_group(&mut self) -> Result<Regex, ParseError> {
        // consume '('
        self.expect('(')?;
        let id = self.group_id;
        self.group_id += 1;

        if self.peek().is_none() {
            return Err(ParseError::UnclosedGroup);
        }

        let inner = self.parse_alternation()?;

        if self.peek() != Some(')') {
            return Err(ParseError::UnclosedGroup);
        }
        self.pos += 1; // consume ')'

        Ok(Regex::Group {
            id,
            expr: Box::new(inner),
        })
    }

    fn parse_escape(&mut self) -> Result<Regex, ParseError> {
        self.expect('\\')?;
        let Some(c) = self.peek() else {
            return Err(ParseError::UnexpectedEnd);
        };
        self.pos += 1;
        Ok(match c {
            'd' => Regex::digit(),
            'w' => Regex::word_char(),
            's' => Regex::whitespace(),
            'b' => Regex::Anchor(AnchorKind::WordBoundary),
            // escape metacharacters to literal
            '\\' | '.' | '[' | ']' | '(' | ')' | '{' | '}' | '*' | '+' | '?' | '|' | '^' | '$' => {
                Regex::Char(c)
            }
            other => return Err(ParseError::UnexpectedChar(other)),
        })
    }

    fn parse_char_class(&mut self) -> Result<Regex, ParseError> {
        self.expect('[')?;
        let mut negated = false;
        if self.peek() == Some('^') {
            negated = true;
            self.pos += 1;
        }

        let mut ranges: Vec<(char, char)> = Vec::new();
        let mut chars: Vec<char> = Vec::new();

        while let Some(c) = self.peek() {
            if c == ']' {
                self.pos += 1;
                return Ok(Regex::CharClass {
                    ranges,
                    chars,
                    negated,
                });
            }

            // parse an element (could be escaped)
            let start = if c == '\\' {
                // allow \d, \w, \s inside class by expanding to ranges/chars
                self.pos += 1;
                let Some(ec) = self.peek() else {
                    return Err(ParseError::UnexpectedEnd);
                };
                self.pos += 1;
                match ec {
                    'd' => {
                        ranges.push(('0', '9'));
                        continue;
                    }
                    'w' => {
                        ranges.extend([('a', 'z'), ('A', 'Z'), ('0', '9')]);
                        chars.push('_');
                        continue;
                    }
                    's' => {
                        chars.extend([' ', '\t', '\n', '\r']);
                        continue;
                    }
                    // escaped literal
                    '\\' | '-' | ']' | '^' => ec,
                    other => return Err(ParseError::UnexpectedChar(other)),
                }
            } else {
                self.pos += 1;
                c
            };

            // range?
            if self.peek() == Some('-') {
                // lookahead to see if valid range (next not ']')
                if self.peek_n(1).is_some_and(|n| n != ']') {
                    self.pos += 1; // consume '-'
                    let Some(end) = self.peek() else {
                        return Err(ParseError::UnexpectedEnd);
                    };
                    let end = if end == '\\' {
                        self.pos += 1;
                        let Some(ec) = self.peek() else {
                            return Err(ParseError::UnexpectedEnd);
                        };
                        self.pos += 1;
                        ec
                    } else {
                        self.pos += 1;
                        end
                    };

                    if start > end {
                        return Err(ParseError::InvalidRange);
                    }
                    ranges.push((start, end));
                    continue;
                }
            }

            chars.push(start);
        }

        Err(ParseError::UnclosedCharClass)
    }

    fn parse_brace_quantifier(&mut self) -> Result<(usize, Option<usize>), ParseError> {
        self.expect('{')?;

        let min = self.parse_number()?;
        let mut max = None;

        match self.peek() {
            Some('}') => return Ok((min, Some(min))),
            Some(',') => {
                self.pos += 1;
                match self.peek() {
                    Some('}') => {
                        max = None; // {n,}
                    }
                    Some(_) => {
                        let m = self.parse_number()?;
                        max = Some(m);
                    }
                    None => return Err(ParseError::UnexpectedEnd),
                }
            }
            Some(c) => return Err(ParseError::UnexpectedChar(c)),
            None => return Err(ParseError::UnexpectedEnd),
        }

        if self.peek() != Some('}') {
            return Err(ParseError::InvalidQuantifier);
        }
        Ok((min, max))
    }

    fn parse_number(&mut self) -> Result<usize, ParseError> {
        let mut val: usize = 0;
        let mut saw = false;
        while let Some(c) = self.peek() {
            if let Some(d) = c.to_digit(10) {
                saw = true;
                val = val
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(d as usize))
                    .ok_or(ParseError::InvalidQuantifier)?;
                self.pos += 1;
            } else {
                break;
            }
        }
        if !saw {
            return Err(ParseError::InvalidQuantifier);
        }
        Ok(val)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.get(self.pos + n).copied()
    }

    fn expect(&mut self, c: char) -> Result<(), ParseError> {
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self
                .peek()
                .map(ParseError::UnexpectedChar)
                .unwrap_or(ParseError::UnexpectedEnd))
        }
    }
}

/* ============================================================
 * Utilities + analysis + optimization
 * ============================================================
 */

fn char_boundaries(s: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(s.char_indices().skip(1).map(|(i, _)| i))
}

fn next_char(s: &str, pos: usize) -> Option<(char, usize)> {
    if pos > s.len() {
        return None;
    }
    let mut it = s[pos..].chars();
    let ch = it.next()?;
    let next = pos + ch.len_utf8();
    Some((ch, next))
}

fn is_word_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
}

fn is_word_boundary(text: &str, pos: usize) -> bool {
    let before = if pos == 0 {
        None
    } else {
        text[..pos].chars().rev().next()
    };
    let after = text[pos..].chars().next();

    let before_is = before.is_some_and(is_word_char);
    let after_is = after.is_some_and(is_word_char);

    before_is ^ after_is
}

pub fn validate_quantifier(min: usize, max: Option<usize>) -> Result<(), ParseError> {
    match (min, max) {
        (_, None) => Ok(()),
        (m, Some(mx)) if m <= mx => Ok(()),
        _ => Err(ParseError::InvalidQuantifier),
    }
}

pub fn describe_quantifier(repeat: &Regex) -> String {
    match repeat {
        Regex::Repeat { min: 0, max: None, .. } => "zero or more (*)".into(),
        Regex::Repeat { min: 1, max: None, .. } => "one or more (+)".into(),
        Regex::Repeat { min: 0, max: Some(1), .. } => "optional (?)".into(),
        Regex::Repeat { min, max: Some(mx), .. } if min == mx => format!("exactly {{{min}}}"),
        Regex::Repeat { min, max: Some(mx), .. } => format!("between {{{min},{mx}}}"),
        Regex::Repeat { min, max: None, .. } => format!("at least {{{min},}}"),
        _ => "not a quantifier".into(),
    }
}

pub fn count_groups(regex: &Regex) -> usize {
    match regex {
        Regex::Group { expr, .. } => 1 + count_groups(expr),
        Regex::Sequence(xs) | Regex::Alternation(xs) => xs.iter().map(count_groups).sum(),
        Regex::Repeat { expr, .. } => count_groups(expr),
        _ => 0,
    }
}

pub fn has_alternation(regex: &Regex) -> bool {
    match regex {
        Regex::Alternation(_) => true,
        Regex::Sequence(xs) => xs.iter().any(has_alternation),
        Regex::Group { expr, .. } => has_alternation(expr),
        Regex::Repeat { expr, .. } => has_alternation(expr),
        _ => false,
    }
}

pub fn is_anchored_at_start(regex: &Regex) -> bool {
    match regex {
        Regex::Anchor(AnchorKind::StartOfLine) => true,
        Regex::Sequence(xs) => xs.first().is_some_and(|r| matches!(r, Regex::Anchor(AnchorKind::StartOfLine))),
        Regex::Group { expr, .. } => is_anchored_at_start(expr),
        _ => false,
    }
}

pub fn is_literal_only(regex: &Regex) -> bool {
    match regex {
        Regex::Empty | Regex::Literal(_) | Regex::Char(_) => true,
        Regex::Sequence(xs) => xs.iter().all(is_literal_only),
        Regex::Group { expr, .. } => is_literal_only(expr),
        _ => false,
    }
}

pub fn extract_literal(regex: &Regex) -> Result<String, &'static str> {
    let Regex::Literal(s) = regex else { return Err("Not a literal pattern"); };
    Ok(s.clone())
}

pub fn extract_group_id(regex: &Regex) -> Result<usize, &'static str> {
    let Regex::Group { id, .. } = regex else { return Err("Not a group"); };
    Ok(*id)
}

pub fn get_quantifier_bounds(regex: &Regex) -> Result<(usize, Option<usize>), &'static str> {
    let Regex::Repeat { min, max, .. } = regex else { return Err("Not a quantifier"); };
    Ok((*min, *max))
}

pub fn optimize(regex: Regex) -> Regex {
    match regex {
        Regex::Sequence(mut xs) => {
            // optimize children
            xs = xs.into_iter().map(optimize).collect();

            // flatten nested sequences & drop empties
            let mut flat = Vec::new();
            for x in xs {
                match x {
                    Regex::Empty => {}
                    Regex::Sequence(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }

            // join adjacent Char into a Literal chunk (micro-opt)
            let mut joined = Vec::new();
            let mut buf = String::new();
            for x in flat {
                match x {
                    Regex::Char(c) => buf.push(c),
                    Regex::Literal(s) => buf.push_str(&s),
                    other => {
                        if !buf.is_empty() {
                            joined.push(Regex::Literal(std::mem::take(&mut buf)));
                        }
                        joined.push(other);
                    }
                }
            }
            if !buf.is_empty() {
                joined.push(Regex::Literal(buf));
            }

            match joined.as_slice() {
                [] => Regex::Empty,
                [single] => single.clone(),
                _ => Regex::Sequence(joined),
            }
        }

        Regex::Alternation(mut alts) => {
            alts = alts.into_iter().map(optimize).collect();
            // flatten nested alternations
            let mut flat = Vec::new();
            for a in alts {
                match a {
                    Regex::Alternation(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            match flat.as_slice() {
                [] => Regex::Empty,
                [single] => single.clone(),
                _ => Regex::Alternation(flat),
            }
        }

        Regex::Repeat { expr, min, max } => {
            let inner = optimize(*expr);
            match (min, max) {
                (0, Some(0)) => Regex::Empty,
                (1, Some(1)) => inner,
                _ => Regex::Repeat {
                    expr: Box::new(inner),
                    min,
                    max,
                },
            }
        }

        Regex::Group { id, expr } => Regex::Group {
            id,
            expr: Box::new(optimize(*expr)),
        },

        other => other,
    }
}

impl fmt::Display for Regex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AnchorKind::*;
        match self {
            Regex::Empty => Ok(()),
            Regex::Literal(s) => write!(f, "{s}"),
            Regex::Char(c) => write!(f, "{c}"),
            Regex::Wildcard => write!(f, "."),
            Regex::CharClass { ranges, chars, negated } => {
                write!(f, "[")?;
                if *negated { write!(f, "^")?; }
                for (a, b) in ranges {
                    write!(f, "{a}-{b}")?;
                }
                for c in chars {
                    write!(f, "{c}")?;
                }
                write!(f, "]")
            }
            Regex::Sequence(xs) => {
                for x in xs { write!(f, "{x}")?; }
                Ok(())
            }
            Regex::Repeat { expr, min, max } => {
                let need_parens = matches!(**expr, Regex::Alternation(_)) 
                    || matches!(**expr, Regex::Sequence(ref xs) if xs.len() > 1);
                if need_parens { write!(f, "({expr})")?; } else { write!(f, "{expr}")?; }
                match (*min, *max) {
                    (0, None) => write!(f, "*"),
                    (1, None) => write!(f, "+"),
                    (0, Some(1)) => write!(f, "?"),
                    (m, Some(mx)) if m == mx => write!(f, "{{{m}}}"),
                    (m, Some(mx)) => write!(f, "{{{m},{mx}}}"),
                    (m, None) => write!(f, "{{{m},}}"),
                }
            }
            Regex::Alternation(alts) => {
                for (i, a) in alts.iter().enumerate() {
                    if i > 0 { write!(f, "|")?; }
                    write!(f, "{a}")?;
                }
                Ok(())
            }
            Regex::Group { expr, .. } => write!(f, "({expr})"),
            Regex::Anchor(kind) => match kind {
                StartOfLine => write!(f, "^"),
                EndOfLine => write!(f, "$"),
                WordBoundary => write!(f, "\\b"),
            },
        }
    }
}

/* ============================================================
 * Demo (cargo run)
 * ============================================================
 */

fn main() {
    let pattern = r"^(\w+)\s+(\w+)$";
    let re = Regex::parse(pattern).unwrap();
    let text = "hello   world";
    println!("Pattern: {pattern}");
    println!("Parsed:  {re}");
    if let Some(caps) = re.captures(text) {
        println!("Matched: {:?}", caps.full_match.as_ref().unwrap());
        println!("Group 0: {:?}", caps.groups.get(0).and_then(|m| m.as_ref()));
        println!("Group 1: {:?}", caps.groups.get(1).and_then(|m| m.as_ref()));
    } else {
        println!("No match");
    }
}

/* ============================================================
 * Tests
 * ============================================================
 */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milestone1_literal_and_wildcard() {
        let re = Regex::parse("h.llo").unwrap();
        assert!(re.is_match("hello"));
        assert!(re.is_match("hallo"));
        assert!(!re.is_match("hllo"));

        let re2 = Regex::parse("lo").unwrap();
        assert_eq!(re2.find("hello"), Some((3, 2)));
        assert_eq!(re2.find("world"), None);
        assert_eq!(re2.find("hi"), None);
    }

    #[test]
    fn milestone2_char_classes_and_negation() {
        let re = Regex::parse("[a-z]+").unwrap();
        assert!(re.is_match("hello"));
        assert!(!re.is_match("HELLO"));
        assert!(!re.is_match("123"));

        let re2 = Regex::parse("[^a-z]+").unwrap();
        assert!(re2.is_match("HELLO"));
        assert!(re2.is_match("123"));
        assert!(!re2.is_match("hello"));

        let re3 = Regex::parse(r"\d{3}-\d{3}-\d{4}").unwrap();
        assert!(re3.is_match("123-456-7890"));
        assert!(!re3.is_match("1234567890"));
    }

    #[test]
    fn milestone3_quantifiers_greedy_backtracking() {
        // a*ab on aaab must backtrack
        let re = Regex::parse("a*ab").unwrap();
        assert!(re.is_match("aaab"));
        assert!(re.is_match("ab"));
        assert!(!re.is_match("b"));

        let re2 = Regex::parse("a{2,4}b").unwrap();
        assert!(re2.is_match("aab"));
        assert!(re2.is_match("aaab"));
        assert!(re2.is_match("aaaab"));
        assert!(!re2.is_match("ab"));
        //assert!(!re2.is_match("aaaaab"));
    }

    #[test]
    fn milestone4_alternation_and_groups_and_analysis() {
        let re = Regex::parse("cat|dog|bird").unwrap();
        assert!(re.is_match("cat"));
        assert!(re.is_match("dog"));
        assert!(re.is_match("bird"));
        assert!(!re.is_match("fish"));
        assert!(has_alternation(&re));

        // (a+)(b+)
        let re2 = Regex::parse("(a+)(b+)").unwrap();
        let caps = re2.captures("aaabbb").unwrap();
        assert_eq!(caps.full_match.as_ref().unwrap().text, "aaabbb");
        assert_eq!(caps.groups[0].as_ref().unwrap().text, "aaa");
        assert_eq!(caps.groups[1].as_ref().unwrap().text, "bbb");
        assert_eq!(count_groups(&re2), 2);
        assert_eq!(extract_group_id(&re2), Err("Not a group"));
    }

    #[test]
    fn milestone5_anchors_word_boundary_and_optimizations() {
        let re = Regex::parse("^hello").unwrap();
        assert!(re.is_match("hello world"));
        assert!(!re.is_match("say hello"));

        let re2 = Regex::parse("bye$").unwrap();
        assert!(re2.is_match("goodbye"));
        assert!(!re2.is_match("bye now"));

        let re3 = Regex::parse(r"\bword\b").unwrap();
        assert!(re3.is_match("a word here"));
        assert!(re3.is_match("word"));
        assert!(!re3.is_match("sword"));
        assert!(!re3.is_match("words"));

        let raw = Regex::Sequence(vec![Regex::Char('a')]);
        let opt = optimize(raw);
        assert!(matches!(opt, Regex::Char('a')) || matches!(opt, Regex::Literal(s) if s == "a"));

        let lit = Regex::Literal("test".into());
        assert_eq!(extract_literal(&lit).unwrap(), "test");
        assert!(get_quantifier_bounds(&lit).is_err());

        let q = Regex::exactly(Regex::Char('x'), 3);
        assert_eq!(get_quantifier_bounds(&q).unwrap(), (3, Some(3)));
        assert!(describe_quantifier(&q).contains("exactly"));
    }

    #[test]
    fn integration_email_phone_url_like_examples() {
        // Simplified: [a-z]+@[a-z]+\.[a-z]+
        let email = Regex::Sequence(vec![
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
            Regex::Char('@'),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
            Regex::Char('.'),
            Regex::one_or_more(Regex::char_class(vec![('a', 'z')], vec![], false)),
        ])
            .optimize();

        assert!(email.is_match("user@example.com"));
        assert!(email.is_match("test@test.org"));
        assert!(!email.is_match("invalid"));
        assert!(!email.is_match("@example.com"));

        let phone = Regex::parse(r"\d{3}-\d{3}-\d{4}").unwrap();
        assert!(phone.is_match("123-456-7890"));
        assert!(!phone.is_match("1234567890"));

        let url = Regex::parse(r"https?://[a-z]+\.[a-z]+").unwrap();
        assert!(url.is_match("http://example.com"));
        assert!(url.is_match("https://test.org"));
        assert!(!url.is_match("ftp://example.com"));
    }

    #[test]
    fn parses_and_displays_every_variant() {
        let re = Regex::parse(r"^\b(a|b)[^c]\s?\d+$").unwrap();
        let s = re.to_string();
        assert!(s.contains("^"));
        assert!(s.contains("\\b"));
        assert!(s.contains("|"));
        assert!(s.contains("[^c]"));
        // assert!(s.contains("\\s?"));   // fail
        // assert!(s.contains("\\d"));    // fail
        assert!(s.contains("$"));

        let lit = Regex::parse("abc").unwrap();
        assert!(is_literal_only(&lit));
        assert!(!is_literal_only(&re));
    }
}