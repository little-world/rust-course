
## Project: Version Control System (Git Clone)

### Problem Statement

Build a functional version control system similar to Git that tracks file changes, manages commits, handles branching, and synchronizes repositories. You'll implement the core Git commands: `init`, `add`, `commit`, `log`, `checkout`, `branch`, `clone`, and `push`, learning how distributed version control works under the hood.

### Use Cases

**When you need this pattern**:
1. **Code versioning**: Track changes to source code over time
2. **Collaboration**: Multiple developers working on same codebase
3. **Backup and recovery**: Revert to previous working versions
4. **Experimentation**: Create branches for new features
5. **History tracking**: Understand why and when changes were made
6. **Distributed workflows**: Work offline, sync later

### Why It Matters

**Real-World Impact**: Version control is fundamental to all software development:

**The Manual Backup Problem**:
```bash
# Inefficient manual versioning:
project/
  main.rs           # Current version
  main_backup.rs    # Yesterday's version
  main_old.rs       # Last week's version
  main_final.rs     # "Final" version
  main_final2.rs    # Actually final version
  main_REALLY_FINAL.rs  # OK this is the real one

# Problems:
# - No metadata (who changed what, when, why?)
# - Can't compare versions easily
# - No branching for experiments
# - Can't collaborate without conflicts
# - Wastes disk space (full copies)
```

**Version Control Benefits**:
- **Complete history**: Every change tracked with author, time, message
- **Branching**: Experiment without affecting main code
- **Diffing**: See exactly what changed between versions
- **Collaboration**: Merge changes from multiple developers
- **Space efficient**: Store only deltas (changes), not full copies
- **Distributed**: Every developer has full history locally

**How Git Works Internally**:
```
.git/
  objects/          # All file versions stored as content-addressed blobs
    a1/b2c3...      # Blob: file content
    d4e5f6...       # Tree: directory structure
    789abc...       # Commit: snapshot + metadata
  refs/
    heads/
      main          # Branch pointer to commit
      feature       # Another branch
    remotes/
      origin/main   # Remote tracking branch
  HEAD              # Current branch pointer
  index             # Staging area
```

**Key Git Concepts**:
1. **Content-addressable storage**: Files stored by SHA-1 hash of content
2. **Immutable objects**: Once created, objects never change
3. **Snapshots, not diffs**: Each commit is full snapshot, deltas computed on-demand
4. **Directed acyclic graph**: Commits form DAG with parent pointers
5. **Branches are pointers**: Lightweight, just point to commits

**Performance**:
- **Space**: Compression + delta encoding saves 10x space vs full copies
- **Speed**: SHA-1 hashing enables fast duplicate detection
- **Network**: Only transfer missing objects on push/pull
- **Local operations**: Most commands instant (no network needed)

### Learning Goals

By completing this project, you will:

1. **Understand Git internals**: How objects, refs, and HEAD work
2. **Content-addressable storage**: Hash-based file systems
3. **Graph algorithms**: DAG traversal for commit history
4. **File I/O patterns**: Efficient file reading, writing, compression
5. **Serialization**: Store objects in custom binary format
6. **Directory operations**: Recursive tree traversal and comparison
7. **Networking basics**: Clone and push over filesystem (simulating remote)

---

### Project Structure

```
mygit/
  src/
    main.rs           # CLI entry point
    lib.rs            # Public API
    objects.rs        # Blob, Tree, Commit objects
    repository.rs     # Repository operations
    index.rs          # Staging area
    refs.rs           # Branch and HEAD management
    diff.rs           # Computing diffs between versions
    hash.rs           # SHA-1 hashing utilities
  .mygit/             # Repository metadata (like .git/)
    objects/          # Object database
    refs/
      heads/          # Local branches
      remotes/        # Remote tracking branches
    HEAD              # Current branch
    index             # Staging area
```

---

### Milestone 1: Repository Initialization and Object Storage

**Goal**: Create repository structure and implement content-addressable object storage.

**Implementation Steps**:

1. **Implement `mygit init`**:
   - Create `.mygit` directory structure
   - Initialize `objects/`, `refs/heads/`, `refs/remotes/`
   - Create `HEAD` file pointing to `refs/heads/main`
   - Create empty `index` file

2. **Implement Git objects**:
   - **Blob**: Raw file content
   - **Tree**: Directory listing (file names → blob hashes)
   - **Commit**: Snapshot with metadata (tree hash, parent, author, message)

3. **Implement SHA-1 hashing**:
   - Compute SHA-1 hash of object content
   - Use hash as filename: `objects/ab/cdef1234...`
   - Store objects in compressed form (optional: use flate2)

4. **Implement object storage**:
   - Write objects to `.mygit/objects/`
   - Read objects from disk
   - Handle object not found errors

**Checkpoint Tests**:
```rust
use std::path::Path;
use std::fs;

#[test]
fn test_init_repository() {
    let temp = tempfile::tempdir().unwrap();

    mygit::init(temp.path()).unwrap();

    assert!(temp.path().join(".mygit").exists());
    assert!(temp.path().join(".mygit/objects").exists());
    assert!(temp.path().join(".mygit/refs/heads").exists());
    assert!(temp.path().join(".mygit/HEAD").exists());

    let head_content = fs::read_to_string(temp.path().join(".mygit/HEAD")).unwrap();
    assert_eq!(head_content, "ref: refs/heads/main\n");
}

#[test]
fn test_create_blob_object() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();

    let repo = Repository::open(temp.path()).unwrap();
    let content = b"Hello, World!";

    let hash = repo.create_blob(content).unwrap();

    // Hash should be deterministic
    assert_eq!(hash.len(), 40); // SHA-1 is 40 hex chars

    // Should be able to read it back
    let blob = repo.read_blob(&hash).unwrap();
    assert_eq!(blob, content);
}

#[test]
fn test_blob_deduplication() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    let content = b"Same content";
    let hash1 = repo.create_blob(content).unwrap();
    let hash2 = repo.create_blob(content).unwrap();

    // Same content should produce same hash
    assert_eq!(hash1, hash2);

    // Should only be stored once
    let object_path = repo.object_path(&hash1);
    assert!(object_path.exists());
}

#[test]
fn test_create_tree_object() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    let blob_hash = repo.create_blob(b"file content").unwrap();

    let mut tree = Tree::new();
    tree.add_entry("file.txt", TreeEntry::Blob(blob_hash));

    let tree_hash = repo.create_tree(&tree).unwrap();

    // Should be able to read it back
    let read_tree = repo.read_tree(&tree_hash).unwrap();
    assert_eq!(read_tree.entries.len(), 1);
}

#[test]
fn test_create_commit_object() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    let tree_hash = repo.create_tree(&Tree::new()).unwrap();

    let commit = Commit {
        tree: tree_hash,
        parent: None,
        author: "Alice <alice@example.com>".to_string(),
        timestamp: SystemTime::now(),
        message: "Initial commit".to_string(),
    };

    let commit_hash = repo.create_commit(&commit).unwrap();

    let read_commit = repo.read_commit(&commit_hash).unwrap();
    assert_eq!(read_commit.message, "Initial commit");
    assert_eq!(read_commit.parent, None);
}
```

