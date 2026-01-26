//! Pattern 1: Memory Ordering Semantics
//!
//! Run with: cargo run --example p1_memory_ordering

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering, fence};
use std::thread;

// Example 1: Relaxed - No ordering guarantees (fastest)
fn relaxed_ordering_example() {
    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                // Relaxed: no synchronization, just atomicity
                counter.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Final value is guaranteed to be correct
    // but intermediate values may have appeared in any order
    println!("Counter (Relaxed): {}", counter.load(Ordering::Relaxed));
}

// Example 2: Acquire/Release - Synchronization without sequential consistency
fn acquire_release_ordering() {
    let data = Arc::new(AtomicUsize::new(0));
    let ready = Arc::new(AtomicBool::new(false));

    let data_clone = Arc::clone(&data);
    let ready_clone = Arc::clone(&ready);

    // Producer
    let producer = thread::spawn(move || {
        // Write data
        data_clone.store(42, Ordering::Relaxed);

        // Release: all previous writes visible to thread that Acquires
        ready_clone.store(true, Ordering::Release);
    });

    let data_consumer = Arc::clone(&data);
    let ready_consumer = Arc::clone(&ready);

    // Consumer
    let consumer = thread::spawn(move || {
        // Acquire: see all writes before the Release
        while !ready_consumer.load(Ordering::Acquire) {
            thread::yield_now();
        }

        // Guaranteed to see data == 42
        let value = data_consumer.load(Ordering::Relaxed);
        println!("Consumer sees: {}", value);
        assert_eq!(value, 42);
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

// Example 3: SeqCst - Sequential consistency (slowest, easiest to reason about)
fn seq_cst_ordering() {
    let x = Arc::new(AtomicBool::new(false));
    let y = Arc::new(AtomicBool::new(false));
    let z1 = Arc::new(AtomicBool::new(false));
    let z2 = Arc::new(AtomicBool::new(false));

    let x1 = Arc::clone(&x);
    let y1 = Arc::clone(&y);
    let z1_clone = Arc::clone(&z1);

    let t1 = thread::spawn(move || {
        x1.store(true, Ordering::SeqCst);
        if !y1.load(Ordering::SeqCst) {
            z1_clone.store(true, Ordering::SeqCst);
        }
    });

    let x2 = Arc::clone(&x);
    let y2 = Arc::clone(&y);
    let z2_clone = Arc::clone(&z2);

    let t2 = thread::spawn(move || {
        y2.store(true, Ordering::SeqCst);
        if !x2.load(Ordering::SeqCst) {
            z2_clone.store(true, Ordering::SeqCst);
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();

    // With SeqCst: cannot have both z1 and z2 true
    // Without SeqCst: theoretically possible (hardware reordering)
    let both = z1.load(Ordering::SeqCst) && z2.load(Ordering::SeqCst);
    println!("Both flags set: {} (should be false with SeqCst)", both);
}

// Example 4: AcqRel - Combine Acquire and Release
fn acq_rel_ordering() {
    let counter = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];
    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                // AcqRel: Acts as Acquire for load, Release for store
                counter.fetch_add(1, Ordering::AcqRel);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Counter (AcqRel): {}", counter.load(Ordering::Acquire));
}

// Example 5: Spinlock with proper ordering
struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }

    fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(
                false,
                true,
                Ordering::Acquire, // Success: acquire lock
                Ordering::Relaxed, // Failure: just retry
            )
            .is_err()
        {
            // Hint to CPU we're spinning
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
    }

    fn unlock(&self) {
        // Release: make all previous writes visible
        self.locked.store(false, Ordering::Release);
    }
}

// Example 6: Double-checked locking for lazy initialization
struct LazyInit<T> {
    data: AtomicUsize, // Actually *mut T
    initialized: AtomicBool,
    _marker: std::marker::PhantomData<T>,
}

impl<T> LazyInit<T> {
    fn new() -> Self {
        Self {
            data: AtomicUsize::new(0),
            initialized: AtomicBool::new(false),
            _marker: std::marker::PhantomData,
        }
    }

    fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        // Fast path: already initialized (Acquire ensures we see the data)
        if self.initialized.load(Ordering::Acquire) {
            unsafe { &*(self.data.load(Ordering::Relaxed) as *const T) }
        } else {
            self.init_slow(init)
        }
    }

    fn init_slow<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let ptr = Box::into_raw(Box::new(init()));

        // Try to publish (use SeqCst for correctness)
        match self.initialized.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => {
                // We won the race
                self.data.store(ptr as usize, Ordering::Release);
                unsafe { &*ptr }
            }
            Err(_) => {
                // Someone else won, clean up our allocation
                unsafe { drop(Box::from_raw(ptr)) };
                unsafe { &*(self.data.load(Ordering::Acquire) as *const T) }
            }
        }
    }
}

