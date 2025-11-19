# Chapter 1: Memory & Ownership Patterns


[Pattern 1: Clone-on-Write (Cow)](#pattern-1-zero-copy-with-clone-on-write-cow)

- Problem: Functions face a dilemma between always cloning (wasteful) or
  awkward API design
- Solution: Use Cow<T> to defer cloning until modification is actually
  needed
- Why It Matters: Eliminates millions of allocations in high-throughput
  systems
- Use Cases: String normalization, path canonicalization, validation, HTML
  escaping

[Pattern 2: Interior Mutability (Cell/RefCell)](#pattern-2-interior-mutability-with-cell-and-refcell)

- Problem: Some designs need mutation through &self, but Rust requires
  &mut self
- Solution: Move borrow checking to runtime with Cell/RefCell
- Why It Matters: Essential for caching, graphs, and observer patterns
- Use Cases: Memoization, counters, graph structures, event systems

[Pattern 3: Thread-Safe Interior Mutability (Mutex/RwLock)](#pattern-3-thread-safe-interior-mutability-mutex-and-rwlock)

- Problem: RefCell isn't thread-safe; need shared mutable state without
  data races
- Solution: Use Mutex<T> or RwLock<T> with Arc<T>
- Why It Matters: Makes data races impossible to compile
- Use Cases: Concurrent servers, parallel algorithms, connection pools

[Pattern 4: RAII and Drop Guards](#pattern-4-raii-and-drop-guards)

- Problem: Manual cleanup is error-prone and early returns skip cleanup
- Solution: Tie resource cleanup to scope using the Drop trait
- Why It Matters: Eliminates resource leaks and enables panic-safe code
- Use Cases: File cleanup, transaction guards, lock guards, metrics

[Pattern 5: Memory Layout Optimization](#pattern-5-memory-layout-optimization)

- Problem: Naive structs waste memory and hurt cache performance
- Solution: Use #[repr] attributes, field ordering, cache alignment
- Why It Matters: Difference between 10 MB/s and 1 GB/s throughput
- Use Cases: Game engines, scientific computing, FFI, SIMD optimization

[Pattern 6: Arena Allocation](#pattern-6-arena-allocation)

- Problem: Allocating many small objects is slow; malloc has overhead
- Solution: Bump allocator that hands out pointers by incrementing a
  counter
- Why It Matters: 10-100x faster than general allocators for small objects
- Use Cases: Compilers, web servers, parsers, game engines

[Pattern 7: Custom Smart Pointers](#pattern-7-custom-smart-pointers)

- Problem: Standard smart pointers have limitations for specialized needs
- Solution: Build custom pointers with NonNull, PhantomData, Deref, Drop
- Why It Matters: Enables patterns impossible with standard types
- Use Cases: Game engines, databases, kernels, custom memory pools

## Overview

Rust's ownership system is its defining feature, enabling memory safety without garbage collection. This chapter explores advanced patterns that leverage ownership, borrowing, and lifetimes to write efficient, safe code. For experienced programmers, understanding these patterns is crucial for designing high-performance systems where memory allocation, cache locality, and zero-copy operations matter.

The ownership model enforces three fundamental rules at compile time:
1. Each value has exactly one owner
2. Values are dropped when their owner goes out of scope
3. References must never outlive their referents

These rules enable sophisticated zero-cost abstractions while preventing entire classes of bugs: use-after-free, double-free, dangling pointers, and data races.

## Type System Foundation

```rust
// Core ownership types
T                    // Owned value, moved by default
&T                   // Shared reference (immutable borrow)
&mut T               // Exclusive reference (mutable borrow)
Box<T>               // Heap-allocated owned value
Rc<T>                // Reference counted (single-threaded)
Arc<T>               // Atomic reference counted (thread-safe)
Cow<'a, T>           // Clone-on-write smart pointer

// Interior mutability (runtime borrow checking)
Cell<T>              // Copy types, no borrows
RefCell<T>           // Runtime-checked borrows, panics on violation
Mutex<T>             // Thread-safe interior mutability
RwLock<T>            // Reader-writer lock pattern
```

## Pattern 1: Zero-Copy with Clone-on-Write (Cow)

**Problem**: Functions that sometimes need to modify their input face a dilemma—always clone (wasteful when no modification is needed), always mutate in-place (requires mutable references and may surprise callers), or return different types (awkward API design).

**Solution**: Use `Cow<T>` (Clone-on-Write), an enum that's either `Borrowed(&T)` or `Owned(T)`. Check if modification is needed; if not, return borrowed data. If yes, clone and return owned data.

**Why It Matters**: Many operations don't actually need to modify their input. For example, when normalizing whitespace, if the input already has normalized whitespace, why allocate a new string? `Cow` enables zero-allocation fast paths. In high-throughput systems (web servers, parsers, validators), this pattern can eliminate millions of allocations per second.

**Use Cases**: String normalization, path canonicalization, configuration with defaults, HTML escaping, parser token extraction, validation with sanitization.

### Examples
```rust
use std::borrow::Cow;

//=======================================================================
// Pattern: Return borrowed data when possible, owned only when necessary
//=======================================================================
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

//=============================
// Pattern: Lazy mutation chain
//=============================
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

//============================================
// Pattern: to_mut() for in-place modification
//============================================
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

//======================================
// Use case: Configuration with defaults
//======================================
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

**Problem**: Rust's borrowing rules require `&mut self` for mutation, but some designs need mutation through shared references (`&self`). Examples: caching computed values, counters in shared structures, graph nodes that need to update neighbors, observer patterns.

**Solution**: Use interior mutability types—`Cell<T>` for `Copy` types (get/set without borrowing), `RefCell<T>` for non-`Copy` types (runtime-checked borrows). These move borrow checking from compile-time to runtime.

**Why It Matters**: Some data structures are impossible without interior mutability. Doubly-linked lists, graphs with cycles, and the observer pattern all require mutation through shared references. Interior mutability is also essential for caching—you want `fn get(&self, key: K)` to cache results internally without requiring `&mut self`.

**Use Cases**: Memoization and caching, incrementing counters behind `&self`, graph structures with bidirectional edges, event systems with subscriber lists, implementing trait methods that require `&self` but need internal mutation.

### Examples
```rust
use std::cell::{Cell, RefCell};

//============================================
// Pattern: Cell for Copy types (no borrowing)
//============================================
struct Counter {
    count: Cell<usize>,
}

impl Counter {
    fn increment(&self) {  // Note: takes &self, not &mut self
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> usize {
        self.count.get()
    }
}

//==============================================================
// Pattern: RefCell for non-Copy types (runtime borrow checking)
//==============================================================
use std::collections::HashMap;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn get_or_compute(&self, key: &str, compute: impl FnOnce() -> String) -> String {
        // Try to get from cache
        if let Some(value) = self.data.borrow().get(key) {
            return value.clone();
        }

        // Not found, compute and insert
        let value = compute();
        self.data.borrow_mut().insert(key.to_string(), value.clone());
        value
    }
}

//===========================================================
// Pattern: Multiple borrows in single scope (borrow scoping)
//===========================================================
fn process_cache(cache: &RefCell<Vec<String>>) {
    // Read operation
    {
        let borrowed = cache.borrow();
        println!("Cache size: {}", borrowed.len());
    } // borrow dropped here

    // Write operation (would panic if borrow still held)
    cache.borrow_mut().push("new_item".to_string());
}

//==============================================
// Pattern: try_borrow for safe runtime checking
//==============================================
fn safe_access(data: &RefCell<Vec<i32>>) -> Result<(), &'static str> {
    if let Ok(mut borrowed) = data.try_borrow_mut() {
        borrowed.push(42);
        Ok(())
    } else {
        Err("Already borrowed")
    }
}

//==============================================
// Use case: Graph with bidirectional references
//==============================================
use std::rc::Rc;

struct Node {
    value: i32,
    edges: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn add_edge(&self, target: Rc<Node>) {
        self.edges.borrow_mut().push(target);
    }

    fn neighbors(&self) -> Vec<Rc<Node>> {
        self.edges.borrow().clone()
    }
}
```

**When to use Cell:**
- Counters, flags, primitive state in shared structures
- No need to borrow the value, only get/set
- Always Copy types (usize, bool, etc.)

**When to use RefCell:**
- Mutable collections behind shared references
- Caching and memoization
- Graph structures with cycles
- Event systems with subscriber lists

**Critical safety note:**
- RefCell panics if borrow rules violated at runtime
- Never hold borrows across unknown code boundaries
- Use try_borrow for fallible operations

## Pattern 3: Thread-Safe Interior Mutability (Mutex & RwLock)

**Problem**: `RefCell<T>` provides interior mutability but panics if used incorrectly across threads. Multi-threaded code needs safe shared mutable state—incrementing counters, updating caches, modifying shared collections—without data races.

**Solution**: Use `Mutex<T>` for exclusive access (like `RefCell` but thread-safe) or `RwLock<T>` for reader-writer patterns (multiple readers OR one writer). Combine with `Arc<T>` to share across threads. These use atomic operations and OS primitives for synchronization.

**Why It Matters**: Multi-threaded programming without data races is notoriously difficult in C/C++. Rust's type system makes it impossible to compile racy code—you must use `Mutex` or `RwLock` for shared mutation. Understanding these patterns is essential for writing concurrent servers, parallel algorithms, and high-performance applications.

**Use Cases**: Shared counters in multi-threaded servers, concurrent caches, thread pools with shared work queues, parallel data processing with result aggregation, connection pools.

### Examples

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

//=============================================
// Pattern: Shared mutable state across threads
//=============================================
fn parallel_counter() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter.lock().unwrap();
                *num += 1;
            } // lock automatically released when guard dropped
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}

//=========================================
// Pattern: RwLock for read-heavy workloads
//=========================================
struct SharedCache {
    data: RwLock<HashMap<String, String>>,
}

impl SharedCache {
    fn get(&self, key: &str) -> Option<String> {
        // Multiple readers can hold read locks simultaneously
        self.data.read().unwrap().get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        // Write lock is exclusive
        self.data.write().unwrap().insert(key, value);
    }

    fn update<F>(&self, key: &str, f: F)
    where
        F: FnOnce(&str) -> String
    {
        let mut cache = self.data.write().unwrap();
        if let Some(old_value) = cache.get(key) {
            let new_value = f(old_value);
            cache.insert(key.to_string(), new_value);
        }
    }
}

//=============================
// Pattern: Minimize lock scope
//=============================
fn optimized_update(shared: &Mutex<Vec<i32>>, new_value: i32) {
    // Bad: hold lock during computation
    // let mut data = shared.lock().unwrap();
    // let computed = expensive_computation(new_value);
    // data.push(computed);

    // Good: compute outside lock
    let computed = expensive_computation(new_value);
    shared.lock().unwrap().push(computed);
}

fn expensive_computation(x: i32) -> i32 {
    x * 2  // Imagine this is expensive
}

//================================================
// Pattern: Deadlock prevention with lock ordering
//================================================
struct Account {
    balance: Mutex<i64>,
}

fn transfer(from: &Account, to: &Account, amount: i64) {
    // Deadlock possible if two threads call transfer(a, b) and transfer(b, a)
    // Solution: acquire locks in consistent order

    let (first, second) = if from as *const _ < to as *const _ {
        (from, to)
    } else {
        (to, from)
    };

    let mut first_balance = first.balance.lock().unwrap();
    let mut second_balance = second.balance.lock().unwrap();

    if from as *const _ < to as *const _ {
        *first_balance -= amount;
        *second_balance += amount;
    } else {
        *second_balance += amount;
        *first_balance -= amount;
    }
}

//==========================================
// Pattern: try_lock for non-blocking access
//==========================================
fn try_update(data: &Mutex<Vec<i32>>) -> Result<(), &'static str> {
    if let Ok(mut guard) = data.try_lock() {
        guard.push(42);
        Ok(())
    } else {
        Err("Lock held by another thread")
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

## Pattern 4: RAII and Custom Drop Guards

**Problem**: Manual resource cleanup is error-prone. Forgetting to close files, release locks, or rollback transactions causes resource leaks, deadlocks, and data corruption. Even with discipline, early returns and panics can skip cleanup code.

**Solution**: Implement the `Drop` trait to tie resource cleanup to scope. Create guard types that acquire resources in their constructor and release them in `Drop`. Rust guarantees `Drop` runs when the value goes out of scope, even during panics.

**Why It Matters**: RAII eliminates entire categories of bugs. You cannot forget to unlock a `Mutex`—`MutexGuard`'s `Drop` releases it automatically. Temporary files are always deleted. Transactions always rollback on error. This pattern is fundamental to Rust's safety guarantees and enables panic-safe code.

**Use Cases**: Temporary file management, database transaction guards, lock guards (mutex, RwLock), metrics timers, state flag restoration, scope-based profiling, connection cleanup in pools.

### Examples

```rust
use std::fs::File;
use std::io::Write;

//==================================
// Pattern: Custom guard for cleanup
//==================================
struct TempFile {
    path: String,
    file: File,
}

impl TempFile {
    fn new(path: String) -> std::io::Result<Self> {
        let file = File::create(&path)?;
        Ok(TempFile { path, file })
    }

    fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.file.write_all(data)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // Cleanup happens automatically when TempFile goes out of scope
        let _ = std::fs::remove_file(&self.path);
    }
}

//=======================================
// Pattern: MutexGuard-like custom guards
//=======================================
struct LockGuard<'a, T> {
    data: &'a mut T,
    locked: &'a Cell<bool>,
}

impl<'a, T> LockGuard<'a, T> {
    fn new(data: &'a mut T, locked: &'a Cell<bool>) -> Option<Self> {
        if locked.get() {
            None
        } else {
            locked.set(true);
            Some(LockGuard { data, locked })
        }
    }
}

impl<T> std::ops::Deref for LockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> std::ops::DerefMut for LockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        self.locked.set(false);
    }
}

//======================================
// Pattern: Panic-safe state restoration
//======================================
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
        *self.state = self.old_value;
    }
}

