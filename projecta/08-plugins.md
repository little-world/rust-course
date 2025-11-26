## Project 8: Plugin System with Trait Objects and Dynamic Dispatch

### Problem Statement

Build a plugin system that allows loading different plugins at runtime. You'll start with a basic trait for plugins, implement heterogeneous collections using trait objects, then build a complete plugin manager with dynamic dispatch.

### Use Cases

**When you need this pattern**:
1. **Plugin architectures**: Load functionality at runtime, not compile-time
2. **Heterogeneous collections**: Vec<Box<dyn Widget>> - different types, same interface
3. **Embedded/WASM**: Binary size matters, dynamic dispatch reduces bloat
4. **Middleware chains**: HTTP middleware, database interceptors
5. **Event systems**: Different event handlers with uniform interface
6. **Component systems**: Game entities with different component types

**Real-World Impact**: Plugin systems are fundamental to extensible software architecture:

**The Monolithic Problem**:
- **Without plugins**: Every feature hardcoded → 100MB binary for text editor with all features
- **Firefox without extensions**: Would need to rebuild browser for every user preference
- **VSCode**: Core is ~50MB, extensions add functionality on-demand
- **Photoshop**: Plugin architecture allows third-party filters without modifying core

### Why It Matters

**Static vs Dynamic Dispatch**:
```rust
// Static dispatch - compile-time known types
fn process<T: Plugin>(plugin: T) {
    plugin.execute();
}
// Compiler generates: process_AudioPlugin(), process_VideoPlugin(), etc.
// Binary size: 50KB per plugin × 100 plugins = 5MB just for dispatch!
```

```rust
// Dynamic dispatch - runtime polymorphism
fn process(plugin: &dyn Plugin) {
    plugin.execute();  // Vtable lookup ~3ns overhead
}
// Binary size: One function, ~500 bytes
// Trade-off: 3ns per call vs 5MB binary size
```

**Performance Numbers**:
- **Static dispatch**: 0ns overhead (direct call), can inline
- **Dynamic dispatch**: ~2-3ns vtable lookup, no inlining
- **Binary size**: Static = N × function_size, Dynamic = 1 × function_size
- **Compilation**: Static = slower (more monomorphization), Dynamic = faster

**Real Production Examples**:
- **Rust compiler plugins**: Procedural macros loaded dynamically
- **Game engines**: Entity components, rendering pipelines
- **Web servers**: Middleware chains (auth, logging, compression)
- **Editors**: VSCode extensions, Vim plugins
- **Browsers**: Firefox WebExtensions, Chrome extensions


**Dynamic Dispatch is Critical When**:
- Types not known at compile-time (loading from disk/network)
- Binary size constrained (embedded systems, WebAssembly)
- Many implementations (100+ plugins → avoid code bloat)
- Hot-loading required (swap implementations at runtime)

### Learning Goals

- Understand trait objects (`&dyn Trait`, `Box<dyn Trait>`)
- Learn object safety rules and constraints
- Compare static vs dynamic dispatch trade-offs
- Build heterogeneous collections
- Implement vtable-based polymorphism
- Measure performance impact of dynamic dispatch

---

### Milestone 1: Basic Plugin Trait with Static Dispatch

**Goal**: Define a plugin trait and implement it for several types.

**Checkpoint Tests**:
```rust
#[test]
fn test_greeter_plugin() {
    let plugin = GreeterPlugin {
        greeting: "Hello, World!".to_string(),
    };

    assert_eq!(plugin.name(), "Greeter");
    assert_eq!(plugin.version(), "1.0.0");
    assert!(plugin.execute().is_ok());
}

#[test]
fn test_calculator_plugin() {
    let plugin = CalculatorPlugin;
    let result = plugin.execute().unwrap();
    assert!(result.contains("="));  // Should have calculation result
}

#[test]
fn test_static_dispatch() {
    let greeter = GreeterPlugin {
        greeting: "Hi!".to_string(),
    };
    let calculator = CalculatorPlugin;

    // Static dispatch - each call is to a different monomorphized function
    run_plugin(&greeter);
    run_plugin(&calculator);
}
```

