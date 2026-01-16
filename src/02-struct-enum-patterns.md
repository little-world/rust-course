# Struct & Enum Patterns

This chapter explores struct and enum patterns for type-safe **data modeling**: choosing struct types, newtype wrappers for domain types, zero-sized types for compile-time guarantees, enums for variants, and advanced techniques for memory efficiency and recursion.

## Pattern 1: Struct Design Patterns

*   **Problem**: It's often unclear when to use a named-field struct, a tuple struct, or a unit struct. Named fields can be verbose for simple types (`Point { x: f64, y: f64 }`), while tuple structs can be ambiguous (`Point(1.0, 2.0)`).
*   **Solution**: Use named-field structs for complex data models where clarity is key. Use tuple structs for simple wrappers and the newtype pattern to create distinct types from primitives.


### Example: Named Field Structs

Named field structs provide self-documenting code where each field's purpose is explicit. Use them for complex data models like users, configurations, or domain entities. The `Self` keyword in constructors makes refactoring easier when the type name changes.

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

// Usage: Create users via the constructor, which sets active=true by default.
// Call deactivate() to change state. Clone is derived automatically.
let mut user = User::new(1, "alice".to_string(), "alice@example.com".to_string());
user.deactivate(); // Sets active to false
```

**Why this matters:** Named fields provide self-documenting code. When you see `user.email`, the intent is clear. They also allow field reordering without breaking code.

### Example: Tuple Structs

Tuple structs access fields by index (`.0`, `.1`) rather than by name. Use them when field names would be redundant, like `Point(x, y)` or `Rgb(r, g, b)`. They're also the foundation of the newtype pattern for creating type-safe wrappers.

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

impl Kilometers {
    fn to_miles(&self) -> Miles {
        Miles(self.0 * 0.621371)
    }
}

// Usage: Access tuple struct fields by index (.0, .1). The newtype pattern
// creates distinct types—Kilometers and Miles can't be accidentally mixed.
let point = Point3D(3.0, 4.0, 0.0);
let km = Kilometers(100.0);
let mi = km.to_miles(); // Type-safe conversion
```

**The pattern:** Use tuple structs when the structure itself conveys meaning more than field names would. They're particularly powerful for the newtype pattern.

### Example: Unit Structs

Unit structs have no fields and occupy zero bytes at runtime. They're used as marker types for type-level programming and state machines. Combined with `PhantomData`, they enable compile-time state enforcement without runtime cost.

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

    fn authenticate(
        self,
        password: &str,
    ) -> Result<Database<Authenticated>, String> {
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
        vec!["result1".to_string(), "result2".to_string()]
    }
}

// Usage: query() is only available on Database<Authenticated>. The typestate
// pattern enforces authentication at compile time with zero runtime cost.
let db = Database::<Unauthenticated>::new("postgres://localhost".to_string());
let auth_db = db.authenticate("secret").unwrap();
let results = auth_db.query("SELECT * FROM users"); // Now allowed
```


## Pattern 2: Newtype and Wrapper Patterns

*   **Problem**: Using raw primitive types like `u64` for different kinds of IDs (`UserId`, `OrderId`) can lead to bugs where they are accidentally mixed up. Primitives can't enforce invariants (e.g., a `String` that must be non-empty) and lack domain-specific meaning.
*   **Solution**: Wrap primitive types in a tuple struct (e.g., `struct UserId(u64)`). This creates a new, distinct type that cannot be mixed with other types, even if they wrap the same primitive.

### Example: Newtype

The newtype pattern wraps a primitive in a tuple struct to create a distinct type. This prevents mixing up semantically different values like `UserId` and `OrderId`. The wrapper has zero runtime cost—it compiles to the same code as the raw primitive.

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


// Usage: UserId and OrderId are distinct types even though both wrap u64.
// The compiler prevents mixing them up, with zero runtime overhead.
let user_id = UserId(42);
let order_id = OrderId(42);
// user_id == order_id; // Won't compile: different types
```

### Example: Transparent Wrappers with Deref

Implementing `Deref` lets your wrapper auto-coerce to the inner type. This means `&Validated<String>` can be used anywhere `&String` is expected. Use this pattern when the wrapper should behave transparently like its contents.

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

