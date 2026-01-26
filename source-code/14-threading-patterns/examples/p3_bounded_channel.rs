//! Pattern 3: Message Passing with Channels
//! Crossbeam Bounded Channel for Backpressure
//!
//! Run with: cargo run --example p3_bounded_channel

use crossbeam::channel::bounded;
use std::thread;
use std::time::Duration;

fn bounded_channel_backpressure() {
    // A channel with a capacity of 2.
    let (tx, rx) = bounded(2);

    // A fast producer.
    thread::spawn(move || {
        for i in 0..10 {
            println!("Producer: trying to send {}", i);
            tx.send(i).unwrap(); // This will block if the channel is full.
            println!("Producer: sent {}", i);
        }
    });

    // A slow consumer.
    thread::sleep(Duration::from_secs(1));
    for _ in 0..10 {
        let value = rx.recv().unwrap();
        println!("Consumer: received {}", value);
        thread::sleep(Duration::from_millis(500));
    }
}

fn main() {
    println!("=== Crossbeam Bounded Channel for Backpressure ===\n");
    bounded_channel_backpressure();

    println!("\n=== Key Points ===");
    println!("1. Bounded channel has fixed capacity");
    println!("2. Producer blocks when channel is full (backpressure)");
    println!("3. Prevents memory exhaustion from fast producers");
}
