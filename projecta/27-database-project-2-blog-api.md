# Chapter 27: Database Patterns

## Project 2: Type-Safe Blog API with SQLx

### Problem Statement

Build a REST API for a blog platform that demonstrates SQLx's compile-time SQL verification. The API must support:
- **User management**: Registration, authentication, user profiles
- **Content operations**: Create/read/update/delete posts and comments
- **Relationships**: Posts belong to users, comments belong to posts
- **Advanced queries**: Filter by tags, search by title/content, paginate results
- **Full-text search**: PostgreSQL tsvector for fast content search
- **Type safety**: All SQL verified at compile time, preventing typos and type mismatches

The key learning goal is experiencing how SQLx catches errors at **compile time** that would normally be **runtime failures** in traditional SQL approaches.

### Why It Matters

**Real-World Impact**: Type safety prevents entire classes of bugs:
- **Runtime SQL errors eliminated**: Typo "usrname" → compile error, not production panic
- **Type mismatches caught early**: `let id: String = row.get(0)` for INT column won't compile
- **SQL injection impossible**: Bound parameters automatically escaped
- **Refactoring safety**: Rename database column → compile errors show all affected queries

**Performance Numbers**:
- **Compile-time verification overhead**: ~1-2s during development (database connection for macro)
- **Runtime overhead**: Zero! No reflection, no dynamic SQL parsing
- **Offline mode**: `sqlx prepare` caches metadata, compiles without database in CI
- **vs ORMs**: SQLx gives SQL control with type safety, ORMs abstract SQL but add complexity

**Rust-Specific Advantage**: Most languages (Python/SQLAlchemy, Node/Sequelize) do SQL verification at **runtime**:
```python
# Python - typo only discovered when this line runs
user = session.query(User).filter_by(usrname="alice").first()  # Runtime error!
```

With SQLx:
```rust
// Rust with SQLx - typo discovered at compile time
let user = sqlx::query!("SELECT * FROM users WHERE usrname = $1", "alice");  // Won't compile!
```

### Use Cases

**When you need this pattern**:
1. **Blog platforms** - Medium, WordPress-style content management with authors, posts, comments
2. **Documentation sites** - Searchable docs with categories, tags, versioning
3. **Content APIs** - Headless CMS for mobile apps, JAMstack sites
4. **News websites** - Articles, authors, categories, full-text search
5. **Knowledge bases** - Stack Overflow-style Q&A, wikis
6. **Social media backends** - Posts, comments, likes, user relationships

**Real Examples**:
- **Ghost (Node.js)**: Open-source blogging platform, 3M+ sites
- **WordPress REST API**: Powers headless WordPress deployments
- **Dev.to**: Community blog platform, Rust backend (Rocket + Diesel)
- **Medium**: 100M+ readers, complex content relationships

### Learning Goals

- Master SQLx `query!()` and `query_as!()` macros for compile-time verification
- Understand relationships (foreign keys, JOINs) with type-safe queries
- Learn dynamic query building with `QueryBuilder` (prevent SQL injection)
- Practice full-text search with PostgreSQL tsvector/tsquery
- Experience offline mode (`sqlx prepare`) for CI/CD pipelines
- Build REST API integrating SQLx with axum framework

---

## Introduction to SQLx and Type-Safe Database Concepts

SQLx brings a revolutionary approach to database programming: compile-time verification of SQL queries. This means errors that would typically crash your application at runtime are caught during compilation. Understanding the concepts behind SQLx transforms how you think about database interactions.

### 1. Compile-Time SQL Verification

Most database libraries verify SQL at runtime—when your code executes:

**Traditional Approach (Runtime Verification)**:
```python
# Python with SQLAlchemy
user = session.query(User).filter_by(usrname="alice").first()  # Typo!
# Error discovered only when this line runs in production
```

**SQLx Approach (Compile-Time Verification)**:
```rust
// SQLx connects to database during compilation
let user = sqlx::query!("SELECT * FROM users WHERE usrname = $1", "alice");
// Won't compile! SQLx checks "usrname" column doesn't exist
```

**How It Works**:
1. During compilation, SQLx macros (`query!`, `query_as!`) connect to your database
2. The actual SQL is sent to PostgreSQL for analysis
3. PostgreSQL returns the query plan, column types, and validates syntax
4. SQLx generates Rust code with exact types inferred from database schema
5. Compiler enforces these types throughout your code

**Benefits**:
- Typos in column names = compile error
- Type mismatches = compile error
- Non-existent tables = compile error
- Wrong number of parameters = compile error
- **Zero runtime overhead** - all verification done at compile time

### 2. The `query!()` and `query_as!()` Macros

These macros are the core of SQLx's type safety:

**`query!()` - Anonymous Records**:
```rust
let row = sqlx::query!("SELECT id, username FROM users WHERE id = $1", 42)
    .fetch_one(&pool)
    .await?;

// row.id has type i32 (inferred from database)
// row.username has type String
// If you typo row.usrname, won't compile!
```

**`query_as!()` - Map to Structs**:
```rust
#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    username: String,
}

let user = sqlx::query_as!(User, "SELECT id, username FROM users WHERE id = $1", 42)
    .fetch_one(&pool)
    .await?;

// Returns User struct directly
// Field names must match SQL columns (compile-time verified)
```

**Key Difference**: `query!()` creates anonymous struct, `query_as!()` maps to your struct.

### 3. Foreign Keys and Referential Integrity

Foreign keys ensure relationships between tables remain valid—you can't have orphaned records.

**Foreign Key Definition**:
```sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    author_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE
);
```

**What This Enforces**:
- Can't insert post with non-existent `author_id` (database rejects it)
- `ON DELETE CASCADE`: Deleting a user automatically deletes their posts
- `ON DELETE RESTRICT`: Prevents deleting user if they have posts
- `ON DELETE SET NULL`: Sets `author_id` to NULL when user deleted

**SQLx Verification**: At compile time, SQLx verifies the `author_id` column exists and is the correct type (INT matching `users.id`).

### 4. JOIN Queries for Efficient Data Retrieval

JOINs combine data from multiple tables in a single query, avoiding the N+1 problem.

**N+1 Problem (Bad)**:
```rust
// 1 query for posts
let posts = get_all_posts().await?;

// N queries for authors (one per post)
for post in posts {
    let author = get_user(post.author_id).await?;  // N separate queries!
    println!("{} by {}", post.title, author.username);
}
// Total: 1 + N queries
```

