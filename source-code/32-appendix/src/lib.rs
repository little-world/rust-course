// Appendix B: Design Patterns Catalog
// This crate demonstrates design patterns from the Appendix B catalog.

pub mod examples {
    //! # Appendix B: Design Patterns Quick Reference
    //!
    //! This crate provides runnable examples for:
    //!
    //! ## Pattern 1: Creational Patterns
    //! - Builder Pattern (fluent API, typestate)
    //! - Factory Pattern (trait objects, enums)
    //! - Singleton Pattern (OnceLock)
    //! - Prototype Pattern (Clone)
    //!
    //! ## Pattern 2: Structural Patterns
    //! - Adapter Pattern (trait objects, generics)
    //! - Decorator Pattern (trait objects, generics)
    //! - Facade Pattern
    //! - Newtype Pattern
    //!
    //! ## Pattern 3: Behavioral Patterns
    //! - Strategy Pattern (trait objects, generics, closures)
    //! - Observer Pattern (trait objects, channels)
    //! - Command Pattern (undo/redo)
    //! - Iterator Pattern (custom iterators, IntoIterator)
    //!
    //! ## Pattern 4: Concurrency Patterns
    //! - Thread Pool Pattern
    //! - Producer-Consumer Pattern
    //! - Fork-Join Pattern
    //! - Actor Pattern
    //! - Async/Await Pattern
    //!
    //! Run individual examples with:
    //! ```bash
    //! cargo run --bin p1_creational
    //! cargo run --bin p2_structural
    //! cargo run --bin p3_behavioral
    //! cargo run --bin p4_concurrency
    //! ```
}
