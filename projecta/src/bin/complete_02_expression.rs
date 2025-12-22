// Complete Expression Parser with Arena Allocation
// Implements all 7 milestones from the project specification

use std::cell::RefCell;
use std::mem;
use std::time::Instant;

// ============================================================================
// Common Types
// ============================================================================

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpType {
    Add,
    Sub,
    Mul,
    Div,
}

impl OpType {
    pub fn eval(&self, left: i64, right: i64) -> Result<i64, String> {
        match self {
            OpType::Add => Ok(left + right),
            OpType::Sub => Ok(left - right),
            OpType::Mul => Ok(left * right),
            OpType::Div => {
                if right == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left / right)
                }
            }
        }
    }
}

// ============================================================================
// Milestone 1: Define AST Types
// ============================================================================

#[derive(Debug, PartialEq)]
pub enum Expr<'arena> {
    Literal(i64),
    BinOp {
        op: OpType,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    },
}

impl<'arena> Expr<'arena> {
    pub fn eval(&self) -> Result<i64, String> {
        match self {
            Expr::Literal(n) => Ok(*n),
            Expr::BinOp { op, left, right } => {
                let left_val = left.eval()?;
                let right_val = right.eval()?;
                op.eval(left_val, right_val)
            }
        }
    }
}

// ============================================================================
// Milestone 2: Box-Based Expression Trees
// ============================================================================

#[derive(Debug, PartialEq)]
pub enum BoxExpr {
    Literal(i64),
    BinOp {
        op: OpType,
        left: Box<BoxExpr>,
        right: Box<BoxExpr>,
    },
}

impl BoxExpr {
    pub fn eval(&self) -> Result<i64, String> {
        match self {
            BoxExpr::Literal(n) => Ok(*n),
            BoxExpr::BinOp { op, left, right } => {
                let left_val = left.eval()?;
                let right_val = right.eval()?;
                op.eval(left_val, right_val)
            }
        }
    }
}

pub struct BoxExprBuilder;

impl BoxExprBuilder {
    pub fn literal(n: i64) -> Box<BoxExpr> {
        Box::new(BoxExpr::Literal(n))
    }

    pub fn binary(op: OpType, left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        Box::new(BoxExpr::BinOp { op, left, right })
    }

    pub fn add(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        Self::binary(OpType::Add, left, right)
    }

    pub fn sub(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        Self::binary(OpType::Sub, left, right)
    }

    pub fn mul(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        Self::binary(OpType::Mul, left, right)
    }

    pub fn div(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        Self::binary(OpType::Div, left, right)
    }
}

// ============================================================================
// Milestone 3: Simple Bump Allocator (Arena)
// ============================================================================

pub struct Arena {
    storage: RefCell<Vec<u8>>,
}

impl Arena {
    pub fn new_with_capacity(capacity: usize) -> Self {
        Arena {
            storage: RefCell::new(Vec::with_capacity(capacity)),
        }
    }

    pub fn alloc<'arena, T>(&'arena self, value: T) -> &'arena T {
        let mut storage = self.storage.borrow_mut();

        // Calculate size and alignment
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();

        // Get current position
        let current_len = storage.len();

        // Calculate aligned position
        let padding = (align - (current_len % align)) % align;
        let start = current_len + padding;

        // Ensure we have space
        storage.resize(start + size, 0);

        // Get pointer to allocated space
        let ptr = &mut storage[start] as *mut u8 as *mut T;

        unsafe {
            // Write value to allocated space
            ptr.write(value);
            // Return reference with arena lifetime
            &*ptr
        }
    }

    /// Reset the arena for reuse without deallocating underlying memory.
    /// SAFETY: Caller must ensure no references to arena-allocated data exist.
    pub fn reset(&self) {
        let mut storage = self.storage.borrow_mut();
        // Clear length but keep capacity - no reallocation needed
        storage.clear();
    }

    pub fn bytes_used(&self) -> usize {
        self.storage.borrow().len()
    }
}

// ============================================================================
// Milestone 4: Build Expressions in Arena
// ============================================================================

pub struct ExprBuilder<'arena> {
    arena: &'arena Arena,
}

impl<'arena> ExprBuilder<'arena> {
    pub fn new(arena: &'arena Arena) -> Self {
        ExprBuilder { arena }
    }

