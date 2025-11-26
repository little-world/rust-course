## Project 2: Arena-Based Expression Parser

### Problem Statement

Build a parser for arithmetic expressions that uses arena (bump) allocation. This demonstrates how arena allocation can dramatically speed up programs that create many small objects.

### Why It Matters

**Real-World Impact**: Compilers, parsers, and interpreters allocate millions of small objects (AST nodes, tokens, symbols). Traditional `malloc`/`free` becomes a bottleneck:

**Performance Disaster with Box<T>**:
- Parsing 10,000 expressions with `Box<Expr>`: Each node = 1 malloc call
- Expression `(1+2)*(3+4)` = 7 nodes = 7 malloc calls
- 10,000 expressions × 7 nodes average = **70,000 allocations**
- Each malloc: ~50-100ns (involves locks, metadata, fragmentation)
- Total time: 70,000 × 75ns = **5.25ms just for allocation**

**Arena Allocation Solution**:
- Pre-allocate 4KB chunk, bump pointer for each node
- Per-allocation cost: **~2-5ns** (pointer increment + write)
- Same 70,000 nodes: 70,000 × 3ns = **0.21ms** for allocation
- **25x faster allocation**, plus better cache locality

**Real Production Examples**:
- **Rust compiler**: Uses arenas for AST, HIR, MIR. Parsing 1M LOC project creates ~10M AST nodes in seconds.
- **V8 JavaScript**: Zone allocation (arena) for parser—parsed millions of nodes per second.
- **LLVM**: BumpPtrAllocator for IR nodes, symbol tables.
- **Databases**: Query plan nodes allocated in per-query arenas.

### Use Cases

**When you need this pattern**:
1. **Compiler frontends**: Lexer tokens, AST nodes, symbol table entries
2. **Web request handlers**: Per-request temporary objects (template AST, JSON parsing)
3. **Game engines**: Per-frame allocations (particle systems, AI pathfinding nodes)
4. **Database query execution**: Query plan nodes, temporary expression trees
5. **Text editors**: Syntax tree for incremental parsing
6. **JSON/XML parsers**: DOM nodes, parsing state

**Key characteristic**: Objects have the same lifetime—allocate many, free all at once.

