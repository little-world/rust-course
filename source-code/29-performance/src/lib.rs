// Performance Optimization Patterns Library
// This module provides documentation for the example patterns.

pub mod examples {
    //! # Performance Optimization Patterns
    //!
    //! This crate demonstrates performance optimization techniques in Rust:
    //!
    //! ## Pattern 1: Profiling Strategies
    //! - CPU profiling with perf/flamegraph
    //! - Memory profiling with dhat/valgrind
    //! - Criterion benchmarks
    //!
    //! ## Pattern 2: Allocation Reduction
    //! - Reusing buffers
    //! - SmallVec for stack allocation
    //! - Cow for clone-on-write
    //! - Arena allocation
    //! - String interning
    //!
    //! ## Pattern 3: Cache-Friendly Data Structures
    //! - Array-of-Structs vs Struct-of-Arrays
    //! - Cache line awareness
    //! - Sequential vs random access
    //! - Linked list vs vector
    //!
    //! ## Pattern 4: Zero-Cost Abstractions
    //! - Branch prediction
    //! - Branchless code
    //! - Pattern matching optimization
    //!
    //! ## Pattern 5: Compiler Optimizations
    //! - Const functions
    //! - Const generics
    //! - Lookup tables
    //! - Static assertions
    //! - Type-level computation
}
