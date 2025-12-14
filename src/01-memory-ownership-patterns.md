#  Memory & Ownership Patterns

Rust's ownership system is its defining feature, enabling memory safety without garbage collection. This chapter explores advanced patterns that leverage ownership, borrowing, and lifetimes to write efficient, safe code. For experienced programmers, understanding these patterns is crucial for designing high-performance systems where memory allocation, cache locality, and zero-copy operations matter.

The ownership model enforces three fundamental rules at compile time:
1. Each value has exactly one owner
2. Values are dropped when their owner goes out of scope
3. References must never outlive their referents

These rules enable sophisticated zero-cost abstractions while preventing entire classes of bugs: use-after-free, double-free, dangling pointers, and data races.

## Pattern 1: Zero-Copy with Clone-on-Write (Cow)

*   **Problem**: Functions that sometimes need to modify their input face a dilemma: always clone the input (which is wasteful if no modification is needed), or require a mutable reference (which makes the API less ergonomic).
*   **Solution**: Use `Cow<T>` (Clone-on-Write). This is a smart pointer that can enclose either borrowed data (`Cow::Borrowed`) or owned data (`Cow::Owned`).
*   **Why It Matters**: This pattern enables a "fast path" for zero-allocation operations. In high-throughput systems like web servers or parsers, avoiding millions of unnecessary string allocations per second can lead to significant performance gains.

### Examples

#### Example: Conditional Modification
A common use for `Cow` is in functions that may or may not need to modify their string-like input. This `normalize_whitespace` function provides a zero-allocation "fast path". It only allocates a new `String` and returns `Cow::Owned` if the input text actually contains characters that need to be replaced. Otherwise, it returns a borrowed slice `Cow::Borrowed` without any heap allocation.

```rust
use std::borrow::Cow;

// Returns borrowed data when possible, owned only when necessary
fn normalize_whitespace(text: &str) -> Cow<str> {
    if text.contains("  ") || text.contains('\t') {
        // Only allocate if we need to modify
        let mut result = text.replace("  ", " ");
        result = result.replace('\t', " ");
        Cow::Owned(result)
    } else {
        // Zero-copy return
        Cow::Borrowed(text)
    }
}
```

#### Example: Lazy Mutation Chains
`Cow` can be used to build a chain of potential modifications. An allocation is performed only on the first step that requires a change. This example demonstrates how a path might be processed, first by expanding the tilde `~` and then by normalizing path separators. The `Cow` will only become `Owned` if one of these conditions is met.

```rust
use std::borrow::Cow;

fn process_path(path: &str) -> Cow<str> {
    let mut result = Cow::Borrowed(path);

    // Expand tilde
    if path.starts_with("~/") {
        result = Cow::Owned(path.replacen("~", "/home/user", 1));
    }

    // Normalize separators (Windows)
    if result.contains('\\') {
        result = Cow::Owned(result.replace('\\', "/"));
    }

    // Only allocates if modifications were needed
    result
}
```

#### Example: In-Place Modification with `to_mut()`
The `to_mut()` method is a powerful tool for getting a mutable reference to the underlying data. If the `Cow` is `Borrowed`, `to_mut()` will clone the data to make it `Owned` and then return a mutable reference. If it's already `Owned`, it returns a mutable reference without any allocation. This is perfect for efficient in-place modifications.

```rust
use std::borrow::Cow;

fn capitalize_first<'a>(s: &'a str) -> Cow<'a, str> {
    if let Some(first_char) = s.chars().next() {
        if first_char.is_lowercase() {
            let mut owned = s.to_string();
            owned[0..first_char.len_utf8()].make_ascii_uppercase();
            Cow::Owned(owned)
        } else {
            Cow::Borrowed(s)
        }
    } else {
        Cow::Borrowed(s)
    }
}
```

#### Use Case: Configuration with Defaults
`Cow` is excellent for handling configuration that involves default values. A `Config` struct can hold borrowed string slices for default values, avoiding allocations. If a user provides an override (an owned `String`), the `Cow` can seamlessly switch to holding the owned data.

```rust
use std::borrow::Cow;

struct Config<'a> {
    host: Cow<'a, str>,
    port: u16,
    database: Cow<'a, str>,
}

impl<'a> Config<'a> {
    fn new(host: &'a str, port: u16) -> Self {
        Config {
            host: Cow::Borrowed(host),
            port,
            // 'default_db' is a &'static str, so it can be borrowed safely.
            database: Cow::Borrowed("default_db"),
        }
    }

    fn with_database(mut self, db: String) -> Self {
        self.database = Cow::Owned(db);
        self
    }
}
```

**When to use Cow:**
- Library APIs that accept string input and may need to modify it
- Processing pipelines where some inputs need transformation, others don't
- Configuration systems with optional overrides
- Parsing where most tokens are substrings of input

**Performance characteristics:**
- Zero allocation when borrowing
- Single allocation when owned
- Same size as a pointer + discriminant (24 bytes on 64-bit)

## Pattern 2: Interior Mutability with Cell and RefCell

