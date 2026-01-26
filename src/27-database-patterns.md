# Database Patterns

This chapter explores database patterns in Rust: connection pooling for resource management, query builders for type safety, transactions for data integrity, migrations for schema evolution, and choosing between ORM and raw SQL based on specific requirements.

## Pattern 1: Connection Pooling

**Problem**: Opening a fresh database connection per request costs handshakes, auth, and session setup, quickly exhausting `max_connections` and adding 100ms+ latency to every query.

**Solution**: Keep a pool of ready connections (r2d2, deadpool). Borrow them with `pool.get()`, configure sensible `max_size`, `min_idle`, and timeouts, and let the smart pointer return them on drop.

**Why It Matters**: Reusing 10–20 pooled connections serves hundreds of requests with <1 ms checkout time, protects the database from connection storms, and preserves prepared statements/buffers.

**Use Cases**: Web servers, background workers, GraphQL resolvers, CLI tools hitting DBs repeatedly, serverless globals, and integration test harnesses needing many short-lived queries.

### Example: r2d2: The Classic Connection Pool
r2d2 is Rust's battle-tested synchronous connection pool. `Pool::builder()` configures `max_size`, `min_idle`, and timeouts; `pool.get()` returns a smart pointer that auto-returns connections on drop.
The pool is `Clone` (Arc-wrapped) for thread sharing, reducing connection overhead from ~100ms to <1ms checkout time.

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

fn create_pool(database_url: &str) -> Result<PostgresPool, Box<dyn Error>> {
    let manager = PostgresConnectionManager::new(database_url.parse()?, NoTls);

    let pool = Pool::builder()
        .max_size(15)
        .min_idle(Some(5))
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)?;
    Ok(pool)
}

async fn fetch_user(pool: &PostgresPool, user_id: i32) -> Result<String, Box<dyn Error>> {
    let mut conn = pool.get()?;  // Blocks if exhausted
    let row = conn.query_one("SELECT username FROM users WHERE id = $1", &[&user_id])?;
    let username: String = row.get(0);
    Ok(username)  // Connection returns to pool on drop
}

fn main() -> Result<(), Box<dyn Error>> {
    let pool = create_pool("postgresql://user:pass@localhost/mydb")?;
    let pool_clone = pool.clone();  // Cheap clone (Arc internally)

    let handle = std::thread::spawn(move || fetch_user(&pool_clone, 42));
    let username = fetch_user(&pool, 42)?;  // Both threads use pool concurrently
    println!("User: {}", username);
    handle.join().unwrap()?;
    Ok(())
}
let pool = create_pool("postgres://...")?; 
let user = fetch_user(&pool, 1)?;
```

The beauty of this pattern is in the `PooledConnection` type. It's a smart pointer that automatically returns the connection to the pool when dropped. You can't forget to return it—Rust's ownership system enforces correct cleanup.

### Example: Tuning Pool Configuration
`max_size` caps connections (too low = queuing, too high = DB overload), `min_idle` pre-warms for bursts, `test_on_check_out` validates connections (~1ms overhead).
`idle_timeout` and `max_lifetime` handle stale connections and memory leaks—start conservative and tune based on load patterns.

```rust
use r2d2::Pool;
use std::time::Duration;

fn configure_pool_detailed(manager: PostgresConnectionManager<NoTls>) -> PostgresPool {
    Pool::builder()
        .max_size(20)                    // More = more concurrency, more DB load
        .min_idle(Some(5))               // Pre-warm for bursts
        .connection_timeout(Duration::from_secs(10))  // Wait before timeout
        .test_on_check_out(true)         // Validate connections (~1ms overhead)
        .idle_timeout(Some(Duration::from_secs(300)))  // Close stale connections
        .max_lifetime(Some(Duration::from_secs(1800))) // Force recreation
        .build(manager)
        .unwrap()
}
let pool = configure_pool_detailed(manager); // max 20, idle 5, 10s timeout
```

For a typical web application:
- **Small app** (< 100 concurrent users): max_size = 5-10
- **Medium app** (100-1000 users): max_size = 10-20
- **Large app** (1000+ users): max_size = 20-50

Going higher than 50 often indicates other bottlenecks.

### Example: deadpool: Async-First Connection Pooling
deadpool is purpose-built for async runtimes—`pool.get().await` yields to other tasks instead of blocking threads like r2d2.
Three timeouts (`wait`, `create`, `recycle`) handle different failure modes; cheap cloning (Arc) enables sharing across spawned tasks.

```rust
// Add to Cargo.toml:
// deadpool = { version = "0.10", features = ["managed"] }
// deadpool-postgres = "0.12"
// tokio-postgres = "0.7"
// tokio = { version = "1", features = ["full"] }

