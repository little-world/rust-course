//! Pattern 4: Self-Referential Structs and Pin
//! Example: Safe Alternative Using Indices
//!
//! Run with: cargo run --example p4_indices

// Graph where nodes reference each other via indices instead of references.
// Indices remain valid even if the vector reallocates.
#[derive(Debug)]
struct Node {
    name: String,
    edges: Vec<usize>, // Indices into graph's `nodes` vector
}

struct Graph {
    nodes: Vec<Node>,
}

impl Graph {
    fn new() -> Self {
        Graph { nodes: Vec::new() }
    }

    fn add_node(&mut self, name: impl Into<String>) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node {
            name: name.into(),
            edges: Vec::new(),
        });
        index
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        if from < self.nodes.len() && to < self.nodes.len() {
            self.nodes[from].edges.push(to);
        }
    }

    fn get_node(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    fn neighbors(&self, index: usize) -> Vec<&Node> {
        self.nodes
            .get(index)
            .map(|node| {
                node.edges
                    .iter()
                    .filter_map(|&i| self.nodes.get(i))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn print_graph(&self) {
        for (i, node) in self.nodes.iter().enumerate() {
            let neighbors: Vec<_> = node
                .edges
                .iter()
                .filter_map(|&j| self.nodes.get(j).map(|n| n.name.as_str()))
                .collect();
            println!("  {} -> {:?}", node.name, neighbors);
        }
    }
}

// Arena-style allocation with indices
struct Arena<T> {
    items: Vec<T>,
}

impl<T> Arena<T> {
    fn new() -> Self {
        Arena { items: Vec::new() }
    }

    fn alloc(&mut self, item: T) -> usize {
        let index = self.items.len();
        self.items.push(item);
        index
    }

    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }
}

// Tree structure using indices
#[derive(Debug)]
struct TreeNode {
    value: i32,
    children: Vec<usize>, // Indices into the arena
}

struct Tree {
    arena: Arena<TreeNode>,
    root: Option<usize>,
}

impl Tree {
    fn new() -> Self {
        Tree {
            arena: Arena::new(),
            root: None,
        }
    }

    fn set_root(&mut self, value: i32) -> usize {
        let index = self.arena.alloc(TreeNode {
            value,
            children: Vec::new(),
        });
        self.root = Some(index);
        index
    }

    fn add_child(&mut self, parent: usize, value: i32) -> Option<usize> {
        let child_index = self.arena.alloc(TreeNode {
            value,
            children: Vec::new(),
        });
        self.arena.get_mut(parent)?.children.push(child_index);
        Some(child_index)
    }

    fn print_tree(&self) {
        if let Some(root) = self.root {
            self.print_node(root, 0);
        }
    }

    fn print_node(&self, index: usize, depth: usize) {
        if let Some(node) = self.arena.get(index) {
            let indent = "  ".repeat(depth);
            println!("{}value: {}", indent, node.value);
            for &child in &node.children {
                self.print_node(child, depth + 1);
            }
        }
    }
}

fn main() {
    println!("=== Graph Using Indices ===");
    // Usage: Indices stay valid even if vec reallocates; avoids self-reference.
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");
    let d = graph.add_node("D");

    graph.add_edge(a, b);
    graph.add_edge(a, c);
    graph.add_edge(b, c);
    graph.add_edge(c, d);

    println!("Graph structure:");
    graph.print_graph();

    println!("\nNeighbors of A:");
    for neighbor in graph.neighbors(a) {
        println!("  - {}", neighbor.name);
    }

    println!("\n=== Arena Allocation ===");
    let mut arena: Arena<String> = Arena::new();
    let hello = arena.alloc(String::from("hello"));
    let world = arena.alloc(String::from("world"));
    let rust = arena.alloc(String::from("rust"));

    println!("Item at {}: {:?}", hello, arena.get(hello));
    println!("Item at {}: {:?}", world, arena.get(world));
    println!("Item at {}: {:?}", rust, arena.get(rust));

    println!("\n=== Tree Using Indices ===");
    let mut tree = Tree::new();
    let root = tree.set_root(1);
    let child1 = tree.add_child(root, 2).unwrap();
    let child2 = tree.add_child(root, 3).unwrap();
    tree.add_child(child1, 4);
    tree.add_child(child1, 5);
    tree.add_child(child2, 6);

    println!("Tree structure:");
    tree.print_tree();

    println!("\n=== Why Indices Work ===");
    println!("- Indices are just numbers (usize), not references");
    println!("- They remain valid even if the underlying Vec reallocates");
    println!("- No lifetime annotations needed");
    println!("- Works naturally with the borrow checker");
    println!("- Common pattern in game engines, compilers, and databases");

    println!("\n=== Tradeoffs ===");
    println!("Pros:");
    println!("  - No lifetime complexity");
    println!("  - Data can be easily serialized");
    println!("  - Supports cyclic structures");
    println!("Cons:");
    println!("  - Bounds checking at runtime");
    println!("  - Indices can become invalid if items are removed");
    println!("  - Slight indirection overhead");
}
