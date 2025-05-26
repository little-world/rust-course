
## Slices


A **slice** is a reference to a **part of a collection**, like an array or a string.
It doesn't own data—just points to a portion of it.

```rust
let arr = [10, 20, 30, 40];
let slice = &arr[1..3];  // includes index 1, excludes 3 → [20, 30]
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


## Slicing Forms

```rust
let data = [1, 2, 3, 4, 5];

let full = &data[..];       // All elements
let from = &data[2..];      // From index 2 to end
let to = &data[..3];        // From start to index 3 (exclusive)
let mid = &data[1..4];      // Indexes 1 to 3
```


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


## Methods

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

## Summary

✅ Slices:

* Borrow part of a collection
* Avoid data copying
* Work well in generic code
* Are safer with bounds checking
* Use `&[T]` or `&str` for array/string slices
* Must respect UTF-8 boundaries for strings