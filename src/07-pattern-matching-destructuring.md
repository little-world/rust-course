# Pattern Matching & Destructuring

Pattern matching is one of Rust's most powerful features, enabling you to write clear, exhaustive, and efficient code for handling complex data structures. Unlike simple switch statements in other languages, Rust's pattern matching provides deep destructuring, guards, bindings, and compile-time exhaustiveness checking.

This chapter explores advanced pattern matching techniques that experienced programmers can leverage to build robust systems. The key insight is that pattern matching isn't just about control flow—it's a way to encode invariants, state transitions, and data transformations directly in your program's structure.

The patterns we'll explore include:
- Advanced match patterns with guards, bindings, and ranges
- Exhaustiveness checking and match ergonomics
- If-let chains and while-let for ergonomic control flow
- Pattern matching for state machines
- Enum-driven architecture patterns

### Pattern Matching Cheat Sheet

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

**Problem**: Simple if-else chains for numeric ranges become unwieldy (checking temperature ranges requires 8+ nested conditions). Extracting and testing values simultaneously requires separate steps (check status code, then extract it).

**Solution**: Use range patterns (`1..=10`) for concise numeric matching. Combine `@` bindings with guards to capture values while testing conditions (`x @ 100..=200 if expensive(x)`).

**Why It Matters**: Range patterns reduce 20 lines of if-else to a single clear match expression. The `@` binding eliminates temporary variables—capture and test in one step.

**Use Cases**: Numeric classification (temperature ranges, HTTP status codes, port numbers), token parsing (keywords, operators, literals), request routing (method + path combinations), user categorization (age/premium/activity), validation with capture (valid ranges that you need to use).

### Example 1: Range Matching

Range patterns provide a concise way to match against a range of values, which is far more readable than a long chain of `if-else` statements.

```rust
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
```

### Example 2: Guards for Complex Conditions

Match guards (`if ...`) allow you to add arbitrary boolean expressions to a pattern. This is useful when the condition for a match is more complex than a simple value comparison.

```rust
struct Response;
struct Error;

fn process_request(status: u16, body: &str) -> Result<Response, Error> {
    match (status, body.len()) {
        (200, len) if len > 0 => Ok(Response), // Guard checks the length
        (200, _) => Err(Error), // Handles 200 but with empty body
        (status @ 400..=499, _) => Err(Error), // Bind status and match range
        (status @ 500..=599, _) => Err(Error),
        _ => Err(Error),
    }
}
```

### Example 3: `@` Bindings to Capture and Test

The `@` symbol lets you bind a value to a variable while simultaneously testing it against a more complex pattern. This avoids the need to re-bind the variable inside the match arm.

```rust
enum Port { WellKnown(u16), Registered(u16), Dynamic(u16) }
struct ValidationError;

fn validate_port(port: u16) -> Result<Port, ValidationError> {
    match port {
        p @ 1..=1023 => Ok(Port::WellKnown(p)), // Bind to `p` and test range
        p @ 1024..=49151 => Ok(Port::Registered(p)),
        p @ 49152..=65535 => Ok(Port::Dynamic(p)),
        0 => Err(ValidationError),
    }
}
```

### Example 4: Nested Destructuring

Pattern matching allows you to "look inside" nested data structures, binding to only the fields you care about and ignoring the rest with `..`.

```rust
struct Point { x: i32, y: i32 }
enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, bottom_right: Point },
}

fn contains_origin(shape: &Shape) -> bool {
    match shape {
        // Destructure directly to the inner `x` and `y` fields.
        Shape::Circle { center: Point { x: 0, y: 0 }, .. } => true,
        Shape::Rectangle {
            top_left: Point { x: x1, y: y1 },
            bottom_right: Point { x: x2, y: y2 },
        // A guard can be used with destructured values.
        } if *x1 <= 0 && *x2 >= 0 && *y1 <= 0 && *y2 >= 0 => true,
        _ => false,
    }
}
```

## Pattern 2: Exhaustiveness and Match Ergonomics

**Problem**: In many languages, if you forget to handle a new enum variant in a `switch` statement, the program may compile but crash at runtime. Wildcard branches (`_` or `default`) can silently swallow new variants, leading to logical bugs.

