//! Pattern 2: Associated Types vs Generics
//! Example: Associated Types with Bounds
//!
//! Run with: cargo run --example p2_associated_bounds

use std::fmt::Display;

// Associated types can have trait bounds
trait Graph {
    type Node: Display; // Node must implement Display
    type Edge: Clone;   // Edge must implement Clone

    fn nodes(&self) -> Vec<Self::Node>;
    fn edges(&self) -> Vec<Self::Edge>;
    fn node_count(&self) -> usize;
    fn edge_count(&self) -> usize;
}

// Implementation must satisfy the bounds
struct SimpleGraph {
    node_names: Vec<String>,
    connections: Vec<(usize, usize)>,
}

impl SimpleGraph {
    fn new() -> Self {
        SimpleGraph {
            node_names: Vec::new(),
            connections: Vec::new(),
        }
    }

    fn add_node(&mut self, name: &str) -> usize {
        let id = self.node_names.len();
        self.node_names.push(name.to_string());
        id
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.connections.push((from, to));
    }
}

impl Graph for SimpleGraph {
    type Node = String;         // String implements Display ✓
    type Edge = (usize, usize); // Tuple implements Clone ✓

    fn nodes(&self) -> Vec<String> {
        self.node_names.clone()
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        self.connections.clone()
    }

    fn node_count(&self) -> usize {
        self.node_names.len()
    }

    fn edge_count(&self) -> usize {
        self.connections.len()
    }
}

// Generic function that uses the bounds
fn print_graph<G: Graph>(graph: &G)
where
    G::Edge: std::fmt::Debug,
{
    println!("Nodes ({}):", graph.node_count());
    for node in graph.nodes() {
        println!("  - {}", node); // Works because Node: Display
    }

    println!("Edges ({}):", graph.edge_count());
    for edge in graph.edges() {
        let edge_copy = edge.clone(); // Works because Edge: Clone
        println!("  {:?}", edge_copy); // Works because Edge: Debug
    }
}

fn main() {
    // Usage: Associated type bounds ensure Node is Display, Edge is Clone.
    let mut graph = SimpleGraph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.add_edge(a, b);
    graph.add_edge(b, c);
    graph.add_edge(a, c);

    print_graph(&graph);

    // Direct iteration using the bounds
    println!("\nDirect node printing (using Display bound):");
    for node in graph.nodes() {
        println!("Node: {}", node);
    }

    println!("\nCloning edges (using Clone bound):");
    let edges = graph.edges();
    let edges_copy = edges.clone();
    println!("Original: {:?}", edges);
    println!("Cloned: {:?}", edges_copy);
}