* **Problem**: Rust's borrowing rules require `&mut self` for mutation, but some designs need mutation through shared references (`&self`). Examples: caching computed values, counters in shared structures, graph nodes that need to update neighbors, observer patterns.
* **Solution**: Use interior mutability types—`Cell<T>` for `Copy` types (get/set without borrowing), `RefCell<T>` for non-`Copy` types (runtime-checked borrows). These move borrow checking from compile-time to runtime.
* **Why It Matters**: Some data structures are impossible without interior mutability. Doubly-linked lists, graphs with cycles, and the observer pattern all require mutation through shared references.

### The Problem: Experiencing the Borrow Checker
Let's start by trying to implement a simple counter. We want to pass this counter to multiple functions that can increment it, but we only have a shared reference (`&Counter`). This code will not compile, because `increment` requires a mutable reference `&mut self`, but `process_item` only has an immutable one.

```rust
// This is our first attempt - it seems reasonable!
struct Counter {
    count: usize,
}

impl Counter {
    fn new() -> Self { Counter { count: 0 } }
    fn increment(&mut self) { self.count += 1; }
    fn get(&self) -> usize { self.count }
}

fn process_item(counter: &Counter) {
    // Inside here, we only have &Counter, not &mut Counter
    // But we need to increment!
    // counter.increment();  // ❌ ERROR: cannot call `&mut self` method with `&self`
}
```

### The Solution for `Copy` Types: `Cell<T>`
For types that are `Copy` (like `usize`), `Cell<T>` solves the problem. It allows you to `get()` a copy of the value or `set()` a new value, even through a shared reference. Notice the `increment` method now takes `&self`, and it works perfectly.

```rust
use std::cell::Cell;

struct Counter {
    count: Cell<usize>,  // Wrapped in Cell!
}

impl Counter {
    fn new() -> Self {
        Counter { count: Cell::new(0) }
    }

    fn increment(&self) {  // ✅ Note: takes &self, not &mut self!
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> usize {
        self.count.get()
    }
}

// Now this works!
fn process_item(counter: &Counter) {
    counter.increment();  // ✅ Works even with &self!
}
```
`Cell` is safe because it never gives out references to the inner data; it only moves `Copy` values in and out.

### The Solution for Non-`Copy` Types: `RefCell<T>`
But what if the data isn't `Copy`, like a `Vec` or `HashMap`? You can't use `Cell`. The solution is `RefCell<T>`, which moves Rust's borrow checking rules from compile-time to *run-time*. You can ask to `borrow()` (immutable) or `borrow_mut()` (mutable). If you violate the rules (e.g., ask for a mutable borrow while an immutable one exists), your program will panic.

This example shows a cache that can be modified internally via `&self`.

```rust
use std::cell::RefCell;
use std::collections::HashMap;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Self {
        Cache { data: RefCell::new(HashMap::new()) }
    }

    fn get_or_compute(&self, key: &str, compute: impl FnOnce() -> String) -> String {
        // Try to get from cache (immutable borrow)
        if let Some(value) = self.data.borrow().get(key) {
            return value.clone();
        }

        // Not found, compute and insert (mutable borrow)
        let value = compute();
        self.data.borrow_mut().insert(key.to_string(), value.clone());
        value
    }
}
```

### RefCell Patterns and Pitfalls

#### Pattern: Careful Borrow Scoping
The most important pattern with `RefCell` is to keep borrow lifetimes as short as possible to avoid panics. A common way to do this is to introduce a new scope `{}`.

```rust
use std::cell::RefCell;

fn process_cache(cache: &RefCell<Vec<String>>) {
    // Read operation in its own scope
    {
        let borrowed = cache.borrow();
        println!("Cache size: {}", borrowed.len());
    } // `borrowed` guard is dropped here, releasing the borrow

    // Write operation is now safe
    cache.borrow_mut().push("new_item".to_string());
}
```

#### Pattern: Non-Panicking Borrows with `try_borrow`
If you're not sure if a borrow will succeed, use `try_borrow()` or `try_borrow_mut()`. These return a `Result` instead of panicking, allowing you to handle the "already borrowed" case gracefully.

```rust
use std::cell::RefCell;

fn safe_access(data: &RefCell<Vec<i32>>) -> Result<(), &'static str> {
    if let Ok(mut borrowed) = data.try_borrow_mut() {
        borrowed.push(42);
        Ok(())
    } else {
        Err("Could not acquire lock: data is already borrowed.")
    }
}
```

