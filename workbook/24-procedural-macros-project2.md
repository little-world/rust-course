# Chapter 24: Procedural Macros - Project 2

## Project 2: Service Orchestration DSL - Infrastructure as Code

### Problem Statement

Build a service orchestration DSL using procedural macros that generates type-safe infrastructure definitions for deploying containers, LLM services, MCP servers, and system tools. The system should parse declarative service definitions, validate dependencies at compile-time, generate Docker Compose/Kubernetes manifests, implement health checks and auto-scaling, and provide a fluent API for infrastructure management. Unlike runtime orchestration tools, all validation happens at compile-time with zero-cost abstractions.

### Use Cases

- **Microservices deployment** - Define and deploy multi-container applications
- **AI/LLM infrastructure** - Orchestrate local and cloud LLM services with cost optimization
- **MCP server management** - Deploy Model Context Protocol servers for LLM tool access
- **DevOps automation** - Integrate system tools (git, kubectl, docker) with AI agents
- **CI/CD pipelines** - Automate build, test, deploy workflows
- **Multi-environment deployments** - Dev, staging, production with different configs
- **Container orchestration** - Docker Compose and Kubernetes manifest generation

### Why It Matters

**Infrastructure as Code Problem**: Traditional infrastructure tools (Docker Compose, Kubernetes YAML, Terraform) lack type safety. A typo in `depends_on: [databse]` instead of `database` only fails at runtime. Missing environment variables discovered in production. Port conflicts found after deployment.

**Macro Solution vs Runtime Tools**:
```yaml
# Docker Compose - Runtime errors
version: '3'
services:
  api:
    image: myapi:latest
    depends_on:
      - databse  # Typo! Runtime error
    environment:
      DB_HOST: ${DB_HOST}  # Missing var? Runtime error
    ports:
      - "8080:8080"
      - "8080:9000"  # Port conflict! Runtime error
```

```rust
// Orchestration DSL - Compile-time validation
deployment! {
    AppStack {
        services: {
            db: Database { password: env!("DB_PASSWORD") },

            api: ApiServer {
                port: 8080,
                depends_on: [databse],  // Compile error: databse not found
            },
        }
    }
}
// Missing DB_PASSWORD? Compile error!
// Port conflict? Compile error!
// Circular dependency? Compile error!
```

**Cost Optimization for LLMs**: Production AI apps spend thousands monthly on API calls. Routing simple queries to local LLM (free) vs complex to GPT-4 ($0.03/1K tokens) saves 70%+ on costs. Macro-generated router implements this at compile-time.

**Type-Safe MCP Integration**: MCP (Model Context Protocol) connects LLMs to tools. Manual integration is error-prone—wrong protocol, missing tools, capability mismatches. Generated code ensures type-safe tool registration.

**Zero Runtime Overhead**: All service definitions compile to constants. Docker spec generation happens once at startup, not per request. Health checks are compiled loops, not interpreted scripts.

Example cost comparison:
```
Manual LLM calls:     100% to GPT-4 = $1000/month
Smart routing DSL:    70% local LLM + 30% GPT-4 = $300/month
Savings: $700/month (70% reduction)
```

Performance comparison:
```
YAML parsing:         5-10ms per service definition
Compiled DSL:         0ms (compile-time generation)
Health check loop:    Compiled code (no interpretation overhead)
```

---

## Milestone 1: Basic Container Service Definition with `#[derive(Service)]`

### Introduction

Before building complex orchestration, understand how procedural macros parse struct definitions and generate container specifications. This milestone teaches derive macros for service definition and Docker/Kubernetes manifest generation.

**Why Start Here**: Container orchestration starts with individual service definitions. Learning to parse `#[container]` attributes, extract environment variables, and generate Docker specs is foundational.

### Architecture

**Macros:**
- `#[derive(Service)]` - Main derive macro for service definition
  - **Pattern**: Applied to struct definitions
  - **Expands to**: Impl blocks with container spec generation
  - **Role**: Core service abstraction

- `#[container(...)]` - Attribute macro for container configuration
  - **Pattern**: `#[container(image = "nginx:latest", port = 80)]`
  - **Expands to**: Metadata used by Service derive
  - **Role**: Container-specific settings

**Key Structs:**
- Service structs (user-defined)
  - **Fields**: Configuration parameters with `#[env]`, `#[volume]` attributes
  - **Methods**: Generated `to_docker_spec()`, `to_compose_yaml()`

- `DockerSpec` - Generated container specification
  - **Fields**: `image`, `ports`, `env`, `volumes`, `networks`
  - **Methods**: `to_compose_yaml()`, `to_kubernetes_yaml()`

### Checkpoint Tests

```rust
use orchestrate::*;

#[derive(Service)]
#[container(image = "nginx:latest", port = 80)]
struct WebServer {
    #[env]
    workers: u32,

    #[volume("/data")]
    data_dir: String,
}

#[derive(Service)]
#[container(image = "postgres:15", port = 5432)]
struct Database {
    #[env("POSTGRES_PASSWORD")]
    password: String,

    #[env("POSTGRES_USER")]
    user: String,

    #[volume("/var/lib/postgresql/data")]
    data_dir: String,
}

#[test]
fn test_docker_spec_generation() {
    let web = WebServer {
        workers: 4,
        data_dir: "/app/data".to_string(),
    };

    let spec = web.to_docker_spec();

    assert_eq!(spec.image, "nginx:latest");
    assert_eq!(spec.ports, vec![80]);
    assert_eq!(spec.env.get("workers"), Some(&"4".to_string()));
    assert!(spec.volumes.contains(&"/app/data:/data".to_string()));
}

#[test]
fn test_docker_compose_yaml() {
    let db = Database {
        password: "secret123".to_string(),
        user: "admin".to_string(),
        data_dir: "/data/postgres".to_string(),
    };

    let yaml = db.to_compose_yaml();

    assert!(yaml.contains("image: postgres:15"));
    assert!(yaml.contains("POSTGRES_PASSWORD: secret123"));
    assert!(yaml.contains("POSTGRES_USER: admin"));
    assert!(yaml.contains("ports:\n      - \"5432:5432\""));
}

#[test]
fn test_multiple_ports() {
    #[derive(Service)]
    #[container(image = "myapp:latest", ports = [8080, 9000])]
    struct MultiPortService {
        #[env]
        debug: bool,
    }

    let service = MultiPortService { debug: true };
    let spec = service.to_docker_spec();

    assert_eq!(spec.ports, vec![8080, 9000]);
}

#[test]
fn test_env_variable_mapping() {
    let db = Database {
        password: "test_pass".to_string(),
        user: "test_user".to_string(),
        data_dir: "/tmp/db".to_string(),
    };

    let spec = db.to_docker_spec();

    assert_eq!(spec.env.get("POSTGRES_PASSWORD"), Some(&"test_pass".to_string()));
    assert_eq!(spec.env.get("POSTGRES_USER"), Some(&"test_user".to_string()));
}

#[test]
fn test_volume_mounting() {
    let web = WebServer {
        workers: 2,
        data_dir: "/var/www".to_string(),
    };

    let spec = web.to_docker_spec();

    assert_eq!(spec.volumes.len(), 1);
    assert_eq!(spec.volumes[0], "/var/www:/data");
}

#[test]
fn test_kubernetes_deployment_yaml() {
    let web = WebServer {
        workers: 4,
        data_dir: "/app/data".to_string(),
    };

    let k8s_yaml = web.to_kubernetes_deployment();

    assert!(k8s_yaml.contains("apiVersion: apps/v1"));
    assert!(k8s_yaml.contains("kind: Deployment"));
    assert!(k8s_yaml.contains("image: nginx:latest"));
    assert!(k8s_yaml.contains("containerPort: 80"));
}
```

### Starter Code

