Here’s a **cookbook-style tutorial for `std::iter`**, the Rust standard library module for **iterators and iterator adaptors**. Iterators in Rust are powerful, lazy, and composable — great for transforming, filtering, and collecting data efficiently.

---

## Rust std::iter Cookbook

> 📦 Module: [`std::iter`](https://doc.rust-lang.org/std/iter/)

Main features:

* `Iterator` trait
* Lazy adaptors: `.map()`, `.filter()`, `.take()`, etc.
* Consumers: `.collect()`, `.sum()`, `.any()`, etc.
* Infinite and repeatable generators

---

## Iterator Basics

---

### Loop Over a Vector

```rust
fn main() {
    let v = vec![1, 2, 3];
    for x in v.iter() {
        println!("{}", x);
    }
}
```

📘 `.iter()` gives `&T`; `.iter_mut()` gives `&mut T`; `.into_iter()` gives `T`.

---

### Map and Collect

```rust
fn main() {
    let squares: Vec<_> = (1..=5).map(|x| x * x).collect();
    println!("{:?}", squares); // [1, 4, 9, 16, 25]
}
```

📘 `.collect()` transforms an iterator into a container like `Vec`, `HashSet`, etc.

---

### Filter Values

```rust
fn main() {
    let evens: Vec<_> = (1..=10).filter(|x| x % 2 == 0).collect();
    println!("{:?}", evens); // [2, 4, 6, 8, 10]
}
```

---

### Sum and Product

```rust
fn main() {
    let sum: i32 = (1..=5).sum();
    let product: i32 = (1..=5).product();
    println!("Sum: {}, Product: {}", sum, product); // 15, 120
}
```

---

### Find First Match

```rust
fn main() {
    let nums = [1, 3, 5, 6, 7];
    if let Some(n) = nums.iter().find(|&&x| x % 2 == 0) {
        println!("First even: {}", n); // 6
    }
}
```

---

## Iterator Adaptors

---

### Zip Two Iterators

```rust
fn main() {
    let names = ["Alice", "Bob"];
    let scores = [10, 20];
    let zipped: Vec<_> = names.iter().zip(scores.iter()).collect();
    println!("{:?}", zipped); // [("Alice", 10), ("Bob", 20)]
}
```

---

### Chain Iterators

```rust
fn main() {
    let all: Vec<_> = [1, 2].iter().chain([3, 4].iter()).collect();
    println!("{:?}", all); // [1, 2, 3, 4]
}
```

---

### Enumerate with Index

```rust
fn main() {
    for (i, v) in ["a", "b", "c"].iter().enumerate() {
        println!("{}: {}", i, v);
    }
}
```

---

### Take and Skip

```rust
fn main() {
    let taken: Vec<_> = (1..).skip(3).take(4).collect();
    println!("{:?}", taken); // [4, 5, 6, 7]
}
```

📘 Powerful for slicing infinite or large sequences.

---

### Flatten Nested Iterators

```rust
fn main() {
    let nested = vec![vec![1, 2], vec![3, 4]];
    let flat: Vec<_> = nested.into_iter().flatten().collect();
    println!("{:?}", flat); // [1, 2, 3, 4]
}
```

---

## Creating Your Own Iterator

---

### Custom Struct That Implements Iterator

```rust
struct Counter {
    current: u32,
}

impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < 5 {
            self.current += 1;
            Some(self.current)
        } else {
            None
        }
    }
}

fn main() {
    let mut c = Counter { current: 0 };
    for n in c {
        println!("{}", n);
    }
}
```

---

### From Function: std::iter::successors()

```rust
fn main() {
    let powers = std::iter::successors(Some(1), |n| n.checked_mul(2));
    for val in powers.take(5) {
        println!("{}", val); // 1, 2, 4, 8, 16
    }
}
```

---

## Infinite and Repeatable Iterators

---

### Use repeat() to Fill Values

```rust
fn main() {
    let infinite = std::iter::repeat("A");
    for s in infinite.take(3) {
        println!("{}", s); // A A A
    }
}
```

---

### Cycle an Iterator

```rust
fn main() {
    let repeated = [1, 2, 3].iter().cycle();
    for x in repeated.take(7) {
        println!("{}", x);
    }
}
```

---

## Summary Table of Common Iterator Methods

| Method         | Purpose                              |
| -------------- | ------------------------------------ |
| `.map()`       | Transform each element               |
| `.filter()`    | Keep elements that match a condition |
| `.take(n)`     | Take the first `n` items             |
| `.skip(n)`     | Skip the first `n` items             |
| `.enumerate()` | Add indices                          |
| `.zip()`       | Combine two iterators                |
| `.chain()`     | Concatenate iterators                |
| `.sum()`       | Add elements                         |
| `.collect()`   | Turn into a `Vec`, `HashMap`, etc.   |


The `itertools` module (a key part of Rust's `std::iter` equivalent in Python) only exposes one function as a direct built-in via `inspect`: `tee`. This is because Python's `itertools` is a built-in C module and most of its functions are not detected via standard introspection.

However, since you're asking about **Rust's `std::iter`**, here is a list of **iterator methods and functions available in `std::iter`**, broken down into categories:

---

## Core Methods from the Iterator Trait

These are available on any type that implements `Iterator`:

### Consumers (execute the iterator)

* `all`
* `any`
* `count`
* `fold`
* `for_each`
* `last`
* `max`
* `min`
* `nth`
* `position`
* `product`
* `sum`
* `collect`
* `find`

### Adaptors (return another iterator)

* `map`
* `filter`
* `filter_map`
* `take`
* `take_while`
* `skip`
* `skip_while`
* `enumerate`
* `peekable`
* `fuse`
* `inspect`
* `by_ref`
* `cloned`
* `copied`
* `chain`
* `zip`
* `cycle`
* `flat_map`
* `flatten`
* `rev`
* `scan`
* `step_by`
* `map_while`

---

## Functions and Structs in std::iter Module

These create iterators or help construct them:

### Functions

* `empty` – returns an empty iterator
* `once` – yields one item
* `repeat` – infinite repetition of an item
* `repeat_with` – repeat using a closure
* `successors` – generate values from previous ones
* `from_fn` – generator from a function
* `once_with` – lazy single value
* `zip` – zip multiple iterators
* `chain` – chain multiple iterators
