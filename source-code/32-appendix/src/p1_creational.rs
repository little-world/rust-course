// Pattern 1: Creational Patterns - Builder, Factory, Singleton, Prototype
// Demonstrates object creation patterns in Rust.

use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

// ============================================================================
// Example: Builder Pattern - Fluent API for Complex Construction
// ============================================================================

struct HttpClient {
    base_url: String,
    timeout: Duration,
    user_agent: String,
    max_retries: u32,
    follow_redirects: bool,
    compression: bool,
}

struct HttpClientBuilder {
    base_url: String,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    max_retries: Option<u32>,
    follow_redirects: bool,
    compression: bool,
}

impl HttpClientBuilder {
    fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            timeout: None,
            user_agent: None,
            max_retries: None,
            follow_redirects: true,
            compression: true,
        }
    }

    fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    #[allow(dead_code)]
    fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    fn build(self) -> HttpClient {
        HttpClient {
            base_url: self.base_url,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            user_agent: self.user_agent.unwrap_or_else(|| "RustClient/1.0".to_string()),
            max_retries: self.max_retries.unwrap_or(3),
            follow_redirects: self.follow_redirects,
            compression: self.compression,
        }
    }
}

fn builder_basic_example() {
    let client = HttpClientBuilder::new("https://api.example.com")
        .timeout(Duration::from_secs(60))
        .max_retries(5)
        .user_agent("MyApp/2.0")
        .build();

    println!("Built HttpClient:");
    println!("  base_url: {}", client.base_url);
    println!("  timeout: {:?}", client.timeout);
    println!("  user_agent: {}", client.user_agent);
    println!("  max_retries: {}", client.max_retries);
    println!("  follow_redirects: {}", client.follow_redirects);
    println!("  compression: {}", client.compression);
}

// ============================================================================
// Example: Typestate Pattern - Compile-time Validation
// ============================================================================

struct Incomplete;
struct Complete;

struct ConfigBuilder<S> {
    host: Option<String>,
    port: Option<u16>,
    _state: PhantomData<S>,
}

impl ConfigBuilder<Incomplete> {
    fn new() -> Self {
        Self {
            host: None,
            port: None,
            _state: PhantomData,
        }
    }

    fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    fn port(self, port: u16) -> ConfigBuilder<Complete> {
        ConfigBuilder {
            host: self.host,
            port: Some(port),
            _state: PhantomData,
        }
    }
}

impl ConfigBuilder<Complete> {
    fn build(self) -> Config {
        Config {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
        }
    }
}

#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
}

fn typestate_example() {
    // Compile-time error if required fields missing
    // let config = ConfigBuilder::new().build(); // Error: no method `build`
    let config = ConfigBuilder::new()
        .host("localhost".to_string())
        .port(8080)
        .build();

    println!("Config: {:?}", config);
}

// ============================================================================
// Example: Factory Pattern with Trait Objects
// ============================================================================

trait Button {
    fn render(&self) -> String;
    fn on_click(&self);
}

struct WindowsButton;
impl Button for WindowsButton {
    fn render(&self) -> String {
        "Rendering Windows button".to_string()
    }
    fn on_click(&self) {
        println!("Windows click event");
    }
}

struct MacButton;
impl Button for MacButton {
    fn render(&self) -> String {
        "Rendering Mac button".to_string()
    }
    fn on_click(&self) {
        println!("Mac click event");
    }
}

trait UIFactory {
    fn create_button(&self) -> Box<dyn Button>;
}

struct WindowsFactory;
impl UIFactory for WindowsFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(WindowsButton)
    }
}

struct MacFactory;
impl UIFactory for MacFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(MacButton)
    }
}

fn render_ui(factory: &dyn UIFactory) {
    let button = factory.create_button();
    println!("{}", button.render());
    button.on_click();
}

fn factory_trait_object_example() {
    println!("Windows Factory:");
    let factory: Box<dyn UIFactory> = Box::new(WindowsFactory);
    render_ui(&*factory);

    println!("\nMac Factory:");
    let factory: Box<dyn UIFactory> = Box::new(MacFactory);
    render_ui(&*factory);
}