**Starter Code**:

```rust
// src/objects.rs
use std::collections::BTreeMap;
use std::time::SystemTime;

/// A blob stores raw file content
#[derive(Debug, Clone)]
pub struct Blob {
    pub content: Vec<u8>,
}

/// A tree entry can be a blob (file) or another tree (subdirectory)
#[derive(Debug, Clone)]
pub enum TreeEntry {
    Blob(String),  // Hash of blob
    Tree(String),  // Hash of subtree
}

/// A tree represents a directory structure
#[derive(Debug, Clone)]
pub struct Tree {
    pub entries: BTreeMap<String, TreeEntry>,  // BTreeMap for deterministic ordering
}

impl Tree {
    pub fn new() -> Self {
        // TODO: Create empty tree
        todo!()
    }

    pub fn add_entry(&mut self, name: &str, entry: TreeEntry) {
        // TODO: Add entry to tree
        todo!()
    }

    /// Serialize tree to bytes
    pub fn serialize(&self) -> Vec<u8> {
        // TODO: Format as: "filename\0type hash\n..."
        // Example: "file.txt\0blob a1b2c3...\nsubdir\0tree d4e5f6...\n"
        todo!()
    }

    /// Deserialize tree from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        // TODO: Parse serialized format
        todo!()
    }
}

/// A commit represents a snapshot with metadata
#[derive(Debug, Clone)]
pub struct Commit {
    pub tree: String,              // Hash of tree
    pub parent: Option<String>,    // Hash of parent commit (None for first commit)
    pub author: String,
    pub timestamp: SystemTime,
    pub message: String,
}

impl Commit {
    /// Serialize commit to bytes
    pub fn serialize(&self) -> Vec<u8> {
        // TODO: Format as:
        // tree <hash>\n
        // parent <hash>\n (if exists)
        // author <author>\n
        // timestamp <unix_timestamp>\n
        // \n
        // <message>
        todo!()
    }

    /// Deserialize commit from bytes
    pub fn deserialize(data: &[u8]) -> Result<Self, String> {
        // TODO: Parse serialized format
        todo!()
    }
}
```

```rust
// src/hash.rs
use sha1::{Sha1, Digest};

/// Compute SHA-1 hash of data
pub fn hash_object(data: &[u8]) -> String {
    // TODO: Compute SHA-1 hash
    // TODO: Return as lowercase hex string
    // Hint: let mut hasher = Sha1::new();
    //       hasher.update(data);
    //       format!("{:x}", hasher.finalize())
    todo!()
}

/// Hash with object type prefix (like Git does)
pub fn hash_with_type(obj_type: &str, data: &[u8]) -> String {
    // TODO: Prepend type and size: "blob 13\0content"
    // Git format: "<type> <size>\0<content>"
    todo!()
}
```

```rust
// src/repository.rs
use std::path::{Path, PathBuf};
use std::fs;
use std::io;

pub struct Repository {
    git_dir: PathBuf,   // Path to .mygit directory
}

impl Repository {
    /// Initialize a new repository
    pub fn init(path: &Path) -> io::Result<Self> {
        let git_dir = path.join(".mygit");

        // TODO: Create directory structure
        // fs::create_dir_all(git_dir.join("objects"))?;
        // fs::create_dir_all(git_dir.join("refs/heads"))?;
        // fs::create_dir_all(git_dir.join("refs/remotes"))?;

        // TODO: Create HEAD file pointing to main branch
        // fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

        // TODO: Create empty index
        // fs::write(git_dir.join("index"), "")?;

        todo!()
    }

    /// Open existing repository
    pub fn open(path: &Path) -> io::Result<Self> {
        let git_dir = path.join(".mygit");
        if !git_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Not a mygit repository"
            ));
        }
        Ok(Repository { git_dir })
    }

    /// Create and store a blob object
    pub fn create_blob(&self, content: &[u8]) -> io::Result<String> {
        // TODO: Hash content
        // TODO: Store in objects/ab/cdef123...
        // TODO: Return hash

        // Hint: let hash = hash_object(content);
        //       let object_path = self.object_path(&hash);
        //       fs::create_dir_all(object_path.parent().unwrap())?;
        //       fs::write(object_path, content)?;

        todo!()
    }

    /// Read a blob object
    pub fn read_blob(&self, hash: &str) -> io::Result<Vec<u8>> {
        // TODO: Read from objects/ab/cdef123...
        let object_path = self.object_path(hash);
        fs::read(object_path)
    }

    /// Get path for object with given hash
    fn object_path(&self, hash: &str) -> PathBuf {
        // TODO: Split hash: objects/ab/cdef123...
        // Git stores objects as: objects/<first 2 chars>/<remaining chars>

        // Hint: self.git_dir.join("objects")
        //           .join(&hash[0..2])
        //           .join(&hash[2..])

        todo!()
    }

    /// Create and store a tree object
    pub fn create_tree(&self, tree: &Tree) -> io::Result<String> {
        // TODO: Serialize tree
        // TODO: Hash serialized content
        // TODO: Store in objects/
        todo!()
    }

    /// Read a tree object
    pub fn read_tree(&self, hash: &str) -> io::Result<Tree> {
        // TODO: Read object
        // TODO: Deserialize as tree
        todo!()
    }

    /// Create and store a commit object
    pub fn create_commit(&self, commit: &Commit) -> io::Result<String> {
        // TODO: Serialize commit
        // TODO: Hash and store
        todo!()
    }

    /// Read a commit object
    pub fn read_commit(&self, hash: &str) -> io::Result<Commit> {
        // TODO: Read and deserialize
        todo!()
    }
}
```

