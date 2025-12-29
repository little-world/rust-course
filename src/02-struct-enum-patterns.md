# Struct & Enum Patterns

Rust doesn’t just give you `struct` and `enum` as containers for data. 
This chapter explores struct and enum patterns for type-safe **data modeling**: choosing struct types, newtype wrappers for domain types, zero-sized types for compile-time guarantees, enums for variants, and advanced techniques for memory efficiency and recursion.

## Pattern 1: Struct Design Patterns

*   **Problem**: It's often unclear when to use a named-field struct, a tuple struct, or a unit struct. Named fields can be verbose for simple types (`Point { x: f64, y: f64 }`), while tuple structs can be ambiguous (`Point(1.0, 2.0)`).
*   **Solution**: Use named-field structs for complex data models where clarity is key. Use tuple structs for simple wrappers and the newtype pattern to create distinct types from primitives.
*   **Why It Matters**: This choice enhances type safety and code clarity. Named fields are self-documenting.

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

*   **Problem**: Using raw primitive types like `u64` for different kinds of IDs (`UserId`, `OrderId`) can lead to bugs where they are accidentally mixed up. Primitives can't enforce invariants (e.g., a `String` that must be non-empty) and lack domain-specific meaning.
*   **Solution**: Wrap primitive types in a tuple struct (e.g., `struct UserId(u64)`). This creates a new, distinct type that cannot be mixed with other types, even if they wrap the same primitive.
*   **Why It Matters**: This pattern provides compile-time type safety at zero runtime cost. It prevents logical errors like passing an `OrderId` to a function expecting a `UserId`.

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

*   **Problem**: Understanding struct update syntax (`..other`) can lead to confusion about ownership and partial moves. Creating variations of a struct immutably can feel clumsy, and the interaction between `Copy` and non-`Copy` fields during updates is not always intuitive.
*   **Solution**: Use the struct update syntax `..other` to create a new struct instance from an old one. Be aware that this will *move* any non-`Copy` fields, making the original struct partially unusable.
*   **Why It Matters**: This syntax enables ergonomic, immutable updates. A clear understanding of the move semantics involved prevents surprising compile-time ownership errors.

> **Note**: For compile-time state checking with phantom types and typestate patterns, see **Chapter 4: Pattern 6 (Phantom Types)** and **Chapter 5: Pattern 2 (Typestate Pattern)**.

### Example: Struct Update Syntax

The struct update syntax `..` is a convenient way to create a new instance of a struct using the values from another instance. Fields that implement the `Copy` trait are copied, while non-`Copy` fields are moved. Because a move occurs, the original instance can no longer be used. To preserve the original, you must `clone()` it.

```rust
#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
}

// Usage with move (original is consumed)
let config1 = Config {
    host: "localhost".to_string(),
    port: 8080,
    timeout_ms: 5000,
};

let config2 = Config {
    port: 9090,
    ..config1 // `config1.host` is moved, `timeout_ms` is copied.
};
// println!("{:?}", config1); // ERROR: `host` field was moved.

// Usage with clone (original is preserved)
let config3 = Config {
    host: "localhost".to_string(),
    port: 8080,
    timeout_ms: 5000,
};
let config4 = Config {
    port: 9090,
    ..config3.clone() // Clones the `host` string.
};
println!("Original: {:?}", config3); // OK
println!("New: {:?}", config4);
```

### Example: Understanding Partial Moves
You can move specific fields out of a struct. If a field does not implement `Copy` (like `String`), moving it means the original struct can no longer be fully accessed, as it is now "partially moved". You can still access the remaining `Copy` fields, but you cannot move the struct as a whole.

```rust
struct Data {
    copyable: i32,      // Implements Copy
    moveable: String,   // Does not implement Copy
}

let data = Data {
    copyable: 42,
    moveable: "hello".to_string(),
};

// Move the non-Copy field out of the struct.
let s = data.moveable;
println!("Moved string: {}", s);

// You can still access the Copy field.
println!("Copyable field: {}", data.copyable);

// But you cannot use the whole struct anymore, as it's partially moved.
// let moved_data = data; // ERROR: use of partially moved value: `data`
```

**The pattern:** When building fluent APIs or config builders, be mindful of moves. Consider consuming `self` and returning `Self`, or use `&mut self` for in-place updates. For full builder pattern coverage, see **Chapter 5: Builder & API Design**.

## Pattern 4: Enum Design Patterns

*   **Problem**: Representing a value that can be one of several related kinds is difficult with structs alone. Using `Option` for optional fields can create invalid states (e.g., a "shipped" order with no shipping address).
*   **Solution**: Use an `enum` to define a type that can be one of several variants. Each variant can have its own associated data.
*   **Why It Matters**: Enums make impossible states unrepresentable. The compiler's exhaustive checking for `match` statements prevents bugs from forgotten cases.

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