#### Use Case: Graph Structures
Interior mutability is essential for graph data structures or any time you have objects that point to each other and need to be modified, like a doubly-linked list. `Rc<RefCell<T>>` is a very common pattern for creating graph-like structures where nodes have shared ownership and can be mutated.

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    value: i32,
    edges: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn add_edge(&self, target: Rc<Node>) {
        self.edges.borrow_mut().push(target);
    }
}
```

### Summary: `Cell` vs. `RefCell`

| Feature | `Cell<T>` | `RefCell<T>` |
|---------|---|---|
| Works with | `Copy` types only | Any `Sized` type |
| API | `get()`, `set()` | `borrow()`, `borrow_mut()` |
| Checking | Compile-time (enforced by `Copy` trait) | Runtime (panics on violation) |
| Overhead | Zero | Small (a runtime borrow flag) |
| Panics? | No | **Yes**, if rules are violated |
| Thread-safe?| No | No |
| Use For | Simple `Copy` data like `u32`, `bool`. | Complex data like `Vec`, `HashMap`. |

**Critical safety note:**
- `RefCell` is for **single-threaded** scenarios only. For multiple threads, you need `Mutex` or `RwLock`.
- Always keep borrow scopes as short as possible. Never hold a borrow guard across a call to an unknown function.

## Pattern 3: Thread-Safe Interior Mutability (Mutex & RwLock)

* **Problem**: `RefCell<T>` provides interior mutability but panics if used incorrectly across threads. Multi-threaded code needs safe shared mutable state—incrementing counters, updating caches, modifying shared collections—without data races.

* **Solution**: Use `Mutex<T>` for exclusive access (like `RefCell` but thread-safe) or `RwLock<T>` for reader-writer patterns (multiple readers OR one writer). Combine with `Arc<T>` to share across threads.

* **Why It Matters**: Multi-threaded programming without data races is notoriously difficult in C/C++. Rust's type system makes it impossible to compile racy code—you must use `Mutex` or `RwLock` for shared mutation.

* **Use Cases**: Shared counters in multi-threaded servers, concurrent caches, thread pools with shared work queues, parallel data processing with result aggregation, connection pools.

### Examples

#### Example: Shared Counter Across Threads
To share mutable state across threads, you wrap it in `Arc<Mutex<T>>`. `Arc` is the "Atomically Reference Counted" pointer that lets multiple threads "own" the data. `Mutex` ensures that only one thread can access the data at a time. When `.lock()` is called, it blocks until the lock is available. The returned guard object automatically releases the lock when it goes out of scope.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn parallel_counter() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            } // lock automatically released when guard `num` is dropped
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
```

#### Example: Reader-Writer Lock for Read-Heavy Workloads
A `Mutex` is exclusive. If you have a situation where many threads need to read data and only a few need to write, a `Mutex` is inefficient. `RwLock` is the solution. It allows any number of readers to access the data simultaneously, but write access is exclusive (it waits for all readers to finish).

```rust
use std::sync::RwLock;
use std::collections::HashMap;

struct SharedCache {
    data: RwLock<HashMap<String, String>>,
}

impl SharedCache {
    fn get(&self, key: &str) -> Option<String> {
        // Multiple readers can hold read locks simultaneously.
        self.data.read().unwrap().get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        // Write lock is exclusive. It will wait for all readers to unlock.
        self.data.write().unwrap().insert(key, value);
    }
}
```

#### Example: Minimize Lock Duration
Locks can become performance bottlenecks. A critical pattern is to hold the lock for the shortest time possible. Perform expensive computations *outside* the lock, and only acquire the lock when you are ready to quickly read or write the shared data.

```rust
use std::sync::Mutex;

fn optimized_update(shared: &Mutex<Vec<i32>>, new_value: i32) {
    // Good: compute outside the lock
    let computed = expensive_computation(new_value);
    
    // Acquire lock only for the quick push operation
    shared.lock().unwrap().push(computed);
}

// Bad: holding the lock during a slow operation
fn unoptimized_update(shared: &Mutex<Vec<i32>>, new_value: i32) {
    let mut data = shared.lock().unwrap();
    let computed = expensive_computation(new_value); // Don't do this!
    data.push(computed);
}

fn expensive_computation(x: i32) -> i32 {
    std::thread::sleep(std::time::Duration::from_millis(50)); // Imagine this is slow
    x * 2
}
```

#### Example: Deadlock Prevention with Lock Ordering
A classic problem in concurrent programming is deadlock. If Thread 1 locks A and waits for B, while Thread 2 locks B and waits for A, they will wait forever. The solution is to ensure all threads acquire locks in a globally consistent order. A simple way to achieve this is to order locks by their memory address.

```rust
use std::sync::Mutex;

struct Account {
    id: u32,
    balance: Mutex<i64>,
}

fn transfer(from: &Account, to: &Account, amount: i64) {
    // To prevent deadlock, we always acquire locks in a consistent order.
    // Here, we use the account ID.
    let (lock1, lock2) = if from.id < to.id {
        (from.balance.lock().unwrap(), to.balance.lock().unwrap())
    } else {
        (to.balance.lock().unwrap(), from.balance.lock().unwrap())
    };

    // Now that locks are acquired, we can perform the logic.
    // Note: this logic is simplified and assumes the `if` branch matches the original intent.
    // A real implementation would need to handle the amounts correctly regardless of lock order.
}
```

#### Example: Non-Blocking Access with `try_lock`
Sometimes, you don't want to wait for a lock. You'd rather do something else if the data is currently locked. `try_lock` returns immediately with a `Result`. If it acquires the lock, it returns `Ok(Guard)`; if not, it returns `Err`.

```rust
use std::sync::Mutex;

fn try_update(data: &Mutex<Vec<i32>>) -> Result<(), &'static str> {
    if let Ok(mut guard) = data.try_lock() {
        guard.push(42);
        Ok(())
    } else {
        Err("Lock held by another thread, skipping update.")
    }
}
```

