Here’s a **cookbook-style tutorial for `std::path`**, Rust’s standard module for safely and idiomatically manipulating **file system paths**, using the `Path` and `PathBuf` types.

---

## Rust std::path Cookbook

> 📦 Module: [`std::path`](https://doc.rust-lang.org/std/path/)

* `Path`: an immutable slice of a path (like `&str`)
* `PathBuf`: an owned, mutable path (like `String`)

Use it with `std::fs`, `std::io`, and `std::env`.

---

## Path Creation and Conversion

---

### Create a Path from a String

```rust
use std::path::Path;

fn main() {
    let path = Path::new("foo/bar.txt");
    println!("Display: {}", path.display());
}
```

📘 `Path::new` creates a borrowed path (`&Path`).

---

### Create a PathBuf (Owned Path)

```rust
use std::path::PathBuf;

fn main() {
    let mut path = PathBuf::from("foo");
    path.push("bar.txt");
    println!("{:?}", path); // "foo/bar.txt"
}
```

---

### Convert Between Path and OsStr / &str

```rust
use std::path::Path;

fn main() {
    let path = Path::new("file.rs");
    if let Some(name) = path.to_str() {
        println!("Path as str: {}", name);
    }
}
```

📘 Always check `.to_str()` result — it may fail on invalid UTF-8.

---

## Path Inspection

---

### Get File Name, Extension, and Components

```rust
use std::path::Path;

fn main() {
    let path = Path::new("/home/user/file.txt");

    println!("File name: {:?}", path.file_name());
    println!("Extension: {:?}", path.extension());
    println!("Parent: {:?}", path.parent());
}
```

---

### Check If Path Exists / Is File / Is Dir

```rust
use std::path::Path;

fn main() {
    let path = Path::new("Cargo.toml");
    println!("Exists? {}", path.exists());
    println!("Is file? {}", path.is_file());
    println!("Is dir? {}", path.is_dir());
}
```

---

## Path Manipulation

---

### Join Paths Safely

```rust
use std::path::{Path, PathBuf};

fn main() {
    let base = Path::new("/usr/bin");
    let full: PathBuf = base.join("rustc");
    println!("{:?}", full); // "/usr/bin/rustc"
}
```

---

### Canonicalize (Resolve Symbolic Links, .., etc.)

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    let canon = fs::canonicalize("src/../Cargo.toml")?;
    println!("{:?}", canon);
    Ok(())
}
```

---

### Iterate Over Components

```rust
use std::path::Path;

fn main() {
    let path = Path::new("/home/user/docs/file.txt");
    for comp in path.components() {
        println!("{:?}", comp);
    }
}
```

---

### Convert Path to Absolute (Manual Method)

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    let rel = PathBuf::from("Cargo.toml");
    let abs = env::current_dir().unwrap().join(rel);
    println!("{:?}", abs);
}
```

---

## Path Use with fs and env

---

### Use in File Operations

```rust
use std::fs;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let path = Path::new("example.txt");
    let content = fs::read_to_string(path)?;
    println!("{}", content);
    Ok(())
}
```

---

### Get Current Working Directory

```rust
use std::env;

fn main() {
    let cwd = env::current_dir().unwrap();
    println!("Current dir: {:?}", cwd);
}
```

---

## Summary Table

| Task                  | Method/Type                         |
| --------------------- | ----------------------------------- |
| Create path           | `Path::new()`, `PathBuf::from()`    |
| Append path           | `PathBuf.push()`                    |
| Join path             | `Path::join()`                      |
| Convert to str        | `path.to_str()`                     |
| File name / extension | `file_name()`, `extension()`        |
| Path existence        | `exists()`, `is_file()`, `is_dir()` |
| Canonicalize path     | `fs::canonicalize()`                |
| Iterate components    | `path.components()`                 |

---

## When to Use std::path

* Handling filesystem paths safely (cross-platform)
* Creating robust CLI tools
* Working with paths in `std::fs` or `walkdir`/`glob` crates

