# Testing & Benchmarking

Unit Test Patterns

- Problem: Business logic bugs slip through type system; manual testing misses edge cases; no verification of error paths
- Solution: Built-in #[test] framework; assert macros; #[should_panic]; organize with nested modules; RAII cleanup
- Why It Matters: Catch logic bugs type system can't detect; error paths rarely tested manually = production bugs
- Use Cases: Function correctness, error handling, panic verification, regression prevention, API contracts

Property-Based Testing

- Problem: Example tests miss edge cases; manually enumerating inputs tedious; corner cases unknown until production
- Solution: proptest/quickcheck generate random inputs; verify properties hold for all inputs; automatic shrinking to minimal failure
- Why It Matters: Finds bugs you didn't think to test; 100+ random inputs vs 3-5 manual examples; shrinking reveals root cause
- Use Cases: Pure functions, data structures, serialization round-trips, parsers, algorithms, invariant verification

Mock and Stub Patterns

- Problem: Testing with real DBs/APIs slow, unreliable, expensive; hard to test error conditions; tests require network/setup
- Solution: Trait-based dependency injection; mock implementations for testing; mockall for advanced expectations; fakes for I/O
- Why It Matters: Fast deterministic tests; test error paths easily; no external dependencies; parallel test execution safe
- Use Cases: Database operations, HTTP clients, file I/O, email services, payment gateways, external APIs, error scenarios

Integration Testing

- Problem: Unit tests don't catch inter-component bugs; need to test public API; binary crates hard to test
- Solution: tests/ directory for integration tests; shared utilities in tests/common/; move binary logic to lib.rs; test transactions
- Why It Matters: Verify components work together; test as external users would; catch integration bugs early
- Use Cases: Multi-component systems, public API verification, database workflows, HTTP servers, end-to-end scenarios

Criterion Benchmarking

- Problem: Guessing optimizations wastes time; no data on performance impact; regressions undetected; scaling behavior unknown
- Solution: Criterion for statistical benchmarks; compare implementations; parameterized tests; throughput measurement; baseline tracking
- Why It Matters: Measure don't guess; statistical rigor detects real improvements; regression detection prevents slowdowns; scaling analysis
- Use Cases: Algorithm comparison, performance optimization, regression detection, throughput analysis, scaling validation


This chapter explores Rust's testing ecosystem: built-in unit tests, property-based testing with proptest, mocking with traits, integration testing patterns, and Criterion benchmarks for performance measurement and regression detection.

## Pattern 1: Unit Test Patterns

**Problem**: Rust's type system catches memory safety bugs and many logic errors, but can't verify business logic correctness, mathematical correctness, or handle edge cases like division by zero, overflow conditions, or invalid state transitions. Manual testing is slow, incomplete, and doesn't catch regressions when refactoring. Error paths (Result::Err, panic conditions) rarely get manual testing but often have bugs. No automated verification that changes don't break existing functionality.

**Solution**: Use Rust's built-in test framework with `#[test]` attribute to mark test functions. Use assertion macros: `assert_eq!` for equality with good error messages, `assert_ne!` for inequality, `assert!` for boolean conditions. Test error cases with `Result::is_err()` and match statements. Use `#[should_panic]` attribute to verify panics occur when expected. Organize tests in nested `#[cfg(test)]` modules keeping tests near code. Use RAII pattern (Drop trait) for automatic test cleanup. Use `#[ignore]` for slow tests, filter with `cargo test <pattern>`.

**Why It Matters**: Type system guarantees memory safety but not correctness—function can compile yet return wrong answer. Manual testing catches only 10-20% of edge cases; automated tests cover 80%+. Error paths are production bug sources: they rarely execute in development but get hit in production. Regression prevention: refactoring without tests = fear-driven development. Documentation: tests show how API is intended to be used. Fast feedback: `cargo test` runs in seconds, finds bugs before commit. Rust's test isolation: each test runs in separate thread, failures don't cascade.

**Use Cases**: Function correctness verification (math functions, string processing, data transformations), error handling validation (Result errors, input validation, boundary conditions), panic verification (invalid inputs should panic, out-of-bounds access), regression prevention (test bugs found in production), API contract enforcement (public interface behavior), edge case coverage (empty inputs, max values, negative numbers), refactoring confidence (tests ensure behavior unchanged).

### The Basics of Rust Testing

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
    // Basic equality
    assert_eq!(5, 2 + 3);

    // Inequality
    assert_ne!(5, 6);

    // Boolean assertions
    assert!(true);
    assert!(5 > 3, "5 should be greater than 3");

    // Custom error messages
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
        // Cleanup happens automatically when TestContext is dropped
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

