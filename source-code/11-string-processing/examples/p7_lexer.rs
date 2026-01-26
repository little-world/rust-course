//! Pattern 7: String Parsing State Machines
//! Lexer Implementation
//!
//! Run with: cargo run --example p7_lexer

fn main() {
    println!("=== Lexer State Machine ===\n");

    let code = r#"
        fn main() {
            let x = 42;
            if x == 42 {
                return x + 10;
            }
        }
        // This is a comment
    "#;

    println!("Source code:");
    println!("{}", code);
    println!("\nTokens:");

    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize();

    for token in tokens {
        println!("  {:?}", token);
    }

    println!("\n=== Key Points ===");
    println!("1. State machine with explicit states");
    println!("2. Lookahead with peek() for multi-char tokens");
    println!("3. Keyword vs identifier discrimination");
    println!("4. Each state handles specific characters");
}

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
