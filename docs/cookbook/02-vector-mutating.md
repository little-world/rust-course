Here's a **cookbook-style tutorial** focused on **Rust's `Vec<T>` methods that mutate the vector in place**. These are useful when performance and memory efficiency matter.

---

## Rust Vector In-Place Mutation Cookbook

Each recipe includes:

* ✅ **Problem**
* 🛠️ **Method**
* 🔢 **Code Example**
* 📘 **Explanation**

---

### push Add an Element

**✅ Problem**: Add an item to the end.

```rust
fn main() {
    let mut v = vec![1, 2];
    v.push(3);
    println!("{:?}", v); // [1, 2, 3]
}
```

**📘 Explanation**: Appends an item, resizing if needed.

---

### pop Remove the Last Element

**✅ Problem**: Remove the last item.

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let last = v.pop();
    println!("{:?}, popped: {:?}", v, last); // [1, 2], Some(3)
}
```

**📘 Explanation**: Returns `None` if empty.

---

### insert Insert at Index

**✅ Problem**: Insert at a specific position.

```rust
fn main() {
    let mut v = vec![1, 3];
    v.insert(1, 2);
    println!("{:?}", v); // [1, 2, 3]
}
```

**📘 Explanation**: Panics if index > length.

---

### remove Remove by Index

**✅ Problem**: Remove a specific element.

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let x = v.remove(1);
    println!("{:?}, removed: {}", v, x); // [1, 3], 2
}
```

---

### retain Keep Matching Elements

**✅ Problem**: Keep only even numbers.

```rust
fn main() {
    let mut v = vec![1, 2, 3, 4];
    v.retain(|&x| x % 2 == 0);
    println!("{:?}", v); // [2, 4]
}
```

---

### clear Empty the Vector

**✅ Problem**: Clear all contents.

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    v.clear();
    println!("{:?}", v); // []
}
```

---

### sort Sort the Vector

**✅ Problem**: Sort in ascending order.

```rust
fn main() {
    let mut v = vec![3, 1, 2];
    v.sort();
    println!("{:?}", v); // [1, 2, 3]
}
```

---

### sortby Custom Sort

**✅ Problem**: Sort in descending order.

```rust
fn main() {
    let mut v = vec![1, 3, 2];
    v.sort_by(|a, b| b.cmp(a));
    println!("{:?}", v); // [3, 2, 1]
}
```

---

### dedup Remove Consecutive Duplicates

**✅ Problem**: Remove duplicates in adjacent positions.

```rust
fn main() {
    let mut v = vec![1, 1, 2, 2, 3];
    v.dedup();
    println!("{:?}", v); // [1, 2, 3]
}
```

---

### reverse Reverse Elements

**✅ Problem**: Reverse the vector.

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    v.reverse();
    println!("{:?}", v); // [3, 2, 1]
}
```

---

### resize Resize with Default Values

**✅ Problem**: Make the vector exactly length 5.

```rust
fn main() {
    let mut v = vec![1, 2];
    v.resize(5, 0);
    println!("{:?}", v); // [1, 2, 0, 0, 0]
}
```

---

### truncate Keep Only the First N

**✅ Problem**: Keep only the first 2 elements.

```rust
fn main() {
    let mut v = vec![1, 2, 3, 4];
    v.truncate(2);
    println!("{:?}", v); // [1, 2]
}
```

---

### drain Remove a Range

**✅ Problem**: Remove elements from index 1 to 3.

```rust
fn main() {
    let mut v = vec![1, 2, 3, 4];
    let drained: Vec<_> = v.drain(1..3).collect();
    println!("Remaining: {:?}, Removed: {:?}", v, drained); // [1, 4], [2, 3]
}
```