// Pattern 5: AsRef for Read-Only Access
use std::path::Path;

// Accept anything that can be viewed as a Path
fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

fn use_asref() {
    // All of these work:
    println!("config.txt exists: {}", file_exists("config.txt"));           // &str
    println!("log.txt exists: {}", file_exists(String::from("log.txt"))); // String
    println!("data.bin exists: {}", file_exists(Path::new("data.bin")));   // &Path

    use std::path::PathBuf;
    println!("/tmp exists: {}", file_exists(PathBuf::from("/tmp")));   // PathBuf
}

// For strings, use AsRef<str>
fn count_words(text: impl AsRef<str>) -> usize {
    text.as_ref().split_whitespace().count()
}

fn use_asref_str() {
    println!("Word count: {}", count_words("hello world"));           // &str
    println!("Word count: {}", count_words(String::from("hi there"))); // String
}

fn main() {
    use_asref();
    use_asref_str();
    println!("AsRef example completed");
}
