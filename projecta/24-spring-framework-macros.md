
## Project: Spring-Style Dependency Injection Framework with Procedural Macros

### Problem Statement

Build a Spring-Boot-inspired web framework using procedural macros for dependency injection, HTTP routing, and request handling. You'll implement annotations like `#[component]`, `#[inject]`, `#[get]`, and `#[post]` that generate boilerplate code at compile-time, creating a framework where developers write minimal code to build REST APIs.

### Use Cases

**When you need this pattern**:
1. **Web frameworks**: Building REST APIs with automatic routing
2. **Dependency injection**: Managing service dependencies declaratively
3. **Annotation-based programming**: Java/Spring-style development in Rust
4. **Framework development**: Understanding how Axum, Actix-web, Rocket work
5. **Code generation**: Reducing boilerplate with compile-time macros
6. **Enterprise applications**: Large apps with many interconnected services

### Why It Matters

**Real-World Impact**: Annotation-based frameworks are the foundation of modern web development:

**The Boilerplate Problem**:
- **Manual wiring**: Creating services manually is error-prone and verbose
- **Route registration**: Hand-written route handlers require repetitive setup code
- **Parameter extraction**: Parsing HTTP requests manually is tedious
- **Testing**: Mocking dependencies requires manual setup

**Procedural Macros Solution**:
```rust
// Without macros - verbose manual setup
struct UserService {
    repo: UserRepository,
}

impl UserService {
    fn new() -> Self {
        let repo = ServiceRegistry::get::<UserRepository>()
            .expect("UserRepository not found");
        UserService { repo }
    }
}

fn main() {
    let mut registry = ServiceRegistry::new();
    registry.register(UserRepository::new());
    registry.register(UserService::new());

    let mut router = Router::new();
    router.add_route(Method::GET, "/users/:id", |req| {
        let id = req.param("id").parse::<u32>().unwrap();
        let service = ServiceRegistry::get::<UserService>();
        // ... more manual extraction
    });
}
```

```rust
// With macros - declarative and clean
#[component]
struct UserService {
    #[inject]
    repo: UserRepository,
}

#[controller("/api/users")]
impl UserController {
    #[get("/{id}")]
    async fn get_user(&self, id: PathParam<u32>) -> Json<User> {
        // Framework handles everything automatically
    }
}
```

**Performance Benefits**:
- **Zero runtime overhead**: All code generation at compile-time
- **Type safety**: Compile-time validation of routes and dependencies
- **No reflection**: Unlike Java, no runtime reflection penalty
- **Optimal code**: Generated code as efficient as hand-written

**Why Procedural Macros are Critical**:
- Annotations require modifying item definitions (functions, structs)
- Declarative macros (`macro_rules!`) can't do this
- Proc macros parse and generate Rust syntax at compile-time
- Enable framework-like APIs in a systems language

---

### Core Concepts

Before diving into the project, let's understand the key concepts that make this framework possible:

#### 1. Procedural Macros

**What are they?**
Procedural macros are Rust's most powerful metaprogramming feature. Unlike declarative macros (`macro_rules!`), procedural macros can:
- Parse Rust code as an abstract syntax tree (AST)
- Analyze and transform code structures
- Generate entirely new code based on the input
- Run arbitrary Rust code during compilation

**Types of Procedural Macros**:
- **Attribute macros**: `#[component]`, `#[get("/path")]` - Attach to items and transform them
- **Derive macros**: `#[derive(Serialize)]` - Automatically implement traits
- **Function-like macros**: `sql!("SELECT * FROM users")` - Look like function calls but run at compile-time

**How they work**:
```rust
// Input (what you write)
#[component]
struct UserService {
    #[inject]
    repo: UserRepository,
}

// Output (what the macro generates)
struct UserService {
    repo: UserRepository,
}

impl UserService {
    fn new() -> Self {
        let repo = ServiceRegistry::global().get::<UserRepository>();
        Self { repo }
    }
}

// Auto-registration code
inventory::submit! {
    ComponentRegistration::new::<UserService>()
}
```

#### 2. The `syn` Crate - Parsing Rust Syntax

**Purpose**: `syn` is the standard library for parsing Rust code in procedural macros.

**Key capabilities**:
- Parse Rust tokens into typed syntax trees
- Provides types for every Rust construct (structs, enums, functions, etc.)
- Handles all Rust syntax, including attributes, generics, lifetimes
- Type-safe API prevents generating invalid Rust code

**Example usage**:
```rust
use syn::{parse_macro_input, ItemStruct, Field};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse input as a struct definition
    let input = parse_macro_input!(item as ItemStruct);

    // Extract struct name
    let name = &input.ident;

    // Access fields
    if let Fields::Named(fields) = &input.fields {
        for field in &fields.named {
            // Check attributes on each field
            for attr in &field.attrs {
                if attr.path().is_ident("inject") {
                    // Found #[inject] attribute
                }
            }
        }
    }
}
```

#### 3. The `quote` Crate - Generating Code

**Purpose**: `quote!` provides an ergonomic way to generate Rust code.

