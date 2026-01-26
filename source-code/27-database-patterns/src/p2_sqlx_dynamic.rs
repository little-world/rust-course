//! Pattern 2: Dynamic Queries with SQLx QueryBuilder
//!
//! Demonstrates building dynamic SQL queries at runtime while
//! maintaining SQL injection protection through parameter binding.

use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};

#[derive(FromRow, Debug)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

/// Dynamic search with optional filters
async fn search_users(
    pool: &PgPool,
    username_filter: Option<&str>,
    email_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
    // QueryBuilder for dynamic queries
    // Start with base query - WHERE 1=1 trick simplifies adding AND clauses
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT id, username, email, created_at FROM users WHERE 1=1");

    // Add optional filters
    if let Some(username) = username_filter {
        query_builder.push(" AND username LIKE ");
        query_builder.push_bind(format!("%{}%", username));
    }

    if let Some(email) = email_filter {
        query_builder.push(" AND email LIKE ");
        query_builder.push_bind(format!("%{}%", email));
    }

    // Add limit
    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);

    // Build and execute
    let users = query_builder
        .build_query_as::<User>()
        .fetch_all(pool)
        .await?;

    Ok(users)
}

/// Demonstrate dynamic ORDER BY
async fn search_with_sort(
    pool: &PgPool,
    sort_column: &str,
    sort_direction: &str,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT id, username, email, created_at FROM users");

    // Validate sort column to prevent SQL injection
    // (can't use push_bind for column names)
    let valid_columns = ["id", "username", "email", "created_at"];
    let column = if valid_columns.contains(&sort_column) {
        sort_column
    } else {
        "id"
    };

    let direction = if sort_direction.to_uppercase() == "DESC" {
        "DESC"
    } else {
        "ASC"
    };

    // ORDER BY can't use bind parameters for column names
    query_builder.push(format!(" ORDER BY {} {}", column, direction));

    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);

    query_builder
        .build_query_as::<User>()
        .fetch_all(pool)
        .await
}

/// Demonstrate IN clause with dynamic values
async fn find_users_by_ids(pool: &PgPool, ids: &[i32]) -> Result<Vec<User>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query_builder: QueryBuilder<Postgres> =
        QueryBuilder::new("SELECT id, username, email, created_at FROM users WHERE id IN (");

    // Add separated values
    let mut separated = query_builder.separated(", ");
    for id in ids {
        separated.push_bind(*id);
    }
    separated.push_unseparated(")");

    query_builder
        .build_query_as::<User>()
        .fetch_all(pool)
        .await
}

fn demonstrate_query_building() {
    println!("=== Dynamic Query Building ===\n");

    println!("--- The WHERE 1=1 Trick ---\n");
    println!("Start with: SELECT * FROM users WHERE 1=1");
    println!("Then add:   AND username LIKE $1");
    println!("And:        AND email LIKE $2");
    println!("No need to track if it's the first condition!\n");

    println!("--- QueryBuilder Methods ---\n");
    println!("push(sql)       - Add raw SQL (be careful!)");
    println!("push_bind(val)  - Add parameterized value (safe)");
    println!("separated(sep)  - Helper for IN clauses\n");

    println!("--- Example Queries ---\n");

    // Show what the queries would look like
    println!("search_users(None, None, 10):");
    println!("  SELECT ... FROM users WHERE 1=1 LIMIT $1\n");

    println!("search_users(Some(\"alice\"), None, 10):");
    println!("  SELECT ... FROM users WHERE 1=1 AND username LIKE $1 LIMIT $2\n");

    println!("search_users(Some(\"alice\"), Some(\"gmail\"), 10):");
    println!("  SELECT ... WHERE 1=1 AND username LIKE $1 AND email LIKE $2 LIMIT $3\n");

    println!("find_users_by_ids(&[1, 2, 3]):");
    println!("  SELECT ... WHERE id IN ($1, $2, $3)\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 2: Dynamic Queries with SQLx QueryBuilder ===\n");

    demonstrate_query_building();

    println!("--- Key Points ---\n");
    println!("1. push_bind() prevents SQL injection (parameterized)");
    println!("2. Can't use bind for column/table names - validate manually!");
    println!("3. Lose compile-time verification (runtime SQL)");
    println!("4. Essential for search endpoints with optional filters\n");

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    match PgPool::connect(&database_url).await {
        Ok(pool) => {
            println!("--- Live Query Test ---\n");

            match search_users(&pool, Some("alice"), None, 10).await {
                Ok(users) => println!("Found {} users", users.len()),
                Err(e) => println!("Query error: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)");
        }
    }

    println!("\nDynamic query example completed!");
    Ok(())
}
