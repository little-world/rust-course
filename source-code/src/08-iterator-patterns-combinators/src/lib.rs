//! # Iterator Patterns & Combinators
//!
//! This crate contains examples for Chapter 8: Iterator Patterns & Combinators.
//!
//! ## Patterns Covered
//!
//! 1. **Custom Iterators and IntoIterator**
//!    - Basic custom iterators (Counter)
//!    - IntoIterator for custom collections (RingBuffer)
//!    - Infinite iterators (Fibonacci)
//!
//! 2. **Zero-Allocation Iteration**
//!    - Chaining adapters without intermediate collections
//!    - Sliding windows for moving averages
//!    - Custom reductions with fold
//!
//! 3. **Advanced Iterator Composition**
//!    - Complex filtering and transformation pipelines
//!    - Stateful iteration with from_fn
//!    - Interleaving iterators
//!    - Cartesian products
//!    - Group by key
//!    - Scan for cumulative operations
//!    - take_while/skip_while for splitting
//!    - Peekable for lookahead parsing
//!
//! 4a. **Streaming Algorithms**
//!    - Line-by-line file processing
//!    - Streaming average calculation
//!    - Top-K elements without sorting
//!    - Sliding window statistics
//!    - Streaming deduplication
//!    - Rate limiting iterator
//!    - Buffered batch processing
//!    - Streaming merge of sorted iterators
//!    - CSV parsing with streaming
//!    - Lazy transformation chains
//!
//! 4b. **Parallel Iteration with Rayon**
//!    - Basic parallel iteration
//!    - Parallel sort
//!    - Parallel chunked processing
//!    - Parallel file processing
//!    - Parallel find with early exit
//!    - Parallel word count
//!    - Parallel partition
//!    - Parallel matrix multiplication
//!    - Parallel bridge for sequential sources
//!    - Controlling parallelism with scope
//!    - Parallel map-reduce pattern
//!    - Parallel pipeline with multiple stages
//!    - Parallel join for independent computations
//!
//! ## Running Examples
//!
//! ```bash
//! # Pattern 1: Custom Iterators and IntoIterator
//! cargo run --example p1_counter
//! cargo run --example p1_ring_buffer
//! cargo run --example p1_fibonacci
//!
//! # Pattern 2: Zero-Allocation Iteration
//! cargo run --example p2_chaining
//! cargo run --example p2_windows
//! cargo run --example p2_fold
//!
//! # Pattern 3: Advanced Iterator Composition
//! cargo run --example p3_log_analysis
//! cargo run --example p3_chunking
//! cargo run --example p3_interleave
//! cargo run --example p3_cartesian_product
//! cargo run --example p3_group_by
//! cargo run --example p3_scan
//! cargo run --example p3_take_skip_while
//! cargo run --example p3_peekable
//!
//! # Pattern 4a: Streaming Algorithms
//! cargo run --example p4a_file_processing
//! cargo run --example p4a_streaming_average
//! cargo run --example p4a_top_k
//! cargo run --example p4a_sliding_window
//! cargo run --example p4a_deduplication
//! cargo run --example p4a_rate_limiting
//! cargo run --example p4a_batch_processing
//! cargo run --example p4a_merge_sorted
//! cargo run --example p4a_csv_parsing
//! cargo run --example p4a_lazy_transformation
//!
//! # Pattern 4b: Parallel Iteration with Rayon
//! cargo run --example p4b_parallel_basics
//! cargo run --example p4b_parallel_sort
//! cargo run --example p4b_parallel_chunks
//! cargo run --example p4b_parallel_files
//! cargo run --example p4b_parallel_find
//! cargo run --example p4b_parallel_word_count
//! cargo run --example p4b_parallel_partition
//! cargo run --example p4b_parallel_matrix
//! cargo run --example p4b_parallel_bridge
//! cargo run --example p4b_parallel_scope
//! cargo run --example p4b_parallel_map_reduce
//! cargo run --example p4b_parallel_pipeline
//! cargo run --example p4b_parallel_join
//! ```
