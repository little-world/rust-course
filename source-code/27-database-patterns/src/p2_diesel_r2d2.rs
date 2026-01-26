//! Pattern 2: Diesel with r2d2 Connection Pooling
//!
//! Demonstrates combining Diesel's type-safe ORM with r2d2 pooling.
//! This is the standard pattern for synchronous Diesel applications.

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Schema definition
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
struct User {
    id: i32,
    username: String,
    email: String,
    created_at: NaiveDateTime,
}

/// Create a pooled connection manager for Diesel
fn create_diesel_pool(database_url: &str) -> Result<Pool, r2d2::Error> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().max_size(15).build(manager)
}

/// Fetch user using pooled connection
fn fetch_user_pooled(pool: &Pool, user_id: i32) -> QueryResult<User> {
    // Get connection from pool
    let mut conn = pool.get().expect("Failed to get connection from pool");

    use self::users::dsl::*;

    // The pooled connection dereferences to &mut PgConnection
    // so existing Diesel queries work unchanged
    users.find(user_id).first(&mut conn)
}

/// Demonstrate pool sharing across threads
fn demonstrate_threaded_access(pool: &Pool) {
    use std::thread;

    let handles: Vec<_> = (1..=3)
        .map(|i| {
            let pool = pool.clone(); // Clone is cheap (Arc)
            thread::spawn(move || {
                println!("Thread {}: getting connection...", i);
                match pool.get() {
                    Ok(mut conn) => {
                        println!("Thread {}: got connection!", i);
                        // Use connection...
                        let result: QueryResult<i64> = diesel::select(diesel::dsl::count_star())
                            .first(&mut conn);
                        match result {
                            Ok(count) => println!("Thread {}: count = {}", i, count),
                            Err(e) => println!("Thread {}: query error: {}", i, e),
                        }
                    }
                    Err(e) => println!("Thread {}: pool error: {}", i, e),
                }
            })
        })
        .collect();

    for handle in handles {
        let _ = handle.join();
    }
}

fn main() {
    println!("=== Pattern 2: Diesel with r2d2 Pooling ===\n");

    println!("--- Why Combine Diesel + r2d2? ---\n");
    println!("Diesel provides: Type-safe query DSL");
    println!("r2d2 provides:   Connection lifecycle management\n");

    println!("--- How It Works ---\n");
    println!("1. ConnectionManager<PgConnection> adapts Diesel to r2d2");
    println!("2. Pool manages creation, recycling, health checks");
    println!("3. Pooled connection dereferences to &mut PgConnection");
    println!("4. Existing Diesel queries work unchanged\n");

    println!("--- For Async Diesel ---\n");
    println!("Use diesel-async with deadpool-diesel instead\n");

    // Try to create pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:pass@localhost/mydb".to_string());

    println!("--- Pool Creation ---\n");

    match create_diesel_pool(&database_url) {
        Ok(pool) => {
            println!("Pool created successfully!");
            let state = pool.state();
            println!("  Connections: {}", state.connections);
            println!("  Idle: {}", state.idle_connections);

            println!("\n--- Single Query ---\n");
            match fetch_user_pooled(&pool, 1) {
                Ok(user) => println!("Fetched: {:?}", user),
                Err(e) => println!("Query error: {}", e),
            }

            println!("\n--- Multi-threaded Access ---\n");
            demonstrate_threaded_access(&pool);
        }
        Err(e) => {
            println!("Pool creation failed: {}", e);
            println!("(Expected without running database)\n");
            println!("This pattern is standard for Diesel applications:");
            println!("  - Pool handles connection management");
            println!("  - Diesel handles type-safe queries");
            println!("  - Both work together seamlessly");
        }
    }

    println!("\nDiesel + r2d2 example completed!");
}