    pub fn literal(&self, n: i64) -> &'arena Expr<'arena> {
        self.arena.alloc(Expr::Literal(n))
    }

    pub fn binary(
        &self,
        op: OpType,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        self.arena.alloc(Expr::BinOp { op, left, right })
    }

    pub fn add(
        &self,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        self.binary(OpType::Add, left, right)
    }

    pub fn sub(
        &self,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        self.binary(OpType::Sub, left, right)
    }

    pub fn mul(
        &self,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        self.binary(OpType::Mul, left, right)
    }

    pub fn div(
        &self,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        self.binary(OpType::Div, left, right)
    }
}

// ============================================================================
// Milestone 5: Lexer (Tokenizer)
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Number(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    End,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> i64 {
        let mut num = 0;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num = num * 10 + (c as i64 - '0' as i64);
                self.advance();
            } else {
                break;
            }
        }
        num
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::End),
            Some(c) => match c {
                '0'..='9' => Ok(Token::Number(self.read_number())),
                '+' => {
                    self.advance();
                    Ok(Token::Plus)
                }
                '-' => {
                    self.advance();
                    Ok(Token::Minus)
                }
                '*' => {
                    self.advance();
                    Ok(Token::Star)
                }
                '/' => {
                    self.advance();
                    Ok(Token::Slash)
                }
                '(' => {
                    self.advance();
                    Ok(Token::LeftParen)
                }
                ')' => {
                    self.advance();
                    Ok(Token::RightParen)
                }
                _ => Err(format!("Unexpected character '{}'", c)),
            },
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            tokens.push(token.clone());
            if token == Token::End {
                break;
            }
        }
        Ok(tokens)
    }
}

// ============================================================================
// Milestone 6: Recursive Descent Parser
// ============================================================================

pub struct Parser<'arena> {
    tokens: Vec<Token>,
    position: usize,
    builder: ExprBuilder<'arena>,
}