**Starter Code**:
```rust
// Plugin: Core trait defining the plugin interface
// Role: Provides uniform API that all plugins must implement
trait Plugin {
    // name: Returns the plugin's identifier
    // Role: Used for logging and plugin registry lookups
    fn name(&self) -> &str;

    // version: Returns semantic version string
    // Role: Version compatibility checks and debugging
    fn version(&self) -> &str;

    // execute: Runs the plugin's main functionality
    // Role: Executes plugin logic, returns success message or error
    fn execute(&self) -> Result<String, String>;
}

// GreeterPlugin: Plugin that generates greeting messages
// Role: Demonstrates stateful plugin with stored configuration
struct GreeterPlugin {
    greeting: String,  // Customizable greeting message to display
}

impl Plugin for GreeterPlugin {
    fn name(&self) -> &str {
        "Greeter"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    // execute: Returns the configured greeting
    // Role: Demonstrates simple string processing plugin
    fn execute(&self) -> Result<String, String> {
        // TODO: Return Ok with greeting message
        // Hint: Ok(self.greeting.clone())
        todo!()
    }
}

// CalculatorPlugin: Plugin that performs arithmetic operations
// Role: Demonstrates stateless plugin (zero-sized type)
struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    // name: Identifies this as the calculator plugin
    fn name(&self) -> &str {
        // TODO: Return plugin name "Calculator"
        todo!()
    }

    // version: Returns calculator version
    fn version(&self) -> &str {
        // TODO: Return version "1.0.0"
        todo!()
    }

    // execute: Performs simple calculation
    // Role: Demonstrates computational plugin
    fn execute(&self) -> Result<String, String> {
        // TODO: Perform a simple calculation and return result
        // Example: Ok("2 + 2 = 4".to_string())
        todo!()
    }
}

// run_plugin: Generic function using static dispatch
// Role: Executes any type implementing Plugin trait
// Note: Compiler generates separate copy for each concrete type (monomorphization)
fn run_plugin<T: Plugin>(plugin: &T) {
    // TODO: Print plugin name and version
    // TODO: Execute plugin and print result or error
    // Hint: println!("{} v{}", plugin.name(), plugin.version());
    // Hint: match plugin.execute() { Ok(msg) => ..., Err(e) => ... }
    todo!()
}
```

**Check Your Understanding**:
- How many versions of `run_plugin` does the compiler generate?
- Can you store `GreeterPlugin` and `CalculatorPlugin` in the same `Vec`? Why not?
- What's the performance of calling `plugin.execute()` with static dispatch?

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Critical Limitations**:
1. **Can't store mixed types**: `Vec<Plugin>` doesn't work - Plugin isn't sized
2. **Binary bloat**: Each plugin type generates separate `run_plugin` function
3. **No runtime flexibility**: Can't load plugins dynamically from config
4. **Collection problem**: Can't have `Vec` of different plugin types

**What we're adding**: **Trait Objects** - dynamic dispatch with `&dyn Plugin`:
- `&dyn Plugin` or `Box<dyn Plugin>` - fat pointer (ptr + vtable)
- Vtable contains function pointers for each trait method
- One `run_plugin` function for all types
- Heterogeneous collections possible

**Improvements**:
- **Heterogeneous collections**: `Vec<Box<dyn Plugin>>` holds any plugin
- **Smaller binary**: One function instead of N monomorphized copies
- **Runtime polymorphism**: Choose which plugin to use at runtime
- **Performance cost**: ~2-3ns vtable lookup per call

**Trade-offs**:
- **Slower**: Vtable indirection prevents inlining
- **Memory**: Fat pointer (16 bytes) vs thin pointer (8 bytes)
- **Object safety**: Not all traits can be trait objects

---

### Milestone 2: Trait Objects for Heterogeneous Collections

**Goal**: Use trait objects to store different plugin types in one collection.

