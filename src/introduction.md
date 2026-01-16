### Part I: Core Language Mechanics
- [memory-ownership-patterns](01-memory-ownership-patterns.md): Master ownership, borrowing, and lifetimes as the foundation for safe Rust.
- [struct-enum-patterns](02-struct-enum-patterns.md): Model complex data with expressive structs, enums, and pattern-centric APIs.
- [trait-design-patterns](03-trait-design-patterns.md): Compose reusable behavior through carefully scoped traits and idioms.
- [generics-polymorphism](04-generics-polymorphism.md): Use generics and trait bounds to build flexible, zero-cost abstractions.
- [builder-api-design](05-builder-api-design.md): Design fluent builder types that validate configuration before construction.
- [lifetime-patterns](06-lifetime-patterns.md): Reason about lifetime annotations to uphold reference safety without clutter.
- [functional-programming](04-functional-programming-patterns.md): Adopt iterator, closure, and declarative styles for clearer logic.
- [pattern-matching-destructuring](07-pattern-matching-destructuring.md): Leverage `match` and destructuring to unpack data cleanly.
- [iterator-patterns-combinators](08-iterator-patterns-combinators.md): Chain iterator adapters and combinators for elegant data pipelines.
- [reference-binding](18-reference-iterator-patterns.md): Handle borrowing, references, and iterator lifetimes in tandem.
- [error-handling-architecture](09-error-handling-architecture.md): Structure `Result` flows, bubbling, and context-rich error types.
### Part II: Collections & Data Structures
- [vec-slice-manipulation](10-vec-slice-manipulation.md): Work efficiently with contiguous buffers via `Vec`, slices, and views.
- [string-processing](11-string-processing.md): Parse, transform, and construct UTF-8 strings without needless copies.
- [hashmap-hashset-patterns](12-hashmap-hashset-patterns.md): Choose and tune hash-based collections for fast lookups.
- [advanced-collections](13-advanced-collections.md): Employ specialized containers such as `BTreeMap` and `BinaryHeap`.
### Part III: Concurrency & Parallelism
- [threading-patterns](14-threading-patterns.md): Launch threads, share state, and coordinate via channels and locks.
- [async-runtime-patterns](15-async-runtime-patterns.md): Drive async tasks atop executors while keeping latency predictable.
- [atomic-lock-free](16-atomic-lock-free.md): Write lock-free data paths using atomics and correct memory ordering.
- [parallel-algorithms](17-parallel-algorithms.md): Apply Rayon-style abstractions for scalable data-parallel workloads.
### Part IV: Smart Pointers & Memory
- [smart-pointer-patterns](18-smart-pointer-patterns.md): Employ `Box`, `Rc`, `Arc`, and custom pointers for ownership control.
- [unsafe-rust-patterns](19-unsafe-rust-patterns.md): Encapsulate unsafe code responsibly with airtight invariants.
### Part V: I/O & Serialization
- [synchronous-io](20-synchronous-io.md): Build blocking I/O services with the standard library’s stream traits.
- [async-io-patterns](21-async-io-patterns.md): Structure non-blocking I/O stacks using async/await primitives.
- [serialization-patterns](22-serialization-patterns.md): Encode and decode data via Serde and custom formats.
### Part VI: Macros & Metaprogramming
- [declarative-macros](23-declarative-macros.md): Author `macro_rules!` DSLs that expand ergonomically.
- [procedural-macros](24-procedural-macros.md): Craft derive and attribute macros with `syn` and `quote`.
### Part VII: Systems Programming
- [ffi-c-interop](25-ffi-c-interop.md): Bridge Rust with C interfaces while honoring safety contracts.
- [network-programming](26-network-programming.md): Implement network clients and servers with std or Tokio.
- [database-patterns](27-database-patterns.md): Integrate SQL/NoSQL backends through Diesel, SQLx, or lower-level APIs.
- [testing-benchmarking](28-testing-benchmarking.md): Build resilient test suites, benches, and property checks.
- [performance-optimization](29-performance-optimization.md): Profile, measure, and tune for predictable performance wins.
- [embedded-realtime-patterns](30-embedded-realtime-patterns.md): Apply Rust in `no_std`, RTIC, and real-time control scenarios.
### Appendices
- [appendix-a-quick-reference](31-appendix-a-quick-reference.md): Keep a handy cheat sheet of syntax, commands, and patterns.
- [appendix-b-design-patterns](32-appendix-b-design-patterns.md): Browse a catalog of reusable Rust design templates.
- [appendix-c-anti-patterns](33-appendix-c-anti-patterns.md): Recognize and avoid common pitfalls and code smells.