```rust
// ================================================
// Crate structure: orchestrate-macros (proc-macro)
// ================================================

// orchestrate-macros/Cargo.toml
/*
[package]
name = "orchestrate-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2.0", features = ["full", "extra-traits"] }
quote = "1.0"
proc-macro2 = "1.0"
*/

// orchestrate-macros/examples/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Attribute};

#[proc_macro_derive(Service, attributes(container, env, volume, health_check))]
pub fn derive_service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // TODO: Parse #[container(...)] attribute
    // TODO: Extract image name and ports
    // TODO: Find fields with #[env] and #[volume] attributes
    // TODO: Generate DockerSpec creation code

    let container_config = parse_container_attr(&input.attrs);
    let env_fields = extract_env_fields(&input);
    let volume_fields = extract_volume_fields(&input);

    // TODO: Generate to_docker_spec() method
    let docker_spec_impl = generate_docker_spec_impl(
        name,
        &container_config,
        &env_fields,
        &volume_fields,
    );

    // TODO: Generate to_compose_yaml() method
    let compose_yaml_impl = generate_compose_yaml_impl(name);

    // TODO: Generate to_kubernetes_deployment() method
    let k8s_impl = generate_kubernetes_impl(name);

    let expanded = quote! {
        impl #name {
            #docker_spec_impl
            #compose_yaml_impl
            #k8s_impl
        }
    };

    TokenStream::from(expanded)
}

// TODO: Parse #[container(image = "...", port = ...)] attribute
fn parse_container_attr(attrs: &[Attribute]) -> ContainerConfig {
    // Hint: Look for attribute with path "container"
    // Parse as syn::MetaList
    // Extract image and port from nested meta items
    todo!("Parse container attributes")
}

struct ContainerConfig {
    image: String,
    ports: Vec<u16>,
}

// TODO: Find fields marked with #[env] or #[env("CUSTOM_NAME")]
fn extract_env_fields(input: &DeriveInput) -> Vec<EnvField> {
    // Hint: Match on input.data as Data::Struct
    // Iterate through fields
    // Check field.attrs for "env" attribute
    // Extract custom env var name if provided
    todo!("Extract env fields")
}

struct EnvField {
    field_name: syn::Ident,
    env_var_name: String,
}

// TODO: Find fields marked with #[volume("/path")]
fn extract_volume_fields(input: &DeriveInput) -> Vec<VolumeField> {
    // Similar to env fields
    todo!("Extract volume fields")
}

struct VolumeField {
    field_name: syn::Ident,
    container_path: String,
}

// TODO: Generate to_docker_spec() method implementation
fn generate_docker_spec_impl(
    name: &syn::Ident,
    config: &ContainerConfig,
    env_fields: &[EnvField],
    volume_fields: &[VolumeField],
) -> proc_macro2::TokenStream {
    let image = &config.image;
    let ports = &config.ports;

    // TODO: Generate code that creates DockerSpec
    // Hint: Build env map from env_fields
    // Build volumes vec from volume_fields

    quote! {
        pub fn to_docker_spec(&self) -> DockerSpec {
            // TODO: Implement spec creation
            todo!()
        }
    }
}

// TODO: Generate to_compose_yaml() method
fn generate_compose_yaml_impl(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn to_compose_yaml(&self) -> String {
            // TODO: Format as Docker Compose YAML
            // Use the DockerSpec and convert to YAML string
            todo!()
        }
    }
}

// TODO: Generate to_kubernetes_deployment() method
fn generate_kubernetes_impl(name: &syn::Ident) -> proc_macro2::TokenStream {
    quote! {
        pub fn to_kubernetes_deployment(&self) -> String {
            // TODO: Format as Kubernetes Deployment YAML
            todo!()
        }
    }
}
```

```rust
// ================================================
// Runtime crate: orchestrate
// ================================================

// orchestrate/Cargo.toml
/*
[package]
name = "orchestrate"
version = "0.1.0"
edition = "2021"

[dependencies]
orchestrate-macros = { path = "../orchestrate-macros" }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
tokio = { version = "1", features = ["full"] }
*/

// orchestrate/examples/lib.rs
pub use orchestrate_macros::*;

use std::collections::HashMap;

/// Docker container specification
#[derive(Debug, Clone)]
pub struct DockerSpec {
    pub image: String,
    pub ports: Vec<u16>,
    pub env: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub networks: Vec<String>,
}

impl DockerSpec {
    pub fn new(image: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            ports: Vec::new(),
            env: HashMap::new(),
            volumes: Vec::new(),
            networks: Vec::new(),
        }
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.ports.push(port);
        self
    }

    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub fn with_volume(mut self, volume: impl Into<String>) -> Self {
        self.volumes.push(volume.into());
        self
    }

    /// Convert to Docker Compose YAML format
    pub fn to_compose_yaml(&self) -> String {
        // TODO: Implement YAML serialization
        // Format:
        // services:
        //   service_name:
        //     image: ...
        //     ports: [...]
        //     environment: {...}
        //     volumes: [...]

        let mut yaml = String::new();
        yaml.push_str(&format!("    image: {}\n", self.image));

        if !self.ports.is_empty() {
            yaml.push_str("    ports:\n");
            for port in &self.ports {
                yaml.push_str(&format!("      - \"{}:{}\"\n", port, port));
            }
        }

        if !self.env.is_empty() {
            yaml.push_str("    environment:\n");
            for (key, value) in &self.env {
                yaml.push_str(&format!("      {}: {}\n", key, value));
            }
        }

        if !self.volumes.is_empty() {
            yaml.push_str("    volumes:\n");
            for volume in &self.volumes {
                yaml.push_str(&format!("      - {}\n", volume));
            }
        }

        yaml
    }

    /// Convert to Kubernetes Deployment YAML
    pub fn to_kubernetes_yaml(&self, name: &str, replicas: u32) -> String {
        // TODO: Implement Kubernetes YAML generation
        // Format: Deployment with spec

        format!(
            r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
spec:
  replicas: {}
  selector:
    matchLabels:
      app: {}
  template:
    metadata:
      labels:
        app: {}
    spec:
      containers:
      - name: {}
        image: {}
        ports:
{}
        env:
{}
"#,
            name,
            replicas,
            name,
            name,
            name,
            self.image,
            self.ports.iter()
                .map(|p| format!("        - containerPort: {}", p))
                .collect::<Vec<_>>()
                .join("\n"),
            self.env.iter()
                .map(|(k, v)| format!("        - name: {}\n          value: \"{}\"", k, v))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

/// Re-export for user convenience
pub use serde::{Serialize, Deserialize};
```

**Implementation Hints:**
1. Use `syn::Attribute::parse_meta()` to parse attribute arguments
2. For `#[container(image = "nginx")]`, parse as `MetaNameValue`
3. Field attributes: iterate `field.attrs` and check `attr.path().is_ident("env")`
4. For env variable names: default to field name uppercase, or use custom from attribute
5. Volume format: `"{host_path}:{container_path}"`
6. YAML indentation matters—use 2 or 4 spaces consistently

---

## Milestone 2: Multi-Service Orchestration with `deployment!` Macro

### Introduction

**Why Milestone 1 Isn't Enough**: Single services are useless—real applications have databases, caches, APIs, web servers working together. Need dependency management, network configuration, and coordinated startup.

**The Improvement**: Implement `deployment!` function-like macro that parses entire service stack, validates dependencies (topological sort), generates Docker Compose with networks, and creates Kubernetes manifests with Services and ConfigMaps.

**Optimization (Dependency Ordering)**: Starting API before database causes crash. Topological sort ensures correct startup order: database → cache → API → web. Compile-time validation prevents circular dependencies.

### Architecture

**New Macros:**
- `deployment!` - Function-like macro for stack definition
  - **Pattern**: `deployment! { StackName { services: {...}, networks: {...} } }`
  - **Expands to**: Struct with service instances and orchestration methods
  - **Role**: Main orchestration interface

**Key Structs:**
- Deployment struct (generated)
  - **Fields**: All service instances
  - **Methods**: `startup_order()`, `to_docker_compose()`, `to_kubernetes()`

- `DependencyGraph` - Internal representation
  - **Fields**: Services and their dependencies
  - **Methods**: `topological_sort()`, `detect_cycles()`

### Checkpoint Tests

