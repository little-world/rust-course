

## Cargo 
### Rusts Build System and Package Manager

### Creating a Project

```bash
cargo new hello_project
cd hello_project
```

This creates:

```
hello_project/
├── Cargo.toml       # project metadata & dependencies
└── src/
    └── main.rs      # entry point
```

### Build and Run

```bash
cargo build      # compile
cargo run        # compile + run
cargo check      # check for errors only
```

---

## Modules

### Inline Module:

```rust
mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

fn main() {
    println!("{}", math::add(2, 3)); // 5
}
```

### File-Based Module:

In `main.rs` or `lib.rs`:

```rust
mod utils;
```

In `src/utils.rs`:

```rust
pub fn greet(name: &str) {
    println!("Hello, {}", name);
}
```

Now you can call:

```rust
utils::greet("Alice");
```

## Nested Modules 
### (Directories)

* Declare in `main.rs`:

```rust
mod services;
```

* Directory structure:

```
src/
├── main.rs
└── services/
    ├── mod.rs
    └── api.rs
```

* Inside `mod.rs`:

```rust
pub mod api;
```

* Inside `api.rs`:

```rust
pub fn handler() {
    println!("Handling request...");
}
```

* Call from `main.rs`:

```rust
services::api::handler();
```

---

## Crates 


### Binary Crate

Has a `main.rs` and a `fn main()` entry point.

```bash
cargo new my_app --bin
```

### Library Crate

No main function. Used as a dependency or helper.

```bash
cargo new my_utils --lib
```

Creates:

```
my_utils/
└── src/lib.rs
```

Add functions here and publish or import into other projects.

---

##  External Crates

Edit your `Cargo.toml`:

```toml
[dependencies]
rand = "0.8"
```

Use it in your code:

```rust
use rand::Rng;

fn main() {
    let x = rand::thread_rng().gen_range(1..=100);
    println!("Random number: {}", x);
}
```

Run:

```bash
cargo build
```

---

## Publishing


1. Register at [https://crates.io](https://crates.io)
2. Login via terminal:

```bash
cargo login YOUR_API_KEY
```

3. Ensure your `Cargo.toml` has a `[package]` section.
4. Publish:

```bash
cargo publish
```



## Summary 

| Concept | Purpose                         | Example                    |
| ------- | ------------------------------- | -------------------------- |
| `Cargo` | Build & dependency manager      | `cargo build`, `cargo run` |
| `mod`   | Declare module (file or inline) | `mod utils;`               |
| `crate` | Reusable package                | Binary or library          |
| `pub`   | Make functions/types public     | `pub fn greet() {}`        |
| `use`   | Bring path into scope           | `use crate::utils::greet;` |