**Solution**: Rust's `match` expressions are **exhaustive**, meaning the compiler guarantees that every possible case is handled. If you add a new variant to an enum, your code will not compile until you update all `match` expressions that use it.

**Why It Matters**: Exhaustiveness checking is one of Rust's most powerful safety features. It makes refactoring and evolving codebases dramatically safer.

**Use Cases**:
-   State machines where new states must be handled correctly everywhere.
-   Protocol implementations that need to support multiple versions.
-   Command parsers that must handle every possible command.
-   API error types, ensuring that callers handle all possible failure modes.

### Example 1: Exhaustive Matching for Safety

If we add a new variant to `RequestState`, the `state_duration` function will no longer compile until the new variant is handled in the `match` expression. This prevents us from forgetting to update critical logic.

```rust
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
        // If a new variant, e.g., `Throttled`, were added to `RequestState`,
        // this match would produce a compile-time error.
    }
}
```

### Example 2: Match Ergonomics

Rust's match ergonomics automatically handle references for you in most cases, reducing visual noise. When you match on a `&Option<String>`, the inner value is automatically a `&String`, not a `String`.

```rust
fn process_option(opt: &Option<String>) {
    match opt {
        // `s` is automatically `&String`, not `String`.
        Some(s) => println!("Got a string: {}", s),
        None => println!("Got nothing."),
    }
}
```

### Example 3: The `#[non_exhaustive]` Attribute

For public libraries, you may want to add new enum variants without it being a breaking change for your users. The `#[non_exhaustive]` attribute tells the compiler that this enum may have more variants in the future, forcing users to include a wildcard (`_`) arm in their `match` expressions.

```rust
#[non_exhaustive]
pub enum HttpVersion {
    Http11,
    Http2,
}

// A user of this library must include a wildcard match.
fn handle_version(version: HttpVersion) {
    match version {
        HttpVersion::Http11 => println!("Using HTTP/1.1"),
        HttpVersion::Http2 => println!("Using HTTP/2"),
        // This is required. If the library adds `Http3`, this code won't break.
        _ => println!("Using an unknown future version"),
    }
}
```

**Exhaustiveness principles:**
1. **Avoid wildcards in application code**: Forces you to handle new variants
2. **Use non_exhaustive for public library enums**: Allows adding variants without breaking changes
3. **Leverage compiler errors**: Let the compiler tell you what you forgot to handle
4. **Prefer match over if-let for complex enums**: Makes missing cases obvious
5. **Group similar cases carefully**: Don't hide distinct behavior behind wildcards
## Pattern 3: `if let`, `while let`, and `let-else`

**Problem**: A full `match` expression can be verbose if you only care about one or two cases. Nested `if-let` statements for a sequence of validations can lead to a "pyramid of doom" that is hard to read.

**Solution**: - Use `if let` to handle a single match case without the boilerplate of a full `match`. - Use `if let` chains (Rust 1.65+) to combine multiple patterns and conditions without nesting.

**Why It Matters**: These constructs make control flow more ergonomic and readable. `if let` chains flatten complex validation logic.

**Use Cases**:
-   Authentication flows (checking a header, parsing a token, validating claims).
-   Configuration parsing with layered fallbacks.
-   Processing items from a queue or stream until it is empty.
-   Navigating nested `Option` or `Result` types.

### Example 1: `if let` and `if let` chains

An `if let` chain can replace nested `if let` statements, making validation logic much flatter and easier to read.

```rust
struct Claims;
struct Token;
struct Request { authorization: Option<String> }
fn parse_token(auth: &str) -> Result<Token, ()> { Ok(Token) }
fn validate_token(token: &Token) -> Result<Claims, ()> { Ok(Claims) }

fn handle_request(req: &Request) {
    // Before if-let chains, this would be nested.
    if let Some(auth) = &req.authorization {
        if let Ok(token) = parse_token(auth) {
            if let Ok(claims) = validate_token(&token) {
                // ... process request with claims
            }
        }
    }
    
    // With if-let chains, it's much cleaner.
    if let Some(auth) = &req.authorization
        && let Ok(token) = parse_token(auth)
        && let Ok(claims) = validate_token(&token)
    {
        // ... process request with claims
    }
}
```

