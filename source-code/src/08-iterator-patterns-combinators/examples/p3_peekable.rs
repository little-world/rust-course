//! Pattern 3: Advanced Iterator Composition
//! Example: Peekable for Lookahead
//!
//! Run with: cargo run --example p3_peekable

#[derive(Debug, PartialEq)]
enum Token {
    Number(i32),
    Op(char),
}

/// Parse a simple expression into tokens using peekable for lookahead.
fn parse_tokens(input: &str) -> Vec<Token> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&ch) = chars.peek() {
        match ch {
            '0'..='9' => {
                // Collect consecutive digits
                let num: String = std::iter::from_fn(|| {
                    match chars.peek() {
                        Some(c) if c.is_ascii_digit() => chars.next(),
                        _ => None,
                    }
                })
                .collect();
                tokens.push(Token::Number(num.parse().unwrap()));
            }
            '+' | '-' | '*' | '/' => {
                tokens.push(Token::Op(ch));
                chars.next();
            }
            ' ' => {
                chars.next(); // Skip whitespace
            }
            _ => {
                chars.next(); // Skip unknown characters
            }
        }
    }

    tokens
}

/// Parse identifiers and numbers with lookahead.
#[derive(Debug)]
enum Value {
    Ident(String),
    Int(i32),
}

fn parse_values(input: &str) -> Vec<Value> {
    let mut chars = input.chars().peekable();
    let mut values = Vec::new();

    while let Some(&ch) = chars.peek() {
        if ch.is_alphabetic() {
            // Collect identifier
            let ident: String = std::iter::from_fn(|| {
                match chars.peek() {
                    Some(c) if c.is_alphanumeric() || *c == '_' => chars.next(),
                    _ => None,
                }
            })
            .collect();
            values.push(Value::Ident(ident));
        } else if ch.is_ascii_digit() {
            // Collect number
            let num: String = std::iter::from_fn(|| {
                match chars.peek() {
                    Some(c) if c.is_ascii_digit() => chars.next(),
                    _ => None,
                }
            })
            .collect();
            values.push(Value::Int(num.parse().unwrap()));
        } else {
            chars.next(); // Skip
        }
    }

    values
}

/// Merge consecutive duplicates.
fn merge_duplicates<T: Clone + PartialEq>(items: Vec<T>) -> Vec<(T, usize)> {
    let mut iter = items.into_iter().peekable();
    let mut result = Vec::new();

    while let Some(item) = iter.next() {
        let mut count = 1;
        while iter.peek() == Some(&item) {
            iter.next();
            count += 1;
        }
        result.push((item, count));
    }

    result
}

fn main() {
    println!("=== Peekable for Lookahead ===\n");

    // Parse simple arithmetic expression
    let tokens = parse_tokens("123 + 456 * 789");
    println!("parse_tokens('123 + 456 * 789'):");
    for token in &tokens {
        println!("  {:?}", token);
    }

    println!("\n=== How peek() Works ===");
    let mut iter = [1, 2, 3].iter().peekable();
    println!("iter.peek() = {:?} (doesn't advance)", iter.peek());
    println!("iter.peek() = {:?} (still same)", iter.peek());
    println!("iter.next() = {:?} (advances)", iter.next());
    println!("iter.peek() = {:?} (now at next element)", iter.peek());

    println!("\n=== Parsing Identifiers and Numbers ===");
    let values = parse_values("foo 123 bar_baz 456 qux789");
    println!("parse_values('foo 123 bar_baz 456 qux789'):");
    for val in &values {
        println!("  {:?}", val);
    }

    println!("\n=== Merge Consecutive Duplicates ===");
    let input = vec!['a', 'a', 'a', 'b', 'b', 'c', 'd', 'd', 'd', 'd'];
    let merged = merge_duplicates(input.clone());
    println!("Input: {:?}", input);
    println!("Merged: {:?}", merged);
    // [('a', 3), ('b', 2), ('c', 1), ('d', 4)]

    println!("\n=== peek_mut() for Modification ===");
    let mut iter = vec![1, 2, 3].into_iter().peekable();
    if let Some(v) = iter.peek_mut() {
        *v *= 10;
    }
    let result: Vec<_> = iter.collect();
    println!("After peek_mut (first * 10): {:?}", result);

    println!("\n=== Key Points ===");
    println!("1. peekable() wraps iterator to add lookahead");
    println!("2. peek() returns Option<&Item> without advancing");
    println!("3. Essential for parsers needing to look at next token");
    println!("4. peek_mut() allows modifying the peeked value");
}
