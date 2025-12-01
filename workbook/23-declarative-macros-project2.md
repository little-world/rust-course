# Chapter 23: Declarative Macros - Project 2

## Project 2: Custom Test Framework with Macro Generation

### Problem Statement

Build a custom test framework using declarative macros that generates test cases from compact specifications, supports property-based testing patterns, parametric tests, and test groups with setup/teardown. The system should auto-generate test functions, provide helpful assertion macros with better error messages than standard library, and generate test reports. Unlike procedural test macros, this demonstrates pure declarative macro capabilities.

### Use Cases

- **Data-driven testing** - Generate 100 test cases from specification table
- **Property-based testing** - Test mathematical properties across ranges
- **Parametric tests** - Same test logic with different inputs
- **API testing** - Generate tests for all endpoints from specification
- **Regression test suites** - Test previous bugs don't reoccur
- **Integration testing** - Test multiple component interactions
- **Benchmark generation** - Create performance tests from templates

### Why It Matters

**Test Explosion Problem**: Testing function with 10 edge cases means writing 10 test functions—300 lines of boilerplate for 30 lines of unique logic. Adding new edge case requires copying entire test function. Test matrix (3 inputs × 4 configs = 12 tests) is 240 lines of copy-paste code.

**Macro Solution**: `generate_tests! { add { positive: (2, 3) => 5, negative: (-2, -3) => -5, ... } }` generates 10 test functions from 10 lines. Adding case = adding one line. Test matrix macro generates all combinations automatically.

**Better Error Messages**: Standard `assert_eq!(a, b)` shows "assertion failed: `(left == right)`" with left/right values. Custom `assert_that!(sum).equals(5)` shows "Expected sum to equal 5, but got 8". Contextual messages 10x faster debugging.

**Compile-Time Test Generation**: Tests generated at compile time, not runtime. Test runner sees N distinct test functions, not one function with loop. Failure in case 47/100 stops at case 47 with its specific inputs, not somewhere in a loop. Parallel test execution works (each generated test runs independently).

Example test reduction:
```
Manual tests:     100 test cases × 20 lines = 2000 lines, 1 week to write
Macro-generated:  1 template + 100 specs = 150 lines, 1 hour to write
Maintenance:      Change 100 tests vs change 1 template
```

---

## Milestone 1: Basic Test Case Generation

### Introduction

Before building complex test frameworks, understand how macros generate multiple functions from templates. This milestone teaches repetition patterns for generating test functions and basic assertion macros.

**Why Start Here**: Core skill is translating compact specification into expanded code. Pattern: one test spec → one `#[test] fn`. Learning this pattern enables all subsequent features.

### Architecture

**Macros:**
- `test_case!` - Generates a single test function
  - **Pattern**: `test_case! { name: test_name, input: (args), expect: result }`
  - **Expands to**: `#[test] fn test_name() { assert_eq!(func(args), result); }`
  - **Role**: Basic building block for generated tests

- `test_suite!` - Generates multiple test functions
  - **Pattern**: `test_suite! { function_name { case1: args => result, case2: ... } }`
  - **Expands to**: Multiple `#[test]` functions
  - **Role**: Main test generation interface

- `assert_that!` - Better assertion macro
  - **Pattern**: `assert_that!(expr).equals(expected)`
  - **Expands to**: Custom assertion with context
  - **Role**: Improved error messages

### Checkpoint Tests

```rust
#[test]
fn test_test_case_macro_generates_function() {
    // This test verifies the macro generates valid test functions
    // We can't directly test macro output, but we can run generated tests
    test_case! {
        name: generated_test,
        run: {
            let x = 2 + 2;
            assert_eq!(x, 4);
        }
    }

    // The generated test will run with `cargo test`
}

#[test]
fn test_suite_generates_multiple_tests() {
    fn add(a: i32, b: i32) -> i32 { a + b }

    test_suite! {
        add {
            positive: (2, 3) => 5,
            negative: (-2, -3) => -5,
            zero: (0, 5) => 5,
        }
    }

    // This generates 3 test functions:
    // - test_add_positive
    // - test_add_negative
    // - test_add_zero
}

#[test]
fn test_assert_that_macro() {
    let value = 10;

    assert_that!(value).equals(10);
    assert_that!(value).is_greater_than(5);
    assert_that!(value).is_less_than(20);

    let text = "hello";
    assert_that!(text).equals("hello");
    assert_that!(text.len()).equals(5);
}

#[test]
#[should_panic(expected = "Expected value to equal 20, but got 10")]
fn test_assert_that_fails_with_message() {
    let value = 10;
    assert_that!(value).equals(20);
}
```

### Starter Code

