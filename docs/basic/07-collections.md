

## Vectors 

### Creating a Vector

```rust
fn main() {
    let mut numbers = Vec::new(); // empty vector of type Vec<i32>
    numbers.push(1);
    numbers.push(2);
    numbers.push(3);

    println!("{:?}", numbers);
}
```

### With Initial Values

```rust
let names = vec!["Alice", "Bob", "Carol"];
```

### Accessing Elements

```rust
let first = numbers[0];       // panics if out of bounds
let maybe = numbers.get(2);   // returns Option<&T>
```

### Iterating

```rust
for n in &numbers {
    println!("{}", n);
}
```


## String

### Creating Strings

```rust
let mut s = String::from("Hello");
s.push_str(", world!");
```

### Concatenation

```rust
let s1 = String::from("Hello");
let s2 = String::from("Rust");
let combined = s1 + " " + &s2; // s1 is moved
```

### String Interpolation

```rust
let name = "Alice";
let greeting = format!("Hello, {}!", name);
```

### Iterating Over Characters

```rust
for c in "hi 😊".chars() {
    println!("{}", c);
}
```

## HashMap 

Requires `use std::collections::HashMap;`

### Creating and Inserting

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    scores.insert("Alice", 10);
    scores.insert("Bob", 20);
}
```

### Accessing Values

```rust
if let Some(score) = scores.get("Alice") {
    println!("Score: {}", score);
}
```

### Iterating

```rust
for (key, value) in &scores {
    println!("{}: {}", key, value);
}
```

### Updating or Inserting If Absent

```rust
scores.entry("Charlie").or_insert(0); // insert 0 if not exists
```

---

## HashSet 

### Create a HashSet

```rust
use std::collections::HashSet;

fn main() {
    let mut fruits = HashSet::new();
    fruits.insert("apple");
    fruits.insert("banana");
    fruits.insert("apple"); // duplicate, ignored

    println!("{:?}", fruits); // only unique values
}
```

### Useful Methods

```rust
fruits.contains("banana"); // true
fruits.remove("apple");
```

---

## Summary

| Collection      | Use Case                    |
| --------------- | --------------------------- |
| `Vec<T>`        | Ordered list, dynamic array |
| `String`        | Text data                   |
| `HashMap<K, V>` | Key-value mapping           |
| `HashSet<T>`    | Unique unordered elements   |

---

### Code Snippet

```rust
use std::collections::{HashMap, HashSet};

fn main() {
    let mut vec = vec![1, 2, 3];
    let mut map = HashMap::new();
    let mut set = HashSet::new();

    vec.push(4);
    map.insert("a", 1);
    set.insert("unique");

    println!("{:?}, {:?}, {:?}", vec, map, set);
}
```