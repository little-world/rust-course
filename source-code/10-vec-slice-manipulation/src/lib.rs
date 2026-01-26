//! Vec & Slice Manipulation Patterns
//!
//! This crate demonstrates patterns for efficient vector and slice operations in Rust.
//!
//! ## Patterns Covered
//!
//! 1. **Capacity Management** - Pre-allocation, reserve, reuse buffers
//! 2. **Slice Algorithms** - Binary search, sorting, partitioning
//! 3. **Chunking and Windowing** - Batch processing, moving averages
//! 4. **Zero-Copy Slicing** - Parsing without allocation
//! 5. **SIMD Operations** - Vectorized processing
//! 6. **Advanced Slice Patterns** - Drain, retain, circular buffers
//!
//! ## Running Examples
//!
//! ```bash
//! cargo run --example p1_preallocate
//! cargo run --example p2_binary_search
//! cargo run --example p3_chunks
//! cargo run --example p4_zero_copy_parsing
//! cargo run --example p5_simd_friendly
//! cargo run --example p6_sliding_window_max
//! ```
