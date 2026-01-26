// Pattern 5: Timeout Patterns
use std::io;
use tokio::time::{sleep, timeout, timeout_at, Duration, Instant};

// Basic timeout pattern
async fn with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Timeout Demo ===\n");

    // timeout() returns Err if the operation exceeds the duration
    println!("Starting operation with 2-second timeout...");

    let result = timeout(
        Duration::from_secs(2),
        async {
            println!("  Operation started, will take 1 second");
            sleep(Duration::from_secs(1)).await;
            Ok::<_, io::Error>("Success!")
        },
    ).await;

    match result {
        Ok(Ok(value)) => println!("Operation completed: {}", value),
        Ok(Err(e)) => println!("Operation failed: {}", e),
        Err(_) => println!("Operation timed out"),
    }

    // Now demonstrate a timeout
    println!("\nStarting operation with 1-second timeout (will timeout)...");

    let result = timeout(
        Duration::from_secs(1),
        async {
            println!("  Operation started, will take 3 seconds");
            sleep(Duration::from_secs(3)).await;
            Ok::<_, io::Error>("This won't be reached")
        },
    ).await;

    match result {
        Ok(Ok(value)) => println!("Operation completed: {}", value),
        Ok(Err(e)) => println!("Operation failed: {}", e),
        Err(_) => println!("Operation timed out (as expected)"),
    }

    Ok(())
}

// Timeout with fallback
async fn timeout_with_fallback() {
    println!("\n=== Timeout with Fallback Demo ===\n");

    async fn fetch_from_primary() -> io::Result<String> {
        println!("  Fetching from primary (slow)...");
        sleep(Duration::from_secs(5)).await;  // Too slow
        Ok("Primary data".to_string())
    }

    async fn fetch_from_fallback() -> io::Result<String> {
        println!("  Fetching from fallback (fast)...");
        sleep(Duration::from_millis(200)).await;  // Fast fallback
        Ok("Fallback data".to_string())
    }

    let result = timeout(
        Duration::from_secs(2),
        fetch_from_primary(),
    ).await;

    let data = match result {
        Ok(Ok(data)) => data,  // Primary succeeded
        _ => {
            // Primary timed out or failed—try fallback
            println!("  Primary timed out, trying fallback...");
            fetch_from_fallback().await.unwrap_or_default()
        }
    };

    println!("Result: {}", data);
}

// Race three operations—return the first to complete
async fn race_operations() {
    println!("\n=== Race Operations Demo ===\n");

    async fn operation_a() -> String {
        println!("  Operation A started (3 seconds)");
        sleep(Duration::from_secs(3)).await;
        "A completed".to_string()
    }

    async fn operation_b() -> String {
        println!("  Operation B started (1 second)");
        sleep(Duration::from_secs(1)).await;
        "B completed".to_string()
    }

    async fn operation_c() -> String {
        println!("  Operation C started (2 seconds)");
        sleep(Duration::from_secs(2)).await;
        "C completed".to_string()
    }

    let start = Instant::now();

    let winner = tokio::select! {
        result = operation_a() => result,
        result = operation_b() => result,
        result = operation_c() => result,
    };
    // The other futures are dropped (canceled) when one completes

    println!("\nWinner: {} (took {:?})", winner, start.elapsed());
    println!("Other operations were canceled");
}

// Collective timeout: all operations must complete within time limit
async fn collective_timeout() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Collective Timeout Demo ===\n");

    async fn async_task(id: usize, delay_ms: u64) -> String {
        println!("  Task {} starting ({} ms)", id, delay_ms);
        sleep(Duration::from_millis(delay_ms)).await;
        format!("Task {} done", id)
    }

    // All tasks must complete within 2 seconds total
    let result = timeout(
        Duration::from_secs(2),
        async {
            let tasks = vec![
                async_task(1, 500),
                async_task(2, 800),
                async_task(3, 300),
            ];
            futures::future::join_all(tasks).await
        },
    ).await;

    match result {
        Ok(results) => {
            println!("\nAll tasks completed:");
            for r in results {
                println!("  {}", r);
            }
        }
        Err(_) => println!("Collective timeout exceeded"),
    }

    Ok(())
}

// Individual timeouts: each operation has its own timeout
async fn individual_timeouts() {
    println!("\n=== Individual Timeouts Demo ===\n");

    async fn async_task(id: usize, delay_ms: u64) -> String {
        println!("  Task {} starting ({} ms)", id, delay_ms);
        sleep(Duration::from_millis(delay_ms)).await;
        format!("Task {} done", id)
    }

    let operations = vec![
        timeout(Duration::from_millis(400), async_task(1, 200)),  // Will succeed
        timeout(Duration::from_millis(400), async_task(2, 600)),  // Will timeout
        timeout(Duration::from_millis(400), async_task(3, 300)),  // Will succeed
    ];

    let results = futures::future::join_all(operations).await;

    println!("\nResults:");
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(s) => println!("  Task {}: {}", i + 1, s),
            Err(_) => println!("  Task {}: TIMED OUT", i + 1),
        }
    }
}

// Deadline-based timeout
async fn deadline_demo() -> io::Result<()> {
    println!("\n=== Deadline-Based Timeout Demo ===\n");

    // All tasks must complete by this deadline
    let deadline = Instant::now() + Duration::from_secs(3);

    async fn process_item(id: usize, deadline: Instant) -> io::Result<String> {
        let delay = Duration::from_millis(id as u64 * 800);
        println!("  Item {} needs {:?}, deadline in {:?}", id, delay, deadline - Instant::now());

        // timeout_at() uses an absolute deadline instead of a duration
        timeout_at(deadline.into(), async move {
            sleep(delay).await;
            Ok(format!("Item {} processed", id))
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Deadline exceeded"))?
    }

    let tasks = vec![
        process_item(1, deadline),
        process_item(2, deadline),
        process_item(3, deadline),
        process_item(4, deadline),
        process_item(5, deadline),
    ];

    let results = futures::future::join_all(tasks).await;

    println!("\nResults:");
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => println!("  Task {}: {}", i + 1, value),
            Err(e) => println!("  Task {}: {}", i + 1, e),
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Timeout Patterns Demo ===\n");

    with_timeout().await?;
    timeout_with_fallback().await;
    race_operations().await;
    collective_timeout().await?;
    individual_timeouts().await;
    deadline_demo().await?;

    println!("\nTimeout patterns demo completed");
    Ok(())
}
