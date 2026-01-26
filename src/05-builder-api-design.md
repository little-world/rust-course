# Builder & API Design

This chapter explores several powerful API design patterns in Rust:
- **Builder Pattern**: For flexible and readable complex object construction.
- **Typestate Pattern**: For compile-time state machine validation.
- **Fluent APIs**: Using method chaining for ergonomic code.
- **Generic Parameters**: Employing `Into`/`AsRef` for flexible function arguments.
- **`#[must_use]` attribute**: To prevent accidental misuse of important return values.



## Pattern 1: Builder Pattern Variations

The Builder pattern provides a flexible and readable way to construct complex objects, especially those with multiple optional fields or a lengthy configuration process.

-   **Problem**: Constructors with numerous parameters are confusing and error-prone. It's hard to remember the order of arguments, and handling optional fields with `Option<T>` is verbose.

-   **Solution**: Instead of creating an object in a single step, a builder object is used to configure the final object piece by piece through a series of method calls. A final `.build()` method then constructs the object.

-   **Why it matters**: This pattern leads to self-documenting code (e.g., `.timeout(30)` is clearer than a positional argument). It improves ergonomics with a fluent, chainable API.

-   **Use cases**:
    -   Building HTTP requests (`reqwest::Client::get()`).
    -   Configuring database connections (`sqlx::postgres::PgConnectOptions`).
    -   Constructing complex UI components or application configuration objects.
    -   Creating test data with varying parameters.

### Example: Basic Consuming Builder

This is the most common builder variant. Each setter method consumes the builder (`takes self`) and returns a new one, allowing for method chaining. The final `.build()` call consumes the builder and returns the constructed object. This ensures the builder cannot be accidentally reused.

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
    // Provide a convenient entry point to the builder.
    pub fn builder(url: impl Into<String>) -> RequestBuilder {
        RequestBuilder::new(url)
    }
}

impl RequestBuilder {
    // The `new` function sets defaults for all fields.
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

    // Each method takes `self` and returns `self` for chaining.
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    pub fn header(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }
    
    // The `build` method consumes builder, creates final object.
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

// Chain setters for readable config; build() creates final object
let request = Request::builder("https://api.example.com")
    .method("POST")
    .header("Authorization", "Bearer token")
    .body("{\"data\": \"value\"}")
    .build();
```

### Example: Builder with Runtime Validation

When some fields are required, the builder can store them as `Option<T>` and the `.build()` method can return a `Result`. This moves validation from compile time to a single runtime check, ensuring no required fields are missed.

```rust
#[derive(Debug)]
pub struct Database {
    host: String,
    port: u16,
    username: String,
}

// The builder stores required fields as `Option`.
pub struct DatabaseBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        DatabaseBuilder { host: None, port: None, username: None }
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

    // `build` returns `Result` to enforce required fields.
    pub fn build(self) -> Result<Database, String> {
        let host = self.host.ok_or("host is required")?;
        let port = self.port.ok_or("port is required")?;
        let username = self.username.ok_or("username is required")?;
        
        Ok(Database { host, port, username })
    }
}

// build() returns Result to catch missing required fields
let db_result = DatabaseBuilder::new()
    .host("localhost")
    .port(5432)
    .username("admin")
    .build();
assert!(db_result.is_ok());

let db_fail = DatabaseBuilder::new().host("localhost").build();
assert!(db_fail.is_err()); // Missing port and username
```

### Example: Non-Consuming (Mutable) Builder

For builders that you might want to reuse or incrementally build, methods can take a mutable reference (`&mut self`). This allows calling methods multiple times and creating multiple objects from the same builder instance.

```rust
#[derive(Debug)]
pub struct Email {
    to: Vec<String>,
    subject: String,
    body: String,
}

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

    // Methods take `&mut self` and return `&mut Self` for chaining.
    pub fn to(&mut self, email: impl Into<String>) -> &mut Self {
        self.to.push(email.into());
        self
    }

    pub fn subject(
        &mut self,
        subject: impl Into<String>,
    ) -> &mut Self {
        self.subject = subject.into();
        self
    }

    pub fn body(&mut self, body: impl Into<String>) -> &mut Self {
        self.body = body.into();
        self
    }
    
    // The build/send method now typically borrows the builder.
    pub fn build(&self) -> Email {
        Email {
            to: self.to.clone(),
            subject: self.subject.clone(),
            body: self.body.clone()
        }
    }
    
    pub fn clear(&mut self) {
        self.to.clear();
        self.subject.clear();
        self.body.clear();
    }
}

