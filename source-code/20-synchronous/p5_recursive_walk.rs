// Pattern 5: Recursive Directory Walking
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Recursive file listing (depth-first search)
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

// Get all files recursively
fn get_all_files(path: &str) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_directory(Path::new(path), &mut files)?;
    Ok(files)
}

// Recursive directory tree printer (visual tree)
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

// Find files matching pattern (like find command)
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

fn main() -> io::Result<()> {
    println!("=== Recursive Directory Walking Demo ===\n");

    // Create test directory structure
    let test_dir = "test_walk";
    fs::create_dir_all(format!("{}/src/utils", test_dir))?;
    fs::create_dir_all(format!("{}/tests", test_dir))?;
    fs::create_dir_all(format!("{}/docs", test_dir))?;

    fs::write(format!("{}/index.md", test_dir), "# Project")?;
    fs::write(format!("{}/Cargo.toml", test_dir), "[package]")?;
    fs::write(format!("{}/src/main.rs", test_dir), "fn main() {}")?;
    fs::write(format!("{}/src/lib.rs", test_dir), "pub mod utils;")?;
    fs::write(format!("{}/src/utils/helpers.rs", test_dir), "pub fn help() {}")?;
    fs::write(format!("{}/tests/test_main.rs", test_dir), "#[test]")?;
    fs::write(format!("{}/tests/test_utils.rs", test_dir), "#[test]")?;
    fs::write(format!("{}/docs/guide.md", test_dir), "# Guide")?;

    // Print tree structure
    println!("=== print_tree ===");
    println!("{}", test_dir);
    print_tree(Path::new(test_dir), "")?;

    // Get all files recursively
    println!("\n=== get_all_files ===");
    let all_files = get_all_files(test_dir)?;
    println!("All files ({} total):", all_files.len());
    for file in &all_files {
        println!("  {}", file.display());
    }

    // Find files matching pattern
    println!("\n=== find_files (pattern: 'test') ===");
    let matches = find_files(Path::new(test_dir), "test")?;
    println!("Files containing 'test' ({} matches):", matches.len());
    for file in &matches {
        println!("  {}", file.display());
    }

    println!("\n=== find_files (pattern: '.rs') ===");
    let rs_files = find_files(Path::new(test_dir), ".rs")?;
    println!("Rust source files ({} matches):", rs_files.len());
    for file in &rs_files {
        println!("  {}", file.display());
    }

    println!("\n=== find_files (pattern: '.md') ===");
    let md_files = find_files(Path::new(test_dir), ".md")?;
    println!("Markdown files ({} matches):", md_files.len());
    for file in &md_files {
        println!("  {}", file.display());
    }

    // Cleanup
    fs::remove_dir_all(test_dir)?;

    println!("\nRecursive walk examples completed");
    Ok(())
}
