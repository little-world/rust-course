// Testing & Benchmarking Patterns Library
// This module re-exports the example modules for documentation purposes.

pub mod examples {
    // Pattern 1: Unit Test Patterns
    // See p1_unit_tests.rs for examples of:
    // - Basic #[test] attribute usage
    // - Assertion macros (assert_eq!, assert_ne!, assert!)
    // - Testing error cases with Result
    // - Testing panics with #[should_panic]
    // - Organizing tests with nested modules
    // - RAII-based setup/teardown
    // - Ignoring and filtering tests
    // - Testing private functions

    // Pattern 2: Property-Based Testing
    // See p2_proptest_basics.rs and p2_quickcheck.rs for examples of:
    // - proptest macro usage
    // - Custom generators with prop_compose!
    // - Testing invariants
    // - QuickCheck alternative syntax

    // Pattern 6: Mock and Stub Patterns
    // See p6_*.rs for examples of:
    // - Trait-based mocking
    // - Dependency injection
    // - Test doubles for I/O

    // Pattern 8: Criterion Benchmarking
    // See benches/*.rs for examples of:
    // - Basic benchmarks with black_box
    // - Comparing implementations
    // - Parameterized benchmarks
    // - Throughput measurement
}
