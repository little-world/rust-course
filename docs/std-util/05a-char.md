Here’s a **cookbook-style tutorial for `std::char`**, Rust’s standard module for working with **Unicode scalar values** — that is, Rust’s `char` type.

---

## Rust std::char Cookbook

> 📦 Module: [`std::char`](https://doc.rust-lang.org/std/char/)

In Rust:

* A `char` is a **Unicode scalar value**, 4 bytes (not a byte).
* It can represent **any valid Unicode code point**.

---

## Creating and Checking chars

---

### Basic Character Literals

```rust
fn main() {
    let letter: char = 'a';
    let emoji: char = '🦀';
    println!("{} {}", letter, emoji);
}
```

📘 `char` literals are wrapped in single quotes (`'x'`), not double.

---

### Check Properties

```rust
fn main() {
    let c = 'R';
    println!("Is alphabetic? {}", c.is_alphabetic());
    println!("Is numeric? {}", c.is_numeric());
    println!("Is lowercase? {}", c.is_lowercase());
}
```

---

### Character Classification

| Method             | Checks for…               |
| ------------------ | ------------------------- |
| `.is_alphabetic()` | A–Z, a–z, Unicode letters |
| `.is_numeric()`    | 0–9, Unicode digits       |
| `.is_ascii()`      | 0x00–0x7F                 |
| `.is_control()`    | Non-printing characters   |
| `.is_whitespace()` | Space, tab, etc.          |

---

## Transforming chars

---

### Convert to Upper/Lower Case

```rust
fn main() {
    let ch = 'ß';
    for up in ch.to_uppercase() {
        print!("{}", up); // SS
    }
}
```

📘 These methods return an **iterator** because a char can map to **multiple chars**.

---

### Escape Characters

```rust
fn main() {
    let c = '\n';
    println!("Escaped: {:?}", c.escape_default().to_string()); // "\\n"
}
```

---

### Convert char u

```rust
fn main() {
    let c = '🦀';
    let code = c as u32;
    println!("Code point: U+{:X}", code); // U+1F980

    if let Some(decoded) = std::char::from_u32(code) {
        println!("Char: {}", decoded); // 🦀
    }
}
```

📘 `from_u32()` checks for validity — not all `u32` values are valid `char`s.

---

## Iteration & Conversion

---

### Iterate Over Characters in a String

```rust
fn main() {
    let word = "hello🦀";
    for c in word.chars() {
        println!("{}", c);
    }
}
```

📘 `chars()` gives each Unicode character (not byte!).

---

### Create String from a char

```rust
fn main() {
    let c = '💡';
    let s = c.to_string();
    println!("{}", s); // 💡
}
```

---

### Parse char from String

```rust
fn main() {
    let input = "a";
    let ch: char = input.chars().next().unwrap();
    println!("{}", ch);
}
```

📘 Use this for parsing individual characters safely.

---

## Summary Table

| Operation                   | Method                             |
| --------------------------- | ---------------------------------- |
| Check alphabetic, numeric   | `is_alphabetic()`, `is_numeric()`  |
| Convert case                | `to_uppercase()`, `to_lowercase()` |
| Convert to `u32`            | `as u32`                           |
| Convert from `u32`          | `char::from_u32(code)`             |
| Escape to printable         | `escape_default()`                 |
| Convert to string           | `to_string()`                      |
| Iterate string by character | `"abc".chars()`                    |

---

## When to Use char

* You need to inspect or manipulate **Unicode characters**
* You’re writing **parsers, lexers**, or **text-processing tools**
* You’re working with **non-ASCII input or emoji**
* You need to **validate** or **sanitize** text

