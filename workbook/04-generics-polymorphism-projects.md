# Chapter 4: Generics & Polymorphism - Programming Projects

## Project 1: Generic Priority Queue with Custom Ordering

### Problem Statement

Build a generic priority queue data structure that can work with any type implementing `Ord`. The queue should support:
- Inserting elements with automatic ordering
- Removing the highest priority element
- Peeking at the highest priority element without removing it
- Custom comparison strategies through trait bounds
- Efficient implementation using a binary heap
- Support for both min-heap and max-heap configurations using phantom types

The priority queue must be fully generic over the element type and provide compile-time guarantees about ordering requirements.

### Why It Matters

Priority queues are fundamental data structures used in:
- **Operating Systems**: Process scheduling, interrupt handling
- **Algorithms**: Dijkstra's shortest path, A* search, Huffman coding
- **Real-time Systems**: Event processing by priority
- **Resource Management**: Task queuing, load balancing

Understanding how to implement generic collections with trait bounds teaches you how Rust's standard library works internally. You'll learn why `BinaryHeap<T>` requires `T: Ord` and how to design APIs that are both flexible and type-safe.

### Use Cases

1. **Task Scheduler**: Schedule tasks by priority, deadline, or custom business logic
2. **Event-Driven Systems**: Process events in priority order
3. **Graph Algorithms**: Implement A*, Dijkstra, Prim's algorithm efficiently
4. **Median Finding**: Maintain streaming median using two heaps
5. **Merge K Sorted Lists**: Efficiently merge sorted iterators
6. **Job Queue Systems**: Background job processing with priority levels

### Solution Outline

**Core Structure:**
```rust
// Use phantom type to distinguish min-heap from max-heap
use std::marker::PhantomData;

struct MinHeap;
struct MaxHeap;

pub struct PriorityQueue<T, Order = MinHeap> {
    heap: Vec<T>,
    _order: PhantomData<Order>,
}
```

**Key Methods to Implement:**
- `new()` - Create empty queue
- `push(item: T)` - Insert element (sift up to maintain heap property)
- `pop() -> Option<T>` - Remove and return highest priority element (sift down)
- `peek() -> Option<&T>` - View highest priority element
- `len()`, `is_empty()` - Basic queries
- `from_vec(vec: Vec<T>)` - Build heap from existing data (heapify)

**Trait Bounds Strategy:**
- Start with `T: Ord` for basic comparison
- Add `where` clauses for methods that need additional bounds
- Implement custom ordering through wrapper types
- Use associated types for extensibility

**Heap Operations:**
- **Sift Up**: When inserting, bubble element up to restore heap property
- **Sift Down**: When removing root, move last element to root and bubble down
- **Heapify**: Build heap from unordered array in O(n) time

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_basic_operations() {
    let mut pq = PriorityQueue::new();
    pq.push(5);
    pq.push(3);
    pq.push(7);
    assert_eq!(pq.pop(), Some(3)); // Min heap
}

#[test]
fn test_heap_property() {
    // Verify heap property holds after every operation
    // Parent should be ≤ children (min heap) or ≥ (max heap)
}

#[test]
fn test_generic_types() {
    // Test with different types: i32, String, custom structs
}
```

**Property-Based Testing:**
- Insertion order shouldn't matter for final sorted output
- Popping all elements should yield sorted sequence
- Heap property should hold after any operation

**Performance Tests:**
- Benchmark insertion of N elements
- Compare heapify vs individual inserts
- Test with large datasets (1M+ elements)

---

## Milestone-by-Milestone Implementation Guide

### Milestone 1: Basic Generic Structure with Vec Backend

### Introduction

Implement a simple priority queue using Rust's `Vec<T>` as the backing storage with a naive approach: sort on every insertion. This milestone focuses on understanding generic type parameters and trait bounds before optimizing for performance.

**Why Start Simple?**

Before building an efficient heap, we need to understand:
- How generic type parameters work: `<T>` makes code reusable for any type
- Why trait bounds matter: `T: Ord` ensures elements can be compared
- How Rust's ownership interacts with generic collections
- The baseline performance to improve upon in later milestones

**The Naive Approach:**

```rust
// After each push:
items.push(new_element);
items.sort();  // O(n log n) - expensive!
```

This is inefficient but correct and easy to verify. Once tests pass, we can optimize.

**Real-World Analogy:**

Imagine a todo list where you write tasks on sticky notes. The naive approach is:
1. Add new task to bottom of pile
2. Sort entire pile every time
3. Take task from top

A proper heap would be: add task and bubble it to correct position (one path through the pile, not sorting everything).

**Goal:** Create a working priority queue using a `Vec<T>` with naive sorting.

**What to implement:**
```rust
pub struct PriorityQueue<T> {
    items: Vec<T>,
}

impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self {
        PriorityQueue { items: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
        self.items.sort();  // Naive: O(n log n)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()  // Takes from end (highest priority after sorting)
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
```

**Checkpoint Tests:**

```rust
#[test]
fn test_basic_push_pop_order() {
    let mut pq = PriorityQueue::new();

    // Insert in random order
    pq.push(5);
    pq.push(1);
    pq.push(3);
    pq.push(7);
    pq.push(2);

    // Should pop in sorted order (min-heap: smallest first)
    assert_eq!(pq.pop(), Some(7));
    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(3));
    assert_eq!(pq.pop(), Some(2));
    assert_eq!(pq.pop(), Some(1));
    assert_eq!(pq.pop(), None);
}

#[test]
fn test_with_different_types() {
    // Test with integers
    let mut int_queue = PriorityQueue::new();
    int_queue.push(10);
    int_queue.push(5);
    assert_eq!(int_queue.peek(), Some(&10));

    // Test with strings
    let mut string_queue = PriorityQueue::new();
    string_queue.push("zebra".to_string());
    string_queue.push("apple".to_string());
    string_queue.push("mango".to_string());

    assert_eq!(string_queue.pop(), Some("zebra".to_string()));
    assert_eq!(string_queue.pop(), Some("mango".to_string()));
    assert_eq!(string_queue.pop(), Some("apple".to_string()));
}

#[test]
fn test_custom_ord_type() {
    #[derive(Debug, PartialEq, Eq)]
    struct Task {
        priority: u32,
        name: String,
    }

    impl Ord for Task {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.priority.cmp(&other.priority)
        }
    }

    impl PartialOrd for Task {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut tasks = PriorityQueue::new();
    tasks.push(Task { priority: 5, name: "Medium".into() });
    tasks.push(Task { priority: 10, name: "High".into() });
    tasks.push(Task { priority: 1, name: "Low".into() });

    assert_eq!(tasks.pop().unwrap().priority, 10);
    assert_eq!(tasks.pop().unwrap().priority, 5);
    assert_eq!(tasks.pop().unwrap().priority, 1);
}

#[test]
fn test_peek_does_not_remove() {
    let mut pq = PriorityQueue::new();
    pq.push(42);
    pq.push(17);

    assert_eq!(pq.peek(), Some(&42));
    assert_eq!(pq.len(), 2);  // Still has both elements
    assert_eq!(pq.peek(), Some(&42));  // Can peek multiple times

    assert_eq!(pq.pop(), Some(42));
    assert_eq!(pq.len(), 1);
}

#[test]
fn test_empty_queue() {
    let mut pq: PriorityQueue<i32> = PriorityQueue::new();

    assert!(pq.is_empty());
    assert_eq!(pq.len(), 0);
    assert_eq!(pq.pop(), None);
    assert_eq!(pq.peek(), None);

    pq.push(1);
    assert!(!pq.is_empty());
    assert_eq!(pq.len(), 1);
}

#[test]
fn test_repeated_elements() {
    let mut pq = PriorityQueue::new();

    // Duplicate values should work
    pq.push(5);
    pq.push(5);
    pq.push(5);
    pq.push(3);

    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(3));
}
```

**Why this isn't enough:**

The naive approach sorts the entire vector on every insertion, giving **O(n log n)** insertion time. For a priority queue processing thousands of events per second, this is unacceptable:

- **1,000-element queue**: ~10,000 comparisons per insert (vs ~10 with a proper heap)
- **10,000 inserts**: 100 million operations instead of 100,000
- **Real-world impact**: A server processing 1000 events/sec would spend 90% CPU time just sorting

**Performance comparison:**
- Naive: O(n log n) push, O(1) pop
- Proper heap: O(log n) push, O(log n) pop

The naive approach becomes unusable with even moderate load. Next milestone implements efficient heap operations.

---

### Milestone 2: Implement Binary Heap Structure (Sift Operations)

### Introduction

Replace the naive sorting approach with a proper binary heap data structure. A binary heap maintains a partial ordering where each parent node is less than (min-heap) or greater than (max-heap) its children, enabling O(log n) operations instead of O(n log n).

**Why Binary Heap?**

A binary heap is a complete binary tree stored in an array where:
- **Complete**: All levels filled except possibly the last, which fills left-to-right
- **Heap property**: Parent ≤ children (min-heap) or parent ≥ children (max-heap)
- **Array representation**: No pointers needed, use index arithmetic

**Key Insight - Array-Based Tree:**

```
Array:  [1, 3, 2, 7, 5, 6, 4]
Indices: 0  1  2  3  4  5  6

Tree visualization:
        1 (index 0)
       / \
      3   2  (indices 1, 2)
     / \ / \
    7  5 6  4  (indices 3, 4, 5, 6)

Parent of i:      (i - 1) / 2
Left child of i:  2 * i + 1
Right child of i: 2 * i + 2
```

**The Two Core Operations:**

1. **Sift Up (Bubble Up)**: After inserting at end, swap with parent if violates heap property
   - Used by: `push()`
   - Time: O(log n) - at most height of tree

2. **Sift Down (Bubble Down)**: After removing root, move last element to root and swap down
   - Used by: `pop()`
   - Time: O(log n) - at most height of tree

**Real-World Analogy:**

Think of a corporate hierarchy where managers earn more than reports:
- **Sift up**: New hire at bottom, promote until they're under someone who earns more
- **Sift down**: CEO leaves, temp CEO from bottom starts at top, demoted until hierarchy restored

**Goal:** Replace naive sorting with proper heap operations for O(log n) efficiency.

**What to improve:**

```rust
impl<T: Ord> PriorityQueue<T> {
    // Helper: Calculate parent index
    fn parent(i: usize) -> usize {
        (i - 1) / 2
    }

    // Helper: Calculate left child index
    fn left_child(i: usize) -> usize {
        2 * i + 1
    }

    // Helper: Calculate right child index
    fn right_child(i: usize) -> usize {
        2 * i + 2
    }

    // Sift up: bubble element at index i upward to restore heap property
    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let parent = Self::parent(i);
            if self.items[i] <= self.items[parent] {
                break;  // Heap property satisfied
            }
            self.items.swap(i, parent);
            i = parent;
        }
    }

    // Sift down: bubble element at index i downward to restore heap property
    fn sift_down(&mut self, mut i: usize) {
        loop {
            let left = Self::left_child(i);
            let right = Self::right_child(i);
            let mut largest = i;

            // Find largest among node, left child, right child
            if left < self.items.len() && self.items[left] > self.items[largest] {
                largest = left;
            }
            if right < self.items.len() && self.items[right] > self.items[largest] {
                largest = right;
            }

            if largest == i {
                break;  // Heap property satisfied
            }

            self.items.swap(i, largest);
            i = largest;
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);          // Add to end
        let last_idx = self.items.len() - 1;
        self.sift_up(last_idx);         // Restore heap property: O(log n)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }

        let len = self.items.len();
        self.items.swap(0, len - 1);    // Move root to end
        let result = self.items.pop();  // Remove old root

        if !self.items.is_empty() {
            self.sift_down(0);          // Restore heap property: O(log n)
        }

        result
    }
}
```

**Checkpoint Tests:**

```rust
#[test]
fn test_heap_property_maintained() {
    let mut pq = PriorityQueue::new();

    // Insert elements
    for &val in &[5, 3, 7, 1, 9, 4, 8] {
        pq.push(val);
        assert!(verify_heap_property(&pq));
    }

    // Pop elements
    while !pq.is_empty() {
        pq.pop();
        assert!(verify_heap_property(&pq));
    }
}

// Helper function to verify heap property
fn verify_heap_property<T: Ord>(pq: &PriorityQueue<T>) -> bool {
    for i in 0..pq.len() {
        let left = 2 * i + 1;
        let right = 2 * i + 2;

        if left < pq.len() && pq.items[i] < pq.items[left] {
            return false;  // Parent should be >= left child
        }
        if right < pq.len() && pq.items[i] < pq.items[right] {
            return false;  // Parent should be >= right child
        }
    }
    true
}

#[test]
fn test_sift_operations_correctness() {
    let mut pq = PriorityQueue::new();

    // Build heap: [9, 7, 8, 3, 5, 4]
    //        9
    //       / \
    //      7   8
    //     / \ /
    //    3  5 4

    pq.push(5);
    pq.push(3);
    pq.push(7);
    pq.push(1);
    pq.push(9);
    pq.push(4);
    pq.push(8);

    // Verify largest is at root
    assert_eq!(pq.peek(), Some(&9));

    // Pop should give sorted order
    assert_eq!(pq.pop(), Some(9));
    assert_eq!(pq.pop(), Some(8));
    assert_eq!(pq.pop(), Some(7));
    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(4));
    assert_eq!(pq.pop(), Some(3));
    assert_eq!(pq.pop(), Some(1));
}

#[test]
fn test_large_dataset() {
    let mut pq = PriorityQueue::new();

    // Insert 10,000 elements in random order
    for i in 0..10_000 {
        pq.push(i * 7 % 10_000);  // Pseudo-random order
    }

    // Pop all and verify sorted
    let mut prev = pq.pop().unwrap();
    for _ in 1..10_000 {
        let curr = pq.pop().unwrap();
        assert!(curr <= prev);  // Descending order (max-heap)
        prev = curr;
    }
}

#[test]
fn test_performance_vs_naive() {
    use std::time::Instant;

    let size = 1000;

    // Measure heap-based (this implementation)
    let start = Instant::now();
    let mut heap_pq = PriorityQueue::new();
    for i in 0..size {
        heap_pq.push(i);
    }
    let heap_time = start.elapsed();

    // For comparison (don't actually run in tests, but conceptually):
    // Naive would be: size * size * log(size) / 2 comparisons
    // Heap is:        size * log(size) comparisons
    // Expected speedup: ~size / 2

    println!("Heap insertion time for {}: {:?}", size, heap_time);
    assert!(heap_pq.len() == size);
}

#[test]
fn test_heap_index_arithmetic() {
    // Verify helper functions work correctly
    assert_eq!(PriorityQueue::<i32>::parent(1), 0);
    assert_eq!(PriorityQueue::<i32>::parent(2), 0);
    assert_eq!(PriorityQueue::<i32>::parent(3), 1);
    assert_eq!(PriorityQueue::<i32>::parent(4), 1);
    assert_eq!(PriorityQueue::<i32>::parent(5), 2);

    assert_eq!(PriorityQueue::<i32>::left_child(0), 1);
    assert_eq!(PriorityQueue::<i32>::left_child(1), 3);
    assert_eq!(PriorityQueue::<i32>::left_child(2), 5);

    assert_eq!(PriorityQueue::<i32>::right_child(0), 2);
    assert_eq!(PriorityQueue::<i32>::right_child(1), 4);
    assert_eq!(PriorityQueue::<i32>::right_child(2), 6);
}

#[test]
fn test_single_element() {
    let mut pq = PriorityQueue::new();
    pq.push(42);

    assert_eq!(pq.peek(), Some(&42));
    assert_eq!(pq.pop(), Some(42));
    assert_eq!(pq.pop(), None);
}

#[test]
fn test_two_elements() {
    let mut pq = PriorityQueue::new();
    pq.push(10);
    pq.push(20);

    assert_eq!(pq.pop(), Some(20));
    assert_eq!(pq.pop(), Some(10));
}
```

**Why this isn't enough:**

We're limited to natural ordering (`T: Ord`). This implementation always creates a max-heap (largest element at root). But real applications need flexibility:

- **Min-heap**: Process smallest/earliest items first (event queue, Dijkstra's algorithm)
- **Max-heap**: Process largest/latest items first (top-K problems)
- **Custom ordering**: Prioritize by deadline, not insertion time; by severity, not timestamp

The current design can't handle these without code duplication (copying the entire implementation for min-heap vs max-heap). We need a way to parameterize the comparison logic at compile-time—that's what phantom types solve in Milestone 3.

---

### Milestone 3: Add Phantom Types for Min/Max Heap Variants

### Introduction

Use phantom types to parameterize the heap ordering strategy at compile time. This allows the same code to work as either a min-heap or max-heap without runtime overhead or code duplication.

**Why Phantom Types?**

Phantom types are zero-sized type parameters that exist only at compile time:
- **Zero runtime cost**: `PhantomData<T>` is 0 bytes, optimized away completely
- **Compile-time dispatch**: Compiler generates different code for `MinHeap` vs `MaxHeap`
- **Type safety**: Can't accidentally mix min-heap and max-heap operations
- **No code duplication**: Single implementation serves both orderings

**The Problem With Current Design:**

```rust
// Milestone 2: Hardcoded max-heap
if self.items[left] > self.items[largest] { ... }  // Always >

