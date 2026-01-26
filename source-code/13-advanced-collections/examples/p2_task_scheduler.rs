//! Pattern 2: BinaryHeap and Priority Queues
//! Task Scheduler with Deadlines
//!
//! Run with: cargo run --example p2_task_scheduler

use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
struct Task {
    id: u64,
    priority: u32,
    deadline: u64,
    duration: u32,
    description: String,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by priority (higher is better)
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Then by deadline (earlier is better, so reverse)
                other.deadline.cmp(&self.deadline)
            }
            other => other,
        }
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct TaskScheduler {
    heap: BinaryHeap<Task>,
    current_time: u64,
    next_id: u64,
}

impl TaskScheduler {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            current_time: 0,
            next_id: 1,
        }
    }

    fn schedule(
        &mut self, desc: String, priority: u32,
        deadline: u64, duration: u32
    ) {
        let task = Task {
            id: self.next_id,
            priority,
            deadline,
            duration,
            description: desc,
        };
        self.next_id += 1;
        self.heap.push(task);
    }

    fn execute_next(&mut self) -> Option<Task> {
        let task = self.heap.pop()?;

        // Check if deadline missed
        if self.current_time > task.deadline {
            println!(
                "Warning: Task {} missed deadline (current={}, deadline={})",
                task.id, self.current_time, task.deadline
            );
        }

        self.current_time += task.duration as u64;
        Some(task)
    }

    fn peek(&self) -> Option<&Task> {
        self.heap.peek()
    }

    fn pending_count(&self) -> usize {
        self.heap.len()
    }

    fn execute_all(&mut self) -> Vec<Task> {
        let mut executed = Vec::new();
        while let Some(task) = self.execute_next() {
            executed.push(task);
        }
        executed
    }

    fn get_current_time(&self) -> u64 {
        self.current_time
    }
}

//==========================================
// Real-world example: CPU process scheduler
//==========================================
#[derive(Debug, Clone, Eq, PartialEq)]
struct Process {
    pid: u32,
    priority: i32,      // Higher is more important
    arrival_time: u64,
    burst_time: u32,
    remaining_time: u32,
}

impl Ord for Process {
    fn cmp(&self, other: &Self) -> Ordering {
        // Highest priority first
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                // Shortest remaining time first (SRT scheduling)
                other.remaining_time.cmp(&self.remaining_time)
            }
            other => other,
        }
    }
}

impl PartialOrd for Process {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct CpuScheduler {
    ready_queue: BinaryHeap<Process>,
    current_time: u64,
}

impl CpuScheduler {
    fn new() -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
            current_time: 0,
        }
    }

    fn add_process(&mut self, process: Process) {
        self.ready_queue.push(process);
    }

    fn run_time_slice(&mut self, time_slice: u32) -> Option<ProcessResult> {
        let mut process = self.ready_queue.pop()?;

        let executed = time_slice.min(process.remaining_time);
        process.remaining_time -= executed;
        self.current_time += executed as u64;

        let result = ProcessResult {
            pid: process.pid,
            time_executed: executed,
            completed: process.remaining_time == 0,
        };

        // Re-queue if not finished
        if process.remaining_time > 0 {
            self.ready_queue.push(process);
        }

        Some(result)
    }

    fn simulate(&mut self, time_slice: u32) {
        println!("Starting CPU scheduler simulation...\n");

        while let Some(result) = self.run_time_slice(time_slice) {
            println!(
                "Time {}: Process {} executed for {}ms {}",
                self.current_time,
                result.pid,
                result.time_executed,
                if result.completed { "(completed)" } else { "" }
            );
        }
    }
}

#[derive(Debug)]
struct ProcessResult {
    pid: u32,
    time_executed: u32,
    completed: bool,
}

fn main() {
    println!("=== Task Scheduler ===\n");

    let mut scheduler = TaskScheduler::new();

    scheduler.schedule("Write report".to_string(), 5, 100, 20);
    scheduler.schedule("Fix bug".to_string(), 10, 50, 15);
    scheduler.schedule("Code review".to_string(), 7, 80, 10);
    scheduler.schedule("Meeting".to_string(), 8, 60, 30);

    println!("Executing tasks in priority order:\n");
    let executed = scheduler.execute_all();

    for task in executed {
        println!(
            "Task {}: {} (priority={}, deadline={})",
            task.id, task.description, task.priority, task.deadline
        );
    }

    println!("\n=== CPU Scheduler ===\n");

    let mut cpu = CpuScheduler::new();

    cpu.add_process(Process {
        pid: 1,
        priority: 5,
        arrival_time: 0,
        burst_time: 30,
        remaining_time: 30,
    });

    cpu.add_process(Process {
        pid: 2,
        priority: 10,
        arrival_time: 0,
        burst_time: 20,
        remaining_time: 20,
    });

    cpu.add_process(Process {
        pid: 3,
        priority: 7,
        arrival_time: 0,
        burst_time: 15,
        remaining_time: 15,
    });

    cpu.simulate(10); // 10ms time slices

    println!("\n=== Key Points ===");
    println!("1. BinaryHeap is a max-heap by default");
    println!("2. O(log n) push and pop operations");
    println!("3. Custom Ord implementation for priority");
    println!("4. Ideal for task scheduling and event simulation");
}
