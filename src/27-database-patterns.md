# Database Patterns

This chapter explores database patterns in Rust: connection pooling for resource management, query builders for type safety, transactions for data integrity, migrations for schema evolution, and choosing between ORM and raw SQL based on specific requirements.

## Pattern 1: Connection Pooling

**Problem**: Opening a fresh database connection per request costs handshakes, auth, and session setup, quickly exhausting `max_connections` and adding 100ms+ latency to every query.

**Solution**: Keep a pool of ready connections (r2d2, deadpool). Borrow them with `pool.get()`, configure sensible `max_size`, `min_idle`, and timeouts, and let the smart pointer return them on drop.

**Why It Matters**: Reusing 10–20 pooled connections serves hundreds of requests with <1 ms checkout time, protects the database from connection storms, and preserves prepared statements/buffers.

**Use Cases**: Web servers, background workers, GraphQL resolvers, CLI tools hitting DBs repeatedly, serverless globals, and integration test harnesses needing many short-lived queries.

### Example: r2d2: The Classic Connection Pool

r2d2 (Resource Reuse & Recycling Daemon) is Rust's original generic connection pool. It works with any resource that implements its traits, not just databases. This makes it extremely flexible.

```rust
// Add to Cargo.toml:
// r2d2 = "0.8"
// r2d2_postgres = "0.18"
// postgres = "0.19"

use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{PostgresConnectionManager, postgres::NoTls};
use std::error::Error;

type PostgresPool = Pool<PostgresConnectionManager<NoTls>>;
type PostgresPooledConnection = PooledConnection<PostgresConnectionManager<NoTls>>;

/// Initialize a connection pool
fn create_pool(database_url: &str) -> Result<PostgresPool, Box<dyn Error>> {
    let manager = PostgresConnectionManager::new(
        database_url.parse()?,
        NoTls,
    );

    let pool = Pool::builder()
        .max_size(15)                    // Maximum 15 connections
        .min_idle(Some(5))               // Keep at least 5 idle connections
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)?;

    Ok(pool)
}

/// Example: Using the pool
async fn fetch_user(pool: &PostgresPool, user_id: i32) -> Result<String, Box<dyn Error>> {
    // Get a connection from the pool
    // This blocks if all connections are in use, until one becomes available
    let mut conn = pool.get()?;

    // Use the connection
    let row = conn.query_one(
        "SELECT username FROM users WHERE id = $1",
        &[&user_id],
    )?;

    let username: String = row.get(0);

    // Connection automatically returns to pool when `conn` is dropped
    Ok(username)
}

fn main() -> Result<(), Box<dyn Error>> {
    let pool = create_pool("postgresql://user:pass@localhost/mydb")?;

    // The pool can be cloned cheaply (Arc internally)
    // and shared across threads
    let pool_clone = pool.clone();

    let handle = std::thread::spawn(move || {
        fetch_user(&pool_clone, 42)
    });

    // Both threads can use the pool concurrently
    let username = fetch_user(&pool, 42)?;
    println!("User: {}", username);

    handle.join().unwrap()?;

    Ok(())
}
```

The beauty of this pattern is in the `PooledConnection` type. It's a smart pointer that automatically returns the connection to the pool when dropped. You can't forget to return it—Rust's ownership system enforces correct cleanup.

### Example: Tuning Pool Configuration

Pool configuration significantly impacts performance. Here's what each parameter controls:

```rust
use r2d2::Pool;
use std::time::Duration;

fn configure_pool_detailed(manager: PostgresConnectionManager<NoTls>) -> PostgresPool {
    Pool::builder()
        // Maximum number of connections to create
        // Higher = more concurrent requests, but more database load
        .max_size(20)

        // Minimum idle connections to maintain
        // Higher = faster response for bursts, but more idle resources
        .min_idle(Some(5))

        // How long to wait for a connection before timing out
        // Too low = errors during load spikes
        // Too high = slow responses when pool exhausted
        .connection_timeout(Duration::from_secs(10))

        // Test connections before use to ensure they're alive
        // Adds overhead but prevents using dead connections
        .test_on_check_out(true)

        // How long a connection can be idle before being closed
        // Prevents accumulating stale connections
        .idle_timeout(Some(Duration::from_secs(300)))

        // Maximum lifetime of a connection before forced recreation
        // Ensures connections don't grow stale over time
        .max_lifetime(Some(Duration::from_secs(1800)))

        .build(manager)
        .unwrap()
}
```