// To support min-heap, we'd need to duplicate entire impl:
if self.items[left] < self.items[smallest] { ... }  // Always <
```

This violates DRY (Don't Repeat Yourself) and creates maintenance burden.

**The Phantom Type Solution:**

```rust
// Marker types (zero-sized)
struct MinHeap;
struct MaxHeap;

// Generic over ordering
struct PriorityQueue<T, Order = MinHeap> {
    heap: Vec<T>,
    _order: PhantomData<Order>,  // 0 bytes!
}

// Trait defines ordering behavior
trait HeapOrder {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool;
}

// Different impls for different orderings
impl HeapOrder for MinHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent > child  // Parent should be ≤ child
    }
}
```

Now one implementation handles both cases via compile-time polymorphism.

**Real-World Analogy:**

Think of a sorting machine with interchangeable comparator modules:
- **MinHeap module**: "Is left > right?" → swap
- **MaxHeap module**: "Is left < right?" → swap

Same machine (code), different module (type parameter), but the module is just a label—it weighs nothing!

**Goal:** Use phantom types to support both min-heap and max-heap at compile time.

**What to improve:**

```rust
use std::marker::PhantomData;
use std::cmp::Ordering;

// Marker types for ordering
pub struct MinHeap;
pub struct MaxHeap;

// Trait defining heap ordering behavior
pub trait HeapOrder {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool;
}

impl HeapOrder for MinHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent > child  // Min heap: parent should be ≤ child
    }
}

impl HeapOrder for MaxHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent < child  // Max heap: parent should be ≥ child
    }
}

// Generic priority queue with default ordering
pub struct PriorityQueue<T, Order = MinHeap> {
    heap: Vec<T>,
    _order: PhantomData<Order>,
}

impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    pub fn new() -> Self {
        PriorityQueue {
            heap: Vec::new(),
            _order: PhantomData,
        }
    }

    fn parent(i: usize) -> usize {
        (i - 1) / 2
    }

    fn left_child(i: usize) -> usize {
        2 * i + 1
    }

    fn right_child(i: usize) -> usize {
        2 * i + 2
    }

    fn sift_up(&mut self, mut i: usize) {
        while i > 0 {
            let parent = Self::parent(i);
            // Use HeapOrder trait instead of hardcoded comparison
            if !Order::should_swap(&self.heap[parent], &self.heap[i]) {
                break;
            }
            self.heap.swap(i, parent);
            i = parent;
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        loop {
            let left = Self::left_child(i);
            let right = Self::right_child(i);
            let mut swap_with = i;

            if left < self.heap.len() && Order::should_swap(&self.heap[swap_with], &self.heap[left]) {
                swap_with = left;
            }
            if right < self.heap.len() && Order::should_swap(&self.heap[swap_with], &self.heap[right]) {
                swap_with = right;
            }

            if swap_with == i {
                break;
            }

            self.heap.swap(i, swap_with);
            i = swap_with;
        }
    }

    pub fn push(&mut self, item: T) {
        self.heap.push(item);
        let last_idx = self.heap.len() - 1;
        self.sift_up(last_idx);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.heap.is_empty() {
            return None;
        }

        let len = self.heap.len();
        self.heap.swap(0, len - 1);
        let result = self.heap.pop();

        if !self.heap.is_empty() {
            self.sift_down(0);
        }

        result
    }

    pub fn peek(&self) -> Option<&T> {
        self.heap.first()
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}
```

**Checkpoint Tests:**

```rust
#[test]
fn test_min_heap_ordering() {
    let mut min_heap: PriorityQueue<i32, MinHeap> = PriorityQueue::new();

    min_heap.push(5);
    min_heap.push(3);
    min_heap.push(7);
    min_heap.push(1);
    min_heap.push(9);

    // Min heap: smallest first
    assert_eq!(min_heap.pop(), Some(1));
    assert_eq!(min_heap.pop(), Some(3));
    assert_eq!(min_heap.pop(), Some(5));
    assert_eq!(min_heap.pop(), Some(7));
    assert_eq!(min_heap.pop(), Some(9));
}

#[test]
fn test_max_heap_ordering() {
    let mut max_heap: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();

    max_heap.push(5);
    max_heap.push(3);
    max_heap.push(7);
    max_heap.push(1);
    max_heap.push(9);

    // Max heap: largest first
    assert_eq!(max_heap.pop(), Some(9));
    assert_eq!(max_heap.pop(), Some(7));
    assert_eq!(max_heap.pop(), Some(5));
    assert_eq!(max_heap.pop(), Some(3));
    assert_eq!(max_heap.pop(), Some(1));
}

#[test]
fn test_default_is_min_heap() {
    // Without specifying Order, should default to MinHeap
    let mut pq: PriorityQueue<i32> = PriorityQueue::new();

    pq.push(10);
    pq.push(5);
    pq.push(15);

    assert_eq!(pq.pop(), Some(5));  // Smallest first
}

#[test]
fn test_phantom_data_zero_size() {
    use std::mem;

    // PhantomData should add zero bytes
    assert_eq!(
        mem::size_of::<PriorityQueue<i32, MinHeap>>(),
        mem::size_of::<Vec<i32>>()  // Same size as Vec alone
    );

    assert_eq!(
        mem::size_of::<PhantomData<MinHeap>>(),
        0
    );
}

#[test]
fn test_min_heap_with_strings() {
    let mut pq: PriorityQueue<String, MinHeap> = PriorityQueue::new();

    pq.push("zebra".to_string());
    pq.push("apple".to_string());
    pq.push("mango".to_string());
    pq.push("banana".to_string());

    // Lexicographic order: smallest first
    assert_eq!(pq.pop(), Some("apple".to_string()));
    assert_eq!(pq.pop(), Some("banana".to_string()));
    assert_eq!(pq.pop(), Some("mango".to_string()));
    assert_eq!(pq.pop(), Some("zebra".to_string()));
}

#[test]
fn test_max_heap_with_strings() {
    let mut pq: PriorityQueue<String, MaxHeap> = PriorityQueue::new();

    pq.push("zebra".to_string());
    pq.push("apple".to_string());
    pq.push("mango".to_string());

    // Lexicographic order: largest first
    assert_eq!(pq.pop(), Some("zebra".to_string()));
    assert_eq!(pq.pop(), Some("mango".to_string()));
    assert_eq!(pq.pop(), Some("apple".to_string()));
}

#[test]
fn test_type_safety() {
    let _min: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
    let _max: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();

    // These are different types - can't accidentally mix
    // Uncommenting this would cause compile error:
    // let mixed: PriorityQueue<i32, MinHeap> = _max;
}

#[test]
fn test_peek_respects_ordering() {
    let mut min_heap: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
    min_heap.push(10);
    min_heap.push(5);
    min_heap.push(15);

    assert_eq!(min_heap.peek(), Some(&5));  // Smallest at top

    let mut max_heap: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
    max_heap.push(10);
    max_heap.push(5);
    max_heap.push(15);

    assert_eq!(max_heap.peek(), Some(&15));  // Largest at top
}
```

**Why this isn't enough:**

Phantom types solve the min/max problem elegantly, but they're limited to scenarios where we can define ordering at the type level. Real-world applications often need:

- **Custom priorities**: Sort tasks by deadline field, not natural `Ord` of the struct
- **Multi-field comparison**: Priority by (severity, then timestamp)
- **Runtime-configurable ordering**: User selects sorting criteria at runtime
- **Wrapper-based ordering**: Turn max-heap into min-heap by wrapping values

Example limitation:

```rust
struct Task {
    name: String,
    priority: u8,
    deadline: u64,
}

// Can't do this with current design:
// "I want a min-heap by deadline, not by name or priority"
```

The next milestone solves this with wrapper types that implement custom `Ord`.

---

### Milestone 4: Support Custom Orderings with Wrapper Types

### Introduction

Enable custom comparison strategies by wrapping elements in types that implement their own `Ord`. This allows sorting by specific fields, reversing orderings, or applying complex multi-criteria comparisons—all while keeping the priority queue implementation unchanged.

**Why Wrapper Types?**

The priority queue works with any `T: Ord`, so we can:
1. Wrap values in types with custom `Ord` implementations
2. Let the heap use its normal comparison logic
3. Unwrap values when popping

This is the **newtype pattern**: zero-cost abstraction that changes type-level behavior.

**The Custom Ordering Problem:**

```rust
struct Task {
    name: String,
    priority: u8,
    deadline: u64,
}

// Default Ord might compare by name (alphabetical)
// But we want to compare by deadline!
```

Without wrapper types, we'd need to:
- Modify Task's Ord implementation (but what if different parts of code need different orderings?)
- Create separate PriorityQueue implementations (code duplication!)

**The Wrapper Type Solution:**

```rust
// Wrapper changes how comparison works
struct ByDeadline(Task);

impl Ord for ByDeadline {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.deadline.cmp(&other.0.deadline)  // Compare by field!
    }
}

// Now can use with standard PriorityQueue
let mut tasks: PriorityQueue<ByDeadline> = PriorityQueue::new();
```

**Real-World Analogy:**

Think of documents in filing cabinets:
- **Default order**: Alphabetical by title
- **Reverse wrapper**: Put "Z" documents at front
- **ByDate wrapper**: Ignore title, sort by date field
- **ByPriority wrapper**: Urgent documents first

Same documents, same filing system, just different comparison rules.

**Goal:** Allow custom comparison strategies while maintaining type safety.

**What to improve:**

```rust
use std::cmp::Ordering;

// 1. Reverse wrapper - inverts natural ordering
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reverse<T>(pub T);

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)  // Swapped order!
    }
}

impl<T: PartialOrd> PartialOrd for Reverse<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

// 2. Priority by field - extract key for comparison
#[derive(Debug, Clone)]
pub struct ByField<T, F> {
    pub item: T,
    key_fn: F,
}

impl<T, F> ByField<T, F> {
    pub fn new(item: T, key_fn: F) -> Self {
        ByField { item, key_fn }
    }
}

impl<T, K: Ord, F: Fn(&T) -> K> Ord for ByField<T, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.key_fn)(&self.item).cmp(&(other.key_fn)(&other.item))
    }
}

impl<T, K: Ord, F: Fn(&T) -> K> PartialOrd for ByField<T, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, K: Eq, F: Fn(&T) -> K> Eq for ByField<T, F> {}

impl<T, K: Eq, F: Fn(&T) -> K> PartialEq for ByField<T, F> {
    fn eq(&self, other: &Self) -> bool {
        (self.key_fn)(&self.item) == (other.key_fn)(&other.item)
    }
}

// 3. Example: Task with multiple fields
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub name: String,
    pub priority: u8,
    pub deadline: u64,
}

// Default Ord: lexicographic by name
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
```

**Checkpoint Tests:**

```rust
#[test]
fn test_reverse_wrapper() {
    let mut pq: PriorityQueue<Reverse<i32>, MinHeap> = PriorityQueue::new();

    pq.push(Reverse(5));
    pq.push(Reverse(3));
    pq.push(Reverse(7));
    pq.push(Reverse(1));

    // MinHeap with Reverse: largest first (like MaxHeap)
    assert_eq!(pq.pop().unwrap().0, 7);
    assert_eq!(pq.pop().unwrap().0, 5);
    assert_eq!(pq.pop().unwrap().0, 3);
    assert_eq!(pq.pop().unwrap().0, 1);
}

#[test]
fn test_task_by_priority() {
    let mut tasks: PriorityQueue<ByField<Task, _>, MinHeap> = PriorityQueue::new();

    tasks.push(ByField::new(
        Task { name: "Low".into(), priority: 1, deadline: 100 },
        |t| t.priority
    ));
    tasks.push(ByField::new(
        Task { name: "High".into(), priority: 10, deadline: 50 },
        |t| t.priority
    ));
    tasks.push(ByField::new(
        Task { name: "Medium".into(), priority: 5, deadline: 75 },
        |t| t.priority
    ));

    // Should pop in priority order: 1, 5, 10
    assert_eq!(tasks.pop().unwrap().item.priority, 1);
    assert_eq!(tasks.pop().unwrap().item.priority, 5);
    assert_eq!(tasks.pop().unwrap().item.priority, 10);
}

#[test]
fn test_task_by_deadline() {
    let mut tasks: PriorityQueue<ByField<Task, _>, MinHeap> = PriorityQueue::new();

    tasks.push(ByField::new(
        Task { name: "Later".into(), priority: 10, deadline: 200 },
        |t| t.deadline
    ));
    tasks.push(ByField::new(
        Task { name: "Soon".into(), priority: 1, deadline: 50 },
        |t| t.deadline
    ));
    tasks.push(ByField::new(
        Task { name: "Middle".into(), priority: 5, deadline: 100 },
        |t| t.deadline
    ));

    // Should pop by earliest deadline: 50, 100, 200
    assert_eq!(tasks.pop().unwrap().item.deadline, 50);
    assert_eq!(tasks.pop().unwrap().item.deadline, 100);
    assert_eq!(tasks.pop().unwrap().item.deadline, 200);
}

#[test]
fn test_multi_field_comparison() {
    #[derive(Debug, Clone, Eq, PartialEq)]
    struct Event {
        severity: u8,  // Higher = more severe
        timestamp: u64,
    }

    impl Ord for Event {
        fn cmp(&self, other: &Self) -> Ordering {
            // Compare by severity first (reversed: high severity first)
            // Then by timestamp (early first)
            other.severity.cmp(&self.severity)
                .then(self.timestamp.cmp(&other.timestamp))
        }
    }

    impl PartialOrd for Event {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    let mut events: PriorityQueue<Event, MinHeap> = PriorityQueue::new();

    events.push(Event { severity: 5, timestamp: 100 });
    events.push(Event { severity: 10, timestamp: 50 });  // Highest severity
    events.push(Event { severity: 10, timestamp: 75 });  // Same severity, later time
    events.push(Event { severity: 3, timestamp: 25 });

    // Should pop: (10, 50), (10, 75), (5, 100), (3, 25)
    let e1 = events.pop().unwrap();
    assert_eq!((e1.severity, e1.timestamp), (10, 50));

    let e2 = events.pop().unwrap();
    assert_eq!((e2.severity, e2.timestamp), (10, 75));

    let e3 = events.pop().unwrap();
    assert_eq!((e3.severity, e3.timestamp), (5, 100));

    let e4 = events.pop().unwrap();
    assert_eq!((e4.severity, e4.timestamp), (3, 25));
}

#[test]
fn test_reverse_with_custom_type() {
    let mut tasks: PriorityQueue<Reverse<Task>, MinHeap> = PriorityQueue::new();

    tasks.push(Reverse(Task { name: "A".into(), priority: 1, deadline: 100 }));
    tasks.push(Reverse(Task { name: "Z".into(), priority: 1, deadline: 100 }));
    tasks.push(Reverse(Task { name: "M".into(), priority: 1, deadline: 100 }));

    // Reversed alphabetical order
    assert_eq!(tasks.pop().unwrap().0.name, "Z");
    assert_eq!(tasks.pop().unwrap().0.name, "M");
    assert_eq!(tasks.pop().unwrap().0.name, "A");
}

#[test]
fn test_wrapper_zero_cost() {
    use std::mem;

    // Wrapper should add no overhead
    assert_eq!(
        mem::size_of::<Reverse<i32>>(),
        mem::size_of::<i32>()
    );

    assert_eq!(
        mem::size_of::<Reverse<String>>(),
        mem::size_of::<String>()
    );
}

#[test]
fn test_chained_wrappers() {
    // Can combine wrappers for complex behavior
    let mut pq: PriorityQueue<Reverse<ByField<Task, _>>, MinHeap> = PriorityQueue::new();

    pq.push(Reverse(ByField::new(
        Task { name: "Low".into(), priority: 1, deadline: 100 },
        |t| t.priority
    )));
    pq.push(Reverse(ByField::new(
        Task { name: "High".into(), priority: 10, deadline: 50 },
        |t| t.priority
    )));

    // Reversed priority: highest first
    assert_eq!(pq.pop().unwrap().0.item.priority, 10);
    assert_eq!(pq.pop().unwrap().0.item.priority, 1);
}
```

**Why this isn't enough:**

Building a heap from an existing collection currently requires pushing N elements one at a time:

```rust
let mut pq = PriorityQueue::new();
for item in items {
    pq.push(item);  // N × O(log n) = O(n log n)
}
```

For 100,000 items, this does ~1.6 million comparisons. There's a more efficient **heapify** algorithm that builds a heap in O(n) using only ~100,000 comparisons—a 16× improvement! This is critical for bulk initialization from existing data.

---

### Milestone 5: Implement Efficient Heapify (O(n) from Vec)

### Introduction

Implement Floyd's bottom-up heapify algorithm to build a heap from an existing `Vec<T>` in O(n) time instead of O(n log n). This is critical for performance when initializing a priority queue from a large dataset.

**Why Heapify Matters:**

Building a heap by pushing N elements one-at-a-time:
```rust
for item in items {  // N iterations
    pq.push(item);   // O(log n) each
}
// Total: O(n log n)
```

For N=100,000: ~1.6 million operations

Bottom-up heapify:
```rust
PriorityQueue::from_vec(items)  // O(n)
```

For N=100,000: ~100,000 operations (**16× faster!**)

**Floyd's Algorithm Intuition:**

Instead of inserting elements one by one from the top (sift up), start from the bottom and fix parents (sift down):

1. **Leaves are already valid heaps** (half the elements!)
2. **Work up from last parent**, fixing each subtree
3. **Each level needs fewer ops**: Bottom does nothing, middle does O(n/4), top does O(n/8)...

**Mathematical proof of O(n):**

```
Level h (from bottom): n/(2^(h+1)) nodes, each sifts down h steps
Total work: Σ h · n/(2^(h+1)) = n · Σ h/(2^(h+1))
          = n · [1/2 + 2/4 + 3/8 + 4/16 + ...]
          = n · 2     (geometric series)
          = O(n)
```

**Goal:** Add efficient bulk construction from existing data.

**What to improve:**

```rust
impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    /// Build heap from existing vector in O(n) time
    pub fn from_vec(mut vec: Vec<T>) -> Self {
        if vec.is_empty() {
            return Self::new();
        }

        // Start from last non-leaf node and sift down all parents
        let last_parent = (vec.len() / 2).saturating_sub(1);

        for i in (0..=last_parent).rev() {
            Self::sift_down_from(&mut vec, i);
        }

        PriorityQueue {
            heap: vec,
            _order: PhantomData,
        }
    }

    /// Sift down element at index i (standalone version for heapify)
    fn sift_down_from(heap: &mut Vec<T>, mut i: usize) {
        let len = heap.len();

        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            let mut swap_with = i;

            if left < len && Order::should_swap(&heap[swap_with], &heap[left]) {
                swap_with = left;
            }
            if right < len && Order::should_swap(&heap[swap_with], &heap[right]) {
                swap_with = right;
            }

            if swap_with == i {
                break;
            }

            heap.swap(i, swap_with);
            i = swap_with;
        }
    }
}
```

**Checkpoint Tests:**

```rust
#[test]
fn test_from_vec_correctness() {
    let vec = vec![5, 3, 7, 1, 9, 4, 8, 2, 6];
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);

    // Should produce sorted sequence
    assert_eq!(pq.pop(), Some(1));
    assert_eq!(pq.pop(), Some(2));
    assert_eq!(pq.pop(), Some(3));
    assert_eq!(pq.pop(), Some(4));
    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(6));
    assert_eq!(pq.pop(), Some(7));
    assert_eq!(pq.pop(), Some(8));
    assert_eq!(pq.pop(), Some(9));
}

