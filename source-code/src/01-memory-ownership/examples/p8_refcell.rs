// Pattern 8: RefCell for Complex Types
use std::cell::RefCell;
use std::collections::HashMap;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Self {
        Cache { data: RefCell::new(HashMap::new()) }
    }

    fn get_or_compute(&self, key: &str, compute: impl FnOnce() -> String) -> String {
        if let Some(value) = self.data.borrow().get(key) {
            return value.clone();
        }
        let value = compute();
        self.data.borrow_mut().insert(key.to_string(), value.clone());
        value
    }
}

fn main() {
    let cache = Cache::new();

    let result1 = cache.get_or_compute("key1", || {
        println!("Computing value for key1...");
        "value1".to_string()
    });
    println!("Result: {}", result1);

    let result2 = cache.get_or_compute("key1", || {
        println!("This won't print - using cached value");
        "value2".to_string()
    });
    println!("Result (cached): {}", result2);

    println!("RefCell example completed");
}
