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

## Step-by-Step Implementation Guide

### Step 1: Basic Generic Structure with Vec Backend

**Goal:** Create a working priority queue using a `Vec<T>` with naive sorting.

**What to implement:**
```rust
pub struct PriorityQueue<T> {
    items: Vec<T>,
}

impl<T: Ord> PriorityQueue<T> {
    pub fn new() -> Self { /* ... */ }
    pub fn push(&mut self, item: T) { /* items.push + items.sort() */ }
    pub fn pop(&mut self) -> Option<T> { /* items.pop() */ }
    pub fn peek(&self) -> Option<&T> { /* items.last() */ }
    pub fn len(&self) -> usize { /* ... */ }
    pub fn is_empty(&self) -> bool { /* ... */ }
}
```

**Check/Test:**
- Insert elements and pop them back in sorted order
- Test with `i32`, `String`, and custom `Ord` types
- Verify basic operations work correctly

**Why this isn't enough:**
The naive approach sorts the entire vector on every insertion, giving O(n log n) insertion time. For a priority queue processing thousands of events per second, this is unacceptable. A 1000-element queue would perform ~10,000 comparisons per insert instead of ~10 with a proper heap.

---

### Step 2: Implement Binary Heap Structure (Sift Operations)

**Goal:** Replace naive sorting with proper heap operations for O(log n) efficiency.

**What to improve:**
- Implement `sift_up()` - bubble newly inserted element to correct position
- Implement `sift_down()` - after removing root, restore heap property
- Change `push()` to: append to end, then sift_up
- Change `pop()` to: swap root with last, remove last, sift_down root

**Key insight - Heap indexing:**
```rust
fn parent(i: usize) -> usize { (i - 1) / 2 }
fn left_child(i: usize) -> usize { 2 * i + 1 }
fn right_child(i: usize) -> usize { 2 * i + 2 }
```

**Check/Test:**
- Write a `verify_heap_property()` helper that checks parent ≤ children
- Test that property holds after each push/pop
- Benchmark: should now handle 10k insertions quickly

**Why this isn't enough:**
We're limited to natural ordering (`T: Ord`). What if we want max-heap instead of min-heap? What if we want custom comparison logic (e.g., prioritize by deadline, not arrival time)? The current design can't handle these without code duplication.

---

### Step 3: Add Phantom Types for Min/Max Heap Variants

**Goal:** Use phantom types to support both min-heap and max-heap at compile time.

**What to improve:**
```rust
pub struct MinHeap;
pub struct MaxHeap;

pub struct PriorityQueue<T, Order = MinHeap> {
    heap: Vec<T>,
    _order: PhantomData<Order>,
}

trait HeapOrder {
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
```

Update sift operations to use `Order::should_swap()`.

**Check/Test:**
- Test `PriorityQueue<i32, MinHeap>` returns smallest first
- Test `PriorityQueue<i32, MaxHeap>` returns largest first
- Verify `PhantomData` has zero size with `mem::size_of`

**Why this isn't enough:**
Phantom types work for min/max, but what about more complex orderings? Real systems need custom priorities: tasks with deadlines, events with categories, items with multi-field comparisons. The heap element type and comparison logic are tightly coupled.

---

### Step 4: Support Custom Orderings with Wrapper Types

**Goal:** Allow custom comparison strategies while maintaining type safety.

**What to improve:**
Create wrapper types that implement custom `Ord`:

```rust
// Reverse natural ordering
pub struct Reverse<T>(pub T);

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)  // Reversed!
    }
}
// Also implement PartialOrd, Eq, PartialEq

// Priority by key
pub struct ByKey<T, K, F> {
    pub item: T,
    key_fn: F,
    _key: PhantomData<K>,
}

impl<T, K: Ord, F: Fn(&T) -> K> Ord for ByKey<T, K, F> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.key_fn)(&self.item).cmp(&(other.key_fn)(&other.item))
    }
}
```

**Usage example:**
```rust
// Max heap using Reverse
let mut max_heap = PriorityQueue::<Reverse<i32>>::new();

// Priority by deadline
struct Task { name: String, deadline: u64 }
let mut tasks = PriorityQueue::new();
tasks.push(ByKey::new(task, |t| t.deadline));
```

**Check/Test:**
- Test reverse ordering with `Reverse<T>`
- Create custom comparison for multi-field structs
- Test priority queue of tasks sorted by deadline

**Why this isn't enough:**
Building a heap from an existing collection currently requires pushing N elements one at a time: O(n log n). There's a faster O(n) heapify algorithm. Also, we're doing redundant comparisons—every operation does bounds checking and comparison separately.

---

### Step 5: Implement Efficient Heapify (O(n) from Vec)

**Goal:** Add efficient bulk construction from existing data.

**What to improve:**
```rust
impl<T: Ord, Order> PriorityQueue<T, Order> {
    pub fn from_vec(mut vec: Vec<T>) -> Self {
        // Build heap bottom-up: start from last parent, sift_down all
        let last_parent = vec.len() / 2;
        for i in (0..=last_parent).rev() {
            Self::sift_down_range(&mut vec, i, vec.len());
        }
        PriorityQueue { heap: vec, _order: PhantomData }
    }

    fn sift_down_range(heap: &mut [T], start: usize, end: usize) {
        // Sift down implementation working on a slice
    }
}
```

**Key insight:** Heapify from bottom-up is O(n) because:
- Half the elements are leaves (do nothing)
- Quarter need 1 comparison
- Eighth need 2 comparisons, etc.
- Sum: n * (1/2·0 + 1/4·1 + 1/8·2 + ...) = O(n)

**Check/Test:**
- Test `from_vec()` produces valid heap
- Benchmark: `from_vec()` vs repeated `push()` for 100k elements
- Should see ~2-3x speedup for bulk construction

**Why this isn't enough:**
Performance is good for single-threaded use, but what about memory efficiency? When elements are large structs, we're moving them around in memory. Can we work with references or indices? Also, no iterator support—can't use this with Rust's powerful iterator ecosystem.

---

### Step 6: Add Iterator Support and Memory Optimizations

**Goal:** Make the priority queue work with Rust's iterator ecosystem and optimize memory usage.

**What to improve:**

**1. Iterator implementations:**
```rust
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
```

**2. FromIterator for easy construction:**
```rust
impl<T: Ord, Order> FromIterator<T> for PriorityQueue<T, Order> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        Self::from_vec(vec)
    }
}
```

**3. Memory optimizations:**
- Add `with_capacity(cap: usize)` for pre-allocation
- Add `shrink_to_fit()` to release excess capacity
- Add `reserve(additional: usize)` for growth planning
- Consider `drain()` method for consuming elements

**Check/Test:**
- Test iterator produces elements in sorted order
- Test `collect()` into PriorityQueue
- Test chaining: `values.into_iter().filter(...).collect::<PriorityQueue<_>>()`
- Benchmark memory usage with large structs
- Test iterator `size_hint()` accuracy

**What this achieves:**
Now your priority queue is a first-class Rust collection:
- Works seamlessly with iterator chains
- Memory-efficient construction from iterators
- Predictable performance through capacity management
- Zero-cost abstractions—compiles to the same code as hand-written loops

**Extensions to explore:**
- `Drain` iterator for partial consumption
- `peek_mut()` for in-place modification (tricky—requires sift on drop!)
- Parallel heapify using Rayon
- `merge()` operation for combining two heaps
- `extend()` for bulk insertions

---
