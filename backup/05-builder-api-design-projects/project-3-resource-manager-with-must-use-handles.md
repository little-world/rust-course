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
