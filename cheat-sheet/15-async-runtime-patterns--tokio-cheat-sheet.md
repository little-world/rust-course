### Tokio Cheat Sheet
```rust
// Runtime creation
#[tokio::main]
async fn main() { }                                  // Default runtime (multi-thread)

#[tokio::main(flavor = "current_thread")]
async fn main() { }                                  // Single-threaded runtime

#[tokio::main(worker_threads = 4)]
async fn main() { }                                  // Custom thread count

Runtime::new().unwrap()                              // Manual runtime creation
Builder::new_multi_thread().build().unwrap()         // Builder pattern

// Basic async/await
async fn my_function() -> Result<T, E> { }          // Define async function
my_function().await                                  // Call and await
my_function().await?                                 // Await and propagate error

// Task spawning
tokio::spawn(async { /* code */ })                  // Spawn task, returns JoinHandle
tokio::spawn(async move { /* code */ })             // Move ownership into task
handle.await                                         // Wait for task completion
handle.await.unwrap()                                // Wait and unwrap result
handle.abort()                                       // Cancel task

// Blocking operations
tokio::task::spawn_blocking(|| { /* sync code */ }) // Run blocking code in thread pool
tokio::task::block_in_place(|| { /* sync code */ }) // Block current worker thread

// Task management
tokio::task::yield_now().await                       // Yield to scheduler
tokio::task::spawn_local(async { /* code */ })      // Spawn on LocalSet (not Send)

// Sleeping & timing
tokio::time::sleep(Duration::from_secs(1)).await    // Async sleep
tokio::time::sleep_until(instant).await              // Sleep until specific time
tokio::time::interval(duration)                      // Create ticker
let mut interval = tokio::time::interval(Duration::from_secs(1));
interval.tick().await                                // Wait for next tick

// Timeout
tokio::time::timeout(duration, future).await         // Timeout future, returns Result
tokio::time::timeout_at(instant, future).await       // Timeout at instant

// Channels (async)
let (tx, rx) = mpsc::channel(capacity)              // Bounded MPSC channel
let (tx, rx) = mpsc::unbounded_channel()            // Unbounded MPSC channel
tx.send(value).await                                 // Send, async, returns Result
rx.recv().await                                      // Receive, returns Option<T>
tx.try_send(value)                                   // Non-blocking send
rx.try_recv()                                        // Non-blocking receive

let (tx, rx) = oneshot::channel()                   // One-shot channel
tx.send(value)                                       // Send once (not async)
rx.await                                             // Receive once

let (tx, rx) = broadcast::channel(capacity)         // Broadcast channel
tx.send(value)                                       // Send to all receivers
rx.recv().await                                      // Receive broadcast message

let (tx, rx) = watch::channel(initial)              // Watch channel (shared state)
tx.send(value)                                       // Update watched value
rx.borrow()                                          // Read current value
rx.changed().await                                   // Wait for change

// Synchronization primitives
let mutex = Mutex::new(data)                         // Async Mutex
let guard = mutex.lock().await                       // Lock asynchronously
let guard = mutex.try_lock()                         // Try lock (non-async)

let rw = RwLock::new(data)                          // Async RwLock
let read = rw.read().await                           // Read lock
let write = rw.write().await                         // Write lock

let semaphore = Semaphore::new(n)                    // Semaphore with n permits
let permit = semaphore.acquire().await.unwrap()      // Acquire permit
let permit = semaphore.try_acquire()                 // Try acquire (non-async)
semaphore.add_permits(n)                             // Add permits

let barrier = Barrier::new(n)                        // Async barrier
barrier.wait().await                                 // Wait for n tasks

let notify = Notify::new()                           // Async notification
notify.notified().await                              // Wait for notification
notify.notify_one()                                  // Notify one waiter
notify.notify_waiters()                              // Notify all waiters

// Select (wait on multiple futures)
tokio::select! {
    result1 = future1 => { /* handle */ },
    result2 = future2 => { /* handle */ },
    else => { /* no future ready */ }
}

// Join (wait for all)
tokio::join!(future1, future2, future3)             // Wait for all, return tuple
let (r1, r2) = tokio::join!(f1, f2);

// Try join (wait for all, propagate errors)
tokio::try_join!(future1, future2)                  // Return Result of tuple

// Pin & unpin
futures::pin_mut!(future)                            // Pin to stack
Box::pin(future)                                     // Pin to heap

// Stream operations (requires tokio-stream crate)
use tokio_stream::StreamExt;
stream.next().await                                  // Get next item
stream.collect::<Vec<_>>().await                    // Collect all items
stream.for_each(|item| async { /* work */ }).await // Process each item
stream.map(|x| x * 2)                               // Transform items
stream.filter(|x| future_returns_bool)              // Filter items
stream.take(n)                                       // Take first n items
stream.skip(n)                                       // Skip first n items

// File I/O (async)
use tokio::fs;
fs::read("file.txt").await                           // Read entire file
fs::write("file.txt", data).await                    // Write entire file
fs::read_to_string("file.txt").await                // Read as string
let file = fs::File::open("file.txt").await         // Open file
let mut file = fs::File::create("file.txt").await   // Create file

use tokio::io::{AsyncReadExt, AsyncWriteExt};
file.read_to_end(&mut buf).await                    // Read to buffer
file.write_all(data).await                           // Write all bytes
file.read_exact(&mut buf).await                     // Read exact amount

// Network I/O
use tokio::net::{TcpListener, TcpStream};
let listener = TcpListener::bind("127.0.0.1:8080").await // Bind TCP listener
let (socket, addr) = listener.accept().await        // Accept connection
let stream = TcpStream::connect("127.0.0.1:8080").await // Connect TCP

stream.readable().await                              // Wait until readable
stream.writable().await                              // Wait until writable
stream.read(&mut buf).await                          // Read from socket
stream.write_all(data).await                         // Write to socket

// UDP
use tokio::net::UdpSocket;
let socket = UdpSocket::bind("127.0.0.1:8080").await // Bind UDP
socket.send_to(data, addr).await                     // Send datagram
socket.recv_from(&mut buf).await                     // Receive datagram

// Process spawning
use tokio::process::Command;
let output = Command::new("ls").output().await       // Run and get output
let child = Command::new("ls").spawn()               // Spawn child process
child.wait().await                                   // Wait for child

// Signal handling
use tokio::signal;
signal::ctrl_c().await                               // Wait for Ctrl+C

// Graceful shutdown pattern
let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
tokio::select! {
    _ = server_task => {},
    _ = shutdown_rx.recv() => {
        // cleanup
    }
}

// Retry pattern with backoff
use tokio::time::{sleep, Duration};
let mut retries = 0;
loop {
    match operation().await {
        Ok(result) => break result,
        Err(e) if retries < 3 => {
            retries += 1;
            sleep(Duration::from_secs(2u64.pow(retries))).await;
        }
        Err(e) => return Err(e),
    }
}

// Common patterns
let shared = Arc::new(Mutex::new(data));             // Shared async state
let clone = Arc::clone(&shared);
tokio::spawn(async move {
    let mut guard = clone.lock().await;
    *guard += 1;
});

// Fan-out pattern (spawn multiple tasks)
let handles: Vec<_> = (0..10)
    .map(|i| tokio::spawn(async move { work(i).await }))
    .collect();
for handle in handles {
    handle.await.unwrap();
}

// Concurrent stream processing
use futures::stream::{self, StreamExt};
stream::iter(items)
    .for_each_concurrent(10, |item| async move {
        process(item).await;
    })
    .await;
```