```rust
use orchestrate::*;

#[derive(Service)]
#[container(image = "postgres:15", port = 5432)]
struct Database {
    #[env("POSTGRES_PASSWORD")]
    password: String,
}

#[derive(Service)]
#[container(image = "redis:7", port = 6379)]
struct Cache {
    #[env]
    max_memory: String,
}

#[derive(Service)]
#[container(image = "myapi:latest", port = 8080)]
struct ApiServer {
    #[env]
    port: u16,
}

#[derive(Service)]
#[container(image = "nginx:latest", port = 80)]
struct WebServer {
    #[env]
    backend_url: String,
}

deployment! {
    AppStack {
        services: {
            db: Database {
                password: "secret123",
            },

            cache: Cache {
                max_memory: "256mb",
            },

            api: ApiServer {
                port: 8080,
                depends_on: [db, cache],
            },

            web: WebServer {
                backend_url: "http://api:8080",
                depends_on: [api],
            },
        },

        networks: {
            backend: [db, cache, api],
            frontend: [api, web],
        }
    }
}

#[test]
fn test_dependency_ordering() {
    let stack = AppStack::new();
    let order = stack.startup_order();

    // db and cache have no dependencies, can start first
    assert!(order[0] == "db" || order[0] == "cache");
    assert!(order[1] == "db" || order[1] == "cache");

    // api depends on both, must start after
    assert_eq!(order[2], "api");

    // web depends on api, must start last
    assert_eq!(order[3], "web");
}

#[test]
fn test_docker_compose_generation() {
    let stack = AppStack::new();
    let compose = stack.to_docker_compose();

    assert!(compose.contains("version: '3.8'"));
    assert!(compose.contains("services:"));

    // Check all services present
    assert!(compose.contains("db:"));
    assert!(compose.contains("cache:"));
    assert!(compose.contains("api:"));
    assert!(compose.contains("web:"));

    // Check dependencies
    assert!(compose.contains("depends_on:\n      - db\n      - cache"));

    // Check networks
    assert!(compose.contains("networks:"));
    assert!(compose.contains("backend:"));
    assert!(compose.contains("frontend:"));
}

#[test]
fn test_network_configuration() {
    let stack = AppStack::new();
    let compose = stack.to_docker_compose();

    // db should be on backend network only
    let db_section = extract_service_section(&compose, "db");
    assert!(db_section.contains("networks:\n      - backend"));
    assert!(!db_section.contains("frontend"));

    // api should be on both networks
    let api_section = extract_service_section(&compose, "api");
    assert!(api_section.contains("backend"));
    assert!(api_section.contains("frontend"));
}

#[test]
fn test_kubernetes_manifests() {
    let stack = AppStack::new();
    let k8s = stack.to_kubernetes();

    // Should generate multiple YAML documents (---)
    assert!(k8s.matches("---").count() >= 4);

    // Check for Deployment resources
    assert!(k8s.contains("kind: Deployment"));

    // Check for Service resources (Kubernetes Services for networking)
    assert!(k8s.contains("kind: Service"));
}

#[test]
#[should_panic(expected = "Circular dependency detected")]
fn test_circular_dependency_detection() {
    deployment! {
        CircularStack {
            services: {
                a: Database {
                    password: "test",
                    depends_on: [b],
                },

                b: Cache {
                    max_memory: "128mb",
                    depends_on: [a],
                },
            }
        }
    }

    CircularStack::new(); // Should panic at compile or runtime
}

#[test]
fn test_isolated_networks() {
    let stack = AppStack::new();
    let networks = stack.network_topology();

    // db and cache should not be directly accessible from web
    assert!(!networks.can_communicate("web", "db"));
    assert!(!networks.can_communicate("web", "cache"));

    // But api can reach everyone
    assert!(networks.can_communicate("api", "db"));
    assert!(networks.can_communicate("api", "cache"));
}

// Helper function for testing
fn extract_service_section(compose: &str, service_name: &str) -> String {
    // Extract YAML section for specific service
    // Simple implementation for testing
    let start = compose.find(&format!("  {}:", service_name)).unwrap();
    let remaining = &compose[start..];
    let end = remaining.find("\n  ").unwrap_or(remaining.len());
    remaining[..end].to_string()
}
```

### Starter Code

```rust
// orchestrate-macros/examples/lib.rs (additions)

use syn::parse::{Parse, ParseStream};
use syn::{Ident, Token, braced};
use std::collections::HashMap;

/// Parse deployment! macro syntax
struct DeploymentDef {
    name: Ident,
    services: Vec<ServiceDef>,
    networks: Vec<NetworkDef>,
}

struct ServiceDef {
    name: Ident,
    service_type: Ident,
    fields: Vec<FieldInit>,
    depends_on: Vec<Ident>,
}

struct FieldInit {
    name: Ident,
    value: syn::Expr,
}

struct NetworkDef {
    name: Ident,
    services: Vec<Ident>,
}

impl Parse for DeploymentDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: Parse deployment syntax
        // deployment_name { services: { ... }, networks: { ... } }

        let name: Ident = input.parse()?;

        let content;
        braced!(content in input);

        // TODO: Parse services section
        // TODO: Parse networks section

        todo!("Parse deployment definition")
    }
}

#[proc_macro]
pub fn deployment(input: TokenStream) -> TokenStream {
    let deployment = parse_macro_input!(input as DeploymentDef);

    // TODO: Validate dependencies (no cycles)
    // TODO: Generate deployment struct
    // TODO: Generate service initialization
    // TODO: Generate startup_order() method
    // TODO: Generate to_docker_compose() method
    // TODO: Generate to_kubernetes() method

    let name = &deployment.name;
    let service_fields = generate_service_fields(&deployment.services);
    let startup_order = generate_startup_order(&deployment.services);
    let docker_compose = generate_docker_compose_method(&deployment);
    let kubernetes = generate_kubernetes_method(&deployment);

    let expanded = quote! {
        pub struct #name {
            #(#service_fields,)*
        }

        impl #name {
            pub fn new() -> Self {
                Self {
                    // TODO: Initialize services
                }
            }

            #startup_order
            #docker_compose
            #kubernetes
        }
    };

    TokenStream::from(expanded)
}

fn generate_service_fields(services: &[ServiceDef]) -> Vec<proc_macro2::TokenStream> {
    // TODO: Generate struct fields for each service
    // Format: pub service_name: ServiceType
    todo!()
}

fn generate_startup_order(services: &[ServiceDef]) -> proc_macro2::TokenStream {
    // TODO: Topological sort based on depends_on
    // Return method that provides startup order

    quote! {
        pub fn startup_order(&self) -> Vec<&'static str> {
            // TODO: Return topologically sorted service names
            todo!()
        }
    }
}

fn generate_docker_compose_method(deployment: &DeploymentDef) -> proc_macro2::TokenStream {
    // TODO: Generate method that creates Docker Compose YAML

    quote! {
        pub fn to_docker_compose(&self) -> String {
            let mut yaml = String::from("version: '3.8'\n\nservices:\n");

            // TODO: Add each service
            // TODO: Add depends_on
            // TODO: Add networks section

            yaml
        }
    }
}

fn generate_kubernetes_method(deployment: &DeploymentDef) -> proc_macro2::TokenStream {
    // TODO: Generate method that creates Kubernetes YAML

    quote! {
        pub fn to_kubernetes(&self) -> String {
            let mut yaml = String::new();

            // TODO: Generate Deployment for each service
            // TODO: Generate Service (K8s) for each exposed port
            // TODO: Generate ConfigMap for env variables

            yaml
        }
    }
}
```

```rust
// orchestrate/examples/lib.rs (additions)

use std::collections::{HashMap, HashSet};

/// Dependency graph for topological sorting
pub struct DependencyGraph {
    nodes: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: String, dependencies: Vec<String>) {
        self.nodes.insert(name, dependencies);
    }

    /// Topological sort - returns nodes in dependency order
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        // TODO: Implement Kahn's algorithm or DFS-based topological sort
        // Return Err if cycle detected

        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut temp_mark = HashSet::new();

        for node in self.nodes.keys() {
            if !visited.contains(node) {
                self.visit(node, &mut visited, &mut temp_mark, &mut result)?;
            }
        }

        result.reverse();
        Ok(result)
    }

    fn visit(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        temp_mark: &mut HashSet<String>,
        result: &mut Vec<String>,
    ) -> Result<(), String> {
        // TODO: DFS with cycle detection
        // temp_mark tracks current path (for cycle detection)

        if temp_mark.contains(node) {
            return Err(format!("Circular dependency detected: {}", node));
        }

        if visited.contains(node) {
            return Ok(());
        }

        temp_mark.insert(node.to_string());

        if let Some(deps) = self.nodes.get(node) {
            for dep in deps {
                self.visit(dep, visited, temp_mark, result)?;
            }
        }

        temp_mark.remove(node);
        visited.insert(node.to_string());
        result.push(node.to_string());

        Ok(())
    }
}

/// Network topology representation
pub struct NetworkTopology {
    networks: HashMap<String, HashSet<String>>,
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            networks: HashMap::new(),
        }
    }

    pub fn add_network(&mut self, name: String, services: Vec<String>) {
        self.networks.insert(name, services.into_iter().collect());
    }

    /// Check if two services can communicate (on same network)
    pub fn can_communicate(&self, service_a: &str, service_b: &str) -> bool {
        for network_services in self.networks.values() {
            if network_services.contains(service_a) && network_services.contains(service_b) {
                return true;
            }
        }
        false
    }
}
```

**Implementation Hints:**
1. Parse `services: { name: Type { fields }, ... }` using custom Parse impl
2. For `depends_on: [a, b]`, parse as array of identifiers
3. Topological sort: Kahn's algorithm or DFS with temp marks for cycle detection
4. Docker Compose YAML: indent with 2 spaces, use `depends_on:` key
5. Kubernetes: generate separate Deployment + Service resources for each service
6. Network isolation: services on same network can communicate

---

## Milestone 3: LLM Service Integration with `#[llm_service]`

### Introduction

**Why Milestone 2 Isn't Enough**: Modern applications need AI capabilities. Integrating LLMs (OpenAI, Anthropic, local models) manually is error-prone—API auth, token limits, streaming, error handling. Need type-safe LLM service definitions.

