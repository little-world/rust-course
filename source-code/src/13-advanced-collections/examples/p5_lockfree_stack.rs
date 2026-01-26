//! Pattern 5: Lock-Free Data Structures
//! Lock-Free Stack
//!
//! Run with: cargo run --example p5_lockfree_stack

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;
use std::sync::Arc;
use std::thread;

struct Node<T> {
    data: T,
    next: *mut Node<T>,
}

struct LockFreeStack<T> {
    head: AtomicPtr<Node<T>>,
}

impl<T> LockFreeStack<T> {
    fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node {
            data,
            next: ptr::null_mut(),
        }));

        loop {
            let head = self.head.load(Ordering::Acquire);
            unsafe {
                (*new_node).next = head;
            }

            // Try to swap: if head unchanged, install new_node
            if self.head.compare_exchange(
                head, new_node,
                Ordering::Release, Ordering::Acquire
            ).is_ok()
            {
                break;
            }
        }
    }

    fn pop(&self) -> Option<T> {
        loop {
            let head = self.head.load(Ordering::Acquire);

            if head.is_null() {
                return None;
            }

            unsafe {
                let next = (*head).next;

                // Try to swap head with next
                if self.head.compare_exchange(
                    head, next,
                    Ordering::Release, Ordering::Acquire
                ).is_ok()
                {
                    let data = ptr::read(&(*head).data);
                    // Note: In production, use proper memory reclamation (epoch-based)
                    // Deallocating here can cause use-after-free in concurrent scenarios
                    // drop(Box::from_raw(head)); // Commented out for safety
                    return Some(data);
                }
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.head.load(Ordering::Acquire).is_null()
    }
}

unsafe impl<T: Send> Send for LockFreeStack<T> {}
unsafe impl<T: Send> Sync for LockFreeStack<T> {}

//======================================
// Real-world: Thread-safe work stealing
//======================================
struct WorkStealingQueue<T> {
    stack: Arc<LockFreeStack<T>>,
}

impl<T: Send + 'static> WorkStealingQueue<T> {
    fn new() -> Self {
        Self {
            stack: Arc::new(LockFreeStack::new()),
        }
    }

    fn push(&self, item: T) {
        self.stack.push(item);
    }

    fn steal(&self) -> Option<T> {
        self.stack.pop()
    }

    fn clone_handle(&self) -> Self {
        Self {
            stack: Arc::clone(&self.stack),
        }
    }
}

fn main() {
    println!("=== Lock-Free Stack ===\n");

    let stack = Arc::new(LockFreeStack::new());

    // Spawn multiple threads pushing concurrently
    let mut handles = vec![];

    for thread_id in 0..4 {
        let stack_clone = Arc::clone(&stack);
        handles.push(thread::spawn(move || {
            for i in 0..100 {
                stack_clone.push(thread_id * 1000 + i);
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Pop all elements
    let mut count = 0;
    while stack.pop().is_some() {
        count += 1;
    }

    println!("Total items pushed and popped: {}", count);

    println!("\n=== Work Stealing ===\n");

    let queue = WorkStealingQueue::new();

    // Producer thread
    let producer_queue = queue.clone_handle();
    let producer = thread::spawn(move || {
        for i in 0..1000 {
            producer_queue.push(i);
        }
    });

    // Consumer threads
    let mut consumers = vec![];
    for _ in 0..3 {
        let consumer_queue = queue.clone_handle();
        consumers.push(thread::spawn(move || {
            let mut stolen = 0;
            while let Some(_) = consumer_queue.steal() {
                stolen += 1;
            }
            stolen
        }));
    }

    producer.join().unwrap();

    let mut total_stolen = 0;
    for consumer in consumers {
        total_stolen += consumer.join().unwrap();
    }

    println!("Total items stolen: {}", total_stolen);

    println!("\n=== Key Points ===");
    println!("1. Compare-and-Swap (CAS) for atomic updates");
    println!("2. No locks means no blocking");
    println!("3. ABA problem requires epoch-based reclamation");
    println!("4. Memory ordering: Acquire/Release semantics");
}
