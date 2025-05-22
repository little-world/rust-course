Here’s a **cookbook-style tutorial for `std::time`**, Rust’s standard module for **handling time**, including durations, timestamps, and measuring elapsed time.

---

## Rust std::time Cookbook

> 📦 Module: [`std::time`](https://doc.rust-lang.org/std/time/)

Main types:

* `Duration` – A span of time (e.g., 2 seconds)
* `Instant` – Monotonic clock for measuring elapsed time
* `SystemTime` – Wall-clock time (e.g., current time/date)

---

## Measuring Time

---

### Measure Elapsed Time Using Instant

```rust
use std::time::Instant;

fn main() {
    let start = Instant::now();

    // Do something...
    let sum: u64 = (1..=1_000_000).sum();
    println!("Sum: {}", sum);

    let duration = start.elapsed();
    println!("Elapsed: {:?}", duration);
}
```

📘 `Instant` is monotonic and safe for benchmarking.

---

### Sleep for a Duration

```rust
use std::{thread, time::Duration};

fn main() {
    println!("Sleeping...");
    thread::sleep(Duration::from_secs(2));
    println!("Woke up!");
}
```

---

## Working with Duration

---

### Create a Duration

```rust
use std::time::Duration;

fn main() {
    let d1 = Duration::new(5, 0); // 5 seconds
    let d2 = Duration::from_secs(3);
    let d3 = Duration::from_millis(500);
    println!("{:?} {:?} {:?}", d1, d2, d3);
}
```

---

### Add, Subtract, Compare Durations

```rust
use std::time::Duration;

fn main() {
    let d1 = Duration::from_secs(2);
    let d2 = Duration::from_millis(500);
    let total = d1 + d2;
    println!("Total: {:?}", total); // 2.5 seconds
    println!("Longer? {}", d1 > d2); // true
}
```

---

## Using SystemTime

---

### Get Current System Time

```rust
use std::time::SystemTime;

fn main() {
    let now = SystemTime::now();
    println!("Now: {:?}", now);
}
```

---

### Get UNIX Timestamp

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    println!("Seconds since epoch: {}", since_epoch.as_secs());
}
```

---

### Compare Timestamps

```rust
use std::time::{SystemTime, Duration};

fn main() {
    let t1 = SystemTime::now();
    let t2 = t1 + Duration::from_secs(60);
    let diff = t2.duration_since(t1).unwrap();
    println!("Diff: {} seconds", diff.as_secs());
}
```

📘 Subtraction yields `Result<Duration, SystemTimeError>`.

---

## Summary Table

| Task                    | API                                            |
| ----------------------- | ---------------------------------------------- |
| Elapsed time            | `Instant::now()` + `.elapsed()`                |
| Sleep                   | `thread::sleep(Duration)`                      |
| Current system time     | `SystemTime::now()`                            |
| Time since UNIX epoch   | `SystemTime::now().duration_since(UNIX_EPOCH)` |
| Create duration         | `Duration::from_secs()`, `.new()`              |
| Add/subtract durations  | `+`, `-`, `.checked_add()`, etc.               |
| Compare durations/times | `>`, `<`, `.duration_since()`                  |

---

## When to Use std::time

* **Benchmarking** with `Instant`
* **Timestamps** with `SystemTime`
* **Sleeping/delays** with `Duration`
* **Timeouts and retries** logic
* **File metadata comparisons** (e.g., from `fs::metadata().modified()`)


