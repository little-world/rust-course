Here’s a **cookbook-style tutorial** for doing **HTTP I/O in Rust**, using popular crates like [`reqwest`](https://crates.io/crates/reqwest) and [`hyper`](https://crates.io/crates/hyper). This focuses on **client-side HTTP** (requests & responses) with both **sync and async** examples.

---

## Rust HTTP I/O Cookbook (Client-Side)

> 🧩 All examples use \[`reqwest`] for simplicity. Add to `Cargo.toml`:

```toml
reqwest = { version = "0.12", features = ["blocking", "json"] }
tokio = { version = "1", features = ["full"] }
```

---

### GET Request (Blocking)

**✅ Problem**: Download a web page

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = reqwest::blocking::get("https://httpbin.org/get")?
        .text()?;

    println!("{}", body);
    Ok(())
}
```

📘 Great for quick scripts, tools.

---

### GET with Headers

```rust
use reqwest::blocking::Client;
use reqwest::header;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .get("https://httpbin.org/headers")
        .header(header::USER_AGENT, "MyRustApp/1.0")
        .send()?
        .text()?;

    println!("{}", res);
    Ok(())
}
```

---

### POST JSON Payload

```rust
use serde_json::json;
use reqwest::blocking::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .post("https://httpbin.org/post")
        .json(&json!({ "name": "Rust", "awesome": true }))
        .send()?
        .text()?;

    println!("{}", res);
    Ok(())
}
```

📘 Automatically sets `Content-Type: application/json`.

---

### Download File to Disk

```rust
use std::fs::File;
use std::io::copy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut response = reqwest::blocking::get("https://httpbin.org/image/png")?;
    let mut out = File::create("image.png")?;
    copy(&mut response, &mut out)?;

    println!("Downloaded image.png");
    Ok(())
}
```

---

### Async GET (Tokio Runtime)

```rust
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = Client::new()
        .get("https://httpbin.org/get")
        .send()
        .await?
        .text()
        .await?;

    println!("{}", body);
    Ok(())
}
```

📘 Use `#[tokio::main]` for async apps.

---

### Parse JSON Response into Struct

```rust
use serde::Deserialize;
use reqwest::blocking::Client;

#[derive(Debug, Deserialize)]
struct IpInfo {
    origin: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let res: IpInfo = client
        .get("https://httpbin.org/ip")
        .send()?
        .json()?;

    println!("Your IP: {}", res.origin);
    Ok(())
}
```

📘 Use `serde` for parsing structured data.

---

### Set Timeouts and Retries

```rust
use reqwest::blocking::Client;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let res = client.get("https://httpbin.org/delay/3").send()?;
    println!("Success: {}", res.status());
    Ok(())
}
```

💡 Retry manually or with `retry` crate if needed.

---

### Handle Redirects

```rust
use reqwest::blocking::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()?;

    let res = client.get("http://httpbin.org/redirect/2").send()?;
    println!("Final URL: {}", res.url());
    Ok(())
}
```

---

### Set Query Parameters

```rust
use reqwest::blocking::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .get("https://httpbin.org/get")
        .query(&[("lang", "rust"), ("level", "intermediate")])
        .send()?
        .text()?;

    println!("{}", res);
    Ok(())
}
```

---

### Send Form Data

```rust
use reqwest::blocking::Client;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = Client::new()
        .post("https://httpbin.org/post")
        .form(&[("username", "rustacean"), ("pwd", "safe")])
        .send()?
        .text()?;

    println!("{}", res);
    Ok(())
}
```

---

## Tips for HTTP I/O

* ✅ Use **blocking mode** for scripts, **async** for web servers or concurrent tasks.
* ✅ Use `Client` when making multiple requests (keeps connection alive).
* 🔐 Handle timeouts and status codes gracefully.
* 📁 Download files using `copy()` or stream chunks for big files.
