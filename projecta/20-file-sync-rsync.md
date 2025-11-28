
## Project: File Synchronization Tool (Simplified rsync)

### Problem Statement

Build a file synchronization tool similar to `rsync` that efficiently copies only changed files between two directories. You'll implement recursive directory traversal with cycle detection, metadata comparison to identify changes, buffered I/O for efficient copying, progress reporting, and error handling for real-world file system issues.

### Use Cases

**When you need this pattern**:
1. **Backup systems**: Incremental backups that only copy changed files
2. **Deployment tools**: Sync application files to servers
3. **Build systems**: Copy updated artifacts to output directories
4. **Cloud sync**: Local-to-remote file synchronization
5. **Content delivery**: Mirror websites or assets across servers
6. **Development workflows**: Sync source files between machines

### Why It Matters

**Real-World Impact**: File synchronization is fundamental to countless production tools:

**The Naive Approach Problem**:
```rust
// Inefficient: Always copy everything
fn naive_sync(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        fs::copy(entry.path(), dst.join(entry.file_name()))?;
        // Problems:
        // - Copies unchanged files (wastes bandwidth/time)
        // - No progress reporting (user waits blindly)
        // - Doesn't handle subdirectories
        // - No symlink cycle detection (infinite loops)
        // - No error recovery (first error aborts everything)
    }
    Ok(())
}
```

**Smart Synchronization Benefits**:
- **Efficiency**: Only copy changed files (10x-100x faster for large trees)
- **Bandwidth**: Critical for remote sync (network transfers expensive)
- **Incremental backups**: Daily backups copy only today's changes
- **User experience**: Progress reporting shows work being done
- **Reliability**: Graceful error handling continues despite permission errors

**Performance Comparison**:
- **Copy everything**: 1000 files × 1MB = 1GB transferred, ~10 seconds
- **Smart sync**: 10 changed files × 1MB = 10MB transferred, ~0.1 seconds
- **100x improvement** for typical workloads with few changes

**Real-World Tools Using These Patterns**:
- `rsync`: Industry-standard file synchronization
- `git`: File tracking and synchronization
- `dropbox/gdrive`: Cloud file sync clients
- Docker image layers: Only copy changed layers
- CI/CD systems: Deploy only changed artifacts

### Learning Goals

By completing this project, you will:

1. **Master directory traversal**: Recursive walking with cycle detection
2. **Understand file metadata**: Modification times, sizes, permissions
3. **Efficient I/O patterns**: Buffered copying with progress tracking
4. **Error handling**: Graceful degradation for file system errors
5. **Pattern matching**: Glob patterns for filtering files
6. **Performance optimization**: Avoid O(N²) operations in file trees
7. **User experience**: Progress reporting and dry-run modes

---

### Milestone 1: Basic Directory Traversal

**Goal**: Recursively walk directory trees and list all files.

**Implementation Steps**:

1. **Implement recursive directory walker**:
   - Use `fs::read_dir()` to list directory contents
   - Recursively descend into subdirectories
   - Distinguish files from directories using `entry.file_type()`
   - Build `Vec<PathBuf>` of all file paths
   - Preserve relative paths from base directory

2. **Handle basic errors**:
   - Permission denied errors (skip with warning)
   - Invalid symlinks (skip with warning)
   - Use `Result` and `?` operator appropriately
   - Continue processing despite individual errors

3. **Implement path filtering**:
   - Filter by file extension (e.g., only `.txt`, `.rs`)
   - Skip hidden files (starting with `.`)
   - Basic glob pattern matching (`*.rs`, `src/**/*.rs`)

4. **Test on real directories**:
   - Walk `/usr/bin` and count files
   - Walk project source tree
   - Handle permission denied gracefully


**Starter Code**:

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Recursively list all files in a directory tree
pub fn list_files_recursive(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // TODO: Call recursive helper
    // Hint: collect_files(dir, dir, &mut files)?;

    todo!()
}

/// Recursive helper that preserves relative paths
fn collect_files(base: &Path, current: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    // TODO: Read directory entries
    // Hint: for entry in fs::read_dir(current)? { ... }

    // TODO: For each entry:
    //   - Get file type (entry.file_type()?)
    //   - If directory: recurse
    //   - If file: add to files vec (use relative path from base)

    // TODO: Handle errors gracefully
    //   - Skip entries that error (permission denied)
    //   - Print warnings with eprintln!
    //   - Continue processing other entries

    // Hint: Use entry.path().strip_prefix(base) to get relative path

    todo!()
}

