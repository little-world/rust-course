# Synchronous I/O

[Basic File Operations](#pattern-1-basic-file-operations)

- Problem: Persist data for a long time
- Solution: Files are the main way programs persist data
- Why It Matters: Files are used all the time
- Use Cases: Configuration, logs, cached results, user documents


[Buffered Reading and Writing](#pattern-2-buffered-reading-and-writing)

- Problem: Reading byte-by-byte makes a syscall per byte (O(N²)); writing many small chunks slow; 100MB unbuffered takes minutes
- Solution: BufReader/BufWriter with 8KB+ buffer; amortizes syscalls; lines() for text; flush() for critical data
- Why It Matters: 1000x faster (unbuffered 100MB in minutes vs buffered in milliseconds); syscall overhead dominant cost
- Use Cases: Log parsing, CSV processing, config files, generating output, any line-oriented text

[Standard Streams (stdin/stdout/stderr)](#pattern-3-standard-streams)

- Problem: Programs need terminal I/O; pipelines break if errors go to stdout; prompts don't appear without flush
- Solution: stdin for input, stdout for data output, stderr for diagnostics; lock() for efficient multi-writes; flush() before reading
- Why It Matters: Correct stream separation enables shell pipelines; locking 50x faster for bulk writes; flushing prevents UX bugs
- Use Cases: CLI tools, filters (cat|grep), interactive prompts, progress indicators, logging

[Memory-Mapped I/O](#pattern-4-memory-mapped-io)

- Problem: Random access with read/seek O(N) per access; large files don't fit in RAM; copying wastes CPU
- Solution: memmap2: treat file as byte slice; OS handles paging; zero-copy parsing; shared memory IPC
- Why It Matters: Random access at memory speed (hot pages); databases/indexes need this; true zero-copy possible
- Use Cases: Databases, binary search in files, shared memory IPC, large read-only data, sparse file access

[Directory Traversal](#pattern-5-directory-traversal)

- Problem: Need to walk file trees; find files by pattern; calculate directory sizes; symlink loops cause infinite recursion
- Solution: fs::read_dir() for single level; recursive walk with path tracking; walkdir crate for production; filter by extension
- Why It Matters: Build systems scan thousands of files; backups walk entire disks; wrong impl hits symlink cycles
- Use Cases: Build systems (find sources), file search, backup tools, disk usage analyzers, batch file operations

[Process Spawning and Piping](#pattern-6-process-spawning-and-piping)

- Problem: Need to run external commands; capture output; chain processes; deadlock reading both stdout and stderr
- Solution: Command::new() with spawn/output/status; Stdio::piped() for capture; threads for concurrent stream reads
- Why It Matters: Integration with system tools; pipelines compose programs; proper stream handling prevents deadlocks
- Use Cases: Build scripts (run compilers), testing (run programs), automation, Unix pipelines, subprocess orchestration

[File IO Foundations](#file-io-foundations)   
- A long list of useful functions  

[Command Foundations](#command-foundations)    
- A long list of useful functions

# Overview
This chapter covers Rust's synchronous I/O patterns—blocking operations that pause threads until complete. Unlike async I/O (Chapter 17), synchronous I/O is simpler, easier to debug, and sufficient for most programs. CLI tools, build scripts, and even high-performance servers with thread pools rely on these patterns.


## Pattern 1: Basic File Operations

**Problem**: Files are the main way programs persist data—configuration, logs, cached results, user documents—but handling them correctly can be tricky. You need to decide how to read the file efficiently, handle different formats, and deal with errors safely.

**Solution**: Do you read the entire file into memory (`fs::read_to_string`) for simplicity, or process it line by line (`BufReader::lines`) to handle files larger than RAM? The right choice depends on your constraints.
Always handle errors using `Result` or `?`, and choose between text (`read_to_string`) and binary (`fs::read`) based on the file format.

**Why It Matters**: Choosing the wrong approach can cause:
Memory exhaustion if a large file is loaded at once.
Slow performance if line-by-line reading is overused for small files.
Crashes or silent failures if errors aren’t handled properly.

**Use Cases**: Reading  JSON file, Processing logs or CSV files , CLI tools or servers that need predictable file I/O behavior.

###  Example: Basic file reading
The simplest case: read a small file entirely into memory.

```rust
use std::fs::File;
use std::io::{self, Read};

//===================================
// Read entire file to string (UTF-8)
//===================================
fn read_to_string(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
    // Allocates a String big enough for the entire file
    // Returns Err if file doesn't exist, isn't readable, or isn't valid UTF-8
}

//===================================
// Read entire file to bytes (binary)
//===================================
fn read_to_bytes(path: &str) -> io::Result<Vec<u8>> {
    std::fs::read(path)
    // Allocates a Vec<u8> and reads all bytes
    // Returns Err if file doesn't exist or isn't readable
}

//===================================
// Manual reading with buffer control
//===================================
fn read_with_buffer(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    // read_to_string reads until EOF, automatically resizing the String
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

//===========================
// Read exact number of bytes
//===========================
fn read_exact_bytes(path: &str, n: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; n];

    // read_exact returns Err if fewer than n bytes are available
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

###  Example: Basic File Writing

Writing is simpler than reading because you control the data format. The main decision: overwrite, append, or create new?

```rust
use std::fs::File;
use std::io::{self, Write};

//===========================================
// Write string to file (overwrites existing)
//===========================================
fn write_string(path: &str, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
    // Creates file if it doesn't exist
    // Truncates (erases) existing content
    // Writes all content in one operation
}

//====================
// Write bytes to file
//====================
fn write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    std::fs::write(path, content)
}

//================================
// Manual writing with file handle
//================================
fn write_with_handle(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    // write_all ensures all bytes are written or returns Err
    // Partial writes are retried automatically
    file.write_all(content.as_bytes())?;
    Ok(())
}

//============================================
// Append to file (preserves existing content)
//============================================
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

###  Example: File Opening Options

`OpenOptions` gives fine-grained control over how files are opened—essential for implementing robust file-based protocols or handling concurrent access.

```rust
use std::fs::OpenOptions;
use std::io;

fn advanced_file_opening() -> io::Result<()> {
    // Read-only mode (default for File::open)
    // Fails if file doesn't exist
    let file = OpenOptions::new()
        .read(true)
        .open("data.txt")?;

    // Write-only mode, create if doesn't exist
    // This is what File::create() does internally
    let file = OpenOptions::new()
        .write(true)
        .create(true)     // Create if missing
        .open("output.txt")?;

    // Append mode (write to end, never truncate)
    // Essential for log files
    let file = OpenOptions::new()
        .append(true)
        .open("log.txt")?;

    // Truncate existing file to zero length
    // Dangerous: erases all existing content!
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("temp.txt")?;

    // Create new file, fail if it already exists
    // Use this to avoid overwriting important files
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)   // Fail if exists (atomic check-and-create)
        .open("unique.txt")?;

    // Read and write mode (for in-place modification)
    // Allows seeking and both reading and writing
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data.bin")?;

    // Custom permissions (Unix only)
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


## Pattern 2: Buffered Reading and Writing

**Problem**: Reading or writing files byte-by-byte makes a system call per byte—catastrophically slow with O(N) syscalls for N bytes. Processing a 100 MB file unbuffered can take minutes. Writing many small chunks suffers the same overhead. Line-by-line processing allocates per line. Without explicit flush(), critical writes may be lost on crash.

**Solution**: Wrap File handles in BufReader/BufWriter which maintain internal buffers (default 8 KB). BufReader amortizes reads: fills buffer with one syscall, serves subsequent reads from memory. BufWriter batches writes: accumulates data in memory, flushes when buffer fills. Use lines() for text processing, read_until() for custom delimiters. Call flush() after critical writes.

**Why It Matters**: Buffering provides 1000x speedup—unbuffered 100 MB file takes minutes, buffered takes milliseconds. Syscall overhead dominates unbuffered I/O. A task processing 1M log lines: unbuffered = O(N) syscalls, buffered = O(N/8192) syscalls. For bulk writes (generating output files), BufWriter similarly transforms performance. Without buffers, disk seeks per write destroy throughput.

**Use Cases**: Log file parsing (line-by-line), CSV processing (buffered reading), config file loading, generating reports (buffered writing), any text-oriented file I/O, binary protocol parsing with custom delimiters.

###  Example: Buffered Line-by-Line Reading

Process large log files or text data that doesn't fit in memory. Need memory-efficient streaming. Want to skip comments or filter lines.

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

//====================================================
// Process large files line by line (memory-efficient)
//====================================================
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

//=========================================
// Filter lines (e.g., only errors)
//=========================================
fn process_errors_only(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines()
        .filter_map(|line| line.ok())  // Skip I/O errors
        .filter(|line| line.contains("ERROR"))
        .collect()
}
```

**Key Benefits**:
- Memory usage: O(1) per line, not O(file size)
- Scales to files larger than RAM
- Easy filtering and transformation
- 1000x faster than byte-by-byte reads



###  Example: Buffered Writing for Performance

Writing many small chunks (like log entries or CSV rows) makes a syscall per write. Generating large output files with unbuffered writes is slow.

```rust
use std::fs::File;
use std::io::{self, BufWriter, Write};

//=============================================
// Buffered writing (essential for performance)
//=============================================
fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);  // 8 KB buffer by default

    for line in lines {
        writeln!(writer, "{}", line)?;  // Writes to buffer
    }

    writer.flush()?;  // Ensure all buffered data written
    Ok(())
}

//============================================
// Append to log file (preserves existing)
//============================================
fn append_log(path: &str, message: &str) -> io::Result<()> {
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    let mut writer = BufWriter::new(file);
    writeln!(writer, "{}", message)?;
    writer.flush()?;  // Critical for logs
    Ok(())
}
```

**Key Benefits**:
- Batches many writes into few syscalls
- 50-100x faster for bulk writes
- Automatic flush on drop (but explicit flush safer)
- Essential for generating large output files

## Pattern 3: Standard Streams

**Problem**: Programs need terminal I/O for user interaction. Shell pipelines break if diagnostics go to stdout instead of stderr. Interactive prompts don't appear without flushing. Multiple small writes to stdout are slow without locking.

**Solution**: Use io::stdin() for input, io::stdout() for data output, io::stderr() for errors/diagnostics. Call flush() after print!() before reading input. Use lock() for efficient bulk writes (acquires stream lock once). Separate normal output (stdout) from diagnostics (stderr) for pipeline compatibility.

**Why It Matters**: Correct stream separation enables Unix pipelines (program | grep). Flushing prevents UX bugs where prompts appear after input. Stream locking provides 50x speedup for bulk writes (10K lines: unlocked = 50 locks/s, locked = 1 lock). Without stderr separation, cannot redirect output while seeing errors.

**Use Cases**: CLI tools (interactive prompts, menus), Unix filters (cat|grep|wc), progress indicators (stderr while stdout pipes data), logging, command-line argument parsing.

###  Example: Interactive Prompts with stdin

Read user input with prompts. Without flushing, prompts don't appear before input.

```rust
use std::io::{self, Write};

//=================
// Read with prompt
//=================
fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;  // CRITICAL: flush before reading

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

//======================================
// Interactive menu
//======================================
fn interactive_menu() -> io::Result<()> {
    loop {
        println!("\n=== Menu ===");
        println!("1. Process data");
        println!("2. View stats");
        println!("3. Exit");

        let choice = prompt("Enter choice: ")?;

        match choice.as_str() {
            "1" => println!("Processing..."),
            "2" => println!("Stats: ..."),
            "3" => break,
            _ => eprintln!("Invalid choice"),  // Error to stderr!
        }
    }
    Ok(())
}
```

**Key Benefits**:
- Flush before reading prevents prompt bugs
- Use stdin.lock() for efficient multi-line reads
- EOF (Ctrl+D/Ctrl+Z) handled gracefully
- Errors go to stderr, output to stdout



## Pattern 4: Memory-Mapped I/O

**Problem**: Random access with read()+seek() is O(N) per access. Large files don't fit in RAM. Copying file contents to memory wastes CPU. Need zero-copy parsing of binary formats. Want shared memory between processes.

**Solution**: Use memmap2 crate to treat files as byte slices. OS handles paging data in/out. Mmap provides pointer-based access without explicit read/write. Hot pages accessed at memory speed. Works for both read-only and mutable mapping. Anonymous maps for IPC without files.

**Why It Matters**: Random access becomes memory-speed (hot pages). Databases need O(1) page access, not O(N) seek+read. Binary search in 1GB file: mmap enables true O(log N), buffered I/O can't match. Zero-copy means parsing structs directly from file bytes. Shared anonymous maps enable fast IPC.

**Use Cases**: Databases (page-based storage), binary search in large files, memory-mapped data structures, shared memory IPC, large read-only assets, sparse file access.

### Basic Memory Mapping

Rust doesn't include mmap in the standard library (it's unsafe and platform-specific). The `memmap2` crate provides a safe abstraction.

```rust
// Note: Add to Cargo.toml:
// [dependencies]
// memmap2 = "0.9"

//=============================================
// This is a conceptual example showing the API
//=============================================
#[cfg(feature = "memmap_example")]
mod memmap_examples {
    use memmap2::{Mmap, MmapMut, MmapOptions};
    use std::fs::File;
    use std::io::{self, Write};

    // Read-only memory map
    // Use this for: Large read-only data files, databases
    fn mmap_read(path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Access memory like a byte slice
        let data: &[u8] = &mmap[..];
        println!("First 10 bytes: {:?}", &data[..10.min(data.len())]);

        // mmap is unmapped when dropped
        Ok(())
    }

    // Mutable memory map
    // Use this for: In-place file modification, persistent data structures
    fn mmap_write(path: &str) -> io::Result<()> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Set file size before mapping
        file.set_len(1024)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        // Write data directly to memory (actually the file)
        mmap[0..5].copy_from_slice(b"Hello");

        // Flush to ensure writes reach disk
        mmap.flush()?;

        Ok(())
    }

    // Anonymous memory map (not backed by file)
    // Use this for: Shared memory IPC, large temporary buffers
    fn mmap_anonymous() -> io::Result<()> {
        let mut mmap = MmapMut::map_anon(1024)?;

        mmap[0..5].copy_from_slice(b"Hello");

        println!("Data: {:?}", &mmap[0..5]);

        Ok(())
    }

    // Large file processing with mmap
    // Use this when: File is too large for RAM but you need random access
    fn process_large_file_mmap(path: &str) -> io::Result<usize> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Count newlines efficiently (CPU-bound, not I/O-bound)
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

## Pattern 5: Directory Traversal

**Problem**: Need to walk file trees to find files, calculate sizes, or batch process. Simple recursion hits symlink loops. Need to filter by extension or pattern. Want to skip hidden files or match glob patterns.

**Solution**: Use fs::read_dir() for single-level listing. Implement recursive walk with visited path tracking (or use walkdir crate). Filter by extension with path.extension(). Track inodes (Unix) to detect symlink cycles. Sort entries for deterministic ordering.

**Why It Matters**: Build systems scan thousands of files to find sources. Backup tools walk entire disks. Wrong implementation hits symlink cycles and recurses forever. Efficient traversal is O(N) in file count; naive approaches degrade to O(N²) from repeated stats.

**Use Cases**: Build systems (find .rs files), file search tools (find by name/pattern), disk usage analyzers, backup tools, batch file operations (chmod/chown recursively).

### Example: Basic Directory Operations

```rust
use std::fs;
use std::io;
use std::path::Path;

//=================
// Create directory
//=================
fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir(path)
    // Fails if parent doesn't exist
    // Fails if directory already exists
}

//============================================
// Create directory and all parent directories
//============================================
// Like mkdir -p in Unix
fn create_directory_all(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
    // Creates parent directories as needed
    // Succeeds if directory already exists
}

//=======================
// Remove empty directory
//=======================
fn remove_directory(path: &str) -> io::Result<()> {
    fs::remove_dir(path)
    // Fails if directory is not empty
}

//===============================================
// Remove directory and all contents (dangerous!)
//===============================================
fn remove_directory_all(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
    // Recursively deletes everything
    // Like rm -rf in Unix
}

//=====================
// Check if path exists
//=====================
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
    // Returns false for broken symlinks
}

//===========================
// Check if path is directory
//===========================
fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
    // Follows symlinks
}
```

**Safety note**: `remove_dir_all` is dangerous. It's equivalent to `rm -rf`. There's no trash bin, no undo. Many programs ask for confirmation before using this.

### Example: Reading Directory Contents

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

### Example: Recursive Directory Traversal

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




## Pattern 6: Process Spawning and Piping

**Problem**: Need to run external commands and capture output. Want to chain processes like Unix pipelines. Reading both stdout and stderr can deadlock. Long-running processes need streaming output. Need to pass environment variables and set working directory.

**Solution**: Use Command::new() with .output() (captures all), .status() (inherits streams), or .spawn() (returns immediately). Set Stdio::piped() to capture output. Use threads to read stdout/stderr concurrently (avoids deadlock). Chain processes by passing child.stdout to next child.stdin. Set .env() and .current_dir() as needed.

**Why It Matters**: Integration with system tools essential for build scripts, testing, automation. Improper stream handling causes deadlocks (child blocks on full pipe, parent blocks reading). Pipelines enable Unix philosophy (compose programs). Environment and working directory control enable sandboxing.

**Use Cases**: Build scripts (invoke compilers), test runners (execute programs and check output), automation tools, implementing Unix pipelines (cat|grep|wc), subprocess orchestration.


### Example: Basic Process Execution

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

### Example: Streaming Output

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

    // Read stdout in one thread
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[OUT] {}", line);
            }
        }
    });

    // Read stderr in another thread
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

### Example: Piping Between Processes

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


## Summary

This chapter covered synchronous I/O patterns:

1. **Buffered Reading/Writing**: BufReader/BufWriter for 1000x speedup via syscall amortization
2. **Standard Streams**: stdin/stdout/stderr separation, locking, flushing for correct CLI behavior
3. **Memory-Mapped I/O**: Zero-copy random access, database-style page reads
4. **Directory Traversal**: Recursive walking, filtering by pattern, avoiding symlink cycles
5. **Process Spawning**: Running commands, capturing output, Unix pipelines

**Key Takeaways**:
- Always buffer I/O—unbuffered is 1000x slower
- Separate stdout (data) from stderr (diagnostics)
- Use mmap for random access, buffered I/O for sequential
- Track inodes to prevent infinite recursion in directory walks
- Use threads when reading multiple process streams

**Performance Guidelines**:
- Buffering: O(N) syscalls → O(N/8192) syscalls
- Stream locking: 50x speedup for bulk writes
- Mmap: O(1) random access vs O(N) seek+read
- Directory walk: O(N) with proper caching

## File IO Foundations
```rust
use std::fs;
use std::io::{self, Read, Write, BufRead, BufReader, BufWriter, Seek, SeekFrom};
use std::path::Path;

// File opening
fs::File::open("file.txt")                          // Open for reading, returns Result<File>
fs::File::create("file.txt")                        // Create/truncate for writing
fs::OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .append(true)
    .truncate(false)
    .open("file.txt")                                // Custom open options

// Quick read operations
fs::read("file.txt")                                 // Read entire file to Vec<u8>
fs::read_to_string("file.txt")                      // Read entire file to String
fs::read_to_string("file.txt")?                     // With error propagation

// Quick write operations
fs::write("file.txt", b"data")                      // Write bytes to file
fs::write("file.txt", "text")                       // Write string to file

// Reading from File
let mut file = fs::File::open("file.txt")?;
let mut buffer = Vec::new();
file.read_to_end(&mut buffer)?                      // Read all bytes
let mut buffer = String::new();
file.read_to_string(&mut buffer)?                   // Read as string
let mut buffer = [0u8; 1024];
let n = file.read(&mut buffer)?                     // Read up to buffer size
file.read_exact(&mut buffer)?                       // Read exact amount or error

// Writing to File
let mut file = fs::File::create("file.txt")?;
file.write(b"data")?                                // Write bytes, returns bytes written
file.write_all(b"data")?                            // Write all bytes or error
file.write_fmt(format_args!("Hello {}", name))?    // Formatted write
write!(file, "Hello {}", name)?                     // Macro for formatted write
writeln!(file, "Hello {}", name)?                   // With newline

// Buffered reading (efficient for line-by-line)
let file = fs::File::open("file.txt")?;
let reader = BufReader::new(file);
let reader = BufReader::with_capacity(size, file);  // Custom buffer size

for line in reader.lines() {                         // Iterate lines
    let line = line?;
    println!("{}", line);
}

reader.read_line(&mut string)?                      // Read one line
reader.read_until(b'\n', &mut buffer)?              // Read until delimiter
reader.split(b'\n')                                 // Split into chunks by byte
reader.lines()                                       // Iterator over lines

// Buffered writing (efficient for many small writes)
let file = fs::File::create("file.txt")?;
let mut writer = BufWriter::new(file);
let mut writer = BufWriter::with_capacity(size, file); // Custom buffer size

writer.write_all(b"data")?                          // Buffered write
writer.flush()?                                      // Flush buffer to disk
drop(writer)                                         // Auto-flush on drop

// Seeking (random access)
file.seek(SeekFrom::Start(0))?                      // Seek to byte position from start
file.seek(SeekFrom::End(-10))?                      // Seek from end
file.seek(SeekFrom::Current(5))?                    // Seek relative to current position
let pos = file.stream_position()?                   // Get current position
file.rewind()?                                       // Seek to start (shorthand)

// Metadata operations
let metadata = fs::metadata("file.txt")?            // Get file metadata
let metadata = file.metadata()?                     // From file handle
metadata.len()                                       // File size in bytes
metadata.is_file()                                   // Check if regular file
metadata.is_dir()                                    // Check if directory
metadata.is_symlink()                               // Check if symbolic link
metadata.permissions()                               // Get permissions
metadata.modified()?                                 // Last modified time
metadata.accessed()?                                 // Last accessed time
metadata.created()?                                  // Creation time

// Permissions
let perms = metadata.permissions();
perms.readonly()                                     // Check if read-only
let mut perms = metadata.permissions();
perms.set_readonly(true);                           // Set read-only
fs::set_permissions("file.txt", perms)?             // Apply permissions

// File operations
fs::copy("src.txt", "dst.txt")?                     // Copy file, returns bytes copied
fs::rename("old.txt", "new.txt")?                   // Rename/move file
fs::remove_file("file.txt")?                        // Delete file
fs::hard_link("original.txt", "link.txt")?         // Create hard link
fs::soft_link("original.txt", "link.txt")?         // Create symbolic link (Unix)
fs::read_link("link.txt")?                          // Read symlink target
fs::canonicalize("./file.txt")?                     // Get absolute path

// Directory operations
fs::create_dir("dir")?                              // Create single directory
fs::create_dir_all("path/to/dir")?                 // Create directory and parents
fs::remove_dir("dir")?                              // Remove empty directory
fs::remove_dir_all("dir")?                          // Remove directory and contents
fs::read_dir("dir")?                                // Get iterator over directory entries

for entry in fs::read_dir("dir")? {
    let entry = entry?;
    println!("{:?}", entry.path());                 // Get full path
    println!("{:?}", entry.file_name());            // Get file name
    let metadata = entry.metadata()?;                // Get metadata
    let file_type = entry.file_type()?;             // Get file type
}

// Path operations
use std::path::{Path, PathBuf};
let path = Path::new("file.txt");
path.exists()                                        // Check if exists
path.is_file()                                       // Check if file
path.is_dir()                                        // Check if directory
path.is_symlink()                                    // Check if symlink
path.is_absolute()                                   // Check if absolute path
path.is_relative()                                   // Check if relative path
path.file_name()                                     // Get file name as OsStr
path.extension()                                     // Get extension
path.file_stem()                                     // Get name without extension
path.parent()                                        // Get parent directory
path.ancestors()                                     // Iterator over ancestor directories
path.components()                                    // Iterator over path components
path.to_str()                                        // Convert to &str (Option)
path.to_string_lossy()                              // Convert to Cow<str>

let mut pathbuf = PathBuf::new();
pathbuf.push("dir");                                // Append to path
pathbuf.push("file.txt");
pathbuf.pop()                                        // Remove last component
pathbuf.set_file_name("new.txt")                   // Change file name
pathbuf.set_extension("md")                         // Change extension
let joined = path.join("subdir/file.txt");          // Join paths

// File synchronization
file.sync_all()?                                     // Sync data and metadata to disk
file.sync_data()?                                    // Sync only data to disk

// File truncation
file.set_len(100)?                                   // Set file size (truncate/extend)

// File handle duplication
let file2 = file.try_clone()?                       // Clone file handle

// Temporary files (requires tempfile crate)
use tempfile::{NamedTempFile, TempDir};
let temp_file = NamedTempFile::new()?               // Create temp file
let temp_dir = TempDir::new()?                      // Create temp directory

// Memory-mapped files (requires memmap2 crate)
use memmap2::MmapOptions;
let file = fs::File::open("file.txt")?;
let mmap = unsafe { MmapOptions::new().map(&file)? }; // Memory map (read)
let data = &mmap[..];                                // Access as slice

// Common patterns
// Read file line by line
let file = fs::File::open("file.txt")?;
let reader = BufReader::new(file);
for line in reader.lines() {
    let line = line?;
    // process line
}

// Write multiple lines
let file = fs::File::create("file.txt")?;
let mut writer = BufWriter::new(file);
for item in items {
    writeln!(writer, "{}", item)?;
}
writer.flush()?;

// Copy with progress
let mut src = fs::File::open("src.txt")?;
let mut dst = fs::File::create("dst.txt")?;
let mut buffer = [0u8; 8192];
loop {
    let n = src.read(&mut buffer)?;
    if n == 0 { break; }
    dst.write_all(&buffer[..n])?;
}

// Append to file
let mut file = fs::OpenOptions::new()
    .append(true)
    .create(true)
    .open("log.txt")?;
writeln!(file, "Log entry")?;

// Read file in chunks
let file = fs::File::open("large.txt")?;
let mut reader = BufReader::new(file);
let mut buffer = [0u8; 4096];
loop {
    let n = reader.read(&mut buffer)?;
    if n == 0 { break; }
    // process buffer[..n]
}

// Check if file exists before opening
if Path::new("file.txt").exists() {
    let content = fs::read_to_string("file.txt")?;
}

// Safe file writing (write to temp, then rename)
let temp = "file.txt.tmp";
fs::write(temp, data)?;
fs::rename(temp, "file.txt")?;

// Recursive directory traversal
fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path)?;
            } else {
                println!("{:?}", path);
            }
        }
    }
    Ok(())
}

