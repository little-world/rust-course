# Struct & Enum Patterns
This chapter explores struct and enum patterns for type-safe data modeling: choosing struct types, newtype wrappers for domain types, zero-sized types for compile-time guarantees, enums for variants, and advanced techniques for memory efficiency and recursion.

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

> **See Also**: For compile-time state machines (typestate pattern) and phantom types, see **Chapter 4: Pattern 6** and **Chapter 5: Pattern 2**.


### Struct Cheat Sheet
```rust
// ===== BASIC STRUCTS =====
// Named field struct
struct User {
    username: String,
    email: String,
    age: u32,
    active: bool,
}

// Create instance
fn basic_struct_example() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    // Access fields
    println!("Username: {}", user.username);
    println!("Email: {}", user.email);
}

// Mutable struct
fn mutable_struct() {
    let mut user = User {
        username: String::from("bob"),
        email: String::from("bob@example.com"),
        age: 25,
        active: true,
    };
    
    // Modify fields
    user.email = String::from("bob_new@example.com");
    user.age += 1;
}

// ===== TUPLE STRUCTS =====
// Tuple struct (fields without names)
struct Color(u8, u8, u8);
struct Point(i32, i32, i32);

fn tuple_struct_example() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
    
    // Access by index
    println!("Red: {}", black.0);
    println!("X: {}", origin.0);
    
    // Pattern matching
    let Color(r, g, b) = black;
    println!("RGB: ({}, {}, {})", r, g, b);
}

// ===== UNIT-LIKE STRUCTS =====
// Struct with no fields
struct Marker;
struct AlwaysEqual;

fn unit_struct_example() {
    let marker = Marker;
    let equal1 = AlwaysEqual;
    let equal2 = AlwaysEqual;
}

// ===== FIELD INIT SHORTHAND =====
fn build_user(email: String, username: String) -> User {
    User {
        username,                                            // Shorthand when variable name matches field
        email,
        age: 0,
        active: true,
    }
}

// ===== STRUCT UPDATE SYNTAX =====
fn struct_update_example() {
    let user1 = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    // Create new struct using fields from another
    let user2 = User {
        email: String::from("alice2@example.com"),
        ..user1                                              // Copy remaining fields
    };
    
    // Note: user1.username and user1.email are moved to user2
    // Can still use user1.age and user1.active (Copy types)
}

// ===== STRUCT METHODS =====
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // Method (takes &self)
    fn area(&self) -> u32 {
        self.width * self.height
    }
    
    // Method with mutable self
    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }
    
    // Method that takes ownership
    fn into_square(self) -> Square {
        Square {
            side: self.width.min(self.height),
        }
    }
    
    // Method with parameters
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

struct Square {
    side: u32,
}

fn method_example() {
    let mut rect = Rectangle {
        width: 30,
        height: 50,
    };
    
    println!("Area: {}", rect.area());
    
    rect.scale(2);
    println!("New width: {}", rect.width);
    
    let square = rect.into_square();
    // rect is now invalid (moved)
}

// ===== ASSOCIATED FUNCTIONS =====
impl Rectangle {
    // Associated function (no self parameter)
    fn new(width: u32, height: u32) -> Rectangle {
        Rectangle { width, height }
    }
    
    // Another constructor
    fn square(size: u32) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }
}

fn associated_function_example() {
    let rect = Rectangle::new(30, 50);
    let square = Rectangle::square(10);
}

// ===== MULTIPLE IMPL BLOCKS =====
impl Rectangle {
    fn perimeter(&self) -> u32 {
        2 * (self.width + self.height)
    }
}

impl Rectangle {
    fn is_square(&self) -> bool {
        self.width == self.height
    }
}

// ===== GENERIC STRUCTS =====
struct Point2D<T> {
    x: T,
    y: T,
}

impl<T> Point2D<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// Implementation for specific type
impl Point2D<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

// Multiple generic parameters
struct Point3D<T, U> {
    x: T,
    y: T,
    z: U,
}

impl<T, U> Point3D<T, U> {
    fn mixup<V, W>(self, other: Point3D<V, W>) -> Point3D<T, W> {
        Point3D {
            x: self.x,
            y: self.y,
            z: other.z,
        }
    }
}

fn generic_struct_example() {
    let integer_point = Point2D { x: 5, y: 10 };
    let float_point = Point2D { x: 1.0, y: 4.0 };
    
    let p1 = Point3D { x: 5, y: 10, z: 1.5 };
    let p2 = Point3D { x: "Hello", y: "World", z: 'c' };
    
    let p3 = p1.mixup(p2);
    // p3 has type Point3D<i32, char>
}

// ===== STRUCT WITH LIFETIMES =====
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }
    
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention: {}", announcement);
        self.part
    }
}

fn lifetime_struct_example() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().expect("Could not find a '.'");
    
    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };
}

// ===== PRIVATE AND PUBLIC FIELDS =====
mod my_module {
    pub struct PublicStruct {
        pub public_field: String,
        private_field: i32,                              // Private by default
    }
    
    impl PublicStruct {
        pub fn new(public_field: String) -> PublicStruct {
            PublicStruct {
                public_field,
                private_field: 0,
            }
        }
        
        pub fn get_private(&self) -> i32 {
            self.private_field
        }
    }
}

fn visibility_example() {
    let s = my_module::PublicStruct::new(String::from("hello"));
    println!("{}", s.public_field);
    // println!("{}", s.private_field);                 // ERROR: private field
}

// ===== DESTRUCTURING =====
fn destructuring_example() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    // Destructure all fields
    let User { username, email, age, active } = user;
    
    // Destructure some fields
    let User { username, email, .. } = user;
    
    // Rename fields during destructuring
    let User { username: name, email: mail, .. } = user;
}

// ===== DERIVE MACROS =====
#[derive(Debug)]                                         // Auto-implement Debug
struct DebugStruct {
    field: i32,
}

#[derive(Clone)]                                         // Auto-implement Clone
struct CloneStruct {
    data: Vec<i32>,
}

#[derive(Copy, Clone)]                                   // Copy requires Clone
struct CopyStruct {
    value: i32,
}

#[derive(PartialEq, Eq)]                                // Equality comparison
struct EqStruct {
    id: i32,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]               // Ordering
struct OrdStruct {
    priority: i32,
}

#[derive(Default)]                                       // Default values
struct DefaultStruct {
    count: i32,
    name: String,
}

#[derive(Hash)]                                          // Hashing
struct HashStruct {
    id: i32,
}

// Multiple derives
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MultiDeriveStruct {
    id: i32,
    name: String,
}

fn derive_example() {
    let s = DebugStruct { field: 42 };
    println!("{:?}", s);                                 // Uses Debug
    
    let default: DefaultStruct = Default::default();
    println!("{}", default.count);                       // 0
}

// ===== PATTERN MATCHING WITH STRUCTS =====
fn pattern_matching() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    match user {
        User { active: true, age, .. } => {
            println!("Active user, age: {}", age);
        }
        User { active: false, .. } => {
            println!("Inactive user");
        }
    }
    
    // If let pattern
    if let User { username, .. } = user {
        println!("Username: {}", username);
    }
}

// ===== BUILDER PATTERN =====
#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
    timeout: u64,
    retries: u32,
}

struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u64>,
    retries: Option<u32>,
}

impl ConfigBuilder {
    fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            timeout: None,
            retries: None,
        }
    }
    
    fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }
    
    fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
            retries: self.retries.unwrap_or(3),
        })
    }
}

fn builder_example() {
    let config = ConfigBuilder::new()
        .host("localhost")
        .port(3000)
        .timeout(60)
        .build()
        .unwrap();
    
    println!("{:?}", config);
}

// ===== NEWTYPE PATTERN =====
struct Meters(f64);
struct Kilometers(f64);

impl Meters {
    fn to_kilometers(&self) -> Kilometers {
        Kilometers(self.0 / 1000.0)
    }
}

impl Kilometers {
    fn to_meters(&self) -> Meters {
        Meters(self.0 * 1000.0)
    }
}

fn newtype_example() {
    let distance = Meters(5000.0);
    let km = distance.to_kilometers();
    
    // Cannot accidentally mix Meters and Kilometers
    // let sum = Meters(100.0) + Kilometers(1.0);       // ERROR
}

// ===== INTERIOR MUTABILITY =====
use std::cell::{Cell, RefCell};

struct Counter {
    count: Cell<i32>,
}

impl Counter {
    fn new() -> Self {
        Counter {
            count: Cell::new(0),
        }
    }
    
    fn increment(&self) {                                // Takes &self, not &mut self
        self.count.set(self.count.get() + 1);
    }
    
    fn get(&self) -> i32 {
        self.count.get()
    }
}

struct Container {
    data: RefCell<Vec<i32>>,
}

impl Container {
    fn add(&self, value: i32) {                          // Takes &self
        self.data.borrow_mut().push(value);
    }
    
    fn get(&self, index: usize) -> Option<i32> {
        self.data.borrow().get(index).copied()
    }
}

// ===== ZERO-SIZED TYPES =====
struct ZeroSized;

fn zero_sized_example() {
    let zst = ZeroSized;
    println!("Size: {}", std::mem::size_of::<ZeroSized>());  // 0
    
    // Useful as markers or tokens
    let vec: Vec<ZeroSized> = vec![ZeroSized; 1000];
    // Takes no heap memory!
}

// ===== PHANTOM DATA =====
use std::marker::PhantomData;

struct PhantomStruct<T> {
    data: i32,
    _marker: PhantomData<T>,                             // Zero-sized
}

impl<T> PhantomStruct<T> {
    fn new(data: i32) -> Self {
        PhantomStruct {
            data,
            _marker: PhantomData,
        }
    }
}

// Different types even with same data
fn phantom_example() {
    let p1: PhantomStruct<i32> = PhantomStruct::new(42);
    let p2: PhantomStruct<String> = PhantomStruct::new(42);
    
    // p1 and p2 are different types
}

// ===== STRUCT WITH ARRAYS =====
struct Grid {
    cells: [[i32; 10]; 10],
}

impl Grid {
    fn new() -> Self {
        Grid {
            cells: [[0; 10]; 10],
        }
    }
    
    fn get(&self, x: usize, y: usize) -> i32 {
        self.cells[x][y]
    }
    
    fn set(&mut self, x: usize, y: usize, value: i32) {
        self.cells[x][y] = value;
    }
}

// ===== STRUCT WITH FUNCTIONS =====
struct Operation {
    name: String,
    func: fn(i32, i32) -> i32,
}

impl Operation {
    fn execute(&self, a: i32, b: i32) -> i32 {
        (self.func)(a, b)
    }
}

fn struct_with_function_example() {
    let add_op = Operation {
        name: String::from("add"),
        func: |a, b| a + b,
    };
    
    println!("Result: {}", add_op.execute(5, 3));
}

// ===== CONST GENERICS =====
struct FixedArray<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> FixedArray<T, N> {
    fn new() -> Self {
        FixedArray {
            data: [T::default(); N],
        }
    }
}

fn const_generic_example() {
    let arr: FixedArray<i32, 5> = FixedArray::new();
    let arr2: FixedArray<i32, 10> = FixedArray::new();
    
    // arr and arr2 are different types
}

// ===== SELF-REFERENTIAL STRUCTS =====
use std::pin::Pin;

// Simple self-referential (unsafe)
struct SelfRef {
    data: String,
    ptr: *const String,                                  // Points to data
}

impl SelfRef {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfRef {
            data: s,
            ptr: std::ptr::null(),
        });
        
        let ptr = &boxed.data as *const String;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).ptr = ptr;
        }
        
        boxed
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: State machine using structs
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        Door { _state: PhantomData }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }
    
    fn open(&self) {
        println!("Door opened");
    }
}

fn state_machine_example() {
    let door = Door::<Locked>::new();
    let door = door.unlock();
    door.open();
    let door = door.lock();
    // door.open();                                     // ERROR: door is locked
}

// Pattern 2: Wrapper for external types
struct Wrapper(Vec<i32>);

impl Wrapper {
    fn sum(&self) -> i32 {
        self.0.iter().sum()
    }
}

// Pattern 3: Type-safe IDs
struct UserId(u32);
struct PostId(u32);

fn get_user(id: UserId) -> String {
    format!("User {}", id.0)
}

fn id_example() {
    let user_id = UserId(1);
    let post_id = PostId(1);
    
    get_user(user_id);
    // get_user(post_id);                               // ERROR: type mismatch
}

// Pattern 4: Cached computation
struct Cached<T> {
    value: Option<T>,
    computation: fn() -> T,
}

impl<T> Cached<T> {
    fn new(computation: fn() -> T) -> Self {
        Cached {
            value: None,
            computation,
        }
    }
    
    fn get(&mut self) -> &T {
        if self.value.is_none() {
            self.value = Some((self.computation)());
        }
        self.value.as_ref().unwrap()
    }
}

// Pattern 5: Struct with validation
struct Email {
    address: String,
}

impl Email {
    fn new(address: String) -> Result<Self, String> {
        if address.contains('@') {
            Ok(Email { address })
        } else {
            Err("Invalid email format".to_string())
        }
    }
    
    fn as_str(&self) -> &str {
        &self.address
    }
}

// Pattern 6: Composed structs
struct Address {
    street: String,
    city: String,
    country: String,
}

struct Person {
    name: String,
    address: Address,
}

// Pattern 7: Optional fields with builder
#[derive(Default)]
struct Options {
    verbose: bool,
    debug: bool,
    output: Option<String>,
}

impl Options {
    fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
    
    fn debug(mut self) -> Self {
        self.debug = true;
        self
    }
    
    fn output(mut self, path: String) -> Self {
        self.output = Some(path);
        self
    }
}

fn options_example() {
    let opts = Options::default()
        .verbose()
        .debug()
        .output(String::from("output.txt"));
}

// Pattern 8: Tagged union (enum alternative)
struct Tagged<T> {
    tag: String,
    data: T,
}

impl<T> Tagged<T> {
    fn new(tag: &str, data: T) -> Self {
        Tagged {
            tag: tag.to_string(),
            data,
        }
    }
}
```