/// Filter files by extension
pub fn filter_by_extension<'a>(
    files: &'a [PathBuf],
    extension: &str,
) -> Vec<&'a PathBuf> {
    // TODO: Iterate through files
    // TODO: Check if path.extension() matches
    // TODO: Return filtered list
    // Hint: files.iter().filter(|p| ...).collect()

    todo!()
}

/// Check if path matches glob pattern
pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
    // TODO: Simple pattern matching
    // Support: *.txt, src/*.rs, **/*.txt (recursive)
    // Hint: Use path.extension() and path.file_name()
    // Advanced: Use glob crate for full glob support

    todo!()
}

#[cfg(test)]
fn create_test_tree() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();

    fs::write(temp.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp.path().join("file2.rs"), "fn main() {}").unwrap();

    let subdir = temp.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("file3.txt"), "content3").unwrap();
    fs::write(subdir.join("file4.rs"), "struct S;").unwrap();

    temp
}
```

**Checkpoint Tests**:
```rust
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

#[test]
fn test_list_files_recursive() {
    // Create test directory structure
    let temp = create_test_tree();
    // temp/
    //   file1.txt
    //   file2.rs
    //   subdir/
    //     file3.txt
    //     file4.rs

    let files = list_files_recursive(temp.path()).unwrap();

    assert_eq!(files.len(), 4);
    assert!(files.iter().any(|p| p.ends_with("file1.txt")));
    assert!(files.iter().any(|p| p.ends_with("subdir/file3.txt")));
}

#[test]
fn test_filter_by_extension() {
    let temp = create_test_tree();

    let files = list_files_recursive(temp.path())
        .unwrap()
        .into_iter()
        .filter(|p| p.extension().map_or(false, |e| e == "txt"))
        .collect::<Vec<_>>();

    assert_eq!(files.len(), 2);
    assert!(files.iter().all(|p| p.extension().unwrap() == "txt"));
}

#[test]
fn test_handle_permission_denied() {
    // Create directory with no read permissions
    #[cfg(unix)]
    {
        let temp = tempfile::tempdir().unwrap();
        let no_read = temp.path().join("forbidden");
        fs::create_dir(&no_read).unwrap();

        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&no_read, fs::Permissions::from_mode(0o000)).unwrap();

        // Should not panic, should skip with warning
        let result = list_files_recursive(temp.path());
        assert!(result.is_ok());
    }
}

#[test]
fn test_empty_directory() {
    let temp = tempfile::tempdir().unwrap();
    let files = list_files_recursive(temp.path()).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_nested_directories() {
    let temp = tempfile::tempdir().unwrap();
    let deep = temp.path().join("a/b/c/d");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("deep.txt"), "content").unwrap();

    let files = list_files_recursive(temp.path()).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("a/b/c/d/deep.txt"));
}
```

**Check Your Understanding**:
- Why use `fs::read_dir()` iterator instead of collecting all entries at once?
- How do we distinguish files from directories?
- Why preserve relative paths instead of absolute paths?
- What errors can occur during directory traversal?

---

### Milestone 2: Symlink Cycle Detection

**Goal**: Detect and prevent infinite loops from circular symlinks.

**Implementation Steps**:

1. **Understand the symlink cycle problem**:
   - Symlink `a/link` → `a` creates cycle
   - Naive traversal loops forever
   - Need to track visited directories

2. **Implement cycle detection**:
   - Use `HashSet<PathBuf>` to track visited directories
   - Before recursing, check if directory already visited
   - Use canonical paths with `fs::canonicalize()` to resolve symlinks
   - Skip directory if already in visited set

3. **Handle symlink errors**:
   - Broken symlinks (point to nonexistent targets)
   - Permission denied on symlink targets
   - Symlinks to files vs directories

4. **Test with real symlinks**:
   - Create test with circular symlinks
   - Verify traversal terminates
   - Count how many times cycle is detected


**Starter Code Extension**:

```rust
use std::collections::HashSet;

/// Recursively list files with symlink cycle detection
pub fn list_files_recursive_safe(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();

    // TODO: Call recursive helper with visited set
    // Hint: collect_files_safe(dir, dir, &mut files, &mut visited)?;

    todo!()
}

