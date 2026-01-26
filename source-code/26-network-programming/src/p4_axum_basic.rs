//! Pattern 4: Basic axum Server
//!
//! Demonstrates a basic HTTP server with routing, extractors, and JSON handling.

use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{Path, Query},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Deserialize)]
struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

#[tokio::main]
async fn main() {
    println!("=== Pattern 4: Basic axum Server ===\n");

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user));

    // Run the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running on http://{}", addr);
    println!("\nTest endpoints:");
    println!("  curl http://localhost:3000/");
    println!("  curl http://localhost:3000/users");
    println!("  curl http://localhost:3000/users?page=1&per_page=5");
    println!("  curl http://localhost:3000/users/1");
    println!("  curl -X POST http://localhost:3000/users -H 'Content-Type: application/json' -d '{{\"username\":\"alice\",\"email\":\"alice@example.com\"}}'");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Handler for GET /
async fn root_handler() -> &'static str {
    "Hello, World! Welcome to the axum server."
}

// Handler for GET /users
async fn list_users(Query(params): Query<ListQuery>) -> Json<Vec<User>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);

    println!("GET /users - page={}, per_page={}", page, per_page);

    // In a real app, fetch from database
    let users = vec![
        User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        },
    ];

    Json(users)
}

// Handler for GET /users/:id
async fn get_user(Path(user_id): Path<u64>) -> Result<Json<User>, StatusCode> {
    println!("GET /users/{}", user_id);

    // Simulate database lookup
    if user_id == 1 {
        Ok(Json(User {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        }))
    } else if user_id == 2 {
        Ok(Json(User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Handler for POST /users
async fn create_user(
    Json(payload): Json<CreateUserRequest>,
) -> (StatusCode, Json<User>) {
    println!("POST /users - Creating user: {}", payload.username);

    // In a real app, save to database and return the created user
    let user = User {
        id: 42, // Would come from database
        username: payload.username,
        email: payload.email,
    };

    (StatusCode::CREATED, Json(user))
}
