# Copy-on-Write Data Structures

## Problem Statement

Build a library of Copy-on-Write (CoW) data structures that enable efficient sharing of data until modification is needed. When data is shared, cloning is O(1) (just increment reference count). When data is modified, make a private copy only if other references exist.

The library must support:
- CoW String with cheap cloning
- CoW Vec with structural sharing
- CoW HashMap with lazy copying
- Automatic copy detection (only copy if shared)
- Configurable sharing strategies
- Performance tracking and optimization

## Why It Matters

**Performance Impact:**
- **Cloning large strings**: Normal clone takes ~1μs per KB, CoW clone takes ~10ns (100x faster!)
- **Configuration systems**: Share config across threads, copy only on write
- **Immutable data structures**: Functional programming patterns in Rust
- **Version control**: Git uses CoW for file storage

**Memory Savings:**
```
Normal clones:  5 copies × 1MB = 5MB memory
CoW clones:     5 references × 8 bytes = 40 bytes (until write)
Savings:        99.9% memory reduction
```

## Use Cases

1. **Configuration Management**: Share config across threads, clone on modification
2. **Caching Systems**: Cache entries share backing data until mutated
3. **Immutable Collections**: Functional-style data structures
4. **Version Control**: Store file versions efficiently (like Git)
5. **String Interning**: Share common strings (like "http", "200 OK")
6. **Game State**: Share game state snapshots for replay/undo

---

## Core Concepts: Copy-on-Write and Structural Sharing

Before diving into implementation, understanding these core concepts will help you appreciate why Copy-on-Write is a fundamental pattern in systems programming and how it enables efficient immutable data structures.

### 1. Copy-on-Write Fundamentals

**The Cloning Problem:**

In Rust, cloning creates a complete deep copy of data:

```rust
let vec1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];  // 10 elements
let vec2 = vec1.clone();  // Full copy: allocate + memcpy 10 elements

// Memory: 2 complete copies (80 bytes if i32)
// Time: O(n) allocation + O(n) copy
```

For large data structures, cloning is expensive:
```
1KB string:     ~1μs to clone
1MB buffer:     ~500μs to clone
1GB dataset:    ~500ms to clone!
```

**The CoW Solution:**

Instead of immediately copying, share the data until modification:

```rust
let vec1 = CowVec::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
let vec2 = vec1.clone();  // O(1) - just increment reference count!

// Memory: 1 copy + 2 pointers (16 bytes overhead vs 80 bytes data)
// Time: O(1) pointer copy

// Reading is shared:
println!("{}", vec1[0]);  // ✅ No copy
println!("{}", vec2[0]);  // ✅ No copy

// Writing triggers copy:
let mut vec3 = vec1.clone();  // O(1) clone
vec3.push(11);  // NOW we copy: O(n)

// Result:
// vec1, vec2: still share original data
// vec3: has private copy with new element
```

**Key Insight:**

```
Copy-on-Write = Lazy Cloning
- Clone operation: O(1) (just reference counting)
- Copy operation: O(n) (only when writing)
- Memory: Shared until divergence
```

### 2. Reference Counting with Arc

**Arc: Atomic Reference Counted Pointer**

`Arc<T>` is a thread-safe smart pointer that tracks how many owners exist:

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);
println!("Count: {}", Arc::strong_count(&data));  // 1

let data2 = Arc::clone(&data);
println!("Count: {}", Arc::strong_count(&data));  // 2

let data3 = Arc::clone(&data);
println!("Count: {}", Arc::strong_count(&data));  // 3

drop(data2);
println!("Count: {}", Arc::strong_count(&data));  // 2

drop(data3);
drop(data);
// Count reaches 0 → data deallocated
```

**Arc Operations:**

```rust
// Create
let arc = Arc::new(value);                    // Cost: O(1)

// Clone (increment refcount)
let arc2 = Arc::clone(&arc);                  // Cost: ~5ns (atomic increment)

// Read (deref)
let val = &*arc;                              // Cost: ~1ns (just pointer deref)

// Check sharing
let is_shared = Arc::strong_count(&arc) > 1;  // Cost: ~1ns (atomic read)

// Unwrap (if exclusive owner)
if let Ok(value) = Arc::try_unwrap(arc) {    // Cost: O(1) or fails
    // Got value without cloning!
}
```

**Performance Characteristics:**

```rust
Box<T>          → No refcounting, ~0ns overhead
Rc<T>           → Non-atomic refcount, ~1ns overhead
Arc<T>          → Atomic refcount, ~5ns overhead
```

The ~4ns difference is worth it for thread-safety!

### 3. Arc::make_mut() - The CoW Primitive

**The Magic Function:**

`Arc::make_mut()` is the key to Copy-on-Write:

```rust
pub fn make_mut<T: Clone>(arc: &mut Arc<T>) -> &mut T {
    if Arc::strong_count(arc) == 1 {
        // Exclusive owner - return mutable reference directly
        unsafe { Arc::get_mut_unchecked(arc) }
    } else {
        // Shared - clone the data and replace Arc
        *arc = Arc::new((**arc).clone());
        Arc::get_mut(arc).unwrap()
    }
}
```

**How It Works:**

```rust
let mut cow = Arc::new(vec![1, 2, 3]);

// Scenario 1: Exclusive ownership
Arc::make_mut(&mut cow).push(4);
// → No clone! Modifies in place
// → Cost: O(1)

// Scenario 2: Shared ownership
let cow2 = Arc::clone(&cow);
Arc::make_mut(&mut cow).push(5);
// → Clones data first
// → Cost: O(n) for clone + O(1) for modification
// → cow now has private copy
// → cow2 still has original data
```

**Visual Example:**

```
Before make_mut():
cow1 ───┐
        ├──> [1, 2, 3] (refcount: 2)
cow2 ───┘

After cow1.make_mut().push(4):
cow1 ────> [1, 2, 3, 4] (refcount: 1, private copy)
cow2 ────> [1, 2, 3] (refcount: 1, original data)
```

### 4. Lazy Evaluation and Structural Sharing

**Lazy Evaluation:**

CoW delays expensive operations until absolutely necessary:

```rust
// No cloning happens here:
let v1 = Cow::new(vec![0; 1_000_000]);  // 1MB vec
let v2 = v1.clone();  // O(1) - just increment refcount
let v3 = v1.clone();  // O(1)
let v4 = v1.clone();  // O(1)

// Memory usage: ~1MB (all share same data)

// First write triggers clone:
let mut v5 = v1.clone();  // O(1)
v5.make_mut().push(1);    // O(n) - copies 1MB!

// Now memory usage: ~2MB (v5 has private copy)
```

**Structural Sharing:**

Unlike deep copying, CoW enables sharing at different granularities:

```rust
// Persistent data structures (like Clojure/Haskell)
let tree1 = PersistentTree::from([1, 2, 3, 4, 5, 6, 7, 8]);

let tree2 = tree1.insert(9);
// Shares most nodes with tree1
// Only creates new nodes along path to insertion
//
//     tree1          tree2
//       4              4
//      / \            / \
//     2   6          2   6 (shared)
//    / \ / \        / \ / \
//   1  3 5  7      1  3 5  9 (new)
//  (shared)           (new)
```

**Memory Efficiency Example:**

```rust
// Scenario: 100 threads need access to 1MB config

// Approach 1: Deep clone
let configs: Vec<_> = (0..100)
    .map(|_| config.clone())  // 100 × 1MB = 100MB
    .collect();

// Approach 2: CoW
let configs: Vec<_> = (0..100)
    .map(|_| cow_config.clone())  // 100 × 8 bytes = 800 bytes
    .collect();

// Memory savings: 99.9%!
```

### 5. Immutable Data Structures

**Why Immutability Matters:**

Immutable data provides powerful guarantees:

```rust
// Mutable approach
fn process_config(config: &mut Config) {
    config.timeout = 60;  // Mutates original!
}

let mut cfg = Config { timeout: 30 };
process_config(&mut cfg);
println!("{}", cfg.timeout);  // 60 - changed!

// Immutable CoW approach
fn process_config(config: &Cow<Config>) -> Cow<Config> {
    let mut new_config = config.clone();  // O(1)
    new_config.make_mut().timeout = 60;   // O(n) only if writing
    new_config
}

