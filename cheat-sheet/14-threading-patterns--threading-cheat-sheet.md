### Threading Cheat Sheet
```rust
// Thread creation
thread::spawn(|| { /* code */ })                    // Spawn new thread, returns JoinHandle
thread::spawn(move || { /* code */ })               // Move ownership into thread
thread::Builder::new()
    .name("thread_name".into())
    .stack_size(size)
    .spawn(|| { /* code */ })                       // Configure thread before spawn

// Thread control
handle.join()                                        // Wait for thread, returns Result<T>
handle.join().unwrap()                               // Wait and unwrap result
thread::sleep(Duration::from_secs(1))               // Sleep current thread
thread::yield_now()                                  // Yield to scheduler
thread::current()                                    // Get current thread handle
thread::current().id()                               // Get thread ID
thread::current().name()                             // Get thread name

// Scoped threads (guaranteed lifetime)
thread::scope(|s| {
    s.spawn(|| { /* code */ });                     // Spawn scoped thread
    s.spawn(|| { /* code */ });
});                                                  // Auto-joins all threads

// Mutex (mutual exclusion)
let m = Mutex::new(data)                            // Create mutex
let guard = m.lock().unwrap()                       // Lock, blocks until available
let guard = m.try_lock()                            // Try lock, returns Result
*guard = new_value                                  // Modify protected data
drop(guard)                                         // Explicit unlock (auto on scope exit)

// RwLock (multiple readers, single writer)
let rw = RwLock::new(data)                          // Create RwLock
let read_guard = rw.read().unwrap()                 // Acquire read lock (shared)
let write_guard = rw.write().unwrap()               // Acquire write lock (exclusive)
let read_guard = rw.try_read()                      // Try read lock
let write_guard = rw.try_write()                    // Try write lock

// Arc (atomic reference counting for shared ownership)
let arc = Arc::new(data)                            // Create Arc
let clone = Arc::clone(&arc)                        // Clone reference (cheap)
let clone = arc.clone()                             // Alternative syntax
Arc::strong_count(&arc)                             // Get reference count
Arc::try_unwrap(arc)                                // Extract value if only one reference

// Channels (message passing)
let (tx, rx) = mpsc::channel()                      // Unbounded channel
let (tx, rx) = mpsc::sync_channel(capacity)         // Bounded channel
tx.send(value)                                      // Send, returns Result
tx.send(value).unwrap()                             // Send and unwrap
rx.recv()                                           // Receive, blocks, returns Result<T>
rx.try_recv()                                       // Non-blocking receive
rx.recv_timeout(duration)                           // Receive with timeout
let tx2 = tx.clone()                                // Clone sender (multiple producers)

// Channel iteration
for msg in rx { /* process */ }                     // Iterate until sender dropped
while let Ok(msg) = rx.recv() { /* process */ }     // Explicit loop

// Atomic types
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, Ordering};
let atom = AtomicI32::new(0)                        // Create atomic
atom.load(Ordering::SeqCst)                         // Read value
atom.store(5, Ordering::SeqCst)                     // Write value
atom.fetch_add(1, Ordering::SeqCst)                 // Atomic increment, return old
atom.fetch_sub(1, Ordering::SeqCst)                 // Atomic decrement
atom.swap(10, Ordering::SeqCst)                     // Swap value
atom.compare_exchange(old, new, success, failure)   // CAS operation

// Memory orderings
Ordering::Relaxed                                    // No ordering guarantees
Ordering::Acquire                                    // Read barrier
Ordering::Release                                    // Write barrier
Ordering::AcqRel                                     // Both acquire and release
Ordering::SeqCst                                     // Sequential consistency (safest)

// Barrier (synchronization point)
let barrier = Arc::new(Barrier::new(n))             // Create barrier for n threads
barrier.wait()                                       // Wait until n threads arrive

// Condvar (condition variable)
let pair = Arc::new((Mutex::new(false), Condvar::new()));
let (lock, cvar) = &*pair;
let mut started = lock.lock().unwrap();
*started = true;
cvar.notify_one()                                    // Wake one waiting thread
cvar.notify_all()                                    // Wake all waiting threads
cvar.wait(guard)                                     // Wait, releases lock until notified
cvar.wait_timeout(guard, duration)                   // Wait with timeout

// Once (one-time initialization)
let once = Once::new()                               // Create Once
once.call_once(|| { /* init code */ })              // Run once across all threads
once.is_completed()                                  // Check if already called

// Thread-local storage
thread_local! {
    static FOO: RefCell<u32> = RefCell::new(1);
}
FOO.with(|f| *f.borrow_mut() += 1)                  // Access thread-local

// Common patterns
let shared = Arc::new(Mutex::new(vec![]))           // Shared mutable state
let shared_clone = Arc::clone(&shared);
thread::spawn(move || {
    shared_clone.lock().unwrap().push(1);
});

let (tx, rx) = mpsc::channel();                      // Producer-consumer
thread::spawn(move || {
    tx.send(42).unwrap();
});
let result = rx.recv().unwrap();

// Rayon (parallel iterators - external crate)
use rayon::prelude::*;
vec.par_iter().map(|x| x * 2).collect()             // Parallel map
vec.par_iter().for_each(|x| { /* work */ })         // Parallel for-each
```
