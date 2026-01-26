
### Tokio File IO Cheat Sheet

```rust
// ===== TOKIO FILE I/O =====
use tokio::fs;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, AsyncBufReadExt, AsyncSeekExt, BufReader, BufWriter};

// File opening (async)
fs::File::open("file.txt").await?                    // Open for reading
fs::File::create("file.txt").await?                  // Create/truncate for writing
fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .open("file.txt")
    .await?                                          // Custom open options

// Quick read operations (async)
fs::read("file.txt").await?                         // Read entire file to Vec<u8>
fs::read_to_string("file.txt").await?               // Read entire file to String

// Quick write operations (async)
fs::write("file.txt", b"data").await?               // Write bytes to file
fs::write("file.txt", "text").await?                // Write string to file

// Reading from File (async)
let mut file = fs::File::open("file.txt").await?;
let mut buffer = Vec::new();
file.read_to_end(&mut buffer).await?                // Read all bytes
let mut buffer = String::new();
file.read_to_string(&mut buffer).await?             // Read as string
let mut buffer = [0u8; 1024];
let n = file.read(&mut buffer).await?               // Read up to buffer size
file.read_exact(&mut buffer).await?                 // Read exact amount or error

// Writing to File (async)
let mut file = fs::File::create("file.txt").await?;
file.write(b"data").await?                          // Write bytes, returns bytes written
file.write_all(b"data").await?                      // Write all bytes or error
file.flush().await?                                  // Flush to disk

// Buffered reading (async)
let file = fs::File::open("file.txt").await?;
let reader = BufReader::new(file);
let mut reader = BufReader::with_capacity(size, file); // Custom buffer size

let mut lines = reader.lines();                      // Get lines stream
while let Some(line) = lines.next_line().await? {
    println!("{}", line);
}

reader.read_line(&mut string).await?                // Read one line
reader.read_until(b'\n', &mut buffer).await?        // Read until delimiter

// Buffered writing (async)
let file = fs::File::create("file.txt").await?;
let mut writer = BufWriter::new(file);
let mut writer = BufWriter::with_capacity(size, file); // Custom buffer size

writer.write_all(b"data").await?                    // Buffered write
writer.flush().await?                                // Flush buffer to disk

// Seeking (async)
file.seek(io::SeekFrom::Start(0)).await?            // Seek to byte position from start
file.seek(io::SeekFrom::End(-10)).await?            // Seek from end
file.seek(io::SeekFrom::Current(5)).await?          // Seek relative to current
file.rewind().await?                                 // Seek to start
let pos = file.stream_position().await?             // Get current position

// Metadata operations (async)
let metadata = fs::metadata("file.txt").await?      // Get file metadata
let metadata = file.metadata().await?               // From file handle
metadata.len()                                       // File size in bytes
metadata.is_file()                                   // Check if regular file
metadata.is_dir()                                    // Check if directory
metadata.is_symlink()                               // Check if symbolic link
metadata.modified()?                                 // Last modified time

// File operations (async)
fs::copy("examples.txt", "dst.txt").await?               // Copy file, returns bytes copied
fs::rename("old.txt", "new.txt").await?             // Rename/move file
fs::remove_file("file.txt").await?                  // Delete file
fs::hard_link("original.txt", "link.txt").await?   // Create hard link
fs::symlink("original.txt", "link.txt").await?     // Create symbolic link
fs::read_link("link.txt").await?                    // Read symlink target
fs::canonicalize("./file.txt").await?               // Get absolute path

// Directory operations (async)
fs::create_dir("dir").await?                        // Create single directory
fs::create_dir_all("path/to/dir").await?           // Create directory and parents
fs::remove_dir("dir").await?                        // Remove empty directory
fs::remove_dir_all("dir").await?                    // Remove directory and contents

let mut entries = fs::read_dir("dir").await?;       // Get directory entries stream
while let Some(entry) = entries.next_entry().await? {
    println!("{:?}", entry.path());                 // Get full path
    println!("{:?}", entry.file_name());            // Get file name
    let metadata = entry.metadata().await?;          // Get metadata
    let file_type = entry.file_type().await?;       // Get file type
}

// File synchronization (async)
file.sync_all().await?                               // Sync data and metadata to disk
file.sync_data().await?                              // Sync only data to disk

// Set file length (async)
file.set_len(100).await?                            // Set file size (truncate/extend)

// Permissions (async)
let perms = metadata.permissions();
fs::set_permissions("file.txt", perms).await?       // Apply permissions

// Common file patterns (async)
// Read file line by line
let file = fs::File::open("file.txt").await?;
let reader = BufReader::new(file);
let mut lines = reader.lines();
while let Some(line) = lines.next_line().await? {
    // process line
}

// Write multiple lines
let file = fs::File::create("file.txt").await?;
let mut writer = BufWriter::new(file);
for item in items {
    writer.write_all(format!("{}\n", item).as_bytes()).await?;
}
writer.flush().await?;

// Copy with progress
let mut src = fs::File::open("examples.txt").await?;
let mut dst = fs::File::create("dst.txt").await?;
let mut buffer = [0u8; 8192];
loop {
    let n = src.read(&mut buffer).await?;
    if n == 0 { break; }
    dst.write_all(&buffer[..n]).await?;
}


```