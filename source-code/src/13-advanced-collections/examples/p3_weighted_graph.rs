//! Pattern 3: Graph Representations
//! Adjacency List with Weighted Edges
//!
//! Run with: cargo run --example p3_weighted_graph

use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug, Clone)]
struct Edge<T> {
    to: T,
    weight: u32,
}

struct WeightedGraph<T> {
    adjacency: HashMap<T, Vec<Edge<T>>>,
    directed: bool,
}

impl<T> WeightedGraph<T>
where
    T: Eq + Hash + Clone,
{
    fn new(directed: bool) -> Self {
        Self {
            adjacency: HashMap::new(),
            directed,
        }
    }

    fn add_vertex(&mut self, vertex: T) {
        self.adjacency.entry(vertex).or_insert_with(Vec::new);
    }

    fn add_edge(&mut self, from: T, to: T, weight: u32) {
        self.adjacency
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push(Edge {
                to: to.clone(),
                weight,
            });

        if !self.directed {
            self.adjacency
                .entry(to)
                .or_insert_with(Vec::new)
                .push(Edge { to: from, weight });
        }
    }

    fn neighbors(&self, vertex: &T) -> Option<&Vec<Edge<T>>> {
        self.adjacency.get(vertex)
    }

    fn vertices(&self) -> Vec<&T> {
        self.adjacency.keys().collect()
    }

    fn edge_count(&self) -> usize {
        let total: usize = self.adjacency.values().map(|edges| edges.len()).sum();
        if self.directed {
            total
        } else {
            total / 2
        }
    }
}

//=========================
// Dijkstra's shortest path
//=========================
#[derive(Eq, PartialEq)]
struct State<T> {
    cost: u32,
    node: T,
}

impl<T: Eq> Ord for State<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // Min-heap
    }
}

impl<T: Eq> PartialOrd for State<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> WeightedGraph<T>
where
    T: Eq + Hash + Clone,
{
    fn dijkstra(&self, start: &T) -> HashMap<T, u32> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(start.clone(), 0);
        heap.push(State {
            cost: 0,
            node: start.clone(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            // Skip if we found a better path
            if let Some(&best) = distances.get(&node) {
                if cost > best {
                    continue;
                }
            }

            // Check neighbors
            if let Some(edges) = self.neighbors(&node) {
                for edge in edges {
                    let next_cost = cost + edge.weight;

                    let is_better = distances
                        .get(&edge.to)
                        .map_or(true, |&current| next_cost < current);

                    if is_better {
                        distances.insert(edge.to.clone(), next_cost);
                        heap.push(State {
                            cost: next_cost,
                            node: edge.to.clone(),
                        });
                    }
                }
            }
        }

        distances
    }

    fn shortest_path(&self, start: &T, end: &T) -> Option<(Vec<T>, u32)> {
        let mut distances: HashMap<T, u32> = HashMap::new();
        let mut previous: HashMap<T, T> = HashMap::new();
        let mut heap = BinaryHeap::new();

        distances.insert(start.clone(), 0);
        heap.push(State {
            cost: 0,
            node: start.clone(),
        });

        while let Some(State { cost, node }) = heap.pop() {
            if node == *end {
                // Reconstruct path
                let mut path = vec![end.clone()];
                let mut current = end;

                while let Some(prev) = previous.get(current) {
                    path.push(prev.clone());
                    current = prev;
                }

                path.reverse();
                return Some((path, cost));
            }

            if let Some(&best) = distances.get(&node) {
                if cost > best {
                    continue;
                }
            }

            if let Some(edges) = self.neighbors(&node) {
                for edge in edges {
                    let next_cost = cost + edge.weight;

                    let is_better = distances
                        .get(&edge.to)
                        .map_or(true, |&current| next_cost < current);

                    if is_better {
                        distances.insert(edge.to.clone(), next_cost);
                        previous.insert(edge.to.clone(), node.clone());
                        heap.push(State {
                            cost: next_cost,
                            node: edge.to.clone(),
                        });
                    }
                }
            }
        }

        None
    }
}

//===========================
// Real-world: Route planning
//===========================
fn main() {
    println!("=== Weighted Graph - Route Planning ===\n");

    let mut map = WeightedGraph::new(false);

    // Cities and distances (km)
    map.add_edge("SF", "LA", 383);
    map.add_edge("SF", "Portland", 635);
    map.add_edge("LA", "Phoenix", 373);
    map.add_edge("Portland", "Seattle", 173);
    map.add_edge("Phoenix", "Denver", 868);
    map.add_edge("Seattle", "Denver", 1316);
    map.add_edge("LA", "Denver", 1016);

    println!("Finding shortest paths from SF:\n");
    let distances = map.dijkstra(&"SF");

    for (city, distance) in &distances {
        println!("  SF -> {}: {}km", city, distance);
    }

    println!("\nShortest path SF -> Denver:");
    if let Some((path, distance)) = map.shortest_path(&"SF", &"Denver") {
        println!("  Path: {:?}", path);
        println!("  Distance: {}km", distance);
    }

    println!("\n=== Key Points ===");
    println!("1. Adjacency list: O(V + E) space");
    println!("2. Dijkstra's algorithm: O((V + E) log V)");
    println!("3. BinaryHeap as min-heap for priority queue");
    println!("4. HashMap-based graph supports any hashable node type");
}
