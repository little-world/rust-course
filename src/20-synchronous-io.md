# Synchronous I/O
This chapter covers Rust's synchronous I/O patterns—blocking operations that pause threads until complete. Unlike async I/O (Chapter 17), synchronous I/O is simpler, easier to debug, and sufficient for most programs. CLI tools, build scripts, and even high-performance servers with thread pools rely on these patterns.


## Pattern 1: Basic File Operations

**Problem**: Files are the main way programs persist data—configuration, logs, cached results, user documents—but handling them correctly can be tricky. You need to decide how to read the file efficiently, handle different formats, and deal with errors safely.

**Solution**: Do you read the entire file into memory (`fs::read_to_string`) for simplicity, or process it line by line (`BufReader::lines`) to handle files larger than RAM? The right choice depends on your constraints.

**Why It Matters**: Choosing the wrong approach can cause: Memory exhaustion if a large file is loaded at once. Slow performance if line-by-line reading is overused for small files.

**Use Cases**: Reading  JSON file, Processing logs or CSV files , CLI tools or servers that need predictable file I/O behavior.

###  Example: Basic file reading
The simplest case: read a small file entirely into memory.

```rust
use std::fs::File;
use std::io::{self, Read};

```

### Example: Read entire file to string (UTF-8)
This example walks through how to read entire file to string (utf-8).

```rust
fn read_to_string(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
    // Allocates a String big enough for the entire file
    // Returns Err if file doesn't exist, isn't readable, or isn't valid UTF-8
}

```

### Example: Read entire file to bytes (binary)
This example walks through how to read entire file to bytes (binary).

```rust
fn read_to_bytes(path: &str) -> io::Result<Vec<u8>> {
    std::fs::read(path)
    // Allocates a Vec<u8> and reads all bytes
    // Returns Err if file doesn't exist or isn't readable
}

```

### Example: Manual reading with buffer control
This example walks through manual reading with buffer control.

```rust
fn read_with_buffer(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    // read_to_string reads until EOF, automatically resizing the String
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

```

### Example: Read exact number of bytes
This example walks through how to read exact number of bytes.

```rust
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

```

### Example: Write string to file (overwrites existing)
This example walks through how to write string to file (overwrites existing).

```rust
fn write_string(path: &str, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
    // Creates file if it doesn't exist
    // Truncates (erases) existing content
    // Writes all content in one operation
}

```

### Example: Write bytes to file
This example walks through how to write bytes to file.

```rust
fn write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    std::fs::write(path, content)
}

```

### Example: Manual writing with file handle
This example walks through manual writing with file handle.

```rust
fn write_with_handle(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    // write_all ensures all bytes are written or returns Err
    // Partial writes are retried automatically
    file.write_all(content.as_bytes())?;
    Ok(())
}

```

### Example: Append to file (preserves existing content)
This example walks through how to append to file (preserves existing content).

```rust
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

**Problem**: Reading or writing files byte-by-byte makes a system call per byte—catastrophically slow with O(N) syscalls for N bytes. Processing a 100 MB file unbuffered can take minutes.

**Solution**: Wrap File handles in BufReader/BufWriter which maintain internal buffers (default 8 KB). BufReader amortizes reads: fills buffer with one syscall, serves subsequent reads from memory.

**Why It Matters**: Buffering provides 1000x speedup—unbuffered 100 MB file takes minutes, buffered takes milliseconds. Syscall overhead dominates unbuffered I/O.

**Use Cases**: Log file parsing (line-by-line), CSV processing (buffered reading), config file loading, generating reports (buffered writing), any text-oriented file I/O, binary protocol parsing with custom delimiters.

###  Example: Buffered Line-by-Line Reading

Process large log files or text data that doesn't fit in memory. Need memory-efficient streaming. Want to skip comments or filter lines.

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

```

### Example: Process large files line by line (memory-efficient)
This example walks through process large files line by line (memory-efficient).

```rust
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

```

### Example: Filter lines (e.g., only errors)
This example walks through how to filter lines (e.g., only errors).

```rust
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

```

### Example: Buffered writing (essential for performance)
This example walks through buffered writing (essential for performance).

```rust
fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);  // 8 KB buffer by default

    for line in lines {
        writeln!(writer, "{}", line)?;  // Writes to buffer
    }

    writer.flush()?;  // Ensure all buffered data written
    Ok(())
}

```

### Example: Append to log file (preserves existing)
This example walks through how to append to log file (preserves existing).

```rust
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

**Problem**: Programs need terminal I/O for user interaction. Shell pipelines break if diagnostics go to stdout instead of stderr.

**Solution**: Use io::stdin() for input, io::stdout() for data output, io::stderr() for errors/diagnostics. Call flush() after print!() before reading input.

**Why It Matters**: Correct stream separation enables Unix pipelines (program | grep). Flushing prevents UX bugs where prompts appear after input.

**Use Cases**: CLI tools (interactive prompts, menus), Unix filters (cat|grep|wc), progress indicators (stderr while stdout pipes data), logging, command-line argument parsing.

###  Example: Interactive Prompts with stdin

Read user input with prompts. Without flushing, prompts don't appear before input.

```rust
use std::io::{self, Write};

```

### Example: Read with prompt
This example walks through how to read with prompt.

```rust
fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;  // CRITICAL: flush before reading

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

