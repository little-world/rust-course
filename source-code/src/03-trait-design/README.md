Pattern 1: Trait Inheritance and Bounds (5 examples)
- p1_supertraits.rs - Supertrait relationships
- p1_multiple_supertraits.rs - Multiple supertraits with +
- p1_trait_bounds.rs - Trait bounds in generic functions
- p1_conditional_impl.rs - Conditional implementations
- p1_bound_patterns.rs - Builder pattern, HRTB

Pattern 2: Associated Types vs Generics (5 examples)
- p2_generic_parser.rs - Generic type parameters
- p2_associated_parser.rs - Associated types
- p2_ergonomics.rs - Comparison of both approaches
- p2_combining.rs - Combining generics and associated types
- p2_associated_bounds.rs - Associated types with bounds (Graph)

Pattern 3: Trait Objects and Dynamic Dispatch (5 examples)
- p3_static_dispatch.rs - Monomorphization
- p3_dynamic_dispatch.rs - Dynamic dispatch, heterogeneous collections
- p3_heterogeneous.rs - Drawable shapes collection
- p3_downcasting.rs - Downcasting with Any
- p3_lifetime_bounds.rs - Trait objects with lifetimes

Pattern 4: Extension Traits (5 examples)
- p4_basic_extension.rs - Basic extension trait
- p4_iterator_extension.rs - Blanket iterator extensions
- p4_result_extension.rs - Error handling extensions
- p4_string_extension.rs - String extensions
- p4_conditional_extension.rs - Extensions based on capabilities

Pattern 5: Sealed Traits (2 examples)
- p5_sealed_trait.rs - Private supertrait pattern
- p5_dependency_injection.rs - DI with traits

Run any example with: cargo run --example <name>   