// Read CSV-like file
let file = fs::File::open("data.csv")?;
let reader = BufReader::new(file);
for line in reader.lines() {
    let line = line?;
    let fields: Vec<&str> = line.split(',').collect();
    // process fields
}

// Binary file reading
let mut file = fs::File::open("data.bin")?;
let mut buffer = [0u8; 4];
file.read_exact(&mut buffer)?;
let value = u32::from_le_bytes(buffer);             // Parse little-endian u32

// Binary file writing
let mut file = fs::File::create("data.bin")?;
file.write_all(&42u32.to_le_bytes())?;             // Write little-endian u32
```


## Command Foundations

```rust
use std::process::{Command, Stdio, Child, ExitStatus, Output};
use std::io::{Write, BufRead, BufReader};

// Basic command execution
Command::new("ls")                                   // Create new command
    .spawn()?                                        // Spawn child process, returns Child
Command::new("ls").output()?                        // Run and capture output, returns Output
Command::new("ls").status()?                        // Run and get exit status

// Command arguments
Command::new("echo")
    .arg("hello")                                    // Add single argument
    .args(&["hello", "world"])                      // Add multiple arguments
    .args(vec!["a", "b", "c"])                      // From vec
    .spawn()?

// Environment variables
Command::new("program")
    .env("KEY", "value")                            // Set single env var
    .envs(vec![("K1", "V1"), ("K2", "V2")])        // Set multiple env vars
    .env_remove("PATH")                             // Remove env var
    .env_clear()                                    // Clear all env vars
    .spawn()?