For a typical web application:
- **Small app** (< 100 concurrent users): max_size = 5-10
- **Medium app** (100-1000 users): max_size = 10-20
- **Large app** (1000+ users): max_size = 20-50

Going higher than 50 often indicates other bottlenecks.

### Example: deadpool: Async-First Connection Pooling

While r2d2 works well with blocking I/O, async applications need async-aware pooling. deadpool is designed specifically for async/await:

```rust
// Add to Cargo.toml:
// deadpool = { version = "0.10", features = ["managed"] }
// deadpool-postgres = "0.12"
// tokio-postgres = "0.7"
// tokio = { version = "1", features = ["full"] }

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;
use std::error::Error;

/// Create an async connection pool
fn create_async_pool() -> Result<Pool, Box<dyn Error>> {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.user = Some("user".to_string());
    cfg.password = Some("password".to_string());
    cfg.dbname = Some("mydb".to_string());

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });

    cfg.pool = Some(deadpool::managed::PoolConfig {
        max_size: 20,
        timeouts: deadpool::managed::Timeouts {
            wait: Some(std::time::Duration::from_secs(5)),
            create: Some(std::time::Duration::from_secs(5)),
            recycle: Some(std::time::Duration::from_secs(5)),
        },
    });

    Ok(cfg.create_pool(None, NoTls)?)
}

/// Fetch user asynchronously
async fn fetch_user_async(pool: &Pool, user_id: i32) -> Result<String, Box<dyn Error>> {
    // Get connection asynchronously
    // This awaits instead of blocking
    let client = pool.get().await?;

    // Execute query
    let row = client
        .query_one("SELECT username FROM users WHERE id = $1", &[&user_id])
        .await?;

    let username: String = row.get(0);

    // Connection returns to pool on drop
    Ok(username)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = create_async_pool()?;

    // Can handle many concurrent queries efficiently
    let tasks = (1..=100).map(|id| {
        let pool = pool.clone();
        tokio::spawn(async move {
            fetch_user_async(&pool, id).await
        })
    });

    // Wait for all queries
    let results = futures::future::join_all(tasks).await;

    println!("Completed {} queries", results.len());

    Ok(())
}
```

The key difference: deadpool's `get()` returns a `Future` that yields to other tasks while waiting for a connection. With r2d2, a thread would block. This makes deadpool much more efficient for high-concurrency async applications.

### Example: Health Checks and Monitoring

Production applications need to monitor pool health:

```rust
use deadpool_postgres::Pool;

async fn monitor_pool_health(pool: &Pool) {
    let status = pool.status();

    println!("Pool status:");
    println!("  Available connections: {}", status.available);
    println!("  Size: {}", status.size);
    println!("  Max size: {}", status.max_size);

    // Alert if pool is exhausted
    if status.available == 0 {
        eprintln!("WARNING: Connection pool exhausted!");
    }

    // Alert if pool is mostly idle (might be oversized)
    if status.available > status.max_size * 3 / 4 {
        eprintln!("INFO: Pool mostly idle, consider reducing size");
    }
}

/// Graceful shutdown
async fn shutdown_pool(pool: Pool) {
    println!("Shutting down pool...");

    // Close the pool
    pool.close();

    // Give active connections time to finish
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    println!("Pool shutdown complete");
}
```

Monitoring helps you tune pool size and detect issues before they impact users.

## Pattern 2: Query Builders

**Problem**: Raw SQL strings hide typos, wrong parameter counts, and type mismatches until runtime—and string concatenation invites SQL injection.

**Solution**: Use compile-time checked builders: SQLx `query!` verifies syntax/columns/types, Diesel’s schema DSL provides type-safe composable queries, and SQLx’s `QueryBuilder` binds parameters for dynamic clauses.

