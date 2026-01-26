//! Pattern 4: actix-web Server
//!
//! Demonstrates the actix-web framework as an alternative to axum.

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    name: String,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    name: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("=== Pattern 4: actix-web Server ===\n");

    let addr = ("127.0.0.1", 3000);
    println!("actix-web server running on http://{}:{}", addr.0, addr.1);
    println!("\nTest endpoints:");
    println!("  curl http://localhost:3000/");
    println!("  curl http://localhost:3000/users");
    println!("  curl http://localhost:3000/users/1");
    println!("  curl -X POST http://localhost:3000/users -H 'Content-Type: application/json' -d '{{\"name\":\"Charlie\"}}'");
    println!("  curl http://localhost:3000/health");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/users", web::get().to(get_users))
            .route("/users", web::post().to(create_user))
            .route("/users/{id}", web::get().to(get_user))
            .route("/health", web::get().to(health_check))
    })
    .bind(addr)?
    .run()
    .await
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello from actix-web!")
}

async fn get_users() -> impl Responder {
    println!("GET /users");

    let users = vec![
        User { id: 1, name: "Alice".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    HttpResponse::Ok().json(users)
}

async fn create_user(user: web::Json<CreateUserRequest>) -> impl Responder {
    println!("POST /users - Creating: {}", user.name);

    let created = User {
        id: 42,
        name: user.name.clone(),
    };

    HttpResponse::Created().json(created)
}

async fn get_user(path: web::Path<u64>) -> impl Responder {
    let user_id = path.into_inner();
    println!("GET /users/{}", user_id);

    match user_id {
        1 => HttpResponse::Ok().json(User {
            id: 1,
            name: "Alice".to_string(),
        }),
        2 => HttpResponse::Ok().json(User {
            id: 2,
            name: "Bob".to_string(),
        }),
        _ => HttpResponse::NotFound().body("User not found"),
    }
}

#[derive(Serialize)]
struct HealthStatus {
    status: String,
    version: String,
}

async fn health_check() -> impl Responder {
    println!("GET /health");

    HttpResponse::Ok().json(HealthStatus {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
    })
}
