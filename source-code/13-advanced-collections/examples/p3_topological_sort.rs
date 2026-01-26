//! Pattern 3: Graph Representations
//! Topological Sort and Dependency Resolution
//!
//! Run with: cargo run --example p3_topological_sort

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

struct DirectedGraph<T> {
    adjacency: HashMap<T, Vec<T>>,
}

impl<T> DirectedGraph<T>
where
    T: Eq + Hash + Clone + std::fmt::Debug,
{
    fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    fn add_edge(&mut self, from: T, to: T) {
        self.adjacency
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push(to.clone());

        // Ensure 'to' vertex exists
        self.adjacency.entry(to).or_insert_with(Vec::new);
    }

    fn vertices(&self) -> Vec<&T> {
        self.adjacency.keys().collect()
    }

    // Kahn's algorithm for topological sort
    fn topological_sort(&self) -> Result<Vec<T>, String> {
        let mut in_degree: HashMap<T, usize> = HashMap::new();

        // Calculate in-degrees
        for vertex in self.vertices() {
            in_degree.entry(vertex.clone()).or_insert(0);
        }

        for edges in self.adjacency.values() {
            for to in edges {
                *in_degree.entry(to.clone()).or_insert(0) += 1;
            }
        }

        // Queue vertices with no incoming edges
        let mut queue: VecDeque<T> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(v, _)| v.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(vertex) = queue.pop_front() {
            result.push(vertex.clone());

            // Reduce in-degree for neighbors
            if let Some(edges) = self.adjacency.get(&vertex) {
                for to in edges {
                    if let Some(degree) = in_degree.get_mut(to) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(to.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.adjacency.len() {
            Err("Graph contains a cycle".to_string())
        } else {
            Ok(result)
        }
    }

    // DFS-based topological sort
    fn topological_sort_dfs(&self) -> Result<Vec<T>, String> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut result = Vec::new();

        for vertex in self.vertices() {
            if !visited.contains(vertex) {
                self.dfs_topo(
                    vertex,
                    &mut visited,
                    &mut rec_stack,
                    &mut result,
                )?;
            }
        }

        result.reverse();
        Ok(result)
    }

    fn dfs_topo(
        &self,
        vertex: &T,
        visited: &mut HashSet<T>,
        rec_stack: &mut HashSet<T>,
        result: &mut Vec<T>,
    ) -> Result<(), String> {
        visited.insert(vertex.clone());
        rec_stack.insert(vertex.clone());

        if let Some(edges) = self.adjacency.get(vertex) {
            for neighbor in edges {
                if !visited.contains(neighbor) {
                    self.dfs_topo(neighbor, visited, rec_stack, result)?;
                } else if rec_stack.contains(neighbor) {
                    return Err(format!("Cycle detected involving {:?}", neighbor));
                }
            }
        }

        rec_stack.remove(vertex);
        result.push(vertex.clone());
        Ok(())
    }

    fn has_cycle(&self) -> bool {
        self.topological_sort().is_err()
    }
}

//===============================================
// Real-world: Build system dependency resolution
//===============================================
struct BuildSystem {
    dependencies: DirectedGraph<String>,
}

impl BuildSystem {
    fn new() -> Self {
        Self {
            dependencies: DirectedGraph::new(),
        }
    }

    fn add_target(&mut self, target: String, depends_on: Vec<String>) {
        for dep in depends_on {
            self.dependencies.add_edge(dep, target.clone());
        }
    }

    fn build_order(&self) -> Result<Vec<String>, String> {
        self.dependencies.topological_sort()
    }

    fn check_cycles(&self) -> bool {
        self.dependencies.has_cycle()
    }
}

//=========================================
// Real-world: Course prerequisite planning
//=========================================
struct CoursePlanner {
    prerequisites: DirectedGraph<String>,
}

impl CoursePlanner {
    fn new() -> Self {
        Self {
            prerequisites: DirectedGraph::new(),
        }
    }

    fn add_course(&mut self, course: String, prerequisites: Vec<String>) {
        for prereq in prerequisites {
            self.prerequisites.add_edge(prereq, course.clone());
        }
    }

    fn course_order(&self) -> Result<Vec<String>, String> {
        self.prerequisites.topological_sort()
    }

    fn can_complete(&self) -> bool {
        !self.prerequisites.has_cycle()
    }
}

fn main() {
    println!("=== Build System ===\n");

    let mut build = BuildSystem::new();

    build.add_target(
        "main.o".into(),
        vec!["main.c".into(), "util.h".into()]);
    build.add_target(
        "util.o".into(),
        vec!["util.c".into(), "util.h".into()]);
    build.add_target(
        "program".into(),
        vec!["main.o".into(), "util.o".into()]);

    match build.build_order() {
        Ok(order) => {
            println!("Build order:");
            for (i, target) in order.iter().enumerate() {
                println!("  {}. {}", i + 1, target);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Course Planning ===\n");

    let mut planner = CoursePlanner::new();

    planner.add_course(
        "Data Structures".into(), vec!["Programming 101".into()]);
    planner.add_course(
        "Algorithms".into(), vec!["Data Structures".into()]);
    planner.add_course(
        "AI".into(),
        vec!["Algorithms".into(), "Linear Algebra".into()]);
    planner.add_course(
        "Machine Learning".into(),
        vec!["AI".into(), "Statistics".into()]);

    if planner.can_complete() {
        match planner.course_order() {
            Ok(order) => {
                println!("Suggested course order:");
                for (i, course) in order.iter().enumerate() {
                    println!("  Semester {}: {}", (i / 2) + 1, course);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Cannot complete - circular prerequisites!");
    }

    println!("\n=== Key Points ===");
    println!("1. Kahn's algorithm: BFS-based, O(V + E)");
    println!("2. DFS-based: detects cycles during traversal");
    println!("3. Build systems use topological sort for dependencies");
    println!("4. Course planning ensures no circular prerequisites");
}