//===========================================
// Usage: State restored even if panic occurs
//===========================================
fn complex_operation(processing: &mut bool) {
    let _guard = StateGuard::new(processing, true);
    // If this panics, processing is reset to old value
    risky_operation();
}

fn risky_operation() {
    // Might panic
}

//===========================================
// Pattern: Scope guard for arbitrary cleanup
//===========================================
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

//=====================================
// Usage: Generic cleanup on scope exit
//=====================================
fn transactional_update() {
    let _guard = ScopeGuard::new(|| {
        println!("Rolling back transaction");
        rollback();
    });

    perform_operations();

    // Commit succeeded, don't rollback
    _guard.disarm();
}

fn rollback() {}
fn perform_operations() {}
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

**Problem**: Naive struct definitions waste memory through padding and hurt performance via poor cache utilization. False sharing in multi-threaded code can cause 10-100x slowdowns. Struct of Arrays (SoA) vs Array of Structs (AoS) choice dramatically affects loop performance.

**Solution**: Use `#[repr(C)]` for predictable layout (FFI), `#[repr(align(N))]` for cache alignment, `#[repr(packed)]` to eliminate padding (with care). Order struct fields from largest to smallest alignment. Pad shared data to cache line boundaries (64 bytes). Consider SoA for performance-critical loops.

