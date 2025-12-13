# Object Pool with Smart Pointer Reuse

## Problem Statement

Build a high-performance object pool that reuses expensive-to-create objects instead of allocating and deallocating them repeatedly. The pool must automatically return objects when they're dropped, track usage statistics, and support both single-threaded and multi-threaded scenarios.

The pool must support:
- Automatic object return on drop (RAII pattern)
- Configurable pool size with overflow handling
- Creation of objects on-demand
- Thread-safe concurrent access
- Usage statistics (allocated, available, total created)
- Type-safe borrowing with custom smart pointers

## Why It Matters

**Real-World Performance Impact:**
- **Database connections**: Creating a TCP connection takes ~50ms, reusing takes <1μs (50,000x faster!)
- **Large buffers**: Allocating 1MB buffer takes ~100μs, reusing takes 0μs
- **Parser states**: Building a regex automaton takes ~1ms, reusing is instant
- **Game objects**: Creating enemies/bullets in games causes GC pauses

**Benchmark Example (HTTP connections):**
```
Without pool:  1,000 requests/sec (50ms per connection)
With pool:     100,000 requests/sec (10μs per request)
Speedup:       100x
```

## Use Cases

1. **Database Connection Pools**: PostgreSQL, MySQL, Redis connection pooling
2. **Thread Pools**: Worker threads that process tasks from a queue
3. **Buffer Pools**: Reusable byte buffers for network I/O
4. **Object Pools in Games**: Bullet pools, particle pools, enemy pools
5. **Parser Pools**: Reusable parsers for high-throughput servers
6. **HTTP Client Pools**: Keep-alive connection pooling

---

## Core Concepts: Object Pooling and Smart Pointer Patterns

Before diving into implementation, understanding these core concepts will help you appreciate why object pools are critical for performance and how smart pointers enable elegant pool implementations.

### 1. Object Pooling Fundamentals

**The Allocation Problem:**

Every time you create an object, the allocator must:
1. Find free memory block of correct size
2. Update memory bookkeeping structures
3. Return pointer to allocated memory

Deallocation reverses this:
1. Mark memory as free
2. Potentially merge adjacent free blocks (coalescing)
3. Update bookkeeping

**Typical Costs:**
```
Small allocation (< 1KB):    ~50-200ns
Large allocation (> 1MB):    ~100-500μs
Database connection:         ~50ms (50,000,000ns!)
Regex compilation:           ~1ms (1,000,000ns)
```

**The Pool Solution:**

Instead of allocating/deallocating repeatedly:
```rust
// Without pool - expensive!
for request in requests {
    let buffer = Vec::with_capacity(1024);  // Allocate
    process(&buffer);
    // Deallocate when buffer drops
}

// With pool - reuse!
for request in requests {
    let buffer = pool.get().unwrap();       // O(1) pop from vec
    process(&buffer);
    // Return to pool on drop
}
```

**Performance Impact:**

Real-world example (HTTP server with 1MB buffers):
```
Without pool:
- 1,000 requests/sec
- Each request: allocate 1MB (100μs) + process (50μs) + free (50μs) = 200μs
- Total: 200ms CPU time per 1000 requests

With pool (10 buffers):
- 100,000 requests/sec
- Each request: pop (1ns) + process (50μs) + push (1ns) ≈ 50μs
- Total: 50ms CPU time per 1000 requests

Speedup: 4x (and eliminates allocation jitter)
```

### 2. The RAII Pattern (Resource Acquisition Is Initialization)

**Core Principle:**

Resources should be tied to object lifetimes:
- Acquire resource in constructor
- Release resource in destructor
- Compiler ensures cleanup (even on panic!)

**Rust's Drop Trait:**

```rust
struct FileHandle {
    fd: i32,
}

impl Drop for FileHandle {
    fn drop(&mut self) {
        unsafe { close(self.fd); }  // Always called when value dropped
    }
}
```

**Why RAII Prevents Bugs:**

```rust
// Bad: Manual cleanup (error-prone)
fn process_file(path: &str) -> Result<String, Error> {
    let file = open_file(path)?;

    if file.size() == 0 {
        close_file(file);  // Must remember!
        return Err(EmptyFile);
    }

    let data = read_file(file)?;
    close_file(file);  // Must remember!
    Ok(data)
}

// Good: RAII (automatic cleanup)
fn process_file(path: &str) -> Result<String, Error> {
    let file = File::open(path)?;  // Implements Drop

    if file.metadata()?.len() == 0 {
        return Err(EmptyFile);  // file.drop() called automatically
    }

    let data = read_to_string(file)?;
    Ok(data)  // file.drop() called automatically
}
```

**RAII in Object Pools:**

```rust
pub struct PooledObject<T> {
    object: Option<T>,
    pool: &'a mut Pool<T>,
}

impl<T> Drop for PooledObject<'_, T> {
    fn drop(&mut self) {
        // Automatically return to pool!
        if let Some(obj) = self.object.take() {
            self.pool.return_object(obj);
        }
    }
}
```

Users can't forget to return objects - the compiler ensures it.

### 3. Custom Smart Pointers: Deref and DerefMut

**The Problem:**

```rust
let pooled = pool.get().unwrap();  // Returns PooledObject<Vec<u8>>

// Want to use like a Vec, but it's wrapped!
pooled.push(42);  // ❌ Error: PooledObject doesn't have push()
```

**The Solution: Deref Coercion**

Implement `Deref` to make your type act like the inner type:

```rust
use std::ops::{Deref, DerefMut};

impl<T> Deref for PooledObject<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T> DerefMut for PooledObject<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}
```

Now it works:
```rust
let mut pooled = pool.get().unwrap();  // PooledObject<Vec<u8>>
pooled.push(42);  // ✅ Deref coercion! Calls Vec::push()
assert_eq!(pooled.len(), 1);  // ✅ Calls Vec::len()
```

**How Deref Coercion Works:**

```rust
// When you write:
pooled.push(42)

// Rust tries:
1. PooledObject::push() - not found
2. Deref: (*pooled).push() -> Vec::push() - found! ✅

// Automatic dereferencing:
pooled.len()
(*pooled).len()  // Same thing
```

**Standard Library Examples:**

```rust
Box<T>        -> Deref<Target = T>
Rc<T>         -> Deref<Target = T>
Arc<T>        -> Deref<Target = T>
String        -> Deref<Target = str>
Vec<T>        -> Deref<Target = [T]>
MutexGuard<T> -> Deref<Target = T>
```

### 4. Lifetime Challenges in Self-Referential Structures

**The Pool Lifetime Problem:**

```rust
pub struct PooledObject<'a, T> {
    object: Option<T>,
    pool: &'a mut Pool<T>,  // Borrows pool
}

impl<T> Pool<T> {
    pub fn get(&mut self) -> Option<PooledObject<T>> {
        let obj = self.objects.pop()?;
        Some(PooledObject {
            object: Some(obj),
            pool: self,  // Borrow self mutably
        })
    }
}
```

**The Issue:**

```rust
let mut pool = Pool::new(|| vec![0u8; 1024]);
pool.preallocate(5);

let obj1 = pool.get().unwrap();  // Borrows pool mutably
let obj2 = pool.get().unwrap();  // ❌ Error: pool already borrowed!
```

The pool is borrowed for the lifetime of `obj1`, so we can't get `obj2`.

**Solution: Rc<RefCell<Pool>>**

Move from `&mut` lifetime to owned reference counting:

```rust
pub type PoolRef<T> = Rc<RefCell<Pool<T>>>;

pub struct PooledObject<T> {
    object: Option<T>,
    pool: PoolRef<T>,  // No lifetime! Owns Rc clone
}

// Now this works:
let pool = Pool::new(|| vec![0u8; 1024], 10);
let obj1 = pool.get().unwrap();  // Clones Rc
let obj2 = pool.get().unwrap();  // Clones Rc again - OK!
```

**Key Insight:**

```
&'a mut T       -> Single borrower, lifetime-bound
Rc<RefCell<T>>  -> Multiple owners, no lifetime constraint
```

### 5. The Rc<RefCell<>> Pattern for Shared Mutable State

**Why RefCell Is Needed:**

```rust
let pool = Rc::new(Pool::new(|| vec![0u8; 1024]));

// Rc gives us &Pool, not &mut Pool!
pool.get();  // ❌ Error: get() requires &mut self
```

`Rc::clone()` gives shared reference (`&T`), but we need `&mut T` to modify pool.

**RefCell: Runtime Borrow Checking**

```rust
let pool = Rc::new(RefCell::new(Pool::new(|| vec![0u8; 1024])));

// Get mutable access through RefCell:
let mut pool_mut = pool.borrow_mut();  // Runtime check
pool_mut.get();  // ✅ Works!
```

**The Rc<RefCell<T>> Pattern:**

```rust
use std::rc::Rc;
use std::cell::RefCell;

type PoolRef<T> = Rc<RefCell<Pool<T>>>;

fn get<T>(pool: &PoolRef<T>) -> Option<PooledObject<T>> {
    let obj = pool.borrow_mut().get_or_create();  // Borrow mutably
    Some(PooledObject {
        object: Some(obj),
        pool: pool.clone(),  // Clone Rc (cheap - just increment counter)
    })
}
```

