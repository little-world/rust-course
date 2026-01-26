// Appendix A: Standard Libraries Examples
// This crate demonstrates patterns from the Appendix A quick reference guide.

pub mod examples {
    //! # Appendix A: Standard Libraries Quick Reference
    //!
    //! This crate provides runnable examples for:
    //!
    //! ## Pattern 1: Type Conversions
    //! - From, Into, TryFrom, TryInto
    //! - AsRef, AsMut
    //! - Cow for zero-copy patterns
    //!
    //! ## Pattern 2: Common Trait Implementations
    //! - Debug, Clone, PartialEq, Eq, Hash
    //! - PartialOrd, Ord
    //! - Default, Display, Error
    //! - Iterator, IntoIterator
    //!
    //! ## Pattern 3: Iterator Combinators
    //! - Adapters: map, filter, filter_map, flat_map, take, skip
    //! - Combining: chain, zip, enumerate
    //! - Consumers: collect, sum, fold, reduce, find
    //!
    //! ## Pattern 4: Collections
    //! - Vec, VecDeque
    //! - HashMap, HashSet
    //! - BTreeMap, BTreeSet
    //! - BinaryHeap, LinkedList
    //!
    //! ## Pattern 5: String and Text Processing
    //! - String vs &str
    //! - Manipulation, searching, slicing
    //! - Parsing, formatting
    //! - Regular expressions (regex crate)
    //!
    //! ## Pattern 6: Option and Result
    //! - Option combinators
    //! - Result combinators
    //! - The ? operator
    //! - Custom error types with thiserror
    //!
    //! ## Pattern 7: Smart Pointers
    //! - Box<T> for heap allocation
    //! - Rc<T> for shared ownership
    //! - Arc<T> for thread-safe sharing
    //! - RefCell<T>, Cell<T> for interior mutability
    //! - Mutex<T>, RwLock<T> for thread-safe mutation
    //!
    //! Run individual examples with:
    //! ```bash
    //! cargo run --bin p1_conversions
    //! cargo run --bin p2_traits
    //! cargo run --bin p3_iterators
    //! cargo run --bin p4_collections
    //! cargo run --bin p5_strings
    //! cargo run --bin p6_option_result
    //! cargo run --bin p7_smart_pointers
    //! ```
}