impl<'arena> Parser<'arena> {
    pub fn new(tokens: Vec<Token>, arena: &'arena Arena) -> Self {
        Parser {
            tokens,
            position: 0,
            builder: ExprBuilder::new(arena),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::End)
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if self.peek() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.peek()))
        }
    }

    // Factor → Number | '(' Expr ')'
    fn parse_factor(&mut self) -> Result<&'arena Expr<'arena>, String> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(self.builder.literal(n))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            token => Err(format!("Expected number or '(', found {:?}", token)),
        }
    }

    // Term → Factor (('*' | '/') Factor)*
    fn parse_term(&mut self) -> Result<&'arena Expr<'arena>, String> {
        let mut left = self.parse_factor()?;

        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = self.builder.mul(left, right);
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left = self.builder.div(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // Expr → Term (('+' | '-') Term)*
    fn parse_expr(&mut self) -> Result<&'arena Expr<'arena>, String> {
        let mut left = self.parse_term()?;

        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = self.builder.add(left, right);
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = self.builder.sub(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    pub fn parse(&mut self) -> Result<&'arena Expr<'arena>, String> {
        let expr = self.parse_expr()?;
        if self.peek() != &Token::End {
            return Err(format!("Unexpected token: {:?}", self.peek()));
        }
        Ok(expr)
    }
}

// Helper function for parsing and evaluating
pub fn parse_and_eval(input: &str) -> Result<i64, String> {
    let arena = Arena::new_with_capacity(4 * 1024);
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, &arena);
    let expr = parser.parse()?;
    expr.eval()
}

// ============================================================================
// Milestone 7: Performance Comparison
// ============================================================================

fn benchmark_arena() -> std::time::Duration {
    // Create ONE arena outside the loop - this is how arenas should be used
    let arena = Arena::new_with_capacity(64 * 1024 * 1024); // 64MB pre-allocated
    let start = Instant::now();
    for _ in 0..1000000 {
        let builder = ExprBuilder::new(&arena);
        // Build expression: (1+2)*(3+4)+(5-2)*7
        let expr = builder.add(
            builder.mul(
                builder.add(builder.literal(1), builder.literal(2)),
                builder.add(builder.literal(3), builder.literal(4)),
            ),
            builder.mul(
                builder.sub(builder.literal(5), builder.literal(2)),
                builder.literal(7),
            ),
        );
        let _ = expr.eval();
    }
    start.elapsed()
    // Single deallocation when arena drops
}

fn benchmark_box() -> std::time::Duration {
    let start = Instant::now();
    for _ in 0..1000000 {
        // Build same expression with Box
        let expr = BoxExprBuilder::add(
            BoxExprBuilder::mul(
                BoxExprBuilder::add(BoxExprBuilder::literal(1), BoxExprBuilder::literal(2)),
                BoxExprBuilder::add(BoxExprBuilder::literal(3), BoxExprBuilder::literal(4)),
            ),
            BoxExprBuilder::mul(
                BoxExprBuilder::sub(BoxExprBuilder::literal(5), BoxExprBuilder::literal(2)),
                BoxExprBuilder::literal(7),
            ),
        );
        let _ = expr.eval();
    }
    start.elapsed()
}


fn benchmark_bulk_deallocation() {
    const ITERATIONS: usize = 10_000;
    const TREE_DEPTH: usize = 10;  // 2^10 - 1 = 2047 nodes per tree

    println!("\n=== Bulk Deallocation Benchmark ===");
    println!("Building {} trees with {} nodes each\n", ITERATIONS, (1 << (TREE_DEPTH + 1)) - 1);

    // Box version: must free each node individually
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        fn build_box_tree(depth: usize) -> Box<BoxExpr> {
            if depth == 0 {
                BoxExprBuilder::literal(1)
            } else {
                BoxExprBuilder::add(
                    build_box_tree(depth - 1),
                    build_box_tree(depth - 1),
                )
            }
        }
        let tree = build_box_tree(TREE_DEPTH);
        let _ = tree.eval();
        // Drop happens here: ~2047 individual free() calls per tree
    }
    let box_time = start.elapsed();

    // Arena version with REUSE: pre-allocate once, reset each iteration
    // This is the proper way to use arenas
    let arena = Arena::new_with_capacity(256 * 1024); // Pre-allocate 256KB once
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let builder = ExprBuilder::new(&arena);

        fn build_arena_tree<'a>(
            builder: &ExprBuilder<'a>,
            depth: usize
        ) -> &'a Expr<'a> {
            if depth == 0 {
                builder.literal(1)
            } else {
                builder.add(
                    build_arena_tree(builder, depth - 1),
                    build_arena_tree(builder, depth - 1),
                )
            }
        }
        let tree = build_arena_tree(&builder, TREE_DEPTH);
        let _ = tree.eval();
        // Reset arena for next iteration - O(1) operation, no free() calls
        arena.reset();
    }
    let arena_reuse_time = start.elapsed();

    println!("Box (alloc + {} deallocations/tree): {:?}", (1 << (TREE_DEPTH + 1)) - 1, box_time);
    println!("Arena (reused, O(1) reset):          {:?}", arena_reuse_time);

    if arena_reuse_time.as_nanos() > 0 {
        let speedup = box_time.as_nanos() as f64 / arena_reuse_time.as_nanos() as f64;
        println!("Arena speedup: {:.2}x faster", speedup);
    }
    println!("===================================\n");
}

pub fn run_benchmarks() {
    println!("\n=== Performance Comparison: Box vs Arena ===");
    let box_duration = benchmark_box();
    let arena_duration = benchmark_arena();
    println!("Box allocation   : {:?}", box_duration);
    println!("Arena allocation : {:?}", arena_duration);
    if arena_duration.as_nanos() > 0 {
        let factor = box_duration.as_millis() as f64 / arena_duration.as_millis() as f64;
        println!("Arena speedup    : {:.2}x faster", factor);
    }
    println!("============================================\n");
}

// ============================================================================
// Main Function - Demonstrates All Milestones
// ============================================================================

