# Advanced Pattern Matching Cookbook

Real-world algorithms, data structures, and patterns using Rust's pattern matching for experienced programmers.

## Table of Contents

1. [Parser Combinators](#parser-combinators)
2. [AST Walking & Transformation](#ast-walking--transformation)
3. [State Machines](#state-machines)
4. [Protocol Parsing](#protocol-parsing)
5. [Error Recovery](#error-recovery)
6. [Binary Tree Operations](#binary-tree-operations)
7. [Graph Traversal](#graph-traversal)
8. [Instruction Dispatch](#instruction-dispatch)
9. [Type-Level State](#type-level-state)
10. [Iterator Patterns](#iterator-patterns)

---

## Parser Combinators

Tokenization and parsing using pattern matching for stream processing.

```rust
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(i64),
    Ident(String),
    Op(char),
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, PartialEq)]
enum Expr {
    Lit(i64),
    Var(String),
    BinOp(Box<Expr>, char, Box<Expr>),
    Call(String, Vec<Expr>),
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> &Token {
        let tok = self.peek();
        self.pos += 1;
        tok
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Token::Number(n) => Ok(Expr::Lit(*n)),
            Token::Ident(name) => {
                match self.peek() {
                    Token::LParen => {
                        self.advance(); // consume '('
                        let mut args = vec![];
                        loop {
                            match self.peek() {
                                Token::RParen => {
                                    self.advance();
                                    break;
                                }
                                _ => {
                                    args.push(self.parse_expr()?);
                                    match self.peek() {
                                        Token::Op(',') => { self.advance(); }
                                        Token::RParen => {}
                                        tok => return Err(format!("Expected ',' or ')', got {:?}", tok)),
                                    }
                                }
                            }
                        }
                        Ok(Expr::Call(name.clone(), args))
                    }
                    _ => Ok(Expr::Var(name.clone()))
                }
            }
            Token::LParen => {
                let expr = self.parse_expr()?;
                match self.advance() {
                    Token::RParen => Ok(expr),
                    tok => Err(format!("Expected ')', got {:?}", tok)),
                }
            }
            tok => Err(format!("Unexpected token: {:?}", tok)),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_primary()?;

        match self.peek() {
            Token::Op(op @ ('+' | '-' | '*' | '/')) => {
                let op = *op;
                self.advance();
                let right = self.parse_expr()?;
                Ok(Expr::BinOp(Box::new(left), op, Box::new(right)))
            }
            _ => Ok(left),
        }
    }
}

fn main() {
    let tokens = vec![
        Token::Ident("add".into()),
        Token::LParen,
        Token::Number(1),
        Token::Op(','),
        Token::Number(2),
        Token::RParen,
    ];

    let mut parser = Parser::new(&tokens);
    match parser.parse_expr() {
        Ok(expr) => println!("{:?}", expr),
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
```

---

## AST Walking & Transformation

Pattern matching for compiler passes, code optimization, and AST transformations.

```rust
#[derive(Debug, Clone, PartialEq)]
enum Expr {
    Lit(i64),
    Var(String),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

// Constant folding optimization pass
fn fold_constants(expr: Expr) -> Expr {
    match expr {
        // 0 + x => x
        Expr::Add(box Expr::Lit(0), box right) => fold_constants(right),
        // x + 0 => x
        Expr::Add(box left, box Expr::Lit(0)) => fold_constants(left),
        // const + const => result
        Expr::Add(box Expr::Lit(a), box Expr::Lit(b)) => Expr::Lit(a + b),
        // Recurse
        Expr::Add(left, right) => {
            Expr::Add(
                Box::new(fold_constants(*left)),
                Box::new(fold_constants(*right)),
            )
        }

        // 1 * x => x
        Expr::Mul(box Expr::Lit(1), box right) => fold_constants(right),
        // x * 1 => x
        Expr::Mul(box left, box Expr::Lit(1)) => fold_constants(left),
        // 0 * _ => 0
        Expr::Mul(box Expr::Lit(0), _) | Expr::Mul(_, box Expr::Lit(0)) => Expr::Lit(0),
        // const * const => result
        Expr::Mul(box Expr::Lit(a), box Expr::Lit(b)) => Expr::Lit(a * b),
        // Recurse
        Expr::Mul(left, right) => {
            Expr::Mul(
                Box::new(fold_constants(*left)),
                Box::new(fold_constants(*right)),
            )
        }

        // if true then a else b => a
        Expr::If(box Expr::Lit(n), then_branch, _) if n != 0 => {
            fold_constants(*then_branch)
        }
        // if false then a else b => b
        Expr::If(box Expr::Lit(0), _, else_branch) => {
            fold_constants(*else_branch)
        }
        // Recurse
        Expr::If(cond, then_branch, else_branch) => {
            Expr::If(
                Box::new(fold_constants(*cond)),
                Box::new(fold_constants(*then_branch)),
                Box::new(fold_constants(*else_branch)),
            )
        }

        other => other,
    }
}

// Dead code elimination
fn eliminate_dead_code(expr: Expr) -> Option<Expr> {
    match expr {
        Expr::If(box Expr::Lit(0), _, else_branch) => {
            eliminate_dead_code(*else_branch)
        }
        Expr::If(box Expr::Lit(_), then_branch, _) => {
            eliminate_dead_code(*then_branch)
        }
        Expr::If(cond, then_b, else_b) => {
            Some(Expr::If(
                Box::new(eliminate_dead_code(*cond)?),
                Box::new(eliminate_dead_code(*then_b)?),
                Box::new(eliminate_dead_code(*else_b)?),
            ))
        }
        other => Some(other),
    }
}

fn main() {
    let expr = Expr::Add(
        Box::new(Expr::Lit(0)),
        Box::new(Expr::Mul(
            Box::new(Expr::Lit(2)),
            Box::new(Expr::Lit(3)),
        )),
    );

    println!("Original: {:?}", expr);
    let optimized = fold_constants(expr);
    println!("Optimized: {:?}", optimized); // Lit(6)
}
```

---

## State Machines

Type-safe state machine using phantom types and pattern matching.

```rust
use std::marker::PhantomData;

// Connection states
struct Disconnected;
struct Connected;
struct Authenticated;

struct Connection<S> {
    host: String,
    _state: PhantomData<S>,
}

impl Connection<Disconnected> {
    fn new(host: String) -> Self {
        Connection {
            host,
            _state: PhantomData,
        }
    }

    fn connect(self) -> Result<Connection<Connected>, String> {
        println!("Connecting to {}", self.host);
        Ok(Connection {
            host: self.host,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn authenticate(self, token: &str) -> Result<Connection<Authenticated>, Self> {
        if token.len() > 8 {
            println!("Authenticated");
            Ok(Connection {
                host: self.host,
                _state: PhantomData,
            })
        } else {
            Err(self)
        }
    }

    fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnected");
        Connection {
            host: self.host,
            _state: PhantomData,
        }
    }
}

impl Connection<Authenticated> {
    fn send_command(&self, cmd: &str) {
        println!("Sending: {}", cmd);
    }

    fn disconnect(self) -> Connection<Disconnected> {
        println!("Logging out and disconnecting");
        Connection {
            host: self.host,
            _state: PhantomData,
        }
    }
}

// Protocol state machine
#[derive(Debug)]
enum ProtocolState {
    Init,
    Handshake { version: u8 },
    Ready { session_id: u64 },
    Closed,
}

impl ProtocolState {
    fn handle_message(&mut self, msg: &[u8]) -> Result<(), String> {
        *self = match (&self, msg) {
            (ProtocolState::Init, [0x01, version, ..]) => {
                ProtocolState::Handshake { version: *version }
            }
            (ProtocolState::Handshake { version }, [0x02, session @ ..]) if *version >= 2 => {
                let session_id = u64::from_be_bytes(session[..8].try_into().unwrap());
                ProtocolState::Ready { session_id }
            }
            (ProtocolState::Ready { .. }, [0x03, ..]) => {
                ProtocolState::Closed
            }
            (state, msg) => {
                return Err(format!("Invalid message {:?} in state {:?}", msg, state));
            }
        };
        Ok(())
    }
}

fn main() {
    // Type-safe state transitions at compile time
    let conn = Connection::new("example.com".into());
    let conn = conn.connect().unwrap();
    let conn = conn.authenticate("secure_token_123").unwrap();
    conn.send_command("GET /data");

    // Runtime state machine
    let mut proto = ProtocolState::Init;
    proto.handle_message(&[0x01, 0x02]).unwrap();
    proto.handle_message(&[0x02, 0, 0, 0, 0, 0, 0, 0x12, 0x34]).unwrap();
    println!("{:?}", proto);
}
```

---

## Protocol Parsing

Binary protocol parsing with slice patterns.

```rust
#[derive(Debug, PartialEq)]
enum Message {
    Ping { seq: u16 },
    Pong { seq: u16 },
    Data { channel: u8, payload: Vec<u8> },
    Close { code: u16 },
}

fn parse_message(bytes: &[u8]) -> Result<Message, String> {
    match bytes {
        // Ping: 0x01 | seq:u16
        [0x01, seq @ ..] if seq.len() >= 2 => {
            let seq = u16::from_be_bytes([seq[0], seq[1]]);
            Ok(Message::Ping { seq })
        }

        // Pong: 0x02 | seq:u16
        [0x02, seq @ ..] if seq.len() >= 2 => {
            let seq = u16::from_be_bytes([seq[0], seq[1]]);
            Ok(Message::Pong { seq })
        }

        // Data: 0x03 | channel:u8 | len:u16 | payload
        [0x03, channel, len_hi, len_lo, payload @ ..] => {
            let len = u16::from_be_bytes([*len_hi, *len_lo]) as usize;
            if payload.len() >= len {
                Ok(Message::Data {
                    channel: *channel,
                    payload: payload[..len].to_vec(),
                })
            } else {
                Err("Incomplete payload".into())
            }
        }

        // Close: 0x04 | code:u16
        [0x04, code @ ..] if code.len() >= 2 => {
            let code = u16::from_be_bytes([code[0], code[1]]);
            Ok(Message::Close { code })
        }

        // HTTP-like header parsing
        [b'G', b'E', b'T', b' ', path @ .., b'\r', b'\n'] => {
            println!("GET {}", String::from_utf8_lossy(path));
            Ok(Message::Data { channel: 0, payload: vec![] })
        }

        _ => Err(format!("Unknown message format")),
    }
}

// More complex: TLV (Type-Length-Value) parser
fn parse_tlv_stream(mut data: &[u8]) -> Vec<(u8, Vec<u8>)> {
    let mut records = vec![];

    while !data.is_empty() {
        match data {
            [typ, len, rest @ ..] if rest.len() >= *len as usize => {
                records.push((*typ, rest[..*len as usize].to_vec()));
                data = &rest[*len as usize..];
            }
            _ => break,
        }
    }

    records
}

fn main() {
    let ping = [0x01, 0x00, 0x42];
    assert_eq!(parse_message(&ping), Ok(Message::Ping { seq: 66 }));

    let data = [0x03, 0x05, 0x00, 0x03, b'f', b'o', b'o'];
    match parse_message(&data) {
        Ok(Message::Data { channel: 5, payload }) => {
            println!("Channel 5: {}", String::from_utf8_lossy(&payload));
        }
        _ => panic!("Unexpected parse result"),
    }

    // TLV parsing
    let tlv_data = [
        1, 3, b'a', b'b', b'c',
        2, 2, 0x12, 0x34,
        3, 0,
    ];
    let records = parse_tlv_stream(&tlv_data);
    println!("TLV records: {:?}", records);
}
```

---

## Error Recovery

Robust error handling patterns for parsers and validators.

```rust
#[derive(Debug)]
enum ParseError {
    UnexpectedToken { expected: String, found: String, pos: usize },
    UnexpectedEof,
    InvalidSyntax { message: String, pos: usize },
}

#[derive(Debug)]
enum RecoveryStrategy {
    Skip,
    Insert(String),
    Resync(Vec<String>),
}

struct ErrorRecovery {
    errors: Vec<ParseError>,
    recovery_points: Vec<String>,
}

impl ErrorRecovery {
    fn new(recovery_points: Vec<String>) -> Self {
        ErrorRecovery {
            errors: vec![],
            recovery_points,
        }
    }

    fn recover(&mut self, error: ParseError, tokens: &[String], pos: usize) -> (RecoveryStrategy, usize) {
        self.errors.push(error);

        // Look for recovery point
        for (i, token) in tokens[pos..].iter().enumerate() {
            match (token.as_str(), &self.recovery_points[..]) {
                // Found statement terminator
                (";", _) => return (RecoveryStrategy::Resync(vec![";".into()]), pos + i + 1),

                // Found block boundary
                ("}" | ")", recovery) if recovery.contains(&token.to_string()) => {
                    return (RecoveryStrategy::Resync(vec![token.clone()]), pos + i);
                }

                _ => continue,
            }
        }

        // No recovery point found, skip token
        (RecoveryStrategy::Skip, pos + 1)
    }
}

// Result chaining with error accumulation
#[derive(Debug)]
struct ValidationErrors {
    errors: Vec<String>,
}

impl ValidationErrors {
    fn new() -> Self {
        ValidationErrors { errors: vec![] }
    }

    fn add(&mut self, error: String) {
        self.errors.push(error);
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

fn validate_user(name: &str, email: &str, age: i32) -> Result<(), ValidationErrors> {
    let mut errors = ValidationErrors::new();

    match name {
        "" => errors.add("Name cannot be empty".into()),
        n if n.len() < 2 => errors.add("Name too short".into()),
        n if n.len() > 50 => errors.add("Name too long".into()),
        _ => {}
    }

    match email {
        "" => errors.add("Email cannot be empty".into()),
        e if !e.contains('@') => errors.add("Invalid email format".into()),
        _ => {}
    }

    match age {
        a if a < 0 => errors.add("Age cannot be negative".into()),
        a if a > 150 => errors.add("Age unrealistic".into()),
        _ => {}
    }

    if errors.has_errors() {
        Err(errors)
    } else {
        Ok(())
    }
}

fn main() {
    match validate_user("", "bad-email", -5) {
        Ok(_) => println!("Valid user"),
        Err(errs) => {
            println!("Validation failed:");
            for err in errs.errors {
                println!("  - {}", err);
            }
        }
    }
}
```

---

## Binary Tree Operations

Pattern matching on recursive tree structures.

```rust
#[derive(Debug, Clone)]
enum Tree<T> {
    Empty,
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T: Ord + Clone> Tree<T> {
    fn insert(&mut self, new_value: T) {
        match self {
            Tree::Empty => {
                *self = Tree::Node {
                    value: new_value,
                    left: Box::new(Tree::Empty),
                    right: Box::new(Tree::Empty),
                };
            }
            Tree::Node { value, left, right } => {
                if new_value < *value {
                    left.insert(new_value);
                } else {
                    right.insert(new_value);
                }
            }
        }
    }

    fn contains(&self, search: &T) -> bool {
        match self {
            Tree::Empty => false,
            Tree::Node { value, left, right } => {
                if search == value {
                    true
                } else if search < value {
                    left.contains(search)
                } else {
                    right.contains(search)
                }
            }
        }
    }

    // Pattern-based tree transformations
    fn map<U, F>(&self, f: &F) -> Tree<U>
    where
        F: Fn(&T) -> U,
        U: Ord + Clone,
    {
        match self {
            Tree::Empty => Tree::Empty,
            Tree::Node { value, left, right } => Tree::Node {
                value: f(value),
                left: Box::new(left.map(f)),
                right: Box::new(right.map(f)),
            },
        }
    }

    // AVL-style rotation patterns
    fn rotate_right(self) -> Tree<T> {
        match self {
            Tree::Node {
                value: y_val,
                left: box Tree::Node {
                    value: x_val,
                    left: x_left,
                    right: x_right,
                },
                right: y_right,
            } => Tree::Node {
                value: x_val,
                left: x_left,
                right: Box::new(Tree::Node {
                    value: y_val,
                    left: x_right,
                    right: y_right,
                }),
            },
            tree => tree,
        }
    }

    // Fold operation on trees
    fn fold<B, F>(&self, init: B, f: &F) -> B
    where
        F: Fn(B, &T) -> B,
        B: Clone,
    {
        match self {
            Tree::Empty => init,
            Tree::Node { value, left, right } => {
                let left_result = left.fold(init.clone(), f);
                let mid_result = f(left_result, value);
                right.fold(mid_result, f)
            }
        }
    }
}

fn main() {
    let mut tree = Tree::Empty;
    for val in [5, 3, 7, 1, 9, 4, 6] {
        tree.insert(val);
    }

    println!("Contains 7: {}", tree.contains(&7));
    println!("Contains 10: {}", tree.contains(&10));

    // Transform tree
    let doubled = tree.map(&|x| x * 2);
    println!("Doubled tree contains 14: {}", doubled.contains(&14));

    // Sum all values
    let sum = tree.fold(0, &|acc, x| acc + x);
    println!("Sum of all nodes: {}", sum);
}
```

---

## Graph Traversal

Pattern matching for graph algorithms and path finding.

```rust
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NodeId(usize);

#[derive(Debug)]
struct Graph {
    edges: HashMap<NodeId, Vec<(NodeId, i32)>>, // node -> [(neighbor, weight)]
}

impl Graph {
    fn new() -> Self {
        Graph {
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: NodeId, to: NodeId, weight: i32) {
        self.edges.entry(from).or_insert_with(Vec::new).push((to, weight));
    }

    // BFS with pattern matching
    fn bfs(&self, start: NodeId, goal: NodeId) -> Option<Vec<NodeId>> {
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if current == goal {
                // Reconstruct path
                let mut path = vec![goal];
                let mut node = goal;
                while let Some(&prev) = parent.get(&node) {
                    path.push(prev);
                    node = prev;
                }
                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.edges.get(&current) {
                for &(neighbor, _) in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    // Dijkstra with pattern matching on priority queue
    fn dijkstra(&self, start: NodeId, goal: NodeId) -> Option<(i32, Vec<NodeId>)> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        let mut dist = HashMap::new();
        let mut parent = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push((Reverse(0), start));

        while let Some((Reverse(cost), node)) = heap.pop() {
            if node == goal {
                let mut path = vec![goal];
                let mut current = goal;
                while let Some(&prev) = parent.get(&current) {
                    path.push(prev);
                    current = prev;
                }
                path.reverse();
                return Some((cost, path));
            }

            // Skip if we've found a better path
            match dist.get(&node) {
                Some(&d) if cost > d => continue,
                _ => {}
            }

            if let Some(neighbors) = self.edges.get(&node) {
                for &(neighbor, weight) in neighbors {
                    let next_cost = cost + weight;

                    let should_update = match dist.get(&neighbor) {
                        Some(&current_cost) => next_cost < current_cost,
                        None => true,
                    };

                    if should_update {
                        dist.insert(neighbor, next_cost);
                        parent.insert(neighbor, node);
                        heap.push((Reverse(next_cost), neighbor));
                    }
                }
            }
        }

        None
    }

    // Cycle detection
    fn has_cycle(&self) -> bool {
        #[derive(Clone, Copy, PartialEq)]
        enum Color { White, Gray, Black }

        let mut colors: HashMap<NodeId, Color> = HashMap::new();

        fn visit(
            graph: &Graph,
            node: NodeId,
            colors: &mut HashMap<NodeId, Color>,
        ) -> bool {
            colors.insert(node, Color::Gray);

            if let Some(neighbors) = graph.edges.get(&node) {
                for &(neighbor, _) in neighbors {
                    match colors.get(&neighbor) {
                        Some(Color::Gray) => return true,  // Back edge = cycle
                        Some(Color::Black) => continue,
                        Some(Color::White) | None => {
                            if visit(graph, neighbor, colors) {
                                return true;
                            }
                        }
                    }
                }
            }

            colors.insert(node, Color::Black);
            false
        }

        for &node in self.edges.keys() {
            match colors.get(&node) {
                Some(Color::White) | None => {
                    if visit(self, node, &mut colors) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }
}

fn main() {
    let mut graph = Graph::new();
    graph.add_edge(NodeId(0), NodeId(1), 4);
    graph.add_edge(NodeId(0), NodeId(2), 1);
    graph.add_edge(NodeId(2), NodeId(1), 2);
    graph.add_edge(NodeId(1), NodeId(3), 1);
    graph.add_edge(NodeId(2), NodeId(3), 5);

    if let Some(path) = graph.bfs(NodeId(0), NodeId(3)) {
        println!("BFS path: {:?}", path);
    }

    if let Some((cost, path)) = graph.dijkstra(NodeId(0), NodeId(3)) {
        println!("Shortest path (cost {}): {:?}", cost, path);
    }

    println!("Has cycle: {}", graph.has_cycle());
}
```

---

## Instruction Dispatch

Virtual machine instruction dispatch and evaluation.

```rust
#[derive(Debug, Clone)]
enum Instruction {
    Push(i64),
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Dup,
    Swap,
    Load(usize),   // Load from memory
    Store(usize),  // Store to memory
    Jmp(usize),    // Unconditional jump
    JmpIf(usize),  // Jump if top of stack != 0
    Call(usize),   // Call function
    Ret,           // Return from function
}

struct VM {
    stack: Vec<i64>,
    memory: Vec<i64>,
    pc: usize,           // Program counter
    call_stack: Vec<usize>,
}

impl VM {
    fn new(memory_size: usize) -> Self {
        VM {
            stack: Vec::new(),
            memory: vec![0; memory_size],
            pc: 0,
            call_stack: Vec::new(),
        }
    }

    fn run(&mut self, program: &[Instruction]) -> Result<i64, String> {
        while self.pc < program.len() {
            match &program[self.pc] {
                Instruction::Push(val) => {
                    self.stack.push(*val);
                    self.pc += 1;
                }

                Instruction::Pop => {
                    self.stack.pop()
                        .ok_or("Stack underflow")?;
                    self.pc += 1;
                }

                Instruction::Add => {
                    let (b, a) = self.pop2()?;
                    self.stack.push(a + b);
                    self.pc += 1;
                }

                Instruction::Sub => {
                    let (b, a) = self.pop2()?;
                    self.stack.push(a - b);
                    self.pc += 1;
                }

                Instruction::Mul => {
                    let (b, a) = self.pop2()?;
                    self.stack.push(a * b);
                    self.pc += 1;
                }

                Instruction::Div => {
                    let (b, a) = self.pop2()?;
                    if b == 0 {
                        return Err("Division by zero".into());
                    }
                    self.stack.push(a / b);
                    self.pc += 1;
                }

                Instruction::Dup => {
                    let val = *self.stack.last()
                        .ok_or("Stack underflow")?;
                    self.stack.push(val);
                    self.pc += 1;
                }

                Instruction::Swap => {
                    let len = self.stack.len();
                    if len < 2 {
                        return Err("Stack underflow".into());
                    }
                    self.stack.swap(len - 1, len - 2);
                    self.pc += 1;
                }

                Instruction::Load(addr) => {
                    let val = self.memory.get(*addr)
                        .ok_or("Invalid memory address")?;
                    self.stack.push(*val);
                    self.pc += 1;
                }

                Instruction::Store(addr) => {
                    let val = self.stack.pop()
                        .ok_or("Stack underflow")?;
                    *self.memory.get_mut(*addr)
                        .ok_or("Invalid memory address")? = val;
                    self.pc += 1;
                }

                Instruction::Jmp(addr) => {
                    self.pc = *addr;
                }

                Instruction::JmpIf(addr) => {
                    let condition = self.stack.pop()
                        .ok_or("Stack underflow")?;
                    if condition != 0 {
                        self.pc = *addr;
                    } else {
                        self.pc += 1;
                    }
                }

                Instruction::Call(addr) => {
                    self.call_stack.push(self.pc + 1);
                    self.pc = *addr;
                }

                Instruction::Ret => {
                    self.pc = self.call_stack.pop()
                        .ok_or("Call stack underflow")?;
                }
            }
        }

        self.stack.pop().ok_or("Empty stack".into())
    }

    fn pop2(&mut self) -> Result<(i64, i64), String> {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;
        Ok((b, a))
    }
}

fn main() {
    // Program: compute (5 + 3) * 2
    let program = vec![
        Instruction::Push(5),
        Instruction::Push(3),
        Instruction::Add,
        Instruction::Push(2),
        Instruction::Mul,
    ];

    let mut vm = VM::new(256);
    match vm.run(&program) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Conditional jump example
    let program2 = vec![
        Instruction::Push(10),
        Instruction::Dup,
        Instruction::Push(5),
        Instruction::Sub,
        Instruction::JmpIf(6),  // Jump to Push(1) if difference != 0
        Instruction::Push(0),
        Instruction::Jmp(7),
        Instruction::Push(1),
    ];

    let mut vm2 = VM::new(256);
    match vm2.run(&program2) {
        Ok(result) => println!("Conditional result: {}", result),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## Type-Level State

Compile-time state validation using generics and pattern matching.

```rust
use std::marker::PhantomData;

// Builder pattern with compile-time validation
struct Unset;
struct Set<T>(T);

struct ConfigBuilder<Host, Port, Timeout> {
    host: Host,
    port: Port,
    timeout: Timeout,
}

impl ConfigBuilder<Unset, Unset, Unset> {
    fn new() -> Self {
        ConfigBuilder {
            host: Unset,
            port: Unset,
            timeout: Unset,
        }
    }
}

impl<Port, Timeout> ConfigBuilder<Unset, Port, Timeout> {
    fn host(self, host: String) -> ConfigBuilder<Set<String>, Port, Timeout> {
        ConfigBuilder {
            host: Set(host),
            port: self.port,
            timeout: self.timeout,
        }
    }
}

impl<Host, Timeout> ConfigBuilder<Host, Unset, Timeout> {
    fn port(self, port: u16) -> ConfigBuilder<Host, Set<u16>, Timeout> {
        ConfigBuilder {
            host: self.host,
            port: Set(port),
            timeout: self.timeout,
        }
    }
}

impl<Host, Port> ConfigBuilder<Host, Port, Unset> {
    fn timeout(self, timeout: u64) -> ConfigBuilder<Host, Port, Set<u64>> {
        ConfigBuilder {
            host: self.host,
            port: self.port,
            timeout: Set(timeout),
        }
    }
}

// Only works when all fields are Set
impl ConfigBuilder<Set<String>, Set<u16>, Set<u64>> {
    fn build(self) -> Config {
        Config {
            host: self.host.0,
            port: self.port.0,
            timeout: self.timeout.0,
        }
    }
}

struct Config {
    host: String,
    port: u16,
    timeout: u64,
}

// Type-level state machine for resources
trait ResourceState {}
struct Open;
struct Closed;

impl ResourceState for Open {}
impl ResourceState for Closed {}

struct Resource<S: ResourceState> {
    data: String,
    _state: PhantomData<S>,
}

impl Resource<Closed> {
    fn new(data: String) -> Self {
        Resource {
            data,
            _state: PhantomData,
        }
    }

    fn open(self) -> Resource<Open> {
        println!("Opening resource");
        Resource {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Resource<Open> {
    fn read(&self) -> &str {
        &self.data
    }

    fn write(&mut self, data: String) {
        self.data = data;
    }

    fn close(self) -> Resource<Closed> {
        println!("Closing resource");
        Resource {
            data: self.data,
            _state: PhantomData,
        }
    }
}

fn main() {
    // Builder enforces all fields are set at compile time
    let config = ConfigBuilder::new()
        .host("localhost".into())
        .port(8080)
        .timeout(30)
        .build();

    println!("Config: {}:{}", config.host, config.port);

    // Resource state transitions enforced at compile time
    let resource = Resource::new("data".into());
    // resource.read(); // Compile error: can't read closed resource

    let mut resource = resource.open();
    println!("Data: {}", resource.read());
    resource.write("new data".into());

    let _resource = resource.close();
    // _resource.read(); // Compile error: can't read closed resource
}
```

---

## Iterator Patterns

Advanced iterator patterns using pattern matching.

```rust
// Window iterator with pattern matching
fn find_pattern<T: PartialEq>(data: &[T], pattern: &[T]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}

// State machine iterator
struct StateMachine {
    state: i32,
}

impl Iterator for StateMachine {
    type Item = &'static str;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_state, output) = match self.state {
            0 => (1, Some("Start")),
            1 => (2, Some("Processing")),
            2 if rand::random::<bool>() => (2, Some("Still processing")),
            2 => (3, Some("Done")),
            3 => (3, None),
            _ => (3, None),
        };
        self.state = next_state;
        output
    }
}

// Peekable with 2-token lookahead
struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a [u8]) -> Self {
        Lexer { input, pos: 0 }
    }

    fn peek2(&self) -> Option<(u8, Option<u8>)> {
        match (self.input.get(self.pos), self.input.get(self.pos + 1)) {
            (Some(&a), Some(&b)) => Some((a, Some(b))),
            (Some(&a), None) => Some((a, None)),
            (None, _) => None,
        }
    }

    fn next_token(&mut self) -> Option<String> {
        match self.peek2()? {
            // Two-character operators
            (b'=', Some(b'=')) => {
                self.pos += 2;
                Some("==".into())
            }
            (b'!', Some(b'=')) => {
                self.pos += 2;
                Some("!=".into())
            }
            (b'<', Some(b'=')) => {
                self.pos += 2;
                Some("<=".into())
            }
            (b'>', Some(b'=')) => {
                self.pos += 2;
                Some(">=".into())
            }

            // Single character
            (ch, _) if ch.is_ascii_alphanumeric() => {
                let start = self.pos;
                while self.input.get(self.pos).map_or(false, |c| c.is_ascii_alphanumeric()) {
                    self.pos += 1;
                }
                Some(String::from_utf8_lossy(&self.input[start..self.pos]).into())
            }

            (ch, _) => {
                self.pos += 1;
                Some((ch as char).to_string())
            }
        }
    }
}

// Chunking with predicate
fn chunk_by<T, F>(data: &[T], mut predicate: F) -> Vec<&[T]>
where
    F: FnMut(&T) -> bool,
{
    let mut chunks = vec![];
    let mut start = 0;

    for (i, item) in data.iter().enumerate() {
        if predicate(item) {
            if start < i {
                chunks.push(&data[start..i]);
            }
            start = i + 1;
        }
    }

    if start < data.len() {
        chunks.push(&data[start..]);
    }

    chunks
}

fn main() {
    // Pattern finding
    let data = vec![1, 2, 3, 4, 5, 3, 4, 6];
    let pattern = vec![3, 4];
    if let Some(pos) = find_pattern(&data, &pattern) {
        println!("Pattern found at position: {}", pos);
    }

    // Lexer with lookahead
    let input = b"x == 10";
    let mut lexer = Lexer::new(input);
    let mut tokens = vec![];
    while let Some(token) = lexer.next_token() {
        if !token.trim().is_empty() {
            tokens.push(token);
        }
    }
    println!("Tokens: {:?}", tokens);

    // Chunking by predicate
    let numbers = vec![1, 2, 0, 3, 4, 0, 5];
    let chunks = chunk_by(&numbers, |&x| x == 0);
    println!("Chunks: {:?}", chunks);
}
```

---

## Summary

Pattern matching in Rust enables:

- **Parser Combinators**: Clean tokenization and recursive descent parsing
- **AST Transformations**: Compiler passes with exhaustive optimization
- **State Machines**: Type-safe state transitions at compile or runtime
- **Protocol Parsing**: Efficient binary format handling with slice patterns
- **Error Recovery**: Robust error accumulation and recovery strategies
- **Tree Operations**: Elegant recursive data structure manipulation
- **Graph Algorithms**: BFS, DFS, Dijkstra with pattern-driven logic
- **VM Dispatch**: Zero-cost instruction interpretation
- **Type-Level State**: Compile-time validation of state transitions
- **Iterator Patterns**: Advanced stream processing and lookahead

Master these patterns for systems programming, compilers, parsers, and high-performance applications.
