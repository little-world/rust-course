# Reference Counted DOM Tree with Weak Parent Pointers

## Problem Statement

Build a Document Object Model (DOM) tree similar to HTML/XML where nodes can have multiple children and need to navigate to their parent. This requires handling circular references safely using `Rc`, `Weak`, and `RefCell`.

The tree must support:
- Adding/removing children
- Navigating to parent nodes
- Finding nodes by ID or class
- Modifying node attributes without mut references
- Automatic cleanup when nodes are removed (no memory leaks)
- Event bubbling (child → parent propagation)

## Why It Matters

**Real-World Applications:**
- **Web Browsers**: Chrome/Firefox use similar structures for HTML rendering
- **XML Parsers**: JAXP, DOM4J parse configuration files
- **GUI Frameworks**: Qt, GTK+ use tree structures for UI hierarchies
- **Game Engines**: Scene graphs for Unity/Unreal represent object hierarchies

**Key Learning Outcomes:**
1. Understanding `Rc<RefCell<T>>` pattern for shared mutable state
2. Using `Weak<T>` to break reference cycles and prevent memory leaks
3. Interior mutability with `RefCell` for mutation through shared references
4. Proper memory management in complex data structures
5. Debugging reference count issues

## Use Cases

1. **HTML Parser**: Parse `<div><p>Hello</p></div>` into navigable tree
2. **Configuration Manager**: Nested settings with parent lookup
3. **File System Browser**: Directories with parent navigation
4. **Scene Graph**: 3D game objects with transform hierarchies
5. **Organization Chart**: Employee reporting structure

---

## Core Concepts: Smart Pointers and Memory Management

Before diving into the implementation, understanding these core concepts will help you appreciate why each milestone makes specific design choices and how they combine to solve real-world problems.

### 1. The Ownership Problem in Tree Structures

**The Challenge:**

Rust's ownership rules state that each value has exactly one owner. This works perfectly for simple data structures like vectors or linked lists, but tree structures present unique challenges:

```rust
// This doesn't work in Rust!
struct Node {
    children: Vec<Node>,
    parent: Node,  // ❌ Can't have owned parent - creates infinite recursion!
}
```

**Why Trees Are Different:**

- **Multiple references needed**: Children need to reference parent, parent owns children
- **Shared state**: The same node might need to be accessed from multiple locations
- **Dynamic modification**: Need to add/remove children without moving the entire tree
- **Cycle prevention**: Parent → child → parent creates reference cycles

Real-world tree structures (DOM, file systems, scene graphs) require patterns beyond simple ownership.

### 2. Reference Counting: Shared Ownership with Rc/Arc

**What is Reference Counting?**

Reference counting is a memory management technique where:
- Each value tracks how many references point to it
- When the count reaches zero, the value is automatically deallocated
- No need for garbage collection

**Rc\<T\> (Reference Counted):**

```rust
use std::rc::Rc;

let data = Rc::new(vec![1, 2, 3]);
let reference1 = Rc::clone(&data);  // Count: 2
let reference2 = Rc::clone(&data);  // Count: 3

drop(reference1);  // Count: 2
drop(reference2);  // Count: 1
// data still alive

drop(data);  // Count: 0, memory freed
```

**Key Properties:**
- `Rc::clone()` is O(1) - just increments counter
- Cheap to create multiple owners
- Automatic cleanup when last reference dropped
- **Single-threaded only** - not thread-safe

**Arc\<T\> (Atomic Reference Counted):**

Same as Rc, but uses atomic operations for thread safety:
- Safe to share across threads
- ~2x slower than Rc (atomic operations have overhead)
- Required when Send/Sync is needed

**When to Use:**
- Rc: Single-threaded shared ownership (most cases)
- Arc: Multi-threaded shared ownership (parallel processing)

### 3. Interior Mutability: Mutation Through Shared References

**The Mutability Problem:**

```rust
let data = Rc::new(vec![1, 2, 3]);
data.push(4);  // ❌ Rc<T> only gives &T, not &mut T
```

Rc gives shared references (`&T`), but we often need to mutate shared data.

**RefCell\<T\>: Runtime Borrow Checking**

RefCell moves borrow checking from compile-time to runtime:

```rust
use std::cell::RefCell;

let data = RefCell::new(vec![1, 2, 3]);

// Multiple immutable borrows OK
let r1 = data.borrow();
let r2 = data.borrow();

// Mutable borrow (must be exclusive)
let mut m = data.borrow_mut();
m.push(4);
```

**Borrow Rules (enforced at runtime):**
- Any number of immutable borrows (`borrow()`)
- OR exactly one mutable borrow (`borrow_mut()`)
- Violation → panic! (not compile error)

**The Rc\<RefCell\<T\>\> Pattern:**

Combining Rc + RefCell gives shared mutable state:

```rust
let node = Rc::new(RefCell::new(Node::new("div")));

// Can share
let node2 = node.clone();

// Can mutate through any reference
node2.borrow_mut().add_attribute("class", "container");

// Change visible through all references
assert_eq!(node.borrow().get_attribute("class"), Some("container"));
```

**RwLock\<T\>: Thread-Safe Interior Mutability**

For multi-threaded code, use RwLock instead of RefCell:

```rust
use std::sync::RwLock;

let data = RwLock::new(vec![1, 2, 3]);

// Multiple readers
let r1 = data.read().unwrap();
let r2 = data.read().unwrap();

// Exclusive writer (blocks until all readers done)
let mut w = data.write().unwrap();
w.push(4);
```

**RefCell vs RwLock:**
- RefCell: Single-threaded, panics on conflict, O(1) check
- RwLock: Multi-threaded, blocks on conflict, OS-level locking

### 4. Weak References: Breaking Reference Cycles

**The Cycle Problem:**

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    parent: Option<Rc<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}

// Creates memory leak!
let parent = Rc::new(RefCell::new(Node { parent: None, children: vec![] }));
let child = Rc::new(RefCell::new(Node { parent: Some(parent.clone()), children: vec![] }));
parent.borrow_mut().children.push(child.clone());

// Reference cycle: parent → child → parent → ...
// Strong count never reaches 0 → MEMORY LEAK!
```

**Why Cycles Leak Memory:**

```
parent (Rc: 2)  ← strong reference from child.parent
  ↓ strong
child (Rc: 2)   ← strong reference from parent.children
  ↑ strong

When we drop parent and child variables:
- parent Rc: 2 → 1 (still referenced by child)
- child Rc: 2 → 1 (still referenced by parent)
Both stay alive forever!
```

**Weak\<T\>: Non-Owning References**

Weak references don't increase the strong count:

```rust
use std::rc::{Rc, Weak};

struct Node {
    parent: Option<Weak<RefCell<Node>>>,  // Weak!
    children: Vec<Rc<RefCell<Node>>>,     // Strong
}

let parent = Rc::new(RefCell::new(Node { parent: None, children: vec![] }));
let child = Rc::new(RefCell::new(Node {
    parent: Some(Rc::downgrade(&parent)),  // Create weak reference
    children: vec![]
}));
parent.borrow_mut().children.push(child.clone());

// No cycle!
// parent (Rc: 1, Weak: 1)
// child (Rc: 2, Weak: 0)
```

**Using Weak References:**

```rust
// Downgrade: Rc → Weak
let weak: Weak<T> = Rc::downgrade(&rc);

// Upgrade: Weak → Option<Rc<T>>
if let Some(rc) = weak.upgrade() {
    // Value still alive, use it
} else {
    // Value was deallocated
}
```

**Reference Counting Mechanics:**

Each Rc has two counters:
- **Strong count**: Number of Rc references (owns the data)
- **Weak count**: Number of Weak references (doesn't own the data)

```
Deallocation rules:
1. Data dropped when strong_count == 0 (even if weak_count > 0)
2. Allocation dropped when strong_count == 0 AND weak_count == 0
```

**Pattern for Trees:**

```
Rule: Parent owns children (strong), children reference parent (weak)

Root (Rc: 1)
  ↓ Rc (strong)
Child (Rc: 1, Weak: 0)
  ↑ Weak (non-owning)

When root is dropped:
1. Child's strong count: 1 → 0 (child deallocated)
2. Child's parent.upgrade() returns None
No memory leak!
```

### 5. Event Systems and Callbacks

**DOM Event Flow:**

Real DOM trees need event propagation:

```
User clicks button:
1. Event created with target = button
2. Handlers on button fire (target phase)
3. Event bubbles to parent div
4. Handlers on div fire (bubble phase)
5. Continues to ancestors until stopPropagation() or root
```

**Implementing Events in Rust:**

```rust
type EventHandler = Rc<dyn Fn(&Event)>;

struct Node {
    // ... other fields ...
    event_listeners: HashMap<String, Vec<EventHandler>>,
}
```

**Closure Ownership:**

Event handlers need to capture state:

```rust
let counter = Rc::new(AtomicUsize::new(0));
let c = counter.clone();

