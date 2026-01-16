  1. [lru-cache](01-lru-cache.md) – Implement an LRU cache that keeps recently
     accessed items and evicts the least-used entry when the capacity limit
     is reached.
  2. [expression-parser-arena](02-expression-parser-arena.md) – Build an
     arithmetic-expression parser that uses arena(bump) allocation so parsed
     trees share storage.
  3. [string-interning](03-string-interning.md) – Create a string interning
     system that stores each unique string once and hands out reusable
     references.
  4. [generic-data-structures](04-advanced-generics.md) – Assemble a const-
     generic collection toolkit whose sizes live in the type system and incur
     zero runtime cost.
  5. [memory-pool](05-memory-pool.md) – Write a pool allocator that pre-allocates
     a big block and serves fixed-size slots quickly to hot code paths.
  6. [reference-count](06-reference-count.md) – Re-implement reference counting
     to learn how Rc/Arc-like structures manage shared ownership.
  7. [safe-configuration](06-safe-configuration.md) – Define configuration
     newtypes and builders that prevent invalid server configs at compile time.
  8. [streaming-iterator-hrtb](06-streaming-iterator-hrtb.md) – Implement a
     streaming iterator that yields borrowed data via higher-ranked trait bounds
     and GATs.
  9. [validated-wrappers](03-reference-project-1-validated-wrappers.md) - Implement validated wrapper types (Email, Url, NonEmptyString, PositiveInt)
  10. [zero-copy-protpocol](06-zero-copy-protocol-parser.md) – Parse text by borrowing slices
      from the input so tokens remain valid without allocations.
  11. [parser-combinator](04-functional-project1-parser-combinator.md)  - functional parser combinator library
  12. [middleware-pipeline](04-functional-project2-middleware-pipeline.md)   - functional middleware pipeline
  13. [json-parser-validator](07-json-parser-validator.md) – Parse JSON into an
     AST and validate against schemas using pattern matching.
  14. [network-packet-inspector](07-network-packet-inspector.md) – Inspect multi-
     layer network packets, enforce firewall rules, and detect malicious
     patterns.
  15. [order-strate-machine](07-order-strate-machine.md) – Model an order
     workflow with enums so illegal state transitions can’t compile.
  16. [regex-engine-pattern-matching](07-regex-engine-pattern-matching.md) –
     Build a regex engine that parses patterns, supports captures/alternation,
     and backtracks correctly.
  17. [csv-parser](08-csv-parser.md) – Stream massive CSV files using iterators
     that handle quoting and keep memory usage flat.
  18. [pagination-iterator](08-pagination-iterator.md) – Expose paginated REST
     endpoints as lazy iterators that fetch pages on demand.
  19. [plugins](08-plugins.md) – Load runtime-selected plugins safely so user
     code can extend the host app.
  20. [config-validator-error-handling](09-config-validator-error-handling.md) –
     Parse TOML/JSON configs and emit precise schema errors for bad inputs.
  21. [parser-combinator](09-parser-combinator.md) – Build parser combinators
     with associated types so small parsers compose ergonomically.
  22. [web-scraper-retry-breaker](09-web-scraper-retry-breaker.md) – Crawl many
     URLs concurrently with retries, rate limits, and circuit breakers.
  23. [generics-queue](10-generics-queue.md) – Implement a generic priority queue
     that accepts any Ord payload.
  24. [csv-processor](10-vec-project-1-csv-processor.md) – Process multi-gig CSVs
     by chunking, transforming, validating, and batching inserts efficiently.
  25. [timeseries](10-vec-project-2-timeseries.md) – Analyze time-series streams
     by computing sliding-window statistics in O(n).
  26. [binary-search](10-vec-project-3-binary-search.md) – Provide binary-search
     helpers that power fast queries over sorted data.
  27. [log-parser](11-string-project-1-log-parser.md) – Parse huge log files into
     structured events with zero-copy string handling.
  28. [text-editor-gap-buffer](11-string-project-2-text-editor-gap-buffer.md)
     – Implement a gap-buffer editor core that makes insert/delete near the
     cursor cheap.
  29. [boyer-moore-search](11-string-project-3-boyer-moore-search.md) – Apply
     Boyer–Moore substring search to find patterns efficiently in large texts.
  30. [analytics-engine](12-hashmap-project-1-analytics-engine.md) – Maintain
     real-time analytics over streaming events with HashMaps for aggregations.
  31. [custom-hash-functions](12-hashmap-project-2-custom-hash-functions.md) –
     Create custom hashers for spatial/geographic data to keep maps performant
     and correct.
  32. [cache-alternative-maps](12-hashmap-project-3-cache-alternative-maps.md)
     – Compare alternative map types to build a multi-tier cache with varied
     access patterns.
  33. [autocomplete](13-advanced-collections-autocomplete.md) – Build a trie-
     based autocomplete engine that returns suggestions fast.
  34. [queue-crossbeam](13-advanced-collections-queue-crossbeam.md) – Use
     crossbeam to implement a lock-free MPMC queue for parallel workloads.
  35. [scheduling](13-advanced-collections-scheduling.md) – Prioritize and
     schedule jobs via binary heaps that respect deadlines.
  36. [image-processor](14-threading-image-processor.md) – Fan out CPU-heavy
     image filters onto a thread pool for throughput.
  37. [producer-consumer](14-threading-producer-consumer.md) – Chain processing
     stages via channels to demonstrate producer/consumer backpressure.
  38. [shared-counter](14-threading-shared-counter.md) – Coordinate shared
     counters across threads using Arc + Mutex.
  39. [webscraper](15-async-runtime-project1-webscraper.md) – Scrape the web
     asynchronously while respecting rate limits and handling failures.
  40. [event-stream](15-async-runtime-project2-event-stream.md) – Consume
     multiple event streams, transform them, and fan results out in real time.
  41. [task-scheduler](15-async-runtime-project3-task-scheduler.md) – Manage
     async tasks with priorities, deadlines, and retry policies.
  42. [image-processing](15-async-runtime-project4-image-processing.md) –
     Build an async pipeline that watches directories and processes images
     concurrently.
  43. [metrics](16-atomic-project1-metrics.md) – Record metrics from many threads
     using lock-free atomics instead of mutexes.
  44. [treiber-stack](16-atomic-project2-treiber-stack.md) – Implement a lock-
     free Treiber stack using CAS loops and hazard pointers.
  45. [ring-buffer](16-atomic-project3-ring-buffer.md) – Implement a wait-free
     ring buffer for producer/consumer communication.
  46. [sorting](17-parallel-project1-sorting.md) – Build a fork-join parallel
     sorting library that scales across cores.
  47. [graph](17-parallel-project2-graph.md) – Traverse graphs in parallel while
     balancing irregular workloads.
  48. [map-reduce](17-parallel-project3-map-reduce.md) – Implement a mini
     MapReduce engine that distributes log analysis jobs.
  49. [kafka](17-parallel-project4-kafka.md) – Simulate a Kafka-style event
     pipeline that handles millions of messages per second.
  50. [tokenizer](17-tokenizer-project.md) – Build character-, word-, and BPE-
     tokenizers for ML pipelines.
  51. [matrix-multiplication](18-matrix-multiplication-project.md) – Optimize
     matrix multiplication from naïve O(n³) up to tiled/SIMD/GPU variants.
  52. [rc-dom](18-smart-pointer-project1-rc-dom.md) – Model a DOM tree with Rc/
     Weak so nodes share ownership and can navigate parents.
  53. [object-pool](18-smart-pointer-project2-object-pool.md) – Reuse expensive
     objects via a smart-pointer-backed pool instead of reallocating.
  54. [cow](18-smart-pointer-project3-cow.md) – Design copy-on-write structures
     that share data until a write occurs.
  55. [cpu-optimization-convolution](19-cpu-optimization-project-convolution.md)
     – Optimize 2D convolutions with cache blocking, SIMD, and threading.
  56. [build-system-pipeline](20-build-system-pipeline.md) – Orchestrate compiler
     stages, capture logs, and re-run only changed targets.
  57. [file-sync-rsync](20-file-sync-rsync.md) – Synchronize directories rsync-
     style by hashing chunks and copying only deltas.
  58. [version-control-git](20-version-control-git.md) – Implement Git primitives
    ](blobs, trees, commits, refs) to track history and sync repos.
  59. [async-http-proxy](21-async-http-proxy.md) – Build an async HTTP proxy with
     connection pooling, backpressure, timeouts, and health checks.
  60. [chat-server-broadcast](21-chat-server-broadcast.md) – Broadcast chat
     messages to many clients while handling backpressure and disconnects.
  61. [ini](22-serialization-project1-ini.md) – Parse and emit INI files with
     serde-friendly structures and validation.
  62. [performance](22-serialization-project2-performance.md) – Benchmark JSON,
     Bincode, and MessagePack to compare size vs speed.
  63. [migration](22-serialization-project3-migration.md) – Migrate old
     serialized config formats to the latest schema transparently.
  64. [query-builder](23-declarative-macros-project1-query-builder.md) – Generate
     SQL query builders with declarative macros for a type-safe DSL.
  65. [test-framework](23-declarative-macros-project2-test-framework.md) – Build
     a macro-driven test harness that expands compact specs into full suites.
  66. [config-dsl](23-declarative-macros-project3-config-dsl.md) – Create a
     declarative config DSL macro that produces typed structs at compile time.
  67. [orchestration](24-procedural-macros-project2-orchestration.md) – Use
     procedural macros to define deployment/orchestration manifests in Rust.
  68. [spring-framework-macros](24-spring-framework-macros.md) – Mimic Spring
     Boot annotations with procedural macros for DI and routing.
  69. [ffi-c1](25-ffi-c-project1.md) – Wrap C’s qsort/bsearch in safe Rust so
     arbitrary types can interop.
  70. [ffi-python2](25-ffi-python-project2.md) – Expose a Rust text-processing
     library as a Python module via PyO3.
  71. [rust-assembly-optimization](25-rust-assembly-optimization.md) – Study
     Rust’s assembly output and inject inline asm to tighten hotspots.
  72. [chat-server](26-network-project-1-chat-server.md) – Grow a simple echo
     server into a full chat service with multiple protocols.
  73. [udp-game-server](26-network-project-3-udp-game-server.md) – Build a UDP-
     based multiplayer server with service discovery and reliability.
  74. [distributed-kv-store](26-network-project-4-distributed-kv-store.md) –
     Create a distributed KV store with replication, persistence, and leader
     election.
  75. [collaborative-editor](26-network-project-6-collaborative-editor.md) – Sync
     shared documents via CRDT-like techniques for concurrent editing.
  76. [task-queue](27-database-project-1-task-queue.md) – Persist a durable task
     queue where workers lease and process jobs safely.
  77. [blog-api](27-database-project-2-blog-api.md) – Expose CRUD blog APIs
     backed by SQLx with compile-time query checking.
  78. [multi-tenant-saas](27-database-project-3-multi-tenant-saas.md) – Partition
     tenant data and enforce tenant-aware queries in a SaaS schema.
  79. [coverage-analyzer](28-testing-project-1-coverage-analyzer.md) – Instrument
     Rust code to report line/branch/function coverage from tests.
  80. [mutation-testing](28-testing-project-2-mutation-testing.md) – Flip AST
     nodes deliberately and ensure the test suite fails appropriately.
  81. [property-test-generator](28-testing-project-3-property-test-generator.md)
     – Auto-generate property tests using proptest based on function signatures.
  82. [profiler](29-performance-project-1-profiler.md) – Profile Rust programs by
     sampling CPU time, memory allocations, and call stacks.
  83. [data-pipeline](29-performance-project-2-data-pipeline.md) – Build a
     streaming data pipeline tuned for cache locality and SIMD throughput.
  84. [cache-structures](29-performance-project-3-cache-structures.md) – Compare
     cache-friendly data layouts to maximize CPU efficiency.
  85. [embedded-hal-sensor-driver](33-embedded-project-1-hal-sensor-driver.md) - Hardware Abstraction Layer using traits
  86. [embedded-realtime-logger](33-embedded-project-2-realtime-logger.md) - Zero-copy DMA transfers for ADC, UART, and SPI
  87. [embedded-interrupt-coordinator](33-embedded-project-3-interrupt-coordinator.md) - Priority-based interrupt management