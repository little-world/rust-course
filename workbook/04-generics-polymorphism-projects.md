# Chapter 4: Generics & Polymorphism - Programming Projects

## Project 1: Generic Priority Queue with Custom Ordering

### Problem Statement

Build a generic priority queue data structure that can work with any type implementing `Ord`. The queue should support:
- Inserting elements with automatic ordering
- Removing the highest priority element
- Peeking at the highest priority element without removing it
- Custom comparison strategies through trait bounds
- Efficient implementation using a binary heap
- Support for both min-heap and max-heap configurations using phantom types

The priority queue must be fully generic over the element type and provide compile-time guarantees about ordering requirements.

### Why It Matters

Priority queues are fundamental data structures used in:
- **Operating Systems**: Process scheduling, interrupt handling
- **Algorithms**: Dijkstra's shortest path, A* search, Huffman coding
- **Real-time Systems**: Event processing by priority
- **Resource Management**: Task queuing, load balancing

Understanding how to implement generic collections with trait bounds teaches you how Rust's standard library works internally. You'll learn why `BinaryHeap<T>` requires `T: Ord` and how to design APIs that are both flexible and type-safe.

### Use Cases

1. **Task Scheduler**: Schedule tasks by priority, deadline, or custom business logic
2. **Event-Driven Systems**: Process events in priority order
3. **Graph Algorithms**: Implement A*, Dijkstra, Prim's algorithm efficiently
4. **Median Finding**: Maintain streaming median using two heaps
5. **Merge K Sorted Lists**: Efficiently merge sorted iterators
6. **Job Queue Systems**: Background job processing with priority levels

### Solution Outline

**Core Structure:**
```rust
// Use phantom type to distinguish min-heap from max-heap
use std::marker::PhantomData;

struct MinHeap;
struct MaxHeap;

pub struct PriorityQueue<T, Order = MinHeap> {
    heap: Vec<T>,
    _order: PhantomData<Order>,
}
```

**Key Methods to Implement:**
- `new()` - Create empty queue
- `push(item: T)` - Insert element (sift up to maintain heap property)
- `pop() -> Option<T>` - Remove and return highest priority element (sift down)
- `peek() -> Option<&T>` - View highest priority element
- `len()`, `is_empty()` - Basic queries
- `from_vec(vec: Vec<T>)` - Build heap from existing data (heapify)

**Trait Bounds Strategy:**
- Start with `T: Ord` for basic comparison
- Add `where` clauses for methods that need additional bounds
- Implement custom ordering through wrapper types
- Use associated types for extensibility

**Heap Operations:**
- **Sift Up**: When inserting, bubble element up to restore heap property
- **Sift Down**: When removing root, move last element to root and bubble down
- **Heapify**: Build heap from unordered array in O(n) time

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_basic_operations() {
    let mut pq = PriorityQueue::new();
    pq.push(5);
    pq.push(3);
    pq.push(7);
    assert_eq!(pq.pop(), Some(3)); // Min heap
}

#[test]
fn test_heap_property() {
    // Verify heap property holds after every operation
    // Parent should be ≤ children (min heap) or ≥ (max heap)
}

#[test]
fn test_generic_types() {
    // Test with different types: i32, String, custom structs
}
```

**Property-Based Testing:**
- Insertion order shouldn't matter for final sorted output
- Popping all elements should yield sorted sequence
- Heap property should hold after any operation

**Performance Tests:**
- Benchmark insertion of N elements
- Compare heapify vs individual inserts
- Test with large datasets (1M+ elements)

---

## Step-by-Step Implementation Guide

### Step 1: Basic Generic Structure with Vec Backend

**Goal:** Create a working priority queue using a `Vec<T>` with naive sorting.

**What to implement:**
```rust
pub struct PriorityQueue<T> {
    items: Vec<T>,
}

impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self { /* ... */ }
    pub fn push(&mut self, item: T) { /* items.push + items.sort() */ }
    pub fn pop(&mut self) -> Option<T> { /* items.pop() */ }
    pub fn peek(&self) -> Option<&T> { /* items.last() */ }
    pub fn len(&self) -> usize { /* ... */ }
    pub fn is_empty(&self) -> bool { /* ... */ }
}
```

**Check/Test:**
- Insert elements and pop them back in sorted order
- Test with `i32`, `String`, and custom `Ord` types
- Verify basic operations work correctly

**Why this isn't enough:**
The naive approach sorts the entire vector on every insertion, giving O(n log n) insertion time. For a priority queue processing thousands of events per second, this is unacceptable. A 1000-element queue would perform ~10,000 comparisons per insert instead of ~10 with a proper heap.

---

### Step 2: Implement Binary Heap Structure (Sift Operations)

**Goal:** Replace naive sorting with proper heap operations for O(log n) efficiency.

**What to improve:**
- Implement `sift_up()` - bubble newly inserted element to correct position
- Implement `sift_down()` - after removing root, restore heap property
- Change `push()` to: append to end, then sift_up
- Change `pop()` to: swap root with last, remove last, sift_down root

**Key insight - Heap indexing:**
```rust
fn parent(i: usize) -> usize { (i - 1) / 2 }
fn left_child(i: usize) -> usize { 2 * i + 1 }
fn right_child(i: usize) -> usize { 2 * i + 2 }
```

**Check/Test:**
- Write a `verify_heap_property()` helper that checks parent ≤ children
- Test that property holds after each push/pop
- Benchmark: should now handle 10k insertions quickly

**Why this isn't enough:**
We're limited to natural ordering (`T: Ord`). What if we want max-heap instead of min-heap? What if we want custom comparison logic (e.g., prioritize by deadline, not arrival time)? The current design can't handle these without code duplication.

---

### Step 3: Add Phantom Types for Min/Max Heap Variants

**Goal:** Use phantom types to support both min-heap and max-heap at compile time.

**What to improve:**
```rust
pub struct MinHeap;
pub struct MaxHeap;

pub struct PriorityQueue<T, Order = MinHeap> {
    heap: Vec<T>,
    _order: PhantomData<Order>,
}

trait HeapOrder {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool;
}

impl HeapOrder for MinHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent > child  // Min heap: parent should be ≤ child
    }
}

impl HeapOrder for MaxHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent < child  // Max heap: parent should be ≥ child
    }
}
```

Update sift operations to use `Order::should_swap()`.

**Check/Test:**
- Test `PriorityQueue<i32, MinHeap>` returns smallest first
- Test `PriorityQueue<i32, MaxHeap>` returns largest first
- Verify `PhantomData` has zero size with `mem::size_of`

**Why this isn't enough:**
Phantom types work for min/max, but what about more complex orderings? Real systems need custom priorities: tasks with deadlines, events with categories, items with multi-field comparisons. The heap element type and comparison logic are tightly coupled.

---

### Step 4: Support Custom Orderings with Wrapper Types

**Goal:** Allow custom comparison strategies while maintaining type safety.

**What to improve:**
Create wrapper types that implement custom `Ord`:

```rust
// Reverse natural ordering
pub struct Reverse<T>(pub T);

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)  // Reversed!
    }
}
// Also implement PartialOrd, Eq, PartialEq

// Priority by key
pub struct ByKey<T, K, F> {
    pub item: T,
    key_fn: F,
    _key: PhantomData<K>,
}

impl<T, K: Ord, F: Fn(&T) -> K> Ord for ByKey<T, K, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.key_fn)(&self.item).cmp(&(other.key_fn)(&other.item))
    }
}
```

**Usage example:**
```rust
// Max heap using Reverse
let mut max_heap = PriorityQueue::<Reverse<i32>>::new();

