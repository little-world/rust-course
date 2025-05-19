Here’s a **cookbook-style tutorial for `std::collections`**, Rust’s powerful standard module for data structures such as `VecDeque`, `HashMap`, `HashSet`, `BTreeMap`, and more.

---

## Rust std::collections Cookbook

> 📦 Module: [`std::collections`](https://doc.rust-lang.org/std/collections/)

---

## Overview of Key Collections

| Collection      | Ordered? | Unique? | Backed By     | Best For                          |
| --------------- | -------- | ------- | ------------- | --------------------------------- |
| `VecDeque<T>`   | ✅        | ❌       | Ring buffer   | Fast front & back insertion       |
| `HashMap<K, V>` | ❌        | ✅ keys  | Hash table    | Fast key lookup (unordered)       |
| `BTreeMap<K,V>` | ✅        | ✅ keys  | Binary search | Sorted keys, range queries        |
| `HashSet<T>`    | ❌        | ✅       | Hash table    | Unique values, fast membership    |
| `BTreeSet<T>`   | ✅        | ✅       | BTree         | Unique sorted values              |
| `BinaryHeap<T>` | ❌        | ❌       | Binary heap   | Priority queues (max-heap)        |
| `LinkedList<T>` | ✅        | ❌       | Doubly linked | Rarely needed; mostly educational |

---

## Recipes by Collection Type

---

### VecDeque Double-Ended Queue

```rust
use std::collections::VecDeque;

fn main() {
    let mut q = VecDeque::new();
    q.push_back(1);
    q.push_front(0);
    println!("{:?}", q); // [0, 1]
    q.pop_back();
    q.pop_front();
}
```

📘 Fast insert/remove at both ends.

---

### HashMap Key/Value Store

```rust
use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    scores.insert("Alice", 90);
    scores.insert("Bob", 85);

    if let Some(score) = scores.get("Alice") {
        println!("Alice: {}", score);
    }

    for (name, score) in &scores {
        println!("{name} scored {score}");
    }
}
```

📘 Use `.entry()` for conditional insert/update.

---

### BTreeMap Sorted Key/Value

```rust
use std::collections::BTreeMap;

fn main() {
    let mut map = BTreeMap::new();
    map.insert("c", 3);
    map.insert("a", 1);
    map.insert("b", 2);

    for (k, v) in &map {
        println!("{k}: {v}"); // Sorted by key
    }
}
```

📘 Maintains order for in-order iteration or range queries.

---

### HashSet Unordered Set of Unique Items

```rust
use std::collections::HashSet;

fn main() {
    let mut fruits = HashSet::new();
    fruits.insert("apple");
    fruits.insert("banana");
    fruits.insert("apple"); // ignored
    println!("{:?}", fruits.contains("banana")); // true
}
```

📘 Great for fast membership checks.

---

### BTreeSet Sorted Set

```rust
use std::collections::BTreeSet;

fn main() {
    let mut numbers = BTreeSet::new();
    numbers.insert(3);
    numbers.insert(1);
    numbers.insert(2);
    for n in &numbers {
        println!("{}", n); // Sorted: 1, 2, 3
    }
}
```

📘 Perfect for unique, ordered data.

---

### BinaryHeap Priority Queue (Max-Heap)

```rust
use std::collections::BinaryHeap;

fn main() {
    let mut heap = BinaryHeap::new();
    heap.push(3);
    heap.push(5);
    heap.push(1);

    while let Some(top) = heap.pop() {
        println!("{}", top); // 5, 3, 1
    }
}
```

📘 Always returns largest item first.

---

### LinkedList Doubly-Linked List (Rarely Needed)

```rust
use std::collections::LinkedList;

fn main() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_front(0);
    println!("{:?}", list);
}
```

📘 Use only if you really need fast insertion/deletion in the middle.

---

## Common Patterns

---

### Conditionally Insert with .entry()

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.entry("a").or_insert(1);
    map.entry("a").or_insert(2); // does nothing
    println!("{:?}", map); // {"a": 1}
}
```

---

### Count Word Occurrences

```rust
use std::collections::HashMap;