// Working directory
Command::new("ls")
    .current_dir("/tmp")                            // Set working directory
    .spawn()?

// Standard I/O configuration
Command::new("program")
    .stdin(Stdio::null())                           // No stdin
    .stdin(Stdio::inherit())                        // Inherit from parent
    .stdin(Stdio::piped())                          // Create pipe
    .stdout(Stdio::null())                          // No stdout
    .stdout(Stdio::inherit())                       // Inherit from parent
    .stdout(Stdio::piped())                         // Create pipe
    .stderr(Stdio::null())                          // No stderr
    .stderr(Stdio::inherit())                       // Inherit from parent
    .stderr(Stdio::piped())                         // Create pipe
    .spawn()?

// Child process methods
let mut child = Command::new("sleep").arg("5").spawn()?;
child.id()                                          // Get process ID
child.kill()?                                       // Kill process
child.wait()?                                       // Wait for completion, returns ExitStatus
child.try_wait()?                                   // Non-blocking wait, returns Option<ExitStatus>
child.wait_with_output()?                          // Wait and capture output

// Accessing child stdio
let mut child = Command::new("cat")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

let stdin = child.stdin.as_mut().unwrap();          // Get stdin handle
stdin.write_all(b"data")?;                          // Write to child's stdin
drop(stdin)                                          // Close stdin (important!)