let cfg1 = Cow::new(Config { timeout: 30 });
let cfg2 = process_config(&cfg1);
println!("{}", cfg1.timeout);  // 30 - unchanged!
println!("{}", cfg2.timeout);  // 60 - new version
```

**Benefits:**

1. **Thread-safety**: Immutable data can be shared without locks
2. **Reasoning**: No spooky action at a distance
3. **Versioning**: Keep old versions cheaply (undo/redo)
4. **Debugging**: Values don't change under your feet
5. **Functional programming**: Enables pure functions

**Trade-offs:**

```
Mutable:
✅ Fast in-place updates
✅ Lower memory (single version)
❌ Hard to share safely
❌ Hard to reason about

Immutable (CoW):
✅ Safe sharing
✅ Easy reasoning
✅ Versioning for free
❌ Write overhead (copy on first write)
❌ Slightly higher memory (multiple versions)
```

### 6. Thread Safety Without Locks

**The Lock-Free Read Pattern:**

CoW with Arc provides lock-free reads:

```rust
// Traditional approach: Mutex
let config = Arc::new(Mutex::new(HashMap::new()));

// Reading requires lock (slow!)
let value = config.lock().unwrap().get("key");  // ~20ns lock overhead

// CoW approach: No locks needed for reads
let config = Cow::new(HashMap::new());

// Reading is lock-free (fast!)
let value = config.get("key");  // ~1ns, no locking
```

**Concurrent Readers:**

```rust
use std::thread;

let data = Cow::new(vec![1, 2, 3, 4, 5]);

// Spawn 100 reader threads
let handles: Vec<_> = (0..100)
    .map(|_| {
        let d = data.clone();  // O(1) atomic increment
        thread::spawn(move || {
            d.iter().sum::<i32>()  // ✅ No locks!
        })
    })
    .collect();

// All readers execute in parallel, no contention!
```

**Write Pattern:**

```rust
// Writer thread
let mut data = shared_data.clone();
data.make_mut().push(6);  // Copies if shared

// Key insight:
// - Readers see old version (no lock needed)
// - Writer gets private copy (no lock needed)
// - No lock contention!
```

**Comparison:**

```rust
// RwLock approach
Arc<RwLock<T>>:
- Read: ~20ns (acquire read lock)
- Write: ~50ns (acquire write lock)
- Contention: Readers block writers, writer blocks everyone

// CoW approach
Cow<T> (Arc<T>):
- Read: ~1ns (just deref)
- Write: O(n) for copy, but no blocking
- Contention: None! Readers and writers never block
```

### 7. Performance Characteristics and Trade-offs

**Operation Costs:**

| Operation | Normal Clone | CoW (Shared) | CoW (Exclusive) |
|-----------|-------------|--------------|----------------|
| Clone | O(n) | O(1) ~5ns | O(1) ~5ns |
| Read | O(1) ~1ns | O(1) ~1ns | O(1) ~1ns |
| Write | O(1) ~1ns | O(n) + O(1) | O(1) ~1ns |
| Memory | n per clone | n + k×(pointer) | n |

**Break-Even Analysis:**

```rust
// When does CoW win?

// Cost of normal approach:
// - k clones × n items = k*n time

// Cost of CoW approach:
// - k clones × 1 = k time (cloning)
// - m writes × n = m*n time (copying)

// CoW wins when: k < m
// i.e., when clones > writes
```

**Real-World Example:**

```rust
// Web server config (read-heavy: 1000 reads, 1 write)

// Normal: 1000 clones × 500μs = 500ms
let configs: Vec<_> = (0..1000)
    .map(|_| config.clone())
    .collect();

// CoW: 1000 clones × 5ns + 1 write × 500μs = 5μs + 500μs = 505μs
let configs: Vec<_> = (0..1000)
    .map(|_| cow_config.clone())
    .collect();

// Speedup: 500ms / 0.505ms = 990x faster!
```

### 8. Memory Management Strategies

**Arc Reference Counting:**

```rust
// Memory layout
Arc<Vec<i32>>
  ├─ Strong count: 3
  ├─ Weak count: 0
  └─ Data: Vec<i32> { ptr, len, cap }
           └─> [1, 2, 3, 4, 5] on heap

// Each Arc clone:
// - 8 bytes (pointer to refcounted allocation)
// - Atomic increment of strong_count
```

**Memory Overhead:**

```rust
// For Vec<i32> with 100 elements:
Direct: 400 bytes (100 × 4 bytes)
Arc<Vec<i32>>: 400 bytes data + 16 bytes (counts) = 416 bytes

// With 10 clones:
Direct clones: 10 × 400 = 4000 bytes
Arc clones: 416 bytes + 10 × 8 bytes = 496 bytes

// Savings: (4000 - 496) / 4000 = 87.6%
```

**Arc::try_unwrap Optimization:**

```rust
let arc = Arc::new(vec![1, 2, 3]);

// Try to extract value without cloning
match Arc::try_unwrap(arc) {
    Ok(vec) => {
        // Success! No clone needed
        // Use vec directly
    }
    Err(arc) => {
        // Still shared, must clone
        let vec = (*arc).clone();
    }
}
```

### 9. Real-World Use Cases

**Use Case 1: Configuration Management**

```rust
// Loaded once, shared everywhere
let config = Cow::new(AppConfig::load());

// Cheap to pass to all components
let server = HttpServer::new(config.clone());
let worker = Worker::new(config.clone());
let logger = Logger::new(config.clone());

// Admin updates config (rare)
fn update_config(old: &Cow<AppConfig>, new_val: u32) -> Cow<AppConfig> {
    let mut updated = old.clone();  // O(1)
    updated.make_mut().timeout = new_val;  // Copy only if still shared
    updated
}
```

**Use Case 2: Version Control**

```rust
// Git-like commit history
struct Commit {
    message: String,
    tree: Cow<FileTree>,  // Shares unchanged files
    parent: Option<Box<Commit>>,
}

impl Commit {
    fn modify_file(&self, path: &str, content: String) -> Commit {
        let mut new_tree = self.tree.clone();  // O(1)
        new_tree.make_mut().insert(path, content);  // Copy modified path

        Commit {
            message: "Update file".into(),
            tree: new_tree,
            parent: Some(Box::new(self.clone())),
        }
    }
}
```

**Use Case 3: Immutable Collections (Functional Programming)**

```rust
// Clojure-style persistent vector
fn functional_update(vec: &Cow<Vec<i32>>, f: impl Fn(i32) -> i32) -> Cow<Vec<i32>> {
    let mut new_vec = vec.clone();  // O(1) if shared
    for x in new_vec.make_mut().iter_mut() {
        *x = f(*x);  // Copies on first modification
    }
    new_vec
}

let v1 = Cow::new(vec![1, 2, 3]);
let v2 = functional_update(&v1, |x| x * 2);
let v3 = functional_update(&v2, |x| x + 1);

// v1 = [1, 2, 3]
// v2 = [2, 4, 6]
// v3 = [3, 5, 7]
// All cheaply derived from each other!
```

### 10. Anti-Patterns and When NOT to Use CoW

**Anti-Pattern 1: Write-Heavy Workloads**

```rust
// BAD: CoW with mostly writes
let mut cow = Cow::new(vec![1, 2, 3]);

for i in 0..1000 {
    cow.make_mut().push(i);  // If shared, copies EVERY iteration!
}

// GOOD: Use regular Vec
let mut vec = vec![1, 2, 3];
for i in 0..1000 {
    vec.push(i);  // In-place, O(1) amortized
}
```

**Anti-Pattern 2: Small Data**

```rust
// BAD: CoW for tiny data
let cow = Cow::new(42_i32);  // 4 bytes data + 16 bytes Arc overhead = 20 bytes

// GOOD: Just copy
let value = 42_i32;  // 4 bytes, trivial to copy
```

**Anti-Pattern 3: Always Modifying After Clone**

```rust
// BAD: Clone then always modify
fn process(data: &Cow<Vec<i32>>) -> Cow<Vec<i32>> {
    let mut result = data.clone();  // O(1)
    result.make_mut().push(1);      // O(n) copy ALWAYS happens
    result
}