**Checkpoint Tests**:
```rust
#[test]
fn test_heterogeneous_collection() {
    let plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(GreeterPlugin { greeting: "Hello!".to_string() }),
        Box::new(CalculatorPlugin),
        Box::new(FileReaderPlugin { path: "data.txt".to_string() }),
    ];

    assert_eq!(plugins.len(), 3);

    // Can call methods on trait objects
    for plugin in &plugins {
        println!("Running: {}", plugin.name());
        let _ = plugin.execute();
    }
}

#[test]
fn test_plugin_manager() {
    let mut manager = PluginManager::new();

    manager.register(Box::new(GreeterPlugin { greeting: "Hi!".to_string() }));
    manager.register(Box::new(CalculatorPlugin));

    // Should have 2 plugins
    assert_eq!(manager.plugins.len(), 2);

    // Can find by name
    assert!(manager.get_plugin("Greeter").is_some());
    assert!(manager.get_plugin("Unknown").is_none());

    manager.run_all();
}

#[test]
fn test_dynamic_dispatch() {
    let plugin: &dyn Plugin = &GreeterPlugin { greeting: "Test".to_string() };

    // Uses vtable lookup
    assert_eq!(plugin.name(), "Greeter");
    let _ = plugin.execute();
}
```

**Starter Code**:
```rust
// Note: Plugin trait from Milestone 1 must be object-safe
// Object safety requirements:
// - No generic methods (methods can't have type parameters)
// - No Self: Sized bound
// - Methods must have &self or &mut self receiver

// FileReaderPlugin: Plugin that reads and processes files
// Role: Demonstrates I/O-based plugin with file path configuration
struct FileReaderPlugin {
    path: String,  // Path to file this plugin will read
}

impl Plugin for FileReaderPlugin {
    fn name(&self) -> &str {
        "FileReader"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    // execute: Simulates reading file content
    // Role: Demonstrates file I/O plugin pattern
    fn execute(&self) -> Result<String, String> {
        // TODO: Read file at self.path (simulate with dummy data for now)
        // Return Ok(content) or Err(error_message)
        // Hint: Ok(format!("Read content from: {}", self.path))
        todo!()
    }
}

// run_plugin_dynamic: Executes plugin using dynamic dispatch
// Role: Demonstrates trait object usage with vtable lookup
// Note: Only one function generated (vs one per type with static dispatch)
fn run_plugin_dynamic(plugin: &dyn Plugin) {
    // TODO: Same as static version but takes trait object
    // Print name, version, execute result
    // Hint: Same implementation as run_plugin<T> but takes &dyn Plugin
    todo!()
}

// PluginManager: Container for heterogeneous plugin collection
// Role: Manages lifecycle and execution of multiple plugin instances
struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,  // Heap-allocated trait objects for heterogeneous storage
}

impl PluginManager {
    // new: Creates empty plugin manager
    // Role: Initializes plugin registry
    fn new() -> Self {
        // TODO: Create PluginManager with empty Vec
        // Hint: PluginManager { plugins: Vec::new() }
        todo!()
    }

    // register: Adds plugin to manager
    // Role: Registers new plugin for execution
    // Note: Takes Box<dyn Plugin> for heap allocation and ownership
    fn register(&mut self, plugin: Box<dyn Plugin>) {
        // TODO: Add plugin to Vec
        // Hint: self.plugins.push(plugin);
        todo!()
    }

    // run_all: Executes all registered plugins in order
    // Role: Batch plugin execution for startup/shutdown hooks
    fn run_all(&self) {
        // TODO: Iterate through plugins and run each one
        // Hint: for plugin in &self.plugins { run_plugin_dynamic(plugin.as_ref()); }
        todo!()
    }

    // get_plugin: Finds plugin by name
    // Role: Plugin lookup for selective execution
    // Returns: Trait object reference if found, None otherwise
    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        // TODO: Find plugin by name
        // Hint: self.plugins.iter().find(|p| p.name() == name).map(|b| b.as_ref())
        todo!()
    }
}
```

**Check Your Understanding**:
- What's the size of `&dyn Plugin` vs `&GreeterPlugin`? (Hint: 16 bytes vs 8 bytes)
- Why can't you have `Vec<dyn Plugin>` (without Box)?
- What happens at runtime when you call `plugin.execute()`?
- How does the vtable know which implementation to call?

