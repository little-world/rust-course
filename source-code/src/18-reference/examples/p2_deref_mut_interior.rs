// Pattern 2: DerefMut and Interior Mutability Interaction
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

struct TrackedMut<T> {
    value: T,
    write_count: RefCell<usize>,
}

impl<T> Deref for TrackedMut<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.value }
}

impl<T> DerefMut for TrackedMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        *self.write_count.borrow_mut() += 1;
        &mut self.value
    }
}

// Key insight: DerefMut requires &mut self, but the RefCell
// allows mutation through &self. This is a valid pattern because
// we're not mutating through DerefMut, we're using interior mutability
// for metadata while DerefMut gives mutable access to the wrapped value.

fn main() {
    let mut tracked = TrackedMut {
        value: String::from("hello"),
        write_count: RefCell::new(0),
    };

    // Read through Deref
    println!("Value: {}", &*tracked);

    // Write through DerefMut
    tracked.push_str(" world");
    tracked.push_str("!");

    println!("Value after writes: {}", &*tracked);
    println!("Write count: {}", tracked.write_count.borrow());
}
