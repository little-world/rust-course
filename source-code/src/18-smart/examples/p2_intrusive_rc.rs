// Pattern 2: Intrusive Reference Counting
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

fn main() {
    // Usage: Single allocation for data + refcount
    let rc1 = IntrusiveRc::new(String::from("hello"));
    let _rc2 = rc1.clone();
    assert_eq!(rc1.refcount(), 2);
    assert_eq!(*rc1, "hello");

    println!("Intrusive Rc example completed");
}