**Key features**:
- Template syntax for code generation
- Interpolation with `#variable` syntax
- Repeating patterns with `#()*`
- Generates `proc_macro2::TokenStream` (convertible to `TokenStream`)

**Example usage**:
```rust
use quote::quote;

let field_name = &field.ident;
let field_type = &field.ty;

let generated = quote! {
    impl MyStruct {
        fn new() -> Self {
            let #field_name = ServiceRegistry::global().get::<#field_type>();
            Self { #field_name }
        }
    }
};
```

**Repeating patterns**:
```rust
let field_names = vec![field1, field2, field3];

quote! {
    Self {
        #(#field_names),*  // Expands to: field1, field2, field3
    }
}
```

#### 4. Dependency Injection (DI)

**What is it?**
Dependency Injection is a design pattern where objects receive their dependencies from external sources rather than creating them internally.

**Without DI**:
```rust
struct UserService {
    repo: UserRepository,
}

impl UserService {
    fn new() -> Self {
        Self {
            repo: UserRepository::new(),  // Hard-coded dependency
        }
    }
}
```

**With DI**:
```rust
struct UserService {
    repo: UserRepository,
}

impl UserService {
    fn new(repo: UserRepository) -> Self {  // Injected dependency
        Self { repo }
    }
}
```

**Benefits**:
- **Testability**: Easy to inject mock dependencies
- **Loose coupling**: Services don't know how dependencies are created
- **Configuration**: Dependencies can be swapped without changing code
- **Lifecycle management**: Framework controls object creation and lifecycle

**DI Container**:
A service registry that:
- Stores registered service instances
- Resolves dependencies automatically
- Ensures singleton behavior (one instance per type)
- Manages service lifecycle

#### 5. Type-Safe Storage with `TypeId` and `Any`

**The Problem**: How do you store different types in the same container?

**The Solution**: Use `TypeId` as keys and `Box<dyn Any>` for values.

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;

struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any>>,
}

impl ServiceRegistry {
    fn register<T: 'static>(&mut self, service: T) {
        let type_id = TypeId::of::<T>();
        self.services.insert(type_id, Box::new(service));
    }

    fn get<T: 'static>(&self) -> &T {
        let type_id = TypeId::of::<T>();
        self.services.get(&type_id)
            .expect("Service not found")
            .downcast_ref::<T>()
            .unwrap()
    }
}
```

**How it works**:
- `TypeId::of::<T>()` generates a unique ID for each type at compile-time
- `Box<dyn Any>` can hold any type, erasing its concrete type
- `downcast_ref::<T>()` safely casts back to the original type
- Type safety: You can only retrieve the type you registered

#### 6. HTTP Routing and Parameter Extraction

**Path Patterns**:
Routes like `/users/{id}/posts/{post_id}` need to:
1. Match incoming request paths
2. Extract parameters (`id`, `post_id`)
3. Parse parameters to correct types (`u32`, `String`, etc.)

**Parameter Sources**:
- **Path parameters**: `/users/{id}` → `id: PathParam<u32>`
- **Query parameters**: `/search?q=rust&limit=10` → `q: String, limit: u32`
- **Request body**: JSON payload → `user: Json<User>`
- **Headers**: `Authorization: Bearer token` → `token: String`

**Type-Safe Extraction**:
```rust
#[get("/users/{id}")]
fn get_user(
    id: PathParam<u32>,           // From path
    #[query] search: SearchQuery,  // From query string
    #[header("Authorization")] token: String,  // From headers
) -> Json<User> {
    // All parameters automatically extracted and parsed
}
```

#### 7. Compile-Time Code Generation

**The Power**: All framework code is generated at compile-time, resulting in:

**Zero Runtime Overhead**:
- No reflection or dynamic dispatch
- No runtime parsing or configuration
- Generated code is as fast as hand-written code
- All type checking happens at compile-time

**Compile-Time Safety**:
```rust
#[get("/users/{id}")]
fn get_user(id: PathParam<u32>) -> Json<User> {
    // If parameter types don't match, compilation fails
    // If route pattern is invalid, compilation fails
    // If dependencies are missing, compilation fails
}
```

**What gets generated**:
1. Dependency resolution code (constructor generation)
2. Route registration code (adding routes to global router)
3. Parameter extraction code (parsing path/query/body/headers)
4. Type conversions (JSON serialization/deserialization)
5. Error handling (missing services, parse errors)

### Connection to This Project

Now that we understand the core concepts, let's see how they all come together in this Spring-style framework:

**1. Procedural Macros as the Foundation**

This project builds three types of macros:
- **`#[component]`**: Transforms structs into managed services with automatic dependency resolution
- **`#[get]`, `#[post]`, etc.**: Transforms functions into HTTP route handlers with parameter extraction
- **`#[controller]`**: Groups related routes and applies dependency injection to controller classes

Each macro uses `syn` to parse the input code and `quote` to generate boilerplate code that you would otherwise write manually.

**2. Dependency Injection Container**

