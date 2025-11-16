# Chapter 16: Synchronous I/O

## Overview

Input/Output is the bridge between your program and the outside world. Files, network sockets, user input, process communication—all are I/O operations that move data across boundaries. In Rust, I/O is synchronous by default: operations block the current thread until they complete. This contrasts with asynchronous I/O (covered in Chapter 17), where operations can pause and resume without blocking threads.

Synchronous I/O is the foundation. It's simpler to reason about, easier to debug, and sufficient for most programs. A command-line tool reading a configuration file doesn't need async complexity. A build script copying files benefits from straightforward, blocking operations. Even high-performance servers often use synchronous I/O in worker threads, relying on thread pools for concurrency rather than async runtimes.

This chapter explores Rust's synchronous I/O ecosystem through practical patterns:

**File operations** cover reading, writing, seeking, and metadata—the bread and butter of persistent storage. Understanding buffering here is crucial: the difference between reading a gigabyte file all at once versus processing it in chunks can mean the difference between instant success and out-of-memory crashes.

**Standard streams** (stdin, stdout, stderr) connect your program to the terminal and shell pipelines. These aren't just for simple programs—even sophisticated applications need robust terminal I/O for configuration, logging, and debugging.

**Memory-mapped I/O** treats files as if they were memory, enabling efficient random access and zero-copy operations. This pattern powers databases, memory-mapped data structures, and high-performance log parsing.

**Directory traversal** walks file trees, filters by patterns, and computes statistics. Every build system, backup tool, and file search utility relies on efficient directory walking.

**Process spawning** launches external commands, pipes data between processes, and manages child process lifecycles. This is how you integrate with the broader system, calling compilers, running tests, or orchestrating multi-process workflows.

**Key principles** that guide Rust's I/O design:
- **Explicit error handling**: I/O operations return `Result<T, io::Error>`, forcing you to handle failures explicitly. A missing file, a full disk, or network timeout—all become errors you must address
- **Zero-cost abstractions**: Buffering is opt-in; you choose when to trade memory for performance
- **Ownership and lifetimes**: File handles close automatically when dropped; you can't accidentally leak resources
- **Cross-platform**: Rust's I/O abstractions work across operating systems, with platform-specific extensions when needed

The patterns you'll learn aren't just about moving bytes—they're about building reliable systems that handle errors gracefully, perform efficiently, and compose well with other Rust code.

---

## File Operations and Buffering

Files are the primary form of persistent storage for most programs. Configuration files, data logs, cached results, user documents—all live in the filesystem. Rust's file I/O API balances convenience with control, offering simple helpers for common cases and low-level access for performance-critical code.

**The fundamental trade-off**: convenience versus control. Do you read the entire file into memory (`fs::read_to_string`) for simplicity, or process it line by line (`BufReader::lines`) to handle files larger than RAM? The right choice depends on your constraints.

### Basic File Reading

Reading files in Rust starts with three questions:
1. **How big is it?** (Whole file vs. streaming)
2. **What format?** (Text vs. binary)
3. **What if it fails?** (Error handling strategy)

The simplest case: read a small file entirely into memory.

```rust
use std::fs::File;
use std::io::{self, Read};

//===================================
// Read entire file to string (UTF-8)
//===================================
// Use this for: Config files, small documents, known-small inputs
fn read_to_string(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
    //==================================================
    // Allocates a String big enough for the entire file
    //==================================================
    // Returns Err if file doesn't exist, isn't readable, or isn't valid UTF-8
}

//===================================
// Read entire file to bytes (binary)
//===================================
// Use this for: Images, compressed files, any binary format
fn read_to_bytes(path: &str) -> io::Result<Vec<u8>> {
    std::fs::read(path)
    //========================================
    // Allocates a Vec<u8> and reads all bytes
    //========================================
    // Returns Err if file doesn't exist or isn't readable
}

//===================================
// Manual reading with buffer control
//===================================
// Use this when: You need to handle large files or control allocation
fn read_with_buffer(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    //==================================================================
    // read_to_string reads until EOF, automatically resizing the String
    //==================================================================
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

//===========================
// Read exact number of bytes
//===========================
// Use this for: Fixed-size headers, record-based binary formats
fn read_exact_bytes(path: &str, n: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; n];

    //===========================================================
    // read_exact returns Err if fewer than n bytes are available
    //===========================================================
    // This guarantees you get all n bytes or an error—no partial reads
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
```

**When each pattern fits**:
- `fs::read_to_string()`: Config files < 10 MB, HTML templates, small data files
- `fs::read()`: Binary files you'll process in-memory (images, compressed data)
- Manual `File::open()` + `read_to_string()`: Same as above but when you need the `File` handle for metadata
- `read_exact()`: Binary protocols with fixed-size headers, database page reads

**The UTF-8 guarantee**: `read_to_string` validates UTF-8. If your file contains invalid Unicode, it returns `Err`. Use `read()` for binary data or `String::from_utf8_lossy()` if you want replacement characters instead of errors.

### Basic File Writing

Writing is simpler than reading because you control the data format. The main decision: overwrite, append, or create new?

