// Pattern 1: Custom Smart Pointers - Copy-on-Write Pointer
use std::rc::Rc;
use std::ops::Deref;

struct CowPtr<T: Clone> {
    data: Rc<T>,
}

impl<T: Clone> CowPtr<T> {
    fn new(data: T) -> Self {
        CowPtr { data: Rc::new(data) }
    }

    fn modify<F: FnOnce(&mut T)>(&mut self, f: F) {
        // Clone only if shared
        if Rc::strong_count(&self.data) > 1 {
            self.data = Rc::new((*self.data).clone());
        }
        f(Rc::get_mut(&mut self.data).unwrap());
    }
}

impl<T: Clone> Deref for CowPtr<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.data }
}

impl<T: Clone> Clone for CowPtr<T> {
    fn clone(&self) -> Self {
        CowPtr { data: Rc::clone(&self.data) }
    }
}

fn main() {
    // Usage: Clones share data until modification
    let original = CowPtr::new(vec![1, 2, 3]);
    let mut copy = original.clone();  // Cheap: shares Rc
    copy.modify(|v| v.push(4));       // Clone happens here
    assert_eq!(original.len(), 3);    // Original unchanged
    assert_eq!(copy.len(), 4);

    println!("Copy-on-write pointer example completed");
}