let stdout = child.stdout.take().unwrap();          // Take ownership of stdout
let reader = BufReader::new(stdout);
for line in reader.lines() {
    println!("{}", line?);
}

child.wait()?;

// Output struct methods
let output = Command::new("echo").arg("hello").output()?;
output.status                                        // ExitStatus
output.stdout                                        // Vec<u8>
output.stderr                                        // Vec<u8>
String::from_utf8_lossy(&output.stdout)             // Convert to string

// ExitStatus methods
let status = Command::new("ls").status()?;
status.success()                                     // Check if exit code 0
status.code()                                        // Get exit code as Option<i32>
status.exit_ok()?                                    // Return Ok(()) if success, Err otherwise

// Unix-specific (ExitStatusExt trait)
#[cfg(unix)]
{
    use std::os::unix::process::ExitStatusExt;
    status.signal()                                  // Get signal if terminated by signal
    status.core_dumped()                            // Check if core dumped
    status.stopped_signal()                         // Get stop signal
    status.continued()                              // Check if continued
}

// Running shell commands
#[cfg(unix)]
Command::new("sh")
    .arg("-c")
    .arg("ls | grep txt")
    .output()?

#[cfg(windows)]
Command::new("cmd")
    .args(&["/C", "dir"])
    .output()?

// Piping between processes
let process1 = Command::new("ls")
    .stdout(Stdio::piped())
    .spawn()?;

