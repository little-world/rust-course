## Project 2: Async Task Scheduler with Pin

### Problem Statement

Build an asynchronous task scheduler that:
- Supports async/await futures with self-referential state
- Uses `Pin` to safely handle futures that reference their own data
- Implements a custom executor for cooperative multitasking
- Handles wake notifications and task scheduling
- Supports futures that hold references across await points
- Implements higher-ranked trait bounds for flexible async closures
- Demonstrates why Pin is necessary for async Rust
- Provides both heap-pinned (Box<Pin>) and stack-pinned futures

The scheduler must safely handle the self-referential nature of async state machines.

### Why It Matters

Understanding Pin is essential for async Rust:
- **Async Runtimes**: Tokio, async-std rely on Pin
- **Self-Referential Futures**: Futures hold pointers to their own stack frames
- **Zero-Copy Async**: Avoid allocations while preserving safety
- **Custom Executors**: Building specialized async runtimes
- **Generator Desugaring**: How async/await translates to state machines

Without Pin:
- Can't safely implement async/await
- Self-referential structs unsound (moving breaks pointers)
- No way to guarantee futures won't move
- Async would require boxing everything (performance cost)

### Use Cases

1. **Custom Async Runtime**: Application-specific task scheduling
2. **Embedded Systems**: Async without heap allocation
3. **Game Loop**: Cooperative multitasking for game logic
4. **State Machines**: Explicit async state management
5. **Protocol Implementations**: Self-referential parser state
6. **Streaming**: Async iterators with borrowed state
7. **Resource Management**: Async RAII with self-references

### Solution Outline

**Core Structure:**
```rust
use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

// Simple future that's self-referential
struct SelfRefFuture {
    data: String,
    // Reference to data (would be invalid if moved)
    data_ptr: *const String,
    _pin: PhantomPinned,
}

impl Future for SelfRefFuture {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safe because we're pinned
        unsafe {
            let data_ref = &*self.data_ptr;
            Poll::Ready(data_ref.as_str())
        }
    }
}

// Executor that schedules tasks
struct Executor {
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push(Box::pin(future));
    }

    fn run(&mut self) {
        // Poll all tasks until complete
    }
}
```

**Key Pin Patterns:**
- **Box::pin()**: Heap-pin for dynamic dispatch
- **Pin::new_unchecked()**: Unsafe pinning for static guarantees
- **Pin projection**: Safely access fields of pinned structs
- **PhantomPinned**: Marker to prevent Unpin auto-impl

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_pinned_future() {
    let future = Box::pin(SelfRefFuture::new("test"));
    // Verify future can't be moved
    // Verify polling works
}

#[test]
fn test_executor_runs_tasks() {
    let mut executor = Executor::new();
    let mut counter = 0;

    executor.spawn(async {
        counter += 1;
    });

    executor.run();
    assert_eq!(counter, 1);
}
```

---

## Step-by-Step Implementation Guide

### Step 1: Synchronous Task Queue

**Goal:** Build a basic task scheduler without async.

**What to implement:**
```rust
type Task = Box<dyn FnOnce()>;

pub struct SyncScheduler {
    tasks: Vec<Task>,
}

impl SyncScheduler {
    pub fn new() -> Self {
        SyncScheduler {
            tasks: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, task: F)
    where
        F: FnOnce() + 'static,
    {
        self.tasks.push(Box::new(task));
    }

    pub fn run(&mut self) {
        while let Some(task) = self.tasks.pop() {
            task();
        }
    }
}
```

**Check/Test:**
- Test spawning and running tasks
- Test tasks execute in order
- Test nested task spawning

**Why this isn't enough:**
No support for waiting/blocking. Tasks run to completion immediately. Can't handle I/O operations efficiently. No cooperative multitasking—once a task starts, it blocks everything else. We need async for non-blocking operations and yielding control.

---

### Step 2: Add Future Trait and Simple Async Execution

**Goal:** Implement basic `Future` trait and polling.

**What to improve:**
```rust
use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
use std::future::Future;
use std::pin::Pin;

// Simple future that completes immediately
struct ReadyFuture<T> {
    value: Option<T>,
}

impl<T> Future for ReadyFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.value.take().unwrap())
    }
}

