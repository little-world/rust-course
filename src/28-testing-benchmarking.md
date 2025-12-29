# Testing & Benchmarking
This chapter explores Rust's testing ecosystem end-to-end: built-in unit tests, property-based testing with proptest/quickcheck, coverage-guided fuzzing via cargo-fuzz, mutation testing tools that stress-check your suite, formal verification with Kani/Prusti, trait-based mocking, integration testing patterns, and Criterion benchmarks for performance measurement and regression detection.

## Pattern 1: Unit Test Patterns

**Problem**: Rust ensures memory safety but not domain correctness; manual spot-checking misses edge cases, error paths, and regressions.

**Solution**: Lean on `#[test]`, assertion macros, and `#[should_panic]` to encode expectations. Keep tests near code via `#[cfg(test)]`, use RAII for setup/teardown, and tag slow suites with `#[ignore]`.

**Why It Matters**: Automated tests document intent, catch bugs before review, and provide fearless refactoring—especially on rarely exercised Err/panic branches.

**Use Cases**: Math/string helpers, API contracts, validation logic, panic semantics, regression reproducers, and any code that changes frequently.

### Example: The Basics of Rust Testing

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

### Example: Assertion Macros

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

### Example: Testing Error Cases

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

### Example: Testing Panics

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

### Example: Organizing Tests

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

### Example: Test Setup and Teardown

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

### Example: Ignoring and Filtering Tests

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

### Example: Testing Private Functions

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

**Problem**: Example-based tests cover only inputs you imagine, leaving unseen edge cases, MIN/MAX values, and weird permutations unchecked.

**Solution**: Property-based testing (proptest/quickcheck) generates hundreds of random inputs and shrinks failures, letting you describe invariants instead of enumerating cases.

**Why It Matters**: Automatic exploration surfaces bugs humans miss, shrinking provides minimal reproducers, and one property can replace dozens of example tests.

**Use Cases**: Pure functions, data-structure invariants, serialization round-trips, parsers, crypto/compression transforms, and deterministic state machines.

## Example: Can do better
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

### Example: Introduction to proptest

proptest is Rust's leading property-based testing library. It generates random test cases and verifies your properties hold:

```rust
// Add to Cargo.toml:
// [dev-dependencies]
// proptest = "1.0"

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

### Example: Shrinking: Finding Minimal Failing Cases

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

### Example: Custom Generators

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

### Example: Testing Invariants

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

### Example: QuickCheck 

QuickCheck is another property-based testing library, inspired by Haskell's QuickCheck:

```rust
// Add to Cargo.toml:
// [dev-dependencies]
// quickcheck = "1.0"
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

### Example: When to Use Property-Based Testing

Property-based testing shines when:

- **Testing pure functions**: Functions without side effects have clear properties
- **Testing data structures**: Invariants like "BST is always sorted" are perfect for properties
- **Finding edge cases**: You want to discover bugs you haven't thought of
- **Testing serialization**: Round-tripping properties like `deserialize(serialize(x)) == x`

It's less useful for:

- **Testing specific business logic**: "User discount is 10% for orders over $100" is better as an example
- **Testing I/O**: Hard to generate meaningful random database queries or file operations
- **Complex stateful systems**: Can work but requires sophisticated generators

## Pattern 3: Coverage-Guided Fuzzing

**Problem**: Static test sets rarely include adversarial byte sequences, so parsers and `unsafe` code still panic or blow up on malformed inputs.

**Solution**: `cargo-fuzz` (libFuzzer/AFL) mutates inputs guided by coverage, hammering targets that accept `&[u8]` or `Arbitrary` structs while sanitizers catch UB.

**Why It Matters**: Fuzzers discover crashers humans never craft, shrink them to minimal reproducers, and can run unattended to guard against future regressions.

**Use Cases**: Binary/text parsers, protocol stacks, CLI argument handling, unsafe abstractions, codecs, deserializers, and any surface open to untrusted data.

### Example: Setting Up cargo-fuzz

```bash
cargo install cargo-fuzz
cargo fuzz init
```