// Priority by deadline
struct Task { name: String, deadline: u64 }
let mut tasks = PriorityQueue::new();
tasks.push(ByKey::new(task, |t| t.deadline));
```

**Check/Test:**
- Test reverse ordering with `Reverse<T>`
- Create custom comparison for multi-field structs
- Test priority queue of tasks sorted by deadline

**Why this isn't enough:**
Building a heap from an existing collection currently requires pushing N elements one at a time: O(n log n). There's a faster O(n) heapify algorithm. Also, we're doing redundant comparisons—every operation does bounds checking and comparison separately.

---

### Step 5: Implement Efficient Heapify (O(n) from Vec)

**Goal:** Add efficient bulk construction from existing data.

**What to improve:**
```rust
impl<T: Ord, Order> PriorityQueue<T, Order> {
    pub fn from_vec(mut vec: Vec<T>) -> Self {
        // Build heap bottom-up: start from last parent, sift_down all
        let last_parent = vec.len() / 2;
        for i in (0..=last_parent).rev() {
            Self::sift_down_range(&mut vec, i, vec.len());
        }
        PriorityQueue { heap: vec, _order: PhantomData }
    }

    fn sift_down_range(heap: &mut [T], start: usize, end: usize) {
        // Sift down implementation working on a slice
    }
}
```

**Key insight:** Heapify from bottom-up is O(n) because:
- Half the elements are leaves (do nothing)
- Quarter need 1 comparison
- Eighth need 2 comparisons, etc.
- Sum: n * (1/2·0 + 1/4·1 + 1/8·2 + ...) = O(n)

**Check/Test:**
- Test `from_vec()` produces valid heap
- Benchmark: `from_vec()` vs repeated `push()` for 100k elements
- Should see ~2-3x speedup for bulk construction

**Why this isn't enough:**
Performance is good for single-threaded use, but what about memory efficiency? When elements are large structs, we're moving them around in memory. Can we work with references or indices? Also, no iterator support—can't use this with Rust's powerful iterator ecosystem.

---

### Step 6: Add Iterator Support and Memory Optimizations

**Goal:** Make the priority queue work with Rust's iterator ecosystem and optimize memory usage.

**What to improve:**

**1. Iterator implementations:**
```rust
impl<T, Order> IntoIterator for PriorityQueue<T, Order>
where
    T: Ord,
    Order: HeapOrder,
{
    type Item = T;
    type IntoIter = IntoIter<T, Order>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { queue: self }
    }
}

pub struct IntoIter<T, Order> {
    queue: PriorityQueue<T, Order>,
}

impl<T: Ord, Order: HeapOrder> Iterator for IntoIter<T, Order> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.queue.len();
        (len, Some(len))
    }
}

impl<T: Ord, Order: HeapOrder> ExactSizeIterator for IntoIter<T, Order> {}
```

**2. FromIterator for easy construction:**
```rust
impl<T: Ord, Order> FromIterator<T> for PriorityQueue<T, Order> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        Self::from_vec(vec)
    }
}
```

**3. Memory optimizations:**
- Add `with_capacity(cap: usize)` for pre-allocation
- Add `shrink_to_fit()` to release excess capacity
- Add `reserve(additional: usize)` for growth planning
- Consider `drain()` method for consuming elements

**Check/Test:**
- Test iterator produces elements in sorted order
- Test `collect()` into PriorityQueue
- Test chaining: `values.into_iter().filter(...).collect::<PriorityQueue<_>>()`
- Benchmark memory usage with large structs
- Test iterator `size_hint()` accuracy

**What this achieves:**
Now your priority queue is a first-class Rust collection:
- Works seamlessly with iterator chains
- Memory-efficient construction from iterators
- Predictable performance through capacity management
- Zero-cost abstractions—compiles to the same code as hand-written loops

**Extensions to explore:**
- `Drain` iterator for partial consumption
- `peek_mut()` for in-place modification (tricky—requires sift on drop!)
- Parallel heapify using Rayon
- `merge()` operation for combining two heaps
- `extend()` for bulk insertions

---

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

## Project 3: Generic Cache with Expiration Strategies

### Problem Statement

Implement a generic in-memory cache that supports:
- Multiple eviction policies (LRU, LFU, FIFO, TTL)
- Compile-time selection of eviction strategy using generics
- Thread-safe concurrent access
- Configurable capacity limits
- Statistics tracking (hit rate, miss rate, eviction count)
- Lazy expiration (items expire on access, not actively)
- Optional write-through to backing store

The cache must work with any key type implementing `Hash + Eq` and any value type, maintaining O(1) get/put performance.

### Why It Matters

Caching is fundamental to high-performance systems:
- **Web Servers**: Cache rendered pages, database queries, session data
- **Databases**: Buffer pool, query result cache
- **Operating Systems**: Page cache, inode cache
- **CDNs**: Cache static assets globally
- **Machine Learning**: Cache computed features, model predictions

Understanding cache implementation teaches:
- How generics enable reusable data structures
- Trade-offs between different eviction policies
- Concurrent data structure design
- Performance optimization techniques

### Use Cases

1. **Web Application**: Cache database query results, API responses
2. **Distributed Systems**: Local cache to reduce network calls
3. **Compilers**: Cache parsed ASTs, compiled artifacts
4. **Image Processing**: Cache thumbnails, transformed images
5. **Game Development**: Asset cache for textures, models
6. **DNS Resolver**: Cache domain name lookups

### Solution Outline

**Core Structure:**
```rust
use std::hash::Hash;
use std::collections::HashMap;

