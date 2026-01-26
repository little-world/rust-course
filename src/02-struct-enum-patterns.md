# Struct & Enum Patterns

This chapter explores struct and enum patterns for type-safe **data modeling**: choosing struct types, newtype wrappers for domain types, zero-sized types for compile-time guarantees, enums for variants, and advanced techniques for memory efficiency and recursion.

## Pattern 1: Struct Design Patterns

*   **Problem**: It's often unclear when to use a named-field struct, a tuple struct, or a unit struct. Named fields can be verbose for simple types (`Point { x: f64, y: f64 }`), while tuple structs can be ambiguous (`Point(1.0, 2.0)`).
*   **Solution**: Use named-field structs for complex data models where clarity is key. Use tuple structs for simple wrappers and the newtype pattern to create distinct types from primitives.


### Example: Named Field Structs

Named field `struct`s provide self-documenting code where each field's purpose is explicit. Use them for complex data models like users, configurations, or domain entities. The `Self` keyword in constructors makes refactoring easier when the type name changes.

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

// Direct construction - field order doesn't matter
let admin = User {
    email: "admin@example.com".to_string(),
    username: "admin".to_string(),
    id: 0,
    active: true,
};

// Constructor hides defaults, enforces invariants
let mut user = User::new(
    1,
    "alice".to_string(),
    "alice@example.com".to_string()
);
user.deactivate();
assert_eq!(admin.email, "admin@example.com");
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

// Usage: Access tuple struct fields by index.
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
    fn query(&self, _sql: &str) -> Vec<String> {
        vec!["result1".into(), "result2".into()]
    }
}

// Type state ensures query() only callable after auth
let db = Database::<Unauthenticated>::new(
    "postgres://localhost".to_string()
);
let auth_db = db.authenticate("secret").unwrap();
let results = auth_db.query("SELECT * FROM users");
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
// Distinct types prevent mixing IDs
let user_id = UserId(42);
let order_id = OrderId(42);
// get_user(order_id); // Won't compile: expected UserId
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

// Deref lets wrapper use inner type's methods
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

let base = Config {
    host: "localhost".into(),
    port: 8080,
    timeout_ms: 5000
};
let updated = Config { port: 9090, ..base.clone() };

// updated has new port, other fields from base
assert_eq!(updated.port, 9090);
assert_eq!(updated.host, "localhost"); // Cloned
assert_eq!(base.port, 8080);           // Unchanged
```

### Example: Understanding Partial Moves

Moving a non-`Copy` field out of a struct makes it "partially moved". You can still access the remaining `Copy` fields, but the struct as a whole becomes unusable. This commonly happens with destructuring or explicit field moves like `let s = data.moveable;`.

```rust
struct Data {
    copyable: i32,      // Implements Copy
    moveable: String,   // Does not implement Copy
}

// After moving non-Copy field, only Copy fields accessible
let data = Data {
    copyable: 42,
    moveable: "hello".to_string()
};
let s = data.moveable;         // Moves the String out
assert_eq!(data.copyable, 42); // Copy field accessible
// println!("{:?}", data);     // Error: partially moved
// let d = data;               // Error: cannot use `data`
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
    ServerError { msg: String, details: Option<String> },
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
            body: "[{\"id\": 1}]".into(),
            headers: vec![(
                "Content-Type".into(),
                "application/json".into(),
            )],
        },
        "/users/create" => HttpResponse::Created {
            id: 123,
            location: "/users/123".into(),
        },
        _ => HttpResponse::NotFound,
    }
}

// Each variant carries its own data; match extracts it
let ok = HttpResponse::Ok {
    body: "Hello".into(),
    headers: vec![]
};
assert_eq!(ok.status_code(), 200);
assert!(ok.is_success());
```

### Example: Enum State Machines

Enums naturally model state machines where each state has different associated data. Transition methods consume `self` and return a new state, preventing invalid state access. Exhaustive matching ensures all transitions from every state are explicitly handled.

```rust
enum OrderStatus {
    Pending { items: Vec<String>, customer: u64 },
    Processing { id: u64, started: std::time::Instant },
    Shipped { id: u64, tracking: String },
    Delivered { id: u64, signature: Option<String> },
    Cancelled { id: u64, reason: String },
}

