# 16. Synchronous I/O

## File Operations and Buffering

### Basic File Reading

```rust
use std::fs::File;
use std::io::{self, Read};

// Read entire file to string
fn read_to_string(path: &str) -> io::Result<String> {
    std::fs::read_to_string(path)
}

// Read entire file to bytes
fn read_to_bytes(path: &str) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

// Manual reading with buffer
fn read_with_buffer(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Read exact number of bytes
fn read_exact_bytes(path: &str, n: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0; n];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
```

### Basic File Writing

```rust
use std::fs::File;
use std::io::{self, Write};

// Write string to file (overwrites)
fn write_string(path: &str, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
}

// Write bytes to file
fn write_bytes(path: &str, content: &[u8]) -> io::Result<()> {
    std::fs::write(path, content)
}

// Manual writing
fn write_with_handle(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

// Append to file
fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(path)?;

    writeln!(file, "{}", content)?;
    Ok(())
}
```

### File Opening Options

```rust
use std::fs::OpenOptions;
use std::io;

fn advanced_file_opening() -> io::Result<()> {
    // Read-only mode
    let file = OpenOptions::new()
        .read(true)
        .open("data.txt")?;

    // Write-only mode, create if doesn't exist
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open("output.txt")?;

    // Append mode
    let file = OpenOptions::new()
        .append(true)
        .open("log.txt")?;

    // Truncate existing file
    let file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("temp.txt")?;

    // Create new file, fail if exists
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open("unique.txt")?;

    // Read and write
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("data.bin")?;

    // Custom permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .mode(0o644)
            .open("secure.txt")?;
    }

    Ok(())
}
```

### Buffered I/O

```rust
use std::fs::File;
use std::io::{self, BufReader, BufWriter, BufRead, Write};

// Buffered reading
fn buffered_read(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }

    Ok(())
}

// Buffered reading with custom buffer size
fn buffered_read_custom_size(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(8192, file);

    for line in reader.lines() {
        println!("{}", line?);
    }

    Ok(())
}

// Buffered writing
fn buffered_write(path: &str, lines: &[&str]) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    for line in lines {
        writeln!(writer, "{}", line)?;
    }

    writer.flush()?; // Ensure all data is written
    Ok(())
}

// Read until delimiter
fn read_until_delimiter(path: &str, delimiter: u8) -> io::Result<Vec<Vec<u8>>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut chunks = Vec::new();

    loop {
        let mut chunk = Vec::new();
        let bytes_read = reader.read_until(delimiter, &mut chunk)?;

        if bytes_read == 0 {
            break;
        }

        chunks.push(chunk);
    }

    Ok(chunks)
}
```

### Line-by-Line Processing

```rust
use std::fs::File;
use std::io::{self, BufRead, BufReader};

// Process large files line by line
fn process_large_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line?;

        // Process each line
        if line.starts_with('#') {
            continue; // Skip comments
        }

        println!("Line {}: {}", index + 1, line);
    }

    Ok(())
}

// Read lines with error handling
fn read_lines_robust(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines().collect()
}

// Read specific line
fn read_line_at_index(path: &str, target: usize) -> io::Result<Option<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    reader.lines()
        .nth(target)
        .transpose()
}
```

### Seeking and Random Access

```rust
use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};

fn seeking_example(path: &str) -> io::Result<()> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    // Write some data
    file.write_all(b"Hello, World!")?;

    // Seek to beginning
    file.seek(SeekFrom::Start(0))?;

    // Read first 5 bytes
    let mut buffer = [0; 5];
    file.read_exact(&mut buffer)?;
    println!("First 5 bytes: {:?}", std::str::from_utf8(&buffer)?);

    // Seek to end
    let file_size = file.seek(SeekFrom::End(0))?;
    println!("File size: {}", file_size);

    // Seek relative to current position
    file.seek(SeekFrom::Current(-5))?;

    // Read from current position
    let mut buffer = [0; 5];
    file.read_exact(&mut buffer)?;
    println!("Last 5 bytes: {:?}", std::str::from_utf8(&buffer)?);

    Ok(())
}

// Random access reads
fn read_at_offset(path: &str, offset: u64, size: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = vec![0; size];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

// Overwrite at specific position
fn write_at_offset(path: &str, offset: u64, data: &[u8]) -> io::Result<()> {
    let mut file = File::options()
        .write(true)
        .open(path)?;

    file.seek(SeekFrom::Start(offset))?;
    file.write_all(data)?;
    Ok(())
}
```