let process2 = Command::new("grep")
    .arg("txt")
    .stdin(process1.stdout.unwrap())
    .stdout(Stdio::piped())
    .spawn()?;

let output = process2.wait_with_output()?;

// Common patterns
// Capture output as string
let output = Command::new("echo")
    .arg("hello")
    .output()?;
let stdout = String::from_utf8_lossy(&output.stdout);
let stderr = String::from_utf8_lossy(&output.stderr);

// Check if command succeeded
let status = Command::new("ls").status()?;
if status.success() {
    println!("Command succeeded");
} else {
    eprintln!("Command failed with: {}", status);
}

// Write to stdin and read from stdout
let mut child = Command::new("cat")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

{
    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(b"Hello from Rust\n")?;
}

let output = child.wait_with_output()?;
println!("{}", String::from_utf8_lossy(&output.stdout));

// Stream output line by line
let mut child = Command::new("ping")
    .arg("localhost")
    .stdout(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let reader = BufReader::new(stdout);

for line in reader.lines() {
    println!("Output: {}", line?);
}

// Run with timeout (requires external crate or manual implementation)
use std::time::Duration;
use std::thread;

let mut child = Command::new("sleep").arg("10").spawn()?;
let timeout = Duration::from_secs(2);

thread::sleep(timeout);
match child.try_wait()? {
    Some(status) => println!("Exited with: {}", status),
    None => {
        child.kill()?;
        println!("Killed after timeout");
    }
}

// Execute and get exit code
let code = Command::new("false")
    .status()?
    .code()
    .unwrap_or(-1);
println!("Exit code: {}", code);

// Redirect stderr to stdout
let output = Command::new("ls")
    .arg("/nonexistent")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()?;

// Builder pattern for complex commands
let output = Command::new("gcc")
    .args(&["-o", "program", "main.c"])
    .current_dir("/tmp")
    .env("CC", "clang")
    .output()?;

// Spawning multiple processes
let mut children = vec![];
for i in 0..5 {
    let child = Command::new("echo")
        .arg(format!("Process {}", i))
        .spawn()?;
    children.push(child);
}

for mut child in children {
    child.wait()?;
}

// Conditional execution based on OS
#[cfg(target_os = "linux")]
let output = Command::new("ps").arg("aux").output()?;

#[cfg(target_os = "windows")]
let output = Command::new("tasklist").output()?;

#[cfg(target_os = "macos")]
let output = Command::new("ps").arg("aux").output()?;

// Execute script file
#[cfg(unix)]
Command::new("bash")
    .arg("script.sh")
    .spawn()?;

#[cfg(windows)]
Command::new("cmd")
    .args(&["/C", "script.bat"])
    .spawn()?;

// Detached process (Unix)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    Command::new("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
}

