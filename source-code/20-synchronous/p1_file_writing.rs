// Pattern 1: Basic File Writing Operations
use std::fs::{File, OpenOptions};
use std::io::{self, Write};

// Write string to file (overwrites existing)
fn write_string(path: &str, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
    // Creates file if it doesn't exist
    // Truncates (erases) existing content
    // Writes all content in one operation
}

// Write bytes to file
fn write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    std::fs::write(path, content)
}

// Manual writing with file handle
fn write_with_handle(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    // write_all ensures all bytes are written or returns Err
    // Partial writes are retried automatically
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Append to file (preserves existing content)
fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)    // Open in append mode
        .create(true)    // Create if doesn't exist
        .open(path)?;

    writeln!(file, "{}", content)?;  // Adds newline automatically
    Ok(())
}

fn main() -> io::Result<()> {
    let test_file = "test_write.txt";
    let binary_file = "test_binary.bin";
    let append_file = "test_append.txt";

    // Usage: Save configuration
    println!("=== write_string ===");
    write_string(test_file, "Hello, world!")?;
    println!("Wrote to {}", test_file);
    println!("Content: {}", std::fs::read_to_string(test_file)?);

    // Usage: Write PNG magic bytes
    println!("\n=== write_bytes ===");
    write_bytes(binary_file, &[0x89, 0x50, 0x4E, 0x47])?;
    println!("Wrote PNG magic bytes to {}", binary_file);
    let bytes = std::fs::read(binary_file)?;
    println!("Content: {:?}", bytes);

    // Usage: Write with explicit file handle
    println!("\n=== write_with_handle ===");
    write_with_handle(test_file, "Manual write data")?;
    println!("Content: {}", std::fs::read_to_string(test_file)?);

    // Usage: Append log entry
    println!("\n=== append_to_file ===");
    append_to_file(append_file, "First entry")?;
    append_to_file(append_file, "Second entry")?;
    append_to_file(append_file, "Third entry")?;
    println!("Content after appending:");
    println!("{}", std::fs::read_to_string(append_file)?);

    // Cleanup
    std::fs::remove_file(test_file)?;
    std::fs::remove_file(binary_file)?;
    std::fs::remove_file(append_file)?;

    println!("File writing examples completed");
    Ok(())
}
