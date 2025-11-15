
# Cookbook: Mutating Operations in Rust

> **Real-world recipes for efficient in-place data manipulation**
>
> These recipes show how to modify collections efficiently using Rust's mutating methods, perfect for performance-critical applications and memory-constrained environments.

## Table of Contents

1. [Stack and Queue Operations](#stack-and-queue-operations)
2. [Shopping Cart and Order Management](#shopping-cart-and-order-management)
3. [Task Queue and Job Processing](#task-queue-and-job-processing)
4. [Undo/Redo Systems](#undoredo-systems)
5. [Cache Management](#cache-management)
6. [Playlist and Media Management](#playlist-and-media-management)
7. [Inventory and Stock Management](#inventory-and-stock-management)
8. [Real-time Data Processing](#real-time-data-processing)
9. [State Machine Patterns](#state-machine-patterns)
10. [Collection Optimization](#collection-optimization)
11. [Sorting and Organization](#sorting-and-organization)
12. [Batch Operations](#batch-operations)
13. [Design Patterns](#design-patterns)
14. [Quick Reference](#quick-reference)

---

## Stack and Queue Operations

### Recipe 1: Browser History (Stack Pattern)

**Problem**: Implement browser back/forward navigation.

**Use Case**: Web browser maintaining navigation history with back button support.

**Design Pattern**: Stack (LIFO)

```rust
struct BrowserHistory {
    history: Vec<String>,
    current_index: usize,
}

impl BrowserHistory {
    fn new(homepage: String) -> Self {
        BrowserHistory {
            history: vec![homepage],
            current_index: 0,
        }
    }

    fn visit(&mut self, url: String) {
        // Remove forward history when visiting new page
        self.history.truncate(self.current_index + 1);
        self.history.push(url);
        self.current_index += 1;
    }

    fn back(&mut self) -> Option<&String> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.history[self.current_index])
        } else {
            None
        }
    }

    fn forward(&mut self) -> Option<&String> {
        if self.current_index < self.history.len() - 1 {
            self.current_index += 1;
            Some(&self.history[self.current_index])
        } else {
            None
        }
    }

    fn current(&self) -> &String {
        &self.history[self.current_index]
    }
}

fn main() {
    let mut browser = BrowserHistory::new("google.com".to_string());

    browser.visit("reddit.com".to_string());
    browser.visit("github.com".to_string());

    println!("Current: {}", browser.current());

    if let Some(url) = browser.back() {
        println!("Back to: {}", url);
    }

    if let Some(url) = browser.forward() {
        println!("Forward to: {}", url);
    }
}
```

**When to use**: Navigation systems, undo/redo, command history, call stacks.

---

### Recipe 2: Print Job Queue (Queue Pattern)

**Problem**: Manage print jobs in a FIFO queue.

**Use Case**: Office printer managing multiple print requests.

**Design Pattern**: Queue (FIFO)

```rust
#[derive(Debug)]
struct PrintJob {
    id: u32,
    document: String,
    pages: u32,
}

struct PrintQueue {
    jobs: Vec<PrintJob>,
}

impl PrintQueue {
    fn new() -> Self {
        PrintQueue { jobs: Vec::new() }
    }

    fn add_job(&mut self, job: PrintJob) {
        println!("📄 Added to queue: {} ({} pages)", job.document, job.pages);
        self.jobs.push(job);
    }

    fn process_next(&mut self) -> Option<PrintJob> {
        if !self.jobs.is_empty() {
            Some(self.jobs.remove(0))
        } else {
            None
        }
    }

    fn cancel_job(&mut self, job_id: u32) -> bool {
        if let Some(pos) = self.jobs.iter().position(|j| j.id == job_id) {
            let job = self.jobs.remove(pos);
            println!("❌ Cancelled: {}", job.document);
            true
        } else {
            false
        }
    }

    fn pending_count(&self) -> usize {
        self.jobs.len()
    }

    fn priority_insert(&mut self, job: PrintJob) {
        // Insert at front for priority
        self.jobs.insert(0, job);
    }
}

fn main() {
    let mut queue = PrintQueue::new();

    queue.add_job(PrintJob { id: 1, document: "Report.pdf".into(), pages: 10 });
    queue.add_job(PrintJob { id: 2, document: "Invoice.pdf".into(), pages: 2 });
    queue.add_job(PrintJob { id: 3, document: "Manual.pdf".into(), pages: 50 });

    // Priority job
    queue.priority_insert(PrintJob { id: 4, document: "URGENT.pdf".into(), pages: 1 });

    println!("\n🖨️  Processing jobs ({} pending):", queue.pending_count());

    while let Some(job) = queue.process_next() {
        println!("  Printing: {} ({} pages)", job.document, job.pages);
    }
}
```

**When to use**: Task scheduling, message queues, request handling, job processing.

---

## Shopping Cart and Order Management

### Recipe 3: E-commerce Shopping Cart

**Problem**: Add, remove, and update items in a shopping cart.

**Use Case**: Online store where users manage their cart before checkout.

```rust
#[derive(Debug, Clone)]
struct CartItem {
    product_id: u32,
    name: String,
    price: f64,
    quantity: u32,
}

struct ShoppingCart {
    items: Vec<CartItem>,
}

impl ShoppingCart {
    fn new() -> Self {
        ShoppingCart { items: Vec::new() }
    }

    fn add_item(&mut self, product_id: u32, name: String, price: f64, quantity: u32) {
        // Check if item already exists
        if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product_id) {
            item.quantity += quantity;
            println!("Updated quantity: {} ({})", item.name, item.quantity);
        } else {
            self.items.push(CartItem { product_id, name: name.clone(), price, quantity });
            println!("Added to cart: {}", name);
        }
    }

    fn remove_item(&mut self, product_id: u32) -> bool {
        if let Some(pos) = self.items.iter().position(|i| i.product_id == product_id) {
            let item = self.items.remove(pos);
            println!("Removed: {}", item.name);
            true
        } else {
            false
        }
    }

    fn update_quantity(&mut self, product_id: u32, new_quantity: u32) -> bool {
        if let Some(item) = self.items.iter_mut().find(|i| i.product_id == product_id) {
            if new_quantity == 0 {
                return false; // Use remove_item instead
            }
            item.quantity = new_quantity;
            println!("Updated {}: {} items", item.name, new_quantity);
            true
        } else {
            false
        }
    }

    fn clear(&mut self) {
        self.items.clear();
        println!("Cart cleared");
    }

    fn total(&self) -> f64 {
        self.items.iter()
            .map(|item| item.price * item.quantity as f64)
            .sum()
    }

    fn apply_discount(&mut self, percentage: f64) {
        for item in self.items.iter_mut() {
            item.price *= 1.0 - (percentage / 100.0);
        }
        println!("Applied {}% discount", percentage);
    }

    fn get_summary(&self) -> String {
        let mut summary = String::from("Shopping Cart:\n");
        for item in &self.items {
            summary.push_str(&format!(
                "  {} x{} @ ${:.2} = ${:.2}\n",
                item.name, item.quantity, item.price,
                item.price * item.quantity as f64
            ));
        }
        summary.push_str(&format!("Total: ${:.2}", self.total()));
        summary
    }
}

fn main() {
    let mut cart = ShoppingCart::new();

    cart.add_item(1, "Laptop".into(), 999.99, 1);
    cart.add_item(2, "Mouse".into(), 29.99, 2);
    cart.add_item(1, "Laptop".into(), 999.99, 1); // Add another laptop

    cart.update_quantity(2, 3); // Change mouse quantity

    cart.apply_discount(10.0); // 10% off

    println!("\n{}", cart.get_summary());

    cart.remove_item(2);
    println!("\nAfter removing mouse:");
    println!("{}", cart.get_summary());
}
```

**When to use**: E-commerce carts, order builders, item collections, wish lists.

---

### Recipe 4: Order Fulfillment Pipeline

**Problem**: Process orders through multiple stages with status updates.

**Use Case**: Warehouse management system tracking order fulfillment.

```rust
#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
}

#[derive(Debug)]
struct Order {
    id: u32,
    customer: String,
    items: Vec<String>,
    status: OrderStatus,
}

struct OrderPipeline {
    orders: Vec<Order>,
}

impl OrderPipeline {
    fn new() -> Self {
        OrderPipeline { orders: Vec::new() }
    }

    fn add_order(&mut self, order: Order) {
        println!("📦 New order #{} from {}", order.id, order.customer);
        self.orders.push(order);
    }

    fn advance_order(&mut self, order_id: u32) -> bool {
        if let Some(order) = self.orders.iter_mut().find(|o| o.id == order_id) {
            order.status = match order.status {
                OrderStatus::Pending => OrderStatus::Processing,
                OrderStatus::Processing => OrderStatus::Shipped,
                OrderStatus::Shipped => OrderStatus::Delivered,
                OrderStatus::Delivered => return false,
            };
            println!("Order #{}: {:?}", order_id, order.status);
            true
        } else {
            false
        }
    }

    fn remove_delivered(&mut self) -> usize {
        let before = self.orders.len();
        self.orders.retain(|order| order.status != OrderStatus::Delivered);
        let removed = before - self.orders.len();
        println!("Archived {} delivered orders", removed);
        removed
    }

    fn get_by_status(&self, status: OrderStatus) -> Vec<&Order> {
        self.orders.iter()
            .filter(|o| o.status == status)
            .collect()
    }

    fn bulk_ship(&mut self) {
        for order in self.orders.iter_mut() {
            if order.status == OrderStatus::Processing {
                order.status = OrderStatus::Shipped;
            }
        }
        println!("🚚 Shipped all processing orders");
    }
}

fn main() {
    let mut pipeline = OrderPipeline::new();

    pipeline.add_order(Order {
        id: 1001,
        customer: "Alice".into(),
        items: vec!["Laptop".into()],
        status: OrderStatus::Pending,
    });

    pipeline.add_order(Order {
        id: 1002,
        customer: "Bob".into(),
        items: vec!["Mouse".into(), "Keyboard".into()],
        status: OrderStatus::Pending,
    });

    // Advance orders through pipeline
    pipeline.advance_order(1001); // Pending -> Processing
    pipeline.advance_order(1001); // Processing -> Shipped
    pipeline.advance_order(1002); // Pending -> Processing

    pipeline.bulk_ship(); // Ship all processing

    println!("\nShipped orders:");
    for order in pipeline.get_by_status(OrderStatus::Shipped) {
        println!("  Order #{}: {}", order.id, order.customer);
    }

    pipeline.advance_order(1001); // Shipped -> Delivered
    pipeline.remove_delivered();
}
```

**When to use**: Order processing, workflow management, pipeline systems, state transitions.

---

## Task Queue and Job Processing

### Recipe 5: Background Job Processor

**Problem**: Manage background tasks with priority and retry logic.

**Use Case**: Application server processing background jobs like email sending, image processing.

```rust
use std::collections::VecDeque;

#[derive(Debug, Clone)]
enum JobPriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
struct Job {
    id: u32,
    task: String,
    priority: JobPriority,
    retry_count: u32,
    max_retries: u32,
}

struct JobQueue {
    jobs: VecDeque<Job>,
}

impl JobQueue {
    fn new() -> Self {
        JobQueue { jobs: VecDeque::new() }
    }

    fn enqueue(&mut self, job: Job) {
        // Insert based on priority
        let position = self.jobs.iter()
            .position(|j| Self::priority_value(&j.priority) < Self::priority_value(&job.priority))
            .unwrap_or(self.jobs.len());

        self.jobs.insert(position, job);
    }

    fn priority_value(priority: &JobPriority) -> u32 {
        match priority {
            JobPriority::Low => 1,
            JobPriority::Normal => 2,
            JobPriority::High => 3,
            JobPriority::Critical => 4,
        }
    }

    fn dequeue(&mut self) -> Option<Job> {
        self.jobs.pop_front()
    }

    fn retry_job(&mut self, mut job: Job) -> bool {
        if job.retry_count < job.max_retries {
            job.retry_count += 1;
            println!("  ⟳ Retrying job #{} (attempt {}/{})",
                job.id, job.retry_count, job.max_retries);
            self.enqueue(job);
            true
        } else {
            println!("  ✗ Job #{} exceeded max retries", job.id);
            false
        }
    }

    fn process_batch(&mut self, batch_size: usize) -> Vec<Job> {
        let mut processed = Vec::new();
        for _ in 0..batch_size {
            if let Some(job) = self.dequeue() {
                processed.push(job);
            } else {
                break;
            }
        }
        processed
    }

    fn size(&self) -> usize {
        self.jobs.len()
    }
}

fn main() {
    let mut queue = JobQueue::new();

    // Add jobs
    queue.enqueue(Job {
        id: 1,
        task: "Send welcome email".into(),
        priority: JobPriority::Normal,
        retry_count: 0,
        max_retries: 3,
    });

    queue.enqueue(Job {
        id: 2,
        task: "Process payment".into(),
        priority: JobPriority::Critical,
        retry_count: 0,
        max_retries: 5,
    });

    queue.enqueue(Job {
        id: 3,
        task: "Generate thumbnail".into(),
        priority: JobPriority::Low,
        retry_count: 0,
        max_retries: 2,
    });

    println!("Queue size: {}", queue.size());

    // Process jobs
    while let Some(job) = queue.dequeue() {
        println!("\n🔨 Processing: {} (Priority: {:?})", job.task, job.priority);

        // Simulate processing (fail payment for demo)
        if job.id == 2 && job.retry_count == 0 {
            println!("  ✗ Failed");
            queue.retry_job(job);
        } else {
            println!("  ✓ Success");
        }
    }
}
```

**When to use**: Background job processing, task scheduling, email queues, retry mechanisms.

---

## Undo/Redo Systems

### Recipe 6: Text Editor with Undo/Redo

**Problem**: Implement undo/redo functionality for text editing.

**Use Case**: Text editor, drawing application, or any system requiring command history.

**Design Pattern**: Command Pattern with Memento

```rust
#[derive(Debug, Clone)]
enum EditCommand {
    Insert { position: usize, text: String },
    Delete { position: usize, length: usize },
    Replace { position: usize, old: String, new: String },
}

struct TextEditor {
    content: String,
    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
}

impl TextEditor {
    fn new() -> Self {
        TextEditor {
            content: String::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    fn insert(&mut self, position: usize, text: String) {
        self.content.insert_str(position, &text);
        self.undo_stack.push(EditCommand::Insert {
            position,
            text: text.clone(),
        });
        self.redo_stack.clear(); // Clear redo on new action
    }

    fn delete(&mut self, position: usize, length: usize) {
        let deleted: String = self.content
            .chars()
            .skip(position)
            .take(length)
            .collect();

        self.content = self.content
            .chars()
            .enumerate()
            .filter(|(i, _)| *i < position || *i >= position + length)
            .map(|(_, c)| c)
            .collect();

        self.undo_stack.push(EditCommand::Delete { position, length });
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> bool {
        if let Some(command) = self.undo_stack.pop() {
            match &command {
                EditCommand::Insert { position, text } => {
                    // Remove inserted text
                    let len = text.chars().count();
                    self.content = self.content
                        .chars()
                        .enumerate()
                        .filter(|(i, _)| *i < *position || *i >= position + len)
                        .map(|(_, c)| c)
                        .collect();
                }
                EditCommand::Delete { position, .. } => {
                    // Can't restore deleted text in this simple implementation
                    // In real editor, you'd save the deleted content
                }
                _ => {}
            }
            self.redo_stack.push(command);
            true
        } else {
            false
        }
    }

    fn redo(&mut self) -> bool {
        if let Some(command) = self.redo_stack.pop() {
            match &command {
                EditCommand::Insert { position, text } => {
                    self.content.insert_str(*position, text);
                }
                _ => {}
            }
            self.undo_stack.push(command);
            true
        } else {
            false
        }
    }

    fn get_content(&self) -> &str {
        &self.content
    }
}

fn main() {
    let mut editor = TextEditor::new();

    editor.insert(0, "Hello".to_string());
    println!("After insert: '{}'", editor.get_content());

    editor.insert(5, " World".to_string());
    println!("After insert: '{}'", editor.get_content());

    editor.undo();
    println!("After undo: '{}'", editor.get_content());

    editor.redo();
    println!("After redo: '{}'", editor.get_content());
}
```

**When to use**: Editors, drawing apps, form builders, any system with reversible actions.

---

## Cache Management

### Recipe 7: LRU Cache Implementation

**Problem**: Implement a Least Recently Used cache with bounded size.

**Use Case**: Web application caching frequently accessed data.

**Design Pattern**: LRU Cache

```rust
use std::collections::HashMap;

struct LRUCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    access_order: Vec<K>,
}

impl<K: Clone + Eq + std::hash::Hash, V> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            cache: HashMap::new(),
            access_order: Vec::new(),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if self.cache.contains_key(key) {
            // Update access order
            self.access_order.retain(|k| k != key);
            self.access_order.push(key.clone());
            self.cache.get(key)
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        if self.cache.contains_key(&key) {
            // Update existing
            self.access_order.retain(|k| k != &key);
        } else if self.cache.len() >= self.capacity {
            // Evict least recently used
            if let Some(lru_key) = self.access_order.first().cloned() {
                self.cache.remove(&lru_key);
                self.access_order.remove(0);
                println!("Evicted LRU item");
            }
        }

        self.cache.insert(key.clone(), value);
        self.access_order.push(key);
    }

    fn size(&self) -> usize {
        self.cache.len()
    }

    fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
    }
}

fn main() {
    let mut cache = LRUCache::new(3);

    cache.put("user:1", "Alice");
    cache.put("user:2", "Bob");
    cache.put("user:3", "Carol");

    println!("Cache size: {}", cache.size());

    // Access user:1 (mark as recently used)
    if let Some(name) = cache.get(&"user:1") {
        println!("Found: {}", name);
    }

    // This will evict user:2 (least recently used)
    cache.put("user:4", "Dave");

    // user:2 should be evicted
    println!("user:2 in cache: {}", cache.get(&"user:2").is_some());
    println!("user:4 in cache: {}", cache.get(&"user:4").is_some());
}
```

**When to use**: Database query caching, session storage, API response caching, memoization.

---

## Playlist and Media Management

### Recipe 8: Music Playlist Manager

**Problem**: Manage a music playlist with shuffle, repeat, and reordering.

**Use Case**: Music player application managing song queues.

```rust
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone)]
struct Song {
    id: u32,
    title: String,
    artist: String,
    duration_secs: u32,
}

struct Playlist {
    name: String,
    songs: Vec<Song>,
    current_index: usize,
}

impl Playlist {
    fn new(name: String) -> Self {
        Playlist {
            name,
            songs: Vec::new(),
            current_index: 0,
        }
    }

    fn add_song(&mut self, song: Song) {
        println!("➕ Added: {} - {}", song.artist, song.title);
        self.songs.push(song);
    }

    fn remove_song(&mut self, song_id: u32) -> bool {
        if let Some(pos) = self.songs.iter().position(|s| s.id == song_id) {
            let song = self.songs.remove(pos);
            println!("➖ Removed: {}", song.title);

            // Adjust current index if necessary
            if pos <= self.current_index && self.current_index > 0 {
                self.current_index -= 1;
            }
            true
        } else {
            false
        }
    }

    fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.songs.shuffle(&mut rng);
        self.current_index = 0;
        println!("🔀 Playlist shuffled");
    }

    fn move_song(&mut self, from: usize, to: usize) {
        if from < self.songs.len() && to < self.songs.len() {
            let song = self.songs.remove(from);
            self.songs.insert(to, song);
            println!("Moved song from position {} to {}", from, to);
        }
    }

    fn sort_by_artist(&mut self) {
        self.songs.sort_by(|a, b| a.artist.cmp(&b.artist));
        println!("Sorted by artist");
    }

    fn reverse(&mut self) {
        self.songs.reverse();
        println!("Playlist reversed");
    }

    fn next(&mut self) -> Option<&Song> {
        if self.current_index < self.songs.len() - 1 {
            self.current_index += 1;
            Some(&self.songs[self.current_index])
        } else {
            None
        }
    }

    fn current(&self) -> Option<&Song> {
        self.songs.get(self.current_index)
    }

    fn total_duration(&self) -> u32 {
        self.songs.iter().map(|s| s.duration_secs).sum()
    }

    fn clear(&mut self) {
        self.songs.clear();
        self.current_index = 0;
        println!("Playlist cleared");
    }
}

fn main() {
    let mut playlist = Playlist::new("My Favorites".into());

    playlist.add_song(Song {
        id: 1,
        title: "Song A".into(),
        artist: "Artist 1".into(),
        duration_secs: 180,
    });

    playlist.add_song(Song {
        id: 2,
        title: "Song B".into(),
        artist: "Artist 2".into(),
        duration_secs: 200,
    });

    playlist.add_song(Song {
        id: 3,
        title: "Song C".into(),
        artist: "Artist 1".into(),
        duration_secs: 210,
    });

    println!("\n📊 Total duration: {} minutes", playlist.total_duration() / 60);

    playlist.shuffle();
    playlist.sort_by_artist();

    if let Some(song) = playlist.current() {
        println!("\n🎵 Now playing: {} - {}", song.artist, song.title);
    }
}
```

**When to use**: Media players, playlist management, queue systems, content rotation.

---

## Inventory and Stock Management

### Recipe 9: Warehouse Inventory System

**Problem**: Track inventory with stock updates, low stock alerts, and reordering.

**Use Case**: Warehouse management system tracking product inventory.

```rust
#[derive(Debug)]
struct InventoryItem {
    sku: String,
    name: String,
    quantity: i32,
    reorder_level: i32,
    price: f64,
}

struct Warehouse {
    inventory: Vec<InventoryItem>,
}

impl Warehouse {
    fn new() -> Self {
        Warehouse { inventory: Vec::new() }
    }

    fn add_product(&mut self, item: InventoryItem) {
        println!("📦 Added product: {} (SKU: {})", item.name, item.sku);
        self.inventory.push(item);
    }

    fn restock(&mut self, sku: &str, quantity: i32) -> bool {
        if let Some(item) = self.inventory.iter_mut().find(|i| i.sku == sku) {
            item.quantity += quantity;
            println!("↑ Restocked {}: +{} units (total: {})",
                item.name, quantity, item.quantity);
            true
        } else {
            false
        }
    }

    fn sell(&mut self, sku: &str, quantity: i32) -> Result<f64, String> {
        if let Some(item) = self.inventory.iter_mut().find(|i| i.sku == sku) {
            if item.quantity >= quantity {
                item.quantity -= quantity;
                let total = item.price * quantity as f64;

                println!("↓ Sold {}: -{} units (remaining: {})",
                    item.name, quantity, item.quantity);

                // Check if below reorder level
                if item.quantity <= item.reorder_level {
                    println!("⚠️  LOW STOCK ALERT: {} ({})",
                        item.name, item.quantity);
                }

                Ok(total)
            } else {
                Err(format!("Insufficient stock: {} available, {} requested",
                    item.quantity, quantity))
            }
        } else {
            Err("Product not found".to_string())
        }
    }

    fn get_low_stock_items(&self) -> Vec<&InventoryItem> {
        self.inventory.iter()
            .filter(|item| item.quantity <= item.reorder_level)
            .collect()
    }

    fn remove_discontinued(&mut self, sku: &str) -> bool {
        if let Some(pos) = self.inventory.iter().position(|i| i.sku == sku) {
            let item = self.inventory.remove(pos);
            println!("🗑️  Removed discontinued: {}", item.name);
            true
        } else {
            false
        }
    }

    fn adjust_inventory(&mut self, sku: &str, new_quantity: i32) -> bool {
        if let Some(item) = self.inventory.iter_mut().find(|i| i.sku == sku) {
            let diff = new_quantity - item.quantity;
            item.quantity = new_quantity;
            println!("Adjusted {}: {} units (diff: {:+})",
                item.name, new_quantity, diff);
            true
        } else {
            false
        }
    }

    fn total_value(&self) -> f64 {
        self.inventory.iter()
            .map(|item| item.price * item.quantity as f64)
            .sum()
    }
}

fn main() {
    let mut warehouse = Warehouse::new();

    warehouse.add_product(InventoryItem {
        sku: "LAP001".into(),
        name: "Laptop".into(),
        quantity: 50,
        reorder_level: 10,
        price: 999.99,
    });

    warehouse.add_product(InventoryItem {
        sku: "MOU001".into(),
        name: "Mouse".into(),
        quantity: 15,
        reorder_level: 20,
        price: 29.99,
    });

    // Sell some items
    match warehouse.sell("LAP001", 45) {
        Ok(total) => println!("Sale total: ${:.2}", total),
        Err(e) => println!("Error: {}", e),
    }

    // Check low stock
    println!("\n📋 Low stock items:");
    for item in warehouse.get_low_stock_items() {
        println!("  {} - {} units (reorder at {})",
            item.name, item.quantity, item.reorder_level);
    }

    warehouse.restock("LAP001", 30);

    println!("\n💰 Total inventory value: ${:.2}", warehouse.total_value());
}
```

**When to use**: Inventory management, stock tracking, warehouse systems, retail POS.

---

## Real-time Data Processing

### Recipe 10: Sliding Window Analytics

**Problem**: Calculate moving statistics on streaming data.

**Use Case**: Real-time monitoring dashboard showing recent metrics.

```rust
use std::collections::VecDeque;

struct SlidingWindow {
    data: VecDeque<f64>,
    max_size: usize,
}

impl SlidingWindow {
    fn new(max_size: usize) -> Self {
        SlidingWindow {
            data: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    fn push(&mut self, value: f64) {
        if self.data.len() >= self.max_size {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    fn average(&self) -> f64 {
        if self.data.is_empty() {
            0.0
        } else {
            self.data.iter().sum::<f64>() / self.data.len() as f64
        }
    }

    fn min(&self) -> Option<f64> {
        self.data.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }

    fn max(&self) -> Option<f64> {
        self.data.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied()
    }

    fn recent(&self, n: usize) -> Vec<f64> {
        self.data.iter()
            .rev()
            .take(n)
            .copied()
            .collect()
    }
}

struct MetricsCollector {
    response_times: SlidingWindow,
    error_counts: VecDeque<u32>,
    window_size: usize,
}

impl MetricsCollector {
    fn new(window_size: usize) -> Self {
        MetricsCollector {
            response_times: SlidingWindow::new(window_size),
            error_counts: VecDeque::with_capacity(window_size),
            window_size,
        }
    }

    fn record_request(&mut self, response_time_ms: f64, is_error: bool) {
        self.response_times.push(response_time_ms);

        if self.error_counts.len() >= self.window_size {
            self.error_counts.pop_front();
        }
        self.error_counts.push_back(if is_error { 1 } else { 0 });
    }

    fn get_metrics(&self) -> Metrics {
        Metrics {
            avg_response_time: self.response_times.average(),
            min_response_time: self.response_times.min().unwrap_or(0.0),
            max_response_time: self.response_times.max().unwrap_or(0.0),
            error_rate: self.calculate_error_rate(),
        }
    }

    fn calculate_error_rate(&self) -> f64 {
        if self.error_counts.is_empty() {
            0.0
        } else {
            let errors: u32 = self.error_counts.iter().sum();
            (errors as f64 / self.error_counts.len() as f64) * 100.0
        }
    }
}

#[derive(Debug)]
struct Metrics {
    avg_response_time: f64,
    min_response_time: f64,
    max_response_time: f64,
    error_rate: f64,
}

fn main() {
    let mut collector = MetricsCollector::new(10);

    // Simulate incoming requests
    let requests = vec![
        (50.0, false),
        (45.0, false),
        (120.0, true),
        (55.0, false),
        (60.0, false),
        (200.0, true),
        (48.0, false),
    ];

    for (response_time, is_error) in requests {
        collector.record_request(response_time, is_error);
    }

    let metrics = collector.get_metrics();
    println!("📊 Current Metrics:");
    println!("  Avg Response Time: {:.2}ms", metrics.avg_response_time);
    println!("  Min/Max: {:.2}ms / {:.2}ms",
        metrics.min_response_time, metrics.max_response_time);
    println!("  Error Rate: {:.2}%", metrics.error_rate);
}
```

**When to use**: Real-time analytics, monitoring dashboards, metrics collection, time-series data.

---

## State Machine Patterns

### Recipe 11: Connection State Manager

**Problem**: Manage connection lifecycle with state transitions.

**Use Case**: Network connection manager handling connect/disconnect/retry logic.

```rust
#[derive(Debug, PartialEq, Clone)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Failed,
}

struct Connection {
    id: u32,
    state: ConnectionState,
    retry_count: u32,
    max_retries: u32,
}

impl Connection {
    fn new(id: u32, max_retries: u32) -> Self {
        Connection {
            id,
            state: ConnectionState::Disconnected,
            retry_count: 0,
            max_retries,
        }
    }

    fn connect(&mut self) -> bool {
        match self.state {
            ConnectionState::Disconnected | ConnectionState::Failed => {
                self.state = ConnectionState::Connecting;
                println!("Connection {}: Connecting...", self.id);
                true
            }
            _ => false,
        }
    }

    fn on_connect_success(&mut self) {
        if self.state == ConnectionState::Connecting ||
           self.state == ConnectionState::Reconnecting {
            self.state = ConnectionState::Connected;
            self.retry_count = 0;
            println!("Connection {}: ✓ Connected", self.id);
        }
    }

    fn on_connect_failure(&mut self) -> bool {
        if self.retry_count < self.max_retries {
            self.retry_count += 1;
            self.state = ConnectionState::Reconnecting;
            println!("Connection {}: ⟳ Retry {}/{}",
                self.id, self.retry_count, self.max_retries);
            true
        } else {
            self.state = ConnectionState::Failed;
            println!("Connection {}: ✗ Failed after {} retries",
                self.id, self.max_retries);
            false
        }
    }

    fn disconnect(&mut self) {
        if self.state == ConnectionState::Connected {
            self.state = ConnectionState::Disconnected;
            println!("Connection {}: Disconnected", self.id);
        }
    }

    fn reset(&mut self) {
        self.state = ConnectionState::Disconnected;
        self.retry_count = 0;
    }
}

struct ConnectionPool {
    connections: Vec<Connection>,
}

impl ConnectionPool {
    fn new() -> Self {
        ConnectionPool { connections: Vec::new() }
    }

    fn add_connection(&mut self, max_retries: u32) -> u32 {
        let id = self.connections.len() as u32 + 1;
        self.connections.push(Connection::new(id, max_retries));
        id
    }

    fn get_available(&mut self) -> Option<&mut Connection> {
        self.connections.iter_mut()
            .find(|c| c.state == ConnectionState::Connected)
    }

    fn cleanup_failed(&mut self) -> usize {
        let before = self.connections.len();
        self.connections.retain(|c| c.state != ConnectionState::Failed);
        before - self.connections.len()
    }

    fn connection_count_by_state(&self, state: ConnectionState) -> usize {
        self.connections.iter()
            .filter(|c| c.state == state)
            .count()
    }
}

fn main() {
    let mut pool = ConnectionPool::new();

    let id1 = pool.add_connection(3);
    let id2 = pool.add_connection(3);

    // Simulate connection attempts
    if let Some(conn) = pool.connections.get_mut(0) {
        conn.connect();
        conn.on_connect_failure(); // First attempt fails
        conn.on_connect_failure(); // Second attempt fails
        conn.on_connect_success(); // Third attempt succeeds
    }

    if let Some(conn) = pool.connections.get_mut(1) {
        conn.connect();
        conn.on_connect_success(); // Succeeds immediately
    }

    println!("\n📊 Pool Status:");
    println!("  Connected: {}",
        pool.connection_count_by_state(ConnectionState::Connected));
    println!("  Failed: {}",
        pool.connection_count_by_state(ConnectionState::Failed));
}
```

**When to use**: Network connections, download managers, state machines, lifecycle management.

---

## Collection Optimization

### Recipe 12: Deduplication and Cleanup

**Problem**: Remove duplicates and clean up collections efficiently.

**Use Case**: Data cleaning, removing duplicate records from datasets.

```rust
#[derive(Debug, Clone, PartialEq)]
struct Record {
    id: u32,
    email: String,
    name: String,
}

fn deduplicate_records(records: &mut Vec<Record>) {
    // Sort first to make consecutive duplicates
    records.sort_by(|a, b| a.email.cmp(&b.email));

    // Remove consecutive duplicates
    records.dedup_by(|a, b| a.email == b.email);

    println!("Deduplicated to {} records", records.len());
}

fn remove_invalid_emails(records: &mut Vec<Record>) {
    let before = records.len();
    records.retain(|r| r.email.contains('@') && r.email.contains('.'));
    println!("Removed {} invalid emails", before - records.len());
}

fn normalize_names(records: &mut Vec<Record>) {
    for record in records.iter_mut() {
        record.name = record.name.trim().to_string();
        // Capitalize first letter
        if let Some(first) = record.name.chars().next() {
            record.name = first.to_uppercase().to_string() +
                         &record.name[1..].to_lowercase();
        }
    }
    println!("Normalized {} names", records.len());
}

fn main() {
    let mut records = vec![
        Record { id: 1, email: "alice@example.com".into(), name: "  alice  ".into() },
        Record { id: 2, email: "bob@example.com".into(), name: "BOB".into() },
        Record { id: 3, email: "alice@example.com".into(), name: "Alice".into() }, // Duplicate
        Record { id: 4, email: "invalid".into(), name: "Invalid".into() },
        Record { id: 5, email: "carol@example.com".into(), name: "carol".into() },
    ];

    println!("Starting with {} records\n", records.len());

    remove_invalid_emails(&mut records);
    deduplicate_records(&mut records);
    normalize_names(&mut records);

    println!("\n✓ Clean records:");
    for record in &records {
        println!("  {} <{}>", record.name, record.email);
    }
}
```

**When to use**: Data cleaning, ETL pipelines, duplicate removal, data normalization.

---

## Sorting and Organization

### Recipe 13: Multi-criteria Sorting

**Problem**: Sort data by multiple criteria with custom comparisons.

**Use Case**: E-commerce product listing with sort by price, rating, and name.

```rust
#[derive(Debug, Clone)]
struct Product {
    id: u32,
    name: String,
    price: f64,
    rating: f64,
    stock: u32,
}

struct ProductCatalog {
    products: Vec<Product>,
}

impl ProductCatalog {
    fn new(products: Vec<Product>) -> Self {
        ProductCatalog { products }
    }

    fn sort_by_price_asc(&mut self) {
        self.products.sort_by(|a, b| {
            a.price.partial_cmp(&b.price).unwrap()
        });
        println!("Sorted by price (ascending)");
    }

    fn sort_by_rating_desc(&mut self) {
        self.products.sort_by(|a, b| {
            b.rating.partial_cmp(&a.rating).unwrap()
        });
        println!("Sorted by rating (descending)");
    }

    fn sort_by_popularity(&mut self) {
        // Complex sort: rating then stock then price
        self.products.sort_by(|a, b| {
            b.rating.partial_cmp(&a.rating).unwrap()
                .then_with(|| b.stock.cmp(&a.stock))
                .then_with(|| a.price.partial_cmp(&b.price).unwrap())
        });
        println!("Sorted by popularity (rating > stock > price)");
    }

    fn sort_by_name(&mut self) {
        self.products.sort_by(|a, b| a.name.cmp(&b.name));
        println!("Sorted alphabetically");
    }

    fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.products.shuffle(&mut rng);
        println!("Products shuffled");
    }

    fn reverse_order(&mut self) {
        self.products.reverse();
        println!("Order reversed");
    }

    fn display(&self, limit: usize) {
        for product in self.products.iter().take(limit) {
            println!("  ${:.2} - {} (⭐ {:.1}) - {} in stock",
                product.price, product.name, product.rating, product.stock);
        }
    }
}

fn main() {
    let mut catalog = ProductCatalog::new(vec![
        Product { id: 1, name: "Laptop Pro".into(), price: 1299.99, rating: 4.8, stock: 15 },
        Product { id: 2, name: "Mouse Wireless".into(), price: 29.99, rating: 4.5, stock: 50 },
        Product { id: 3, name: "Keyboard Mechanical".into(), price: 89.99, rating: 4.9, stock: 20 },
        Product { id: 4, name: "Monitor 27\"".into(), price: 399.99, rating: 4.7, stock: 10 },
    ]);

    println!("By price:");
    catalog.sort_by_price_asc();
    catalog.display(4);

    println!("\nBy rating:");
    catalog.sort_by_rating_desc();
    catalog.display(4);

    println!("\nBy popularity:");
    catalog.sort_by_popularity();
    catalog.display(4);
}
```

**When to use**: E-commerce sorting, search results ranking, data organization, leaderboards.

---

## Batch Operations

### Recipe 14: Bulk Updates and Batch Processing

**Problem**: Apply bulk updates efficiently to large datasets.

**Use Case**: Database bulk updates, batch price changes, mass email updates.

```rust
#[derive(Debug)]
struct UserAccount {
    id: u32,
    email: String,
    subscription_tier: String,
    credits: u32,
    active: bool,
}

struct UserDatabase {
    users: Vec<UserAccount>,
}

impl UserDatabase {
    fn new(users: Vec<UserAccount>) -> Self {
        UserDatabase { users }
    }

    fn bulk_add_credits(&mut self, tier: &str, amount: u32) -> usize {
        let mut updated = 0;
        for user in self.users.iter_mut() {
            if user.subscription_tier == tier && user.active {
                user.credits += amount;
                updated += 1;
            }
        }
        println!("Added {} credits to {} {} users", amount, updated, tier);
        updated
    }

    fn deactivate_zero_credit_users(&mut self) -> usize {
        let mut deactivated = 0;
        for user in self.users.iter_mut() {
            if user.credits == 0 && user.active {
                user.active = false;
                deactivated += 1;
            }
        }
        println!("Deactivated {} users with zero credits", deactivated);
        deactivated
    }

    fn upgrade_tier(&mut self, user_ids: &[u32], new_tier: String) -> usize {
        let mut upgraded = 0;
        for user in self.users.iter_mut() {
            if user_ids.contains(&user.id) {
                user.subscription_tier = new_tier.clone();
                upgraded += 1;
            }
        }
        println!("Upgraded {} users to {}", upgraded, new_tier);
        upgraded
    }

    fn bulk_email_update(&mut self, updates: &[(u32, String)]) -> usize {
        let mut updated = 0;
        for (user_id, new_email) in updates {
            if let Some(user) = self.users.iter_mut().find(|u| u.id == *user_id) {
                user.email = new_email.clone();
                updated += 1;
            }
        }
        println!("Updated {} email addresses", updated);
        updated
    }

    fn purge_inactive(&mut self) -> usize {
        let before = self.users.len();
        self.users.retain(|user| user.active);
        let removed = before - self.users.len();
        println!("Purged {} inactive accounts", removed);
        removed
    }

    fn reset_monthly_credits(&mut self, base_amount: u32) {
        for user in self.users.iter_mut() {
            if user.active {
                user.credits = match user.subscription_tier.as_str() {
                    "free" => base_amount,
                    "pro" => base_amount * 5,
                    "enterprise" => base_amount * 20,
                    _ => base_amount,
                };
            }
        }
        println!("Reset monthly credits for all active users");
    }

    fn get_stats(&self) -> (usize, usize, u32) {
        let active = self.users.iter().filter(|u| u.active).count();
        let total = self.users.len();
        let total_credits = self.users.iter().map(|u| u.credits).sum();
        (active, total, total_credits)
    }
}

fn main() {
    let mut db = UserDatabase::new(vec![
        UserAccount {
            id: 1,
            email: "alice@example.com".into(),
            subscription_tier: "pro".into(),
            credits: 100,
            active: true,
        },
        UserAccount {
            id: 2,
            email: "bob@example.com".into(),
            subscription_tier: "free".into(),
            credits: 0,
            active: true,
        },
        UserAccount {
            id: 3,
            email: "carol@example.com".into(),
            subscription_tier: "pro".into(),
            credits: 50,
            active: true,
        },
    ]);

    db.bulk_add_credits("pro", 50);
    db.deactivate_zero_credit_users();
    db.upgrade_tier(&[2], "pro".to_string());

    let (active, total, credits) = db.get_stats();
    println!("\n📊 Stats: {} active / {} total, {} total credits",
        active, total, credits);
}
```

**When to use**: Bulk database updates, batch processing, mass operations, administrative tasks.

---

## Design Patterns

### Recipe 15: Builder Pattern with Progressive Construction

**Problem**: Build complex objects step-by-step with validation.

**Use Case**: Form builder, configuration builder, query builder.

**Design Pattern**: Builder Pattern

```rust
#[derive(Debug, Clone)]
struct DatabaseConfig {
    host: String,
    port: u16,
    database: String,
    username: Option<String>,
    password: Option<String>,
    pool_size: u32,
    timeout_secs: u32,
    ssl_enabled: bool,
}

struct DatabaseConfigBuilder {
    host: Option<String>,
    port: u16,
    database: Option<String>,
    username: Option<String>,
    password: Option<String>,
    pool_size: u32,
    timeout_secs: u32,
    ssl_enabled: bool,
}

impl DatabaseConfigBuilder {
    fn new() -> Self {
        DatabaseConfigBuilder {
            host: None,
            port: 5432,
            database: None,
            username: None,
            password: None,
            pool_size: 10,
            timeout_secs: 30,
            ssl_enabled: false,
        }
    }

    fn host(&mut self, host: String) -> &mut Self {
        self.host = Some(host);
        self
    }

    fn port(&mut self, port: u16) -> &mut Self {
        self.port = port;
        self
    }

    fn database(&mut self, database: String) -> &mut Self {
        self.database = Some(database);
        self
    }

    fn credentials(&mut self, username: String, password: String) -> &mut Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    fn pool_size(&mut self, size: u32) -> &mut Self {
        self.pool_size = size;
        self
    }

    fn timeout(&mut self, secs: u32) -> &mut Self {
        self.timeout_secs = secs;
        self
    }

    fn enable_ssl(&mut self) -> &mut Self {
        self.ssl_enabled = true;
        self
    }

    fn build(&self) -> Result<DatabaseConfig, String> {
        Ok(DatabaseConfig {
            host: self.host.clone().ok_or("Host is required")?,
            port: self.port,
            database: self.database.clone().ok_or("Database name is required")?,
            username: self.username.clone(),
            password: self.password.clone(),
            pool_size: self.pool_size,
            timeout_secs: self.timeout_secs,
            ssl_enabled: self.ssl_enabled,
        })
    }
}

fn main() {
    let mut builder = DatabaseConfigBuilder::new();

    let config = builder
        .host("localhost".to_string())
        .port(5432)
        .database("myapp".to_string())
        .credentials("admin".to_string(), "secret".to_string())
        .pool_size(20)
        .enable_ssl()
        .build();

    match config {
        Ok(cfg) => println!("✓ Database config created: {:?}", cfg),
        Err(e) => println!("✗ Error: {}", e),
    }
}
```

**When to use**: Configuration builders, complex object construction, API clients, form builders.

---

### Recipe 16: Observer Pattern with Event Broadcasting

**Problem**: Notify multiple subscribers when data changes.

**Use Case**: Stock price updates, notification system, event broadcasting.

**Design Pattern**: Observer Pattern

```rust
trait Subscriber {
    fn notify(&mut self, event: &str, data: &str);
}

struct EmailSubscriber {
    email: String,
}

impl Subscriber for EmailSubscriber {
    fn notify(&mut self, event: &str, data: &str) {
        println!("📧 Email to {}: {} - {}", self.email, event, data);
    }
}

struct SMSSubscriber {
    phone: String,
}

impl Subscriber for SMSSubscriber {
    fn notify(&mut self, event: &str, data: &str) {
        println!("📱 SMS to {}: {} - {}", self.phone, event, data);
    }
}

struct EventBroadcaster {
    subscribers: Vec<Box<dyn Subscriber>>,
}

impl EventBroadcaster {
    fn new() -> Self {
        EventBroadcaster { subscribers: Vec::new() }
    }

    fn subscribe(&mut self, subscriber: Box<dyn Subscriber>) {
        self.subscribers.push(subscriber);
        println!("New subscriber added (total: {})", self.subscribers.len());
    }

    fn unsubscribe(&mut self, index: usize) {
        if index < self.subscribers.len() {
            self.subscribers.remove(index);
            println!("Subscriber removed");
        }
    }

    fn broadcast(&mut self, event: &str, data: &str) {
        println!("\n📢 Broadcasting: {}", event);
        for subscriber in &mut self.subscribers {
            subscriber.notify(event, data);
        }
    }

    fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
}

fn main() {
    let mut broadcaster = EventBroadcaster::new();

    broadcaster.subscribe(Box::new(EmailSubscriber {
        email: "alice@example.com".to_string(),
    }));

    broadcaster.subscribe(Box::new(SMSSubscriber {
        phone: "+1234567890".to_string(),
    }));

    broadcaster.broadcast("PRICE_ALERT", "Stock XYZ reached $100");
    broadcaster.broadcast("SYSTEM_UPDATE", "Maintenance scheduled");

    println!("\nTotal subscribers: {}", broadcaster.subscriber_count());
}
```

**When to use**: Event systems, notification services, pub/sub patterns, data binding.

---

## Quick Reference

### Method Selection Guide

**Adding Elements:**
- `push()` - Add to end (O(1))
- `insert(index, value)` - Insert at position (O(n))
- `extend()` - Add multiple items
- `append(&mut vec)` - Move all from another vec

**Removing Elements:**
- `pop()` - Remove from end (O(1))
- `remove(index)` - Remove at position (O(n))
- `swap_remove(index)` - Fast remove, doesn't preserve order (O(1))
- `retain(|x| ...)` - Keep only matching
- `clear()` - Remove all

**Modifying Elements:**
- `iter_mut()` - Mutate each element
- `swap(i, j)` - Swap two elements
- Update via indexing: `vec[i] = value`

**Reordering:**
- `sort()` / `sort_unstable()` - Sort collection
- `reverse()` - Reverse order
- `rotate_left(k)` / `rotate_right(k)` - Rotate elements
- `shuffle()` - Random order (requires rand crate)

**Resizing:**
- `truncate(len)` - Keep first n elements
- `resize(len, value)` - Grow or shrink
- `reserve(n)` - Reserve capacity
- `shrink_to_fit()` - Release unused memory

**Optimization:**
- Use `swap_remove()` instead of `remove()` when order doesn't matter
- Use `retain()` for filtering in place
- Reserve capacity before bulk insertions
- Use `truncate()` instead of repeated `pop()`

### Common Patterns

**Stack (LIFO):**
```rust
let mut stack = Vec::new();
stack.push(item);      // Add
let item = stack.pop(); // Remove
```

**Queue (FIFO):**
```rust
let mut queue = VecDeque::new();
queue.push_back(item);      // Enqueue
let item = queue.pop_front(); // Dequeue
```

**In-place filtering:**
```rust
vec.retain(|x| condition(x));
```

**Bulk updates:**
```rust
for item in vec.iter_mut() {
    *item = transform(*item);
}
```

**Conditional removal:**
```rust
vec.retain(|item| !should_remove(item));
```

### Performance Tips

1. **Reserve capacity**: `vec.reserve(1000)` before bulk operations
2. **Use swap_remove**: When order doesn't matter (O(1) vs O(n))
3. **Batch operations**: Better than individual operations
4. **In-place modification**: Use `iter_mut()` instead of creating new vec
5. **Avoid repeated allocations**: Reuse vectors with `clear()`

### Common Mistakes

❌ **Modifying while iterating by index:**
```rust
for i in 0..vec.len() {
    vec.remove(i); // Indices shift!
}
```

✅ **Use retain instead:**
```rust
vec.retain(|x| should_keep(x));
```

---

❌ **Removing in forward loop:**
```rust
let mut i = 0;
while i < vec.len() {
    if condition {
        vec.remove(i);
        // Bug: don't increment i
    } else {
        i += 1;
    }
}
```

✅ **Use retain or reverse iteration:**
```rust
vec.retain(|x| !condition(x));
```

---

## Summary

Mutating operations in Rust provide:

* ✅ **Efficiency** - In-place modifications avoid allocations
* ✅ **Performance** - O(1) operations like push, pop, swap_remove
* ✅ **Memory control** - Explicit capacity management
* ✅ **Flexibility** - Rich API for common operations
* ✅ **Safety** - Borrow checker prevents common bugs

**Key Takeaways:**
- Choose the right operation for your use case
- Understand time complexity (O(1) vs O(n))
- Use `swap_remove` when order doesn't matter
- Reserve capacity for bulk operations
- Prefer `retain` over manual removal loops
- Consider design patterns for complex scenarios

Master these patterns to build efficient, memory-safe data structures and systems in Rust!
