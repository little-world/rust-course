# Reference-Counted Smart Pointer

### Problem Statement


Reference counting is fundamental to memory management in languages without garbage collection. Understanding `Rc` and `Arc` teaches you about shared ownership, reference cycles, and the trade-offs between compile-time and runtime safety. These patterns appear in GUI frameworks, graph structures, caches, and any system where data has multiple owners.


### What we will write 
Implement a custom reference-counted smart pointer (similar to `Rc<T>`) that allows multiple ownership of heap-allocated data. Your implementation should automatically free memory when the last reference is dropped and provide interior mutability through `RefCell`-like semantics.

Your smart pointer should support:
- Multiple owners sharing the same data
- Automatic cleanup when reference count reaches zero
- Weak references to break reference cycles
- Interior mutability patterns
- Clone-on-write optimization

---

## Understanding Reference Counting and Smart Pointers

Before implementing reference-counted pointers, let's understand the fundamental concepts of ownership, shared data, and the trade-offs between compile-time and runtime memory management.

### What is Reference Counting?

**Reference counting** is a memory management technique where each object tracks how many pointers (references) point to it. When the count drops to zero, the object is automatically freed.

**The Ownership Problem**:
```rust
// Rust's ownership: only ONE owner
let data = String::from("hello");
let owner1 = data;  // data moved to owner1
// let owner2 = data;  // ❌ ERROR: data was moved

// What if we NEED multiple owners?
// Example: GUI widget shared by multiple event handlers
// Example: Graph node with multiple incoming edges
```

**Reference Counting Solution**:
```rust
// Multiple owners sharing data
let data = Rc::new(String::from("hello"));
let owner1 = Rc::clone(&data);   // ✓ OK: count = 2
let owner2 = Rc::clone(&data);   // ✓ OK: count = 3
// All three point to the same String
// String freed when all dropped (count → 0)
```

**How It Works**:
```
Create Rc<String>("hello"):
┌─────────────────────┐
│   RcInner           │
│  ┌────────────────┐ │
│  │ strong: 1      │ │ ← Reference count
│  │ data: "hello"  │ │ ← Actual data
│  └────────────────┘ │
└─────────────────────┘
        ↑
       rc1

After rc2 = rc1.clone():
┌─────────────────────┐
│   RcInner           │
│  ┌────────────────┐ │
│  │ strong: 2      │ │ ← Incremented
│  │ data: "hello"  │ │
│  └────────────────┘ │
└─────────────────────┘
    ↑           ↑
   rc1         rc2

After drop(rc1):
┌─────────────────────┐
│   RcInner           │
│  ┌────────────────┐ │
│  │ strong: 1      │ │ ← Decremented
│  │ data: "hello"  │ │
│  └────────────────┘ │
└─────────────────────┘
                ↑
               rc2

After drop(rc2):
┌─────────────────────┐
│   RcInner           │
│  ┌────────────────┐ │
│  │ strong: 0      │ │ ← Zero: FREE MEMORY!
│  │ data: "hello"  │ │ ← Dropped
│  └────────────────┘ │
└─────────────────────┘
      (deallocated)
```

---

### Why Reference Counting?

**1. Multiple Ownership**

Some data structures naturally require shared ownership:

```rust
// Graph with cycles
struct Node {
    value: i32,
    neighbors: Vec<Rc<Node>>,  // Multiple edges can point to same node
}

let node_a = Rc::new(Node { value: 1, neighbors: vec![] });
let node_b = Rc::new(Node { value: 2, neighbors: vec![Rc::clone(&node_a)] });
let node_c = Rc::new(Node { value: 3, neighbors: vec![Rc::clone(&node_a)] });

// node_a has 3 owners: itself + node_b + node_c
// All three can access node_a's data
```

**2. Dynamic Lifetime**

Sometimes you don't know which reference will be dropped last:

```rust
// GUI event handlers
struct Button {
    label: Rc<String>,
    on_click: Box<dyn Fn()>,
}

let label = Rc::new(String::from("Submit"));

let button1 = Button {
    label: Rc::clone(&label),
    on_click: Box::new(|| println!("Clicked")),
};

let button2 = Button {
    label: Rc::clone(&label),
    on_click: Box::new(|| println!("Also clicked")),
};

// Don't know if button1 or button2 will be dropped first
// Label stays alive until BOTH are dropped
```

**3. Caching and Deduplication**

Share identical data to save memory:

```rust
// String interning: share common strings
let mut cache: HashMap<&str, Rc<String>> = HashMap::new();

fn intern(cache: &mut HashMap<&str, Rc<String>>, s: &str) -> Rc<String> {
    cache.entry(s)
        .or_insert_with(|| Rc::new(s.to_string()))
        .clone()
}

let s1 = intern(&mut cache, "hello");  // Allocates "hello"
let s2 = intern(&mut cache, "hello");  // Reuses same allocation
// s1 and s2 point to the SAME String
// Saves memory when "hello" appears many times
```

---

### Rc vs Box vs &T

**Box<T>**: Single owner, heap allocation
```rust
let b = Box::new(42);
// Exactly one owner
// Freed when b goes out of scope
// Compile-time ownership tracking
```

**&T**: Borrowed reference, no ownership
```rust
let x = 42;
let r = &x;
// r borrows x
// Compiler ensures x outlives r
// Compile-time borrow checking
```

**Rc<T>**: Multiple owners, heap allocation
```rust
let rc = Rc::new(42);
let rc2 = Rc::clone(&rc);
// Multiple owners
// Freed when BOTH drop
// Runtime reference counting
```

**Comparison**:
```
┌─────────┬────────────┬───────────┬──────────────┬────────────┐
│         │ Ownership  │ Lifetime  │ Safety Check │ Overhead   │
├─────────┼────────────┼───────────┼──────────────┼────────────┤
│ Box<T>  │ Single     │ Scoped    │ Compile-time │ None       │
│ &T      │ Borrowed   │ Scoped    │ Compile-time │ None       │
│ Rc<T>   │ Multiple   │ Dynamic   │ Runtime      │ Refcount   │
└─────────┴────────────┴───────────┴──────────────┴────────────┘
```

