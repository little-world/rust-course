//! Pattern 4: SQLx Migrations
//!
//! Demonstrates SQLx's built-in migration system.
//! Migrations are SQL files tracked by timestamp.

use sqlx::postgres::PgPoolOptions;

/// Run migrations programmatically
async fn run_migrations(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run all pending migrations
    // The migrate! macro embeds SQL at compile time
    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Migrations completed successfully");

    Ok(())
}

fn explain_sqlx_migrations() {
    println!("=== SQLx Migrations ===\n");

    println!("--- Setup ---\n");
    println!("mkdir -p migrations");
    println!("sqlx migrate add create_users_table\n");

    println!("This creates: migrations/20240101000000_create_users_table.sql\n");

    println!("--- Example Migration ---\n");
    println!("-- migrations/20240101000000_create_users_table.sql");
    println!("CREATE TABLE users (");
    println!("    id SERIAL PRIMARY KEY,");
    println!("    username VARCHAR(255) NOT NULL UNIQUE,");
    println!("    email VARCHAR(255) NOT NULL UNIQUE,");
    println!("    password_hash VARCHAR(255) NOT NULL,");
    println!("    created_at TIMESTAMP NOT NULL DEFAULT NOW(),");
    println!("    updated_at TIMESTAMP NOT NULL DEFAULT NOW()");
    println!(");");
    println!("");
    println!("CREATE INDEX idx_users_email ON users(email);");
    println!("CREATE INDEX idx_users_username ON users(username);\n");

    println!("--- Run Migrations ---\n");

    println!("Programmatically:");
    println!("  sqlx::migrate!(\"./migrations\")");
    println!("      .run(&pool)");
    println!("      .await?;\n");

    println!("Via CLI:");
    println!("  sqlx migrate run          # Apply pending");
    println!("  sqlx migrate revert       # Revert last");
    println!("  sqlx migrate info         # Show status\n");

    println!("--- How It Works ---\n");
    println!("1. SQLx tracks applied migrations in _sqlx_migrations table");
    println!("2. Timestamp ordering handles concurrent development");
    println!("3. migrate!() embeds SQL at compile time (self-contained binary)");
    println!("4. Each migration runs in a transaction\n");

    println!("--- Best Practices ---\n");
    println!("1. Commit migrations to version control");
    println!("2. Never modify applied migrations");
    println!("3. Use descriptive names (create_users, add_email_index)");
    println!("4. Keep migrations small and focused\n");
}

fn explain_reversible_migrations() {
    println!("=== Reversible Migrations ===\n");

    println!("SQLx migrations are up-only by default.");
    println!("For reversibility, create .up.sql and .down.sql:\n");

    println!("migrations/");
    println!("  20240101000000_create_users.up.sql");
    println!("  20240101000000_create_users.down.sql\n");

    println!("-- up.sql");
    println!("CREATE TABLE users (...);");
    println!("");
    println!("-- down.sql");
    println!("DROP TABLE users;\n");

    println!("Or use Diesel for full up/down support.\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 4: SQLx Migrations ===\n");

    explain_sqlx_migrations();
    explain_reversible_migrations();

    // Note: We can't actually run migrations without a database
    // and the migrations directory doesn't exist yet
    println!("--- Live Migration Test ---\n");

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("To run migrations:");
    println!("1. Create migrations/ directory");
    println!("2. Add migration files");
    println!("3. Run: sqlx migrate run");
    println!("   Or: sqlx::migrate!(\"./migrations\").run(&pool).await?\n");

    // Try to connect
    match PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
    {
        Ok(_pool) => {
            println!("Connected to database!");
            // Would run migrations here if they existed:
            // sqlx::migrate!("./migrations").run(&pool).await?;
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nMigrations example completed!");
    Ok(())
}
