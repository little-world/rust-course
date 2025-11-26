# Chapter 5: Builder & API Design - Programming Projects

## Project 1: Configuration System with Type-Safe Builder

### Problem Statement

Design a flexible, type-safe configuration system for a web server application that handles:
- Required fields (host, port) that must be set before use
- Optional fields with sensible defaults (timeout, max connections, log level)
- Environment-specific configurations (development, staging, production)
- Validation rules (port in valid range, positive timeout values)
- Nested configuration sections (TLS settings, database settings, cache settings)
- Multiple configuration sources (default → file → environment variables → CLI args)
- Type-state enforcement: can't start server without valid configuration
- Zero-cost abstractions: configuration resolved at compile time where possible

The system must provide both runtime flexibility (loading from files) and compile-time safety (required fields checked by type system).

### Why It Matters

Configuration management is critical in production systems:
- **Microservices**: Each service needs complex configuration management
- **Cloud Deployments**: Different configs for dev/staging/prod environments
- **Operational Safety**: Invalid configurations should fail fast at startup, not in production
- **Security**: Sensitive values (passwords, API keys) need special handling
- **Maintainability**: Self-documenting configuration reduces errors

Type-safe configuration prevents entire classes of bugs:
- Starting a server on port 0 or negative port (compile error)
- Missing required database credentials (compile error)
- Forgetting to set TLS certificates in production (type-state error)

### Use Cases

1. **Web Servers**: Configure HTTP/HTTPS, ports, timeouts, connection limits
2. **Database Applications**: Connection pools, retry policies, transaction settings
3. **Microservices**: Service discovery, health checks, circuit breakers
4. **CLI Tools**: Argument parsing with validation and defaults
5. **Game Servers**: Player limits, map settings, game rules
6. **IoT Devices**: Device-specific settings with hardware constraints
7. **Distributed Systems**: Cluster configuration, replication settings

### Solution Outline

**Core Structure:**
```rust
// Environment markers
pub struct Development;
pub struct Staging;
pub struct Production;

// Required field markers
pub struct NoHost;
pub struct HasHost;
pub struct NoPort;
pub struct HasPort;

// Configuration builder with type-state
pub struct ConfigBuilder<Env, Host, Port> {
    host: Option<String>,
    port: Option<u16>,
    max_connections: u32,
    timeout_secs: u64,
    log_level: LogLevel,
    tls: Option<TlsConfig>,
    database: Option<DatabaseConfig>,
    _env: PhantomData<Env>,
    _host: PhantomData<Host>,
    _port: PhantomData<Port>,
}

// Only buildable when Host and Port are set
impl ConfigBuilder<Production, HasHost, HasPort> {
    pub fn build(self) -> Result<Config, ConfigError> { /* ... */ }
}
```

**Key Design Elements:**
- **Builder Pattern**: Fluent API for configuration
- **Type-State**: Track required fields at compile time
- **Nested Builders**: TLS and database configs have own builders
- **Validation**: Port range, timeout values, file paths
- **Defaults**: Per-environment defaults
- **Serialization**: Load from TOML/JSON files
- **Override Chain**: defaults → file → env vars → CLI

**Configuration Sources Priority:**
1. Defaults (lowest priority)
2. Configuration file
3. Environment variables
4. Command-line arguments (highest priority)

### Testing Hints

**Compile-Time Tests:**
```rust
// Should NOT compile - missing required fields
fn test_missing_host() {
    let config = ConfigBuilder::<Production>::new()
        .port(8080)
        .build(); // ERROR: Host not set
}

// Should compile - all required fields set
fn test_complete_config() {
    let config = ConfigBuilder::<Production>::new()
        .host("0.0.0.0")
        .port(8080)
        .build()
        .unwrap();
}
```

**Runtime Tests:**
```rust
#[test]
fn test_port_validation() {
    let result = ConfigBuilder::<Production>::new()
        .host("0.0.0.0")
        .port(0) // Invalid
        .build();

    assert!(result.is_err());
}

#[test]
fn test_environment_overrides() {
    // Test that env vars override file settings
    std::env::set_var("SERVER_PORT", "9090");
    let config = Config::load_from_env().unwrap();
    assert_eq!(config.port(), 9090);
}

#[test]
fn test_default_values() {
    let config = ConfigBuilder::<Development>::new()
        .host("localhost")
        .port(3000)
        .build()
        .unwrap();

    assert_eq!(config.max_connections(), 100); // Default for dev
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Builder with Optional Fields

**Goal:** Create a working configuration builder using `Option<T>` for all fields.

**What to implement:**
```rust
pub struct Config {
    host: String,
    port: u16,
    max_connections: u32,
    timeout_secs: u64,
    log_level: LogLevel,
}

pub struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<u32>,
    timeout_secs: Option<u64>,
    log_level: Option<LogLevel>,
}

pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            max_connections: None,
            timeout_secs: None,
            log_level: None,
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

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = Some(level);
        self
    }

    pub fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.ok_or("port is required")?,
            max_connections: self.max_connections.unwrap_or(100),
            timeout_secs: self.timeout_secs.unwrap_or(30),
            log_level: self.log_level.unwrap_or(LogLevel::Info),
        })
    }
}
```

**Check/Test:**
- Build config with all fields, verify it works
- Try building without required fields, verify runtime error
- Test default values are applied for optional fields
- Test fluent chaining works ergonomically

**Why this isn't enough:**
Errors happen at runtime when `build()` is called. A developer could write code that configures everything except the host, and only discover the error when the server starts—potentially after deployment. The API allows nonsensical states like calling `build()` multiple times or setting the same field twice. We're relying on runtime validation instead of compile-time guarantees. No environment-specific defaults.

---

### Step 2: Add Type-State for Required Fields

**Goal:** Use phantom types to enforce required fields at compile time.

**What to improve:**
```rust
use std::marker::PhantomData;

// Field state markers
pub struct NoHost;
pub struct HasHost;
pub struct NoPort;
pub struct HasPort;

pub struct ConfigBuilder<Host, Port> {
    host: Option<String>,
    port: Option<u16>,
    max_connections: u32,
    timeout_secs: u64,
    log_level: LogLevel,
    _host: PhantomData<Host>,
    _port: PhantomData<Port>,
}