---

### The Reference Cycle Problem

**Reference cycles** cause memory leaks with reference counting:

```rust
// MEMORY LEAK: Cycle never freed!
struct Node {
    value: i32,
    next: Option<Rc<RefCell<Node>>>,
}

let node1 = Rc::new(RefCell::new(Node { value: 1, next: None }));
let node2 = Rc::new(RefCell::new(Node { value: 2, next: None }));

// Create cycle: node1 → node2 → node1
node1.borrow_mut().next = Some(Rc::clone(&node2));
node2.borrow_mut().next = Some(Rc::clone(&node1));

// Drop both:
drop(node1);  // node1 refcount: 2 → 1 (still alive! node2 holds ref)
drop(node2);  // node2 refcount: 2 → 1 (still alive! node1 holds ref)

// ❌ MEMORY LEAK: Both nodes have refcount=1, never freed
//    They reference each other, but nothing else references them
```

**Visualization**:
```
After drop(node1) and drop(node2):

   ┌──────────────────┐
   │ Node { value: 1 }│
   │ refcount: 1      │
   │ next: ───────┐   │
   └──────────────│───┘
                  │
                  ↓
   ┌──────────────│───┐
   │ Node { value: 2 }│
   │ refcount: 1      │
   │ next: ───────┐   │
   └──────────────│───┘
                  │
                  └──────────────→ (back to Node 1)

No external references, but refcounts never reach 0!
MEMORY LEAKED: Unreachable but not freed.
```

**Solution: Weak References**

`Weak<T>` doesn't increment the strong count, breaking cycles:

```rust
struct Node {
    value: i32,
    parent: Option<Weak<RefCell<Node>>>,  // ← Weak instead of Rc
    children: Vec<Rc<RefCell<Node>>>,
}

let parent = Rc::new(RefCell::new(Node {
    value: 1,
    parent: None,
    children: vec![],
}));

let child = Rc::new(RefCell::new(Node {
    value: 2,
    parent: Some(Rc::downgrade(&parent)),  // ← Weak reference
    children: vec![],
}));

parent.borrow_mut().children.push(Rc::clone(&child));

// Strong references:
// parent: 1 (our variable)
// child: 2 (our variable + parent's children vec)

// Weak references:
// parent: 1 (child's parent field)

// Drop parent:
drop(parent);  // parent refcount: 1 → 0 → FREED!

// child's weak reference to parent becomes invalid
// child.parent.upgrade() → None

// Drop child:
drop(child);  // child refcount: 1 → 0 → FREED!

// ✓ NO LEAK: All memory freed
```

---

### Strong vs Weak References

**Strong Reference (Rc<T>)**:
- Keeps data alive
- Counted in `strong_count`
- Data freed when `strong_count` reaches 0

**Weak Reference (Weak<T>)**:
- Doesn't keep data alive
- Counted in `weak_count`
- Can become invalid (data freed while weak ref exists)
- Must `upgrade()` to `Rc<T>` to access data

**Reference Counting**:
```rust
struct RcInner<T> {
    strong_count: usize,  // Number of Rc<T> pointers
    weak_count: usize,    // Number of Weak<T> pointers
    data: T,
}

// Rules:
// 1. Data (T) freed when strong_count = 0
// 2. RcInner freed when strong_count = 0 AND weak_count = 0
// 3. Weak refs can exist after data is freed
```

**Example**:
```rust
let strong = Rc::new(String::from("data"));
// strong_count: 1, weak_count: 0

let weak1 = Rc::downgrade(&strong);
// strong_count: 1, weak_count: 1

let weak2 = Rc::downgrade(&strong);
// strong_count: 1, weak_count: 2

let strong2 = weak1.upgrade().unwrap();
// strong_count: 2, weak_count: 2

drop(strong);
// strong_count: 1, weak_count: 2

drop(strong2);
// strong_count: 0 → Data (String) freed!
// weak_count: 2 → RcInner still alive (for weak refs)

weak1.upgrade();  // Returns None (data gone)
weak2.upgrade();  // Returns None

drop(weak1);
// weak_count: 1

drop(weak2);
// weak_count: 0 → RcInner freed!
```

---

### Interior Mutability: RefCell<T>

**The Problem**: `Rc<T>` gives shared references (`&T`), but we often need mutation.

```rust
let rc = Rc::new(vec![1, 2, 3]);
let rc2 = Rc::clone(&rc);

// Can't mutate through shared reference:
// rc.push(4);  // ❌ ERROR: can't call push on &Vec<i32>
```

**Solution: RefCell<T>** provides interior mutability with runtime borrow checking.

```rust
let rc = Rc::new(RefCell::new(vec![1, 2, 3]));
let rc2 = Rc::clone(&rc);

// Borrow mutably at runtime:
rc.borrow_mut().push(4);  // ✓ OK: runtime check passes

println!("{:?}", rc2.borrow());  // [1, 2, 3, 4]
// Both rc and rc2 see the mutation!
```

**How RefCell Works**:

```rust
struct RefCell<T> {
    value: UnsafeCell<T>,      // The actual data (allows mut through &)
    borrow_state: Cell<isize>,  // Tracks borrows
}

// borrow_state values:
//  0: Not borrowed
// >0: N immutable borrows active
// -1: One mutable borrow active
```

