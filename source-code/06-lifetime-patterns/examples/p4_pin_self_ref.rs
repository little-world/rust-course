//! Pattern 4: Self-Referential Structs and Pin
//! Example: Pinned Self-Referential Struct
//!
//! Run with: cargo run --example p4_pin_self_ref

use std::marker::PhantomPinned;
use std::pin::Pin;

// A self-referential struct using Pin and raw pointers.
// This is advanced and requires unsafe code.
struct Unmovable {
    data: String,
    // Raw pointer to the data field (not a reference to avoid lifetime issues)
    slice: *const str,
    // PhantomPinned makes this type !Unpin, preventing safe moves after pinning
    _pin: PhantomPinned,
}

impl Unmovable {
    // Creates a new Unmovable, already pinned in a Box
    fn new(data: String) -> Pin<Box<Self>> {
        let res = Unmovable {
            data,
            // Can't initialize slice yet - data isn't pinned
            // Use slice_from_raw_parts to create a fat null pointer for *const str
            slice: std::ptr::slice_from_raw_parts(std::ptr::null::<u8>(), 0) as *const str,
            _pin: PhantomPinned,
        };

        // Pin the struct in a Box
        let mut boxed = Box::pin(res);

        // Now that data is pinned, create a pointer to it
        let slice = &boxed.data[..] as *const str;

        // Update the slice field with the correct pointer
        // This is safe because we're pinned and won't move
        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).slice = slice;
        }

        boxed
    }

    fn data(&self) -> &str {
        &self.data
    }

    fn slice(&self) -> &str {
        // Safe because we're pinned - the pointer is valid
        unsafe { &*self.slice }
    }
}

// Simpler example: a struct that tracks its own address
struct AddressTracker {
    data: i32,
    address: *const i32,
    _pin: PhantomPinned,
}

impl AddressTracker {
    fn new(data: i32) -> Pin<Box<Self>> {
        let tracker = AddressTracker {
            data,
            address: std::ptr::null(),
            _pin: PhantomPinned,
        };

        let mut boxed = Box::pin(tracker);

        let address = &boxed.data as *const i32;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).address = address;
        }

        boxed
    }

    fn data(&self) -> i32 {
        self.data
    }

    fn verify_address(&self) -> bool {
        // Check if the stored address still points to data
        std::ptr::eq(self.address, &self.data)
    }

    fn stored_address(&self) -> *const i32 {
        self.address
    }

    fn actual_address(&self) -> *const i32 {
        &self.data
    }
}

// Demonstrating why Pin is necessary
fn demonstrate_pin_safety() {
    println!("\n=== Pin Safety Demonstration ===");

    // Without Pin, moving would invalidate internal pointers
    // Pin prevents this by making moves a compile error

    let pinned = AddressTracker::new(42);
    println!("Initial data: {}", pinned.data());
    println!("Address valid: {}", pinned.verify_address());
    println!("Stored:  {:p}", pinned.stored_address());
    println!("Actual:  {:p}", pinned.actual_address());

    // This would NOT compile because Unmovable is !Unpin:
    // let moved = *pinned; // ERROR: cannot move out of Pin

    // We can still access the data through the Pin
    println!("Data through pin: {}", pinned.data());
}

fn main() {
    println!("=== Self-Referential Struct with Pin ===");

    // Create a pinned, self-referential struct
    let unmovable = Unmovable::new(String::from("Hello, Pin!"));

    // Access the data
    println!("Data: {}", unmovable.data());
    println!("Slice: {}", unmovable.slice());

    // The slice pointer correctly points to the data
    assert_eq!(unmovable.data(), unmovable.slice());
    println!("Data and slice match!");

    println!("\n=== Multiple Pinned Instances ===");
    let pinned1 = Unmovable::new(String::from("First"));
    let pinned2 = Unmovable::new(String::from("Second"));

    println!("Pinned 1 data: {}", pinned1.data());
    println!("Pinned 1 slice: {}", pinned1.slice());
    println!("Pinned 2 data: {}", pinned2.data());
    println!("Pinned 2 slice: {}", pinned2.slice());

    demonstrate_pin_safety();

    println!("\n=== Why Pin Exists ===");
    println!("Normal structs can be moved, invalidating internal pointers.");
    println!("Pin<T> guarantees the value won't be moved in memory.");
    println!("This makes self-referential structs safe.");

    println!("\n=== Pin in Async Rust ===");
    println!("Futures are often self-referential:");
    println!("  - They store state across await points");
    println!("  - References in that state would dangle if moved");
    println!("  - Pin ensures futures stay in place while polled");

    println!("\n=== Key Concepts ===");
    println!("- PhantomPinned: marker that makes type !Unpin");
    println!("- !Unpin types cannot be moved out of Pin");
    println!("- Pin::new_unchecked: creates Pin (requires unsafe)");
    println!("- Box::pin: creates Pin<Box<T>> safely");
    println!("- Pin::as_mut: gets Pin<&mut T> from Pin<Box<T>>");

    println!("\n=== When to Use Pin ===");
    println!("- Implementing Future trait manually");
    println!("- Self-referential data structures");
    println!("- Intrusive collections");
    println!("- FFI with callbacks that store pointers");
    println!("\nFor most code, prefer indices or restructuring!");
}
