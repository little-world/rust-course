# Synchronous I/O
This chapter covers Rust's synchronous I/O patterns—blocking operations that pause threads until complete. Unlike async I/O (the next chapter), synchronous I/O is simpler, easier to debug, and sufficient for most programs. CLI tools, build scripts, and even high-performance servers with thread pools rely on these patterns.


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

One function call that opens, reads, and closes—pre-allocates based on file size to avoid reallocations.
Returns an error if the file doesn't exist, lacks read permission, or contains invalid UTF-8. Best for config files and data under ~10 MB.

```rust
fn read_to_string(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
    // Allocates a String big enough for the entire file
    // Err if missing, unreadable, or invalid UTF-8
}

// Usage: Load configuration file
let content = read_to_string("config.json")?;
```

### Example: Read entire file to bytes (binary)

Use `fs::read()` for binary files where UTF-8 validation would fail—returns raw bytes for parsers or image libraries.
Pre-allocates based on file metadata like `read_to_string()`. Essential for images, executables, and compressed archives.

```rust
fn read_to_bytes(path: &str) -> io::Result<Vec<u8>> {
    std::fs::read(path)
    // Allocates a Vec<u8> and reads all bytes
    // Returns Err if file doesn't exist or isn't readable
}

// Usage: Load image for processing
let bytes = read_to_bytes("image.png")?;
```

### Example: Manual reading with buffer control

Use when you need the `File` handle for metadata, seeking, or multiple operations after reading.
The `read_to_string()` method reads until EOF, resizing the String as needed while keeping the file open.

```rust
fn read_with_buffer(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();

    // Reads until EOF, auto-resizing the String
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Usage: Read file while keeping handle for metadata
let data = read_with_buffer("data.txt")?;
```

### Example: Read exact number of bytes

For fixed-size headers (PNG magic bytes, ZIP headers), `read_exact()` guarantees exactly N bytes or an error.
Safer than `read()` which might return fewer bytes—critical for binary protocols where partial reads mean corruption.

```rust
fn read_exact_bytes(path: &str, n: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; n];

    // Err if fewer than n bytes available
    // Guarantees all n bytes or error—no partial reads
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

// Usage: Read fixed-size file header
let header = read_exact_bytes("file.bin", 128)?;
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

One function call that creates, truncates, writes, and closes—**erases existing content** before writing.
Atomic at syscall level (all bytes or error). Use for output files and state where you want complete replacement.

```rust
fn write_string(path: &str, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
    // Creates file if it doesn't exist
    // Truncates (erases) existing content
    // Writes all content in one operation
}

// Usage: Save configuration
write_string("output.txt", "Hello, world!")?;
```

### Example: Write bytes to file

Accepts `&[u8]}` for binary files—images, serialized data, protocol buffers, or raw dumps.
Generic over `AsRef<[u8]>` (accepts `&[u8]`, `Vec<u8>`, `String`, `&str`). Overwrites existing files completely.

```rust
fn write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    std::fs::write(path, content)
}

// Usage: Write PNG magic bytes
write_bytes("data.bin", &[0x89, 0x50, 0x4E, 0x47])?;
```

### Example: Manual writing with file handle

Use when you need the handle for multiple writes, seeking, or explicit `flush()` to disk.
`write_all()` loops until all bytes are written (unlike `write()` which might write fewer). Guarantees complete writes or error.

```rust
fn write_with_handle(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;

    // write_all ensures all bytes are written or returns Err
    // Partial writes are retried automatically
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Usage: Write with explicit file handle
write_with_handle("output.txt", "data")?;
```

### Example: Append to file (preserves existing content)

Essential for logs: `.append(true)` seeks to end before every write, `.create(true)` handles missing files.
Multiple processes can safely append (atomic up to ~4KB). The `writeln!` macro adds newlines automatically.

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

// Usage: Append log entry
append_to_file("log.txt", "New entry")?;
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
        .create_new(true)   // Fail if exists (atomic)
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
            .mode(0o644)      // rw-r--r--
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

Stream gigabyte files without loading into memory—`BufReader` maintains an 8KB buffer, serving one line at a time.
Memory usage is O(max line length), not O(file size). Each `line?` handles I/O errors individually.

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

// Usage: Process large log file efficiently
process_large_file("access.log")?;
```

### Example: Filter lines (e.g., only errors)

Combine iterator adapters with buffered reading—`filter_map(|line| line.ok())` skips I/O errors silently.
Chain filters for complex queries (search, exclude, transform). Collects matches into a `Vec`.

```rust
fn process_errors_only(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(reader.lines()
        .filter_map(|line| line.ok())  // Skip I/O errors
        .filter(|line| line.contains("ERROR"))
        .collect())
}

// Usage: Extract all error lines from log
let errors = process_errors_only("app.log")?;
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

`BufWriter` accumulates writes in an 8KB buffer, transforming O(N) syscalls into O(N/8192)—50-100x speedup.
Always call `flush()` at the end to ensure the final partial buffer reaches disk before closing.

```rust
fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);  // 8 KB default buf

    for line in lines {
        writeln!(writer, "{}", line)?;  // Writes to buffer
    }

    writer.flush()?;  // Ensure all buffered data written
    Ok(())
}

