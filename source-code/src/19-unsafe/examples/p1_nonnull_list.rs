// Pattern 1: Null Pointer Optimization with NonNull
use std::ptr::NonNull;

/// A node in a linked list using NonNull for efficiency.
struct Node<T> {
    value: T,
    next: Option<NonNull<Node<T>>>,  // Same size as *mut Node<T>
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node { value, next: None }
    }
}

/// A simple singly-linked list.
struct LinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
    len: usize,
}

impl<T> LinkedList<T> {
    fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    fn push_back(&mut self, value: T) {
        let node = Box::new(Node::new(value));
        let node_ptr = NonNull::new(Box::into_raw(node)).unwrap();

        unsafe {
            if let Some(mut tail) = self.tail {
                tail.as_mut().next = Some(node_ptr);
            } else {
                self.head = Some(node_ptr);
            }

            self.tail = Some(node_ptr);
        }

        self.len += 1;
    }

    fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        let mut current = self.head;

        while let Some(node_ptr) = current {
            unsafe {
                let node = Box::from_raw(node_ptr.as_ptr());
                current = node.next;
            }
        }
    }
}

fn main() {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);
    println!("List length: {}", list.len());

    println!("NonNull linked list example completed");
}
