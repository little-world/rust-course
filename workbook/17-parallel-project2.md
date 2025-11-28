# Chapter 17: Parallel Programming - Project 2

## Project 2: Parallel Graph Processing Engine

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