---

### 🔄 Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Remaining Issues**:
1. **No plugin metadata**: Can't query capabilities, dependencies, etc.
2. **No lifecycle management**: No initialization, cleanup hooks
3. **No configuration**: Plugins can't receive config at load time
4. **Object safety constraints**: What if we want to add methods with generics?

**What we're adding**:
- **Lifecycle hooks**: `initialize()` and `cleanup()` methods
- **Plugin metadata**: Separate metadata trait
- **Configuration**: Pass config during initialization
- **Builder pattern**: Ergonomic plugin construction
- **Object safety workaround**: Separate traits for object-safe vs generic methods

**Improvements**:
- **Complete lifecycle**: Plugins can setup/teardown resources
- **Rich metadata**: Query dependencies, capabilities
- **Configurable**: Pass settings to plugins
- **Type-safe config**: Use generics where needed, trait objects where not

---

### Milestone 3: Complete Plugin System with Lifecycle

**Goal**: Build a production-ready plugin system with initialization, configuration, and metadata.

**Checkpoint Tests**:
```rust
#[test]
fn test_plugin_lifecycle() {
    let mut plugin = LoggingPlugin::new();

    // Not initialized yet
    assert!(!plugin.initialized);

    // Initialize with config
    let mut config = PluginConfig::new();
    config.set("log_level".to_string(), "DEBUG".to_string());

    assert!(plugin.initialize(&config).is_ok());
    assert!(plugin.initialized);
    assert_eq!(plugin.log_level, "DEBUG");

    // Execute should work now
    assert!(plugin.execute().is_ok());

    // Cleanup
    assert!(plugin.cleanup().is_ok());
    assert!(!plugin.initialized);
}

#[test]
fn test_enhanced_manager() {
    let mut manager = EnhancedPluginManager::new();

    let mut config = PluginConfig::new();
    config.set("log_level".to_string(), "INFO".to_string());

    // Register and initialize
    let plugin = Box::new(LoggingPlugin::new());
    assert!(manager.register_and_init(plugin, &config).is_ok());

    // Execute by name
    let result = manager.execute_plugin("Logger");
    assert!(result.is_ok());

    // Shutdown
    let errors = manager.shutdown();
    assert!(errors.is_empty());
    assert_eq!(manager.plugins.len(), 0);
}

#[test]
fn test_metadata() {
    let plugin = LoggingPlugin::new();

    // Check metadata
    assert_eq!(plugin.author(), "Plugin Team");
    assert!(!plugin.description().is_empty());
    assert_eq!(plugin.dependencies().len(), 0);
}
```

