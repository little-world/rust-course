### File IO Cheat Sheet
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


