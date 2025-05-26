Here’s a **cookbook-style tutorial for `std::boxed`**, which defines [`Box<T>`](https://doc.rust-lang.org/std/boxed/struct.Box.html) — Rust’s standard **heap allocation smart pointer**.

---

## Rust std::boxed Cookbook

> 📦 Module: [`std::boxed`](https://doc.rust-lang.org/std/boxed/)

`Box<T>` is:

* **Heap-allocated**
* **Single-owner** (not shared)
* **Zero-cost abstraction** (pointer overhead only)
* Used for **recursive types**, **large data**, **trait objects**

---

## Basics

---

### Box a Value

```rust
fn main() {
    let b = Box::new(42);
    println!("Boxed value: {}", b);
}
```

📘 `Box<T>` allocates `T` on the heap and returns a smart pointer.

---

### Dereference a Box

```rust
fn main() {
    let b = Box::new("hello");
    println!("Deref: {}", *b); // works like a reference
}
```

📘 `Box<T>` implements `Deref`, so you can use `*b` or `b.len()` etc.

---

### Move Ownership Out of a Box

```rust
fn main() {
    let b = Box::new(String::from("Rust"));
    let s: String = *b; // moves the String out
    println!("{}", s);
}
```

---

## Recursive and Trait Types

---

### Box a Recursive Type

```rust
enum List {
    Node(i32, Box<List>),
    Nil,
}

fn main() {
    let list = List::Node(1, Box::new(List::Node(2, Box::new(List::Nil))));
}
```

📘 Without `Box`, this recursive enum wouldn't compile due to unknown size.

---

### Box Trait Objects (dyn Trait)

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) {
        println!("Woof!");
    }
}

fn main() {
    let a: Box<dyn Animal> = Box::new(Dog);
    a.speak();
}
```

📘 Trait objects require indirection — `Box<dyn Trait>` is the idiom.

---

## Advanced Use Cases

---

### Pass Boxed Value to a Function

```rust
fn consume(b: Box<i32>) {
    println!("Got {}", b);
}

fn main() {
    let x = Box::new(10);
    consume(x); // moves ownership
}
```

---

### Return a Box from a Function

```rust
fn make_box() -> Box<String> {
    Box::new(String::from("boxed!"))
}

fn main() {
    let s = make_box();
    println!("{}", s);
}
```

---

### Box with Custom Drop Order

```rust
struct Resource;

impl Drop for Resource {
    fn drop(&mut self) {
        println!("Resource dropped!");
    }
}

fn main() {
    let _res = Box::new(Resource);
    println!("Before drop");
}
```

📘 Box deallocates on drop — useful for managing resources.

---

### Use Box to Avoid Stack Overflow

```rust
fn recursive(n: u32) -> Box<u32> {
    if n == 0 {
        Box::new(0)
    } else {
        Box::new(n + *recursive(n - 1))
    }
}

fn main() {
    println!("{}", recursive(10));
}
```

📘 Boxing intermediate values can avoid stack overflow in deep recursions.

---

## Summary Table

| Feature         | `Box<T>`                       |
| --------------- | ------------------------------ |
| Memory location | Heap                           |
| Ownership       | Exclusive                      |
| Deref coercion  | Yes                            |
| Used for        | Recursive types, trait objects |
| Drop behavior   | Automatic                      |
| Smart pointer?  | ✅ Yes                          |

---

## When to Use Box<T>

* You need **heap allocation** for large or recursive types
* You're working with **trait objects**
* You want to **avoid stack overflows** in deep recursion
* You need a **single-owner smart pointer**