```rust
use std::fs::File;
use std::io::{self, Write};

//===========================================
// Write string to file (overwrites existing)
//===========================================
// Use this for: Writing configuration, saving user data, generating output files
fn write_string(path: &str, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
    //=================================
    // Creates file if it doesn't exist
    //=================================
    // Truncates (erases) existing content
    //====================================
    // Writes all content in one operation
    //====================================
}

//====================
// Write bytes to file
//====================
// Use this for: Binary data, serialized structures, any non-text data
fn write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    std::fs::write(path, content)
}

//================================
// Manual writing with file handle
//================================
// Use this when: You need multiple write calls or explicit error handling per write
fn write_with_handle(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    //=======================================================
    // write_all ensures all bytes are written or returns Err
    //=======================================================
    // Partial writes are retried automatically
    file.write_all(content.as_bytes())?;
    Ok(())
}

//============================================
// Append to file (preserves existing content)
//============================================
// Use this for: Log files, audit trails, incremental data collection
fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .append(true)    // Open in append mode
        .create(true)    // Create if doesn't exist
        .open(path)?;

    writeln!(file, "{}", content)?;  // Adds newline automatically
    Ok(())
}
```

**Critical distinction**: `File::create()` truncates (erases) existing files. If you mean to append, use `OpenOptions`. Many bugs come from accidentally truncating when you meant to append.

**Automatic flushing**: When a `File` is dropped, Rust automatically flushes buffered writes and closes the file. You usually don't need explicit `flush()` unless you're coordinating with external processes that need immediate visibility.

### File Opening Options

`OpenOptions` gives fine-grained control over how files are opened—essential for implementing robust file-based protocols or handling concurrent access.

```rust
use std::fs::OpenOptions;
use std::io;

fn advanced_file_opening() -> io::Result<()> {
    //========================================
    // Read-only mode (default for File::open)
    //========================================
    // Fails if file doesn't exist
    let file = OpenOptions::new()
        .read(true)
        .open("data.txt")?;

    //=========================================
    // Write-only mode, create if doesn't exist
    //=========================================
    // This is what File::create() does internally
    let file = OpenOptions::new()
        .write(true)
        .create(true)     // Create if missing
        .open("output.txt")?;

    //===========================================
    // Append mode (write to end, never truncate)
    //===========================================
    // Essential for log files
    let file = OpenOptions::new()
        .append(true)
        .open("log.txt")?;

    //======================================
    // Truncate existing file to zero length
    //======================================
    // Dangerous: erases all existing content!
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("temp.txt")?;

    //===========================================
    // Create new file, fail if it already exists
    //===========================================
    // Use this to avoid overwriting important files
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)   // Fail if exists (atomic check-and-create)
        .open("unique.txt")?;

    //================================================
    // Read and write mode (for in-place modification)
    //================================================
    // Allows seeking and both reading and writing
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data.bin")?;

    //===============================
    // Custom permissions (Unix only)
    //===============================
    // Set file mode bits (rwxrwxrwx)
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o644)      // rw-r--r-- (owner can write, others can read)
            .open("secure.txt")?;
    }

    Ok(())
}
```

**Common patterns**:
- **Append-only log**: `.append(true).create(true)` — Never loses data, safe for multiple writers
- **Exclusive creation**: `.write(true).create_new(true)` — Atomic: either you create it or fail
- **Read-modify-write**: `.read(true).write(true)` — Allows seeking to update parts of file

**Concurrency gotcha**: Opening a file for writing doesn't lock it on most platforms. Multiple processes can open the same file simultaneously and corrupt it. Use file locking (platform-specific) or atomic file replacement (write to temp, then rename) for safe concurrent access.

### Buffered I/O

The single most important I/O optimization is buffering. Reading or writing one byte at a time makes a system call per byte—catastrophically slow. Buffering batches operations, amortizing system call overhead across thousands of bytes.

**The performance cliff**: Reading a 100 MB file byte-by-byte takes minutes. Buffered reading takes milliseconds. The difference is 1000x.

```rust
use std::fs::File;
use std::io::{self, BufReader, BufWriter, BufRead, Write};

//=========================================================
// Buffered reading (essential for line-by-line processing)
//=========================================================
// Use this for: Log files, CSV files, any line-oriented text data
fn buffered_read(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);  // Wraps File in 8 KB buffer

    for line in reader.lines() {
        let line = line?;  // Each line is String, allocation per line
        println!("{}", line);
    }

    Ok(())
}

//=========================================
// Buffered reading with custom buffer size
//=========================================
// Use this when: Default 8 KB isn't right (larger for network files, smaller for embedded)
fn buffered_read_custom_size(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);  // 64 KB buffer

    for line in reader.lines() {
        println!("{}", line?);
    }

    Ok(())
}

//=============================================
// Buffered writing (essential for performance)
//=============================================
// Use this for: Generating output files, writing logs, any repeated write operations
fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);  // 8 KB buffer by default

    for line in lines {
        //================================================
        // writeln! writes to buffer, not directly to file
        //================================================
        // Much faster than unbuffered writes
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?;  // Ensure all buffered data is written before returning
    Ok(())
}

//===================================================
// Read until delimiter (useful for binary protocols)
//===================================================
// Use this for: Null-terminated strings, record separators, custom delimiters
fn read_until_delimiter(path: &str, delimiter: u8) -> io::Result<Vec<Vec<u8>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();

    loop {
        let mut chunk = Vec::new();
        let bytes_read = reader.read_until(delimiter, &mut chunk)?;

        if bytes_read == 0 {
            break;  // EOF reached
        }

        chunks.push(chunk);
    }

    Ok(chunks)
}
```