impl ConfigBuilder<NoHost, NoPort> {
    pub fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            max_connections: 100,
            timeout_secs: 30,
            log_level: LogLevel::Info,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

// Setting host transitions from NoHost to HasHost
impl<Port> ConfigBuilder<NoHost, Port> {
    pub fn host(self, host: impl Into<String>) -> ConfigBuilder<HasHost, Port> {
        ConfigBuilder {
            host: Some(host.into()),
            port: self.port,
            max_connections: self.max_connections,
            timeout_secs: self.timeout_secs,
            log_level: self.log_level,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

// Setting port transitions from NoPort to HasPort
impl<Host> ConfigBuilder<Host, NoPort> {
    pub fn port(self, port: u16) -> ConfigBuilder<Host, HasPort> {
        ConfigBuilder {
            host: self.host,
            port: Some(port),
            max_connections: self.max_connections,
            timeout_secs: self.timeout_secs,
            log_level: self.log_level,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

// Optional fields available on any state
impl<Host, Port> ConfigBuilder<Host, Port> {
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self
    }
}

// build() only available when both Host and Port are set
impl ConfigBuilder<HasHost, HasPort> {
    pub fn build(self) -> Config {
        Config {
            host: self.host.unwrap(), // Safe - type system guarantees this
            port: self.port.unwrap(), // Safe
            max_connections: self.max_connections,
            timeout_secs: self.timeout_secs,
            log_level: self.log_level,
        }
    }
}
```

**Check/Test:**
- Verify `build()` requires both host and port (compile error without)
- Test that optional fields can be set in any order
- Test that required fields can be set in any order
- Verify type-state transitions work correctly

**Why this isn't enough:**
While we now have compile-time checking for required fields, we don't have validation. Someone could call `.port(0)` or `.port(999999)`, both invalid. We also don't have environment-specific configurations (dev defaults vs production requirements). The boilerplate for field copying is getting unwieldy. We need validation and environment awareness.

---

### Step 3: Add Validation and Environment-Specific Builders

**Goal:** Validate field values and provide environment-specific defaults.

**What to improve:**

**1. Add validation:**
```rust
#[derive(Debug)]
pub enum ConfigError {
    InvalidPort(u16),
    InvalidTimeout(u64),
    InvalidMaxConnections(u32),
    ValidationFailed(String),
}

impl<Host> ConfigBuilder<Host, NoPort> {
    pub fn port(self, port: u16) -> Result<ConfigBuilder<Host, HasPort>, ConfigError> {
        if port == 0 {
            return Err(ConfigError::InvalidPort(port));
        }
        if port < 1024 && port != 80 && port != 443 {
            return Err(ConfigError::InvalidPort(port));
        }

        Ok(ConfigBuilder {
            host: self.host,
            port: Some(port),
            max_connections: self.max_connections,
            timeout_secs: self.timeout_secs,
            log_level: self.log_level,
            _host: PhantomData,
            _port: PhantomData,
        })
    }
}

impl<Host, Port> ConfigBuilder<Host, Port> {
    pub fn max_connections(self, max: u32) -> Result<Self, ConfigError> {
        if max == 0 || max > 10_000 {
            return Err(ConfigError::InvalidMaxConnections(max));
        }

        Ok(ConfigBuilder {
            max_connections: max,
            ..self
        })
    }

    pub fn timeout_secs(self, secs: u64) -> Result<Self, ConfigError> {
        if secs == 0 || secs > 3600 {
            return Err(ConfigError::InvalidTimeout(secs));
        }

        Ok(ConfigBuilder {
            timeout_secs: secs,
            ..self
        })
    }
}
```

**2. Add environment-specific builders:**
```rust
pub struct Development;
pub struct Staging;
pub struct Production;

pub struct ConfigBuilder<Env, Host, Port> {
    // Same fields as before
    _env: PhantomData<Env>,
    _host: PhantomData<Host>,
    _port: PhantomData<Port>,
}

impl ConfigBuilder<Development, NoHost, NoPort> {
    pub fn dev() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            max_connections: 10,        // Low for dev
            timeout_secs: 300,          // Long for debugging
            log_level: LogLevel::Debug, // Verbose in dev
            _env: PhantomData,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

impl ConfigBuilder<Staging, NoHost, NoPort> {
    pub fn staging() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            max_connections: 100,
            timeout_secs: 60,
            log_level: LogLevel::Info,
            _env: PhantomData,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

impl ConfigBuilder<Production, NoHost, NoPort> {
    pub fn production() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            max_connections: 1000,
            timeout_secs: 30,
            log_level: LogLevel::Warning,
            _env: PhantomData,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

// Production builds require TLS (we'll add this in next step)
impl ConfigBuilder<Production, HasHost, HasPort> {
    pub fn build(self) -> Result<Config, ConfigError> {
        // Production-specific validation
        if self.log_level == LogLevel::Debug {
            return Err(ConfigError::ValidationFailed(
                "Debug logging not allowed in production".into()
            ));
        }

        Ok(Config {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
            max_connections: self.max_connections,
            timeout_secs: self.timeout_secs,
            log_level: self.log_level,
        })
    }
}
```

**Check/Test:**
- Test port validation rejects 0 and privileged ports (< 1024)
- Test max_connections bounds checking
- Test environment-specific defaults
- Test production builds reject debug logging
- Test chaining with Result-returning methods (`?` operator)

**Why this isn't enough:**
Validation is good, but now all builder methods return `Result`, making the API more verbose. We have environment-specific defaults but no nested configurations—real systems need TLS settings, database configs, cache configs as sub-objects. We also don't have configuration loading from files or environment variables yet. The builder is getting complex but lacks real-world features.

---

### Step 4: Add Nested Configuration with Sub-Builders

**Goal:** Support complex nested configurations using sub-builders for TLS, database, and cache settings.

**What to improve:**

**1. Create nested configuration types:**
```rust
#[derive(Debug, Clone)]
pub struct TlsConfig {
    cert_path: PathBuf,
    key_path: PathBuf,
    ca_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    url: String,
    pool_size: u32,
    timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    enabled: bool,
    max_size_mb: usize,
    ttl_secs: u64,
}

// Main config now includes nested configs
pub struct Config {
    host: String,
    port: u16,
    max_connections: u32,
    timeout_secs: u64,
    log_level: LogLevel,
    tls: Option<TlsConfig>,
    database: Option<DatabaseConfig>,
    cache: Option<CacheConfig>,
}
```

**2. Create sub-builders:**
```rust
pub struct TlsConfigBuilder {
    cert_path: Option<PathBuf>,
    key_path: Option<PathBuf>,
    ca_path: Option<PathBuf>,
}

impl TlsConfigBuilder {
    pub fn new() -> Self {
        TlsConfigBuilder {
            cert_path: None,
            key_path: None,
            ca_path: None,
        }
    }

    pub fn cert(mut self, path: impl AsRef<Path>) -> Self {
        self.cert_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn key(mut self, path: impl AsRef<Path>) -> Self {
        self.key_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn ca(mut self, path: impl AsRef<Path>) -> Self {
        self.ca_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn build(self) -> Result<TlsConfig, ConfigError> {
        let cert_path = self.cert_path
            .ok_or_else(|| ConfigError::ValidationFailed("TLS cert required".into()))?;
        let key_path = self.key_path
            .ok_or_else(|| ConfigError::ValidationFailed("TLS key required".into()))?;

        // Validate files exist
        if !cert_path.exists() {
            return Err(ConfigError::ValidationFailed(
                format!("Cert file not found: {:?}", cert_path)
            ));
        }
        if !key_path.exists() {
            return Err(ConfigError::ValidationFailed(
                format!("Key file not found: {:?}", key_path)
            ));
        }

        Ok(TlsConfig {
            cert_path,
            key_path,
            ca_path: self.ca_path,
        })
    }
}

pub struct DatabaseConfigBuilder {
    url: Option<String>,
    pool_size: u32,
    timeout_secs: u64,
}

impl DatabaseConfigBuilder {
    pub fn new() -> Self {
        DatabaseConfigBuilder {
            url: None,
            pool_size: 10,
            timeout_secs: 30,
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn pool_size(mut self, size: u32) -> Result<Self, ConfigError> {
        if size == 0 || size > 100 {
            return Err(ConfigError::InvalidMaxConnections(size));
        }
        self.pool_size = size;
        Ok(self)
    }

    pub fn timeout_secs(mut self, secs: u64) -> Result<Self, ConfigError> {
        if secs == 0 {
            return Err(ConfigError::InvalidTimeout(secs));
        }
        self.timeout_secs = secs;
        Ok(self)
    }

    pub fn build(self) -> Result<DatabaseConfig, ConfigError> {
        let url = self.url
            .ok_or_else(|| ConfigError::ValidationFailed("Database URL required".into()))?;

        Ok(DatabaseConfig {
            url,
            pool_size: self.pool_size,
            timeout_secs: self.timeout_secs,
        })
    }
}
```

**3. Integrate sub-builders into main builder:**
```rust
impl<Env, Host, Port> ConfigBuilder<Env, Host, Port> {
    pub fn tls<F>(mut self, config_fn: F) -> Result<Self, ConfigError>
    where
        F: FnOnce(TlsConfigBuilder) -> Result<TlsConfig, ConfigError>,
    {
        let tls = config_fn(TlsConfigBuilder::new())?;
        self.tls = Some(tls);
        Ok(self)
    }

    pub fn database<F>(mut self, config_fn: F) -> Result<Self, ConfigError>
    where
        F: FnOnce(DatabaseConfigBuilder) -> Result<DatabaseConfig, ConfigError>,
    {
        let db = config_fn(DatabaseConfigBuilder::new())?;
        self.database = Some(db);
        Ok(self)
    }

    pub fn cache<F>(mut self, config_fn: F) -> Self
    where
        F: FnOnce(CacheConfigBuilder) -> CacheConfig,
    {
        self.cache = Some(config_fn(CacheConfigBuilder::new()));
        self
    }
}
```

**Usage:**
```rust
let config = ConfigBuilder::<Production>::production()
    .host("0.0.0.0")?
    .port(443)?
    .tls(|tls| {
        tls.cert("/etc/ssl/cert.pem")
            .key("/etc/ssl/key.pem")
            .ca("/etc/ssl/ca.pem")
            .build()
    })?
    .database(|db| {
        db.url("postgres://localhost/myapp")?
            .pool_size(20)?
            .timeout_secs(60)?
            .build()
    })?
    .build()?;
```

**Check/Test:**
- Test nested builder construction
- Test sub-builder validation (missing required fields)
- Test file path validation for TLS
- Test that nested configs are optional
- Test fluent chaining with nested builders

**Why this isn't enough:**
The nested builder approach is powerful but now we have verbose `Result` handling everywhere. We also can't load configuration from files yet—all configs are hard-coded. Real applications need to load from TOML/JSON/YAML files, merge with environment variables, and override with CLI arguments. We need a configuration loading system.

---

### Step 5: Add Configuration File Loading and Merging

**Goal:** Load configuration from files and merge with environment variables.

**What to improve:**

**1. Add serialization support:**
```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    server: ServerConfig,
    #[serde(default)]
    tls: Option<TlsConfigFile>,
    #[serde(default)]
    database: Option<DatabaseConfigFile>,
    #[serde(default)]
    cache: Option<CacheConfigFile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerConfig {
    host: Option<String>,
    port: Option<u16>,
    max_connections: Option<u32>,
    timeout_secs: Option<u64>,
    log_level: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: None,
            port: None,
            max_connections: None,
            timeout_secs: None,
            log_level: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TlsConfigFile {
    cert_path: String,
    key_path: String,
    ca_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfigFile {
    url: String,
    pool_size: Option<u32>,
    timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheConfigFile {
    enabled: bool,
    max_size_mb: Option<usize>,
    ttl_secs: Option<u64>,
}
```

**2. Implement loading and merging:**
```rust
impl<Env, Host, Port> ConfigBuilder<Env, Host, Port> {
    pub fn from_file(mut self, path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::ValidationFailed(format!("Failed to read config: {}", e)))?;

        let file_config: ConfigFile = toml::from_str(&content)
            .map_err(|e| ConfigError::ValidationFailed(format!("Failed to parse config: {}", e)))?;

        // Merge file config into builder
        if let Some(host) = file_config.server.host {
            self.host = Some(host);
        }
        if let Some(port) = file_config.server.port {
            self.port = Some(port);
        }
        if let Some(max_conn) = file_config.server.max_connections {
            self.max_connections = max_conn;
        }
        if let Some(timeout) = file_config.server.timeout_secs {
            self.timeout_secs = timeout;
        }
        if let Some(level_str) = file_config.server.log_level {
            self.log_level = parse_log_level(&level_str)?;
        }

        // Merge nested configs
        if let Some(tls_file) = file_config.tls {
            self.tls = Some(TlsConfig {
                cert_path: PathBuf::from(tls_file.cert_path),
                key_path: PathBuf::from(tls_file.key_path),
                ca_path: tls_file.ca_path.map(PathBuf::from),
            });
        }

        if let Some(db_file) = file_config.database {
            self.database = Some(DatabaseConfig {
                url: db_file.url,
                pool_size: db_file.pool_size.unwrap_or(10),
                timeout_secs: db_file.timeout_secs.unwrap_or(30),
            });
        }

        Ok(self)
    }

    pub fn from_env(mut self) -> Result<Self, ConfigError> {
        if let Ok(host) = std::env::var("SERVER_HOST") {
            self.host = Some(host);
        }
        if let Ok(port_str) = std::env::var("SERVER_PORT") {
            let port = port_str.parse()
                .map_err(|_| ConfigError::ValidationFailed("Invalid SERVER_PORT".into()))?;
            self.port = Some(port);
        }
        if let Ok(max_str) = std::env::var("SERVER_MAX_CONNECTIONS") {
            let max = max_str.parse()
                .map_err(|_| ConfigError::ValidationFailed("Invalid SERVER_MAX_CONNECTIONS".into()))?;
            self.max_connections = max;
        }
        if let Ok(timeout_str) = std::env::var("SERVER_TIMEOUT_SECS") {
            let timeout = timeout_str.parse()
                .map_err(|_| ConfigError::ValidationFailed("Invalid SERVER_TIMEOUT_SECS".into()))?;
            self.timeout_secs = timeout;
        }
        if let Ok(level) = std::env::var("SERVER_LOG_LEVEL") {
            self.log_level = parse_log_level(&level)?;
        }

        // Environment variables for nested configs
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            let mut db = self.database.unwrap_or_else(|| DatabaseConfig {
                url: String::new(),
                pool_size: 10,
                timeout_secs: 30,
            });
            db.url = db_url;
            self.database = Some(db);
        }

        Ok(self)
    }

    pub fn merge(mut self, other: ConfigFile) -> Result<Self, ConfigError> {
        // Merge another config source (useful for layered configs)
        // Similar to from_file but merges instead of replacing
        Ok(self)
    }
}
```

**3. Create a complete loading strategy:**
```rust
impl Config {
    pub fn load<Env>() -> Result<Self, ConfigError>
    where
        Env: EnvironmentMarker,
    {
        let builder = ConfigBuilder::<Env>::for_env()
            .from_file("config/default.toml")?
            .from_file(format!("config/{}.toml", Env::name()))?
            .from_env()?;

        // Type state requires host and port, so we need to handle that
        // In practice, you'd have different loading paths for different environments
        Env::finalize_build(builder)
    }
}

pub trait EnvironmentMarker {
    fn name() -> &'static str;
    fn for_env() -> ConfigBuilder<Self, NoHost, NoPort>
    where
        Self: Sized;
    fn finalize_build(builder: ConfigBuilder<Self, Host, Port>) -> Result<Config, ConfigError>
    where
        Self: Sized;
}
```

**Example config file (TOML):**
```toml
[server]
host = "0.0.0.0"
port = 8080
max_connections = 100
timeout_secs = 30
log_level = "info"

[tls]
cert_path = "/etc/ssl/cert.pem"
key_path = "/etc/ssl/key.pem"

[database]
url = "postgres://localhost/myapp"
pool_size = 20
timeout_secs = 60

[cache]
enabled = true
max_size_mb = 512
ttl_secs = 3600
```

**Check/Test:**
- Test loading from TOML file
- Test environment variable overrides
- Test merging multiple config sources
- Test priority order (file < env < explicit)
- Test parsing errors are handled gracefully
- Test missing optional sections work correctly

**Why this isn't enough:**
We can load and merge configurations, but the type-state tracking gets lost when loading from files—everything becomes runtime validation again. Also, no mechanism for watching config changes at runtime (hot reload). The error messages from validation could be better (what exactly was wrong with the config?). We need better error reporting and a way to handle runtime config updates.

---

### Step 6: Add Hot Reload and Comprehensive Error Reporting

**Goal:** Support runtime configuration reloading and provide detailed validation errors.

**What to improve:**

**1. Enhanced error reporting:**
```rust
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    InvalidPort { value: u16, reason: String },
    InvalidTimeout { value: u64, max: u64 },
    InvalidMaxConnections { value: u32, range: (u32, u32) },
    MissingRequired { field: String },
    FileNotFound { path: PathBuf },
    ParseError { line: usize, message: String },
    ValidationFailed { errors: Vec<ValidationError> },
}

#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub suggestion: Option<String>,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::InvalidPort { value, reason } => {
                write!(f, "Invalid port {}: {}", value, reason)?;
                write!(f, "\n  Suggestion: Use a port >= 1024 or standard ports (80, 443)")
            }
            ConfigError::InvalidTimeout { value, max } => {
                write!(f, "Timeout {} exceeds maximum {}", value, max)?;
                write!(f, "\n  Suggestion: Use a value between 1 and {}", max)
            }
            ConfigError::ValidationFailed { errors } => {
                writeln!(f, "Configuration validation failed:")?;
                for (i, error) in errors.iter().enumerate() {
                    writeln!(f, "  {}. {}: {}", i + 1, error.field, error.message)?;
                    if let Some(suggestion) = &error.suggestion {
                        writeln!(f, "     Suggestion: {}", suggestion)?;
                    }
                }
                Ok(())
            }
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for ConfigError {}
```

**2. Comprehensive validation:**
```rust
impl ConfigBuilder<HasHost, HasPort> {
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // Validate host
        if let Some(ref host) = self.host {
            if host.is_empty() {
                errors.push(ValidationError {
                    field: "host".into(),
                    message: "Host cannot be empty".into(),
                    suggestion: Some("Use '0.0.0.0' for all interfaces or 'localhost' for local only".into()),
                });
            }
        }

        // Validate port
        if let Some(port) = self.port {
            if port < 1024 && port != 80 && port != 443 {
                errors.push(ValidationError {
                    field: "port".into(),
                    message: format!("Port {} requires root privileges", port),
                    suggestion: Some("Use a port >= 1024 or run with elevated privileges".into()),
                });
            }
        }

        // Validate max connections
        if self.max_connections == 0 {
            errors.push(ValidationError {
                field: "max_connections".into(),
                message: "Cannot be zero".into(),
                suggestion: Some("Use at least 1, typically 100-1000 for production".into()),
            });
        }

        // Validate TLS for production
        if std::any::type_name::<Self>().contains("Production") {
            if self.tls.is_none() {
                errors.push(ValidationError {
                    field: "tls".into(),
                    message: "TLS is required in production".into(),
                    suggestion: Some("Configure TLS certificates with .tls(...)".into()),
                });
            }
        }

        if !errors.is_empty() {
            return Err(ConfigError::ValidationFailed { errors });
        }

        Ok(())
    }
}
```

**3. Hot reload support:**
```rust
use std::sync::{Arc, RwLock};
use std::time::Duration;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        ConfigManager {
            config: Arc::new(RwLock::new(config)),
            config_path,
        }
    }

    pub fn get(&self) -> Config {
        self.config.read().unwrap().clone()
    }

    pub fn start_watching(&self) -> Result<(), ConfigError> {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(2))
            .map_err(|e| ConfigError::ValidationFailed(vec![ValidationError {
                field: "watcher".into(),
                message: format!("Failed to create watcher: {}", e),
                suggestion: None,
            }]))?;

        watcher.watch(&self.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::ValidationFailed(vec![ValidationError {
                field: "watcher".into(),
                message: format!("Failed to watch config file: {}", e),
                suggestion: None,
            }]))?;

        let config = Arc::clone(&self.config);
        let config_path = self.config_path.clone();

        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(event) => {
                        println!("Config file changed: {:?}", event);
                        match Self::reload_config(&config_path) {
                            Ok(new_config) => {
                                let mut config_guard = config.write().unwrap();
                                *config_guard = new_config;
                                println!("Configuration reloaded successfully");
                            }
                            Err(e) => {
                                eprintln!("Failed to reload config: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Watch error: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn reload_config(path: &Path) -> Result<Config, ConfigError> {
        // Reload and rebuild config
        ConfigBuilder::<Production>::production()
            .from_file(path)?
            .from_env()?
            .build()
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&Config) + Send + 'static,
    {
        // Callback mechanism for config changes
        // Could use channels, callbacks, or async streams
    }
}
```

**4. Testing utilities:**
```rust
#[cfg(test)]
pub mod testing {
    use super::*;

    impl Config {
        pub fn test_config() -> Self {
            ConfigBuilder::<Development>::dev()
                .host("localhost")
                .port(3000)
                .build()
        }

        pub fn with_host(host: &str) -> Self {
            ConfigBuilder::<Development>::dev()
                .host(host)
                .port(3000)
                .build()
        }
    }

    pub fn assert_config_valid(config: &Config) {
        assert!(config.port() >= 1024);
        assert!(config.max_connections() > 0);
        assert!(!config.host().is_empty());
    }
}
```

**Check/Test:**
- Test detailed error messages are helpful
- Test validation catches all invalid states
- Test hot reload detects file changes
- Test concurrent access to shared config
- Test error messages include suggestions
- Benchmark config access performance

**What this achieves:**
Now we have a production-ready configuration system:
- **Type-safe**: Required fields enforced at compile time
- **Flexible**: Load from files, env vars, CLI args
- **Validated**: Comprehensive runtime validation with helpful errors
- **Observable**: Hot reload for runtime config updates
- **Ergonomic**: Fluent builder API with sensible defaults
- **Performant**: Arc<RwLock> for concurrent access

**Extensions to explore:**
- Secret management: Integration with vault/AWS Secrets Manager
- Schema validation: JSON Schema or similar
- Configuration diffing: Show what changed in hot reload
- Audit logging: Track configuration changes
- Remote configuration: Load from consul/etcd
- Feature flags: Runtime feature toggling

---

## Project 2: Type-Safe SQL Query Builder

### Problem Statement

Build a fluent, type-safe SQL query builder that:
- Prevents invalid SQL at compile time (can't ORDER BY before FROM)
- Tracks query state (select → from → where → order by → limit)
- Supports multiple database backends (PostgreSQL, MySQL, SQLite)
- Provides compile-time table/column name validation (where possible)
- Generates parameterized queries (prevents SQL injection)
- Handles complex queries (joins, subqueries, aggregations)
- Returns correctly-typed results based on query structure
- Optimizes query generation at compile time

The builder must ensure that only valid SQL can be constructed, with the type system preventing malformed queries.

### Why It Matters

SQL injection is in the OWASP Top 10. Hand-written SQL is error-prone:
- **Security**: Parameterized queries prevent SQL injection
- **Correctness**: Invalid SQL caught at compile time, not runtime
- **Refactoring**: Rename columns, compiler finds all uses
- **Type Safety**: Query results typed based on selected columns
- **Maintainability**: Self-documenting query construction

Type-safe query builders appear in:
- **ORMs**: Diesel, SeaORM, SQLx use type-level SQL
- **GraphQL**: Query structure validated before execution
- **Database Migrations**: Schema changes verified at compile time

### Use Cases

1. **Web Applications**: CRUD operations with compile-time safety
2. **Admin Dashboards**: Complex filtering and reporting queries
3. **API Backends**: Safe query construction from user input
4. **Data Analytics**: Complex aggregations and joins
5. **Microservices**: Database queries with type guarantees
6. **CLI Tools**: Database exploration and querying
7. **ETL Pipelines**: Data extraction with validated queries

### Solution Outline

**Core Type-State Structure:**
```rust
// Query states
pub struct Empty;
pub struct HasSelect;
pub struct HasFrom;
pub struct HasWhere;
pub struct Finalized;

pub struct Query<State, Backend> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by_fields: Vec<String>,
    limit_value: Option<usize>,
    params: Vec<QueryParam>,
    _state: PhantomData<State>,
    _backend: PhantomData<Backend>,
}

// Only certain methods available in certain states
impl<B> Query<Empty, B> {
    pub fn select(fields: &[&str]) -> Query<HasSelect, B> { /* ... */ }
}

impl<B> Query<HasSelect, B> {
    pub fn from(table: &str) -> Query<HasFrom, B> { /* ... */ }
}

impl<B> Query<HasFrom, B> {
    pub fn where_clause(condition: &str) -> Query<HasWhere, B> { /* ... */ }
    pub fn finalize(self) -> Query<Finalized, B> { /* ... */ }
}

impl<B: Backend> Query<Finalized, B> {
    pub fn to_sql(&self) -> String { /* ... */ }
}
```

**Key Features:**
- **Type-State**: Track query construction stages
- **Backend Abstraction**: Different SQL dialects
- **Parameterization**: Safe value binding
- **Compile-Time Validation**: Wrong order = compile error
- **Fluent API**: Readable query construction

**SQL Generation:**
```rust
// PostgreSQL: SELECT * FROM users WHERE id = $1
// MySQL: SELECT * FROM users WHERE id = ?
// SQLite: SELECT * FROM users WHERE id = ?
```

### Testing Hints

**Compile-Time Tests:**
```rust
// Should NOT compile
fn invalid_query() {
    let q = Query::select(&["id", "name"])
        .where_clause("age > 18")  // ERROR: can't WHERE without FROM
        .to_sql();
}

// Should compile
fn valid_query() {
    let q = Query::select(&["id", "name"])
        .from("users")
        .where_clause("age > ?")
        .order_by("name")
        .limit(10)
        .to_sql();
}
```

**Runtime Tests:**
```rust
#[test]
fn test_simple_select() {
    let sql = Query::select(&["*"])
        .from("users")
        .to_sql();

    assert_eq!(sql, "SELECT * FROM users");
}

#[test]
fn test_parameterized_query() {
    let sql = Query::select(&["id", "name"])
        .from("users")
        .where_clause("age > ?")
        .bind(18)
        .to_sql();

    assert_eq!(sql, "SELECT id, name FROM users WHERE age > $1");
}

#[test]
fn test_backend_differences() {
    let pg_query = Query::<_, PostgreSQL>::select(&["id"]).from("users").to_sql();
    let mysql_query = Query::<_, MySQL>::select(&["id"]).from("users").to_sql();

    assert!(pg_query.contains("$1"));
    assert!(mysql_query.contains("?"));
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Query Builder with String Concatenation

**Goal:** Create a working query builder using simple string concatenation.

**What to implement:**
```rust
pub struct Query {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            select_fields: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
        }
    }

    pub fn select(mut self, fields: &[&str]) -> Self {
        self.select_fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn from(mut self, table: &str) -> Self {
        self.from_table = Some(table.to_string());
        self
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by.push(field.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::from("SELECT ");

        if self.select_fields.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.select_fields.join(", "));
        }

        if let Some(ref table) = self.from_table {
            sql.push_str(&format!(" FROM {}", table));
        }

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }

        sql
    }
}
```

**Check/Test:**
- Test simple SELECT * FROM table
- Test with WHERE clauses
- Test ORDER BY and LIMIT
- Test method chaining
- Verify generated SQL is correct

**Why this isn't enough:**
This naive implementation allows invalid SQL. You can call `where_clause()` before `from()`, or call `order_by()` without a `from()`. The type system doesn't prevent SQL injection—users can pass raw strings with malicious content. No parameterization means values are directly concatenated into queries. We need type-state to enforce correct ordering and parameterization for safety.

---

### Step 2: Add Type-State for Query Stages

**Goal:** Use phantom types to enforce correct query construction order.

**What to improve:**
```rust
use std::marker::PhantomData;

// State markers
pub struct Empty;
pub struct HasSelect;
pub struct HasFrom;

pub struct Query<State> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    _state: PhantomData<State>,
}

// Start with SELECT
impl Query<Empty> {
    pub fn new() -> Self {
        Query {
            select_fields: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            _state: PhantomData,
        }
    }

    pub fn select(self, fields: &[&str]) -> Query<HasSelect> {
        Query {
            select_fields: fields.iter().map(|s| s.to_string()).collect(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            _state: PhantomData,
        }
    }
}

// After SELECT, must have FROM
impl Query<HasSelect> {
    pub fn from(self, table: &str) -> Query<HasFrom> {
        Query {
            select_fields: self.select_fields,
            from_table: Some(table.to_string()),
            where_clauses: self.where_clauses,
            order_by: self.order_by,
            limit: self.limit,
            _state: PhantomData,
        }
    }
}

// After FROM, can add WHERE, ORDER BY, LIMIT
impl Query<HasFrom> {
    pub fn where_clause(mut self, condition: &str) -> Self {
        self.where_clauses.push(condition.to_string());
        self
    }

    pub fn order_by(mut self, field: &str) -> Self {
        self.order_by.push(field.to_string());
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn to_sql(&self) -> String {
        // Same as before
        let mut sql = String::from("SELECT ");
        if self.select_fields.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.select_fields.join(", "));
        }
        if let Some(ref table) = self.from_table {
            sql.push_str(&format!(" FROM {}", table));
        }
        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }
        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }
        sql
    }
}
```

**Check/Test:**
- Verify cannot call `where_clause()` before `from()` (compile error)
- Verify cannot call `order_by()` on `Query<HasSelect>` (compile error)
- Test that valid query order compiles
- Verify `to_sql()` only available after `from()`

**Why this isn't enough:**
Type-state prevents ordering errors, but we still have no parameterization—SQL injection is still possible. We're also limited to a single query type (SELECT). No support for INSERT, UPDATE, DELETE. No backend abstraction for different SQL dialects (PostgreSQL uses $1, MySQL uses ?). We need parameterized queries and backend support.

---

### Step 3: Add Parameterized Queries and SQL Injection Prevention

**Goal:** Support safe parameter binding to prevent SQL injection.

**What to improve:**
```rust
#[derive(Debug, Clone)]
pub enum QueryParam {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

pub struct Query<State> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    params: Vec<QueryParam>,
    _state: PhantomData<State>,
}

impl Query<HasFrom> {
    // Instead of raw string, use placeholders
    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("{} = ${}", field, param_index));
        self.params.push(value.into());
        self
    }

    pub fn where_gt(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("{} > ${}", field, param_index));
        self.params.push(value.into());
        self
    }

    pub fn where_lt(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        self.where_clauses.push(format!("{} < ${}", field, param_index));
        self.params.push(value.into());
        self
    }

    pub fn where_in(mut self, field: &str, values: &[impl Clone + Into<QueryParam>]) -> Self {
        let start_index = self.params.len() + 1;
        let placeholders: Vec<String> = (start_index..start_index + values.len())
            .map(|i| format!("${}", i))
            .collect();

        self.where_clauses.push(format!("{} IN ({})", field, placeholders.join(", ")));

        for value in values {
            self.params.push(value.clone().into());
        }

        self
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.params.clone())
    }
}

// Implement Into<QueryParam> for common types
impl From<String> for QueryParam {
    fn from(s: String) -> Self {
        QueryParam::String(s)
    }
}

impl From<&str> for QueryParam {
    fn from(s: &str) -> Self {
        QueryParam::String(s.to_string())
    }
}

impl From<i64> for QueryParam {
    fn from(i: i64) -> Self {
        QueryParam::Int(i)
    }
}

impl From<i32> for QueryParam {
    fn from(i: i32) -> Self {
        QueryParam::Int(i as i64)
    }
}

impl From<f64> for QueryParam {
    fn from(f: f64) -> Self {
        QueryParam::Float(f)
    }
}

impl From<bool> for QueryParam {
    fn from(b: bool) -> Self {
        QueryParam::Bool(b)
    }
}
```

**Usage:**
```rust
let (sql, params) = Query::new()
    .select(&["id", "name", "email"])
    .from("users")
    .where_eq("active", true)
    .where_gt("age", 18)
    .where_in("status", &["active", "pending"])
    .to_sql_with_params();

// sql: "SELECT id, name, email FROM users WHERE active = $1 AND age > $2 AND status IN ($3, $4)"
// params: [Bool(true), Int(18), String("active"), String("pending")]
```

**Check/Test:**
- Test parameterized queries generate correct placeholders
- Test parameter values are collected correctly
- Test various data types (string, int, float, bool)
- Test IN clause with multiple values
- Verify user input cannot inject SQL

**Why this isn't enough:**
We have parameterization for PostgreSQL-style ($1, $2), but different databases use different placeholder syntax. MySQL and SQLite use `?`, Oracle uses `:1, :2`. We also only support SELECT queries—no INSERT, UPDATE, DELETE. The query builder is growing but lacks proper backend abstraction and query type generality.

---

### Step 4: Add Backend Abstraction for Multiple Databases

**Goal:** Support multiple SQL dialects (PostgreSQL, MySQL, SQLite).

**What to improve:**
```rust
// Backend trait
pub trait Backend {
    fn placeholder(index: usize) -> String;
    fn quote_identifier(name: &str) -> String;
}

pub struct PostgreSQL;
pub struct MySQL;
pub struct SQLite;

impl Backend for PostgreSQL {
    fn placeholder(index: usize) -> String {
        format!("${}", index)
    }

    fn quote_identifier(name: &str) -> String {
        format!("\"{}\"", name)
    }
}

impl Backend for MySQL {
    fn placeholder(_index: usize) -> String {
        "?".to_string()
    }

    fn quote_identifier(name: &str) -> String {
        format!("`{}`", name)
    }
}

impl Backend for SQLite {
    fn placeholder(_index: usize) -> String {
        "?".to_string()
    }

    fn quote_identifier(name: &str) -> String {
        format!("\"{}\"", name)
    }
}

// Add backend to query state
pub struct Query<State, Backend> {
    select_fields: Vec<String>,
    from_table: Option<String>,
    where_clauses: Vec<String>,
    order_by: Vec<String>,
    limit: Option<usize>,
    params: Vec<QueryParam>,
    _state: PhantomData<State>,
    _backend: PhantomData<Backend>,
}

impl<B: Backend> Query<Empty, B> {
    pub fn new() -> Self {
        Query {
            select_fields: Vec::new(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            params: Vec::new(),
            _state: PhantomData,
            _backend: PhantomData,
        }
    }

    pub fn select(self, fields: &[&str]) -> Query<HasSelect, B> {
        Query {
            select_fields: fields.iter().map(|s| s.to_string()).collect(),
            from_table: None,
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            params: Vec::new(),
            _state: PhantomData,
            _backend: PhantomData,
        }
    }
}

impl<B: Backend> Query<HasFrom, B> {
    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_field = B::quote_identifier(field);
        self.where_clauses.push(format!("{} = {}", quoted_field, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = String::from("SELECT ");

        if self.select_fields.is_empty() {
            sql.push('*');
        } else {
            let quoted_fields: Vec<String> = self.select_fields
                .iter()
                .map(|f| B::quote_identifier(f))
                .collect();
            sql.push_str(&quoted_fields.join(", "));
        }

        if let Some(ref table) = self.from_table {
            sql.push_str(&format!(" FROM {}", B::quote_identifier(table)));
        }

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let quoted_order: Vec<String> = self.order_by
                .iter()
                .map(|f| B::quote_identifier(f))
                .collect();
            sql.push_str(&quoted_order.join(", "));
        }

        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }

        sql
    }
}
```

**Type-safe usage:**
```rust
// PostgreSQL query
let pg_query = Query::<_, PostgreSQL>::new()
    .select(&["id", "name"])
    .from("users")
    .where_eq("active", true)
    .to_sql();
// SELECT "id", "name" FROM "users" WHERE "active" = $1

// MySQL query
let mysql_query = Query::<_, MySQL>::new()
    .select(&["id", "name"])
    .from("users")
    .where_eq("active", true)
    .to_sql();
// SELECT `id`, `name` FROM `users` WHERE `active` = ?
```

**Check/Test:**
- Test PostgreSQL generates $1, $2, etc.
- Test MySQL generates ? placeholders
- Test SQLite generates ? placeholders
- Test identifier quoting differs by backend
- Test same query code works for all backends

**Why this isn't enough:**
We support multiple backends, but only for SELECT queries. Real applications need INSERT, UPDATE, DELETE. Also, no support for JOINs, subqueries, or aggregations (COUNT, SUM, AVG). The builder is growing complex but lacks full SQL feature support. We need more query types and advanced features.

---

### Step 5: Add INSERT, UPDATE, DELETE and JOIN Support

**Goal:** Support full CRUD operations and table joins.

**What to improve:**

**1. INSERT queries:**
```rust
pub struct InsertQuery<Backend> {
    table: String,
    columns: Vec<String>,
    values: Vec<QueryParam>,
    _backend: PhantomData<Backend>,
}

impl<B: Backend> InsertQuery<B> {
    pub fn into(table: &str) -> Self {
        InsertQuery {
            table: table.to_string(),
            columns: Vec::new(),
            values: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub fn column(mut self, name: &str, value: impl Into<QueryParam>) -> Self {
        self.columns.push(name.to_string());
        self.values.push(value.into());
        self
    }

    pub fn columns(mut self, cols: &[(&str, impl Clone + Into<QueryParam>)]) -> Self {
        for (name, value) in cols {
            self.columns.push(name.to_string());
            self.values.push(value.clone().into());
        }
        self
    }

    pub fn to_sql(&self) -> String {
        let quoted_table = B::quote_identifier(&self.table);
        let quoted_cols: Vec<String> = self.columns
            .iter()
            .map(|c| B::quote_identifier(c))
            .collect();

        let placeholders: Vec<String> = (1..=self.values.len())
            .map(|i| B::placeholder(i))
            .collect();

        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            quoted_table,
            quoted_cols.join(", "),
            placeholders.join(", ")
        )
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.values.clone())
    }
}
```

**2. UPDATE queries:**
```rust
pub struct UpdateQuery<B: Backend> {
    table: String,
    set_clauses: Vec<String>,
    where_clauses: Vec<String>,
    params: Vec<QueryParam>,
    _backend: PhantomData<B>,
}

impl<B: Backend> UpdateQuery<B> {
    pub fn table(table: &str) -> Self {
        UpdateQuery {
            table: table.to_string(),
            set_clauses: Vec::new(),
            where_clauses: Vec::new(),
            params: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub fn set(mut self, column: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_col = B::quote_identifier(column);
        self.set_clauses.push(format!("{} = {}", quoted_col, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_field = B::quote_identifier(field);
        self.where_clauses.push(format!("{} = {}", quoted_field, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("UPDATE {}", B::quote_identifier(&self.table));

        if !self.set_clauses.is_empty() {
            sql.push_str(" SET ");
            sql.push_str(&self.set_clauses.join(", "));
        }

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        sql
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.params.clone())
    }
}
```

**3. DELETE queries:**
```rust
pub struct DeleteQuery<B: Backend> {
    table: String,
    where_clauses: Vec<String>,
    params: Vec<QueryParam>,
    _backend: PhantomData<B>,
}

impl<B: Backend> DeleteQuery<B> {
    pub fn from(table: &str) -> Self {
        DeleteQuery {
            table: table.to_string(),
            where_clauses: Vec::new(),
            params: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub fn where_eq(mut self, field: &str, value: impl Into<QueryParam>) -> Self {
        let param_index = self.params.len() + 1;
        let placeholder = B::placeholder(param_index);
        let quoted_field = B::quote_identifier(field);
        self.where_clauses.push(format!("{} = {}", quoted_field, placeholder));
        self.params.push(value.into());
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("DELETE FROM {}", B::quote_identifier(&self.table));

        if !self.where_clauses.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_clauses.join(" AND "));
        }

        sql
    }

    pub fn to_sql_with_params(&self) -> (String, Vec<QueryParam>) {
        (self.to_sql(), self.params.clone())
    }
}
```

**4. JOIN support for SELECT:**
```rust
impl<B: Backend> Query<HasFrom, B> {
    pub fn inner_join(mut self, table: &str, on_condition: &str) -> Self {
        let join_clause = format!(
            "INNER JOIN {} ON {}",
            B::quote_identifier(table),
            on_condition
        );
        // Store in a joins vector (add to struct)
        self
    }

    pub fn left_join(mut self, table: &str, on_condition: &str) -> Self {
        let join_clause = format!(
            "LEFT JOIN {} ON {}",
            B::quote_identifier(table),
            on_condition
        );
        self
    }
}
```

**Usage examples:**
```rust
// INSERT
let (sql, params) = InsertQuery::<PostgreSQL>::into("users")
    .column("name", "Alice")
    .column("email", "alice@example.com")
    .column("age", 30)
    .to_sql_with_params();

// UPDATE
let (sql, params) = UpdateQuery::<PostgreSQL>::table("users")
    .set("name", "Bob")
    .set("email", "bob@example.com")
    .where_eq("id", 1)
    .to_sql_with_params();

// DELETE
let (sql, params) = DeleteQuery::<PostgreSQL>::from("users")
    .where_eq("id", 1)
    .to_sql_with_params();

// JOIN
let sql = Query::<_, PostgreSQL>::new()
    .select(&["users.name", "orders.total"])
    .from("users")
    .inner_join("orders", "users.id = orders.user_id")
    .where_eq("users.active", true)
    .to_sql();
```

**Check/Test:**
- Test INSERT generates correct SQL
- Test UPDATE with multiple SET clauses
- Test DELETE with WHERE clause
- Test INNER JOIN and LEFT JOIN
- Test parameterization for all query types
- Verify backend-specific syntax

**Why this isn't enough:**
We now have full CRUD and joins, but no support for aggregations (COUNT, SUM, GROUP BY, HAVING). No transactions support. No query execution—just SQL generation. The builder is getting quite large and complex. We need better organization and actual database integration.

---

### Step 6: Add Aggregations, Transactions, and Query Execution

**Goal:** Complete the query builder with aggregations, transactions, and actual database execution.

**What to improve:**

**1. Aggregation support:**
```rust
pub enum Aggregation {
    Count { field: String, alias: Option<String> },
    Sum { field: String, alias: Option<String> },
    Avg { field: String, alias: Option<String> },
    Max { field: String, alias: Option<String> },
    Min { field: String, alias: Option<String> },
}

impl<B: Backend> Query<HasFrom, B> {
    pub fn count(mut self, field: &str, alias: Option<&str>) -> Self {
        let count_expr = if field == "*" {
            "COUNT(*)".to_string()
        } else {
            format!("COUNT({})", B::quote_identifier(field))
        };

        let full_expr = if let Some(alias) = alias {
            format!("{} AS {}", count_expr, B::quote_identifier(alias))
        } else {
            count_expr
        };

        self.select_fields.push(full_expr);
        self
    }

    pub fn sum(mut self, field: &str, alias: Option<&str>) -> Self {
        let sum_expr = format!("SUM({})", B::quote_identifier(field));
        let full_expr = if let Some(alias) = alias {
            format!("{} AS {}", sum_expr, B::quote_identifier(alias))
        } else {
            sum_expr
        };
        self.select_fields.push(full_expr);
        self
    }

    pub fn group_by(mut self, field: &str) -> Self {
        // Add group_by field to struct
        self
    }

    pub fn having(mut self, condition: &str) -> Self {
        // Add having clause to struct
        self
    }
}
```

**2. Transaction support:**
```rust
pub struct Transaction<B: Backend> {
    connection: Connection, // Actual DB connection
    _backend: PhantomData<B>,
}

impl<B: Backend> Transaction<B> {
    pub fn begin(conn: Connection) -> Result<Self, QueryError> {
        // Execute BEGIN
        conn.execute("BEGIN")?;
        Ok(Transaction {
            connection: conn,
            _backend: PhantomData,
        })
    }

    pub fn execute_query<S>(&mut self, query: &Query<HasFrom, B>) -> Result<Vec<Row>, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn execute_insert(&mut self, query: &InsertQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)?;
        Ok(self.connection.last_insert_id())
    }

    pub fn execute_update(&mut self, query: &UpdateQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        let rows_affected = self.connection.execute_with_params(&sql, &params)?;
        Ok(rows_affected)
    }

    pub fn commit(self) -> Result<(), QueryError> {
        self.connection.execute("COMMIT")?;
        Ok(())
    }

    pub fn rollback(self) -> Result<(), QueryError> {
        self.connection.execute("ROLLBACK")?;
        Ok(())
    }
}

#[must_use = "Transaction must be committed or rolled back"]
impl<B: Backend> Drop for Transaction<B> {
    fn drop(&mut self) {
        // Auto-rollback if not explicitly committed
        let _ = self.connection.execute("ROLLBACK");
    }
}
```

**3. Query execution:**
```rust
pub struct QueryExecutor<B: Backend> {
    connection: Connection,
    _backend: PhantomData<B>,
}

impl<B: Backend> QueryExecutor<B> {
    pub fn new(connection: Connection) -> Self {
        QueryExecutor {
            connection,
            _backend: PhantomData,
        }
    }

    pub fn execute<S>(&self, query: &Query<HasFrom, B>) -> Result<Vec<Row>, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn execute_one<S>(&self, query: &Query<HasFrom, B>) -> Result<Option<Row>, QueryError> {
        let mut results = self.execute(query)?;
        Ok(results.pop())
    }

    pub fn execute_insert(&self, query: &InsertQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)?;
        Ok(self.connection.last_insert_id())
    }

    pub fn execute_update(&self, query: &UpdateQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn execute_delete(&self, query: &DeleteQuery<B>) -> Result<u64, QueryError> {
        let (sql, params) = query.to_sql_with_params();
        self.connection.execute_with_params(&sql, &params)
    }

    pub fn begin_transaction(&self) -> Result<Transaction<B>, QueryError> {
        Transaction::begin(self.connection.clone())
    }
}
```

**4. Macro for type-safe table/column definitions:**
```rust
macro_rules! define_table {
    ($table_name:ident { $($column:ident: $type:ty),* $(,)? }) => {
        pub struct $table_name;

        impl $table_name {
            pub fn table_name() -> &'static str {
                stringify!($table_name)
            }

            $(
                pub fn $column() -> Column<$type> {
                    Column {
                        name: stringify!($column),
                        _type: PhantomData,
                    }
                }
            )*
        }
    };
}

pub struct Column<T> {
    name: &'static str,
    _type: PhantomData<T>,
}

// Usage
define_table!(users {
    id: i64,
    name: String,
    email: String,
    age: i32,
    active: bool,
});

// Type-safe query
let query = Query::<_, PostgreSQL>::new()
    .select(&[users::id(), users::name(), users::email()])
    .from(users::table_name())
    .where_eq(users::active(), true)
    .where_gt(users::age(), 18);
```

**Complete usage example:**
```rust
let executor = QueryExecutor::<PostgreSQL>::new(connection);

// Select with aggregation
let query = Query::new()
    .select(&["status"])
    .count("*", Some("total"))
    .from("orders")
    .group_by("status")
    .having("COUNT(*) > 10");

let results = executor.execute(&query)?;

// Transaction
let mut tx = executor.begin_transaction()?;

let insert = InsertQuery::into("users")
    .column("name", "Charlie")
    .column("email", "charlie@example.com");

let user_id = tx.execute_insert(&insert)?;

let update = UpdateQuery::table("profiles")
    .set("user_id", user_id)
    .where_eq("email", "charlie@example.com");

tx.execute_update(&update)?;

tx.commit()?; // Must commit or auto-rollback on drop
```

**Check/Test:**
- Test aggregation functions generate correct SQL
- Test GROUP BY and HAVING clauses
- Test transaction commit and rollback
- Test auto-rollback on Drop
- Test query execution with real database connection
- Test type-safe column references with macro
- Benchmark query execution performance

**What this achieves:**
A production-ready, type-safe SQL query builder:
- **Type-Safe**: Invalid query order prevented at compile time
- **SQL Injection Prevention**: All values parameterized
- **Multi-Backend**: PostgreSQL, MySQL, SQLite support
- **Full CRUD**: SELECT, INSERT, UPDATE, DELETE
- **Advanced Features**: Joins, aggregations, transactions
- **Safe Transactions**: Must commit or auto-rollback
- **Ergonomic**: Fluent API, macro-generated type-safe tables
- **Performant**: Zero-cost abstractions, prepared statements

**Extensions to explore:**
- Async query execution with tokio/async-std
- Connection pooling integration
- Query result deserialization to structs
- Compile-time schema validation
- Query plan optimization hints
- Streaming result iterators for large datasets

---

## Project 3: Resource Manager with Must-Use Handles

### Problem Statement

Create a resource management system that enforces proper resource lifecycle through the type system:
- File handles that must be explicitly closed or flushed
- Database transactions that must commit or rollback
- Lock guards that must be held for their scope
- Network connections that must be properly shut down
- Temporary files/directories that must be cleaned up
- Resource pools that prevent leaks
- Linear types ensuring resources used exactly once
- Compile-time warnings for unused resources

The system must make resource leaks impossible through type-level guarantees and #[must_use] attributes.

### Why It Matters

Resource leaks are among the most common bugs:
- **File Descriptors**: Leaking file handles exhausts system resources
- **Database Connections**: Connection pool exhaustion crashes services
- **Memory**: Not explicitly managed resources cause memory leaks
- **Locks**: Forgetting to release locks causes deadlocks
- **Sockets**: Leaking connections wastes network resources

Type-safe resource management appears in:
- **Operating Systems**: Kernel resource management
- **Databases**: Connection and transaction management
- **Web Servers**: Request/response lifecycle
- **Game Engines**: Asset loading and unloading

### Use Cases

1. **File Operations**: Ensure files are closed, buffers flushed
2. **Database Systems**: Transactions committed/rolled back properly
3. **Network Services**: Connections gracefully closed
4. **Temporary Resources**: Temp files/dirs cleaned up automatically
5. **Distributed Systems**: Distributed locks released properly
6. **Resource Pools**: Connections returned to pool
7. **Streaming Data**: Ensure streams are completed or aborted

### Solution Outline

**Core Structure:**
```rust
// Resource states
pub struct Acquired;
pub struct Used;
pub struct Released;

#[must_use = "resource must be explicitly released"]
pub struct Resource<T, State> {
    inner: Option<T>,
    _state: PhantomData<State>,
}

impl<T> Resource<T, Acquired> {
    pub fn new(resource: T) -> Self { /* ... */ }
    pub fn use_resource(self) -> Resource<T, Used> { /* ... */ }
}

impl<T> Resource<T, Used> {
    pub fn release(self) -> Result<(), Error> { /* ... */ }
}

// Automatic cleanup on drop
impl<T: Drop, S> Drop for Resource<T, S> {
    fn drop(&mut self) {
        if let Some(resource) = self.inner.take() {
            // Cleanup based on state
        }
    }
}
```

**Key Features:**
- **#[must_use]**: Compiler warnings for unused resources
- **Type-State**: Track resource lifecycle
- **RAII**: Automatic cleanup on scope exit
- **Linear Types**: Resources used exactly once
- **Guards**: Scope-based resource management

### Testing Hints

**Compile-Time Tests:**
```rust
// Should warn
fn unused_resource() {
    let file = File::create("test.txt"); // WARNING: unused File
}

// Should compile
fn proper_usage() {
    let file = File::create("test.txt")?;
    file.write_all(b"data")?;
    file.sync_all()?;
    // Auto-closes on drop
}
```

**Runtime Tests:**
```rust
#[test]
fn test_resource_cleanup() {
    let path = "/tmp/test_file";
    {
        let file = ManagedFile::create(path).unwrap();
        file.write(b"test").unwrap();
        // File auto-closed here
    }
    assert!(std::fs::metadata(path).is_ok());
}

#[test]
fn test_transaction_rollback() {
    let tx = Transaction::begin().unwrap();
    tx.execute("INSERT ...").unwrap();
    // Drop without commit = auto-rollback
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic File Handle with Manual Cleanup

**Goal:** Create a file wrapper that requires explicit close().

**What to implement:**
```rust
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct ManagedFile {
    file: Option<File>,
    path: PathBuf,
}

impl ManagedFile {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::create(&path)?;
        Ok(ManagedFile {
            file: Some(file),
            path,
        })
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        Ok(ManagedFile {
            file: Some(file),
            path,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            file.write_all(data)?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "File already closed"))
        }
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut file) = self.file {
            file.flush()?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "File already closed"))
        }
    }

    pub fn close(mut self) -> io::Result<()> {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
            // File drops here, closing the handle
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "File already closed"))
        }
    }
}
```

**Check/Test:**
- Test file creation and writing
- Test explicit close
- Test error when writing after close
- Test flush before close

**Why this isn't enough:**
Users can forget to call `close()`, leading to unflushed buffers. The API allows calling `write()` after `close()`, requiring runtime checks. No compiler warning if `close()` is never called. We need `#[must_use]` and better type-state tracking to prevent misuse.

---

### Step 2: Add #[must_use] and Type-State

**Goal:** Use #[must_use] and phantom types to enforce proper usage.

**What to improve:**
```rust
use std::marker::PhantomData;

pub struct Open;
pub struct Closed;

#[must_use = "file must be explicitly closed or it will be closed on drop"]
pub struct ManagedFile<State> {
    file: Option<File>,
    path: PathBuf,
    _state: PhantomData<State>,
}

impl ManagedFile<Open> {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::create(&path)?;
        Ok(ManagedFile {
            file: Some(file),
            path,
            _state: PhantomData,
        })
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        // file is guaranteed to be Some in Open state
        self.file.as_mut().unwrap().write_all(data)
    }

    pub fn flush(&mut self) -> io::Result<()> {
        self.file.as_mut().unwrap().flush()
    }

    pub fn close(mut self) -> io::Result<ManagedFile<Closed>> {
        self.flush()?;
        let file = self.file.take().unwrap();
        drop(file); // Explicit close

        Ok(ManagedFile {
            file: None,
            path: self.path,
            _state: PhantomData,
        })
    }
}

impl ManagedFile<Closed> {
    // Closed files can't do anything
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// Auto-cleanup on drop
impl<S> Drop for ManagedFile<S> {
    fn drop(&mut self) {
        if let Some(mut file) = self.file.take() {
            let _ = file.flush();
            // File closes on drop
        }
    }
}
```

**Check/Test:**
- Verify #[must_use] generates warning if file unused
- Test cannot call write() on Closed file (compile error)
- Test auto-flush on drop
- Test explicit close transitions state

**Why this isn't enough:**
We have type-state and #[must_use], but only for files. Real systems have many resource types: database connections, network sockets, temporary directories, lock guards. We need a generic resource management framework. Also, no resource pools or scoped guards yet.

---

### Step 3: Create Generic Resource Manager

**Goal:** Build a generic framework for any resource type.

**What to improve:**
```rust
// Resource lifecycle trait
pub trait Resource {
    type Error;

    fn cleanup(&mut self) -> Result<(), Self::Error>;
}

// Generic resource manager
pub struct Acquired;
pub struct Released;

#[must_use = "resource must be explicitly released or will auto-release on drop"]
pub struct Managed<R: Resource, State> {
    resource: Option<R>,
    _state: PhantomData<State>,
}

impl<R: Resource> Managed<R, Acquired> {
    pub fn new(resource: R) -> Self {
        Managed {
            resource: Some(resource),
            _state: PhantomData,
        }
    }

    pub fn get(&self) -> &R {
        self.resource.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.resource.as_mut().unwrap()
    }

    pub fn release(mut self) -> Result<Managed<R, Released>, R::Error> {
        let mut resource = self.resource.take().unwrap();
        resource.cleanup()?;

        Ok(Managed {
            resource: Some(resource),
            _state: PhantomData,
        })
    }

    pub fn into_inner(mut self) -> R {
        self.resource.take().unwrap()
    }
}

impl<R: Resource> Drop for Managed<R, Acquired> {
    fn drop(&mut self) {
        if let Some(mut resource) = self.resource.take() {
            let _ = resource.cleanup();
        }
    }
}

// Implement Resource for File
impl Resource for File {
    type Error = io::Error;

    fn cleanup(&mut self) -> Result<(), Self::Error> {
        self.flush()
    }
}

// Implement for other types
pub struct DbConnection {
    // ...
}

impl Resource for DbConnection {
    type Error = DbError;

    fn cleanup(&mut self) -> Result<(), Self::Error> {
        // Close connection
        Ok(())
    }
}
```

**Usage:**
```rust
let file = Managed::new(File::create("test.txt")?);
file.get_mut().write_all(b"data")?;
file.release()?; // Explicit release

// Or auto-release on drop
{
    let file = Managed::new(File::create("test.txt")?);
    file.get_mut().write_all(b"data")?;
} // Auto-cleanup here
```

**Check/Test:**
- Test generic resource management with File
- Test with database connections
- Test auto-cleanup on drop
- Test explicit release
- Verify #[must_use] works for generic type

**Why this isn't enough:**
We have generic resource management, but no scoped guards (RAII pattern). No resource pools for reusing expensive resources like database connections. No temporary resource pattern (temp files that auto-delete). These are common patterns needed in real systems.

---

### Step 4: Add Scoped Guards and RAII Pattern

**Goal:** Implement scope-based resource management with guards.

**What to improve:**
```rust
// Guard that automatically releases on scope exit
pub struct Guard<R: Resource> {
    resource: Option<R>,
}

impl<R: Resource> Guard<R> {
    pub fn new(resource: R) -> Self {
        Guard {
            resource: Some(resource),
        }
    }

    pub fn get(&self) -> &R {
        self.resource.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.resource.as_mut().unwrap()
    }
}

impl<R: Resource> Drop for Guard<R> {
    fn drop(&mut self) {
        if let Some(mut resource) = self.resource.take() {
            let _ = resource.cleanup();
        }
    }
}

impl<R: Resource> Deref for Guard<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<R: Resource> DerefMut for Guard<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

// Lock guard pattern
pub struct Mutex<T> {
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Mutex {
            data: UnsafeCell::new(data),
            locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self.locked.compare_exchange(
            false,
            true,
            Ordering::Acquire,
            Ordering::Relaxed,
        ).is_err() {
            std::hint::spin_loop();
        }

        MutexGuard { mutex: self }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}
```

**Temporary file with auto-delete:**
```rust
pub struct TempFile {
    path: PathBuf,
    file: File,
}

impl TempFile {
    pub fn new() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!("temp_{}", uuid::Uuid::new_v4()));
        let file = File::create(&path)?;
        Ok(TempFile { path, file })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.file.write_all(data)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

impl Resource for TempFile {
    type Error = io::Error;

    fn cleanup(&mut self) -> Result<(), Self::Error> {
        self.file.flush()?;
        std::fs::remove_file(&self.path)?;
        Ok(())
    }
}
```

**Check/Test:**
- Test guard auto-releases on scope exit
- Test Deref/DerefMut make guard transparent
- Test mutex guard releases lock on drop
- Test temp file auto-deletes on drop
- Test guard with early return still cleans up

**Why this isn't enough:**
Guards work well for single resources, but what about resource pools? Database connection pools are critical for performance—creating a connection each time is too slow. We need pooling with automatic return-to-pool semantics. Also, no async support for async/await code.

---

### Step 5: Implement Resource Pool with Auto-Return

**Goal:** Create a connection pool that automatically returns resources.

**What to improve:**
```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct Pool<R> {
    available: Arc<Mutex<VecDeque<R>>>,
    max_size: usize,
    factory: Arc<dyn Fn() -> Result<R, PoolError> + Send + Sync>,
}

pub struct PooledResource<R> {
    resource: Option<R>,
    pool: Arc<Mutex<VecDeque<R>>>,
}

#[derive(Debug)]
pub enum PoolError {
    CreationFailed(String),
    PoolExhausted,
}

impl<R> Pool<R> {
    pub fn new<F>(max_size: usize, factory: F) -> Self
    where
        F: Fn() -> Result<R, PoolError> + Send + Sync + 'static,
    {
        Pool {
            available: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            factory: Arc::new(factory),
        }
    }

    pub fn get(&self) -> Result<PooledResource<R>, PoolError> {
        let mut pool = self.available.lock().unwrap();

        let resource = if let Some(resource) = pool.pop_front() {
            resource
        } else if pool.len() < self.max_size {
            drop(pool); // Release lock while creating
            (self.factory)()?
        } else {
            return Err(PoolError::PoolExhausted);
        };

        Ok(PooledResource {
            resource: Some(resource),
            pool: Arc::clone(&self.available),
        })
    }

    pub fn size(&self) -> usize {
        self.available.lock().unwrap().len()
    }
}

impl<R> PooledResource<R> {
    pub fn get(&self) -> &R {
        self.resource.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.resource.as_mut().unwrap()
    }
}

// Automatic return to pool on drop
impl<R> Drop for PooledResource<R> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let mut pool = self.pool.lock().unwrap();
            pool.push_back(resource);
        }
    }
}

impl<R> Deref for PooledResource<R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<R> DerefMut for PooledResource<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}
```

**Usage:**
```rust
// Create a connection pool
let pool = Pool::new(10, || {
    DbConnection::connect("localhost:5432")
        .map_err(|e| PoolError::CreationFailed(e.to_string()))
});

// Get connection from pool
{
    let mut conn = pool.get()?;
    conn.execute("SELECT * FROM users")?;
    // Automatically returned to pool here
}

// Connection available for reuse
let conn2 = pool.get()?; // Reuses the returned connection
```

**Check/Test:**
- Test pool creates resources up to max_size
- Test resources returned to pool on drop
- Test pool reuses returned resources
- Test concurrent access from multiple threads
- Test pool exhaustion error
- Verify no resource leaks

**Why this isn't enough:**
Pool works for sync code, but modern Rust is increasingly async. We need async support with tokio/async-std. Also, no health checking—what if a pooled connection is stale or broken? No timeout for acquiring resources. These are critical for production systems.

---

### Step 6: Add Async Support and Health Checking

**Goal:** Support async/await and add resource health validation.

**What to improve:**

**1. Async pool:**
```rust
use tokio::sync::{Mutex as AsyncMutex, Semaphore};
use std::time::Duration;

pub struct AsyncPool<R> {
    available: Arc<AsyncMutex<VecDeque<R>>>,
    semaphore: Arc<Semaphore>,
    factory: Arc<dyn Fn() -> BoxFuture<'static, Result<R, PoolError>> + Send + Sync>,
    health_check: Arc<dyn Fn(&R) -> BoxFuture<'_, bool> + Send + Sync>,
}

impl<R: Send + 'static> AsyncPool<R> {
    pub fn new<F, H>(max_size: usize, factory: F, health_check: H) -> Self
    where
        F: Fn() -> BoxFuture<'static, Result<R, PoolError>> + Send + Sync + 'static,
        H: Fn(&R) -> BoxFuture<'_, bool> + Send + Sync + 'static,
    {
        AsyncPool {
            available: Arc::new(AsyncMutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(max_size)),
            factory: Arc::new(factory),
            health_check: Arc::new(health_check),
        }
    }

    pub async fn get(&self) -> Result<AsyncPooledResource<R>, PoolError> {
        // Acquire semaphore permit
        let permit = self.semaphore.acquire().await
            .map_err(|_| PoolError::PoolExhausted)?;

        let mut pool = self.available.lock().await;

        let resource = loop {
            if let Some(resource) = pool.pop_front() {
                // Health check
                if (self.health_check)(&resource).await {
                    break resource;
                }
                // Unhealthy, try next or create new
            } else {
                // Create new resource
                drop(pool); // Release lock while creating
                let resource = (self.factory)().await?;
                break resource;
            }
        };

        Ok(AsyncPooledResource {
            resource: Some(resource),
            pool: Arc::clone(&self.available),
            _permit: permit,
        })
    }

    pub async fn get_timeout(&self, timeout: Duration) -> Result<AsyncPooledResource<R>, PoolError> {
        tokio::time::timeout(timeout, self.get())
            .await
            .map_err(|_| PoolError::Timeout)?
    }
}

pub struct AsyncPooledResource<R> {
    resource: Option<R>,
    pool: Arc<AsyncMutex<VecDeque<R>>>,
    _permit: tokio::sync::SemaphorePermit<'static>,
}

impl<R> AsyncPooledResource<R> {
    pub fn get(&self) -> &R {
        self.resource.as_ref().unwrap()
    }

    pub fn get_mut(&mut self) -> &mut R {
        self.resource.as_mut().unwrap()
    }
}

impl<R> Drop for AsyncPooledResource<R> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let pool = Arc::clone(&self.pool);
            tokio::spawn(async move {
                let mut pool = pool.lock().await;
                pool.push_back(resource);
            });
        }
    }
}
```

**2. Health checking trait:**
```rust
#[async_trait]
pub trait HealthCheck {
    async fn is_healthy(&self) -> bool;
}

#[async_trait]
impl HealthCheck for DbConnection {
    async fn is_healthy(&self) -> bool {
        // Ping database
        self.execute("SELECT 1").await.is_ok()
    }
}
```

**3. Resource lifecycle management:**
```rust
pub struct ManagedPool<R> {
    pool: AsyncPool<R>,
    metrics: Arc<AsyncMutex<PoolMetrics>>,
}

pub struct PoolMetrics {
    total_created: u64,
    total_acquired: u64,
    total_released: u64,
    total_health_check_failures: u64,
    current_in_use: usize,
}

impl<R: Send + HealthCheck + 'static> ManagedPool<R> {
    pub async fn new<F>(max_size: usize, factory: F) -> Self
    where
        F: Fn() -> BoxFuture<'static, Result<R, PoolError>> + Send + Sync + 'static,
    {
        let metrics = Arc::new(AsyncMutex::new(PoolMetrics::default()));

        let pool = AsyncPool::new(
            max_size,
            {
                let metrics = Arc::clone(&metrics);
                move || {
                    let metrics = Arc::clone(&metrics);
                    Box::pin(async move {
                        let result = factory().await;
                        if result.is_ok() {
                            let mut m = metrics.lock().await;
                            m.total_created += 1;
                        }
                        result
                    })
                }
            },
            |resource: &R| Box::pin(async move {
                resource.is_healthy().await
            }),
        );

        ManagedPool { pool, metrics }
    }

    pub async fn get(&self) -> Result<AsyncPooledResource<R>, PoolError> {
        let resource = self.pool.get().await?;

        let mut metrics = self.metrics.lock().await;
        metrics.total_acquired += 1;
        metrics.current_in_use += 1;

        Ok(resource)
    }

    pub async fn metrics(&self) -> PoolMetrics {
        self.metrics.lock().await.clone()
    }

    pub async fn cleanup_stale(&self, max_idle_time: Duration) {
        // Periodically remove stale connections
    }
}
```

**Usage:**
```rust
// Async pool with health checking
let pool = ManagedPool::new(10, || {
    Box::pin(async {
        DbConnection::connect("localhost:5432").await
            .map_err(|e| PoolError::CreationFailed(e.to_string()))
    })
}).await;

// Get connection with timeout
let conn = pool.get_timeout(Duration::from_secs(5)).await?;
conn.execute("SELECT * FROM users").await?;
// Auto-returned on drop

// Get pool metrics
let metrics = pool.metrics().await;
println!("Total created: {}", metrics.total_created);
println!("Current in use: {}", metrics.current_in_use);
```

**Check/Test:**
- Test async pool with tokio runtime
- Test health checking rejects bad connections
- Test timeout on pool exhaustion
- Test metrics tracking
- Test concurrent async access
- Benchmark async vs sync pool performance

**What this achieves:**
A production-ready resource management system:
- **Type-Safe**: #[must_use] prevents resource leaks
- **Generic**: Works with any resource type
- **RAII**: Automatic cleanup on scope exit
- **Pooling**: Efficient resource reuse
- **Async**: Full async/await support
- **Health Checking**: Validates resource state
- **Metrics**: Observability into pool usage
- **Timeout**: Prevents indefinite blocking

**Extensions to explore:**
- Distributed resource management (etcd/consul)
- Resource priority levels
- Graceful degradation when pool exhausted
- Custom eviction policies (LRU, LFU)
- Resource warming (pre-populate pool)
- Circuit breaker pattern for failing resources

---

## Summary

These three projects teach essential API design patterns in Rust:

1. **Configuration System**: Builder pattern, type-state, validation, file loading, hot reload—all the patterns needed for production configuration management.

2. **SQL Query Builder**: Type-safe DSLs, fluent APIs, compile-time validation, backend abstraction, and prevention of security vulnerabilities through type system.

3. **Resource Manager**: #[must_use], RAII, scoped guards, resource pooling, async support—the patterns that prevent resource leaks and ensure correct resource lifecycle.

All three emphasize:
- **Compile-time safety**: Invalid states prevented by types
- **Ergonomic APIs**: Fluent, self-documenting interfaces
- **Zero-cost abstractions**: Type-level guarantees with no runtime overhead
- **Production-ready**: Real-world features (hot reload, pooling, metrics)

Students will understand how to design Rust APIs that are impossible to misuse, catching errors at compile time instead of runtime.
