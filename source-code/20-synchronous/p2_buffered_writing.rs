// Pattern 2: Buffered Writing
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};

// Buffered writing (essential for performance)
fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);  // 8 KB buffer by default

    for line in lines {
        writeln!(writer, "{}", line)?;  // Writes to buffer
    }

    writer.flush()?;  // Ensure all buffered data written
    Ok(())
}

// Append to log file (preserves existing)
fn append_log(path: &str, message: &str) -> io::Result<()> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    let mut writer = BufWriter::new(file);
    writeln!(writer, "{}", message)?;
    writer.flush()?;  // Critical for logs
    Ok(())
}

// Generate a large file to demonstrate buffering benefits
fn generate_large_file(path: &str, num_lines: usize) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for i in 0..num_lines {
        writeln!(writer, "Line {}: This is test data for buffered writing demonstration", i + 1)?;
    }

    writer.flush()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let output_file = "test_buffered_output.txt";
    let log_file = "test_buffered_log.txt";
    let large_file = "test_large_output.txt";

    // Usage: Write many lines efficiently
    println!("=== buffered_write ===");
    let lines = vec![
        "First line of output",
        "Second line of output",
        "Third line of output",
        "Fourth line of output",
    ];
    buffered_write(output_file, &lines)?;
    println!("Wrote {} lines to {}", lines.len(), output_file);
    println!("Content:");
    println!("{}", std::fs::read_to_string(output_file)?);

    // Usage: Append log entry with flush
    println!("=== append_log ===");
    append_log(log_file, "Server started")?;
    append_log(log_file, "Processing request 1")?;
    append_log(log_file, "Processing request 2")?;
    append_log(log_file, "Server stopped")?;
    println!("Log content:");
    println!("{}", std::fs::read_to_string(log_file)?);

    // Demonstrate generating a larger file
    println!("=== generate_large_file ===");
    let num_lines = 1000;
    generate_large_file(large_file, num_lines)?;
    let metadata = std::fs::metadata(large_file)?;
    println!("Generated {} lines ({} bytes)", num_lines, metadata.len());

    // Cleanup
    std::fs::remove_file(output_file)?;
    std::fs::remove_file(log_file)?;
    std::fs::remove_file(large_file)?;

    println!("\nBuffered writing examples completed");
    Ok(())
}
