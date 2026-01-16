### Part I: Core Language Mechanics
- [memory-ownership-patterns](01-memory-ownership-patterns.md): Master ownership, borrowing, and lifetimes as the foundation for safe Rust.
- [struct-enum-patterns](02-struct-enum-patterns.md): Model complex data with expressive structs, enums, and pattern-centric APIs.
- **trait-design-patterns**: Compose reusable behavior through carefully scoped traits and idioms.
- **generics-polymorphism**: Use generics and trait bounds to build flexible, zero-cost abstractions.
- **builder-api-design**: Design fluent builder types that validate configuration before construction.
- **lifetime-patterns**: Reason about lifetime annotations to uphold reference safety without clutter.
- **functional-programming**: Adopt iterator, closure, and declarative styles for clearer logic.
- **pattern-matching-destructuring**: Leverage `match` and destructuring to unpack data cleanly.
- **iterator-patterns-combinators**: Chain iterator adapters and combinators for elegant data pipelines.
- **reference-binding**: Handle borrowing, references, and iterator lifetimes in tandem.
- **error-handling-architecture**: Structure `Result` flows, bubbling, and context-rich error types.
### Part II: Collections & Data Structures
- **vec-slice-manipulation**: Work efficiently with contiguous buffers via `Vec`, slices, and views.
- **string-processing**: Parse, transform, and construct UTF-8 strings without needless copies.
- **hashmap-hashset-patterns**: Choose and tune hash-based collections for fast lookups.
- **advanced-collections**: Employ specialized containers such as `BTreeMap` and `BinaryHeap`.
### Part III: Concurrency & Parallelism
- **threading-patterns**: Launch threads, share state, and coordinate via channels and locks.
- **async-runtime-patterns**: Drive async tasks atop executors while keeping latency predictable.
- **atomic-lock-free**: Write lock-free data paths using atomics and correct memory ordering.
- **parallel-algorithms**: Apply Rayon-style abstractions for scalable data-parallel workloads.
### Part IV: Smart Pointers & Memory
- **smart-pointer-patterns**: Employ `Box`, `Rc`, `Arc`, and custom pointers for ownership control.
- **unsafe-rust-patterns**: Encapsulate unsafe code responsibly with airtight invariants.
### Part V: I/O & Serialization
- **synchronous-io**: Build blocking I/O services with the standard library’s stream traits.
- **async-io-patterns**: Structure non-blocking I/O stacks using async/await primitives.
- **serialization-patterns**: Encode and decode data via Serde and custom formats.
### Part VI: Macros & Metaprogramming
- **declarative-macros**: Author `macro_rules!` DSLs that expand ergonomically.
- **procedural-macros**: Craft derive and attribute macros with `syn` and `quote`.
### Part VII: Systems Programming
- **ffi-c-interop**: Bridge Rust with C interfaces while honoring safety contracts.
- **network-programming**: Implement network clients and servers with std or Tokio.
- **database-patterns**: Integrate SQL/NoSQL backends through Diesel, SQLx, or lower-level APIs.
- **testing-benchmarking**: Build resilient test suites, benches, and property checks.
- **performance-optimization**: Profile, measure, and tune for predictable performance wins.
- **embedded-realtime-patterns**: Apply Rust in `no_std`, RTIC, and real-time control scenarios.
### Appendices
- **appendix-a-quick-reference**: Keep a handy cheat sheet of syntax, commands, and patterns.
- **appendix-b-design-patterns**: Browse a catalog of reusable Rust design templates.
- **appendix-c-anti-patterns**: Recognize and avoid common pitfalls and code smells.