// GOOD: Just use regular clone
fn process(data: &Vec<i32>) -> Vec<i32> {
    let mut result = data.clone();  // O(n) but honest
    result.push(1);
    result
}
```

**When to Use CoW:**

✅ Read-heavy (>80% reads)
✅ Large data structures (>1KB)
✅ Sharing across threads
✅ Immutable data modeling
✅ Version control / undo functionality
✅ Configuration sharing

**When NOT to Use CoW:**

❌ Write-heavy (>50% writes)
❌ Small data (<100 bytes)
❌ Always modified after clone
❌ Need guaranteed O(1) writes
❌ Single-threaded + exclusive ownership

### 11. Comparison with std::borrow::Cow

**Rust's Standard Library Cow:**

```rust
use std::borrow::Cow;

// Cow in std: Clone-on-Write OR Borrowed
let owned: Cow<str> = Cow::Owned(String::from("hello"));
let borrowed: Cow<str> = Cow::Borrowed("hello");

// Converts to owned when needed:
let mut cow = Cow::Borrowed("hello");
cow.to_mut().push_str(" world");  // Now Owned
```

**Differences:**

```
std::borrow::Cow<'a, T>:
- Borrowed XOR Owned
- Lifetime-bound
- For avoiding allocations
- Used in APIs to accept &str or String

Our Cow<T> (Arc-based):
- Always owned (via Arc)
- No lifetimes
- For sharing across threads
- Used for immutable data structures
```

**When to Use Which:**

```rust
// Use std::borrow::Cow for API flexibility:
fn process(s: Cow<str>) {
    // Can accept &str OR String without allocation
}
process(Cow::Borrowed("static"));
process(Cow::Owned(dynamic_string));

// Use Arc<T> based Cow for sharing:
fn share_config(cfg: Cow<Config>) {
    thread::spawn(move || {
        // cfg can outlive parent scope
        use_config(cfg);
    });
}
```

---

## Connection to This Project

This project progressively builds a production-quality Copy-on-Write library, with each milestone introducing essential patterns for efficient immutable data structures.

### Milestone Progression and Learning Path

| Milestone | Data Structure | Technique | Capabilities | Real-World Equivalent |
|-----------|---------------|-----------|--------------|----------------------|
| 1. Basic CoW | String | Arc + make_mut | Lazy cloning | String interning |
| 2. Collections | Vec | Structural sharing | Indexed access, modification | Immutable vectors |
| 3. Mappings | HashMap | Lazy copying | Key-value operations | Config systems |
| 4. Generic | Cow\<T\> | Unified wrapper | Works with any Clone type | Persistent data structures |
| 5. Thread-Safe | Arc + Send/Sync | Lock-free sharing | Concurrent reads/writes | Multi-threaded config |
| 6. Optimized | Metrics | Performance tracking | Observability | Production systems |

### Why Each Pattern Matters

**Milestone 1 (Arc + make_mut): The CoW Foundation**

Establishes core concepts:
- Arc for reference counting
- make_mut for copy-on-write
- Deref for transparent access
- O(1) clone, O(n) first write

**Key learning:**
```rust
// Before understanding CoW:
let s1 = String::from("hello");
let s2 = s1.clone();  // O(n) - full copy

// After understanding CoW:
let s1 = CowString::new("hello");
let s2 = s1.clone();  // O(1) - just refcount++
```

**Milestone 2 (Vec with Index): Collection Operations**

Solves: Indexed access to shared collections
- IndexMut triggers copy
- Iterator doesn't trigger copy
- Element modification strategies

**Real-world impact:**
```rust
// Sharing game state across replay system
let game_state = CowVec::from(entities);

// 60 FPS × 10 seconds = 600 frames
let replay_buffer: Vec<_> = (0..600)
    .map(|_| game_state.clone())  // 600 × O(1) = O(1)
    .collect();

// vs normal: 600 × O(n) = O(600n) - way too slow!
```

**Milestone 3 (HashMap): Complex Structures**

Solves: Configuration and key-value sharing
- Insert/remove operations
- Entry API challenges
- Iteration without copying

**The config sharing pattern:**
```rust
// Load config once
let config: CowHashMap<String, Value> = load_config();

// Share across all workers (100 threads)
for _ in 0..100 {
    let cfg = config.clone();  // O(1)
    spawn_worker(cfg);
}

// Memory: 1 × config size (vs 100 × config size)
// Savings: 99%!
```

**Milestone 4 (Generic Cow\<T\>): Unification**

Solves: Code duplication
- Single implementation for all types
- Consistent API
- Works with custom structs

**Before:**
```rust
CowString  → 200 lines
CowVec     → 200 lines
CowHashMap → 200 lines
Total: 600 lines, lots of duplication
```

**After:**
```rust
Cow<T>     → 100 lines
Works with String, Vec, HashMap, custom types!
```

**Milestone 5 (Thread-Safety): Concurrent Sharing**

Solves: Multi-threaded access
- Lock-free reads (Arc deref is fast)
- Safe writes (copy-on-write isolation)
- No deadlocks possible

**Performance comparison:**
```rust
// Mutex approach (blocking)
Arc<Mutex<Config>>:
- 100 readers: ~2μs total (sequential due to lock)
- 1 writer: blocks all readers

// CoW approach (lock-free)
Cow<Config>:
- 100 readers: ~100ns total (parallel!)
- 1 writer: gets private copy, doesn't block readers

Speedup: 20x for readers!
```

**Milestone 6 (Metrics): Production Readiness**

Solves: Observability
- How often do we actually copy?
- Is CoW saving memory?
- What's the copy rate?
- Should we optimize differently?

**Metrics-driven optimization:**
```
Initial: 1000 clones, 500 copies → 50% copy rate
Problem: Too many writes, CoW not helping!

Action: Cache frequently modified data separately
Result: 1000 clones, 50 copies → 5% copy rate
Success: CoW now effective!
```

### Performance Journey

Understanding the trade-offs at each stage:

| Pattern | Clone Cost | Write Cost | Memory (10 clones) | Use Case |
|---------|-----------|------------|-------------------|----------|
| Direct clone | O(n) ~1μs/KB | O(1) ~1ns | 10 × size | Exclusive ownership |
| Arc (immutable) | O(1) ~5ns | ❌ Can't write | 1 × size | Read-only sharing |
| Arc<Mutex> | O(1) ~5ns | ~20ns + O(1) | 1 × size | Rare writes with locks |
| **Cow (Arc + make_mut)** | O(1) ~5ns | O(n) first, O(1) after | 1-10 × size | **Read-heavy, lock-free** |

**The sweet spot:** Cow excels when reads > 10× writes

### Real-World Impact Examples

**Example 1: Web Server Configuration**

```rust
// Problem: 1000 worker threads need config access
// Config: 10KB HashMap

// Without CoW:
let configs: Vec<_> = (0..1000)
    .map(|_| config.clone())  // 1000 × 10KB = 10MB
    .collect();

// With CoW:
let cow_config = Cow::new(config);
let configs: Vec<_> = (0..1000)
    .map(|_| cow_config.clone())  // 1000 × 8 bytes = 8KB
    .collect();

// Memory saved: 10MB - 8KB = 9.992MB (99.9% reduction!)

// Performance:
// - Without CoW: 1000 × 5μs = 5ms to distribute config
// - With CoW: 1000 × 5ns = 5μs to distribute config
// - Speedup: 1000x!
```

**Example 2: Game State Replay System**

```rust
// Problem: Store 600 game states for 10-second replay at 60 FPS
// State size: 1MB (entities, physics, etc.)

struct GameState {
    entities: Vec<Entity>,
    physics: PhysicsWorld,
    // ... other state
}

// Without CoW:
let mut replay: Vec<GameState> = Vec::new();
for frame in 0..600 {
    replay.push(current_state.clone());  // 600 × 1MB = 600MB!
}

// With CoW:
let mut replay: Vec<Cow<GameState>> = Vec::new();
for frame in 0..600 {
    replay.push(Cow::new(current_state.clone()));  // ~600 frames share data
}

// If only 10% of entities change each frame:
// Memory: ~60MB (10% × 600 frames) vs 600MB
// Savings: 90%!
```

**Example 3: Immutable Document Editor**

```rust
// Problem: Text editor with undo/redo
// Document: 1MB text

struct Document {
    content: Cow<String>,
    cursor: usize,
}

