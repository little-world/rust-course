// Pattern 2: Buffered Reading
use std::fs::File;
use std::io::{self, BufRead, BufReader};

// Process large files line by line (memory-efficient)
fn process_large_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);  // 8KB buffer by default

    for (index, line) in reader.lines().enumerate() {
        let line = line?;  // Handle I/O errors per line

        // Process each line
        if line.starts_with('#') {
            continue;  // Skip comments
        }

        println!("Line {}: {}", index + 1, line);
    }

    Ok(())
}

// Filter lines (e.g., only errors)
fn process_errors_only(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines()
        .filter_map(|line| line.ok())  // Skip I/O errors
        .filter(|line| line.contains("ERROR"))
        .collect())
}

fn main() -> io::Result<()> {
    // Create a test log file
    let test_file = "test_log.txt";
    let log_content = r#"# This is a comment
2024-01-01 INFO: Application started
2024-01-01 DEBUG: Loading configuration
2024-01-01 ERROR: Failed to connect to database
2024-01-01 INFO: Retrying connection
2024-01-01 ERROR: Connection timeout
2024-01-01 INFO: Application shutdown
"#;
    std::fs::write(test_file, log_content)?;

    // Usage: Process large log file efficiently
    println!("=== process_large_file (skips comments) ===");
    process_large_file(test_file)?;

    // Usage: Extract all error lines from log
    println!("\n=== process_errors_only ===");
    let errors = process_errors_only(test_file)?;
    println!("Found {} error lines:", errors.len());
    for error in &errors {
        println!("  {}", error);
    }

    // Cleanup
    std::fs::remove_file(test_file)?;

    println!("\nBuffered reading examples completed");
    Ok(())
}
