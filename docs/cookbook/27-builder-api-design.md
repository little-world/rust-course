# 27. Builder & API Design

API design is where your code meets its users. A well-designed API feels natural, guides users toward correct usage, and catches errors at compile time. Rust's type system enables sophisticated API patterns that make invalid states unrepresentable and correct usage obvious.

This chapter explores patterns for building expressive, type-safe APIs in Rust. From builder patterns that construct complex objects to the typestate pattern that encodes state machines in types, we'll see how to leverage Rust's features to create APIs that are both powerful and pleasant to use.

## Builder Pattern Variations

The builder pattern addresses a common problem: constructing objects with many optional parameters. Instead of constructors with long parameter lists or many setter methods, builders provide a fluent, chainable interface for configuration.

### The Problem: Complex Construction

Consider constructing an HTTP request:

```rust
// Bad: Too many parameters, unclear what each means
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

// Call site is confusing
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

### Basic Builder Pattern

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

### Builder with Required Fields

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

### Consuming Builder Pattern

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

### Non-Consuming Builder Pattern

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

## Typestate Pattern

The typestate pattern uses Rust's type system to encode state machines. Different states become different types, making it impossible to call methods inappropriate for the current state. This catches state errors at compile time.

### The Problem: State Validation

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

### Basic Typestate Pattern

Use different types for different states:

```rust
use std::marker::PhantomData;

// State markers
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

### Typestate with Builder

Combine typestate with the builder pattern:

```rust
// States
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

### Typestate for Protocols

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

### Combining Typestate and Lifetimes

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

## Fluent Interfaces

Fluent interfaces use method chaining to create readable, expression-like code. The goal is code that reads like natural language, making the API intuitive and self-documenting.

### Basic Fluent Interface

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

### Conditional Fluent Chains

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

### Fluent Interface with References

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

### Fluent Error Handling

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

## Extension Traits for Libraries

Extension traits add functionality to types you don't own. This is crucial for library design—you can extend standard library types or types from other crates without modifying their source.

### Basic Extension Trait

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

### Extension Traits for Generic Types

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

### Extension Traits for Error Handling

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

### Conditional Extension Traits

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

## Sealed Trait Pattern

The sealed trait pattern prevents external crates from implementing your trait. This is crucial when you want to add methods to a trait without breaking compatibility.

### Why Seal Traits?

Without sealing, adding methods to a trait is a breaking change:

```rust
// Your library v1.0
pub trait Operation {
    fn execute(&self);
}

// User implements your trait
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

### Basic Sealed Trait

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

### Sealed Trait with Associated Types

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

### Partially Sealed Traits

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

### Sealed Trait for Marker Traits

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

## Conclusion

API design in Rust is about leveraging the type system to create interfaces that are both powerful and safe. The patterns we've explored enable you to build APIs that guide users toward correct usage while catching errors at compile time.

**Key principles:**

1. **Builder pattern** provides flexible object construction with clear, self-documenting syntax
2. **Typestate pattern** encodes state machines in types, making invalid states unrepresentable
3. **Fluent interfaces** create readable, chainable APIs that feel natural to use
4. **Extension traits** add functionality to external types without modifying them
5. **Sealed traits** prevent external implementation, enabling non-breaking additions

**Design guidelines:**

- **Start simple**: Don't use complex patterns until you need them
- **Consider the caller**: APIs should be intuitive from the user's perspective
- **Fail at compile time**: Type-level guarantees are better than runtime checks
- **Document invariants**: Make assumptions explicit, especially with typestate
- **Think about evolution**: Sealed traits and careful design enable non-breaking changes

The best APIs feel like natural extensions of the language. They leverage Rust's strengths—the type system, ownership, and traits—to create interfaces that are impossible to misuse. When you write code using a well-designed API, the compiler guides you toward correct usage. Errors are clear, fixes are obvious, and the code reads like documentation.

As you design APIs, think about your users. What will their code look like? Where might they make mistakes? How can the type system prevent those mistakes? The patterns in this chapter are tools for answering these questions, helping you create APIs that are both delightful to use and impossible to misuse.

Remember: a great API makes correct code easy to write and incorrect code hard to write. Use Rust's type system to enforce your invariants, and your users will thank you with fewer bugs and clearer code.
