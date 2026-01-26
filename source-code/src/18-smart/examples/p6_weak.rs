// Pattern 6: Reference Counting Optimization - Weak for Non-Owning References
use std::rc::{Rc, Weak};

struct Observer {
    subject: Weak<Vec<i32>>,
}

impl Observer {
    fn observe(&self) {
        // Temporarily upgrade to access data
        if let Some(data) = self.subject.upgrade() {
            println!("Observed: {} items", data.len());
        } else {
            println!("Subject is gone");
        }
        // Rc dropped immediately, no permanent ownership
    }
}

fn main() {
    // Usage
    let data = Rc::new(vec![1, 2, 3]);
    let observer = Observer { subject: Rc::downgrade(&data) };
    observer.observe();  // "Observed: 3 items"
    drop(data);
    observer.observe();  // "Subject is gone"

    println!("Weak reference example completed");
}