// Future that yields once before completing
struct YieldOnceFuture<T> {
    value: Option<T>,
    yielded: bool,
}

impl<T> Future for YieldOnceFuture<T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.yielded {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready(self.value.take().unwrap())
        }
    }
}

// Simple executor
pub struct SimpleExecutor {
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            tasks: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.push(Box::pin(future));
    }

    pub fn run(&mut self) {
        let waker = create_noop_waker();
        let mut context = Context::from_waker(&waker);

        while !self.tasks.is_empty() {
            self.tasks.retain_mut(|task| {
                match task.as_mut().poll(&mut context) {
                    Poll::Ready(()) => false,  // Remove completed
                    Poll::Pending => true,      // Keep pending
                }
            });
        }
    }
}

fn create_noop_waker() -> Waker {
    unsafe fn clone(_: *const ()) -> RawWaker {
        raw_waker()
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}

    fn raw_waker() -> RawWaker {
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
        )
    }

    unsafe { Waker::from_raw(raw_waker()) }
}
```

**Check/Test:**
- Test ReadyFuture completes immediately
- Test YieldOnceFuture yields then completes
- Test executor runs all tasks
- Test multiple concurrent tasks

**Why this isn't enough:**
All futures are heap-allocated with `Box::pin()`. No demonstration of *why* Pin is needed—our futures don't actually have self-references yet. The executor is naive (busy-loops on pending tasks). We need to show the actual problem Pin solves: self-referential futures.

---

### Step 3: Demonstrate Self-Referential Problem and Pin Solution

**Goal:** Show why moving self-referential structs is unsound, then solve with Pin.

**What to improve:**

**1. The problem (won't compile):**
```rust
// This CANNOT be implemented safely!
struct SelfReferential {
    data: String,
    reference: &'??? String,  // Can't reference self.data
}

// Even this doesn't work:
struct Attempted<'a> {
    data: String,
    reference: &'a String,
}

impl<'a> Attempted<'a> {
    fn new(s: String) -> Self {
        Attempted {
            data: s,
            reference: &s,  // ERROR: can't borrow s after moving
        }
    }
}
```

**2. Raw pointer approach (unsafe but shows the issue):**
```rust
struct UnsafeSelfRef {
    data: String,
    data_ptr: *const String,  // Raw pointer to data
}

impl UnsafeSelfRef {
    fn new(s: String) -> Self {
        let mut this = UnsafeSelfRef {
            data: s,
            data_ptr: std::ptr::null(),
        };

        // Set pointer to our own data
        this.data_ptr = &this.data as *const String;
        this
    }

    fn get_ref(&self) -> &str {
        unsafe { &*self.data_ptr }
    }
}

fn demonstrate_problem() {
    let s = UnsafeSelfRef::new(String::from("hello"));
    println!("{}", s.get_ref());  // OK

    // Move the struct!
    let s2 = s;

    // BUG: s2.data_ptr still points to OLD location
    // println!("{}", s2.get_ref());  // Use-after-move!
}
```

**3. Pin solution:**
```rust
use std::marker::PhantomPinned;
use std::pin::Pin;

struct PinnedSelfRef {
    data: String,
    data_ptr: *const String,
    _pin: PhantomPinned,  // Opts out of Unpin
}

impl PinnedSelfRef {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(PinnedSelfRef {
            data: s,
            data_ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // Safe: boxed is pinned, won't move
        let data_ptr: *const String = &boxed.data;

        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).data_ptr = data_ptr;
        }

        boxed
    }

    fn get_ref(self: Pin<&Self>) -> &str {
        // Safe: we're pinned, pointer is valid
        unsafe { &*self.data_ptr }
    }
}

fn demonstrate_solution() {
    let pinned = PinnedSelfRef::new(String::from("hello"));
    println!("{}", pinned.as_ref().get_ref());

    // Cannot move out of Pin!
    // let moved = *pinned;  // ERROR: cannot move out of Pin
}
```

**4. Future that's actually self-referential:**
```rust
struct JoinFuture<F1, F2>
where
    F1: Future,
    F2: Future,
{
    future1: F1,
    future2: F2,
    // Store reference to future1's output across polls
    future1_output: Option<F1::Output>,
    _pin: PhantomPinned,
}

