//! # Generics & Polymorphism in Rust
//!
//! This crate demonstrates the eight major patterns for generics and polymorphism:
//!
//! ## Pattern 1: Type-Safe Generic Functions
//! - Basic generic functions with type inference
//! - Trait bounds for capability requirements
//! - Multiple bounds with `+` and `where` clauses
//! - Turbofish syntax for explicit types
//!
//! ## Pattern 2: Generic Structs and Enums
//! - Basic generic structs (Point, Pair)
//! - Multiple type parameters
//! - Generic enums (BinaryTree)
//! - Wrapper types with transformation
//! - Specialized implementations
//!
//! ## Pattern 3: Trait Bounds and Constraints
//! - Single and multiple trait bounds
//! - Bounds on associated types
//! - Lifetime bounds (`T: 'a`, `T: 'static`)
//! - Conditional method implementations
//! - Sized and ?Sized bounds
//! - impl Trait in arguments and returns
//!
//! ## Pattern 4: Associated Types vs Generic Parameters
//! - Associated types (one impl per type)
//! - Generic parameters (multiple impls per type)
//! - Generic Associated Types (GAT)
//! - Type families
//!
//! ## Pattern 5: Blanket Implementations
//! - Blanket impl for trait bounds
//! - Extension traits
//! - Into from From pattern
//! - Orphan rules
//!
//! ## Pattern 6: Phantom Types and Type-Level State
//! - Protocol state machines
//! - Units of measure
//! - Builder with required fields
//! - FFI ownership markers
//!
//! ## Pattern 7: Higher-Ranked Trait Bounds (HRTBs)
//! - Basic HRTB for closures with references
//! - HRTB vs regular lifetime parameters
//! - Callback storage
//! - When to use HRTB
//!
//! ## Pattern 8: Const Generics
//! - Basic const generic arrays
//! - Compile-time size validation
//! - Matrix with dimension checking
//! - Ring buffers
//! - Protocol frames
//!
//! Run examples with: `cargo run --example <name>`