This creates a `fuzz/` workspace. Add a target:

```rust
// fuzz_targets/parse_expr.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(expr) = my_crate::Expr::parse_from_bytes(data) {
        // Round trip: serialize then parse again
        let encoded = expr.to_bytes();
        let decoded = my_crate::Expr::parse_from_bytes(&encoded).unwrap();
        assert_eq!(expr, decoded);
    }
});
```

Run it with `cargo fuzz run parse_expr`. Crashes are saved in `fuzz/artifacts/parse_expr/`.

### Example: Fuzzing Structured Inputs with `arbitrary`

```rust
// fuzz_targets/http_request.rs
#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Request<'a> {
    method: &'a str,
    path: &'a str,
    body: &'a [u8],
}

fuzz_target!(|req: Request| {
    let _ = my_http::handle_request(req.method, req.path, req.body);
});
```

The `arbitrary` derive creates structured random data (methods, paths, payloads), enabling deeper protocol coverage. Persist interesting seeds by copying them into `fuzz/corpus/http_request/`.

### Example: Sanitizers and CI

Enable sanitizers to catch undefined behavior:

```bash
RUSTFLAGS="-Zsanitizer=address" \
    RUSTC_BOOTSTRAP=1 \
    cargo fuzz run parse_expr
```

In CI, run fuzzers for a bounded time:

```bash
cargo fuzz run parse_expr -- -max_total_time=60
```

Store the corpus to reuse progress between runs.

## Pattern 4: Mutation Testing

**Problem**: Coverage numbers say code executed, not that tests would fail if behavior changes; weak assertions let bugs slip through untouched.

**Solution**: Mutation tools (`cargo-mutants`, `mutagen`) systematically tweak operators, constants, and control flow, then re-run tests to see which mutations survive.

**Why It Matters**: Surviving mutants highlight missing or shallow assertions, giving a concrete to-do list for hardening critical logic.

**Use Cases**: Pricing/auth pipelines, parsers, financial formulas, protocol state machines—any code where regressions are costly.

### Example: cargo-mutants Workflow

```bash
cargo install cargo-mutants
cargo mutants
```

Sample output:

```
Mutant 12: src/calculator.rs:42 replaced `>` with `>=`
Result: survived (tests passed)
```

Add or strengthen tests until important mutants die, then re-run with `cargo mutants --mutants 12` to confirm.

### Example: Targeted Mutants

Focus on hot modules:

```bash
cargo mutants --mutate src/pricing.rs --mutate src/tax.rs
```

Pair with `--list` to inspect generated mutants before running them.

### Example: Mutagen Annotations

`mutagen` instruments code at compile time:

```rust
// Cargo.toml
[dev-dependencies]
mutagen = "0.1"

// src/lib.rs
#[cfg_attr(test, mutagen::mutate)]
pub fn is_eligible(age: u8) -> bool {
    age >= 18
}
```

Running `cargo test` under `RUSTFLAGS="--cfg mutate"` generates mutants on the fly, surfacing weak tests without separate tooling.

## Pattern 5: Formal Verification with Kani

**Problem**: Even deep tests only sample behaviors; safety-critical code sometimes needs proofs that no input can violate invariants.

**Solution**: Model checkers like Kani or provers like Prusti explore all executions within bounds using `#[kani::proof]` functions and nondeterministic inputs.

**Why It Matters**: Proofs guarantee absence of panics/overflow in small kernels, validating `unsafe` code or financial logic beyond what fuzzing can cover.

**Use Cases**: Unsafe abstractions, lock-free primitives, crypto/math kernels, serialization code, and embedded control algorithms.

### Example: Verifying a Safe Add

```rust
// src/lib.rs
pub fn checked_add(a: u32, b: u32) -> Option<u32> {
    a.checked_add(b)
}

// proofs/add.rs
#[kani::proof]
fn checked_add_never_wraps() {
    let a = kani::any::<u32>();
    let b = kani::any::<u32>();
    if let Some(sum) = checked_add(a, b) {
        assert!(sum >= a && sum >= b);
    }
}
```

