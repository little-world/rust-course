# Chapter 02: Struct & Enum Patterns - Programming Projects


## Project 1: Type-Safe Configuration System with Newtype Pattern

### Problem Statement

Build a configuration system for a web server that uses newtype wrappers to prevent common configuration errors. You'll start with basic structs, then add type safety through newtypes, validated types, and finally a fluent builder API.

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

**Real Production Examples**:
- **AWS SDK**: Uses newtypes for region names, bucket names, instance IDs
- **Kubernetes**: Type-safe wrappers for namespace, pod name, service name
- **Database drivers**: Connection strings, table names, column names as types
- **Web frameworks**: PathBuf vs AssetPath vs TemplatePath - prevent mixing

### Use Cases

**When you need this pattern**:
1. **Server configuration**: Ports, hostnames, URLs, timeouts - prevent mixing
2. **Database configuration**: Connection strings, pool sizes, credentials
3. **API clients**: Endpoints, API keys, rate limits, retry policies
4. **File paths**: Config vs data vs cache paths - type-safe separation
5. **Resource limits**: Memory limits, CPU limits, connection limits - enforce positivity
6. **Credentials**: Username, password, API tokens - hide in Debug output

**Type Safety Prevents**:
- Passing milliseconds where seconds expected (1000x bug!)
- Using database port for HTTP server
- Negative values for counts/sizes
- Empty strings for required fields
- Mixing development and production settings

### Learning Goals

- Understand newtype pattern for compile-time type safety
- Implement validated types that enforce invariants
- Build smart constructors that prevent invalid states
- Create fluent builder APIs with method chaining
- Use `Deref` for ergonomic access to wrapped values
- Hide sensitive data in `Debug` implementations

---

### Milestone 1: Basic Configuration Struct

**Goal**: Create a basic configuration struct `ServerConfig` with named fields to group related configuration data together.

**Why This Milestone Matters**:

This milestone establishes the **foundation** for type-safe configuration. Even though we'll identify problems with this approach, understanding the baseline is crucial for appreciating the improvements in later milestones.

**What We're Building**:

A simple struct that groups all server configuration parameters:

```rust
struct ServerConfig {
    host: String,
    port: u16,
    timeout_seconds: u64,
    max_connections: u32,
}
```

**Why Use Structs Instead of Individual Parameters?**

**Without struct** (parameter soup):
```rust
fn start_server(
    host: String,
    port: u16,
    timeout: u64,
    max_conn: u32,
    buffer_size: usize,
    threads: usize,
    // ... 20 more parameters ...
) { }

// Easy to swap parameters accidentally:
start_server(
    "localhost".to_string(),
    30,      // Oops! Passed timeout as port
    8080,    // Oops! Passed port as timeout
    100,
    4096,
    4,
);
```

**With struct** (grouped data):
```rust
fn start_server(config: ServerConfig) { }

let config = ServerConfig {
    host: "localhost".to_string(),
    port: 8080,
    timeout_seconds: 30,
    max_connections: 100,
};
start_server(config); // Named fields prevent mistakes
```

**Benefits of Basic Structs**:

1. **Named fields**: `config.port` is clearer than function parameter #2
2. **Single unit**: Pass one `ServerConfig` instead of 10 parameters
3. **Default values**: Can implement `Default` trait
4. **Extensibility**: Add fields without breaking function signatures
5. **Documentation**: Struct definition documents all configuration options

**The Problems We'll Discover**:

1. **No validation**: Can create `ServerConfig { port: 0, ... }` (invalid!)
2. **No type safety**: Both `host` and `database_url` are `String` (easy to mix up!)
3. **No semantic meaning**: Is `timeout_seconds` really seconds? Could be milliseconds!
4. **Public fields**: Users can create invalid configurations directly

**Real-World Analogy**:

Think of this like a paper form:
- **Milestone 1**: Blank form with labeled boxes (can write anything in any box)
- **Milestone 2**: Form with validation rules (postal code must be 5 digits)
- **Milestone 3**: Digital form with smart defaults and helpful error messages

**Memory Layout**:

```rust
ServerConfig:
  host: String          [24 bytes: ptr + len + capacity]
  port: u16            [2 bytes]
  timeout_seconds: u64 [8 bytes]
  max_connections: u32 [4 bytes]
  [+ 6 bytes padding for alignment]
Total: ~44 bytes
```

No runtime overhead for grouping—just the sum of field sizes plus alignment.

**Key Learning Points**:

- **Structs are product types**: Contain all fields simultaneously (AND relationship)
- **Field naming**: Makes code self-documenting
- **Ownership**: Struct owns its fields
- **Move semantics**: Moving struct moves all fields

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

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

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

**Why This Milestone Matters**:

This is where Rust's **zero-cost abstractions** shine. The newtype pattern lets us add compile-time safety without any runtime overhead—it's pure compile-time magic!

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

**The Newtype Pattern Explained**:

A newtype is a tuple struct with a single field:

```rust
struct Port(u16);  // New type wrapping u16

impl Port {
    fn new(port: u16) -> Result<Self, String> {
        if port == 0 {
            Err("Port must be > 0".to_string())
        } else {
            Ok(Port(port))  // Wrap the validated value
        }
    }

    fn get(&self) -> u16 {
        self.0  // Access tuple field
    }
}
```

**Why Use `NonZeroU32`?**

`std::num::NonZeroU32` is a standard library type that **guarantees** non-zero at the type level:

```rust
// Without NonZeroU32 - runtime check every time
struct MaxConnections(u32);
impl MaxConnections {
    fn get(&self) -> u32 {
        assert!(self.0 > 0);  // Runtime check!
        self.0
    }
}

// With NonZeroU32 - guaranteed by type system
struct MaxConnections(NonZeroU32);
impl MaxConnections {
    fn get(&self) -> u32 {
        self.0.get()  // No check needed!
    }
}
```

**Compiler optimizations**: `Option<NonZeroU32>` is same size as `u32` (niche optimization)!

**Smart Constructors**:

Newtypes use **smart constructors** that validate inputs:

```rust
impl Port {
    fn new(port: u16) -> Result<Self, String> {
        // Validation logic here
        if port == 0 {
            Err("Port must be greater than 0".to_string())
        } else {
            Ok(Port(port))
        }
    }
}
```

This ensures **invalid values can't be constructed**—the only way to get a `Port` is through the validated `new` function.

**Type Safety in Action**:

```rust
// Before: These are all u16, compiler treats them identically
let port: u16 = 8080;
let pool_size: u16 = 100;
start_server(pool_size, port);  // Oops! Swapped, but compiles

// After: These are distinct types
let port: Port = Port::new(8080).unwrap();
let pool_size: PoolSize = PoolSize::new(100).unwrap();
start_server(pool_size, port);  // ❌ Compile error!
```

**Zero Runtime Cost**:

Newtype wrappers have **zero memory overhead**:

```rust
assert_eq!(
    std::mem::size_of::<u16>(),
    std::mem::size_of::<Port>()
);  // Both are 2 bytes!
```

The wrapper only exists at compile-time. At runtime, `Port(8080)` is just `8080` in memory.

**Pattern Matching Works**:

```rust
let port = Port::new(8080).unwrap();

match port {
    Port(8080) => println!("Standard HTTP"),
    Port(443) => println!("HTTPS"),
    Port(p) => println!("Custom port: {}", p),
}
```

**Real-World Examples**:

1. **Rust compiler**: `Symbol`, `DefId`, `NodeId` - all just integers wrapped in newtypes
2. **AWS SDK**: `BucketName`, `RegionName`, `InstanceId` - prevent mixing identifiers
3. **Databases**: `TableName`, `ColumnName`, `IndexName` - type-safe schema references
4. **Web frameworks**: `PathBuf` vs `AssetPath` vs `TemplatePath` - same underlying type, different semantics

**Common Newtype Use Cases**:

- **Units**: `Meters(f64)`, `Kilograms(f64)` - prevent mixing units
- **IDs**: `UserId(u64)`, `ProductId(u64)` - prevent mixing ID types
- **Handles**: `FileDescriptor(i32)`, `SocketHandle(u32)` - type-safe resource handles
- **Validated strings**: `Email(String)`, `Url(String)` - guarantee format validity
- **Currencies**: `USD(f64)`, `EUR(f64)` - prevent mixing monetary values

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

impl Timeout {
    fn from_secs(secs: u64) -> Result<Self, String> {
        // Positivity check
        if secs == 0 {
            return Err("Timeout must be > 0 seconds".to_string());
        }
        Ok(Timeout(Duration::from_secs(secs)))
    }
}

