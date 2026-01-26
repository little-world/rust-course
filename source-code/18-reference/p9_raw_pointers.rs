// Pattern 9: Raw Pointer Conversions
use std::alloc::{alloc, dealloc, Layout};

struct Handle {
    ptr: *mut u8,
    len: usize,
}

impl Handle {
    fn new(len: usize) -> Self {
        let layout = Layout::from_size_align(len, 1).unwrap();
        let ptr = unsafe { alloc(layout) };
        Handle { ptr, len }
    }

    // into_raw: leak memory, caller takes ownership
    fn into_raw(self) -> *mut u8 {
        let ptr = self.ptr;
        std::mem::forget(self);  // Prevent destructor
        ptr
    }

    // from_raw: reclaim ownership from raw pointer
    unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        Handle { ptr, len }
    }

    // as_ptr: borrow as raw pointer (no ownership transfer)
    fn as_ptr(&self) -> *const u8 { self.ptr }
    fn as_mut_ptr(&mut self) -> *mut u8 { self.ptr }
}

impl Drop for Handle {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                dealloc(
                    self.ptr,
                    Layout::from_size_align_unchecked(self.len, 1)
                );
            }
        }
    }
}

fn main() {
    // Create a handle
    let mut handle = Handle::new(16);

    // Write some data using as_mut_ptr
    unsafe {
        let ptr = handle.as_mut_ptr();
        for i in 0..16 {
            *ptr.add(i) = i as u8;
        }
    }

    // Read using as_ptr
    unsafe {
        let ptr = handle.as_ptr();
        print!("Data: ");
        for i in 0..16 {
            print!("{:02x} ", *ptr.add(i));
        }
        println!();
    }

    // Convert to raw pointer (caller takes ownership)
    let len = handle.len;
    let raw = handle.into_raw();

    // Reclaim from raw pointer
    let handle2 = unsafe { Handle::from_raw(raw, len) };
    println!("Reclaimed handle with len: {}", handle2.len);

    // handle2 will be dropped here, freeing memory

    println!("Raw pointers example completed");
}
