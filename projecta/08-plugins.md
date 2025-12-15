# Plugin System with Trait Objects

### Problem Statement

Build a plugin system that allows loading different plugins at runtime. You'll start with a basic trait for plugins, implement heterogeneous collections using trait objects, then build a complete plugin manager with dynamic dispatch.

---

## Key Concepts Explained

This project demonstrates how Rust enables runtime polymorphism through trait objects while maintaining safety guarantees.

### 1. Static vs Dynamic Dispatch

Two ways to achieve polymorphism in Rust:

```rust
// Static dispatch - compile-time resolution
fn process<T: Plugin>(plugin: T) {
    plugin.execute();
}
// Compiler generates: process_AudioPlugin(), process_VideoPlugin()
// Fast (direct call), but binary bloat

// Dynamic dispatch - runtime resolution
fn process(plugin: &dyn Plugin) {
    plugin.execute();  // Vtable lookup
}
// Slower (~3ns overhead), but flexible and compact
```

**Why it matters**: Static = speed, Dynamic = flexibility. Choose based on requirements.

### 2. Trait Objects and Fat Pointers

Trait objects enable runtime polymorphism:

```rust
let plugin: &dyn Plugin = &GreeterPlugin { ... };
// Size: 16 bytes (2 pointers)
// [data_ptr: 8 bytes][vtable_ptr: 8 bytes]
```

**Fat pointer**:
- **Data pointer**: Points to actual object
- **Vtable pointer**: Points to function table

**vs Thin pointer**:
```rust
let plugin: &GreeterPlugin = &GreeterPlugin { ... };
// Size: 8 bytes (1 pointer)
```

### 3. Vtables and Function Dispatch

Virtual method table contains function pointers:

```
GreeterPlugin Vtable:
  name     -> GreeterPlugin::name
  version  -> GreeterPlugin::version
  execute  -> GreeterPlugin::execute
  drop     -> GreeterPlugin::drop

CalculatorPlugin Vtable:
  name     -> CalculatorPlugin::name
  version  -> CalculatorPlugin::version
  execute  -> CalculatorPlugin::execute
  drop     -> CalculatorPlugin::drop
```

**Runtime call**:
```rust
plugin.execute()
// 1. Load vtable pointer from fat pointer
// 2. Index into vtable for execute
// 3. Call function pointer
// Total: ~2-3ns overhead
```

### 4. Object Safety Rules

Not all traits can be trait objects:

```rust
// ❌ Not object-safe - has generic method
trait NotObjectSafe {
    fn process<T>(&self, item: T);  // Generic method
}

// ❌ Not object-safe - returns Self
trait AlsoNotObjectSafe {
    fn clone_self(&self) -> Self;  // Self is not sized
}

// ✅ Object-safe
trait ObjectSafe {
    fn name(&self) -> &str;
    fn execute(&self) -> Result<String, String>;
}
```

