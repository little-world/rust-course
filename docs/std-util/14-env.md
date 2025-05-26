Here’s a **cookbook-style tutorial for `std::env`**, Rust’s standard library module for interacting with the **process environment** — variables, arguments, working directory, and executable path.

---

## Rust std::env Cookbook

> 📦 Module: [`std::env`](https://doc.rust-lang.org/std/env/)

Common tasks include:

* Reading and setting environment variables
* Accessing command-line arguments
* Querying current directory and executable path

---

## Environment Variables

---

### Read an Environment Variable

```rust
use std::env;

fn main() {
    if let Ok(home) = env::var("HOME") {
        println!("Your home is: {}", home);
    } else {
        println!("HOME not set");
    }
}
```

---

### Set an Environment Variable

```rust
use std::env;

fn main() {
    env::set_var("MY_VAR", "hello");
    println!("MY_VAR = {}", env::var("MY_VAR").unwrap());
}
```

📘 Useful in tests or subprocess configurations.

---

### Remove an Environment Variable

```rust
use std::env;

fn main() {
    env::remove_var("MY_VAR");
}
```

---

### Iterate Over All Environment Variables

```rust
use std::env;

fn main() {
    for (key, value) in env::vars() {
        println!("{} = {}", key, value);
    }
}
```

---

## Command-line Arguments

---

### Read All Arguments

```rust
use std::env;

fn main() {
    for (i, arg) in env::args().enumerate() {
        println!("arg[{}] = {}", i, arg);
    }
}
```

📘 The first argument is usually the program name.

---

### Read Arguments as OS Strings (lossless)

```rust
use std::env;

fn main() {
    for arg in env::args_os() {
        println!("{:?}", arg);
    }
}
```

📘 Use this when dealing with non-UTF-8 paths or platforms.

---

## Current Directory and Executable Path

---

### Get Current Directory

```rust
use std::env;

fn main() {
    let cwd = env::current_dir().unwrap();
    println!("Current dir: {:?}", cwd);
}
```

---

### Set Current Directory

```rust
use std::env;

fn main() {
    env::set_current_dir("/tmp").unwrap();
    println!("Changed directory!");
}
```

---

### Get Path to Current Executable

```rust
use std::env;

fn main() {
    let path = env::current_exe().unwrap();
    println!("Executable path: {:?}", path);
}
```

---

## Summary Table

| Task                     | Function                     |
| ------------------------ | ---------------------------- |
| Get environment variable | `env::var("KEY")`            |
| Set environment variable | `env::set_var("KEY", "val")` |
| Remove variable          | `env::remove_var("KEY")`     |
| Iterate all variables    | `env::vars()`                |
| Read command-line args   | `env::args()`                |
| Get current dir          | `env::current_dir()`         |
| Set current dir          | `env::set_current_dir(path)` |
| Get executable path      | `env::current_exe()`         |

---

## When to Use std::env

* Reading **config from environment variables**
* Building **command-line tools**
* Setting **test or build flags**
* Managing **current working directory**
