### SQLx Cheat Sheet
```rust
// Cargo.toml:
/*
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "tls-rustls", "postgres", "mysql", "sqlite", "chrono", "uuid", "migrate"] }
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
*/

use sqlx::{
    postgres::{PgPool, PgPoolOptions, PgRow, Postgres},
    mysql::{MySqlPool, MySqlPoolOptions},
    sqlite::{SqlitePool, SqlitePoolOptions},
    Row, FromRow, Executor, query, query_as, query_scalar,
};
use chrono::NaiveDateTime;
use uuid::Uuid;

// ===== DATABASE CONNECTION =====
// PostgreSQL connection pool
#[tokio::main]
async fn pg_pool() -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)                                  // Max connections
        .min_connections(1)                                  // Min connections
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect("postgres://user:pass@localhost/dbname")
        .await?;
    
    Ok(pool)
}

// MySQL connection pool
async fn mysql_pool() -> Result<MySqlPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://user:pass@localhost/dbname")
        .await?;
    
    Ok(pool)
}

// SQLite connection pool
async fn sqlite_pool() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:test.db")
        .await?;
    
    Ok(pool)
}

// Single connection (no pool)
async fn single_connection() -> Result<sqlx::PgConnection, sqlx::Error> {
    use sqlx::Connection;
    
    let conn = sqlx::PgConnection::connect("postgres://user:pass@localhost/dbname")
        .await?;
    
    Ok(conn)
}

// ===== MODELS =====
use serde::{Deserialize, Serialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: NaiveDateTime,
}

#[derive(Debug, FromRow, Serialize)]
struct Post {
    id: i32,
    title: String,
    body: String,
    published: bool,
    user_id: i32,
    created_at: NaiveDateTime,
}

// Partial model for inserts
#[derive(Debug)]
struct NewUser {
    username: String,
    email: String,
}

// ===== CREATE (INSERT) =====
// Insert single record
async fn create_user(pool: &PgPool, username: &str, email: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *"
    )
    .bind(username)
    .bind(email)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}

// Insert and get ID only
async fn create_user_get_id(pool: &PgPool, username: &str, email: &str) -> Result<i32, sqlx::Error> {
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id"
    )
    .bind(username)
    .bind(email)
    .fetch_one(pool)
    .await?;
    
    Ok(id)
}

// Insert multiple records
async fn create_users_bulk(pool: &PgPool, users: Vec<NewUser>) -> Result<Vec<User>, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut results = Vec::new();
    
    for user in users {
        let inserted = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *"
        )
        .bind(&user.username)
        .bind(&user.email)
        .fetch_one(&mut *tx)
        .await?;
        
        results.push(inserted);
    }
    
    tx.commit().await?;
    Ok(results)
}

// Insert with ON CONFLICT (upsert)
async fn upsert_user(pool: &PgPool, username: &str, email: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email)
        VALUES ($1, $2)
        ON CONFLICT (email) DO UPDATE
        SET username = EXCLUDED.username
        RETURNING *
        "#
    )
    .bind(username)
    .bind(email)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}

// ===== READ (SELECT) =====
// Get all records
async fn get_all_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool)
        .await?;
    
    Ok(users)
}

// Get single record by ID
async fn get_user_by_id(pool: &PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    
    Ok(user)
}

// Get one record (errors if not found or multiple found)
async fn get_user_one(pool: &PgPool, id: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await?;
    
    Ok(user)
}

// Filter with WHERE clause
async fn find_users_by_username(pool: &PgPool, username: &str) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE username = $1"
    )
    .bind(username)
    .fetch_all(pool)
    .await?;
    
    Ok(users)
}

// Complex query with multiple conditions
async fn find_users_complex(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE username LIKE $1
        AND email IS NOT NULL
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind("%john%")
    .bind(10i64)
    .bind(0i64)
    .fetch_all(pool)
    .await?;
    
    Ok(users)
}

// Select specific columns
async fn get_usernames(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let usernames: Vec<String> = sqlx::query_scalar("SELECT username FROM users")
        .fetch_all(pool)
        .await?;
    
    Ok(usernames)
}

// Select multiple columns (tuple)
async fn get_user_info(pool: &PgPool) -> Result<Vec<(i32, String, String)>, sqlx::Error> {
    let info = sqlx::query_as::<_, (i32, String, String)>(
        "SELECT id, username, email FROM users"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(info)
}

// Count records
async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    
    Ok(count)
}

// Stream results (for large datasets)
use futures::TryStreamExt;

async fn stream_users(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut rows = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch(pool);
    
    while let Some(user) = rows.try_next().await? {
        println!("User: {:?}", user);
    }
    
    Ok(())
}

// ===== UPDATE =====
// Update single record
async fn update_user(pool: &PgPool, id: i32, new_username: &str) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET username = $1 WHERE id = $2 RETURNING *"
    )
    .bind(new_username)
    .bind(id)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}

// Update multiple fields
async fn update_user_fields(
    pool: &PgPool,
    id: i32,
    username: Option<&str>,
    email: Option<&str>,
) -> Result<User, sqlx::Error> {
    let mut query = String::from("UPDATE users SET ");
    let mut params: Vec<String> = Vec::new();
    let mut param_num = 1;
    
    if let Some(u) = username {
        params.push(format!("username = ${}", param_num));
        param_num += 1;
    }
    
    if let Some(e) = email {
        if !params.is_empty() {
            query.push_str(", ");
        }
        params.push(format!("email = ${}", param_num));
        param_num += 1;
    }
    
    query.push_str(&params.join(", "));
    query.push_str(&format!(" WHERE id = ${} RETURNING *", param_num));
    
    // Note: Dynamic query building like this requires manual parameter binding
    // In production, use a query builder or macro
    
    let user = sqlx::query_as::<_, User>(&query)
        .bind(username)
        .bind(email)
        .bind(id)
        .fetch_one(pool)
        .await?;
    
    Ok(user)
}

// Update multiple records
async fn update_all_users(pool: &PgPool, new_username: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE users SET username = $1")
        .bind(new_username)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected())
}

// Conditional update
async fn update_inactive_users(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE users SET username = 'inactive' WHERE created_at < NOW() - INTERVAL '30 days'"
    )
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected())
}

// ===== DELETE =====
// Delete single record
async fn delete_user(pool: &PgPool, id: i32) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected())
}

// Delete with filter
async fn delete_users_by_username(pool: &PgPool, username: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(username)
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected())
}

// Delete all
async fn delete_all_users(pool: &PgPool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users")
        .execute(pool)
        .await?;
    
    Ok(result.rows_affected())
}

// ===== JOINS =====
// Inner join
#[derive(Debug, FromRow)]
struct UserWithPost {
    user_id: i32,
    username: String,
    post_id: i32,
    title: String,
}

async fn get_users_with_posts(pool: &PgPool) -> Result<Vec<UserWithPost>, sqlx::Error> {
    let results = sqlx::query_as::<_, UserWithPost>(
        r#"
        SELECT 
            u.id as user_id,
            u.username,
            p.id as post_id,
            p.title
        FROM users u
        INNER JOIN posts p ON u.id = p.user_id
        "#
    )
    .fetch_all(pool)
    .await?;
    
    Ok(results)
}

// Left join
async fn get_all_users_and_posts(pool: &PgPool) -> Result<Vec<(User, Option<Post>)>, sqlx::Error> {
    let results = sqlx::query(
        r#"
        SELECT 
            u.id, u.username, u.email, u.created_at,
            p.id, p.title, p.body, p.published, p.user_id, p.created_at
        FROM users u
        LEFT JOIN posts p ON u.id = p.user_id
        "#
    )
    .fetch_all(pool)
    .await?;
    
    // Manual mapping needed for Option<Post>
    let mut user_posts = Vec::new();
    for row in results {
        let user = User {
            id: row.get(0),
            username: row.get(1),
            email: row.get(2),
            created_at: row.get(3),
        };
        
        let post = if let Ok(post_id) = row.try_get::<i32, _>(4) {
            Some(Post {
                id: post_id,
                title: row.get(5),
                body: row.get(6),
                published: row.get(7),
                user_id: row.get(8),
                created_at: row.get(9),
            })
        } else {
            None
        };
        
        user_posts.push((user, post));
    }
    
    Ok(user_posts)
}

// Get posts for specific user
async fn get_user_posts(pool: &PgPool, user_id: i32) -> Result<Vec<Post>, sqlx::Error> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT * FROM posts WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    
    Ok(posts)
}

// ===== TRANSACTIONS =====
async fn transaction_example(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Insert user
    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *"
    )
    .bind("john")
    .bind("john@example.com")
    .fetch_one(&mut *tx)
    .await?;
    
    // Insert post
    sqlx::query(
        "INSERT INTO posts (title, body, user_id) VALUES ($1, $2, $3)"
    )
    .bind("First Post")
    .bind("Hello, World!")
    .bind(user.id)
    .execute(&mut *tx)
    .await?;
    
    tx.commit().await?;
    Ok(())
}

// Rollback on error
async fn transaction_with_rollback(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    match sqlx::query("INSERT INTO users (username, email) VALUES ($1, $2)")
        .bind("test")
        .bind("test@example.com")
        .execute(&mut *tx)
        .await
    {
        Ok(_) => tx.commit().await?,
        Err(e) => {
            tx.rollback().await?;
            return Err(e);
        }
    }
    
    Ok(())
}

// Savepoints
async fn transaction_with_savepoint(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    sqlx::query("INSERT INTO users (username, email) VALUES ($1, $2)")
        .bind("user1")
        .bind("user1@example.com")
        .execute(&mut *tx)
        .await?;
    
    // Create savepoint
    sqlx::query("SAVEPOINT sp1")
        .execute(&mut *tx)
        .await?;
    
    // This might fail
    let result = sqlx::query("INSERT INTO users (username, email) VALUES ($1, $2)")
        .bind("user2")
        .bind("invalid-email")
        .execute(&mut *tx)
        .await;
    
    if result.is_err() {
        // Rollback to savepoint
        sqlx::query("ROLLBACK TO SAVEPOINT sp1")
            .execute(&mut *tx)
            .await?;
    }
    
    tx.commit().await?;
    Ok(())
}

// ===== PREPARED STATEMENTS =====
async fn prepared_statements(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Statements are automatically prepared and cached
    for i in 1..10 {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(i)
        .fetch_optional(pool)
        .await?;
    }
    
    Ok(())
}

// ===== QUERY MACROS (Compile-time checked) =====
// Requires DATABASE_URL environment variable at compile time

async fn compile_time_checked(pool: &PgPool) -> Result<(), sqlx::Error> {
    // query! macro - compile-time SQL checking
    let user = sqlx::query!(
        "SELECT id, username, email FROM users WHERE id = $1",
        1i32
    )
    .fetch_one(pool)
    .await?;
    
    println!("User: {} {}", user.id, user.username);
    
    // query_as! macro - with FromRow
    let users = sqlx::query_as!(
        User,
        "SELECT * FROM users"
    )
    .fetch_all(pool)
    .await?;
    
    // query_scalar! macro
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM users"
    )
    .fetch_one(pool)
    .await?;
    
    Ok(())
}

// ===== MIGRATIONS =====
// Run migrations from code
async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    
    Ok(())
}

// Create migration file: migrations/TIMESTAMP_create_users.sql
/*
-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Rollback
-- DROP TABLE users;
*/

// ===== AGGREGATIONS =====
async fn count_posts_by_user(pool: &PgPool, user_id: i32) -> Result<i64, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM posts WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    
    Ok(count)
}

// Group by
#[derive(Debug, FromRow)]
struct PostCount {
    user_id: i32,
    count: i64,
}

async fn posts_count_by_user(pool: &PgPool) -> Result<Vec<PostCount>, sqlx::Error> {
    let counts = sqlx::query_as::<_, PostCount>(
        "SELECT user_id, COUNT(*) as count FROM posts GROUP BY user_id"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(counts)
}

// ===== PAGINATION =====
async fn paginate_users(
    pool: &PgPool,
    page: i64,
    per_page: i64,
) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users LIMIT $1 OFFSET $2"
    )
    .bind(per_page)
    .bind(page * per_page)
    .fetch_all(pool)
    .await?;
    
    Ok(users)
}

// ===== SUBQUERIES =====
async fn users_with_posts(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE id IN (SELECT DISTINCT user_id FROM posts)
        "#
    )
    .fetch_all(pool)
    .await?;
    
    Ok(users)
}

// ===== CUSTOM TYPES =====
// Enum type
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
enum UserRole {
    Admin,
    User,
    Guest,
}

#[derive(Debug, FromRow)]
struct UserWithRole {
    id: i32,
    username: String,
    role: UserRole,
}

// JSON type
use serde_json::Value as JsonValue;

#[derive(Debug, FromRow)]
struct UserWithMetadata {
    id: i32,
    username: String,
    metadata: JsonValue,
}

async fn insert_json(pool: &PgPool) -> Result<(), sqlx::Error> {
    let metadata = serde_json::json!({
        "age": 30,
        "city": "New York"
    });
    
    sqlx::query(
        "INSERT INTO users (username, metadata) VALUES ($1, $2)"
    )
    .bind("john")
    .bind(metadata)
    .execute(pool)
    .await?;
    
    Ok(())
}

// ===== COMMON PATTERNS =====

// Pattern 1: Find or create
async fn find_or_create_user(
    pool: &PgPool,
    email: &str,
) -> Result<User, sqlx::Error> {
    if let Some(user) = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?
    {
        Ok(user)
    } else {
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING *"
        )
        .bind(email.split('@').next().unwrap())
        .bind(email)
        .fetch_one(pool)
        .await?;
        
        Ok(user)
    }
}

// Pattern 2: Batch insert
async fn batch_insert(pool: &PgPool, users: Vec<NewUser>) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    for user in users {
        sqlx::query(
            "INSERT INTO users (username, email) VALUES ($1, $2)"
        )
        .bind(&user.username)
        .bind(&user.email)
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(())
}

// Pattern 3: Exists check
async fn user_exists(pool: &PgPool, email: &str) -> Result<bool, sqlx::Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
    )
    .bind(email)
    .fetch_one(pool)
    .await?;
    
    Ok(exists)
}

// Pattern 4: Soft delete
async fn soft_delete(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET deleted_at = NOW() WHERE id = $1"
    )
    .bind(id)
    .execute(pool)
    .await?;
    
    Ok(())
}

async fn get_active_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE deleted_at IS NULL"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(users)
}

// Pattern 5: Connection testing
async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    
    Ok(())
}

// Pattern 6: Dynamic query building
async fn dynamic_search(
    pool: &PgPool,
    username: Option<&str>,
    email: Option<&str>,
) -> Result<Vec<User>, sqlx::Error> {
    let mut query = String::from("SELECT * FROM users WHERE 1=1");
    let mut bindings = Vec::new();
    
    if let Some(u) = username {
        query.push_str(&format!(" AND username LIKE ${}", bindings.len() + 1));
        bindings.push(format!("%{}%", u));
    }
    
    if let Some(e) = email {
        query.push_str(&format!(" AND email = ${}", bindings.len() + 1));
        bindings.push(e.to_string());
    }
    
    let mut q = sqlx::query_as::<_, User>(&query);
    for binding in bindings {
        q = q.bind(binding);
    }
    
    let users = q.fetch_all(pool).await?;
    Ok(users)
}
```