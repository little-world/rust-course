# Arena-Based Expression Parser

### Problem Statement

Build a parser for arithmetic expressions, that uses arena (bump) allocation. This demonstrates how arena allocation can dramatically speed up programs that create many small objects.
Wew go from simple expression enums to lexer to parser

---
**Key Learning Points**:
- ASTs represent program structure as trees
- Lexers simplify parsing by handling character-level details
- Recursive descent is an intuitive parsing technique
- Grammar structure encodes operator precedence
- Arena allocation can dramatically speed up tree construction

---

### Use Cases

**When you need this pattern**:
1. **Compiler frontends**: Lexer tokens, AST nodes, symbol table entries
2. **Web request handlers**: Per-request temporary objects (template AST, JSON parsing)
3. **Game engines**: Per-frame allocations (particle systems, AI pathfinding nodes)
4. **Database query execution**: Query plan nodes, temporary expression trees
5. **Text editors**: Syntax tree for incremental parsing
6. **JSON/XML parsers**: DOM nodes, parsing state


## Understanding Parsers: ASTs, Expressions, Lexers, and Recursive Descent

Before diving into the implementation, let's understand the fundamental concepts that make parsers work. This project implements a complete parser pipeline from scratch, giving you hands-on experience with concepts used in every compiler, interpreter, and language tool.

### What is a Parser?

A **parser** is a program that reads text (source code, JSON, configuration files, etc.) and converts it into a structured representation that computers can work with. Every programming language, database query language, and markup language needs a parser.

**The fundamental problem**: Computers can't directly understand text like `"2 + 3 * 4"`. They need this converted into a **tree structure** that represents the operations and their precedence.

**Example transformation**:
```
Text input: "2 + 3 * 4"

Parser converts to tree:
        (+)
       /   \
      2    (*)
          /   \
         3     4

This tree says: "First multiply 3 and 4, then add 2 to the result"
Result: 2 + 12 = 14 (not 20!)
```

---

### Abstract Syntax Trees (ASTs)

An **Abstract Syntax Tree (AST)** is a tree representation of the structure of source code. Each node in the tree represents a construct in the code.

**Why "Abstract"?**
- The tree abstracts away syntactic details like parentheses, whitespace, and semicolons
- It captures **meaning** (semantics), not just **form** (syntax)
- Example: `(2+3)` and `2+3` produce the same AST even though the text is different

**AST vs Parse Tree**:
```
Input: "2 + 3"

Parse Tree (concrete, includes all syntax):
    Expr
     |
    Term
     |
   Factor  '+'  Factor
     |            |
    '2'          '3'

AST (abstract, only meaning):
    (+)
   /   \
  2     3
```

**AST Structure for Arithmetic Expressions**:

```rust
// Our AST nodes
enum Expr {
    Literal(i64),              // A number: 42
    BinOp {                     // Binary operation: left op right
        op: OpType,             // +, -, *, /
        left: &Expr,           // Left sub-expression
        right: &Expr,          // Right sub-expression
    }
}
```

**Example ASTs**:

```
Expression: "5"
AST: Literal(5)

Expression: "2 + 3"
AST:
    BinOp {
        op: Add,
        left: Literal(2),
        right: Literal(3)
    }

Expression: "2 + 3 * 4"
AST:
    BinOp {
        op: Add,
        left: Literal(2),
        right: BinOp {
            op: Mul,
            left: Literal(3),
            right: Literal(4)
        }
    }

Expression: "(2 + 3) * 4"
AST:
    BinOp {
        op: Mul,
        left: BinOp {
            op: Add,
            left: Literal(2),
            right: Literal(3)
        },
        right: Literal(4)
    }
```

**Key AST Properties**:

1. **Recursive Structure**: Trees can be arbitrarily nested
   - `1 + 2` is a tree
   - `(1 + 2) * (3 + 4)` is a tree containing two smaller trees

2. **Evaluation by Tree Walk**: To evaluate an expression, walk the tree recursively:
   ```rust
   fn eval(expr: &Expr) -> i64 {
       match expr {
           Expr::Literal(n) => *n,  // Base case
           Expr::BinOp { op, left, right } => {
               let l = eval(left);   // Recursive call
               let r = eval(right);  // Recursive call
               apply_op(op, l, r)
           }
       }
   }
   ```

3. **Precedence is Encoded in Structure**: Higher-precedence operations are deeper in the tree
   - In `2 + 3 * 4`, the multiplication is a child of addition
   - The multiplication must evaluate first (depth-first)

---

### What are Expressions?

An **expression** is a combination of values, variables, and operators that can be **evaluated** to produce a value.

**Expression Examples**:
```
42                    → Evaluates to 42
2 + 3                 → Evaluates to 5
2 + 3 * 4             → Evaluates to 14
(2 + 3) * 4           → Evaluates to 20
((1 + 2) * 3) - 4     → Evaluates to 5
```