**Starter Code**:
```rust
use std::collections::HashMap;

// PluginConfig: Key-value configuration store for plugins
// Role: Provides runtime configuration to plugins during initialization
#[derive(Debug, Clone)]
struct PluginConfig {
    settings: HashMap<String, String>,  // Stores plugin configuration as key-value pairs
}

impl PluginConfig {
    // new: Creates empty configuration
    // Role: Initializes fresh config for plugin setup
    fn new() -> Self {
        // TODO: Create empty config
        // Hint: PluginConfig { settings: HashMap::new() }
        todo!()
    }

    // set: Adds or updates configuration value
    // Role: Sets plugin parameters (e.g., "log_level" => "DEBUG")
    fn set(&mut self, key: String, value: String) {
        // TODO: Insert key-value pair
        // Hint: self.settings.insert(key, value);
        todo!()
    }

    // get: Retrieves configuration value by key
    // Role: Allows plugins to read their configuration
    fn get(&self, key: &str) -> Option<&str> {
        // TODO: Get value by key
        // Hint: self.settings.get(key).map(|s| s.as_str())
        todo!()
    }
}

// PluginMetadata: Trait for plugin metadata (separate from Plugin)
// Role: Provides descriptive information without coupling to execution
// Note: Separate trait allows metadata queries without needing mutable access
trait PluginMetadata {
    // author: Returns plugin author/maintainer
    fn author(&self) -> &str;

    // description: Returns human-readable plugin description
    fn description(&self) -> &str;

    // dependencies: Lists plugin dependencies (default: none)
    // Role: Enables dependency resolution in plugin managers
    fn dependencies(&self) -> Vec<&str> {
        vec![]  // Default: no dependencies
    }
}

// Plugin: Core trait with full lifecycle management
// Role: Defines plugin interface with init, execute, cleanup hooks
trait Plugin {
    // name: Plugin identifier
    fn name(&self) -> &str;

    // version: Semantic version
    fn version(&self) -> &str;

    // initialize: Setup hook called before first use
    // Role: Configures plugin state from PluginConfig
    // Note: &mut self allows state modification
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        // Default: no-op initialization
        Ok(())
    }

    // execute: Main plugin functionality
    // Role: Performs plugin's primary task
    fn execute(&self) -> Result<String, String>;

    // cleanup: Teardown hook called during shutdown
    // Role: Releases resources, saves state
    fn cleanup(&mut self) -> Result<(), String> {
        // Default: no-op cleanup
        Ok(())
    }
}

// LoggingPlugin: Example plugin with stateful lifecycle
// Role: Demonstrates configurable plugin with init/cleanup
struct LoggingPlugin {
    log_level: String,  // Configured logging level (DEBUG, INFO, WARN, ERROR)
    initialized: bool,  // Tracks whether initialize() has been called
}

impl LoggingPlugin {
    // new: Creates plugin in uninitialized state
    // Role: Constructor called before initialize()
    fn new() -> Self {
        // TODO: Create with default values
        // Hint: LoggingPlugin { log_level: "INFO".to_string(), initialized: false }
        todo!()
    }
}

impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        "Logger"
    }

    fn version(&self) -> &str {
        "2.0.0"
    }

    // initialize: Reads configuration and sets up plugin
    // Role: Transitions plugin from created to ready state
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        // TODO: Read log_level from config, set initialized = true
        // If config has "log_level", use it; otherwise keep default "INFO"
        // Hint: if let Some(level) = config.get("log_level") { self.log_level = level.to_string(); }
        // Hint: self.initialized = true;
        todo!()
    }

    // execute: Performs logging operation
    // Role: Returns log message if initialized, error otherwise
    fn execute(&self) -> Result<String, String> {
        // TODO: Check if initialized, return error if not
        // Otherwise, return log message with current level
        // Hint: if !self.initialized { return Err("Not initialized".to_string()); }
        // Hint: Ok(format!("Logging at level: {}", self.log_level))
        todo!()
    }

    // cleanup: Resets plugin to uninitialized state
    // Role: Prepares plugin for shutdown or reinitialization
    fn cleanup(&mut self) -> Result<(), String> {
        // TODO: Set initialized = false, reset state
        // Hint: self.initialized = false; Ok(())
        todo!()
    }
}

impl PluginMetadata for LoggingPlugin {
    fn author(&self) -> &str {
        "Plugin Team"
    }

    fn description(&self) -> &str {
        "Provides logging functionality with configurable levels"
    }
}

// EnhancedPluginManager: Lifecycle-aware plugin container
// Role: Manages initialization, execution, and cleanup of plugin collection
struct EnhancedPluginManager {
    plugins: Vec<Box<dyn Plugin>>,  // Heterogeneous collection of initialized plugins
}

impl EnhancedPluginManager {
    // new: Creates empty manager
    fn new() -> Self {
        // TODO: Create with empty Vec
        todo!()
    }

    // register_and_init: Registers plugin and initializes it
    // Role: Atomic registration+initialization to ensure all plugins are ready
    fn register_and_init(
        &mut self,
        mut plugin: Box<dyn Plugin>,
        config: &PluginConfig,
    ) -> Result<(), String> {
        // TODO: Initialize plugin with config
        // TODO: If initialization succeeds, add to Vec
        // TODO: If fails, return error (plugin not added)
        // Hint: plugin.initialize(config)?; self.plugins.push(plugin); Ok(())
        todo!()
    }

    // execute_plugin: Runs specific plugin by name
    // Role: Selective plugin execution
    fn execute_plugin(&self, name: &str) -> Result<String, String> {
        // TODO: Find plugin by name and execute it
        // Return Err if not found
        // Hint: self.plugins.iter().find(|p| p.name() == name)...
        todo!()
    }

    // shutdown: Cleanly shuts down all plugins
    // Role: Calls cleanup on all plugins, collects errors, clears registry
    fn shutdown(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // TODO: Call cleanup() on all plugins
        // Collect any errors (don't stop on first error)
        // Clear the plugins Vec
        // Hint: for plugin in &mut self.plugins { if let Err(e) = plugin.cleanup() { errors.push(e); } }
        // Hint: self.plugins.clear();

        errors
    }
}
```

