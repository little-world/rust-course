use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut total_example_count = 0;

    println!("Searching for 'Example' in markdown files from: {:?}", current_dir);

    process_directory(&current_dir, &mut total_example_count)?;

    println!(
        "\nTotal occurrences of 'Example' in all markdown files: {}",
        total_example_count
    );

    Ok(())
}

fn process_directory(dir_path: &Path, total_count: &mut usize) -> io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            process_directory(&path, total_count)?;
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Process markdown files
            if let Ok(count) = count_example_in_file(&path) {
                *total_count += count;
                println!("  Processed {:?}: Found {} 'Example'(s)", path, count);
            } else {
                eprintln!("  Error reading file: {:?}", path);
            }
        }
    }
    Ok(())
}

fn count_example_in_file(file_path: &Path) -> io::Result<usize> {
    let mut file = fs::File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents.matches("Example").count())
}
