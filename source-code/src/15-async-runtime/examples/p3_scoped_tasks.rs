//! Pattern 3: Async/Await Patterns
//! Scoped tasks with JoinSet (guaranteed completion)
//!
//! Run with: cargo run --example p3_scoped_tasks

use tokio::task::JoinSet;

async fn scoped_tasks_with_joinset() {
    let data = vec![1, 2, 3, 4, 5];
    let mut set = JoinSet::new();

    for item in data {
        set.spawn(async move {
            // Process item
            item * 2
        });
    }

    // Wait for all tasks - guaranteed to complete before we continue
    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }

    println!("Results: {:?}", results);
}

#[tokio::main]
async fn main() {
    scoped_tasks_with_joinset().await;
}
