
# Type-Safe Configuration System

### Problem Statement

Build a configuration system for a web server that uses newtype wrappers to prevent common configuration errors. You'll start with basic structs, then add type safety through newtypes, validated types, and finally a fluent builder API.

### Use Cases

**When you need this pattern**:
1. **Server configuration**: Ports, hostnames, URLs, timeouts - prevent mixing
2. **Database configuration**: Connection strings, pool sizes, credentials
3. **API clients**: Endpoints, API keys, rate limits, retry policies
4. **File paths**: Config vs data vs cache paths - type-safe separation
5. **Resource limits**: Memory limits, CPU limits, connection limits - enforce positivity
6. **Credentials**: Username, password, API tokens - hide in Debug output

---

## Understanding Type-Safe Configuration

Before implementing the configuration system, let's understand the fundamental concepts of type safety, newtypes, and builder patterns that make configuration robust and maintainable.

### What is the Newtype Pattern?

The **newtype pattern** creates a new type that wraps an existing type, giving it a distinct identity in the type system. It's called "newtype" because you create a new type with the same representation as an existing type.

**Basic Syntax**:
```rust
struct Port(u16);  // Port is a NEW type, distinct from u16
```

Even though `Port` contains a `u16`, Rust treats them as **completely different types**.

**The Problem: Primitive Obsession**:
```rust
fn start_server(host: String, port: u16, timeout: u64) {
    // What units is timeout? Seconds? Milliseconds?
    // What if someone passes port as timeout?
    // All we know is: String, u16, u64 (too generic!)
}

// Bugs waiting to happen:
start_server("localhost".to_string(), 30, 8080);  // Swapped port and timeout!
start_server("8080".to_string(), 8080, 30);       // Passed port as host!
```

**Solution with Newtypes**:
```rust
struct Hostname(String);
struct Port(u16);
struct Timeout(Duration);

fn start_server(host: Hostname, port: Port, timeout: Timeout) {
    // Types make it IMPOSSIBLE to swap parameters!
}

// These won't compile:
start_server(Port(8080), Hostname("localhost".into()), ...);  // ❌ Wrong order
start_server(Hostname("8080".into()), Port(30), ...);         // ❌ Nonsensical values
```

---

### Why Newtypes? The Benefits

**1. Type Safety at Compile Time**

```rust
struct Meters(f64);
struct Feet(f64);

fn build_bridge(length: Meters) { }

let length_meters = Meters(100.0);
let length_feet = Feet(328.0);

build_bridge(length_meters);  // ✓ OK
build_bridge(length_feet);    // ❌ Compile error!
```

**Real disaster**: Mars Climate Orbiter (1999) - $327 million spacecraft lost because one team used metric units, another used imperial units. A newtype could have prevented this!

**2. Self-Documenting Code**

```rust
// Without newtypes - confusing
fn configure(a: u16, b: u16, c: u16) -> Result<(), String> {
    // Which is port? Which is timeout? Which is max connections?
}

// With newtypes - crystal clear
fn configure(
    port: Port,
    timeout_ms: TimeoutMillis,
    max_conn: MaxConnections
) -> Result<(), String> {
    // Intent is immediately obvious!
}
```

**3. Encapsulation and Validation**

```rust
struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Result<Self, String> {
        if port == 0 || port > 65535 {
            Err("Port must be 1-65535".into())
        } else {
            Ok(Port(port))
        }
    }

    // Private: can only create Port through new()
    // Guarantees: all Port values are valid!
}

// Once created, Port is GUARANTEED valid
fn bind(port: Port) {
    // No need to check if port is valid - type system ensures it!
    TcpListener::bind(("0.0.0.0", port.0)).unwrap();
}
```

**4. Zero Runtime Cost**

```rust
struct Port(u16);

// At runtime:
assert_eq!(std::mem::size_of::<Port>(), std::mem::size_of::<u16>());
// Both are 2 bytes - no overhead!

// Compiler optimizes away the wrapper completely
```

This is called a **zero-cost abstraction**: safety without performance penalty.

---

### Parse, Don't Validate: A Philosophy

**The Traditional Approach (Validation)**:
```rust
fn process_config(port: u16) {
    if port == 0 {
        panic!("Invalid port");
    }
    // ... use port ...

    some_other_function(port);  // Must validate AGAIN!
}

fn some_other_function(port: u16) {
    if port == 0 {  // Duplicate validation everywhere!
        panic!("Invalid port");
    }
    // ... use port ...
}
```

**Problems**:
- Validation logic scattered everywhere
- Easy to forget validation
- No compile-time guarantee

**The Newtype Approach (Parsing)**:
```rust
fn process_config(port_str: &str) -> Result<(), String> {
    let port = Port::new(port_str.parse()?)?;  // Parse ONCE
    // From here on, port is GUARANTEED valid!

    some_other_function(port);  // No re-validation needed!
}

fn some_other_function(port: Port) {
    // Type signature says: port is valid
    // No validation code needed!
}
```

**Benefits**:
- Validation happens exactly once (at boundary)
- Type system propagates validity guarantee
- Impossible to forget validation

**Slogan**: **"Parse, don't validate"** - convert untrustworthy input into trustworthy types once, then rely on types.

---

### NonZeroU32: Built-in Safety

Rust provides `NonZeroU32` (and similar types) to encode "never zero" at the type level.

**Why It Exists**:
```rust
// Without NonZeroU32
struct MaxConnections(u32);

impl MaxConnections {
    fn new(count: u32) -> Result<Self, String> {
        if count == 0 {
            Err("Count must be > 0".into())
        } else {
            Ok(MaxConnections(count))
        }
    }
}

// Problem: Must validate manually, easy to forget check
```

**With NonZeroU32**:
```rust
use std::num::NonZeroU32;

struct MaxConnections(NonZeroU32);

impl MaxConnections {
    fn new(count: u32) -> Result<Self, String> {
        NonZeroU32::new(count)
            .map(MaxConnections)
            .ok_or_else(|| "Count must be > 0".into())
    }
}

// Benefits:
// 1. Type guarantees: can't create NonZeroU32 with 0
// 2. Niche optimization: Option<NonZeroU32> is same size as NonZeroU32
// 3. Documentation: type signature self-documents the constraint
```

