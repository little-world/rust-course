// Anti-Patterns Part 3: Safety Anti-Patterns
// Demonstrates safety mistakes and their correct solutions.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// Anti-Pattern: Unsafe for Convenience
// ============================================================================

mod unsafe_convenience {
    // ANTI-PATTERN: Unsafe to "fix" borrow checker errors
    // This shows what NOT to do - creates undefined behavior
    #[allow(dead_code)]
    struct CacheBad {
        data: Vec<String>,
    }

    #[allow(dead_code)]
    impl CacheBad {
        fn get_mut_two_bad(&mut self, i: usize, j: usize) -> (&mut String, &mut String) {
            // "I know what I'm doing" famous last words
            unsafe {
                let ptr = self.data.as_mut_ptr();
                (&mut *ptr.add(i), &mut *ptr.add(j))
            }
            // What if i == j? Undefined behavior!
        }
    }

    // CORRECT: Safe solution
    pub struct Cache {
        pub data: Vec<String>,
    }

    impl Cache {
        pub fn new(data: Vec<String>) -> Self {
            Cache { data }
        }

        pub fn get_mut_two(
            &mut self,
            i: usize,
            j: usize,
        ) -> Option<(&mut String, &mut String)> {
            if i == j {
                return None; // Can't return two mutable refs to same element
            }

            if i >= self.data.len() || j >= self.data.len() {
                return None;
            }

            // Safe split_at_mut
            if i < j {
                let (left, right) = self.data.split_at_mut(j);
                Some((&mut left[i], &mut right[0]))
            } else {
                let (left, right) = self.data.split_at_mut(i);
                Some((&mut right[0], &mut left[j]))
            }
        }

        pub fn get_mut_two_unchecked(
            &mut self,
            i: usize,
            j: usize,
        ) -> (&mut String, &mut String) {
            assert!(i != j);
            assert!(i < self.data.len());
            assert!(j < self.data.len());

            // Still use safe split_at_mut
            if i < j {
                let (left, right) = self.data.split_at_mut(j);
                (&mut left[i], &mut right[0])
            } else {
                let (left, right) = self.data.split_at_mut(i);
                (&mut right[0], &mut left[j])
            }
        }
    }

    pub fn demo() {
        println!("=== Unsafe for Convenience Anti-Pattern ===");
        let mut cache = Cache::new(vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]);

        if let Some((a, b)) = cache.get_mut_two(0, 2) {
            *a = "modified first".to_string();
            *b = "modified third".to_string();
            println!("Modified two elements safely");
        }

        println!("Cache data: {:?}", cache.data);

        // Same index returns None
        if cache.get_mut_two(1, 1).is_none() {
            println!("Correctly rejected same index access");
        }
    }
}

// ============================================================================
// Anti-Pattern: Unwrap() in Production Code
// ============================================================================

mod unwrap_production {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Config {
        pub name: String,
        pub value: i32,
    }

    #[derive(Debug, Clone)]
    pub struct User {
        pub age: u32,
    }

    #[derive(Debug)]
    pub enum ConfigError {
        Io(std::io::Error),
        Parse(serde_json::Error),
    }

    impl From<std::io::Error> for ConfigError {
        fn from(err: std::io::Error) -> Self {
            ConfigError::Io(err)
        }
    }

    impl From<serde_json::Error> for ConfigError {
        fn from(err: serde_json::Error) -> Self {
            ConfigError::Parse(err)
        }
    }

    // ANTI-PATTERN: Unwrap everywhere
    #[allow(dead_code)]
    fn load_config_bad(_path: &str) -> Config {
        // let contents = std::fs::read_to_string(path).unwrap();  // Panics if file missing
        // serde_json::from_str(&contents).unwrap()  // Panics if invalid JSON
        panic!("This is an example of what NOT to do");
    }

    // CORRECT: Proper error handling
    pub fn load_config(json_str: &str) -> Result<Config, ConfigError> {
        let config: Config = serde_json::from_str(json_str)?;
        Ok(config)
    }

    pub fn get_user_age(users: &HashMap<String, User>, id: &str) -> Option<u32> {
        users.get(id).map(|user| user.age)
    }