**The Improvement**: Add `#[llm_service]` attribute macro that generates LLM client code, handles authentication, implements token counting, provides streaming responses, and enables cost optimization through smart routing.

**Optimization (Cost)**: Production AI apps make millions of API calls. GPT-4 costs $0.03/1K tokens input, $0.06/1K output. Local LLMs (Llama, Mistral) free but slower. Smart router: simple queries → local (free), complex → cloud (accurate). This saves 60-80% on API costs.

### Architecture

**New Macros:**
- `#[llm_service]` - Attribute macro for LLM configuration
  - **Pattern**: `#[llm_service(model = "gpt-4", provider = "openai", ...)]`
  - **Expands to**: LLM client initialization and methods
  - **Role**: LLM-specific service type

**Key Structs:**
- LLM service structs
  - **Fields**: API keys, prompts, configuration with `#[api_key]`, `#[system_prompt]`
  - **Methods**: Generated `query()`, `stream()`, `count_tokens()`

- `LLMClient` - Internal client abstraction
  - **Fields**: Provider, model, config
  - **Methods**: `send_request()`, `stream_response()`

### Checkpoint Tests

```rust
use orchestrate::*;

#[derive(Service)]
#[llm_service(
    model = "gpt-4",
    provider = "openai",
    max_tokens = 4096,
    temperature = 0.7
)]
struct ChatAgent {
    #[api_key]
    openai_key: String,

    #[system_prompt]
    prompt: &'static str,
}

#[derive(Service)]
#[llm_service(
    model = "llama-3.1-70b",
    provider = "ollama",
    local = true
)]
struct LocalLLM {
    #[context_length]
    context: usize,

    #[gpu_layers]
    gpu: u32,
}

#[test]
fn test_llm_service_config() {
    let agent = ChatAgent {
        openai_key: "sk-test-key".to_string(),
        prompt: "You are a helpful assistant",
    };

    assert_eq!(agent.model(), "gpt-4");
    assert_eq!(agent.max_tokens(), 4096);
    assert_eq!(agent.temperature(), 0.7);
}

#[test]
async fn test_llm_query() {
    let agent = ChatAgent {
        openai_key: "sk-test-key".to_string(),
        prompt: "You are a helpful assistant",
    };

    // Mock response for testing
    let response = agent.query("What is 2+2?").await;

    // Response should have structure
    assert!(response.content.len() > 0);
    assert_eq!(response.model, "gpt-4");
}

#[test]
async fn test_token_counting() {
    let agent = ChatAgent {
        openai_key: "sk-test-key".to_string(),
        prompt: "You are a helpful assistant",
    };

    let prompt = "Hello, how are you?";
    let tokens = agent.count_tokens(prompt);

    // Simple heuristic: ~1 token per 4 characters
    assert!(tokens > 0);
    assert!(tokens < prompt.len());
}

#[test]
fn test_local_llm_config() {
    let local = LocalLLM {
        context: 8192,
        gpu: 33,  // All layers on GPU
    };

    assert_eq!(local.context_length(), 8192);
    assert_eq!(local.gpu_layers(), 33);
    assert!(local.is_local());
}

#[test]
async fn test_streaming_response() {
    let agent = ChatAgent {
        openai_key: "sk-test-key".to_string(),
        prompt: "You are a helpful assistant",
    };

    let mut stream = agent.stream("Tell me a story").await;

    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next().await {
        chunks.push(chunk);
    }

    assert!(chunks.len() > 0);
}

#[test]
fn test_cost_estimation() {
    let agent = ChatAgent {
        openai_key: "sk-test-key".to_string(),
        prompt: "You are a helpful assistant",
    };

    // Estimate cost for a query
    let prompt = "A".repeat(1000);  // 1000 chars ~ 250 tokens
    let estimated_cost = agent.estimate_cost(&prompt, 500);

    // GPT-4: $0.03 per 1K input tokens, $0.06 per 1K output
    // 250 input + 500 output = 750 tokens
    // Cost: (250 * 0.03 + 500 * 0.06) / 1000 = $0.0375
    assert!(estimated_cost > 0.03);
    assert!(estimated_cost < 0.05);
}
```

### Starter Code

```rust
// orchestrate-macros/examples/lib.rs (additions)

#[proc_macro_attribute]
pub fn llm_service(attr: TokenStream, item: TokenStream) -> TokenStream {
    let llm_config = parse_macro_input!(attr as LLMServiceConfig);
    let mut input = parse_macro_input!(item as DeriveInput);

    // TODO: Extract fields with #[api_key], #[system_prompt] attributes
    // TODO: Generate LLMClient initialization
    // TODO: Generate query() method
    // TODO: Generate stream() method
    // TODO: Generate count_tokens() method
    // TODO: Generate estimate_cost() method

    let name = &input.ident;
    let model = &llm_config.model;
    let provider = &llm_config.provider;

    let expanded = quote! {
        #[derive(Service)]
        #input

        impl #name {
            pub fn model(&self) -> &'static str {
                #model
            }

            pub fn provider(&self) -> &'static str {
                #provider
            }

            pub async fn query(&self, prompt: &str) -> LLMResponse {
                // TODO: Implement API call
                todo!()
            }

            pub async fn stream(&self, prompt: &str) -> LLMStream {
                // TODO: Implement streaming API call
                todo!()
            }

            pub fn count_tokens(&self, text: &str) -> usize {
                // TODO: Implement token counting (tiktoken or estimate)
                // Simple estimate: ~1 token per 4 characters
                (text.len() as f64 / 4.0).ceil() as usize
            }

            pub fn estimate_cost(&self, input: &str, expected_output_tokens: usize) -> f64 {
                // TODO: Calculate cost based on provider pricing
                todo!()
            }
        }
    };

    TokenStream::from(expanded)
}

struct LLMServiceConfig {
    model: String,
    provider: String,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    local: bool,
}

impl Parse for LLMServiceConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: Parse key-value pairs from attribute
        // model = "gpt-4", provider = "openai", ...
        todo!()
    }
}
```

```rust
// orchestrate/examples/llm.rs (new file)

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// LLM response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: usize,
    pub finish_reason: String,
}

/// Streaming response handler
pub struct LLMStream {
    receiver: mpsc::Receiver<String>,
}

impl LLMStream {
    pub async fn next(&mut self) -> Option<String> {
        self.receiver.recv().await
    }
}

/// LLM client abstraction
pub enum LLMProvider {
    OpenAI { api_key: String },
    Anthropic { api_key: String },
    Ollama { base_url: String },
}

impl LLMProvider {
    pub async fn send_request(
        &self,
        model: &str,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<LLMResponse, Box<dyn std::error::Error>> {
        // TODO: Implement API calls for each provider
        match self {
            LLMProvider::OpenAI { api_key } => {
                // TODO: Call OpenAI API
                todo!()
            }
            LLMProvider::Anthropic { api_key } => {
                // TODO: Call Anthropic API
                todo!()
            }
            LLMProvider::Ollama { base_url } => {
                // TODO: Call local Ollama API
                todo!()
            }
        }
    }

    pub async fn stream_request(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<LLMStream, Box<dyn std::error::Error>> {
        // TODO: Implement streaming for each provider
        let (tx, rx) = mpsc::channel(100);

        // Spawn task that streams chunks
        tokio::spawn(async move {
            // TODO: Stream chunks and send via tx
        });

        Ok(LLMStream { receiver: rx })
    }
}

/// Smart LLM router - chooses provider based on query complexity
pub struct LLMRouter {
    local: Option<Box<dyn LLMService>>,
    cloud: Option<Box<dyn LLMService>>,
    strategy: RoutingStrategy,
}

pub enum RoutingStrategy {
    CostOptimized,   // Prefer local for simple queries
    LatencyOptimized, // Prefer cloud for fast response
    QualityOptimized, // Always use best model
}

impl LLMRouter {
    pub async fn query(&self, prompt: &str) -> LLMResponse {
        match self.strategy {
            RoutingStrategy::CostOptimized => {
                // Simple queries (< 100 tokens, no code) → local
                // Complex queries → cloud
                let tokens = self.estimate_tokens(prompt);
                let has_code = prompt.contains("```") || prompt.contains("fn ");

                if tokens < 100 && !has_code {
                    if let Some(local) = &self.local {
                        return local.query(prompt).await;
                    }
                }

                if let Some(cloud) = &self.cloud {
                    cloud.query(prompt).await
                } else {
                    panic!("No cloud LLM configured");
                }
            }
            _ => todo!("Implement other strategies"),
        }
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        (text.len() as f64 / 4.0).ceil() as usize
    }
}