impl Document {
    fn insert_char(&self, c: char) -> Document {
        let mut new_content = self.content.clone();  // O(1)
        new_content.make_mut().insert(self.cursor, c);  // O(n) copy

        Document {
            content: new_content,
            cursor: self.cursor + 1,
        }
    }
}

// Undo stack: Vec<Document>
// With CoW: Only modified versions consume memory
// Without CoW: Each version is 1MB → unusable

// For 100 edits with 10% changes each:
// CoW: ~1.1MB total (original + 10 deltas)
// Direct: 100MB total (100 full copies)
// Savings: 99%!
```

### Architectural Insights

**Pattern 1: Arc::make_mut for Lazy Copying**

```rust
// Encapsulation of copy-on-write logic:
pub struct Cow<T: Clone> {
    data: Arc<T>,
}

impl<T: Clone> Cow<T> {
    pub fn make_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.data)
        // Automatically copies if shared!
    }
}
```

**Pattern 2: Deref for Transparency**

```rust
// Cow acts like the inner type for reading:
impl<T: Clone> Deref for Cow<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.data }
}

// Usage:
let cow = Cow::new(vec![1, 2, 3]);
println!("{}", cow.len());  // Deref to Vec::len()
println!("{}", cow[0]);     // Deref to Vec::index()
```

**Pattern 3: Metrics for Validation**

```rust
// Track copy behavior:
struct CowStats {
    clones: AtomicUsize,  // How many times cloned
    copies: AtomicUsize,  // How many times actually copied
}

// Validate CoW is helping:
let stats = cow.stats();
if stats.copy_rate() > 0.5 {
    // More than 50% copy rate - CoW not effective!
    // Consider different approach
}
```

### Skills Transferred to Other Domains

After completing this project, you'll understand patterns used in:

1. **Functional Programming Languages** (Clojure, Haskell, OCaml)
   - Persistent data structures
   - Structural sharing
   - Immutable by default

2. **Version Control Systems** (Git, Mercurial)
   - Blob storage with CoW
   - Cheap branching
   - Content-addressable storage

3. **Databases** (PostgreSQL MVCC, CouchDB)
   - Multi-version concurrency control
   - Snapshot isolation
   - Copy-on-write B-trees

4. **Operating Systems** (Linux fork(), ZFS)
   - Process forking with CoW pages
   - Copy-on-write filesystems
   - Memory-efficient snapshots

5. **Libraries** (im crate, Immutable.js)
   - Persistent collections
   - Functional data structures
   - React state management

### Key Takeaways

1. **Arc::make_mut is the key**: Automatic copy detection based on refcount

2. **Read-heavy workflows win big**: 10:1 read-to-write ratio = ~10x speedup

3. **Memory efficiency scales**: More clones = more savings (until first write)

4. **Thread-safe without locks**: Readers never block, writers get private copies

5. **Measure to validate**: Use metrics to ensure CoW is actually helping

6. **Anti-pattern awareness**: Don't use CoW for write-heavy or tiny data

7. **Immutability enables reasoning**: Pure functions, easier debugging, safe sharing

8. **Structural sharing is powerful**: Share unchanged portions, copy only deltas

This project teaches you the patterns behind immutable data structures, version control systems, and functional programming - all built on the simple but powerful Copy-on-Write principle.

---

## Milestone 1: Basic CoW String with Arc

**Goal:** Create a copy-on-write string that shares data until modification.

### Introduction

We start with the simplest CoW implementation: a string that:
- Uses `Arc<String>` for shared data
- Cloning is O(1) (just increment refcount)
- First write makes a private copy
- Supports transparent read access

**Limitations we'll address later:**
- Always copies entire string on first write
- No slice sharing
- Only works for String, not Vec or HashMap
- No performance tracking

### Architecture

```rust
use std::sync::Arc;

pub struct CowString {
    data: Arc<String>,
}
```

**Key Concepts:**
- `Arc::strong_count()`: Check if data is shared
- Clone makes copy only if `strong_count() > 1`
- `Arc::make_mut()`: Gets mutable reference, copying if needed

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create() {
        let s = CowString::new("hello");
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_cheap_clone() {
        let s1 = CowString::new("hello world");
        let s2 = s1.clone();

        // Both point to same data
        assert_eq!(s1.as_str(), "hello world");
        assert_eq!(s2.as_str(), "hello world");
        assert_eq!(Arc::strong_count(&s1.data), 2);
    }

    #[test]
    fn test_copy_on_write() {
        let s1 = CowString::new("hello");
        let mut s2 = s1.clone();

        // s2 shares data with s1
        assert_eq!(Arc::strong_count(&s1.data), 2);

        // Modify s2 - should copy
        s2.push_str(" world");

        assert_eq!(s1.as_str(), "hello");
        assert_eq!(s2.as_str(), "hello world");

        // s2 now has independent copy
        assert_eq!(Arc::strong_count(&s1.data), 1);
        assert_eq!(Arc::strong_count(&s2.data), 1);
    }

    #[test]
    fn test_exclusive_modification() {
        let mut s = CowString::new("hello");

        // Not shared - no copy needed
        s.push_str(" world");

        assert_eq!(s.as_str(), "hello world");
        assert_eq!(Arc::strong_count(&s.data), 1);
    }

    #[test]
    fn test_multiple_clones() {
        let s1 = CowString::new("shared");
        let s2 = s1.clone();
        let s3 = s1.clone();

        assert_eq!(Arc::strong_count(&s1.data), 3);

        drop(s2);
        assert_eq!(Arc::strong_count(&s1.data), 2);

        drop(s3);
        assert_eq!(Arc::strong_count(&s1.data), 1);
    }

    #[test]
    fn test_from_string() {
        let s = String::from("hello");
        let cow = CowString::from(s);
        assert_eq!(cow.as_str(), "hello");
    }

    #[test]
    fn test_deref() {
        let s = CowString::new("hello world");

        // Can use String methods through Deref
        assert_eq!(s.len(), 11);
        assert!(s.starts_with("hello"));
        assert_eq!(&s[0..5], "hello");
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub struct CowString {
    data: Arc<String>,
}

impl CowString {
    pub fn new(s: impl Into<String>) -> Self {
        todo!("
        Wrap string in Arc:
        CowString {
            data: Arc::new(s.into()),
        }
        ")
    }

    pub fn as_str(&self) -> &str {
        todo!("Return &str from inner Arc<String>")
    }

    pub fn push_str(&mut self, s: &str) {
        todo!("
        Get mutable reference with Arc::make_mut:
        1. Arc::make_mut(&mut self.data) - copies if shared
        2. Call push_str on the String

        Arc::make_mut automatically:
        - Returns &mut String if strong_count == 1
        - Clones and returns &mut String if shared
        ")
    }

    pub fn is_shared(&self) -> bool {
        todo!("Return Arc::strong_count(&self.data) > 1")
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }
}

impl From<String> for CowString {
    fn from(s: String) -> Self {
        CowString::new(s)
    }
}

impl From<&str> for CowString {
    fn from(s: &str) -> Self {
        CowString::new(s)
    }
}

impl Deref for CowString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        todo!("Return &str from Arc<String>")
    }
}
```

---

## Milestone 2: CoW Vec with Structural Sharing

**Goal:** Extend CoW pattern to `Vec<T>` with element-level sharing.

### Introduction

**Why Milestone 1 Isn't Enough:**

Strings are simple, but Vecs have more complex operations:
1. **Push/pop**: Modify size
2. **Indexing**: Access/modify individual elements
3. **Slicing**: View subsets
4. **Generic types**: Must work with any `T: Clone`

**Real-world scenario:** Configuration system with arrays:
```rust
let config = CowVec::from(vec![1, 2, 3, 4, 5]);

// 10 threads read config (cheap clones)
let threads: Vec<_> = (0..10)
    .map(|_| {
        let cfg = config.clone(); // O(1) clone
        thread::spawn(move || process(cfg))
    })
    .collect();

// One thread modifies (triggers copy)
let mut modified = config.clone();
modified.push(6); // Copy happens here
```

**Challenge:** Implement Index, IndexMut, push, pop, etc.

### Architecture

```rust
use std::sync::Arc;

pub struct CowVec<T> {
    data: Arc<Vec<T>>,
}
```

