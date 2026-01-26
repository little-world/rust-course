//! Pattern 4: Custom Error Types with Context
//! Example: Database Query Error with Structured Metadata
//!
//! Run with: cargo run --example p4_query_error

use thiserror::Error;
use std::time::{Duration, Instant};

/// Database query error with rich debugging information.
#[derive(Error, Debug)]
#[error("Database query failed")]
pub struct QueryError {
    pub query: String,
    pub duration_ms: u64,
    pub row_count: Option<usize>,
    pub parameters: Vec<String>,
    #[source]
    pub source: Box<dyn std::error::Error + Send + Sync>,
}

impl QueryError {
    /// Format detailed error for logging.
    pub fn display_detailed(&self) -> String {
        let mut lines = vec!["Query Error Report:".to_string()];
        lines.push(format!("  Query: {}", self.query));
        lines.push(format!("  Duration: {}ms", self.duration_ms));

        if !self.parameters.is_empty() {
            lines.push(format!("  Parameters: {:?}", self.parameters));
        }

        if let Some(count) = self.row_count {
            lines.push(format!("  Rows processed: {}", count));
        }

        lines.push(format!("  Error: {}", self.source));

        lines.join("\n")
    }
}

/// Builder for QueryError.
pub struct QueryErrorBuilder {
    query: String,
    start_time: Instant,
    parameters: Vec<String>,
    row_count: Option<usize>,
}

impl QueryErrorBuilder {
    pub fn new(query: impl Into<String>) -> Self {
        QueryErrorBuilder {
            query: query.into(),
            start_time: Instant::now(),
            parameters: Vec::new(),
            row_count: None,
        }
    }

    pub fn with_params(mut self, params: Vec<String>) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_row_count(mut self, count: usize) -> Self {
        self.row_count = Some(count);
        self
    }

    pub fn build(self, source: impl std::error::Error + Send + Sync + 'static) -> QueryError {
        QueryError {
            query: self.query,
            duration_ms: self.start_time.elapsed().as_millis() as u64,
            row_count: self.row_count,
            parameters: self.parameters,
            source: Box::new(source),
        }
    }
}

/// Simulate a database query.
fn execute_query(query: &str, should_fail: bool) -> Result<Vec<String>, QueryError> {
    let builder = QueryErrorBuilder::new(query)
        .with_params(vec!["user_id=42".into()]);

    // Simulate some work
    std::thread::sleep(Duration::from_millis(50));

    if should_fail {
        let io_err = std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "database connection lost"
        );
        Err(builder.build(io_err))
    } else {
        Ok(vec!["row1".into(), "row2".into()])
    }
}

fn main() {
    println!("=== Database Query Error with Metadata ===\n");

    // Successful query
    println!("=== Successful Query ===");
    match execute_query("SELECT * FROM users WHERE id = ?", false) {
        Ok(rows) => println!("  Result: {:?}\n", rows),
        Err(e) => println!("{}\n", e.display_detailed()),
    }

    // Failed query
    println!("=== Failed Query ===");
    match execute_query("SELECT * FROM users WHERE id = ?", true) {
        Ok(rows) => println!("  Result: {:?}", rows),
        Err(e) => {
            println!("Standard Display:");
            println!("  {}\n", e);

            println!("Detailed Report:");
            println!("{}", e.display_detailed());
        }
    }

    println!("\n=== Why Structured Metadata? ===");
    println!("1. Query text helps reproduce the issue");
    println!("2. Duration identifies slow queries");
    println!("3. Parameters show exact input values");
    println!("4. Row count shows progress before failure");
    println!("5. Source error preserved for debugging");

    println!("\n=== Key Points ===");
    println!("1. Capture timing at start, compute duration at build");
    println!("2. Builder pattern collects context incrementally");
    println!("3. Display for users, display_detailed for logs");
    println!("4. #[source] preserves error chain");
}
