//! Pattern 3: Optimistic Locking
//!
//! Detect conflicts at write time rather than preventing them with locks.
//! Ideal for long-running edits and distributed systems.

use sqlx::{FromRow, PgPool};

#[derive(FromRow, Debug)]
struct Document {
    id: i32,
    content: String,
    version: i32,
}

/// Update document with optimistic locking
async fn update_with_optimistic_lock(
    pool: &PgPool,
    doc_id: i32,
    new_content: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 5;

    for attempt in 1..=max_retries {
        // Fetch current document (with version)
        let doc: Document =
            sqlx::query_as("SELECT id, content, version FROM documents WHERE id = $1")
                .bind(doc_id)
                .fetch_one(pool)
                .await?;

        println!("Attempt {}: Read version {}", attempt, doc.version);

        // Try to update if version hasn't changed
        let result = sqlx::query(
            r#"
            UPDATE documents
            SET content = $1, version = version + 1
            WHERE id = $2 AND version = $3
            "#,
        )
        .bind(&new_content)
        .bind(doc_id)
        .bind(doc.version)
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            // Success - we updated it
            println!("Update successful on attempt {}", attempt);
            return Ok(());
        }

        // Someone else updated it - retry
        println!("Conflict detected, retrying...");
        tokio::time::sleep(std::time::Duration::from_millis(100 * attempt as u64)).await;
    }

    Err("Max retries exceeded".into())
}

/// Optimistic lock with user-friendly conflict handling
async fn update_with_conflict_resolution(
    pool: &PgPool,
    doc_id: i32,
    new_content: String,
    expected_version: i32,
) -> Result<UpdateResult, Box<dyn std::error::Error>> {
    // Try to update with expected version
    let result = sqlx::query(
        r#"
        UPDATE documents
        SET content = $1, version = version + 1
        WHERE id = $2 AND version = $3
        "#,
    )
    .bind(&new_content)
    .bind(doc_id)
    .bind(expected_version)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        return Ok(UpdateResult::Success);
    }

    // Conflict - fetch current state for user to resolve
    let current: Document =
        sqlx::query_as("SELECT id, content, version FROM documents WHERE id = $1")
            .bind(doc_id)
            .fetch_one(pool)
            .await?;

    Ok(UpdateResult::Conflict {
        current_version: current.version,
        current_content: current.content,
    })
}

enum UpdateResult {
    Success,
    Conflict {
        current_version: i32,
        current_content: String,
    },
}

fn explain_optimistic_locking() {
    println!("=== Optimistic Locking ===\n");

    println!("--- How It Works ---\n");
    println!("1. Each row has a 'version' column");
    println!("2. Read the row (including version)");
    println!("3. UPDATE ... WHERE id = $id AND version = $old_version");
    println!("4. If rows_affected() == 0, someone else updated it");
    println!("5. Retry or notify user of conflict\n");

    println!("--- The SQL Pattern ---\n");
    println!("UPDATE documents");
    println!("SET content = $1, version = version + 1");
    println!("WHERE id = $2 AND version = $3\n");

    println!("If another transaction updated the row,");
    println!("version won't match and UPDATE affects 0 rows.\n");

    println!("--- When to Use ---\n");
    println!("Good for:");
    println!("  - Long-running user edits (document editing)");
    println!("  - Distributed systems where locks are expensive");
    println!("  - Read-heavy workloads (conflicts are rare)");
    println!("  - APIs with 'If-Match' headers\n");

    println!("Not ideal for:");
    println!("  - High-contention scenarios (retry storms)");
    println!("  - Very short transactions (use pessimistic locking)\n");

    println!("--- Comparison ---\n");
    println!("Pessimistic (SELECT FOR UPDATE):");
    println!("  - Lock acquired before read");
    println!("  - Other transactions wait");
    println!("  - Safe but can cause deadlocks\n");

    println!("Optimistic (version check):");
    println!("  - No locks held during edit");
    println!("  - Conflict detected at write time");
    println!("  - Requires retry/merge logic\n");

    println!("--- Retry Strategy ---\n");
    println!("for attempt in 1..=max_retries {{");
    println!("    let doc = fetch_with_version(id).await?;");
    println!("    ");
    println!("    if try_update(id, content, doc.version).await? > 0 {{");
    println!("        return Ok(());  // Success!");
    println!("    }}");
    println!("    ");
    println!("    // Conflict - wait and retry");
    println!("    sleep(backoff * attempt).await;");
    println!("}}");
    println!("return Err(\"Max retries exceeded\");\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: Optimistic Locking ===\n");

    explain_optimistic_locking();

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Live Test ---\n");

    match PgPool::connect(&database_url).await {
        Ok(pool) => {
            match update_with_optimistic_lock(&pool, 1, "New content".to_string()).await {
                Ok(()) => println!("Document updated successfully!"),
                Err(e) => println!("Update failed: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nOptimistic locking example completed!");
    Ok(())
}
