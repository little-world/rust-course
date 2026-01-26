//! Pattern 4: Select and Timeout Patterns
//! Select in a loop
//!
//! Run with: cargo run --example p4_select_loop

use std::time::Duration;
use tokio::sync::mpsc;

async fn select_loop() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(10);
    let (tx2, mut rx2) = mpsc::channel::<String>(10);

    // Spawn producers
    tokio::spawn(async move {
        for i in 0..5 {
            tokio::time::sleep(Duration::from_millis(100)).await;
            tx1.send(i).await.unwrap();
        }
    });

    tokio::spawn(async move {
        for i in 0..3 {
            tokio::time::sleep(Duration::from_millis(150)).await;
            tx2.send(format!("msg_{}", i)).await.unwrap();
        }
    });

    let mut done1 = false;
    let mut done2 = false;

    loop {
        tokio::select! {
            result = rx1.recv(), if !done1 => {
                match result {
                    Some(num) => println!("Number: {}", num),
                    None => done1 = true,
                }
            }
            result = rx2.recv(), if !done2 => {
                match result {
                    Some(msg) => println!("Message: {}", msg),
                    None => done2 = true,
                }
            }
            else => {
                println!("Both channels closed");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    select_loop().await;
}
