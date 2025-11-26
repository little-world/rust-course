## Project 3: Arena Allocator with Lifetime Variance

### Problem Statement

Build a type-safe arena allocator that:
- Allocates objects with tied lifetimes (all objects live as long as arena)
- Supports efficient bulk allocation and deallocation
- Demonstrates lifetime variance (covariance, invariance)
- Provides typed and untyped arena variants
- Implements safe iteration over allocated objects
- Supports interior mutability with lifetime safety
- Uses phantom data to control variance
- Enables self-referential object graphs within arena

The allocator must leverage Rust's variance rules for maximum flexibility while maintaining safety.

### Why It Matters

Arena allocators are critical for performance:
- **Compilers**: AST nodes allocated in arena
- **Game Engines**: Frame-by-frame entity allocation
- **Parsers**: Parse tree nodes in arena
- **Database Systems**: Query plan nodes
- **Graphics**: Scene graph allocation

Understanding variance is essential:
- **Lifetime Flexibility**: Longer lifetimes usable where shorter expected
- **Soundness**: Invariance prevents lifetime bugs with mutation
- **API Design**: Choose correct variance for custom pointer types
- **Generic Collections**: Understand why `Vec<T>` covariant, `Cell<T>` invariant

### Use Cases

1. **AST Construction**: Parse tree with arena-allocated nodes
2. **Graph Algorithms**: Nodes/edges in arena
3. **Game Objects**: Entities, components in frame arena
4. **String Interning**: Deduplicated strings with arena lifetime
5. **Temporary Allocations**: Batch allocations for request handling
6. **Bump Allocator**: Fast linear allocation for short-lived objects
7. **Object Pools**: Reusable typed object allocation

### Solution Outline

**Core Structure:**
```rust
use std::cell::UnsafeCell;
use std::marker::PhantomData;

pub struct Arena<'a> {
    chunks: Vec<Chunk>,
    _marker: PhantomData<&'a ()>,  // Covariant over 'a
}

struct Chunk {
    data: Vec<u8>,
    offset: usize,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self { /* ... */ }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T { /* ... */ }

    pub fn alloc_slice<T>(&'a self, len: usize) -> &'a mut [T] { /* ... */ }
}

// Typed arena (invariant for safety)
pub struct TypedArena<T> {
    chunks: Vec<Vec<T>>,
    current: UnsafeCell<Vec<T>>,
}

impl<T> TypedArena<T> {
    pub fn alloc(&self, value: T) -> &mut T { /* ... */ }

    pub fn alloc_iter<I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T> { /* ... */ }
}
```

**Variance Patterns:**
- **Covariant Arena**: `PhantomData<&'a ()>` makes `Arena<'static>` usable as `Arena<'short>`
- **Invariant TypedArena**: Interior mutability requires invariance
- **Lifetime Bounds**: Objects in arena can reference each other

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_arena_allocation() {
    let arena = Arena::new();
    let x = arena.alloc(42);
    let y = arena.alloc(100);

    assert_eq!(*x, 42);
    assert_eq!(*y, 100);
}

#[test]
fn test_variance() {
    fn takes_short_arena(arena: &Arena<'_>) {
        let _ = arena.alloc(42);
    }

    let long_arena: Arena<'static> = Arena::new();
    takes_short_arena(&long_arena);  // OK: covariant
}

#[test]
fn test_self_referential_graph() {
    struct Node<'a> {
        value: i32,
        next: Option<&'a Node<'a>>,
    }

    let arena = Arena::new();
    let node1 = arena.alloc(Node { value: 1, next: None });
    let node2 = arena.alloc(Node { value: 2, next: Some(node1) });

    assert_eq!(node2.next.unwrap().value, 1);
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Basic Bump Allocator with Unsafe

**Goal:** Implement simple arena using unsafe pointer arithmetic.

**What to implement:**
```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

pub struct BumpAllocator {
    buffer: *mut u8,
    capacity: usize,
    offset: usize,
}

impl BumpAllocator {
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 8).unwrap();
        let buffer = unsafe { alloc(layout) };

        BumpAllocator {
            buffer,
            capacity,
            offset: 0,
        }
    }

    pub fn alloc<T>(&mut self, value: T) -> *mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Align offset
        let padding = (align - (self.offset % align)) % align;
        self.offset += padding;

        if self.offset + size > self.capacity {
            panic!("Arena out of memory");
        }

        let ptr = unsafe { self.buffer.add(self.offset) as *mut T };
        self.offset += size;

        unsafe {
            ptr.write(value);
        }

        ptr
    }

    pub fn reset(&mut self) {
        self.offset = 0;
        // Note: doesn't call destructors!
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 8).unwrap();
        unsafe {
            dealloc(self.buffer, layout);
        }
    }
}
```

