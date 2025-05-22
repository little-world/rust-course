use std::{env, fs};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::error::Error;

/// Reads all files in `dir`, sorts them by filename,
/// computes keys by stripping leading numbers/dash and extension,
/// and writes `key=filename` lines into `file`.
fn write_properties(dir: &Path, file: &mut impl Write) -> io::Result<()> {
    // Collect (filename, key) tuples
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().is_file() {
            let file_name = entry.file_name()
                .into_string()
                .unwrap_or_else(|_| String::from("<invalid UTF-8>"));
            // Compute key: remove leading segment before first '-' and drop extension
            let key = {
                let name = if let Some(pos) = file_name.find('-') {
                    &file_name[pos + 1..]
                } else {
                    &file_name[..]
                };
                if let Some(dot_pos) = name.rfind('.') {
                    &name[..dot_pos]
                } else {
                    name
                }
            }.to_string();
            entries.push((file_name.clone(), key));
        }
    }
    // Sort by filename ascending
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    // Write each as a property
    for (file_name, key) in entries {
        writeln!(file, "{}={}", key, file_name)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Expect one argument: the root directory containing subdirectories
    let root = "docs";
    let root_path = Path::new(&root);

    if !root_path.is_dir() {
        eprintln!("Error: '{}' is not a directory", root);
        std::process::exit(1);
    }

    // For each subdirectory, create a <dirname>.properties file
    for entry in fs::read_dir(root_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            // Create the .toc file in the root directory
            let output_path = root_path.join(format!("{}.toc", dir_name));
            let mut file = File::create(&output_path)?;
            write_properties(&path, &mut file)?;
            println!("Wrote properties to {}", output_path.display());
        }
    }

    Ok(())
}