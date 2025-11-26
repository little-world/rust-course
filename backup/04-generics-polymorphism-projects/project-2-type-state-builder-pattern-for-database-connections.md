## Project 2: Type-State Builder Pattern for Database Connections

### Problem Statement

Design a database connection builder using phantom types to enforce a correct connection lifecycle at compile time. The system must ensure:
- Configuration methods can only be called in the appropriate state
- Connections cannot be opened without required configuration
- Opened connections cannot be reconfigured
- Transactions follow ACID properties through types
- Invalid state transitions are impossible (compiler errors, not runtime panics)

### Why It Matters

The type-state pattern leverages Rust's type system to make invalid states unrepresentable. This pattern is crucial for:
- **Safety-Critical Systems**: Medical devices, aerospace, automotive software where runtime failures are unacceptable
- **API Design**: Forcing users to use your API correctly at compile time
- **Protocol Implementation**: Network protocols, file format handlers where state must be tracked
- **Resource Management**: Ensuring resources are acquired, used, and released correctly

Type-state patterns appear throughout Rust's ecosystem: `std::net::TcpStream` states (connecting, connected, listening), file handles (read-only, write-only, read-write), and transaction systems.

### Use Cases

1. **Database Connection Pools**: Enforce authentication before query execution
2. **Network Protocol Handlers**: Ensure handshake completion before data transfer
3. **File Operations**: Distinguish read/write/append modes at type level
4. **State Machines**: Game states, UI workflows, business processes
5. **Builder APIs**: Ensure required fields are set before building
6. **Hardware Interfaces**: Ensure initialization before device access

### Solution Outline

**State Markers (Zero-Sized Types):**
```rust
pub struct Disconnected;
pub struct Configured;
pub struct Connected;
pub struct InTransaction;

pub struct ConnectionBuilder<State> {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    // Actual connection handle (only Some in Connected state)
    handle: Option<DbHandle>,
    _state: PhantomData<State>,
}
```

**State Transitions:**
- `new()` → `Disconnected`
- `host()`, `port()`, `database()` → `Configured`
- `connect()` → `Connected` (only from Configured)
- `begin_transaction()` → `InTransaction`
- `commit()`/`rollback()` → `Connected`

**Type Safety:**
```rust
impl ConnectionBuilder<Disconnected> {
    pub fn new() -> Self { /* ... */ }
    pub fn host(self, host: String) -> ConnectionBuilder<Configured> { /* ... */ }
}

impl ConnectionBuilder<Configured> {
    pub fn port(self, port: u16) -> Self { /* ... */ }
    pub fn connect(self) -> Result<ConnectionBuilder<Connected>, Error> { /* ... */ }
}

impl ConnectionBuilder<Connected> {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> { /* ... */ }
    pub fn begin_transaction(self) -> ConnectionBuilder<InTransaction> { /* ... */ }
}

impl ConnectionBuilder<InTransaction> {
    pub fn execute(&mut self, sql: &str) -> Result<(), Error> { /* ... */ }
    pub fn commit(self) -> Result<ConnectionBuilder<Connected>, Error> { /* ... */ }
    pub fn rollback(self) -> ConnectionBuilder<Connected> { /* ... */ }
}
```

### Testing Hints

**Compile-Time Tests:**
```rust
// Should compile
let conn = ConnectionBuilder::new()
    .host("localhost".into())
    .port(5432)
    .connect()?;

// Should NOT compile (test with compile_fail attribute)
#[test]
#[should_panic] // or use trybuild crate
fn cannot_connect_without_host() {
    let conn = ConnectionBuilder::new().connect(); // ERROR: no method
}
```

**Runtime Tests:**
```rust
#[test]
fn test_connection_lifecycle() {
    let conn = ConnectionBuilder::new()
        .host("localhost".into())
        .connect()
        .expect("connection failed");

    let tx = conn.begin_transaction();
    tx.execute("INSERT ...").unwrap();
    tx.commit().unwrap();
}

#[test]
fn test_transaction_rollback() {
    // Verify rollback works and returns to Connected state
}
```

