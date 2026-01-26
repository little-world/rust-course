//! Pattern 3: Basic Transactions with SQLx
//!
//! Demonstrates ACID transactions with automatic rollback on error.
//! The classic money transfer problem: debit and credit atomically.

use sqlx::{PgPool, Postgres, Transaction};

/// Transfer money between accounts atomically
async fn transfer_money(
    pool: &PgPool,
    from_account: i32,
    to_account: i32,
    amount: f64,
) -> Result<(), sqlx::Error> {
    // Start a transaction
    let mut tx: Transaction<Postgres> = pool.begin().await?;

    // Debit from account
    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(from_account)
        .execute(&mut *tx)
        .await?;

    // Credit to account
    sqlx::query("UPDATE accounts SET balance = balance + $1 WHERE id = $2")
        .bind(amount)
        .bind(to_account)
        .execute(&mut *tx)
        .await?;

    // Commit the transaction
    // If we return early (error), the transaction auto-rollbacks on drop
    tx.commit().await?;

    Ok(())
}

/// Demonstrate the RAII rollback behavior
async fn demonstrate_auto_rollback(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Do some work...
    sqlx::query("INSERT INTO logs (message) VALUES ($1)")
        .bind("Transaction started")
        .execute(&mut *tx)
        .await?;

    // Simulate an error condition
    let should_fail = true;
    if should_fail {
        // Early return WITHOUT commit
        // The transaction automatically rolls back when tx is dropped!
        return Err(sqlx::Error::Protocol("Simulated failure".into()));
    }

    // This line never reached if should_fail is true
    tx.commit().await?;
    Ok(())
}

/// Transaction with explicit rollback
async fn transaction_with_condition(
    pool: &PgPool,
    account_id: i32,
    amount: f64,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Check balance first
    let row: (f64,) = sqlx::query_as("SELECT balance FROM accounts WHERE id = $1")
        .bind(account_id)
        .fetch_one(&mut *tx)
        .await?;

    let current_balance = row.0;

    if current_balance < amount {
        // Explicit rollback (though drop would also rollback)
        tx.rollback().await?;
        println!("Insufficient funds: {} < {}", current_balance, amount);
        return Ok(());
    }

    // Proceed with withdrawal
    sqlx::query("UPDATE accounts SET balance = balance - $1 WHERE id = $2")
        .bind(amount)
        .bind(account_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    println!("Withdrawal successful!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: Basic Transactions with SQLx ===\n");

    println!("--- How Transactions Work ---\n");
    println!("1. pool.begin().await starts a transaction");
    println!("2. All queries on &mut *tx run in that transaction");
    println!("3. tx.commit().await makes changes permanent");
    println!("4. If tx drops without commit, changes roll back\n");

    println!("--- RAII Safety Pattern ---\n");
    println!("The Transaction type is a guard:");
    println!("  - Drop without commit = automatic rollback");
    println!("  - Can't forget to handle errors");
    println!("  - Type system enforces correctness\n");

    println!("--- Example: Money Transfer ---\n");
    println!("async fn transfer_money(pool, from, to, amount) {{");
    println!("    let mut tx = pool.begin().await?;");
    println!("    ");
    println!("    // Debit");
    println!("    sqlx::query(\"UPDATE accounts SET balance = balance - $1 WHERE id = $2\")");
    println!("        .bind(amount).bind(from)");
    println!("        .execute(&mut *tx).await?;");
    println!("    ");
    println!("    // Credit");
    println!("    sqlx::query(\"UPDATE accounts SET balance = balance + $1 WHERE id = $2\")");
    println!("        .bind(amount).bind(to)");
    println!("        .execute(&mut *tx).await?;");
    println!("    ");
    println!("    // If debit succeeds but credit fails, both roll back!");
    println!("    tx.commit().await?;");
    println!("    Ok(())");
    println!("}}\n");

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    match PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("--- Live Transaction Test ---\n");

            match transfer_money(&pool, 1, 2, 100.0).await {
                Ok(()) => println!("Transfer completed!"),
                Err(e) => println!("Transfer failed (rolled back): {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nTransaction example completed!");
    Ok(())
}
