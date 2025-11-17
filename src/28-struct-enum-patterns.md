# Chapter 28: Struct & Enum Patterns

Structs and enums form the backbone of data modeling in Rust. While they may seem like simple data containers at first glance, mastering their patterns unlocks powerful techniques for encoding invariants, building type-safe APIs, and creating expressive domain models.

This chapter explores how expert Rust developers leverage structs and enums to write safer, more maintainable code. We'll see how choosing the right pattern can eliminate entire classes of bugs at compile time.

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

## Newtype and Wrapper Patterns

The newtype pattern wraps an existing type to create a distinct type with its own semantics:

```rust
use std::fmt;

//=============================
// Newtype for semantic clarity
//=============================
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UserId(u64);

#[derive(Debug, Clone, Copy)]
struct OrderId(u64);

//================================
// Prevent accidentally mixing IDs
//================================
fn get_user(id: UserId) -> User {
    println!("Fetching user {}", id.0);
    // ... fetch user
    unimplemented!()
}

//====================
// This won't compile:
//====================
// let order_id = OrderId(123);
//===================================
// get_user(order_id); // Type error!
//===================================

//=================================
// Wrapper for adding functionality
//=================================
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

//==============================
// Usage prevents invalid states
//==============================
let num = PositiveInteger::new(42).unwrap();
//=======================================================
// let invalid = PositiveInteger::new(-5); // Returns Err
//=======================================================
```

**Why wrappers matter:** They encode invariants in the type system. Once you have a `PositiveInteger`, you know it's valid. This eliminates defensive checks throughout your codebase.

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

## Type-Safe State Machines with Enums

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
    // Type-state door
    let door = Door::<Closed>::new();
    let door = door.open();
    let door = door.close();
    let door = door.lock();
    // door.open(); // Compile error! Can't open a locked door
    let door = door.unlock();
    let _door = door.open();

    // Enum-based connection
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
            id: format!("pay_{}", uuid::Uuid::new_v4()),
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
    // payment.capture(); // Compile error! Can't capture pending payment

    let payment = payment.authorize("AUTH123".to_string());
    let payment = payment.capture();
    // payment.authorize(...); // Compile error! Already captured

    let _result = payment.refund("Customer requested".to_string());
}
```

**The architecture:** Each state transition consumes the old state and returns a new one. Invalid transitions don't exist in the type system.

## Conclusion

Mastering struct and enum patterns in Rust means:

1. **Choose the right struct form**: Named fields for clarity, tuple structs for semantic types, unit structs for markers
2. **Use newtypes liberally**: Prevent mixing incompatible IDs and values
3. **Encode state in types**: Make invalid states unrepresentable
4. **Leverage enum variants**: Each variant carries exactly what it needs
5. **Pattern match exhaustively**: Let the compiler ensure you handle all cases
6. **Build state machines**: Use type states and enums to prevent bugs at compile time

The patterns in this chapter eliminate entire classes of runtime errors by moving validation to compile time. When you find yourself writing runtime checks for state validity, consider whether the type system could enforce those invariants instead.

Well-designed structs and enums don't just organize data—they make wrong code impossible to write.