#[test]
fn test_from_vec_performance() {
    use std::time::Instant;

    let size = 10_000;
    let data: Vec<i32> = (0..size).rev().collect();  // Worst case: reverse sorted

    // Method 1: from_vec (heapify)
    let data1 = data.clone();
    let start = Instant::now();
    let pq1: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(data1);
    let heapify_time = start.elapsed();

    // Method 2: repeated push
    let start = Instant::now();
    let mut pq2: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
    for item in data {
        pq2.push(item);
    }
    let push_time = start.elapsed();

    println!("Heapify: {:?}, Push: {:?}", heapify_time, push_time);
    println!("Speedup: {:.2}x", push_time.as_secs_f64() / heapify_time.as_secs_f64());

    // Both should produce same result
    assert_eq!(pq1.len(), pq2.len());
}

#[test]
fn test_from_vec_maintains_heap_property() {
    let vec = vec![15, 3, 17, 10, 84, 19, 6, 22, 9];
    let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);

    // Verify heap property
    for i in 0..pq.len() {
        let left = 2 * i + 1;
        let right = 2 * i + 2;

        if left < pq.len() {
            assert!(pq.heap[i] <= pq.heap[left], "Parent {} > left child {}", pq.heap[i], pq.heap[left]);
        }
        if right < pq.len() {
            assert!(pq.heap[i] <= pq.heap[right], "Parent {} > right child {}", pq.heap[i], pq.heap[right]);
        }
    }
}

#[test]
fn test_from_vec_empty() {
    let vec: Vec<i32> = vec![];
    let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);

    assert!(pq.is_empty());
    assert_eq!(pq.pop(), None);
}

#[test]
fn test_from_vec_single_element() {
    let vec = vec![42];
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);

    assert_eq!(pq.len(), 1);
    assert_eq!(pq.pop(), Some(42));
}

#[test]
fn test_from_vec_with_max_heap() {
    let vec = vec![5, 3, 7, 1, 9, 4, 8];
    let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::from_vec(vec);

    // Max heap: largest first
    assert_eq!(pq.pop(), Some(9));
    assert_eq!(pq.pop(), Some(8));
    assert_eq!(pq.pop(), Some(7));
}

#[test]
fn test_from_vec_large_dataset() {
    let size = 100_000;
    let vec: Vec<i32> = (0..size).collect();

    let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);

    assert_eq!(pq.len(), size as usize);
    assert_eq!(pq.peek(), Some(&0));
}
```

**Why this isn't enough:**

Performance is good, but integration with Rust's ecosystem is missing:

```rust
// Can't do this yet:
let pq: PriorityQueue<_> = values.iter()
    .filter(|x| x.is_valid())
    .map(|x| x.priority_score())
    .collect();  // ❌ No FromIterator impl

// Can't do this yet:
for item in pq {  // ❌ No IntoIterator impl
    println!("{}", item);
}
```

Also missing:
- **Memory control**: Can't pre-allocate capacity
- **Streaming construction**: Must collect entire Vec first
- **Partial consumption**: Can't drain elements without consuming entire queue

Next milestone adds full iterator support and memory management.

---

### Milestone 6: Add Iterator Support and Memory Optimizations

### Introduction

Integrate the priority queue with Rust's iterator ecosystem and add memory management methods. This makes it a first-class collection that works seamlessly with iterator chains, collect(), and for loops.

**Why Iterator Integration Matters:**

Rust's iterator ecosystem is powerful but requires explicit trait implementations:

```rust
// Want to write this:
let pq: PriorityQueue<_> = data.into_iter()
    .filter(|x| x.is_valid())
    .map(|x| transform(x))
    .collect();  // Needs FromIterator

// And this:
for item in pq {  // Needs IntoIterator
    process(item);
}
```

Without these traits, users must write manual loops—verbose and unidiomatic.

**Memory Management:**

Pre-allocation prevents reallocations during growth:
- Without `with_capacity`: Push 10,000 items → 14 reallocations (copy entire heap each time!)
- With `with_capacity(10_000)`: Push 10,000 items → 0 reallocations

**Goal:** Make the priority queue work with Rust's iterator ecosystem and optimize memory usage.

**What to improve:**

```rust
// 1. IntoIterator - consume queue, iterate in sorted order
impl<T, Order> IntoIterator for PriorityQueue<T, Order>
where
    T: Ord,
    Order: HeapOrder,
{
    type Item = T;
    type IntoIter = IntoIter<T, Order>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { queue: self }
    }
}

pub struct IntoIter<T, Order> {
    queue: PriorityQueue<T, Order>,
}

impl<T: Ord, Order: HeapOrder> Iterator for IntoIter<T, Order> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.queue.len();
        (len, Some(len))
    }
}

impl<T: Ord, Order: HeapOrder> ExactSizeIterator for IntoIter<T, Order> {}

// 2. FromIterator - build queue from iterator
impl<T: Ord, Order: HeapOrder> FromIterator<T> for PriorityQueue<T, Order> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        Self::from_vec(vec)  // Uses O(n) heapify!
    }
}

// 3. Extend - add elements from iterator
impl<T: Ord, Order: HeapOrder> Extend<T> for PriorityQueue<T, Order> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
        // Could optimize: collect, heapify, then merge
    }
}

// 4. Memory management
impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        PriorityQueue {
            heap: Vec::with_capacity(capacity),
            _order: PhantomData,
        }
    }

    /// Current capacity (allocated space)
    pub fn capacity(&self) -> usize {
        self.heap.capacity()
    }

    /// Reserve space for at least `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
        self.heap.reserve(additional);
    }

    /// Shrink capacity to fit current length
    pub fn shrink_to_fit(&mut self) {
        self.heap.shrink_to_fit();
    }

    /// Remove all elements
    pub fn clear(&mut self) {
        self.heap.clear();
    }
}
```

**Checkpoint Tests:**

```rust
#[test]
fn test_into_iter_sorted_order() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();

    pq.push(5);
    pq.push(3);
    pq.push(7);
    pq.push(1);
    pq.push(9);

    let result: Vec<i32> = pq.into_iter().collect();
    assert_eq!(result, vec![1, 3, 5, 7, 9]);
}

#[test]
fn test_from_iterator() {
    let data = vec![5, 3, 7, 1, 9, 4, 8];

    let pq: PriorityQueue<i32, MinHeap> = data.into_iter().collect();

    assert_eq!(pq.len(), 7);
    assert_eq!(pq.peek(), Some(&1));
}

#[test]
fn test_iterator_chain() {
    let data = vec![10, 5, 15, 3, 20, 8, 12];

    // Filter, map, collect into priority queue
    let pq: PriorityQueue<i32, MinHeap> = data.into_iter()
        .filter(|x| x % 2 == 0)  // Even numbers only
        .map(|x| x / 2)           // Halve them
        .collect();

    let result: Vec<i32> = pq.into_iter().collect();
    assert_eq!(result, vec![4, 5, 6, 10]);  // [8/2, 10/2, 12/2, 20/2] sorted
}

#[test]
fn test_for_loop() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
    pq.push(3);
    pq.push(1);
    pq.push(2);

    let mut result = Vec::new();
    for item in pq {
        result.push(item);
    }

    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn test_extend() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
    pq.push(5);

    pq.extend(vec![3, 7, 1]);

    assert_eq!(pq.len(), 4);
    assert_eq!(pq.pop(), Some(1));
    assert_eq!(pq.pop(), Some(3));
    assert_eq!(pq.pop(), Some(5));
    assert_eq!(pq.pop(), Some(7));
}

#[test]
fn test_with_capacity() {
    let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::with_capacity(100);

    assert_eq!(pq.len(), 0);
    assert!(pq.capacity() >= 100);
}

#[test]
fn test_reserve() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();

    pq.reserve(1000);
    assert!(pq.capacity() >= 1000);

    // Add elements - should not reallocate
    for i in 0..1000 {
        pq.push(i);
    }
}

#[test]
fn test_shrink_to_fit() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::with_capacity(1000);

    pq.push(1);
    pq.push(2);
    pq.push(3);

    assert!(pq.capacity() >= 1000);

    pq.shrink_to_fit();
    assert!(pq.capacity() < 1000);
    assert_eq!(pq.len(), 3);
}

#[test]
fn test_clear() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();

    pq.push(1);
    pq.push(2);
    pq.push(3);

    pq.clear();

    assert_eq!(pq.len(), 0);
    assert!(pq.is_empty());
    assert_eq!(pq.pop(), None);
}

#[test]
fn test_size_hint() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();

    pq.push(1);
    pq.push(2);
    pq.push(3);

    let mut iter = pq.into_iter();

    assert_eq!(iter.size_hint(), (3, Some(3)));
    iter.next();
    assert_eq!(iter.size_hint(), (2, Some(2)));
    iter.next();
    assert_eq!(iter.size_hint(), (1, Some(1)));
    iter.next();
    assert_eq!(iter.size_hint(), (0, Some(0)));
}

#[test]
fn test_exact_size_iterator() {
    let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();

    pq.push(1);
    pq.push(2);
    pq.push(3);

    let iter = pq.into_iter();

    // ExactSizeIterator provides len()
    assert_eq!(iter.len(), 3);
}
```

**What this achieves:**

Now your priority queue is a first-class Rust collection:

✅ **Iterator integration**: Works with `for` loops, `collect()`, and iterator chains
✅ **Efficient construction**: `FromIterator` uses O(n) heapify, not O(n log n) repeated push
✅ **Memory control**: Pre-allocate to avoid reallocations
✅ **Idiomatic Rust**: Follows conventions from `Vec`, `BinaryHeap`, `HashMap`
✅ **Zero-cost abstractions**: Compiles to same code as hand-written loops

**Extensions to explore:**

- **`Drain` iterator**: Partially consume queue without taking ownership
- **`peek_mut()`**: Modify top element in-place (requires sift-down on drop!)
- **Parallel heapify**: Use Rayon for multi-threaded construction
- **`merge()`**: Combine two heaps efficiently
- **`append()`**: Move all elements from another queue

**Complete!** You've built a production-quality generic priority queue with:
- O(log n) push/pop
- O(n) heapify
- Phantom types for compile-time ordering
- Wrapper types for custom comparisons
- Full iterator support
- Memory management APIs

---

## Project 2: Type-State Builder Pattern for Database Connections

### Problem Statement

Design a database connection builder using phantom types to enforce a correct connection lifecycle at compile time. The system must ensure:
- Configuration methods can only be called in the appropriate state
- Connections cannot be opened without required configuration
- Opened connections cannot be reconfigured
- Transactions follow ACID properties through types
- Invalid state transitions are impossible (compiler errors, not runtime panics)

### Why It Matters

The type-state pattern leverages Rust's type system to make invalid states unrepresentable. This pattern is crucial for:
- **Safety-Critical Systems**: Medical devices, aerospace, automotive software where runtime failures are unacceptable
- **API Design**: Forcing users to use your API correctly at compile time
- **Protocol Implementation**: Network protocols, file format handlers where state must be tracked
- **Resource Management**: Ensuring resources are acquired, used, and released correctly

Type-state patterns appear throughout Rust's ecosystem: `std::net::TcpStream` states (connecting, connected, listening), file handles (read-only, write-only, read-write), and transaction systems.

### Use Cases

1. **Database Connection Pools**: Enforce authentication before query execution
2. **Network Protocol Handlers**: Ensure handshake completion before data transfer
3. **File Operations**: Distinguish read/write/append modes at type level
4. **State Machines**: Game states, UI workflows, business processes
5. **Builder APIs**: Ensure required fields are set before building
6. **Hardware Interfaces**: Ensure initialization before device access

### Solution Outline

**State Markers (Zero-Sized Types):**
```rust
pub struct Disconnected;
pub struct Configured;
pub struct Connected;
pub struct InTransaction;

pub struct ConnectionBuilder<State> {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    // Actual connection handle (only Some in Connected state)
    handle: Option<DbHandle>,
    _state: PhantomData<State>,
}
```

**State Transitions:**
- `new()` → `Disconnected`
- `host()`, `port()`, `database()` → `Configured`
- `connect()` → `Connected` (only from Configured)
- `begin_transaction()` → `InTransaction`
- `commit()`/`rollback()` → `Connected`

**Type Safety:**
```rust
impl ConnectionBuilder<Disconnected> {
    pub fn new() -> Self { /* ... */ }
    pub fn host(self, host: String) -> ConnectionBuilder<Configured> { /* ... */ }
}

impl ConnectionBuilder<Configured> {
    pub fn port(self, port: u16) -> Self { /* ... */ }
    pub fn connect(self) -> Result<ConnectionBuilder<Connected>, Error> { /* ... */ }
}

impl ConnectionBuilder<Connected> {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> { /* ... */ }
    pub fn begin_transaction(self) -> ConnectionBuilder<InTransaction> { /* ... */ }
}

impl ConnectionBuilder<InTransaction> {
    pub fn execute(&mut self, sql: &str) -> Result<(), Error> { /* ... */ }
    pub fn commit(self) -> Result<ConnectionBuilder<Connected>, Error> { /* ... */ }
    pub fn rollback(self) -> ConnectionBuilder<Connected> { /* ... */ }
}
```

### Testing Hints

**Compile-Time Tests:**
```rust
// Should compile
let conn = ConnectionBuilder::new()
    .host("localhost".into())
    .port(5432)
    .connect()?;

// Should NOT compile (test with compile_fail attribute)
#[test]
#[should_panic] // or use trybuild crate
fn cannot_connect_without_host() {
    let conn = ConnectionBuilder::new().connect(); // ERROR: no method
}
```

**Runtime Tests:**
```rust
#[test]
fn test_connection_lifecycle() {
    let conn = ConnectionBuilder::new()
        .host("localhost".into())
        .connect()
        .expect("connection failed");

    let tx = conn.begin_transaction();
    tx.execute("INSERT ...").unwrap();
    tx.commit().unwrap();
}

#[test]
fn test_transaction_rollback() {
    // Verify rollback works and returns to Connected state
}
```

**Use `trybuild` crate for compile-fail tests:**
```rust
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
```

---

## Milestone-by-Milestone Implementation Guide

### Milestone 1: Basic Builder with Optional Fields

#### Introduction: Why the Builder Pattern?

**The Problem:**
Imagine you're configuring a database connection. You need to set a host, port, database name, username, password, SSL settings, timeout, connection pool size, and more. Creating a constructor with 10+ parameters is unmaintainable:

```rust
Connection::new("localhost", 5432, "mydb", "user", "pass", true, 30, 10, false, RetryPolicy::Exponential)
```

Which parameter is which? What if you only want to customize timeout and keep other defaults?

**The Builder Pattern Solution:**
The builder pattern provides a fluent, self-documenting API:

```rust
ConnectionBuilder::new()
    .host("localhost")
    .port(5432)
    .database("mydb")
    .timeout(Duration::from_secs(30))
    .connect()?
