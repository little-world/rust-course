
##  Smart Pointers

Smart pointers in Rust are data structures that **own memory** and offer **extra capabilities** like:

* Heap allocation (`Box`)
* Shared ownership (`Rc`)
* Runtime-checked borrowing (`RefCell`)
* Thread-safe shared ownership (`Arc`)



## Box<T> 
### Heap Allocation (Single Ownership)

Stores data on the heap with known size at compile time.

### Example:

```rust
fn main() {
    let b = Box::new(5);
    println!("Boxed value: {}", b);
}
```

### When to use:

* Recursive types
* Large data
* Trait objects

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```


## Rc<T> 
### Reference Counted (Shared Ownership, Single Thread)

Multiple owners of the same value. Uses **reference counting**.

### Example:

```rust
use std::rc::Rc;

fn main() {
    let a = Rc::new(String::from("hello"));
    let b = Rc::clone(&a); // increments reference count

    println!("a: {}, b: {}", a, b);
    println!("Reference count: {}", Rc::strong_count(&a));
}
```

> `Rc` is **not thread-safe**.



## RefCell<T> 
### Mutable Borrowing at Runtime

Allows **interior mutability**: you can mutate data even when it's not declared `mut`, but borrow rules are checked at **runtime**, not compile time.

### Example:

```rust
use std::cell::RefCell;

fn main() {
    let x = RefCell::new(5);
    *x.borrow_mut() += 1;
    println!("x = {}", x.borrow());
}
```

> Use only in **single-threaded** scenarios where you need mutable access in shared data.



## Rc<RefCell<T>> 
### Shared Ownership + Interior Mutability

Common pattern in tree/graph structures.

```rust
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let shared = Rc::new(RefCell::new(10));
    let a = Rc::clone(&shared);
    *a.borrow_mut() += 5;
    println!("shared = {}", shared.borrow()); // 15
}
```



## Arc<T> 
### Atomically Reference Counted (Thread-Safe Rc)

Like `Rc`, but **safe for multiple threads**.

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3]);

    for _ in 0..3 {
        let d = Arc::clone(&data);
        thread::spawn(move || {
            println!("{:?}", d);
        });
    }
}
```

> Use when you need to **share read-only data across threads**.


## Summary 

| Smart Pointer | Use Case                              | Thread Safe | Mutable?              |
| ------------- | ------------------------------------- | ----------- | --------------------- |
| `Box<T>`      | Heap allocation, recursive types      | ✅ Yes       | Only by owner         |
| `Rc<T>`       | Shared ownership (read-only)          | ❌ No        | ❌ Immutable           |
| `RefCell<T>`  | Interior mutability (runtime checked) | ❌ No        | ✅ Yes (runtime check) |
| `Rc<RefCell>` | Shared + mutable in single thread     | ❌ No        | ✅ Yes                 |
| `Arc<T>`      | Shared ownership across threads       | ✅ Yes       | ❌ Immutable           |



| Scenario                           | Use              |
| ---------------------------------- | ---------------- |
| Own heap data                      | `Box<T>`         |
| Share data within one thread       | `Rc<T>`          |
| Mutate data with single owner      | `RefCell<T>`     |
| Share **and** mutate in one thread | `Rc<RefCell<T>>` |
| Share data across threads          | `Arc<T>`         |