**Borrow Rules (Runtime Enforced):**

```rust
let pool = Rc::new(RefCell::new(Pool::new(|| vec![])));

// Multiple immutable borrows OK:
let r1 = pool.borrow();
let r2 = pool.borrow();  // ✅

// One mutable borrow (exclusive):
let mut w = pool.borrow_mut();  // ✅

// Can't mix:
let r = pool.borrow();
let w = pool.borrow_mut();  // ❌ Panics! Already borrowed immutably
```

**When to Use:**

```
Rc<T>            -> Shared ownership, immutable
Rc<RefCell<T>>   -> Shared ownership, mutable (single-threaded)
Arc<Mutex<T>>    -> Shared ownership, mutable (multi-threaded)
```

### 6. Thread Safety: From Rc/RefCell to Arc/Mutex

**The Send and Sync Traits:**

```rust
// Send: Can transfer ownership across threads
// Sync: Can share references (&T) across threads

Rc<T>        -> NOT Send, NOT Sync
RefCell<T>   -> NOT Send, NOT Sync
Arc<T>       -> Send + Sync (if T: Send + Sync)
Mutex<T>     -> Send + Sync (if T: Send)
```

**Why Rc/RefCell Aren't Thread-Safe:**

```rust
// Rc uses non-atomic reference counting:
fn clone(&self) -> Self {
    self.count += 1;  // ❌ RACE CONDITION in multithreaded context!
    Rc { ptr: self.ptr }
}

// RefCell uses simple integer:
fn borrow_mut(&self) -> RefMut<T> {
    if self.borrow_count != 0 {  // ❌ RACE CONDITION!
        panic!("already borrowed");
    }
    self.borrow_count = -1;
    // ...
}
```

**Thread-Safe Alternative: Arc<Mutex<>>**

```rust
use std::sync::{Arc, Mutex};

type PoolRef<T> = Arc<Mutex<Pool<T>>>;

// Arc: Atomic reference counting (thread-safe Rc)
// Mutex: Blocking mutual exclusion (thread-safe RefCell)

fn get<T: Send>(pool: &PoolRef<T>) -> Option<PooledObject<T>> {
    let obj = pool.lock().unwrap().get_or_create();  // Blocks if already locked
    Some(PooledObject {
        object: Some(obj),
        pool: pool.clone(),  // Arc clone
    })
}
```

**Performance Comparison:**

```
Operation           Rc<RefCell<T>>    Arc<Mutex<T>>
Clone               ~1ns              ~5ns (atomic)
Borrow              ~1ns              ~20ns (lock acquisition)
Concurrent access   ❌ Panic          ✅ Blocks and waits
```

**When Thread-Safety Costs Are Worth It:**

```rust
// Single-threaded server (1 core):
// Rc<RefCell>: Handles 100k req/sec
// Arc<Mutex>:  Handles 95k req/sec (5% slower)

// Multi-threaded server (8 cores):
// Rc<RefCell>: Can't use ❌
// Arc<Mutex>:  Handles 600k req/sec (8x benefit from parallelism!)
```

### 7. The Builder Pattern

**The Problem: Many Optional Parameters**

```rust
// Bad: Constructor with many parameters
let pool = Pool::new(
    factory,
    Some(reset_fn),
    100,        // max_size
    true,       // preallocate?
    Some(50),   // preallocate_count
    true,       // stats?
);  // Hard to read, easy to mix up parameters
```

**The Builder Solution:**

```rust
let pool = Pool::builder()
    .factory(|| vec![0u8; 1024])
    .reset(|v| v.clear())
    .max_size(100)
    .build();
```

**Implementation Pattern:**

```rust
pub struct PoolBuilder<T> {
    factory: Option<Box<dyn Fn() -> T>>,
    reset: Option<Box<dyn Fn(&mut T)>>,
    max_size: usize,
}

impl<T> PoolBuilder<T> {
    pub fn factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        self.factory = Some(Box::new(factory));
        self  // Return self for chaining!
    }

    pub fn reset<F>(mut self, reset: F) -> Self
    where
        F: Fn(&mut T) + 'static,
    {
        self.reset = Some(Box::new(reset));
        self
    }

    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    pub fn build(self) -> Pool<T> {
        Pool {
            factory: self.factory.expect("factory is required"),
            reset: self.reset,
            max_size: self.max_size,
            objects: Vec::new(),
        }
    }
}
```

**Advantages:**

1. **Named parameters**: Clear what each value means
2. **Optional fields**: Only specify what you need
3. **Validation**: `build()` can validate configuration
4. **Type state**: Can enforce build order at compile time
5. **Fluent API**: Readable method chaining

**Real-World Examples:**

- `std::thread::Builder`
- `tokio::runtime::Builder`
- `reqwest::Client::builder()`
- Database connection builders (r2d2, deadpool)

### 8. Object Reset Strategies

**Why Reset Matters:**

Objects often accumulate state that must be cleared before reuse:

```rust
// Scenario: Buffer pool for HTTP requests

let pool = Pool::new(|| Vec::with_capacity(4096), 10);

// Request 1: Receives username/password
{
    let mut buf = pool.get().unwrap();
    buf.extend_from_slice(b"username=admin&password=secret123");
    send_to_server(&buf);
}  // Buffer returned to pool WITH SENSITIVE DATA!

// Request 2: Receives user profile
{
    let mut buf = pool.get().unwrap();
    // BUG: buf still contains "password=secret123"!
    // Could leak to logs, other users, etc.
}
```

**Reset Strategies:**

```rust
// 1. Clear (preserve capacity)
.reset(|vec: &mut Vec<u8>| vec.clear())

// 2. Truncate to specific size
.reset(|vec: &mut Vec<u8>| vec.truncate(0))

// 3. Rebuild (for complex state)
.reset(|conn: &mut Connection| {
    conn.rollback();
    conn.clear_cache();
    conn.reset_session();
})

// 4. Conditional reset
.reset(|buf: &mut Vec<u8>| {
    if buf.len() > 1024 {
        *buf = Vec::with_capacity(1024);  // Replace if too large
    } else {
        buf.clear();
    }
})
```

**Performance Considerations:**

```rust
// Clearing 1MB buffer: ~100μs
// Creating new 1MB buffer: ~500μs
// Speedup: 5x even with reset cost!

// But for small objects:
// Clearing 100-byte buffer: ~10ns
// Creating new 100-byte buffer: ~50ns
// Speedup: 5x
```

**Security Note:**

For sensitive data, use `zeroize` crate:
```rust
use zeroize::Zeroize;

.reset(|buf: &mut Vec<u8>| {
    buf.zeroize();  // Securely overwrites memory
    buf.clear();
})
```

### 9. Performance Monitoring and Metrics

**Key Metrics for Object Pools:**

```rust
pub struct PoolStats {
    pub created_count: usize,    // Total objects ever created
    pub gets: usize,              // Total get() calls
    pub hits: usize,              // get() found available object
    pub misses: usize,            // get() had to create new object
    pub peak_allocated: usize,    // Max simultaneous allocations
}

impl PoolStats {
    pub fn hit_rate(&self) -> f64 {
        self.hits as f64 / self.gets as f64
    }

    pub fn miss_rate(&self) -> f64 {
        self.misses as f64 / self.gets as f64
    }
}
```

**Interpreting Metrics:**

```rust
// Good pool sizing:
Hit rate: 95%+
Peak allocated: < max_size
Misses: Only during warmup or spikes

// Pool too small:
Hit rate: <80%
Peak allocated: >> max_size
Misses: Constant
→ Action: Increase max_size

// Pool too large:
Hit rate: 99%
Peak allocated: 10
Max size: 100
→ Action: Reduce max_size to save memory

// Reset too expensive:
Time per get: 100μs
Reset time: 90μs
→ Action: Optimize reset or create new objects instead
```

**Capacity Planning:**

```rust
fn recommend_pool_size(stats: &PoolStats) -> usize {
    // Set max_size to peak + 20% buffer
    (stats.peak_allocated as f64 * 1.2).ceil() as usize
}
```

### 10. Memory Management Strategies

**Pool Sizing Strategies:**

```rust
// 1. Fixed size (simple, predictable)
.max_size(10)  // Never grow beyond 10

// 2. Unbounded (dangerous!)
.max_size(usize::MAX)  // Can OOM

// 3. Adaptive (complex, optimal)
pool.adjust_size_based_on_metrics();

// 4. Per-core (for thread pools)
.max_size(num_cpus::get() * 2)
```

**Eviction Policies:**

When pool is full and object returned:

```rust
// 1. Drop overflow (default)
fn return_object(&mut self, obj: T) {
    if self.objects.len() < self.max_size {
        self.objects.push(obj);
    }  // Otherwise drop (destructor runs)
}

// 2. Replace oldest (LRU)
fn return_object(&mut self, obj: T) {
    if self.objects.len() >= self.max_size {
        self.objects.remove(0);  // Drop oldest
    }
    self.objects.push(obj);
}

// 3. Replace largest (minimize memory)
fn return_object(&mut self, obj: T) {
    if self.objects.len() >= self.max_size {
        let largest_idx = self.objects.iter()
            .enumerate()
            .max_by_key(|(_, o)| o.memory_size())
            .map(|(i, _)| i);
        if let Some(idx) = largest_idx {
            self.objects.remove(idx);
        }
    }
    self.objects.push(obj);
}
```

