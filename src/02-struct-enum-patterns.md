# Struct & Enum Patterns
This chapter explores struct and enum patterns for type-safe data modeling: choosing struct types, newtype wrappers for domain types, zero-sized types for compile-time guarantees, enums for variants, and advanced techniques for memory efficiency and recursion.

## Pattern 1: Struct Design Patterns

**Problem**: Confusion about when to use named field structs vs tuple structs vs unit structs. Named fields verbose for simple types (Point needs x, y, z names). Tuple structs unclear which field is which (Point3D(1.0, 2.0, 3.0)—which is x?). No semantic distinction between similar types (both u64 but UserId vs OrderId). Zero-sized marker types need PhantomData but compiler warns about unused type parameters. Want compile-time state tracking without runtime cost.

**Solution**: Use named field structs for data models where field names add clarity (`struct User { id: u64, username: String }`). Use tuple structs for simple types where position conveys meaning (`struct Point3D(f64, f64, f64)`) and newtype pattern (`struct UserId(u64)`). Use unit structs for zero-sized markers in typestate pattern (`struct Authenticated;`). PhantomData for phantom type parameters (`PhantomData<State>`). Choose based on: need for field names, distinctness requirements, zero-size marker needs.

**Why It Matters**: Named fields self-document: `user.email` is clear, `user.2` is not. Tuple structs create distinct types: UserId(1) and OrderId(1) are different types despite both wrapping u64. Unit structs enable typestate pattern at zero runtime cost: `Database<Authenticated>` vs `Database<Unauthenticated>` enforced at compile-time. Field reordering: named fields can reorder without breaking code, tuple fields can't. Memory layout: all three have same efficiency, choice is semantic not performance. Clarity vs brevity trade-off: named for complex data, tuple for simple wrappers.

**Use Cases**: Named field structs for data models (User, Config, Request/Response), domain entities, API types, database models, complex state. Tuple structs for newtype pattern (UserId, Kilometers), coordinates (Point3D, Color RGB), simple wrappers, creating distinct types from primitives. Unit structs for typestate markers (Authenticated, Open/Closed), phantom type parameters, zero-cost compile-time tags, capability markers (ReadPermission), builder pattern states.

### Example: Named Field Structs

```rust
#[derive(Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

impl User {
    fn new(id: u64, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            active: true,
        }
    }

    fn deactivate(&mut self) {
        self.active = false;
    }
}

// Usage
let user = User::new(1, "alice".to_string(), "alice@example.com".to_string());
println!("User {} is active: {}", user.username, user.active);
```

**Why this matters:** Named fields provide self-documenting code. When you see `user.email`, the intent is clear. They also allow field reordering without breaking code.

### Example: Tuple Structs

Tuple structs are useful when field names would be redundant or when you want to create distinct types:

```rust
// Coordinates where position matters more than names
struct Point3D(f64, f64, f64);

// Type-safe wrappers (newtype pattern)
struct Kilometers(f64);
struct Miles(f64);

impl Point3D {
    fn origin() -> Self {
        Point3D(0.0, 0.0, 0.0)
    }

    fn distance_from_origin(&self) -> f64 {
        (self.0.powi(2) + self.1.powi(2) + self.2.powi(2)).sqrt()
    }
}

// Usage
let point = Point3D(3.0, 4.0, 0.0);
println!("Distance: {}", point.distance_from_origin());

// Type safety prevents mixing units
let distance_km = Kilometers(100.0);
let distance_mi = Miles(62.0);
// let total = distance_km.0 + distance_mi.0; // Compiles but semantically wrong!
```

**The pattern:** Use tuple structs when the structure itself conveys meaning more than field names would. They're particularly powerful for the newtype pattern.

### Example: Unit Structs

Unit structs carry no data but can implement traits and provide type-level information:

```rust
// Marker types for type-level programming
struct Authenticated;
struct Unauthenticated;

// Zero-sized types for phantom data
struct Database<State> {
    connection_string: String,
    _state: std::marker::PhantomData<State>,
}

impl Database<Unauthenticated> {
    fn new(connection_string: String) -> Self {
        Database {
            connection_string,
            _state: std::marker::PhantomData,
        }
    }

    fn authenticate(self, password: &str) -> Result<Database<Authenticated>, String> {
        if password == "secret" {
            Ok(Database {
                connection_string: self.connection_string,
                _state: std::marker::PhantomData,
            })
        } else {
            Err("Invalid password".to_string())
        }
    }
}

impl Database<Authenticated> {
    fn query(&self, sql: &str) -> Vec<String> {
        println!("Executing: {}", sql);
        vec!["result1".to_string(), "result2".to_string()]
    }
}

// Usage
let db = Database::new("postgres://localhost".to_string());
// db.query("SELECT *"); // Error! Can't query unauthenticated database
let db = db.authenticate("secret").unwrap();
let results = db.query("SELECT * FROM users"); // Now this works
```

