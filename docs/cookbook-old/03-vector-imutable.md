Great! Here's a **cookbook-style guide** for **immutable transformations on vectors** in Rust — functions that **do not mutate the original vector** but instead **produce new data** from it.

These are perfect for **functional-style programming** and **safe, predictable behavior**.

---

## Rust Vector Immutable Transformations Cookbook

Each recipe includes:

* ✅ **Problem**
* 🧰 **Method**
* 🔢 **Code Example**
* 📘 **Explanation**

---

### iter().map() Transform Each Item

**✅ Problem**: Square each number.

```rust
fn main() {
    let v = vec![1, 2, 3];
    let squared: Vec<_> = v.iter().map(|x| x * x).collect();
    println!("{:?}", squared); // [1, 4, 9]
}
```

**📘 Explanation**: `map` returns an iterator; `collect` turns it back into a `Vec`.

---

### iter().filter() Select Matching Items

**✅ Problem**: Keep only even numbers.

```rust
fn main() {
    let v = vec![1, 2, 3, 4];
    let evens: Vec<_> = v.iter().filter(|&&x| x % 2 == 0).collect();
    println!("{:?}", evens); // [2, 4]
}
```

---

### iter().fold() Reduce to a Single Value

**✅ Problem**: Sum all elements.

```rust
fn main() {
    let v = vec![1, 2, 3];
    let sum = v.iter().fold(0, |acc, x| acc + x);
    println!("{}", sum); // 6
}
```

---

### iter().enumerate() Get Index and Value

**✅ Problem**: Print index and value.

```rust
fn main() {
    let v = vec!["a", "b", "c"];
    for (i, val) in v.iter().enumerate() {
        println!("{}: {}", i, val);
    }
}
```

---

### iter().zip() Combine Two Vectors

**✅ Problem**: Pair elements from two vectors.

```rust
fn main() {
    let names = vec!["Alice", "Bob"];
    let scores = vec![90, 80];
    let paired: Vec<_> = names.iter().zip(scores.iter()).collect();
    println!("{:?}", paired); // [("Alice", 90), ("Bob", 80)]
}
```

---

### clone() Copy a Vector

**✅ Problem**: Make a new copy of a vector.

```rust
fn main() {
    let v = vec![1, 2, 3];
    let v2 = v.clone();
    println!("{:?}", v2); // [1, 2, 3]
}
```

---

### chunks() Work With Sub-slices

**✅ Problem**: Break into parts of 2.

```rust
fn main() {
    let v = vec![1, 2, 3, 4, 5];
    for chunk in v.chunks(2) {
        println!("{:?}", chunk);
    }
    // [1, 2]
    // [3, 4]
    // [5]
}
```

---

### windows() Sliding Windows

**✅ Problem**: Analyze pairs.

```rust
fn main() {
    let v = vec![1, 2, 3, 4];
    for window in v.windows(2) {
        println!("{:?}", window);
    }
    // [1, 2]
    // [2, 3]
    // [3, 4]
}
```

---

### iter().any() / all() Boolean Checks

**✅ Problem**: Check if all elements are positive.

```rust
fn main() {
    let v = vec![1, 2, 3];
    let all_positive = v.iter().all(|&x| x > 0);
    println!("{}", all_positive); // true
}
```

---

### iter().find() Find the First Match

**✅ Problem**: Find first element > 2.

```rust
fn main() {
    let v = vec![1, 2, 3, 4];
    if let Some(x) = v.iter().find(|&&x| x > 2) {
        println!("Found: {}", x);
    }
}
```

---

### intoiter() Ownership-Based Transformation

**✅ Problem**: Transform and consume.

```rust
fn main() {
    let v = vec!["a", "b"];
    let upper: Vec<_> = v.into_iter().map(|s| s.to_uppercase()).collect();
    println!("{:?}", upper); // ["A", "B"]
}
```

---

### collect() Convert to Other Collections

**✅ Problem**: Convert `Vec` to `HashSet` (removes duplicates).

```rust
use std::collections::HashSet;

fn main() {
    let v = vec![1, 2, 2, 3];
    let set: HashSet<_> = v.iter().cloned().collect();
    println!("{:?}", set); // {1, 2, 3}
}
```
