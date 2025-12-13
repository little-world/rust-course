use std::cmp::Ordering;
use std::iter::FromIterator;
use std::marker::PhantomData;

// =============================================================================
// Milestone 3: Phantom types drive heap ordering (MinHeap / MaxHeap)
// =============================================================================

pub struct MinHeap;
pub struct MaxHeap;

pub trait HeapOrder {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool;
}

impl HeapOrder for MinHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent > child
    }
}

impl HeapOrder for MaxHeap {
    fn should_swap<T: Ord>(parent: &T, child: &T) -> bool {
        parent < child
    }
}

// =============================================================================
// Milestone 1: Basic generic structure and queue API
// =============================================================================

pub struct PriorityQueue<T, Order = MinHeap> {
    items: Vec<T>,
    _order: PhantomData<Order>,
}

impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    pub fn new() -> Self {
        PriorityQueue {
            items: Vec::new(),
            _order: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.first()
    }

    // =============================================================================
    // Milestone 2: Binary heap helpers (array-based tree utilities)
    // =============================================================================
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
            if !Order::should_swap(&self.items[parent], &self.items[i]) {
                break;
            }
            self.items.swap(i, parent);
            i = parent;
        }
    }

    fn sift_down(&mut self, mut i: usize) {
        loop {
            let left = Self::left_child(i);
            let right = Self::right_child(i);
            let mut swap_with = i;

            if left < self.items.len() && Order::should_swap(&self.items[swap_with], &self.items[left])
            {
                swap_with = left;
            }
            if right < self.items.len() && Order::should_swap(&self.items[swap_with], &self.items[right])
            {
                swap_with = right;
            }

            if swap_with == i {
                break;
            }

            self.items.swap(i, swap_with);
            i = swap_with;
        }
    }

    pub fn push(&mut self, item: T) {
        self.items.push(item);
        let last = self.items.len() - 1;
        self.sift_up(last);
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            return None;
        }

        let len = self.items.len();
        self.items.swap(0, len - 1);
        let result = self.items.pop();

        if !self.items.is_empty() {
            self.sift_down(0);
        }

        result
    }
}

// =============================================================================
// Milestone 4: Wrapper types for custom ordering strategies
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reverse<T>(pub T);

impl<T: Ord> Ord for Reverse<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl<T: PartialOrd> PartialOrd for Reverse<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Task {
    pub name: String,
    pub priority: u8,
    pub deadline: u64,
}

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

// =============================================================================
// Milestone 6: Iterator integration and memory APIs
// =============================================================================

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

impl<T: Ord, Order: HeapOrder> FromIterator<T> for PriorityQueue<T, Order> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        Self::from_vec(vec)
    }
}

impl<T: Ord, Order: HeapOrder> Extend<T> for PriorityQueue<T, Order> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T: Ord, Order: HeapOrder> PriorityQueue<T, Order> {
    /// Milestone 5 helper: shared sift for heapify
    fn sift_down_from(items: &mut Vec<T>, mut i: usize) {
        let len = items.len();
        loop {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            let mut swap_with = i;

            if left < len && Order::should_swap(&items[swap_with], &items[left]) {
                swap_with = left;
            }
            if right < len && Order::should_swap(&items[swap_with], &items[right]) {
                swap_with = right;
            }

            if swap_with == i {
                break;
            }

            items.swap(i, swap_with);
            i = swap_with;
        }
    }

    /// Milestone 5: O(n) heap construction
    pub fn from_vec(mut vec: Vec<T>) -> Self {
        if vec.is_empty() {
            return Self::new();
        }

        let last_parent = (vec.len() / 2).saturating_sub(1);
        for idx in (0..=last_parent).rev() {
            Self::sift_down_from(&mut vec, idx);
        }

        PriorityQueue {
            items: vec,
            _order: PhantomData,
        }
    }

    // =============================================================================
    // Milestone 6: Memory management utilities
    // =============================================================================

    pub fn with_capacity(capacity: usize) -> Self {
        PriorityQueue {
            items: Vec::with_capacity(capacity),
            _order: PhantomData,
        }
    }

    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.items.shrink_to_fit();
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

// =============================================================================
// Tests covering milestones 1-6
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- Milestone 1 tests -----
    #[test]
    fn test_basic_push_pop_order() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();

        pq.push(5);
        pq.push(1);
        pq.push(3);
        pq.push(7);
        pq.push(2);