#[test]
fn test_with_temp_directory() {
    let ctx = TestContext::new();

    // Use ctx.temp_dir for testing
    let test_file = ctx.temp_dir.join("test.txt");
    std::fs::write(&test_file, "test content").unwrap();

    assert!(test_file.exists());

    // ctx is dropped here, cleaning up temp_dir
}
```

This pattern uses RAII (Resource Acquisition Is Initialization) for automatic cleanup. The compiler guarantees cleanup happens, even if the test panics.

### Ignoring and Filtering Tests

During development, you might want to skip expensive or unfinished tests:

```rust
#[test]
#[ignore]
fn expensive_test() {
    // This test takes a long time
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
        // Can test private functions
        assert_eq!(internal_helper(5), 10);
    }

    #[test]
    fn test_public_api() {
        assert_eq!(public_api(5), 11);
    }
}
```

This is a deliberate design decision. Tests in the same module are part of the implementation, so they can access private details.

## Pattern 2: Property-Based Testing

**Problem**: Example-based tests check specific inputs (2+2=4, [3,1,2] sorted is [1,2,3]) but miss edge cases you didn't think to test. Manually enumerating all edge cases is tedious and incomplete—what about empty lists? Single elements? Duplicates? MAX/MIN values? Pathological inputs like reverse-sorted arrays? You discover edge case bugs in production, not development. Coverage is limited by imagination: you test cases you think of, miss ones you don't. Writing exhaustive tests for every possible input is impossible.

**Solution**: Use property-based testing (proptest or quickcheck) to generate random inputs and verify properties hold universally. Define properties instead of examples: "sorted output length equals input length", "sorted output is ascending", "sorted contains same elements". proptest generates 100+ random inputs, runs tests, and if failure found, "shrinks" to minimal failing case. Write custom generators for domain-specific inputs (emails, valid dates, constrained ranges). Verify invariants: data structure properties that must always hold regardless of inputs. Test round-trip properties: `deserialize(serialize(x)) == x`.

**Why It Matters**: Finds bugs you didn't think to test—proptest explores input space systematically, discovering edge cases. Shrinking is killer feature: finds `i32::MIN` as minimal failing case for overflow, not random large negative number. Higher confidence: 100 random inputs provide better coverage than 5 manual examples. Invariant verification: properties like "BST left < node < right" tested across all possible trees. Regression prevention: new code path triggered by random input reveals bugs. Mathematical correctness: commutativity (a+b = b+a), associativity, identity properties verified universally. Saves time: write one property test instead of 50 example tests.

**Use Cases**: Pure function testing (math, string operations, no side effects), data structure invariants (BST ordering, heap properties, graph validity), serialization round-trips (JSON, bincode, protobuf), parser correctness (parse then unparse), algorithm properties (sorting, searching, hashing), cryptographic properties (encryption then decryption), compression round-trips (compress then decompress), state machine transitions (all transitions maintain invariants).

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

        // Property 1: Output length equals input length
        prop_assert_eq!(sorted.len(), vec.len());

        // Property 2: Output is sorted
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1] <= sorted[i]);
        }

        // Property 3: Output contains same elements as input
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

        // Property 1: All keys from both maps are in the result
        for key in a.keys().chain(b.keys()) {
            prop_assert!(merged.contains_key(key));
        }

        // Property 2: Values are summed correctly
        for key in merged.keys() {
            let expected = a.get(key).unwrap_or(&0) + b.get(key).unwrap_or(&0);
            prop_assert_eq!(merged[key], expected);
        }

        // Property 3: Merging with empty map is identity
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

## Pattern 3: Mock and Stub Patterns

**Problem**: Testing with real external dependencies (databases, HTTP APIs, SMTP servers, file systems, payment gateways) is slow (network latency, I/O overhead), unreliable (network failures, service downtime), expensive (API rate limits, paid services), requires setup (database installation, service credentials), hard to test error conditions (how to make database fail on command?), prevents parallel test execution (shared state conflicts), couples tests to external systems (tests break when external service changes).

**Solution**: Use trait-based dependency injection—define trait for dependency (EmailService, Database, PaymentGateway), implement real version for production, implement mock version for tests. Mock records calls for verification, stub returns predetermined values. Use mockall crate for advanced mocking with expectations (times called, parameter matching, return values). Create fakes for complex dependencies (in-memory database, fake file system using HashMap). Inject dependencies via generic parameters or trait objects. Use builder pattern for complex setup.

**Why It Matters**: Fast tests: mock returns instantly vs HTTP round-trip (1000x faster). Deterministic: no flaky tests from network issues. Error testing easy: mock can simulate any error condition on demand. Parallel execution safe: each test has own mock, no shared state. No external setup: tests run anywhere, CI doesn't need database/API credentials. Isolated testing: test one component without entire system. Verification: mocks ensure correct interaction (called with right params, right number of times). Cost savings: no API calls during testing.

**Use Cases**: Database operations testing (SQL queries, transactions without PostgreSQL/MySQL), HTTP client testing (API calls, retry logic, error handling without real servers), email service testing (verify emails sent without SMTP), file system operations (read/write without disk I/O), payment gateway testing (charge logic without Stripe/PayPal), authentication testing (login without auth server), cache testing (Redis operations without real Redis), message queue testing (Kafka/RabbitMQ logic without brokers).

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
        // Actually send email via SMTP
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
        // ... registration logic ...

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

        // Set expectations
        mock.expect_get_user()
            .with(eq(42))
            .times(1)
            .returning(|_| Some(User { id: 42, name: "Alice".to_string() }));

        mock.expect_save_user()
            .times(1)
            .returning(|_| Ok(()));

        // Use the mock
        let user = mock.get_user(42).unwrap();
        assert_eq!(user.name, "Alice");

        mock.save_user(user).unwrap();

        // Automatically verifies expectations were met
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
    // Hard-coded dependency
}

impl PaymentProcessor {
    fn process_payment(&self, amount: f64) -> Result<(), String> {
        // Directly calls external API
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

## Pattern 4: Integration Testing

**Problem**: Unit tests verify components in isolation but don't catch bugs in component interactions—components work individually but fail together. Public API untested: unit tests can access private functions, but users can't. Binary crates (src/main.rs) have no natural place for tests. Component integration bugs: database + business logic + HTTP work separately but fail combined. End-to-end workflows untested: multi-step operations like "register user, verify email, login" not covered. Test setup duplication: each unit test rebuilds mocks/fixtures. External user perspective missing: tests don't reflect how library actually used.

**Solution**: Create tests/ directory for integration tests—each file compiles as separate crate, only accesses public API. Shared utilities in tests/common/mod.rs prevent treating common as test file. Move binary logic to lib.rs, keep main.rs thin wrapper—makes binary testable. Use test transactions for database tests (begin transaction, run test, rollback—database unchanged). Start test servers for HTTP integration tests. Create test fixtures and builders for complex setup. Use #[tokio::test] for async integration tests.

**Why It Matters**: Integration bugs appear only when components combined—unit tests miss these. Public API testing ensures library usable as intended by external users. Real-world workflows tested: user registration flow, payment processing, data pipelines. Database integration tests catch SQL errors, schema mismatches, transaction issues. HTTP tests verify routing, middleware, serialization work together. Test isolation: each test file = separate crate = clean slate. CI/CD confidence: integration tests prove deployable code. Refactoring safety: can change internals if public API tests still pass.

**Use Cases**: Multi-component systems (web server + database + cache), public library API verification (ensure usability), database workflow testing (CRUD operations, transactions, migrations), HTTP server testing (routes, middleware, auth), binary application testing (CLI tools, services), end-to-end scenarios (user flows, data pipelines), cross-crate interaction testing (workspace members), deployment smoke tests (can system start and respond).

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

    // Test using shared utilities
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
    // Application logic
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

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap();

    // Clear existing data
    sqlx::query("TRUNCATE TABLE users, posts CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

#[tokio::test]
async fn test_user_creation() {
    let pool = setup_test_db().await;

    // Test code
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

    // Always rollback - test changes never persist
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

    // Database is unchanged - transaction was rolled back
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

    // Wait for server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make real HTTP request
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

## Pattern 5: Criterion Benchmarking

**Problem**: Guessing at optimizations wastes time—spent hours optimizing wrong function. No data on performance impact: did change make code faster or slower? No quantitative comparison between implementations (loop vs iterator vs fold). Performance regressions undetected until production: new feature slows entire system. Microbenchmarks unreliable: single run affected by system noise, CPU throttling, cache state. Don't know how performance scales: algorithm fast for small inputs, unacceptable for large. No baseline for comparison: is 10ms good or bad?

**Solution**: Use Criterion for statistical benchmarking—runs code multiple times, detects outliers, reports confidence intervals. Compare implementations: benchmark loop vs iterator, measure actual difference. Parameterized benchmarks: test with sizes 10, 100, 1000, 10000—reveal scaling behavior. Throughput measurement: report bytes/sec or operations/sec. Historical tracking: save baselines with `--save-baseline`, compare with `--baseline`. Use black_box to prevent compiler optimizing away code. Profile integration: generate flamegraphs showing where time spent. Regression detection: automatically flag slowdowns.

**Why It Matters**: Measure don't guess: optimization based on data, not intuition. Statistical rigor: Criterion detects real changes from noise (95% confidence intervals). Regression prevention: catch slowdowns before production—"20% slower" alert fails CI. Algorithm selection: choose fastest implementation based on benchmarks, not assumptions. Scaling analysis: understand O(N) vs O(N²) empirically. Optimization ROI: quantify improvement—"optimized from 100ms to 10ms" justifies time spent. Historical tracking: detect performance trends over time. Profiling integration: benchmark identifies slow function, profiler explains why.

**Use Cases**: Algorithm comparison (sort implementations, hash functions, compression algorithms), optimization validation (did refactor improve speed?), regression detection (CI fails if >10% slower), throughput analysis (parser MB/s, serialization throughput), scaling validation (performance vs input size), data structure benchmarks (Vec vs HashMap lookup), cache effectiveness (measure cache hit impact), optimization prioritization (profile then benchmark hot spots).

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

## Summary

This chapter covered comprehensive testing and benchmarking patterns for Rust:

1. **Unit Test Patterns**: Built-in #[test] framework, assertion macros, error/panic testing, RAII cleanup
2. **Property-Based Testing**: proptest/quickcheck random input generation, shrinking, invariant verification
3. **Mock and Stub Patterns**: Trait-based dependency injection, mockall expectations, fake implementations
4. **Integration Testing**: tests/ directory, public API testing, database transactions, HTTP servers
5. **Criterion Benchmarking**: Statistical analysis, implementation comparison, regression detection, throughput measurement

**Key Takeaways**:
- Type system catches memory bugs, tests catch logic bugs—both essential
- Property-based testing finds edge cases through randomness + shrinking—better than manual enumeration
- Mock external dependencies for fast, deterministic tests—1000x faster than real services
- Integration tests verify components work together—unit tests miss interaction bugs
- Benchmark before optimizing—measure don't guess, statistical rigor detects real improvements

**Testing Strategy**:
- Unit tests: 80% coverage focusing on business logic, error paths, edge cases
- Property tests: Pure functions, data structures, serialization—verify invariants hold
- Mocks: External dependencies (DB, HTTP, file I/O)—fast isolated tests
- Integration tests: Public API, multi-component workflows, end-to-end scenarios
- Benchmarks: Hot paths, algorithm comparisons, regression detection in CI

**Performance Guidelines**:
- Unit tests: Run in milliseconds, parallel execution, isolated state
- Property tests: 100-256 cases per test (configurable), automatic shrinking
- Mock tests: Instant execution, no I/O overhead, fully deterministic
- Integration tests: Seconds per test, may require setup (database, server)
- Benchmarks: Minutes for full suite, statistical significance requires iterations

**Common Patterns**:
```rust
// Unit test with error handling
#[test]
fn test_divide() {
    assert_eq!(divide(10, 2), Ok(5));
    assert!(divide(10, 0).is_err());
}

