//! Pattern 4: Trie and Radix Tree Structures
//! Radix Tree for Compressed Trie
//!
//! Run with: cargo run --example p4_radix_tree

use std::collections::HashMap;

#[derive(Debug)]
struct RadixNode {
    children: HashMap<char, Box<RadixNode>>,
    edge_label: String,
    is_end: bool,
    value: Option<String>,
}

impl RadixNode {
    fn new(label: String) -> Self {
        Self {
            children: HashMap::new(),
            edge_label: label,
            is_end: false,
            value: None,
        }
    }
}

struct RadixTree {
    root: RadixNode,
    size: usize,
}

impl RadixTree {
    fn new() -> Self {
        Self {
            root: RadixNode::new(String::new()),
            size: 0,
        }
    }

    fn insert(&mut self, key: &str, value: String) {
        if key.is_empty() {
            return;
        }

        Self::insert_recursive(&mut self.root, key, value);
        self.size += 1;
    }

    fn insert_recursive(node: &mut RadixNode, key: &str, value: String) {
        if key.is_empty() {
            node.is_end = true;
            node.value = Some(value);
            return;
        }

        let first_char = key.chars().next().unwrap();

        // Find matching child
        if let Some(child) = node.children.get_mut(&first_char) {
            let label = child.edge_label.clone();
            let common_prefix_len = common_prefix_length(key, &label);

            if common_prefix_len == label.len() {
                // Full match: continue down
                let remaining = &key[common_prefix_len..];
                Self::insert_recursive(child, remaining, value);
            } else {
                // Partial match: split node
                let common = &label[..common_prefix_len];
                let old_suffix = &label[common_prefix_len..];
                let new_suffix = &key[common_prefix_len..];

                // Create new intermediate node
                let mut intermediate =
                    Box::new(RadixNode::new(common.to_string()));

                // Move old child under intermediate
                let old_child = node.children.remove(&first_char).unwrap();
                let old_first = old_suffix.chars().next().unwrap();

                let mut relocated = old_child;
                relocated.edge_label = old_suffix.to_string();
                intermediate.children.insert(old_first, relocated);

                // Add new branch
                if !new_suffix.is_empty() {
                    let new_first = new_suffix.chars().next().unwrap();
                    let mut new_node =
                        Box::new(RadixNode::new(new_suffix.to_string()));
                    new_node.is_end = true;
                    new_node.value = Some(value);
                    intermediate.children.insert(new_first, new_node);
                } else {
                    intermediate.is_end = true;
                    intermediate.value = Some(value);
                }

                node.children.insert(first_char, intermediate);
            }
        } else {
            // No matching child: create new
            let mut new_node = Box::new(RadixNode::new(key.to_string()));
            new_node.is_end = true;
            new_node.value = Some(value);
            node.children.insert(first_char, new_node);
        }
    }

    fn search(&self, key: &str) -> Option<&String> {
        Self::search_recursive(&self.root, key)
    }

    fn search_recursive<'a>(node: &'a RadixNode, key: &str) -> Option<&'a String> {
        if key.is_empty() {
            return if node.is_end {
                node.value.as_ref()
            } else {
                None
            };
        }

        let first_char = key.chars().next().unwrap();

        if let Some(child) = node.children.get(&first_char) {
            let label = &child.edge_label;
            let common_len = common_prefix_length(key, label);

            if common_len == label.len() {
                let remaining = &key[common_len..];
                Self::search_recursive(child, remaining)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn starts_with(&self, prefix: &str) -> Vec<String> {
        let mut results = Vec::new();
        Self::collect_with_prefix(&self.root, prefix, String::new(), &mut results);
        results
    }

    fn collect_with_prefix(
        node: &RadixNode,
        remaining_prefix: &str,
        current_key: String,
        results: &mut Vec<String>,
    ) {
        if remaining_prefix.is_empty() {
            // Collect all keys under this node
            Self::collect_all(node, current_key, results);
            return;
        }

        let first_char = remaining_prefix.chars().next().unwrap();

        if let Some(child) = node.children.get(&first_char) {
            let label = &child.edge_label;
            let common_len = common_prefix_length(remaining_prefix, label);

            let mut new_key = current_key.clone();
            new_key.push_str(&label[..common_len]);

            if common_len == label.len() {
                let new_remaining = &remaining_prefix[common_len..];
                Self::collect_with_prefix(child, new_remaining, new_key, results);
            } else if common_len == remaining_prefix.len() {
                // Prefix matches completely
                Self::collect_all(child, new_key, results);
            }
        }
    }

    fn collect_all(node: &RadixNode, current_key: String, results: &mut Vec<String>) {
        if node.is_end {
            results.push(current_key.clone());
        }

        for (_, child) in &node.children {
            let mut new_key = current_key.clone();
            new_key.push_str(&child.edge_label);
            Self::collect_all(child, new_key, results);
        }
    }

    fn len(&self) -> usize {
        self.size
    }
}

fn common_prefix_length(s1: &str, s2: &str) -> usize {
    s1.chars()
        .zip(s2.chars())
        .take_while(|(a, b)| a == b)
        .count()
}

//=============================
// Real-world: IP routing table
//=============================
struct RoutingTable {
    tree: RadixTree,
}

impl RoutingTable {
    fn new() -> Self {
        Self {
            tree: RadixTree::new(),
        }
    }

    fn add_route(&mut self, cidr: &str, gateway: &str) {
        self.tree.insert(cidr, gateway.to_string());
    }

    fn lookup(&self, ip: &str) -> Option<&String> {
        self.tree.search(ip)
    }

    fn routes_for_prefix(&self, prefix: &str) -> Vec<String> {
        self.tree.starts_with(prefix)
    }
}

fn main() {
    println!("=== Radix Tree ===\n");

    let mut tree = RadixTree::new();

    tree.insert("test", "value1".to_string());
    tree.insert("testing", "value2".to_string());
    tree.insert("team", "value3".to_string());
    tree.insert("toast", "value4".to_string());

    println!("Search 'test': {:?}", tree.search("test"));
    println!("Search 'testing': {:?}", tree.search("testing"));
    println!("Search 'team': {:?}", tree.search("team"));

    println!("\nKeys starting with 'te':");
    for key in tree.starts_with("te") {
        println!("  {}", key);
    }

    println!("\n=== IP Routing Table ===\n");

    let mut routing = RoutingTable::new();

    routing.add_route("192.168.1.0", "gateway1");
    routing.add_route("192.168.2.0", "gateway2");
    routing.add_route("192.168.1.100", "gateway3");

    println!("192.168.1.0: {:?}", routing.lookup("192.168.1.0"));
    println!("192.168.1.100: {:?}", routing.lookup("192.168.1.100"));

    println!("\nRoutes for '192.168.1':");
    for route in routing.routes_for_prefix("192.168.1") {
        println!("  {}", route);
    }

    println!("\n=== Key Points ===");
    println!("1. Radix tree compresses common prefixes");
    println!("2. More space-efficient than standard trie");
    println!("3. Used in IP routing tables and file systems");
    println!("4. Trade-off: more complex implementation");
}
