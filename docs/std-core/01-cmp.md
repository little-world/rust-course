Here’s a **cookbook-style guide for `std::cmp`**, which is part of Rust’s standard library and provides **comparison traits and utilities** like `PartialEq`, `PartialOrd`, `Eq`, `Ord`, and tools like `min`, `max`, and `Ordering`.

---

## Rust std::cmp Cookbook

> 📦 Module: [`std::cmp`](https://doc.rust-lang.org/std/cmp/index.html)

Each recipe includes:

* ✅ **Problem**
* 🔧 **Solution**
* 📘 **Explanation**

---

### Compare Two Values with == and !=

```rust
fn main() {
    let a = 10;
    let b = 20;
    println!("Equal? {}", a == b);
    println!("Not equal? {}", a != b);
}
```

📘 Uses the `PartialEq` trait automatically derived for most types.

---

### Use > < >= <= with Numbers

```rust
fn main() {
    let x = 3.5;
    let y = 2.0;
    println!("x > y? {}", x > y);
}
```

📘 Floats use `PartialOrd`, since `NaN` makes some comparisons invalid.

---

### Get Ordering Result with cmp()

```rust
use std::cmp::Ordering;

fn main() {
    let a = 3;
    let b = 5;

    match a.cmp(&b) {
        Ordering::Less => println!("a < b"),
        Ordering::Equal => println!("a == b"),
        Ordering::Greater => println!("a > b"),
    }
}
```

📘 Uses the `Ord` trait, which provides total ordering.

---

### Derive Comparison Traits for Structs

```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let a = Point { x: 1, y: 2 };
    let b = Point { x: 1, y: 3 };
    println!("a < b? {}", a < b);
}
```

📘 Deriving `Ord` gives lexicographic ordering (`x` then `y`).

---

### Custom Sort with cmp() and Ordering

```rust
use std::cmp::Ordering;

#[derive(Debug)]
struct Item {
    name: String,
    priority: u8,
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority) // descending
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Item {}

fn main() {
    let mut items = vec![
        Item { name: "A".into(), priority: 1 },
        Item { name: "B".into(), priority: 3 },
    ];

    items.sort();
    println!("{:?}", items);
}
```

📘 Required when you need custom logic or reverse sort.

---

### Find Minimum / Maximum Values

```rust
use std::cmp::{min, max};

fn main() {
    let a = 8;
    let b = 5;
    println!("min: {}", min(a, b));
    println!("max: {}", max(a, b));
}
```

📘 Works with any type implementing `Ord`.

---

### Clamp a Value Between Min and Max

```rust
fn main() {
    let value = 15;
    let clamped = value.clamp(0, 10);
    println!("Clamped: {}", clamped); // 10
}
```

📘 Added in Rust 1.50+. Uses `Ord`.

---

### Manually Implement PartialEq for Custom Logic

```rust
struct CaseInsensitiveStr<'a>(&'a str);

impl<'a> PartialEq for CaseInsensitiveStr<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

fn main() {
    let a = CaseInsensitiveStr("Rust");
    let b = CaseInsensitiveStr("rust");
    println!("Equal? {}", a == b);
}
```

📘 Customize equality behavior without modifying the original type.

---

### Sort by a Field with sortbykey

```rust
#[derive(Debug)]
struct User {
    name: String,
    age: u8,
}

fn main() {
    let mut users = vec![
        User { name: "Alice".into(), age: 30 },
        User { name: "Bob".into(), age: 25 },
    ];

    users.sort_by_key(|u| u.age);
    println!("{:?}", users);
}
```

📘 Cleaner than implementing `Ord` when sorting on one field.

---

### Floating-Point Comparison with partialcmp

```rust
fn main() {
    let a = 2.0;
    let b = f64::NAN;

    match a.partial_cmp(&b) {
        Some(order) => println!("Order: {:?}", order),
        None => println!("Cannot compare (NaN)"),
    }
}
```

📘 Always use `partial_cmp` with floats due to `NaN`.

---

## Summary of Traits in std::cmp

| Trait        | Purpose                         |
| ------------ | ------------------------------- |
| `PartialEq`  | Equality check with `==`, `!=`  |
| `Eq`         | Marker trait for total equality |
| `PartialOrd` | Comparison: `<`, `>`, etc.      |
| `Ord`        | Total ordering with `cmp()`     |
