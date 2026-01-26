// Pattern 8: Rc<RefCell<T>> for Shared Mutable Data
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    value: i32,
    neighbors: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            neighbors: RefCell::new(Vec::new()),
        })
    }

    fn add_neighbor(&self, neighbor: Rc<Node>) {
        self.neighbors.borrow_mut().push(neighbor);
    }
}

fn main() {
    let a = Node::new(1);
    let b = Node::new(2);
    a.add_neighbor(Rc::clone(&b));
    b.add_neighbor(Rc::clone(&a)); // Cycle is allowed

    println!("Node a has value {} with {} neighbors", a.value, a.neighbors.borrow().len());
    println!("Node b has value {} with {} neighbors", b.value, b.neighbors.borrow().len());
    println!("Rc<RefCell> example completed");
}