/// Trait for LLM services (allows polymorphism)
pub trait LLMService: Send + Sync {
    fn query(&self, prompt: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = LLMResponse> + Send + '_>>;
}
```

**Implementation Hints:**
1. Parse `#[api_key]` attribute to identify API key field
2. For OpenAI: POST to `https://api.openai.com/v1/chat/completions`
3. Token counting: use tiktoken library or estimate (1 token ≈ 4 chars)
4. Cost calculation: GPT-4 $0.03/1K input, $0.06/1K output tokens
5. Streaming: use Server-Sent Events (SSE) or chunked responses
6. Router strategy: check token count and complexity heuristics

---

## Milestone 4: MCP Server Orchestration with `#[mcp_server]`

### Introduction

**Why Milestone 3 Isn't Enough**: LLMs alone are limited—they can't read files, query databases, or execute commands. MCP (Model Context Protocol) connects LLMs to tools. Manual MCP integration requires protocol handling, resource management, lifecycle management.

**The Improvement**: Add `#[mcp_server]` attribute that generates MCP server process management, protocol handlers (stdio, SSE, HTTP), tool discovery and registration, automatic context attachment to LLMs, and graceful lifecycle management.

**Optimization (Lazy Loading)**: MCP servers consume resources. Starting all servers at boot wastes memory. Lazy loading: start server only when LLM requests that tool. File operations → start FileSystemMCP. Database query → start DatabaseMCP.

### Architecture

**New Macros:**
- `#[mcp_server]` - Attribute for MCP server definition
  - **Pattern**: `#[mcp_server(protocol = "stdio", port = 9000)]`
  - **Expands to**: MCP server process manager
  - **Role**: Tool server abstraction

**Key Structs:**
- MCP server structs
  - **Fields**: Configuration with `#[root_path]`, `#[permissions]`
  - **Methods**: Generated `start()`, `stop()`, `list_tools()`, `call_tool()`

- `MCPProcess` - Process manager
  - **Fields**: Child process, protocol type, status
  - **Methods**: `spawn()`, `send_request()`, `receive_response()`

### Checkpoint Tests

```rust
use orchestrate::*;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Service)]
#[mcp_server(protocol = "stdio")]
struct FileSystemMCP {
    #[root_path]
    allowed_paths: Vec<PathBuf>,

    #[permissions]
    read_only: bool,
}

#[derive(Service)]
#[mcp_server(protocol = "http", port = 9000)]
struct DatabaseMCP {
    #[connection]
    db_url: String,

    #[max_query_time]
    timeout: Duration,
}

#[test]
async fn test_mcp_server_startup() {
    let fs_server = FileSystemMCP {
        allowed_paths: vec![PathBuf::from("/tmp/test")],
        read_only: false,
    };

    fs_server.start().await.unwrap();

    assert!(fs_server.is_running());
    assert_eq!(fs_server.protocol(), "stdio");

    fs_server.stop().await.unwrap();
    assert!(!fs_server.is_running());
}

#[test]
async fn test_tool_discovery() {
    let fs_server = FileSystemMCP {
        allowed_paths: vec![PathBuf::from("/data")],
        read_only: false,
    };

    fs_server.start().await.unwrap();

    let tools = fs_server.list_tools().await.unwrap();

    // FileSystem MCP provides: read_file, write_file, list_directory, etc.
    assert!(tools.iter().any(|t| t.name == "read_file"));
    assert!(tools.iter().any(|t| t.name == "write_file"));
    assert!(tools.iter().any(|t| t.name == "list_directory"));
}

#[test]
async fn test_tool_invocation() {
    let fs_server = FileSystemMCP {
        allowed_paths: vec![PathBuf::from("/tmp")],
        read_only: false,
    };

    fs_server.start().await.unwrap();

    // Write a test file
    let result = fs_server.call_tool(
        "write_file",
        serde_json::json!({
            "path": "/tmp/test.txt",
            "content": "Hello, MCP!"
        })
    ).await.unwrap();

    assert!(result.success);

    // Read it back
    let result = fs_server.call_tool(
        "read_file",
        serde_json::json!({
            "path": "/tmp/test.txt"
        })
    ).await.unwrap();

    assert_eq!(result.content, "Hello, MCP!");
}

#[test]
async fn test_permission_enforcement() {
    let fs_server = FileSystemMCP {
        allowed_paths: vec![PathBuf::from("/data")],
        read_only: true,
    };

    fs_server.start().await.unwrap();

    // Writing should fail on read-only server
    let result = fs_server.call_tool(
        "write_file",
        serde_json::json!({
            "path": "/data/test.txt",
            "content": "test"
        })
    ).await;

    assert!(result.is_err());
}

#[test]
async fn test_path_restriction() {
    let fs_server = FileSystemMCP {
        allowed_paths: vec![PathBuf::from("/data")],
        read_only: false,
    };

    fs_server.start().await.unwrap();

    // Accessing outside allowed paths should fail
    let result = fs_server.call_tool(
        "read_file",
        serde_json::json!({
            "path": "/etc/passwd"  // Outside /data
        })
    ).await;

    assert!(result.is_err());
}

#[test]
async fn test_llm_with_mcp() {
    deployment! {
        MCPStack {
            services: {
                fs_server: FileSystemMCP {
                    allowed_paths: vec![PathBuf::from("/data")],
                    read_only: false,
                },

                agent: ChatAgent {
                    openai_key: env!("OPENAI_API_KEY"),
                    prompt: "You are a helpful assistant",
                    mcp_servers: [fs_server],
                },
            }
        }
    }

    let stack = MCPStack::new();
    stack.start().await;

    // LLM can now use file system tools
    let response = stack.agent.query("Read the file /data/config.json").await;

    // Response should indicate tool was used
    assert!(response.tools_used.contains(&"read_file".to_string()));
}
```

### Starter Code

```rust
// orchestrate-macros/examples/lib.rs (additions)

#[proc_macro_attribute]
pub fn mcp_server(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mcp_config = parse_macro_input!(attr as MCPServerConfig);
    let input = parse_macro_input!(item as DeriveInput);

    // TODO: Extract configuration fields
    // TODO: Generate start() method that spawns MCP server process
    // TODO: Generate stop() method
    // TODO: Generate list_tools() method
    // TODO: Generate call_tool() method

    let name = &input.ident;
    let protocol = &mcp_config.protocol;

    let expanded = quote! {
        #[derive(Service)]
        #input

        impl #name {
            pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
                // TODO: Spawn MCP server process based on protocol
                todo!()
            }

            pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
                // TODO: Terminate server process gracefully
                todo!()
            }

            pub fn is_running(&self) -> bool {
                // TODO: Check process status
                todo!()
            }

            pub fn protocol(&self) -> &'static str {
                #protocol
            }

            pub async fn list_tools(&self) -> Result<Vec<MCPTool>, Box<dyn std::error::Error>> {
                // TODO: Query server for available tools
                todo!()
            }

            pub async fn call_tool(
                &self,
                tool_name: &str,
                args: serde_json::Value,
            ) -> Result<MCPToolResult, Box<dyn std::error::Error>> {
                // TODO: Invoke tool via MCP protocol
                todo!()
            }
        }
    };

    TokenStream::from(expanded)
}

struct MCPServerConfig {
    protocol: String,
    port: Option<u16>,
}

impl Parse for MCPServerConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: Parse protocol and optional port
        todo!()
    }
}
```

```rust
// orchestrate/examples/mcp.rs (new file)

use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// MCP protocol types
#[derive(Debug, Clone, Copy)]
pub enum MCPProtocol {
    Stdio,   // Standard input/output
    SSE,     // Server-Sent Events
    HTTP,    // HTTP endpoints
}

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool invocation result
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPToolResult {
    pub success: bool,
    pub content: String,
    pub error: Option<String>,
}

/// MCP server process manager
pub struct MCPProcess {
    child: Option<Child>,
    protocol: MCPProtocol,
    stdin: Option<tokio::process::ChildStdin>,
    stdout_reader: Option<BufReader<tokio::process::ChildStdout>>,
}

impl MCPProcess {
    pub fn new(protocol: MCPProtocol) -> Self {
        Self {
            child: None,
            protocol,
            stdin: None,
            stdout_reader: None,
        }
    }

    /// Spawn MCP server process
    pub async fn spawn(&mut self, command: &str, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        match self.protocol {
            MCPProtocol::Stdio => {
                let mut child = Command::new(command)
                    .args(args)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()?;

                let stdin = child.stdin.take().ok_or("Failed to open stdin")?;
                let stdout = child.stdout.take().ok_or("Failed to open stdout")?;

                self.stdin = Some(stdin);
                self.stdout_reader = Some(BufReader::new(stdout));
                self.child = Some(child);

                Ok(())
            }
            MCPProtocol::HTTP => {
                // TODO: Start HTTP server
                todo!()
            }
            MCPProtocol::SSE => {
                // TODO: Start SSE server
                todo!()
            }
        }
    }

    /// Send request to MCP server
    pub async fn send_request(&mut self, request: &MCPRequest) -> Result<MCPResponse, Box<dyn std::error::Error>> {
        match self.protocol {
            MCPProtocol::Stdio => {
                // Serialize request as JSON + newline
                let json = serde_json::to_string(request)?;

                let stdin = self.stdin.as_mut().ok_or("Stdin not available")?;
                stdin.write_all(json.as_bytes()).await?;
                stdin.write_all(b"\n").await?;
                stdin.flush().await?;

                // Read response
                let reader = self.stdout_reader.as_mut().ok_or("Stdout not available")?;
                let mut line = String::new();
                reader.read_line(&mut line).await?;

                let response: MCPResponse = serde_json::from_str(&line)?;
                Ok(response)
            }
            _ => todo!("Implement HTTP/SSE protocols"),
        }
    }

    /// Terminate server process
    pub async fn kill(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = self.child.take() {
            child.kill().await?;
        }
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.child.is_some()
    }
}

/// MCP request structure
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// MCP response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<MCPError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
}
```

**Implementation Hints:**
1. MCP uses JSON-RPC 2.0 protocol over stdio/HTTP/SSE
2. For stdio: write JSON + newline to stdin, read JSON from stdout
3. Tool discovery: send `tools/list` method request
4. Tool call: send `tools/call` with tool name and arguments
5. Path validation: check if requested path starts with allowed_paths
6. Process lifecycle: spawn server, keep stdin/stdout handles, kill on drop

---

## Milestone 5: System Tool Integration with `#[system_tool]`

### Introduction

**Why Milestone 4 Isn't Enough**: MCP servers handle structured tools, but real DevOps needs system commands—git, docker, kubectl, npm. Manual command execution is dangerous (injection attacks) and error-prone (output parsing, error handling).

**The Improvement**: Add `#[system_tool]` attribute that generates safe command execution wrappers, argument validation and escaping, output capture and parsing, error handling with retries, and async process spawning.

**Optimization (Security)**: Command injection is #1 OWASP vulnerability. User input `; rm -rf /` can destroy systems. Generated code validates arguments, escapes special characters, uses array args (not shell strings), and whitelists allowed commands.

### Architecture

**New Macros:**
- `#[system_tool]` - Attribute for system command wrapper
  - **Pattern**: `#[system_tool]` on struct, `#[tool("command", args = [...])]` on methods
  - **Expands to**: Safe command execution methods
  - **Role**: System command abstraction

**Key Structs:**
- System tool structs
  - **Fields**: Command paths, working directories
  - **Methods**: Generated command execution methods

- `CommandExecutor` - Internal executor
  - **Fields**: Command, args, env, working_dir
  - **Methods**: `execute()`, `capture_output()`, `stream_output()`

### Checkpoint Tests

```rust
use orchestrate::*;
use std::path::PathBuf;

#[derive(Service)]
#[system_tool]
struct GitService {
    #[command("git")]
    git_path: PathBuf,

    #[working_dir]
    repo_dir: PathBuf,
}

impl GitService {
    #[tool("clone", args = ["url", "dest"])]
    async fn clone(&self, url: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Generated: execute git clone with proper error handling
    }

    #[tool("commit", args = ["message"])]
    async fn commit(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Generated: execute git commit -m "message"
    }

    #[tool("push", args = [])]
    async fn push(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Generated: execute git push
    }
}

#[derive(Service)]
#[system_tool]
struct DockerCLI {
    #[command("docker")]
    docker_path: PathBuf,
}

impl DockerCLI {
    #[tool("build", args = ["context", "tag"])]
    async fn build(&self, context: &str, tag: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Returns image ID
    }

    #[tool("run", args = ["image", "command"])]
    async fn run(&self, image: &str, command: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Returns container ID
    }
}

#[test]
async fn test_git_clone() {
    let git = GitService {
        git_path: PathBuf::from("git"),
        repo_dir: PathBuf::from("/tmp/test"),
    };

    std::fs::create_dir_all("/tmp/test").unwrap();

    git.clone("https://github.com/rust-lang/rust", "rust")
        .await
        .unwrap();

    assert!(PathBuf::from("/tmp/test/rust").exists());
}

#[test]
async fn test_git_commit() {
    let git = GitService {
        git_path: PathBuf::from("git"),
        repo_dir: PathBuf::from("/tmp/test/repo"),
    };

    // Initialize repo
    std::fs::create_dir_all("/tmp/test/repo").unwrap();
    git.execute_raw(&["init"]).await.unwrap();

    // Create file
    std::fs::write("/tmp/test/repo/test.txt", "test").unwrap();
    git.execute_raw(&["add", "."]).await.unwrap();

    // Commit
    git.commit("Initial commit").await.unwrap();

    // Verify commit exists
    let output = git.execute_raw(&["log", "--oneline"]).await.unwrap();
    assert!(output.contains("Initial commit"));
}

#[test]
async fn test_command_injection_prevention() {
    let git = GitService {
        git_path: PathBuf::from("git"),
        repo_dir: PathBuf::from("/tmp/test"),
    };

    // Attempt injection attack
    let malicious_message = "; rm -rf /; echo \"hacked";

    // Should escape or sanitize
    let result = git.commit(malicious_message).await;

    // Command should either succeed (sanitized) or fail (rejected)
    // But should NOT execute rm -rf /

    // Verify no files deleted (would fail test if injection worked)
    assert!(PathBuf::from("/tmp").exists());
}

#[test]
async fn test_docker_build() {
    let docker = DockerCLI {
        docker_path: PathBuf::from("docker"),
    };

    // Create simple Dockerfile
    std::fs::create_dir_all("/tmp/test/app").unwrap();
    std::fs::write(
        "/tmp/test/app/Dockerfile",
        "FROM alpine:latest\nCMD [\"echo\", \"hello\"]",
    ).unwrap();

    let image_id = docker.build("/tmp/test/app", "test:latest")
        .await
        .unwrap();

    assert!(image_id.len() > 0);
}

#[test]
async fn test_output_capture() {
    let git = GitService {
        git_path: PathBuf::from("git"),
        repo_dir: PathBuf::from("/tmp/test"),
    };

    let version_output = git.execute_raw(&["--version"]).await.unwrap();

    assert!(version_output.contains("git version"));
}

#[test]
async fn test_error_handling() {
    let git = GitService {
        git_path: PathBuf::from("git"),
        repo_dir: PathBuf::from("/tmp/test"),
    };

    // Try to clone to existing directory
    let result = git.clone(
        "https://github.com/nonexistent/repo",
        "/tmp/test/existing",
    ).await;

    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.to_string().contains("fatal") || err.to_string().contains("error"));
}
```

### Starter Code

```rust
// orchestrate-macros/examples/lib.rs (additions)

#[proc_macro_attribute]
pub fn system_tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    // TODO: Find fields with #[command] and #[working_dir] attributes
    // TODO: Generate execute_raw() method for running arbitrary commands

    let name = &input.ident;

    let expanded = quote! {
        #[derive(Service)]
        #input

        impl #name {
            /// Execute raw command (internal use)
            pub async fn execute_raw(
                &self,
                args: &[&str],
            ) -> Result<String, Box<dyn std::error::Error>> {
                // TODO: Build command from self.command_path
                // TODO: Set working directory
                // TODO: Execute and capture output
                todo!()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn tool(attr: TokenStream, item: TokenStream) -> TokenStream {
    let tool_config = parse_macro_input!(attr as ToolConfig);
    let func = parse_macro_input!(item as syn::ItemFn);

    // TODO: Generate method that:
    // 1. Validates arguments
    // 2. Escapes special characters
    // 3. Builds command array
    // 4. Calls execute_raw()
    // 5. Parses output

    let command = &tool_config.command;
    let args = &tool_config.args;

    let expanded = quote! {
        #func

        // TODO: Generate actual implementation
    };

    TokenStream::from(expanded)
}

struct ToolConfig {
    command: String,
    args: Vec<String>,
}

impl Parse for ToolConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // TODO: Parse "command", args = ["arg1", "arg2"]
        todo!()
    }
}
```

```rust
// orchestrate/examples/command.rs (new file)

use tokio::process::Command;
use std::path::PathBuf;

/// Safe command executor
pub struct CommandExecutor {
    command: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    working_dir: Option<PathBuf>,
}

impl CommandExecutor {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            working_dir: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Execute command and capture output
    pub async fn execute(self) -> Result<String, Box<dyn std::error::Error>> {
        // TODO: Validate arguments (no shell metacharacters if not intended)
        // TODO: Build Command
        // TODO: Execute and capture output

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Command failed: {}", stderr).into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    /// Validate argument doesn't contain shell injection
    fn validate_arg(arg: &str) -> Result<(), String> {
        // TODO: Check for dangerous characters
        // Dangerous: ; | & $ ` \n ( ) < > \