fn main() {
    let text = "a b a b b";
    let mut count = HashMap::new();

    for word in text.split_whitespace() {
        *count.entry(word).or_insert(0) += 1;
    }

    println!("{:?}", count); // {"a": 2, "b": 3}
}
```

---

### Set Operations

```rust
use std::collections::HashSet;

fn main() {
    let a: HashSet<_> = [1, 2, 3].iter().cloned().collect();
    let b: HashSet<_> = [3, 4, 5].iter().cloned().collect();

    println!("Union: {:?}", a.union(&b).collect::<Vec<_>>());
    println!("Intersection: {:?}", a.intersection(&b).collect::<Vec<_>>());
}
```

---

## Summary Table

| Collection   | Fastest For               | Ordered? | Key Features             |
| ------------ | ------------------------- | -------- | ------------------------ |
| `VecDeque`   | Push/pop front/back       | ✅        | Ring buffer              |
| `HashMap`    | Key lookup                | ❌        | Fast, no order           |
| `BTreeMap`   | Sorted map access         | ✅        | Range queries            |
| `HashSet`    | Membership test           | ❌        | Unique unordered values  |
| `BTreeSet`   | Unique + sorted items     | ✅        | Range-enabled sets       |
| `BinaryHeap` | Priority queue (max heap) | ❌        | Efficient max pop        |
| `LinkedList` | Doubly linked list        | ✅        | Rarely better than `Vec` |

---



## Ranges and Arrays

| Feature | Module / Type             | Description                            |
| ------- | ------------------------- | -------------------------------------- |
| `Array` | `[T; N]` in core language | Fixed-size, stack-allocated collection |
| `Range` | \[`std::ops::Range`]      | Iterator-like syntax: `start..end`     |

These are both **primitives** and don't live under `std::collections`, but they work seamlessly with iterator-based APIs.

---

### Arrays ([T; N])

#### Basic Array Creation

```rust
fn main() {
    let a = [1, 2, 3];
    let b = [0; 5]; // five zeros
    println!("{:?}", a);
}
```

#### Indexing and Slicing

```rust
fn main() {
    let arr = [10, 20, 30, 40];
    println!("{}", arr[2]);      // 30
    println!("{:?}", &arr[1..3]); // [20, 30]
}
```

#### Iterating

```rust
fn main() {
    let arr = ["a", "b", "c"];
    for item in arr.iter() {
        println!("{}", item);
    }
}
```

---

### Ranges (start..end, start..=end)

#### Simple Range Iteration

```rust
fn main() {
    for i in 1..5 {
        println!("{}", i); // 1 2 3 4
    }
}
```

#### Inclusive Range

```rust
fn main() {
    for i in 1..=5 {
        println!("{}", i); // 1 2 3 4 5
    }
}
```

#### Range as an Iterator

```rust
fn main() {
    let range = 1..4;
    let sum: i32 = range.sum();
    println!("Sum: {}", sum); // 6
}
```

#### Range with .stepby()

```rust
fn main() {
    for i in (0..10).step_by(2) {
        println!("{}", i); // 0 2 4 6 8
    }
}
```

---

## Summary

| Type     | Description                     | Module / Origin         |
| -------- | ------------------------------- | ----------------------- |
| `[T; N]` | Fixed-size array                | Built-in type           |
| `Range`  | Iterator from `a..b` or `a..=b` | `std::ops::Range`       |
| `Vec<T>` | Growable heap collection        | `std::collections::Vec` |

---

### Arrays:

* Fast, stack-allocated
* Use when size is fixed at compile time
* Convert to slice: `&array`

### Ranges:

* Lazy iterator of numbers
* Use in loops, iterators, slicing
