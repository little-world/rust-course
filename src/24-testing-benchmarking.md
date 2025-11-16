# 24. Testing & Benchmarking

Testing is where Rust's philosophy of "if it compiles, it works" meets reality. The compiler catches many bugs, but it can't verify business logic, edge cases, or performance characteristics. A comprehensive testing strategy—from unit tests to property-based testing to benchmarks—ensures your code works correctly and performs well.

This chapter explores Rust's testing ecosystem, from the built-in test framework to advanced property-based testing and performance benchmarking. We'll see how Rust's testing tools leverage the type system and ownership model to make tests more reliable and easier to maintain.

## Unit Test Patterns

Unit tests are the foundation of software quality. They verify that individual components work correctly in isolation. Rust makes writing unit tests remarkably easy with built-in tooling and a simple, intuitive syntax.

### The Basics of Rust Testing

Rust's test framework is built into the language. Any function annotated with `#[test]` becomes a test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_subtraction() {
        let result = 10 - 3;
        assert_eq!(result, 7);
    }
}
```

Run tests with `cargo test`. The test runner automatically discovers and executes all test functions, reporting successes and failures.

The `#[cfg(test)]` attribute ensures test modules only compile during testing. This keeps test code out of release builds, reducing binary size.

### Assertion Macros

Rust provides several assertion macros for different scenarios:

```rust
#[test]
fn assertion_examples() {
    //===============
    // Basic equality
    //===============
    assert_eq!(5, 2 + 3);

    //===========
    // Inequality
    //===========
    assert_ne!(5, 6);

    //===================
    // Boolean assertions
    //===================
    assert!(true);
    assert!(5 > 3, "5 should be greater than 3");

    //======================
    // Custom error messages
    //======================
    let x = 10;
    assert_eq!(x, 10, "x should be 10, but was {}", x);
}
```

The `assert_eq!` and `assert_ne!` macros provide better error messages than `assert!` because they show both the expected and actual values when tests fail.

### Testing Error Cases

Good tests verify both success and failure paths. Rust makes this elegant:

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_success() {
        assert_eq!(divide(10, 2), Ok(5));
    }

    #[test]
    fn test_divide_by_zero() {
        assert!(divide(10, 0).is_err());
    }

    #[test]
    fn test_divide_error_message() {
        match divide(10, 0) {
            Err(msg) => assert_eq!(msg, "Division by zero"),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }
}
```

Testing error cases is crucial—many bugs lurk in error paths because they're harder to test manually.

### Testing Panics

Some functions should panic in certain conditions. Test this with `#[should_panic]`:

```rust
fn validate_age(age: u32) {
    if age > 150 {
        panic!("Age {} is unrealistic", age);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_invalid_age_panics() {
        validate_age(200);
    }

    #[test]
    #[should_panic(expected = "unrealistic")]
    fn test_panic_message() {
        validate_age(200);
    }
}
```

The `expected` parameter verifies that the panic message contains specific text, ensuring you're panicking for the right reason.

### Organizing Tests

As codebases grow, test organization becomes important. Here are common patterns:

```rust
//============
// src/math.rs
//============
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(test)]
mod tests {
    use super::*;

    mod addition_tests {
        use super::*;

        #[test]
        fn test_positive_numbers() {
            assert_eq!(add(2, 3), 5);
        }

        #[test]
        fn test_negative_numbers() {
            assert_eq!(add(-2, -3), -5);
        }

        #[test]
        fn test_mixed_signs() {
            assert_eq!(add(-2, 3), 1);
        }
    }

    mod multiplication_tests {
        use super::*;

        #[test]
        fn test_positive_numbers() {
            assert_eq!(multiply(2, 3), 6);
        }

        #[test]
        fn test_by_zero() {
            assert_eq!(multiply(5, 0), 0);
        }
    }
}
```

Nested modules help organize related tests, making the test suite easier to navigate and maintain.

### Test Setup and Teardown

Sometimes tests need setup or cleanup. Rust doesn't have built-in setup/teardown hooks, but you can use regular Rust patterns:

```rust
struct TestContext {
    temp_dir: std::path::PathBuf,
}

impl TestContext {
    fn new() -> Self {
        let temp_dir = std::env::temp_dir().join(format!("test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        TestContext { temp_dir }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        //==========================================================
        // Cleanup happens automatically when TestContext is dropped
        //==========================================================
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_with_temp_directory() {
    let ctx = TestContext::new();

    //=============================
    // Use ctx.temp_dir for testing
    //=============================
    let test_file = ctx.temp_dir.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    assert!(test_file.exists());

    //==========================================
    // ctx is dropped here, cleaning up temp_dir
    //==========================================
}
```

This pattern uses RAII (Resource Acquisition Is Initialization) for automatic cleanup. The compiler guarantees cleanup happens, even if the test panics.

### Ignoring and Filtering Tests

During development, you might want to skip expensive or unfinished tests:

```rust
#[test]
#[ignore]
fn expensive_test() {
    //============================
    // This test takes a long time
    //============================
    std::thread::sleep(std::time::Duration::from_secs(10));
}

#[test]
#[ignore = "Not yet implemented"]
fn todo_test() {
    unimplemented!()
}
```

Run ignored tests with `cargo test -- --ignored`. Run all tests (including ignored) with `cargo test -- --include-ignored`.

Filter tests by name:

```bash
# Run only tests with "addition" in the name
cargo test addition

# Run tests in a specific module
cargo test math::tests::addition_tests
```

### Testing Private Functions

Tests in the same file can access private functions:

```rust
fn internal_helper(x: i32) -> i32 {
    x * 2
}

pub fn public_api(x: i32) -> i32 {
    internal_helper(x) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_helper() {
        //===========================
        // Can test private functions
        //===========================
        assert_eq!(internal_helper(5), 10);
    }

    #[test]
    fn test_public_api() {
        assert_eq!(public_api(5), 11);
    }
}
```

This is a deliberate design decision. Tests in the same module are part of the implementation, so they can access private details.

## Property-Based Testing

Traditional unit tests check specific examples: "2 + 2 = 4", "10 / 2 = 5". Property-based testing verifies general properties: "for any integers a and b, a + b = b + a". This approach finds edge cases you wouldn't think to test manually.

### The Problem with Example-Based Testing

Consider testing a sorting function:

```rust
fn sort(mut vec: Vec<i32>) -> Vec<i32> {
    vec.sort();
    vec
}

#[test]
fn test_sort() {
    assert_eq!(sort(vec![3, 1, 2]), vec![1, 2, 3]);
}
```

This test is fine, but what about:
- Empty vectors?
- Single-element vectors?
- Already-sorted vectors?
- Reverse-sorted vectors?
- Duplicate elements?
- Very large vectors?
- Vectors with MIN and MAX values?

You could write tests for each case, but you'd still miss edge cases. Property-based testing explores the input space automatically.

### Introduction to proptest

proptest is Rust's leading property-based testing library. It generates random test cases and verifies your properties hold:

```rust
//===================
// Add to Cargo.toml:
//===================
// [dev-dependencies]
//=================
// proptest = "1.0"
//=================

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sort_properties(mut vec: Vec<i32>) {
        let sorted = sort(vec.clone());

        //==============================================
        // Property 1: Output length equals input length
        //==============================================
        prop_assert_eq!(sorted.len(), vec.len());

        //=============================
        // Property 2: Output is sorted
        //=============================
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1] <= sorted[i]);
        }

        //===================================================
        // Property 3: Output contains same elements as input
        //===================================================
        vec.sort();
        prop_assert_eq!(sorted, vec);
    }
}
```

proptest generates hundreds of random vectors and verifies all three properties hold. If it finds a failure, it "shrinks" the input to find the minimal failing case.

### Shrinking: Finding Minimal Failing Cases

Shrinking is proptest's killer feature. When a test fails, proptest tries smaller, simpler inputs to find the smallest case that still fails:

```rust
fn buggy_absolute_value(x: i32) -> i32 {
    if x < 0 {
        -x
    } else {
        x
    }
}

proptest! {
    #[test]
    fn test_absolute_value(x: i32) {
        let result = buggy_absolute_value(x);
        prop_assert!(result >= 0);
    }
}
```

This test fails because `buggy_absolute_value(i32::MIN)` panics (overflow). proptest might initially find the failure with a large negative number, but it shrinks to the simplest failing case: `i32::MIN`.

### Custom Generators

Sometimes you need specific input patterns:

```rust
use proptest::prelude::*;

//=================================
// Generate vectors of length 1-100
//=================================
prop_compose! {
    fn vec_1_to_100()(vec in prop::collection::vec(any::<i32>(), 1..=100)) -> Vec<i32> {
        vec
    }
}

//============================
// Generate email-like strings
//============================
prop_compose! {
    fn email_strategy()(
        username in "[a-z]{3,10}",
        domain in "[a-z]{3,10}",
        tld in "(com|org|net)"
    ) -> String {
        format!("{}@{}.{}", username, domain, tld)
    }
}

proptest! {
    #[test]
    fn test_with_custom_generator(vec in vec_1_to_100()) {
        prop_assert!(!vec.is_empty());
        prop_assert!(vec.len() <= 100);
    }

    #[test]
    fn test_email_parsing(email in email_strategy()) {
        prop_assert!(email.contains('@'));
        prop_assert!(email.contains('.'));
    }
}
```

Custom generators let you focus testing on realistic inputs while still getting proptest's shrinking and reporting.

### Testing Invariants

Property-based testing excels at verifying invariants—properties that should always hold:

```rust
use std::collections::HashMap;

fn merge_maps(mut a: HashMap<String, i32>, b: HashMap<String, i32>) -> HashMap<String, i32> {
    for (k, v) in b {
        *a.entry(k).or_insert(0) += v;
    }
    a
}

proptest! {
    #[test]
    fn test_merge_properties(
        a: HashMap<String, i32>,
        b: HashMap<String, i32>,
    ) {
        let merged = merge_maps(a.clone(), b.clone());

        //======================================================
        // Property 1: All keys from both maps are in the result
        //======================================================
        for key in a.keys().chain(b.keys()) {
            prop_assert!(merged.contains_key(key));
        }

        //========================================
        // Property 2: Values are summed correctly
        //========================================
        for key in merged.keys() {
            let expected = a.get(key).unwrap_or(&0) + b.get(key).unwrap_or(&0);
            prop_assert_eq!(merged[key], expected);
        }

        //===============================================
        // Property 3: Merging with empty map is identity
        //===============================================
        let empty: HashMap<String, i32> = HashMap::new();
        prop_assert_eq!(merge_maps(a.clone(), empty.clone()), a);
    }
}
```

These properties completely specify `merge_maps`' behavior without testing specific examples.

### QuickCheck: Alternative Approach

QuickCheck is another property-based testing library, inspired by Haskell's QuickCheck:

```rust
//===================
// Add to Cargo.toml:
//===================
// [dev-dependencies]
//===================
// quickcheck = "1.0"
//===================
// quickcheck_macros = "1.0"

use quickcheck::{quickcheck, TestResult};
use quickcheck_macros::quickcheck;

#[quickcheck]
fn reverse_twice_is_identity(vec: Vec<i32>) -> bool {
    let mut reversed = vec.clone();
    reversed.reverse();
    reversed.reverse();
    vec == reversed
}

#[quickcheck]
fn concat_length(a: Vec<i32>, b: Vec<i32>) -> bool {
    let mut c = a.clone();
    c.extend(b.iter());
    c.len() == a.len() + b.len()
}
```

QuickCheck's syntax is slightly different from proptest, but the concept is the same. Choose based on your preference—both are excellent.

### When to Use Property-Based Testing

Property-based testing shines when:

- **Testing pure functions**: Functions without side effects have clear properties
- **Testing data structures**: Invariants like "BST is always sorted" are perfect for properties
- **Finding edge cases**: You want to discover bugs you haven't thought of
- **Testing serialization**: Round-tripping properties like `deserialize(serialize(x)) == x`

It's less useful for:

- **Testing specific business logic**: "User discount is 10% for orders over $100" is better as an example
- **Testing I/O**: Hard to generate meaningful random database queries or file operations
- **Complex stateful systems**: Can work but requires sophisticated generators

## Mock and Stub Patterns

Testing code with external dependencies—databases, HTTP APIs, file systems—is challenging. You don't want tests to depend on external services (slow, unreliable, expensive), but you need to verify your code interacts with them correctly. Mocking and stubbing solve this problem.

### Understanding Mocks vs Stubs

The terminology is often confused:

- **Stub**: A simple implementation that returns predetermined values
- **Mock**: A stub that also verifies it was called correctly
- **Fake**: A working implementation (e.g., in-memory database)
- **Spy**: Records calls for later verification

Rust's type system makes these patterns particularly elegant.

### Trait-Based Mocking

The most idiomatic approach uses traits:

```rust
//==================================
// Define a trait for the dependency
//==================================
trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String>;
}

//====================
// Real implementation
//====================
struct SmtpEmailService {
    server: String,
}

impl EmailService for SmtpEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        //=============================
        // Actually send email via SMTP
        //=============================
        println!("Sending to {} via {}", to, self.server);
        Ok(())
    }
}

//=================
// Mock for testing
//=================
struct MockEmailService {
    sent_emails: std::sync::Mutex<Vec<(String, String, String)>>,
}

impl MockEmailService {
    fn new() -> Self {
        MockEmailService {
            sent_emails: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn emails_sent(&self) -> Vec<(String, String, String)> {
        self.sent_emails.lock().unwrap().clone()
    }
}

impl EmailService for MockEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), String> {
        self.sent_emails.lock().unwrap().push((
            to.to_string(),
            subject.to_string(),
            body.to_string(),
        ));
        Ok(())
    }
}

//================================
// Application code uses the trait
//================================
struct UserService<E: EmailService> {
    email_service: E,
}

impl<E: EmailService> UserService<E> {
    fn register_user(&self, email: &str) -> Result<(), String> {
        //===========================
        // ... registration logic ...
        //===========================

        self.email_service.send_email(
            email,
            "Welcome!",
            "Thanks for registering",
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_registration_sends_email() {
        let mock = MockEmailService::new();
        let service = UserService {
            email_service: &mock,
        };

        service.register_user("user@example.com").unwrap();

        let emails = mock.emails_sent();
        assert_eq!(emails.len(), 1);
        assert_eq!(emails[0].0, "user@example.com");
        assert_eq!(emails[0].1, "Welcome!");
    }
}
```

This pattern is powerful: the real code uses `EmailService` trait, tests use `MockEmailService`, production uses `SmtpEmailService`. No mocking framework needed.

### Using mockall for Advanced Mocking

For complex mocking needs, the `mockall` crate provides a powerful framework:

```rust
//===================
// Add to Cargo.toml:
//===================
// [dev-dependencies]
//=================
// mockall = "0.12"
//=================

use mockall::{automock, predicate::*};

#[automock]
trait Database {
    fn get_user(&self, id: i32) -> Option<User>;
    fn save_user(&mut self, user: User) -> Result<(), String>;
}

struct User {
    id: i32,
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_mock() {
        let mut mock = MockDatabase::new();

        //=================
        // Set expectations
        //=================
        mock.expect_get_user()
            .with(eq(42))
            .times(1)
            .returning(|_| Some(User { id: 42, name: "Alice".to_string() }));

        mock.expect_save_user()
            .times(1)
            .returning(|_| Ok(()));

        //=============
        // Use the mock
        //=============
        let user = mock.get_user(42).unwrap();
        assert_eq!(user.name, "Alice");

        mock.save_user(user).unwrap();

        //=============================================
        // Automatically verifies expectations were met
        //=============================================
    }
}
```

mockall automatically generates mock implementations and verifies expectations, similar to mocking frameworks in other languages.

### Dependency Injection Patterns

Dependency injection makes testing easier by making dependencies explicit:

```rust
//===================
// Poor: Hard to test
//===================
struct PaymentProcessor {
    //======================
    // Hard-coded dependency
    //======================
}

impl PaymentProcessor {
    fn process_payment(&self, amount: f64) -> Result<(), String> {
        //============================
        // Directly calls external API
        //============================
        external_api::charge_card(amount)
    }
}

//=============================
// Better: Dependency injection
//=============================
trait PaymentGateway {
    fn charge(&self, amount: f64) -> Result<String, String>;
}

struct PaymentProcessor<G: PaymentGateway> {
    gateway: G,
}

impl<G: PaymentGateway> PaymentProcessor<G> {
    fn new(gateway: G) -> Self {
        PaymentProcessor { gateway }
    }

    fn process_payment(&self, amount: f64) -> Result<(), String> {
        let transaction_id = self.gateway.charge(amount)?;
        println!("Processed payment: {}", transaction_id);
        Ok(())
    }
}

//=======================
// Test with mock gateway
//=======================
struct MockGateway {
    should_succeed: bool,
}

impl PaymentGateway for MockGateway {
    fn charge(&self, amount: f64) -> Result<String, String> {
        if self.should_succeed {
            Ok(format!("txn_{}", amount))
        } else {
            Err("Payment failed".to_string())
        }
    }
}

#[test]
fn test_successful_payment() {
    let gateway = MockGateway { should_succeed: true };
    let processor = PaymentProcessor::new(gateway);

    assert!(processor.process_payment(99.99).is_ok());
}

#[test]
fn test_failed_payment() {
    let gateway = MockGateway { should_succeed: false };
    let processor = PaymentProcessor::new(gateway);

    assert!(processor.process_payment(99.99).is_err());
}
```

