# async/await in Rust with tokio

- [`tokio`](https://crates.io/crates/tokio)

— Rust’s most popular async runtime. We'll focus on **realistic patterns**: HTTP calls, file I/O, delays, tasks, and concurrency.

---

## Rust async/await with Tokio Cookbook

> 📦 Add to `Cargo.toml`:

```toml
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "gzip", "brotli", "deflate", "cookies", "stream"] }
```

---

### Basic Async Function

```rust
#[tokio::main]
async fn main() {
    let result = say_hello().await;
    println!("{}", result);
}

async fn say_hello() -> &'static str {
    "Hello, async world!"
}
```

📘 `#[tokio::main]` starts the async runtime.

---

### Sleep with Delay

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("Waiting...");
    sleep(Duration::from_secs(2)).await;
    println!("Done!");
}
```

📘 Prefer `tokio::time::sleep` over `std::thread::sleep`.

---

### HTTP GET with reqwest + async

```rust
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = Client::new();
    let body = client.get("https://httpbin.org/get")
        .send().await?
        .text().await?;

    println!("{}", body);
    Ok(())
}
```

---

### Spawn Concurrent Tasks

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let t1 = task::spawn(async {
        println!("Task 1");
    });

    let t2 = task::spawn(async {
        println!("Task 2");
    });

    t1.await.unwrap();
    t2.await.unwrap();
}
```

---

### Join Multiple Futures

```rust
use tokio::join;

async fn task1() -> i32 {
    1
}
async fn task2() -> i32 {
    2
}

#[tokio::main]
async fn main() {
    let (a, b) = join!(task1(), task2());
    println!("Results: {} + {} = {}", a, b, a + b);
}
```

---

### Use select! to Race Tasks

```rust
use tokio::time::{sleep, Duration};
use tokio::select;

#[tokio::main]
async fn main() {
    select! {
        _ = sleep(Duration::from_secs(1)) => println!("1s timeout"),
        _ = sleep(Duration::from_secs(2)) => println!("2s timeout"),
    }
}
```

📘 First one to complete wins.

---

### Async Read File to String

```rust
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let content = fs::read_to_string("data.txt").await?;
    println!("{}", content);
    Ok(())
}
```

---

### Write to File Asynchronously

```rust
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut file = File::create("output.txt").await?;
    file.write_all(b"Async Rust FTW!\n").await?;
    Ok(())
}
```

---

### Create a Simple Async HTTP Server (with tokio + hyper)

```rust
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn hello(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello from async server")))
}

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let make_svc = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(hello)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
```

📘 Use `hyper`, `warp`, or `axum` for production servers.

---

### Timeout a Future

```rust
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    let result = timeout(Duration::from_secs(2), async {
        // pretend this takes time
        tokio::time::sleep(Duration::from_secs(5)).await;
        "done"
    }).await;

    match result {
        Ok(msg) => println!("Finished: {}", msg),
        Err(_) => println!("Timed out!"),
    }
}
```

---

## Common Patterns

| Pattern             | Tool / Crate                       |
| ------------------- | ---------------------------------- |
| Concurrency         | `tokio::spawn`, `join!`, `select!` |
| Timers & timeouts   | `tokio::time`                      |
| File I/O            | `tokio::fs`                        |
| HTTP client         | `reqwest`                          |
| HTTP server         | `hyper`, `warp`, `axum`            |
| Channels            | `tokio::sync::mpsc`                |
| Mutex, RwLock, etc. | `tokio::sync`                      |

