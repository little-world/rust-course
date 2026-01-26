// Anti-Patterns Part 4: API Design Mistakes
// Demonstrates API design mistakes and their correct solutions.

use std::collections::HashMap;
use std::hash::Hash;

// ============================================================================
// Anti-Pattern: Stringly-Typed APIs
// ============================================================================

mod stringly_typed {
    // ANTI-PATTERN: String-based API
    #[allow(dead_code)]
    fn set_log_level_bad(level: &str) {
        match level {
            "debug" | "info" | "warn" | "error" => { /* ... */ }
            _ => panic!("Invalid log level"),
        }
    }

    // CORRECT: Type-safe enums
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum LogLevel {
        Debug,
        Info,
        Warn,
        Error,
    }

    pub fn set_log_level(level: LogLevel) {
        match level {
            LogLevel::Debug => println!("Log level set to DEBUG"),
            LogLevel::Info => println!("Log level set to INFO"),
            LogLevel::Warn => println!("Log level set to WARN"),
            LogLevel::Error => println!("Log level set to ERROR"),
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Color {
        Red,
        Green,
        Blue,
        Rgb(u8, u8, u8),
    }

    impl Color {
        pub fn to_rgb(self) -> (u8, u8, u8) {
            match self {
                Color::Red => (255, 0, 0),
                Color::Green => (0, 255, 0),
                Color::Blue => (0, 0, 255),
                Color::Rgb(r, g, b) => (r, g, b),
            }
        }
    }

    pub fn demo() {
        println!("=== Stringly-Typed APIs Anti-Pattern ===");

        // Compile-time checked, IDE autocomplete
        set_log_level(LogLevel::Debug);
        // set_log_level(LogLevel::Degub);  // Compile error!

        let color = Color::Red;
        let custom = Color::Rgb(128, 128, 128);
        println!("Red RGB: {:?}", color.to_rgb());
        println!("Custom RGB: {:?}", custom.to_rgb());
    }
}

// ============================================================================
// Anti-Pattern: Boolean Parameter Trap
// ============================================================================

mod boolean_trap {
    // ANTI-PATTERN: Unclear boolean parameters
    #[allow(dead_code)]
    fn connect_bad(
        _host: &str,
        _encrypted: bool,
        _persistent: bool,
        _verbose: bool,
    ) {
        // ...
    }

    // CORRECT: Explicit enum parameters
    #[derive(Debug, Clone, Copy)]
    pub enum Encryption {
        Encrypted,
        Plaintext,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ConnectionType {
        Persistent,
        Transient,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Verbosity {
        Verbose,
        Quiet,
    }

    pub fn connect(
        host: &str,
        encryption: Encryption,
        connection: ConnectionType,
        verbosity: Verbosity,
    ) {
        println!(
            "Connecting to {} with {:?}, {:?}, {:?}",
            host, encryption, connection, verbosity
        );
    }

    // Or use builder pattern
    pub struct ConnectionBuilder {
        host: String,
        encrypted: bool,
        persistent: bool,
        verbose: bool,
    }

    pub struct Connection {
        pub host: String,
        pub encrypted: bool,
        pub persistent: bool,
        pub verbose: bool,
    }

    impl ConnectionBuilder {
        pub fn new(host: impl Into<String>) -> Self {
            Self {
                host: host.into(),
                encrypted: false,
                persistent: false,
                verbose: false,
            }
        }

        pub fn encrypted(mut self) -> Self {
            self.encrypted = true;
            self
        }

        pub fn persistent(mut self) -> Self {
            self.persistent = true;
            self
        }

        pub fn verbose(mut self) -> Self {
            self.verbose = true;
            self
        }

        pub fn connect(self) -> Connection {
            println!(
                "Builder: Connecting to {} (encrypted: {}, persistent: {}, verbose: {})",
                self.host, self.encrypted, self.persistent, self.verbose
            );
            Connection {
                host: self.host,
                encrypted: self.encrypted,
                persistent: self.persistent,
                verbose: self.verbose,
            }
        }
    }

    pub fn demo() {
        println!("\n=== Boolean Parameter Trap Anti-Pattern ===");

        // Clear intent at call site with enums
        connect(
            "localhost",
            Encryption::Encrypted,
            ConnectionType::Transient,
            Verbosity::Verbose,
        );

        // Clear and fluent with builder
        let _conn = ConnectionBuilder::new("localhost")
            .encrypted()
            .verbose()
            .connect();
    }
}

// ============================================================================
// Anti-Pattern: Leaky Abstractions
// ============================================================================

mod leaky_abstractions {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct Row {
        pub columns: Vec<String>,
    }

    #[derive(Debug)]
    pub struct Connection {
        #[allow(dead_code)]
        connection_string: String,
    }

    impl Connection {
        pub fn new(connection_string: &str) -> Result<Self, Error> {
            Ok(Connection {
                connection_string: connection_string.to_string(),
            })
        }

        pub fn execute(&mut self, sql: &str) -> Result<Vec<Row>, Error> {
            // Simulated query execution
            Ok(vec![Row {
                columns: vec![sql.to_string()],
            }])
        }
    }

    #[derive(Debug)]
    pub enum Error {
        NoConnections,
        #[allow(dead_code)]
        QueryFailed,
    }

    // ANTI-PATTERN: Exposing internal details
    #[allow(dead_code)]
    pub struct DatabaseBad {
        pub connection_pool: Vec<Connection>, // Internal detail exposed
        pub cache: HashMap<String, Vec<u8>>,  // Implementation leaked
    }

    // CORRECT: Proper encapsulation
    pub struct Database {
        connection_pool: Vec<Connection>, // Private
        cache: HashMap<String, Vec<Row>>, // Private
    }

    impl Database {
        pub fn new(connection_string: &str) -> Result<Self, Error> {
            Ok(Self {
                connection_pool: vec![Connection::new(connection_string)?],
                cache: HashMap::new(),
            })
        }

        pub fn query(&mut self, sql: &str) -> Result<Vec<Row>, Error> {
            // Returns proper type, not implementation detail
            if let Some(cached) = self.cache.get(sql) {
                return Ok(cached.clone());
            }

            let conn = self.get_connection_internal()?;
            let result = conn.execute(sql)?;
            self.cache.insert(sql.to_string(), result.clone());
            Ok(result)
        }

        // Private helper
        fn get_connection_internal(&mut self) -> Result<&mut Connection, Error> {
            self.connection_pool.first_mut().ok_or(Error::NoConnections)
        }
    }

    pub fn demo() {
        println!("\n=== Leaky Abstractions Anti-Pattern ===");

        let mut db = Database::new("postgres://localhost").unwrap();
        let result = db.query("SELECT * FROM users").unwrap();
        println!("Query result: {:?}", result);

        // Second query uses cache
        let result2 = db.query("SELECT * FROM users").unwrap();
        println!("Cached result: {:?}", result2);
    }
}

// ============================================================================
// Anti-Pattern: Returning Owned When Borrowed Suffices
// ============================================================================

mod returning_owned {
    // ANTI-PATTERN: Unnecessary ownership transfer
    #[allow(dead_code)]
    struct UserBad {
        name: String,
        email: String,
    }

    #[allow(dead_code)]
    impl UserBad {
        fn get_name(&self) -> String {
            self.name.clone() // Allocates on every call
        }

        fn get_email(&self) -> String {
            self.email.clone() // Unnecessary clone
        }
    }

    // CORRECT: Return references
    pub struct User {
        name: String,
        email: String,
    }

    impl User {
        pub fn new(name: &str, email: &str) -> Self {
            User {
                name: name.to_string(),
                email: email.to_string(),
            }
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn email(&self) -> &str {
            &self.email
        }

        // Only return owned when you create new data
        pub fn display_name(&self) -> String {
            format!("{} ({})", self.name, self.email)
            // Creating new data, ownership transfer makes sense
        }
    }

    pub fn format_user(user: &User) -> String {
        format!("{} <{}>", user.name(), user.email())
        // No extra allocations for reading
    }

    pub fn demo() {
        println!("\n=== Returning Owned When Borrowed Suffices Anti-Pattern ===");

        let user = User::new("Alice", "alice@example.com");

        // Zero-cost access
        println!("Name: {}", user.name());
        println!("Email: {}", user.email());

        // Owned when creating new data
        println!("Display: {}", user.display_name());

        // Formatted output
        println!("Formatted: {}", format_user(&user));
    }
}

// ============================================================================
// Anti-Pattern: Overengineered Generic APIs
// ============================================================================

mod overengineered_generics {
    use super::*;

    // ANTI-PATTERN: Overly generic for no benefit
    #[allow(dead_code)]
    fn print_items_bad<I, T>(items: I)
    where
        I: IntoIterator<Item = T>,
        T: std::fmt::Display + std::fmt::Debug + Clone + Send + Sync + 'static,
    {
        for item in items {
            println!("{}", item);
        }
    }

    // CORRECT: Simple and clear
    pub fn print_items(items: &[impl std::fmt::Display]) {
        for item in items {
            println!("{}", item);
        }
    }

    // Or be generic only where it adds value
    pub fn print_items_generic<T: std::fmt::Display>(items: &[T]) {
        for item in items {
            println!("{}", item);
        }
    }

    // Reserve complex bounds for when truly needed
    pub fn count_occurrences<T>(items: &[T]) -> HashMap<&T, usize>
    where
        T: Hash + Eq, // Actually needed for HashMap
    {
        let mut counts = HashMap::new();
        for item in items {
            *counts.entry(item).or_insert(0) += 1;
        }
        counts
    }

    pub fn demo() {
        println!("\n=== Overengineered Generic APIs Anti-Pattern ===");

        let numbers = vec![1, 2, 3, 4, 5];
        println!("Simple print_items:");
        print_items(&numbers);

        let strings = vec!["hello", "world"];
        println!("\nGeneric print_items:");
        print_items_generic(&strings);

        let items = vec!["a", "b", "a", "c", "a", "b"];
        println!("\nCount occurrences:");
        let counts = count_occurrences(&items);
        for (item, count) in counts {
            println!("  '{}' appears {} times", item, count);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level() {
        // Just verify it compiles and runs
        stringly_typed::set_log_level(stringly_typed::LogLevel::Debug);
        stringly_typed::set_log_level(stringly_typed::LogLevel::Error);
    }

    #[test]
    fn test_color_rgb() {
        assert_eq!(stringly_typed::Color::Red.to_rgb(), (255, 0, 0));
        assert_eq!(stringly_typed::Color::Green.to_rgb(), (0, 255, 0));
        assert_eq!(stringly_typed::Color::Blue.to_rgb(), (0, 0, 255));
        assert_eq!(stringly_typed::Color::Rgb(128, 64, 32).to_rgb(), (128, 64, 32));
    }

    #[test]
    fn test_connection_builder() {
        let conn = boolean_trap::ConnectionBuilder::new("localhost")
            .encrypted()
            .persistent()
            .connect();

        assert_eq!(conn.host, "localhost");
        assert!(conn.encrypted);
        assert!(conn.persistent);
        assert!(!conn.verbose);
    }

    #[test]
    fn test_connection_builder_defaults() {
        let conn = boolean_trap::ConnectionBuilder::new("example.com").connect();

        assert_eq!(conn.host, "example.com");
        assert!(!conn.encrypted);
        assert!(!conn.persistent);
        assert!(!conn.verbose);
    }

    #[test]
    fn test_database() {
        let mut db = leaky_abstractions::Database::new("test://localhost").unwrap();
        let result = db.query("SELECT 1").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_database_caching() {
        let mut db = leaky_abstractions::Database::new("test://localhost").unwrap();

        // First query
        let result1 = db.query("SELECT * FROM test").unwrap();

        // Second query should return cached result
        let result2 = db.query("SELECT * FROM test").unwrap();

        assert_eq!(result1.len(), result2.len());
    }

    #[test]
    fn test_user_references() {
        let user = returning_owned::User::new("Bob", "bob@example.com");

        assert_eq!(user.name(), "Bob");
        assert_eq!(user.email(), "bob@example.com");
    }

    #[test]
    fn test_user_display_name() {
        let user = returning_owned::User::new("Charlie", "charlie@example.com");
        let display = user.display_name();

        assert!(display.contains("Charlie"));
        assert!(display.contains("charlie@example.com"));
    }

    #[test]
    fn test_format_user() {
        let user = returning_owned::User::new("Diana", "diana@example.com");
        let formatted = returning_owned::format_user(&user);

        assert_eq!(formatted, "Diana <diana@example.com>");
    }

    #[test]
    fn test_count_occurrences() {
        let items = vec!["a", "b", "a", "c", "a"];
        let counts = overengineered_generics::count_occurrences(&items);

        assert_eq!(*counts.get(&"a").unwrap(), 3);
        assert_eq!(*counts.get(&"b").unwrap(), 1);
        assert_eq!(*counts.get(&"c").unwrap(), 1);
    }

    #[test]
    fn test_count_occurrences_numbers() {
        let items = vec![1, 2, 2, 3, 3, 3];
        let counts = overengineered_generics::count_occurrences(&items);

        assert_eq!(*counts.get(&1).unwrap(), 1);
        assert_eq!(*counts.get(&2).unwrap(), 2);
        assert_eq!(*counts.get(&3).unwrap(), 3);
    }
}

fn main() {
    println!("Anti-Patterns Part 4: API Design Mistakes");
    println!("==========================================\n");

    stringly_typed::demo();
    boolean_trap::demo();
    leaky_abstractions::demo();
    returning_owned::demo();
    overengineered_generics::demo();
}
