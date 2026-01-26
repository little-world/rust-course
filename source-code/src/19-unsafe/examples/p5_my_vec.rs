// Pattern 5: Building a Safe Vec
use std::ptr;
use std::alloc::{alloc, realloc, dealloc, Layout};

pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}

impl<T> MyVec<T> {
    /// Creates an empty vector.
    pub fn new() -> Self {
        MyVec {
            ptr: std::ptr::null_mut(),  // Null is fine when cap == 0
            len: 0,
            cap: 0,
        }
    }

    /// Adds an element to the end of the vector.
    pub fn push(&mut self, value: T) {
        if self.len == self.cap {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }

        self.len += 1;
    }

    /// Removes and returns the last element, or None if empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.add(self.len)))
            }
        }
    }

    /// Returns a reference to the element at the given index.
    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            unsafe {
                Some(&*self.ptr.add(index))
            }
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Grows the capacity, doubling it or setting to 1 if currently 0.
    fn grow(&mut self) {
        let new_cap = if self.cap == 0 { 1 } else { self.cap * 2 };
        let new_layout = Layout::array::<T>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                realloc(
                    self.ptr as *mut u8,
                    old_layout,
                    new_layout.size(),
                ) as *mut T
            }
        };

        if new_ptr.is_null() {
            panic!("Allocation failed");
        }

        self.ptr = new_ptr;
        self.cap = new_cap;
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        // Drop all elements
        while self.pop().is_some() {}

        // Deallocate memory
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

// Safety: MyVec<T> can be sent to another thread if T can
unsafe impl<T: Send> Send for MyVec<T> {}
unsafe impl<T: Sync> Sync for MyVec<T> {}

fn main() {
    // Usage: Safe API, unsafe internals hidden
    let mut v = MyVec::new();
    v.push(1);
    v.push(2);
    v.push(3);

    println!("Length: {}", v.len());
    println!("Element at 0: {:?}", v.get(0));
    println!("Element at 1: {:?}", v.get(1));
    println!("Element at 2: {:?}", v.get(2));

    assert_eq!(v.pop(), Some(3));
    println!("After pop, length: {}", v.len());

    println!("MyVec example completed");
}
