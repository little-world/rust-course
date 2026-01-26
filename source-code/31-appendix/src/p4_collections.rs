// Pattern 4: Collections - Vec, HashMap, HashSet, and More
// Demonstrates Vec, VecDeque, HashMap, HashSet, BTreeMap, BTreeSet, BinaryHeap.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};

// ============================================================================
// Example: Vec - Contiguous Growable Array
// ============================================================================

fn vec_examples() {
    // Creating vectors
    let v1: Vec<i32> = Vec::new();
    let v2 = vec![1, 2, 3];
    let v3: Vec<i32> = Vec::with_capacity(100);
    let v4 = vec![0; 5];

    println!("Empty vec: {:?}", v1);
    println!("vec! macro: {:?}", v2);
    println!("With capacity: len={}, capacity={}", v3.len(), v3.capacity());
    println!("Filled with zeros: {:?}", v4);

    // Adding elements
    let mut numbers = Vec::new();
    numbers.push(1);
    numbers.extend([2, 3, 4]);
    numbers.append(&mut vec![5, 6]);
    numbers.insert(0, 0);
    println!("After adding: {:?}", numbers);

    // Accessing elements
    let first = numbers[0];
    let second = numbers.get(1);
    let last = numbers.last();
    let slice = &numbers[1..4];
    println!("First: {}, Second: {:?}, Last: {:?}", first, second, last);
    println!("Slice [1..4]: {:?}", slice);

    // Removing elements
    let last = numbers.pop();
    println!("Popped: {:?}, remaining: {:?}", last, numbers);

    let removed = numbers.remove(0);
    println!("Removed index 0: {}, remaining: {:?}", removed, numbers);

    // Capacity management
    let mut v = Vec::with_capacity(10);
    v.push(1);
    println!(
        "Capacity management: len={}, capacity={}",
        v.len(),
        v.capacity()
    );
    v.reserve(20);
    println!("After reserve(20): capacity={}", v.capacity());

    // Deduplication and sorting
    let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5];
    data.sort();
    data.dedup();
    println!("Sorted & deduped: {:?}", data);

    // Binary search
    let sorted = vec![1, 2, 3, 4, 5];
    match sorted.binary_search(&3) {
        Ok(index) => println!("Found 3 at index {}", index),
        Err(index) => println!("Not found, would insert at {}", index),
    }
}

// ============================================================================
// Example: VecDeque - Double-Ended Queue
// ============================================================================

fn vecdeque_examples() {
    // Creating VecDeque
    let mut deque = VecDeque::new();
    let deque2 = VecDeque::from(vec![1, 2, 3]);
    println!("From vec: {:?}", deque2);

    // Adding at both ends
    deque.push_back(1);
    deque.push_front(0);
    println!("After push_back(1), push_front(0): {:?}", deque);

    // Removing from both ends
    let back = deque.pop_back();
    let front = deque.pop_front();
    println!("Popped back: {:?}, front: {:?}", back, front);

    // Queue (FIFO)
    let mut queue = VecDeque::new();
    queue.push_back(1);
    queue.push_back(2);
    queue.push_back(3);
    println!("Queue: {:?}", queue);
    println!("FIFO pop: {:?}", queue.pop_front());

    // Stack-like at back (LIFO)
    println!("LIFO pop: {:?}", queue.pop_back());
}

// ============================================================================
// Example: HashMap - Hash-Based Key-Value Store
// ============================================================================

fn hashmap_examples() {
    // Creating HashMaps
    let mut scores = HashMap::new();

    // Inserting and updating
    scores.insert("Alice".to_string(), 10);
    scores.insert("Bob".to_string(), 20);
    let old = scores.insert("Alice".to_string(), 15);
    println!("Inserted Alice twice, old value: {:?}", old);

    // Accessing values
    let alice_score = scores.get("Alice");
    println!("Alice's score: {:?}", alice_score);

    // Safe indexing
    let charlie_score = scores.get("Charlie").unwrap_or(&0);
    println!("Charlie's score (default 0): {}", charlie_score);

    // Checking existence
    if scores.contains_key("Alice") {
        println!("Alice has a score");
    }

    // Removing entries
    let removed = scores.remove("Bob");
    println!("Removed Bob: {:?}", removed);

    // Entry API
    scores.entry("Charlie".to_string()).or_insert(0);
    println!("After or_insert for Charlie: {:?}", scores);

    // Modify with entry
    let alice = scores.entry("Alice".to_string()).or_insert(0);
    *alice += 5;
    println!("After modifying Alice: {:?}", scores);

    // Word counting pattern
    let word_counts = vec!["apple", "banana", "apple"];
    let mut counts = HashMap::new();
    for word in word_counts {
        let count = counts.entry(word).or_insert(0);
        *count += 1;
    }
    println!("Word counts: {:?}", counts);

    // Iteration
    println!("Iterating:");
    for (key, value) in &scores {
        println!("  {}: {}", key, value);
    }

    // Convert from tuples
    let pairs = vec![("a", 1), ("b", 2)];
    let map: HashMap<_, _> = pairs.into_iter().collect();
    println!("From tuples: {:?}", map);
}

