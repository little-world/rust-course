// Pattern 5: Cancellation with CancellationToken
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, Instant};
use tokio_util::sync::CancellationToken;

// Cancellable operation using CancellationToken
async fn cancellable_operation(token: CancellationToken, id: usize) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    let mut count = 0;

    loop {
        tokio::select! {
            // Check if cancellation was requested
            _ = token.cancelled() => {
                println!("  Worker {}: Received cancellation signal, stopping", id);
                break;
            }
            // Do work
            _ = interval.tick() => {
                count += 1;
                println!("  Worker {}: Working... (iteration {})", id, count);
            }
        }
    }

    println!("  Worker {}: Cleanup complete", id);
}

// Demonstrate CancellationToken usage
async fn cancellation_token_demo() {
    println!("=== CancellationToken Demo ===\n");

    let token = CancellationToken::new();

    // Spawn multiple workers with the same token
    let mut handles = vec![];
    for i in 0..3 {
        let worker_token = token.clone();
        let handle = tokio::spawn(async move {
            cancellable_operation(worker_token, i).await;
        });
        handles.push(handle);
    }

    // Let workers run for 2 seconds
    println!("Workers running for 2 seconds...\n");
    sleep(Duration::from_secs(2)).await;

    // Cancel all workers
    println!("\nCancelling all workers...");
    token.cancel();

    // Wait for all workers to finish
    for handle in handles {
        handle.await.unwrap();
    }

    println!("\nAll workers stopped");
}

// Child tokens for hierarchical cancellation
async fn hierarchical_cancellation_demo() {
    println!("\n=== Hierarchical Cancellation Demo ===\n");

    let parent_token = CancellationToken::new();

    // Create child tokens
    let child1 = parent_token.child_token();
    let child2 = parent_token.child_token();

    let handle1 = tokio::spawn({
        let token = child1.clone();
        async move {
            loop {
                tokio::select! {
                    _ = token.cancelled() => {
                        println!("  Child 1: Cancelled");
                        break;
                    }
                    _ = sleep(Duration::from_millis(300)) => {
                        println!("  Child 1: Working...");
                    }
                }
            }
        }
    });

    let handle2 = tokio::spawn({
        let token = child2.clone();
        async move {
            loop {
                tokio::select! {
                    _ = token.cancelled() => {
                        println!("  Child 2: Cancelled");
                        break;
                    }
                    _ = sleep(Duration::from_millis(400)) => {
                        println!("  Child 2: Working...");
                    }
                }
            }
        }
    });

    sleep(Duration::from_secs(1)).await;

    // Cancel only child1
    println!("\nCancelling child 1 only...");
    child1.cancel();
    sleep(Duration::from_millis(500)).await;

    // Cancel parent (which also cancels child2)
    println!("\nCancelling parent (affects child 2)...");
    parent_token.cancel();

    handle1.await.unwrap();
    handle2.await.unwrap();

    println!("\nHierarchical cancellation complete");
}

// Graceful shutdown with broadcast channel
async fn graceful_shutdown_demo() {
    println!("\n=== Graceful Shutdown Demo ===\n");

    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    let start = Instant::now();

    // Spawn multiple worker tasks
    let mut handles = vec![];
    for i in 0..3 {
        let mut shutdown_rx = shutdown_tx.subscribe();
        let handle = tokio::spawn(async move {
            let mut count = 0;
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        println!("  Worker {}: Shutdown signal received at {:?}", i, start.elapsed());
                        // Perform cleanup
                        sleep(Duration::from_millis(100)).await;
                        println!("  Worker {}: Cleanup complete", i);
                        break;
                    }
                    _ = sleep(Duration::from_millis(200)) => {
                        count += 1;
                        println!("  Worker {}: Processing item {} at {:?}", i, count, start.elapsed());
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Let workers run
    sleep(Duration::from_secs(1)).await;

    // Send shutdown signal
    println!("\nSending shutdown signal...");
    let _ = shutdown_tx.send(());

    // Wait for all workers to finish cleanup
    for handle in handles {
        handle.await.unwrap();
    }

    println!("\nAll workers shut down gracefully");
}

// Biased select for priority handling
async fn biased_select_demo() {
    println!("\n=== Biased Select Demo ===\n");

    let token = CancellationToken::new();
    let token_clone = token.clone();

    // Spawn a task that will cancel after 1 second
    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        println!("Requesting cancellation...");
        token_clone.cancel();
    });

    let mut count = 0;
    let start = Instant::now();

    loop {
        tokio::select! {
            biased;  // Check branches in order (not randomly)

            // Cancellation has highest priority
            _ = token.cancelled() => {
                println!("\nCancellation takes priority!");
                break;
            }

            // Normal work
            _ = sleep(Duration::from_millis(100)) => {
                count += 1;
                println!("Working... (iteration {} at {:?})", count, start.elapsed());
            }
        }
    }

    println!("Completed {} iterations before cancellation", count);
}

#[tokio::main]
async fn main() {
    println!("=== Cancellation Patterns Demo ===\n");

    cancellation_token_demo().await;
    hierarchical_cancellation_demo().await;
    graceful_shutdown_demo().await;
    biased_select_demo().await;

    println!("\nCancellation patterns demo completed");
}
