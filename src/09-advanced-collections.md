# Advanced Collections

[VecDeque and Ring Buffers](#vecdeque-and-ring-buffers)

- Problem: Vec only O(1) at one end; queue operations O(N); manual ring
  buffer wrapping
- Solution: VecDeque with O(1) push/pop at both ends; built-in ring buffer
  behavior
- Why It Matters: 1000x faster for queues (O(N) vs O(N²)); eliminates
  error-prone wrapping
- Use Cases: FIFO queues, deques, ring buffers, sliding windows, BFS,
  undo/redo

[BinaryHeap and Priority Queues](#binaryheap-and-priority-queues)   

- Problem: Sorting after each insert O(N log N); finding min/max O(N);
  full sort for top-K wasteful
- Solution: BinaryHeap with O(log N) insert/pop, O(1) peek; Reverse for
  min-heap
- Why It Matters: 1000x faster than sorting; Dijkstra O((V+E) log V) vs
  O(V²)
- Use Cases: Task scheduling, event simulation, pathfinding, top-K,
  deadline scheduling

[Graph Representations](#graph-representations)

- Problem: Recursive structures fight ownership; Rc verbose; wrong choice
  kills performance
- Solution: Adjacency list Vec<Vec>; adjacency matrix for dense; HashMap
  for dynamic
- Why It Matters: Sparse graph: list 400MB vs matrix 1TB; 1000x algorithm
  difference
- Use Cases: Adjacency list for social networks/sparse; matrix for
  dense/grid; HashMap for dynamic

[Trie and Radix Tree Structures](#trie-and-radix-tree-structures)

- Problem: HashMap prefix search O(N); autocomplete checks all words;
  shared prefixes waste memory
- Solution: Trie with O(M) prefix search where M = prefix length; radix
  tree compresses
- Why It Matters: 10,000x faster autocomplete (O(M) vs O(N)); IP routing
  O(32) vs O(N)
- Use Cases: Autocomplete, spell check, IP routing, phonebook, DNA
  matching, compression

[Lock-Free Data Structures](#lock-free-data-structures)

- Problem: Mutex serializes all access; 80% time waiting; deadlocks;
  priority inversion
- Solution: Atomic operations with CAS loops; crossbeam queues; Arc for
  sharing
- Why It Matters: True parallelism: 8 cores → 8x vs 1x with Mutex;
  100-1000x better under contention
- Use Cases: MPMC queues, atomic counters, lock-free stacks, concurrent
  maps, real-time systems


## Overview
This chapter explores advanced collection types and data structures beyond the standard Vec and HashMap. We'll cover double-ended queues, priority queues, graph representations, prefix trees, and lock-free concurrent data structures through practical, real-world examples.



## Pattern 1: VecDeque and Ring Buffers

**Problem**: Vec only supports O(1) operations at one end—`push_front()` requires shifting all elements making it O(N). Implementing queues (FIFO) with Vec is inefficient: either `pop(0)` is O(N) or reversing is needed. Ring buffers with Vec require manual index wrapping. Sliding window algorithms with Vec allocate for every window position.

**Solution**: Use `VecDeque<T>` which maintains a ring buffer internally with head/tail pointers. Operations `push_front()`, `push_back()`, `pop_front()`, `pop_back()` are all O(1). Access elements by index in O(1). Use as circular buffer by limiting capacity. Leverage for sliding windows, task queues, LRU caches, and breadth-first search.

**Why It Matters**: VecDeque enables efficient double-ended operations impossible with Vec. A task queue processing 1M items: Vec with `remove(0)` is O(N) per operation = O(N²) total. VecDeque is O(1) per operation = O(N) total—1000x faster. Sliding windows, undo/redo stacks, and BFS all benefit. Ring buffer implementations are built-in instead of error-prone manual wrapping.

**Use Cases**: FIFO queues (task processing, message queues), deques (double-ended queues), ring buffers (audio/video streaming, fixed-size logs), sliding windows (moving averages, pattern matching), BFS traversal, undo/redo stacks.

### Example: Task Queue with Priority Lanes

 A multi-lane task queue where tasks can be added and removed from both ends efficiently.

```rust
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
        fn remove_from_queue(queue: &mut VecDeque<Task>, id: u64) -> Option<Task> {
            let pos = queue.iter().position(|t| t.id == id)?;
            queue.remove(pos)
        }

        remove_from_queue(&mut self.high_priority, id)
            .or_else(|| remove_from_queue(&mut self.normal_priority, id))
            .or_else(|| remove_from_queue(&mut self.low_priority, id))
    }

    fn len(&self) -> usize {
        self.high_priority.len() + self.normal_priority.len() + self.low_priority.len()
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

//==============
// Example usage
//==============
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
}
```

**Key VecDeque Operations**:
- `push_front()` / `push_back()`: O(1) insertion at either end
- `pop_front()` / `pop_back()`: O(1) removal from either end
- `front()` / `back()`: O(1) peek at either end
- Random access: O(1) with indexing

---

### Example: Ring Buffer for Real-Time Data

A fixed-size circular buffer that overwrites oldest data when full, commonly used for sensor data, logging, and audio processing.


```rust
use std::collections::VecDeque;

struct RingBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn push(&mut self, item: T) {
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front(); // Remove oldest
        }
        self.buffer.push_back(item);
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.buffer.get(index)
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.buffer.iter()
    }

    fn len(&self) -> usize {
        self.buffer.len()
    }

    fn is_full(&self) -> bool {
        self.buffer.len() >= self.capacity
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn as_slice(&self) -> (&[T], &[T]) {
        self.buffer.as_slices()
    }
}

//=======================================
// Specialized: Sliding window statistics
//=======================================
struct SlidingWindowStats {
    buffer: RingBuffer<f64>,
}

impl SlidingWindowStats {
    fn new(window_size: usize) -> Self {
        Self {
            buffer: RingBuffer::new(window_size),
        }
    }

    fn add(&mut self, value: f64) {
        self.buffer.push(value);
    }

    fn mean(&self) -> Option<f64> {
        if self.buffer.len() == 0 {
            return None;
        }

        let sum: f64 = self.buffer.iter().sum();
        Some(sum / self.buffer.len() as f64)
    }

    fn min(&self) -> Option<f64> {
        self.buffer.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn max(&self) -> Option<f64> {
        self.buffer.iter().copied().max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn variance(&self) -> Option<f64> {
        if self.buffer.len() < 2 {
            return None;
        }

        let mean = self.mean()?;
        let sum_squared_diff: f64 = self.buffer
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum();

        Some(sum_squared_diff / self.buffer.len() as f64)
    }

    fn std_dev(&self) -> Option<f64> {
        self.variance().map(|v| v.sqrt())
    }
}

//========================================
// Real-world example: Audio sample buffer
//========================================
struct AudioBuffer {
    samples: RingBuffer<f32>,
    sample_rate: u32,
}

impl AudioBuffer {
    fn new(duration_seconds: f32, sample_rate: u32) -> Self {
        let capacity = (duration_seconds * sample_rate as f32) as usize;
        Self {
            samples: RingBuffer::new(capacity),
            sample_rate,
        }
    }

    fn add_sample(&mut self, sample: f32) {
        self.samples.push(sample);
    }

    fn add_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.add_sample(sample);
        }
    }

    fn rms(&self) -> f32 {
        if self.samples.len() == 0 {
            return 0.0;
        }

        let sum_squares: f32 = self.samples.iter().map(|&s| s * s).sum();
        (sum_squares / self.samples.len() as f32).sqrt()
    }

    fn peak(&self) -> f32 {
        self.samples
            .iter()
            .map(|&s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    fn zero_crossing_rate(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        let mut crossings = 0;
        let samples: Vec<_> = self.samples.iter().copied().collect();

        for i in 0..samples.len() - 1 {
            if (samples[i] >= 0.0 && samples[i + 1] < 0.0)
                || (samples[i] < 0.0 && samples[i + 1] >= 0.0)
            {
                crossings += 1;
            }
        }

        crossings as f32 / (samples.len() - 1) as f32
    }
}

//==============
// Example usage
//==============
fn main() {
    println!("=== Sliding Window Stats ===\n");

    let mut stats = SlidingWindowStats::new(5);

    for value in [10.0, 20.0, 15.0, 25.0, 30.0, 18.0, 22.0] {
        stats.add(value);
        println!("Added {}: mean={:.2}, std_dev={:.2}",
                 value,
                 stats.mean().unwrap_or(0.0),
                 stats.std_dev().unwrap_or(0.0));
    }

    println!("\n=== Audio Buffer ===\n");

    let mut audio = AudioBuffer::new(0.1, 44100); // 100ms buffer at 44.1kHz

    // Simulate sine wave
    for i in 0..4410 {
        let t = i as f32 / 44100.0;
        let sample = (2.0 * std::f32::consts::PI * 440.0 * t).sin(); // 440 Hz
        audio.add_sample(sample * 0.5); // 50% amplitude
    }

    println!("RMS: {:.4}", audio.rms());
    println!("Peak: {:.4}", audio.peak());
    println!("Zero crossing rate: {:.4}", audio.zero_crossing_rate());
}
```

**Ring Buffer Use Cases**:
- Sensor data buffering
- Audio/video processing
- Network packet buffering
- Undo/redo history (fixed size)
- Performance monitoring (sliding window)

---

### Example: Deque-Based Sliding Window Maximum

Find the maximum value in every sliding window of size k in an array efficiently (O(n) time).

```rust
use std::collections::VecDeque;

struct SlidingWindowMax {
    deque: VecDeque<(usize, i32)>, // (index, value)
    window_size: usize,
}

impl SlidingWindowMax {
    fn new(window_size: usize) -> Self {
        Self {
            deque: VecDeque::new(),
            window_size,
        }
    }

    fn add(&mut self, index: usize, value: i32) -> Option<i32> {
        // Remove elements outside window
        while let Some(&(idx, _)) = self.deque.front() {
            if idx + self.window_size <= index {
                self.deque.pop_front();
            } else {
                break;
            }
        }

        // Remove elements smaller than current
        while let Some(&(_, val)) = self.deque.back() {
            if val <= value {
                self.deque.pop_back();
            } else {
                break;
            }
        }

        self.deque.push_back((index, value));

        // Return max if window is full
        if index >= self.window_size - 1 {
            self.deque.front().map(|(_, val)| *val)
        } else {
            None
        }
    }

    fn max_in_windows(arr: &[i32], k: usize) -> Vec<i32> {
        let mut solver = Self::new(k);
        let mut result = Vec::new();

        for (i, &val) in arr.iter().enumerate() {
            if let Some(max) = solver.add(i, val) {
                result.push(max);
            }
        }

        result
    }
}

//=============================================
// Real-world application: Stock price analysis
//=============================================
struct StockAnalyzer {
    prices: Vec<f64>,
}

impl StockAnalyzer {
    fn new(prices: Vec<f64>) -> Self {
        Self { prices }
    }

    fn resistance_levels(&self, window_size: usize) -> Vec<f64> {
        self.sliding_max(window_size)
    }

    fn support_levels(&self, window_size: usize) -> Vec<f64> {
        self.sliding_min(window_size)
    }

    fn sliding_max(&self, window_size: usize) -> Vec<f64> {
        let mut deque = VecDeque::new();
        let mut result = Vec::new();

        for (i, &price) in self.prices.iter().enumerate() {
            // Remove old elements
            while let Some(&idx) = deque.front() {
                if idx + window_size <= i {
                    deque.pop_front();
                } else {
                    break;
                }
            }

            // Maintain decreasing order
            while let Some(&idx) = deque.back() {
                if self.prices[idx] <= price {
                    deque.pop_back();
                } else {
                    break;
                }
            }

            deque.push_back(i);

            if i >= window_size - 1 {
                result.push(self.prices[*deque.front().unwrap()]);
            }
        }

        result
    }

    fn sliding_min(&self, window_size: usize) -> Vec<f64> {
        let mut deque = VecDeque::new();
        let mut result = Vec::new();

        for (i, &price) in self.prices.iter().enumerate() {
            while let Some(&idx) = deque.front() {
                if idx + window_size <= i {
                    deque.pop_front();
                } else {
                    break;
                }
            }

            // Maintain increasing order (opposite of max)
            while let Some(&idx) = deque.back() {
                if self.prices[idx] >= price {
                    deque.pop_back();
                } else {
                    break;
                }
            }

            deque.push_back(i);

            if i >= window_size - 1 {
                result.push(self.prices[*deque.front().unwrap()]);
            }
        }

        result
    }

    fn volatility(&self, window_size: usize) -> Vec<f64> {
        let max_values = self.sliding_max(window_size);
        let min_values = self.sliding_min(window_size);

        max_values
            .iter()
            .zip(min_values.iter())
            .map(|(max, min)| max - min)
            .collect()
    }
}

fn main() {
    println!("=== Sliding Window Maximum ===\n");

    let arr = vec![1, 3, -1, -3, 5, 3, 6, 7];
    let k = 3;

    let result = SlidingWindowMax::max_in_windows(&arr, k);
    println!("Array: {:?}", arr);
    println!("Window size: {}", k);
    println!("Maximums: {:?}", result);

    println!("\n=== Stock Analysis ===\n");

    let prices = vec![
        100.0, 102.0, 101.0, 105.0, 103.0, 108.0, 107.0, 110.0, 109.0, 112.0,
    ];

    let analyzer = StockAnalyzer::new(prices.clone());

    println!("Prices: {:?}", prices);
    println!("\nResistance (5-day high): {:?}", analyzer.resistance_levels(5));
    println!("Support (5-day low): {:?}", analyzer.support_levels(5));
    println!("Volatility (5-day range): {:?}", analyzer.volatility(5));
}
```

**Algorithm Complexity**:
- Time: O(n) - each element added/removed at most once
- Space: O(k) - deque size bounded by window size
- Better than naive O(n*k) approach

---

## Pattern 2: BinaryHeap and Priority Queues

**Problem**: Maintaining a sorted collection with frequent insertions is expensive—sorting after each insert is O(N log N). Finding the min/max element in unsorted Vec is O(N). Priority-based task scheduling requires efficiently extracting highest-priority item. Top-K problems need partial sorting but full sort wastes work. Event scheduling requires ordered timestamp processing.

**Solution**: Use `BinaryHeap<T>` which implements a max-heap: O(log N) insertion, O(log N) pop of maximum, O(1) peek at maximum. Wrap values in `Reverse<T>` for min-heap behavior. Use `peek()` to check top without removal. Leverage for priority queues, event scheduling, top-K algorithms (with fixed-size heap), and Dijkstra's shortest path.

**Why It Matters**: BinaryHeap provides optimal performance for priority operations. Task scheduler with 10K tasks: sorting after each insert = O(N log N) per insert. BinaryHeap = O(log N) per insert—1000x faster. Dijkstra's algorithm with BinaryHeap is O((V+E) log V), sorting-based is O(V²). Top-K with size-K heap uses O(K) memory vs O(N) for full sort. Event simulation with millions of events becomes tractable.

**Use Cases**: Priority task scheduling, event simulation (process by timestamp), Dijkstra/A* pathfinding, top-K element finding (median, percentiles), merge K sorted lists, deadline scheduling, rate limiting.

### Example: Task Scheduler with Deadlines

Schedule tasks based on priority and deadlines, ensuring high-priority tasks are executed first.

```rust
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

    fn schedule(&mut self, description: String, priority: u32, deadline: u64, duration: u32) {
        let task = Task {
            id: self.next_id,
            priority,
            deadline,
            duration,
            description,
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
}
```

**BinaryHeap Characteristics**:
- Max-heap by default (largest element at top)
- O(log n) push and pop
- O(1) peek
- Good for: priority queues, event scheduling, top-k problems

---

### Example: K-way Merge and Median Tracking
 Merge k sorted lists efficiently, and track the median of a stream of numbers

```rust
use std::collections::BinaryHeap;
use std::cmp::{Ordering, Reverse};

//======================================
// K-way merge: merge k sorted iterators
//======================================
struct KWayMerge<T> {
    heap: BinaryHeap<MergeItem<T>>,
}

struct MergeItem<T> {
    value: T,
    source_id: usize,
}

impl<T: Ord> Ord for MergeItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap behavior
        other.value.cmp(&self.value)
    }
}

impl<T: Ord> PartialOrd for MergeItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Eq for MergeItem<T> {}

impl<T: Ord> PartialEq for MergeItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Ord + Clone> KWayMerge<T> {
    fn merge(lists: Vec<Vec<T>>) -> Vec<T> {
        let mut heap = BinaryHeap::new();
        let mut iters: Vec<_> = lists.into_iter().map(|v| v.into_iter()).collect();

        // Initialize heap with first element from each list
        for (id, iter) in iters.iter_mut().enumerate() {
            if let Some(value) = iter.next() {
                heap.push(MergeItem {
                    value,
                    source_id: id,
                });
            }
        }

        let mut result = Vec::new();

        while let Some(item) = heap.pop() {
            result.push(item.value);

            // Get next element from same source
            if let Some(value) = iters[item.source_id].next() {
                heap.push(MergeItem {
                    value,
                    source_id: item.source_id,
                });
            }
        }

        result
    }
}

//=======================================
// Running median tracker using two heaps
//=======================================
struct MedianTracker {
    lower_half: BinaryHeap<i32>,              // max-heap
    upper_half: BinaryHeap<Reverse<i32>>,     // min-heap
}

impl MedianTracker {
    fn new() -> Self {
        Self {
            lower_half: BinaryHeap::new(),
            upper_half: BinaryHeap::new(),
        }
    }

    fn add(&mut self, num: i32) {
        // Add to appropriate heap
        if self.lower_half.is_empty() || num <= *self.lower_half.peek().unwrap() {
            self.lower_half.push(num);
        } else {
            self.upper_half.push(Reverse(num));
        }

        // Rebalance: ensure size difference <= 1
        if self.lower_half.len() > self.upper_half.len() + 1 {
            if let Some(val) = self.lower_half.pop() {
                self.upper_half.push(Reverse(val));
            }
        } else if self.upper_half.len() > self.lower_half.len() {
            if let Some(Reverse(val)) = self.upper_half.pop() {
                self.lower_half.push(val);
            }
        }
    }

    fn median(&self) -> Option<f64> {
        if self.lower_half.is_empty() && self.upper_half.is_empty() {
            return None;
        }

        if self.lower_half.len() > self.upper_half.len() {
            Some(*self.lower_half.peek().unwrap() as f64)
        } else if self.upper_half.len() > self.lower_half.len() {
            Some(self.upper_half.peek().unwrap().0 as f64)
        } else {
            let lower = *self.lower_half.peek().unwrap() as f64;
            let upper = self.upper_half.peek().unwrap().0 as f64;
            Some((lower + upper) / 2.0)
        }
    }

    fn count(&self) -> usize {
        self.lower_half.len() + self.upper_half.len()
    }
}

//==========================================
// Real-world: External sort for large files
//==========================================
struct ExternalSorter {
    chunk_size: usize,
}

impl ExternalSorter {
    fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    fn sort(&self, data: Vec<i32>) -> Vec<i32> {
        // Phase 1: Sort chunks
        let mut chunks: Vec<Vec<i32>> = data
            .chunks(self.chunk_size)
            .map(|chunk| {
                let mut sorted = chunk.to_vec();
                sorted.sort();
                sorted
            })
            .collect();

        // Phase 2: K-way merge
        KWayMerge::merge(chunks)
    }
}

fn main() {
    println!("=== K-Way Merge ===\n");

    let lists = vec![
        vec![1, 4, 7, 10],
        vec![2, 5, 8, 11],
        vec![3, 6, 9, 12],
    ];

    let merged = KWayMerge::merge(lists.clone());
    println!("Input lists: {:?}", lists);
    println!("Merged: {:?}", merged);

    println!("\n=== Running Median ===\n");

    let mut tracker = MedianTracker::new();

    for num in [5, 15, 1, 3, 8, 7, 9, 2] {
        tracker.add(num);
        println!("Added {}: median = {:.1}", num, tracker.median().unwrap());
    }

    println!("\n=== External Sort ===\n");

    let data: Vec<i32> = (0..20).rev().collect();
    println!("Unsorted: {:?}", data);

    let sorter = ExternalSorter::new(5);
    let sorted = sorter.sort(data);
    println!("Sorted: {:?}", sorted);
}
```

**Median Tracker Analysis**:
- Time: O(log n) per insertion
- Space: O(n)
- Works by maintaining two heaps: max-heap (lower half) and min-heap (upper half)
- Median is either top of one heap or average of both tops

---

### Example: Top-K Frequent Elements

Find the k most frequent elements in a stream efficiently.

```rust
use std::collections::{HashMap, BinaryHeap};
use std::cmp::{Ordering, Reverse};

#[derive(Eq, PartialEq)]
struct FreqItem<T> {
    item: T,
    count: usize,
}

impl<T: Eq> Ord for FreqItem<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl<T: Eq> PartialOrd for FreqItem<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct TopKFrequent<T> {
    counts: HashMap<T, usize>,
    k: usize,
}

impl<T> TopKFrequent<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    fn new(k: usize) -> Self {
        Self {
            counts: HashMap::new(),
            k,
        }
    }

    fn add(&mut self, item: T) {
        *self.counts.entry(item).or_insert(0) += 1;
    }

    fn add_batch(&mut self, items: Vec<T>) {
        for item in items {
            self.add(item);
        }
    }

    fn top_k(&self) -> Vec<(T, usize)> {
        // Use min-heap to keep only top k
        let mut heap: BinaryHeap<Reverse<FreqItem<&T>>> = BinaryHeap::new();

        for (item, &count) in &self.counts {
            heap.push(Reverse(FreqItem { item, count }));

            if heap.len() > self.k {
                heap.pop();
            }
        }

        heap.into_iter()
            .map(|Reverse(freq_item)| (freq_item.item.clone(), freq_item.count))
            .collect()
    }

    fn top_k_sorted(&self) -> Vec<(T, usize)> {
        let mut result = self.top_k();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

//=========================
// Real-world: Log analysis
//=========================
struct LogAnalyzer {
    error_tracker: TopKFrequent<String>,
    ip_tracker: TopKFrequent<String>,
    endpoint_tracker: TopKFrequent<String>,
}

impl LogAnalyzer {
    fn new(k: usize) -> Self {
        Self {
            error_tracker: TopKFrequent::new(k),
            ip_tracker: TopKFrequent::new(k),
            endpoint_tracker: TopKFrequent::new(k),
        }
    }

    fn process_log(&mut self, log_entry: LogEntry) {
        if let Some(error) = log_entry.error {
            self.error_tracker.add(error);
        }
        self.ip_tracker.add(log_entry.ip);
        self.endpoint_tracker.add(log_entry.endpoint);
    }

    fn report(&self) {
        println!("Top Errors:");
        for (error, count) in self.error_tracker.top_k_sorted() {
            println!("  {}: {}", error, count);
        }

        println!("\nTop IP Addresses:");
        for (ip, count) in self.ip_tracker.top_k_sorted() {
            println!("  {}: {}", ip, count);
        }

        println!("\nTop Endpoints:");
        for (endpoint, count) in self.endpoint_tracker.top_k_sorted() {
            println!("  {}: {}", endpoint, count);
        }
    }
}

#[derive(Debug, Clone)]
struct LogEntry {
    ip: String,
    endpoint: String,
    error: Option<String>,
}

fn main() {
    println!("=== Top-K Frequent Elements ===\n");

    let mut tracker = TopKFrequent::new(3);

    let words = vec![
        "apple", "banana", "apple", "cherry", "banana", "apple",
        "date", "banana", "apple", "cherry",
    ];

    tracker.add_batch(words.iter().map(|&s| s.to_string()).collect());

    println!("Top 3 words:");
    for (word, count) in tracker.top_k_sorted() {
        println!("  {}: {}", word, count);
    }

    println!("\n=== Log Analysis ===\n");

    let mut analyzer = LogAnalyzer::new(3);

    // Simulate logs
    let logs = vec![
        LogEntry {
            ip: "192.168.1.1".to_string(),
            endpoint: "/api/users".to_string(),
            error: None,
        },
        LogEntry {
            ip: "192.168.1.2".to_string(),
            endpoint: "/api/posts".to_string(),
            error: Some("404 Not Found".to_string()),
        },
        LogEntry {
            ip: "192.168.1.1".to_string(),
            endpoint: "/api/users".to_string(),
            error: None,
        },
        LogEntry {
            ip: "192.168.1.3".to_string(),
            endpoint: "/api/posts".to_string(),
            error: Some("500 Internal Error".to_string()),
        },
        LogEntry {
            ip: "192.168.1.1".to_string(),
            endpoint: "/api/comments".to_string(),
            error: Some("404 Not Found".to_string()),
        },
    ];

    for log in logs {
        analyzer.process_log(log);
    }

    analyzer.report();
}
```

**Top-K Pattern**:
- Maintain min-heap of size k
- For each element, add to heap and remove smallest if size > k
- Time: O(n log k) vs O(n log n) for full sort
- Space: O(k) for heap vs O(n) for sorting all elements

---

## Pattern 3: Graph Representations

**Problem**: Naive graph implementations with recursive structures hit Rust's ownership rules—nodes can't mutually reference each other without causing cycles. Using `Rc<RefCell<Node>>` everywhere is verbose and has runtime overhead. Dense graphs with adjacency matrices waste O(V²) memory when edges are sparse. Edge-list representations make neighbor queries O(E). Choosing wrong representation kills algorithm performance.

**Solution**: Use adjacency list with `Vec<Vec<usize>>` (node IDs as indices) for most graphs. Use adjacency matrix `Vec<Vec<bool>>` for dense graphs or when edge checks must be O(1). Use `HashMap<NodeId, Vec<NodeId>>` for dynamic graphs. Represent edges as separate array with node indices. Choose based on: graph density (sparse vs dense), operation patterns (neighbor queries vs edge checks), and mutability needs.

**Why It Matters**: Graph representation determines algorithm performance. Dijkstra's with adjacency list: O((V+E) log V). With adjacency matrix: O(V²). For sparse graphs (E << V²), this is 1000x difference. Social network with 1M users, 50M friendships: adjacency list uses 400MB, matrix uses 1TB. Ownership-based designs avoid runtime RefCell checks. Wrong choice makes simple algorithms intractable.

**Use Cases**: Adjacency list for social networks, dependency graphs, road networks (sparse). Adjacency matrix for complete graphs, grid-based pathfinding, dense weighted graphs. HashMap-based for dynamic graphs (adding/removing nodes), unknown node sets.

### Example: Adjacency List with Weighted Edges

A graph with weighted edges for algorithms like Dijkstra's shortest path.

```rust
use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug, Clone)]
struct Edge<T> {
    to: T,
    weight: u32,
}

struct WeightedGraph<T> {
    adjacency: HashMap<T, Vec<Edge<T>>>,
    directed: bool,
}

impl<T> WeightedGraph<T>
where
    T: Eq + Hash + Clone,
{
    fn new(directed: bool) -> Self {
        Self {
            adjacency: HashMap::new(),
            directed,
        }
    }

    fn add_vertex(&mut self, vertex: T) {
        self.adjacency.entry(vertex).or_insert_with(Vec::new);
    }

    fn add_edge(&mut self, from: T, to: T, weight: u32) {
        self.adjacency
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push(Edge {
                to: to.clone(),
                weight,
            });

        if !self.directed {
            self.adjacency
                .entry(to)
                .or_insert_with(Vec::new)
                .push(Edge { to: from, weight });
        }
    }

    fn neighbors(&self, vertex: &T) -> Option<&Vec<Edge<T>>> {
        self.adjacency.get(vertex)
    }

    fn vertices(&self) -> Vec<&T> {
        self.adjacency.keys().collect()
    }

    fn edge_count(&self) -> usize {
        let total: usize = self.adjacency.values().map(|edges| edges.len()).sum();
        if self.directed {
            total
        } else {
            total / 2
        }
    }
}

//=========================
// Dijkstra's shortest path
//=========================
#[derive(Eq, PartialEq)]
struct State<T> {
    cost: u32,
    node: T,
}

impl<T: Eq> Ord for State<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // Min-heap
    }
}

impl<T: Eq> PartialOrd for State<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> WeightedGraph<T>
where
    T: Eq + Hash + Clone,
{
    fn dijkstra(&self, start: &T) -> HashMap<T, u32> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(start.clone(), 0);
        heap.push(State {
            cost: 0,
            node: start.clone(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            // Skip if we found a better path
            if let Some(&best) = distances.get(&node) {
                if cost > best {
                    continue;
                }
            }

            // Check neighbors
            if let Some(edges) = self.neighbors(&node) {
                for edge in edges {
                    let next_cost = cost + edge.weight;

                    let is_better = distances
                        .get(&edge.to)
                        .map_or(true, |&current| next_cost < current);

                    if is_better {
                        distances.insert(edge.to.clone(), next_cost);
                        heap.push(State {
                            cost: next_cost,
                            node: edge.to.clone(),
                        });
                    }
                }
            }
        }

        distances
    }

    fn shortest_path(&self, start: &T, end: &T) -> Option<(Vec<T>, u32)> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut previous: HashMap<T, T> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(start.clone(), 0);
        heap.push(State {
            cost: 0,
            node: start.clone(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            if node == *end {
                // Reconstruct path
                let mut path = vec![end.clone()];
                let mut current = end;

                while let Some(prev) = previous.get(current) {
                    path.push(prev.clone());
                    current = prev;
                }

                path.reverse();
                return Some((path, cost));
            }

            if let Some(&best) = distances.get(&node) {
                if cost > best {
                    continue;
                }
            }

            if let Some(edges) = self.neighbors(&node) {
                for edge in edges {
                    let next_cost = cost + edge.weight;

                    let is_better = distances
                        .get(&edge.to)
                        .map_or(true, |&current| next_cost < current);

                    if is_better {
                        distances.insert(edge.to.clone(), next_cost);
                        previous.insert(edge.to.clone(), node.clone());
                        heap.push(State {
                            cost: next_cost,
                            node: edge.to.clone(),
                        });
                    }
                }
            }
        }

        None
    }
}

//===========================
// Real-world: Route planning
//===========================
fn main() {
    println!("=== Weighted Graph - Route Planning ===\n");

    let mut map = WeightedGraph::new(false);

    // Cities and distances (km)
    map.add_edge("SF", "LA", 383);
    map.add_edge("SF", "Portland", 635);
    map.add_edge("LA", "Phoenix", 373);
    map.add_edge("Portland", "Seattle", 173);
    map.add_edge("Phoenix", "Denver", 868);
    map.add_edge("Seattle", "Denver", 1316);
    map.add_edge("LA", "Denver", 1016);

    println!("Finding shortest paths from SF:\n");
    let distances = map.dijkstra(&"SF");

    for (city, distance) in &distances {
        println!("  SF -> {}: {}km", city, distance);
    }

    println!("\n Shortest path SF -> Denver:");
    if let Some((path, distance)) = map.shortest_path(&"SF", &"Denver") {
        println!("  Path: {:?}", path);
        println!("  Distance: {}km", distance);
    }
}
```

**Graph Representation Trade-offs**:
- **Adjacency List**: Space O(V + E), good for sparse graphs
- **Adjacency Matrix**: Space O(V²), good for dense graphs, O(1) edge lookup
- **Edge List**: Simple, good for iterating all edges

---

### Example: Topological Sort and Dependency Resolution

Order tasks respecting dependencies, detect cycles in dependency graphs.

```rust
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

struct DirectedGraph<T> {
    adjacency: HashMap<T, Vec<T>>,
}

impl<T> DirectedGraph<T>
where
    T: Eq + Hash + Clone,
{
    fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: T, to: T) {
        self.adjacency
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push(to.clone());

        // Ensure 'to' vertex exists
        self.adjacency.entry(to).or_insert_with(Vec::new);
    }

    fn vertices(&self) -> Vec<&T> {
        self.adjacency.keys().collect()
    }

    // Kahn's algorithm for topological sort
    fn topological_sort(&self) -> Result<Vec<T>, String> {
        let mut in_degree: HashMap<T, usize> = HashMap::new();

        // Calculate in-degrees
        for vertex in self.vertices() {
            in_degree.entry(vertex.clone()).or_insert(0);
        }

        for edges in self.adjacency.values() {
            for to in edges {
                *in_degree.entry(to.clone()).or_insert(0) += 1;
            }
        }

        // Queue vertices with no incoming edges
        let mut queue: VecDeque<T> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(v, _)| v.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(vertex) = queue.pop_front() {
            result.push(vertex.clone());

            // Reduce in-degree for neighbors
            if let Some(edges) = self.adjacency.get(&vertex) {
                for to in edges {
                    if let Some(degree) = in_degree.get_mut(to) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(to.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.adjacency.len() {
            Err("Graph contains a cycle".to_string())
        } else {
            Ok(result)
        }
    }

    // DFS-based topological sort
    fn topological_sort_dfs(&self) -> Result<Vec<T>, String> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut result = Vec::new();

        for vertex in self.vertices() {
            if !visited.contains(vertex) {
                self.dfs_topo(
                    vertex,
                    &mut visited,
                    &mut rec_stack,
                    &mut result,
                )?;
            }
        }

        result.reverse();
        Ok(result)
    }

    fn dfs_topo(
        &self,
        vertex: &T,
        visited: &mut HashSet<T>,
        rec_stack: &mut HashSet<T>,
        result: &mut Vec<T>,
    ) -> Result<(), String> {
        visited.insert(vertex.clone());
        rec_stack.insert(vertex.clone());

        if let Some(edges) = self.adjacency.get(vertex) {
            for neighbor in edges {
                if !visited.contains(neighbor) {
                    self.dfs_topo(neighbor, visited, rec_stack, result)?;
                } else if rec_stack.contains(neighbor) {
                    return Err(format!("Cycle detected involving {:?}", neighbor));
                }
            }
        }

        rec_stack.remove(vertex);
        result.push(vertex.clone());
        Ok(())
    }

    fn has_cycle(&self) -> bool {
        self.topological_sort().is_err()
    }
}

//===============================================
// Real-world: Build system dependency resolution
//===============================================
struct BuildSystem {
    dependencies: DirectedGraph<String>,
}

impl BuildSystem {
    fn new() -> Self {
        Self {
            dependencies: DirectedGraph::new(),
        }
    }

    fn add_target(&mut self, target: String, depends_on: Vec<String>) {
        for dep in depends_on {
            self.dependencies.add_edge(dep, target.clone());
        }
    }

    fn build_order(&self) -> Result<Vec<String>, String> {
        self.dependencies.topological_sort()
    }

    fn check_cycles(&self) -> bool {
        self.dependencies.has_cycle()
    }
}

//=========================================
// Real-world: Course prerequisite planning
//=========================================
struct CoursePlanner {
    prerequisites: DirectedGraph<String>,
}

impl CoursePlanner {
    fn new() -> Self {
        Self {
            prerequisites: DirectedGraph::new(),
        }
    }

    fn add_course(&mut self, course: String, prerequisites: Vec<String>) {
        for prereq in prerequisites {
            self.prerequisites.add_edge(prereq, course.clone());
        }
    }

    fn course_order(&self) -> Result<Vec<String>, String> {
        self.prerequisites.topological_sort()
    }

    fn can_complete(&self) -> bool {
        !self.prerequisites.has_cycle()
    }
}

fn main() {
    println!("=== Build System ===\n");

    let mut build = BuildSystem::new();

    build.add_target("main.o".to_string(), vec!["main.c".to_string(), "util.h".to_string()]);
    build.add_target("util.o".to_string(), vec!["util.c".to_string(), "util.h".to_string()]);
    build.add_target("program".to_string(), vec!["main.o".to_string(), "util.o".to_string()]);

    match build.build_order() {
        Ok(order) => {
            println!("Build order:");
            for (i, target) in order.iter().enumerate() {
                println!("  {}. {}", i + 1, target);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Course Planning ===\n");

    let mut planner = CoursePlanner::new();

    planner.add_course("Data Structures".to_string(), vec!["Programming 101".to_string()]);
    planner.add_course("Algorithms".to_string(), vec!["Data Structures".to_string()]);
    planner.add_course("AI".to_string(), vec!["Algorithms".to_string(), "Linear Algebra".to_string()]);
    planner.add_course("Machine Learning".to_string(), vec!["AI".to_string(), "Statistics".to_string()]);

    if planner.can_complete() {
        match planner.course_order() {
            Ok(order) => {
                println!("Suggested course order:");
                for (i, course) in order.iter().enumerate() {
                    println!("  Semester {}: {}", (i / 2) + 1, course);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Cannot complete - circular prerequisites!");
    }
}
```

**Topological Sort Algorithms**:
1. **Kahn's Algorithm** (BFS-based):
   - Time: O(V + E)
   - Easier to detect cycles
   - Produces one valid ordering

2. **DFS-based**:
   - Time: O(V + E)
   - Can find strongly connected components
   - Multiple valid orderings possible

---

## Pattern 4: Trie and Radix Tree Structures

**Problem**: Finding all strings with a given prefix in HashMap requires checking every key—O(N) with N strings. Autocomplete for 1M words checks all 1M. Longest common prefix requires pairwise comparisons. IP routing tables need longest prefix matching. HashSet can't efficiently answer "words starting with 'pre'". Storing dictionary with shared prefixes wastes memory ("pre", "prefix", "preview" store "pre" three times).

**Solution**: Use Trie (prefix tree) where each node represents a character, paths from root spell strings. Prefix search is O(M) where M is prefix length, not number of strings. Radix tree (compressed trie) merges single-child chains to reduce nodes. Use for autocomplete, spell checking, IP routing, dictionary compression. Query all strings with prefix by traversing to prefix node then collecting subtree.

**Why It Matters**: Tries enable efficient prefix operations impossible with hash tables. Autocomplete in 1M-word dictionary: HashMap O(N) scan per query. Trie O(M) where M is typed prefix—10,000x faster for 10-character prefix. IP routing with 500K routes: linear scan is O(N), trie longest-prefix-match is O(32) for IPv4. Memory: compressed trie shares common prefixes—"antiestablishmentarianism" variations stored once. Dictionary apps, autocomplete, routers all rely on tries.

**Use Cases**: Autocomplete (search engines, IDEs, command completion), spell checkers (dictionary lookup, suggestions), IP routing (longest prefix match), phonebook search by prefix, DNA sequence matching, text compression (shared prefix storage).

### Example: Trie for Autocomplete and Prefix Search

Autocomplete with fast prefix matching.

```rust
use std::collections::HashMap;

#[derive(Default, Debug)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end: bool,
    count: usize, // Frequency/popularity
}

struct Trie {
    root: TrieNode,
}

impl Trie {
    fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    fn insert(&mut self, word: &str) {
        self.insert_with_count(word, 1);
    }

    fn insert_with_count(&mut self, word: &str, count: usize) {
        let mut node = &mut self.root;

        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }

        node.is_end = true;
        node.count += count;
    }

    fn search(&self, word: &str) -> bool {
        self.find_node(word).map_or(false, |node| node.is_end)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    fn find_node(&self, prefix: &str) -> Option<&TrieNode> {
        let mut node = &self.root;

        for ch in prefix.chars() {
            node = node.children.get(&ch)?;
        }

        Some(node)
    }

    fn autocomplete(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();

        if let Some(node) = self.find_node(prefix) {
            self.collect_words(node, prefix.to_string(), &mut results);
        }

        results
    }

    fn collect_words(&self, node: &TrieNode, current: String, results: &mut Vec<String>) {
        if node.is_end {
            results.push(current.clone());
        }

        for (&ch, child) in &node.children {
            let mut next = current.clone();
            next.push(ch);
            self.collect_words(child, next, results);
        }
    }

    fn top_k_autocomplete(&self, prefix: &str, k: usize) -> Vec<(String, usize)> {
        let mut results = Vec::new();

        if let Some(node) = self.find_node(prefix) {
            self.collect_words_with_count(node, prefix.to_string(), &mut results);
        }

        // Sort by count (descending) and take top k
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(k);
        results
    }

    fn collect_words_with_count(
        &self,
        node: &TrieNode,
        current: String,
        results: &mut Vec<(String, usize)>,
    ) {
        if node.is_end {
            results.push((current.clone(), node.count));
        }

        for (&ch, child) in &node.children {
            let mut next = current.clone();
            next.push(ch);
            self.collect_words_with_count(child, next, results);
        }
    }

    fn delete(&mut self, word: &str) -> bool {
        self.delete_helper(&mut self.root, word, 0)
    }

    fn delete_helper(&mut self, node: &mut TrieNode, word: &str, index: usize) -> bool {
        if index == word.len() {
            if !node.is_end {
                return false;
            }
            node.is_end = false;
            return node.children.is_empty();
        }

        let ch = word.chars().nth(index).unwrap();

        if let Some(child) = node.children.get_mut(&ch) {
            let should_delete = self.delete_helper(child, word, index + 1);

            if should_delete {
                node.children.remove(&ch);
                return !node.is_end && node.children.is_empty();
            }
        }

        false
    }
}

//=======================================
// Real-world: Search engine autocomplete
//=======================================
struct SearchAutocomplete {
    trie: Trie,
}

impl SearchAutocomplete {
    fn new() -> Self {
        Self {
            trie: Trie::new(),
        }
    }

    fn add_search_query(&mut self, query: &str) {
        // Normalize: lowercase
        let normalized = query.to_lowercase();
        self.trie.insert_with_count(&normalized, 1);
    }

    fn suggest(&self, prefix: &str, limit: usize) -> Vec<(String, usize)> {
        let normalized = prefix.to_lowercase();
        self.trie.top_k_autocomplete(&normalized, limit)
    }
}

//========================================
// Real-world: Dictionary with spell check
//========================================
struct Dictionary {
    trie: Trie,
}

impl Dictionary {
    fn new() -> Self {
        Self {
            trie: Trie::new(),
        }
    }

    fn add_word(&mut self, word: &str) {
        self.trie.insert(&word.to_lowercase());
    }

    fn contains(&self, word: &str) -> bool {
        self.trie.search(&word.to_lowercase())
    }

    fn suggest_corrections(&self, word: &str, max_suggestions: usize) -> Vec<String> {
        let word = word.to_lowercase();

        // Try prefixes of increasing length
        for len in (1..=word.len()).rev() {
            let prefix = &word[..len];
            let suggestions = self.trie.autocomplete(prefix);

            if !suggestions.is_empty() {
                let mut results: Vec<_> = suggestions
                    .into_iter()
                    .filter(|s| self.edit_distance(s, &word) <= 2)
                    .collect();

                results.truncate(max_suggestions);

                if !results.is_empty() {
                    return results;
                }
            }
        }

        vec![]
    }

    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        let mut dp = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };

                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        dp[len1][len2]
    }
}

fn main() {
    println!("=== Autocomplete ===\n");

    let mut autocomplete = SearchAutocomplete::new();

    // Simulate search queries
    autocomplete.add_search_query("rust programming");
    autocomplete.add_search_query("rust tutorial");
    autocomplete.add_search_query("rust tutorial");
    autocomplete.add_search_query("rust book");
    autocomplete.add_search_query("python programming");

    println!("Suggestions for 'rust':");
    for (query, count) in autocomplete.suggest("rust", 5) {
        println!("  {} (searched {} times)", query, count);
    }

    println!("\n=== Dictionary ===\n");

    let mut dict = Dictionary::new();

    for word in ["hello", "help", "helper", "world", "word", "work"] {
        dict.add_word(word);
    }

    let test_word = "helo";
    println!("Is '{}' in dictionary? {}", test_word, dict.contains(test_word));

    println!("Suggestions for '{}':", test_word);
    for suggestion in dict.suggest_corrections(test_word, 3) {
        println!("  {}", suggestion);
    }
}
```

**Trie Complexity**:
- Insert: O(m) where m = word length
- Search: O(m)
- Space: O(ALPHABET_SIZE * N * M) worst case
- Ideal for: autocomplete, spell check, IP routing

---

### Example: Radix Tree for Compressed Trie

A space-efficient radix tree (compressed trie) for storing strings with common prefixes.

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct RadixNode {
    children: HashMap<char, Box<RadixNode>>,
    edge_label: String,
    is_end: bool,
    value: Option<String>,
}

impl RadixNode {
    fn new(label: String) -> Self {
        Self {
            children: HashMap::new(),
            edge_label: label,
            is_end: false,
            value: None,
        }
    }
}

struct RadixTree {
    root: RadixNode,
    size: usize,
}

impl RadixTree {
    fn new() -> Self {
        Self {
            root: RadixNode::new(String::new()),
            size: 0,
        }
    }

    fn insert(&mut self, key: &str, value: String) {
        if key.is_empty() {
            return;
        }

        self.insert_helper(&mut self.root, key, value);
        self.size += 1;
    }

    fn insert_helper(&mut self, node: &mut RadixNode, key: &str, value: String) {
        if key.is_empty() {
            node.is_end = true;
            node.value = Some(value);
            return;
        }

        let first_char = key.chars().next().unwrap();

        // Find matching child
        if let Some(child) = node.children.get_mut(&first_char) {
            let label = &child.edge_label;
            let common_prefix_len = common_prefix_length(key, label);

            if common_prefix_len == label.len() {
                // Full match: continue down
                let remaining = &key[common_prefix_len..];
                self.insert_helper(child, remaining, value);
            } else {
                // Partial match: split node
                let old_label = label.clone();
                let common = &old_label[..common_prefix_len];
                let old_suffix = &old_label[common_prefix_len..];
                let new_suffix = &key[common_prefix_len..];

                // Create new intermediate node
                let mut intermediate = Box::new(RadixNode::new(common.to_string()));

                // Move old child under intermediate
                let old_child = node.children.remove(&first_char).unwrap();
                let old_first = old_suffix.chars().next().unwrap();

                let mut relocated = old_child;
                relocated.edge_label = old_suffix.to_string();
                intermediate.children.insert(old_first, relocated);

                // Add new branch
                if !new_suffix.is_empty() {
                    let new_first = new_suffix.chars().next().unwrap();
                    let mut new_node = Box::new(RadixNode::new(new_suffix.to_string()));
                    new_node.is_end = true;
                    new_node.value = Some(value);
                    intermediate.children.insert(new_first, new_node);
                } else {
                    intermediate.is_end = true;
                    intermediate.value = Some(value);
                }

                node.children.insert(first_char, intermediate);
            }
        } else {
            // No matching child: create new
            let mut new_node = Box::new(RadixNode::new(key.to_string()));
            new_node.is_end = true;
            new_node.value = Some(value);
            node.children.insert(first_char, new_node);
        }
    }

    fn search(&self, key: &str) -> Option<&String> {
        self.search_helper(&self.root, key)
    }

    fn search_helper(&self, node: &RadixNode, key: &str) -> Option<&String> {
        if key.is_empty() {
            return if node.is_end {
                node.value.as_ref()
            } else {
                None
            };
        }

        let first_char = key.chars().next().unwrap();

        if let Some(child) = node.children.get(&first_char) {
            let label = &child.edge_label;
            let common_len = common_prefix_length(key, label);

            if common_len == label.len() {
                let remaining = &key[common_len..];
                self.search_helper(child, remaining)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn starts_with(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        self.collect_with_prefix(&self.root, prefix, String::new(), &mut results);
        results
    }

    fn collect_with_prefix(
        &self,
        node: &RadixNode,
        remaining_prefix: &str,
        current_key: String,
        results: &mut Vec<String>,
    ) {
        if remaining_prefix.is_empty() {
            // Collect all keys under this node
            self.collect_all(node, current_key, results);
            return;
        }

        let first_char = remaining_prefix.chars().next().unwrap();

        if let Some(child) = node.children.get(&first_char) {
            let label = &child.edge_label;
            let common_len = common_prefix_length(remaining_prefix, label);

            let mut new_key = current_key.clone();
            new_key.push_str(&label[..common_len]);

            if common_len == label.len() {
                let new_remaining = &remaining_prefix[common_len..];
                self.collect_with_prefix(child, new_remaining, new_key, results);
            } else if common_len == remaining_prefix.len() {
                // Prefix matches completely
                self.collect_all(child, new_key, results);
            }
        }
    }

    fn collect_all(&self, node: &RadixNode, current_key: String, results: &mut Vec<String>) {
        if node.is_end {
            results.push(current_key.clone());
        }

        for (_, child) in &node.children {
            let mut new_key = current_key.clone();
            new_key.push_str(&child.edge_label);
            self.collect_all(child, new_key, results);
        }
    }

    fn len(&self) -> usize {
        self.size
    }
}

fn common_prefix_length(s1: &str, s2: &str) -> usize {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(a, b)| a == b)
        .count()
}

//=============================
// Real-world: IP routing table
//=============================
struct RoutingTable {
    tree: RadixTree,
}

impl RoutingTable {
    fn new() -> Self {
        Self {
            tree: RadixTree::new(),
        }
    }

    fn add_route(&mut self, cidr: &str, gateway: &str) {
        self.tree.insert(cidr, gateway.to_string());
    }

    fn lookup(&self, ip: &str) -> Option<&String> {
        self.tree.search(ip)
    }

    fn routes_for_prefix(&self, prefix: &str) -> Vec<String> {
        self.tree.starts_with(prefix)
    }
}

fn main() {
    println!("=== Radix Tree ===\n");

    let mut tree = RadixTree::new();

    tree.insert("test", "value1".to_string());
    tree.insert("testing", "value2".to_string());
    tree.insert("team", "value3".to_string());
    tree.insert("toast", "value4".to_string());

    println!("Search 'test': {:?}", tree.search("test"));
    println!("Search 'testing': {:?}", tree.search("testing"));
    println!("Search 'team': {:?}", tree.search("team"));

    println!("\nKeys starting with 'te':");
    for key in tree.starts_with("te") {
        println!("  {}", key);
    }

    println!("\n=== IP Routing Table ===\n");

    let mut routing = RoutingTable::new();

    routing.add_route("192.168.1.0", "gateway1");
    routing.add_route("192.168.2.0", "gateway2");
    routing.add_route("192.168.1.100", "gateway3");

    println!("Lookup 192.168.1.0: {:?}", routing.lookup("192.168.1.0"));
    println!("Lookup 192.168.1.100: {:?}", routing.lookup("192.168.1.100"));

    println!("\nRoutes for '192.168.1':");
    for route in routing.routes_for_prefix("192.168.1") {
        println!("  {}", route);
    }
}
```

**Radix Tree Benefits**:
- More space-efficient than trie (compressed edges)
- Fewer nodes for strings with long common prefixes
- Used in: routing tables, memory allocators, file systems
- Trade-off: more complex implementation

---

## Pattern 5: Lock-Free Data Structures

**Problem**: Mutex-based data structures serialize all access—threads wait even when operating on different elements. Lock contention causes 80% of multi-threaded time spent waiting. Priority inversion: low-priority thread holds lock, blocking high-priority thread. Deadlocks from lock ordering mistakes. Panics while holding lock poison the mutex. Real-time systems can't tolerate lock-induced latency spikes.

**Solution**: Use atomic operations (`AtomicUsize`, `AtomicBool`, etc.) for lock-free primitives. Implement lock-free algorithms with compare-and-swap (CAS) loops. Use `crossbeam::queue::ArrayQueue` for bounded SPSC/MPMC. Use `Arc` with atomic refcounts for shared ownership. Leverage memory orderings (`Acquire`, `Release`, `SeqCst`) for correct synchronization. ABA problem requires generation counters or garbage collection.

**Why It Matters**: Lock-free structures enable true parallelism. Multi-threaded counter with Mutex: serialized updates = 1 core performance. Atomic counter: linear scaling = 8 cores → 8x throughput. MPMC queue with DashMap or crossbeam: 100-1000x better than Mutex<VecDeque> under contention. Real-time audio processing requires lock-free queues to prevent dropouts. Database systems use lock-free structures for transaction processing at millions/second.

**Use Cases**: MPMC queues (work-stealing schedulers, actor systems), atomic counters (metrics, rate limiting), lock-free stacks (memory allocators), concurrent hash maps (caches, indexes), real-time systems (audio, trading), high-throughput servers.

### Example: Lock-Free Stack

A thread-safe stack without using mutexes, allowing multiple threads to push/pop concurrently.

```rust
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
            if self
                .head
                .compare_exchange(head, new_node, Ordering::Release, Ordering::Acquire)
                .is_ok()
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
                if self
                    .head
                    .compare_exchange(head, next, Ordering::Release, Ordering::Acquire)
                    .is_ok()
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
}
```

**Lock-Free Principles**:
- **Compare-and-Swap (CAS)**: Atomic operation for lock-free algorithms
- **ABA Problem**: Must be handled with epoch-based reclamation
- **Memory Ordering**: Acquire/Release semantics for correct synchronization
- **Progress Guarantee**: At least one thread makes progress

### Example: Lock-Free Queue with Crossbeam

Production-ready lock-free MPMC (Multi-Producer Multi-Consumer) queue.

```rust
//============================================
// Note: Add `crossbeam = "0.8"` to Cargo.toml
//============================================
use crossbeam::queue::{ArrayQueue, SegQueue};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

//===================
// Bounded MPMC queue
//===================
struct BoundedWorkQueue<T> {
    queue: Arc<ArrayQueue<T>>,
}

impl<T> BoundedWorkQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            queue: Arc::new(ArrayQueue::new(capacity)),
        }
    }

    fn push(&self, item: T) -> Result<(), T> {
        self.queue.push(item)
    }

    fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn is_full(&self) -> bool {
        self.queue.is_full()
    }

    fn clone_handle(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
        }
    }
}

//=====================
// Unbounded MPMC queue
//=====================
struct UnboundedWorkQueue<T> {
    queue: Arc<SegQueue<T>>,
}

impl<T> UnboundedWorkQueue<T> {
    fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    fn push(&self, item: T) {
        self.queue.push(item);
    }

    fn pop(&self) -> Option<T> {
        self.queue.pop()
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clone_handle(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
        }
    }
}

//==================================================
// Real-world: Thread pool with lock-free task queue
//==================================================
struct ThreadPool {
    task_queue: UnboundedWorkQueue<Box<dyn FnOnce() + Send + 'static>>,
    workers: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl ThreadPool {
    fn new(num_threads: usize) -> Self {
        let task_queue = UnboundedWorkQueue::new();
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut workers = Vec::new();

        for id in 0..num_threads {
            let queue_clone = task_queue.clone_handle();
            let shutdown_clone = Arc::clone(&shutdown);

            workers.push(thread::spawn(move || {
                while !shutdown_clone.load(std::sync::atomic::Ordering::Acquire) {
                    if let Some(task) = queue_clone.pop() {
                        task();
                    } else {
                        thread::sleep(Duration::from_micros(100));
                    }
                }
            }));
        }

        Self {
            task_queue,
            workers,
            shutdown,
        }
    }

    fn execute<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_queue.push(Box::new(task));
    }

    fn shutdown(self) {
        self.shutdown
            .store(true, std::sync::atomic::Ordering::Release);

        for worker in self.workers {
            worker.join().unwrap();
        }
    }
}

//==============================
// Benchmark: Lock-free vs Mutex
//==============================
use std::sync::Mutex;

fn benchmark_lockfree_vs_mutex() {
    const ITEMS: usize = 100_000;
    const THREADS: usize = 4;

    // Lock-free queue
    println!("Lock-free queue:");
    let start = Instant::now();
    let lockfree_queue = UnboundedWorkQueue::new();

    let mut producers = vec![];
    for _ in 0..THREADS {
        let queue = lockfree_queue.clone_handle();
        producers.push(thread::spawn(move || {
            for i in 0..ITEMS {
                queue.push(i);
            }
        }));
    }

    let mut consumers = vec![];
    for _ in 0..THREADS {
        let queue = lockfree_queue.clone_handle();
        consumers.push(thread::spawn(move || {
            let mut count = 0;
            loop {
                if queue.pop().is_some() {
                    count += 1;
                    if count >= ITEMS {
                        break;
                    }
                }
            }
        }));
    }

    for p in producers {
        p.join().unwrap();
    }
    for c in consumers {
        c.join().unwrap();
    }

    let lockfree_time = start.elapsed();
    println!("  Time: {:?}", lockfree_time);

    // Mutex-based queue
    println!("\nMutex-based queue:");
    let start = Instant::now();
    let mutex_queue = Arc::new(Mutex::new(std::collections::VecDeque::new()));

    let mut producers = vec![];
    for _ in 0..THREADS {
        let queue = Arc::clone(&mutex_queue);
        producers.push(thread::spawn(move || {
            for i in 0..ITEMS {
                queue.lock().unwrap().push_back(i);
            }
        }));
    }

    let mut consumers = vec![];
    for _ in 0..THREADS {
        let queue = Arc::clone(&mutex_queue);
        consumers.push(thread::spawn(move || {
            let mut count = 0;
            loop {
                if queue.lock().unwrap().pop_front().is_some() {
                    count += 1;
                    if count >= ITEMS {
                        break;
                    }
                }
            }
        }));
    }

    for p in producers {
        p.join().unwrap();
    }
    for c in consumers {
        c.join().unwrap();
    }

    let mutex_time = start.elapsed();
    println!("  Time: {:?}", mutex_time);

    println!(
        "\nSpeedup: {:.2}x",
        mutex_time.as_secs_f64() / lockfree_time.as_secs_f64()
    );
}

fn main() {
    println!("=== Lock-Free Queue ===\n");

    let queue = UnboundedWorkQueue::new();

    // Producer thread
    let producer = queue.clone_handle();
    let p = thread::spawn(move || {
        for i in 0..1000 {
            producer.push(i);
        }
    });

    // Consumer threads
    let mut consumers = vec![];
    for _ in 0..3 {
        let consumer = queue.clone_handle();
        consumers.push(thread::spawn(move || {
            let mut sum = 0;
            while let Some(val) = consumer.pop() {
                sum += val;
            }
            sum
        }));
    }

    p.join().unwrap();

    let total: i32 = consumers.into_iter().map(|h| h.join().unwrap()).sum();
    println!("Total consumed: {}", total);

    println!("\n=== Thread Pool ===\n");

    let pool = ThreadPool::new(4);

    for i in 0..10 {
        pool.execute(move || {
            println!("Task {} executing", i);
            thread::sleep(Duration::from_millis(100));
        });
    }

    thread::sleep(Duration::from_secs(2));
    pool.shutdown();

    println!("\n=== Performance Benchmark ===\n");
    benchmark_lockfree_vs_mutex();
}
```

**Lock-Free Queue Benefits**:
- **No blocking**: Threads never wait for locks
- **Better scalability**: Performance scales with cores
- **Progress guarantee**: System-wide progress even if threads are paused
- **2-10x faster** than mutex-based queues under contention

**Crossbeam Features**:
- `ArrayQueue`: Bounded MPMC, faster for fixed capacity
- `SegQueue`: Unbounded MPMC, grows dynamically
- Epoch-based memory reclamation (solves ABA problem)

---

## Summary

This chapter covered advanced collection types:

1. **VecDeque**: O(1) push/pop at both ends, ring buffers, sliding windows
2. **BinaryHeap**: Priority queues, task scheduling, top-k problems, median tracking
3. **Graphs**: Weighted edges, Dijkstra's algorithm, topological sort, dependency resolution
4. **Tries**: Autocomplete, prefix search, dictionary operations
5. **Radix Trees**: Compressed tries, IP routing, space-efficient string storage
6. **Lock-Free Structures**: CAS-based stack, MPMC queues, thread pools without locks

**Key Takeaways**:
- Choose the right collection for your access pattern
- VecDeque is ideal for queues and sliding windows
- BinaryHeap provides efficient priority-based access
- Graph representation affects algorithm performance
- Tries excel at prefix operations
- Lock-free structures enable high-concurrency scenarios

**Performance Guidelines**:
- VecDeque: O(1) amortized for push/pop at ends
- BinaryHeap: O(log n) insert/remove, O(1) peek
- Trie: O(m) operations where m = key length
- Lock-free: No blocking, better scalability under contention