### File Metadata and Permissions

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

    if let Ok(modified) = metadata.modified() {
        println!("Modified: {:?}", modified);
    }

    if let Ok(accessed) = metadata.accessed() {
        println!("Accessed: {:?}", accessed);
    }

    if let Ok(created) = metadata.created() {
        println!("Created: {:?}", created);
    }

    // Unix-specific metadata
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        println!("UID: {}", metadata.uid());
        println!("GID: {}", metadata.gid());
        println!("Mode: {:o}", metadata.mode());
    }

    Ok(())
}

// Set permissions
fn set_permissions(path: &str, readonly: bool) -> io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_readonly(readonly);
    fs::set_permissions(path, perms)?;
    Ok(())
}

// Unix-specific: Set mode
#[cfg(unix)]
fn set_mode(path: &str, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms)?;
    Ok(())
}
```

### Copying and Moving Files

```rust
use std::fs;
use std::io;
use std::path::Path;

// Copy file
fn copy_file(src: &str, dst: &str) -> io::Result<u64> {
    fs::copy(src, dst)
}

// Copy with progress tracking
fn copy_file_with_progress(src: &str, dst: &str) -> io::Result<()> {
    use std::io::{BufReader, BufWriter, Read, Write};

    let src_file = File::open(src)?;
    let dst_file = File::create(dst)?;

    let mut reader = BufReader::new(src_file);
    let mut writer = BufWriter::new(dst_file);

    let total_size = reader.get_ref().metadata()?.len();
    let mut copied = 0u64;
    let mut buffer = [0; 8192];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }

        writer.write_all(&buffer[..n])?;
        copied += n as u64;

        let progress = (copied as f64 / total_size as f64) * 100.0;
        print!("\rProgress: {:.2}%", progress);
    }

    println!("\nCopy complete!");
    Ok(())
}

// Move/rename file
fn move_file(src: &str, dst: &str) -> io::Result<()> {
    fs::rename(src, dst)
}

// Hard link
fn create_hard_link(src: &str, dst: &str) -> io::Result<()> {
    fs::hard_link(src, dst)
}

// Symbolic link
#[cfg(unix)]
fn create_symlink(src: &str, dst: &str) -> io::Result<()> {
    std::os::unix::fs::symlink(src, dst)
}

// Remove file
fn delete_file(path: &str) -> io::Result<()> {
    fs::remove_file(path)
}
```

## Standard Streams (stdin/stdout/stderr)

### Reading from stdin

```rust
use std::io::{self, BufRead, Write};

// Read single line
fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// Read with prompt
fn prompt(message: &str) -> io::Result<String> {
    print!("{}", message);
    io::stdout().flush()?; // Ensure prompt is displayed
    read_line()
}

// Read multiple lines
fn read_lines() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let reader = stdin.lock();

    reader.lines().collect()
}

// Read until empty line
fn read_until_empty() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let mut lines = Vec::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        lines.push(line);
    }

    Ok(lines)
}