**Check Your Understanding**:
- Why use SHA-1 hash as filename instead of sequential IDs?
- Why split object storage into subdirectories (`ab/cdef...`)?
- How does content-addressable storage enable deduplication?
- What's the Git object format with type prefix?

---

### Milestone 2: Staging Area and Commit

**Goal**: Implement `add` and `commit` commands with staging area.

**Implementation Steps**:

1. **Implement staging area (index)**:
   - Store mapping: filename → blob hash
   - Serialize/deserialize index to `.mygit/index`
   - Track which files are staged for commit

2. **Implement `mygit add <file>`**:
   - Read file content from working directory
   - Create blob object with content
   - Add filename → blob hash to index
   - Handle directories recursively

3. **Implement tree building from index**:
   - Convert flat index into tree hierarchy
   - Handle nested directories
   - Create tree objects for each directory

4. **Implement `mygit commit -m "message"`**:
   - Build tree from current index
   - Get current branch and parent commit
   - Create commit object
   - Update branch pointer to new commit
   - Clear staging area (optional)

**Checkpoint Tests**:
```rust
#[test]
fn test_add_single_file() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();

    // Create test file
    fs::write(temp.path().join("file.txt"), "content").unwrap();

    let repo = Repository::open(temp.path()).unwrap();
    repo.add("file.txt").unwrap();

    // File should be in index
    let index = repo.read_index().unwrap();
    assert!(index.contains_key("file.txt"));
}

#[test]
fn test_add_multiple_files() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();

    fs::write(temp.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp.path().join("file2.txt"), "content2").unwrap();

    let repo = Repository::open(temp.path()).unwrap();
    repo.add("file1.txt").unwrap();
    repo.add("file2.txt").unwrap();

    let index = repo.read_index().unwrap();
    assert_eq!(index.len(), 2);
}

#[test]
fn test_add_directory() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();

    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp.path().join("src/lib.rs"), "pub fn foo() {}").unwrap();

    let repo = Repository::open(temp.path()).unwrap();
    repo.add("src").unwrap();

    let index = repo.read_index().unwrap();
    assert!(index.contains_key("src/main.rs"));
    assert!(index.contains_key("src/lib.rs"));
}

#[test]
fn test_first_commit() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();

    fs::write(temp.path().join("README.md"), "# Project").unwrap();

    let repo = Repository::open(temp.path()).unwrap();
    repo.add("README.md").unwrap();

    let commit_hash = repo.commit("Initial commit", "Alice <alice@example.com>").unwrap();

    // Commit should exist
    let commit = repo.read_commit(&commit_hash).unwrap();
    assert_eq!(commit.message, "Initial commit");
    assert_eq!(commit.parent, None);

    // Branch should point to commit
    let main_hash = repo.read_ref("refs/heads/main").unwrap();
    assert_eq!(main_hash, commit_hash);
}

#[test]
fn test_second_commit_has_parent() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // First commit
    fs::write(temp.path().join("file1.txt"), "v1").unwrap();
    repo.add("file1.txt").unwrap();
    let commit1 = repo.commit("First", "Alice <alice@example.com>").unwrap();

    // Second commit
    fs::write(temp.path().join("file2.txt"), "v2").unwrap();
    repo.add("file2.txt").unwrap();
    let commit2 = repo.commit("Second", "Alice <alice@example.com>").unwrap();

    // Second commit should have first as parent
    let commit = repo.read_commit(&commit2).unwrap();
    assert_eq!(commit.parent, Some(commit1));
}

#[test]
fn test_build_tree_from_index() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    fs::create_dir_all(temp.path().join("src/utils")).unwrap();
    fs::write(temp.path().join("README.md"), "readme").unwrap();
    fs::write(temp.path().join("src/main.rs"), "main").unwrap();
    fs::write(temp.path().join("src/utils/helper.rs"), "helper").unwrap();

    repo.add(".").unwrap(); // Add all files

    let tree_hash = repo.build_tree_from_index().unwrap();
    let tree = repo.read_tree(&tree_hash).unwrap();

    // Root tree should have README.md and src/
    assert!(tree.entries.contains_key("README.md"));
    assert!(tree.entries.contains_key("src"));
}
```

**Starter Code Extension**:

```rust
// src/index.rs
use std::collections::HashMap;
use std::path::Path;
use std::io;
use std::fs;

pub type Index = HashMap<String, String>;  // filename -> blob hash

/// Read index from disk
pub fn read_index(git_dir: &Path) -> io::Result<Index> {
    let index_path = git_dir.join("index");

    if !index_path.exists() {
        return Ok(HashMap::new());
    }

    // TODO: Read and deserialize index
    // Format: "filename\0hash\n..."

    todo!()
}

/// Write index to disk
pub fn write_index(git_dir: &Path, index: &Index) -> io::Result<()> {
    // TODO: Serialize and write index
    // Format: "filename\0hash\n..."

    todo!()
}
```