**Check/Test:**
- Test allocation of various types
- Test alignment is correct
- Test out-of-memory panic
- Test reset reuses buffer
- Memory leak test (valgrind/miri)

**Why this isn't enough:**
Returns raw pointers—no lifetime safety! Can use-after-free if arena dropped while pointers exist. No Drop called for allocated objects—leaks resources. Type-unsafe (just `*mut T`). No bulk operations. We need lifetimes to tie allocations to arena lifetime.

---

### Step 2: Add Lifetimes and Safe References

**Goal:** Use lifetimes to prevent use-after-free.

**What to improve:**
```rust
use std::cell::Cell;

pub struct Arena<'a> {
    buffer: Vec<u8>,
    offset: Cell<usize>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self {
        Arena::with_capacity(4096)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Arena {
            buffer: Vec::with_capacity(capacity),
            offset: Cell::new(0),
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let mut offset = self.offset.get();

        // Align
        let padding = (align - (offset % align)) % align;
        offset += padding;

        let new_offset = offset + size;

        // Grow buffer if needed
        if new_offset > self.buffer.capacity() {
            panic!("Arena out of memory");
        }

        unsafe {
            // Write value at aligned offset
            let ptr = self.buffer.as_ptr().add(offset) as *mut T;
            ptr.write(value);

            self.offset.set(new_offset);

            &mut *ptr
        }
    }

    pub fn alloc_slice<T: Copy>(&'a self, slice: &[T]) -> &'a mut [T] {
        let len = slice.len();
        let size = std::mem::size_of::<T>() * len;
        let align = std::mem::align_of::<T>();

        let mut offset = self.offset.get();
        let padding = (align - (offset % align)) % align;
        offset += padding;

        let new_offset = offset + size;

        if new_offset > self.buffer.capacity() {
            panic!("Arena out of memory");
        }

        unsafe {
            let ptr = self.buffer.as_ptr().add(offset) as *mut T;

            // Copy slice data
            ptr.copy_from_nonoverlapping(slice.as_ptr(), len);

            self.offset.set(new_offset);

            std::slice::from_raw_parts_mut(ptr, len)
        }
    }
}
```

**Check/Test:**
- Test references tied to arena lifetime
- Test cannot use reference after arena dropped (compile error)
- Test multiple allocations share arena lifetime
- Test slice allocation

**Why this isn't enough:**
Growing buffer invalidates all previous pointers! When we grow `Vec`, it reallocates, moving data to a new address. All existing `&'a mut T` now point to freed memory. We need a chunked approach where each chunk has a stable address. Also, no Drop support—allocated objects don't destruct.

---

### Step 3: Implement Chunked Arena and Drop Support

**Goal:** Fix pointer invalidation and support destructors.