**Preallocation Trade-offs:**

```rust
// Without preallocation:
let pool = Pool::builder().factory(|| expensive_create()).build();
// First N gets() are slow (create objects)
// Lower memory usage

// With preallocation:
pool.lock().unwrap().preallocate(100);
// First gets() are fast (objects ready)
// Higher memory usage
// Better for latency-sensitive apps
```

**Object Lifecycle:**

```
┌─────────────────────────────────────────────┐
│ Object Lifecycle in Pool                    │
├─────────────────────────────────────────────┤
│                                             │
│  [Factory]                                  │
│     ↓ create()                              │
│  [New Object]                               │
│     ↓                                       │
│  ╔════════════════╗                        │
│  ║ Available Pool ║ ←──┐                   │
│  ╚════════════════╝    │                   │
│     ↓ get()            │ return/drop       │
│  [In Use]              │                   │
│     ↓                  │                   │
│  [Reset Hook]──────────┘                   │
│     │                                       │
│     ↓ if pool.len() >= max_size           │
│  [Drop/Deallocate]                         │
│                                             │
└─────────────────────────────────────────────┘
```

---

## Connection to This Project

This project progressively builds a production-quality object pool, with each milestone introducing essential patterns used in high-performance Rust applications.

### Milestone Progression and Learning Path

| Milestone | Pattern | Capabilities | Limitations | Real-World Equivalent |
|-----------|---------|--------------|-------------|----------------------|
| 1. Manual | `Vec<T>` | Basic pooling, manual return | Easy to forget return, not thread-safe | Prototype/testing |
| 2. RAII | Custom Drop | Automatic return, panic-safe | Fixed pool size, lifetime issues | r2d2 basic usage |
| 3. Rc Pool | `Rc<RefCell<>>` | Dynamic growth, multiple refs | Single-threaded only | tokio LocalPool |
| 4. Reset | Hook pattern | Clean reuse, security | Adds overhead | Production pools |
| 5. Thread-Safe | `Arc<Mutex<>>` | Concurrent access | Locking overhead | deadpool, r2d2 |
| 6. Monitoring | Metrics | Observability, tuning | Memory overhead | Production monitoring |

### Why Each Pattern Matters

**Milestone 1 (Manual Return): Understanding the Problem**

Establishes baseline:
- Simple Vec-based storage
- Pop/push mechanics
- Reveals why manual management fails

**Limitations that force evolution:**
- Users forget to call `return_object()` → memory leaks
- Early returns skip cleanup → pool depletion
- Panics prevent return → lost objects

**Milestone 2 (RAII): Automatic Resource Management**

Solves: Guaranteed cleanup
- Drop trait ensures objects always return
- Works even with panics (exception safety)
- Eliminates entire class of bugs

**The custom smart pointer pattern:**
```rust
PooledObject<T> { object: Option<T>, pool: &'a mut Pool<T> }
                    └─ Deref ─┘        └─ Drop returns ─┘
```

**Real-world analogs:**
- `MutexGuard` (auto-unlocks)
- `File` (auto-closes)
- `Box`, `Rc`, `Arc` (auto-deallocate)

**Milestone 3 (Rc<RefCell<>>): Dynamic Growth**

Solves: Lifetime constraints
- Can't get multiple objects with `&mut Pool`
- Need shared ownership without lifetimes
- Enable on-demand object creation

**The Rc<RefCell<>> pattern enables:**
```rust
let pool = Pool::new(|| buffer(), 10);
let obj1 = pool.get().unwrap();  // Rc clone
let obj2 = pool.get().unwrap();  // Rc clone - OK!
let obj3 = pool.get().unwrap();  // Creates new if pool empty
```

**Pattern used everywhere:**
- GUI event handlers (shared state)
- Game entity systems (shared components)
- Parser state (shared symbol tables)

**Milestone 4 (Reset Hooks): Clean Reuse**

Solves: State accumulation
- Buffers contain old data
- Connections have stale state
- Security leaks from previous use

**Real-world bugs prevented:**
```rust
// Without reset:
let buf = pool.get();
buf.extend(b"password=secret");
// Returns to pool with password!

let buf = pool.get();
// Next user sees password ❌

// With reset:
.reset(|buf| buf.clear())
// Password cleared before reuse ✅
```

**Performance impact:**
```
1KB buffer:
- Create new: 500ns
- Reset + reuse: 50ns
- Speedup: 10x

DB connection:
- Create new: 50ms
- Reset + reuse: 10μs
- Speedup: 5,000x
```

**Milestone 5 (Arc<Mutex<>>): Thread Safety**

Solves: Concurrent access
- Web server: N worker threads share pool
- Parallel processing: M cores need buffers
- Async runtime: Many tasks need connections

**Single vs multi-threaded:**
```rust
// Single-threaded
Rc<RefCell<Pool>>:
- get(): ~10ns
- Handles: 100M ops/sec on 1 core

// Multi-threaded
Arc<Mutex<Pool>>:
- get(): ~30ns (atomic + lock)
- Handles: 300M ops/sec on 8 cores
- 3x faster despite slower per-op cost!
```

**Critical for:**
- HTTP servers (Actix, Axum, Hyper)
- Database pools (r2d2, deadpool)
- Job queues (worker thread pools)

**Milestone 6 (Monitoring): Production Readiness**

Solves: Observability
- How often do we hit/miss?
- Is pool sized correctly?
- What's the peak usage?
- When should we scale?

**Metrics-driven optimization:**
```
Initial: max_size=10, hit_rate=60%, peak=25
→ Increase to max_size=30

After: max_size=30, hit_rate=95%, peak=18
→ Optimal! Hit rate high, not over-provisioned

Alert: hit_rate < 80% → trigger auto-scaling
```

### Performance Journey

Understanding performance at each stage:

| Pattern | Get Cost | Thread-Safe | Peak Throughput | Use Case |
|---------|----------|-------------|-----------------|----------|
| Direct Allocation | 500ns | N/A | 2M/sec | No reuse |
| Manual Pool | 50ns | No | 20M/sec | Prototype |
| RAII Pool | 50ns | No | 20M/sec | Single-thread app |
| Rc<RefCell> | 10ns | No | 100M/sec | Async single-threaded |
| Arc<Mutex> | 30ns | Yes | 300M/sec (8 cores) | **Production multi-threaded** |

**The 95% case:** `Arc<Mutex<Pool>>` with reset hooks is the production standard.

### Real-World Impact Examples

**Example 1: HTTP Server Connection Pool**

```rust
// Setup: 8-core server handling API requests

let pool = Pool::builder()
    .factory(|| {
        PgConnection::connect("postgres://localhost/db")
            .expect("connection failed")
    })
    .reset(|conn| {
        conn.rollback_transaction();  // Clean state
    })
    .max_size(20)  // 2.5 connections per core
    .build();

// Worker threads
for _ in 0..8 {
    let pool = pool.clone();
    thread::spawn(move || {
        loop {
            let request = receive_request();

            // Get connection from pool (30ns vs 50ms for new connection)
            let mut conn = pool.get().unwrap();

            // Process request
            let result = process_query(&mut *conn, &request);
            send_response(result);

            // conn automatically returned on drop
        }
    });
}
```

**Performance:**
- Without pool: 20 req/sec (50ms per connection)
- With pool: 10,000 req/sec (100μs per query)
- Speedup: 500x

**Example 2: Game Object Pool**

```rust
// Game engine: Pooling bullets to avoid GC pauses

struct Bullet {
    position: Vec3,
    velocity: Vec3,
    damage: u32,
    active: bool,
}

let bullet_pool = Pool::builder()
    .factory(|| Bullet {
        position: Vec3::ZERO,
        velocity: Vec3::ZERO,
        damage: 0,
        active: false,
    })
    .reset(|bullet| {
        bullet.active = false;
        bullet.position = Vec3::ZERO;
        bullet.velocity = Vec3::ZERO;
    })
    .max_size(1000)  // Max 1000 bullets on screen
    .build();

// Preallocate to avoid frame hitches
bullet_pool.lock().unwrap().preallocate(500);

// In game loop
fn player_shoot(pool: &PoolRef<Bullet>, direction: Vec3) {
    let mut bullet = pool.get().unwrap();
    bullet.position = player.position;
    bullet.velocity = direction * 100.0;
    bullet.damage = 25;
    bullet.active = true;

    active_bullets.push(bullet);  // Held for lifetime
}

// When bullet expires
fn despawn_bullet(bullet: PooledObject<Bullet>) {
    // Just drop - automatically returned and reset
}
```

**Impact:**
- Without pool: 16ms GC pause when 100 bullets created (frame drop!)
- With pool: 0ms pause, smooth 60 FPS

**Example 3: Async Task Buffer Pool**