**JOIN Solution (Good)**:
```rust
// Single query fetches posts WITH authors
let posts = sqlx::query_as!(
    PostWithAuthor,
    "SELECT posts.*, users.username FROM posts JOIN users ON posts.author_id = users.id"
)
.fetch_all(&pool)
.await?;

// Total: 1 query
for post in posts {
    println!("{} by {}", post.title, post.username);
}
```

**JOIN Types**:
- `INNER JOIN`: Only rows with matches in both tables
- `LEFT JOIN`: All rows from left table, NULL for missing right table matches
- `RIGHT JOIN`: All rows from right table, NULL for missing left table matches

### 5. Many-to-Many Relationships with Junction Tables

When entities have multiple relationships (post has many tags, tag has many posts), use a junction table:

**Schema**:
```sql
CREATE TABLE tags (id SERIAL PRIMARY KEY, name VARCHAR(50));
CREATE TABLE post_tags (
    post_id INT REFERENCES posts(id),
    tag_id INT REFERENCES tags(id),
    PRIMARY KEY (post_id, tag_id)  -- Composite key prevents duplicates
);
```

**Querying**:
```rust
// Find all posts with "rust" tag
let posts = sqlx::query_as!(
    Post,
    "SELECT posts.* FROM posts
     JOIN post_tags ON posts.id = post_tags.post_id
     JOIN tags ON post_tags.tag_id = tags.id
     WHERE tags.name = $1",
    "rust"
)
.fetch_all(&pool)
.await?;
```

**Pattern**: Join through the junction table to connect the two entities.

### 6. Dynamic Query Building with QueryBuilder

Hard-coded queries can't handle optional filters. `QueryBuilder` builds SQL dynamically while preventing SQL injection:

**The Problem**:
```rust
// API endpoint: /posts?author=alice&tag=rust&published=true
// Need different WHERE clauses based on which params are provided
```

**QueryBuilder Solution**:
```rust
let mut query = QueryBuilder::new("SELECT * FROM posts WHERE 1=1");

if let Some(author) = filter.author {
    query.push(" AND author_id = ");
    query.push_bind(author);  // Safely bound, not concatenated
}

if let Some(tag) = filter.tag {
    query.push(" AND id IN (SELECT post_id FROM post_tags WHERE tag_id = ");
    query.push_bind(tag);
    query.push(")");
}

let posts = query.build_query_as::<Post>().fetch_all(pool).await?;
```

**SQL Injection Prevention**: `push_bind()` uses parameterized queries—values are never concatenated into SQL string, preventing injection attacks.

### 7. Aggregation Functions and GROUP BY

Aggregations compute summary statistics across multiple rows:

**Common Aggregates**:
```sql
COUNT(*)        -- Number of rows
COUNT(DISTINCT column) -- Unique values
SUM(amount)     -- Total
AVG(rating)     -- Average
MAX(created_at) -- Latest timestamp
MIN(price)      -- Lowest price
```

**GROUP BY Pattern**:
```rust
struct UserPostCount {
    username: String,
    post_count: i64,  // Note: ! in query means NOT NULL
}

let stats = sqlx::query_as!(
    UserPostCount,
    r#"
    SELECT users.username, COUNT(posts.id) as "post_count!"
    FROM users
    LEFT JOIN posts ON users.id = posts.author_id
    GROUP BY users.id, users.username
    HAVING COUNT(posts.id) > 5  -- Filter groups
    ORDER BY post_count DESC
    "#
)
.fetch_all(&pool)
.await?;
```

**Key Insight**: `GROUP BY` collapses rows, aggregates compute values for each group.

### 8. PostgreSQL Full-Text Search

Full-text search is far superior to `LIKE` queries for text searching:

**LIKE Approach (Slow)**:
```sql
-- Scans entire table, no ranking, case-sensitive
SELECT * FROM posts WHERE content LIKE '%rust%';
```

**Full-Text Search (Fast)**:
```sql
-- Uses GIN index, ranked results, stemming support
SELECT *, ts_rank(search_vector, query) as rank
FROM posts, to_tsquery('english', 'rust & programming') query
WHERE search_vector @@ query
ORDER BY rank DESC;
```

**Key Components**:
- **tsvector**: Pre-processed, indexed text column (stored in database)
- **tsquery**: Search query with operators (`&` = AND, `|` = OR, `!` = NOT)
- **GIN index**: Makes search 1000x faster than LIKE
- **ts_rank**: Computes relevance score
- **Stemming**: Searches "running" matches "run", "runs", "runner"

**Setup**:
```sql
ALTER TABLE posts ADD COLUMN search_vector tsvector;
CREATE INDEX idx_search ON posts USING GIN(search_vector);

-- Auto-update trigger
CREATE TRIGGER update_search
BEFORE INSERT OR UPDATE ON posts
FOR EACH ROW EXECUTE FUNCTION
tsvector_update_trigger(search_vector, 'pg_catalog.english', title, content);
```

### 9. Database Migrations

Migrations are version-controlled schema changes, ensuring all environments have the same database structure.

**Migration File** (`migrations/001_create_users.sql`):
```sql
-- Up migration
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE
);

-- Down migration (in separate file or section)
DROP TABLE users;
```

**Running Migrations**:
```bash
sqlx migrate run  # Apply pending migrations
sqlx migrate revert  # Rollback last migration
```

**Benefits**:
- Database schema in version control (git)
- Reproducible: Same schema in dev, staging, production
- Rollback support: Undo migrations if needed
- Team collaboration: Everyone applies same schema changes

### 10. Offline Mode for CI/CD

SQLx macros need database access at compile time. Offline mode solves the CI/CD problem:

**The Problem**: GitHub Actions doesn't have your PostgreSQL database.

**The Solution**: Cache query metadata locally.

**Workflow**:
```bash
# Developer (with database running)
cargo sqlx prepare
# Creates .sqlx/query-abc123.json files with type metadata

# Commit to git
git add .sqlx/
git commit -m "Update SQLx metadata"

# CI (no database)
SQLX_OFFLINE=true cargo build
# Uses cached metadata instead of connecting to database
```

**Metadata Files**: JSON files containing column names, types, nullability for each query.

**Verification**:
```bash
cargo sqlx prepare --check  # Fails if metadata outdated
```

### Connection to This Project

This blog API project applies every SQLx concept in a practical, real-world context:

**Compile-Time Verification (All Milestones)**: Every `query!()` and `query_as!()` call is verified against your actual database schema. Renaming a column triggers compile errors everywhere that column is used—refactoring with confidence.

