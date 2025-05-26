
## Threads
### Spawning Threads

Rust uses **`std::thread`** for native OS threads.

```rust
use std::thread;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..5 {
            println!("From thread: {}", i);
        }
    });

    for i in 1..5 {
        println!("From main: {}", i);
    }

    handle.join().unwrap(); // wait for thread to finish
}
```

* `spawn()` runs code in a new thread.
* `join()` blocks until it’s done.


### Using move in Threads

To send data **into** the thread:

```rust
let data = vec![1, 2, 3];

let handle = thread::spawn(move || {
    println!("Moved data: {:?}", data);
});

// println!("{:?}", data); // ❌ cannot access after moved
handle.join().unwrap();
```

* The `move` keyword **transfers ownership** into the thread.



## Channels: 
### Thread Communication

Rust channels are used to **send messages** between threads safely.

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send("hello from thread").unwrap();
    });

    let received = rx.recv().unwrap();
    println!("Got: {}", received);
}
```

* `tx.send(...)` sends data.
* `rx.recv()` blocks until a value is received.

### Multiple senders:

```rust
let (tx, rx) = mpsc::channel();
let tx1 = tx.clone();
```

---

## Mutex
### Shared Mutable State Arc<Mutex<T>

You can **safely share and mutate data across threads** using:

* `Arc` for atomic reference-counted ownership.
* `Mutex` for mutual exclusion.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap(); // lock the mutex
            *num += 1;
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
}
```

> `Arc<Mutex<T>>` is the go-to combo for shared mutation across threads.

---

## Async
### Brief Intro to Async/Await

Rust also supports **async/await** for asynchronous concurrency using **Futures** and **executors** (e.g., `tokio`, `async-std`).

```rust
async fn greet() {
    println!("Hello from async!");
}

#[tokio::main]
async fn main() {
    greet().await;
}
```

> Async is **not the same as threading** — it’s for I/O concurrency, not CPU-heavy tasks.


## Summary 

| Feature         | Use For                          | Crate Needed |
| --------------- | -------------------------------- | ----------- |
| `thread::spawn` | Run code in parallel threads     | Std only    |
| `move`          | Pass ownership to a thread       | Std only    |
| `mpsc::channel` | Communicate between threads      | Std only    |
| `Mutex<T>`      | Mutually exclusive access        | Std only    |
| `Arc<T>`        | Shared ownership between threads | Std only    |
| `async/await`   | Asynchronous I/O concurrency     | Yes (tokio) |



## Best Practices

* Use `thread::spawn` for **CPU-bound** tasks.
* Use channels or `Arc<Mutex<T>>` for **safe communication and shared state**.
* Prefer `async` for **I/O-bound** work (e.g., network or file I/O).
* Avoid data races with `Mutex` and thread safety tools.