// Eviction strategy trait
pub trait EvictionPolicy<K> {
    fn on_get(&mut self, key: &K);
    fn on_put(&mut self, key: K);
    fn evict_candidate(&self) -> Option<K>;
}

// Cache with generic eviction policy
pub struct Cache<K, V, E: EvictionPolicy<K>> {
    data: HashMap<K, V>,
    eviction: E,
    capacity: usize,
    stats: CacheStats,
}

pub struct CacheStats {
    hits: usize,
    misses: usize,
    evictions: usize,
}
```

**Eviction Policies to Implement:**

1. **LRU (Least Recently Used)**
   - Track access order with doubly-linked list
   - Evict least recently accessed item
   - Use case: General-purpose caching

2. **LFU (Least Frequently Used)**
   - Track access frequency counter
   - Evict least frequently accessed item
   - Use case: Popular item caching

3. **FIFO (First In First Out)**
   - Track insertion order
   - Evict oldest item
   - Use case: Simple caching, log buffers

4. **TTL (Time To Live)**
   - Track insertion/access timestamp
   - Evict expired items
   - Use case: Session caching, rate limiting

**Performance Targets:**
- `get()`: O(1) average case
- `put()`: O(1) average case (amortized for eviction)
- Memory overhead: < 50% of stored data

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_lru_eviction() {
    let mut cache = Cache::with_lru(2);
    cache.put(1, "a");
    cache.put(2, "b");
    cache.get(&1); // Access 1, making it more recent
    cache.put(3, "c"); // Should evict 2 (least recent)

    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), None); // Evicted
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn test_capacity_limit() {
    // Verify cache never exceeds capacity
}

#[test]
fn test_hit_miss_stats() {
    // Verify statistics are accurately tracked
}
```

