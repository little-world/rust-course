//! Advanced Atomic Patterns
//!
//! Run with: cargo run --example advanced_patterns

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// Striped counter (reduce contention)
struct StripedCounter {
    stripes: Vec<AtomicUsize>,
}

impl StripedCounter {
    fn new(num_stripes: usize) -> Self {
        let mut stripes = Vec::new();
        for _ in 0..num_stripes {
            stripes.push(AtomicUsize::new(0));
        }

        Self { stripes }
    }

    fn increment(&self) {
        let thread_id = std::thread::current().id();
        let index = format!("{:?}", thread_id).len() % self.stripes.len();
        self.stripes[index].fetch_add(1, Ordering::Relaxed);
    }

    fn get(&self) -> usize {
        self.stripes
            .iter()
            .map(|s| s.load(Ordering::Relaxed))
            .sum()
    }
}

// Exponential backoff
struct Backoff {
    current: Duration,
    max: Duration,
}

impl Backoff {
    fn new() -> Self {
        Self {
            current: Duration::from_nanos(1),
            max: Duration::from_micros(1000),
        }
    }

    fn spin(&mut self) {
        for _ in 0..(self.current.as_nanos() / 10) {
            std::hint::spin_loop();
        }

        self.current = (self.current * 2).min(self.max);
    }

    fn reset(&mut self) {
        self.current = Duration::from_nanos(1);
    }
}

fn cas_with_backoff(counter: &AtomicUsize) {
    let mut backoff = Backoff::new();

    loop {
        let current = counter.load(Ordering::Relaxed);

        match counter.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                backoff.reset();
                break;
            }
            Err(_) => {
                backoff.spin();
            }
        }
    }
}

// Atomic min/max
struct AtomicMinMax {
    min: AtomicU64,
    max: AtomicU64,
}

impl AtomicMinMax {
    fn new() -> Self {
        Self {
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    fn update(&self, value: u64) {
        // Update min
        let mut current_min = self.min.load(Ordering::Relaxed);
        while value < current_min {
            match self.min.compare_exchange_weak(
                current_min,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }

        // Update max
        let mut current_max = self.max.load(Ordering::Relaxed);
        while value > current_max {
            match self.max.compare_exchange_weak(
                current_max,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    fn get(&self) -> (u64, u64) {
        (
            self.min.load(Ordering::Relaxed),
            self.max.load(Ordering::Relaxed),
        )
    }
}

// Once flag for initialization
const INCOMPLETE: usize = 0;
const RUNNING: usize = 1;
const COMPLETE: usize = 2;

struct OnceFlag {
    state: AtomicUsize,
}

impl OnceFlag {
    fn new() -> Self {
        Self {
            state: AtomicUsize::new(INCOMPLETE),
        }
    }

    fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.state.load(Ordering::Acquire) == COMPLETE {
            return;
        }

        match self.state.compare_exchange(
            INCOMPLETE,
            RUNNING,
            Ordering::Acquire,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                // We won the race
                f();
                self.state.store(COMPLETE, Ordering::Release);
            }
            Err(RUNNING) => {
                // Someone else is running, wait
                while self.state.load(Ordering::Acquire) == RUNNING {
                    std::hint::spin_loop();
                }
            }
            Err(COMPLETE) => {
                // Already done
            }
            _ => unreachable!(),
        }
    }

    fn is_completed(&self) -> bool {
        self.state.load(Ordering::Acquire) == COMPLETE
    }
}

// Atomic swap chain
struct SwapChain<T> {
    value: AtomicUsize, // Actually *mut T
    _phantom: std::marker::PhantomData<T>,
}

impl<T> SwapChain<T> {
    fn new(initial: T) -> Self {
        let ptr = Box::into_raw(Box::new(initial));
        Self {
            value: AtomicUsize::new(ptr as usize),
            _phantom: std::marker::PhantomData,
        }
    }

    fn swap(&self, new_value: T) -> T {
        let new_ptr = Box::into_raw(Box::new(new_value));
        let old_ptr = self.value.swap(new_ptr as usize, Ordering::AcqRel) as *mut T;

        unsafe {
            let old_value = std::ptr::read(old_ptr);
            drop(Box::from_raw(old_ptr));
            old_value
        }
    }

    fn load(&self) -> T
    where
        T: Clone,
    {
        let ptr = self.value.load(Ordering::Acquire) as *mut T;
        unsafe { (*ptr).clone() }
    }
}

impl<T> Drop for SwapChain<T> {
    fn drop(&mut self) {
        let ptr = self.value.load(Ordering::Acquire) as *mut T;
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }
}

fn main() {
    println!("=== Striped Counter ===\n");

    let counter = Arc::new(StripedCounter::new(16));
    let mut handles = vec![];

    let start = Instant::now();

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100_000 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Count: {} in {:?}", counter.get(), start.elapsed());

    println!("\n=== Backoff ===\n");

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..10_000 {
                cas_with_backoff(&counter);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Count: {}", counter.load(Ordering::Relaxed));

    println!("\n=== Atomic Min/Max ===\n");

    let minmax = Arc::new(AtomicMinMax::new());
    let mut handles = vec![];

    for i in 0..10 {
        let minmax = Arc::clone(&minmax);
        handles.push(thread::spawn(move || {
            for j in 0..1000 {
                minmax.update(i * 1000 + j);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (min, max) = minmax.get();
    println!("Min: {}, Max: {}", min, max);

    println!("\n=== Once Flag ===\n");

    let once = Arc::new(OnceFlag::new());
    let mut handles = vec![];

    for i in 0..10 {
        let once = Arc::clone(&once);
        handles.push(thread::spawn(move || {
            once.call_once(|| {
                println!("Initialization by thread {}", i);
            });
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Completed: {}", once.is_completed());

    println!("\n=== Swap Chain ===\n");

    let chain = SwapChain::new("initial".to_string());
    println!("Initial: {}", chain.load());

    let old = chain.swap("updated".to_string());
    println!("Old: {}, New: {}", old, chain.load());
}
