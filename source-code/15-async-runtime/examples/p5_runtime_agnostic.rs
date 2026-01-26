//! Pattern 5: Runtime Comparison
//! Runtime-agnostic code using futures
//!
//! Run with: cargo run --example p5_runtime_agnostic

mod runtime_agnostic {
    use futures::future::join_all;
    use std::future::Future;

    pub async fn process_items<F, Fut>(
        items: Vec<i32>,
        process: F,
    ) -> Vec<i32>
    where
        F: Fn(i32) -> Fut,
        Fut: Future<Output = i32>,
    {
        let futures: Vec<_> = items.into_iter().map(process).collect();
        join_all(futures).await
    }
}

#[tokio::main]
async fn main() {
    let results = runtime_agnostic::process_items(
        vec![1, 2, 3, 4, 5],
        |x| async move { x * 2 },
    ).await;
    println!("Results: {:?}", results);
}
