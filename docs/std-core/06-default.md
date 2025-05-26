Here’s a **cookbook-style tutorial for `std::default`**, the Rust standard library module that provides the `Default` trait — used to create sensible default values for types.

---

## Rust std::default Cookbook

> 📦 Module: [`std::default`](https://doc.rust-lang.org/std/default/)

The `Default` trait defines a single method:

```rust
fn default() -> Self;
```

It's commonly used in:

* Struct initialization
* Generic code
* Builders / configs
* Data deserialization (e.g., `serde`)

---

## Recipes and Examples

---

### Use T::default() on Built-in Types

```rust
fn main() {
    let n: i32 = Default::default();     // 0
    let b: bool = Default::default();    // false
    let s: String = Default::default();  // ""
    let v: Vec<u8> = Default::default(); // empty vector

    println!("{n} {b} '{s}' {:?}", v);
}
```

📘 All primitives and standard containers implement `Default`.

---

### Use .unwrapordefault() for Fallbacks

```rust
fn main() {
    let maybe_name: Option<String> = None;
    let name = maybe_name.unwrap_or_default();
    println!("Name: '{}'", name); // Name: ''
}
```

📘 Often used to simplify fallback logic with `Option` or `Result`.

---

### Derive Default for Structs

```rust
#[derive(Default, Debug)]
struct Config {
    verbose: bool,
    threads: usize,
}

fn main() {
    let cfg = Config::default();
    println!("{:?}", cfg);
}
```

📘 All fields must also implement `Default`.

---

### Partial Struct Initialization with ..Default::default()

```rust
#[derive(Default, Debug)]
struct ServerConfig {
    host: String,
    port: u16,
    timeout: u64,
}

fn main() {
    let cfg = ServerConfig {
        host: "localhost".into(),
        ..Default::default()
    };

    println!("{:?}", cfg);
}
```

📘 Great for builder-style patterns.

---

### Manually Implement Default for Custom Logic

```rust
#[derive(Debug)]
struct Custom {
    id: u32,
    label: String,
}

impl Default for Custom {
    fn default() -> Self {
        Custom {
            id: 1000,
            label: "N/A".into(),
        }
    }
}

fn main() {
    let c = Custom::default();
    println!("{:?}", c);
}
```

📘 Useful when you want defaults that aren’t just `0`, `false`, etc.

---

### Default with Generic Types

```rust
fn new_with_default<T: Default>() -> T {
    T::default()
}

fn main() {
    let v: Vec<i32> = new_with_default();
    println!("{:?}", v);
}
```

📘 Enables highly reusable, zero-config constructors.

---

### Use Default in Option<T> Initialization

```rust
fn main() {
    let maybe_val: Option<i32> = None;
    let val = maybe_val.unwrap_or_else(i32::default);
    println!("Value: {}", val); // 0
}
```

---

### Use Default in Structs with Optional Fields

```rust
#[derive(Default, Debug)]
struct AppConfig {
    host: String,
    port: u16,
    enable_logs: bool,
}

fn load_config() -> AppConfig {
    AppConfig {
        port: 8080,
        ..Default::default()
    }
}

fn main() {
    let config = load_config();
    println!("{:?}", config);
}
```

---

## Summary Table

| Type/Class        | Default Value            |
| ----------------- | ------------------------ |
| `i32`, `u8`, etc. | `0`                      |
| `bool`            | `false`                  |
| `String`          | `""` (empty string)      |
| `Vec<T>`          | `vec![]`                 |
| `Option<T>`       | `None`                   |
| `Result<T, E>`    | `Ok(Default::default())` |
| Custom struct     | Implement or derive      |

---