// Example 7: Fence for non-atomic data
fn fence_example() {
    let data = Arc::new(AtomicUsize::new(0));
    let ready = Arc::new(AtomicBool::new(false));

    let data_clone = Arc::clone(&data);
    let ready_clone = Arc::clone(&ready);

    // Producer
    let producer = thread::spawn(move || {
        // Write data
        data_clone.store(42, Ordering::Relaxed);

        // Fence ensures all previous writes are visible
        fence(Ordering::Release);

        // Signal ready
        ready_clone.store(true, Ordering::Relaxed);
    });

    // Consumer
    thread::sleep(std::time::Duration::from_millis(10));

    if ready.load(Ordering::Relaxed) {
        // Fence ensures we see all writes before the Release fence
        fence(Ordering::Acquire);

        // Now safe to read data
        println!("Data via fence: {}", data.load(Ordering::Relaxed));
    }

    producer.join().unwrap();
}

// Example 8: Compiler fence (prevents compiler reordering only)
fn compiler_fence_example() {
    let x = AtomicUsize::new(0);
    let y = AtomicUsize::new(0);

    x.store(1, Ordering::Relaxed);

    // Prevent compiler from reordering (hardware can still reorder)
    std::sync::atomic::compiler_fence(Ordering::SeqCst);

    y.store(2, Ordering::Relaxed);

    println!("Compiler fence: x={}, y={}",
             x.load(Ordering::Relaxed),
             y.load(Ordering::Relaxed));
}

// Example 9: DMA buffer (conceptual)
#[repr(C)]
struct DmaBuffer {
    data: [u8; 4096],
    ready: AtomicBool,
}

impl DmaBuffer {
    fn new() -> Self {
        Self {
            data: [0u8; 4096],
            ready: AtomicBool::new(false),
        }
    }

    fn write_for_dma(&mut self, src: &[u8]) {
        let len = src.len().min(self.data.len());
        self.data[..len].copy_from_slice(&src[..len]);

        // Ensure all writes complete before signaling device
        fence(Ordering::Release);

        self.ready.store(true, Ordering::Relaxed);
    }

    fn read_from_dma(&self) -> Option<&[u8]> {
        if !self.ready.load(Ordering::Relaxed) {
            return None;
        }

        // Ensure we see all device writes
        fence(Ordering::Acquire);

        Some(&self.data)
    }
}

fn main() {
    println!("=== Relaxed Ordering ===\n");
    relaxed_ordering_example();

    println!("\n=== Acquire/Release Ordering ===\n");
    acquire_release_ordering();

    println!("\n=== SeqCst Ordering ===\n");
    seq_cst_ordering();

    println!("\n=== AcqRel Ordering ===\n");
    acq_rel_ordering();

    println!("\n=== Spinlock ===\n");
    let lock = Spinlock::new();
    lock.lock();
    println!("Lock acquired");
    lock.unlock();
    println!("Lock released");

    println!("\n=== Lazy Init ===\n");
    let lazy: LazyInit<String> = LazyInit::new();
    let value = lazy.get_or_init(|| {
        println!("Initializing...");
        "Hello, World!".to_string()
    });
    println!("Lazy value: {}", value);

    println!("\n=== Fence ===\n");
    fence_example();

    println!("\n=== Compiler Fence ===\n");
    compiler_fence_example();

    println!("\n=== DMA Buffer ===\n");
    let mut dma = DmaBuffer::new();
    dma.write_for_dma(b"Hello DMA");
    if let Some(data) = dma.read_from_dma() {
        println!("DMA data: {:?}", &data[..9]);
    }
}