// ============================================================================
// Example: HashSet - Hash-Based Set
// ============================================================================

fn hashset_examples() {
    // Creating HashSets
    let mut set = HashSet::new();
    let set2: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
    println!("From array: {:?}", set2);

    // Adding and removing
    let inserted = set.insert(1);
    let duplicate = set.insert(1);
    println!("First insert: {}, duplicate: {}", inserted, duplicate);

    set.insert(2);
    set.insert(3);
    set.remove(&2);
    println!("After remove(2): {:?}", set);

    // Set operations
    let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
    let set2: HashSet<_> = [2, 3, 4].iter().cloned().collect();

    let union: HashSet<_> = set1.union(&set2).cloned().collect();
    println!("Union: {:?}", union);

    let intersection: HashSet<_> = set1.intersection(&set2).cloned().collect();
    println!("Intersection: {:?}", intersection);

    let diff: HashSet<_> = set1.difference(&set2).cloned().collect();
    println!("Difference (set1 - set2): {:?}", diff);

    let sym_diff: HashSet<_> = set1.symmetric_difference(&set2).cloned().collect();
    println!("Symmetric difference: {:?}", sym_diff);

    // Subset and superset
    let small = HashSet::from([1, 2]);
    let large = HashSet::from([1, 2, 3]);
    println!("small is_subset large: {}", small.is_subset(&large));
    println!("large is_superset small: {}", large.is_superset(&small));

    // Deduplication pattern
    let numbers = vec![1, 2, 2, 3, 3, 3];
    let unique: HashSet<_> = numbers.into_iter().collect();
    println!("Deduplicated: {:?}", unique);
}

// ============================================================================
// Example: BTreeMap and BTreeSet - Ordered Collections
// ============================================================================

fn btree_examples() {
    // BTreeMap: Sorted keys
    let mut scores = BTreeMap::new();
    scores.insert("Charlie", 30);
    scores.insert("Alice", 10);
    scores.insert("Bob", 20);

    println!("BTreeMap (sorted by key):");
    for (name, score) in &scores {
        println!("  {}: {}", name, score);
    }

    // Range queries
    let numbers: BTreeMap<i32, &str> =
        [(1, "one"), (5, "five"), (10, "ten")].iter().cloned().collect();

    println!("Range 2..8:");
    for (key, value) in numbers.range(2..8) {
        println!("  {}: {}", key, value);
    }

    // First and last
    println!("First: {:?}", scores.first_key_value());
    println!("Last: {:?}", scores.last_key_value());

    // BTreeSet: Sorted set
    let mut set = BTreeSet::new();
    set.insert(5);
    set.insert(1);
    set.insert(3);

    println!("BTreeSet (sorted):");
    for num in &set {
        print!("{} ", num);
    }
    println!();

    // Range iteration
    print!("Range 2..=5: ");
    for num in set.range(2..=5) {
        print!("{} ", num);
    }
    println!();
}

// ============================================================================
// Example: BinaryHeap - Priority Queue
// ============================================================================

fn binaryheap_examples() {
    let mut heap = BinaryHeap::new();

    // Adding elements
    heap.push(3);
    heap.push(1);
    heap.push(5);
    heap.push(2);

    // Peeking at largest
    println!("Peek (largest): {:?}", heap.peek());

    // Removing (max-heap order)
    print!("Pop order: ");
    while let Some(max) = heap.pop() {
        print!("{} ", max);
    }
    println!();

    // Min-heap using Reverse
    let mut min_heap = BinaryHeap::new();
    min_heap.push(Reverse(3));
    min_heap.push(Reverse(1));
    min_heap.push(Reverse(5));

    print!("Min-heap order: ");
    while let Some(Reverse(min)) = min_heap.pop() {
        print!("{} ", min);
    }
    println!();
}

// ============================================================================
// Example: Priority Queue with Custom Type
// ============================================================================

#[derive(Debug, Eq, PartialEq)]
struct Task {
    priority: u32,
    description: String,
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

fn task_priority_queue() {
    let mut tasks = BinaryHeap::new();
    tasks.push(Task {
        priority: 1,
        description: "Low".to_string(),
    });
    tasks.push(Task {
        priority: 5,
        description: "High".to_string(),
    });
    tasks.push(Task {
        priority: 3,
        description: "Medium".to_string(),
    });

    println!("Processing tasks by priority:");
    while let Some(task) = tasks.pop() {
        println!("  [{:?}] {}", task.priority, task.description);
    }
}

// ============================================================================
// Example: LinkedList
// ============================================================================

fn linkedlist_examples() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_front(0);
    println!("LinkedList: {:?}", list);

