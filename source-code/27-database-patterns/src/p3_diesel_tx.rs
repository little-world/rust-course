//! Pattern 3: Diesel Transactions
//!
//! Diesel uses a closure-based transaction API.
//! Return Ok to commit, return Err to rollback.

use diesel::prelude::*;
use diesel::result::Error;

// Schema definition
diesel::table! {
    accounts (id) {
        id -> Int4,
        balance -> Int4,
    }
}

#[derive(Queryable, Debug)]
#[allow(dead_code)]
struct Account {
    id: i32,
    balance: i32,
}

/// Transfer money using Diesel's transaction API
fn transfer_with_diesel(
    conn: &mut PgConnection,
    from: i32,
    to: i32,
    amount: i32,
) -> Result<(), Error> {
    conn.transaction(|conn| {
        // All operations inside this closure are in a transaction

        // Debit from account
        diesel::update(accounts::table.find(from))
            .set(accounts::balance.eq(accounts::balance - amount))
            .execute(conn)?;

        // Credit to account
        diesel::update(accounts::table.find(to))
            .set(accounts::balance.eq(accounts::balance + amount))
            .execute(conn)?;

        // Return Ok to commit, Err to rollback
        Ok(())
    })
}

/// Transaction with validation
fn withdraw_with_check(conn: &mut PgConnection, account_id: i32, amount: i32) -> Result<i32, Error> {
    conn.transaction(|conn| {
        // Get current balance
        let account: Account = accounts::table.find(account_id).first(conn)?;

        // Validate
        if account.balance < amount {
            // Return Err to rollback
            return Err(Error::RollbackTransaction);
        }

        // Proceed with withdrawal
        diesel::update(accounts::table.find(account_id))
            .set(accounts::balance.eq(accounts::balance - amount))
            .execute(conn)?;

        // Return new balance
        Ok(account.balance - amount)
    })
}

/// Nested transactions (savepoints) with Diesel
fn nested_transaction_diesel(conn: &mut PgConnection) -> Result<(), Error> {
    conn.transaction(|conn| {
        println!("Outer transaction started");

        // This creates a savepoint
        let inner_result = conn.transaction(|conn| {
            println!("  Inner transaction (savepoint) started");

            // Do some work...
            diesel::sql_query("SELECT 1").execute(conn)?;

            // Simulate failure
            Err::<(), Error>(Error::RollbackTransaction)
        });

        // Inner failure doesn't fail outer transaction
        match inner_result {
            Ok(()) => println!("  Inner succeeded"),
            Err(_) => println!("  Inner failed (savepoint rolled back)"),
        }

        println!("Outer transaction continues");

        // Outer transaction can still commit
        Ok(())
    })
}

fn explain_diesel_transactions() {
    println!("=== Diesel Transaction Pattern ===\n");

    println!("--- Closure-Based API ---\n");
    println!("conn.transaction(|conn| {{");
    println!("    // All queries here are in a transaction");
    println!("    ");
    println!("    diesel::insert_into(table)...");
    println!("    diesel::update(table)...");
    println!("    ");
    println!("    // Return Ok(value) to commit and get value back");
    println!("    // Return Err(e) to rollback and propagate error");
    println!("    Ok(result)");
    println!("}})\n");

    println!("--- Benefits ---\n");
    println!("1. Can't forget to commit or rollback");
    println!("2. The ? operator naturally triggers rollback");
    println!("3. Closure scopes the transaction clearly");
    println!("4. Nested calls create savepoints\n");

    println!("--- Comparison with SQLx ---\n");
    println!("SQLx (explicit):        Diesel (closure):");
    println!("  let tx = begin()?;      transaction(|conn| {{");
    println!("  query(&mut tx)?;          query(conn)?;");
    println!("  tx.commit()?;             Ok(())");
    println!("                          }})\n");

    println!("--- Error Handling ---\n");
    println!("// Use ? operator - any error rolls back:");
    println!("conn.transaction(|conn| {{");
    println!("    let user = users::table.find(id).first(conn)?;  // Error? Rollback!");
    println!("    diesel::update(...).execute(conn)?;             // Error? Rollback!");
    println!("    Ok(user)  // Success? Commit!");
    println!("}})\n");

    println!("// Explicit rollback:");
    println!("if some_condition {{");
    println!("    return Err(Error::RollbackTransaction);");
    println!("}}\n");
}

fn main() {
    println!("=== Pattern 3: Diesel Transactions ===\n");

    explain_diesel_transactions();

    // Try to connect
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Connection Test ---\n");

    match PgConnection::establish(&database_url) {
        Ok(mut conn) => {
            println!("Connected successfully!\n");

            match transfer_with_diesel(&mut conn, 1, 2, 100) {
                Ok(()) => println!("Transfer completed!"),
                Err(e) => println!("Transfer failed: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nDiesel transactions example completed!");
}
