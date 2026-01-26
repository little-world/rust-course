// Pattern 5: Building Safe APIs with Unsafe Internals
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// Uses atomic operations and unsafe cell internally,
/// but provides safe locking through RAII guards.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub fn new(data: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the lock, blocking until available.
    /// Returns a guard that provides access to the data
    /// and releases the lock when dropped.
    pub fn lock(&self) -> SpinLockGuard<T> {
        while self.locked.swap(true, Ordering::Acquire) {
            std::hint::spin_loop();
        }
        SpinLockGuard { lock: self }
    }
}

impl<'a, T> std::ops::Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> std::ops::DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

// SAFETY: SpinLock properly synchronizes access to T
// The Acquire/Release ordering ensures memory visibility
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

fn main() {
    // Usage: RAII guard auto-unlocks
    let lock = SpinLock::new(0);

    {
        let mut guard = lock.lock();
        *guard += 1;
        println!("Value after increment: {}", *guard);
    } // guard drops here, releasing the lock

    // Lock again
    {
        let guard = lock.lock();
        println!("Value: {}", *guard);
    }

    // Multi-threaded example
    use std::sync::Arc;
    use std::thread;

    let counter = Arc::new(SpinLock::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..100 {
                *counter_clone.lock() += 1;
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock());
    assert_eq!(*counter.lock(), 1000);

    println!("SpinLock example completed");
}