// Usage: Write many lines efficiently
buffered_write("output.txt", &["line1", "line2"])?;
```

### Example: Append to log file (preserves existing)

Combines append mode with buffering for efficient log writes. The explicit `flush()` is critical—without it, a crash loses buffered entries.
This is the foundation of application logging: append-only, buffered, crash-aware.

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

// Usage: Append log entry with flush
append_log("app.log", "Server started")?;
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



### Example: Read with prompt

Classic interactive pattern: `print!` keeps cursor on same line, `flush()` is critical since `print!` is line-buffered.
Without flush, the prompt might not appear before `read_line` blocks. Use `trim()` to remove the trailing newline.

```rust
fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?;  // CRITICAL: flush before reading

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// Usage: Interactive prompt
let name = prompt("Enter your name: ")?;
```

### Example: Interactive menu

Build CLIs with a loop: display options, read input, dispatch actions. Use `println!` for output, `eprintln!` for errors.
The loop handles invalid input gracefully. This pattern is the basis of REPLs and configuration wizards.

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

// Usage: Run interactive menu loop
interactive_menu()?;
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


### Example: Memory-Mapped File Operations

Memory mapping treats files as byte arrays—OS pages data in/out automatically. Provides O(1) random access vs O(N) seek+read.
The `unsafe` is required because external processes could modify the mapped file, invalidating Rust's guarantees.

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
        let n = 10.min(data.len());
        println!("First {} bytes: {:?}", n, &data[..n]);

        // mmap is unmapped when dropped
        Ok(())
    }

    // Mutable memory map
    // In-place file modification, persistent data structures
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
    // For files too large for RAM requiring random access
    fn process_large_file_mmap(path: &str) -> io::Result<usize> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Count newlines efficiently (CPU-bound, not I/O-bound)
        let count = mmap.iter().filter(|&&b| b == b'\n').count();

        Ok(count)
    }

    // Usage: Count lines via memory-mapped file
    let count = process_large_file_mmap("huge.log")?;
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


### Example: Create directory

Creates a single directory—parent must exist. Fails if directory already exists (unlike `create_dir_all`).
Use when you expect the parent to exist and want to detect re-creation. For idempotent scripts, prefer `create_dir_all`.

```rust
fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir(path)
    // Fails if parent doesn't exist
    // Fails if directory already exists
}

// Usage: Create single directory
create_directory("new_folder")?;
```

### Example: Create directory and all parent directories

Like `mkdir -p`—creates entire path including missing parents. Idempotent: succeeds if directory exists.
Essential for setup scripts and build systems. Race-condition safe with concurrent creation.

```rust
// Like mkdir -p in Unix
fn create_directory_all(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
    // Creates parent directories as needed
    // Succeeds if directory already exists
}

// Usage: Create nested directory structure
create_directory_all("a/b/c")?;
```

### Example: Remove empty directory

Removes directory only if empty—fails if files or subdirectories remain. Safe default preventing accidental deletion.
Common pattern: delete files first, then `remove_dir` on empty parent. Prevents catastrophic data loss.

```rust
fn remove_directory(path: &str) -> io::Result<()> {
    fs::remove_dir(path)
    // Fails if directory is not empty
}

// Usage: Remove empty directory only
remove_directory("empty_folder")?;
```

### Example: Remove directory and all contents (dangerous!)

Recursively deletes everything—equivalent to `rm -rf`. No trash bin, no undo. Data is gone forever.
Validate paths carefully; a bug passing `/` or `$HOME` is catastrophic. Consider user confirmation or trash libraries.

```rust
fn remove_directory_all(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
    // Recursively deletes everything
    // Like rm -rf in Unix
}

// Usage: DANGEROUS - recursively delete everything
remove_directory_all("temp")?;
```

### Example: Check if path exists

Works for files, directories, and symlinks (following to target). Returns `false` for broken symlinks.
Note: TOCTOU race possible—file could be deleted after check. For atomic ops, use `create_new(true)`.

```rust
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
    // Returns false for broken symlinks
}

// Usage: Check before processing
if path_exists("config.toml") { /* load config */ }
```

### Example: Check if path is directory

Essential before `read_dir()` or recursion. Follows symlinks (symlink to directory returns `true`).
Returns `false` if path doesn't exist—no separate existence check needed.

```rust
fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
    // Follows symlinks
}

