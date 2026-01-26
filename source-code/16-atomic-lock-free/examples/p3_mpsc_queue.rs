//! Pattern 3: Lock-Free Queues and Stacks - MPSC Queue
//!
//! Run with: cargo run --example p3_mpsc_queue

use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

struct QueueNode<T> {
    data: Option<T>,
    next: AtomicPtr<QueueNode<T>>,
}

pub struct MpscQueue<T> {
    head: AtomicPtr<QueueNode<T>>,
    tail: AtomicPtr<QueueNode<T>>,
}

impl<T> MpscQueue<T> {
    pub fn new() -> Self {
        let sentinel = Box::into_raw(Box::new(QueueNode {
            data: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        Self {
            head: AtomicPtr::new(sentinel),
            tail: AtomicPtr::new(sentinel),
        }
    }

    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(QueueNode {
            data: Some(data),
            next: AtomicPtr::new(ptr::null_mut()),
        }));

        // Insert at tail
        loop {
            let tail = self.tail.load(Ordering::Acquire);

            unsafe {
                let next = (*tail).next.load(Ordering::Acquire);

                if next.is_null() {
                    // Tail is actually the last node
                    if (*tail)
                        .next
                        .compare_exchange(
                            ptr::null_mut(),
                            new_node,
                            Ordering::Release,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        // Try to update tail (optional, helps next push)
                        let _ = self.tail.compare_exchange(
                            tail,
                            new_node,
                            Ordering::Release,
                            Ordering::Acquire,
                        );
                        break;
                    }
                } else {
                    // Help other threads by updating tail
                    let _ = self.tail.compare_exchange(
                        tail,
                        next,
                        Ordering::Release,
                        Ordering::Acquire,
                    );
                }
            }
        }
    }

    pub fn pop(&self) -> Option<T> {
        unsafe {
            let head = self.head.load(Ordering::Acquire);
            let next = (*head).next.load(Ordering::Acquire);

            if next.is_null() {
                return None;
            }

            // Move head forward
            self.head.store(next, Ordering::Release);

            // Take data from old sentinel
            let data = (*next).data.take();

            // Drop old sentinel (safe because we're single consumer)
            drop(Box::from_raw(head));

            data
        }
    }
}

unsafe impl<T: Send> Send for MpscQueue<T> {}
unsafe impl<T: Send> Sync for MpscQueue<T> {}

impl<T> Drop for MpscQueue<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
        // Drop the sentinel
        unsafe {
            let head = self.head.load(Ordering::Relaxed);
            if !head.is_null() {
                drop(Box::from_raw(head));
            }
        }
    }
}

// Bounded SPSC queue (Single Producer Single Consumer)
pub struct BoundedSpscQueue<T> {
    buffer: Vec<Option<T>>,
    head: AtomicUsize,
    tail: AtomicUsize,
    capacity: usize,
}

impl<T> BoundedSpscQueue<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }

        Self {
            buffer,
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            capacity,
        }
    }

    pub fn push(&mut self, data: T) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity;

        if next_tail == self.head.load(Ordering::Acquire) {
            return false; // Full
        }

        self.buffer[tail] = Some(data);
        self.tail.store(next_tail, Ordering::Release);
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        let head = self.head.load(Ordering::Relaxed);

        if head == self.tail.load(Ordering::Acquire) {
            return None; // Empty
        }

        let data = self.buffer[head].take();
        self.head.store((head + 1) % self.capacity, Ordering::Release);
        data
    }

    pub fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire)
    }

    pub fn is_full(&self) -> bool {
        let tail = self.tail.load(Ordering::Relaxed);
        let next_tail = (tail + 1) % self.capacity;
        next_tail == self.head.load(Ordering::Acquire)
    }
}

fn main() {
    println!("=== MPSC Queue ===\n");

    let queue = Arc::new(MpscQueue::new());
    let mut handles = vec![];

    // Multiple producers
    for i in 0..5 {
        let queue = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                queue.push(i * 100 + j);
            }
        }));
    }

    // Wait for producers
    for handle in handles {
        handle.join().unwrap();
    }

    // Single consumer
    let mut count = 0;
    while queue.pop().is_some() {
        count += 1;
    }

    println!("Consumed {} items", count);

    println!("\n=== Bounded SPSC Queue ===\n");

    let mut spsc: BoundedSpscQueue<i32> = BoundedSpscQueue::new(10);

    // Push some items
    for i in 0..8 {
        if spsc.push(i) {
            println!("Pushed: {}", i);
        } else {
            println!("Queue full, couldn't push: {}", i);
        }
    }

    println!("\nQueue full: {}", spsc.is_full());

    // Pop some items
    while let Some(val) = spsc.pop() {
        println!("Popped: {}", val);
    }

    println!("\nQueue empty: {}", spsc.is_empty());
}