**How buffering works**: `BufReader` maintains an internal buffer (default 8 KB). When you read, it fills the buffer with one system call, then serves subsequent reads from memory. When the buffer empties, it refills. This transforms many small system calls into a few large ones.

**When to use larger buffers**:
- Network-mounted files: 64 KB or 1 MB buffers reduce round-trips
- Compression/encryption: Larger buffers improve throughput
- Sequential reads of huge files: Larger buffers can prefetch more data

**When to use smaller buffers**:
- Embedded systems with limited RAM
- Interactive applications where low latency matters more than throughput

**Flushing**: Buffered writers hold data in memory until the buffer fills or you explicitly `flush()`. If your program crashes before flushing, buffered writes are lost. For critical data (logs, database commits), flush after each important write.

### Line-by-Line Processing

Processing large files line-by-line is a fundamental pattern. Log analysis, data transformation, text processing—all benefit from streaming rather than loading the entire file.

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

//====================================================
// Process large files line by line (memory-efficient)
//====================================================
// This pattern scales to files larger than RAM
fn process_large_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line?;  // Handle I/O errors per line

        //==================
        // Process each line
        //==================
        if line.starts_with('#') {
            continue;  // Skip comments
        }

        println!("Line {}: {}", index + 1, line);
    }

    Ok(())
}

//=======================================================
// Read all lines into memory (simpler but uses more RAM)
//=======================================================
// Use this only for small files where convenience trumps memory usage
fn read_lines_robust(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    //=====================================
    // collect() gathers all lines into Vec
    //=====================================
    // If any line fails to read, the entire operation fails
    reader.lines().collect()
}

//==========================================================
// Read specific line by index (inefficient for large files)
//==========================================================
// For frequently accessing specific lines, build an index or use mmap
fn read_line_at_index(path: &str, target: usize) -> io::Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    //===============================
    // nth() skips to the target line
    //===============================
    // Inefficient: still reads all prior lines, just discards them
    reader.lines()
        .nth(target)
        .transpose()  // Converts Option<Result> to Result<Option>
}
```

**Memory usage**: `lines()` allocates a `String` per line. For a file with 1 million lines averaging 100 bytes each, that's ~100 MB of allocations (though only one line is live at a time). If allocation overhead matters, read into a reused buffer with `read_line()`.

**Performance tip**: If you're filtering lines (e.g., only processing lines containing "ERROR"), combine filtering with iteration:

```rust
fn process_errors_only(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines()
        .filter_map(|line| line.ok())  // Convert Result to Option, discarding errors
        .filter(|line| line.contains("ERROR"))
        .collect()
}
```

**Error handling strategy**: Should one bad line fail the entire operation? It depends:
- **Strict**: Use `.collect()` to fail on first error (data integrity critical)
- **Best-effort**: Use `.filter_map(Result::ok)` to skip bad lines (log parsing where some corruption is acceptable)

### Seeking and Random Access

Files aren't always read sequentially. Databases jump to specific record offsets. Binary formats have headers followed by data sections. Video players seek to arbitrary timestamps. For these cases, you need random access.

```rust
use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};

fn seeking_example(path: &str) -> io::Result<()> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    //================
    // Write some data
    //================
    file.write_all(b"Hello, World!")?;

    //==================
    // Seek to beginning
    //==================
    file.seek(SeekFrom::Start(0))?;  // Absolute offset from start

    //===================
    // Read first 5 bytes
    //===================
    let mut buffer = [0; 5];
    file.read_exact(&mut buffer)?;
    println!("First 5 bytes: {:?}", std::str::from_utf8(&buffer)?);  // "Hello"

    //================================
    // Seek to end (returns file size)
    //================================
    let file_size = file.seek(SeekFrom::End(0))?;
    println!("File size: {}", file_size);  // 13 bytes

    //==================================
    // Seek relative to current position
    //==================================
    file.seek(SeekFrom::Current(-5))?;  // Back up 5 bytes from end

    //===========================
    // Read from current position
    //===========================
    let mut buffer = [0; 5];
    file.read_exact(&mut buffer)?;
    println!("Last 5 bytes: {:?}", std::str::from_utf8(&buffer)?);  // "orld!"

    Ok(())
}

//======================================
// Random access read at specific offset
//======================================
// Use this for: Database page reads, binary format parsing, indexed files
fn read_at_offset(path: &str, offset: u64, size: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0; size];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

