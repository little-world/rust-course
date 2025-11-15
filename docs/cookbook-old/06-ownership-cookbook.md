# Cookbook: Ownership, Borrowing, and Lifetimes

> Practical recipes demonstrating Rust's memory management through classic algorithms and design patterns

## Table of Contents

1. [Data Structures with Ownership](#data-structures-with-ownership)
2. [Sorting Algorithms](#sorting-algorithms)
3. [Search and Tree Algorithms](#search-and-tree-algorithms)
4. [Graph Algorithms](#graph-algorithms)
5. [Design Patterns](#design-patterns)
6. [Iterator Patterns](#iterator-patterns)
7. [Smart Pointer Patterns](#smart-pointer-patterns)
8. [Concurrency Patterns](#concurrency-patterns)
9. [Memory Management Patterns](#memory-management-patterns)
10. [Quick Reference](#quick-reference)

---

## Data Structures with Ownership

### Recipe 1: Stack (LIFO) - Ownership Transfer

**Problem**: Implement a stack where push takes ownership and pop returns ownership.

**Use Case**: Expression evaluation, undo/redo systems, browser history.

**Pattern**: LIFO (Last In, First Out)

```rust
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }

    // Takes ownership of item
    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    // Returns ownership to caller
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    // Borrows to peek without taking ownership
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
}

fn main() {
    let mut stack = Stack::new();

    let s1 = String::from("hello");
    stack.push(s1);  // s1 moved into stack
    // println!("{}", s1);  // Error: s1 no longer valid

    let s2 = String::from("world");
    stack.push(s2);

    // Peek borrows, doesn't take ownership
    if let Some(top) = stack.peek() {
        println!("Top: {}", top);
    }

    // Pop returns ownership
    if let Some(item) = stack.pop() {
        println!("Popped: {}", item);
    }  // item dropped here
}
```

**Key Concepts**:
- `push()` takes ownership (parameter `T`)
- `pop()` returns ownership (`Option<T>`)
- `peek()` borrows immutably (`Option<&T>`)

---

### Recipe 2: Queue (FIFO) - Borrowing for Inspection

**Problem**: Implement a queue that allows inspection without removing elements.

**Use Case**: Task scheduling, message queues, BFS algorithm.

**Pattern**: FIFO (First In, First Out)

```rust
struct Queue<T> {
    items: Vec<T>,
}

impl<T> Queue<T> {
    fn new() -> Self {
        Queue { items: Vec::new() }
    }

    fn enqueue(&mut self, item: T) {
        self.items.push(item);
    }

    fn dequeue(&mut self) -> Option<T> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    // Borrow to inspect front
    fn front(&self) -> Option<&T> {
        self.items.first()
    }

    // Borrow to iterate without consuming
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

fn main() {
    let mut queue = Queue::new();

    queue.enqueue(1);
    queue.enqueue(2);
    queue.enqueue(3);

    // Inspect without removing
    println!("Front: {:?}", queue.front());
    println!("Queue still has {} items", queue.len());

    // Iterate without consuming
    for item in queue.iter() {
        println!("Item: {}", item);
    }

    // Now dequeue
    while let Some(item) = queue.dequeue() {
        println!("Dequeued: {}", item);
    }
}
```

**Key Concepts**:
- `enqueue()` takes ownership
- `dequeue()` returns ownership
- `front()` and `iter()` borrow without consuming

---

### Recipe 3: Singly Linked List - Box and Option

**Problem**: Implement a linked list using heap allocation.

**Use Case**: Dynamic data structures, understanding Box ownership.

**Pattern**: Linked List with owned nodes

```rust
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

struct LinkedList<T> {
    head: Option<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList { head: None }
    }

    fn push_front(&mut self, value: T) {
        let new_node = Box::new(Node {
            value,
            next: self.head.take(),  // Take ownership from head
        });
        self.head = Some(new_node);
    }

    fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.value
        })
    }

    // Borrow to peek
    fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    // Mutable borrow to peek and modify
    fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| &mut node.value)
    }
}

fn main() {
    let mut list = LinkedList::new();

    list.push_front(1);
    list.push_front(2);
    list.push_front(3);

    // Peek without consuming
    if let Some(val) = list.peek() {
        println!("Front: {}", val);
    }

    // Modify through mutable borrow
    if let Some(val) = list.peek_mut() {
        *val += 10;
    }

    // Pop and consume
    while let Some(val) = list.pop_front() {
        println!("Popped: {}", val);
    }
}
```

**Key Concepts**:
- `Box<T>` provides heap allocation with exclusive ownership
- `Option::take()` moves ownership out, leaving `None`
- `as_ref()` and `as_mut()` convert `Option<Box<T>>` to `Option<&T>`

---

### Recipe 4: Binary Search Tree - Recursive Ownership

**Problem**: Implement a BST with insertion and search.

**Use Case**: Ordered data storage, understanding recursive data structures.

**Pattern**: Binary Search Tree

```rust
#[derive(Debug)]
struct TreeNode<T> {
    value: T,
    left: Option<Box<TreeNode<T>>>,
    right: Option<Box<TreeNode<T>>>,
}

struct BST<T> {
    root: Option<Box<TreeNode<T>>>,
}

impl<T: Ord> BST<T> {
    fn new() -> Self {
        BST { root: None }
    }

    fn insert(&mut self, value: T) {
        self.root = Self::insert_recursive(self.root.take(), value);
    }

    fn insert_recursive(node: Option<Box<TreeNode<T>>>, value: T) -> Option<Box<TreeNode<T>>> {
        match node {
            None => Some(Box::new(TreeNode {
                value,
                left: None,
                right: None,
            })),
            Some(mut n) => {
                if value < n.value {
                    n.left = Self::insert_recursive(n.left.take(), value);
                } else {
                    n.right = Self::insert_recursive(n.right.take(), value);
                }
                Some(n)
            }
        }
    }

    // Borrow to search
    fn search(&self, value: &T) -> bool {
        Self::search_recursive(&self.root, value)
    }

    fn search_recursive(node: &Option<Box<TreeNode<T>>>, value: &T) -> bool {
        match node {
            None => false,
            Some(n) => {
                if value == &n.value {
                    true
                } else if value < &n.value {
                    Self::search_recursive(&n.left, value)
                } else {
                    Self::search_recursive(&n.right, value)
                }
            }
        }
    }

    // In-order traversal with borrowing
    fn inorder(&self) -> Vec<&T> {
        let mut result = Vec::new();
        Self::inorder_recursive(&self.root, &mut result);
        result
    }

    fn inorder_recursive<'a>(node: &'a Option<Box<TreeNode<T>>>, result: &mut Vec<&'a T>) {
        if let Some(n) = node {
            Self::inorder_recursive(&n.left, result);
            result.push(&n.value);
            Self::inorder_recursive(&n.right, result);
        }
    }
}

fn main() {
    let mut bst = BST::new();

    bst.insert(5);
    bst.insert(3);
    bst.insert(7);
    bst.insert(1);
    bst.insert(9);

    println!("Search 7: {}", bst.search(&7));
    println!("Search 4: {}", bst.search(&4));

    // Borrow values for iteration
    let values = bst.inorder();
    println!("In-order: {:?}", values);
}
```

**Key Concepts**:
- Recursive data structures use `Box` for heap allocation
- `take()` temporarily moves ownership for reconstruction
- Search borrows, doesn't need ownership
- Lifetimes in `inorder_recursive` tie returned references to tree

---

## Sorting Algorithms

### Recipe 5: Merge Sort - Ownership and Splitting

**Problem**: Implement merge sort demonstrating ownership transfer.

**Use Case**: Stable sorting, understanding ownership in divide-and-conquer.

**Algorithm**: Merge Sort (O(n log n))

```rust
fn merge_sort<T: Ord + Clone>(mut vec: Vec<T>) -> Vec<T> {
    let len = vec.len();
    if len <= 1 {
        return vec;
    }

    // Split ownership: vec is moved
    let mid = len / 2;
    let right = vec.split_off(mid);  // right takes ownership of second half
    let left = vec;                   // left is the first half

    // Recursively sort (takes ownership, returns ownership)
    let left = merge_sort(left);
    let right = merge_sort(right);

    // Merge (takes ownership of both)
    merge(left, right)
}

fn merge<T: Ord + Clone>(left: Vec<T>, right: Vec<T>) -> Vec<T> {
    let mut result = Vec::with_capacity(left.len() + right.len());
    let mut left_iter = left.into_iter();
    let mut right_iter = right.into_iter();

    let mut left_peek = left_iter.next();
    let mut right_peek = right_iter.next();

    loop {
        match (&left_peek, &right_peek) {
            (Some(l), Some(r)) => {
                if l <= r {
                    result.push(left_peek.take().unwrap());
                    left_peek = left_iter.next();
                } else {
                    result.push(right_peek.take().unwrap());
                    right_peek = right_iter.next();
                }
            }
            (Some(_), None) => {
                result.push(left_peek.take().unwrap());
                result.extend(left_iter);
                break;
            }
            (None, Some(_)) => {
                result.push(right_peek.take().unwrap());
                result.extend(right_iter);
                break;
            }
            (None, None) => break,
        }
    }

    result
}

fn main() {
    let vec = vec![64, 34, 25, 12, 22, 11, 90];
    println!("Original: {:?}", vec);

    let sorted = merge_sort(vec);  // vec moved into function
    println!("Sorted: {:?}", sorted);

    // println!("{:?}", vec);  // Error: vec was moved
}
```

**Key Concepts**:
- `split_off()` transfers ownership of second half
- Function takes ownership and returns new owned Vec
- `into_iter()` consumes the vector, transferring ownership of elements
- `extend()` consumes the iterator

---

### Recipe 6: Quick Sort (In-Place) - Mutable Borrowing

**Problem**: Implement in-place quick sort using mutable borrows.

**Use Case**: Fast sorting without extra allocation.

**Algorithm**: Quick Sort (O(n log n) average)

```rust
fn quick_sort<T: Ord>(slice: &mut [T]) {
    if slice.len() <= 1 {
        return;
    }

    let pivot_index = partition(slice);

    // Split into two mutable borrows (disjoint slices)
    let (left, right) = slice.split_at_mut(pivot_index);

    quick_sort(left);
    quick_sort(&mut right[1..]);  // Skip pivot
}

fn partition<T: Ord>(slice: &mut [T]) -> usize {
    let len = slice.len();
    let pivot_index = len / 2;

    slice.swap(pivot_index, len - 1);

    let mut i = 0;
    for j in 0..len - 1 {
        if slice[j] <= slice[len - 1] {
            slice.swap(i, j);
            i += 1;
        }
    }

    slice.swap(i, len - 1);
    i
}

fn main() {
    let mut vec = vec![64, 34, 25, 12, 22, 11, 90];
    println!("Original: {:?}", vec);

    quick_sort(&mut vec);  // Borrow mutably
    println!("Sorted: {:?}", vec);

    // vec still valid - it was only borrowed
    println!("Still accessible: {:?}", vec);
}
```

**Key Concepts**:
- `&mut [T]` borrows slice mutably
- `split_at_mut()` creates two non-overlapping mutable borrows
- No ownership transfer - original vector stays valid
- In-place modification is efficient

---

### Recipe 7: Bubble Sort - Iterator with Mutable Borrowing

**Problem**: Implement bubble sort showing iteration with mutation.

**Use Case**: Understanding mutable iteration patterns.

**Algorithm**: Bubble Sort (O(n²))

```rust
fn bubble_sort<T: Ord>(slice: &mut [T]) {
    let len = slice.len();

    for i in 0..len {
        let mut swapped = false;

        // Mutable windows for comparison
        for j in 0..len - i - 1 {
            if slice[j] > slice[j + 1] {
                slice.swap(j, j + 1);
                swapped = true;
            }
        }

        if !swapped {
            break;
        }
    }
}

// Alternative: using chunks_exact_mut (more idiomatic)
fn bubble_sort_chunks<T: Ord>(slice: &mut [T]) {
    let len = slice.len();

    for i in 0..len {
        let mut swapped = false;

        // Borrow overlapping pairs mutably
        for j in 0..len - i - 1 {
            let (left, right) = slice[j..].split_first_mut().unwrap();
            if left > &right[0] {
                std::mem::swap(left, &mut right[0]);
                swapped = true;
            }
        }

        if !swapped {
            break;
        }
    }
}

fn main() {
    let mut vec = vec![64, 34, 25, 12, 22, 11, 90];

    bubble_sort(&mut vec);
    println!("Sorted: {:?}", vec);
}
```

**Key Concepts**:
- `swap()` requires mutable borrow of slice
- `split_first_mut()` borrows first element and rest separately
- Multiple mutable borrows work when accessing different indices

---

## Search and Tree Algorithms

### Recipe 8: Binary Search - Immutable Borrowing

**Problem**: Implement binary search that borrows without ownership.

**Use Case**: Efficient searching in sorted data.

**Algorithm**: Binary Search (O(log n))

```rust
fn binary_search<T: Ord>(slice: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = slice.len();

    while left < right {
        let mid = left + (right - left) / 2;

        match slice[mid].cmp(target) {
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }

    None
}

// Recursive version with lifetimes
fn binary_search_recursive<'a, T: Ord>(
    slice: &'a [T],
    target: &T,
    offset: usize,
) -> Option<usize> {
    if slice.is_empty() {
        return None;
    }

    let mid = slice.len() / 2;

    match slice[mid].cmp(target) {
        std::cmp::Ordering::Equal => Some(offset + mid),
        std::cmp::Ordering::Less => {
            binary_search_recursive(&slice[mid + 1..], target, offset + mid + 1)
        }
        std::cmp::Ordering::Greater => {
            binary_search_recursive(&slice[..mid], target, offset)
        }
    }
}

fn main() {
    let vec = vec![1, 3, 5, 7, 9, 11, 13, 15];

    if let Some(index) = binary_search(&vec, &7) {
        println!("Found 7 at index {}", index);
    }

    if let Some(index) = binary_search_recursive(&vec, &13, 0) {
        println!("Found 13 at index {}", index);
    }

    // vec still valid - only borrowed
    println!("Vector: {:?}", vec);
}
```

**Key Concepts**:
- `&[T]` immutable slice borrow
- Lifetime `'a` ties slice lifetime to function
- Slice syntax `&slice[mid..]` creates sub-slices without allocation
- Original data remains untouched

---

### Recipe 9: Depth-First Search (DFS) - Graph Traversal

**Problem**: Implement DFS showing ownership of visited state.

**Use Case**: Graph traversal, maze solving, topological sort.

**Algorithm**: Depth-First Search

```rust
use std::collections::{HashMap, HashSet};

struct Graph {
    edges: HashMap<usize, Vec<usize>>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.entry(from).or_insert(Vec::new()).push(to);
    }

    // DFS with mutable borrow of visited set
    fn dfs(&self, start: usize, visited: &mut HashSet<usize>) {
        if visited.contains(&start) {
            return;
        }

        visited.insert(start);
        println!("Visiting: {}", start);

        if let Some(neighbors) = self.edges.get(&start) {
            for &neighbor in neighbors {
                self.dfs(neighbor, visited);
            }
        }
    }

    // DFS that returns owned result
    fn dfs_path(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        if self.dfs_path_helper(start, end, &mut visited, &mut path) {
            Some(path)
        } else {
            None
        }
    }

    fn dfs_path_helper(
        &self,
        current: usize,
        end: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
    ) -> bool {
        visited.insert(current);
        path.push(current);

        if current == end {
            return true;
        }

        if let Some(neighbors) = self.edges.get(&current) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    if self.dfs_path_helper(neighbor, end, visited, path) {
                        return true;
                    }
                }
            }
        }

        path.pop();
        false
    }
}

fn main() {
    let mut graph = Graph::new();
    graph.add_edge(0, 1);
    graph.add_edge(0, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 3);
    graph.add_edge(3, 4);

    let mut visited = HashSet::new();
    println!("DFS traversal:");
    graph.dfs(0, &mut visited);

    if let Some(path) = graph.dfs_path(0, 4) {
        println!("Path from 0 to 4: {:?}", path);
    }
}
```

**Key Concepts**:
- `&self` borrows graph immutably
- `&mut HashSet` borrows visited set mutably
- Multiple functions can borrow visited, but only one mutably
- Returned `Vec` transfers ownership to caller

---

### Recipe 10: Breadth-First Search (BFS) - Queue Pattern

**Problem**: Implement BFS using queue with owned elements.

**Use Case**: Shortest path, level-order traversal.

**Algorithm**: Breadth-First Search

```rust
use std::collections::{HashMap, HashSet, VecDeque};

struct Graph {
    edges: HashMap<usize, Vec<usize>>,
}

impl Graph {
    fn new() -> Self {
        Graph {
            edges: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.entry(from).or_insert(Vec::new()).push(to);
    }

    // BFS with owned queue
    fn bfs(&self, start: usize) -> Vec<usize> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            result.push(node);

            if let Some(neighbors) = self.edges.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        result
    }

    // BFS shortest path
    fn shortest_path(&self, start: usize, end: usize) -> Option<Vec<usize>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<usize, usize> = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(node) = queue.pop_front() {
            if node == end {
                // Reconstruct path
                let mut path = Vec::new();
                let mut current = end;
                path.push(current);

                while let Some(&p) = parent.get(&current) {
                    path.push(p);
                    current = p;
                }

                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.edges.get(&node) {
                for &neighbor in neighbors {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        parent.insert(neighbor, node);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }
}

fn main() {
    let mut graph = Graph::new();
    graph.add_edge(0, 1);
    graph.add_edge(0, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 3);
    graph.add_edge(3, 4);

    let traversal = graph.bfs(0);
    println!("BFS traversal: {:?}", traversal);

    if let Some(path) = graph.shortest_path(0, 4) {
        println!("Shortest path: {:?}", path);
    }
}
```

**Key Concepts**:
- `VecDeque` owns elements in the queue
- `pop_front()` transfers ownership out
- `push_back()` transfers ownership in
- HashMap and HashSet own their data

---

## Design Patterns

### Recipe 11: Builder Pattern - Consuming Self

**Problem**: Implement builder pattern with ownership transfer.

**Use Case**: Configuration builders, fluent APIs.

**Pattern**: Builder Pattern

```rust
struct Database {
    host: String,
    port: u16,
    username: String,
    password: String,
    max_connections: u32,
}

struct DatabaseBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    max_connections: Option<u32>,
}

impl DatabaseBuilder {
    fn new() -> Self {
        DatabaseBuilder {
            host: None,
            port: None,
            username: None,
            password: None,
            max_connections: None,
        }
    }

    // Consumes self, returns self (ownership transfer)
    fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = Some(max);
        self
    }

    // Consumes builder, returns final product
    fn build(self) -> Result<Database, String> {
        Ok(Database {
            host: self.host.ok_or("Host is required")?,
            port: self.port.unwrap_or(5432),
            username: self.username.ok_or("Username is required")?,
            password: self.password.ok_or("Password is required")?,
            max_connections: self.max_connections.unwrap_or(10),
        })
    }
}

fn main() {
    let db = DatabaseBuilder::new()
        .host("localhost".to_string())
        .port(5432)
        .username("admin".to_string())
        .password("secret".to_string())
        .max_connections(20)
        .build()
        .expect("Failed to build database");

    println!("Connected to {}:{}", db.host, db.port);
}
```

**Key Concepts**:
- Each method takes `self` (not `&self`), consuming the builder
- Returns `Self` to enable chaining
- `build()` consumes the builder, preventing reuse
- Forces linear construction flow

---

### Recipe 12: Command Pattern - Boxed Trait Objects

**Problem**: Implement command pattern with dynamic dispatch.

**Use Case**: Undo/redo, task queues, event handling.

**Pattern**: Command Pattern

```rust
// Command trait
trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}

// Concrete commands
struct AddCommand {
    receiver: Box<TextEditor>,
    text: String,
}

impl Command for AddCommand {
    fn execute(&mut self) {
        self.receiver.add_text(&self.text);
    }

    fn undo(&mut self) {
        self.receiver.delete_text(self.text.len());
    }
}

struct DeleteCommand {
    receiver: Box<TextEditor>,
    deleted_text: String,
    count: usize,
}

impl Command for DeleteCommand {
    fn execute(&mut self) {
        self.deleted_text = self.receiver.delete_text(self.count);
    }

    fn undo(&mut self) {
        self.receiver.add_text(&self.deleted_text);
    }
}

// Receiver
struct TextEditor {
    content: String,
}

impl TextEditor {
    fn new() -> Self {
        TextEditor {
            content: String::new(),
        }
    }

    fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    fn delete_text(&mut self, count: usize) -> String {
        let split_pos = self.content.len().saturating_sub(count);
        let deleted = self.content[split_pos..].to_string();
        self.content.truncate(split_pos);
        deleted
    }

    fn get_content(&self) -> &str {
        &self.content
    }
}

// Invoker
struct CommandManager {
    history: Vec<Box<dyn Command>>,
    position: usize,
}

impl CommandManager {
    fn new() -> Self {
        CommandManager {
            history: Vec::new(),
            position: 0,
        }
    }

    fn execute(&mut self, mut command: Box<dyn Command>) {
        command.execute();

        // Remove any commands after current position
        self.history.truncate(self.position);

        self.history.push(command);
        self.position += 1;
    }

    fn undo(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            self.history[self.position].undo();
        }
    }

    fn redo(&mut self) {
        if self.position < self.history.len() {
            self.history[self.position].execute();
            self.position += 1;
        }
    }
}

fn main() {
    let mut manager = CommandManager::new();
    let editor = Box::new(TextEditor::new());

    // Note: In real implementation, you'd use Rc<RefCell<>> to share editor
    // This is simplified for demonstration

    println!("Command pattern with ownership transfer demonstrated");
}
```

**Key Concepts**:
- `Box<dyn Command>` provides heap allocation and dynamic dispatch
- Trait objects require indirection (Box, Rc, etc.)
- Commands own their state and receivers
- History vector owns all command objects

---

### Recipe 13: Strategy Pattern - Borrowed Functions

**Problem**: Implement strategy pattern with borrowed callbacks.

**Use Case**: Sorting strategies, compression algorithms, payment methods.

**Pattern**: Strategy Pattern

```rust
// Strategy trait
trait SortStrategy {
    fn sort(&self, data: &mut [i32]);
}

// Concrete strategies
struct BubbleSort;

impl SortStrategy for BubbleSort {
    fn sort(&self, data: &mut [i32]) {
        let len = data.len();
        for i in 0..len {
            for j in 0..len - i - 1 {
                if data[j] > data[j + 1] {
                    data.swap(j, j + 1);
                }
            }
        }
    }
}

struct QuickSortStrategy;

impl SortStrategy for QuickSortStrategy {
    fn sort(&self, data: &mut [i32]) {
        if data.len() <= 1 {
            return;
        }
        let pivot_index = partition(data);
        let (left, right) = data.split_at_mut(pivot_index);
        self.sort(left);
        self.sort(&mut right[1..]);
    }
}

fn partition(data: &mut [i32]) -> usize {
    let len = data.len();
    let pivot_index = len / 2;
    data.swap(pivot_index, len - 1);

    let mut i = 0;
    for j in 0..len - 1 {
        if data[j] <= data[len - 1] {
            data.swap(i, j);
            i += 1;
        }
    }
    data.swap(i, len - 1);
    i
}

// Context
struct Sorter<'a> {
    strategy: &'a dyn SortStrategy,
}

impl<'a> Sorter<'a> {
    fn new(strategy: &'a dyn SortStrategy) -> Self {
        Sorter { strategy }
    }

    fn sort(&self, data: &mut [i32]) {
        self.strategy.sort(data);
    }
}

fn main() {
    let mut data1 = vec![64, 34, 25, 12, 22, 11, 90];
    let mut data2 = data1.clone();

    let bubble = BubbleSort;
    let quick = QuickSortStrategy;

    let sorter1 = Sorter::new(&bubble);
    sorter1.sort(&mut data1);
    println!("Bubble sort: {:?}", data1);

    let sorter2 = Sorter::new(&quick);
    sorter2.sort(&mut data2);
    println!("Quick sort: {:?}", data2);
}
```

**Key Concepts**:
- Lifetime `'a` ties strategy reference to Sorter
- `&dyn SortStrategy` borrows trait object
- Strategy doesn't own data, only borrows during operation
- Multiple strategies can exist without ownership conflicts

---

### Recipe 14: Observer Pattern - Rc and RefCell

**Problem**: Implement observer pattern with shared ownership.

**Use Case**: Event systems, pub-sub, GUI frameworks.

**Pattern**: Observer Pattern

```rust
use std::cell::RefCell;
use std::rc::Rc;

// Observer trait
trait Observer {
    fn update(&mut self, message: &str);
}

// Concrete observer
struct EmailNotifier {
    email: String,
}

impl Observer for EmailNotifier {
    fn update(&mut self, message: &str) {
        println!("Sending email to {}: {}", self.email, message);
    }
}

struct SmsNotifier {
    phone: String,
}

impl Observer for SmsNotifier {
    fn update(&mut self, message: &str) {
        println!("Sending SMS to {}: {}", self.phone, message);
    }
}

// Subject
struct NewsAgency {
    observers: Vec<Rc<RefCell<dyn Observer>>>,
    news: String,
}

impl NewsAgency {
    fn new() -> Self {
        NewsAgency {
            observers: Vec::new(),
            news: String::new(),
        }
    }

    fn attach(&mut self, observer: Rc<RefCell<dyn Observer>>) {
        self.observers.push(observer);
    }

    fn set_news(&mut self, news: String) {
        self.news = news;
        self.notify();
    }

    fn notify(&self) {
        for observer in &self.observers {
            observer.borrow_mut().update(&self.news);
        }
    }
}

fn main() {
    let mut agency = NewsAgency::new();

    let email_observer = Rc::new(RefCell::new(EmailNotifier {
        email: "user@example.com".to_string(),
    }));

    let sms_observer = Rc::new(RefCell::new(SmsNotifier {
        phone: "+1234567890".to_string(),
    }));

    agency.attach(email_observer.clone());
    agency.attach(sms_observer.clone());

    agency.set_news("Breaking: Rust 2.0 Released!".to_string());

    // Observers still accessible
    email_observer.borrow_mut().update("Direct message");
}
```

**Key Concepts**:
- `Rc<RefCell<T>>` enables shared mutable ownership
- `Rc` provides reference counting (shared ownership)
- `RefCell` enables interior mutability (runtime borrow checking)
- `clone()` on Rc increments reference count, doesn't deep copy

---

### Recipe 15: Factory Pattern - Returning Owned Objects

**Problem**: Implement factory pattern returning owned products.

**Use Case**: Object creation, dependency injection.

**Pattern**: Factory Pattern

```rust
// Product trait
trait Button {
    fn render(&self) -> String;
    fn on_click(&self);
}

// Concrete products
struct WindowsButton {
    label: String,
}

impl Button for WindowsButton {
    fn render(&self) -> String {
        format!("[Windows Button: {}]", self.label)
    }

    fn on_click(&self) {
        println!("Windows button clicked: {}", self.label);
    }
}

struct MacButton {
    label: String,
}

impl Button for MacButton {
    fn render(&self) -> String {
        format!("(Mac Button: {})", self.label)
    }

    fn on_click(&self) {
        println!("Mac button clicked: {}", self.label);
    }
}

// Factory trait
trait UIFactory {
    fn create_button(&self, label: String) -> Box<dyn Button>;
}

// Concrete factories
struct WindowsFactory;

impl UIFactory for WindowsFactory {
    fn create_button(&self, label: String) -> Box<dyn Button> {
        Box::new(WindowsButton { label })
    }
}

struct MacFactory;

impl UIFactory for MacFactory {
    fn create_button(&self, label: String) -> Box<dyn Button> {
        Box::new(MacButton { label })
    }
}

// Client code
fn create_ui(factory: &dyn UIFactory) {
    let button1 = factory.create_button("OK".to_string());
    let button2 = factory.create_button("Cancel".to_string());

    println!("{}", button1.render());
    println!("{}", button2.render());

    button1.on_click();
    button2.on_click();
}

fn main() {
    let windows = WindowsFactory;
    let mac = MacFactory;

    println!("Windows UI:");
    create_ui(&windows);

    println!("\nMac UI:");
    create_ui(&mac);
}
```

**Key Concepts**:
- Factory returns `Box<dyn Trait>` transferring ownership
- Caller owns created objects
- Factory borrows label String, moves it into new object
- Trait objects enable polymorphism

---

## Iterator Patterns

### Recipe 16: Custom Iterator - Lending Iterator

**Problem**: Implement iterator that yields borrowed values.

**Use Case**: Iterating over collections without copying.

**Pattern**: Iterator Pattern

```rust
struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    write_pos: usize,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        RingBuffer {
            data: Vec::with_capacity(capacity),
            capacity,
            write_pos: 0,
        }
    }

    fn push(&mut self, item: T) {
        if self.data.len() < self.capacity {
            self.data.push(item);
        } else {
            self.data[self.write_pos] = item;
        }
        self.write_pos = (self.write_pos + 1) % self.capacity;
    }

    // Borrow to iterate
    fn iter(&self) -> RingBufferIter<T> {
        RingBufferIter {
            buffer: self,
            position: 0,
        }
    }
}

// Iterator that borrows from RingBuffer
struct RingBufferIter<'a, T> {
    buffer: &'a RingBuffer<T>,
    position: usize,
}

impl<'a, T> Iterator for RingBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.buffer.data.len() {
            let item = &self.buffer.data[self.position];
            self.position += 1;
            Some(item)
        } else {
            None
        }
    }
}

fn main() {
    let mut buffer = RingBuffer::new(3);

    buffer.push(1);
    buffer.push(2);
    buffer.push(3);
    buffer.push(4);  // Overwrites 1

    // Iterate without consuming
    for item in buffer.iter() {
        println!("{}", item);
    }

    // Buffer still valid
    buffer.push(5);

    for item in buffer.iter() {
        println!("{}", item);
    }
}
```

**Key Concepts**:
- Iterator has lifetime `'a` tied to collection
- Yields `&'a T` - references that live as long as collection
- Collection must outlive iterator
- Non-consuming iteration

---

### Recipe 17: Consuming Iterator - IntoIterator

**Problem**: Implement iterator that takes ownership.

**Use Case**: Consuming collections, transforming ownership.

**Pattern**: Consuming Iterator

```rust
struct TreeNode<T> {
    value: T,
    children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    fn new(value: T) -> Self {
        TreeNode {
            value,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }
}

// Consuming iterator - takes ownership
struct TreeIntoIter<T> {
    stack: Vec<TreeNode<T>>,
}

impl<T> Iterator for TreeIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            // Add children to stack (ownership transferred)
            for child in node.children.into_iter().rev() {
                self.stack.push(child);
            }
            node.value
        })
    }
}

impl<T> IntoIterator for TreeNode<T> {
    type Item = T;
    type IntoIter = TreeIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        TreeIntoIter { stack: vec![self] }
    }
}

fn main() {
    let mut root = TreeNode::new(1);

    let mut child1 = TreeNode::new(2);
    child1.add_child(TreeNode::new(4));
    child1.add_child(TreeNode::new(5));

    let mut child2 = TreeNode::new(3);
    child2.add_child(TreeNode::new(6));

    root.add_child(child1);
    root.add_child(child2);

    // Consuming iteration
    for value in root {  // root is moved
        println!("{}", value);
    }

    // println!("{}", root.value);  // Error: root was moved
}
```

**Key Concepts**:
- `into_iter()` consumes collection
- Iterator owns its data
- Each `next()` call transfers ownership of item
- Original collection no longer accessible

---

## Smart Pointer Patterns

### Recipe 18: Reference Counted List - Rc

**Problem**: Implement shared ownership with reference counting.

**Use Case**: Shared data structures, graph with multiple owners.

**Pattern**: Shared Ownership with Rc

```rust
use std::rc::Rc;

#[derive(Debug)]
struct Node {
    value: i32,
    next: Option<Rc<Node>>,
}

fn main() {
    // Create shared nodes
    let node3 = Rc::new(Node {
        value: 3,
        next: None,
    });

    let node2 = Rc::new(Node {
        value: 2,
        next: Some(Rc::clone(&node3)),
    });

    let node1a = Rc::new(Node {
        value: 1,
        next: Some(Rc::clone(&node2)),
    });

    let node1b = Rc::new(Node {
        value: 1,
        next: Some(Rc::clone(&node2)),
    });

    println!("node2 reference count: {}", Rc::strong_count(&node2));
    println!("node3 reference count: {}", Rc::strong_count(&node3));

    // Both lists share node2 and node3
    println!("List 1a: {:?}", node1a);
    println!("List 1b: {:?}", node1b);
}
```

**Key Concepts**:
- `Rc<T>` provides shared ownership
- `Rc::clone()` increments reference count (cheap)
- Last owner dropping causes deallocation
- Single-threaded only

---

### Recipe 19: Thread-Safe Reference Counting - Arc

**Problem**: Implement shared ownership across threads.

**Use Case**: Parallel processing, shared configuration.

**Pattern**: Thread-Safe Shared Ownership

```rust
use std::sync::Arc;
use std::thread;

struct SharedData {
    values: Vec<i32>,
}

impl SharedData {
    fn sum(&self) -> i32 {
        self.values.iter().sum()
    }
}

fn main() {
    let data = Arc::new(SharedData {
        values: vec![1, 2, 3, 4, 5],
    });

    let mut handles = vec![];

    // Spawn multiple threads sharing data
    for i in 0..3 {
        let data_clone = Arc::clone(&data);

        let handle = thread::spawn(move || {
            let sum = data_clone.sum();
            println!("Thread {} sees sum: {}", i, sum);
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("Main thread still has data: {:?}", data.values);
    println!("Reference count: {}", Arc::strong_count(&data));
}
```

**Key Concepts**:
- `Arc<T>` is thread-safe version of `Rc<T>`
- Atomic reference counting (slight overhead)
- Can be moved across threads
- Immutable shared access

---

### Recipe 20: Interior Mutability - RefCell

**Problem**: Implement mutable access through immutable reference.

**Use Case**: Mock objects, caching, implementation details.

**Pattern**: Interior Mutability

```rust
use std::cell::RefCell;

struct Metrics {
    call_count: RefCell<u32>,
    cache: RefCell<Vec<String>>,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            call_count: RefCell::new(0),
            cache: RefCell::new(Vec::new()),
        }
    }

    // Takes &self but mutates interior
    fn log_call(&self, operation: &str) {
        *self.call_count.borrow_mut() += 1;
        self.cache.borrow_mut().push(operation.to_string());
    }

    fn get_call_count(&self) -> u32 {
        *self.call_count.borrow()
    }

    fn get_operations(&self) -> Vec<String> {
        self.cache.borrow().clone()
    }
}

struct Database {
    metrics: Metrics,
}

impl Database {
    fn new() -> Self {
        Database {
            metrics: Metrics::new(),
        }
    }

    // Immutable methods that update metrics
    fn query(&self, sql: &str) {
        self.metrics.log_call("query");
        println!("Executing: {}", sql);
    }

    fn insert(&self, table: &str) {
        self.metrics.log_call("insert");
        println!("Inserting into: {}", table);
    }

    fn report(&self) {
        println!("Total calls: {}", self.metrics.get_call_count());
        println!("Operations: {:?}", self.metrics.get_operations());
    }
}

fn main() {
    let db = Database::new();

    db.query("SELECT * FROM users");
    db.insert("users");
    db.query("SELECT * FROM posts");

    db.report();
}
```

**Key Concepts**:
- `RefCell<T>` enables mutation through `&self`
- Runtime borrow checking (panics on violation)
- `borrow()` returns immutable guard
- `borrow_mut()` returns mutable guard
- Guards enforce borrowing rules at runtime

---

### Recipe 21: Shared Mutable State - Arc<Mutex<T>>

**Problem**: Implement thread-safe mutable shared state.

**Use Case**: Parallel computation with shared results.

**Pattern**: Thread-Safe Shared Mutable State

```rust
use std::sync::{Arc, Mutex};
use std::thread;

struct Counter {
    value: Mutex<i32>,
}

impl Counter {
    fn new() -> Self {
        Counter {
            value: Mutex::new(0),
        }
    }

    fn increment(&self) {
        let mut val = self.value.lock().unwrap();
        *val += 1;
    }

    fn get(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

fn main() {
    let counter = Arc::new(Counter::new());
    let mut handles = vec![];

    for _ in 0..10 {
        let counter_clone = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                counter_clone.increment();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", counter.get());
}
```

**Key Concepts**:
- `Arc<Mutex<T>>` combines shared ownership and mutability
- `Mutex` ensures exclusive access across threads
- `lock()` blocks until mutex is available
- MutexGuard automatically unlocks when dropped

---

## Concurrency Patterns

### Recipe 22: Message Passing - Channel Ownership

**Problem**: Implement communication between threads using channels.

**Use Case**: Producer-consumer, pipeline processing.

**Pattern**: Message Passing

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct Task {
    id: usize,
    data: Vec<i32>,
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Producer thread
    let producer = thread::spawn(move || {
        for i in 0..5 {
            let task = Task {
                id: i,
                data: vec![i as i32; 100],
            };

            println!("Sending task {}", i);
            tx.send(task).unwrap();  // Ownership transferred
            thread::sleep(Duration::from_millis(100));
        }
        // tx dropped here, signaling completion
    });

    // Consumer thread
    let consumer = thread::spawn(move || {
        while let Ok(task) = rx.recv() {
            println!("Processing task {}", task.id);
            let sum: i32 = task.data.iter().sum();
            println!("Task {} sum: {}", task.id, sum);
        }
        println!("All tasks processed");
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}
```

**Key Concepts**:
- `send()` transfers ownership through channel
- Sender and receiver can be in different threads
- Channel closed when all senders dropped
- Zero-copy transfer of ownership

---

### Recipe 23: Parallel Map-Reduce

**Problem**: Implement parallel map-reduce pattern.

**Use Case**: Data processing, aggregation, parallel computation.

**Pattern**: Map-Reduce

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn parallel_map_reduce<T, M, R>(
    data: Vec<T>,
    num_threads: usize,
    map_fn: M,
    reduce_fn: R,
) -> Vec<i32>
where
    T: Send + 'static,
    M: Fn(T) -> i32 + Send + Sync + 'static,
    R: Fn(i32, i32) -> i32 + Send + Sync + 'static,
{
    let data = Arc::new(Mutex::new(data));
    let results = Arc::new(Mutex::new(Vec::new()));
    let map_fn = Arc::new(map_fn);
    let mut handles = vec![];

    for _ in 0..num_threads {
        let data_clone = Arc::clone(&data);
        let results_clone = Arc::clone(&results);
        let map_fn_clone = Arc::clone(&map_fn);

        let handle = thread::spawn(move || {
            loop {
                let item = {
                    let mut d = data_clone.lock().unwrap();
                    d.pop()
                };

                match item {
                    Some(item) => {
                        let result = map_fn_clone(item);
                        results_clone.lock().unwrap().push(result);
                    }
                    None => break,
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(results)
        .unwrap()
        .into_inner()
        .unwrap()
}

fn main() {
    let data: Vec<i32> = (1..=100).collect();

    let results = parallel_map_reduce(
        data,
        4,
        |x| x * x,           // Map: square each number
        |a, b| a + b,        // Reduce: sum (not used in this simplified version)
    );

    let sum: i32 = results.iter().sum();
    println!("Sum of squares: {}", sum);
}
```

**Key Concepts**:
- `Arc<Mutex<Vec<T>>>` shares mutable collection
- Worker threads take ownership of items
- Results collected in shared vector
- `Arc::try_unwrap()` recovers ownership when all threads done

---

## Memory Management Patterns

### Recipe 24: Pool Pattern - Reusable Objects

**Problem**: Implement object pool to avoid allocation overhead.

**Use Case**: Connection pooling, buffer reuse, performance optimization.

**Pattern**: Object Pool

```rust
struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    fn new(size: usize) -> Self {
        Buffer {
            data: vec![0; size],
        }
    }

    fn clear(&mut self) {
        self.data.fill(0);
    }
}

struct BufferPool {
    available: Vec<Buffer>,
    size: usize,
}

impl BufferPool {
    fn new(pool_size: usize, buffer_size: usize) -> Self {
        let mut available = Vec::new();
        for _ in 0..pool_size {
            available.push(Buffer::new(buffer_size));
        }

        BufferPool {
            available,
            size: buffer_size,
        }
    }

    // Takes ownership from pool
    fn acquire(&mut self) -> Option<Buffer> {
        self.available.pop()
    }

    // Returns ownership to pool
    fn release(&mut self, mut buffer: Buffer) {
        buffer.clear();
        self.available.push(buffer);
    }

    fn available_count(&self) -> usize {
        self.available.len()
    }
}

fn main() {
    let mut pool = BufferPool::new(3, 1024);

    println!("Available buffers: {}", pool.available_count());

    // Acquire buffers
    let buffer1 = pool.acquire().unwrap();
    let buffer2 = pool.acquire().unwrap();

    println!("Available buffers: {}", pool.available_count());

    // Use buffers...

    // Return to pool
    pool.release(buffer1);
    pool.release(buffer2);

    println!("Available buffers: {}", pool.available_count());
}
```

**Key Concepts**:
- Pool owns all objects
- `acquire()` transfers ownership to caller
- `release()` returns ownership to pool
- Avoids repeated allocation/deallocation

---

### Recipe 25: RAII Pattern - Automatic Cleanup

**Problem**: Implement automatic resource management.

**Use Case**: File handles, network connections, locks.

**Pattern**: RAII (Resource Acquisition Is Initialization)

```rust
use std::fs::File;
use std::io::{self, Write};

struct FileWriter {
    file: File,
    bytes_written: usize,
}

impl FileWriter {
    fn new(path: &str) -> io::Result<Self> {
        Ok(FileWriter {
            file: File::create(path)?,
            bytes_written: 0,
        })
    }

    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        self.file.write_all(data)?;
        self.bytes_written += data.len();
        Ok(())
    }

    fn bytes_written(&self) -> usize {
        self.bytes_written
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        println!("Closing file, wrote {} bytes", self.bytes_written);
        // File automatically closed when dropped
    }
}

struct Transaction {
    name: String,
    committed: bool,
}

impl Transaction {
    fn new(name: String) -> Self {
        println!("Starting transaction: {}", name);
        Transaction {
            name,
            committed: false,
        }
    }

    fn commit(&mut self) {
        println!("Committing transaction: {}", self.name);
        self.committed = true;
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed {
            println!("Rolling back transaction: {}", self.name);
        }
    }
}

fn main() -> io::Result<()> {
    {
        let mut writer = FileWriter::new("/tmp/test.txt")?;
        writer.write(b"Hello, ")?;
        writer.write(b"World!")?;
    }  // FileWriter dropped here, file closed

    {
        let mut tx = Transaction::new("update_user".to_string());
        // Some work...
        tx.commit();
    }  // Transaction dropped, but was committed

    {
        let _tx = Transaction::new("failed_operation".to_string());
        // Simulating error - transaction not committed
    }  // Transaction dropped, rolls back automatically

    Ok(())
}
```

**Key Concepts**:
- `Drop` trait runs cleanup when value goes out of scope
- Automatic, deterministic cleanup
- No need for explicit close/cleanup calls
- Exception-safe (even with panic)

---

## Quick Reference

### Ownership Rules

1. **Each value has exactly one owner**
2. **When owner goes out of scope, value is dropped**
3. **Ownership can be transferred (moved)**

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 moved to s2
// s1 is now invalid
```

### Borrowing Rules

1. **Either one mutable reference OR any number of immutable references**
2. **References must always be valid**

```rust
let mut s = String::from("hello");

let r1 = &s;      // OK
let r2 = &s;      // OK
// let r3 = &mut s;  // Error! Can't have &mut while & exists

let r4 = &mut s;  // OK after r1, r2 no longer used
```

### Common Patterns Summary

| Pattern | Signature | Use Case |
|---------|-----------|----------|
| Take ownership | `fn consume(s: String)` | Function needs to own data |
| Borrow immutably | `fn read(s: &String)` | Function needs to read only |
| Borrow mutably | `fn modify(s: &mut String)` | Function needs to modify |
| Return ownership | `fn create() -> String` | Transfer ownership to caller |
| Builder | `fn build(self) -> T` | Fluent API, consuming builder |
| Iterator (borrow) | `fn iter(&self) -> Iter<T>` | Iterate without consuming |
| Iterator (consume) | `fn into_iter(self) -> IntoIter<T>` | Consume and iterate |

### Smart Pointers

| Type | Use Case | Thread-Safe |
|------|----------|-------------|
| `Box<T>` | Heap allocation, single owner | N/A |
| `Rc<T>` | Multiple owners, immutable | No |
| `Arc<T>` | Multiple owners across threads | Yes |
| `RefCell<T>` | Interior mutability (runtime checks) | No |
| `Mutex<T>` | Interior mutability + thread safety | Yes |
| `RwLock<T>` | Multiple readers or one writer | Yes |

### Common Combinations

```rust
// Shared ownership, single-threaded
use std::rc::Rc;
let shared = Rc::new(data);

// Shared ownership, multi-threaded
use std::sync::Arc;
let shared = Arc::new(data);

// Shared mutable, single-threaded
use std::cell::RefCell;
use std::rc::Rc;
let shared = Rc::new(RefCell::new(data));

// Shared mutable, multi-threaded
use std::sync::{Arc, Mutex};
let shared = Arc::new(Mutex::new(data));
```

### Lifetime Syntax

```rust
// Function with lifetime
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct with lifetime
struct Wrapper<'a> {
    data: &'a str,
}

// Multiple lifetimes
fn compare<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x  // Return tied to 'a, not 'b
}
```

---

## Summary

This cookbook demonstrated ownership, borrowing, and lifetimes through:

1. **Data Structures**: Stack, Queue, Linked List, BST
2. **Sorting**: Merge Sort, Quick Sort, Bubble Sort
3. **Search**: Binary Search, DFS, BFS
4. **Design Patterns**: Builder, Command, Strategy, Observer, Factory
5. **Iterators**: Borrowing and consuming iterators
6. **Smart Pointers**: Box, Rc, Arc, RefCell, Mutex
7. **Concurrency**: Message passing, parallel processing
8. **Memory Management**: Pool, RAII

**Key Takeaways**:

- **Ownership** transfers prevent double-free and use-after-free
- **Borrowing** enables access without ownership transfer
- **Lifetimes** ensure references remain valid
- **Smart pointers** enable flexible ownership patterns
- **Drop trait** provides automatic, deterministic cleanup

Understanding these patterns through real algorithms helps build intuition for Rust's memory safety guarantees.