**The Niche Optimization**:
```rust
// Regular u32
assert_eq!(std::mem::size_of::<u32>(), 4);
assert_eq!(std::mem::size_of::<Option<u32>>(), 8);  // Need discriminant

// NonZeroU32 uses 0 as "None" sentinel
assert_eq!(std::mem::size_of::<NonZeroU32>(), 4);
assert_eq!(std::mem::size_of::<Option<NonZeroU32>>(), 4);  // ← Same size!

// Rust uses 0 to represent None, avoiding extra space for discriminant
```

---

### The Builder Pattern

The **builder pattern** provides a fluent API for constructing complex objects step-by-step.

**The Problem with Constructors**:
```rust
// Too many parameters - hard to remember order
ServerConfig::new(
    "localhost".to_string(),
    8080,
    30,
    100,
    true,
    false,
    LogLevel::Info,
    Some("/var/log".into()),
);

// Which number is what? Easy to mix up!
```

**Solution: Builder Pattern**:
```rust
let config = ServerConfig::builder()
    .host("localhost")       // Clear what each does
    .port(8080)
    .timeout_secs(30)
    .max_connections(100)
    .enable_logging(true)
    .log_level(LogLevel::Info)
    .build()?;
```

**How It Works**:

```rust
struct ServerConfigBuilder {
    host: Option<String>,       // None = not set yet
    port: Option<u16>,
    timeout_secs: Option<u64>,
    // ... other fields ...
}

impl ServerConfigBuilder {
    fn new() -> Self {
        Self {
            host: None,
            port: None,
            timeout_secs: None,
        }
    }

    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self  // Return self for chaining!
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn build(self) -> Result<ServerConfig, Vec<String>> {
        // Validate all fields and construct ServerConfig
    }
}
```

**Key Design Decisions**:

1. **Consuming `self` vs `&mut self`**:
```rust
// Option A: &mut self (mutable reference)
fn port(&mut self, port: u16) -> &mut Self {
    self.port = Some(port);
    self
}

// Usage
let mut builder = ServerConfig::builder();
builder.port(8080).timeout_secs(30);

// Option B: self (consuming)
fn port(mut self, port: u16) -> Self {
    self.port = Some(port);
    self
}

// Usage - more ergonomic!
let config = ServerConfig::builder()
    .port(8080)
    .timeout_secs(30)
    .build();
```

Taking `self` by value:
- ✓ More ergonomic (no `mut` needed)
- ✓ Prevents reuse of partial builders
- ✓ Feels more "fluent"
- ✗ Slightly less flexible

2. **`impl Into<T>` for Flexibility**:
```rust
fn host(mut self, host: impl Into<String>) -> Self {
    self.host = Some(host.into());
    self
}

// Now accepts:
builder.host("localhost")             // &str
builder.host(hostname_string)         // String
builder.host(cow_string)              // Cow<str>
builder.host(format!("host{}", 1))   // String from format!
```

**Much better than**:
```rust
fn host(mut self, host: String) -> Self {
    // Forces callers to call .to_string() everywhere!
}
```

3. **Validation at `build()`, Not Setters**:
```rust
// ❌ Bad: Validate in setters
fn port(mut self, port: u16) -> Result<Self, String> {
    if port == 0 {
        return Err("Invalid port".into());
    }
    self.port = Some(port);
    Ok(self)
}

// Annoying: every setter returns Result!
let config = ServerConfig::builder()
    .port(8080)?           // Awkward
    .timeout_secs(30)?     // Lots of ?
    .build()?;

// ✓ Good: Validate in build()
fn port(mut self, port: u16) -> Self {
    self.port = Some(port);  // Just store, don't validate yet
    self
}

fn build(self) -> Result<ServerConfig, Vec<String>> {
    // Validate everything here, collect ALL errors
}

// Clean API:
let config = ServerConfig::builder()
    .port(8080)
    .timeout_secs(30)
    .build()?;  // Single ? at end
```

---

### Comprehensive Error Reporting

**The Problem: Fail-Fast Validation**:
```rust
fn validate(config: &Config) -> Result<(), String> {
    if config.port == 0 {
        return Err("Invalid port".into());  // Stop here!
    }
    if config.timeout == 0 {
        return Err("Invalid timeout".into());  // Never reached if port invalid
    }
    if config.max_conn == 0 {
        return Err("Invalid max_conn".into());  // Never reached if timeout invalid
    }
    Ok(())
}

// User experience:
// Run 1: "Invalid port" → fix port
// Run 2: "Invalid timeout" → fix timeout
// Run 3: "Invalid max_conn" → fix max_conn
// THREE iterations to find all errors!
```

**Solution: Collect All Errors**:
```rust
fn build(self) -> Result<ServerConfig, Vec<String>> {
    let mut errors = Vec::new();

    // Validate port
    let port = match self.port {
        Some(p) => match Port::new(p) {
            Ok(port) => port,
            Err(e) => {
                errors.push(format!("Port: {}", e));
                Port::new(8080).unwrap()  // Placeholder to continue
            }
        },
        None => Port::new(8080).unwrap(),  // Default
    };

    // Validate timeout (even if port failed!)
    let timeout = match self.timeout_secs {
        Some(t) => match Timeout::from_secs(t) {
            Ok(timeout) => timeout,
            Err(e) => {
                errors.push(format!("Timeout: {}", e));
                Timeout::from_secs(30).unwrap()
            }
        },
        None => Timeout::from_secs(30).unwrap(),
    };

    // ... validate all fields ...

    if !errors.is_empty() {
        Err(errors)  // Return ALL errors at once!
    } else {
        Ok(ServerConfig::new(port, timeout, ...))
    }
}

// User experience:
// Run 1: "Port: must be > 0, Timeout: must be > 0, MaxConn: must be > 0"
// ONE iteration to find all errors!
```

---

### The Deref Trait: Transparent Access

`Deref` allows a type to behave like a reference to another type.

**Without Deref**:
```rust
struct Port(u16);

impl Port {
    fn get(&self) -> u16 {
        self.0
    }
}

let port = Port(8080);
println!("Port: {}", port.get());  // Verbose
if port.get() > 1024 { }           // Clunky
```

