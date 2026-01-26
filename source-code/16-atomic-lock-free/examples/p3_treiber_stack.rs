//! Pattern 3: Lock-Free Queues and Stacks - Treiber Stack
//!
//! Run with: cargo run --example p3_treiber_stack

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

pub struct TreiberStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> TreiberStack<T> {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
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

    pub fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                return None;
            }

            unsafe {
                let next = (*head).next;

                if self
                    .head
                    .compare_exchange_weak(head, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*head).data);
                    // WARNING: This is unsafe! We should use hazard pointers or epoch-based GC
                    // For now, we leak the node to avoid use-after-free
                    // drop(Box::from_raw(head));
                    return Some(data);
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

// Work-stealing deque (simplified)
pub struct WorkStealingDeque<T> {
    bottom: AtomicPtr<Node<T>>,
    top: AtomicPtr<Node<T>>,
}

impl<T> WorkStealingDeque<T> {
    pub fn new() -> Self {
        Self {
            bottom: AtomicPtr::new(ptr::null_mut()),
            top: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let bottom = self.bottom.load(Ordering::Relaxed);
            unsafe {
                (*new_node).next = bottom;
            }

            if self
                .bottom
                .compare_exchange_weak(bottom, new_node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                // If this is the first node, also set top
                if bottom.is_null() {
                    let _ = self.top.compare_exchange(
                        ptr::null_mut(),
                        new_node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                }
                break;
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        // Owner pops from bottom (LIFO - better cache locality)
        loop {
            let bottom = self.bottom.load(Ordering::Acquire);

            if bottom.is_null() {
                return None;
            }

            unsafe {
                let next = (*bottom).next;

                if self
                    .bottom
                    .compare_exchange_weak(bottom, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*bottom).data);
                    return Some(data);
                }
            }
        }
    }

    pub fn steal(&self) -> Option<T> {
        // Thieves steal from top (FIFO - oldest work)
        loop {
            let top = self.top.load(Ordering::Acquire);

            if top.is_null() {
                return None;
            }

            unsafe {
                let next = (*top).next;

                if self
                    .top
                    .compare_exchange_weak(top, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
                {
                    let data = ptr::read(&(*top).data);
                    return Some(data);
                }
            }
        }
    }
}

unsafe impl<T: Send> Send for WorkStealingDeque<T> {}
unsafe impl<T: Send> Sync for WorkStealingDeque<T> {}

fn main() {
    println!("=== Treiber Stack ===\n");

    let stack = Arc::new(TreiberStack::new());
    let mut handles = vec![];

    // Producers
    for i in 0..5 {
        let stack = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                stack.push(i * 100 + j);
            }
        }));
    }

    // Wait for producers
    for handle in handles {
        handle.join().unwrap();
    }

    // Consumer
    let mut count = 0;
    while stack.pop().is_some() {
        count += 1;
    }

    println!("Popped {} items", count);
    println!("Stack empty: {}", stack.is_empty());

    println!("\n=== Work Stealing Deque ===\n");

    let deque = Arc::new(WorkStealingDeque::new());

    // Owner thread
    let owner_deque = Arc::clone(&deque);
    let owner = thread::spawn(move || {
        for i in 0..100 {
            owner_deque.push(i);
        }

        let mut popped = 0;
        while owner_deque.pop().is_some() {
            popped += 1;
        }
        println!("Owner popped: {}", popped);
    });

    // Thief threads
    let mut thieves = vec![];
    for id in 0..3 {
        let thief_deque = Arc::clone(&deque);
        thieves.push(thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(10));
            let mut stolen = 0;
            while thief_deque.steal().is_some() {
                stolen += 1;
            }
            println!("Thief {} stole: {}", id, stolen);
        }));
    }

    owner.join().unwrap();
    for thief in thieves {
        thief.join().unwrap();
    }
}