add_event_listener(&button, "click", move |_event| {
    c.fetch_add(1, Ordering::SeqCst);  // Captures c
});

// counter still accessible here
println!("Clicks: {}", counter.load(Ordering::SeqCst));
```

### 6. Thread Safety: Send and Sync

**The Send and Sync Traits:**

- **Send**: Type can be transferred to another thread (ownership transfer)
- **Sync**: Type can be shared between threads (&T is Send)

```rust
// Single-threaded types
Rc<T>        // NOT Send, NOT Sync
RefCell<T>   // NOT Send, NOT Sync

// Thread-safe types
Arc<T>       // Send + Sync (if T: Send + Sync)
RwLock<T>    // Send + Sync (if T: Send)
Mutex<T>     // Send + Sync (if T: Send)
```

**Why Rc/RefCell aren't thread-safe:**
- Rc uses regular increment/decrement (not atomic)
- RefCell uses simple integer for borrow checking (race conditions possible)

**Converting to Thread-Safe:**

```rust
// Single-threaded
type NodeRef = Rc<RefCell<Node>>;

// Multi-threaded
type NodeRef = Arc<RwLock<Node>>;
```

### 7. Deadlock Prevention

**The Deadlock Problem:**

```rust
// Bad: Holding lock while calling function that acquires same lock
pub fn pretty_print(node: &NodeRef) -> String {
    let n = node.read().unwrap();  // Acquire lock
    let mut result = format!("<{}>", n.tag());

    for child in &n.children() {
        result.push_str(&pretty_print(child));  // Tries to lock child
        // ❌ DEADLOCK if child tries to access parent!
    }
    result
}
```

**Solution: Clone Before Recursion:**

```rust
pub fn pretty_print(node: &NodeRef) -> String {
    let (tag, children) = {
        let n = node.read().unwrap();
        (n.tag().to_string(), n.children().clone())
    };  // Lock released here!

    let mut result = format!("<{}>", tag);
    for child in children {
        result.push_str(&pretty_print(&child));  // Safe - no lock held
    }
    result
}
```

**Lock Ordering:**

When acquiring multiple locks, always do so in a consistent order:

```rust
// Bad: Inconsistent lock order
fn swap(a: &NodeRef, b: &NodeRef) {
    let mut a_lock = a.write().unwrap();
    let mut b_lock = b.write().unwrap();  // Could deadlock!
}

// Good: Consistent order
fn swap(a: &NodeRef, b: &NodeRef) {
    let (first, second) = if Rc::as_ptr(a) < Rc::as_ptr(b) {
        (a, b)
    } else {
        (b, a)
    };
    let mut first_lock = first.write().unwrap();
    let mut second_lock = second.write().unwrap();
}
```

### 8. Performance Characteristics

**Memory Overhead:**

| Type | Size | Overhead | Notes |
|------|------|----------|-------|
| `Box<T>` | 8 bytes | None | Just a pointer |
| `Rc<T>` | 8 bytes | 16 bytes (2 counters) | Strong + weak counts |
| `Arc<T>` | 8 bytes | 16 bytes (atomic) | Atomic operations |
| `RefCell<T>` | size_of(T) + 8 | 8 bytes | Borrow state counter |
| `RwLock<T>` | size_of(T) + ~40 | ~40 bytes | OS lock structure |

**Operation Costs:**

| Operation | Box | Rc | Arc | RefCell | RwLock |
|-----------|-----|----|----|---------|--------|
| Clone | Deep copy | O(1) | O(1) atomic | N/A | N/A |
| Deref | O(1) | O(1) | O(1) | O(1) check | Lock wait |
| Modify | Direct | Need RefCell | Need Mutex/RwLock | O(1) check | Lock wait |

**When to Use Each:**

```
Box<T>           → Unique ownership, heap allocation
Rc<T>            → Shared ownership, single-threaded
Arc<T>           → Shared ownership, multi-threaded
RefCell<T>       → Interior mutability, single-threaded
RwLock<T>        → Interior mutability, multi-threaded
Rc<RefCell<T>>   → Shared mutable state, single-threaded (DOM trees)
Arc<RwLock<T>>   → Shared mutable state, multi-threaded (concurrent DOM)
Weak<T>          → Break cycles, non-owning references
```

### 9. Real-World DOM Implementation Patterns

**Browser Engine Architecture:**

Modern web browsers use similar patterns:

```rust
// Simplified browser DOM node
pub struct DOMNode {
    element_data: ElementData,      // Tag, attributes, id, class
    layout_box: Option<LayoutBox>,  // Computed styles, position, size
    parent: Option<Weak<Node>>,     // Weak reference to parent
    children: Vec<Rc<Node>>,        // Strong references to children
    event_handlers: HashMap<String, Vec<EventHandler>>,
}
```

**Separation of Concerns:**

1. **DOM Tree**: Logical structure (this project)
2. **Render Tree**: Visual elements (derived from DOM)
3. **Layout Tree**: Computed positions (geometry)
4. **Paint**: Actual pixels (GPU)

**Trade-offs in Production Systems:**

Chrome/Firefox optimizations:
- **Arena allocation**: Allocate nodes in contiguous blocks for cache locality
- **Generational references**: Most DOM nodes are temporary (optimize for quick cleanup)
- **Shadow DOM**: Isolated subtrees with encapsulated styles
- **Virtual DOM**: React/Vue pattern - diff before applying changes

### 10. Query Selector Implementation Strategies

**CSS Selector Types:**

```rust
// Simple selectors
"div"           → Tag name
"#main"         → ID attribute
".container"    → Class attribute

// Combinators
"div p"         → Descendant (p inside div)
"div > p"       → Direct child
"div + p"       → Adjacent sibling
"div ~ p"       → General sibling

// Pseudo-classes
"p:first-child" → First child of parent
"p:nth-child(2)"→ Second child
"a:hover"       → State-based (requires event system)
```

**Matching Algorithm:**

```rust
fn matches_selector(node: &Node, selector: &str) -> bool {
    match selector.chars().next() {
        Some('#') => {
            // ID selector: #main
            node.get_attribute("id") == Some(&selector[1..])
        }
        Some('.') => {
            // Class selector: .btn
            node.get_attribute("class")
                .map(|classes| classes.split_whitespace().any(|c| c == &selector[1..]))
                .unwrap_or(false)
        }
        _ => {
            // Tag selector: div
            node.tag() == selector
        }
    }
}
```

**Complex Selectors:**

For "div .btn" (btn with class inside div):
1. Find all div elements
2. For each div, traverse descendants
3. Filter for elements with class="btn"
4. Return matches

---

## Connection to This Project

This project progressively builds a production-quality DOM tree implementation, with each milestone introducing essential smart pointer patterns and solving real-world problems.

### Milestone Progression and Learning Path

| Milestone | Smart Pointers | Capabilities | Limitations | Real-World Equivalent |
|-----------|---------------|--------------|-------------|----------------------|
| 1. Box | `Box<Node>` | Basic tree, owned children | No parent navigation, immutable sharing | Static XML parser |
| 2. Rc | `Rc<Node>` | Shared nodes, cheap cloning | Still immutable, no cycles | Read-only DOM |
| 3. RefCell | `Rc<RefCell<Node>>` | Mutable shared state | Memory leaks with cycles | Basic browser DOM |
| 4. Weak | `Weak<RefCell<Node>>` | Parent pointers, no leaks | Single-threaded only | Chrome/Firefox DOM |
| 5. Events | Event handlers | Dynamic behavior | Single-threaded | Full browser DOM |
| 6. Arc | `Arc<RwLock<Node>>` | Thread-safe, parallel | Locking overhead | Parallel rendering engine |

### Why Each Pattern Matters

**Milestone 1 (Box): Understanding the Problem**

Establishes the baseline:
- Clear ownership semantics
- Simple and fast
- Reveals limitations that necessitate advanced patterns

**Limitations that force evolution:**
- Can't navigate to parent (need shared references)
- Can't share nodes (need reference counting)
- Can't modify without `&mut` to root (need interior mutability)

**Milestone 2 (Rc): Introducing Shared Ownership**

Solves: Multiple references to same node
- GUI framework: Same widget in multiple containers
- Web browser: getElementById() returns references
- Game engine: Entity referenced by multiple systems

**Real-world impact:**
- Enables shared subtrees (common footer across pages)
- Allows keeping references for later use
- Makes tree navigation possible

**Milestone 3 (RefCell): Enabling Mutation**

Solves: Modifying shared data
- Update button text after user input
- Change styles on hover
- Add/remove children dynamically

**The `Rc<RefCell<T>>` pattern is ubiquitous in Rust:**
- Used in: egui, gtk-rs, iced (GUI frameworks)
- Used in: quick-xml, roxmltree (XML parsers)
- Used in: game scene graphs, configuration trees

**Milestone 4 (Weak): Preventing Memory Leaks**

Solves: Reference cycles
- Parent → child → parent creates cycle
- Without Weak: Memory leak (nodes never freed)
- With Weak: Automatic cleanup when tree dismantled

**Critical for production:**
- Long-running applications (browsers run for days)
- Dynamic DOM (pages constantly added/removed)
- Server applications (memory leaks → crashes)

**Pattern used everywhere:**
- Every tree in Rust with parent pointers
- Observer patterns (weak listeners)
- Cache invalidation (weak references to cached data)

**Milestone 5 (Events): Dynamic Behavior**

Solves: Interactive systems
- User clicks button → form submits
- Mouse over element → tooltip appears
- Child event bubbles to parent handlers

**Demonstrates:**
- Closure capture with Rc
- Event propagation algorithms
- Query selectors (DOM API)

**Real-world complexity:**
- 10-100 event types in real browsers
- Event capture vs bubble phase
- preventDefault(), stopPropagation()
- Touch events, keyboard events, custom events

**Milestone 6 (Arc/RwLock): Parallelism**

Solves: Multi-threaded access
- Layout on separate thread
- Parallel style computation
- Background parsing

**Performance implications:**

```
Single-threaded (Rc<RefCell<T>>):
- Clone: ~1ns
- Read: ~1ns
- Write: ~1ns