// Read and parse integer
fn read_integer() -> io::Result<i32> {
    let input = prompt("Enter a number: ")?;
    input.parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

// Interactive menu
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

### Writing to stdout/stderr

```rust
use std::io::{self, Write};

fn stdout_examples() -> io::Result<()> {
    // Basic println
    println!("Hello, World!");

    // Write to stdout directly
    io::stdout().write_all(b"Direct write\n")?;

    // Buffered writing
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "Buffered write")?;
    handle.flush()?;

    // Write without newline
    print!("No newline ");
    io::stdout().flush()?;
    println!("here!");

    Ok(())
}

fn stderr_examples() -> io::Result<()> {
    // Write to stderr
    eprintln!("Error message");

    // Direct stderr write
    io::stderr().write_all(b"Direct error\n")?;

    // Formatted error
    let error_code = 42;
    eprintln!("Error code: {}", error_code);

    Ok(())
}

// Progress indicator
fn progress_indicator() -> io::Result<()> {
    for i in 0..=100 {
        print!("\rProgress: {}%", i);
        io::stdout().flush()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    println!(); // New line after progress
    Ok(())
}

// Colored output (requires termcolor crate in real usage)
fn colored_output() {
    println!("\x1b[31mRed text\x1b[0m");
    println!("\x1b[32mGreen text\x1b[0m");
    println!("\x1b[33mYellow text\x1b[0m");
    println!("\x1b[1mBold text\x1b[0m");
}
```

### Locking Streams for Performance

```rust
use std::io::{self, BufRead, Write};

// Efficient stdout writing
fn efficient_stdout_writing(lines: &[&str]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock(); // Lock once for multiple writes

    for line in lines {
        writeln!(handle, "{}", line)?;
    }

    // Lock is automatically released when handle goes out of scope
    Ok(())
}

// Efficient stdin reading
fn efficient_stdin_reading() -> io::Result<Vec<String>> {
    let stdin = io::stdin();
    let reader = stdin.lock(); // Lock once for multiple reads

    reader.lines().collect()
}

// Combined stdin/stdout operations
fn echo_uppercase() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut reader = stdin.lock();
    let mut writer = stdout.lock();

    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        writeln!(writer, "{}", line.to_uppercase())?;
        line.clear();
    }

    Ok(())
}
```

### Redirecting Streams

```rust
use std::fs::File;
use std::io::{self, Write};

// Capture stdout to file
fn redirect_stdout_to_file(path: &str) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = io::BufWriter::new(file);

    writeln!(writer, "This goes to file instead of stdout")?;
    writeln!(writer, "Another line")?;

    Ok(())
}

// Tee: write to both stdout and file
fn tee_output(path: &str, message: &str) -> io::Result<()> {
    // Write to stdout
    println!("{}", message);

    // Also write to file
    let mut file = File::create(path)?;
    writeln!(file, "{}", message)?;

    Ok(())
}

// Custom writer that writes to multiple destinations
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
            writer.write_all(buf)?;
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

## Memory-Mapped I/O

### Basic Memory Mapping (using memmap2 crate)

```rust
// Note: In real code, add to Cargo.toml:
// [dependencies]
// memmap2 = "0.9"

// This is a conceptual example showing the API
#[cfg(feature = "memmap_example")]
mod memmap_examples {
    use memmap2::{Mmap, MmapMut, MmapOptions};
    use std::fs::File;
    use std::io::{self, Write};

    // Read-only memory map
    fn mmap_read(path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Access memory like a byte slice
        let data: &[u8] = &mmap[..];
        println!("First 10 bytes: {:?}", &data[..10.min(data.len())]);

        Ok(())
    }

    // Mutable memory map
    fn mmap_write(path: &str) -> io::Result<()> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // Set file size
        file.set_len(1024)?;

        let mut mmap = unsafe { MmapMut::map_mut(&file)? };

        // Write data
        mmap[0..5].copy_from_slice(b"Hello");

        // Flush to disk
        mmap.flush()?;

        Ok(())
    }

    // Anonymous memory map (not backed by file)
    fn mmap_anonymous() -> io::Result<()> {
        let mut mmap = MmapMut::map_anon(1024)?;

        mmap[0..5].copy_from_slice(b"Hello");

        println!("Data: {:?}", &mmap[0..5]);

        Ok(())
    }

    // Large file processing with mmap
    fn process_large_file_mmap(path: &str) -> io::Result<usize> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Count newlines efficiently
        let count = mmap.iter().filter(|&&b| b == b'\n').count();

        Ok(count)
    }
}
```

### Manual Memory Mapping (Unix)

```rust
#[cfg(unix)]
mod unix_mmap {
    use std::fs::File;
    use std::io;
    use std::os::unix::io::AsRawFd;

    pub struct Mmap {
        ptr: *mut libc::c_void,
        len: usize,
    }

    impl Mmap {
        pub unsafe fn map(file: &File, len: usize) -> io::Result<Self> {
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                file.as_raw_fd(),
                0,
            );

            if ptr == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            Ok(Mmap { ptr, len })
        }

        pub fn as_slice(&self) -> &[u8] {
            unsafe {
                std::slice::from_raw_parts(self.ptr as *const u8, self.len)
            }
        }

        pub fn as_mut_slice(&mut self) -> &mut [u8] {
            unsafe {
                std::slice::from_raw_parts_mut(self.ptr as *mut u8, self.len)
            }
        }
    }

    impl Drop for Mmap {
        fn drop(&mut self) {
            unsafe {
                libc::munmap(self.ptr, self.len);
            }
        }
    }
}
```

### File-backed Shared Memory

```rust
use std::fs::File;
use std::io::{self, Write, Seek, SeekFrom};

// Simulate memory-mapped behavior without external crate
fn shared_file_buffer(path: &str) -> io::Result<()> {
    // Create and initialize file
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    file.set_len(1024)?; // Pre-allocate space

    // Write at beginning
    file.seek(SeekFrom::Start(0))?;
    file.write_all(b"Process 1 data")?;

    // Write at offset
    file.seek(SeekFrom::Start(100))?;
    file.write_all(b"Process 2 data")?;

    file.sync_all()?; // Ensure written to disk

    Ok(())
}
```

## Directory Traversal

### Basic Directory Operations

```rust
use std::fs;
use std::io;
use std::path::Path;

