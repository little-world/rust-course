// Pattern 10: Pin and Self-Referential Types
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    // This would point into data - requires Pin
    ptr: *const String,
    _pin: PhantomPinned,  // Opts out of Unpin
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::new(SelfReferential {
            data,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // Safe because we're setting up the self-reference
        // before returning the Pin
        let ptr: *const String = &boxed.data;
        boxed.ptr = ptr;

        // SAFETY: we never move the data after this
        unsafe { Pin::new_unchecked(boxed) }
    }

    fn data(self: Pin<&Self>) -> &str {
        &self.get_ref().data
    }

    // Pin projection: Pin<&Self> -> Pin<&Field>
    // Only valid if Field: Unpin, otherwise must be unsafe
    fn project_data(self: Pin<&Self>) -> &String {
        // String: Unpin, so this is safe
        &self.get_ref().data
    }

    // Demonstrate the self-reference is valid
    fn check_self_ref(self: Pin<&Self>) -> bool {
        let inner = self.get_ref();
        let data_addr = &inner.data as *const String;
        data_addr == inner.ptr
    }
}

fn main() {
    let pinned = SelfReferential::new(String::from("Hello, Pin!"));

    // Access data through Pin
    println!("Data: {}", pinned.as_ref().data());
    println!("Projected data: {}", pinned.as_ref().project_data());
    println!("Self-reference valid: {}", pinned.as_ref().check_self_ref());

    // The following would not compile because SelfReferential is !Unpin:
    // let moved = *pinned; // Error: cannot move out of Pin

    println!("Pin example completed");
}