```rust
use tokio;

// Async HTTP client with buffer pooling

let buffer_pool = Pool::builder()
    .factory(|| Vec::with_capacity(8192))
    .reset(|buf: &mut Vec<u8>| buf.clear())
    .max_size(100)
    .build();

#[tokio::main]
async fn main() {
    // Spawn 1000 concurrent tasks
    let mut tasks = vec![];

    for i in 0..1000 {
        let pool = buffer_pool.clone();

        let task = tokio::spawn(async move {
            let mut buf = pool.get().unwrap();

            // Download data into pooled buffer
            download_url(&format!("https://api.example.com/{}", i), &mut *buf).await;

            // Process buffer
            parse_json(&buf);

            // Buffer returned when task completes
        });

        tasks.push(task);
    }

    futures::future::join_all(tasks).await;

    println!("Stats: {}", buffer_pool.lock().unwrap().stats_report());
    // Hit rate: 99%, Peak: 87, Created: 100
    // Perfect! Pool sized correctly for load
}
```

### Architectural Insights

**Pattern 1: Smart Pointer Composition**

```rust
// Each layer adds capability:
T                          → Raw type
PooledObject<T>            → + Auto-return (Drop)
  └─ Deref -> T            → + Transparent usage
     pool: Rc<RefCell<P>>  → + Shared ownership, interior mutability

// Thread-safe variant:
PooledObject<T>
  └─ pool: Arc<Mutex<P>>   → + Thread-safety
```

**Pattern 2: Builder for Configuration**

```rust
// Builder separates construction from configuration:
Pool::builder()          → Create builder
  .factory(|| ...)       → Required: how to create
  .reset(|t| ...)        → Optional: how to clean
  .max_size(N)           → Optional: capacity limit
  .build()               → Construct pool

// Enables fluent, readable configuration
```

**Pattern 3: Statistics for Observability**

```rust
// Track all operations:
get() → stats.gets++
hit   → stats.hits++
miss  → stats.misses++, stats.created_count++

// Derive insights:
hit_rate = hits / gets           → Efficiency
peak_allocated                   → Capacity planning
current_allocated = created - available → Current load
```

### Skills Transferred to Other Domains

After completing this project, you'll understand patterns used in:

1. **Database Libraries** (diesel, sqlx, r2d2, deadpool)
   - Connection pooling
   - Automatic return on drop
   - Statistics and monitoring

2. **Async Runtimes** (tokio, async-std)
   - Thread pool management
   - Task queuing
   - Work-stealing pools

3. **Network Libraries** (hyper, reqwest)
   - Keep-alive connection pools
   - Buffer pools for zero-copy I/O
   - Client connection management

4. **Game Engines** (bevy, ggez)
   - Entity pooling
   - Component pools
   - Asset caching

5. **Memory Allocators** (jemalloc, mimalloc)
   - Free list management
   - Size-class pools
   - Thread-local caches

### Key Takeaways

1. **RAII prevents leaks**: Drop trait ensures cleanup even on panic

2. **Deref makes smart pointers transparent**: Users don't see the wrapper

3. **Rc<RefCell<>> solves lifetime issues**: Shared mutable state without `&mut`

4. **Arc<Mutex<>> enables parallelism**: Thread-safety costs ~3x but enables 8x speedup on 8 cores

5. **Builder pattern improves ergonomics**: Named parameters, optional configuration

6. **Reset hooks are critical**: Prevent security leaks and logic bugs

7. **Metrics enable optimization**: Can't improve what you don't measure

8. **Pools trade memory for speed**: Pre-allocate to eliminate allocation overhead

This project teaches you the patterns behind every high-performance pool in the Rust ecosystem - from database connections to thread pools to buffer pools.

---

## Milestone 1: Basic Pool with Vec and Manual Return

**Goal:** Create a simple object pool where objects must be manually returned.

### Introduction

We start with the simplest possible pool design:
- Store objects in a `Vec`
- `get()` pops an object from the vec
- `return_object()` pushes it back

**Limitations we'll address later:**
- Easy to forget to return objects (memory leak)
- No automatic cleanup
- Not thread-safe
- No creation of new objects when pool is empty
- No statistics tracking

### Architecture

```rust
pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
}
```

**Key Structures:**

- **`Pool<T>`**: Stores available objects and a factory function
  - `objects`: Vec of available objects
  - `factory`: Function to create new objects when pool is empty

**Key Functions:**

- `Pool::new(factory)`: Create pool with object factory
- `get(&mut self) -> Option<T>`: Take object from pool
- `return_object(&mut self, obj: T)`: Return object to pool
- `len(&self) -> usize`: Number of available objects

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pool() {
        let pool = Pool::new(|| vec![0u8; 1024]);
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_preallocate() {
        let mut pool = Pool::new(|| vec![0u8; 1024]);
        pool.preallocate(10);
        assert_eq!(pool.len(), 10);
    }

    #[test]
    fn test_get_and_return() {
        let mut pool = Pool::new(|| vec![0u8; 1024]);
        pool.preallocate(5);

        let obj = pool.get().unwrap();
        assert_eq!(obj.len(), 1024);
        assert_eq!(pool.len(), 4);

        pool.return_object(obj);
        assert_eq!(pool.len(), 5);
    }

    #[test]
    fn test_empty_pool() {
        let mut pool = Pool::new(|| String::from("test"));
        assert!(pool.get().is_none());
    }

    #[test]
    fn test_reuse() {
        let mut pool = Pool::new(|| Vec::with_capacity(1024));
        pool.preallocate(1);

        let mut obj1 = pool.get().unwrap();
        let ptr1 = obj1.as_ptr();

        obj1.push(42);
        pool.return_object(obj1);

        let obj2 = pool.get().unwrap();
        let ptr2 = obj2.as_ptr();

        // Same object reused
        assert_eq!(ptr1, ptr2);
    }

    #[test]
    fn test_multiple_gets() {
        let mut pool = Pool::new(|| vec![0u8; 100]);
        pool.preallocate(3);

        let obj1 = pool.get().unwrap();
        let obj2 = pool.get().unwrap();
        let obj3 = pool.get().unwrap();

        assert_eq!(pool.len(), 0);
        assert!(pool.get().is_none());

        pool.return_object(obj1);
        pool.return_object(obj2);
        pool.return_object(obj3);

        assert_eq!(pool.len(), 3);
    }
}
```

### Starter Code

```rust
pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
}

impl<T> Pool<T> {
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        todo!("
        Create pool with:
        - Empty objects vec
        - Box the factory function
        ")
    }

