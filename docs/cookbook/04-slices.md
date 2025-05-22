
## Immutable Slice (&[T])

> ✅ All examples below use borrowed slices and **do not mutate or clone** unless absolutely necessary.

---

### map + filter via Iterators on a Slice

**✅ Problem**: Square numbers and keep those > 10

```rust
fn main() {
    let slice: &[i32] = &[1, 2, 3, 4, 5];
    let result: Vec<_> = slice.iter()
        .map(|x| x * x)
        .filter(|x| *x > 10)
        .collect();

    println!("{:?}", result); // [16, 25]
}
```

---

### Grouping by Key with Slice Input

**✅ Problem**: Group strings by length

```rust
use std::collections::HashMap;

fn group_by_length(words: &[&str]) -> HashMap<usize, Vec<&str>> {
    let mut map = HashMap::new();
    for &word in words {
        map.entry(word.len()).or_default().push(word);
    }
    map
}

fn main() {
    let words = ["a", "to", "tea", "ted", "ten"];
    let grouped = group_by_length(&words);
    println!("{:#?}", grouped);
}
```

---

### Flat Map from Nested Slices

**✅ Problem**: Flatten `&[&[T]]` to `Vec<T>`

```rust
fn main() {
    let nested: &[&[i32]] = &[&[1, 2], &[3, 4]];
    let flat: Vec<i32> = nested.iter()
        .flat_map(|s| s.iter().copied())
        .collect();

    println!("{:?}", flat); // [1, 2, 3, 4]
}
```

---

### Cartesian Product from Slices

```rust
fn main() {
    let a = [1, 2];
    let b = ["x", "y"];
    let product: Vec<_> = a.iter()
        .flat_map(|&x| b.iter().map(move |&y| (x, y)))
        .collect();

    println!("{:?}", product);
    // [(1, "x"), (1, "y"), (2, "x"), (2, "y")]
}
```

---

### Find Duplicates in a Slice

```rust
use std::collections::HashSet;

fn find_duplicates(slice: &[i32]) -> HashSet<i32> {
    let mut seen = HashSet::new();
    let mut dupes = HashSet::new();

    for &x in slice {
        if !seen.insert(x) {
            dupes.insert(x);
        }
    }

    dupes
}

fn main() {
    let nums = [1, 2, 3, 2, 4, 3];
    println!("{:?}", find_duplicates(&nums)); // {2, 3}
}
```

---

### Partition a Slice

```rust
fn main() {
    let nums = [1, 2, 3, 4, 5];
    let (even, odd): (Vec<_>, Vec<_>) = nums.iter()
        .partition(|&&x| x % 2 == 0);

    println!("Even: {:?}, Odd: {:?}", even, odd);
}
```

---

### Sliding Window from Slice

```rust
fn main() {
    let data = [10.0, 20.0, 30.0, 40.0, 50.0];
    let moving_avg: Vec<f64> = data.windows(3)
        .map(|w| w.iter().sum::<f64>() / w.len() as f64)
        .collect();

    println!("{:?}", moving_avg); // [20.0, 30.0, 40.0]
}
```

---

### Transpose a Matrix Using Slice of Slices

```rust
fn transpose(matrix: &[&[i32]]) -> Vec<Vec<i32>> {
    if matrix.is_empty() { return vec![]; }
    let cols = matrix[0].len();
    (0..cols)
        .map(|i| matrix.iter().map(|row| row[i]).collect())
        .collect()
}

fn main() {
    let matrix = [&[1, 2, 3][..], &[4, 5, 6]];
    let result = transpose(&matrix);
    println!("{:?}", result); // [[1, 4], [2, 5], [3, 6]]
}
```

---

### Index-Based Slice Transformation

```rust
fn main() {
    let data = ["a", "b", "c", "d"];
    let transformed: Vec<_> = data.iter()
        .enumerate()
        .map(|(i, val)| if i % 2 == 0 { "x" } else { *val })
        .collect();

    println!("{:?}", transformed); // ["x", "b", "x", "d"]
}
```

---

### All / Any Matching with Slices

```rust
fn main() {
    let nums = [1, 2, 3];
    let all_gt_0 = nums.iter().all(|&x| x > 0);
    let any_even = nums.iter().any(|&x| x % 2 == 0);

    println!("All > 0: {}, Any even: {}", all_gt_0, any_even);
}
```

---

## Summary: Why Use Slices?

* ✅ Zero-copy, efficient read-only access
* ✅ Great for APIs that should not take ownership
* ✅ Easily combined with iterator adapters
* ✅ Safer for embedded or real-time systems