The `ServiceRegistry` demonstrates:
- Using `TypeId` and `Box<dyn Any>` for type-safe heterogeneous storage
- Thread-safe global state with `lazy_static` and `RwLock`
- Automatic dependency resolution in generated constructors
- The `#[inject]` attribute marks fields that should be resolved from the registry

**3. HTTP Routing System**

The routing implementation shows:
- Pattern matching: Converting `/users/{id}` into path parameter extraction
- Method dispatch: Routing based on HTTP method (GET, POST, etc.)
- Parameter extraction: Automatically parsing path params, query strings, headers, and JSON bodies
- Type safety: All parameters are parsed to the correct types at compile-time

**4. Code Generation Strategy**

Each milestone generates increasingly sophisticated code:
- **Milestone 1**: Simple trait implementations and registration
- **Milestone 2**: Constructor generation with dependency resolution
- **Milestone 3**: Route registration and handler wrappers
- **Milestone 4**: Complex parameter extraction with multiple sources
- **Milestone 5**: Complete application bootstrap with component scanning

**5. Framework Design Patterns**

This project demonstrates how modern web frameworks work:
- **Axum**: Uses similar macro-based routing and parameter extraction
- **Rocket**: Pioneered compile-time route validation in Rust
- **Actix-web**: Uses attributes for routes and middleware
- **Spring Boot**: The inspiration for annotation-based configuration

**6. Why This Matters**

By building this framework, you'll understand:
- How frameworks eliminate boilerplate through code generation
- Why Rust's proc macros are powerful yet zero-cost
- How to design extensible, type-safe APIs
- The trade-offs between runtime flexibility and compile-time safety
- How dependency injection works under the hood

**What You'll Build**:
A complete framework where this minimal code:
```rust
#[component]
struct UserService {
    #[inject]
    repo: UserRepository,
}

#[controller("/api/users")]
impl UserController {
    #[get("/{id}")]
    async fn get_user(&self, id: PathParam<u32>) -> Json<User> {
        // Implementation
    }
}
```

Generates hundreds of lines of boilerplate including:
- Service registration and retrieval
- Dependency resolution constructors
- Route parsing and registration
- HTTP parameter extraction
- JSON serialization/deserialization
- Error handling and type conversions

All validated at compile-time with zero runtime overhead!

---

### Learning Goals

By completing this project, you will:

1. **Master procedural macros**: Write attribute macros and derive macros
2. **Parse Rust syntax**: Use `syn` crate to parse complex code structures
3. **Generate code**: Use `quote!` macro to emit clean Rust code
4. **Build frameworks**: Understand how Axum, Rocket, and Actix-web work internally
5. **Dependency injection**: Implement a type-safe service container
6. **HTTP routing**: Build path matching and parameter extraction
7. **Trait design**: Create extensible framework APIs

---

### Project Structure

This project requires **two crates**:
1. **`myframework-macros`**: Procedural macro definitions (proc-macro crate)
2. **`myframework`**: Runtime support and core framework (normal library crate)

```
myframework/
├── myframework-macros/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── myframework/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── container.rs
│       ├── routing.rs
│       └── http.rs
└── examples/
    └── user_api.rs
```

---

### Milestone 1: Basic DI Container with `#[component]`

**Goal**: Implement dependency injection container with service registration.

**Implementation Steps**:

1. **Create the proc-macro crate structure**:
   - Create `myframework-macros/Cargo.toml` with `proc-macro = true`
   - Add dependencies: `syn = "2.0"`, `quote = "1.0"`, `proc-macro2 = "1.0"`
   - Set up lib.rs with `#[proc_macro_attribute]`

2. **Create the runtime crate**:
   - Create `myframework/Cargo.toml`
   - Depend on `myframework-macros`
   - Add `lazy_static = "1.4"` for global registry

3. **Implement `ServiceRegistry`**:
   - Use `HashMap<TypeId, Box<dyn Any>>` to store services
   - Implement `register<T>()` and `get<T>()` methods
   - Make it thread-safe with `RwLock`
   - Create global `REGISTRY` using `lazy_static`

4. **Implement `#[component]` macro**:
   - Parse struct definition using `syn::parse_macro_input`
   - Generate `Component` trait implementation
   - Generate registration code in `inventory` pattern
   - Emit original struct plus generated code

5. **Test manual registration**:
   - Create test services without `#[inject]` yet
   - Manually retrieve from registry
   - Verify type safety and panic on missing services


**Starter Code**:

```rust
// myframework-macros/Cargo.toml
[package]
name = "myframework-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full"] }
quote = "1.0"
proc-macro2 = "1.0"
```