fn collect_files_safe(
    base: &Path,
    current: &Path,
    files: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> io::Result<()> {
    // TODO: Get canonical path (resolves symlinks)
    // Hint: let canonical = fs::canonicalize(current)?;

    // TODO: Check if already visited
    // Hint: if !visited.insert(canonical.clone()) { return Ok(()); }

    // TODO: Read directory entries
    // TODO: For each entry:
    //   - Check if it's a symlink (entry.file_type()?.is_symlink())
    //   - If directory: recurse with visited set
    //   - If file: add to files vec

    // TODO: Handle errors:
    //   - canonicalize() fails for broken symlinks
    //   - read_dir() fails for permission denied

    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
#[cfg(unix)]
fn test_symlink_cycle_detection() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();

    // Create directory structure with cycle:
    // temp/
    //   a/
    //     file.txt
    //     link -> ../a  (points to parent, creates cycle)

    let a_dir = temp.path().join("a");
    fs::create_dir(&a_dir).unwrap();
    fs::write(a_dir.join("file.txt"), "content").unwrap();

    // Create symlink cycle
    symlink(&a_dir, a_dir.join("link")).unwrap();

    // Should not hang, should detect cycle
    let files = list_files_recursive_safe(temp.path()).unwrap();

    // Should find file1.txt exactly once, not infinite times
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("file.txt"));
}

#[test]
#[cfg(unix)]
fn test_broken_symlink() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let nonexistent = temp.path().join("nonexistent");
    symlink(&nonexistent, temp.path().join("broken_link")).unwrap();

    // Should handle broken symlinks gracefully
    let files = list_files_recursive_safe(temp.path()).unwrap();
    assert!(files.is_empty()); // Broken symlink skipped
}

#[test]
#[cfg(unix)]
fn test_symlink_to_file() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("file.txt"), "content").unwrap();
    symlink(
        temp.path().join("file.txt"),
        temp.path().join("link_to_file"),
    )
    .unwrap();

    let files = list_files_recursive_safe(temp.path()).unwrap();

    // Should include both original file and symlink
    // (or just original, depending on implementation)
    assert!(files.len() >= 1);
}

#[test]
fn test_very_deep_nesting() {
    let temp = tempfile::tempdir().unwrap();

    // Create deeply nested structure (100 levels)
    let mut path = temp.path().to_path_buf();
    for i in 0..100 {
        path = path.join(format!("level{}", i));
        fs::create_dir(&path).unwrap();
    }
    fs::write(path.join("deep.txt"), "found me!").unwrap();

    let files = list_files_recursive_safe(temp.path()).unwrap();
    assert_eq!(files.len(), 1);
}
```

**Check Your Understanding**:
- Why use canonical paths instead of raw paths?
- What's the difference between `entry.file_type()` and `entry.metadata()`?
- How does `HashSet::insert()` return value help detect cycles?
- What happens if `canonicalize()` fails?

---

### Milestone 3: Metadata Comparison and Change Detection

**Goal**: Compare file metadata to identify which files need syncing.

**Implementation Steps**:

1. **Implement metadata extraction**:
   - Get modification time (`metadata.modified()`)
   - Get file size (`metadata.len()`)
   - Compare modification times between source and destination
   - Determine if file is newer, older, or same

2. **Build sync plan**:
   - Compare source and destination directory trees
   - Identify files that exist only in source (new files)
   - Identify files that exist in both but are different (modified files)
   - Identify files that exist only in destination (deleted from source)
   - Return `SyncPlan` with lists of actions

3. **Implement comparison strategies**:
   - **Timestamp-based**: Compare `modified()` times
   - **Size-based**: Compare file sizes
   - **Checksum-based**: Compute and compare SHA256 hashes (slower but accurate)
   - Allow choosing strategy

4. **Handle edge cases**:
   - Files with identical timestamps but different content
   - Timestamp precision issues (filesystem-dependent)
   - Files being modified during sync


**Starter Code**:

```rust
use std::time::SystemTime;

/// Compare source and destination directories, return sync plan
pub fn build_sync_plan(src: &Path, dst: &Path) -> io::Result<Vec<SyncItem>> {
    // TODO: List all files in source
    // TODO: List all files in destination
    // TODO: For each source file:
    //   - Check if exists in destination
    //   - If not: action = Copy
    //   - If yes: compare metadata
    //     - If src newer: action = Update
    //     - If same: action = Skip
    // TODO: Return sorted list of SyncItems

    todo!()
}

