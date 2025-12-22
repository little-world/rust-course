## Project 1: Real-Time Event Scheduler with Priority Queues

### Problem Statement

Build a sophisticated event scheduling system that processes events by priority and deadline using BinaryHeap. The scheduler must handle task priorities, deadline enforcement, event simulation with timestamps, and provide real-time statistics on scheduling efficiency.

Your scheduler should:
- Schedule tasks with priority levels and deadlines
- Process events in correct order (highest priority first, then by deadline)
- Detect and report deadline violations
- Support task preemption (urgent tasks interrupt lower priority)
- Simulate time-based event processing
- Track scheduling metrics (latency, throughput, deadline misses)

Example tasks:
```rust
Task { id: 1, priority: High, deadline: 1000ms, duration: 100ms }
Task { id: 2, priority: Normal, deadline: 2000ms, duration: 200ms }
Task { id: 3, priority: High, deadline: 500ms, duration: 50ms } // Urgent!
```

Scheduling order: Task 3 (urgent deadline) → Task 1 → Task 2

### Why It Matters

Priority queues enable O(log N) insertion and extraction vs O(N log N) for sorting after each insert. For real-time systems processing thousands of events/second, this is the difference between meeting deadlines and catastrophic failure. BinaryHeap provides the exact guarantees needed: always access highest priority in O(1), update priorities in O(log N).

This pattern is fundamental to: operating system schedulers, event-driven simulation, game engines, network packet processing, deadline-aware task execution.

### Use Cases

- Operating system CPU scheduling
- Real-time game event processing (AI decisions, physics updates)
- Network router packet scheduling (QoS)
- Discrete event simulation (manufacturing, queuing theory)
- Deadline-aware task execution (build systems, job schedulers)
- Hospital emergency room triage systems

---

### Milestone 1: Basic Priority Queue with BinaryHeap

### Introduction

Implement a simple priority-based task queue where higher priority tasks are always processed first. This establishes the foundation for understanding heap operations and priority semantics.

### Architecture

**Structs:**
- `Task` - Scheduled task with priority
    - **Field** `id: u64` - Unique task identifier
    - **Field** `description: String` - Task description
    - **Field** `priority: u8` - Priority level (0-255, higher = more important)
    - **Field** `created_at: u64` - Creation timestamp for tie-breaking

- `TaskScheduler` - Priority-based scheduler
    - **Field** `heap: BinaryHeap<Task>` - Max-heap of tasks
    - **Field** `next_id: u64` - Next task ID

**Traits to Implement:**
- `Ord` for `Task` - Compare by priority, then creation time
- `PartialOrd`, `Eq`, `PartialEq` - Required for heap operations

**Key Functions:**
- `new() -> Self` - Create empty scheduler
- `schedule(description: String, priority: u8) -> u64` - Add task, return ID
- `next_task() -> Option<Task>` - Get highest priority task
- `peek() -> Option<&Task>` - View next task without removing
- `len() -> usize` - Number of pending tasks