//=======================================
// Overwrite specific region (dangerous!)
//=======================================
// Use this for: In-place updates, database writes, binary format modification
fn write_at_offset(path: &str, offset: u64, data: &[u8]) -> io::Result<()> {
    let mut file = File::options()
        .write(true)  // Write mode required
        .open(path)?;

    file.seek(SeekFrom::Start(offset))?;
    file.write_all(data)?;
    Ok(())
}
```

**Seeking rules**:
- `SeekFrom::Start(n)`: Absolute position from beginning (most common)
- `SeekFrom::End(n)`: Relative to end (n is usually 0 or negative)
- `SeekFrom::Current(n)`: Relative to current position (positive or negative)

**Performance**: Seeking is cheap on modern filesystems—it's just updating an offset. But seeking breaks sequential I/O patterns, hurting prefetch and disk cache. If you're reading most of a file, sequential reads are faster even if you skip data.

**Caveat**: You can seek past the end of a file and write there. This creates a "sparse file" with a hole. Reading the hole returns zeros. Not all filesystems support this.

### File Metadata and Permissions

Files carry metadata: size, timestamps, permissions. You often need this information before deciding how to process a file.

```rust
use std::fs;
use std::io;
use std::time::SystemTime;

fn file_metadata_example(path: &str) -> io::Result<()> {
    let metadata = fs::metadata(path)?;

    println!("File size: {} bytes", metadata.len());
    println!("Is directory: {}", metadata.is_dir());
    println!("Is file: {}", metadata.is_file());
    println!("Is symlink: {}", metadata.is_symlink());
    println!("Read-only: {}", metadata.permissions().readonly());

    //===================================================
    // Timestamps (may not be available on all platforms)
    //===================================================
    if let Ok(modified) = metadata.modified() {
        println!("Modified: {:?}", modified);
    }

    if let Ok(accessed) = metadata.accessed() {
        println!("Accessed: {:?}", accessed);
    }

    if let Ok(created) = metadata.created() {
        println!("Created: {:?}", created);
    }

    //=======================
    // Unix-specific metadata
    //=======================
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        println!("UID: {}", metadata.uid());
        println!("GID: {}", metadata.gid());
        println!("Mode: {:o}", metadata.mode());  // Octal format for readability
    }

    Ok(())
}

//======================================================
// Set permissions (cross-platform: read-only flag only)
//======================================================
fn set_permissions(path: &str, readonly: bool) -> io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_readonly(readonly);
    fs::set_permissions(path, perms)?;
    Ok(())
}

//========================================
// Unix-specific: Set full permission mode
//========================================
#[cfg(unix)]
fn set_mode(path: &str, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms)?;
    Ok(())
}
```

**Use cases for metadata**:
- **Size checks**: Refuse to load files > 100 MB into memory
- **Timestamp checks**: Invalidate caches when source file is newer than cached version
- **Permission checks**: Warn if sensitive files are world-readable
- **Type checks**: Skip directories when expecting files

**Symlink behavior**: `fs::metadata()` follows symlinks (returns info about the target). Use `fs::symlink_metadata()` to get info about the symlink itself.

### Copying and Moving Files

File operations beyond read/write are common in build systems, installers, and file management tools.

```rust
use std::fs;
use std::io;
use std::path::Path;

//===========================
// Copy file (simple version)
//===========================
// Returns number of bytes copied
fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    fs::copy(src, dst)
    //=================================
    // Copies permissions too (on Unix)
    //=================================
    // Overwrites destination if it exists
}

//============================
// Copy with progress tracking
//============================
// Use this for: Large file copies, user-facing file operations
fn copy_file_with_progress(src: &str, dst: &str) -> io::Result<()> {
    use std::io::{BufReader, BufWriter, Read, Write};

    let src_file = File::open(src)?;
    let dst_file = File::create(dst)?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    let total_size = reader.get_ref().metadata()?.len();
    let mut copied = 0u64;
    let mut buffer = [0; 8192];  // 8 KB buffer

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;  // EOF
        }

        writer.write_all(&buffer[..n])?;
        copied += n as u64;

        let progress = (copied as f64 / total_size as f64) * 100.0;
        print!("\rProgress: {:.2}%", progress);
        io::stdout().flush()?;  // Ensure progress is displayed
    }

    println!("\nCopy complete!");
    Ok(())
}

//=================
// Move/rename file
//=================
// Atomic within same filesystem; cross-filesystem does copy+delete
fn move_file(src: &str, dst: &str) -> io::Result<()> {
    fs::rename(src, dst)
    //===============================================
    // Fails if destination exists and is a directory
    //===============================================
    // Fails if crossing filesystem boundaries (use copy+delete instead)
}

//=======================================
// Hard link (same inode, multiple names)
//=======================================
// Changes to one affect the other; both point to same data
fn create_hard_link(src: &str, dst: &str) -> io::Result<()> {
    fs::hard_link(src, dst)
    //==================================
    // Only works within same filesystem
    //==================================
    // Can't hardlink directories (prevents filesystem cycles)
}

//========================================
// Symbolic link (pointer to another path)
//========================================
#[cfg(unix)]
fn create_symlink(src: &str, dst: &str) -> io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
    //===================================
    // Target can be relative or absolute
    //===================================
    // Target doesn't have to exist (dangling symlink)
    //=========================
    // Works across filesystems
    //=========================
}