**The insight:** Unit structs enable compile-time state tracking without runtime overhead. This is the typestate pattern in action.

## Pattern 2: Newtype and Wrapper Patterns

**Problem**: Mixing incompatible types causes bugs—UserId(42) and OrderId(42) both u64, accidentally pass OrderId to get_user(). No invariant enforcement: PositiveInteger is just i32, negative values slip through. Raw primitives lack domain meaning: is this u64 a UserId, timestamp, or count? Can't implement external traits on external types (orphan rule): want `impl Display for Vec<T>` but can't. Defensive validation everywhere: every function checks if number is positive. Type aliases don't create new types: `type UserId = u64` doesn't prevent mixing.

**Solution**: Newtype pattern: `struct UserId(u64)` creates distinct type wrapping u64. Validated wrappers enforce invariants: `PositiveInteger::new()` returns Result, guarantees positivity. Smart constructors prevent invalid construction. Deref trait for transparent access: `impl Deref for Validated<T>` allows calling T's methods. Derive traits to propagate functionality (Debug, Clone, PartialEq). Accessor methods (`.get()`) when direct field access undesired. Workaround orphan rule: wrap external type to impl external trait.

**Why It Matters**: Type safety prevents bugs: compiler rejects `get_user(order_id)`, catches at compile-time not runtime. Invariants in types: once you have PositiveInteger, it's guaranteed positive—no defensive checks needed. Zero runtime cost: newtype compiles to same representation as wrapped type. Self-documenting code: UserId vs u64 shows intent. Orphan rule workaround: `struct Wrapper(Vec<T>)` lets you `impl Display for Wrapper`. API clarity: domain types vs primitives makes interfaces clearer. Eliminates entire bug classes: no mixing IDs, no invalid states.

**Use Cases**: Domain-specific IDs (UserId, OrderId, ProductId—prevent mixing), units (Kilometers, Miles, Seconds—prevent unit confusion), validated types (PositiveInteger, NonEmptyString, Email—enforce invariants), semantic wrappers (ConnectionString, Password—hide internals), orphan rule workaround (wrap external type to impl trait), database handles (ConnectionId, SessionId), newtype index pattern (prevent indexing wrong Vec), sensitive data (Password type hides value in Debug).

### Example: Newtype
```rust
use std::fmt;

// Newtype for semantic clarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UserId(u64);

#[derive(Debug, Clone, Copy)]
struct OrderId(u64);

// Prevent accidentally mixing IDs
fn get_user(id: UserId) -> User {
    println!("Fetching user {}", id.0);
    // ... fetch user
    unimplemented!()
}

// This won't compile:
// let order_id = OrderId(123);
// get_user(order_id); // Type error!

// Wrapper for adding functionality
struct PositiveInteger(i32);

impl PositiveInteger {
    fn new(value: i32) -> Result<Self, String> {
        if value > 0 {
            Ok(PositiveInteger(value))
        } else {
            Err(format!("{} is not positive", value))
        }
    }

    fn get(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for PositiveInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Usage prevents invalid states
let num = PositiveInteger::new(42).unwrap();
// let invalid = PositiveInteger::new(-5); // Returns Err
```

**Why wrappers matter:** They encode invariants in the type system. Once you have a `PositiveInteger`, you know it's valid. This eliminates defensive checks throughout your codebase.

### Example: Transparent Wrappers with Deref

For ergonomic access to the wrapped type:

```rust
use std::ops::Deref;

struct Validated<T> {
    value: T,
    validated_at: std::time::Instant,
}

impl<T> Validated<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            validated_at: std::time::Instant::now(),
        }
    }

    fn age(&self) -> std::time::Duration {
        self.validated_at.elapsed()
    }
}

impl<T> Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// Usage
let validated_string = Validated::new("hello".to_string());
println!("Length: {}", validated_string.len()); // Deref to String
println!("Age: {:?}", validated_string.age());  // Validated method
```

## Pattern 3: Struct Memory and Update Patterns