**Type Safety (Milestone 1)**: You'll experience the difference between runtime SQL errors (other languages) and compile-time verification (Rust+SQLx). A typo in a column name won't compile, period.

**Foreign Keys (Milestone 2)**: The `posts.author_id → users.id` relationship is enforced by PostgreSQL and verified by SQLx at compile time. Deleting a user cascades to their posts, preventing orphaned data.

**JOINs (Milestone 2)**: Fetching posts with author information demonstrates the N+1 problem and how a single JOIN query is more efficient than multiple queries.

**Many-to-Many (Milestone 3)**: The `posts ↔ tags` relationship via `post_tags` junction table is the standard pattern for tagging systems (like Medium, Dev.to).

**QueryBuilder (Milestone 3)**: The dynamic search API (`/posts?author=alice&tag=rust&page=2`) requires building queries at runtime. QueryBuilder provides flexibility without SQL injection risk.

**Aggregations (Milestone 4)**: Analytics like "most prolific authors" and "most commented posts" use `GROUP BY` and `COUNT()` to generate statistics—common in admin dashboards.

**Full-Text Search (Milestone 5)**: The `/search?q=rust+programming` endpoint uses PostgreSQL's tsvector/tsquery, providing Google-like search with ranking and stemming.

**Migrations (All Milestones)**: Each milestone adds tables/columns via migration files, showing how to evolve database schema over time in a controlled, version-controlled way.

**Offline Mode (Milestone 6)**: The `cargo sqlx prepare` workflow enables CI/CD pipelines to build your project without database access, critical for GitHub Actions and Docker builds.

By the end of this project, you'll have built a **production-ready blog API** with the same architecture as Medium, Dev.to, and WordPress—with compile-time guarantees that prevent entire classes of runtime errors.

---

## Milestone 1: Basic CRUD with query!() Macro

### Introduction

**Starting Point**: Learn SQLx's core feature—compile-time verified queries with the `query!()` macro.

**What We're Building**: A users table with:
- Basic operations: INSERT, SELECT, UPDATE, DELETE
- Compile-time verification: Wrong column names won't compile
- Type inference: Return types automatically inferred from SQL

**Key Feature**: If you typo a column name or use wrong types, your code won't compile. This is revolutionary for database programming.

### Key Concepts

**Database Schema**:
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    bio TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
```

**SQLx Macros**:
- `query!()` - Returns anonymous record with compile-time verified fields
- `query_as!()` - Maps result to a struct
- Both connect to database at compile time to verify SQL

**Structs/Types**:
```rust
#[derive(sqlx::FromRow)]
struct User {
    id: i32,
    username: String,
    email: String,
    bio: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}
```

**Functions and Their Roles**:
```rust
async fn create_user(pool: &PgPool, username: &str, email: &str) -> Result<User>
    // INSERT user, RETURNING all fields
    // Uses query_as!() macro for compile-time verification

async fn get_user_by_id(pool: &PgPool, user_id: i32) -> Result<Option<User>>
    // SELECT * FROM users WHERE id = $1
    // Returns Option<User> (None if not found)

async fn update_bio(pool: &PgPool, user_id: i32, bio: &str) -> Result<()>
    // UPDATE users SET bio = $1 WHERE id = $2

async fn delete_user(pool: &PgPool, user_id: i32) -> Result<()>
    // DELETE FROM users WHERE id = $1
```

### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn test_create_user(pool: PgPool) -> sqlx::Result<()> {
        let user = create_user(&pool, "alice", "alice@example.com").await?;

        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
        assert!(user.id > 0);
        assert!(user.bio.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_user_by_id(pool: PgPool) -> sqlx::Result<()> {
        let created = create_user(&pool, "bob", "bob@example.com").await?;

        let fetched = get_user_by_id(&pool, created.id).await?;
        assert!(fetched.is_some());

        let user = fetched.unwrap();
        assert_eq!(user.id, created.id);
        assert_eq!(user.username, "bob");

        Ok(())
    }

    #[sqlx::test]
    async fn test_update_bio(pool: PgPool) -> sqlx::Result<()> {
        let user = create_user(&pool, "charlie", "charlie@example.com").await?;

        update_bio(&pool, user.id, "Rust enthusiast").await?;

        let updated = get_user_by_id(&pool, user.id).await?.unwrap();
        assert_eq!(updated.bio, Some("Rust enthusiast".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_user(pool: PgPool) -> sqlx::Result<()> {
        let user = create_user(&pool, "dave", "dave@example.com").await?;

        delete_user(&pool, user.id).await?;

        let result = get_user_by_id(&pool, user.id).await?;
        assert!(result.is_none());

        Ok(())
    }

    #[sqlx::test]
    async fn test_unique_constraint(pool: PgPool) -> sqlx::Result<()> {
        create_user(&pool, "eve", "eve@example.com").await?;

        // Try to create duplicate username
        let result = create_user(&pool, "eve", "different@example.com").await;
        assert!(result.is_err()); // Should fail due to unique constraint

        Ok(())
    }
}
```

### Starter Code

```rust
use sqlx::{PgPool, FromRow};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow)]
struct User {
    id: i32,
    username: String,
    email: String,
    bio: Option<String>,
    created_at: DateTime<Utc>,
}

/// Create a new user
async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
) -> Result<User, sqlx::Error> {
    // TODO: Use query_as! macro to INSERT and RETURN user
    // The macro will verify at compile time that:
    // - Table "users" exists
    // - Columns "username", "email" exist
    // - Return type matches User struct

    let user = todo!(); // sqlx::query_as!(User, "INSERT INTO users (username, email) VALUES ($1, $2) RETURNING id, username, email, bio, created_at", username, email).fetch_one(pool).await?

    Ok(user)
}

/// Get user by ID
async fn get_user_by_id(
    pool: &PgPool,
    user_id: i32,
) -> Result<Option<User>, sqlx::Error> {
    // TODO: SELECT user by ID
    // fetch_optional() returns Option<User> instead of error if not found

    let user = todo!(); // sqlx::query_as!(User, "SELECT id, username, email, bio, created_at FROM users WHERE id = $1", user_id).fetch_optional(pool).await?

    Ok(user)
}

/// Update user bio
async fn update_bio(
    pool: &PgPool,
    user_id: i32,
    bio: &str,
) -> Result<(), sqlx::Error> {
    // TODO: UPDATE bio field

    todo!(); // sqlx::query!("UPDATE users SET bio = $1 WHERE id = $2", bio, user_id).execute(pool).await?

    Ok(())
}

/// Delete user
async fn delete_user(
    pool: &PgPool,
    user_id: i32,
) -> Result<(), sqlx::Error> {
    // TODO: DELETE user by ID

    todo!(); // sqlx::query!("DELETE FROM users WHERE id = $1", user_id).execute(pool).await?

    Ok(())
}

/// Create database pool
async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = "postgresql://user:pass@localhost/blogdb";
    let pool = create_pool(database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Example usage
    let user = create_user(&pool, "alice", "alice@example.com").await?;
    println!("Created user: {:?}", user);

    Ok(())
}
```

