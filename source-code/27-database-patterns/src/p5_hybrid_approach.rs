//! Pattern 5: Hybrid ORM + Raw SQL Approach
//!
//! Use ORM for simple CRUD, raw SQL for complex queries.
//! Get the best of both worlds.

use chrono::NaiveDateTime;
use diesel::prelude::*;
use sqlx::{FromRow, PgPool};

// ============================================
// Diesel setup for simple CRUD
// ============================================

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        created_at -> Timestamp,
    }
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
struct DieselUser {
    id: i32,
    username: String,
    email: String,
    created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    username: &'a str,
    email: &'a str,
}

// ============================================
// SQLx setup for complex queries
// ============================================

#[derive(FromRow, Debug)]
struct SalesData {
    day: Option<NaiveDateTime>,
    total_orders: Option<i64>,
    total_revenue: Option<f64>,
    avg_order_value: Option<f64>,
}

#[derive(FromRow, Debug)]
struct SearchResult {
    id: i32,
    title: String,
    content: String,
}

// ============================================
// Simple CRUD with ORM (Diesel)
// ============================================

fn create_user_orm(conn: &mut PgConnection, name: &str, email: &str) -> QueryResult<DieselUser> {
    let new_user = NewUser {
        username: name,
        email,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(DieselUser::as_returning())
        .get_result(conn)
}

fn get_user_orm(conn: &mut PgConnection, user_id: i32) -> QueryResult<DieselUser> {
    users::table.find(user_id).first(conn)
}

fn update_user_orm(conn: &mut PgConnection, user_id: i32, new_email: &str) -> QueryResult<DieselUser> {
    diesel::update(users::table.find(user_id))
        .set(users::email.eq(new_email))
        .returning(DieselUser::as_returning())
        .get_result(conn)
}

fn delete_user_orm(conn: &mut PgConnection, user_id: i32) -> QueryResult<usize> {
    diesel::delete(users::table.find(user_id)).execute(conn)
}

// ============================================
// Complex analytics with raw SQL (SQLx)
// ============================================

async fn get_sales_report(pool: &PgPool) -> Result<Vec<SalesData>, sqlx::Error> {
    sqlx::query_as::<_, SalesData>(
        r#"
        SELECT
            date_trunc('day', created_at) as day,
            COUNT(*) as total_orders,
            SUM(amount) as total_revenue,
            AVG(amount) as avg_order_value
        FROM orders
        WHERE created_at >= NOW() - INTERVAL '30 days'
        GROUP BY day
        ORDER BY day DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

// ============================================
// Database-specific features with raw SQL
// ============================================

async fn full_text_search(pool: &PgPool, query: &str) -> Result<Vec<SearchResult>, sqlx::Error> {
    sqlx::query_as::<_, SearchResult>(
        r#"
        SELECT id, title, content
        FROM documents
        WHERE to_tsvector('english', title || ' ' || content)
              @@ plainto_tsquery('english', $1)
        ORDER BY ts_rank(
            to_tsvector('english', title || ' ' || content),
            plainto_tsquery('english', $1)
        ) DESC
        LIMIT 20
        "#,
    )
    .bind(query)
    .fetch_all(pool)
    .await
}

// ============================================
// CTE (Common Table Expression) example
// ============================================

async fn get_user_activity_summary(pool: &PgPool, user_id: i32) -> Result<(), sqlx::Error> {
    let _result = sqlx::query(
        r#"
        WITH user_orders AS (
            SELECT * FROM orders WHERE user_id = $1
        ),
        order_stats AS (
            SELECT
                COUNT(*) as order_count,
                SUM(amount) as total_spent,
                AVG(amount) as avg_order
            FROM user_orders
        )
        SELECT * FROM order_stats
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(())
}

fn explain_hybrid_approach() {
    println!("=== Hybrid ORM + Raw SQL ===\n");

    println!("--- When to Use ORM (Diesel) ---\n");
    println!("1. Standard CRUD operations");
    println!("2. Type safety is critical");
    println!("3. Queries map cleanly to tables");
    println!("4. Team benefits from schema enforcement\n");

    println!("Example:");
    println!("  users::table.find(id).first(conn)  // Simple, type-safe\n");

    println!("--- When to Use Raw SQL (SQLx) ---\n");
    println!("1. Complex joins and subqueries");
    println!("2. Aggregations and analytics");
    println!("3. Database-specific features:");
    println!("   - PostgreSQL: to_tsvector, JSONB, arrays");
    println!("   - CTEs (WITH clauses)");
    println!("   - Window functions");
    println!("4. Performance-critical queries\n");

    println!("Example:");
    println!("  SELECT date_trunc('day', created_at), SUM(amount)");
    println!("  FROM orders GROUP BY 1  // Analytics\n");

    println!("--- Practical Boundaries ---\n");
    println!("+-------------------+--------+-------+");
    println!("| Operation         | ORM    | SQLx  |");
    println!("+-------------------+--------+-------+");
    println!("| CRUD              | *      |       |");
    println!("| Simple filters    | *      |       |");
    println!("| Basic joins       | *      |       |");
    println!("| Aggregations      |        | *     |");
    println!("| Full-text search  |        | *     |");
    println!("| Window functions  |        | *     |");
    println!("| CTEs              |        | *     |");
    println!("| JSONB operators   |        | *     |");
    println!("+-------------------+--------+-------+\n");

    println!("--- Architecture ---\n");
    println!("struct UserRepository {{");
    println!("    diesel_conn: PgConnection,  // For CRUD");
    println!("    sqlx_pool: PgPool,          // For analytics");
    println!("}}");
    println!("");
    println!("impl UserRepository {{");
    println!("    // CRUD uses Diesel");
    println!("    fn create(&mut self, name: &str) -> DieselUser {{ ... }}");
    println!("    fn get(&mut self, id: i32) -> DieselUser {{ ... }}");
    println!("    ");
    println!("    // Analytics uses SQLx");
    println!("    async fn activity_report(&self) -> Vec<Activity> {{ ... }}");
    println!("}}\n");
}

fn main() {
    println!("=== Pattern 5: Hybrid ORM + Raw SQL ===\n");

    explain_hybrid_approach();

    println!("--- Summary ---\n");
    println!("Don't force everything into one approach.");
    println!("ORM for CRUD, SQL for complex queries.");
    println!("Both can coexist in the same codebase.\n");

    println!("Hybrid approach example completed!");
}