// ============================================================================
// Example: Factory Pattern with Enums (Zero-Cost)
// ============================================================================

#[derive(Clone, Copy)]
enum Platform {
    Windows,
    Mac,
}

enum PlatformButton {
    Windows(WindowsButton),
    Mac(MacButton),
}

impl PlatformButton {
    fn new(platform: Platform) -> Self {
        match platform {
            Platform::Windows => PlatformButton::Windows(WindowsButton),
            Platform::Mac => PlatformButton::Mac(MacButton),
        }
    }

    fn render(&self) -> String {
        match self {
            PlatformButton::Windows(btn) => btn.render(),
            PlatformButton::Mac(btn) => btn.render(),
        }
    }
}

fn factory_enum_example() {
    // No heap allocation, no dynamic dispatch
    let button = PlatformButton::new(Platform::Windows);
    println!("Enum-based factory: {}", button.render());
}

// ============================================================================
// Example: Singleton Pattern with OnceLock
// ============================================================================

struct AppConfig {
    api_key: String,
    debug_mode: bool,
}

impl AppConfig {
    fn global() -> &'static AppConfig {
        static CONFIG: OnceLock<AppConfig> = OnceLock::new();
        CONFIG.get_or_init(|| AppConfig {
            api_key: std::env::var("API_KEY").unwrap_or_else(|_| "default_key".to_string()),
            debug_mode: cfg!(debug_assertions),
        })
    }
}

fn singleton_example() {
    let config = AppConfig::global();
    println!("Singleton AppConfig:");
    println!("  api_key: {}", config.api_key);
    println!("  debug_mode: {}", config.debug_mode);

    // Same instance
    let config2 = AppConfig::global();
    println!("  Same instance: {}", std::ptr::eq(config, config2));
}

// ============================================================================
// Example: Dependency Injection (Preferred over Singleton)
// ============================================================================

struct Database {
    connection_string: String,
}

impl Database {
    fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

struct UserService<'a> {
    db: &'a Database,
}

impl<'a> UserService<'a> {
    fn new(db: &'a Database) -> Self {
        Self { db }
    }

    fn get_connection(&self) -> &str {
        &self.db.connection_string
    }
}

fn dependency_injection_example() {
    // Explicit dependencies, testable, no global state
    let db = Database::new("postgres://localhost".to_string());
    let service = UserService::new(&db);
    println!("DI Example - Connection: {}", service.get_connection());
}

// ============================================================================
// Example: Prototype Pattern with Clone
// ============================================================================

#[derive(Clone)]
struct TemplateEngine {
    templates: HashMap<String, String>,
    config: TemplateConfig,
}

#[derive(Clone)]
struct TemplateConfig {
    strict_mode: bool,
    cache_enabled: bool,
}

fn load_template(_name: &str) -> String {
    // Expensive operation
    String::from("template content")
}

impl TemplateEngine {
    fn new() -> Self {
        let mut templates = HashMap::new();
        // Expensive initialization
        templates.insert("header".to_string(), load_template("header.html"));
        templates.insert("footer".to_string(), load_template("footer.html"));

        Self {
            templates,
            config: TemplateConfig {
                strict_mode: true,
                cache_enabled: true,
            },
        }
    }

    fn with_different_config(&self, config: TemplateConfig) -> Self {
        let mut cloned = self.clone();
        cloned.config = config;
        cloned // Reuses expensive template loading
    }
}

fn prototype_example() {
    let base_engine = TemplateEngine::new(); // Expensive

    // Cheap clones with variations
    let dev_engine = base_engine.with_different_config(TemplateConfig {
        strict_mode: false,
        cache_enabled: false,
    });
    let prod_engine = base_engine.clone(); // Reuses all data

    println!("Prototype Pattern:");
    println!("  Base engine templates: {}", base_engine.templates.len());
    println!("  Dev engine strict_mode: {}", dev_engine.config.strict_mode);
    println!("  Prod engine cache_enabled: {}", prod_engine.config.cache_enabled);
}

// ============================================================================
// Example: Deep vs Shallow Cloning
// ============================================================================

