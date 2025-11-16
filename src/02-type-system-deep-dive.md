# Chapter 2: Type System Deep Dive

## Introduction

Rust's type system is one of the most sophisticated in any mainstream programming language. It combines the expressiveness of ML-family languages with zero-cost abstractions, enabling you to encode invariants at the type level that would otherwise require runtime checks or extensive documentation.

This chapter explores advanced type system patterns that experienced programmers can leverage to write safer, more maintainable code. The key insight is that Rust's type system allows you to move validation from runtime to compile-time, catching entire classes of bugs before your code ever runs.

The patterns we'll explore include:
- Struct design patterns for modeling domain concepts
- Using newtypes to prevent mixing incompatible values
- Enum-driven architecture for precise state modeling
- Encoding state machines in the type system
- Advanced pattern matching techniques
- Visitor patterns with enums
- Optimizing trait objects for dynamic dispatch
- Associated types vs generic parameters

**Note**: For advanced type-level programming including Generic Associated Types (GATs), const generics, and phantom type theory, see Chapter 33: Type-Level Programming.

## Type System Foundation

```rust
// Core type system concepts
struct Point<T> { x: T, y: T }           // Generic structs
enum Option<T> { Some(T), None }         // Generic enums
trait Display { fn fmt(&self); }         // Traits (interfaces)
impl Display for Point<i32> { }          // Trait implementation

// Advanced features covered in this chapter
type Meters = f64;                       // Type alias (no safety)
struct Meters(f64);                      // Newtype (type safety)
struct State<S> { _marker: PhantomData<S> }  // Phantom types for state machines
trait Container { type Item; }           // Associated types

// Polymorphism approaches
Box<dyn Display>                         // Trait object (dynamic dispatch)
&dyn Display                             // Borrowed trait object
fn generic<T: Display>(x: T) { }         // Generic function (static dispatch)
```

## Struct Design Patterns

Rust offers three struct varieties, each optimized for different use cases. Understanding when to use each form is crucial for writing idiomatic code.

### Named Field Structs

Named field structs are the most common form, offering clarity and flexibility:

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

//======
// Usage
//======
let user = User::new(1, "alice".to_string(), "alice@example.com".to_string());
println!("User {} is active: {}", user.username, user.active);
```

**Why this matters:** Named fields provide self-documenting code. When you see `user.email`, the intent is clear. They also allow field reordering without breaking code.

### Tuple Structs

Tuple structs are useful when field names would be redundant or when you want to create distinct types:

```rust
//===================================================
// Coordinates where position matters more than names
//===================================================
struct Point3D(f64, f64, f64);

//=====================================
// Type-safe wrappers (newtype pattern)
//=====================================
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

//======
// Usage
//======
let point = Point3D(3.0, 4.0, 0.0);
println!("Distance: {}", point.distance_from_origin());

//==================================
// Type safety prevents mixing units
//==================================
let distance_km = Kilometers(100.0);
let distance_mi = Miles(62.0);
//===============================================================================
// let total = distance_km.0 + distance_mi.0; // Compiles but semantically wrong!
//===============================================================================
```

**The pattern:** Use tuple structs when the structure itself conveys meaning more than field names would. They're particularly powerful for the newtype pattern.

### Unit Structs

Unit structs carry no data but can implement traits and provide type-level information:

```rust
//========================================
// Marker types for type-level programming
//========================================
struct Authenticated;
struct Unauthenticated;

//==================================
// Zero-sized types for phantom data
//==================================
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

//======
// Usage
//======
let db = Database::new("postgres://localhost".to_string());
//=====================================================================
// db.query("SELECT *"); // Error! Can't query unauthenticated database
//=====================================================================
let db = db.authenticate("secret").unwrap();
let results = db.query("SELECT * FROM users"); // Now this works
```

**The insight:** Unit structs enable compile-time state tracking without runtime overhead. This is the typestate pattern in action.

---

Now that we've seen the three forms of structs, let's explore one of the most powerful patterns in Rust: using tuple structs to create distinct types from primitives.

## Pattern 1: Newtype Pattern for Type Safety

The newtype pattern wraps a single value in a struct, creating a distinct type that prevents accidental mixing of values that are semantically different but have the same underlying representation.

```rust
//========================================
// Problem: Type aliases provide no safety
//========================================
type UserId = u64;
type ProductId = u64;

