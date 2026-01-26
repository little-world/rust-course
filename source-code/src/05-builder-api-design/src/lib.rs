//! # Builder & API Design Patterns
//!
//! This crate contains examples for Chapter 5: Builder & API Design.
//!
//! ## Patterns Covered
//!
//! 1. **Builder Pattern Variations**
//!    - Consuming builders (most common)
//!    - Runtime validation with Result
//!    - Mutable/reusable builders
//!
//! 2. **Typestate Pattern**
//!    - Encoding state in the type system
//!    - Compile-time state machine validation
//!    - Typestate builders for required fields
//!
//! 3. **#[must_use] Attribute**
//!    - Preventing ignored return values
//!    - Ensuring builders are finalized
//!    - Guarding critical resources
//!
//! ## Running Examples
//!
//! ```bash
//! # Pattern 1: Builder Pattern Variations
//! cargo run --example p1_consuming_builder
//! cargo run --example p1_runtime_validation
//! cargo run --example p1_mutable_builder
//!
//! # Pattern 2: Typestate Pattern
//! cargo run --example p2_typestate_connection
//! cargo run --example p2_typestate_builder
//!
//! # Pattern 3: #[must_use]
//! cargo run --example p3_must_use
//! ```
