# Advanced Smart Pointer Patterns

You've used `Box`, `Rc`, and `Arc`. Now you want to know how they work—and when to build your own.

This chapter goes beneath the standard library. You'll implement smart pointers from scratch, embed reference counts directly in your data structures, and optimize memory layout for cache efficiency. These are the techniques behind high-performance Rust libraries: `parking_lot`'s mutexes, `triomphe`'s Arc, Servo's DOM, and game engine ECS systems.

This chapter covers:
- **Building custom smart pointers** with `Deref`, `DerefMut`, and `Drop`—lazy initialization, access tracking, copy-on-write
- **Intrusive data structures** where nodes contain their own links—the Linux kernel approach
- **Memory layout optimization**: field ordering, cache alignment, struct-of-arrays
- **Generational indices** for stable handles without reference counting—the ECS pattern
- **Reference counting tricks** to squeeze out the last nanoseconds

These patterns require `unsafe` code and careful reasoning about invariants. If Chapter 1 taught you to work with the borrow checker, this chapter teaches you to work around it—safely.

## Pattern 1: Custom Smart Pointers

**Problem**: Standard smart pointers (Box, Rc, Arc) are general-purpose. Some applications need specialized behavior: access tracking, lazy initialization, or domain-specific semantics.

**Solution**: Implement `Deref`, `DerefMut`, and `Drop` traits to create custom pointer types with controlled access and cleanup.

**Why It Matters**: Understanding how smart pointers work internally helps you debug memory issues, write unsafe code correctly, and create domain-specific abstractions. Libraries like `parking_lot`, `once_cell`, and `triomphe` all use these techniques.

### Example: Simple Custom Box

Minimal Box using `Box::into_raw` for raw pointer and `Box::from_raw` in Drop for cleanup. Foundation for all custom smart pointers.

```rust
use std::ops::{Deref, DerefMut};

struct MyBox<T> {
    data: *mut T,
}

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        let data = Box::into_raw(Box::new(value));
        MyBox { data }
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.data)); }
    }
}

// Usage
let mut b = MyBox::new(42);
*b = 100;
println!("{}", *b); // 100
```

### Example: Logging Pointer (Access Tracking)

Tracks every read/write for debugging or profiling using Cell for counter mutation through `&self`. Useful for finding hot paths or detecting unexpected access patterns.

```rust
use std::ops::{Deref, DerefMut};
use std::cell::Cell;

struct LoggingPtr<T> {
    data: Box<T>,
    reads: Cell<usize>,
    writes: Cell<usize>,
}

impl<T> LoggingPtr<T> {
    fn new(value: T) -> Self {
        Self {
            data: Box::new(value),
            reads: Cell::new(0),
            writes: Cell::new(0),
        }
    }

    fn stats(&self) -> (usize, usize) {
        (self.reads.get(), self.writes.get())
    }
}

impl<T> Deref for LoggingPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.reads.set(self.reads.get() + 1);
        &self.data
    }
}

impl<T> DerefMut for LoggingPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writes.set(self.writes.get() + 1);
        &mut self.data
    }
}

// Usage: Profile hot paths
let mut p = LoggingPtr::new(vec![1, 2, 3]);
let _ = p.len();      // read
let _ = p.len();      // read
p.push(4);            // write
println!("{:?}", p.stats()); // (2, 1)
```

### Example: Lazy Initialization

Defer expensive computation until first access using UnsafeCell for mutation through `&self`. For production, use `once_cell::Lazy` or `std::sync::LazyLock` for thread safety.

```rust
use std::cell::UnsafeCell;

struct Lazy<T, F: FnOnce() -> T> {
    value: UnsafeCell<Option<T>>,
    init: UnsafeCell<Option<F>>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    fn new(init: F) -> Self {
        Self {
            value: UnsafeCell::new(None),
            init: UnsafeCell::new(Some(init)),
        }
    }

    fn get(&self) -> &T {
        unsafe {
            if (*self.value.get()).is_none() {
                let init = (*self.init.get()).take().unwrap();
                *self.value.get() = Some(init());
            }
            (*self.value.get()).as_ref().unwrap()
        }
    }
}

// Usage: Expensive init runs only on first access
fn expensive_init() -> Vec<i32> {
    println!("Computing...");
    (0..1000).collect()
}

let lazy = Lazy::new(expensive_init);
// Nothing computed yet
let data = lazy.get(); // "Computing..." printed here
let data2 = lazy.get(); // No print, returns cached value
```

