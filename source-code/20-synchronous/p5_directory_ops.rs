// Pattern 5: Directory Operations
use std::fs;
use std::io;
use std::path::Path;

// Create directory
fn create_directory(path: &str) -> io::Result<()> {
    fs::create_dir(path)
    // Fails if parent doesn't exist
    // Fails if directory already exists
}

// Create directory and all parent directories (like mkdir -p)
fn create_directory_all(path: &str) -> io::Result<()> {
    fs::create_dir_all(path)
    // Creates parent directories as needed
    // Succeeds if directory already exists
}

// Remove empty directory
fn remove_directory(path: &str) -> io::Result<()> {
    fs::remove_dir(path)
    // Fails if directory is not empty
}

// Remove directory and all contents (dangerous!)
fn remove_directory_all(path: &str) -> io::Result<()> {
    fs::remove_dir_all(path)
    // Recursively deletes everything
    // Like rm -rf in Unix
}

// Check if path exists
fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
    // Returns false for broken symlinks
}

// Check if path is directory
fn is_directory(path: &str) -> bool {
    Path::new(path).is_dir()
    // Follows symlinks
}

fn main() -> io::Result<()> {
    println!("=== Directory Operations Demo ===\n");

    // Create a single directory
    println!("=== create_directory ===");
    let single_dir = "test_single_dir";
    create_directory(single_dir)?;
    println!("Created: {}", single_dir);
    println!("Exists: {}", path_exists(single_dir));
    println!("Is directory: {}", is_directory(single_dir));

    // Try to create again - should fail
    match create_directory(single_dir) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (already exists): {}", e.kind()),
    }

    // Create nested directory structure
    println!("\n=== create_directory_all ===");
    let nested_dir = "test_nested/level1/level2/level3";
    create_directory_all(nested_dir)?;
    println!("Created nested: {}", nested_dir);
    println!("Exists: {}", path_exists(nested_dir));

    // Idempotent - calling again succeeds
    create_directory_all(nested_dir)?;
    println!("Called again (idempotent): OK");

    // Check existence and type
    println!("\n=== path_exists and is_directory ===");
    println!("'{}' exists: {}", single_dir, path_exists(single_dir));
    println!("'{}' is_dir: {}", single_dir, is_directory(single_dir));
    println!("'nonexistent' exists: {}", path_exists("nonexistent"));
    println!("'nonexistent' is_dir: {}", is_directory("nonexistent"));

    // Remove empty directory
    println!("\n=== remove_directory (empty only) ===");
    remove_directory(single_dir)?;
    println!("Removed empty directory: {}", single_dir);
    println!("Exists after removal: {}", path_exists(single_dir));

    // Try to remove non-empty directory - should fail
    println!("\n=== remove_directory on non-empty ===");
    match remove_directory("test_nested") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error (not empty): {}", e.kind()),
    }

    // Remove directory and all contents
    println!("\n=== remove_directory_all (DANGEROUS) ===");
    remove_directory_all("test_nested")?;
    println!("Recursively removed: test_nested");
    println!("Exists after removal: {}", path_exists("test_nested"));

    println!("\nDirectory operations examples completed");
    Ok(())
}