### Check Your Understanding

- **What happens if you typo "username" as "usrname" in the query?** Won't compile! SQLx macro checks column exists at compile time.
- **What if you use wrong type, like `let id: String` for an INT column?** Compile error! SQLx infers types from database schema.
- **Why use `fetch_optional()` vs `fetch_one()`?** `fetch_one()` returns error if no rows, `fetch_optional()` returns `None` (better for lookups).
- **How does the macro know the database schema at compile time?** Connects to database using `DATABASE_URL` environment variable during compilation.

---

## Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Missing Features**:
1. **No relationships**: Real blogs have posts belonging to users, comments belonging to posts
2. **No JOINs**: Can't efficiently fetch user with their posts in one query
3. **Single table only**: Real apps have multiple related tables

**What We're Adding**:
- **posts table**: Foreign key to users (author_id)
- **comments table**: Foreign keys to users and posts
- **JOIN queries**: Fetch posts with author information
- **Type-safe relationships**: Compile-time verified foreign keys

**Improvement**:
- **Relational data**: Model real-world relationships (users → posts → comments)
- **Efficient queries**: JOINs avoid N+1 problem (fetch posts + authors in one query)
- **Referential integrity**: Foreign keys enforced by database

---

## Milestone 2: Relationships and JOINs

### Introduction

**The Problem**: Real applications have related data. A blog post has an author (user). A comment belongs to both a post and a user.

**The Solution**: Foreign keys and JOIN queries:
- `posts.author_id` references `users.id`
- `comments.post_id` references `posts.id`, `comments.user_id` references `users.id`
- JOIN queries fetch related data efficiently

**Key Learning**: SQLx verifies foreign key columns exist and types match at compile time.

### Key Concepts

**Database Schema**:
```sql
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    author_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    published BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    post_id INT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_posts_author ON posts(author_id);
CREATE INDEX idx_comments_post ON comments(post_id);
CREATE INDEX idx_comments_user ON comments(user_id);
```

**Structs for Relationships**:
```rust
#[derive(FromRow)]
struct Post {
    id: i32,
    author_id: i32,
    title: String,
    content: String,
    published: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct PostWithAuthor {
    // Post fields
    id: i32,
    title: String,
    content: String,
    published: bool,
    created_at: DateTime<Utc>,
    // Author fields (from JOIN)
    author_id: i32,
    author_username: String,
    author_email: String,
}

#[derive(FromRow)]
struct Comment {
    id: i32,
    post_id: i32,
    user_id: i32,
    content: String,
    created_at: DateTime<Utc>,
}
```

**Functions**:
```rust
async fn create_post(pool: &PgPool, author_id: i32, title: &str, content: &str) -> Result<Post>
    // INSERT INTO posts with foreign key to users

async fn get_posts_with_authors(pool: &PgPool) -> Result<Vec<PostWithAuthor>>
    // SELECT posts.*, users.username, users.email FROM posts JOIN users

async fn create_comment(pool: &PgPool, post_id: i32, user_id: i32, content: &str) -> Result<Comment>
    // INSERT INTO comments with foreign keys

async fn get_post_comments(pool: &PgPool, post_id: i32) -> Result<Vec<Comment>>
    // SELECT comments for a specific post
```

### Checkpoint Tests

```rust
#[sqlx::test]
async fn test_create_post(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "author", "author@example.com").await?;
    let post = create_post(&pool, user.id, "My First Post", "Hello world!").await?;

    assert_eq!(post.author_id, user.id);
    assert_eq!(post.title, "My First Post");
    assert!(!post.published); // Default false

    Ok(())
}

#[sqlx::test]
async fn test_posts_with_authors_join(pool: PgPool) -> sqlx::Result<()> {
    let user1 = create_user(&pool, "alice", "alice@example.com").await?;
    let user2 = create_user(&pool, "bob", "bob@example.com").await?;

    create_post(&pool, user1.id, "Alice's Post", "Content 1").await?;
    create_post(&pool, user2.id, "Bob's Post", "Content 2").await?;

    let posts = get_posts_with_authors(&pool).await?;

    assert_eq!(posts.len(), 2);
    assert_eq!(posts[0].author_username, "alice");
    assert_eq!(posts[1].author_username, "bob");

    Ok(())
}

#[sqlx::test]
async fn test_foreign_key_cascade(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "temp", "temp@example.com").await?;
    let post = create_post(&pool, user.id, "Temporary", "Content").await?;

    // Delete user (should cascade delete post)
    delete_user(&pool, user.id).await?;

    // Post should be gone
    let posts = sqlx::query_as!(Post, "SELECT * FROM posts WHERE id = $1", post.id)
        .fetch_optional(&pool)
        .await?;

    assert!(posts.is_none());

    Ok(())
}

#[sqlx::test]
async fn test_comments_relationships(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "commenter", "commenter@example.com").await?;
    let post = create_post(&pool, user.id, "Post", "Content").await?;

    let comment = create_comment(&pool, post.id, user.id, "Great post!").await?;

    assert_eq!(comment.post_id, post.id);
    assert_eq!(comment.user_id, user.id);

    let comments = get_post_comments(&pool, post.id).await?;
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].content, "Great post!");

    Ok(())
}
```

### Starter Code