### Example: Copy-on-Write Pointer

Share data until mutation, then clone automatically. The `modify` method checks `strong_count`—if >1, clone before mutating. Avoids copies for read-heavy workloads.

```rust
use std::rc::Rc;
use std::ops::Deref;

struct CowPtr<T: Clone> {
    data: Rc<T>,
}

impl<T: Clone> CowPtr<T> {
    fn new(data: T) -> Self {
        CowPtr { data: Rc::new(data) }
    }

    fn modify<F: FnOnce(&mut T)>(&mut self, f: F) {
        // Clone only if shared
        if Rc::strong_count(&self.data) > 1 {
            self.data = Rc::new((*self.data).clone());
        }
        f(Rc::get_mut(&mut self.data).unwrap());
    }
}

impl<T: Clone> Deref for CowPtr<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.data }
}

impl<T: Clone> Clone for CowPtr<T> {
    fn clone(&self) -> Self {
        CowPtr { data: Rc::clone(&self.data) }
    }
}

// Usage: Clones share data until modification
let original = CowPtr::new(vec![1, 2, 3]);
let mut copy = original.clone();  // Cheap: shares Rc
copy.modify(|v| v.push(4));       // Clone happens here
assert_eq!(original.len(), 3);    // Original unchanged
assert_eq!(copy.len(), 4);
```

## Pattern 2: Intrusive Reference Counting

**Problem**: Standard Rc/Arc allocate the ref count separately from the data—two allocations, poor cache locality.

**Solution**: Store the reference count inside the data structure itself. One allocation, better performance for many small objects.

**Why It Matters**: When you have millions of small shared objects (graph nodes, AST nodes, cache entries), the overhead of standard Rc/Arc becomes significant. Intrusive reference counting is used in production systems like Servo's DOM implementation and Linux kernel data structures.

### Example: Intrusive Rc

Refcount lives inside the node, eliminating separate allocation. NonNull and PhantomData ensure proper pointer semantics. Single-allocation design improves memory usage and cache performance.

```rust
use std::ptr::NonNull;
use std::marker::PhantomData;
use std::cell::Cell;
use std::ops::Deref;

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

    fn refcount(&self) -> usize {
        unsafe { self.ptr.as_ref().refcount.get() }
    }
}

impl<T> Clone for IntrusiveRc<T> {
    fn clone(&self) -> Self {
        let node = unsafe { self.ptr.as_ref() };
        node.refcount.set(node.refcount.get() + 1);
        IntrusiveRc { ptr: self.ptr, _marker: PhantomData }
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

// Usage: Single allocation for data + refcount
let rc1 = IntrusiveRc::new(String::from("hello"));
let rc2 = rc1.clone();
assert_eq!(rc1.refcount(), 2);
assert_eq!(*rc1, "hello");
```

## Pattern 3: Intrusive Data Structures

**Problem**: Standard linked lists allocate separate nodes pointing to data—two allocations per element, poor cache locality.

**Solution**: Embed link pointers directly in data structures. One allocation per element, data and links are contiguous.

**Why It Matters**: Intrusive data structures are fundamental in systems programming. The Linux kernel uses them extensively for schedulers, filesystems, and drivers. When data needs to belong to multiple collections simultaneously or when allocation overhead matters, intrusive design is the answer.

### Example: Intrusive Singly-Linked List

Minimal list where each node contains data and next pointer. O(1) push_front/pop_front. Avoids double indirection of `Box<Node<Box<T>>>` in standard implementations.

```rust
use std::ptr;
use std::marker::PhantomData;

struct ListNode<T> {
    next: *mut ListNode<T>,
    data: T,
}

struct IntrusiveList<T> {
    head: *mut ListNode<T>,
    _phantom: PhantomData<T>,
}

impl<T> IntrusiveList<T> {
    fn new() -> Self {
        IntrusiveList { head: ptr::null_mut(), _phantom: PhantomData }
    }

    fn push_front(&mut self, data: T) {
        let node = Box::into_raw(Box::new(ListNode {
            next: self.head,
            data,
        }));
        self.head = node;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }
        unsafe {
            let node = Box::from_raw(self.head);
            self.head = node.next;
            Some(node.data)
        }
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        struct Iter<'a, T> {
            current: *mut ListNode<T>,
            _phantom: PhantomData<&'a T>,
        }
        impl<'a, T> Iterator for Iter<'a, T> {
            type Item = &'a T;
            fn next(&mut self) -> Option<Self::Item> {
                if self.current.is_null() {
                    None
                } else {
                    unsafe {
                        let data = &(*self.current).data;
                        self.current = (*self.current).next;
                        Some(data)
                    }
                }
            }
        }
        Iter { current: self.head, _phantom: PhantomData }
    }
}

impl<T> Drop for IntrusiveList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// Usage
let mut list = IntrusiveList::new();
list.push_front(3);
list.push_front(2);
list.push_front(1);
for item in list.iter() {
    println!("{}", item); // 1, 2, 3
}
```