**What to improve:**
```rust
use std::cell::{Cell, RefCell};
use std::ptr::NonNull;

struct Chunk {
    data: Vec<u8>,
    offset: usize,
}

impl Chunk {
    fn new(capacity: usize) -> Self {
        Chunk {
            data: Vec::with_capacity(capacity),
            offset: 0,
        }
    }

    fn remaining(&self) -> usize {
        self.data.capacity() - self.offset
    }

    fn alloc<T>(&mut self, value: T) -> NonNull<T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let padding = (align - (self.offset % align)) % align;
        self.offset += padding;

        assert!(self.offset + size <= self.data.capacity());

        unsafe {
            let ptr = self.data.as_mut_ptr().add(self.offset) as *mut T;
            ptr.write(value);
            self.offset += size;

            NonNull::new_unchecked(ptr)
        }
    }
}

pub struct Arena<'a> {
    chunks: RefCell<Vec<Chunk>>,
    chunk_size: usize,
    // Track allocated objects for Drop
    destructors: RefCell<Vec<Box<dyn FnOnce()>>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    pub fn new() -> Self {
        Arena::with_chunk_size(4096)
    }

    pub fn with_chunk_size(chunk_size: usize) -> Self {
        let mut chunks = Vec::new();
        chunks.push(Chunk::new(chunk_size));

        Arena {
            chunks: RefCell::new(chunks),
            chunk_size,
            destructors: RefCell::new(Vec::new()),
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        let size = std::mem::size_of::<T>();

        // Find or create chunk with enough space
        let mut chunks = self.chunks.borrow_mut();

        let current_chunk = chunks.last_mut().unwrap();
        if current_chunk.remaining() < size + std::mem::align_of::<T>() {
            // Need new chunk
            let new_size = self.chunk_size.max(size * 2);
            chunks.push(Chunk::new(new_size));
        }

        let chunk = chunks.last_mut().unwrap();
        let ptr = chunk.alloc(value);

        // Register destructor if T needs drop
        if std::mem::needs_drop::<T>() {
            let destructor = Box::new(move || unsafe {
                ptr::drop_in_place(ptr.as_ptr());
            });
            self.destructors.borrow_mut().push(destructor);
        }

        unsafe { &mut *ptr.as_ptr() }
    }

    pub fn alloc_many<T, I>(&'a self, iter: I) -> &'a mut [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let len = iter.len();

        if len == 0 {
            return &mut [];
        }

        let size = std::mem::size_of::<T>() * len;
        let align = std::mem::align_of::<T>();

        let mut chunks = self.chunks.borrow_mut();

        // Ensure current chunk has space
        if chunks.last().unwrap().remaining() < size + align {
            chunks.push(Chunk::new(self.chunk_size.max(size * 2)));
        }

        let chunk = chunks.last_mut().unwrap();

        // Allocate space
        let padding = (align - (chunk.offset % align)) % align;
        chunk.offset += padding;

        let ptr = unsafe {
            chunk.data.as_mut_ptr().add(chunk.offset) as *mut T
        };

        chunk.offset += size;

        // Write items
        for (i, item) in iter.enumerate() {
            unsafe {
                ptr.add(i).write(item);
            }

            if std::mem::needs_drop::<T>() {
                let item_ptr = unsafe { ptr.add(i) };
                let destructor = Box::new(move || unsafe {
                    ptr::drop_in_place(item_ptr);
                });
                self.destructors.borrow_mut().push(destructor);
            }
        }

        unsafe { std::slice::from_raw_parts_mut(ptr, len) }
    }
}

impl<'a> Drop for Arena<'a> {
    fn drop(&mut self) {
        // Run all destructors
        let destructors = std::mem::take(&mut *self.destructors.borrow_mut());
        for destructor in destructors {
            destructor();
        }
    }
}
```

**Check/Test:**
- Test chunked allocation doesn't invalidate pointers
- Test Drop is called for allocated objects
- Test allocating many items at once
- Test large allocations get dedicated chunks
- Memory safety with Miri

**Why this isn't enough:**
Interior mutability (`RefCell`) makes the arena invariant over its lifetime `'a`. This prevents useful variance patterns. Also, the Drop tracking is inefficient—storing a closure per object is heavyweight. We should use a typed arena for better performance when all objects are the same type. Let's add variance control.

---

### Step 4: Add Variance Control and Typed Arena

**Goal:** Control variance with PhantomData and create typed arena variant.

**What to improve:**

**1. Covariant arena (immutable allocations):**
```rust
// Covariant over 'a - can use Arena<'long> where Arena<'short> expected
pub struct CovariantArena<'a> {
    chunks: UnsafeCell<Vec<Chunk>>,
    chunk_size: usize,
    _marker: PhantomData<&'a ()>,  // Covariant!
}

impl<'a> CovariantArena<'a> {
    pub fn alloc<T>(&'a self, value: T) -> &'a T {
        // Returns immutable reference for covariance
        unsafe {
            let chunks = &mut *self.chunks.get();
            // ... allocation logic ...
            &*ptr
        }
    }
}

// Can use with variance:
fn use_arena(arena: &CovariantArena<'_>) {
    let _ = arena.alloc(42);
}

let static_arena: CovariantArena<'static> = CovariantArena::new();
use_arena(&static_arena);  // OK: 'static coerces to shorter lifetime
```