**Not expressions** (these are statements, they don't produce values):
```
let x = 5;            // Variable declaration
if x > 0 { ... }      // Conditional statement
while true { ... }    // Loop statement
```

**Expression Components**:

1. **Literals**: Concrete values like `42`, `3.14`, `"hello"`
2. **Operators**: Symbols that combine values like `+`, `-`, `*`, `/`
3. **Operands**: The values operators work on
4. **Precedence**: Rules for which operators bind tighter
   - `*` and `/` bind tighter than `+` and `-`
   - `2 + 3 * 4` = `2 + (3 * 4)`, not `(2 + 3) * 4`

5. **Associativity**: When operators have equal precedence, which direction to evaluate
   - Left associative: `10 - 5 - 2` = `(10 - 5) - 2` = 3
   - Right associative: `2 ^ 3 ^ 2` = `2 ^ (3 ^ 2)` = 512 (in languages with exponentiation)

**Operator Precedence Table** (for this project):
```
Highest: ( )         Parentheses (force evaluation order)
         * /         Multiplication and Division
Lowest:  + -         Addition and Subtraction

Examples:
2 + 3 * 4     = 2 + (3 * 4) = 14
8 / 4 / 2     = (8 / 4) / 2 = 1    (left associative)
2 + 3 - 1     = (2 + 3) - 1 = 4    (left associative)
(2 + 3) * 4   = 5 * 4 = 20         (parens override precedence)
```

**Why Expressions Matter**:

Every programming language has expressions:
- **JavaScript**: `x + y`, `foo() && bar()`, `a ? b : c`
- **Python**: `x + y`, `[i*2 for i in range(10)]`, `a if cond else b`
- **Rust**: `x + y`, `Some(42)`, `vec![1, 2, 3]`
- **SQL**: `price * quantity`, `UPPER(name)`, `age > 18 AND active = true`

Understanding how to parse expressions is fundamental to working with any language.

---

### What is a Lexer (Tokenizer)?

A **lexer** (also called tokenizer or scanner) is the first stage of parsing. It breaks raw text into meaningful chunks called **tokens**.

**The Lexer's Job**:
```
Input:  "(2 + 3) * 4"    ← Raw string of characters
Output: [LeftParen, Number(2), Plus, Number(3), RightParen, Star, Number(4), End]
                         ↑ Tokens
```

**Why We Need Lexers**:

1. **Simplification**: The parser doesn't have to worry about:
   - Skipping whitespace
   - Reading multi-character numbers
   - Handling comments
   - Unicode vs ASCII

2. **Separation of Concerns**:
   - Lexer handles **character-level** details
   - Parser handles **structural** details

3. **Performance**: Can optimize lexer separately (e.g., SIMD for digit scanning)

4. **Reusability**: Same token stream can feed different parsers

**Token Types**:

```rust
enum Token {
    Number(i64),        // 42, 123, 9876
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    LeftParen,          // (
    RightParen,         // )
    End,                // End of input
}
```

**Lexer Algorithm**:

```
function next_token():
    1. Skip whitespace (spaces, tabs, newlines)
    2. Look at current character:
       - If digit: read_number() → Token::Number(n)
       - If '+': return Token::Plus
       - If '-': return Token::Minus
       - If '*': return Token::Star
       - If '/': return Token::Slash
       - If '(': return Token::LeftParen
       - If ')': return Token::RightParen
       - If end of input: return Token::End
       - Otherwise: ERROR (unexpected character)
    3. Advance position past the token
    4. Return the token
```

**Example Tokenization**:

```
Input: "10 + 5 * 2"

Step-by-step:
Position 0: '1' is digit → read_number() reads "10" → Token::Number(10)
Position 2: ' ' is space → skip
Position 3: '+' → Token::Plus
Position 4: ' ' is space → skip
Position 5: '5' is digit → read_number() reads "5" → Token::Number(5)
Position 6: ' ' is space → skip
Position 7: '*' → Token::Star
Position 8: ' ' is space → skip
Position 9: '2' is digit → read_number() reads "2" → Token::Number(2)
Position 10: End of input → Token::End

Result: [Number(10), Plus, Number(5), Star, Number(2), End]
```

**Reading Multi-Character Tokens**:

```rust
fn read_number() -> i64 {
    let mut num = 0;
    while current char is digit {
        num = num * 10 + (char - '0');  // Build number digit by digit
        advance();
    }
    return num;
}
```
Example: "123"
Start: num = 0
See '1': num = 0*10 + 1 = 1
See '2': num = 1*10 + 2 = 12
See '3': num = 12*10 + 3 = 123
See ' ': not a digit, stop
Return 123


**Lexer State**:

```rust
struct Lexer {
    input: Vec<char>,     // Input text as characters
    position: usize,      // Current position in input
}
```

**Why Vec<char> instead of &str**?
- Easy indexing by character (not byte)
- Handles multi-byte Unicode correctly
- Simple position counter

---

### What is Recursive Descent Parsing?

**Recursive descent** is a parsing technique where:
1. Each grammar rule becomes a function
2. Functions call each other recursively to match nested structures
3. The call stack mirrors the parse tree structure

**Our Grammar** (for arithmetic expressions):
```
Expr   → Term (('+' | '-') Term)*      // Lowest precedence
Term   → Factor (('*' | '/') Factor)*  // Medium precedence
Factor → Number | '(' Expr ')'         // Highest precedence
```

**Reading the Grammar**:
- `→` means "is defined as"
- `|` means "or"
- `*` means "zero or more"
- `()` groups elements

**Translation to Functions**:

```rust
// Expr → Term (('+' | '-') Term)*
fn parse_expr() -> Expr {
    let mut left = parse_term();      // Start with a Term
    while current token is '+' or '-' {
        let op = consume operator;
        let right = parse_term();
        left = BinOp(op, left, right);
    }
    return left;
}

// Term → Factor (('*' | '/') Factor)*
fn parse_term() -> Expr {
    let mut left = parse_factor();    // Start with a Factor
    while current token is '*' or '/' {
        let op = consume operator;
        let right = parse_factor();
        left = BinOp(op, left, right);
    }
    return left;
}

// Factor → Number | '(' Expr ')'
fn parse_factor() -> Expr {
    if current token is Number(n) {
        consume token;
        return Literal(n);
    }
    if current token is '(' {
        consume '(';
        let expr = parse_expr();     // Recursive call!
        expect ')';
        return expr;
    }
    error("Expected number or '('");
}
```

**How Precedence Works**:

The grammar encodes precedence through **nesting depth**:
- `Factor` (deepest) = highest precedence
- `Term` (middle) = medium precedence
- `Expr` (top) = lowest precedence

**Example Parse: `2 + 3 * 4`**

```
Tokens: [Number(2), Plus, Number(3), Star, Number(4), End]

parse_expr():
  left = parse_term():
    left = parse_factor():
      See Number(2) → return Literal(2)
    See '+' (not '*' or '/') → return Literal(2)

  See '+' → consume it

  right = parse_term():
    left = parse_factor():
      See Number(3) → return Literal(3)
    See '*' → consume it
    right = parse_factor():
      See Number(4) → return Literal(4)
    left = BinOp(Mul, Literal(3), Literal(4))
    No more '*' or '/' → return BinOp(Mul, 3, 4)

  left = BinOp(Add, Literal(2), BinOp(Mul, 3, 4))

  No more '+' or '-' → return result

Result AST:
    Add
   /   \
  2    Mul
      /   \
     3     4
```

**Why Three Levels?**

This ensures `3 * 4` is fully parsed **before** returning to the addition:
- `parse_expr()` calls `parse_term()` for "3 * 4"
- `parse_term()` consumes both "3" and "* 4" as a unit
- Returns `Mul(3, 4)` as a **single node**
- `parse_expr()` then builds `Add(2, Mul(3, 4))`

**Example Parse: `(2 + 3) * 4`**

```
Tokens: [LeftParen, Number(2), Plus, Number(3), RightParen, Star, Number(4), End]

parse_expr():
  left = parse_term():
    left = parse_factor():
      See '(' → consume it

      Recursive call to parse_expr():  ← RECURSION!
        left = parse_term():
          left = parse_factor():
            See Number(2) → return Literal(2)
          No '*' or '/' → return Literal(2)
        See '+' → consume it
        right = parse_term():
          left = parse_factor():
            See Number(3) → return Literal(3)
          No '*' or '/' → return Literal(3)
        left = BinOp(Add, Literal(2), Literal(3))
        No more '+' or '-' → return Add(2, 3)

      Expect ')' → found it, consume
      return Add(2, 3)  ← Returns from recursive call

    See '*' → consume it
    right = parse_factor():
      See Number(4) → return Literal(4)
    left = BinOp(Mul, Add(2, 3), Literal(4))
    No more '*' or '/' → return Mul(Add(2, 3), 4)

  No '+' or '-' → return result

Result AST:
     Mul
    /   \
  Add    4
 /   \
2     3
```

**Key Insight**: The parentheses forced `parse_factor()` to recursively call `parse_expr()`, which parsed the entire `2 + 3` before returning. This is how parentheses override precedence!

---

### The Complete Parser Pipeline

Putting it all together:

```
Step 1: LEXER (Character → Tokens)
Input:  "(2 + 3) * 4"
Output: [LeftParen, Number(2), Plus, Number(3), RightParen, Star, Number(4), End]

Step 2: PARSER (Tokens → AST)
Input:  [LeftParen, Number(2), Plus, Number(3), RightParen, Star, Number(4), End]
Output:      Mul
            /   \
          Add    4
         /   \
        2     3

Step 3: EVALUATOR (AST → Result)
Input:  AST tree
Process:
  - Evaluate Add(2, 3) → 5
  - Evaluate Mul(5, 4) → 20
Output: 20
```

**Why This Separation?**

1. **Lexer** handles messy character-level details
2. **Parser** focuses on structure and meaning
3. **Evaluator** (or code generator, or interpreter) uses the clean AST

Each stage is simpler and more testable because of this separation!

---

### Real-World Applications

**Compilers and Interpreters**:
- **C compiler**: Parses `int x = 5;` into an AST, generates assembly
- **Python**: Parses code into AST, interprets or compiles to bytecode
- **JavaScript V8**: Parses JS code, generates optimized machine code

**Data Formats**:
- **JSON**: Parses `{"name": "Alice"}` into object representation
- **XML/HTML**: Parses tags into DOM tree
- **YAML**: Parses configuration into nested structures

**Query Languages**:
- **SQL**: Parses `SELECT * FROM users WHERE age > 18` into query plan
- **GraphQL**: Parses queries into execution plan

**Domain-Specific Languages (DSLs)**:
- **CSS selectors**: `.class > #id` parsed into selector tree
- **Regex**: `/a+b*/` parsed into state machine
- **Build systems**: `Makefile` rules parsed into dependency graph

---

## Rust Programming Concepts for This Project

This project requires understanding several advanced Rust concepts related to memory management, lifetimes, and unsafe code. These concepts enable building high-performance systems that would be difficult or impossible in garbage-collected languages.

### Lifetimes: Expressing Object Dependencies

**The Problem**: Rust needs to know how long references live to prevent dangling pointers. When we build a tree of references, we need to express that all the references share the same lifetime.

```rust
// This doesn't compile - lifetime unclear
struct TreeNode {
    left: &TreeNode,   // ❌ Error: missing lifetime
    right: &TreeNode,  // ❌ Error: missing lifetime
}

// This works - all references tied to 'tree lifetime
struct TreeNode<'tree> {
    left: &'tree TreeNode<'tree>,
    right: &'tree TreeNode<'tree>,
}
```

**What is a Lifetime?**

A lifetime is a **compile-time annotation** that tells Rust how long a reference is valid. It's not a runtime concept—it exists purely for the compiler's static analysis.

**Lifetime Notation**:
```rust
// 'arena is a lifetime parameter
// Read as: "apostrophe arena"
fn alloc<'arena>(&'arena self, value: T) -> &'arena T

// Multiple lifetimes
fn example<'a, 'b>(x: &'a i32, y: &'b i32) -> &'a i32
```

**The Arena Lifetime Pattern**:

In this project, all AST nodes live in an arena, and all references point into that arena:

```rust
#[derive(Debug, PartialEq)]
enum Expr<'arena> {
    Literal(i64),
    BinOp {
        op: OpType,
        left: &'arena Expr<'arena>,   // Reference valid for 'arena lifetime
        right: &'arena Expr<'arena>,  // Same lifetime
    },
}

struct Arena {
    storage: RefCell<Vec<u8>>,
}

impl Arena {
    fn alloc<'arena, T>(&'arena self, value: T) -> &'arena T {
        // Allocate T in arena, return reference with 'arena lifetime
        // The reference is valid as long as the arena exists
    }
}
```

**Key Insight**: The `'arena` lifetime connects three things:
1. **The arena itself** - must live as long as `'arena`
2. **All allocated objects** - stored in the arena
3. **All references to objects** - can't outlive the arena

**Why This Works**:
```rust
{
    let arena = Arena::new();           // Arena created
    let expr = build_tree(&arena);      // AST built in arena
    let result = expr.eval();           // Can use AST
    // arena dropped here, all references invalidated
}
// Can't use expr here - compiler prevents it!
```

**What Lifetimes Prevent**:
```rust
let dangling_ref = {
    let arena = Arena::new();
    let expr = arena.alloc(Expr::Literal(42));
    expr  // ❌ Error: expr references arena, which is about to be dropped
};
// If this compiled, we'd have a dangling pointer!
```

**Lifetime Elision** (when you don't see lifetimes):

Sometimes Rust infers lifetimes:
```rust
// Written:
fn first(x: &i32, y: &i32) -> &i32 { x }

// Compiler sees:
fn first<'a, 'b>(x: &'a i32, y: &'b i32) -> &'a i32 { x }
```

**Why We Need Explicit Lifetimes for Arena**:

Self-referential structures require explicit lifetime annotations because Rust can't infer the relationship:
```rust
// Compiler can't infer these relationships automatically
enum Expr<'arena> {
    BinOp {
        left: &'arena Expr<'arena>,   // Must be explicit
        right: &'arena Expr<'arena>,
    }
}
```

---

### Arena Allocation: The Core Performance Technique

**The Traditional Allocation Problem**:

```rust
// Each Box::new() calls malloc() - expensive!
let expr = Box::new(BinOp {
    op: Add,
    left: Box::new(Literal(2)),      // malloc #1
    right: Box::new(Literal(3)),     // malloc #2
});                                   // malloc #3

// For expression (1+2)*(3+4):
// - 7 nodes = 7 malloc calls
// - Each malloc: ~50-100ns
// - Total: ~525ns just for allocation
```

**The Arena Solution**:

An **arena allocator** (also called bump allocator) pre-allocates a large chunk of memory and hands out pieces by incrementing a pointer.

```rust
struct Arena {
    storage: RefCell<Vec<u8>>,  // Big buffer of bytes
}

impl Arena {
    fn alloc<T>(&self, value: T) -> &mut T {
        // 1. Calculate size and alignment
        // 2. Bump pointer forward
        // 3. Write value at new position
        // 4. Return reference
        // Total: ~2-5ns (25x faster than malloc!)
    }
}
```

**How Arena Allocation Works**:

```
Initial state:
┌─────────────────────────────────────────┐
│ [empty buffer, 4096 bytes]              │
└─────────────────────────────────────────┘
 ↑
 position = 0

After alloc(42i64):
┌─────────────────────────────────────────┐
│ [42][empty space...]                    │
└─────────────────────────────────────────┘
     ↑
     position = 8 (size of i64)

After alloc(100i32):
┌─────────────────────────────────────────┐
│ [42][100][empty space...]               │
└─────────────────────────────────────────┘
         ↑
         position = 12 (8 + 4)

After alloc(Expr::Literal(5)):
┌─────────────────────────────────────────┐
│ [42][100][Literal(5)][empty space...]   │
└─────────────────────────────────────────┘
                     ↑
                     position = 12 + sizeof(Expr)
```

**Key Characteristics**:

1. **Fast Allocation**: Just pointer arithmetic and write
   ```rust
   // Pseudocode for allocation:
   let start = current_position;
   current_position += size;
   write_value_at(start, value);
   return reference_to(start);
   ```

2. **No Individual Deallocation**: Can't free single objects
   ```rust
   let arena = Arena::new();
   let x = arena.alloc(42);
   // No way to free just x!
   // Drop arena → everything freed at once
   ```

3. **Perfect for Phase-Based Allocation**: Allocate many objects, use them, discard all at once
   ```rust
   fn parse(input: &str) -> Result<i64, String> {
       let arena = Arena::new();        // Create arena
       let ast = parse_to_ast(input, &arena);  // Allocate many nodes
       let result = eval(ast);          // Use AST
       Ok(result)
       // arena dropped here → all nodes freed instantly
   }
   ```

4. **Better Cache Locality**: Objects allocated sequentially are stored sequentially
   ```
   Box allocations (scattered in memory):
   Node1 @ 0x1000, Node2 @ 0x5000, Node3 @ 0x2000  ← Cache misses!

   Arena allocations (contiguous):
   Node1 @ 0x1000, Node2 @ 0x1018, Node3 @ 0x1030  ← Cache hits!
   ```

**Performance Comparison**:

| Operation | Box<T> | Arena<T> | Speedup |
|-----------|--------|----------|---------|
| Single allocation | ~75ns | ~3ns | 25x |
| 10,000 nodes | 750μs | 30μs | 25x |
| Cache misses | High | Low | 2-3x |
| **Total speedup** | - | - | **15-20x** |

**When to Use Arena Allocation**:

✅ **Good fit**:
- Parsing (ASTs, JSON, XML)
- Compilers (IR nodes, symbol tables)
- Game engines (per-frame objects)
- Request handlers (per-request temps)
- Any "allocate many, free all" pattern

❌ **Bad fit**:
- Long-lived objects with individual lifecycles
- Objects that need to be freed independently
- Incremental data structures (growing over time)

---

### Memory Alignment: Why It Matters

**The Problem**: CPUs require certain types to be stored at addresses that are multiples of their size. Misaligned access can crash (ARM) or be slow (x86).

```rust
// Good: u64 at address 0x1000 (8-byte aligned)
// Bad:  u64 at address 0x1001 (not 8-byte aligned) → CRASH or 2x slower!
```

**Alignment Requirements**:

| Type | Size | Alignment | Valid Addresses |
|------|------|-----------|-----------------|
| `u8` | 1 byte | 1 | Any address |
| `u16` | 2 bytes | 2 | 0x1000, 0x1002, 0x1004... |
| `u32` | 4 bytes | 4 | 0x1000, 0x1004, 0x1008... |
| `u64` | 8 bytes | 8 | 0x1000, 0x1008, 0x1010... |
| `Expr` | varies | 8 (largest field) | 0x1000, 0x1008... |

**Why Alignment in Arena Allocation**:

When we bump allocate, we must ensure each allocation is properly aligned:

```rust
fn alloc<T>(&self, value: T) -> &mut T {
    let size = std::mem::size_of::<T>();
    let align = std::mem::align_of::<T>();  // e.g., 8 for u64

    let current_len = self.storage.borrow().len();

    // Calculate padding needed for alignment
    let padding = (align - (current_len % align)) % align;
    let start = current_len + padding;  // Now aligned!

    // ... allocate at start ...
}
```

**Example Calculation**:

```
Current position: 13 (after allocating u8 at 12)
Want to allocate u64 (needs 8-byte alignment)

current_len = 13
align = 8
current_len % align = 13 % 8 = 5
padding = (8 - 5) % 8 = 3

start = 13 + 3 = 16 (divisible by 8 ✓)

Memory layout:
┌────────────────────────────────────┐
│ [prev][X][X][X][u64 goes here...] │
└────────────────────────────────────┘
       ^pad^    ^16 (aligned)
```

**The Modulo Formula**:
```rust
let padding = (align - (current_len % align)) % align;
```

Why the second `% align`? Handle the case when already aligned:
```
current_len = 16, align = 8
16 % 8 = 0  (already aligned)
(8 - 0) % 8 = 0  (no padding needed) ✓

Without the second %:
(8 - 0) = 8  (would add unnecessary padding!) ✗
```

---

### Unsafe Rust: Working with Raw Pointers

**Why We Need Unsafe**:

Rust's safety guarantees rely on the borrow checker. But arena allocation requires operations the borrow checker can't verify:
- Converting raw bytes to typed references
- Writing to uninitialized memory
- Extending reference lifetimes

**The Unsafe Operations We Use**:

1. **`std::ptr::write`**: Write a value to a raw pointer without reading the old value (for uninitialized memory)

```rust
unsafe {
    std::ptr::write(ptr, value);  // Write value to ptr
    // Doesn't call Drop on old value (because there isn't one!)
}
```

vs. normal assignment:
```rust
*ptr = value;  // Reads old value, calls Drop, then writes new value
```

2. **Raw pointer casting**: Convert between pointer types

```rust
let byte_ptr: *mut u8 = &mut storage[start];
let typed_ptr: *mut T = byte_ptr as *mut T;  // Cast to correct type
```

3. **Dereferencing raw pointers**: Access data through raw pointer

```rust
unsafe {
    let reference: &mut T = &mut *typed_ptr;  // Create reference
}
```

**Our Arena Allocation (with unsafe)**:

```rust
fn alloc<T>(&self, value: T) -> &mut T {
    let mut storage = self.storage.borrow_mut();

    // Safe: size and alignment calculation
    let size = std::mem::size_of::<T>();
    let align = std::mem::align_of::<T>();

    // Safe: calculating aligned position
    let current_len = storage.len();
    let padding = (align - (current_len % align)) % align;
    let start = current_len + padding;

    // Safe: resizing buffer
    storage.resize(start + size, 0);

    // UNSAFE: Casting bytes to typed pointer
    let ptr = &mut storage[start] as *mut u8 as *mut T;

    unsafe {
        // UNSAFE: Writing to uninitialized memory
        std::ptr::write(ptr, value);

        // UNSAFE: Creating reference from raw pointer
        &mut *ptr
    }
}
```

**Why Each `unsafe` Is Safe** (programmer reasoning):

1. **Casting**: We ensured `start` is aligned for `T`, so `ptr` is valid
2. **`std::ptr::write`**: We resized storage to have space, so `ptr` points to valid memory
3. **Creating reference**: The memory contains a valid `T` (we just wrote it), and the lifetime is tied to the arena

**The Contract**:

When you write `unsafe`, you're telling the compiler: "I've verified these safety properties manually. Trust me."

**What Could Go Wrong** (if we make mistakes):

```rust
// Bug 1: Forget to align → CRASH
let ptr = &mut storage[current_len] as *mut u8 as *mut T;  // Not aligned!

// Bug 2: Not enough space → CORRUPTION
// storage.resize(start + size, 0);  // Forgot this line!
unsafe { std::ptr::write(ptr, value); }  // Writes past end of buffer!

// Bug 3: Use after free → UNDEFINED BEHAVIOR
let expr = {
    let arena = Arena::new();
    arena.alloc(Expr::Literal(42))
};  // arena dropped, but expr still references it!
```

**Guidelines for Unsafe Code**:

1. **Minimize unsafe blocks**: Keep them small and well-commented
2. **Document invariants**: Explain why the unsafe code is safe
3. **Test thoroughly**: Unsafe bugs can be silent (corruption, not crashes)
4. **Use tools**: Miri can detect some undefined behavior

---

### RefCell: Interior Mutability for Arena

**The Problem**: The arena's `alloc()` method takes `&self` (shared reference), but needs to modify the internal `Vec<u8>` storage.

```rust
impl Arena {
    fn alloc<T>(&self, value: T) -> &mut T {
        // Need to mutate storage, but only have &self!
        self.storage.push(value);  // ❌ Error: can't mutate through &self
    }
}
```

**Why `&self` Instead of `&mut self`?**

We need multiple references into the arena simultaneously:

```rust
let arena = Arena::new();
let two = arena.alloc(Expr::Literal(2));      // Borrow 1
let three = arena.alloc(Expr::Literal(3));    // Borrow 2
let sum = arena.alloc(Expr::BinOp {
    op: Add,
    left: two,     // Still using borrow 1
    right: three,  // Still using borrow 2
});
```

With `&mut self`, we could only have one allocation at a time!

**The Solution: RefCell**:

```rust
use std::cell::RefCell;

struct Arena {
    storage: RefCell<Vec<u8>>,  // Interior mutability
}

impl Arena {
    fn alloc<T>(&self, value: T) -> &mut T {
        let mut storage = self.storage.borrow_mut();  // Get mutable borrow
        // ... modify storage ...
        // Borrow released when `storage` drops at end of function
    }
}
```

**How RefCell Works**:

- **Compile-time**: Allows mutation through `&self`
- **Runtime**: Tracks borrows dynamically, panics if rules violated
- **Cost**: Small overhead (~2-3 CPU cycles) for borrow checking

**Borrow Rules** (enforced at runtime):
- Multiple readers OR one writer
- Not both simultaneously

**Example of RefCell Panic**:

```rust
let arena = Arena::new();
let storage1 = arena.storage.borrow_mut();  // Acquire write lock
let storage2 = arena.storage.borrow_mut();  // ❌ PANIC: already borrowed!
```

**Why Our Code Is Safe**:

Each `alloc()` call borrows, does its work, and releases before the next borrow:

```rust
fn alloc(&self, value: T) -> &mut T {
    {
        let mut storage = self.storage.borrow_mut();  // Borrow
        // ... work ...
    }  // Borrow released here

    // Return reference (points into arena, not into RefCell)
}
```

The returned reference points to data in the arena's buffer, not the `RefCell` itself, so we can have many of them simultaneously.

---

### Result Type: Structured Error Handling

**The Problem**: Parsing can fail in many ways—invalid syntax, unexpected tokens, division by zero. We need to propagate errors up the call stack with context.

```rust
// Bad: Using panic for expected failures
fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::BinOp { op: Div, right, .. } if right.eval() == 0 => {
            panic!("Division by zero");  // ❌ Too harsh!
        }
        // ...
    }
}

// Good: Using Result
fn eval(expr: &Expr) -> Result<i64, String> {
    match expr {
        Expr::BinOp { op: Div, left, right } => {
            let r = right.eval()?;  // Propagate error if any
            if r == 0 {
                return Err("Division by zero".to_string());
            }
            Ok(left.eval()? / r)
        }
        // ...
    }
}
```

**Result Type Basics**:

```rust
enum Result<T, E> {
    Ok(T),      // Success case with value
    Err(E),     // Failure case with error
}
```

**The `?` Operator**: Syntactic sugar for error propagation

```rust
// Without ?
let left_val = match left.eval() {
    Ok(v) => v,
    Err(e) => return Err(e),  // Propagate error
};

// With ? (equivalent)
let left_val = left.eval()?;
```

**Our Error Types**:

```rust
// Simple string errors (fine for learning project)
Result<i64, String>
Result<&'arena Expr<'arena>, String>
Result<Vec<Token>, String>

// Examples:
Err("Expected number, found '+'"to_string())
Err("Division by zero".to_string())
Err("Unmatched parenthesis".to_string())
```

**Error Propagation in Parsing**:

```rust
fn parse_expr(&mut self) -> Result<&'arena Expr<'arena>, String> {
    let left = self.parse_term()?;  // If error, return immediately

    while matches!(self.peek(), Token::Plus | Token::Minus) {
        let op = self.consume_op()?;
        let right = self.parse_term()?;
        left = self.builder.binary(op, left, right);
    }

    Ok(left)
}
```

If any nested call returns `Err`, it bubbles up through all the `?` operators automatically!

---

### Pattern Matching: Structural Decomposition

Pattern matching is Rust's way of deconstructing enums and extracting data. This project uses it extensively.

**Basic Enum Matching**:

```rust
match expr {
    Expr::Literal(n) => Ok(*n),  // Extract the number
    Expr::BinOp { op, left, right } => {  // Extract all fields
        // Use op, left, right
    }
}
```

**Matching Tokens in Parser**:

```rust
match self.peek() {
    Token::Number(n) => {
        let n = *n;  // Copy the value
        self.advance();
        Ok(self.builder.literal(n))
    }
    Token::LeftParen => {
        self.advance();
        let expr = self.parse_expr()?;
        self.expect(Token::RightParen)?;
        Ok(expr)
    }
    token => Err(format!("Unexpected token: {:?}", token)),
}
```

**Guards and Nested Patterns**:

```rust
match token {
    Token::Plus | Token::Minus => OpType::Additive,  // Match either
    Token::Star | Token::Slash => OpType::Multiplicative,
    _ => unreachable!(),  // All other cases impossible here
}
```

**Destructuring in Let Bindings**:

```rust
let Expr::BinOp { op, left, right } = expr else {
    panic!("Expected binary operation");
};
```

---

### Performance Concepts: Why Arena Wins

**Cache Locality**:

Modern CPUs have a memory hierarchy. Accessing RAM is ~100x slower than L1 cache. Arena allocation keeps related objects close together in memory.

```
Box allocations:
Node1 @ 0x1000 → Node2 @ 0x8000 → Node3 @ 0x2000
Each access: likely cache miss (~100 cycles)

Arena allocations:
Node1 @ 0x1000 → Node2 @ 0x1018 → Node3 @ 0x1030
All in same cache line (~4 cycles after first load)
```

**Allocation Overhead**:

```rust
// Box<T>: Global allocator
Box::new(value)
  → lock allocator mutex (~10ns)
  → find free block (~20ns)
  → update metadata (~10ns)
  → unlock mutex (~10ns)
  Total: ~50-100ns

// Arena: Bump allocator
arena.alloc(value)
  → calculate position (~1ns)
  → write value (~1ns)
  → return reference (~1ns)
  Total: ~2-5ns

Speedup: 25x per allocation!
```

**Deallocation Overhead**:

```rust
// Box<T>: Individual drops
drop(box1);  // ~50ns
drop(box2);  // ~50ns
drop(box3);  // ~50ns
// ... thousands of drops

// Arena: Bulk free
drop(arena);  // ~10ns total
// OS reclaims entire buffer at once
```

---

## Connection to This Project

In this project, you'll implement the complete pipeline:

1. **Milestone 1-2**: Define AST types (`Expr` enum)
2. **Milestone 3-4**: Optimize AST allocation with arena (bump allocator)
3. **Milestone 5**: Build the lexer to convert text to tokens
4. **Milestone 6**: Implement recursive descent parser
5. **Milestone 7**: Compare performance with traditional allocation



## Build The Project

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


## Complete Working Example
```rust
// Common types used across milestones
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

// Milestone 1: Define AST Types
mod milestone_1 {
    use super::OpType;

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

    #[cfg(test)]
    mod tests {
        use super::*;

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
            let add = Expr::BinOp { op: OpType::Add, left: &two, right: &three, };
            let mul = Expr::BinOp { op: OpType::Mul, left: &add, right: &four, };
            assert_eq!(mul.eval(), Ok(20));
        }

        #[test]
        fn test_division_by_zero() {
            let ten = Expr::Literal(10);
            let zero = Expr::Literal(0);
            let expr = Expr::BinOp { op: OpType::Div, left: &ten, right: &zero, };
            assert!(expr.eval().is_err());
        }
    }
}

// Milestone 2: Box-Based Expression Trees
mod milestone_2 {
    use super::OpType;

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

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_box_expr_literal() {
            assert_eq!(BoxExprBuilder::literal(42).eval(), Ok(42));
        }
        #[test]
        fn test_box_expr_addition() {
            let expr = BoxExprBuilder::add(BoxExprBuilder::literal(10), BoxExprBuilder::literal(5));
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
    }
}

// Milestone 3: Simple Bump Allocator
// Milestone 4: Build Expressions in Arena
// We combine these as the Arena and Builder are tightly coupled.
mod milestone_3_4 {
    use super::OpType;
    use std::cell::RefCell;
    use std::mem;

    // From Milestone 1
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
                    op.eval(left.eval()?, right.eval()?)
                }
            }
        }
    }

    // Milestone 3
    pub struct Arena {
        storage: RefCell<Vec<u8>>,
    }

    impl Arena {
        pub fn new() -> Self {
            Arena {
                storage: RefCell::new(Vec::with_capacity(4096)),
            }
        }

        pub fn alloc<'arena, T>(&'arena self, value: T) -> &'arena T {
            let mut storage = self.storage.borrow_mut();
            let size = mem::size_of::<T>();
            let align = mem::align_of::<T>();
            let current_len = storage.len();
            let padding = (align - (current_len % align)) % align;
            let start = current_len + padding;
            storage.resize(start + size, 0);
            let ptr = &mut storage[start] as *mut u8 as *mut T;
            unsafe {
                ptr.write(value);
                &*ptr
            }
        }
    }

    // Milestone 4
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
        pub fn binary(&self, op: OpType, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
            self.arena.alloc(Expr::BinOp { op, left, right })
        }
        pub fn add(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
            self.binary(OpType::Add, left, right)
        }
        pub fn sub(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
            self.binary(OpType::Sub, left, right)
        }
        pub fn mul(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
            self.binary(OpType::Mul, left, right)
        }
        pub fn div(&self, left: &'arena Expr<'arena>, right: &'arena Expr<'arena>) -> &'arena Expr<'arena> {
            self.binary(OpType::Div, left, right)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Milestone 3 Tests
        #[test]
        fn test_arena_alloc_int() {
            let arena = Arena::new();
            let x = arena.alloc(42);
            assert_eq!(*x, 42);
        }
        #[test]
        fn test_arena_multiple_allocs() {
            let arena = Arena::new();
            assert_eq!(*arena.alloc(1), 1);
            assert_eq!(*arena.alloc(2), 2);
            assert_eq!(*arena.alloc(3), 3);
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
            let num = arena.alloc(1234u64);
            assert_eq!((num as *const u64 as usize) % 8, 0);
        }

        // Milestone 4 Tests
        #[test]
        fn test_builder() {
            let arena = Arena::new();
            let builder = ExprBuilder::new(&arena);
            let expr = builder.mul(builder.add(builder.literal(2), builder.literal(3)), builder.literal(4));
            assert_eq!(expr.eval(), Ok(20));
        }
        #[test]
        fn test_complex_expression() {
            let arena = Arena::new();
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
    }
}

// Milestone 5 & 6: Lexer and Parser
mod milestone_5_6 {
    use super::milestone_3_4::{Arena, Expr, ExprBuilder};
    use super::OpType;

    // Milestone 5
    #[derive(Debug, PartialEq, Clone)]
    pub enum Token {
        Number(i64), Plus, Minus, Star, Slash, LeftParen, RightParen, End,
    }

    pub struct Lexer {
        input: Vec<char>,
        position: usize,
    }

    impl Lexer {
        pub fn new(input: &str) -> Self {
            Lexer { input: input.chars().collect(), position: 0 }
        }
        fn peek(&self) -> Option<char> { self.input.get(self.position).copied() }
        fn advance(&mut self) { self.position += 1; }
        fn skip_whitespace(&mut self) {
            while let Some(c) = self.peek() {
                if c.is_whitespace() { self.advance(); } else { break; }
            }
        }
        fn read_number(&mut self) -> i64 {
            let mut num = 0;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num = num * 10 + (c as i64 - '0' as i64);
                    self.advance();
                } else { break; }
            }
            num
        }
        pub fn next_token(&mut self) -> Result<Token, String> {
            self.skip_whitespace();
            match self.peek() {
                None => Ok(Token::End),
                Some(c) => match c {
                    '0'..='9' => Ok(Token::Number(self.read_number())),
                    '+' => { self.advance(); Ok(Token::Plus) },
                    '-' => { self.advance(); Ok(Token::Minus) },
                    '*' => { self.advance(); Ok(Token::Star) },
                    '/' => { self.advance(); Ok(Token::Slash) },
                    '(' => { self.advance(); Ok(Token::LeftParen) },
                    ')' => { self.advance(); Ok(Token::RightParen) },
                    _ => Err(format!("Unexpected character '{}'", c)),
                },
            }
        }
        pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
            let mut tokens = Vec::new();
            loop {
                let token = self.next_token()?;
                tokens.push(token.clone());
                if token == Token::End { break; }
            }
            Ok(tokens)
        }
    }

    // Milestone 6
    pub struct Parser<'arena> {
        tokens: Vec<Token>,
        position: usize,
        builder: ExprBuilder<'arena>,
    }
    
    impl<'arena> Parser<'arena> {
        pub fn new(tokens: Vec<Token>, arena: &'arena Arena) -> Self {
            Parser { tokens, position: 0, builder: ExprBuilder::new(arena) }
        }
        fn peek(&self) -> &Token { self.tokens.get(self.position).unwrap_or(&Token::End) }
        fn advance(&mut self) { self.position += 1; }
        fn expect(&mut self, expected: Token) -> Result<(), String> {
            if self.peek() == &expected { self.advance(); Ok(()) } 
            else { Err(format!("Expected {:?}, found {:?}", expected, self.peek())) }
        }
        fn parse_factor(&mut self) -> Result<&'arena Expr<'arena>, String> {
            match self.peek().clone() {
                Token::Number(n) => { self.advance(); Ok(self.builder.literal(n)) },
                Token::LeftParen => {
                    self.advance();
                    let expr = self.parse_expr()?;
                    self.expect(Token::RightParen)?;
                    Ok(expr)
                },
                token => Err(format!("Expected number or '(', found {:?}", token)),
            }
        }
        fn parse_term(&mut self) -> Result<&'arena Expr<'arena>, String> {
            let mut left = self.parse_factor()?;
            while let Token::Star | Token::Slash = self.peek() {
                let op = if matches!(self.peek(), Token::Star) { OpType::Mul } else { OpType::Div };
                self.advance();
                let right = self.parse_factor()?;
                left = self.builder.binary(op, left, right);
            }
            Ok(left)
        }
        fn parse_expr(&mut self) -> Result<&'arena Expr<'arena>, String> {
            let mut left = self.parse_term()?;
            while let Token::Plus | Token::Minus = self.peek() {
                let op = if matches!(self.peek(), Token::Plus) { OpType::Add } else { OpType::Sub };
                self.advance();
                let right = self.parse_term()?;
                left = self.builder.binary(op, left, right);
            }
            Ok(left)
        }
        pub fn parse(&mut self) -> Result<&'arena Expr<'arena>, String> {
            let expr = self.parse_expr()?;
            if self.peek() != &Token::End { Err(format!("Unexpected token: {:?}", self.peek())) }
            else { Ok(expr) }
        }
    }
    
    // Helper for tests
    pub fn parse_and_eval(input: &str) -> Result<i64, String> {
        let arena = Arena::new();
        let tokens = Lexer::new(input).tokenize()?;
        let expr = Parser::new(tokens, &arena).parse()?;
        expr.eval()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Milestone 5 Tests
        #[test] fn test_lexer_numbers() {
            let tokens = Lexer::new("123 456").tokenize().unwrap();
            assert_eq!(tokens, vec![Token::Number(123), Token::Number(456), Token::End]);
        }
        #[test] fn test_lexer_operators() {
            let tokens = Lexer::new("+ - * /").tokenize().unwrap();
            assert_eq!(tokens, vec![Token::Plus, Token::Minus, Token::Star, Token::Slash, Token::End]);
        }
        #[test] fn test_lexer_expression() {
            let tokens = Lexer::new("(2 + 3) * 4").tokenize().unwrap();
            assert_eq!(tokens, vec![Token::LeftParen, Token::Number(2), Token::Plus, Token::Number(3), Token::RightParen, Token::Star, Token::Number(4), Token::End]);
        }
        #[test] fn test_lexer_error() { assert!(Lexer::new("2 & 3").tokenize().is_err()); }
        
        // Milestone 6 Tests
        #[test] fn test_parse_number() { assert_eq!(parse_and_eval("42"), Ok(42)); }
        #[test] fn test_parse_addition() { assert_eq!(parse_and_eval("2 + 3"), Ok(5)); }
        #[test] fn test_parse_precedence() { assert_eq!(parse_and_eval("2 + 3 * 4"), Ok(14)); }
        #[test] fn test_parse_parentheses() { assert_eq!(parse_and_eval("(2 + 3) * 4"), Ok(20)); }
        #[test] fn test_parse_complex() { assert_eq!(parse_and_eval("(10 - 5) * 2 + 8 / 4"), Ok(12)); }
        #[test] fn test_parse_nested() { assert_eq!(parse_and_eval("((1 + 2) * (3 + 4)) / (5 - 2)"), Ok(7)); }
        #[test] fn test_parse_error() {
            assert!(parse_and_eval("2 + + 3").is_err());
            assert!(parse_and_eval("(2 + 3").is_err());
        }
    }
}

// Milestone 7: Performance Comparison
mod milestone_7 {
    use super::milestone_2::{BoxExprBuilder};
    use super::milestone_3_4::{Arena, ExprBuilder};
    use std::time::Instant;

    pub fn run_benchmarks() {
        println!("\n--- Running Performance Benchmarks ---");
        let box_duration = benchmark_box();
        let arena_duration = benchmark_arena();
        println!("Box      : {:?}", box_duration);
        println!("Arena    : {:?}", arena_duration);
        if arena_duration.as_nanos() > 0 {
            let factor = box_duration.as_nanos() as f64 / arena_duration.as_nanos() as f64;
            println!("Speedup  : {:.2}x", factor);
        }
        println!("------------------------------------");
    }

    fn benchmark_arena() -> std::time::Duration {
        let start = Instant::now();
        for _ in 0..10000 {
            let arena = Arena::new();
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
        start.elapsed()
    }

    fn benchmark_box() -> std::time::Duration {
        let start = Instant::now();
        for _ in 0..10000 {
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
}
```

// To run benchmarks, you could add this to a main function:
// milestone_7::run_benchmarks();