**With Deref**:
```rust
use std::ops::Deref;

impl Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

let port = Port(8080);
println!("Port: {}", *port);  // Dereference to get u16
if *port > 1024 { }           // Direct comparison

// Auto-deref in many contexts:
if port > 1024 { }            // Compiler auto-inserts *
```

**How Deref Coercion Works**:
```rust
impl Deref for Port {
    type Target = u16;
    fn deref(&self) -> &u16 { &self.0 }
}

fn takes_u16(n: &u16) { }

let port = Port(8080);
takes_u16(&port);  // Compiler: &Port → &u16 via Deref!

// Compiler automatically tries:
// 1. Is &Port compatible with &u16? No
// 2. Does Port implement Deref? Yes, with Target = u16
// 3. Convert &Port to &u16 by calling deref()
```

**When to Use Deref**:

✅ **Good**:
- Smart pointers (`Box<T>`, `Rc<T>`, `Arc<T>`)
- Newtypes that are "essentially" the inner type
- Want transparent access to wrapped value

❌ **Avoid**:
- Type has additional semantics beyond wrapper
- Would hide important type distinction
- Inner type is implementation detail

**Example of BAD Deref use**:
```rust
// DON'T do this!
struct Password(String);

impl Deref for Password {
    type Target = String;
    fn deref(&self) -> &String { &self.0 }
}

// Now Password behaves like String:
let pw = Password("secret123".into());
println!("{}", pw);  // ❌ Accidentally prints password!

// Better: NO Deref, force explicit access
// pw.as_str()  // Explicit, harder to accidentally leak
```

---

### Default Values and Required Fields

**Design Pattern: Optional vs Required**:

```rust
impl ServerConfigBuilder {
    fn build(self) -> Result<ServerConfig, Vec<String>> {
        let mut errors = Vec::new();

        // REQUIRED field: Error if missing
        let host = match self.host {
            Some(h) if !h.is_empty() => Hostname(h),
            Some(_) => {
                errors.push("Host cannot be empty".into());
                Hostname("localhost".into())  // Placeholder for error path
            }
            None => {
                errors.push("Host is required".into());
                Hostname("localhost".into())
            }
        };

        // OPTIONAL field: Default if missing
        let port = match self.port {
            Some(p) => Port::new(p)?,
            None => Port::new(8080).unwrap(),  // Sensible default
        };

        // ...
    }
}
```

**Choosing Good Defaults**:

✓ **Good defaults**:
- Port 8080 (common HTTP alternate)
- Timeout 30 seconds (reasonable for web)
- Max connections 100 (prevents resource exhaustion)

✗ **Bad defaults**:
- Port 0 (invalid!)
- Timeout 0 (no timeout = hang forever)
- Max connections unlimited (resource exhaustion)

**Document defaults**:
```rust
impl ServerConfig {
    /// Creates a builder with these defaults:
    /// - port: 8080
    /// - timeout: 30 seconds
    /// - max_connections: 100
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
    }
}
```

---

### Connection to This Project

In this project, you'll implement all these concepts:

1. **Milestone 1**: Basic struct with primitive types
   - See the problems with type-unsafe config
   - Understand why validation is insufficient

2. **Milestone 2**: Newtype wrappers with validation
   - Create distinct types for each config value
   - Implement smart constructors
   - Use NonZeroU32 for guaranteed non-zero values
   - Achieve zero-cost type safety

3. **Milestone 3**: Builder pattern with defaults
   - Fluent API for construction
   - Comprehensive error collection
   - Deref for ergonomic access
   - Optional fields with sensible defaults

**Key Learning Points**:
- Newtypes provide compile-time safety at zero runtime cost
- Smart constructors enforce invariants once at creation
- Builder pattern makes complex construction ergonomic
- Collecting all errors improves user experience
- Deref enables transparent access while maintaining type safety

**Real-World Applications**:
- AWS SDK: All service clients use builder pattern
- reqwest: HTTP client configuration
- tokio: Runtime and task configuration
- diesel: Database connection configuration
- clap: Command-line argument parsing
- serde: Serialization configuration

---

### Why It Matters

**Real-World Impact**: Configuration bugs are a major source of production outages:

**The Configuration Hell Problem**:
- **Knight Capital (2012)**: Wrong server configuration caused $440M loss in 45 minutes
- **AWS S3 Outage (2017)**: Typo in configuration command took down large portion of internet
- **Common bugs**: Port 80 vs 8080, localhost vs production host, mixing dev/prod credentials
- **Type confusion**: Passing timeout in seconds vs milliseconds, mixing database connection strings

**Without Type Safety**:
```rust
struct Config {
    host: String,      // Could be "localhost", "localhost:8080", or "https://..."
    port: u16,         // Could be 0, 99999, or negative number
    timeout: u64,      // Seconds? Milliseconds? Minutes?
    max_connections: i32, // Could be negative!
}

// Bug: Accidentally swap host and database_url
let config = Config {
    host: "postgres://localhost/db".to_string(), // Wrong!
    port: 5432,  // Database port, not HTTP port!
    timeout: 30, // 30 what?
    max_connections: -1, // Negative connections?
};
```

**With Newtype Pattern**:
```rust
struct Hostname(String);
struct Port(u16);
struct Timeout(Duration);
struct MaxConnections(NonZeroU32);

// Compiler prevents mixing types
fn connect(host: Hostname, port: Port) { }
connect(Port(8080), Hostname("localhost")); // ❌ Compile error!
```

**Performance Benefits**:
- **Zero runtime cost**: Newtypes compile to same memory layout as wrapped type
- **Compile-time guarantees**: Invalid states impossible to represent
- **Elimination of defensive code**: No need to check if port > 0, it's guaranteed


**Type Safety Prevents**:
- Passing milliseconds where seconds expected (1000x bug!)
- Using database port for HTTP server
- Negative values for counts/sizes
- Empty strings for required fields
- Mixing development and production settings


---

### Milestone 1: Basic Configuration Struct

**Goal**: Create a basic configuration struct `ServerConfig` with named fields to group related configuration data

**struct** ServerConfig
- **field**: host: String,           // Server hostname/IP address (e.g., "localhost", "0.0.0.0")
- **field**: port: u16,             // TCP port number (1-65535)
- **field**: timeout_seconds: u64,  // Connection timeout duration in seconds
- **field**: max_connections: u32,  // Maximum concurrent client connections allowed