```rust
#[derive(Debug, FromRow)]
struct Post {
    id: i32,
    author_id: i32,
    title: String,
    content: String,
    published: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct PostWithAuthor {
    id: i32,
    title: String,
    content: String,
    published: bool,
    created_at: DateTime<Utc>,
    author_id: i32,
    author_username: String,
    author_email: String,
}

#[derive(Debug, FromRow)]
struct Comment {
    id: i32,
    post_id: i32,
    user_id: i32,
    content: String,
    created_at: DateTime<Utc>,
}

/// Create a new post
async fn create_post(
    pool: &PgPool,
    author_id: i32,
    title: &str,
    content: &str,
) -> Result<Post, sqlx::Error> {
    // TODO: INSERT post with foreign key to users
    let post = todo!(); // sqlx::query_as!(Post, "INSERT INTO posts (author_id, title, content) VALUES ($1, $2, $3) RETURNING id, author_id, title, content, published, created_at, updated_at", author_id, title, content).fetch_one(pool).await?

    Ok(post)
}

/// Get posts with author information (JOIN)
async fn get_posts_with_authors(pool: &PgPool) -> Result<Vec<PostWithAuthor>, sqlx::Error> {
    // TODO: SELECT posts with JOIN to users
    // Notice how SQLx verifies the JOIN columns exist and types match
    let posts = todo!(); // sqlx::query_as!(PostWithAuthor, r#"SELECT posts.id, posts.title, posts.content, posts.published, posts.created_at, posts.author_id, users.username as author_username, users.email as author_email FROM posts JOIN users ON posts.author_id = users.id ORDER BY posts.created_at DESC"#).fetch_all(pool).await?

    Ok(posts)
}

/// Create a comment
async fn create_comment(
    pool: &PgPool,
    post_id: i32,
    user_id: i32,
    content: &str,
) -> Result<Comment, sqlx::Error> {
    // TODO: INSERT comment with foreign keys to posts and users
    let comment = todo!(); // sqlx::query_as!(Comment, "INSERT INTO comments (post_id, user_id, content) VALUES ($1, $2, $3) RETURNING id, post_id, user_id, content, created_at", post_id, user_id, content).fetch_one(pool).await?

    Ok(comment)
}

/// Get comments for a post
async fn get_post_comments(pool: &PgPool, post_id: i32) -> Result<Vec<Comment>, sqlx::Error> {
    // TODO: SELECT comments WHERE post_id = $1
    let comments = todo!(); // sqlx::query_as!(Comment, "SELECT id, post_id, user_id, content, created_at FROM comments WHERE post_id = $1 ORDER BY created_at ASC", post_id).fetch_all(pool).await?

    Ok(comments)
}
```

---

## Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Missing Features**:
1. **No dynamic filtering**: Can't search posts by tag, filter by author, sort by date
2. **Hard-coded queries**: Real APIs need runtime query building based on user input
3. **SQL injection risk**: String concatenation for dynamic queries is dangerous

**What We're Adding**:
- **QueryBuilder**: Build queries dynamically while maintaining type safety
- **Tags table**: Many-to-many relationship (posts ↔ tags via junction table)
- **Search filters**: Filter by tag, author, published status
- **Pagination**: Limit/offset for large result sets
- **SQL injection prevention**: All values bound as parameters

**Improvement**:
- **Flexibility**: API accepts filter params, builds query dynamically
- **Security**: QueryBuilder prevents SQL injection automatically
- **Scalability**: Pagination handles thousands of posts efficiently

---

## Milestone 3: Dynamic Queries with QueryBuilder

### Introduction

**The Problem**: APIs need dynamic queries based on user input:
- Search posts by tag: `/posts?tag=rust`
- Filter by author: `/posts?author=alice`
- Pagination: `/posts?page=2&per_page=10`

**The Solution**: SQLx's `QueryBuilder` allows building queries programmatically:
- Conditionally add WHERE clauses based on parameters
- All values automatically bound (SQL injection impossible)
- Less type safety than macros, but still safe from injection

### Key Concepts

**Tags Schema (Many-to-Many)**:
```sql
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE post_tags (
    post_id INT NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    tag_id INT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (post_id, tag_id)
);

CREATE INDEX idx_post_tags_post ON post_tags(post_id);
CREATE INDEX idx_post_tags_tag ON post_tags(tag_id);
```

**QueryBuilder Pattern**:
```rust
let mut query = QueryBuilder::new("SELECT * FROM posts WHERE 1=1");

if let Some(author_id) = filter.author_id {
    query.push(" AND author_id = ");
    query.push_bind(author_id);
}

if let Some(tag) = filter.tag {
    query.push(" AND id IN (SELECT post_id FROM post_tags JOIN tags ON tag_id = tags.id WHERE tags.name = ");
    query.push_bind(tag);
    query.push(")");
}

let posts = query.build_query_as::<Post>().fetch_all(pool).await?;
```

**Functions**:
```rust
async fn search_posts(pool: &PgPool, filter: PostFilter) -> Result<Vec<Post>>
    // Dynamically build query based on filter fields
    // Add WHERE clauses only for provided filters

async fn add_tag_to_post(pool: &PgPool, post_id: i32, tag_name: &str) -> Result<()>
    // INSERT INTO post_tags (many-to-many relationship)

async fn get_posts_by_tag(pool: &PgPool, tag_name: &str) -> Result<Vec<Post>>
    // JOIN through post_tags to find posts with specific tag
```

### Checkpoint Tests

```rust
#[derive(Default)]
struct PostFilter {
    author_id: Option<i32>,
    tag: Option<String>,
    published: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[sqlx::test]
async fn test_dynamic_filter_by_author(pool: PgPool) -> sqlx::Result<()> {
    let user1 = create_user(&pool, "alice", "alice@example.com").await?;
    let user2 = create_user(&pool, "bob", "bob@example.com").await?;

    create_post(&pool, user1.id, "Alice's Post", "Content").await?;
    create_post(&pool, user2.id, "Bob's Post", "Content").await?;

    let filter = PostFilter {
        author_id: Some(user1.id),
        ..Default::default()
    };

    let posts = search_posts(&pool, filter).await?;
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].author_id, user1.id);

    Ok(())
}

#[sqlx::test]
async fn test_search_by_tag(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "author", "author@example.com").await?;
    let post1 = create_post(&pool, user.id, "Rust Post", "Content").await?;
    let post2 = create_post(&pool, user.id, "Python Post", "Content").await?;

    add_tag_to_post(&pool, post1.id, "rust").await?;
    add_tag_to_post(&pool, post2.id, "python").await?;

    let posts = get_posts_by_tag(&pool, "rust").await?;
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].id, post1.id);

    Ok(())
}

#[sqlx::test]
async fn test_pagination(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "author", "author@example.com").await?;

    // Create 15 posts
    for i in 0..15 {
        create_post(&pool, user.id, &format!("Post {}", i), "Content").await?;
    }

    // First page (10 posts)
    let filter = PostFilter {
        limit: Some(10),
        offset: Some(0),
        ..Default::default()
    };
    let page1 = search_posts(&pool, filter).await?;
    assert_eq!(page1.len(), 10);

    // Second page (5 posts)
    let filter = PostFilter {
        limit: Some(10),
        offset: Some(10),
        ..Default::default()
    };
    let page2 = search_posts(&pool, filter).await?;
    assert_eq!(page2.len(), 5);

    Ok(())
}

#[sqlx::test]
async fn test_sql_injection_prevention(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "victim", "victim@example.com").await?;
    create_post(&pool, user.id, "Safe Post", "Content").await?;

    // Attempt SQL injection
    let malicious_tag = "rust'; DROP TABLE posts; --";

    // QueryBuilder binds this as a string value, doesn't execute SQL
    let posts = get_posts_by_tag(&pool, malicious_tag).await?;
    assert_eq!(posts.len(), 0); // No posts with that tag

    // Verify posts table still exists
    let all_posts = sqlx::query_as!(Post, "SELECT * FROM posts")
        .fetch_all(&pool)
        .await?;
    assert_eq!(all_posts.len(), 1); // Table intact!

    Ok(())
}
```