//============
// Remove file
//============
fn delete_file(path: &str) -> io::Result<()> {
    fs::remove_file(path)
    //======================================================
    // Fails if path is a directory (use remove_dir instead)
    //======================================================
    // Fails if file doesn't exist
}
```

**Copy performance**: `fs::copy()` uses platform-specific optimizations (like `sendfile` on Linux) that can be 10x faster than manual read/write loops. Use the manual approach only when you need progress tracking or filtering during copy.

**Rename atomicity**: On Unix, `rename()` is atomic within the same filesystem. This is how you implement safe file replacement: write to temp file, then rename over the original. Either the rename succeeds completely or fails completely—no partial updates.

**Cross-filesystem moves**: `fs::rename()` fails if source and destination are on different filesystems. Handle this by falling back to copy + delete.

---

## Standard Streams (stdin/stdout/stderr)

Every Unix process has three file descriptors by default: stdin (input), stdout (output), and stderr (errors). These connect your program to the terminal, enable shell pipelines, and separate normal output from diagnostics.

Understanding standard streams is essential for command-line tools. A program that writes errors to stdout instead of stderr breaks shell pipelines. A program that doesn't flush stdout before reading stdin creates confusing UX where prompts don't appear.

### Reading from stdin

User input, piped data from other programs, redirected files—all come through stdin.

```rust
use std::io::{self, BufRead, Write};

//============================
// Read single line from stdin
//============================
// Use this for: Simple prompts, single-input commands
fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())  // Remove trailing newline
}

//=================
// Read with prompt
//=================
// Essential pattern for interactive programs
fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;  // CRITICAL: flush before reading
    //==============================================================
    // Without flush, prompt might not appear until after user input
    //==============================================================
    read_line()
}

//==========================
// Read all lines from stdin
//==========================
// Use this for: Processing piped input, reading until EOF
fn read_lines() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let reader = stdin.lock();  // Lock for efficient multiple reads

    reader.lines().collect()
    //======================================
    // Returns Err if any line has I/O error
    //======================================
    // EOF is not an error; returns empty Vec
}

//====================================================
// Read until empty line (useful for multi-line input)
//====================================================
fn read_until_empty() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let mut lines = Vec::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;  // Empty line signals end
        }
        lines.push(line);
    }

    Ok(lines)
}

//=======================
// Read and parse integer
//=======================
// Demonstrates error conversion
fn read_integer() -> io::Result<i32> {
    let input = prompt("Enter a number: ")?;

    //=================================
    // Convert parse error to I/O error
    //=================================
    input.parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

//======================================
// Interactive menu (common CLI pattern)
//======================================
fn interactive_menu() -> io::Result<()> {
    loop {
        println!("\n=== Menu ===");
        println!("1. Option 1");
        println!("2. Option 2");
        println!("3. Exit");

        let choice = prompt("Enter choice: ")?;

        match choice.as_str() {
            "1" => println!("Selected option 1"),
            "2" => println!("Selected option 2"),
            "3" => break,
            _ => println!("Invalid choice"),
        }
    }

    Ok(())
}
```

**Why locking matters**: `stdin.lock()` acquires a lock on stdin for the duration of the returned handle. This is faster than locking per operation. Use it when reading multiple lines.

**Flushing before reading**: Stdout is line-buffered by default (flushes on newline). But `print!()` (no newline) doesn't trigger a flush. Always flush before reading if you used `print!()` for a prompt.

**EOF handling**: On Unix, Ctrl+D sends EOF. On Windows, Ctrl+Z then Enter. Your code should handle EOF gracefully (it's not an error).

### Writing to stdout/stderr

**Fundamental rule**: Normal output goes to stdout. Diagnostics and errors go to stderr. This allows users to redirect output (`program > output.txt`) while still seeing errors on the terminal.

```rust
use std::io::{self, Write};

fn stdout_examples() -> io::Result<()> {
    //=============================
    // Basic println! (most common)
    //=============================
    println!("Hello, World!");  // Writes to stdout with newline

    //=========================
    // Write to stdout directly
    //=========================
    io::stdout().write_all(b"Direct write\n")?;

    //=================================
    // Buffered writing for performance
    //=================================
    let stdout = io::stdout();
    let mut handle = stdout.lock();  // Lock once for multiple writes
    writeln!(handle, "Buffered write")?;
    handle.flush()?;

    //======================
    // Write without newline
    //======================
    print!("No newline ");
    io::stdout().flush()?;  // Ensure it appears immediately
    println!("here!");

    Ok(())
}

fn stderr_examples() -> io::Result<()> {
    //=============================================
    // Write to stderr (for errors and diagnostics)
    //=============================================
    eprintln!("Error message");  // Like println! but goes to stderr

    //====================
    // Direct stderr write
    //====================
    io::stderr().write_all(b"Direct error\n")?;

    //================
    // Formatted error
    //================
    let error_code = 42;
    eprintln!("Error code: {}", error_code);

    Ok(())
}

//=====================================
// Progress indicator (classic pattern)
//=====================================
fn progress_indicator() -> io::Result<()> {
    for i in 0..=100 {
        print!("\rProgress: {}%", i);  // \r returns to start of line
        io::stdout().flush()?;  // Force update
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!();  // Final newline
    Ok(())
}

//=======================================
// Colored output using ANSI escape codes
//=======================================
// Works on Unix terminals and modern Windows (Windows 10+)
fn colored_output() {
    println!("\x1b[31mRed text\x1b[0m");      // Red
    println!("\x1b[32mGreen text\x1b[0m");    // Green
    println!("\x1b[33mYellow text\x1b[0m");   // Yellow
    println!("\x1b[1mBold text\x1b[0m");      // Bold
    //==========================
    // \x1b[0m resets formatting
    //==========================
}
```

**When to use stderr**:
- Error messages
- Warnings
- Debug output (when not using a logging framework)
- Progress indicators (so they don't pollute piped output)

**When to use stdout**:
- Program results
- Data to be piped to other programs
- JSON/CSV output
- Any "real" output that users care about

**Bad practice**: Writing errors to stdout. This breaks pipelines where users expect only data on stdout.

### Locking Streams for Performance

Each write to stdout acquires and releases a lock. For thousands of writes, this overhead adds up. Locking once and holding the lock through many writes is faster.

```rust
use std::io::{self, BufRead, Write};

//==========================================================
// Efficient stdout writing (10-100x faster for many writes)
//==========================================================
fn efficient_stdout_writing(lines: &[&str]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();  // Lock once

    for line in lines {
        writeln!(handle, "{}", line)?;  // Many writes, one lock
    }

    //==============================================
    // Lock automatically released when handle drops
    //==============================================
    Ok(())
}

//========================
// Efficient stdin reading
//========================
fn efficient_stdin_reading() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let reader = stdin.lock();  // Lock once

    reader.lines().collect()
}

//=================================
// Combined stdin/stdout operations
//=================================
// Classic Unix filter pattern
fn echo_uppercase() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        writeln!(writer, "{}", line.to_uppercase())?;
        line.clear();  // Reuse String buffer
    }

    Ok(())
}
```

**Performance impact**: For 10,000 lines, locked writing is ~50x faster than unlocked. The difference is massive in tight loops.

**When locking doesn't help**: Single writes don't benefit. The overhead of acquiring the lock is the same. Lock only when you have multiple writes in sequence.

### Redirecting Streams

Sometimes you want to capture output that would normally go to the terminal, or send your program's output to a file.

```rust
use std::fs::File;
use std::io::{self, Write};

