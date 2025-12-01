### Atomic Cheat Sheet
```rust
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
```
