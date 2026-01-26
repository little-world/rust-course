// Pattern 9: Tree with Parent Pointers
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct TreeNode {
    value: i32,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    fn add_child(parent: &Rc<TreeNode>, value: i32) -> Rc<TreeNode> {
        let child = TreeNode::new(value);
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }
}

fn main() {
    let root = TreeNode::new(1);
    let child = TreeNode::add_child(&root, 2);

    if let Some(parent) = child.parent.borrow().upgrade() {
        println!("Parent value: {}", parent.value);
    }

    println!("Root has {} children", root.children.borrow().len());
    println!("Weak reference example completed");
}