**Concurrency Tests:**
```rust
#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(Cache::with_lru(100));
    let mut handles = vec![];

    for i in 0..10 {
        let cache = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                cache.put(i * 100 + j, format!("value{}", j));
                cache.get(&(i * 100 + j));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Performance Tests:**
```rust
#[bench]
fn bench_cache_operations(b: &mut Bencher) {
    let mut cache = Cache::with_lru(1000);
    b.iter(|| {
        for i in 0..1000 {
            cache.put(i, i * 2);
        }
        for i in 0..1000 {
            black_box(cache.get(&i));
        }
    });
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic HashMap-Backed Cache with Fixed Size

**Goal:** Create a simple cache using `HashMap` with naive eviction (random).

**What to implement:**
```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct Cache<K, V> {
    data: HashMap<K, V>,
    capacity: usize,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity && !self.data.contains_key(&key) {
            // Naive: remove first key found (random due to HashMap iteration)
            if let Some(k) = self.data.keys().next().cloned() {
                self.data.remove(&k);
            }
        }
        self.data.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
```

**Check/Test:**
- Test basic get/put operations
- Verify cache respects capacity limit
- Test with different key/value types (String, i32, custom structs)

**Why this isn't enough:**
Random eviction is useless for real caching—no locality benefit, poor hit rate. A cache that evicts randomly performs barely better than no cache at all. We need smart eviction policies (LRU, LFU) that keep hot data. Also, no statistics—we can't measure cache effectiveness.

---

### Step 2: Add LRU Eviction with Doubly-Linked List

**Goal:** Implement proper LRU (Least Recently Used) eviction policy.

**What to improve:**

Rust doesn't have a built-in doubly-linked list suitable for this, so we'll use indices and a VecDeque:

```rust
use std::collections::{HashMap, VecDeque};

pub struct LruCache<K, V> {
    data: HashMap<K, V>,
    order: VecDeque<K>,  // Front = most recent, back = least recent
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCache {
            data: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.data.contains_key(key) {
            // Move to front (most recent)
            self.touch(key);
            self.data.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            // Update existing
            self.touch(&key);
            self.data.insert(key, value);
        } else {
            // Evict if at capacity
            if self.data.len() >= self.capacity {
                if let Some(lru_key) = self.order.pop_back() {
                    self.data.remove(&lru_key);
                }
            }

            self.data.insert(key.clone(), value);
            self.order.push_front(key);
        }
    }

    fn touch(&mut self, key: &K) {
        // Remove from current position and move to front
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_front(key.clone());
        }
    }
}
```

**Check/Test:**
- Test LRU behavior: least recently accessed item is evicted
- Test that `get()` updates access order
- Test that updating existing keys doesn't grow cache
- Verify O(n) worst case for touch operation (problem for next step)

**Why this isn't enough:**
The `touch()` operation is O(n) because we're searching through `VecDeque` to find and remove the key. For a large cache (10k+ items), this kills performance. We need O(1) access to list nodes. Also, we only have LRU—what about other policies? The structure is hardcoded for one strategy.

---

### Step 3: Make Eviction Policy Generic with Trait

**Goal:** Abstract eviction logic behind a trait, supporting multiple policies.

**What to improve:**
```rust
use std::hash::Hash;
use std::collections::HashMap;

// Trait for eviction policies
pub trait EvictionPolicy<K> {
    fn new(capacity: usize) -> Self;
    fn on_get(&mut self, key: &K);
    fn on_put(&mut self, key: &K);
    fn on_remove(&mut self, key: &K);
    fn evict_candidate(&self) -> Option<K>;
    fn len(&self) -> usize;
}

// Generic cache
pub struct Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    data: HashMap<K, V>,
    eviction: E,
    capacity: usize,
}

impl<K, V, E> Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::with_capacity(capacity),
            eviction: E::new(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let result = self.data.get(key);
        if result.is_some() {
            self.eviction.on_get(key);
        }
        result
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            self.eviction.on_put(&key);
            self.data.insert(key, value);
        } else {
            if self.data.len() >= self.capacity {
                if let Some(victim) = self.eviction.evict_candidate() {
                    self.data.remove(&victim);
                    self.eviction.on_remove(&victim);
                }
            }

            self.eviction.on_put(&key);
            self.data.insert(key, value);
        }
    }
}

// Implement LRU policy
pub struct LruPolicy<K> {
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Clone + Eq> EvictionPolicy<K> for LruPolicy<K> {
    fn new(capacity: usize) -> Self {
        LruPolicy {
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        // Move to front
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_front(key.clone());
        }
    }

    fn on_put(&mut self, key: &K) {
        self.order.push_front(key.clone());
    }

    fn on_remove(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    fn evict_candidate(&self) -> Option<K> {
        self.order.back().cloned()
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

// Implement FIFO policy
pub struct FifoPolicy<K> {
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Clone + Eq> EvictionPolicy<K> for FifoPolicy<K> {
    fn new(capacity: usize) -> Self {
        FifoPolicy {
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, _key: &K) {
        // FIFO doesn't care about access
    }

    fn on_put(&mut self, key: &K) {
        self.order.push_front(key.clone());
    }

    fn on_remove(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    fn evict_candidate(&self) -> Option<K> {
        self.order.back().cloned()
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

// Type aliases for convenience
pub type LruCache<K, V> = Cache<K, V, LruPolicy<K>>;
pub type FifoCache<K, V> = Cache<K, V, FifoPolicy<K>>;
```

**Check/Test:**
- Test both LRU and FIFO caches behave correctly
- Verify policy trait abstraction works
- Test that FIFO doesn't update order on `get()`

**Why this isn't enough:**
Still O(n) performance for LRU due to VecDeque search. We need a better data structure—an intrusive doubly-linked list backed by HashMap for O(1) operations. Also, no statistics tracking yet (hit rate, miss rate). We can't measure cache effectiveness.

---

### Step 4: Optimize LRU to O(1) with HashMap + Linked List

**Goal:** Achieve true O(1) get/put for LRU using a custom linked list structure.

**What to improve:**

Use a pattern similar to `std::collections::LinkedHashMap` (not in std, but we can build it):

```rust
use std::collections::HashMap;
use std::ptr::NonNull;

// Doubly-linked list node
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

pub struct LruCacheOptimized<K, V>
where
    K: Hash + Eq + Clone,
{
    map: HashMap<K, NonNull<Node<K, V>>>,
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCacheOptimized<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCacheOptimized {
            map: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&node_ptr) = self.map.get(key) {
            unsafe {
                self.move_to_front(node_ptr);
                Some(&(*node_ptr.as_ptr()).value)
            }
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            unsafe {
                (*node_ptr.as_ptr()).value = value;
                self.move_to_front(node_ptr);
            }
        } else {
            if self.map.len() >= self.capacity {
                self.remove_tail();
            }

            let mut node = Box::new(Node {
                key: key.clone(),
                value,
                prev: None,
                next: self.head,
            });

            let node_ptr = NonNull::new(Box::into_raw(node)).unwrap();

            if let Some(mut head) = self.head {
                unsafe {
                    (*head.as_ptr()).prev = Some(node_ptr);
                }
            } else {
                self.tail = Some(node_ptr);
            }

            self.head = Some(node_ptr);
            self.map.insert(key, node_ptr);
        }
    }

    unsafe fn move_to_front(&mut self, node_ptr: NonNull<Node<K, V>>) {
        let node = node_ptr.as_ptr();

        if self.head == Some(node_ptr) {
            return; // Already at front
        }

        // Remove from current position
        if let Some(mut prev) = (*node).prev {
            (*prev.as_ptr()).next = (*node).next;
        }

        if let Some(mut next) = (*node).next {
            (*next.as_ptr()).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }

        // Move to front
        (*node).prev = None;
        (*node).next = self.head;

        if let Some(mut head) = self.head {
            (*head.as_ptr()).prev = Some(node_ptr);
        }

        self.head = Some(node_ptr);
    }

    fn remove_tail(&mut self) {
        if let Some(tail_ptr) = self.tail {
            unsafe {
                let tail = tail_ptr.as_ptr();
                let key = (*tail).key.clone();

                self.map.remove(&key);

                if let Some(mut prev) = (*tail).prev {
                    (*prev.as_ptr()).next = None;
                    self.tail = Some(prev);
                } else {
                    self.head = None;
                    self.tail = None;
                }

                // Free the node
                drop(Box::from_raw(tail));
            }
        }
    }
}

impl<K, V> Drop for LruCacheOptimized<K, V>
where
    K: Hash + Eq + Clone,
{
    fn drop(&mut self) {
        let mut current = self.head;
        while let Some(node_ptr) = current {
            unsafe {
                let node = Box::from_raw(node_ptr.as_ptr());
                current = node.next;
            }
        }
    }
}
```

**Important:** This uses unsafe code. In a real implementation, consider using a safe library like `lru` crate or refactoring with indices.

**Check/Test:**
- Benchmark O(1) performance for large caches (10k+ items)
- Test with Miri or AddressSanitizer for memory safety
- Verify no memory leaks with valgrind or similar

**Why this isn't enough:**
We've optimized LRU, but:
1. No statistics tracking (hit/miss rate, eviction count)
2. Not thread-safe—can't share across threads
3. No TTL (time-based expiration) support
4. No LFU (Least Frequently Used) policy

Let's add statistics and thread safety next.

---

### Step 5: Add Statistics and Thread Safety

**Goal:** Track cache performance metrics and make the cache thread-safe.

**What to improve:**

**1. Add statistics:**
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct CacheStats {
    hits: AtomicUsize,
    misses: AtomicUsize,
    evictions: AtomicUsize,
    inserts: AtomicUsize,
}

impl CacheStats {
    pub fn hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn summary(&self) -> StatsSummary {
        StatsSummary {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            inserts: self.inserts.load(Ordering::Relaxed),
        }
    }
}

pub struct StatsSummary {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub inserts: usize,
}
```

**2. Add thread safety with RwLock:**
```rust
use std::sync::{Arc, RwLock};

pub struct ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    inner: Arc<RwLock<Cache<K, V, E>>>,
    stats: Arc<CacheStats>,
}