**Use `trybuild` crate for compile-fail tests:**
```rust
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Builder with Optional Fields

**Goal:** Create a working connection builder using `Option<T>` for all fields.

**What to implement:**
```rust
pub struct ConnectionBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ConnectionBuilder {
    pub fn new() -> Self { /* all None */ }
    pub fn host(mut self, host: String) -> Self { /* self.host = Some(host); self */ }
    pub fn port(mut self, port: u16) -> Self { /* ... */ }
    // ... other setters

    pub fn connect(self) -> Result<Connection, Error> {
        // Runtime validation: host.ok_or(Error::MissingHost)?
        Ok(Connection { /* ... */ })
    }
}

pub struct Connection {
    // connection handle
}

impl Connection {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> { /* ... */ }
}
```

**Check/Test:**
- Build connection with all fields, verify it connects
- Try building without required fields, verify runtime error
- Test setter chaining works ergonomically

**Why this isn't enough:**
Errors happen at runtime. If a developer forgets to set the host, they only discover it when the code runs—potentially in production. The API allows nonsensical code like calling `connect()` twice or setting host after connection. We're relying on runtime validation instead of compile-time guarantees.

---

### Step 2: Introduce Phantom Types for Basic States

**Goal:** Use phantom types to distinguish Disconnected, Configured, and Connected states.

**What to improve:**
```rust
use std::marker::PhantomData;

pub struct Disconnected;
pub struct Configured;
pub struct Connected;

pub struct ConnectionBuilder<State> {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    _state: PhantomData<State>,
}

impl ConnectionBuilder<Disconnected> {
    pub fn new() -> Self {
        ConnectionBuilder {
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            _state: PhantomData,
        }
    }

    pub fn host(self, host: String) -> ConnectionBuilder<Configured> {
        ConnectionBuilder {
            host: Some(host),
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password,
            _state: PhantomData,
        }
    }
}

impl ConnectionBuilder<Configured> {
    pub fn port(mut self, port: u16) -> Self { /* ... */ }
    pub fn database(mut self, db: String) -> Self { /* ... */ }

    pub fn connect(self) -> Result<ConnectionBuilder<Connected>, Error> {
        // Actually establish connection
        Ok(ConnectionBuilder {
            host: self.host,
            // ... carry over all fields
            _state: PhantomData,
        })
    }
}

impl ConnectionBuilder<Connected> {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> { /* ... */ }
}
```

**Check/Test:**
- Verify cannot call `connect()` on `Disconnected` state (compile error)
- Verify cannot call `query()` before `connect()` (compile error)
- Test successful connection flow compiles and works

**Why this isn't enough:**
We still have `Option` fields and runtime validation. Setting host is required, but the type system doesn't enforce it—we moved the error from `connect()` to `query()`. Plus, there's lots of boilerplate copying fields between states. We need required vs optional field tracking.

---

### Step 3: Enforce Required vs Optional Fields with More Phantom Types

**Goal:** Use phantom types for each configurable field to track required at compile time.

**What to improve:**
```rust
// Field state markers
pub struct NotSet;
pub struct IsSet;

pub struct ConnectionBuilder<State, Host, Port> {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    _state: PhantomData<State>,
    _host: PhantomData<Host>,
    _port: PhantomData<Port>,
}

impl ConnectionBuilder<Disconnected, NotSet, NotSet> {
    pub fn new() -> Self { /* ... */ }
}

impl<State, Port> ConnectionBuilder<State, NotSet, Port> {
    pub fn host(self, host: String) -> ConnectionBuilder<State, IsSet, Port> {
        ConnectionBuilder {
            host: Some(host),
            port: self.port,
            database: self.database,
            _state: PhantomData,
            _host: PhantomData,
            _port: PhantomData,
        }
    }
}

impl<State, Host> ConnectionBuilder<State, Host, NotSet> {
    pub fn port(self, port: u16) -> ConnectionBuilder<State, Host, IsSet> { /* ... */ }
}