**Borrow Checking at Runtime**:
```rust
let cell = RefCell::new(42);

// Multiple immutable borrows OK:
let b1 = cell.borrow();      // borrow_state: 0 → 1
let b2 = cell.borrow();      // borrow_state: 1 → 2
drop(b1);                     // borrow_state: 2 → 1
drop(b2);                     // borrow_state: 1 → 0

// Mutable borrow requires exclusive access:
let mut b = cell.borrow_mut();  // borrow_state: 0 → -1
// cell.borrow();               // ❌ PANIC: already mutably borrowed
drop(b);                         // borrow_state: -1 → 0

// Rules (enforced at runtime):
// 1. Many immutable borrows OR one mutable borrow
// 2. Violation = panic!
```

**Compile-Time vs Runtime**:

```rust
// Compile-time borrow checking (normal Rust):
let mut x = 42;
let r1 = &x;
let r2 = &x;
// let r3 = &mut x;  // ❌ COMPILE ERROR

// Runtime borrow checking (RefCell):
let cell = RefCell::new(42);
let r1 = cell.borrow();
let r2 = cell.borrow();
let r3 = cell.borrow_mut();  // ❌ RUNTIME PANIC!
```

**Trade-offs**:
```
Compile-time (&T, &mut T):
✓ Zero runtime cost
✓ Errors caught at compile time
✗ Can't express some valid patterns

Runtime (RefCell<T>):
✓ More flexible (can mutate through &)
✓ Enables patterns impossible with & alone
✗ Runtime overhead (checking borrows)
✗ Errors found at runtime (panics)
```

---

### The Rc<RefCell<T>> Pattern

Combining `Rc` and `RefCell` enables **shared mutable state**:

```rust
// Pattern: Rc<RefCell<T>>
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data2 = Rc::clone(&data);
let data3 = Rc::clone(&data);

// All three can mutate the same Vec:
data.borrow_mut().push(4);
println!("{:?}", data2.borrow());  // [1, 2, 3, 4]

data2.borrow_mut().push(5);
println!("{:?}", data3.borrow());  // [1, 2, 3, 4, 5]
```

**When to Use**:
- Graphs with mutable nodes
- Observer pattern (multiple observers, mutable subject)
- Cached values that need updating
- Parent-child relationships with mutations

**Example: Tree with Parent Pointers**:
```rust
struct TreeNode {
    value: i32,
    parent: Option<Weak<RefCell<TreeNode>>>,  // Weak to avoid cycle
    children: Vec<Rc<RefCell<TreeNode>>>,     // Strong to keep children alive
}

let root = Rc::new(RefCell::new(TreeNode {
    value: 1,
    parent: None,
    children: vec![],
}));

let child = Rc::new(RefCell::new(TreeNode {
    value: 2,
    parent: Some(Rc::downgrade(&root)),
    children: vec![],
}));

// Add child to parent:
root.borrow_mut().children.push(Rc::clone(&child));

// Mutate child:
child.borrow_mut().value = 42;

// Access parent through child:
if let Some(parent_rc) = child.borrow().parent.as_ref().unwrap().upgrade() {
    println!("Parent value: {}", parent_rc.borrow().value);  // 1
}
```

---

### NonNull<T>: Raw Pointers Done Right

`NonNull<T>` is a wrapper around `*mut T` with two guarantees:
1. Pointer is never null
2. Pointer is properly aligned

**Why Not *mut T?**:
```rust
// *mut T can be null:
let ptr: *mut i32 = std::ptr::null_mut();
// Dangerous: no null check enforced

// NonNull<T> cannot be null:
// let ptr: NonNull<i32> = NonNull::new(std::ptr::null_mut()).unwrap();
// ✗ Panics: new() returns None for null
```

**Benefits**:
```rust
struct MyRc<T> {
    // ptr: *mut RcInner<T>,  // Could be null, not clear ownership
    ptr: NonNull<RcInner<T>>,  // ✓ Never null, covariant, clearer intent
}

// NonNull properties:
// 1. Size: same as *mut T (one usize)
// 2. Null check eliminated (guaranteed non-null)
// 3. Proper variance for T (covariant)
// 4. Explicit unsafe operations
```

**Usage**:
```rust
// Creating NonNull:
let boxed = Box::new(42);
let ptr = NonNull::new(Box::into_raw(boxed)).unwrap();

// Dereferencing (unsafe):
unsafe {
    let value = ptr.as_ref();  // &T
    let value_mut = ptr.as_mut();  // &mut T
    let raw = ptr.as_ptr();  // *mut T
}

// Freeing:
unsafe {
    drop(Box::from_raw(ptr.as_ptr()));
}
```

---

### UnsafeCell<T>: Foundation of Interior Mutability

`UnsafeCell<T>` is the **only** legal way to get `&mut T` from `&UnsafeCell<T>`.

**The Problem**:
```rust
struct Container {
    value: i32,
}

impl Container {
    fn mutate(&self) {
        // ❌ Can't do this: have &self, need &mut self.value
        // self.value = 42;
    }
}
```

**Solution with UnsafeCell**:
```rust
struct Container {
    value: UnsafeCell<i32>,
}

impl Container {
    fn mutate(&self) {
        unsafe {
            *self.value.get() = 42;  // ✓ OK: UnsafeCell allows this
        }
    }
}
```

**Why It's Safe**:
- `UnsafeCell<T>` opts out of Rust's aliasing guarantees
- You promise: no simultaneous `&` and `&mut` to the same data
- RefCell uses UnsafeCell + runtime checks to uphold this

**RefCell Implementation**:
```rust
pub struct RefCell<T> {
    value: UnsafeCell<T>,       // Interior mutability
    borrow_state: Cell<isize>,  // Track borrows
}

impl<T> RefCell<T> {
    pub fn borrow(&self) -> Ref<T> {
        // Check no mutable borrows:
        assert!(self.borrow_state.get() >= 0);

        self.borrow_state.set(self.borrow_state.get() + 1);

        Ref {
            value: unsafe { &*self.value.get() },  // ← UnsafeCell magic
            borrow: &self.borrow_state,
        }
    }
}
```

---

### Connection to This Project

In this project, you'll implement all these concepts:

1. **Milestone 1**: Basic reference counting with `Rc<T>`
   - Heap allocation with `Box`
   - Reference counting with clone/drop
   - NonNull for safe raw pointers
   - PhantomData for variance

2. **Milestone 2**: Weak references to break cycles
   - Two-count system (strong + weak)
   - Conditional freeing logic
   - Safe upgrading from weak to strong

3. **Milestone 3**: Interior mutability with `RefCell<T>`
   - UnsafeCell for legal mutation through &
   - Runtime borrow tracking
   - RAII guards (Ref/RefMut)
   - Panic on violations

---

#### Milestone 1: Basic Reference Counter
**Goal**: Implement a simple `MyRc<T>` with reference counting.

**What to implement**:
- Heap-allocated data with reference count
- Clone to increment count
- Drop to decrement and potentially free

**Key concepts**:
**Structs**: 
- `MyRc<T>`
  - **Field**: `strong_count: usize` - Number of MyRc pointers 
  - **Field**: `data: T` - The actual data                     
- `RcInner<T>`
  - **Field**:  `ptr: NonNull<RcInner<T>>` - Pointer to heap data           
  - **Field**:  `_marker: PhantomData<RcInner<T>>` - Ensure proper variance 
- **Functions**:
    - `new(value: T) -> MyRc<T>` - Allocates and initializes
    - `clone() -> MyRc<T>` - Increments ref count
    - `drop()` - Decrements, frees if zero
    - `strong_count() -> usize` - Returns current count

---




**Starter Code**:

```rust
use std::ops::Deref;
use std::ptr::NonNull;

/// Inner structure holding the data and reference count
struct RcInner<T> {
    strong_count: usize,
    data: T,
}

/// A reference-counted smart pointer
pub struct MyRc<T> {
    ptr: NonNull<RcInner<T>>,
    _marker: std::marker::PhantomData<RcInner<T>>,
}

impl<T> MyRc<T> {
    /// Creates a new reference-counted pointer
    /// Role: Allocate on heap with count=1
    pub fn new(value: T) -> Self {
        todo!("Allocate RcInner on heap")
    }

    /// Returns the current strong reference count
    /// Role: Query reference count
    pub fn strong_count(this: &Self) -> usize {
        todo!("Read strong_count from inner")
    }

    /// Gets a reference to the inner data
    /// Role: Access to inner structure
    fn inner(&self) -> &RcInner<T> {
        todo!("Dereference ptr safely")
    }

    /// Gets a mutable reference to the inner data
    /// Role: Mutable access (requires unique ownership)
    fn inner_mut(&mut self) -> &mut RcInner<T> {
        todo!("Dereference ptr mutably")
    }
}

impl<T> Clone for MyRc<T> {
    /// Clones the Rc by incrementing the reference count
    /// Role: Share ownership
    fn clone(&self) -> Self {
        todo!("Increment count, return new MyRc with same ptr")
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    /// Allows treating MyRc<T> as &T
    /// Role: Transparent access to data
    fn deref(&self) -> &Self::Target {
        todo!("Return reference to data field")
    }
}

impl<T> Drop for MyRc<T> {
    /// Decrements count, frees memory if count reaches zero
    /// Role: Automatic cleanup
    fn drop(&mut self) {
        todo!("Decrement count, deallocate if zero")
    }
}
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rc_creation() {
        let rc = MyRc::new(42);
        assert_eq!(*rc, 42);
        assert_eq!(MyRc::strong_count(&rc), 1);
    }

    #[test]
    fn test_rc_clone_increments_count() {
        let rc1 = MyRc::new(100);
        let rc2 = rc1.clone();
        assert_eq!(MyRc::strong_count(&rc1), 2);
        assert_eq!(MyRc::strong_count(&rc2), 2);
        assert_eq!(*rc1, *rc2);
    }

    #[test]
    fn test_rc_drop_decrements_count() {
        let rc1 = MyRc::new(String::from("hello"));
        {
            let rc2 = rc1.clone();
            assert_eq!(MyRc::strong_count(&rc1), 2);
            drop(rc2);
        }
        assert_eq!(MyRc::strong_count(&rc1), 1);
    }

    #[test]
    fn test_data_freed_when_count_zero() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let dropped = Arc::new(AtomicBool::new(false));

        struct DropDetector {
            flag: Arc<AtomicBool>,
        }

        impl Drop for DropDetector {
            fn drop(&mut self) {
                self.flag.store(true, Ordering::SeqCst);
            }
        }

        {
            let rc = MyRc::new(DropDetector { flag: dropped.clone() });
            let _rc2 = rc.clone();
        }

        assert!(dropped.load(Ordering::SeqCst));
    }
}
```


#### Milestone 2: Weak References
**Goal**: Add weak references to prevent reference cycles.

**Why the previous Milestone is not enough**: Strong references create cycles (e.g., parent->child, child->parent) that never get freed.

**What's the improvement**: `MyWeak<T>` doesn't increment strong count. It can upgrade to `MyRc<T>` if data still exists, or return `None` if data was freed.

**Architecture**:
- **Field**: `weak_count: usize` (add to RcInner)

**Structs**:                                              
- `MyWeak<T>`: Non-owning reference
  - **field**: `ptr: NonNull<RcInner<T>>` - Pointer to heap data

**Functions**:
- `upgrade()` - Try to get MyRc if data alive
- `clone()` - Increment weak count
- `drop()` - Decrement weak count                       



**Starter Code**:

```rust
/// Inner structure now tracks both strong and weak counts
struct RcInner<T> {
    strong_count: usize,
    weak_count: usize,
    data: T,
}

/// A weak reference that doesn't own the data
pub struct MyWeak<T> {
    ptr: NonNull<RcInner<T>>,
    _marker: std::marker::PhantomData<RcInner<T>>,
}

impl<T> MyRc<T> {
    /// Creates a weak reference
    /// Role: Create non-owning pointer
    pub fn downgrade(this: &Self) -> MyWeak<T> {
        todo!("Increment weak_count, create MyWeak")
    }

    /// Returns the weak reference count
    /// Role: Query weak references
    pub fn weak_count(this: &Self) -> usize {
        todo!("Read weak_count")
    }
}

impl<T> MyWeak<T> {
    /// Attempts to upgrade to a strong reference
    /// Role: Convert weak to strong if data exists
    pub fn upgrade(&self) -> Option<MyRc<T>> {
        todo!("Check strong_count > 0, increment and return MyRc")
    }

    /// Returns the strong count if data still exists
    /// Role: Query without upgrading
    pub fn strong_count(&self) -> usize {
        todo!("Read strong_count")
    }
}

impl<T> Clone for MyWeak<T> {
    /// Clone the weak reference
    /// Role: Share weak reference
    fn clone(&self) -> Self {
        todo!("Increment weak_count")
    }
}

impl<T> Drop for MyWeak<T> {
    /// Decrement weak count, free RcInner if both counts are zero
    /// Role: Cleanup weak reference
    fn drop(&mut self) {
        todo!("Decrement weak_count, free if strong=0 and weak=0")
    }
}

// Update MyRc::drop to only free data when strong=0,
// but keep RcInner alive if weak_count > 0
```

---
**Checkpoint Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_creation() {
        let rc = MyRc::new(42);
        let weak = MyRc::downgrade(&rc);
        assert_eq!(MyRc::weak_count(&rc), 1);
        assert_eq!(MyRc::strong_count(&rc), 1);
    }

    #[test]
    fn test_weak_upgrade_success() {
        let rc = MyRc::new(String::from("data"));
        let weak = MyRc::downgrade(&rc);

        let upgraded = weak.upgrade();
        assert!(upgraded.is_some());
        assert_eq!(*upgraded.unwrap(), "data");
    }

    #[test]
    fn test_weak_upgrade_fails_after_drop() {
        let weak = {
            let rc = MyRc::new(100);
            let weak = MyRc::downgrade(&rc);
            weak
        }; // rc dropped here

        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_weak_doesnt_keep_alive() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let dropped = Arc::new(AtomicBool::new(false));

        struct DropDetector {
            flag: Arc<AtomicBool>,
        }

        impl Drop for DropDetector {
            fn drop(&mut self) {
                self.flag.store(true, Ordering::SeqCst);
            }
        }

        let weak = {
            let rc = MyRc::new(DropDetector { flag: dropped.clone() });
            MyRc::downgrade(&rc)
        };

        assert!(dropped.load(Ordering::SeqCst));
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_break_cycle() {
        use std::cell::RefCell;

        struct Node {
            parent: Option<MyWeak<RefCell<Node>>>,
            children: Vec<MyRc<RefCell<Node>>>,
        }

        let parent = MyRc::new(RefCell::new(Node {
            parent: None,
            children: vec![],
        }));

        let child = MyRc::new(RefCell::new(Node {
            parent: Some(MyRc::downgrade(&parent)),
            children: vec![],
        }));

        parent.borrow_mut().children.push(child.clone());

        // No cycle - weak ref breaks it
        assert_eq!(MyRc::strong_count(&parent), 1);
        assert_eq!(MyRc::strong_count(&child), 2);
    }
}
```

---

#### Milestone 3: Interior Mutability with RefCell
**Goal**: Combine `MyRc` with interior mutability to allow mutation through shared references.

**Why the previous Milestone is not enough**: `MyRc` gives shared `&T` references. We need `&mut T` even with multiple owners.

**What's the improvement**: Implement `MyRefCell<T>` with runtime borrow checking. Track borrows at runtime and panic on violations.

**Architecture**:
**Structs**:
- `MyRefCell<T>`: Interior mutability container
  - `value: UnsafeCell<T>` - Actual data
  - `borrow_state: Cell<isize>` - >0: N immutable borrows, -1: mutable borrow
- `Ref<'a, T>`: Immutable borrow guard
- `RefMut<'a, T>`: Mutable borrow guard

**Functions**:
- `new(value) `- Create new RefCell
- `borrow()` - Get Ref<T>, panic if mutably borrowed
- `borrow_mut()` - Get RefMut<T>, panic if any borrows exist
- `try_borrow()` - Non-panicking version
- `try_borrow_mut()` - Non-panicking version                                      

---


---

**Starter Code**:

```rust
use std::cell::{Cell, UnsafeCell};

/// A cell with runtime borrow checking
pub struct MyRefCell<T> {
    value: UnsafeCell<T>,
    borrow_state: Cell<isize>,
}

pub struct Ref<'a, T> {
    value: &'a T,
    borrow: &'a Cell<isize>,
}

pub struct RefMut<'a, T> {
    value: &'a mut T,
    borrow: &'a Cell<isize>,
}

impl<T> MyRefCell<T> {
    /// Creates a new RefCell
    /// Role: Initialize with borrow_state=0
    pub fn new(value: T) -> Self {
        todo!("Initialize UnsafeCell and borrow state")
    }

    /// Borrows the value immutably
    /// Role: Increment borrow count, return Ref
    pub fn borrow(&self) -> Ref<T> {
        todo!("Check not mutably borrowed, increment count")
    }

    /// Borrows the value mutably
    /// Role: Set borrow state to -1, return RefMut
    pub fn borrow_mut(&self) -> RefMut<T> {
        todo!("Check no borrows exist, set state=-1")
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    /// Role: Transparent access
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> Drop for Ref<'_, T> {
    /// Decrements borrow count
    /// Role: Release immutable borrow
    fn drop(&mut self) {
        todo!("Decrement borrow_state")
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    /// Role: Transparent access
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    /// Role: Mutable access
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T> Drop for RefMut<'_, T> {
    /// Resets borrow state to 0
    /// Role: Release mutable borrow
    fn drop(&mut self) {
        todo!("Set borrow_state to 0")
    }
}

// Safety: MyRefCell can be Send if T is Send
unsafe impl<T: Send> Send for MyRefCell<T> {}
// Note: MyRefCell is NOT Sync - can't share &MyRefCell across threads
```