**Check Your Understanding**:
```rust
#[test]
fn test_plugin_lifecycle() {
    let mut plugin = LoggingPlugin::new();

    // Not initialized yet
    assert!(!plugin.initialized);

    // Initialize with config
    let mut config = PluginConfig::new();
    config.set("log_level".to_string(), "DEBUG".to_string());

    assert!(plugin.initialize(&config).is_ok());
    assert!(plugin.initialized);
    assert_eq!(plugin.log_level, "DEBUG");

    // Execute should work now
    assert!(plugin.execute().is_ok());

    // Cleanup
    assert!(plugin.cleanup().is_ok());
    assert!(!plugin.initialized);
}

#[test]
fn test_enhanced_manager() {
    let mut manager = EnhancedPluginManager::new();

    let mut config = PluginConfig::new();
    config.set("log_level".to_string(), "INFO".to_string());

    // Register and initialize
    let plugin = Box::new(LoggingPlugin::new());
    assert!(manager.register_and_init(plugin, &config).is_ok());

    // Execute by name
    let result = manager.execute_plugin("Logger");
    assert!(result.is_ok());

    // Shutdown
    let errors = manager.shutdown();
    assert!(errors.is_empty());
    assert_eq!(manager.plugins.len(), 0);
}

#[test]
fn test_metadata() {
    let plugin = LoggingPlugin::new();

    // Check metadata
    assert_eq!(plugin.author(), "Plugin Team");
    assert!(!plugin.description().is_empty());
    assert_eq!(plugin.dependencies().len(), 0);
}
```

**Check Your Understanding**:
- Why separate `PluginMetadata` from `Plugin` trait?
- Could `initialize` take a generic `T: Config` instead of `&PluginConfig`?
- Why would that break object safety?
- How do lifecycle hooks compare to constructors/destructors?
- When would you need `&mut dyn Plugin` vs `&dyn Plugin`?

---

### Complete Working Example

Here's the fully implemented plugin system combining all three milestones:

```rust
use std::collections::HashMap;

// ============================================================================
// MILESTONE 1 & 2: Core Plugin Trait and Trait Objects
// ============================================================================

// Plugin trait - object-safe for dynamic dispatch
trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self) -> Result<String, String>;
}

// Stateful plugin
struct GreeterPlugin {
    greeting: String,
}

impl Plugin for GreeterPlugin {
    fn name(&self) -> &str {
        "Greeter"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self) -> Result<String, String> {
        Ok(self.greeting.clone())
    }
}

// Stateless plugin (zero-sized type)
struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str {
        "Calculator"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self) -> Result<String, String> {
        Ok("2 + 2 = 4".to_string())
    }
}

struct FileReaderPlugin {
    path: String,
}

impl Plugin for FileReaderPlugin {
    fn name(&self) -> &str {
        "FileReader"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self) -> Result<String, String> {
        Ok(format!("Read content from: {}", self.path))
    }
}

// Static dispatch (monomorphization)
fn run_plugin<T: Plugin>(plugin: &T) {
    println!("{} v{}", plugin.name(), plugin.version());
    match plugin.execute() {
        Ok(msg) => println!("✓ {}", msg),
        Err(e) => println!("✗ Error: {}", e),
    }
}

// Dynamic dispatch (trait objects)
fn run_plugin_dynamic(plugin: &dyn Plugin) {
    println!("{} v{}", plugin.name(), plugin.version());
    match plugin.execute() {
        Ok(msg) => println!("✓ {}", msg),
        Err(e) => println!("✗ Error: {}", e),
    }
}

// Basic plugin manager
struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    fn run_all(&self) {
        for plugin in &self.plugins {
            run_plugin_dynamic(plugin.as_ref());
        }
    }

    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|b| b.as_ref())
    }
}

// ============================================================================
// MILESTONE 3: Lifecycle-Aware Plugin System
// ============================================================================

#[derive(Debug, Clone)]
struct PluginConfig {
    settings: HashMap<String, String>,
}

impl PluginConfig {
    fn new() -> Self {
        PluginConfig {
            settings: HashMap::new(),
        }
    }

    fn set(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }
}

trait PluginMetadata {
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
}

trait PluginWithLifecycle {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        Ok(())
    }

    fn execute(&self) -> Result<String, String>;

    fn cleanup(&mut self) -> Result<(), String> {
        Ok(())
    }
}

struct LoggingPlugin {
    log_level: String,
    initialized: bool,
}

impl LoggingPlugin {
    fn new() -> Self {
        LoggingPlugin {
            log_level: "INFO".to_string(),
            initialized: false,
        }
    }
}

impl PluginWithLifecycle for LoggingPlugin {
    fn name(&self) -> &str {
        "Logger"
    }

    fn version(&self) -> &str {
        "2.0.0"
    }

    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        if let Some(level) = config.get("log_level") {
            self.log_level = level.to_string();
        }
        self.initialized = true;
        Ok(())
    }

    fn execute(&self) -> Result<String, String> {
        if !self.initialized {
            return Err("Plugin not initialized".to_string());
        }
        Ok(format!("Logging at level: {}", self.log_level))
    }

    fn cleanup(&mut self) -> Result<(), String> {
        self.initialized = false;
        self.log_level = "INFO".to_string();
        Ok(())
    }
}

impl PluginMetadata for LoggingPlugin {
    fn author(&self) -> &str {
        "Plugin Team"
    }

    fn description(&self) -> &str {
        "Provides logging functionality with configurable levels"
    }
}

struct EnhancedPluginManager {
    plugins: Vec<Box<dyn PluginWithLifecycle>>,
}

impl EnhancedPluginManager {
    fn new() -> Self {
        EnhancedPluginManager {
            plugins: Vec::new(),
        }
    }

    fn register_and_init(
        &mut self,
        mut plugin: Box<dyn PluginWithLifecycle>,
        config: &PluginConfig,
    ) -> Result<(), String> {
        plugin.initialize(config)?;
        self.plugins.push(plugin);
        Ok(())
    }

    fn execute_plugin(&self, name: &str) -> Result<String, String> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?
            .execute()
    }

    fn shutdown(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        for plugin in &mut self.plugins {
            if let Err(e) = plugin.cleanup() {
                errors.push(format!("{}: {}", plugin.name(), e));
            }
        }

        self.plugins.clear();
        errors
    }
}

// ============================================================================
// Example Usage
// ============================================================================

fn main() {
    println!("=== Static Dispatch Example ===\n");

    let greeter = GreeterPlugin {
        greeting: "Hello from Rust!".to_string(),
    };
    let calculator = CalculatorPlugin;

    run_plugin(&greeter);      // Monomorphized to run_plugin_GreeterPlugin
    run_plugin(&calculator);   // Monomorphized to run_plugin_CalculatorPlugin

    println!("\n=== Dynamic Dispatch Example ===\n");

    let mut manager = PluginManager::new();
    manager.register(Box::new(GreeterPlugin {
        greeting: "Dynamic greeting!".to_string(),
    }));
    manager.register(Box::new(CalculatorPlugin));
    manager.register(Box::new(FileReaderPlugin {
        path: "data.txt".to_string(),
    }));

    manager.run_all();

    println!("\n=== Plugin Lookup ===");
    if let Some(plugin) = manager.get_plugin("Calculator") {
        println!("Found plugin: {} v{}", plugin.name(), plugin.version());
    }

    println!("\n=== Lifecycle-Aware Plugin System ===\n");

    let mut enhanced_manager = EnhancedPluginManager::new();

    // Configure and initialize logging plugin
    let mut config = PluginConfig::new();
    config.set("log_level".to_string(), "DEBUG".to_string());

    let logger = Box::new(LoggingPlugin::new());
    match enhanced_manager.register_and_init(logger, &config) {
        Ok(_) => println!("Logger plugin initialized successfully"),
        Err(e) => println!("Failed to initialize logger: {}", e),
    }

    // Execute plugin by name
    match enhanced_manager.execute_plugin("Logger") {
        Ok(msg) => println!("Logger result: {}", msg),
        Err(e) => println!("Error: {}", e),
    }

    // Shutdown
    println!("\n=== Shutting Down ===");
    let errors = enhanced_manager.shutdown();
    if errors.is_empty() {
        println!("All plugins cleaned up successfully");
    } else {
        println!("Cleanup errors: {:?}", errors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_dispatch() {
        let greeter = GreeterPlugin {
            greeting: "Test".to_string(),
        };
        assert_eq!(greeter.name(), "Greeter");
        assert!(greeter.execute().is_ok());
    }

    #[test]
    fn test_heterogeneous_collection() {
        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(GreeterPlugin {
                greeting: "Hi".to_string(),
            }),
            Box::new(CalculatorPlugin),
        ];
        assert_eq!(plugins.len(), 2);
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(CalculatorPlugin));
        assert!(manager.get_plugin("Calculator").is_some());
        assert!(manager.get_plugin("Unknown").is_none());
    }

    #[test]
    fn test_lifecycle() {
        let mut plugin = LoggingPlugin::new();
        assert!(!plugin.initialized);

        let mut config = PluginConfig::new();
        config.set("log_level".to_string(), "DEBUG".to_string());

        plugin.initialize(&config).unwrap();
        assert!(plugin.initialized);
        assert_eq!(plugin.log_level, "DEBUG");

        plugin.cleanup().unwrap();
        assert!(!plugin.initialized);
    }

    #[test]
    fn test_enhanced_manager() {
        let mut manager = EnhancedPluginManager::new();
        let mut config = PluginConfig::new();

        let logger = Box::new(LoggingPlugin::new());
        assert!(manager.register_and_init(logger, &config).is_ok());
        assert!(manager.execute_plugin("Logger").is_ok());

        let errors = manager.shutdown();
        assert!(errors.is_empty());
        assert_eq!(manager.plugins.len(), 0);
    }
}
```