### Starter Code

```rust
use sqlx::QueryBuilder;

#[derive(Default)]
struct PostFilter {
    author_id: Option<i32>,
    tag: Option<String>,
    published: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// Search posts with dynamic filters
async fn search_posts(
    pool: &PgPool,
    filter: PostFilter,
) -> Result<Vec<Post>, sqlx::Error> {
    // TODO: Build dynamic query
    let mut query = QueryBuilder::new(
        "SELECT id, author_id, title, content, published, created_at, updated_at FROM posts WHERE 1=1"
    );

    // TODO: Add filters conditionally
    if let Some(author_id) = filter.author_id {
        query.push(" AND author_id = ");
        todo!(); // query.push_bind(author_id);
    }

    if let Some(published) = filter.published {
        query.push(" AND published = ");
        todo!(); // query.push_bind(published);
    }

    if let Some(tag) = filter.tag {
        query.push(" AND id IN (SELECT post_id FROM post_tags JOIN tags ON tag_id = tags.id WHERE tags.name = ");
        todo!(); // query.push_bind(tag);
        query.push(")");
    }

    // TODO: Add ordering
    query.push(" ORDER BY created_at DESC");

    // TODO: Add pagination
    if let Some(limit) = filter.limit {
        query.push(" LIMIT ");
        todo!(); // query.push_bind(limit);
    }

    if let Some(offset) = filter.offset {
        query.push(" OFFSET ");
        todo!(); // query.push_bind(offset);
    }

    // TODO: Execute query
    let posts = todo!(); // query.build_query_as::<Post>().fetch_all(pool).await?

    Ok(posts)
}

/// Add tag to post (many-to-many)
async fn add_tag_to_post(
    pool: &PgPool,
    post_id: i32,
    tag_name: &str,
) -> Result<(), sqlx::Error> {
    // TODO: Get or create tag
    let tag_id: i32 = sqlx::query_scalar!(
        "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id",
        tag_name
    )
    .fetch_one(pool)
    .await?;

    // TODO: Insert into junction table
    todo!(); // sqlx::query!("INSERT INTO post_tags (post_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING", post_id, tag_id).execute(pool).await?

    Ok(())
}

/// Get posts by tag
async fn get_posts_by_tag(
    pool: &PgPool,
    tag_name: &str,
) -> Result<Vec<Post>, sqlx::Error> {
    // TODO: JOIN through post_tags
    let posts = todo!(); // sqlx::query_as!(Post, r#"SELECT posts.id, posts.author_id, posts.title, posts.content, posts.published, posts.created_at, posts.updated_at FROM posts JOIN post_tags ON posts.id = post_tags.post_id JOIN tags ON post_tags.tag_id = tags.id WHERE tags.name = $1"#, tag_name).fetch_all(pool).await?

    Ok(posts)
}
```

---

## Why Milestone 3 Isn't Enough → Moving to Milestone 4

**Missing Features**:
1. **No aggregations**: Can't count posts per user, comments per post
2. **No analytics**: Need statistics (most commented posts, top authors)
3. **No GROUP BY**: Can't calculate totals, averages, counts

**What We're Adding**:
- **Aggregation queries**: COUNT, SUM, AVG with GROUP BY
- **Statistics endpoints**: Most popular posts, prolific authors
- **Subqueries**: Complex analytics (posts with >10 comments)

**Improvement**:
- **Analytics dashboard**: Show meaningful statistics
- **Complex queries**: SQLx handles GROUP BY, HAVING, subqueries
- **Type safety**: Aggregate results verified at compile time

---

## Milestone 4: Aggregations and Analytics

### Introduction

**Real-World Need**: APIs need statistics:
- Dashboard: Total posts, total users, posts per user
- Leaderboard: Most prolific authors, most commented posts
- Filtering: Show only posts with >5 comments

**The Solution**: SQL aggregation functions with type-safe results.

### Key Concepts

**Aggregation Functions**:
- `COUNT(*)` - Count rows
- `COUNT(DISTINCT column)` - Count unique values
- `SUM(column)` - Sum numeric column
- `AVG(column)` - Average
- `MAX/MIN(column)` - Extremes

**GROUP BY Queries**:
```rust
struct UserPostCount {
    user_id: i32,
    username: String,
    post_count: i64,
}

let stats = sqlx::query_as!(
    UserPostCount,
    r#"
    SELECT users.id as user_id, users.username, COUNT(posts.id) as "post_count!"
    FROM users
    LEFT JOIN posts ON users.id = posts.author_id
    GROUP BY users.id, users.username
    ORDER BY post_count DESC
    "#
)
.fetch_all(pool)
.await?;
```

**Note**: `"post_count!"` tells SQLx the column is NOT NULL (COUNT never returns NULL).

### Checkpoint Tests

```rust
#[derive(FromRow)]
struct UserStats {
    user_id: i32,
    username: String,
    post_count: i64,
    comment_count: i64,
}

#[sqlx::test]
async fn test_user_post_counts(pool: PgPool) -> sqlx::Result<()> {
    let user1 = create_user(&pool, "alice", "alice@example.com").await?;
    let user2 = create_user(&pool, "bob", "bob@example.com").await?;

    // Alice: 3 posts, Bob: 1 post
    for _ in 0..3 {
        create_post(&pool, user1.id, "Title", "Content").await?;
    }
    create_post(&pool, user2.id, "Title", "Content").await?;

    let stats = get_user_post_counts(&pool).await?;

    assert_eq!(stats.len(), 2);
    assert_eq!(stats[0].username, "alice");
    assert_eq!(stats[0].post_count, 3);
    assert_eq!(stats[1].username, "bob");
    assert_eq!(stats[1].post_count, 1);

    Ok(())
}

#[sqlx::test]
async fn test_most_commented_posts(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "author", "author@example.com").await?;
    let post1 = create_post(&pool, user.id, "Popular", "Content").await?;
    let post2 = create_post(&pool, user.id, "Unpopular", "Content").await?;

    // post1: 5 comments, post2: 1 comment
    for _ in 0..5 {
        create_comment(&pool, post1.id, user.id, "Comment").await?;
    }
    create_comment(&pool, post2.id, user.id, "Comment").await?;

    let popular = get_most_commented_posts(&pool, 10).await?;

    assert_eq!(popular[0].post_id, post1.id);
    assert_eq!(popular[0].comment_count, 5);
    assert_eq!(popular[1].post_id, post2.id);
    assert_eq!(popular[1].comment_count, 1);

    Ok(())
}
```

