//! # Lifetime Patterns
//!
//! This crate contains examples for Chapter 6: Lifetime Patterns.
//!
//! ## Patterns Covered
//!
//! 1. **Named Lifetimes and Elision**
//!    - Basic lifetime annotations
//!    - Elision rules
//!    - The 'static lifetime
//!
//! 2. **Lifetime Bounds**
//!    - T: 'a bounds
//!    - Where clauses for complex bounds
//!
//! 3. **Higher-Ranked Trait Bounds (HRTBs)**
//!    - for<'a> Fn(&'a str) syntax
//!    - Lifetime-polymorphic closures
//!
//! 4. **Self-Referential Structs and Pin**
//!    - Safe alternatives using indices
//!    - Pin for truly self-referential types
//!    - Restructuring designs
//!
//! 5. **Variance and Subtyping**
//!    - Covariant, invariant, contravariant types
//!    - PhantomData for variance control
//!
//! 6. **Advanced Lifetime Patterns**
//!    - Streaming iterators with GAT
//!    - Closures with lifetimes
//!    - Anonymous lifetimes
//!
//! ## Running Examples
//!
//! ```bash
//! # Pattern 1: Named Lifetimes and Elision
//! cargo run --example p1_basic_lifetimes
//! cargo run --example p1_static_lifetime
//!
//! # Pattern 2: Lifetime Bounds
//! cargo run --example p2_lifetime_bounds
//!
//! # Pattern 3: Higher-Ranked Trait Bounds
//! cargo run --example p3_hrtb
//!
//! # Pattern 4: Self-Referential Structs
//! cargo run --example p4_indices
//! cargo run --example p4_pin_self_ref
//! cargo run --example p4_restructure
//!
//! # Pattern 5: Variance and Subtyping
//! cargo run --example p5_variance
//! cargo run --example p5_phantomdata_variance
//!
//! # Pattern 6: Advanced Lifetime Patterns
//! cargo run --example p6_streaming_iterator
//! cargo run --example p6_advanced_lifetimes
//! ```
