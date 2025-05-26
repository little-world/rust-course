--

##  Generics

**Generics** let you write code that works for **any type**. Like templates in C++ or generics in Java.

### Without Generics:

```rust
fn square_i32(x: i32) -> i32 {
    x * x
}
```

### With Generics:

```rust
fn square<T: std::ops::Mul<Output = T> + Copy>(x: T) -> T {
    x * x
}
```

> This works for any type `T` that can be multiplied and copied.



##  Functions

```rust
fn print_item<T>(item: T)
where
    T: std::fmt::Debug,
{
    println!("{:?}", item);
}
```

### Usage:

```rust
print_item(42);          // i32
print_item("Hello");     // &str
```



##  Structs

```rust
struct Point<T> {
    x: T,
    y: T,
}

fn main() {
    let int_point = Point { x: 1, y: 2 };
    let float_point = Point { x: 1.1, y: 2.2 };
}
```



###  Enums 
#### (e.g., Option, Result)

You’ve already used them!

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

You can define your own:

```rust
enum Wrapper<T> {
    One(T),
    Many(Vec<T>),
}
```



##  Methods

Add methods to generic structs:

```rust
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}
```

You can restrict method availability to certain types:

```rust
impl Point<f64> {
    fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```


## Traits Bounds

Use `T: Trait` to **restrict** a generic type:

```rust
fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T {
    a + b
}
```

Or with `where` for cleaner style:

```rust
fn describe<T>(item: T)
where
    T: std::fmt::Display + std::fmt::Debug,
{
    println!("Item: {}, {:?}", item, item);
}
```

---

## Summary

| Use Case          | Syntax Example                         |
| ----------------- | -------------------------------------- |
| Generic function  | `fn f<T>(x: T) {}`                     |
| Trait bounds      | `T: Display` or `where T: Display`     |
| Generic struct    | `struct S<T> { field: T }`             |
| Generic method    | `impl<T> Struct<T> { fn x(&self) {} }` |
| Multiple generics | `fn f<T, U>(a: T, b: U)`               |
| Restrict types    | `impl Struct<f64>`                     |