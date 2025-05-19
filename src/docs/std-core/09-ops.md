Here's a **cookbook-style tutorial for `std::ops`**, the Rust standard library module that defines **operator overloading traits** — such as `Add`, `Mul`, `Index`, `Deref`, etc. This lets you implement or customize the behavior of `+`, `*`, `[]`, `*x`, and more.

---

## Rust std::ops Cookbook

> 📦 Module: [`std::ops`](https://doc.rust-lang.org/std/ops/)

---

## Operator Traits by Category

| Operator      | Trait                             | Symbol                  |          |
| ------------- | --------------------------------- | ----------------------- | -------- |
| Arithmetic    | `Add`, `Sub`, `Mul`, `Div`, `Rem` | `+`, `-`, `*`, `/`, `%` |          |
| Assignment    | `AddAssign`, `SubAssign`, etc.    | `+=`, `-=`, etc.        |          |
| Logical       | `Not`, `BitAnd`, `BitOr`, etc.    | `!`, `&`, \`            | \`, etc. |
| Indexing      | `Index`, `IndexMut`               | `[]`                    |          |
| Dereferencing | `Deref`, `DerefMut`               | `*x`                    |          |
| Range         | `Range`, `RangeInclusive`, etc.   | `a..b`, `a..=b`         |          |

---

## Recipes and Examples

---

### Implement Add for a Custom Struct

```rust
use std::ops::Add;

#[derive(Debug)]
struct Point(i32, i32);

impl Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point(self.0 + other.0, self.1 + other.1)
    }
}

fn main() {
    let p1 = Point(1, 2);
    let p2 = Point(3, 4);
    println!("{:?}", p1 + p2); // Point(4, 6)
}
```

📘 The `Add` trait allows use of the `+` operator.

---

### Implement AddAssign (+=)

```rust
use std::ops::AddAssign;

#[derive(Debug)]
struct Counter(i32);

impl AddAssign<i32> for Counter {
    fn add_assign(&mut self, rhs: i32) {
        self.0 += rhs;
    }
}

fn main() {
    let mut c = Counter(10);
    c += 5;
    println!("{:?}", c); // Counter(15)
}
```

---

### Implement Index and IndexMut

```rust
use std::ops::{Index, IndexMut};

struct Buffer {
    data: Vec<u8>,
}

impl Index<usize> for Buffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Buffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

fn main() {
    let mut buf = Buffer { data: vec![1, 2, 3] };
    buf[1] = 42;
    println!("{}", buf[1]); // 42
}
```

---

### Use Deref to Smart-Pointerify a Wrapper

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn main() {
    let x = MyBox(String::from("hello"));
    println!("{}", x.len()); // via deref coercion
}
```

📘 `Deref` lets you use `.` method syntax and coercion to references.

---

### Overload via Deref and DerefMut

```rust
use std::ops::{Deref, DerefMut};

struct Wrapper(i32);

impl Deref for Wrapper {
    type Target = i32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Wrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn main() {
    let mut w = Wrapper(10);
    *w += 5;
    println!("{}", *w); // 15
}
```

---

### Use Bitwise Operators (BitAnd, BitOr)

```rust
use std::ops::BitOr;

#[derive(Debug)]
struct Flags(u8);

impl BitOr for Flags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Flags(self.0 | rhs.0)
    }
}

fn main() {
    let a = Flags(0b0001);
    let b = Flags(0b0010);
    let c = a | b;
    println!("{:?}", c); // Flags(0b0011)
}
```

---

### Implement a Range-Like Struct

```rust
struct MyRange {
    current: i32,
    end: i32,
}

impl Iterator for MyRange {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let result = self.current;
            self.current += 1;
            Some(result)
        } else {
            None
        }
    }
}

fn main() {
    let r = MyRange { current: 0, end: 3 };
    for i in r {
        println!("{}", i); // 0, 1, 2
    }
}
```

📘 Custom iterator to simulate `a..b`.

---

## Summary Table of Key Traits

| Trait                       | Symbol              | Use Case                        |                 |
| --------------------------- | ------------------- | ------------------------------- | --------------- |
| `Add` / `AddAssign`         | `+` / `+=`          | Arithmetic or custom math types |                 |
| `Sub`, `Mul`, `Div`, `Rem`  | `-`, `*`, `/`, `%`  | Numeric overloads               |                 |
| `BitAnd`, `BitOr`, `BitXor` | `&`, \`             | `, `^\`                         | Bitmasks, flags |
| `Deref`, `DerefMut`         | `*x` / method deref | Smart pointers, coercion        |                 |
| `Index`, `IndexMut`         | `x[i]`              | Array/map access                |                 |
| `Neg`, `Not`                | `-x`, `!x`          | Negation, logic inversion       |                 |

---

## When to Use std::ops

* Custom numeric types (e.g., complex numbers, vectors)
* DSLs (domain-specific languages)
* Smart pointers, containers, or wrappers
* Mimicking built-in types with operator support
