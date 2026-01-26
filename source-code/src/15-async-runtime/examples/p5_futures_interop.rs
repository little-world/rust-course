//! Pattern 5: Runtime Comparison
//! Using futures crate for compatibility
//!
//! Run with: cargo run --example p5_futures_interop

use futures::executor::block_on;
use futures::future::join;

async fn runtime_independent_function() -> i32 {
    42
}

fn interop_example() {
    // Can run with any executor
    let result = block_on(async {
        let (a, b) = join(
            runtime_independent_function(),
            runtime_independent_function(),
        ).await;
        a + b
    });

    println!("Interop result: {}", result);
}

fn main() {
    interop_example();
}