// Usage: Deref lets you call the wrapped type's methods directly. Here,
// validated.len() calls String::len, while validated.age() is the wrapper's own method.
let validated = Validated::new("hello".to_string());
assert_eq!(validated.len(), 5); // String::len via Deref
```

## Pattern 3: Struct Memory and Update Patterns

*   **Problem**: Understanding struct update syntax (`..other`) can lead to confusion about ownership and partial moves. Creating variations of a struct immutably can feel clumsy, and the interaction between `Copy` and non-`Copy` fields during updates is not always intuitive.
*   **Solution**: Use the struct update syntax `..other` to create a new struct instance from an old one. Be aware that this will *move* any non-`Copy` fields, making the original struct partially unusable.

### Example: Struct Update Syntax

The struct update syntax `..other` creates a new instance using values from an existing one. Fields with `Copy` are copied, while non-`Copy` fields are moved from the original. To preserve the original struct, clone it before the update: `..base.clone()`.

```rust
#[derive(Debug, Clone, PartialEq)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
}

// Usage: Use ..base.clone() to create a modified copy while preserving the
// original. Without clone, non-Copy fields are moved from base.
let base = Config { host: "localhost".to_string(), port: 8080, timeout_ms: 5000 };
let updated = Config { port: 9090, ..base.clone() }; // base still usable
```

### Example: Understanding Partial Moves

Moving a non-`Copy` field out of a struct makes it "partially moved". You can still access the remaining `Copy` fields, but the struct as a whole becomes unusable. This commonly happens with destructuring or explicit field moves like `let s = data.moveable;`.

```rust
struct Data {
    copyable: i32,      // Implements Copy
    moveable: String,   // Does not implement Copy
}

// Usage: Moving a non-Copy field makes the struct partially moved. Copy fields
// remain accessible, but the whole struct can't be used. Clone or borrow instead.
let data = Data { copyable: 42, moveable: "hello".to_string() };
let s = data.moveable;      // Moves the String out
assert_eq!(data.copyable, 42); // Copy field still accessible
```

## Pattern 4: Enum Design Patterns

*   **Problem**: Representing a value that can be one of several related kinds is difficult with structs alone. Using `Option` for optional fields can create invalid states (e.g., a "shipped" order with no shipping address).
*   **Solution**: Use an `enum` to define a type that can be one of several variants. Each variant can have its own associated data.

### Example: Basic Enum with Pattern Matching

Enums let each variant carry different associated data. Pattern matching with `match` extracts this data and ensures all variants are handled. The compiler errors if you forget a case, making refactoring safe.

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
        matches!(
            self,
            HttpResponse::Ok { .. }
                | HttpResponse::Created { .. }
                | HttpResponse::NoContent
        )
    }
}

// Usage
fn handle_request(path: &str) -> HttpResponse {
    match path {
        "/users" => HttpResponse::Ok {
            body: "[{\"id\": 1}]".to_string(),
            headers: vec![(
                "Content-Type".to_string(),
                "application/json".to_string(),
            )],
        },
        "/users/create" => HttpResponse::Created {
            id: 123,
            location: "/users/123".to_string(),
        },
        _ => HttpResponse::NotFound,
    }
}

// Usage: Each variant carries its own data. Use status_code() and is_success()
// to handle responses uniformly regardless of variant.
let ok = HttpResponse::Ok { body: "Hello".to_string(), headers: vec![] };
assert_eq!(ok.status_code(), 200);
assert!(ok.is_success());
```

### Example: Enum State Machines

Enums naturally model state machines where each state has different associated data. Transition methods consume `self` and return a new state, preventing invalid state access. Exhaustive matching ensures all transitions from every state are explicitly handled.

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
        matches!(
            self,
            OrderStatus::Pending { .. } | OrderStatus::Processing { .. }
        )
    }
}