---

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refcell_basic_borrow() {
        let cell = MyRefCell::new(42);
        let borrowed = cell.borrow();
        assert_eq!(*borrowed, 42);
    }

    #[test]
    fn test_refcell_multiple_immutable() {
        let cell = MyRefCell::new(100);
        let b1 = cell.borrow();
        let b2 = cell.borrow();
        assert_eq!(*b1, *b2);
    }

    #[test]
    fn test_refcell_mutable_borrow() {
        let cell = MyRefCell::new(String::from("hello"));
        {
            let mut borrowed = cell.borrow_mut();
            borrowed.push_str(" world");
        }
        assert_eq!(&*cell.borrow(), "hello world");
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_refcell_panic_on_double_mut() {
        let cell = MyRefCell::new(42);
        let _b1 = cell.borrow_mut();
        let _b2 = cell.borrow_mut(); // Should panic
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_refcell_panic_mut_while_immutable() {
        let cell = MyRefCell::new(42);
        let _b1 = cell.borrow();
        let _b2 = cell.borrow_mut(); // Should panic
    }

    #[test]
    fn test_rc_refcell_combination() {
        let data = MyRc::new(MyRefCell::new(vec![1, 2, 3]));
        let data2 = data.clone();

        data.borrow_mut().push(4);
        assert_eq!(*data2.borrow(), vec![1, 2, 3, 4]);
    }
}
```

### Complete Working Example

```rust
// Complete Reference-Counted Smart Pointer Implementation
// Implements custom Rc<T>, Weak<T>, and RefCell<T>

use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// ============================================================================
// Milestone 1: Basic Reference Counting (MyRc<T>)
// ============================================================================

struct RcInner<T> {
    strong_count: usize,
    weak_count: usize,
    data: ManuallyDrop<T>,
}

pub struct MyRc<T> {
    ptr: NonNull<RcInner<T>>,
    _marker: PhantomData<RcInner<T>>,
}

impl<T> MyRc<T> {
    pub fn new(value: T) -> Self {
        let inner = Box::new(RcInner {
            strong_count: 1,
            weak_count: 0,
            data: ManuallyDrop::new(value),
        });

        MyRc {
            ptr: NonNull::new(Box::into_raw(inner)).unwrap(),
            _marker: PhantomData,
        }
    }

    pub fn strong_count(this: &Self) -> usize {
        this.inner().strong_count
    }

    pub fn weak_count(this: &Self) -> usize {
        this.inner().weak_count
    }

    fn inner(&self) -> &RcInner<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn downgrade(this: &Self) -> MyWeak<T> {
        unsafe {
            let inner = this.ptr.as_ptr();
            (*inner).weak_count += 1;
        }

        MyWeak {
            ptr: this.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for MyRc<T> {
    fn clone(&self) -> Self {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).strong_count += 1;
        }

        MyRc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for MyRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner().data
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).strong_count -= 1;

            if (*inner).strong_count == 0 {
                // Snapshot weak_count before dropping data
                let had_weak_refs = (*inner).weak_count > 0;

                // Temporarily increment weak_count to prevent deallocation during data drop
                // This ensures weak refs dropped during data destruction don't free the RcInner
                (*inner).weak_count += 1;

                // Always drop the data
                ManuallyDrop::drop(&mut (*inner).data);

                // Decrement the temporary weak_count
                (*inner).weak_count -= 1;

                // Only deallocate if there were originally no weak refs AND none remain
                if !had_weak_refs && (*inner).weak_count == 0 {
                    drop(Box::from_raw(inner));
                }
            }
        }
    }
}

// ============================================================================
// Milestone 2: Weak References (MyWeak<T>)
// ============================================================================

pub struct MyWeak<T> {
    ptr: NonNull<RcInner<T>>,
    _marker: PhantomData<RcInner<T>>,
}

impl<T> MyWeak<T> {
    pub fn upgrade(&self) -> Option<MyRc<T>> {
        unsafe {
            let inner = self.ptr.as_ptr();

            if (*inner).strong_count == 0 {
                None
            } else {
                (*inner).strong_count += 1;
                Some(MyRc {
                    ptr: self.ptr,
                    _marker: PhantomData,
                })
            }
        }
    }

    pub fn strong_count(&self) -> usize {
        unsafe { (*self.ptr.as_ptr()).strong_count }
    }

    pub fn weak_count(&self) -> usize {
        unsafe { (*self.ptr.as_ptr()).weak_count }
    }
}

impl<T> Clone for MyWeak<T> {
    fn clone(&self) -> Self {
        unsafe {
            (*self.ptr.as_ptr()).weak_count += 1;
        }

        MyWeak {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for MyWeak<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).weak_count -= 1;

            // Only free RcInner if both counts are zero
            if (*inner).strong_count == 0 && (*inner).weak_count == 0 {
                drop(Box::from_raw(inner));
            }
        }
    }
}

// ============================================================================
// Milestone 3: Interior Mutability (MyRefCell<T>)
// ============================================================================

pub struct MyRefCell<T> {
    value: UnsafeCell<T>,
    borrow_state: Cell<isize>,
}

pub struct Ref<'a, T> {
    value: &'a T,
    borrow: &'a Cell<isize>,
}

pub struct RefMut<'a, T> {
    value: &'a mut T,
    borrow: &'a Cell<isize>,
}

