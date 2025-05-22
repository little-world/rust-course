Here’s a **Rust Cookbook-style tutorial** on **standard math functions** from the `std` library, covering both integer and floating-point utilities.

Rust doesn’t have a unified `std::math` module, but standard math functions are mostly implemented as:

* **methods on primitive numeric types**, like `f64::sqrt()`
* **constants and traits** in modules like `std::f32`, `std::f64`

---

## Basic Math Operations

Rust supports the usual arithmetic operators:

```rust
fn main() {
    let a = 10;
    let b = 3;

    println!("Add: {}", a + b);
    println!("Sub: {}", a - b);
    println!("Mul: {}", a * b);
    println!("Div: {}", a / b);
    println!("Mod: {}", a % b);
}
```

For floats:

```rust
let x = 5.5;
let y = 2.0;
println!("Floating-point division: {}", x / y);
```

---

## Float Methods

```rust
fn main() {
    let x = 2.0_f64;

    println!("sqrt: {}", x.sqrt());
    println!("powf: {}", x.powf(3.0)); // x^3
    println!("exp: {}", x.exp());     // e^x
    println!("ln: {}", x.ln());       // natural log
    println!("log10: {}", x.log10()); // log base 10
    println!("sin: {}", x.sin());
    println!("cos: {}", x.cos());
}
```

All these methods are available on `f32` and `f64`.

---

## Constants

```rust
fn main() {
    println!("PI: {}", std::f64::consts::PI);
    println!("E: {}", std::f64::consts::E);
}
```

Other constants in `std::f64::consts`:

* `FRAC_PI_2` (π/2), `LN_2`, `SQRT_2`, etc.

---

## Min, Max, Clamp

```rust
fn main() {
    let x = 42;
    println!("Min: {}", x.min(10));       // returns 10
    println!("Max: {}", x.max(100));      // returns 100
    println!("Clamp: {}", x.clamp(0, 50)); // restricts x to [0, 50]
}
```

Same methods are available for `f32`, `f64`, etc.

---

## Rounding Functions

```rust
fn main() {
    let x = 3.7_f64;

    println!("floor: {}", x.floor()); // 3.0
    println!("ceil: {}", x.ceil());   // 4.0
    println!("round: {}", x.round()); // 4.0
    println!("trunc: {}", x.trunc()); // 3.0
    println!("fract: {}", x.fract()); // 0.7
}
```

---

## Absolute Value and Sign

```rust
fn main() {
    let x = -42;
    println!("abs: {}", x.abs());
    println!("signum: {}", (-3.5f64).signum()); // -1.0
}
```

---

## Integer Utilities

```rust
fn main() {
    let x: i32 = -10;
    let y: u32 = 0b1010;

    println!("abs: {}", x.abs());
    println!("count_ones: {}", y.count_ones());
    println!("leading_zeros: {}", y.leading_zeros());
    println!("trailing_zeros: {}", y.trailing_zeros());
}
```

---

## Checked/Overflow Math

```rust
fn main() {
    let a: u8 = 250;

    println!("Checked add: {:?}", a.checked_add(10));
    println!("Saturating add: {}", a.saturating_add(10));
    println!("Wrapping add: {}", a.wrapping_add(10));
}
```

---

## Bonus: Random Numbers (using rand crate)

While not in `std`, it’s common in math-related code:

```toml
# Cargo.toml
[dependencies]
rand = "0.8"
```

```rust
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let x: f64 = rng.gen(); // random float [0.0, 1.0)
    let y = rng.gen_range(1..=10); // random integer in range
    println!("Random f64: {}, Random int: {}", x, y);
}
```