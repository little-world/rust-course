// Pattern 7: Smart Pointers - Box, Rc, Arc, RefCell
// Demonstrates heap allocation, shared ownership, and interior mutability.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// ============================================================================
// Example: Box<T> - Heap Allocation
// ============================================================================

fn box_basics() {
    // Creating boxed values
    let boxed_int = Box::new(42);
    let boxed_string = Box::new(String::from("hello"));

    println!("Boxed int: {}", boxed_int);
    println!("Boxed string: {}", boxed_string);

    // Dereferencing
    let value = *boxed_int;
    println!("Dereferenced: {}", value);
}

// Recursive data structure requires Box
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

fn recursive_list() {
    use List::{Cons, Nil};

    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    println!("Recursive list: {:?}", list);
}

// Large struct that should be heap-allocated
struct LargeStruct {
    data: [u8; 1000],
}

fn large_data_on_heap() {
    let large = Box::new(LargeStruct { data: [0; 1000] });
    println!("Large struct on heap, first byte: {}", large.data[0]);
}

// Trait objects with Box
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) {
        println!("Woof!");
    }
}

struct Cat;
impl Animal for Cat {
    fn speak(&self) {
        println!("Meow!");
    }
}

fn trait_objects() {
    let animals: Vec<Box<dyn Animal>> = vec![Box::new(Dog), Box::new(Cat)];

    for animal in &animals {
        animal.speak();
    }
}

fn box_ownership() {
    let boxed = Box::new(42);
    let moved = boxed;
    // boxed is now invalid
    println!("Moved box: {}", moved);
}

// ============================================================================
// Example: Rc<T> - Reference Counted Shared Ownership
// ============================================================================

fn rc_basics() {
    let rc1 = Rc::new(42);
    let rc2 = Rc::clone(&rc1);
    let rc3 = rc1.clone();

    println!("rc1: {}, rc2: {}, rc3: {}", rc1, rc2, rc3);
    println!("Strong count: {}", Rc::strong_count(&rc1));

    drop(rc2);
    println!("After drop rc2, count: {}", Rc::strong_count(&rc1));
}

// Shared graph nodes with Rc
struct Node {
    value: i32,
    children: Vec<Rc<Node>>,
}

fn shared_graph() {
    let leaf = Rc::new(Node {
        value: 3,
        children: vec![],
    });

    let node = Rc::new(Node {
        value: 5,
        children: vec![Rc::clone(&leaf)],
    });

    println!("Leaf strong count: {}", Rc::strong_count(&leaf));
    println!("Node value: {}", node.value);
}

// Weak references to break cycles
struct Parent {
    #[allow(dead_code)]
    children: RefCell<Vec<Rc<Child>>>,
}

struct Child {
    #[allow(dead_code)]
    parent: Weak<Parent>,
}

fn weak_references() {
    let parent = Rc::new(Parent {
        children: RefCell::new(vec![]),
    });

    let child = Rc::new(Child {
        parent: Rc::downgrade(&parent),
    });

    parent.children.borrow_mut().push(Rc::clone(&child));

    println!("Parent strong count: {}", Rc::strong_count(&parent));
    println!("Child strong count: {}", Rc::strong_count(&child));

    // Access weak reference
    if let Some(_parent_rc) = child.parent.upgrade() {
        println!("Child can access parent");
    }
}

// ============================================================================
// Example: Arc<T> - Atomic Reference Counted (Thread-Safe)
// ============================================================================

fn arc_basics() {
    let arc1 = Arc::new(42);
    let arc2 = Arc::clone(&arc1);

    println!("arc1: {}, arc2: {}", arc1, arc2);
    println!("Strong count: {}", Arc::strong_count(&arc1));
}

