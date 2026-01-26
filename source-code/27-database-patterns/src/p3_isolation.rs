//! Pattern 3: Transaction Isolation Levels
//!
//! Different isolation levels provide different consistency guarantees.
//! Higher isolation = more consistency but more contention.

use sqlx::PgPool;

/// Set transaction isolation level
async fn serializable_transaction(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Set isolation level - must be first statement in transaction
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *tx)
        .await?;

    // Perform operations with serializable isolation
    // This prevents phantom reads and ensures true serializability

    // ... your queries here ...

    tx.commit().await?;

    Ok(())
}

/// Repeatable read isolation
async fn repeatable_read_transaction(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut *tx)
        .await?;

    // All reads in this transaction see a consistent snapshot
    // Re-reading the same row returns the same data

    tx.commit().await?;

    Ok(())
}

/// Read committed (PostgreSQL default)
async fn read_committed_transaction(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // This is the default, but can be explicit
    sqlx::query("SET TRANSACTION ISOLATION LEVEL READ COMMITTED")
        .execute(&mut *tx)
        .await?;

    // Only sees committed data
    // But re-reading might see new commits

    tx.commit().await?;

    Ok(())
}

fn explain_isolation_levels() {
    println!("=== Transaction Isolation Levels ===\n");

    println!("--- The Four Standard Levels ---\n");

    println!("1. READ UNCOMMITTED");
    println!("   - Can see uncommitted changes (dirty reads)");
    println!("   - Not supported in PostgreSQL (treated as READ COMMITTED)");
    println!("   - Almost never used\n");

    println!("2. READ COMMITTED (PostgreSQL default)");
    println!("   - Only sees committed data");
    println!("   - Re-reading same row might return different values");
    println!("   - Good for most web applications\n");

    println!("3. REPEATABLE READ");
    println!("   - Sees consistent snapshot for duration of transaction");
    println!("   - Re-reads return same data");
    println!("   - Phantom rows can still appear in range queries");
    println!("   - Good for reports that need consistency\n");

    println!("4. SERIALIZABLE");
    println!("   - Full isolation, as if transactions ran one-at-a-time");
    println!("   - May fail with serialization error (retry needed!)");
    println!("   - Good for financial transactions\n");

    println!("--- Anomalies Prevented ---\n");
    println!("+------------------+-------+-------+-------+-------+");
    println!("| Anomaly          | R.U.  | R.C.  | R.R.  | SER.  |");
    println!("+------------------+-------+-------+-------+-------+");
    println!("| Dirty Read       |   X   |   -   |   -   |   -   |");
    println!("| Non-repeatable   |   X   |   X   |   -   |   -   |");
    println!("| Phantom Read     |   X   |   X   |   X   |   -   |");
    println!("+------------------+-------+-------+-------+-------+");
    println!("X = possible, - = prevented\n");

    println!("--- Choosing an Isolation Level ---\n");

    println!("READ COMMITTED (default):");
    println!("  - Web applications with short transactions");
    println!("  - Most CRUD operations");
    println!("  - When latest data is preferred\n");

    println!("REPEATABLE READ:");
    println!("  - Reports spanning multiple queries");
    println!("  - Backup operations");
    println!("  - When consistent view is important\n");

    println!("SERIALIZABLE:");
    println!("  - Financial transactions");
    println!("  - Inventory systems");
    println!("  - When correctness is critical");
    println!("  - Requires retry logic for serialization failures!\n");
}

fn demonstrate_serialization_retry() {
    println!("--- Serialization Retry Pattern ---\n");

    println!("async fn with_retry<F, T>(pool: &PgPool, f: F) -> Result<T, Error>");
    println!("where F: Fn(&mut Transaction) -> Future<Output = Result<T, Error>>");
    println!("{{");
    println!("    loop {{");
    println!("        let mut tx = pool.begin().await?;");
    println!("        sqlx::query(\"SET TRANSACTION ISOLATION LEVEL SERIALIZABLE\")");
    println!("            .execute(&mut *tx).await?;");
    println!("        ");
    println!("        match f(&mut tx).await {{");
    println!("            Ok(result) => {{");
    println!("                tx.commit().await?;");
    println!("                return Ok(result);");
    println!("            }}");
    println!("            Err(e) if is_serialization_error(&e) => {{");
    println!("                // Retry!");
    println!("                continue;");
    println!("            }}");
    println!("            Err(e) => return Err(e),");
    println!("        }}");
    println!("    }}");
    println!("}}\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: Transaction Isolation Levels ===\n");

    explain_isolation_levels();
    demonstrate_serialization_retry();

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    match PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("--- Live Isolation Test ---\n");

            match serializable_transaction(&pool).await {
                Ok(()) => println!("Serializable transaction completed!"),
                Err(e) => println!("Transaction failed: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nIsolation levels example completed!");
    Ok(())
}