// Mutable builder can be reused; clear() resets for next build
let mut builder = EmailBuilder::new();
builder.to("a@example.com").subject("First").body("Hello");
let email1 = builder.build();
builder.clear();
builder.to("b@example.com").subject("Second").body("World");
let email2 = builder.build();
```

---

## Pattern 2: Typestate Pattern

The Typestate pattern encodes the state of an object into the type system itself. This makes invalid state transitions a compile-time error rather than a runtime panic.

-   **Problem**: State machines are often implemented with enums and checked at runtime (e.g., `if self.state == State::Connected`). This is error-prone, as it's easy to forget a state, handle a transition incorrectly, or call a method in the wrong state, leading to panics.

-   **Solution**: Represent each state with a distinct type. State transitions are functions that consume an object in one state and return a new object in another state.

-   **Why It Matters**: Invalid state transitions become compile errors, not runtime panics. You cannot call `send()` on a disconnected socket—the method simply doesn't exist for that type. This eliminates entire classes of bugs at zero runtime cost.

-   **Use cases**:
    -   Database connections (`Unauthenticated` -> `Authenticated`).
    -   File handles (`Open` -> `Written` -> `Closed`).
    -   Protocol state machines (e.g., HTTP request/response cycle).
    -   Builder patterns that require fields to be set in a specific order (see Example 2).
    -   Resource lifecycle management (`Acquired` -> `Released`).

### Example: Typestate for a Connection

This example models a connection that can be `Disconnected` or `Connected`. Methods like `send` are only available on a `Connection<Connected>`, which the compiler enforces.

```rust
use std::marker::PhantomData;
use std::io::{self, Write};
use std::net::TcpStream;

// State marker types are zero-sized structs.
#[derive(Debug)]
struct Disconnected;
#[derive(Debug)]
struct Connected;

// The Connection is generic over its state.
struct Connection<State> {
    stream: Option<TcpStream>,
    _state: PhantomData<State>,
}

// In the `Disconnected` state, we can only connect.
impl Connection<Disconnected> {
    fn new() -> Self {
        Connection { stream: None, _state: PhantomData }
    }

    fn connect(
        self,
        addr: &str,
    ) -> io::Result<Connection<Connected>> {
        let stream = TcpStream::connect(addr)?;
        println!("Connected to {}", addr);
        Ok(Connection { stream: Some(stream), _state: PhantomData })
    }
}

// In the `Connected` state, we can send data.
impl Connection<Connected> {
    fn send(&mut self, data: &[u8]) -> io::Result<()> {
        let stream = self.stream.as_mut()
            .expect("Stream must exist in Connected state");
        stream.write_all(data)?;
        println!("Sent data: {}", String::from_utf8_lossy(data));
        Ok(())
    }

    fn close(self) -> Connection<Disconnected> {
        if let Some(stream) = self.stream {
            drop(stream); // Close the connection.
        }
        println!("Connection closed.");
        Connection { stream: None, _state: PhantomData }
    }
}
// send() only exists on Connected; compile error if Disconnected
let conn = Connection::new();
// conn.send(b"hello"); // ERROR: no `send` on Disconnected
let mut connected = conn.connect("127.0.0.1:8080")?;
connected.send(b"hello")?;
let _closed = connected.close();
```

### Example: Typestate Builder for Compile-Time Validation

The typestate pattern can be combined with a builder to enforce that required fields are set *at compile time*. The `.build()` method is only made available on the final state type, after all required setters have been called.

```rust
use std::marker::PhantomData;