    pub fn divide(a: i32, b: i32) -> Option<i32> {
        a.checked_div(b)
    }

    // If unwrap is genuinely safe, document why
    pub fn get_first_line(text: &str) -> &str {
        // Handle empty string case - lines() returns empty iterator for ""
        if text.is_empty() {
            return "";
        }
        text.lines()
            .next()
            .unwrap() // Safe: non-empty text always has at least one line
    }

    pub fn demo() {
        println!("\n=== Unwrap() in Production Code Anti-Pattern ===");

        // Good error handling
        let valid_json = r#"{"name": "test", "value": 42}"#;
        match load_config(valid_json) {
            Ok(config) => println!("Loaded config: {:?}", config),
            Err(e) => println!("Error: {:?}", e),
        }

        let invalid_json = r#"{"name": "test"}"#;
        match load_config(invalid_json) {
            Ok(config) => println!("Loaded config: {:?}", config),
            Err(e) => println!("Expected error for invalid JSON: {:?}", e),
        }

        // Option handling
        let mut users = HashMap::new();
        users.insert("alice".to_string(), User { age: 30 });

        println!("Alice's age: {:?}", get_user_age(&users, "alice"));
        println!("Bob's age: {:?}", get_user_age(&users, "bob"));

        // Division
        println!("10 / 2 = {:?}", divide(10, 2));
        println!("10 / 0 = {:?}", divide(10, 0));

        // Safe unwrap with documentation
        println!("First line: '{}'", get_first_line("Hello\nWorld"));
        println!("First line of empty: '{}'", get_first_line(""));
    }
}

// ============================================================================
// Anti-Pattern: RefCell/Mutex Without Consideration
// ============================================================================

mod refcell_mutex {
    use super::*;

    // ANTI-PATTERN: RefCell everywhere
    #[allow(dead_code)]
    struct ApplicationBad {
        state: RefCell<AppState>,
        config: RefCell<Config>,
    }

    #[allow(dead_code)]
    #[derive(Default)]
    struct AppState {
        counter: i32,
    }

    #[allow(dead_code)]
    #[derive(Default)]
    struct Config {
        name: String,
    }

    // CORRECT: Proper mutability
    pub struct Application {
        state: AppState,
        config: Config,
    }

    impl Application {
        pub fn new() -> Self {
            Application {
                state: AppState::default(),
                config: Config::default(),
            }
        }

        pub fn process(&mut self) {
            // Honest about mutation
            self.state.counter += 1;
            println!("Counter: {}", self.state.counter);
        }

        pub fn read_config(&self) -> &Config {
            &self.config // No runtime overhead
        }
    }

    // Use RefCell only when necessary (e.g., graph structures, caching)
    pub struct Node {
        pub value: i32,
        pub children: RefCell<Vec<Node>>, // Justified: allows mutation during traversal
    }

    impl Node {
        pub fn new(value: i32) -> Self {
            Node {
                value,
                children: RefCell::new(Vec::new()),
            }
        }

        pub fn add_child(&self, child: Node) {
            self.children.borrow_mut().push(child);
        }

        pub fn child_count(&self) -> usize {
            self.children.borrow().len()
        }
    }

    // For shared ownership with mutation, use Arc<Mutex<T>>
    pub fn concurrent_modification() {
        let data = Arc::new(Mutex::new(vec![1, 2, 3]));

        let data_clone = Arc::clone(&data);
        let handle = thread::spawn(move || {
            data_clone.lock().unwrap().push(4);
        });

        handle.join().unwrap();
        println!("Data after concurrent modification: {:?}", data.lock().unwrap());
    }

    pub fn demo() {
        println!("\n=== RefCell/Mutex Without Consideration Anti-Pattern ===");

        // Proper mutability
        let mut app = Application::new();
        app.process();
        app.process();
        println!("Config name: '{}'", app.read_config().name);

        // RefCell when justified
        let root = Node::new(1);
        root.add_child(Node::new(2));
        root.add_child(Node::new(3));
        println!("Root has {} children", root.child_count());

        // Concurrent modification
        concurrent_modification();
    }
}

