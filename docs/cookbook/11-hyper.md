
## 1. Create a new project

```bash
cargo new hyper-tutorial
cd hyper-tutorial
```

---

## 2. Add dependencies

Edit your `Cargo.toml` to include Hyper (with its full feature set) and Tokio:

```toml
[dependencies]
tokio  = { version = "1", features = ["full"] }
hyper  = { version = "0.14", features = ["full"] }
```

* **Tokio** provides the async runtime (`#[tokio::main]` macro, async TCP, timers, etc.).
* **Hyper** gives you an HTTP/1⁄1 & HTTP/2 client/server library with zero-cost abstractions.

---

## 3. Boilerplate server code

Replace the contents of `src/main.rs` with:

```rust
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use std::net::SocketAddr;

/// Our request handler: always returns a 200 OK with “Hello, World!”
async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!(">> {} {}", req.method(), req.uri());
    Ok(Response::new(Body::from("Hello, World!")))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Define the socket address to bind to
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    // 2. Create a “make service” to handle each connection
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our `handle` fn into a `Service`
        Ok::<_, Infallible>(service_fn(handle))
    });

    // 3. Build and run the server
    let server = Server::bind(&addr).serve(make_svc);

    // 4. Await the server future (will run until killed)
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
```

**What’s happening?**

1. **Imports**

   ```rust
   use hyper::{Body, Request, Response, Server};
   use hyper::service::{make_service_fn, service_fn};
   ```

    * `Body` is the HTTP body type.
    * `make_service_fn` and `service_fn` let you turn an async function into a connection handler.

2. **Handler**

   ```rust
   async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> { … }
   ```

    * Logs the incoming request.
    * Returns a simple `200 OK` response.

3. **Server bootstrap**

   ```rust
   let make_svc = make_service_fn(|_conn| async {
       Ok::<_, Infallible>(service_fn(handle))
   });
   let server = Server::bind(&addr).serve(make_svc);
   ```

    * `Server::bind(&addr)` opens the TCP listener.
    * `.serve(...)` drives the accept loop internally.

4. **Run**

   ```rust
   #[tokio::main]
   async fn main() { … server.await … }
   ```

    * Starts the Tokio runtime and awaits the `Server` future.

---

## 4. Run and test

```bash
cargo run
# → Listening on http://127.0.0.1:3000
```

In another terminal:

```bash
curl http://127,0,0,1:3000
# → Hello, World!
```

---

## 5. Adding routing

For more complex routing you can match on `req.uri().path()`:

```rust
async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match req.uri().path() {
        "/" => Ok(Response::new(Body::from("Welcome!"))),
        "/health" => Ok(Response::new(Body::from("OK"))),
        _ => {
            let mut not_found = Response::new(Body::from("Not Found"));
            *not_found.status_mut() = hyper::StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
```

---

## 6. Simple Hyper client

You can also make HTTP requests with Hyper’s client API:

```rust
use hyper::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();
    let uri = "http://httpbin.org/ip".parse()?;

    let resp = client.get(uri).await?;
    println!("Status: {}", resp.status());

    let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
    println!("Body: {}", String::from_utf8_lossy(&body_bytes));

    Ok(())
}
```