```rust
//================================================
// Milestone 1: Basic test generation macros
//================================================

// TODO: Implement test_case! macro for single test generation
macro_rules! test_case {
    (
        name: $name:ident,
        run: $body:block
    ) => {
        // TODO: Generate a test function
        // Hint: Use #[test] attribute and expand $body
        #[test]
        fn $name() {
            $body
        }
    };
}

// TODO: Implement test_suite! macro for multiple test generation
macro_rules! test_suite {
    (
        $fn_name:ident {
            $(
                $test_name:ident: ($($input:expr),*) => $expected:expr
            ),* $(,)?
        }
    ) => {
        // TODO: Generate multiple test functions
        // Each test should call $fn_name with inputs and assert result
        // Hint: Use repetition $( ... )*
        $(
            #[test]
            fn $test_name() {
                // TODO: Call function and assert
                // Hint: let result = $fn_name($($input),*);
                //       assert_eq!(result, $expected);
                todo!("Generate test body")
            }
        )*
    };
}

// TODO: Implement assert_that! macro for better assertions
macro_rules! assert_that {
    ($value:expr) => {
        // TODO: Create assertion builder
        // Return a struct that has methods like .equals(), .is_greater_than()
        // For now, create an inline implementation
        AssertionBuilder {
            value: $value,
            expr: stringify!($value),
        }
    };
}

// Helper struct for assertion builder pattern
struct AssertionBuilder<T> {
    value: T,
    expr: &'static str,
}

impl<T: std::fmt::Debug + PartialEq> AssertionBuilder<T> {
    fn equals(self, expected: T) {
        if self.value != expected {
            panic!(
                "Expected {} to equal {:?}, but got {:?}",
                self.expr, expected, self.value
            );
        }
    }
}

impl<T: std::fmt::Debug + PartialOrd> AssertionBuilder<T> {
    fn is_greater_than(self, expected: T) {
        // TODO: Implement comparison assertion
        // Hint: Similar to equals but use >
        todo!("Implement is_greater_than")
    }

    fn is_less_than(self, expected: T) {
        // TODO: Implement comparison assertion
        todo!("Implement is_less_than")
    }
}

fn main() {
    println!("Run `cargo test` to execute generated tests");
}
```

**Implementation Hints:**
1. Test functions need `#[test]` attribute to be recognized by test runner
2. Use `stringify!($expr)` to get string representation of expression for error messages
3. Repetition `$( ... )*` generates code for each test case
4. Test function names must be unique—use `$test_name` directly
5. `AssertionBuilder` implements fluent API pattern for chaining assertions

---

## Milestone 2: Parametric Tests with Test Matrix

### Introduction

**Why Milestone 1 Isn't Enough**: Testing function with multiple inputs and multiple configurations requires Cartesian product of test cases. Testing `parse_int` with ["123", "-45", "0"] × [base 10, base 16] = 6 tests. Manually writing 6 test functions is tedious.

**The Improvement**: Generate test matrix automatically from separate lists of parameters. `test_matrix! { inputs: [...], configs: [...] }` generates all combinations.

**Optimization (Test Coverage)**: Combinatorial testing finds bugs in parameter interactions. Testing each input individually and each config individually misses bugs that only occur with specific combinations. Matrix testing catches these with minimal code.

### Architecture

**New Macros:**
- `test_matrix!` - Generates Cartesian product of test cases
  - **Pattern**: `test_matrix! { fn_name: base { params1: [values], params2: [values] } }`
  - **Expands to**: N×M test functions for all combinations
  - **Role**: Exhaustive combination testing

- `parametric_test!` - Single test with multiple parameter sets
  - **Pattern**: `parametric_test! { test_name { case1: [params], case2: [params] } }`
  - **Expands to**: Multiple test functions from same template
  - **Role**: DRY for similar tests with different data

### Checkpoint Tests

```rust
#[test]
fn test_parametric_test_generation() {
    fn multiply(a: i32, b: i32) -> i32 { a * b }

    parametric_test! {
        multiply_tests {
            two_times_three: [2, 3, 6],
            five_times_four: [5, 4, 20],
            zero_times_ten: [0, 10, 0],
        }
        |a, b, expected| {
            assert_eq!(multiply(a, b), expected);
        }
    }
}

#[test]
fn test_matrix_generation() {
    fn parse_int(s: &str, base: u32) -> Result<i32, std::num::ParseIntError> {
        i32::from_str_radix(s, base)
    }

    test_matrix! {
        parse_int_matrix {
            inputs: ["10", "20", "FF"],
            bases: [10, 16],
        }
        |input, base| {
            let result = parse_int(input, base);
            assert!(result.is_ok());
        }
    }

    // Generates 3 × 2 = 6 test functions
}

#[test]
fn test_three_dimensional_matrix() {
    fn format_string(s: &str, uppercase: bool, prefix: &str) -> String {
        let s = if uppercase { s.to_uppercase() } else { s.to_lowercase() };
        format!("{}{}", prefix, s)
    }

    test_matrix! {
        format_matrix {
            strings: ["hello", "world"],
            uppercase: [true, false],
            prefixes: [">>", "**"],
        }
        |s, upper, prefix| {
            let result = format_string(s, upper, prefix);
            assert!(result.starts_with(prefix));
        }
    }

    // Generates 2 × 2 × 2 = 8 test functions
}
```

### Starter Code

