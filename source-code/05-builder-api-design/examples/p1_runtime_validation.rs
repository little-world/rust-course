//! Pattern 1: Builder Pattern Variations
//! Example: Builder with Runtime Validation
//!
//! Run with: cargo run --example p1_runtime_validation

#[derive(Debug)]
pub struct Database {
    host: String,
    port: u16,
    username: String,
    password: Option<String>,
    database: Option<String>,
}

// The builder stores required fields as `Option`.
#[derive(Default)]
pub struct DatabaseBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    database: Option<String>,
}

impl DatabaseBuilder {
    pub fn new() -> Self {
        DatabaseBuilder::default()
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    // `build` returns `Result` to enforce required fields.
    pub fn build(self) -> Result<Database, String> {
        let host = self.host.ok_or("host is required")?;
        let port = self.port.ok_or("port is required")?;
        let username = self.username.ok_or("username is required")?;

        Ok(Database {
            host,
            port,
            username,
            password: self.password,
            database: self.database,
        })
    }
}

fn main() {
    println!("=== Builder with Runtime Validation ===");
    // Usage: build() returns Result to catch missing required fields at runtime.
    let db_result = DatabaseBuilder::new()
        .host("localhost")
        .port(5432)
        .username("admin")
        .password("secret")
        .database("myapp")
        .build();

    match db_result {
        Ok(db) => println!("Successfully built database config: {:#?}", db),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Missing Required Fields ===");
    let db_fail = DatabaseBuilder::new().host("localhost").build();

    match db_fail {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("\n=== Missing Only Port ===");
    let db_fail2 = DatabaseBuilder::new()
        .host("localhost")
        .username("admin")
        .build();

    match db_fail2 {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("\n=== Optional Fields Can Be Omitted ===");
    let db_minimal = DatabaseBuilder::new()
        .host("localhost")
        .port(5432)
        .username("admin")
        .build();

    match db_minimal {
        Ok(db) => {
            println!("Built with required fields only: {:#?}", db);
            println!("password is None: {}", db.password.is_none());
            println!("database is None: {}", db.database.is_none());
        }
        Err(e) => println!("Error: {}", e),
    }
}