/// Compare two files by metadata (timestamp and size)
fn should_update(src_path: &Path, dst_path: &Path) -> io::Result<bool> {
    // TODO: Get metadata for both files
    // TODO: Compare modification times
    // TODO: Compare file sizes
    // TODO: Return true if source is newer or different size

    // Hint:
    // let src_meta = fs::metadata(src_path)?;
    // let dst_meta = fs::metadata(dst_path)?;
    // let src_mtime = src_meta.modified()?;
    // let dst_mtime = dst_meta.modified()?;
    // Ok(src_mtime > dst_mtime || src_meta.len() != dst_meta.len())

    todo!()
}

/// Compute SHA256 checksum of file
fn compute_checksum(path: &Path) -> io::Result<String> {
    use std::io::Read;
    use sha2::{Sha256, Digest};

    // TODO: Open file with BufReader
    // TODO: Read in chunks and update hasher
    // TODO: Finalize hash and return as hex string

    // Hint:
    // let mut file = BufReader::new(File::open(path)?);
    // let mut hasher = Sha256::new();
    // let mut buffer = [0u8; 8192];
    // loop {
    //     let n = file.read(&mut buffer)?;
    //     if n == 0 { break; }
    //     hasher.update(&buffer[..n]);
    // }
    // Ok(format!("{:x}", hasher.finalize()))

    todo!()
}

/// Build sync plan using checksum comparison
pub fn build_sync_plan_checksum(src: &Path, dst: &Path) -> io::Result<Vec<SyncItem>> {
    // TODO: Similar to build_sync_plan but use checksums instead of timestamps
    // TODO: Only compute checksums when file exists in both locations
    // TODO: Skip checksum if size differs (optimization)

    todo!()
}
```

**Check Your Understanding**:
- Why might timestamp comparison give false positives?
- What are the trade-offs of checksum vs timestamp comparison?
- How does filesystem timestamp precision affect comparisons?
- Why check size before computing expensive checksums?

---
**Checkpoint Tests**:
```rust
use std::time::SystemTime;

#[derive(Debug, PartialEq)]
pub enum SyncAction {
    Copy,      // File doesn't exist in destination
    Update,    // File exists but is older
    Skip,      // File is up-to-date
    Delete,    // File exists in dest but not source (optional)
}

#[derive(Debug)]
pub struct SyncItem {
    pub path: PathBuf,
    pub action: SyncAction,
}

#[test]
fn test_detect_new_files() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("new.txt"), "content").unwrap();

    let plan = build_sync_plan(src.path(), dst.path()).unwrap();

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, SyncAction::Copy);
    assert!(plan[0].path.ends_with("new.txt"));
}

#[test]
fn test_detect_modified_files() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    // Create file in both, but src is newer
    fs::write(dst.path().join("file.txt"), "old content").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::write(src.path().join("file.txt"), "new content").unwrap();

    let plan = build_sync_plan(src.path(), dst.path()).unwrap();

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, SyncAction::Update);
}

#[test]
fn test_detect_unchanged_files() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    let content = "same content";
    fs::write(src.path().join("file.txt"), content).unwrap();
    fs::write(dst.path().join("file.txt"), content).unwrap();

    // Set same modification time
    let src_file = src.path().join("file.txt");
    let dst_file = dst.path().join("file.txt");
    let mtime = fs::metadata(&src_file).unwrap().modified().unwrap();
    filetime::set_file_mtime(&dst_file, filetime::FileTime::from_system_time(mtime)).unwrap();

    let plan = build_sync_plan(src.path(), dst.path()).unwrap();

    assert_eq!(plan.len(), 1);
    assert_eq!(plan[0].action, SyncAction::Skip);
}

#[test]
fn test_size_difference_detection() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("file.txt"), "longer content").unwrap();
    fs::write(dst.path().join("file.txt"), "short").unwrap();

    let plan = build_sync_plan(src.path(), dst.path()).unwrap();

    assert_eq!(plan[0].action, SyncAction::Update);
}