```rust
// TODO: Implement parametric_test! macro
macro_rules! parametric_test {
    (
        $test_group:ident {
            $(
                $test_name:ident: [$($param:expr),* $(,)?]
            ),* $(,)?
        }
        |$($param_name:ident),*| $body:block
    ) => {
        // TODO: Generate one test function per parameter set
        // Each test passes parameters to the test body
        $(
            #[test]
            fn $test_name() {
                // TODO: Bind parameters and execute body
                // Need to destructure the parameter array
                // This is tricky—may need nested macro
                todo!("Generate parametric test")
            }
        )*
    };
}

// TODO: Implement test_matrix! for Cartesian product
macro_rules! test_matrix {
    (
        $test_group:ident {
            $param1_name:ident: [$($param1:expr),* $(,)?],
            $param2_name:ident: [$($param2:expr),* $(,)?],
        }
        |$arg1:ident, $arg2:ident| $body:block
    ) => {
        // TODO: Generate nested loops as separate tests
        // For each param1 value, for each param2 value, generate test
        // Need to create unique test names: paste crate or manual naming
        paste::paste! {
            $(
                $(
                    #[test]
                    fn [<test_ $test_group _ $param1_name _ $param1 _ $param2_name _ $param2>]() {
                        let $arg1 = $param1;
                        let $arg2 = $param2;
                        $body
                    }
                )*
            )*
        }
    };
}

// Simpler version without paste crate (requires manual indexing)
macro_rules! test_matrix_simple {
    (
        $test_base:ident {
            inputs: [$($input:expr),* $(,)?],
            configs: [$($config:expr),* $(,)?],
        }
        |$arg1:ident, $arg2:ident| $body:block
    ) => {
        // TODO: Generate tests with indexed names
        // Challenge: Need unique names for each combination
        // Without paste crate, use counter macro or indices
        todo!("Implement simple test matrix")
    };
}
```

**Implementation Hints:**
1. Use `paste` crate for identifier concatenation: `paste::paste! { [<test_ $a _ $b>] }`
2. Nested repetitions: `$( $( ... )* )*` for Cartesian product
3. Parameter binding: capture params in closure or directly in test body
4. Test naming: must be unique, consider hashing or sequential numbering
5. For 3D matrix, triple-nest repetitions

---

## Milestone 3: Setup/Teardown and Test Groups

### Introduction

**Why Milestone 2 Isn't Enough**: Many tests need common setup (create test database, temp files) and cleanup. Repeating setup in every test is boilerplate. Tests that share resources need grouped execution.

**The Improvement**: Add `setup!` and `teardown!` blocks that run before/after each test. Group related tests with shared context. Generate test modules with common fixtures.

**Optimization (Test Isolation)**: Proper teardown prevents test pollution—test A's leftover state affects test B. Setup/teardown ensures each test runs in clean environment. Parallel test execution safe when tests properly isolated.

### Architecture

**New Macros:**
- `test_group!` - Groups tests with shared setup/teardown
  - **Pattern**: `test_group! { name { setup: {...}, tests: {...}, teardown: {...} } }`
  - **Expands to**: Module with test functions and setup/teardown
  - **Role**: Test organization and resource management

- `with_fixtures!` - Tests that receive initialized fixtures
  - **Pattern**: `with_fixtures! { fixture_name => fixture_expr, test: ... }`
  - **Expands to**: Tests with automatic fixture creation
  - **Role**: DRY fixture management

### Checkpoint Tests

```rust
#[test]
fn test_group_with_setup_teardown() {
    use std::fs;
    use std::path::PathBuf;

    test_group! {
        file_operations {
            setup: {
                let temp_dir = std::env::temp_dir().join("test_group");
                fs::create_dir_all(&temp_dir).unwrap();
                temp_dir
            },

            tests: {
                test_create_file(temp_dir) {
                    let file_path = temp_dir.join("test.txt");
                    fs::write(&file_path, "test content").unwrap();
                    assert!(file_path.exists());
                },

                test_read_file(temp_dir) {
                    let file_path = temp_dir.join("data.txt");
                    fs::write(&file_path, "data").unwrap();
                    let content = fs::read_to_string(&file_path).unwrap();
                    assert_eq!(content, "data");
                },
            },

            teardown: {
                fs::remove_dir_all(temp_dir).ok();
            }
        }
    }
}

#[test]
fn test_fixtures_macro() {
    #[derive(Debug)]
    struct TestDatabase {
        data: Vec<i32>,
    }

    impl TestDatabase {
        fn new() -> Self {
            TestDatabase { data: vec![1, 2, 3] }
        }

        fn insert(&mut self, value: i32) {
            self.data.push(value);
        }

        fn count(&self) -> usize {
            self.data.len()
        }
    }

    with_fixtures! {
        db => TestDatabase::new(),

        test_insert {
            db.insert(4);
            assert_eq!(db.count(), 4);
        },

        test_initial_state {
            assert_eq!(db.count(), 3);
        },
    }
}

#[test]
fn test_nested_test_groups() {
    test_group! {
        math_tests {
            setup: {
                println!("Setting up math tests");
            },

            tests: {
                test_addition {
                    assert_eq!(2 + 2, 4);
                },

                test_multiplication {
                    assert_eq!(3 * 4, 12);
                },
            },

            teardown: {
                println!("Cleaning up math tests");
            }
        }
    }
}
```

### Starter Code

