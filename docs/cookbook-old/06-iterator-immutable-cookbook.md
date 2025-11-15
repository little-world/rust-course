# Cookbook: Immutable Iterator Methods in Rust

> **Real-world recipes for processing data with iterators**
>
> All examples use immutable iterator methods that don't modify original data, making them perfect for functional-style data processing pipelines.

## Table of Contents

1. [Data Transformation Recipes](#data-transformation-recipes)
2. [Data Filtering Recipes](#data-filtering-recipes)
3. [Data Aggregation Recipes](#data-aggregation-recipes)
4. [Search and Validation Recipes](#search-and-validation-recipes)
5. [Data Combining Recipes](#data-combining-recipes)
6. [Pagination and Windowing Recipes](#pagination-and-windowing-recipes)
7. [Text Processing Recipes](#text-processing-recipes)
8. [Log Analysis Recipes](#log-analysis-recipes)
9. [Data Parsing Recipes](#data-parsing-recipes)
10. [Business Logic Recipes](#business-logic-recipes)
11. [Performance Optimization Recipes](#performance-optimization-recipes)
12. [Error Handling Recipes](#error-handling-recipes)
13. [Quick Reference](#quick-reference)

---

## Data Transformation Recipes

### Recipe 1: Convert API Response to Display Format

**Problem**: You have user data from an API and need to format it for display in a table.

**Use Case**: Dashboard showing user information with formatted names and status.

```rust
#[derive(Debug)]
struct User {
    id: u32,
    first_name: String,
    last_name: String,
    active: bool,
}

#[derive(Debug)]
struct DisplayUser {
    full_name: String,
    status: String,
}

fn main() {
    let users = vec![
        User { id: 1, first_name: "Alice".into(), last_name: "Smith".into(), active: true },
        User { id: 2, first_name: "Bob".into(), last_name: "Jones".into(), active: false },
        User { id: 3, first_name: "Carol".into(), last_name: "White".into(), active: true },
    ];

    let display_users: Vec<DisplayUser> = users.iter()
        .map(|user| DisplayUser {
            full_name: format!("{} {}", user.first_name, user.last_name),
            status: if user.active { "Active" } else { "Inactive" }.to_string(),
        })
        .collect();

    for user in &display_users {
        println!("{}: {}", user.full_name, user.status);
    }
}
```

**When to use**: API response transformation, DTO conversion, data formatting for UI.

---

### Recipe 2: Calculate Price After Discount

**Problem**: Apply different discount tiers to product prices for a sales report.

**Use Case**: E-commerce platform showing discounted prices during a sale.

```rust
#[derive(Debug)]
struct Product {
    name: String,
    price: f64,
    category: String,
}

fn main() {
    let products = vec![
        Product { name: "Laptop".into(), price: 999.99, category: "Electronics".into() },
        Product { name: "Book".into(), price: 29.99, category: "Books".into() },
        Product { name: "Phone".into(), price: 699.99, category: "Electronics".into() },
    ];

    // Apply category-based discounts
    let discounted: Vec<(String, f64, f64)> = products.iter()
        .map(|p| {
            let discount = match p.category.as_str() {
                "Electronics" => 0.15, // 15% off
                "Books" => 0.10,       // 10% off
                _ => 0.0,
            };
            let final_price = p.price * (1.0 - discount);
            (p.name.clone(), p.price, final_price)
        })
        .collect();

    for (name, original, final_price) in discounted {
        println!("{}: ${:.2} -> ${:.2} (saved ${:.2})",
            name, original, final_price, original - final_price);
    }
}
```

**When to use**: Price calculations, tax computations, promotional pricing, billing systems.

---

### Recipe 3: Normalize Database Records

**Problem**: Convert timestamps and normalize user input from database records.

**Use Case**: Data cleanup pipeline before analysis or reporting.

```rust
use std::collections::HashMap;

fn main() {
    let raw_data = vec![
        ("user_123", "  Alice  ", "2024-01-15", 100),
        ("user_456", "Bob", "2024-01-20", 150),
        ("user_789", "CAROL  ", "2024-01-25", 200),
    ];

    let normalized: Vec<HashMap<String, String>> = raw_data.iter()
        .map(|(id, name, date, amount)| {
            let mut record = HashMap::new();
            record.insert("id".to_string(), id.to_string());
            record.insert("name".to_string(), name.trim().to_lowercase());
            record.insert("date".to_string(), date.to_string());
            record.insert("amount".to_string(), amount.to_string());
            record
        })
        .collect();

    for record in normalized {
        println!("{:?}", record);
    }
}
```

**When to use**: ETL pipelines, data import, CSV processing, database migration.

---

## Data Filtering Recipes

### Recipe 4: Filter Active Subscriptions

**Problem**: Get all active subscriptions that haven't expired.

**Use Case**: Monthly billing system needs to process only active subscriptions.

```rust
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug)]
struct Subscription {
    user_id: u32,
    plan: String,
    active: bool,
    expires_at: u64, // Unix timestamp
}

fn main() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let subscriptions = vec![
        Subscription { user_id: 1, plan: "Pro".into(), active: true, expires_at: now + 86400 },
        Subscription { user_id: 2, plan: "Basic".into(), active: true, expires_at: now - 86400 },
        Subscription { user_id: 3, plan: "Pro".into(), active: false, expires_at: now + 86400 },
        Subscription { user_id: 4, plan: "Enterprise".into(), active: true, expires_at: now + 172800 },
    ];

    let active_subs: Vec<_> = subscriptions.iter()
        .filter(|sub| sub.active && sub.expires_at > now)
        .collect();

    println!("Active subscriptions to bill: {}", active_subs.len());
    for sub in active_subs {
        println!("  User {}: {} plan", sub.user_id, sub.plan);
    }
}
```

**When to use**: Subscription management, billing systems, user access control, license validation.

---

### Recipe 5: Filter High-Value Transactions

**Problem**: Identify transactions above a threshold for fraud detection.

**Use Case**: Financial monitoring system flagging suspicious large transactions.

```rust
#[derive(Debug)]
struct Transaction {
    id: String,
    amount: f64,
    timestamp: String,
    user_id: u32,
}

fn main() {
    let transactions = vec![
        Transaction { id: "tx1".into(), amount: 50.0, timestamp: "10:00".into(), user_id: 1 },
        Transaction { id: "tx2".into(), amount: 5000.0, timestamp: "10:05".into(), user_id: 2 },
        Transaction { id: "tx3".into(), amount: 100.0, timestamp: "10:10".into(), user_id: 3 },
        Transaction { id: "tx4".into(), amount: 10000.0, timestamp: "10:15".into(), user_id: 1 },
    ];

    let threshold = 1000.0;
    let high_value: Vec<_> = transactions.iter()
        .filter(|tx| tx.amount > threshold)
        .collect();

    println!("⚠️  High-value transactions detected:");
    for tx in high_value {
        println!("  {} - User {}: ${:.2} at {}",
            tx.id, tx.user_id, tx.amount, tx.timestamp);
    }
}
```

**When to use**: Fraud detection, anomaly detection, alert systems, compliance monitoring.

---

### Recipe 6: Parse and Filter Valid Configuration

**Problem**: Load configuration entries, keeping only valid parseable values.

**Use Case**: Application startup reading config file with some invalid entries.

```rust
fn main() {
    let config_lines = vec![
        "max_connections=100",
        "timeout=30",
        "invalid_line",
        "port=8080",
        "host=",  // Invalid - empty value
        "debug=true",
    ];

    let valid_config: Vec<(String, String)> = config_lines.iter()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 && !parts[1].is_empty() {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    println!("Valid configuration:");
    for (key, value) in valid_config {
        println!("  {} = {}", key, value);
    }
}
```

**When to use**: Configuration parsing, environment variable loading, .env file processing.

---

## Data Aggregation Recipes

### Recipe 7: Calculate Order Statistics

**Problem**: Compute total revenue, average order value, and order count.

**Use Case**: E-commerce daily sales report.

```rust
#[derive(Debug)]
struct Order {
    id: u32,
    customer: String,
    total: f64,
}

fn main() {
    let orders = vec![
        Order { id: 1, customer: "Alice".into(), total: 150.50 },
        Order { id: 2, customer: "Bob".into(), total: 89.99 },
        Order { id: 3, customer: "Carol".into(), total: 234.75 },
        Order { id: 4, customer: "Alice".into(), total: 67.25 },
    ];

    let total_revenue: f64 = orders.iter().map(|o| o.total).sum();
    let order_count = orders.len();
    let avg_order_value = total_revenue / order_count as f64;

    let min_order = orders.iter()
        .map(|o| o.total)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let max_order = orders.iter()
        .map(|o| o.total)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    println!("📊 Sales Report");
    println!("  Orders: {}", order_count);
    println!("  Total Revenue: ${:.2}", total_revenue);
    println!("  Average Order: ${:.2}", avg_order_value);
    println!("  Range: ${:.2} - ${:.2}", min_order, max_order);
}
```

**When to use**: Sales reports, analytics dashboards, financial summaries, KPI calculations.

---

### Recipe 8: Group and Count Items

**Problem**: Count occurrences of each item type in inventory.

**Use Case**: Warehouse inventory summary by category.

```rust
use std::collections::HashMap;

fn main() {
    let items = vec![
        "laptop", "phone", "laptop", "tablet",
        "phone", "laptop", "phone", "desktop",
    ];

    let counts = items.iter()
        .fold(HashMap::new(), |mut acc, &item| {
            *acc.entry(item).or_insert(0) += 1;
            acc
        });

    println!("Inventory Summary:");
    for (item, count) in counts.iter() {
        println!("  {}: {}", item, count);
    }

    // Find most common item
    let most_common = counts.iter()
        .max_by_key(|(_, count)| *count)
        .map(|(item, count)| (item, count));

    if let Some((item, count)) = most_common {
        println!("\nMost common: {} ({} units)", item, count);
    }
}
```

**When to use**: Inventory management, log analysis, word frequency, histogram generation.

---

### Recipe 9: Calculate Running Totals

**Problem**: Generate cumulative sales figures for trend analysis.

**Use Case**: Financial dashboard showing running total of monthly sales.

```rust
fn main() {
    let monthly_sales = vec![
        ("Jan", 15000.0),
        ("Feb", 18000.0),
        ("Mar", 22000.0),
        ("Apr", 19500.0),
        ("May", 25000.0),
    ];

    let running_totals: Vec<_> = monthly_sales.iter()
        .scan(0.0, |total, (month, sales)| {
            *total += sales;
            Some((*month, *sales, *total))
        })
        .collect();

    println!("Monthly Sales with Running Total:");
    println!("{:<6} {:>10} {:>15}", "Month", "Sales", "Cumulative");
    println!("{:-<35}", "");

    for (month, sales, cumulative) in running_totals {
        println!("{:<6} ${:>9.2} ${:>14.2}", month, sales, cumulative);
    }
}
```

**When to use**: Financial reports, cumulative statistics, progress tracking, trend analysis.

---

## Search and Validation Recipes

### Recipe 10: Find User by Email

**Problem**: Locate a user account by email address.

**Use Case**: User login system or password reset functionality.

```rust
#[derive(Debug, Clone)]
struct User {
    id: u32,
    email: String,
    name: String,
}

fn main() {
    let users = vec![
        User { id: 1, email: "alice@example.com".into(), name: "Alice".into() },
        User { id: 2, email: "bob@example.com".into(), name: "Bob".into() },
        User { id: 3, email: "carol@example.com".into(), name: "Carol".into() },
    ];

    let search_email = "bob@example.com";

    let found_user = users.iter()
        .find(|user| user.email == search_email);

    match found_user {
        Some(user) => println!("Found: {} (ID: {})", user.name, user.id),
        None => println!("No user found with email: {}", search_email),
    }

    // Find position for logging
    if let Some(pos) = users.iter().position(|u| u.email == search_email) {
        println!("User is at index: {}", pos);
    }
}
```

**When to use**: User lookup, authentication, search functionality, data retrieval.

---

### Recipe 11: Validate All Records Meet Criteria

**Problem**: Check if all uploaded files meet size and format requirements.

**Use Case**: File upload validation before processing batch.

```rust
#[derive(Debug)]
struct UploadedFile {
    name: String,
    size_bytes: u64,
    format: String,
}

fn main() {
    let files = vec![
        UploadedFile { name: "doc1.pdf".into(), size_bytes: 1024 * 500, format: "pdf".into() },
        UploadedFile { name: "doc2.pdf".into(), size_bytes: 1024 * 800, format: "pdf".into() },
        UploadedFile { name: "doc3.pdf".into(), size_bytes: 1024 * 300, format: "pdf".into() },
    ];

    let max_size = 1024 * 1024; // 1 MB
    let allowed_formats = vec!["pdf", "docx"];

    let all_valid = files.iter().all(|file| {
        file.size_bytes <= max_size &&
        allowed_formats.contains(&file.format.as_str())
    });

    if all_valid {
        println!("✅ All {} files passed validation", files.len());
        println!("Proceeding with batch processing...");
    } else {
        println!("❌ Validation failed");

        // Find invalid files
        let invalid: Vec<_> = files.iter()
            .filter(|f| f.size_bytes > max_size || !allowed_formats.contains(&f.format.as_str()))
            .collect();

        for file in invalid {
            println!("  Invalid: {} ({} bytes, {})", file.name, file.size_bytes, file.format);
        }
    }
}
```

**When to use**: Input validation, batch processing, quality assurance, data integrity checks.

---

### Recipe 12: Check for Any Errors in Batch

**Problem**: Determine if any operation in a batch failed.

**Use Case**: ETL pipeline checking if any record failed to process.

```rust
#[derive(Debug)]
struct ProcessingResult {
    record_id: u32,
    success: bool,
    error_message: Option<String>,
}

fn main() {
    let results = vec![
        ProcessingResult { record_id: 1, success: true, error_message: None },
        ProcessingResult { record_id: 2, success: true, error_message: None },
        ProcessingResult { record_id: 3, success: false, error_message: Some("Invalid format".into()) },
        ProcessingResult { record_id: 4, success: true, error_message: None },
    ];

    let has_errors = results.iter().any(|r| !r.success);

    if has_errors {
        println!("⚠️  Batch processing completed with errors");

        let errors: Vec<_> = results.iter()
            .filter(|r| !r.success)
            .collect();

        println!("Failed records:");
        for result in errors {
            println!("  Record {}: {}",
                result.record_id,
                result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }
    } else {
        println!("✅ All {} records processed successfully", results.len());
    }
}
```

**When to use**: Batch processing, data validation, quality control, error reporting.

---

## Data Combining Recipes

### Recipe 13: Merge User Data with Orders

**Problem**: Combine user information with their order data for a report.

**Use Case**: Customer relationship management system generating user activity reports.

```rust
#[derive(Debug)]
struct User {
    id: u32,
    name: String,
}

#[derive(Debug)]
struct Order {
    order_id: u32,
    amount: f64,
}

fn main() {
    let users = vec![
        User { id: 1, name: "Alice".into() },
        User { id: 2, name: "Bob".into() },
        User { id: 3, name: "Carol".into() },
    ];

    let orders = vec![
        Order { order_id: 101, amount: 150.0 },
        Order { order_id: 102, amount: 200.0 },
        Order { order_id: 103, amount: 89.99 },
    ];

    let user_orders: Vec<_> = users.iter()
        .zip(orders.iter())
        .map(|(user, order)| {
            format!("{} placed order #{} for ${:.2}",
                user.name, order.order_id, order.amount)
        })
        .collect();

    for report_line in user_orders {
        println!("{}", report_line);
    }
}
```

**When to use**: Data joining, report generation, combining multiple data sources.

---

### Recipe 14: Combine Multiple API Responses

**Problem**: Merge data from multiple service endpoints into a single view.

**Use Case**: Dashboard aggregating data from different microservices.

```rust
fn main() {
    // Simulated API responses
    let user_data = vec![("user1", "Alice"), ("user2", "Bob")];
    let activity_counts = vec![45, 78];
    let last_login = vec!["2024-01-20", "2024-01-19"];

    let dashboard_data: Vec<_> = user_data.iter()
        .zip(activity_counts.iter())
        .zip(last_login.iter())
        .map(|(((id, name), activity), login)| {
            format!("{} ({}): {} activities, last seen {}",
                name, id, activity, login)
        })
        .collect();

    println!("User Dashboard:");
    for entry in dashboard_data {
        println!("  {}", entry);
    }
}
```

**When to use**: Microservices aggregation, API composition, dashboard data assembly.

---

### Recipe 15: Concatenate Multiple Data Sources

**Problem**: Combine logs from multiple servers for analysis.

**Use Case**: Centralized logging system aggregating from multiple sources.

```rust
fn main() {
    let server1_logs = vec!["[Server1] Request received", "[Server1] Processing"];
    let server2_logs = vec!["[Server2] Connection established", "[Server2] Data sent"];
    let server3_logs = vec!["[Server3] Cache hit", "[Server3] Response ready"];

    let all_logs: Vec<_> = server1_logs.iter()
        .chain(server2_logs.iter())
        .chain(server3_logs.iter())
        .collect();

    println!("Aggregated Logs ({} entries):", all_logs.len());
    for (i, log) in all_logs.iter().enumerate() {
        println!("{:3}. {}", i + 1, log);
    }

    // Filter to specific server
    let server1_only: Vec<_> = all_logs.iter()
        .filter(|log| log.contains("Server1"))
        .collect();

    println!("\nServer1 logs only: {} entries", server1_only.len());
}
```

**When to use**: Log aggregation, data consolidation, multi-source monitoring.

---

## Pagination and Windowing Recipes

### Recipe 16: Implement API Pagination

**Problem**: Return paginated results from a large dataset.

**Use Case**: REST API endpoint with page size and page number parameters.

```rust
#[derive(Debug)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

fn get_page(products: &[Product], page: usize, page_size: usize) -> Vec<&Product> {
    products.iter()
        .skip(page * page_size)
        .take(page_size)
        .collect()
}

fn main() {
    let products: Vec<Product> = (1..=25)
        .map(|i| Product {
            id: i,
            name: format!("Product {}", i),
            price: 10.0 + (i as f64),
        })
        .collect();

    let page_size = 10;
    let page = 1; // Zero-indexed: 0 = first page, 1 = second page

    let page_results = get_page(&products, page, page_size);

    println!("Page {} (items {} - {}):",
        page + 1,
        page * page_size + 1,
        (page * page_size + page_results.len())
    );

    for product in page_results {
        println!("  #{}: {} - ${:.2}", product.id, product.name, product.price);
    }

    let total_pages = (products.len() + page_size - 1) / page_size;
    println!("\nTotal pages: {}", total_pages);
}
```

**When to use**: REST APIs, database query results, large list displays, infinite scroll.

---

### Recipe 17: Process Data in Chunks

**Problem**: Process large dataset in batches to avoid memory issues.

**Use Case**: Batch email sending or data export in manageable chunks.

```rust
fn main() {
    let email_recipients: Vec<String> = (1..=100)
        .map(|i| format!("user{}@example.com", i))
        .collect();

    let batch_size = 25;

    // Process in chunks
    let batches: Vec<Vec<&String>> = email_recipients
        .chunks(batch_size)
        .map(|chunk| chunk.iter().collect())
        .collect();

    println!("Processing {} recipients in {} batches of {}",
        email_recipients.len(), batches.len(), batch_size);

    for (batch_num, batch) in batches.iter().enumerate() {
        println!("\n📧 Batch {}: Sending to {} recipients", batch_num + 1, batch.len());

        // Simulate sending
        for recipient in batch {
            println!("  Sent to {}", recipient);
        }

        println!("  ✅ Batch {} complete", batch_num + 1);
    }
}
```

**When to use**: Bulk operations, email campaigns, data migration, rate-limited API calls.

---

### Recipe 18: Moving Average Calculation

**Problem**: Calculate moving average for time series data.

**Use Case**: Stock price analysis or sensor data smoothing.

```rust
fn main() {
    let stock_prices = vec![
        100.0, 102.0, 101.0, 105.0, 103.0,
        107.0, 110.0, 108.0, 112.0, 115.0,
    ];

    let window_size = 3;

    let moving_averages: Vec<f64> = stock_prices
        .windows(window_size)
        .map(|window| window.iter().sum::<f64>() / window_size as f64)
        .collect();

    println!("Stock Price Moving Average (window = {}):", window_size);
    println!("{:<8} {:<10} {:<10}", "Day", "Price", "MA");
    println!("{:-<30}", "");

    for (i, (&price, &ma)) in stock_prices.iter()
        .skip(window_size - 1)
        .zip(moving_averages.iter())
        .enumerate()
    {
        println!("{:<8} ${:<9.2} ${:<9.2}", i + window_size, price, ma);
    }
}
```

**When to use**: Time series analysis, data smoothing, trend detection, technical indicators.

---

## Text Processing Recipes

### Recipe 19: Extract Email Addresses from Text

**Problem**: Parse email addresses from unstructured text.

**Use Case**: Data extraction from customer feedback or support tickets.

```rust
fn main() {
    let text = "Contact us at support@example.com or sales@example.com for assistance. \
                Our CEO john.doe@company.com is also available.";

    let words: Vec<&str> = text.split_whitespace().collect();

    let emails: Vec<&str> = words.iter()
        .filter(|word| word.contains('@') && word.contains('.'))
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric() && c != '@' && c != '.'))
        .collect();

    println!("Extracted email addresses:");
    for email in emails {
        println!("  {}", email);
    }
}
```

**When to use**: Data extraction, text mining, contact parsing, email validation.

---

### Recipe 20: Word Frequency Analysis

**Problem**: Count word occurrences in a document.

**Use Case**: SEO analysis, content analysis, keyword extraction.

```rust
use std::collections::HashMap;

fn main() {
    let document = "the quick brown fox jumps over the lazy dog the fox was quick";

    let word_counts = document
        .split_whitespace()
        .map(|word| word.to_lowercase())
        .fold(HashMap::new(), |mut acc, word| {
            *acc.entry(word).or_insert(0) += 1;
            acc
        });

    let mut sorted_words: Vec<_> = word_counts.iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(a.1)); // Sort by frequency descending

    println!("Word Frequency Analysis:");
    for (word, count) in sorted_words.iter().take(5) {
        println!("  {}: {}", word, count);
    }
}
```

**When to use**: Text analysis, SEO, content optimization, natural language processing.

---

### Recipe 21: Split and Clean CSV Data

**Problem**: Parse and clean CSV data with inconsistent formatting.

**Use Case**: Data import from user-uploaded CSV files.

```rust
fn main() {
    let csv_data = "Name,Age,City\n\
                    Alice  , 30,New York\n\
                    Bob,25  ,  Los Angeles  \n\
                      Carol,35,Chicago\n";

    let rows: Vec<Vec<String>> = csv_data
        .lines()
        .skip(1) // Skip header
        .map(|line| {
            line.split(',')
                .map(|field| field.trim().to_string())
                .collect()
        })
        .collect();

    println!("Cleaned CSV Data:");
    for (i, row) in rows.iter().enumerate() {
        println!("Record {}: {:?}", i + 1, row);
    }

    // Extract specific column (e.g., ages)
    let ages: Vec<i32> = rows.iter()
        .filter_map(|row| row.get(1)?.parse().ok())
        .collect();

    println!("\nAges: {:?}", ages);
    println!("Average age: {:.1}", ages.iter().sum::<i32>() as f64 / ages.len() as f64);
}
```

**When to use**: CSV import, data cleaning, ETL processes, file parsing.

---

## Log Analysis Recipes

### Recipe 22: Parse and Filter Application Logs

**Problem**: Extract error logs from application log files.

**Use Case**: Debugging production issues by analyzing error patterns.

```rust
#[derive(Debug)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    let parts: Vec<&str> = line.splitn(3, '|').collect();
    if parts.len() == 3 {
        Some(LogEntry {
            timestamp: parts[0].trim().to_string(),
            level: parts[1].trim().to_string(),
            message: parts[2].trim().to_string(),
        })
    } else {
        None
    }
}

fn main() {
    let log_lines = vec![
        "2024-01-20 10:00:00 | INFO | Application started",
        "2024-01-20 10:00:15 | DEBUG | Connecting to database",
        "2024-01-20 10:00:20 | ERROR | Connection timeout",
        "2024-01-20 10:00:25 | INFO | Retrying connection",
        "2024-01-20 10:00:30 | ERROR | Database unavailable",
        "2024-01-20 10:00:35 | WARN | Using cached data",
    ];

    let errors: Vec<LogEntry> = log_lines.iter()
        .filter_map(|line| parse_log_line(line))
        .filter(|entry| entry.level == "ERROR")
        .collect();

    println!("🔴 Error Log Entries ({}):", errors.len());
    for entry in errors {
        println!("  [{}] {}", entry.timestamp, entry.message);
    }
}
```

**When to use**: Log analysis, debugging, monitoring, incident response.

---

### Recipe 23: Calculate Request Rate per Endpoint

**Problem**: Analyze API request logs to find most-used endpoints.

**Use Case**: API performance analysis and capacity planning.

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct ApiRequest {
    endpoint: String,
    method: String,
    response_time_ms: u64,
}

fn main() {
    let requests = vec![
        ApiRequest { endpoint: "/api/users".into(), method: "GET".into(), response_time_ms: 50 },
        ApiRequest { endpoint: "/api/products".into(), method: "GET".into(), response_time_ms: 75 },
        ApiRequest { endpoint: "/api/users".into(), method: "POST".into(), response_time_ms: 120 },
        ApiRequest { endpoint: "/api/users".into(), method: "GET".into(), response_time_ms: 45 },
        ApiRequest { endpoint: "/api/products".into(), method: "GET".into(), response_time_ms: 80 },
        ApiRequest { endpoint: "/api/orders".into(), method: "POST".into(), response_time_ms: 200 },
    ];

    // Count requests per endpoint
    let endpoint_counts = requests.iter()
        .fold(HashMap::new(), |mut acc, req| {
            *acc.entry(&req.endpoint).or_insert(0) += 1;
            acc
        });

    // Calculate average response time per endpoint
    let avg_response_times: HashMap<&String, u64> = requests.iter()
        .fold(HashMap::new(), |mut acc, req| {
            let entry = acc.entry(&req.endpoint).or_insert((0, 0));
            entry.0 += req.response_time_ms;
            entry.1 += 1;
            acc
        })
        .iter()
        .map(|(endpoint, (total, count))| (*endpoint, total / count))
        .collect();

    println!("API Endpoint Analytics:");
    for (endpoint, count) in endpoint_counts.iter() {
        let avg_time = avg_response_times.get(endpoint).unwrap_or(&0);
        println!("  {}: {} requests, avg {}ms", endpoint, count, avg_time);
    }
}
```

**When to use**: API monitoring, performance analysis, capacity planning, usage analytics.

---

## Data Parsing Recipes

### Recipe 24: Parse JSON-like Strings

**Problem**: Extract key-value pairs from structured string data.

**Use Case**: Parsing query parameters or simple configuration strings.

```rust
use std::collections::HashMap;

fn main() {
    let query_string = "user=alice&age=30&city=NYC&premium=true";

    let params: HashMap<String, String> = query_string
        .split('&')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.split('=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    println!("Parsed Parameters:");
    for (key, value) in &params {
        println!("  {} = {}", key, value);
    }

    // Use parsed data
    if let Some(user) = params.get("user") {
        println!("\nWelcome, {}!", user);
    }

    if let Some(premium) = params.get("premium") {
        if premium == "true" {
            println!("Premium features enabled");
        }
    }
}
```

**When to use**: URL parsing, query string handling, simple data deserialization.

---

### Recipe 25: Convert Structured Data to Report

**Problem**: Transform database results into formatted report.

**Use Case**: Automated report generation from query results.

```rust
#[derive(Debug)]
struct SalesRecord {
    region: String,
    sales: f64,
    quarter: u8,
}

fn main() {
    let records = vec![
        SalesRecord { region: "North".into(), sales: 50000.0, quarter: 1 },
        SalesRecord { region: "South".into(), sales: 45000.0, quarter: 1 },
        SalesRecord { region: "North".into(), sales: 55000.0, quarter: 2 },
        SalesRecord { region: "South".into(), sales: 48000.0, quarter: 2 },
    ];

    // Group by quarter
    use std::collections::HashMap;

    let quarterly_totals = records.iter()
        .fold(HashMap::new(), |mut acc, record| {
            *acc.entry(record.quarter).or_insert(0.0) += record.sales;
            acc
        });

    println!("Quarterly Sales Report");
    println!("{:-<30}", "");

    for quarter in 1..=2 {
        if let Some(total) = quarterly_totals.get(&quarter) {
            let records_in_quarter: Vec<_> = records.iter()
                .filter(|r| r.quarter == quarter)
                .collect();

            println!("\nQ{}: ${:.2}", quarter, total);
            for record in records_in_quarter {
                println!("  {}: ${:.2}", record.region, record.sales);
            }
        }
    }
}
```

**When to use**: Report generation, data visualization prep, executive summaries.

---

## Business Logic Recipes

### Recipe 26: Calculate Shipping Costs

**Problem**: Determine shipping cost based on weight and destination.

**Use Case**: E-commerce checkout calculating delivery fees.

```rust
#[derive(Debug)]
struct Order {
    items: Vec<String>,
    total_weight_kg: f64,
    destination: String,
}

fn calculate_shipping(order: &Order) -> f64 {
    let base_rate = match order.destination.as_str() {
        "domestic" => 5.0,
        "international" => 15.0,
        _ => 10.0,
    };

    let weight_charge = order.total_weight_kg * 2.0;

    base_rate + weight_charge
}

fn main() {
    let orders = vec![
        Order {
            items: vec!["Book".into(), "Pen".into()],
            total_weight_kg: 0.5,
            destination: "domestic".into(),
        },
        Order {
            items: vec!["Laptop".into()],
            total_weight_kg: 2.0,
            destination: "international".into(),
        },
    ];

    let shipping_costs: Vec<f64> = orders.iter()
        .map(|order| calculate_shipping(order))
        .collect();

    for (order, cost) in orders.iter().zip(shipping_costs.iter()) {
        println!("{} to {} - ${:.2}",
            order.items.join(", "),
            order.destination,
            cost
        );
    }

    let total_shipping: f64 = shipping_costs.iter().sum();
    println!("\nTotal shipping revenue: ${:.2}", total_shipping);
}
```

**When to use**: E-commerce pricing, logistics, cost calculations, dynamic pricing.

---

### Recipe 27: Apply Business Rules to Data

**Problem**: Filter and categorize customers based on multiple criteria.

**Use Case**: Customer segmentation for targeted marketing campaigns.

```rust
#[derive(Debug)]
struct Customer {
    name: String,
    total_purchases: f64,
    account_age_days: u32,
    email_verified: bool,
}

#[derive(Debug)]
enum CustomerTier {
    Premium,
    Regular,
    New,
}

fn categorize_customer(customer: &Customer) -> CustomerTier {
    if customer.total_purchases > 1000.0 && customer.account_age_days > 365 {
        CustomerTier::Premium
    } else if customer.email_verified && customer.account_age_days > 30 {
        CustomerTier::Regular
    } else {
        CustomerTier::New
    }
}

fn main() {
    let customers = vec![
        Customer {
            name: "Alice".into(),
            total_purchases: 1500.0,
            account_age_days: 400,
            email_verified: true,
        },
        Customer {
            name: "Bob".into(),
            total_purchases: 200.0,
            account_age_days: 60,
            email_verified: true,
        },
        Customer {
            name: "Carol".into(),
            total_purchases: 50.0,
            account_age_days: 10,
            email_verified: false,
        },
    ];

    let categorized: Vec<_> = customers.iter()
        .map(|c| (c, categorize_customer(c)))
        .collect();

    println!("Customer Segmentation:");
    for (customer, tier) in categorized {
        println!("  {} - {:?} (${:.2} spent, {} days old)",
            customer.name, tier, customer.total_purchases, customer.account_age_days);
    }

    // Count by tier
    use std::collections::HashMap;
    let tier_counts = customers.iter()
        .map(|c| categorize_customer(c))
        .fold(HashMap::new(), |mut acc, tier| {
            *acc.entry(format!("{:?}", tier)).or_insert(0) += 1;
            acc
        });

    println!("\nTier Distribution:");
    for (tier, count) in tier_counts {
        println!("  {}: {}", tier, count);
    }
}
```

**When to use**: Customer segmentation, business rules engine, eligibility checks, tier systems.

---

## Performance Optimization Recipes

### Recipe 28: Lazy Evaluation for Large Datasets

**Problem**: Process large dataset without loading everything into memory.

**Use Case**: Processing millions of records from a data stream or file.

```rust
fn main() {
    // Simulate large dataset (only first 5 processed for demo)
    let data_stream = 1..=1_000_000;

    // Lazy pipeline - nothing computed until needed
    let results: Vec<i32> = data_stream
        .filter(|x| x % 2 == 0)       // Only even numbers
        .map(|x| x * x)                 // Square them
        .filter(|x| x % 3 == 0)        // Divisible by 3
        .take(5)                        // Only first 5 results
        .collect();

    println!("First 5 results from 1 million items: {:?}", results);
    println!("Computed lazily - only {} items were actually processed!",
        results.len());
}
```

**When to use**: Large file processing, streaming data, memory-constrained environments.

---

### Recipe 29: Short-Circuit Early Exit

**Problem**: Stop processing as soon as condition is met.

**Use Case**: Finding first match in large dataset to save CPU cycles.

```rust
fn main() {
    let large_dataset: Vec<i32> = (1..=1_000_000).collect();

    // Find first number divisible by 12345
    let start = std::time::Instant::now();

    let found = large_dataset.iter()
        .find(|&&x| x % 12345 == 0);

    let elapsed = start.elapsed();

    match found {
        Some(num) => println!("Found: {} in {:?}", num, elapsed),
        None => println!("Not found"),
    }

    // Compare with full iteration (don't actually do this)
    println!("Short-circuited after checking ~{} items instead of 1 million!",
        found.unwrap_or(&0) / 1);
}
```

**When to use**: Search operations, validation, early termination conditions.

---

### Recipe 30: Reuse Iterator Chains

**Problem**: Build reusable data processing pipeline.

**Use Case**: Common filtering/transformation applied across multiple datasets.

```rust
fn main() {
    // Reusable pipeline function
    fn clean_and_validate(data: &[i32]) -> Vec<i32> {
        data.iter()
            .filter(|&&x| x > 0)           // Remove non-positive
            .filter(|&&x| x < 1000)        // Remove too large
            .map(|&x| x * 2)                // Double the value
            .collect()
    }

    let dataset1 = vec![5, -3, 100, 500, 2000, 10];
    let dataset2 = vec![-1, 50, 999, 1, 5000];

    let cleaned1 = clean_and_validate(&dataset1);
    let cleaned2 = clean_and_validate(&dataset2);

    println!("Cleaned dataset 1: {:?}", cleaned1);
    println!("Cleaned dataset 2: {:?}", cleaned2);
}
```

**When to use**: Reusable transformations, data pipelines, consistent processing logic.

---

## Error Handling Recipes

### Recipe 31: Parse Multiple Values, Collect Errors

**Problem**: Parse data, identifying which items failed.

**Use Case**: Bulk data import with error reporting.

```rust
fn main() {
    let input_data = vec!["42", "abc", "100", "xyz", "200"];

    // Separate successes from failures
    let (successes, failures): (Vec<_>, Vec<_>) = input_data.iter()
        .map(|s| s.parse::<i32>())
        .partition(Result::is_ok);

    let numbers: Vec<i32> = successes.into_iter()
        .map(|r| r.unwrap())
        .collect();

    let errors: Vec<String> = failures.into_iter()
        .map(|r| r.unwrap_err().to_string())
        .collect();

    println!("✅ Successfully parsed: {:?}", numbers);
    println!("❌ Failed to parse: {} items", errors.len());

    if !errors.is_empty() {
        println!("\nError details:");
        for (i, error) in errors.iter().enumerate() {
            println!("  {}. {}", i + 1, error);
        }
    }
}
```

**When to use**: Bulk operations, data import, validation with error tracking.

---

### Recipe 32: Fail Fast on First Error

**Problem**: Stop processing on first parse error.

**Use Case**: Configuration validation where any error invalidates entire config.

```rust
fn main() {
    let config_values = vec!["100", "200", "300", "400"];

    let result: Result<Vec<i32>, _> = config_values.iter()
        .map(|s| s.parse::<i32>())
        .collect();

    match result {
        Ok(numbers) => {
            println!("✅ All values parsed successfully");
            println!("Configuration: {:?}", numbers);
        }
        Err(e) => {
            println!("❌ Configuration invalid: {}", e);
            println!("Aborting startup...");
        }
    }

    // Example with error
    let bad_config = vec!["100", "bad", "300"];
    let result: Result<Vec<i32>, _> = bad_config.iter()
        .map(|s| s.parse::<i32>())
        .collect();

    if let Err(e) = result {
        println!("\n❌ Failed at first error: {}", e);
    }
}
```

**When to use**: Critical validations, configuration loading, atomic operations.

---

### Recipe 33: Filter Out Errors, Keep Valid Data

**Problem**: Process what you can, skip invalid items.

**Use Case**: Best-effort data processing where partial success is acceptable.

```rust
fn main() {
    let mixed_data = vec!["10", "20", "invalid", "30", "bad", "40"];

    let valid_numbers: Vec<i32> = mixed_data.iter()
        .filter_map(|s| s.parse::<i32>().ok())
        .collect();

    println!("Input items: {}", mixed_data.len());
    println!("Valid numbers: {} {:?}", valid_numbers.len(), valid_numbers);
    println!("Skipped: {} invalid items", mixed_data.len() - valid_numbers.len());

    if valid_numbers.is_empty() {
        println!("⚠️  Warning: No valid data to process!");
    }
}
```

**When to use**: Lenient parsing, scraping data, user input processing.

---

## Quick Reference

### Iterator Method Selection Guide

**Need to transform data?**
- `map()` - Transform each element
- `flat_map()` - Transform and flatten nested results
- `scan()` - Transform with running state

**Need to filter data?**
- `filter()` - Keep elements matching predicate
- `filter_map()` - Filter and transform in one step
- `take()` / `skip()` - Limit or skip elements
- `take_while()` / `skip_while()` - Conditional limiting

**Need to aggregate data?**
- `fold()` / `reduce()` - Custom aggregation
- `sum()` / `product()` - Arithmetic aggregation
- `min()` / `max()` - Find extremes
- `count()` - Count elements

**Need to search?**
- `find()` - Find first matching element
- `position()` - Find index of match
- `any()` - Check if any match
- `all()` - Check if all match

**Need to combine data?**
- `zip()` - Pair up two iterators
- `chain()` - Concatenate iterators
- `enumerate()` - Add indices

**Common Patterns:**

```rust
// ETL Pipeline
data.iter()
    .filter(|x| is_valid(x))      // Extract valid
    .map(|x| transform(x))         // Transform
    .collect()                     // Load

// Aggregation with grouping
data.iter()
    .fold(HashMap::new(), |acc, x| {
        // Group logic
        acc
    })

// Validation
data.iter().all(|x| is_valid(x))

// Search with position
data.iter().position(|x| matches(x))

// Pagination
data.iter()
    .skip(page * size)
    .take(size)
    .collect()
```

### Performance Tips

1. **Lazy evaluation**: Iterators don't execute until consumed
2. **Short-circuit**: Use `any()`, `all()`, `find()` for early exit
3. **Avoid unnecessary collection**: Chain operations before `collect()`
4. **Use `filter_map` over `filter().map()`**: Single pass is more efficient
5. **Reuse iterator logic**: Extract to functions for repeated patterns

### Common Mistakes

❌ Forgetting to consume iterator:
```rust
vec.iter().map(|x| x * 2); // Does nothing!
```

✅ Proper consumption:
```rust
let result: Vec<_> = vec.iter().map(|x| x * 2).collect();
```

---

❌ Multiple allocations:
```rust
data.iter().map(|x| x * 2).collect::<Vec<_>>().into_iter().filter(|x| x > 10).collect()
```

✅ Single pass:
```rust
data.iter().map(|x| x * 2).filter(|x| *x > 10).collect()
```

---

## Summary

Iterator methods in Rust provide:

* ✅ **Zero-cost abstractions** - Compile to efficient machine code
* ✅ **Composable pipelines** - Chain operations naturally
* ✅ **Lazy evaluation** - Compute only what's needed
* ✅ **Type safety** - Compile-time guarantees
* ✅ **Functional style** - Immutable, declarative code

**Key takeaways:**
- Use iterators for data transformation pipelines
- Choose the right method for your use case
- Leverage lazy evaluation for performance
- Chain operations before collecting
- Handle errors appropriately for your use case

Master these patterns to write elegant, efficient Rust code for real-world data processing!