```rust
// src/repository.rs (additions)

impl Repository {
    /// Add file(s) to staging area
    pub fn add(&self, path: &str) -> io::Result<()> {
        let full_path = self.work_dir().join(path);

        if !full_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path not found: {}", path)
            ));
        }

        let mut index = read_index(&self.git_dir)?;

        if full_path.is_file() {
            // TODO: Add single file
            // let content = fs::read(&full_path)?;
            // let hash = self.create_blob(&content)?;
            // index.insert(path.to_string(), hash);
        } else if full_path.is_dir() {
            // TODO: Add directory recursively
            // Walk directory, create blobs for all files
        }

        write_index(&self.git_dir, &index)?;

        todo!()
    }

    /// Build tree from current index
    pub fn build_tree_from_index(&self) -> io::Result<String> {
        let index = read_index(&self.git_dir)?;

        // TODO: Convert flat index to tree hierarchy
        // Example index:
        //   "README.md" -> blob_hash1
        //   "src/main.rs" -> blob_hash2
        //   "src/lib.rs" -> blob_hash3
        //
        // Should build:
        //   root_tree:
        //     README.md -> blob_hash1
        //     src -> src_tree_hash
        //   src_tree:
        //     main.rs -> blob_hash2
        //     lib.rs -> blob_hash3

        todo!()
    }

    /// Create a commit
    pub fn commit(&self, message: &str, author: &str) -> io::Result<String> {
        // TODO: Build tree from index
        // TODO: Get current branch
        // TODO: Get parent commit (head of current branch)
        // TODO: Create commit object
        // TODO: Update branch reference

        // Hint:
        // let tree_hash = self.build_tree_from_index()?;
        // let current_branch = self.current_branch()?;
        // let parent = self.read_ref(&current_branch).ok();
        // let commit = Commit { tree: tree_hash, parent, ... };
        // let commit_hash = self.create_commit(&commit)?;
        // self.update_ref(&current_branch, &commit_hash)?;

        todo!()
    }

    fn work_dir(&self) -> &Path {
        self.git_dir.parent().unwrap()
    }
}
```

```rust
// src/refs.rs
use std::path::Path;
use std::io;
use std::fs;

/// Read a reference (branch or tag)
pub fn read_ref(git_dir: &Path, ref_name: &str) -> io::Result<String> {
    // TODO: Read refs/heads/main or refs/tags/v1.0
    let ref_path = git_dir.join(ref_name);

    if !ref_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Reference not found: {}", ref_name)
        ));
    }

    let content = fs::read_to_string(ref_path)?;
    Ok(content.trim().to_string())
}

/// Update a reference to point to a commit
pub fn update_ref(git_dir: &Path, ref_name: &str, commit_hash: &str) -> io::Result<()> {
    // TODO: Write commit hash to refs/heads/main
    let ref_path = git_dir.join(ref_name);

    // Create parent directories if needed
    if let Some(parent) = ref_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(ref_path, format!("{}\n", commit_hash))?;
    Ok(())
}

/// Get current branch name
pub fn current_branch(git_dir: &Path) -> io::Result<String> {
    // TODO: Read HEAD file
    // If contains "ref: refs/heads/main", return "refs/heads/main"
    // If contains a hash, return the hash (detached HEAD)

    let head_content = fs::read_to_string(git_dir.join("HEAD"))?;

    if head_content.starts_with("ref: ") {
        Ok(head_content.trim().strip_prefix("ref: ").unwrap().to_string())
    } else {
        Ok(head_content.trim().to_string())
    }
}
```

**Check Your Understanding**:
- Why have a staging area instead of committing working directory directly?
- How do we handle nested directories in the index?
- What's the difference between a flat index and tree hierarchy?
- Why update the branch pointer on commit?

---

### Milestone 3: Commit History and Checkout

**Goal**: View commit history and restore previous versions.

**Implementation Steps**:

1. **Implement `mygit log`**:
   - Start from current commit (HEAD)
   - Follow parent pointers backwards
   - Display commit hash, author, timestamp, message
   - Stop when reaching initial commit (no parent)

2. **Implement commit traversal**:
   - Walk commit DAG from any starting point
   - Handle merge commits (multiple parents)
   - Topological ordering of commits

3. **Implement `mygit checkout <commit>`**:
   - Read commit object
   - Extract tree from commit
   - Write tree contents to working directory
   - Update HEAD to point to commit (detached HEAD)
   - Handle uncommitted changes (warn user)

4. **Implement tree extraction**:
   - Recursively extract tree objects
   - Write blobs to files
   - Preserve directory structure
   - Restore file permissions (optional)

**Checkpoint Tests**:
```rust
#[test]
fn test_log_single_commit() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    fs::write(temp.path().join("file.txt"), "v1").unwrap();
    repo.add("file.txt").unwrap();
    let commit_hash = repo.commit("First", "Alice <alice@example.com>").unwrap();

    let log = repo.log(None).unwrap();

    assert_eq!(log.len(), 1);
    assert_eq!(log[0].hash, commit_hash);
    assert_eq!(log[0].message, "First");
}

#[test]
fn test_log_multiple_commits() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // Create 3 commits
    for i in 1..=3 {
        fs::write(temp.path().join("file.txt"), format!("v{}", i)).unwrap();
        repo.add("file.txt").unwrap();
        repo.commit(&format!("Commit {}", i), "Alice <alice@example.com>").unwrap();
    }

    let log = repo.log(None).unwrap();

    assert_eq!(log.len(), 3);
    assert_eq!(log[0].message, "Commit 3"); // Most recent first
    assert_eq!(log[1].message, "Commit 2");
    assert_eq!(log[2].message, "Commit 1");
}

#[test]
fn test_checkout_previous_version() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // First commit
    fs::write(temp.path().join("file.txt"), "version 1").unwrap();
    repo.add("file.txt").unwrap();
    let commit1 = repo.commit("First", "Alice <alice@example.com>").unwrap();

    // Second commit
    fs::write(temp.path().join("file.txt"), "version 2").unwrap();
    repo.add("file.txt").unwrap();
    repo.commit("Second", "Alice <alice@example.com>").unwrap();

    // Checkout first commit
    repo.checkout(&commit1).unwrap();

    // File should contain version 1
    let content = fs::read_to_string(temp.path().join("file.txt")).unwrap();
    assert_eq!(content, "version 1");
}

#[test]
fn test_checkout_restores_deleted_files() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // Commit with file
    fs::write(temp.path().join("file.txt"), "content").unwrap();
    repo.add("file.txt").unwrap();
    let commit1 = repo.commit("Add file", "Alice <alice@example.com>").unwrap();

    // Delete file and commit
    fs::remove_file(temp.path().join("file.txt")).unwrap();
    repo.commit("Delete file", "Alice <alice@example.com>").unwrap();

    // Checkout first commit
    repo.checkout(&commit1).unwrap();

    // File should be restored
    assert!(temp.path().join("file.txt").exists());
}

#[test]
fn test_checkout_directory_structure() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(temp.path().join("README.md"), "# Project").unwrap();

    repo.add(".").unwrap();
    let commit = repo.commit("Initial", "Alice <alice@example.com>").unwrap();

    // Delete everything
    fs::remove_dir_all(temp.path().join("src")).unwrap();
    fs::remove_file(temp.path().join("README.md")).unwrap();

    // Checkout
    repo.checkout(&commit).unwrap();

    // Everything should be restored
    assert!(temp.path().join("src/main.rs").exists());
    assert!(temp.path().join("README.md").exists());
}
```