    pub fn preallocate(&mut self, count: usize) {
        todo!("
        Create 'count' objects using factory and add to vec:
        for _ in 0..count {
            let obj = (self.factory)();
            self.objects.push(obj);
        }
        ")
    }

    pub fn get(&mut self) -> Option<T> {
        todo!("
        Pop object from vec:
        self.objects.pop()

        Returns None if pool is empty
        ")
    }

    pub fn return_object(&mut self, obj: T) {
        todo!("
        Push object back to vec:
        self.objects.push(obj)
        ")
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}
```

---

## Milestone 2: Automatic Return with Custom Smart Pointer

**Goal:** Use RAII pattern so objects automatically return to pool when dropped.

### Introduction

**Why Milestone 1 Isn't Enough:**

Manual return is error-prone:
1. **Forget to return**: Object leaked, pool depleted
2. **Exception safety**: If code panics, object not returned
3. **Early returns**: Must remember to return on every path
4. **Verbose**: Requires explicit `return_object()` call

**Real-world bug example:**
```rust
fn process_data(pool: &mut Pool<Buffer>) {
    let mut buf = pool.get().unwrap();

    if buf.is_empty() {
        return; // BUG: Forgot to return buffer!
    }

    // ... process ...
    pool.return_object(buf); // Only returned on happy path
}
```

**Solution:** Create a custom smart pointer `PooledObject<T>` that returns the object on drop.

**Pattern:** This is the same pattern used by:
- `MutexGuard` (auto-unlocks on drop)
- `File` (auto-closes on drop)
- `TcpStream` (auto-closes on drop)

### Architecture

```rust
pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
}

pub struct PooledObject<'a, T> {
    object: Option<T>,
    pool: &'a mut Pool<T>,
}

impl<T> Drop for PooledObject<'_, T> {
    fn drop(&mut self) {
        // Automatically return to pool
    }
}
```

**Key Insight:** When `PooledObject` goes out of scope, its `Drop` implementation automatically returns the object to the pool.

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_return() {
        let mut pool = Pool::new(|| vec![0u8; 1024]);
        pool.preallocate(5);

        {
            let _obj = pool.get().unwrap();
            assert_eq!(pool.len(), 4);
        } // obj dropped here

        // Object automatically returned
        assert_eq!(pool.len(), 5);
    }

    #[test]
    fn test_early_return() {
        fn process(pool: &mut Pool<Vec<u8>>) -> bool {
            let _obj = pool.get().unwrap();

            if true {
                return false; // Early return
            }

            true
        }

        let mut pool = Pool::new(|| vec![0u8; 100]);
        pool.preallocate(1);

        process(&mut pool);

        // Object returned despite early return
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_panic_safety() {
        use std::panic::catch_unwind;
        use std::panic::AssertUnwindSafe;

        let mut pool = Pool::new(|| vec![0u8; 100]);
        pool.preallocate(1);

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _obj = pool.get().unwrap();
            panic!("Simulated panic");
        }));

        assert!(result.is_err());

        // Object still returned after panic
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_deref() {
        let mut pool = Pool::new(|| vec![1, 2, 3]);
        pool.preallocate(1);

        let obj = pool.get().unwrap();

        // Can use like normal Vec
        assert_eq!(obj.len(), 3);
        assert_eq!(obj[0], 1);
    }

    #[test]
    fn test_deref_mut() {
        let mut pool = Pool::new(|| vec![0u8; 10]);
        pool.preallocate(1);

        let mut obj = pool.get().unwrap();

        // Can mutate through smart pointer
        obj.push(42);
        obj[0] = 99;

        assert_eq!(obj[0], 99);
    }

    #[test]
    fn test_multiple_scopes() {
        let mut pool = Pool::new(|| String::from("test"));
        pool.preallocate(2);

        {
            let _obj1 = pool.get();
            assert_eq!(pool.len(), 1);

            {
                let _obj2 = pool.get();
                assert_eq!(pool.len(), 0);
            }

            assert_eq!(pool.len(), 1);
        }

        assert_eq!(pool.len(), 2);
    }
}
```

### Starter Code

```rust
use std::ops::{Deref, DerefMut};

pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
}

impl<T> Pool<T> {
    pub fn new<F>(factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Pool {
            objects: Vec::new(),
            factory: Box::new(factory),
        }
    }

    pub fn get(&mut self) -> Option<PooledObject<T>> {
        todo!("
        1. Pop object from self.objects
        2. Wrap in PooledObject:
           Some(PooledObject {
               object: Some(obj),
               pool: self,
           })
        ")
    }

    fn return_object(&mut self, obj: T) {
        self.objects.push(obj);
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn preallocate(&mut self, count: usize) {
        for _ in 0..count {
            self.objects.push((self.factory)());
        }
    }
}

pub struct PooledObject<'a, T> {
    object: Option<T>,
    pool: &'a mut Pool<T>,
}

impl<T> Drop for PooledObject<'_, T> {
    fn drop(&mut self) {
        todo!("
        Return object to pool:
        1. Take object from self.object (Option::take())
        2. If Some(obj), call self.pool.return_object(obj)
        ")
    }
}

impl<T> Deref for PooledObject<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        todo!("
        Return reference to inner object:
        self.object.as_ref().unwrap()

        Safe to unwrap because object is always Some until drop
        ")
    }
}

impl<T> DerefMut for PooledObject<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        todo!("
        Return mutable reference to inner object:
        self.object.as_mut().unwrap()
        ")
    }
}
```

---

## Milestone 3: Grow-on-Demand with Rc Pool Reference

**Goal:** Automatically create new objects when pool is empty, using `Rc` to share pool reference.

### Introduction

**Why Milestone 2 Isn't Enough:**

Current limitations:
1. **Fixed size**: Must preallocate; returns `None` if empty
2. **Inflexible**: Can't handle traffic spikes
3. **Lifetime issues**: `PooledObject<'a>` ties object lifetime to pool borrow

**Real-world scenario:** HTTP server with connection pool:
- Normal load: 10 concurrent connections (pool size 10)
- Traffic spike: 100 concurrent requests
- Current behavior: 90 requests fail!
- Desired behavior: Create new connections temporarily

**Problem with current design:**
```rust
fn handle_request(pool: &mut Pool<Connection>) {
    let conn = pool.get().unwrap(); // Borrows pool mutably

    // PROBLEM: Can't get another connection while conn is alive!
    // let conn2 = pool.get(); // ERROR: pool already borrowed
}
```

**Solution:** Use `Rc<RefCell<Pool>>` so multiple `PooledObject`s can coexist.

### Architecture

```rust
use std::rc::Rc;
use std::cell::RefCell;

pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
    created_count: usize,
}

pub type PoolRef<T> = Rc<RefCell<Pool<T>>>;

pub struct PooledObject<T> {
    object: Option<T>,
    pool: PoolRef<T>,
}
```

**Key Changes:**
- Pool wrapped in `Rc<RefCell<>>` for shared mutable access
- `PooledObject` holds `Rc` clone instead of `&mut` reference
- Can create objects on-demand when pool is empty
- Track statistics (total created, max size)

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grow_on_demand() {
        let pool = Pool::new(|| vec![0u8; 1024], 10);

        // Pool starts empty
        assert_eq!(pool.borrow().len(), 0);

        let obj = pool.get();

        // Object created on-demand
        assert!(obj.is_some());
        assert_eq!(pool.borrow().created_count(), 1);
    }

    #[test]
    fn test_multiple_simultaneous_objects() {
        let pool = Pool::new(|| vec![0u8; 100], 5);

        let obj1 = pool.get().unwrap();
        let obj2 = pool.get().unwrap();
        let obj3 = pool.get().unwrap();

        // All three can exist simultaneously
        assert_eq!(obj1.len(), 100);
        assert_eq!(obj2.len(), 100);
        assert_eq!(obj3.len(), 100);

        assert_eq!(pool.borrow().created_count(), 3);
    }

    #[test]
    fn test_reuse_after_return() {
        let pool = Pool::new(|| vec![0u8; 100], 10);

        {
            let _obj1 = pool.get();
            assert_eq!(pool.borrow().created_count(), 1);
        }

        {
            let _obj2 = pool.get();
            // Reused, didn't create new
            assert_eq!(pool.borrow().created_count(), 1);
        }
    }

    #[test]
    fn test_max_size_limit() {
        let pool = Pool::new(|| vec![0u8; 100], 2);

        let obj1 = pool.get().unwrap();
        let obj2 = pool.get().unwrap();
        let obj3 = pool.get().unwrap(); // Creates even beyond max_size

        drop(obj1);
        drop(obj2);
        drop(obj3);

        // Pool keeps only max_size objects
        assert_eq!(pool.borrow().len(), 2);
    }

    #[test]
    fn test_statistics() {
        let pool = Pool::new(|| String::from("test"), 5);

        let o1 = pool.get();
        let o2 = pool.get();

        assert_eq!(pool.borrow().available(), 0);
        assert_eq!(pool.borrow().allocated(), 2);
        assert_eq!(pool.borrow().created_count(), 2);

        drop(o1);

        assert_eq!(pool.borrow().available(), 1);
        assert_eq!(pool.borrow().allocated(), 1);
    }

    #[test]
    fn test_reset_object() {
        let pool = Pool::new(
            || vec![0u8; 10],
            5,
        );

        {
            let mut obj = pool.get().unwrap();
            obj.push(1);
            obj.push(2);
            obj.push(3);
        } // Object returned

        // Get same object back
        let obj = pool.get().unwrap();

        // Should be reset (if we implement clear in factory)
        // For now, it still has the data
        assert_eq!(obj.len(), 13); // 10 + 3 pushed
    }
}
```

### Starter Code

```rust
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
    created_count: usize,
}

pub type PoolRef<T> = Rc<RefCell<Pool<T>>>;

