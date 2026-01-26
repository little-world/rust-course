// Pattern 5: Directory Listing
use std::fs;
use std::io;
use std::path::PathBuf;

// List directory contents
fn list_directory(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;  // Each entry can fail
        entries.push(entry.path());
    }

    Ok(entries)
}

// List only files (skip directories)
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

// List files with specific extension
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

fn main() -> io::Result<()> {
    println!("=== Directory Listing Demo ===\n");

    // Create test directory structure
    let test_dir = "test_listing";
    fs::create_dir_all(test_dir)?;
    fs::write(format!("{}/file1.txt", test_dir), "content1")?;
    fs::write(format!("{}/file2.txt", test_dir), "content2")?;
    fs::write(format!("{}/data.json", test_dir), "{}")?;
    fs::write(format!("{}/image.png", test_dir), &[0x89, 0x50, 0x4E, 0x47])?;
    fs::create_dir(format!("{}/subdir", test_dir))?;
    fs::write(format!("{}/subdir/nested.txt", test_dir), "nested")?;

    // List all entries
    println!("=== list_directory ===");
    let entries = list_directory(test_dir)?;
    println!("All entries in '{}':", test_dir);
    for entry in &entries {
        println!("  {:?}", entry.file_name().unwrap());
    }

    // List only files
    println!("\n=== list_files_only ===");
    let files = list_files_only(test_dir)?;
    println!("Files only (no directories):");
    for file in &files {
        println!("  {:?}", file.file_name().unwrap());
    }

    // List by extension
    println!("\n=== list_by_extension (txt) ===");
    let txt_files = list_by_extension(test_dir, "txt")?;
    println!(".txt files:");
    for file in &txt_files {
        println!("  {:?}", file.file_name().unwrap());
    }

    println!("\n=== list_by_extension (json) ===");
    let json_files = list_by_extension(test_dir, "json")?;
    println!(".json files:");
    for file in &json_files {
        println!("  {:?}", file.file_name().unwrap());
    }

    // List with metadata
    println!("\n=== list_with_metadata ===");
    let items = list_with_metadata(test_dir)?;
    println!("Entries with metadata:");
    for (path, metadata) in &items {
        let file_type = if metadata.is_dir() { "DIR " } else { "FILE" };
        println!("  {} {:>6} bytes  {:?}",
            file_type,
            metadata.len(),
            path.file_name().unwrap()
        );
    }

    // Cleanup
    fs::remove_dir_all(test_dir)?;

    println!("\nDirectory listing examples completed");
    Ok(())
}