**Mutex vs RwLock trade-offs:**
- **Mutex**: Simpler, lower overhead, exclusive access
- **RwLock**: Multiple readers, write-heavy can starve readers
- RwLock ~3x slower for writes, but allows concurrent reads
- Use Mutex unless >70% reads and contention is proven issue

**Lock granularity strategies:**
- Fine-grained: More parallelism, higher overhead, deadlock risk
- Coarse-grained: Less parallelism, simpler reasoning
- Profile first, optimize second

## Pattern 4: Custom Drop Guards

* **Problem**: Manual resource cleanup is error-prone. Forgetting to close files, release locks, or rollback transactions causes resource leaks, deadlocks, and data corruption.
* **Solution**: Implement the `Drop` trait to tie resource cleanup to scope. Create guard types that acquire resources in their constructor and release them in `Drop`.
* **Why It Matters**: RAII eliminates entire categories of bugs. You cannot forget to unlock a `Mutex`—`MutexGuard`'s `Drop` releases it automatically.


### Examples

#### Example: Temporary File Guard
This `TempFile` struct creates a file upon construction. The `Drop` implementation ensures that no matter how the function exits—success, error, or panic—the file is guaranteed to be deleted.

```rust
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

struct TempFile {
    path: PathBuf,
    file: File,
}

impl TempFile {
    fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::create(&path)?;
        Ok(TempFile { path, file })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // Cleanup happens automatically when TempFile goes out of scope.
        println!("Dropping TempFile, deleting {}", self.path.display());
        let _ = std::fs::remove_file(&self.path);
    }
}
```

#### Example: Custom Lock Guard
You can create your own guards that behave like `MutexGuard`. This `LockGuard` uses a `Cell<bool>` to track the lock state. When the guard is created, it sets the flag to `true`. When it's dropped, it sets it back to `false`. The `Deref` and `DerefMut` traits provide ergonomic access to the inner data.

```rust
use std::ops::{Deref, DerefMut};
use std::cell::Cell;

struct MyLock<T> {
    locked: Cell<bool>,
    data: T,
}

struct LockGuard<'a, T> {
    lock: &'a MyLock<T>,
}

impl<'a, T> LockGuard<'a, T> {
    fn new(lock: &'a MyLock<T>) -> Option<Self> {
        if lock.locked.get() {
            None // Already locked
        } else {
            lock.locked.set(true);
            Some(LockGuard { lock })
        }
    }
}

impl<T> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.set(false);
    }
}

impl<T> Deref for LockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.lock.data
    }
}
```

#### Example: Panic-Safe State Restoration
A guard can be used to ensure state is restored, even in the case of a panic. This `StateGuard` sets a boolean flag to a new value on creation and restores the old value when it's dropped. This is useful for things like a "processing" flag.

```rust
struct StateGuard<'a> {
    state: &'a mut bool,
    old_value: bool,
}

impl<'a> StateGuard<'a> {
    fn new(state: &'a mut bool, new_value: bool) -> Self {
        let old_value = *state;
        *state = new_value;
        StateGuard { state, old_value }
    }
}

impl Drop for StateGuard<'_> {
    fn drop(&mut self) {
        // Restore the original state, no matter what.
        *self.state = self.old_value;
    }
}

// Usage: State is restored even if a panic occurs
fn complex_operation(processing: &mut bool) {
    let _guard = StateGuard::new(processing, true);
    // If this panics, `_guard` is dropped and `processing` is reset to its old value.
    // risky_operation(); 
}
```

#### Example: Generic Scope Guard
For arbitrary cleanup logic, a generic `ScopeGuard` can be used. It takes a closure and executes it on `drop`. This is useful for things like database transaction rollbacks. If the operation completes successfully, the guard can be `disarm`ed to prevent the cleanup from running.

```rust
struct ScopeGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    fn new(cleanup: F) -> Self {
        ScopeGuard { cleanup: Some(cleanup) }
    }

    fn disarm(mut self) {
        self.cleanup = None;
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

// Usage: Generic cleanup on scope exit
fn transactional_update() {
    println!("Starting transaction...");
    let guard = ScopeGuard::new(|| {
        println!("Rolling back transaction due to error or panic.");
    });

    // perform_operations();

    // If we get here, the operation was successful.
    println!("Committing transaction.");
    guard.disarm(); // Don't run the rollback closure.
}
```

**RAII benefits:**
- Impossible to forget cleanup
- Exception-safe (panic-safe in Rust)
- Scope-based reasoning about resources
- Composable (guards can be nested)

**Common guard patterns:**
- File handles (automatic close)
- Locks (automatic release)
- Transactions (automatic rollback)
- Metrics/timers (automatic reporting)
- State flags (automatic reset)

## Pattern 5: Memory Layout Optimization

**Problem**: Naive struct definitions waste memory through padding and hurt performance via poor cache utilization. False sharing in multi-threaded code can cause 10-100x slowdowns.