impl<K, V, E> Clone for ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    fn clone(&self) -> Self {
        ThreadSafeCache {
            inner: Arc::clone(&self.inner),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl<K, V, E> ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    V: Clone,
    E: EvictionPolicy<K>,
{
    pub fn new(capacity: usize) -> Self {
        ThreadSafeCache {
            inner: Arc::new(RwLock::new(Cache::new(capacity))),
            stats: Arc::new(CacheStats::default()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write().unwrap();
        let result = cache.get(key).cloned();

        if result.is_some() {
            self.stats.hit();
        } else {
            self.stats.miss();
        }

        result
    }

    pub fn put(&self, key: K, value: V) {
        let mut cache = self.inner.write().unwrap();
        let was_full = cache.len() >= cache.capacity();

        cache.put(key, value);

        if was_full {
            self.stats.eviction();
        }
        self.stats.insert();
    }

    pub fn stats(&self) -> StatsSummary {
        self.stats.summary()
    }
}
```

**Check/Test:**
- Test concurrent access from multiple threads
- Verify statistics are accurate under concurrent load
- Test that hit/miss rate calculation is correct
- Benchmark performance with contention

**Why this isn't enough:**
Write lock for reads is inefficient—multiple readers could access simultaneously, but `get()` updates LRU order (mutable). This creates contention. Also, still no TTL support for time-based expiration. Large caches can grow stale without expiration.

---

### Step 6: Add TTL Support and Lazy Expiration

**Goal:** Implement time-based expiration with lazy eviction on access.

**What to improve:**

**1. Add TTL policy:**
```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct TtlPolicy<K> {
    timestamps: HashMap<K, Instant>,
    ttl: Duration,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for TtlPolicy<K> {
    fn new(capacity: usize) -> Self {
        TtlPolicy {
            timestamps: HashMap::with_capacity(capacity),
            ttl: Duration::from_secs(300), // Default 5 minutes
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        // Update timestamp on access
        self.timestamps.insert(key.clone(), Instant::now());
    }

    fn on_put(&mut self, key: &K) {
        self.timestamps.insert(key.clone(), Instant::now());
    }

    fn on_remove(&mut self, key: &K) {
        self.timestamps.remove(key);
    }

    fn evict_candidate(&self) -> Option<K> {
        let now = Instant::now();

        // Find first expired item
        self.timestamps
            .iter()
            .find(|(_, &timestamp)| now.duration_since(timestamp) > self.ttl)
            .map(|(k, _)| k.clone())
            .or_else(|| {
                // If none expired, evict oldest
                self.timestamps
                    .iter()
                    .min_by_key(|(_, &timestamp)| timestamp)
                    .map(|(k, _)| k.clone())
            })
    }

    fn len(&self) -> usize {
        self.timestamps.len()
    }
}

impl<K> TtlPolicy<K> {
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        TtlPolicy {
            timestamps: HashMap::with_capacity(capacity),
            ttl,
            capacity,
        }
    }
}
```

**2. Add lazy expiration check to Cache:**
```rust
impl<K, V, E> Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check for expiration first
        if self.is_expired(key) {
            self.remove(key);
            return None;
        }

        let result = self.data.get(key);
        if result.is_some() {
            self.eviction.on_get(key);
        }
        result
    }

    fn is_expired(&self, key: &K) -> bool {
        // Policies can implement expiration check
        // For TTL: check if timestamp + ttl < now
        false // Default: no expiration
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.eviction.on_remove(key);
        self.data.remove(key)
    }

    pub fn cleanup_expired(&mut self) -> usize {
        // Periodic cleanup of expired items
        let mut count = 0;
        let expired: Vec<K> = self.data
            .keys()
            .filter(|k| self.is_expired(k))
            .cloned()
            .collect();

        for key in expired {
            self.remove(&key);
            count += 1;
        }

        count
    }
}
```

**3. Add LFU (Least Frequently Used) policy:**
```rust
pub struct LfuPolicy<K> {
    frequencies: HashMap<K, usize>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for LfuPolicy<K> {
    fn new(capacity: usize) -> Self {
        LfuPolicy {
            frequencies: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        *self.frequencies.entry(key.clone()).or_insert(0) += 1;
    }

    fn on_put(&mut self, key: &K) {
        self.frequencies.insert(key.clone(), 1);
    }

    fn on_remove(&mut self, key: &K) {
        self.frequencies.remove(key);
    }

    fn evict_candidate(&self) -> Option<K> {
        self.frequencies
            .iter()
            .min_by_key(|(_, &freq)| freq)
            .map(|(k, _)| k.clone())
    }

    fn len(&self) -> usize {
        self.frequencies.len()
    }
}
```

**Type aliases:**
```rust
pub type LruCache<K, V> = Cache<K, V, LruPolicy<K>>;
pub type LfuCache<K, V> = Cache<K, V, LfuPolicy<K>>;
pub type FifoCache<K, V> = Cache<K, V, FifoPolicy<K>>;
pub type TtlCache<K, V> = Cache<K, V, TtlPolicy<K>>;
```

**Check/Test:**
- Test TTL expiration: items expire after configured duration
- Test LFU evicts least frequently used items
- Test lazy expiration only triggers on access
- Test `cleanup_expired()` removes all expired items
- Benchmark different policies under various workloads

**What this achieves:**
- **Multiple Eviction Policies**: LRU, LFU, FIFO, TTL all supported through generic trait
- **Time-Based Expiration**: Items automatically expire after TTL
- **Performance**: O(1) operations for all policies (except expiration cleanup)
- **Thread Safety**: Safe concurrent access with Arc<RwLock>
- **Statistics**: Track hit rate, miss rate, eviction count
- **Flexibility**: Generic over key/value types and eviction policy
- **Type Safety**: Compile-time policy selection

**Extensions to explore:**
- Write-through cache: sync to backing store
- Cache warming: pre-populate from disk
- Tiered caching: L1 (memory) + L2 (disk)
- Async support: async get/put for network caches
- Bloom filters: fast negative lookups
- Compression: compress values to save memory

---

## Summary

These three projects teach complementary aspects of Rust's generics and polymorphism:

1. **Priority Queue**: Generic data structures, trait bounds, phantom types for compile-time configuration, efficient algorithms

2. **Type-State Connection Builder**: Phantom types for state machines, zero-cost state guarantees, builder pattern, compile-time safety

3. **Generic Cache**: Trait-based polymorphism, multiple implementations of abstraction, performance optimization, thread safety, statistics

All three emphasize:
- Zero-cost abstractions through generics
- Compile-time safety guarantees
- Performance-conscious design
- Practical, real-world patterns

Students should come away understanding how to design flexible, type-safe, performant generic APIs—the foundation of modern Rust systems programming.