**Checkpoint Tests**:
```rust
use myframework::*;

#[component]
struct DatabaseConnection {
    url: String,
}

impl DatabaseConnection {
    fn new() -> Self {
        Self { url: "localhost".to_string() }
    }
}

#[component]
struct UserRepository {
    // No dependencies yet
}

impl UserRepository {
    fn new() -> Self {
        Self {}
    }
}

#[test]
fn test_component_registration() {
    let db = DatabaseConnection::new();
    ServiceRegistry::global().register(db);

    let retrieved = ServiceRegistry::global().get::<DatabaseConnection>();
    assert_eq!(retrieved.url, "localhost");
}

#[test]
fn test_multiple_components() {
    ServiceRegistry::global().register(DatabaseConnection::new());
    ServiceRegistry::global().register(UserRepository::new());

    let db = ServiceRegistry::global().get::<DatabaseConnection>();
    let repo = ServiceRegistry::global().get::<UserRepository>();

    // Both exist
    assert!(db.url.len() > 0);
}

#[test]
#[should_panic(expected = "Service not found")]
fn test_missing_component() {
    // Clear registry
    ServiceRegistry::global().get::<String>(); // Should panic
}
```

```rust
// myframework-macros/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

/// Marks a struct as a managed component in the DI container
#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // TODO: Generate Component trait implementation
    // TODO: Generate automatic registration code
    // Hint: Use inventory crate or lazy_static for auto-registration

    let expanded = quote! {
        #input

        // TODO: Implement Component trait
        // impl Component for #name {
        //     fn register_self() {
        //         // Auto-register in global registry
        //     }
        // }
    };

    TokenStream::from(expanded)
}
```

```rust
// myframework/Cargo.toml
[package]
name = "myframework"
version = "0.1.0"
edition = "2021"

[dependencies]
myframework-macros = { path = "../myframework-macros" }
lazy_static = "1.4"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```rust
// myframework/src/lib.rs
pub use myframework_macros::*;

mod container;
pub use container::*;

// Re-export for user convenience
pub use lazy_static::lazy_static;
```

```rust
// myframework/src/container.rs
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::RwLock;

/// Global service registry for dependency injection
pub struct ServiceRegistry {
    services: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl ServiceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        // TODO: Initialize empty HashMap wrapped in RwLock
        todo!()
    }

    /// Register a service instance
    pub fn register<T: 'static + Send + Sync>(&self, service: T) {
        // TODO: Get TypeId for T
        // TODO: Box the service as dyn Any
        // TODO: Insert into services map
        // Hint: self.services.write().unwrap().insert(...)
        todo!()
    }

    /// Retrieve a service by type
    pub fn get<T: 'static>(&self) -> &T {
        // TODO: Get TypeId for T
        // TODO: Lookup in services map
        // TODO: Downcast Box<dyn Any> to &T
        // TODO: Panic with helpful message if not found
        // Hint: services.get(&type_id).expect("Service not found")
        // Hint: .downcast_ref::<T>().unwrap()
        todo!()
    }

    /// Get the global registry instance
    pub fn global() -> &'static ServiceRegistry {
        // TODO: Use lazy_static to create global instance
        // Hint: Already implemented below
        &GLOBAL_REGISTRY
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: ServiceRegistry = ServiceRegistry::new();
}

/// Trait for components that can be registered
pub trait Component {
    fn register_self();
}
```

**Check Your Understanding**:
- Why do we need `Box<dyn Any>` instead of generics for the registry?
- What is `TypeId` and how does it enable type-safe retrieval?
- Why must services be `'static + Send + Sync`?
- How does `lazy_static` ensure thread-safe initialization?

---

### Milestone 2: Constructor Injection with `#[inject]`

**Goal**: Auto-generate constructors that resolve dependencies from registry.

**Implementation Steps**:

1. **Parse `#[inject]` field attributes**:
   - Iterate through struct fields in macro
   - Identify fields marked with `#[inject]`
   - Extract field names and types

2. **Generate `new()` constructor**:
   - For each `#[inject]` field, call `ServiceRegistry::global().get::<Type>()`
   - Generate constructor that assembles struct from dependencies
   - Handle fields without `#[inject]` (require in `new()` parameters)

3. **Implement dependency validation**:
   - At compile-time: Generate code that will panic if missing
   - At runtime: Provide helpful error messages with dependency chain
   - Detect circular dependencies (bonus: compile-time check)

4. **Test automatic injection**:
   - Create services with injected dependencies
   - Verify automatic resolution
   - Test error messages for missing dependencies


**Starter Code Extension**:

```rust
// myframework-macros/src/lib.rs (updated)
use syn::{Field, Fields};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    // TODO: Extract fields from struct
    // TODO: Find fields with #[inject] attribute
    // TODO: Generate new() method that calls ServiceRegistry::get() for each injected field

    let injected_fields = extract_injected_fields(&input.fields);

    // TODO: Generate constructor
    let constructor = generate_constructor(&name, &injected_fields);

    let expanded = quote! {
        #input

        impl #name {
            #constructor
        }
    };

    TokenStream::from(expanded)
}

fn extract_injected_fields(fields: &Fields) -> Vec<&Field> {
    // TODO: Iterate through fields
    // TODO: Check each field for #[inject] attribute
    // TODO: Return list of injected fields
    // Hint: field.attrs.iter().any(|attr| attr.path().is_ident("inject"))
    todo!()
}

fn generate_constructor(name: &syn::Ident, injected_fields: &[&Field]) -> proc_macro2::TokenStream {
    // TODO: For each injected field, generate:
    // let field_name = ServiceRegistry::global().get::<FieldType>();

    // TODO: Generate struct construction:
    // Self { field1, field2, ... }

    // Hint: Use quote! macro
    todo!()
}
```
**Checkpoint Tests**:
```rust
use myframework::*;

#[component]
struct DatabaseConnection {
    url: String,
}

impl DatabaseConnection {
    fn new() -> Self {
        Self { url: "localhost".to_string() }
    }
}

#[component]
struct UserRepository {
    #[inject]
    db: DatabaseConnection,
}

#[component]
struct UserService {
    #[inject]
    repo: UserRepository,

    #[inject]
    db: DatabaseConnection,
}

#[test]
fn test_inject_single_dependency() {
    ServiceRegistry::global().register(DatabaseConnection::new());

    // Should auto-resolve db dependency
    let repo = UserRepository::new();
    assert_eq!(repo.db.url, "localhost");
}

#[test]
fn test_inject_multiple_dependencies() {
    ServiceRegistry::global().register(DatabaseConnection::new());
    ServiceRegistry::global().register(UserRepository::new());

    let service = UserService::new();
    assert_eq!(service.db.url, "localhost");
    assert_eq!(service.repo.db.url, "localhost");
}

#[test]
fn test_inject_chain() {
    // Register in any order - should resolve dependencies
    ServiceRegistry::global().register(DatabaseConnection::new());
    let repo = UserRepository::new();
    ServiceRegistry::global().register(repo);

    let service = UserService::new();
    // All dependencies resolved
}
```


**Check Your Understanding**:
- How do we detect the `#[inject]` attribute on fields?
- What happens if a dependency is not registered when `new()` is called?
- Could we detect circular dependencies at compile-time? How?
- Why generate `new()` instead of implementing `Default`?

---

### Milestone 3: Web Routing with `#[get]`, `#[post]`

**Goal**: Implement HTTP method macros that register route handlers.

**Implementation Steps**:

1. **Create HTTP types**:
   - `Request` struct with method, path, headers, body
   - `Response` struct with status, headers, body
   - `Method` enum (GET, POST, PUT, DELETE)
   - `StatusCode` enum (200, 404, 500, etc.)

2. **Build Router**:
   - `Router` struct with route table
   - Path matching with parameter extraction (e.g., `/users/{id}`)
   - Method-based dispatch
   - Handler trait: `Fn(Request) -> Response`

3. **Implement `#[get]` macro**:
   - Parse path pattern from attribute: `#[get("/users/{id}")]`
   - Extract path parameters from pattern
   - Parse function signature
   - Generate wrapper that extracts parameters and calls original function
   - Register route in global router

4. **Implement `#[post]`, `#[put]`, `#[delete]` similarly**

5. **Path parameter extraction**:
   - Parse `/users/{id}/posts/{post_id}` pattern
   - Generate code to extract and parse parameters
   - Match to function parameters by name


**Starter Code**:

```rust
// myframework/src/http.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    OK = 200,
    CREATED = 201,
    NOT_FOUND = 404,
    INTERNAL_SERVER_ERROR = 500,
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub fn get(path: &str) -> Self {
        // TODO: Create GET request
        todo!()
    }

    pub fn post(path: &str) -> Self {
        // TODO: Create POST request
        todo!()
    }
}

pub struct Response {
    pub status: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn ok(body: impl Into<String>) -> Self {
        // TODO: Create 200 OK response
        todo!()
    }

    pub fn created(body: impl Into<String>) -> Self {
        // TODO: Create 201 Created response
        todo!()
    }

    pub fn not_found() -> Self {
        // TODO: Create 404 Not Found response
        todo!()
    }
}

/// Wrapper for path parameters
pub struct PathParam<T>(pub T);
```

```rust
// myframework/src/routing.rs
use crate::http::*;
use std::collections::HashMap;
use std::sync::RwLock;

type Handler = Box<dyn Fn(Request) -> Response + Send + Sync>;

pub struct Route {
    pub method: Method,
    pub pattern: String,
    pub handler: Handler,
}

pub struct Router {
    routes: RwLock<Vec<Route>>,
}

impl Router {
    pub fn new() -> Self {
        // TODO: Initialize with empty routes
        todo!()
    }

    pub fn add_route(&self, method: Method, pattern: &str, handler: Handler) {
        // TODO: Add route to routes list
        // Hint: routes.write().unwrap().push(...)
        todo!()
    }

    pub fn route(&self, req: Request) -> Response {
        // TODO: Find matching route by method and path
        // TODO: Extract path parameters
        // TODO: Call handler
        // TODO: Return 404 if no match
        // Hint: Iterate through routes, match pattern with regex
        todo!()
    }

    pub fn global() -> &'static Router {
        &GLOBAL_ROUTER
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_ROUTER: Router = Router::new();
}

/// Match path pattern like "/users/{id}" against actual path "/users/42"
/// Returns Some(params) if match, None otherwise
pub fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    // TODO: Split pattern and path by '/'
    // TODO: Match segment by segment
    // TODO: Extract {param} segments as parameters
    // TODO: Return map of param_name -> value
    // Example: pattern="/users/{id}", path="/users/42" → Some({"id": "42"})
    todo!()
}
```