This pattern—using traits for dependencies and injecting implementations—is idiomatic Rust and makes testing straightforward.

### Test Doubles for I/O

File system and network operations need special handling:

```rust
trait FileSystem {
    fn read_file(&self, path: &str) -> std::io::Result<String>;
    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()>;
}

//====================
// Real implementation
//====================
struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_file(&self, path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()> {
        std::fs::write(path, content)
    }
}

//===========================
// In-memory fake for testing
//===========================
use std::collections::HashMap;
use std::sync::Mutex;

struct FakeFileSystem {
    files: Mutex<HashMap<String, String>>,
}

impl FakeFileSystem {
    fn new() -> Self {
        FakeFileSystem {
            files: Mutex::new(HashMap::new()),
        }
    }
}

impl FileSystem for FakeFileSystem {
    fn read_file(&self, path: &str) -> std::io::Result<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found"
            ))
    }

    fn write_file(&self, path: &str, content: &str) -> std::io::Result<()> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_string());
        Ok(())
    }
}

#[test]
fn test_file_operations() {
    let fs = FakeFileSystem::new();

    fs.write_file("/test.txt", "hello").unwrap();
    let content = fs.read_file("/test.txt").unwrap();

    assert_eq!(content, "hello");
}
```

This fake is fast, deterministic, and doesn't touch the actual file system.

## Integration Testing

Integration tests verify that multiple components work together correctly. Unlike unit tests, they test the public API of your crate and can test interactions between crates.

### Integration Test Structure

Integration tests live in the `tests/` directory:

```
my_project/
├── src/
│   ├── lib.rs
│   └── utils.rs
├── tests/
│   ├── integration_test.rs
│   └── common/
│       └── mod.rs
└── Cargo.toml
```

Each file in `tests/` is compiled as a separate crate:

```rust
//==========================
// tests/integration_test.rs
//==========================
use my_project::*;

#[test]
fn test_public_api() {
    let result = public_function();
    assert_eq!(result, expected_value);
}
```

Integration tests only have access to your crate's public API, just like external users.

### Common Test Code

Shared test utilities go in `tests/common/`:

```rust
//====================
// tests/common/mod.rs
//====================
use my_project::*;

pub fn setup_test_database() -> Database {
    Database::new(":memory:")
}

pub fn create_test_user() -> User {
    User {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
    }
}
```

```rust
//==========================
// tests/integration_test.rs
//==========================
mod common;

#[test]
fn test_with_common_utilities() {
    let db = common::setup_test_database();
    let user = common::create_test_user();

    //============================
    // Test using shared utilities
    //============================
}
```

The `common/mod.rs` pattern prevents Rust from treating `common` as a test file.

### Testing Binary Crates

Binary crates can be tested by moving logic to a library:

```
my_binary/
├── src/
│   ├── main.rs       # Thin wrapper
│   └── lib.rs        # Business logic
└── tests/
    └── integration.rs
```

```rust
//===========
// src/lib.rs
//===========
pub fn run(args: Args) -> Result<(), Error> {
    //==================
    // Application logic
    //==================
}

//============
// src/main.rs
//============
fn main() {
    let args = parse_args();
    if let Err(e) = my_binary::run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

//=====================
// tests/integration.rs
//=====================
use my_binary::*;

#[test]
fn test_application_logic() {
    let args = Args { /* ... */ };
    assert!(run(args).is_ok());
}
```

This structure makes the binary testable while keeping `main.rs` simple.

### Database Integration Tests

