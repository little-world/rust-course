# Project 2: Object Pool with Smart Pointer Reuse

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
