# Chapter 4: Pattern Matching & Destructuring

[Pattern 1: Advanced Match Patterns](#pattern-1-advanced-match-patterns)

- Problem: Unwieldy if-else chains, separate extract-and-test steps, code
  duplication
- Solution: Range patterns, @ bindings with guards, or-patterns, deep
  destructuring
- Why It Matters: 20 lines of if-else → single match; capture and test in
  one step
- Use Cases: Numeric classification, token parsing, request routing, user
  categorization

[Pattern 2: Exhaustiveness and Match Ergonomics](#pattern-2-exhaustiveness-and-match-ergonomics)

- Problem: Missing cases cause runtime crashes; wildcards hide bugs when
  adding variants
- Solution: Leverage exhaustiveness checking, avoid wildcards, use
  #[non_exhaustive]
- Why It Matters: Compile-time guarantee all cases handled; safe enum
  refactoring
- Use Cases: Evolving state machines, protocol implementations, command
  parsing

[Pattern 3: If-Let Chains and While-Let](#pattern-3-if-let-chains-and-while-let)

- Problem: Verbose single-pattern matches, nested validation creates
  pyramid of doom
- Solution: if let for extraction, if-let chains for sequences, let-else
  for early returns
- Why It Matters: 60% less boilerplate; eliminates rightward drift;
  natural validation flow
- Use Cases: Auth flows, config parsing, queue processing, stream parsing

[Pattern 4: State Machines with Type-State Pattern](#pattern-4-state-machines-with-type-state-pattern)

- Problem: Runtime state machines allow invalid method calls; docs don't
  enforce order
- Solution: Zero-sized state types, methods consume self, invalid
  transitions don't compile
- Why It Matters: Impossible to express invalid transitions; stronger than
  testing
- Use Cases: Network connections, file handles, transactions, builders,
  protocols

[Pattern 5: Enum-Driven Architecture](#pattern-5-enum-driven-architecture)

- Problem: OOP command patterns verbose; scattered business logic; untyped
  events
- Solution: Operations/events/states as enums; centralized behavior via
  match
- Why It Matters: Centralized exhaustive behavior; can't forget to handle
  cases; 10x less code
- Use Cases: CQRS, API design, message passing, workflows, parser ASTs

[Pattern 6: Destructuring in Practice](#pattern-6-destructuring-in-practice)

- Problem: Verbose field extraction, unclear intent, manual array
  indexing, ownership issues
- Solution: Destructure in bindings/parameters, .. to ignore, ref for
  borrowing
- Why It Matters: 20 lines → 2 lines; intent explicit; signature-level
  documentation
- Use Cases: Function params, nested data, iterators, ownership control,
  arrays

## Overview

Pattern matching is one of Rust's most powerful features, enabling you to write clear, exhaustive, and efficient code for handling complex data structures. Unlike simple switch statements in other languages, Rust's pattern matching provides deep destructuring, guards, bindings, and compile-time exhaustiveness checking.

This chapter explores advanced pattern matching techniques that experienced programmers can leverage to build robust systems. The key insight is that pattern matching isn't just about control flow—it's a way to encode invariants, state transitions, and data transformations directly in your program's structure.

The patterns we'll explore include:
- Advanced match patterns with guards, bindings, and ranges
- Exhaustiveness checking and match ergonomics
- If-let chains and while-let for ergonomic control flow
- Pattern matching for state machines
- Enum-driven architecture patterns

## Pattern Matching Foundation

```rust
// Basic patterns
match value {
    literal => { /* exact match */ }
    _ => { /* wildcard */ }
}

// Destructuring patterns
match tuple {
    (x, y) => { /* bind variables */ }
    (0, _) => { /* ignore parts */ }
}

// Enum patterns
match option {
    Some(x) => { /* extract value */ }
    None => { /* handle absence */ }
}

// Guards and bindings
match value {
    x if x > 0 => { /* conditional */ }
    x @ 1..=10 => { /* range with binding */ }
    _ => { /* default */ }
}

// Reference patterns
match &value {
    &x => { /* dereference */ }
    ref x => { /* create reference */ }
}

// Or patterns and multiple cases
match ch {
    'a' | 'e' | 'i' | 'o' | 'u' => { /* vowel */ }
    _ => { /* consonant */ }
}
```

## Pattern 1: Advanced Match Patterns

**Problem**: Simple if-else chains for numeric ranges become unwieldy (checking temperature ranges requires 8+ nested conditions). Extracting and testing values simultaneously requires separate steps (check status code, then extract it). Multiple similar conditions duplicate code. Complex boolean logic in guards becomes unreadable.

**Solution**: Use range patterns (`1..=10`) for concise numeric matching. Combine `@` bindings with guards to capture values while testing conditions (`x @ 100..=200 if expensive(x)`). Use or-patterns (`'a' | 'e' | 'i'`) to avoid duplication. Nest destructuring deeply to extract data directly in match arms.

**Why It Matters**: Range patterns reduce 20 lines of if-else to a single clear match expression. The `@` binding eliminates temporary variables—capture and test in one step. Guards let you incorporate arbitrary logic without sacrificing pattern matching's exhaustiveness checking. This makes complex classification logic (temperature ranges, HTTP status codes, user categories) both readable and provably complete.

**Use Cases**: Numeric classification (temperature ranges, HTTP status codes, port numbers), token parsing (keywords, operators, literals), request routing (method + path combinations), user categorization (age/premium/activity), validation with capture (valid ranges that you need to use).

### Examples

```rust
//====================================
// Pattern: Range matching with guards
//====================================
fn classify_temperature(temp: i32) -> &'static str {
    match temp {
        i32::MIN..=-40 => "extreme cold",
        -39..=-20 => "very cold",
        -19..=0 => "cold",
        1..=15 => "cool",
        16..=25 => "comfortable",
        26..=35 => "warm",
        36..=45 => "hot",
        46..=i32::MAX => "extreme heat",
    }
}

//=======================================
// Pattern: Guards for complex conditions
//=======================================
fn process_request(status: u16, body: &str) -> Result<Response, Error> {
    match (status, body.len()) {
        (200, len) if len > 0 => Ok(Response::Success(body.to_string())),
        (200, _) => Err(Error::EmptyResponse),
        (status @ 400..=499, _) => Err(Error::ClientError(status)),
        (status @ 500..=599, _) => Err(Error::ServerError(status)),
        (status, _) => Err(Error::UnknownStatus(status)),
    }
}

//==================================================
// Pattern: Binding with @ for capturing and testing
//==================================================
fn validate_port(port: u16) -> Result<Port, ValidationError> {
    match port {
        p @ 1..=1023 => Ok(Port::WellKnown(p)),
        p @ 1024..=49151 => Ok(Port::Registered(p)),
        p @ 49152..=65535 => Ok(Port::Dynamic(p)),
        0 => Err(ValidationError::InvalidPort),
    }
}

//===========================================
// Pattern: Multiple guards and complex logic
//===========================================
fn categorize_user(age: u8, is_premium: bool, posts: usize) -> UserCategory {
    match (age, is_premium, posts) {
        (_, true, p) if p > 100 => UserCategory::PowerUser,
        (a, true, _) if a >= 18 => UserCategory::PremiumAdult,
        (a, true, _) if a < 18 => UserCategory::PremiumYouth,
        (a, false, p) if a >= 18 && p > 50 => UserCategory::ActiveAdult,
        (a, false, p) if a < 18 && p > 50 => UserCategory::ActiveYouth,
        (a, false, _) if a >= 18 => UserCategory::RegularAdult,
        _ => UserCategory::RegularYouth,
    }
}

//==============================
// Pattern: Nested destructuring
//==============================
struct Point { x: i32, y: i32 }
enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, bottom_right: Point },
}

fn contains_origin(shape: &Shape) -> bool {
    match shape {
        Shape::Circle { center: Point { x: 0, y: 0 }, .. } => true,
        Shape::Circle { center, radius } => {
            let dist = ((center.x * center.x + center.y * center.y) as f64).sqrt();
            dist <= *radius
        }
        Shape::Rectangle {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        } if *x1 <= 0 && *x2 >= 0 && *y1 >= 0 && *y2 <= 0 => true,
        _ => false,
    }
}

//==================================
// Pattern: Or patterns (Rust 1.53+)
//==================================
fn is_delimiter(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r' | ',' | ';' | ':')
}

fn parse_token(input: &str) -> Token {
    match input {
        "true" | "TRUE" | "True" => Token::Bool(true),
        "false" | "FALSE" | "False" => Token::Bool(false),
        "null" | "NULL" | "None" => Token::Null,
        s if s.parse::<i64>().is_ok() => Token::Number(s.parse().unwrap()),
        s => Token::String(s.to_string()),
    }
}
```

**Guidelines for advanced patterns:**
1. **Use ranges for numeric matching**: More readable than multiple guards
2. **Combine @ with guards**: Capture the value while testing conditions
3. **Leverage or-patterns**: Avoid code duplication for similar cases
4. **Keep guards simple**: Complex logic should be extracted to functions
5. **Order matters**: Place specific cases before general ones

## Pattern 2: Exhaustiveness and Match Ergonomics

**Problem**: Missing cases in switch statements cause runtime errors in C/Java (forgot to handle new enum variant? Production crash). Wildcards (`_`) hide bugs when new variants are added—code compiles but silently handles new cases incorrectly. Reference handling requires manual dereferencing, cluttering code with `*` and `&`.

**Solution**: Leverage Rust's exhaustiveness checking—compiler errors if any enum variant is unhandled. Avoid wildcards in application code; list all variants explicitly so adding new ones breaks compilation at update sites. Use `#[non_exhaustive]` on public library enums to allow future additions. Let match ergonomics auto-dereference references (`&Option<T>` matches as `Option<&T>` automatically).

**Why It Matters**: Exhaustiveness checking catches bugs at compile time that would be production outages in other languages. When you add a new `DatabaseState` variant, the compiler forces you to update every match—no forgotten edge cases. This is transformative for evolving codebases: refactoring enums is safe because the compiler finds every place that needs updating. Match ergonomics eliminates 50% of reference-handling boilerplate.

**Use Cases**: State machines that evolve (adding states breaks compilation where needed), protocol implementations (version handling), command parsing (ensure all commands handled), API error types (exhaustive error handling), event systems (all events processed).

### Examples

```rust
//================================================
// Pattern: Non-exhaustive enums for extensibility
//================================================
#[non_exhaustive]
pub enum HttpVersion {
    Http10,
    Http11,
    Http2,
    Http3,
}

// Users must include wildcard to handle future variants
fn version_name(version: &HttpVersion) -> &str {
    match version {
        HttpVersion::Http10 => "HTTP/1.0",
        HttpVersion::Http11 => "HTTP/1.1",
        HttpVersion::Http2 => "HTTP/2.0",
        HttpVersion::Http3 => "HTTP/3.0",
        // Required for non_exhaustive enums from other crates
        _ => "Unknown",
    }
}

//========================================
// Pattern: Exhaustive matching for safety
//========================================
enum RequestState {
    Pending,
    InProgress { started_at: u64 },
    Completed { result: String },
    Failed { error: String },
}

fn state_duration(state: &RequestState, now: u64) -> Option<u64> {
    match state {
        RequestState::Pending => None,
        RequestState::InProgress { started_at } => Some(now - started_at),
        RequestState::Completed { .. } => None,
        RequestState::Failed { .. } => None,
        // Compiler error if new variant is added without handling it here
    }
}

//====================================================
// Pattern: Match ergonomics (automatic dereferencing)
//====================================================
fn process_option(opt: &Option<String>) {
    match opt {
        // Automatically dereferences &Option<String> to Option<&String>
        Some(s) => println!("Got: {}", s),
        None => println!("Nothing"),
    }
}

fn process_result(res: &Result<Vec<u8>, String>) {
    match res {
        // Match ergonomics automatically handles references
        Ok(bytes) => println!("Success: {} bytes", bytes.len()),
        Err(e) => println!("Error: {}", e),
    }
}

//================================
// Pattern: Forcing move vs borrow
//================================
struct Resource(String);

fn handle_resource(res: Option<Resource>) {
    match res {
        Some(Resource(s)) => {
            // Moves out of option
            println!("Took ownership: {}", s);
        }
        None => {}
    }
    // res is now moved
}

fn handle_resource_ref(res: &Option<Resource>) {
    match res {
        Some(Resource(ref s)) => {
            // Borrows instead of moving
            println!("Borrowed: {}", s);
        }
        None => {}
    }
    // res is still valid
}

//========================================================
// Pattern: Compile-time exhaustiveness for critical logic
//========================================================
#[derive(Debug)]
enum DatabaseState {
    Disconnected,
    Connecting,
    Connected,
    QueryInProgress,
    Error,
}

impl DatabaseState {
    fn can_query(&self) -> bool {
        match self {
            DatabaseState::Connected => true,
            DatabaseState::Disconnected => false,
            DatabaseState::Connecting => false,
            DatabaseState::QueryInProgress => false,
            DatabaseState::Error => false,
            // Adding new state without updating this will cause compile error
        }
    }
}
```

**Exhaustiveness principles:**
1. **Avoid wildcards in application code**: Forces you to handle new variants
2. **Use non_exhaustive for public library enums**: Allows adding variants without breaking changes
3. **Leverage compiler errors**: Let the compiler tell you what you forgot to handle
4. **Prefer match over if-let for complex enums**: Makes missing cases obvious
5. **Group similar cases carefully**: Don't hide distinct behavior behind wildcards

## Pattern 3: If-Let Chains and While-Let

**Problem**: Full `match` expressions for single-pattern checks are verbose (matching `Some` requires 5 lines when you only care about the success case). Nested if-let for validation sequences creates rightward drift (3-4 levels deep). Early returns with `match` require awkward pattern: match then return in `None` arm. Consuming iterators with `while` + pattern match is boilerplate-heavy.

**Solution**: Use `if let` for single-pattern extraction without else cases. Use if-let chains (Rust 1.65+) to combine multiple conditions without nesting (`if let Some(x) = opt && x > 0`). Use let-else for early returns with inverted logic (`let Some(x) = opt else { return }`). Use `while let` to consume iterators or stateful types until exhaustion.

**Why It Matters**: If-let reduces boilerplate by 60% compared to match for simple checks. If-let chains eliminate the "pyramid of doom" from nested validation—5 levels of indentation become one line. Let-else makes validation sequences read naturally: "ensure this condition, otherwise bail". While-let is the idiomatic way to drain queues, process streams, and implement state machines. These constructs make Rust feel high-level without sacrificing safety.

**Use Cases**: Authentication flows (check header, extract token, validate claims), configuration parsing (layered validation), queue processing (drain until empty), stream parsing (read until EOF), optional chaining (navigate nested Options/Results), guard clauses in functions.

### Examples

```rust
//========================================
// Pattern: If-let for optional extraction
//========================================
fn process_config(config: Option<Config>) {
    if let Some(cfg) = config {
        println!("Using config: {:?}", cfg);
        cfg.apply();
    } else {
        println!("No config provided, using defaults");
    }
}

//====================================
// Pattern: If-let chains (Rust 1.65+)
//====================================
fn handle_request(req: &Request) -> Response {
    if let Some(auth) = &req.headers.authorization
        && let Ok(token) = parse_token(auth)
        && let Ok(claims) = validate_token(&token)
    {
        Response::Success(claims)
    } else {
        Response::Unauthorized
    }
}

//====================================
// Pattern: Let-else for early returns
//====================================
fn get_user_id(request: &Request) -> Result<UserId, Error> {
    let Some(auth_header) = request.headers.get("Authorization") else {
        return Err(Error::MissingAuth);
    };
    
    let Some(token) = auth_header.strip_prefix("Bearer ") else {
        return Err(Error::InvalidAuthFormat);
    };
    
    let Ok(claims) = decode_jwt(token) else {
        return Err(Error::InvalidToken);
    };
    
    Ok(claims.user_id)
}

//=================================
// Pattern: While-let for iteration
//=================================
fn drain_queue(queue: &mut VecDeque<Task>) {
    while let Some(task) = queue.pop_front() {
        task.execute();
    }
}

//======================================
// Pattern: While-let for state machines
//======================================
fn process_stream(stream: &mut ByteStream) -> Vec<Message> {
    let mut messages = Vec::new();
    
    while let Some(msg) = read_message(stream) {
        match msg {
            Message::Data(d) => messages.push(Message::Data(d)),
            Message::End => break,
            Message::Error(e) => {
                eprintln!("Error: {}", e);
                continue;
            }
        }
    }
    
    messages
}

//================================================
// Pattern: Combining if-let with other conditions
//================================================
fn should_process(item: Option<Item>, force: bool) -> bool {
    if force {
        return true;
    }
    
    if let Some(item) = item {
        item.is_valid() && item.priority() > 5
    } else {
        false
    }
}

//==============================================
// Pattern: Nested if-let for complex extraction
//==============================================
fn extract_nested(data: Option<Result<Vec<String>, Error>>) {
    if let Some(result) = data {
        if let Ok(items) = result {
            for item in items {
                println!("{}", item);
            }
        } else {
            eprintln!("Error in result");
        }
    }
}

//==========================================
// Better: Use match for deeply nested cases
//==========================================
fn extract_nested_better(data: Option<Result<Vec<String>, Error>>) {
    match data {
        Some(Ok(items)) => {
            for item in items {
                println!("{}", item);
            }
        }
        Some(Err(_)) => eprintln!("Error in result"),
        None => {}
    }
}
```

**If-let and while-let guidelines:**
1. **Use if-let for single pattern**: More concise than match with one arm
2. **Avoid deep nesting**: Switch to match for complex cases
3. **Let-else for early returns**: Cleaner than if-let with nested code
4. **While-let for iterators**: Natural for consuming data structures
5. **If-let chains for validation sequences**: Replaces nested if-let

## Pattern 4: State Machines with Type-State Pattern

**Problem**: Runtime state machines (enum-based) allow calling methods in wrong states—`connection.send()` when disconnected compiles but fails at runtime. Documentation says "call connect() before send()" but nothing enforces it. State transition bugs cause security issues (sending unencrypted data), data corruption (writing to closed files), and crashes (using released resources).

**Solution**: Encode states as zero-sized types and parameterize structs by state (`Connection<Disconnected>` vs `Connection<Connected>`). Methods that change state consume `self` and return new state type (`fn connect(self) -> Connection<Connected>`). Invalid transitions don't compile—`Connection<Connected>::connect()` doesn't exist. Use PhantomData to track state at zero runtime cost.

**Why It Matters**: Type-state pattern makes invalid state transitions impossible to express, not just incorrect. You cannot call `send()` on a disconnected connection—the method simply doesn't exist for that type. Builders can enforce "URL is required" at compile time by making `build()` only available after `url()` is called. This is stronger than any amount of testing: if it compiles, state transitions are valid. The typestate pattern has caught real bugs in TLS implementations and database libraries.

**Use Cases**: Network connections (TCP state machine, TLS handshake), file handles (open/closed states), database transactions (begin/commit/rollback), builder patterns (required fields), protocol implementations (HTTP request lifecycle), resource lifecycle (allocated/initialized/released).

### Examples

```rust
//=====================================================
// Pattern: Type-state pattern for connection lifecycle
//=====================================================
use std::marker::PhantomData;

// State types (zero-sized)
struct Disconnected;
struct Connecting;
struct Connected;
struct Closed;

// Connection parameterized by state
struct Connection<State> {
    socket: TcpStream,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new(socket: TcpStream) -> Self {
        Connection {
            socket,
            _state: PhantomData,
        }
    }
    
    // Only available in Disconnected state
    fn connect(self, addr: &str) -> Result<Connection<Connecting>, Error> {
        // Initiate connection
        Ok(Connection {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl Connection<Connecting> {
    // Only available in Connecting state
    fn wait(self) -> Result<Connection<Connected>, Error> {
        // Wait for connection to establish
        Ok(Connection {
            socket: self.socket,
            _state: PhantomData,
        })
    }
    
    fn abort(self) -> Connection<Closed> {
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

impl Connection<Connected> {
    // Only available in Connected state
    fn send(&mut self, data: &[u8]) -> Result<(), Error> {
        // Send data
        Ok(())
    }
    
    fn receive(&mut self) -> Result<Vec<u8>, Error> {
        // Receive data
        Ok(vec![])
    }
    
    fn close(self) -> Connection<Closed> {
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

//===============================================
// Usage - invalid transitions are compile errors
//===============================================
fn use_connection() -> Result<(), Error> {
    let conn = Connection::new(create_socket()?);
    let conn = conn.connect("example.com:80")?;
    let mut conn = conn.wait()?;
    
    conn.send(b"GET / HTTP/1.1\r\n")?;
    let response = conn.receive()?;
    
    // conn.connect(); // Compile error! Can't connect when already connected
    
    let conn = conn.close();
    // conn.send(b"data"); // Compile error! Can't send when closed
    
    Ok(())
}

//============================================
// Pattern: Enum-based state machine (runtime)
//============================================
enum ConnectionState {
    Disconnected,
    Connecting { started_at: u64 },
    Connected { socket: TcpStream },
    Closed { reason: String },
}

struct ConnectionStateMachine {
    state: ConnectionState,
}

impl ConnectionStateMachine {
    fn new() -> Self {
        ConnectionStateMachine {
            state: ConnectionState::Disconnected,
        }
    }
    
    fn connect(&mut self, addr: &str) -> Result<(), Error> {
        match &self.state {
            ConnectionState::Disconnected => {
                // Start connection
                self.state = ConnectionState::Connecting {
                    started_at: current_time(),
                };
                Ok(())
            }
            ConnectionState::Connecting { .. } => {
                Err(Error::AlreadyConnecting)
            }
            ConnectionState::Connected { .. } => {
                Err(Error::AlreadyConnected)
            }
            ConnectionState::Closed { .. } => {
                Err(Error::Closed)
            }
        }
    }
    
    fn finalize(&mut self, socket: TcpStream) -> Result<(), Error> {
        match &self.state {
            ConnectionState::Connecting { .. } => {
                self.state = ConnectionState::Connected { socket };
                Ok(())
            }
            _ => Err(Error::InvalidStateTransition),
        }
    }
    
    fn close(&mut self, reason: String) {
        self.state = ConnectionState::Closed { reason };
    }
}

//================================================
// Pattern: Builder pattern with state transitions
//================================================
struct RequestBuilder<State> {
    url: Option<String>,
    method: Option<String>,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
    _state: PhantomData<State>,
}

struct NoUrl;
struct HasUrl;
struct Ready;

impl RequestBuilder<NoUrl> {
    fn new() -> Self {
        RequestBuilder {
            url: None,
            method: None,
            headers: Vec::new(),
            body: None,
            _state: PhantomData,
        }
    }
    
    fn url(self, url: impl Into<String>) -> RequestBuilder<HasUrl> {
        RequestBuilder {
            url: Some(url.into()),
            method: self.method,
            headers: self.headers,
            body: self.body,
            _state: PhantomData,
        }
    }
}

impl RequestBuilder<HasUrl> {
    fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }
    
    fn header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }
    
    fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }
    
    fn build(self) -> Request {
        Request {
            url: self.url.unwrap(),
            method: self.method.unwrap_or_else(|| "GET".to_string()),
            headers: self.headers,
            body: self.body,
        }
    }
}
```

**State machine patterns:**
1. **Type-state for compile-time safety**: Use when states are known at compile time
2. **Enum-based for runtime flexibility**: Use when states are determined at runtime
3. **Consume self for transitions**: Prevents use after state change
4. **PhantomData for zero-cost**: State types don't affect runtime size
5. **Builder pattern with states**: Enforce construction order at compile time

## Pattern 5: Enum-Driven Architecture

**Problem**: Object-oriented command patterns require classes, inheritance, and dynamic dispatch for each operation. API responses mix success/error states in ways that force runtime checking. Event sourcing with string-based events loses type safety. Message passing with untyped channels causes deserialization errors. Business logic scattered across services makes behavior changes require hunting through code.

**Solution**: Model operations, events, and states as enums with associated data. Use match expressions to implement behavior—all cases in one place. Commands become `enum Command { Create {...}, Update {...} }` with centralized `execute()`. Events are typed enums that aggregate `apply()` to rebuild state. API responses use enums for explicit success/partial/error variants. Message channels carry typed enum payloads.

**Why It Matters**: Enum-driven architecture centralizes behavior and makes it exhaustive. Adding a new command means adding one enum variant—the match expression in `execute()` fails to compile until you handle it. This is transformative for maintainability: you can't forget to handle a command because the compiler forces it. Event sourcing with typed enums prevents "replay failed because event format changed". Command pattern without OOP boilerplate is 10x less code. Enum dispatch is faster than vtables.

**Use Cases**: CQRS systems (commands and events as enums), API design (explicit response variants), message-passing systems (typed message enums), workflow engines (pipeline steps as enums), parser ASTs (expression trees), protocol state machines (request/response variants).

### Examples

```rust
//====================================
// Pattern: Command pattern with enums
//====================================
#[derive(Debug, Clone)]
enum Command {
    CreateUser { username: String, email: String },
    DeleteUser { user_id: u64 },
    UpdateEmail { user_id: u64, new_email: String },
    ChangePassword { user_id: u64, old_pass: String, new_pass: String },
    ListUsers { page: usize, page_size: usize },
}

impl Command {
    fn execute(&self, db: &mut Database) -> Result<CommandResult, Error> {
        match self {
            Command::CreateUser { username, email } => {
                let user_id = db.create_user(username, email)?;
                Ok(CommandResult::UserCreated { user_id })
            }
            Command::DeleteUser { user_id } => {
                db.delete_user(*user_id)?;
                Ok(CommandResult::UserDeleted)
            }
            Command::UpdateEmail { user_id, new_email } => {
                db.update_email(*user_id, new_email)?;
                Ok(CommandResult::EmailUpdated)
            }
            Command::ChangePassword { user_id, old_pass, new_pass } => {
                db.change_password(*user_id, old_pass, new_pass)?;
                Ok(CommandResult::PasswordChanged)
            }
            Command::ListUsers { page, page_size } => {
                let users = db.list_users(*page, *page_size)?;
                Ok(CommandResult::UserList { users })
            }
        }
    }
    
    fn requires_auth(&self) -> bool {
        match self {
            Command::ListUsers { .. } => false,
            _ => true,
        }
    }
    
    fn audit_log(&self) -> String {
        match self {
            Command::CreateUser { username, .. } => {
                format!("Created user: {}", username)
            }
            Command::DeleteUser { user_id } => {
                format!("Deleted user: {}", user_id)
            }
            Command::UpdateEmail { user_id, new_email } => {
                format!("Updated email for user {}: {}", user_id, new_email)
            }
            Command::ChangePassword { user_id, .. } => {
                format!("Changed password for user {}", user_id)
            }
            Command::ListUsers { .. } => {
                "Listed users".to_string()
            }
        }
    }
}

//===================================
// Pattern: Event sourcing with enums
//===================================
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Event {
    UserRegistered { user_id: u64, username: String, timestamp: u64 },
    EmailVerified { user_id: u64, timestamp: u64 },
    PasswordChanged { user_id: u64, timestamp: u64 },
    AccountLocked { user_id: u64, reason: String, timestamp: u64 },
    AccountUnlocked { user_id: u64, timestamp: u64 },
}

struct UserAggregate {
    id: u64,
    username: String,
    email_verified: bool,
    locked: bool,
    version: u64,
}

impl UserAggregate {
    fn apply(&mut self, event: &Event) {
        match event {
            Event::UserRegistered { user_id, username, .. } => {
                self.id = *user_id;
                self.username = username.clone();
                self.email_verified = false;
                self.locked = false;
            }
            Event::EmailVerified { .. } => {
                self.email_verified = true;
            }
            Event::PasswordChanged { .. } => {
                // Update internal state if needed
            }
            Event::AccountLocked { .. } => {
                self.locked = true;
            }
            Event::AccountUnlocked { .. } => {
                self.locked = false;
            }
        }
        self.version += 1;
    }
    
    fn from_events(events: &[Event]) -> Self {
        let mut aggregate = UserAggregate {
            id: 0,
            username: String::new(),
            email_verified: false,
            locked: false,
            version: 0,
        };
        
        for event in events {
            aggregate.apply(event);
        }
        
        aggregate
    }
}

//=============================================
// Pattern: Message passing with typed channels
//=============================================
enum WorkerMessage {
    Process { data: Vec<u8>, reply_to: Sender<Result<Vec<u8>, Error>> },
    Shutdown,
    GetStatus { reply_to: Sender<WorkerStatus> },
}

enum WorkerStatus {
    Idle,
    Processing { task_id: u64 },
    ShuttingDown,
}

fn worker_thread(rx: Receiver<WorkerMessage>) {
    let mut status = WorkerStatus::Idle;
    
    loop {
        match rx.recv() {
            Ok(WorkerMessage::Process { data, reply_to }) => {
                status = WorkerStatus::Processing { task_id: generate_id() };
                let result = process_data(&data);
                let _ = reply_to.send(result);
                status = WorkerStatus::Idle;
            }
            Ok(WorkerMessage::Shutdown) => {
                status = WorkerStatus::ShuttingDown;
                break;
            }
            Ok(WorkerMessage::GetStatus { reply_to }) => {
                let _ = reply_to.send(status.clone());
            }
            Err(_) => break,
        }
    }
}

//===========================================
// Pattern: Parse result with detailed errors
//===========================================
#[derive(Debug)]
enum ParseResult<T> {
    Success { value: T, remaining: String },
    Incomplete { needed: usize },
    Error { message: String, position: usize, context: String },
}

fn parse_json(input: &str) -> ParseResult<JsonValue> {
    match try_parse_json(input) {
        Ok((value, rest)) => ParseResult::Success {
            value,
            remaining: rest.to_string(),
        },
        Err(ParseError::Incomplete { needed }) => {
            ParseResult::Incomplete { needed }
        }
        Err(ParseError::Invalid { pos, msg }) => {
            let context = get_context(input, pos);
            ParseResult::Error {
                message: msg,
                position: pos,
                context,
            }
        }
    }
}

//=============================================
// Pattern: API response with structured errors
//=============================================
#[derive(Debug, Serialize)]
#[serde(tag = "status")]
enum ApiResponse<T> {
    #[serde(rename = "success")]
    Success {
        data: T,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<ResponseMetadata>,
    },
    
    #[serde(rename = "error")]
    Error {
        code: String,
        message: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        details: Vec<ErrorDetail>,
    },
    
    #[serde(rename = "partial")]
    Partial {
        data: T,
        warnings: Vec<String>,
    },
}

impl<T: Serialize> ApiResponse<T> {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    fn status_code(&self) -> u16 {
        match self {
            ApiResponse::Success { .. } => 200,
            ApiResponse::Error { code, .. } if code.starts_with("AUTH") => 401,
            ApiResponse::Error { code, .. } if code.starts_with("FORBIDDEN") => 403,
            ApiResponse::Error { code, .. } if code.starts_with("NOT_FOUND") => 404,
            ApiResponse::Error { .. } => 400,
            ApiResponse::Partial { .. } => 206,
        }
    }
}

//======================================
// Pattern: Workflow with explicit steps
//======================================
enum WorkflowStep {
    Validate { input: InputData },
    Transform { validated: ValidatedData },
    Persist { transformed: TransformedData },
    Notify { persisted: PersistedData },
    Complete { result: WorkflowResult },
}

struct Workflow {
    step: WorkflowStep,
}

impl Workflow {
    fn new(input: InputData) -> Self {
        Workflow {
            step: WorkflowStep::Validate { input },
        }
    }
    
    fn advance(&mut self) -> Result<(), Error> {
        self.step = match std::mem::replace(&mut self.step, WorkflowStep::Complete { 
            result: WorkflowResult::default() 
        }) {
            WorkflowStep::Validate { input } => {
                let validated = validate(input)?;
                WorkflowStep::Transform { validated }
            }
            WorkflowStep::Transform { validated } => {
                let transformed = transform(validated)?;
                WorkflowStep::Persist { transformed }
            }
            WorkflowStep::Persist { transformed } => {
                let persisted = persist(transformed)?;
                WorkflowStep::Notify { persisted }
            }
            WorkflowStep::Notify { persisted } => {
                let result = notify(persisted)?;
                WorkflowStep::Complete { result }
            }
            WorkflowStep::Complete { result } => {
                return Ok(());
            }
        };
        
        Ok(())
    }
    
    fn run(mut self) -> Result<WorkflowResult, Error> {
        loop {
            match &self.step {
                WorkflowStep::Complete { result } => {
                    return Ok(result.clone());
                }
                _ => self.advance()?,
            }
        }
    }
}
```

**Enum-driven architecture principles:**
1. **Make illegal states unrepresentable**: Use enum variants to encode valid states
2. **Encode business logic in types**: Match expressions document behavior
3. **Exhaustiveness catches bugs**: Adding enum variants forces updates everywhere
4. **Use enums for command/event patterns**: Natural fit for CQRS and event sourcing
5. **Combine with pattern matching**: Extract data and route logic simultaneously

## Pattern 6: Destructuring in Practice

**Problem**: Extracting fields from nested structures requires verbose intermediate variables (`let x = point.x; let y = point.y;`). Renaming for clarity creates extra bindings. Ignoring irrelevant fields isn't explicit. Array pattern matching requires manual indexing and length checks. Ownership issues arise when destructuring moves values you meant to borrow.

**Solution**: Destructure directly in let bindings, function parameters, and match arms. Use `..` to explicitly ignore remaining fields. Rename during destructure (`User { id: user_id, .. }`). Match array patterns with `[first, middle @ .., last]` syntax. Control ownership with `ref`/`ref mut` patterns to borrow instead of move. Combine destructuring with guards for conditional extraction.

**Why It Matters**: Destructuring eliminates intermediate variables—20 lines of field extraction become 2 lines. It makes intent explicit: `let User { age, .. }` says "I only care about age". Function parameters that destructure (`fn format((x, y): (i32, i32))`) document what data is used at the signature level. Slice destructuring with `@..` enables head/tail operations without manual indexing. This is about writing code that reads like specifications.

**Use Cases**: Function parameters (destructure tuples/structs inline), nested data extraction (JSON parsing, config objects), iterator processing (for loops with tuple destructuring), ownership control (ref patterns prevent moves), array/slice operations (head/tail, pattern length checking), expression trees (recursive destructuring in parsers).

### Examples

```rust
//============================================
// Pattern: Struct destructuring with renaming
//============================================
struct User {
    id: u64,
    name: String,
    email: String,
    age: u8,
}

fn process_user(user: &User) {
    // Rename fields for clarity
    let User { 
        id: user_id,
        name: user_name,
        email: contact_email,
        age,
    } = user;
    
    println!("User {} ({}): {} - age {}", user_id, user_name, contact_email, age);
}

//===============================
// Pattern: Ignore fields with ..
//===============================
fn is_adult(user: &User) -> bool {
    let User { age, .. } = user;
    *age >= 18
}

//=====================================================
// Pattern: Nested destructuring in function parameters
//=====================================================
fn format_location((x, y): (f64, f64)) -> String {
    format!("({:.2}, {:.2})", x, y)
}

fn process_result(Ok(value) | Err(value): Result<i32, i32>) -> i32 {
    value.abs()
}

//=======================================
// Pattern: Array and slice destructuring
//=======================================
fn process_coords(coords: &[i32]) {
    match coords {
        [] => println!("Empty"),
        [x] => println!("Single point: {}", x),
        [x, y] => println!("2D point: ({}, {})", x, y),
        [x, y, z] => println!("3D point: ({}, {}, {})", x, y, z),
        [first, middle @ .., last] => {
            println!("First: {}, Last: {}, Middle: {:?}", first, last, middle);
        }
    }
}

//====================================
// Pattern: Complex enum destructuring
//====================================
enum Expression {
    Literal(i64),
    Variable(String),
    BinaryOp { op: String, left: Box<Expression>, right: Box<Expression> },
    Call { func: String, args: Vec<Expression> },
}

fn evaluate(expr: &Expression, vars: &HashMap<String, i64>) -> Result<i64, String> {
    match expr {
        Expression::Literal(n) => Ok(*n),
        
        Expression::Variable(name) => {
            vars.get(name)
                .copied()
                .ok_or_else(|| format!("Undefined variable: {}", name))
        }
        
        Expression::BinaryOp { op, left, right } => {
            let l = evaluate(left, vars)?;
            let r = evaluate(right, vars)?;
            
            match op.as_str() {
                "+" => Ok(l + r),
                "-" => Ok(l - r),
                "*" => Ok(l * r),
                "/" => Ok(l / r),
                _ => Err(format!("Unknown operator: {}", op)),
            }
        }
        
        Expression::Call { func, args } if func == "max" && args.len() == 2 => {
            let vals: Result<Vec<_>, _> = args.iter()
                .map(|e| evaluate(e, vars))
                .collect();
            Ok(vals?.into_iter().max().unwrap())
        }
        
        Expression::Call { func, .. } => {
            Err(format!("Unknown function: {}", func))
        }
    }
}

//====================================
// Pattern: Destructuring in for loops
//====================================
fn process_entries(entries: Vec<(String, i32, bool)>) {
    for (name, count, active) in entries {
        if active {
            println!("{}: {}", name, count);
        }
    }
}

//======================================
// Pattern: Destructuring with ownership
//======================================
struct Container {
    data: Vec<u8>,
    metadata: String,
}

fn take_data(container: Container) -> Vec<u8> {
    let Container { data, metadata: _ } = container;
    // Takes ownership of data, drops metadata
    data
}

fn borrow_data(container: &Container) -> usize {
    let Container { data, .. } = container;
    data.len()
}

//============================================
// Pattern: Matching references to avoid moves
//============================================
fn process_option(opt: &Option<String>) -> usize {
    match opt {
        Some(s) => s.len(),
        None => 0,
    }
}

fn process_option_mut(opt: &mut Option<String>) {
    match opt {
        Some(s) => s.push_str(" (modified)"),
        None => *opt = Some("default".to_string()),
    }
}

//=======================================
// Pattern: Destructuring in let bindings
//=======================================
fn parse_header(header: &str) -> Result<(String, String), Error> {
    let parts: Vec<_> = header.splitn(2, ':').collect();
    
    let [key, value] = parts.as_slice() else {
        return Err(Error::InvalidHeader);
    };
    
    Ok((key.trim().to_string(), value.trim().to_string()))
}

//=========================================
// Pattern: Match guards with destructuring
//=========================================
fn classify_point((x, y): (i32, i32)) -> &'static str {
    match (x, y) {
        (0, 0) => "origin",
        (x, 0) if x > 0 => "positive x-axis",
        (x, 0) if x < 0 => "negative x-axis",
        (0, y) if y > 0 => "positive y-axis",
        (0, y) if y < 0 => "negative y-axis",
        (x, y) if x > 0 && y > 0 => "quadrant I",
        (x, y) if x < 0 && y > 0 => "quadrant II",
        (x, y) if x < 0 && y < 0 => "quadrant III",
        _ => "quadrant IV",
    }
}
```

**Destructuring best practices:**
1. **Destructure in parameter position**: More concise than extracting in body
2. **Use .. to ignore irrelevant fields**: Makes intent clear
3. **Rename fields for context**: Use @ or field: name syntax
4. **Match slices with patterns**: Handle different lengths explicitly
5. **Be mindful of moves**: Use ref/&mut when needed to avoid consuming values

## Summary

Pattern matching in Rust is a powerful tool that goes beyond simple control flow. By leveraging advanced patterns, exhaustiveness checking, state machines, and enum-driven architecture, you can:

- **Encode invariants at compile time**: Make invalid states unrepresentable
- **Eliminate entire classes of bugs**: Exhaustiveness prevents missing cases
- **Express complex logic clearly**: Pattern matching documents behavior
- **Build robust state machines**: Type-state pattern enforces valid transitions
- **Design better APIs**: Enums make interfaces explicit and type-safe

**Key takeaways:**
1. Use range patterns and guards for expressive numeric matching
2. Leverage exhaustiveness checking—avoid wildcards in application code
3. Apply if-let chains and let-else for ergonomic validation sequences
4. Encode state machines in types to prevent invalid transitions
5. Design subsystems around enums to make illegal states unrepresentable
6. Destructure deeply to extract exactly what you need

Pattern matching isn't just a language feature—it's a design philosophy that shapes how you model domains, handle errors, and structure programs. Master these patterns to write Rust code that is not only safe and fast, but also clear and maintainable.
```

