#!/usr/bin/env rust-script

use std::fs;
use std::path::Path;

fn main() {
    let cookbook_dir = "docs/cookbook";
    
    // Get all markdown files in the cookbook directory
    let entries = fs::read_dir(cookbook_dir).expect("Failed to read cookbook directory");
    
    for entry in entries[0..3] {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        
        // Process only markdown files
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            println!("Processing: {:?}", path);
            process_file(&path);
        }
    }
    
    println!("\nDone!");
}

fn process_file(path: &Path) {
    let content = fs::read_to_string(path).expect("Failed to read file");
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_rust_block = false;
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i];
        
        // Detect rust code block start/end
        if line.trim().starts_with("```rust") {
            in_rust_block = true;
            result.push(line.to_string());
            i += 1;
            continue;
        } else if line.trim() == "```" && in_rust_block {
            in_rust_block = false;
            result.push(line.to_string());
            i += 1;
            continue;
        }
        
        // Process lines inside rust blocks
        if in_rust_block {

            if line.contains("  //===")  {
                println!("removing: {}. \t{}", i , line)
            }
        }
        
        result.push(line.to_string());
        i += 1;
    }
    
    // Write back to file
    let new_content = result.join("\n");
    // Add final newline if original had one
    let final_content = if content.ends_with('\n') {
        format!("{}\n", new_content)
    } else {
        new_content
    };
    
    fs::write(path, final_content).expect("Failed to write file");
}

fn is_underline_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("//") {
        return false;
    }
    
    // Check if it's just // followed by = characters
    let after_slashes = &trimmed[2..];
    !after_slashes.is_empty() && after_slashes.trim() == after_slashes.replace(|c: char| c != '=', "")
}

fn create_underline(comment: &str) -> String {
    // Count the number of characters in the comment
    let length = comment.len();
    
    // Create underline with = characters matching the length
    format!("//{}", "=".repeat(length - 2))
}