**Why It Matters**: Modern CPUs are dominated by memory hierarchy—cache misses cost 100-200 cycles while arithmetic costs 1-4 cycles. A cache miss is 50-100x slower than a cache hit. False sharing (two threads modifying different variables on the same cache line) serializes supposedly-parallel code. Understanding memory layout is the difference between 10 MB/s and 1 GB/s in data processing.

**Use Cases**: High-frequency trading systems, game engines, scientific computing, embedded systems, FFI with C libraries, SIMD optimization, lock-free data structures.

### Examples

```rust
//==========================================
// Pattern: #[repr(C)] for FFI compatibility
//==========================================
#[repr(C)]
struct Point {
    x: f64,
    y: f64,
}

//==============================================
// Pattern: #[repr(packed)] to eliminate padding
//==============================================
// WARNING: Can cause misaligned access, use carefully
#[repr(packed)]
struct Packed {
    a: u8,
    b: u32,  // No padding between a and b
}

//========================================================
// Pattern: Explicit alignment for cache line optimization
//========================================================
#[repr(align(64))]  // Cache line size on most systems
struct CacheAligned {
    data: [u8; 64],
}

//======================================================
// Pattern: Prevent false sharing in multi-threaded code
//======================================================
#[repr(align(64))]
struct Padded<T> {
    value: T,
}

struct SharedCounters {
    counter1: Padded<AtomicUsize>,  // Separate cache lines
    counter2: Padded<AtomicUsize>,  // Prevents false sharing
}

use std::sync::atomic::{AtomicUsize, Ordering};

//================================================
// Pattern: Field ordering to minimize struct size
//================================================
// Bad: 24 bytes due to padding
struct Unoptimized {
    a: u8,     // 1 byte + 7 padding
    b: u64,    // 8 bytes
    c: u8,     // 1 byte + 7 padding
}

// Good: 16 bytes (optimal packing)
struct Optimized {
    b: u64,    // 8 bytes
    a: u8,     // 1 byte
    c: u8,     // 1 byte + 6 padding
}

// Verify sizes
const _: () = assert!(std::mem::size_of::<Optimized>() == 16);

//================================
// Pattern: Enum size optimization
//================================
// Bad: Size determined by largest variant
enum Large {
    Small(u8),
    Big([u8; 1024]),  // Makes entire enum 1024+ bytes
}

// Good: Box large variants
enum Optimized {
    Small(u8),
    Big(Box<[u8; 1024]>),  // Enum size = max(sizeof(u8), sizeof(Box))
}

//=================================================
// Pattern: Manual discriminant for C compatibility
//=================================================
#[repr(u8)]
enum Status {
    Idle = 0,
    Running = 1,
    Failed = 2,
}

//=================================================
// Pattern: Zero-sized types for type-state pattern
//=================================================
struct Locked;
struct Unlocked;

struct Resource<State> {
    data: Vec<u8>,
    _state: PhantomData<State>,
}

use std::marker::PhantomData;

impl Resource<Unlocked> {
    fn lock(self) -> Resource<Locked> {
        Resource {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Resource<Locked> {
    fn unlock(self) -> Resource<Unlocked> {
        Resource {
            data: self.data,
            _state: PhantomData,
        }
    }

    fn modify(&mut self) {
        // Only available when locked
        self.data.push(42);
    }
}

//===================================================
// Pattern: Data-oriented design for cache efficiency
//===================================================
// Bad: Array of structs (AoS) - poor cache locality
struct ParticleAoS {
    position: [f32; 3],
    velocity: [f32; 3],
    mass: f32,
}

fn update_aos(particles: &mut [ParticleAoS]) {
    for p in particles {
        // Accessing position requires loading entire struct
        p.position[0] += p.velocity[0];
        p.position[1] += p.velocity[1];
        p.position[2] += p.velocity[2];
    }
}

// Good: Struct of arrays (SoA) - excellent cache locality
struct ParticlesSoA {
    positions_x: Vec<f32>,
    positions_y: Vec<f32>,
    positions_z: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    velocities_z: Vec<f32>,
}

impl ParticlesSoA {
    fn update(&mut self) {
        // All x positions contiguous in memory - cache friendly
        for i in 0..self.positions_x.len() {
            self.positions_x[i] += self.velocities_x[i];
            self.positions_y[i] += self.velocities_y[i];
            self.positions_z[i] += self.velocities_z[i];
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

**Problem**: Allocating many small objects with `Box::new()` or `Vec::push()` is slow—each allocation calls into the system allocator (`malloc`), which involves locks and metadata management. Individually freeing objects is even slower. Compilers and parsers allocate millions of AST nodes; web servers create objects per request.

**Solution**: Arena allocation (bump allocation)—pre-allocate a large memory chunk and hand out pointers by incrementing a position counter. Deallocation is a no-op for individual objects; the entire arena is freed at once when dropped. This reduces allocation from a complex operation to a pointer increment.

**Why It Matters**: Arena allocation is 10-100x faster than general-purpose allocators for small objects. For compilers, this means parsing is allocation-limited—arena allocation can halve compile times. For web servers handling 10,000 requests/second, per-request arenas eliminate allocation overhead entirely.

**Use Cases**: Compiler frontends (AST, IR, symbol tables), web server request handlers, game engine frame allocations, graph algorithms with temporary structures, template engines, parsers and lexers.

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

**Problem**: Standard smart pointers (`Box`, `Rc`, `Arc`) have limitations—`Rc`/`Arc` use separate heap allocations for refcounts, indices into growing vectors can be invalidated, and some patterns need intrusive reference counting for cache efficiency. Game engines, databases, and kernels need specialized ownership semantics.

**Solution**: Build custom smart pointers using `NonNull<T>`, `PhantomData`, `Deref`, `DerefMut`, and `Drop`. Intrusive reference counting embeds refcounts in the object itself. Generational indices combine indices with generation counters to detect stale references. Copy-on-write wrappers enforce immutability.

**Why It Matters**: Custom smart pointers enable patterns impossible with standard types. Intrusive `Rc` saves one allocation per object (critical for millions of small objects). Generational arenas let you use stable indices instead of pointers, simplifying serialization and debugging. Understanding these techniques is essential for high-performance systems programming.

**Use Cases**: Game engines (entity-component systems with generational indices), database systems (buffer pool management), embedded systems (intrusive data structures for minimal overhead), kernel development, custom memory pools.

### Examples

```rust
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::marker::PhantomData;