**2. Typed arena (no Drop tracking overhead):**
```rust
pub struct TypedArena<T> {
    chunks: RefCell<Vec<Vec<T>>>,
    current: RefCell<Vec<T>>,
}

impl<T> TypedArena<T> {
    pub fn new() -> Self {
        TypedArena {
            chunks: RefCell::new(Vec::new()),
            current: RefCell::new(Vec::with_capacity(64)),
        }
    }

    pub fn alloc(&self, value: T) -> &mut T {
        let mut current = self.current.borrow_mut();

        // Move full chunk to chunks list
        if current.len() == current.capacity() {
            let new_current = Vec::with_capacity(current.capacity() * 2);
            let old_current = std::mem::replace(&mut *current, new_current);
            self.chunks.borrow_mut().push(old_current);
        }

        current.push(value);

        // Safe: reference is valid as long as TypedArena exists
        unsafe {
            let ptr = current.last_mut().unwrap() as *mut T;
            &mut *ptr
        }
    }

    pub fn alloc_many<I>(&self, iter: I) -> &mut [T]
    where
        I: IntoIterator<Item = T>,
    {
        let mut current = self.current.borrow_mut();

        let start = current.len();
        current.extend(iter);
        let end = current.len();

        unsafe {
            let ptr = current.as_mut_ptr().add(start);
            std::slice::from_raw_parts_mut(ptr, end - start)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        // Iterate over all allocated objects
        self.chunks.borrow().iter()
            .flat_map(|chunk| chunk.iter())
            .chain(self.current.borrow().iter())
    }
}

// Drop automatically called for all T when TypedArena drops
```

**3. Demonstrate variance:**
```rust
#[test]
fn test_covariance() {
    fn accepts_short<'a>(arena: &CovariantArena<'a>, data: &'a str) {
        let _ = arena.alloc(data);
    }

    let long_arena: CovariantArena<'static> = CovariantArena::new();
    let static_str: &'static str = "hello";

    // 'static coerces to shorter lifetime
    accepts_short(&long_arena, static_str);  // OK!
}

#[test]
fn test_typed_arena_variance() {
    // TypedArena<&'a str> is covariant over 'a
    fn process_strings<'a>(arena: &TypedArena<&'a str>) {
        let _ = arena.alloc("short-lived");
    }

    let arena: TypedArena<&'static str> = TypedArena::new();
    process_strings(&arena);  // OK: covariant
}
```

**Check/Test:**
- Test variance allows lifetime coercion
- Test typed arena performance vs untyped
- Test iterator over allocated objects
- Test Drop called for all objects in TypedArena
- Benchmark: typed vs untyped arena

**Why this isn't enough:**
Can't have self-referential objects yet. What if we want graph nodes in the arena that reference each other? Current API doesn't support two-phase initialization. Also no string interning (deduplicate identical strings). Let's add those features.

---

### Step 5: Add Self-Referential Support and String Interning

**Goal:** Support object graphs with interior references.

**What to improve:**

**1. Two-phase initialization for self-references:**
```rust
impl<'a> Arena<'a> {
    pub fn alloc_uninit<T>(&'a self) -> &'a mut std::mem::MaybeUninit<T> {
        // Allocate uninitialized memory
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        unsafe {
            let chunks = &mut *self.chunks.get();
            // ... allocate space ...
            &mut *(ptr as *mut std::mem::MaybeUninit<T>)
        }
    }

    pub fn alloc_with<T, F>(&'a self, f: F) -> &'a mut T
    where
        F: FnOnce(&'a Arena<'a>) -> T,
    {
        // Call f with arena reference, allows creating self-refs
        let value = f(self);
        self.alloc(value)
    }
}

// Usage: Self-referential graph
#[test]
fn test_self_referential_graph() {
    struct Node<'a> {
        value: i32,
        neighbors: Vec<&'a Node<'a>>,
    }

    let arena = Arena::new();

    let node1 = arena.alloc(Node {
        value: 1,
        neighbors: Vec::new(),
    });

    let node2 = arena.alloc_with(|arena| {
        Node {
            value: 2,
            neighbors: vec![node1],  // Reference to node1!
        }
    });

    let node3 = arena.alloc_with(|arena| {
        Node {
            value: 3,
            neighbors: vec![node1, node2],
        }
    });

    assert_eq!(node3.neighbors.len(), 2);
    assert_eq!(node3.neighbors[0].value, 1);
    assert_eq!(node3.neighbors[1].value, 2);
}
```

