## Project 2: Arena-Based Expression Parser

### Problem Statement

Build a parser for arithmetic expressions, that uses arena (bump) allocation. This demonstrates how arena allocation can dramatically speed up programs that create many small objects.
Wew go from simple expression enums to lexer to parser

### Use Cases

**When you need this pattern**:
1. **Compiler frontends**: Lexer tokens, AST nodes, symbol table entries
2. **Web request handlers**: Per-request temporary objects (template AST, JSON parsing)
3. **Game engines**: Per-frame allocations (particle systems, AI pathfinding nodes)
4. **Database query execution**: Query plan nodes, temporary expression trees
5. **Text editors**: Syntax tree for incremental parsing
6. **JSON/XML parsers**: DOM nodes, parsing state

### Why It Matters

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


### Learning Goals

- Understand arena/bump allocation and when it's appropriate
- Work with lifetimes in AST structures (`'arena` lifetime)
- Experience 10-100x allocation speedup
- Practice recursive descent parsing
- Understand memory layout and alignment requirements

---

### Milestone 1: Define AST Types

 Create the expression tree data structures that represent arithmetic expressions.

**Architecture**:
- **enum**: `Expr` - Expression Type
  - **field**: `Literal` - a literal number
  - **field**: `BinOp`   - needs to store: the operator and left and right sub-expressions

- **enum**: `OpType` - Operator Type
  - **field**: `Add` - addition
  - **field**: `Sub` - subtraction
  - **field**: `Mul` - multiplication
  - **field**: `Div` - division
 
**functions**:
- `eval()` method on `OpType` that takes two numbers and returns the result
    - Handle division by zero by returning a `Result<i64, String>`
- `eval()` method on `Expr` that recursively evaluates the expression tree
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

#### Why Milestone 1 Isn't Enough 

**Limitation**: We've defined the types, but how do we actually create these AST nodes? Using stack allocation limits us to small, fixed-size trees. We need heap allocation.

**What we're adding**: First, we'll implement the traditional `Box` approach to understand the baseline, then optimize with arena allocation.

---

### Milestone 2: Box-Based Expression Trees

Implement expressions using `Box<Expr>` to understand traditional heap allocation.

**Architecture**:
- Each AST node gets its own heap allocation via `Box::new()`
- Each node has its own drop when the tree is freed
- This is the "normal" approach used in many programming languages

**Design Changes**:
Instead of using references with lifetimes for `left` and `right`, we'll use `Box` pointers:


**solution**
```rust
#[derive(Debug, PartialEq)]
enum BoxExpr {
    Literal(i64),
    BinOp {
        op: OpType,
        left: Box<BoxExpr>,
        right: Box<BoxExpr>,
    },
}

impl BoxExpr {
    fn eval(&self) -> Result<i64, String> {
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
```

**Builder Pattern**:
```rust
struct BoxExprBuilder;

impl BoxExprBuilder {
    fn literal(n: i64) -> Box<BoxExpr> {
        // TODO: Allocate BoxExpr::Literal(n) using Box::new()
        todo!()
    }

    fn binary(op: OpType, left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Allocate BoxExpr::BinOp using Box::new()
        todo!()
    }

    fn add(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Add
        todo!()
    }

    fn sub(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Sub
        todo!()
    }

    fn mul(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Mul
        todo!()
    }

    fn div(left: Box<BoxExpr>, right: Box<BoxExpr>) -> Box<BoxExpr> {
        // TODO: Call binary() with OpType::Div
        todo!()
    }
}
```
**Checkpoint Tests**:
```rust
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
    // Build: (2 + 3) * 4 = 20
    let expr = BoxExprBuilder::mul(
        BoxExprBuilder::add(
            BoxExprBuilder::literal(2),
            BoxExprBuilder::literal(3),
        ),
        BoxExprBuilder::literal(4),
    );
    assert_eq!(expr.eval(), Ok(20));
}

#[test]
fn test_box_expr_complex() {
    // Build: ((10 - 5) * 2) + (8 / 4) = 12
    let expr = BoxExprBuilder::add(
        BoxExprBuilder::mul(
            BoxExprBuilder::sub(
                BoxExprBuilder::literal(10),
                BoxExprBuilder::literal(5),
            ),
            BoxExprBuilder::literal(2),
        ),
        BoxExprBuilder::div(
            BoxExprBuilder::literal(8),
            BoxExprBuilder::literal(4),
        ),
    );
    assert_eq!(expr.eval(), Ok(12));
}
```