// State markers for the builder
#[derive(Default)]
struct NoName;
#[derive(Default)]
struct HasName;
#[derive(Default)]
struct NoEmail;
#[derive(Default)]
struct HasEmail;

#[derive(Debug)]
struct User { name: String, email: String }

// The builder is generic over its name and email states.
struct UserBuilder<NameState, EmailState> {
    name: Option<String>,
    email: Option<String>,
    _name_state: PhantomData<NameState>,
    _email_state: PhantomData<EmailState>,
}

// Initial state: no name, no email.
impl Default for UserBuilder<NoName, NoEmail> {
    fn default() -> Self {
        UserBuilder {
            name: None,
            email: None,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

// Methods transition builder to new state by returning new type.
impl<E> UserBuilder<NoName, E> {
    fn name(
        self,
        name: impl Into<String>,
    ) -> UserBuilder<HasName, E> {
        UserBuilder {
            name: Some(name.into()),
            email: self.email,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<N> UserBuilder<N, NoEmail> {
    fn email(
        self,
        email: impl Into<String>,
    ) -> UserBuilder<N, HasEmail> {
        UserBuilder {
            name: self.name,
            email: Some(email.into()),
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

// `build` only available in `HasName, HasEmail` state.
impl UserBuilder<HasName, HasEmail> {
    fn build(self) -> User {
        User {
            name: self.name.expect("guaranteed by typestate"),
            email: self.email.expect("guaranteed by typestate"),
        }
    }
}
// build() only available after both name() and email() called
let user = UserBuilder::default()
    .name("Alice")
    .email("alice@example.com")
    .build();
// UserBuilder::default().name("Bob").build(); // ERROR: no build
```

---

## Pattern 3: `#[must_use]` for Critical Return Values

This attribute signals that a function's return value is important and should not be ignored. The compiler will issue a warning if a value marked `#[must_use]` is discarded.

-   **Problem**: Functions that return a `Result` or `Option` can have their return value silently ignored, leading to unhandled errors or logic bugs. Some types, like builders or transaction guards, are useless unless a final method (`.build()`, `.commit()`) is called.

-   **Solution**: Apply the `#[must_use]` attribute to functions or types. This instructs the compiler to generate a warning if a returned value of that type is not "used" in some way (e.g., assigned to a variable, passed to another function, or having a method called on it).

-   **Why It Matters**: Silent errors are insidious—code appears to work but fails in production. `#[must_use]` turns forgotten error handling into compiler warnings. This is why `Result` and iterators are `#[must_use]` by default in the standard library.

-   **Use cases**:
    -   `Result` and `Option` types are the canonical examples.
    -   Builder patterns, to ensure `.build()` is called.
    -   Resource handles that must be explicitly closed or released.
    -   Transaction guards that must be `.commit()`-ed or rolled back.
    -   Futures, which do nothing unless they are `.await`-ed.

### Example: Applying `#[must_use]` to Functions and Types

The standard library widely uses `#[must_use]`, for example on `Result` and `Option`. You can apply it to your own types and functions to guide users towards correct usage.

```rust
// Applying `#[must_use]` to a function's return value.
// A custom message explains why it's important.
#[must_use = "this Result may be an error to handle"]
pub fn connect_to_db() -> Result<(), &'static str> {
    Err("Failed to connect")
}

// Applying `#[must_use]` to a type definition.
// Any function returning this type will implicitly be `must_use`.
#[must_use = "a builder does nothing unless you call `.build()`"]
pub struct ConnectionBuilder;

impl ConnectionBuilder {
    pub fn new() -> Self { ConnectionBuilder }
    pub fn build(self) {}
}

// Usage: Compiler warns if #[must_use] return value is ignored.
// ConnectionBuilder::new(); // WARNING: unused builder
if let Err(e) = connect_to_db() { println!("Error: {}", e); }
ConnectionBuilder::new().build(); // Correct: return value is used
```