    // Splitting and merging
    let mut list1 = LinkedList::from([1, 2, 3]);
    let mut list2 = LinkedList::from([4, 5, 6]);
    list1.append(&mut list2);
    println!("After append: {:?}", list1);
    println!("list2 is now empty: {:?}", list2);

    // Note: LinkedList is rarely optimal!
    println!("(Note: Vec/VecDeque usually better than LinkedList)");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_operations() {
        let mut v = vec![1, 2, 3];
        v.push(4);
        assert_eq!(v, vec![1, 2, 3, 4]);
        assert_eq!(v.pop(), Some(4));
        assert_eq!(v.get(1), Some(&2));
        assert_eq!(v.get(10), None);
    }

    #[test]
    fn test_vec_sort_dedup() {
        let mut v = vec![3, 1, 2, 1, 3];
        v.sort();
        v.dedup();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_vecdeque_operations() {
        let mut dq = VecDeque::new();
        dq.push_back(1);
        dq.push_front(0);
        assert_eq!(dq.pop_front(), Some(0));
        assert_eq!(dq.pop_back(), Some(1));
    }

    #[test]
    fn test_hashmap_operations() {
        let mut map = HashMap::new();
        map.insert("key", 1);
        assert_eq!(map.get("key"), Some(&1));
        assert_eq!(map.get("missing"), None);

        *map.entry("key").or_insert(0) += 1;
        assert_eq!(map.get("key"), Some(&2));
    }

    #[test]
    fn test_hashmap_entry_api() {
        let mut counts: HashMap<&str, i32> = HashMap::new();
        for word in ["a", "b", "a", "c", "a"] {
            *counts.entry(word).or_insert(0) += 1;
        }
        assert_eq!(counts.get("a"), Some(&3));
        assert_eq!(counts.get("b"), Some(&1));
    }

    #[test]
    fn test_hashset_operations() {
        let mut set = HashSet::new();
        assert!(set.insert(1));
        assert!(!set.insert(1));
        assert!(set.contains(&1));
        assert!(set.remove(&1));
        assert!(!set.contains(&1));
    }

    #[test]
    fn test_hashset_set_operations() {
        let set1: HashSet<i32> = [1, 2, 3].iter().cloned().collect();
        let set2: HashSet<i32> = [2, 3, 4].iter().cloned().collect();

        let union: HashSet<_> = set1.union(&set2).cloned().collect();
        let intersection: HashSet<_> = set1.intersection(&set2).cloned().collect();

        assert_eq!(union.len(), 4);
        assert_eq!(intersection, HashSet::from([2, 3]));
    }

    #[test]
    fn test_btreemap_sorted() {
        let mut map = BTreeMap::new();
        map.insert(3, "c");
        map.insert(1, "a");
        map.insert(2, "b");

        let keys: Vec<_> = map.keys().collect();
        assert_eq!(keys, vec![&1, &2, &3]);
    }

    #[test]
    fn test_btreemap_range() {
        let map: BTreeMap<i32, &str> = [(1, "a"), (2, "b"), (3, "c"), (4, "d")]
            .iter()
            .cloned()
            .collect();

        let in_range: Vec<_> = map.range(2..4).map(|(&k, _)| k).collect();
        assert_eq!(in_range, vec![2, 3]);
    }

    #[test]
    fn test_binaryheap_max() {
        let mut heap = BinaryHeap::from(vec![1, 5, 3, 2, 4]);
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(4));
    }

    #[test]
    fn test_binaryheap_min() {
        let mut heap: BinaryHeap<Reverse<i32>> =
            vec![1, 5, 3].into_iter().map(Reverse).collect();
        assert_eq!(heap.pop(), Some(Reverse(1)));
        assert_eq!(heap.pop(), Some(Reverse(3)));
    }

    #[test]
    fn test_task_priority() {
        let mut heap = BinaryHeap::new();
        heap.push(Task {
            priority: 1,
            description: "low".to_string(),
        });
        heap.push(Task {
            priority: 5,
            description: "high".to_string(),
        });

        assert_eq!(heap.pop().unwrap().description, "high");
    }
}

fn main() {
    println!("Pattern 4: Collections Reference");
    println!("=================================\n");

    println!("=== Vec ===");
    vec_examples();
    println!();

    println!("=== VecDeque ===");
    vecdeque_examples();
    println!();

    println!("=== HashMap ===");
    hashmap_examples();
    println!();

    println!("=== HashSet ===");
    hashset_examples();
    println!();

    println!("=== BTreeMap & BTreeSet ===");
    btree_examples();
    println!();

    println!("=== BinaryHeap ===");
    binaryheap_examples();
    println!();

    println!("=== Task Priority Queue ===");
    task_priority_queue();
    println!();

    println!("=== LinkedList ===");
    linkedlist_examples();
}