#[derive(Clone)]
struct SharedData {
    // Shallow clone: reference counted
    cache: Rc<Vec<u8>>,
    // Deep clone: clones the String
    user_id: String,
}

fn clone_example() {
    let original = SharedData {
        cache: Rc::new(vec![1, 2, 3]),
        user_id: "user123".to_string(),
    };

    let cloned = original.clone();

    // cache is shared (reference count increased)
    // user_id is copied
    println!("Deep vs Shallow Clone:");
    println!("  Original cache ref count: {}", Rc::strong_count(&original.cache));
    println!("  Cache is shared: {}", Rc::strong_count(&original.cache) == 2);
    assert_eq!(Rc::strong_count(&original.cache), 2);
    println!("  user_id cloned separately: {}", cloned.user_id);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let client = HttpClientBuilder::new("https://example.com").build();
        assert_eq!(client.timeout, Duration::from_secs(30));
        assert_eq!(client.user_agent, "RustClient/1.0");
        assert_eq!(client.max_retries, 3);
        assert!(client.follow_redirects);
        assert!(client.compression);
    }

    #[test]
    fn test_builder_custom_values() {
        let client = HttpClientBuilder::new("https://example.com")
            .timeout(Duration::from_secs(60))
            .user_agent("MyApp")
            .max_retries(5)
            .build();

        assert_eq!(client.timeout, Duration::from_secs(60));
        assert_eq!(client.user_agent, "MyApp");
        assert_eq!(client.max_retries, 5);
    }

    #[test]
    fn test_typestate_builder() {
        let config = ConfigBuilder::new()
            .host("localhost".to_string())
            .port(8080)
            .build();

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_factory_trait_object() {
        let windows: Box<dyn UIFactory> = Box::new(WindowsFactory);
        let mac: Box<dyn UIFactory> = Box::new(MacFactory);

        let win_btn = windows.create_button();
        let mac_btn = mac.create_button();

        assert!(win_btn.render().contains("Windows"));
        assert!(mac_btn.render().contains("Mac"));
    }

    #[test]
    fn test_factory_enum() {
        let win_btn = PlatformButton::new(Platform::Windows);
        let mac_btn = PlatformButton::new(Platform::Mac);

        assert!(win_btn.render().contains("Windows"));
        assert!(mac_btn.render().contains("Mac"));
    }

    #[test]
    fn test_singleton() {
        let config1 = AppConfig::global();
        let config2 = AppConfig::global();
        assert!(std::ptr::eq(config1, config2));
    }

    #[test]
    fn test_prototype_clone() {
        let engine = TemplateEngine::new();
        let cloned = engine.clone();

        assert_eq!(engine.templates.len(), cloned.templates.len());
        assert_eq!(engine.config.strict_mode, cloned.config.strict_mode);
    }

    #[test]
    fn test_prototype_with_config() {
        let engine = TemplateEngine::new();
        let modified = engine.with_different_config(TemplateConfig {
            strict_mode: false,
            cache_enabled: false,
        });

        assert!(engine.config.strict_mode);
        assert!(!modified.config.strict_mode);
    }

    #[test]
    fn test_shallow_clone() {
        let original = SharedData {
            cache: Rc::new(vec![1, 2, 3]),
            user_id: "user".to_string(),
        };

        let _cloned = original.clone();
        assert_eq!(Rc::strong_count(&original.cache), 2);
    }
}

fn main() {
    println!("Pattern 1: Creational Patterns");
    println!("===============================\n");

    println!("=== Builder Pattern ===");
    builder_basic_example();
    println!();

    println!("=== Typestate Builder ===");
    typestate_example();
    println!();

    println!("=== Factory Pattern (Trait Objects) ===");
    factory_trait_object_example();
    println!();

    println!("=== Factory Pattern (Enums) ===");
    factory_enum_example();
    println!();

    println!("=== Singleton Pattern ===");
    singleton_example();
    println!();

    println!("=== Dependency Injection ===");
    dependency_injection_example();
    println!();

    println!("=== Prototype Pattern ===");
    prototype_example();
    println!();

    println!("=== Deep vs Shallow Clone ===");
    clone_example();
}
