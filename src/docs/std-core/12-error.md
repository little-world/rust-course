Here’s a **cookbook-style tutorial for `std::error`**, Rust’s standard library module for defining and working with **custom and composable error types** using the `Error` trait.

---

## Rust std::error Cookbook

> 📦 Module: [`std::error`](https://doc.rust-lang.org/std/error/)

The key trait:

```rust
pub trait Error: Debug + Display {
    fn source(&self) -> Option<&(dyn Error + 'static)> { ... }
}
```

---

## Essentials

---

### The Error Trait

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct MyError;

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "something went wrong")
    }
}

impl Error for MyError {}

fn do_something() -> Result<(), MyError> {
    Err(MyError)
}
```

📘 Implement `Debug`, `Display`, and optionally `source()` for nesting.

---

### Return Boxed Error

```rust
fn fallible() -> Result<(), Box<dyn std::error::Error>> {
    let num: i32 = "abc".parse()?; // parse::<i32> returns Result
    println!("{}", num);
    Ok(())
}
```

📘 `Box<dyn Error>` is useful when you want to return **multiple error types**.

---

### Using source() to Chain Errors

```rust
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct WrapperError {
    source: std::io::Error,
}

impl fmt::Display for WrapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IO failed")
    }
}

impl Error for WrapperError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}
```

📘 Enables structured error reporting (`cause`/`source` chains).

---

## Custom Error Types

---

### Custom Error Enum

```rust
use std::fmt;

#[derive(Debug)]
enum AppError {
    NotFound,
    ParseError(std::num::ParseIntError),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::NotFound => write!(f, "not found"),
            AppError::ParseError(_) => write!(f, "parse error"),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::ParseError(e) => Some(e),
            _ => None,
        }
    }
}
```

📘 Enums are great for grouping different error kinds.

---

### Use ? with Custom Errors via From

```rust
impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> AppError {
        AppError::ParseError(err)
    }
}

fn parse_stuff(s: &str) -> Result<i32, AppError> {
    let num = s.parse()?; // will be converted to AppError
    Ok(num)
}
```

---

## Using Error Helpers

---

### Use anyhow for Easy Error Handling

```rust
use anyhow::{Result, Context};

fn do_something() -> Result<()> {
    let file = std::fs::read_to_string("missing.txt")
        .context("Failed to read file")?;
    println!("{}", file);
    Ok(())
}
```

📘 `anyhow` simplifies error propagation and formatting.

---

### Use thiserror for Derive-Based Custom Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
enum MyError {
    #[error("not found")]
    NotFound,

    #[error("parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}
```

📘 Great for clean, readable error enums with less boilerplate.

---

## Summary Table

| Trait/Tool          | Purpose                           |
| ------------------- | --------------------------------- |
| `std::error::Error` | Base trait for custom error types |
| `Box<dyn Error>`    | Return multiple error types       |
| `source()`          | Chain inner errors                |
| `From<T>`           | Use `?` with conversions          |
| `anyhow` crate      | Convenient, dynamic errors        |
| `thiserror` crate   | Easy custom error definition      |

---

## Use When

* You need **structured error reporting**
* You want to **wrap or convert errors**
* You're building a **library or CLI with good diagnostics**