```rust
// myframework-macros/src/lib.rs (add route macros)

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro(Method::GET, attr, item)
}

#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    route_macro(Method::POST, attr, item)
}

fn route_macro(method: Method, attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_pattern = parse_macro_input!(attr as syn::LitStr).value();
    let func = parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;

    // TODO: Extract path parameters from pattern (e.g., {id}, {post_id})
    // TODO: Parse function parameters
    // TODO: Generate wrapper function that:
    //       1. Extracts path params from request
    //       2. Parses them to appropriate types
    //       3. Calls original function
    //       4. Returns response

    // TODO: Generate registration code:
    // Router::global().add_route(Method::GET, pattern, Box::new(wrapper));

    let expanded = quote! {
        #func

        // TODO: Generate registration code using inventory or ctor crate
        // Or use lazy_static with Once for registration
    };

    TokenStream::from(expanded)
}
```

**Checkpoint Tests**:
```rust
use myframework::*;

#[get("/hello")]
fn hello_world() -> Response {
    Response::ok("Hello, World!")
}

#[get("/users/{id}")]
fn get_user(id: PathParam<u32>) -> Response {
    Response::ok(format!("User ID: {}", id.0))
}

#[post("/users")]
fn create_user() -> Response {
    Response::created("User created")
}

#[get("/users/{user_id}/posts/{post_id}")]
fn get_post(user_id: PathParam<u32>, post_id: PathParam<u32>) -> Response {
    Response::ok(format!("User {} Post {}", user_id.0, post_id.0))
}

#[test]
fn test_route_registration() {
    let router = Router::global();

    // Routes should be registered automatically
    let req = Request::get("/hello");
    let resp = router.route(req);
    assert_eq!(resp.status, StatusCode::OK);
    assert_eq!(resp.body, "Hello, World!");
}

#[test]
fn test_path_parameters() {
    let router = Router::global();

    let req = Request::get("/users/42");
    let resp = router.route(req);
    assert_eq!(resp.body, "User ID: 42");
}

#[test]
fn test_multiple_parameters() {
    let router = Router::global();

    let req = Request::get("/users/1/posts/99");
    let resp = router.route(req);
    assert_eq!(resp.body, "User 1 Post 99");
}

#[test]
fn test_method_matching() {
    let router = Router::global();

    let req = Request::post("/users");
    let resp = router.route(req);
    assert_eq!(resp.status, StatusCode::CREATED);
}

#[test]
fn test_not_found() {
    let router = Router::global();

    let req = Request::get("/nonexistent");
    let resp = router.route(req);
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}
```

**Check Your Understanding**:
- How do we extract `{id}` from the path pattern?
- Why use a global `Router` instead of passing it around?
- What's the type of the `handler` function?
- How would we handle regex patterns instead of just `{param}`?

---

### Milestone 4: Request/Response Handling with `#[body]`, `#[query]`

**Goal**: Implement parameter extraction attributes for JSON bodies, query params, headers.

**Implementation Steps**:

1. **Implement `Json<T>` wrapper**:
   - Generic wrapper for JSON serialization/deserialization
   - Use serde for automatic conversion
   - Implement `From<Json<T>> for Response`

2. **Implement `#[body]` attribute**:
   - Mark function parameter for body deserialization
   - Generate code: `let param = serde_json::from_slice(&req.body)?;`
   - Handle errors gracefully (return 400 Bad Request)

3. **Implement `#[query]` attribute**:
   - Parse query string: `?name=John&age=30`
   - Deserialize into struct using serde
   - Support individual parameters or struct

4. **Implement `#[header]` attribute**:
   - Extract specific header by name
   - Return as String or Option<String>

5. **Error handling**:
   - Return appropriate status codes for parse errors
   - Provide error details in response body


**Starter Code Extension**:

```rust
// myframework/src/http.rs (additions)
use serde::{Serialize, Deserialize};

/// JSON wrapper for automatic serialization/deserialization
pub struct Json<T>(pub T);

impl<T: Serialize> From<Json<T>> for Response {
    fn from(json: Json<T>) -> Self {
        // TODO: Serialize T to JSON string
        // TODO: Create response with application/json content-type
        todo!()
    }
}

impl Request {
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn query_params(&self) -> HashMap<String, String> {
        // TODO: Parse query string from path
        // Example: "/search?q=rust&limit=10" → {"q": "rust", "limit": "10"}
        todo!()
    }
}
```