```rust
// TODO: Implement test_group! macro with setup/teardown
macro_rules! test_group {
    (
        $group_name:ident {
            setup: $setup:block,
            tests: {
                $(
                    $test_name:ident($fixture:ident) $test_body:block
                ),* $(,)?
            },
            teardown: $teardown:block
        }
    ) => {
        // TODO: Generate a module containing test functions
        // Each test runs setup, test body, then teardown
        mod $group_name {
            use super::*;

            $(
                #[test]
                fn $test_name() {
                    // TODO: Run setup to get fixture
                    let $fixture = $setup;

                    // TODO: Run test body
                    $test_body

                    // TODO: Run teardown
                    // Problem: $fixture is used in teardown but may be moved
                    // Solution: pass it explicitly or use different pattern
                    let _ = $fixture; // Use fixture to prevent unused warning
                    $teardown
                }
            )*
        }
    };
}

// TODO: Implement with_fixtures! for automatic fixture management
macro_rules! with_fixtures {
    (
        $fixture_name:ident => $fixture_init:expr,

        $(
            $test_name:ident $test_body:block
        ),* $(,)?
    ) => {
        // TODO: Generate tests where each gets fresh fixture
        $(
            #[test]
            fn $test_name() {
                let mut $fixture_name = $fixture_init;
                $test_body
            }
        )*
    };
}

// Alternative: fixture as parameter
macro_rules! with_fixture {
    (
        $fixture_name:ident: $fixture_type:ty = $fixture_init:expr;

        $(
            fn $test_name:ident($param:ident: $ptype:ty) $test_body:block
        )*
    ) => {
        // TODO: Generate tests with typed fixture parameter
        todo!("Implement typed fixture tests")
    };
}
```

**Implementation Hints:**
1. Setup block should return the fixture value: `let fixture = $setup;`
2. Each test gets fresh fixture (setup runs per test, not once for group)
3. Teardown block should have access to fixture (may need to restructure)
4. Use module (`mod $group_name`) to namespace test group
5. Consider `drop` for automatic cleanup instead of explicit teardown

---

## Milestone 4: Property-Based Testing Patterns

### Introduction

**Why Milestone 3 Isn't Enough**: Example-based tests only cover specific inputs. Property-based tests verify invariants across entire input domains. Testing `reverse(reverse(x)) == x` for all strings catches edge cases examples miss.

**The Improvement**: Generate tests that check mathematical properties across ranges of inputs. `property_test! { forall x in 0..100: reverse(reverse(x)) == x }` generates 100 test cases automatically.

**Optimization (Bug Finding)**: Property-based testing finds edge cases developers don't think of. Example tests verify "reverse([1,2,3]) == [3,2,1]". Property test finds "reverse([]) panics on empty vec" by trying all sizes including 0.

### Architecture

**New Macros:**
- `property_test!` - Generate tests for property over range
  - **Pattern**: `property_test! { forall x in range: property(x) }`
  - **Expands to**: Multiple test cases covering range
  - **Role**: Property verification

- `forall!` - Universal quantification over inputs
  - **Pattern**: `forall! { x in values, y in values => property }`
  - **Expands to**: Nested loops testing all combinations
  - **Role**: Exhaustive property checking

- `check_property!` - Assertion with property context
  - **Pattern**: `check_property!(condition, "property description")`
  - **Expands to**: Assert with context about what property failed
  - **Role**: Clear property test failures

### Checkpoint Tests

```rust
#[test]
fn test_property_based_testing() {
    fn reverse<T: Clone>(v: &[T]) -> Vec<T> {
        v.iter().rev().cloned().collect()
    }

    // Property: reversing twice returns original
    property_test! {
        forall v in [vec![1, 2, 3], vec![4, 5], vec![], vec![100]]:
        {
            let reversed_twice = reverse(&reverse(&v));
            assert_eq!(reversed_twice, v);
        }
    }
}

#[test]
fn test_property_over_range() {
    fn is_even(n: i32) -> bool {
        n % 2 == 0
    }

    property_test! {
        forall n in (0..100).step_by(2):
        {
            check_property!(is_even(n), "even numbers should pass is_even");
        }
    }
}

#[test]
fn test_commutative_property() {
    fn add(a: i32, b: i32) -> i32 { a + b }

    forall! {
        a in [1, 2, 3, 4, 5],
        b in [10, 20, 30]
        =>
        {
            // Commutative property: a + b == b + a
            assert_eq!(add(a, b), add(b, a));
        }
    }
}

#[test]
fn test_associative_property() {
    fn add(a: i32, b: i32) -> i32 { a + b }

    forall! {
        a in [1, 2],
        b in [3, 4],
        c in [5, 6]
        =>
        {
            // Associative property: (a + b) + c == a + (b + c)
            assert_eq!(add(add(a, b), c), add(a, add(b, c)));
        }
    }
}

#[test]
fn test_string_properties() {
    property_test! {
        forall s in ["", "a", "hello", "test123"]:
        {
            // Property: length of reversed string equals original
            let reversed: String = s.chars().rev().collect();
            assert_eq!(reversed.len(), s.len());
        }
    }
}
```

### Starter Code