**Solution**: Use `#[repr(C)]` for predictable layout (FFI), `#[repr(align(N))]` for cache alignment, `#[repr(packed)]` to eliminate padding (with care). Order struct fields from largest to smallest alignment.

**Why It Matters**: Modern CPUs are dominated by memory hierarchy—cache misses cost 100-200 cycles while arithmetic costs 1-4 cycles. A cache miss is 50-100x slower than a cache hit.

**Use Cases**: High-frequency trading systems, game engines, scientific computing, embedded systems, FFI with C libraries, SIMD optimization, lock-free data structures.


 **What is Alignment?**
 CPUs do not read memory one byte at a time. They fetch it in chunks, typically the size of a machine word (e.g., 8 bytes on a 64-bit system). Access is fastest when a data type of size N is located at a memory address that is a multiple of N. For example, a `u64` (8 bytes) should ideally start at an address like 0, 8, 16, etc. This is its **alignment requirement**. Accessing a `u64` at an unaligned address (e.g., address 1) would be slow, as the CPU might need to perform two memory reads instead of one.

 **What is Padding?**
 To satisfy these alignment requirements, the Rust compiler may insert invisible, unused bytes into a struct. This is called **padding**. The goal is to ensure every field is properly aligned.

 There are two rules for a struct's layout:
 1. Each field must be placed at an offset that is a multiple of its alignment.
 2. The total size of the struct must be a multiple of the struct's overall alignment, which is the largest alignment of any of its fields.

### Examples

#### Example: Field Ordering to Minimize Padding
By default, Rust reorders struct fields to minimize padding, but with `#[repr(C)]` the order is fixed. Understanding the rules helps in all cases. By ordering fields from largest to smallest, you can minimize wasted space.

```rust
// In this example, we use `#[repr(C)]` to disable the automatic field
// reordering that Rust would normally perform. This lets us see the
// effects of padding manually.

// Bad: 24 bytes due to padding
#[repr(C)] 
struct Unoptimized {
    a: u8,
    b: u64,
    c: u8,
}
// How the compiler lays this out:
// - `a: u8` (size 1, align 1): offset 0.
// - 7 bytes of padding are added to align `b`.
// - `b: u64` (size 8, align 8): offset 8.
// - `c: u8` (size 1, align 1): offset 16.
// - 7 bytes of padding are added at the end to make the total size a multiple of 8.
// - Total size = 24 bytes.

// Good: 16 bytes by reordering fields
#[repr(C)]
struct Optimized {
    b: u64, // Largest alignment first
    a: u8,
    c: u8,
}
// How this improves things:
// - `b: u64`: offset 0.
// - `a: u8`: offset 8.
// - `c: u8`: offset 9.
// - 6 bytes of padding at the end makes the total size 16.
// - Total size = 16 bytes.

// Verify sizes
const _: () = assert!(std::mem::size_of::<Unoptimized>() == 24);
const _: () = assert!(std::mem::size_of::<Optimized>() == 16);
```

#### Example: Layout Attributes `#[repr(...)]`
Rust provides attributes to control memory layout.
- `#[repr(C)]`: Guarantees the same layout as a C struct. Essential for FFI.
- `#[repr(packed)]`: Removes all padding. This can lead to unaligned-access performance penalties or even crashes on some architectures. Use with extreme care.
- `#[repr(align(N))]`: Forces the struct's alignment to be at least `N` bytes.
- `#[repr(u8)]`: Specifies the memory representation for an enum's discriminant.

```rust
// For FFI compatibility
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

// To eliminate padding (use carefully!)
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,  // `b` may be at an unaligned address
}

// To align to a cache line (e.g., 64 bytes)
#[repr(align(64))]
struct CacheAligned {
    data: [u8; 64],
}

// To define an enum's size
#[repr(u8)]
enum Status {
    Idle = 0,
    Running = 1,
    Failed = 2,
}
```

#### Example: Preventing False Sharing
False sharing is a silent performance killer in multi-threaded code. It happens when two threads write to different variables that happen to live on the same CPU cache line. The CPU's cache coherency protocol forces the cores to fight over the cache line, serializing execution. The fix is to pad data to ensure contended variables are on different cache lines.

```rust
use std::sync::atomic::AtomicUsize;

const CACHE_LINE_SIZE: usize = 64;

#[repr(align(CACHE_LINE_SIZE))]
struct Padded<T> {
    value: T,
}

// With this structure, counter1 and counter2 are guaranteed to be on
// different cache lines, preventing false sharing when updated by different threads.
struct SharedCounters {
    counter1: Padded<AtomicUsize>,
    counter2: Padded<AtomicUsize>,
}
```

#### Example: Optimizing Enum Size
An enum's size is determined by its largest variant. If one variant is huge, the whole enum becomes huge. To fix this, you can `Box` the large variant. This makes the variant a pointer, and the enum's size becomes the size of the pointer plus a tag, which is much smaller.

```rust
// Bad: Size is over 1024 bytes
enum LargeEnum {
    Small(u8),
    Big([u8; 1024]),
}

// Good: Size is the size of a Box (a pointer) + a tag.
enum OptimizedEnum {
    Small(u8),
    Big(Box<[u8; 1024]>),
}
```

