// Pattern 1: Pattern Matching with Enums and Nested References
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

impl<T> Tree<T> {
    fn left(&self) -> Option<&Tree<T>> {
        match self {
            Tree::Node(left, _) => Some(left),
            // left: &Box<Tree<T>>, deref coercion gives &Tree<T>
            Tree::Leaf(_) => None,
        }
    }

    fn left_mut(&mut self) -> Option<&mut Tree<T>> {
        match self {
            Tree::Node(ref mut left, _) => Some(left),
            // Explicit ref mut needed when matching on &mut self
            // to clarify we want &mut Box, not to move Box
            Tree::Leaf(_) => None,
        }
    }
}

fn main() {
    let tree = Tree::Node(
        Box::new(Tree::Leaf(1)),
        Box::new(Tree::Leaf(2)),
    );

    if let Some(left) = tree.left() {
        println!("Has left subtree");
        let _ = left;
    }

    let mut tree2 = Tree::Node(
        Box::new(Tree::Leaf(10)),
        Box::new(Tree::Leaf(20)),
    );

    if let Some(_left) = tree2.left_mut() {
        println!("Has mutable left subtree");
    }

    println!("Enum patterns example completed");
}