// Usage: Only walk directories
if is_directory("src") { walk_it(); }
```


**Safety note**: `remove_dir_all` is dangerous. It's equivalent to `rm -rf`. There's no trash bin, no undo. Many programs ask for confirmation before using this.



### Example: List directory contents

Reads all entries (files, subdirectories, symlinks). Each `DirEntry` can fail independently; order not guaranteed.
Does not recurse—for recursive listing, see the walk pattern below. Sort explicitly if needed.

```rust
fn list_directory(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;  // Each entry can fail
        entries.push(entry.path());
    }

    Ok(entries)
}

// Usage: Get all entries in directory
let entries = list_directory("src")?;
```

### Example: List only files (skip directories)

Filter to regular files only—`file_type()` avoids extra stat syscall by using cached `DirEntry` metadata.
Common pattern for "find all files" without recursion. Symlinks to files return `is_file() == true`.

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

// Usage: Get only files, not directories
let files = list_files_only("docs")?;
```

### Example: List files with specific extension

Foundation of build tools—`extension()` returns `None` for no extension, excludes the dot (e.g., "rs" not ".rs").
Case-sensitive on most platforms. Combine with recursive walking for "find all .rs files in project".

```rust
// Use this for: Finding all .rs files, .txt files, etc.
fn list_by_extension(
    path: &str, ext: &str
) -> io::Result<Vec<PathBuf>> {
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

// Usage: Find all Rust source files
let rs_files = list_by_extension("src", "rs")?;
```

### Example: Get directory entries with metadata

Retrieve sizes, timestamps, permissions for `ls -l` style listings. `metadata()` uses cached `DirEntry` data when available.
Essential for sorting by size or date. Returns symlink target metadata; use `symlink_metadata()` for symlink itself.

```rust
// Use this for: Sorting by size, filtering by date, etc.
fn list_with_metadata(path: &str)
    -> io::Result<Vec<(PathBuf, fs::Metadata)>>
{
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push((entry.path(), metadata));
    }

    Ok(entries)
}

// Usage: Get files with size and timestamps
let items = list_with_metadata(".")?;
```


**Error handling**: `read_dir()` can fail (directory doesn't exist, no permission). Each call to `entry?` can also fail (permission denied on individual files). Handle both.


### Example: Recursive file listing

Classic depth-first search—recurses into subdirectories, collecting files while skipping directories.
Uses mutable Vec by reference to avoid allocations. Warning: doesn't detect symlink cycles.

```rust
// Classic depth-first search pattern
fn walk_directory(
    path: &Path,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
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

Convenience wrapper that creates the Vec and calls the recursive walker—caller doesn't manage accumulator.
For millions of files, consider returning an iterator instead of collecting. Pre-allocate if count is known.

```rust
fn get_all_files(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory(Path::new(path), &mut files)?;
    Ok(files)
}

// Usage: Get all files in project recursively
let all_files = get_all_files("project")?;
```

### Example: Recursive directory tree printer (visual tree)

Produces Unix `tree`-like output with box-drawing characters. Sorts entries alphabetically for consistency.
Tracks last-entry status for correct connectors (└── vs ├──). Prefix accumulates to show nesting depth.

```rust
// Produces output like the `tree` command
fn print_tree(path: &Path, prefix: &str) -> io::Result<()> {
    let entries = fs::read_dir(path)?;
    let mut entries: Vec<_> = entries.collect::<Result<_, _>>()?;
    entries.sort_by_key(|e| e.path());

    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == entries.len() - 1;
        let conn = if is_last { "└── " } else { "├── " };
        let extension = if is_last { "    " } else { "│   " };

        let name = entry.file_name();
        println!("{}{}{}", prefix, conn, name.to_string_lossy());

        if entry.file_type()?.is_dir() {
            let new_prefix = format!("{}{}", prefix, extension);
            print_tree(&entry.path(), &new_prefix)?;
        }
    }

    Ok(())
}