```rust
// myframework-macros/src/lib.rs (parameter extraction)

/// Attribute for marking function parameter as request body
#[proc_macro_attribute]
pub fn body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This is applied to individual function parameters
    // The actual work happens in the route macro which sees all parameters
    // Just pass through for now
    item
}

// Similar for query and header
#[proc_macro_attribute]
pub fn query(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn header(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// Update route_macro to handle parameter attributes:
fn route_macro(method: Method, attr: TokenStream, item: TokenStream) -> TokenStream {
    let path_pattern = parse_macro_input!(attr as syn::LitStr).value();
    let func = parse_macro_input!(item as syn::ItemFn);

    // TODO: For each function parameter, check for attributes:
    // - #[body]: deserialize from request.body
    // - #[query]: deserialize from request.query_params()
    // - #[header("Name")]: extract from request.headers
    // - PathParam<T>: extract from path parameters

    // TODO: Generate wrapper that extracts each parameter type

    // Example generated code:
    // let param1 = serde_json::from_slice::<User>(&req.body)?;
    // let param2 = extract_path_param::<u32>(&req, "id")?;
    // let result = original_func(param1, param2);

    todo!()
}
```

**Checkpoint Tests**:
```rust
use myframework::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    email: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<u32>,
}

#[post("/users")]
fn create_user(#[body] user: Json<User>) -> Json<User> {
    // user is automatically deserialized
    Json(user.0)
}

#[get("/search")]
fn search(#[query] query: SearchQuery) -> Response {
    Response::ok(format!("Search: {} (limit: {:?})", query.q, query.limit))
}

#[get("/protected")]
fn protected(#[header("Authorization")] token: String) -> Response {
    Response::ok(format!("Token: {}", token))
}

#[get("/users/{id}")]
fn get_user_full(
    id: PathParam<u32>,
    #[query] query: SearchQuery,
    #[header("User-Agent")] agent: String,
) -> Json<User> {
    Json(User {
        name: format!("User {}", id.0),
        email: "user@example.com".to_string(),
    })
}

#[test]
fn test_json_body() {
    let router = Router::global();

    let user = User {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    };
    let body = serde_json::to_vec(&user).unwrap();

    let req = Request::post("/users").with_body(body);
    let resp = router.route(req);

    assert_eq!(resp.status, StatusCode::OK);
    let returned: User = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(returned.name, "Alice");
}

#[test]
fn test_query_params() {
    let router = Router::global();

    let req = Request::get("/search?q=rust&limit=10");
    let resp = router.route(req);

    assert!(resp.body.contains("rust"));
    assert!(resp.body.contains("10"));
}

#[test]
fn test_headers() {
    let router = Router::global();

    let req = Request::get("/protected")
        .with_header("Authorization", "Bearer token123");
    let resp = router.route(req);

    assert!(resp.body.contains("token123"));
}

#[test]
fn test_combined_parameters() {
    let router = Router::global();

    let req = Request::get("/users/42?q=test&limit=5")
        .with_header("User-Agent", "TestClient/1.0");
    let resp = router.route(req);

    assert_eq!(resp.status, StatusCode::OK);
}
```

**Check Your Understanding**:
- Why wrap types in `Json<T>` instead of using `T` directly?
- How do we parse query strings into structs?
- What happens if deserialization fails?
- How would we support multiple body formats (JSON, Form, XML)?

---

### Milestone 5: Integration with `#[controller]` and `#[main]`

**Goal**: Tie everything together with controller grouping and application bootstrap.

**Implementation Steps**:

1. **Implement `#[controller]` macro**:
   - Parse base path: `#[controller("/api/users")]`
   - Inject services into controller struct
   - Prepend base path to all route methods
   - Support both struct methods and standalone functions

2. **Implement `#[main]` macro**:
   - Scan and initialize all components
   - Build dependency graph
   - Start HTTP server (simple TCP listener or use existing crate)
   - Graceful shutdown handling

3. **Component scanning**:
   - Use `inventory` crate to collect all components
   - Initialize in dependency order
   - Detect circular dependencies

4. **Build complete example application**:
   - Multi-layer architecture (Controller → Service → Repository)
   - CRUD operations for a resource
   - Demonstrate all features working together


**Starter Code**:

```rust
// myframework-macros/src/lib.rs (controller macro)

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let base_path = parse_macro_input!(attr as syn::LitStr).value();
    let input = parse_macro_input!(item as ItemStruct);

    // TODO: Extract methods from impl blocks
    // TODO: For each method with route attribute (#[get], #[post]):
    //       - Prepend base_path to route path
    //       - Inject self parameter (controller instance)
    // TODO: Apply #[component] behavior for DI

    let expanded = quote! {
        #[component]
        #input

        // TODO: Generate route registrations for all methods
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as syn::ItemFn);
    let func_name = &func.sig.ident;

    // TODO: Generate application bootstrap code:
    // - Initialize component registry
    // - Scan and register all components
    // - Build dependency graph
    // - Start HTTP server
    // - Call user's main function

    let expanded = quote! {
        #[tokio::main]
        async fn #func_name() {
            // Initialize framework
            myframework::Application::init();

            // User code
            #func

            // Start server
            myframework::Application::run("127.0.0.1:8080").await;
        }
    };

    TokenStream::from(expanded)
}
```