impl OrderStatus {
    fn process(self) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Pending { items, .. } => {
                if items.is_empty() {
                    return Err("Empty order".into());
                }
                Ok(OrderStatus::Processing {
                    id: 12345,
                    started: std::time::Instant::now(),
                })
            }
            _ => Err("Not in pending state".into()),
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending { .. }
                | OrderStatus::Processing { .. }
        )
    }
}

// State transitions consume self and return new state
let order = OrderStatus::Pending {
    items: vec!["Book".into()],
    customer: 42
};
let processing = order.process().unwrap();
assert!(processing.can_cancel());
```

## Pattern 5: Advanced Enum Techniques

*   **Problem**: Enums can have issues with memory usage if one variant is much larger than the others. Recursive enums (like a tree where a node contains other nodes) are impossible to define directly.
*   **Solution**: Use `Box<T>` to heap-allocate the data for large or recursive variants. This makes the size of the variant a pointer size, not the size of the data itself.

### Example: Recursive Enums with Box

Recursive types like trees need `Box` to break the infinite size calculation. Without `Box`, the compiler can't determine the enum's size since it contains itself. The `Box` provides indirection—the enum stores a fixed-size pointer to heap-allocated children.

```rust
// Binary tree - Box breaks infinite size
enum Tree<T> {
    Leaf(T),
    Node { value: T, left: Box<Tree<T>>, right: Box<Tree<T>> },
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

let expr = Expr::Mul(
    Box::new(Expr::Add(
        Box::new(Expr::Number(2)),
        Box::new(Expr::Number(3)))),
    Box::new(Expr::Number(4)),
);
assert_eq!(expr.eval(), 20); // (2 + 3) * 4 = 20
```

### Example: Memory-Efficient Large Variants

An enum's size equals its largest variant plus a discriminant. Boxing large variants keeps the enum small—only 8 bytes for the pointer. This improves cache performance when most instances use smaller variants.

```rust
// Without Box: enum = largest variant size
enum Inefficient {
    Small(u8),
    Large([u8; 1024]),  // 1KB - every variant uses this
}

// With Box: enum = pointer size (8 bytes)
enum Efficient {
    Small(u8),
    Large(Box<[u8; 1024]>),  // Allocates only when used
}

use std::mem::size_of;
assert!(size_of::<Inefficient>() >= 1024); // Huge
assert!(size_of::<Efficient>() <= 16);     // Compact
```

## Pattern 6: Visitor Pattern with Enums

*   **Problem**: You have a complex, tree-like data structure, such as an Abstract Syntax Tree (AST). You want to perform various operations on this structure (e.g., pretty-printing, evaluation, type-checking) without cluttering the data structure's definition with all of this logic.
*   **Solution**: Define a `Visitor` trait with a `visit` method for each variant of your enum-based data structure. Each operation is then implemented as a separate struct that implements the `Visitor` trait.

### 1. The Data Structure (AST)
First, define the enum that represents the tree-like structure. For a simple expression language, this is the Abstract Syntax Tree (AST). Note the use of `Box<AstExpr>` to handle recursion.

**Note:** The following code blocks form a complete example. They should be combined in the same module to compile.

```rust
// AST for a simple expression language
enum AstExpr {
    Number(f64),
    Variable(String),
    BinaryOp {
        op: BinOp,
        left: Box<AstExpr>,
        right: Box<AstExpr>,
    },
    UnaryOp {
        op: UnOp,
        expr: Box<AstExpr>,
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

// Build an AST representing "2 + x"
let expr = AstExpr::BinaryOp {
    op: BinOp::Add,
    left: Box::new(AstExpr::Number(2.0)),
    right: Box::new(AstExpr::Variable("x".into())),
};
```

### 2. The Visitor Trait
Next, define the `AstVisitor` trait. It has a `visit` method for each variant of the `AstExpr` enum. The `visit` method on the trait itself handles dispatching to the correct specific method.
```rust
// Visitor trait
trait AstVisitor {
    type Output;

    fn visit(&mut self, expr: &AstExpr) -> Self::Output {
        match expr {
            AstExpr::Number(n) => self.visit_number(*n),
            AstExpr::Variable(name) => self.visit_variable(name),
            AstExpr::BinaryOp { op, left, right } => {
                self.visit_binary_op(op, left, right)
            }
            AstExpr::UnaryOp { op, expr } => {
                self.visit_unary_op(op, expr)
            }
        }
    }

    fn visit_number(&mut self, n: f64) -> Self::Output;
    fn visit_variable(&mut self, name: &str) -> Self::Output;
    fn visit_binary_op(
        &mut self,
        op: &BinOp,
        left: &AstExpr,
        right: &AstExpr,
    ) -> Self::Output;
    fn visit_unary_op(
        &mut self,
        op: &UnOp,
        expr: &AstExpr,
    ) -> Self::Output;
}

// Call visitor.visit(&expr) to traverse the AST.
// Trait dispatches to appropriate visit_* method.
```

### 3. Visitor Implementations
Finally, implement the visitors. Each visitor is a separate struct that implements the `AstVisitor` trait, providing concrete logic for each `visit_*` method. This separates the concerns of pretty-printing and evaluation from the data structure itself.

```rust
use std::collections::HashMap;

// Pretty printer visitor
struct PrettyPrinter;

impl AstVisitor for PrettyPrinter {
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
        left: &AstExpr,
        right: &AstExpr,
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
        expr: &AstExpr,
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
    variables: HashMap<String, f64>,
}

impl AstVisitor for Evaluator {
    type Output = Result<f64, String>;

    fn visit_number(&mut self, n: f64) -> Self::Output {
        Ok(n)
    }

    fn visit_variable(&mut self, name: &str) -> Self::Output {
        self.variables
            .get(name)
            .copied()
            .ok_or_else(|| format!("Undefined: {name}"))
    }

    fn visit_binary_op(
        &mut self,
        op: &BinOp,
        left: &AstExpr,
        right: &AstExpr,
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
        expr: &AstExpr,
    ) -> Self::Output {
        let val = self.visit(expr)?;
        match op {
            UnOp::Negate => Ok(-val),
            UnOp::Abs => Ok(val.abs()),
        }
    }
}

// Usage: PrettyPrinter formats AST as string
let expr = AstExpr::BinaryOp {
    op: BinOp::Add,
    left: Box::new(AstExpr::Number(2.0)),
    right: Box::new(AstExpr::Number(3.0)),
};
let mut printer = PrettyPrinter;
assert_eq!(printer.visit(&expr), "(2 + 3)");

// Evaluator computes result with variable bindings
let expr_var = AstExpr::BinaryOp {
    op: BinOp::Multiply,
    left: Box::new(AstExpr::Variable("x".into())),
    right: Box::new(AstExpr::Number(10.0)),
};
let mut eval = Evaluator {
    variables: HashMap::from([("x".into(), 5.0)]),
};
assert_eq!(eval.visit(&expr_var), Ok(50.0)); // x*10=50
```

### Summary

**Design Principles**:
- Named fields for clarity, tuple when type is meaningful
- Wrap primitives in domain types (UserId not u64)
- Encode invariants in types (PositiveInteger)
- Enums for "one of", structs for "all of"
- Box large/recursive enum variants

**Performance Characteristics**:
- Newtype: zero cost, same representation as wrapped type
- Enum size: largest variant + discriminant (1-8 bytes)
- Boxing: reduces enum to pointer size, adds indirection

**Memory Layout**:
- Named struct: fields in order (subject to alignment)
- Tuple struct: same as tuple with same types
- Unit struct: 0 bytes
- Enum: size_of(largest variant) + discriminant
- Box<T>: pointer size (8 bytes on 64-bit)

**Pattern Decision Matrix**:
- **Multiple types, all fields present**: Named struct
- **Simple wrapper, distinct type**: Tuple struct (newtype)
- **No data, marker only**: Unit struct
- **One of several types**: Enum
- **Recursive structure**: Enum with Box
- **Validated type**: Newtype with smart constructor
- **Domain-specific ID**: Newtype (struct UserId(u64))

**Anti-Patterns to Avoid**:
- Using u64 for IDs instead of newtypes
- Multiple Option fields instead of enum
- Large enum variants without Box
- Missing exhaustive match (use `_`)
- Type aliases (`type UserId = u64` doesn't prevent mixing)