// ============================================================================
// Anti-Pattern: Ignoring Send/Sync Implications
// ============================================================================

mod send_sync {
    use super::*;

    // CORRECT: Thread-safe types
    pub fn share_across_threads_safely() {
        let data = Arc::new(Mutex::new(vec![1, 2, 3]));
        let data_clone = Arc::clone(&data);

        let handle = thread::spawn(move || {
            data_clone.lock().unwrap().push(4);
        });

        handle.join().unwrap();

        let final_data = data.lock().unwrap();
        println!("Shared data: {:?}", *final_data);
    }

    // Or use message passing (often better)
    pub fn message_passing() {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(vec![1, 2, 3, 4]).unwrap();
        });

        let data = rx.recv().unwrap();
        println!("Received via channel: {:?}", data);
    }

    pub fn demo() {
        println!("\n=== Ignoring Send/Sync Implications Anti-Pattern ===");

        println!("Sharing with Arc<Mutex<T>>:");
        share_across_threads_safely();

        println!("\nUsing message passing:");
        message_passing();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_get_mut_two() {
        let mut cache = unsafe_convenience::Cache::new(vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);

        let (a, c) = cache.get_mut_two(0, 2).unwrap();
        *a = "A".to_string();
        *c = "C".to_string();

        assert_eq!(cache.data, vec!["A", "b", "C"]);
    }

    #[test]
    fn test_cache_same_index() {
        let mut cache = unsafe_convenience::Cache::new(vec!["a".to_string()]);
        assert!(cache.get_mut_two(0, 0).is_none());
    }

    #[test]
    fn test_cache_out_of_bounds() {
        let mut cache = unsafe_convenience::Cache::new(vec!["a".to_string()]);
        assert!(cache.get_mut_two(0, 10).is_none());
    }

    #[test]
    fn test_load_config_valid() {
        let json = r#"{"name": "test", "value": 42}"#;
        let config = unwrap_production::load_config(json).unwrap();
        assert_eq!(config.name, "test");
        assert_eq!(config.value, 42);
    }

    #[test]
    fn test_load_config_invalid() {
        let json = r#"{"name": "test"}"#; // missing value
        assert!(unwrap_production::load_config(json).is_err());
    }

    #[test]
    fn test_get_user_age() {
        let mut users = HashMap::new();
        users.insert("alice".to_string(), unwrap_production::User { age: 30 });

        assert_eq!(unwrap_production::get_user_age(&users, "alice"), Some(30));
        assert_eq!(unwrap_production::get_user_age(&users, "bob"), None);
    }

    #[test]
    fn test_divide() {
        assert_eq!(unwrap_production::divide(10, 2), Some(5));
        assert_eq!(unwrap_production::divide(10, 0), None);
    }

    #[test]
    fn test_get_first_line() {
        assert_eq!(unwrap_production::get_first_line("hello\nworld"), "hello");
        assert_eq!(unwrap_production::get_first_line("single"), "single");
        assert_eq!(unwrap_production::get_first_line(""), "");
    }

    #[test]
    fn test_application() {
        let mut app = refcell_mutex::Application::new();
        app.process();
        app.process();
    }

    #[test]
    fn test_node_refcell() {
        let root = refcell_mutex::Node::new(1);
        root.add_child(refcell_mutex::Node::new(2));
        root.add_child(refcell_mutex::Node::new(3));
        assert_eq!(root.child_count(), 2);
    }

    #[test]
    fn test_arc_mutex() {
        let data = Arc::new(Mutex::new(0));
        let data_clone = Arc::clone(&data);

        let handle = thread::spawn(move || {
            *data_clone.lock().unwrap() = 42;
        });

        handle.join().unwrap();
        assert_eq!(*data.lock().unwrap(), 42);
    }

    #[test]
    fn test_message_passing() {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            tx.send(42).unwrap();
        });

        assert_eq!(rx.recv().unwrap(), 42);
    }
}

fn main() {
    println!("Anti-Patterns Part 3: Safety Anti-Patterns");
    println!("===========================================\n");

    unsafe_convenience::demo();
    unwrap_production::demo();
    refcell_mutex::demo();
    send_sync::demo();
}