Run `cargo kani proofs/add.rs`. Kani symbolically explores all `u32` combinations and proves the postcondition.

### Example: Proving State Machines

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
enum DoorState { Locked, Unlocked }

fn next(state: DoorState, code_entered: bool) -> DoorState {
    match (state, code_entered) {
        (DoorState::Locked, true) => DoorState::Unlocked,
        (DoorState::Unlocked, false) => DoorState::Locked,
        _ => state,
    }
}

#[kani::proof]
fn door_never_skips_locked_state() {
    let code1 = kani::any::<bool>();
    let code2 = kani::any::<bool>();

    let s1 = next(DoorState::Locked, code1);
    let s2 = next(s1, code2);

    assert!(matches!(s2, DoorState::Locked | DoorState::Unlocked));
}
```

For more complex models, consider Prusti or Creusot for contract-based verification.

## Pattern 6: Mock and Stub Patterns

**Problem**: Tests that talk to real databases, HTTP APIs, or queues are slow, flaky, and hard to coerce into failure modes.

**Solution**: Depend on traits, supply real implementations in production and mocks/fakes/stubs in tests (handwritten or via `mockall`), and inject them via generics or builders.

**Why It Matters**: Mocked tests run instantly, can simulate any error, and remain deterministic/parallelizable without external setup.

**Use Cases**: Database adapters, HTTP clients, payment/email integrations, file/queue abstractions, cache layers, or any boundary crossing process.

### Example: Trait-Based Mocking

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

### Example: Using mockall for Advanced Mocking

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

### Example: Dependency Injection Patterns

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

### Example: Test Doubles for I/O

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

## Pattern 7: Integration Testing

**Problem**: Unit tests hit internals but miss failures in public APIs, cross-component wiring, and real workflows.

**Solution**: Place integration tests under `tests/` so each file is its own crate using only the public surface, share setup via `tests/common`, and spin up real dependencies (DB transactions, HTTP servers, binaries).

**Why It Matters**: Validates that components cooperate as deployed, catches schema/serialization/API mismatches, and documents usage exactly as consumers experience it.

**Use Cases**: Web stacks (HTTP + DB + cache), CLI binaries, public libraries, migrations, multi-step business flows, and workspace crates that must interoperate.

### Example: Integration Tests Structure
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

### Example: Common Test Code

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

### Example: Testing Binary Crates

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

### Example: Database Integration Tests

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

### Example: HTTP Integration Tests

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

## Pattern 8: Criterion Benchmarking

**Problem**: Optimizing without measurements wastes time and hides regressions; single-run microbenchmarks are noisy and opaque about scaling.

**Solution**: Criterion automates statistical benchmarking, comparing implementations, varying input sizes, measuring throughput, and storing baselines while `black_box` thwarts dead-code elimination.

**Why It Matters**: Data-driven performance work avoids guesswork, flags slowdowns before release, and quantifies improvement with confidence intervals.

**Use Cases**: Algorithm comparisons, hot-path validation, throughput analysis, regression detection in CI, and picking between alternative data structures or implementations.

### Example: Basic Criterion Benchmarks

```rust
// Add to Cargo.toml:
// [dev-dependencies]
// criterion = "0.5"
//
// [[bench]]
// name = "my_benchmark"
// harness = false

// benches/my_benchmark.rs
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

### Example: Comparing Implementations

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

### Example: Parameterized Benchmarks

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

### Example: Throughput Measurement

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

### Example: Profiling Integration

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

### Example: Regression Testing

Criterion saves baseline measurements:

```bash
# Save current performance as baseline
cargo bench -- --save-baseline master

# Make changes...

# Compare against baseline
cargo bench -- --baseline master
```

Criterion reports whether performance regressed, improved, or stayed the same.

### Example: Best Practices for Benchmarking

1. **Benchmark realistic scenarios**: Synthetic microbenchmarks can be misleading

2. **Run benchmarks in isolation**: Close other programs, disable CPU scaling

3. **Use black_box**: Prevent compiler optimizations that wouldn't happen in real code

