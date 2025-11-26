## Project 1: Plugin System with Trait Objects and Dynamic Dispatch

### Problem Statement

Build a plugin system that allows loading different plugins at runtime. You'll start with a basic trait for plugins, implement heterogeneous collections using trait objects, then build a complete plugin manager with dynamic dispatch.

### Why It Matters

**Real-World Impact**: Plugin systems are fundamental to extensible software architecture:

**The Monolithic Problem**:
- **Without plugins**: Every feature hardcoded → 100MB binary for text editor with all features
- **Firefox without extensions**: Would need to rebuild browser for every user preference
- **VSCode**: Core is ~50MB, extensions add functionality on-demand
- **Photoshop**: Plugin architecture allows third-party filters without modifying core

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

### Use Cases

**When you need this pattern**:
1. **Plugin architectures**: Load functionality at runtime, not compile-time
2. **Heterogeneous collections**: Vec<Box<dyn Widget>> - different types, same interface
3. **Embedded/WASM**: Binary size matters, dynamic dispatch reduces bloat
4. **Middleware chains**: HTTP middleware, database interceptors
5. **Event systems**: Different event handlers with uniform interface
6. **Component systems**: Game entities with different component types

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

**Starter Code**:
```rust
// Plugin trait - what all plugins must implement
trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self) -> Result<String, String>;
}

// Concrete plugin implementations
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
        // TODO: Return Ok with greeting message
        todo!()
    }
}

struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str {
        // TODO: Return plugin name
        todo!()
    }

    fn version(&self) -> &str {
        // TODO: Return version
        todo!()
    }

    fn execute(&self) -> Result<String, String> {
        // TODO: Perform a simple calculation and return result
        // Example: "2 + 2 = 4"
        todo!()
    }
}

// Static dispatch version - different function for each type
fn run_plugin<T: Plugin>(plugin: &T) {
    // TODO: Print plugin name and version
    // TODO: Execute plugin and print result or error
    todo!()
}
```

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

**Starter Code**:
```rust
// Check if Plugin trait is object-safe
// Requirements:
// - No generic methods (methods can't have type parameters)
// - No Self: Sized bound
// - Methods must have &self or &mut self receiver

// Add a plugin that reads files
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
        // TODO: Read file at self.path (simulate with dummy data)
        // Return Ok(content) or Err(error_message)
        todo!()
    }
}

// Dynamic dispatch version - one function for all types
fn run_plugin_dynamic(plugin: &dyn Plugin) {
    // TODO: Same as static version but takes trait object
    // Print name, version, execute result
    todo!()
}

// Plugin collection manager
struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        // TODO: Create PluginManager with empty Vec
        todo!()
    }

    fn register(&mut self, plugin: Box<dyn Plugin>) {
        // TODO: Add plugin to Vec
        todo!()
    }

    fn run_all(&self) {
        // TODO: Iterate through plugins and run each one
        // Hint: for plugin in &self.plugins { run_plugin_dynamic(plugin.as_ref()); }
        todo!()
    }

    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        // TODO: Find plugin by name
        // Hint: self.plugins.iter().find(|p| p.name() == name).map(|b| b.as_ref())
        todo!()
    }
}
```

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

**Starter Code**:
```rust
use std::collections::HashMap;

// Configuration for plugins
#[derive(Debug, Clone)]
struct PluginConfig {
    settings: HashMap<String, String>,
}

impl PluginConfig {
    fn new() -> Self {
        // TODO: Create empty config
        todo!()
    }

    fn set(&mut self, key: String, value: String) {
        // TODO: Insert key-value pair
        todo!()
    }

    fn get(&self, key: &str) -> Option<&str> {
        // TODO: Get value by key
        todo!()
    }
}

// Metadata about a plugin (separate from Plugin trait for flexibility)
trait PluginMetadata {
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    fn dependencies(&self) -> Vec<&str> {
        vec![]  // Default: no dependencies
    }
}

// Main plugin trait with lifecycle
trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;

    // Lifecycle hooks
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        // TODO: Default implementation that does nothing
        Ok(())
    }

    fn execute(&self) -> Result<String, String>;

    fn cleanup(&mut self) -> Result<(), String> {
        // TODO: Default implementation
        Ok(())
    }
}

// Example: Logging plugin with state
struct LoggingPlugin {
    log_level: String,
    initialized: bool,
}

impl LoggingPlugin {
    fn new() -> Self {
        // TODO: Create with default values
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

    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        // TODO: Read log_level from config, set initialized = true
        // If config has "log_level", use it; otherwise use "INFO"
        todo!()
    }

    fn execute(&self) -> Result<String, String> {
        // TODO: Check if initialized, return error if not
        // Otherwise, return log message with current level
        todo!()
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // TODO: Set initialized = false, reset state
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

// Enhanced plugin manager with lifecycle
struct EnhancedPluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl EnhancedPluginManager {
    fn new() -> Self {
        // TODO: Create with empty Vec
        todo!()
    }

    fn register_and_init(
        &mut self,
        mut plugin: Box<dyn Plugin>,
        config: &PluginConfig,
    ) -> Result<(), String> {
        // TODO: Initialize plugin with config
        // TODO: If initialization succeeds, add to Vec
        // TODO: If fails, return error
        todo!()
    }

    fn execute_plugin(&self, name: &str) -> Result<String, String> {
        // TODO: Find plugin by name and execute it
        // Return Err if not found
        todo!()
    }

    fn shutdown(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // TODO: Call cleanup() on all plugins
        // Collect any errors
        // Clear the plugins Vec

        errors
    }
}
```

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

**Check Your Understanding**:
- Why separate `PluginMetadata` from `Plugin` trait?
- Could `initialize` take a generic `T: Config` instead of `&PluginConfig`?
- Why would that break object safety?
- How do lifecycle hooks compare to constructors/destructors?
- When would you need `&mut dyn Plugin` vs `&dyn Plugin`?

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