**2. String interning:**
```rust
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

// Wrapper that hashes by content
struct InternedStr<'a>(&'a str);

impl<'a> Hash for InternedStr<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'a> PartialEq for InternedStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<'a> Eq for InternedStr<'a> {}

pub struct StringInterner<'a> {
    arena: Arena<'a>,
    map: RefCell<HashMap<InternedStr<'a>, &'a str>>,
}

impl<'a> StringInterner<'a> {
    pub fn new() -> Self {
        StringInterner {
            arena: Arena::new(),
            map: RefCell::new(HashMap::new()),
        }
    }

    pub fn intern(&'a self, s: &str) -> &'a str {
        let mut map = self.map.borrow_mut();

        // Check if already interned
        if let Some(&interned) = map.get(&InternedStr(s)) {
            return interned;
        }

        // Allocate new string in arena
        let bytes = self.arena.alloc_slice(s.as_bytes());
        let interned = unsafe {
            std::str::from_utf8_unchecked(bytes)
        };

        map.insert(InternedStr(interned), interned);
        interned
    }

    pub fn get(&self, s: &str) -> Option<&'a str> {
        self.map.borrow().get(&InternedStr(s)).copied()
    }
}

#[test]
fn test_string_interning() {
    let interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");
    let s3 = interner.intern("world");

    // Same string, same pointer
    assert!(std::ptr::eq(s1, s2));
    assert!(!std::ptr::eq(s1, s3));
}
```

**3. Iteration and collection:**
```rust
impl<T> TypedArena<T> {
    pub fn into_vec(self) -> Vec<T> {
        let mut result = Vec::new();

        let chunks = self.chunks.into_inner();
        for chunk in chunks {
            result.extend(chunk);
        }

        result.extend(self.current.into_inner());
        result
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.chunks.get_mut().iter_mut()
            .flat_map(|chunk| chunk.iter_mut())
            .chain(self.current.get_mut().iter_mut())
    }
}
```

**Check/Test:**
- Test self-referential graph allocation
- Test string interning deduplicates strings
- Test iteration over arena objects
- Test complex graph structures (trees, DAGs)
- Verify pointer equality for interned strings

**Why this isn't enough:**
No support for parallel allocation (thread-safe arena). Also, the iteration borrows the entire arena mutably—can't allocate while iterating. Real-world use cases like parallel parsing need thread-local arenas. Let's add thread safety.

---

### Step 6: Add Thread-Safe Scoped Arena and Parallel Allocation

**Goal:** Support concurrent allocation with scoped lifetimes.

**What to improve:**

**1. Thread-safe arena:**
```rust
use std::sync::{Arc, Mutex};

pub struct ThreadSafeArena<'a> {
    chunks: Arc<Mutex<Vec<Chunk>>>,
    chunk_size: usize,
    _marker: PhantomData<&'a ()>,
}

impl<'a> ThreadSafeArena<'a> {
    pub fn new() -> Self {
        let chunks = vec![Chunk::new(4096)];
        ThreadSafeArena {
            chunks: Arc::new(Mutex::new(chunks)),
            chunk_size: 4096,
            _marker: PhantomData,
        }
    }

    pub fn alloc<T>(&'a self, value: T) -> &'a mut T {
        let mut chunks = self.chunks.lock().unwrap();

        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Find/create chunk with space
        if chunks.last().unwrap().remaining() < size + align {
            chunks.push(Chunk::new(self.chunk_size));
        }

        let chunk = chunks.last_mut().unwrap();
        let ptr = chunk.alloc(value);

        unsafe { &mut *ptr.as_ptr() }
    }
}

unsafe impl<'a> Send for ThreadSafeArena<'a> {}
unsafe impl<'a> Sync for ThreadSafeArena<'a> {}
```

**2. Scoped arena with crossbeam:**
```rust
use crossbeam::thread;

pub fn scoped_arena<F, R>(f: F) -> R
where
    F: for<'scope> FnOnce(&'scope Arena<'scope>) -> R,
{
    let arena = Arena::new();
    f(&arena)
    // arena dropped here, all references invalid
}

#[test]
fn test_scoped_arena() {
    let result = scoped_arena(|arena| {
        let x = arena.alloc(42);
        let y = arena.alloc(100);
        *x + *y
    });

    assert_eq!(result, 142);
    // Cannot use x or y here - lifetime ended
}
```

