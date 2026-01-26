//! Pattern 3: Message Passing with Channels
//! Basic Producer-Consumer with MPSC Channel
//!
//! Run with: cargo run --example p3_mpsc_channel

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn basic_mpsc_channel() {
    let (tx, rx) = mpsc::channel(); // tx = transmitter, rx = receiver

    // Spawn a producer thread.
    thread::spawn(move || {
        for i in 0..5 {
            println!("Sending: {}", i);
            tx.send(i).unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    // The receiver can be used as an iterator that blocks until a message is received.
    for received in rx {
        println!("Received: {}", received);
    }
}

fn main() {
    println!("=== Basic Producer-Consumer with MPSC Channel ===\n");
    basic_mpsc_channel();

    println!("\n=== Key Points ===");
    println!("1. mpsc = Multiple Producer, Single Consumer");
    println!("2. Channel is unbounded by default");
    println!("3. Receiver blocks until message arrives or channel closes");
    println!("4. Channel closes when all senders are dropped");
}