```rust
// myframework/src/lib.rs (application bootstrap)

pub struct Application;

impl Application {
    /// Initialize the application (scan components, build DI graph)
    pub fn init() {
        // TODO: Scan all registered components
        // TODO: Build dependency graph
        // TODO: Initialize components in order
        // TODO: Detect circular dependencies
        println!("Initializing application...");
    }

    /// Run the HTTP server
    pub async fn run(addr: &str) {
        // TODO: Create TCP listener
        // TODO: Accept connections in loop
        // TODO: Parse HTTP requests
        // TODO: Route to handlers
        // TODO: Send responses

        println!("Server running on {}", addr);

        // Simple blocking version (no real async):
        // let listener = std::net::TcpListener::bind(addr).unwrap();
        // for stream in listener.incoming() {
        //     handle_connection(stream.unwrap());
        // }
    }
}
```

**Checkpoint Tests**:
```rust
use myframework::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: u32,
    name: String,
    email: String,
}

// Repository layer
#[component]
struct UserRepository {
    users: std::sync::Mutex<Vec<User>>,
}

impl UserRepository {
    fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(vec![
                User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
                User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
            ]),
        }
    }

    fn find_by_id(&self, id: u32) -> Option<User> {
        self.users.lock().unwrap()
            .iter()
            .find(|u| u.id == id)
            .cloned()
    }

    fn save(&self, user: User) -> User {
        let mut users = self.users.lock().unwrap();
        users.push(user.clone());
        user
    }
}

// Service layer
#[component]
struct UserService {
    #[inject]
    repo: UserRepository,
}

impl UserService {
    fn get_user(&self, id: u32) -> Option<User> {
        self.repo.find_by_id(id)
    }

    fn create_user(&self, user: User) -> User {
        self.repo.save(user)
    }
}

// Controller layer
#[controller("/api/users")]
#[component]
struct UserController {
    #[inject]
    service: UserService,
}

impl UserController {
    #[get("/{id}")]
    fn get(&self, id: PathParam<u32>) -> Result<Json<User>, StatusCode> {
        self.service.get_user(id.0)
            .map(Json)
            .ok_or(StatusCode::NOT_FOUND)
    }

    #[post("/")]
    fn create(&self, #[body] user: Json<User>) -> Json<User> {
        let created = self.service.create_user(user.0);
        Json(created)
    }

    #[get("/")]
    fn list(&self) -> Json<Vec<User>> {
        // Return all users
        Json(vec![])
    }
}

#[main]
async fn main() {
    println!("Server starting on http://localhost:8080");
    // Framework auto-starts server
}

#[test]
fn test_full_integration() {
    // Initialize framework
    Application::init();

    let router = Router::global();

    // Test GET
    let req = Request::get("/api/users/1");
    let resp = router.route(req);
    assert_eq!(resp.status, StatusCode::OK);

    let user: User = serde_json::from_str(&resp.body).unwrap();
    assert_eq!(user.name, "Alice");

    // Test POST
    let new_user = User {
        id: 3,
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
    };
    let body = serde_json::to_vec(&new_user).unwrap();
    let req = Request::post("/api/users/").with_body(body);
    let resp = router.route(req);
    assert_eq!(resp.status, StatusCode::OK);

    // Test 404
    let req = Request::get("/api/users/999");
    let resp = router.route(req);
    assert_eq!(resp.status, StatusCode::NOT_FOUND);
}
```



**Check Your Understanding**:
- How does `#[controller]` combine with `#[component]`?
- What order should components be initialized in?
- How would we detect circular dependencies?
- Why use `#[tokio::main]` in the generated code?

---

### Complete Project Summary

**What You Built**:
1. Procedural macros for dependency injection (`#[component]`, `#[inject]`)
2. HTTP routing macros (`#[get]`, `#[post]`, `#[controller]`)
3. Parameter extraction (`#[body]`, `#[query]`, `#[header]`, `PathParam`)
4. Service registry with type-safe dependency resolution
5. HTTP router with path matching and method dispatch
6. Application bootstrap with component scanning
7. Complete Spring-like framework in Rust

**Key Concepts Practiced**:
- Procedural macro development with `syn` and `quote`
- Dependency injection patterns
- HTTP routing and parameter extraction
- Type-safe service containers
- Code generation at compile-time
- Framework design principles

**Real-World Applications**:
- Understanding Axum, Rocket, Actix-web internals
- Building custom web frameworks
- Creating annotation-based APIs
- Enterprise application architecture in Rust

**Next Steps**:
- Add middleware support (`#[before]`, `#[after]`)
- Implement validation (`#[validate]`)
- Add OpenAPI/Swagger generation
- Support WebSocket routes
- Add security annotations (`#[authorized]`, `#[role]`)
- Implement request/response filters
- Add metrics and logging decorators

**Performance Considerations**:
- All code generation happens at compile-time (zero runtime overhead)
- Type-safe dependency resolution prevents runtime errors
- Generated code is as efficient as hand-written
- No reflection or dynamic dispatch (unlike Java/Spring)

This framework demonstrates how Rust's procedural macros enable framework-like APIs while maintaining zero-cost abstractions and compile-time safety!
