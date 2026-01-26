// Pattern 1: File Opening Options
use std::fs::OpenOptions;
use std::io::{self, Write};

fn demonstrate_open_options() -> io::Result<()> {
    let test_file = "test_options.txt";

    // First create a file with some content
    std::fs::write(test_file, "Initial content\n")?;

    // Read-only mode (default for File::open)
    // Fails if file doesn't exist
    println!("=== Read-only mode ===");
    let file = OpenOptions::new()
        .read(true)
        .open(test_file)?;
    println!("Opened {} for reading", test_file);
    drop(file);

    // Write-only mode, create if doesn't exist
    // This is what File::create() does internally
    println!("\n=== Write-only mode ===");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(test_file)?;
    file.write_all(b"New content\n")?;
    println!("Wrote new content (truncated old)");
    drop(file);
    println!("Content: {}", std::fs::read_to_string(test_file)?);

    // Append mode (write to end, never truncate)
    // Essential for log files
    println!("=== Append mode ===");
    let mut file = OpenOptions::new()
        .append(true)
        .open(test_file)?;
    file.write_all(b"Appended line\n")?;
    drop(file);
    println!("Content after append:");
    println!("{}", std::fs::read_to_string(test_file)?);

    // Read and write mode (for in-place modification)
    // Allows seeking and both reading and writing
    println!("=== Read and write mode ===");
    let _file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(test_file)?;
    println!("Opened for both reading and writing");

    // Create new file, fail if it already exists
    // Use this to avoid overwriting important files
    println!("\n=== Create new (exclusive) ===");
    let unique_file = "unique_test.txt";
    // Remove if exists from previous run
    let _ = std::fs::remove_file(unique_file);

    let file = OpenOptions::new()
        .write(true)
        .create_new(true)   // Fail if exists (atomic check-and-create)
        .open(unique_file)?;
    println!("Created new file: {}", unique_file);
    drop(file);

    // Try to create again - should fail
    let result = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(unique_file);
    match result {
        Ok(_) => println!("Unexpected: file was created again"),
        Err(e) => println!("Expected error (file exists): {}", e.kind()),
    }

    // Custom permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        println!("\n=== Custom permissions (Unix) ===");
        let secure_file = "secure_test.txt";
        let _file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)      // rw------- (owner only)
            .open(secure_file)?;
        println!("Created {} with mode 0600", secure_file);
        std::fs::remove_file(secure_file)?;
    }

    // Cleanup
    std::fs::remove_file(test_file)?;
    std::fs::remove_file(unique_file)?;

    println!("\nOpenOptions examples completed");
    Ok(())
}

fn main() -> io::Result<()> {
    demonstrate_open_options()
}