### Example: LRU Cache with Intrusive Doubly-Linked List

O(1) lookup via HashMap plus O(1) eviction via doubly-linked list. HashMap stores node pointers directly; list tracks recency. Intrusive design: single allocation per entry in both structures.

```rust
use std::collections::HashMap;
use std::ptr;

struct LruNode<K, V> {
    key: K,
    value: V,
    prev: *mut LruNode<K, V>,
    next: *mut LruNode<K, V>,
}

struct LruCache<K, V> {
    map: HashMap<K, *mut LruNode<K, V>>,
    head: *mut LruNode<K, V>,  // Most recent
    tail: *mut LruNode<K, V>,  // Least recent
    capacity: usize,
}

impl<K: Eq + std::hash::Hash + Clone, V> LruCache<K, V> {
    fn new(capacity: usize) -> Self {
        LruCache {
            map: HashMap::new(),
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            capacity,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        let node_ptr = *self.map.get(key)?;
        unsafe {
            self.detach(node_ptr);
            self.attach_front(node_ptr);
            Some(&(*node_ptr).value)
        }
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            unsafe {
                (*node_ptr).value = value;
                self.detach(node_ptr);
                self.attach_front(node_ptr);
            }
            return;
        }

        // Evict if at capacity
        if self.map.len() >= self.capacity && !self.tail.is_null() {
            unsafe {
                let tail_key = (*self.tail).key.clone();
                let old_tail = self.tail;
                self.detach(old_tail);
                self.map.remove(&tail_key);
                drop(Box::from_raw(old_tail));
            }
        }

        let node = Box::into_raw(Box::new(LruNode {
            key: key.clone(),
            value,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }));

        self.map.insert(key, node);
        unsafe { self.attach_front(node); }
    }

    unsafe fn detach(&mut self, node: *mut LruNode<K, V>) {
        let prev = (*node).prev;
        let next = (*node).next;

        if !prev.is_null() { (*prev).next = next; }
        else { self.head = next; }

        if !next.is_null() { (*next).prev = prev; }
        else { self.tail = prev; }

        (*node).prev = ptr::null_mut();
        (*node).next = ptr::null_mut();
    }

    unsafe fn attach_front(&mut self, node: *mut LruNode<K, V>) {
        (*node).next = self.head;
        if !self.head.is_null() { (*self.head).prev = node; }
        self.head = node;
        if self.tail.is_null() { self.tail = node; }
    }
}

impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        unsafe {
            let mut current = self.head;
            while !current.is_null() {
                let next = (*current).next;
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}

// Usage
let mut cache = LruCache::new(2);
cache.put("a", 1);
cache.put("b", 2);
cache.get(&"a");      // "a" becomes most recent
cache.put("c", 3);    // Evicts "b" (least recent)
assert!(cache.get(&"b").is_none());
```

## Pattern 4: Memory Layout Optimization

**Problem**: Poor struct layout wastes memory (padding) and hurts cache performance. False sharing destroys multi-threaded performance.

**Solution**: Order fields by alignment, use repr attributes, pad to cache lines.

**Why It Matters**: In high-performance code, memory layout directly impacts speed. A 50% size reduction means 50% more data fits in cache. Eliminating false sharing can provide 10x speedup in multi-threaded hot paths. These optimizations are essential for games, databases, and scientific computing.

### Example: Field Ordering

Order fields largest to smallest alignment to minimize padding. Without `#[repr(C)]` compiler may reorder, but explicit ordering documents intent. Can cut struct size by 30-50%.

```rust
// Bad: 24 bytes with padding
#[repr(C)]
struct Unoptimized {
    a: u8,      // 1 byte + 7 padding
    b: u64,     // 8 bytes
    c: u8,      // 1 byte + 7 padding
}               // Total: 24 bytes

// Good: 16 bytes
#[repr(C)]
struct Optimized {
    b: u64,     // 8 bytes (largest first)
    a: u8,      // 1 byte
    c: u8,      // 1 byte + 6 padding
}               // Total: 16 bytes

assert_eq!(std::mem::size_of::<Unoptimized>(), 24);
assert_eq!(std::mem::size_of::<Optimized>(), 16);
```