// Usage: Print directory tree like `tree` command
print_tree(Path::new("src"), "")?;
```

### Example: Find files matching pattern (like find command)

Implements `find . -name '*pattern*'`—combines recursive walking with filename matching.
Uses `contains()` for substring matching; use regex or glob crates for full patterns. Returns all matches.

```rust
fn find_files(
    root: &Path, pattern: &str
) -> io::Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    fn search(
        path: &Path,
        pattern: &str,
        matches: &mut Vec<PathBuf>,
    ) -> io::Result<()> {
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

// Usage: Find all files containing "test" in name
let matches = find_files(Path::new("."), "test")?;
```


**Symlink loops**: This code doesn't detect symlink cycles. If `/a/b` symlinks to `/a`, you'll recurse forever. Production code should track visited inodes (Unix) or use a depth limit.

**Performance**: For very large directories (millions of files), consider parallel traversal or using OS-specific optimizations (like Linux's `readdir64`).




## Pattern 6: Process Spawning and Piping

**Problem**: Need to run external commands and capture output. Want to chain processes like Unix pipelines.

**Solution**: Use Command::new() with .output() (captures all), .status() (inherits streams), or .spawn() (returns immediately). Set Stdio::piped() to capture output.

**Why It Matters**: Integration with system tools essential for build scripts, testing, automation. Improper stream handling causes deadlocks (child blocks on full pipe, parent blocks reading).

**Use Cases**: Build scripts (invoke compilers), test runners (execute programs and check output), automation tools, implementing Unix pipelines (cat|grep|wc), subprocess orchestration.


### Example: Run command and capture output

Simplest approach—`output()` blocks until exit, returning stdout, stderr, and status in memory.
Use `String::from_utf8_lossy` for potentially invalid UTF-8. Best for short commands; use `spawn()` for long-running ones.

```rust
fn run_command() -> io::Result<()> {
    let output = Command::new("ls")
        .arg("-la")
        .output()?;  // Waits for completion, captures all output

    println!("Status: {}", output.status);
    let out = String::from_utf8_lossy(&output.stdout);
    let err = String::from_utf8_lossy(&output.stderr);
    println!("Stdout: {}", out);
    println!("Stderr: {}", err);

    Ok(())
}

// Usage: Run command and get all output
run_command()?;
```

### Example: Check if command succeeded

When you only care about success/failure—`status()` inherits streams so users see output in real-time.
Returns exit status only; use `success()` for boolean or `code()` for exit code. Ideal for builds and tests.

```rust
fn run_command_check() -> io::Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .status()?;  // Inherits streams, waits for exit

    if status.success() {
        println!("Build succeeded!");
    } else {
        println!("Build failed with: {}", status);
    }

    Ok(())
}

// Usage: Run command, check exit status
run_command_check()?;
```

### Example: Run with environment variables

Standard Unix pattern—`.env()` adds or overrides variables; child inherits others from parent.
Use `.env_clear()` for minimal environment. Essential for builds (`CC`, `CFLAGS`) and databases.

```rust
fn run_with_env() -> io::Result<()> {
    let output = Command::new("printenv")
        .env("MY_VAR", "my_value")
        .env("ANOTHER_VAR", "another_value")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

// Usage: Set environment variables for child process
run_with_env()?;
```

### Example: Run in specific directory

Run in a different working directory without changing parent's. Essential for build tools.
Directory must exist or `spawn()` fails. Combine with `.env()` and `.arg()` for full configuration.

```rust
fn run_in_directory() -> io::Result<()> {
    let output = Command::new("pwd")
        .current_dir("/tmp")
        .output()?;

    let cwd = String::from_utf8_lossy(&output.stdout);
    println!("Working directory: {}", cwd);
    Ok(())
}

// Usage: Run command in different directory
run_in_directory()?;
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

For long-running commands, `spawn()` returns immediately with `Child` handle; `Stdio::piped()` captures stdout.
Wrap in `BufReader` for line-by-line iteration. Call `wait()` after reading to get exit status.

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

// Usage: Stream command output in real-time
stream_output()?;
```

### Example: Capture both stdout and stderr separately

Reading both streams requires threads—reading one while the other fills causes deadlock.
Spawn separate threads for each stream, join both before `wait()`. Essential for separating output from errors.

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

// Usage: Read stdout and stderr concurrently
capture_both_streams()?;
```


**Deadlock warning**: If you read stdout while the child is blocked writing to stderr (and vice versa), you deadlock. Use threads or async I/O to read both concurrently.

### Example: Piping Between Processes

Implement Unix pipelines—`Stdio::from()` connects one process's stdout to another's stdin without temp files.
Data streams through OS pipe buffer; OS manages backpressure (writer blocks when buffer fills).

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

// Usage: Pipe ls output to grep (ls | grep txt)
pipe_commands()?;
```

### Example: Complex pipeline: cat file | grep pattern | wc -l

Chain multiple processes—each runs concurrently with data flowing through as produced.
Use `wait_with_output()` on final process. Scales to any number of stages by chaining stdin/stdout.

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
    let count = String::from_utf8_lossy(&output.stdout);
    println!("Lines matching '{}': {}", pattern, count.trim());

    Ok(())
}

// Usage: Count lines matching pattern (cat | grep | wc -l)
complex_pipeline("log.txt", "ERROR")?;
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