use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;
use std::error::Error;

fn create_async_pool() -> Result<Pool, Box<dyn Error>> {  // Async-native pool
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

async fn fetch_user_async(pool: &Pool, user_id: i32) -> Result<String, Box<dyn Error>> {
    let client = pool.get().await?;  // Awaits instead of blocking
    let row = client.query_one("SELECT username FROM users WHERE id = $1", &[&user_id]).await?;
    let username: String = row.get(0);
    Ok(username)  // Returns to pool on drop
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pool = create_async_pool()?;

    let tasks = (1..=100).map(|id| {  // 100 concurrent queries
        let pool = pool.clone();
        tokio::spawn(async move { fetch_user_async(&pool, id).await })
    });

    let results = futures::future::join_all(tasks).await;
    println!("Completed {} queries", results.len());
    Ok(())
}
fetch_user_async(&pool, 42).await?; // Non-blocking async pool access
```

The key difference: deadpool's `get()` returns a `Future` that yields to other tasks while waiting for a connection. With r2d2, a thread would block. This makes deadpool much more efficient for high-concurrency async applications.

### Example: Health Checks and Monitoring
`pool.status()` provides real-time metrics: `available == 0` means exhausted (requests will block), `available > 75%` means oversized.
Graceful shutdown via `pool.close()` stops new checkouts and waits for active connections—export metrics to Prometheus/Grafana for alerting.

```rust
use deadpool_postgres::Pool;

async fn monitor_pool_health(pool: &Pool) {
    let status = pool.status();
    println!("Available: {}, Size: {}, Max: {}", status.available, status.size, status.max_size);

    if status.available == 0 { eprintln!("WARNING: Pool exhausted!"); }
    if status.available > status.max_size * 3 / 4 { eprintln!("INFO: Pool oversized"); }
}

async fn shutdown_pool(pool: Pool) {  // Graceful shutdown
    pool.close();
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;  // Wait for active conns
}
monitor_pool_health(&pool).await; // Check available/size/max_size
```

Monitoring helps you tune pool size and detect issues before they impact users.

## Pattern 2: Query Builders

**Problem**: Raw SQL strings hide typos, wrong parameter counts, and type mismatches until runtime—and string concatenation invites SQL injection.

**Solution**: Use compile-time checked builders: SQLx `query!` verifies syntax/columns/types, Diesel’s schema DSL provides type-safe composable queries, and SQLx’s `QueryBuilder` binds parameters for dynamic clauses.

**Why It Matters**: Mistakes surface as compiler errors instead of production crashes, refactors update in one place, and bound parameters shut the door on injection.

**Use Cases**: Everyday CRUD endpoints, GraphQL resolvers, dashboards with dynamic filters, reporting queries, and any service that needs confidence its SQL matches the schema.

### Example: SQLx: The Compile-Time Checked Query Builder
SQLx's `query!` and `query_as!` macros verify SQL syntax, table/column existence, and type compatibility at compile time—misspell a column and it won't compile.
`#[derive(FromRow)]` maps results to structs; `fetch_one`/`fetch_optional`/`fetch_all` return typed results. Trade-off: requires DB access during compilation.

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
    PgPool::connect(database_url).await  // Built-in connection pooling
}

