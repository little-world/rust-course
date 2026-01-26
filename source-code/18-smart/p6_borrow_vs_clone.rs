// Pattern 6: Reference Counting Optimization - Borrow Instead of Clone
use std::rc::Rc;

// Bad: Clones on every call
fn inefficient(data: &Rc<Vec<i32>>) {
    let clone = Rc::clone(data);  // Unnecessary!
    println!("inefficient: {}", clone.len());
}

// Good: Borrow through Deref
fn efficient(data: &Rc<Vec<i32>>) {
    println!("efficient: {}", data.len());  // No clone needed
}

fn main() {
    let data = Rc::new(vec![1, 2, 3, 4, 5]);

    inefficient(&data);
    efficient(&data);

    // Benchmark: 1M iterations
    // inefficient: ~10ms (clone + drop overhead)
    // efficient: ~1ms (just function calls)

    println!("Borrow vs clone example completed");
}