#[test]
fn test_checksum_comparison() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    // Same content, different timestamps
    let content = "identical content";
    fs::write(src.path().join("file.txt"), content).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));

    fs::write(dst.path().join("file.txt"), content).unwrap();

    // Checksum-based comparison should detect they're identical
    let plan = build_sync_plan_checksum(src.path(), dst.path()).unwrap();

    assert_eq!(plan[0].action, SyncAction::Skip);
}
```


### Milestone 4: Efficient File Copying with Progress

**Goal**: Copy files using buffered I/O and report progress.

**Implementation Steps**:

1. **Implement buffered file copy**:
   - Use `BufReader` for source file
   - Use `BufWriter` for destination file
   - Copy in chunks (8KB or 64KB)
   - Preserve file permissions and timestamps

2. **Add progress reporting**:
   - Track bytes copied
   - Print progress to stdout with `\r` for same-line updates
   - Use `Write::flush()` to force immediate display
   - Show percentage, bytes copied, and speed

3. **Handle copy errors**:
   - Disk full during write
   - Permission denied on destination
   - Source file deleted during copy
   - Resume or cleanup on failure

4. **Preserve metadata**:
   - Copy modification time using `filetime` crate
   - Copy permissions (Unix: mode bits)
   - Optionally copy owner/group


**Starter Code**:

```rust
use std::io::{BufReader, BufWriter, Read, Write};
use std::fs::File;

/// Copy file with buffered I/O
pub fn copy_file(src: &Path, dst: &Path) -> io::Result<()> {
    // TODO: Open source file with BufReader
    // TODO: Create destination file with BufWriter
    // TODO: Copy in chunks (use io::copy or manual loop)
    // TODO: Flush writer to ensure all data written

    // Hint:
    // let mut reader = BufReader::new(File::open(src)?);
    // let mut writer = BufWriter::new(File::create(dst)?);
    // io::copy(&mut reader, &mut writer)?;
    // writer.flush()?;

    todo!()
}

/// Copy file and preserve metadata (mtime, permissions)
pub fn copy_file_preserve_metadata(src: &Path, dst: &Path) -> io::Result<()> {
    // TODO: Copy file content
    // TODO: Get source metadata
    // TODO: Set destination modification time
    // TODO: Set destination permissions

    // Hint:
    // copy_file(src, dst)?;
    // let metadata = fs::metadata(src)?;
    // let mtime = metadata.modified()?;
    // filetime::set_file_mtime(dst, filetime::FileTime::from_system_time(mtime))?;
    // #[cfg(unix)]
    // fs::set_permissions(dst, metadata.permissions())?;

    todo!()
}

/// Copy file with progress callback
pub fn copy_file_with_progress<F>(
    src: &Path,
    dst: &Path,
    mut progress: F,
) -> io::Result<()>
where
    F: FnMut(u64, u64),
{
    // TODO: Get total file size
    // TODO: Open source and destination with buffering
    // TODO: Copy in chunks, calling progress callback after each chunk
    // TODO: Show percentage and speed

    // Hint:
    // let total_size = fs::metadata(src)?.len();
    // let mut reader = BufReader::new(File::open(src)?);
    // let mut writer = BufWriter::new(File::create(dst)?);
    // let mut buffer = [0u8; 8192];
    // let mut copied = 0u64;
    //
    // loop {
    //     let n = reader.read(&mut buffer)?;
    //     if n == 0 { break; }
    //     writer.write_all(&buffer[..n])?;
    //     copied += n as u64;
    //     progress(copied, total_size);
    // }

    todo!()
}

/// Display progress on stdout (same line, updates in place)
pub fn display_progress(path: &Path, bytes_copied: u64, total_bytes: u64) {
    // TODO: Calculate percentage
    // TODO: Format bytes (KB, MB, GB)
    // TODO: Print with \r to overwrite previous line
    // TODO: Use stdout().flush() to show immediately

    // Hint:
    // let percentage = (bytes_copied as f64 / total_bytes as f64) * 100.0;
    // print!("\r{}: {:.1}% ({} / {})",
    //     path.display(),
    //     percentage,
    //     format_bytes(bytes_copied),
    //     format_bytes(total_bytes)
    // );
    // std::io::stdout().flush().unwrap();

    todo!()
}

