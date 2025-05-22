Here’s a **cookbook-style tutorial for `std::marker`**, a module in Rust’s standard library that provides **zero-sized marker traits and types**. These are used to convey important type-level information at compile time, especially around ownership, concurrency, and lifetimes.

---

## Rust std::marker Cookbook

> 📦 Module: [`std::marker`](https://doc.rust-lang.org/std/marker/)

Contains:

* Marker **traits**: `Copy`, `Send`, `Sync`, `Sized`, `Unpin`
* Marker **types**: `PhantomData<T>`

---

## Marker Traits

---

### Copy Implicit Bitwise Copying

```rust
#[derive(Copy, Clone)]
struct Point(i32, i32);

fn main() {
    let a = Point(1, 2);
    let b = a; // No move, just copy
    println!("{:?} {:?}", a.0, b.1);
}
```

📘 For cheap, simple types. Automatically implies `Clone`.

💡 Don’t use `Copy` on heap-owning types like `String` or `Vec`.

---

### Send Safe to Transfer Between Threads

```rust
fn main() {
    let handle = std::thread::spawn(|| {
        println!("Runs in another thread!");
    });

    handle.join().unwrap();
}
```

📘 Most types are `Send` unless they use non-thread-safe things like `Rc`.

---

### Sync Safe to Share Across Threads

```rust
use std::sync::Arc;

fn main() {
    let data = Arc::new(vec![1, 2, 3]);

    let d1 = Arc::clone(&data);
    std::thread::spawn(move || println!("{:?}", d1)).join().unwrap();
}
```

📘 `Arc<T>` is `Sync` if `T` is `Send`.

💡 Think: `Send = move`, `Sync = shared`.

---

### Sized Type Has Known Compile-Time Size

```rust
fn takes_sized<T: Sized>(_val: T) {}

fn takes_unsized<T: ?Sized>(_val: &T) {}
```

📘 All types are `Sized` unless explicitly handled as `?Sized` (e.g., traits, slices).

---

### Unpin Can Be Moved Safely

```rust
use std::pin::Pin;

fn use_pin(data: Pin<&mut String>) {
    println!("{}", data);
}
```

📘 Used mainly with async/futures and `Pin`.

💡 Most types are `Unpin` by default.

---

## Marker Types

---

### PhantomData<T> Type-Level Ownership Without Value

```rust
use std::marker::PhantomData;

struct MyVec<T> {
    ptr: *const T,
    _marker: PhantomData<T>,
}
```

📘 Tells the compiler “this struct logically owns a T” for drop, variance, or `Send`/`Sync`.

---

### PhantomData for Lifetimes

```rust
use std::marker::PhantomData;

struct Borrowing<'a, T> {
    _phantom: PhantomData<&'a T>,
}

fn main() {
    let _x: Borrowing<'static, i32> = Borrowing { _phantom: PhantomData };
}
```

📘 Use this to **tie a lifetime** to a struct without actually borrowing the data.

---

### Control Drop Behavior with PhantomData<T>

```rust
use std::marker::PhantomData;

struct NoDrop<T> {
    data: *const T,
    _marker: PhantomData<T>, // Not dropped
}
```

📘 Prevents unsafe memory behavior when holding raw pointers or foreign references.

---

## Summary Table

| Marker           | Type  | Purpose                                |
| ---------------- | ----- | -------------------------------------- |
| `Copy`           | Trait | Implicit bitwise copy                  |
| `Clone`          | Trait | Explicit cloning                       |
| `Send`           | Trait | Transfer across threads                |
| `Sync`           | Trait | Share between threads                  |
| `Sized`          | Trait | Compile-time known size                |
| `Unpin`          | Trait | Safe to move after pinning             |
| `PhantomData<T>` | Type  | Track type or lifetime with no storage |

---

### When to Use

* `Copy`, `Clone` — small, plain data types
* `Send`, `Sync` — thread safety
* `PhantomData<T>` — lifetimes, variance, FFI, raw pointers

---