Testing with real databases requires setup and teardown:

```rust
//==============================
// tests/database_integration.rs
//==============================
use sqlx::PgPool;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/test".to_string());

    let pool = PgPool::connect(&database_url).await.unwrap();

    //===============
    // Run migrations
    //===============
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    //====================
    // Clear existing data
    //====================
    sqlx::query("TRUNCATE TABLE users, posts CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

#[tokio::test]
async fn test_user_creation() {
    let pool = setup_test_db().await;

    //==========
    // Test code
    //==========
    let user = create_user(&pool, "test@example.com").await.unwrap();
    assert_eq!(user.email, "test@example.com");
}
```

For parallel tests, use separate test databases or transactions:

```rust
use sqlx::{PgPool, Postgres, Transaction};

async fn run_in_transaction<F, Fut>(pool: &PgPool, test: F)
where
    F: FnOnce(Transaction<Postgres>) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut tx = pool.begin().await.unwrap();

    test(tx).await;

    //=============================================
    // Always rollback - test changes never persist
    //=============================================
    // (Transaction is dropped here, triggering rollback)
}

#[tokio::test]
async fn test_in_transaction() {
    let pool = setup_test_db().await;

    run_in_transaction(&pool, |mut tx| async move {
        sqlx::query("INSERT INTO users (email) VALUES ('test@example.com')")
            .execute(&mut *tx)
            .await
            .unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&mut *tx)
            .await
            .unwrap();

        assert_eq!(count.0, 1);
    }).await;

    //====================================================
    // Database is unchanged - transaction was rolled back
    //====================================================
}
```

This pattern runs tests in isolation without slow database cleanup.

### HTTP Integration Tests

Testing HTTP servers requires starting a test server:

```rust
//==========================
// tests/http_integration.rs
//==========================
use axum::{Router, routing::get};
use hyper::StatusCode;

async fn create_test_app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/user/:id", get(get_user))
}

#[tokio::test]
async fn test_hello_endpoint() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/")
                .body(axum::body::Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"Hello, World!");
}
```

For end-to-end tests with a running server:

```rust
use tokio::net::TcpListener;

#[tokio::test]
async fn test_full_server() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener.into_std().unwrap())
            .unwrap()
            .serve(create_test_app().await.into_make_service())
            .await
            .unwrap();
    });

    //=========================
    // Wait for server to start
    //=========================
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    //=======================
    // Make real HTTP request
    //=======================
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/", addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Hello, World!");
}
```

## Criterion Benchmarking

Performance matters, but guessing at optimizations is wasteful. Benchmarking measures actual performance, helping you make data-driven optimization decisions.

### Why Criterion?

Rust's standard library includes basic benchmarking, but it's unstable and limited. Criterion provides:

- **Statistical rigor**: Outlier detection, confidence intervals
- **Historical tracking**: Compare against previous runs
- **Regression detection**: Automatically flag performance degradations
- **Beautiful reports**: HTML graphs and tables

### Basic Criterion Benchmarks

```rust
//===================
// Add to Cargo.toml:
//===================
// [dev-dependencies]
//==================
// criterion = "0.5"
//==================
//
//==========
// [[bench]]
//==========
// name = "my_benchmark"
//================
// harness = false
//================

//========================
// benches/my_benchmark.rs
//========================
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

Run with `cargo bench`. Criterion runs the benchmark multiple times, detects and removes outliers, and reports statistics:

```
fib 20                  time:   [26.029 µs 26.251 µs 26.509 µs]
```

The `black_box` function prevents the compiler from optimizing away the computation.

### Comparing Implementations

Benchmark multiple implementations to choose the best:

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn sum_loop(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        sum += x;
    }
    sum
}

fn sum_iterator(data: &[i32]) -> i32 {
    data.iter().sum()
}

fn sum_fold(data: &[i32]) -> i32 {
    data.iter().fold(0, |acc, &x| acc + x)
}

fn benchmark_sum_implementations(c: &mut Criterion) {
    let mut group = c.benchmark_group("sum_implementations");

    let data: Vec<i32> = (0..1000).collect();

    group.bench_with_input(BenchmarkId::new("loop", data.len()), &data, |b, data| {
        b.iter(|| sum_loop(black_box(data)))
    });

    group.bench_with_input(BenchmarkId::new("iterator", data.len()), &data, |b, data| {
        b.iter(|| sum_iterator(black_box(data)))
    });

    group.bench_with_input(BenchmarkId::new("fold", data.len()), &data, |b, data| {
        b.iter(|| sum_fold(black_box(data)))
    });

    group.finish();
}

criterion_group!(benches, benchmark_sum_implementations);
criterion_main!(benches);
```