async fn fetch_user(pool: &PgPool, user_id: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(  // Compile-time verified: syntax, tables, columns, types
        User,
        "SELECT id, username, email, created_at FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

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
let user = fetch_user(&pool, 1).await?; // Compile-time verified SQL
```

If you misspell a column name or use the wrong type, your code won't compile. This is a game-changer for database programming.

#### Example: Offline Mode for CI/CD
`cargo sqlx prepare` extracts query metadata to `.sqlx/` directory; with `SQLX_OFFLINE=true`, SQLx uses cached metadata instead of connecting.
Commit `.sqlx/` to version control for CI builds without database access—re-run `prepare` after schema or query changes.

```bash
# Save query metadata locally
cargo sqlx prepare

# This creates .sqlx/ directory with query metadata
# Commit this to version control

# Now compilation works without database
cargo build
```


### Example: Dynamic Queries with SQLx
`QueryBuilder<Postgres>` constructs SQL safely at runtime: `push()` adds raw SQL, `push_bind()` adds parameterized values (injection-safe).
The `WHERE 1=1` trick simplifies conditional clauses; `build_query_as::<T>()` compiles the result. Essential for search endpoints with optional filters.

```rust
use sqlx::{PgPool, Postgres, QueryBuilder};

async fn search_users(  // Dynamic query building
    pool: &PgPool,
    username_filter: Option<&str>,
    email_filter: Option<&str>,
    limit: i64,
) -> Result<Vec<User>, sqlx::Error> {
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
search_users(&pool, Some("alice"), None, 10).await?; // Dynamic WHERE clauses
```

This is type-safe and SQL-injection safe (all values are bound parameters), while still allowing runtime flexibility.

### Example: Diesel: Full-Featured ORM
Diesel provides a Rust DSL that compiles to SQL: `table!` creates type-safe columns, `#[derive(Queryable)]` maps rows to structs, queries read like Rust code.
The compiler catches type mismatches at compile time. Trade-off: steeper learning curve, DSL can't express CTEs or window functions.

```rust
// Add to Cargo.toml:
// diesel = { version = "2.1", features = ["postgres", "chrono"] }
// diesel_migrations = "2.1"

use diesel::prelude::*;
use diesel::pg::PgConnection;

table! {  // Schema (typically generated by Diesel CLI)
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
    }
}

#[derive(Queryable, Selectable, Debug)]  // Model
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

fn establish_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url).unwrap()
}

fn fetch_user_diesel(conn: &mut PgConnection, user_id: i32) -> QueryResult<User> {
    use self::users::dsl::*;
    users.find(user_id).select(User::as_select()).first(conn)
}

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

    diesel::insert_into(users).values(&new_user).returning(User::as_returning()).get_result(conn)
}

fn find_active_users(conn: &mut PgConnection) -> QueryResult<Vec<User>> {  // Complex query
    use self::users::dsl::*;
    users.filter(created_at.gt(chrono::Utc::now().naive_utc() - chrono::Duration::days(30)))
        .order(username.asc()).limit(100).select(User::as_select()).load(conn)
}
let user = fetch_user_diesel(&mut conn, 1)?; // Type-safe Diesel DSL
```

Diesel's approach is entirely type-safe. The compiler ensures your queries are correct at compile time. The trade-off is a steeper learning curve and less flexibility for complex queries.

### Example: Diesel with r2d2
Diesel's `diesel::r2d2` module provides seamless integration—`ConnectionManager<PgConnection>` adapts to r2d2's pool interface.
The pooled connection dereferences to `&mut PgConnection`, so existing queries work unchanged. For async, use `diesel-async` with `deadpool-diesel`.

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
let pool = create_diesel_pool("postgres://..."); fetch_user_pooled(&pool, 1)?;
```

This pattern is common in Diesel applications—the ORM handles queries, while the pool manages connections.

## Pattern 3: Transaction Patterns

**Problem**: Multi-step operations without transactions leave money half-transferred, rows orphaned, and concurrent edits overwriting each other.

**Solution**: Wrap sequences in transactional APIs (`pool.begin()`, Diesel’s `transaction` closure) so they commit only on success, use savepoints for partial rollbacks, and add optimistic locking/version checks for concurrent writers.

**Why It Matters**: ACID semantics prevent corruption, the type system forces explicit commits or automatic rollbacks, and conflict detection avoids silent lost updates.

**Use Cases**: Payments, inventory reservations, multi-table writes, user onboarding flows, audit logging, and collaborative editors that need optimistic concurrency.

### Example: Basic Transactions with SQLx
This example demonstrates the classic money transfer problem: debit one account and credit another atomically. `pool.begin().await` starts a transaction and returns a `Transaction<Postgres>` handle. All queries using `&mut *tx` execute within that transaction. The critical safety feature: if you return early (via `?` or explicit return) without calling `tx.commit()`, the transaction automatically rolls back when `tx` is dropped. This RAII pattern makes it impossible to accidentally leave a transaction half-committed. The type system enforces correctness—you must explicitly commit or let the destructor rollback.

```rust
use sqlx::{PgPool, Postgres, Transaction};

async fn transfer_money(
    pool: &PgPool, from_account: i32, to_account: i32, amount: f64,
) -> Result<(), sqlx::Error> {
    let mut tx: Transaction<Postgres> = pool.begin().await?;

    sqlx::query!("UPDATE accounts SET balance = balance - $1 WHERE id = $2", amount, from_account)
        .execute(&mut *tx).await?;

    sqlx::query!("UPDATE accounts SET balance = balance + $1 WHERE id = $2", amount, to_account)
        .execute(&mut *tx).await?;

    tx.commit().await?;  // Auto-rollback on early return/error
    Ok(())
}
transfer_money(&pool, 1, 2, 100.0).await?; // Atomic transfer with rollback on error
```

The key insight: if any operation fails, the transaction automatically rolls back when `tx` is dropped. You can't forget to handle errors—the type system enforces it.

### Example: Nested Transactions (Savepoints)
Savepoints create checkpoints within a transaction that you can roll back to without aborting the entire transaction. Calling `tx.begin().await` inside an existing transaction creates a savepoint (not a new transaction). If the inner operation fails, you `rollback()` just the savepoint—the outer transaction continues and can still commit. This pattern is valuable for "best-effort" operations: log the failure, skip the bad record, continue processing. PostgreSQL, MySQL, and SQLite all support savepoints. The nested `Transaction` type prevents accidentally committing the outer transaction from within the inner scope.

Some databases support savepoints for nested transactions:

```rust
use sqlx::{Acquire, PgPool, Postgres, Transaction};

async fn complex_operation(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!("INSERT INTO logs (message) VALUES ('Starting')").execute(&mut *tx).await?;

    let mut savepoint = tx.begin().await?;  // Savepoint for partial rollback

    match risky_operation(&mut savepoint).await {
        Ok(_) => savepoint.commit().await?,       // Commit savepoint
        Err(e) => { eprintln!("Failed: {}", e); savepoint.rollback().await?; }  // Rollback only savepoint
    }

    sqlx::query!("INSERT INTO logs (message) VALUES ('Completed')").execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn risky_operation(tx: &mut Transaction<'_, Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query!("INSERT INTO risky_table (value) VALUES (100)")
        .execute(&mut **tx)
        .await?;

    Ok(())
}
complex_operation(&pool).await?; // Savepoint allows partial rollback
```

Savepoints allow partial rollbacks, which is useful for complex multi-step operations.

### Example: Transaction Isolation Levels
Isolation levels control what concurrent transactions can see of each other's uncommitted changes. `READ COMMITTED` (PostgreSQL default) sees only committed data but may see different results if re-reading the same row. `REPEATABLE READ` sees a consistent snapshot—re-reads return the same data—but phantom rows can appear in range queries. `SERIALIZABLE` provides full isolation as if transactions ran one-at-a-time, but requires retry logic for serialization failures. Higher isolation means more consistency guarantees but also more contention, potential deadlocks, and performance overhead. Choose based on your consistency requirements: most web apps use READ COMMITTED; financial transactions may need SERIALIZABLE.

Different isolation levels provide different guarantees:

```rust
use sqlx::{PgPool, Postgres};

async fn set_isolation_level(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL SERIALIZABLE").execute(&mut *tx).await?;
    // Operations here have full isolation (prevents phantom reads)
    tx.commit().await?;
    Ok(())
}
set_isolation_level(&pool).await?; // SERIALIZABLE prevents phantom reads
```

The four standard isolation levels are:

1. **Read Uncommitted**: Can see uncommitted changes (dirty reads)
2. **Read Committed**: Only sees committed data (default in PostgreSQL)
3. **Repeatable Read**: Sees a consistent snapshot
4. **Serializable**: Full isolation, as if transactions ran serially

Higher isolation means fewer anomalies but more contention and potential for deadlocks.

### Example: Diesel Transactions
Diesel's `connection.transaction(|conn| { ... })` closure-based API is elegant and foolproof. The closure receives a mutable connection reference scoped to the transaction. Return `Ok(value)` to commit and get `value` back; return `Err(e)` to rollback and propagate the error. The `?` operator naturally triggers rollback on any failed query. You cannot forget to commit or rollback—the closure's return type determines the outcome. This pattern also enables nested transactions via `conn.transaction()` inside another transaction closure (creates savepoints). The trade-off vs SQLx: less explicit control, but impossible to misuse.

Diesel uses a different pattern for transactions:

```rust
use diesel::prelude::*;
use diesel::result::Error;

fn transfer_with_diesel(conn: &mut PgConnection, from: i32, to: i32, amount: i32) -> Result<(), Error> {
    conn.transaction(|conn| {  // Ok = commit, Err = rollback
        diesel::update(accounts::table.find(from))
            .set(accounts::balance.eq(accounts::balance - amount)).execute(conn)?;
        diesel::update(accounts::table.find(to))
            .set(accounts::balance.eq(accounts::balance + amount)).execute(conn)?;
        Ok(())
    })
}
transfer_with_diesel(&mut conn, 1, 2, 100)?; // Closure-based transaction
```

The closure-based API is elegant: returning `Ok` commits, returning `Err` rolls back. You can't accidentally forget to commit or rollback.

### Example: Optimistic Locking
Optimistic locking assumes conflicts are rare and detects them at write time rather than preventing them with locks. Each row has a `version` column. Before updating, read the current version. The UPDATE includes `WHERE version = $old_version` and increments the version. If `rows_affected() == 0`, another transaction updated the row first—your version was stale. The retry loop re-reads and re-attempts. This pattern avoids database locks entirely, making it ideal for: long-running user edits (document editing), distributed systems where locks are expensive, and read-heavy workloads where conflicts are genuinely rare. The trade-off: high-contention scenarios cause retry storms.

For concurrent updates, optimistic locking prevents lost updates:

```rust
use sqlx::PgPool;

#[derive(sqlx::FromRow)]
struct Document {
    id: i32,
    content: String,
    version: i32,
}

async fn update_with_optimistic_lock(pool: &PgPool, doc_id: i32, new_content: String) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let doc = sqlx::query_as!(Document, "SELECT id, content, version FROM documents WHERE id = $1", doc_id)
            .fetch_one(pool).await?;

        let result = sqlx::query!(
            "UPDATE documents SET content = $1, version = version + 1 WHERE id = $2 AND version = $3",
            new_content, doc_id, doc.version
        ).execute(pool).await?;

        if result.rows_affected() > 0 { return Ok(()); }  // Success

        println!("Conflict, retrying...");  // Version changed - retry
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}
update_with_optimistic_lock(&pool, 1, "new content".into()).await?;
```

This pattern avoids database locks while preventing lost updates. It's ideal for long-running operations or distributed systems.

## Pattern 4: Migration Strategies

**Problem**: Ad-hoc SQL changes leave environments out of sync, makes onboarding painful, and gives no safe rollback when a deployment goes wrong.

**Solution**: Treat schema changes as code: timestamped up/down migrations via SQLx or Diesel, committed to Git, run automatically (or via CLI) in every environment, and split risky changes into multi-phase, zero-downtime steps.

**Why It Matters**: Reproducible schemas, automated CI checks, audit trails, straightforward rollbacks, and collaborative workflows all depend on consistent, versioned migrations.

**Use Cases**: Any production database, CI pipelines, blue/green deploys, multi-developer teams coordinating schema changes, and new hires needing one command to match prod.

### Example: SQLx Migrations
SQLx migrations are simple SQL files in a `migrations/` directory, named with timestamps for ordering (e.g., `20240101000000_create_users.sql`). The `sqlx migrate add` command creates timestamped files. `sqlx::migrate!("./migrations").run(&pool).await` runs all pending migrations, tracking which have run in a `_sqlx_migrations` table. SQLx migrations are "up-only" by default—write reversible SQL manually if needed. The macro embeds migration SQL at compile time, so the binary is self-contained. For teams, commit migrations to git; SQLx's timestamp ordering handles concurrent branches better than sequential numbering.

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
    let pool = PgPoolOptions::new().max_connections(5)
        .connect("postgresql://user:pass@localhost/mydb").await?;

    sqlx::migrate!("./migrations").run(&pool).await?;  // Run pending migrations
    Ok(())
}
sqlx::migrate!("./migrations").run(&pool).await?; // Run pending migrations
```

Or use the CLI:

```bash
# Run all pending migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Example: Diesel Migrations
Diesel's migration system separates `up.sql` (apply) and `down.sql` (revert) into paired files, enforcing reversibility. The `diesel migration generate name` command creates both files. `diesel migration run` applies pending migrations; `diesel migration revert` undoes the last one; `diesel migration redo` reverts then re-applies (useful for testing). Critically, Diesel automatically regenerates `src/schema.rs` after migrations, keeping your Rust table definitions synchronized with the database. This eliminates the "schema.rs doesn't match database" class of errors. The CLI also validates that down migrations actually undo up migrations.

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
These practices prevent migration disasters in production. Reversible migrations (`up.sql` + `down.sql`) enable rollback when deployments fail. Never modify applied migrations—other environments have already run the old version; create a new migration to fix issues. Test migrations with `run`/`revert`/`redo` cycles before production. Keep migrations small and focused: one logical change per file makes debugging easier and allows partial rollbacks. Large "kitchen sink" migrations are hard to review, hard to debug, and impossible to partially revert.

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
Data migrations transform existing data, not just schema structure. The SQL example shows a multi-step pattern: add nullable column → populate with transformed data → add constraints → create indexes. For complex transformations (parsing, external API calls, business logic), use application code instead of SQL. The Rust example processes in batches (1000 rows) to prevent: memory exhaustion from loading millions of rows, transaction log bloat, and lock contention. Wrap the entire migration in a transaction for atomicity, or use per-batch commits with idempotent logic for resumability after failures.

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
run_data_migration(&pool).await?; // Batch process 1000 rows at a time
```

Processing in batches prevents memory exhaustion on large datasets.

### Example: Zero-Downtime Migrations
Zero-downtime migrations split risky changes across multiple deploys to avoid breaking running applications. The key insight: at any moment, two versions of your application may be running (during rolling deploys). Phase 1 adds the nullable column—old code ignores it, new code can start writing to it. Phase 2 backfills existing data while both column versions exist. Phase 3 makes the column NOT NULL after all data is migrated. Phase 4 removes the old column after confirming the new one works. Each phase is independently deployable and rollback-able. This pattern applies to any breaking change: renaming columns, changing types, or splitting tables.

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
Real applications benefit from using both ORM and raw SQL based on each query's needs. Simple CRUD operations (`create_user_orm`) get Diesel's full type safety—column renames update in one place, type mismatches are compile errors. Complex analytics (`get_sales_report`) use SQLx for `date_trunc`, `GROUP BY`, and aggregations that would be awkward in ORM syntax. Database-specific features (`full_text_search`) require PostgreSQL's `to_tsvector` and `@@` operators that ORMs can't express. The boundary is clear: if Diesel expresses it naturally, use Diesel; if you're fighting the DSL or need database-specific SQL, use SQLx. Both are in the same codebase, same pool, same transactions when needed.

The best solution often combines both:

```rust
use sqlx::PgPool;
use diesel::prelude::*;

fn create_user_orm(conn: &mut PgConnection, name: &str) -> QueryResult<User> {  // ORM for CRUD
    diesel::insert_into(users::table).values(users::username.eq(name)).get_result(conn)
}

async fn get_sales_report(pool: &PgPool) -> Result<Vec<SalesData>, sqlx::Error> {  // Raw SQL for analytics
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

async fn full_text_search(pool: &PgPool, query: &str) -> Result<Vec<Document>, sqlx::Error> {  // DB-specific features
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
ORM for CRUD, raw SQL for analytics/full-text search
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
This example shows production-ready architecture combining all patterns. The `AppState` struct holds the connection pool, passed to web framework handlers. `UserRepository` encapsulates all user-related queries using the Repository pattern—the rest of the application calls `repo.create()` without knowing SQL. The `Analytics` struct separates complex reporting queries that use CTEs and window functions (`SUM() OVER()`). This separation means simple CRUD gets maximum type safety while analytics gets full SQL expressiveness. The `#[derive(sqlx::FromRow)]` structs define the contract between database and application. This architecture scales: add more repositories for other entities, add more analytics modules for different reports.

Here's a realistic application structure combining patterns:

```rust
use sqlx::PgPool;
use deadpool_postgres::Pool;

pub struct AppState { pool: PgPool }

pub struct UserRepository {  // Repository pattern
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

pub struct Analytics {  // Raw SQL for complex queries
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
let repo = UserRepository::new(pool); repo.create("alice", "a@b.com").await?;
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