**Key Methods:**
- `push(&mut self, value: T)`: Append element
- `pop(&mut self) -> Option<T>`: Remove last
- `get(&self, index: usize) -> Option<&T>`: Read element
- `get_mut(&mut self, index: usize) -> Option<&mut T>`: Write element

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vec() {
        let v = CowVec::from(vec![1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(v.get(0), Some(&1));
    }

    #[test]
    fn test_clone_vec() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let v2 = v1.clone();

        assert_eq!(v1.strong_count(), 2);
        assert_eq!(v1[0], 1);
        assert_eq!(v2[0], 1);
    }

    #[test]
    fn test_copy_on_push() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let mut v2 = v1.clone();

        assert_eq!(v1.strong_count(), 2);

        v2.push(4);

        // v2 copied
        assert_eq!(v1.len(), 3);
        assert_eq!(v2.len(), 4);
        assert_eq!(v1.strong_count(), 1);
        assert_eq!(v2.strong_count(), 1);
    }

    #[test]
    fn test_copy_on_modify() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let mut v2 = v1.clone();

        // Modify via index
        v2[0] = 99;

        assert_eq!(v1[0], 1);
        assert_eq!(v2[0], 99);
    }

    #[test]
    fn test_pop() {
        let mut v = CowVec::from(vec![1, 2, 3]);

        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_iter() {
        let v = CowVec::from(vec![1, 2, 3, 4, 5]);

        let sum: i32 = v.iter().sum();
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_shared_iter() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let v2 = v1.clone();

        // Both can iterate
        assert_eq!(v1.iter().sum::<i32>(), 6);
        assert_eq!(v2.iter().sum::<i32>(), 6);

        // Still shared
        assert_eq!(v1.strong_count(), 2);
    }

    #[test]
    fn test_into_vec() {
        let cow = CowVec::from(vec![1, 2, 3]);

        let vec = cow.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_into_vec_shared() {
        let v1 = CowVec::from(vec![1, 2, 3]);
        let v2 = v1.clone();

        // Must clone because shared
        let vec = v1.into_vec();
        assert_eq!(vec, vec![1, 2, 3]);

        // v2 still valid
        assert_eq!(v2[0], 1);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::{Deref, Index, IndexMut};

#[derive(Clone)]
pub struct CowVec<T> {
    data: Arc<Vec<T>>,
}

impl<T: Clone> CowVec<T> {
    pub fn new() -> Self {
        CowVec {
            data: Arc::new(Vec::new()),
        }
    }

    pub fn push(&mut self, value: T) {
        todo!("
        Use Arc::make_mut to get mutable Vec:
        Arc::make_mut(&mut self.data).push(value);
        ")
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!("Get mut ref and pop")
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        todo!("Return self.data.get(index)")
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        todo!("
        Get mutable reference:
        Arc::make_mut(&mut self.data).get_mut(index)
        ")
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        todo!("Return self.data.iter()")
    }

    pub fn into_vec(self) -> Vec<T> {
        todo!("
        Try to unwrap Arc:
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())

        If successful (not shared), returns Vec without clone.
        If shared, clones the Vec.
        ")
    }
}

impl<T: Clone> From<Vec<T>> for CowVec<T> {
    fn from(vec: Vec<T>) -> Self {
        CowVec {
            data: Arc::new(vec),
        }
    }
}

impl<T: Clone> Index<usize> for CowVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T: Clone> IndexMut<usize> for CowVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        todo!("
        Get mutable reference via Arc::make_mut:
        &mut Arc::make_mut(&mut self.data)[index]
        ")
    }
}
```

---

## Milestone 3: CoW HashMap with Lazy Copying

**Goal:** Implement CoW for HashMap to enable efficient configuration sharing.

### Introduction

**Why Milestone 2 Isn't Enough:**

HashMaps are more complex than Vecs:
1. **Key-value pairs**: Must handle both
2. **No indexing**: Use `get()` and `insert()`
3. **Iteration**: Over keys, values, or pairs
4. **Entry API**: Complex mutable access pattern

**Real-world scenario:** Web server configuration:
```rust
// Load config once
let config: CowHashMap<String, String> = load_config();

// Each request handler gets cheap clone
for request in requests {
    let cfg = config.clone(); // O(1)
    handle_request(request, cfg);
}

// Admin updates config (triggers copy)
let mut new_config = config.clone();
new_config.insert("feature_flag".into(), "enabled".into());
```

**Performance Benefit:**
- 1000 concurrent requests × 1KB config = 1MB with CoW
- 1000 concurrent requests × 1KB config = 1GB without CoW
- **1000x memory savings!**

### Architecture

```rust
use std::sync::Arc;
use std::collections::HashMap;

pub struct CowHashMap<K, V> {
    data: Arc<HashMap<K, V>>,
}
```

**Challenges:**
- Entry API (`entry().or_insert()`) needs mutable access
- Iteration should not trigger copy
- `insert()` returns old value

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_map() {
        let mut map = CowHashMap::new();
        map.insert("key".to_string(), 42);

        assert_eq!(map.get("key"), Some(&42));
    }

    #[test]
    fn test_clone_map() {
        let mut m1 = CowHashMap::new();
        m1.insert("a".to_string(), 1);
        m1.insert("b".to_string(), 2);

        let m2 = m1.clone();

        assert_eq!(m1.strong_count(), 2);
        assert_eq!(m1.get("a"), Some(&1));
        assert_eq!(m2.get("a"), Some(&1));
    }

    #[test]
    fn test_copy_on_insert() {
        let mut m1 = CowHashMap::new();
        m1.insert("shared".to_string(), 100);

        let mut m2 = m1.clone();
        assert_eq!(m1.strong_count(), 2);

        m2.insert("new".to_string(), 200);

        // m2 copied
        assert!(m1.get("new").is_none());
        assert_eq!(m2.get("new"), Some(&200));
        assert_eq!(m1.strong_count(), 1);
    }

    #[test]
    fn test_copy_on_remove() {
        let mut m1 = CowHashMap::new();
        m1.insert("key".to_string(), 42);

        let mut m2 = m1.clone();

        m2.remove("key");

        assert_eq!(m1.get("key"), Some(&42));
        assert!(m2.get("key").is_none());
    }

    #[test]
    fn test_iter_no_copy() {
        let mut m1 = CowHashMap::new();
        m1.insert("a".to_string(), 1);
        m1.insert("b".to_string(), 2);

        let m2 = m1.clone();

        // Iteration doesn't copy
        let sum: i32 = m1.values().sum();
        assert_eq!(sum, 3);

        assert_eq!(m1.strong_count(), 2);
    }

    #[test]
    fn test_contains_key() {
        let mut map = CowHashMap::new();
        map.insert("exists".to_string(), 1);

        assert!(map.contains_key("exists"));
        assert!(!map.contains_key("missing"));
    }

    #[test]
    fn test_from_hashmap() {
        let mut hm = HashMap::new();
        hm.insert("a".to_string(), 1);
        hm.insert("b".to_string(), 2);

        let cow = CowHashMap::from(hm);

        assert_eq!(cow.len(), 2);
        assert_eq!(cow.get("a"), Some(&1));
    }

    #[test]
    fn test_into_hashmap() {
        let mut cow = CowHashMap::new();
        cow.insert("a".to_string(), 1);

        let hm = cow.into_hashmap();
        assert_eq!(hm.get("a"), Some(&1));
    }

    #[test]
    fn test_keys_values() {
        let mut map = CowHashMap::new();
        map.insert("x".to_string(), 10);
        map.insert("y".to_string(), 20);

        let keys: Vec<_> = map.keys().collect();
        let values: Vec<_> = map.values().collect();

        assert_eq!(keys.len(), 2);
        assert_eq!(values.len(), 2);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
pub struct CowHashMap<K, V> {
    data: Arc<HashMap<K, V>>,
}

impl<K, V> CowHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        CowHashMap {
            data: Arc::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        todo!("Return self.data.get(key)")
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        todo!("
        Get mutable HashMap and insert:
        Arc::make_mut(&mut self.data).insert(key, value)
        ")
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        todo!("Get mut HashMap and remove")
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.values()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.data.iter()
    }

    pub fn into_hashmap(self) -> HashMap<K, V> {
        todo!("
        Try to unwrap Arc:
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
        ")
    }
}

impl<K, V> From<HashMap<K, V>> for CowHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn from(map: HashMap<K, V>) -> Self {
        CowHashMap {
            data: Arc::new(map),
        }
    }
}

impl<K, V> Default for CowHashMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Milestone 4: Generic CoW Wrapper

**Goal:** Create a generic `Cow<T>` wrapper that works with any cloneable type.

### Introduction

**Why Milestone 3 Isn't Enough:**

We've implemented CoW for String, Vec, and HashMap separately:
- Lots of code duplication
- Hard to add new types
- Inconsistent API

**Solution:** Generic wrapper `Cow<T>` that works for any `T: Clone`.

**Benefits:**
- Works with any type (String, Vec, HashMap, custom structs)
- Consistent API
- Less code to maintain

**Challenge:** How to provide mutable access? We can't implement `DerefMut` for all types.

### Architecture

```rust
use std::sync::Arc;

pub struct Cow<T: Clone> {
    data: Arc<T>,
}
```

**API Design:**
- `Cow::new(value)`: Create from value
- `clone()`: Cheap refcount increment
- `make_mut() -> &mut T`: Get mutable ref, copying if shared
- `is_shared() -> bool`: Check if data is shared
- `into_inner() -> T`: Consume and get inner value

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_cow_string() {
        let s1 = Cow::new(String::from("hello"));
        let mut s2 = s1.clone();

        assert_eq!(s1.strong_count(), 2);

        s2.make_mut().push_str(" world");

        assert_eq!(&**s1, "hello");
        assert_eq!(&**s2, "hello world");
    }

    #[test]
    fn test_cow_vec() {
        let v1 = Cow::new(vec![1, 2, 3]);
        let mut v2 = v1.clone();

        v2.make_mut().push(4);

        assert_eq!(&**v1, &[1, 2, 3]);
        assert_eq!(&**v2, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_cow_hashmap() {
        let mut map = HashMap::new();
        map.insert("a", 1);

        let m1 = Cow::new(map);
        let mut m2 = m1.clone();

        m2.make_mut().insert("b", 2);

        assert_eq!(m1.get("b"), None);
        assert_eq!(m2.get("b"), Some(&2));
    }

    #[test]
    fn test_custom_struct() {
        #[derive(Clone, PartialEq, Debug)]
        struct Config {
            host: String,
            port: u16,
        }

        let c1 = Cow::new(Config {
            host: "localhost".into(),
            port: 8080,
        });

        let mut c2 = c1.clone();

        c2.make_mut().port = 9090;

        assert_eq!(c1.port, 8080);
        assert_eq!(c2.port, 9090);
    }

    #[test]
    fn test_make_mut_exclusive() {
        let mut cow = Cow::new(vec![1, 2, 3]);

        // Not shared - no copy
        let ptr1 = cow.data.as_ptr();
        cow.make_mut().push(4);
        let ptr2 = cow.data.as_ptr();

        assert_eq!(ptr1, ptr2); // Same allocation
    }

    #[test]
    fn test_into_inner() {
        let cow = Cow::new(String::from("test"));
        let s = cow.into_inner();

        assert_eq!(s, "test");
    }

    #[test]
    fn test_into_inner_shared() {
        let cow1 = Cow::new(vec![1, 2, 3]);
        let cow2 = cow1.clone();

        // Must clone because shared
        let vec = cow1.into_inner();

        assert_eq!(vec, vec![1, 2, 3]);
        assert_eq!(&**cow2, &[1, 2, 3]);
    }

    #[test]
    fn test_map() {
        let cow = Cow::new(5);
        let mapped = cow.map(|n| n * 2);

        assert_eq!(*mapped, 10);
    }

    #[test]
    fn test_map_shared() {
        let c1 = Cow::new(10);
        let c2 = c1.clone();

        let c3 = c1.map(|n| n + 1);

        assert_eq!(*c1, 10);
        assert_eq!(*c2, 10);
        assert_eq!(*c3, 11);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub struct Cow<T: Clone> {
    data: Arc<T>,
}

impl<T: Clone> Cow<T> {
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        todo!("
        Use Arc::make_mut:
        Arc::make_mut(&mut self.data)

        This automatically:
        - Returns &mut T if strong_count == 1
        - Clones and returns &mut T if shared
        ")
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn into_inner(self) -> T {
        todo!("
        Try to unwrap Arc:
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
        ")
    }

    pub fn map<F, U>(&self, f: F) -> Cow<U>
    where
        F: FnOnce(&T) -> U,
        U: Clone,
    {
        todo!("
        Apply function to inner value:
        let result = f(&*self.data);
        Cow::new(result)
        ")
    }

    pub fn get(&self) -> &T {
        &self.data
    }
}

impl<T: Clone> Deref for Cow<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Clone> From<T> for Cow<T> {
    fn from(value: T) -> Self {
        Cow::new(value)
    }
}
```

---

## Milestone 5: Thread-Safe CoW with Arc (Already Thread-Safe!)

**Goal:** Verify and optimize thread-safe sharing of CoW structures.

### Introduction

**Why Milestone 4 is Almost Enough:**

Good news: `Cow<T>` using `Arc` is already thread-safe if `T: Send + Sync`!

However, we need to:
1. **Verify safety**: Add Send/Sync bounds
2. **Add utilities**: Thread-safe modification helpers
3. **Optimize**: Reduce contention on writes
4. **Document**: Clear thread-safety guarantees

**Thread-safety properties:**
- Multiple threads can clone and read simultaneously
- Writes are safe (each thread gets private copy)
- No locks needed for reads (unlike Mutex)
- Lock-free for read-heavy workloads

### Architecture

```rust
use std::sync::Arc;
use std::marker::PhantomData;

pub struct Cow<T: Clone + Send + Sync> {
    data: Arc<T>,
    _marker: PhantomData<T>,
}
```

**Thread-safety guarantees:**
- `Clone`: Lock-free atomic refcount increment
- `Deref`: Lock-free read access
- `make_mut`: Clones if shared (no waiting)
- No deadlocks possible

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;

    #[test]
    fn test_concurrent_clone() {
        let cow = Cow::new(vec![1, 2, 3, 4, 5]);
        let mut handles = vec![];

        for _ in 0..10 {
            let c = cow.clone();
            let handle = thread::spawn(move || {
                let sum: i32 = c.iter().sum();
                sum
            });
            handles.push(handle);
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 15);
        }
    }

    #[test]
    fn test_concurrent_read() {
        let cow = Cow::new(String::from("shared data"));
        let mut handles = vec![];

        for i in 0..20 {
            let c = cow.clone();
            let handle = thread::spawn(move || {
                assert_eq!(&*c, "shared data");
                c.len()
            });
            handles.push(handle);
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 11);
        }
    }

    #[test]
    fn test_concurrent_write() {
        let cow = Cow::new(vec![1, 2, 3]);
        let mut handles = vec![];

        // 10 threads each make their own modification
        for i in 0..10 {
            let mut c = cow.clone();
            let handle = thread::spawn(move || {
                c.make_mut().push(i);
                c.clone()
            });
            handles.push(handle);
        }

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        // Each thread got its own copy
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.len(), 4);
            assert_eq!(result[3], i);
        }

        // Original unchanged
        assert_eq!(&*cow, &[1, 2, 3]);
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<Cow<Vec<i32>>>();
        assert_sync::<Cow<Vec<i32>>>();
    }

    #[test]
    fn test_shared_config() {
        use std::collections::HashMap;

        let mut config = HashMap::new();
        config.insert("workers", 4);
        config.insert("timeout", 30);

        let cow_config = Cow::new(config);
        let mut handles = vec![];

        // 100 worker threads using shared config
        for worker_id in 0..100 {
            let cfg = cow_config.clone();
            let handle = thread::spawn(move || {
                let workers = cfg.get("workers").unwrap();
                let timeout = cfg.get("timeout").unwrap();
                (*workers, *timeout, worker_id)
            });
            handles.push(handle);
        }

        for handle in handles {
            let (workers, timeout, _id) = handle.join().unwrap();
            assert_eq!(workers, 4);
            assert_eq!(timeout, 30);
        }

        // Still shared!
        assert_eq!(cow_config.strong_count(), 1);
    }

    #[test]
    fn test_memory_efficiency() {
        use std::mem::size_of;

        let vec = vec![0u8; 1_000_000]; // 1MB
        let cow1 = Cow::new(vec);

        // Clone 100 times
        let clones: Vec<_> = (0..100).map(|_| cow1.clone()).collect();

        // Memory used: ~1MB data + 100 * 8 bytes = ~1MB
        // vs 100MB if each clone copied

        assert_eq!(cow1.strong_count(), 101);

        // Size of Cow itself
        assert_eq!(size_of::<Cow<Vec<u8>>>(), size_of::<Arc<Vec<u8>>>());
    }

    #[test]
    fn test_update_check() {
        let counter = StdArc::new(AtomicUsize::new(0));
        let cow = Cow::new(vec![1, 2, 3]);

        let mut handles = vec![];

        for _ in 0..50 {
            let mut c = cow.clone();
            let cnt = counter.clone();

            let handle = thread::spawn(move || {
                // Modify triggers copy
                c.make_mut().push(4);
                cnt.fetch_add(1, Ordering::SeqCst);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.load(Ordering::SeqCst), 50);
        // Original still unchanged
        assert_eq!(&*cow, &[1, 2, 3]);
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::ops::Deref;

#[derive(Clone)]
pub struct Cow<T>
where
    T: Clone + Send + Sync,
{
    data: Arc<T>,
}

// Explicitly implement Send + Sync
unsafe impl<T: Clone + Send + Sync> Send for Cow<T> {}
unsafe impl<T: Clone + Send + Sync> Sync for Cow<T> {}

impl<T> Cow<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        Arc::make_mut(&mut self.data)
    }

    pub fn try_update<F, E>(&mut self, f: F) -> Result<(), E>
    where
        F: FnOnce(&mut T) -> Result<(), E>,
    {
        todo!("
        Apply function to mutable reference:
        f(self.make_mut())
        ")
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        todo!("Apply function to make_mut()")
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        todo!("Use Arc::ptr_eq to check if both point to same data")
    }
}

impl<T> Deref for Cow<T>
where
    T: Clone + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<T> for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn from(value: T) -> Self {
        Cow::new(value)
    }
}
```

---

## Milestone 6: Performance Tracking and Optimization

**Goal:** Add metrics to track copy frequency and optimize hot paths.

### Introduction

**Why Milestone 5 Isn't Enough:**

Production CoW structures need observability:
1. **Copy tracking**: How often does copy-on-write trigger?
2. **Sharing metrics**: What's the sharing ratio?
3. **Memory profiling**: Is CoW actually saving memory?
4. **Performance validation**: Is CoW faster than clone?

**Metrics to track:**
- Total clones (refcount increments)
- Actual copies (data duplicated)
- Copy rate = copies / clones
- Memory saved = (clones - copies) × size
- Strong count distribution

### Architecture

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Cow<T: Clone + Send + Sync> {
    data: Arc<T>,
    stats: Arc<CowStats>,
}

struct CowStats {
    clones: AtomicUsize,
    copies: AtomicUsize,
}
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_stats() {
        let cow = Cow::new(vec![1, 2, 3]);

        let _c1 = cow.clone();
        let _c2 = cow.clone();
        let _c3 = cow.clone();

        let stats = cow.stats();
        assert_eq!(stats.clones, 3);
        assert_eq!(stats.copies, 0);
    }

    #[test]
    fn test_copy_stats() {
        let cow = Cow::new(vec![1, 2, 3]);

        let mut c1 = cow.clone();
        let mut c2 = cow.clone();

        c1.make_mut().push(4);
        c2.make_mut().push(5);

        let stats = cow.stats();
        assert_eq!(stats.clones, 2);
        assert_eq!(stats.copies, 2);
        assert_eq!(stats.copy_rate(), 1.0);
    }

    #[test]
    fn test_copy_rate() {
        let cow = Cow::new(String::from("test"));

        // 10 clones
        let clones: Vec<_> = (0..10).map(|_| cow.clone()).collect();

        // 5 copies
        let mut mutated: Vec<_> = clones.into_iter().take(5).collect();
        for c in &mut mutated {
            c.make_mut().push_str("!");
        }

        let stats = cow.stats();
        assert_eq!(stats.clones, 10);
        assert_eq!(stats.copies, 5);
        assert_eq!(stats.copy_rate(), 0.5);
    }

    #[test]
    fn test_memory_savings() {
        use std::mem::size_of_val;

        let data = vec![0u8; 1_000_000]; // 1MB
        let size = size_of_val(&*data);

        let cow = Cow::new(data);

        // 100 clones
        let _clones: Vec<_> = (0..100).map(|_| cow.clone()).collect();

        let stats = cow.stats();
        let saved = stats.memory_saved(size);

        // Saved = (100 clones - 0 copies) * 1MB = 100MB
        assert_eq!(saved, 100 * 1_000_000);
    }

    #[test]
    fn test_stats_report() {
        let cow = Cow::new(vec![0u8; 1024]);

        let mut clones: Vec<_> = (0..10).map(|_| cow.clone()).collect();

        for i in 0..5 {
            clones[i].make_mut().push(1);
        }

        let report = cow.stats_report(1024);
        assert!(report.contains("Clones:"));
        assert!(report.contains("Copies:"));
        assert!(report.contains("Copy rate:"));
        assert!(report.contains("Memory saved:"));
    }
}
```

### Starter Code

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Deref;

#[derive(Clone)]
pub struct Cow<T>
where
    T: Clone + Send + Sync,
{
    data: Arc<T>,
    stats: Arc<CowStats>,
}

struct CowStats {
    clones: AtomicUsize,
    copies: AtomicUsize,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub clones: usize,
    pub copies: usize,
}

impl Stats {
    pub fn copy_rate(&self) -> f64 {
        todo!("Calculate copies / clones (handle division by zero)")
    }

    pub fn memory_saved(&self, item_size: usize) -> usize {
        todo!("Calculate (clones - copies) * item_size")
    }
}

impl<T> Cow<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
            stats: Arc::new(CowStats {
                clones: AtomicUsize::new(0),
                copies: AtomicUsize::new(0),
            }),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        todo!("
        Check if shared before calling Arc::make_mut:
        let was_shared = self.is_shared();
        let result = Arc::make_mut(&mut self.data);

        if was_shared {
            self.stats.copies.fetch_add(1, Ordering::Relaxed);
        }

        result
        ")
    }

    pub fn stats(&self) -> Stats {
        Stats {
            clones: self.stats.clones.load(Ordering::Relaxed),
            copies: self.stats.copies.load(Ordering::Relaxed),
        }
    }

    pub fn stats_report(&self, item_size: usize) -> String {
        todo!("
        Format stats into string:
        - Clones: {}
        - Copies: {}
        - Copy rate: {:.1}%
        - Memory saved: {} bytes ({:.1} MB)
        ")
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }
}

impl<T> Clone for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn clone(&self) -> Self {
        todo!("
        Increment clone counter:
        self.stats.clones.fetch_add(1, Ordering::Relaxed);

        Clone data Arc and stats Arc:
        Cow {
            data: self.data.clone(),
            stats: self.stats.clone(),
        }
        ")
    }
}

impl<T> Deref for Cow<T>
where
    T: Clone + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
```

---

## Complete Working Example

Here's a production-quality CoW implementation with full feature set:

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Deref;
use std::fmt;

// ============================================================================
// Statistics Tracking
// ============================================================================

struct CowStats {
    clones: AtomicUsize,
    copies: AtomicUsize,
}

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub clones: usize,
    pub copies: usize,
}

impl Stats {
    pub fn copy_rate(&self) -> f64 {
        if self.clones == 0 {
            0.0
        } else {
            self.copies as f64 / self.clones as f64
        }
    }

    pub fn memory_saved(&self, item_size: usize) -> usize {
        if self.copies >= self.clones {
            0
        } else {
            (self.clones - self.copies) * item_size
        }
    }
}

// ============================================================================
// Copy-on-Write Wrapper
// ============================================================================

pub struct Cow<T>
where
    T: Clone + Send + Sync,
{
    data: Arc<T>,
    stats: Arc<CowStats>,
}

impl<T> Cow<T>
where
    T: Clone + Send + Sync,
{
    pub fn new(value: T) -> Self {
        Cow {
            data: Arc::new(value),
            stats: Arc::new(CowStats {
                clones: AtomicUsize::new(0),
                copies: AtomicUsize::new(0),
            }),
        }
    }

    pub fn make_mut(&mut self) -> &mut T {
        let was_shared = self.is_shared();
        let result = Arc::make_mut(&mut self.data);

        if was_shared {
            self.stats.copies.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    pub fn update<F>(&mut self, f: F)
    where
        F: FnOnce(&mut T),
    {
        f(self.make_mut());
    }

    pub fn is_shared(&self) -> bool {
        Arc::strong_count(&self.data) > 1
    }

    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    pub fn into_inner(self) -> T {
        Arc::try_unwrap(self.data)
            .unwrap_or_else(|arc| (*arc).clone())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    pub fn stats(&self) -> Stats {
        Stats {
            clones: self.stats.clones.load(Ordering::Relaxed),
            copies: self.stats.copies.load(Ordering::Relaxed),
        }
    }

    pub fn stats_report(&self, item_size: usize) -> String {
        let stats = self.stats();
        format!(
            "CoW Statistics:\n\
             - Clones: {}\n\
             - Copies: {}\n\
             - Copy rate: {:.1}%\n\
             - Memory saved: {} bytes ({:.2} MB)",
            stats.clones,
            stats.copies,
            stats.copy_rate() * 100.0,
            stats.memory_saved(item_size),
            stats.memory_saved(item_size) as f64 / 1_000_000.0
        )
    }
}

impl<T> Clone for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn clone(&self) -> Self {
        self.stats.clones.fetch_add(1, Ordering::Relaxed);

        Cow {
            data: self.data.clone(),
            stats: self.stats.clone(),
        }
    }
}

impl<T> Deref for Cow<T>
where
    T: Clone + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> From<T> for Cow<T>
where
    T: Clone + Send + Sync,
{
    fn from(value: T) -> Self {
        Cow::new(value)
    }
}

impl<T> fmt::Debug for Cow<T>
where
    T: Clone + Send + Sync + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cow")
            .field("data", &*self.data)
            .field("shared", &self.is_shared())
            .field("strong_count", &self.strong_count())
            .finish()
    }
}

unsafe impl<T: Clone + Send + Sync> Send for Cow<T> {}
unsafe impl<T: Clone + Send + Sync> Sync for Cow<T> {}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    use std::collections::HashMap;
    use std::thread;

    println!("=== CoW String Example ===\n");

    let s1 = Cow::new(String::from("Hello, CoW!"));
    println!("Created: {:?}", s1);

    let s2 = s1.clone();
    let s3 = s1.clone();
    println!("Cloned 2 times, shared: {}", s1.is_shared());

    let mut s4 = s1.clone();
    s4.make_mut().push_str(" - Modified");
    println!("Modified clone: {}", s4);
    println!("Original: {}\n", s1);

    println!("{}\n", s1.stats_report(s1.len()));

    println!("=== CoW Vec Example ===\n");

    let v = Cow::new(vec![1, 2, 3, 4, 5]);

    // Share across 10 threads
    let mut handles = vec![];

    for i in 0..10 {
        let mut vc = v.clone();

        let handle = thread::spawn(move || {
            if i % 2 == 0 {
                // Even threads modify (triggers copy)
                vc.make_mut().push(i * 10);
            }
            vc.iter().sum::<i32>()
        });

        handles.push(handle);
    }

    for (i, handle) in handles.into_iter().enumerate() {
        let sum = handle.join().unwrap();
        println!("Thread {}: sum = {}", i, sum);
    }

    println!("\nOriginal vec: {:?}", &*v);
    println!("{}\n", v.stats_report(std::mem::size_of::<Vec<i32>>()));

    println!("=== CoW HashMap Config Example ===\n");

    let mut config = HashMap::new();
    config.insert("workers", 4);
    config.insert("timeout", 30);
    config.insert("max_connections", 1000);

    let cow_config = Cow::new(config);

    // Simulate 100 worker threads using config
    let mut handles = vec![];

    for worker_id in 0..100 {
        let cfg = cow_config.clone();

        let handle = thread::spawn(move || {
            let workers = *cfg.get("workers").unwrap();
            // Simulate work
            std::thread::sleep(std::time::Duration::from_micros(10));
            workers
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Config shared across 100 threads!");
    println!("{}", cow_config.stats_report(std::mem::size_of::<HashMap<&str, i32>>()));

    println!("\nDone!");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_workflow() {
        let v1 = Cow::new(vec![1, 2, 3]);
        let v2 = v1.clone();
        let mut v3 = v1.clone();

        assert_eq!(v1.strong_count(), 3);

        v3.make_mut().push(4);

        assert_eq!(&*v1, &[1, 2, 3]);
        assert_eq!(&*v2, &[1, 2, 3]);
        assert_eq!(&*v3, &[1, 2, 3, 4]);

        let stats = v1.stats();
        assert_eq!(stats.clones, 2);
        assert_eq!(stats.copies, 1);
    }
}
```

**Example Output:**
```
=== CoW String Example ===

Created: Cow { data: "Hello, CoW!", shared: false, strong_count: 1 }
Cloned 2 times, shared: true
Modified clone: Hello, CoW! - Modified
Original: Hello, CoW!

CoW Statistics:
- Clones: 3
- Copies: 1
- Copy rate: 33.3%
- Memory saved: 22 bytes (0.00 MB)

=== CoW Vec Example ===

Thread 0: sum = 15
Thread 1: sum = 15
Thread 2: sum = 35
Thread 3: sum = 15
Thread 4: sum = 55
Thread 5: sum = 15
Thread 6: sum = 75
Thread 7: sum = 15
Thread 8: sum = 95
Thread 9: sum = 15

Original vec: [1, 2, 3, 4, 5]
CoW Statistics:
- Clones: 10
- Copies: 5
- Copy rate: 50.0%
- Memory saved: 120 bytes (0.00 MB)

=== CoW HashMap Config Example ===

Config shared across 100 threads!
CoW Statistics:
- Clones: 100
- Copies: 0
- Copy rate: 0.0%
- Memory saved: 4800 bytes (0.00 MB)

Done!
```

---

## Summary

You've built a complete Copy-on-Write library with production-grade features!

### Features Implemented
1. ✅ CoW String (Milestone 1)
2. ✅ CoW Vec (Milestone 2)
3. ✅ CoW HashMap (Milestone 3)
4. ✅ Generic Cow<T> (Milestone 4)
5. ✅ Thread-safe sharing (Milestone 5)
6. ✅ Performance tracking (Milestone 6)

### Smart Pointer Patterns Used
- `Arc<T>`: Atomic reference counting for thread-safe sharing
- `Arc::make_mut()`: Copy-on-write primitive
- `Arc::try_unwrap()`: Extract value without copy if possible
- `Arc::strong_count()`: Check sharing status
- `Deref`: Transparent read access

### Performance Characteristics
| Operation | Normal Clone | CoW Clone | Speedup |
|-----------|-------------|-----------|---------|
| 1KB string | 1μs | 10ns | 100x |
| 1MB buffer | 500μs | 10ns | 50,000x |
| HashMap | 5μs | 10ns | 500x |
| Modify after clone | 0 | Copy cost | N/A |

### When to Use CoW
✅ **Use CoW when:**
- Read-heavy workloads (10:1 read:write ratio)
- Sharing data across threads
- Implementing immutable data structures
- Cloning large data structures
- Version control systems

❌ **Don't use CoW when:**
- Write-heavy workloads (copies negate benefits)
- Data is always modified after clone
- Small data (clone cost negligible)
- Need guaranteed O(1) writes

### Real-World Uses
- **Git**: Blob storage with content-addressable CoW
- **Immutable.js**: Persistent data structures
- **Rust std::borrow::Cow**: Standard library CoW
- **Arc<RwLock<T>>**: Common Rust pattern
- **im crate**: Immutable collections

### Key Lessons
1. **Arc::make_mut is magic**: Automatic copy detection
2. **Read-heavy wins**: CoW excels with rare writes
3. **Memory vs speed**: Trade write speed for memory efficiency
4. **Thread-safety**: Arc makes CoW naturally thread-safe
5. **Measure impact**: Use stats to validate performance

Congratulations! You understand the CoW patterns used in functional programming languages, version control systems, and immutable data structures!