**Memory Layout**:

```rust
ServerConfig:
  host: String         [24 bytes: ptr + len + capacity]
  port: u16            [2 bytes]
  timeout_seconds: u64 [8 bytes]
  max_connections: u32 [4 bytes]
  [+ 6 bytes padding for alignment]
Total: ~44 bytes
```

No runtime overhead for grouping is just the sum of field sizes plus alignment.

**Checkpoint Tests**:
```rust
#[test]
fn test_basic_config() {
    let config = ServerConfig::new(
        "localhost".to_string(),
        8080,
        30,
        100,
    );

    assert_eq!(config.host, "localhost");
    assert_eq!(config.port, 8080);
}

#[test]
fn test_can_create_invalid_config() {
    // This compiles but is semantically wrong!
    let bad_config = ServerConfig::new(
        "".to_string(),           // Empty host
        0,                        // Invalid port
        0,                        // Zero timeout
        0,                        // Zero connections
    );

    // No way to prevent this at compile time yet
    assert_eq!(bad_config.port, 0);
}
```

**Starter Code**:
```rust
// ServerConfig: Main configuration struct for web server settings
// Role: Groups all server configuration parameters together
#[derive(Debug, Clone)]
struct ServerConfig {
    host: String,           // Server hostname/IP address (e.g., "localhost", "0.0.0.0")
    port: u16,             // TCP port number (1-65535)
    timeout_seconds: u64,  // Connection timeout duration in seconds
    max_connections: u32,  // Maximum concurrent client connections allowed
}

impl ServerConfig {
    // new: Constructor that creates a ServerConfig instance
    // Role: Initializes configuration with provided values
    fn new(host: String, port: u16, timeout_seconds: u64, max_connections: u32) -> Self {
        // TODO: Create ServerConfig with given values
        todo!()
    }
}
```

**Check Your Understanding**:
- What's wrong with allowing `port: 0` or `max_connections: 0`?
- How could we accidentally pass the wrong string to the host parameter?
- What happens if someone passes timeout in milliseconds by mistake?

---

### Why Milestone 1 Isn't Enough 

**Critical Limitations**:
1. **No type safety**: Can swap `host` and `database_url` parameters - both are `String`
2. **No validation**: Can create config with `port: 0`, `max_connections: 0`, empty host
3. **No semantic meaning**: Is timeout in seconds? Milliseconds? Minutes?
4. **Easy to mix up**: `ServerConfig::new(port_str, host_str)` compiles if you swap them
5. **Debug leaks secrets**: `println!("{:?}", config)` might print passwords

**What we're adding**: **Newtype wrappers** for each configuration value:
- `Port(u16)`, `Hostname(String)`, `Timeout(Duration)`, `MaxConnections(NonZeroU32)`
- Each is a distinct type - compiler prevents mixing them up
- Smart constructors validate inputs
- Custom `Debug` implementations hide sensitive data

**Improvements**:
- **Type safety**: Can't pass `Port` where `Hostname` expected
- **Validation**: `Port::new(0)` returns `Err` - can't create invalid port
- **Clarity**: `Timeout(Duration::from_secs(30))` is unambiguous
- **Security**: `Password` type hides value in Debug output

---

### Milestone 2: Newtype Wrappers for Type Safety

**Goal**: Wrap each configuration field in a distinct newtype to prevent accidental mixing and enable field-specific validation.


**The Core Problem: Primitive Obsession**:

```rust
// All these are just numbers or strings!
fn configure(
    port: u16,           // Could be 0-65535
    timeout: u64,        // Could be anything
    max_conn: u32,       // Could be 0 or negative logic
    pool_size: u16,      // Same type as port!
) { }

// Compiler can't help you here:
configure(
    30,      // Meant to be timeout, passed as port!
    8080,    // Meant to be port, passed as timeout!
    0,       // Invalid but compiles!
    100,
);
```

**The Solution: Newtype Pattern**:

```rust
struct Port(u16);
struct Timeout(Duration);
struct MaxConnections(NonZeroU32);
struct PoolSize(NonZeroU16);

fn configure(
    port: Port,
    timeout: Timeout,
    max_conn: MaxConnections,
    pool_size: PoolSize,
) { }

// Now this won't compile!
configure(
    Timeout::from_secs(30),  // ❌ Expected Port, got Timeout
    Port::new(8080).unwrap(), // ❌ Expected Timeout, got Port
    ...
);
```

**What We're Building**:

Four newtype wrappers with validation:

1. **`Hostname(String)`**: Type-safe string that's specifically a hostname
2. **`Port(u16)`**: Validated port number (1-65535)
3. **`Timeout(Duration)`**: Positive duration with clear units
4. **`MaxConnections(NonZeroU32)`**: Guaranteed positive connection limit


**The Validation Strategy**:

Each newtype implements validation in its constructor:

```rust
impl Port {
    fn new(port: u16) -> Result<Self, String> {
        // Range check
        if port == 0 {
            return Err("Port must be > 0".to_string());
        }
        Ok(Port(port))
    }
}
```

**Key Design Principle: Parse, Don't Validate**:

Once a value is wrapped in a newtype, it's **guaranteed valid**. No need to re-check:

```rust
fn start_server(port: Port) {
    // No need to check if port > 0
    // Type system guarantees it!
    let socket = TcpListener::bind(("0.0.0.0", port.get())).unwrap();
}
```


