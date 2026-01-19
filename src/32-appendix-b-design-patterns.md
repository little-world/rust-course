# Appendix B: Design Pattern Catalog

**Creational Patterns:**

- [Builder Pattern](#builder-pattern)
- [Factory Pattern](#factory-pattern)
- [Singleton Pattern](#singleton-pattern)
- [Prototype Pattern](#prototype-pattern)

**Structural Patterns:**

- [Adapter Pattern](#adapter-pattern)
- [Decorator Pattern](#decorator-pattern)
- [Facade Pattern](#facade-pattern)
- [Newtype Pattern](#newtype-pattern)

**Behavioral Patterns:**

- [Strategy Pattern](#strategy-pattern)
- [Observer Pattern](#observer-pattern)
- [Command Pattern](#command-pattern)
- [Iterator Pattern](#iterator-pattern)

**Concurrency Patterns:**

- [Thread Pool Pattern](#thread-pool-pattern)
- [Producer-Consumer Pattern](#producer-consumer-pattern)
- [Fork-Join Pattern](#fork-join-pattern)
- [Actor Pattern](#actor-pattern)
- [Async/Await Pattern](#asyncawait-pattern)


Design patterns are reusable solutions to common programming problems. They're not finished code you can copy, but templates for how to solve a problem in many different situations. In Rust, classic design patterns take on unique characteristics due to the language's ownership model, zero-cost abstractions, and powerful type system.

This catalog adapts traditional object-oriented patterns to Rust's paradigm while introducing patterns unique to systems programming and Rust's ecosystem. Some patterns that require runtime polymorphism in other languages can be implemented at compile-time in Rust through generics and traits. Others that rely on shared mutable state require different approaches due to Rust's borrowing rules.

The patterns are organized into four categories:

**Creational patterns** control object creation, managing complexity when instantiating objects requires more than simple construction. In Rust, these often leverage the type system to enforce invariants at compile-time.

**Structural patterns** organize relationships between entities, composing objects to form larger structures. Rust's trait system and zero-cost abstractions enable elegant implementations without runtime overhead.

**Behavioral patterns** focus on communication between objects, defining how they interact and distribute responsibility. Rust's ownership model influences how we implement delegation and message passing.

**Concurrency patterns** address the challenges of parallel and asynchronous execution. Rust's fearless concurrency model, with its compile-time race detection, enables patterns that would be unsafe in other languages.

Each pattern includes:
- **Intent**: What problem does it solve?
- **Motivation**: When and why would you use it?
- **Implementation**: Idiomatic Rust code
- **Trade-offs**: Advantages and limitations
- **Variations**: Common adaptations

---

## Creational Patterns

Creational patterns abstract the instantiation process, making systems independent of how objects are created, composed, and represented. In Rust, these patterns often encode constraints in the type system, moving validation from runtime to compile-time.

### Builder Pattern

**Intent**: Separate the construction of a complex object from its representation, allowing the same construction process to create different representations.

**Motivation**: When creating an object requires many optional parameters or complex initialization logic, constructors become unwieldy. The builder pattern provides a fluent interface for step-by-step construction, improving readability and enabling compile-time validation of required fields.

```rust

// Problem: Too many constructor parameters

struct HttpClient {
    base_url: String,
    timeout: Duration,
    user_agent: String,
    max_retries: u32,
    follow_redirects: bool,
    compression: bool,
}

// Unwieldy constructor
impl HttpClient {
    fn new(
        base_url: String,
        timeout: Duration,
        user_agent: String,
        max_retries: u32,
        follow_redirects: bool,
        compression: bool,
    ) -> Self {
        // ...
    }
}

// Solution: Builder pattern
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

let client = HttpClientBuilder::new("https://api.example.com")
    .timeout(Duration::from_secs(60))
    .max_retries(5)
    .user_agent("MyApp/2.0")
    .build();
```

**Advanced: Typestate pattern for compile-time validation**

The typestate pattern uses phantom types to encode state in the type system, catching errors at compile-time:

```rust
use std::marker::PhantomData;

// State markers
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


// build() only available on Complete state

impl ConfigBuilder<Complete> {
    fn build(self) -> Config {
        Config {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
        }
    }
}

struct Config {
    host: String,
    port: u16,
}

=====
// Compile-time error if required fields missing
=====
// let config = ConfigBuilder::new().build(); // Error: no method `build`
let config = ConfigBuilder::new()
    .host("localhost".to_string())
    .port(8080)
    .build();  // OK
```

**Trade-offs**:
- **Pros**: Ergonomic API, clear intent, optional parameters, compile-time validation with typestate
- **Cons**: More boilerplate, separate builder type, consumes self (can't reuse builder)

**When to use**: Complex initialization, many optional parameters, validation requirements, library APIs

---

### Factory Pattern

**Intent**: Define an interface for creating objects, but let subclasses or implementors decide which concrete type to instantiate.

**Motivation**: When the exact type of object to create isn't known until runtime, or when creation logic needs to be abstracted, factories encapsulate instantiation. In Rust, this typically uses trait objects or enums rather than inheritance.

```rust
// Abstract product
trait Button {
    fn render(&self) -> String;
    fn on_click(&self);
}

// Concrete products
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

// Factory interface
trait UIFactory {
    fn create_button(&self) -> Box<dyn Button>;
}

// Concrete factories
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

// Client code
fn render_ui(factory: &dyn UIFactory) {
    let button = factory.create_button();
    println!("{}", button.render());
    button.on_click();
}

let factory: Box<dyn UIFactory> = if cfg!(target_os = "windows") {
    Box::new(WindowsFactory)
} else {
    Box::new(MacFactory)
};
render_ui(&*factory);
```

**Rust idiom: Enum-based factory (zero-cost)**

When the set of types is closed, enums provide a zero-cost alternative:

```rust
enum Button {
    Windows(WindowsButton),
    Mac(MacButton),
}

impl Button {
    fn new(platform: Platform) -> Self {
        match platform {
            Platform::Windows => Button::Windows(WindowsButton),
            Platform::Mac => Button::Mac(MacButton),
        }
    }

    fn render(&self) -> String {
        match self {
            Button::Windows(btn) => btn.render(),
            Button::Mac(btn) => btn.render(),
        }
    }
}

enum Platform {
    Windows,
    Mac,
}

// No heap allocation, no dynamic dispatch
let button = Button::new(Platform::Windows);
```

**Trade-offs**:
- **Trait objects**: Runtime polymorphism, heap allocation, open for extension
- **Enums**: Compile-time dispatch, zero-cost, closed set of types

**When to use**: Plugin systems, platform abstraction, testing (mock factories), runtime type selection

---

### Singleton Pattern

**Intent**: Ensure a class has only one instance and provide a global point of access to it.

**Motivation**: Some resources should exist only once: configuration, connection pools, logging systems. Rust's ownership model makes traditional singletons challenging, but several idiomatic alternatives exist.

**Rust approach: Static with lazy initialization**

```rust
use std::sync::OnceLock;

struct AppConfig {
    api_key: String,
    debug_mode: bool,
}

impl AppConfig {
    fn global() -> &'static AppConfig {
        static CONFIG: OnceLock<AppConfig> = OnceLock::new();
        CONFIG.get_or_init(|| {
            AppConfig {
                api_key: std::env::var("API_KEY").unwrap_or_default(),
                debug_mode: cfg!(debug_assertions),
            }
        })
    }
}

let config = AppConfig::global();
println!("API Key: {}", config.api_key);
```

**Alternative: Dependency injection (preferred)**

Rather than global state, pass dependencies explicitly:

```rust
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
}

========
// Explicit dependencies, testable, no global state
========
let db = Database::new("postgres://localhost".to_string());
let service = UserService::new(&db);
```

**Trade-offs**:
- **OnceLock**: Simple, thread-safe, but global mutable state is discouraged in Rust
- **Dependency injection**: More flexible, testable, but requires passing dependencies
- **Avoid**: Lazy static crates if OnceLock suffices (std library is preferred)

**When to use**: Truly global resources (logging, metrics), avoid when possible in favor of dependency injection

---

### Prototype Pattern

**Intent**: Create new objects by cloning existing instances rather than constructing from scratch.

**Motivation**: When object creation is expensive (parsing, network calls, complex initialization), cloning an existing instance can be faster. Rust's `Clone` trait makes this pattern native to the language.

```rust
use std::collections::HashMap;

#[derive(Clone)]
struct TemplateEngine {
    templates: HashMap<String, String>,
    config: Config,
}

#[derive(Clone)]
struct Config {
    strict_mode: bool,
    cache_enabled: bool,
}

impl TemplateEngine {
    fn new() -> Self {
        let mut templates = HashMap::new();
        // Expensive initialization
        templates.insert("header".to_string(), load_template("header.html"));
        templates.insert("footer".to_string(), load_template("footer.html"));

        Self {
            templates,
            config: Config {
                strict_mode: true,
                cache_enabled: true,
            },
        }
    }

    fn with_different_config(&self, config: Config) -> Self {
        let mut cloned = self.clone();
        cloned.config = config;
        cloned  // Reuses expensive template loading
    }
}

fn load_template(_name: &str) -> String {
    // Expensive operation
    String::from("template content")
}

let base_engine = TemplateEngine::new();  // Expensive

// Cheap clones with variations
let dev_engine = base_engine.with_different_config(Config {
    strict_mode: false,
    cache_enabled: false,
});
let prod_engine = base_engine.clone();  // Reuses all data
```

**Deep vs shallow cloning**

```rust
use std::rc::Rc;

#[derive(Clone)]
struct SharedData {
    // Shallow clone: reference counted
    cache: Rc<Vec<u8>>,
    // Deep clone: clones the String
    user_id: String,
}

let original = SharedData {
    cache: Rc::new(vec![1, 2, 3]),
    user_id: "user123".to_string(),
};

let cloned = original.clone();
===
// cache is shared (reference count increased)
===
// user_id is copied
assert_eq!(Rc::strong_count(&original.cache), 2);
```

**Trade-offs**:
- **Pros**: Reduces initialization cost, isolates complex object creation
- **Cons**: Deep cloning can be expensive, shared state with Rc requires care

**When to use**: Expensive initialization, template objects, copy-on-write scenarios

---

## Structural Patterns

Structural patterns compose objects and types to form larger structures while keeping the system flexible and efficient. Rust's trait system and zero-cost abstractions enable elegant structural compositions without runtime overhead.

### Adapter Pattern

**Intent**: Convert the interface of a type into another interface that clients expect, allowing incompatible interfaces to work together.

**Motivation**: When integrating third-party libraries or legacy code, interfaces often don't match your requirements. Adapters wrap existing types to provide the interface you need without modifying the original.

```rust
// Target interface your code expects
trait MediaPlayer {
    fn play(&self, filename: &str);
}

=============
// Existing third-party library with different interface
=============
struct VlcPlayer;
impl VlcPlayer {
    fn play_vlc(&self, file_path: &str) {
        println!("Playing VLC: {}", file_path);
    }
}

struct Mp3Player;
impl Mp3Player {
    fn play_mp3(&self, file_name: &str) {
        println!("Playing MP3: {}", file_name);
    }
}

// Adapters to make them compatible
struct VlcAdapter {
    player: VlcPlayer,
}

impl MediaPlayer for VlcAdapter {
    fn play(&self, filename: &str) {
        self.player.play_vlc(filename);
    }
}

struct Mp3Adapter {
    player: Mp3Player,
}

impl MediaPlayer for Mp3Adapter {
    fn play(&self, filename: &str) {
        self.player.play_mp3(filename);
    }
}


// Client code works with uniform interface

fn play_media(player: &dyn MediaPlayer, file: &str) {
    player.play(file);
}

let vlc = VlcAdapter { player: VlcPlayer };
let mp3 = Mp3Adapter { player: Mp3Player };
play_media(&vlc, "video.mp4");
play_media(&mp3, "song.mp3");
```

**Zero-cost adapter with generics**

```rust
struct GenericAdapter<T> {
    inner: T,
}

trait PlayVlc {
    fn play_vlc(&self, path: &str);
}

impl<T: PlayVlc> MediaPlayer for GenericAdapter<T> {
    fn play(&self, filename: &str) {
        self.inner.play_vlc(filename);
    }
}

===============
// No trait object overhead, monomorphized at compile-time
===============
```

**Trade-offs**:
- **Pros**: Preserves single responsibility, enables interface compatibility
- **Cons**: Additional layer of indirection, potential heap allocation with trait objects

**When to use**: Third-party integration, legacy code adaptation, interface standardization

---

### Decorator Pattern

**Intent**: Attach additional responsibilities to an object dynamically, providing a flexible alternative to subclassing for extending functionality.

**Motivation**: When you need to add behavior to individual objects without affecting other instances, decorators wrap objects with new capabilities. In Rust, this often uses trait objects or generic wrappers.

```rust
trait DataSource {
    fn read(&self) -> String;
    fn write(&mut self, data: &str);
}

// Concrete component
struct FileDataSource {
    filename: String,
    contents: String,
}

impl DataSource for FileDataSource {
    fn read(&self) -> String {
        self.contents.clone()
    }

    fn write(&mut self, data: &str) {
        self.contents = data.to_string();
    }
}

// Decorator: Encryption
struct EncryptionDecorator {
    wrapped: Box<dyn DataSource>,
}

impl DataSource for EncryptionDecorator {
    fn read(&self) -> String {
        let encrypted = self.wrapped.read();
        decrypt(&encrypted)  // Add decryption behavior
    }

    fn write(&mut self, data: &str) {
        let encrypted = encrypt(data);  // Add encryption behavior
        self.wrapped.write(&encrypted);
    }
}

// Decorator: Compression
struct CompressionDecorator {
    wrapped: Box<dyn DataSource>,
}

impl DataSource for CompressionDecorator {
    fn read(&self) -> String {
        let compressed = self.wrapped.read();
        decompress(&compressed)
    }

    fn write(&mut self, data: &str) {
        let compressed = compress(data);
        self.wrapped.write(&compressed);
    }
}

fn encrypt(data: &str) -> String { format!("encrypted({})", data) }
fn decrypt(data: &str) -> String { data.trim_start_matches("encrypted(").trim_end_matches(')').to_string() }
fn compress(data: &str) -> String { format!("compressed({})", data) }
fn decompress(data: &str) -> String { data.trim_start_matches("compressed(").trim_end_matches(')').to_string() }

let file = FileDataSource {
    filename: "data.txt".to_string(),
    contents: "sensitive data".to_string(),
};

let mut source: Box<dyn DataSource> = Box::new(file);
source = Box::new(EncryptionDecorator { wrapped: source });
source = Box::new(CompressionDecorator { wrapped: source });

source.write("secret");
// Writes: compressed(encrypted(secret))
```

**Type-safe decorator with generics**

```rust
struct Encrypted<T>(T);
struct Compressed<T>(T);

trait Read {
    fn read(&self) -> String;
}

impl Read for String {
    fn read(&self) -> String {
        self.clone()
    }
}

impl<T: Read> Read for Encrypted<T> {
    fn read(&self) -> String {
        decrypt(&self.0.read())
    }
}

impl<T: Read> Read for Compressed<T> {
    fn read(&self) -> String {
        decompress(&self.0.read())
    }
}

// Compile-time composition, zero-cost
let data = String::from("data");
let secure = Compressed(Encrypted(data));
println!("{}", secure.read());
```

**Trade-offs**:
- **Trait objects**: Runtime flexibility, heap allocation
- **Generic wrappers**: Compile-time composition, zero-cost, but fixed at compile-time

**When to use**: Logging wrappers, encryption/compression layers, middleware, cross-cutting concerns

---

### Facade Pattern

**Intent**: Provide a unified, simplified interface to a complex subsystem, making the subsystem easier to use.

**Motivation**: Complex libraries or modules can have dozens of types and methods. A facade presents a clean, high-level API that hides the complexity, making common use cases simple while still allowing access to advanced features when needed.

```rust
// Complex subsystem
mod video_processing {
    pub struct VideoDecoder;
    impl VideoDecoder {
        pub fn decode(&self, _file: &str) -> Vec<u8> {
            println!("Decoding video...");
            vec![1, 2, 3]
        }
    }

    pub struct AudioExtractor;
    impl AudioExtractor {
        pub fn extract(&self, _data: &[u8]) -> Vec<u8> {
            println!("Extracting audio...");
            vec![4, 5, 6]
        }
    }

    pub struct CodecManager;
    impl CodecManager {
        pub fn configure(&self) {
            println!("Configuring codecs...");
        }
    }

    pub struct FormatConverter;
    impl FormatConverter {
        pub fn convert(&self, _data: &[u8], _format: &str) -> String {
            println!("Converting format...");
            "output.mp4".to_string()
        }
    }
}

// Facade: Simple interface
struct VideoConverter {
    decoder: video_processing::VideoDecoder,
    audio: video_processing::AudioExtractor,
    codec: video_processing::CodecManager,
    converter: video_processing::FormatConverter,
}

impl VideoConverter {
    fn new() -> Self {
        Self {
            decoder: video_processing::VideoDecoder,
            audio: video_processing::AudioExtractor,
            codec: video_processing::CodecManager,
            converter: video_processing::FormatConverter,
        }
    }

    // Simple API for common use case
    fn convert_to_mp4(&self, filename: &str) -> String {
        self.codec.configure();
        let video_data = self.decoder.decode(filename);
        let audio_data = self.audio.extract(&video_data);
        self.converter.convert(&audio_data, "mp4")
    }
}

// Client code: Simple and clean
let converter = VideoConverter::new();
let output = converter.convert_to_mp4("input.avi");
println!("Converted: {}", output);
```

**Trade-offs**:
- **Pros**: Simplifies complex APIs, reduces coupling, easier testing
- **Cons**: May hide useful functionality, another layer of abstraction

**When to use**: Complex third-party libraries, legacy system integration, API simplification

---

### Newtype Pattern

**Intent**: Create a distinct type by wrapping an existing type in a single-field struct, enabling type safety and trait implementation.

**Motivation**: Type aliases provide no safety—a `UserId` alias for `u64` can be accidentally mixed with `ProductId`. Newtypes create distinct types at compile-time with zero runtime cost, preventing bugs and enabling custom trait implementations.

```rust
==
// Problem: Type aliases don't prevent mixing
==
type Meters = f64;
type Feet = f64;

fn calculate_distance(m: Meters) -> Meters { m * 2.0 }

let feet: Feet = 10.0;
let result = calculate_distance(feet);  // Compiles but wrong!

// Solution: Newtype pattern
#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Feet(f64);

impl Meters {
    fn new(value: f64) -> Self {
        Meters(value)
    }

    fn to_feet(self) -> Feet {
        Feet(self.0 * 3.28084)
    }
}

impl Feet {
    fn new(value: f64) -> Self {
        Feet(value)
    }

    fn to_meters(self) -> Meters {
        Meters(self.0 / 3.28084)
    }
}

fn calculate_distance_safe(m: Meters) -> Meters {
    Meters(m.0 * 2.0)
}

let feet = Feet::new(10.0);
======================
// let result = calculate_distance_safe(feet);  // Compile error!
======================
let meters = feet.to_meters();
let result = calculate_distance_safe(meters);  // OK
```

**Implementing external traits**

Rust's orphan rule prevents implementing external traits for external types, but newtypes provide a workaround:

```rust
use std::fmt;

===================
// Can't implement Display for Vec<i32> directly (orphan rule)
===================
// impl fmt::Display for Vec<i32> { }  // Error!

// Newtype enables custom implementation
struct IntList(Vec<i32>);

impl fmt::Display for IntList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

let list = IntList(vec![1, 2, 3]);
println!("{}", list);  // [1, 2, 3]
```

**Trade-offs**:
- **Pros**: Zero-cost type safety, enables trait implementations, prevents mixing incompatible values
- **Cons**: Requires explicit construction/extraction, more verbose than type aliases

**When to use**: Domain modeling (UserId, Email), unit types (Meters, Seconds), orphan rule workaround

---

## Behavioral Patterns

Behavioral patterns define how objects interact and distribute responsibility. They describe not just patterns of objects or classes, but also the patterns of communication between them. Rust's ownership model influences these patterns significantly.

### Strategy Pattern

**Intent**: Define a family of algorithms, encapsulate each one, and make them interchangeable. Strategy lets the algorithm vary independently from clients that use it.

**Motivation**: When multiple algorithms exist for a specific task, strategy pattern allows selecting the algorithm at runtime. In Rust, this uses trait objects for runtime polymorphism or generics for compile-time selection.

```rust
// Strategy trait
trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
    fn decompress(&self, data: &[u8]) -> Vec<u8>;
}

// Concrete strategies
struct ZipCompression;
impl CompressionStrategy for ZipCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        println!("ZIP compressing {} bytes", data.len());
        data.to_vec()  // Simplified
    }

    fn decompress(&self, data: &[u8]) -> Vec<u8> {
        println!("ZIP decompressing {} bytes", data.len());
        data.to_vec()
    }
}

struct RarCompression;
impl CompressionStrategy for RarCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        println!("RAR compressing {} bytes", data.len());
        data.to_vec()
    }

    fn decompress(&self, data: &[u8]) -> Vec<u8> {
        println!("RAR decompressing {} bytes", data.len());
        data.to_vec()
    }
}

// Context
struct FileCompressor {
    strategy: Box<dyn CompressionStrategy>,
}

impl FileCompressor {
    fn new(strategy: Box<dyn CompressionStrategy>) -> Self {
        Self { strategy }
    }

    fn set_strategy(&mut self, strategy: Box<dyn CompressionStrategy>) {
        self.strategy = strategy;
    }

    fn compress_file(&self, data: &[u8]) -> Vec<u8> {
        self.strategy.compress(data)
    }
}

let data = vec![1, 2, 3, 4, 5];
let mut compressor = FileCompressor::new(Box::new(ZipCompression));
compressor.compress_file(&data);

compressor.set_strategy(Box::new(RarCompression));
compressor.compress_file(&data);
```

**Zero-cost strategy with generics**

```rust
struct StaticCompressor<S> {
    strategy: S,
}

impl<S: CompressionStrategy> StaticCompressor<S> {
    fn new(strategy: S) -> Self {
        Self { strategy }
    }

    fn compress_file(&self, data: &[u8]) -> Vec<u8> {
        self.strategy.compress(data)
    }
}

===========
// Compile-time strategy selection, no heap allocation
===========
let compressor = StaticCompressor::new(ZipCompression);
compressor.compress_file(&data);
```

**Functional strategy with closures**

```rust
struct FunctionalCompressor<F>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    compress_fn: F,
}

impl<F> FunctionalCompressor<F>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    fn new(compress_fn: F) -> Self {
        Self { compress_fn }
    }

    fn compress(&self, data: &[u8]) -> Vec<u8> {
        (self.compress_fn)(data)
    }
}

// Strategy as closure
let zip_fn = |data: &[u8]| -> Vec<u8> {
    println!("ZIP compressing");
    data.to_vec()
};
let compressor = FunctionalCompressor::new(zip_fn);
```

**Trade-offs**:
- **Trait objects**: Runtime flexibility, heap allocation, dynamic dispatch
- **Generics**: Zero-cost, compile-time selection, monomorphization
- **Closures**: Concise, captures environment, good for simple strategies

**When to use**: Multiple algorithms, runtime selection needed, testability (inject mock strategies)

---

### Observer Pattern

**Intent**: Define a one-to-many dependency between objects so that when one object changes state, all its dependents are notified automatically.

**Motivation**: GUIs, event systems, and reactive programming need to notify multiple listeners when state changes. Rust's ownership makes traditional observer patterns challenging, but channels and callbacks provide idiomatic solutions.

```rust
use std::sync::{Arc, Mutex};

// Observer trait
trait Observer {
    fn update(&mut self, temperature: f32);
}

// Concrete observers
struct TemperatureDisplay {
    name: String,
}

impl Observer for TemperatureDisplay {
    fn update(&mut self, temperature: f32) {
        println!("{} display: {}°C", self.name, temperature);
    }
}

struct TemperatureLogger {
    log: Vec<f32>,
}

impl Observer for TemperatureLogger {
    fn update(&mut self, temperature: f32) {
        self.log.push(temperature);
        println!("Logged: {}°C (total: {} readings)", temperature, self.log.len());
    }
}

// Subject
struct WeatherStation {
    temperature: f32,
    observers: Vec<Arc<Mutex<dyn Observer + Send>>>,
}

impl WeatherStation {
    fn new() -> Self {
        Self {
            temperature: 0.0,
            observers: Vec::new(),
        }
    }

    fn attach(&mut self, observer: Arc<Mutex<dyn Observer + Send>>) {
        self.observers.push(observer);
    }

    fn set_temperature(&mut self, temp: f32) {
        self.temperature = temp;
        self.notify();
    }

    fn notify(&self) {
        for observer in &self.observers {
            observer.lock().unwrap().update(self.temperature);
        }
    }
}

let mut station = WeatherStation::new();

let display = Arc::new(Mutex::new(TemperatureDisplay {
    name: "Main".to_string(),
}));
let logger = Arc::new(Mutex::new(TemperatureLogger { log: Vec::new() }));

station.attach(display);
station.attach(logger);

station.set_temperature(25.5);
station.set_temperature(26.0);
```

**Channel-based observer (more idiomatic)**

```rust
use std::sync::mpsc;
use std::thread;

struct Event {
    temperature: f32,
}

// Publisher
struct Publisher {
    subscribers: Vec<mpsc::Sender<Event>>,
}

impl Publisher {
    fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    fn subscribe(&mut self) -> mpsc::Receiver<Event> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    fn publish(&self, event: Event) {
        self.subscribers.retain(|tx| tx.send(event.clone()).is_ok());
    }
}

#[derive(Clone)]
struct Event {
    temperature: f32,
}

let mut publisher = Publisher::new();

let rx1 = publisher.subscribe();
let rx2 = publisher.subscribe();

thread::spawn(move || {
    for event in rx1 {
        println!("Observer 1: {}°C", event.temperature);
    }
});

thread::spawn(move || {
    for event in rx2 {
        println!("Observer 2: {}°C", event.temperature);
    }
});

publisher.publish(Event { temperature: 25.5 });
```

**Trade-offs**:
- **Classic observer**: Familiar pattern, but requires Arc<Mutex<_>> for shared mutable state
- **Channels**: More idiomatic in Rust, natural parallelism, no shared state
- **Callbacks**: Simple for single-threaded scenarios

**When to use**: Event systems, GUIs, reactive programming, pub/sub architectures

---

### Command Pattern

**Intent**: Encapsulate a request as an object, allowing you to parameterize clients with different requests, queue or log requests, and support undoable operations.

**Motivation**: When you need to decouple the object that invokes an operation from the one that performs it, commands encapsulate all information needed to execute an action. This enables undo/redo, macros, and request queuing.

```rust
// Command trait
trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}

// Receiver
struct TextEditor {
    content: String,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn write(&mut self, text: &str) {
        self.content.push_str(text);
    }

    fn delete_last(&mut self, count: usize) {
        let new_len = self.content.len().saturating_sub(count);
        self.content.truncate(new_len);
    }

    fn get_content(&self) -> &str {
        &self.content
    }
}

// Concrete commands
struct WriteCommand {
    editor: std::rc::Rc<std::cell::RefCell<TextEditor>>,
    text: String,
}

impl Command for WriteCommand {
    fn execute(&mut self) {
        self.editor.borrow_mut().write(&self.text);
    }

    fn undo(&mut self) {
        self.editor.borrow_mut().delete_last(self.text.len());
    }
}

struct DeleteCommand {
    editor: std::rc::Rc<std::cell::RefCell<TextEditor>>,
    deleted_text: String,
    count: usize,
}

impl Command for DeleteCommand {
    fn execute(&mut self) {
        let editor = self.editor.borrow();
        let content = editor.get_content();
        let start = content.len().saturating_sub(self.count);
        self.deleted_text = content[start..].to_string();
        drop(editor);

        self.editor.borrow_mut().delete_last(self.count);
    }

    fn undo(&mut self) {
        self.editor.borrow_mut().write(&self.deleted_text);
    }
}

// Invoker
struct CommandHistory {
    history: Vec<Box<dyn Command>>,
    current: usize,
}

impl CommandHistory {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            current: 0,
        }
    }

    fn execute(&mut self, mut command: Box<dyn Command>) {
        command.execute();
        // Discard any undone commands
        self.history.truncate(self.current);
        self.history.push(command);
        self.current += 1;
    }

    fn undo(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.history[self.current].undo();
        }
    }

    fn redo(&mut self) {
        if self.current < self.history.len() {
            self.history[self.current].execute();
            self.current += 1;
        }
    }
}

use std::rc::Rc;
use std::cell::RefCell;

let editor = Rc::new(RefCell::new(TextEditor::new()));
let mut history = CommandHistory::new();

history.execute(Box::new(WriteCommand {
    editor: editor.clone(),
    text: "Hello ".to_string(),
}));
history.execute(Box::new(WriteCommand {
    editor: editor.clone(),
    text: "World".to_string(),
}));

println!("{}", editor.borrow().get_content());  // "Hello World"

history.undo();
println!("{}", editor.borrow().get_content());  // "Hello "

history.redo();
println!("{}", editor.borrow().get_content());  // "Hello World"
```

**Functional command pattern**

```rust
struct FunctionalCommand {
    execute_fn: Box<dyn FnMut()>,
    undo_fn: Box<dyn FnMut()>,
}

impl FunctionalCommand {
    fn new(execute_fn: Box<dyn FnMut()>, undo_fn: Box<dyn FnMut()>) -> Self {
        Self { execute_fn, undo_fn }
    }

    fn execute(&mut self) {
        (self.execute_fn)();
    }

    fn undo(&mut self) {
        (self.undo_fn)();
    }
}
```

**Trade-offs**:
- **Pros**: Decouples invoker from receiver, enables undo/redo, command queuing, macros
- **Cons**: More objects, requires shared mutable state (Rc<RefCell<_>>)

**When to use**: Undo/redo systems, transaction systems, job queues, macro recording

---

### Iterator Pattern

**Intent**: Provide a way to access elements of a collection sequentially without exposing the underlying representation.

**Motivation**: Rust has first-class support for the iterator pattern through the `Iterator` trait. This is the most idiomatic way to traverse collections and is deeply integrated into the language.

```rust
// Custom iterator
struct Fibonacci {
    current: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Self { current: 0, next: 1 }
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        self.current = self.next;
        self.next = current + self.next;
        Some(current)
    }
}

==
==
let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
println!("{:?}", fibs);  // [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

**Implementing IntoIterator for custom collections**

```rust
struct MyCollection {
    items: Vec<String>,
}

impl MyCollection {
    fn new() -> Self {
        Self { items: Vec::new() }
    }

    fn add(&mut self, item: String) {
        self.items.push(item);
    }
}

// Owned iterator
impl IntoIterator for MyCollection {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// Borrowed iterator
impl<'a> IntoIterator for &'a MyCollection {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

let mut collection = MyCollection::new();
collection.add("Item 1".to_string());
collection.add("Item 2".to_string());

for item in &collection {
    println!("{}", item);
}
// collection still valid

for item in collection {
    println!("{}", item);
}
// collection moved
```

**Trade-offs**:
- **Pros**: Lazy evaluation, composable, integrates with language, zero-cost abstractions
- **Cons**: None—this is the idiomatic Rust approach

**When to use**: Always, for any sequential access. This is a fundamental Rust pattern.

---

## Concurrency Patterns

Concurrency patterns address the challenges of parallel and asynchronous execution. Rust's ownership system prevents data races at compile-time, enabling fearless concurrency. These patterns leverage threads, async/await, and synchronization primitives.

### Thread Pool Pattern

**Intent**: Manage a pool of worker threads to execute tasks efficiently, amortizing thread creation cost and limiting resource usage.

**Motivation**: Creating a thread per task is expensive. Thread pools maintain a fixed number of workers that process tasks from a queue, improving throughput and controlling concurrency.

```rust
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect();

        ThreadPool { workers, sender }
    }

    fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.sender.send(Box::new(job)).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Self {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();

            match job {
                Ok(job) => {
                    println!("Worker {} executing job", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} shutting down", id);
                    break;
                }
            }
        });

        Worker { id, thread }
    }
}

let pool = ThreadPool::new(4);

for i in 0..10 {
    pool.execute(move || {
        println!("Task {} running", i);
        thread::sleep(std::time::Duration::from_millis(100));
    });
}
```

**Using rayon for data parallelism**

```rust
use rayon::prelude::*;

================
// Parallel iterator (much simpler than manual thread pool)
================
let numbers: Vec<i32> = (0..1000).collect();
let sum: i32 = numbers.par_iter().map(|&x| x * 2).sum();
```

**Trade-offs**:
- **Manual thread pool**: Full control, custom scheduling, but complex implementation
- **Rayon**: Simple, efficient, but less control over task scheduling
- **Tokio runtime**: For async I/O tasks, not CPU-bound work

**When to use**: CPU-bound parallel tasks, limiting concurrency, long-running workers

---

### Producer-Consumer Pattern

**Intent**: Decouple producers that generate data from consumers that process it, using a queue as a buffer.

**Motivation**: When production and consumption happen at different rates, a queue prevents blocking and enables concurrent processing. Rust's channels provide a natural implementation.

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Producer
fn producer(tx: mpsc::Sender<i32>) {
    for i in 0..10 {
        println!("Producing {}", i);
        tx.send(i).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
    // Channel closes when tx is dropped
}

// Consumer
fn consumer(rx: mpsc::Receiver<i32>) {
    for item in rx {
        println!("  Consuming {}", item);
        thread::sleep(Duration::from_millis(200));  // Slower than producer
    }
}

let (tx, rx) = mpsc::channel();

thread::spawn(move || producer(tx));
thread::spawn(move || consumer(rx));

thread::sleep(Duration::from_secs(3));
```

**Multiple producers, single consumer**

```rust
let (tx, rx) = mpsc::channel();

for id in 0..3 {
    let tx_clone = tx.clone();
    thread::spawn(move || {
        for i in 0..5 {
            tx_clone.send((id, i)).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });
}
drop(tx);  // Drop original to allow channel to close

thread::spawn(move || {
    for (id, item) in rx {
        println!("Producer {} sent {}", id, item);
    }
});
```

**Bounded channel for backpressure**

```rust
use std::sync::mpsc::sync_channel;

let (tx, rx) = sync_channel(3);  // Buffer size of 3

// Producer blocks if queue is full
thread::spawn(move || {
    for i in 0..10 {
        println!("Sending {}", i);
        tx.send(i).unwrap();  // Blocks if buffer full
    }
});

thread::spawn(move || {
    thread::sleep(Duration::from_secs(1));  // Delay consumer
    for item in rx {
        println!("Received {}", item);
    }
});
```

**Trade-offs**:
- **Unbounded channel**: No backpressure, can cause unbounded memory growth
- **Bounded channel**: Backpressure control, but producers may block
- **Crossbeam**: More efficient channels, select! support

**When to use**: Pipeline architectures, async work queues, rate limiting, buffering

---

### Fork-Join Pattern

**Intent**: Split a task into subtasks that can run in parallel, then join the results.

**Motivation**: Divide-and-conquer algorithms benefit from parallel execution. Fork-join splits work across threads and combines results.

```rust
use std::thread;

fn parallel_sum(data: &[i32]) -> i32 {
    const THRESHOLD: usize = 100;

    if data.len() <= THRESHOLD {
        // Base case: sequential sum
        data.iter().sum()
    } else {
        // Fork: split into two halves
        let mid = data.len() / 2;
        let (left, right) = data.split_at(mid);

        let left_data = left.to_vec();
        let handle = thread::spawn(move || parallel_sum(&left_data));

        let right_sum = parallel_sum(right);
        let left_sum = handle.join().unwrap();

        // Join: combine results
        left_sum + right_sum
    }
}

let data: Vec<i32> = (1..=1000).collect();
let total = parallel_sum(&data);
println!("Sum: {}", total);
```

**Using rayon for automatic fork-join**

```rust
use rayon::prelude::*;

fn rayon_sum(data: &[i32]) -> i32 {
    data.par_iter().sum()  // Automatically parallelizes
}

// Parallel recursion with rayon
fn quicksort<T: Ord + Send>(mut data: Vec<T>) -> Vec<T> {
    if data.len() <= 1 {
        return data;
    }

    let pivot = data.remove(0);
    let (mut less, mut greater): (Vec<_>, Vec<_>) = data
        .into_par_iter()  // Parallel partition
        .partition(|x| x < &pivot);

    // Parallel recursive calls
    let (sorted_less, sorted_greater) = rayon::join(
        || quicksort(less),
        || quicksort(greater),
    );

    let mut result = sorted_less;
    result.push(pivot);
    result.extend(sorted_greater);
    result
}
```

**Trade-offs**:
- **Manual fork-join**: Full control, but must manage thread creation
- **Rayon**: Automatic work-stealing, simpler, efficient

**When to use**: Divide-and-conquer algorithms, parallel recursion, data parallelism

---

### Actor Pattern

**Intent**: Encapsulate state and behavior in isolated actors that communicate only through message passing.

**Motivation**: Shared mutable state is the root of concurrency bugs. Actors eliminate shared state by giving each actor exclusive ownership of its data, communicating only via messages.

```rust
use std::sync::mpsc;
use std::thread;

// Message types
enum AccountMessage {
    Deposit(u64),
    Withdraw(u64),
    GetBalance(mpsc::Sender<u64>),
    Shutdown,
}

// Actor
struct BankAccount {
    balance: u64,
    receiver: mpsc::Receiver<AccountMessage>,
}

impl BankAccount {
    fn new(initial: u64) -> (Self, mpsc::Sender<AccountMessage>) {
        let (tx, rx) = mpsc::channel();
        let account = BankAccount {
            balance: initial,
            receiver: rx,
        };
        (account, tx)
    }

    fn run(mut self) {
        thread::spawn(move || {
            for msg in self.receiver {
                match msg {
                    AccountMessage::Deposit(amount) => {
                        self.balance += amount;
                        println!("Deposited {}. Balance: {}", amount, self.balance);
                    }
                    AccountMessage::Withdraw(amount) => {
                        if self.balance >= amount {
                            self.balance -= amount;
                            println!("Withdrew {}. Balance: {}", amount, self.balance);
                        } else {
                            println!("Insufficient funds");
                        }
                    }
                    AccountMessage::GetBalance(reply) => {
                        reply.send(self.balance).unwrap();
                    }
                    AccountMessage::Shutdown => {
                        println!("Shutting down account");
                        break;
                    }
                }
            }
        });
    }
}


let (account, handle) = BankAccount::new(100);
account.run();

handle.send(AccountMessage::Deposit(50)).unwrap();
handle.send(AccountMessage::Withdraw(30)).unwrap();

let (tx, rx) = mpsc::channel();
handle.send(AccountMessage::GetBalance(tx)).unwrap();
let balance = rx.recv().unwrap();
println!("Final balance: {}", balance);

handle.send(AccountMessage::Shutdown).unwrap();
```

**Using Actix framework**

```rust
// With actix-rt (external crate)
// Much more ergonomic for complex actor systems
/*
use actix::prelude::*;

struct BankAccount {
    balance: u64,
}

impl Actor for BankAccount {
    type Context = Context<Self>;
}

struct Deposit(u64);
impl Message for Deposit {
    type Result = ();
}

impl Handler<Deposit> for BankAccount {
    type Result = ();

    fn handle(&mut self, msg: Deposit, _ctx: &mut Context<Self>) {
        self.balance += msg.0;
    }
}
*/
```

**Trade-offs**:
- **Pros**: No shared state, natural concurrency model, isolated failures
- **Cons**: Message passing overhead, complexity for simple cases
- **Frameworks**: Actix provides full-featured actor system

**When to use**: Stateful services, concurrent servers, distributed systems, isolation requirements

---

### Async/Await Pattern

**Intent**: Write asynchronous code that looks like synchronous code, improving readability while maintaining non-blocking execution.

**Motivation**: Callback-based async code becomes nested and hard to follow. Async/await provides sequential syntax for asynchronous operations, compiling to efficient state machines.

```rust
use tokio;
use std::time::Duration;

async fn fetch_user(id: u64) -> String {
    // Simulated async operation
    tokio::time::sleep(Duration::from_millis(100)).await;
    format!("User {}", id)
}

async fn fetch_posts(user: &str) -> Vec<String> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    vec![
        format!("{}'s post 1", user),
        format!("{}'s post 2", user),
    ]
}

async fn display_user_data(id: u64) {
    // Sequential async operations
    let user = fetch_user(id).await;
    println!("Fetched: {}", user);

    let posts = fetch_posts(&user).await;
    for post in posts {
        println!("  - {}", post);
    }
}

#[tokio::main]
async fn main() {
    display_user_data(1).await;
}
```

**Parallel async operations with join!**

```rust
use tokio;

async fn parallel_fetch() {
    // Wait for all to complete
    let (user1, user2, user3) = tokio::join!(
        fetch_user(1),
        fetch_user(2),
        fetch_user(3),
    );

    println!("{}, {}, {}", user1, user2, user3);
}
```

**Select pattern: First to complete**

```rust
async fn timeout_example() {
    let fetch = fetch_user(1);
    let timeout = tokio::time::sleep(Duration::from_millis(50));

    tokio::select! {
        user = fetch => println!("Got user: {}", user),
        _ = timeout => println!("Timeout!"),
    }
}
```

**Trade-offs**:
- **Pros**: Readable async code, efficient (single-threaded event loop), scales to many connections
- **Cons**: Runtime dependency (tokio/async-std), learning curve, colored functions
- **vs Threads**: Async for I/O-bound, threads for CPU-bound

**When to use**: Web servers, network services, I/O-heavy applications, high concurrency with low CPU usage

---

### Summary

Design patterns in Rust take unique forms due to the language's ownership model, type system, and zero-cost abstractions. Many patterns that require runtime polymorphism in object-oriented languages can be implemented at compile-time in Rust through generics and traits, eliminating overhead.

### Key Takeaways

**Creational patterns** (Builder, Factory, Singleton, Prototype) benefit from Rust's type system:
- Builders enable fluent APIs; typestate pattern enforces correctness at compile-time
- Factories use traits or enums; enums provide zero-cost closed variants
- Singletons use `OnceLock`; prefer dependency injection when possible
- Prototypes leverage `Clone` trait, a first-class Rust concept

**Structural patterns** (Adapter, Decorator, Facade, Newtype) compose types efficiently:
- Adapters bridge incompatible interfaces; generics eliminate runtime cost
- Decorators add behavior; trait objects enable runtime composition, generics enable compile-time
- Facades simplify complex subsystems
- Newtypes provide zero-cost type safety and orphan rule workarounds

**Behavioral patterns** (Strategy, Observer, Command, Iterator) define interactions:
- Strategies use traits; closures provide functional alternative
- Observers use channels rather than callbacks; more idiomatic and thread-safe
- Commands enable undo/redo; require shared mutable state (Rc<RefCell<_>>)
- Iterators are first-class in Rust; the Iterator trait is ubiquitous

**Concurrency patterns** (Thread Pool, Producer-Consumer, Fork-Join, Actor, Async/Await) leverage Rust's fearless concurrency:
- Thread pools manage worker threads; rayon simplifies data parallelism
- Producer-consumer uses channels naturally; bounded channels provide backpressure
- Fork-join parallelizes divide-and-conquer; rayon automates work-stealing
- Actors eliminate shared state through message passing
- Async/await provides readable asynchronous code for I/O-bound tasks

### Choosing the Right Pattern

When facing a design decision:

1. **Prefer zero-cost abstractions**: Use generics over trait objects when types are known at compile-time
2. **Embrace ownership**: Design with moves, borrows, and lifetimes rather than fighting them
3. **Use standard traits**: Implement `Iterator`, `From`, `Display` rather than custom abstractions
4. **Leverage the type system**: Encode invariants in types (newtype, typestate) rather than runtime checks
5. **Consider the async ecosystem**: For I/O-heavy code, async/await often beats threads
6. **Don't over-pattern**: Simple code beats clever patterns; only apply patterns when they solve real problems

Rust's patterns emphasize compile-time guarantees, zero-cost abstractions, and fearless concurrency. Master these patterns to write code that is both safe and performant, idiomatic and maintainable.