**Why It Matters**: Mistakes surface as compiler errors instead of production crashes, refactors update in one place, and bound parameters shut the door on injection.

**Use Cases**: Everyday CRUD endpoints, GraphQL resolvers, dashboards with dynamic filters, reporting queries, and any service that needs confidence its SQL matches the schema.

### Example: SQLx: The Compile-Time Checked Query Builder

SQLx is remarkable: it connects to your database at compile time and verifies your queries. This means SQL errors become compile errors:

```rust
// Add to Cargo.toml:
// sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "macros"] }

use sqlx::{PgPool, FromRow};

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

/// Compile-time verified query
async fn fetch_user(pool: &PgPool, user_id: i32) -> Result<User, sqlx::Error> {
    // The query! macro verifies this SQL at compile time
    // It checks:
    // - SQL syntax is valid
    // - Table and columns exist
    // - Parameter types match
    // - Return types match
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, email, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

/// Insert with query!
async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, created_at)
        VALUES ($1, $2, NOW())
        RETURNING id, username, email, created_at
        "#,
        username,
        email
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}
```

If you misspell a column name or use the wrong type, your code won't compile. This is a game-changer for database programming.

#### Example: Offline Mode for CI/CD

The compile-time checking requires a database connection. For CI/CD where you might not have a live database, SQLx provides offline mode:

```bash
# Save query metadata locally
cargo sqlx prepare

# This creates .sqlx/ directory with query metadata
# Commit this to version control

# Now compilation works without database
cargo build
```

This workflow gives you compile-time safety without requiring a database in CI.

### Example: Dynamic Queries with SQLx

Sometimes you need to build queries dynamically. SQLx supports this too:

```rust
use sqlx::{PgPool, Postgres, QueryBuilder};

async fn search_users(
    pool: &PgPool,
    username_filter: Option<&str>,
    email_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
    // QueryBuilder for dynamic queries
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT id, username, email, created_at FROM users WHERE 1=1"
    );

    if let Some(username) = username_filter {
        query_builder.push(" AND username LIKE ");
        query_builder.push_bind(format!("%{}%", username));
    }

    if let Some(email) = email_filter {
        query_builder.push(" AND email LIKE ");
        query_builder.push_bind(format!("%{}%", email));
    }

    query_builder.push(" LIMIT ");
    query_builder.push_bind(limit);

    let users = query_builder
        .build_query_as::<User>()
        .fetch_all(pool)
        .await?;

    Ok(users)
}
```

This is type-safe and SQL-injection safe (all values are bound parameters), while still allowing runtime flexibility.

### Example: Diesel: Full-Featured ORM

Diesel is Rust's most mature ORM. It provides a complete type-safe query DSL that never requires raw SQL:

```rust
// Add to Cargo.toml:
// diesel = { version = "2.1", features = ["postgres", "chrono"] }
// diesel_migrations = "2.1"

use diesel::prelude::*;
use diesel::pg::PgConnection;

//==================================================
// Define schema (typically generated by Diesel CLI)
//==================================================
table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
    }
}

//=============
// Define model
//=============
#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    username: &'a str,
    email: &'a str,
}

/// Connect to database
fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Fetch user with Diesel's query DSL
fn fetch_user_diesel(conn: &mut PgConnection, user_id: i32) -> QueryResult<User> {
    use self::users::dsl::*;

    users
        .find(user_id)
        .select(User::as_select())
        .first(conn)
}

/// Create user
fn create_user_diesel(
    conn: &mut PgConnection,
    new_username: &str,
    new_email: &str,
) -> QueryResult<User> {
    use self::users::dsl::*;

    let new_user = NewUser {
        username: new_username,
        email: new_email,
    };

    diesel::insert_into(users)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(conn)
}

/// Complex query with joins
fn find_active_users(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
    use self::users::dsl::*;

    users
        .filter(created_at.gt(chrono::Utc::now().naive_utc() - chrono::Duration::days(30)))
        .order(username.asc())
        .limit(100)
        .select(User::as_select())
        .load(conn)
}
```

