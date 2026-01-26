//! Pattern 6: Phantom Types and Type-Level State
//! Example: FFI Ownership Marker
//!
//! Run with: cargo run --example p6_ffi_ownership

use std::marker::PhantomData;

// Trait to control whether a buffer owns its memory
trait OwnershipKind {
    const OWNS_MEMORY: bool;
}

struct Owned;
impl OwnershipKind for Owned {
    const OWNS_MEMORY: bool = true;
}

struct Borrowed;
impl OwnershipKind for Borrowed {
    const OWNS_MEMORY: bool = false;
}

// Buffer with phantom ownership marker
struct Buffer<O: OwnershipKind> {
    ptr: *mut u8,
    len: usize,
    _ownership: PhantomData<O>,
}

impl Buffer<Owned> {
    /// Create a new owned buffer from data
    fn new(data: &[u8]) -> Self {
        let boxed = data.to_vec().into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed) as *mut u8;
        println!("Allocated {} bytes at {:?}", len, ptr);
        Buffer {
            ptr,
            len,
            _ownership: PhantomData,
        }
    }

    /// Get a borrowed view of this buffer
    fn borrow(&self) -> Buffer<Borrowed> {
        Buffer {
            ptr: self.ptr,
            len: self.len,
            _ownership: PhantomData,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    fn resize(&mut self, new_data: &[u8]) {
        // Free old data
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.ptr, self.len);
            drop(Box::from_raw(slice));
        }

        // Allocate new
        let boxed = new_data.to_vec().into_boxed_slice();
        self.len = boxed.len();
        self.ptr = Box::into_raw(boxed) as *mut u8;
        println!("Resized to {} bytes at {:?}", self.len, self.ptr);
    }
}

impl Buffer<Borrowed> {
    /// Create a borrowed view from a raw pointer (unsafe)
    #[allow(dead_code)]
    unsafe fn from_ptr(ptr: *mut u8, len: usize) -> Self {
        Buffer {
            ptr,
            len,
            _ownership: PhantomData,
        }
    }
    // Note: No Drop needed - we don't own the data
}

// Methods available for both ownership types
impl<O: OwnershipKind> Buffer<O> {
    fn len(&self) -> usize {
        self.len
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

// Drop only for owned buffers - using generic impl with runtime check
impl<O: OwnershipKind> Drop for Buffer<O> {
    fn drop(&mut self) {
        if O::OWNS_MEMORY {
            println!("Freeing {} bytes at {:?}", self.len, self.ptr);
            unsafe {
                let slice = std::slice::from_raw_parts_mut(self.ptr, self.len);
                drop(Box::from_raw(slice));
            }
        } else {
            println!("Borrowed buffer dropped - no deallocation");
        }
    }
}

fn main() {
    println!("=== Owned Buffer ===");
    {
        // Usage: Owned has Drop to free memory
        let mut owned = Buffer::<Owned>::new(b"Hello, World!");
        println!("Buffer len: {}", owned.len());
        println!("Buffer content: {:?}", owned.as_slice());

        // Can mutate owned buffer
        owned.as_mut_slice()[0] = b'h';
        println!("After mutation: {:?}", owned.as_slice());

        // Resize owned buffer
        owned.resize(b"New content");
        println!("After resize: {:?}", owned.as_slice());
    } // owned dropped here, memory freed

    println!("\n=== Borrowed Buffer ===");
    {
        let owned = Buffer::<Owned>::new(b"Source data");

        // Create borrowed view
        let borrowed = owned.borrow();
        println!("Borrowed len: {}", borrowed.len());
        println!("Borrowed content: {:?}", borrowed.as_slice());

        // borrowed.as_mut_slice(); // ERROR: no as_mut_slice for Borrowed
        // borrowed.resize(...);    // ERROR: no resize for Borrowed

        // borrowed goes out of scope - no deallocation
        drop(borrowed);

        // owned still valid
        println!("Owned still valid: {:?}", owned.as_slice());
    } // owned dropped here, memory freed

    println!("\n=== Type Safety Guarantees ===");
    println!("Buffer<Owned>:");
    println!("  - Frees memory on drop");
    println!("  - Can be mutated (as_mut_slice)");
    println!("  - Can be resized");
    println!();
    println!("Buffer<Borrowed>:");
    println!("  - Does NOT free memory on drop");
    println!("  - Read-only access only");
    println!("  - Cannot resize");
    println!();
    println!("The phantom type parameter ensures correct memory management");
    println!("at compile time with zero runtime overhead!");
    println!("(The OWNS_MEMORY check is optimized away by the compiler)");
}
