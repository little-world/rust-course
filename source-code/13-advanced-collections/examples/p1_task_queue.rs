//! Pattern 1: VecDeque and Ring Buffers
//! Task Queue with Priority Lanes
//!
//! Run with: cargo run --example p1_task_queue

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Priority {
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone)]
struct Task {
    id: u64,
    description: String,
    priority: Priority,
}

struct TaskQueue {
    high_priority: VecDeque<Task>,
    normal_priority: VecDeque<Task>,
    low_priority: VecDeque<Task>,
    next_id: u64,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            next_id: 1,
        }
    }

    fn enqueue(&mut self, description: String, priority: Priority) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task {
            id,
            description,
            priority: priority.clone(),
        };

        match priority {
            Priority::High => self.high_priority.push_back(task),
            Priority::Normal => self.normal_priority.push_back(task),
            Priority::Low => self.low_priority.push_back(task),
        }

        id
    }

    fn enqueue_urgent(&mut self, description: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task {
            id,
            description,
            priority: Priority::High,
        };

        // Add to front of high priority queue
        self.high_priority.push_front(task);
        id
    }

    fn dequeue(&mut self) -> Option<Task> {
        // Try high priority first
        if let Some(task) = self.high_priority.pop_front() {
            return Some(task);
        }

        // Then normal priority
        if let Some(task) = self.normal_priority.pop_front() {
            return Some(task);
        }

        // Finally low priority
        self.low_priority.pop_front()
    }

    fn peek(&self) -> Option<&Task> {
        self.high_priority
            .front()
            .or_else(|| self.normal_priority.front())
            .or_else(|| self.low_priority.front())
    }

    fn remove_by_id(&mut self, id: u64) -> Option<Task> {
        // Helper to remove from a specific queue
        fn remove_from_queue(
            queue: &mut VecDeque<Task>, id: u64
        ) -> Option<Task> {
            let pos = queue.iter().position(|t| t.id == id)?;
            queue.remove(pos)
        }

        remove_from_queue(&mut self.high_priority, id)
            .or_else(|| remove_from_queue(&mut self.normal_priority, id))
            .or_else(|| remove_from_queue(&mut self.low_priority, id))
    }

    fn len(&self) -> usize {
        self.high_priority.len()
            + self.normal_priority.len()
            + self.low_priority.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn clear(&mut self) {
        self.high_priority.clear();
        self.normal_priority.clear();
        self.low_priority.clear();
    }
}

// Example usage
fn main() {
    let mut queue = TaskQueue::new();

    queue.enqueue("Process data".to_string(), Priority::Normal);
    queue.enqueue("Backup database".to_string(), Priority::Low);
    queue.enqueue("Handle error".to_string(), Priority::High);
    queue.enqueue_urgent("Critical security patch".to_string());

    println!("Processing tasks:");
    while let Some(task) = queue.dequeue() {
        println!("  [{:?}] {}", task.priority, task.description);
    }

    println!("\n=== Key Points ===");
    println!("1. VecDeque provides O(1) push/pop at both ends");
    println!("2. push_front() for urgent items");
    println!("3. Multiple queues for priority lanes");
    println!("4. peek() without consuming");
}