**Key Takeaways from Complete Example**:

1. **Static vs Dynamic Dispatch**:
    - Static: `run_plugin<T>` generates code for each type, zero overhead
    - Dynamic: `run_plugin_dynamic(&dyn Plugin)` uses vtable, one function for all types

2. **Object Safety**: Plugin traits must follow rules (no generic methods, etc.)

3. **Heterogeneous Collections**: `Vec<Box<dyn Plugin>>` stores different types

4. **Lifecycle Management**: Init/execute/cleanup pattern for stateful plugins

5. **Trade-offs**:
    - Static: Fast, large binary, compile-time known types
    - Dynamic: Small binary, runtime polymorphism, slight overhead

---

### Complete Project Summary

**What You Built**:
1. Basic plugin trait with static dispatch
2. Trait objects for heterogeneous plugin collections
3. Complete plugin system with lifecycle and configuration
4. Understanding of object safety and vtable dispatch

**Key Concepts Practiced**:
- Trait objects (`&dyn Trait`, `Box<dyn Trait>`)
- Static vs dynamic dispatch trade-offs
- Object safety rules
- Heterogeneous collections
- Lifecycle management patterns

**Performance Characteristics**:
- Static dispatch: 0-1ns, can inline, large binary
- Dynamic dispatch: 2-3ns vtable lookup, small binary, no inline
- Fat pointer: 16 bytes (8-byte ptr + 8-byte vtable ptr)
- Thin pointer: 8 bytes

**Real-World Applications**:
- VSCode extension system
- Game engine component systems
- Web server middleware chains
- Database plugin architectures

---