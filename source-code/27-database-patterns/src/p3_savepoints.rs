//! Pattern 3: Nested Transactions with Savepoints
//!
//! Savepoints allow partial rollback within a transaction.
//! Useful for "best-effort" operations where some parts can fail.

use sqlx::{Acquire, PgPool, Postgres, Transaction};

/// Risky operation that might fail
async fn risky_operation(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO risky_table (value) VALUES (100)")
        .execute(&mut **tx)
        .await?;

    // Simulate failure
    Err(sqlx::Error::Protocol("Simulated risky operation failure".into()))
}

/// Complex operation with nested transaction (savepoint)
async fn complex_operation(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Operation 1 - always runs
    sqlx::query("INSERT INTO logs (message) VALUES ('Starting')")
        .execute(&mut *tx)
        .await?;

    // Create a savepoint for nested transaction
    let mut savepoint = tx.begin().await?;

    // Try a risky operation
    match risky_operation(&mut savepoint).await {
        Ok(_) => {
            // Success - commit the savepoint
            savepoint.commit().await?;
            println!("Risky operation succeeded");
        }
        Err(e) => {
            // Failure - rollback just the savepoint
            // The outer transaction continues!
            println!("Risky operation failed: {}", e);
            savepoint.rollback().await?;
        }
    }

    // Operation 2 - happens regardless of risky_operation outcome
    sqlx::query("INSERT INTO logs (message) VALUES ('Completed')")
        .execute(&mut *tx)
        .await?;

    // Commit the outer transaction
    tx.commit().await?;

    Ok(())
}

/// Multiple savepoints example
async fn multi_savepoint_operation(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    println!("Starting main transaction");

    // First savepoint
    {
        let mut sp1 = tx.begin().await?;
        println!("  Savepoint 1: Processing batch A");

        sqlx::query("INSERT INTO logs (message) VALUES ('Batch A')")
            .execute(&mut *sp1)
            .await?;

        sp1.commit().await?;
        println!("  Savepoint 1: Committed");
    }

    // Second savepoint - this one fails
    {
        let mut sp2 = tx.begin().await?;
        println!("  Savepoint 2: Processing batch B (will fail)");

        sqlx::query("INSERT INTO logs (message) VALUES ('Batch B')")
            .execute(&mut *sp2)
            .await?;

        // Simulate failure
        println!("  Savepoint 2: Rolling back");
        sp2.rollback().await?;
    }

    // Third savepoint - still runs after sp2 failure
    {
        let mut sp3 = tx.begin().await?;
        println!("  Savepoint 3: Processing batch C");

        sqlx::query("INSERT INTO logs (message) VALUES ('Batch C')")
            .execute(&mut *sp3)
            .await?;

        sp3.commit().await?;
        println!("  Savepoint 3: Committed");
    }

    // Final commit - includes sp1 and sp3, not sp2
    tx.commit().await?;
    println!("Main transaction committed (includes batches A and C)");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: Nested Transactions with Savepoints ===\n");

    println!("--- What Are Savepoints? ---\n");
    println!("Savepoints are checkpoints within a transaction.");
    println!("You can roll back to a savepoint without aborting");
    println!("the entire transaction.\n");

    println!("--- How It Works ---\n");
    println!("let mut tx = pool.begin().await?;      // Start transaction");
    println!("let mut savepoint = tx.begin().await?; // Create savepoint\n");

    println!("// If inner operation fails:");
    println!("savepoint.rollback().await?;           // Rollback savepoint only");
    println!("// Outer transaction continues!\n");

    println!("tx.commit().await?;                    // Commit outer transaction\n");

    println!("--- Use Cases ---\n");
    println!("1. Best-effort batch processing");
    println!("2. Try alternative approaches on failure");
    println!("3. Partial success is acceptable");
    println!("4. Log failures without losing main work\n");

    println!("--- Example Flow ---\n");
    println!("TX: Insert 'Starting'");
    println!("  SP: Try risky operation");
    println!("  SP: [FAILS] -> Rollback savepoint only");
    println!("TX: Insert 'Completed'  // Still happens!");
    println!("TX: Commit              // 'Starting' and 'Completed' saved\n");

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    match PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("--- Live Savepoint Test ---\n");

            match complex_operation(&pool).await {
                Ok(()) => println!("Complex operation completed!"),
                Err(e) => println!("Operation failed: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nSavepoints example completed!");
    Ok(())
}
