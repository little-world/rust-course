
# Rust Slices A Practical Tutorial

---

## What is a Slice?

A **slice** is a reference to a **part of a collection**, like an array or a string.
It doesn't own data—just points to a portion of it.

```rust
let arr = [10, 20, 30, 40];
let slice = &arr[1..3];  // includes index 1, excludes 3 → [20, 30]
```

---

## Slicing Basics

### Syntax

```rust
&collection[start..end]  // from start (inclusive) to end (exclusive)
```

### Example with Arrays

```rust
let numbers = [1, 2, 3, 4, 5];
let part = &numbers[1..4];  // &[2, 3, 4]
```

### Example with Strings

```rust
let text = "hello world";
let hello = &text[0..5];  // "hello"
```

**⚠️ Caution:** slicing strings must respect character boundaries (no halfway UTF-8 chars).

---

## Full Slicing Forms

```rust
let data = [1, 2, 3, 4, 5];

let full = &data[..];       // All elements
let from = &data[2..];      // From index 2 to end
let to = &data[..3];        // From start to index 3 (exclusive)
let mid = &data[1..4];      // Indexes 1 to 3
```

---

## Immutable vs Mutable Slices

### Immutable

```rust
let v = vec![10, 20, 30];
let s: &[i32] = &v[0..2];  // Can read, not write
```

### Mutable

```rust
let mut v = vec![1, 2, 3];
let s: &mut [i32] = &mut v[0..2];
s[0] = 99;
```

---

## Slice Methods

### .len()

```rust
let s = &[10, 20, 30];
println!("Length: {}", s.len());  // 3
```

### .isempty()

```rust
if s.is_empty() {
    println!("Empty slice");
}
```

### .iter() Iterate over items

```rust
for val in s.iter() {
    println!("Value: {val}");
}
```

### .get(index) Safe access

```rust
let s = &[1, 2, 3];
if let Some(val) = s.get(1) {
    println!("Value: {val}");
}
```

---

## When to Use Slices

Use slices when:

* You want a view into part of an array or string
* You want to avoid copying data
* You want to write generic functions

---

## Example: Generic Function with Slices

```rust
fn print_all(items: &[i32]) {
    for item in items {
        println!("{item}");
    }
}

let data = [10, 20, 30, 40];
print_all(&data[1..3]);  // prints 20 and 30
```

---

## Slices in Strings

### Use carefully with UTF-!

```rust
let s = "Здравствуйте";  // 24 bytes, 12 characters
let part = &s[0..4];     // valid: gives 2 chars (Зд)
```

### This panics:

```rust
let bad = &s[0..1];  // ❌ index not on character boundary
```

---

## Test if Two Slices Are Equal

```rust
let a = &[1, 2, 3];
let b = &[1, 2, 3];
assert_eq!(a, b);  // true
```

---

## Bonus: Split a Slice

```rust
let nums = [1, 2, 3, 4, 5];
let (first, second) = nums.split_at(3);

assert_eq!(first, &[1, 2, 3]);
assert_eq!(second, &[4, 5]);
```

---

## Summary

✅ Slices:

* Borrow part of a collection
* Avoid data copying
* Work well in generic code
* Are safer with bounds checking
* Use `&[T]` or `&str` for array/string slices
* Must respect UTF-8 boundaries for strings