fn format_bytes(bytes: u64) -> String {
    // TODO: Format as KB, MB, GB
    // Hint: if bytes < 1024 { format!("{}B", bytes) }
    //       else if bytes < 1024*1024 { format!("{:.1}KB", bytes as f64 / 1024.0) }
    //       ...

    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_copy_file_with_buffer() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    let content = "test content".repeat(1000); // 12KB
    fs::write(src.path().join("file.txt"), &content).unwrap();

    copy_file(
        &src.path().join("file.txt"),
        &dst.path().join("file.txt"),
    )
    .unwrap();

    let copied = fs::read_to_string(dst.path().join("file.txt")).unwrap();
    assert_eq!(copied, content);
}

#[test]
fn test_preserve_modification_time() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("file.txt"), "content").unwrap();

    let src_file = src.path().join("file.txt");
    let dst_file = dst.path().join("file.txt");

    let src_mtime = fs::metadata(&src_file).unwrap().modified().unwrap();

    copy_file_preserve_metadata(&src_file, &dst_file).unwrap();

    let dst_mtime = fs::metadata(&dst_file).unwrap().modified().unwrap();

    assert_eq!(src_mtime, dst_mtime);
}

#[test]
#[cfg(unix)]
fn test_preserve_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    let src_file = src.path().join("file.txt");
    fs::write(&src_file, "content").unwrap();

    // Set specific permissions
    fs::set_permissions(&src_file, fs::Permissions::from_mode(0o644)).unwrap();

    copy_file_preserve_metadata(&src_file, &dst.path().join("file.txt")).unwrap();

    let dst_perms = fs::metadata(dst.path().join("file.txt"))
        .unwrap()
        .permissions()
        .mode();

    assert_eq!(dst_perms & 0o777, 0o644);
}

#[test]
fn test_copy_large_file() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    // Create 10MB file
    let content = vec![0u8; 10 * 1024 * 1024];
    fs::write(src.path().join("large.bin"), &content).unwrap();

    let progress = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let progress_clone = progress.clone();

    copy_file_with_progress(
        &src.path().join("large.bin"),
        &dst.path().join("large.bin"),
        |bytes_copied, total_bytes| {
            progress_clone.lock().unwrap().push((bytes_copied, total_bytes));
        },
    )
    .unwrap();

    // Verify file copied correctly
    let copied = fs::read(dst.path().join("large.bin")).unwrap();
    assert_eq!(copied.len(), content.len());

    // Verify progress was reported
    let progress_updates = progress.lock().unwrap();
    assert!(!progress_updates.is_empty());
    assert_eq!(progress_updates.last().unwrap().0, content.len() as u64);
}

#[test]
fn test_error_on_disk_full() {
    // Difficult to test portably, but document expected behavior:
    // - Should return io::Error with ErrorKind::StorageFull (or similar)
    // - Should not leave partial file (or mark it as incomplete)
    // - Should cleanup destination file on error
}
```

**Check Your Understanding**:
- Why use `BufReader`/`BufWriter` instead of raw `File`?
- What buffer size is optimal for file copying?
- Why does `\r` work for same-line updates?
- Why must we call `flush()` for immediate output?

---

### Milestone 5: Complete Sync Tool with Dry-Run

**Goal**: Integrate all features into a complete sync tool with CLI.

**Implementation Steps**:

1. **Build complete sync function**:
   - Build sync plan (Milestone 3)
   - Execute plan with progress (Milestone 4)
   - Handle errors for each file without aborting
   - Return summary (files copied, skipped, failed)

2. **Implement dry-run mode**:
   - Build sync plan but don't copy files
   - Print what *would* be done
   - Show file sizes and paths
   - Useful for previewing large syncs

3. **Add filtering options**:
   - Include/exclude patterns
   - Filter by file extension
   - Filter by size (e.g., skip files > 100MB)
   - Filter by date (e.g., only files modified in last 7 days)

4. **Create CLI interface**:
   - Parse command-line arguments
   - Options: `--dry-run`, `--checksum`, `--verbose`, `--exclude`
   - Show summary at end
   - Exit codes for success/failure

**Checkpoint Tests**:
```rust
#[test]
fn test_complete_sync() {
    let src = create_complex_test_tree();
    let dst = tempfile::tempdir().unwrap();

    let summary = sync_directories(
        src.path(),
        dst.path(),
        &SyncOptions::default(),
    )
    .unwrap();

    assert_eq!(summary.files_copied, 5);
    assert_eq!(summary.files_skipped, 0);
    assert_eq!(summary.files_failed, 0);

    // Verify all files copied
    assert!(dst.path().join("file1.txt").exists());
    assert!(dst.path().join("subdir/file2.txt").exists());
}

