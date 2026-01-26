// Pattern 3: Behavioral Patterns - Strategy, Observer, Command, Iterator
// Demonstrates patterns for communication between objects.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

// ============================================================================
// Example: Strategy Pattern with Trait Objects
// ============================================================================

trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
    fn decompress(&self, data: &[u8]) -> Vec<u8>;
    fn name(&self) -> &str;
}

struct ZipCompression;
impl CompressionStrategy for ZipCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        println!("ZIP compressing {} bytes", data.len());
        data.to_vec()
    }

    fn decompress(&self, data: &[u8]) -> Vec<u8> {
        println!("ZIP decompressing {} bytes", data.len());
        data.to_vec()
    }

    fn name(&self) -> &str {
        "ZIP"
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

    fn name(&self) -> &str {
        "RAR"
    }
}

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

    fn strategy_name(&self) -> &str {
        self.strategy.name()
    }
}

fn strategy_trait_object_example() {
    let data = vec![1, 2, 3, 4, 5];
    let mut compressor = FileCompressor::new(Box::new(ZipCompression));
    println!("Using strategy: {}", compressor.strategy_name());
    compressor.compress_file(&data);

    compressor.set_strategy(Box::new(RarCompression));
    println!("Switched to: {}", compressor.strategy_name());
    compressor.compress_file(&data);
}

// ============================================================================
// Example: Zero-cost Strategy with Generics
// ============================================================================

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

fn strategy_generic_example() {
    let data = vec![1, 2, 3, 4, 5];
    // Compile-time strategy selection, no heap allocation
    let compressor = StaticCompressor::new(ZipCompression);
    println!("Generic strategy:");
    compressor.compress_file(&data);
}

// ============================================================================
// Example: Functional Strategy with Closures
// ============================================================================

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

fn strategy_closure_example() {
    // Strategy as closure
    let zip_fn = |data: &[u8]| -> Vec<u8> {
        println!("Closure ZIP compressing {} bytes", data.len());
        data.to_vec()
    };
    let compressor = FunctionalCompressor::new(zip_fn);
    compressor.compress(&[1, 2, 3]);
}

// ============================================================================
// Example: Observer Pattern with Trait Objects
// ============================================================================

trait Observer {
    fn update(&mut self, temperature: f32);
}

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
        println!(
            "Logged: {}°C (total: {} readings)",
            temperature,
            self.log.len()
        );
    }
}

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

fn observer_trait_object_example() {
    let mut station = WeatherStation::new();

    let display = Arc::new(Mutex::new(TemperatureDisplay {
        name: "Main".to_string(),
    }));
    let logger = Arc::new(Mutex::new(TemperatureLogger { log: Vec::new() }));

    station.attach(display);
    station.attach(logger);

    station.set_temperature(25.5);
    station.set_temperature(26.0);
}

// ============================================================================
// Example: Channel-based Observer (More Idiomatic)
// ============================================================================

#[derive(Clone)]
struct TemperatureEvent {
    temperature: f32,
}

struct Publisher {
    subscribers: Vec<mpsc::Sender<TemperatureEvent>>,
}

impl Publisher {
    fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    fn subscribe(&mut self) -> mpsc::Receiver<TemperatureEvent> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    fn publish(&mut self, event: TemperatureEvent) {
        self.subscribers
            .retain(|tx| tx.send(event.clone()).is_ok());
    }
}

fn observer_channel_example() {
    let mut publisher = Publisher::new();

    let rx1 = publisher.subscribe();
    let rx2 = publisher.subscribe();

    let h1 = thread::spawn(move || {
        if let Ok(event) = rx1.recv() {
            println!("Observer 1: {}°C", event.temperature);
        }
    });

    let h2 = thread::spawn(move || {
        if let Ok(event) = rx2.recv() {
            println!("Observer 2: {}°C", event.temperature);
        }
    });

    publisher.publish(TemperatureEvent { temperature: 25.5 });

    h1.join().unwrap();
    h2.join().unwrap();
}

// ============================================================================
// Example: Command Pattern
// ============================================================================

trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}

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

struct WriteCommand {
    editor: Rc<RefCell<TextEditor>>,
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
    editor: Rc<RefCell<TextEditor>>,
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

fn command_example() {
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

    println!("After writes: {}", editor.borrow().get_content());

    history.undo();
    println!("After undo: {}", editor.borrow().get_content());

    history.redo();
    println!("After redo: {}", editor.borrow().get_content());
}

// ============================================================================
// Example: Functional Command Pattern
// ============================================================================

struct FunctionalCommand {
    execute_fn: Box<dyn FnMut()>,
    undo_fn: Box<dyn FnMut()>,
}

impl FunctionalCommand {
    fn new(execute_fn: Box<dyn FnMut()>, undo_fn: Box<dyn FnMut()>) -> Self {
        Self {
            execute_fn,
            undo_fn,
        }
    }

    fn execute(&mut self) {
        (self.execute_fn)();
    }

