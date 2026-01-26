//! Pattern 4: axum with Shared State
//!
//! Demonstrates managing shared application state with Arc and RwLock.

use axum::{
    routing::{get, post},
    Router,
    extract::State,
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;

#[derive(Clone)]
struct AppState {
    // Use Arc for shared ownership across tasks
    // RwLock allows multiple readers or one writer
    db: Arc<RwLock<Database>>,
    config: Arc<Config>,
}

struct Database {
    users: Vec<User>,
    next_id: u64,
}

struct Config {
    max_users: usize,
}

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
}

#[tokio::main]
async fn main() {
    println!("=== Pattern 4: axum with Shared State ===\n");

    let state = AppState {
        db: Arc::new(RwLock::new(Database {
            users: vec![
                User { id: 1, name: "Alice".to_string() },
                User { id: 2, name: "Bob".to_string() },
            ],
            next_id: 3,
        })),
        config: Arc::new(Config {
            max_users: 100,
        }),
    };

    let app = Router::new()
        .route("/users", get(get_all_users).post(create_user))
        .route("/stats", get(get_stats))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Stateful server running on http://{}", addr);
    println!("\nTest endpoints:");
    println!("  curl http://localhost:3000/users");
    println!("  curl http://localhost:3000/stats");
    println!("  curl -X POST http://localhost:3000/users -H 'Content-Type: application/json' -d '{{\"name\":\"Charlie\"}}'");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_all_users(
    State(state): State<AppState>,
) -> Json<Vec<User>> {
    println!("GET /users");

    // Acquire read lock (multiple readers allowed)
    let db = state.db.read().await;
    Json(db.users.clone())
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    println!("POST /users - Creating: {}", payload.name);

    // Check config limit
    let db = state.db.read().await;
    if db.users.len() >= state.config.max_users {
        return Err((StatusCode::BAD_REQUEST, "Max users reached".to_string()));
    }
    drop(db); // Release read lock before acquiring write lock

    // Acquire write lock (exclusive)
    let mut db = state.db.write().await;

    let user = User {
        id: db.next_id,
        name: payload.name,
    };
    db.next_id += 1;
    db.users.push(user.clone());

    println!("  Created user with id {}", user.id);

    Ok((StatusCode::CREATED, Json(user)))
}

#[derive(Serialize)]
struct Stats {
    total_users: usize,
    max_users: usize,
}

async fn get_stats(
    State(state): State<AppState>,
) -> Json<Stats> {
    println!("GET /stats");

    let db = state.db.read().await;
    Json(Stats {
        total_users: db.users.len(),
        max_users: state.config.max_users,
    })
}
