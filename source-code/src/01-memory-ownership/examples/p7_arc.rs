// Pattern 7: Arc for Thread-Safe Sharing
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    let mut handles = vec![];

    for i in 0..3 {
        let data = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            println!("Thread {}: sum = {}", i, data.iter().sum::<i32>());
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Arc example completed");
}
