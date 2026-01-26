// Pattern 5: PhantomData for Type Safety
use std::marker::PhantomData;
use std::ptr::NonNull;

/// A raw pointer wrapper that tracks ownership.
pub struct RawPtr<T> {
    ptr: NonNull<T>,
    // PhantomData tells compiler we "own" a T
    _marker: PhantomData<T>,
}

impl<T> RawPtr<T> {
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        RawPtr {
            ptr: NonNull::new(Box::into_raw(boxed)).unwrap(),
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for RawPtr<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.ptr.as_ptr()));
        }
    }
}

// Safe because we own the T
unsafe impl<T: Send> Send for RawPtr<T> {}
unsafe impl<T: Sync> Sync for RawPtr<T> {}

fn main() {
    // Usage: Owned heap value with safe API
    let p = RawPtr::new(42);
    println!("Value: {}", p.as_ref());

    // With a more complex type
    let mut s = RawPtr::new(String::from("Hello, PhantomData!"));
    println!("String: {}", s.as_ref());

    // Mutation through as_mut
    s.as_mut().push_str(" Modified.");
    println!("Modified: {}", s.as_ref());

    println!("PhantomData example completed");
}