#### Example: Data-Oriented Design (SoA vs. AoS)
For performance-critical loops, memory access patterns are key. "Array of Structs" (AoS) is common but can be bad for cache performance if you only need one field per iteration. "Struct of Arrays" (SoA) organizes the data by field, ensuring that when you iterate over one field, all the data for that field is contiguous in memory.

```rust
// Bad: Array of Structs (AoS) - poor cache locality for single-field access
struct ParticleAoS {
    position: [f32; 3],
    velocity: [f32; 3],
    mass: f32,
}

fn update_aos(particles: &mut [ParticleAoS]) {
    for p in particles {
        // When accessing p.position, the CPU loads the entire struct (position,
        // velocity, mass) into the cache, even though we don't need the other fields.
        p.position[0] += p.velocity[0];
    }
}

// Good: Struct of Arrays (SoA) - excellent cache locality
struct ParticlesSoA {
    positions_x: Vec<f32>,
    velocities_x: Vec<f32>,
    // ... and so on for other fields
}

impl ParticlesSoA {
    fn update_positions(&mut self) {
        // All the x positions are contiguous in memory. The CPU can prefetch
        // them efficiently, leading to far fewer cache misses.
        for i in 0..self.positions_x.len() {
            self.positions_x[i] += self.velocities_x[i];
        }
    }
}
```

**Memory layout principles:**
- Order struct fields from largest to smallest alignment
- Use `#[repr(C)]` when layout matters (FFI, serialization)
- Pad to cache lines (64 bytes) to prevent false sharing
- Box large enum variants to keep enum size small
- Consider SoA over AoS for performance-critical loops

**Performance characteristics:**
- False sharing can degrade performance by 10-100x
- Proper alignment enables SIMD operations
- Cache line is typically 64 bytes
- L1 cache miss: ~4 cycles, L3 miss: ~40 cycles, RAM: ~200 cycles

## Pattern 6: Arena Allocation

*   **Problem**: Allocating many small objects with `Box::new()` or `Vec::push()` is slow. Each call invokes the system's general-purpose allocator (`malloc`), which involves locking and metadata overhead.
*   **Solution**: Use an arena allocator (also called a bump allocator). Pre-allocate a large, contiguous chunk of memory.
*   **Why It Matters**: Arena allocation is 10-100x faster than general-purpose allocators for scenarios involving many small objects. For applications like compilers (which create millions of AST nodes) or web servers (which create objects per-request), this can dramatically improve performance by reducing allocation bottlenecks.

### Examples

```rust
//================================
// Pattern: Simple arena allocator
//================================
struct Arena {
    chunks: Vec<Vec<u8>>,
    current: Vec<u8>,
    position: usize,
}

impl Arena {
    fn new() -> Self {
        Arena {
            chunks: Vec::new(),
            current: vec![0; 4096],
            position: 0,
        }
    }

    fn alloc<T>(&mut self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Align position
        let padding = (align - (self.position % align)) % align;
        self.position += padding;

        // Check if we need a new chunk
        if self.position + size > self.current.len() {
            let old = std::mem::replace(&mut self.current, vec![0; 4096]);
            self.chunks.push(old);
            self.position = 0;
        }

        // Allocate
        let ptr = &mut self.current[self.position] as *mut u8 as *mut T;
        self.position += size;

        unsafe {
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }
}

//===================================
// Use case: AST nodes during parsing
//===================================
struct AstArena {
    arena: Arena,
}

enum Expr<'a> {
    Number(i64),
    Add(&'a Expr<'a>, &'a Expr<'a>),
    Multiply(&'a Expr<'a>, &'a Expr<'a>),
}

impl AstArena {
    fn new() -> Self {
        AstArena { arena: Arena::new() }
    }

    fn number(&mut self, n: i64) -> &Expr {
        self.arena.alloc(Expr::Number(n))
    }

    fn add<'a>(&'a mut self, left: &'a Expr, right: &'a Expr) -> &'a Expr<'a> {
        self.arena.alloc(Expr::Add(left, right))
    }
}

//============================================
// Pattern: Typed arena with better ergonomics
//============================================
use typed_arena::Arena as TypedArena;

struct Parser<'ast> {
    arena: &'ast TypedArena<Expr<'ast>>,
}

impl<'ast> Parser<'ast> {
    fn parse_number(&self, n: i64) -> &'ast Expr<'ast> {
        self.arena.alloc(Expr::Number(n))
    }

    fn parse_binary(&self, left: &'ast Expr<'ast>, right: &'ast Expr<'ast>)
        -> &'ast Expr<'ast>
    {
        self.arena.alloc(Expr::Add(left, right))
    }
}

//================================================
// Pattern: Arena for temporary string allocations
//================================================
struct StringArena {
    arena: TypedArena<String>,
}

impl StringArena {
    fn new() -> Self {
        StringArena { arena: TypedArena::new() }
    }

    fn alloc(&self, s: &str) -> &str {
        let owned = self.arena.alloc(s.to_string());
        owned.as_str()
    }
}

//===================================================
// Use case: Request-scoped allocations in web server
//===================================================
struct RequestContext<'arena> {
    arena: &'arena TypedArena<Vec<u8>>,
}

impl<'arena> RequestContext<'arena> {
    fn allocate_buffer(&self, size: usize) -> &'arena mut Vec<u8> {
        self.arena.alloc(vec![0; size])
    }
}
```

