Here's a **cookbook-style tutorial** for the **Rust standard macros**, designed with practical, real-world examples. Each "recipe" includes a **problem**, the **macro to use**, a **code snippet**, and a brief **explanation**.

---

## Rust Standard Macros Cookbook

### println! Print to Console

**Problem**: Print a message or variable to the console.

```rust
fn main() {
    let name = "Alice";
    println!("Hello, {}!", name);
}
```

**Explanation**: `println!` writes to standard output with formatting.

---

### format! Format Strings Without Printing

**Problem**: Create a string from formatted content.

```rust
fn main() {
    let name = "Alice";
    let greeting = format!("Hello, {}!", name);
    println!("{}", greeting);
}
```

**Explanation**: Like `println!`, but returns the result as a `String`.

---

### dbg! Quick and Dirty Debugging

**Problem**: Debug variables during development.

```rust
fn main() {
    let a = 2;
    let b = dbg!(a * 3); // prints the value with file and line number
}
```

**Explanation**: Use `dbg!` to quickly inspect values. It prints to stderr.

---

### vec! Create Vectors Easily

**Problem**: Initialize a vector with values.

```rust
fn main() {
    let numbers = vec![1, 2, 3];
    println!("{:?}", numbers);
}
```

**Explanation**: `vec!` constructs a `Vec<T>` easily.

---

### macrorules! Write Custom Macros

**Problem**: Define your own macro.

```rust
macro_rules! say_hello {
    () => {
        println!("Hello from macro!");
    };
}

fn main() {
    say_hello!();
}
```

**Explanation**: `macro_rules!` defines declarative macros.

---

### assert!, asserteq!, assertne! Runtime Assertions

**Problem**: Ensure certain conditions hold at runtime.

```rust
fn main() {
    let x = 5;
    assert!(x > 0);
    assert_eq!(x, 5);
    assert_ne!(x, 0);
}
```

**Explanation**: These macros help with testing and enforcing assumptions.

---

### includestr! / includebytes! Embed Files at Compile Time

**Problem**: Include static files in the binary.

```rust
fn main() {
    let content = include_str!("README.md");
    println!("{}", content);
}
```

**Explanation**: Embed files directly into the executable.

---

### concat! Concatenate Literals

**Problem**: Merge string literals at compile time.

```rust
fn main() {
    let s = concat!("foo", "bar");
    println!("{}", s); // "foobar"
}
```

---

### env! / optionenv! Access Environment Variables at Compile Time

```rust
fn main() {
    let rust_version = env!("CARGO_PKG_VERSION");
    println!("Running version: {}", rust_version);
}
```

**Explanation**: Use these to access build-time environment variables.

---

### matches! Pattern Matching Simplified

**Problem**: Check if a value matches a pattern.

```rust
fn main() {
    let x = Some(3);
    if matches!(x, Some(_)) {
        println!("Got something!");
    }
}
```

---

### todo! / unimplemented! / unreachable! Mark Incomplete or Invalid Code

```rust
fn future_feature() {
    todo!("Implement this later");
}

fn main() {
    // future_feature(); // panics at runtime
}
```