Multi-threaded (Arc<RwLock<T>>):
- Clone: ~5ns (atomic operations)
- Read: ~50ns (lock acquisition)
- Write: ~50ns (exclusive lock)

But enables:
- 4-16 cores working in parallel
- 10-100x speedup for CPU-bound tasks
```

### Performance Journey

Understanding the trade-offs at each stage:

| Pattern | Memory/Node | Clone Cost | Mutation | Thread-Safe | Use Case |
|---------|------------|------------|----------|-------------|----------|
| `Box<Node>` | +0 bytes | Deep copy | Direct | No | Simple trees |
| `Rc<Node>` | +16 bytes | ~1ns | Immutable | No | Read-only shared |
| `Rc<RefCell<Node>>` | +24 bytes | ~1ns | Runtime check | No | **Most DOM trees** |
| `Arc<RwLock<Node>>` | +56 bytes | ~5ns | Lock wait | Yes | Parallel processing |

**The 95% case:** `Rc<RefCell<Node>>` is the sweet spot for most applications.

### Real-World Impact Examples

**Example 1: Web Browser**

```rust
// User clicks "Like" button
let button = query_selector(&document, "#like-button").unwrap();

// Update button state (interior mutability)
button.borrow_mut().set_attribute("aria-pressed", "true");

// Fire event (bubbles to analytics tracker)
dispatch_event(&button, "click");
// → button.onclick() executes
// → parent div.onclick() executes (bubbling)
// → document.onclick() logs analytics

// Update parent counter (weak parent reference)
if let Some(parent) = button.borrow().parent() {
    let count: u32 = parent.borrow()
        .get_attribute("data-like-count")
        .unwrap_or("0")
        .parse()
        .unwrap();
    parent.borrow_mut().set_attribute("data-like-count", &(count + 1).to_string());
}
```

**Example 2: GUI Framework**

```rust
// Build settings dialog
let dialog = create_element("dialog");
let form = create_element("form");
let button = create_element("button");

// Nest elements
add_child(&dialog, &form);
add_child(&form, &button);

// Add to multiple places (shared ownership)
let sidebar = create_element("div");
sidebar.borrow_mut().children.push(button.clone());  // Button in 2 places!

// Close dialog when button clicked (event bubbling)
add_event_listener(&button, "click", move |e| {
    e.stop_propagation();
    if let Some(form) = button.borrow().parent() {
        if let Some(dialog) = form.borrow().parent() {
            dialog.borrow_mut().set_attribute("open", "false");
        }
    }
});
```

**Example 3: Parallel Web Scraper**

```rust
// Thread 1: Parse HTML into DOM
let doc = parse_html(html_string);  // Arc<RwLock<Node>>

// Thread 2: Extract all links (parallel read)
let doc1 = Arc::clone(&doc);
thread::spawn(move || {
    let links = query_selector_all(&doc1, "a");
    // Process links...
});

// Thread 3: Extract metadata (parallel read)
let doc2 = Arc::clone(&doc);
thread::spawn(move || {
    if let Some(title) = query_selector(&doc2, "title") {
        println!("{}", title.read().unwrap().text_content());
    }
});

// Thread 4: Modify DOM (exclusive write)
let doc3 = Arc::clone(&doc);
thread::spawn(move || {
    let scripts = query_selector_all(&doc3, "script");
    for script in scripts {
        remove_from_parent(&script);  // Strip scripts
    }
});
```

### Architectural Insights

**Pattern 1: Ownership Hierarchy**

```
Root owns tree (strong references down)
  ↓ Rc
Children reference parents (weak references up)
  ↑ Weak

This ensures:
- Dropping root frees entire tree
- No reference cycles
- Children can safely navigate to parents
```

**Pattern 2: Smart Pointer Composition**

```rust
// Each layer adds a capability:
T                    → Base type
Rc<T>                → + Shared ownership
Rc<RefCell<T>>       → + Interior mutability
Weak<RefCell<T>>     → + Cycle breaking
Arc<RwLock<T>>       → + Thread safety
```

**Pattern 3: Lock Minimization**

```rust
// Bad: Hold lock during entire operation
let node = tree.write().unwrap();
expensive_computation(&node);  // Lock held!

// Good: Clone data, release lock
let data = {
    let node = tree.read().unwrap();
    node.clone_relevant_data()
};  // Lock released
expensive_computation(&data);
```

### Skills Transferred to Other Domains

After completing this project, you'll understand patterns used in:

1. **GUI Frameworks** (egui, iced, druid)
   - Widget trees with parent/child relationships
   - Event propagation
   - Shared state management

2. **Game Engines** (Bevy ECS, specs)
   - Entity hierarchies
   - Component systems
   - Scene graphs

3. **Parsers** (quick-xml, syn, pest)
   - AST nodes
   - Visitor patterns
   - Tree transformations

4. **Databases** (SQLite FFI, B-trees)
   - Index structures
   - Reference counting for cached nodes
   - Concurrent access control

5. **Operating Systems** (file systems)
   - Directory trees
   - Inode reference counting
   - Process parent/child relationships

### Key Takeaways

1. **There is no single "right" smart pointer** - choose based on requirements:
   - Need unique ownership? → `Box`
   - Need shared ownership? → `Rc`/`Arc`
   - Need mutation through shared ref? → `RefCell`/`RwLock`
   - Have cycles? → `Weak`

2. **Composition is key**: `Rc<RefCell<T>>` combines two patterns to solve DOM problem

3. **Memory leaks are possible in safe Rust**: Reference cycles require `Weak` to break

4. **Thread safety isn't free**: Arc/RwLock add ~5-10x overhead vs Rc/RefCell

5. **Lock discipline prevents deadlocks**: Clone before recursion, consistent lock ordering

6. **Pattern matching real-world systems**: This project mirrors actual browser implementations

This project teaches you to think in terms of ownership, sharing, mutability, and safety - the core skills needed for systems programming in Rust.

---

## Milestone 1: Basic Tree with Box and Owned Children

**Goal:** Create a simple tree where each node owns its children using `Box`.

### Introduction

We start with the simplest possible tree: each node owns its children through `Box<Node>`. This gives us:
- Clear ownership (parent owns children)
- Simple to implement
- No reference counting overhead

**Limitations we'll address later:**
- Can't navigate from child to parent
- Can't share nodes between multiple parents
- Can't modify nodes without mutable reference to root
- Must pass `&mut self` for all modifications

### Architecture

```rust
pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<Box<Node>>,
}
```

**Key Structures:**

- **`Node`**: Tree node with tag name, attributes, and owned children
  - `tag`: Element name like "div", "p", "span"
  - `attributes`: Key-value pairs like `{"id": "main", "class": "container"}`
  - `children`: Owned child nodes

**Key Functions:**

- `Node::new(tag)`: Create new node
- `add_child(&mut self, child)`: Append child node
- `find_by_id(&self, id) -> Option<&Node>`: Depth-first search
- `pretty_print(&self)`: Display tree structure

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = Node::new("div");
        assert_eq!(node.tag(), "div");
        assert_eq!(node.children().len(), 0);
    }

    #[test]
    fn test_add_children() {
        let mut root = Node::new("html");
        let mut body = Node::new("body");
        let p = Node::new("p");

        body.add_child(Box::new(p));
        root.add_child(Box::new(body));

        assert_eq!(root.children().len(), 1);
        assert_eq!(root.children()[0].children().len(), 1);
    }

    #[test]
    fn test_attributes() {
        let mut node = Node::new("div");
        node.set_attribute("id", "main");
        node.set_attribute("class", "container");

        assert_eq!(node.get_attribute("id"), Some("main"));
        assert_eq!(node.get_attribute("class"), Some("container"));
    }

    #[test]
    fn test_find_by_id() {
        let mut root = Node::new("div");
        let mut child1 = Node::new("p");
        child1.set_attribute("id", "intro");

        let mut child2 = Node::new("p");
        child2.set_attribute("id", "content");

        root.add_child(Box::new(child1));
        root.add_child(Box::new(child2));

        let found = root.find_by_id("content");
        assert!(found.is_some());
        assert_eq!(found.unwrap().tag(), "p");
    }

    #[test]
    fn test_depth_first_traversal() {
        let mut root = Node::new("div");
        let mut ul = Node::new("ul");
        ul.add_child(Box::new(Node::new("li")));
        ul.add_child(Box::new(Node::new("li")));
        root.add_child(Box::new(ul));

        let tags: Vec<&str> = root.traverse().map(|n| n.tag()).collect();
        assert_eq!(tags, vec!["div", "ul", "li", "li"]);
    }

    #[test]
    fn test_pretty_print() {
        let mut root = Node::new("html");
        let mut body = Node::new("body");
        let p = Node::new("p");
        body.add_child(Box::new(p));
        root.add_child(Box::new(body));

        let output = root.pretty_print();
        assert!(output.contains("<html>"));
        assert!(output.contains("  <body>"));
        assert!(output.contains("    <p>"));
    }
}
```

