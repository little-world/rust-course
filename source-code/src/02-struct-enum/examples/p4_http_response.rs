//! Pattern 4: Enum Design Patterns
//! Example: Basic Enum with Pattern Matching
//!
//! Run with: cargo run --example p4_http_response

// Model HTTP responses precisely
enum HttpResponse {
    Ok {
        body: String,
        headers: Vec<(String, String)>,
    },
    Created {
        id: u64,
        location: String,
    },
    NoContent,
    BadRequest {
        error: String,
    },
    Unauthorized,
    NotFound,
    ServerError {
        message: String,
        details: Option<String>,
    },
}

impl HttpResponse {
    fn status_code(&self) -> u16 {
        match self {
            HttpResponse::Ok { .. } => 200,
            HttpResponse::Created { .. } => 201,
            HttpResponse::NoContent => 204,
            HttpResponse::BadRequest { .. } => 400,
            HttpResponse::Unauthorized => 401,
            HttpResponse::NotFound => 404,
            HttpResponse::ServerError { .. } => 500,
        }
    }

    fn is_success(&self) -> bool {
        matches!(
            self,
            HttpResponse::Ok { .. } | HttpResponse::Created { .. } | HttpResponse::NoContent
        )
    }
}

fn handle_request(path: &str) -> HttpResponse {
    match path {
        "/users" => HttpResponse::Ok {
            body: "[{\"id\": 1}]".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        },
        "/users/create" => HttpResponse::Created {
            id: 123,
            location: "/users/123".to_string(),
        },
        "/health" => HttpResponse::NoContent,
        "/secret" => HttpResponse::Unauthorized,
        _ => HttpResponse::NotFound,
    }
}

fn main() {
    // Usage: Each variant carries its own data; match extracts it safely.
    let ok = HttpResponse::Ok {
        body: "Hello".to_string(),
        headers: vec![],
    };
    assert_eq!(ok.status_code(), 200);
    assert!(ok.is_success());

    println!("Testing various endpoints:\n");

    let paths = ["/users", "/users/create", "/health", "/secret", "/unknown"];

    for path in paths {
        let response = handle_request(path);
        println!("GET {} -> {} (success: {})",
            path,
            response.status_code(),
            response.is_success()
        );

        // Pattern match to extract data
        match &response {
            HttpResponse::Ok { body, headers } => {
                println!("  Body: {}", body);
                println!("  Headers: {:?}", headers);
            }
            HttpResponse::Created { id, location } => {
                println!("  Created ID: {}, Location: {}", id, location);
            }
            HttpResponse::ServerError { message, details } => {
                println!("  Error: {}", message);
                if let Some(d) = details {
                    println!("  Details: {}", d);
                }
            }
            _ => {}
        }
        println!();
    }
}