```rust
// TODO: Implement property_test! macro
macro_rules! property_test {
    (
        forall $var:ident in [$($value:expr),* $(,)?]: $body:block
    ) => {
        // TODO: Generate a test for each value
        // Each test binds $var to one value and runs $body
        $(
            {
                let $var = $value;
                $body
            }
        )*
    };

    // Alternative pattern: range-based
    (
        forall $var:ident in $range:expr: $body:block
    ) => {
        {
            // TODO: Iterate over range and test property
            for $var in $range {
                $body
            }
        }
    };
}

// TODO: Implement forall! for multiple variables
macro_rules! forall {
    (
        $var1:ident in [$($val1:expr),* $(,)?],
        $var2:ident in [$($val2:expr),* $(,)?]
        => $body:block
    ) => {
        // TODO: Nested iteration over both sets of values
        // Generate Cartesian product of test cases
        $(
            $(
                {
                    let $var1 = $val1;
                    let $var2 = $val2;
                    $body
                }
            )*
        )*
    };

    // Three-variable version
    (
        $var1:ident in [$($val1:expr),* $(,)?],
        $var2:ident in [$($val2:expr),* $(,)?],
        $var3:ident in [$($val3:expr),* $(,)?]
        => $body:block
    ) => {
        // TODO: Triple-nested iteration
        todo!("Implement three-variable forall")
    };
}

// TODO: Implement check_property! for better error messages
macro_rules! check_property {
    ($condition:expr, $description:expr) => {
        {
            if !$condition {
                panic!(
                    "Property '{}' failed: {}",
                    $description,
                    stringify!($condition)
                );
            }
        }
    };

    ($condition:expr, $description:expr, $($arg:tt)*) => {
        {
            if !$condition {
                panic!(
                    "Property '{}' failed: {} - {}",
                    $description,
                    stringify!($condition),
                    format!($($arg)*)
                );
            }
        }
    };
}
```

**Implementation Hints:**
1. Property tests run inside single test function (not separate tests per value)
2. Use `for` loop for ranges: `for $var in $range { $body }`
3. Nested repetitions for multi-variable: `$( $( ... )* )*`
4. `check_property!` should include variable values in error message
5. Consider shrinking on failure (advanced: find minimal failing case)

---

## Milestone 5: Benchmark Generation and Performance Testing

### Introduction

**Why Milestone 4 Isn't Enough**: Performance regressions are bugs too. Need automated performance tests to catch slowdowns. Manually writing benchmarks for each function variant is tedious.

**The Improvement**: Generate benchmark suite from specification. Compare multiple implementations automatically. Track performance metrics across test runs.

**Optimization (Performance Tracking)**: Benchmarks as tests catch regressions in CI. "Optimized" function that's actually slower fails benchmark test. Comparing implementations side-by-side shows real performance differences, not guesses.

### Architecture

**New Macros:**
- `bench_suite!` - Generate benchmark functions
  - **Pattern**: `bench_suite! { name { cases: [...], measure: ... } }`
  - **Expands to**: Benchmark harness code
  - **Role**: Performance test generation

- `compare_impls!` - Benchmark multiple implementations
  - **Pattern**: `compare_impls! { impl1, impl2, impl3 over inputs }`
  - **Expands to**: Comparative benchmarks
  - **Role**: Implementation comparison

- `assert_performance!` - Performance assertions
  - **Pattern**: `assert_performance! { function takes_less_than 100ms }`
  - **Expands to**: Timed execution with assertion
  - **Role**: Performance regression testing

### Checkpoint Tests