impl<F1, F2> Future for JoinFuture<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is simplified - real version needs unsafe projection
        todo!("Requires pin projection")
    }
}
```

**Check/Test:**
- Test UnsafeSelfRef shows the bug
- Test PinnedSelfRef prevents moving
- Verify PhantomPinned makes type !Unpin
- Test cannot extract value from Pin

**Why this isn't enough:**
We've shown Pin solves self-references, but accessing fields of pinned structs is cumbersome. We used unsafe everywhere. Real futures need "pin projection"—safely accessing fields while maintaining pin guarantees. Also, no actual useful executor yet—just demonstrations.

---

### Step 4: Implement Pin Projection and Useful Futures

**Goal:** Use pin-project for safe field access and build useful combinators.

**What to improve:**

**1. Pin projection (using pin-project crate):**
```rust
use pin_project::pin_project;

#[pin_project]
struct Join<F1, F2> {
    #[pin]
    future1: F1,
    #[pin]
    future2: F2,
    state: JoinState,
}

enum JoinState {
    BothPending,
    FirstComplete,
    SecondComplete,
}

impl<F1, F2> Future for Join<F1, F2>
where
    F1: Future,
    F2: Future,
{
    type Output = (F1::Output, F2::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();  // Pin projection magic

        match this.state {
            JoinState::BothPending => {
                match this.future1.poll(cx) {
                    Poll::Ready(output1) => {
                        *this.state = JoinState::FirstComplete;
                        // Store output1 somehow...
                    }
                    Poll::Pending => {}
                }

                match this.future2.poll(cx) {
                    Poll::Ready(output2) => {
                        *this.state = JoinState::SecondComplete;
                        // Store output2 somehow...
                    }
                    Poll::Pending => {}
                }

                Poll::Pending
            }
            _ => todo!(),
        }
    }
}
```

**2. Useful future combinators:**
```rust
// Map combinator
#[pin_project]
struct Map<F, G> {
    #[pin]
    future: F,
    mapper: Option<G>,
}

