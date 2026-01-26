//! Pattern 5: Opaque Types and Safe Wrappers
//!
//! Demonstrates wrapping opaque C types (forward-declared structs) in safe Rust APIs.
//! Shows the pattern used by real-world FFI wrappers like rusqlite, openssl-rs, etc.

use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};

// Error codes (matching C header)
const SUCCESS: c_int = 0;

// Opaque type declaration - we don't know its internal structure
#[repr(C)]
pub struct db_connection {
    _private: [u8; 0], // Zero-sized, can't be constructed in Rust
}

extern "C" {
    fn db_open(path: *const c_char) -> *mut db_connection;
    fn db_execute(conn: *mut db_connection, sql: *const c_char) -> c_int;
    fn db_get_last_error(conn: *mut db_connection) -> *const c_char;
    fn db_close(conn: *mut db_connection);
}

// ===========================================
// Safe Rust wrapper around opaque C type
// ===========================================

#[derive(Debug)]
pub enum DatabaseError {
    OpenFailed(String),
    QueryFailed(String),
    NullPointer,
    InvalidUtf8,
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::OpenFailed(path) => write!(f, "Failed to open database: {}", path),
            DatabaseError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            DatabaseError::NullPointer => write!(f, "Null pointer error"),
            DatabaseError::InvalidUtf8 => write!(f, "Invalid UTF-8 in string"),
        }
    }
}

impl std::error::Error for DatabaseError {}

/// Safe wrapper around the opaque db_connection type
pub struct Database {
    handle: *mut db_connection,
    path: String,
}

impl Database {
    /// Opens a database connection
    pub fn open(path: &str) -> Result<Self, DatabaseError> {
        let c_path = CString::new(path)
            .map_err(|_| DatabaseError::InvalidUtf8)?;

        let handle = unsafe { db_open(c_path.as_ptr()) };

        if handle.is_null() {
            return Err(DatabaseError::OpenFailed(path.to_string()));
        }

        Ok(Database {
            handle,
            path: path.to_string(),
        })
    }

    /// Executes a SQL query
    pub fn execute(&self, sql: &str) -> Result<(), DatabaseError> {
        let c_sql = CString::new(sql)
            .map_err(|_| DatabaseError::InvalidUtf8)?;

        let result = unsafe { db_execute(self.handle, c_sql.as_ptr()) };

        if result != SUCCESS {
            let error_msg = self.get_last_error();
            return Err(DatabaseError::QueryFailed(error_msg));
        }

        Ok(())
    }

    /// Gets the last error message from the database
    fn get_last_error(&self) -> String {
        unsafe {
            let ptr = db_get_last_error(self.handle);
            if ptr.is_null() {
                return "Unknown error".to_string();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }

    /// Returns the database path
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        unsafe {
            db_close(self.handle);
        }
        println!("  [Rust] Database handle dropped for: {}", self.path);
    }
}

// Mark as Send if the C library is thread-safe
// unsafe impl Send for Database {}

// ===========================================
// Builder pattern for complex initialization
// ===========================================

pub struct DatabaseBuilder {
    path: Option<String>,
    readonly: bool,
    create_if_missing: bool,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        DatabaseBuilder {
            path: None,
            readonly: false,
            create_if_missing: true,
        }
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    pub fn create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
        self
    }

    pub fn open(self) -> Result<Database, DatabaseError> {
        let path = self.path.ok_or(DatabaseError::OpenFailed("No path specified".to_string()))?;

        // In a real implementation, we'd pass these flags to the C library
        println!("  [Rust] Opening with options: readonly={}, create_if_missing={}",
            self.readonly, self.create_if_missing);

        Database::open(&path)
    }
}

fn main() {
    println!("=== Pattern 5: Opaque Types and Safe Wrappers ===\n");

    // --- Basic Database Operations ---
    println!("--- Basic Database Operations ---");

    {
        println!("Opening database in inner scope:");

        match Database::open("test.db") {
            Ok(db) => {
                println!("  Database opened: {}", db.path());

                // Execute some queries
                let queries = [
                    "CREATE TABLE users (id INT, name TEXT)",
                    "INSERT INTO users VALUES (1, 'Alice')",
                    "SELECT * FROM users",
                    "TRIGGER ERROR SIMULATION", // This will fail
                ];

                for sql in queries {
                    match db.execute(sql) {
                        Ok(()) => println!("  Query OK: {}", sql),
                        Err(e) => println!("  Query failed: {} ({})", sql, e),
                    }
                }
            }
            Err(e) => println!("  Failed to open database: {}", e),
        }

        println!("Leaving inner scope...");
    } // Database is automatically closed here

    // --- Multiple Database Handles ---
    println!("\n--- Multiple Database Handles ---");

    let db1 = Database::open("database1.db");
    let db2 = Database::open("database2.db");

    if let (Ok(ref d1), Ok(ref d2)) = (&db1, &db2) {
        println!("Opened two databases: {}, {}", d1.path(), d2.path());

        d1.execute("SELECT 1").ok();
        d2.execute("SELECT 2").ok();
    }

    // Explicit drop to show cleanup order
    drop(db1);
    println!("  First database explicitly dropped");
    drop(db2);
    println!("  Second database explicitly dropped");

    // --- Builder Pattern ---
    println!("\n--- Builder Pattern ---");

    let db = DatabaseBuilder::new()
        .path("configured.db")
        .readonly(false)
        .create_if_missing(true)
        .open();

    match db {
        Ok(database) => {
            println!("  Builder-created database: {}", database.path());
            database.execute("SELECT 'builder test'").ok();
        }
        Err(e) => println!("  Builder failed: {}", e),
    }

    // --- Error Handling Showcase ---
    println!("\n--- Error Handling Showcase ---");

    // Invalid path (contains null)
    let result = Database::open("invalid\0path");
    println!("  Opening 'invalid\\0path': {:?}", result.err());

    // Normal operation with error
    if let Ok(db) = Database::open("error_test.db") {
        match db.execute("CAUSE ERROR") {
            Ok(()) => println!("  Unexpected success"),
            Err(e) => println!("  Expected error: {}", e),
        }
    }

    // --- Demonstrating Opaque Type Safety ---
    println!("\n--- Opaque Type Safety ---");

    println!("  db_connection is zero-sized: {} bytes",
        std::mem::size_of::<db_connection>());
    println!("  Database wrapper size: {} bytes",
        std::mem::size_of::<Database>());
    println!("  (Contains pointer + String for path)");

    // You CANNOT do this (would be compile error):
    // let fake = db_connection { _private: [] }; // Error: private field
    // This is exactly what we want - only C can create db_connection instances

    println!("\nAll opaque type examples completed successfully!");
}
