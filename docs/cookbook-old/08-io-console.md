Here’s a **cookbook-style tutorial** for **console I/O in Rust**, covering common patterns for reading from and writing to the console. It focuses on using the standard library only (`std::io`, `println!`, `stdin()`, etc.).

---

## Rust Console I/O Cookbook

Each recipe includes:

* ✅ **Problem**
* 🔧 **Solution**
* 📘 **Explanation**
* 💡 **Tips**

---

### Print a Line to Console

**✅ Problem**: Output a message

```rust
fn main() {
    println!("Hello, world!");
}
```

📘 **Explanation**: Adds a newline after the output.

💡 Use `print!()` (no newline) for inline prompts.

---

### Print Variables with Formatting

```rust
fn main() {
    let name = "Alice";
    let age = 30;
    println!("Name: {}, Age: {}", name, age);
}
```

📘 Use `{}` for general display, `{:?}` for debug.

---

### Read a Line from Input

**✅ Problem**: Read a string from the user

```rust
use std::io;

fn main() {
    let mut input = String::new();
    println!("Enter your name:");
    io::stdin().read_line(&mut input).expect("Failed to read");
    println!("Hello, {}", input.trim());
}
```

📘 `read_line()` includes the newline character — trim it!

---

### Parse Input into a Number

**✅ Problem**: Read and convert input to a number

```rust
use std::io;

fn main() {
    let mut input = String::new();
    println!("Enter a number:");

    io::stdin().read_line(&mut input).expect("Failed to read");

    let number: i32 = input.trim().parse().expect("Invalid number");
    println!("You entered: {}", number);
}
```

💡 Handle parsing errors with `match` or `Result`.

---

### Prompt and Read Multiple Values

**✅ Problem**: Read space-separated values

```rust
use std::io;

fn main() {
    let mut input = String::new();
    println!("Enter two numbers:");

    io::stdin().read_line(&mut input).unwrap();
    let nums: Vec<i32> = input
        .trim()
        .split_whitespace()
        .map(|s| s.parse().unwrap())
        .collect();

    println!("Sum: {}", nums[0] + nums[1]);
}
```

---

### Loop Until Valid Input

**✅ Problem**: Keep asking until the user enters a number

```rust
use std::io;

fn main() {
    loop {
        let mut input = String::new();
        println!("Enter a valid integer:");

        io::stdin().read_line(&mut input).unwrap();

        match input.trim().parse::<i32>() {
            Ok(n) => {
                println!("Got it: {}", n);
                break;
            }
            Err(_) => println!("Try again."),
        }
    }
}
```

---

### Flush Console Output

**✅ Problem**: Ensure `print!()` is shown before input prompt

```rust
use std::io::{self, Write};

fn main() {
    print!("Enter your name: ");
    io::stdout().flush().unwrap();

    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();

    println!("Hello, {}", name.trim());
}
```

📘 Use `flush()` when `print!()` doesn't auto-flush.

---

### Read Multiple Lines Until EOF

**✅ Problem**: Read until Ctrl+D (Unix) or Ctrl+Z (Windows)

```rust
use std::io::{self, BufRead};

fn main() {
    println!("Enter lines (Ctrl+D to end):");

    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        println!("You typed: {}", line);
    }
}
```

---

### Use Buffered Writing (for Performance)

```rust
use std::io::{self, BufWriter, Write};

fn main() {
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    writeln!(writer, "Buffered output").unwrap();
}
```

💡 Ideal for large outputs like logs, CSVs, etc.

---

### Redirect Input/Output (Testable Design)

**✅ Problem**: Allow testing by using traits

```rust
use std::io::{self, Read, Write};

fn echo<R: Read, W: Write>(mut input: R, mut output: W) {
    let mut buffer = String::new();
    input.read_to_string(&mut buffer).unwrap();
    writeln!(output, "Echo: {}", buffer.trim()).unwrap();
}

fn main() {
    echo(io::stdin(), io::stdout());
}
```