fn get_user(id: UserId) -> User { /* ... */ User }
fn get_product(id: ProductId) -> Product { /* ... */ Product }

// Compiles but wrong!
let user_id: UserId = 42;
let product = get_product(user_id);  // Type error not caught!

//==========================
// Solution: Newtype pattern
//==========================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProductId(u64);

fn get_user_safe(id: UserId) -> User { /* ... */ User }
fn get_product_safe(id: ProductId) -> Product { /* ... */ Product }

// Now this won't compile!
let user_id = UserId(42);
// let product = get_product_safe(user_id);  // Compile error!

//==============================================
// Pattern: Implement Deref for ergonomic access
//==============================================
use std::ops::Deref;

impl Deref for UserId {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}

//====================================================
// Pattern: Validated construction with private fields
//====================================================
pub struct Email(String);

impl Email {
    pub fn new(s: String) -> Result<Self, &'static str> {
        if s.contains('@') && s.contains('.') {
            Ok(Email(s))
        } else {
            Err("Invalid email format")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Cannot construct invalid Email
// let invalid = Email("not-an-email".to_string());  // Private field!
let valid = Email::new("user@example.com".to_string()).unwrap();

//============================================
// Pattern: Unit newtypes for different scales
//============================================
#[derive(Debug, Clone, Copy)]
struct Meters(f64);

#[derive(Debug, Clone, Copy)]
struct Feet(f64);

impl Meters {
    fn to_feet(self) -> Feet {
        Feet(self.0 * 3.28084)
    }
}

impl Feet {
    fn to_meters(self) -> Meters {
        Meters(self.0 / 3.28084)
    }
}

fn calculate_area(width: Meters, height: Meters) -> f64 {
    width.0 * height.0
}

// Won't compile: prevents mixing units
// let area = calculate_area(Meters(10.0), Feet(5.0));

//============================================
// Pattern: Opaque newtypes for API boundaries
//============================================
pub struct Token(String);

impl Token {
    pub(crate) fn new(s: String) -> Self {
        Token(s)
    }

    // No way to extract the string from outside the crate
}

pub fn authenticate(username: &str, password: &str) -> Option<Token> {
    // Validation logic
    if username.len() > 3 && password.len() > 8 {
        Some(Token::new(format!("{}:{}", username, password)))
    } else {
        None
    }
}

pub fn authorize(token: &Token) -> bool {
    // Can only be called with a properly constructed Token
    !token.0.is_empty()
}

//======================================================================
// Pattern: Newtype for index types (prevents indexing wrong collection)
//======================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StudentId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CourseId(usize);

struct Database {
    students: Vec<Student>,
    courses: Vec<Course>,
}

impl Database {
    fn get_student(&self, id: StudentId) -> Option<&Student> {
        self.students.get(id.0)
    }

    fn get_course(&self, id: CourseId) -> Option<&Course> {
        self.courses.get(id.0)
    }
}

struct Student;
struct Course;
struct User;
struct Product;
```

**When to use newtypes:**
- Domain modeling with distinct types (UserId, ProductId, Email)
- Units of measure (Meters, Seconds, Celsius)
- Validated strings (Email, URL, PhoneNumber)
- API boundaries where you want to prevent misuse
- Preventing index confusion between collections

**Performance characteristics:**
- Zero runtime cost (optimized away by compiler)
- Same memory layout as wrapped type
- No vtable or indirection
- Perfect for performance-critical code

### Transparent Wrappers with Deref

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

//======
// Usage
//======
let validated_string = Validated::new("hello".to_string());
println!("Length: {}", validated_string.len()); // Deref to String
println!("Age: {:?}", validated_string.age());  // Validated method
```

---

Having explored how to create distinct types with newtypes, let's look at how to work with struct instances efficiently using Rust's struct update syntax.

## Struct Update Syntax and Partial Moves

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

//======================
// Builder-style updates
//======================
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

//=====================================================
// config is now partially moved - can't use it anymore
//=====================================================
// println!("{:?}", config); // Error!

//================================
// Safe pattern: clone when needed
//================================
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

//========================
// Both configs are usable
//========================
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

//=============
// Partial move
//=============
let s = data.moveable;  // Moves String out
let n = data.copyable;  // Copies i32

//==============================================================
// data.moveable is moved, but data.copyable is still accessible
//==============================================================
println!("Copyable: {}", data.copyable); // OK
//===================================================================
// println!("{}", data.moveable); // Error: value borrowed after move
//===================================================================
```

**The pattern:** When building fluent APIs or config builders, be mindful of moves. Consider consuming self and returning Self, or use `&mut self` for in-place updates.

---

We've explored structs for grouping related data. Now let's turn to enums, which let you model data that can be one of several alternatives—each carrying its own specific data.

## Enum-Driven Architecture

Enums in Rust are far more powerful than in most languages. They enable algebraic data types that model complex domains precisely:

```rust
//===============================
// Model HTTP responses precisely
//===============================
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

//======
// Usage
//======
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

**The power:** Each variant can carry exactly the data it needs. There's no `null` or `undefined` - if a variant needs an ID, it has one. If it doesn't, it can't have one.

### Enum Variants with Rich Data

Enums shine when modeling state machines or complex workflows:

```rust
enum OrderStatus {
    Pending {
        items: Vec<String>,
        customer_id: u64,
    },
    Processing {
        order_id: u64,
        started_at: std::time::Instant,
    },
    Shipped {
        order_id: u64,
        tracking_number: String,
        carrier: String,
    },
    Delivered {
        order_id: u64,
        delivered_at: std::time::SystemTime,
        signature: Option<String>,
    },
    Cancelled {
        order_id: u64,
        reason: String,
    },
}

impl OrderStatus {
    fn process(self) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Pending { items, customer_id } => {
                if items.is_empty() {
                    return Err("Cannot process empty order".to_string());
                }
                Ok(OrderStatus::Processing {
                    order_id: 12345, // Generated
                    started_at: std::time::Instant::now(),
                })
            }
            _ => Err("Order is not in pending state".to_string()),
        }
    }

    fn ship(self, tracking_number: String, carrier: String) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Processing { order_id, .. } => {
                Ok(OrderStatus::Shipped {
                    order_id,
                    tracking_number,
                    carrier,
                })
            }
            _ => Err("Can only ship processing orders".to_string()),
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(self, OrderStatus::Pending { .. } | OrderStatus::Processing { .. })
    }
}
```

**The benefit:** Invalid state transitions become impossible. You can't ship a cancelled order because the types don't align.

---

We've seen how enums can carry rich data. Now let's explore how to extract and work with that data using Rust's powerful pattern matching features.

## Advanced Pattern Matching

Pattern matching extracts data from enums elegantly:

```rust
//========================
// Nested pattern matching
//========================
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(Color),
}

enum Color {
    Rgb(u8, u8, u8),
    Hsv(u8, u8, u8),
}

fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quit message received");
        }
        Message::Move { x, y } => {
            println!("Move to ({}, {})", x, y);
        }
        Message::Write(text) if text.len() > 100 => {
            println!("Long message: {}...", &text[..100]);
        }
        Message::Write(text) => {
            println!("Message: {}", text);
        }
        Message::ChangeColor(Color::Rgb(r, g, b)) => {
            println!("RGB color: ({}, {}, {})", r, g, b);
        }
        Message::ChangeColor(Color::Hsv(h, s, v)) => {
            println!("HSV color: ({}, {}, {})", h, s, v);
        }
    }
}

//==========================
// Match guards and bindings
//==========================
fn classify_response(response: &HttpResponse) -> &str {
    match response {
        HttpResponse::Ok { body, .. } if body.contains("error") => {
            "Ok response with error in body"
        }
        HttpResponse::Ok { .. } => "Success",
        HttpResponse::Created { .. } => "Created",
        HttpResponse::NoContent => "No content",
        HttpResponse::BadRequest { .. }
        | HttpResponse::Unauthorized
        | HttpResponse::NotFound => "Client error",
        HttpResponse::ServerError { details: Some(_), .. } => {
            "Server error with details"
        }
        HttpResponse::ServerError { .. } => "Server error",
    }
}
```

**Pattern matching exhaustiveness:** The compiler ensures you handle all cases. Add a new variant? The compiler tells you everywhere you need to update.

## Visitor Pattern with Enums

The visitor pattern in Rust leverages enums for traversing complex structures:

```rust
//=====================================
// AST for a simple expression language
//=====================================
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

//==============
// Visitor trait
//==============
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

//=======================
// Pretty printer visitor
//=======================
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

//==================
// Evaluator visitor
//==================
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

//======
// Usage
//======
fn demo_visitor() {
    //============
    // (3 + 4) * 2
    //============
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

## Type-Safe State Machines

State machines prevent invalid states and transitions at compile time:

```rust
//===========================
// Simple state machine: Door
//===========================
struct Open;
struct Closed;
struct Locked;

struct Door<State> {
    _state: std::marker::PhantomData<State>,
}

impl Door<Closed> {
    fn new() -> Self {
        println!("Door created in closed state");
        Door { _state: std::marker::PhantomData }
    }

    fn open(self) -> Door<Open> {
        println!("Opening door");
        Door { _state: std::marker::PhantomData }
    }

    fn lock(self) -> Door<Locked> {
        println!("Locking door");
        Door { _state: std::marker::PhantomData }
    }
}

impl Door<Open> {
    fn close(self) -> Door<Closed> {
        println!("Closing door");
        Door { _state: std::marker::PhantomData }
    }
}

impl Door<Locked> {
    fn unlock(self) -> Door<Closed> {
        println!("Unlocking door");
        Door { _state: std::marker::PhantomData }
    }
}

//=======================================
// Complex state machine with enum states
//=======================================
#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Authenticated { session_id: String, user_id: u64 },
}

struct Connection {
    state: ConnectionState,
    max_retries: u32,
}

impl Connection {
    fn new() -> Self {
        Connection {
            state: ConnectionState::Disconnected,
            max_retries: 3,
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        match &self.state {
            ConnectionState::Disconnected => {
                self.state = ConnectionState::Connecting { attempt: 1 };
                Ok(())
            }
            ConnectionState::Connecting { attempt } if *attempt < self.max_retries => {
                self.state = ConnectionState::Connecting { attempt: attempt + 1 };
                Ok(())
            }
            _ => Err("Cannot connect in current state".to_string()),
        }
    }

    fn establish(&mut self, session_id: String) -> Result<(), String> {
        match &self.state {
            ConnectionState::Connecting { .. } => {
                self.state = ConnectionState::Connected { session_id };
                Ok(())
            }
            _ => Err("Not in connecting state".to_string()),
        }
    }

    fn authenticate(&mut self, user_id: u64) -> Result<(), String> {
        match &self.state {
            ConnectionState::Connected { session_id } => {
                self.state = ConnectionState::Authenticated {
                    session_id: session_id.clone(),
                    user_id,
                };
                Ok(())
            }
            _ => Err("Must be connected to authenticate".to_string()),
        }
    }

    fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    fn is_authenticated(&self) -> bool {
        matches!(self.state, ConnectionState::Authenticated { .. })
    }
}

fn demo_state_machine() {
    //================
    // Type-state door
    //================
    let door = Door::<Closed>::new();
    let door = door.open();
    let door = door.close();
    let door = door.lock();
    //========================================================
    // door.open(); // Compile error! Can't open a locked door
    //========================================================
    let door = door.unlock();
    let _door = door.open();

    //======================
    // Enum-based connection
    //======================
    let mut conn = Connection::new();
    conn.connect().unwrap();
    conn.establish("session-123".to_string()).unwrap();
    conn.authenticate(42).unwrap();
    assert!(conn.is_authenticated());
    println!("Connection state: {:?}", conn.state);
}
```

**Why this works:** The type system enforces valid state transitions. You can't accidentally call `unlock()` on an open door because that method simply doesn't exist for `Door<Open>`.

### Combining Enums with Type States

For maximum safety, combine both approaches:

```rust
//=================================
// Payment processing state machine
//=================================
struct Pending;
struct Authorized;
struct Captured;
struct Refunded;

struct Payment<State> {
    id: String,
    amount: u64,
    state_data: State,
}

//============================
// Each state has its own data
//============================
impl Payment<Pending> {
    fn new(amount: u64) -> Self {
        Payment {
            id: format!("pay_{}", 12345),
            amount,
            state_data: Pending,
        }
    }

