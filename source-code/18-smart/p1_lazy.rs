// Pattern 1: Custom Smart Pointers - Lazy Initialization
use std::cell::UnsafeCell;

struct Lazy<T, F: FnOnce() -> T> {
    value: UnsafeCell<Option<T>>,
    init: UnsafeCell<Option<F>>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    fn new(init: F) -> Self {
        Self {
            value: UnsafeCell::new(None),
            init: UnsafeCell::new(Some(init)),
        }
    }

    fn get(&self) -> &T {
        unsafe {
            if (*self.value.get()).is_none() {
                let init = (*self.init.get()).take().unwrap();
                *self.value.get() = Some(init());
            }
            (*self.value.get()).as_ref().unwrap()
        }
    }
}

fn main() {
    // Usage: Expensive init runs only on first access
    fn expensive_init() -> Vec<i32> {
        println!("Computing...");
        (0..1000).collect()
    }

    let lazy = Lazy::new(expensive_init);
    // Nothing computed yet
    let data = lazy.get(); // "Computing..." printed here
    println!("Length: {}", data.len());
    let _data2 = lazy.get(); // No print, returns cached value

    println!("Lazy initialization example completed");
}