impl<F, G, T> Future for Map<F, G>
where
    F: Future,
    G: FnOnce(F::Output) -> T,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.future.poll(cx) {
            Poll::Ready(output) => {
                let mapper = this.mapper.take().unwrap();
                Poll::Ready(mapper(output))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// Then combinator (sequential composition)
#[pin_project(project = ThenProj)]
enum Then<F1, F2> {
    First {
        #[pin]
        future1: F1,
        future2_factory: Option<F2Factory>,
    },
    Second {
        #[pin]
        future2: F2,
    },
}

// AndThen combinator
// Select combinator (first to complete)
// Timeout combinator
```

**3. Stack pinning (no allocation):**
```rust
use std::pin::pin;

fn stack_pin_example() {
    let future = async {
        println!("Hello from future!");
    };

    // Pin on stack (Rust 1.68+)
    let mut pinned = pin!(future);

    let waker = create_noop_waker();
    let mut context = Context::from_waker(&waker);

    match pinned.as_mut().poll(&mut context) {
        Poll::Ready(()) => println!("Done!"),
        Poll::Pending => println!("Not yet"),
    }
}
```

**Check/Test:**
- Test pin projection allows safe field access
- Test map combinator transforms outputs
- Test then combinator sequences futures
- Test stack pinning works without allocation
- Benchmark stack-pinned vs heap-pinned

**Why this isn't enough:**
Combinators are useful but our executor is still naive. A real executor needs:
- Wake mechanism (don't poll unless notified)
- Task queue (fair scheduling)
- Thread pool (parallel execution)
- Timers and I/O readiness

Let's build a proper executor.

---

### Step 5: Build Executor with Wake Support and Task Queue

**Goal:** Implement a real executor with proper wake notifications.

**What to improve:**

**1. Task structure with waker:**
```rust
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::task::{Context, Poll, Waker};

struct Task {
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    executor: Arc<Executor>,
}

impl Task {
    fn poll(self: &Arc<Self>) {
        let waker = self.create_waker();
        let mut context = Context::from_waker(&waker);

        let mut future = self.future.lock().unwrap();

        match future.as_mut().poll(&mut context) {
            Poll::Ready(()) => {
                // Task complete
            }
            Poll::Pending => {
                // Task yielded, will be woken later
            }
        }
    }

    fn create_waker(self: &Arc<Self>) -> Waker {
        // Create waker that reschedules this task
        Arc::clone(self).into_waker()
    }
}

// Implement Waker interface
fn task_into_waker(task: Arc<Task>) -> Waker {
    unsafe fn clone_raw(ptr: *const ()) -> RawWaker {
        let task = Arc::from_raw(ptr as *const Task);
        let cloned = Arc::clone(&task);
        std::mem::forget(task);
        RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
    }

    unsafe fn wake_raw(ptr: *const ()) {
        let task = Arc::from_raw(ptr as *const Task);
        task.executor.schedule(Arc::clone(&task));
    }

    unsafe fn wake_by_ref_raw(ptr: *const ()) {
        let task = Arc::from_raw(ptr as *const Task);
        task.executor.schedule(Arc::clone(&task));
        std::mem::forget(task);
    }

    unsafe fn drop_raw(ptr: *const ()) {
        drop(Arc::from_raw(ptr as *const Task));
    }

    static VTABLE: RawWakerVTable = RawWakerVTable::new(
        clone_raw,
        wake_raw,
        wake_by_ref_raw,
        drop_raw,
    );

    unsafe {
        Waker::from_raw(RawWaker::new(
            Arc::into_raw(task) as *const (),
            &VTABLE,
        ))
    }
}
```

**2. Executor with task queue:**
```rust
pub struct Executor {
    queue: Mutex<VecDeque<Arc<Task>>>,
}

impl Executor {
    pub fn new() -> Arc<Self> {
        Arc::new(Executor {
            queue: Mutex::new(VecDeque::new()),
        })
    }

    pub fn spawn<F>(self: &Arc<Self>, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: Arc::clone(self),
        });

        self.schedule(task);
    }

    fn schedule(&self, task: Arc<Task>) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(task);
    }

    pub fn run(&self) {
        loop {
            let task = {
                let mut queue = self.queue.lock().unwrap();
                queue.pop_front()
            };

            match task {
                Some(task) => task.poll(),
                None => break,  // No more tasks
            }
        }
    }
}
```

**3. Timer future (demonstrates wake mechanism):**
```rust
use std::time::{Duration, Instant};
use std::thread;

struct TimerFuture {
    deadline: Instant,
    waker_sent: bool,
}

impl TimerFuture {
    fn new(duration: Duration) -> Self {
        TimerFuture {
            deadline: Instant::now() + duration,
            waker_sent: false,
        }
    }
}

impl Future for TimerFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.deadline {
            return Poll::Ready(());
        }

        if !self.waker_sent {
            let waker = cx.waker().clone();
            let deadline = self.deadline;

            // Spawn thread to wake us
            thread::spawn(move || {
                let now = Instant::now();
                if now < deadline {
                    thread::sleep(deadline - now);
                }
                waker.wake();
            });

            self.waker_sent = true;
        }

        Poll::Pending
    }
}
```

**Usage:**
```rust
let executor = Executor::new();

executor.spawn(async {
    println!("Starting task 1");
    TimerFuture::new(Duration::from_secs(1)).await;
    println!("Task 1 after 1 second");
});

executor.spawn(async {
    println!("Starting task 2");
    TimerFuture::new(Duration::from_millis(500)).await;
    println!("Task 2 after 500ms");
});

executor.run();
```

**Check/Test:**
- Test tasks wake correctly after timer
- Test multiple concurrent tasks
- Test wake from different threads
- Verify fair scheduling (all tasks make progress)
- Test executor stops when no tasks remain

**Why this isn't enough:**
Single-threaded executor is a bottleneck. Need work-stealing thread pool for parallelism. Also no I/O support (files, network). Real executors integrate with OS event loops (epoll, kqueue, IOCP). Let's add multi-threading and I/O.

---

### Step 6: Add Multi-Threading and Higher-Ranked Trait Bounds

**Goal:** Implement work-stealing thread pool and demonstrate HRTB with async closures.

**What to improve:**

**1. Work-stealing executor:**
```rust
use crossbeam::deque::{Injector, Stealer, Worker};
use std::thread;