**Starter Code Extension**:

```rust
// src/repository.rs (additions)

#[derive(Debug)]
pub struct LogEntry {
    pub hash: String,
    pub author: String,
    pub timestamp: SystemTime,
    pub message: String,
    pub parent: Option<String>,
}

impl Repository {
    /// Get commit history starting from given commit (or HEAD if None)
    pub fn log(&self, start: Option<&str>) -> io::Result<Vec<LogEntry>> {
        // TODO: Get starting commit (HEAD if not specified)
        // TODO: Walk backwards following parent pointers
        // TODO: Collect log entries

        let mut log = Vec::new();
        let mut current = match start {
            Some(hash) => hash.to_string(),
            None => {
                let branch = current_branch(&self.git_dir)?;
                read_ref(&self.git_dir, &branch)?
            }
        };

        loop {
            // TODO: Read commit
            // TODO: Add to log
            // TODO: Follow parent pointer
            // TODO: Stop when no parent

            let commit = self.read_commit(&current)?;

            log.push(LogEntry {
                hash: current.clone(),
                author: commit.author.clone(),
                timestamp: commit.timestamp,
                message: commit.message.clone(),
                parent: commit.parent.clone(),
            });

            match commit.parent {
                Some(parent) => current = parent,
                None => break,
            }
        }

        Ok(log)
    }

    /// Checkout a specific commit
    pub fn checkout(&self, commit_hash: &str) -> io::Result<()> {
        // TODO: Read commit
        // TODO: Get tree from commit
        // TODO: Clear working directory (except .mygit)
        // TODO: Extract tree to working directory
        // TODO: Update HEAD to commit (detached HEAD)

        let commit = self.read_commit(commit_hash)?;
        let tree = self.read_tree(&commit.tree)?;

        // Clear working directory
        self.clear_working_directory()?;

        // Extract tree
        self.extract_tree(&tree, self.work_dir())?;

        // Update HEAD (detached)
        fs::write(self.git_dir.join("HEAD"), format!("{}\n", commit_hash))?;

        Ok(())
    }

    /// Extract tree to directory
    fn extract_tree(&self, tree: &Tree, target_dir: &Path) -> io::Result<()> {
        // TODO: For each entry in tree:
        //   - If blob: write file
        //   - If tree: create directory and recurse

        for (name, entry) in &tree.entries {
            let target_path = target_dir.join(name);

            match entry {
                TreeEntry::Blob(hash) => {
                    // TODO: Read blob and write to file
                    let content = self.read_blob(hash)?;
                    fs::write(target_path, content)?;
                }
                TreeEntry::Tree(hash) => {
                    // TODO: Create directory and extract subtree
                    fs::create_dir_all(&target_path)?;
                    let subtree = self.read_tree(hash)?;
                    self.extract_tree(&subtree, &target_path)?;
                }
            }
        }

        Ok(())
    }

    /// Clear working directory (except .mygit)
    fn clear_working_directory(&self) -> io::Result<()> {
        // TODO: Remove all files and directories except .mygit

        for entry in fs::read_dir(self.work_dir())? {
            let entry = entry?;
            let path = entry.path();

            if path.file_name().unwrap() == ".mygit" {
                continue;
            }

            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }

        Ok(())
    }
}
```

**Check Your Understanding**:
- Why walk backwards from HEAD instead of forwards from initial commit?
- What is a "detached HEAD" state?
- How do we handle merge commits with multiple parents?
- Why clear the working directory before checkout?

---

### Milestone 4: Branching and Merging

**Goal**: Create and switch between branches, merge changes.

**Implementation Steps**:

1. **Implement `mygit branch <name>`**:
   - Create new branch pointing to current commit
   - Store in `refs/heads/<name>`
   - Don't switch to new branch (just create)

2. **Implement `mygit checkout <branch>`**:
   - Switch to existing branch
   - Update HEAD to `ref: refs/heads/<branch>`
   - Extract branch's commit to working directory

3. **Implement `mygit merge <branch>`**:
   - Find common ancestor (merge base)
   - Three-way merge: base, current, other
   - Handle conflicts (simple strategy: fail if conflicts)
   - Create merge commit with two parents

4. **Implement simple merge strategies**:
   - Fast-forward: Current is ancestor of other
   - Three-way merge: Changes from both branches
   - Conflict detection: Same file modified in both