Diesel's approach is entirely type-safe. The compiler ensures your queries are correct at compile time. The trade-off is a steeper learning curve and less flexibility for complex queries.

### Example: Diesel with r2d2

Combining Diesel with r2d2 gives you the best of both worlds:

```rust
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn create_diesel_pool(database_url: &str) -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .expect("Failed to create pool")
}

fn fetch_user_pooled(pool: &Pool, user_id: i32) -> QueryResult<User> {
    let mut conn = pool.get()
        .expect("Failed to get connection from pool");

    use self::users::dsl::*;

    users.find(user_id).first(&mut conn)
}
```

This pattern is common in Diesel applications—the ORM handles queries, while the pool manages connections.

## Pattern 3: Transaction Patterns

**Problem**: Multi-step operations without transactions leave money half-transferred, rows orphaned, and concurrent edits overwriting each other.

**Solution**: Wrap sequences in transactional APIs (`pool.begin()`, Diesel’s `transaction` closure) so they commit only on success, use savepoints for partial rollbacks, and add optimistic locking/version checks for concurrent writers.

**Why It Matters**: ACID semantics prevent corruption, the type system forces explicit commits or automatic rollbacks, and conflict detection avoids silent lost updates.

**Use Cases**: Payments, inventory reservations, multi-table writes, user onboarding flows, audit logging, and collaborative editors that need optimistic concurrency.

### Example: Basic Transactions with SQLx

```rust
use sqlx::{PgPool, Postgres, Transaction};

async fn transfer_money(
    pool: &PgPool,
    from_account: i32,
    to_account: i32,
    amount: f64,
) -> Result<(), sqlx::Error> {
    // Start a transaction
    let mut tx: Transaction<Postgres> = pool.begin().await?;

    // Debit from account
    sqlx::query!(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        amount,
        from_account
    )
    .execute(&mut *tx)
    .await?;

    // Credit to account
    sqlx::query!(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount,
        to_account
    )
    .execute(&mut *tx)
    .await?;

    // Commit the transaction
    // If we return early (error), the transaction auto-rollbacks on drop
    tx.commit().await?;

    Ok(())
}
```

The key insight: if any operation fails, the transaction automatically rolls back when `tx` is dropped. You can't forget to handle errors—the type system enforces it.

### Example: Nested Transactions (Savepoints)

Some databases support savepoints for nested transactions:

```rust
use sqlx::{PgPool, Postgres, Transaction};

async fn complex_operation(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Operation 1
    sqlx::query!("INSERT INTO logs (message) VALUES ('Starting')")
        .execute(&mut *tx)
        .await?;

    // Create a savepoint for nested transaction
    let mut savepoint = tx.begin().await?;

    // Try a risky operation
    match risky_operation(&mut savepoint).await {
        Ok(_) => {
            // Success - commit the savepoint
            savepoint.commit().await?;
        }
        Err(e) => {
            // Failure - rollback just the savepoint
            // The outer transaction continues
            eprintln!("Risky operation failed: {}", e);
            savepoint.rollback().await?;
        }
    }

    // Operation 2 (happens regardless of risky_operation outcome)
    sqlx::query!("INSERT INTO logs (message) VALUES ('Completed')")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

async fn risky_operation(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query!("INSERT INTO risky_table (value) VALUES (100)")
        .execute(&mut **tx)
        .await?;

    Ok(())
}
```

Savepoints allow partial rollbacks, which is useful for complex multi-step operations.

### Example: Transaction Isolation Levels

Different isolation levels provide different guarantees:

```rust
use sqlx::{PgPool, Postgres};

async fn set_isolation_level(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Set isolation level
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE")
        .execute(&mut *tx)
        .await?;

    // Perform operations with serializable isolation
    // This prevents phantom reads and ensures true serializability

    tx.commit().await?;

    Ok(())
}
```

The four standard isolation levels are:

1. **Read Uncommitted**: Can see uncommitted changes (dirty reads)
2. **Read Committed**: Only sees committed data (default in PostgreSQL)
3. **Repeatable Read**: Sees a consistent snapshot
4. **Serializable**: Full isolation, as if transactions ran serially