impl<T> MyRefCell<T> {
    pub fn new(value: T) -> Self {
        MyRefCell {
            value: UnsafeCell::new(value),
            borrow_state: Cell::new(0),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        let state = self.borrow_state.get();

        if state < 0 {
            panic!("already mutably borrowed");
        }

        self.borrow_state.set(state + 1);

        Ref {
            value: unsafe { &*self.value.get() },
            borrow: &self.borrow_state,
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        let state = self.borrow_state.get();

        if state != 0 {
            panic!("already borrowed");
        }

        self.borrow_state.set(-1);

        RefMut {
            value: unsafe { &mut *self.value.get() },
            borrow: &self.borrow_state,
        }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        let state = self.borrow.get();
        self.borrow.set(state - 1);
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        self.borrow.set(0);
    }
}

unsafe impl<T: Send> Send for MyRefCell<T> {}

// ============================================================================
// Main Function - Demonstrates All Components
// ============================================================================

fn main() {
    println!("=== Reference-Counted Smart Pointer ===\n");

    // Milestone 1: Basic Reference Counting
    println!("--- Part 1: Basic Reference Counting ---");
    {
        let rc1 = MyRc::new(42);
        println!("rc1: {}, count: {}", *rc1, MyRc::strong_count(&rc1));

        let rc2 = rc1.clone();
        println!("After clone, count: {}", MyRc::strong_count(&rc1));
        println!("rc2: {}, count: {}", *rc2, MyRc::strong_count(&rc2));

        drop(rc2);
        println!("After drop rc2, count: {}", MyRc::strong_count(&rc1));
    }
    println!();

    // Milestone 2: Weak References
    println!("--- Part 2: Weak References ---");
    {
        let strong = MyRc::new(String::from("data"));
        let weak = MyRc::downgrade(&strong);

        println!("Strong count: {}", MyRc::strong_count(&strong));
        println!("Weak count: {}", MyRc::weak_count(&strong));

        if let Some(upgraded) = weak.upgrade() {
            println!("Upgraded: {}", *upgraded);
        }

        drop(strong);

        if weak.upgrade().is_none() {
            println!("Upgrade failed - data was dropped");
        }
    }
    println!();

    // Milestone 3: Breaking Reference Cycles
    println!("--- Part 3: Breaking Cycles with Weak ---");
    {
        struct Node {
            value: i32,
            parent: Option<MyWeak<MyRefCell<Node>>>,
            children: Vec<MyRc<MyRefCell<Node>>>,
        }

        let parent = MyRc::new(MyRefCell::new(Node {
            value: 1,
            parent: None,
            children: vec![],
        }));

        let child = MyRc::new(MyRefCell::new(Node {
            value: 2,
            parent: Some(MyRc::downgrade(&parent)),
            children: vec![],
        }));

        parent.borrow_mut().children.push(child.clone());

        println!("Parent value: {}", (*parent.borrow()).value);
        println!("Child value: {}", (*child.borrow()).value);

        // Access parent through child's weak reference
        let child_borrow = child.borrow();
        if let Some(weak_ref) = &child_borrow.parent {
            if let Some(parent_rc) = weak_ref.upgrade() {
                println!("Child's parent value: {}", (*parent_rc.borrow()).value);
            }
        }
    }
    println!();

    // Milestone 4: Rc<RefCell<T>> Pattern
    println!("--- Part 4: Rc<RefCell<T>> Pattern ---");
    {
        let data = MyRc::new(MyRefCell::new(vec![1, 2, 3]));
        let data2 = data.clone();
        let data3 = data.clone();

        println!("Initial: {:?}", *data.borrow());

        data.borrow_mut().push(4);
        println!("After data.push(4): {:?}", *data2.borrow());

        data2.borrow_mut().push(5);
        println!("After data2.push(5): {:?}", *data3.borrow());
    }
    println!();

    // Milestone 5: RefCell Borrow Checking
    println!("--- Part 5: RefCell Borrow Checking ---");
    {
        let cell = MyRefCell::new(100);

        {
            let b1 = cell.borrow();
            let b2 = cell.borrow();
            println!("Multiple immutable borrows: {} and {}", *b1, *b2);
        }

        {
            let mut b = cell.borrow_mut();
            *b += 50;
            println!("Mutable borrow, new value: {}", *b);
        }

        println!("Final value: {}", *cell.borrow());
    }
    println!();

    println!("=== All Components Complete! ===");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1: MyRc Tests
    #[test]
    fn test_rc_basic() {
        let rc = MyRc::new(42);
        assert_eq!(*rc, 42);
        assert_eq!(MyRc::strong_count(&rc), 1);
    }

    #[test]
    fn test_rc_clone() {
        let rc1 = MyRc::new(100);
        let rc2 = rc1.clone();
        assert_eq!(MyRc::strong_count(&rc1), 2);
        assert_eq!(*rc1, *rc2);
    }

    #[test]
    fn test_rc_multiple_clones() {
        let rc1 = MyRc::new(String::from("hello"));
        let rc2 = rc1.clone();
        let rc3 = rc1.clone();
        let rc4 = rc2.clone();

        assert_eq!(MyRc::strong_count(&rc1), 4);
        assert_eq!(*rc1, "hello");
        assert_eq!(*rc2, "hello");
        assert_eq!(*rc3, "hello");
        assert_eq!(*rc4, "hello");
    }

    #[test]
    fn test_rc_drop() {
        let rc1 = MyRc::new(42);
        let rc2 = rc1.clone();
        let rc3 = rc1.clone();

        assert_eq!(MyRc::strong_count(&rc1), 3);
        drop(rc2);
        assert_eq!(MyRc::strong_count(&rc1), 2);
        drop(rc3);
        assert_eq!(MyRc::strong_count(&rc1), 1);
    }

    // Milestone 2: MyWeak Tests
    #[test]
    fn test_weak_upgrade() {
        let strong = MyRc::new(42);
        let weak = MyRc::downgrade(&strong);

        assert!(weak.upgrade().is_some());
        assert_eq!(*weak.upgrade().unwrap(), 42);

        drop(strong);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_weak_counts() {
        let strong = MyRc::new(100);
        assert_eq!(MyRc::strong_count(&strong), 1);
        assert_eq!(MyRc::weak_count(&strong), 0);

        let weak1 = MyRc::downgrade(&strong);
        assert_eq!(MyRc::strong_count(&strong), 1);
        assert_eq!(MyRc::weak_count(&strong), 1);

        let _weak2 = weak1.clone();
        assert_eq!(MyRc::weak_count(&strong), 2);

        drop(weak1);
        assert_eq!(MyRc::weak_count(&strong), 1);
    }

    #[test]
    fn test_weak_survives_strong_drop() {
        let strong = MyRc::new(String::from("data"));
        let weak = MyRc::downgrade(&strong);

        assert_eq!(weak.strong_count(), 1);
        drop(strong);
        assert_eq!(weak.strong_count(), 0);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_multiple_weak_refs() {
        let strong = MyRc::new(42);
        let weak1 = MyRc::downgrade(&strong);
        let weak2 = MyRc::downgrade(&strong);
        let weak3 = weak1.clone();

        assert_eq!(MyRc::weak_count(&strong), 3);

        assert_eq!(*weak1.upgrade().unwrap(), 42);
        assert_eq!(*weak2.upgrade().unwrap(), 42);
        assert_eq!(*weak3.upgrade().unwrap(), 42);
    }

    // Milestone 3: MyRefCell Tests
    #[test]
    fn test_refcell_borrow() {
        let cell = MyRefCell::new(42);
        let b1 = cell.borrow();
        let b2 = cell.borrow();
        assert_eq!(*b1, *b2);
        assert_eq!(*b1, 42);
    }

    #[test]
    fn test_refcell_borrow_mut() {
        let cell = MyRefCell::new(42);
        *cell.borrow_mut() = 100;
        assert_eq!(*cell.borrow(), 100);
    }

    #[test]
    fn test_refcell_sequential_borrows() {
        let cell = MyRefCell::new(0);

        {
            let b = cell.borrow();
            assert_eq!(*b, 0);
        }

        {
            let mut b = cell.borrow_mut();
            *b = 10;
        }

        {
            let b = cell.borrow();
            assert_eq!(*b, 10);
        }
    }

    #[test]
    #[should_panic(expected = "already mutably borrowed")]
    fn test_refcell_panic_immut_while_mut() {
        let cell = MyRefCell::new(42);
        let _b1 = cell.borrow_mut();
        let _b2 = cell.borrow(); // Should panic
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_refcell_panic_mut_while_immut() {
        let cell = MyRefCell::new(42);
        let _b1 = cell.borrow();
        let _b2 = cell.borrow_mut(); // Should panic
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_refcell_panic_mut_while_mut() {
        let cell = MyRefCell::new(42);
        let _b1 = cell.borrow_mut();
        let _b2 = cell.borrow_mut(); // Should panic
    }

    #[test]
    fn test_refcell_multiple_immutable_borrows() {
        let cell = MyRefCell::new(vec![1, 2, 3]);
        let b1 = cell.borrow();
        let b2 = cell.borrow();
        let b3 = cell.borrow();

        assert_eq!(*b1, vec![1, 2, 3]);
        assert_eq!(*b2, vec![1, 2, 3]);
        assert_eq!(*b3, vec![1, 2, 3]);
    }

    #[test]
    fn test_refcell_into_inner() {
        let cell = MyRefCell::new(42);
        *cell.borrow_mut() = 100;
        assert_eq!(cell.into_inner(), 100);
    }

    // Combined Tests
    #[test]
    fn test_rc_refcell_pattern() {
        let data = MyRc::new(MyRefCell::new(vec![1, 2, 3]));
        let data2 = data.clone();

        data.borrow_mut().push(4);
        assert_eq!(*data2.borrow(), vec![1, 2, 3, 4]);

        data2.borrow_mut().push(5);
        assert_eq!(*data.borrow(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_tree_structure() {
        struct TreeNode {
            value: i32,
            parent: Option<MyWeak<MyRefCell<TreeNode>>>,
            children: Vec<MyRc<MyRefCell<TreeNode>>>,
        }

        let root = MyRc::new(MyRefCell::new(TreeNode {
            value: 1,
            parent: None,
            children: vec![],
        }));

        let child1 = MyRc::new(MyRefCell::new(TreeNode {
            value: 2,
            parent: Some(MyRc::downgrade(&root)),
            children: vec![],
        }));

        let child2 = MyRc::new(MyRefCell::new(TreeNode {
            value: 3,
            parent: Some(MyRc::downgrade(&root)),
            children: vec![],
        }));

        root.borrow_mut().children.push(child1.clone());
        root.borrow_mut().children.push(child2.clone());

        assert_eq!((*root.borrow()).value, 1);
        assert_eq!((*root.borrow()).children.len(), 2);

        // Access parent through child
        let child1_borrow = child1.borrow();
        let parent = child1_borrow.parent.as_ref().unwrap().upgrade().unwrap();
        assert_eq!((*parent.borrow()).value, 1);
    }

    #[test]
    fn test_drop_behavior() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let drop_count = Arc::new(AtomicUsize::new(0));

        struct DropCounter {
            count: Arc<AtomicUsize>,
        }

        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.count.fetch_add(1, Ordering::SeqCst);
            }
        }

        {
            let rc1 = MyRc::new(DropCounter {
                count: drop_count.clone(),
            });
            let rc2 = rc1.clone();
            let rc3 = rc1.clone();

            assert_eq!(drop_count.load(Ordering::SeqCst), 0);
            drop(rc1);
            assert_eq!(drop_count.load(Ordering::SeqCst), 0);
            drop(rc2);
            assert_eq!(drop_count.load(Ordering::SeqCst), 0);
            drop(rc3);
        }

        assert_eq!(drop_count.load(Ordering::SeqCst), 1);
    }
}

```
