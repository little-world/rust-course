//! Pattern 4: axum Middleware
//!
//! Demonstrates request/response middleware for logging, timing, and auth.

use axum::{
    Router,
    routing::get,
    middleware::{self, Next},
    response::{Response, IntoResponse},
    http::{Request, StatusCode},
    body::Body,
};
use std::time::Instant;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    println!("=== Pattern 4: axum Middleware ===\n");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/public", get(|| async { "This is public" }))
        .route("/protected", get(|| async { "This is protected - you must have auth!" }))
        // Add middleware to all routes
        // Note: Middleware runs in reverse order (last added runs first)
        .layer(middleware::from_fn(timing_middleware))
        .layer(middleware::from_fn(logging_middleware));

    // Protected route with auth middleware
    let protected_app = Router::new()
        .route("/admin", get(|| async { "Admin area" }))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(timing_middleware))
        .layer(middleware::from_fn(logging_middleware));

    let app = app.merge(protected_app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server with middleware running on http://{}", addr);
    println!("\nTest endpoints:");
    println!("  curl http://localhost:3000/");
    println!("  curl http://localhost:3000/public");
    println!("  curl http://localhost:3000/protected");
    println!("  curl http://localhost:3000/admin              (401 - no auth)");
    println!("  curl -H 'Authorization: Bearer secret' http://localhost:3000/admin");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Middleware that logs requests
async fn logging_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    println!("[LOG] --> {} {}", method, uri);

    let response = next.run(request).await;

    println!("[LOG] <-- {} {} ({})", method, uri, response.status());

    response
}

/// Middleware that measures request timing
async fn timing_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Response {
    let start = Instant::now();
    let uri = request.uri().clone();

    let response = next.run(request).await;

    let elapsed = start.elapsed();
    println!("[TIMING] {} took {:?}", uri, elapsed);

    response
}

/// Middleware that checks for authentication
async fn auth_middleware(
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, impl IntoResponse> {
    // Check for auth header
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                println!("[AUTH] Valid Bearer token found");
                // Valid auth, continue
                return Ok(next.run(request).await);
            }
        }
    }

    println!("[AUTH] No valid auth header - rejecting");

    // No valid auth - return 401
    Err((
        StatusCode::UNAUTHORIZED,
        "Missing or invalid Authorization header. Use: Authorization: Bearer <token>"
    ))
}