Higher isolation means fewer anomalies but more contention and potential for deadlocks.

### Example: Diesel Transactions

Diesel uses a different pattern for transactions:

```rust
use diesel::prelude::*;
use diesel::result::Error;

fn transfer_with_diesel(
    conn: &mut PgConnection,
    from: i32,
    to: i32,
    amount: i32,
) -> Result<(), Error> {
    conn.transaction(|conn| {
        // All operations inside this closure are in a transaction

        diesel::update(accounts::table.find(from))
            .set(accounts::balance.eq(accounts::balance - amount))
            .execute(conn)?;

        diesel::update(accounts::table.find(to))
            .set(accounts::balance.eq(accounts::balance + amount))
            .execute(conn)?;

        // Return Ok to commit, Err to rollback
        Ok(())
    })
}
```

The closure-based API is elegant: returning `Ok` commits, returning `Err` rolls back. You can't accidentally forget to commit or rollback.

### Example: Optimistic Locking

For concurrent updates, optimistic locking prevents lost updates:

```rust
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct Document {
    id: i32,
    content: String,
    version: i32,
}

async fn update_with_optimistic_lock(
    pool: &PgPool,
    doc_id: i32,
    new_content: String,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // Fetch current document
        let doc = sqlx::query_as!(
            Document,
            "SELECT id, content, version FROM documents WHERE id = $1",
            doc_id
        )
        .fetch_one(pool)
        .await?;

        // Try to update if version hasn't changed
        let result = sqlx::query!(
            r#"
            UPDATE documents
            SET content = $1, version = version + 1
            WHERE id = $2 AND version = $3
            "#,
            new_content,
            doc_id,
            doc.version
        )
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            // Success - we updated it
            return Ok(());
        }

        // Someone else updated it - retry
        println!("Conflict detected, retrying...");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
```

This pattern avoids database locks while preventing lost updates. It's ideal for long-running operations or distributed systems.

## Pattern 4: Migration Strategies

**Problem**: Ad-hoc SQL changes leave environments out of sync, makes onboarding painful, and gives no safe rollback when a deployment goes wrong.

**Solution**: Treat schema changes as code: timestamped up/down migrations via SQLx or Diesel, committed to Git, run automatically (or via CLI) in every environment, and split risky changes into multi-phase, zero-downtime steps.

**Why It Matters**: Reproducible schemas, automated CI checks, audit trails, straightforward rollbacks, and collaborative workflows all depend on consistent, versioned migrations.

**Use Cases**: Any production database, CI pipelines, blue/green deploys, multi-developer teams coordinating schema changes, and new hires needing one command to match prod.

### Example: SQLx Migrations

SQLx includes a built-in migration system:

```bash
# Create migrations directory
mkdir -p migrations

# Add a migration
sqlx migrate add create_users_table
```

This creates a file like `migrations/20240101000000_create_users_table.sql`:

```sql
-- migrations/20240101000000_create_users_table.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
```

Run migrations programmatically:

```rust
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://user:pass@localhost/mydb")
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    println!("Migrations completed successfully");

    Ok(())
}
```

Or use the CLI:

```bash
# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Example: Diesel Migrations

Diesel has a more sophisticated migration system:

```bash
# Install Diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# Setup Diesel
diesel setup

# Create migration
diesel migration generate create_users_table
```

This creates up and down migration files:

```sql
-- migrations/2024-01-01-000000_create_users_table/up.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

```sql
-- migrations/2024-01-01-000000_create_users_table/down.sql
DROP TABLE users;
```

Run migrations:

```bash
# Apply all pending migrations
diesel migration run

# Revert last migration
diesel migration revert

# Redo last migration (useful for development)
diesel migration redo
```

Diesel automatically updates `src/schema.rs` with the table definitions, keeping your Rust code in sync with the database.

### Example: Migration Best Practices

Follow these principles for successful migrations:

#### 1. Make Migrations Reversible

Always provide down migrations:

```sql
-- up.sql
ALTER TABLE users ADD COLUMN phone VARCHAR(20);

-- down.sql
ALTER TABLE users DROP COLUMN phone;
```

This allows you to roll back changes if something goes wrong.

#### 2. Never Modify Existing Migrations

Once a migration runs in production, it's immutable. Create a new migration to fix issues:

```sql
-- WRONG: Modifying old migration
-- migrations/001_create_users.sql (don't edit this!)

-- RIGHT: Create new migration
-- migrations/002_fix_users_table.sql
ALTER TABLE users ALTER COLUMN email TYPE VARCHAR(320);
```

#### 3. Test Migrations Thoroughly

Test both up and down migrations:

```bash
# Test up migration
diesel migration run

# Test down migration
diesel migration revert

# Test redo (down then up)
diesel migration redo
```

#### 4. Keep Migrations Small

Small migrations are easier to review and debug:

```sql
-- GOOD: One focused change
-- migrations/003_add_user_index.sql
CREATE INDEX idx_users_created_at ON users(created_at);

-- AVOID: Multiple unrelated changes
-- migrations/004_big_changes.sql
CREATE INDEX idx_users_created_at ON users(created_at);
ALTER TABLE posts ADD COLUMN featured BOOLEAN;
CREATE TABLE tags (...);
```

### Example: Data Migrations

Sometimes you need to migrate data, not just schema:

```sql
-- migrations/005_normalize_emails/up.sql

-- Add new column
ALTER TABLE users ADD COLUMN email_normalized VARCHAR(255);

-- Populate with normalized emails
UPDATE users SET email_normalized = LOWER(email);

-- Make it NOT NULL
ALTER TABLE users ALTER COLUMN email_normalized SET NOT NULL;

-- Add unique constraint
ALTER TABLE users ADD CONSTRAINT users_email_normalized_unique UNIQUE (email_normalized);

-- Create index
CREATE INDEX idx_users_email_normalized ON users(email_normalized);
```

For complex data migrations, use application code:

```rust
use sqlx::PgPool;

async fn run_data_migration(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // Fetch users in batches
    let mut offset = 0;
    let batch_size = 1000;

    loop {
        let users: Vec<(i32, String)> = sqlx::query_as(
            "SELECT id, email FROM users LIMIT $1 OFFSET $2"
        )
        .bind(batch_size)
        .bind(offset)
        .fetch_all(&mut *tx)
        .await?;

        if users.is_empty() {
            break;
        }

        // Process each user
        for (id, email) in users {
            let normalized = email.to_lowercase();
            sqlx::query!(
                "UPDATE users SET email_normalized = $1 WHERE id = $2",
                normalized,
                id
            )
            .execute(&mut *tx)
            .await?;
        }

        offset += batch_size;
    }

    tx.commit().await?;

    Ok(())
}
```

Processing in batches prevents memory exhaustion on large datasets.

### Example: Zero-Downtime Migrations

For production systems that can't have downtime, follow this pattern:

1. **Add new column (nullable)**: Deploy this first, application ignores it
2. **Backfill data**: Populate the new column
3. **Update application**: Start using the new column
4. **Make column NOT NULL**: After all data is backfilled
5. **Remove old column**: After confirming new column works

Example timeline:

```sql
-- Migration 1: Add nullable column
ALTER TABLE users ADD COLUMN new_email VARCHAR(320);

-- Migration 2: Backfill data (run with application handling both columns)
UPDATE users SET new_email = old_email WHERE new_email IS NULL;

-- Migration 3: Make NOT NULL (after deploy using new column)
ALTER TABLE users ALTER COLUMN new_email SET NOT NULL;

-- Migration 4: Drop old column (after confirming everything works)
ALTER TABLE users DROP COLUMN old_email;
```

Each step can be deployed independently without downtime.

## Pattern 5: ORM vs Raw SQL

**Problem**: ORMs make CRUD safe but struggle with complex SQL features, while raw SQL gives full power at the expense of compile-time checks and refactor safety.

**Solution**: Mix and match—use Diesel (or similar ORM) for routine operations where type safety shines, and fall back to SQLx/raw SQL for analytics, window functions, CTEs, or database-specific features, still binding parameters to avoid injection.

