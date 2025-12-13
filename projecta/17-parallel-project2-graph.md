
# Parallel Graph Processing Engine

### Problem Statement

Build a high-performance parallel graph processing engine that handles irregular workloads and dynamic work generation. Implement fundamental graph algorithms (BFS, shortest path, PageRank, connected components) with efficient parallelization strategies for graphs with millions of vertices and edges, achieving 5-10x speedup despite irregular workload distribution.

The system must:
- Represent graphs efficiently (adjacency lists, CSR format)
- Handle skewed degree distributions (power-law graphs)
- Implement level-synchronous BFS with frontier expansion
- Compute shortest paths and PageRank iteratively
- Use atomic operations to avoid race conditions
- Achieve work stealing for load balancing
- Process graphs with 1M+ vertices and 10M+ edges

### Use Cases

- **Social Networks**: Friend recommendations, influence analysis (Facebook, Twitter graphs)
- **Web Search**: PageRank for ranking web pages (Google's original algorithm)
- **Route Planning**: GPS navigation, delivery optimization (road networks)
- **Network Analysis**: Internet topology, protein interaction networks
- **Fraud Detection**: Transaction graph analysis, money laundering detection
- **Recommendation Systems**: Product graphs, collaborative filtering

### Why It Matters

**Graph Characteristics:**
```
Social network (power-law distribution):
- Average degree: 50
- Max degree: 10,000+ (celebrities)
- 99% vertices: < 100 edges
- 1% vertices: > 1,000 edges

This creates massive load imbalance!
```

**Sequential vs Parallel:**
```
BFS on 1M vertex graph:
Sequential: 500ms (depth-first, cache-friendly)
Naive parallel: 1000ms (worse! overhead dominates)
Optimized parallel: 80ms (6x speedup with frontier-based)
```

**Why Irregular Parallelism is Hard:**
1. **Load imbalance**: Some threads finish instantly, others take forever
2. **Dynamic work**: Frontier grows and shrinks unpredictably
3. **Memory contention**: Atomic updates to shared frontier
4. **Cache behavior**: Random access pattern (poor locality)

**Real-World Performance:**
- GraphLab (CMU): 10-100x speedup on PageRank
- Ligra (MIT): 5-20x speedup on graph traversal
- Galois (UT Austin): Work stealing for irregular graphs

**Amdahl's Law Challenge:**
For graphs where 10% of vertices have 90% of edges, the sequential bottleneck (processing high-degree vertices) limits speedup to ~10x even with infinite cores.

---

## Key Concepts Explained

### 1. Graph Representations: Adjacency List vs CSR

**What Is It?**
Graphs can be stored in different formats, each with trade-offs for memory usage, cache efficiency, and mutability.

**Adjacency List:**
Each vertex stores a dynamic vector of its neighbors:
```rust
// Flexible but pointer-heavy
struct Graph {
    adjacency: Vec<Vec<usize>>,  // Vec of Vecs
}

// Memory layout (scattered):
// adjacency[0] → heap: [1, 2, 5]
// adjacency[1] → heap: [3, 4]
// adjacency[2] → heap: [0, 6, 7, 8]
```

**Pros:**
- Easy to modify (add/remove edges)
- Simple implementation
- Natural fit for dynamic graphs

**Cons:**
- Pointer indirection (cache misses)
- Memory overhead (each Vec has capacity, length, pointer)
- Poor cache locality (neighbors scattered in heap)

**Compressed Sparse Row (CSR):**
Flatten all edges into one array, use offsets to mark boundaries:
```rust
struct GraphCSR {
    offsets: Vec<usize>,  // offsets[v] = start of v's neighbors
    edges: Vec<usize>,    // All edges in one flat array
}

// Same graph in CSR:
// offsets: [0, 3, 5, 9]
// edges:   [1, 2, 5, 3, 4, 0, 6, 7, 8]
//          |-------| |---| |-----------|
//          vertex 0  vert1  vertex 2

// neighbors(v) = edges[offsets[v]..offsets[v+1]]
```

**Pros:**
- Cache-friendly (sequential memory access)
- Minimal memory overhead (just two flat arrays)
- 2-3x faster iteration over neighbors
- Used by GraphBLAS, Ligra, GraphChi

**Cons:**
- Immutable (hard to add edges)
- Requires building entire graph first
- More complex implementation

**Performance Comparison:**
```rust
// Benchmark: Iterate over all edges
Graph (adjacency list): 150ms
GraphCSR:               50ms  (3x faster)

// Why? Cache misses:
// Adjacency list: ~40% cache misses (pointer chasing)
// CSR:            ~5% cache misses (sequential access)
```

**When to Use What:**
- **Adjacency List**: Dynamic graphs, frequent edge modifications
- **CSR**: Static graphs, read-heavy workloads, high-performance traversal

---

### 2. Power-Law Distributions and Skewed Degree Distribution

**What Is It?**
Real-world graphs (social networks, web graphs, citation networks) follow power-law degree distributions: most vertices have few edges, a tiny minority have massive connectivity.

**Formal Definition:**
```
P(degree = k) ∝ k^(-α)

Where α is typically 2-3 (power-law exponent)
```

**Visual Example:**
```
Social Network (1M users):

Degree distribution:
|  *
|  *
|  *        Regular graph (everyone has ~50 friends)
|  *        ↓
|  *    * * * * * * * * *
|  * * * * * * * * * * * * * * *
+--------------------------------> Degree
0   10  20  30  40  50  60  70

Power-law graph (realistic):
|*
|*
|* *
|*  *                          ← 0.1% vertices have 10,000+ edges
|*   *  *
|*    *   *   *
|*     *    *    *  *  *  *  * * ← 99% vertices have < 100 edges
+--------------------------------> Degree
0    100       1000      10000

Key insight: Long tail with extreme outliers!
```

**Real-World Examples:**
```
Twitter follower graph:
- Median user: 61 followers
- @BarackObama: 133 million followers (2+ million times median)
- 99.9% users: < 10,000 followers
- 0.1% users: > 100,000 followers

Facebook friend graph:
- Average: 338 friends
- Max: 5,000 (limit enforced)
- Power-law constrained by platform

Web page graph:
- Average in-links: 5-10
- Popular pages: 100,000+ in-links
- Exponent α ≈ 2.1
```

**Why It Matters for Parallelism:**
```rust
// Naive parallel BFS on power-law graph:
Thread 0: Process celebrity vertex (10,000 neighbors) → 50ms
Thread 1: Process normal vertex (30 neighbors) → 0.1ms
Thread 2: Process normal vertex (45 neighbors) → 0.1ms
Thread 3: Process normal vertex (28 neighbors) → 0.1ms

// Result: Thread 0 takes 500x longer!
// Parallel time = max(50ms, 0.1ms) = 50ms
// Wasted resources: Threads 1-3 sit idle for 49.9ms
```

**Load Imbalance Factor:**
```
Imbalance = max_work / avg_work

Power-law graph example:
- 4 threads
- Total edges to process: 10,000
- Thread 0: 9,000 edges (celebrity)
- Thread 1-3: 333 edges each

Imbalance = 9000 / 2500 = 3.6x

This means 3 threads spend 72% of their time idle!
```

**Barabási-Albert Model (Generating Power-Law Graphs):**
```rust
// Preferential attachment algorithm:
fn generate_power_law_graph(n: usize, m: usize) -> Graph {
    let mut graph = Graph::new(n);

    // Start with complete graph on m vertices
    for i in 0..m {
        for j in i+1..m {
            graph.add_edge(i, j);
            graph.add_edge(j, i);
        }
    }

    // Add vertices one at a time
    for v in m..n {
        // Connect to m existing vertices
        // Probability ∝ degree (rich get richer!)
        let degrees: Vec<usize> = (0..v).map(|u| graph.degree(u)).collect();
        let total_degree: usize = degrees.iter().sum();

        for _ in 0..m {
            // Sample proportional to degree
            let target = sample_proportional(&degrees, total_degree);
            graph.add_edge(v, target);
        }
    }

    graph
}

// Key insight: Popular vertices get more connections!
// "Rich get richer" → creates hubs
```

---

### 3. Irregular Parallelism and Load Imbalance

**What Is It?**
Irregular parallelism occurs when tasks have unpredictable workload that can't be divided evenly, common in graph algorithms due to skewed degree distributions.

**Regular vs Irregular Parallelism:**
```rust
// REGULAR: Matrix multiplication (predictable work)
// Each thread does exactly 250,000 iterations
(0..1000).into_par_iter().for_each(|row| {
    for col in 0..1000 {
        c[row][col] = a[row].dot(&b_cols[col]);  // Same work
    }
});

Load balance: Perfect (1.0x imbalance factor)

// IRREGULAR: Graph BFS (unpredictable work)
frontier.par_iter().for_each(|&vertex| {
    for &neighbor in graph.neighbors(vertex) {
        // Degree varies 1 to 10,000!
        visit(neighbor);
    }
});

Load balance: Poor (3-10x imbalance factor typical)
```

**Four Challenges of Irregular Parallelism:**

**1. Static Work Distribution Fails:**
```rust
// Naive: Split vertices evenly
let vertices_per_thread = n / num_threads;

// Thread 0: vertices 0-250 (total degree: 2,500)
// Thread 1: vertices 250-500 (total degree: 95,000) ← Celebrity!
// Thread 2: vertices 500-750 (total degree: 3,200)
// Thread 3: vertices 750-1000 (total degree: 2,800)

// Thread 1 takes 30x longer!
```

**2. Dynamic Work Generation:**
```rust
// BFS frontier size changes unpredictably:
Level 0: [start_vertex]              → 1 vertex
Level 1: neighbors of Level 0        → 50 vertices
Level 2: neighbors of Level 1        → 2,500 vertices (explosion!)
Level 3: neighbors of Level 2        → 180 vertices (decay)
Level 4: neighbors of Level 3        → 8 vertices

// Can't predict frontier size in advance
// Work varies 300x between levels!
```

**3. Memory Contention:**
```rust
// Multiple threads updating shared visited array:
let visited: Vec<AtomicBool> = ...;

// All threads try to mark neighbors as visited:
if !visited[v].swap(true, Ordering::Relaxed) {
    next_frontier.push(v);  // Race condition!
}

// Contention when many threads hit same vertex
// False sharing if vertices map to same cache line
```

**4. Poor Cache Locality:**
```rust
// Random access pattern:
for &vertex in frontier {
    for &neighbor in graph.neighbors(vertex) {
        // Neighbors are random indices → cache misses!
        visit(neighbor);
    }
}

// Sequential algorithm has better locality
// Paradox: Parallel can be slower due to cache misses!
```

**Solutions to Irregular Parallelism:**

**Work Stealing:**
```rust
// Each thread has local work queue
// When idle, steal from busy threads

Thread 0: [v1, v2, ..., v1000]  ← Busy
Thread 1: []                     ← Idle, steal from Thread 0!
Thread 2: [v5001, v5002]
Thread 3: []                     ← Idle, steal from Thread 0!

// Rayon implements work stealing automatically
// Results in better load balance (1.5-2x imbalance typical)
```

**Dynamic Chunking:**
```rust
// Instead of static assignment, use dynamic chunks
frontier.par_iter()
    .with_min_len(64)  // Process at least 64 vertices per task
    .for_each(|&vertex| {
        // Process vertex
    });

// Rayon splits work into chunks
// Threads grab chunks dynamically
// Smaller chunks → better balance, but more overhead
```

**Frontier Compaction:**
```rust
// Remove duplicates and sort frontier
// Better cache locality, less redundant work

let mut next_frontier: Vec<usize> = ...;
next_frontier.sort_unstable();
next_frontier.dedup();

// Reduces frontier size by 30-50% typically
// Sequential access = better cache performance
```

---

### 4. Level-Synchronous BFS and Frontier-Based Algorithms

**What Is It?**
Level-synchronous BFS processes graph in layers, where each layer (frontier) is processed in parallel before moving to the next.

**Traditional BFS (Queue-Based):**
```rust
// Sequential: One vertex at a time
fn bfs_queue(graph: &Graph, start: usize) -> Vec<usize> {
    let mut distances = vec![usize::MAX; graph.num_vertices];
    let mut queue = VecDeque::new();

    distances[start] = 0;
    queue.push_back(start);

    while let Some(u) = queue.pop_front() {
        for &v in graph.neighbors(u) {
            if distances[v] == usize::MAX {
                distances[v] = distances[u] + 1;
                queue.push_back(v);  // Add to end
            }
        }
    }

    distances
}

// Inherently sequential: FIFO order matters
// Can't parallelize queue operations efficiently
```

**Level-Synchronous BFS (Frontier-Based):**
```rust
// Process entire level in parallel
fn bfs_frontier(graph: &Graph, start: usize) -> Vec<usize> {
    let mut distances = vec![usize::MAX; graph.num_vertices];
    let mut frontier = vec![start];
    distances[start] = 0;
    let mut level = 0;

    while !frontier.is_empty() {
        level += 1;

        // PARALLEL: Process all vertices in current frontier
        let next_frontier: Vec<usize> = frontier
            .par_iter()
            .flat_map(|&u| {
                graph.neighbors(u)
                    .iter()
                    .filter(|&&v| distances[v] == usize::MAX)
                    .map(|&v| {
                        distances[v] = level;
                        v
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        frontier = next_frontier;
    }

    distances
}

// Key: All vertices in frontier are independent!
// Can process them in any order, in parallel
```

**Visual Comparison:**
```
Graph:
    0
   /|\
  1 2 3
  |\ \|
  4 5 6

Queue-based BFS order:
Step 1: Process 0 → enqueue [1,2,3]
Step 2: Process 1 → enqueue [1,2,3,4,5]
Step 3: Process 2 → enqueue [1,2,3,4,5,5,6]
Step 4: Process 3 → enqueue [1,2,3,4,5,5,6,6]
...
(Sequential, one at a time)

Frontier-based BFS:
Level 0: [0]           ← Process in parallel (1 thread)
Level 1: [1, 2, 3]     ← Process in parallel (3 threads)
Level 2: [4, 5, 6]     ← Process in parallel (3 threads)

Synchronization barrier between levels!
```

**Frontier Growth Patterns:**
```
Small-world graph (6 degrees of separation):
Level:    0    1    2     3       4      5     6
Size:     1    50   2500  125000  500000 50000 1000

Explosive growth then rapid decay
Peak at "middle" of graph (diameter/2)

Tree-like graph (balanced binary tree):
Level:    0    1   2   3    4    5    6
Size:     1    2   4   8   16   32   64

Steady exponential growth
Predictable pattern

Grid graph (2D lattice):
Level:    0    1    2    3    4     5
Size:     1    4    8   12   16    20

Linear growth (perimeter of square)
```

**Race Condition and Solution:**
```rust
// WRONG: Race condition on visited array
fn process_frontier_wrong(frontier: &[usize]) -> Vec<usize> {
    let mut visited = vec![false; n];

    frontier.par_iter().flat_map(|&u| {
        graph.neighbors(u).iter().filter_map(|&v| {
            if !visited[v] {  // ← Read
                visited[v] = true;  // ← Write (DATA RACE!)
                Some(v)
            } else {
                None
            }
        }).collect::<Vec<_>>()
    }).collect()
}

// RIGHT: Use atomic test-and-set
fn process_frontier_correct(frontier: &[usize]) -> Vec<usize> {
    let visited: Vec<AtomicBool> = (0..n)
        .map(|_| AtomicBool::new(false))
        .collect();

    frontier.par_iter().flat_map(|&u| {
        graph.neighbors(u).iter().filter_map(|&v| {
            // Atomic swap: test-and-set in one operation
            if !visited[v].swap(true, Ordering::Relaxed) {
                Some(v)
            } else {
                None
            }
        }).collect::<Vec<_>>()
    }).collect()
}
```

**Performance Trade-offs:**
```
Sequential BFS:
+ Better cache locality (sequential queue access)
+ No synchronization overhead
+ Simpler code
- Single-threaded (slow on large graphs)

Parallel Level-Synchronous BFS:
+ Exploits parallelism within each level
+ Scales well on large frontiers
- Synchronization barrier between levels
- Atomic operations overhead (10-20ns each)
- Worse cache behavior (random access)

Speedup depends on frontier size:
Small frontiers (<100):  0.5-1x (overhead dominates)
Medium frontiers (1k-100k): 3-6x
Large frontiers (>100k): 6-10x
```

---

### 5. Atomic Operations for Concurrent Graph Updates

**What Is It?**
Graph algorithms require thread-safe updates to shared state (visited flags, distances, component IDs). Atomic operations provide lock-free synchronization.

**Key Atomic Types for Graphs:**
```rust
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Visited tracking (BFS, DFS)
let visited: Vec<AtomicBool> = (0..n)
    .map(|_| AtomicBool::new(false))
    .collect();

// Distance arrays (shortest path)
let distances: Vec<AtomicU32> = (0..n)
    .map(|_| AtomicU32::new(u32::MAX))
    .collect();

// Component IDs (union-find)
let parent: Vec<AtomicUsize> = (0..n)
    .map(|i| AtomicUsize::new(i))
    .collect();
```

**Atomic Test-and-Set Pattern:**
```rust
// BFS: Mark vertex as visited atomically
if !visited[v].swap(true, Ordering::Relaxed) {
    // We're first to visit v!
    next_frontier.push(v);
} else {
    // Already visited by another thread
}

// Hardware implementation (x86):
// LOCK XCHG instruction (atomic exchange)
// Takes ~20-30 CPU cycles (10-15ns)
```

**Compare-and-Swap for Distance Updates:**
```rust
// Shortest path: Update distance if shorter
fn try_update_distance(dist: &AtomicU32, new_dist: u32) -> bool {
    loop {
        let current = dist.load(Ordering::Relaxed);

        if new_dist >= current {
            return false;  // Not an improvement
        }

        // Try to update (CAS loop)
        match dist.compare_exchange_weak(
            current,
            new_dist,
            Ordering::Relaxed,
            Ordering::Relaxed
        ) {
            Ok(_) => return true,   // Success!
            Err(_) => continue,     // Retry (another thread updated)
        }
    }
}

// Usage in parallel Dijkstra/delta-stepping:
for &(neighbor, weight) in graph.neighbors(u) {
    let new_dist = current_dist + weight;
    if try_update_distance(&distances[neighbor], new_dist) {
        // Updated distance, add to bucket
        buckets[bucket_index(new_dist)].push(neighbor);
    }
}
```

**Memory Ordering for Graphs:**
```rust
// Most graph algorithms can use Relaxed ordering:
visited[v].store(true, Ordering::Relaxed);

// Why? Correctness doesn't depend on ordering
// between different atomic operations

// Example: BFS visited flags
// Thread 1: visited[5] = true
// Thread 2: visited[7] = true
// Don't care which happens first!

// Exception: When combining with locks or channels
let visited = visited_flag.load(Ordering::Acquire);
if visited {
    let data = distance[v].load(Ordering::Relaxed);
    // Acquire ensures data is visible
}
```

**False Sharing Problem:**
```rust
// BAD: Adjacent vertices in same cache line
struct Graph {
    visited: Vec<AtomicBool>,  // 1 byte each, 64 in cache line
}

// If threads update nearby vertices:
// Thread 0: visited[0] = true    ← Cache line 0
// Thread 1: visited[1] = true    ← Same cache line!
// Thread 2: visited[2] = true    ← Same cache line!

// Each update invalidates entire cache line
// Cache coherency traffic → 5-10x slowdown!

// GOOD: Pad to cache line size
#[repr(align(64))]
struct AlignedAtomicBool(AtomicBool);

struct Graph {
    visited: Vec<AlignedAtomicBool>,  // 64 bytes each
}

// Now each vertex has own cache line
// No false sharing → 5-10x faster!
```

**Atomic vs Mutex Performance:**
```rust
// Benchmark: 1M vertex BFS, mark vertices visited

// Using Mutex:
let visited = Arc::new(Mutex::new(vec![false; n]));
frontier.par_iter().for_each(|&u| {
    for &v in graph.neighbors(u) {
        let mut vis = visited.lock().unwrap();  // Lock!
        if !vis[v] {
            vis[v] = true;
        }
    }
});
// Time: 850ms (lock contention!)

// Using AtomicBool:
let visited: Vec<AtomicBool> = ...;
frontier.par_iter().for_each(|&u| {
    for &v in graph.neighbors(u) {
        visited[v].swap(true, Ordering::Relaxed);  // No lock
    }
});
// Time: 80ms (10x faster!)

// Why? Lock = ~100ns, Atomic = ~10ns
// Lock also serializes updates (bottleneck)
```

**Atomic Operations Cost:**
```
Operation:          Latency:    Throughput:
store (Relaxed)     ~1-2ns      64 GB/s
load (Relaxed)      ~1-2ns      64 GB/s
swap (Relaxed)      ~10ns       10M ops/sec
CAS (Relaxed)       ~20ns       5M ops/sec
fetch_add (Relaxed) ~15ns       6M ops/sec

Compare to:
Regular write       ~0.5ns      64 GB/s
Mutex lock/unlock   ~100ns      100k ops/sec

Atomics are 50x faster than mutexes!
But still 10x slower than regular operations
```

---

### 6. Work Stealing and Load Balancing

**What Is It?**
Work stealing is a scheduling strategy where idle threads steal work from busy threads, essential for load balancing in irregular workloads like graph processing.

**How It Works:**
```
Initial work distribution:
Thread 0: [v0, v1, v2, ..., v249]     ← 250 vertices
Thread 1: [v250, v251, ..., v499]     ← 250 vertices
Thread 2: [v500, v501, ..., v749]     ← 250 vertices
Thread 3: [v750, v751, ..., v999]     ← 250 vertices

Without work stealing:
Thread 0: Processing celebrity (10,000 neighbors) → 50ms
Thread 1: Done after 2ms → idle for 48ms
Thread 2: Done after 1.5ms → idle for 48.5ms
Thread 3: Done after 2.2ms → idle for 47.8ms

Utilization: 25% (3 threads wasted!)

With work stealing:
Thread 0: [v0, v1, ..., v249]
          ↓ (steal from bottom)
Thread 1: → steals [v200-249] from Thread 0
Thread 2: → steals [v150-199] from Thread 0
Thread 3: → steals [v100-149] from Thread 0

Now all threads help process celebrity's neighbors
Utilization: 90%+ (much better!)
```

**Rayon's Work Stealing Implementation:**
```rust
// Rayon automatically implements work stealing
use rayon::prelude::*;

// Each parallel operation splits work recursively
frontier.par_iter().for_each(|&vertex| {
    // Process vertex
});

// Under the hood:
// 1. Rayon splits collection into chunks
// 2. Each worker thread has local deque
// 3. Thread pushes/pops from HEAD of own deque
// 4. Idle threads steal from TAIL of other deques
// 5. Minimizes contention (opposite ends)
```

**Work Stealing Deque:**
```
Thread's local deque (double-ended queue):

        HEAD                    TAIL
         ↓                       ↓
    [v1][v2][v3][v4][v5][v6][v7][v8]
     ↑                           ↑
    Push/Pop                   Steal from here
    (owner thread)             (thief threads)

Owner thread (Thread 0):
- Push new work at HEAD
- Pop work from HEAD (LIFO for cache locality)

Thief threads (Thread 1, 2, 3):
- Steal from TAIL (oldest work, likely larger chunks)
- Minimizes contention (opposite ends)
- Uses atomic CAS for theft

Benefits:
- Owner thread never blocks (no locks for push/pop)
- Thieves only contend with other thieves (rare)
- LIFO for owner = better cache locality
- Stealing from TAIL = larger work chunks (less overhead)
```

**Load Balance Metrics:**
```rust
pub struct LoadBalanceMetrics {
    work_per_thread: Vec<usize>,
}

impl LoadBalanceMetrics {
    pub fn imbalance_factor(&self) -> f64 {
        let max = *self.work_per_thread.iter().max().unwrap() as f64;
        let avg = self.work_per_thread.iter().sum::<usize>() as f64
                / self.work_per_thread.len() as f64;
        max / avg
    }

    pub fn efficiency(&self) -> f64 {
        let total_work: usize = self.work_per_thread.iter().sum();
        let max_work = *self.work_per_thread.iter().max().unwrap();
        let num_threads = self.work_per_thread.len();

        (total_work as f64) / (max_work as f64 * num_threads as f64)
    }
}

// Interpretation:
// Imbalance factor = 1.0: Perfect balance
// Imbalance factor = 2.0: Busiest thread does 2x average work
// Efficiency = 1.0: All threads busy 100% of time
// Efficiency = 0.5: Threads idle 50% of time (wasted resources)
```

**Benchmarks on Power-Law Graph:**
```
Graph: 100,000 vertices, power-law distribution
Task: Parallel BFS from random start

Without work stealing (static distribution):
Thread 0: 45,000 edges processed → 42ms
Thread 1: 3,200 edges processed → 3ms
Thread 2: 2,800 edges processed → 2.5ms
Thread 3: 3,100 edges processed → 2.8ms

Imbalance factor: 45000/13525 = 3.3x
Efficiency: 54100 / (45000 * 4) = 30%
Total time: 42ms (dominated by Thread 0)

With work stealing (Rayon):
Thread 0: 15,000 edges → 14ms
Thread 1: 14,200 edges → 13ms
Thread 2: 13,500 edges → 12.5ms
Thread 3: 11,400 edges → 11ms

Imbalance factor: 15000/13525 = 1.1x
Efficiency: 54100 / (15000 * 4) = 90%
Total time: 14ms (3x faster!)
```

**Chunk Size Trade-off:**
```rust
// Small chunks: Better balance, more overhead
frontier.par_iter()
    .with_min_len(1)  // Every vertex is separate task
    .for_each(|&v| process(v));

// + Perfect load balance
// - High task creation overhead
// - Poor cache locality

// Large chunks: Less overhead, worse balance
frontier.par_iter()
    .with_min_len(1000)  // 1000 vertices per task
    .for_each(|&v| process(v));

// + Low overhead
// + Better cache locality
// - Poor load balance (if few chunks)

// Optimal: Adaptive based on graph
let chunk_size = frontier.len() / (num_threads * 4);
frontier.par_iter()
    .with_min_len(chunk_size)
    .for_each(|&v| process(v));

// Rule of thumb: 2-8x more tasks than threads
```

---

### 7. Delta-Stepping Algorithm for Weighted Shortest Paths

**What Is It?**
Delta-stepping is a parallel shortest path algorithm that relaxes Dijkstra's strict ordering requirement by bucketing vertices into distance ranges.

**Dijkstra's Algorithm (Sequential):**
```rust
fn dijkstra(graph: &WeightedGraph, start: usize) -> Vec<f32> {
    let mut dist = vec![f32::INFINITY; n];
    let mut heap = BinaryHeap::new();

    dist[start] = 0.0;
    heap.push((Reverse(0.0), start));

    while let Some((Reverse(d), u)) = heap.pop() {
        if d > dist[u] { continue; }  // Outdated entry

        // Relax all outgoing edges
        for &(v, weight) in graph.neighbors(u) {
            let new_dist = dist[u] + weight;
            if new_dist < dist[v] {
                dist[v] = new_dist;
                heap.push((Reverse(new_dist), v));
            }
        }
    }

    dist
}

// Why not parallelizable?
// Must process vertices in strict distance order
// Priority queue is inherently sequential
```

**Delta-Stepping Insight:**
```
Instead of strict ordering, use approximate ordering:

Dijkstra: Process vertices in exact distance order
          0.00 → 0.01 → 0.02 → 0.03 → ...
          (Sequential bottleneck)

Delta-stepping: Process vertices in distance buckets (Δ = 1.0)
          Bucket 0: [0.0, 1.0)    ← Process all in parallel
          Bucket 1: [1.0, 2.0)    ← Process all in parallel
          Bucket 2: [2.0, 3.0)    ← Process all in parallel
          ...

Trade-off: Larger Δ = more parallelism, more redundant work
```

**Algorithm:**
```rust
fn delta_stepping(graph: &WeightedGraph, start: usize, delta: f32) -> Vec<f32> {
    let mut dist = vec![f32::INFINITY; n];
    dist[start] = 0.0;

    // Buckets indexed by ⌊distance / delta⌋
    let mut buckets: Vec<Vec<usize>> = vec![Vec::new(); num_buckets];
    buckets[0].push(start);

    for bucket_idx in 0.. {
        // Find first non-empty bucket
        if buckets[bucket_idx].is_empty() {
            if all_buckets_empty() { break; }
            continue;
        }

        // PARALLEL: Process all vertices in bucket
        let vertices = std::mem::take(&mut buckets[bucket_idx]);

        let updates: Vec<_> = vertices.par_iter()
            .flat_map(|&u| {
                graph.neighbors(u).iter().filter_map(|&(v, weight)| {
                    let new_dist = dist[u] + weight;

                    // Try to update distance (atomic CAS)
                    if try_update_distance(&dist, v, new_dist) {
                        Some((v, new_dist))
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Re-insert updated vertices into appropriate buckets
        for (v, d) in updates {
            let bucket = (d / delta).floor() as usize;
            buckets[bucket].push(v);
        }
    }

    dist
}
```

**Visual Example:**
```
Graph with edge weights:

    (1.2)
0 -------→ 1
|          |(0.5)
|(0.8)     ↓
↓          3
2 -------→ 4
   (2.1)

Delta-stepping with Δ = 1.0:

Step 1: Process Bucket 0 [0.0, 1.0)
  Vertices: [0]
  dist[0] = 0.0

  Relax edges from 0:
    dist[1] = min(∞, 0.0 + 1.2) = 1.2 → Bucket 1
    dist[2] = min(∞, 0.0 + 0.8) = 0.8 → Bucket 0 (re-insert!)

Step 2: Process Bucket 0 again [0.0, 1.0)
  Vertices: [2]
  dist[2] = 0.8

  Relax edges from 2:
    dist[4] = min(∞, 0.8 + 2.1) = 2.9 → Bucket 2

Step 3: Process Bucket 1 [1.0, 2.0)
  Vertices: [1]
  dist[1] = 1.2

  Relax edges from 1:
    dist[3] = min(∞, 1.2 + 0.5) = 1.7 → Bucket 1 (re-insert!)

Step 4: Process Bucket 1 again [1.0, 2.0)
  Vertices: [3]
  dist[3] = 1.7

  Relax edges from 3:
    dist[4] = min(2.9, 1.7 + ...) = ... (depends on graph)

Final distances: [0.0, 1.2, 0.8, 1.7, 2.9]
```

**Delta Parameter Tuning:**
```
Small Δ (e.g., 0.1):
+ More accurate (closer to Dijkstra)
+ Less redundant work
- Less parallelism (smaller buckets)
- More synchronization overhead

Large Δ (e.g., 10.0):
+ More parallelism (larger buckets)
+ Fewer synchronization barriers
- More redundant work (vertices processed multiple times)
- May find suboptimal paths (need more iterations)

Optimal Δ:
Δ ≈ average edge weight / 2
Or tune empirically for your graph
```

**Performance Comparison:**
```
Graph: 1M vertices, 10M edges, random weights [0.1, 10.0]

Dijkstra (sequential):           850ms
Delta-stepping (Δ=0.5, 8 cores): 180ms (4.7x speedup)
Delta-stepping (Δ=1.0, 8 cores): 145ms (5.9x speedup)
Delta-stepping (Δ=5.0, 8 cores): 220ms (3.9x speedup, too coarse)

Best performance: Δ ≈ average edge weight
```

---

### 8. Iterative Algorithms: PageRank

**What Is It?**
PageRank is an iterative algorithm that computes importance scores by propagating rank through the graph until convergence.

**PageRank Formula:**
```
PR(v) = (1 - d)/N + d × Σ PR(u) / outdegree(u)
                        u→v

Where:
- d = damping factor (typically 0.85)
- N = total number of vertices
- u→v means u has edge to v
```

**Intuition:**
```
A vertex is important if important vertices link to it.

Example: Academic citations
- Paper A cited by 10 obscure papers: low rank
- Paper B cited by 3 highly-cited papers: high rank

Random surfer model:
- Start at random page
- With probability d: Follow random link
- With probability 1-d: Jump to random page
- PageRank = steady-state probability distribution
```

**Sequential Implementation:**
```rust
fn pagerank(graph: &Graph, iterations: usize, damping: f32) -> Vec<f32> {
    let n = graph.num_vertices;
    let mut ranks = vec![1.0 / n as f32; n];  // Initialize uniformly
    let mut new_ranks = vec![0.0; n];

    for _ in 0..iterations {
        // Reset new ranks to random jump probability
        for v in 0..n {
            new_ranks[v] = (1.0 - damping) / n as f32;
        }

        // Distribute rank from each vertex to its neighbors
        for u in 0..n {
            let rank_contribution = damping * ranks[u] / graph.degree(u) as f32;

            for &v in graph.neighbors(u) {
                new_ranks[v] += rank_contribution;
            }
        }

        // Swap buffers
        std::mem::swap(&mut ranks, &mut new_ranks);
    }

    ranks
}
```

**Why It's Embarrassingly Parallel:**
```rust
// Each vertex update is independent!
// Can compute all new ranks in parallel

fn parallel_pagerank(graph: &Graph, iterations: usize, damping: f32) -> Vec<f32> {
    let n = graph.num_vertices;
    let ranks = Arc::new(Mutex::new(vec![1.0 / n as f32; n]));

    for _ in 0..iterations {
        let new_ranks: Vec<f32> = (0..n).into_par_iter()
            .map(|v| {
                // PARALLEL: Compute each rank independently
                let base = (1.0 - damping) / n as f32;
                let mut sum = 0.0;

                // Sum contributions from in-neighbors
                for u in 0..n {
                    if graph.has_edge(u, v) {
                        let old_ranks = ranks.lock().unwrap();
                        sum += old_ranks[u] / graph.degree(u) as f32;
                    }
                }

                base + damping * sum
            })
            .collect();

        *ranks.lock().unwrap() = new_ranks;
    }

    Arc::try_unwrap(ranks).unwrap().into_inner().unwrap()
}

// Expected speedup: Near-linear (7-8x on 8 cores)
// Why? No dependencies between vertex updates
```

**Convergence Detection:**
```rust
fn pagerank_until_convergence(
    graph: &Graph,
    damping: f32,
    epsilon: f32
) -> (Vec<f32>, usize) {
    let n = graph.num_vertices;
    let mut ranks = vec![1.0 / n as f32; n];
    let mut iterations = 0;

    loop {
        let new_ranks = compute_next_iteration(&ranks, graph, damping);

        // Check convergence: max absolute change
        let max_change = ranks.iter()
            .zip(&new_ranks)
            .map(|(old, new)| (old - new).abs())
            .fold(0.0f32, f32::max);

        ranks = new_ranks;
        iterations += 1;

        if max_change < epsilon {
            break;  // Converged!
        }

        if iterations > 1000 {
            break;  // Safety: prevent infinite loop
        }
    }

    (ranks, iterations)
}

// Typical convergence:
// epsilon = 0.01: 15-30 iterations
// epsilon = 0.001: 30-50 iterations
// epsilon = 0.0001: 50-100 iterations
```

**Optimized Implementation (CSR Format):**
```rust
// Use CSR for cache-friendly iteration
fn parallel_pagerank_csr(graph: &GraphCSR, iterations: usize, damping: f32) -> Vec<f32> {
    let n = graph.num_vertices;
    let mut ranks = vec![1.0 / n as f32; n];
    let mut new_ranks = vec![0.0; n];

    // Precompute out-degrees
    let out_degrees: Vec<f32> = (0..n)
        .map(|v| graph.neighbors(v).len() as f32)
        .collect();

    for _ in 0..iterations {
        // PARALLEL: Compute contributions from each vertex
        new_ranks.par_iter_mut()
            .enumerate()
            .for_each(|(v, rank)| {
                *rank = (1.0 - damping) / n as f32;

                // Iterate over in-neighbors (requires transpose graph)
                // Or: scatter contributions from each vertex
            });

        // Better: Scatter approach (no transpose needed)
        (0..n).into_par_iter().for_each(|u| {
            let contribution = damping * ranks[u] / out_degrees[u];

            // Each vertex distributes rank to neighbors
            for &v in graph.neighbors(u) {
                // Atomic add to avoid race condition
                atomic_add(&new_ranks[v], contribution);
            }
        });

        std::mem::swap(&mut ranks, &mut new_ranks);
        new_ranks.fill(0.0);
    }

    ranks
}
```

**Performance:**
```
Graph: 100k vertices, power-law distribution
20 iterations

Sequential:         180ms
Parallel (2 cores): 95ms (1.9x)
Parallel (4 cores): 52ms (3.5x)
Parallel (8 cores): 28ms (6.4x)

Near-linear scaling!
Why? No synchronization within iteration
Only barrier between iterations
```

---

### 9. Union-Find with Path Compression and Union by Rank

**What Is It?**
Union-Find (disjoint set union) is a data structure for tracking connected components with near-constant-time operations.

**Operations:**
1. **Find(v)**: Return root of v's component
2. **Union(u, v)**: Merge components containing u and v

**Naive Implementation:**
```rust
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect()  // Each vertex is own parent
        }
    }

    fn find(&self, mut v: usize) -> usize {
        // Follow parent pointers to root
        while self.parent[v] != v {
            v = self.parent[v];
        }
        v
    }

    fn union(&mut self, u: usize, v: usize) {
        let root_u = self.find(u);
        let root_v = self.find(v);

        if root_u != root_v {
            self.parent[root_u] = root_v;  // Link roots
        }
    }
}

// Problem: Can create long chains
// Find becomes O(n) in worst case!
```

**Path Compression:**
```rust
// Flatten tree during find
fn find(&mut self, v: usize) -> usize {
    if self.parent[v] != v {
        // Recursively find root and update parent
        self.parent[v] = self.find(self.parent[v]);
    }
    self.parent[v]
}

// Visual:
// Before:     After path compression:
//     5           5
//     |          /|\
//     4         1 2 3
//     |            |
//     3            4
//     |
//     2
//     |
//     1
//
// find(1) flattens tree: all nodes point directly to root
// Future finds: O(1) instead of O(depth)
```

**Union by Rank:**
```rust
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,  // Upper bound on tree height
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn union(&mut self, u: usize, v: usize) -> bool {
        let root_u = self.find(u);
        let root_v = self.find(v);

        if root_u == root_v {
            return false;  // Already connected
        }

        // Attach smaller tree under larger tree
        if self.rank[root_u] < self.rank[root_v] {
            self.parent[root_u] = root_v;
        } else if self.rank[root_u] > self.rank[root_v] {
            self.parent[root_v] = root_u;
        } else {
            self.parent[root_v] = root_u;
            self.rank[root_u] += 1;  // Same rank: increase by 1
        }

        true
    }
}

// Keeps trees balanced → O(log n) depth
// Combined with path compression → O(α(n)) amortized
// Where α(n) is inverse Ackermann function ≈ 4 for all practical n
```

**Complexity Analysis:**
```
Without optimizations:
- Find: O(n) worst case (chain)
- Union: O(n) (includes find)

With path compression only:
- Find: O(log n) amortized
- Union: O(log n) amortized

With union by rank only:
- Find: O(log n) worst case
- Union: O(log n) worst case

With both optimizations:
- Find: O(α(n)) amortized ≈ O(1) in practice
- Union: O(α(n)) amortized ≈ O(1) in practice

Where α(n) < 5 for n < 10^80 (more atoms than universe!)
```

**Parallel Union-Find:**
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

struct ParallelUnionFind {
    parent: Vec<AtomicUsize>,
    rank: Vec<AtomicUsize>,
}

impl ParallelUnionFind {
    fn find(&self, mut v: usize) -> usize {
        loop {
            let parent = self.parent[v].load(Ordering::Relaxed);
            if parent == v {
                return v;
            }

            // Attempt path compression (opportunistic)
            let grandparent = self.parent[parent].load(Ordering::Relaxed);
            let _ = self.parent[v].compare_exchange(
                parent,
                grandparent,
                Ordering::Relaxed,
                Ordering::Relaxed
            );

            v = parent;
        }
    }

    fn union(&self, u: usize, v: usize) -> bool {
        loop {
            let root_u = self.find(u);
            let root_v = self.find(v);

            if root_u == root_v {
                return false;
            }

            // Ensure root_u < root_v (deterministic order)
            let (small, large) = if root_u < root_v {
                (root_u, root_v)
            } else {
                (root_v, root_u)
            };

            // Try to link large → small using CAS
            match self.parent[large].compare_exchange(
                large,
                small,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => return true,  // Success
                Err(_) => continue,    // Retry (root changed)
            }
        }
    }
}
```

**Connected Components Example:**
```rust
fn connected_components(graph: &Graph) -> Vec<usize> {
    let uf = UnionFind::new(graph.num_vertices);

    // Union all edges
    for u in 0..graph.num_vertices {
        for &v in graph.neighbors(u) {
            uf.union(u, v);
        }
    }

    // Map vertices to component IDs
    (0..graph.num_vertices)
        .map(|v| uf.find(v))
        .collect()
}

// Example:
// Graph: 0-1-2  3-4  5
//
// After unions:
// parent: [0, 0, 0, 3, 3, 5]
//
// Components:
// [0, 0, 0, 3, 3, 5]
//  └─ comp 0 ─┘ └comp 3┘ comp 5
```

---

### 10. Memory Contention and Cache Effects in Graph Processing

**What Is It?**
Graph algorithms exhibit poor cache behavior due to irregular access patterns and high memory contention from concurrent updates.

**Cache Hierarchy:**
```
L1 cache:  32-64 KB per core,  ~1ns latency,  ~1 TB/s bandwidth
L2 cache:  256-512 KB per core, ~3ns latency,  ~500 GB/s bandwidth
L3 cache:  8-32 MB shared,      ~15ns latency, ~200 GB/s bandwidth
RAM:       16-64 GB,            ~80ns latency, ~40 GB/s bandwidth

Cache line size: 64 bytes
```

**Sequential vs Random Access:**
```rust
// Sequential access (cache-friendly):
let data = vec![0u64; 10_000_000];
for i in 0..data.len() {
    sum += data[i];  // Predictable, prefetchable
}
// Time: 10ms
// Cache miss rate: ~2% (only first access per cache line)

// Random access (cache-hostile):
let indices: Vec<usize> = random_permutation(10_000_000);
for &i in &indices {
    sum += data[i];  // Unpredictable, no prefetching
}
// Time: 180ms (18x slower!)
// Cache miss rate: ~95% (nearly every access misses)
```

**Graph Traversal Access Pattern:**
```rust
// BFS frontier processing
for &vertex in frontier {
    for &neighbor in graph.neighbors(vertex) {
        // neighbor is essentially random!
        if !visited[neighbor] {
            visited[neighbor] = true;  // Random write
            next_frontier.push(neighbor);
        }
    }
}

// Access pattern:
// vertex=0 → neighbors=[42, 157, 9045, 3291]
// vertex=1 → neighbors=[5, 99042, 123]
// ...
// Completely random → ~90% cache misses
```

**Cache Line Conflicts:**
```rust
// Multiple threads updating nearby array elements
let visited: Vec<AtomicBool> = ...;  // 1 byte each

// Cache line 0: visited[0..63]
// Cache line 1: visited[64..127]
// ...

// Thread 0: visited[5] = true   ← Cache line 0
// Thread 1: visited[10] = true  ← Same cache line!
// Thread 2: visited[15] = true  ← Same cache line!

// Cache coherency protocol (MESI):
// Each write invalidates cache line in other cores
// Other cores must reload → cache coherency traffic

// Result: 5-10x slowdown from false sharing!
```

**False Sharing Solution:**
```rust
// BAD: Tight packing
struct Counters {
    count0: AtomicUsize,  // Offset 0
    count1: AtomicUsize,  // Offset 8
    count2: AtomicUsize,  // Offset 16
    count3: AtomicUsize,  // Offset 24
}  // All in same 64-byte cache line!

// GOOD: Cache line padding
#[repr(align(64))]
struct PaddedCounter {
    count: AtomicUsize,
    _padding: [u8; 56],  // Fill rest of cache line
}

struct Counters {
    count0: PaddedCounter,  // Offset 0
    count1: PaddedCounter,  // Offset 64
    count2: PaddedCounter,  // Offset 128
    count3: PaddedCounter,  // Offset 192
}  // Each in separate cache line

// Benchmark:
// Tight packing:  850ns per update (contention)
// Padded:          45ns per update (no contention)
// 19x faster!
```

**Graph-Specific Optimizations:**

**1. CSR Format for Sequential Access:**
```rust
// Adjacency list: Scattered in memory
for &v in frontier {
    for &neighbor in graph.adjacency[v] {  // Heap allocation, pointer chase
        ...
    }
}

// CSR: Sequential memory access
for &v in frontier {
    let start = offsets[v];
    let end = offsets[v + 1];
    for i in start..end {
        let neighbor = edges[i];  // Sequential array access!
        ...
    }
}

// 2-3x faster due to cache locality
```

**2. Frontier Sorting:**
```rust
// Unsorted frontier: Random access to visited array
for &v in frontier {  // v = [7042, 15, 9999, 203, 8500, ...]
    visited[v] = true;  // Cache misses everywhere
}

// Sorted frontier: Improved locality
frontier.sort_unstable();  // v = [15, 203, 7042, 8500, 9999, ...]
for &v in frontier {
    visited[v] = true;  // Better cache reuse
}

// Benchmark:
// Unsorted: 120ms (90% cache misses)
// Sorted:    85ms (60% cache misses)
// 1.4x speedup
```

**3. Frontier Compression (Remove Duplicates):**
```rust
// Duplicates cause redundant work and cache pollution
let mut frontier: Vec<usize> = ...;  // [5, 42, 5, 99, 42, 15, 5]

frontier.sort_unstable();
frontier.dedup();  // [5, 15, 42, 99]

// Benefits:
// - Smaller frontier → faster iteration
// - No redundant work on same vertex
// - Better cache utilization

// Typical savings: 30-50% reduction in frontier size
```

**Memory Bandwidth Bottleneck:**
```rust
// Graph algorithms are memory-bound
// Computation: Simple operations (mark visited, update distance)
// Memory: Massive random access (neighbors, visited, distances)

// Roofline analysis:
// Arithmetic intensity = FLOPs / Bytes accessed
// BFS: ~0.1 (1 comparison per 8 bytes loaded)
// Matrix multiply: ~10 (many FLOPs per element)

// BFS is 100x more memory-bound than matmul!
// Can't utilize full CPU compute (waiting on memory)

// Bandwidth utilization:
// L3 to RAM: ~40 GB/s peak
// BFS achieved: ~8 GB/s (20% utilization)
// Why? Random access pattern doesn't allow prefetching
```

**NUMA Effects:**
```
Multi-socket system:
Socket 0 (cores 0-7):  Local memory: 0-32 GB
Socket 1 (cores 8-15): Local memory: 32-64 GB

// Thread on core 0 accessing memory at 40 GB (socket 1):
// Latency: ~150ns (2x remote penalty)
// Bandwidth: ~20 GB/s (half local bandwidth)

// Graph data distributed across sockets:
// 50% accesses remote → 1.5x slowdown

// Solution: NUMA-aware allocation
// Pin threads to cores
// Allocate memory local to thread
```

---

### 11. Amdahl's Law for Irregular Workloads

**What Is It?**
Amdahl's Law quantifies the maximum speedup possible when parallelizing a program with both parallel and sequential portions. Irregular workloads have inherent sequential bottlenecks that limit scalability.

**Amdahl's Law Formula:**
```
Speedup = 1 / (S + P/N)

Where:
- S = fraction of execution time that is sequential
- P = fraction of execution time that is parallel (S + P = 1)
- N = number of processors

Maximum speedup (N → ∞):
Speedup_max = 1 / S
```

**Example Calculation:**
```
Algorithm with 10% sequential code (S = 0.1, P = 0.9):

With 2 cores:  Speedup = 1 / (0.1 + 0.9/2)  = 1.82x
With 4 cores:  Speedup = 1 / (0.1 + 0.9/4)  = 3.08x
With 8 cores:  Speedup = 1 / (0.1 + 0.9/8)  = 4.71x
With 16 cores: Speedup = 1 / (0.1 + 0.9/16) = 6.40x
With ∞ cores:  Speedup = 1 / 0.1             = 10x (maximum!)

Key insight: 10% sequential limits speedup to 10x,
regardless of how many cores you add!
```

**Why Graphs Have High Sequential Fraction:**

**1. Synchronization Barriers:**
```rust
// Level-synchronous BFS
for level in 0.. {
    // PARALLEL: Process frontier
    let next_frontier: Vec<usize> = frontier
        .par_iter()
        .flat_map(|&v| expand_neighbors(v))
        .collect();

    // SEQUENTIAL: Barrier + setup next iteration
    if next_frontier.is_empty() { break; }
    frontier = next_frontier;
}

// Each iteration has parallel work + sequential barrier
// If frontiers are small (early/late levels): barrier dominates!
```

**2. High-Degree Vertices (Power-Law Graphs):**
```
Power-law graph: 1% of vertices have 90% of edges

Example: 1M vertices, 10M edges
- 10,000 vertices (1%): 9M edges (90%)
- 990,000 vertices (99%): 1M edges (10%)

Processing high-degree vertex:
Thread 0: Process celebrity (900k neighbors) → 100ms
Threads 1-7: Process normal vertices → 5ms each

Parallel time per level = max(100ms, 5ms) = 100ms
Sequential equivalent = 105ms
Parallel efficiency = 105 / (100 * 8) = 13%

Sequential fraction S = 100 / 105 = 95%!
Maximum speedup = 1 / 0.95 = 1.05x (terrible!)
```

**3. Load Imbalance:**
```rust
// Static work distribution with skewed workload
let chunk_size = frontier.len() / num_threads;

// Thread 0: 250 vertices, 2M edges → 85ms
// Thread 1: 250 vertices, 300k edges → 12ms
// Thread 2: 250 vertices, 280k edges → 11ms
// Thread 3: 250 vertices, 320k edges → 13ms

// Parallel time = 85ms (limited by slowest thread)
// Ideal parallel time = (2M + 300k + 280k + 320k) / 4 / rate = 30ms
// Efficiency = 30 / 85 = 35%

// Effective sequential fraction = 1 - 0.35 = 65%
// This is due to load imbalance, not inherent sequential work!
```

**Measuring Sequential Fraction:**
```rust
// Empirical measurement:
let seq_time = measure_sequential(&graph);      // 800ms
let par_time_8 = measure_parallel(&graph, 8);   // 180ms

// Solve for S:
// 180 = 800 * (S + (1-S)/8)
// 180/800 = S + (1-S)/8
// 0.225 = S + 0.125 - 0.125*S
// 0.1 = 0.875*S
// S = 0.114 (11.4% sequential)

// Maximum speedup with infinite cores:
let max_speedup = 1.0 / 0.114;  // = 8.77x

// With 8 cores achieved 4.4x
// Could potentially reach 8.77x with infinite cores
// But realistically limited by sequential portion
```

**Scalability Analysis:**
```
Strong scaling (fixed problem size, more cores):

Graph: 1M vertices, 10M edges

Cores:  Time:   Speedup:  Efficiency:
1       800ms   1.00x     100%
2       420ms   1.90x     95%
4       230ms   3.48x     87%
8       140ms   5.71x     71%
16      95ms    8.42x     53%
32      75ms    10.67x    33%

Observations:
- Speedup sublinear (diminishes with more cores)
- Efficiency drops (wasted parallelism)
- Approaching maximum speedup (~11x)

Weak scaling (problem size grows with cores):

Cores:  Vertices:   Time:    Efficiency:
1       125k        100ms    100%
2       250k        105ms    95%
4       500k        115ms    87%
8       1M          140ms    71%
16      2M          180ms    56%

Better efficiency, but still degraded
(due to increased synchronization, contention)
```

**Reducing Sequential Fraction:**

**1. Work Stealing (reduce imbalance):**
```
Without: S_effective = 60% (imbalance)
With:    S_effective = 20% (better balance)

Maximum speedup: 1/0.20 = 5x (vs 1.67x)
```

**2. Frontier Aggregation (reduce barriers):**
```rust
// Instead of barrier every level:
let mut combined_frontier = vec![start];
let mut level = 0;

while !combined_frontier.is_empty() {
    // Process multiple levels before barrier
    for _ in 0..batch_size {
        combined_frontier = expand_frontier_parallel(&combined_frontier);
    }
}

// Fewer barriers = less sequential overhead
```

**3. Asynchronous Algorithms:**
```rust
// No global barriers, threads work independently
// (May find suboptimal solutions, but faster)

// Each thread continuously pulls work from shared queue
loop {
    if let Some(vertex) = work_queue.pop() {
        let neighbors = graph.neighbors(vertex);
        work_queue.extend(neighbors);  // Atomic push
    }
}

// No synchronization → S approaches 0
// Can achieve near-linear speedup
// Trade-off: Correctness (may visit vertices multiple times)
```

**Realistic Expectations:**
```
Graph algorithm speedups on real hardware (8 cores):

Regular graphs (uniform degree):
- Theoretical max: 8x
- Achieved: 6-7x (75-87% efficiency)
- Limited by: Memory bandwidth, cache coherency

Power-law graphs (skewed degree):
- Theoretical max: 10x (Amdahl's Law)
- Achieved: 3-5x (37-62% efficiency)
- Limited by: Load imbalance, sequential bottleneck

Very skewed graphs (social networks):
- Theoretical max: 5x (high S)
- Achieved: 2-3x (25-37% efficiency)
- Limited by: Celebrity vertices, memory contention

Lesson: Don't expect linear scaling on irregular workloads!
5x speedup on 8 cores is excellent for graph processing.
```

---

## Connection to This Project

This section maps the concepts explained above to specific milestones in the parallel graph processing project.

### Milestone 1: Graph Representation and Sequential BFS

**Concepts Used:**
- **Graph Representations (CSR vs Adjacency List)**: Implement both formats to understand trade-offs; CSR provides 2-3x faster iteration for BFS
- **Power-Law Distributions**: Generate realistic graphs using Barabási-Albert model with preferential attachment
- **Sequential BFS Algorithm**: Foundation using queue-based traversal; baseline for parallel comparison

**Key Insights:**
- CSR format is immutable but cache-friendly (contiguous memory access)
- Power-law graphs create extreme degree variance (median 50, max 10,000+)
- Sequential BFS time complexity O(V + E), but memory access pattern determines real performance

**Why This Matters:**
Understanding sequential baseline and realistic graph structures is essential before attempting parallelization. The CSR format will be crucial for performance in later milestones.

---

### Milestone 2: Level-Synchronous Parallel BFS

**Concepts Used:**
- **Level-Synchronous BFS and Frontier-Based Algorithms**: Replace queue with explicit frontier representation to enable parallelization
- **Atomic Operations for Concurrent Updates**: Use `AtomicBool` for visited array to prevent race conditions during concurrent marking
- **Irregular Parallelism and Load Imbalance**: Frontier size varies by level (1 → 2,500 → 180 → 8); requires dynamic load balancing
- **Memory Ordering**: Relaxed ordering sufficient for visited flags since operations are independent

**Key Insights:**
- Each level is a synchronization barrier; vertices within level are independent
- `swap()` operation provides atomic test-and-set for visited marking
- Expected speedup limited to 3-5x on 8 cores due to varying frontier sizes
- Small frontiers (early/late levels) don't benefit from parallelism

**Performance Trade-offs:**
- Atomic operations add 10-20ns overhead per operation
- Synchronization barriers between levels limit scalability
- Random memory access reduces cache hit rate compared to sequential

---

### Milestone 3: Parallel Shortest Path (Delta-Stepping)

**Concepts Used:**
- **Delta-Stepping Algorithm**: Bucket-based relaxation enables parallelism by processing approximate distance ranges
- **Atomic CAS Operations**: Use compare-and-swap loops to update distances when shorter path found
- **Work Stealing and Load Balancing**: Rayon automatically distributes vertices within buckets; critical for skewed graphs
- **Cache Effects**: Bucket sorting improves locality compared to pure random access

**Key Insights:**
- Delta parameter trades accuracy for parallelism (Δ ≈ avg_edge_weight / 2 optimal)
- Multiple threads may relax same vertex (redundant work, but safe with CAS)
- Expected speedup 4-6x on 8 cores (better than BFS due to longer-running buckets)
- Requires WeightedGraph representation with edge weights

**Algorithm Complexity:**
- Sequential Dijkstra: O((V + E) log V)
- Delta-stepping: O(V + E + nΔ) work, O(d) span where d = diameter/Δ

---

### Milestone 4: PageRank Algorithm

**Concepts Used:**
- **Iterative Algorithms (PageRank)**: Fixed-point iteration until convergence; each iteration is embarrassingly parallel
- **Work Stealing**: Rayon distributes vertex updates; perfect load balance since all vertices do equal work
- **Convergence Detection**: Monitor max change between iterations; adaptive termination
- **Memory Contention**: Multiple reads of ranks array (read-heavy, minimal contention)

**Key Insights:**
- Near-linear speedup (6-8x on 8 cores) because vertex updates are independent
- No atomic operations needed (double buffering eliminates races)
- Convergence typically 20-50 iterations for ε = 0.001
- CSR format not required (no edge iteration, only neighbor lookups)

**Performance:**
- Best parallelism in entire project (no irregular workload)
- Memory bandwidth becomes bottleneck at high core counts
- Cache-friendly: Sequential iteration over vertices

---

### Milestone 5: Connected Components with Union-Find

**Concepts Used:**
- **Union-Find with Path Compression**: Amortized O(α(n)) ≈ O(1) find operations through tree flattening
- **Union by Rank**: Keep trees balanced; smaller tree attached under larger
- **Atomic Operations**: Parallel union-find requires CAS on parent pointers for concurrent unions
- **Race Conditions**: Multiple threads may try to link same roots; CAS handles conflicts

**Key Insights:**
- Path compression creates read-after-write dependencies (hard to parallelize)
- Deterministic union order (smaller root → larger root) prevents deadlocks
- Expected speedup limited to 3-5x due to sequential bottleneck in find operations
- Amdahl's Law: find operations are effectively sequential

**Algorithm Trade-offs:**
- Sequential union-find: Near-constant time with both optimizations
- Parallel union-find: Limited speedup due to frequent CAS retries
- Alternative: Shiloach-Vishkin algorithm (more parallel, but different approach)

---

### Milestone 6: Work Stealing and Load Balancing

**Concepts Used:**
- **Work Stealing**: Rayon's deque-based work distribution; idle threads steal from busy threads
- **Irregular Parallelism**: Power-law graphs create 3-10x load imbalance; work stealing reduces to 1.5-2x
- **Load Balance Metrics**: Imbalance factor = max_work / avg_work; efficiency = utilization percentage
- **Cache Effects and False Sharing**: Measure impact of cache line padding on concurrent updates
- **Amdahl's Law for Irregular Workloads**: Quantify sequential bottleneck from high-degree vertices

**Key Insights:**
- Work stealing improves efficiency from 30% to 90% on power-law graphs
- Imbalance factor drops from 3-5x (static) to 1.1-1.5x (dynamic)
- Cache line padding provides 5-10x speedup by eliminating false sharing
- Sequential fraction S = 10-30% typical for graph algorithms (max speedup 3-10x)

**Measurement Techniques:**
- Instrument each thread to count edges processed
- Compare static vs dynamic distribution on same graph
- Benchmark with/without cache line alignment
- Profile to identify sequential bottlenecks

**Expected Results:**
- Static distribution: 30-40% efficiency on power-law graph
- Work stealing: 80-90% efficiency
- Speedup: 5-6x on 8 cores (close to Amdahl's limit)

---

## Summary Table

| Milestone | Key Concepts | Expected Speedup | Main Challenge |
|-----------|--------------|------------------|----------------|
| M1: Sequential BFS | Graph representations, Power-law distributions | 1x (baseline) | Understanding irregular structure |
| M2: Parallel BFS | Level-synchronous, Atomics, Frontiers | 3-5x | Varying frontier sizes |
| M3: Delta-Stepping | Bucket relaxation, CAS loops, Delta tuning | 4-6x | Redundant work vs parallelism |
| M4: PageRank | Iterative convergence, Embarrassingly parallel | 6-8x | Memory bandwidth |
| M5: Union-Find | Path compression, Atomic CAS, Rank optimization | 3-5x | Sequential find operations |
| M6: Load Balancing | Work stealing, Metrics, False sharing, Amdahl's Law | N/A (analysis) | Quantifying bottlenecks |

**Overall Learning:**
This project demonstrates that irregular parallelism (graphs, trees, dynamic workloads) is fundamentally harder than regular parallelism (matrices, arrays). Achieving 5x speedup on 8 cores is excellent for graph algorithms, unlike 7-8x typical for regular workloads. The key is understanding Amdahl's Law, load imbalance, and cache effects specific to random-access patterns.

---

# Build The Project

## Milestone 1: Graph Representation and Sequential BFS

### Introduction

Implement efficient graph representation and sequential breadth-first search (BFS). BFS is the foundation for many graph algorithms and demonstrates the irregular workload pattern.

**Adjacency List vs CSR:**
- Adjacency List: Easy to build, pointer chasing
- CSR (Compressed Sparse Row): Cache-friendly, harder to build

**BFS Algorithm:**
```
Level 0: [start_vertex]
Level 1: All neighbors of Level 0
Level 2: All neighbors of Level 1 (not visited)
...
```

### Architecture

**Structs:**
- `Graph` - Adjacency list representation
  - **Field** `adjacency: Vec<Vec<usize>>` - adj[v] = list of neighbors
  - **Field** `num_vertices: usize` - Vertex count
  - **Field** `num_edges: usize` - Edge count
  - **Function** `new(n: usize) -> Self` - Create empty graph
  - **Function** `add_edge(&mut self, u: usize, v: usize)` - Add edge
  - **Function** `from_edge_list(edges: &[(usize, usize)], n: usize) -> Self`
  - **Function** `degree(&self, v: usize) -> usize` - Get vertex degree

- `GraphCSR` - Compressed Sparse Row format
  - **Field** `offsets: Vec<usize>` - offset[v] = start of v's neighbors
  - **Field** `edges: Vec<usize>` - Flat array of all edges
  - **Field** `num_vertices: usize`
  - **Function** `from_graph(g: &Graph) -> Self` - Convert to CSR
  - **Function** `neighbors(&self, v: usize) -> &[usize]` - Get neighbors

**Key Functions:**
- `bfs(graph: &Graph, start: usize) -> Vec<Option<usize>>` - BFS distances
- `bfs_levels(graph: &Graph, start: usize) -> Vec<Vec<usize>>` - Level-by-level
- `generate_random_graph(n: usize, avg_degree: usize) -> Graph` - Random graph
- `generate_power_law_graph(n: usize) -> Graph` - Realistic degree distribution

**CSR Format:**
```
Graph: 0→[1,2], 1→[2], 2→[0,1]

offsets: [0, 2, 3, 5]
edges:   [1, 2, 2, 0, 1]

neighbors(0) = edges[0..2] = [1, 2]
neighbors(1) = edges[2..3] = [2]
neighbors(2) = edges[3..5] = [0, 1]
```

**Role Each Plays:**
- Adjacency list: Easy modification, pointer overhead
- CSR: Compact, cache-friendly, immutable
- BFS: Foundation for reachability, shortest paths
- Frontier: Current level of exploration


### Starter Code

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Graph {
    adjacency: Vec<Vec<usize>>,
    num_vertices: usize,
    num_edges: usize,
}

impl Graph {
    pub fn new(n: usize) -> Self {
        // TODO: Create empty graph
        // Self {
        //     adjacency: vec![Vec::new(); n],
        //     num_vertices: n,
        //     num_edges: 0,
        // }
        todo!()
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        // TODO: Add directed edge u → v
        // self.adjacency[u].push(v);
        // self.num_edges += 1;
        todo!()
    }

    pub fn from_edge_list(edges: &[(usize, usize)], n: usize) -> Self {
        // TODO: Build graph from edge list
        // let mut g = Graph::new(n);
        // for &(u, v) in edges {
        //     g.add_edge(u, v);
        // }
        // g
        todo!()
    }

    pub fn degree(&self, v: usize) -> usize {
        self.adjacency[v].len()
    }

    pub fn neighbors(&self, v: usize) -> &[usize] {
        &self.adjacency[v]
    }
}

#[derive(Debug, Clone)]
pub struct GraphCSR {
    offsets: Vec<usize>,
    edges: Vec<usize>,
    num_vertices: usize,
}

impl GraphCSR {
    pub fn from_graph(g: &Graph) -> Self {
        // TODO: Convert adjacency list to CSR
        //
        // 1. Build offsets array
        // let mut offsets = vec![0];
        // for v in 0..g.num_vertices {
        //     offsets.push(offsets[v] + g.degree(v));
        // }
        //
        // 2. Flatten edges
        // let mut edges = Vec::with_capacity(g.num_edges);
        // for v in 0..g.num_vertices {
        //     edges.extend(g.neighbors(v));
        // }
        //
        // Self {
        //     offsets,
        //     edges,
        //     num_vertices: g.num_vertices,
        // }
        todo!()
    }

    pub fn neighbors(&self, v: usize) -> &[usize] {
        // TODO: Return slice of neighbors
        // &self.edges[self.offsets[v]..self.offsets[v + 1]]
        todo!()
    }
}

pub fn bfs(graph: &Graph, start: usize) -> Vec<Option<usize>> {
    // TODO: Sequential BFS
    //
    // let mut distances = vec![None; graph.num_vertices];
    // let mut queue = VecDeque::new();
    //
    // distances[start] = Some(0);
    // queue.push_back(start);
    //
    // while let Some(u) = queue.pop_front() {
    //     let dist_u = distances[u].unwrap();
    //
    //     for &v in graph.neighbors(u) {
    //         if distances[v].is_none() {
    //             distances[v] = Some(dist_u + 1);
    //             queue.push_back(v);
    //         }
    //     }
    // }
    //
    // distances
    todo!()
}

pub fn bfs_levels(graph: &Graph, start: usize) -> Vec<Vec<usize>> {
    // TODO: BFS returning vertices grouped by level
    //
    // let mut levels = vec![vec![start]];
    // let mut visited = vec![false; graph.num_vertices];
    // visited[start] = true;
    //
    // while !levels.last().unwrap().is_empty() {
    //     let current_level = levels.last().unwrap();
    //     let mut next_level = Vec::new();
    //
    //     for &u in current_level {
    //         for &v in graph.neighbors(u) {
    //             if !visited[v] {
    //                 visited[v] = true;
    //                 next_level.push(v);
    //             }
    //         }
    //     }
    //
    //     if !next_level.is_empty() {
    //         levels.push(next_level);
    //     }
    // }
    //
    // levels
    todo!()
}

pub fn generate_random_graph(n: usize, avg_degree: usize) -> Graph {
    // TODO: Generate random graph
    // use rand::Rng;
    // let mut rng = rand::thread_rng();
    // let mut g = Graph::new(n);
    //
    // for u in 0..n {
    //     for _ in 0..avg_degree {
    //         let v = rng.gen_range(0..n);
    //         if u != v {
    //             g.add_edge(u, v);
    //         }
    //     }
    // }
    //
    // g
    todo!()
}

pub fn generate_power_law_graph(n: usize) -> Graph {
    // TODO: Generate power-law (scale-free) graph
    //
    // Use preferential attachment (Barabási-Albert model):
    // - Start with small complete graph
    // - Add vertices one by one
    // - Connect new vertex to existing with probability ∝ degree
    //
    // This creates hubs (high-degree vertices)
    todo!()
}
```

---

### Checkpoint Tests

```rust
#[test]
fn test_graph_creation() {
    let mut g = Graph::new(5);
    g.add_edge(0, 1);
    g.add_edge(0, 2);
    g.add_edge(1, 3);

    assert_eq!(g.degree(0), 2);
    assert_eq!(g.degree(1), 1);
    assert_eq!(g.num_edges, 3);
}

#[test]
fn test_csr_conversion() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1);
    g.add_edge(0, 2);
    g.add_edge(1, 2);

    let csr = GraphCSR::from_graph(&g);

    assert_eq!(csr.neighbors(0), &[1, 2]);
    assert_eq!(csr.neighbors(1), &[2]);
}

#[test]
fn test_bfs_simple() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1);
    g.add_edge(1, 2);
    g.add_edge(2, 3);

    let distances = bfs(&g, 0);

    assert_eq!(distances[0], Some(0));
    assert_eq!(distances[1], Some(1));
    assert_eq!(distances[2], Some(2));
    assert_eq!(distances[3], Some(3));
}

#[test]
fn test_bfs_disconnected() {
    let mut g = Graph::new(4);
    g.add_edge(0, 1);
    // 2 and 3 are disconnected

    let distances = bfs(&g, 0);

    assert_eq!(distances[0], Some(0));
    assert_eq!(distances[1], Some(1));
    assert_eq!(distances[2], None);  // Unreachable
    assert_eq!(distances[3], None);
}

#[test]
fn benchmark_sequential_bfs() {
    use std::time::Instant;

    let sizes = vec![1_000, 10_000, 100_000];

    for size in sizes {
        let g = generate_random_graph(size, 10);

        let start = Instant::now();
        let _ = bfs(&g, 0);
        let time = start.elapsed();

        println!("BFS on {} vertices: {:?}", size, time);
    }
}

#[test]
fn test_power_law_graph() {
    let g = generate_power_law_graph(1000);

    // Check degree distribution
    let mut degrees: Vec<_> = (0..1000).map(|v| g.degree(v)).collect();
    degrees.sort();

    let max_degree = degrees[999];
    let median_degree = degrees[500];

    println!("Power-law graph:");
    println!("  Max degree: {}", max_degree);
    println!("  Median degree: {}", median_degree);

    // Power-law should have high-degree hubs
    assert!(max_degree > median_degree * 10);
}
```

## Milestone 2: Level-Synchronous Parallel BFS

### Introduction

**Why Milestone 1 Is Not Enough:**
Sequential BFS processes one vertex at a time. For large graphs, this is slow. All vertices in the same level are independent and can be processed in parallel!

**What We're Improving:**
Implement level-synchronous parallel BFS: process all vertices at each level in parallel using Rayon.

**Parallelization Strategy:**
```
Level 0: [v0]             (1 vertex)
         ↓ (parallel)
Level 1: [v1, v2, v3]     (3 vertices, process in parallel)
         ↓ (parallel)
Level 2: [v4, v5, ..., v99]  (96 vertices, process in parallel)
```

**Challenge:** Different levels have different sizes → load imbalance

**Expected Speedup:** 3-5x on 8 cores (limited by frontier size variability)

### Architecture

**Key Functions:**
- `parallel_bfs(graph: &Graph, start: usize) -> Vec<Option<usize>>` - Parallel BFS
- `process_frontier_parallel(graph: &Graph, frontier: &[usize]) -> Vec<usize>` - Expand frontier
- Use `rayon::par_iter()` for parallel frontier processing

**Synchronization:**
- Need atomic operations for visited array
- Use `AtomicBool` or `DashMap` for thread-safe visited tracking

**Role Each Plays:**
- Frontier: Current level vertices
- Visited: Prevent revisiting (race condition!)
- Atomics: Thread-safe marking
- Rayon: Automatic work distribution

### Checkpoint Tests

```rust
#[test]
fn test_parallel_bfs_correctness() {
    let g = generate_random_graph(1000, 10);

    let seq_dist = bfs(&g, 0);
    let par_dist = parallel_bfs(&g, 0);

    assert_eq!(seq_dist, par_dist);
}

#[test]
fn benchmark_parallel_bfs() {
    use std::time::Instant;

    let sizes = vec![10_000, 100_000, 1_000_000];

    for size in sizes {
        let g = generate_random_graph(size, 10);

        let start = Instant::now();
        let _ = bfs(&g, 0);
        let seq_time = start.elapsed();

        let start = Instant::now();
        let _ = parallel_bfs(&g, 0);
        let par_time = start.elapsed();

        let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();

        println!("BFS on {} vertices:", size);
        println!("  Sequential: {:?}", seq_time);
        println!("  Parallel:   {:?} ({:.2}x speedup)", par_time, speedup);
    }
}

#[test]
fn test_frontier_growth() {
    let g = generate_power_law_graph(10000);

    let levels = bfs_levels(&g, 0);

    println!("\nFrontier size by level:");
    for (level, vertices) in levels.iter().enumerate() {
        println!("  Level {}: {} vertices", level, vertices.len());
    }

    // Power-law graph should have explosive growth then decay
}
```

### Starter Code

```rust
use rayon::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn parallel_bfs(graph: &Graph, start: usize) -> Vec<Option<usize>> {
    // TODO: Parallel BFS
    //
    // let mut distances = vec![None; graph.num_vertices];
    // let visited: Vec<AtomicBool> = (0..graph.num_vertices)
    //     .map(|_| AtomicBool::new(false))
    //     .collect();
    //
    // distances[start] = Some(0);
    // visited[start].store(true, Ordering::Relaxed);
    //
    // let mut frontier = vec![start];
    // let mut level = 0;
    //
    // while !frontier.is_empty() {
    //     level += 1;
    //
    //     // PARALLEL: Process frontier
    //     let next_frontier: Vec<_> = frontier
    //         .par_iter()
    //         .flat_map(|&u| {
    //             graph.neighbors(u)
    //                 .iter()
    //                 .filter_map(|&v| {
    //                     // Atomic test-and-set
    //                     if !visited[v].swap(true, Ordering::Relaxed) {
    //                         Some(v)
    //                     } else {
    //                         None
    //                     }
    //                 })
    //                 .collect::<Vec<_>>()
    //         })
    //         .collect();
    //
    //     // Set distances for next frontier
    //     for &v in &next_frontier {
    //         distances[v] = Some(level);
    //     }
    //
    //     frontier = next_frontier;
    // }
    //
    // distances
    todo!()
}
```

---

## Milestone 3: Parallel Shortest Path (Delta-Stepping)

### Introduction

**Why Milestone 2 Is Not Enough:**
BFS only works for unweighted graphs. For weighted graphs, we need Dijkstra or similar. But Dijkstra is inherently sequential (priority queue). Delta-stepping is a parallelizable alternative.

**What We're Improving:**
Implement delta-stepping algorithm: relaxation-based shortest path with configurable delta parameter.

**Delta-Stepping:**
```
Partition vertices into buckets by distance range:
Bucket 0: [0, Δ)
Bucket 1: [Δ, 2Δ)
Bucket 2: [2Δ, 3Δ)
...

Process each bucket in parallel
```

**Expected Speedup:** 4-6x on weighted graphs

### Architecture

**Structs:**
- `WeightedGraph` - Graph with edge weights
  - **Field** `adjacency: Vec<Vec<(usize, f32)>>` - (neighbor, weight)
  - **Function** `add_weighted_edge(&mut self, u: usize, v: usize, w: f32)`

**Key Functions:**
- `dijkstra(graph: &WeightedGraph, start: usize) -> Vec<f32>` - Sequential baseline
- `delta_stepping(graph: &WeightedGraph, start: usize, delta: f32) -> Vec<f32>` - Parallel
- `relax_edges(...)` - Edge relaxation step

**Role Each Plays:**
- Buckets: Group vertices by distance range
- Delta: Tuning parameter (trade-off: work vs parallelism)
- Relaxation: Update distances if shorter path found

### Checkpoint Tests

```rust
#[test]
fn test_dijkstra() {
    let mut g = WeightedGraph::new(4);
    g.add_weighted_edge(0, 1, 1.0);
    g.add_weighted_edge(1, 2, 2.0);
    g.add_weighted_edge(0, 2, 4.0);

    let dist = dijkstra(&g, 0);

    assert_eq!(dist[0], 0.0);
    assert_eq!(dist[1], 1.0);
    assert_eq!(dist[2], 3.0);  // via 1, not direct
}

#[test]
fn test_delta_stepping_correctness() {
    let g = generate_weighted_random_graph(1000, 10);

    let dij_dist = dijkstra(&g, 0);
    let delta_dist = delta_stepping(&g, 0, 1.0);

    for i in 0..1000 {
        assert!((dij_dist[i] - delta_dist[i]).abs() < 0.01);
    }
}

#[test]
fn benchmark_delta_stepping() {
    use std::time::Instant;

    let g = generate_weighted_random_graph(100_000, 10);

    let start = Instant::now();
    let _ = dijkstra(&g, 0);
    let dij_time = start.elapsed();

    let start = Instant::now();
    let _ = delta_stepping(&g, 0, 1.0);
    let delta_time = start.elapsed();

    println!("Dijkstra: {:?}", dij_time);
    println!("Delta-stepping: {:?} ({:.2}x speedup)",
        delta_time, dij_time.as_secs_f64() / delta_time.as_secs_f64());
}
```

### Starter Code

```rust
#[derive(Debug, Clone)]
pub struct WeightedGraph {
    adjacency: Vec<Vec<(usize, f32)>>,  // (neighbor, weight)
    num_vertices: usize,
}

impl WeightedGraph {
    pub fn new(n: usize) -> Self {
        Self {
            adjacency: vec![Vec::new(); n],
            num_vertices: n,
        }
    }

    pub fn add_weighted_edge(&mut self, u: usize, v: usize, weight: f32) {
        self.adjacency[u].push((v, weight));
    }

    pub fn neighbors(&self, v: usize) -> &[(usize, f32)] {
        &self.adjacency[v]
    }
}

pub fn dijkstra(graph: &WeightedGraph, start: usize) -> Vec<f32> {
    // TODO: Sequential Dijkstra
    //
    // use std::collections::BinaryHeap;
    //
    // let mut dist = vec![f32::INFINITY; graph.num_vertices];
    // let mut heap = BinaryHeap::new();
    //
    // dist[start] = 0.0;
    // heap.push((Reverse(0.0), start));
    //
    // while let Some((Reverse(d), u)) = heap.pop() {
    //     if d > dist[u] { continue; }
    //
    //     for &(v, weight) in graph.neighbors(u) {
    //         let new_dist = dist[u] + weight;
    //         if new_dist < dist[v] {
    //             dist[v] = new_dist;
    //             heap.push((Reverse(new_dist), v));
    //         }
    //     }
    // }
    //
    // dist
    todo!()
}

pub fn delta_stepping(graph: &WeightedGraph, start: usize, delta: f32) -> Vec<f32> {
    // TODO: Parallel delta-stepping
    //
    // 1. Initialize distances
    // 2. Create buckets indexed by ⌊distance/delta⌋
    // 3. While buckets non-empty:
    //    a. Find first non-empty bucket
    //    b. Process all vertices in bucket (PARALLEL)
    //    c. Relax edges, add to appropriate buckets
    //
    // Key: Vertices in same bucket processed in parallel
    todo!()
}

pub fn generate_weighted_random_graph(n: usize, avg_degree: usize) -> WeightedGraph {
    // TODO: Generate random weighted graph
    // Random weights in range [0.1, 10.0]
    todo!()
}
```

---

## Milestone 4: PageRank Algorithm

### Introduction

**Why Milestone 3 Is Not Enough:**
BFS and shortest path are traversal algorithms. Many graph analytics require iterative computation: PageRank, clustering, centrality measures.

**What We're Improving:**
Implement PageRank: iterative algorithm that computes importance scores for web pages (vertices).

**PageRank Formula:**
```
PR(v) = (1-d)/N + d × Σ PR(u)/outdegree(u)
                      u→v

d = damping factor (typically 0.85)
N = total vertices
```

**Parallelization:** Each iteration updates all vertices independently → embarrassingly parallel!

**Expected Speedup:** 6-8x (high degree of parallelism)

### Architecture

**Key Functions:**
- `pagerank(graph: &Graph, iterations: usize, damping: f32) -> Vec<f32>` - Sequential
- `parallel_pagerank(graph: &Graph, iterations: usize, damping: f32) -> Vec<f32>` - Parallel
- `pagerank_until_convergence(...)` - Stop when ranks stabilize

**Convergence Check:**
```
if max_change < epsilon {
    break;  // Converged
}
```

**Role Each Plays:**
- Damping factor: Probability of random jump
- Iterations: Trade-off accuracy vs time
- Convergence: Adaptive termination

### Checkpoint Tests

```rust
#[test]
fn test_pagerank_simple() {
    // Simple graph: 0 → 1 → 2 → 0 (cycle)
    let mut g = Graph::new(3);
    g.add_edge(0, 1);
    g.add_edge(1, 2);
    g.add_edge(2, 0);

    let ranks = pagerank(&g, 100, 0.85);

    // All vertices should have equal rank (symmetric)
    assert!((ranks[0] - 1.0/3.0).abs() < 0.01);
    assert!((ranks[1] - 1.0/3.0).abs() < 0.01);
    assert!((ranks[2] - 1.0/3.0).abs() < 0.01);
}

#[test]
fn test_pagerank_hub() {
    // Hub: 0 ← 1, 2, 3 (all point to 0)
    let mut g = Graph::new(4);
    g.add_edge(1, 0);
    g.add_edge(2, 0);
    g.add_edge(3, 0);

    let ranks = pagerank(&g, 100, 0.85);

    // Vertex 0 should have highest rank
    assert!(ranks[0] > ranks[1]);
    assert!(ranks[0] > ranks[2]);
    assert!(ranks[0] > ranks[3]);
}

#[test]
fn benchmark_pagerank() {
    use std::time::Instant;

    let g = generate_power_law_graph(100_000);

    let start = Instant::now();
    let _ = pagerank(&g, 20, 0.85);
    let seq_time = start.elapsed();

    let start = Instant::now();
    let _ = parallel_pagerank(&g, 20, 0.85);
    let par_time = start.elapsed();

    println!("PageRank (20 iterations, 100k vertices):");
    println!("  Sequential: {:?}", seq_time);
    println!("  Parallel:   {:?} ({:.2}x speedup)",
        par_time, seq_time.as_secs_f64() / par_time.as_secs_f64());
}
```

### Starter Code

```rust
pub fn pagerank(graph: &Graph, iterations: usize, damping: f32) -> Vec<f32> {
    // TODO: Sequential PageRank
    //
    // let n = graph.num_vertices;
    // let mut ranks = vec![1.0 / n as f32; n];
    // let mut new_ranks = vec![0.0; n];
    //
    // for _ in 0..iterations {
    //     for v in 0..n {
    //         let mut sum = 0.0;
    //
    //         // Sum contributions from in-neighbors
    //         for u in 0..n {
    //             if graph.neighbors(u).contains(&v) {
    //                 sum += ranks[u] / graph.degree(u) as f32;
    //             }
    //         }
    //
    //         new_ranks[v] = (1.0 - damping) / n as f32 + damping * sum;
    //     }
    //
    //     std::mem::swap(&mut ranks, &mut new_ranks);
    // }
    //
    // ranks
    todo!()
}

pub fn parallel_pagerank(graph: &Graph, iterations: usize, damping: f32) -> Vec<f32> {
    // TODO: Parallel PageRank
    //
    // Same algorithm, but parallelize vertex updates
    //
    // use rayon::prelude::*;
    //
    // for _ in 0..iterations {
    //     new_ranks.par_iter_mut().enumerate().for_each(|(v, rank)| {
    //         let mut sum = 0.0;
    //         // ... compute rank
    //         *rank = (1.0 - damping) / n as f32 + damping * sum;
    //     });
    //
    //     std::mem::swap(&mut ranks, &mut new_ranks);
    // }
    todo!()
}

pub fn pagerank_until_convergence(
    graph: &Graph,
    damping: f32,
    epsilon: f32
) -> (Vec<f32>, usize) {
    // TODO: Adaptive PageRank (stop when converged)
    //
    // let mut iterations = 0;
    // loop {
    //     // Update ranks
    //     // ...
    //
    //     // Check convergence
    //     let max_change = ranks.iter().zip(&new_ranks)
    //         .map(|(r1, r2)| (r1 - r2).abs())
    //         .fold(0.0f32, f32::max);
    //
    //     iterations += 1;
    //     if max_change < epsilon {
    //         break;
    //     }
    // }
    //
    // (ranks, iterations)
    todo!()
}
```

---

## Milestone 5: Connected Components with Union-Find

### Introduction

**Why Milestone 4 Is Not Enough:**
PageRank assumes connected graph. Many graphs have multiple components. Need to identify connected components efficiently.

**What We're Improving:**
Implement parallel union-find (disjoint set union) for connected components.

**Union-Find:**
- Each vertex has parent pointer
- Find: Follow pointers to root
- Union: Link roots together
- Path compression: Optimize find

**Parallelization Challenge:** Concurrent unions can conflict → need atomic CAS

**Expected Speedup:** 3-5x (limited by sequential bottleneck)

### Architecture

**Structs:**
- `UnionFind` - Disjoint set data structure
  - **Field** `parent: Vec<AtomicUsize>` - Parent pointers
  - **Field** `rank: Vec<AtomicUsize>` - Tree heights
  - **Function** `new(n: usize) -> Self`
  - **Function** `find(&self, v: usize) -> usize` - Find root
  - **Function** `union(&self, u: usize, v: usize) -> bool` - Merge components

**Key Functions:**
- `connected_components(graph: &Graph) -> Vec<usize>` - Component IDs
- `parallel_connected_components(graph: &Graph) -> Vec<usize>` - Parallel

### Checkpoint Tests

```rust
#[test]
fn test_union_find() {
    let uf = UnionFind::new(5);

    uf.union(0, 1);
    uf.union(2, 3);

    assert_eq!(uf.find(0), uf.find(1));
    assert_eq!(uf.find(2), uf.find(3));
    assert_ne!(uf.find(0), uf.find(2));
}

#[test]
fn test_connected_components() {
    let mut g = Graph::new(6);
    // Component 1: 0-1-2
    g.add_edge(0, 1);
    g.add_edge(1, 2);
    // Component 2: 3-4
    g.add_edge(3, 4);
    // Component 3: 5 (isolated)

    let components = connected_components(&g);

    assert_eq!(components[0], components[1]);
    assert_eq!(components[1], components[2]);
    assert_eq!(components[3], components[4]);
    assert_ne!(components[0], components[3]);
}
```

### Starter Code

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct UnionFind {
    parent: Vec<AtomicUsize>,
    rank: Vec<AtomicUsize>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self {
        Self {
            parent: (0..n).map(|i| AtomicUsize::new(i)).collect(),
            rank: (0..n).map(|_| AtomicUsize::new(0)).collect(),
        }
    }

    pub fn find(&self, mut v: usize) -> usize {
        // TODO: Find with path compression
        //
        // loop {
        //     let parent = self.parent[v].load(Ordering::Relaxed);
        //     if parent == v {
        //         return v;
        //     }
        //     v = parent;
        // }
        todo!()
    }

    pub fn union(&self, u: usize, v: usize) -> bool {
        // TODO: Union by rank with CAS
        //
        // let root_u = self.find(u);
        // let root_v = self.find(v);
        //
        // if root_u == root_v {
        //     return false;  // Already in same set
        // }
        //
        // // Union by rank
        // let rank_u = self.rank[root_u].load(Ordering::Relaxed);
        // let rank_v = self.rank[root_v].load(Ordering::Relaxed);
        //
        // if rank_u < rank_v {
        //     self.parent[root_u].store(root_v, Ordering::Relaxed);
        // } else if rank_u > rank_v {
        //     self.parent[root_v].store(root_u, Ordering::Relaxed);
        // } else {
        //     self.parent[root_v].store(root_u, Ordering::Relaxed);
        //     self.rank[root_u].fetch_add(1, Ordering::Relaxed);
        // }
        //
        // true
        todo!()
    }
}

pub fn connected_components(graph: &Graph) -> Vec<usize> {
    // TODO: Find connected components
    //
    // let uf = UnionFind::new(graph.num_vertices);
    //
    // for u in 0..graph.num_vertices {
    //     for &v in graph.neighbors(u) {
    //         uf.union(u, v);
    //     }
    // }
    //
    // // Map to component IDs
    // (0..graph.num_vertices).map(|v| uf.find(v)).collect()
    todo!()
}

pub fn parallel_connected_components(graph: &Graph) -> Vec<usize> {
    // TODO: Parallel component finding
    //
    // Process edges in parallel, using atomic union-find
    //
    // use rayon::prelude::*;
    //
    // let uf = UnionFind::new(graph.num_vertices);
    //
    // (0..graph.num_vertices).into_par_iter().for_each(|u| {
    //     for &v in graph.neighbors(u) {
    //         uf.union(u, v);
    //     }
    // });
    //
    // (0..graph.num_vertices).map(|v| uf.find(v)).collect()
    todo!()
}
```

---

## Milestone 6: Work Stealing and Load Balancing

### Introduction

**Why Milestone 5 Is Not Enough:**
All previous milestones assume relatively balanced work. Power-law graphs have extreme imbalance: 1% of vertices do 90% of work.

**What We're Improving:**
Implement explicit work stealing for severely imbalanced graphs. Measure load balance metrics.

**Work Stealing:**
- Each thread has local queue
- When idle, steal work from busy threads
- Rayon does this automatically, but we'll measure it

**Expected Improvement:** Better utilization on skewed graphs

### Architecture

**Metrics:**
- `LoadBalanceMetrics` - Measure work distribution
  - **Field** `work_per_thread: Vec<usize>` - Work done by each thread
  - **Function** `imbalance_factor() -> f64` - max/avg work ratio
  - **Function** `print_report()`

**Key Functions:**
- `measure_work_distribution(...)` - Instrument parallel algorithm
- `compare_load_balancing(...)` - Compare strategies

### Checkpoint Tests

```rust
#[test]
fn test_load_balance_metrics() {
    let g = generate_power_law_graph(100_000);

    let metrics = measure_work_distribution(&g);

    println!("\nLoad Balance Report:");
    metrics.print_report();

    // Check that work stealing helps
    assert!(metrics.imbalance_factor() < 2.0);  // Max is < 2x average
}
```

### Starter Code

```rust
pub struct LoadBalanceMetrics {
    work_per_thread: Vec<usize>,
}

impl LoadBalanceMetrics {
    pub fn imbalance_factor(&self) -> f64 {
        let max_work = *self.work_per_thread.iter().max().unwrap() as f64;
        let avg_work = self.work_per_thread.iter().sum::<usize>() as f64
                     / self.work_per_thread.len() as f64;
        max_work / avg_work
    }

    pub fn print_report(&self) {
        println!("Work per thread:");
        for (i, &work) in self.work_per_thread.iter().enumerate() {
            println!("  Thread {}: {} units", i, work);
        }
        println!("Imbalance factor: {:.2}x", self.imbalance_factor());
    }
}

pub fn measure_work_distribution(graph: &Graph) -> LoadBalanceMetrics {
    // TODO: Instrument BFS to measure work per thread
    todo!()
}
```

---

## Complete Working Example

*(Similar structure to Project 1, implementing all the algorithms)*

This completes Project 2: Parallel Graph Processing Engine!