```

### Example: Interactive menu
This example walks through interactive menu.

```rust
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

**Problem**: Random access with read()+seek() is O(N) per access. Large files don't fit in RAM.

**Solution**: Use memmap2 crate to treat files as byte slices. OS handles paging data in/out.

**Why It Matters**: Random access becomes memory-speed (hot pages). Databases need O(1) page access, not O(N) seek+read.

**Use Cases**: Databases (page-based storage), binary search in large files, memory-mapped data structures, shared memory IPC, large read-only assets, sparse file access.

### Basic Memory Mapping

Rust doesn't include mmap in the standard library (it's unsafe and platform-specific). The `memmap2` crate provides a safe abstraction.

```rust
// Note: Add to Cargo.toml:
// [dependencies]
// memmap2 = "0.9"

```

### Example: This is a conceptual example showing the API
This example walks through this is a conceptual example showing the api.

```rust
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

**Problem**: Need to walk file trees to find files, calculate sizes, or batch process. Simple recursion hits symlink loops.

**Solution**: Use fs::read_dir() for single-level listing. Implement recursive walk with visited path tracking (or use walkdir crate).

**Why It Matters**: Build systems scan thousands of files to find sources. Backup tools walk entire disks.

**Use Cases**: Build systems (find .rs files), file search tools (find by name/pattern), disk usage analyzers, backup tools, batch file operations (chmod/chown recursively).

### Example: Basic Directory Operations

```rust
use std::fs;
use std::io;
use std::path::Path;

```

### Example: Create directory
This example walks through how to create directory.

```rust
fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir(path)
    // Fails if parent doesn't exist
    // Fails if directory already exists
}

```

### Example: Create directory and all parent directories
This example walks through how to create directory and all parent directories.

```rust
// Like mkdir -p in Unix
fn create_directory_all(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
    // Creates parent directories as needed
    // Succeeds if directory already exists
}

```

### Example: Remove empty directory
This example walks through remove empty directory.

```rust
fn remove_directory(path: &str) -> io::Result<()> {
    fs::remove_dir(path)
    // Fails if directory is not empty
}

```

### Example: Remove directory and all contents (dangerous!)
This example walks through remove directory and all contents (dangerous!).

```rust
fn remove_directory_all(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
    // Recursively deletes everything
    // Like rm -rf in Unix
}

```

### Example: Check if path exists
This example walks through check if path exists.

```rust
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
    // Returns false for broken symlinks
}

```

### Example: Check if path is directory
This example walks through check if path is directory.

```rust
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

```

### Example: List directory contents
This example walks through list directory contents.

```rust
fn list_directory(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;  // Each entry can fail
        entries.push(entry.path());
    }

    Ok(entries)
}

```

### Example: List only files (skip directories)
This example walks through list only files (skip directories).

```rust
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

```

### Example: List files with specific extension
This example walks through list files with specific extension.

```rust
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

```

### Example: Get directory entries with metadata
This example walks through get directory entries with metadata.

```rust
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

```

### Example: Recursive file listing
This example walks through recursive file listing.

```rust
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

```

### Example: Get all files recursively
This example walks through get all files recursively.

```rust
fn get_all_files(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory(Path::new(path), &mut files)?;
    Ok(files)
}

```

### Example: Recursive directory tree printer (visual tree)
This example walks through recursive directory tree printer (visual tree).

```rust
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

```

### Example: Find files matching pattern (like find command)
This example walks through find files matching pattern (like find command).

```rust
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

**Problem**: Need to run external commands and capture output. Want to chain processes like Unix pipelines.

**Solution**: Use Command::new() with .output() (captures all), .status() (inherits streams), or .spawn() (returns immediately). Set Stdio::piped() to capture output.

**Why It Matters**: Integration with system tools essential for build scripts, testing, automation. Improper stream handling causes deadlocks (child blocks on full pipe, parent blocks reading).

**Use Cases**: Build scripts (invoke compilers), test runners (execute programs and check output), automation tools, implementing Unix pipelines (cat|grep|wc), subprocess orchestration.


### Example: Basic Process Execution

```rust
use std::process::{Command, Stdio};
use std::io::{self, Write};

```

### Example: Run command and capture output
This example walks through run command and capture output.

```rust
fn run_command() -> io::Result<()> {
    let output = Command::new("ls")
        .arg("-la")
        .output()?;  // Waits for completion, captures all output

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

```

### Example: Check if command succeeded
This example walks through check if command succeeded.

```rust
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

```

### Example: Run with environment variables
This example walks through run with environment variables.

```rust
fn run_with_env() -> io::Result<()> {
    let output = Command::new("printenv")
        .env("MY_VAR", "my_value")
        .env("ANOTHER_VAR", "another_value")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

```

### Example: Run in specific directory
This example walks through run in specific directory.

```rust
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

```

### Example: Stream stdout in real-time
This example walks through stream stdout in real-time.

```rust
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

```

### Example: Capture both stdout and stderr separately
This example walks through capture both stdout and stderr separately.

```rust
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

```

### Example: Pipe output from one command to another (ls | grep txt)
This example walks through pipe output from one command to another (ls | grep txt).

```rust
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

```

### Example: Complex pipeline: cat file | grep pattern | wc -l
This example walks through complex pipeline: cat file | grep pattern | wc -l.

```rust
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


### Summary

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