        assert_eq!(pq.pop(), Some(7));
        assert_eq!(pq.pop(), Some(5));
        assert_eq!(pq.pop(), Some(3));
        assert_eq!(pq.pop(), Some(2));
        assert_eq!(pq.pop(), Some(1));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn test_with_different_types() {
        let mut int_queue: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        int_queue.push(10);
        int_queue.push(5);
        assert_eq!(int_queue.peek(), Some(&10));

        let mut string_queue: PriorityQueue<String, MaxHeap> = PriorityQueue::new();
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
        struct Job {
            priority: u32,
            name: String,
        }

        impl Ord for Job {
            fn cmp(&self, other: &Self) -> Ordering {
                self.priority.cmp(&other.priority)
            }
        }

        impl PartialOrd for Job {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut jobs: PriorityQueue<Job, MaxHeap> = PriorityQueue::new();
        jobs.push(Job { priority: 5, name: "Medium".into() });
        jobs.push(Job { priority: 10, name: "High".into() });
        jobs.push(Job { priority: 1, name: "Low".into() });

        assert_eq!(jobs.pop().unwrap().priority, 10);
        assert_eq!(jobs.pop().unwrap().priority, 5);
        assert_eq!(jobs.pop().unwrap().priority, 1);
    }

    #[test]
    fn test_peek_does_not_remove() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        pq.push(42);
        pq.push(17);

        assert_eq!(pq.peek(), Some(&42));
        assert_eq!(pq.len(), 2);
        assert_eq!(pq.peek(), Some(&42));

        assert_eq!(pq.pop(), Some(42));
        assert_eq!(pq.len(), 1);
    }

    #[test]
    fn test_empty_queue() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
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
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        pq.push(5);
        pq.push(5);
        pq.push(5);
        pq.push(3);

