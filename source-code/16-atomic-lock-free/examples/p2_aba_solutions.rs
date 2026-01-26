//! Pattern 2: ABA Problem and Solutions
//!
//! Run with: cargo run --example p2_aba_solutions

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::thread;

// Solution 1: Tagged pointers (version counter)
struct TaggedPtr {
    value: AtomicU64, // Upper 16 bits: tag, Lower 48 bits: pointer
}

impl TaggedPtr {
    fn new(ptr: *mut u8) -> Self {
        Self {
            value: AtomicU64::new(ptr as u64),
        }
    }

    fn load(&self, ordering: Ordering) -> (*mut u8, u16) {
        let packed = self.value.load(ordering);
        let ptr = (packed & 0x0000_FFFF_FFFF_FFFF) as *mut u8;
        let tag = (packed >> 48) as u16;
        (ptr, tag)
    }

    fn store(&self, ptr: *mut u8, tag: u16, ordering: Ordering) {
        let packed = ((tag as u64) << 48) | ((ptr as u64) & 0x0000_FFFF_FFFF_FFFF);
        self.value.store(packed, ordering);
    }

    fn compare_exchange(
        &self,
        current_ptr: *mut u8,
        current_tag: u16,
        new_ptr: *mut u8,
        new_tag: u16,
        success: Ordering,
        failure: Ordering,
    ) -> Result<(), ()> {
        let current = ((current_tag as u64) << 48) | ((current_ptr as u64) & 0x0000_FFFF_FFFF_FFFF);
        let new = ((new_tag as u64) << 48) | ((new_ptr as u64) & 0x0000_FFFF_FFFF_FFFF);

        self.value
            .compare_exchange(current, new, success, failure)
            .map(|_| ())
            .map_err(|_| ())
    }
}

// Solution 2: Version counter approach
struct VersionedStack<T> {
    head: AtomicU64, // Upper 32 bits: version, Lower 32 bits: index
    nodes: Vec<Option<VersionedNode<T>>>,
}

struct VersionedNode<T> {
    data: T,
    next: u32,
    #[allow(dead_code)]
    version: u32,
}

impl<T> VersionedStack<T> {
    fn pack(index: u32, version: u32) -> u64 {
        ((version as u64) << 32) | (index as u64)
    }

    fn unpack(packed: u64) -> (u32, u32) {
        let index = (packed & 0xFFFF_FFFF) as u32;
        let version = (packed >> 32) as u32;
        (index, version)
    }

    fn new(capacity: usize) -> Self {
        Self {
            head: AtomicU64::new(Self::pack(u32::MAX, 0)), // NULL with version 0
            nodes: (0..capacity).map(|_| None).collect(),
        }
    }

    fn push(&mut self, data: T) -> bool {
        // Find free slot (simplified - real impl would use free list)
        let free_idx = self.nodes.iter().position(|n| n.is_none());
        let idx = match free_idx {
            Some(i) => i as u32,
            None => return false,
        };

        let current = self.head.load(Ordering::Relaxed);
        let (head_idx, version) = Self::unpack(current);

        self.nodes[idx as usize] = Some(VersionedNode {
            data,
            next: head_idx,
            version: version + 1,
        });

        let new_head = Self::pack(idx, version + 1);
        self.head
            .compare_exchange(current, new_head, Ordering::Release, Ordering::Relaxed)
            .is_ok()
    }

    fn peek(&self) -> Option<&T> {
        let current = self.head.load(Ordering::Acquire);
        let (idx, _) = Self::unpack(current);
        if idx == u32::MAX {
            return None;
        }
        self.nodes.get(idx as usize)?.as_ref().map(|n| &n.data)
    }
}

// Solution 3: Epoch-based reclamation (simplified)
struct EpochGC {
    global_epoch: AtomicUsize,
}

impl EpochGC {
    fn new() -> Self {
        Self {
            global_epoch: AtomicUsize::new(0),
        }
    }

    fn pin(&self) -> usize {
        self.global_epoch.load(Ordering::Acquire)
    }

    fn try_advance(&self) {
        self.global_epoch.fetch_add(1, Ordering::Release);
    }

    fn is_safe_to_free(&self, allocation_epoch: usize) -> bool {
        let current = self.global_epoch.load(Ordering::Acquire);
        current > allocation_epoch + 2 // Conservative: 2 epochs old
    }
}

// Real-world: ABA-safe counter
struct ABACounter {
    value: AtomicU64, // Upper 32 bits: version, Lower 32 bits: count
}

impl ABACounter {
    fn new(initial: u32) -> Self {
        Self {
            value: AtomicU64::new(initial as u64),
        }
    }

    fn increment(&self) {
        loop {
            let current = self.value.load(Ordering::Relaxed);
            let count = (current & 0xFFFF_FFFF) as u32;
            let version = (current >> 32) as u32;

            let new_count = count.wrapping_add(1);
            let new_version = version.wrapping_add(1);
            let new_value = ((new_version as u64) << 32) | (new_count as u64);

            if self
                .value
                .compare_exchange_weak(current, new_value, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    fn get(&self) -> u32 {
        let packed = self.value.load(Ordering::Relaxed);
        (packed & 0xFFFF_FFFF) as u32
    }

    fn get_with_version(&self) -> (u32, u32) {
        let packed = self.value.load(Ordering::Relaxed);
        let count = (packed & 0xFFFF_FFFF) as u32;
        let version = (packed >> 32) as u32;
        (count, version)
    }
}

fn main() {
    println!("=== Tagged Pointer ===\n");

    let data = Box::into_raw(Box::new(42u8));
    let tagged = TaggedPtr::new(data);

    let (ptr, tag) = tagged.load(Ordering::Relaxed);
    println!("Initial: ptr={:?}, tag={}", ptr, tag);

    tagged.store(ptr, 1, Ordering::Relaxed);
    let (ptr, tag) = tagged.load(Ordering::Relaxed);
    println!("After store: ptr={:?}, tag={}", ptr, tag);

    // Clean up
    unsafe { drop(Box::from_raw(data)) };

    println!("\n=== Versioned Stack ===\n");

    let mut stack: VersionedStack<i32> = VersionedStack::new(10);
    stack.push(1);
    stack.push(2);
    stack.push(3);

    println!("Top: {:?}", stack.peek());

    println!("\n=== ABA Counter ===\n");

    let counter = Arc::new(ABACounter::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let (count, version) = counter.get_with_version();
    println!("Count: {}, Version: {}", count, version);

    println!("\n=== Epoch GC ===\n");

    let gc = EpochGC::new();

    let epoch1 = gc.pin();
    println!("Pinned at epoch {}", epoch1);

    gc.try_advance();
    gc.try_advance();
    gc.try_advance();

    println!("Safe to free epoch {}? {}", epoch1, gc.is_safe_to_free(epoch1));
}