**Rules**:
- No generic methods (vtable can't list all possible T)
- No `Self: Sized` bound
- Methods must have `&self` or `&mut self` receiver
- No associated types with generics

### 5. Box for Heap Allocation

`Box<dyn Trait>` owns trait object on heap:

```rust
let plugin: Box<dyn Plugin> = Box::new(GreeterPlugin { ... });
// Stack: Box (16 bytes - data ptr + vtable ptr)
// Heap: GreeterPlugin data
```

**Why Box**:
- Ownership: Box owns the data, can transfer ownership
- Sized: Box has known size, Vec can store it
- Heterogeneous: Different sized types in same collection

### 6. Heterogeneous Collections

Store different types in one collection:

```rust
// ❌ Can't do this - different sizes
let mut vec = Vec::new();
vec.push(GreeterPlugin { ... });      // 24 bytes
vec.push(CalculatorPlugin);           // 0 bytes
vec.push(FileReaderPlugin { ... });   // 24 bytes

// ✅ Box makes uniform size
let vec: Vec<Box<dyn Plugin>> = vec![
    Box::new(GreeterPlugin { ... }),      // Box: 16 bytes
    Box::new(CalculatorPlugin),           // Box: 16 bytes
    Box::new(FileReaderPlugin { ... }),   // Box: 16 bytes
];
```

**Benefit**: Iterate over different plugin types uniformly.

### 7. Monomorphization vs Code Reuse

Static dispatch generates code per type:

```rust
fn run<T: Plugin>(p: T) { p.execute(); }

run(GreeterPlugin { ... });     // Generates run_GreeterPlugin
run(CalculatorPlugin);           // Generates run_CalculatorPlugin
run(FileReaderPlugin { ... });   // Generates run_FileReaderPlugin

// Binary: 3 functions × ~500 bytes = 1.5KB
```

Dynamic dispatch reuses one function:

```rust
fn run(p: &dyn Plugin) { p.execute(); }

run(&GreeterPlugin { ... });
run(&CalculatorPlugin);
run(&FileReaderPlugin { ... });

// Binary: 1 function × ~500 bytes = 0.5KB
```

**Trade-off**: 100 plugins = 50KB (static) vs 0.5KB (dynamic).

### 8. Lifecycle Hooks Pattern

Initialize → Execute → Cleanup pattern:

```rust
trait Plugin {
    fn initialize(&mut self, config: &Config) -> Result<(), String>;
    fn execute(&self) -> Result<String, String>;
    fn cleanup(&mut self) -> Result<(), String>;
}

// Usage
let mut plugin = create_plugin();
plugin.initialize(&config)?;    // Setup resources
plugin.execute()?;              // Use plugin
plugin.cleanup()?;              // Release resources
```

**Benefit**: Resource management (files, connections, memory) handled explicitly.

### 9. Separation of Concerns

Split traits for different purposes:

```rust
// Metadata - immutable queries
trait PluginMetadata {
    fn author(&self) -> &str;
    fn description(&self) -> &str;
}

// Execution - mutable operations
trait Plugin {
    fn initialize(&mut self, config: &Config) -> Result<(), String>;
    fn execute(&self) -> Result<String, String>;
}
```

**Benefit**: Query metadata without requiring mutable access or execution.

### 10. Builder Pattern for Configuration

Fluent API for plugin setup:

```rust
let config = PluginConfig::new()
    .set("log_level", "DEBUG")
    .set("output_file", "app.log")
    .set("max_size", "10MB");

plugin.initialize(&config)?;
```

**vs Manual construction**:
```rust
let mut config = HashMap::new();
config.insert("log_level".to_string(), "DEBUG".to_string());
config.insert("output_file".to_string(), "app.log".to_string());
config.insert("max_size".to_string(), "10MB".to_string());
```

---

## Connection to This Project

Here's how each milestone applies these concepts to build a production-ready plugin system.

### Milestone 1: Basic Plugin Trait with Static Dispatch

**Concepts applied**:
- **Trait definition**: Common interface for all plugins
- **Static dispatch**: `run_plugin<T: Plugin>` generates code per type
- **Monomorphization**: Compiler creates separate function per plugin type

**Why this matters**: Foundation of polymorphism - define common behavior.

**Real-world impact**:
- Text editor: Syntax highlighting plugins all implement `SyntaxPlugin`
- Game engine: Enemy AI all implements `AIBehavior`
- Web framework: Middleware all implements `Middleware`

**Performance**: Zero overhead - direct function calls, can inline.

**Limitation**: Can't store mixed types in `Vec<_>`.

---

### Milestone 2: Trait Objects for Heterogeneous Collections

**Concepts applied**:
- **Trait objects**: `&dyn Plugin` and `Box<dyn Plugin>`
- **Fat pointers**: 16 bytes (data ptr + vtable ptr)
- **Vtable dispatch**: Function pointer lookup
- **Object safety**: Plugin trait must follow rules
- **Heterogeneous collections**: `Vec<Box<dyn Plugin>>`

**Why this matters**: Store different plugin types together, iterate uniformly.

**Comparison**:

| Aspect | Static Dispatch | Dynamic Dispatch |
|--------|----------------|------------------|
| Call overhead | 0ns (direct) | ~3ns (vtable lookup) |
| Binary size (100 plugins) | ~50KB | ~0.5KB | **100× smaller** |
| Inlining | Yes | No |
| Heterogeneous collections | No | Yes |
| Runtime loading | No | Yes |

**Real-world example**: VSCode extensions
- 10,000+ extensions available
- Load at runtime based on user selection
- Can't statically compile all extensions
- **Must use dynamic dispatch**

**Memory layout**:
```rust
Vec<Box<dyn Plugin>>
  [Box 16b][Box 16b][Box 16b]...
     ↓        ↓        ↓
  [Greeter][Calc][FileReader] (on heap)
```

---

### Milestone 3: Complete Plugin System with Lifecycle

**Concepts applied**:
- **Lifecycle hooks**: `initialize()`, `execute()`, `cleanup()`
- **Mutable state**: `&mut self` for initialization/cleanup
- **Configuration**: Pass settings via `PluginConfig`
- **Metadata separation**: `PluginMetadata` trait for queries
- **Error handling**: `Result` for recoverable failures

**Why this matters**: Production plugins need proper resource management.

**Lifecycle example**:
```rust
// Database connection plugin
impl Plugin for DatabasePlugin {
    fn initialize(&mut self, config: &PluginConfig) -> Result<(), String> {
        // Open database connection
        self.connection = Database::connect(config.get("url"))?;
        Ok(())
    }

    fn execute(&self) -> Result<String, String> {
        // Use connection
        self.connection.query("SELECT * FROM users")
    }

    fn cleanup(&mut self) -> Result<(), String> {
        // Close connection
        self.connection.close()?;
        Ok(())
    }
}
```

**Without lifecycle hooks**:
- Connection leak: Forget to close connections
- Initialization errors: Plugin crashes at first use
- No cleanup: Resources not released properly

**With lifecycle hooks**:
- Managed: Manager calls init/cleanup automatically
- Validated: Plugins can't execute without initialization
- Safe: Cleanup guaranteed even on errors

---

### Project-Wide Benefits

**Concrete comparisons** - Plugin system with 100 plugins:

| Metric | Static Only | Dynamic (M2) | Full Lifecycle (M3) | Improvement |
|--------|-------------|--------------|---------------------|-------------|
| Binary size | ~50KB | ~0.5KB | ~1KB | **50× smaller** |
| Can load at runtime | No | Yes | Yes | **Flexible** |
| Call overhead | 0ns | 3ns | 3ns | **Acceptable** |
| Resource management | Manual | Manual | Automatic | **Safe** |
| Configuration | Hardcoded | Hardcoded | Runtime config | **Flexible** |
| Memory per plugin | 0-24 bytes | 16 bytes (Box) | 16 bytes (Box) | **Uniform** |

**Real-world validation**:
- **VSCode**: Extensions loaded dynamically via JS engine
- **Firefox**: WebExtensions use similar plugin architecture
- **Vim**: Plugins loaded at startup with lifecycle hooks
- **Game engines**: Unity/Unreal use component-based plugins
- **Kubernetes**: Admission controllers as dynamic plugins

**Production requirements met**:
- ✅ Runtime loading (load plugins from config)
- ✅ Heterogeneous storage (different plugin types in one Vec)
- ✅ Type safety (compiler ensures trait implementation)
- ✅ Memory safety (no dangling pointers, automatic cleanup)
- ✅ Resource management (init/cleanup hooks)
- ✅ Configuration (pass settings at initialization)
- ✅ Metadata queries (author, version, dependencies)
- ✅ Small binary (dynamic dispatch prevents bloat)

**Performance characteristics**:
- Plugin loading: ~100μs per plugin (includes initialization)
- Plugin execution: 3ns overhead per call
- Memory overhead: 16 bytes per plugin (Box)
- Binary overhead: ~1KB total (vs 50KB static)

**Trade-offs understood**:
- **Slower execution**: 3ns per call (acceptable for plugins)
- **No inlining**: Vtable prevents optimization
- **Larger pointer**: 16 bytes vs 8 bytes
- **Worth it**: Runtime flexibility + small binary

This project teaches patterns used in production plugin systems powering extensible applications used by millions daily.

---

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


**Dynamic Dispatch is Critical When**:
- Types not known at compile-time (loading from disk/network)
- Binary size constrained (embedded systems, WebAssembly)
- Many implementations (100+ plugins → avoid code bloat)
- Hot-loading required (swap implementations at runtime)

---

### Milestone 1: Basic Plugin Trait with Static Dispatch

**Goal**: Define a plugin trait and implement it for several types.

**Architecture**
**trait** `Plugin`
**functions**
- `fn name()`  - for logging and plugin registry lookups
- `fn version()` - version compatibility checks and debugging
- `fn execute()` -  executes plugin, returns success message or error

**structs**  - impl Trait
- `GreeterPlugin`
   - **field**: `greeting` - customizable greeting message to display
- `CalculatorPlugin`

**functions**
- `fn run_plugin(plugin: &T)` - runs plugin, dispatches based on plugin type

**Starter Code**:
```rust
trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
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
        todo!()
    }
}

// CalculatorPlugin: Plugin that performs arithmetic operations
// Role: Demonstrates stateless plugin (zero-sized type)
struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    // name: Identifies this as the calculator plugin
    fn name(&self) -> &str {
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
        todo!()
    }
}

// run_plugin: Generic function using static dispatch
// Role: Executes any type implementing Plugin trait
// Note: Compiler generates separate copy for each concrete type (monomorphization)
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

### Why Milestone 1 Isn't Enough

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
        todo!()
    }
}

// run_plugin_dynamic: Executes plugin using dynamic dispatch
// Role: Demonstrates trait object usage with vtable lookup
// Note: Only one function generated (vs one per type with static dispatch)
fn run_plugin_dynamic(plugin: &dyn Plugin) {
    // TODO: Same as static version but takes trait object
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
        todo!()
    }

    // register: Adds plugin to manager
    // Role: Registers new plugin for execution
    // Note: Takes Box<dyn Plugin> for heap allocation and ownership
    fn register(&mut self, plugin: Box<dyn Plugin>) {
        // TODO: Add plugin to Vec
        todo!()
    }

    // run_all: Executes all registered plugins in order
    // Role: Batch plugin execution for startup/shutdown hooks
    fn run_all(&self) {
        // TODO: Iterate through plugins and run each one
        todo!()
    }

    // get_plugin: Finds plugin by name
    // Role: Plugin lookup for selective execution
    // Returns: Trait object reference if found, None otherwise
    fn get_plugin(&self, name: &str) -> Option<&dyn Plugin> {
        // TODO: Find plugin by name
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

### Why Milestone 2 Isn't Enough

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
        todo!()
    }

    // set: Adds or updates configuration value
    // Role: Sets plugin parameters (e.g., "log_level" => "DEBUG")
    fn set(&mut self, key: String, value: String) {
        // TODO: Insert key-value pair
        todo!()
    }

    // get: Retrieves configuration value by key
    // Role: Allows plugins to read their configuration
    fn get(&self, key: &str) -> Option<&str> {
        // TODO: Get value by key
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
        todo!()
    }

    // execute: Performs logging operation
    // Role: Returns log message if initialized, error otherwise
    fn execute(&self) -> Result<String, String> {
        // TODO: Check if initialized, return error if not
        // Otherwise, return log message with current level
        todo!()
    }

    // cleanup: Resets plugin to uninitialized state
    // Role: Prepares plugin for shutdown or reinitialization
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
        todo!()
    }

    // execute_plugin: Runs specific plugin by name
    // Role: Selective plugin execution
    fn execute_plugin(&self, name: &str) -> Result<String, String> {
        // TODO: Find plugin by name and execute it
        // Return Err if not found
        todo!()
    }

    // shutdown: Cleanly shuts down all plugins
    // Role: Calls cleanup on all plugins, collects errors, clears registry
    fn shutdown(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        // TODO: Call cleanup() on all plugins
        // Collect any errors (don't stop on first error)
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

### Complete Working Example

```rust
use std::collections::HashMap;

// =============================================================================
// Milestone 1 & 2: Core plugin trait, static dispatch, and dynamic dispatch
// =============================================================================

trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self) -> Result<String, String>;
}

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