// Property test
proptest! {
    #[test]
    fn test_sort_property(mut vec: Vec<i32>) {
        let sorted = sort(vec.clone());
        vec.sort();
        prop_assert_eq!(sorted, vec);
    }
}

// Trait-based mock
trait Database {
    fn get_user(&self, id: i32) -> Option<User>;
}

struct MockDatabase {
    users: HashMap<i32, User>,
}

// Integration test
#[tokio::test]
async fn test_api_endpoint() {
    let app = create_test_app().await;
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), 200);
}

// Criterion benchmark
fn bench_implementations(c: &mut Criterion) {
    c.bench_function("algorithm", |b| {
        b.iter(|| expensive_function(black_box(input)))
    });
}
```

**Best Practices**:
- Test error paths: Error handling is bug-prone, test Err variants and panics
- Use RAII for cleanup: Drop trait ensures teardown even if test panics
- Organize with modules: Nested test modules group related tests
- Filter tests: `cargo test pattern` runs subset, `#[ignore]` for slow tests
- Mock external deps: Fast deterministic tests, no network/disk I/O
- Shrinking for debugging: proptest finds minimal failing case automatically
- Parameterized benchmarks: Test multiple input sizes to reveal scaling
- Baseline tracking: Compare against saved baseline to detect regressions
- Black box inputs: Prevent compiler optimizing away benchmarked code
- Statistical rigor: Criterion's outlier detection and confidence intervals prevent false conclusions

**CI/CD Integration**:
- `cargo test`: Run all tests (unit + integration), fail build on any failure
- `cargo test --ignored`: Run expensive tests in nightly CI
- `cargo bench -- --save-baseline main`: Save baseline on main branch
- `cargo bench -- --baseline main`: Compare PR against baseline, fail if >10% slower
- Test coverage tools: tarpaulin, llvm-cov for coverage reports
- Parallel execution: Tests run concurrently by default (use `--test-threads=1` to serialize)