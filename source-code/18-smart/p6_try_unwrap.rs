// Pattern 6: Reference Counting Optimization - try_unwrap for Sole Owner
use std::rc::Rc;

fn make_owned(data: Rc<Vec<i32>>) -> Vec<i32> {
    // If we're the only owner, unwrap without cloning
    Rc::try_unwrap(data).unwrap_or_else(|rc| (*rc).clone())
}

fn main() {
    // Usage
    let data = Rc::new(vec![1, 2, 3]);
    let owned = make_owned(data);  // No clone: we were sole owner
    println!("Owned data: {:?}", owned);

    // With shared ownership
    let data2 = Rc::new(vec![4, 5, 6]);
    let _shared = Rc::clone(&data2);
    let owned2 = make_owned(data2);  // Clone needed: was shared
    println!("Owned data2: {:?}", owned2);

    println!("try_unwrap example completed");
}