#[test]
fn test_incremental_sync() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    // First sync
    fs::write(src.path().join("file1.txt"), "content").unwrap();
    sync_directories(src.path(), dst.path(), &SyncOptions::default()).unwrap();

    // Add new file
    fs::write(src.path().join("file2.txt"), "new content").unwrap();

    // Second sync
    let summary = sync_directories(
        src.path(),
        dst.path(),
        &SyncOptions::default(),
    )
    .unwrap();

    assert_eq!(summary.files_copied, 1); // Only new file
    assert_eq!(summary.files_skipped, 1); // Original file skipped
}

#[test]
fn test_dry_run_mode() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("file.txt"), "content").unwrap();

    let options = SyncOptions {
        dry_run: true,
        ..Default::default()
    };

    let summary = sync_directories(src.path(), dst.path(), &options).unwrap();

    // Should report what would be copied
    assert_eq!(summary.files_copied, 1);

    // But not actually copy
    assert!(!dst.path().join("file.txt").exists());
}

#[test]
fn test_exclude_patterns() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("file.txt"), "include").unwrap();
    fs::write(src.path().join("file.log"), "exclude").unwrap();

    let options = SyncOptions {
        exclude_patterns: vec!["*.log".to_string()],
        ..Default::default()
    };

    sync_directories(src.path(), dst.path(), &options).unwrap();

    assert!(dst.path().join("file.txt").exists());
    assert!(!dst.path().join("file.log").exists());
}

#[test]
fn test_error_handling_continues() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("file1.txt"), "ok").unwrap();
    fs::write(src.path().join("file2.txt"), "ok").unwrap();

    // Create read-only destination directory
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir(dst.path().join("readonly")).unwrap();
        fs::set_permissions(
            dst.path().join("readonly"),
            fs::Permissions::from_mode(0o444),
        )
        .unwrap();
    }

    fs::write(src.path().join("readonly/file3.txt"), "fail").unwrap();

    let summary = sync_directories(
        src.path(),
        dst.path(),
        &SyncOptions::default(),
    )
    .unwrap();

    // Should copy successful files and report failure for readonly
    assert_eq!(summary.files_copied, 2);
    assert_eq!(summary.files_failed, 1);
}

#[test]
fn test_verbose_output() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    fs::write(src.path().join("file.txt"), "content").unwrap();

    let options = SyncOptions {
        verbose: true,
        ..Default::default()
    };

    // Capture stdout
    let summary = sync_directories(src.path(), dst.path(), &options).unwrap();

    // In real implementation, would verify output includes:
    // - "Copying file.txt"
    // - "1 file(s) copied, 0 skipped, 0 failed"
}
```

**Complete Implementation**:

```rust
use std::path::{Path, PathBuf};
use std::io;

#[derive(Debug, Default)]
pub struct SyncOptions {
    pub dry_run: bool,
    pub use_checksum: bool,
    pub verbose: bool,
    pub exclude_patterns: Vec<String>,
    pub max_file_size: Option<u64>,
}

#[derive(Debug, Default)]
pub struct SyncSummary {
    pub files_copied: usize,
    pub files_skipped: usize,
    pub files_failed: usize,
    pub bytes_copied: u64,
}

/// Synchronize source directory to destination
pub fn sync_directories(
    src: &Path,
    dst: &Path,
    options: &SyncOptions,
) -> io::Result<SyncSummary> {
    // TODO: Build sync plan
    // TODO: Filter plan based on options.exclude_patterns
    // TODO: For each item in plan:
    //   - If dry_run: print what would be done
    //   - If not dry_run: copy file with error handling
    // TODO: Accumulate summary statistics
    // TODO: Print summary if verbose

    todo!()
}

