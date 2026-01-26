//! Pattern 3: Message Passing with Channels
//! Multiple Producers
//!
//! Run with: cargo run --example p3_multiple_producers

use std::sync::mpsc;
use std::thread;

fn multiple_producers() {
    let (tx, rx) = mpsc::channel();

    for thread_id in 0..3 {
        // Clone the transmitter for each new thread.
        let tx_clone = tx.clone();
        thread::spawn(move || {
            for i in 0..3 {
                let msg = format!("Thread {} msg {}", thread_id, i);
                tx_clone.send(msg).unwrap();
            }
        });
    }

    // Drop the original transmitter so the receiver knows when to stop waiting.
    drop(tx);

    // The receiver will automatically close when all transmitters have been dropped.
    for received in rx {
        println!("Received: {}", received);
    }
}

fn main() {
    println!("=== Multiple Producers ===\n");
    multiple_producers();

    println!("\n=== Key Points ===");
    println!("1. Clone tx to give each producer its own sender");
    println!("2. Drop original tx so receiver knows when all are done");
    println!("3. Messages arrive in non-deterministic order");
}
