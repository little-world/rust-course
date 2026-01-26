//! Pattern 2: Compare-and-Swap Patterns
//!
//! Run with: cargo run --example p2_cas_patterns

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

// Example 1: Basic CAS loop
fn cas_increment(counter: &AtomicUsize) {
    loop {
        let current = counter.load(Ordering::Relaxed);
        let new_value = current + 1;

        // Try to update: succeeds only if value hasn't changed
        if counter
            .compare_exchange_weak(
                current,
                new_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            break;
        }

        // Spurious failure or actual contention - retry
    }
}

// Example 2: compare_exchange vs compare_exchange_weak
fn compare_exchange_variants() {
    let value = AtomicUsize::new(0);

    // compare_exchange: never spurious failure, use in non-loop
    let result = value.compare_exchange(
        0,
        1,
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
    assert!(result.is_ok());

    // compare_exchange_weak: may spuriously fail, use in loop
    loop {
        if value
            .compare_exchange_weak(1, 2, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            break;
        }
    }

    println!("Final value: {}", value.load(Ordering::SeqCst));
}

// Example 3: CAS with data transformation
fn cas_update<F>(counter: &AtomicUsize, f: F)
where
    F: Fn(usize) -> usize,
{
    let mut current = counter.load(Ordering::Relaxed);

    loop {
        let new_value = f(current);

        match counter.compare_exchange_weak(
            current,
            new_value,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(actual) => current = actual, // Update current and retry
        }
    }
}

// Example 4: Lock-free max tracking
struct MaxTracker {
    max: AtomicUsize,
}

impl MaxTracker {
    fn new() -> Self {
        Self {
            max: AtomicUsize::new(0),
        }
    }

    fn update(&self, value: usize) {
        let mut current = self.max.load(Ordering::Relaxed);

        loop {
            if value <= current {
                // Already have a larger max
                break;
            }

            match self.max.compare_exchange_weak(
                current,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }

    fn get(&self) -> usize {
        self.max.load(Ordering::Relaxed)
    }
}

// Example 5: Lock-free accumulator
struct Accumulator {
    sum: AtomicUsize,
    count: AtomicUsize,
}

impl Accumulator {
    fn new() -> Self {
        Self {
            sum: AtomicUsize::new(0),
            count: AtomicUsize::new(0),
        }
    }

    fn add(&self, value: usize) {
        self.sum.fetch_add(value, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    fn average(&self) -> f64 {
        let sum = self.sum.load(Ordering::Relaxed);
        let count = self.count.load(Ordering::Relaxed);

        if count == 0 {
            0.0
        } else {
            sum as f64 / count as f64
        }
    }

    fn reset(&self) -> (usize, usize) {
        let sum = self.sum.swap(0, Ordering::Relaxed);
        let count = self.count.swap(0, Ordering::Relaxed);
        (sum, count)
    }
}

// Example 6: Conditional update
struct ConditionalCounter {
    value: AtomicUsize,
}

impl ConditionalCounter {
    fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
        }
    }

    fn increment_if_below(&self, threshold: usize) -> bool {
        let mut current = self.value.load(Ordering::Relaxed);

        loop {
            if current >= threshold {
                return false; // Can't increment
            }

            match self.value.compare_exchange_weak(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    fn get(&self) -> usize {
        self.value.load(Ordering::Relaxed)
    }
}

fn main() {
    println!("=== CAS Increment ===\n");

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                cas_increment(&counter);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.load(Ordering::Relaxed));

    println!("\n=== Compare Exchange Variants ===\n");
    compare_exchange_variants();

    println!("\n=== CAS Update ===\n");
    let counter = AtomicUsize::new(10);
    cas_update(&counter, |x| x * 2);
    println!("After doubling: {}", counter.load(Ordering::Relaxed));

    println!("\n=== Max Tracker ===\n");

    let tracker = Arc::new(MaxTracker::new());
    let mut handles = vec![];

    for i in 0..10 {
        let tracker = Arc::clone(&tracker);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                tracker.update(i * 100 + j);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Max value: {}", tracker.get());

    println!("\n=== Accumulator ===\n");

    let acc = Accumulator::new();

    for i in 1..=100 {
        acc.add(i);
    }

    println!("Average: {:.2}", acc.average());
    println!("Reset: {:?}", acc.reset());
    println!("After reset: {:.2}", acc.average());

    println!("\n=== Conditional Counter ===\n");

    let counter = ConditionalCounter::new(0);

    for _ in 0..15 {
        if counter.increment_if_below(10) {
            println!("Incremented to {}", counter.get());
        } else {
            println!("Threshold reached: {}", counter.get());
        }
    }
}