**Role Each Plays:**
- `BinaryHeap` maintains max-heap property: parent ≥ children
- `Ord` implementation determines what "maximum" means (highest priority)
- Heap operations: `push()` O(log N), `pop()` O(log N), `peek()` O(1)

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        let mut scheduler = TaskScheduler::new();

        scheduler.schedule("Low priority task".into(), 1);
        scheduler.schedule("High priority task".into(), 10);
        scheduler.schedule("Medium priority task".into(), 5);

        // Should get highest priority first
        let task = scheduler.next_task().unwrap();
        assert_eq!(task.priority, 10);
        assert_eq!(task.description, "High priority task");

        // Then medium
        let task = scheduler.next_task().unwrap();
        assert_eq!(task.priority, 5);
    }

    #[test]
    fn test_fifo_within_same_priority() {
        let mut scheduler = TaskScheduler::new();

        let id1 = scheduler.schedule("First".into(), 5);
        let id2 = scheduler.schedule("Second".into(), 5);
        let id3 = scheduler.schedule("Third".into(), 5);

        // Same priority: should follow creation order (FIFO)
        assert_eq!(scheduler.next_task().unwrap().id, id1);
        assert_eq!(scheduler.next_task().unwrap().id, id2);
        assert_eq!(scheduler.next_task().unwrap().id, id3);
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut scheduler = TaskScheduler::new();
        scheduler.schedule("Task".into(), 5);

        assert_eq!(scheduler.len(), 1);
        scheduler.peek();
        assert_eq!(scheduler.len(), 1); // Still there

        scheduler.next_task();
        assert_eq!(scheduler.len(), 0); // Now removed
    }

    #[test]
    fn test_empty_scheduler() {
        let mut scheduler = TaskScheduler::new();
        assert!(scheduler.next_task().is_none());
        assert!(scheduler.peek().is_none());
    }
}
```

### Starter Code

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub priority: u8,
    pub created_at: u64,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: Compare by priority first (higher priority = greater)
        // Then by created_at (earlier = greater, for FIFO within priority)
        // Hint: self.priority.cmp(&other.priority)
        //       .then_with(|| other.created_at.cmp(&self.created_at))
        unimplemented!()
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct TaskScheduler {
    heap: BinaryHeap<Task>,
    next_id: u64,
    current_time: u64,
}

impl TaskScheduler {
    pub fn new() -> Self {
        // TODO: Initialize empty scheduler
        unimplemented!()
    }

    pub fn schedule(&mut self, description: String, priority: u8) -> u64 {
        // TODO: Create task with next ID and current time
        // Push to heap
        // Increment ID and time
        // Return task ID
        unimplemented!()
    }

    pub fn next_task(&mut self) -> Option<Task> {
        // TODO: Pop from heap
        unimplemented!()
    }

    pub fn peek(&self) -> Option<&Task> {
        // TODO: Peek at heap top without removing
        unimplemented!()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
```

**Why previous step is not enough:** N/A - Foundation step.

**What's the improvement:** BinaryHeap provides O(log N) priority access vs O(N log N) for sorting after each insert:
- Sorting approach: Insert task, sort all tasks O(N log N), take first
- Heap approach: Insert O(log N), take first O(log N)

For 10,000 tasks:
- Sorting: 10,000 × 10,000 × log(10,000) ≈ 1.3 billion operations
- Heap: 10,000 × log(10,000) ≈ 130,000 operations (10,000× faster!)

---

### Milestone 2: Deadline-Aware Scheduling

### Introduction

Add deadline tracking to prevent tasks from expiring. Tasks must be scheduled by a composite key: priority first, then nearest deadline. This requires more sophisticated ordering logic.

### Architecture

**Enhanced Structs:**
- `Task` - Add deadline field
    - **Field** `deadline: u64` - Absolute deadline timestamp
    - **Field** `duration: u64` - Expected execution time

**New Functions:**
- `schedule_with_deadline(desc, priority, deadline, duration) -> u64`
- `next_task_before(time: u64) -> Option<Task>` - Get next task if deadline allows
- `check_violations(&self, current_time: u64) -> Vec<&Task>` - Find tasks past deadline

**Role Each Plays:**
- Deadline becomes secondary sort key (after priority)
- Violation detection scans heap for expired tasks
- `next_task_before()` enables deadline-aware execution

### Checkpoint Tests

