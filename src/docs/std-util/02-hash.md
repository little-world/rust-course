Here’s a **cookbook-style tutorial for `std::hash`**, the Rust standard module used to **hash values for collections**, comparisons, and more. It enables types to be used in **`HashMap`**, **`HashSet`**, and custom hashing scenarios.

---

## Rust std::hash Cookbook

> 📦 Module: [`std::hash`](https://doc.rust-lang.org/std/hash/)

Main components:

* `Hash` trait: for types that can be hashed
* `Hasher` trait: defines the hash function logic
* `BuildHasher`: used by hash-based collections to construct hashers

---

## Core Recipes

---

### Hash a Basic Type

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn main() {
    let mut hasher = DefaultHasher::new();
    "hello".hash(&mut hasher);
    let hash = hasher.finish();

    println!("Hash: {}", hash);
}
```

📘 `DefaultHasher` is used by `HashMap` and `HashSet`.

---

### Implement Hash for a Custom Struct

```rust
use std::hash::{Hash, Hasher};

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}
```

📘 Often unnecessary if you use `#[derive(Hash)]`.

---

### Use HashMap with Custom Key Type

```rust
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Debug)]
struct Point(i32, i32);

fn main() {
    let mut map = HashMap::new();
    map.insert(Point(1, 2), "Origin");

    println!("{:?}", map.get(&Point(1, 2)));
}
```

📘 Custom keys must implement `Hash`, `Eq`, and `PartialEq`.

---

### Derive Hash, Eq, and PartialEq

```rust
#[derive(Hash, Eq, PartialEq, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}
```

📘 Required if the type is used in a hash-based container.

---

### Use HashSet with Custom Types

```rust
use std::collections::HashSet;

#[derive(Hash, Eq, PartialEq, Debug)]
struct Item(u32);

fn main() {
    let mut set = HashSet::new();
    set.insert(Item(1));
    set.insert(Item(2));
    println!("{:?}", set.contains(&Item(1))); // true
}
```

---

## Hashing Internals and Advanced Use

---

### Compare Hash Values for Debugging

```rust
fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

fn main() {
    let h1 = calculate_hash(&"rust");
    let h2 = calculate_hash(&"Rust");
    println!("'rust': {}, 'Rust': {}", h1, h2);
}
```

📘 Useful for debugging collisions or testing hash functions.

---

### Use a Custom Hasher with HashMap

```rust
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct AlwaysZero;

impl Hasher for AlwaysZero {
    fn finish(&self) -> u64 { 0 }
    fn write(&mut self, _: &[u8]) {}
}

type ZeroHasherMap<K, V> = HashMap<K, V, BuildHasherDefault<AlwaysZero>>;

fn main() {
    let mut map: ZeroHasherMap<&str, i32> = HashMap::default();
    map.insert("a", 1);
    map.insert("b", 2);
    println!("{:?}", map);
}
```

📘 For education or controlled testing (not production!).

---

### Hash Multiple Values Together

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_combined<A: Hash, B: Hash>(a: &A, b: &B) -> u64 {
    let mut hasher = DefaultHasher::new();
    a.hash(&mut hasher);
    b.hash(&mut hasher);
    hasher.finish()
}

fn main() {
    let h = hash_combined(&"apple", &42);
    println!("Combined hash: {}", h);
}
```

---

### Custom Hash Key Wrapper

```rust
use std::hash::{Hash, Hasher};
use std::collections::HashSet;

#[derive(Debug)]
struct CaseInsensitive(String);

impl PartialEq for CaseInsensitive {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}
impl Eq for CaseInsensitive {}

impl Hash for CaseInsensitive {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

fn main() {
    let mut set = HashSet::new();
    set.insert(CaseInsensitive("HELLO".into()));
    println!("{}", set.contains(&CaseInsensitive("hello".into()))); // true
}
```

📘 Handy for case-insensitive keys, deduplication, etc.

---

## Summary Table

| Trait / Struct        | Purpose                                   |
| --------------------- | ----------------------------------------- |
| `Hash`                | Marks a type as hashable                  |
| `Hasher`              | Defines how to compute a hash             |
| `BuildHasher`         | Creates hashers for use in collections    |
| `DefaultHasher`       | Standard hasher (SipHash) used by HashMap |
| `HashMap` / `HashSet` | Uses hash to store and lookup keys        |

---

## Use Cases

* Efficient key/value storage (`HashMap`, `HashSet`)
* Custom equality and hashing logic
* Testing and benchmarking hash collisions
* Domain-specific key wrappers (e.g., case-insensitive)