### Starter Code

```rust
#[derive(FromRow)]
struct UserPostCount {
    user_id: i32,
    username: String,
    post_count: i64,
}

#[derive(FromRow)]
struct PostCommentCount {
    post_id: i32,
    title: String,
    comment_count: i64,
}

/// Get post count per user
async fn get_user_post_counts(pool: &PgPool) -> Result<Vec<UserPostCount>, sqlx::Error> {
    // TODO: GROUP BY user, COUNT posts
    let stats = todo!(); // sqlx::query_as!(UserPostCount, r#"SELECT users.id as user_id, users.username, COUNT(posts.id) as "post_count!" FROM users LEFT JOIN posts ON users.id = posts.author_id GROUP BY users.id, users.username ORDER BY post_count DESC"#).fetch_all(pool).await?

    Ok(stats)
}

/// Get most commented posts
async fn get_most_commented_posts(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<PostCommentCount>, sqlx::Error> {
    // TODO: JOIN posts with comments, COUNT, ORDER BY count DESC
    let posts = todo!(); // sqlx::query_as!(PostCommentCount, r#"SELECT posts.id as post_id, posts.title, COUNT(comments.id) as "comment_count!" FROM posts LEFT JOIN comments ON posts.id = comments.post_id GROUP BY posts.id, posts.title ORDER BY comment_count DESC LIMIT $1"#, limit).fetch_all(pool).await?

    Ok(posts)
}

/// Get total statistics
async fn get_global_stats(pool: &PgPool) -> Result<GlobalStats, sqlx::Error> {
    #[derive(FromRow)]
    struct GlobalStats {
        total_users: i64,
        total_posts: i64,
        total_comments: i64,
    }

    // TODO: Use scalar aggregates
    let stats = todo!(); // sqlx::query_as!(GlobalStats, r#"SELECT (SELECT COUNT(*) FROM users) as "total_users!", (SELECT COUNT(*) FROM posts) as "total_posts!", (SELECT COUNT(*) FROM comments) as "total_comments!""#).fetch_one(pool).await?

    Ok(stats)
}
```

---

## Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Missing Feature**: Full-text search is slow and limited:
- `WHERE content LIKE '%rust%'` scans entire table (slow for millions of rows)
- Case-sensitive, no stemming (searching "running" won't find "run")
- No relevance ranking (which result is most relevant?)

**What We're Adding**:
- **PostgreSQL tsvector**: Indexed full-text search column
- **tsquery**: Search query language (supports AND/OR/NOT, phrase search)
- **Ranking**: Order results by relevance
- **GIN index**: Makes search fast even with millions of posts

**Improvement**:
- **Performance**: GIN index makes search 1000x faster than LIKE
- **Relevance**: Results ranked by match quality
- **Stemming**: Search "running" finds "run", "runs", "runner"

---

## Milestone 5: Full-Text Search with PostgreSQL

### Introduction

**The Problem**: LIKE queries are slow and limited:
```sql
-- Slow, no ranking, no stemming
SELECT * FROM posts WHERE content LIKE '%rust programming%';
```

**The Solution**: PostgreSQL's built-in full-text search:
```sql
-- Fast (indexed), ranked, stemming support
SELECT * FROM posts WHERE search_vector @@ to_tsquery('rust & programming')
ORDER BY ts_rank(search_vector, to_tsquery('rust & programming')) DESC;
```

### Key Concepts

**Schema Changes**:
```sql
ALTER TABLE posts ADD COLUMN search_vector tsvector;

-- Create GIN index for fast search
CREATE INDEX idx_posts_search ON posts USING GIN(search_vector);

-- Auto-update search_vector on INSERT/UPDATE
CREATE TRIGGER posts_search_update
BEFORE INSERT OR UPDATE ON posts
FOR EACH ROW EXECUTE FUNCTION
tsvector_update_trigger(search_vector, 'pg_catalog.english', title, content);
```

**Search Syntax**:
- `rust` - Single word
- `rust & programming` - Both words (AND)
- `rust | python` - Either word (OR)
- `!python` - NOT python
- `'rust programming'` - Exact phrase

### Checkpoint Tests

```rust
#[sqlx::test]
async fn test_full_text_search(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "author", "author@example.com").await?;

    create_post(&pool, user.id, "Rust Programming", "Learn Rust fundamentals").await?;
    create_post(&pool, user.id, "Python Guide", "Python tutorial for beginners").await?;
    create_post(&pool, user.id, "Rust Async", "Asynchronous Rust programming").await?;

    // Search for "rust"
    let results = search_posts_fulltext(&pool, "rust").await?;
    assert_eq!(results.len(), 2); // Two posts contain "rust"

    // Search for "rust & async"
    let results = search_posts_fulltext(&pool, "rust & async").await?;
    assert_eq!(results.len(), 1); // Only one has both

    // Search for "rust | python"
    let results = search_posts_fulltext(&pool, "rust | python").await?;
    assert_eq!(results.len(), 3); // All three match

    Ok(())
}

#[sqlx::test]
async fn test_search_ranking(pool: PgPool) -> sqlx::Result<()> {
    let user = create_user(&pool, "author", "author@example.com").await?;

    create_post(&pool, user.id, "Rust", "rust rust rust").await?; // Many matches
    create_post(&pool, user.id, "Other", "One mention of rust").await?;

    let results = search_posts_fulltext(&pool, "rust").await?;

    // First result should have higher rank (more matches)
    assert!(results[0].title == "Rust");

    Ok(())
}
```

### Starter Code

```rust
#[derive(FromRow)]
struct SearchResult {
    id: i32,
    title: String,
    content: String,
    rank: f32,
}

/// Full-text search posts
async fn search_posts_fulltext(
    pool: &PgPool,
    query: &str,
) -> Result<Vec<SearchResult>, sqlx::Error> {
    // TODO: Use to_tsquery and ts_rank for full-text search
    let results = todo!(); // sqlx::query_as!(SearchResult, r#"SELECT id, title, content, ts_rank(search_vector, to_tsquery('english', $1)) as "rank!" FROM posts WHERE search_vector @@ to_tsquery('english', $1) ORDER BY rank DESC"#, query).fetch_all(pool).await?

    Ok(results)
}

/// Search with highlighting (show matches in context)
async fn search_with_highlights(
    pool: &PgPool,
    query: &str,
) -> Result<Vec<(Post, String)>, sqlx::Error> {
    // TODO: Use ts_headline to highlight matches
    let rows = sqlx::query!(
        r#"
        SELECT
            id, author_id, title, content, published, created_at, updated_at,
            ts_headline('english', content, to_tsquery('english', $1)) as headline
        FROM posts
        WHERE search_vector @@ to_tsquery('english', $1)
        ORDER BY ts_rank(search_vector, to_tsquery('english', $1)) DESC
        "#,
        query
    )
    .fetch_all(pool)
    .await?;

    let results = rows.into_iter().map(|row| {
        let post = Post {
            id: row.id,
            author_id: row.author_id,
            title: row.title,
            content: row.content.clone(),
            published: row.published,
            created_at: row.created_at,
            updated_at: row.updated_at,
        };
        (post, row.headline.unwrap_or(row.content))
    }).collect();

    Ok(results)
}
```

---

## Why Milestone 5 Isn't Enough → Moving to Milestone 6

**Missing Operational Feature**: CI/CD requires database for compilation (macros connect to DB).

**The Problem**: In CI pipelines, you might not have a live database:
- GitHub Actions: No PostgreSQL by default
- Docker builds: Don't want database dependency
- Offline development: Working without network

**What We're Adding**:
- **Offline mode**: `sqlx prepare` caches query metadata
- **`.sqlx/` directory**: Committed to git, contains type information
- **Compilation without database**: Uses cached metadata

**Improvement**:
- **CI/CD**: Builds work without database connection
- **Faster builds**: No database connection overhead during compilation
- **Reliability**: No network dependency for compilation

---

## Milestone 6: Offline Mode and CI/CD Integration

### Introduction

**The Solution**: SQLx can cache query metadata locally:
```bash
# During development (with database):
cargo sqlx prepare

# In CI (without database):
cargo build  # Uses .sqlx/ metadata
```

**Workflow**:
1. Developer writes queries with macros
2. Run `cargo sqlx prepare` → creates `.sqlx/query-*.json` files
3. Commit `.sqlx/` directory to git
4. CI builds using cached metadata (no database needed)

### Key Concepts

**Commands**:
```bash
# Prepare (cache metadata)
cargo sqlx prepare

# Check if metadata is up-to-date
cargo sqlx prepare --check

# Build with offline mode
SQLX_OFFLINE=true cargo build
```

**Example Workflow**:
```yaml
# .github/workflows/ci.yml
name: CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - name: Build (offline mode)
        run: SQLX_OFFLINE=true cargo build --release
```

### Checkpoint Tests

These tests verify the offline workflow works:

```rust
// No special tests needed - offline mode is a build-time feature
// Test by:
// 1. cargo sqlx prepare
// 2. SQLX_OFFLINE=true cargo build
// 3. Verify builds without database connection
```

### Starter Code

```bash
#!/bin/bash
# scripts/prepare-sqlx.sh

# Ensure database is running
if ! pg_isready -h localhost -p 5432 > /dev/null 2>&1; then
    echo "Error: PostgreSQL not running"
    exit 1
fi

# Run migrations
sqlx migrate run

# Prepare offline metadata
cargo sqlx prepare

echo "SQLx metadata prepared. Commit .sqlx/ directory to git."
```

**CI Configuration**:
```yaml
# .github/workflows/ci.yml
name: Rust CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Check SQLx metadata is up-to-date
        run: |
          cargo install sqlx-cli
          cargo sqlx prepare --check

      - name: Build (offline mode)
        run: SQLX_OFFLINE=true cargo build --release

      - name: Test (offline mode)
        run: SQLX_OFFLINE=true cargo test
```

---

### Testing Strategies

1. **Compile-Time Verification**:
   - Intentionally typo column names → verify won't compile
   - Use wrong types → verify type mismatch error
   - Reference non-existent tables → verify compile error

2. **CRUD Operations**:
   - Test all CREATE, READ, UPDATE, DELETE operations
   - Verify foreign key constraints enforced
   - Test CASCADE deletes work correctly

3. **Dynamic Queries**:
   - Test QueryBuilder with various filter combinations
   - Verify SQL injection attempts safely handled
   - Test pagination with edge cases (offset > total)

4. **Full-Text Search**:
   - Test various search operators (AND, OR, NOT, phrase)
   - Verify ranking orders results correctly
   - Test non-English text (if supporting i18n)

5. **Offline Mode**:
   - Run `cargo sqlx prepare`
   - Build with `SQLX_OFFLINE=true`
   - Verify metadata stays in sync (use `--check` in CI)

---

### Complete Working Example

This blog API demonstrates:
- **Compile-time SQL verification** - Catches typos and type errors before runtime
- **Relationships** - Foreign keys, JOINs, many-to-many (tags)
- **Dynamic queries** - QueryBuilder for runtime filtering
- **Aggregations** - Statistics, analytics, GROUP BY queries
- **Full-text search** - PostgreSQL tsvector/tsquery with ranking
- **Offline mode** - CI/CD without database dependency

**Production Deployment**:
```bash
# Development
cargo sqlx prepare
git add .sqlx/
git commit -m "Update SQLx metadata"

# CI/CD
SQLX_OFFLINE=true cargo build --release

# Production
./target/release/blog-api
```

**API Endpoints** (with axum integration):
- `POST /users` - Create user
- `GET /users/:id` - Get user with post count
- `POST /posts` - Create post (authenticated)
- `GET /posts?tag=rust&author=alice&page=1` - Search posts
- `POST /posts/:id/comments` - Add comment
- `GET /search?q=rust+programming` - Full-text search

This implementation is production-ready with:
- **Type safety**: No runtime SQL errors
- **Security**: SQL injection impossible
- **Performance**: Indexed searches, optimized queries
- **Scalability**: Pagination, efficient JOINs
- **CI/CD ready**: Offline mode for fast builds