Criterion generates comparative graphs showing which implementation is fastest.

### Parameterized Benchmarks

Benchmark across different input sizes:

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn sort_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut data: Vec<i32> = (0..size).rev().collect();
            b.iter(|| {
                let mut d = data.clone();
                d.sort();
                black_box(d);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, sort_benchmark);
criterion_main!(benches);
```

This reveals how performance scales with input size—crucial for understanding algorithmic complexity.

### Throughput Measurement

Measure operations per second or bytes per second:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn parse_numbers(data: &str) -> Vec<i32> {
    data.lines()
        .filter_map(|line| line.parse().ok())
        .collect()
}

fn throughput_benchmark(c: &mut Criterion) {
    let data = (0..10000).map(|i| i.to_string()).collect::<Vec<_>>().join("\n");
    let data_bytes = data.len();

    let mut group = c.benchmark_group("parse_throughput");
    group.throughput(Throughput::Bytes(data_bytes as u64));

    group.bench_function("parse", |b| {
        b.iter(|| parse_numbers(black_box(&data)))
    });

    group.finish();
}

criterion_group!(benches, throughput_benchmark);
criterion_main!(benches);
```

Output includes throughput:

```
parse_throughput/parse  time:   [1.2034 ms 1.2156 ms 1.2289 ms]
                        thrpt:  [37.428 MiB/s 37.835 MiB/s 38.216 MiB/s]
```

### Profiling Integration

Criterion can integrate with profilers:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

fn profiled_benchmark(c: &mut Criterion) {
    c.bench_function("expensive_function", |b| {
        b.iter(|| expensive_function())
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = profiled_benchmark
}
criterion_main!(benches);
```

This generates flamegraphs showing where time is spent.

### Regression Testing

Criterion saves baseline measurements:

```bash
# Save current performance as baseline
cargo bench -- --save-baseline master

# Make changes...

# Compare against baseline
cargo bench -- --baseline master
```

Criterion reports whether performance regressed, improved, or stayed the same.

### Best Practices for Benchmarking

1. **Benchmark realistic scenarios**: Synthetic microbenchmarks can be misleading

2. **Run benchmarks in isolation**: Close other programs, disable CPU scaling

3. **Use black_box**: Prevent compiler optimizations that wouldn't happen in real code

4. **Warm up before measuring**: Account for cache effects

5. **Benchmark multiple input sizes**: Understand scaling behavior

6. **Track historical performance**: Detect regressions early

7. **Profile before optimizing**: Benchmarks tell you what's slow, profilers tell you why

## Conclusion

Testing and benchmarking are essential practices for reliable, performant software. Rust's testing ecosystem provides tools for every level:

- **Unit tests** verify individual components work correctly
- **Property-based testing** finds edge cases through randomized testing
- **Mocks and stubs** isolate code from external dependencies
- **Integration tests** verify components work together
- **Benchmarks** measure and track performance

The patterns we've explored—from basic assertions to property-based testing to Criterion benchmarks—form a comprehensive quality assurance strategy. Rust's type system and ownership model make many classes of bugs impossible, but thorough testing catches the bugs that remain.

Key takeaways:

1. **Write tests first, or at least early**—they're easier to write before you forget the requirements
2. **Use property-based testing for algorithms and data structures**—it finds bugs you wouldn't think to test
3. **Mock external dependencies**—fast, reliable tests that you can run anywhere
4. **Benchmark before optimizing**—measure, don't guess
5. **Track performance over time**—regression detection prevents slowdowns

Remember: The goal isn't 100% code coverage. The goal is confidence that your code works. Strategic testing—focusing on critical paths, edge cases, and complex logic—provides that confidence without drowning in test maintenance.

As your Rust projects grow, invest in good testing practices. The compile-time guarantees are powerful, but they're not magic. Comprehensive testing—unit, integration, property-based, and performance—completes the picture, giving you the confidence to ship reliable software.