### Enum Cheat Sheet

```rust
// ===== BASIC ENUMS =====
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

fn basic_enum_example() {
    let dir = Direction::North;
    
    match dir {
        Direction::North => println!("Going north"),
        Direction::South => println!("Going south"),
        Direction::East => println!("Going east"),
        Direction::West => println!("Going west"),
    }
}

// ===== ENUMS WITH DATA =====
// Each variant can hold different types and amounts of data
enum Message {
    Quit,                                                // No data
    Move { x: i32, y: i32 },                            // Named fields
    Write(String),                                       // Single value
    ChangeColor(u8, u8, u8),                            // Multiple values
}

fn enum_with_data_example() {
    let msg1 = Message::Quit;
    let msg2 = Message::Move { x: 10, y: 20 };
    let msg3 = Message::Write(String::from("Hello"));
    let msg4 = Message::ChangeColor(255, 0, 0);
}

// ===== PATTERN MATCHING =====
fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quitting");
        }
        Message::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        Message::Write(text) => {
            println!("Writing: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println!("Changing color to RGB({}, {}, {})", r, g, b);
        }
    }
}

// Match with guards
fn match_with_guard(msg: Message) {
    match msg {
        Message::Move { x, y } if x > 0 && y > 0 => {
            println!("Moving to positive quadrant");
        }
        Message::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        _ => {}
    }
}

// ===== OPTION ENUM =====
// Option<T> is defined in standard library as:
// enum Option<T> {
//     Some(T),
//     None,
// }

fn option_example() {
    let some_number = Some(5);
    let some_string = Some("Hello");
    let absent_number: Option<i32> = None;
    
    // Pattern matching
    match some_number {
        Some(n) => println!("Number: {}", n),
        None => println!("No number"),
    }
    
    // if let
    if let Some(n) = some_number {
        println!("Got number: {}", n);
    }
    
    // Unwrap (panics if None)
    let n = some_number.unwrap();
    
    // Unwrap with default
    let n = absent_number.unwrap_or(0);
    
    // Unwrap with closure
    let n = absent_number.unwrap_or_else(|| 10);
    
    // Map
    let doubled = some_number.map(|n| n * 2);
    
    // And then (flatMap)
    let result = some_number.and_then(|n| Some(n * 2));
}

// ===== RESULT ENUM =====
// Result<T, E> is defined as:
// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

fn result_example() {
    let result = divide(10, 2);
    
    // Pattern matching
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(e) => println!("Error: {}", e),
    }
    
    // Unwrap (panics on Err)
    let value = divide(10, 2).unwrap();
    
    // Expect (custom panic message)
    let value = divide(10, 2).expect("Division failed");
    
    // Unwrap or default
    let value = divide(10, 0).unwrap_or(0);
    
    // Question mark operator
    fn propagate_error() -> Result<i32, String> {
        let result = divide(10, 2)?;                     // Returns Err early
        Ok(result * 2)
    }
    
    // Map
    let doubled = divide(10, 2).map(|n| n * 2);
    
    // Map error
    let result = divide(10, 0).map_err(|e| format!("Error: {}", e));
}

// ===== ENUM METHODS =====
impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit called"),
            Message::Move { x, y } => println!("Move to ({}, {})", x, y),
            Message::Write(text) => println!("Write: {}", text),
            Message::ChangeColor(r, g, b) => println!("Color: RGB({}, {}, {})", r, g, b),
        }
    }
    
    fn is_quit(&self) -> bool {
        matches!(self, Message::Quit)
    }
}

fn enum_methods_example() {
    let msg = Message::Write(String::from("Hello"));
    msg.call();
    println!("Is quit: {}", msg.is_quit());
}

// ===== GENERIC ENUMS =====
enum Result2<T, E> {
    Ok(T),
    Err(E),
}

enum Option2<T> {
    Some(T),
    None,
}

// Multiple generic parameters
enum Either<L, R> {
    Left(L),
    Right(R),
}

fn generic_enum_example() {
    let left: Either<i32, String> = Either::Left(42);
    let right: Either<i32, String> = Either::Right(String::from("Hello"));
    
    match left {
        Either::Left(n) => println!("Number: {}", n),
        Either::Right(s) => println!("String: {}", s),
    }
}

// ===== IF LET =====
fn if_let_example() {
    let some_value = Some(3);
    
    // Instead of match
    if let Some(n) = some_value {
        println!("Number: {}", n);
    }
    
    // With else
    if let Some(n) = some_value {
        println!("Number: {}", n);
    } else {
        println!("No number");
    }
    
    // Multiple if let
    let msg = Message::Move { x: 10, y: 20 };
    
    if let Message::Move { x, y } = msg {
        println!("Move to ({}, {})", x, y);
    } else if let Message::Write(text) = msg {
        println!("Write: {}", text);
    } else {
        println!("Other message");
    }
}

// ===== WHILE LET =====
fn while_let_example() {
    let mut stack = vec![1, 2, 3, 4, 5];
    
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
}

// ===== MATCHES! MACRO =====
fn matches_example() {
    let msg = Message::Write(String::from("Hello"));
    
    // Check if matches pattern
    if matches!(msg, Message::Write(_)) {
        println!("It's a Write message");
    }
    
    // With guard
    let num = Some(4);
    if matches!(num, Some(x) if x < 5) {
        println!("Number is less than 5");
    }
}

// ===== ENUM DISCRIMINANTS =====
#[repr(u8)]
enum Status {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}

fn discriminant_example() {
    let status = Status::Active;
    let value = status as u8;
    println!("Status value: {}", value);
}

// ===== C-LIKE ENUMS =====
#[repr(C)]
enum Color {
    Red = 0xFF0000,
    Green = 0x00FF00,
    Blue = 0x0000FF,
}

#[repr(i32)]
enum ErrorCode {
    Success = 0,
    NotFound = -1,
    PermissionDenied = -2,
    InternalError = -500,
}

// ===== DERIVE MACROS FOR ENUMS =====
#[derive(Debug)]
enum LogLevel {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Copy)]
enum Direction2 {
    North,
    South,
    East,
    West,
}

#[derive(PartialEq, Eq)]
enum State {
    Active,
    Inactive,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Priority {
    Low,
    Medium,
    High,
}

fn derive_example() {
    let level = LogLevel::Info;
    println!("{:?}", level);
    
    let state1 = State::Active;
    let state2 = State::Active;
    println!("Equal: {}", state1 == state2);
    
    let p1 = Priority::Low;
    let p2 = Priority::High;
    println!("p1 < p2: {}", p1 < p2);
}

// ===== ENUMS WITH LIFETIMES =====
enum Cow<'a> {
    Borrowed(&'a str),
    Owned(String),
}

fn lifetime_enum_example() {
    let borrowed = Cow::Borrowed("Hello");
    let owned = Cow::Owned(String::from("World"));
}

// ===== RECURSIVE ENUMS =====
// Requires Box for indirection
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> Self {
        List::Nil
    }
    
    fn prepend(self, elem: i32) -> Self {
        List::Cons(elem, Box::new(self))
    }
    
    fn len(&self) -> usize {
        match self {
            List::Cons(_, tail) => 1 + tail.len(),
            List::Nil => 0,
        }
    }
}

fn recursive_enum_example() {
    let list = List::new()
        .prepend(1)
        .prepend(2)
        .prepend(3);
    
    println!("Length: {}", list.len());
}

// Binary tree
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

// ===== ENUM AS STATE MACHINE =====
enum ConnectionState {
    Disconnected,
    Connecting { retry_count: u32 },
    Connected { session_id: String },
    Error { message: String },
}

impl ConnectionState {
    fn connect(self) -> Self {
        match self {
            ConnectionState::Disconnected => {
                ConnectionState::Connecting { retry_count: 0 }
            }
            _ => self,
        }
    }
    
    fn establish(self, session_id: String) -> Self {
        match self {
            ConnectionState::Connecting { .. } => {
                ConnectionState::Connected { session_id }
            }
            _ => self,
        }
    }
    
    fn disconnect(self) -> Self {
        ConnectionState::Disconnected
    }
}

// ===== ENUM WITH ASSOCIATED DATA PATTERNS =====
// Linked list
enum LinkedList<T> {
    Empty,
    Node(T, Box<LinkedList<T>>),
}

// JSON-like value
#[derive(Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::HashMap<String, JsonValue>),
}

fn json_example() {
    use std::collections::HashMap;
    
    let mut obj = HashMap::new();
    obj.insert(String::from("name"), JsonValue::String(String::from("Alice")));
    obj.insert(String::from("age"), JsonValue::Number(30.0));
    obj.insert(String::from("active"), JsonValue::Bool(true));
    
    let json = JsonValue::Object(obj);
    println!("{:?}", json);
}

// ===== NESTED ENUMS =====
enum OuterMessage {
    Inner(InnerMessage),
    Other,
}

enum InnerMessage {
    Data(i32),
    Text(String),
}

fn nested_enum_example() {
    let msg = OuterMessage::Inner(InnerMessage::Data(42));
    
    match msg {
        OuterMessage::Inner(InnerMessage::Data(n)) => {
            println!("Data: {}", n);
        }
        OuterMessage::Inner(InnerMessage::Text(s)) => {
            println!("Text: {}", s);
        }
        OuterMessage::Other => {
            println!("Other");
        }
    }
}

// ===== ENUM SIZE AND LAYOUT =====
fn enum_size() {
    println!("Option<i32> size: {}", std::mem::size_of::<Option<i32>>());
    println!("Result<i32, String> size: {}", std::mem::size_of::<Result<i32, String>>());
    
    // Enum size is size of largest variant + discriminant
    enum Large {
        Small(u8),
        Large([u8; 100]),
    }
    
    println!("Large enum size: {}", std::mem::size_of::<Large>());
}

// ===== ENUM DESTRUCTURING =====
fn destructure_enum() {
    let msg = Message::ChangeColor(255, 0, 0);
    
    // Destructure in match
    match msg {
        Message::ChangeColor(r, g, b) => {
            println!("R: {}, G: {}, B: {}", r, g, b);
        }
        _ => {}
    }
    
    // Destructure in let
    if let Message::ChangeColor(r, g, b) = msg {
        println!("R: {}, G: {}, B: {}", r, g, b);
    }
    
    // Destructure struct variant
    let msg = Message::Move { x: 10, y: 20 };
    if let Message::Move { x, y } = msg {
        println!("X: {}, Y: {}", x, y);
    }
}

// ===== ENUM CONVERSIONS =====
// From/Into implementations
impl From<bool> for Status {
    fn from(active: bool) -> Self {
        if active {
            Status::Active
        } else {
            Status::Inactive
        }
    }
}

fn conversion_example() {
    let status: Status = true.into();
}

// TryFrom for fallible conversions
use std::convert::TryFrom;

impl TryFrom<i32> for Status {
    type Error = String;
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Status::Active),
            2 => Ok(Status::Inactive),
            3 => Ok(Status::Pending),
            _ => Err(format!("Invalid status code: {}", value)),
        }
    }
}

// ===== NON-EXHAUSTIVE ENUMS =====
#[non_exhaustive]
pub enum ApiError {
    NotFound,
    Unauthorized,
    InternalError,
}

// Users must use _ pattern to handle future variants
fn handle_error(error: ApiError) {
    match error {
        ApiError::NotFound => println!("Not found"),
        ApiError::Unauthorized => println!("Unauthorized"),
        ApiError::InternalError => println!("Internal error"),
        _ => println!("Unknown error"),                 // Required for non_exhaustive
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: Option chaining
fn option_chaining() {
    let value = Some(5)
        .map(|x| x * 2)
        .and_then(|x| if x > 5 { Some(x) } else { None })
        .unwrap_or(0);
    
    println!("Value: {}", value);
}

// Pattern 2: Result chaining
fn result_chaining() -> Result<i32, String> {
    divide(10, 2)?
        .checked_mul(3)
        .ok_or_else(|| "Overflow".to_string())
}

// Pattern 3: Custom error enum
#[derive(Debug)]
enum AppError {
    IoError(std::io::Error),
    ParseError(std::num::ParseIntError),
    CustomError(String),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(error: std::num::ParseIntError) -> Self {
        AppError::ParseError(error)
    }
}

fn app_function() -> Result<i32, AppError> {
    let contents = std::fs::read_to_string("file.txt")?;
    let number = contents.trim().parse::<i32>()?;
    Ok(number)
}

// Pattern 4: Event system
#[derive(Debug)]
enum Event {
    KeyPress(char),
    MouseClick { x: i32, y: i32 },
    Resize { width: u32, height: u32 },
    Quit,
}

fn handle_event(event: Event) {
    match event {
        Event::KeyPress(c) => println!("Key pressed: {}", c),
        Event::MouseClick { x, y } => println!("Mouse clicked at ({}, {})", x, y),
        Event::Resize { width, height } => println!("Resized to {}x{}", width, height),
        Event::Quit => println!("Quitting"),
    }
}

// Pattern 5: Command pattern
enum Command {
    Create { name: String },
    Update { id: u32, name: String },
    Delete { id: u32 },
    List,
}

impl Command {
    fn execute(&self) {
        match self {
            Command::Create { name } => println!("Creating: {}", name),
            Command::Update { id, name } => println!("Updating {} to {}", id, name),
            Command::Delete { id } => println!("Deleting {}", id),
            Command::List => println!("Listing all"),
        }
    }
}

// Pattern 6: Parser result
enum ParseResult<T> {
    Success(T, usize),                                   // value, bytes consumed
    Incomplete(usize),                                   // bytes needed
    Error(String),
}

// Pattern 7: Cow-like enum
enum MaybeOwned<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> MaybeOwned<'a> {
    fn as_str(&self) -> &str {
        match self {
            MaybeOwned::Borrowed(s) => s,
            MaybeOwned::Owned(s) => s.as_str(),
        }
    }
}

// Pattern 8: Validation result
enum Validation<T, E> {
    Valid(T),
    Invalid(Vec<E>),
}

impl<T, E> Validation<T, E> {
    fn is_valid(&self) -> bool {
        matches!(self, Validation::Valid(_))
    }
    
    fn errors(&self) -> Option<&Vec<E>> {
        match self {
            Validation::Invalid(errors) => Some(errors),
            _ => None,
        }
    }
}

// Pattern 9: Notification system
#[derive(Debug)]
enum Notification {
    Email { to: String, subject: String, body: String },
    Sms { to: String, message: String },
    Push { device_id: String, title: String, body: String },
}

impl Notification {
    fn send(&self) {
        match self {
            Notification::Email { to, subject, .. } => {
                println!("Sending email to {} with subject: {}", to, subject);
            }
            Notification::Sms { to, message } => {
                println!("Sending SMS to {}: {}", to, message);
            }
            Notification::Push { device_id, title, .. } => {
                println!("Sending push to {}: {}", device_id, title);
            }
        }
    }
}

// Pattern 10: AST (Abstract Syntax Tree)
enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> Result<i32, String> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::Add(left, right) => Ok(left.eval()? + right.eval()?),
            Expr::Subtract(left, right) => Ok(left.eval()? - right.eval()?),
            Expr::Multiply(left, right) => Ok(left.eval()? * right.eval()?),
            Expr::Divide(left, right) => {
                let r = right.eval()?;
                if r == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left.eval()? / r)
                }
            }
        }
    }
}

fn ast_example() {
    // (2 + 3) * 4
    let expr = Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Number(2)),
            Box::new(Expr::Number(3)),
        )),
        Box::new(Expr::Number(4)),
    );
    
    println!("Result: {}", expr.eval().unwrap());
}
```