### Example: Cache Line Alignment

Prevent false sharing by padding to cache line boundaries (64 bytes on x86). False sharing: threads modify different variables on same cache line, causing bounce between cores—destroys parallel performance.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

const CACHE_LINE: usize = 64;

#[repr(align(64))]
struct Padded<T> {
    value: T,
}

struct Counters {
    counter1: Padded<AtomicUsize>,  // Own cache line
    counter2: Padded<AtomicUsize>,  // Own cache line
}

// Usage: Threads can update counters without false sharing
let counters = Counters {
    counter1: Padded { value: AtomicUsize::new(0) },
    counter2: Padded { value: AtomicUsize::new(0) },
};

// Thread 1 updates counter1, Thread 2 updates counter2
// No cache line bouncing between cores
counters.counter1.value.fetch_add(1, Ordering::Relaxed);
```

### Example: Optimizing Enum Size

Enums sized to largest variant—one large variant bloats all. Box large variants to keep enum small. Important for recursive types and enums in collections.

```rust
// Bad: 1024+ bytes for every instance
enum Large {
    Small(u8),
    Big([u8; 1024]),
}

// Good: ~16 bytes (pointer + discriminant)
enum Optimized {
    Small(u8),
    Big(Box<[u8; 1024]>),
}

assert!(std::mem::size_of::<Large>() > 1024);
assert!(std::mem::size_of::<Optimized>() <= 16);
```

### Example: Struct of Arrays (SoA) for Cache Efficiency

When loops access single field across many objects, SoA maximizes cache utilization. AoS loads entire structs; SoA keeps fields contiguous for efficient CPU prefetching.

```rust
// Array of Structs: poor locality when accessing single field
struct ParticleAoS {
    position: [f32; 3],
    velocity: [f32; 3],
    mass: f32,
}

fn update_aos(particles: &mut [ParticleAoS]) {
    for p in particles {
        // CPU loads entire struct even though we only need position and velocity
        p.position[0] += p.velocity[0];
    }
}

// Struct of Arrays: excellent locality
struct ParticlesSoA {
    positions_x: Vec<f32>,
    velocities_x: Vec<f32>,
}

impl ParticlesSoA {
    fn update(&mut self) {
        // positions_x is contiguous; CPU prefetches efficiently
        for i in 0..self.positions_x.len() {
            self.positions_x[i] += self.velocities_x[i];
        }
    }
}
```

## Pattern 5: Generational Indices

**Problem**: Vec indices are unstable—removing elements invalidates subsequent indices. Stale indices cause use-after-free bugs.

**Solution**: Pair index with generation counter. When slot is reused, generation increments. Stale handles have wrong generation.

**Why It Matters**: Entity-Component-System (ECS) architectures, game engines, and object pools all need stable handles to objects that may be created and destroyed frequently. Generational indices provide safe, O(1) access without the overhead of Rc/Arc and without use-after-free bugs.

### Example: Generational Arena

Handle contains index + generation counter. When slot reused, generation increments, invalidating old handles. Safe access without refcounting; catches stale handles at runtime.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Handle {
    index: usize,
    generation: u64,
}

struct Slot<T> {
    value: Option<T>,
    generation: u64,
}

struct GenArena<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<usize>,
}

impl<T> GenArena<T> {
    fn new() -> Self {
        GenArena { slots: Vec::new(), free_list: Vec::new() }
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
        let slot = self.slots.get_mut(handle.index)?;
        if slot.generation != handle.generation {
            return None;
        }
        self.free_list.push(handle.index);
        slot.value.take()
    }
}

// Usage: Handles remain valid even after removals
let mut arena = GenArena::new();
let h1 = arena.insert("first");
let h2 = arena.insert("second");

arena.remove(h1);  // Slot 0 freed, generation incremented

let h3 = arena.insert("third");  // Reuses slot 0 with new generation

// Old handle is safely rejected
assert!(arena.get(h1).is_none());  // Stale handle!
assert_eq!(arena.get(h3), Some(&"third"));
```

## Pattern 6: Reference Counting Optimization

**Problem**: Rc::clone costs ~10ns per call. In hot loops, unnecessary clones waste CPU.

**Solution**: Borrow `&Rc<T>` instead of cloning when possible. Use try_unwrap to avoid clones when you're the sole owner.