**Problem**: Understanding struct memory layout and update syntax. Struct update syntax (`..other`) can cause partial moves. Efficient struct transformations unclear. Copy vs Move field interaction confusing. Immutable update patterns feel clumsy.

**Solution**: Struct update syntax `..other` copies/moves remaining fields. Understand Copy fields copy, non-Copy fields move. Clone before update to preserve original. Builder-style methods consume `self`, return `Self`. Use `mut self` for in-place transformation chains.

**Why It Matters**: Enables ergonomic immutable updates. Prevents accidental partial moves. Foundation for builder patterns (see Chapter 5). Memory layout awareness prevents surprises. Functional-style transformations possible.

**Use Cases**: Configuration updates, immutable data patterns, fluent method chains, functional-style transformations, struct cloning with modifications.

> **Note**: For compile-time state checking with phantom types and typestate patterns, see **Chapter 4: Pattern 6 (Phantom Types)** and **Chapter 5: Pattern 2 (Typestate Pattern)**.

### Example: Struct Update Syntax and Partial Moves

Rust's struct update syntax enables elegant immutable updates while understanding partial moves is crucial for ownership:

```rust
#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
    retry_count: u32,
}

impl Config {
    fn with_port(self, port: u16) -> Self {
        Config { port, ..self }
    }

    fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

// Builder-style updates
let config = Config {
    host: "localhost".to_string(),
    port: 8080,
    timeout_ms: 5000,
    retry_count: 3,
};

let new_config = Config {
    port: 9090,
    ..config // Moves non-Copy fields!
};

// config is now partially moved - can't use it anymore
// println!("{:?}", config); // Error!

// Safe pattern: clone when needed
let config = Config {
    host: "localhost".to_string(),
    port: 8080,
    timeout_ms: 5000,
    retry_count: 3,
};

let new_config = Config {
    host: "production.example.com".to_string(),
    ..config.clone()
};

// Both configs are usable
println!("Old: {:?}", config);
println!("New: {:?}", new_config);
```

**Understanding partial moves:**

```rust
struct Data {
    copyable: i32,      // Implements Copy
    moveable: String,   // Does not implement Copy
}

let data = Data {
    copyable: 42,
    moveable: "hello".to_string(),
};

// Partial move
let s = data.moveable;  // Moves String out
let n = data.copyable;  // Copies i32

// data.moveable is moved, but data.copyable is still accessible
println!("Copyable: {}", data.copyable); // OK
// println!("{}", data.moveable); // Error: value borrowed after move
```

**The pattern:** When building fluent APIs or config builders, be mindful of moves. Consider consuming self and returning Self, or use `&mut self` for in-place updates. For full builder pattern coverage, see **Chapter 5: Builder & API Design**.

## Pattern 4: Enum Design Patterns

**Problem**: Multiple related types without relationship (Circle, Rectangle, Triangle all separate). Optional data represented as separate Option fields messy. State machines unclear: is connection open or closed? Error types need context but String loses structure. No way to represent "one of several types". Exhaustive handling not enforced (forgot to handle variant). Multiple return types require Result<Box<dyn Trait>, Error>.

**Solution**: Enums for variants: `enum Shape { Circle(f64), Rectangle(f64, f64), Triangle(f64, f64, f64) }`. Pattern matching enforces exhaustiveness—compiler ensures all variants handled. Option<T>/Result<T, E> built-in enums. State machines as enums: `enum ConnectionState { Connecting, Connected, Disconnected }`. Custom error types as enums with context. Methods on enums via impl blocks. Exhaustive match prevents forgetting cases.

**Why It Matters**: Exhaustive matching catches all cases: adding enum variant causes compile errors in incomplete matches. Impossible states unrepresentable: can't have both Ok and Err simultaneously. Zero-cost abstraction: enum memory = size of largest variant + discriminant (usually 1 byte). Clear intent: enum shows all possibilities. Type-safe state machines: state transitions enforced. Error handling with context: custom error enum better than String. Pattern matching provides compile-time guarantees.

**Use Cases**: State machines (ConnectionState, HttpRequestState), error types (custom Error enums with variants), optional values (Option<T> replacement), message types (WebSocket messages, RPC calls), command patterns (Command enum with variants), AST nodes (expression trees, parse trees), protocol parsing (packet types), event handling (Event enum), sum types (Either, Result).

### Example: Basic Enum with Pattern Matching

