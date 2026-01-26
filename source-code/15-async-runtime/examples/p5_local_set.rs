//! Pattern 5: Runtime Comparison
//! Local task set (for !Send futures)
//!
//! Run with: cargo run --example p5_local_set

use tokio::task::LocalSet;

async fn local_task_set_example() {
    use std::rc::Rc;

    let local = LocalSet::new();

    let nonsend_data = Rc::new(42);

    local.run_until(async move {
        let data = Rc::clone(&nonsend_data);

        tokio::task::spawn_local(async move {
            println!("Local task with Rc: {}", data);
        }).await.unwrap();
    }).await;
}

#[tokio::main]
async fn main() {
    local_task_set_example().await;
}
