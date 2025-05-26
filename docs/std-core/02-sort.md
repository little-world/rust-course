

## Sorting

> 📦 All sorting in Rust uses an **adaptive merge sort** (Timsort) via `.sort()` and `.sort_by()`. It’s stable, fast, and built into the standard library.

---

### Default Ascending Sort with .sort()

```rust
fn main() {
    let mut nums = vec![3, 1, 4, 2];
    nums.sort();
    println!("{:?}", nums); // [1, 2, 3, 4]
}
```

📘 Uses `Ord`. Works with numbers, strings, tuples, etc.

---

### Descending Sort with .sortby()

```rust
fn main() {
    let mut nums = vec![3, 1, 4, 2];
    nums.sort_by(|a, b| b.cmp(a));
    println!("{:?}", nums); // [4, 3, 2, 1]
}
```

📘 Reverse the comparison to sort descending.

---

### Stable Sort with .sortbykey()

```rust
#[derive(Debug)]
struct Person { name: String, age: u8 }

fn main() {
    let mut people = vec![
        Person { name: "Alice".into(), age: 30 },
        Person { name: "Bob".into(), age: 30 },
        Person { name: "Charlie".into(), age: 25 },
    ];

    people.sort_by_key(|p| p.age); // Stable: keeps relative name order
    println!("{:?}", people);
}
```

📘 `.sort_by_key()` is **stable** — equal keys keep original order.

---

### Sort by Multiple Fields (Tie-breaker Strategy)

```rust
#[derive(Debug)]
struct User { name: String, age: u8 }

fn main() {
    let mut users = vec![
        User { name: "Bob".into(), age: 30 },
        User { name: "Alice".into(), age: 30 },
        User { name: "Charlie".into(), age: 25 },
    ];

    users.sort_by(|a, b| {
        a.age.cmp(&b.age)
            .then_with(|| a.name.cmp(&b.name))
    });

    println!("{:?}", users);
}
```

📘 Use `Ordering::then_with()` for multi-key sorts.

---

### Custom Comparator Function

```rust
fn main() {
    let mut words = vec!["banana", "apple", "pear"];
    
    // Sort by string length, descending
    words.sort_by(|a, b| b.len().cmp(&a.len()));

    println!("{:?}", words); // ["banana", "apple", "pear"]
}
```

📘 `.sort_by()` gives full control with access to each element.

---

### Case-Insensitive String Sort

```rust
fn main() {
    let mut items = vec!["Banana", "apple", "Pear"];
    items.sort_by_key(|s| s.to_lowercase());
    println!("{:?}", items);
}
```

💡 Use `to_lowercase()` for Unicode-aware case folding.

---

### Sorting Floats with NaN Handling

```rust
fn main() {
    let mut nums = vec![3.0, f64::NAN, 1.0];
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    println!("{:?}", nums); // [1.0, 3.0, NaN]
}
```

📘 Float comparisons require `partial_cmp`.

---

### Reverse Sort with .rev() on Iterator Output

```rust
fn main() {
    let mut nums = vec![1, 2, 3];
    let sorted: Vec<_> = nums.into_iter().rev().collect();
    println!("{:?}", sorted); // [3, 2, 1]
}
```

💡 `.rev()` is useful after `.sort()` or on any iterator.

---

### Sort Strings Numerically Using Natural Ordering

```rust
fn main() {
    let mut items = vec!["item2", "item10", "item1"];
    items.sort_by_key(|s| {
        s.trim_start_matches("item").parse::<u32>().unwrap_or(0)
    });
    println!("{:?}", items); // ["item1", "item2", "item10"]
}
```

📘 Handy for natural "human" sorting when strings encode numbers.

---

### Sort by Enum Discriminant

```rust
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Priority {
    Low,
    Medium,
    High,
}

fn main() {
    let mut priorities = vec![Priority::Medium, Priority::High, Priority::Low];
    priorities.sort();
    println!("{:?}", priorities); // [Low, Medium, High]
}
```

📘 Enums with derived `Ord` sort by variant order.

---

## Summary: Sorting Tools in Rust

| Method             | Description                          |
| ------------------ | ------------------------------------ |
| `.sort()`          | Uses `Ord`, ascending                |
| `.sort_by()`       | Custom comparison logic              |
| `.sort_by_key()`   | Sort by derived key, stable          |
| `Ordering::then()` | Multi-field sort chaining            |
| `rev()`            | Reverses iterators (not sort itself) |

-