// connect() only available when Host and Port are IsSet
impl ConnectionBuilder<Configured, IsSet, IsSet> {
    pub fn connect(self) -> Result<ConnectionBuilder<Connected, IsSet, IsSet>, Error> {
        // Now we KNOW host and port are set
        let host = self.host.unwrap(); // Safe!
        let port = self.port.unwrap(); // Safe!
        // Actually connect...
    }
}
```

**Check/Test:**
- Verify `connect()` requires both host and port set (compile error without)
- Verify `connect()` works when both are provided
- Test that field setters can be called in any order

**Why this isn't enough:**
The boilerplate is getting out of hand—lots of field copying, many type parameters, complex impl blocks. Also, we don't have transaction support yet. Real databases need transaction state tracking (begin, commit, rollback). Let's add that and simplify with a macro.

---

### Step 4: Add Transaction States and Simplify with Macros

**Goal:** Support database transactions as additional states and reduce boilerplate.

**What to improve:**

**1. Add transaction states:**
```rust
pub struct InTransaction;

impl ConnectionBuilder<Connected, IsSet, IsSet> {
    pub fn begin_transaction(self) -> TransactionBuilder<InTransaction> {
        TransactionBuilder {
            connection: self,
            _state: PhantomData,
        }
    }
}

pub struct TransactionBuilder<State> {
    connection: ConnectionBuilder<Connected, IsSet, IsSet>,
    _state: PhantomData<State>,
}

impl TransactionBuilder<InTransaction> {
    pub fn execute(&mut self, sql: &str) -> Result<(), Error> {
        // Execute in transaction context
    }

    pub fn commit(self) -> Result<ConnectionBuilder<Connected, IsSet, IsSet>, Error> {
        // Commit transaction, return connection
        Ok(self.connection)
    }

    pub fn rollback(self) -> ConnectionBuilder<Connected, IsSet, IsSet> {
        // Rollback transaction, return connection
        self.connection
    }
}
```

**2. Reduce boilerplate with a macro:**
```rust
macro_rules! impl_setter {
    ($state:ty, $method:ident, $field:ident, $type:ty) => {
        impl<Host, Port> ConnectionBuilder<$state, Host, Port> {
            pub fn $method(mut self, $field: $type) -> Self {
                self.$field = Some($field);
                self
            }
        }
    };
}

impl_setter!(Configured, database, database, String);
impl_setter!(Configured, username, username, String);
// etc.
```

**Check/Test:**
- Test transaction lifecycle: begin, execute, commit
- Test rollback returns to Connected state
- Test cannot execute queries outside transaction when transaction is active
- Verify state transitions compile correctly

**Why this isn't enough:**
Transactions are all-or-nothing currently. What about nested transactions or savepoints? Also, we don't have connection pooling—creating a connection each time is expensive. Real applications maintain a pool of reused connections. Let's add connection state lifecycle management.

---

### Step 5: Add Connection Pooling with State Transitions

**Goal:** Implement a connection pool that manages lifecycle states automatically.

**What to improve:**
```rust
pub struct ConnectionPool {
    available: Vec<ConnectionBuilder<Connected, IsSet, IsSet>>,
    max_size: usize,
    config: PoolConfig,
}

pub struct PoolConfig {
    host: String,
    port: u16,
    database: String,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        ConnectionPool {
            available: Vec::new(),
            max_size: config.max_connections,
            config,
        }
    }

    pub fn get_connection(&mut self) -> Result<PooledConnection, Error> {
        let conn = if let Some(conn) = self.available.pop() {
            conn // Reuse existing
        } else {
            // Create new connection with proper states
            ConnectionBuilder::new()
                .host(self.config.host.clone())
                .port(self.config.port)
                .database(self.config.database.clone())
                .connect()?
        };

        Ok(PooledConnection {
            inner: Some(conn),
            pool: self,
        })
    }
}

// Smart pointer that returns connection to pool on drop
pub struct PooledConnection<'a> {
    inner: Option<ConnectionBuilder<Connected, IsSet, IsSet>>,
    pool: &'a mut ConnectionPool,
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.inner.take() {
            self.pool.available.push(conn);
        }
    }
}

