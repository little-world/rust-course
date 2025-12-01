### Actix-web Cheat Sheet
```rust
// ===== ACTIX-WEB =====
// Cargo.toml:
/*
[dependencies]
actix-web = "4"
actix-files = "0.6"
actix-multipart = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/

use actix_web::{
    web, App, HttpServer, HttpRequest, HttpResponse, Responder,
    middleware as actix_middleware,
};
use actix_files as fs;

// ===== ACTIX BASIC SERVER =====
#[actix_web::main]
async fn actix_basic() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(actix_root))
            .route("/hello/{name}", web::get().to(actix_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn actix_root() -> impl Responder {
    "Hello, World!"
}

async fn actix_hello(path: web::Path<String>) -> impl Responder {
    format!("Hello, {}!", path.into_inner())
}

// ===== ACTIX ROUTES =====
fn actix_routes() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .route("/", web::get().to(actix_root))
        .service(
            web::scope("/users")
                .route("", web::get().to(actix_get_users))
                .route("", web::post().to(actix_create_user))
                .route("/{id}", web::get().to(actix_get_user))
                .route("/{id}", web::put().to(actix_update_user))
                .route("/{id}", web::delete().to(actix_delete_user))
        )
}

// ===== ACTIX HANDLERS =====
// Path parameters
async fn actix_get_user(path: web::Path<u32>) -> impl Responder {
    format!("User ID: {}", path.into_inner())
}

// Multiple path parameters
async fn actix_get_post(path: web::Path<(u32, u32)>) -> impl Responder {
    let (user_id, post_id) = path.into_inner();
    format!("User: {}, Post: {}", user_id, post_id)
}

// Query parameters
async fn actix_get_users(query: web::Query<Pagination>) -> impl Responder {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    format!("Page: {}, Limit: {}", page, limit)
}

// ===== ACTIX JSON =====
// Return JSON
async fn actix_get_user_json(path: web::Path<u32>) -> impl Responder {
    let user = User {
        id: path.into_inner(),
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    
    web::Json(user)
}

// Accept JSON
async fn actix_create_user(user: web::Json<User>) -> impl Responder {
    println!("Creating user: {:?}", user);
    HttpResponse::Created().json(user.into_inner())
}

// ===== ACTIX STATE =====
use actix_web::web::Data;

struct ActixAppState {
    db: Arc<Mutex<Vec<User>>>,
}

async fn actix_list_users(data: Data<ActixAppState>) -> impl Responder {
    let db = data.db.lock().await;
    web::Json(db.clone())
}

async fn actix_add_user(
    data: Data<ActixAppState>,
    user: web::Json<User>,
) -> impl Responder {
    let mut db = data.db.lock().await;
    db.push(user.into_inner());
    HttpResponse::Created()
}

fn actix_with_state() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let state = Data::new(ActixAppState {
        db: Arc::new(Mutex::new(Vec::new())),
    });
    
    App::new()
        .app_data(state)
        .route("/users", web::get().to(actix_list_users))
        .route("/users", web::post().to(actix_add_user))
}

// ===== ACTIX HEADERS =====
async fn actix_read_headers(req: HttpRequest) -> impl Responder {
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    format!("User-Agent: {}", user_agent)
}

// Return custom headers
async fn actix_with_headers() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("X-Custom-Header", "value"))
        .body("Response with headers")
}

// ===== ACTIX RESPONSE TYPES =====
// HTML response
async fn actix_html() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<h1>Hello, World!</h1>")
}

// Redirect
async fn actix_redirect() -> impl Responder {
    HttpResponse::Found()
        .insert_header(("Location", "/"))
        .finish()
}

// Custom status
async fn actix_not_found() -> impl Responder {
    HttpResponse::NotFound().body("Not found")
}

// ===== ACTIX ERROR HANDLING =====
use actix_web::error::ResponseError;
use std::fmt;

#[derive(Debug)]
struct ActixAppError {
    message: String,
}

impl fmt::Display for ActixAppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for ActixAppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": self.message
        }))
    }
}

async fn actix_fallible() -> Result<String, ActixAppError> {
    Ok("Success".to_string())
}

// ===== ACTIX MIDDLEWARE =====
use actix_web::middleware::{Logger, Compress};

fn actix_with_middleware() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .wrap(Logger::default())                             // Logging
        .wrap(Compress::default())                           // Compression
        .route("/", web::get().to(actix_root))
}

// Custom middleware
use actix_web::dev::{Service, Transform, ServiceRequest, ServiceResponse};
use actix_web::Error as ActixError;
use futures::future::{ready, Ready, LocalBoxFuture};

struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    
    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        if let Some(auth) = auth_header {
            if auth.to_str().unwrap_or("").starts_with("Bearer ") {
                let fut = self.service.call(req);
                return Box::pin(async move { fut.await });
            }
        }
        
        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("Unauthorized"))
        })
    }
}

// ===== ACTIX FILE UPLOADS =====
use actix_multipart::Multipart;
use futures::StreamExt;

async fn actix_upload(mut payload: Multipart) -> Result<HttpResponse, ActixError> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        let content_disposition = field.content_disposition();
        let filename = content_disposition.get_filename().unwrap();
        
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            data.extend_from_slice(&chunk?);
        }
        
        tokio::fs::write(format!("uploads/{}", filename), data).await?;
    }
    
    Ok(HttpResponse::Ok().body("Files uploaded"))
}

// ===== ACTIX WEBSOCKETS =====
use actix_web_actors::ws;
use actix::{Actor, StreamHandler};

struct MyWebSocket;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("Received: {}", text);
                ctx.text(format!("Echo: {}", text));
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
            }
            _ => {}
        }
    }
}

async fn actix_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, ActixError> {
    ws::start(MyWebSocket {}, &req, stream)
}

// ===== ACTIX STATIC FILES =====
fn actix_static_files() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .service(fs::Files::new("/static", "static").show_files_listing())
        .route("/", web::get().to(actix_root))
}

// ===== ACTIX GUARDS =====
use actix_web::guard;

fn actix_with_guards() -> App
    impl actix_web::dev::ServiceFactory
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .service(
            web::resource("/")
                .guard(guard::Get())
                .to(actix_root)
        )
        .service(
            web::resource("/api")
                .guard(guard::Header("content-type", "application/json"))
                .to(actix_api)
        )
}

async fn actix_api() -> impl Responder {
    "API endpoint"
}

// Stub implementations for missing functions
async fn update_user() -> &'static str { "Updated" }
async fn delete_user() -> &'static str { "Deleted" }
async fn actix_update_user() -> impl Responder { "Updated" }
async fn actix_delete_user() -> impl Responder { "Deleted" }
```