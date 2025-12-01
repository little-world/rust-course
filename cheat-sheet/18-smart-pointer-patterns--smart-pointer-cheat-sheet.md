### Smart Pointer Cheat Sheet

```rust
// Box<T> - Heap allocation
Box::new(value)                                      // Allocate value on heap
Box::new(5)                                          // Box a primitive
Box::new(MyStruct { })                               // Box a struct

// Rc<T> - Reference counted (single-threaded)
Rc::new(value)                                       // Create new Rc
Rc::clone(&rc)                                       // Clone reference (cheap)
rc.clone()                                           // Alternative syntax

// Arc<T> - Atomic reference counted (thread-safe)
Arc::new(value)                                      // Create new Arc
Arc::clone(&arc)                                     // Clone reference (cheap, atomic)
arc.clone()                                          // Alternative syntax

// Weak<T> - Weak reference (doesn't prevent deallocation)
Rc::downgrade(&rc)                                   // Create weak from Rc
Arc::downgrade(&arc)                                 // Create weak from Arc
weak.upgrade()                                       // Try to upgrade to Rc/Arc, returns Option

// Cell<T> - Interior mutability (single-threaded, Copy types)
Cell::new(value)                                     // Create new Cell
cell.get()                                           // Get copy of value (T must be Copy)
cell.set(value)                                      // Set value

// RefCell<T> - Interior mutability with runtime borrow checking
RefCell::new(value)                                  // Create new RefCell
cell.borrow()                                        // Immutable borrow, returns Ref<T>
cell.borrow_mut()                                    // Mutable borrow, returns RefMut<T>

// Cow<T> - Clone-on-write
Cow::Borrowed(&value)                                // Borrow without ownership
Cow::Owned(value)                                    // Take ownership
cow.to_mut()                                         // Get mutable ref, clone if needed
cow.into_owned()                                     // Convert to owned value
Cow::from("string")                                  // From &str or String

// Pin<P> - Prevent moving (for self-referential types)
Pin::new(&value)                                     // Pin reference
Pin::new(&mut value)                                 // Pin mutable reference
Box::pin(value)                                      // Create pinned Box

// Mutex/RwLock smart pointer methods (lock guards)
mutex.lock().unwrap()                                // Returns MutexGuard<T>
rwlock.read().unwrap()                               // Returns RwLockReadGuard<T>
rwlock.write().unwrap()                              // Returns RwLockWriteGuard<T>
*guard = value                                       // Modify through guard
drop(guard)                                          // Explicit unlock

// Common patterns
// Shared ownership (single-threaded)
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let clone = Rc::clone(&data);
data.borrow_mut().push(4);

// Shared ownership (thread-safe)
let data = Arc::new(Mutex::new(vec![1, 2, 3]));
let clone = Arc::clone(&data);
thread::spawn(move || {
    clone.lock().unwrap().push(4);
});

// Weak references to break cycles
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

// Clone-on-write pattern
fn process(input: Cow<str>) -> Cow<str> {
    if needs_modification(&input) {
        Cow::Owned(input.to_uppercase())
    } else {
        input
    }
}

// Interior mutability with Rc
let shared = Rc::new(RefCell::new(5));
*shared.borrow_mut() += 1;

// Thread-safe shared state
let counter = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..10).map(|_| {
    let counter = Arc::clone(&counter);
    thread::spawn(move || {
        *counter.lock().unwrap() += 1;
    })
}).collect();

// Box for trait objects
let shape: Box<dyn Shape> = Box::new(Circle { radius: 5.0 });

// Box for recursive types
enum List {
    Cons(i32, Box<List>),
    Nil,
}

// Lazy initialization
let cell: OnceCell<ExpensiveData> = OnceCell::new();
let data = cell.get_or_init(|| compute_expensive_data());

// Type conversions
Box::from(rc)                                        // Convert Rc to Box
Rc::from(box_value)                                  // Convert Box to Rc
Arc::from(box_value)                                 // Convert Box to Arc
```