### Example 2: `let-else` for Early Returns

`let-else` is perfect for guard clauses at the beginning of a function. It allows you to destructure a value and `return` or `break` if the pattern doesn't match.

```rust
struct Claims { user_id: u64 }
enum Error { MissingAuth, InvalidToken }

fn get_user_id(request: &Request) -> Result<u64, Error> {
    let Some(auth_header) = &request.authorization else {
        return Err(Error::MissingAuth);
    };

    let Ok(claims) = validate_token(&Token) else {
        return Err(Error::InvalidToken);
    };

    Ok(claims.user_id)
}
```

### Example 3: `while let` for Iteration

`while let` is the idiomatic way to loop as long as a pattern continues to match. It's often used to process items from an iterator or a queue.

```rust
use std::collections::VecDeque;

// This loop continues as long as `pop_front` returns `Some(task)`.
fn drain_queue(queue: &mut VecDeque<String>) {
    while let Some(task) = queue.pop_front() {
        println!("Processing task: {}", task);
    }
}
```

**If-let and while-let guidelines:**
1. **Use if-let for single pattern**: More concise than match with one arm
2. **Avoid deep nesting**: Switch to match for complex cases
3. **Let-else for early returns**: Cleaner than if-let with nested code
4. **While-let for iterators**: Natural for consuming data structures
5. **If-let chains for validation sequences**: Replaces nested if-let
## Pattern 4: Enum-Driven Architecture
## Pattern 4: State Machines with Type-State Pattern

**Problem**: In many architectures, business logic is scattered across different services or classes. Adding a new operation can require hunting through the codebase to make updates.

**Solution**: Model the core operations, events, and states of your application as **enums with associated data**. Use `match` expressions to implement behavior in a centralized and exhaustive way.

**Why It Matters**: This architecture centralizes your business logic and leverages Rust's exhaustiveness checking for maintainability. When you add a new command, the compiler forces you to handle it in your `execute` function.

**Use Cases**:
-   CQRS (Command Query Responsibility Segregation) systems.
-   Event sourcing, where application state is derived from a sequence of events.
-   Message-passing systems, like actor models or service buses.
-   Defining explicit, type-safe API response structures.

### Example 1: Command Pattern with Enums

Instead of an object-oriented command pattern, you can define all possible operations as variants of a single `Command` enum. A central `execute` method then dispatches on the variant.

```rust
enum Command {
    CreateUser { username: String, email: String },
    DeleteUser { user_id: u64 },
}

fn execute_command(command: Command) {
    match command {
        Command::CreateUser { username, email } => {
            println!("Creating user {} with email {}", username, email);
        }
        Command::DeleteUser { user_id } => {
            println!("Deleting user {}", user_id);
        }
    }
}
```

### Example 2: Event Sourcing with Enums

In an event-sourcing system, state is rebuilt by applying a series of events. Using an enum for events ensures that every event is a well-defined, typed structure, and that your state aggregates can handle all of them.

```rust
// All possible events in the system are defined here.
enum UserEvent {
    UserRegistered { user_id: u64, username: String },
    EmailVerified { user_id: u64 },
}

struct User {
    id: u64,
    username: String,
    is_verified: bool,
}

impl User {
    // A user's state is built by applying events in order.
    fn from_events(events: &[UserEvent]) -> Self {
        let mut user = User { id: 0, username: "".into(), is_verified: false };
        for event in events {
            user.apply(event);
        }
        user
    }

    fn apply(&mut self, event: &UserEvent) {
        match event {
            UserEvent::UserRegistered { user_id, username } => {
                self.id = *user_id;
                self.username = username.clone();
            }
            UserEvent::EmailVerified { .. } => {
                self.is_verified = true;
            }
        }
    }
}
```

**Destructuring best practices:**
1. **Destructure in parameter position**: More concise than extracting in body
2. **Use .. to ignore irrelevant fields**: Makes intent clear
3. **Rename fields for context**: Use @ or field: name syntax
4. **Match slices with patterns**: Handle different lengths explicitly
5. **Be mindful of moves**: Use ref/&mut when needed to avoid consuming values

### Summary

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