**Counter-examples** (DON'T use arenas):
- Long-lived objects with individual lifetimes
- Objects that need to outlive the arena
- Memory that needs individual deallocation

### Learning Goals

- Understand arena/bump allocation and when it's appropriate
- Work with lifetimes in AST structures (`'arena` lifetime)
- Experience 10-100x allocation speedup
- Practice recursive descent parsing
- Understand memory layout and alignment requirements

---

### Milestone 1: Define AST Types

**Goal**: Create the expression tree data structures that represent arithmetic expressions.

**Your Task**:

Design and implement two types:

1. **Expression Type** (`Expr`):
   - Should represent either a literal number or a binary operation
   - For binary operations, needs to store: the operator type and references to left and right sub-expressions
   - Should use an `'arena` lifetime parameter for references
   - Must derive `Debug` and `PartialEq` for testing

2. **Operator Type** (`OpType`):
   - Should represent the four arithmetic operations: addition, subtraction, multiplication, and division
   - Should be `Copy` since it's just an enum with no data
   - Must derive `Debug`, `PartialEq`, `Clone`, and `Copy`

3. **Evaluation**:
   - Implement an `eval()` method on `OpType` that takes two numbers and returns the result
   - Handle division by zero by returning a `Result<i64, String>`
   - Implement an `eval()` method on `Expr` that recursively evaluates the expression tree
   - For binary operations, evaluate both sides first, then apply the operator

**Design Hints**:
- Think about what variants your `Expr` enum needs
- Consider what data each variant should hold
- Remember that references in the tree need a lifetime annotation
- Binary operations need to store three pieces of information

**Implementation Hints**:
- Use pattern matching to handle different expression types
- For recursive evaluation, use the `?` operator to propagate errors
- Return appropriate error messages for invalid operations



**Checkpoint Tests**:
```rust
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
    // (2 + 3) * 4 = 20
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
```

**Check Your Understanding**:
- What does the `'arena` lifetime mean?
- Why do we use `&'arena Expr<'arena>` instead of `Box<Expr>`?
- How does the recursive `eval()` work?


**Solution**:
```rust
#[derive(Debug, PartialEq)]
enum Expr<'arena> {
    Literal(i64),
    BinOp {
        op: OpType,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    },
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum OpType {
    Add,
    Sub,
    Mul,
    Div,
}

impl OpType {
    fn eval(&self, left: i64, right: i64) -> Result<i64, String> {
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
```

**Add evaluation**:
```rust
impl<'arena> Expr<'arena> {
    fn eval(&self) -> Result<i64, String> {
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
```

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: We've defined the types, but how do we actually create these AST nodes? Using stack allocation limits us to small, fixed-size trees. We need heap allocation.

**Traditional approach** (`Box<Expr>`):
- Each node: separate heap allocation
- For expression `(1+2)*(3+4)`: 7 allocations, 7 deallocations
- Allocation overhead dominates parsing time

**What we're adding**: **Arena allocator** - bump allocation strategy:
1. Allocate large chunk (4KB)
2. For each object: bump pointer forward, return reference
3. Deallocate entire chunk at once when done

**Improvements**:
- **Speed**: Allocation is pointer increment (~3ns) vs malloc (~75ns) = **25x faster**
- **Memory**: Better cache locality (nodes allocated sequentially)
- **Simplicity**: No individual frees—drop arena, free everything
- **Alignment**: Must handle properly (u8 at any address, u64 needs 8-byte alignment)

**Complexity trade-off**: Can't free individual objects. Only works when all objects have same lifetime.

---

### Milestone 2: Simple Bump Allocator

**Goal**: Implement a basic arena that can allocate objects.

**Starter Code**:
```rust
use std::cell::RefCell;
use std::ptr::NonNull;

struct Arena {
    storage: RefCell<Vec<u8>>,
}

impl Arena {
    fn new() -> Self {
        Arena {
            storage: RefCell::new(Vec::with_capacity(4096)),
        }
    }

    fn alloc<T>(&self, value: T) -> &mut T {
        let mut storage = self.storage.borrow_mut();

        // TODO: Calculate size and alignment using std::mem functions
        let size = todo!("Get size of T");
        let align = todo!("Get alignment of T");

        // TODO: Get current position in storage
        let current_len = todo!();

        // TODO: Calculate aligned position
        // Hint: padding = (align - (current_len % align)) % align
        let padding = todo!();
        let start = todo!("current_len + padding");

        // TODO: Ensure we have space in storage
        // Hint: Use storage.resize(start + size, 0)
        todo!();

        // TODO: Get pointer to allocated space
        // Hint: &mut storage[start] as *mut u8 as *mut T
        let ptr = todo!();

        unsafe {
            // TODO: Write value to allocated space using std::ptr::write
            todo!();
            // TODO: Return mutable reference with arena lifetime
            todo!()
        }
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_arena_alloc_int() {
    let arena = Arena::new();
    let x = arena.alloc(42);
    assert_eq!(*x, 42);

    *x = 100;
    assert_eq!(*x, 100);
}

#[test]
fn test_arena_multiple_allocs() {
    let arena = Arena::new();
    let x = arena.alloc(1);
    let y = arena.alloc(2);
    let z = arena.alloc(3);

    assert_eq!(*x, 1);
    assert_eq!(*y, 2);
    assert_eq!(*z, 3);
}

#[test]
fn test_arena_alloc_string() {
    let arena = Arena::new();
    let s = arena.alloc(String::from("hello"));
    assert_eq!(s, "hello");
}

#[test]
fn test_arena_alignment() {
    let arena = Arena::new();
    let _byte = arena.alloc(1u8);
    let num = arena.alloc(1234u64);  // Needs 8-byte alignment

    let ptr = num as *const u64 as usize;
    assert_eq!(ptr % 8, 0, "u64 should be 8-byte aligned");
}
```

**Check Your Understanding**:
- Why do we need alignment?
- What does `std::ptr::write` do?
- Why is the function marked `unsafe`?
- What lifetime does the returned reference have?

---

### Milestone 3: Build Expressions in Arena

**Goal**: Use arena to create expression trees.

**Starter Code**:
```rust
struct ExprBuilder<'arena> {
    arena: &'arena Arena,
}

impl<'arena> ExprBuilder<'arena> {
    fn new(arena: &'arena Arena) -> Self {
        // TODO: Create ExprBuilder with reference to arena
        todo!()
    }

    fn literal(&self, n: i64) -> &'arena Expr<'arena> {
        // TODO: Allocate Expr::Literal(n) in arena and return reference
        todo!()
    }

    fn binary(
        &self,
        op: OpType,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        // TODO: Allocate Expr::BinOp in arena with given op, left, right
        todo!()
    }

    fn add(
        &self,
        left: &'arena Expr<'arena>,
        right: &'arena Expr<'arena>,
    ) -> &'arena Expr<'arena> {
        // TODO: Call binary() with OpType::Add
        todo!()
    }

    // TODO: Add methods for sub, mul, div following the same pattern
    fn sub(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
        todo!()
    }

    fn mul(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
        todo!()
    }

    fn div(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_builder() {
    let arena = Arena::new();
    let builder = ExprBuilder::new(&arena);

    // Build: (2 + 3) * 4
    let two = builder.literal(2);
    let three = builder.literal(3);
    let four = builder.literal(4);

    let sum = builder.add(two, three);
    let product = builder.mul(sum, four);

    assert_eq!(product.eval(), Ok(20));
}

#[test]
fn test_complex_expression() {
    let arena = Arena::new();
    let builder = ExprBuilder::new(&arena);

    // Build: ((10 - 5) * 2) + (8 / 4)
    let expr = builder.add(
        builder.mul(
            builder.sub(builder.literal(10), builder.literal(5)),
            builder.literal(2)
        ),
        builder.div(builder.literal(8), builder.literal(4))
    );

    assert_eq!(expr.eval(), Ok(12)); // (5 * 2) + 2 = 12
}
```

**Check Your Understanding**:
- Why does the builder need a reference to the arena?
- Can expressions outlive the arena?
- How many heap allocations happen for a 3-node tree?

---

### Milestone 4: Lexer (Tokenizer)

**Goal**: Break input string into tokens.

**Starter Code**:
```rust
#[derive(Debug, PartialEq, Clone)]
enum Token {
    Number(i64),
    Plus,
    Minus,
    Star,
    Slash,
    LeftParen,
    RightParen,
    End,
}

struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    fn new(input: &str) -> Self {
        // TODO: Create Lexer with input converted to Vec<char> and position 0
        todo!()
    }

    fn peek(&self) -> Option<char> {
        // TODO: Return the character at current position (or None if at end)
        // Hint: self.input.get(self.position).copied()
        todo!()
    }

    fn advance(&mut self) {
        // TODO: Increment position by 1
        todo!()
    }

    fn skip_whitespace(&mut self) {
        // TODO: Loop while current character is whitespace
        // Hint: Use peek() and ch.is_whitespace(), call advance() for each whitespace
        todo!()
    }

    fn read_number(&mut self) -> i64 {
        // TODO: Build up a number by reading consecutive digits
        // Hint: Start with num = 0, for each digit: num = num * 10 + digit_value
        // Use ch.is_ascii_digit() to check, convert with (ch as i64 - '0' as i64)
        todo!()
    }

    fn next_token(&mut self) -> Result<Token, String> {
        // TODO: Skip whitespace first
        todo!();

        // TODO: Match on peek() to determine token type
        // - None → Token::End
        // - '0'..='9' → Token::Number(self.read_number())
        // - '+' → advance and return Token::Plus
        // - '-' → advance and return Token::Minus
        // - '*' → advance and return Token::Star
        // - '/' → advance and return Token::Slash
        // - '(' → advance and return Token::LeftParen
        // - ')' → advance and return Token::RightParen
        // - anything else → Err with message
        todo!()
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        // TODO: Create empty Vec for tokens
        // TODO: Loop calling next_token() until Token::End
        // TODO: Push each token to Vec (including End token), then break
        // TODO: Return Ok(tokens)
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
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
    assert_eq!(tokens, vec![
        Token::LeftParen,
        Token::Number(2),
        Token::Plus,
        Token::Number(3),
        Token::RightParen,
        Token::Star,
        Token::Number(4),
        Token::End,
    ]);
}

#[test]
fn test_lexer_error() {
    let mut lexer = Lexer::new("2 & 3");
    assert!(lexer.tokenize().is_err());
}
```

**Check Your Understanding**:
- Why do we skip whitespace?
- How does `read_number()` build up the number?
- What happens if we forget to `advance()` after a token?

---

### Milestone 5: Recursive Descent Parser

**Goal**: Parse tokens into an AST using the arena.

**Grammar**:
```
Expr   → Term (('+' | '-') Term)*
Term   → Factor (('*' | '/') Factor)*
Factor → Number | '(' Expr ')'
```

**Starter Code**:
```rust
struct Parser<'arena> {
    tokens: Vec<Token>,
    position: usize,
    builder: ExprBuilder<'arena>,
}

impl<'arena> Parser<'arena> {
    fn new(tokens: Vec<Token>, arena: &'arena Arena) -> Self {
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
        match self.peek() {
            Token::Number(n) => {
                let n = *n;
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
        // TODO: Similar to parse_term but for + and -
        // Start with parse_term(), then loop handling + and -
        todo!()
    }

    fn parse(&mut self) -> Result<&'arena Expr<'arena>, String> {
        let expr = self.parse_expr()?;
        if self.peek() != &Token::End {
            return Err(format!("Unexpected token: {:?}", self.peek()));
        }
        Ok(expr)
    }
}

// Helper function
fn parse_and_eval(input: &str) -> Result<i64, String> {
    let arena = Arena::new();
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens, &arena);
    let expr = parser.parse()?;
    expr.eval()
}
```

**Checkpoint Tests**:
```rust
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
    assert_eq!(parse_and_eval("2 + 3 * 4"), Ok(14)); // Not 20!
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
    assert!(parse_and_eval("(2 + 3").is_err());  // Unclosed paren
}
```

**Check Your Understanding**:
- Why does the grammar have three levels (Expr, Term, Factor)?
- How does this handle operator precedence?
- Why do we parse Factor in Term and Term in Expr?
- When do we create nodes in the arena?

---

### Milestone 6: Performance Comparison

**Goal**: Compare arena allocation vs Box allocation.

**Box-based Version**:
```rust
enum BoxExpr {
    Literal(i64),
    BinOp {
        op: OpType,
        left: Box<BoxExpr>,
        right: Box<BoxExpr>,
    },
}

// Implement eval() and parser for BoxExpr
// Each node uses Box::new() instead of arena.alloc()
```

**Benchmark Code**:
```rust
use std::time::Instant;

fn benchmark_arena() {
    let start = Instant::now();
    for _ in 0..10000 {
        let arena = Arena::new();
        let _ = parse_with_arena("(1+2)*(3+4)+(5-2)*7", &arena);
    }
    let duration = start.elapsed();
    println!("Arena: {:?}", duration);
}

fn benchmark_box() {
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = parse_with_box("(1+2)*(3+4)+(5-2)*7");
    }
    let duration = start.elapsed();
    println!("Box: {:?}", duration);
}
```

**Expected Results**: Arena should be 5-20x faster depending on expression complexity.

**Check Your Understanding**:
- Why is arena allocation faster?
- When would Box be better than arena?
- What's the memory trade-off?

---

### Complete Project Summary

**What You Built**:
1. AST types with arena lifetimes
2. Bump allocator with proper alignment
3. Expression builder using arena
4. Lexer for tokenization
5. Recursive descent parser
6. Performance comparison

**Key Concepts Practiced**:
- Lifetimes and arena allocation
- Recursive descent parsing
- Unsafe Rust for low-level allocation
- Performance measurement and trade-offs

---
