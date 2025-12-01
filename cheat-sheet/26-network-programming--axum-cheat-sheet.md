### Axum Cheat Sheet

```rust
// ===== AXUM =====
// Cargo.toml:
/*
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "trace", "cors"] }
*/

use axum::{
    routing::{get, post, put, delete},
    Router, Json, extract::{Path, Query, State},
    response::{IntoResponse, Response, Html},
    http::{StatusCode, HeaderMap, Method},
    middleware,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

// ===== AXUM BASIC SERVER =====
#[tokio::main]
async fn axum_basic() {
    let app = Router::new()
        .route("/", get(root))
        .route("/hello/:name", get(hello));
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    println!("Server running on http://localhost:3000");
    
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn hello(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}

// ===== AXUM ROUTES =====
fn axum_routes() -> Router {
    Router::new()
        .route("/", get(root))                               // GET /
        .route("/users", get(get_users).post(create_user))  // GET, POST /users
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .route("/static/*path", get(serve_static))          // Wildcard route
}

// ===== AXUM HANDLERS =====
// Path parameters
async fn get_user(Path(id): Path<u32>) -> String {
    format!("User ID: {}", id)
}

// Multiple path parameters
async fn get_post(Path((user_id, post_id)): Path<(u32, u32)>) -> String {
    format!("User: {}, Post: {}", user_id, post_id)
}

// Query parameters
#[derive(Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn get_users(Query(pagination): Query<Pagination>) -> String {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(10);
    format!("Page: {}, Limit: {}", page, limit)
}

// ===== AXUM JSON =====
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Return JSON
async fn get_user_json(Path(id): Path<u32>) -> Json<User> {
    let user = User {
        id,
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    Json(user)
}

// Accept JSON
async fn create_user(Json(user): Json<User>) -> (StatusCode, Json<User>) {
    println!("Creating user: {:?}", user);
    (StatusCode::CREATED, Json(user))
}

// ===== AXUM STATE =====
// Shared application state
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Vec<User>>>,
}

fn axum_with_state() -> Router {
    let state = AppState {
        db: Arc::new(Mutex::new(Vec::new())),
    };
    
    Router::new()
        .route("/users", get(list_users).post(add_user))
        .with_state(state)
}

async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    let db = state.db.lock().await;
    Json(db.clone())
}

async fn add_user(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> StatusCode {
    let mut db = state.db.lock().await;
    db.push(user);
    StatusCode::CREATED
}

// ===== AXUM HEADERS =====
async fn read_headers(headers: HeaderMap) -> String {
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    format!("User-Agent: {}", user_agent)
}

// Return custom headers
async fn with_headers() -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.insert("X-Custom-Header", "value".parse().unwrap());
    (headers, "Response with headers")
}

// ===== AXUM RESPONSE TYPES =====
use axum::response::Redirect;

// HTML response
async fn html_response() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

// Redirect
async fn redirect_handler() -> Redirect {
    Redirect::to("/")
}

// Status code with body
async fn not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

// Custom response
async fn custom_response() -> Response {
    (
        StatusCode::OK,
        [("X-Custom", "value")],
        "Custom response",
    )
        .into_response()
}

// ===== AXUM ERROR HANDLING =====
use axum::http::header;

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn fallible_handler() -> Result<String, AppError> {
    // May return error
    Ok("Success".to_string())
}

// ===== AXUM MIDDLEWARE =====
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};

fn axum_with_middleware() -> Router {
    let app = Router::new()
        .route("/", get(root))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())           // Logging
                .layer(CorsLayer::permissive())              // CORS
        );
    
    app
}

// Custom middleware
use axum::middleware::Next;
use axum::extract::Request;

async fn auth_middleware(
    headers: HeaderMap,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    
    if let Some(auth) = auth_header {
        if auth.starts_with("Bearer ") {
            return Ok(next.run(req).await);
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

fn with_auth_middleware() -> Router {
    Router::new()
        .route("/protected", get(protected_route))
        .layer(middleware::from_fn(auth_middleware))
}

async fn protected_route() -> &'static str {
    "Protected resource"
}

// ===== AXUM FILE UPLOADS =====
use axum::extract::Multipart;

async fn upload_file(mut multipart: Multipart) -> Result<String, StatusCode> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        
        println!("Field: {}, Size: {} bytes", name, data.len());
        
        // Save file
        tokio::fs::write(format!("uploads/{}", name), data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok("Files uploaded".to_string())
}

// ===== AXUM WEBSOCKETS =====
use axum::extract::ws::{WebSocket, WebSocketUpgrade, Message};

async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => break,
        };
        
        match msg {
            Message::Text(text) => {
                println!("Received: {}", text);
                socket.send(Message::Text(format!("Echo: {}", text)))
                    .await
                    .unwrap();
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

// ===== AXUM NESTED ROUTES =====
fn axum_nested() -> Router {
    let api_routes = Router::new()
        .route("/users", get(get_users))
        .route("/posts", get(get_posts));
    
    Router::new()
        .route("/", get(root))
        .nest("/api", api_routes)                            // /api/users, /api/posts
        .nest("/admin", admin_routes())
}

fn admin_routes() -> Router {
    Router::new()
        .route("/dashboard", get(admin_dashboard))
        .route("/settings", get(admin_settings))
}

async fn get_posts() -> &'static str { "Posts" }
async fn admin_dashboard() -> &'static str { "Dashboard" }
async fn admin_settings() -> &'static str { "Settings" }

// ===== AXUM STATIC FILES =====
use tower_http::services::ServeDir;

fn axum_static_files() -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", get(root))
}

async fn serve_static(Path(path): Path<String>) -> Response {
    // Custom static file handling
    (StatusCode::OK, format!("Serving: {}", path)).into_response()
}
```