impl<T> Pool<T> {
    pub fn new<F>(factory: F, max_size: usize) -> PoolRef<T>
    where
        F: Fn() -> T + 'static,
    {
        todo!("
        Wrap Pool in Rc<RefCell<>>:
        Rc::new(RefCell::new(Pool {
            objects: Vec::new(),
            factory: Box::new(factory),
            max_size,
            created_count: 0,
        }))
        ")
    }

    pub fn get_or_create(&mut self) -> T {
        todo!("
        Try to pop from objects vec.
        If None, create new object:
        1. Call self.factory()
        2. Increment self.created_count
        3. Return object
        ")
    }

    fn return_object(&mut self, obj: T) {
        todo!("
        Push object back if under max_size:
        if self.objects.len() < self.max_size {
            self.objects.push(obj);
        }
        // Otherwise, drop it (destructor runs)
        ")
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn created_count(&self) -> usize {
        self.created_count
    }

    pub fn available(&self) -> usize {
        self.objects.len()
    }

    pub fn allocated(&self) -> usize {
        self.created_count - self.objects.len()
    }
}

// Helper function for getting from pool
pub fn get<T>(pool: &PoolRef<T>) -> Option<PooledObject<T>> {
    todo!("
    1. Borrow pool mutably: pool.borrow_mut()
    2. Get or create object
    3. Wrap in PooledObject with Rc clone
    ")
}

pub struct PooledObject<T> {
    object: Option<T>,
    pool: PoolRef<T>,
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        todo!("
        Return object to pool:
        1. Take object from self.object
        2. Borrow pool mutably
        3. Call pool.return_object(obj)
        ")
    }
}

impl<T> Deref for PooledObject<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T> DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}
```

---

## Milestone 4: Object Reset Hook for Clean Reuse

**Goal:** Add hooks to reset object state before reuse.

### Introduction

**Why Milestone 3 Isn't Enough:**

Objects often accumulate state that must be cleared:
1. **Buffers**: Need to be cleared before reuse
2. **Connections**: Must reset to clean state
3. **Parsers**: Must clear internal state
4. **Caches**: Should be invalidated

**Real-world bug:**
```rust
let pool = Pool::new(|| Vec::new(), 10);

{
    let mut buf = pool.get().unwrap();
    buf.extend_from_slice(b"secret password");
} // Returned to pool with data!

{
    let buf = pool.get().unwrap();
    // BUG: buf still contains "secret password"!
}
```

**Solution:** Add reset hook that runs before returning to pool.

**Performance consideration:**
- Clearing a 1MB buffer: ~100μs
- Creating new 1MB buffer: ~500μs
- **Speedup: 5x** even with reset cost

### Architecture

```rust
pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
    reset: Option<Box<dyn Fn(&mut T)>>,
    max_size: usize,
    created_count: usize,
}
```

**Reset Strategies:**
1. **Clear**: `vec.clear()` for buffers
2. **Rebuild**: `*obj = factory()` for complex objects
3. **Custom**: User-defined cleanup

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_reset() {
        let pool = Pool::builder()
            .factory(|| Vec::with_capacity(1024))
            .reset(|vec: &mut Vec<u8>| vec.clear())
            .max_size(5)
            .build();

        {
            let mut buf = pool.get().unwrap();
            buf.extend_from_slice(b"test data");
            assert_eq!(buf.len(), 9);
        }

        {
            let buf = pool.get().unwrap();
            // Reset to empty
            assert_eq!(buf.len(), 0);
            // But capacity preserved
            assert_eq!(buf.capacity(), 1024);
        }
    }

    #[test]
    fn test_connection_reset() {
        #[derive(Debug)]
        struct Connection {
            id: usize,
            is_authenticated: bool,
            transaction_active: bool,
        }

        let pool = Pool::builder()
            .factory(|| Connection {
                id: 0,
                is_authenticated: false,
                transaction_active: false,
            })
            .reset(|conn| {
                conn.is_authenticated = false;
                conn.transaction_active = false;
            })
            .max_size(3)
            .build();

        {
            let mut conn = pool.get().unwrap();
            conn.is_authenticated = true;
            conn.transaction_active = true;
        }

        {
            let conn = pool.get().unwrap();
            assert!(!conn.is_authenticated);
            assert!(!conn.transaction_active);
        }
    }

    #[test]
    fn test_no_reset() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 10])
            .max_size(5)
            .build(); // No reset hook

        {
            let mut buf = pool.get().unwrap();
            buf[0] = 42;
        }

        {
            let buf = pool.get().unwrap();
            // State preserved (no reset)
            assert_eq!(buf[0], 42);
        }
    }

    #[test]
    fn test_rebuild_reset() {
        let pool = Pool::builder()
            .factory(|| vec![1, 2, 3])
            .reset(|vec| {
                vec.clear();
                vec.extend_from_slice(&[1, 2, 3]);
            })
            .max_size(2)
            .build();

        {
            let mut v = pool.get().unwrap();
            v.clear();
            v.push(99);
        }

        {
            let v = pool.get().unwrap();
            assert_eq!(&*v, &[1, 2, 3]); // Reset to initial state
        }
    }

    #[test]
    fn test_complex_reset() {
        use std::collections::HashMap;

        let pool = Pool::builder()
            .factory(|| HashMap::with_capacity(100))
            .reset(|map: &mut HashMap<String, i32>| {
                map.clear();
                // Capacity preserved
            })
            .max_size(3)
            .build();

        {
            let mut map = pool.get().unwrap();
            map.insert("key".to_string(), 42);
            assert_eq!(map.len(), 1);
        }

        {
            let map = pool.get().unwrap();
            assert_eq!(map.len(), 0);
            assert!(map.capacity() >= 100);
        }
    }

    #[test]
    fn test_reset_called_on_return() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc as StdArc;

        let reset_count = StdArc::new(AtomicUsize::new(0));
        let rc = reset_count.clone();

        let pool = Pool::builder()
            .factory(|| vec![0u8; 10])
            .reset(move |vec| {
                vec.clear();
                rc.fetch_add(1, Ordering::SeqCst);
            })
            .max_size(2)
            .build();

        {
            let _obj = pool.get();
        } // Reset called here

        assert_eq!(reset_count.load(Ordering::SeqCst), 1);

        {
            let _obj = pool.get();
        }

        assert_eq!(reset_count.load(Ordering::SeqCst), 2);
    }
}
```

### Starter Code

```rust
use std::rc::Rc;
use std::cell::RefCell;

pub struct Pool<T> {
    objects: Vec<T>,
    factory: Box<dyn Fn() -> T>,
    reset: Option<Box<dyn Fn(&mut T)>>,
    max_size: usize,
    created_count: usize,
}

pub struct PoolBuilder<T> {
    factory: Option<Box<dyn Fn() -> T>>,
    reset: Option<Box<dyn Fn(&mut T)>>,
    max_size: usize,
}

impl<T> Pool<T> {
    pub fn builder() -> PoolBuilder<T> {
        todo!("Return PoolBuilder with defaults")
    }
}

impl<T> PoolBuilder<T> {
    pub fn factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        todo!("Set factory and return self")
    }

    pub fn reset<F>(mut self, reset: F) -> Self
    where
        F: Fn(&mut T) + 'static,
    {
        todo!("Set reset hook and return self")
    }

    pub fn max_size(mut self, size: usize) -> Self {
        todo!("Set max_size and return self")
    }

    pub fn build(self) -> PoolRef<T> {
        todo!("
        Create Pool from builder:
        1. Unwrap factory (or panic if not set)
        2. Wrap in Rc<RefCell<>>
        ")
    }
}

impl<T> Pool<T> {
    fn return_object(&mut self, mut obj: T) {
        todo!("
        1. If reset hook exists, call it:
           if let Some(ref reset) = self.reset {
               reset(&mut obj);
           }

        2. Push to objects if under max_size
        ")
    }

    // ... rest of implementation from Milestone 3 ...
}
```

---

## Milestone 5: Thread-Safe Pool with Arc and Mutex

**Goal:** Make the pool thread-safe for concurrent access from multiple threads.

### Introduction

**Why Milestone 4 Isn't Enough:**

`Rc<RefCell<Pool>>` is not thread-safe:
1. **Not Send**: Can't transfer across threads
2. **Not Sync**: Can't share references across threads
3. **RefCell panics**: No blocking on contention

**Real-world scenario:** Web server with connection pool:
- 10 worker threads handling requests
- All sharing same database connection pool
- Need thread-safe concurrent access

**Solution:** Replace `Rc` → `Arc`, `RefCell` → `Mutex`.

**Performance Impact:**
- `Mutex::lock()`: ~20ns overhead per access
- Worth it for thread-safety
- Alternative: Lock-free pool (advanced, see Milestone 6 hint)

### Architecture

```rust
use std::sync::{Arc, Mutex};

pub struct Pool<T> {
    objects: Vec<T>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
    created_count: usize,
}

pub type PoolRef<T> = Arc<Mutex<Pool<T>>>;
```

**Key Changes:**
- `Rc` → `Arc`: Atomic reference counting
- `RefCell` → `Mutex`: Blocking mutual exclusion
- `Box<dyn Fn>` → `Arc<dyn Fn + Send + Sync>`: Thread-safe closures
- `T: Send`: Objects can be transferred between threads

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    #[test]
    fn test_concurrent_get() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .max_size(20)
            .build();

        let mut handles = vec![];

        for _ in 0..10 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                let obj = pool_clone.get().unwrap();
                assert_eq!(obj.len(), 1024);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_get_and_return() {
        let pool = Pool::builder()
            .factory(|| Vec::with_capacity(100))
            .reset(|v: &mut Vec<u8>| v.clear())
            .max_size(5)
            .build();

        let counter = StdArc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for _ in 0..100 {
            let pool_clone = pool.clone();
            let c = counter.clone();

            let handle = thread::spawn(move || {
                let mut obj = pool_clone.get().unwrap();
                obj.push(42);
                c.fetch_add(1, Ordering::SeqCst);
                // obj automatically returned on drop
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 100);

        // Pool should have max_size objects
        let final_count = pool.lock().unwrap().len();
        assert_eq!(final_count, 5);
    }

    #[test]
    fn test_statistics_thread_safe() {
        let pool = Pool::builder()
            .factory(|| String::from("test"))
            .max_size(10)
            .build();

        let mut handles = vec![];

        for _ in 0..5 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                let _obj = pool_clone.get();
                thread::sleep(std::time::Duration::from_millis(10));
            });
            handles.push(handle);
        }

        // While threads hold objects
        thread::sleep(std::time::Duration::from_millis(5));

        let stats = pool.lock().unwrap();
        let allocated = stats.allocated();
        let available = stats.available();

        assert!(allocated <= 5);
        assert_eq!(allocated + available, stats.created_count());

        drop(stats);

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<PoolRef<Vec<u8>>>();
        assert_sync::<PoolRef<Vec<u8>>>();
    }

    #[test]
    fn test_high_contention() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .max_size(2) // Only 2 objects to force contention
            .build();

        let mut handles = vec![];

        for i in 0..20 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let _obj = pool_clone.get().unwrap();
                    // Simulate work
                    thread::sleep(std::time::Duration::from_micros(100));
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should have created objects on-demand
        let created = pool.lock().unwrap().created_count();
        assert!(created >= 2);
    }