### Starter Code

```rust
use std::collections::HashMap;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<Box<Node>>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        todo!("Create new node with tag name")
        // Hint: Initialize empty attributes and children
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        todo!("Add key-value pair to attributes")
    }

    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        todo!("Return attribute value if exists")
    }

    pub fn add_child(&mut self, child: Box<Node>) {
        todo!("Append child to children vec")
    }

    pub fn children(&self) -> &[Box<Node>] {
        &self.children
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Node> {
        todo!("
        Implement depth-first search:
        1. Check if current node has id attribute matching target
        2. If yes, return Some(self)
        3. Otherwise, recursively search each child
        4. Return first match found, or None
        ")
    }

    pub fn traverse(&self) -> NodeIterator {
        todo!("Return iterator for depth-first traversal")
        // Hint: Store Vec<&Node> and process recursively
    }

    pub fn pretty_print(&self) -> String {
        self.pretty_print_with_indent(0)
    }

    fn pretty_print_with_indent(&self, indent: usize) -> String {
        todo!("
        Format tree with indentation:
        1. Create indent string: '  ' * indent
        2. Format opening tag with attributes
        3. Recursively format children with indent + 1
        4. Format closing tag
        Example output:
        <div id='main'>
          <p>
          </p>
        </div>
        ")
    }
}

// Iterator for tree traversal
pub struct NodeIterator<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("
        Pop node from stack, push its children (right to left),
        return the node
        ")
    }
}
```

---

## Milestone 2: Shared Ownership with Rc

**Goal:** Use `Rc<Node>` to allow multiple references to the same node.

### Introduction

**Why Milestone 1 Isn't Enough:**

The `Box<Node>` approach has fundamental limitations:
1. **Single ownership**: Each node can only have one parent
2. **Can't share subtrees**: Copying a subtree requires deep cloning
3. **No references**: Can't keep references to nodes for later use

**Real-world scenario:** In a GUI framework, the same button widget might appear in:
- The visual tree (for rendering)
- The focus chain (for tab navigation)
- An event handler list (for click events)

**Solution:** Use `Rc<Node>` for shared ownership with reference counting.

**Performance Impact:**
- **Memory**: +16 bytes per node (strong/weak counts)
- **Speed**: Clone is O(1) (just increment counter)
- **Flexibility**: Multiple owners, shared subtrees

### Architecture

```rust
use std::rc::Rc;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<Rc<Node>>,
}
```

