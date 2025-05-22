Here’s a **cookbook-style tutorial for `std::convert`**, a Rust standard library module that provides **traits for type conversions** — both infallible (`From`, `Into`) and fallible (`TryFrom`, `TryInto`).

---

## Rust std::convert Cookbook

> 📦 Module: [`std::convert`](https://doc.rust-lang.org/std/convert/)

Key traits:

* `From<T>` – for infallible conversion **into** Self
* `Into<U>` – for infallible conversion **from** Self into U
* `TryFrom<T>` – for fallible conversion **into** Self
* `TryInto<U>` – for fallible conversion **from** Self into U
* `AsRef<T>` / `AsMut<T>` – for cheap borrowing conversions

---

### Rule of Thumb

If you implement `From<T> for U`, then `T: Into<U>` **automatically**.

---

## Infallible Conversions (From, Into)

---

### Use .into() to Convert Types

```rust
fn main() {
    let s: String = "hello".into(); // &str -> String
    println!("{}", s);
}
```

📘 `&str` implements `Into<String>` via `From<&str> for String`.

---

### Implement From<T> for Custom Type

```rust
struct UserId(String);

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        UserId(s.to_uppercase())
    }
}

fn main() {
    let id: UserId = "bob123".into();
}
```

📘 This allows ergonomic `.into()` or `UserId::from(...)` conversions.

---

### Convert Number Types with From

```rust
fn main() {
    let x = u8::from(5u16); // Error: From<u16> for u8 is not implemented
    let x = 5u16 as u8;      // Use casting for numeric narrowing

    let x: u32 = 42u8.into(); // Works: widening
    println!("{}", x);
}
```

📘 Rust does not auto-implement narrowing `From`; use `TryFrom`.

---

### Convert to String with ToString and From

```rust
fn main() {
    let n = 42;
    let s = n.to_string(); // uses `ToString` -> `Display`
    let s2 = String::from("42");
    println!("{} {}", s, s2);
}
```

📘 All `T: Display` automatically implement `ToString`.

---

## Fallible Conversions (TryFrom, TryInto)

---

### Use TryInto for Checked Conversion

```rust
use std::convert::TryInto;

fn main() {
    let big: u16 = 300;
    let small: u8 = big.try_into().expect("Out of range");

    println!("{}", small);
}
```

📘 Panics only if the value doesn’t fit in target type.

---

### Implement TryFrom<T> for Custom Type

```rust
use std::convert::TryFrom;

struct Positive(u32);

impl TryFrom<i32> for Positive {
    type Error = &'static str;

    fn try_from(n: i32) -> Result<Self, Self::Error> {
        if n >= 0 {
            Ok(Positive(n as u32))
        } else {
            Err("Negative number not allowed")
        }
    }
}

fn main() {
    let p = Positive::try_from(5).unwrap();
    // let p = Positive::try_from(-5).unwrap(); // panics
}
```

---

## Borrow Conversions (AsRef, AsMut)

---

### Use AsRef<T> for Generic Borrowing

```rust
fn print_bytes<T: AsRef<[u8]>>(input: T) {
    println!("{:?}", input.as_ref());
}

fn main() {
    print_bytes("hello");         // &str → &[u8]
    print_bytes(b"hello");        // &[u8]
    print_bytes(vec![104, 101]);  // Vec<u8> → &[u8]
}
```

📘 Ideal for APIs that accept multiple borrowed types.

---

### Use AsMut<T> for Generic Mutable Access

```rust
fn zero_out<T: AsMut<[u8]>>(buffer: &mut T) {
    for byte in buffer.as_mut() {
        *byte = 0;
    }
}

fn main() {
    let mut data = vec![1, 2, 3];
    zero_out(&mut data);
    println!("{:?}", data); // [0, 0, 0]
}
```

---

## Summary Table

| Trait        | Direction          | Fallible? | Common Use                |
| ------------ | ------------------ | --------- | ------------------------- |
| `From<T>`    | T → Self           | ❌         | Struct builders, strings  |
| `Into<U>`    | Self → U           | ❌         | Ergonomic APIs            |
| `TryFrom<T>` | T → Self           | ✅         | Safe narrowing/conversion |
| `TryInto<U>` | Self → U           | ✅         | Used with `?` operator    |
| `AsRef<T>`   | Borrow as `&T`     | ❌         | Generic, cheap access     |
| `AsMut<T>`   | Borrow as `&mut T` | ❌         | Generic mutable access    |

