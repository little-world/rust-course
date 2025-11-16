# Chapter 1: Memory & Ownership Patterns

## Overview

Rust's ownership system is its defining feature, enabling memory safety without garbage collection. This chapter explores advanced patterns that leverage ownership, borrowing, and lifetimes to write efficient, safe code. For experienced programmers, understanding these patterns is crucial for designing high-performance systems where memory allocation, cache locality, and zero-copy operations matter.

The ownership model enforces three fundamental rules at compile time:
1. Each value has exactly one owner
2. Values are dropped when their owner goes out of scope
3. References must never outlive their referents

These rules enable sophisticated zero-cost abstractions while preventing entire classes of bugs: use-after-free, double-free, dangling pointers, and data races.

## Type System Foundation

```rust
//=====================
// Core ownership types
//=====================
T                    // Owned value, moved by default
&T                   // Shared reference (immutable borrow)
&mut T               // Exclusive reference (mutable borrow)
Box<T>               // Heap-allocated owned value
Rc<T>                // Reference counted (single-threaded)
Arc<T>               // Atomic reference counted (thread-safe)
Cow<'a, T>           // Clone-on-write smart pointer

//==============================================
// Interior mutability (runtime borrow checking)
//==============================================
Cell<T>              // Copy types, no borrows
RefCell<T>           // Runtime-checked borrows, panics on violation
Mutex<T>             // Thread-safe interior mutability
RwLock<T>            // Reader-writer lock pattern
```

## Pattern 1: Zero-Copy with Clone-on-Write (Cow)

The `Cow` (Clone-on-Write) type is a smart pointer that defers cloning until mutation is required. This pattern is essential for APIs that sometimes need to modify data but usually don't, avoiding unnecessary allocations.

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

Rust's borrowing rules are enforced at compile time, but sometimes you need runtime flexibility. Interior mutability allows mutation through shared references, moving borrow checking to runtime.

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

For multi-threaded programs, `Mutex` and `RwLock` provide interior mutability with thread-safety guarantees. These types use atomic operations and OS synchronization primitives.

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

RAII (Resource Acquisition Is Initialization) ties resource lifetime to scope. Rust's `Drop` trait enables automatic cleanup, making resource management elegant and safe.

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

// Usage: State restored even if panic occurs
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

// Usage: Generic cleanup on scope exit
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

Understanding memory layout is crucial for performance. Cache efficiency, false sharing, and struct padding significantly impact real-world performance.

### Memory Alignment Fundamentals

**Alignment** is the requirement that a value's memory address must be a multiple of its alignment value. Every type in Rust has an alignment requirement, which is a power of 2 (1, 2, 4, 8, 16, etc.).

**Why alignment matters:**
1. **Hardware requirement**: Many CPUs cannot access misaligned data, causing faults
2. **Performance**: Aligned access is faster (1 cycle vs multiple cycles for misaligned)
3. **Atomics**: Atomic operations require proper alignment to work correctly
4. **SIMD**: Vector operations require 16/32/64-byte alignment

**Alignment rules:**
- `u8`/`i8`/`bool`: 1-byte alignment (can be at any address)
- `u16`/`i16`: 2-byte alignment (address must be multiple of 2)
- `u32`/`i32`/`f32`: 4-byte alignment (address must be multiple of 4)
- `u64`/`i64`/`f64`/pointers: 8-byte alignment (address must be multiple of 8)
- `u128`/`i128`: 16-byte alignment
- Structs: alignment = max(alignment of all fields)
- Arrays: alignment = element alignment

**Padding** is unused space inserted by the compiler to satisfy alignment requirements:

```rust
//==============================================
// Understanding padding with detailed breakdown
//==============================================

// Example 1: Padding between fields
struct Example1 {
    a: u8,     // Offset 0, size 1, align 1
    // [padding: 3 bytes inserted here]
    b: u32,    // Offset 4, size 4, align 4
    c: u8,     // Offset 8, size 1, align 1
    // [padding: 3 bytes at end for alignment]
}
// Total size: 12 bytes (not 6!)
// Alignment: 4 (max of field alignments)

// Why 12 bytes?
// - 'a' at offset 0 (1 byte)
// - 'b' needs 4-byte alignment, so offset must be multiple of 4
//   Next valid offset is 4, so 3 padding bytes inserted
// - 'b' at offset 4 (4 bytes)
// - 'c' at offset 8 (1 byte)
// - Struct must be aligned to 4 bytes for array/packing
//   Size must be multiple of alignment, so 3 padding bytes at end

// Example 2: Optimal field ordering
struct Example2 {
    b: u32,    // Offset 0, size 4, align 4
    a: u8,     // Offset 4, size 1, align 1
    c: u8,     // Offset 5, size 1, align 1
    // [padding: 2 bytes at end]
}
// Total size: 8 bytes (33% smaller!)
// Same data, better layout

// Example 3: No padding needed
struct Example3 {
    x: u64,    // Offset 0, size 8, align 8
    y: u32,    // Offset 8, size 4, align 4
    z: u32,    // Offset 12, size 4, align 4
}
// Total size: 16 bytes, perfectly packed

// Example 4: Nested structs
struct Inner {
    a: u8,
    // [3 bytes padding]
    b: u32,
}  // size: 8, align: 4

struct Outer {
    x: u8,
    // [3 bytes padding]
    inner: Inner,  // Needs 4-byte alignment
    y: u8,
    // [3 bytes padding]
}  // size: 16, align: 4

//===============================================
// Inspecting alignment and size at compile time
//===============================================
const _: () = {
    assert!(std::mem::size_of::<Example1>() == 12);
    assert!(std::mem::align_of::<Example1>() == 4);

    assert!(std::mem::size_of::<Example2>() == 8);
    assert!(std::mem::align_of::<Example2>() == 4);
};

// Runtime inspection
fn inspect_layout<T>() {
    println!("Size: {} bytes", std::mem::size_of::<T>());
    println!("Alignment: {} bytes", std::mem::align_of::<T>());
}

//==========================================
// Pattern: #[repr(C)] for FFI compatibility
//==========================================
// Rust's default representation is undefined and may reorder fields
// #[repr(C)] guarantees C-compatible layout (no reordering)

#[repr(C)]
struct Point {
    x: f64,    // Guaranteed at offset 0
    y: f64,    // Guaranteed at offset 8
}
// Size: 16, align: 8
// Layout is stable and matches C struct

// Without #[repr(C)], Rust could theoretically reorder fields
struct NotC {
    x: f64,
    y: f64,
}
// Size: 16, align: 8 (happens to match, but not guaranteed)

//==============================================
// Pattern: #[repr(packed)] to eliminate padding
//==============================================
// WARNING: Removes ALL padding, can cause misaligned access
// Can break on architectures that require alignment
// References to fields may be UB if misaligned

#[repr(packed)]
struct Packed {
    a: u8,     // Offset 0
    b: u32,    // Offset 1 (misaligned! normally needs offset 4)
}
// Size: 5 bytes, align: 1

// Safer: specify alignment while packing
#[repr(packed(2))]
struct PackedAlign2 {
    a: u8,     // Offset 0
    b: u32,    // Offset 2 (2-byte aligned, but not 4-byte)
    c: u8,     // Offset 6
}
// Size: 7, align: 2

// Danger: Taking references to packed struct fields
#[repr(packed)]
struct DangerousPacked {
    a: u8,
    b: u64,
}

fn unsafe_example() {
    let packed = DangerousPacked { a: 1, b: 2 };

    // ⚠️  UB: Creating reference to misaligned field
    // let ref_b = &packed.b;  // DON'T DO THIS!

    // ✓ Safe: Read by value
    let value = packed.b;  // Compiler generates safe unaligned load

    // ✓ Safe: Use ptr::read_unaligned
    let value = unsafe {
        std::ptr::addr_of!(packed.b).read_unaligned()
    };
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

Arena allocators (also called bump allocators or region allocators) provide fast allocation by pre-allocating a large chunk of memory and handing out pointers sequentially. Deallocation happens all at once when the arena is dropped.

### Arena Allocation Fundamentals

**What is an arena allocator?**
An arena is a memory management strategy that:
1. Pre-allocates a large contiguous chunk of memory
2. Satisfies allocation requests by "bumping" a pointer forward
3. Never frees individual allocations
4. Frees all allocations at once when the arena is destroyed

**Why use arenas?**
- **Speed**: 10-100x faster than malloc/free for small objects (just pointer increment)
- **Cache locality**: Allocated objects are contiguous in memory
- **No fragmentation**: Linear allocation prevents memory fragmentation
- **Predictable performance**: No allocation algorithm complexity
- **Bulk deallocation**: O(1) cleanup regardless of allocation count

**Trade-offs:**
- Cannot free individual allocations (memory grows monotonically)
- Holds memory until entire arena is dropped
- Not suitable for long-lived objects with mixed lifetimes
- Requires lifetime discipline (arena must outlive all allocations)

**Memory layout:**
```
Arena memory chunk:
┌────────────────────────────────────────────────────────┐
│ [Obj1][Obj2][padding][Obj3][Obj4]    [free space]      │
│  ^                                     ^               │
│  allocated                             bump pointer    │
└────────────────────────────────────────────────────────┘

