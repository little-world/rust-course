// Pattern 6: Mock and Stub Patterns - Test Doubles for I/O
// Demonstrates fake filesystem implementation for testing.

use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// Example: Test Doubles for I/O
// ============================================================================

trait FileSystem {
    fn read_file(&self, path: &str) -> std::io::Result<String>;
    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()>;
}

// Real implementation
struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_file(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()> {
        std::fs::write(path, content)
    }
}

// In-memory fake for testing
struct FakeFileSystem {
    files: Mutex<HashMap<String, String>>,
}

impl FakeFileSystem {
    fn new() -> Self {
        FakeFileSystem {
            files: Mutex::new(HashMap::new()),
        }
    }

    fn with_file(self, path: &str, content: &str) -> Self {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_string());
        self
    }
}

impl FileSystem for FakeFileSystem {
    fn read_file(&self, path: &str) -> std::io::Result<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found"
            ))
    }

    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_string());
        Ok(())
    }
}

// Application code that uses the filesystem
struct ConfigLoader<F: FileSystem> {
    fs: F,
}

impl<F: FileSystem> ConfigLoader<F> {
    fn new(fs: F) -> Self {
        ConfigLoader { fs }
    }

    fn load_config(&self, path: &str) -> std::io::Result<String> {
        let content = self.fs.read_file(path)?;
        // In real code, you might parse JSON/TOML here
        Ok(content.trim().to_string())
    }

    fn save_config(&self, path: &str, config: &str) -> std::io::Result<()> {
        self.fs.write_file(path, config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations() {
        let fs = FakeFileSystem::new();

        fs.write_file("/test.txt", "hello").unwrap();
        let content = fs.read_file("/test.txt").unwrap();

        assert_eq!(content, "hello");
    }

    #[test]
    fn test_file_not_found() {
        let fs = FakeFileSystem::new();

        let result = fs.read_file("/nonexistent.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_loader_with_fake() {
        let fs = FakeFileSystem::new()
            .with_file("/config.json", "{ \"debug\": true }");

        let loader = ConfigLoader::new(fs);
        let config = loader.load_config("/config.json").unwrap();

        assert_eq!(config, "{ \"debug\": true }");
    }

    #[test]
    fn test_config_save_and_load() {
        let fs = FakeFileSystem::new();
        let loader = ConfigLoader::new(fs);

        loader.save_config("/config.json", "new config").unwrap();
        let config = loader.load_config("/config.json").unwrap();

        assert_eq!(config, "new config");
    }
}

fn main() {
    // Demo with fake filesystem
    let fake_fs = FakeFileSystem::new();
    let loader = ConfigLoader::new(fake_fs);

    println!("Test Doubles Demo:");
    loader.save_config("/app/config.json", "{ \"mode\": \"test\" }").unwrap();
    let config = loader.load_config("/app/config.json").unwrap();
    println!("Loaded config: {}", config);

    // Demo with real filesystem (commented out to avoid file operations)
    // let real_fs = RealFileSystem;
    // let real_loader = ConfigLoader::new(real_fs);
    // let config = real_loader.load_config("config.json")?;
}