    #[test]
    fn test_concurrent_reset() {
        use std::collections::HashSet;
        use std::sync::Mutex as StdMutex;

        let reset_values = StdArc::new(StdMutex::new(HashSet::new()));
        let rv = reset_values.clone();

        let pool = Pool::builder()
            .factory(|| vec![0u8; 10])
            .reset(move |vec| {
                vec.clear();
                rv.lock().unwrap().insert(vec.as_ptr() as usize);
            })
            .max_size(5)
            .build();

        let mut handles = vec![];

        for _ in 0..50 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                let mut obj = pool_clone.get().unwrap();
                obj.push(42);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Reset should have been called many times
        let count = reset_values.lock().unwrap().len();
        assert!(count >= 5);
    }
}
```

### Starter Code

```rust
use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

pub struct Pool<T: Send> {
    objects: Vec<T>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
    created_count: usize,
}

pub type PoolRef<T> = Arc<Mutex<Pool<T>>>;

pub struct PoolBuilder<T: Send> {
    factory: Option<Arc<dyn Fn() -> T + Send + Sync>>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
}

impl<T: Send> Pool<T> {
    pub fn builder() -> PoolBuilder<T> {
        PoolBuilder {
            factory: None,
            reset: None,
            max_size: 100,
        }
    }
}

impl<T: Send> PoolBuilder<T> {
    pub fn factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        todo!("Wrap factory in Arc and store")
    }

    pub fn reset<F>(mut self, reset: F) -> Self
    where
        F: Fn(&mut T) + Send + Sync + 'static,
    {
        todo!("Wrap reset in Arc and store")
    }

    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    pub fn build(self) -> PoolRef<T> {
        todo!("
        Create Arc<Mutex<Pool>>:
        Arc::new(Mutex::new(Pool {
            objects: Vec::new(),
            factory: self.factory.expect('factory required'),
            reset: self.reset,
            max_size: self.max_size,
            created_count: 0,
        }))
        ")
    }
}

// Helper trait for PoolRef
pub trait PoolExt<T: Send> {
    fn get(&self) -> Option<PooledObject<T>>;
}

impl<T: Send> PoolExt<T> for PoolRef<T> {
    fn get(&self) -> Option<PooledObject<T>> {
        todo!("
        1. Lock pool: self.lock().unwrap()
        2. Get or create object
        3. Wrap in PooledObject with Arc clone
        ")
    }
}

pub struct PooledObject<T: Send> {
    object: Option<T>,
    pool: PoolRef<T>,
}

impl<T: Send> Drop for PooledObject<T> {
    fn drop(&mut self) {
        todo!("
        1. Take object from self.object
        2. Lock pool
        3. Return object (with reset if configured)
        ")
    }
}

impl<T: Send> Deref for PooledObject<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T: Send> DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}
```

---

## Milestone 6: Performance Monitoring and Optimization

**Goal:** Add detailed metrics and identify performance bottlenecks.

### Introduction

**Why Milestone 5 Isn't Enough:**

Production pools need observability:
1. **Performance metrics**: Hit rate, miss rate, allocation rate
2. **Resource tracking**: Peak usage, average usage
3. **Bottleneck identification**: Contention, slow resets
4. **Capacity planning**: When to increase pool size

**Real-world monitoring:**
```
Pool Statistics:
- Gets: 1,000,000
- Hits: 950,000 (95% hit rate)
- Misses: 50,000 (5% miss rate)
- Peak allocated: 45
- Avg allocated: 23
- Recommendation: Increase pool size to 50
```

**Performance Optimizations:**
1. **Preallocate**: Warm up pool on startup
2. **Tune max_size**: Based on metrics
3. **Optimize reset**: Profile reset function
4. **Consider lock-free**: For extremely high throughput

### Architecture

```rust
pub struct Pool<T: Send> {
    objects: Vec<T>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
    stats: PoolStats,
}

pub struct PoolStats {
    created_count: usize,
    gets: usize,
    hits: usize,
    misses: usize,
    peak_allocated: usize,
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_hit_rate() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .max_size(5)
            .build();

        // Preallocate
        pool.lock().unwrap().preallocate(5);

        // All hits (pool has objects)
        for _ in 0..10 {
            let _obj = pool.get();
        }

        let stats = pool.lock().unwrap().stats();
        assert_eq!(stats.hits, 10);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate(), 1.0);
    }

    #[test]
    fn test_miss_rate() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .max_size(10)
            .build();

        // All misses (pool starts empty)
        for _ in 0..5 {
            let _obj = pool.get();
        }

        let stats = pool.lock().unwrap().stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 5);
        assert_eq!(stats.miss_rate(), 1.0);
    }

    #[test]
    fn test_peak_allocated() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 100])
            .max_size(10)
            .build();

        let objs: Vec<_> = (0..7).map(|_| pool.get().unwrap()).collect();

        let stats = pool.lock().unwrap().stats();
        assert_eq!(stats.peak_allocated, 7);

        drop(objs);

        let stats = pool.lock().unwrap().stats();
        // Peak stays at 7 even after returning
        assert_eq!(stats.peak_allocated, 7);
    }

    #[test]
    fn test_comprehensive_stats() {
        let pool = Pool::builder()
            .factory(|| String::from("test"))
            .max_size(5)
            .build();

        pool.lock().unwrap().preallocate(3);

        // 3 hits, 2 misses
        let _o1 = pool.get(); // hit
        let _o2 = pool.get(); // hit
        let _o3 = pool.get(); // hit
        let _o4 = pool.get(); // miss (created new)
        let _o5 = pool.get(); // miss (created new)

        let stats = pool.lock().unwrap().stats();
        assert_eq!(stats.gets, 5);
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 2);
        assert_eq!(stats.created_count, 5);
        assert_eq!(stats.peak_allocated, 5);
    }

    #[test]
    fn test_stats_report() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .max_size(10)
            .build();

        pool.lock().unwrap().preallocate(5);

        for _ in 0..100 {
            let _obj = pool.get();
        }

        let report = pool.lock().unwrap().stats_report();

        assert!(report.contains("Total gets:"));
        assert!(report.contains("Hit rate:"));
        assert!(report.contains("Peak allocated:"));
    }

    #[test]
    fn test_concurrent_stats() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 100])
            .max_size(20)
            .build();

        let mut handles = vec![];

        for _ in 0..10 {
            let pool_clone = pool.clone();
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let _obj = pool_clone.get();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = pool.lock().unwrap().stats();
        assert_eq!(stats.gets, 100);
        assert!(stats.peak_allocated <= 20);
    }
}
```

### Starter Code

```rust
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Default)]
pub struct PoolStats {
    pub created_count: usize,
    pub gets: usize,
    pub hits: usize,
    pub misses: usize,
    pub peak_allocated: usize,
}

impl PoolStats {
    pub fn hit_rate(&self) -> f64 {
        todo!("Calculate hits / gets (handle division by zero)")
    }

    pub fn miss_rate(&self) -> f64 {
        todo!("Calculate misses / gets")
    }

    pub fn current_allocated(&self, available: usize) -> usize {
        self.created_count - available
    }
}

pub struct Pool<T: Send> {
    objects: Vec<T>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
    stats: PoolStats,
}

impl<T: Send> Pool<T> {
    fn get_or_create(&mut self) -> T {
        todo!("
        Update stats:
        1. Increment self.stats.gets
        2. Try to pop from objects
        3. If Some(obj):
           - Increment self.stats.hits
           - Update peak_allocated if needed
           - Return obj
        4. If None:
           - Increment self.stats.misses
           - Create new object
           - Increment self.stats.created_count
           - Update peak_allocated
           - Return obj
        ")
    }

    pub fn stats(&self) -> PoolStats {
        self.stats
    }

    pub fn stats_report(&self) -> String {
        todo!("
        Format stats into readable string:
        - Total gets: {}
        - Hits: {} ({:.1}%)
        - Misses: {} ({:.1}%)
        - Created: {}
        - Peak allocated: {}
        - Current allocated: {}
        - Available: {}
        ")
    }

    pub fn preallocate(&mut self, count: usize) {
        for _ in 0..count {
            let obj = (self.factory)();
            self.objects.push(obj);
            self.stats.created_count += 1;
        }
    }
}

// Add benchmark helper
#[cfg(test)]
mod benches {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_pool_vs_allocate() {
        const ITERATIONS: usize = 10_000;

        // With pool
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .max_size(10)
            .build();

        pool.lock().unwrap().preallocate(10);

        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _obj = pool.get();
        }
        let pool_duration = start.elapsed();

        // Without pool (direct allocation)
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _obj = vec![0u8; 1024];
        }
        let direct_duration = start.elapsed();

        println!("Pool:   {:?}", pool_duration);
        println!("Direct: {:?}", direct_duration);
        println!(
            "Speedup: {:.2}x",
            direct_duration.as_nanos() as f64 / pool_duration.as_nanos() as f64
        );

        let report = pool.lock().unwrap().stats_report();
        println!("\n{}", report);
    }
}
```

---

## Complete Working Example

Here's a production-quality object pool implementation:

```rust
use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

