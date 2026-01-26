//! Pattern 3: #[must_use] for Critical Return Values
//! Example: Applying #[must_use] to Functions and Types
//!
//! Run with: cargo run --example p3_must_use

// Applying `#[must_use]` to a function's return value.
// A custom message explains why it's important.
#[must_use = "this Result may contain an error that should be handled"]
pub fn connect_to_db(host: &str) -> Result<DbConnection, &'static str> {
    if host.is_empty() {
        Err("Host cannot be empty")
    } else if host == "invalid" {
        Err("Failed to connect to database")
    } else {
        Ok(DbConnection {
            host: host.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct DbConnection {
    host: String,
}

// Applying `#[must_use]` to a type definition.
// Any function returning this type will implicitly be `must_use`.
#[must_use = "a Transaction does nothing unless you call `.commit()` or `.rollback()`"]
pub struct Transaction {
    id: u64,
    committed: bool,
}

impl Transaction {
    pub fn new(id: u64) -> Self {
        println!("Started transaction {}", id);
        Transaction {
            id,
            committed: false,
        }
    }

    pub fn execute(&mut self, query: &str) {
        println!("Transaction {}: executing '{}'", self.id, query);
    }

    pub fn commit(mut self) {
        self.committed = true;
        println!("Transaction {} committed", self.id);
    }

    pub fn rollback(self) {
        println!("Transaction {} rolled back", self.id);
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed {
            println!("WARNING: Transaction {} dropped without commit!", self.id);
        }
    }
}

// Builder with #[must_use]
#[must_use = "a builder does nothing unless you call `.build()`"]
pub struct ConfigBuilder {
    name: String,
    debug: bool,
}

impl ConfigBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        ConfigBuilder {
            name: name.into(),
            debug: false,
        }
    }

    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }

    pub fn build(self) -> Config {
        Config {
            name: self.name,
            debug: self.debug,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    name: String,
    debug: bool,
}

fn main() {
    println!("=== #[must_use] on Functions ===");

    // Correct usage: handle the Result
    match connect_to_db("localhost") {
        Ok(conn) => println!("Connected to: {:?}", conn),
        Err(e) => println!("Connection error: {}", e),
    }

    // Using if let
    if let Err(e) = connect_to_db("invalid") {
        println!("Expected error: {}", e);
    }

    // This would produce a warning:
    // connect_to_db("localhost"); // WARNING: unused Result

    println!("\n=== #[must_use] on Types ===");

    // Correct usage: use the transaction
    let mut tx = Transaction::new(1);
    tx.execute("INSERT INTO users VALUES (1, 'Alice')");
    tx.commit();

    // Correct: explicit rollback
    let mut tx2 = Transaction::new(2);
    tx2.execute("DELETE FROM users WHERE id = 1");
    tx2.rollback();

    // This would produce a warning:
    // Transaction::new(3); // WARNING: unused Transaction

    println!("\n=== #[must_use] on Builders ===");

    // Correct usage: call build()
    let config = ConfigBuilder::new("MyApp").debug(true).build();
    println!("Built config: {:?}", config);

    // This would produce a warning:
    // ConfigBuilder::new("Unused"); // WARNING: unused ConfigBuilder

    println!("\n=== Why #[must_use] Matters ===");
    println!("- Prevents silent error ignoring (Result, Option)");
    println!("- Ensures builders are finalized (.build())");
    println!("- Guards against resource leaks (transactions, locks)");
    println!("- Documents API intent clearly");
    println!("- Compiler catches mistakes at build time");

    println!("\n=== Common #[must_use] Applications ===");
    println!("- Result<T, E> (standard library)");
    println!("- Option<T> (standard library)");
    println!("- Future<T> (must be .await-ed)");
    println!("- MutexGuard (lock must be held)");
    println!("- Builder types");
    println!("- Transaction/Connection handles");
}