4. **Warm up before measuring**: Account for cache effects

5. **Benchmark multiple input sizes**: Understand scaling behavior

6. **Track historical performance**: Detect regressions early

7. **Profile before optimizing**: Benchmarks tell you what's slow, profilers tell you why

### Summary

This chapter covered comprehensive testing and benchmarking patterns for Rust:

1. **Unit Test Patterns**: Built-in #[test] framework, assertion macros, error/panic testing, RAII cleanup
2. **Property-Based Testing**: proptest/quickcheck random input generation, shrinking, invariant verification
3. **Coverage-Guided Fuzzing**: cargo-fuzz/libFuzzer targets, sanitizers, corpus management
4. **Mutation Testing**: cargo-mutants/mutagen workflows to measure assertion strength
5. **Formal Verification**: Kani/Prusti proofs for critical invariants and `unsafe` code
6. **Mock and Stub Patterns**: Trait-based dependency injection, mockall expectations, fake implementations
7. **Integration Testing**: tests/ directory, public API testing, database transactions, HTTP servers
8. **Criterion Benchmarking**: Statistical analysis, implementation comparison, regression detection, throughput measurement

**Key Takeaways**:
- Type system catches memory bugs; layered test techniques catch logic bugs—use both
- Property tests and fuzzers explore input space automatically, surfacing edge-case crashers humans miss
- Mutation testing proves your suite fails when behavior changes, preventing false confidence from raw coverage numbers
- Formal verification tools offer mathematical guarantees for small, critical components
- Mocking and integration tests provide fast feedback on both isolated components and end-to-end flows
- Benchmark before optimizing—Criterion’s statistics prevent chasing noise

**Testing Strategy**:
- Unit tests: Cover business logic, panics, and regression scenarios; run on every commit
- Property tests: Apply to pure functions/data structures; run in CI with reasonable case limits
- Fuzzers: Run locally for long sessions and in CI with `-max_total_time` budgets; persist corpora
- Mutation tests: Schedule periodically (e.g., nightly) on core modules to detect assertion gaps
- Formal proofs: Target `unsafe`, financial, or safety-critical code paths with `cargo kani` or Prusti
- Mocks & integration tests: Exercise external interactions quickly, then confirm workflows end-to-end
- Benchmarks: Track hot paths and compare implementations before shipping optimizations

**Performance Guidelines**:
- Unit/property/mocked tests run in milliseconds; keep them parallelizable
- Fuzzing sessions often run minutes to hours—use timeouts for CI and longer runs locally
- Mutation test suites can take minutes per module; narrow scope with `--mutate` filters
- Formal proofs may take seconds-minutes depending on bounds; break proofs into small focused functions
- Integration tests may need dedicated resources (DB, HTTP servers) and run in seconds
- Benchmarks require multiple iterations for significance—expect minutes for complete suites

**Best Practices**:
- Test error paths; use RAII for cleanup; group related tests with nested modules
- Filter tests via `cargo test pattern`; tag slow ones with `#[ignore]`
- Store fuzz corpora and crash reproducers; run fuzzers under sanitizers for maximum signal
- Review surviving mutants immediately; they highlight missing assertions
- Keep proofs small and composable; verify helper functions before large ones
- Track benchmark baselines and compare across commits; use `black_box` to prevent dead-code elimination

**CI/CD Integration**:
- `cargo test`: Run all tests (unit + integration); fail build on any failure
- `cargo test --ignored`: Schedule expensive/slow suites nightly
- `cargo fuzz run target -- -max_total_time=60`: Run fuzzers with capped time in CI; archive updated corpora
- `cargo mutants --mutate src/core.rs --report`: Periodic mutation runs with HTML reports
- `cargo kani proofs/add.rs`: Prove critical invariants before merging changes to safety-critical modules
- `cargo bench -- --save-baseline main` / `--baseline main`: Track performance regressions
- Coverage + lint tooling (tarpaulin, llvm-cov, clippy) + parallel test threads keep feedback fast (use `--test-threads=1` when shared fixtures demand serialization)
