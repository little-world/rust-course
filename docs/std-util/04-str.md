Here’s a **cookbook-style tutorial for `std::str`**, the Rust standard module for working with **string slices (`&str`)** — one of the most essential types in the language. It covers searching, splitting, converting, and parsing, all without allocating new memory.

---

## Rust std::str Cookbook

> 📦 Module: [`std::str`](https://doc.rust-lang.org/std/str/)

* A `&str` is a borrowed, immutable UTF-8 string slice.
* Many methods come from the [`str`](https://doc.rust-lang.org/std/primitive.str.html) primitive type itself.

---

## Basics

---

### Create a String Slice

```rust
fn main() {
    let s: &str = "Hello, world!";
    println!("{}", s);
}
```

📘 `&str` is typically a slice of a `String` or a string literal.

---

### Get Length and Characters

```rust
fn main() {
    let s = "Rust 🦀";
    println!("Length: {}", s.len()); // Bytes
    println!("Characters: {}", s.chars().count()); // Unicode
}
```

📘 `len()` returns **bytes**, not characters.

---

## Searching & Matching

---

### Check Prefix / Suffix

```rust
fn main() {
    let s = "rustacean";
    println!("{}", s.starts_with("rust"));  // true
    println!("{}", s.ends_with("cean"));    // true
}
```

---

### Find a Substring

```rust
fn main() {
    let s = "find the crab";
    if let Some(i) = s.find("crab") {
        println!("Found at index: {}", i);
    }
}
```

📘 Returns `Option<usize>` with byte offset.

---

### Check Contains

```rust
fn main() {
    let s = "safe and fast";
    println!("{}", s.contains("fast")); // true
}
```

---

## Splitting and Joining

---

### Split by a Character

```rust
fn main() {
    let csv = "a,b,c";
    for part in csv.split(',') {
        println!("{}", part);
    }
}
```

---

### Split by Whitespace

```rust
fn main() {
    let text = "split by space";
    let words: Vec<_> = text.split_whitespace().collect();
    println!("{:?}", words);
}
```

---

### Split Once

```rust
fn main() {
    let s = "key=value";
    if let Some((key, value)) = s.split_once('=') {
        println!("{} => {}", key, value);
    }
}
```

📘 Useful for config parsers and key-value formats.

---

### Join Strings

```rust
fn main() {
    let parts = ["one", "two", "three"];
    let joined = parts.join("-");
    println!("{}", joined); // one-two-three
}
```

📘 `join()` is available on slices of `&str`.

---

## Trimming and Changing Case

---

### Trim Whitespace

```rust
fn main() {
    let dirty = "  trimmed  ";
    println!("[{}]", dirty.trim()); // [trimmed]
}
```

---

### To Upper or Lower Case

```rust
fn main() {
    let shout = "hello";
    println!("{}", shout.to_uppercase()); // HELLO
}
```

---

## Iterating and Parsing

---

### Iterate Over Characters

```rust
fn main() {
    let s = "abc";
    for c in s.chars() {
        println!("{}", c);
    }
}
```

---

### Convert to String or Slice

```rust
fn main() {
    let owned = "hello".to_string(); // &str → String
    let slice: &str = &owned;        // String → &str
}
```

---

### Parse into Number

```rust
fn main() {
    let s = "42";
    let n: i32 = s.parse().expect("not a number");
    println!("{}", n + 1);
}
```

📘 Works when the target type implements `FromStr`.

---

## Safety & Encoding

---

### Safe Substrings with .get()

```rust
fn main() {
    let s = "emoji 🦀";
    if let Some(sub) = s.get(0..6) {
        println!("{}", sub); // emoji 
    }
}
```

📘 Prevents invalid UTF-8 slicing.

---

### Convert Bytes to \&str

```rust
fn main() {
    let bytes = b"rust";
    let s = std::str::from_utf8(bytes).expect("invalid utf-8");
    println!("{}", s);
}
```

---

## Summary Table

| Method            | Purpose                       |
| ----------------- | ----------------------------- |
| `.len()`          | Byte length                   |
| `.chars()`        | Iterator over characters      |
| `.find()`         | Substring position            |
| `.contains()`     | Check for substring           |
| `.split()`        | Split by pattern              |
| `.split_once()`   | Split into two parts          |
| `.trim()`         | Remove surrounding whitespace |
| `.parse()`        | Convert to a number or type   |
| `.to_uppercase()` | Return uppercase string       |
| `from_utf8()`     | Bytes → \&str                 |