Allocation process:
1. Check if enough space: (position + size) <= capacity
2. Align position to object's alignment requirement
3. Return pointer at current position
4. Bump position forward by object size
```

### How Alignment Works in Arenas

When allocating, the arena must ensure each object is properly aligned:

```rust
//==============================================
// Alignment calculation in detail
//==============================================

// Example: Allocating u8, then u32
// Initial state: position = 0
//
// Allocate u8 (size=1, align=1):
//   - position % 1 == 0, no padding needed
//   - place u8 at offset 0
//   - position = 0 + 1 = 1
//
// Allocate u32 (size=4, align=4):
//   - position % 4 == 1, needs alignment
//   - padding = (4 - (1 % 4)) % 4 = 3
//   - skip 3 bytes, position = 1 + 3 = 4
//   - place u32 at offset 4 (aligned!)
//   - position = 4 + 4 = 8

fn calculate_padding(position: usize, align: usize) -> usize {
    // Formula: padding = (align - (position % align)) % align
    // This gives the number of bytes to skip to reach next aligned address

    let remainder = position % align;
    if remainder == 0 {
        0  // Already aligned
    } else {
        align - remainder  // Skip to next alignment boundary
    }
}

// Alternative using bit manipulation (faster for power-of-2 alignments)
fn calculate_padding_fast(position: usize, align: usize) -> usize {
    // align must be power of 2
    debug_assert!(align.is_power_of_two());
    (align - (position & (align - 1))) & (align - 1)
}

//==============================================
// Visualizing alignment padding
//==============================================

/*
Memory addresses:  0   1   2   3   4   5   6   7   8
                  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
After u8:         │ X │   │   │   │   │   │   │   │   │
                  └───┴───┴───┴───┴───┴───┴───┴───┴───┘
                   ↑
                   position=1, need align=4 for u32

                  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
Add 3 padding:    │ X │ P │ P │ P │   │   │   │   │   │
                  └───┴───┴───┴───┴───┴───┴───┴───┴───┘
                                   ↑
                                   position=4 (aligned!)

                  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐
After u32:        │ X │ P │ P │ P │ Y │ Y │ Y │ Y │   │
                  └───┴───┴───┴───┴───┴───┴───┴───┴───┘
                                                   ↑
                                                   position=8
*/
```

### Simple Arena Implementation

```rust
//================================
// Pattern: Simple arena allocator
//================================
struct Arena {
    chunks: Vec<Vec<u8>>,  // Previously filled chunks
    current: Vec<u8>,       // Current allocation chunk
    position: usize,        // Next allocation offset in current chunk
}