    #[allow(dead_code)]
    fn undo(&mut self) {
        (self.undo_fn)();
    }
}

fn functional_command_example() {
    let mut cmd = FunctionalCommand::new(
        Box::new(|| println!("Executing functional command")),
        Box::new(|| println!("Undoing functional command")),
    );
    cmd.execute();
}

// ============================================================================
// Example: Iterator Pattern - Custom Iterator
// ============================================================================

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

fn iterator_fibonacci_example() {
    let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("Fibonacci: {:?}", fibs);
}

// ============================================================================
// Example: Iterator Pattern - Custom Collection
// ============================================================================

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

fn iterator_collection_example() {
    let mut collection = MyCollection::new();
    collection.add("Item 1".to_string());
    collection.add("Item 2".to_string());

    println!("Borrowed iteration:");
    for item in &collection {
        println!("  {}", item);
    }

    println!("Owned iteration:");
    for item in collection {
        println!("  {}", item);
    }
    // collection moved
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_zip() {
        let compressor = FileCompressor::new(Box::new(ZipCompression));
        let data = vec![1, 2, 3];
        let result = compressor.compress_file(&data);
        assert_eq!(result, data);
        assert_eq!(compressor.strategy_name(), "ZIP");
    }

    #[test]
    fn test_strategy_switch() {
        let mut compressor = FileCompressor::new(Box::new(ZipCompression));
        assert_eq!(compressor.strategy_name(), "ZIP");

        compressor.set_strategy(Box::new(RarCompression));
        assert_eq!(compressor.strategy_name(), "RAR");
    }

    #[test]
    fn test_generic_strategy() {
        let compressor = StaticCompressor::new(ZipCompression);
        let data = vec![1, 2, 3];
        let result = compressor.compress_file(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_functional_strategy() {
        let compressor = FunctionalCompressor::new(|data: &[u8]| data.to_vec());
        let data = vec![1, 2, 3];
        let result = compressor.compress(&data);
        assert_eq!(result, data);
    }

    #[test]
    fn test_weather_station() {
        let mut station = WeatherStation::new();
        let display = Arc::new(Mutex::new(TemperatureDisplay {
            name: "Test".to_string(),
        }));
        station.attach(display);
        station.set_temperature(25.0);
    }

    #[test]
    fn test_publisher_subscribe() {
        let mut publisher = Publisher::new();
        let rx = publisher.subscribe();
        publisher.publish(TemperatureEvent { temperature: 20.0 });

        let event = rx.recv().unwrap();
        assert!((event.temperature - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_command_write() {
        let editor = Rc::new(RefCell::new(TextEditor::new()));
        let mut history = CommandHistory::new();

        history.execute(Box::new(WriteCommand {
            editor: editor.clone(),
            text: "Hello".to_string(),
        }));

        assert_eq!(editor.borrow().get_content(), "Hello");
    }

    #[test]
    fn test_command_undo() {
        let editor = Rc::new(RefCell::new(TextEditor::new()));
        let mut history = CommandHistory::new();

        history.execute(Box::new(WriteCommand {
            editor: editor.clone(),
            text: "Hello".to_string(),
        }));

        history.undo();
        assert_eq!(editor.borrow().get_content(), "");
    }

    #[test]
    fn test_command_redo() {
        let editor = Rc::new(RefCell::new(TextEditor::new()));
        let mut history = CommandHistory::new();

        history.execute(Box::new(WriteCommand {
            editor: editor.clone(),
            text: "Hello".to_string(),
        }));

        history.undo();
        history.redo();
        assert_eq!(editor.borrow().get_content(), "Hello");
    }

    #[test]
    fn test_fibonacci() {
        let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
        assert_eq!(fibs, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_collection_borrowed() {
        let mut collection = MyCollection::new();
        collection.add("a".to_string());
        collection.add("b".to_string());

        let items: Vec<&String> = (&collection).into_iter().collect();
        assert_eq!(items.len(), 2);

        // collection still available
        assert_eq!(collection.items.len(), 2);
    }

    #[test]
    fn test_collection_owned() {
        let mut collection = MyCollection::new();
        collection.add("a".to_string());
        collection.add("b".to_string());

        let items: Vec<String> = collection.into_iter().collect();
        assert_eq!(items, vec!["a", "b"]);
    }

    #[test]
    fn test_delete_command() {
        let editor = Rc::new(RefCell::new(TextEditor::new()));
        editor.borrow_mut().write("Hello World");

        let mut cmd = DeleteCommand {
            editor: editor.clone(),
            deleted_text: String::new(),
            count: 5,
        };

        cmd.execute();
        assert_eq!(editor.borrow().get_content(), "Hello ");

        cmd.undo();
        assert_eq!(editor.borrow().get_content(), "Hello World");
    }
}

fn main() {
    println!("Pattern 3: Behavioral Patterns");
    println!("===============================\n");

    println!("=== Strategy Pattern (Trait Objects) ===");
    strategy_trait_object_example();
    println!();

    println!("=== Strategy Pattern (Generics) ===");
    strategy_generic_example();
    println!();

    println!("=== Strategy Pattern (Closures) ===");
    strategy_closure_example();
    println!();

    println!("=== Observer Pattern (Trait Objects) ===");
    observer_trait_object_example();
    println!();

    println!("=== Observer Pattern (Channels) ===");
    observer_channel_example();
    println!();

    println!("=== Command Pattern ===");
    command_example();
    println!();

    println!("=== Functional Command Pattern ===");
    functional_command_example();
    println!();

    println!("=== Iterator Pattern (Fibonacci) ===");
    iterator_fibonacci_example();
    println!();

    println!("=== Iterator Pattern (Custom Collection) ===");
    iterator_collection_example();
}