//===========================================
// Capture output to file instead of terminal
//===========================================
fn redirect_stdout_to_file(path: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = io::BufWriter::new(file);

    writeln!(writer, "This goes to file instead of stdout")?;
    writeln!(writer, "Another line")?;

    Ok(())
}

//===================================
// Tee: write to both stdout and file
//===================================
// Like Unix `tee` command
fn tee_output(path: &str, message: &str) -> io::Result<()> {
    //================
    // Write to stdout
    //================
    println!("{}", message);

    //===================
    // Also write to file
    //===================
    let mut file = File::create(path)?;
    writeln!(file, "{}", message)?;

    Ok(())
}

//========================================================
// Custom writer that multiplexes to multiple destinations
//========================================================
struct MultiWriter {
    writers: Vec<Box<dyn Write>>,
}

impl MultiWriter {
    fn new() -> Self {
        MultiWriter {
            writers: Vec::new(),
        }
    }

    fn add_writer(&mut self, writer: Box<dyn Write>) {
        self.writers.push(writer);
    }
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        for writer in &mut self.writers {
            writer.write_all(buf)?;  // Write to all destinations
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?;
        }
        Ok(())
    }
}
```

**Use cases**:
- Logging to both file and console
- Testing code that writes to stdout
- Building command-line tools that support `--output` flags

---

## Memory-Mapped I/O

Memory-mapped I/O (mmap) makes files appear as if they were memory. Instead of explicit read/write calls, you access bytes through pointers. The operating system handles paging data in and out automatically.

**When mmap shines**:
- Random access patterns (databases, indexes)
- Large files processed in chunks
- Shared memory between processes
- Zero-copy parsing (treat file bytes as structs directly)

**When mmap hurts**:
- Sequential reads (buffered I/O is simpler and often faster)
- Small files (setup overhead exceeds benefits)
- Windows (fewer guarantees about mmap semantics)

### Basic Memory Mapping

Rust doesn't include mmap in the standard library (it's unsafe and platform-specific). The `memmap2` crate provides a safe abstraction.

```rust
//=========================
// Note: Add to Cargo.toml:
//=========================
// [dependencies]
//================
// memmap2 = "0.9"
//================

//=============================================
// This is a conceptual example showing the API
//=============================================
#[cfg(feature = "memmap_example")]
mod memmap_examples {
    use memmap2::{Mmap, MmapMut, MmapOptions};
    use std::fs::File;
    use std::io::{self, Write};

    //=====================
    // Read-only memory map
    //=====================
    // Use this for: Large read-only data files, databases
    fn mmap_read(path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        //================================
        // Access memory like a byte slice
        //================================
        let data: &[u8] = &mmap[..];
        println!("First 10 bytes: {:?}", &data[..10.min(data.len())]);

        //==============================
        // mmap is unmapped when dropped
        //==============================
        Ok(())
    }

