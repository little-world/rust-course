Here’s a **cookbook-style tutorial for `std::fmt`**, the Rust standard library module for **formatting and printing**. It powers macros like `println!`, `format!`, and allows you to define how your types are displayed using traits like `Display` and `Debug`.

---

## Rust std::fmt Cookbook

> 📦 Module: [`std::fmt`](https://doc.rust-lang.org/std/fmt/)

Used for:

* Console output (`println!`, `format!`)
* String formatting (`format_args!`)
* Custom printing for your types (`Display`, `Debug`, `LowerHex`, etc.)

---

## Basics

---

### Use println! with Format Strings

```rust
fn main() {
    let name = "Rust";
    let version = 2021;
    println!("Language: {}, Version: {}", name, version);
}
```

📘 Works just like Python’s `format()` or C’s `printf`.

---

### format! Returns a String

```rust
fn main() {
    let s = format!("{} + {} = {}", 2, 2, 2 + 2);
    println!("{}", s);
}
```

📘 `format!` does not print; it builds a `String`.

---

## Formatting Traits

---

### Debug Trait with {:?}

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 1, y: 2 };
    println!("{:?}", p);
}
```

📘 Use `#[derive(Debug)]` for quick inspection.

---

### Pretty-Print with {:#?}

```rust
fn main() {
    let data = vec!["apple", "banana", "pear"];
    println!("{:#?}", data);
}
```

📘 Pretty-printed `Debug` output — useful for complex types.

---

### Implement Display for Custom Types

```rust
use std::fmt;

struct User {
    name: String,
    age: u8,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.age)
    }
}

fn main() {
    let user = User { name: "Alice".into(), age: 30 };
    println!("{}", user); // uses Display
}
```

📘 Required for human-readable output.

---

## Format Specifiers

---

### Format Numbers

```rust
fn main() {
    let n = 42;
    println!("Decimal: {}", n);
    println!("Hex: {:x}", n);
    println!("Binary: {:b}", n);
    println!("Padded: {:05}", n); // 00042
}
```

📘 Many formatting options available: width, alignment, precision, etc.

---

### Format Floating Points

```rust
fn main() {
    let pi = 3.14159;
    println!("Default: {}", pi);
    println!("Fixed: {:.2}", pi); // 3.14
    println!("Right-aligned: {:>8.2}", pi); // "    3.14"
}
```

---

### Named Arguments in format!

```rust
fn main() {
    let msg = format!("{lang} is {adj}", lang = "Rust", adj = "awesome");
    println!("{}", msg);
}
```

---

### Pad, Align, and Justify

```rust
fn main() {
    println!("Left:  {:<5}", "hi");
    println!("Right: {:>5}", "hi");
    println!("Center:{:^5}", "hi");
}
```

📘 Useful for CLI tables and alignment.

---

## Advanced std::fmt Usage

---

### Implement LowerHex, Binary, etc.

```rust
use std::fmt;

struct Id(u32);

impl fmt::LowerHex for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

fn main() {
    let id = Id(255);
    println!("{:x}", id); // ff
}
```

---

### Write to a Custom Buffer

```rust
use std::fmt::Write;

fn main() {
    let mut s = String::new();
    write!(s, "Hello {}", "world").unwrap();
    println!("{}", s);
}
```

📘 Use `std::fmt::Write` to build strings efficiently.

---

### Use formatargs! for Custom Logging

```rust
use std::fmt::Arguments;

fn log(args: Arguments) {
    println!("[LOG] {}", args);
}

fn main() {
    log(format_args!("Hello, {}", "Rust"));
}
```

📘 Avoids heap allocation compared to `format!`.

---

## Summary: Format Traits

| Trait      | Example Usage    | Meaning          |
| ---------- | ---------------- | ---------------- |
| `Display`  | `{}`             | Human-readable   |
| `Debug`    | `{:?}` / `{:#?}` | Debug formatting |
| `Binary`   | `{:b}`           | Binary           |
| `LowerHex` | `{:x}`           | Lowercase hex    |
| `UpperHex` | `{:X}`           | Uppercase hex    |
| `Pointer`  | `{:p}`           | Memory address   |

