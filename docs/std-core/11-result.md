Here’s a **cookbook-style tutorial for `std::result`**, Rust’s core type for **error handling** via the `Result<T, E>` enum. It enables **safe, composable, and expressive error propagation** without exceptions.

---

## Rust std::result Cookbook

> 📦 Module: [`std::result`](https://doc.rust-lang.org/std/result/)

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

---

## Basic Usage

---

### Match on a Result

```rust
fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 {
        Err("division by zero")
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10, 2) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

📘 Use `match` when you need full control.

---

### Unwrap Safely with .unwrapor()

```rust
fn main() {
    let result: Result<i32, &str> = Err("error");
    let value = result.unwrap_or(0);
    println!("Value: {}", value); // 0
}
```

📘 Also try `.unwrap_or_else(|e| { ... })` for dynamic fallback.

---

### Propagate Errors with ? Operator

```rust
fn double_even_number(n: i32) -> Result<i32, &'static str> {
    if n % 2 == 0 {
        Ok(n * 2)
    } else {
        Err("not even")
    }
}

fn wrapper() -> Result<i32, &'static str> {
    let doubled = double_even_number(4)?; // if Err, return early
    Ok(doubled + 1)
}

fn main() {
    println!("{:?}", wrapper()); // Ok(9)
}
```

📘 Use `?` to simplify nested `Result` logic.

---

## Transformation Methods

---

### Transform Ok with .map()

```rust
fn main() {
    let result = Ok(2).map(|x| x * 5);
    println!("{:?}", result); // Ok(10)
}
```

---

### Transform Err with .maperr()

```rust
fn main() {
    let err: Result<i32, &str> = Err("oops");
    let handled = err.map_err(|e| format!("Error: {}", e));
    println!("{:?}", handled); // Err("Error: oops")
}
```

---

### Chain Results with .andthen()

```rust
fn square(n: i32) -> Result<i32, &'static str> {
    Ok(n * n)
}

fn main() {
    let res = Ok(3).and_then(square).and_then(square);
    println!("{:?}", res); // Ok(81)
}
```

📘 Great for building pipelines of fallible steps.

---

## Conversions & Utilities

---

### Convert from Option<T> with .okor()

```rust
fn main() {
    let maybe = Some("hello");
    let res: Result<_, &str> = maybe.ok_or("was none");
    println!("{:?}", res); // Ok("hello")
}
```

---

### Convert Result to Option with .ok()

```rust
fn main() {
    let res: Result<i32, &str> = Ok(42);
    let opt = res.ok();
    println!("{:?}", opt); // Some(42)
}
```

---

### Early Return in Loops with ?

```rust
fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, std::num::ParseIntError> {
    inputs.iter().map(|s| s.parse()).collect()
}

fn main() {
    let inputs = vec!["1", "2", "3"];
    let numbers = parse_all(&inputs).unwrap();
    println!("{:?}", numbers);
}
```

📘 Works because `collect()` is implemented for `Result`.

---

## Pattern Matching Shortcuts

---

### if let Ok(val) / if let Err(e)

```rust
fn main() {
    let res: Result<i32, &str> = Ok(7);

    if let Ok(n) = res {
        println!("Got value: {}", n);
    }
}
```

---

### Check for Success/Failure

```rust
fn main() {
    let r: Result<i32, &str> = Err("fail");
    println!("Is ok? {}", r.is_ok());
    println!("Is err? {}", r.is_err());
}
```

---

## Summary Table

| Method                 | Purpose                                 |
| ---------------------- | --------------------------------------- |
| `unwrap()`             | Panic on `Err`                          |
| `unwrap_or(x)`         | Use default if `Err`                    |
| `map()`                | Transform the `Ok` value                |
| `map_err()`            | Transform the `Err` value               |
| `and_then()`           | Chain fallible computations             |
| `ok()`                 | Convert to `Option<T>`                  |
| `ok_or()`              | Convert `Option<T>` into `Result<T, E>` |
| `is_ok()` / `is_err()` | Check result type                       |

---

## Use Cases

* Return from fallible parsing functions
* Represent recoverable errors
* Use in CLI tools, parsers, I/O, HTTP, and more

