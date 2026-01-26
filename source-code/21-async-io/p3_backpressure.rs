// Pattern 3: Backpressure Handling
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

// Producer with backpressure
async fn producer_with_backpressure(tx: mpsc::Sender<i32>, count: i32) {
    for i in 0..count {
        // send() provides backpressure—blocks when channel is full
        // This ensures we don't overwhelm the consumer
        if tx.send(i).await.is_err() {
            println!("Producer: Receiver dropped, stopping");
            break;
        }
        println!("Producer: Sent {}", i);
    }
    println!("Producer: Finished");
}

// Consumer (intentionally slow to demonstrate backpressure)
async fn consumer(mut rx: mpsc::Receiver<i32>, delay_ms: u64) {
    while let Some(value) = rx.recv().await {
        println!("Consumer: Processing {}", value);
        // Simulate slow processing
        sleep(Duration::from_millis(delay_ms)).await;
    }
    println!("Consumer: Channel closed");
}

// Demonstrate backpressure with bounded channel
async fn backpressure_demo() {
    println!("=== Backpressure Demo ===");
    println!("Producer sends fast, consumer processes slowly");
    println!("Channel capacity: 5, so producer will wait when full\n");

    // Bounded channel with capacity 5
    // Producer can get ahead by 5 items, then must wait
    let (tx, rx) = mpsc::channel(5);

    let start = Instant::now();

    let producer = tokio::spawn(async move {
        producer_with_backpressure(tx, 15).await;
    });

    let consumer = tokio::spawn(async move {
        consumer(rx, 100).await;  // 100ms per item
    });

    let _ = tokio::join!(producer, consumer);

    println!("\nTotal time: {:?}", start.elapsed());
    println!("Without backpressure, producer would finish instantly");
    println!("With backpressure, producer waits for consumer\n");
}

// Demonstrate unbounded vs bounded channel behavior
async fn compare_bounded_unbounded() {
    println!("=== Bounded vs Unbounded Comparison ===\n");

    // Unbounded channel (dangerous - can cause OOM)
    println!("Unbounded channel demo (limited to 100 items for safety):");
    let (tx_unbounded, mut rx_unbounded) = mpsc::unbounded_channel();

    let start = Instant::now();
    for i in 0..100 {
        let _ = tx_unbounded.send(i);
    }
    let unbounded_time = start.elapsed();
    drop(tx_unbounded);

    // Drain the channel
    while rx_unbounded.recv().await.is_some() {}

    println!("Unbounded: Sent 100 items in {:?} (no waiting)", unbounded_time);

    // Bounded channel with same items
    println!("\nBounded channel demo (capacity 10):");
    let (tx_bounded, mut rx_bounded) = mpsc::channel(10);

    let start = Instant::now();

    let sender = tokio::spawn(async move {
        for i in 0..100 {
            let _ = tx_bounded.send(i).await;
        }
    });

    let receiver = tokio::spawn(async move {
        while rx_bounded.recv().await.is_some() {
            // Fast receiver
        }
    });

    let _ = tokio::join!(sender, receiver);
    let bounded_time = start.elapsed();

    println!("Bounded: Sent 100 items in {:?}", bounded_time);
    println!("\nNote: Bounded channels provide flow control but add coordination overhead");
}

// Stream backpressure with buffer_unordered
async fn stream_backpressure_demo() {
    use futures::stream::{self, StreamExt};

    println!("\n=== Stream Backpressure with buffer_unordered ===");
    println!("Processing 10 items with max 3 concurrent\n");

    let start = Instant::now();

    stream::iter(0..10)
        .map(|i| async move {
            println!("  Starting task {}", i);
            sleep(Duration::from_millis(200)).await;
            println!("  Completed task {}", i);
            i
        })
        // buffer_unordered(3) means at most 3 futures run concurrently
        // This provides backpressure: we won't start future #4 until one completes
        .buffer_unordered(3)
        .for_each(|_| async {})
        .await;

    println!("\nTotal time: {:?}", start.elapsed());
    println!("With buffer_unordered(3), tasks complete in batches");
}

#[tokio::main]
async fn main() {
    println!("=== Backpressure Handling Demo ===\n");

    backpressure_demo().await;
    compare_bounded_unbounded().await;
    stream_backpressure_demo().await;

    println!("\nBackpressure demo completed");
}
