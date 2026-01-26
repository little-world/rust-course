// Pattern 8: Cell for Copy Types
use std::cell::Cell;

struct Counter {
    count: Cell<usize>,
}

impl Counter {
    fn new() -> Self {
        Counter { count: Cell::new(0) }
    }

    fn increment(&self) {  // Takes &self, not &mut self!
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> usize {
        self.count.get()
    }
}

fn main() {
    let counter = Counter::new();
    counter.increment();
    counter.increment();
    println!("Count: {}", counter.get()); // 2
    println!("Cell example completed");
}
