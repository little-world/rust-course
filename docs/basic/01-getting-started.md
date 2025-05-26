
## Getting Started
Use **rustup**, the official installer:
Open a terminal and run:

```bash
curl https://sh.rustup.rs -sSf | sh
```

Follow the prompts. This installs:

* `rustc` (Rust compiler)
* `cargo` (build and package manager)
* `rustup` (version manager)
 
Verify installation:

```bash
rustc --version
cargo --version
```

---

### First Project 

```bash
cargo new hello_rust
cd hello_rust
```

This creates:

```
hello_rust/
├── Cargo.toml      # metadata and dependencies
└── src/
    └── main.rs     # your Rust code
```

---

## First Program

In `src/main.rs`:

```rust
fn main() {
    println!("Hello, Rust!");
}
```

### Run the program:

```bash
cargo run
```

You’ll see:

```
Hello, Rust!
```

---

## Basic Elements

### Variables

```rust
let x = 5;         // immutable
let mut y = 10;    // mutable
y += 1;
```

### Data Types

```rust
let a: i32 = 10;
let b: f64 = 3.14;
let name: &str = "Rust";
let flag: bool = true;
```

### Functions

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    println!("{}", add(2, 3)); // 5
}
```


## Control Flow

### if
```rust
let number = 7;

if number > 5 {
    println!("Greater than 5");
} else {
    println!("5 or less");
}
```
### For Loop

```rust
for i in 1..=3 {
    println!("{}", i);
}
```


##  Packages 

Edit `Cargo.toml`:

```toml
[dependencies]
rand = "0.8"
```

Use it in your code:

```rust
use rand::Rng;

fn main() {
    let n = rand::thread_rng().gen_range(1..=100);
    println!("Random number: {}", n);
}
```

Run:

```bash
cargo run
```


## Tests

In `src/lib.rs` or `main.rs`:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```

Run tests:

```bash
cargo test
```



## Summary

| Concept        | Example                |
| -------------- | ---------------------- |
| Install        | `rustup`               |
| Create project | `cargo new my_project` |
| Run            | `cargo run`            |
| Build          | `cargo build`          |
| Add crate      | Add to `Cargo.toml`    |
| Write function | `fn my_func() {}`      |
| Conditional    | `if`, `match`          |
| Looping        | `for`, `while`, `loop` |