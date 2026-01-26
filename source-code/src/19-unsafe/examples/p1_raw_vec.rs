// Pattern 1: Building a Raw Vec-like Structure
use std::alloc::{alloc, dealloc, realloc, Layout};

/// Manages raw memory allocation for a vector.
/// Does NOT track which elements are initialized!
pub struct RawVec<T> {
    ptr: *mut T,     // Pointer to allocated memory
    cap: usize,      // Capacity (number of T that fit)
}

impl<T> RawVec<T> {
    /// Creates an empty RawVec with no allocation.
    pub fn new() -> Self {
        RawVec {
            ptr: std::ptr::null_mut(),  // null_mut() is a safe operation
            cap: 0,
        }
    }

    /// Allocates memory for `cap` elements.
    pub fn with_capacity(cap: usize) -> Self {
        let layout = Layout::array::<T>(cap).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };

        if ptr.is_null() {
            panic!("Allocation failed");
        }

        RawVec { ptr, cap }
    }

    /// Doubles capacity, or sets it to 1 if currently zero.
    pub fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                realloc(
                    self.ptr as *mut u8,  // realloc works with u8 pointers
                    old_layout,
                    new_layout.size()
                ) as *mut T
            }
        };

        if new_ptr.is_null() {
            panic!("Allocation failed");
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }

    pub fn ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn cap(&self) -> usize {
        self.cap
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

fn main() {
    let mut v: RawVec<i32> = RawVec::with_capacity(10);
    println!("Initial capacity: {}", v.cap());
    v.grow(); // Doubles capacity to 20
    println!("After grow: {}", v.cap());

    println!("RawVec example completed");
}
