# Smart Pointer Patterns

This chapter explores smart pointer patterns in Rust, covering heap allocation with Box, reference counting with Rc/Arc, preventing memory leaks with Weak references, implementing custom smart pointers, intrusive data structures, and optimization techniques. We'll cover practical, production-ready examples for managing complex ownership scenarios.

## Table of Contents

1. [Box, Rc, Arc Usage Patterns](#box-rc-arc-usage-patterns)
2. [Weak References and Cycles](#weak-references-and-cycles)
3. [Custom Smart Pointers](#custom-smart-pointers)
4. [Intrusive Data Structures](#intrusive-data-structures)
5. [Reference Counting Optimization](#reference-counting-optimization)

---

## Box, Rc, Arc Usage Patterns

Smart pointers enable different ownership models beyond Rust's default move semantics.

### Recipe 1: Box for Heap Allocation

**Problem**: Store data on the heap for recursive types, large data, or trait objects.

**Solution**:

```rust
use std::mem;

//=======================================
// Pattern 1: Recursive types require Box
//=======================================
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn new() -> Self {
        List::Nil
    }

    fn prepend(self, elem: T) -> Self {
        List::Cons(elem, Box::new(self))
    }

    fn len(&self) -> usize {
        match self {
            List::Cons(_, tail) => 1 + tail.len(),
            List::Nil => 0,
        }
    }

    fn iter(&self) -> ListIter<T> {
        ListIter { current: self }
    }
}

struct ListIter<'a, T> {
    current: &'a List<T>,
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            List::Cons(value, tail) => {
                self.current = tail;
                Some(value)
            }
            List::Nil => None,
        }
    }
}

//=================================
// Pattern 2: Large structs on heap
//=================================
struct LargeData {
    buffer: [u8; 1024 * 1024], // 1MB
}

fn stack_overflow_risk() -> LargeData {
    //===============================
    // This could overflow the stack!
    //===============================
    LargeData {
        buffer: [0; 1024 * 1024],
    }
}

fn heap_allocation() -> Box<LargeData> {
    //========================
    // Safe: allocated on heap
    //========================
    Box::new(LargeData {
        buffer: [0; 1024 * 1024],
    })
}

//=====================================
// Pattern 3: Trait objects require Box
//=====================================
trait Drawable {
    fn draw(&self);
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

fn trait_objects() {
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle {
            width: 10.0,
            height: 20.0,
        }),
    ];

    for shape in shapes {
        shape.draw();
    }
}

//========================
// Real-world: Binary tree
//========================
#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

impl<T: Ord> TreeNode<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }

    fn insert(&mut self, value: T) {
        if value < self.value {
            match &mut self.left {
                Some(node) => node.insert(value),
                None => self.left = Some(Box::new(TreeNode::new(value))),
            }
        } else {
            match &mut self.right {
                Some(node) => node.insert(value),
                None => self.right = Some(Box::new(TreeNode::new(value))),
            }
        }
    }

    fn contains(&self, value: &T) -> bool {
        if value == &self.value {
            true
        } else if value < &self.value {
            self.left.as_ref().map_or(false, |node| node.contains(value))
        } else {
            self.right.as_ref().map_or(false, |node| node.contains(value))
        }
    }

    fn inorder_iter(&self) -> Vec<&T> {
        let mut result = Vec::new();

        if let Some(left) = &self.left {
            result.extend(left.inorder_iter());
        }

        result.push(&self.value);

        if let Some(right) = &self.right {
            result.extend(right.inorder_iter());
        }

        result
    }
}

//======================================
// Pattern 4: Box for ownership transfer
//======================================
fn process_large_data(data: Box<LargeData>) {
    //================================
    // Takes ownership without copying
    //================================
    println!("Processing {} bytes", data.buffer.len());
}

fn main() {
    println!("=== Linked List ===\n");

    let list = List::new()
        .prepend(3)
        .prepend(2)
        .prepend(1);

    println!("List length: {}", list.len());
    println!("List items: {:?}", list.iter().collect::<Vec<_>>());

    println!("\n=== Stack vs Heap ===\n");

    println!("LargeData size: {} bytes", mem::size_of::<LargeData>());
    println!("Box<LargeData> size: {} bytes", mem::size_of::<Box<LargeData>>());

    let heap_data = heap_allocation();
    process_large_data(heap_data);

    println!("\n=== Trait Objects ===\n");
    trait_objects();

    println!("\n=== Binary Tree ===\n");

    let mut tree = TreeNode::new(5);
    for value in [3, 7, 1, 4, 6, 9] {
        tree.insert(value);
    }

    println!("Tree contains 4: {}", tree.contains(&4));
    println!("Tree contains 8: {}", tree.contains(&8));
    println!("Inorder: {:?}", tree.inorder_iter());
}
```

**Box Use Cases**:
- **Recursive types**: Lists, trees, graphs
- **Large data**: Avoid stack overflow
- **Trait objects**: Dynamic dispatch
- **Ownership transfer**: Move without copying

---

### Recipe 2: Rc for Shared Ownership

**Problem**: Multiple owners need read-only access to the same data.

**Solution**:

```rust
use std::rc::Rc;

//================================
// Pattern 1: Shared configuration
//================================
struct Config {
    database_url: String,
    max_connections: usize,
    timeout_ms: u64,
}

struct DatabasePool {
    config: Rc<Config>,
}

struct CacheService {
    config: Rc<Config>,
}

struct ApiServer {
    config: Rc<Config>,
}

impl DatabasePool {
    fn new(config: Rc<Config>) -> Self {
        println!("DB Pool using: {}", config.database_url);
        Self { config }
    }
}

impl CacheService {
    fn new(config: Rc<Config>) -> Self {
        println!("Cache using timeout: {}ms", config.timeout_ms);
        Self { config }
    }
}

impl ApiServer {
    fn new(config: Rc<Config>) -> Self {
        println!("API Server max connections: {}", config.max_connections);
        Self { config }
    }
}

fn shared_config() {
    let config = Rc::new(Config {
        database_url: "postgresql://localhost/mydb".to_string(),
        max_connections: 100,
        timeout_ms: 5000,
    });

    println!("Initial ref count: {}", Rc::strong_count(&config));

    let db_pool = DatabasePool::new(Rc::clone(&config));
    println!("After DB pool: {}", Rc::strong_count(&config));

    let cache = CacheService::new(Rc::clone(&config));
    println!("After cache: {}", Rc::strong_count(&config));

    let api = ApiServer::new(Rc::clone(&config));
    println!("After API: {}", Rc::strong_count(&config));

    //========================================================
    // config, db_pool, cache, api all dropped at end of scope
    //========================================================
    // Reference count goes to 0, memory freed
}

//================================
// Pattern 2: Shared data in graph
//================================
#[derive(Debug)]
struct Node {
    id: usize,
    value: String,
}

struct Graph {
    nodes: Vec<Rc<Node>>,
    edges: Vec<(Rc<Node>, Rc<Node>)>,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn add_node(&mut self, id: usize, value: String) -> Rc<Node> {
        let node = Rc::new(Node { id, value });
        self.nodes.push(Rc::clone(&node));
        node
    }

    fn add_edge(&mut self, from: Rc<Node>, to: Rc<Node>) {
        self.edges.push((from, to));
    }

    fn print_edges(&self) {
        for (from, to) in &self.edges {
            println!("{} -> {}", from.value, to.value);
        }
    }
}

//===================================
// Real-world: Immutable data sharing
//===================================
#[derive(Debug, Clone)]
struct Document {
    content: String,
    metadata: String,
}

struct DocumentVersion {
    version: usize,
    doc: Rc<Document>,
}

struct VersionControl {
    versions: Vec<DocumentVersion>,
}

impl VersionControl {
    fn new(initial_content: String) -> Self {
        let doc = Rc::new(Document {
            content: initial_content,
            metadata: "v1".to_string(),
        });

        Self {
            versions: vec![DocumentVersion { version: 1, doc }],
        }
    }

    fn add_version(&mut self, content: String) {
        let version = self.versions.len() + 1;
        let doc = Rc::new(Document {
            content,
            metadata: format!("v{}", version),
        });

        self.versions.push(DocumentVersion { version, doc });
    }

    fn get_version(&self, version: usize) -> Option<Rc<Document>> {
        self.versions
            .get(version - 1)
            .map(|v| Rc::clone(&v.doc))
    }

    fn compare_versions(&self, v1: usize, v2: usize) {
        if let (Some(doc1), Some(doc2)) = (self.get_version(v1), self.get_version(v2)) {
            println!("Version {}: {}", v1, doc1.content);
            println!("Version {}: {}", v2, doc2.content);
        }
    }
}

//=======================================
// Pattern 3: Rc with interior mutability
//=======================================
use std::cell::RefCell;

struct Sensor {
    id: usize,
    reading: RefCell<f64>,
}

struct SensorNetwork {
    sensors: Vec<Rc<Sensor>>,
}

impl SensorNetwork {
    fn new() -> Self {
        Self {
            sensors: Vec::new(),
        }
    }

    fn add_sensor(&mut self, id: usize) -> Rc<Sensor> {
        let sensor = Rc::new(Sensor {
            id,
            reading: RefCell::new(0.0),
        });
        self.sensors.push(Rc::clone(&sensor));
        sensor
    }

    fn update_readings(&self) {
        for sensor in &self.sensors {
            *sensor.reading.borrow_mut() = rand::random::<f64>() * 100.0;
        }
    }

    fn average_reading(&self) -> f64 {
        let sum: f64 = self.sensors.iter().map(|s| *s.reading.borrow()).sum();
        sum / self.sensors.len() as f64
    }
}

fn main() {
    println!("=== Shared Config ===\n");
    shared_config();

    println!("\n=== Graph with Shared Nodes ===\n");

    let mut graph = Graph::new();

    let a = graph.add_node(1, "A".to_string());
    let b = graph.add_node(2, "B".to_string());
    let c = graph.add_node(3, "C".to_string());

    graph.add_edge(Rc::clone(&a), Rc::clone(&b));
    graph.add_edge(Rc::clone(&b), Rc::clone(&c));
    graph.add_edge(Rc::clone(&a), Rc::clone(&c));

    println!("Edges:");
    graph.print_edges();

    println!("\nNode A ref count: {}", Rc::strong_count(&a));

    println!("\n=== Version Control ===\n");

    let mut vc = VersionControl::new("Initial content".to_string());
    vc.add_version("Updated content".to_string());
    vc.add_version("Final content".to_string());

    vc.compare_versions(1, 3);

    println!("\n=== Sensor Network ===\n");

    let mut network = SensorNetwork::new();
    let sensor1 = network.add_sensor(1);
    let sensor2 = network.add_sensor(2);

    network.update_readings();
    println!("Sensor 1: {:.2}", sensor1.reading.borrow());
    println!("Sensor 2: {:.2}", sensor2.reading.borrow());
    println!("Average: {:.2}", network.average_reading());
}
```

**Rc Characteristics**:
- **Single-threaded**: Not thread-safe
- **Shared ownership**: Multiple owners, last one frees
- **Reference counting**: Overhead of counter updates
- **Interior mutability**: Combine with RefCell for mutation

---

### Recipe 3: Arc for Thread-Safe Sharing

**Problem**: Share data across threads safely.

**Solution**:

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

//================================================
// Pattern 1: Shared read-only data across threads
//================================================
fn arc_readonly() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);

    let mut handles = vec![];

    for i in 0..5 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            println!("Thread {}: sum = {}", i, data.iter().sum::<i32>());
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

//===================================================
// Pattern 2: Arc with Mutex for shared mutable state
//===================================================
struct SharedCounter {
    count: Arc<Mutex<usize>>,
}

impl SharedCounter {
    fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }

    fn get(&self) -> usize {
        *self.count.lock().unwrap()
    }

    fn clone_handle(&self) -> Self {
        Self {
            count: Arc::clone(&self.count),
        }
    }
}

fn arc_mutex_example() {
    let counter = SharedCounter::new();
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = counter.clone_handle();
        handles.push(thread::spawn(move || {
            for _ in 0..1000 {
                counter.increment();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.get());
}

//====================================================
// Pattern 3: Arc with RwLock for read-heavy workloads
//====================================================
struct Cache {
    data: Arc<RwLock<std::collections::HashMap<String, String>>>,
}

impl Cache {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.data.write().unwrap().insert(key, value);
    }

    fn clone_handle(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

fn arc_rwlock_example() {
    let cache = Cache::new();

    //==============
    // Writer thread
    //==============
    let writer_cache = cache.clone_handle();
    let writer = thread::spawn(move || {
        for i in 0..100 {
            writer_cache.set(format!("key_{}", i), format!("value_{}", i));
            thread::sleep(Duration::from_millis(10));
        }
    });

    //===============
    // Reader threads
    //===============
    let mut readers = vec![];
    for id in 0..5 {
        let reader_cache = cache.clone_handle();
        readers.push(thread::spawn(move || {
            for i in 0..50 {
                if let Some(value) = reader_cache.get(&format!("key_{}", i * 2)) {
                    if id == 0 && i % 10 == 0 {
                        println!("Reader {}: {}", id, value);
                    }
                }
                thread::sleep(Duration::from_millis(5));
            }
        }));
    }

    writer.join().unwrap();
    for reader in readers {
        reader.join().unwrap();
    }
}

//===============================================
// Real-world: Thread pool with shared work queue
//===============================================
use std::sync::mpsc;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {} executing job", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} shutting down", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(f)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

fn main() {
    println!("=== Arc Read-Only ===\n");
    arc_readonly();

    println!("\n=== Arc + Mutex ===\n");
    arc_mutex_example();

    println!("\n=== Arc + RwLock ===\n");
    arc_rwlock_example();

    println!("\n=== Thread Pool ===\n");

    let pool = ThreadPool::new(4);

    for i in 0..10 {
        pool.execute(move || {
            println!("Job {} executing", i);
            thread::sleep(Duration::from_millis(100));
        });
    }

    thread::sleep(Duration::from_secs(2));
}
```

**Arc vs Rc**:
- **Arc**: Atomic reference counting (thread-safe)
- **Rc**: Non-atomic (single-threaded only)
- **Performance**: Rc is faster (no atomic operations)
- **Use case**: Arc for multi-threaded, Rc for single-threaded

---

## Weak References and Cycles

Weak references prevent memory leaks from reference cycles.

### Recipe 4: Breaking Reference Cycles

**Problem**: Prevent memory leaks when data structures have circular references.

**Solution**:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

//=========================================================
// Problem: This creates a reference cycle and leaks memory
//=========================================================
#[derive(Debug)]
struct NodeWithCycle {
    value: i32,
    next: Option<Rc<RefCell<NodeWithCycle>>>,
    prev: Option<Rc<RefCell<NodeWithCycle>>>, // Strong reference - BAD!
}

//=======================================
// Solution: Use Weak for back-references
//=======================================
#[derive(Debug)]
struct Node {
    value: i32,
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Weak<RefCell<Node>>>, // Weak reference - GOOD!
}

impl Node {
    fn new(value: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            value,
            next: None,
            prev: None,
        }))
    }
}

//========================================
// Pattern 1: Doubly-linked list with Weak
//========================================
struct DoublyLinkedList {
    head: Option<Rc<RefCell<Node>>>,
    tail: Option<Rc<RefCell<Node>>>,
}

impl DoublyLinkedList {
    fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    fn push_back(&mut self, value: i32) {
        let new_node = Node::new(value);

        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(Rc::clone(&new_node));
                new_node.borrow_mut().prev = Some(Rc::downgrade(&old_tail));
                self.tail = Some(new_node);
            }
            None => {
                self.head = Some(Rc::clone(&new_node));
                self.tail = Some(new_node);
            }
        }
    }

    fn print_forward(&self) {
        let mut current = self.head.as_ref().map(Rc::clone);

        while let Some(node) = current {
            print!("{} -> ", node.borrow().value);
            current = node.borrow().next.as_ref().map(Rc::clone);
        }
        println!("None");
    }

    fn print_backward(&self) {
        let mut current = self.tail.as_ref().map(Rc::clone);

        while let Some(node) = current {
            print!("{} -> ", node.borrow().value);
            current = node
                .borrow()
                .prev
                .as_ref()
                .and_then(|weak| weak.upgrade());
        }
        println!("None");
    }
}

//=====================================
// Pattern 2: Tree with parent pointers
//=====================================
#[derive(Debug)]
struct TreeNode {
    value: i32,
    parent: Option<Weak<RefCell<TreeNode>>>,
    children: Vec<Rc<RefCell<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(TreeNode {
            value,
            parent: None,
            children: Vec::new(),
        }))
    }

    fn add_child(parent: &Rc<RefCell<TreeNode>>, child_value: i32) -> Rc<RefCell<TreeNode>> {
        let child = TreeNode::new(child_value);
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(Rc::clone(&child));
        child
    }

    fn print_path_to_root(node: &Rc<RefCell<TreeNode>>) {
        let mut path = Vec::new();
        let mut current = Some(Rc::clone(node));

        while let Some(node_rc) = current {
            path.push(node_rc.borrow().value);
            current = node_rc
                .borrow()
                .parent
                .as_ref()
                .and_then(|weak| weak.upgrade());
        }

        println!("Path to root: {:?}", path);
    }
}

//=======================================
// Real-world: Observer pattern with Weak
//=======================================
trait Observer {
    fn notify(&self, message: &str);
}

struct Subject {
    observers: Vec<Weak<dyn Observer>>,
}

impl Subject {
    fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    fn attach(&mut self, observer: Weak<dyn Observer>) {
        self.observers.push(observer);
    }

    fn notify_all(&mut self, message: &str) {
        //===============================================
        // Clean up dead observers and notify living ones
        //===============================================
        self.observers.retain(|weak| {
            if let Some(observer) = weak.upgrade() {
                observer.notify(message);
                true // Keep
            } else {
                false // Remove dead observer
            }
        });
    }
}

struct ConcreteObserver {
    id: usize,
}

impl Observer for ConcreteObserver {
    fn notify(&self, message: &str) {
        println!("Observer {} received: {}", self.id, message);
    }
}

//=======================================
// Real-world: Cache with weak references
//=======================================
struct WeakCache<K, V> {
    cache: std::collections::HashMap<K, Weak<V>>,
}

impl<K, V> WeakCache<K, V>
where
    K: Eq + std::hash::Hash,
{
    fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<Rc<V>> {
        self.cache.get(key).and_then(|weak| weak.upgrade())
    }

    fn insert(&mut self, key: K, value: Rc<V>) {
        self.cache.insert(key, Rc::downgrade(&value));
    }

    fn cleanup(&mut self) {
        self.cache.retain(|_, weak| weak.strong_count() > 0);
    }

    fn len(&self) -> usize {
        self.cache.len()
    }
}

fn main() {
    println!("=== Doubly-Linked List ===\n");

    let mut list = DoublyLinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    print!("Forward: ");
    list.print_forward();
    print!("Backward: ");
    list.print_backward();

    println!("\n=== Tree with Parent Pointers ===\n");

    let root = TreeNode::new(1);
    let child1 = TreeNode::add_child(&root, 2);
    let child2 = TreeNode::add_child(&root, 3);
    let grandchild = TreeNode::add_child(&child1, 4);

    TreeNode::print_path_to_root(&grandchild);
    TreeNode::print_path_to_root(&child2);

    println!("\n=== Observer Pattern ===\n");

    let mut subject = Subject::new();

    let observer1 = Rc::new(ConcreteObserver { id: 1 });
    let observer2 = Rc::new(ConcreteObserver { id: 2 });

    subject.attach(Rc::downgrade(&observer1));
    subject.attach(Rc::downgrade(&observer2));

    subject.notify_all("First message");

    drop(observer1); // Observer 1 goes away

    subject.notify_all("Second message"); // Only observer 2 gets this

    println!("\n=== Weak Cache ===\n");

    let mut cache = WeakCache::new();

    {
        let value = Rc::new("cached data".to_string());
        cache.insert("key1", Rc::clone(&value));
        println!("Cache size: {}", cache.len());

        if let Some(cached) = cache.get(&"key1") {
            println!("Found in cache: {}", cached);
        }

        //===================
        // value dropped here
        //===================
    }

    //==================================
    // Try to get after value is dropped
    //==================================
    if cache.get(&"key1").is_none() {
        println!("Cache entry expired");
    }

    cache.cleanup();
    println!("Cache size after cleanup: {}", cache.len());
}
```

**Weak Reference Patterns**:
- **Parent-child**: Child holds Weak to parent
- **Observers**: Subject holds Weak to observers
- **Cache**: Cache holds Weak to values
- **Breaking cycles**: Use Weak for back-references

---

## Custom Smart Pointers

Custom smart pointers enable domain-specific ownership semantics.

### Recipe 5: Implementing Custom Smart Pointers

**Problem**: Create custom pointer types with specialized behavior.

**Solution**:

```rust
use std::ops::{Deref, DerefMut};
use std::fmt;

//=============================
// Pattern 1: Simple custom Box
//=============================
struct MyBox<T> {
    data: *mut T,
}

impl<T> MyBox<T> {
    fn new(value: T) -> Self {
        let data = Box::into_raw(Box::new(value));
        MyBox { data }
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data }
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.data));
        }
    }
}

//===========================================
// Pattern 2: Logging pointer (tracks access)
//===========================================
struct LoggingPtr<T> {
    data: Box<T>,
    reads: std::cell::Cell<usize>,
    writes: std::cell::Cell<usize>,
}

impl<T> LoggingPtr<T> {
    fn new(value: T) -> Self {
        Self {
            data: Box::new(value),
            reads: std::cell::Cell::new(0),
            writes: std::cell::Cell::new(0),
        }
    }

    fn get_stats(&self) -> (usize, usize) {
        (self.reads.get(), self.writes.get())
    }
}

impl<T> Deref for LoggingPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.reads.set(self.reads.get() + 1);
        &self.data
    }
}

impl<T> DerefMut for LoggingPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.writes.set(self.writes.get() + 1);
        &mut self.data
    }
}

//=======================================
// Pattern 3: Lazy initialization pointer
//=======================================
struct Lazy<T, F>
where
    F: FnOnce() -> T,
{
    value: std::cell::UnsafeCell<Option<T>>,
    init: std::cell::UnsafeCell<Option<F>>,
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    fn new(init: F) -> Self {
        Self {
            value: std::cell::UnsafeCell::new(None),
            init: std::cell::UnsafeCell::new(Some(init)),
        }
    }

    fn get(&self) -> &T {
        unsafe {
            if (*self.value.get()).is_none() {
                let init = (*self.init.get()).take().unwrap();
                *self.value.get() = Some(init());
            }

            (*self.value.get()).as_ref().unwrap()
        }
    }
}

//================================================================
// Real-world: Reference-counted string (like Arc<str> but custom)
//================================================================
struct RcStr {
    data: *mut RcStrInner,
}

struct RcStrInner {
    ref_count: std::sync::atomic::AtomicUsize,
    data: str,
}

impl RcStr {
    fn new(s: &str) -> Self {
        let layout = std::alloc::Layout::from_size_align(
            std::mem::size_of::<std::sync::atomic::AtomicUsize>() + s.len(),
            std::mem::align_of::<std::sync::atomic::AtomicUsize>(),
        )
        .unwrap();

        unsafe {
            let ptr = std::alloc::alloc(layout) as *mut RcStrInner;

            std::ptr::write(
                &mut (*ptr).ref_count,
                std::sync::atomic::AtomicUsize::new(1),
            );

            let data_ptr = (&mut (*ptr).data) as *mut str as *mut u8;
            std::ptr::copy_nonoverlapping(s.as_ptr(), data_ptr, s.len());

            RcStr { data: ptr }
        }
    }

    fn as_str(&self) -> &str {
        unsafe { &(*self.data).data }
    }

    fn ref_count(&self) -> usize {
        unsafe { (*self.data).ref_count.load(std::sync::atomic::Ordering::Acquire) }
    }
}

impl Clone for RcStr {
    fn clone(&self) -> Self {
        unsafe {
            (*self.data)
                .ref_count
                .fetch_add(1, std::sync::atomic::Ordering::Release);
        }
        RcStr { data: self.data }
    }
}

impl Drop for RcStr {
    fn drop(&mut self) {
        unsafe {
            if (*self.data)
                .ref_count
                .fetch_sub(1, std::sync::atomic::Ordering::Release)
                == 1
            {
                let layout = std::alloc::Layout::from_size_align(
                    std::mem::size_of::<std::sync::atomic::AtomicUsize>() + (*self.data).data.len(),
                    std::mem::align_of::<std::sync::atomic::AtomicUsize>(),
                )
                .unwrap();

                std::alloc::dealloc(self.data as *mut u8, layout);
            }
        }
    }
}

impl Deref for RcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Display for RcStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

//=====================================
// Pattern 4: Owned-or-borrowed pointer
//=====================================
enum Cow<'a, T: 'a + ToOwned + ?Sized> {
    Borrowed(&'a T),
    Owned(<T as ToOwned>::Owned),
}

impl<'a, T> Cow<'a, T>
where
    T: ToOwned + ?Sized,
{
    fn to_mut(&mut self) -> &mut <T as ToOwned>::Owned {
        match self {
            Cow::Owned(owned) => owned,
            Cow::Borrowed(borrowed) => {
                *self = Cow::Owned(borrowed.to_owned());
                match self {
                    Cow::Owned(owned) => owned,
                    _ => unreachable!(),
                }
            }
        }
    }
}

fn main() {
    println!("=== Custom MyBox ===\n");

    let mut my_box = MyBox::new(42);
    println!("Value: {}", *my_box);
    *my_box = 100;
    println!("Updated: {}", *my_box);

    println!("\n=== Logging Pointer ===\n");

    let mut logged = LoggingPtr::new(String::from("hello"));

    let _read1 = logged.len();
    let _read2 = logged.chars().count();
    logged.push_str(" world");

    let (reads, writes) = logged.get_stats();
    println!("Reads: {}, Writes: {}", reads, writes);
    println!("Value: {}", *logged);

    println!("\n=== Lazy Initialization ===\n");

    let lazy = Lazy::new(|| {
        println!("Initializing expensive computation...");
        42
    });

    println!("Before access");
    println!("Value: {}", lazy.get());
    println!("Value again: {}", lazy.get()); // No re-initialization

    println!("\n=== Custom RcStr ===\n");

    let s1 = RcStr::new("Hello, world!");
    println!("s1: {}", s1);
    println!("s1 ref count: {}", s1.ref_count());

    let s2 = s1.clone();
    println!("s1 ref count after clone: {}", s1.ref_count());
    println!("s2 ref count: {}", s2.ref_count());

    drop(s2);
    println!("s1 ref count after drop s2: {}", s1.ref_count());
}
```

**Custom Smart Pointer Requirements**:
- **Deref**: Enable `*` and method calls
- **Drop**: Clean up resources
- **Clone** (optional): For reference counting
- **Send/Sync** (optional): For thread safety

---

## Intrusive Data Structures

Intrusive structures embed pointers within nodes, enabling efficient operations without separate allocations.

### Recipe 6: Intrusive Linked Lists

**Problem**: Implement efficient linked lists where nodes are embedded in objects.

**Solution**:

```rust
use std::ptr;
use std::marker::PhantomData;

//========================================
// Pattern 1: Intrusive singly-linked list
//========================================
struct IntrusiveList<T> {
    head: *mut ListNode<T>,
    _phantom: PhantomData<T>,
}

struct ListNode<T> {
    next: *mut ListNode<T>,
    data: T,
}

impl<T> IntrusiveList<T> {
    fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            _phantom: PhantomData,
        }
    }

    fn push_front(&mut self, data: T) {
        let node = Box::into_raw(Box::new(ListNode {
            next: self.head,
            data,
        }));

        self.head = node;
    }

    fn pop_front(&mut self) -> Option<T> {
        if self.head.is_null() {
            return None;
        }

        unsafe {
            let node = Box::from_raw(self.head);
            self.head = node.next;
            Some(node.data)
        }
    }

    fn iter(&self) -> IntrusiveListIter<T> {
        IntrusiveListIter {
            current: self.head,
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for IntrusiveList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

struct IntrusiveListIter<'a, T> {
    current: *mut ListNode<T>,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T> Iterator for IntrusiveListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let data = &(*self.current).data;
                self.current = (*self.current).next;
                Some(data)
            }
        }
    }
}

//=======================================================
// Real-world: Intrusive doubly-linked list for LRU cache
//=======================================================
struct LruCache<K, V> {
    map: std::collections::HashMap<K, *mut LruNode<K, V>>,
    head: *mut LruNode<K, V>,
    tail: *mut LruNode<K, V>,
    capacity: usize,
}

struct LruNode<K, V> {
    key: K,
    value: V,
    prev: *mut LruNode<K, V>,
    next: *mut LruNode<K, V>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
{
    fn new(capacity: usize) -> Self {
        Self {
            map: std::collections::HashMap::new(),
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
            capacity,
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        let node_ptr = *self.map.get(key)?;

        unsafe {
            //==============
            // Move to front
            //==============
            self.detach(node_ptr);
            self.attach_front(node_ptr);

            Some(&(*node_ptr).value)
        }
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&node_ptr) = self.map.get(&key) {
            unsafe {
                (*node_ptr).value = value;
                self.detach(node_ptr);
                self.attach_front(node_ptr);
            }
            return;
        }

        //=====================
        // Evict if at capacity
        //=====================
        if self.map.len() >= self.capacity {
            unsafe {
                if !self.tail.is_null() {
                    let tail_key = (*self.tail).key.clone();
                    self.detach(self.tail);
                    drop(Box::from_raw(self.tail));
                    self.map.remove(&tail_key);
                }
            }
        }

        //================
        // Create new node
        //================
        let node = Box::into_raw(Box::new(LruNode {
            key: key.clone(),
            value,
            prev: ptr::null_mut(),
            next: ptr::null_mut(),
        }));

        self.map.insert(key, node);
        unsafe {
            self.attach_front(node);
        }
    }

    unsafe fn detach(&mut self, node: *mut LruNode<K, V>) {
        let prev = (*node).prev;
        let next = (*node).next;

        if !prev.is_null() {
            (*prev).next = next;
        } else {
            self.head = next;
        }

        if !next.is_null() {
            (*next).prev = prev;
        } else {
            self.tail = prev;
        }

        (*node).prev = ptr::null_mut();
        (*node).next = ptr::null_mut();
    }

    unsafe fn attach_front(&mut self, node: *mut LruNode<K, V>) {
        (*node).next = self.head;
        (*node).prev = ptr::null_mut();

        if !self.head.is_null() {
            (*self.head).prev = node;
        }

        self.head = node;

        if self.tail.is_null() {
            self.tail = node;
        }
    }
}

impl<K, V> Drop for LruCache<K, V> {
    fn drop(&mut self) {
        unsafe {
            let mut current = self.head;
            while !current.is_null() {
                let next = (*current).next;
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}

fn main() {
    println!("=== Intrusive List ===\n");

    let mut list = IntrusiveList::new();

    list.push_front(3);
    list.push_front(2);
    list.push_front(1);

    println!("List items:");
    for item in list.iter() {
        println!("  {}", item);
    }

    while let Some(item) = list.pop_front() {
        println!("Popped: {}", item);
    }

    println!("\n=== LRU Cache ===\n");

    let mut cache = LruCache::new(3);

    cache.put("a", 1);
    cache.put("b", 2);
    cache.put("c", 3);

    println!("Get a: {:?}", cache.get(&"a"));
    println!("Get b: {:?}", cache.get(&"b"));

    cache.put("d", 4); // Evicts c (least recently used)

    println!("Get c (evicted): {:?}", cache.get(&"c"));
    println!("Get d: {:?}", cache.get(&"d"));
}
```

**Intrusive List Benefits**:
- **No extra allocations**: Node is part of data
- **Cache-friendly**: Better locality
- **Constant-time removal**: No search needed
- **Use case**: Kernel data structures, high-performance caches

---

## Reference Counting Optimization

Optimizing reference counting reduces overhead and improves performance.

### Recipe 7: Reference Counting Optimizations

**Problem**: Reduce the overhead of reference counting operations.

**Solution**:

```rust
use std::rc::Rc;
use std::sync::Arc;

//====================================
// Pattern 1: Avoid unnecessary clones
//====================================
fn inefficient_clones(data: &Rc<Vec<i32>>) {
    //===============================
    // Bad: Clone for every operation
    //===============================
    let clone1 = Rc::clone(data);
    println!("Length: {}", clone1.len());

    let clone2 = Rc::clone(data);
    println!("First: {}", clone2[0]);
}

fn efficient_borrows(data: &Rc<Vec<i32>>) {
    //======================
    // Good: Borrow directly
    //======================
    println!("Length: {}", data.len());
    println!("First: {}", data[0]);
}

//=========================================
// Pattern 2: Make owned data when possible
//=========================================
fn make_owned(data: Rc<Vec<i32>>) -> Vec<i32> {
    //======================================
    // Try to unwrap if we're the only owner
    //======================================
    Rc::try_unwrap(data).unwrap_or_else(|rc| (*rc).clone())
}

//============================================
// Pattern 3: Batch reference counting updates
//============================================
fn batch_updates() {
    let data = Rc::new(vec![1, 2, 3, 4, 5]);

    //====================================
    // Bad: Multiple increments/decrements
    //====================================
    {
        let _clone1 = Rc::clone(&data);
        let _clone2 = Rc::clone(&data);
        let _clone3 = Rc::clone(&data);
    }

    //======================================
    // Better: Pass references when possible
    //======================================
    {
        process_data(&data);
        process_data(&data);
        process_data(&data);
    }
}

fn process_data(data: &Rc<Vec<i32>>) {
    println!("Processing {} items", data.len());
}

//======================================
// Pattern 4: Use Cow for clone-on-write
//======================================
use std::borrow::Cow;

fn process_string(s: Cow<str>) -> Cow<str> {
    if s.contains("replace") {
        //==================================
        // Need to modify - convert to owned
        //==================================
        Cow::Owned(s.replace("replace", "replaced"))
    } else {
        //================================
        // No modification - keep borrowed
        //================================
        s
    }
}

//=====================================
// Real-world: String interning with Rc
//=====================================
use std::collections::HashMap;

struct StringInterner {
    map: HashMap<String, Rc<str>>,
}

impl StringInterner {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn intern(&mut self, s: &str) -> Rc<str> {
        if let Some(interned) = self.map.get(s) {
            //=================================
            // Already interned - just clone Rc
            //=================================
            Rc::clone(interned)
        } else {
            //==================================
            // Not interned - create new Rc<str>
            //==================================
            let rc: Rc<str> = Rc::from(s);
            self.map.insert(s.to_string(), Rc::clone(&rc));
            rc
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn memory_saved(&self, total_strings: usize) -> usize {
        //===================================
        // Estimate memory saved by interning
        //===================================
        let unique = self.len();
        let duplicates = total_strings - unique;
        duplicates * std::mem::size_of::<String>()
    }
}

//=========================================
// Pattern 5: Weak upgrades to avoid clones
//=========================================
use std::rc::Weak;

struct Observer {
    subject: Weak<Vec<i32>>,
}

impl Observer {
    fn observe(&self) {
        //=================================================
        // Upgrade temporarily, don't keep strong reference
        //=================================================
        if let Some(subject) = self.subject.upgrade() {
            println!("Observing: {} items", subject.len());
        }
    }
}

//======================================
// Pattern 6: Arc performance comparison
//======================================
fn arc_performance_test() {
    use std::time::Instant;

    let data = Arc::new(vec![0; 1_000_000]);

    //========================
    // Test 1: Many Arc clones
    //========================
    let start = Instant::now();
    let mut clones = Vec::new();
    for _ in 0..1000 {
        clones.push(Arc::clone(&data));
    }
    let clone_time = start.elapsed();

    drop(clones);

    //==================================
    // Test 2: Many borrows (no cloning)
    //==================================
    let start = Instant::now();
    for _ in 0..1000 {
        let _borrow = &data;
    }
    let borrow_time = start.elapsed();

    println!("Arc clone time: {:?}", clone_time);
    println!("Borrow time: {:?}", borrow_time);
    println!("Speedup: {:.2}x", clone_time.as_nanos() as f64 / borrow_time.as_nanos() as f64);
}

//====================================
// Pattern 7: Rc vs owned in hot loops
//====================================
fn rc_vs_owned_benchmark() {
    use std::time::Instant;

    let data = vec![1, 2, 3, 4, 5];

    //======================================
    // With Rc (reference counting overhead)
    //======================================
    let start = Instant::now();
    let rc_data = Rc::new(data.clone());
    for _ in 0..1_000_000 {
        let _clone = Rc::clone(&rc_data);
    }
    let rc_time = start.elapsed();

    //=========================================
    // With owned (no overhead but more memory)
    //=========================================
    let start = Instant::now();
    for _ in 0..1_000_000 {
        let _clone = data.clone();
    }
    let owned_time = start.elapsed();

    println!("Rc clones: {:?}", rc_time);
    println!("Owned clones: {:?}", owned_time);
}

fn main() {
    println!("=== Efficient vs Inefficient ===\n");

    let data = Rc::new(vec![1, 2, 3, 4, 5]);
    println!("Ref count: {}", Rc::strong_count(&data));

    efficient_borrows(&data);
    println!("After borrows: {}", Rc::strong_count(&data));

    println!("\n=== Make Owned ===\n");

    let data = Rc::new(vec![1, 2, 3]);
    println!("Initial ref count: {}", Rc::strong_count(&data));

    let owned = make_owned(data);
    println!("Owned data: {:?}", owned);

    println!("\n=== String Interning ===\n");

    let mut interner = StringInterner::new();

    let strings = vec!["hello", "world", "hello", "rust", "world", "hello"];
    let mut interned = Vec::new();

    for s in &strings {
        interned.push(interner.intern(s));
    }

    println!("Total strings: {}", strings.len());
    println!("Unique strings: {}", interner.len());
    println!("Memory saved: ~{} bytes", interner.memory_saved(strings.len()));

    println!("\n=== Arc Performance ===\n");
    arc_performance_test();

    println!("\n=== Rc vs Owned ===\n");
    rc_vs_owned_benchmark();
}
```

**Optimization Strategies**:
1. **Borrow instead of clone**: Use `&Rc<T>` instead of `Rc::clone()`
2. **try_unwrap**: Get owned data if only owner
3. **Weak references**: Avoid strong references when possible
4. **String interning**: Share common strings
5. **Cow**: Clone-on-write for conditional modification
6. **Profile**: Measure before optimizing

---

## Summary

This chapter covered smart pointer patterns in Rust:

1. **Box, Rc, Arc**: Heap allocation, single-threaded sharing, thread-safe sharing
2. **Weak References**: Prevent cycles, observer pattern, caches
3. **Custom Smart Pointers**: Deref, Drop, logging, lazy initialization
4. **Intrusive Structures**: Embedded pointers, LRU cache, kernel-style lists
5. **RC Optimization**: Avoid clones, try_unwrap, string interning, Cow

**Key Takeaways**:
- **Box**: Heap allocation, recursive types, trait objects
- **Rc**: Single-threaded shared ownership
- **Arc**: Thread-safe shared ownership (atomic overhead)
- **Weak**: Break cycles, prevent memory leaks
- **Custom pointers**: Deref + Drop for domain logic
- **Intrusive**: Embed pointers for efficiency

**Smart Pointer Selection Guide**:

| Pattern | Use Case | Thread-Safe | Overhead |
|---------|----------|-------------|----------|
| Box | Heap allocation, recursion | No | Minimal |
| Rc | Shared ownership (single-thread) | No | Reference counting |
| Arc | Shared ownership (multi-thread) | Yes | Atomic RC |
| Weak | Break cycles, observers | Depends | Weak counter |
| RefCell + Rc | Interior mutability | No | Runtime checks |
| Mutex + Arc | Shared mutable state | Yes | Lock overhead |

**Common Patterns**:
- **Rc<RefCell<T>>**: Single-threaded shared mutation
- **Arc<Mutex<T>>**: Multi-threaded shared mutation
- **Arc<RwLock<T>>**: Read-heavy multi-threaded
- **Weak<T>**: Observer, cache, parent pointers

**Performance Tips**:
- Borrow `&Rc<T>` instead of cloning
- Use `try_unwrap` to get owned data
- Intern strings to reduce duplicates
- Profile before optimizing RC
- Consider `Cow` for conditional cloning

**Memory Leak Prevention**:
- Use Weak for back-references
- Break cycles in Drop
- Use Weak in observer pattern
- Profile with valgrind/leak sanitizer

**Safety**:
- Smart pointers are safe (type-checked)
- Custom pointers need unsafe (be careful!)
- Rc/Arc prevent use-after-free
- Weak prevents dangling pointers
