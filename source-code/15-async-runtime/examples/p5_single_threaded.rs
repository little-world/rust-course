//! Pattern 5: Runtime Comparison
//! Single-threaded runtime
//!
//! Run with: cargo run --example p5_single_threaded

#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("Running on single-threaded runtime");

    let thread_id = std::thread::current().id();

    for i in 0..5 {
        tokio::spawn(async move {
            println!("Task {} on thread {:?}", i, std::thread::current().id());
        }).await.unwrap();
    }

    println!("All tasks ran on thread {:?}", thread_id);
}