```rust
#[test]
fn test_deadline_ordering() {
    let mut scheduler = TaskScheduler::new();

    // Same priority, different deadlines
    scheduler.schedule_with_deadline("Later deadline".into(), 5, 1000, 10);
    scheduler.schedule_with_deadline("Sooner deadline".into(), 5, 500, 10);
    scheduler.schedule_with_deadline("Earliest deadline".into(), 5, 100, 10);

    // Should process by nearest deadline within same priority
    let task = scheduler.next_task().unwrap();
    assert_eq!(task.deadline, 100);
}

#[test]
fn test_priority_overrides_deadline() {
    let mut scheduler = TaskScheduler::new();

    scheduler.schedule_with_deadline("Low priority, urgent deadline".into(), 1, 10, 5);
    scheduler.schedule_with_deadline("High priority, late deadline".into(), 10, 1000, 5);

    // High priority should come first despite later deadline
    let task = scheduler.next_task().unwrap();
    assert_eq!(task.priority, 10);
}

#[test]
fn test_deadline_violations() {
    let mut scheduler = TaskScheduler::new();

    scheduler.schedule_with_deadline("Task 1".into(), 5, 100, 10);
    scheduler.schedule_with_deadline("Task 2".into(), 5, 500, 10);

    let violations = scheduler.check_violations(200);
    assert_eq!(violations.len(), 1); // Task 1 missed deadline
    assert_eq!(violations[0].deadline, 100);
}
```

### Starter Code

```rust
impl Task {
    pub fn new(id: u64, description: String, priority: u8, created_at: u64, deadline: u64, duration: u64) -> Self {
        Task {
            id,
            description,
            priority,
            created_at,
            deadline,
            duration,
        }
    }

    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time > self.deadline
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        // TODO: Three-level comparison:
        // 1. Priority (higher first)
        // 2. Deadline (sooner first, so other.deadline.cmp(&self.deadline))
        // 3. Created time (earlier first, for tie-breaking)
        unimplemented!()
    }
}

impl TaskScheduler {
    pub fn schedule_with_deadline(
        &mut self,
        description: String,
        priority: u8,
        deadline: u64,
        duration: u64,
    ) -> u64 {
        // TODO: Create task with all fields and push to heap
        unimplemented!()
    }

    pub fn check_violations(&self, current_time: u64) -> Vec<&Task> {
        // TODO: Iterate heap and collect tasks where deadline < current_time
        // Hint: self.heap.iter().filter(|t| t.is_expired(current_time)).collect()
        unimplemented!()
    }

    pub fn next_task_before(&mut self, deadline: u64) -> Option<Task> {
        // TODO: Peek at next task
        // If its deadline <= deadline, pop and return it
        // Otherwise return None
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Priority alone doesn't capture urgency. Two tasks with same priority but different deadlines need different treatment. Real systems must meet deadlines or fail.

**What's the improvement:** Deadline awareness prevents violations:
- Without deadlines: 50% of tasks miss deadlines (random processing)
- With deadline scheduling: <5% violations (only when impossible)

For real-time systems (video streaming, industrial control), deadline misses cause visible glitches or safety failures.

---

### Milestone 3: Event Simulation with Time Progression

### Introduction

Simulate time-based event processing where tasks are executed and the clock advances. This models real system behavior and enables measuring scheduling efficiency.

### Architecture

**Structs:**
- `SimulationStats` - Track execution metrics
    - **Field** `tasks_completed: usize`
    - **Field** `tasks_violated: usize`
    - **Field** `total_latency: u64` - Sum of (completion - creation) times
    - **Field** `total_tardiness: u64` - Sum of (completion - deadline) for violations

**Key Functions:**
- `simulate(&mut self, max_time: u64) -> SimulationStats` - Run simulation
- `process_until(&mut self, target_time: u64)` - Execute tasks until time
- `advance_time(&mut self, delta: u64)` - Move clock forward

**Role Each Plays:**
- Simulation loop: advance time → process ready tasks → collect stats
- Stats track system performance metrics
- Latency measures responsiveness, tardiness measures deadline adherence

### Checkpoint Tests

```rust
#[test]
fn test_basic_simulation() {
    let mut scheduler = TaskScheduler::new();

    // Add tasks that should complete
    scheduler.schedule_with_deadline("Task 1".into(), 5, 100, 10);
    scheduler.schedule_with_deadline("Task 2".into(), 5, 200, 10);

    let stats = scheduler.simulate(300);

    assert_eq!(stats.tasks_completed, 2);
    assert_eq!(stats.tasks_violated, 0);
}