**When to use arenas:**
- Compiler frontends (AST, IR nodes)
- Request handlers in servers
- Graph algorithms with temporary nodes
- Game engine frame allocations
- Any scenario with bulk deallocation

**Performance characteristics:**
- Allocation: O(1), just increment pointer
- Deallocation: O(1), drop entire arena
- 10-100x faster than malloc/free for small objects
- Better cache locality (allocated objects are contiguous)
- Cannot free individual objects (trade-off)

## Pattern 7: Custom Smart Pointers

*   **Problem**: The standard smart pointers (`Box`, `Rc`, `Arc`) are excellent general-purpose tools, but they have limitations. `Rc`/`Arc` require a separate heap allocation for their reference counts, and simple vector indices can be invalidated by insertions or removals.
*   **Solution**: Build custom smart pointers using `unsafe` Rust primitives like `NonNull<T>`, `PhantomData`, and the `Deref`, `DerefMut`, and `Drop` traits. This allows for patterns like intrusive reference counting (where the count is stored in the object itself) or generational indices (which prevent use-after-free errors with vector-like containers).
*   **Why It Matters**: Custom smart pointers unlock performance and memory layout patterns that are impossible with standard types. An intrusive `Rc` can save one allocation per object, which is critical when creating millions of them.

### Examples

#### Example: Intrusive Reference Counting
Standard `Rc` and `Arc` perform two allocations: one for the object, and one for the reference-count block. An *intrusive* counter stores the count inside the object itself, saving an allocation. This is critical when you have millions of small, reference-counted objects. This example shows a simplified intrusive `Rc`.

```rust
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::cell::Cell;
use std::ops::Deref;

// The data and its refcount live in the same heap allocation.
struct IntrusiveNode<T> {
    refcount: Cell<usize>,
    data: T,
}

struct IntrusiveRc<T> {
    ptr: NonNull<IntrusiveNode<T>>,
    _marker: PhantomData<T>,
}

impl<T> IntrusiveRc<T> {
    fn new(data: T) -> Self {
        let node = Box::new(IntrusiveNode {
            refcount: Cell::new(1),
            data,
        });
        IntrusiveRc {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(node)) },
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for IntrusiveRc<T> {
    fn clone(&self) -> Self {
        let node = unsafe { self.ptr.as_ref() };
        let count = node.refcount.get();
        node.refcount.set(count + 1);
        IntrusiveRc { ptr: self.ptr, _marker: PhantomData }
    }
}

impl<T> Drop for IntrusiveRc<T> {
    fn drop(&mut self) {
        unsafe {
            let node = self.ptr.as_ref();
            let count = node.refcount.get();
            if count == 1 {
                // Last reference, so deallocate the whole Box.
                drop(Box::from_raw(self.ptr.as_ptr()));
            } else {
                // Decrement the refcount.
                node.refcount.set(count - 1);
            }
        }
    }
}

impl<T> Deref for IntrusiveRc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &self.ptr.as_ref().data }
    }
}
```

#### Example: Generational Arena for Stable Handles
When you store objects in a `Vec`, their indices are not stable. If you remove an element from the middle, all subsequent indices change. A **generational arena** solves this. It gives you a stable `Handle` (or ID) for an object. The handle contains both an index and a "generation" number. When an object is removed, its slot is marked free, and its generation is incremented. If old code tries to use a stale handle, the generation numbers won't match, preventing use-after-free bugs. This is a cornerstone of modern Entity-Component-System (ECS) game engines.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
struct Handle {
    index: usize,
    generation: u64,
}

struct Slot<T> {
    value: Option<T>,
    generation: u64,
}

struct GenerationalArena<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<usize>,
}

impl<T> GenerationalArena<T> {
    fn new() -> Self {
        GenerationalArena { slots: Vec::new(), free_list: Vec::new() }
    }

    fn insert(&mut self, value: T) -> Handle {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index];
            slot.generation += 1;
            slot.value = Some(value);
            Handle { index, generation: slot.generation }
        } else {
            let index = self.slots.len();
            self.slots.push(Slot { value: Some(value), generation: 0 });
            Handle { index, generation: 0 }
        }
    }

    fn get(&self, handle: Handle) -> Option<&T> {
        self.slots.get(handle.index)
            .filter(|slot| slot.generation == handle.generation)
            .and_then(|slot| slot.value.as_ref())
    }

    fn remove(&mut self, handle: Handle) -> Option<T> {
        if let Some(slot) = self.slots.get_mut(handle.index) {
            if slot.generation == handle.generation {
                self.free_list.push(handle.index);
                slot.generation += 1; // Invalidate existing handles
                return slot.value.take();
            }
        }
        None
    }
}
```

#### Example: Copy-on-Write Smart Pointer
This custom `Immutable<T>` pointer makes a type immutable by default, but allows for cheap clones. Clones share the same underlying data. Only when `modify` is called does the data get copied, ensuring that modifications don't affect other copies. This is a simplified, custom version of the standard library's `Cow`.

```rust
use std::rc::Rc;
use std::ops::Deref;