```rust
#[test]
fn test_simple_benchmark() {
    fn fibonacci(n: u32) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            n => fibonacci(n - 1) + fibonacci(n - 2),
        }
    }

    bench_suite! {
        fibonacci_bench {
            small: fibonacci(10),
            medium: fibonacci(20),
        }
    }
}

#[test]
fn test_compare_implementations() {
    fn sort_bubble(mut v: Vec<i32>) -> Vec<i32> {
        for i in 0..v.len() {
            for j in 0..v.len() - 1 - i {
                if v[j] > v[j + 1] {
                    v.swap(j, j + 1);
                }
            }
        }
        v
    }

    fn sort_builtin(mut v: Vec<i32>) -> Vec<i32> {
        v.sort();
        v
    }

    let test_data = vec![5, 2, 8, 1, 9];

    compare_impls! {
        sort_bubble,
        sort_builtin
        over test_data.clone()
    }
}

#[test]
fn test_performance_assertion() {
    fn fast_function() {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    assert_performance! {
        fast_function() takes_less_than 50
    };
}

#[test]
#[should_panic(expected = "exceeded time limit")]
fn test_performance_assertion_fails() {
    fn slow_function() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    assert_performance! {
        slow_function() takes_less_than 50
    };
}

#[test]
fn test_benchmark_with_iterations() {
    fn small_work(n: usize) -> usize {
        (0..n).sum()
    }

    bench_iterations! {
        small_work_bench {
            iterations: 1000,
            work: small_work(100),
        }
    }
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};

// TODO: Implement bench_suite! macro
macro_rules! bench_suite {
    (
        $suite_name:ident {
            $(
                $bench_name:ident: $expr:expr
            ),* $(,)?
        }
    ) => {
        // TODO: Generate benchmark functions
        // Measure time for each expression
        $(
            #[allow(dead_code)]
            fn $bench_name() -> Duration {
                let start = Instant::now();
                let _ = $expr;
                start.elapsed()
            }
        )*

        // TODO: Print benchmark results
        #[test]
        fn [<run_ $suite_name>]() {
            println!("Benchmark suite: {}", stringify!($suite_name));
            $(
                let duration = $bench_name();
                println!("  {}: {:?}", stringify!($bench_name), duration);
            )*
        }
    };
}

// TODO: Implement compare_impls! macro
macro_rules! compare_impls {
    (
        $($impl_fn:ident),+ $(,)?
        over $input:expr
    ) => {
        {
            println!("Comparing implementations:");
            $(
                {
                    let start = Instant::now();
                    let result = $impl_fn($input);
                    let duration = start.elapsed();
                    println!("  {}: {:?}", stringify!($impl_fn), duration);
                    let _ = result; // Use result to prevent optimization
                }
            )+
        }
    };
}

// TODO: Implement assert_performance! macro
macro_rules! assert_performance {
    ($expr:expr takes_less_than $millis:expr) => {
        {
            let start = Instant::now();
            $expr;
            let elapsed = start.elapsed();

            let limit = Duration::from_millis($millis);
            if elapsed > limit {
                panic!(
                    "Performance assertion failed: {} took {:?}, exceeded time limit of {:?}",
                    stringify!($expr),
                    elapsed,
                    limit
                );
            } else {
                println!(
                    "Performance OK: {} took {:?} (limit: {:?})",
                    stringify!($expr),
                    elapsed,
                    limit
                );
            }
        }
    };
}

// TODO: Implement bench_iterations! for statistical benchmarking
macro_rules! bench_iterations {
    (
        $bench_name:ident {
            iterations: $n:expr,
            work: $expr:expr,
        }
    ) => {
        #[test]
        fn $bench_name() {
            let mut times = Vec::new();

            for _ in 0..$n {
                let start = Instant::now();
                let _ = $expr;
                times.push(start.elapsed());
            }

            let total: Duration = times.iter().sum();
            let avg = total / $n as u32;
            let min = times.iter().min().unwrap();
            let max = times.iter().max().unwrap();

            println!("Benchmark {}: {} iterations", stringify!($bench_name), $n);
            println!("  Average: {:?}", avg);
            println!("  Min: {:?}", min);
            println!("  Max: {:?}", max);
        }
    };
}
```

**Implementation Hints:**
1. Use `std::time::Instant::now()` for timing
2. Store result of benchmarked expression to prevent dead code elimination
3. Run multiple iterations for stable measurements
4. Use `#[allow(dead_code)]` for generated benchmark functions
5. Consider warmup runs to account for JIT/cache effects

---

## Milestone 6: Test Report Generation and Custom Test Runner

### Introduction

**Why Milestone 5 Isn't Enough**: Test results need summarization—passed/failed/skipped counts, timing, coverage. Default test output minimal. Need custom reports: HTML, JSON, integration with CI systems.

**The Improvement**: Generate test metadata at compile time. Custom test runner collects results and formats reports. Export test results in multiple formats.

**Optimization (CI/CD Integration)**: Machine-readable test output (JSON) enables automated analysis. Track test timing trends to detect slowdowns. Generate badges, charts, historical data—all from test metadata.

### Architecture

**New Macros:**
- `test_with_metadata!` - Tests with annotations
  - **Pattern**: `test_with_metadata! { tags: [...], timeout: N, test: ... }`
  - **Expands to**: Test with metadata for reporting
  - **Role**: Rich test information

- `test_report!` - Generate report from test runs
  - **Pattern**: `test_report! { format: json, output: "report.json" }`
  - **Expands to**: Report generation code
  - **Role**: Test result export

- `custom_test_runner!` - Define custom test harness
  - **Pattern**: `custom_test_runner! { before_all: ..., after_each: ... }`
  - **Expands to**: Test runner with hooks
  - **Role**: Test execution control

### Checkpoint Tests

```rust
#[test]
fn test_with_metadata_simple() {
    test_with_metadata! {
        name: critical_test,
        tags: ["critical", "fast"],
        timeout: 1000,
        test: {
            assert_eq!(2 + 2, 4);
        }
    }
}

#[test]
fn test_with_metadata_slow() {
    test_with_metadata! {
        name: slow_test,
        tags: ["slow", "integration"],
        timeout: 5000,
        test: {
            std::thread::sleep(std::time::Duration::from_millis(100));
            assert!(true);
        }
    }
}

#[test]
fn test_conditional_execution() {
    test_with_metadata! {
        name: conditional_test,
        tags: ["conditional"],
        skip_if: std::env::var("SKIP_SLOW").is_ok(),
        test: {
            // Only runs if SKIP_SLOW env var not set
            assert!(true);
        }
    }
}

#[test]
fn test_retry_on_failure() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ATTEMPT: AtomicUsize = AtomicUsize::new(0);

    test_with_metadata! {
        name: flaky_test,
        tags: ["flaky"],
        retry: 3,
        test: {
            // Fails first 2 attempts, succeeds on 3rd
            let attempt = ATTEMPT.fetch_add(1, Ordering::SeqCst);
            assert!(attempt >= 2);
        }
    }
}
```

