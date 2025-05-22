Here’s a **cookbook-style tutorial for `std::option`**, one of Rust's most fundamental types for **handling optional values safely** — without `null`. It’s widely used for error handling, configuration defaults, and fallible operations.

---

## Rust std::option Cookbook

> 📦 Module: [`std::option`](https://doc.rust-lang.org/std/option/)

The enum:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

---

## Essentials

---

### Create and Match an Option

```rust
fn main() {
    let name: Option<String> = Some("Rust".into());

    match name {
        Some(n) => println!("Name: {}", n),
        None => println!("No name"),
    }
}
```

📘 Pattern matching is the most flexible way to handle `Option`.

---

### Use .unwrap() ( Only if sure)

```rust
fn main() {
    let x = Some(42);
    println!("{}", x.unwrap()); // OK

    let y: Option<i32> = None;
    // println!("{}", y.unwrap()); // panics!
}
```

📘 Only use `unwrap()` if you're absolutely sure it's `Some`.

---

### Use .unwrapor() and .unwraporelse()

```rust
fn main() {
    let x = None;
    println!("{}", x.unwrap_or(100)); // fallback value

    let computed = x.unwrap_or_else(|| expensive_default());
}

fn expensive_default() -> i32 {
    println!("Computing default...");
    99
}
```

📘 Prefer `.unwrap_or_else()` if the default is costly to compute.

---

### Use .map() to Transform Contents

```rust
fn main() {
    let x = Some("rust");
    let len = x.map(|s| s.len());
    println!("{:?}", len); // Some(4)
}
```

📘 Keeps `Option<T>` wrapped while transforming the inner value.

---

### Use .andthen() for Chained Option Logic

```rust
fn square_root(x: f64) -> Option<f64> {
    if x >= 0.0 { Some(x.sqrt()) } else { None }
}

fn main() {
    let result = Some(16.0)
        .and_then(square_root)
        .and_then(square_root);
    
    println!("{:?}", result); // Some(2.0)
}
```

📘 Similar to `flatMap` in functional languages.

---

### Use .filter() to Conditionally Keep a Value

```rust
fn main() {
    let x = Some(42);
    let even = x.filter(|&n| n % 2 == 0);
    let odd = x.filter(|&n| n % 2 != 0);
    println!("{:?} {:?}", even, odd); // Some(42) None
}
```

📘 Keeps `Some(value)` only if the condition is true.

---

### Convert from Option to Result

```rust
fn main() {
    let maybe = Some("data");
    let result: Result<_, &str> = maybe.ok_or("No value");
    println!("{:?}", result); // Ok("data")
}
```

📘 Use `.ok_or()` or `.ok_or_else()` to provide an error.

---

### Pattern Match with if let

```rust
fn main() {
    let value = Some("hello");

    if let Some(s) = value {
        println!("Got: {}", s);
    }
}
```

📘 Cleaner than `match` when you only care about `Some`.

---

### Combine Options with .or() and .and()

```rust
fn main() {
    let a = Some(1);
    let b = Some(2);
    let none: Option<i32> = None;

    println!("{:?}", a.or(b));    // Some(1)
    println!("{:?}", none.or(b)); // Some(2)
    println!("{:?}", a.and(b));   // Some(2)
}
```

📘 Think of `.or()` as fallback, `.and()` as chaining presence.

---

### Use .issome() / .isnone() for Checks

```rust
fn main() {
    let x: Option<i32> = Some(10);
    println!("Is some? {}", x.is_some()); // true
    println!("Is none? {}", x.is_none()); // false
}
```

---

## Summary Table of Common Methods

| Method          | Purpose                             |
| --------------- | ----------------------------------- |
| `.unwrap()`     | Panic if `None`                     |
| `.unwrap_or(x)` | Default value if `None`             |
| `.map(f)`       | Transform inner value               |
| `.and_then(f)`  | Chain operations returning `Option` |
| `.filter(f)`    | Keep only if predicate passes       |
| `.ok_or(e)`     | Convert to `Result<T, E>`           |
| `.is_some()`    | Check if value is present           |
| `.take()`       | Take the value, leaving `None`      |

---

### Use Cases

* **Optional config values**: `Option<String>`
* **Parsing/validation**: `Option<T>` as intermediate fallible result
* **APIs that might return something**: `get()`, `find()`, etc.

