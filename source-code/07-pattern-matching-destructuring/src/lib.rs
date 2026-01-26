//! # Pattern Matching & Destructuring
//!
//! This crate contains examples for Chapter 7: Pattern Matching & Destructuring.
//!
//! ## Patterns Covered
//!
//! 1. **Advanced Match Patterns**
//!    - Range matching for numeric classification
//!    - Guards for complex conditions
//!    - @ bindings to capture and test
//!    - Nested destructuring
//!
//! 2. **Exhaustiveness and Match Ergonomics**
//!    - Exhaustive matching for safety
//!    - Match ergonomics (automatic reference handling)
//!    - The #[non_exhaustive] attribute
//!
//! 3. **if let, while let, and let-else**
//!    - if let and if let chains
//!    - let-else for early returns
//!    - while let for iteration
//!
//! 4. **State Machines and Enum-Driven Architecture**
//!    - Command pattern with enums
//!    - Event sourcing with enums
//!
//! ## Running Examples
//!
//! ```bash
//! # Pattern 1: Advanced Match Patterns
//! cargo run --example p1_range_matching
//! cargo run --example p1_guards
//! cargo run --example p1_at_bindings
//! cargo run --example p1_nested_destructuring
//!
//! # Pattern 2: Exhaustiveness and Match Ergonomics
//! cargo run --example p2_exhaustiveness
//! cargo run --example p2_match_ergonomics
//! cargo run --example p2_non_exhaustive
//!
//! # Pattern 3: if let, while let, and let-else
//! cargo run --example p3_if_let
//! cargo run --example p3_let_else
//! cargo run --example p3_while_let
//!
//! # Pattern 4: State Machines and Enum-Driven Architecture
//! cargo run --example p4_command_pattern
//! cargo run --example p4_event_sourcing
//! ```
