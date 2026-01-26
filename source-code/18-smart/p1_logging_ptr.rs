// Pattern 1: Custom Smart Pointers - Logging Pointer (Access Tracking)
use std::ops::{Deref, DerefMut};
use std::cell::Cell;

struct LoggingPtr<T> {
    data: Box<T>,
    reads: Cell<usize>,
    writes: Cell<usize>,
}

impl<T> LoggingPtr<T> {
    fn new(value: T) -> Self {
        Self {
            data: Box::new(value),
            reads: Cell::new(0),
            writes: Cell::new(0),
        }
    }

    fn stats(&self) -> (usize, usize) {
        (self.reads.get(), self.writes.get())
    }
}

impl<T> Deref for LoggingPtr<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.reads.set(self.reads.get() + 1);
        &self.data
    }
}

impl<T> DerefMut for LoggingPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writes.set(self.writes.get() + 1);
        &mut self.data
    }
}

fn main() {
    // Usage: Profile hot paths
    let mut p = LoggingPtr::new(vec![1, 2, 3]);
    let _ = p.len();      // read
    let _ = p.len();      // read
    p.push(4);            // write
    println!("{:?}", p.stats()); // (2, 1)

    println!("Logging pointer example completed");
}