struct CalculatorPlugin;

impl Plugin for CalculatorPlugin {
    fn name(&self) -> &str {
        "Calculator"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self) -> Result<String, String> {
        let lhs = 2;
        let rhs = 2;
        Ok(format!("{lhs} + {rhs} = {}", lhs + rhs))
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
        Ok(format!("Read content from '{}': <simulated>", self.path))
    }
}

fn run_plugin<T: Plugin>(plugin: &T) {
    println!("Running {} v{}", plugin.name(), plugin.version());
    match plugin.execute() {
        Ok(output) => println!("[ok] {output}"),
        Err(err) => println!("[err] {err}"),
    }
}

fn run_plugin_dynamic(plugin: &dyn Plugin) {
    println!("Running {} v{}", plugin.name(), plugin.version());
    match plugin.execute() {
        Ok(output) => println!("[ok] {output}"),
        Err(err) => println!("[err] {err}"),
    }
}

struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new() -> Self {
        Self { plugins: Vec::new() }
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
            .map(|p| p.as_ref())
    }
}

// =============================================================================
// Milestone 3: Lifecycle-aware plugin system
// =============================================================================

#[derive(Debug, Clone)]
struct PluginConfig {
    settings: HashMap<String, String>,
}

impl PluginConfig {
    fn new() -> Self {
        Self {
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

    fn initialize(&mut self, _config: &PluginConfig) -> Result<(), String> {
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
        Self {
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
        Self { plugins: Vec::new() }
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
        self
            .plugins
            .iter()
            .find(|p| p.name() == name)
            .ok_or_else(|| format!("Plugin '{}' not found", name))?
            .execute()
    }

    fn shutdown(&mut self) -> Vec<String> {
        let mut errors = Vec::new();

        for plugin in &mut self.plugins {
            if let Err(err) = plugin.cleanup() {
                errors.push(format!("{}: {}", plugin.name(), err));
            }
        }

        self.plugins.clear();
        errors
    }
}

fn main() {
    println!("=== Milestone 1: Static dispatch ===");
    let greeter = GreeterPlugin {
        greeting: "Hello from Rust!".to_string(),
    };
    let calculator = CalculatorPlugin;
    run_plugin(&greeter);
    run_plugin(&calculator);

    println!("\n=== Milestone 2: Dynamic dispatch and manager ===");
    let mut manager = PluginManager::new();
    manager.register(Box::new(GreeterPlugin {
        greeting: "Dynamic greeting".to_string(),
    }));
    manager.register(Box::new(CalculatorPlugin));
    manager.register(Box::new(FileReaderPlugin {
        path: "data.txt".to_string(),
    }));
    manager.run_all();

    if let Some(plugin) = manager.get_plugin("Calculator") {
        println!("Found plugin {} version {}", plugin.name(), plugin.version());
    }

    println!("\n=== Milestone 3: Lifecycle-aware manager ===");
    let mut enhanced_manager = EnhancedPluginManager::new();
    let mut config = PluginConfig::new();
    config.set("log_level".to_string(), "DEBUG".to_string());

    let logger = Box::new(LoggingPlugin::new());
    if let Err(err) = enhanced_manager.register_and_init(logger, &config) {
        eprintln!("Failed to init logger: {err}");
    }

    match enhanced_manager.execute_plugin("Logger") {
        Ok(msg) => println!("Logger output: {msg}"),
        Err(err) => eprintln!("Logger error: {err}"),
    }

    let errors = enhanced_manager.shutdown();
    if errors.is_empty() {
        println!("All plugins cleaned up successfully");
    } else {
        println!("Cleanup errors: {errors:?}");
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
        assert_eq!(greeter.version(), "1.0.0");
        assert_eq!(greeter.execute().unwrap(), "Test");

        let calculator = CalculatorPlugin;
        assert!(calculator.execute().unwrap().contains('='));
    }

    #[test]
    fn test_heterogeneous_collection() {
        let plugins: Vec<Box<dyn Plugin>> = vec![
            Box::new(GreeterPlugin {
                greeting: "Hello".to_string(),
            }),
            Box::new(CalculatorPlugin),
            Box::new(FileReaderPlugin {
                path: "input.txt".to_string(),
            }),
        ];
        assert_eq!(plugins.len(), 3);
        for plugin in plugins {
            assert!(plugin.execute().is_ok());
        }
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(CalculatorPlugin));
        assert!(manager.get_plugin("Calculator").is_some());
        assert!(manager.get_plugin("Missing").is_none());
    }

    #[test]
    fn test_plugin_config() {
        let mut config = PluginConfig::new();
        config.set("k".to_string(), "v".to_string());
        assert_eq!(config.get("k"), Some("v"));
        assert!(config.get("missing").is_none());
    }

    #[test]
    fn test_logging_plugin_lifecycle() {
        let mut plugin = LoggingPlugin::new();
        assert!(!plugin.initialized);
        assert_eq!(plugin.version(), "2.0.0");

        let mut config = PluginConfig::new();
        config.set("log_level".to_string(), "TRACE".to_string());
        plugin.initialize(&config).unwrap();
        assert!(plugin.initialized);
        assert_eq!(plugin.log_level, "TRACE");

        assert!(plugin.execute().unwrap().contains("TRACE"));
        plugin.cleanup().unwrap();
        assert!(!plugin.initialized);
    }

    #[test]
    fn test_enhanced_manager() {
        let mut manager = EnhancedPluginManager::new();
        let config = PluginConfig::new();
        manager
            .register_and_init(Box::new(LoggingPlugin::new()), &config)
            .unwrap();
        assert!(manager.execute_plugin("Logger").is_ok());
        let errors = manager.shutdown();
        assert!(errors.is_empty());
        assert_eq!(manager.plugins.len(), 0);
    }

    #[test]
    fn test_metadata_trait() {
        let plugin = LoggingPlugin::new();
        assert_eq!(plugin.author(), "Plugin Team");
        assert!(plugin.description().contains("logging"));
        assert!(plugin.dependencies().is_empty());
    }
}

```


