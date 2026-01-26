//! Pattern 2: Stream Processing
//! Database query result stream
//!
//! Run with: cargo run --example p2_db_stream

use std::time::Duration;
use futures::Stream;
use tokio_stream::StreamExt;

#[derive(Debug)]
struct Row { id: u64, data: String }

async fn database_query_stream(_query: String) -> impl Stream<Item = Row> {
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(async move {
        for i in 0..10 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            if tx.send(Row { id: i, data: format!("Data {}", i) }).await.is_err() { break; }
        }
    });
    tokio_stream::wrappers::ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() {
    let mut stream = database_query_stream("SELECT * FROM users".to_string()).await;
    while let Some(row) = stream.next().await {
        println!("Row: {:?}", row);
    }
}
