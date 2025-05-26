Here's a practical **Rust Cookbook-style tutorial for `std::num`**, the module in the Rust standard library that provides numeric traits, conversions, and error types.

---

## Overview of std::num

The `std::num` module provides types and utilities for numerical operations, including:

* **Numeric Types**: `u8`, `i32`, `f64`, etc.
* **Error Types**:

  * `ParseIntError`
  * `ParseFloatError`
  * `TryFromIntError`
* **Utilities**:

  * `Wrapping<T>` – for arithmetic with wraparound.
  * `Saturating<T>` (via nightly or external crate)
  * `NonZero*` – types that can never be zero (e.g., `NonZeroU32`).

---

## Parse String to Number

```rust
fn main() -> Result<(), std::num::ParseIntError> {
    let number: i32 = "42".parse()?;
    println!("Parsed number: {}", number);
    Ok(())
}
```

🔍 **Error Handling**:

```rust
match "abc".parse::<i32>() {
    Ok(n) => println!("Parsed: {}", n),
    Err(e) => println!("Failed to parse: {}", e),
}
```

---

## Handle Integer Overflow with Wrapping

```rust
use std::num::Wrapping;

fn main() {
    let a = Wrapping(u8::MAX);
    let b = Wrapping(1);
    let result = a + b; // Wraps around to 0
    println!("Wrapped result: {}", result.0);
}
```

---

## Safe Integer Conversion

```rust
fn main() {
    let big: u16 = 300;
    let small: u8 = u8::try_from(big).unwrap_or(255); // handle overflow
    println!("Safely converted: {}", small);
}
```

Or use match for safer control:

```rust
match u8::try_from(300u16) {
    Ok(val) => println!("Converted: {}", val),
    Err(e) => println!("Conversion failed: {}", e),
}
```

---

## Use NonZero Types

```rust
use std::num::NonZeroU32;

fn main() {
    let val = NonZeroU32::new(5);
    if let Some(nz) = val {
        println!("Non-zero: {}", nz);
    } else {
        println!("Was zero");
    }
}
```

These are useful for optimizations (e.g., `Option<NonZeroU32>` has the same size as `u32` due to niche optimization).

---

## Parse Floating-Point Numbers

```rust
fn main() -> Result<(), std::num::ParseFloatError> {
    let f: f64 = "3.1415".parse()?;
    println!("Parsed float: {}", f);
    Ok(())
}
```

---

## Constants and Limits

```rust
fn main() {
    println!("Max i32: {}", i32::MAX);
    println!("Min f64: {}", f64::MIN);
}
```

---

## Bonus: Math with Edge Cases

Rust doesn't automatically check for overflows in release mode. Use:

* `checked_add`, `checked_sub` – return `None` on overflow
* `saturating_add`, `wrapping_add` – alternatives with defined behaviors

```rust
fn main() {
    let x: u8 = 250;
    println!("Checked: {:?}", x.checked_add(10));     // None
    println!("Saturating: {}", x.saturating_add(10)); // 255
    println!("Wrapping: {}", x.wrapping_add(10));     // 4
}
```

-