**Starter Code**:
```rust
use std::fmt;
use std::num::NonZeroU32;
use std::time::Duration;

// Hostname: Newtype wrapper for server hostname
// Role: Prevents mixing hostname strings with other string types
#[derive(Debug, Clone, PartialEq)]
struct Hostname(String);  // Inner field: the hostname string value

// Port: Newtype wrapper for TCP port numbers with validation
// Role: Ensures port is always valid (1-65535), prevents mixing with other integers
#[derive(Debug, Clone, Copy, PartialEq)]
struct Port(u16);  // Inner field: validated port number

impl Port {
    // new: Smart constructor that validates port range
    // Role: Ensures only valid ports can be created
    fn new(port: u16) -> Result<Self, String> {
        // TODO: Validate port is in range 1..=65535
        // Hint: Check port > 0, return Ok(Port(port)) or Err with message
        todo!()
    }

    // get: Accessor for inner port value
    // Role: Extracts the raw u16 port number
    fn get(&self) -> u16 {
        // TODO: Return the inner u16 value
        todo!()
    }
}

// Timeout: Newtype wrapper for timeout durations
// Role: Enforces timeout is positive duration, prevents mixing raw numbers
#[derive(Debug, Clone, Copy, PartialEq)]
struct Timeout(Duration);  // Inner field: std::time::Duration value

impl Timeout {
    // from_secs: Constructor that creates timeout from seconds
    // Role: Validates positive timeout and wraps in Duration type
    fn from_secs(secs: u64) -> Result<Self, String> {
        // TODO: Validate secs > 0, wrap in Duration
        // Hint: Duration::from_secs(secs)
        todo!()
    }

    // as_duration: Accessor for inner Duration
    // Role: Provides access to underlying Duration for time operations
    fn as_duration(&self) -> Duration {
        // TODO: Return inner Duration
        todo!()
    }
}

// MaxConnections: Newtype wrapper for connection limits using NonZeroU32
// Role: Guarantees connection limit is always positive (can't be zero)
#[derive(Debug, Clone, Copy, PartialEq)]
struct MaxConnections(NonZeroU32);  // Inner field: guaranteed non-zero value

impl MaxConnections {
    // new: Smart constructor that ensures non-zero connection count
    // Role: Validates and creates non-zero connection limit
    fn new(count: u32) -> Result<Self, String> {
        // TODO: Convert to NonZeroU32, handle zero case
        // Hint: NonZeroU32::new(count).ok_or_else(|| "count must be > 0")
        todo!()
    }

    // get: Accessor for inner connection count
    // Role: Extracts the raw u32 value (guaranteed non-zero)
    fn get(&self) -> u32 {
        // TODO: Return inner value as u32
        // Hint: self.0.get()
        todo!()
    }
}

// ServerConfig: Updated configuration using type-safe newtypes
// Role: Groups validated, type-safe configuration parameters
#[derive(Debug, Clone)]
struct ServerConfig {
    host: Hostname,                   // Type-safe hostname
    port: Port,                       // Validated port number
    timeout: Timeout,                 // Validated timeout duration
    max_connections: MaxConnections,  // Guaranteed positive connection limit
}

impl ServerConfig {
    // new: Constructor accepting only validated newtype values
    // Role: Creates config from type-safe components (no validation needed here)
    fn new(
        host: Hostname,
        port: Port,
        timeout: Timeout,
        max_connections: MaxConnections,
    ) -> Self {
        // TODO: Create ServerConfig with newtype fields
        todo!()
    }
}
```
**Checkpoint Tests**:
```rust
#[test]
fn test_port_validation() {
    assert!(Port::new(8080).is_ok());
    assert!(Port::new(0).is_err());  // Invalid port
    assert!(Port::new(65535).is_ok()); // Max valid port
}

#[test]
fn test_cannot_swap_types() {
    // This won't compile - demonstrates type safety!
    // let port = Port::new(8080).unwrap();
    // let host = Hostname("localhost".to_string());
    // let config = ServerConfig::new(port, host, ...); // ❌ Type error!
}

#[test]
fn test_timeout_validation() {
    assert!(Timeout::from_secs(30).is_ok());
    assert!(Timeout::from_secs(0).is_err()); // Zero timeout invalid
}

#[test]
fn test_max_connections() {
    assert!(MaxConnections::new(100).is_ok());
    assert!(MaxConnections::new(0).is_err()); // Zero connections invalid
}

#[test]
fn test_valid_config() {
    let config = ServerConfig::new(
        Hostname("localhost".to_string()),
        Port::new(8080).unwrap(),
        Timeout::from_secs(30).unwrap(),
        MaxConnections::new(100).unwrap(),
    );

    assert_eq!(config.port.get(), 8080);
}
```

**Check Your Understanding**:
- Why can't you accidentally swap `Port` and `MaxConnections` now?
- What happens at compile-time if you try `Port::new(8080).unwrap().as_duration()`?
- Why use `NonZeroU32` instead of validating `u32 > 0` manually?
- What's the memory overhead of these newtypes? (Hint: zero!)

---

### Why Milestone 2 Isn't Enough 

**Remaining Issues**:
1. **Verbose construction**: Must call `.unwrap()` multiple times, lots of `Result` handling
2. **Inflexible**: Can't create config incrementally or with defaults
3. **Poor ergonomics**: `config.port.get()` is clunky compared to `config.port` direct access
4. **No validation context**: Errors don't say *which* field failed

**What we're adding**:
- **Builder pattern**: Fluent API for constructing config step-by-step
- **Default values**: Reasonable defaults for optional fields
- **Better error handling**: Collect all validation errors, not just first one
- **Deref implementation**: Transparent access to inner values

**Improvements**:
- **Ergonomics**: `config.port` instead of `config.port.get()` via `Deref`
- **Flexibility**: `ServerConfig::builder().port(8080).timeout_secs(30).build()`
- **Better errors**: "Invalid port: 0, Invalid timeout: 0" (all errors at once)
- **Defaults**: Can omit optional fields, builder provides sensible defaults

---

### Milestone 3: Builder Pattern with Defaults

**Goal**: Create a fluent builder API that makes construction ergonomic, provides sensible defaults, and collects all validation errors at once.

**Why This Milestone Matters**:

**The Ergonomics Problem**:

```rust
// Milestone 2 API - verbose and error-prone
let config = ServerConfig::new(
    Hostname("localhost".to_string()),
    Port::new(8080).unwrap(),  // Panic if invalid!
    Timeout::from_secs(30).unwrap(),
    MaxConnections::new(100).unwrap(),
);
```

**Problems**:
1. **Verbose**: Many function calls, lots of `.unwrap()`
2. **Panic-prone**: `.unwrap()` panics on invalid input
3. **No defaults**: Must specify every field explicitly
4. **Error handling**: First error stops construction, others ignored

**The Builder Solution**:

```rust
// Milestone 3 API - fluent and safe
let config = ServerConfig::builder()
    .host("localhost")          // Strings auto-converted
    .port(8080)                 // Validation deferred
    .timeout_secs(30)
    .max_connections(100)
    .build()?;                  // All errors reported at once

// With defaults
let config = ServerConfig::builder()
    .host("localhost")
    .build()?;  // Uses default port, timeout, max_connections
```