        let dangerous_chars = [';', '|', '&', '$', '`', '\n', '(', ')'];

        for ch in dangerous_chars {
            if arg.contains(ch) {
                return Err(format!("Argument contains dangerous character: {}", ch));
            }
        }

        Ok(())
    }
}

/// Escape argument for shell safety
pub fn escape_arg(arg: &str) -> String {
    // TODO: Properly escape for shell
    // For POSIX shells: wrap in single quotes and escape any single quotes

    if arg.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/') {
        // Safe characters, no escaping needed
        arg.to_string()
    } else {
        // Wrap in single quotes and escape existing single quotes
        format!("'{}'", arg.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_arg() {
        assert_eq!(escape_arg("simple"), "simple");
        assert_eq!(escape_arg("hello world"), "'hello world'");
        assert_eq!(escape_arg("it's"), r"'it'\''s'");
        assert_eq!(escape_arg("; rm -rf /"), "'; rm -rf /'");
    }

    #[test]
    fn test_validate_arg() {
        assert!(CommandExecutor::validate_arg("safe_arg").is_ok());
        assert!(CommandExecutor::validate_arg("hello; rm -rf /").is_err());
        assert!(CommandExecutor::validate_arg("test | grep foo").is_err());
    }
}
```

**Implementation Hints:**
1. Never use `sh -c` with user input—use Command::new() with array args
2. Validate arguments: reject or escape shell metacharacters (`;`, `|`, `&`, `$`, `` ` ``)
3. For git commit: use `git commit -m "message"` (not via shell)
4. Capture stdout and stderr separately for better error messages
5. Working directory: use `current_dir()` on Command
6. For docker build: parse image ID from output (last line with `sha256:`)