**Why It Matters**: You get the best of both worlds: painless CRUD refactors plus full SQL expressiveness when needed, without forcing the entire codebase into one paradigm.

**Use Cases**: CRUD-heavy modules, admin panels, and migrations via ORM; reporting, full-text search, JSONB operators, and tuned analytical queries via SQLx or handcrafted SQL.

### Example: Hybrid Approach

The best solution often combines both:

```rust
use sqlx::PgPool;
use diesel::prelude::*;

//=====================
// Simple CRUD: Use ORM
//=====================
fn create_user_orm(conn: &mut PgConnection, name: &str) -> QueryResult<User> {
    diesel::insert_into(users::table)
        .values(users::username.eq(name))
        .get_result(conn)
}

//===============================
// Complex analytics: Use raw SQL
//===============================
async fn get_sales_report(pool: &PgPool) -> Result<Vec<SalesData>, sqlx::Error> {
    sqlx::query_as!(SalesData, r#"
        SELECT
            date_trunc('day', created_at) as day,
            COUNT(*) as total_orders,
            SUM(amount) as total_revenue,
            AVG(amount) as avg_order_value
        FROM orders
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY day
        ORDER BY day DESC
    "#)
    .fetch_all(pool)
    .await
}

//========================================
// Database-specific features: Use raw SQL
//========================================
async fn full_text_search(pool: &PgPool, query: &str) -> Result<Vec<Document>, sqlx::Error> {
    sqlx::query_as!(Document, r#"
        SELECT id, title, content
        FROM documents
        WHERE to_tsvector('english', title || ' ' || content) @@ plainto_tsquery('english', $1)
        ORDER BY ts_rank(to_tsvector('english', title || ' ' || content), plainto_tsquery('english', $1)) DESC
        LIMIT 20
    "#, query)
    .fetch_all(pool)
    .await
}
```

### When to Use Each

**Use an ORM (Diesel) when:**
- Building standard CRUD operations
- Type safety is critical
- You want database portability
- Your team is learning SQL
- Migrations are frequent

**Use raw SQL (SQLx) when:**
- Queries are complex (joins, subqueries, CTEs)
- Performance is critical
- Using database-specific features
- Queries are dynamic
- Debugging is important

**Use both when:**
- Building a real application (most cases)
- Different parts have different needs
- You want flexibility

### Example: Complete Application Example

Here's a realistic application structure combining patterns:

```rust
use sqlx::PgPool;
use deadpool_postgres::Pool;

//==========================
// Application configuration
//==========================
pub struct AppState {
    pool: PgPool,
}

//===========================
// User repository using SQLx
//===========================
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: &str, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, username, email, created_at
            "#,
            username,
            email
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email, created_at FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_active(&self, days: i32) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, created_at
            FROM users
            WHERE created_at > NOW() - $1 * INTERVAL '1 day'
            ORDER BY created_at DESC
            "#,
            days
        )
        .fetch_all(&self.pool)
        .await
    }
}

//============================================
// Analytics using raw SQL for complex queries
//============================================
pub struct Analytics {
    pool: PgPool,
}

impl Analytics {
    pub async fn user_growth_report(&self) -> Result<Vec<GrowthMetric>, sqlx::Error> {
        sqlx::query_as!(
            GrowthMetric,
            r#"
            WITH daily_signups AS (
                SELECT
                    date_trunc('day', created_at) as signup_date,
                    COUNT(*) as new_users
                FROM users
                GROUP BY signup_date
            )
            SELECT
                signup_date as date,
                new_users,
                SUM(new_users) OVER (ORDER BY signup_date) as cumulative_users
            FROM daily_signups
            ORDER BY signup_date DESC
            LIMIT 30
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(sqlx::FromRow)]
pub struct GrowthMetric {
    pub date: Option<chrono::NaiveDateTime>,
    pub new_users: Option<i64>,
    pub cumulative_users: Option<i64>,
}
```

This structure separates concerns: simple operations use type-safe queries, while analytics uses raw SQL for complex aggregations.

