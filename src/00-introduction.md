
## **Rust Cookbook: Production Patterns & Algorithms**

For Experienced Programmers

### **Part I: Core Language Mechanics**

[**01. Memory & Ownership Patterns**](01-memory-ownership-patterns.md)
- Zero-copy patterns (Cow, borrowing strategies)
- Interior mutability (Cell, RefCell, Mutex, RwLock)
- Custom allocators and arena patterns
- Memory layout optimization (repr, alignment)
- RAII patterns and drop guards

[**02. Type System Deep Dive**](02-type-system-deep-dive.md)
- Newtype pattern and type safety
- Phantom types and zero-cost abstractions
- GATs (Generic Associated Types)
- Type-level programming
- Trait object optimization

[**03. Error Handling Architecture**](03-error-handling-architecture.md)
- Error type design patterns
- Error propagation strategies (?, thiserror, anyhow)
- Custom error types with context
- Recoverable vs unrecovable errors
- Error handling in async contexts

[**04. Pattern Matching & Destructuring**](04-pattern-matching-destructuring.md)
- Advanced match patterns (guards, bindings, ranges)
- Exhaustiveness and match ergonomics
- If-let chains and while-let
- Pattern matching for state machines
- Enum-driven architecture

[**05. Iterator Patterns & Combinators**](05-iterator-patterns-combinators.md)
- Custom iterators and IntoIterator
- Zero-allocation iteration
- Iterator adapters composition
- Streaming algorithms
- Parallel iteration (rayon patterns)

### **Part II: Collections & Data Structures**

[**06. Vec & Slice Manipulation**](06-vec-slice-manipulation.md)
- Capacity management and amortization
- Slice algorithms (searching, sorting, partitioning)
- Chunking and windowing patterns
- Zero-copy slicing
- SIMD operations

[**07. String Processing**](07-string-processing.md)
- Type overview (String, &str, Cow, OsString, Path)
- Zero-copy string operations
- UTF-8 handling and validation
- Parsing state machines
- Rope and gap buffer structures

[**08. HashMap & HashSet Patterns**](08-hashmap-hashset-patterns.md)
- Entry API patterns
- Custom hash functions
- Capacity and load factor optimization
- Alternative maps (BTreeMap, FxHashMap)
- Concurrent maps (DashMap)

[**09. Advanced Collections**](09-advanced-collections.md)
- VecDeque and ring buffers
- BinaryHeap and priority queues
- Graph representations
- Trie and radix tree structures
- Lock-free data structures

### **Part III: Concurrency & Parallelism**

[**10. Threading Patterns**](10-threading-patterns.md)
- Thread spawn and join patterns
- Thread pools and work stealing
- Message passing (channels)
- Shared state with Arc/Mutex
- Barrier and Condvar patterns

[**11. Async Runtime Patterns**](11-async-runtime-patterns.md)
- Future composition
- Stream processing
- async/await patterns
- Select and timeout patterns
- Runtime comparison (tokio, async-std)

[**12. Atomic Operations & Lock-Free**](12-atomic-operations-lock-free.md)
- Memory ordering semantics
- Compare-and-swap patterns
- Lock-free queues and stacks
- Hazard pointers
- Seqlock pattern

[**13. Parallel Algorithms**](13-parallel-algorithms.md)
- Rayon patterns (par_iter, par_bridge)
- Work partitioning strategies
- Parallel reduce and fold
- Pipeline parallelism
- SIMD parallelism

### **Part IV: Smart Pointers & Memory**

[**14. Smart Pointer Patterns**](14-smart-pointer-patterns.md)
- Box, Rc, Arc usage patterns
- Weak references and cycles
- Custom smart pointers
- Intrusive data structures
- Reference counting optimization

[**15. Unsafe Rust Patterns**](15-unsafe-rust-patterns.md)
- Raw pointer manipulation
- FFI patterns and C interop
- Uninitialized memory handling
- Transmute and type punning
- Writing safe abstractions over unsafe

### **Part V: I/O & Serialization**

[**16. Synchronous I/O**](16-synchronous-io.md)
- File operations and buffering
- Standard streams (stdin/stdout/stderr)
- Memory-mapped I/O
- Directory traversal
- Process spawning and piping

[**17. Async I/O Patterns**](17-async-io-patterns.md)
- Tokio file and network I/O
- Buffered async streams
- Backpressure handling
- Connection pooling
- Timeout and cancellation

[**18. Serialization Patterns**](18-serialization-patterns.md)
- Serde patterns (derive, custom serializers)
- Zero-copy deserialization
- Schema evolution
- Binary vs text formats
- Streaming serialization

### **Part VI: Macros & Metaprogramming**

[**19. Declarative Macros**](19-declarative-macros.md)
- Macro patterns and repetition
- Hygiene and scoping
- DSL construction
- Code generation patterns
- Macro debugging

[**20. Procedural Macros**](20-procedural-macros.md)
- Derive macros
- Attribute macros
- Function-like macros
- Token stream manipulation
- Macro helper crates (syn, quote)

### **Part VII: Systems Programming**

[**21. FFI & C Interop**](21-ffi-c-interop.md)
- C ABI compatibility
- String conversions (CString, OsString)
- Callback patterns
- Error handling across FFI
- bindgen patterns

[**22. Network Programming**](22-network-programming.md)
- TCP server/client patterns
- UDP patterns
- HTTP client (reqwest)
- HTTP server (axum, actix-web)
- WebSocket patterns

[**23. Database Patterns**](23-database-patterns.md)
- Connection pooling (r2d2, deadpool)
- Query builders (diesel, sqlx)
- Transaction patterns
- Migration strategies
- ORM vs raw SQL

[**24. Testing & Benchmarking**](24-testing-benchmarking.md)
- Unit test patterns
- Property-based testing (proptest, quickcheck)
- Mock and stub patterns
- Integration testing
- Criterion benchmarking

### **Part VIII: Advanced Topics**

[**25. Trait Design Patterns**](25-trait-design-patterns.md)
- Trait inheritance and bounds
- Associated types vs generics
- Trait objects and dynamic dispatch
- Extension traits
- Blanket implementations

[**26. Struct & Enum Patterns**](26-struct-enum-patterns.md)
- Struct design patterns (tuple, unit, named fields)
- Enum-driven architecture
- Newtype and wrapper patterns
- Zero-Sized Types and Markers
- Struct update syntax and partial moves
- Enum variants with data payloads
- Pattern matching for enums
- Visitor pattern with enums
- Type-safe state machines with enums


[**27. Builder & API Design**](27-builder-api-design.md)
- Builder pattern variations
- Typestate pattern
- Fluent interfaces
- Extension traits for libraries
- Sealed trait pattern

[**28. Lifetime Patterns**](28-lifetime-patterns.md)
- Named lifetimes and elision rules
- Lifetime bounds and where clauses
- Higher-ranked trait bounds (for<'a>)
- Self-referential structures
- Variance and subtyping


[**29. Performance Optimization**](29-performance-optimization.md)
- Profiling strategies (perf, flamegraph)
- Allocation reduction techniques
- Cache-friendly data structures
- Branch prediction optimization
- Compile-time evaluation

### **Appendices**

[**A. Quick Reference**](00-quick-reference.md)
- Type conversion cheatsheet
- Common trait implementations
- Iterator combinators reference
- Cargo commands

[**B. Design Pattern Catalog**](00-design-patterns.md)
- Creational patterns
- Structural patterns
- Behavioral patterns
- Concurrency patterns

[**C. Anti-Patterns**](00-anti-patterns.md)
- Common pitfalls
- Performance anti-patterns
- Safety anti-patterns
- API design mistakes