// Create directory
fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir(path)
}

// Create directory and all parent directories
fn create_directory_all(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
}

// Remove empty directory
fn remove_directory(path: &str) -> io::Result<()> {
    fs::remove_dir(path)
}

// Remove directory and all contents
fn remove_directory_all(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
}

// Check if path exists
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

// Check if path is directory
fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
}
```

### Reading Directory Contents

```rust
use std::fs;
use std::io;
use std::path::PathBuf;

// List directory contents
fn list_directory(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        entries.push(entry.path());
    }

    Ok(entries)
}

// List with filtering
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

// List with extension filter
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

// Get directory entries with metadata
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

### Recursive Directory Traversal

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Recursive file listing
fn walk_directory(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                walk_directory(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

// Get all files recursively
fn get_all_files(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory(Path::new(path), &mut files)?;
    Ok(files)
}

// Recursive directory tree printer
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

// Find files matching pattern
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

### Walking with Iterators (using walkdir crate pattern)

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Custom directory walker
struct DirWalker {
    stack: Vec<PathBuf>,
}

impl DirWalker {
    fn new(path: impl AsRef<Path>) -> Self {
        DirWalker {
            stack: vec![path.as_ref().to_path_buf()],
        }
    }
}

impl Iterator for DirWalker {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            if path.is_dir() {
                match fs::read_dir(&path) {
                    Ok(entries) => {
                        for entry in entries {
                            match entry {
                                Ok(e) => self.stack.push(e.path()),
                                Err(e) => return Some(Err(e)),
                            }
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            } else {
                return Some(Ok(path));
            }
        }
        None
    }
}

// Usage
fn use_walker() -> io::Result<()> {
    for entry in DirWalker::new(".") {
        let path = entry?;
        println!("{}", path.display());
    }
    Ok(())
}
```

### Directory Statistics

```rust
use std::fs;
use std::io;
use std::path::Path;

struct DirStats {
    files: usize,
    dirs: usize,
    total_size: u64,
}

fn calculate_dir_stats(path: &Path) -> io::Result<DirStats> {
    let mut stats = DirStats {
        files: 0,
        dirs: 0,
        total_size: 0,
    };

    fn visit(path: &Path, stats: &mut DirStats) -> io::Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                stats.dirs += 1;
                visit(&path, stats)?;
            } else {
                stats.files += 1;
                stats.total_size += metadata.len();
            }
        }
        Ok(())
    }

    visit(path, &mut stats)?;
    Ok(stats)
}

fn print_dir_stats(path: &str) -> io::Result<()> {
    let stats = calculate_dir_stats(Path::new(path))?;

    println!("Directory statistics for: {}", path);
    println!("  Files: {}", stats.files);
    println!("  Directories: {}", stats.dirs);
    println!("  Total size: {} bytes", stats.total_size);
    println!("  Total size: {:.2} MB", stats.total_size as f64 / 1_048_576.0);

    Ok(())
}
```

## Process Spawning and Piping

### Basic Process Execution

```rust
use std::process::{Command, Stdio};
use std::io::{self, Write};

// Run command and wait
fn run_command() -> io::Result<()> {
    let output = Command::new("ls")
        .arg("-la")
        .output()?;

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(())
}

// Check if command succeeded
fn run_command_check() -> io::Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .status()?;

    if status.success() {
        println!("Build succeeded!");
    } else {
        println!("Build failed with: {}", status);
    }

    Ok(())
}

// Run with environment variables
fn run_with_env() -> io::Result<()> {
    let output = Command::new("printenv")
        .env("MY_VAR", "my_value")
        .env("ANOTHER_VAR", "another_value")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

// Run in specific directory
fn run_in_directory() -> io::Result<()> {
    let output = Command::new("pwd")
        .current_dir("/tmp")
        .output()?;

    println!("Working directory: {}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
```

### Streaming Output

```rust
use std::process::{Command, Stdio};
use std::io::{self, BufRead, BufReader};

// Stream stdout in real-time
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

// Capture both stdout and stderr separately
fn capture_both_streams() -> io::Result<()> {
    let mut child = Command::new("cargo")
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Read stdout
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("[OUT] {}", line);
            }
        }
    });

    // Read stderr
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

### Piping Between Processes

```rust
use std::process::{Command, Stdio};
use std::io::{self, Write};

// Pipe output from one command to another (like: ls | grep txt)
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

// More complex pipeline: cat file | grep pattern | wc -l
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

### Writing to Child Process stdin

```rust
use std::process::{Command, Stdio};
use std::io::{self, Write};