pub struct WorkStealingExecutor {
    global_queue: Arc<Injector<Arc<Task>>>,
    stealers: Vec<Stealer<Arc<Task>>>,
    workers: Vec<thread::JoinHandle<()>>,
}

impl WorkStealingExecutor {
    pub fn new(num_threads: usize) -> Arc<Self> {
        let global_queue = Arc::new(Injector::new());
        let mut local_queues = Vec::new();
        let mut stealers = Vec::new();

        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            local_queues.push(worker);
        }

        let executor = Arc::new(WorkStealingExecutor {
            global_queue,
            stealers,
            workers: Vec::new(),
        });

        // Start worker threads
        // ...

        executor
    }

    fn worker_loop(
        local: Worker<Arc<Task>>,
        global: Arc<Injector<Arc<Task>>>,
        stealers: Vec<Stealer<Arc<Task>>>,
    ) {
        loop {
            // Try local queue first
            let task = local.pop()
                .or_else(|| {
                    // Try stealing from global
                    global.steal_batch_and_pop(&local).success()
                })
                .or_else(|| {
                    // Try stealing from other workers
                    stealers.iter()
                        .map(|s| s.steal())
                        .find_map(|s| s.success())
                });

            match task {
                Some(task) => task.poll(),
                None => {
                    thread::yield_now();
                }
            }
        }
    }
}
```

**2. Higher-ranked trait bounds with async:**
```rust
// HRTB: closure that works with any lifetime
pub fn with_async_context<F, Fut>(f: F)
where
    F: for<'a> FnOnce(&'a Context) -> Fut,
    Fut: Future<Output = ()>,
{
    // f can be called with Context of any lifetime
}

// Async function that accepts HRTB closure
pub async fn process_items<F, Fut>(items: Vec<String>, processor: F)
where
    F: for<'a> Fn(&'a str) -> Fut,
    Fut: Future<Output = ()>,
{
    for item in &items {
        processor(item).await;
    }
}

// Usage
async {
    let items = vec!["a".to_string(), "b".to_string()];

    process_items(items, |s| async move {
        println!("Processing: {}", s);
        TimerFuture::new(Duration::from_millis(100)).await;
    }).await;
}
```

**3. Async channel for communication:**
```rust
use std::sync::mpsc::{channel, Sender, Receiver};

struct AsyncChannel<T> {
    sender: Sender<T>,
    receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> AsyncChannel<T> {
    fn new() -> Self {
        let (sender, receiver) = channel();
        AsyncChannel {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    fn send(&self, value: T) -> Result<(), T> {
        self.sender.send(value).map_err(|e| e.0)
    }

    async fn recv(&self) -> Option<T> {
        // Poll receiver, park if empty, wake when data arrives
        todo!()
    }
}
```

**4. Complete example:**
```rust
#[tokio::main]  // Or our custom executor
async fn main() {
    let executor = WorkStealingExecutor::new(4);

    executor.spawn(async {
        println!("Parallel task 1");
        TimerFuture::new(Duration::from_secs(1)).await;
        println!("Task 1 done");
    });

    executor.spawn(async {
        println!("Parallel task 2");
        TimerFuture::new(Duration::from_secs(1)).await;
        println!("Task 2 done");
    });

    // With HRTB
    let items = vec!["item1".to_string(), "item2".to_string()];
    executor.spawn(async move {
        process_items(items, |item| async move {
            println!("Processing: {}", item);
        }).await;
    });

    executor.run();
}
```

**Check/Test:**
- Test work-stealing load balances across threads
- Test HRTB closures with various lifetimes
- Test concurrent task execution
- Benchmark: multi-threaded vs single-threaded
- Test channel communication between tasks
- Verify Pin safety maintained across threads

**What this achieves:**
A production-ready async executor:
- **Pin-Safe**: Properly handles self-referential futures
- **Multi-Threaded**: Work-stealing for parallelism
- **Wake Mechanism**: Efficient task scheduling
- **HRTB Support**: Flexible async closures
- **Zero-Cost**: Pin is compile-time only
- **Practical**: Can run real async workloads

**Extensions to explore:**
- I/O integration (async file/network)
- Task priorities and deadlines
- Async cancellation/timeout
- Async streams (async iterators)
- Integration with existing runtimes (Tokio, async-std)

---
