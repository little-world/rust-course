//! Pattern 1: Struct Design Patterns
//! Example: Unit Structs (Zero-sized types for type-level programming)
//!
//! Run with: cargo run --example p1_unit_struct

// Marker types for type-level programming
struct Authenticated;
struct Unauthenticated;

// Zero-sized types for phantom data
struct Database<State> {
    connection_string: String,
    _state: std::marker::PhantomData<State>,
}

impl Database<Unauthenticated> {
    fn new(connection_string: String) -> Self {
        Database {
            connection_string,
            _state: std::marker::PhantomData,
        }
    }

    fn authenticate(self, password: &str) -> Result<Database<Authenticated>, String> {
        if password == "secret" {
            Ok(Database {
                connection_string: self.connection_string,
                _state: std::marker::PhantomData,
            })
        } else {
            Err("Invalid password".to_string())
        }
    }
}

impl Database<Authenticated> {
    fn query(&self, sql: &str) -> Vec<String> {
        println!("Executing query: {}", sql);
        vec!["result1".to_string(), "result2".to_string()]
    }
}

fn main() {
    // Usage: Type state ensures query() is only callable after authentication.
    let db = Database::<Unauthenticated>::new("postgres://localhost".to_string());

    // This won't compile - query() not available on Unauthenticated:
    // db.query("SELECT * FROM users");

    println!("Attempting authentication...");
    match db.authenticate("secret") {
        Ok(auth_db) => {
            println!("Authentication successful!");
            let results = auth_db.query("SELECT * FROM users");
            println!("Query results: {:?}", results);
        }
        Err(e) => {
            println!("Authentication failed: {}", e);
        }
    }

    // Demonstrate failed authentication
    let db2 = Database::<Unauthenticated>::new("postgres://localhost".to_string());
    match db2.authenticate("wrong_password") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("\nFailed login attempt: {}", e),
    }

    // Show that marker types are zero-sized
    println!("\nSize of Authenticated: {} bytes", std::mem::size_of::<Authenticated>());
    println!("Size of Unauthenticated: {} bytes", std::mem::size_of::<Unauthenticated>());
}