impl MaxConnections {
    fn new(count: u32) -> Result<Self, String> {
        // Use type system for validation
        NonZeroU32::new(count)
            .map(MaxConnections)
            .ok_or_else(|| "Connection count must be > 0".to_string())
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

**Check Your Understanding**:
- Why can't you accidentally swap `Port` and `MaxConnections` now?
- What happens at compile-time if you try `Port::new(8080).unwrap().as_duration()`?
- Why use `NonZeroU32` instead of validating `u32 > 0` manually?
- What's the memory overhead of these newtypes? (Hint: zero!)

---

### Why Milestone 2 Isn't Enough → Moving to Milestone 3

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

Milestone 2 gave us type safety, but the API is **clunky**. Creating a config requires many `.unwrap()` calls and looks ugly. The **builder pattern** solves this by providing a fluent, chainable API that's pleasant to use while maintaining all our safety guarantees.

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

## Project 2: Order Processing State Machine with Enums

### Problem Statement

Build a type-safe order processing system that uses enums to model state transitions. You'll start with basic enum variants, add exhaustive pattern matching for state transitions, then implement compile-time state checking using the typestate pattern.

### Why It Matters

**Real-World Impact**: State management bugs are expensive:

**The State Confusion Problem**:
- **Amazon (2006)**: Order processing bug caused incorrect shipments, millions in losses
- **Payment processors**: Process same payment twice due to state confusion
- **E-commerce**: Ship orders that were cancelled, refund orders not yet paid
- **Booking systems**: Double-book resources, allow modifications after confirmation

**Without Type-Safe States**:
```rust
struct Order {
    id: u64,
    state: String,  // "pending", "paid", "shipped"???
    tracking_number: Option<String>,
    payment_id: Option<String>,
}

fn ship_order(order: &mut Order) {
    // Runtime checks everywhere!
    if order.state == "paid" {  // String comparison prone to typos
        order.tracking_number = Some(generate_tracking());
        order.state = "shipped".to_string();  // Typo: "shiped"?
    }
    // What if state was "cancelled"? Silent failure!
}
```

**With Enum State Machine**:
```rust
enum OrderState {
    Pending { items: Vec<Item> },
    Paid { payment_id: String },
    Shipped { tracking: String },
    Cancelled { reason: String },
}

impl OrderState {
    fn ship(self) -> Result<OrderState, String> {
        match self {
            OrderState::Paid { payment_id } => {
                Ok(OrderState::Shipped {
                    tracking: generate_tracking()
                })
            }
            _ => Err("Can only ship paid orders")
        }
    }
}
```

**Performance & Safety Benefits**:
- **Exhaustive matching**: Compiler forces handling of all states
- **Impossible states**: Can't have both `tracking_number` and be unpaid
- **Zero runtime cost**: Enum same size as largest variant + 1-byte discriminant
- **Self-documenting**: All valid states visible in type definition

**Real Production Examples**:
- **Payment gateways**: Pending → Authorized → Captured → Settled states
- **Workflow engines**: Draft → Review → Approved → Published transitions
- **Connection pools**: Idle → Active → Closing → Closed states
- **HTTP clients**: Connecting → Connected → Reading → Complete states

### Use Cases

**When you need this pattern**:
1. **Order/Payment processing**: Pending → Paid → Shipped → Delivered
2. **Document workflows**: Draft → Review → Approved → Published
3. **Network connections**: Connecting → Connected → Closing → Closed
4. **Game states**: Menu → Playing → Paused → GameOver
5. **User authentication**: Anonymous → LoggedIn → Verified → Admin
6. **File uploads**: Validating → Uploading → Processing → Complete

**Enum State Machines Prevent**:
- Shipping unpaid orders
- Charging paid orders twice
- Cancelling already shipped orders
- Accessing data from wrong state
- Forgetting to handle edge cases

### Learning Goals

- Use enums to model state machines with exhaustive matching
- Implement state transitions that consume and transform states
- Understand pattern matching for compile-time guarantees
- Build typestate pattern for impossible-states-as-unrepresentable
- Compare runtime state checking vs compile-time state checking
- Handle associated data per state variant

---

### Milestone 1: Basic Order Enum with States

**Goal**: Define an enum representing different order states, where each variant carries state-specific data.

**Why This Milestone Matters**:

This milestone introduces **enums as state machines**—one of Rust's most powerful patterns. Unlike structs (which group related data), enums represent **alternatives**—a value is exactly one variant at any time.

**Structs vs Enums**:

```rust
// Struct: Has ALL fields at once (AND)
struct User {
    name: String,     // AND
    email: String,    // AND
    age: u32,        // AND
}

// Enum: Is EXACTLY ONE variant (OR)
enum LoginState {
    Anonymous,              // OR
    LoggedIn { user_id: u64 },  // OR
    Admin { user_id: u64, permissions: Vec<String> },  // OR
}
```

**The Problem We're Solving**:

**Bad approach** (struct with optional fields):
```rust
struct Order {
    id: u64,
    state: String,  // "pending", "paid", "shipped"
    items: Option<Vec<Item>>,         // Only in pending
    payment_id: Option<String>,       // Only in paid/shipped
    tracking_number: Option<String>,  // Only in shipped
}
```

**Problems**:
1. **Impossible states are possible**: What if `payment_id` is Some but `state` is "pending"?
2. **Runtime checks everywhere**: Must check `state` string before accessing fields
3. **Easy to forget**: Nothing forces you to handle all states
4. **Typos**: `state == "shiped"` compiles but is wrong!

**Good approach** (enum with variants):
```rust
enum OrderState {
    Pending {
        items: Vec<Item>,
        customer_id: u64,
    },
    Paid {
        order_id: u64,
        payment_id: String,
        amount: f64,
    },
    Shipped {
        order_id: u64,
        tracking_number: String,
    },
}
```

**Benefits**:
1. **Impossible states impossible**: Can't have `payment_id` without being `Paid`
2. **Exhaustive matching**: Compiler forces handling all variants
3. **Type-safe**: Typos in variant names caught at compile-time
4. **Self-documenting**: All states visible in type definition

**What We're Building**:

Five order states representing the complete order lifecycle:

1. **`Pending`**: Order created, awaiting payment
   - Contains: `items`, `customer_id`
   - Can transition to: `Paid`, `Cancelled`

2. **`Paid`**: Payment received, awaiting shipment
   - Contains: `order_id`, `payment_id`, `amount`
   - Can transition to: `Shipped`, `Cancelled`

3. **`Shipped`**: Package shipped, in transit
   - Contains: `order_id`, `tracking_number`
   - Can transition to: `Delivered`

4. **`Delivered`**: Package delivered to customer
   - Contains: `order_id`, `delivered_at`
   - Terminal state (no further transitions)

5. **`Cancelled`**: Order cancelled
   - Contains: `order_id`, `reason`
   - Terminal state

**Enum Syntax**:

```rust
enum OrderState {
    // Unit variant (no data)
    Simple,

    // Tuple variant (unnamed fields)
    WithData(String, u64),

    // Struct variant (named fields) - most common
    WithNamedData {
        field1: String,
        field2: u64,
    },
}
```

We use **struct variants** for clarity—named fields are self-documenting.

**Pattern Matching**:

The primary way to work with enums is pattern matching:

```rust
let order = OrderState::Pending {
    items: vec![...],
    customer_id: 123,
};

match order {
    OrderState::Pending { items, customer_id } => {
        println!("Order for customer {} has {} items", customer_id, items.len());
    }
    OrderState::Paid { payment_id, amount, .. } => {
        println!("Payment {} for ${}", payment_id, amount);
    }
    OrderState::Shipped { tracking_number, .. } => {
        println!("Shipped: {}", tracking_number);
    }
    OrderState::Delivered { .. } => {
        println!("Delivered!");
    }
    OrderState::Cancelled { reason, .. } => {
        println!("Cancelled: {}", reason);
    }
}
```

**Exhaustive Matching**:

The compiler **forces** you to handle all variants:

```rust
match order {
    OrderState::Pending { .. } => { },
    OrderState::Paid { .. } => { },
    // ❌ Compile error: missing Shipped, Delivered, Cancelled!
}
```

This prevents bugs where you forget to handle a case.

**Memory Layout**:

Enums use a **discriminant** (tag) to track which variant is active:

```rust
enum OrderState {
    Pending { items: Vec<Item>, customer_id: u64 },  // 32 bytes
    Paid { order_id: u64, payment_id: String, amount: f64 },  // 40 bytes
    Shipped { order_id: u64, tracking_number: String },  // 32 bytes
}
```

**Memory size**: `max(all variants) + discriminant`
- Largest variant: `Paid` (40 bytes)
- Discriminant: 1 byte (actually 8 bytes with alignment)
- **Total**: ~48 bytes

All variants share the same memory space. Only one is active at a time.

**Why Use Enum for State Machines?**

**State machines** have:
1. **Finite states**: Known, fixed set of states
2. **Transitions**: Rules for moving between states
3. **State-specific data**: Each state needs different information
4. **Exclusive states**: Can't be in two states at once

Enums are **perfect** for this! Each variant = one state, pattern matching = transitions.

**Real-World State Machine Examples**:

1. **HTTP connections**:
   ```rust
   enum Connection {
       Idle,
       Connecting { start_time: Instant },
       Connected { socket: TcpStream },
       Closing,
       Closed,
   }
   ```

2. **File uploads**:
   ```rust
   enum Upload {
       Pending { file_name: String, size: u64 },
       Uploading { bytes_sent: u64, total: u64 },
       Processing { job_id: String },
       Complete { url: String },
       Failed { error: String },
   }
   ```

3. **Game states**:
   ```rust
   enum GameState {
       MainMenu,
       Playing { level: u32, score: u64 },
       Paused { saved_state: Box<PlayingState> },
       GameOver { final_score: u64 },
   }
   ```

**Common Methods on Enums**:

```rust
impl OrderState {
    // Query method - doesn't consume self
    fn status_string(&self) -> &str {
        match self {
            OrderState::Pending { .. } => "Pending",
            OrderState::Paid { .. } => "Paid",
            // ...
        }
    }

    // Check variant type
    fn is_paid(&self) -> bool {
        matches!(self, OrderState::Paid { .. })
    }

    // Extract data (if specific variant)
    fn get_tracking_number(&self) -> Option<&str> {
        match self {
            OrderState::Shipped { tracking_number, .. } => Some(tracking_number),
            _ => None,
        }
    }
}
```

**Key Design Principle: Make Illegal States Unrepresentable**:

With enums, you **can't create** invalid combinations:

```rust
// ❌ Can't do this with enums!
let order = OrderState::Pending {
    items: vec![],
    payment_id: "PAY123".to_string(),  // ❌ Pending has no payment_id field!
};

// ❌ Can't have two variants at once!
let order = OrderState::Paid { .. } | OrderState::Shipped { .. };  // ❌ Not possible!
```

Compare to the struct approach where these are possible (and bugs)!

**Checkpoint Tests**:
```rust
#[test]
fn test_create_pending_order() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];
    let order = OrderState::new_pending(items, 123);

    assert_eq!(order.status_string(), "Pending");
}

#[test]
fn test_all_states() {
    // TODO: Test creating each state variant
    let pending = OrderState::Pending {
        items: vec![],
        customer_id: 1,
    };

    let paid = OrderState::Paid {
        order_id: 1,
        payment_id: "pay_123".to_string(),
        amount: 99.99,
    };

    // Test pattern matching works
    match pending {
        OrderState::Pending { .. } => { /* OK */ }
        _ => panic!("Should be pending"),
    }
}
```

**Starter Code**:
```rust
use std::time::Instant;

// Item: Represents a product in an order
// Role: Stores product details for order line items
#[derive(Debug, Clone)]
struct Item {
    product_id: u64,  // Unique identifier for the product
    name: String,     // Product display name
    price: f64,       // Product price in dollars
}

// OrderState: Enum representing all possible order states
// Role: Type-safe state machine where each variant has state-specific data
// TODO: Define variants:
// - Pending { items: Vec<Item>, customer_id: u64 }
// - Paid { order_id: u64, payment_id: String, amount: f64 }
// - Shipped { order_id: u64, tracking_number: String }
// - Delivered { order_id: u64, delivered_at: Instant }
// - Cancelled { order_id: u64, reason: String }
#[derive(Debug, Clone)]
enum OrderState {
    // TODO: Add variants here
}

impl OrderState {
    // new_pending: Creates a new order in Pending state
    // Role: Constructor for initial order state with items and customer
    fn new_pending(items: Vec<Item>, customer_id: u64) -> Self {
        // TODO: Create Pending variant
        todo!()
    }

    // status_string: Returns human-readable status
    // Role: Provides string representation of current state for display
    fn status_string(&self) -> &str {
        // TODO: Match on self and return appropriate status string
        // Hint: "Pending", "Paid", "Shipped", "Delivered", "Cancelled"
        todo!()
    }
}
```

**Check Your Understanding**:
- Why does each variant have different associated data?
- What prevents you from accessing `payment_id` on a `Pending` order?
- How does this compare to having optional fields on a single struct?

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitations**:
1. **No state transitions**: Can create any state, but can't safely transition between them
2. **Validation missing**: Nothing prevents creating `Paid` order with negative amount
3. **No business rules**: Could go directly from Pending to Delivered, skipping payment
4. **Manual state checking**: Users must pattern match everywhere to get data

**What we're adding**: **State transition methods** that:
- Consume the current state and return a new state
- Enforce valid transitions (can't ship before paying)
- Validate business rules (can't cancel after delivery)
- Use `Result` for error handling

**Improvements**:
- **Type-safe transitions**: `order.pay()` consumes `Pending`, returns `Paid`
- **Exhaustive matching**: Compiler ensures all current states handled
- **Business logic**: Validation happens in transition methods
- **Self-documenting**: Method names show valid transitions

---

### Milestone 2: State Transitions with Pattern Matching

**Goal**: Implement methods that transition between states with validation, using pattern matching to consume states and enforce valid transitions.

**Why This Milestone Matters**:

Milestone 1 gave us **type-safe state representation**, but states just sit there—we can create any state directly! This milestone adds **controlled transitions**—the only way to move from `Pending` to `Paid` is through the `pay()` method, which enforces business rules.

**The Uncontrolled State Problem**:

```rust
// Milestone 1 - can create any state directly!
let order = OrderState::Paid {
    order_id: 1,
    payment_id: "PAY_123".to_string(),
    amount: 99.99,  // Who validated this payment actually happened?
};

// Or worse - manually manipulate states
let mut order = OrderState::Pending { items, customer_id };
order = OrderState::Shipped {  // Skip payment entirely!
    order_id: 1,
    tracking_number: "TRACK_123".to_string(),
};
```

**Problems**:
1. **No validation**: Can create `Paid` state without actually processing payment
2. **Skip steps**: Can go directly from `Pending` to `Shipped`, bypassing payment
3. **No business logic**: Amount calculation, inventory checks, fraud detection—all missing
4. **Manual construction**: Easy to forget required fields or use wrong data

**The Controlled Transition Solution**:

```rust
impl OrderState {
    // Only way to go from Pending to Paid
    fn pay(self, payment_id: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { items, customer_id } => {
                // ✅ Validate items
                if items.is_empty() {
                    return Err("Cannot pay for empty order".to_string());
                }

                // ✅ Calculate total
                let amount: f64 = items.iter().map(|i| i.price).sum();

                // ✅ Process payment (in real system)
                // payment_processor.charge(payment_id, amount)?;

                // ✅ Transition to Paid state
                Ok(OrderState::Paid {
                    order_id: customer_id,
                    payment_id,
                    amount,
                })
            }
            // ✅ Reject invalid transitions
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }
}

// Now the only way to get Paid state:
let order = OrderState::new_pending(items, 123);
let order = order.pay("PAY_123".to_string())?;  // Validated!
```

**What We're Building**:

Four transition methods representing the order lifecycle:

1. **`pay(self, payment_id) -> Result<OrderState, String>`**
   - Consumes: `Pending`
   - Produces: `Paid` or error
   - Validates: Items not empty, calculates total amount
   - Business rule: Must have items to pay for

2. **`ship(self, tracking_number) -> Result<OrderState, String>`**
   - Consumes: `Paid`
   - Produces: `Shipped` or error
   - Business rule: Can't ship unpaid orders

3. **`deliver(self) -> Result<OrderState, String>`**
   - Consumes: `Shipped`
   - Produces: `Delivered` or error
   - Adds: Delivery timestamp
   - Business rule: Can't deliver unshipped orders

4. **`cancel(self, reason) -> Result<OrderState, String>`**
   - Consumes: `Pending` or `Paid` only
   - Produces: `Cancelled` or error
   - Business rule: Can't cancel after shipping

**Why Take `self` (Ownership)?**

This is **crucial** for state machine safety:

```rust
// Taking self by value (ownership)
fn pay(self, payment_id: String) -> Result<OrderState, String> {
    // self is consumed here
}

// Usage:
let pending_order = OrderState::Pending { ... };
let paid_order = pending_order.pay("PAY_123".to_string())?;
// pending_order is gone - can't use it again! ✅

// Compare to taking &self (borrow):
fn pay(&self, payment_id: String) -> Result<OrderState, String> {
    // self still exists after this call
}

// Usage:
let pending_order = OrderState::Pending { ... };
let paid_order = pending_order.pay("PAY_123".to_string())?;
pending_order.pay("PAY_456".to_string())?;  // ❌ Can pay twice!
```

**Consuming transitions** prevent:
- Using old states after transition
- Paying for same order twice
- Concurrent access to transitioning state
- Forgetting to use the new state

**The Power of Exhaustive Matching**:

```rust
fn ship(self, tracking_number: String) -> Result<Self, String> {
    match self {
        OrderState::Paid { order_id, .. } => {
            // Only valid transition
            Ok(OrderState::Shipped { order_id, tracking_number })
        }
        // Compiler forces you to handle ALL other states!
        _ => Err("Can only ship paid orders".to_string()),
    }
}
```

If you forget a variant:
```rust
match self {
    OrderState::Paid { .. } => { /* OK */ }
    OrderState::Pending { .. } => { /* OK */ }
    // ❌ Compile error: missing Shipped, Delivered, Cancelled!
}
```

**Pattern Matching for State Extraction**:

```rust
fn pay(self, payment_id: String) -> Result<Self, String> {
    match self {
        // Extract data from Pending variant
        OrderState::Pending { items, customer_id } => {
            // items and customer_id are now available
            let amount = items.iter().map(|i| i.price).sum();
            Ok(OrderState::Paid {
                order_id: customer_id,
                payment_id,
                amount,
            })
        }
        // Match all other variants
        _ => Err("Can only pay for pending orders".to_string()),
    }
}
```

**Multi-Variant Matching with `|`**:

```rust
fn cancel(self, reason: String) -> Result<Self, String> {
    match self {
        // Cancel allowed from EITHER Pending OR Paid
        OrderState::Pending { customer_id, .. } | OrderState::Paid { order_id: customer_id, .. } => {
            Ok(OrderState::Cancelled {
                order_id: customer_id,
                reason,
            })
        }
        // Not allowed after shipping
        OrderState::Shipped { .. } | OrderState::Delivered { .. } => {
            Err("Cannot cancel after shipping".to_string())
        }
        OrderState::Cancelled { .. } => {
            Err("Already cancelled".to_string())
        }
    }
}
```

**Validation Strategy**:

Each transition method performs three tasks:

1. **Validate current state** (via pattern matching)
   ```rust
   match self {
       OrderState::Paid { .. } => { /* OK */ }
       _ => return Err("Wrong state".to_string()),
   }
   ```

2. **Validate business rules** (within the match arm)
   ```rust
   if items.is_empty() {
       return Err("No items".to_string());
   }
   let amount: f64 = items.iter().map(|i| i.price).sum();
   if amount <= 0.0 {
       return Err("Invalid amount".to_string());
   }
   ```

3. **Produce new state** (return new variant)
   ```rust
   Ok(OrderState::Paid {
       order_id: generate_id(),
       payment_id,
       amount,
   })
   ```

**Real-World State Transition Examples**:

1. **Payment processing**:
   ```rust
   enum PaymentState {
       Created { amount: f64 },
       Authorized { auth_code: String },
       Captured { transaction_id: String },
       Refunded { refund_id: String },
   }

   impl PaymentState {
       fn authorize(self, auth_code: String) -> Result<Self, String> {
           match self {
               PaymentState::Created { amount } => {
                   // Call payment gateway
                   Ok(PaymentState::Authorized { auth_code })
               }
               _ => Err("Can only authorize created payments"),
           }
       }
   }
   ```

2. **Document workflow**:
   ```rust
   enum DocumentState {
       Draft { content: String },
       InReview { content: String, reviewers: Vec<String> },
       Approved { content: String, approver: String },
       Published { url: String },
   }

   impl DocumentState {
       fn submit_for_review(self, reviewers: Vec<String>) -> Result<Self, String> {
           match self {
               DocumentState::Draft { content } => {
                   if content.is_empty() {
                       return Err("Cannot submit empty document");
                   }
                   Ok(DocumentState::InReview { content, reviewers })
               }
               _ => Err("Can only submit drafts for review"),
           }
       }
   }
   ```

3. **Connection lifecycle**:
   ```rust
   enum Connection {
       Idle,
       Connecting { address: String },
       Connected { socket: TcpStream },
       Closing,
       Closed { reason: String },
   }

   impl Connection {
       fn connect(self, address: String) -> Result<Self, IoError> {
           match self {
               Connection::Idle => {
                   // Initiate connection
                   Ok(Connection::Connecting { address })
               }
               _ => Err(IoError::new(ErrorKind::Other, "Already connecting")),
           }
       }
   }
   ```

**Error Handling Design**:

```rust
// Return Result for transitions that can fail
fn pay(self, payment_id: String) -> Result<Self, String> {
    match self {
        OrderState::Pending { items, .. } => {
            if items.is_empty() {
                // Business rule violation
                return Err("Cannot pay for order with no items".to_string());
            }
            // Success case
            Ok(OrderState::Paid { /* ... */ })
        }
        // State violation
        _ => Err("Can only pay for pending orders".to_string()),
    }
}

// Usage with ? operator for error propagation
fn process_order(order: OrderState) -> Result<OrderState, String> {
    let order = order.pay("PAY_123".to_string())?;  // Propagate error
    let order = order.ship("TRACK_ABC".to_string())?;
    let order = order.deliver()?;
    Ok(order)
}
```

**Helper Methods for State Queries**:

```rust
impl OrderState {
    // Check if in specific state (doesn't consume)
    fn is_paid(&self) -> bool {
        matches!(self, OrderState::Paid { .. })
    }

    // Check if transition is allowed (doesn't consume)
    fn can_cancel(&self) -> bool {
        matches!(
            self,
            OrderState::Pending { .. } | OrderState::Paid { .. }
        )
    }

    // Get status string for display
    fn status_string(&self) -> &str {
        match self {
            OrderState::Pending { .. } => "Pending",
            OrderState::Paid { .. } => "Paid",
            OrderState::Shipped { .. } => "Shipped",
            OrderState::Delivered { .. } => "Delivered",
            OrderState::Cancelled { .. } => "Cancelled",
        }
    }
}
```

**Memory and Performance**:

- **No allocation overhead**: Transitions just move data, no extra allocations
- **No copying**: `self` moved by value, not copied
- **Same memory size**: `OrderState` always `max(variants) + discriminant`
- **Stack-based**: Entire state machine lives on stack

**State Machine Guarantees**:

✅ **Compile-time guarantees**:
- All variants handled in match (exhaustiveness)
- Type-correct data in each variant
- Can't create variant without required fields

✅ **Runtime guarantees** (via transitions):
- Can't skip payment and go straight to shipped
- Can't pay for empty order
- Can't cancel after shipping
- Can't pay for same order twice (consuming `self`)

❌ **Still possible** (will fix in Milestone 3):
- Calling `order.ship()` on `Pending` order (returns `Err` at runtime)
- IDE shows all methods on all states (no compile-time filtering)
- Can store mixed states in collections but lose type info

**Checkpoint Tests**:
```rust
#[test]
fn test_valid_transitions() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let order = OrderState::new_pending(items, 123);
    let order = order.pay("payment_123".to_string()).unwrap();
    assert_eq!(order.status_string(), "Paid");

    let order = order.ship("TRACK123".to_string()).unwrap();
    assert_eq!(order.status_string(), "Shipped");

    let order = order.deliver().unwrap();
    assert_eq!(order.status_string(), "Delivered");
}

#[test]
fn test_invalid_transitions() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let order = OrderState::new_pending(items, 123);

    // Can't ship before paying
    assert!(order.clone().ship("TRACK123".to_string()).is_err());

    // Can pay
    let order = order.pay("payment_123".to_string()).unwrap();

    // Can't pay again
    assert!(order.clone().pay("payment_456".to_string()).is_err());
}

#[test]
fn test_cancellation_rules() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    // Can cancel pending order
    let order = OrderState::new_pending(items.clone(), 123);
    assert!(order.can_cancel());
    assert!(order.cancel("Customer request".to_string()).is_ok());

    // Can cancel paid order
    let order = OrderState::new_pending(items.clone(), 123)
        .pay("payment_123".to_string())
        .unwrap();
    assert!(order.can_cancel());

    // Cannot cancel shipped order
    let order = OrderState::new_pending(items, 123)
        .pay("payment_123".to_string())
        .unwrap()
        .ship("TRACK123".to_string())
        .unwrap();
    assert!(!order.can_cancel());
    assert!(order.cancel("Too late".to_string()).is_err());
}
```

**Starter Code**:
```rust
impl OrderState {
    // pay: Transitions from Pending to Paid state
    // Role: Processes payment, validates items, calculates total
    // Consumes Pending state, returns Paid state or error
    fn pay(self, payment_id: String) -> Result<Self, String> {
        // Match on current state to enforce valid transitions
        match self {
            OrderState::Pending { items, customer_id } => {
                // TODO: Validate items not empty
                // TODO: Calculate total amount by summing item prices
                // TODO: Return Paid variant with order_id, payment_id, amount
                // Hint: Generate order_id from customer_id (e.g., customer_id as order_id)
                todo!()
            }
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }

    // ship: Transitions from Paid to Shipped state
    // Role: Records shipment with tracking number
    // Consumes Paid state, returns Shipped state or error
    fn ship(self, tracking_number: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Paid: extract order_id, return Shipped with order_id and tracking_number
        // - Otherwise: return Err("Can only ship paid orders")
        todo!()
    }

    // deliver: Transitions from Shipped to Delivered state
    // Role: Marks order as delivered with timestamp
    // Consumes Shipped state, returns Delivered state or error
    fn deliver(self) -> Result<Self, String> {
        // TODO: Match on self
        // - If Shipped: extract order_id, return Delivered with order_id and Instant::now()
        // - Otherwise: return Err("Can only deliver shipped orders")
        todo!()
    }

    // cancel: Transitions to Cancelled state (only from Pending/Paid)
    // Role: Cancels order with reason, enforces business rules
    // Consumes current state, returns Cancelled state or error
    fn cancel(self, reason: String) -> Result<Self, String> {
        // TODO: Match on self
        // - If Pending or Paid: extract order_id (or use customer_id), return Cancelled
        // - If Shipped or Delivered: return Err("Cannot cancel after shipping")
        // Hint: Use | to match multiple variants: OrderState::Pending { .. } | OrderState::Paid { .. }
        todo!()
    }

    // can_cancel: Checks if order can be cancelled
    // Role: Query method that doesn't consume state
    fn can_cancel(&self) -> bool {
        // TODO: Return true only for Pending or Paid states
        // Hint: Use matches! macro: matches!(self, OrderState::Pending { .. } | OrderState::Paid { .. })
        todo!()
    }
}
```

**Check Your Understanding**:
- Why do transition methods take `self` (ownership) instead of `&self`?
- How does the compiler help you handle all possible current states?
- What happens if you forget to handle a variant in a match?
- Why is returning `Result` better than panicking on invalid transitions?

---

### 🔄 Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Remaining Issues**:
1. **Runtime checks**: Still possible to call wrong method at runtime, just returns `Err`
2. **API not self-documenting**: IDE doesn't show which methods available for current state
3. **Enum still mutable**: Someone could manually construct invalid transitions
4. **No compile-time guarantees**: `order.ship()` compiles even on `Pending` order

**What we're adding**: **Typestate pattern** - use the type system to encode states:
- Each state is a separate type: `struct Pending`, `struct Paid`, etc.
- Generic wrapper: `Order<State>` where `State` is the current state type
- Methods only available on appropriate state: `Order<Pending>` can't call `ship()`
- Transitions return new type: `pay()` consumes `Order<Pending>`, returns `Order<Paid>`

**Improvements**:
- **Compile-time checking**: `pending_order.ship()` doesn't compile!
- **Zero runtime cost**: State stored in type, not value
- **IDE support**: Autocomplete only shows valid methods for current state
- **Impossible states impossible**: Can't have `Order<Shipped>` without going through payment

**Trade-offs**:
- **More complex**: More types and trait bounds
- **Less dynamic**: Can't store mixed states in Vec without trait objects
- **Worth it when**: State transitions known at compile-time

---

### Milestone 3: Typestate Pattern for Compile-Time Safety

**Goal**: Use phantom types to encode states in the type system, making invalid state transitions impossible to compile rather than just returning errors at runtime.

**Why This Milestone Matters**:

Milestone 2 gave us **runtime safety** through `Result` types, but the type system doesn't help—`order.ship()` compiles even on a `Pending` order, it just returns `Err` at runtime. The **typestate pattern** moves state checking from runtime to **compile-time**—invalid transitions won't even compile!

**The Runtime Checking Problem**:

```rust
// Milestone 2 - compiles but fails at runtime
let pending_order = OrderState::Pending { items, customer_id };

// ❌ This compiles! (but returns Err at runtime)
pending_order.ship("TRACK_123".to_string())?;
//              ^^^^
// IDE shows 'ship' as available method
// Compiler allows this
// Only fails when you run the code
```

**Problems with runtime checking**:
1. **Late error detection**: Find bugs when testing, not when coding
2. **Poor IDE support**: Autocomplete shows all methods on all states
3. **Defensive programming**: Must handle all `Result::Err` cases
4. **Lost type information**: `Vec<OrderState>` loses which specific state each order is in
5. **Runtime cost**: Every transition checks state discriminant

**The Compile-Time Solution: Typestate Pattern**:

```rust
// Different type for each state!
struct Pending;
struct Paid;
struct Shipped;

// Generic struct parameterized by state
struct Order<State> {
    id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,  // Zero-sized marker
}

// Only Pending orders have pay() method
impl Order<Pending> {
    fn pay(self, payment_id: String) -> Result<Order<Paid>, String> {
        // Consumes Order<Pending>, returns Order<Paid>
        Ok(Order { id: self.id, items: self.items, _state: PhantomData })
    }
}

// Only Paid orders have ship() method
impl Order<Paid> {
    fn ship(self, tracking: String) -> Order<Shipped> {
        // Consumes Order<Paid>, returns Order<Shipped>
        Order { id: self.id, items: self.items, _state: PhantomData }
    }
}

// Now this won't compile!
let pending = Order::<Pending>::new(1, items)?;
pending.ship("TRACK_123".to_string());  // ❌ Compile error!
//      ^^^^ method not found in `Order<Pending>`

// Must go through pay() first
let paid = pending.pay("PAY_123".to_string())?;  // Order<Paid>
let shipped = paid.ship("TRACK_123".to_string()); // Order<Shipped> ✅
```

**What We're Building**:

Five marker types and one generic struct:

**State Markers** (zero-sized types):
```rust
struct Pending;    // 0 bytes
struct Paid;       // 0 bytes
struct Shipped;    // 0 bytes
struct Delivered;  // 0 bytes
struct Cancelled;  // 0 bytes
```

**Generic Order**:
```rust
struct Order<State> {
    id: u64,
    customer_id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,  // Zero-sized!
}
```

**Memory layout**: All `Order<State>` variants are **exactly the same size**!
```rust
assert_eq!(
    std::mem::size_of::<Order<Pending>>(),
    std::mem::size_of::<Order<Paid>>(),
);  // Both same size! State is compile-time only
```

**What is `PhantomData<State>`?**

`PhantomData` is a **zero-sized type** that tells the compiler "this struct owns a `State` type, even though we don't actually store it":

```rust
use std::marker::PhantomData;

struct Order<State> {
    id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,  // "Pretend" we have a State
}

// Why PhantomData is needed:
struct OrderBroken<State> {  // ❌ Error: parameter `State` is never used
    id: u64,
    items: Vec<Item>,
}

struct OrderFixed<State> {   // ✅ OK: State appears in PhantomData
    id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,
}
```

**PhantomData properties**:
- **Size**: 0 bytes (optimized away at compile-time)
- **Purpose**: Make generic parameter `State` "used" so compiler accepts it
- **Ownership**: Tells compiler about ownership/lifetime relationships
- **Convention**: Field name starts with `_` to indicate "unused at runtime"

**State-Specific Implementations**:

Each state gets its own `impl` block with only valid transitions:

```rust
// Pending state: can pay or cancel
impl Order<Pending> {
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        // Constructor creates Order<Pending>
    }

    fn pay(self, payment_id: String) -> Result<Order<Paid>, String> {
        // Transition: Pending → Paid
    }

    fn cancel(self, reason: String) -> Order<Cancelled> {
        // Transition: Pending → Cancelled
    }

    // NO ship() method! ✅
    // NO deliver() method! ✅
}

// Paid state: can ship or cancel
impl Order<Paid> {
    fn ship(self, tracking: String) -> Order<Shipped> {
        // Transition: Paid → Shipped
    }

    fn cancel(self, reason: String) -> Order<Cancelled> {
        // Transition: Paid → Cancelled
    }

    // NO pay() method! (already paid) ✅
    // NO deliver() method! (not shipped yet) ✅
}

// Shipped state: can only deliver
impl Order<Shipped> {
    fn deliver(self) -> Order<Delivered> {
        // Transition: Shipped → Delivered
    }

    // NO cancel() method! (too late) ✅
}

// Delivered and Cancelled: terminal states (no transitions)
impl Order<Delivered> { /* no transition methods */ }
impl Order<Cancelled> { /* no transition methods */ }
```

**The Magic of Type-Based Method Availability**:

```rust
let pending: Order<Pending> = Order::new(1, items)?;

// IDE autocomplete shows:
pending.
  - new()          ✅ Available
  - pay()          ✅ Available
  - cancel()       ✅ Available
  - id()           ✅ Available (common method)
  - customer_id()  ✅ Available (common method)
  // ship() and deliver() NOT shown! ✅

let paid: Order<Paid> = pending.pay("PAY_123")?;

// IDE autocomplete shows:
paid.
  - ship()         ✅ Available
  - cancel()       ✅ Available
  - id()           ✅ Available
  - customer_id()  ✅ Available
  // pay() NOT shown! ✅ (already paid)
  // deliver() NOT shown! ✅ (not shipped yet)
```

**Common Methods Across All States**:

Use generic `impl<State>` for methods available in all states:

```rust
impl<State> Order<State> {
    // These work on Order<Pending>, Order<Paid>, Order<Shipped>, etc.
    fn id(&self) -> u64 {
        self.id
    }

    fn customer_id(&self) -> u64 {
        self.customer_id
    }

    fn items(&self) -> &[Item] {
        &self.items
    }
}

// Can call on any state:
let pending = Order::<Pending>::new(1, items)?;
println!("Order ID: {}", pending.id());  // ✅

let paid = pending.pay("PAY_123")?;
println!("Order ID: {}", paid.id());     // ✅

let shipped = paid.ship("TRACK_123");
println!("Order ID: {}", shipped.id());  // ✅
```

**Consuming Transitions for Type Safety**:

Each transition **consumes** the old state and **returns** the new state:

```rust
fn pay(self, payment_id: String) -> Result<Order<Paid>, String> {
    //  ^^^^ takes ownership
    Ok(Order {
        id: self.id,
        customer_id: self.customer_id,
        items: self.items,
        _state: PhantomData,  // New type marker!
    })
}

// Usage:
let pending = Order::<Pending>::new(1, items)?;
let paid = pending.pay("PAY_123")?;
// pending is gone! Can't use it anymore ✅

// This won't compile:
println!("{}", pending.id());  // ❌ borrow of moved value: `pending`
```

**Compile-Time Error Messages**:

When you make a mistake, the compiler tells you exactly what's wrong:

```rust
let pending = Order::<Pending>::new(1, items)?;
pending.ship("TRACK_123");

// ❌ Compile error:
// error[E0599]: no method named `ship` found for struct `Order<Pending>` in the current scope
//   --> src/main.rs:10:13
//    |
// 10 |     pending.ship("TRACK_123");
//    |             ^^^^ method not found in `Order<Pending>`
//    |
//    = note: the method is defined for `Order<Paid>`, but not for `Order<Pending>`
//    = help: consider calling `.pay()` first to transition to the Paid state
```

**Real-World Typestate Pattern Examples**:

1. **Database connections**:
   ```rust
   struct Connecting;
   struct Connected;
   struct Closed;

   struct DbConnection<State> {
       config: DbConfig,
       _state: PhantomData<State>,
   }

   impl DbConnection<Connecting> {
       fn connect() -> Result<DbConnection<Connected>, Error> { /* ... */ }
   }

   impl DbConnection<Connected> {
       fn execute(&self, query: &str) -> Result<Vec<Row>, Error> { /* ... */ }
       fn close(self) -> DbConnection<Closed> { /* ... */ }
   }

   impl DbConnection<Closed> {
       // No methods! Can't use closed connection
   }

   // Won't compile:
   let conn = DbConnection::connect()?;
   conn.close();
   conn.execute("SELECT * FROM users")?;  // ❌ conn is Closed!
   ```

2. **File handles**:
   ```rust
   struct ReadOnly;
   struct WriteOnly;
   struct ReadWrite;

   struct File<Mode> {
       handle: RawHandle,
       _mode: PhantomData<Mode>,
   }

   impl File<ReadOnly> {
       fn read(&mut self, buf: &mut [u8]) -> Result<usize> { /* ... */ }
       // NO write() method! ✅
   }

   impl File<WriteOnly> {
       fn write(&mut self, buf: &[u8]) -> Result<usize> { /* ... */ }
       // NO read() method! ✅
   }

   impl File<ReadWrite> {
       fn read(&mut self, buf: &mut [u8]) -> Result<usize> { /* ... */ }
       fn write(&mut self, buf: &[u8]) -> Result<usize> { /* ... */ }
   }
   ```

3. **HTTP request builder** (like `reqwest`):
   ```rust
   struct NoUrl;
   struct HasUrl;

   struct RequestBuilder<State> {
       url: Option<String>,
       headers: HashMap<String, String>,
       _state: PhantomData<State>,
   }

   impl RequestBuilder<NoUrl> {
       fn new() -> Self { /* ... */ }
       fn url(self, url: impl Into<String>) -> RequestBuilder<HasUrl> { /* ... */ }
       // NO send() method! Must set URL first ✅
   }

   impl RequestBuilder<HasUrl> {
       fn send(self) -> Result<Response> { /* ... */ }
       // Only available after URL is set! ✅
   }

   // Won't compile:
   let request = RequestBuilder::new();
   request.send()?;  // ❌ No send() on RequestBuilder<NoUrl>
   ```

**Advantages of Typestate Pattern**:

✅ **Compile-time safety**: Invalid transitions caught before runtime
✅ **Better IDE support**: Autocomplete shows only valid methods for current state
✅ **Self-documenting**: Type signatures show state flow
✅ **Zero runtime cost**: State stored in type, not value
✅ **Impossible states impossible**: Can't have `Order<Shipped>` without paying first
✅ **Clearer error messages**: Compiler explains what went wrong and suggests fixes

**Disadvantages of Typestate Pattern**:

❌ **More complex**: More types and `impl` blocks than enum approach
❌ **Less dynamic**: Can't store `Vec<Order<?>>` with mixed states easily
❌ **Verbose generics**: Type signatures get longer: `Order<Pending>` vs `OrderState`
❌ **Trait objects difficult**: Need trait bounds for dynamic dispatch
❌ **Learning curve**: PhantomData and type-level programming are advanced concepts

**When to Use Typestate vs Enum States**:

| Use Typestate When... | Use Enum When... |
|----------------------|------------------|
| State flow is known at compile-time | State changes based on runtime data |
| Want maximum compile-time safety | Need to store mixed states (`Vec<OrderState>`) |
| Building APIs where mistakes are costly | Building flexible workflow engines |
| IDE support is critical | Dynamic state transitions (e.g., config-driven) |
| Zero runtime cost is important | Simplicity is more important than type safety |

**Examples**:
- **Typestate**: Database connections, file handles, protocol parsers, builder APIs
- **Enum**: Order processing, payment workflows, game states, document lifecycles

**Hybrid Approach**:

Sometimes you need both! Use typestate for compile-time safety, wrap in enum for storage:

```rust
// Typestate for API safety
struct Order<State> { /* ... */ }

// Enum for storage
enum AnyOrder {
    Pending(Order<Pending>),
    Paid(Order<Paid>),
    Shipped(Order<Shipped>),
    Delivered(Order<Delivered>),
    Cancelled(Order<Cancelled>),
}

// Store mixed states
let orders: Vec<AnyOrder> = vec![
    AnyOrder::Pending(pending_order),
    AnyOrder::Shipped(shipped_order),
];
```

**Checkpoint Tests**:
```rust
#[test]
fn test_typestate_valid_flow() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let order = Order::<Pending>::new(1, items).unwrap();
    let order = order.pay("payment_123".to_string()).unwrap();
    let order = order.ship("TRACK123".to_string());
    let order = order.deliver();

    // order is now Order<Delivered>
    assert_eq!(order.customer_id(), 1);
}

#[test]
fn test_compile_time_enforcement() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let pending_order = Order::<Pending>::new(1, items).unwrap();

    // These won't compile! Uncomment to see errors:
    // pending_order.ship("TRACK123".to_string()); // ❌ No ship method on Pending
    // pending_order.deliver(); // ❌ No deliver method on Pending

    let paid_order = pending_order.pay("payment_123".to_string()).unwrap();
    // paid_order.pay("payment_456".to_string()); // ❌ No pay method on Paid (consumed)

    let shipped_order = paid_order.ship("TRACK123".to_string());
    // shipped_order.cancel("Oops".to_string()); // ❌ No cancel method on Shipped!
}

#[test]
fn test_cancellation_only_early_states() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    // Can cancel pending
    let order = Order::<Pending>::new(1, items.clone()).unwrap();
    let _cancelled = order.cancel("Customer request".to_string());

    // Can cancel paid
    let order = Order::<Pending>::new(1, items).unwrap();
    let order = order.pay("payment_123".to_string()).unwrap();
    let _cancelled = order.cancel("Changed mind".to_string());

    // Shipped orders don't have cancel method - compile-time enforcement!
}

#[test]
fn test_common_methods_all_states() {
    let items = vec![
        Item { product_id: 1, name: "Widget".to_string(), price: 9.99 },
    ];

    let pending = Order::<Pending>::new(1, items.clone()).unwrap();
    assert_eq!(pending.customer_id(), 1);

    let paid = Order::<Pending>::new(1, items).unwrap()
        .pay("payment_123".to_string())
        .unwrap();
    assert_eq!(paid.customer_id(), 1);

    // Common methods available in all states
}
```

**Starter Code**:
```rust
use std::marker::PhantomData;

// State marker types (zero-sized types)
// Role: Compile-time type markers that carry no runtime data
struct Pending;    // Order created, awaiting payment
struct Paid;       // Payment received, awaiting shipment
struct Shipped;    // Order shipped, in transit
struct Delivered;  // Order delivered to customer
struct Cancelled;  // Order cancelled (terminal state)

// Order<State>: Generic order struct parameterized by state
// Role: Holds order data, state encoded in type parameter
struct Order<State> {
    id: u64,                    // Unique order identifier
    customer_id: u64,           // Customer who placed order
    items: Vec<Item>,           // Items in the order
    _state: PhantomData<State>, // Zero-sized type marker for compile-time state
}

// Pending state implementation
// Role: Methods available only when Order is in Pending state
impl Order<Pending> {
    // new: Creates a new order in Pending state
    // Role: Constructor validating items and initializing order
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        // TODO: Validate items not empty
        // TODO: Create Order<Pending> with generated id (e.g., customer_id)
        // Hint: Use PhantomData for _state field: _state: PhantomData
        todo!()
    }

    // pay: Transitions from Pending to Paid
    // Role: Processes payment, consumes Order<Pending>, returns Order<Paid>
    fn pay(self, payment_id: String) -> Result<Order<Paid>, String> {
        // TODO: Validate items, calculate total amount
        // TODO: Simulate payment processing
        // TODO: Return Order<Paid> with same id, customer_id, items
        // Hint: Order { id: self.id, customer_id: self.customer_id, items: self.items, _state: PhantomData }
        todo!()
    }

    // cancel: Transitions from Pending to Cancelled
    // Role: Cancels order before payment
    fn cancel(self, reason: String) -> Order<Cancelled> {
        // TODO: Return Order<Cancelled> with same data
        // Note: Doesn't return Result since cancellation always allowed from Pending
        todo!()
    }
}

// Paid state implementation
// Role: Methods available only when Order is in Paid state
impl Order<Paid> {
    // ship: Transitions from Paid to Shipped
    // Role: Marks order as shipped with tracking number
    fn ship(self, tracking_number: String) -> Order<Shipped> {
        // TODO: Return Order<Shipped>
        // Note: In real system, would store tracking_number in Order struct
        // For this exercise, just transition the state
        todo!()
    }

    // cancel: Transitions from Paid to Cancelled
    // Role: Cancels order after payment but before shipping
    fn cancel(self, reason: String) -> Order<Cancelled> {
        // TODO: Return Order<Cancelled> with same data
        todo!()
    }
}

// Shipped state implementation
// Role: Methods available only when Order is in Shipped state
impl Order<Shipped> {
    // deliver: Transitions from Shipped to Delivered
    // Role: Marks order as delivered (terminal state)
    fn deliver(self) -> Order<Delivered> {
        // TODO: Return Order<Delivered>
        todo!()
    }

    // Note: No cancel method! Can't cancel after shipping - enforced at compile-time
}

// Delivered state implementation (terminal state)
// Role: No state transitions available from Delivered
impl Order<Delivered> {
    // No state transition methods - terminal state
}

// Common methods available in all states
// Role: Generic implementation over any state type
impl<State> Order<State> {
    // id: Returns order ID
    // Role: Accessor available in all states
    fn id(&self) -> u64 {
        self.id
    }

    // customer_id: Returns customer ID
    // Role: Accessor available in all states
    fn customer_id(&self) -> u64 {
        self.customer_id
    }

    // items: Returns reference to order items
    // Role: Accessor available in all states
    fn items(&self) -> &[Item] {
        &self.items
    }
}
```

**Check Your Understanding**:
- Why is `_state: PhantomData<State>` needed?
- What's the memory size of `Order<Pending>` vs `Order<Paid>`? (Hint: same!)
- Why can't you store `Vec<Order<??>>` with mixed states?
- How does IDE autocomplete know which methods are available?
- When would you prefer runtime enum states vs compile-time typestates?

---

### Complete Project Summary

**What You Built**:
1. Enum-based state machine with associated data per state
2. State transition methods with exhaustive pattern matching
3. Typestate pattern for compile-time state checking
4. Comparison of runtime vs compile-time state enforcement

**Key Concepts Practiced**:
- Enum variants with different associated data
- Exhaustive pattern matching
- Consuming transitions (taking `self`)
- Phantom types and zero-sized types
- Compile-time state machines

**Runtime vs Compile-Time Comparison**:

| Aspect | Enum States (Runtime) | Typestate (Compile-Time) |
|--------|----------------------|--------------------------|
| **Flexibility** | Can store mixed states in `Vec<OrderState>` | Can't store mixed states easily |
| **Validation** | Returns `Result`, checked at runtime | Compile error for invalid transitions |
| **IDE Support** | Shows all methods on all states | Shows only valid methods for current state |
| **Memory** | Size = largest variant + discriminant | Size = data only, state in type |
| **Complexity** | Simpler, one type | More complex, multiple types |
| **Use Case** | Dynamic state, runtime decisions | Known state flow, API safety |

**Real-World Applications**:
- **Enum approach**: Payment processors, workflow engines with dynamic states
- **Typestate approach**: Database connections, file handles, builder patterns

---

### Complete Working Example

Here's the fully implemented solution for both approaches:

```rust
use std::time::Instant;
use std::marker::PhantomData;

// ============================================================================
// MILESTONE 1 & 2: Enum-Based State Machine
// ============================================================================

#[derive(Debug, Clone)]
struct Item {
    product_id: u64,
    name: String,
    price: f64,
}

#[derive(Debug, Clone)]
enum OrderState {
    Pending {
        items: Vec<Item>,
        customer_id: u64,
    },
    Paid {
        order_id: u64,
        payment_id: String,
        amount: f64,
    },
    Shipped {
        order_id: u64,
        tracking_number: String,
    },
    Delivered {
        order_id: u64,
        delivered_at: Instant,
    },
    Cancelled {
        order_id: u64,
        reason: String,
    },
}

impl OrderState {
    fn new_pending(items: Vec<Item>, customer_id: u64) -> Self {
        OrderState::Pending { items, customer_id }
    }

    fn status_string(&self) -> &str {
        match self {
            OrderState::Pending { .. } => "Pending",
            OrderState::Paid { .. } => "Paid",
            OrderState::Shipped { .. } => "Shipped",
            OrderState::Delivered { .. } => "Delivered",
            OrderState::Cancelled { .. } => "Cancelled",
        }
    }

    // State transition: Pending -> Paid
    fn pay(self, payment_id: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { items, customer_id } => {
                if items.is_empty() {
                    return Err("Cannot pay for order with no items".to_string());
                }
                let amount: f64 = items.iter().map(|item| item.price).sum();
                Ok(OrderState::Paid {
                    order_id: customer_id, // Using customer_id as order_id
                    payment_id,
                    amount,
                })
            }
            _ => Err("Can only pay for pending orders".to_string()),
        }
    }

    // State transition: Paid -> Shipped
    fn ship(self, tracking_number: String) -> Result<Self, String> {
        match self {
            OrderState::Paid { order_id, .. } => {
                Ok(OrderState::Shipped {
                    order_id,
                    tracking_number,
                })
            }
            _ => Err("Can only ship paid orders".to_string()),
        }
    }

    // State transition: Shipped -> Delivered
    fn deliver(self) -> Result<Self, String> {
        match self {
            OrderState::Shipped { order_id, .. } => {
                Ok(OrderState::Delivered {
                    order_id,
                    delivered_at: Instant::now(),
                })
            }
            _ => Err("Can only deliver shipped orders".to_string()),
        }
    }

    // State transition: Pending/Paid -> Cancelled
    fn cancel(self, reason: String) -> Result<Self, String> {
        match self {
            OrderState::Pending { customer_id, .. } => {
                Ok(OrderState::Cancelled {
                    order_id: customer_id,
                    reason,
                })
            }
            OrderState::Paid { order_id, .. } => {
                Ok(OrderState::Cancelled { order_id, reason })
            }
            OrderState::Shipped { .. } | OrderState::Delivered { .. } => {
                Err("Cannot cancel after shipping".to_string())
            }
            OrderState::Cancelled { .. } => {
                Err("Order already cancelled".to_string())
            }
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(self, OrderState::Pending { .. } | OrderState::Paid { .. })
    }
}

// ============================================================================
// MILESTONE 3: Typestate Pattern
// ============================================================================

// Zero-sized state marker types
struct Pending;
struct Paid;
struct Shipped;
struct Delivered;
struct Cancelled;

// Generic Order with typestate
struct Order<State> {
    id: u64,
    customer_id: u64,
    items: Vec<Item>,
    _state: PhantomData<State>,
}

impl Order<Pending> {
    fn new(customer_id: u64, items: Vec<Item>) -> Result<Self, String> {
        if items.is_empty() {
            return Err("Cannot create order with no items".to_string());
        }
        Ok(Order {
            id: customer_id, // Using customer_id as order id
            customer_id,
            items,
            _state: PhantomData,
        })
    }

    fn pay(self, _payment_id: String) -> Result<Order<Paid>, String> {
        let total: f64 = self.items.iter().map(|item| item.price).sum();
        if total <= 0.0 {
            return Err("Order total must be positive".to_string());
        }
        Ok(Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        })
    }

    fn cancel(self, _reason: String) -> Order<Cancelled> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }
}

impl Order<Paid> {
    fn ship(self, _tracking_number: String) -> Order<Shipped> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }

    fn cancel(self, _reason: String) -> Order<Cancelled> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }
}

impl Order<Shipped> {
    fn deliver(self) -> Order<Delivered> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            items: self.items,
            _state: PhantomData,
        }
    }
    // Note: No cancel method - enforced at compile time!
}

impl Order<Delivered> {
    // Terminal state - no transitions
}

impl Order<Cancelled> {
    // Terminal state - no transitions
}

// Common methods available in all states
impl<State> Order<State> {
    fn id(&self) -> u64 {
        self.id
    }

    fn customer_id(&self) -> u64 {
        self.customer_id
    }

    fn items(&self) -> &[Item] {
        &self.items
    }
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    println!("=== Enum-Based State Machine ===\n");

    // Example 1: Complete order flow
    let items = vec![
        Item {
            product_id: 1,
            name: "Rust Book".to_string(),
            price: 39.99,
        },
        Item {
            product_id: 2,
            name: "Mechanical Keyboard".to_string(),
            price: 129.99,
        },
    ];

    let order = OrderState::new_pending(items.clone(), 12345);
    println!("Order status: {}", order.status_string());

    let order = order.pay("PAY_ABC123".to_string()).unwrap();
    println!("Order status after payment: {}", order.status_string());

    let order = order.ship("TRACK_XYZ789".to_string()).unwrap();
    println!("Order status after shipping: {}", order.status_string());

    let order = order.deliver().unwrap();
    println!("Order status after delivery: {}\n", order.status_string());

    // Example 2: Cancellation flow
    let order2 = OrderState::new_pending(items.clone(), 67890);
    let order2 = order2.pay("PAY_DEF456".to_string()).unwrap();
    println!("Can cancel paid order: {}", order2.can_cancel());

    let order2 = order2.cancel("Customer changed mind".to_string()).unwrap();
    println!("Order cancelled: {}\n", order2.status_string());

    // Example 3: Invalid transitions (caught at runtime)
    let order3 = OrderState::new_pending(items.clone(), 11111);
    match order3.ship("INVALID".to_string()) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Error (expected): {}\n", e),
    }

    println!("=== Typestate Pattern (Compile-Time Safety) ===\n");

    // Example 4: Typestate complete flow
    let order = Order::<Pending>::new(22222, items.clone()).unwrap();
    println!("Created order ID: {}", order.id());

    let order = order.pay("PAY_GHI789".to_string()).unwrap();
    println!("Payment processed");

    let order = order.ship("TRACK_JKL012".to_string());
    println!("Order shipped");

    let order = order.deliver();
    println!("Order delivered, ID: {}\n", order.id());

    // Example 5: Typestate cancellation
    let order = Order::<Pending>::new(33333, items.clone()).unwrap();
    let order = order.pay("PAY_MNO345".to_string()).unwrap();
    let order = order.cancel("Refund requested".to_string());
    println!("Order cancelled, ID: {}", order.id());

    // Example 6: These won't compile! (Uncomment to see compile errors)
    // let pending = Order::<Pending>::new(44444, items).unwrap();
    // pending.ship("ERROR".to_string()); // ❌ Compile error: no ship method on Pending
    // pending.deliver(); // ❌ Compile error: no deliver method on Pending

    // let shipped = pending.pay("PAY".to_string()).unwrap().ship("TRACK".to_string());
    // shipped.cancel("Too late".to_string()); // ❌ Compile error: no cancel on Shipped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_state_machine_complete_flow() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = OrderState::new_pending(items, 1);
        let order = order.pay("payment".to_string()).unwrap();
        let order = order.ship("tracking".to_string()).unwrap();
        let order = order.deliver().unwrap();

        assert_eq!(order.status_string(), "Delivered");
    }

    #[test]
    fn test_enum_invalid_transitions() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = OrderState::new_pending(items, 1);

        // Can't ship before paying
        assert!(order.clone().ship("track".to_string()).is_err());

        let order = order.pay("payment".to_string()).unwrap();
        let order = order.ship("track".to_string()).unwrap();

        // Can't cancel after shipping
        assert!(order.cancel("reason".to_string()).is_err());
    }

    #[test]
    fn test_typestate_complete_flow() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = Order::<Pending>::new(1, items).unwrap();
        let order = order.pay("payment".to_string()).unwrap();
        let order = order.ship("tracking".to_string());
        let order = order.deliver();

        assert_eq!(order.id(), 1);
    }

    #[test]
    fn test_typestate_cancellation() {
        let items = vec![Item {
            product_id: 1,
            name: "Test".to_string(),
            price: 10.0,
        }];

        let order = Order::<Pending>::new(1, items.clone()).unwrap();
        let _cancelled = order.cancel("reason".to_string());

        let order = Order::<Pending>::new(1, items).unwrap();
        let order = order.pay("payment".to_string()).unwrap();
        let _cancelled = order.cancel("reason".to_string());

        // Shipped orders can't be cancelled - enforced by type system
    }
}
```

**Key Takeaways from Complete Example**:

**Enum Approach (Runtime)**:
1. **Flexibility**: Single type can represent all states
2. **Dynamic**: Can store `Vec<OrderState>` with mixed states
3. **Runtime checks**: Invalid transitions return `Err` at runtime
4. **Simpler**: One enum definition with pattern matching

**Typestate Approach (Compile-Time)**:
1. **Type safety**: Invalid transitions won't compile
2. **Zero runtime cost**: State in type system, not data
3. **IDE support**: Autocomplete shows only valid methods
4. **More complex**: Multiple types and implementations

**When to Use Each**:
- **Enum**: When state is determined at runtime, need to store mixed states
- **Typestate**: When state flow is known, want maximum compile-time safety

---