    fn authorize(self, auth_code: String) -> Payment<Authorized> {
        Payment {
            id: self.id,
            amount: self.amount,
            state_data: Authorized { auth_code },
        }
    }

    fn cancel(self) -> PaymentResult {
        PaymentResult::Cancelled { payment_id: self.id }
    }
}

struct Authorized {
    auth_code: String,
}

impl Payment<Authorized> {
    fn capture(self) -> Payment<Captured> {
        Payment {
            id: self.id,
            amount: self.amount,
            state_data: Captured {
                auth_code: self.state_data.auth_code,
                captured_at: std::time::SystemTime::now(),
            },
        }
    }

    fn void(self) -> PaymentResult {
        PaymentResult::Voided {
            payment_id: self.id,
            auth_code: self.state_data.auth_code,
        }
    }
}

struct Captured {
    auth_code: String,
    captured_at: std::time::SystemTime,
}

impl Payment<Captured> {
    fn refund(self, reason: String) -> Payment<Refunded> {
        Payment {
            id: self.id,
            amount: self.amount,
            state_data: Refunded {
                auth_code: self.state_data.auth_code,
                captured_at: self.state_data.captured_at,
                refunded_at: std::time::SystemTime::now(),
                reason,
            },
        }
    }
}

struct Refunded {
    auth_code: String,
    captured_at: std::time::SystemTime,
    refunded_at: std::time::SystemTime,
    reason: String,
}

enum PaymentResult {
    Cancelled { payment_id: String },
    Voided { payment_id: String, auth_code: String },
}

//=======================================
// Usage demonstrates compile-time safety
//=======================================
fn process_payment() {
    let payment = Payment::<Pending>::new(10000);
    //===================================================================
    // payment.capture(); // Compile error! Can't capture pending payment
    //===================================================================

    let payment = payment.authorize("AUTH123".to_string());
    let payment = payment.capture();
    //===========================================================
    // payment.authorize(...); // Compile error! Already captured
    //===========================================================

    let _result = payment.refund("Customer requested".to_string());
}
```

**The architecture:** Each state transition consumes the old state and returns a new one. Invalid transitions don't exist in the type system.

## Pattern 5: Trait Object Optimization

Trait objects enable dynamic dispatch, trading compile-time polymorphism for runtime flexibility. Understanding how to optimize trait objects is crucial for performance-sensitive code that requires dynamic behavior.

```rust
//=============================
// Pattern: Trait object basics
//=============================
trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

// Dynamic dispatch with trait objects
fn draw_shapes(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();  // Virtual function call
    }
}

//=======================================================
// Pattern: Minimize trait object size with thin pointers
//=======================================================
// Bad: Wide trait objects (multiple vtable pointers)
trait BadTrait: std::fmt::Debug + Clone + Send {}

// Good: Single trait, compose at usage site
trait GoodTrait: Send {}

fn process<T: GoodTrait + std::fmt::Debug>(value: T) {
    // Use trait bounds instead of multi-trait objects
}

//===============================================
// Pattern: Object-safe vs non-object-safe traits
//===============================================
// Object-safe: Can be made into trait object
trait ObjectSafe {
    fn method(&self);  // Takes &self
}

// Not object-safe: Generic methods
trait NotObjectSafe {
    fn generic<T>(&self, value: T);  // Can't be called on dyn NotObjectSafe
}

// Not object-safe: Returns Self
trait AlsoNotObjectSafe {
    fn clone(&self) -> Self;  // Self size unknown in trait object
}

//===================================
// Pattern: Making traits object-safe
//===================================
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

impl<T: Clone + 'static> Cloneable for T {
    fn clone_box(&self) -> Box<dyn Cloneable> {
        Box::new(self.clone())
    }
}

