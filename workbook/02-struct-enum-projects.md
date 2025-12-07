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

**Check Your Understanding**:
- What's wrong with allowing `port: 0` or `max_connections: 0`?
- How could we accidentally pass the wrong string to the host parameter?
- What happens if someone passes timeout in milliseconds by mistake?

---

### Why Milestone 1 Isn't Enough → Moving to Milestone 2

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


**What We're Building**:

Four newtype wrappers with validation:

1. **`Hostname(String)`**: Type-safe string that's specifically a hostname
2. **`Port(u16)`**: Validated port number (1-65535)
3. **`Timeout(Duration)`**: Positive duration with clear units
4. **`MaxConnections(NonZeroU32)`**: Guaranteed positive connection limit

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

Milestone 2 gave us type safety, but the API is **clunky**. Creating a config requires many `.unwrap()` calls and looks ugly. The **builder pattern** solves this by providing a fluent, chainable API that's pleasant to use while maintaining all our safety guarantees.

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



### Milestone 1: Basic Order Enum with States

**Goal**: Define an enum representing different order states, where each variant carries state-specific data.

**Why This Milestone Matters**:

This milestone introduces **enums as state machines**—one of Rust's most powerful patterns. Unlike structs (which group related data), enums represent **alternatives**—a value is exactly one variant at any time.


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

**Check Your Understanding**:
- Why does each variant have different associated data?
- What prevents you from accessing `payment_id` on a `Pending` order?
- How does this compare to having optional fields on a single struct?

---

### Why Milestone 1 Isn't Enough 

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


**Check Your Understanding**:
- Why do transition methods take `self` (ownership) instead of `&self`?
- How does the compiler help you handle all possible current states?
- What happens if you forget to handle a variant in a match?
- Why is returning `Result` better than panicking on invalid transitions?

---

###  Why Milestone 2 Isn't Enough 

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