---

## Milestone 6: Complete Orchestration with Health Checks and Auto-Scaling

### Introduction

**Why Milestone 5 Isn't Enough**: Production systems need monitoring, auto-healing, and scaling. Services crash, traffic spikes, resources exhaust. Need health checks, auto-restart, horizontal scaling, resource limits, monitoring integration.

**The Improvement**: Add health check loops, auto-scaling based on metrics (CPU, memory, custom), Kubernetes HPA (Horizontal Pod Autoscaler) generation, Prometheus metrics export, Grafana dashboard JSON, rolling update strategies, graceful shutdown handlers.

**Optimization (Resilience)**: Service crashes cost money. 1 minute downtime = lost revenue. Health checks detect failures in 10 seconds, auto-restart in 5 seconds = 15 second outage vs manual restart (5+ minutes). Auto-scaling handles traffic spikes automatically—Black Friday traffic 10x normal, system scales up without human intervention.

### Architecture

**New Attributes:**
- `#[health_check]` - Health monitoring
  - **Pattern**: `#[health_check(path = "/health", interval = "30s")]`
  - **Expands to**: Health check loop

- `#[scaling]` - Auto-scaling configuration
  - **Pattern**: `#[scaling(min = 2, max = 10, metric = "cpu", threshold = 70)]`
  - **Expands to**: Scaling logic and K8s HPA

- `#[resource_limits]` - Resource constraints
  - **Pattern**: `#[resource_limits(memory = "2Gi", cpu = "1000m")]`
  - **Expands to**: Docker/K8s resource limits

**Generated Code:**
- Health check monitoring loops
- Auto-scaling decision logic
- Kubernetes HPA manifests
- Prometheus exporters
- Grafana dashboards
- Rolling update orchestration

### Checkpoint Tests

```rust
use orchestrate::*;
use std::time::Duration;

#[derive(Service)]
#[container(image = "myapi:latest", port = 8080)]
struct ApiServer {
    #[env]
    port: u16,

    #[health_check(path = "/health", interval = "10s", retries = 3)]
    health: HealthCheck,

    #[scaling(min = 2, max = 10, metric = "cpu", threshold = 70)]
    autoscale: AutoScaleConfig,
}

#[derive(Service)]
#[container(image = "postgres:15", port = 5432)]
struct Database {
    #[env("POSTGRES_PASSWORD")]
    password: String,

    #[health_check(command = "pg_isready", interval = "30s")]
    health: HealthCheck,

    #[resource_limits(memory = "4Gi", cpu = "2000m")]
    limits: ResourceLimits,
}

deployment! {
    ProductionStack {
        services: {
            db: Database {
                password: env!("DB_PASSWORD"),
                health: HealthCheck::command("pg_isready"),
                limits: ResourceLimits::new("4Gi", "2000m"),
            },

            api: ApiServer {
                port: 8080,
                depends_on: [db],
                health: HealthCheck::http("/health", Duration::from_secs(10)),
                autoscale: AutoScaleConfig {
                    min: 2,
                    max: 10,
                    metric: ScalingMetric::CPU,
                    threshold: 70,
                },
            },
        },

        monitoring: {
            prometheus: PrometheusConfig {
                scrape_interval: Duration::from_secs(15),
                targets: [db, api],
            },

            grafana: GrafanaConfig {
                dashboards: ["system", "application"],
            },
        }
    }
}

#[test]
async fn test_health_check_monitoring() {
    let api = ApiServer {
        port: 8080,
        health: HealthCheck::http("/health", Duration::from_secs(10)),
        autoscale: AutoScaleConfig::default(),
    };

    // Start service
    api.start().await;

    // Health check should pass
    assert!(api.is_healthy().await);

    // Simulate service failure
    api.stop().await;

    tokio::time::sleep(Duration::from_secs(11)).await;

    // Health check should detect failure
    assert!(!api.is_healthy().await);
}

#[test]
async fn test_auto_restart_on_failure() {
    let api = ApiServer {
        port: 8080,
        health: HealthCheck::http("/health", Duration::from_secs(5)),
        autoscale: AutoScaleConfig::default(),
    };

    api.start_with_monitoring().await;

    // Kill the service
    api.force_stop().await;

    // Wait for health check to detect and restart
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Should be running again
    assert!(api.is_running());
}

#[test]
async fn test_autoscaling_scale_up() {
    let stack = ProductionStack::new();
    stack.start().await;

    // Simulate high CPU usage
    stack.api.set_cpu_usage(80.0);

    // Wait for scaling decision
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Should scale up
    let initial_replicas = stack.api.replica_count();
    assert!(initial_replicas > 2); // Started with min=2
}

#[test]
async fn test_autoscaling_scale_down() {
    let stack = ProductionStack::new();
    stack.start().await;

    // Scale up first
    stack.api.set_replica_count(5);

    // Then reduce load
    stack.api.set_cpu_usage(30.0);

    // Wait for scale down
    tokio::time::sleep(Duration::from_secs(60)).await; // Scale down is slower

    // Should scale down
    assert!(stack.api.replica_count() < 5);
    assert!(stack.api.replica_count() >= 2); // Not below min
}

#[test]
fn test_kubernetes_hpa_generation() {
    let stack = ProductionStack::new();
    let k8s = stack.to_kubernetes();

    // Should include HPA manifest
    assert!(k8s.contains("kind: HorizontalPodAutoscaler"));
    assert!(k8s.contains("apiVersion: autoscaling/v2"));
    assert!(k8s.contains("minReplicas: 2"));
    assert!(k8s.contains("maxReplicas: 10"));
    assert!(k8s.contains("targetCPUUtilizationPercentage: 70"));
}

#[test]
fn test_resource_limits_in_k8s() {
    let db = Database {
        password: "test".to_string(),
        health: HealthCheck::command("pg_isready"),
        limits: ResourceLimits::new("4Gi", "2000m"),
    };

    let k8s = db.to_kubernetes_deployment();

    assert!(k8s.contains("resources:"));
    assert!(k8s.contains("limits:"));
    assert!(k8s.contains("memory: 4Gi"));
    assert!(k8s.contains("cpu: 2000m"));
}

#[test]
fn test_prometheus_exporter() {
    let stack = ProductionStack::new();
    stack.start().await;

    // Should expose metrics endpoint
    let metrics = stack.api.get_metrics().await;

    assert!(metrics.contains("http_requests_total"));
    assert!(metrics.contains("cpu_usage"));
    assert!(metrics.contains("memory_usage"));
}

#[test]
fn test_grafana_dashboard_generation() {
    let stack = ProductionStack::new();
    let dashboard = stack.monitoring.grafana.generate_dashboard("application");

    // Should be valid Grafana JSON
    let json: serde_json::Value = serde_json::from_str(&dashboard).unwrap();

    assert_eq!(json["title"], "application");
    assert!(json["panels"].is_array());
}
```

### Starter Code