    //===================
    // Mutable memory map
    //===================
    // Use this for: In-place file modification, persistent data structures
    fn mmap_write(path: &str) -> io::Result<()> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        //=============================
        // Set file size before mapping
        //=============================
        file.set_len(1024)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        //==================================================
        // Write data directly to memory (actually the file)
        //==================================================
        mmap[0..5].copy_from_slice(b"Hello");

        //==================================
        // Flush to ensure writes reach disk
        //==================================
        mmap.flush()?;

        Ok(())
    }

    //==========================================
    // Anonymous memory map (not backed by file)
    //==========================================
    // Use this for: Shared memory IPC, large temporary buffers
    fn mmap_anonymous() -> io::Result<()> {
        let mut mmap = MmapMut::map_anon(1024)?;

        mmap[0..5].copy_from_slice(b"Hello");

        println!("Data: {:?}", &mmap[0..5]);

        Ok(())
    }

    //================================
    // Large file processing with mmap
    //================================
    // Use this when: File is too large for RAM but you need random access
    fn process_large_file_mmap(path: &str) -> io::Result<usize> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        //======================================================
        // Count newlines efficiently (CPU-bound, not I/O-bound)
        //======================================================
        let count = mmap.iter().filter(|&&b| b == b'\n').count();

        Ok(count)
    }
}
```

**Why `unsafe`**: The OS can change mapped memory at any time (e.g., if another process modifies the file). Rust can't guarantee your references remain valid. The `memmap2` crate encapsulates this unsafety.

**Performance characteristics**:
- **Cold access**: First access to a page causes page fault (OS loads page from disk). Slower than buffered read.
- **Hot access**: Subsequent access to same page is pure memory speed. Faster than buffered read.
- **Random access**: Mmap excels. Buffered I/O requires seeks.

**Gotcha**: Mmap doesn't necessarily improve performance. For sequential reads, `BufReader` is simpler and often faster. Measure first.

---

## Directory Traversal

File trees are fundamental: source code directories, log directories, photo libraries. You need to walk these trees to find files, calculate sizes, or perform batch operations.

### Basic Directory Operations

```rust
use std::fs;
use std::io;
use std::path::Path;

//=================
// Create directory
//=================
fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir(path)
    //==============================
    // Fails if parent doesn't exist
    //==============================
    // Fails if directory already exists
}

//============================================
// Create directory and all parent directories
//============================================
// Like mkdir -p in Unix
fn create_directory_all(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
    //=====================================
    // Creates parent directories as needed
    //=====================================
    // Succeeds if directory already exists
}

//=======================
// Remove empty directory
//=======================
fn remove_directory(path: &str) -> io::Result<()> {
    fs::remove_dir(path)
    //================================
    // Fails if directory is not empty
    //================================
}

//===============================================
// Remove directory and all contents (dangerous!)
//===============================================
fn remove_directory_all(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
    //===============================
    // Recursively deletes everything
    //===============================
    // Like rm -rf in Unix
}

//=====================
// Check if path exists
//=====================
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
    //==================================
    // Returns false for broken symlinks
    //==================================
}

//===========================
// Check if path is directory
//===========================
fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
    //=================
    // Follows symlinks
    //=================
}
```

**Safety note**: `remove_dir_all` is dangerous. It's equivalent to `rm -rf`. There's no trash bin, no undo. Many programs ask for confirmation before using this.

### Reading Directory Contents

Listing a directory is simple, but the iterator-based API requires careful error handling.

```rust
use std::fs;
use std::io;
use std::path::PathBuf;

//========================
// List directory contents
//========================
fn list_directory(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;  // Each entry can fail
        entries.push(entry.path());
    }

    Ok(entries)
}

//===================================
// List only files (skip directories)
//===================================
fn list_files_only(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }

    Ok(files)
}

//===================================
// List files with specific extension
//===================================
// Use this for: Finding all .rs files, .txt files, etc.
fn list_by_extension(path: &str, ext: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some(ext) {
            files.push(path);
        }
    }

    Ok(files)
}

//====================================
// Get directory entries with metadata
//====================================
// Use this for: Sorting by size, filtering by date, etc.
fn list_with_metadata(path: &str) -> io::Result<Vec<(PathBuf, fs::Metadata)>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push((entry.path(), metadata));
    }

    Ok(entries)
}
```

**Error handling**: `read_dir()` can fail (directory doesn't exist, no permission). Each call to `entry?` can also fail (permission denied on individual files). Handle both.

### Recursive Directory Traversal

Walking entire directory trees is common but requires careful handling of errors and cycles (symlink loops).

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

//=======================
// Recursive file listing
//=======================
// Classic depth-first search pattern
fn walk_directory(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                walk_directory(&path, files)?;  // Recurse
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

//==========================
// Get all files recursively
//==========================
fn get_all_files(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory(Path::new(path), &mut files)?;
    Ok(files)
}

//===============================================
// Recursive directory tree printer (visual tree)
//===============================================
// Produces output like the `tree` command
fn print_tree(path: &Path, prefix: &str) -> io::Result<()> {
    let entries = fs::read_dir(path)?;
    let mut entries: Vec<_> = entries.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let extension = if is_last { "    " } else { "│   " };

        println!("{}{}{}", prefix, connector, entry.file_name().to_string_lossy());

        if entry.file_type()?.is_dir() {
            let new_prefix = format!("{}{}", prefix, extension);
            print_tree(&entry.path(), &new_prefix)?;
        }
    }

    Ok(())
}

//================================================
// Find files matching pattern (like find command)
//================================================
fn find_files(root: &Path, pattern: &str) -> io::Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    fn search(path: &Path, pattern: &str, matches: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                search(&path, pattern, matches)?;
            } else if path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains(pattern))
                .unwrap_or(false)
            {
                matches.push(path);
            }
        }
        Ok(())
    }

    search(root, pattern, &mut matches)?;
    Ok(matches)
}
```

