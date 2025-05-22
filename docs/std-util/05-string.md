Here’s a **cookbook-style tutorial for `std::string`**, Rust’s standard module that defines the **owned string type**: [`String`](https://doc.rust-lang.org/std/string/struct.String.html). Unlike `&str`, which is a borrowed view into a string, `String` is **heap-allocated**, **mutable**, and **growable**.

---

## Rust std::string Cookbook

> 📦 Module: [`std::string`](https://doc.rust-lang.org/std/string/)

```rust
pub struct String { /* heap-allocated UTF-8 text */ }
```

---

## Creating and Converting

---

### Create a String

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = "world".to_string();
    println!("{} {}", s1, s2);
}
```

📘 `String::from()` and `.to_string()` are equivalent.

---

### Convert String &str

```rust
fn main() {
    let s = String::from("Rust");
    let slice: &str = &s; // String → &str
    let again = slice.to_string(); // &str → String
}
```

---

### From Characters or Bytes

```rust
fn main() {
    let from_chars: String = ['H', 'i'].iter().collect();
    let from_utf8 = String::from_utf8(vec![82, 117, 115, 116]).unwrap();
    println!("{}, {}", from_chars, from_utf8);
}
```

---

## Modifying Strings

---

### Push Characters or Strings

```rust
fn main() {
    let mut s = String::from("hello");
    s.push(' ');
    s.push_str("world");
    println!("{}", s); // hello world
}
```

---

### Remove Characters by Index

```rust
fn main() {
    let mut s = String::from("rust!");
    s.pop(); // removes last char
    println!("{}", s); // rust
}
```

📘 Use `.remove(index)` to remove by position.

---

### Replace or Truncate

```rust
fn main() {
    let mut s = String::from("goodbye");
    s.truncate(4);
    println!("{}", s); // good

    let replaced = "I like cats".replace("cats", "dogs");
    println!("{}", replaced); // I like dogs
}
```

---

### Insert and Replace Ranges

```rust
fn main() {
    let mut s = String::from("ace!");
    s.insert(0, 'F');
    s.replace_range(0..1, "f");
    println!("{}", s); // face!
}
```

---

## Querying and Analyzing

---

### Check Contents or Length

```rust
fn main() {
    let s = String::from("Rustacean");
    println!("{}", s.contains("Rust"));  // true
    println!("{}", s.len());             // bytes
    println!("{}", s.chars().count());   // characters
}
```

---

### Split and Iterate

```rust
fn main() {
    let s = String::from("a,b,c");
    for part in s.split(',') {
        println!("{}", part);
    }
}
```

---

### Trim Whitespace

```rust
fn main() {
    let raw = String::from("  trimmed  ");
    println!("[{}]", raw.trim()); // [trimmed]
}
```

---

## Transforming

---

### Change Case

```rust
fn main() {
    let s = String::from("Rust");
    println!("{}", s.to_uppercase()); // RUST
}
```

---

### Convert to Integer or Other Type

```rust
fn main() {
    let num = String::from("42");
    let parsed: i32 = num.parse().unwrap();
    println!("{}", parsed + 1);
}
```

---

## Ownership & Safety

---

### Move vs Clone

```rust
fn main() {
    let s1 = String::from("data");
    let s2 = s1.clone(); // deep copy
    println!("{}", s2);
    // println!("{}", s1); // would error if not cloned
}
```

---

### Access by Slicing Safely

```rust
fn main() {
    let s = String::from("hello");
    let slice = &s[0..2];
    println!("{}", slice);
}
```

📘 Make sure the range matches UTF-8 boundaries.

---

## Common Constructors

---

| Use Case               | Example                           |
| ---------------------- | --------------------------------- |
| Empty string           | `String::new()`                   |
| From `&str`            | `String::from("text")`            |
| From literal           | `"text".to_string()`              |
| Repeating characters   | `"a".repeat(3)` → `"aaa"`         |
| From iterator of chars | `['a', 'b'].iter().collect()`     |
| From `Vec<u8>`         | `String::from_utf8(vec![82,117])` |

---

## Summary Table

| Method                | Purpose                              |
| --------------------- | ------------------------------------ |
| `push`, `push_str`    | Append char or string                |
| `truncate`, `replace` | Cut or modify parts                  |
| `insert`, `remove`    | Change characters by index           |
| `split`, `trim`       | Break or clean text                  |
| `parse::<T>()`        | Convert string to number/other types |
| `to_lowercase()`      | Unicode-aware lowercase              |
| `from_utf8()`         | Convert `Vec<u8>` to `String` safely |



## String vs &str vs Cow<str>

| Feature         | `String`                         | `&str`                     | `Cow<'a, str>`                        |
| --------------- | -------------------------------- | -------------------------- | ------------------------------------- |
| Owned?          | ✅ Yes                            | ❌ No                       | ✅/❌ Owned or borrowed (dual mode)     |
| Heap Allocation | ✅ Always                         | ❌ Never                    | ❌ When borrowed, ✅ when cloned        |
| Mutability      | ✅ Mutable                        | ❌ Immutable                | ❌ Immutable unless owned              |
| Sized?          | ✅ Yes                            | ❌ Often unsized (`?Sized`) | ✅ Yes (enum)                          |
| Lifetime        | `'static` or bound to owner      | ✅ Required                 | ✅ Required                            |
| Zero-cost?      | ❌ No (always allocates)          | ✅ Yes                      | ✅ Sometimes (borrowed)                |
| Common Source   | `.to_string()`, `String::from()` | `"literal"` or `&s`        | `Cow::Borrowed(s)` or `Cow::Owned(s)` |

---

## Examples and Use Cases

---

### String: Owned, Growable Heap Text

```rust
fn main() {
    let mut s = String::from("hello");
    s.push_str(" world");
    println!("{}", s); // hello world
}
```

✅ Use when:

* You need to **modify** or **store** text
* You want **heap allocation**
* You need ownership to **return or move** the value

---

### &str: Borrowed View into UTF- Text

```rust
fn main() {
    let s = "static string";       // &'static str
    let owned = String::from("abc");
    let borrowed: &str = &owned;   // &str from String
    println!("{}", borrowed);
}
```

✅ Use when:

* You want to **avoid allocation**
* You're **passing or reading** a string, not modifying
* You're working with **string literals** or slices

---

### Cow<str>: Clone-On-Write Flexibility

```rust
use std::borrow::Cow;

fn normalize(input: &str) -> Cow<str> {
    if input.contains('_') {
        Cow::Owned(input.replace("_", "-"))
    } else {
        Cow::Borrowed(input)
    }
}

fn main() {
    let a = normalize("ready_go");
    let b = normalize("ready-go");

    println!("{}", a); // owned
    println!("{}", b); // borrowed
}
```

✅ Use when:

* You want to **avoid unnecessary allocations**
* You sometimes modify a string, sometimes not
* You're writing **generic APIs** that take `Into<Cow<str>>`

---

## Performance Comparison

| Scenario                          | Best Type      |
| --------------------------------- | -------------- |
| Literal or borrowed text          | `&str`         |
| Modifiable string buffer          | `String`       |
| Return a value that might change  | `Cow<str>`     |
| Avoiding clone unless needed      | `Cow<str>`     |
| Fixed-length, compile-time string | `&'static str` |

---

## Interconversion

```rust
let s: String = "hi".to_string();         // &str → String
let r: &str = &s;                         // String → &str
let c: Cow<str> = Cow::from("hi");        // &str → Cow::Borrowed
let c2: Cow<str> = Cow::from(s.clone());  // String → Cow::Owned
```

---

## Summary

| You want to...                            | Use            |
| ----------------------------------------- | -------------- |
| Store and grow a string                   | `String`       |
| Use or pass around read-only string       | `&str`         |
| Write APIs that flexibly handle both      | `Cow<str>`     |
| Avoid allocation if no mutation is needed | `Cow<str>`     |
| Work with static strings                  | `&'static str` |

