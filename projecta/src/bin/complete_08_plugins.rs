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