**Checkpoint Tests**:
```rust
#[test]
fn test_create_branch() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // Create initial commit
    fs::write(temp.path().join("file.txt"), "v1").unwrap();
    repo.add("file.txt").unwrap();
    let commit1 = repo.commit("First", "Alice <alice@example.com>").unwrap();

    // Create branch
    repo.create_branch("feature").unwrap();

    // Branch should exist and point to current commit
    let branch_hash = repo.read_ref("refs/heads/feature").unwrap();
    assert_eq!(branch_hash, commit1);
}

#[test]
fn test_switch_branch() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    fs::write(temp.path().join("file.txt"), "v1").unwrap();
    repo.add("file.txt").unwrap();
    repo.commit("First", "Alice <alice@example.com>").unwrap();

    repo.create_branch("feature").unwrap();
    repo.checkout_branch("feature").unwrap();

    // HEAD should point to feature branch
    let head = fs::read_to_string(temp.path().join(".mygit/HEAD")).unwrap();
    assert_eq!(head.trim(), "ref: refs/heads/feature");
}

#[test]
fn test_branch_diverges() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // Initial commit on main
    fs::write(temp.path().join("file.txt"), "v1").unwrap();
    repo.add("file.txt").unwrap();
    repo.commit("First", "Alice <alice@example.com>").unwrap();

    // Create and switch to feature branch
    repo.create_branch("feature").unwrap();
    repo.checkout_branch("feature").unwrap();

    // Commit on feature
    fs::write(temp.path().join("feature.txt"), "feature work").unwrap();
    repo.add("feature.txt").unwrap();
    let feature_commit = repo.commit("Feature work", "Alice <alice@example.com>").unwrap();

    // Switch back to main
    repo.checkout_branch("main").unwrap();

    // Commit on main
    fs::write(temp.path().join("main.txt"), "main work").unwrap();
    repo.add("main.txt").unwrap();
    let main_commit = repo.commit("Main work", "Alice <alice@example.com>").unwrap();

    // Branches should point to different commits
    assert_ne!(feature_commit, main_commit);
}

#[test]
fn test_fast_forward_merge() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // Commit on main
    fs::write(temp.path().join("file.txt"), "v1").unwrap();
    repo.add("file.txt").unwrap();
    repo.commit("First", "Alice <alice@example.com>").unwrap();

    // Create feature branch and add commit
    repo.create_branch("feature").unwrap();
    repo.checkout_branch("feature").unwrap();
    fs::write(temp.path().join("feature.txt"), "feature").unwrap();
    repo.add("feature.txt").unwrap();
    let feature_commit = repo.commit("Feature", "Alice <alice@example.com>").unwrap();

    // Switch to main and merge
    repo.checkout_branch("main").unwrap();
    repo.merge("feature").unwrap();

    // Main should now point to feature commit (fast-forward)
    let main_hash = repo.read_ref("refs/heads/main").unwrap();
    assert_eq!(main_hash, feature_commit);
}

#[test]
fn test_three_way_merge() {
    let temp = tempfile::tempdir().unwrap();
    mygit::init(temp.path()).unwrap();
    let repo = Repository::open(temp.path()).unwrap();

    // Base commit
    fs::write(temp.path().join("file.txt"), "base").unwrap();
    repo.add("file.txt").unwrap();
    repo.commit("Base", "Alice <alice@example.com>").unwrap();

    // Feature branch changes
    repo.create_branch("feature").unwrap();
    repo.checkout_branch("feature").unwrap();
    fs::write(temp.path().join("feature.txt"), "feature").unwrap();
    repo.add("feature.txt").unwrap();
    repo.commit("Feature work", "Alice <alice@example.com>").unwrap();

    // Main branch changes
    repo.checkout_branch("main").unwrap();
    fs::write(temp.path().join("main.txt"), "main").unwrap();
    repo.add("main.txt").unwrap();
    repo.commit("Main work", "Alice <alice@example.com>").unwrap();

    // Merge feature into main
    repo.merge("feature").unwrap();

    // Working directory should have both files
    assert!(temp.path().join("feature.txt").exists());
    assert!(temp.path().join("main.txt").exists());
}
```

**Starter Code Extension**:

```rust
// src/repository.rs (additions)

impl Repository {
    /// Create a new branch
    pub fn create_branch(&self, name: &str) -> io::Result<()> {
        // TODO: Get current commit
        // TODO: Create refs/heads/<name> pointing to it

        let current = self.current_commit()?;
        update_ref(&self.git_dir, &format!("refs/heads/{}", name), &current)?;
        Ok(())
    }

    /// Switch to a branch
    pub fn checkout_branch(&self, name: &str) -> io::Result<()> {
        // TODO: Verify branch exists
        // TODO: Get commit from branch
        // TODO: Checkout commit
        // TODO: Update HEAD to point to branch

        let branch_ref = format!("refs/heads/{}", name);
        let commit = read_ref(&self.git_dir, &branch_ref)?;

        self.checkout(&commit)?;

        // Update HEAD to point to branch
        fs::write(
            self.git_dir.join("HEAD"),
            format!("ref: {}\n", branch_ref)
        )?;

        Ok(())
    }

    /// Get current commit hash
    fn current_commit(&self) -> io::Result<String> {
        let head = fs::read_to_string(self.git_dir.join("HEAD"))?;

        if head.starts_with("ref: ") {
            // HEAD points to branch
            let branch = head.trim().strip_prefix("ref: ").unwrap();
            read_ref(&self.git_dir, branch)
        } else {
            // Detached HEAD
            Ok(head.trim().to_string())
        }
    }

    /// Merge another branch into current branch
    pub fn merge(&self, branch_name: &str) -> io::Result<()> {
        // TODO: Get current commit
        // TODO: Get other branch commit
        // TODO: Find merge base (common ancestor)
        // TODO: Determine merge strategy:
        //   - If current == base: fast-forward to other
        //   - If other == base: already up to date
        //   - Else: three-way merge

        let current = self.current_commit()?;
        let other = read_ref(&self.git_dir, &format!("refs/heads/{}", branch_name))?;

        if current == other {
            println!("Already up to date");
            return Ok(());
        }

        // Find merge base
        let base = self.find_merge_base(&current, &other)?;

        if base == current {
            // Fast-forward merge
            println!("Fast-forward merge");
            let current_branch = current_branch(&self.git_dir)?;
            update_ref(&self.git_dir, &current_branch, &other)?;
            self.checkout(&other)?;
        } else if base == other {
            println!("Already up to date");
        } else {
            // Three-way merge
            println!("Three-way merge");
            self.three_way_merge(&base, &current, &other, branch_name)?;
        }

        Ok(())
    }

    /// Find common ancestor of two commits
    fn find_merge_base(&self, commit1: &str, commit2: &str) -> io::Result<String> {
        // TODO: Get all ancestors of commit1
        // TODO: Walk commit2 backwards until finding common ancestor

        let ancestors1 = self.get_ancestors(commit1)?;
        let mut current = commit2.to_string();

        loop {
            if ancestors1.contains(&current) {
                return Ok(current);
            }

            let commit = self.read_commit(&current)?;
            match commit.parent {
                Some(parent) => current = parent,
                None => break,
            }
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "No common ancestor found"
        ))
    }

    /// Get all ancestors of a commit
    fn get_ancestors(&self, start: &str) -> io::Result<HashSet<String>> {
        let mut ancestors = HashSet::new();
        let mut current = start.to_string();

        loop {
            ancestors.insert(current.clone());

            let commit = self.read_commit(&current)?;
            match commit.parent {
                Some(parent) => current = parent,
                None => break,
            }
        }

        Ok(ancestors)
    }

    /// Perform three-way merge
    fn three_way_merge(
        &self,
        base: &str,
        current: &str,
        other: &str,
        other_branch: &str,
    ) -> io::Result<()> {
        // TODO: Get trees for base, current, other
        // TODO: Compare trees to find changes
        // TODO: Apply changes from both branches
        // TODO: Detect conflicts
        // TODO: Create merge commit with two parents

        // Simple strategy: Extract other's tree on top of current
        // In real Git, this would do proper three-way diff

        let other_commit = self.read_commit(other)?;
        let other_tree = self.read_tree(&other_commit.tree)?;

        self.extract_tree(&other_tree, self.work_dir())?;

        // Stage all changes
        self.add(".")?;

        // Create merge commit
        let tree_hash = self.build_tree_from_index()?;
        let commit = Commit {
            tree: tree_hash,
            parent: Some(current.to_string()),
            // TODO: Add second parent for merge commit
            author: "System <system@example.com>".to_string(),
            timestamp: SystemTime::now(),
            message: format!("Merge branch '{}'", other_branch),
        };

        let commit_hash = self.create_commit(&commit)?;
        let current_branch = current_branch(&self.git_dir)?;
        update_ref(&self.git_dir, &current_branch, &commit_hash)?;

        Ok(())
    }
}
```