/// Execute sync plan (copy files)
fn execute_sync_plan(
    src_base: &Path,
    dst_base: &Path,
    plan: &[SyncItem],
    options: &SyncOptions,
) -> SyncSummary {
    let mut summary = SyncSummary::default();

    for item in plan {
        // TODO: Build full source and destination paths
        // TODO: Create destination directory if needed
        // TODO: Copy file with progress
        // TODO: Handle errors without aborting
        // TODO: Update summary

        if options.dry_run {
            // TODO: Print what would be done
            if options.verbose {
                println!("Would copy: {}", item.path.display());
            }
            summary.files_copied += 1;
        } else {
            // TODO: Actual copy
            match execute_sync_item(src_base, dst_base, item, options) {
                Ok(bytes) => {
                    summary.files_copied += 1;
                    summary.bytes_copied += bytes;
                    if options.verbose {
                        println!("Copied: {}", item.path.display());
                    }
                }
                Err(e) => {
                    summary.files_failed += 1;
                    eprintln!("Failed to copy {}: {}", item.path.display(), e);
                }
            }
        }
    }

    summary
}

fn execute_sync_item(
    src_base: &Path,
    dst_base: &Path,
    item: &SyncItem,
    options: &SyncOptions,
) -> io::Result<u64> {
    // TODO: Build paths
    let src_path = src_base.join(&item.path);
    let dst_path = dst_base.join(&item.path);

    // TODO: Create parent directory
    if let Some(parent) = dst_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // TODO: Copy file
    if options.verbose {
        copy_file_with_progress(&src_path, &dst_path, |copied, total| {
            display_progress(&item.path, copied, total);
        })?;
        println!(); // New line after progress
    } else {
        copy_file_preserve_metadata(&src_path, &dst_path)?;
    }

    // TODO: Return bytes copied
    Ok(fs::metadata(&src_path)?.len())
}

/// CLI main function
pub fn main() {
    // TODO: Parse command-line arguments
    // TODO: Validate arguments
    // TODO: Call sync_directories
    // TODO: Print summary
    // TODO: Exit with appropriate code

    // Example:
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 3 {
    //     eprintln!("Usage: filesync <source> <destination> [options]");
    //     std::process::exit(1);
    // }
    //
    // let src = Path::new(&args[1]);
    // let dst = Path::new(&args[2]);
    //
    // let options = parse_options(&args[3..]);
    //
    // match sync_directories(src, dst, &options) {
    //     Ok(summary) => {
    //         println!("Sync complete: {} copied, {} skipped, {} failed",
    //             summary.files_copied,
    //             summary.files_skipped,
    //             summary.files_failed
    //         );
    //     }
    //     Err(e) => {
    //         eprintln!("Sync failed: {}", e);
    //         std::process::exit(1);
    //     }
    // }
}
```

**Check Your Understanding**:
- Why continue processing files after one fails?
- How does dry-run mode help before large sync operations?
- What's the difference between verbose and normal output?
- How would you add parallel copying with thread pool?

---

### Complete Project Summary

**What You Built**:
1. Recursive directory traversal with error handling
2. Symlink cycle detection using canonical paths
3. Metadata comparison (timestamp, size, checksum)
4. Efficient buffered file copying with progress
5. Complete sync tool with dry-run and filtering
6. CLI with multiple options and error reporting

**Key Concepts Practiced**:
- Synchronous I/O patterns (file reading, writing)
- Directory traversal and file system operations
- Buffered I/O for performance (`BufReader`, `BufWriter`)
- Progress reporting with `flush()`
- Error handling and graceful degradation
- Metadata operations (timestamps, permissions)
- Pattern matching and filtering

**Performance Optimizations**:
- Only copy changed files (skip unnecessary transfers)
- Buffered I/O reduces syscall overhead
- Size check before checksum computation
- Parallel copying option (bonus)

**Real-World Applications**:
- Backup tools (Time Machine, Duplicati)
- Deployment systems (Capistrano, Ansible)
- Build tools (Cargo, npm)
- Cloud sync clients (Dropbox, Google Drive)
- Content distribution (CDN sync)

**Extension Ideas**:
1. **Network sync**: Sync over SSH/SFTP
2. **Incremental backups**: Keep multiple versions
3. **Compression**: Compress during transfer
4. **Parallel copying**: Use thread pool for concurrent copies
5. **Watch mode**: Continuously sync on file changes
6. **Two-way sync**: Bidirectional synchronization
7. **Conflict resolution**: Handle both sides modified
8. **Resume support**: Resume interrupted transfers
9. **Bandwidth limiting**: Throttle copy speed
10. **Database tracking**: Store sync state in SQLite

This project teaches the core patterns used in production file synchronization tools while demonstrating efficient I/O, error handling, and user experience design!
