# File I/O in Rust using the standard library (std::fs and std::io).
This covers reading, writing, appending, and more — all safely and idiomatically.



### Read an Entire File to a String

**✅ Problem**: Load a file's content as `String`.

```rust
use std::fs;

fn main() {
    let content = fs::read_to_string("data.txt").expect("Failed to read file");
    println!("{}", content);
}
```

📘 Efficient and safe. Panics if the file doesn't exist.

💡 Use `?` instead of `expect()` in real apps.

---

### Write a String to a File (Overwrite)

```rust
use std::fs;

fn main() {
    fs::write("output.txt", "Hello, file!").expect("Failed to write");
}
```

📘 Overwrites the file if it exists, creates it if not.

---

### Append to a File

```rust
use std::fs::OpenOptions;
use std::io::Write;

fn main() {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("log.txt")
        .expect("Failed to open file");

    writeln!(file, "Log line").expect("Write failed");
}
```

💡 `writeln!` adds a newline.

---

### Read File Line by Line

```rust
use std::fs::File;
use std::io::{self, BufRead};

fn main() {
    let file = File::open("data.txt").expect("Cannot open file");
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        println!("{}", line);
    }
}
```

📘 Uses buffered reading to be memory-efficient.

---

### Read Binary File into Bytes

```rust
use std::fs;

fn main() {
    let bytes = fs::read("image.png").expect("Failed to read");
    println!("{} bytes read", bytes.len());
}
```

📘 Useful for media, encryption, etc.

---

### Write Binary Data

```rust
use std::fs::File;
use std::io::Write;

fn main() {
    let data = [1u8, 2, 3, 4];
    let mut file = File::create("data.bin").expect("Failed to create");
    file.write_all(&data).expect("Write failed");
}
```

---

### Buffered File Writing (Efficient Logging)

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    let file = File::create("buffered.txt").expect("Failed to open");
    let mut writer = BufWriter::new(file);

    writeln!(writer, "Buffered write!").unwrap();
}
```

💡 Reduces system calls for frequent small writes.

---

### Check If a File Exists

```rust
use std::path::Path;

fn main() {
    if Path::new("data.txt").exists() {
        println!("File exists!");
    }
}
```

---

### Read File Metadata (Size, Modified Time)

```rust
use std::fs;

fn main() {
    let metadata = fs::metadata("data.txt").expect("No metadata");
    println!("File size: {}", metadata.len());
}
```

💡 Use `metadata.modified()?` for timestamps.

---

### Delete a File

```rust
use std::fs;

fn main() {
    fs::remove_file("old.txt").expect("Failed to delete");
}
```

📘 Won’t panic if the file is already gone — handle gracefully.

---

### Create a Directory

```rust
use std::fs;

fn main() {
    fs::create_dir_all("logs/2025").expect("Failed to create dir");
}
```

💡 Use `create_dir_all()` for nested dirs.

---

### List Files in a Directory

```rust
use std::fs;

fn main() {
    let entries = fs::read_dir(".").expect("Read dir failed");

    for entry in entries {
        let entry = entry.expect("Failed entry");
        println!("{:?}", entry.path());
    }
}
```

---

### Copy or Rename Files

```rust
use std::fs;

fn main() {
    fs::copy("a.txt", "b.txt").expect("Copy failed");
    fs::rename("b.txt", "c.txt").expect("Rename failed");
}
```

---

## Tips for File I/O in Rust

* Use `BufReader`/`BufWriter` for performance on large files.
* Prefer `?` over `unwrap()` for better error handling in real apps.
* Always check paths with `Path` API before assuming access.
* Wrap file logic in functions returning `Result<_, io::Error>`.

