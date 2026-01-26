//! Pattern 4: Hazard Pointers
//!
//! Run with: cargo run --example p4_hazard_pointers

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

const MAX_HAZARDS: usize = 128;

struct HazardPointer {
    pointer: AtomicPtr<u8>,
}

impl HazardPointer {
    fn new() -> Self {
        Self {
            pointer: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn protect(&self, ptr: *mut u8) {
        self.pointer.store(ptr, Ordering::Release);
    }

    fn clear(&self) {
        self.pointer.store(ptr::null_mut(), Ordering::Release);
    }

    fn get(&self) -> *mut u8 {
        self.pointer.load(Ordering::Acquire)
    }
}

struct HazardPointerDomain {
    hazards: Vec<HazardPointer>,
    retired: AtomicPtr<RetiredNode>,
    retired_count: AtomicUsize,
}

struct RetiredNode {
    ptr: *mut u8,
    next: *mut RetiredNode,
    deleter: unsafe fn(*mut u8),
}

impl HazardPointerDomain {
    fn new() -> Self {
        let mut hazards = Vec::new();
        for _ in 0..MAX_HAZARDS {
            hazards.push(HazardPointer::new());
        }

        Self {
            hazards,
            retired: AtomicPtr::new(ptr::null_mut()),
            retired_count: AtomicUsize::new(0),
        }
    }

    fn acquire(&self) -> Option<usize> {
        for (i, hp) in self.hazards.iter().enumerate() {
            let current = hp.get();
            if current.is_null() {
                // Try to claim this hazard pointer
                if hp
                    .pointer
                    .compare_exchange(
                        ptr::null_mut(),
                        1 as *mut u8, // Non-null marker
                        Ordering::Acquire,
                        Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    return Some(i);
                }
            }
        }
        None
    }

    fn protect(&self, index: usize, ptr: *mut u8) {
        self.hazards[index].protect(ptr);
    }

    fn release(&self, index: usize) {
        self.hazards[index].clear();
    }

    fn retire(&self, ptr: *mut u8, deleter: unsafe fn(*mut u8)) {
        let node = Box::into_raw(Box::new(RetiredNode {
            ptr,
            next: ptr::null_mut(),
            deleter,
        }));

        // Add to retired list
        loop {
            let head = self.retired.load(Ordering::Acquire);
            unsafe {
                (*node).next = head;
            }

            if self
                .retired
                .compare_exchange_weak(head, node, Ordering::Release, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }

        let count = self.retired_count.fetch_add(1, Ordering::Relaxed);

        // Trigger reclamation if too many retired
        if count > MAX_HAZARDS * 2 {
            self.scan();
        }
    }

    fn scan(&self) {
        // Collect all protected pointers
        let mut protected = HashSet::new();
        for hp in &self.hazards {
            let ptr = hp.get();
            if !ptr.is_null() && ptr != 1 as *mut u8 {
                protected.insert(ptr);
            }
        }

        // Try to reclaim retired nodes
        let mut current = self.retired.swap(ptr::null_mut(), Ordering::Acquire);
        let mut kept = Vec::new();

        unsafe {
            while !current.is_null() {
                let next = (*current).next;

                if protected.contains(&(*current).ptr) {
                    // Still protected, keep it
                    kept.push(current);
                } else {
                    // Safe to delete
                    ((*current).deleter)((*current).ptr);
                    drop(Box::from_raw(current));
                    self.retired_count.fetch_sub(1, Ordering::Relaxed);
                }

                current = next;
            }
        }

        // Re-add kept nodes
        for node in kept {
            loop {
                let head = self.retired.load(Ordering::Acquire);
                unsafe {
                    (*node).next = head;
                }

                if self
                    .retired
                    .compare_exchange_weak(head, node, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    break;
                }
            }
        }
    }
}

// Stack with hazard pointers
struct SafeNode<T> {
    data: T,
    next: *mut SafeNode<T>,
}

struct SafeStack<T> {
    head: AtomicPtr<SafeNode<T>>,
    hp_domain: HazardPointerDomain,
}

impl<T> SafeStack<T> {
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
            hp_domain: HazardPointerDomain::new(),
        }
    }

    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(SafeNode {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::Relaxed);
            unsafe {
                (*new_node).next = head;
            }

            if self
                .head
                .compare_exchange_weak(head, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    fn pop(&self) -> Option<T> {
        let hp_index = self.hp_domain.acquire()?;

        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                self.hp_domain.release(hp_index);
                return None;
            }

            // Protect head from deletion
            self.hp_domain.protect(hp_index, head as *mut u8);

            // Verify head hasn't changed (avoid ABA)
            if self.head.load(Ordering::Acquire) != head {
                continue;
            }

            unsafe {
                let next = (*head).next;

                if self
                    .head
                    .compare_exchange_weak(head, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*head).data);

                    // Retire the node for later deletion
                    self.hp_domain.retire(head as *mut u8, |ptr| {
                        drop(Box::from_raw(ptr as *mut SafeNode<T>));
                    });

                    self.hp_domain.release(hp_index);
                    return Some(data);
                }
            }
        }
    }
}

unsafe impl<T: Send> Send for SafeStack<T> {}
unsafe impl<T: Send> Sync for SafeStack<T> {}

fn main() {
    println!("=== Safe Stack with Hazard Pointers ===\n");

    let stack = Arc::new(SafeStack::new());
    let mut handles = vec![];

    // Producers
    for i in 0..5 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for j in 0..1000 {
                stack.push(i * 1000 + j);
            }
        }));
    }

    // Consumers
    for _ in 0..5 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            let mut count = 0;
            while stack.pop().is_some() {
                count += 1;
            }
            let _ = count; // Use count to suppress warning
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Stack operations completed safely");
}