**Symlink loops**: This code doesn't detect symlink cycles. If `/a/b` symlinks to `/a`, you'll recurse forever. Production code should track visited inodes (Unix) or use a depth limit.

**Performance**: For very large directories (millions of files), consider parallel traversal or using OS-specific optimizations (like Linux's `readdir64`).

---

## Process Spawning and Piping

Launching external programs and piping data between them is fundamental Unix philosophy. Rust's `std::process` provides ergonomic, safe process management.

### Basic Process Execution

```rust
use std::process::{Command, Stdio};
use std::io::{self, Write};

//===============================
// Run command and capture output
//===============================
fn run_command() -> io::Result<()> {
    let output = Command::new("ls")
        .arg("-la")
        .output()?;  // Waits for completion, captures all output

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

//===========================
// Check if command succeeded
//===========================
fn run_command_check() -> io::Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .status()?;  // Inherits stdin/stdout/stderr, just waits for completion

    if status.success() {
        println!("Build succeeded!");
    } else {
        println!("Build failed with: {}", status);
    }

    Ok(())
}

//===============================
// Run with environment variables
//===============================
fn run_with_env() -> io::Result<()> {
    let output = Command::new("printenv")
        .env("MY_VAR", "my_value")
        .env("ANOTHER_VAR", "another_value")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

//==========================
// Run in specific directory
//==========================
fn run_in_directory() -> io::Result<()> {
    let output = Command::new("pwd")
        .current_dir("/tmp")
        .output()?;

    println!("Working directory: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
```

**API choices**:
- `.output()`: Captures stdout/stderr, waits for exit. Use for short commands.
- `.status()`: Inherits streams, just returns exit code. Use for interactive commands.
- `.spawn()`: Returns immediately with `Child` handle. Use for async or long-running processes.

### Streaming Output

Capturing all output before processing isn't always feasible. Long-running processes need real-time feedback.

```rust
use std::process::{Command, Stdio};
use std::io::{self, BufRead, BufReader};

//===========================
// Stream stdout in real-time
//===========================
fn stream_output() -> io::Result<()> {
    let mut child = Command::new("ping")
        .arg("-c")
        .arg("5")
        .arg("8.8.8.8")
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        println!("Output: {}", line?);
    }

    let status = child.wait()?;
    println!("Exit status: {}", status);

    Ok(())
}

//==========================================
// Capture both stdout and stderr separately
//==========================================
// Requires threading to avoid deadlock
fn capture_both_streams() -> io::Result<()> {
    let mut child = Command::new("cargo")
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    //==========================
    // Read stdout in one thread
    //==========================
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[OUT] {}", line);
            }
        }
    });

    //==============================
    // Read stderr in another thread
    //==============================
    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                eprintln!("[ERR] {}", line);
            }
        }
    });

    stdout_thread.join().unwrap();
    stderr_thread.join().unwrap();

    let status = child.wait()?;
    println!("Process exited with: {}", status);

    Ok(())
}
```

**Deadlock warning**: If you read stdout while the child is blocked writing to stderr (and vice versa), you deadlock. Use threads or async I/O to read both concurrently.

### Piping Between Processes

Unix pipelines (`cat file | grep pattern | wc -l`) chain processes, streaming data without intermediate files.

```rust
use std::process::{Command, Stdio};
use std::io::{self, Write};

//========================================================
// Pipe output from one command to another (ls | grep txt)
//========================================================
fn pipe_commands() -> io::Result<()> {
    let ls = Command::new("ls")
        .stdout(Stdio::piped())
        .spawn()?;

    let grep = Command::new("grep")
        .arg("txt")
        .stdin(Stdio::from(ls.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = grep.wait_with_output()?;
    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

//==================================================
// Complex pipeline: cat file | grep pattern | wc -l
//==================================================
fn complex_pipeline(file: &str, pattern: &str) -> io::Result<()> {
    let cat = Command::new("cat")
        .arg(file)
        .stdout(Stdio::piped())
        .spawn()?;

    let grep = Command::new("grep")
        .arg(pattern)
        .stdin(Stdio::from(cat.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let wc = Command::new("wc")
        .arg("-l")
        .stdin(Stdio::from(grep.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()?;

    let output = wc.wait_with_output()?;
    println!("Lines matching '{}': {}", pattern, String::from_utf8_lossy(&output.stdout).trim());

    Ok(())
}
```

**How piping works**: `Stdio::from(child.stdout.unwrap())` passes the child's stdout as stdin to the next process. The OS manages the buffer between processes.

This comprehensive guide covers synchronous I/O patterns from file operations to process management, providing the foundation for reliable systems programming in Rust.
