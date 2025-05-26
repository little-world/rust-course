
##  Variables

In Rust, variables are **immutable by default**. You use `let` to declare a variable:

```rust
fn main() {
    let x = 5; // Immutable
    println!("x is: {}", x);
    
    let mut y = 10; // Mutable
    y = 15;
    println!("y is: {}", y);
}
```

Use `mut` for **mutable** variables.

---

## Common Types

Rust is statically typed, so every variable has a type. You can let the compiler infer the type, or explicitly specify it.

### Scalar types:

| Type   | Description             | Example                  |
|--------|-------------------------|--------------------------|
| `i32`  | 32-bit signed integer   | `let x: i32 = -10;`      |
| `u32`  | 32-bit unsigned integer | `let x: u32 = 10;`       |
| `f64`  | 64-bit floating point   | `let pi: f64 = 3.14;`    |
| `bool` | Boolean                 | `let flag: bool = true;` |
| `char` | Unicode scalar          | `let c: char = '♥';`     |


### Compound types:

* **Tuple**

```rust
  let tup: (i32, f64, u8) = (500, 6.4, 1);
  let (x, y, z) = tup;
  println!("x: {}, y: {}, z: {}", x, y, z);
```

* **Array**

```rust
  let a: [i32; 3] = [1, 2, 3];
  println!("First element: {}", a[0]);
  ```

---

## Typecasting

Rust **doesn't allow implicit type coercion** between primitive types. You must **explicitly cast** using `as`.

### Integer to float:

```rust
let x: i32 = 10;
let y: f64 = x as f64;
```

### Float to integer (may lose precision):

```rust
let pi: f64 = 3.14159;
let int_pi: i32 = pi as i32; // Result: 3
```

### Between integers:

```rust
let big: i64 = 1000;
let small: u8 = big as u8; // May truncate!
```



### Example 

```rust
fn main() {
    let a: i32 = 5;
    let b: f64 = 2.5;

    let result = a as f64 + b;
    println!("Result: {}", result); // 7.5

    let c: u8 = 255;
    let d: i8 = c as i8; // -1 due to overflow (u8 -> i8)
    println!("d: {}", d);
}
```


## Summary

* Use `let` without type if obvious:

  ```rust
  let name = "Rust"; // inferred as &str
  ```

* Use explicit casting when mixing types.

* Use `.parse::<T>()` for string-to-type conversions:

  ```rust
  let input = "42";
  let number: i32 = input.parse().expect("Not a number!");
  ```