### Starter Code

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

// TODO: Test metadata structure
#[derive(Debug, Clone)]
struct TestMetadata {
    name: String,
    tags: Vec<String>,
    timeout_ms: Option<u64>,
    skip: bool,
    retries: u32,
}

// TODO: Test result structure
#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    duration: Duration,
    error: Option<String>,
}

// TODO: Implement test_with_metadata! macro
macro_rules! test_with_metadata {
    (
        name: $name:ident,
        tags: [$($tag:expr),* $(,)?],
        timeout: $timeout:expr,
        test: $body:block
    ) => {
        #[test]
        fn $name() {
            let metadata = TestMetadata {
                name: stringify!($name).to_string(),
                tags: vec![$($tag.to_string()),*],
                timeout_ms: Some($timeout),
                skip: false,
                retries: 0,
            };

            println!("Running test: {} {:?}", metadata.name, metadata.tags);

            let start = Instant::now();
            $body
            let duration = start.elapsed();

            if let Some(timeout_ms) = metadata.timeout_ms {
                let limit = Duration::from_millis(timeout_ms);
                if duration > limit {
                    panic!("Test exceeded timeout: {:?} > {:?}", duration, limit);
                }
            }

            println!("Test {} completed in {:?}", metadata.name, duration);
        }
    };

    // Pattern with skip_if condition
    (
        name: $name:ident,
        tags: [$($tag:expr),*],
        skip_if: $skip_condition:expr,
        test: $body:block
    ) => {
        #[test]
        fn $name() {
            if $skip_condition {
                println!("Skipping test: {}", stringify!($name));
                return;
            }

            $body
        }
    };

    // Pattern with retry
    (
        name: $name:ident,
        tags: [$($tag:expr),*],
        retry: $retries:expr,
        test: $body:block
    ) => {
        #[test]
        fn $name() {
            let mut attempts = 0;
            let max_retries = $retries;

            loop {
                attempts += 1;
                println!("Test attempt {}/{}", attempts, max_retries);

                let result = std::panic::catch_unwind(|| {
                    $body
                });

                if result.is_ok() || attempts >= max_retries {
                    result.unwrap();
                    break;
                }

                println!("Test failed, retrying...");
            }
        }
    };
}

// TODO: Implement test_report! macro
macro_rules! test_report {
    (
        format: json,
        output: $output:expr,
        results: $results:expr
    ) => {
        {
            // TODO: Generate JSON report from test results
            // For now, just print
            println!("Generating JSON report to: {}", $output);
            println!("Results: {:?}", $results);

            // In real implementation, write JSON to file
            // use serde_json::to_string_pretty(&$results)?;
        }
    };

    (
        format: html,
        output: $output:expr,
        results: $results:expr
    ) => {
        {
            // TODO: Generate HTML report
            println!("Generating HTML report to: {}", $output);
            // Would generate actual HTML with styled table of results
        }
    };
}

// TODO: Implement custom_test_runner! macro
macro_rules! custom_test_runner {
    (
        before_all: $setup:block,
        after_all: $teardown:block,
        tests: {
            $($test_name:ident: $test_body:block),* $(,)?
        }
    ) => {
        // TODO: Generate test runner with custom hooks
        #[test]
        fn run_custom_tests() {
            println!("=== Custom Test Runner ===");

            // Run before_all hook
            $setup

            let mut results = Vec::new();

            // Run each test
            $(
                {
                    println!("\nRunning: {}", stringify!($test_name));
                    let start = Instant::now();

                    let result = std::panic::catch_unwind(|| {
                        $test_body
                    });

                    let duration = start.elapsed();
                    let passed = result.is_ok();

                    results.push(TestResult {
                        name: stringify!($test_name).to_string(),
                        passed,
                        duration,
                        error: result.err().map(|_| "Test panicked".to_string()),
                    });

                    println!("{}: {} ({:?})",
                        stringify!($test_name),
                        if passed { "PASSED" } else { "FAILED" },
                        duration
                    );
                }
            )*

            // Run after_all hook
            $teardown

            // Print summary
            let passed = results.iter().filter(|r| r.passed).count();
            let failed = results.len() - passed;
            println!("\n=== Summary ===");
            println!("Passed: {}, Failed: {}", passed, failed);
        }
    };
}
```

**Implementation Hints:**
1. Use `std::panic::catch_unwind` to capture test panics
2. Collect test results in Vec for reporting
3. Use `serde_json` crate for JSON serialization (if available)
4. HTML reports can use templates or simple string formatting
5. Consider test discovery via inventory crate for automatic registration

---

## Complete Working Example

Here's a full implementation demonstrating all milestones:

```rust
use std::time::{Duration, Instant};