**Key Changes:**
- `Vec<Box<Node>>` → `Vec<Rc<Node>>`: Children are reference counted
- `add_child(Rc<Node>)`: Accept already-wrapped nodes
- `clone()`: Cheap - just increments reference count

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_shared_ownership() {
        let child = Rc::new(Node::new("shared"));

        let mut parent1 = Node::new("div");
        let mut parent2 = Node::new("section");

        parent1.add_child(child.clone());
        parent2.add_child(child.clone());

        // Both parents share the same child
        assert_eq!(Rc::strong_count(&child), 3); // child + 2 parents
    }

    #[test]
    fn test_reference_counting() {
        let node = Rc::new(Node::new("test"));
        assert_eq!(Rc::strong_count(&node), 1);

        let node2 = node.clone();
        assert_eq!(Rc::strong_count(&node), 2);

        drop(node2);
        assert_eq!(Rc::strong_count(&node), 1);
    }

    #[test]
    fn test_shared_subtree() {
        let footer = Rc::new({
            let mut f = Node::new("footer");
            f.set_attribute("class", "page-footer");
            f
        });

        let mut page1 = Node::new("div");
        let mut page2 = Node::new("div");

        page1.add_child(footer.clone());
        page2.add_child(footer.clone());

        // Same footer instance in both pages
        assert_eq!(Rc::strong_count(&footer), 3);
    }

    #[test]
    fn test_find_shared_node() {
        let target = Rc::new({
            let mut n = Node::new("p");
            n.set_attribute("id", "target");
            n
        });

        let mut root = Node::new("div");
        root.add_child(target.clone());

        let found = root.find_by_id("target");
        assert!(found.is_some());

        // Can compare Rc pointers
        assert!(Rc::ptr_eq(found.unwrap(), &target));
    }

    #[test]
    fn test_memory_cleanup() {
        let node = Rc::new(Node::new("test"));
        let weak_ref = Rc::downgrade(&node);

        assert_eq!(weak_ref.strong_count(), 1);

        drop(node);

        // Node should be deallocated
        assert_eq!(weak_ref.strong_count(), 0);
        assert!(weak_ref.upgrade().is_none());
    }
}
```

### Starter Code

```rust
use std::rc::Rc;
use std::collections::HashMap;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<Rc<Node>>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        todo!("Same as Milestone 1, but children is Vec<Rc<Node>>")
    }

    pub fn add_child(&mut self, child: Rc<Node>) {
        todo!("Push Rc<Node> to children")
    }

    pub fn find_by_id(&self, id: &str) -> Option<Rc<Node>> {
        todo!("
        Similar to Milestone 1, but return Rc<Node>:
        1. Check current node's id attribute
        2. If match, return Some(Rc::clone(self))...
           PROBLEM: We don't have Rc<Self> here!

        This reveals a limitation: we need to change the API
        to work with Rc from the start.

        Better approach:
        - Make find_by_id a free function: find_by_id(root: &Rc<Node>, id: &str)
        - Or return &Node instead of Rc<Node>
        ")
    }

    pub fn strong_count(&self) -> usize {
        todo!("
        PROBLEM: We can't get strong_count from inside Node!
        This method only makes sense when called on Rc<Node>.

        This teaches an important lesson: some operations only make
        sense on the smart pointer, not the inner type.
        ")
    }
}

// Helper function for finding nodes
pub fn find_by_id(root: &Rc<Node>, id: &str) -> Option<Rc<Node>> {
    todo!("
    Now we can clone the Rc when we find it:
    1. Check if root.get_attribute('id') == Some(id)
    2. If yes, return Some(Rc::clone(root))
    3. Otherwise, search children
    ")
}
```

---

## Milestone 3: Interior Mutability with RefCell

**Goal:** Enable mutation through shared references using `Rc<RefCell<Node>>`.

### Introduction

**Why Milestone 2 Isn't Enough:**

`Rc<Node>` gives us shared ownership but has a critical problem:
1. **Immutable only**: `Rc::clone()` gives `&Node`, not `&mut Node`
2. **Can't modify**: Can't add children or change attributes after creation
3. **Awkward API**: Must reconstruct entire tree to make changes

**Real-world scenario:** A GUI button that needs to:
- Update its text label when clicked
- Change background color on hover
- Add child elements dynamically

Without interior mutability, we'd need `&mut` to the root just to change a leaf node!

**Solution:** Use `Rc<RefCell<Node>>` for shared mutable state.

**Performance Impact:**
- **Runtime checking**: `borrow()` and `borrow_mut()` check at runtime
- **Panic risk**: Calling `borrow_mut()` twice panics
- **No overhead when not borrowed**: Zero cost until actually used

### Architecture

```rust
use std::rc::Rc;
use std::cell::RefCell;

pub type NodeRef = Rc<RefCell<Node>>;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<NodeRef>,
}
```

**Key Concepts:**
- `RefCell<T>`: Provides interior mutability (mutation through `&`)
- `borrow()`: Get `Ref<T>` (shared reference) - can have many
- `borrow_mut()`: Get `RefMut<T>` (exclusive reference) - only one at a time
- **Runtime checking**: Violating borrow rules causes panic (not compile error)

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interior_mutability() {
        let node = NodeRef::new(RefCell::new(Node::new("div")));

        // Can mutate through shared reference
        node.borrow_mut().set_attribute("class", "container");

        assert_eq!(node.borrow().get_attribute("class"), Some("container"));
    }

    #[test]
    fn test_add_child_after_sharing() {
        let parent = NodeRef::new(RefCell::new(Node::new("div")));
        let parent_clone = parent.clone();

        // Can modify through clone
        let child = NodeRef::new(RefCell::new(Node::new("p")));
        parent_clone.borrow_mut().add_child(child);

        // Change visible through original reference
        assert_eq!(parent.borrow().children().len(), 1);
    }

    #[test]
    fn test_multiple_borrows() {
        let node = NodeRef::new(RefCell::new(Node::new("div")));

        // Multiple immutable borrows OK
        let borrow1 = node.borrow();
        let borrow2 = node.borrow();
        assert_eq!(borrow1.tag(), "div");
        assert_eq!(borrow2.tag(), "div");
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn test_borrow_conflict() {
        let node = NodeRef::new(RefCell::new(Node::new("div")));

        let _borrow = node.borrow();
        let _mut_borrow = node.borrow_mut(); // Panics!
    }

    #[test]
    fn test_modify_shared_subtree() {
        let shared = NodeRef::new(RefCell::new(Node::new("footer")));

        let mut page1 = Node::new("div");
        let mut page2 = Node::new("div");

        page1.add_child(shared.clone());
        page2.add_child(shared.clone());

        // Modify through one reference
        shared.borrow_mut().set_attribute("version", "1.0");

        // Visible through all references
        assert_eq!(
            page1.children()[0].borrow().get_attribute("version"),
            Some("1.0")
        );
    }

    #[test]
    fn test_builder_pattern() {
        let node = NodeRef::new(RefCell::new(Node::new("div")));

        {
            let mut n = node.borrow_mut();
            n.set_attribute("id", "main");
            n.set_attribute("class", "container");
        } // Drop borrow

        let child = NodeRef::new(RefCell::new(Node::new("p")));
        node.borrow_mut().add_child(child);

        assert_eq!(node.borrow().children().len(), 1);
    }
}
```

### Starter Code

```rust
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub type NodeRef = Rc<RefCell<Node>>;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    children: Vec<NodeRef>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        Node {
            tag: tag.into(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        todo!("Insert into attributes map")
    }

    pub fn get_attribute(&self, key: &str) -> Option<&str> {
        todo!("Get from attributes map")
    }

    pub fn add_child(&mut self, child: NodeRef) {
        todo!("Push child to children vec")
    }

    pub fn children(&self) -> &[NodeRef] {
        &self.children
    }
}

// Helper to create and configure nodes
pub fn create_element(tag: &str) -> NodeRef {
    todo!("Return Rc::new(RefCell::new(Node::new(tag)))")
}

// Find by ID (works with RefCell)
pub fn find_by_id(root: &NodeRef, id: &str) -> Option<NodeRef> {
    todo!("
    1. Borrow root: let node = root.borrow();
    2. Check if node.get_attribute('id') matches
    3. If yes, return Some(root.clone())
    4. Otherwise, search children

    Important: Drop borrow before recursive call!
    ")
}

// Pretty print with RefCell
pub fn pretty_print(node: &NodeRef) -> String {
    fn print_indent(node: &NodeRef, indent: usize) -> String {
        todo!("
        1. Borrow node
        2. Format with indentation
        3. Recursively print children
        4. Don't forget to drop borrow before recursing!
        ")
    }
    print_indent(node, 0)
}
```

---

## Milestone 4: Parent Pointers with Weak References

**Goal:** Add parent pointers using `Weak<RefCell<Node>>` to enable upward navigation without memory leaks.

### Introduction

**Why Milestone 3 Isn't Enough:**

Currently we can only navigate downward (parent → children). Many DOM operations need parent access:
1. **Event bubbling**: Click on button → bubbles to div → bubbles to body
2. **Style inheritance**: Child inherits font from parent
3. **Remove from parent**: `node.remove()` needs to find parent
4. **Sibling access**: `node.next_sibling()` goes through parent

**The Cycle Problem:**

```rust
// Attempt 1: Use Rc for parent (MEMORY LEAK!)
struct Node {
    parent: Option<Rc<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
}

// Creates cycle: parent → child → parent → child → ...
// Reference count never reaches 0 → MEMORY LEAK!
```

**Solution:** Use `Weak<RefCell<Node>>` for parent pointers.

**How Weak Works:**
- `Weak<T>` doesn't increase strong count
- Child dropped when no strong references exist (only weak ones OK)
- `weak.upgrade()` returns `Option<Rc<T>>` (None if deallocated)
- Breaks cycles automatically

### Architecture

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub type NodeRef = Rc<RefCell<Node>>;
pub type WeakNodeRef = Weak<RefCell<Node>>;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    parent: Option<WeakNodeRef>,  // Weak to break cycles
    children: Vec<NodeRef>,        // Strong ownership
}
```

**Memory Safety:**
```
Root (Rc: 1)
  ↓ Rc
Child (Rc: 1, Weak: 0)
  ↑ Weak (doesn't prevent deallocation)
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parent_child_relationship() {
        let parent = create_element("div");
        let child = create_element("p");

        add_child(&parent, &child);

        // Child knows parent
        assert!(child.borrow().parent().is_some());

        // Parent is correct
        let parent_ref = child.borrow().parent().unwrap();
        assert_eq!(parent_ref.borrow().tag(), "div");
    }

    #[test]
    fn test_no_memory_leak() {
        let parent = create_element("div");
        let child = create_element("p");

        add_child(&parent, &child);

        let weak_parent = Rc::downgrade(&parent);
        let weak_child = Rc::downgrade(&child);

        // Both alive
        assert!(weak_parent.upgrade().is_some());
        assert!(weak_child.upgrade().is_some());

        drop(parent); // Drop only strong reference to parent

        // Parent deallocated (child's weak ref doesn't keep it alive)
        assert!(weak_parent.upgrade().is_none());

        // Child still alive (we hold a strong reference)
        assert!(weak_child.upgrade().is_some());
    }

    #[test]
    fn test_orphan_node() {
        let child = create_element("p");

        // Child without parent
        assert!(child.borrow().parent().is_none());
    }

    #[test]
    fn test_reparenting() {
        let parent1 = create_element("div");
        let parent2 = create_element("section");
        let child = create_element("p");

        add_child(&parent1, &child);
        assert_eq!(child.borrow().parent().unwrap().borrow().tag(), "div");

        // Move to new parent
        remove_child(&parent1, &child);
        add_child(&parent2, &child);

        assert_eq!(child.borrow().parent().unwrap().borrow().tag(), "section");
    }

    #[test]
    fn test_ancestors() {
        let root = create_element("html");
        let body = create_element("body");
        let div = create_element("div");
        let p = create_element("p");

        add_child(&root, &body);
        add_child(&body, &div);
        add_child(&div, &p);

        let ancestors: Vec<String> = get_ancestors(&p)
            .iter()
            .map(|n| n.borrow().tag().to_string())
            .collect();

        assert_eq!(ancestors, vec!["div", "body", "html"]);
    }

    #[test]
    fn test_remove_from_parent() {
        let parent = create_element("div");
        let child = create_element("p");

        add_child(&parent, &child);
        assert_eq!(parent.borrow().children().len(), 1);

        remove_from_parent(&child);

        assert_eq!(parent.borrow().children().len(), 0);
        assert!(child.borrow().parent().is_none());
    }
}
```

### Starter Code

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::collections::HashMap;

pub type NodeRef = Rc<RefCell<Node>>;
pub type WeakNodeRef = Weak<RefCell<Node>>;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    parent: Option<WeakNodeRef>,
    children: Vec<NodeRef>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        Node {
            tag: tag.into(),
            attributes: HashMap::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn parent(&self) -> Option<NodeRef> {
        todo!("
        Upgrade weak reference:
        self.parent.as_ref()?.upgrade()

        Returns None if:
        - No parent set
        - Parent was deallocated
        ")
    }

    pub fn set_parent(&mut self, parent: WeakNodeRef) {
        todo!("Store weak reference to parent")
    }

    pub fn clear_parent(&mut self) {
        todo!("Set parent to None")
    }

    // ... other methods from Milestone 3 ...
}

pub fn create_element(tag: &str) -> NodeRef {
    Rc::new(RefCell::new(Node::new(tag)))
}

pub fn add_child(parent: &NodeRef, child: &NodeRef) {
    todo!("
    1. Add child to parent's children vec
    2. Set child's parent to Weak reference to parent:
       child.borrow_mut().set_parent(Rc::downgrade(parent))
    ")
}

pub fn remove_child(parent: &NodeRef, child: &NodeRef) {
    todo!("
    1. Remove child from parent's children vec
       (use Vec::retain or find index + remove)
    2. Clear child's parent reference
    ")
}

pub fn remove_from_parent(child: &NodeRef) {
    todo!("
    1. Get parent via child.borrow().parent()
    2. If Some(parent), call remove_child(parent, child)
    ")
}

pub fn get_ancestors(node: &NodeRef) -> Vec<NodeRef> {
    todo!("
    Build vec of ancestors from node to root:
    1. Start with current node
    2. While node.parent() is Some:
       - Add parent to vec
       - Move to parent
    3. Return vec
    ")
}

pub fn lowest_common_ancestor(node1: &NodeRef, node2: &NodeRef) -> Option<NodeRef> {
    todo!("
    Find first common ancestor:
    1. Get all ancestors of node1 into HashSet
    2. Walk up from node2 checking if ancestor in set
    3. Return first match
    ")
}
```