//==========================================================
// Pattern: Intrusive reference counting (like Linux kernel)
//==========================================================
struct IntrusiveRc<T> {
    ptr: NonNull<IntrusiveNode<T>>,
    _marker: PhantomData<T>,
}

struct IntrusiveNode<T> {
    refcount: Cell<usize>,
    data: T,
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
        node.refcount.set(node.refcount.get() + 1);
        IntrusiveRc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for IntrusiveRc<T> {
    fn drop(&mut self) {
        unsafe {
            let node = self.ptr.as_ref();
            let count = node.refcount.get();
            if count == 1 {
                drop(Box::from_raw(self.ptr.as_ptr()));
            } else {
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

//============================================================
// Pattern: Copy-on-write smart pointer (immutable by default)
//============================================================
struct Immutable<T: Clone> {
    data: Rc<T>,
}

impl<T: Clone> Immutable<T> {
    fn new(data: T) -> Self {
        Immutable { data: Rc::new(data) }
    }

    fn modify<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T)
    {
        // Clone if shared
        if Rc::strong_count(&self.data) > 1 {
            self.data = Rc::new((*self.data).clone());
        }

        // Safe because we have unique ownership
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
        Immutable { data: Rc::clone(&self.data) }
    }
}

//====================================================================
// Pattern: Owning handle with generation counter (for stable indices)
//====================================================================
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
        GenerationalArena {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    fn insert(&mut self, value: T) -> Handle {
        if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index];
            slot.generation += 1;
            slot.value = Some(value);
            Handle { index, generation: slot.generation }
        } else {
            let index = self.slots.len();
            self.slots.push(Slot {
                value: Some(value),
                generation: 0,
            });
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
                return slot.value.take();
            }
        }
        None
    }
}

use std::rc::Rc;
```

**When to build custom smart pointers:**
- Specialized allocation patterns (pools, arenas)
- Intrusive data structures for cache efficiency
- Game engines (generational indices)
- Systems with unique ownership semantics
- Performance-critical code where std overhead matters

## Performance Summary

| Pattern | Allocation Cost | Access Cost | Best Use Case |
|---------|----------------|-------------|---------------|
| `Box<T>` | O(1) heap | O(1) | Heap allocation, trait objects |
| `Rc<T>` | O(1) heap | O(1) + refcount | Shared ownership, single-threaded |
| `Arc<T>` | O(1) heap | O(1) + atomic | Shared ownership, multi-threaded |
| `Cow<T>` | O(0) or O(n) | O(1) | Conditional cloning |
| `RefCell<T>` | O(0) | O(1) + check | Interior mutability, single-threaded |
| `Mutex<T>` | O(0) | O(lock) | Interior mutability, multi-threaded |
| Arena | O(1) bump | O(1) | Bulk allocation/deallocation |

## Quick Reference: Choosing Ownership Patterns

```rust
// Need to modify through shared reference, single-threaded?
Cell<T>        // For Copy types
RefCell<T>     // For non-Copy types

// Need to modify through shared reference, multi-threaded?
Mutex<T>       // Exclusive access
RwLock<T>      // Read-heavy workloads

// Need shared ownership?
Rc<T>          // Single-threaded
Arc<T>         // Multi-threaded

// Conditional cloning?
Cow<'a, T>     // Return borrowed when possible

// Automatic cleanup?
Drop trait     // Custom RAII guards

// Fast allocation with bulk deallocation?
Arena          // Bump allocator

// Memory layout matters?
#[repr(C)]     // FFI compatibility
#[repr(align(N))] // Cache alignment
```

## Common Anti-Patterns

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

## Key Takeaways

1. **Ownership is zero-cost**: Compile-time enforcement means no runtime overhead
2. **Interior mutability is a tool, not a default**: Use sparingly, prefer immutability
3. **Cow enables zero-copy APIs**: Essential for library design
4. **Lock scope matters**: Minimize critical sections for concurrency
5. **RAII eliminates cleanup bugs**: Resources tied to scope
6. **Memory layout affects performance**: Consider cache lines and alignment
7. **Arenas are fast for bulk allocation**: Trade flexibility for speed
8. **Profile before optimizing**: Measure, don't guess

Understanding these patterns transforms Rust from "fighting the borrow checker" to leveraging one of the most sophisticated memory management systems in any programming language.