**Check Your Understanding**:
- What's the difference between a branch and a tag?
- How does Git determine if a merge can be fast-forwarded?
- What is a merge base and how do we find it?
- Why do merge commits have two parents?

---

### Milestone 5: Clone and Push (Remote Operations)

**Goal**: Clone repositories and push changes (filesystem-based remote).

**Implementation Steps**:

1. **Implement `mygit clone <source> <destination>`**:
   - Copy entire `.mygit` directory
   - Extract HEAD commit to working directory
   - Set up remote tracking (refs/remotes/origin/main)
   - Configure remote URL in config file

2. **Implement remote tracking**:
   - Store remote refs in `refs/remotes/origin/*`
   - Track which remote branch local branches follow
   - Update remote refs on fetch/pull

3. **Implement `mygit push`**:
   - Find commits that remote doesn't have
   - Copy missing objects to remote
   - Update remote branch pointer
   - Handle push rejection (remote has newer commits)

4. **Implement `mygit pull`**:
   - Fetch objects from remote
   - Merge remote branch into current branch
   - Update remote tracking refs

**Checkpoint Tests**:
```rust
#[test]
fn test_clone_repository() {
    let source = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();

    // Create source repo with commit
    mygit::init(source.path()).unwrap();
    let repo = Repository::open(source.path()).unwrap();
    fs::write(source.path().join("file.txt"), "content").unwrap();
    repo.add("file.txt").unwrap();
    repo.commit("Initial", "Alice <alice@example.com>").unwrap();

    // Clone
    mygit::clone(source.path(), dest.path()).unwrap();

    // Destination should have .mygit and working files
    assert!(dest.path().join(".mygit").exists());
    assert!(dest.path().join("file.txt").exists());

    let content = fs::read_to_string(dest.path().join("file.txt")).unwrap();
    assert_eq!(content, "content");
}

#[test]
fn test_clone_preserves_history() {
    let source = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();

    mygit::init(source.path()).unwrap();
    let repo = Repository::open(source.path()).unwrap();

    // Create multiple commits
    for i in 1..=3 {
        fs::write(source.path().join("file.txt"), format!("v{}", i)).unwrap();
        repo.add("file.txt").unwrap();
        repo.commit(&format!("Commit {}", i), "Alice <alice@example.com>").unwrap();
    }

    // Clone
    mygit::clone(source.path(), dest.path()).unwrap();

    // Check history in cloned repo
    let cloned = Repository::open(dest.path()).unwrap();
    let log = cloned.log(None).unwrap();
    assert_eq!(log.len(), 3);
}

#[test]
fn test_push_new_commits() {
    let remote = tempfile::tempdir().unwrap();
    let local = tempfile::tempdir().unwrap();

    // Setup remote
    mygit::init(remote.path()).unwrap();
    let remote_repo = Repository::open(remote.path()).unwrap();
    fs::write(remote.path().join("file.txt"), "v1").unwrap();
    remote_repo.add("file.txt").unwrap();
    remote_repo.commit("First", "Alice <alice@example.com>").unwrap();

    // Clone
    mygit::clone(remote.path(), local.path()).unwrap();

    // Make local commit
    let local_repo = Repository::open(local.path()).unwrap();
    fs::write(local.path().join("file.txt"), "v2").unwrap();
    local_repo.add("file.txt").unwrap();
    local_repo.commit("Second", "Bob <bob@example.com>").unwrap();

    // Push
    local_repo.push().unwrap();

    // Remote should have new commit
    let remote_log = remote_repo.log(None).unwrap();
    assert_eq!(remote_log.len(), 2);
    assert_eq!(remote_log[0].message, "Second");
}

#[test]
fn test_push_transfers_objects() {
    let remote = tempfile::tempdir().unwrap();
    let local = tempfile::tempdir().unwrap();

    mygit::init(remote.path()).unwrap();
    mygit::clone(remote.path(), local.path()).unwrap();

    // Create file in local
    let local_repo = Repository::open(local.path()).unwrap();
    fs::write(local.path().join("new.txt"), "new content").unwrap();
    local_repo.add("new.txt").unwrap();
    local_repo.commit("Add new file", "Alice <alice@example.com>").unwrap();

    // Push
    local_repo.push().unwrap();

    // Checkout in remote should work
    let remote_repo = Repository::open(remote.path()).unwrap();
    let commit = remote_repo.current_commit().unwrap();
    remote_repo.checkout(&commit).unwrap();

    assert!(remote.path().join("new.txt").exists());
}
```

**Final Implementation**:

```rust
// src/repository.rs (additions)

impl Repository {
    /// Clone a repository
    pub fn clone(source: &Path, dest: &Path) -> io::Result<Self> {
        // TODO: Create destination directory
        fs::create_dir_all(dest)?;

        // TODO: Initialize new repo
        let repo = Self::init(dest)?;

        // TODO: Copy all objects from source
        let source_git = source.join(".mygit");
        copy_directory(&source_git.join("objects"), &repo.git_dir.join("objects"))?;

        // TODO: Copy refs
        copy_directory(&source_git.join("refs"), &repo.git_dir.join("refs"))?;

        // TODO: Copy HEAD
        fs::copy(source_git.join("HEAD"), repo.git_dir.join("HEAD"))?;

        // TODO: Set up remote tracking
        fs::write(
            repo.git_dir.join("config"),
            format!("remote = {}\n", source.display())
        )?;

        // TODO: Checkout HEAD
        let head_commit = repo.current_commit()?;
        repo.checkout(&head_commit)?;

        Ok(repo)
    }

    /// Push commits to remote
    pub fn push(&self) -> io::Result<()> {
        // TODO: Read remote path from config
        let config = fs::read_to_string(self.git_dir.join("config"))?;
        let remote_path = config
            .lines()
            .find(|l| l.starts_with("remote = "))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No remote configured"))?
            .strip_prefix("remote = ")
            .unwrap();

        let remote_git = Path::new(remote_path).join(".mygit");

        // TODO: Get current commit
        let local_commit = self.current_commit()?;

        // TODO: Get remote's current commit
        let remote_repo = Repository::open(Path::new(remote_path))?;
        let remote_commit = remote_repo.current_commit().ok();

        // TODO: Find commits to push (all commits from remote to local)
        let commits_to_push = self.find_commits_to_push(&local_commit, remote_commit.as_deref())?;

        // TODO: Copy missing objects
        for commit_hash in &commits_to_push {
            self.copy_object_to_remote(commit_hash, &remote_git)?;

            // Also copy tree and blobs
            let commit = self.read_commit(commit_hash)?;
            self.copy_tree_to_remote(&commit.tree, &remote_git)?;
        }

        // TODO: Update remote branch
        let current_branch = current_branch(&self.git_dir)?;
        let branch_name = current_branch.strip_prefix("refs/heads/").unwrap();
        update_ref(&remote_git, &format!("refs/heads/{}", branch_name), &local_commit)?;

        println!("Pushed {} commit(s)", commits_to_push.len());

        Ok(())
    }

    fn find_commits_to_push(
        &self,
        local: &str,
        remote: Option<&str>,
    ) -> io::Result<Vec<String>> {
        // TODO: Walk from local back to remote
        // TODO: Collect all commits in between

        let mut commits = Vec::new();
        let mut current = local.to_string();

        loop {
            if Some(current.as_str()) == remote {
                break;
            }

            commits.push(current.clone());

            let commit = self.read_commit(&current)?;
            match commit.parent {
                Some(parent) => current = parent,
                None => break,
            }
        }

        commits.reverse(); // Push oldest first
        Ok(commits)
    }

    fn copy_object_to_remote(&self, hash: &str, remote_git: &Path) -> io::Result<()> {
        let src = self.object_path(hash);
        let dst = remote_git.join("objects")
            .join(&hash[0..2])
            .join(&hash[2..]);

        if dst.exists() {
            return Ok(()); // Already exists
        }

        fs::create_dir_all(dst.parent().unwrap())?;
        fs::copy(src, dst)?;
        Ok(())
    }

    fn copy_tree_to_remote(&self, tree_hash: &str, remote_git: &Path) -> io::Result<()> {
        self.copy_object_to_remote(tree_hash, remote_git)?;

        let tree = self.read_tree(tree_hash)?;
        for entry in tree.entries.values() {
            match entry {
                TreeEntry::Blob(hash) => {
                    self.copy_object_to_remote(hash, remote_git)?;
                }
                TreeEntry::Tree(hash) => {
                    self.copy_tree_to_remote(hash, remote_git)?;
                }
            }
        }

        Ok(())
    }
}

fn copy_directory(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_directory(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
```

**Check Your Understanding**:
- Why copy the entire `.mygit` directory for clone?
- How do we determine which commits to push?
- What happens if remote has commits we don't have?
- How would we implement fetch (download without merging)?

---

### Complete Project Summary

**What You Built**:
1. Repository initialization and object storage (content-addressable)
2. Staging area and commit creation
3. Commit history traversal and checkout
4. Branching and merging (fast-forward and three-way)
5. Clone and push operations
6. Complete version control system matching core Git features

**Key Git Concepts Implemented**:
- Content-addressable storage (SHA-1 hashing)
- Immutable objects (blobs, trees, commits)
- DAG structure (commits with parent pointers)
- Branches as lightweight pointers
- Staging area (index)
- Three-way merge algorithm
- Distributed architecture (clone/push)

**File I/O Patterns Used**:
- Directory traversal (`fs::read_dir`)
- File metadata operations
- Content hashing (SHA-1)
- Buffered I/O for object storage
- Recursive tree operations
- Atomic file operations

**Real-World Applications**:
- Understanding Git internals
- Building custom VCS tools
- Content-addressable storage systems
- Backup systems with deduplication
- Distributed synchronization

**Extension Ideas**:
1. **Compression**: Use flate2 to compress objects
2. **Pack files**: Store multiple objects in single file
3. **Network protocol**: Clone/push over HTTP or SSH
4. **Conflict resolution**: Interactive merge conflict handling
5. **Rebase**: Replay commits on top of another branch
6. **Stash**: Temporarily save uncommitted changes
7. **Tags**: Named pointers to commits
8. **Submodules**: Nested repositories
9. **Hooks**: Run scripts on commit, push, etc.
10. **Garbage collection**: Remove unreachable objects

**Performance Characteristics**:
- O(1) object lookup by hash
- O(N) commit traversal (N = commits)
- O(M) tree extraction (M = files)
- Space-efficient through deduplication
- Fast branching (just pointer creation)

This project teaches the core architecture of Git while practicing file I/O, hashing, graph algorithms, and distributed system concepts!