*   **Problem**: Enums can have issues with memory usage if one variant is much larger than the others. Recursive enums (like a tree where a node contains other nodes) are impossible to define directly.
*   **Solution**: Use `Box<T>` to heap-allocate the data for large or recursive variants. This makes the size of the variant a pointer size, not the size of the data itself.
*   **Why It Matters**: Boxing variants is crucial for two reasons: it makes recursive enum definitions possible, and it makes enums with large variants memory-efficient, improving cache performance. Implementing methods and conversion traits on enums leads to cleaner, more idiomatic, and more reusable code.


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

*   **Problem**: You have a complex, tree-like data structure, such as an Abstract Syntax Tree (AST). You want to perform various operations on this structure (e.g., pretty-printing, evaluation, type-checking) without cluttering the data structure's definition with all of this logic.
*   **Solution**: Define a `Visitor` trait with a `visit` method for each variant of your enum-based data structure. Each operation is then implemented as a separate struct that implements the `Visitor` trait.
*   **Why It Matters**: This pattern decouples the logic of an operation from the data structure it operates on. This makes it easy to add new operations (just add a new visitor struct) without modifying the (potentially complex) data structure code.

The visitor pattern in Rust leverages enums for traversing complex structures. It involves three parts: the data structure, the visitor trait, and one or more visitor implementations.

### 1. The Data Structure (AST)
First, define the enum that represents the tree-like structure. For a simple expression language, this is the Abstract Syntax Tree (AST). Note the use of `Box<Expr>` to handle recursion.

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
```

### 2. The Visitor Trait
Next, define the `ExprVisitor` trait. It has a `visit` method for each variant of the `Expr` enum. The `visit` method on the trait itself handles dispatching to the correct specific method.

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
            Expr.Variable(name) => self.visit_variable(name),
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
```

### 3. Visitor Implementations
Finally, implement the visitors. Each visitor is a separate struct that implements the `ExprVisitor` trait, providing concrete logic for each `visit_*` method. This separates the concerns of pretty-printing and evaluation from the data structure itself.

```rust
# // AST for a simple expression language
# enum Expr {
#     Number(f64),
#     Variable(String),
#     BinaryOp {
#         op: BinOp,
#         left: Box<Expr>,
#         right: Box<Expr>,
#     },
#     UnaryOp {
#         op: UnOp,
#         expr: Box<Expr>,
#     },
# }
# 
# enum BinOp {
#     Add,
#     Subtract,
#     Multiply,
#     Divide,
# }
# 
# enum UnOp {
#     Negate,
#     Abs,
# }
# 
# // Visitor trait
# trait ExprVisitor {
#     type Output;
# 
#     fn visit(&mut self, expr: &Expr) -> Self::Output {
#         match expr {
#             Expr::Number(n) => self.visit_number(*n),
#             Expr::Variable(name) => self.visit_variable(name),
#             Expr::BinaryOp { op, left, right } => {
#                 self.visit_binary_op(op, left, right)
#             }
#             Expr::UnaryOp { op, expr } => {
#                 self.visit_unary_op(op, expr)
#             }
#         }
#     }
# 
#     fn visit_number(&mut self, n: f64) -> Self::Output;
#     fn visit_variable(&mut self, name: &str) -> Self::Output;
#     fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Self::Output;
#     fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> Self::Output;
# }

// Pretty printer visitor
struct PrettyPrinter;

impl ExprVisitor for PrettyPrinter {
    type Output = String;

    fn visit_number(&mut self, n: f64) -> String { n.to_string() }
    fn visit_variable(&mut self, name: &str) -> String { name.to_string() }

    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> String {
        let op_str = match op {
            BinOp::Add => "+", BinOp::Subtract => "-",
            BinOp::Multiply => "*", BinOp::Divide => "/",
        };
        format!("({} {} {})", self.visit(left), op_str, self.visit(right))
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> String {
        let op_str = match op { UnOp::Negate => "-", UnOp::Abs => "abs" };
        format!("{}({})", op_str, self.visit(expr))
    }
}

// Evaluator visitor
struct Evaluator {
    variables: std::collections::HashMap<String, f64>,
}

impl ExprVisitor for Evaluator {
    type Output = Result<f64, String>;

    fn visit_number(&mut self, n: f64) -> Self::Output { Ok(n) }

    fn visit_variable(&mut self, name: &str) -> Self::Output {
        self.variables.get(name).copied().ok_or_else(|| format!("Undefined variable: {}", name))
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Self::Output {
        let left_val = self.visit(left)?;
        let right_val = self.visit(right)?;
        match op {
            BinOp::Add => Ok(left_val + right_val),
            BinOp::Subtract => Ok(left_val - right_val),
            BinOp::Multiply => Ok(left_val * right_val),
            BinOp::Divide => Ok(left_val / right_val),
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