**Check Your Understanding**:
- How many heap allocations occur for the expression `(2 + 3) * 4`?
- What happens when a `Box<BoxExpr>` goes out of scope?
- Why does the builder consume (take ownership of) the `Box` parameters?
- What are the performance implications of many small allocations?

---

#### Why Milestone 2 Isn't Enough 

**Performance Problem**: Every single AST node requires a separate heap allocation with `Box::new()`. Let's analyze the cost:

**Allocation Overhead**:
- Expression `(1+2)*(3+4)` = 7 nodes
- Each `Box::new()`: ~50-100ns (involves malloc, locks, metadata)
- Total allocation time: 7 × 75ns = **525ns just for allocations**
- Parsing 10,000 expressions: 70,000 allocations = **5.25ms**

**Memory Fragmentation**:
- Nodes scattered across heap memory
- Poor cache locality (next node likely in different cache line)
- Each allocation has ~16 bytes overhead for allocator metadata

**What we're adding**: 
**Arena allocator** - bump allocation strategy:
An arena (also called a bump allocator) is a simple memory allocator that hands out memory by continuously "bumping" a pointer forward inside a pre-allocated buffer. Individual allocations are extremely cheap (often just pointer arithmetic), and deallocation is even simpler: you free everything at once by dropping the arena.

How it works at a glance:
1. Reserve a big chunk of memory (e.g., 4 KB).
2. Keep an offset (the "bump" pointer) into that chunk.
3. To allocate `T`, round the offset up to `align_of::<T>()`, ensure there’s room, then return a pointer/reference to that slot and advance the offset by `size_of::<T>()`.
4. When the arena goes out of scope, the whole chunk is freed at once.

Why use it for ASTs and similar graphs:
- Many small nodes created together and dropped together at the end of parsing/evaluation.
- Significantly fewer calls to the global allocator → better performance and cache locality.

Contrast with `Box<T>` per node:
- `Box<T>`: many small, scattered allocations; each `Box` is freed individually.
- Arena: one or few big allocations; trivial per-object allocation; single bulk free.

**Improvements**:
- **Speed**: Allocation is pointer increment (~2-5ns) vs malloc (~75ns) = **25x faster**
- **Memory**: Better cache locality (nodes allocated sequentially)
- **Simplicity**: No individual frees—drop arena, free everything
- **Alignment**: Must handle properly (u8 at any address, u64 needs 8-byte alignment)

**Complexity trade-off**: Can't free individual objects. Only works when all objects have same lifetime.

---

### Milestone 3: Simple Bump Allocator

Implement a basic arena that can allocate objects.

**Architecture**
- **struct**: `Arena`  
    - **field**: `storage`: RefCell<Vec<u8>>
**functions**:
- `new() ` 
- `alloc()`



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

### Milestone 4: Build Expressions in Arena

Use the arena allocator to create expression trees with the builder pattern.


Now that we have a working arena allocator (Milestone 3), we need a clean API to use it. Directly calling `arena.alloc()` everywhere would be verbose and error-prone. The **Builder Pattern** provides a fluent, type-safe interface for constructing expression trees.

**What We're Building**:

The `ExprBuilder` wraps the arena and provides convenient methods like `literal()`, `add()`, `mul()` that hide the allocation details. Compare:

```rust
// Without builder (verbose, easy to mess up lifetimes):
let two = arena.alloc(Expr::Literal(2));
let three = arena.alloc(Expr::Literal(3));
let sum = arena.alloc(Expr::BinOp {
    op: OpType::Add,
    left: two,
    right: three,
});

// With builder (clean, fluent):
let two = builder.literal(2);
let three = builder.literal(3);
let sum = builder.add(two, three);
```

**Design Decisions**:

1. **Builder holds `&'arena Arena`**: The builder doesn't own the arena—it just borrows it. This allows multiple builders to share one arena if needed.

2. **All methods return `&'arena Expr<'arena>`**: Every expression we allocate lives in the arena, and the lifetime annotation ensures they can't outlive it.

3. **Convenience methods** (`add`, `mul`, etc.): These wrap the generic `binary()` method, making expression construction more readable.

**The Lifetime Dance**:

Notice the signature: `fn literal(&self, n: i64) -> &'arena Expr<'arena>`. We take `&self` (short borrow of builder), but return `&'arena` (long-lived reference tied to arena's lifetime). This works because:
- The builder holds `&'arena Arena`
- We allocate in that arena
- The returned reference lives as long as the arena, not the builder

#### Architecture

- **Struct**: `ExprBuilder<'arena>`
  - **Fields**: `arena: &'arena Arena`
**Functions**:
- `new(arena: &'arena Arena) -> Self` - Creates builder wrapping arena
- `literal(&self, n: i64) -> &'arena Expr<'arena>` - Allocates literal expression
- `binary(&self, op: OpType, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena>` - Generic binary operation
- `add(...)`, `sub(...)`, `mul(...)`, `div(...)` - Convenience wrappers


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

### Milestone 5: Lexer (Tokenizer)

Transform raw text input into a stream of tokens that the parser can work with.

So far, we've been manually constructing expression trees using the builder. But real parsers work with text input like `"(2 + 3) * 4"`. The **lexer** (also called tokenizer or scanner) is the first stage of parsing that breaks this text into meaningful chunks called **tokens**.

**The Two-Stage Pipeline**:

```
Text → Lexer → Tokens → Parser → AST
"2+3"  →       [Num(2), Plus, Num(3)]  →  BinOp{Add, 2, 3}
```

Separating lexing from parsing is a fundamental compiler design pattern because:
1. **Separation of concerns**: Lexing handles character-level details (whitespace, digits), parsing handles structure (precedence, grammar)
2. **Simplification**: Parser doesn't worry about whitespace or number parsing
3. **Reusability**: Same token stream can feed multiple parsers
4. **Performance**: Can optimize lexer separately (e.g., SIMD for digit scanning)

**What We're Building**:

A **Lexer** that walks through input text character-by-character and identifies:
- **Numbers**: Sequences of digits like `123`, `0`, `9876`
- **Operators**: `+`, `-`, `*`, `/`
- **Parentheses**: `(`, `)`
- **Whitespace**: Skipped (not significant in arithmetic)
- **End of input**: Special `End` token

**The Lexer State**:

```rust
struct Lexer {
    input: Vec<char>,   // Input text as characters
    position: usize,    // Current position in input
}
```

We convert the string to `Vec<char>` because:
- Easy indexing by character (not byte)
- Handles multi-byte Unicode properly (though our grammar is ASCII-only)
- Simple `position` counter tracks where we are

**functions**:

- `peek()` - Look at current character without moving forward
- `advance()` - Move position forward by one character
- `skip_whitespace()` - Skip spaces, tabs, newlines
- `read_number()` - Consume consecutive digits and build an integer
- `next_token()` - Return the next token from input
- `tokenize()` - Convert entire input to `Vec<Token>`

**Example Tokenization**:

```rust
Input: "(10 + 5) * 2"

Steps:
1. Skip nothing, see '(' → Token::LeftParen, advance
2. Skip space, see '1' → read_number() → Token::Number(10), advance twice
3. Skip space, see '+' → Token::Plus, advance
4. Skip space, see '5' → read_number() → Token::Number(5), advance
5. Skip nothing, see ')' → Token::RightParen, advance
6. Skip space, see '*' → Token::Star, advance
7. Skip space, see '2' → read_number() → Token::Number(2), advance
8. At end → Token::End

Output: [LeftParen, Number(10), Plus, Number(5), RightParen, Star, Number(2), End]
```

**Error Handling**:

The lexer must detect invalid characters:
```rust
Input: "2 & 3"  // '&' is not a valid operator
Result: Err("Unexpected character '&'")
```

Returning `Result<Token, String>` allows propagating errors up to the caller.


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


**Check Your Understanding**:
- Why do we skip whitespace?
- How does `read_number()` build up the number?
- What happens if we forget to `advance()` after a token?

---

### Milestone 6: Recursive Descent Parser

 Transform the token stream from the lexer into an Abstract Syntax Tree (AST) stored in the arena, respecting operator precedence and parentheses.


The parser is the **brain** of the compiler—it understands the **structure** and **meaning** of code. While the lexer breaks text into tokens, the parser answers questions like:
- Does `2 + 3 * 4` mean `(2 + 3) * 4` or `2 + (3 * 4)`? (Answer: second one, multiplication binds tighter)
- Are the parentheses balanced in `((1 + 2) * 3`? (Answer: no, missing closing paren)
- Is `+ + 3` valid? (Answer: no, can't have two operators in a row)

**Complete Pipeline**:

```
Text → Lexer → Tokens → Parser → AST → Evaluator → Result
"2+3*4" →  [Num(2), Plus, Num(3), Star, Num(4)]  →
           BinOp{Add, 2, BinOp{Mul, 3, 4}}  →  14
```

**Recursive Descent Parsing**:

We'll implement a **recursive descent parser**, which means:
1. Each grammar rule becomes a function
2. Functions call each other recursively to match nested structures
3. The call stack mirrors the parse tree structure

This is one of the simplest and most intuitive parsing techniques. Other approaches (LR, LALR, Pratt parsing) are more powerful but complex.

**The Grammar and Operator Precedence**:

Our grammar has **three levels** to encode operator precedence:

```
Expr   → Term (('+' | '-') Term)*      // Lowest precedence: addition/subtraction
Term   → Factor (('*' | '/') Factor)*  // Medium precedence: multiplication/division
Factor → Number | '(' Expr ')'         // Highest precedence: atoms and parens
```

**Why three levels?** This encodes the precedence rules:
- **Factor** (highest): Numbers and parenthesized expressions bind tightest
- **Term** (medium): `*` and `/` bind tighter than `+` and `-`
- **Expr** (lowest): `+` and `-` bind loosest

**How Precedence Works**:

For `2 + 3 * 4`:

```
parse_expr() calls:
  parse_term() for "2"
    parse_factor() returns Literal(2)
  Sees '+', continues
  parse_term() for "3 * 4"
    parse_factor() returns Literal(3)
    Sees '*', continues
    parse_factor() returns Literal(4)
    Returns Mul(3, 4)
  Returns Add(2, Mul(3, 4))
```

Notice: `parse_term()` consumed `3 * 4` as a unit **before** returning to `parse_expr()`. This is how multiplication binds tighter than addition!

**Parsing Strategy for Each Level**:

1. **`parse_expr()`**: Parse a term, then loop consuming `+` or `-` operators
   ```
   2 + 3 - 4  →  Sub(Add(2, 3), 4)
   ```

2. **`parse_term()`**: Parse a factor, then loop consuming `*` or `/` operators
   ```
   2 * 3 / 4  →  Div(Mul(2, 3), 4)
   ```

3. **`parse_factor()`**: Parse atomic elements
    - If number: return literal
    - If `(`: recursively parse expression, expect `)`
    - Otherwise: error

**Architecture**:

```rust
struct Parser<'arena> {
    tokens: Vec<Token>,           // All tokens from lexer
    position: usize,              // Current position in token stream
    builder: ExprBuilder<'arena>, // For allocating AST nodes in arena
}
```

**functions**:

- `peek()` - Look at current token without advancing
- `advance()` - Move to next token
- `expect(token)` - Verify current token matches expected, advance, or error
- `parse_factor()` - Parse numbers and parenthesized expressions
- `parse_term()` - Parse multiplication and division
- `parse_expr()` - Parse addition and subtraction
- `parse()` - Entry point that parses and verifies we consumed all tokens

**Detailed Example: Parsing `(2 + 3) * 4`**:

```
Tokens: [LeftParen, Number(2), Plus, Number(3), RightParen, Star, Number(4), End]

parse() calls parse_expr():
  parse_expr() calls parse_term():
    parse_term() calls parse_factor():
      See '(' → advance, call parse_expr() recursively:
        parse_expr() calls parse_term():
          parse_term() calls parse_factor():
            See Number(2) → return Literal(2)
          No '*' or '/', return Literal(2)
        See '+', advance, call parse_term():
          parse_term() calls parse_factor():
            See Number(3) → return Literal(3)
          No '*' or '/', return Literal(3)
        Build Add(Literal(2), Literal(3))
      Expect ')' → found it, advance
      Return Add(2, 3)
    See '*', advance, call parse_factor():
      See Number(4) → return Literal(4)
    Build Mul(Add(2, 3), Literal(4))
  No '+' or '-', return Mul(...)
parse() verifies Token::End

Result: Mul(Add(2, 3), 4)
```

**Error Handling**:

The parser must catch:
- **Unexpected tokens**: `2 + + 3` (two operators)
- **Missing operands**: `2 +` (nothing after +)
- **Unbalanced parens**: `(2 + 3` (missing closing paren)
- **Trailing input**: `2 + 3 4` (unexpected 4 at end)

All parse functions return `Result<&'arena Expr<'arena>, String>` to propagate errors.

**Why Recursive Descent?**

**Advantages**:
- **Simple**: Each grammar rule = one function
- **Clear error messages**: Know exactly where parsing failed
- **Debuggable**: Can step through and see call stack
- **Hand-optimizable**: Can add special cases for performance
- **No external tools**: No parser generator needed

**Disadvantages**:
- **Left recursion**: Can't handle grammars like `Expr → Expr '+' Term` (infinite loop)
- **Backtracking**: Inefficient for ambiguous grammars (not our case)
- **Grammar restrictions**: Not all grammars work

**The Connection to Arena Allocation**:

Notice: The parser allocates many AST nodes while parsing. With arena allocation:
- Each node: 1 arena bump (~3ns)
- Total for `(2+3)*4`: 5 nodes = ~15ns allocation time
- With Box: 5 mallocs = ~375ns

For complex expressions with hundreds of nodes, the arena speedup is dramatic!

**Grammar**:
```
Expr   → Term (('+' | '-') Term)*
Term   → Factor (('*' | '/') Factor)*
Factor → Number | '(' Expr ')'
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

**Check Your Understanding**:
- Why does the grammar have three levels (Expr, Term, Factor)?
- How does this handle operator precedence?
- Why do we parse Factor in Term and Term in Expr?
- When do we create nodes in the arena?

---

### Milestone 7: Performance Comparison

Compare arena allocation vs Box allocation using the implementations from Milestones 2 and 3.

**Benchmark Code**:
```rust
use std::time::Instant;

fn benchmark_arena() {
    let start = Instant::now();
    for _ in 0..10000 {
        let arena = Arena::new();
        // Build expression: (1+2)*(3+4)+(5-2)*7
        let builder = ExprBuilder::new(&arena);
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
    let duration = start.elapsed();
    println!("Arena: {:?}", duration);
}

fn benchmark_box() {
    let start = Instant::now();
    for _ in 0..10000 {
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
    let duration = start.elapsed();
    println!("Box: {:?}", duration);
}

fn main() {
    println!("Benchmarking expression allocation...");
    benchmark_box();
    benchmark_arena();
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