**What We're Building**:

Three key components:

1. **`ServerConfigBuilder`**: Collects configuration values with `Option` fields
2. **Fluent methods**: Each method returns `self` for chaining
3. **`build()` method**: Validates everything, applies defaults, returns `Result`

**The Builder Pattern Structure**:

```rust
struct ServerConfigBuilder {
    host: Option<String>,        // Not built yet
    port: Option<u16>,          // Raw value, not validated
    timeout_secs: Option<u64>,
    max_connections: Option<u32>,
}

impl ServerConfigBuilder {
    fn new() -> Self {
        ServerConfigBuilder {
            host: None,
            port: None,
            timeout_secs: None,
            max_connections: None,
        }
    }

    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self  // Return self for chaining!
    }

    fn build(self) -> Result<ServerConfig, Vec<String>> {
        // Validate everything here
    }
}
```

**Why Take `self` Not `&mut self`?**

This is a subtle but important design choice:

```rust
// With &mut self (mutable reference)
fn port(&mut self, port: u16) -> &mut Self {
    self.port = Some(port);
    self
}

// Usage - less ergonomic
let mut builder = ServerConfig::builder();
builder.port(8080).timeout_secs(30);

// With self (consuming)
fn port(mut self, port: u16) -> Self {
    self.port = Some(port);
    self
}

// Usage - more ergonomic
let config = ServerConfig::builder()
    .port(8080)      // Consumes and returns new builder
    .timeout_secs(30)  // Chains naturally
    .build();
```

Taking `self` by value **prevents reuse** of partially-built builders, which is usually what you want.

**The Power of `impl Into<String>`**:

```rust
fn host(mut self, host: impl Into<String>) -> Self {
    self.host = Some(host.into());
    self
}
```

This accepts **any type that can be converted to `String`**:
- `&str`: `builder.host("localhost")`
- `String`: `builder.host(hostname_variable)`
- `Cow<str>`: `builder.host(cow_string)`

More flexible than `host: String`, which requires `.to_string()` everywhere!

**Validation Strategy: Collect All Errors**:

```rust
fn build(self) -> Result<ServerConfig, Vec<String>> {
    let mut errors = Vec::new();

    // Validate host
    let host = match self.host {
        Some(h) if !h.is_empty() => Hostname(h),
        Some(_) => {
            errors.push("Host cannot be empty".to_string());
            Hostname("localhost".to_string())  // Placeholder
        }
        None => {
            errors.push("Host is required".to_string());
            Hostname("localhost".to_string())
        }
    };

    // Validate port (with default)
    let port = match self.port {
        Some(p) => match Port::new(p) {
            Ok(port) => port,
            Err(e) => {
                errors.push(format!("Invalid port: {}", e));
                Port::new(8080).unwrap()  // Safe default
            }
        },
        None => Port::new(8080).unwrap(),  // Default
    };

    // ... similar for other fields ...

    if !errors.is_empty() {
        Err(errors)  // Return ALL errors
    } else {
        Ok(ServerConfig::new(host, port, timeout, max_connections))
    }
}
```

**Why Collect All Errors?**

**Bad UX** (stop on first error):
```
❌ Port must be greater than 0

Fix it, run again...

❌ Timeout must be greater than 0 seconds

Fix it, run again...

❌ Connection count must be greater than 0
```

**Good UX** (report all errors):
```
❌ Multiple validation errors:
  - Port must be greater than 0
  - Timeout must be greater than 0 seconds
  - Connection count must be greater than 0

Fix all three at once!
```

**The `Deref` Trait for Ergonomics**:

```rust
impl Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
```

Now you can use `Port` almost like a `u16`:

```rust
let port = Port::new(8080).unwrap();

// Without Deref
println!("Port: {}", port.get());

// With Deref
println!("Port: {}", *port);  // Dereference to u16

// Even auto-derefs in many contexts
if port > 1024 {  // Auto-derefs to u16 for comparison!
    println!("Unprivileged port");
}
```

**When to Use Deref**:

✅ **Good use cases**:
- Newtypes wrapping a single value
- Want transparent access to inner value
- Inner value is "the essence" of the type

❌ **Avoid Deref when**:
- Type has additional semantics beyond the wrapped value
- Deref would expose internal implementation details
- Want to prevent confusion with the inner type

**Default Values Design**:

```rust
// Required fields: None = error
let host = match self.host {
    Some(h) => h,
    None => {
        errors.push("Host is required".to_string());
        "localhost".to_string()  // Placeholder for error path
    }
};

// Optional fields: None = default
let port = match self.port {
    Some(p) => Port::new(p)?,
    None => Port::new(8080).unwrap(),  // Sensible default
};
```

**Real-World Builder Examples**:

1. **reqwest::Client**: HTTP client builder
   ```rust
   let client = Client::builder()
       .timeout(Duration::from_secs(10))
       .gzip(true)
       .build()?;
   ```

2. **tokio::Runtime**: Async runtime builder
   ```rust
   let runtime = Runtime::builder()
       .worker_threads(4)
       .thread_name("my-pool")
       .build()?;
   ```

3. **AWS SDK**: Service client builders
   ```rust
   let client = S3Client::builder()
       .region(Region::new("us-east-1"))
       .credentials_provider(provider)
       .build();
   ```

**Type State Builder (Advanced)**:

You can even use the typestate pattern on builders to enforce "host must be set before build":

```rust
struct NoHost;
struct HasHost;

struct ServerConfigBuilder<State> {
    host: Option<String>,
    _state: PhantomData<State>,
}

impl ServerConfigBuilder<NoHost> {
    fn host(self, host: impl Into<String>) -> ServerConfigBuilder<HasHost> {
        // Returns different type!
    }
}

impl ServerConfigBuilder<HasHost> {
    fn build(self) -> Result<ServerConfig, Vec<String>> {
        // Only callable after host() was called!
    }
}
```


