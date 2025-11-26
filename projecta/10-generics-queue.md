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


struct: `PriorityQueue<T>` 
fields: items: Vec<T>
functions:
- `new()` - Create empty queue
- `push(item: T)` - Insert element (sift up to maintain heap property)
- `pop() -> Option<T>` - Remove and return highest priority element (sift down)
- `peek() -> Option<&T>` - View highest priority element
- `len()`, `is_empty()` - Basic queries


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

**Starter Code**
```rust
pub struct PriorityQueue<T> {
    items: Vec<T>,
}

impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self {
       todo!()
    }

    pub fn push(&mut self, item: T) {
       todo!()
    }

    pub fn pop(&mut self) -> Option<T> {
       todo!()  // Takes from end (highest priority after sorting)
    }

    pub fn peek(&self) -> Option<&T> {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()

    pub fn is_empty(&self) -> bool {
       todo!()
    }
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



**Goal:** Replace naive sorting with proper heap operations for O(log n) efficiency.

functions
- `sift_up()`: bubble element at index i upward 
- `sift down()`: bubble element at index i downward

change `push()` and `pop()` to use the `sift` functions

**What to improve:**


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

**Starter Code**

```rust
impl<T: Ord> PriorityQueue<T> {
    // Helper: Calculate parent index
    fn parent(i: usize) -> usize {
        todo!()
    }

    // Helper: Calculate left child index
    fn left_child(i: usize) -> usize {
        todo!()
    }

    // Helper: Calculate right child index
    fn right_child(i: usize) -> usize {
       todo!()
    }

    // Sift up: bubble element at index i upward to restore heap property
    fn sift_up(&mut self, mut i: usize) {
        todo!()
    }

    // Sift down: bubble element at index i downward to restore heap property
    fn sift_down(&mut self, mut i: usize) {
        loop {
            let left = Self::left_child(i);
            let right = Self::right_child(i);
            
            todo!()
        }
    }

    pub fn push(&mut self, item: T) { 
        todo!()        // Restore heap property: 
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }
}
```


**Solution**

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

**Starter Code**

```rust
use std::marker::PhantomData;
use std::cmp::Ordering;

// Marker types for ordering


// Trait defining heap ordering behavior
pub trait HeapOrder {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool;
}

impl HeapOrder for MinHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        //TODO: Min heap: parent should be ≤ child
    }
}

impl HeapOrder for MaxHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        // TODO: Max heap: parent should be ≥ child
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
            // TODO: Use HeapOrder trait instead of hardcoded comparison
           
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        loop {
            let left = Self::left_child(i);
            let right = Self::right_child(i);
            let mut swap_with = i;

            // TODO: Use HeapOrder trait instead of hardcoded comparison

            if swap_with == i {
                break;
            }

            self.heap.swap(i, swap_with);
            i = swap_with;
        }
    }

    pub fn push(&mut self, item: T) {
       todo!()
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!()
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


**Solution**

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


**Why this isn't enough:**

Phantom types solve the min/max problem elegantly, but they're limited to scenarios where we can define ordering at the type level. Real-world applications often need:

- **Custom priorities**: Sort tasks by deadline field, not natural `Ord` of the struct
- **Multi-field comparison**: Priority by (severity, then timestamp)
- **Runtime-configurable ordering**: User selects sorting criteria at runtime
- **Wrapper-based ordering**: Turn max-heap into min-heap by wrapping values


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


**Starter Code**

```rust
use std::cmp::Ordering;

// 1. Reverse wrapper - inverts natural ordering
// TODO: #[derive(..)]
pub struct Reverse<T>(pub T);

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}

impl<T: PartialOrd> PartialOrd for Reverse<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        todo!()
    }
}

// 2. Priority by field - extract key for comparison
// TODO: #[derive(..)]
pub struct ByField<T, F> {
    pub item: T,
    key_fn: F,
}

impl<T, F> ByField<T, F> {
    pub fn new(item: T, key_fn: F) -> Self {
       todo!()
    }
}

impl<T, K: Ord, F: Fn(&T) -> K> Ord for ByField<T, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}

impl<T, K: Ord, F: Fn(&T) -> K> PartialOrd for ByField<T, F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        todo!()
    }
}

impl<T, K: Eq, F: Fn(&T) -> K> Eq for ByField<T, F> {}

impl<T, K: Eq, F: Fn(&T) -> K> PartialEq for ByField<T, F> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

// 3. Example: Task with multiple fields
// TODO: #[derive(..)]
pub struct Task {
    pub name: String,
    pub priority: u8,
    pub deadline: u64,
}

// Default Ord: lexicographic by name
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        todo!()
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        todo!()
    }
}
```

**Solution**


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

Add function:   
 - `from_vec()`
**Starter Code:**
```rust
impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    /// Build heap from existing vector in O(n) time
   Self {
        todo!();
    }

    /// Sift down element at index i (standalone version for heapify)
    fn sift_down_from(heap: &mut Vec<T>, mut i: usize) {
       todo!()
    }
}
```
**solution**
```rust
impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    /// Build heap from existing vector in O(n) time
   Self {
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
**Starter Code**

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
       todo!()
    }
}

pub struct IntoIter<T, Order> {
    queue: PriorityQueue<T, Order>,
}

impl<T: Ord, Order: HeapOrder> Iterator for IntoIter<T, Order> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
      todo!()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
       todo!()
    }
}

impl<T: Ord, Order: HeapOrder> ExactSizeIterator for IntoIter<T, Order> {}

// 2. FromIterator - build queue from iterator
impl<T: Ord, Order: HeapOrder> FromIterator<T> for PriorityQueue<T, Order> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
       todo!()
}

// 3. Extend - add elements from iterator
impl<T: Ord, Order: HeapOrder> Extend<T> for PriorityQueue<T, Order> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        todo!()
        // Could optimize: collect, heapify, then merge
    }
}

// 4. Memory management
impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
      todo!()
    }

    /// Current capacity (allocated space)
    pub fn capacity(&self) -> usize {
        todo!()
    }

    /// Reserve space for at least `additional` more elements
    pub fn reserve(&mut self, additional: usize) {
       todo!()
    }

    /// Shrink capacity to fit current length
    pub fn shrink_to_fit(&mut self) {
        todo!()
    }

    /// Remove all elements
    pub fn clear(&mut self) {
        todo!()
    }
}
```

**Solution**

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