**Why It Matters**: In hot paths, even 10ns per operation adds up. A million unnecessary clones costs 10ms—noticeable in games running at 60fps (16ms budget) or in high-throughput servers. These micro-optimizations compound in real applications.

### Example: Borrow Instead of Clone

Borrow through Deref instead of cloning Rc when you only need to read. Rc::clone has ~10ns overhead (atomic for Arc). Reserve cloning for when you actually need shared ownership.

```rust
use std::rc::Rc;

// Bad: Clones on every call
fn inefficient(data: &Rc<Vec<i32>>) {
    let clone = Rc::clone(data);  // Unnecessary!
    println!("{}", clone.len());
}

// Good: Borrow through Deref
fn efficient(data: &Rc<Vec<i32>>) {
    println!("{}", data.len());  // No clone needed
}

// Benchmark: 1M iterations
// inefficient: ~10ms (clone + drop overhead)
// efficient: ~1ms (just function calls)
```

### Example: try_unwrap for Sole Owner

`try_unwrap` returns inner value without cloning if refcount is 1. Falls back to clone only when shared—eliminates redundant copies in common cases.

```rust
use std::rc::Rc;

fn make_owned(data: Rc<Vec<i32>>) -> Vec<i32> {
    // If we're the only owner, unwrap without cloning
    Rc::try_unwrap(data).unwrap_or_else(|rc| (*rc).clone())
}

// Usage
let data = Rc::new(vec![1, 2, 3]);
let owned = make_owned(data);  // No clone: we were sole owner
```

### Example: String Interning

Deduplicate strings so identical values share single allocation via map from content to Rc<str>. Common in compilers, JSON parsers, apps with repeated strings.

```rust
use std::rc::Rc;
use std::collections::HashMap;

struct StringInterner {
    map: HashMap<String, Rc<str>>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner { map: HashMap::new() }
    }

    fn intern(&mut self, s: &str) -> Rc<str> {
        if let Some(interned) = self.map.get(s) {
            Rc::clone(interned)
        } else {
            let rc: Rc<str> = Rc::from(s);
            self.map.insert(s.to_string(), Rc::clone(&rc));
            rc
        }
    }
}

// Usage: Repeated strings share allocation
let mut interner = StringInterner::new();
let s1 = interner.intern("hello");
let s2 = interner.intern("hello");  // Returns same Rc
assert!(Rc::ptr_eq(&s1, &s2));      // Same allocation!
```

### Example: Weak for Non-Owning References

Weak observes without preventing deallocation. `upgrade()` returns None if data dropped. Essential for observer patterns, caches, breaking reference cycles.

```rust
use std::rc::{Rc, Weak};

struct Observer {
    subject: Weak<Vec<i32>>,
}

impl Observer {
    fn observe(&self) {
        // Temporarily upgrade to access data
        if let Some(data) = self.subject.upgrade() {
            println!("Observed: {} items", data.len());
        }
        // Rc dropped immediately, no permanent ownership
    }
}

// Usage
let data = Rc::new(vec![1, 2, 3]);
let observer = Observer { subject: Rc::downgrade(&data) };
observer.observe();  // "Observed: 3 items"
drop(data);
observer.observe();  // Nothing (data is gone)
```

---

## Performance Summary

| Pattern | Allocation | Access | Best Use Case |
|---------|------------|--------|---------------|
| Custom Ptr | O(1) heap | O(1) | Logging, lazy init, specialized behavior |
| Intrusive Rc | O(1) heap | O(1) + count | Many small shared objects |
| Intrusive List | O(1) heap | O(1) | O(1) removal, cache locality |
| GenArena | O(1) | O(1) | Stable handles, ECS patterns |

## When to Use Each Pattern

| Pattern | Use When |
|---------|----------|
| Custom smart pointer | Need specialized behavior (logging, lazy init) |
| Intrusive Rc | Many small objects, cache matters |
| Intrusive list | Need O(1) removal, cache locality |
| LRU cache | Fixed capacity, O(1) operations |
| Memory alignment | Multi-threaded counters, SIMD |
| SoA layout | Hot loops accessing single field |
| Generational index | ECS, object pools, stable handles |
| Rc optimization | Hot paths where clone overhead matters |

## Safety Notes

- Custom smart pointers require `unsafe`—ensure proper Drop implementation
- Intrusive structures need careful lifetime management
- Generational indices prevent use-after-free but not double-free
- Memory layout changes can break FFI compatibility
- Always test with Miri for undefined behavior detection
