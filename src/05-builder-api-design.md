# Builder & API Design

[Builder Pattern Variations](#pattern-1-builder-pattern-variations)

- Problem: Constructors with many parameters unclear; optional fields confusing; can't enforce required fields at compile-time
- Solution: Basic builder (mut self), consuming builder (self), non-consuming (&mut self); Result from build() for validation
- Why It Matters: Self-documenting code; fluent API; compile-time required fields with typestate; defaults for optional fields
- Use Cases: HTTP requests, database connections, configuration objects, query builders, complex object construction

[Typestate Pattern](#pattern-2-typestate-pattern)

- Problem: Invalid state transitions possible; runtime state checks; can't enforce "must call authenticate before query" at compile-time
- Solution: Different types for different states; state transitions consume old type, return new type; methods only on valid states
- Why It Matters: Impossible states unrepresentable; compile-time state machine; zero runtime cost; API misuse prevented
- Use Cases: Database connections (Unauthenticated→Authenticated), file handles (Open/Closed), builders (incomplete→complete), protocols

[Method Chaining and Fluent APIs](#pattern-3-method-chaining-and-fluent-apis)

- Problem: Repeated object references; verbose configuration; unclear operation order; mutation vs consumption unclear
- Solution: Return Self or &mut Self for chaining; consuming pattern (self) prevents reuse; named parameters via methods
- Why It Matters: Ergonomic configuration; clear intent; compiler prevents invalid chains; self-documenting
- Use Cases: Builders, query DSLs, test assertions, configuration, iterators, command patterns

[Into/AsRef for Flexible Parameters](#pattern-4-intoasref-for-flexible-parameters)

- Problem: String vs &str parameters force conversions; accepting only one type limits flexibility; allocations when borrowing sufficient
- Solution: Use Into<String> for owned parameters, AsRef<str> for borrowed; generic conversions with Into/From traits
- Why It Matters: Ergonomic APIs accept both owned and borrowed; no forced allocations; caller convenience; zero-cost abstractions
- Use Cases: String parameters, path parameters, any parameter with multiple valid types, builder methods, flexible APIs

[Must-Use Types and Linear Types](#pattern-5-must-use-types-and-linear-types)

- Problem: Ignoring important return values (errors, connections); forgetting to call build(); resource leaks possible
- Solution: #[must_use] attribute; linear types (must be consumed); typestate prevents partial usage; Result<T, E> must be handled
- Why It Matters: Compiler warnings for ignored values; prevents resource leaks; enforces API contracts; no silent failures
- Use Cases: Error handling, builders, resource handles (files, connections), guards (MutexGuard), transaction types

[Builder Cheat Sheet] (#builder-cheat-sheet)
- This comprehensive guide covers both builder patterns (basic and typestate) and typestate patterns for state machines.

## Overview

This chapter explores API design patterns: builder pattern variations for complex construction, typestate pattern for compile-time state machines, fluent APIs via method chaining, flexible parameters with Into/AsRef, and must-use types to prevent misuse.

## Pattern 1: Builder Pattern Variations

**Problem**: Constructors with many parameters are unclear—which parameter is which in `new(url, method, headers, body, timeout, retry, follow)`? Optional fields represented as Option in constructor still require passing None. Can't enforce required fields at compile-time (forgot to set username). Many combinations of optional parameters = constructor explosion. Defaults for optional fields not obvious. Function signature changes break all callers.

**Solution**: Basic builder: methods take `mut self`, return Self, enable chaining, call build() to construct. Consuming builder: methods take `self`, prevent reuse, ensure single use. Non-consuming builder: methods take `&mut self`, allow reuse. Required fields: Option in builder, build() returns Result validating all set. Typestate builder: different type per state, build() only available when complete. Defaults in builder::new(). Each method self-documents its purpose.

**Why It Matters**: Self-documenting code: `.timeout(30)` clearer than positional parameter. Fluent API improves ergonomics: chain methods, clear intent. Compile-time required fields with typestate: forgot field = compile error not runtime. Defaults obvious: builder::new() shows defaults. Backward compatible: adding optional field doesn't break existing builders. Type safety: wrong order impossible with named methods. Zero cost: builder compiles away.

**Use Cases**: HTTP request builders (method, headers, body, timeout), database connection builders (host, port, credentials, pool size), configuration objects (app config with many optional settings), query builders (SQL construction fluent API), complex object construction (many fields with sensible defaults), test data builders (factories for test objects), CLI argument parsing (command builders), email construction (to, subject, body, attachments).

### The Problem: Complex Construction

```rust
//==================================================
// Bad: Too many parameters, unclear what each means
//==================================================
fn make_request(
    url: &str,
    method: &str,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout: Option<Duration>,
    retry_count: Option<u32>,
    follow_redirects: bool,
) -> Request {
    // ...
}

//=======================
// Call site is confusing
//=======================
let request = make_request(
    "https://api.example.com",
    "POST",
    vec![("Authorization".to_string(), "Bearer token".to_string())],
    Some("{\"data\": \"value\"}".to_string()),
    Some(Duration::from_secs(30)),
    Some(3),
    true,
);
```

This is hard to read and maintain. Which parameter is which? What if you only want to set timeout but accept defaults for everything else?

### Example: Basic Builder Pattern

The builder pattern solves this elegantly:

```rust
use std::time::Duration;

#[derive(Debug)]
pub struct Request {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout: Option<Duration>,
    retry_count: u32,
    follow_redirects: bool,
}

pub struct RequestBuilder {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout: Option<Duration>,
    retry_count: u32,
    follow_redirects: bool,
}

impl Request {
    pub fn builder(url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(url)
    }
}

impl RequestBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        RequestBuilder {
            url: url.into(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
            timeout: None,
            retry_count: 0,
            follow_redirects: true,
        }
    }

    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    pub fn retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    pub fn build(self) -> Request {
        Request {
            url: self.url,
            method: self.method,
            headers: self.headers,
            body: self.body,
            timeout: self.timeout,
            retry_count: self.retry_count,
            follow_redirects: self.follow_redirects,
        }
    }
}

fn example() {
    let request = Request::builder("https://api.example.com")
        .method("POST")
        .header("Authorization", "Bearer token")
        .body("{\"data\": \"value\"}")
        .timeout(Duration::from_secs(30))
        .retry_count(3)
        .build();

    println!("{:?}", request);
}
```

Now the code is self-documenting. Each method call clearly states what it's configuring. You can set only the options you care about, accepting defaults for the rest.

### Example: Builder with Required Fields

Sometimes certain fields must be provided. You can enforce this at compile time:

```rust
pub struct Database {
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
    pool_size: u32,
    timeout: Duration,
}

pub struct DatabaseBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
    pool_size: u32,
    timeout: Duration,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        DatabaseBuilder {
            host: None,
            port: None,
            username: None,
            password: None,
            database: None,
            pool_size: 10,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    pub fn build(self) -> Result<Database, String> {
        Ok(Database {
            host: self.host.ok_or("host is required")?,
            port: self.port.ok_or("port is required")?,
            username: self.username.ok_or("username is required")?,
            password: self.password.ok_or("password is required")?,
            database: self.database.ok_or("database is required")?,
            pool_size: self.pool_size,
            timeout: self.timeout,
        })
    }
}

fn example() -> Result<(), String> {
    let db = DatabaseBuilder::new()
        .host("localhost")
        .port(5432)
        .username("admin")
        .password("secret")
        .database("myapp")
        .pool_size(20)
        .build()?;

    Ok(())
}
```

This approach validates at runtime—`build()` returns `Result`. If you forget a required field, you get a clear error message. However, the error only appears when you call `build()`, not at compile time. The typestate pattern solves this.

### Example: Consuming Builder Pattern

Some builders are consumed as they're built:

```rust
pub struct QueryBuilder {
    table: String,
    conditions: Vec<String>,
    limit: Option<usize>,
}

impl QueryBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        QueryBuilder {
            table: table.into(),
            conditions: Vec::new(),
            limit: None,
        }
    }

    // Takes ownership, returns ownership
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    // Consumes the builder, returns the final product
    pub fn to_sql(self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        sql
    }
}

fn example() {
    let query = QueryBuilder::new("users")
        .where_clause("age > 18")
        .where_clause("active = true")
        .limit(10)
        .to_sql();

    println!("{}", query);
    // SELECT * FROM users WHERE age > 18 AND active = true LIMIT 10
}
```

Each method takes `self` (not `&mut self`), consumes the builder, modifies it, and returns it. This enables method chaining while maintaining move semantics. The builder is consumed by `to_sql()`, preventing accidental reuse.

### Example: Non-Consuming Builder Pattern

For builders that you might reuse, use `&mut self`:

```rust
pub struct EmailBuilder {
    to: Vec<String>,
    subject: String,
    body: String,
}

impl EmailBuilder {
    pub fn new() -> Self {
        EmailBuilder {
            to: Vec::new(),
            subject: String::new(),
            body: String::new(),
        }
    }

    pub fn to(&mut self, email: impl Into<String>) -> &mut Self {
        self.to.push(email.into());
        self
    }

    pub fn subject(&mut self, subject: impl Into<String>) -> &mut Self {
        self.subject = subject.into();
        self
    }

    pub fn body(&mut self, body: impl Into<String>) -> &mut Self {
        self.body = body.into();
        self
    }

    pub fn send(&self) -> Result<(), String> {
        if self.to.is_empty() {
            return Err("No recipients".to_string());
        }
        if self.subject.is_empty() {
            return Err("No subject".to_string());
        }

        println!("Sending email to: {:?}", self.to);
        println!("Subject: {}", self.subject);
        println!("Body: {}", self.body);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.to.clear();
        self.subject.clear();
        self.body.clear();
    }
}

fn example() -> Result<(), String> {
    let mut builder = EmailBuilder::new();

    builder
        .to("alice@example.com")
        .subject("First email")
        .body("Hello, Alice!")
        .send()?;

    // Reuse the builder
    builder.clear();
    builder
        .to("bob@example.com")
        .subject("Second email")
        .body("Hello, Bob!")
        .send()?;

    Ok(())
}
```

This pattern allows reusing the builder for multiple operations.

## Pattern 2: Typestate Pattern

**Problem**: State machines checked at runtime—"if connected { query() } else { panic!() }"—slow and error-prone. Can't enforce "must authenticate before query" at compile-time. Invalid state transitions possible (query on disconnected connection). Forgot to transition state = runtime panic. State represented as enum requires matching everywhere. Builder allows build() before all required fields set. File operations possible after close().

**Solution**: Different types for different states: `Connection<Connecting>`, `Connection<Connected>`, `Connection<Disconnected>`. State transitions consume old type, return new type: `fn connect(self) -> Connection<Connected>`. Methods only available on appropriate states: only Connected has query(). PhantomData<State> for zero-sized state marker. Typestate builder: Builder<NoFields> → Builder<WithUrl> → Builder<Complete>, build() only on Complete. Compiler enforces state machine.

**Why It Matters**: Impossible states unrepresentable: can't have Connection in invalid state. Compile-time state machine: wrong transition = compile error. Zero runtime cost: PhantomData is 0 bytes, states are compile-time only. API misuse prevented: can't call query() on unauthenticated connection. Self-documenting: Connection<Authenticated> shows state in type. No runtime checks: state verified at compile-time. Exhaustive transitions: all transitions explicit in type signatures.

**Use Cases**: Database connections (Unauthenticated → Authenticated), file handles (Open/Closed states), protocol state machines (HTTP connection states), builder pattern (ensure all fields set), resource lifecycle (Acquired/Released), async operations (Pending/Ready), payment processing (Pending→Authorized→Captured), document workflow (Draft→Review→Published).

## Problem: Runtime Checks

Consider a TCP connection:

```rust
struct Connection {
    state: ConnectionState,
    socket: Option<TcpStream>,
}

enum ConnectionState {
    Disconnected,
    Connected,
    Closed,
}

impl Connection {
    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        // Must check state at runtime
        if !matches!(self.state, ConnectionState::Connected) {
            return Err("Not connected".to_string());
        }

        // Send data...
        Ok(())
    }
}
```

Runtime checks are error-prone. You might forget to check the state, leading to bugs. The typestate pattern moves these checks to compile time.

### Example: Basic Typestate Pattern

Use different types for different states:

```rust
use std::marker::PhantomData;

//==============
// State markers
//==============
struct Disconnected;
struct Connected;
struct Closed;

struct Connection<State> {
    socket: Option<TcpStream>,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new() -> Self {
        Connection {
            socket: None,
            _state: PhantomData,
        }
    }

    fn connect(self, addr: &str) -> Result<Connection<Connected>, String> {
        // Attempt connection
        let socket = TcpStream::connect(addr)
            .map_err(|e| e.to_string())?;

        Ok(Connection {
            socket: Some(socket),
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn send(&mut self, data: &[u8]) -> Result<(), String> {
        // No state check needed - compiler guarantees we're connected!
        self.socket
            .as_mut()
            .unwrap()
            .write_all(data)
            .map_err(|e| e.to_string())
    }

    fn close(self) -> Connection<Closed> {
        Connection {
            socket: None,
            _state: PhantomData,
        }
    }
}

impl Connection<Closed> {
    // No methods - can't do anything with a closed connection
}

use std::net::TcpStream;
use std::io::Write;

fn example() -> Result<(), String> {
    let conn = Connection::new();

    // conn.send(b"data"); // Compile error! Not connected

    let mut conn = conn.connect("127.0.0.1:8080")?;

    conn.send(b"Hello, server!")?; // OK - we're connected

    let conn = conn.close();

    // conn.send(b"data"); // Compile error! Connection is closed

    Ok(())
}
```

The compiler prevents calling `send()` on a disconnected or closed connection. Invalid state transitions are impossible.

### Example: Typestate with Builder

Combine typestate with the builder pattern:

```rust
//=======
// States
//=======
struct NoName;
struct HasName;
struct NoEmail;
struct HasEmail;

struct UserBuilder<NameState, EmailState> {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    _name_state: PhantomData<NameState>,
    _email_state: PhantomData<EmailState>,
}

impl UserBuilder<NoName, NoEmail> {
    fn new() -> Self {
        UserBuilder {
            name: None,
            email: None,
            age: None,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<E> UserBuilder<NoName, E> {
    fn name(self, name: impl Into<String>) -> UserBuilder<HasName, E> {
        UserBuilder {
            name: Some(name.into()),
            email: self.email,
            age: self.age,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<N> UserBuilder<N, NoEmail> {
    fn email(self, email: impl Into<String>) -> UserBuilder<N, HasEmail> {
        UserBuilder {
            name: self.name,
            email: Some(email.into()),
            age: self.age,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<N, E> UserBuilder<N, E> {
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
}

impl UserBuilder<HasName, HasEmail> {
    fn build(self) -> User {
        User {
            name: self.name.unwrap(),
            email: self.email.unwrap(),
            age: self.age,
        }
    }
}

struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

fn example() {
    let user = UserBuilder::new()
        .name("Alice")
        .email("alice@example.com")
        .age(30)
        .build();

    // This won't compile - missing email:
    // let user = UserBuilder::new()
    //     .name("Bob")
    //     .build();

    // This won't compile - wrong order (no name yet):
    // let user = UserBuilder::new()
    //     .email("charlie@example.com")
    //     .build();
}
```

The type system enforces that you can only build when all required fields are set. The error messages are clear and occur at compile time.

### Example: Typestate for Protocols

Typestate is excellent for encoding communication protocols:

```rust
struct Handshake;
struct Active;
struct Terminating;

struct Protocol<State> {
    buffer: Vec<u8>,
    _state: PhantomData<State>,
}

impl Protocol<Handshake> {
    fn new() -> Self {
        Protocol {
            buffer: Vec::new(),
            _state: PhantomData,
        }
    }

    fn send_hello(mut self) -> Result<Protocol<Active>, String> {
        self.buffer.extend_from_slice(b"HELLO");
        // Send buffer...
        Ok(Protocol {
            buffer: Vec::new(),
            _state: PhantomData,
        })
    }
}

impl Protocol<Active> {
    fn send_data(&mut self, data: &[u8]) -> Result<(), String> {
        self.buffer.extend_from_slice(data);
        // Send buffer...
        Ok(())
    }

    fn begin_shutdown(self) -> Protocol<Terminating> {
        Protocol {
            buffer: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl Protocol<Terminating> {
    fn send_goodbye(mut self) -> Result<(), String> {
        self.buffer.extend_from_slice(b"GOODBYE");
        // Send buffer...
        Ok(())
    }
}

fn example() -> Result<(), String> {
    let proto = Protocol::new();
    let mut proto = proto.send_hello()?;

    proto.send_data(b"message 1")?;
    proto.send_data(b"message 2")?;

    let proto = proto.begin_shutdown();
    proto.send_goodbye()?;

    Ok(())
}
```

The protocol can only be used in the correct sequence. You can't send data before handshake, and you can't send data after beginning shutdown.

### Example: Combining Typestate and Lifetimes

Typestate can ensure references remain valid:

```rust
struct Unparsed;
struct Parsed<'a>;

struct Document<'a, State> {
    content: &'a str,
    parsed: Option<Vec<&'a str>>,
    _state: PhantomData<State>,
}

impl<'a> Document<'a, Unparsed> {
    fn new(content: &'a str) -> Self {
        Document {
            content,
            parsed: None,
            _state: PhantomData,
        }
    }

    fn parse(self) -> Document<'a, Parsed<'a>> {
        let parsed = self.content.lines().collect();
        Document {
            content: self.content,
            parsed: Some(parsed),
            _state: PhantomData,
        }
    }
}

impl<'a> Document<'a, Parsed<'a>> {
    fn get_line(&self, index: usize) -> Option<&'a str> {
        self.parsed.as_ref()?.get(index).copied()
    }

    fn line_count(&self) -> usize {
        self.parsed.as_ref().map(|p| p.len()).unwrap_or(0)
    }
}

fn example() {
    let content = "line 1\nline 2\nline 3";
    let doc = Document::new(content);

    // doc.get_line(0); // Compile error - not parsed yet

    let doc = doc.parse();

    println!("Line count: {}", doc.line_count());
    println!("First line: {:?}", doc.get_line(0));
}
```

## Pattern 3: Method Chaining and Fluent APIs

**Problem**: Repeated object references verbose: `builder.set_x(); builder.set_y(); builder.set_z()`. Configuration code unclear—which operations are related? Operation order not obvious from code structure. Mutation vs consumption unclear (does method move value?). Intermediate state exposed between related operations. Can't enforce operation order.

**Solution**: Return Self or &mut Self for chaining: `builder.x().y().z()`. Consuming pattern (self) prevents reuse, enforces single-use. Non-consuming (&mut self) allows reuse. Named parameters via methods: `.timeout(30)` clearer than positional args. Method names read like sentences: `query().select("*").from("users").where("active")`. Builder ensures all operations complete before use.

**Why It Matters**: Ergonomic configuration: chain methods, no intermediate variables. Clear intent: method names show purpose. Compiler prevents invalid chains: wrong order = type error with typestate. Self-documenting: reads like natural language. Reduced boilerplate: no repeated references. Type safety: methods only available when valid. Fluent APIs guide users to correct usage through API design.

**Use Cases**: Query builders (SQL DSLs), test assertions (expect(x).to_be(y)), configuration objects (builder pattern), iterator combinators (map/filter/collect), command builders (CLI construction), HTTP request builders, async chain operations (then/and_then), reactive programming (Observable methods).

### Example: Fluent API
```rust
struct QueryBuilder {
    select: Vec<String>,
    from: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
}

impl QueryBuilder {
    fn new() -> Self {
        QueryBuilder {
            select: Vec::new(),
            from: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
        }
    }

    fn select(mut self, field: impl Into<String>) -> Self {
        self.select.push(field.into());
        self
    }

    fn from(mut self, table: impl Into<String>) -> Self {
        self.from = Some(table.into());
        self
    }

    fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.where_clauses.push(condition.into());
        self
    }

    fn order_by(mut self, field: impl Into<String>) -> Self {
        self.order_by.push(field.into());
        self
    }

    fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    fn build(self) -> String {
        let mut query = String::from("SELECT ");

        if self.select.is_empty() {
            query.push('*');
        } else {
            query.push_str(&self.select.join(", "));
        }

        if let Some(table) = self.from {
            query.push_str(&format!(" FROM {}", table));
        }

        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            query.push_str(" ORDER BY ");
            query.push_str(&self.order_by.join(", "));
        }

        if let Some(n) = self.limit {
            query.push_str(&format!(" LIMIT {}", n));
        }

        query
    }
}

fn example() {
    let query = QueryBuilder::new()
        .select("id")
        .select("name")
        .select("email")
        .from("users")
        .where_clause("age > 18")
        .where_clause("active = true")
        .order_by("name")
        .limit(10)
        .build();

    println!("{}", query);
    // SELECT id, name, email FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10
}
```

The code reads naturally: "select id, name, email from users where...". This is the hallmark of a good fluent interface.

### Example: Conditional Fluent Chains

Support conditional configuration:

```rust
impl QueryBuilder {
    fn where_clause_if(
        self,
        condition: bool,
        clause: impl Into<String>,
    ) -> Self {
        if condition {
            self.where_clause(clause)
        } else {
            self
        }
    }

    fn order_by_if(
        self,
        condition: bool,
        field: impl Into<String>,
    ) -> Self {
        if condition {
            self.order_by(field)
        } else {
            self
        }
    }
}

fn example_conditional(include_email: bool, sort_by_name: bool) {
    let mut builder = QueryBuilder::new()
        .select("id")
        .select("name");

    if include_email {
        builder = builder.select("email");
    }

    let query = builder
        .from("users")
        .where_clause("active = true")
        .order_by_if(sort_by_name, "name")
        .build();

    println!("{}", query);
}
```

### Example: Fluent Interface with References

For non-consuming builders, use `&mut self`:

```rust
struct EmailComposer {
    to: Vec<String>,
    cc: Vec<String>,
    subject: String,
    body: String,
}

impl EmailComposer {
    fn new() -> Self {
        EmailComposer {
            to: Vec::new(),
            cc: Vec::new(),
            subject: String::new(),
            body: String::new(),
        }
    }

    fn to(&mut self, address: impl Into<String>) -> &mut Self {
        self.to.push(address.into());
        self
    }

    fn cc(&mut self, address: impl Into<String>) -> &mut Self {
        self.cc.push(address.into());
        self
    }

    fn subject(&mut self, subject: impl Into<String>) -> &mut Self {
        self.subject = subject.into();
        self
    }

    fn body(&mut self, body: impl Into<String>) -> &mut Self {
        self.body = body.into();
        self
    }

    fn send(&self) -> Result<(), String> {
        if self.to.is_empty() {
            return Err("No recipients".to_string());
        }

        println!("Sending email:");
        println!("  To: {}", self.to.join(", "));
        if !self.cc.is_empty() {
            println!("  CC: {}", self.cc.join(", "));
        }
        println!("  Subject: {}", self.subject);
        println!("  Body: {}", self.body);

        Ok(())
    }
}

fn example() -> Result<(), String> {
    let mut email = EmailComposer::new();

    email
        .to("alice@example.com")
        .to("bob@example.com")
        .cc("manager@example.com")
        .subject("Team Update")
        .body("Here's the latest...")
        .send()?;

    Ok(())
}
```

### Example: Fluent Error Handling

Fluent interfaces can integrate error handling:

```rust
struct Pipeline<T> {
    value: Result<T, String>,
}

impl<T> Pipeline<T> {
    fn new(value: T) -> Self {
        Pipeline { value: Ok(value) }
    }

    fn map<U, F>(self, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        Pipeline {
            value: self.value.map(f),
        }
    }

    fn and_then<U, F>(self, f: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> Result<U, String>,
    {
        Pipeline {
            value: self.value.and_then(f),
        }
    }

    fn finalize(self) -> Result<T, String> {
        self.value
    }
}

fn example() -> Result<(), String> {
    let result = Pipeline::new("  hello world  ")
        .map(|s| s.trim())
        .map(|s| s.to_uppercase())
        .and_then(|s| {
            if s.len() > 5 {
                Ok(s)
            } else {
                Err("String too short".to_string())
            }
        })
        .map(|s| format!("[{}]", s))
        .finalize()?;

    println!("{}", result); // [HELLO WORLD]

    Ok(())
}
```

Errors short-circuit the pipeline, making error handling natural and composable.

## Pattern 4: Into/AsRef for Flexible Parameters

**Problem**: String vs &str parameter dilemma—accepting String forces allocation, accepting &str forces caller to own String. Path vs &Path, Vec vs &[T] same issue. Function parameters inflexible—can't accept both owned and borrowed. Type conversions explicit and verbose. Builder methods require specific types. API forces unnecessary allocations.

**Solution**: Use `impl Into<String>` for parameters needing owned values—accepts String, &str, Cow<str>. Use `impl AsRef<str>` for parameters needing borrowed access—accepts String, &str, any reference. Generic conversions: Into<T>/From<T> for type conversions, AsRef<T> for borrowing. Builder methods use Into for ergonomic chaining. Path parameters: `impl AsRef<Path>` accepts String, &str, PathBuf, &Path. Zero-cost: monomorphization eliminates abstraction overhead.

**Why It Matters**: Ergonomic APIs: accept both owned and borrowed without forcing conversions. No unnecessary allocations: AsRef borrows when possible. Caller convenience: can pass literal strings, owned strings, or references. Zero-cost abstractions: Into/AsRef compile to efficient code. Type flexibility: same function works with many types. Future-proof: new types implementing Into/AsRef work automatically. Less boilerplate: no manual .to_string()/.as_ref() calls.

**Use Cases**: String parameters (impl Into<String> or impl AsRef<str>), path parameters (impl AsRef<Path>), any parameter with owned/borrowed variants, builder methods (ergonomic chaining), generic collection parameters (impl AsRef<[T]>), conversion-heavy APIs, library public interfaces, configuration builders.

### Example: String Extension
```rust
trait StringExt {
    fn truncate_words(&self, max_words: usize) -> String;
    fn reverse_words(&self) -> String;
}

impl StringExt for str {
    fn truncate_words(&self, max_words: usize) -> String {
        self.split_whitespace()
            .take(max_words)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn reverse_words(&self) -> String {
        self.split_whitespace()
            .rev()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn example() {
    let text = "The quick brown fox jumps over the lazy dog";

    println!("{}", text.truncate_words(3)); // "The quick brown"
    println!("{}", text.reverse_words()); // "dog lazy the over jumps fox brown quick The"
}
```

Anyone who imports your `StringExt` trait gets these methods on all strings.

### Example: Extension Traits for Generic Types

Extend generic types with trait bounds:

```rust
trait IteratorExt: Iterator {
    fn collect_string(self) -> String
    where
        Self: Sized,
        Self::Item: std::fmt::Display,
    {
        self.map(|item| item.to_string()).collect()
    }

    fn intersperse_with<F>(self, separator: F) -> IntersperseWith<Self, F>
    where
        Self: Sized,
        F: Fn() -> Self::Item,
    {
        IntersperseWith {
            iter: self,
            separator,
            needs_separator: false,
        }
    }
}

impl<I: Iterator> IteratorExt for I {}

struct IntersperseWith<I, F> {
    iter: I,
    separator: F,
    needs_separator: bool,
}

impl<I, F> Iterator for IntersperseWith<I, F>
where
    I: Iterator,
    F: Fn() -> I::Item,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.needs_separator {
            self.needs_separator = false;
            Some((self.separator)())
        } else {
            self.needs_separator = true;
            self.iter.next()
        }
    }
}

fn example() {
    let result = vec![1, 2, 3]
        .into_iter()
        .intersperse_with(|| 0)
        .collect_string();

    println!("{}", result); // "10203"
}
```

### Example: Extension Traits for Error Handling

Make error handling more ergonomic:

```rust
trait ResultExt<T, E> {
    fn log_error(self, context: &str) -> Self;
    fn unwrap_or_log(self, default: T) -> T
    where
        E: std::fmt::Display;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn log_error(self, context: &str) -> Self {
        if let Err(ref e) = self {
            eprintln!("[ERROR] {}: {}", context, e);
        }
        self
    }

    fn unwrap_or_log(self, default: T) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                eprintln!("[ERROR] {}", e);
                default
            }
        }
    }
}

fn example() {
    let result: Result<i32, &str> = Err("something went wrong");

    let value = result
        .log_error("Failed to compute value")
        .unwrap_or_log(0);

    println!("Value: {}", value);
}
```

### Example: Conditional Extension Traits

Provide extensions only when certain traits are implemented:

```rust
trait SliceExt<T> {
    fn find_max(&self) -> Option<&T>
    where
        T: Ord;

    fn sum_all(&self) -> T
    where
        T: std::iter::Sum + Copy;
}

impl<T> SliceExt<T> for [T] {
    fn find_max(&self) -> Option<&T>
    where
        T: Ord,
    {
        self.iter().max()
    }

    fn sum_all(&self) -> T
    where
        T: std::iter::Sum + Copy,
    {
        self.iter().copied().sum()
    }
}

fn example() {
    let numbers = vec![1, 5, 3, 9, 2];

    println!("Max: {:?}", numbers.find_max()); // Some(9)
    println!("Sum: {}", numbers.sum_all()); // 20

    let strings = vec!["apple", "banana", "cherry"];
    println!("Max string: {:?}", strings.find_max()); // Some("cherry")
    // strings.sum_all(); // Compile error - &str doesn't implement Sum
}
```

## Pattern 5: Must-Use Types and Linear Types

**Problem**: Ignoring important return values causes bugs—ignored Result hides errors, ignored connection leaks resources. Forgetting to call build() on builder leaves incomplete state. Resource leaks: file handle not closed, mutex guard dropped too early. Silent failures: error ignored, program continues. No warning for unused values. Connection acquired but never used. Transaction started but not committed.

**Solution**: #[must_use] attribute on types/functions generates compiler warnings for unused values. Linear types pattern: value must be consumed exactly once (builders). Typestate prevents partial usage: can only call certain methods in certain states. Result<T, E> automatically #[must_use]. Guards (MutexGuard, file handles) must be assigned. API design: return types require handling. Consuming methods (self) enforce single use.

**Why It Matters**: Compiler warnings prevent bugs: unused Result = warning. Resource leak prevention: must handle handles. Enforces API contracts: builder must call build(). No silent failures: errors must be handled. Linear types ensure correctness: value used exactly once. Type system enforces discipline: can't ignore critical values. Production safety: catch mistakes at compile-time.

**Use Cases**: Error handling (Result must be handled), builders (must call build()), resource handles (files, connections must be used/closed), guards (MutexGuard, RwLockGuard), transaction types (must commit or rollback), iterators (must consume or iterator does nothing), async futures (must await), lock guards (must be held for scope).

### Problem: Breaking Changes

```rust
//==================
// Your library v1.0
//==================
pub trait Operation {
    fn execute(&self);
}

//===========================
// User implements your trait
//===========================
struct UserOperation;
impl Operation for UserOperation {
    fn execute(&self) {
        println!("User operation");
    }
}

// Your library v1.1 - BREAKING CHANGE!
// pub trait Operation {
//     fn execute(&self);
//     fn validate(&self) -> bool; // New method breaks user code
// }
```

Adding `validate()` breaks all external implementations. The sealed trait pattern prevents this.

### Example: Basic Sealed Trait

```rust
mod private {
    pub trait Sealed {}
}

pub trait Operation: private::Sealed {
    fn execute(&self);

    // Can add this later without breaking compatibility
    fn validate(&self) -> bool {
        true // Default implementation
    }
}

struct InternalOperation;

impl private::Sealed for InternalOperation {}

impl Operation for InternalOperation {
    fn execute(&self) {
        println!("Internal operation");
    }
}
// External crates cannot implement Operation because they can't implement private::Sealed
```

Users can use the trait but can't implement it. You can add methods without breaking compatibility.

### Example: Sealed Trait with Associated Types

```rust
mod sealed {
    pub trait Sealed {
        type Internal;
    }
}

pub trait DataSource: sealed::Sealed {
    type Item;

    fn fetch(&self) -> Vec<Self::Item>;

    // Added in v2.0 - not a breaking change
    fn count(&self) -> usize {
        self.fetch().len()
    }
}

struct Database;

impl sealed::Sealed for Database {
    type Internal = ();
}

impl DataSource for Database {
    type Item = String;

    fn fetch(&self) -> Vec<String> {
        vec!["data1".to_string(), "data2".to_string()]
    }
}

fn use_data_source<T: DataSource>(source: T) {
    println!("Count: {}", source.count());
}
```

### Example: Partially Sealed Traits

Sometimes you want to seal some parts but not others:

```rust
mod sealed {
    pub trait Sealed {}
}

// Sealed trait - cannot implement externally
pub trait ProtocolHandler: sealed::Sealed {
    fn handle(&self, data: &[u8]);
}

// Open trait - can implement externally
pub trait MessageTransform {
    fn transform(&self, message: String) -> String;
}

struct Handler<T: MessageTransform> {
    transformer: T,
}

impl<T: MessageTransform> sealed::Sealed for Handler<T> {}

impl<T: MessageTransform> ProtocolHandler for Handler<T> {
    fn handle(&self, data: &[u8]) {
        let message = String::from_utf8_lossy(data).to_string();
        let transformed = self.transformer.transform(message);
        println!("Handled: {}", transformed);
    }
}

// Users can implement MessageTransform
struct UppercaseTransform;

impl MessageTransform for UppercaseTransform {
    fn transform(&self, message: String) -> String {
        message.to_uppercase()
    }
}

fn example() {
    let handler = Handler {
        transformer: UppercaseTransform,
    };

    handler.handle(b"hello");
}
```

Users can provide custom transformers but can't implement the handler trait itself.

### Example: Sealed Trait for Marker Traits

Seal marker traits to create closed sets:

```rust
mod sealed {
    pub trait Sealed {}

    impl Sealed for i32 {}
    impl Sealed for i64 {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

pub trait Numeric: sealed::Sealed {
    fn zero() -> Self;
    fn one() -> Self;
}

impl Numeric for i32 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
}

impl Numeric for i64 {
    fn zero() -> Self { 0 }
    fn one() -> Self { 1 }
}

impl Numeric for f32 {
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
}

impl Numeric for f64 {
    fn zero() -> Self { 0.0 }
    fn one() -> Self { 1.0 }
}

fn compute<T: Numeric>(value: T) -> T {
    // Can add methods to Numeric without breaking this code
    value
}
```

Only your predefined types can implement `Numeric`.

## Summary

This chapter covered API design patterns for creating type-safe, ergonomic Rust APIs:

1. **Builder Pattern Variations**: Basic (mut self), consuming (self), non-consuming (&mut self), Result validation
2. **Typestate Pattern**: Different types per state, transitions consume old/return new, compile-time state machines
3. **Method Chaining and Fluent APIs**: Return Self/&mut Self, consuming for single-use, reads like natural language
4. **Into/AsRef for Flexible Parameters**: Accept owned/borrowed via Into<T>/AsRef<T>, zero-cost ergonomics
5. **Must-Use Types and Linear Types**: #[must_use] warnings, value consumed exactly once, enforces handling

**Key Takeaways**:
- Builder pattern self-documents: `.timeout(30)` clearer than positional parameters
- Typestate impossible states unrepresentable: can't query() unauthenticated connection
- Method chaining ergonomic: `builder.x().y().z()` no intermediate variables
- Into/AsRef flexible: accept String, &str, Cow<str> with one parameter type
- #[must_use] prevents bugs: unused Result generates warning

**API Design Principles**:
- Self-documenting: method names show intent, types show state
- Compile-time safety: type errors better than runtime panics
- Ergonomic: fluent chaining, flexible parameters, sensible defaults
- Impossible to misuse: wrong usage doesn't compile
- Future-proof: sealed traits allow non-breaking additions
- Zero-cost: abstractions compile away

**Pattern Selection**:
- Use builder for complex construction (many optional parameters)
- Use typestate for state machines (compile-time state checking)
- Use method chaining for configuration (fluent, ergonomic)
- Use Into/AsRef for flexible parameters (owned or borrowed)
- Use #[must_use] for critical returns (errors, resources)


**Performance Considerations**:
- Builder: zero runtime cost, compiles to direct construction
- Typestate: PhantomData is 0 bytes, purely compile-time
- Method chaining: inlined by compiler, no overhead
- Into/AsRef: monomorphization eliminates abstraction cost
- Must-use: compile-time only, no runtime impact

**Anti-Patterns to Avoid**:
- Too many positional parameters (use builder)
- Runtime state checks (use typestate)
- Exposing intermediate builder state (consume builder)
- Forcing String allocation when &str sufficient (use AsRef)
- Ignoring Result (use #[must_use])
- Public mutable fields (use builder/setters)
- Missing documentation for complex state transitions

**Testing APIs**:
- Test happy path with builder
- Test missing required fields (compile error with typestate)
- Test invalid state transitions (compile error)
- Test method chaining readability
- Test parameter flexibility (both owned/borrowed work)
- Test must-use warnings (should warn if ignored)


## Builder Cheat Sheet
```rust
// ===== BASIC BUILDER PATTERN =====
#[derive(Debug)]
struct User {
    username: String,
    email: String,
    age: Option<u32>,
    active: bool,
}

struct UserBuilder {
    username: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    active: bool,
}

impl UserBuilder {
    fn new() -> Self {
        UserBuilder {
            username: None,
            email: None,
            age: None,
            active: true,
        }
    }
    
    fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }
    
    fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
    
    fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    
    fn build(self) -> Result<User, String> {
        Ok(User {
            username: self.username.ok_or("username is required")?,
            email: self.email.ok_or("email is required")?,
            age: self.age,
            active: self.active,
        })
    }
}

fn basic_builder_example() {
    let user = UserBuilder::new()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build()
        .unwrap();
    
    println!("{:?}", user);
}

// ===== BUILDER WITH REFERENCES =====
struct ConfigBuilder<'a> {
    host: Option<&'a str>,
    port: Option<u16>,
    timeout: Option<u64>,
}

impl<'a> ConfigBuilder<'a> {
    fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            timeout: None,
        }
    }
    
    fn host(mut self, host: &'a str) -> Self {
        self.host = Some(host);
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
    
    fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or("localhost").to_string(),
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}

#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
    timeout: u64,
}

// ===== BUILDER WITH DEFAULTS =====
#[derive(Debug, Default)]
struct ServerConfig {
    host: String,
    port: u16,
    workers: usize,
    timeout: u64,
}

impl ServerConfig {
    fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::default()
    }
}

#[derive(Default)]
struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    workers: Option<usize>,
    timeout: Option<u64>,
}

impl ServerConfigBuilder {
    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }
    
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    fn workers(mut self, workers: usize) -> Self {
        self.workers = Some(workers);
        self
    }
    
    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn build(self) -> ServerConfig {
        ServerConfig {
            host: self.host.unwrap_or_else(|| "0.0.0.0".to_string()),
            port: self.port.unwrap_or(8080),
            workers: self.workers.unwrap_or(4),
            timeout: self.timeout.unwrap_or(30),
        }
    }
}

fn builder_with_defaults() {
    let config = ServerConfig::builder()
        .host("localhost")
        .port(3000)
        .build();
    
    println!("{:?}", config);
}

// ===== TYPESTATE BUILDER PATTERN =====
// Compile-time enforcement of builder state

use std::marker::PhantomData;

// State markers
struct Empty;
struct HasUsername;
struct HasEmail;

// Builder with typestate
struct TypedUserBuilder<UsernameState, EmailState> {
    username: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    _username_state: PhantomData<UsernameState>,
    _email_state: PhantomData<EmailState>,
}

impl TypedUserBuilder<Empty, Empty> {
    fn new() -> Self {
        TypedUserBuilder {
            username: None,
            email: None,
            age: None,
            _username_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<EmailState> TypedUserBuilder<Empty, EmailState> {
    fn username(self, username: String) -> TypedUserBuilder<HasUsername, EmailState> {
        TypedUserBuilder {
            username: Some(username),
            email: self.email,
            age: self.age,
            _username_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<UsernameState> TypedUserBuilder<UsernameState, Empty> {
    fn email(self, email: String) -> TypedUserBuilder<UsernameState, HasEmail> {
        TypedUserBuilder {
            username: self.username,
            email: Some(email),
            age: self.age,
            _username_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<UsernameState, EmailState> TypedUserBuilder<UsernameState, EmailState> {
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
}

// Only allow build when both username and email are set
impl TypedUserBuilder<HasUsername, HasEmail> {
    fn build(self) -> User {
        User {
            username: self.username.unwrap(),
            email: self.email.unwrap(),
            age: self.age,
            active: true,
        }
    }
}

fn typestate_builder_example() {
    let user = TypedUserBuilder::new()
        .username("alice".to_string())
        .email("alice@example.com".to_string())
        .age(30)
        .build();
    
    // This won't compile - missing required fields:
    // let user = TypedUserBuilder::new()
    //     .username("alice".to_string())
    //     .build(); // ERROR: no method `build` found
    
    println!("{:?}", user);
}

// ===== TYPESTATE PATTERN FOR STATE MACHINES =====
// Door state machine with typestate

struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        println!("Creating locked door");
        Door { _state: PhantomData }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        println!("Unlocking door");
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        println!("Locking door");
        Door { _state: PhantomData }
    }
    
    fn open(&self) {
        println!("Opening door");
    }
}

fn door_state_example() {
    let door = Door::<Locked>::new();
    // door.open(); // ERROR: method not found for Door<Locked>
    
    let door = door.unlock();
    door.open(); // OK
    
    let door = door.lock();
    // door.open(); // ERROR: method not found for Door<Locked>
}

// ===== CONNECTION STATE MACHINE =====
struct Disconnected;
struct Connecting;
struct Connected;
struct Failed;

struct Connection<State> {
    address: String,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new(address: String) -> Self {
        Connection {
            address,
            _state: PhantomData,
        }
    }
    
    fn connect(self) -> Result<Connection<Connecting>, Connection<Failed>> {
        println!("Attempting to connect to {}", self.address);
        Ok(Connection {
            address: self.address,
            _state: PhantomData,
        })
    }
}

impl Connection<Connecting> {
    fn establish(self) -> Result<Connection<Connected>, Connection<Failed>> {
        println!("Establishing connection...");
        Ok(Connection {
            address: self.address,
            _state: PhantomData,
        })
    }
}

impl Connection<Connected> {
    fn send(&self, data: &str) {
        println!("Sending data: {}", data);
    }
    
    fn receive(&self) -> String {
        "Received data".to_string()
    }
    
    fn disconnect(self) -> Connection<Disconnected> {
        println!("Disconnecting");
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

impl Connection<Failed> {
    fn retry(self) -> Connection<Disconnected> {
        println!("Retrying connection");
        Connection {
            address: self.address,
            _state: PhantomData,
        }
    }
}

fn connection_state_example() {
    let conn = Connection::<Disconnected>::new("localhost:8080".to_string());
    
    let conn = match conn.connect() {
        Ok(conn) => conn,
        Err(failed) => return,
    };
    
    let conn = match conn.establish() {
        Ok(conn) => conn,
        Err(failed) => return,
    };
    
    conn.send("Hello");
    let data = conn.receive();
    println!("{}", data);
    
    let conn = conn.disconnect();
}

// ===== FILE HANDLE STATE MACHINE =====
struct Closed;
struct Open;

struct File<State> {
    path: String,
    _state: PhantomData<State>,
}

impl File<Closed> {
    fn new(path: String) -> Self {
        File {
            path,
            _state: PhantomData,
        }
    }
    
    fn open(self) -> std::io::Result<File<Open>> {
        println!("Opening file: {}", self.path);
        Ok(File {
            path: self.path,
            _state: PhantomData,
        })
    }
}

impl File<Open> {
    fn read(&self) -> String {
        format!("Contents of {}", self.path)
    }
    
    fn write(&mut self, data: &str) {
        println!("Writing to {}: {}", self.path, data);
    }
    
    fn close(self) -> File<Closed> {
        println!("Closing file: {}", self.path);
        File {
            path: self.path,
            _state: PhantomData,
        }
    }
}

// ===== TRANSACTION STATE MACHINE =====
struct NotStarted;
struct InProgress;
struct Committed;
struct RolledBack;

struct Transaction<State> {
    id: u64,
    _state: PhantomData<State>,
}

impl Transaction<NotStarted> {
    fn new(id: u64) -> Self {
        Transaction {
            id,
            _state: PhantomData,
        }
    }
    
    fn begin(self) -> Transaction<InProgress> {
        println!("Beginning transaction {}", self.id);
        Transaction {
            id: self.id,
            _state: PhantomData,
        }
    }
}

impl Transaction<InProgress> {
    fn execute(&mut self, query: &str) {
        println!("Executing in transaction {}: {}", self.id, query);
    }
    
    fn commit(self) -> Transaction<Committed> {
        println!("Committing transaction {}", self.id);
        Transaction {
            id: self.id,
            _state: PhantomData,
        }
    }
    
    fn rollback(self) -> Transaction<RolledBack> {
        println!("Rolling back transaction {}", self.id);
        Transaction {
            id: self.id,
            _state: PhantomData,
        }
    }
}

fn transaction_example() {
    let tx = Transaction::<NotStarted>::new(1);
    let mut tx = tx.begin();
    
    tx.execute("INSERT INTO users VALUES (1, 'alice')");
    tx.execute("INSERT INTO users VALUES (2, 'bob')");
    
    let tx = tx.commit();
}

// ===== PROTOCOL STATE MACHINE =====
struct Handshake;
struct Authenticated;
struct Active;

struct Protocol<State> {
    session_id: String,
    _state: PhantomData<State>,
}

impl Protocol<Handshake> {
    fn new() -> Self {
        Protocol {
            session_id: uuid::Uuid::new_v4().to_string(),
            _state: PhantomData,
        }
    }
    
    fn authenticate(self, token: &str) -> Result<Protocol<Authenticated>, String> {
        if token == "valid_token" {
            Ok(Protocol {
                session_id: self.session_id,
                _state: PhantomData,
            })
        } else {
            Err("Invalid token".to_string())
        }
    }
}

impl Protocol<Authenticated> {
    fn activate(self) -> Protocol<Active> {
        Protocol {
            session_id: self.session_id,
            _state: PhantomData,
        }
    }
}

impl Protocol<Active> {
    fn send_message(&self, message: &str) {
        println!("Sending: {}", message);
    }
    
    fn receive_message(&self) -> String {
        "Received message".to_string()
    }
}

// ===== BUILDER WITH TYPESTATE AND VALIDATION =====
struct NoHost;
struct HasHost;
struct NoPort;
struct HasPort;

struct ServerBuilder<HostState, PortState> {
    host: Option<String>,
    port: Option<u16>,
    workers: usize,
    _host_state: PhantomData<HostState>,
    _port_state: PhantomData<PortState>,
}

impl ServerBuilder<NoHost, NoPort> {
    fn new() -> Self {
        ServerBuilder {
            host: None,
            port: None,
            workers: 4,
            _host_state: PhantomData,
            _port_state: PhantomData,
        }
    }
}

impl<PortState> ServerBuilder<NoHost, PortState> {
    fn host(self, host: impl Into<String>) -> ServerBuilder<HasHost, PortState> {
        ServerBuilder {
            host: Some(host.into()),
            port: self.port,
            workers: self.workers,
            _host_state: PhantomData,
            _port_state: PhantomData,
        }
    }
}

impl<HostState> ServerBuilder<HostState, NoPort> {
    fn port(self, port: u16) -> ServerBuilder<HostState, HasPort> {
        ServerBuilder {
            host: self.host,
            port: Some(port),
            workers: self.workers,
            _host_state: PhantomData,
            _port_state: PhantomData,
        }
    }
}

impl<HostState, PortState> ServerBuilder<HostState, PortState> {
    fn workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }
}

impl ServerBuilder<HasHost, HasPort> {
    fn build(self) -> Server {
        Server {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
            workers: self.workers,
        }
    }
}

#[derive(Debug)]
struct Server {
    host: String,
    port: u16,
    workers: usize,
}

fn typed_server_builder_example() {
    let server = ServerBuilder::new()
        .host("localhost")
        .port(8080)
        .workers(8)
        .build();
    
    println!("{:?}", server);
    
    // Won't compile - missing required fields:
    // let server = ServerBuilder::new()
    //     .host("localhost")
    //     .build(); // ERROR
}

// ===== PHANTOM TYPE FOR UNITS =====
struct Meters;
struct Kilometers;

struct Distance<Unit> {
    value: f64,
    _unit: PhantomData<Unit>,
}

impl Distance<Meters> {
    fn meters(value: f64) -> Self {
        Distance {
            value,
            _unit: PhantomData,
        }
    }
    
    fn to_kilometers(self) -> Distance<Kilometers> {
        Distance {
            value: self.value / 1000.0,
            _unit: PhantomData,
        }
    }
}

impl Distance<Kilometers> {
    fn kilometers(value: f64) -> Self {
        Distance {
            value,
            _unit: PhantomData,
        }
    }
    
    fn to_meters(self) -> Distance<Meters> {
        Distance {
            value: self.value * 1000.0,
            _unit: PhantomData,
        }
    }
}

fn distance_example() {
    let d1 = Distance::<Meters>::meters(5000.0);
    let d2 = d1.to_kilometers();
    println!("Distance: {} km", d2.value);
    
    // Won't compile - different units:
    // let d3 = Distance::<Meters>::meters(100.0);
    // let d4 = Distance::<Kilometers>::kilometers(1.0);
    // let sum = d3.value + d4.value; // Types are different!
}

// ===== COMPILE-TIME GUARANTEES =====
struct Draft;
struct Published;

struct BlogPost<State> {
    title: String,
    content: String,
    _state: PhantomData<State>,
}

impl BlogPost<Draft> {
    fn new(title: String) -> Self {
        BlogPost {
            title,
            content: String::new(),
            _state: PhantomData,
        }
    }
    
    fn write(&mut self, content: &str) {
        self.content.push_str(content);
    }
    
    fn publish(self) -> BlogPost<Published> {
        println!("Publishing: {}", self.title);
        BlogPost {
            title: self.title,
            content: self.content,
            _state: PhantomData,
        }
    }
}

impl BlogPost<Published> {
    fn view(&self) -> String {
        format!("{}\n\n{}", self.title, self.content)
    }
    
    // Cannot write to published post
}

fn blog_post_example() {
    let mut post = BlogPost::<Draft>::new("My Post".to_string());
    post.write("This is the content.");
    // post.view(); // ERROR: method not available on Draft
    
    let post = post.publish();
    println!("{}", post.view());
    // post.write("More content"); // ERROR: method not available on Published
}

// ===== PAYMENT STATE MACHINE =====
struct Pending;
struct Authorized;
struct Captured;
struct Refunded;

struct Payment<State> {
    amount: u64,
    transaction_id: String,
    _state: PhantomData<State>,
}

impl Payment<Pending> {
    fn new(amount: u64) -> Self {
        Payment {
            amount,
            transaction_id: uuid::Uuid::new_v4().to_string(),
            _state: PhantomData,
        }
    }
    
    fn authorize(self) -> Result<Payment<Authorized>, String> {
        println!("Authorizing payment of ${}", self.amount);
        Ok(Payment {
            amount: self.amount,
            transaction_id: self.transaction_id,
            _state: PhantomData,
        })
    }
}

impl Payment<Authorized> {
    fn capture(self) -> Payment<Captured> {
        println!("Capturing payment");
        Payment {
            amount: self.amount,
            transaction_id: self.transaction_id,
            _state: PhantomData,
        }
    }
}

impl Payment<Captured> {
    fn refund(self) -> Payment<Refunded> {
        println!("Refunding payment");
        Payment {
            amount: self.amount,
            transaction_id: self.transaction_id,
            _state: PhantomData,
        }
    }
}

// Mock uuid for example
mod uuid {
    pub struct Uuid;
    impl Uuid {
        pub fn new_v4() -> Self { Uuid }
        pub fn to_string(&self) -> String { "uuid".to_string() }
    }
}

fn payment_example() {
    let payment = Payment::<Pending>::new(10000);
    let payment = payment.authorize().unwrap();
    let payment = payment.capture();
    // Can only refund captured payments
    let payment = payment.refund();
}
```