### Summary

This chapter covered database patterns for production Rust applications:

1. **Connection Pooling**: r2d2 (sync), deadpool (async); reuse connections eliminating 50-200ms overhead
2. **Query Builders**: SQLx compile-time verification, Diesel type-safe DSL, prevent SQL errors at compile-time
3. **Transaction Patterns**: ACID guarantees via type system, savepoints, optimistic locking, isolation levels
4. **Migration Strategies**: Version-controlled schema as code, up/down migrations, zero-downtime deployments
5. **ORM vs Raw SQL**: Hybrid approach—Diesel for CRUD, SQLx for complex queries, maximize both safety and power

**Key Takeaways**:
- Connection pooling mandatory for production: 50-200x faster than per-request connections
- Compile-time SQL verification eliminates runtime errors: typos become compile errors
- Type system enforces transaction safety: auto-rollback on drop, explicit commit required
- Migrations as code = reproducible schemas, rollback capability, team coordination
- Hybrid ORM/SQL approach: type safety for simple queries, SQL power for complex analytics

**Performance Guidelines**:
- Pool sizing: small apps 5-10, medium 10-20, large 20-50 connections (rarely need >50)
- Async pooling (deadpool) for high-concurrency: avoids thread blocking, better multiplexing
- Transaction isolation: READ COMMITTED (default) vs REPEATABLE READ vs SERIALIZABLE (trade-off: consistency vs concurrency)
- Prepared statements: reused via pooled connections, query planning cached
- Batch processing: process data migrations in batches (1000-10000 rows) prevents memory exhaustion

**Configuration Examples**:
```rust
// Connection pool for web server handling 100 req/s
Pool::builder()
    .max_size(20)               // 20 connections handle 100+ req/s
    .min_idle(Some(5))          // Keep 5 warm for quick response
    .connection_timeout(Duration::from_secs(5))
    .test_on_check_out(true)    // Validate before use

// Transaction with optimistic locking
UPDATE documents
SET content = $1, version = version + 1
WHERE id = $2 AND version = $3  // Only update if version unchanged

// Zero-downtime migration (multi-phase)
// Phase 1: Add nullable column
ALTER TABLE users ADD COLUMN new_email VARCHAR(320);
// Phase 2: Backfill data (deploy app handling both)
UPDATE users SET new_email = old_email WHERE new_email IS NULL;
// Phase 3: Make NOT NULL
ALTER TABLE users ALTER COLUMN new_email SET NOT NULL;
// Phase 4: Drop old column
ALTER TABLE users DROP COLUMN old_email;
```

**Common Patterns**:
```rust
// Connection pooling with Arc for sharing
let pool = create_pool().await?;
let pool_clone = pool.clone();  // Cheap Arc clone

// Compile-time verified query
let user = sqlx::query_as!(User,
    "SELECT id, username FROM users WHERE id = $1",
    user_id
).fetch_one(&pool).await?;

// Transaction with auto-rollback
let mut tx = pool.begin().await?;
query!(...).execute(&mut *tx).await?;
tx.commit().await?;  // Explicit commit required

// Diesel CRUD
diesel::insert_into(users::table)
    .values(&new_user)
    .get_result(&mut conn)?;

// Dynamic SQLx query
let mut builder = QueryBuilder::new("SELECT * FROM users WHERE 1=1");
if let Some(name) = filter {
    builder.push(" AND username = ").push_bind(name);
}
```

**Best Practices**:
- Monitor pool health: alert if available = 0 (exhausted) or > 75% (oversized)
- Use transactions for multi-step operations: prevents partial failures
- Never modify applied migrations: create new migration to fix
- Test migrations: `run` then `revert` then `redo` before production
- Keep migrations small: one logical change per file
- Use savepoints for partial rollback: outer transaction continues if inner fails
- Implement connection validation: test_on_check_out detects dead connections
- Configure timeouts: connection_timeout prevents indefinite waiting
- Batch data migrations: process 1000-10000 rows at a time
- Use indexes: query performance depends on proper indexing (outside scope of patterns, but critical)