```rust
// Model HTTP responses precisely
enum HttpResponse {
    Ok { body: String, headers: Vec<(String, String)> },
    Created { id: u64, location: String },
    NoContent,
    BadRequest { error: String },
    Unauthorized,
    NotFound,
    ServerError { message: String, details: Option<String> },
}

impl HttpResponse {
    fn status_code(&self) -> u16 {
        match self {
            HttpResponse::Ok { .. } => 200,
            HttpResponse::Created { .. } => 201,
            HttpResponse::NoContent => 204,
            HttpResponse::BadRequest { .. } => 400,
            HttpResponse::Unauthorized => 401,
            HttpResponse::NotFound => 404,
            HttpResponse::ServerError { .. } => 500,
        }
    }

    fn is_success(&self) -> bool {
        matches!(self, HttpResponse::Ok { .. } | HttpResponse::Created { .. } | HttpResponse::NoContent)
    }
}

// Usage
fn handle_request(path: &str) -> HttpResponse {
    match path {
        "/users" => HttpResponse::Ok {
            body: "[{\"id\": 1}]".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        },
        "/users/create" => HttpResponse::Created {
            id: 123,
            location: "/users/123".to_string(),
        },
        _ => HttpResponse::NotFound,
    }
}
```

**The power:** Each variant carries exactly the data it needs. No null or undefined—if a variant needs an ID, it has one.

### Example: Enum State Machines

Enums model state machines with exhaustive matching:

```rust
enum OrderStatus {
    Pending { items: Vec<String>, customer_id: u64 },
    Processing { order_id: u64, started_at: std::time::Instant },
    Shipped { order_id: u64, tracking_number: String },
    Delivered { order_id: u64, signature: Option<String> },
    Cancelled { order_id: u64, reason: String },
}

impl OrderStatus {
    fn process(self) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Pending { items, .. } => {
                if items.is_empty() {
                    return Err("Cannot process empty order".to_string());
                }
                Ok(OrderStatus::Processing {
                    order_id: 12345,
                    started_at: std::time::Instant::now(),
                })
            }
            _ => Err("Order is not in pending state".to_string()),
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(self, OrderStatus::Pending { .. } | OrderStatus::Processing { .. })
    }
}
```

> **Note:** For compile-time enforced state machines using types (typestate pattern), see **Chapter 5: Pattern 2 (Typestate Pattern)**.

## Pattern 5: Advanced Enum Techniques

**Problem**: Large enum variants waste memory—enum size = largest variant + discriminant, small variants waste space. Recursive enums (AST nodes with children) have infinite size—compiler error. Need interior mutability in enum. Want to extend enum behavior without modifying definition. Conversions between related types verbose. Enum size unpredictable affecting performance. Can't determine variant without matching.

**Solution**: Box large variants to reduce enum size: `enum Message { Small(u8), Large(Box<HugeStruct>) }` makes enum smaller. Box for recursive enums: `enum Node { Leaf(i32), Branch(Box<Node>, Box<Node>) }` breaks infinite size. Implement methods on enums with impl blocks. Match expressions for transformation. From/TryFrom traits for conversions. Use #[repr(u8)] for explicit discriminant. mem::size_of to check enum size. `matches!` macro for simple checks.

**Why It Matters**: Memory efficiency: boxing large variants reduces enum from size of largest to size of pointer. Recursion enabled: Box breaks infinite size allowing AST nodes, linked lists. Method dispatch via match: same interface different behavior per variant. Ergonomic APIs: methods on enums cleaner than separate functions. Conversion patterns: From/TryFrom standardize conversions. Performance: smaller enums = better cache locality. Discriminant control: #[repr] ensures layout for FFI. Variant checking: matches! avoids full match.

**Use Cases**: AST nodes (Box for child nodes), large variant optimization (Box rarely-used variants), state machines (methods for transitions), protocol handlers (dispatch via match), command dispatch (Command enum with execute() method), Option/Result extensions (custom enums), recursive data structures (trees, lists), error conversion (From for error types), enum-based visitor pattern.

### Example: Enum as Contract