//================================================================
// Pattern: Enum dispatch instead of trait objects (when possible)
//================================================================
enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

impl Shape {
    fn draw(&self) {
        match self {
            Shape::Circle(c) => c.draw(),
            Shape::Rectangle(r) => r.draw(),
        }
    }
}

// Enum dispatch is faster: no vtable lookup
fn draw_shapes_fast(shapes: &[Shape]) {
    for shape in shapes {
        shape.draw();  // Direct call, compiler can inline
    }
}

//=====================================================
// Pattern: Small vector optimization for trait objects
//=====================================================
use std::mem;

enum SmallVec<T> {
    Inline([T; 3], usize),
    Heap(Vec<T>),
}

impl<T: Default + Copy> SmallVec<T> {
    fn new() -> Self {
        SmallVec::Inline([T::default(); 3], 0)
    }

    fn push(&mut self, value: T) {
        match self {
            SmallVec::Inline(arr, len) if *len < 3 => {
                arr[*len] = value;
                *len += 1;
            }
            SmallVec::Inline(arr, len) => {
                let mut vec = arr[..*len].to_vec();
                vec.push(value);
                *self = SmallVec::Heap(vec);
            }
            SmallVec::Heap(vec) => {
                vec.push(value);
            }
        }
    }
}

//===================================================
// Pattern: Trait object with static dispatch wrapper
//===================================================
trait Operation {
    fn execute(&self) -> i32;
}