---

## Milestone 5: Event Bubbling and Query Selectors

**Goal:** Implement DOM-like event bubbling and CSS-style selectors.

### Introduction

**Why Milestone 4 Isn't Enough:**

We have a navigable tree, but it's not very useful yet. Real DOM trees support:
1. **Event bubbling**: Events propagate from target → ancestors
2. **Query selectors**: Find nodes by tag, class, or complex criteria
3. **Event handlers**: Attach callbacks to nodes
4. **Event capture**: Parent can intercept child events

**Real-world scenario:** Clicking a button in a form:
```html
<form id="login">           <!-- onsubmit handler -->
  <div class="field">       <!-- No handler -->
    <button id="submit">    <!-- onclick handler -->
      Click me
    </button>
  </div>
</form>
```

Event flow: button.click() → div → form.onsubmit()

**New Capabilities:**
- `node.dispatch_event("click")` bubbles to ancestors
- `node.query_selector(".field button")` finds descendants
- `node.add_event_listener("click", callback)`

### Architecture

```rust
pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    parent: Option<WeakNodeRef>,
    children: Vec<NodeRef>,
    event_listeners: HashMap<String, Vec<EventHandler>>,
}

type EventHandler = Rc<dyn Fn(&Event)>;

pub struct Event {
    event_type: String,
    target: WeakNodeRef,
    current_target: WeakNodeRef,
    bubbles: bool,
    stop_propagation: RefCell<bool>,
}
```