//======================
// Milestone 1: Basic test generation
//======================
macro_rules! test_suite {
    (
        $fn_name:ident {
            $(
                $test_name:ident: ($($input:expr),*) => $expected:expr
            ),* $(,)?
        }
    ) => {
        $(
            #[test]
            fn $test_name() {
                let result = $fn_name($($input),*);
                assert_eq!(result, $expected,
                    "Test {} failed: expected {:?}, got {:?}",
                    stringify!($test_name), $expected, result
                );
            }
        )*
    };
}

//======================
// Milestone 2: Parametric tests
//======================
macro_rules! parametric_test {
    (
        $test_group:ident {
            $(
                $test_name:ident: [$($param:expr),*]
            ),* $(,)?
        }
        |$($param_name:ident),*| $body:block
    ) => {
        mod $test_group {
            use super::*;

            $(
                #[test]
                fn $test_name() {
                    let ($($param_name),*) = ($($param),*);
                    $body
                }
            )*
        }
    };
}

//======================
// Milestone 3: Test groups with fixtures
//======================
macro_rules! with_fixtures {
    (
        $fixture_name:ident => $fixture_init:expr,

        $(
            $test_name:ident $test_body:block
        ),* $(,)?
    ) => {
        $(
            #[test]
            fn $test_name() {
                let mut $fixture_name = $fixture_init;
                $test_body
            }
        )*
    };
}

//======================
// Milestone 4: Property testing
//======================
macro_rules! property_test {
    (
        forall $var:ident in [$($value:expr),* $(,)?]: $body:block
    ) => {
        $(
            {
                let $var = $value;
                $body
            }
        )*
    };
}

macro_rules! check_property {
    ($condition:expr, $description:expr) => {
        {
            if !$condition {
                panic!(
                    "Property '{}' failed: {}",
                    $description,
                    stringify!($condition)
                );
            }
        }
    };
}

//======================
// Milestone 5: Benchmarking
//======================
macro_rules! assert_performance {
    ($expr:expr takes_less_than $millis:expr) => {
        {
            let start = Instant::now();
            $expr;
            let elapsed = start.elapsed();

            let limit = Duration::from_millis($millis);
            if elapsed > limit {
                panic!(
                    "Performance assertion failed: {} took {:?}, exceeded {:?}",
                    stringify!($expr),
                    elapsed,
                    limit
                );
            }
        }
    };
}

//======================
// Example tests using all features
//======================

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

fn reverse<T: Clone>(v: &[T]) -> Vec<T> {
    v.iter().rev().cloned().collect()
}

// Basic test suite
test_suite! {
    add {
        test_add_positive: (2, 3) => 5,
        test_add_negative: (-2, -3) => -5,
        test_add_zero: (0, 5) => 5,
        test_add_identity: (42, 0) => 42,
    }
}

// Parametric tests
parametric_test! {
    multiply_tests {
        test_2x3: [2, 3, 6],
        test_5x4: [5, 4, 20],
        test_0x10: [0, 10, 0],
    }
    |a, b, expected| {
        assert_eq!(multiply(a, b), expected);
    }
}

// Property-based tests
#[test]
fn test_reverse_properties() {
    property_test! {
        forall v in [vec![1, 2, 3], vec![4, 5], vec![], vec![100]]:
        {
            // Property 1: double reverse is identity
            let reversed_twice = reverse(&reverse(&v));
            check_property!(reversed_twice == v, "reverse(reverse(v)) == v");

            // Property 2: length preserved
            let reversed = reverse(&v);
            check_property!(reversed.len() == v.len(), "length preserved");
        }
    }
}

// Fixture-based tests
#[derive(Debug)]
struct TestDB {
    data: Vec<i32>,
}

impl TestDB {
    fn new() -> Self {
        TestDB { data: vec![1, 2, 3] }
    }

    fn insert(&mut self, value: i32) {
        self.data.push(value);
    }

    fn count(&self) -> usize {
        self.data.len()
    }
}

with_fixtures! {
    db => TestDB::new(),

    test_db_insert {
        db.insert(4);
        assert_eq!(db.count(), 4);
    },

    test_db_initial_state {
        assert_eq!(db.count(), 3);
    },

    test_db_multiple_inserts {
        db.insert(10);
        db.insert(20);
        assert_eq!(db.count(), 5);
    },
}

// Performance tests
#[test]
fn test_fast_operations() {
    assert_performance! {
        add(2, 3) takes_less_than 1
    };

    assert_performance! {
        {
            let v = vec![1, 2, 3, 4, 5];
            reverse(&v)
        } takes_less_than 10
    };
}

fn main() {
    println!("Run `cargo test` to execute all generated tests");
    println!("This framework generates:");
    println!("- Basic test suites with test_suite!");
    println!("- Parametric tests with parametric_test!");
    println!("- Property-based tests with property_test!");
    println!("- Fixture-based tests with with_fixtures!");
    println!("- Performance tests with assert_performance!");
}
```

This complete implementation demonstrates:
1. **Test case generation** - From compact specifications
2. **Parametric testing** - Same test, different data
3. **Fixture management** - Automatic setup/teardown
4. **Property-based testing** - Verify invariants
5. **Performance testing** - Catch regressions
6. **Custom assertions** - Better error messages

The framework generates hundreds of lines of test code from concise macro invocations—a production-ready foundation for comprehensive testing with minimal boilerplate.