struct Immutable<T: Clone> {
    data: Rc<T>,
}

impl<T: Clone> Immutable<T> {
    fn new(data: T) -> Self {
        Immutable { data: Rc::new(data) }
    }

    fn modify<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        // If the data is shared (more than one reference exists)...
        if Rc::strong_count(&self.data) > 1 {
            // ...clone it to create a new, unique copy.
            self.data = Rc::new((*self.data).clone());
        }
        // Now we have the only reference, so we can safely get a mutable one.
        let data_mut = Rc::get_mut(&mut self.data).unwrap();
        f(data_mut);
    }
}

impl<T: Clone> Deref for Immutable<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T: Clone> Clone for Immutable<T> {
    fn clone(&self) -> Self {
        // Cloning is cheap: it just clones the Rc, incrementing the ref count.
        Immutable {
            data: Rc::clone(&self.data),
        }
    }
}
```

**When to build custom smart pointers:**
- Specialized allocation patterns (pools, arenas)
- Intrusive data structures for cache efficiency
- Game engines (generational indices)
- Systems with unique ownership semantics
- Performance-critical code where std overhead matters

### Performance Summary

| Pattern | Allocation Cost | Access Cost | Best Use Case |
|---------|----------------|-------------|---------------|
| `Box<T>` | O(1) heap | O(1) | Heap allocation, trait objects |
| `Rc<T>` | O(1) heap | O(1) + refcount | Shared ownership, single-threaded |
| `Arc<T>` | O(1) heap | O(1) + atomic | Shared ownership, multi-threaded |
| `Cow<T>` | O(0) or O(n) | O(1) | Conditional cloning |
| `RefCell<T>` | O(0) | O(1) + check | Interior mutability, single-threaded |
| `Mutex<T>` | O(0) | O(lock) | Interior mutability, multi-threaded |
| Arena | O(1) bump | O(1) | Bulk allocation/deallocation |

### Common Anti-Patterns

```rust
// ❌ Holding RefCell borrow across function call
let borrowed = data.borrow();
might_borrow_again(&data);  // Runtime panic!

// ✓ Scope borrows tightly
{
    let borrowed = data.borrow();
    use_data(&borrowed);
} // Dropped here
might_borrow_again(&data);  // Safe

// ❌ Arc<Mutex<T>> when single-threaded
let shared = Arc::new(Mutex::new(data));  // Unnecessary overhead

// ✓ Use Rc<RefCell<T>> for single-threaded
let shared = Rc::new(RefCell::new(data));

// ❌ Cloning Cow unnecessarily
fn process(s: Cow<str>) -> String {
    s.into_owned()  // Always allocates
}

// ✓ Return Cow to defer cloning
fn process(s: &str) -> Cow<str> {
    if needs_modification(s) {
        Cow::Owned(modify(s))
    } else {
        Cow::Borrowed(s)
    }
}

fn needs_modification(_s: &str) -> bool { true }
fn modify(s: &str) -> String { s.to_uppercase() }
```


### Ownership and Borrowing Cheat Sheet
```rust
let s1 = String::from("hello");
let s2 = s1; // s1 moved to s2, s1 invalid
             // println!("{}", s1);                              // ERROR: s1 moved

let x = 5;
let y = x; // Copied (i32 is Copy)
println!("{}", x); // OK: x still valid

// Clone for deep copy
let s1 = String::from("hello");
let s2 = s1.clone(); // Deep copy
println!("{} {}", s1, s2); // Both valid

// ===== OWNERSHIP WITH FUNCTIONS =====
// Passing ownership to function
fn takes_ownership(s: String) {
    // s owns the String
    println!("{}", s);
} // s dropped here

let s = String::from("hello");
takes_ownership(s); // s moved
// println!("{}", s);                               // ERROR: s moved

// Return ownership from function
fn gives_ownership() -> String {
    String::from("hello") // Returns ownership
}

let s = gives_ownership(); // s owns returned String

// Taking and returning ownership
fn takes_and_gives(s: String) -> String {
    s // Return ownership
}

// ===== BORROWING (REFERENCES) =====
// Immutable borrowing
let s1 = String::from("hello");
let s2 = s1; // s1's ownership is moved to s2. s1 is no longer valid.

let x = 5;   // i32 implements the `Copy` trait.
let y = x;   // x is copied to y. Both are valid.

// ==== BORROWING ====
fn calculate_length(s: &String) -> usize { // s is a reference (a borrow).
    s.len()
} // s goes out of scope, but the data it refers to is not dropped.

fn change(s: &mut String) { // A mutable borrow.
    s.push_str(", world");
}

// ==== CLONING ====
let s1 = String::from("original");
let s2 = s1.clone(); // s2 is a deep copy of s1. Both are valid.

// ==== LIFETIMES ====
// Ensures references do not outlive the data they point to.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

