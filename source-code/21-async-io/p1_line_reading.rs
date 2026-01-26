// Pattern 1: Async Line Reading
use std::io;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

// Read all lines into memory
async fn read_lines(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    let mut line_stream = reader.lines();
    // lines() returns a stream that yields one line at a time
    // Lines are split on \n or \r\n, with the newline removed

    while let Some(line) = line_stream.next_line().await? {
        lines.push(line);
    }

    Ok(lines)
}

// Process large file line by line without loading into memory
async fn process_large_file(path: &str) -> io::Result<usize> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut count = 0;
    while let Some(line) = lines.next_line().await? {
        if !line.starts_with('#') {
            // Process non-comment lines
            // Each line is processed and dropped before reading the next
            count += 1;
        }
    }

    Ok(count)
}

// Read first N lines (useful for previews or headers)
async fn read_first_n_lines(path: &str, n: usize) -> io::Result<Vec<String>> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut result = Vec::new();

    for _ in 0..n {
        if let Some(line) = lines.next_line().await? {
            result.push(line);
        } else {
            break;  // File has fewer than n lines
        }
    }

    Ok(result)
}

// Count lines matching a pattern
async fn count_matching_lines(path: &str, pattern: &str) -> io::Result<usize> {
    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut count = 0;

    while let Some(line) = lines.next_line().await? {
        if line.contains(pattern) {
            count += 1;
        }
    }

    Ok(count)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== Async Line Reading Demo ===\n");

    // Create a test file with multiple lines
    let test_file = "test_lines.txt";
    let content = "# This is a comment
Line 1: Hello World
Line 2: Rust is awesome
# Another comment
Line 3: Async I/O rocks
Line 4: Error handling
Line 5: Final line";

    tokio::fs::write(test_file, content).await?;

    // Read all lines
    println!("=== read_lines ===");
    let all_lines = read_lines(test_file).await?;
    println!("Total lines: {}", all_lines.len());
    for (i, line) in all_lines.iter().enumerate() {
        println!("  {}: {}", i + 1, line);
    }

    // Process file (skip comments)
    println!("\n=== process_large_file (skip comments) ===");
    let non_comment_count = process_large_file(test_file).await?;
    println!("Non-comment lines: {}", non_comment_count);

    // Read first N lines
    println!("\n=== read_first_n_lines (first 3) ===");
    let first_3 = read_first_n_lines(test_file, 3).await?;
    for line in &first_3 {
        println!("  {}", line);
    }

    // Count matching lines
    println!("\n=== count_matching_lines (pattern: 'Line') ===");
    let matching = count_matching_lines(test_file, "Line").await?;
    println!("Lines containing 'Line': {}", matching);

    // Cleanup
    tokio::fs::remove_file(test_file).await?;

    println!("\nLine reading examples completed");
    Ok(())
}