```rust
// orchestrate/examples/health.rs (new file)

use std::time::Duration;
use tokio::time::sleep;

/// Health check configuration
pub enum HealthCheck {
    HTTP { path: String, interval: Duration },
    TCP { port: u16, interval: Duration },
    Command { command: String, interval: Duration },
}

impl HealthCheck {
    pub fn http(path: impl Into<String>, interval: Duration) -> Self {
        HealthCheck::HTTP {
            path: path.into(),
            interval,
        }
    }

    pub fn command(command: impl Into<String>) -> Self {
        HealthCheck::Command {
            command: command.into(),
            interval: Duration::from_secs(30),
        }
    }

    /// Run health check
    pub async fn check(&self) -> bool {
        match self {
            HealthCheck::HTTP { path, .. } => {
                // TODO: Make HTTP request to localhost:port/path
                // Return true if status 200-299
                todo!()
            }
            HealthCheck::TCP { port, .. } => {
                // TODO: Try to connect to localhost:port
                // Return true if connection succeeds
                todo!()
            }
            HealthCheck::Command { command, .. } => {
                // TODO: Execute command
                // Return true if exit code 0
                todo!()
            }
        }
    }

    /// Start monitoring loop
    pub async fn monitor<F>(&self, on_failure: F)
    where
        F: Fn() + Send + 'static,
    {
        let interval = match self {
            HealthCheck::HTTP { interval, .. } => *interval,
            HealthCheck::TCP { interval, .. } => *interval,
            HealthCheck::Command { interval, .. } => *interval,
        };

        loop {
            sleep(interval).await;

            if !self.check().await {
                on_failure();
            }
        }
    }
}

/// Auto-scaling configuration
pub struct AutoScaleConfig {
    pub min: u32,
    pub max: u32,
    pub metric: ScalingMetric,
    pub threshold: u32,
}

impl Default for AutoScaleConfig {
    fn default() -> Self {
        Self {
            min: 1,
            max: 1,
            metric: ScalingMetric::CPU,
            threshold: 80,
        }
    }
}

pub enum ScalingMetric {
    CPU,
    Memory,
    RequestRate,
    Custom(String),
}

impl AutoScaleConfig {
    /// Decide if scaling is needed
    pub fn should_scale(&self, current_replicas: u32, current_metric: f64) -> ScalingDecision {
        if current_metric > self.threshold as f64 && current_replicas < self.max {
            ScalingDecision::ScaleUp
        } else if current_metric < (self.threshold as f64 * 0.5) && current_replicas > self.min {
            ScalingDecision::ScaleDown
        } else {
            ScalingDecision::NoChange
        }
    }

    /// Generate Kubernetes HPA YAML
    pub fn to_kubernetes_hpa(&self, service_name: &str) -> String {
        format!(
            r#"apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {}-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {}
  minReplicas: {}
  maxReplicas: {}
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: {}
"#,
            service_name, service_name, self.min, self.max, self.threshold
        )
    }
}

pub enum ScalingDecision {
    ScaleUp,
    ScaleDown,
    NoChange,
}

/// Resource limits
pub struct ResourceLimits {
    pub memory: String,
    pub cpu: String,
}

impl ResourceLimits {
    pub fn new(memory: impl Into<String>, cpu: impl Into<String>) -> Self {
        Self {
            memory: memory.into(),
            cpu: cpu.into(),
        }
    }

    /// Generate Kubernetes resource spec
    pub fn to_kubernetes_spec(&self) -> String {
        format!(
            r#"        resources:
          limits:
            memory: {}
            cpu: {}
          requests:
            memory: {}
            cpu: {}
"#,
            self.memory,
            self.cpu,
            // Requests = 50% of limits
            self.scale_resource(&self.memory, 0.5),
            self.scale_resource(&self.cpu, 0.5),
        )
    }

    fn scale_resource(&self, resource: &str, factor: f64) -> String {
        // TODO: Parse resource (e.g., "4Gi" -> 4 * 1024 * factor)
        // For now, simple implementation
        resource.to_string()
    }
}
```

```rust
// orchestrate/examples/monitoring.rs (new file)

use serde_json::json;
use std::time::Duration;

/// Prometheus configuration
pub struct PrometheusConfig {
    pub scrape_interval: Duration,
    pub targets: Vec<String>,
}

impl PrometheusConfig {
    /// Generate prometheus.yml
    pub fn to_yaml(&self) -> String {
        let targets_yaml = self.targets
            .iter()
            .map(|t| format!("      - '{}'", t))
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"global:
  scrape_interval: {}s

scrape_configs:
  - job_name: 'services'
    static_configs:
    - targets:
{}
"#,
            self.scrape_interval.as_secs(),
            targets_yaml
        )
    }
}

/// Grafana configuration
pub struct GrafanaConfig {
    pub dashboards: Vec<String>,
}

impl GrafanaConfig {
    /// Generate Grafana dashboard JSON
    pub fn generate_dashboard(&self, name: &str) -> String {
        let dashboard = json!({
            "dashboard": {
                "title": name,
                "panels": [
                    {
                        "id": 1,
                        "title": "CPU Usage",
                        "type": "graph",
                        "targets": [
                            {
                                "expr": "rate(cpu_usage[5m])",
                            }
                        ]
                    },
                    {
                        "id": 2,
                        "title": "Memory Usage",
                        "type": "graph",
                        "targets": [
                            {
                                "expr": "memory_usage_bytes",
                            }
                        ]
                    },
                    {
                        "id": 3,
                        "title": "Request Rate",
                        "type": "graph",
                        "targets": [
                            {
                                "expr": "rate(http_requests_total[5m])",
                            }
                        ]
                    }
                ],
                "refresh": "10s"
            }
        });

        serde_json::to_string_pretty(&dashboard).unwrap()
    }
}
```

**Implementation Hints:**
1. Health checks: use `reqwest` for HTTP, `tokio::net::TcpStream` for TCP
2. Monitoring loop: spawn background task with `tokio::spawn`
3. Auto-scaling: check metrics every 30 seconds, scale up fast, down slow (5 min cooldown)
4. HPA: Kubernetes Horizontal Pod Autoscaler uses metrics-server
5. Prometheus: expose metrics on `/metrics` endpoint in standard format
6. Grafana: dashboard JSON includes panels, queries, refresh interval

---

## Complete Working Example

```rust
// Complete example combining all milestones

use orchestrate::*;
use std::path::PathBuf;
use std::time::Duration;

// Define all services
#[derive(Service)]
#[container(image = "postgres:15", port = 5432)]
struct Database {
    #[env("POSTGRES_PASSWORD")]
    password: String,

    #[health_check(command = "pg_isready")]
    health: HealthCheck,
}

#[derive(Service)]
#[container(image = "redis:7", port = 6379)]
struct Cache {
    #[env]
    max_memory: String,
}

#[derive(Service)]
#[llm_service(model = "gpt-4", provider = "openai")]
struct DevOpsAgent {
    #[api_key]
    key: String,

    #[system_prompt]
    prompt: &'static str,
}

#[derive(Service)]
#[mcp_server(protocol = "stdio")]
struct GitHubMCP {
    #[auth_token]
    token: String,
}

#[derive(Service)]
#[system_tool]
struct KubectlTool {
    #[command("kubectl")]
    kubectl: PathBuf,
}

// Orchestrate everything
deployment! {
    ProductionPlatform {
        services: {
            db: Database {
                password: env!("DB_PASSWORD"),
                health: HealthCheck::command("pg_isready"),
            },

            cache: Cache {
                max_memory: "512mb",
            },

            github: GitHubMCP {
                token: env!("GITHUB_TOKEN"),
            },

            kubectl: KubectlTool {
                kubectl: PathBuf::from("kubectl"),
            },

            agent: DevOpsAgent {
                key: env!("OPENAI_API_KEY"),
                prompt: "You are a DevOps automation assistant",
                mcp_servers: [github],
                system_tools: [kubectl],
                depends_on: [db, cache],
            },
        },

        networks: {
            backend: [db, cache, agent],
        },

        monitoring: {
            prometheus: PrometheusConfig {
                scrape_interval: Duration::from_secs(15),
                targets: [db, cache, agent],
            },
        }
    }
}

#[orchestrate::main]
async fn main() {
    println!("🚀 Starting Production Platform...");

    let platform = ProductionPlatform::deploy().await;

    println!("✅ All services started");
    println!("📊 Monitoring: http://localhost:9090 (Prometheus)");
    println!("🤖 Agent ready for DevOps automation");

    // Platform runs with health checks and auto-scaling
    platform.run_forever().await;
}
```

This complete implementation demonstrates:
1. **Container orchestration** - Docker/Kubernetes manifest generation
2. **Multi-service dependencies** - Topological ordering and network isolation
3. **LLM integration** - OpenAI with cost optimization
4. **MCP servers** - Tool access for LLMs
5. **System tools** - Safe git/kubectl execution
6. **Production features** - Health checks, auto-scaling, monitoring

The orchestration DSL provides type-safe, production-ready infrastructure as code with compile-time validation!
