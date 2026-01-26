// Pattern 1: Custom Smart Pointers - Simple Custom Box
use std::ops::{Deref, DerefMut};

struct MyBox<T> {
    data: *mut T,
}

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        let data = Box::into_raw(Box::new(value));
        MyBox { data }
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.data)); }
    }
}

fn main() {
    // Usage
    let mut b = MyBox::new(42);
    *b = 100;
    println!("{}", *b); // 100

    println!("Custom box example completed");
}