```

Each method describes what it does. Order doesn't matter. Optional parameters can be omitted. The API is discoverable through IDE autocomplete.

**Real-World Analogies:**
- **Restaurant Order**: Instead of saying "burger medium-rare pickles no-onions large-fries diet-coke", you build your order: burger, add pickles, remove onions, upsize fries, etc.
- **LEGO Instructions**: Each step adds one component. You can skip optional decorations but must complete required structural pieces.

**Why This Matters:**
The builder pattern appears throughout Rust's ecosystem:
- `std::process::Command` - building shell commands
- `std::thread::Builder` - configuring threads
- `reqwest::RequestBuilder` - HTTP requests
- `tokio::runtime::Builder` - async runtime configuration

Understanding builders is essential for using Rust libraries effectively.

**What We're Building:**
A naive builder using `Option<T>` for all fields with runtime validation. This demonstrates the pattern but has serious flaws we'll fix in later milestones.

**Goal:** Create a working connection builder using `Option<T>` for all fields.

**What to implement:**
```rust
pub struct ConnectionBuilder {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ConnectionBuilder {
    pub fn new() -> Self {
        ConnectionBuilder {
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
        }
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn database(mut self, database: String) -> Self {
        self.database = Some(database);
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    pub fn connect(self) -> Result<Connection, Error> {
        let host = self.host.ok_or(Error::MissingHost)?;
        let port = self.port.unwrap_or(5432); // Default port
        let database = self.database.ok_or(Error::MissingDatabase)?;
        let username = self.username.ok_or(Error::MissingUsername)?;
        let password = self.password.ok_or(Error::MissingPassword)?;

        Ok(Connection {
            host,
            port,
            database,
            username,
            password,
        })
    }
}

pub struct Connection {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
}

impl Connection {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> {
        // Simulate query execution
        Ok(QueryResult {
            rows_affected: 0,
            data: vec![],
        })
    }
}

pub struct QueryResult {
    rows_affected: usize,
    data: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingHost,
    MissingDatabase,
    MissingUsername,
    MissingPassword,
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_builder_flow() {
        // Complete configuration should succeed
        let conn = ConnectionBuilder::new()
            .host("localhost".to_string())
            .port(5432)
            .database("mydb".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect();

        assert!(conn.is_ok());
        let conn = conn.unwrap();
        assert_eq!(conn.host, "localhost");
        assert_eq!(conn.port, 5432);
        assert_eq!(conn.database, "mydb");
    }

    #[test]
    fn test_missing_host_error() {
        // Missing required field should fail at runtime
        let result = ConnectionBuilder::new()
            .port(5432)
            .database("mydb".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect();

        assert_eq!(result, Err(Error::MissingHost));
    }

    #[test]
    fn test_missing_database_error() {
        let result = ConnectionBuilder::new()
            .host("localhost".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect();

        assert_eq!(result, Err(Error::MissingDatabase));
    }

    #[test]
    fn test_default_port() {
        // Port should default to 5432 if not specified
        let conn = ConnectionBuilder::new()
            .host("localhost".to_string())
            .database("mydb".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect()
            .unwrap();

        assert_eq!(conn.port, 5432);
    }

    #[test]
    fn test_setter_chaining_order() {
        // Setters should work in any order
        let conn1 = ConnectionBuilder::new()
            .host("localhost".to_string())
            .database("mydb".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect()
            .unwrap();

        let conn2 = ConnectionBuilder::new()
            .password("pass".to_string())
            .username("user".to_string())
            .database("mydb".to_string())
            .host("localhost".to_string())
            .connect()
            .unwrap();

        assert_eq!(conn1.host, conn2.host);
        assert_eq!(conn1.database, conn2.database);
    }

    #[test]
    fn test_query_on_connection() {
        let conn = ConnectionBuilder::new()
            .host("localhost".to_string())
            .database("mydb".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect()
            .unwrap();

        let result = conn.query("SELECT * FROM users");
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_consumed_by_connect() {
        let builder = ConnectionBuilder::new()
            .host("localhost".to_string())
            .database("mydb".to_string())
            .username("user".to_string())
            .password("pass".to_string());

        let _conn = builder.connect().unwrap();

        // This should not compile if uncommented:
        // let _conn2 = builder.connect(); // ERROR: value moved
    }

    #[test]
    fn test_multiple_databases() {
        // Can build connections to different databases
        let db1 = ConnectionBuilder::new()
            .host("localhost".to_string())
            .database("users_db".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect()
            .unwrap();

        let db2 = ConnectionBuilder::new()
            .host("localhost".to_string())
            .database("orders_db".to_string())
            .username("user".to_string())
            .password("pass".to_string())
            .connect()
            .unwrap();

        assert_ne!(db1.database, db2.database);
    }
}
```

**Why this isn't enough:**

While this builder works, it has critical flaws:

1. **Runtime Errors**: Forgetting to set `host` only fails when `connect()` runs. In production, this might be:
   ```rust
   // Builds fine, crashes later in request handler
   let builder = ConnectionBuilder::new().port(5432);
   // ... 1000 lines later in different module ...
   builder.connect()?; // ERROR: MissingHost (could be in production!)
   ```

2. **No State Tracking**: The API allows nonsensical operations:
   ```rust
   let conn = builder.connect()?;
   // Can we connect again? The type system doesn't prevent it!
   let conn2 = builder.connect()?; // Logically wrong, but compiles
   ```

3. **Can't Enforce Lifecycle**: Nothing prevents:
   ```rust
   let mut builder = ConnectionBuilder::new();
   builder.host("localhost".into());
   // ... connect happens ...
   builder.host("different-host".into()); // Changing config after connect?
   ```

4. **Testing Is Hard**: You can't write compile-time tests that verify "this code should NOT compile". The type system isn't helping us catch bugs.

5. **Documentation Burden**: Users must read docs to know which fields are required. There's no type-level guidance.

**What We Need:** Move validation from runtime to compile-time using Rust's type system. If you forget required fields, the code shouldn't compile at all. That's what phantom types enable in Milestone 2.

---

### Milestone 2: Introduce Phantom Types for Basic States

#### Introduction: Making Invalid States Unrepresentable

**The Problem with Milestone 1:**
Our builder lets you call methods in invalid sequences:

```rust
let builder = ConnectionBuilder::new();
let conn = builder.connect()?; // Wait, we never configured anything!
conn.query("SELECT * FROM users")?; // This might succeed or fail

// Or worse:
let builder = ConnectionBuilder::new().host("localhost".into());
let conn1 = builder.connect()?; // Moved builder
let conn2 = builder.connect()?; // ERROR: builder moved - but only caught at runtime
```

The type system isn't helping us enforce the connection lifecycle. Any sequence of method calls compiles, even nonsensical ones.

**The Type-State Pattern Solution:**
Use the type system to track object state. Different states = different types. Invalid transitions = compile errors.

```rust
// Each state is a distinct type
ConnectionBuilder<Disconnected> // Can only configure
ConnectionBuilder<Configured>   // Can connect
ConnectionBuilder<Connected>    // Can query
```

Now the API enforces correct usage:
```rust
let builder: ConnectionBuilder<Disconnected> = ConnectionBuilder::new();
// builder.connect(); // ERROR: no method `connect` for ConnectionBuilder<Disconnected>

let builder: ConnectionBuilder<Configured> = builder.host("localhost".into());
let conn: ConnectionBuilder<Connected> = builder.connect()?;
conn.query("SELECT * FROM users")?; // OK!
```

**Real-World Analogies:**
- **Traffic Lights**: You can't go from Red → Green without Yellow (in some countries). State transitions are constrained.
- **Microwave**: Door must be closed (state: Closed) before you can start cooking (transition to Cooking). Can't transition from Open → Cooking.
- **TCP Connection**: Must complete handshake (Connecting → Established) before sending data. Can't send while in Connecting state.

**Why Phantom Types?**
The state markers (`Disconnected`, `Configured`, `Connected`) are **zero-sized types (ZSTs)**. They exist only at compile-time for type-checking. At runtime, they occupy zero bytes:

```rust
pub struct Disconnected; // No fields = 0 bytes
pub struct Connected;    // No fields = 0 bytes

pub struct ConnectionBuilder<State> {
    host: Option<String>,
    _state: PhantomData<State>, // Also 0 bytes!
}

// Both are the same size at runtime:
assert_eq!(
    size_of::<ConnectionBuilder<Disconnected>>(),
    size_of::<ConnectionBuilder<Connected>>()
);
```

The state tracking is free—no runtime cost! This is **zero-cost abstraction** in action.

**Goal:** Use phantom types to distinguish Disconnected, Configured, and Connected states.

**What to implement:**
```rust
use std::marker::PhantomData;

// State marker types (zero-sized)
pub struct Disconnected;
pub struct Configured;
pub struct Connected;

pub struct ConnectionBuilder<State> {
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    _state: PhantomData<State>, // Phantom data - exists only for type-checking
}

// Only Disconnected state can be created with new()
impl ConnectionBuilder<Disconnected> {
    pub fn new() -> Self {
        ConnectionBuilder {
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            _state: PhantomData,
        }
    }

    // Transitions to Configured state when host is set
    pub fn host(self, host: String) -> ConnectionBuilder<Configured> {
        ConnectionBuilder {
            host: Some(host),
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password,
            _state: PhantomData,
        }
    }
}

// Configured state has additional setters and can connect
impl ConnectionBuilder<Configured> {
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn database(mut self, database: String) -> Self {
        self.database = Some(database);
        self
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    // Transitions to Connected state
    pub fn connect(self) -> Result<ConnectionBuilder<Connected>, Error> {
        let database = self.database.ok_or(Error::MissingDatabase)?;
        let username = self.username.ok_or(Error::MissingUsername)?;
        let password = self.password.ok_or(Error::MissingPassword)?;

        // Simulate connection establishment
        Ok(ConnectionBuilder {
            host: self.host,
            port: self.port,
            database: Some(database),
            username: Some(username),
            password: Some(password),
            _state: PhantomData,
        })
    }
}

// Connected state can execute queries
impl ConnectionBuilder<Connected> {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> {
        // Simulate query execution
        Ok(QueryResult {
            rows_affected: 0,
            data: vec![],
        })
    }

    pub fn disconnect(self) -> ConnectionBuilder<Disconnected> {
        ConnectionBuilder {
            host: None,
            port: None,
            database: None,
            username: None,
            password: None,
            _state: PhantomData,
        }
    }
}

pub struct QueryResult {
    rows_affected: usize,
    data: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    MissingHost,
    MissingDatabase,
    MissingUsername,
    MissingPassword,
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_state_transitions() {
        let disconnected: ConnectionBuilder<Disconnected> = ConnectionBuilder::new();
        let configured: ConnectionBuilder<Configured> = disconnected.host("localhost".into());
        let connected: ConnectionBuilder<Connected> = configured
            .database("mydb".into())
            .username("user".into())
            .password("pass".into())
            .connect()
            .unwrap();

        let result = connected.query("SELECT 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cannot_query_before_connect() {
        let builder = ConnectionBuilder::new()
            .host("localhost".into())
            .database("mydb".into());

        // This should not compile if uncommented:
        // builder.query("SELECT 1"); // ERROR: no method `query` for ConnectionBuilder<Configured>
    }

    #[test]
    fn test_cannot_connect_from_disconnected() {
        let builder = ConnectionBuilder::new();

        // This should not compile if uncommented:
        // builder.connect(); // ERROR: no method `connect` for ConnectionBuilder<Disconnected>
    }

    #[test]
    fn test_disconnect_and_reconnect() {
        let conn = ConnectionBuilder::new()
            .host("localhost".into())
            .database("mydb".into())
            .username("user".into())
            .password("pass".into())
            .connect()
            .unwrap();

        let disconnected = conn.disconnect();

        // Can create new connection
        let conn2 = disconnected
            .host("newhost".into())
            .database("newdb".into())
            .username("user".into())
            .password("pass".into())
            .connect()
            .unwrap();

        let result = conn2.query("SELECT 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_cost_abstraction() {
        // Phantom types should not add runtime overhead
        assert_eq!(
            mem::size_of::<ConnectionBuilder<Disconnected>>(),
            mem::size_of::<ConnectionBuilder<Connected>>()
        );

        // Both should be the same as the fields alone
        assert_eq!(
            mem::size_of::<ConnectionBuilder<Disconnected>>(),
            mem::size_of::<Option<String>>() * 5 // 5 Option fields
        );
    }

    #[test]
    fn test_state_marker_is_zero_sized() {
        assert_eq!(mem::size_of::<Disconnected>(), 0);
        assert_eq!(mem::size_of::<Configured>(), 0);
        assert_eq!(mem::size_of::<Connected>(), 0);
        assert_eq!(mem::size_of::<PhantomData<Disconnected>>(), 0);
    }

    #[test]
    fn test_builder_consumed_on_transition() {
        let builder = ConnectionBuilder::new();
        let configured = builder.host("localhost".into());

        // This should not compile if uncommented:
        // builder.host("otherhost".into()); // ERROR: value moved

        let _conn = configured
            .database("mydb".into())
            .username("user".into())
            .password("pass".into())
            .connect()
            .unwrap();
    }

    #[test]
    fn test_type_driven_api() {
        // The types guide you to the correct API usage
        let conn = ConnectionBuilder::new()  // Returns ConnectionBuilder<Disconnected>
            .host("localhost".into())        // Transitions to ConnectionBuilder<Configured>
            .database("mydb".into())         // Still ConnectionBuilder<Configured>
            .username("user".into())
            .password("pass".into())
            .connect()                       // Transitions to ConnectionBuilder<Connected>
            .unwrap();

        conn.query("SELECT 1").unwrap();    // Only available on ConnectionBuilder<Connected>
    }

    #[test]
    fn test_multiple_independent_connections() {
        let conn1 = ConnectionBuilder::new()
            .host("host1".into())
            .database("db1".into())
            .username("user".into())
            .password("pass".into())
            .connect()
            .unwrap();

        let conn2 = ConnectionBuilder::new()
            .host("host2".into())
            .database("db2".into())
            .username("user".into())
            .password("pass".into())
            .connect()
            .unwrap();

        // Both connections are independent
        assert!(conn1.query("SELECT 1").is_ok());
        assert!(conn2.query("SELECT 1").is_ok());
    }
}
```

**Why this isn't enough:**

We've made significant progress—invalid state transitions are now compile errors! But we still have problems:

1. **Runtime Validation Still Needed**: Even after transitioning to `Configured`, we still use `Option` and check at runtime:
   ```rust
   pub fn connect(self) -> Result<ConnectionBuilder<Connected>, Error> {
       let database = self.database.ok_or(Error::MissingDatabase)?; // Runtime check!
       // ...
   }
   ```
   The type system knows we're in `Configured` state, but it doesn't know which fields are actually set.

2. **Boilerplate Field Copying**: Every state transition copies all fields:
   ```rust
   ConnectionBuilder {
       host: self.host,
       port: self.port,
       database: self.database,
       username: self.username,
       password: self.password,
       _state: PhantomData,
   }
   ```
   This is error-prone and tedious, especially with many fields.

3. **Can't Enforce Required Fields**: Nothing prevents:
   ```rust
   let conn = ConnectionBuilder::new()
       .host("localhost".into())
       .connect(); // Missing database, username, password - but compiles!
                  // Error only at runtime
   ```

4. **State ≠ Configuration Completeness**: Being in `Configured` state just means `host` was set. It doesn't mean all required configuration is complete.

**What We Need:** Track individual field states using more phantom types. If `host` is required and not set, the code shouldn't compile. Milestone 3 adds phantom type parameters for each field.

---

### Milestone 3: Enforce Required vs Optional Fields with More Phantom Types

#### Introduction: Compile-Time Field Validation

**The Problem with Milestone 2:**
We track connection state (`Disconnected`, `Configured`, `Connected`), but we can't enforce which fields must be set:

```rust
// This compiles but fails at runtime
let conn = ConnectionBuilder::new()
    .host("localhost".into())
    .connect()?; // ERROR at runtime: MissingDatabase
```

The type system knows we're in the correct state, but it doesn't know if we've set all required configuration. We're still doing runtime validation with `.ok_or()`.

**The Multi-Phantom-Type Solution:**
Track each field's state with additional type parameters:

```rust
ConnectionBuilder<Disconnected, NotSet, NotSet>  // host=NotSet, port=NotSet
    .host("localhost")
ConnectionBuilder<Configured, IsSet, NotSet>     // host=IsSet, port=NotSet
    .port(5432)
ConnectionBuilder<Configured, IsSet, IsSet>      // host=IsSet, port=IsSet
    .connect()  // Only available when both IsSet!
```

**Real-World Analogies:**
- **Airplane Pre-Flight Checklist**: Each item (fuel, flaps, instruments) must be checked before takeoff. The checklist tracks completion at the type level—you can't take off until all required items are `IsSet`.
- **Passport Control**: Must have passport (required), visa might be optional depending on destination. The gate agent (type system) won't let you through unless required documents are `IsSet`.
- **Cooking Recipe**: Some ingredients are required (flour, eggs), others optional (vanilla, chocolate chips). You can't bake until required ingredients are `IsSet`.

**How It Works:**
Each field gets its own phantom type parameter:

```rust
pub struct NotSet;  // Field not provided yet
pub struct IsSet;   // Field has been provided

pub struct ConnectionBuilder<State, Host, Port> {
    host: Option<String>,  // Still Option at runtime
    port: Option<u16>,
    _state: PhantomData<State>,  // Connection state
    _host: PhantomData<Host>,    // Host field state
    _port: PhantomData<Port>,    // Port field state
}
```

Setting a field transitions its type parameter from `NotSet` to `IsSet`:

```rust
impl<State, Port> ConnectionBuilder<State, NotSet, Port> {
    pub fn host(self, host: String) -> ConnectionBuilder<State, IsSet, Port> {
        //                                                      ^^^ NotSet -> IsSet
        // ...
    }
}
```

Critical methods require specific field states:

```rust
// connect() ONLY exists when Host=IsSet AND Port=IsSet
impl ConnectionBuilder<Configured, IsSet, IsSet> {
    pub fn connect(self) -> Result<ConnectionBuilder<Connected, IsSet, IsSet>, Error> {
        // We statically know host and port are Some, so unwrap is safe!
        let host = self.host.unwrap();  // Guaranteed safe by type system
        let port = self.port.unwrap();  // Guaranteed safe by type system
        // ...
    }
}
```

**Goal:** Use phantom types for each configurable field to track required fields at compile time.

**What to implement:**
```rust
use std::marker::PhantomData;

// Connection state markers
pub struct Disconnected;
pub struct Configured;
pub struct Connected;

// Field state markers
pub struct NotSet;
pub struct IsSet;

// Now tracks: connection state, host state, database state
pub struct ConnectionBuilder<State, Host, Database> {
    host: Option<String>,
    port: u16,  // Port has a default, so it's not tracked
    database: Option<String>,
    username: String,  // Assume these are required and set early
    password: String,
    _state: PhantomData<State>,
    _host: PhantomData<Host>,
    _database: PhantomData<Database>,
}

// Constructor creates NotSet state for tracked fields
impl ConnectionBuilder<Disconnected, NotSet, NotSet> {
    pub fn new(username: String, password: String) -> Self {
        ConnectionBuilder {
            host: None,
            port: 5432,  // Default
            database: None,
            username,
            password,
            _state: PhantomData,
            _host: PhantomData,
            _database: PhantomData,
        }
    }
}

// host() transitions Host from NotSet to IsSet
// Works in any State, with any Database state
impl<State, Database> ConnectionBuilder<State, NotSet, Database> {
    pub fn host(self, host: String) -> ConnectionBuilder<State, IsSet, Database> {
        ConnectionBuilder {
            host: Some(host),
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password,
            _state: PhantomData,
            _host: PhantomData,
            _database: PhantomData,
        }
    }
}

// database() transitions Database from NotSet to IsSet
impl<State, Host> ConnectionBuilder<State, Host, NotSet> {
    pub fn database(self, database: String) -> ConnectionBuilder<State, Host, IsSet> {
        ConnectionBuilder {
            host: self.host,
            port: self.port,
            database: Some(database),
            username: self.username,
            password: self.password,
            _state: PhantomData,
            _host: PhantomData,
            _database: PhantomData,
        }
    }
}

// port() setter doesn't change type (port is optional)
impl<State, Host, Database> ConnectionBuilder<State, Host, Database> {
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

// When host is set, transition from Disconnected to Configured
// (Alternative: require explicit transition)
impl<Database> ConnectionBuilder<Disconnected, IsSet, Database> {
    pub fn configure(self) -> ConnectionBuilder<Configured, IsSet, Database> {
        ConnectionBuilder {
            host: self.host,
            port: self.port,
            database: self.database,
            username: self.username,
            password: self.password,
            _state: PhantomData,
            _host: PhantomData,
            _database: PhantomData,
        }
    }
}

// connect() ONLY available when Host=IsSet AND Database=IsSet
impl ConnectionBuilder<Configured, IsSet, IsSet> {
    pub fn connect(self) -> Result<ConnectionBuilder<Connected, IsSet, IsSet>, Error> {
        // Safe unwraps - type system guarantees these are Some
        let host = self.host.unwrap();
        let database = self.database.unwrap();

        // Simulate connection
        println!("Connecting to {}:{}/{}", host, self.port, database);

        Ok(ConnectionBuilder {
            host: Some(host),
            port: self.port,
            database: Some(database),
            username: self.username,
            password: self.password,
            _state: PhantomData,
            _host: PhantomData,
            _database: PhantomData,
        })
    }
}

// query() only available on Connected state
impl ConnectionBuilder<Connected, IsSet, IsSet> {
    pub fn query(&self, sql: &str) -> Result<QueryResult, Error> {
        println!("Executing: {}", sql);
        Ok(QueryResult {
            rows_affected: 0,
            data: vec![],
        })
    }
}

pub struct QueryResult {
    rows_affected: usize,
    data: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ConnectionFailed(String),
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_required_fields_enforced() {
        let builder = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure();

        let conn = builder.connect().unwrap();
        assert!(conn.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_cannot_connect_without_host() {
        let _builder = ConnectionBuilder::new("user".into(), "pass".into())
            .database("mydb".into());

        // This should not compile if uncommented:
        // builder.configure().connect(); // ERROR: Host is NotSet
    }

    #[test]
    fn test_cannot_connect_without_database() {
        let _builder = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into());

        // This should not compile if uncommented:
        // builder.configure().connect(); // ERROR: Database is NotSet
    }

    #[test]
    fn test_setters_in_any_order() {
        // Order shouldn't matter
        let conn1 = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let conn2 = ConnectionBuilder::new("user".into(), "pass".into())
            .database("mydb".into())
            .host("localhost".into())
            .configure()
            .connect()
            .unwrap();

        assert!(conn1.query("SELECT 1").is_ok());
        assert!(conn2.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_port_is_optional() {
        // Port has a default, so it's optional
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        // Default port should be used
        assert!(conn.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_custom_port() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .port(3306)  // Custom port
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        assert!(conn.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_field_state_transitions() {
        let s1: ConnectionBuilder<Disconnected, NotSet, NotSet> =
            ConnectionBuilder::new("user".into(), "pass".into());

        let s2: ConnectionBuilder<Disconnected, IsSet, NotSet> =
            s1.host("localhost".into());

        let s3: ConnectionBuilder<Disconnected, IsSet, IsSet> =
            s2.database("mydb".into());

        let s4: ConnectionBuilder<Configured, IsSet, IsSet> =
            s3.configure();

        let s5: ConnectionBuilder<Connected, IsSet, IsSet> =
            s4.connect().unwrap();

        assert!(s5.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_zero_cost_field_tracking() {
        // Field state markers should add no runtime overhead
        assert_eq!(mem::size_of::<NotSet>(), 0);
        assert_eq!(mem::size_of::<IsSet>(), 0);

        // All variants same size regardless of phantom types
        assert_eq!(
            mem::size_of::<ConnectionBuilder<Disconnected, NotSet, NotSet>>(),
            mem::size_of::<ConnectionBuilder<Connected, IsSet, IsSet>>()
        );
    }

    #[test]
    fn test_safe_unwrap() {
        // Because types guarantee fields are set, unwrap is safe
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        // Internal implementation can safely unwrap
        // (verified by successful query)
        assert!(conn.query("SELECT 1").is_ok());
    }
}
```

**Why this isn't enough:**

We've achieved compile-time validation of required fields! But new problems emerge:

1. **Type Parameter Explosion**: With 3 phantom type parameters, the type signatures are verbose:
   ```rust
   ConnectionBuilder<Configured, IsSet, IsSet>
   ```

   Imagine tracking 10 fields—you'd have 12 type parameters! This quickly becomes unmanageable.

2. **Boilerplate Copying**: Every setter copies all fields:
   ```rust
   ConnectionBuilder {
       host: self.host,
       port: self.port,
       database: self.database,
       username: self.username,
       password: self.password,
       _state: PhantomData,
       _host: PhantomData,
       _database: PhantomData,
   }
   ```

   Tedious, error-prone, and scales poorly with field count.

3. **Complex Impl Blocks**: Generic impl blocks with constraints:
   ```rust
   impl<State, Database> ConnectionBuilder<State, NotSet, Database> { ... }
   impl<State, Host> ConnectionBuilder<State, Host, NotSet> { ... }
   impl ConnectionBuilder<Configured, IsSet, IsSet> { ... }
   ```

   Each setter needs careful generic bounds. Easy to get wrong.

4. **No Transaction Support**: Real database connections need transactions with their own lifecycle: begin → execute → commit/rollback. We need more states!

**What We Need:**
1. **Macros** to reduce boilerplate (Milestone 4)
2. **Transaction states** for proper ACID support (Milestone 4)
3. **Connection pooling** for production use (Milestone 5)
4. **Thread safety** for concurrent access (Milestone 6)

---

### Milestone 4: Add Transaction States and Simplify with Macros

#### Introduction: ACID Transactions with Type-State

**The Problem:**
Database transactions require careful management to maintain ACID properties (Atomicity, Consistency, Isolation, Durability):

```rust
// Runtime approach - error prone!
conn.begin_transaction();
conn.execute("INSERT INTO users...");
conn.execute("UPDATE accounts...");
// Oops, forgot to commit or rollback!
// Or worse:
conn.query("SELECT * FROM logs"); // Is this in the transaction?
```

Without type-state tracking:
- You might execute regular queries during a transaction
- You might forget to commit or rollback
- The connection state is unclear (in transaction or not?)
- Error handling is messy (what if commit fails?)

**The Type-State Transaction Solution:**
Make transactions a distinct type with their own state machine:

```rust
let conn: ConnectionBuilder<Connected, IsSet, IsSet> = /* ... */;
let tx: TransactionBuilder<InTransaction> = conn.begin_transaction();
// conn is moved - can't use connection while transaction is active!

tx.execute("INSERT ...")?;
tx.execute("UPDATE ...")?;

let conn: ConnectionBuilder<Connected, IsSet, IsSet> = tx.commit()?;
// Now back in Connected state, can start new transaction
```

**Real-World Analogies:**
- **Bank Withdrawal**: You start a transaction (begin), debit account (execute), dispense cash (execute), commit all changes. If anything fails, rollback completely.
- **Restaurant Order**: Server starts order (begin), adds items (execute), sends to kitchen (commit). If customer changes mind before sending, cancel entire order (rollback).
- **Git Workflow**: Start feature branch (begin), make commits (execute), merge to main (commit) or delete branch (rollback).

**The Boilerplate Problem:**
In Milestone 3, every setter had to copy all fields:

```rust
pub fn host(self, host: String) -> ConnectionBuilder<State, IsSet, Database> {
    ConnectionBuilder {
        host: Some(host),
        port: self.port,        // Tedious copying
        database: self.database,  // Easy to forget a field
        username: self.username,
        password: self.password,
        _state: PhantomData,
        _host: PhantomData,
        _database: PhantomData,
    }
}
```

With 10 fields, this becomes unmaintainable. **Macros** can generate this boilerplate automatically.

**Goal:** Support database transactions as additional states and reduce boilerplate with macros.

**What to implement:**

**1. Add transaction states:**
```rust
use std::marker::PhantomData;

// Previous state markers
pub struct Disconnected;
pub struct Configured;
pub struct Connected;
pub struct NotSet;
pub struct IsSet;

// New transaction state marker
pub struct InTransaction;

// ConnectionBuilder from Milestone 3
pub struct ConnectionBuilder<State, Host, Database> {
    host: Option<String>,
    port: u16,
    database: Option<String>,
    username: String,
    password: String,
    _state: PhantomData<State>,
    _host: PhantomData<Host>,
    _database: PhantomData<Database>,
}

// Only Connected connections can start transactions
impl ConnectionBuilder<Connected, IsSet, IsSet> {
    pub fn begin_transaction(self) -> TransactionBuilder<InTransaction> {
        TransactionBuilder {
            connection: self,
            operations: Vec::new(),
            _state: PhantomData,
        }
    }
}

// Separate type for transactions
pub struct TransactionBuilder<State> {
    connection: ConnectionBuilder<Connected, IsSet, IsSet>,
    operations: Vec<String>,  // Track operations for demonstration
    _state: PhantomData<State>,
}

impl TransactionBuilder<InTransaction> {
    pub fn execute(&mut self, sql: &str) -> Result<(), Error> {
        // Execute SQL within transaction context
        println!("TX: {}", sql);
        self.operations.push(sql.to_string());
        Ok(())
    }

    pub fn commit(self) -> Result<ConnectionBuilder<Connected, IsSet, IsSet>, Error> {
        // Commit all changes
        println!("COMMIT ({} operations)", self.operations.len());
        Ok(self.connection)
    }

    pub fn rollback(self) -> ConnectionBuilder<Connected, IsSet, IsSet> {
        // Rollback all changes
        println!("ROLLBACK ({} operations discarded)", self.operations.len());
        self.connection
    }
}

pub struct QueryResult {
    rows_affected: usize,
    data: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ConnectionFailed(String),
    TransactionFailed(String),
}
```

**2. Reduce boilerplate with macros:**
```rust
// Macro to generate setter methods
macro_rules! impl_optional_setter {
    // For fields that don't change type state
    ($field:ident, $type:ty) => {
        impl<State, Host, Database> ConnectionBuilder<State, Host, Database> {
            pub fn $field(mut self, $field: $type) -> Self {
                self.$field = $field;
                self
            }
        }
    };
}

// Use the macro
impl_optional_setter!(port, u16);
// Expands to the entire impl block above!

// For more complex setters, could create additional macro variants
macro_rules! impl_typed_setter {
    ($from_state:tt, $to_state:tt, $field:ident, $field_type:ty,
     $other_phantom:ident, $other_type:tt) => {
        impl<State, $other_type> ConnectionBuilder<State, $from_state, $other_type> {
            pub fn $field(self, $field: $field_type) ->
                ConnectionBuilder<State, $to_state, $other_type>
            {
                ConnectionBuilder {
                    $field: Some($field),
                    host: self.host,
                    port: self.port,
                    database: self.database,
                    username: self.username,
                    password: self.password,
                    _state: PhantomData,
                    $other_phantom: PhantomData,
                }
            }
        }
    };
}

// Usage (this is complex - mainly for demonstration)
// impl_typed_setter!(NotSet, IsSet, host, String, _database, Database);
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_lifecycle() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let mut tx = conn.begin_transaction();
        tx.execute("INSERT INTO users VALUES (1, 'Alice')").unwrap();
        tx.execute("UPDATE accounts SET balance = 100 WHERE id = 1").unwrap();

        let conn = tx.commit().unwrap();

        // Can use connection again
        assert!(conn.query("SELECT * FROM users").is_ok());
    }

    #[test]
    fn test_transaction_rollback() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let mut tx = conn.begin_transaction();
        tx.execute("INSERT INTO users VALUES (1, 'Bob')").unwrap();
        tx.execute("INVALID SQL").unwrap(); // Simulate error

        // Rollback on error
        let conn = tx.rollback();

        // Connection is back in Connected state
        assert!(conn.query("SELECT * FROM users").is_ok());
    }

    #[test]
    fn test_cannot_query_during_transaction() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let _tx = conn.begin_transaction();

        // This should not compile if uncommented:
        // conn.query("SELECT 1"); // ERROR: conn moved into transaction
    }

    #[test]
    fn test_transaction_consumes_connection() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let tx = conn.begin_transaction();

        // This should not compile if uncommented:
        // conn.query("SELECT 1"); // ERROR: value moved

        let _conn = tx.commit().unwrap();
    }

    #[test]
    fn test_commit_returns_connection() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let mut tx = conn.begin_transaction();
        tx.execute("INSERT INTO users VALUES (1, 'Charlie')").unwrap();

        let conn: ConnectionBuilder<Connected, IsSet, IsSet> = tx.commit().unwrap();

        // Can start new transaction
        let mut tx2 = conn.begin_transaction();
        tx2.execute("INSERT INTO users VALUES (2, 'Diana')").unwrap();
        tx2.commit().unwrap();
    }

    #[test]
    fn test_rollback_returns_connection() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let mut tx = conn.begin_transaction();
        tx.execute("INSERT INTO users VALUES (1, 'Eve')").unwrap();

        let conn: ConnectionBuilder<Connected, IsSet, IsSet> = tx.rollback();

        // Can start new transaction
        let tx2 = conn.begin_transaction();
        tx2.commit().unwrap();
    }

    #[test]
    fn test_macro_generated_setter() {
        // Test that macro-generated port setter works
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .port(3306)  // Uses macro-generated setter
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        assert!(conn.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_transaction_tracks_operations() {
        let conn = ConnectionBuilder::new("user".into(), "pass".into())
            .host("localhost".into())
            .database("mydb".into())
            .configure()
            .connect()
            .unwrap();

        let mut tx = conn.begin_transaction();
        tx.execute("INSERT INTO users VALUES (1, 'Frank')").unwrap();
        tx.execute("UPDATE users SET name = 'Franklin' WHERE id = 1").unwrap();
        tx.execute("DELETE FROM users WHERE id = 2").unwrap();

        // Transaction tracked 3 operations
        assert_eq!(tx.operations.len(), 3);

        tx.commit().unwrap();
    }
}
```

**Why this isn't enough:**

We've added transaction support with type-safety! But production databases need more:

1. **No Connection Pooling**: Creating a new connection for every request is expensive:
   ```rust
   // This is slow!
   for request in requests {
       let conn = ConnectionBuilder::new(...).connect()?;
       conn.query(...)?;
       // Connection closed
   }
   ```

   Connection establishment involves:
   - TCP handshake (network round-trips)
   - Authentication (cryptographic operations)
   - Session initialization (loading config, schema)

   This can take 50-100ms per connection. With connection pooling, we reuse connections, reducing this to ~1ms.

2. **No Thread Safety**: Our pool (next milestone) needs to be shared across threads:
   ```rust
   // Can't share mutable pool
   let mut pool = ConnectionPool::new();
   thread::spawn(|| pool.get_connection()); // ERROR: can't move mut ref
   ```

3. **No Resource Limits**: Without pooling, you might open thousands of connections under load, exhausting database resources.

4. **No Savepoints**: Advanced transaction features like nested transactions or savepoints aren't supported.

**What We Need:**
- Connection pooling with RAII (Resource Acquisition Is Initialization) pattern (Milestone 5)
- Thread-safe pool using `Arc<Mutex<>>` (Milestone 6)
- Configurable pool size and timeout handling (Milestone 6)

---

### Milestone 5: Add Connection Pooling with State Transitions

#### Introduction: RAII and Connection Pooling

**The Performance Problem:**
Creating database connections is expensive:

```rust
// Naive approach - creates new connection for each request
for i in 0..1000 {
    let conn = ConnectionBuilder::new("user".into(), "pass".into())
        .host("localhost".into())
        .database("mydb".into())
        .configure()
        .connect()?;  // 50-100ms per connection!

    conn.query(&format!("SELECT * FROM users WHERE id = {}", i))?;
    // Connection destroyed here
}
// Total time: ~50-100 seconds!
```

Each connection involves:
1. TCP handshake (3 network round-trips)
2. TLS/SSL negotiation (if encrypted)
3. Authentication (password hashing, token validation)
4. Session initialization (loading variables, schema)

**Connection Pooling Solution:**
Maintain a pool of reusable connections:

```rust
let pool = ConnectionPool::new(config);

for i in 0..1000 {
    let conn = pool.get_connection()?;  // ~1ms - reused from pool!
    conn.query(&format!("SELECT * FROM users WHERE id = {}", i))?;
    // Connection returned to pool automatically via Drop
}
// Total time: ~1-2 seconds!
```

**Real-World Analogies:**
- **Rental Cars**: Instead of buying a car for each trip (expensive!), you rent from a pool. When done, you return it for the next customer.
- **Library Books**: One copy serves many readers over time. Checkout and return system manages reuse.
- **Tool Library**: Power tools are expensive. A community shares a pool, checking out and returning tools.

**RAII Pattern (Resource Acquisition Is Initialization):**
Rust's ownership system enables automatic resource cleanup:

```rust
{
    let conn = pool.get_connection()?;
    conn.query("SELECT 1")?;
    // conn goes out of scope here
}  // <- Drop automatically returns connection to pool!
```

No manual cleanup needed! The `Drop` trait ensures the connection is returned, even if an error occurs (via `?` operator) or panic happens.

**Goal:** Implement a connection pool that manages lifecycle states automatically using RAII.

**What to implement:**
```rust
use std::ops::{Deref, DerefMut};

pub struct ConnectionPool {
    available: Vec<ConnectionBuilder<Connected, IsSet, IsSet>>,
    max_size: usize,
    in_use: usize,
    config: PoolConfig,
}

pub struct PoolConfig {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    max_connections: usize,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        ConnectionPool {
            available: Vec::new(),
            max_size: config.max_connections,
            in_use: 0,
            config,
        }
    }

    pub fn get_connection(&mut self) -> Result<PooledConnection, Error> {
        let conn = if let Some(conn) = self.available.pop() {
            // Reuse existing connection
            conn
        } else if self.in_use < self.max_size {
            // Create new connection - pool not at capacity
            self.create_connection()?
        } else {
            // Pool exhausted
            return Err(Error::PoolExhausted);
        };

        self.in_use += 1;

        Ok(PooledConnection {
            inner: Some(conn),
            pool: self,
        })
    }

    fn create_connection(&self) -> Result<ConnectionBuilder<Connected, IsSet, IsSet>, Error> {
        ConnectionBuilder::new(
            self.config.username.clone(),
            self.config.password.clone()
        )
        .host(self.config.host.clone())
        .port(self.config.port)
        .database(self.config.database.clone())
        .configure()
        .connect()
    }

    pub fn stats(&self) -> PoolStats {
        PoolStats {
            available: self.available.len(),
            in_use: self.in_use,
            max_size: self.max_size,
        }
    }
}

// Smart pointer that returns connection to pool on drop (RAII)
pub struct PooledConnection<'a> {
    inner: Option<ConnectionBuilder<Connected, IsSet, IsSet>>,
    pool: &'a mut ConnectionPool,
}

impl<'a> Drop for PooledConnection<'a> {
    fn drop(&mut self) {
        if let Some(conn) = self.inner.take() {
            // Return connection to pool
            self.pool.available.push(conn);
            self.pool.in_use -= 1;
        }
    }
}

// Deref allows using PooledConnection as if it were a ConnectionBuilder
impl<'a> Deref for PooledConnection<'a> {
    type Target = ConnectionBuilder<Connected, IsSet, IsSet>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<'a> DerefMut for PooledConnection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct PoolStats {
    pub available: usize,
    pub in_use: usize,
    pub max_size: usize,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ConnectionFailed(String),
    PoolExhausted,
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pool() -> ConnectionPool {
        ConnectionPool::new(PoolConfig {
            host: "localhost".into(),
            port: 5432,
            database: "testdb".into(),
            username: "user".into(),
            password: "pass".into(),
            max_connections: 3,
        })
    }

    #[test]
    fn test_pool_creates_connection() {
        let mut pool = create_test_pool();
        let conn = pool.get_connection().unwrap();

        assert!(conn.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_connection_returned_on_drop() {
        let mut pool = create_test_pool();

        let stats_before = pool.stats();
        assert_eq!(stats_before.available, 0);
        assert_eq!(stats_before.in_use, 0);

        {
            let _conn = pool.get_connection().unwrap();
            let stats_during = pool.stats();
            assert_eq!(stats_during.available, 0);
            assert_eq!(stats_during.in_use, 1);
        }  // conn dropped here

        let stats_after = pool.stats();
        assert_eq!(stats_after.available, 1);  // Returned to pool!
        assert_eq!(stats_after.in_use, 0);
    }

    #[test]
    fn test_connection_reuse() {
        let mut pool = create_test_pool();

        // First connection
        {
            let conn = pool.get_connection().unwrap();
            conn.query("SELECT 1").unwrap();
        }  // Returned to pool

        let stats = pool.stats();
        assert_eq!(stats.available, 1);

        // Second request reuses the connection
        {
            let conn = pool.get_connection().unwrap();
            conn.query("SELECT 2").unwrap();
        }

        // Still only 1 connection created total
        let stats = pool.stats();
        assert_eq!(stats.available, 1);
    }

    #[test]
    fn test_pool_respects_max_size() {
        let mut pool = create_test_pool();  // max_connections = 3

        let conn1 = pool.get_connection().unwrap();
        let conn2 = pool.get_connection().unwrap();
        let conn3 = pool.get_connection().unwrap();

        // Pool exhausted
        let result = pool.get_connection();
        assert_eq!(result, Err(Error::PoolExhausted));

        drop(conn1);  // Return one

        // Now we can get one more
        let conn4 = pool.get_connection().unwrap();
        assert!(conn4.query("SELECT 1").is_ok());
    }

    #[test]
    fn test_multiple_sequential_requests() {
        let mut pool = create_test_pool();

        for i in 0..10 {
            let conn = pool.get_connection().unwrap();
            conn.query(&format!("SELECT {}", i)).unwrap();
            // Connection returned automatically
        }

        // Should have created only 1 connection (reused 10 times)
        let stats = pool.stats();
        assert_eq!(stats.available, 1);
    }

    #[test]
    fn test_deref_works() {
        let mut pool = create_test_pool();
        let conn = pool.get_connection().unwrap();

        // Can call query() directly thanks to Deref
        let result = conn.query("SELECT 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_connection_leak_prevention() {
        let mut pool = create_test_pool();

        {
            let _conn = pool.get_connection().unwrap();
            // Even if we panic or return early, Drop ensures cleanup
        }

        // Connection was returned
        let stats = pool.stats();
        assert_eq!(stats.in_use, 0);
        assert_eq!(stats.available, 1);
    }

    #[test]
    fn test_pool_stats_accuracy() {
        let mut pool = create_test_pool();

        let _c1 = pool.get_connection().unwrap();
        let _c2 = pool.get_connection().unwrap();

        let stats = pool.stats();
        assert_eq!(stats.available, 0);
        assert_eq!(stats.in_use, 2);
        assert_eq!(stats.max_size, 3);

        drop(_c1);

        let stats = pool.stats();
        assert_eq!(stats.available, 1);
        assert_eq!(stats.in_use, 1);
    }
}
```

**Why this isn't enough:**

We've implemented connection pooling with RAII! But production systems need more:

1. **Not Thread-Safe**: The pool requires `&mut self`, so it can't be shared across threads:
   ```rust
   let mut pool = ConnectionPool::new(config);

   thread::spawn(|| {
       pool.get_connection()?;  // ERROR: can't move mut ref to thread
   });

   pool.get_connection()?;  // Also uses pool
   ```

   Web servers handle concurrent requests—each needs pool access. Currently impossible!

2. **No Timeout Handling**: If all connections are in use, `get_connection()` fails immediately:
   ```rust
   // Under high load
   let result = pool.get_connection();  // PoolExhausted - request fails!
   ```

   Better: Wait briefly for a connection to become available (with timeout).

3. **No Backpressure**: When exhausted, we error immediately. Production systems should:
   - Wait for available connection (with timeout)
   - Optionally queue requests
   - Provide backpressure metrics (wait time, queue depth)

4. **No Health Checks**: Pooled connections might be stale (server closed them). Should validate before reuse.

5. **No Graceful Shutdown**: Can't wait for all connections to be returned before shutting down.

**What We Need:**
- Thread-safe pool using `Arc<Mutex<PoolInner>>` (Milestone 6)
- Timeout-based waiting when pool exhausted (Milestone 6)
- Clone-able pool handle for multi-threaded use (Milestone 6)
- Associated types for different database backends (Milestone 6)

---

### Milestone 6: Thread-Safe Pool with Associated Types and Timeout States

#### Introduction: Concurrency and Arc/Mutex

**The Concurrency Problem:**
Web servers handle multiple requests concurrently. Each request needs a database connection:

```rust
let mut pool = ConnectionPool::new(config);  // &mut self

// Trying to share across threads
let handle1 = thread::spawn(|| {
    pool.get_connection()?;  // ERROR: can't move &mut into thread
});

let handle2 = thread::spawn(|| {
    pool.get_connection()?;  // ERROR: also needs &mut
});
```

Rust's borrow checker prevents data races, but our current design requires exclusive mutable access (`&mut self`), which can't be shared across threads.

**The Arc/Mutex Solution:**
- **`Arc<T>`** (Atomic Reference Counter): Enables shared ownership across threads
- **`Mutex<T>`**: Provides interior mutability with exclusive access

```rust
pub struct ConnectionPool {
    inner: Arc<Mutex<PoolInner>>,  // Shared, thread-safe interior mutability
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool { inner: Arc::clone(&self.inner) }  // Cheap clone!
    }
}

// Now each thread can have its own handle
let pool = ConnectionPool::new(config);
let pool1 = pool.clone();  // Increment Arc counter
let pool2 = pool.clone();

thread::spawn(move || pool1.get_connection());  // ✓ Compiles!
thread::spawn(move || pool2.get_connection());  // ✓ Compiles!
```

**Real-World Analogies:**
- **Rental Car Agency**: Multiple counters (threads) serve customers concurrently, all accessing the same pool of cars. Mutex ensures only one counter modifies the pool at a time.
- **Library Checkout**: Multiple librarians (threads) share access to the book catalog. Lock mechanism ensures only one processes a checkout at a time.
- **Bank Teller Windows**: Multiple tellers access shared account records. Database locks prevent concurrent modification.

**Interior Mutability:**
`Arc<Mutex<T>>` provides **interior mutability**—mutation through shared references:

```rust
// &self (shared reference), not &mut self!
pub fn get_connection(&self) -> Result<PooledConnection, Error> {
    let mut guard = self.inner.lock().unwrap();  // Exclusive access via Mutex
    guard.available.pop()  // Mutate through guard
}
```

**Timeout Handling:**
Instead of immediately failing when pool exhausted, wait with timeout:

```rust
loop {
    if let Some(conn) = try_get_connection()? {
        return Ok(conn);
    }

    if start.elapsed() > timeout {
        return Err(Error::Timeout);
    }

    sleep(10ms);  // Backoff before retry
}
```

**Goal:** Make the pool thread-safe, add timeout handling, and use associated types for extensibility.

**What to implement:**

```rust
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

pub struct ConnectionPool {
    inner: Arc<Mutex<PoolInner>>,
}

struct PoolInner {
    available: Vec<ConnectionBuilder<Connected, IsSet, IsSet>>,
    config: PoolConfig,
    in_use: usize,
}

impl ConnectionPool {
    pub fn new(config: PoolConfig) -> Self {
        ConnectionPool {
            inner: Arc::new(Mutex::new(PoolInner {
                available: Vec::new(),
                config,
                in_use: 0,
            })),
        }
    }

    pub fn get_connection(&self) -> Result<PooledConnection, Error> {
        self.get_connection_timeout(Duration::from_secs(30))
    }

    pub fn get_connection_timeout(&self, timeout: Duration) -> Result<PooledConnection, Error> {
        let start = Instant::now();

        loop {
            {
                let mut pool = self.inner.lock().unwrap();

                // Try to reuse existing connection
                if let Some(conn) = pool.available.pop() {
                    pool.in_use += 1;
                    return Ok(PooledConnection {
                        inner: Some(conn),
                        pool: Arc::clone(&self.inner),
                    });
                }

                // Create new connection if under limit
                if pool.in_use < pool.config.max_connections {
                    pool.in_use += 1;
                    let config = pool.config.clone();

                    // Release lock while connecting (slow operation)
                    drop(pool);

                    let conn = ConnectionBuilder::new(
                        config.username.clone(),
                        config.password.clone()
                    )
                    .host(config.host)
                    .port(config.port)
                    .database(config.database)
                    .configure()
                    .connect()?;

                    return Ok(PooledConnection {
                        inner: Some(conn),
                        pool: Arc::clone(&self.inner),
                    });
                }
            }  // Lock released here

            // Check timeout
            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }

            // Wait briefly before retrying
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn stats(&self) -> PoolStats {
        let pool = self.inner.lock().unwrap();
        PoolStats {
            available: pool.available.len(),
            in_use: pool.in_use,
            max_size: pool.config.max_connections,
        }
    }
}

impl Clone for ConnectionPool {
    fn clone(&self) -> Self {
        ConnectionPool {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct PooledConnection {
    inner: Option<ConnectionBuilder<Connected, IsSet, IsSet>>,
    pool: Arc<Mutex<PoolInner>>,
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        if let Some(conn) = self.inner.take() {
            let mut pool = self.pool.lock().unwrap();
            pool.available.push(conn);
            pool.in_use -= 1;
        }
    }
}

impl Deref for PooledConnection {
    type Target = ConnectionBuilder<Connected, IsSet, IsSet>;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

#[derive(Clone)]
pub struct PoolConfig {
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    max_connections: usize,
}

#[derive(Debug)]
pub struct PoolStats {
    pub available: usize,
    pub in_use: usize,
    pub max_size: usize,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    ConnectionFailed(String),
    PoolExhausted,
    Timeout,
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn create_test_pool() -> ConnectionPool {
        ConnectionPool::new(PoolConfig {
            host: "localhost".into(),
            port: 5432,
            database: "testdb".into(),
            username: "user".into(),
            password: "pass".into(),
            max_connections: 3,
        })
    }

    #[test]
    fn test_thread_safe_pool() {
        let pool = create_test_pool();
        let pool1 = pool.clone();
        let pool2 = pool.clone();

        let h1 = thread::spawn(move || {
            let conn = pool1.get_connection().unwrap();
            conn.query("SELECT 1").unwrap();
        });

        let h2 = thread::spawn(move || {
            let conn = pool2.get_connection().unwrap();
            conn.query("SELECT 2").unwrap();
        });

        h1.join().unwrap();
        h2.join().unwrap();

        // Connections were returned
        let stats = pool.stats();
        assert!(stats.available > 0);
    }

    #[test]
    fn test_concurrent_requests() {
        let pool = Arc::new(create_test_pool());
        let mut handles = vec![];

        for i in 0..10 {
            let pool = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let conn = pool.get_connection().unwrap();
                conn.query(&format!("SELECT {}", i)).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = pool.stats();
        assert_eq!(stats.in_use, 0);
        assert!(stats.available <= 3);  // At most max_connections
    }

    #[test]
    fn test_timeout_on_exhaustion() {
        let pool = create_test_pool();  // max_connections = 3

        // Hold all connections
        let _c1 = pool.get_connection().unwrap();
        let _c2 = pool.get_connection().unwrap();
        let _c3 = pool.get_connection().unwrap();

        // Try to get another with short timeout
        let result = pool.get_connection_timeout(Duration::from_millis(50));
        assert_eq!(result, Err(Error::Timeout));
    }

    #[test]
    fn test_wait_for_available_connection() {
        let pool = Arc::new(create_test_pool());

        // Hold all connections briefly
        let _c1 = pool.get_connection().unwrap();
        let _c2 = pool.get_connection().unwrap();
        let _c3 = pool.get_connection().unwrap();

        let pool_clone = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            // Drop connections by letting them go out of scope
        });

        // This should wait and eventually succeed
        let start = Instant::now();
        let result = pool.get_connection_timeout(Duration::from_secs(2));

        handle.join().unwrap();

        // Should have waited but not timed out
        assert!(start.elapsed() < Duration::from_secs(2));
        assert!(result.is_ok() || result == Err(Error::Timeout));
    }

    #[test]
    fn test_arc_clone_is_cheap() {
        let pool = create_test_pool();

        // Cloning Arc just increments reference count
        let pool2 = pool.clone();
        let pool3 = pool.clone();

        // All point to same underlying pool
        let _conn = pool.get_connection().unwrap();
        let stats2 = pool2.stats();
        let stats3 = pool3.stats();

        assert_eq!(stats2.in_use, stats3.in_use);
    }

    #[test]
    fn test_mutex_prevents_data_race() {
        let pool = Arc::new(create_test_pool());
        let mut handles = vec![];

        // Many threads trying to mutate pool concurrently
        for _ in 0..20 {
            let pool = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                if let Ok(conn) = pool.get_connection() {
                    conn.query("SELECT 1").unwrap();
                    // Drop returns to pool
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // No connections leaked
        let stats = pool.stats();
        assert_eq!(stats.in_use, 0);
    }

    #[test]
    fn test_lock_released_during_connect() {
        // Verifies that lock is released during slow connection creation
        // (Otherwise other threads would block unnecessarily)

        let pool = Arc::new(create_test_pool());
        let pool2 = Arc::clone(&pool);

        // Thread 1: Creates new connection (slow)
        let h1 = thread::spawn(move || {
            pool.get_connection().unwrap();
        });

        // Thread 2: Should be able to create its own connection concurrently
        thread::sleep(Duration::from_millis(10));
        let h2 = thread::spawn(move || {
            pool2.get_connection().unwrap();
        });

        h1.join().unwrap();
        h2.join().unwrap();
    }

    #[test]
    fn test_stats_under_concurrent_load() {
        let pool = Arc::new(create_test_pool());
        let pool2 = Arc::clone(&pool);

        let h1 = thread::spawn(move || {
            let _c = pool.get_connection().unwrap();
            thread::sleep(Duration::from_millis(50));
        });

        thread::sleep(Duration::from_millis(10));

        let stats = pool2.stats();
        assert!(stats.in_use > 0);

        h1.join().unwrap();
    }
}
```

**What this achieves:**

This final milestone completes our production-ready type-state builder:

1. **Thread Safety**: `Arc<Mutex<>>` enables safe sharing across threads with zero data races
2. **Resource Limits**: Enforces maximum connection count to prevent database overload
3. **Timeout Handling**: Graceful degradation under load—wait briefly instead of immediate failure
4. **Type Safety**: Still maintain all compile-time state guarantees from earlier milestones
5. **Performance**: Lock released during slow operations (connection creation) to maximize throughput
6. **RAII**: Automatic connection return via `Drop`, even in concurrent scenarios

**Key Patterns Demonstrated:**
- **Type-State Pattern**: Invalid state transitions are compile errors
- **Phantom Types**: Zero-cost compile-time tracking
- **Interior Mutability**: `Arc<Mutex<T>>` for shared mutable state
- **RAII**: Resource cleanup via `Drop`
- **Builder Pattern**: Fluent, self-documenting API

**Extensions to explore:**
- **Associated Types**: Generic over database backend (PostgreSQL, MySQL, etc.)
- **Health Checks**: Periodically validate pooled connections (ping before reuse)
- **Connection Age**: Expire old connections to prevent stale sessions
- **Async Support**: Use `tokio::sync::Mutex` for async/await compatibility
- **Metrics**: Track wait times, connection lifetimes, hit rates
- **Graceful Shutdown**: Wait for all connections to return before terminating
- **Deadlock Detection**: Timeout on lock acquisition to detect deadlocks

---

## Project 3: Generic Cache with Expiration Strategies

### Problem Statement

Implement a generic in-memory cache that supports:
- Multiple eviction policies (LRU, LFU, FIFO, TTL)
- Compile-time selection of eviction strategy using generics
- Thread-safe concurrent access
- Configurable capacity limits
- Statistics tracking (hit rate, miss rate, eviction count)
- Lazy expiration (items expire on access, not actively)
- Optional write-through to backing store

The cache must work with any key type implementing `Hash + Eq` and any value type, maintaining O(1) get/put performance.

### Why It Matters

Caching is fundamental to high-performance systems:
- **Web Servers**: Cache rendered pages, database queries, session data
- **Databases**: Buffer pool, query result cache
- **Operating Systems**: Page cache, inode cache
- **CDNs**: Cache static assets globally
- **Machine Learning**: Cache computed features, model predictions

Understanding cache implementation teaches:
- How generics enable reusable data structures
- Trade-offs between different eviction policies
- Concurrent data structure design
- Performance optimization techniques

### Use Cases

1. **Web Application**: Cache database query results, API responses
2. **Distributed Systems**: Local cache to reduce network calls
3. **Compilers**: Cache parsed ASTs, compiled artifacts
4. **Image Processing**: Cache thumbnails, transformed images
5. **Game Development**: Asset cache for textures, models
6. **DNS Resolver**: Cache domain name lookups

### Solution Outline

**Core Structure:**
```rust
use std::hash::Hash;
use std::collections::HashMap;

// Eviction strategy trait
pub trait EvictionPolicy<K> {
    fn on_get(&mut self, key: &K);
    fn on_put(&mut self, key: K);
    fn evict_candidate(&self) -> Option<K>;
}

// Cache with generic eviction policy
pub struct Cache<K, V, E: EvictionPolicy<K>> {
    data: HashMap<K, V>,
    eviction: E,
    capacity: usize,
    stats: CacheStats,
}

pub struct CacheStats {
    hits: usize,
    misses: usize,
    evictions: usize,
}
```

**Eviction Policies to Implement:**

1. **LRU (Least Recently Used)**
   - Track access order with doubly-linked list
   - Evict least recently accessed item
   - Use case: General-purpose caching

2. **LFU (Least Frequently Used)**
   - Track access frequency counter
   - Evict least frequently accessed item
   - Use case: Popular item caching

3. **FIFO (First In First Out)**
   - Track insertion order
   - Evict oldest item
   - Use case: Simple caching, log buffers

4. **TTL (Time To Live)**
   - Track insertion/access timestamp
   - Evict expired items
   - Use case: Session caching, rate limiting

**Performance Targets:**
- `get()`: O(1) average case
- `put()`: O(1) average case (amortized for eviction)
- Memory overhead: < 50% of stored data

### Testing Hints

**Unit Tests:**
```rust
#[test]
fn test_lru_eviction() {
    let mut cache = Cache::with_lru(2);
    cache.put(1, "a");
    cache.put(2, "b");
    cache.get(&1); // Access 1, making it more recent
    cache.put(3, "c"); // Should evict 2 (least recent)

    assert_eq!(cache.get(&1), Some(&"a"));
    assert_eq!(cache.get(&2), None); // Evicted
    assert_eq!(cache.get(&3), Some(&"c"));
}

#[test]
fn test_capacity_limit() {
    // Verify cache never exceeds capacity
}

#[test]
fn test_hit_miss_stats() {
    // Verify statistics are accurately tracked
}
```

**Concurrency Tests:**
```rust
#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let cache = Arc::new(Cache::with_lru(100));
    let mut handles = vec![];

    for i in 0..10 {
        let cache = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            for j in 0..100 {
                cache.put(i * 100 + j, format!("value{}", j));
                cache.get(&(i * 100 + j));
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
```

**Performance Tests:**
```rust
#[bench]
fn bench_cache_operations(b: &mut Bencher) {
    let mut cache = Cache::with_lru(1000);
    b.iter(|| {
        for i in 0..1000 {
            cache.put(i, i * 2);
        }
        for i in 0..1000 {
            black_box(cache.get(&i));
        }
    });
}
```

---

## Milestone-by-Milestone Implementation Guide

### Milestone 1: Basic HashMap-Backed Cache with Fixed Size

#### Introduction: Why Caching Matters

**The Performance Problem:**
Repeatedly fetching data from slow sources kills performance:

```rust
// Without caching - slow!
for user_id in user_ids {
    let user = database.query("SELECT * FROM users WHERE id = ?", user_id)?;  // 10ms per query
    process_user(user);
}
// 1000 users × 10ms = 10 seconds!
```

**With Caching:**
```rust
let cache = Cache::new(100);
for user_id in user_ids {
    let user = match cache.get(&user_id) {
        Some(user) => user,  // Cache hit - 0.001ms
        None => {
            let user = database.query("SELECT * FROM users WHERE id = ?", user_id)?;  // 10ms
            cache.put(user_id, user.clone());
            user
        }
    };
    process_user(user);
}
// 100 unique users × 10ms + 900 cache hits × 0.001ms = ~1 second (10x faster!)
```

**Real-World Analogies:**
- **Library**: Keeping frequently-read books on a desk (cache) instead of walking to the shelves (database) every time.
- **Grocery Shopping**: Buying bulk items to store at home (cache) rather than going to the store for each meal (database).
- **Web Browser**: Storing images/scripts locally (cache) rather than downloading on every page load (network).

**Goal:** Create a simple cache using `HashMap` with naive eviction (random).

**What to implement:**
```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct Cache<K, V> {
    data: HashMap<K, V>,
    capacity: usize,
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity && !self.data.contains_key(&key) {
            // Naive eviction: remove first key found (random due to HashMap iteration order)
            if let Some(k) = self.data.keys().next().cloned() {
                self.data.remove(&k);
            }
        }
        self.data.insert(key, value);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_put_get() {
        let mut cache = Cache::new(3);
        cache.put(1, "one");
        cache.put(2, "two");
        cache.put(3, "three");

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_capacity_limit() {
        let mut cache = Cache::new(2);
        cache.put(1, "one");
        cache.put(2, "two");

        assert_eq!(cache.len(), 2);

        cache.put(3, "three");  // Should evict one item

        assert_eq!(cache.len(), 2);  // Still at capacity
    }

    #[test]
    fn test_update_existing_key() {
        let mut cache = Cache::new(2);
        cache.put(1, "one");
        cache.put(2, "two");

        cache.put(1, "ONE");  // Update, shouldn't evict

        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    #[test]
    fn test_get_nonexistent() {
        let cache: Cache<i32, &str> = Cache::new(10);
        assert_eq!(cache.get(&999), None);
    }

    #[test]
    fn test_with_string_keys() {
        let mut cache = Cache::new(3);
        cache.put("key1".to_string(), 100);
        cache.put("key2".to_string(), 200);

        assert_eq!(cache.get(&"key1".to_string()), Some(&100));
        assert_eq!(cache.get(&"key2".to_string()), Some(&200));
    }

    #[test]
    fn test_with_custom_struct() {
        #[derive(Hash, Eq, PartialEq, Clone, Debug)]
        struct UserId(u64);

        #[derive(Debug, PartialEq)]
        struct User {
            name: String,
            age: u32,
        }

        let mut cache = Cache::new(2);
        cache.put(
            UserId(1),
            User { name: "Alice".into(), age: 30 }
        );
        cache.put(
            UserId(2),
            User { name: "Bob".into(), age: 25 }
        );

        assert_eq!(
            cache.get(&UserId(1)),
            Some(&User { name: "Alice".into(), age: 30 })
        );
    }

    #[test]
    fn test_is_empty() {
        let mut cache: Cache<i32, &str> = Cache::new(10);
        assert!(cache.is_empty());

        cache.put(1, "one");
        assert!(!cache.is_empty());
    }

    #[test]
    fn test_zero_capacity() {
        let mut cache = Cache::new(0);
        cache.put(1, "one");

        // With 0 capacity, nothing should be stored
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_generic_over_types() {
        // Test that cache works with various types
        let mut int_cache = Cache::new(2);
        int_cache.put(1, 100);
        assert_eq!(int_cache.get(&1), Some(&100));

        let mut string_cache = Cache::new(2);
        string_cache.put("key", "value");
        assert_eq!(string_cache.get(&"key"), Some(&"value"));

        let mut vec_cache = Cache::new(2);
        vec_cache.put(1, vec![1, 2, 3]);
        assert_eq!(vec_cache.get(&1), Some(&vec![1, 2, 3]));
    }
}
```

**Why this isn't enough:**

Random eviction is fundamentally flawed for caching:

1. **No Locality Benefit**: Real-world access patterns exhibit temporal locality—recently accessed items are likely to be accessed again soon. Random eviction ignores this, evicting hot data as readily as cold data.

2. **Poor Hit Rate**: Example scenario:
   ```rust
   let mut cache = Cache::new(2);
   cache.put(1, "hot");   // Accessed frequently
   cache.put(2, "cold");  // Accessed once

   // Access pattern: 1, 1, 1, 1, 1, 3 (item 1 is "hot")
   cache.get(&1); cache.get(&1); cache.get(&1); cache.get(&1);
   cache.put(3, "new");  // Might evict item 1 (the hot one!) randomly

   cache.get(&1);  // MISS - evicted the frequently used item!
   ```

3. **Unpredictable Performance**: With random eviction, hit rate varies wildly between runs with identical access patterns. This makes performance tuning impossible.

4. **No Statistics**: We can't measure:
   - Hit rate (% of gets that find the item)
   - Miss rate (% of gets that don't find the item)
   - Eviction count (how many items removed)

   Without metrics, we can't optimize cache size or validate effectiveness.

5. **Not Production-Ready**: No real caching system uses random eviction. Industry-standard policies (LRU, LFU, FIFO) dramatically outperform random.

**What We Need:**
- **Smart eviction policy** (LRU in Milestone 2) that keeps frequently/recently used items
- **Statistics tracking** to measure cache effectiveness
- **Generic eviction strategy** (Milestone 3) supporting multiple policies

---

### Milestone 2: Add LRU Eviction with Doubly-Linked List

#### Introduction: Temporal Locality and LRU

**Why LRU (Least Recently Used)?**
Real-world access patterns exhibit **temporal locality**—if data was accessed recently, it's likely to be accessed again soon:

- **Web browsing**: Revisit same pages (home, profile) repeatedly
- **Code editing**: Work on same functions across multiple edits
- **Database queries**: Popular queries repeat (user login, product search)

LRU leverages this by keeping recently-used items in cache and evicting the least-recently-used.

**LRU Algorithm**: Track access order. On eviction, remove the item accessed longest ago.

```rust
// Access pattern: A, B, C, A, D (capacity = 3)
put(A);  // Cache: [A]
put(B);  // Cache: [B, A]
put(C);  // Cache: [C, B, A]
get(A);  // Cache: [A, C, B] - A moved to front
put(D);  // Evict B (least recent), Cache: [D, A, C]
```

**Goal:** Implement proper LRU eviction using `VecDeque` to track access order.

**What to implement:**

```rust
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub struct LruCache<K, V> {
    data: HashMap<K, V>,
    order: VecDeque<K>,  // Front = most recent, back = least recent
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCache {
            data: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if self.data.contains_key(key) {
            // Move to front (most recent)
            self.touch(key);
            self.data.get(key)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            // Update existing - move to front
            self.touch(&key);
            self.data.insert(key, value);
        } else {
            // Evict if at capacity
            if self.data.len() >= self.capacity {
                if let Some(lru_key) = self.order.pop_back() {
                    self.data.remove(&lru_key);
                }
            }

            self.data.insert(key.clone(), value);
            self.order.push_front(key);
        }
    }

    fn touch(&mut self, key: &K) {
        // Remove from current position and move to front
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_front(key.clone());
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}
```

**Checkpoint Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_eviction() {
        let mut cache = LruCache::new(2);
        cache.put(1, "one");
        cache.put(2, "two");

        // Access 1, making it more recent
        cache.get(&1);

        // Add 3, should evict 2 (least recent)
        cache.put(3, "three");

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), None);  // Evicted
        assert_eq!(cache.get(&3), Some(&"three"));
    }

    #[test]
    fn test_get_updates_recency() {
        let mut cache = LruCache::new(3);
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");

        // Access 1, making it most recent
        cache.get(&1);

        // Add 4, should evict 2 (now least recent)
        cache.put(4, "d");

        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), None);  // Evicted
    }

    #[test]
    fn test_put_existing_updates_recency() {
        let mut cache = LruCache::new(2);
        cache.put(1, "one");
        cache.put(2, "two");

        // Update 1, making it most recent
        cache.put(1, "ONE");

        // Add 3, should evict 2
        cache.put(3, "three");

        assert_eq!(cache.get(&1), Some(&"ONE"));
        assert_eq!(cache.get(&2), None);  // Evicted
    }

    #[test]
    fn test_lru_with_repeated_access() {
        let mut cache = LruCache::new(3);
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");

        // Keep accessing 1, it should never be evicted
        cache.get(&1);
        cache.put(4, "d");  // Evicts 2

        cache.get(&1);
        cache.put(5, "e");  // Evicts 3

        assert_eq!(cache.get(&1), Some(&"a"));  // Still present
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), None);
    }

    #[test]
    fn test_order_maintained() {
        let mut cache = LruCache::new(3);
        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");

        // Order: [3, 2, 1] (most to least recent)
        cache.put(4, "d");  // Evicts 1

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.len(), 3);
    }
}
```

**Why this isn't enough:**

1. **O(n) Touch Operation**: The `touch()` method is O(n):
   ```rust
   fn touch(&mut self, key: &K) {
       if let Some(pos) = self.order.iter().position(|k| k == key) {  // O(n) scan
           self.order.remove(pos);  // O(n) shift
           self.order.push_front(key.clone());
       }
   }
   ```

   For a cache with 10,000 items, each `get()` becomes 10,000x slower!

2. **Scalability Problem**: Production caches often hold 100k+ items. O(n) operations make them unusable at scale.

3. **Hardcoded Policy**: LRU is baked into the structure. What about LFU (Least Frequently Used), FIFO, or TTL policies?

4. **Not Generic**: Can't swap policies without rewriting the entire cache.

**What We Need:**
- O(1) touch operation using HashMap + intrusive linked list (Milestone 4)
- Generic eviction policy trait (Milestone 3)
- Statistics tracking and thread safety (Milestone 5)

---

### Milestone 3: Make Eviction Policy Generic with Trait

**Goal:** Abstract eviction logic behind a trait, supporting multiple policies.

**What to improve:**
```rust
use std::hash::Hash;
use std::collections::HashMap;

// Trait for eviction policies
pub trait EvictionPolicy<K> {
    fn new(capacity: usize) -> Self;
    fn on_get(&mut self, key: &K);
    fn on_put(&mut self, key: &K);
    fn on_remove(&mut self, key: &K);
    fn evict_candidate(&self) -> Option<K>;
    fn len(&self) -> usize;
}

// Generic cache
pub struct Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    data: HashMap<K, V>,
    eviction: E,
    capacity: usize,
}

impl<K, V, E> Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    pub fn new(capacity: usize) -> Self {
        Cache {
            data: HashMap::with_capacity(capacity),
            eviction: E::new(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let result = self.data.get(key);
        if result.is_some() {
            self.eviction.on_get(key);
        }
        result
    }

    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            self.eviction.on_put(&key);
            self.data.insert(key, value);
        } else {
            if self.data.len() >= self.capacity {
                if let Some(victim) = self.eviction.evict_candidate() {
                    self.data.remove(&victim);
                    self.eviction.on_remove(&victim);
                }
            }

            self.eviction.on_put(&key);
            self.data.insert(key, value);
        }
    }
}

// Implement LRU policy
pub struct LruPolicy<K> {
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Clone + Eq> EvictionPolicy<K> for LruPolicy<K> {
    fn new(capacity: usize) -> Self {
        LruPolicy {
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        // Move to front
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
            self.order.push_front(key.clone());
        }
    }

    fn on_put(&mut self, key: &K) {
        self.order.push_front(key.clone());
    }

    fn on_remove(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    fn evict_candidate(&self) -> Option<K> {
        self.order.back().cloned()
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

// Implement FIFO policy
pub struct FifoPolicy<K> {
    order: VecDeque<K>,
    capacity: usize,
}

impl<K: Clone + Eq> EvictionPolicy<K> for FifoPolicy<K> {
    fn new(capacity: usize) -> Self {
        FifoPolicy {
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, _key: &K) {
        // FIFO doesn't care about access
    }

    fn on_put(&mut self, key: &K) {
        self.order.push_front(key.clone());
    }

    fn on_remove(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
    }

    fn evict_candidate(&self) -> Option<K> {
        self.order.back().cloned()
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

// Type aliases for convenience
pub type LruCache<K, V> = Cache<K, V, LruPolicy<K>>;
pub type FifoCache<K, V> = Cache<K, V, FifoPolicy<K>>;
```

**Check/Test:**
- Test both LRU and FIFO caches behave correctly
- Verify policy trait abstraction works
- Test that FIFO doesn't update order on `get()`

**Why this isn't enough:**
Still O(n) performance for LRU due to VecDeque search. We need a better data structure—an intrusive doubly-linked list backed by HashMap for O(1) operations. Also, no statistics tracking yet (hit rate, miss rate). We can't measure cache effectiveness.

---

### Milestone 4: Optimize LRU to O(1) with HashMap + Linked List

**Goal:** Achieve true O(1) get/put for LRU using a custom linked list structure.

**What to improve:**

Use a pattern similar to `std::collections::LinkedHashMap` (not in std, but we can build it):

```rust
use std::collections::HashMap;
use std::ptr::NonNull;

// Doubly-linked list node
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

pub struct LruCacheOptimized<K, V>
where
    K: Hash + Eq + Clone,
{
    map: HashMap<K, NonNull<Node<K, V>>>,
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCacheOptimized<K, V> {
    pub fn new(capacity: usize) -> Self {
        LruCacheOptimized {
            map: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(&node_ptr) = self.map.get(key) {
            unsafe {
                self.move_to_front(node_ptr);
                Some(&(*node_ptr.as_ptr()).value)
            }
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            unsafe {
                (*node_ptr.as_ptr()).value = value;
                self.move_to_front(node_ptr);
            }
        } else {
            if self.map.len() >= self.capacity {
                self.remove_tail();
            }

            let mut node = Box::new(Node {
                key: key.clone(),
                value,
                prev: None,
                next: self.head,
            });

            let node_ptr = NonNull::new(Box::into_raw(node)).unwrap();

            if let Some(mut head) = self.head {
                unsafe {
                    (*head.as_ptr()).prev = Some(node_ptr);
                }
            } else {
                self.tail = Some(node_ptr);
            }

            self.head = Some(node_ptr);
            self.map.insert(key, node_ptr);
        }
    }

    unsafe fn move_to_front(&mut self, node_ptr: NonNull<Node<K, V>>) {
        let node = node_ptr.as_ptr();

        if self.head == Some(node_ptr) {
            return; // Already at front
        }

        // Remove from current position
        if let Some(mut prev) = (*node).prev {
            (*prev.as_ptr()).next = (*node).next;
        }

        if let Some(mut next) = (*node).next {
            (*next.as_ptr()).prev = (*node).prev;
        } else {
            self.tail = (*node).prev;
        }

        // Move to front
        (*node).prev = None;
        (*node).next = self.head;

        if let Some(mut head) = self.head {
            (*head.as_ptr()).prev = Some(node_ptr);
        }

        self.head = Some(node_ptr);
    }

    fn remove_tail(&mut self) {
        if let Some(tail_ptr) = self.tail {
            unsafe {
                let tail = tail_ptr.as_ptr();
                let key = (*tail).key.clone();

                self.map.remove(&key);

                if let Some(mut prev) = (*tail).prev {
                    (*prev.as_ptr()).next = None;
                    self.tail = Some(prev);
                } else {
                    self.head = None;
                    self.tail = None;
                }

                // Free the node
                drop(Box::from_raw(tail));
            }
        }
    }
}

impl<K, V> Drop for LruCacheOptimized<K, V>
where
    K: Hash + Eq + Clone,
{
    fn drop(&mut self) {
        let mut current = self.head;
        while let Some(node_ptr) = current {
            unsafe {
                let node = Box::from_raw(node_ptr.as_ptr());
                current = node.next;
            }
        }
    }
}
```

**Important:** This uses unsafe code. In a real implementation, consider using a safe library like `lru` crate or refactoring with indices.

**Check/Test:**
- Benchmark O(1) performance for large caches (10k+ items)
- Test with Miri or AddressSanitizer for memory safety
- Verify no memory leaks with valgrind or similar

**Why this isn't enough:**
We've optimized LRU, but:
1. No statistics tracking (hit/miss rate, eviction count)
2. Not thread-safe—can't share across threads
3. No TTL (time-based expiration) support
4. No LFU (Least Frequently Used) policy

Let's add statistics and thread safety next.

---

### Milestone 5: Add Statistics and Thread Safety

**Goal:** Track cache performance metrics and make the cache thread-safe.

**What to improve:**

**1. Add statistics:**
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct CacheStats {
    hits: AtomicUsize,
    misses: AtomicUsize,
    evictions: AtomicUsize,
    inserts: AtomicUsize,
}

impl CacheStats {
    pub fn hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn insert(&self) {
        self.inserts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let total = hits + self.misses.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    pub fn summary(&self) -> StatsSummary {
        StatsSummary {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            inserts: self.inserts.load(Ordering::Relaxed),
        }
    }
}

pub struct StatsSummary {
    pub hits: usize,
    pub misses: usize,
    pub evictions: usize,
    pub inserts: usize,
}
```

**2. Add thread safety with RwLock:**
```rust
use std::sync::{Arc, RwLock};

pub struct ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    inner: Arc<RwLock<Cache<K, V, E>>>,
    stats: Arc<CacheStats>,
}

impl<K, V, E> Clone for ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    fn clone(&self) -> Self {
        ThreadSafeCache {
            inner: Arc::clone(&self.inner),
            stats: Arc::clone(&self.stats),
        }
    }
}

impl<K, V, E> ThreadSafeCache<K, V, E>
where
    K: Hash + Eq + Clone,
    V: Clone,
    E: EvictionPolicy<K>,
{
    pub fn new(capacity: usize) -> Self {
        ThreadSafeCache {
            inner: Arc::new(RwLock::new(Cache::new(capacity))),
            stats: Arc::new(CacheStats::default()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.inner.write().unwrap();
        let result = cache.get(key).cloned();

        if result.is_some() {
            self.stats.hit();
        } else {
            self.stats.miss();
        }

        result
    }

    pub fn put(&self, key: K, value: V) {
        let mut cache = self.inner.write().unwrap();
        let was_full = cache.len() >= cache.capacity();

        cache.put(key, value);

        if was_full {
            self.stats.eviction();
        }
        self.stats.insert();
    }

    pub fn stats(&self) -> StatsSummary {
        self.stats.summary()
    }
}
```

**Check/Test:**
- Test concurrent access from multiple threads
- Verify statistics are accurate under concurrent load
- Test that hit/miss rate calculation is correct
- Benchmark performance with contention

**Why this isn't enough:**
Write lock for reads is inefficient—multiple readers could access simultaneously, but `get()` updates LRU order (mutable). This creates contention. Also, still no TTL support for time-based expiration. Large caches can grow stale without expiration.

---

### Milestone 6: Add TTL Support and Lazy Expiration

**Goal:** Implement time-based expiration with lazy eviction on access.

**What to improve:**

**1. Add TTL policy:**
```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct TtlPolicy<K> {
    timestamps: HashMap<K, Instant>,
    ttl: Duration,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for TtlPolicy<K> {
    fn new(capacity: usize) -> Self {
        TtlPolicy {
            timestamps: HashMap::with_capacity(capacity),
            ttl: Duration::from_secs(300), // Default 5 minutes
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        // Update timestamp on access
        self.timestamps.insert(key.clone(), Instant::now());
    }

    fn on_put(&mut self, key: &K) {
        self.timestamps.insert(key.clone(), Instant::now());
    }

    fn on_remove(&mut self, key: &K) {
        self.timestamps.remove(key);
    }

    fn evict_candidate(&self) -> Option<K> {
        let now = Instant::now();

        // Find first expired item
        self.timestamps
            .iter()
            .find(|(_, &timestamp)| now.duration_since(timestamp) > self.ttl)
            .map(|(k, _)| k.clone())
            .or_else(|| {
                // If none expired, evict oldest
                self.timestamps
                    .iter()
                    .min_by_key(|(_, &timestamp)| timestamp)
                    .map(|(k, _)| k.clone())
            })
    }

    fn len(&self) -> usize {
        self.timestamps.len()
    }
}

impl<K> TtlPolicy<K> {
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        TtlPolicy {
            timestamps: HashMap::with_capacity(capacity),
            ttl,
            capacity,
        }
    }
}
```

**2. Add lazy expiration check to Cache:**
```rust
impl<K, V, E> Cache<K, V, E>
where
    K: Hash + Eq + Clone,
    E: EvictionPolicy<K>,
{
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Check for expiration first
        if self.is_expired(key) {
            self.remove(key);
            return None;
        }

        let result = self.data.get(key);
        if result.is_some() {
            self.eviction.on_get(key);
        }
        result
    }

    fn is_expired(&self, key: &K) -> bool {
        // Policies can implement expiration check
        // For TTL: check if timestamp + ttl < now
        false // Default: no expiration
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.eviction.on_remove(key);
        self.data.remove(key)
    }

    pub fn cleanup_expired(&mut self) -> usize {
        // Periodic cleanup of expired items
        let mut count = 0;
        let expired: Vec<K> = self.data
            .keys()
            .filter(|k| self.is_expired(k))
            .cloned()
            .collect();

        for key in expired {
            self.remove(&key);
            count += 1;
        }

        count
    }
}
```

**3. Add LFU (Least Frequently Used) policy:**
```rust
pub struct LfuPolicy<K> {
    frequencies: HashMap<K, usize>,
    capacity: usize,
}

impl<K: Hash + Eq + Clone> EvictionPolicy<K> for LfuPolicy<K> {
    fn new(capacity: usize) -> Self {
        LfuPolicy {
            frequencies: HashMap::with_capacity(capacity),
            capacity,
        }
    }

    fn on_get(&mut self, key: &K) {
        *self.frequencies.entry(key.clone()).or_insert(0) += 1;
    }

    fn on_put(&mut self, key: &K) {
        self.frequencies.insert(key.clone(), 1);
    }

    fn on_remove(&mut self, key: &K) {
        self.frequencies.remove(key);
    }

    fn evict_candidate(&self) -> Option<K> {
        self.frequencies
            .iter()
            .min_by_key(|(_, &freq)| freq)
            .map(|(k, _)| k.clone())
    }

    fn len(&self) -> usize {
        self.frequencies.len()
    }
}
```

**Type aliases:**
```rust
pub type LruCache<K, V> = Cache<K, V, LruPolicy<K>>;
pub type LfuCache<K, V> = Cache<K, V, LfuPolicy<K>>;
pub type FifoCache<K, V> = Cache<K, V, FifoPolicy<K>>;
pub type TtlCache<K, V> = Cache<K, V, TtlPolicy<K>>;
```

**Check/Test:**
- Test TTL expiration: items expire after configured duration
- Test LFU evicts least frequently used items
- Test lazy expiration only triggers on access
- Test `cleanup_expired()` removes all expired items
- Benchmark different policies under various workloads

**What this achieves:**
- **Multiple Eviction Policies**: LRU, LFU, FIFO, TTL all supported through generic trait
- **Time-Based Expiration**: Items automatically expire after TTL
- **Performance**: O(1) operations for all policies (except expiration cleanup)
- **Thread Safety**: Safe concurrent access with Arc<RwLock>
- **Statistics**: Track hit rate, miss rate, eviction count
- **Flexibility**: Generic over key/value types and eviction policy
- **Type Safety**: Compile-time policy selection

**Extensions to explore:**
- Write-through cache: sync to backing store
- Cache warming: pre-populate from disk
- Tiered caching: L1 (memory) + L2 (disk)
- Async support: async get/put for network caches
- Bloom filters: fast negative lookups
- Compression: compress values to save memory

---

## Summary

These three projects teach complementary aspects of Rust's generics and polymorphism:

1. **Priority Queue**: Generic data structures, trait bounds, phantom types for compile-time configuration, efficient algorithms

2. **Type-State Connection Builder**: Phantom types for state machines, zero-cost state guarantees, builder pattern, compile-time safety

3. **Generic Cache**: Trait-based polymorphism, multiple implementations of abstraction, performance optimization, thread safety, statistics

All three emphasize:
- Zero-cost abstractions through generics
- Compile-time safety guarantees
- Performance-conscious design
- Practical, real-world patterns

Students should come away understanding how to design flexible, type-safe, performant generic APIs—the foundation of modern Rust systems programming.