fn arc_across_threads() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let data_clone = Arc::clone(&data);
            thread::spawn(move || {
                println!("Thread {}: {:?}", i, data_clone);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Main thread: {:?}", data);
}

// Shared immutable config
struct Config {
    max_connections: usize,
    timeout_ms: u64,
}

fn shared_config() {
    let config = Arc::new(Config {
        max_connections: 100,
        timeout_ms: 5000,
    });

    let config_clone = Arc::clone(&config);
    let handle = thread::spawn(move || {
        println!("Thread: max_connections = {}", config_clone.max_connections);
    });

    println!("Main: timeout_ms = {}", config.timeout_ms);
    handle.join().unwrap();
}

// ============================================================================
// Example: RefCell<T> - Interior Mutability
// ============================================================================

fn refcell_basics() {
    let cell = RefCell::new(42);

    // Borrow mutably through shared ref
    {
        let mut borrow = cell.borrow_mut();
        *borrow += 1;
    }

    let value = cell.borrow();
    println!("RefCell value: {}", *value);
}

fn refcell_runtime_checks() {
    let cell = RefCell::new(42);

    // try_borrow to avoid panics
    if let Ok(value) = cell.try_borrow() {
        println!("Borrowed: {}", *value);
    }

    // Multiple borrows
    let borrow1 = cell.borrow();
    let borrow2 = cell.borrow();
    println!("Two borrows: {}, {}", *borrow1, *borrow2);
    drop(borrow1);
    drop(borrow2);

    // try_borrow_mut
    let mut_borrow = cell.borrow_mut();
    if cell.try_borrow().is_err() {
        println!("Cannot borrow immutably while mutably borrowed");
    }
    drop(mut_borrow);
}

// Rc<RefCell<T>> pattern - multiple owners with mutation
fn rc_refcell_pattern() {
    struct SharedData {
        value: RefCell<i32>,
    }

    let data = Rc::new(SharedData {
        value: RefCell::new(0),
    });

    let data_clone = Rc::clone(&data);

    *data.value.borrow_mut() += 1;
    *data_clone.value.borrow_mut() += 1;

    println!("Shared value after modifications: {}", data.value.borrow());
}

// ============================================================================
// Example: Cell<T> - Simple Interior Mutability
// ============================================================================

fn cell_basics() {
    let cell = Cell::new(42);

    // Getting and setting
    let value = cell.get();
    cell.set(100);
    let new_value = cell.get();

    println!("Cell: {} -> {}", value, new_value);

    // Swapping and updating
    let old = cell.replace(200);
    println!("Replaced: {} with 200", old);

    // Note: Cell only works with Copy types
}

// Counter using Cell
struct Counter {
    count: Cell<u32>,
}

impl Counter {
    fn new() -> Self {
        Counter {
            count: Cell::new(0),
        }
    }

    fn increment(&self) {
        // Takes &self, not &mut self!
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> u32 {
        self.count.get()
    }
}

fn cell_counter() {
    let counter = Counter::new();
    counter.increment();
    counter.increment();
    counter.increment();
    println!("Counter: {}", counter.get());
}

// ============================================================================
// Example: Mutex<T> and RwLock<T> - Thread-Safe Interior Mutability
// ============================================================================

fn mutex_basics() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter_clone.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Mutex counter: {}", *counter.lock().unwrap());
}

fn rwlock_basics() {
    let data = Arc::new(RwLock::new(vec![1, 2, 3]));

    // Multiple readers
    let data_clone1 = Arc::clone(&data);
    let reader1 = thread::spawn(move || {
        let vec = data_clone1.read().unwrap();
        println!("Reader 1: {:?}", *vec);
    });

    let data_clone2 = Arc::clone(&data);
    let reader2 = thread::spawn(move || {
        let vec = data_clone2.read().unwrap();
        println!("Reader 2: {:?}", *vec);
    });

    // One writer
    let data_clone3 = Arc::clone(&data);
    let writer = thread::spawn(move || {
        let mut vec = data_clone3.write().unwrap();
        vec.push(4);
        println!("Writer added 4");
    });

    reader1.join().unwrap();
    reader2.join().unwrap();
    writer.join().unwrap();

    println!("Final data: {:?}", *data.read().unwrap());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_basics() {
        let boxed = Box::new(42);
        assert_eq!(*boxed, 42);
    }

    #[test]
    fn test_box_move() {
        let boxed = Box::new(String::from("hello"));
        let moved = boxed;
        assert_eq!(*moved, "hello");
    }

    #[test]
    fn test_rc_basics() {
        let rc1 = Rc::new(42);
        let rc2 = Rc::clone(&rc1);

        assert_eq!(*rc1, 42);
        assert_eq!(*rc2, 42);
        assert_eq!(Rc::strong_count(&rc1), 2);

        drop(rc2);
        assert_eq!(Rc::strong_count(&rc1), 1);
    }

    #[test]
    fn test_weak_upgrade() {
        let strong = Rc::new(42);
        let weak = Rc::downgrade(&strong);

        assert!(weak.upgrade().is_some());
        drop(strong);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_arc_basics() {
        let arc1 = Arc::new(42);
        let arc2 = Arc::clone(&arc1);

        assert_eq!(*arc1, 42);
        assert_eq!(*arc2, 42);
        assert_eq!(Arc::strong_count(&arc1), 2);
    }

    #[test]
    fn test_refcell_basics() {
        let cell = RefCell::new(42);

        *cell.borrow_mut() += 1;
        assert_eq!(*cell.borrow(), 43);
    }

    #[test]
    fn test_refcell_multiple_immutable_borrows() {
        let cell = RefCell::new(42);

        let borrow1 = cell.borrow();
        let borrow2 = cell.borrow();
        assert_eq!(*borrow1, *borrow2);
    }

    #[test]
    fn test_refcell_try_borrow() {
        let cell = RefCell::new(42);

        let _borrow = cell.borrow_mut();
        assert!(cell.try_borrow().is_err());
        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    fn test_cell_basics() {
        let cell = Cell::new(42);

        assert_eq!(cell.get(), 42);
        cell.set(100);
        assert_eq!(cell.get(), 100);
    }

    #[test]
    fn test_cell_replace() {
        let cell = Cell::new(42);
        let old = cell.replace(100);

        assert_eq!(old, 42);
        assert_eq!(cell.get(), 100);
    }

    #[test]
    fn test_counter() {
        let counter = Counter::new();

        counter.increment();
        counter.increment();
        assert_eq!(counter.get(), 2);
    }

    #[test]
    fn test_rc_refcell() {
        let data = Rc::new(RefCell::new(0));
        let data_clone = Rc::clone(&data);

        *data.borrow_mut() += 1;
        *data_clone.borrow_mut() += 1;

        assert_eq!(*data.borrow(), 2);
    }

    #[test]
    fn test_mutex() {
        let counter = Arc::new(Mutex::new(0));

        {
            let mut num = counter.lock().unwrap();
            *num += 1;
        }

        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn test_rwlock() {
        let data = RwLock::new(vec![1, 2, 3]);

        {
            let read = data.read().unwrap();
            assert_eq!(*read, vec![1, 2, 3]);
        }

        {
            let mut write = data.write().unwrap();
            write.push(4);
        }

        assert_eq!(*data.read().unwrap(), vec![1, 2, 3, 4]);
    }
}

fn main() {
    println!("Pattern 7: Smart Pointers");
    println!("=========================\n");

    println!("=== Box<T>: Heap Allocation ===");
    box_basics();
    println!();

    println!("=== Recursive Data with Box ===");
    recursive_list();
    println!();

    println!("=== Large Data on Heap ===");
    large_data_on_heap();
    println!();

    println!("=== Trait Objects with Box ===");
    trait_objects();
    println!();

    println!("=== Box Ownership ===");
    box_ownership();
    println!();

    println!("=== Rc<T>: Reference Counting ===");
    rc_basics();
    println!();

    println!("=== Shared Graph with Rc ===");
    shared_graph();
    println!();

    println!("=== Weak References ===");
    weak_references();
    println!();

    println!("=== Arc<T>: Thread-Safe Reference Counting ===");
    arc_basics();
    println!();

    println!("=== Arc Across Threads ===");
    arc_across_threads();
    println!();

    println!("=== Shared Config with Arc ===");
    shared_config();
    println!();

    println!("=== RefCell<T>: Interior Mutability ===");
    refcell_basics();
    println!();

    println!("=== RefCell Runtime Checks ===");
    refcell_runtime_checks();
    println!();

    println!("=== Rc<RefCell<T>> Pattern ===");
    rc_refcell_pattern();
    println!();

    println!("=== Cell<T>: Simple Interior Mutability ===");
    cell_basics();
    println!();

    println!("=== Cell Counter ===");
    cell_counter();
    println!();

    println!("=== Mutex<T>: Thread-Safe Mutation ===");
    mutex_basics();
    println!();

    println!("=== RwLock<T>: Multiple Readers, One Writer ===");
    rwlock_basics();
}
