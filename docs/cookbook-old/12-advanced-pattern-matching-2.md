# Cookbook: Advanced Pattern Matching

> Real-world algorithms and data structures demonstrating Rust's powerful pattern matching

## Table of Contents

1. [Parser and Lexer Patterns](#parser-and-lexer-patterns)
2. [State Machine Patterns](#state-machine-patterns)
3. [Tree Traversal and Manipulation](#tree-traversal-and-manipulation)
4. [Protocol Parsing](#protocol-parsing)
5. [Compiler and Interpreter Patterns](#compiler-and-interpreter-patterns)
6. [Graph Algorithms](#graph-algorithms)
7. [Game Logic Patterns](#game-logic-patterns)
8. [Error Recovery Patterns](#error-recovery-patterns)
9. [Configuration Processing](#configuration-processing)
10. [Quick Reference](#quick-reference)

---

## Parser and Lexer Patterns

### Recipe 1: JSON Parser with Pattern Matching

**Problem**: Parse JSON tokens into a structured representation.

**Use Case**: API response parsing, configuration files, data serialization.

**Algorithm**: Recursive descent parser

```rust
#[derive(Debug, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Colon,        // :
    Comma,        // ,
    String(String),
    Number(f64),
    True,
    False,
    Null,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn parse(&mut self) -> Result<JsonValue, String> {
        self.parse_value()
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        match self.peek() {
            Some(Token::LeftBrace) => self.parse_object(),
            Some(Token::LeftBracket) => self.parse_array(),
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(JsonValue::String(s))
            }
            Some(Token::Number(n)) => {
                let n = *n;
                self.advance();
                Ok(JsonValue::Number(n))
            }
            Some(Token::True) => {
                self.advance();
                Ok(JsonValue::Bool(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(JsonValue::Bool(false))
            }
            Some(Token::Null) => {
                self.advance();
                Ok(JsonValue::Null)
            }
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect(Token::LeftBrace)?;
        let mut pairs = Vec::new();

        loop {
            match self.peek() {
                Some(Token::RightBrace) => {
                    self.advance();
                    break;
                }
                Some(Token::String(key)) => {
                    let key = key.clone();
                    self.advance();
                    self.expect(Token::Colon)?;
                    let value = self.parse_value()?;
                    pairs.push((key, value));

                    match self.peek() {
                        Some(Token::Comma) => {
                            self.advance();
                        }
                        Some(Token::RightBrace) => {}
                        _ => return Err("Expected ',' or '}'".to_string()),
                    }
                }
                _ => return Err("Expected string key".to_string()),
            }
        }

        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect(Token::LeftBracket)?;
        let mut elements = Vec::new();

        loop {
            match self.peek() {
                Some(Token::RightBracket) => {
                    self.advance();
                    break;
                }
                _ => {
                    elements.push(self.parse_value()?);
                    match self.peek() {
                        Some(Token::Comma) => {
                            self.advance();
                        }
                        Some(Token::RightBracket) => {}
                        _ => return Err("Expected ',' or ']'".to_string()),
                    }
                }
            }
        }

        Ok(JsonValue::Array(elements))
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        match self.peek() {
            Some(token) if std::mem::discriminant(token) == std::mem::discriminant(&expected) => {
                self.advance();
                Ok(())
            }
            _ => Err(format!("Expected {:?}", expected)),
        }
    }
}

fn main() {
    let tokens = vec![
        Token::LeftBrace,
        Token::String("name".to_string()),
        Token::Colon,
        Token::String("Alice".to_string()),
        Token::Comma,
        Token::String("age".to_string()),
        Token::Colon,
        Token::Number(30.0),
        Token::RightBrace,
    ];

    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(json) => println!("{:?}", json),
        Err(e) => println!("Parse error: {}", e),
    }
}
```

**Key Patterns**:
- Discriminant matching for token comparison
- Recursive descent for nested structures
- Result propagation with `?`

---

### Recipe 2: Expression Evaluator with Operator Precedence

**Problem**: Evaluate arithmetic expressions respecting operator precedence.

**Use Case**: Calculator, formula evaluation, scripting languages.

**Algorithm**: Pratt parser (precedence climbing)

```rust
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Number(f64),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnOp,
        expr: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
enum UnOp {
    Neg,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

impl BinOp {
    fn precedence(&self) -> u8 {
        match self {
            BinOp::Add | BinOp::Sub => 1,
            BinOp::Mul | BinOp::Div => 2,
            BinOp::Pow => 3,
        }
    }

    fn eval(&self, left: f64, right: f64) -> f64 {
        match self {
            BinOp::Add => left + right,
            BinOp::Sub => left - right,
            BinOp::Mul => left * right,
            BinOp::Div => left / right,
            BinOp::Pow => left.powf(right),
        }
    }
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(tokens: Vec<Token>) -> Self {
        ExprParser { tokens, pos: 0 }
    }

    fn parse(&mut self) -> Result<Expr, String> {
        self.parse_expr(0)
    }

    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;

        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Caret) => BinOp::Pow,
                _ => break,
            };

            let prec = op.precedence();
            if prec < min_prec {
                break;
            }

            self.advance();

            let right = self.parse_expr(prec + 1)?;

            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Number(n)) => {
                let n = *n;
                self.advance();
                Ok(Expr::Number(n))
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(Expr::UnaryOp {
                    op: UnOp::Neg,
                    expr: Box::new(expr),
                })
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr(0)?;
                match self.peek() {
                    Some(Token::RParen) => {
                        self.advance();
                        Ok(expr)
                    }
                    _ => Err("Expected ')'".to_string()),
                }
            }
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}

fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::BinaryOp { op, left, right } => {
            let l = eval(left);
            let r = eval(right);
            op.eval(l, r)
        }
        Expr::UnaryOp { op: UnOp::Neg, expr } => -eval(expr),
    }
}

fn main() {
    // 2 + 3 * 4
    let tokens = vec![
        Token::Number(2.0),
        Token::Plus,
        Token::Number(3.0),
        Token::Star,
        Token::Number(4.0),
    ];

    let mut parser = ExprParser::new(tokens);
    match parser.parse() {
        Ok(expr) => {
            println!("Expression: {:?}", expr);
            println!("Result: {}", eval(&expr));
        }
        Err(e) => println!("Parse error: {}", e),
    }
}
```

**Key Patterns**:
- Precedence climbing algorithm
- Pattern matching on operator types
- Recursive expression evaluation

---

## State Machine Patterns

### Recipe 3: TCP Connection State Machine

**Problem**: Model TCP connection states and transitions.

**Use Case**: Network protocol implementation, connection management.

**Pattern**: Finite State Machine

```rust
#[derive(Debug, Clone, PartialEq)]
enum TcpState {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
}

#[derive(Debug)]
enum TcpEvent {
    PassiveOpen,
    ActiveOpen,
    SynReceived,
    SynAckReceived,
    AckReceived,
    Close,
    FinReceived,
    FinAckReceived,
    Timeout,
}

struct TcpConnection {
    state: TcpState,
}

impl TcpConnection {
    fn new() -> Self {
        TcpConnection {
            state: TcpState::Closed,
        }
    }

    fn handle_event(&mut self, event: TcpEvent) -> Result<(), String> {
        let new_state = match (&self.state, &event) {
            // From Closed
            (TcpState::Closed, TcpEvent::PassiveOpen) => TcpState::Listen,
            (TcpState::Closed, TcpEvent::ActiveOpen) => TcpState::SynSent,

            // From Listen
            (TcpState::Listen, TcpEvent::SynReceived) => TcpState::SynReceived,
            (TcpState::Listen, TcpEvent::Close) => TcpState::Closed,

            // From SynSent
            (TcpState::SynSent, TcpEvent::SynAckReceived) => TcpState::Established,
            (TcpState::SynSent, TcpEvent::SynReceived) => TcpState::SynReceived,
            (TcpState::SynSent, TcpEvent::Close) => TcpState::Closed,

            // From SynReceived
            (TcpState::SynReceived, TcpEvent::AckReceived) => TcpState::Established,
            (TcpState::SynReceived, TcpEvent::Close) => TcpState::FinWait1,

            // From Established
            (TcpState::Established, TcpEvent::Close) => TcpState::FinWait1,
            (TcpState::Established, TcpEvent::FinReceived) => TcpState::CloseWait,

            // From FinWait1
            (TcpState::FinWait1, TcpEvent::AckReceived) => TcpState::FinWait2,
            (TcpState::FinWait1, TcpEvent::FinReceived) => TcpState::Closing,
            (TcpState::FinWait1, TcpEvent::FinAckReceived) => TcpState::TimeWait,

            // From FinWait2
            (TcpState::FinWait2, TcpEvent::FinReceived) => TcpState::TimeWait,

            // From CloseWait
            (TcpState::CloseWait, TcpEvent::Close) => TcpState::LastAck,

            // From Closing
            (TcpState::Closing, TcpEvent::AckReceived) => TcpState::TimeWait,

            // From LastAck
            (TcpState::LastAck, TcpEvent::AckReceived) => TcpState::Closed,

            // From TimeWait
            (TcpState::TimeWait, TcpEvent::Timeout) => TcpState::Closed,

            // Invalid transitions
            _ => {
                return Err(format!(
                    "Invalid transition from {:?} on {:?}",
                    self.state, event
                ))
            }
        };

        println!("{:?} + {:?} -> {:?}", self.state, event, new_state);
        self.state = new_state;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.state == TcpState::Established
    }

    fn is_closed(&self) -> bool {
        self.state == TcpState::Closed
    }
}

fn main() {
    let mut conn = TcpConnection::new();

    // Active open (client)
    conn.handle_event(TcpEvent::ActiveOpen).unwrap();
    conn.handle_event(TcpEvent::SynAckReceived).unwrap();

    assert!(conn.is_connected());

    // Close connection
    conn.handle_event(TcpEvent::Close).unwrap();
    conn.handle_event(TcpEvent::AckReceived).unwrap();
    conn.handle_event(TcpEvent::FinReceived).unwrap();
    conn.handle_event(TcpEvent::Timeout).unwrap();

    assert!(conn.is_closed());
}
```

**Key Patterns**:
- Exhaustive state transition matching
- Tuple pattern for (state, event) combinations
- Guard clauses for invalid transitions

---

### Recipe 4: Regex Matcher (NFA)

**Problem**: Match strings against regular expression patterns.

**Use Case**: Text searching, validation, lexical analysis.

**Algorithm**: Non-deterministic Finite Automaton (NFA)

```rust
#[derive(Debug, Clone)]
enum NfaState {
    Char(char),
    Split(usize, usize),  // Two outgoing edges
    Match,
}

struct Nfa {
    states: Vec<NfaState>,
    start: usize,
}

impl Nfa {
    fn from_pattern(pattern: &str) -> Self {
        let mut states = Vec::new();
        let mut fragments = Vec::new();

        for ch in pattern.chars() {
            match ch {
                '|' => {
                    // Alternation
                    if let (Some(frag2), Some(frag1)) = (fragments.pop(), fragments.pop()) {
                        let split = states.len();
                        states.push(NfaState::Split(frag1, frag2));
                        fragments.push(split);
                    }
                }
                '*' => {
                    // Kleene star
                    if let Some(frag) = fragments.pop() {
                        let split = states.len();
                        states.push(NfaState::Split(frag, split + 1));
                        fragments.push(split);
                    }
                }
                c => {
                    // Regular character
                    let state = states.len();
                    states.push(NfaState::Char(c));
                    fragments.push(state);
                }
            }
        }

        // Add final match state
        states.push(NfaState::Match);

        // Connect fragments
        let start = if let Some(&last_frag) = fragments.last() {
            last_frag
        } else {
            states.len() - 1
        };

        Nfa { states, start }
    }

    fn matches(&self, text: &str) -> bool {
        let mut current_states = vec![self.start];
        let chars: Vec<char> = text.chars().collect();

        for &ch in &chars {
            let mut next_states = Vec::new();

            for &state_id in &current_states {
                self.step(state_id, ch, &mut next_states);
            }

            if next_states.is_empty() {
                return false;
            }

            current_states = next_states;
        }

        // Check if any current state is a match state
        self.has_match_state(&current_states)
    }

    fn step(&self, state_id: usize, ch: char, next_states: &mut Vec<usize>) {
        if state_id >= self.states.len() {
            return;
        }

        match &self.states[state_id] {
            NfaState::Char(c) if *c == ch => {
                if state_id + 1 < self.states.len() {
                    self.add_state(state_id + 1, next_states);
                }
            }
            NfaState::Split(s1, s2) => {
                self.step(*s1, ch, next_states);
                self.step(*s2, ch, next_states);
            }
            _ => {}
        }
    }

    fn add_state(&self, state_id: usize, states: &mut Vec<usize>) {
        if states.contains(&state_id) {
            return;
        }

        states.push(state_id);

        // Follow epsilon transitions (Split states)
        if let Some(NfaState::Split(s1, s2)) = self.states.get(state_id) {
            self.add_state(*s1, states);
            self.add_state(*s2, states);
        }
    }

    fn has_match_state(&self, states: &[usize]) -> bool {
        states.iter().any(|&s| {
            matches!(self.states.get(s), Some(NfaState::Match))
        })
    }
}

fn main() {
    let nfa = Nfa::from_pattern("ab");
    assert!(nfa.matches("ab"));
    assert!(!nfa.matches("abc"));

    println!("Pattern matching tests passed!");
}
```

**Key Patterns**:
- Recursive state exploration
- Pattern matching on state types
- Multiple simultaneous states (NFA property)

---

## Tree Traversal and Manipulation

### Recipe 5: AST Pattern Matching and Transformation

**Problem**: Transform abstract syntax trees for optimization or code generation.

**Use Case**: Compiler optimizations, code refactoring, linting.

**Algorithm**: Tree rewriting

```rust
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Num(i32),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

// Constant folding optimization
fn optimize(expr: Expr) -> Expr {
    match expr {
        // Base cases
        Expr::Num(_) | Expr::Var(_) => expr,

        // Addition patterns
        Expr::Add(left, right) => {
            let left = optimize(*left);
            let right = optimize(*right);

            match (&left, &right) {
                (Expr::Num(0), r) => r.clone(),
                (l, Expr::Num(0)) => l.clone(),
                (Expr::Num(a), Expr::Num(b)) => Expr::Num(a + b),
                _ => Expr::Add(Box::new(left), Box::new(right)),
            }
        }

        // Subtraction patterns
        Expr::Sub(left, right) => {
            let left = optimize(*left);
            let right = optimize(*right);

            match (&left, &right) {
                (l, Expr::Num(0)) => l.clone(),
                (Expr::Num(a), Expr::Num(b)) => Expr::Num(a - b),
                (l, r) if l == r => Expr::Num(0),
                _ => Expr::Sub(Box::new(left), Box::new(right)),
            }
        }

        // Multiplication patterns
        Expr::Mul(left, right) => {
            let left = optimize(*left);
            let right = optimize(*right);

            match (&left, &right) {
                (Expr::Num(0), _) | (_, Expr::Num(0)) => Expr::Num(0),
                (Expr::Num(1), r) => r.clone(),
                (l, Expr::Num(1)) => l.clone(),
                (Expr::Num(a), Expr::Num(b)) => Expr::Num(a * b),
                _ => Expr::Mul(Box::new(left), Box::new(right)),
            }
        }

        // Division patterns
        Expr::Div(left, right) => {
            let left = optimize(*left);
            let right = optimize(*right);

            match (&left, &right) {
                (Expr::Num(0), Expr::Num(n)) if *n != 0 => Expr::Num(0),
                (l, Expr::Num(1)) => l.clone(),
                (Expr::Num(a), Expr::Num(b)) if *b != 0 => Expr::Num(a / b),
                (l, r) if l == r => Expr::Num(1),
                _ => Expr::Div(Box::new(left), Box::new(right)),
            }
        }
    }
}

// Pretty print
fn print_expr(expr: &Expr) -> String {
    match expr {
        Expr::Num(n) => n.to_string(),
        Expr::Var(v) => v.clone(),
        Expr::Add(l, r) => format!("({} + {})", print_expr(l), print_expr(r)),
        Expr::Sub(l, r) => format!("({} - {})", print_expr(l), print_expr(r)),
        Expr::Mul(l, r) => format!("({} * {})", print_expr(l), print_expr(r)),
        Expr::Div(l, r) => format!("({} / {})", print_expr(l), print_expr(r)),
    }
}

fn main() {
    // (x + 0) * 1
    let expr = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Var("x".to_string())),
            Box::new(Expr::Num(0)),
        )),
        Box::new(Expr::Num(1)),
    );

    println!("Original: {}", print_expr(&expr));

    let optimized = optimize(expr);
    println!("Optimized: {}", print_expr(&optimized));
    // Output: x
}
```

**Key Patterns**:
- Nested pattern matching on tree nodes
- Guards with `if` conditions
- Algebraic simplification rules

---

### Recipe 6: Binary Tree Algorithms

**Problem**: Implement tree traversal and search algorithms.

**Use Case**: Database indices, expression trees, decision trees.

**Algorithms**: In-order, pre-order, post-order traversal

```rust
#[derive(Debug)]
struct Node<T> {
    value: T,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T: std::fmt::Display> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }

    // In-order traversal (left, root, right)
    fn inorder(&self) {
        if let Some(ref left) = self.left {
            left.inorder();
        }
        print!("{} ", self.value);
        if let Some(ref right) = self.right {
            right.inorder();
        }
    }

    // Pre-order traversal (root, left, right)
    fn preorder(&self) {
        print!("{} ", self.value);
        if let Some(ref left) = self.left {
            left.preorder();
        }
        if let Some(ref right) = self.right {
            right.preorder();
        }
    }

    // Post-order traversal (left, right, root)
    fn postorder(&self) {
        if let Some(ref left) = self.left {
            left.postorder();
        }
        if let Some(ref right) = self.right {
            right.postorder();
        }
        print!("{} ", self.value);
    }

    // Calculate tree height
    fn height(&self) -> usize {
        match (&self.left, &self.right) {
            (None, None) => 0,
            (Some(left), None) => 1 + left.height(),
            (None, Some(right)) => 1 + right.height(),
            (Some(left), Some(right)) => 1 + left.height().max(right.height()),
        }
    }

    // Count nodes
    fn count(&self) -> usize {
        1 + match (&self.left, &self.right) {
            (None, None) => 0,
            (Some(left), None) => left.count(),
            (None, Some(right)) => right.count(),
            (Some(left), Some(right)) => left.count() + right.count(),
        }
    }

    // Check if tree is balanced
    fn is_balanced(&self) -> bool {
        self.check_balance().is_some()
    }

    fn check_balance(&self) -> Option<usize> {
        match (&self.left, &self.right) {
            (None, None) => Some(0),
            (Some(left), None) => {
                let lh = left.check_balance()?;
                if lh < 2 {
                    Some(lh + 1)
                } else {
                    None
                }
            }
            (None, Some(right)) => {
                let rh = right.check_balance()?;
                if rh < 2 {
                    Some(rh + 1)
                } else {
                    None
                }
            }
            (Some(left), Some(right)) => {
                let lh = left.check_balance()?;
                let rh = right.check_balance()?;
                if (lh as i32 - rh as i32).abs() <= 1 {
                    Some(1 + lh.max(rh))
                } else {
                    None
                }
            }
        }
    }
}

fn main() {
    let mut root = Node::new(1);
    root.left = Some(Box::new(Node::new(2)));
    root.right = Some(Box::new(Node::new(3)));

    if let Some(ref mut left) = root.left {
        left.left = Some(Box::new(Node::new(4)));
        left.right = Some(Box::new(Node::new(5)));
    }

    print!("In-order: ");
    root.inorder();
    println!();

    print!("Pre-order: ");
    root.preorder();
    println!();

    print!("Post-order: ");
    root.postorder();
    println!();

    println!("Height: {}", root.height());
    println!("Count: {}", root.count());
    println!("Balanced: {}", root.is_balanced());
}
```

**Key Patterns**:
- Option matching for tree navigation
- Recursive pattern matching on both children
- Early return with `?` operator

---

## Protocol Parsing

### Recipe 7: HTTP Request Parser

**Problem**: Parse HTTP requests from raw bytes.

**Use Case**: Web servers, proxies, HTTP clients.

**Protocol**: HTTP/1.1

```rust
#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    version: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpRequest {
    fn parse(data: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(data)
            .map_err(|_| "Invalid UTF-8")?;

        let parts: Vec<&str> = text.split("\r\n\r\n").collect();

        match parts.as_slice() {
            [headers_text, body_text] | [headers_text] => {
                let lines: Vec<&str> = headers_text.lines().collect();

                let (method, path, version) = match lines.first() {
                    Some(request_line) => {
                        let parts: Vec<&str> = request_line.split_whitespace().collect();
                        match parts.as_slice() {
                            [method, path, version] => (
                                method.to_string(),
                                path.to_string(),
                                version.to_string(),
                            ),
                            _ => return Err("Invalid request line".to_string()),
                        }
                    }
                    None => return Err("Empty request".to_string()),
                };

                let mut headers = Vec::new();
                for line in &lines[1..] {
                    match line.split_once(':') {
                        Some((key, value)) => {
                            headers.push((key.trim().to_string(), value.trim().to_string()));
                        }
                        None if line.is_empty() => continue,
                        _ => return Err(format!("Invalid header: {}", line)),
                    }
                }

                let body = if parts.len() > 1 {
                    body_text.as_bytes().to_vec()
                } else {
                    Vec::new()
                };

                Ok(HttpRequest {
                    method,
                    path,
                    version,
                    headers,
                    body,
                })
            }
            _ => Err("Invalid HTTP format".to_string()),
        }
    }

    fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

fn main() {
    let request_data = b"GET /index.html HTTP/1.1\r\n\
                         Host: example.com\r\n\
                         User-Agent: Rust/1.0\r\n\
                         Accept: */*\r\n\
                         \r\n";

    match HttpRequest::parse(request_data) {
        Ok(req) => {
            println!("Method: {}", req.method);
            println!("Path: {}", req.path);
            println!("Version: {}", req.version);
            println!("Host: {:?}", req.get_header("Host"));
        }
        Err(e) => println!("Parse error: {}", e),
    }
}
```

**Key Patterns**:
- Slice pattern matching
- `split_once()` for key-value pairs
- Iterator pattern matching with `find()`

---

### Recipe 8: Binary Protocol Decoder (TLV)

**Problem**: Decode Type-Length-Value (TLV) binary protocol.

**Use Case**: Network protocols, file formats, serialization.

**Format**: TLV (Type, Length, Value)

```rust
#[derive(Debug, PartialEq)]
enum TlvType {
    String = 1,
    Integer = 2,
    Boolean = 3,
    Array = 4,
}

#[derive(Debug)]
enum TlvValue {
    String(String),
    Integer(i32),
    Boolean(bool),
    Array(Vec<TlvValue>),
}

struct TlvDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> TlvDecoder<'a> {
    fn new(data: &'a [u8]) -> Self {
        TlvDecoder { data, pos: 0 }
    }

    fn decode(&mut self) -> Result<Vec<TlvValue>, String> {
        let mut values = Vec::new();

        while self.pos < self.data.len() {
            values.push(self.decode_value()?);
        }

        Ok(values)
    }

    fn decode_value(&mut self) -> Result<TlvValue, String> {
        if self.pos + 2 > self.data.len() {
            return Err("Insufficient data for TLV header".to_string());
        }

        let type_byte = self.data[self.pos];
        self.pos += 1;

        let length = self.data[self.pos] as usize;
        self.pos += 1;

        if self.pos + length > self.data.len() {
            return Err("Insufficient data for TLV value".to_string());
        }

        let value_bytes = &self.data[self.pos..self.pos + length];
        self.pos += length;

        match type_byte {
            1 => {
                // String
                let s = std::str::from_utf8(value_bytes)
                    .map_err(|_| "Invalid UTF-8 in string")?
                    .to_string();
                Ok(TlvValue::String(s))
            }
            2 => {
                // Integer (big-endian i32)
                if value_bytes.len() != 4 {
                    return Err("Invalid integer length".to_string());
                }
                let value = i32::from_be_bytes([
                    value_bytes[0],
                    value_bytes[1],
                    value_bytes[2],
                    value_bytes[3],
                ]);
                Ok(TlvValue::Integer(value))
            }
            3 => {
                // Boolean
                match value_bytes {
                    [0] => Ok(TlvValue::Boolean(false)),
                    [1] => Ok(TlvValue::Boolean(true)),
                    _ => Err("Invalid boolean value".to_string()),
                }
            }
            4 => {
                // Array
                let mut decoder = TlvDecoder::new(value_bytes);
                let elements = decoder.decode()?;
                Ok(TlvValue::Array(elements))
            }
            _ => Err(format!("Unknown type: {}", type_byte)),
        }
    }
}

fn main() {
    // String "hello": type=1, length=5, value="hello"
    // Integer 42: type=2, length=4, value=0x0000002A
    let data: Vec<u8> = vec![
        1, 5, b'h', b'e', b'l', b'l', b'o',
        2, 4, 0, 0, 0, 42,
        3, 1, 1,  // Boolean true
    ];

    let mut decoder = TlvDecoder::new(&data);
    match decoder.decode() {
        Ok(values) => {
            for value in values {
                println!("{:?}", value);
            }
        }
        Err(e) => println!("Decode error: {}", e),
    }
}
```

**Key Patterns**:
- Byte array pattern matching
- Fixed-size array patterns for integers
- Recursive decoding for nested structures

---

## Compiler and Interpreter Patterns

### Recipe 9: Bytecode Interpreter

**Problem**: Execute bytecode instructions.

**Use Case**: Virtual machines, scripting languages, JIT compilers.

**Algorithm**: Stack-based VM

```rust
#[derive(Debug, Clone)]
enum Instruction {
    Push(i32),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Print,
    Halt,
}

struct VirtualMachine {
    stack: Vec<i32>,
    pc: usize,
    instructions: Vec<Instruction>,
}

impl VirtualMachine {
    fn new(instructions: Vec<Instruction>) -> Self {
        VirtualMachine {
            stack: Vec::new(),
            pc: 0,
            instructions,
        }
    }

    fn run(&mut self) -> Result<(), String> {
        loop {
            if self.pc >= self.instructions.len() {
                return Err("Program counter out of bounds".to_string());
            }

            let instruction = &self.instructions[self.pc].clone();
            self.pc += 1;

            match instruction {
                Instruction::Push(value) => {
                    self.stack.push(*value);
                }
                Instruction::Pop => {
                    self.stack.pop()
                        .ok_or("Stack underflow")?;
                }
                Instruction::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a + b);
                }
                Instruction::Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a - b);
                }
                Instruction::Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(a * b);
                }
                Instruction::Div => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if b == 0 {
                        return Err("Division by zero".to_string());
                    }
                    self.stack.push(a / b);
                }
                Instruction::Print => {
                    let value = self.stack.last()
                        .ok_or("Stack underflow")?;
                    println!("{}", value);
                }
                Instruction::Halt => {
                    return Ok(());
                }
            }
        }
    }
}

fn main() {
    // Calculate: (5 + 3) * 2
    let program = vec![
        Instruction::Push(5),
        Instruction::Push(3),
        Instruction::Add,
        Instruction::Push(2),
        Instruction::Mul,
        Instruction::Print,
        Instruction::Halt,
    ];

    let mut vm = VirtualMachine::new(program);
    match vm.run() {
        Ok(()) => println!("Program executed successfully"),
        Err(e) => println!("Runtime error: {}", e),
    }
}
```

**Key Patterns**:
- Exhaustive instruction matching
- Error propagation with `?`
- Stack manipulation patterns

---

## Graph Algorithms

### Recipe 10: Graph Pattern Matching and Traversal

**Problem**: Find patterns in graphs (e.g., triangles, cliques).

**Use Case**: Social network analysis, fraud detection, recommendation systems.

**Algorithm**: Subgraph matching

```rust
use std::collections::{HashMap, HashSet};

type NodeId = usize;

struct Graph {
    edges: HashMap<NodeId, HashSet<NodeId>>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: NodeId, to: NodeId) {
        self.edges.entry(from).or_insert_with(HashSet::new).insert(to);
        self.edges.entry(to).or_insert_with(HashSet::new).insert(from);
    }

    fn neighbors(&self, node: NodeId) -> Option<&HashSet<NodeId>> {
        self.edges.get(&node)
    }

    // Find all triangles in the graph
    fn find_triangles(&self) -> Vec<(NodeId, NodeId, NodeId)> {
        let mut triangles = Vec::new();

        for (&node1, neighbors1) in &self.edges {
            for &node2 in neighbors1 {
                if node2 <= node1 {
                    continue;
                }

                if let Some(neighbors2) = self.edges.get(&node2) {
                    let common: Vec<_> = neighbors1
                        .intersection(neighbors2)
                        .filter(|&&n| n > node2)
                        .copied()
                        .collect();

                    for node3 in common {
                        triangles.push((node1, node2, node3));
                    }
                }
            }
        }

        triangles
    }

    // Check if nodes form a clique
    fn is_clique(&self, nodes: &[NodeId]) -> bool {
        for i in 0..nodes.len() {
            for j in i + 1..nodes.len() {
                match (self.neighbors(nodes[i]), self.neighbors(nodes[j])) {
                    (Some(neighbors_i), _) if !neighbors_i.contains(&nodes[j]) => {
                        return false;
                    }
                    _ => {}
                }
            }
        }
        true
    }

    // Find all cliques of given size
    fn find_cliques(&self, size: usize) -> Vec<Vec<NodeId>> {
        let nodes: Vec<_> = self.edges.keys().copied().collect();
        let mut cliques = Vec::new();

        self.find_cliques_recursive(&nodes, Vec::new(), 0, size, &mut cliques);

        cliques
    }

    fn find_cliques_recursive(
        &self,
        candidates: &[NodeId],
        current: Vec<NodeId>,
        start: usize,
        target_size: usize,
        result: &mut Vec<Vec<NodeId>>,
    ) {
        if current.len() == target_size {
            if self.is_clique(&current) {
                result.push(current);
            }
            return;
        }

        for i in start..candidates.len() {
            let mut new_current = current.clone();
            new_current.push(candidates[i]);
            self.find_cliques_recursive(candidates, new_current, i + 1, target_size, result);
        }
    }
}

fn main() {
    let mut graph = Graph::new();

    // Create a graph with triangles
    graph.add_edge(1, 2);
    graph.add_edge(2, 3);
    graph.add_edge(3, 1);
    graph.add_edge(3, 4);
    graph.add_edge(4, 5);
    graph.add_edge(5, 3);

    let triangles = graph.find_triangles();
    println!("Triangles found: {:?}", triangles);

    let cliques = graph.find_cliques(3);
    println!("3-cliques: {:?}", cliques);
}
```

**Key Patterns**:
- HashSet intersection for common neighbors
- Recursive backtracking with pattern matching
- Option matching for graph navigation

---

## Game Logic Patterns

### Recipe 11: Chess Move Validation

**Problem**: Validate chess moves according to piece rules.

**Use Case**: Board games, move validation, AI game playing.

**Domain**: Chess

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    White,
    Black,
}

type Position = (i8, i8);  // (row, col)

#[derive(Debug, Clone, Copy)]
struct ChessPiece {
    piece: Piece,
    color: Color,
}

fn is_valid_move(
    piece: ChessPiece,
    from: Position,
    to: Position,
    capture: bool,
) -> bool {
    let (from_row, from_col) = from;
    let (to_row, to_col) = to;
    let row_diff = (to_row - from_row).abs();
    let col_diff = (to_col - from_col).abs();

    match piece.piece {
        Piece::Pawn => {
            let direction = match piece.color {
                Color::White => 1,
                Color::Black => -1,
            };

            match (capture, to_row - from_row, col_diff) {
                // Forward move (1 square)
                (false, r, 0) if r == direction => true,
                // Forward move (2 squares from start)
                (false, r, 0) if r == 2 * direction => {
                    let start_row = match piece.color {
                        Color::White => 1,
                        Color::Black => 6,
                    };
                    from_row == start_row
                }
                // Diagonal capture
                (true, r, 1) if r == direction => true,
                _ => false,
            }
        }

        Piece::Knight => {
            matches!(
                (row_diff, col_diff),
                (2, 1) | (1, 2)
            )
        }

        Piece::Bishop => {
            row_diff == col_diff && row_diff > 0
        }

        Piece::Rook => {
            (row_diff > 0 && col_diff == 0) || (row_diff == 0 && col_diff > 0)
        }

        Piece::Queen => {
            // Combination of rook and bishop
            (row_diff == col_diff && row_diff > 0) ||
            (row_diff > 0 && col_diff == 0) ||
            (row_diff == 0 && col_diff > 0)
        }

        Piece::King => {
            row_diff <= 1 && col_diff <= 1 && (row_diff + col_diff) > 0
        }
    }
}

fn main() {
    let white_pawn = ChessPiece {
        piece: Piece::Pawn,
        color: Color::White,
    };

    // Test pawn moves
    assert!(is_valid_move(white_pawn, (1, 0), (2, 0), false)); // 1 forward
    assert!(is_valid_move(white_pawn, (1, 0), (3, 0), false)); // 2 forward from start
    assert!(is_valid_move(white_pawn, (1, 0), (2, 1), true));  // Diagonal capture
    assert!(!is_valid_move(white_pawn, (1, 0), (2, 1), false)); // Invalid: diagonal without capture

    let knight = ChessPiece {
        piece: Piece::Knight,
        color: Color::White,
    };

    assert!(is_valid_move(knight, (0, 1), (2, 2), false)); // L-shape
    assert!(!is_valid_move(knight, (0, 1), (2, 1), false)); // Invalid

    println!("Chess move validation tests passed!");
}
```

**Key Patterns**:
- Tuple destructuring in match arms
- Multiple pattern alternatives with `|`
- Guard clauses for complex conditions

---

## Error Recovery Patterns

### Recipe 12: Resilient JSON Parser with Error Recovery

**Problem**: Parse JSON while recovering from errors.

**Use Case**: Log parsing, data migration, fault-tolerant systems.

**Strategy**: Error recovery and partial parsing

```rust
#[derive(Debug, Clone, PartialEq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Debug)]
enum ParseError {
    UnexpectedToken(String),
    InvalidNumber(String),
    UnterminatedString,
    MissingComma,
}

struct ResilientParser {
    input: Vec<char>,
    pos: usize,
    errors: Vec<(usize, ParseError)>,
}

impl ResilientParser {
    fn new(input: &str) -> Self {
        ResilientParser {
            input: input.chars().collect(),
            pos: 0,
            errors: Vec::new(),
        }
    }

    fn parse(&mut self) -> Option<JsonValue> {
        self.skip_whitespace();

        match self.peek() {
            Some('{') => self.parse_object_resilient(),
            Some('[') => self.parse_array_resilient(),
            Some('"') => self.parse_string().ok(),
            Some('t') | Some('f') => self.parse_bool().ok(),
            Some('n') => self.parse_null().ok(),
            Some(c) if c.is_numeric() || *c == '-' => self.parse_number().ok(),
            _ => {
                self.record_error(ParseError::UnexpectedToken(
                    self.peek().map(|c| c.to_string()).unwrap_or_default()
                ));
                None
            }
        }
    }

    fn parse_object_resilient(&mut self) -> Option<JsonValue> {
        self.consume('{');
        let mut pairs = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                Some('}') => {
                    self.advance();
                    break;
                }
                Some('"') => {
                    if let Ok(key) = self.parse_string() {
                        self.skip_whitespace();

                        match self.consume(':') {
                            true => {
                                if let Some(value) = self.parse() {
                                    pairs.push((key, value));
                                } else {
                                    // Skip to next comma or brace
                                    self.skip_to_recovery_point(&[',', '}']);
                                }
                            }
                            false => {
                                self.record_error(ParseError::UnexpectedToken(":".to_string()));
                                self.skip_to_recovery_point(&[',', '}']);
                            }
                        }

                        self.skip_whitespace();
                        match self.peek() {
                            Some(',') => {
                                self.advance();
                            }
                            Some('}') => {}
                            _ => {
                                self.record_error(ParseError::MissingComma);
                            }
                        }
                    }
                }
                Some(_) => {
                    // Skip invalid token
                    self.advance();
                }
                None => break,
            }
        }

        Some(JsonValue::Object(pairs))
    }

    fn parse_array_resilient(&mut self) -> Option<JsonValue> {
        self.consume('[');
        let mut elements = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                Some(']') => {
                    self.advance();
                    break;
                }
                Some(_) => {
                    if let Some(value) = self.parse() {
                        elements.push(value);
                    } else {
                        self.skip_to_recovery_point(&[',', ']']);
                    }

                    self.skip_whitespace();
                    match self.peek() {
                        Some(',') => {
                            self.advance();
                        }
                        Some(']') => {}
                        _ => {
                            self.record_error(ParseError::MissingComma);
                        }
                    }
                }
                None => break,
            }
        }

        Some(JsonValue::Array(elements))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.consume('"');
        let mut s = String::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    return Ok(s);
                }
                Some('\\') => {
                    self.advance();
                    match self.peek() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some(c) => s.push(*c),
                        None => return Err(ParseError::UnterminatedString),
                    }
                    self.advance();
                }
                Some(c) => {
                    s.push(*c);
                    self.advance();
                }
                None => return Err(ParseError::UnterminatedString),
            }
        }
    }

    fn parse_number(&mut self) -> Result<f64, ParseError> {
        let mut num_str = String::new();

        while let Some(&c) = self.peek() {
            if c.is_numeric() || c == '.' || c == '-' || c == 'e' || c == 'E' {
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        num_str.parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(num_str))
    }

    fn parse_bool(&mut self) -> Result<bool, ParseError> {
        let start = self.pos;

        for &expected in b"true" {
            if self.peek() == Some(&(expected as char)) {
                self.advance();
            } else {
                self.pos = start;
                for &expected in b"false" {
                    if self.peek() == Some(&(expected as char)) {
                        self.advance();
                    } else {
                        return Err(ParseError::UnexpectedToken("bool".to_string()));
                    }
                }
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn parse_null(&mut self) -> Result<JsonValue, ParseError> {
        for &expected in b"null" {
            if self.peek() == Some(&(expected as char)) {
                self.advance();
            } else {
                return Err(ParseError::UnexpectedToken("null".to_string()));
            }
        }
        Ok(JsonValue::Null)
    }

    fn skip_to_recovery_point(&mut self, markers: &[char]) {
        while let Some(&c) = self.peek() {
            if markers.contains(&c) {
                break;
            }
            self.advance();
        }
    }

    fn record_error(&mut self, error: ParseError) {
        self.errors.push((self.pos, error));
    }

    fn peek(&self) -> Option<&char> {
        self.input.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn consume(&mut self, expected: char) -> bool {
        if self.peek() == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
}

fn main() {
    // Malformed JSON with missing quotes and extra commas
    let json = r#"{"name": John, "age": 30,, "city": "NYC"}"#;

    let mut parser = ResilientParser::new(json);
    match parser.parse() {
        Some(value) => {
            println!("Parsed (with errors): {:?}", value);
            println!("Errors: {:?}", parser.errors);
        }
        None => println!("Failed to parse"),
    }
}
```

**Key Patterns**:
- Error recovery with skip-to-marker strategy
- Partial success pattern
- Error collection while continuing parse

---

## Configuration Processing

### Recipe 13: Config File Parser with Validation

**Problem**: Parse and validate configuration files.

**Use Case**: Application configuration, deployment settings, feature flags.

**Format**: Custom INI-like format

```rust
use std::collections::HashMap;

#[derive(Debug)]
enum ConfigValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

#[derive(Debug)]
struct Config {
    sections: HashMap<String, HashMap<String, ConfigValue>>,
}

impl Config {
    fn parse(input: &str) -> Result<Self, String> {
        let mut sections = HashMap::new();
        let mut current_section = String::from("default");
        sections.insert(current_section.clone(), HashMap::new());

        for (line_num, line) in input.lines().enumerate() {
            let line = line.trim();

            match line {
                // Empty line or comment
                "" | s if s.starts_with('#') => continue,

                // Section header
                s if s.starts_with('[') && s.ends_with(']') => {
                    current_section = s[1..s.len() - 1].to_string();
                    sections.insert(current_section.clone(), HashMap::new());
                }

                // Key-value pair
                s => {
                    match s.split_once('=') {
                        Some((key, value)) => {
                            let key = key.trim().to_string();
                            let value = Self::parse_value(value.trim())?;

                            sections.get_mut(&current_section)
                                .ok_or_else(|| format!("No section at line {}", line_num + 1))?
                                .insert(key, value);
                        }
                        None => {
                            return Err(format!("Invalid line {}: {}", line_num + 1, line));
                        }
                    }
                }
            }
        }

        Ok(Config { sections })
    }

    fn parse_value(s: &str) -> Result<ConfigValue, String> {
        match s {
            // Boolean
            "true" => Ok(ConfigValue::Boolean(true)),
            "false" => Ok(ConfigValue::Boolean(false)),

            // String (quoted)
            s if s.starts_with('"') && s.ends_with('"') => {
                Ok(ConfigValue::String(s[1..s.len() - 1].to_string()))
            }

            // Array
            s if s.starts_with('[') && s.ends_with(']') => {
                let elements: Result<Vec<_>, _> = s[1..s.len() - 1]
                    .split(',')
                    .map(|e| Self::parse_value(e.trim()))
                    .collect();
                elements.map(ConfigValue::Array)
            }

            // Integer
            s => {
                s.parse::<i64>()
                    .map(ConfigValue::Integer)
                    .or_else(|_| Ok(ConfigValue::String(s.to_string())))
            }
        }
    }

    fn get(&self, section: &str, key: &str) -> Option<&ConfigValue> {
        self.sections.get(section).and_then(|s| s.get(key))
    }

    fn get_string(&self, section: &str, key: &str) -> Option<&str> {
        match self.get(section, key) {
            Some(ConfigValue::String(s)) => Some(s),
            _ => None,
        }
    }

    fn get_int(&self, section: &str, key: &str) -> Option<i64> {
        match self.get(section, key) {
            Some(ConfigValue::Integer(n)) => Some(*n),
            _ => None,
        }
    }

    fn get_bool(&self, section: &str, key: &str) -> Option<bool> {
        match self.get(section, key) {
            Some(ConfigValue::Boolean(b)) => Some(*b),
            _ => None,
        }
    }
}

fn main() {
    let config_text = r#"
# Database configuration
[database]
host = "localhost"
port = 5432
username = "admin"
max_connections = 100
ssl_enabled = true
replicas = ["db1", "db2", "db3"]

[cache]
enabled = true
ttl = 300
"#;

    match Config::parse(config_text) {
        Ok(config) => {
            println!("Database host: {:?}", config.get_string("database", "host"));
            println!("Database port: {:?}", config.get_int("database", "port"));
            println!("SSL enabled: {:?}", config.get_bool("database", "ssl_enabled"));
            println!("Replicas: {:?}", config.get("database", "replicas"));
        }
        Err(e) => println!("Config parse error: {}", e),
    }
}
```

**Key Patterns**:
- Multi-level pattern matching (section/key/value)
- String pattern matching with guards
- Type-safe value extraction

---

## Quick Reference

### Pattern Matching Essentials

```rust
// Literal matching
match value {
    0 => "zero",
    1 => "one",
    _ => "other",
}

// Multiple patterns
match value {
    1 | 2 | 3 => "small",
    _ => "large",
}

// Range patterns
match value {
    0..=100 => "percentage",
    _ => "invalid",
}

// Tuple destructuring
match point {
    (0, 0) => "origin",
    (0, y) => "on y-axis",
    (x, 0) => "on x-axis",
    (x, y) => "general",
}

// Struct destructuring
match person {
    Person { age: 0..=17, .. } => "minor",
    Person { age: 18..=64, .. } => "adult",
    Person { age: 65.., .. } => "senior",
}

// Enum matching
match result {
    Ok(value) => value,
    Err(e) => handle_error(e),
}

// Guards
match value {
    n if n < 0 => "negative",
    0 => "zero",
    n if n > 0 => "positive",
    _ => unreachable!(),
}

// Binding
match value {
    Some(x @ 0..=100) => x,
    Some(x) => 100,
    None => 0,
}
```

### Common Patterns

| Pattern | Example | Use Case |
|---------|---------|----------|
| Option unwrapping | `if let Some(x) = opt` | Handling optionals |
| Result handling | `match result { Ok(v) => ..., Err(e) => ... }` | Error handling |
| Iterator patterns | `for (key, value) in map` | Collection iteration |
| Slice patterns | `match slice { [first, rest @ ..] => ... }` | Sequence processing |
| Reference patterns | `match &value { Some(ref x) => ... }` | Borrowing in match |

---

## Summary

Advanced pattern matching enables:

1. **Parser Implementation**: Recursive descent, precedence climbing
2. **State Machines**: Finite automata, protocol handlers
3. **Tree Algorithms**: AST transformation, traversal
4. **Protocol Parsing**: Binary and text protocols
5. **Interpreters**: Bytecode execution, expression evaluation
6. **Graph Algorithms**: Pattern detection, subgraph matching
7. **Game Logic**: Rule validation, move generation
8. **Error Recovery**: Resilient parsing, partial success
9. **Configuration**: Validation, type-safe access

**Key Techniques**:
- Exhaustive matching for safety
- Guard clauses for complex conditions
- Recursive patterns for nested structures
- Error recovery with skip-ahead
- Type-safe extraction with pattern binding