**Event Flow:**
1. **Capture phase** (optional): root → target
2. **Target phase**: Handlers on target fire
3. **Bubble phase**: target → ancestors

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_listener() {
        let node = create_element("button");
        let counter = Rc::new(AtomicUsize::new(0));

        let c = counter.clone();
        add_event_listener(&node, "click", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        dispatch_event(&node, "click");

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_bubbling() {
        let parent = create_element("div");
        let child = create_element("button");
        add_child(&parent, &child);

        let parent_counter = Rc::new(AtomicUsize::new(0));
        let child_counter = Rc::new(AtomicUsize::new(0));

        let pc = parent_counter.clone();
        add_event_listener(&parent, "click", move |_| {
            pc.fetch_add(1, Ordering::SeqCst);
        });

        let cc = child_counter.clone();
        add_event_listener(&child, "click", move |_| {
            cc.fetch_add(1, Ordering::SeqCst);
        });

        dispatch_event(&child, "click");

        // Both fire (bubbles to parent)
        assert_eq!(child_counter.load(Ordering::SeqCst), 1);
        assert_eq!(parent_counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_stop_propagation() {
        let parent = create_element("div");
        let child = create_element("button");
        add_child(&parent, &child);

        let parent_fired = Rc::new(AtomicUsize::new(0));
        let pf = parent_fired.clone();

        add_event_listener(&parent, "click", move |_| {
            pf.fetch_add(1, Ordering::SeqCst);
        });

        add_event_listener(&child, "click", move |event| {
            event.stop_propagation();
        });

        dispatch_event(&child, "click");

        // Parent shouldn't fire
        assert_eq!(parent_fired.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_query_selector_by_tag() {
        let root = create_element("div");
        let p1 = create_element("p");
        let p2 = create_element("p");
        let span = create_element("span");

        add_child(&root, &p1);
        add_child(&root, &p2);
        add_child(&root, &span);

        let results = query_selector_all(&root, "p");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_selector_by_id() {
        let root = create_element("div");
        let target = create_element("button");
        target.borrow_mut().set_attribute("id", "submit");

        add_child(&root, &target);

        let result = query_selector(&root, "#submit");
        assert!(result.is_some());
        assert_eq!(result.unwrap().borrow().tag(), "button");
    }

    #[test]
    fn test_query_selector_by_class() {
        let root = create_element("div");

        let btn1 = create_element("button");
        btn1.borrow_mut().set_attribute("class", "primary");

        let btn2 = create_element("button");
        btn2.borrow_mut().set_attribute("class", "secondary");

        add_child(&root, &btn1);
        add_child(&root, &btn2);

        let results = query_selector_all(&root, ".primary");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_complex_selector() {
        let root = create_element("div");
        let form = create_element("form");
        let button = create_element("button");
        button.borrow_mut().set_attribute("class", "submit");

        add_child(&root, &form);
        add_child(&form, &button);

        // "form .submit" - button with class "submit" inside form
        let result = query_selector(&root, "form .submit");
        assert!(result.is_some());
    }
}
```

### Starter Code

```rust
use std::rc::Rc;
use std::cell::RefCell;

type EventHandler = Rc<dyn Fn(&Event)>;

pub struct Node {
    // ... previous fields ...
    event_listeners: RefCell<HashMap<String, Vec<EventHandler>>>,
}

pub struct Event {
    event_type: String,
    target: WeakNodeRef,
    current_target: WeakNodeRef,
    bubbles: bool,
    stop_propagation: RefCell<bool>,
}

impl Event {
    pub fn new(event_type: String, target: WeakNodeRef) -> Self {
        todo!("Initialize event with target, bubbles=true, stop_propagation=false")
    }

    pub fn stop_propagation(&self) {
        todo!("Set stop_propagation to true")
    }

    pub fn should_propagate(&self) -> bool {
        todo!("Return !stop_propagation")
    }
}

pub fn add_event_listener<F>(node: &NodeRef, event_type: &str, handler: F)
where
    F: Fn(&Event) + 'static,
{
    todo!("
    1. Wrap handler in Rc
    2. Get or create vec for event_type in event_listeners map
    3. Push handler to vec
    ")
}

pub fn dispatch_event(node: &NodeRef, event_type: &str) {
    todo!("
    1. Create Event with node as target
    2. Fire handlers on target node
    3. Get parent and bubble:
       - Walk up parent chain
       - Fire handlers on each ancestor
       - Stop if event.should_propagate() is false
    ")
}

// Query selector implementation
pub fn query_selector(root: &NodeRef, selector: &str) -> Option<NodeRef> {
    todo!("Return first match of query_selector_all")
}

pub fn query_selector_all(root: &NodeRef, selector: &str) -> Vec<NodeRef> {
    todo!("
    Parse selector and find matching nodes:

    1. Tag selector ('p'): match tag name
    2. ID selector ('#main'): match id attribute
    3. Class selector ('.btn'): match class attribute
    4. Descendant selector ('div p'): p inside div

    Algorithm:
    1. Parse selector into parts
    2. Traverse tree depth-first
    3. Test each node against selector
    4. Collect matches
    ")
}

fn matches_selector(node: &NodeRef, selector: &str) -> bool {
    todo!("
    Check if node matches simple selector:
    - 'p' -> tag == 'p'
    - '#main' -> id == 'main'
    - '.btn' -> class contains 'btn'
    ")
}
```

---

## Milestone 6: Thread-Safe DOM with Arc

**Goal:** Make the tree thread-safe using `Arc` and `Mutex`/`RwLock` for concurrent access.

### Introduction

**Why Milestone 5 Isn't Enough:**

`Rc<RefCell<T>>` only works in single-threaded contexts:
1. **Not Send/Sync**: Can't share across threads
2. **No thread safety**: RefCell panics instead of blocking
3. **No concurrent reads**: Even immutable access requires borrow

**Real-world scenario:** Web browser rendering:
- **Main thread**: Handles user input, builds DOM
- **Layout thread**: Calculates positions and sizes
- **Paint thread**: Draws pixels
- All need concurrent read access to DOM tree

**Solution:** Replace `Rc` → `Arc`, `RefCell` → `RwLock`.

**Performance Impact:**
- `Arc`: Atomic reference counting (~2x slower than Rc)
- `RwLock`: OS-level locking (much slower than RefCell)
- **Benefit**: True parallelism on multi-core systems

### Architecture

```rust
use std::sync::{Arc, Weak, RwLock};

pub type NodeRef = Arc<RwLock<Node>>;
pub type WeakNodeRef = Weak<RwLock<Node>>;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    parent: Option<WeakNodeRef>,
    children: Vec<NodeRef>,
    event_listeners: HashMap<String, Vec<EventHandler>>,
}
```

**Key Changes:**
- `Rc` → `Arc`: Atomic reference counting (thread-safe)
- `RefCell` → `RwLock`: Multiple readers OR one writer
- `borrow()` → `read().unwrap()`: Blocks instead of panicking
- `borrow_mut()` → `write().unwrap()`: Exclusive lock

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_concurrent_reads() {
        let root = create_element("div");

        for i in 0..10 {
            let child = create_element("p");
            child.write().unwrap().set_attribute("index", &i.to_string());
            add_child(&root, &child);
        }

        let mut handles = vec![];

        // 10 threads reading concurrently
        for _ in 0..10 {
            let root_clone = root.clone();
            let handle = thread::spawn(move || {
                let node = root_clone.read().unwrap();
                node.children().len()
            });
            handles.push(handle);
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 10);
        }
    }

    #[test]
    fn test_concurrent_writes() {
        let root = create_element("div");
        let counter = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        // 10 threads adding children concurrently
        for i in 0..10 {
            let root_clone = root.clone();
            let c = counter.clone();

            let handle = thread::spawn(move || {
                let child = create_element(&format!("p{}", i));
                add_child(&root_clone, &child);
                c.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(root.read().unwrap().children().len(), 10);
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_parallel_query() {
        let root = create_element("div");

        // Add 1000 children
        for i in 0..1000 {
            let child = create_element("p");
            if i % 100 == 0 {
                child.write().unwrap().set_attribute("class", "special");
            }
            add_child(&root, &child);
        }

        let mut handles = vec![];

        // 4 threads searching concurrently
        for _ in 0..4 {
            let root_clone = root.clone();
            let handle = thread::spawn(move || {
                query_selector_all(&root_clone, ".special").len()
            });
            handles.push(handle);
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 10);
        }
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<NodeRef>();
        assert_sync::<NodeRef>();
    }

    #[test]
    fn test_deadlock_free_traversal() {
        let root = create_element("html");
        let body = create_element("body");
        let div = create_element("div");

        add_child(&root, &body);
        add_child(&body, &div);

        // Multiple threads traversing
        let mut handles = vec![];

        for _ in 0..5 {
            let root_clone = root.clone();
            let handle = thread::spawn(move || {
                traverse_depth_first(&root_clone, |node| {
                    let _tag = node.read().unwrap().tag().to_string();
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_parallel_event_dispatch() {
        let root = create_element("div");
        let counter = Arc::new(AtomicUsize::new(0));

        // Add 100 children with event listeners
        for i in 0..100 {
            let child = create_element("button");
            let c = counter.clone();

            add_event_listener(&child, "click", move |_| {
                c.fetch_add(1, Ordering::SeqCst);
            });

            add_child(&root, &child);
        }

        let mut handles = vec![];

        // Fire events from 10 threads
        for i in 0..10 {
            let root_clone = root.clone();
            let handle = thread::spawn(move || {
                let children = root_clone.read().unwrap().children().clone();
                for j in 0..10 {
                    dispatch_event(&children[i * 10 + j], "click");
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 100);
    }
}
```

### Starter Code

```rust
use std::sync::{Arc, Weak, RwLock, Mutex};
use std::collections::HashMap;

pub type NodeRef = Arc<RwLock<Node>>;
pub type WeakNodeRef = Weak<RwLock<Node>>;
type EventHandler = Arc<dyn Fn(&Event) + Send + Sync>;

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    parent: Option<WeakNodeRef>,
    children: Vec<NodeRef>,
    event_listeners: HashMap<String, Vec<EventHandler>>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        todo!("Same as before")
    }

    pub fn parent(&self) -> Option<NodeRef> {
        todo!("Upgrade weak reference")
    }

    // ... other methods ...
}

pub fn create_element(tag: &str) -> NodeRef {
    todo!("Return Arc::new(RwLock::new(Node::new(tag)))")
}

pub fn add_child(parent: &NodeRef, child: &NodeRef) {
    todo!("
    1. Get write lock on parent
    2. Add child to children
    3. Get write lock on child
    4. Set parent weak reference

    Important: Don't hold both locks simultaneously!
    Release parent lock before acquiring child lock.
    ")
}

pub fn traverse_depth_first<F>(root: &NodeRef, mut visitor: F)
where
    F: FnMut(&NodeRef),
{
    todo!("
    Visit nodes depth-first:
    1. Call visitor(root)
    2. Get children (must clone Vec to avoid deadlock)
    3. Recursively visit each child

    Deadlock prevention:
    - Clone children vec before recursing
    - Don't hold read lock while recursing
    ")
}

pub fn query_selector_all(root: &NodeRef, selector: &str) -> Vec<NodeRef> {
    todo!("
    Thread-safe version:
    1. Parse selector
    2. Traverse tree
    3. For each node:
       - Acquire read lock
       - Test selector
       - Release lock before continuing
    ")
}

pub fn dispatch_event(target: &NodeRef, event_type: &str) {
    todo!("
    Thread-safe event dispatch:
    1. Create event
    2. Clone event listeners before invoking
       (to avoid holding lock during callback)
    3. Invoke handlers
    4. Bubble to parent
    ")
}

// Parallel tree operations
pub fn parallel_map<F, T>(root: &NodeRef, f: F) -> Vec<T>
where
    F: Fn(&NodeRef) -> T + Send + Sync,
    T: Send,
{
    todo!("
    Use rayon to map function over all nodes in parallel:
    1. Collect all nodes into Vec
    2. Use rayon::par_iter()
    3. Map function over nodes
    ")
}
```

---

## Complete Working Example

Here's a production-quality implementation with all features:

```rust
use std::sync::{Arc, Weak, RwLock};
use std::collections::HashMap;

// ============================================================================
// Type Aliases
// ============================================================================

pub type NodeRef = Arc<RwLock<Node>>;
pub type WeakNodeRef = Weak<RwLock<Node>>;
type EventHandler = Arc<dyn Fn(&Event) + Send + Sync>;

// ============================================================================
// Node Structure
// ============================================================================

pub struct Node {
    tag: String,
    attributes: HashMap<String, String>,
    parent: Option<WeakNodeRef>,
    children: Vec<NodeRef>,
    event_listeners: HashMap<String, Vec<EventHandler>>,
}

impl Node {
    pub fn new(tag: impl Into<String>) -> Self {
        Node {
            tag: tag.into(),
            attributes: HashMap::new(),
            parent: None,
            children: Vec::new(),
            event_listeners: HashMap::new(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn get_attribute(&self, key: &str) -> Option<String> {
        self.attributes.get(key).cloned()
    }

    pub fn parent(&self) -> Option<NodeRef> {
        self.parent.as_ref()?.upgrade()
    }

    pub fn children(&self) -> Vec<NodeRef> {
        self.children.clone()
    }
}

// ============================================================================
// Event System
// ============================================================================

pub struct Event {
    event_type: String,
    target: WeakNodeRef,
    current_target: Option<WeakNodeRef>,
    stop_propagation: Arc<RwLock<bool>>,
}

impl Event {
    pub fn new(event_type: String, target: WeakNodeRef) -> Self {
        Event {
            event_type,
            target,
            current_target: None,
            stop_propagation: Arc::new(RwLock::new(false)),
        }
    }

    pub fn stop_propagation(&self) {
        *self.stop_propagation.write().unwrap() = true;
    }

    pub fn should_propagate(&self) -> bool {
        !*self.stop_propagation.read().unwrap()
    }

    pub fn target(&self) -> Option<NodeRef> {
        self.target.upgrade()
    }
}

// ============================================================================
// DOM Manipulation
// ============================================================================

pub fn create_element(tag: &str) -> NodeRef {
    Arc::new(RwLock::new(Node::new(tag)))
}

pub fn add_child(parent: &NodeRef, child: &NodeRef) {
    // Add child to parent
    parent.write().unwrap().children.push(child.clone());

    // Set parent reference on child
    child.write().unwrap().parent = Some(Arc::downgrade(parent));
}

pub fn remove_child(parent: &NodeRef, child: &NodeRef) {
    parent.write().unwrap().children.retain(|c| !Arc::ptr_eq(c, child));
    child.write().unwrap().parent = None;
}

pub fn remove_from_parent(child: &NodeRef) {
    if let Some(parent) = child.read().unwrap().parent() {
        remove_child(&parent, child);
    }
}

// ============================================================================
// Event Listeners
// ============================================================================

pub fn add_event_listener<F>(node: &NodeRef, event_type: &str, handler: F)
where
    F: Fn(&Event) + Send + Sync + 'static,
{
    let mut node_mut = node.write().unwrap();
    node_mut
        .event_listeners
        .entry(event_type.to_string())
        .or_insert_with(Vec::new)
        .push(Arc::new(handler));
}

pub fn dispatch_event(target: &NodeRef, event_type: &str) {
    let mut event = Event::new(event_type.to_string(), Arc::downgrade(target));

    // Fire on target
    fire_event_on_node(target, &event);

    // Bubble to ancestors
    let mut current = target.read().unwrap().parent();
    while let Some(node) = current {
        if !event.should_propagate() {
            break;
        }

        fire_event_on_node(&node, &event);
        current = node.read().unwrap().parent();
    }
}

fn fire_event_on_node(node: &NodeRef, event: &Event) {
    let handlers = {
        let node_read = node.read().unwrap();
        node_read
            .event_listeners
            .get(&event.event_type)
            .cloned()
            .unwrap_or_default()
    };

    for handler in handlers {
        handler(event);
    }
}

// ============================================================================
// Query Selectors
// ============================================================================

pub fn query_selector(root: &NodeRef, selector: &str) -> Option<NodeRef> {
    query_selector_all(root, selector).into_iter().next()
}

pub fn query_selector_all(root: &NodeRef, selector: &str) -> Vec<NodeRef> {
    let mut results = Vec::new();
    collect_matching_nodes(root, selector, &mut results);
    results
}

fn collect_matching_nodes(node: &NodeRef, selector: &str, results: &mut Vec<NodeRef>) {
    if matches_selector(node, selector) {
        results.push(node.clone());
    }

    let children = node.read().unwrap().children();
    for child in children {
        collect_matching_nodes(&child, selector, results);
    }
}

fn matches_selector(node: &NodeRef, selector: &str) -> bool {
    let node_read = node.read().unwrap();

    if selector.starts_with('#') {
        // ID selector
        let id = &selector[1..];
        node_read.get_attribute("id").as_deref() == Some(id)
    } else if selector.starts_with('.') {
        // Class selector
        let class = &selector[1..];
        node_read
            .get_attribute("class")
            .map(|c| c.split_whitespace().any(|cl| cl == class))
            .unwrap_or(false)
    } else {
        // Tag selector
        node_read.tag() == selector
    }
}

// ============================================================================
// Tree Traversal
// ============================================================================

pub fn get_ancestors(node: &NodeRef) -> Vec<NodeRef> {
    let mut ancestors = Vec::new();
    let mut current = node.read().unwrap().parent();

    while let Some(parent) = current {
        ancestors.push(parent.clone());
        current = parent.read().unwrap().parent();
    }

    ancestors
}

pub fn traverse_depth_first<F>(root: &NodeRef, mut visitor: F)
where
    F: FnMut(&NodeRef),
{
    visitor(root);

    let children = root.read().unwrap().children();
    for child in children {
        traverse_depth_first(&child, &mut visitor);
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

pub fn pretty_print(node: &NodeRef) -> String {
    pretty_print_indent(node, 0)
}

fn pretty_print_indent(node: &NodeRef, indent: usize) -> String {
    let node_read = node.read().unwrap();
    let indent_str = "  ".repeat(indent);
    let mut result = format!("{}<{}", indent_str, node_read.tag());

    // Add attributes
    for (key, value) in &node_read.attributes {
        result.push_str(&format!(" {}=\"{}\"", key, value));
    }
    result.push_str(">\n");

    // Recursively print children
    let children = node_read.children();
    drop(node_read); // Release lock before recursing

    for child in children {
        result.push_str(&pretty_print_indent(&child, indent + 1));
    }

    result.push_str(&format!("{}</{}>\n", indent_str, node.read().unwrap().tag()));
    result
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Build DOM tree
    let html = create_element("html");
    let body = create_element("body");
    let div = create_element("div");

    {
        let mut div_mut = div.write().unwrap();
        div_mut.set_attribute("id", "main");
        div_mut.set_attribute("class", "container");
    }

    let button = create_element("button");
    {
        let mut btn_mut = button.write().unwrap();
        btn_mut.set_attribute("id", "submit");
        btn_mut.set_attribute("class", "btn primary");
    }

    add_child(&html, &body);
    add_child(&body, &div);
    add_child(&div, &button);

    println!("DOM Tree:\n{}", pretty_print(&html));

    // Query selectors
    println!("Finding #submit:");
    if let Some(found) = query_selector(&html, "#submit") {
        println!("  Found: <{}>", found.read().unwrap().tag());
    }

    println!("\nFinding .primary:");
    let primary_elements = query_selector_all(&html, ".primary");
    println!("  Found {} elements", primary_elements.len());

    // Event bubbling
    let click_count = Arc::new(AtomicUsize::new(0));

    let c1 = click_count.clone();
    add_event_listener(&button, "click", move |_| {
        println!("  Button clicked!");
        c1.fetch_add(1, Ordering::SeqCst);
    });

    let c2 = click_count.clone();
    add_event_listener(&div, "click", move |_| {
        println!("  Div received click (bubbled)!");
        c2.fetch_add(1, Ordering::SeqCst);
    });

    println!("\nDispatching click event:");
    dispatch_event(&button, "click");
    println!("Total handlers fired: {}", click_count.load(Ordering::SeqCst));

    // Ancestors
    println!("\nAncestors of button:");
    for ancestor in get_ancestors(&button) {
        println!("  <{}>", ancestor.read().unwrap().tag());
    }

    // Thread-safe concurrent access
    use std::thread;

    println!("\nConcurrent reads from 5 threads:");
    let mut handles = vec![];

    for i in 0..5 {
        let html_clone = html.clone();
        let handle = thread::spawn(move || {
            let tag = html_clone.read().unwrap().tag().to_string();
            println!("  Thread {}: root tag = {}", i, tag);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nDone!");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_example() {
        let root = create_element("div");
        let child = create_element("p");

        child.write().unwrap().set_attribute("id", "text");

        add_child(&root, &child);

        assert_eq!(root.read().unwrap().children().len(), 1);
        assert!(child.read().unwrap().parent().is_some());

        let found = query_selector(&root, "#text");
        assert!(found.is_some());
    }
}
```

**Example Output:**
```
DOM Tree:
<html>
  <body>
    <div id="main" class="container">
      <button id="submit" class="btn primary">
      </button>
    </div>
  </body>
</html>

Finding #submit:
  Found: <button>

Finding .primary:
  Found 1 elements

Dispatching click event:
  Button clicked!
  Div received click (bubbled)!
Total handlers fired: 2

Ancestors of button:
  <div>
  <body>
  <html>

Concurrent reads from 5 threads:
  Thread 0: root tag = html
  Thread 1: root tag = html
  Thread 2: root tag = html
  Thread 3: root tag = html
  Thread 4: root tag = html

Done!
```

---

## Summary

You've built a production-grade DOM tree implementation with:

### Features Implemented
1. ✅ Reference-counted nodes with `Rc`/`Arc`
2. ✅ Interior mutability with `RefCell`/`RwLock`
3. ✅ Parent pointers using `Weak` (no memory leaks!)
4. ✅ Event bubbling and listeners
5. ✅ CSS-style query selectors
6. ✅ Thread-safe concurrent access

### Smart Pointer Patterns Mastered
- `Box<T>`: Unique ownership on heap
- `Rc<T>`: Shared ownership (single-threaded)
- `Arc<T>`: Atomic shared ownership (multi-threaded)
- `Weak<T>`: Non-owning references (break cycles)
- `RefCell<T>`: Interior mutability (single-threaded)
- `RwLock<T>`: Interior mutability (multi-threaded)

### Performance Characteristics
| Pattern | Clone Cost | Access Cost | Thread-Safe | Cyclic? |
|---------|-----------|-------------|-------------|---------|
| `Box` | Deep copy | Zero | No | No |
| `Rc<RefCell>` | O(1) | O(1) | No | Yes (use Weak) |
| `Arc<RwLock>` | O(1) | Lock | Yes | Yes (use Weak) |

### Real-World Applications
- Web browser DOM (Chrome, Firefox)
- GUI frameworks (GTK, Qt, egui)
- XML parsers (serde-xml-rs)
- Game scene graphs (Bevy, Amethyst)

### Key Lessons
1. **Rc vs Arc**: Use Rc for single-threaded, Arc for multi-threaded
2. **RefCell vs RwLock**: RefCell panics, RwLock blocks
3. **Weak breaks cycles**: Always use Weak for parent pointers
4. **Lock carefully**: Release locks before recursive calls to avoid deadlocks
5. **Clone strategically**: Clone collections before holding locks

Congratulations! You now understand the smart pointer patterns used in every major Rust GUI framework and browser engine.
