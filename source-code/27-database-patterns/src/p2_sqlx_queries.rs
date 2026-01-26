//! Pattern 2: SQLx Compile-Time Checked Queries
//!
//! Demonstrates SQLx's query! and query_as! macros that verify SQL at compile time.
//! Note: Compile-time checking requires DATABASE_URL or offline mode.

use sqlx::{FromRow, PgPool};

#[derive(FromRow, Debug)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    // SQLx has built-in connection pooling
    PgPool::connect(database_url).await
}

/// Compile-time verified query using query_as!
/// Note: This requires DATABASE_URL at compile time for verification
async fn fetch_user(pool: &PgPool, user_id: i32) -> Result<User, sqlx::Error> {
    // In real code with DATABASE_URL set:
    // sqlx::query_as!(
    //     User,
    //     "SELECT id, username, email, created_at FROM users WHERE id = $1",
    //     user_id
    // )
    // .fetch_one(pool)
    // .await

    // For demonstration without compile-time checking:
    sqlx::query_as::<_, User>("SELECT id, username, email, created_at FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
}

/// Insert with returning
async fn create_user(pool: &PgPool, username: &str, email: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, created_at)
        VALUES ($1, $2, NOW())
        RETURNING id, username, email, created_at
        "#,
    )
    .bind(username)
    .bind(email)
    .fetch_one(pool)
    .await
}

/// Demonstrate different fetch methods
async fn demonstrate_fetch_methods(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("--- Fetch Methods ---\n");

    // fetch_one: Expects exactly one row, errors otherwise
    println!("fetch_one: Expects exactly one row");
    let _user: User =
        sqlx::query_as("SELECT id, username, email, created_at FROM users WHERE id = $1")
            .bind(1)
            .fetch_one(pool)
            .await?;

    // fetch_optional: Returns Option<T>
    println!("fetch_optional: Returns Option<T>");
    let _maybe_user: Option<User> =
        sqlx::query_as("SELECT id, username, email, created_at FROM users WHERE id = $1")
            .bind(999)
            .fetch_optional(pool)
            .await?;

    // fetch_all: Returns Vec<T>
    println!("fetch_all: Returns Vec<T>");
    let _users: Vec<User> = sqlx::query_as("SELECT id, username, email, created_at FROM users")
        .fetch_all(pool)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 2: SQLx Compile-Time Checked Queries ===\n");

    println!("--- How It Works ---\n");
    println!("SQLx's query! macro connects to database during compilation:");
    println!("  - Verifies SQL syntax");
    println!("  - Checks table/column existence");
    println!("  - Validates parameter types");
    println!("  - Ensures return types match struct fields\n");

    println!("Benefits:");
    println!("  - Misspell a column? Won't compile.");
    println!("  - Wrong type? Won't compile.");
    println!("  - Table doesn't exist? Won't compile.\n");

    println!("--- Fetch Methods ---\n");
    println!("fetch_one:      Expects exactly 1 row (error otherwise)");
    println!("fetch_optional: Returns Option<T>");
    println!("fetch_all:      Returns Vec<T>\n");

    println!("--- Offline Mode ---\n");
    println!("For CI/CD without database access:");
    println!("  1. cargo sqlx prepare  (saves query metadata to .sqlx/)");
    println!("  2. Commit .sqlx/ to version control");
    println!("  3. Set SQLX_OFFLINE=true for builds\n");

    // Try to connect
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Connecting to Database ---\n");

    match create_pool(&database_url).await {
        Ok(pool) => {
            println!("Connected successfully!\n");

            match fetch_user(&pool, 1).await {
                Ok(user) => println!("Fetched user: {:?}", user),
                Err(e) => println!("Error fetching user: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("\nThis is expected without a running database.");
            println!("The pattern demonstrates compile-time SQL verification.");
        }
    }

    println!("\nSQLx query example completed!");
    Ok(())
}
