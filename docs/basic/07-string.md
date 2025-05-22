
Here's a beginner-friendly **Rust Tutorial on `String` and `&str`**, covering:

* The difference between `String` and `&str`
* Creating and converting between them
* Common operations
* Ownership and borrowing with strings

---

## What's the Difference?

| Type     | Meaning                             | Heap? | Mutable? |
| -------- | ----------------------------------- | ----- | -------- |
| `String` | Growable, heap-allocated            | ✅ Yes | ✅ Yes    |
| `&str`   | String **slice**, usually immutable | ❌ No  | ❌ No     |

* `String` owns its data (like `Vec<u8>`)
* `&str` is a **reference** to a string (often a slice of a `String` or a string literal)

### Example:

```rust
let s1: String = String::from("hello");
let s2: &str = "world"; // string literal (&str)
```

---

## Creating a String

```rust
let s = String::new();
let s = String::from("hello");
let s = "hello".to_string();
```

---

## Converting Between String and &str

### String &str:

```rust
let s = String::from("hello");
let slice: &str = &s;         // borrow
```

### &str String:

```rust
let s: &str = "world";
let string: String = s.to_string();
```

---

## Common String Methods

### Concatenation:

```rust
let a = String::from("Hello, ");
let b = String::from("world!");
let c = a + &b; // `a` is moved, `b` is borrowed
```

### Push characters/strings:

```rust
let mut s = String::from("Hi");
s.push('!');
s.push_str(" Rustaceans");
```

### Length and Iteration:

```rust
let s = "Hello";
println!("Length: {}", s.len()); // bytes, not chars
println!("{}", s.chars().count()); // char count (correct for Unicode)

```

### Substrings with slices:

```rust
let s = "hello";
let slice = &s[0..2]; // "he" (valid UTF-8 boundary!)
```
---

## Why Not Index Strings with s[]?

Because Rust strings are UTF-8 encoded. Indexing by byte can break multibyte characters like emojis or foreign characters.

Use `.chars().nth(n)`:

```rust
let ch = "👋🌍".chars().nth(1);
println!("{:?}", ch); // Some('🌍')
```

---

## Comparing Strings

```rust
let a = String::from("hi");
let b = "hi"; // &str
assert_eq!(a, b); // comparison works!
```

---

## String Formatting

```rust
let name = "Alice";
let msg = format!("Hello, {}!", name);
```

---


Here’s a list of **useful Rust string functions** and methods with quick examples — covering both `String` and `&str`. These are handy for **manipulation, inspection, and conversion**.

---

##  Case Conversion

```rust
let name = "Rust";
println!("{}", name.to_uppercase()); // RUST
println!("{}", name.to_lowercase()); // rust
```

---

## Search and Replace

```rust
let text = "hello world";
text.contains("world");           // true
text.starts_with("hello");        // true
text.ends_with("ld");             // true
text.find("o");                   // Some(4)

let replaced = text.replace("world", "Rust"); // "hello Rust"
```

---

## Trim and Clean

```rust
let messy = "  trimmed  \n";
println!("'{}'", messy.trim());      // "trimmed"
println!("'{}'", messy.trim_start());
println!("'{}'", messy.trim_end());
```

---

## Split and Join

```rust
let data = "a,b,c";
for item in data.split(',') {
    println!("{}", item);
}

let words = vec!["Hello", "Rust"];
let joined = words.join(" "); // "Hello Rust"
```

---

##  Iteration

```rust
let s = "Hi 😊";
for c in s.chars() {
    println!("{}", c); // Unicode-safe
}

for b in s.bytes() {
    println!("{}", b); // raw bytes
}
```

---

## Repeat and Padding

```rust
let s = "ha";
println!("{}", s.repeat(3)); // "hahaha"

let padded = format!("{:>8}", "hi"); // right-align: "      hi"
```

---

##  Parsing and Conversion

```rust
let n: i32 = "42".parse().unwrap();
let s = n.to_string();
```
---

## Escape Characters

```rust
let escaped = "Line 1\nLine 2\tTabbed";
println!("{}", escaped);
```
---

## Summary Table

| Operation                 | Example                   |
| ------------------------- | ------------------------- |
| Create `String`           | `String::from("hi")`      |
| Convert `&str` → `String` | `"hello".to_string()`     |
| Convert `String` → `&str` | `&my_string`              |
| Add strings               | `s1 + &s2`                |
| Append to `String`        | `push`, `push_str`        |
| Get length in bytes       | `s.len()`                 |
| Iterate characters        | `s.chars()`               |
| Format new string         | `format!("Hi, {}", name)` |
| Check if substring exists     | `contains("...")`   |
| Replace substring             | `replace("a", "b")` |
| Split into iterator           | `split(',')`        |
| Join items into string        | `join(" ")`         |
| Remove whitespace             | `trim()`            |
| Uppercase version             | `to_uppercase()`    |
| Convert to number/type        | `parse::<T>()`      |
| Convert anything into a string| `to_string()`       |


---

## Tips

* Prefer `&str` for read-only strings.
* Use `String` when you need to **own** or **mutate**.
* Avoid slicing unless you're sure of UTF-8 boundaries.