// Set process group (Unix)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    Command::new("program")
        .process_group(0)                            // Create new process group
        .spawn()?;
}

// Set user/group (Unix, requires privileges)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    Command::new("program")
        .uid(1000)                                   // Set user ID
        .gid(1000)                                   // Set group ID
        .spawn()?;
}

// Execute and replace current process (Unix only)
#[cfg(unix)]
{
    use std::os::unix::process::CommandExt;
    let error = Command::new("ls").exec();          // Never returns on success
    eprintln!("Failed to exec: {}", error);
}

// Real-time output streaming with both stdout and stderr
let mut child = Command::new("program")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let stdout = child.stdout.take().unwrap();
let stderr = child.stderr.take().unwrap();

let stdout_thread = thread::spawn(move || {
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        println!("STDOUT: {}", line.unwrap());
    }
});

let stderr_thread = thread::spawn(move || {
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        eprintln!("STDERR: {}", line.unwrap());
    }
});

stdout_thread.join().unwrap();
stderr_thread.join().unwrap();
child.wait()?;

// Check if command exists
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// Run command and ignore errors
let _ = Command::new("optional_tool").status();

// Chain commands with AND logic
let status1 = Command::new("cmd1").status()?;
if status1.success() {
    let status2 = Command::new("cmd2").status()?;
}
```