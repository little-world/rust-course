// Pattern 2: Buffered Async Streams
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, ReadBuf};

// Buffered reading with custom buffer size
async fn buffered_read(path: &str) -> io::Result<()> {
    let file = File::open(path).await?;

    // Create BufReader with 8KB buffer (adjust based on your workload)
    let reader = BufReader::with_capacity(8192, file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        println!("{}", line);
        // Processing each line is fast because BufReader minimizes system calls
    }

    Ok(())
}

// Buffered writing with custom buffer size
async fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path).await?;

    // BufWriter accumulates writes, flushes when buffer is full
    let mut writer = BufWriter::with_capacity(8192, file);

    for line in lines {
        // These writes don't immediately hit disk—they go to the buffer
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    // flush() ensures all buffered data is written to disk
    // Without this, some data might remain in the buffer!
    writer.flush().await?;
    Ok(())
}

// Copy with buffering
async fn buffered_copy(src: &str, dst: &str) -> io::Result<u64> {
    let src_file = File::open(src).await?;
    let dst_file = File::create(dst).await?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    // copy() efficiently transfers data, using the buffers to minimize system calls
    let bytes = tokio::io::copy(&mut reader, &mut writer).await?;

    // Ensure all buffered data is written
    writer.flush().await?;

    Ok(bytes)
}

// Custom async reader that uppercases data
struct UppercaseReader<R> {
    inner: R,
}

impl<R: AsyncRead + Unpin> AsyncRead for UppercaseReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Track how many bytes were in the buffer before reading
        let before_len = buf.filled().len();

        // Delegate to the inner reader
        match Pin::new(&mut self.inner).poll_read(cx, buf) {
            Poll::Ready(Ok(())) => {
                // Uppercase the newly read bytes
                let filled = buf.filled_mut();
                for byte in &mut filled[before_len..] {
                    if byte.is_ascii_lowercase() {
                        *byte = byte.to_ascii_uppercase();
                    }
                }
                Poll::Ready(Ok(()))
            }
            other => other,
        }
    }
}

// Usage example for UppercaseReader
async fn use_uppercase_reader(path: &str) -> io::Result<String> {
    let file = File::open(path).await?;
    let mut reader = UppercaseReader { inner: file };

    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).await?;

    Ok(buffer)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("=== Buffered Async Streams Demo ===\n");

    // Create test files
    let source_file = "test_buffered_src.txt";
    let dest_file = "test_buffered_dst.txt";
    let output_file = "test_buffered_out.txt";

    let test_content = "Hello World!\nThis is a test file.\nWith multiple lines.\nFor buffered I/O demo.";
    tokio::fs::write(source_file, test_content).await?;

    // Buffered reading
    println!("=== buffered_read ===");
    buffered_read(source_file).await?;

    // Buffered writing
    println!("\n=== buffered_write ===");
    let lines = vec!["Line 1: Written with buffering", "Line 2: More efficient", "Line 3: Fewer syscalls"];
    buffered_write(output_file, &lines).await?;
    println!("Wrote {} lines to {}", lines.len(), output_file);

    // Verify written content
    let written = tokio::fs::read_to_string(output_file).await?;
    println!("Written content:\n{}", written);

    // Buffered copy
    println!("=== buffered_copy ===");
    let bytes_copied = buffered_copy(source_file, dest_file).await?;
    println!("Copied {} bytes from {} to {}", bytes_copied, source_file, dest_file);

    // Custom uppercase reader
    println!("\n=== UppercaseReader ===");
    let uppercased = use_uppercase_reader(source_file).await?;
    println!("Uppercased content:\n{}", uppercased);

    // Cleanup
    tokio::fs::remove_file(source_file).await?;
    tokio::fs::remove_file(dest_file).await?;
    tokio::fs::remove_file(output_file).await?;

    println!("\nBuffered I/O demo completed");
    Ok(())
}