impl Arena {
    fn new() -> Self {
        Arena {
            chunks: Vec::new(),
            current: vec![0; 4096],  // 4KB initial chunk
            position: 0,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Arena {
            chunks: Vec::new(),
            current: vec![0; capacity],
            position: 0,
        }
    }

    fn alloc<T>(&mut self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Step 1: Calculate padding for alignment
        let padding = (align - (self.position % align)) % align;
        self.position += padding;

        // Step 2: Check if we need a new chunk
        if self.position + size > self.current.len() {
            // Save current chunk and allocate new one
            let old = std::mem::replace(
                &mut self.current,
                vec![0; self.current.len().max(size * 2)]  // Grow if needed
            );
            self.chunks.push(old);
            self.position = 0;
        }

        // Step 3: Get pointer at current position
        let ptr = &mut self.current[self.position] as *mut u8 as *mut T;
        self.position += size;

        // Step 4: Write value and return reference
        unsafe {
            std::ptr::write(ptr, value);  // Move value into arena
            &mut *ptr                      // Return mutable reference
        }
    }

    // Allocate uninitialized space (useful for buffers)
    fn alloc_slice<T>(&mut self, len: usize) -> &mut [T] {
        let size = std::mem::size_of::<T>() * len;
        let align = std::mem::align_of::<T>();

        let padding = (align - (self.position % align)) % align;
        self.position += padding;

        if self.position + size > self.current.len() {
            let old = std::mem::replace(
                &mut self.current,
                vec![0; self.current.len().max(size * 2)]
            );
            self.chunks.push(old);
            self.position = 0;
        }

        let ptr = &mut self.current[self.position] as *mut u8 as *mut T;
        self.position += size;

        unsafe {
            std::slice::from_raw_parts_mut(ptr, len)
        }
    }

    // Reset arena without deallocating (reuse memory)
    fn reset(&mut self) {
        self.position = 0;
        // Keep first chunk, drop others
        if let Some(chunk) = self.chunks.pop() {
            self.current = chunk;
            self.chunks.clear();
        }
    }

    // Get statistics
    fn allocated_bytes(&self) -> usize {
        let chunk_total: usize = self.chunks.iter().map(|c| c.len()).sum();
        chunk_total + self.position
    }

    fn capacity_bytes(&self) -> usize {
        let chunk_total: usize = self.chunks.iter().map(|c| c.len()).sum();
        chunk_total + self.current.len()
    }

    fn wasted_bytes(&self) -> usize {
        self.capacity_bytes() - self.allocated_bytes()
    }
}

// Example usage showing performance characteristics
fn arena_example() {
    let mut arena = Arena::new();

    // Allocation is just pointer bumping - extremely fast
    let x = arena.alloc(42u32);
    let y = arena.alloc(3.14f64);
    let z = arena.alloc([1, 2, 3, 4, 5]);

    // All allocations are contiguous in memory
    println!("x: {:p}, y: {:p}, z: {:p}", x, y, z);

    // Statistics
    println!("Allocated: {} bytes", arena.allocated_bytes());
    println!("Capacity: {} bytes", arena.capacity_bytes());
    println!("Wasted: {} bytes", arena.wasted_bytes());

    // All memory freed at once when arena drops - O(1)
}

//==============================================
// Pattern: Arena with Drop support
//==============================================
// Problem: Simple arena doesn't call destructors
// Solution: Track allocated objects and drop them

struct DroppingArena {
    arena: Arena,
    drops: Vec<Box<dyn FnOnce()>>,  // Cleanup functions
}

impl DroppingArena {
    fn new() -> Self {
        DroppingArena {
            arena: Arena::new(),
            drops: Vec::new(),
        }
    }

    fn alloc<T>(&mut self, value: T) -> &mut T {
        let ptr = self.arena.alloc(value);

        // Register destructor if T needs Drop
        if std::mem::needs_drop::<T>() {
            let ptr_copy = ptr as *mut T;
            self.drops.push(Box::new(move || unsafe {
                std::ptr::drop_in_place(ptr_copy);
            }));
        }

        ptr
    }
}

impl Drop for DroppingArena {
    fn drop(&mut self) {
        // Call all destructors in reverse order
        for drop_fn in self.drops.drain(..).rev() {
            drop_fn();
        }
    }
}

//===================================
// Use case: AST nodes during parsing
//===================================
// Problem: Recursive data structures like ASTs need many small allocations
// Solution: Arena allocation is perfect - all nodes freed when parsing completes

struct AstArena {
    arena: Arena,
}

enum Expr<'a> {
    Number(i64),
    Variable(&'a str),
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

    fn variable(&mut self, name: &str) -> &Expr {
        // Arena-allocate the string too
        let name_ref = {
            let buf = self.arena.alloc_slice(name.len());
            buf.copy_from_slice(name.as_bytes());
            unsafe { std::str::from_utf8_unchecked(buf) }
        };
        self.arena.alloc(Expr::Variable(name_ref))
    }

    fn add<'a>(&'a mut self, left: &'a Expr, right: &'a Expr) -> &'a Expr<'a> {
        self.arena.alloc(Expr::Add(left, right))
    }

    fn multiply<'a>(&'a mut self, left: &'a Expr, right: &'a Expr) -> &'a Expr<'a> {
        self.arena.alloc(Expr::Multiply(left, right))
    }
}

