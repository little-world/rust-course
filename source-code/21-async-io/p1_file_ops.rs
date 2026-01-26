// Pattern 1: Tokio File Operations
use std::io;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Read entire file into a String
async fn read_file(path: &str) -> io::Result<String> {
    tokio::fs::read_to_string(path).await
    // Convenience method: allocates a String, reads entire file
    // Returns Err if file missing, unreadable, or contains invalid UTF-8
}

// Read entire file into Vec<u8>
async fn read_bytes(path: &str) -> io::Result<Vec<u8>> {
    tokio::fs::read(path).await
    // Reads entire file into memory
    // More efficient than read_to_string for binary data
}

// Write string to file (overwrites existing content)
async fn write_file(path: &str, content: &str) -> io::Result<()> {
    tokio::fs::write(path, content).await
    // Convenience method: creates file, writes content, closes file
    // Overwrites existing file! Use append_to_file if you want to append
}

// Manual read with buffer control
async fn read_with_buffer(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path).await?;
    let mut buffer = Vec::new();

    // read_to_end() allocates as needed while reading
    file.read_to_end(&mut buffer).await?;
    Ok(buffer)
}

// Manual write with explicit handle
async fn write_with_handle(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path).await?;

    // write_all() ensures all bytes are written (loops if needed)
    file.write_all(data).await?;

    // flush() ensures buffered data reaches the OS
    // (Tokio buffers writes internally for efficiency)
    file.flush().await?;
    Ok(())
}

// Append to existing file (or create if missing)
async fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)   // Append mode: writes go to end of file
        .create(true)   // Create file if it doesn't exist
        .open(path)
        .await?;

    file.write_all(content.as_bytes()).await?;
    file.write_all(b"\n").await?;  // Add newline separator
    Ok(())
}

// Copy file asynchronously
async fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    tokio::fs::copy(src, dst).await
    // Efficiently copies src to dst using OS-level optimizations when possible
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== Tokio File Operations Demo ===\n");

    // Create a test file
    let test_file = "test_async.txt";
    let test_content = "Hello from async Rust!\nThis is line 2.\nLine 3 here.";

    // Write file
    println!("=== write_file ===");
    write_file(test_file, test_content).await?;
    println!("Wrote file: {}", test_file);

    // Read file as string
    println!("\n=== read_file ===");
    let content = read_file(test_file).await?;
    println!("Content:\n{}", content);

    // Read file as bytes
    println!("\n=== read_bytes ===");
    let bytes = read_bytes(test_file).await?;
    println!("Read {} bytes", bytes.len());

    // Read with buffer
    println!("\n=== read_with_buffer ===");
    let buffer = read_with_buffer(test_file).await?;
    println!("Buffer size: {} bytes", buffer.len());

    // Write with explicit handle
    println!("\n=== write_with_handle ===");
    let binary_file = "test_binary.bin";
    write_with_handle(binary_file, &[0x48, 0x65, 0x6c, 0x6c, 0x6f]).await?;
    println!("Wrote binary file: {}", binary_file);

    // Append to file
    println!("\n=== append_to_file ===");
    let log_file = "test_log.txt";
    append_to_file(log_file, "First log entry").await?;
    append_to_file(log_file, "Second log entry").await?;
    append_to_file(log_file, "Third log entry").await?;
    let log_content = read_file(log_file).await?;
    println!("Log file content:\n{}", log_content);

    // Copy file
    println!("\n=== copy_file ===");
    let copy_dest = "test_copy.txt";
    let bytes_copied = copy_file(test_file, copy_dest).await?;
    println!("Copied {} bytes to {}", bytes_copied, copy_dest);

    // Cleanup
    tokio::fs::remove_file(test_file).await?;
    tokio::fs::remove_file(binary_file).await?;
    tokio::fs::remove_file(log_file).await?;
    tokio::fs::remove_file(copy_dest).await?;

    println!("\nFile operations completed successfully");
    Ok(())
}