        assert_eq!(pq.pop(), Some(5));
        assert_eq!(pq.pop(), Some(5));
        assert_eq!(pq.pop(), Some(5));
        assert_eq!(pq.pop(), Some(3));
    }

    // ----- Milestone 2 tests -----
    #[test]
    fn test_heap_property_maintained() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();

        for &val in &[5, 3, 7, 1, 9, 4, 8] {
            pq.push(val);
            assert!(verify_heap_property(&pq));
        }

        while !pq.is_empty() {
            pq.pop();
            assert!(verify_heap_property(&pq));
        }
    }

    fn verify_heap_property<T: Ord>(pq: &PriorityQueue<T, MaxHeap>) -> bool {
        for i in 0..pq.len() {
            let left = 2 * i + 1;
            let right = 2 * i + 2;

            if left < pq.len() && pq.items[i] < pq.items[left] {
                return false;
            }
            if right < pq.len() && pq.items[i] < pq.items[right] {
                return false;
            }
        }
        true
    }

    #[test]
    fn test_sift_operations_correctness() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();

        pq.push(5);
        pq.push(3);
        pq.push(7);
        pq.push(1);
        pq.push(9);
        pq.push(4);
        pq.push(8);

        assert_eq!(pq.peek(), Some(&9));
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
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        for i in 0..10_000 {
            pq.push(i * 7 % 10_000);
        }

        let mut prev = pq.pop().unwrap();
        for _ in 1..10_000 {
            let curr = pq.pop().unwrap();
            assert!(curr <= prev);
            prev = curr;
        }
    }

    #[test]
    fn test_heap_index_arithmetic() {
        assert_eq!(PriorityQueue::<i32, MaxHeap>::parent(1), 0);
        assert_eq!(PriorityQueue::<i32, MaxHeap>::parent(2), 0);
        assert_eq!(PriorityQueue::<i32, MaxHeap>::parent(3), 1);
        assert_eq!(PriorityQueue::<i32, MaxHeap>::left_child(0), 1);
        assert_eq!(PriorityQueue::<i32, MaxHeap>::right_child(0), 2);
    }

    #[test]
    fn test_single_element() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        pq.push(42);
        assert_eq!(pq.peek(), Some(&42));
        assert_eq!(pq.pop(), Some(42));
        assert_eq!(pq.pop(), None);
    }

    #[test]
    fn test_two_elements() {
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        pq.push(10);
        pq.push(20);
        assert_eq!(pq.pop(), Some(20));
        assert_eq!(pq.pop(), Some(10));
    }

    // ----- Milestone 3 tests -----
    #[test]
    fn test_min_heap_ordering() {
        let mut min_heap: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
        min_heap.push(5);
        min_heap.push(3);
        min_heap.push(7);
        min_heap.push(1);
        min_heap.push(9);

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

        assert_eq!(max_heap.pop(), Some(9));
        assert_eq!(max_heap.pop(), Some(7));
        assert_eq!(max_heap.pop(), Some(5));
        assert_eq!(max_heap.pop(), Some(3));
        assert_eq!(max_heap.pop(), Some(1));
    }

    #[test]
    fn test_default_is_min_heap() {
        let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
        pq.push(10);
        pq.push(5);
        pq.push(15);
        assert_eq!(pq.pop(), Some(5));
    }

    #[test]
    fn test_phantom_data_zero_size() {
        use std::mem;
        assert_eq!(
            mem::size_of::<PriorityQueue<i32, MinHeap>>(),
            mem::size_of::<Vec<i32>>()
        );
        assert_eq!(mem::size_of::<PhantomData<MinHeap>>(), 0);
    }

    #[test]
    fn test_min_heap_with_strings() {
        let mut pq: PriorityQueue<String, MinHeap> = PriorityQueue::new();
        pq.push("zebra".to_string());
        pq.push("apple".to_string());
        pq.push("mango".to_string());
        pq.push("banana".to_string());

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

        assert_eq!(pq.pop(), Some("zebra".to_string()));
        assert_eq!(pq.pop(), Some("mango".to_string()));
        assert_eq!(pq.pop(), Some("apple".to_string()));
    }

    #[test]
    fn test_type_safety() {
        let _min: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
        let _max: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
    }

    #[test]
    fn test_peek_respects_ordering() {
        let mut min_heap: PriorityQueue<i32, MinHeap> = PriorityQueue::new();
        min_heap.push(10);
        min_heap.push(5);
        min_heap.push(15);
        assert_eq!(min_heap.peek(), Some(&5));

        let mut max_heap: PriorityQueue<i32, MaxHeap> = PriorityQueue::new();
        max_heap.push(10);
        max_heap.push(5);
        max_heap.push(15);
        assert_eq!(max_heap.peek(), Some(&15));
    }

    // ----- Milestone 4 tests -----
    #[test]
    fn test_reverse_wrapper() {
        let mut pq: PriorityQueue<Reverse<i32>, MinHeap> = PriorityQueue::new();
        pq.push(Reverse(5));
        pq.push(Reverse(3));
        pq.push(Reverse(7));
        pq.push(Reverse(1));

        assert_eq!(pq.pop().unwrap().0, 7);
        assert_eq!(pq.pop().unwrap().0, 5);
        assert_eq!(pq.pop().unwrap().0, 3);
        assert_eq!(pq.pop().unwrap().0, 1);
    }

    #[test]
    fn test_task_by_priority() {
        let mut tasks: PriorityQueue<ByField<Task, fn(&Task) -> u8>, MinHeap> = PriorityQueue::new();
        tasks.push(ByField::new(Task { name: "Low".into(), priority: 1, deadline: 100 }, |t: &Task| t.priority));
        tasks.push(ByField::new(Task { name: "High".into(), priority: 10, deadline: 50 }, |t: &Task| t.priority));
        tasks.push(ByField::new(Task { name: "Medium".into(), priority: 5, deadline: 75 }, |t: &Task| t.priority));

        assert_eq!(tasks.pop().unwrap().item.priority, 1);
        assert_eq!(tasks.pop().unwrap().item.priority, 5);
        assert_eq!(tasks.pop().unwrap().item.priority, 10);
    }

    #[test]
    fn test_task_by_deadline() {
        let mut tasks: PriorityQueue<ByField<Task, fn(&Task) -> u64>, MinHeap> = PriorityQueue::new();
        tasks.push(ByField::new(Task { name: "Later".into(), priority: 10, deadline: 200 }, |t: &Task| t.deadline));
        tasks.push(ByField::new(Task { name: "Soon".into(), priority: 1, deadline: 50 }, |t: &Task| t.deadline));
        tasks.push(ByField::new(Task { name: "Middle".into(), priority: 5, deadline: 100 }, |t: &Task| t.deadline));

        assert_eq!(tasks.pop().unwrap().item.deadline, 50);
        assert_eq!(tasks.pop().unwrap().item.deadline, 100);
        assert_eq!(tasks.pop().unwrap().item.deadline, 200);
    }

    #[test]
    fn test_multi_field_comparison() {
        #[derive(Debug, Clone, Eq, PartialEq)]
        struct Event {
            severity: u8,
            timestamp: u64,
        }

        impl Ord for Event {
            fn cmp(&self, other: &Self) -> Ordering {
                other.severity.cmp(&self.severity).then(self.timestamp.cmp(&other.timestamp))
            }
        }

        impl PartialOrd for Event {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut events: PriorityQueue<Event, MinHeap> = PriorityQueue::new();
        events.push(Event { severity: 5, timestamp: 100 });
        events.push(Event { severity: 10, timestamp: 50 });
        events.push(Event { severity: 10, timestamp: 75 });
        events.push(Event { severity: 3, timestamp: 25 });

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

        assert_eq!(tasks.pop().unwrap().0.name, "Z");
        assert_eq!(tasks.pop().unwrap().0.name, "M");
        assert_eq!(tasks.pop().unwrap().0.name, "A");
    }

    #[test]
    fn test_wrapper_zero_cost() {
        use std::mem;
        assert_eq!(mem::size_of::<Reverse<i32>>(), mem::size_of::<i32>());
        assert_eq!(mem::size_of::<Reverse<String>>(), mem::size_of::<String>());
    }

    #[test]
    fn test_chained_wrappers() {
        let mut pq: PriorityQueue<Reverse<ByField<Task, fn(&Task) -> u8>>, MinHeap> = PriorityQueue::new();
        pq.push(Reverse(ByField::new(
            Task { name: "Low".into(), priority: 1, deadline: 100 },
            |t: &Task| t.priority,
        )));
        pq.push(Reverse(ByField::new(
            Task { name: "High".into(), priority: 10, deadline: 50 },
            |t: &Task| t.priority,
        )));

        assert_eq!(pq.pop().unwrap().0.item.priority, 10);
        assert_eq!(pq.pop().unwrap().0.item.priority, 1);
    }

    // ----- Milestone 5 tests -----
    #[test]
    fn test_from_vec_correctness() {
        let vec = vec![5, 3, 7, 1, 9, 4, 8, 2, 6];
        let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);
        let mut result = Vec::new();
        while let Some(val) = pq.pop() {
            result.push(val);
        }
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_from_vec_maintains_heap_property() {
        let vec = vec![15, 3, 17, 10, 84, 19, 6, 22, 9];
        let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);
        for i in 0..pq.len() {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            if left < pq.len() {
                assert!(pq.items[i] <= pq.items[left]);
            }
            if right < pq.len() {
                assert!(pq.items[i] <= pq.items[right]);
            }
        }
    }

    #[test]
    fn test_from_vec_with_max_heap() {
        let vec = vec![5, 3, 7, 1, 9, 4, 8];
        let mut pq: PriorityQueue<i32, MaxHeap> = PriorityQueue::from_vec(vec);
        assert_eq!(pq.pop(), Some(9));
        assert_eq!(pq.pop(), Some(8));
        assert_eq!(pq.pop(), Some(7));
    }

    #[test]
    fn test_from_vec_large_dataset() {
        let vec: Vec<i32> = (0..1000).collect();
        let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec);
        assert_eq!(pq.len(), 1000);
        assert_eq!(pq.peek(), Some(&0));
    }

    #[test]
    fn test_from_vec_empty() {
        let pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(Vec::new());
        assert!(pq.is_empty());
    }

    #[test]
    fn test_from_vec_single_element() {
        let mut pq: PriorityQueue<i32, MinHeap> = PriorityQueue::from_vec(vec![42]);
        assert_eq!(pq.pop(), Some(42));
    }

    // ----- Milestone 6 tests -----
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
        let pq: PriorityQueue<i32, MinHeap> = data
            .into_iter()
            .filter(|x| x % 2 == 0)
            .map(|x| x / 2)
            .collect();

        let result: Vec<i32> = pq.into_iter().collect();
        assert_eq!(result, vec![4, 5, 6, 10]);
    }

    #[test]
    fn test_for_loop_iteration() {
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
        for i in 0..1000 {
            pq.push(i);
        }
        assert_eq!(pq.len(), 1000);
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
        assert_eq!(iter.len(), 3);
    }
}

fn main() {}