// Example: Building an expression tree
fn parse_example() {
    let mut arena = AstArena::new();

    // Build: (x + 5) * (y + 3)
    let x = arena.variable("x");
    let five = arena.number(5);
    let y = arena.variable("y");
    let three = arena.number(3);

    let left = arena.add(x, five);      // x + 5
    let right = arena.add(y, three);    // y + 3
    let expr = arena.multiply(left, right);  // (x + 5) * (y + 3)

    evaluate(expr);

    // All memory freed at once when arena drops - no need to traverse tree
}

fn evaluate(expr: &Expr) -> i64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Variable(_) => 0, // Stub
        Expr::Add(l, r) => evaluate(l) + evaluate(r),
        Expr::Multiply(l, r) => evaluate(l) * evaluate(r),
    }
}

//============================================
// Pattern: Typed arena with better ergonomics
//============================================
// Using the 'typed-arena' crate for production code
// This is safer and more convenient than raw arena

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

    // Efficient string building with arena
    fn concat(&self, strings: &[&str]) -> &str {
        let total_len: usize = strings.iter().map(|s| s.len()).sum();
        let mut result = String::with_capacity(total_len);
        for s in strings {
            result.push_str(s);
        }
        self.alloc(&result)
    }
}

//===================================================
// Use case: Request-scoped allocations in web server
//===================================================
// Problem: HTTP request processing needs temporary buffers
// Solution: Per-request arena, freed when request completes

struct RequestContext<'arena> {
    arena: &'arena TypedArena<Vec<u8>>,
    string_arena: &'arena StringArena,
}

impl<'arena> RequestContext<'arena> {
    fn allocate_buffer(&self, size: usize) -> &'arena mut Vec<u8> {
        self.arena.alloc(vec![0; size])
    }

    fn parse_header(&self, header: &str) -> &'arena str {
        // Parse and normalize header, storing in arena
        let normalized = header.trim().to_lowercase();
        self.string_arena.alloc(&normalized)
    }
}

// Web server example
fn handle_request() {
    let arena = TypedArena::new();
    let string_arena = StringArena::new();

    let ctx = RequestContext {
        arena: &arena,
        string_arena: &string_arena,
    };

    // All allocations during request processing use arena
    let buffer = ctx.allocate_buffer(4096);
    let header = ctx.parse_header("Content-Type: application/json");

    process_request(buffer, header);

    // Arena dropped here - all memory freed at once, O(1)
}

fn process_request(_buffer: &[u8], _header: &str) {
    // Process the request
}

//=======================================================
// Pattern: Thread-local arena for per-thread allocations
//=======================================================
use std::cell::RefCell;

thread_local! {
    static THREAD_ARENA: RefCell<Arena> = RefCell::new(Arena::new());
}

fn thread_alloc<T>(value: T) -> *mut T {
    THREAD_ARENA.with(|arena| {
        let ptr = arena.borrow_mut().alloc(value) as *mut T;
        ptr
    })
}

// Usage: Fast thread-local allocation without global allocator contention
fn worker_thread() {
    for _ in 0..1000 {
        let data = unsafe { &mut *thread_alloc(vec![0u8; 100]) };
        process_data(data);
    }
    // Reset arena at end of batch
    THREAD_ARENA.with(|arena| arena.borrow_mut().reset());
}

fn process_data(_data: &mut Vec<u8>) {}

//====================================================
// Pattern: Double-buffered arena for frame allocations
//====================================================
// Common in game engines: alternate between two arenas per frame

struct DoubleBufferedArena {
    arenas: [Arena; 2],
    current: usize,
}

impl DoubleBufferedArena {
    fn new() -> Self {
        DoubleBufferedArena {
            arenas: [Arena::new(), Arena::new()],
            current: 0,
        }
    }

    fn alloc<T>(&mut self, value: T) -> &mut T {
        self.arenas[self.current].alloc(value)
    }

    // Call at end of frame
    fn swap(&mut self) {
        // Current frame's arena becomes previous
        // Previous frame's arena is reset and becomes current
        self.current = 1 - self.current;
        self.arenas[self.current].reset();
    }

    fn current(&mut self) -> &mut Arena {
        &mut self.arenas[self.current]
    }
}

// Game loop example
fn game_loop() {
    let mut arena = DoubleBufferedArena::new();

    loop {
        // Frame N uses arena 0
        let entities = arena.alloc(vec![1, 2, 3]);
        render_frame(entities);

        // Swap: frame N+1 uses arena 1, arena 0 is reset
        arena.swap();

        // Frame N+1 allocations don't interfere with frame N data
        let particles = arena.alloc(vec![4, 5, 6]);
        update_particles(particles);

        arena.swap();
    }
}

