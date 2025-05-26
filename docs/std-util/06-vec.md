Here’s a **cookbook-style tutorial for `std::vec`**, the Rust standard module defining `Vec<T>`, Rust’s most commonly used **growable, heap-allocated, contiguous array type**.

---

## Rust std::vec Cookbook

> 📦 Module: [`std::vec`](https://doc.rust-lang.org/std/vec/)

```rust
pub struct Vec<T> { /* ... */ }
```

---

## Basics

---

### Create a Vector

```rust
fn main() {
    let v = vec![1, 2, 3];
    println!("{:?}", v);
}
```

📘 `vec![]` is a macro that allocates on the heap.

---

### Create an Empty or Preallocated Vector

```rust
fn main() {
    let mut v: Vec<i32> = Vec::new();
    let with_capacity = Vec::with_capacity(10);
}
```

---

### Push and Pop Elements

```rust
fn main() {
    let mut v = vec![1, 2];
    v.push(3);
    v.pop(); // removes 3
    println!("{:?}", v); // [1, 2]
}
```

---

### Access by Index or Get

```rust
fn main() {
    let v = vec![10, 20, 30];
    println!("{}", v[1]); // panics if out of bounds
    println!("{:?}", v.get(5)); // returns Option
}
```

---

### Iterate Over a Vector

```rust
fn main() {
    let v = vec!["a", "b", "c"];
    for item in v.iter() {
        println!("{}", item);
    }
}
```

---

## Mutation and Growth

---

### Insert and Remove at Index

```rust
fn main() {
    let mut v = vec![1, 2, 4];
    v.insert(2, 3); // [1, 2, 3, 4]
    v.remove(0);    // [2, 3, 4]
    println!("{:?}", v);
}
```

---

### Truncate and Clear

```rust
fn main() {
    let mut v = vec![1, 2, 3, 4];
    v.truncate(2);
    println!("{:?}", v); // [1, 2]
    v.clear(); // []
}
```

---

### Resize Vector

```rust
fn main() {
    let mut v = vec![1, 2];
    v.resize(4, 0); // [1, 2, 0, 0]
    println!("{:?}", v);
}
```

---

## Transformation & Access

---

### Sort and Reverse

```rust
fn main() {
    let mut v = vec![3, 1, 2];
    v.sort();
    v.reverse();
    println!("{:?}", v); // [3, 2, 1]
}
```

---

### Retain Elements Conditionally

```rust
fn main() {
    let mut v = vec![1, 2, 3, 4];
    v.retain(|&x| x % 2 == 0);
    println!("{:?}", v); // [2, 4]
}
```

---

### Map, Filter, Collect

```rust
fn main() {
    let v = vec![1, 2, 3];
    let doubled: Vec<_> = v.into_iter().map(|x| x * 2).collect();
    println!("{:?}", doubled); // [2, 4, 6]
}
```

---

## Conversion

---

### From and Into

```rust
fn main() {
    let v: Vec<i32> = (1..=3).collect();
    let arr = [1, 2, 3];
    let v2 = Vec::from(arr);
    println!("{:?}, {:?}", v, v2);
}
```

---

### Split and Join

```rust
fn main() {
    let v = vec!["a", "b", "c"];
    let joined = v.join("-");
    println!("{}", joined); // a-b-c
}
```

---

### Extend and Append

```rust
fn main() {
    let mut v1 = vec![1, 2];
    let mut v2 = vec![3, 4];
    v1.extend(&v2); // copies
    v1.append(&mut v2); // moves
    println!("{:?}, {:?}", v1, v2);
}
```

---

## Summary Table

| Method                      | Description                         |
| --------------------------- | ----------------------------------- |
| `vec![]`                    | Create a vector                     |
| `.push()` / `.pop()`        | Add/remove from end                 |
| `.insert()` / `.remove()`   | Add/remove at index                 |
| `.get()`                    | Safe access (returns `Option`)      |
| `.resize()` / `.truncate()` | Adjust size                         |
| `.sort()` / `.reverse()`    | Rearrange                           |
| `.retain()`                 | Keep elements matching condition    |
| `.into_iter()`              | Consume and iterate                 |
| `.collect()`                | Turn iterator into Vec              |
| `.from()` / `.extend()`     | Add elements from another container |

---

## Use When

* You need a **resizable array**
* You want **heap allocation**
* You need to **move** or **own** elements
* You want to use **iterator chains** and collection

---
