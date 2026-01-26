//! # Trait Design Patterns in Rust
//!
//! This crate demonstrates the five major trait design patterns:
//!
//! ## Pattern 1: Trait Inheritance and Bounds
//! - Supertraits (`trait A: B`)
//! - Multiple supertraits with `+`
//! - Trait bounds in generic functions
//! - Conditional implementations
//! - Advanced bound patterns (Builder, HRTB)
//!
//! ## Pattern 2: Associated Types vs Generics
//! - Generic type parameters
//! - Associated types
//! - When to use each
//! - Combining both approaches
//!
//! ## Pattern 3: Trait Objects and Dynamic Dispatch
//! - Static vs dynamic dispatch
//! - Heterogeneous collections
//! - Object safety rules
//! - Downcasting with Any
//! - Lifetime bounds on trait objects
//!
//! ## Pattern 4: Extension Traits
//! - Basic extension traits
//! - Blanket implementations
//! - Extending standard library types
//! - Conditional extensions
//!
//! ## Pattern 5: Sealed Traits
//! - Private supertrait pattern
//! - Dependency injection with traits
//!
//! Run examples with: `cargo run --example <name>`
