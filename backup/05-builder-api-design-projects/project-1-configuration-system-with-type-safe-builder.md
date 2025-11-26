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