```rust
// Model HTTP responses precisely
enum HttpResponse {
    Ok { body: String, headers: Vec<(String, String)> },
    Created { id: u64, location: String },
    NoContent,
    BadRequest { error: String },
    Unauthorized,
    NotFound,
    ServerError { message: String, details: Option<String> },
}

impl HttpResponse {
    fn status_code(&self) -> u16 {
        match self {
            HttpResponse::Ok { .. } => 200,
            HttpResponse::Created { .. } => 201,
            HttpResponse::NoContent => 204,
            HttpResponse::BadRequest { .. } => 400,
            HttpResponse::Unauthorized => 401,
            HttpResponse::NotFound => 404,
            HttpResponse::ServerError { .. } => 500,
        }
    }

    fn format(&self) -> String {
        match self {
            HttpResponse::Ok { body, .. } => body.clone(),
            HttpResponse::Created { id, location } => {
                format!("Created resource {} at {}", id, location)
            }
            HttpResponse::NoContent => String::new(),
            HttpResponse::BadRequest { error } => {
                format!("Bad request: {}", error)
            }
            HttpResponse::Unauthorized => "Unauthorized".to_string(),
            HttpResponse::NotFound => "Not found".to_string(),
            HttpResponse::ServerError { message, details } => {
                if let Some(d) = details {
                    format!("Error: {} ({})", message, d)
                } else {
                    format!("Error: {}", message)
                }
            }
        }
    }
}

// Usage
fn handle_request(path: &str) -> HttpResponse {
    match path {
        "/users" => HttpResponse::Ok {
            body: "[{\"id\": 1, \"name\": \"Alice\"}]".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        },
        "/users/create" => HttpResponse::Created {
            id: 123,
            location: "/users/123".to_string(),
        },
        _ => HttpResponse::NotFound,
    }
}
```

**The power:** Each variant can carry exactly the data it needs. There's no `null` or `undefined` - if a variant needs an ID, it has one.

### Example: Recursive Enums with Box

```rust
// Binary tree - recursive enum needs Box to break infinite size
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T: std::fmt::Debug> Tree<T> {
    fn depth(&self) -> usize {
        match self {
            Tree::Leaf(_) => 1,
            Tree::Node { left, right, .. } => {
                1 + left.depth().max(right.depth())
            }
        }
    }
}

// AST nodes often use Box for recursion
enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> i32 {
        match self {
            Expr::Number(n) => *n,
            Expr::Add(l, r) => l.eval() + r.eval(),
            Expr::Mul(l, r) => l.eval() * r.eval(),
        }
    }
}
```

### Example: Memory-Efficient Large Variants

```rust
// Without Box: enum size = size of largest variant (LargeData)
enum Inefficient {
    Small(u8),
    Large([u8; 1024]),  // 1KB - every variant takes this space
}

// With Box: enum size = size of pointer (8 bytes on 64-bit)
enum Efficient {
    Small(u8),
    Large(Box<[u8; 1024]>),  // Only allocates when this variant is used
}

fn check_sizes() {
    println!("Inefficient: {} bytes", std::mem::size_of::<Inefficient>());
    println!("Efficient: {} bytes", std::mem::size_of::<Efficient>());
}
```

## Pattern 6: Visitor Pattern with Enums

The visitor pattern in Rust leverages enums for traversing complex structures:

```rust
// AST for a simple expression language
enum Expr {
    Number(f64),
    Variable(String),
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

enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

enum UnOp {
    Negate,
    Abs,
}

// Visitor trait
trait ExprVisitor {
    type Output;

    fn visit(&mut self, expr: &Expr) -> Self::Output {
        match expr {
            Expr::Number(n) => self.visit_number(*n),
            Expr::Variable(name) => self.visit_variable(name),
            Expr::BinaryOp { op, left, right } => {
                self.visit_binary_op(op, left, right)
            }
            Expr::UnaryOp { op, expr } => {
                self.visit_unary_op(op, expr)
            }
        }
    }

    fn visit_number(&mut self, n: f64) -> Self::Output;
    fn visit_variable(&mut self, name: &str) -> Self::Output;
    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Self::Output;
    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> Self::Output;
}

// Pretty printer visitor
struct PrettyPrinter {
    indent: usize,
}

impl ExprVisitor for PrettyPrinter {
    type Output = String;

    fn visit_number(&mut self, n: f64) -> String {
        n.to_string()
    }

    fn visit_variable(&mut self, name: &str) -> String {
        name.to_string()
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> String {
        let op_str = match op {
            BinOp::Add => "+",
            BinOp::Subtract => "-",
            BinOp::Multiply => "*",
            BinOp::Divide => "/",
        };

        format!("({} {} {})",
            self.visit(left),
            op_str,
            self.visit(right))
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> String {
        let op_str = match op {
            UnOp::Negate => "-",
            UnOp::Abs => "abs",
        };

        format!("{}({})", op_str, self.visit(expr))
    }
}

// Evaluator visitor
struct Evaluator {
    variables: std::collections::HashMap<String, f64>,
}

impl ExprVisitor for Evaluator {
    type Output = Result<f64, String>;

    fn visit_number(&mut self, n: f64) -> Self::Output {
        Ok(n)
    }

    fn visit_variable(&mut self, name: &str) -> Self::Output {
        self.variables.get(name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: {}", name))
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Self::Output {
        let left_val = self.visit(left)?;
        let right_val = self.visit(right)?;

        match op {
            BinOp::Add => Ok(left_val + right_val),
            BinOp::Subtract => Ok(left_val - right_val),
            BinOp::Multiply => Ok(left_val * right_val),
            BinOp::Divide => {
                if right_val == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left_val / right_val)
                }
            }
        }
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> Self::Output {
        let val = self.visit(expr)?;
        match op {
            UnOp::Negate => Ok(-val),
            UnOp::Abs => Ok(val.abs()),
        }
    }
}

// Usage
fn demo_visitor() {
    // (3 + 4) * 2
    let expr = Expr::BinaryOp {
        op: BinOp::Multiply,
        left: Box::new(Expr::BinaryOp {
            op: BinOp::Add,
            left: Box::new(Expr::Number(3.0)),
            right: Box::new(Expr::Number(4.0)),
        }),
        right: Box::new(Expr::Number(2.0)),
    };

    let mut printer = PrettyPrinter { indent: 0 };
    println!("Expression: {}", printer.visit(&expr));

    let mut evaluator = Evaluator {
        variables: std::collections::HashMap::new(),
    };
    match evaluator.visit(&expr) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

**The pattern:** Visitors separate traversal logic from data structure. You can add new operations without modifying the enum definition.

### Summary

This chapter covered struct and enum patterns for type-safe data modeling:

1. **Struct Design Patterns**: Named fields for clarity, tuple for newtypes/position, unit for markers
2. **Newtype and Wrapper Patterns**: Domain IDs, validated types, invariant enforcement, orphan rule workaround
3. **Struct Memory and Update Patterns**: Struct update syntax, partial moves, builder-style transformations
4. **Enum Design Patterns**: Variants for related types, exhaustive matching, state machines, error types
5. **Advanced Enum Techniques**: Box for large/recursive variants, methods on enums, memory optimization
6. **Visitor Pattern**: Separating traversal logic from data structure with enums

**Key Takeaways**:
- Struct choice is semantic: named for data models, tuple for wrappers, unit for markers
- Newtype pattern: UserId(u64) vs OrderId(u64) prevents mixing at zero cost
- Enums enforce exhaustiveness: adding variant causes compile errors in incomplete matches
- Box breaks infinite size for recursive enums and reduces memory for large variants

**Design Principles**:
- Use named fields when clarity matters, tuple when type itself is meaningful
- Wrap primitives in domain types (UserId not u64) for type safety
- Encode invariants in types (PositiveInteger guaranteed positive)
- Enums for "one of" types, structs for "all of" types
- Box large/recursive enum variants for memory efficiency

**Performance Characteristics**:
- Newtype: zero runtime cost, same representation as wrapped type
- Enum size: largest variant + discriminant (usually 1 byte)
- Boxing: reduces enum to pointer size, adds indirection

**Memory Layout**:
- Named struct: fields in declaration order (subject to alignment)
- Tuple struct: same as tuple with same types
- Unit struct: 0 bytes
- Enum: size_of(largest variant) + discriminant
- Box<T>: size_of pointer (8 bytes on 64-bit)

**Pattern Decision Matrix**:
- **Multiple types, all fields present**: Named struct
- **Simple wrapper, distinct type**: Tuple struct (newtype)
- **No data, marker only**: Unit struct
- **One of several types**: Enum
- **Recursive structure**: Enum with Box
- **Validated type**: Newtype with smart constructor
- **Domain-specific ID**: Newtype (struct UserId(u64))

**Anti-Patterns to Avoid**:
- Using u64 for IDs instead of newtypes (loses type safety)
- Multiple Option fields instead of enum (unclear which combinations valid)
- Large enum variants without Box (wastes memory)
- Missing exhaustive match (non-exhaustive pattern use `_`)
- Type aliases for distinct types (`type UserId = u64` doesn't prevent mixing)

> **See Also**: For compile-time state machines (typestate pattern) and phantom types, see **Chapter 4: Pattern 6** and **Chapter 5: Pattern 2**.
