//! Pattern 5: Complete Application Example
//!
//! Production-ready architecture combining all patterns:
//! - Connection pooling
//! - Repository pattern
//! - Compile-time checked queries
//! - Analytics with raw SQL

use chrono::NaiveDateTime;
use sqlx::{FromRow, PgPool};

// ============================================
// Domain Models
// ============================================

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

#[derive(FromRow, Debug)]
pub struct GrowthMetric {
    pub date: Option<NaiveDateTime>,
    pub new_users: Option<i64>,
    pub cumulative_users: Option<i64>,
}

// ============================================
// Application State
// ============================================

pub struct AppState {
    pool: PgPool,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub fn user_repo(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    pub fn analytics(&self) -> Analytics {
        Analytics::new(self.pool.clone())
    }
}

// ============================================
// User Repository (CRUD operations)
// ============================================

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, username: &str, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, created_at)
            VALUES ($1, $2, NOW())
            RETURNING id, username, email, created_at
            "#,
        )
        .bind(username)
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: i32) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, email, created_at FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list_active(&self, days: i32) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, created_at
            FROM users
            WHERE created_at > NOW() - $1 * INTERVAL '1 day'
            ORDER BY created_at DESC
            "#,
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_email(&self, id: i32, new_email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET email = $1
            WHERE id = $2
            RETURNING id, username, email, created_at
            "#,
        )
        .bind(new_email)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn delete(&self, id: i32) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

// ============================================
// Analytics (Complex queries)
// ============================================

pub struct Analytics {
    pool: PgPool,
}

impl Analytics {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn user_growth_report(&self) -> Result<Vec<GrowthMetric>, sqlx::Error> {
        sqlx::query_as::<_, GrowthMetric>(
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
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn user_count(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0)
    }
}

// ============================================
// Example Usage
// ============================================

async fn demonstrate_app(app: &AppState) -> Result<(), sqlx::Error> {
    let user_repo = app.user_repo();
    let analytics = app.analytics();

    // CRUD operations
    println!("--- CRUD Operations ---\n");

    // Create
    let user = user_repo.create("alice", "alice@example.com").await?;
    println!("Created: {:?}", user);

    // Read
    if let Some(found) = user_repo.find_by_id(user.id).await? {
        println!("Found: {:?}", found);
    }

    // Update
    let updated = user_repo.update_email(user.id, "alice@newdomain.com").await?;
    println!("Updated: {:?}", updated);

    // List
    let recent = user_repo.list_active(7).await?;
    println!("Recent users: {} found", recent.len());

    // Analytics
    println!("\n--- Analytics ---\n");

    let count = analytics.user_count().await?;
    println!("Total users: {}", count);

    let growth = analytics.user_growth_report().await?;
    println!("Growth report: {} days of data", growth.len());

    // Cleanup
    user_repo.delete(user.id).await?;
    println!("\nDeleted test user");

    Ok(())
}

fn explain_architecture() {
    println!("=== Complete Application Architecture ===\n");

    println!("--- Layers ---\n");
    println!("AppState");
    println!("  ├── pool: PgPool (shared connection pool)");
    println!("  ├── user_repo() -> UserRepository");
    println!("  └── analytics() -> Analytics\n");

    println!("UserRepository");
    println!("  ├── create(username, email) -> User");
    println!("  ├── find_by_id(id) -> Option<User>");
    println!("  ├── find_by_email(email) -> Option<User>");
    println!("  ├── list_active(days) -> Vec<User>");
    println!("  ├── update_email(id, email) -> User");
    println!("  └── delete(id) -> bool\n");

    println!("Analytics");
    println!("  ├── user_growth_report() -> Vec<GrowthMetric>");
    println!("  └── user_count() -> i64\n");

    println!("--- Benefits ---\n");
    println!("1. Clear separation of concerns");
    println!("2. Repository encapsulates all user queries");
    println!("3. Analytics isolated for complex SQL");
    println!("4. Pool shared across all repositories");
    println!("5. Easy to test (mock repositories)\n");

    println!("--- Integration with Web Framework ---\n");
    println!("// Axum example");
    println!("async fn get_user(");
    println!("    State(app): State<AppState>,");
    println!("    Path(id): Path<i32>,");
    println!(") -> impl IntoResponse {{");
    println!("    let repo = app.user_repo();");
    println!("    match repo.find_by_id(id).await {{");
    println!("        Ok(Some(user)) => Json(user).into_response(),");
    println!("        Ok(None) => StatusCode::NOT_FOUND.into_response(),");
    println!("        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),");
    println!("    }}");
    println!("}}\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 5: Complete Application Example ===\n");

    explain_architecture();

    // Try real connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Live Test ---\n");

    match AppState::new(&database_url).await {
        Ok(app) => {
            match demonstrate_app(&app).await {
                Ok(()) => println!("\nApplication demo completed!"),
                Err(e) => println!("Error: {}", e),
            }
        }
        Err(e) => {
            println!("Connection failed: {}", e);
            println!("(Expected without running database)\n");
            println!("This architecture works with any web framework:");
            println!("  - Axum: State<AppState>");
            println!("  - Actix: web::Data<AppState>");
            println!("  - Rocket: State<AppState>");
        }
    }

    println!("\nComplete application example completed!");
    Ok(())
}
