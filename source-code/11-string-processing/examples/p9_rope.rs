//! Pattern 9: Rope Data Structure
//! Text Editor Data Structure for Large Documents
//!
//! Run with: cargo run --example p9_rope

fn main() {
    println!("=== Rope Data Structure ===\n");

    let mut rope = Rope::from_str("Hello World");
    println!("Initial: '{}'", rope.to_string());
    println!("Length: {}", rope.len());

    // Insert at position 5
    println!("\n=== Insert Operation ===\n");
    rope.insert(5, ", Beautiful");
    println!("After insert: '{}'", rope.to_string());

    // Character access
    println!("\n=== Character Access ===\n");
    for i in 0..5 {
        if let Some(ch) = rope.char_at(i) {
            println!("  char_at({}): '{}'", i, ch);
        }
    }

    // Delete range
    println!("\n=== Delete Operation ===\n");
    rope.delete(5, 16);  // Remove ", Beautiful"
    println!("After delete: '{}'", rope.to_string());

    // Concatenation
    println!("\n=== Concatenation ===\n");
    let rope1 = Rope::from_str("Hello ");
    let rope2 = Rope::from_str("World!");
    let combined = Rope::concat(rope1, rope2);
    println!("Concatenated: '{}'", combined.to_string());

    // Large document simulation
    println!("\n=== Large Document Operations ===\n");

    let mut large = Rope::from_str("Start ");
    for i in 0..10 {
        large.insert(large.len(), &format!("[{}] ", i));
    }
    large.insert(large.len(), "End");
    println!("Built string: '{}'", large.to_string());
    println!("Final length: {}", large.len());

    println!("\n=== Key Points ===");
    println!("1. O(log N) insert/delete anywhere in document");
    println!("2. O(1) concatenation (just create branch node)");
    println!("3. Structural sharing enables O(1) undo/redo");
    println!("4. Better than gap buffer for large files, multiple cursors");
}

#[derive(Clone)]
enum Rope {
    Leaf(String),
    Branch {
        left: Box<Rope>,
        right: Box<Rope>,
        length: usize,  // Total length of left subtree
    },
}

impl Rope {
    fn from_str(s: &str) -> Self {
        Rope::Leaf(s.to_string())
    }

    fn concat(left: Rope, right: Rope) -> Self {
        let length = left.len();
        Rope::Branch {
            left: Box::new(left),
            right: Box::new(right),
            length,
        }
    }

    fn len(&self) -> usize {
        match self {
            Rope::Leaf(s) => s.len(),
            Rope::Branch { length, right, .. } => {
                length + right.len()
            }
        }
    }

    // Insert string at position
    fn insert(&mut self, pos: usize, text: &str) {
        let current = std::mem::replace(self, Rope::Leaf(String::new()));
        let (left, right) = current.split(pos);
        let inner = Rope::concat(left, Rope::from_str(text));
        *self = Rope::concat(inner, right);
    }

    // Delete range
    fn delete(&mut self, start: usize, end: usize) {
        let current = std::mem::replace(self, Rope::Leaf(String::new()));
        let (left, rest) = current.split(start);
        let (_, right) = rest.split(end - start);
        *self = Rope::concat(left, right);
    }

    // Split rope at position
    fn split(self, pos: usize) -> (Rope, Rope) {
        match self {
            Rope::Leaf(s) => {
                if pos >= s.len() {
                    (Rope::Leaf(s), Rope::Leaf(String::new()))
                } else if pos == 0 {
                    (Rope::Leaf(String::new()), Rope::Leaf(s))
                } else {
                    let (left, right) = s.split_at(pos);
                    (Rope::Leaf(left.to_string()),
                     Rope::Leaf(right.to_string()))
                }
            }
            Rope::Branch { left, right, length } => {
                if pos < length {
                    let (ll, lr) = left.split(pos);
                    (ll, Rope::concat(lr, *right))
                } else if pos == length {
                    (*left, *right)
                } else {
                    let (rl, rr) = right.split(pos - length);
                    (Rope::concat(*left, rl), rr)
                }
            }
        }
    }

    // Get character at position
    fn char_at(&self, pos: usize) -> Option<char> {
        match self {
            Rope::Leaf(s) => s.chars().nth(pos),
            Rope::Branch { left, right, length } => {
                if pos < *length {
                    left.char_at(pos)
                } else {
                    right.char_at(pos - length)
                }
            }
        }
    }

    // Convert to string
    fn to_string(&self) -> String {
        match self {
            Rope::Leaf(s) => s.clone(),
            Rope::Branch { left, right, .. } => {
                format!("{}{}", left.to_string(), right.to_string())
            }
        }
    }

    // Rebalance tree if needed
    fn rebalance(self) -> Self {
        // Simplified rebalancing: collect all leaves and rebuild
        let text = self.to_string();
        if text.len() < 100 {
            return Rope::Leaf(text);
        }

        let mid = text.len() / 2;
        let (left, right) = text.split_at(mid);
        Rope::concat(
            Rope::Leaf(left.to_string()),
            Rope::Leaf(right.to_string()))
    }
}