**Starter Code**:
```rust
use std::ops::Deref;

// Deref implementation for Port
// Role: Allows transparent access to inner u16 value using * operator
impl Deref for Port {
    type Target = u16;  // Dereferencing produces a &u16

    // deref: Provides reference to inner value
    // Role: Enables ergonomic access like *port instead of port.get()
    fn deref(&self) -> &Self::Target {
        // TODO: Return reference to inner u16
        todo!()
    }
}

// TODO: Implement Deref for other newtypes similarly (Timeout, MaxConnections)

// ServerConfigBuilder: Fluent builder for constructing ServerConfig
// Role: Collects configuration values step-by-step with validation and defaults
struct ServerConfigBuilder {
    host: Option<String>,        // Optional hostname (required field)
    port: Option<u16>,          // Optional port (defaults to 8080)
    timeout_secs: Option<u64>,  // Optional timeout (defaults to 30 seconds)
    max_connections: Option<u32>, // Optional connection limit (defaults to 100)
}

impl ServerConfigBuilder {
    // new: Creates empty builder
    // Role: Initializes all fields to None, ready for configuration
    fn new() -> Self {
        // TODO: Create builder with all None values
        todo!()
    }

    // host: Sets hostname in builder
    // Role: Accepts any type convertible to String for flexibility
    fn host(mut self, host: impl Into<String>) -> Self {
        // TODO: Set host field and return self for chaining
        // Hint: self.host = Some(host.into()); self
        todo!()
    }

    // port: Sets port number in builder
    // Role: Stores port for later validation during build()
    fn port(mut self, port: u16) -> Self {
        // TODO: Set port and return self
        todo!()
    }

    // timeout_secs: Sets timeout duration in seconds
    // Role: Stores timeout for validation and Duration conversion during build()
    fn timeout_secs(mut self, secs: u64) -> Self {
        // TODO: Set timeout_secs and return self
        todo!()
    }

    // max_connections: Sets maximum connection limit
    // Role: Stores connection limit for validation during build()
    fn max_connections(mut self, max: u32) -> Self {
        // TODO: Set max_connections and return self
        todo!()
    }

    // build: Validates all fields and constructs ServerConfig
    // Role: Applies defaults, validates all values, collects all errors
    fn build(self) -> Result<ServerConfig, Vec<String>> {
        let mut errors = Vec::new();

        // Validate and extract host, or add error
        let host = match self.host {
            Some(h) if !h.is_empty() => Hostname(h),
            Some(_) => {
                errors.push("Host cannot be empty".to_string());
                Hostname("localhost".to_string()) // Placeholder
            }
            None => {
                errors.push("Host is required".to_string());
                Hostname("localhost".to_string())
            }
        };

        // Validate port or use default 8080
        let port = match self.port {
            Some(p) => match Port::new(p) {
                Ok(port) => port,
                Err(e) => {
                    errors.push(format!("Invalid port: {}", e));
                    Port::new(8080).unwrap() // Safe default
                }
            },
            None => Port::new(8080).unwrap(), // Default
        };

        // TODO: Similar for timeout (default 30s)
        let timeout = todo!();

        // TODO: Similar for max_connections (default 100)
        let max_connections = todo!();

        // If errors, return Err(errors), else Ok(config)
        // Role: Reports all validation errors at once, or returns valid config
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ServerConfig::new(host, port, timeout, max_connections))
        }
    }
}

impl ServerConfig {
    // builder: Entry point for fluent configuration API
    // Role: Creates new builder instance to start configuration chain
    fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_builder_fluent_api() {
    let config = ServerConfig::builder()
        .host("localhost")
        .port(8080)
        .timeout_secs(30)
        .max_connections(100)
        .build()
        .unwrap();

    assert_eq!(*config.port, 8080); // Deref in action
}

#[test]
fn test_builder_defaults() {
    let config = ServerConfig::builder()
        .host("localhost")
        .build()
        .unwrap();

    // Should use default values
    assert_eq!(*config.port, 8080);
    assert_eq!(*config.max_connections, 100);
}

#[test]
fn test_builder_validation_errors() {
    let result = ServerConfig::builder()
        .port(0)  // Invalid
        .timeout_secs(0)  // Invalid
        .build();

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.len() >= 2); // At least port and timeout errors
}

#[test]
fn test_builder_missing_required() {
    let result = ServerConfig::builder()
        .port(8080)
        .build();

    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| e.contains("Host")));
}
```


**Check Your Understanding**:
- How does `Deref` allow `*config.port` to work?
- Why return `Self` from builder methods instead of `&mut Self`?
- What's the benefit of collecting all errors vs returning on first error?
- How does `impl Into<String>` make the API more flexible?

---

### Complete Project Summary

**What You Built**:
1. Basic configuration struct with named fields
2. Newtype wrappers for type safety and validation
3. Fluent builder API with defaults and comprehensive error reporting
4. Deref implementation for ergonomic access

**Key Concepts Practiced**:
- Newtype pattern for compile-time type safety
- Smart constructors with validation
- Builder pattern for ergonomic APIs
- Deref trait for transparent access
- Collecting multiple validation errors

**Real-World Application**: This pattern is used in:
- AWS SDK configuration builders
- HTTP client configuration (reqwest, hyper)
- Database connection configuration
- Server/application configuration libraries

---

### Complete Working Example

Here's the fully implemented solution combining all three milestones:

```rust
use std::fmt;
use std::num::NonZeroU32;
use std::time::Duration;
use std::ops::Deref;

// ============================================================================
// MILESTONE 2: Newtype Wrappers
// ============================================================================

// Hostname: Type-safe wrapper for server hostname
#[derive(Debug, Clone, PartialEq)]
struct Hostname(String);

// Port: Validated TCP port number (1-65535)
#[derive(Debug, Clone, Copy, PartialEq)]
struct Port(u16);

impl Port {
    fn new(port: u16) -> Result<Self, String> {
        if port == 0 {
            Err("Port must be greater than 0".to_string())
        } else {
            Ok(Port(port))
        }
    }

    fn get(&self) -> u16 {
        self.0
    }
}

// Deref implementation for ergonomic access
impl Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Timeout: Validated timeout duration
#[derive(Debug, Clone, Copy, PartialEq)]
struct Timeout(Duration);

impl Timeout {
    fn from_secs(secs: u64) -> Result<Self, String> {
        if secs == 0 {
            Err("Timeout must be greater than 0 seconds".to_string())
        } else {
            Ok(Timeout(Duration::from_secs(secs)))
        }
    }

    fn as_duration(&self) -> Duration {
        self.0
    }
}

impl Deref for Timeout {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// MaxConnections: Guaranteed non-zero connection limit
#[derive(Debug, Clone, Copy, PartialEq)]
struct MaxConnections(NonZeroU32);

impl MaxConnections {
    fn new(count: u32) -> Result<Self, String> {
        NonZeroU32::new(count)
            .map(MaxConnections)
            .ok_or_else(|| "Connection count must be greater than 0".to_string())
    }

    fn get(&self) -> u32 {
        self.0.get()
    }
}

impl Deref for MaxConnections {
    type Target = NonZeroU32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ============================================================================
// ServerConfig: Type-safe configuration struct
// ============================================================================

#[derive(Debug, Clone)]
struct ServerConfig {
    host: Hostname,
    port: Port,
    timeout: Timeout,
    max_connections: MaxConnections,
}

impl ServerConfig {
    fn new(
        host: Hostname,
        port: Port,
        timeout: Timeout,
        max_connections: MaxConnections,
    ) -> Self {
        ServerConfig {
            host,
            port,
            timeout,
            max_connections,
        }
    }

    fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
    }
}

// ============================================================================
// MILESTONE 3: Builder Pattern
// ============================================================================

struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout_secs: Option<u64>,
    max_connections: Option<u32>,
}

impl ServerConfigBuilder {
    fn new() -> Self {
        ServerConfigBuilder {
            host: None,
            port: None,
            timeout_secs: None,
            max_connections: None,
        }
    }

    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    fn build(self) -> Result<ServerConfig, Vec<String>> {
        let mut errors = Vec::new();

        // Validate hostname
        let host = match self.host {
            Some(h) if !h.is_empty() => Hostname(h),
            Some(_) => {
                errors.push("Host cannot be empty".to_string());
                Hostname("localhost".to_string())
            }
            None => {
                errors.push("Host is required".to_string());
                Hostname("localhost".to_string())
            }
        };

        // Validate port with default
        let port = match self.port {
            Some(p) => match Port::new(p) {
                Ok(port) => port,
                Err(e) => {
                    errors.push(format!("Invalid port: {}", e));
                    Port::new(8080).unwrap()
                }
            },
            None => Port::new(8080).unwrap(),
        };

        // Validate timeout with default
        let timeout = match self.timeout_secs {
            Some(secs) => match Timeout::from_secs(secs) {
                Ok(timeout) => timeout,
                Err(e) => {
                    errors.push(format!("Invalid timeout: {}", e));
                    Timeout::from_secs(30).unwrap()
                }
            },
            None => Timeout::from_secs(30).unwrap(),
        };

        // Validate max_connections with default
        let max_connections = match self.max_connections {
            Some(max) => match MaxConnections::new(max) {
                Ok(mc) => mc,
                Err(e) => {
                    errors.push(format!("Invalid max_connections: {}", e));
                    MaxConnections::new(100).unwrap()
                }
            },
            None => MaxConnections::new(100).unwrap(),
        };

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ServerConfig::new(host, port, timeout, max_connections))
        }
    }
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    // Example 1: Using builder with all fields
    let config = ServerConfig::builder()
        .host("0.0.0.0")
        .port(8080)
        .timeout_secs(30)
        .max_connections(100)
        .build()
        .unwrap();

    println!("Config 1: {:?}", config);
    println!("  Port: {}", *config.port);  // Deref in action!

    // Example 2: Using builder with defaults
    let config2 = ServerConfig::builder()
        .host("localhost")
        .build()
        .unwrap();

    println!("\nConfig 2 (with defaults): {:?}", config2);

    // Example 3: Handling validation errors
    let result = ServerConfig::builder()
        .port(0)  // Invalid!
        .timeout_secs(0)  // Invalid!
        .max_connections(0)  // Invalid!
        .build();

    match result {
        Ok(_) => println!("\nUnexpected success"),
        Err(errors) => {
            println!("\nValidation errors:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }

    // Example 4: Type safety prevents mistakes
    let port = Port::new(8080).unwrap();
    let timeout = Timeout::from_secs(30).unwrap();

    // This won't compile - type safety!
    // let bad_config = ServerConfig::new(
    //     Hostname("localhost".to_string()),
    //     timeout,  // ❌ Wrong type! Expected Port, got Timeout
    //     port,     // ❌ Wrong type! Expected Timeout, got Port
    //     MaxConnections::new(100).unwrap(),
    // );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtype_validation() {
        assert!(Port::new(8080).is_ok());
        assert!(Port::new(0).is_err());
        assert!(Timeout::from_secs(30).is_ok());
        assert!(Timeout::from_secs(0).is_err());
        assert!(MaxConnections::new(100).is_ok());
        assert!(MaxConnections::new(0).is_err());
    }

    #[test]
    fn test_builder_success() {
        let config = ServerConfig::builder()
            .host("localhost")
            .port(8080)
            .timeout_secs(30)
            .max_connections(100)
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_builder_defaults() {
        let config = ServerConfig::builder()
            .host("localhost")
            .build()
            .unwrap();

        assert_eq!(*config.port, 8080);
        assert_eq!(config.max_connections.get(), 100);
    }

    #[test]
    fn test_builder_validation_errors() {
        let result = ServerConfig::builder()
            .port(0)
            .timeout_secs(0)
            .build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 3); // host required, invalid port, invalid timeout
    }

    #[test]
    fn test_deref_trait() {
        let port = Port::new(8080).unwrap();
        assert_eq!(*port, 8080);

        let timeout = Timeout::from_secs(30).unwrap();
        assert_eq!(timeout.as_secs(), 30);
    }
}
```

**Key Takeaways from Complete Example**:

1. **Type Safety**: Compiler prevents mixing up `Port`, `Timeout`, and `MaxConnections`
2. **Validation**: Smart constructors ensure only valid values can be created
3. **Ergonomics**: Builder pattern + Deref trait make API pleasant to use
4. **Error Handling**: Collects all validation errors, not just first one
5. **Zero Cost**: All newtypes compile to same size as underlying types
6. **Defaults**: Builder provides sensible defaults for optional fields

---
