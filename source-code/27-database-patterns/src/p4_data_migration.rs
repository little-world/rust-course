//! Pattern 4: Data Migrations
//!
//! Demonstrates migrating data (not just schema) in batches.
//! Essential for large datasets to prevent memory exhaustion.

use sqlx::PgPool;

/// Run a data migration in batches
async fn run_data_migration(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("Starting data migration...");

    // Start transaction for atomicity
    let mut tx = pool.begin().await?;

    let batch_size: i64 = 1000;
    let mut offset: i64 = 0;
    let mut total_processed = 0;

    loop {
        // Fetch users in batches
        let users: Vec<(i32, String)> =
            sqlx::query_as("SELECT id, email FROM users LIMIT $1 OFFSET $2")
                .bind(batch_size)
                .bind(offset)
                .fetch_all(&mut *tx)
                .await?;

        if users.is_empty() {
            break;
        }

        let batch_count = users.len();

        // Process each user
        for (id, email) in users {
            let normalized = email.to_lowercase();

            sqlx::query("UPDATE users SET email_normalized = $1 WHERE id = $2")
                .bind(normalized)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        total_processed += batch_count;
        offset += batch_size;

        println!("Processed {} rows (batch of {})", total_processed, batch_count);
    }

    tx.commit().await?;

    println!("Migration complete! Total rows: {}", total_processed);

    Ok(())
}

/// Data migration with per-batch commits (for very large datasets)
async fn run_large_data_migration(pool: &PgPool) -> Result<(), sqlx::Error> {
    println!("Starting large data migration with per-batch commits...");

    let batch_size: i64 = 1000;
    let mut last_id: i32 = 0;
    let mut total_processed = 0;

    loop {
        // Use keyset pagination (more efficient than OFFSET)
        let users: Vec<(i32, String)> = sqlx::query_as(
            "SELECT id, email FROM users WHERE id > $1 ORDER BY id LIMIT $2",
        )
        .bind(last_id)
        .bind(batch_size)
        .fetch_all(pool)
        .await?;

        if users.is_empty() {
            break;
        }

        // Update last_id for next batch
        last_id = users.last().map(|(id, _)| *id).unwrap_or(last_id);

        // Process this batch in its own transaction
        let mut tx = pool.begin().await?;

        for (id, email) in &users {
            let normalized = email.to_lowercase();

            sqlx::query("UPDATE users SET email_normalized = $1 WHERE id = $2")
                .bind(normalized)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        total_processed += users.len();
        println!("Committed batch: {} total rows processed", total_processed);
    }

    println!("Migration complete! Total rows: {}", total_processed);

    Ok(())
}

fn explain_data_migrations() {
    println!("=== Data Migrations ===\n");

    println!("--- Schema vs Data Migration ---\n");
    println!("Schema migration: CREATE TABLE, ALTER TABLE");
    println!("Data migration:   UPDATE existing rows, transform data\n");

    println!("--- The Problem ---\n");
    println!("Large tables can't be processed in one query:");
    println!("  - Memory exhaustion (loading millions of rows)");
    println!("  - Transaction log bloat");
    println!("  - Lock contention");
    println!("  - Timeout issues\n");

    println!("--- Solution: Batch Processing ---\n");
    println!("Process 1000-10000 rows at a time:\n");

    println!("loop {{");
    println!("    let users = query_batch(LIMIT $1 OFFSET $2);");
    println!("    if users.is_empty() {{ break; }}");
    println!("    ");
    println!("    for user in users {{");
    println!("        update_user(user);");
    println!("    }}");
    println!("    ");
    println!("    offset += batch_size;");
    println!("}}\n");

    println!("--- Keyset Pagination ---\n");
    println!("Better than OFFSET for large tables:\n");

    println!("// Instead of:");
    println!("SELECT * FROM users LIMIT 1000 OFFSET 100000  // Slow!\n");

    println!("// Use:");
    println!("SELECT * FROM users WHERE id > $last_id ORDER BY id LIMIT 1000\n");

    println!("--- Transaction Strategy ---\n");
    println!("Option 1: Single transaction (all-or-nothing)");
    println!("  + Atomic: all rows or none");
    println!("  - Can't resume after failure");
    println!("  - Long lock held\n");

    println!("Option 2: Per-batch commits");
    println!("  + Resumable after failure (track last_id)");
    println!("  + Shorter locks");
    println!("  - Not atomic (partial state possible)");
    println!("  - Need idempotent logic\n");

    println!("--- Idempotent Migration ---\n");
    println!("Make migrations safe to re-run:\n");

    println!("// Bad: Will fail on re-run");
    println!("INSERT INTO table VALUES (1, 'data');\n");

    println!("// Good: Safe to re-run");
    println!("INSERT INTO table VALUES (1, 'data') ON CONFLICT DO NOTHING;\n");

    println!("// Or check before acting:");
    println!("UPDATE users SET normalized = lower(email)");
    println!("WHERE normalized IS NULL;\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 4: Data Migrations ===\n");

    explain_data_migrations();

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Live Test ---\n");

    match PgPool::connect(&database_url).await {
        Ok(pool) => {
            match run_data_migration(&pool).await {
                Ok(()) => println!("Data migration completed!"),
                Err(e) => println!("Migration failed: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nData migration example completed!");
    Ok(())
}
