// Complete Reference-Counted Smart Pointer Implementation
// Implements custom Rc<T>, Weak<T>, and RefCell<T>

use std::cell::{Cell, UnsafeCell};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// ============================================================================
// Part 1: Basic Reference Counting (MyRc<T>)
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
// Part 2: Weak References (MyWeak<T>)
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
// Part 3: Interior Mutability (MyRefCell<T>)
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

    // Part 1: Basic Reference Counting
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

    // Part 2: Weak References
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

    // Part 3: Breaking Reference Cycles
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

    // Part 4: Rc<RefCell<T>> Pattern
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

    // Part 5: RefCell Borrow Checking
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

    // Part 1: MyRc Tests
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

    // Part 2: MyWeak Tests
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

    // Part 3: MyRefCell Tests
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
