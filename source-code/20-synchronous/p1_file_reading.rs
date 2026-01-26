// Pattern 1: Basic File Reading Operations
use std::fs::File;
use std::io::{self, Read};

// Read entire file to string (UTF-8)
fn read_to_string(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
    // Allocates a String big enough for the entire file
    // Returns Err if file doesn't exist, isn't readable, or isn't valid UTF-8
}

// Read entire file to bytes (binary)
fn read_to_bytes(path: &str) -> io::Result<Vec<u8>> {
    std::fs::read(path)
    // Allocates a Vec<u8> and reads all bytes
    // Returns Err if file doesn't exist or isn't readable
}

// Manual reading with buffer control
fn read_with_buffer(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    // read_to_string reads until EOF, automatically resizing the String
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Read exact number of bytes
fn read_exact_bytes(path: &str, n: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; n];

    // read_exact returns Err if fewer than n bytes are available
    // This guarantees you get all n bytes or an error—no partial reads
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn main() -> io::Result<()> {
    // Create a test file first
    let test_file = "test_read.txt";
    std::fs::write(test_file, "Hello, World!\nThis is a test file.\nLine 3.")?;

    // Usage: Load configuration file
    let content = read_to_string(test_file)?;
    println!("=== read_to_string ===");
    println!("{}", content);

    // Usage: Load binary file
    let bytes = read_to_bytes(test_file)?;
    println!("\n=== read_to_bytes ===");
    println!("Read {} bytes", bytes.len());

    // Usage: Read file while keeping handle for metadata
    let data = read_with_buffer(test_file)?;
    println!("\n=== read_with_buffer ===");
    println!("{}", data);

    // Usage: Read fixed-size header
    let header = read_exact_bytes(test_file, 5)?;
    println!("\n=== read_exact_bytes (first 5 bytes) ===");
    println!("{:?} = \"{}\"", header, String::from_utf8_lossy(&header));

    // Cleanup
    std::fs::remove_file(test_file)?;

    println!("\nFile reading examples completed");
    Ok(())
}