impl<'a> Deref for PooledConnection<'a> {
    type Target = ConnectionBuilder<Connected, IsSet, IsSet>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'a> DerefMut for PooledConnection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}
```

**Check/Test:**
- Test pool creates connections up to max_size
- Test connections are returned to pool on drop
- Test concurrent access with multiple threads
- Verify no connection leaks (all connections returned)

**Why this isn't enough:**
The pool is not thread-safe. Multiple threads can't share it safely. We need `Arc<Mutex<>>` for thread safety, but that's runtime overhead. Can we use the type system to provide thread-safe access patterns? Also, no timeout handling—what if a connection hangs?

---

### Step 6: Thread-Safe Pool with Associated Types and Timeout States

**Goal:** Make the pool thread-safe, add timeout handling, and use associated types for extensibility.

**What to improve:**

**1. Thread-safe pool:**
```rust
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct ConnectionPool {
    inner: Arc<Mutex<PoolInner>>,
}

struct PoolInner {
    available: Vec<ConnectionBuilder<Connected, IsSet, IsSet>>,
    config: PoolConfig,
    in_use: usize,
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool { inner: Arc::clone(&self.inner) }
    }
}

impl ConnectionPool {
    pub fn get_connection(&self) -> Result<PooledConnection, Error> {
        self.get_connection_timeout(Duration::from_secs(30))
    }

    pub fn get_connection_timeout(&self, timeout: Duration) -> Result<PooledConnection, Error> {
        let start = Instant::now();

        loop {
            let mut pool = self.inner.lock().unwrap();

            if let Some(conn) = pool.available.pop() {
                pool.in_use += 1;
                return Ok(PooledConnection {
                    inner: Some(conn),
                    pool: Arc::clone(&self.inner),
                });
            } else if pool.in_use < pool.config.max_connections {
                // Create new connection
                pool.in_use += 1;
                drop(pool); // Release lock while connecting

                let conn = self.create_connection()?;

                return Ok(PooledConnection {
                    inner: Some(conn),
                    pool: Arc::clone(&self.inner),
                });
            }

            drop(pool); // Release lock while waiting

            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

pub struct PooledConnection {
    inner: Option<ConnectionBuilder<Connected, IsSet, IsSet>>,
    pool: Arc<Mutex<PoolInner>>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.inner.take() {
            let mut pool = self.pool.lock().unwrap();
            pool.available.push(conn);
            pool.in_use -= 1;
        }
    }
}
```

**2. Add trait for different database backends:**
```rust
pub trait DatabaseBackend {
    type Connection;
    type QueryResult;
    type Error;

    fn connect(config: &ConnectionConfig) -> Result<Self::Connection, Self::Error>;
    fn query(conn: &Self::Connection, sql: &str) -> Result<Self::QueryResult, Self::Error>;
}

pub struct PostgresBackend;
pub struct MySqlBackend;

// ConnectionBuilder becomes generic over backend
pub struct ConnectionBuilder<State, Backend: DatabaseBackend> {
    connection: Option<Backend::Connection>,
    _state: PhantomData<State>,
    _backend: PhantomData<Backend>,
}
```

**Check/Test:**
- Test concurrent access from multiple threads
- Test timeout behavior when pool is exhausted
- Test connection limit is respected
- Benchmark performance under load (1000+ concurrent requests)
- Test with different database backends (if multiple implemented)

**What this achieves:**
- **Thread Safety**: Pool can be safely shared across threads with `Arc`
- **Resource Limits**: Enforces maximum connection count
- **Timeout Handling**: Prevents indefinite blocking
- **Extensibility**: Associated types allow different database backends
- **Type Safety**: Still maintain all compile-time state guarantees
- **Performance**: Connections are reused, avoiding expensive connect/disconnect

**Extensions to explore:**
- Health checks: Periodically validate pooled connections
- Connection age: Expire old connections
- Async support: Use `tokio::sync::Mutex` for async/await
- Metrics: Track pool statistics (wait time, connection lifetime)
- Graceful shutdown: Drain pool cleanly

---
