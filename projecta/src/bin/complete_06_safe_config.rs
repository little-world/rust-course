// Complete Type-Safe Configuration System
// Demonstrates newtype pattern, validation, and builder pattern

use std::num::NonZeroU32;
use std::ops::Deref;
use std::time::Duration;

//==============================================================================
// Milestone 1: Basic Configuration Struct
//==============================================================================

/// BasicServerConfig: Simple configuration struct without type safety
/// Demonstrates the problems with using primitive types directly
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BasicServerConfig {
    host: String,
    port: u16,
    timeout_seconds: u64,
    max_connections: u32,
}

impl BasicServerConfig {
    /// Creates a basic config - allows invalid values!
    #[allow(dead_code)]
    fn new(host: String, port: u16, timeout_seconds: u64, max_connections: u32) -> Self {
        BasicServerConfig {
            host,
            port,
            timeout_seconds,
            max_connections,
        }
    }
}

//==============================================================================
// Milestone 2: Newtype Wrappers for Type Safety
//==============================================================================

/// Hostname: Type-safe wrapper for server hostname
/// Prevents mixing hostname strings with other string types
#[derive(Debug, Clone, PartialEq)]
struct Hostname(String);

impl Hostname {
    #[allow(dead_code)]
    fn new(hostname: String) -> Self {
        Hostname(hostname)
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

/// Port: Validated TCP port number (1-65535)
/// Prevents invalid port values at compile time
#[derive(Debug, Clone, Copy, PartialEq)]
struct Port(u16);

impl Port {
    /// Smart constructor that validates port range
    fn new(port: u16) -> Result<Self, String> {
        if port == 0 {
            Err("Port must be greater than 0".to_string())
        } else {
            Ok(Port(port))
        }
    }

    #[allow(dead_code)]
    fn get(&self) -> u16 {
        self.0
    }
}

impl Deref for Port {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Timeout: Validated timeout duration
/// Ensures timeout is positive and has clear units (Duration)
#[derive(Debug, Clone, Copy, PartialEq)]
struct Timeout(Duration);

impl Timeout {
    /// Smart constructor that validates positive timeout
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

/// MaxConnections: Guaranteed non-zero connection limit
/// Uses NonZeroU32 for additional niche optimization
#[derive(Debug, Clone, Copy, PartialEq)]
struct MaxConnections(NonZeroU32);

impl MaxConnections {
    /// Smart constructor that ensures non-zero count
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

//==============================================================================
// ServerConfig: Type-Safe Configuration
//==============================================================================

/// ServerConfig: Main configuration struct using type-safe newtypes
/// All fields are validated at construction time
#[derive(Debug, Clone)]
struct ServerConfig {
    host: Hostname,
    port: Port,
    timeout: Timeout,
    max_connections: MaxConnections,
}

impl ServerConfig {
    /// Creates a new ServerConfig from validated newtypes
    fn new(host: Hostname, port: Port, timeout: Timeout, max_connections: MaxConnections) -> Self {
        ServerConfig {
            host,
            port,
            timeout,
            max_connections,
        }
    }

    /// Entry point for fluent builder API
    fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
    }
}

//==============================================================================
// Milestone 3: Builder Pattern with Defaults
//==============================================================================

/// ServerConfigBuilder: Fluent builder for constructing ServerConfig
/// Collects configuration values step-by-step with validation and defaults
struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout_secs: Option<u64>,
    max_connections: Option<u32>,
}

impl ServerConfigBuilder {
    /// Creates empty builder with all fields None
    fn new() -> Self {
        ServerConfigBuilder {
            host: None,
            port: None,
            timeout_secs: None,
            max_connections: None,
        }
    }

    /// Sets hostname (required field)
    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets port number (defaults to 8080 if not specified)
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets timeout in seconds (defaults to 30 if not specified)
    fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Sets maximum connections (defaults to 100 if not specified)
    fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    /// Validates all fields and constructs ServerConfig
    /// Collects ALL validation errors, not just the first one
    fn build(self) -> Result<ServerConfig, Vec<String>> {
        let mut errors = Vec::new();

        // Validate hostname (required field)
        let host = match self.host {
            Some(h) if !h.is_empty() => Hostname(h),
            Some(_) => {
                errors.push("Host cannot be empty".to_string());
                Hostname("localhost".to_string()) // Placeholder for error path
            }
            None => {
                errors.push("Host is required".to_string());
                Hostname("localhost".to_string())
            }
        };

        // Validate port with default 8080
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

        // Validate timeout with default 30 seconds
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

        // Validate max_connections with default 100
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

        // Return all errors or valid config
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(ServerConfig::new(host, port, timeout, max_connections))
        }
    }
}

//==============================================================================
// Example Usage and Main
//==============================================================================

fn main() {
    println!("=== Type-Safe Configuration System ===\n");

    // Example 1: Using builder with all fields
    println!("Example 1: Complete configuration");
    let config = ServerConfig::builder()
        .host("0.0.0.0")
        .port(8080)
        .timeout_secs(30)
        .max_connections(100)
        .build()
        .unwrap();

    println!("Config: {:?}", config);
    println!("  Host: {}", config.host.as_str());
    println!("  Port: {}", *config.port); // Deref in action!
    println!("  Timeout: {:?}", config.timeout.as_duration());
    println!("  Max connections: {}", config.max_connections.get());
    println!();

    // Example 2: Using builder with defaults
    println!("Example 2: Using defaults");
    let config2 = ServerConfig::builder().host("localhost").build().unwrap();

    println!("Config with defaults: {:?}", config2);
    println!("  Default port: {}", *config2.port);
    println!("  Default timeout: {:?}", config2.timeout.as_duration());
    println!(
        "  Default max_connections: {}",
        config2.max_connections.get()
    );
    println!();

    // Example 3: Handling validation errors
    println!("Example 3: Validation errors (collected all at once)");
    let result = ServerConfig::builder()
        .port(0) // Invalid!
        .timeout_secs(0) // Invalid!
        .max_connections(0) // Invalid!
        .build();

    match result {
        Ok(_) => println!("Unexpected success"),
        Err(errors) => {
            println!("Validation errors:");
            for error in errors {
                println!("  - {}", error);
            }
        }
    }
    println!();

    // Example 4: Type safety demonstration
    println!("Example 4: Type safety prevents mistakes");
    let port = Port::new(8080).unwrap();
    let timeout = Timeout::from_secs(30).unwrap();

    println!("Created port: {}", *port);
    println!("Created timeout: {:?}", *timeout);

    // This won't compile - type safety!
    // let bad_config = ServerConfig::new(
    //     Hostname("localhost".to_string()),
    //     timeout,  // ❌ Wrong type! Expected Port, got Timeout
    //     port,     // ❌ Wrong type! Expected Timeout, got Port
    //     MaxConnections::new(100).unwrap(),
    // );
    println!("(Compiler prevents mixing up types at compile time!)");
    println!();

    // Example 5: Milestone 1 comparison
    println!("Example 5: Why Milestone 1 isn't enough");
    let basic = BasicServerConfig::new(
        "".to_string(), // Empty host - invalid!
        0,              // Invalid port!
        0,              // Zero timeout!
        0,              // Zero connections!
    );
    println!(
        "BasicServerConfig allows invalid values: port={}, timeout={}, max_conn={}",
        basic.port, basic.timeout_seconds, basic.max_connections
    );
    println!("(No compile-time or runtime validation!)");
    println!();

    // Example 6: Deref trait demonstration
    println!("Example 6: Deref trait for ergonomic access");
    let port = Port::new(8080).unwrap();
    println!("Port value: {}", *port); // Deref to u16
    if *port > 1024 {
        println!("Unprivileged port (> 1024)");
    }

    let timeout = Timeout::from_secs(30).unwrap();
    println!("Timeout: {} seconds", timeout.as_secs()); // Deref to Duration
}

//==============================================================================
// Tests
//==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 Tests
    #[test]
    fn test_basic_config() {
        let config = BasicServerConfig::new("localhost".to_string(), 8080, 30, 100);

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_can_create_invalid_config() {
        // This compiles but is semantically wrong!
        let bad_config = BasicServerConfig::new("".to_string(), 0, 0, 0);

        // No way to prevent this at compile time
        assert_eq!(bad_config.port, 0);
        assert_eq!(bad_config.timeout_seconds, 0);
    }

    // Milestone 2 Tests
    #[test]
    fn test_port_validation() {
        assert!(Port::new(8080).is_ok());
        assert!(Port::new(1).is_ok());
        assert!(Port::new(65535).is_ok());
        assert!(Port::new(0).is_err()); // Invalid port
    }

    #[test]
    fn test_timeout_validation() {
        assert!(Timeout::from_secs(30).is_ok());
        assert!(Timeout::from_secs(1).is_ok());
        assert!(Timeout::from_secs(0).is_err()); // Zero timeout invalid
    }

    #[test]
    fn test_max_connections() {
        assert!(MaxConnections::new(100).is_ok());
        assert!(MaxConnections::new(1).is_ok());
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
        assert_eq!(config.timeout.as_duration().as_secs(), 30);
        assert_eq!(config.max_connections.get(), 100);
    }

    #[test]
    fn test_newtype_validation() {
        assert!(Port::new(8080).is_ok());
        assert!(Port::new(0).is_err());
        assert!(Timeout::from_secs(30).is_ok());
        assert!(Timeout::from_secs(0).is_err());
        assert!(MaxConnections::new(100).is_ok());
        assert!(MaxConnections::new(0).is_err());
    }

    // Milestone 3 Tests
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
        assert_eq!(config.timeout.as_duration().as_secs(), 30);
        assert_eq!(config.max_connections.get(), 100);
    }

    #[test]
    fn test_builder_defaults() {
        let config = ServerConfig::builder().host("localhost").build().unwrap();

        // Should use default values
        assert_eq!(*config.port, 8080);
        assert_eq!(config.timeout.as_duration().as_secs(), 30);
        assert_eq!(config.max_connections.get(), 100);
    }

    #[test]
    fn test_builder_validation_errors() {
        let result = ServerConfig::builder()
            .port(0) // Invalid
            .timeout_secs(0) // Invalid
            .build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // At least port and timeout errors
        assert!(errors.iter().any(|e| e.contains("port")));
        assert!(errors.iter().any(|e| e.contains("timeout")));
    }

    #[test]
    fn test_builder_missing_required() {
        let result = ServerConfig::builder().port(8080).build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("Host")));
    }

    #[test]
    fn test_builder_empty_host() {
        let result = ServerConfig::builder().host("").build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("empty")));
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
    fn test_builder_multiple_errors() {
        let result = ServerConfig::builder()
            .port(0) // Invalid
            .timeout_secs(0) // Invalid
            .max_connections(0) // Invalid
            .build();

        assert!(result.is_err());
        let errors = result.unwrap_err();
        // Should have at least 4 errors: missing host, invalid port, timeout, max_connections
        assert!(errors.len() >= 3);
    }

    #[test]
    fn test_deref_trait() {
        let port = Port::new(8080).unwrap();
        assert_eq!(*port, 8080);

        let timeout = Timeout::from_secs(30).unwrap();
        assert_eq!(timeout.as_secs(), 30);

        let max_conn = MaxConnections::new(100).unwrap();
        assert_eq!(max_conn.get(), 100);
    }

    #[test]
    fn test_port_deref_comparison() {
        let port = Port::new(8080).unwrap();
        // Explicit deref for comparison
        assert!(*port > 1024);
        assert!(*port < 65535);
    }

    #[test]
    fn test_nonzero_u32_size_optimization() {
        use std::mem::size_of;

        // Niche optimization: Option<NonZeroU32> is same size as NonZeroU32
        assert_eq!(size_of::<NonZeroU32>(), size_of::<Option<NonZeroU32>>());

        // Regular u32 needs extra space for Option discriminant
        assert!(size_of::<u32>() < size_of::<Option<u32>>());
    }

    #[test]
    fn test_builder_with_string_types() {
        // Test that builder accepts &str, String, etc.
        let config1 = ServerConfig::builder().host("localhost").build().unwrap();
        assert_eq!(config1.host.as_str(), "localhost");

        let hostname = String::from("example.com");
        let config2 = ServerConfig::builder().host(hostname).build().unwrap();
        assert_eq!(config2.host.as_str(), "example.com");
    }

    #[test]
    fn test_zero_cost_abstraction() {
        use std::mem::size_of;

        // Newtypes have no runtime overhead
        assert_eq!(size_of::<Port>(), size_of::<u16>());
        assert_eq!(size_of::<Timeout>(), size_of::<Duration>());
        assert_eq!(size_of::<MaxConnections>(), size_of::<NonZeroU32>());
    }

    #[test]
    fn test_config_clone() {
        let config = ServerConfig::builder()
            .host("localhost")
            .port(8080)
            .build()
            .unwrap();

        let config2 = config.clone();
        assert_eq!(*config.port, *config2.port);
        assert_eq!(config.host.as_str(), config2.host.as_str());
    }
}
