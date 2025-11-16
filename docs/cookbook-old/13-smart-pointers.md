# Comprehensive Guide to Smart Pointers in Rust

Smart pointers are data structures that act like pointers but have additional metadata and capabilities. This guide covers all major smart pointers with practical examples using real data structures and design patterns.

## Table of Contents

1. [Introduction to Smart Pointers](#introduction-to-smart-pointers)
2. [Box\<T\> - Heap Allocation](#boxt---heap-allocation)
3. [Rc\<T\> - Reference Counting](#rct---reference-counting)
4. [Arc\<T\> - Atomic Reference Counting](#arct---atomic-reference-counting)
5. [RefCell\<T\> - Interior Mutability](#refcellt---interior-mutability)
6. [Mutex\<T\> - Thread-Safe Mutability](#mutext---thread-safe-mutability)
7. [RwLock\<T\> - Read-Write Lock](#rwlockt---read-write-lock)
8. [Cell\<T\> - Simple Interior Mutability](#cellt---simple-interior-mutability)
9. [Weak\<T\> - Weak References](#weakt---weak-references)
10. [Cow\<T\> - Clone-On-Write](#cowt---clone-on-write)
11. [Combining Smart Pointers](#combining-smart-pointers)
12. [Design Patterns](#design-patterns)
13. [Performance Considerations](#performance-considerations)
14. [Common Pitfalls](#common-pitfalls)
15. [Quick Reference](#quick-reference)

---

## Introduction to Smart Pointers

### What Are Smart Pointers?

Smart pointers are types that implement `Deref` and often `Drop` traits, providing:
- Automatic memory management
- Custom cleanup behavior
- Additional capabilities beyond raw pointers

```rust
use std::ops::Deref;

fn main() {
    let x = 5;
    let y = Box::new(x);

    // Box implements Deref, so we can use it like a reference
    assert_eq!(5, *y);

    // Automatically cleaned up when y goes out of scope
}
```

### Common Smart Pointers

| Type | Use Case | Thread-Safe | Cost |
|------|----------|-------------|------|
| `Box<T>` | Heap allocation | No | Minimal |
| `Rc<T>` | Multiple ownership | No | Reference counting |
| `Arc<T>` | Shared ownership across threads | Yes | Atomic ref counting |
| `RefCell<T>` | Interior mutability | No | Runtime borrow checking |
| `Mutex<T>` | Thread-safe interior mutability | Yes | Locking overhead |
| `RwLock<T>` | Multiple readers, one writer | Yes | Lock overhead |
| `Cell<T>` | Copy types interior mutability | No | Minimal |
| `Weak<T>` | Non-owning references | Depends | Minimal |

---

## Box\<T\> - Heap Allocation

`Box<T>` allocates data on the heap and provides exclusive ownership.

### Recipe 1: Basic Heap Allocation

**Problem**: Store large data on the heap to avoid stack overflow.

```rust
fn main() {
    // Large array on heap instead of stack
    let large_data = Box::new([0u8; 1_000_000]);

    println!("Data allocated on heap");

    // Automatically deallocated when box goes out of scope
}
```

**When to use**: Large data structures, avoiding stack overflow.

---

### Recipe 2: Recursive Data Structures - Linked List

**Problem**: Create a singly-linked list (recursive type).

```rust
#[derive(Debug)]
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node { value, next: None }
    }

    fn append(&mut self, value: T) {
        match self.next {
            None => self.next = Some(Box::new(Node::new(value))),
            Some(ref mut next) => next.append(value),
        }
    }

    fn iter(&self) -> NodeIter<T> {
        NodeIter { current: Some(self) }
    }
}

struct NodeIter<'a, T> {
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.next.as_deref();
            &node.value
        })
    }
}

fn main() {
    let mut list = Node::new(1);
    list.append(2);
    list.append(3);
    list.append(4);

    print!("List: ");
    for value in list.iter() {
        print!("{} -> ", value);
    }
    println!("None");
}
```

---

### Recipe 3: Binary Search Tree

**Problem**: Implement a binary search tree.

```rust
#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T: Ord> TreeNode<T> {
    fn new(value: T) -> Self {
        TreeNode {
            value,
            left: None,
            right: None,
        }
    }

    fn insert(&mut self, value: T) {
        if value < self.value {
            match self.left {
                None => self.left = Some(Box::new(TreeNode::new(value))),
                Some(ref mut left) => left.insert(value),
            }
        } else {
            match self.right {
                None => self.right = Some(Box::new(TreeNode::new(value))),
                Some(ref mut right) => right.insert(value),
            }
        }
    }

    fn contains(&self, value: &T) -> bool {
        if *value == self.value {
            true
        } else if *value < self.value {
            self.left.as_ref().map_or(false, |left| left.contains(value))
        } else {
            self.right.as_ref().map_or(false, |right| right.contains(value))
        }
    }

    fn inorder_traversal(&self) -> Vec<&T> {
        let mut result = Vec::new();
        if let Some(ref left) = self.left {
            result.extend(left.inorder_traversal());
        }
        result.push(&self.value);
        if let Some(ref right) = self.right {
            result.extend(right.inorder_traversal());
        }
        result
    }
}

fn main() {
    let mut tree = TreeNode::new(5);
    tree.insert(3);
    tree.insert(7);
    tree.insert(1);
    tree.insert(4);
    tree.insert(6);
    tree.insert(9);

    println!("Contains 4: {}", tree.contains(&4));
    println!("Contains 8: {}", tree.contains(&8));

    println!("Inorder traversal: {:?}", tree.inorder_traversal());
    // Output: [1, 3, 4, 5, 6, 7, 9]
}
```

---

### Recipe 4: Trait Objects for Polymorphism

**Problem**: Store different types that implement the same trait.

```rust
trait Shape {
    fn area(&self) -> f64;
    fn describe(&self) -> String;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    fn describe(&self) -> String {
        format!("Circle with radius {}", self.radius)
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn describe(&self) -> String {
        format!("Rectangle {}x{}", self.width, self.height)
    }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 4.0, height: 6.0 }),
        Box::new(Circle { radius: 3.0 }),
    ];

    let total_area: f64 = shapes.iter().map(|s| s.area()).sum();

    for shape in &shapes {
        println!("{}: area = {:.2}", shape.describe(), shape.area());
    }

    println!("Total area: {:.2}", total_area);
}
```

---

## Rc\<T\> - Reference Counting

`Rc<T>` enables multiple ownership through reference counting (single-threaded).

### Recipe 5: Shared Tree Nodes

**Problem**: Multiple parents referencing same child nodes.

```rust
use std::rc::Rc;

#[derive(Debug)]
struct FileSystemNode {
    name: String,
    children: Vec<Rc<FileSystemNode>>,
}

impl FileSystemNode {
    fn new(name: &str) -> Rc<Self> {
        Rc::new(FileSystemNode {
            name: name.to_string(),
            children: vec![],
        })
    }

    fn with_children(name: &str, children: Vec<Rc<FileSystemNode>>) -> Rc<Self> {
        Rc::new(FileSystemNode {
            name: name.to_string(),
            children,
        })
    }

    fn print(&self, indent: usize) {
        println!("{}{}", "  ".repeat(indent), self.name);
        for child in &self.children {
            child.print(indent + 1);
        }
    }
}

fn main() {
    // Shared files
    let shared_lib = FileSystemNode::new("libshared.so");

    // Multiple directories reference the same library
    let bin_dir = FileSystemNode::with_children(
        "bin",
        vec![Rc::clone(&shared_lib)],
    );

    let lib_dir = FileSystemNode::with_children(
        "lib",
        vec![Rc::clone(&shared_lib)],
    );

    let root = FileSystemNode::with_children(
        "root",
        vec![bin_dir, lib_dir],
    );

    println!("Reference count for shared_lib: {}", Rc::strong_count(&shared_lib));
    // Output: 3 (original + 2 clones)

    root.print(0);
}
```

---

### Recipe 6: Graph with Shared Vertices

**Problem**: Implement a directed graph with shared nodes.

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct GraphNode {
    id: usize,
    value: String,
    edges: RefCell<Vec<Rc<GraphNode>>>,
}

impl GraphNode {
    fn new(id: usize, value: String) -> Rc<Self> {
        Rc::new(GraphNode {
            id,
            value,
            edges: RefCell::new(vec![]),
        })
    }

    fn add_edge(&self, target: Rc<GraphNode>) {
        self.edges.borrow_mut().push(target);
    }

    fn neighbors(&self) -> Vec<Rc<GraphNode>> {
        self.edges.borrow().clone()
    }
}

fn main() {
    // Create nodes
    let node1 = GraphNode::new(1, "A".to_string());
    let node2 = GraphNode::new(2, "B".to_string());
    let node3 = GraphNode::new(3, "C".to_string());
    let node4 = GraphNode::new(4, "D".to_string());

    // Build graph: A -> B, A -> C, B -> D, C -> D
    node1.add_edge(Rc::clone(&node2));
    node1.add_edge(Rc::clone(&node3));
    node2.add_edge(Rc::clone(&node4));
    node3.add_edge(Rc::clone(&node4));

    // Traverse
    println!("Node {}: neighbors = {:?}",
        node1.id,
        node1.neighbors().iter().map(|n| &n.value).collect::<Vec<_>>()
    );

    println!("Reference count for node4: {}", Rc::strong_count(&node4));
    // Output: 3 (original + 2 edges)
}
```

---

### Recipe 7: Immutable Shared Data Cache

**Problem**: Share expensive-to-compute data across multiple consumers.

```rust
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Report {
    data: String,
    computed_value: f64,
}

impl Report {
    fn generate(id: u32) -> Self {
        // Simulate expensive computation
        println!("Generating report {}...", id);
        Report {
            data: format!("Report #{}", id),
            computed_value: (id as f64).sqrt(),
        }
    }
}

struct ReportCache {
    cache: HashMap<u32, Rc<Report>>,
}

impl ReportCache {
    fn new() -> Self {
        ReportCache {
            cache: HashMap::new(),
        }
    }

    fn get(&mut self, id: u32) -> Rc<Report> {
        self.cache.entry(id)
            .or_insert_with(|| Rc::new(Report::generate(id)))
            .clone()
    }
}

fn main() {
    let mut cache = ReportCache::new();

    // First access - generates report
    let report1 = cache.get(42);
    println!("Got: {:?}", report1);

    // Second access - uses cached version
    let report2 = cache.get(42);
    println!("Got: {:?}", report2);

    // Same data, same memory location
    println!("Same instance: {}", Rc::ptr_eq(&report1, &report2));
    println!("Reference count: {}", Rc::strong_count(&report1));
}
```

---

## Arc\<T\> - Atomic Reference Counting

`Arc<T>` is the thread-safe version of `Rc<T>`, using atomic operations.

### Recipe 8: Shared Configuration Across Threads

**Problem**: Share read-only configuration across multiple threads.

```rust
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone)]
struct Config {
    max_connections: usize,
    timeout_ms: u64,
    server_name: String,
}

fn main() {
    let config = Arc::new(Config {
        max_connections: 100,
        timeout_ms: 5000,
        server_name: "MyServer".to_string(),
    });

    let mut handles = vec![];

    for i in 0..5 {
        let config_clone = Arc::clone(&config);

        let handle = thread::spawn(move || {
            println!(
                "Thread {}: Using config: {} (max conn: {})",
                i, config_clone.server_name, config_clone.max_connections
            );
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final reference count: {}", Arc::strong_count(&config));
}
```

---

### Recipe 9: Thread Pool with Shared Task Queue

**Problem**: Implement a simple thread pool with shared state.

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct TaskQueue {
    tasks: Mutex<Vec<String>>,
}

impl TaskQueue {
    fn new() -> Arc<Self> {
        Arc::new(TaskQueue {
            tasks: Mutex::new(vec![]),
        })
    }

    fn add_task(&self, task: String) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
    }

    fn get_task(&self) -> Option<String> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.pop()
    }
}

fn worker(id: usize, queue: Arc<TaskQueue>) {
    loop {
        match queue.get_task() {
            Some(task) => {
                println!("Worker {} processing: {}", id, task);
                thread::sleep(Duration::from_millis(100));
            }
            None => {
                thread::sleep(Duration::from_millis(50));
                break;
            }
        }
    }
    println!("Worker {} finished", id);
}

fn main() {
    let queue = TaskQueue::new();

    // Add tasks
    for i in 1..=10 {
        queue.add_task(format!("Task {}", i));
    }

    // Spawn workers
    let mut handles = vec![];
    for i in 0..3 {
        let queue_clone = Arc::clone(&queue);
        let handle = thread::spawn(move || worker(i, queue_clone));
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("All tasks completed!");
}
```

---

### Recipe 10: Parallel Data Processing

**Problem**: Process data in parallel with shared results collection.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let results = Arc::new(Mutex::new(Vec::new()));

    let chunk_size = 3;
    let chunks: Vec<_> = data.chunks(chunk_size).collect();

    let mut handles = vec![];

    for chunk in chunks {
        let results_clone = Arc::clone(&results);
        let chunk = chunk.to_vec();

        let handle = thread::spawn(move || {
            let processed: Vec<i32> = chunk.iter().map(|x| x * x).collect();

            let mut results = results_clone.lock().unwrap();
            results.extend(processed);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let final_results = results.lock().unwrap();
    println!("Results: {:?}", *final_results);
}
```

---

## RefCell\<T\> - Interior Mutability

`RefCell<T>` allows mutation of data even through immutable references (runtime borrow checking).

### Recipe 11: Mutable Tree Traversal

**Problem**: Modify tree nodes during traversal without mutable references.

```rust
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
struct TreeNode {
    value: i32,
    visit_count: RefCell<usize>,
    left: Option<Rc<TreeNode>>,
    right: Option<Rc<TreeNode>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            visit_count: RefCell::new(0),
            left: None,
            right: None,
        })
    }

    fn with_children(value: i32, left: Option<Rc<TreeNode>>, right: Option<Rc<TreeNode>>) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            visit_count: RefCell::new(0),
            left,
            right,
        })
    }

    fn visit(&self) {
        *self.visit_count.borrow_mut() += 1;
    }

    fn traverse(&self) {
        self.visit();
        if let Some(ref left) = self.left {
            left.traverse();
        }
        if let Some(ref right) = self.right {
            right.traverse();
        }
    }

    fn get_visit_count(&self) -> usize {
        *self.visit_count.borrow()
    }
}

fn main() {
    let leaf1 = TreeNode::new(3);
    let leaf2 = TreeNode::new(7);
    let branch = TreeNode::with_children(5, Some(Rc::clone(&leaf1)), Some(Rc::clone(&leaf2)));

    // Traverse multiple times
    branch.traverse();
    branch.traverse();
    leaf1.traverse(); // Additional visit to leaf1

    println!("Branch visited: {} times", branch.get_visit_count());
    println!("Leaf1 visited: {} times", leaf1.get_visit_count());
    println!("Leaf2 visited: {} times", leaf2.get_visit_count());
}
```

---

### Recipe 12: Observer Pattern

**Problem**: Implement the observer pattern with runtime mutability.

```rust
use std::cell::RefCell;
use std::rc::Rc;

trait Observer {
    fn update(&self, message: &str);
}

struct ConcreteObserver {
    id: usize,
    messages: RefCell<Vec<String>>,
}

impl ConcreteObserver {
    fn new(id: usize) -> Self {
        ConcreteObserver {
            id,
            messages: RefCell::new(vec![]),
        }
    }

    fn get_messages(&self) -> Vec<String> {
        self.messages.borrow().clone()
    }
}

impl Observer for ConcreteObserver {
    fn update(&self, message: &str) {
        println!("Observer {} received: {}", self.id, message);
        self.messages.borrow_mut().push(message.to_string());
    }
}

struct Subject {
    observers: RefCell<Vec<Rc<dyn Observer>>>,
}

impl Subject {
    fn new() -> Self {
        Subject {
            observers: RefCell::new(vec![]),
        }
    }

    fn attach(&self, observer: Rc<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }

    fn notify(&self, message: &str) {
        for observer in self.observers.borrow().iter() {
            observer.update(message);
        }
    }
}

fn main() {
    let subject = Subject::new();

    let observer1 = Rc::new(ConcreteObserver::new(1));
    let observer2 = Rc::new(ConcreteObserver::new(2));

    subject.attach(Rc::clone(&observer1) as Rc<dyn Observer>);
    subject.attach(Rc::clone(&observer2) as Rc<dyn Observer>);

    subject.notify("Event A occurred");
    subject.notify("Event B occurred");

    println!("\nObserver 1 messages: {:?}", observer1.get_messages());
    println!("Observer 2 messages: {:?}", observer2.get_messages());
}
```

---

### Recipe 13: Lazy Initialization

**Problem**: Initialize data on first access.

```rust
use std::cell::RefCell;

struct LazyValue<T> {
    value: RefCell<Option<T>>,
    init: fn() -> T,
}

impl<T> LazyValue<T> {
    fn new(init: fn() -> T) -> Self {
        LazyValue {
            value: RefCell::new(None),
            init,
        }
    }

    fn get(&self) -> std::cell::Ref<T> {
        if self.value.borrow().is_none() {
            println!("Initializing value...");
            *self.value.borrow_mut() = Some((self.init)());
        }

        std::cell::Ref::map(self.value.borrow(), |opt| {
            opt.as_ref().unwrap()
        })
    }
}

fn expensive_computation() -> Vec<i32> {
    println!("Computing expensive value...");
    (1..=10).collect()
}

fn main() {
    let lazy = LazyValue::new(expensive_computation);

    println!("LazyValue created, but not initialized yet");

    // First access triggers initialization
    println!("First access: {:?}", *lazy.get());

    // Subsequent accesses use cached value
    println!("Second access: {:?}", *lazy.get());
}
```

---

## Mutex\<T\> - Thread-Safe Mutability

`Mutex<T>` provides mutual exclusion for thread-safe interior mutability.

### Recipe 14: Shared Counter

**Problem**: Increment a counter from multiple threads safely.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for i in 0..10 {
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let mut num = counter_clone.lock().unwrap();
                *num += 1;
            }
            println!("Thread {} finished", i);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
    // Output: 1000
}
```

---

### Recipe 15: Thread-Safe Cache

**Problem**: Implement a concurrent cache with LRU eviction.

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

struct Cache<K, V> {
    data: Mutex<HashMap<K, V>>,
    max_size: usize,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    fn new(max_size: usize) -> Arc<Self> {
        Arc::new(Cache {
            data: Mutex::new(HashMap::new()),
            max_size,
        })
    }

    fn get(&self, key: &K) -> Option<V> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    fn insert(&self, key: K, value: V) {
        let mut data = self.data.lock().unwrap();

        if data.len() >= self.max_size && !data.contains_key(&key) {
            // Simple eviction: remove first key (not true LRU, just for demo)
            if let Some(first_key) = data.keys().next().cloned() {
                data.remove(&first_key);
            }
        }

        data.insert(key, value);
    }

    fn size(&self) -> usize {
        self.data.lock().unwrap().len()
    }
}

fn main() {
    let cache = Cache::new(100);
    let mut handles = vec![];

    // Multiple threads inserting
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache);

        let handle = thread::spawn(move || {
            for j in 0..50 {
                let key = i * 50 + j;
                cache_clone.insert(key, format!("Value {}", key));
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Cache size: {}", cache.size());
    println!("Sample value: {:?}", cache.get(&42));
}
```

---

### Recipe 16: Producer-Consumer Pattern

**Problem**: Implement producer-consumer with a bounded buffer.

```rust
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

struct BoundedBuffer<T> {
    buffer: Mutex<VecDeque<T>>,
    not_empty: Condvar,
    not_full: Condvar,
    capacity: usize,
}

impl<T> BoundedBuffer<T> {
    fn new(capacity: usize) -> Arc<Self> {
        Arc::new(BoundedBuffer {
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        })
    }

    fn push(&self, item: T) {
        let mut buffer = self.buffer.lock().unwrap();

        // Wait while buffer is full
        while buffer.len() >= self.capacity {
            buffer = self.not_full.wait(buffer).unwrap();
        }

        buffer.push_back(item);
        self.not_empty.notify_one();
    }

    fn pop(&self) -> T {
        let mut buffer = self.buffer.lock().unwrap();

        // Wait while buffer is empty
        while buffer.is_empty() {
            buffer = self.not_empty.wait(buffer).unwrap();
        }

        let item = buffer.pop_front().unwrap();
        self.not_full.notify_one();
        item
    }
}

fn main() {
    let buffer = BoundedBuffer::new(5);

    // Producer
    let buffer_clone = Arc::clone(&buffer);
    let producer = thread::spawn(move || {
        for i in 1..=10 {
            println!("Producing {}", i);
            buffer_clone.push(i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    // Consumer
    let buffer_clone = Arc::clone(&buffer);
    let consumer = thread::spawn(move || {
        for _ in 1..=10 {
            let item = buffer_clone.pop();
            println!("Consumed {}", item);
            thread::sleep(Duration::from_millis(150));
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

---

## RwLock\<T\> - Read-Write Lock

`RwLock<T>` allows multiple readers or one writer at a time.

### Recipe 17: Read-Heavy Cache

**Problem**: Optimize for many concurrent reads with occasional writes.

```rust
use std::sync::{Arc, RwLock};
use std::thread;
use std::collections::HashMap;

struct ReadHeavyCache {
    data: RwLock<HashMap<String, String>>,
}

impl ReadHeavyCache {
    fn new() -> Arc<Self> {
        Arc::new(ReadHeavyCache {
            data: RwLock::new(HashMap::new()),
        })
    }

    fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    fn insert(&self, key: String, value: String) {
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
    }

    fn update(&self, key: &str, value: String) -> bool {
        let mut data = self.data.write().unwrap();
        if data.contains_key(key) {
            data.insert(key.to_string(), value);
            true
        } else {
            false
        }
    }
}

fn main() {
    let cache = ReadHeavyCache::new();

    // Populate cache
    cache.insert("user:1".to_string(), "Alice".to_string());
    cache.insert("user:2".to_string(), "Bob".to_string());

    let mut handles = vec![];

    // Many readers
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let value = cache_clone.get("user:1");
                if i == 0 && value.is_some() {
                    println!("Reader {}: {:?}", i, value);
                }
            }
        });

        handles.push(handle);
    }

    // One writer
    let cache_clone = Arc::clone(&cache);
    let writer = thread::spawn(move || {
        for i in 0..10 {
            cache_clone.update("user:1", format!("Alice-{}", i));
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    handles.push(writer);

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final value: {:?}", cache.get("user:1"));
}
```

---

## Cell\<T\> - Simple Interior Mutability

`Cell<T>` provides interior mutability for `Copy` types without runtime checks.

### Recipe 18: Reference Counting Implementation

**Problem**: Implement a simple reference counter.

```rust
use std::cell::Cell;

struct RefCounter {
    count: Cell<usize>,
}

impl RefCounter {
    fn new() -> Self {
        RefCounter {
            count: Cell::new(0),
        }
    }

    fn increment(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn decrement(&self) {
        self.count.set(self.count.get() - 1);
    }

    fn get(&self) -> usize {
        self.count.get()
    }
}

fn main() {
    let counter = RefCounter::new();

    println!("Initial: {}", counter.get());

    counter.increment();
    counter.increment();
    counter.increment();

    println!("After increments: {}", counter.get());

    counter.decrement();

    println!("After decrement: {}", counter.get());
}
```

---

### Recipe 19: Interior Mutability for Flags

**Problem**: Toggle flags without mutable references.

```rust
use std::cell::Cell;

struct Connection {
    id: usize,
    connected: Cell<bool>,
    retry_count: Cell<u32>,
}

impl Connection {
    fn new(id: usize) -> Self {
        Connection {
            id,
            connected: Cell::new(false),
            retry_count: Cell::new(0),
        }
    }

    fn connect(&self) -> Result<(), &'static str> {
        if self.retry_count.get() >= 3 {
            return Err("Max retries exceeded");
        }

        // Simulate connection attempt
        if self.id % 2 == 0 {
            self.connected.set(true);
            self.retry_count.set(0);
            Ok(())
        } else {
            self.retry_count.set(self.retry_count.get() + 1);
            Err("Connection failed")
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.get()
    }

    fn disconnect(&self) {
        self.connected.set(false);
    }
}

fn main() {
    let conn1 = Connection::new(2);
    let conn2 = Connection::new(3);

    match conn1.connect() {
        Ok(_) => println!("Connection 1: Connected"),
        Err(e) => println!("Connection 1: {}", e),
    }

    for _ in 0..4 {
        match conn2.connect() {
            Ok(_) => println!("Connection 2: Connected"),
            Err(e) => println!("Connection 2: {}", e),
        }
    }

    println!("Conn1 connected: {}", conn1.is_connected());
    println!("Conn2 connected: {}", conn2.is_connected());
}
```

---

## Weak\<T\> - Weak References

`Weak<T>` creates non-owning references that don't prevent deallocation.

### Recipe 20: Avoiding Circular References in Trees

**Problem**: Create parent-child relationships without memory leaks.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

#[derive(Debug)]
struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }

    fn add_child(parent: &Rc<Node>, child: Rc<Node>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    fn get_parent(&self) -> Option<Rc<Node>> {
        self.parent.borrow().upgrade()
    }
}

fn main() {
    let root = Node::new(1);
    let child1 = Node::new(2);
    let child2 = Node::new(3);
    let grandchild = Node::new(4);

    Node::add_child(&root, Rc::clone(&child1));
    Node::add_child(&root, Rc::clone(&child2));
    Node::add_child(&child1, Rc::clone(&grandchild));

    println!("Root value: {}", root.value);
    println!("Root children count: {}", root.children.borrow().len());

    // Access parent from child
    if let Some(parent) = grandchild.get_parent() {
        println!("Grandchild's parent: {}", parent.value);
    }

    // Check reference counts
    println!("Root strong count: {}", Rc::strong_count(&root));
    println!("Child1 strong count: {}", Rc::strong_count(&child1));
    println!("Grandchild strong count: {}", Rc::strong_count(&grandchild));
}
```

---

### Recipe 21: Cache with Weak References

**Problem**: Implement a cache that doesn't prevent garbage collection.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

struct WeakCache<K, V> {
    cache: RefCell<HashMap<K, Weak<V>>>,
}

impl<K: Eq + std::hash::Hash + Clone, V> WeakCache<K, V> {
    fn new() -> Self {
        WeakCache {
            cache: RefCell::new(HashMap::new()),
        }
    }

    fn get(&self, key: &K) -> Option<Rc<V>> {
        let mut cache = self.cache.borrow_mut();

        if let Some(weak) = cache.get(key) {
            if let Some(strong) = weak.upgrade() {
                return Some(strong);
            } else {
                // Value was dropped, remove from cache
                cache.remove(key);
            }
        }

        None
    }

    fn insert(&self, key: K, value: Rc<V>) {
        let weak = Rc::downgrade(&value);
        self.cache.borrow_mut().insert(key, weak);
    }

    fn cleanup(&self) {
        let mut cache = self.cache.borrow_mut();
        cache.retain(|_, weak| weak.strong_count() > 0);
    }

    fn size(&self) -> usize {
        self.cache.borrow().len()
    }
}

fn main() {
    let cache = WeakCache::new();

    {
        let data1 = Rc::new("Data 1".to_string());
        let data2 = Rc::new("Data 2".to_string());

        cache.insert(1, Rc::clone(&data1));
        cache.insert(2, Rc::clone(&data2));

        println!("Cache size: {}", cache.size());
        println!("Get 1: {:?}", cache.get(&1));

        // data1 and data2 dropped here
    }

    println!("\nAfter values dropped:");
    println!("Cache size before cleanup: {}", cache.size());
    println!("Get 1: {:?}", cache.get(&1)); // None - value was dropped

    cache.cleanup();
    println!("Cache size after cleanup: {}", cache.size());
}
```

---

## Cow\<T\> - Clone-On-Write

`Cow<T>` provides clone-on-write semantics, avoiding unnecessary clones.

### Recipe 22: Efficient String Processing

**Problem**: Process strings efficiently, only cloning when necessary.

```rust
use std::borrow::Cow;

fn process_text(input: &str) -> Cow<str> {
    if input.contains("ERROR") {
        // Need to modify - clone and create owned
        Cow::Owned(input.replace("ERROR", "WARNING"))
    } else {
        // No modification needed - return borrowed
        Cow::Borrowed(input)
    }
}

fn normalize_path(path: &str) -> Cow<str> {
    if path.contains('\\') {
        // Windows path - convert to Unix
        Cow::Owned(path.replace('\\', "/"))
    } else {
        // Already Unix style
        Cow::Borrowed(path)
    }
}

fn main() {
    let logs = vec![
        "INFO: System started",
        "ERROR: Connection failed",
        "INFO: Request processed",
        "ERROR: Timeout occurred",
    ];

    for log in logs {
        let processed = process_text(log);
        println!("{} -> {}", log, processed);

        // Check if it was cloned
        match processed {
            Cow::Borrowed(_) => println!("  (borrowed)"),
            Cow::Owned(_) => println!("  (owned - cloned)"),
        }
    }

    println!("\nPath normalization:");
    let paths = vec![
        "/home/user/file.txt",
        "C:\\Users\\User\\file.txt",
    ];

    for path in paths {
        let normalized = normalize_path(path);
        println!("{} -> {}", path, normalized);
    }
}
```

---

### Recipe 23: Configuration with Default Values

**Problem**: Use default config but allow overrides without cloning.

```rust
use std::borrow::Cow;
use std::collections::HashMap;

struct Config<'a> {
    settings: HashMap<String, Cow<'a, str>>,
}

impl<'a> Config<'a> {
    fn new() -> Self {
        let mut settings = HashMap::new();

        // Default values (borrowed, no allocation)
        settings.insert("host".to_string(), Cow::Borrowed("localhost"));
        settings.insert("port".to_string(), Cow::Borrowed("8080"));
        settings.insert("timeout".to_string(), Cow::Borrowed("30"));

        Config { settings }
    }

    fn set(&mut self, key: String, value: String) {
        // Override with owned value
        self.settings.insert(key, Cow::Owned(value));
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|cow| cow.as_ref())
    }

    fn print_stats(&self) {
        let borrowed = self.settings.values()
            .filter(|v| matches!(v, Cow::Borrowed(_)))
            .count();
        let owned = self.settings.values()
            .filter(|v| matches!(v, Cow::Owned(_)))
            .count();

        println!("Config stats: {} borrowed, {} owned", borrowed, owned);
    }
}

fn main() {
    let mut config = Config::new();

    println!("Default config:");
    println!("  host: {}", config.get("host").unwrap());
    println!("  port: {}", config.get("port").unwrap());
    config.print_stats();

    // Override some values
    config.set("host".to_string(), "0.0.0.0".to_string());
    config.set("port".to_string(), "3000".to_string());

    println!("\nAfter overrides:");
    println!("  host: {}", config.get("host").unwrap());
    println!("  port: {}", config.get("port").unwrap());
    config.print_stats();
}
```

---

## Combining Smart Pointers

### Recipe 24: Shared Mutable Graph

**Problem**: Create a mutable graph with shared ownership.

```rust
use std::rc::Rc;
use std::cell::RefCell;

type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug)]
struct Node {
    id: usize,
    value: String,
    neighbors: Vec<NodeRef>,
}

impl Node {
    fn new(id: usize, value: String) -> NodeRef {
        Rc::new(RefCell::new(Node {
            id,
            value,
            neighbors: vec![],
        }))
    }

    fn add_edge(from: &NodeRef, to: &NodeRef) {
        from.borrow_mut().neighbors.push(Rc::clone(to));
    }

    fn print_neighbors(node: &NodeRef) {
        let node = node.borrow();
        print!("Node {}: neighbors = [", node.id);
        for (i, neighbor) in node.neighbors.iter().enumerate() {
            if i > 0 { print!(", "); }
            print!("{}", neighbor.borrow().id);
        }
        println!("]");
    }
}

fn main() {
    let node1 = Node::new(1, "A".to_string());
    let node2 = Node::new(2, "B".to_string());
    let node3 = Node::new(3, "C".to_string());

    // Build graph
    Node::add_edge(&node1, &node2);
    Node::add_edge(&node1, &node3);
    Node::add_edge(&node2, &node3);

    // Print structure
    Node::print_neighbors(&node1);
    Node::print_neighbors(&node2);
    Node::print_neighbors(&node3);

    // Modify a node
    node2.borrow_mut().value = "B-modified".to_string();

    println!("\nNode 2 value: {}", node2.borrow().value);
}
```

---

### Recipe 25: Thread-Safe Doubly Linked List

**Problem**: Implement a concurrent doubly-linked list.

```rust
use std::sync::{Arc, Mutex, Weak};

type NodeRef<T> = Arc<Mutex<Node<T>>>;
type WeakNodeRef<T> = Weak<Mutex<Node<T>>>;

struct Node<T> {
    value: T,
    next: Option<NodeRef<T>>,
    prev: WeakNodeRef<T>,
}

struct DoublyLinkedList<T> {
    head: Option<NodeRef<T>>,
    tail: WeakNodeRef<T>,
}

impl<T> DoublyLinkedList<T> {
    fn new() -> Self {
        DoublyLinkedList {
            head: None,
            tail: Weak::new(),
        }
    }

    fn push_back(&mut self, value: T) {
        let new_node = Arc::new(Mutex::new(Node {
            value,
            next: None,
            prev: Weak::new(),
        }));

        if let Some(old_tail) = self.tail.upgrade() {
            new_node.lock().unwrap().prev = Arc::downgrade(&old_tail);
            old_tail.lock().unwrap().next = Some(Arc::clone(&new_node));
            self.tail = Arc::downgrade(&new_node);
        } else {
            // First element
            self.head = Some(Arc::clone(&new_node));
            self.tail = Arc::downgrade(&new_node);
        }
    }

    fn print_forward(&self) where T: std::fmt::Display {
        let mut current = self.head.clone();

        print!("Forward: ");
        while let Some(node) = current {
            let node = node.lock().unwrap();
            print!("{} -> ", node.value);
            current = node.next.clone();
        }
        println!("None");
    }
}

fn main() {
    let mut list = DoublyLinkedList::new();

    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    list.print_forward();
}
```

---

## Design Patterns

### Recipe 26: Builder Pattern with Rc

**Problem**: Build complex objects with shared components.

```rust
use std::rc::Rc;

#[derive(Clone)]
struct Engine {
    horsepower: u32,
    fuel_type: String,
}

#[derive(Clone)]
struct Transmission {
    gears: u8,
    auto: bool,
}

struct Car {
    engine: Rc<Engine>,
    transmission: Rc<Transmission>,
    color: String,
    model: String,
}

struct CarBuilder {
    engine: Option<Rc<Engine>>,
    transmission: Option<Rc<Transmission>>,
    color: Option<String>,
    model: Option<String>,
}

impl CarBuilder {
    fn new() -> Self {
        CarBuilder {
            engine: None,
            transmission: None,
            color: None,
            model: None,
        }
    }

    fn engine(mut self, engine: Rc<Engine>) -> Self {
        self.engine = Some(engine);
        self
    }

    fn transmission(mut self, transmission: Rc<Transmission>) -> Self {
        self.transmission = Some(transmission);
        self
    }

    fn color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    fn model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    fn build(self) -> Result<Car, &'static str> {
        Ok(Car {
            engine: self.engine.ok_or("Engine required")?,
            transmission: self.transmission.ok_or("Transmission required")?,
            color: self.color.ok_or("Color required")?,
            model: self.model.ok_or("Model required")?,
        })
    }
}

fn main() {
    // Shared components
    let v8_engine = Rc::new(Engine {
        horsepower: 450,
        fuel_type: "Gasoline".to_string(),
    });

    let auto_trans = Rc::new(Transmission {
        gears: 8,
        auto: true,
    });

    // Build multiple cars sharing same components
    let car1 = CarBuilder::new()
        .engine(Rc::clone(&v8_engine))
        .transmission(Rc::clone(&auto_trans))
        .color("Red".to_string())
        .model("Sedan".to_string())
        .build()
        .unwrap();

    let car2 = CarBuilder::new()
        .engine(Rc::clone(&v8_engine))
        .transmission(Rc::clone(&auto_trans))
        .color("Blue".to_string())
        .model("Coupe".to_string())
        .build()
        .unwrap();

    println!("Engine shared: {}", Rc::ptr_eq(&car1.engine, &car2.engine));
    println!("Engine ref count: {}", Rc::strong_count(&v8_engine));
}
```

---

### Recipe 27: Flyweight Pattern

**Problem**: Share common data to reduce memory usage.

```rust
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct FontData {
    family: String,
    size: u32,
    weight: String,
}

struct Character {
    char: char,
    font: Rc<FontData>,
    position: (i32, i32),
}

struct FontFactory {
    fonts: HashMap<FontData, Rc<FontData>>,
}

impl FontFactory {
    fn new() -> Self {
        FontFactory {
            fonts: HashMap::new(),
        }
    }

    fn get_font(&mut self, family: String, size: u32, weight: String) -> Rc<FontData> {
        let key = FontData { family, size, weight };

        self.fonts.entry(key.clone())
            .or_insert_with(|| {
                println!("Creating new font: {:?}", key);
                Rc::new(key)
            })
            .clone()
    }

    fn font_count(&self) -> usize {
        self.fonts.len()
    }
}

fn main() {
    let mut factory = FontFactory::new();
    let mut characters = vec![];

    // Create many characters with few unique fonts
    let arial_12 = factory.get_font("Arial".to_string(), 12, "normal".to_string());
    let times_14 = factory.get_font("Times".to_string(), 14, "bold".to_string());

    for i in 0..100 {
        let font = if i % 2 == 0 {
            Rc::clone(&arial_12)
        } else {
            Rc::clone(&times_14)
        };

        characters.push(Character {
            char: (b'A' + (i % 26) as u8) as char,
            font,
            position: (i * 10, 100),
        });
    }

    println!("Created {} characters", characters.len());
    println!("Using {} unique fonts", factory.font_count());
    println!("Arial ref count: {}", Rc::strong_count(&arial_12));
}
```

---

## Performance Considerations

### Smart Pointer Overhead

```rust
use std::mem::size_of;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;

fn main() {
    println!("Size comparisons:");
    println!("i32: {} bytes", size_of::<i32>());
    println!("&i32: {} bytes", size_of::<&i32>());
    println!("Box<i32>: {} bytes", size_of::<Box<i32>>());
    println!("Rc<i32>: {} bytes", size_of::<Rc<i32>>());
    println!("Arc<i32>: {} bytes", size_of::<Arc<i32>>());
    println!("RefCell<i32>: {} bytes", size_of::<RefCell<i32>>());

    println!("\nMemory layout:");
    println!("Raw pointer contains: address only");
    println!("Box contains: pointer to heap");
    println!("Rc contains: pointer to (data + strong count + weak count)");
    println!("Arc contains: pointer to (data + atomic strong + atomic weak)");
}
```

---

### Choosing the Right Smart Pointer

```rust
// ✅ Use Box when:
// - You need heap allocation
// - Single ownership
// - Recursive types
let _boxed: Box<i32> = Box::new(42);

// ✅ Use Rc when:
// - Multiple ownership (single thread)
// - Immutable shared data
use std::rc::Rc;
let _shared: Rc<i32> = Rc::new(42);

// ✅ Use Arc when:
// - Multiple ownership across threads
// - Immutable shared data (concurrent)
use std::sync::Arc;
let _thread_safe: Arc<i32> = Arc::new(42);

// ✅ Use RefCell when:
// - Interior mutability needed (single thread)
// - Runtime borrow checking is acceptable
use std::cell::RefCell;
let _mutable: RefCell<i32> = RefCell::new(42);

// ✅ Use Mutex when:
// - Interior mutability across threads
// - Exclusive access needed
use std::sync::Mutex;
let _locked: Mutex<i32> = Mutex::new(42);

// ✅ Use RwLock when:
// - Many readers, few writers
// - Performance critical reads
use std::sync::RwLock;
let _rw: RwLock<i32> = RwLock::new(42);
```

---

## Common Pitfalls

### Pitfall 1: Circular References with Rc

```rust
use std::rc::Rc;
use std::cell::RefCell;

// ❌ Memory leak - circular reference
fn create_cycle() {
    #[derive(Debug)]
    struct Node {
        next: RefCell<Option<Rc<Node>>>,
    }

    let a = Rc::new(Node { next: RefCell::new(None) });
    let b = Rc::new(Node { next: RefCell::new(None) });

    *a.next.borrow_mut() = Some(Rc::clone(&b));
    *b.next.borrow_mut() = Some(Rc::clone(&a)); // Cycle!

    println!("Reference counts: a={}, b={}",
        Rc::strong_count(&a), Rc::strong_count(&b));
    // Both have count 2, will never be freed
}

// ✅ Solution: Use Weak references
fn avoid_cycle() {
    use std::rc::Weak;

    #[derive(Debug)]
    struct Node {
        next: RefCell<Option<Rc<Node>>>,
        prev: RefCell<Weak<Node>>,
    }

    let a = Rc::new(Node {
        next: RefCell::new(None),
        prev: RefCell::new(Weak::new()),
    });

    let b = Rc::new(Node {
        next: RefCell::new(None),
        prev: RefCell::new(Weak::new()),
    });

    *a.next.borrow_mut() = Some(Rc::clone(&b));
    *b.prev.borrow_mut() = Rc::downgrade(&a); // Weak - no cycle

    println!("No memory leak!");
}

fn main() {
    create_cycle();
    avoid_cycle();
}
```

---

### Pitfall 2: RefCell Borrow Panics

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(vec![1, 2, 3]);

    // ❌ This will panic - can't have mutable and immutable borrows
    // let r1 = data.borrow();
    // let r2 = data.borrow_mut(); // PANIC!

    // ✅ Solution: Ensure borrows don't overlap
    {
        let r1 = data.borrow();
        println!("{:?}", r1);
    } // r1 dropped here

    {
        let mut r2 = data.borrow_mut();
        r2.push(4);
    } // r2 dropped here

    println!("{:?}", data.borrow());
}
```

---

### Pitfall 3: Mutex Poisoning

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);

    let handle = thread::spawn(move || {
        let mut d = data_clone.lock().unwrap();
        d.push(4);
        panic!("Thread panicked while holding lock!");
    });

    let _ = handle.join(); // Thread panics

    // ❌ Mutex is now poisoned
    match data.lock() {
        Ok(_) => println!("Got lock"),
        Err(poisoned) => {
            println!("Mutex was poisoned!");
            // ✅ Can still recover the data
            let _recovered = poisoned.into_inner();
        }
    }
}
```

---

### Pitfall 4: Deadlocks with Multiple Locks

```rust
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let lock1 = Arc::new(Mutex::new(1));
    let lock2 = Arc::new(Mutex::new(2));

    let lock1_clone = Arc::clone(&lock1);
    let lock2_clone = Arc::clone(&lock2);

    // Thread 1: lock1 then lock2
    let t1 = thread::spawn(move || {
        let _l1 = lock1_clone.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _l2 = lock2_clone.lock().unwrap(); // Deadlock!
        println!("Thread 1");
    });

    // Thread 2: lock2 then lock1
    let t2 = thread::spawn(move || {
        let _l2 = lock2.lock().unwrap();
        thread::sleep(Duration::from_millis(10));
        let _l1 = lock1.lock().unwrap(); // Deadlock!
        println!("Thread 2");
    });

    // ✅ Solution: Always acquire locks in same order
    // Or use try_lock with timeout

    // This will hang forever
    // t1.join().unwrap();
    // t2.join().unwrap();

    println!("Demonstrating deadlock (won't finish)");
}
```

---

## Quick Reference

### Smart Pointer Decision Tree

```
Need heap allocation?
├─ Single owner → Box<T>
└─ Multiple owners
   ├─ Single thread?
   │  ├─ Immutable → Rc<T>
   │  └─ Mutable → Rc<RefCell<T>>
   └─ Multiple threads?
      ├─ Immutable → Arc<T>
      └─ Mutable
         ├─ Exclusive access → Arc<Mutex<T>>
         └─ Read-heavy → Arc<RwLock<T>>

Need interior mutability?
├─ Single thread?
│  ├─ Copy types → Cell<T>
│  └─ Non-Copy → RefCell<T>
└─ Multiple threads?
   ├─ Exclusive → Mutex<T>
   └─ Shared reads → RwLock<T>

Prevent cycles?
└─ Use Weak<T>

Avoid clones?
└─ Use Cow<T>
```

---

### Common Combinations

| Pattern | Use Case | Example |
|---------|----------|---------|
| `Rc<RefCell<T>>` | Shared mutable data (single thread) | Graphs, trees with parent refs |
| `Arc<Mutex<T>>` | Shared mutable data (multi-thread) | Concurrent cache, counters |
| `Arc<RwLock<T>>` | Many readers, few writers | Config, read-heavy cache |
| `Rc<T>` with `Weak<T>` | Avoid cycles | Parent-child trees |
| `Box<dyn Trait>` | Trait objects | Polymorphic collections |
| `Cow<'a, T>` | Conditional cloning | String processing, config |

---

## Summary

### Key Takeaways

* ✅ **Box<T>**: Simplest smart pointer - heap allocation, single ownership
* ✅ **Rc<T> / Arc<T>**: Reference counting for shared ownership
* ✅ **RefCell<T> / Mutex<T>**: Interior mutability patterns
* ✅ **Weak<T>**: Break cycles, non-owning references
* ✅ **Cow<T>**: Optimize by avoiding unnecessary clones

### Best Practices

1. **Start simple**: Use `Box` unless you need sharing
2. **Avoid cycles**: Use `Weak` for parent references
3. **Choose by thread-safety**: `Rc` vs `Arc`, `RefCell` vs `Mutex`
4. **Minimize lock scope**: Lock for shortest time possible
5. **Consistent lock order**: Prevent deadlocks
6. **Profile before optimizing**: Smart pointers have costs

### When to Use What

- **Heap allocation**: `Box<T>`
- **Shared ownership (ST)**: `Rc<T>`
- **Shared ownership (MT)**: `Arc<T>`
- **Mutable sharing (ST)**: `Rc<RefCell<T>>`
- **Mutable sharing (MT)**: `Arc<Mutex<T>>` or `Arc<RwLock<T>>`
- **Break cycles**: `Weak<T>`
- **Conditional cloning**: `Cow<T>`
- **Simple flags**: `Cell<T>`

Master smart pointers to write safe, efficient Rust code with complex ownership patterns!