// ============================================================================
// Pool Statistics
// ============================================================================

#[derive(Clone, Copy, Default, Debug)]
pub struct PoolStats {
    pub created_count: usize,
    pub gets: usize,
    pub hits: usize,
    pub misses: usize,
    pub peak_allocated: usize,
}

impl PoolStats {
    pub fn hit_rate(&self) -> f64 {
        if self.gets == 0 {
            0.0
        } else {
            self.hits as f64 / self.gets as f64
        }
    }

    pub fn miss_rate(&self) -> f64 {
        if self.gets == 0 {
            0.0
        } else {
            self.misses as f64 / self.gets as f64
        }
    }
}

// ============================================================================
// Pool Implementation
// ============================================================================

pub struct Pool<T: Send> {
    objects: Vec<T>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
    stats: PoolStats,
}

pub type PoolRef<T> = Arc<Mutex<Pool<T>>>;

impl<T: Send> Pool<T> {
    pub fn builder() -> PoolBuilder<T> {
        PoolBuilder::new()
    }

    fn get_or_create(&mut self) -> T {
        self.stats.gets += 1;

        if let Some(obj) = self.objects.pop() {
            self.stats.hits += 1;

            // Update peak
            let allocated = self.stats.created_count - self.objects.len();
            if allocated > self.stats.peak_allocated {
                self.stats.peak_allocated = allocated;
            }

            obj
        } else {
            self.stats.misses += 1;
            let obj = (self.factory)();
            self.stats.created_count += 1;

            if self.stats.created_count > self.stats.peak_allocated {
                self.stats.peak_allocated = self.stats.created_count;
            }

            obj
        }
    }

    fn return_object(&mut self, mut obj: T) {
        if let Some(ref reset) = self.reset {
            reset(&mut obj);
        }

        if self.objects.len() < self.max_size {
            self.objects.push(obj);
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn allocated(&self) -> usize {
        self.stats.created_count - self.objects.len()
    }

    pub fn available(&self) -> usize {
        self.objects.len()
    }

    pub fn stats(&self) -> PoolStats {
        self.stats
    }

    pub fn stats_report(&self) -> String {
        format!(
            "Pool Statistics:\n\
             - Total gets: {}\n\
             - Hits: {} ({:.1}%)\n\
             - Misses: {} ({:.1}%)\n\
             - Created: {}\n\
             - Peak allocated: {}\n\
             - Current allocated: {}\n\
             - Available: {}",
            self.stats.gets,
            self.stats.hits,
            self.stats.hit_rate() * 100.0,
            self.stats.misses,
            self.stats.miss_rate() * 100.0,
            self.stats.created_count,
            self.stats.peak_allocated,
            self.allocated(),
            self.available()
        )
    }

    pub fn preallocate(&mut self, count: usize) {
        for _ in 0..count {
            let obj = (self.factory)();
            self.objects.push(obj);
            self.stats.created_count += 1;
        }
    }
}

// ============================================================================
// Pool Builder
// ============================================================================

pub struct PoolBuilder<T: Send> {
    factory: Option<Arc<dyn Fn() -> T + Send + Sync>>,
    reset: Option<Arc<dyn Fn(&mut T) + Send + Sync>>,
    max_size: usize,
}

impl<T: Send> PoolBuilder<T> {
    pub fn new() -> Self {
        PoolBuilder {
            factory: None,
            reset: None,
            max_size: 100,
        }
    }

    pub fn factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        self.factory = Some(Arc::new(factory));
        self
    }

    pub fn reset<F>(mut self, reset: F) -> Self
    where
        F: Fn(&mut T) + Send + Sync + 'static,
    {
        self.reset = Some(Arc::new(reset));
        self
    }

    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = size;
        self
    }

    pub fn build(self) -> PoolRef<T> {
        Arc::new(Mutex::new(Pool {
            objects: Vec::new(),
            factory: self.factory.expect("factory is required"),
            reset: self.reset,
            max_size: self.max_size,
            stats: PoolStats::default(),
        }))
    }
}

// ============================================================================
// Pooled Object (RAII Wrapper)
// ============================================================================

pub struct PooledObject<T: Send> {
    object: Option<T>,
    pool: PoolRef<T>,
}

impl<T: Send> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.object.take() {
            self.pool.lock().unwrap().return_object(obj);
        }
    }
}

impl<T: Send> Deref for PooledObject<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.object.as_ref().unwrap()
    }
}

impl<T: Send> DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.object.as_mut().unwrap()
    }
}

// ============================================================================
// Pool Extension Trait
// ============================================================================

pub trait PoolExt<T: Send> {
    fn get(&self) -> Option<PooledObject<T>>;
}

impl<T: Send> PoolExt<T> for PoolRef<T> {
    fn get(&self) -> Option<PooledObject<T>> {
        let obj = self.lock().unwrap().get_or_create();
        Some(PooledObject {
            object: Some(obj),
            pool: self.clone(),
        })
    }
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    use std::thread;
    use std::time::Duration;

    // Create buffer pool
    let pool = Pool::builder()
        .factory(|| Vec::with_capacity(1024))
        .reset(|vec: &mut Vec<u8>| vec.clear())
        .max_size(10)
        .build();

    // Preallocate
    pool.lock().unwrap().preallocate(5);

    println!("Initial state:");
    println!("{}\n", pool.lock().unwrap().stats_report());

    // Simulate work
    println!("Processing 20 tasks across 4 threads...\n");

    let mut handles = vec![];

    for thread_id in 0..4 {
        let pool_clone = pool.clone();

        let handle = thread::spawn(move || {
            for task_id in 0..5 {
                let mut buffer = pool_clone.get().unwrap();

                // Simulate work
                buffer.extend_from_slice(format!("Thread {} Task {}", thread_id, task_id).as_bytes());
                thread::sleep(Duration::from_millis(10));

                println!("Thread {} completed task {}", thread_id, task_id);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nFinal statistics:");
    println!("{}", pool.lock().unwrap().stats_report());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_workflow() {
        let pool = Pool::builder()
            .factory(|| vec![0u8; 1024])
            .reset(|v: &mut Vec<u8>| v.clear())
            .max_size(5)
            .build();

        pool.lock().unwrap().preallocate(3);

        {
            let mut b1 = pool.get().unwrap();
            let mut b2 = pool.get().unwrap();

            b1.push(1);
            b2.push(2);
        }

        let stats = pool.lock().unwrap().stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(pool.lock().unwrap().available(), 3);
    }
}
```

**Example Output:**
```
Initial state:
Pool Statistics:
- Total gets: 0
- Hits: 0 (0.0%)
- Misses: 0 (0.0%)
- Created: 5
- Peak allocated: 0
- Current allocated: 0
- Available: 5

Processing 20 tasks across 4 threads...

Thread 0 completed task 0
Thread 1 completed task 0
Thread 2 completed task 0
Thread 3 completed task 0
Thread 0 completed task 1
...

Final statistics:
Pool Statistics:
- Total gets: 20
- Hits: 15 (75.0%)
- Misses: 5 (25.0%)
- Created: 10
- Peak allocated: 4
- Current allocated: 0
- Available: 10
```

---

## Summary

You've built a production-grade object pool with all the features of real-world pools!

### Features Implemented
1. ✅ Manual object management (Milestone 1)
2. ✅ Automatic RAII return (Milestone 2)
3. ✅ Dynamic growth with Rc (Milestone 3)
4. ✅ Object reset hooks (Milestone 4)
5. ✅ Thread-safe with Arc/Mutex (Milestone 5)
6. ✅ Performance monitoring (Milestone 6)

### Smart Pointer Patterns Used
- `Box<dyn Fn>`: Store factory and reset functions
- `Rc<RefCell<>>`: Shared pool (single-threaded)
- `Arc<Mutex<>>`: Shared pool (multi-threaded)
- Custom Drop: RAII automatic cleanup
- Deref/DerefMut: Transparent smart pointer

### Performance Impact (Typical)
| Operation | Without Pool | With Pool | Speedup |
|-----------|-------------|-----------|---------|
| 1KB buffer | 500ns | 50ns | 10x |
| DB connection | 50ms | 10μs | 5,000x |
| Regex engine | 1ms | 0ns | ∞ |

### Real-World Uses
- **r2d2**: Rust DB connection pooling (PostgreSQL, MySQL)
- **deadpool**: Async-aware connection pools
- **object-pool**: General-purpose crate
- **threadpool**: Worker thread pooling

### Key Lessons
1. **RAII is powerful**: Automatic cleanup prevents leaks
2. **Builder pattern**: Makes complex initialization clean
3. **Reset hooks**: Essential for correct reuse
4. **Statistics matter**: Production systems need observability
5. **Thread-safety costs**: Arc/Mutex adds overhead but enables parallelism

Congratulations! You understand the patterns behind every high-performance pool in Rust!
