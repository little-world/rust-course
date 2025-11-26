## Project 5: Reference-Counted Smart Pointer

### Problem Statement

Implement a custom reference-counted smart pointer (similar to `Rc<T>`) that allows multiple ownership of heap-allocated data. Your implementation should automatically free memory when the last reference is dropped and provide interior mutability through `RefCell`-like semantics.

Your smart pointer should support:
- Multiple owners sharing the same data
- Automatic cleanup when reference count reaches zero
- Weak references to break reference cycles
- Interior mutability patterns
- Clone-on-write optimization

### Why It Matters

Reference counting is fundamental to memory management in languages without garbage collection. Understanding `Rc` and `Arc` teaches you about shared ownership, reference cycles, and the trade-offs between compile-time and runtime safety. These patterns appear in GUI frameworks, graph structures, caches, and any system where data has multiple owners.

### Use Cases

- GUI frameworks: Widgets sharing application state
- Graph structures: Nodes with multiple incoming edges
- Caching: Multiple references to cached data
- Event systems: Subscribers sharing event data
- Plugin systems: Shared configuration across plugins

### Solution Outline

#### Milestone 1: Basic Reference Counter
**Goal**: Implement a simple `MyRc<T>` with reference counting.

**What to implement**:
- Heap-allocated data with reference count
- Clone to increment count
- Drop to decrement and potentially free

**Key concepts**:
- Structs: `MyRc<T>`, `RcInner<T>`
- Fields: `ptr: *mut RcInner<T>`, `strong_count: usize`, `data: T`
- Functions:
    - `new(value: T) -> MyRc<T>` - Allocates and initializes
    - `clone() -> MyRc<T>` - Increments ref count
    - `drop()` - Decrements, frees if zero
    - `strong_count() -> usize` - Returns current count

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

---

**Starter Code**:

```rust
use std::ops::Deref;
use std::ptr::NonNull;

/// Inner structure holding the data and reference count
///
/// Structs:
/// - RcInner<T>: Heap-allocated reference counted data
///
/// Fields:
/// - strong_count: usize - Number of MyRc pointers
/// - data: T - The actual data
struct RcInner<T> {
    strong_count: usize,
    data: T,
}

/// A reference-counted smart pointer
///
/// Structs:
/// - MyRc<T>: Smart pointer with shared ownership
///
/// Fields:
/// - ptr: NonNull<RcInner<T>> - Pointer to heap data
/// - _marker: PhantomData<RcInner<T>> - Ensure proper variance
///
/// Functions:
/// - new(value: T) - Creates new Rc with count=1
/// - clone() - Increments count, returns new Rc
/// - strong_count() - Returns current reference count
/// - drop() - Decrements count, frees if zero
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

#### Milestone 2: Weak References
**Goal**: Add weak references to prevent reference cycles.

**Why the previous Milestone is not enough**: Strong references create cycles (e.g., parent->child, child->parent) that never get freed.

**What's the improvement**: `MyWeak<T>` doesn't increment strong count. It can upgrade to `MyRc<T>` if data still exists, or return `None` if data was freed.

**Key concepts**:
- Structs: `MyWeak<T>`
- Fields: `weak_count: usize` (add to RcInner)
- Functions:
    - `MyRc::downgrade() -> MyWeak<T>` - Creates weak ref
    - `MyWeak::upgrade() -> Option<MyRc<T>>` - Try to get strong ref
    - `weak_count()` - Returns weak reference count

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

**Starter Code**:

```rust
/// Inner structure now tracks both strong and weak counts
///
/// Fields:
/// - strong_count: usize - Number of MyRc pointers
/// - weak_count: usize - Number of MyWeak pointers
/// - data: T - The actual data
struct RcInner<T> {
    strong_count: usize,
    weak_count: usize,
    data: T,
}

/// A weak reference that doesn't own the data
///
/// Structs:
/// - MyWeak<T>: Non-owning reference
///
/// Fields:
/// - ptr: NonNull<RcInner<T>> - Pointer to heap data
///
/// Functions:
/// - upgrade() - Try to get MyRc if data alive
/// - clone() - Increment weak count
/// - drop() - Decrement weak count
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

#### Milestone 3: Interior Mutability with RefCell
**Goal**: Combine `MyRc` with interior mutability to allow mutation through shared references.

**Why the previous Milestone is not enough**: `MyRc` gives shared `&T` references. We need `&mut T` even with multiple owners.

**What's the improvement**: Implement `MyRefCell<T>` with runtime borrow checking. Track borrows at runtime and panic on violations.

**Key concepts**:
- Structs: `MyRefCell<T>`, `Ref<T>`, `RefMut<T>`
- Fields: `value: UnsafeCell<T>`, `borrow_state: Cell<isize>`
- Functions:
    - `borrow() -> Ref<T>` - Get immutable reference
    - `borrow_mut() -> RefMut<T>` - Get mutable reference
    - Ref/RefMut implement Deref and update borrow state on drop

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

---

**Starter Code**:

```rust
use std::cell::{Cell, UnsafeCell};

/// A cell with runtime borrow checking
///
/// Structs:
/// - MyRefCell<T>: Interior mutability container
/// - Ref<'a, T>: Immutable borrow guard
/// - RefMut<'a, T>: Mutable borrow guard
///
/// MyRefCell Fields:
/// - value: UnsafeCell<T> - Actual data
/// - borrow_state: Cell<isize> - >0: N immutable borrows, -1: mutable borrow
///
/// Functions:
/// - new(value) - Create new RefCell
/// - borrow() - Get Ref<T>, panic if mutably borrowed
/// - borrow_mut() - Get RefMut<T>, panic if any borrows exist
/// - try_borrow() - Non-panicking version
/// - try_borrow_mut() - Non-panicking version
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

### Testing Strategies

1. **Reference Counting Tests**:
    - Verify counts increment/decrement correctly
    - Test that memory is freed when count reaches zero
    - Use drop detectors to verify cleanup

2. **Weak Reference Tests**:
    - Test upgrade succeeds while data alive
    - Test upgrade fails after data dropped
    - Verify weak refs don't keep data alive
    - Test breaking reference cycles

3. **Interior Mutability Tests**:
    - Test multiple immutable borrows work
    - Test mutable borrow is exclusive
    - Verify panics on borrow violations
    - Test Rc<RefCell<T>> combination

4. **Memory Safety**:
    - Use Miri to detect undefined behavior
    - Test with AddressSanitizer
    - Verify no use-after-free
    - Test thread safety boundaries

---

### Complete Working Example

```rust
use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

//==============================================================================
// Part 1: Basic Reference Counting
//==============================================================================

struct RcInner<T> {
    strong_count: usize,
    weak_count: usize,
    data: T,
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
            data: value,
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

    fn inner_mut(&mut self) -> &mut RcInner<T> {
        unsafe { self.ptr.as_mut() }
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
        &self.inner().data
    }
}

impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ptr();
            (*inner).strong_count -= 1;

            if (*inner).strong_count == 0 {
                // Drop the data
                std::ptr::drop_in_place(&mut (*inner).data);

                // If no weak references, free the entire RcInner
                if (*inner).weak_count == 0 {
                    drop(Box::from_raw(inner));
                }
            }
        }
    }
}

//==============================================================================
// Part 2: Weak References
//==============================================================================

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

//==============================================================================
// Part 3: Interior Mutability
//==============================================================================

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

    pub fn borrow(&self) -> Ref<T> {
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

    pub fn borrow_mut(&self) -> RefMut<T> {
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

//==============================================================================
// Example Usage
//==============================================================================

fn main() {
    println!("=== Reference Counting Examples ===\n");

    // Example 1: Basic Rc
    println!("Example 1: Basic Reference Counting");
    {
        let rc1 = MyRc::new(42);
        println!("rc1: {}, count: {}", *rc1, MyRc::strong_count(&rc1));

        let rc2 = rc1.clone();
        println!("After clone, count: {}", MyRc::strong_count(&rc1));

        drop(rc2);
        println!("After drop rc2, count: {}", MyRc::strong_count(&rc1));
    }
    println!();

    // Example 2: Weak references
    println!("Example 2: Weak References");
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

    // Example 3: Breaking reference cycles
    println!("Example 3: Breaking Cycles with Weak");
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

        println!("Parent value: {}", parent.borrow().value);
        println!("Child value: {}", child.borrow().value);

        // Access parent through child's weak reference
        if let Some(parent_rc) = child.borrow().parent.as_ref().unwrap().upgrade() {
            println!("Child's parent value: {}", parent_rc.borrow().value);
        }
    }
    println!();

    // Example 4: Rc<RefCell<T>> pattern
    println!("Example 4: Rc<RefCell<T>> Pattern");
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

    // Example 5: RefCell borrow checking
    println!("Example 5: RefCell Borrow Checking");
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_weak_upgrade() {
        let strong = MyRc::new(42);
        let weak = MyRc::downgrade(&strong);

        assert!(weak.upgrade().is_some());
        drop(strong);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_refcell_borrow() {
        let cell = MyRefCell::new(42);
        let b1 = cell.borrow();
        let b2 = cell.borrow();
        assert_eq!(*b1, *b2);
    }

    #[test]
    fn test_refcell_borrow_mut() {
        let cell = MyRefCell::new(42);
        *cell.borrow_mut() = 100;
        assert_eq!(*cell.borrow(), 100);
    }

    #[test]
    #[should_panic]
    fn test_refcell_panic() {
        let cell = MyRefCell::new(42);
        let _b1 = cell.borrow();
        let _b2 = cell.borrow_mut(); // Should panic
    }
}
```

This complete example demonstrates:
- **Part 1**: Custom `Rc<T>` with reference counting
- **Part 2**: `Weak<T>` references to break cycles
- **Part 3**: `RefCell<T>` for interior mutability
- **Examples**: Real-world patterns like parent-child relationships
- **Tests**: Comprehensive verification of behavior

The implementation teaches fundamental concepts of memory management, ownership, and the runtime vs compile-time safety trade-offs in Rust.
