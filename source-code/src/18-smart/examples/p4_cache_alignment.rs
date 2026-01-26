// Pattern 4: Memory Layout Optimization - Cache Line Alignment
use std::sync::atomic::{AtomicUsize, Ordering};

#[allow(dead_code)]
const CACHE_LINE: usize = 64;

#[repr(align(64))]
struct Padded<T> {
    value: T,
}

struct Counters {
    counter1: Padded<AtomicUsize>,  // Own cache line
    counter2: Padded<AtomicUsize>,  // Own cache line
}

fn main() {
    // Usage: Threads can update counters without false sharing
    let counters = Counters {
        counter1: Padded { value: AtomicUsize::new(0) },
        counter2: Padded { value: AtomicUsize::new(0) },
    };

    // Thread 1 updates counter1, Thread 2 updates counter2
    // No cache line bouncing between cores
    counters.counter1.value.fetch_add(1, Ordering::Relaxed);
    counters.counter2.value.fetch_add(1, Ordering::Relaxed);

    println!("Counter1: {}", counters.counter1.value.load(Ordering::Relaxed));
    println!("Counter2: {}", counters.counter2.value.load(Ordering::Relaxed));
    println!("Cache alignment example completed");
}