// Send data to child process
fn write_to_child() -> io::Result<()> {
    let mut child = Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(b"Hello, child process!\n")?;
        stdin.write_all(b"Second line\n")?;
    } // stdin is closed when it goes out of scope

    let output = child.wait_with_output()?;
    println!("Child output: {}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

// Interactive with child process
fn interactive_child() -> io::Result<()> {
    use std::io::BufReader;

    let mut child = Command::new("bc") // Calculator
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();

    // Send computation
    writeln!(stdin, "2 + 2")?;
    writeln!(stdin, "quit")?;

    // Read result
    let reader = BufReader::new(stdout);
    for line in reader.lines() {
        println!("Result: {}", line?);
    }

    child.wait()?;
    Ok(())
}
```

### Process Timeout and Killing

```rust
use std::process::{Command, Stdio};
use std::io;
use std::time::Duration;
use std::thread;

// Run with timeout
fn run_with_timeout(timeout_secs: u64) -> io::Result<()> {
    let mut child = Command::new("sleep")
        .arg("10")
        .spawn()?;

    let timeout = Duration::from_secs(timeout_secs);

    thread::sleep(timeout);

    match child.try_wait()? {
        Some(status) => println!("Process finished: {}", status),
        None => {
            println!("Timeout reached, killing process");
            child.kill()?;
            child.wait()?;
        }
    }

    Ok(())
}

// Monitor child process
fn monitor_child() -> io::Result<()> {
    let mut child = Command::new("long_running_task")
        .spawn()?;

    loop {
        match child.try_wait()? {
            Some(status) => {
                println!("Process exited with: {}", status);
                break;
            }
            None => {
                println!("Still running...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    Ok(())
}
```

### Spawning Detached Processes

```rust
use std::process::{Command, Stdio};
use std::io;

// Spawn background process (Unix)
#[cfg(unix)]
fn spawn_daemon() -> io::Result<()> {
    Command::new("my_daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Parent can exit, daemon continues
    Ok(())
}

// Double-fork daemon pattern (Unix)
#[cfg(unix)]
fn daemonize() -> io::Result<()> {
    unsafe {
        match libc::fork() {
            -1 => return Err(io::Error::last_os_error()),
            0 => {
                // Child process
                libc::setsid(); // Create new session

                match libc::fork() {
                    -1 => std::process::exit(1),
                    0 => {
                        // Grandchild - actual daemon
                        // Daemon code here
                        println!("Running as daemon");
                    }
                    _ => {
                        // Child exits
                        std::process::exit(0);
                    }
                }
            }
            _ => {
                // Parent exits
                std::process::exit(0);
            }
        }
    }

    Ok(())
}
```

### Process Builder Pattern

```rust
use std::process::{Command, Stdio};
use std::io;
use std::path::PathBuf;

struct ProcessBuilder {
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: Option<PathBuf>,
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

impl ProcessBuilder {
    fn new(program: impl Into<String>) -> Self {
        ProcessBuilder {
            program: program.into(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
            stdin: Stdio::inherit(),
            stdout: Stdio::inherit(),
            stderr: Stdio::inherit(),
        }
    }

    fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.push((key.into(), val.into()));
        self
    }

    fn current_dir(mut self, dir: PathBuf) -> Self {
        self.cwd = Some(dir);
        self
    }

    fn stdin(mut self, cfg: Stdio) -> Self {
        self.stdin = cfg;
        self
    }

    fn stdout(mut self, cfg: Stdio) -> Self {
        self.stdout = cfg;
        self
    }

    fn stderr(mut self, cfg: Stdio) -> Self {
        self.stderr = cfg;
        self
    }

    fn spawn(self) -> io::Result<std::process::Child> {
        let mut cmd = Command::new(self.program);
        cmd.args(self.args)
            .stdin(self.stdin)
            .stdout(self.stdout)
            .stderr(self.stderr);

        for (key, val) in self.env {
            cmd.env(key, val);
        }

        if let Some(cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        cmd.spawn()
    }

    fn output(self) -> io::Result<std::process::Output> {
        self.spawn()?.wait_with_output()
    }
}

// Usage
fn use_process_builder() -> io::Result<()> {
    let output = ProcessBuilder::new("ls")
        .arg("-la")
        .arg("/tmp")
        .env("MY_VAR", "value")
        .stdout(Stdio::piped())
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
```

This comprehensive guide covers all essential synchronous I/O patterns in Rust, from basic file operations to advanced process management.