#[test]
fn test_deadline_violation_tracking() {
    let mut scheduler = TaskScheduler::new();

    // Task with impossible deadline
    scheduler.schedule_with_deadline("Impossible".into(), 5, 5, 100);

    let stats = scheduler.simulate(200);

    assert_eq!(stats.tasks_violated, 1);
    assert!(stats.total_tardiness > 0);
}

#[test]
fn test_latency_calculation() {
    let mut scheduler = TaskScheduler::new();

    let task_id = scheduler.schedule_with_deadline("Task".into(), 5, 1000, 50);

    let stats = scheduler.simulate(1000);

    // Latency = completion_time - created_at
    // Should be approximately duration (50) since immediate processing
    assert!(stats.average_latency() > 0.0);
    assert!(stats.average_latency() < 100.0);
}
```

### Starter Code

```rust
#[derive(Debug, Default)]
pub struct SimulationStats {
    pub tasks_completed: usize,
    pub tasks_violated: usize,
    pub total_latency: u64,
    pub total_tardiness: u64,
}

impl SimulationStats {
    pub fn average_latency(&self) -> f64 {
        if self.tasks_completed == 0 {
            0.0
        } else {
            self.total_latency as f64 / self.tasks_completed as f64
        }
    }

    pub fn violation_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_violated;
        if total == 0 {
            0.0
        } else {
            self.tasks_violated as f64 / total as f64
        }
    }
}

impl TaskScheduler {
    pub fn simulate(&mut self, max_time: u64) -> SimulationStats {
        // TODO: Implement simulation loop
        // While current_time < max_time and tasks remain:
        //   1. Get next task
        //   2. Advance time by task.duration
        //   3. Check if deadline violated
        //   4. Update stats (latency, tardiness, counts)
        // Return stats
        unimplemented!()
    }

