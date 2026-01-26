//! Pattern 5: Synchronization Primitives
//! Condvar for a Bounded Queue
//!
//! Run with: cargo run --example p5_condvar_queue

use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;
use std::thread;

struct BoundedQueue<T> {
    queue: Mutex<VecDeque<T>>,
    condvar: Condvar,
    capacity: usize,
}

impl<T> BoundedQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            condvar: Condvar::new(),
            capacity,
        }
    }

    fn push(&self, item: T) {
        let mut queue = self.queue.lock().unwrap();
        // Wait while the queue is full.
        while queue.len() >= self.capacity {
            queue = self.condvar.wait(queue).unwrap();
        }
        queue.push_back(item);
        // Notify one waiting consumer that there's new data.
        self.condvar.notify_one();
    }

    fn pop(&self) -> T {
        let mut queue = self.queue.lock().unwrap();
        // Wait while the queue is empty.
        while queue.is_empty() {
            queue = self.condvar.wait(queue).unwrap();
        }
        let item = queue.pop_front().unwrap();
        // Notify one waiting producer that there's new space.
        self.condvar.notify_one();
        item
    }
}

fn condvar_for_queue() {
    let queue = Arc::new(BoundedQueue::new(3));

    // Producer thread
    let queue_clone_p = Arc::clone(&queue);
    let producer = thread::spawn(move || {
        for i in 0..10 {
            println!("Producer: pushing {}", i);
            queue_clone_p.push(i);
        }
    });

    // Consumer thread
    let queue_clone_c = Arc::clone(&queue);
    let consumer = thread::spawn(move || {
        for _ in 0..10 {
            let item = queue_clone_c.pop();
            println!("Consumer: popped {}", item);
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

fn main() {
    println!("=== Condvar for a Bounded Queue ===\n");
    condvar_for_queue();

    println!("\n=== Key Points ===");
    println!("1. Condvar allows efficient waiting for a condition");
    println!("2. Always use with a Mutex (Condvar releases lock while waiting)");
    println!("3. Use while loop to re-check condition (spurious wakeups)");
    println!("4. notify_one() wakes one waiter, notify_all() wakes all");
}