**3. Thread-local arena pool:**
```rust
use std::cell::RefCell;

thread_local! {
    static ARENA: RefCell<Option<TypedArena<u8>>> = RefCell::new(None);
}

pub fn with_thread_arena<F, R>(f: F) -> R
where
    F: FnOnce(&TypedArena<u8>) -> R,
{
    ARENA.with(|cell| {
        let mut arena_opt = cell.borrow_mut();

        if arena_opt.is_none() {
            *arena_opt = Some(TypedArena::new());
        }

        let arena = arena_opt.as_ref().unwrap();
        f(arena)
    })
}
```

**4. Parallel parsing example:**
```rust
#[test]
fn test_parallel_parsing() {
    use rayon::prelude::*;

    let inputs = vec![
        "line 1 data",
        "line 2 data",
        "line 3 data",
    ];

    let results: Vec<_> = inputs.par_iter()
        .map(|input| {
            // Each thread gets its own arena
            let arena = Arena::new();
            parse_line(&arena, input)
        })
        .collect();

    assert_eq!(results.len(), 3);
}

fn parse_line<'a>(arena: &'a Arena<'a>, input: &str) -> Vec<&'a str> {
    input.split_whitespace()
        .map(|word| {
            let bytes = arena.alloc_slice(word.as_bytes());
            unsafe { std::str::from_utf8_unchecked(bytes) }
        })
        .collect()
}
```

**5. Complete variance demonstration:**
```rust
// Demonstrate all variance types
use std::cell::Cell;

// Covariant: &'a T
fn covariant_example() {
    let long: &'static str = "long";
    let short: &str = long;  // OK
}

// Invariant: Cell<&'a T>
fn invariant_example() {
    let long: Cell<&'static str> = Cell::new("long");
    // let short: Cell<&str> = long;  // ERROR: invariant
}

// Arena variance
fn arena_variance() {
    fn use_arena<'a>(arena: &Arena<'a>) {
        let _ = arena.alloc(42);
    }

    let static_arena: Arena<'static> = Arena::new();
    use_arena(&static_arena);  // OK: covariant over 'a
}

// PhantomData variance control
struct CovariantWrapper<'a, T> {
    _marker: PhantomData<&'a T>,
}

struct InvariantWrapper<'a, T> {
    _marker: PhantomData<Cell<&'a T>>,
}
```

**Check/Test:**
- Test thread-safe arena from multiple threads
- Test scoped arena lifetime enforcement
- Test parallel parsing with thread-local arenas
- Verify variance rules with compile tests
- Benchmark: parallel vs sequential allocation
- Test cannot leak references outside scoped lifetime

**What this achieves:**
A production-ready arena allocator:
- **Lifetime Safety**: References tied to arena lifetime
- **Performance**: O(1) allocation, bulk deallocation
- **Variance**: Covariant for flexibility
- **Thread Safety**: Concurrent allocation support
- **Self-References**: Object graphs within arena
- **String Interning**: Deduplication for strings
- **Scoped Lifetimes**: Safe temporary allocation

**Extensions to explore:**
- Reset/clear arena for reuse
- Statistics (bytes allocated, objects count)
- Alignment guarantees for SIMD types
- Integration with allocator API
- Stack-like pop/restore checkpoints

---

## Summary

These three projects teach essential lifetime patterns in Rust:

1. **Zero-Copy Parser**: Lifetime elision, explicit lifetimes, multiple lifetime parameters, borrowed data management—patterns for high-performance text processing.

2. **Async Scheduler with Pin**: Self-referential structs, Pin safety, HRTB for closures, async/await internals—understanding what makes async Rust work.

3. **Arena Allocator**: Variance, lifetime bounds, interior mutability, scoped lifetimes—advanced memory management with compile-time safety.

All three emphasize:
- **Zero-Cost Abstractions**: Lifetimes are compile-time only
- **Safety**: Prevent use-after-free at compile time
- **Flexibility**: Variance enables ergonomic APIs
- **Performance**: Lifetimes enable zero-copy patterns

Students will understand how Rust's lifetime system enables both safety and performance, preventing entire classes of bugs that plague C/C++ while maintaining zero runtime overhead.