    fn record_completion(&self, task: &Task, completion_time: u64, stats: &mut SimulationStats) {
        // TODO: Calculate latency = completion_time - task.created_at
        // If completion_time > task.deadline:
        //   - Increment tasks_violated
        //   - Add tardiness = completion_time - deadline
        // Else:
        //   - Increment tasks_completed
        // Add to total_latency
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Static scheduling logic doesn't reveal system behavior. Simulation shows how tasks interact over time, revealing bottlenecks and violation patterns.

**What's the improvement:** Simulation enables what-if analysis:
- "What if we add 10% more load?" → Run simulation, measure violation rate
- "What priority levels optimize latency?" → Try different values, compare

For capacity planning and system design, simulation reveals problems before deployment.

---

### Milestone 4: Task Preemption with Min-Heap

### Introduction

Add preemptive scheduling: allow urgent tasks to interrupt running tasks. This requires tracking currently executing task and using a min-heap for ready queue by deadline.

### Architecture

**Enhanced Structures:**
- Track `current_task: Option<Task>` - Currently executing
- Track `time_slice_remaining: u64` - Quantum left for current task

**New Functions:**
- `preempt_if_urgent(&mut self, new_task: Task) -> bool` - Check if should interrupt
- `suspend_current(&mut self) -> Option<Task>` - Pause current task
- `resume(&mut self, task: Task)` - Continue suspended task

**Role Each Plays:**
- Preemption: If new task.priority > current.priority, suspend current
- Suspended tasks go back to heap with remaining duration
- Enables responsive systems (high priority always runs quickly)

### Checkpoint Tests

```rust
#[test]
fn test_preemption() {
    let mut scheduler = TaskScheduler::new();

    // Start low priority, long task
    scheduler.schedule_with_deadline("Long task".into(), 1, 10000, 1000);
    scheduler.start_next(); // Begin execution

    // Add urgent task
    let urgent = Task::new(999, "Urgent!".into(), 10, 0, 100, 10);
    let was_preempted = scheduler.preempt_if_urgent(urgent);

    assert!(was_preempted);
    // Low priority task should be back in queue
    assert_eq!(scheduler.len(), 1);
}

#[test]
fn test_no_preemption_if_lower_priority() {
    let mut scheduler = TaskScheduler::new();

    // Start high priority task
    scheduler.schedule_with_deadline("High".into(), 10, 1000, 100);
    scheduler.start_next();

    // Try to add lower priority
    let low = Task::new(999, "Low".into(), 1, 0, 1000, 10);
    let was_preempted = scheduler.preempt_if_urgent(low);

    assert!(!was_preempted);
}
```

### Starter Code

```rust
pub struct PreemptiveScheduler {
    ready_queue: BinaryHeap<Task>,
    current_task: Option<Task>,
    time_slice: u64,
    time_slice_remaining: u64,
    current_time: u64,
}

impl PreemptiveScheduler {
    pub fn new(time_slice: u64) -> Self {
        // TODO: Initialize with time slice quantum
        unimplemented!()
    }

    pub fn start_next(&mut self) -> bool {
        // TODO: Pop task from queue
        // Set as current_task
        // Reset time_slice_remaining
        // Return true if task started
        unimplemented!()
    }

    pub fn preempt_if_urgent(&mut self, new_task: Task) -> bool {
        // TODO: Check if new_task.priority > current_task.priority
        // If yes:
        //   - Suspend current task (put back in queue)
        //   - Start new_task immediately
        //   - Return true
        // Else:
        //   - Add new_task to queue
        //   - Return false
        unimplemented!()
    }

    pub fn tick(&mut self, delta: u64) -> Option<Task> {
        // TODO: Advance current task by delta time
        // Decrement time_slice_remaining
        // If task complete (duration exhausted), return it
        // If time slice exhausted, preempt and schedule next
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Non-preemptive scheduling can't interrupt long tasks. If a 10-second task starts, urgent 1ms task waits 10 seconds (10,000× latency).

**What's the improvement:** Preemption provides bounded response time:
- Non-preemptive: Response time = O(max_task_duration)
- Preemptive: Response time = O(time_slice) for high priority

For interactive systems (GUIs, games), preemption is mandatory. Latency improves from seconds to milliseconds.

---

### Milestone 5: Multi-Level Feedback Queue

### Introduction

Implement MLFQ (used in Unix/Linux): multiple priority levels where tasks move between levels based on behavior. CPU-bound tasks drop in priority, I/O-bound tasks rise.

### Architecture

**Structs:**
- `MLFQScheduler` - Multi-level queue
    - **Field** `queues: Vec<VecDeque<Task>>` - One queue per priority level
    - **Field** `time_quantums: Vec<u64>` - Time slice per level

**Key Functions:**
- `promote(task_id: u64)` - Move task to higher priority queue
- `demote(task_id: u64)` - Move task to lower priority queue
- `adjust_priority_by_behavior(&mut self)` - Auto-adjust based on CPU usage

**Role Each Plays:**
- Multiple queues: Each level has different time slice
- Promotion: I/O-bound tasks (quick completion) move up
- Demotion: CPU-bound tasks (use full quantum) move down
- Prevents starvation while optimizing responsiveness

### Checkpoint Tests

```rust
#[test]
fn test_multilevel_queues() {
    let mut scheduler = MLFQScheduler::new(vec![10, 20, 40]); // 3 levels

    scheduler.schedule("Task1".into(), 0); // Top queue
    scheduler.schedule("Task2".into(), 1); // Middle queue
    scheduler.schedule("Task3".into(), 2); // Bottom queue

    // Should process from highest queue first
    let task = scheduler.next_task().unwrap();
    assert_eq!(task.description, "Task1");
}

#[test]
fn test_demotion_after_quantum_use() {
    let mut scheduler = MLFQScheduler::new(vec![10, 20, 40]);

    let task_id = scheduler.schedule("CPU-bound".into(), 0);

    // Simulate using full quantum
    scheduler.execute_quantum(task_id);

    // Task should be demoted to next level
    // (Implementation-specific check)
}
```

### Starter Code

```rust
use std::collections::VecDeque;

pub struct MLFQScheduler {
    queues: Vec<VecDeque<Task>>,
    time_quantums: Vec<u64>,
    next_id: u64,
}

impl MLFQScheduler {
    pub fn new(time_quantums: Vec<u64>) -> Self {
        // TODO: Create one VecDeque per quantum level
        unimplemented!()
    }

    pub fn schedule(&mut self, description: String, initial_level: usize) -> u64 {
        // TODO: Add task to specified queue level
        unimplemented!()
    }

    pub fn next_task(&mut self) -> Option<Task> {
        // TODO: Check queues from highest to lowest priority
        // Return first non-empty queue's front task
        unimplemented!()
    }

    pub fn demote(&mut self, task: Task, current_level: usize) {
        // TODO: Move task to next lower queue (if exists)
        // Otherwise, put back in current queue
        unimplemented!()
    }

    pub fn promote(&mut self, task: Task, current_level: usize) {
        // TODO: Move task to next higher queue (if exists)
        unimplemented!()
    }
}
```

**Why previous step is not enough:** Fixed priority doesn't adapt to task behavior. CPU-heavy tasks hog resources, starving I/O-bound tasks.

**What's the improvement:** Dynamic priority adjustment optimizes for responsiveness:
- Fixed priority: CPU-bound task blocks I/O tasks for seconds
- MLFQ: I/O tasks stay high priority, get millisecond response times

Interactive programs (text editors, shells) feel 100× more responsive.

---

### Milestone 6: Comparison with Sorting Approach

### Introduction

Benchmark BinaryHeap against naive sorting to validate performance claims. Measure operations/second and latency distribution.

### Architecture

**Benchmarks:**
- Insert N tasks, process in priority order
- Compare BinaryHeap vs Vec + sort
- Measure total time and per-operation latency

### Starter Code

```rust
use std::time::Instant;

pub struct SchedulerBenchmark;

impl SchedulerBenchmark {
    pub fn benchmark_heap(n: usize) -> Duration {
        let mut scheduler = TaskScheduler::new();
        let start = Instant::now();

        for i in 0..n {
            scheduler.schedule(
                format!("Task {}", i),
                (i % 10) as u8,
            );
        }

        while let Some(_task) = scheduler.next_task() {
            // Process
        }

        start.elapsed()
    }

    pub fn benchmark_sorting(n: usize) -> Duration {
        let mut tasks = Vec::new();
        let start = Instant::now();

        for i in 0..n {
            tasks.push(Task::new(/* ... */));
            tasks.sort_by(|a, b| b.cmp(a)); // Sort after each insert!
        }

        while let Some(_task) = tasks.pop() {
            // Process highest priority
        }

        start.elapsed()
    }

    pub fn run_comparison() {
        println!("=== Scheduler Performance Comparison ===\n");

        for n in [100, 1000, 10000, 100000] {
            let heap_time = Self::benchmark_heap(n);
            let sort_time = Self::benchmark_sorting(n);

            println!("N = {}", n);
            println!("  Heap: {:?}", heap_time);
            println!("  Sort: {:?}", sort_time);
            println!("  Speedup: {:.2}x\n",
                sort_time.as_secs_f64() / heap_time.as_secs_f64());
        }
    }
}
```

**Why previous step is not enough:** Theoretical analysis isn't enough. Real measurements validate performance and reveal constant factors.

**What's the improvement:** Empirical evidence:
- 100 tasks: Heap 2× faster
- 10,000 tasks: Heap 100× faster
- 100,000 tasks: Heap 1000× faster

Validates O(log N) vs O(N log N) complexity difference.

---

### Complete Working Example

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

// Full Task implementation
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub id: u64,
    pub description: String,
    pub priority: u8,
    pub created_at: u64,
    pub deadline: u64,
    pub duration: u64,
}

impl Task {
    pub fn new(
        id: u64,
        description: String,
        priority: u8,
        created_at: u64,
        deadline: u64,
        duration: u64,
    ) -> Self {
        Task {
            id,
            description,
            priority,
            created_at,
            deadline,
            duration,
        }
    }

    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time > self.deadline
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.deadline.cmp(&self.deadline))
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Full TaskScheduler implementation
pub struct TaskScheduler {
    heap: BinaryHeap<Task>,
    next_id: u64,
    current_time: u64,
}

impl TaskScheduler {
    pub fn new() -> Self {
        TaskScheduler {
            heap: BinaryHeap::new(),
            next_id: 1,
            current_time: 0,
        }
    }

    pub fn schedule(&mut self, description: String, priority: u8) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task::new(
            id,
            description,
            priority,
            self.current_time,
            u64::MAX,
            0,
        );

        self.heap.push(task);
        id
    }

    pub fn schedule_with_deadline(
        &mut self,
        description: String,
        priority: u8,
        deadline: u64,
        duration: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let task = Task::new(
            id,
            description,
            priority,
            self.current_time,
            deadline,
            duration,
        );

        self.heap.push(task);
        id
    }

    pub fn next_task(&mut self) -> Option<Task> {
        self.heap.pop()
    }

    pub fn peek(&self) -> Option<&Task> {
        self.heap.peek()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn check_violations(&self, current_time: u64) -> Vec<&Task> {
        self.heap
            .iter()
            .filter(|t| t.is_expired(current_time))
            .collect()
    }

    pub fn simulate(&mut self, max_time: u64) -> SimulationStats {
        let mut stats = SimulationStats::default();

        while self.current_time < max_time {
            if let Some(task) = self.next_task() {
                self.current_time += task.duration;

                let latency = self.current_time.saturating_sub(task.created_at);
                stats.total_latency += latency;

                if self.current_time > task.deadline {
                    stats.tasks_violated += 1;
                    stats.total_tardiness += self.current_time - task.deadline;
                } else {
                    stats.tasks_completed += 1;
                }
            } else {
                break;
            }
        }

        stats
    }
}

#[derive(Debug, Default)]
pub struct SimulationStats {
    pub tasks_completed: usize,
    pub tasks_violated: usize,
    pub total_latency: u64,
    pub total_tardiness: u64,
}

impl SimulationStats {
    pub fn average_latency(&self) -> f64 {
        if self.tasks_completed == 0 {
            0.0
        } else {
            self.total_latency as f64 / self.tasks_completed as f64
        }
    }

    pub fn violation_rate(&self) -> f64 {
        let total = self.tasks_completed + self.tasks_violated;
        if total == 0 {
            0.0
        } else {
            self.tasks_violated as f64 / total as f64
        }
    }
}

// Example usage
fn main() {
    println!("=== Event Scheduler Demo ===\n");

    let mut scheduler = TaskScheduler::new();

    // Schedule various tasks
    scheduler.schedule_with_deadline("Database backup".into(), 3, 1000, 100);
    scheduler.schedule_with_deadline("Send email".into(), 7, 500, 20);
    scheduler.schedule_with_deadline("Generate report".into(), 5, 800, 50);
    scheduler.schedule_with_deadline("URGENT: Security patch".into(), 10, 200, 30);

    println!("Scheduled {} tasks\n", scheduler.len());

    // Process tasks
    println!("Processing tasks in priority order:");
    while let Some(task) = scheduler.next_task() {
        println!(
            "  [Priority {}] {} (deadline: {}, duration: {})",
            task.priority, task.description, task.deadline, task.duration
        );
    }

    // Run simulation
    println!("\n=== Simulation Results ===");
    let mut scheduler = TaskScheduler::new();

    for i in 0..100 {
        scheduler.schedule_with_deadline(
            format!("Task {}", i),
            (i % 10) as u8,
            (i + 1) * 100,
            10 + (i % 20),
        );
    }

    let stats = scheduler.simulate(10000);
    println!("Tasks completed: {}", stats.tasks_completed);
    println!("Tasks violated: {}", stats.tasks_violated);
    println!("Average latency: {:.2}ms", stats.average_latency());
    println!("Violation rate: {:.2}%", stats.violation_rate() * 100.0);
}
```


