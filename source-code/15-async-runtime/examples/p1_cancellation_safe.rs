//! Pattern 1: Future Composition
//! Cancellation-safe write
//!
//! Run with: cargo run --example p1_cancellation_safe

async fn cancellation_safe_write(data: String) -> Result<(), std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let mut file = File::create("output.txt").await?;
    file.write_all(data.as_bytes()).await?;
    file.sync_all().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    match cancellation_safe_write("Hello, World!".to_string()).await {
        Ok(_) => println!("File written successfully"),
        Err(e) => println!("Write failed: {}", e),
    }
}