struct Add(i32, i32);
impl Operation for Add {
    fn execute(&self) -> i32 { self.0 + self.1 }
}

struct Multiply(i32, i32);
impl Operation for Multiply {
    fn execute(&self) -> i32 { self.0 * self.1 }
}

// Static dispatch: monomorphization
fn execute_static<T: Operation>(op: &T) -> i32 {
    op.execute()  // Inlined
}

// Dynamic dispatch: trait object
fn execute_dynamic(op: &dyn Operation) -> i32 {
    op.execute()  // Virtual call
}

//================================
// Pattern: Downcast trait objects
//================================
use std::any::Any;

trait Component: Any {
    fn update(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct Position { x: f32, y: f32 }

impl Component for Position {
    fn update(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn get_position(component: &dyn Component) -> Option<&Position> {
    component.as_any().downcast_ref::<Position>()
}

//====================================================
// Pattern: Function pointers instead of trait objects
//====================================================
type DrawFn = fn(&str);

fn draw_circle(name: &str) {
    println!("Drawing circle: {}", name);
}

fn draw_rectangle(name: &str) {
    println!("Drawing rectangle: {}", name);
}

struct ShapeWithFn {
    name: String,
    draw: DrawFn,
}

// Slightly faster than trait objects: no vtable lookup
fn draw_with_fn(shapes: &[ShapeWithFn]) {
    for shape in shapes {
        (shape.draw)(&shape.name);
    }
}
```

**Trait object performance characteristics:**
- Dynamic dispatch: 1 indirect call (vtable lookup)
- Prevents inlining
- Cache misses on vtable access
- Larger binary size (no monomorphization duplication)

**Optimization strategies:**
- Use enum dispatch when variants are known
- Minimize trait object passing in hot paths
- Cache trait method results
- Use static dispatch in generic inner functions
- Consider function pointers for simple cases

**When to use trait objects:**
- Heterogeneous collections
- Plugin systems
- Dynamic loading
- When compile time polymorphism is impractical
- When binary size matters more than performance

## Pattern 6: Associated Types vs Generic Parameters

Understanding when to use associated types versus generic parameters is crucial for designing clean, ergonomic APIs.

```rust
//=============================================
// Pattern: Associated types for "output" types
//=============================================
trait Iterator {
    type Item;  // Output type

    fn next(&mut self) -> Option<Self::Item>;
}

// Cleaner than generic parameter:
// trait Iterator<Item> { fn next(&mut self) -> Option<Item>; }
// Because Item is always determined by the iterator type

//==============================================
// Pattern: Generic parameters for "input" types
//==============================================
trait From<T> {
    fn from(value: T) -> Self;
}

// Multiple From implementations for same type
impl From<i32> for String {
    fn from(n: i32) -> String {
        n.to_string()
    }
}

impl From<&str> for String {
    fn from(s: &str) -> String {
        s.to_string()
    }
}

//===========================================
// Pattern: Mix associated types and generics
//===========================================
trait Converter {
    type Output;

    fn convert<T: Into<Self::Output>>(&self, value: T) -> Self::Output;
}

//==================================
// Pattern: Associated type families
//==================================
trait Graph {
    type Node;
    type Edge;

    fn nodes(&self) -> Vec<Self::Node>;
    fn edges(&self) -> Vec<Self::Edge>;
    fn neighbors(&self, node: &Self::Node) -> Vec<Self::Node>;
}

struct AdjacencyList {
    adjacency: Vec<Vec<usize>>,
}

impl Graph for AdjacencyList {
    type Node = usize;
    type Edge = (usize, usize);

    fn nodes(&self) -> Vec<Self::Node> {
        (0..self.adjacency.len()).collect()
    }

    fn edges(&self) -> Vec<Self::Edge> {
        let mut edges = Vec::new();
        for (from, neighbors) in self.adjacency.iter().enumerate() {
            for &to in neighbors {
                edges.push((from, to));
            }
        }
        edges
    }

    fn neighbors(&self, node: &Self::Node) -> Vec<Self::Node> {
        self.adjacency.get(*node).cloned().unwrap_or_default()
    }
}
```

**Decision matrix:**
- **Associated type**: One natural output per implementation
- **Generic parameter**: Multiple implementations per type
- **Both**: Output types with input flexibility

## Performance Comparison

| Pattern | Compile Time | Runtime | Binary Size | Flexibility |
|---------|--------------|---------|-------------|-------------|
| Newtype | ✓ Fast | ✓ Zero cost | ✓ Small | Medium |
| Phantom types | ✓ Fast | ✓ Zero cost | ✓ Small | Low |
| Enum dispatch | ✓ Fast | ✓ Fast | Medium | Low |
| Trait objects | ✓ Fast | ✗ Dynamic dispatch | ✓ Small | High |

## Quick Reference

```rust
// Three struct forms
struct Named { field: i32 }                // Named fields
struct Tuple(i32, i32);                    // Tuple struct
struct Unit;                               // Unit struct

// Type safety without runtime cost
struct UserId(u64);                        // Newtype

// State machines in types
struct Connection<State> { _s: PhantomData<State> }

// Enum-driven architecture
enum Response { Ok { data: String }, Error(String) }

// Dynamic dispatch
Box<dyn Trait>                             // Heap-allocated trait object
&dyn Trait                                 // Borrowed trait object

// Static dispatch
fn generic<T: Trait>(x: T) { }             // Monomorphization
```

## Common Anti-Patterns

```rust
// ❌ Trait object when enum suffices
Box<dyn Operation>  // Slow

// ✓ Enum for closed set of types
enum Operation { Add, Multiply }  // Fast

// ❌ Generic parameter for single output type
trait Parser<Output> { fn parse(&self) -> Output; }

// ✓ Associated type for single output
trait Parser { type Output; fn parse(&self) -> Self::Output; }

// ❌ Type alias for domain types
type UserId = u64;  // No safety

// ✓ Newtype for domain types
struct UserId(u64);  // Type safe

// ❌ Over-engineering with phantom types
struct SimpleCounter<State> { count: usize, _s: PhantomData<State> }

// ✓ Simple when state machine not needed
struct SimpleCounter { count: usize }

// ❌ Runtime checks for state validity
fn ship_order(status: &str) -> Result<(), String> {
    if status != "processing" {
        return Err("Can only ship processing orders".to_string());
    }
    Ok(())
}

// ✓ Encode state in type system
impl OrderStatus {
    fn ship(self) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Processing { .. } => Ok(OrderStatus::Shipped { .. }),
            _ => Err("Can only ship processing orders".to_string()),
        }
    }
}
```

## Key Takeaways

1. **Choose the right struct form**: Named fields for clarity, tuple structs for semantic types, unit structs for markers
2. **Newtypes are free**: Use them liberally for domain modeling
3. **Encode state in types**: Make invalid states unrepresentable with enums and type states
4. **Leverage enum variants**: Each variant carries exactly what it needs
5. **Pattern match exhaustively**: Let the compiler ensure you handle all cases
6. **Trait objects have cost**: Profile before using in hot paths
7. **Enum dispatch often faster**: Closed set of types? Use enum
8. **Associated types for single outputs**: Generic parameters for inputs
9. **Type system is your friend**: Move validation to compile time

## Conclusion

Rust's type system is a powerful tool for writing correct, efficient software. The patterns in this chapter demonstrate a fundamental principle: **make illegal states unrepresentable**. When you encode invariants in types rather than runtime checks, you move entire classes of bugs from "things to test for" to "things that cannot compile."

### Choosing the Right Pattern

The patterns in this chapter solve different problems:

**For domain modeling:**
- Use **newtypes** when you have primitives that shouldn't mix (IDs, units, validated strings)
- Use **enums** when you have a closed set of alternatives, each with its own data
- Use **named structs** when you have related data with clear field names

**For state management:**
- Use **phantom types** for compile-time state machines where invalid transitions must be impossible
- Use **enum states** when states carry different data and runtime state checks are acceptable
- Combine both for maximum safety with per-state data

**For polymorphism:**
- Use **enum dispatch** when you have a closed set of types and performance matters
- Use **trait objects** when you need open extensibility or heterogeneous collections
- Use **generics** (not covered here) when types are known at compile time

**For API design:**
- Use **associated types** when there's one natural output type per implementation
- Use **generic parameters** when you want multiple implementations for the same type
- Use **builder patterns** with phantom types for complex construction with compile-time validation

### Common Pitfalls to Avoid

1. **Over-engineering**: Not everything needs phantom types. Simple runtime checks are fine for non-critical code.
2. **Type alias abuse**: `type UserId = u64` provides no safety. Use newtypes instead.
3. **Trait objects everywhere**: Dynamic dispatch has cost. Profile before optimizing, but prefer enums when possible.
4. **Ignoring exhaustiveness**: Pattern matching ensures you handle all cases. Don't silence warnings.
5. **Complex visitor patterns**: Not every tree needs the visitor pattern. Start simple, add complexity when needed.

### Performance Considerations

The patterns in this chapter demonstrate Rust's "zero-cost abstractions":
- **Newtypes**: Compiled away entirely, zero runtime cost
- **Phantom types**: Zero-sized, exist only at compile time
- **Enum dispatch**: Direct jumps, compiler can inline
- **Trait objects**: One vtable lookup, prevents inlining
- **Associated types**: Zero cost, resolved at compile time

### What's Next

This chapter covered practical type system patterns for everyday Rust development. To go deeper:

- **Chapter 33: Type-Level Programming** - Advanced techniques with GATs, const generics, and type-level computation
- **Chapter on Lifetimes** - How the type system tracks borrowing and memory safety
- **Chapter on Traits** - Deep dive into trait design, blanket implementations, and advanced trait patterns

### Final Thoughts

Rust's type system enables you to encode invariants that would be runtime checks (or bugs) in other languages. The patterns in this chapter show how to leverage structs, enums, and the type system to write code that's simultaneously safer and faster than traditional approaches.

Well-designed types don't just organize data—they make wrong code impossible to write. Start applying these patterns to your own code, beginning with simple newtypes and building up to state machines as you gain confidence. The compiler is your ally: when it rejects your code, it's often pointing you toward a design that's safer and clearer.

Master these patterns, and you'll write Rust code that's not just correct, but obviously correct—where the types themselves document and enforce your program's invariants.