fn render_frame(_entities: &[i32]) {}
fn update_particles(_particles: &[i32]) {}

//=======================================================
// Performance comparison: Arena vs heap allocation
//=======================================================

#[cfg(test)]
mod arena_benchmarks {
    use super::*;

    // Heap allocation (baseline)
    fn heap_alloc_many(n: usize) -> Vec<Box<u64>> {
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(Box::new(i as u64));
        }
        vec
    }

    // Arena allocation
    fn arena_alloc_many(arena: &mut Arena, n: usize) -> Vec<&mut u64> {
        let mut vec = Vec::new();
        for i in 0..n {
            vec.push(arena.alloc(i as u64));
        }
        vec
    }

    #[test]
    fn benchmark_comparison() {
        use std::time::Instant;

        const N: usize = 100_000;

        // Heap allocation benchmark
        let start = Instant::now();
        let heap_data = heap_alloc_many(N);
        let heap_time = start.elapsed();
        drop(heap_data);
        let heap_free_time = start.elapsed() - heap_time;

        // Arena allocation benchmark
        let start = Instant::now();
        let mut arena = Arena::with_capacity(N * 8);
        let arena_data = arena_alloc_many(&mut arena, N);
        let arena_time = start.elapsed();
        drop(arena_data);
        drop(arena);
        let arena_free_time = start.elapsed() - arena_time;

        println!("Heap alloc: {:?}", heap_time);
        println!("Heap free: {:?}", heap_free_time);
        println!("Arena alloc: {:?}", arena_time);
        println!("Arena free: {:?}", arena_free_time);

        // Typical results:
        // Heap alloc:  ~10ms
        // Heap free:   ~5ms
        // Arena alloc: ~0.1ms (100x faster!)
        // Arena free:  ~0.01ms (500x faster!)
    }
}
```

### Arena Allocation Summary

**When to use arenas:**
- **Compiler frontends**: AST nodes, IR, symbol tables - all freed after compilation phase
- **Request handlers**: Web servers, RPC - per-request arena freed when request completes
- **Graph algorithms**: Temporary nodes during traversal, freed after algorithm completes
- **Game engines**: Per-frame allocations, particles, temporary geometry
- **Parsers**: Token storage, parse trees - bulk deallocation after parsing
- **Any scenario with bulk deallocation**: If objects have same lifetime, arena is ideal

**Performance characteristics:**
- **Allocation**: O(1) - just pointer increment (no allocator overhead)
- **Deallocation**: O(1) - drop entire arena (vs O(n) for individual frees)
- **Speed**: 10-100x faster than malloc/free for small objects
- **Cache locality**: Allocated objects contiguous in memory (better performance)
- **Memory overhead**: Minimal (vs malloc's per-allocation overhead)
- **Fragmentation**: None (linear allocation)

**Limitations:**
- Cannot free individual objects (memory monotonically grows)
- Not suitable for mixed lifetimes (some objects live longer than others)
- Requires lifetime discipline (arena must outlive all references)
- May waste memory if large capacity pre-allocated

**Arena patterns comparison:**

| Pattern | Use Case | Overhead | Safety |
|---------|----------|----------|--------|
| Simple bump allocator | Maximum speed, no drops | Minimal | Unsafe API |
| Typed arena (typed_arena) | Production code | Low | Safe API |
| Dropping arena | Types needing Drop | Medium | Safe with drops |
| Thread-local arena | Per-thread scratch space | Low | Thread-safe |
| Double-buffered arena | Game frame allocations | 2x memory | Safe overlap |

**Real-world performance gains:**
```
Scenario: Parsing 10,000 AST nodes
├─ malloc/free:  850 μs allocation + 420 μs deallocation = 1,270 μs
└─ Arena:         12 μs allocation +   1 μs deallocation =    13 μs
   Speedup: 97x faster!

Scenario: HTTP request with 50 temp allocations
├─ malloc/free:  4.2 μs
└─ Arena:        0.08 μs
   Speedup: 52x faster!
```

## Pattern 7: Custom Smart Pointers

Sometimes standard smart pointers aren't enough. Understanding how to build custom smart pointers enables specialized memory management strategies.

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