// Usage: process() consumes self and returns a new state. can_cancel() uses
// matches! to check multiple variants. Invalid transitions return Err.
let order = OrderStatus::Pending { items: vec!["Book".to_string()], customer_id: 42 };
let processing = order.process().unwrap(); // Transitions to Processing
assert!(processing.can_cancel()); // Processing orders can still be cancelled
```

## Pattern 5: Advanced Enum Techniques

*   **Problem**: Enums can have issues with memory usage if one variant is much larger than the others. Recursive enums (like a tree where a node contains other nodes) are impossible to define directly.
*   **Solution**: Use `Box<T>` to heap-allocate the data for large or recursive variants. This makes the size of the variant a pointer size, not the size of the data itself.

### Example: Recursive Enums with Box

Recursive types like trees need `Box` to break the infinite size calculation. Without `Box`, the compiler can't determine the enum's size since it contains itself. The `Box` provides indirection—the enum stores a fixed-size pointer to heap-allocated children.

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

// Usage: Box breaks the infinite size calculation for recursive types.
// Build trees or ASTs by boxing child nodes.
let expr = Expr::Mul(
    Box::new(Expr::Add(Box::new(Expr::Number(2)), Box::new(Expr::Number(3)))),
    Box::new(Expr::Number(4)),
);
assert_eq!(expr.eval(), 20); // (2 + 3) * 4 = 20
```

### Example: Memory-Efficient Large Variants

An enum's size equals its largest variant plus a discriminant. Boxing large variants keeps the enum small—only 8 bytes for the pointer. This improves cache performance when most instances use smaller variants.

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

// Usage: Boxing the large variant keeps the enum pointer-sized (≤16 bytes)
// instead of 1KB+. The heap allocation only happens for the Large variant.
use std::mem::size_of;
assert!(size_of::<Inefficient>() >= 1024); // Huge
assert!(size_of::<Efficient>() <= 16);     // Compact
```

## Pattern 6: Visitor Pattern with Enums

*   **Problem**: You have a complex, tree-like data structure, such as an Abstract Syntax Tree (AST). You want to perform various operations on this structure (e.g., pretty-printing, evaluation, type-checking) without cluttering the data structure's definition with all of this logic.
*   **Solution**: Define a `Visitor` trait with a `visit` method for each variant of your enum-based data structure. Each operation is then implemented as a separate struct that implements the `Visitor` trait.

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
    fn visit_binary_op(
        &mut self,
        op: &BinOp,
        left: &Expr,
        right: &Expr,
    ) -> Self::Output;
    fn visit_unary_op(
        &mut self,
        op: &UnOp,
        expr: &Expr,
    ) -> Self::Output;
}
```

### 3. Visitor Implementations
Finally, implement the visitors. Each visitor is a separate struct that implements the `ExprVisitor` trait, providing concrete logic for each `visit_*` method. This separates the concerns of pretty-printing and evaluation from the data structure itself.

```rust
// Pretty printer visitor
struct PrettyPrinter;

impl ExprVisitor for PrettyPrinter {
    type Output = String;

    fn visit_number(&mut self, n: f64) -> String {
        n.to_string()
    }
    fn visit_variable(&mut self, name: &str) -> String {
        name.to_string()
    }

    fn visit_binary_op(
        &mut self,
        op: &BinOp,
        left: &Expr,
        right: &Expr,
    ) -> String {
        let op_str = match op {
            BinOp::Add => "+",
            BinOp::Subtract => "-",
            BinOp::Multiply => "*",
            BinOp::Divide => "/",
        };
        let l = self.visit(left);
        let r = self.visit(right);
        format!("({} {} {})", l, op_str, r)
    }

    fn visit_unary_op(
        &mut self,
        op: &UnOp,
        expr: &Expr,
    ) -> String {
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
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| format!("Undefined: {}", name))
    }

    fn visit_binary_op(
        &mut self,
        op: &BinOp,
        left: &Expr,
        right: &Expr,
    ) -> Self::Output {
        let l = self.visit(left)?;
        let r = self.visit(right)?;
        match op {
            BinOp::Add => Ok(l + r),
            BinOp::Subtract => Ok(l - r),
            BinOp::Multiply => Ok(l * r),
            BinOp::Divide => Ok(l / r),
        }
    }

    fn visit_unary_op(
        &mut self,
        op: &UnOp,
        expr: &Expr,
    ) -> Self::Output {
        let val = self.visit(expr)?;
        match op {
            UnOp::Negate => Ok(-val),
            UnOp::Abs => Ok(val.abs()),
        }
    }
}

// Usage: Different visitors implement different operations on the same AST.
// PrettyPrinter outputs a string, Evaluator computes a numeric result.
let expr = Expr::BinaryOp {
    op: BinOp::Add,
    left: Box::new(Expr::Number(2.0)),
    right: Box::new(Expr::Number(3.0)),
};
let mut printer = PrettyPrinter;
assert_eq!(printer.visit(&expr), "(2 + 3)");
```

### Summary

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