fn main() {
    println!("=== Expression Parser with Arena Allocation ===\n");

    // Milestone 1: Basic AST evaluation
    println!("--- Milestone 1: Define AST Types ---");
    let two = Expr::Literal(2);
    let three = Expr::Literal(3);
    let add = Expr::BinOp {
        op: OpType::Add,
        left: &two,
        right: &three,
    };
    println!("2 + 3 = {:?}", add.eval());

    // Milestone 2: Box-based expressions
    println!("\n--- Milestone 2: Box-Based Expression Trees ---");
    let expr = BoxExprBuilder::mul(
        BoxExprBuilder::add(BoxExprBuilder::literal(2), BoxExprBuilder::literal(3)),
        BoxExprBuilder::literal(4),
    );
    println!("(2 + 3) * 4 = {:?}", expr.eval());

    // Milestone 3 & 4: Arena allocation
    println!("\n--- Milestone 3 & 4: Arena Allocation ---");
    let arena = Arena::new_with_capacity(4 * 1024);
    let builder = ExprBuilder::new(&arena);
    let expr = builder.mul(
        builder.add(builder.literal(2), builder.literal(3)),
        builder.literal(4),
    );
    println!("(2 + 3) * 4 = {:?}", expr.eval());

    // Milestone 5: Lexer
    println!("\n--- Milestone 5: Lexer (Tokenizer) ---");
    let mut lexer = Lexer::new("(2 + 3) * 4");
    let tokens = lexer.tokenize().unwrap();
    println!("Input: \"(2 + 3) * 4\"");
    println!("Tokens: {:?}", tokens);

    // Milestone 6: Parser
    println!("\n--- Milestone 6: Recursive Descent Parser ---");
    let test_cases = vec![
        "42",
        "2 + 3",
        "2 + 3 * 4",
        "(2 + 3) * 4",
        "10 - 5 * 2 + 8 / 4",
        "((1 + 2) * (3 + 4)) / (5 - 2)",
    ];

    for input in test_cases {
        match parse_and_eval(input) {
            Ok(result) => println!("{:<30} = {}", input, result),
            Err(e) => println!("{:<30} ERROR: {}", input, e),
        }
    }

    // Test error cases
    println!("\nError handling:");
    let error_cases = vec!["2 + + 3", "(2 + 3", "2 / 0"];
    for input in error_cases {
        match parse_and_eval(input) {
            Ok(result) => println!("{:<30} = {}", input, result),
            Err(e) => println!("{:<30} ERROR: {}", input, e),
        }
    }

    // Milestone 7: Performance comparison
    println!("\n--- Milestone 7: Performance Comparison ---");
    run_benchmarks();
    benchmark_bulk_deallocation();

    println!("=== All Milestones Complete! ===");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 Tests
    #[test]
    fn test_literal_eval() {
        let expr = Expr::Literal(42);
        assert_eq!(expr.eval(), Ok(42));
    }

    #[test]
    fn test_binop_eval() {
        let left = Expr::Literal(10);
        let right = Expr::Literal(5);
        let expr = Expr::BinOp {
            op: OpType::Add,
            left: &left,
            right: &right,
        };
        assert_eq!(expr.eval(), Ok(15));
    }

    #[test]
    fn test_nested_eval() {
        let two = Expr::Literal(2);
        let three = Expr::Literal(3);
        let four = Expr::Literal(4);
        let add = Expr::BinOp {
            op: OpType::Add,
            left: &two,
            right: &three,
        };
        let mul = Expr::BinOp {
            op: OpType::Mul,
            left: &add,
            right: &four,
        };
        assert_eq!(mul.eval(), Ok(20));
    }

    #[test]
    fn test_division_by_zero() {
        let ten = Expr::Literal(10);
        let zero = Expr::Literal(0);
        let expr = Expr::BinOp {
            op: OpType::Div,
            left: &ten,
            right: &zero,
        };
        assert!(expr.eval().is_err());
    }

    // Milestone 2 Tests
    #[test]
    fn test_box_expr_literal() {
        let expr = BoxExprBuilder::literal(42);
        assert_eq!(expr.eval(), Ok(42));
    }

    #[test]
    fn test_box_expr_addition() {
        let expr = BoxExprBuilder::add(
            BoxExprBuilder::literal(10),
            BoxExprBuilder::literal(5),
        );
        assert_eq!(expr.eval(), Ok(15));
    }

    #[test]
    fn test_box_expr_nested() {
        let expr = BoxExprBuilder::mul(
            BoxExprBuilder::add(BoxExprBuilder::literal(2), BoxExprBuilder::literal(3)),
            BoxExprBuilder::literal(4),
        );
        assert_eq!(expr.eval(), Ok(20));
    }

    #[test]
    fn test_box_expr_complex() {
        let expr = BoxExprBuilder::add(
            BoxExprBuilder::mul(
                BoxExprBuilder::sub(BoxExprBuilder::literal(10), BoxExprBuilder::literal(5)),
                BoxExprBuilder::literal(2),
            ),
            BoxExprBuilder::div(BoxExprBuilder::literal(8), BoxExprBuilder::literal(4)),
        );
        assert_eq!(expr.eval(), Ok(12));
    }

    // Milestone 3 Tests
    #[test]
    fn test_arena_alloc_int() {
        let arena = Arena::new_with_capacity(4 * 1024);
        let x = arena.alloc(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_arena_multiple_allocs() {
        let arena = Arena::new_with_capacity(4 * 1024);
        let x = arena.alloc(1);
        let y = arena.alloc(2);
        let z = arena.alloc(3);
        assert_eq!(*x, 1);
        assert_eq!(*y, 2);
        assert_eq!(*z, 3);
    }

    #[test]
    fn test_arena_alloc_string() {
        let arena = Arena::new_with_capacity( 4* 1024);
        let s = arena.alloc(String::from("hello"));
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_arena_alignment() {
        let arena = Arena::new_with_capacity(4 * 1024);
        let _byte = arena.alloc(1u8);
        let num = arena.alloc(1234u64);
        let ptr = num as *const u64 as usize;
        assert_eq!(ptr % 8, 0, "u64 should be 8-byte aligned");
    }

    // Milestone 4 Tests
    #[test]
    fn test_builder() {
        let arena = Arena::new_with_capacity(4 * 1024);
        let builder = ExprBuilder::new(&arena);
        let two = builder.literal(2);
        let three = builder.literal(3);
        let four = builder.literal(4);
        let sum = builder.add(two, three);
        let product = builder.mul(sum, four);
        assert_eq!(product.eval(), Ok(20));
    }

    #[test]
    fn test_complex_expression() {
        let arena = Arena::new_with_capacity(4 * 1024);
        let builder = ExprBuilder::new(&arena);
        let expr = builder.add(
            builder.mul(
                builder.sub(builder.literal(10), builder.literal(5)),
                builder.literal(2),
            ),
            builder.div(builder.literal(8), builder.literal(4)),
        );
        assert_eq!(expr.eval(), Ok(12));
    }

    // Milestone 5 Tests
    #[test]
    fn test_lexer_numbers() {
        let mut lexer = Lexer::new("123 456");
        assert_eq!(lexer.next_token(), Ok(Token::Number(123)));
        assert_eq!(lexer.next_token(), Ok(Token::Number(456)));
        assert_eq!(lexer.next_token(), Ok(Token::End));
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("+ - * /");
        assert_eq!(lexer.next_token(), Ok(Token::Plus));
        assert_eq!(lexer.next_token(), Ok(Token::Minus));
        assert_eq!(lexer.next_token(), Ok(Token::Star));
        assert_eq!(lexer.next_token(), Ok(Token::Slash));
    }

    #[test]
    fn test_lexer_expression() {
        let mut lexer = Lexer::new("(2 + 3) * 4");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Number(2),
                Token::Plus,
                Token::Number(3),
                Token::RightParen,
                Token::Star,
                Token::Number(4),
                Token::End,
            ]
        );
    }

    #[test]
    fn test_lexer_error() {
        let mut lexer = Lexer::new("2 & 3");
        assert!(lexer.tokenize().is_err());
    }

    // Milestone 6 Tests
    #[test]
    fn test_parse_number() {
        assert_eq!(parse_and_eval("42"), Ok(42));
    }

    #[test]
    fn test_parse_addition() {
        assert_eq!(parse_and_eval("2 + 3"), Ok(5));
    }

    #[test]
    fn test_parse_precedence() {
        assert_eq!(parse_and_eval("2 + 3 * 4"), Ok(14));
    }

    #[test]
    fn test_parse_parentheses() {
        assert_eq!(parse_and_eval("(2 + 3) * 4"), Ok(20));
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse_and_eval("(10 - 5) * 2 + 8 / 4"), Ok(12));
    }

    #[test]
    fn test_parse_nested() {
        assert_eq!(parse_and_eval("((1 + 2) * (3 + 4)) / (5 - 2)"), Ok(7));
    }

    #[test]
    fn test_parse_error() {
        assert!(parse_and_eval("2 + + 3").is_err());
        assert!(parse_and_eval("(2 + 3").is_err());
    }

    #[test]
    fn test_division_by_zero_parse() {
        assert!(parse_and_eval("10 / 0").is_err());
    }
}
