# Appendix A: Standard Libraries
**Sections:**

- [Type Conversion Cheatsheet](#type-conversion-cheatsheet)
- [Common Trait Implementations](#common-trait-implementations)
- [Iterator Combinators Reference](#iterator-combinators-reference)
- [Collections: Vec, HashMap, HashSet, and More](#collections-vec-hashmap-hashset-and-more)
- [String and Text Processing](#string-and-text-processing)
- [Cargo Commands Reference](#cargo-commands-reference)



As you develop production Rust code, certain patterns recur constantly: converting between types, implementing common traits, chaining iterators, and managing projects with Cargo. While the main chapters explore these concepts in depth, this appendix provides a condensed reference for the patterns you'll reach for daily.

Think of this as your muscle memory guide. The type conversions you implement most frequently. The trait derivations that make your structs ergonomic. The iterator combinators that transform data pipelines. The Cargo commands that streamline your workflow. Each section distills practical knowledge into actionable patterns, organized for quick lookup when you know what you need but need a syntax reminder.

This reference is designed for experienced programmers who understand the underlying concepts and need fast access to implementation patterns. If you're new to a concept, follow the cross-references to the detailed chapters where we explore the why and when alongside the how.

---

## Type Conversion Cheatsheet

Type conversions in Rust are explicit by design, preventing the subtle bugs that plague languages with implicit coercion. The standard library provides a hierarchy of conversion traits, each with different guarantees about safety, cost, and ownership.

### The Conversion Hierarchy

Understanding when to use each conversion trait is crucial for API design. Here's the mental model:

**Infallible conversions** (always succeed) use `From` and `Into`. These represent transformations where the source type's entire value space maps cleanly into the target type. A `u8` can always become a `u32` because every 8-bit value fits in 32 bits. A `&str` can always become a `String` through allocation.

**Fallible conversions** (might fail) use `TryFrom` and `TryInto`. These handle narrowing conversions where the source type's value space exceeds the target. Converting `u32` to `u8` might fail for values above 255. Parsing strings into numbers might fail for invalid input.

**Borrow conversions** (zero-cost) use `AsRef` and `AsMut`. These provide cheap reference conversions, enabling generic functions to accept multiple concrete types through a shared borrowed view.
```rust
// The conversion trait landscape
use std::convert::{From, Into, TryFrom, TryInto, AsRef, AsMut};

//===============================================
// Pattern 1: Implementing From (Into comes free)
//===============================================
struct UserId(u64);
struct DatabaseId(u64);

impl From<DatabaseId> for UserId {
    fn from(db_id: DatabaseId) -> Self {
        UserId(db_id.0)  // Infallible conversion
    }
}

// Now both directions work
let db_id = DatabaseId(42);
let user_id: UserId = db_id.into();           // Into is automatic
let user_id2 = UserId::from(DatabaseId(43));  // From is explicit

//==========================================================
// Pattern 2: Implementing TryFrom for validated conversions
//==========================================================
use std::num::TryFromIntError;

struct Port(u16);

impl TryFrom<u32> for Port {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value <= 65535 {
            Ok(Port(value as u16))
        } else {
            Err("Port number exceeds maximum (65535)")
        }
    }
}

// Use with ? for error propagation
fn parse_port(input: u32) -> Result<Port, &'static str> {
    let port = Port::try_from(input)?;
    Ok(port)
}

//===================================
// Pattern 3: AsRef for flexible APIs
//===================================
fn log_message<S: AsRef<str>>(msg: S) {
    println!("{}", msg.as_ref());
}

// Works with many string types
log_message("literal");           // &str
log_message(String::from("owned")); // String
log_message("owned".to_string());  // String

//======================================
// Pattern 4: AsMut for generic mutation
//======================================
fn clear_buffer<T: AsMut<[u8]>>(buffer: &mut T) {
    for byte in buffer.as_mut() {
        *byte = 0;
    }
}

let mut vec_buffer = vec![1, 2, 3];
let mut array_buffer = [4, 5, 6];
clear_buffer(&mut vec_buffer);
clear_buffer(&mut array_buffer);
```

### Common Conversion Patterns

**String Conversions** are the most frequent in real code. The ecosystem has converged on patterns that balance efficiency with ergonomics:

```rust
// &str → String (allocation required)
let owned: String = "borrowed".to_string();     // Uses Display
let owned: String = "borrowed".to_owned();      // Uses ToOwned
let owned: String = String::from("borrowed");   // Uses From
let owned: String = "borrowed".into();          // Uses Into (needs type hint)

// String → &str (cheap borrow)
let s = String::from("hello");
let borrowed: &str = &s;           // Deref coercion
let borrowed: &str = s.as_str();   // Explicit

// &str → Cow<str> (zero-copy when possible)
use std::borrow::Cow;

fn maybe_uppercase(s: &str, should_uppercase: bool) -> Cow<str> {
    if should_uppercase {
        Cow::Owned(s.to_uppercase())  // Allocates
    } else {
        Cow::Borrowed(s)              // Zero-copy
    }
}
```

**Numeric Conversions** require care because Rust prevents silent overflow. The patterns reflect whether you're widening (always safe) or narrowing (potentially lossy):

```rust
// Widening: use From/Into (infallible)
let x: u8 = 255;
let y: u32 = x.into();           // Always succeeds
let z: u32 = u32::from(x);       // Equivalent

// Narrowing: use TryFrom/TryInto (fallible)
let big: u32 = 300;
let small: Result<u8, _> = big.try_into();  // Err for values > 255

// Lossy: use as for explicit truncation
let truncated = big as u8;       // Compiles but truncates to 44

// Floating point conversions (always explicit)
let precise: f64 = 3.14159;
let rough = precise as f32;      // Loses precision
let integer = precise as i32;    // Truncates to 3
```

**Collection Conversions** leverage `FromIterator` and `IntoIterator` to transform between collection types:

```rust
use std::collections::{HashSet, HashMap, BTreeSet};

// Vec → HashSet (deduplication)
let numbers = vec![1, 2, 2, 3, 3, 3];
let unique: HashSet<_> = numbers.into_iter().collect();

// Vec<(K, V)> → HashMap
let pairs = vec![("a", 1), ("b", 2)];
let map: HashMap<_, _> = pairs.into_iter().collect();

// HashSet → Vec (ordering unspecified)
let set: HashSet<_> = [3, 1, 2].iter().cloned().collect();
let vec: Vec<_> = set.into_iter().collect();

// HashSet → BTreeSet (ordered)
let hash_set: HashSet<_> = [3, 1, 2].iter().cloned().collect();
let tree_set: BTreeSet<_> = hash_set.into_iter().collect();

// Array → Vec (ownership transfer)
let array = [1, 2, 3, 4, 5];
let vec = array.to_vec();        // Clone
let vec = Vec::from(array);      // Also clone
```

### Quick Reference Table

| From          | To            | Trait                 | Notes                                    |
|---------------|---------------|-----------------------|------------------------------------------|
| `&str`        | `String`      | `Into/From`           | Allocates                                |
| `String`      | `&str`        | `AsRef/Deref`         | Zero-cost borrow                         |
| `&[T]`        | `Vec<T>`      | `Into/From`           | Clones elements                          |
| `Vec<T>`      | `&[T]`        | `AsRef/Deref`         | Zero-cost borrow                         |
| `[T; N]`      | `Vec<T>`      | `From`                | Moves or clones depending on T           |
| `u8`          | `u32`         | `Into/From`           | Widening (safe)                          |
| `u32`         | `u8`          | `TryInto/TryFrom`     | Narrowing (may fail)                     |
| `i32`         | `f64`         | `Into/From`           | Exact representation                     |
| `f64`         | `i32`         | `as` cast             | Truncates, may overflow                  |
| `Option<T>`   | `Result<T,E>` | `ok_or/ok_or_else`    | Provide error for None case              |
| `Result<T,E>` | `Option<T>`   | `ok()`                | Discards error                           |
| `&Path`       | `&OsStr`      | `AsRef`               | Zero-cost                                |
| `PathBuf`     | `OsString`    | `Into`                | Ownership transfer                       |

---

## Common Trait Implementations

Rust's trait system enables code reuse through composition rather than inheritance. The standard library defines dozens of traits, but a handful appear repeatedly in production code. Understanding which traits to derive, which to implement manually, and how they interact is essential for ergonomic API design.

### The Derivable Core

Most custom types should derive these traits unless they have specific reasons not to:

```rust
// The standard derive bundle for data types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct User {
    id: u64,
    username: String,
    email: String,
}

// Debug: Required for debugging, logging, and error messages
// Clone: Enables explicit duplication
// PartialEq/Eq: Enables == comparisons and use in HashSet/HashMap
// Hash: Enables use as HashMap keys
```

**When to skip derives:**

- **Skip `Clone`** for large structs where cloning is expensive and you want to make ownership transfers explicit
- **Skip `PartialEq`** for types where equality is ambiguous (floating point, time ranges)
- **Skip `Hash`** for types that shouldn't be used as keys (mutable state, large blobs)
- **Skip `Eq`** for types containing `f32`/`f64` (NaN breaks reflexivity)

```rust
// Example: Skipping Clone for large, move-only types
#[derive(Debug)]
struct LargeBuffer {
    data: Vec<u8>,  // Intentionally move-only
}

// Example: Custom PartialEq for case-insensitive comparison
#[derive(Debug, Clone)]
struct CaseInsensitiveString(String);

impl PartialEq for CaseInsensitiveString {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}
```

### Ordering Traits: PartialOrd and Ord

Ordering enables sorting, binary search, and range operations. The distinction between `PartialOrd` and `Ord` reflects whether all values are comparable:

```rust
use std::cmp::Ordering;

// Ord: Total ordering (all values comparable)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Priority {
    level: u8,      // Compared first (due to field order)
    timestamp: u64, // Tiebreaker
}

// Use in sorted collections
use std::collections::BTreeSet;
let mut tasks = BTreeSet::new();
tasks.insert(Priority { level: 1, timestamp: 100 });
tasks.insert(Priority { level: 2, timestamp: 50 });
// Automatically sorted by priority, then timestamp

// Custom Ord for reverse ordering
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReverseScore(u32);

impl Ord for ReverseScore {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the natural ordering
        other.0.cmp(&self.0)
    }
}

impl PartialOrd for ReverseScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
```

**PartialOrd without Ord** is necessary for types with incomparable values:

```rust
// FloatWrapper can't implement Ord because NaN isn't comparable
#[derive(Debug, Clone, PartialEq)]
struct FloatWrapper(f64);

impl PartialOrd for FloatWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Returns None for NaN comparisons
        self.0.partial_cmp(&other.0)
    }
}
```

### Default: Zero-Cost Initialization

The `Default` trait provides canonical "zero" values, enabling concise initialization and builder patterns:

```rust
// Derive Default when all fields implement Default
#[derive(Debug, Default)]
struct Config {
    timeout_ms: u64,     // Defaults to 0
    retries: u32,        // Defaults to 0
    verbose: bool,       // Defaults to false
}

// Custom Default for better defaults
#[derive(Debug)]
struct Connection {
    host: String,
    port: u16,
    timeout_ms: u64,
}

impl Default for Connection {
    fn default() -> Self {
        Connection {
            host: "localhost".to_string(),
            port: 8080,
            timeout_ms: 5000,
        }
    }
}

// Use with struct update syntax
let conn = Connection {
    host: "api.example.com".to_string(),
    ..Default::default()  // Fill remaining fields
};

// Use with Option::unwrap_or_default
fn get_config(maybe_config: Option<Config>) -> Config {
    maybe_config.unwrap_or_default()
}
```

### Display and Debug: Human-Readable Output

`Debug` is for developers; `Display` is for users. The distinction guides how you present your types:

```rust
use std::fmt;

#[derive(Debug)]  // Auto-derived for development
struct Timestamp {
    unix_seconds: i64,
}

// Manual Display for user-facing output
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Timestamp({})", self.unix_seconds)
    }
}

// Using both
let ts = Timestamp { unix_seconds: 1609459200 };
println!("{:?}", ts);  // Debug: Timestamp { unix_seconds: 1609459200 }
println!("{}", ts);    // Display: Timestamp(1609459200)

// Pretty-printing with {:#?}
#[derive(Debug)]
struct Nested {
    users: Vec<String>,
    config: Config,
}
// {:#?} formats with indentation for nested structures
```

### Error: Making Errors First-Class

Types implementing `Error` integrate with Rust's error handling ecosystem, enabling `?` propagation and error context:

```rust
use std::error::Error;
use std::fmt;

// Minimal error type
#[derive(Debug)]
enum ApiError {
    NetworkFailure(String),
    InvalidResponse,
    Unauthorized,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::NetworkFailure(msg) => write!(f, "Network error: {}", msg),
            ApiError::InvalidResponse => write!(f, "Invalid response from server"),
            ApiError::Unauthorized => write!(f, "Authentication required"),
        }
    }
}

impl Error for ApiError {}

// Use in Result types
fn fetch_data() -> Result<String, ApiError> {
    Err(ApiError::NetworkFailure("Connection timeout".to_string()))
}

// Chain with ? operator
fn process() -> Result<(), Box<dyn Error>> {
    let data = fetch_data()?;  // Automatically converts
    Ok(())
}
```

**Using thiserror for ergonomic error types:**

```rust
// With thiserror crate (recommended for libraries)
use thiserror::Error;

#[derive(Error, Debug)]
enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error at line {line}: {msg}")]
    Parse { line: usize, msg: String },

    #[error("Invalid format")]
    InvalidFormat,
}
// Derives Display, Error, and From conversions automatically
```

### Iterator: Making Types Traversable

Implementing `Iterator` allows your types to work with for loops and iterator combinators:

```rust
// Simple iterator over a range
struct CountDown {
    count: u32,
}

impl Iterator for CountDown {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            let current = self.count;
            self.count -= 1;
            Some(current)
        } else {
            None
        }
    }
}

// Use with for loops
for n in CountDown { count: 5 } {
    println!("{}", n);  // 5, 4, 3, 2, 1
}

// Use with combinators
let countdown = CountDown { count: 5 };
let sum: u32 = countdown.filter(|&n| n % 2 == 0).sum();  // 4 + 2 = 6
```

**Implementing IntoIterator for ergonomic iteration:**

```rust
struct Playlist {
    songs: Vec<String>,
}

// Implement IntoIterator to enable for loops directly
impl IntoIterator for Playlist {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}
// Now works directly in for loops
let playlist = Playlist {
    songs: vec!["Song 1".to_string(), "Song 2".to_string()],
};

for song in playlist {  // No need for .songs.into_iter()
    println!("{}", song);
}
```

### Quick Reference Table

| Trait           | Purpose                       | Derive? | When to Implement Manually                    |
|-----------------|-------------------------------|---------|-----------------------------------------------|
| `Debug`         | Developer debugging           | Yes     | Rarely; only for security-sensitive types     |
| `Clone`         | Explicit duplication          | Yes     | Skip for move-only types                      |
| `Copy`          | Implicit duplication          | Yes     | Only for small, bitwise-copyable types        |
| `PartialEq`     | Equality comparison           | Yes     | Custom equality logic (case-insensitive, etc) |
| `Eq`            | Reflexive equality            | Yes     | Skip for types with NaN (f32/f64)             |
| `PartialOrd`    | Partial ordering              | Yes     | Custom comparison logic                       |
| `Ord`           | Total ordering                | Yes     | Reverse ordering, multi-field priority        |
| `Hash`          | Hash computation              | Yes     | Custom hash logic, skip for unhashable fields |
| `Default`       | Zero/empty value              | Yes     | Better defaults than all-zeros                |
| `Display`       | User-facing output            | No      | Always manual                                 |
| `Error`         | Error type integration        | No      | Always manual (or use thiserror)              |
| `Iterator`      | Sequential traversal          | No      | Always manual                                 |
| `IntoIterator`  | Enable for loops              | No      | Collection-like types                         |
| `From`/`Into`   | Type conversion               | No      | Always manual                                 |
| `Deref`         | Smart pointer behavior        | No      | Wrapper types needing transparent access      |
| `Drop`          | Custom cleanup                | No      | Resource management (files, locks, etc)       |

---

## Iterator Combinators Reference

Iterators are Rust's primary abstraction for sequential processing. They're lazy, composable, and often compile down to the same machine code as hand-written loops. Mastering iterator combinators transforms verbose imperative code into concise, declarative pipelines.

The key insight: iterators are **adapters and consumers**. Adapters transform iterators into new iterators (lazy). Consumers process iterators and return concrete values (eager). Chaining adapters builds up computation; calling a consumer executes it.

### Creating Iterators

Every collection can produce iterators in three flavors, each with different ownership semantics:

```rust
// Usage: iter() borrows, iter_mut() borrows mutably, into_iter() consumes
let data = vec![1, 2, 3];

// iter() - borrows elements immutably
for &item in data.iter() {
    println!("{}", item);  // item is &i32, pattern &item extracts i32
}
// data is still valid

// iter_mut() - borrows elements mutably
let mut data = vec![1, 2, 3];
for item in data.iter_mut() {
    *item *= 2;  // item is &mut i32
}
// data is still valid, now [2, 4, 6]

// into_iter() - takes ownership, consumes collection
for item in data.into_iter() {
    println!("{}", item);  // item is i32
}
// data is now invalid (moved)
```

**Manual iterator creation:**

```rust
// Range iterators
(0..10)           // 0 to 9
(0..=10)          // 0 to 10 (inclusive)
(0..).take(100)   // Infinite iterator, take first 100

// From functions
std::iter::once(42)                    // Single element
std::iter::repeat(7).take(5)           // [7, 7, 7, 7, 7]
std::iter::repeat_with(|| rand::random()) // Computed on each iteration
std::iter::empty::<i32>()              // Empty iterator

// From existing values
std::iter::from_fn(|| Some(42))        // Custom generation logic
```

### Adapter Combinators: Transforming Iterators

Adapters are lazy—they don't compute anything until consumed. This enables efficient chaining without intermediate allocations.

#### Mapping: Transforming Elements

```rust
// map: Transform each element
let numbers = vec![1, 2, 3];
let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();
// [2, 4, 6]

// map with complex transformations
let users = vec!["Alice", "Bob"];
let greetings: Vec<_> = users.iter()
    .map(|name| format!("Hello, {}!", name))
    .collect();
// ["Hello, Alice!", "Hello, Bob!"]

// filter_map: Map and filter in one pass
let inputs = vec!["42", "abc", "100"];
let numbers: Vec<i32> = inputs.iter()
    .filter_map(|s| s.parse().ok())  // Parse, keep only successes
    .collect();
// [42, 100]

// flat_map: Map and flatten
let words = vec!["hello", "world"];
let chars: Vec<_> = words.iter()
    .flat_map(|word| word.chars())
    .collect();
// ['h', 'e', 'l', 'l', 'o', 'w', 'o', 'r', 'l', 'd']
```

#### Filtering: Selecting Elements

```rust
// filter: Keep elements matching predicate
let numbers = vec![1, 2, 3, 4, 5, 6];
let evens: Vec<_> = numbers.iter()
    .filter(|&&x| x % 2 == 0)
    .copied()  // Convert &i32 to i32
    .collect();
// [2, 4, 6]

// take: First N elements
let first_three: Vec<_> = (1..=100).take(3).collect();
// [1, 2, 3]

// take_while: Elements until predicate fails
let less_than_five: Vec<_> = (1..=10)
    .take_while(|&x| x < 5)
    .collect();
// [1, 2, 3, 4]

// skip: Skip first N elements
let skip_first: Vec<_> = vec![1, 2, 3, 4, 5].into_iter().skip(2).collect();
// [3, 4, 5]

// skip_while: Skip until predicate fails
let skip_small: Vec<_> = vec![1, 2, 3, 4, 5].into_iter()
    .skip_while(|&x| x < 3)
    .collect();
// [3, 4, 5]
```

#### Combining: Multiple Iterators

```rust
// chain: Concatenate iterators
let a = vec![1, 2];
let b = vec![3, 4];
let combined: Vec<_> = a.iter().chain(b.iter()).copied().collect();
// [1, 2, 3, 4]

// zip: Pair elements from two iterators
let names = vec!["Alice", "Bob"];
let ages = vec![30, 25];
let people: Vec<_> = names.iter().zip(ages.iter()).collect();
// [("Alice", 30), ("Bob", 25)]

// Stops at shortest iterator
let short = vec![1, 2];
let long = vec![10, 20, 30, 40];
let pairs: Vec<_> = short.iter().zip(long.iter()).collect();
// [(1, 10), (2, 20)]

// enumerate: Add indices
let letters = vec!['a', 'b', 'c'];
let indexed: Vec<_> = letters.iter().enumerate().collect();
// [(0, 'a'), (1, 'b'), (2, 'c')]
```

#### Inspection: Observing Elements

```rust
// inspect: Peek at elements without consuming
let sum: i32 = (1..=5)
    .inspect(|x| println!("Processing {}", x))
    .map(|x| x * 2)
    .inspect(|x| println!("Doubled to {}", x))
    .sum();
// Prints each step, then returns 30
```

### Consumer Combinators: Producing Values

Consumers are eager—they process the entire iterator and return a result.

#### Collecting: Building Collections

```rust
use std::collections::{HashMap, HashSet, BTreeSet};

// collect into Vec
let vec: Vec<i32> = (1..=5).collect();

// collect into HashSet (deduplicates)
let set: HashSet<_> = vec![1, 2, 2, 3].into_iter().collect();
// {1, 2, 3}

// collect into HashMap from tuples
let map: HashMap<_, _> = vec![("a", 1), ("b", 2)].into_iter().collect();

// collect into String
let chars = vec!['h', 'e', 'l', 'l', 'o'];
let word: String = chars.into_iter().collect();
// "hello"

// partition: Split into two collections //======================================
let numbers = vec![1, 2, 3, 4, 5];
let (evens, odds): (Vec<_>, Vec<_>) = numbers.into_iter()
    .partition(|&x| x % 2 == 0);
// evens: [2, 4], odds: [1, 3, 5]
```

#### Searching: Finding Elements

```rust
// find: First element matching predicate
let numbers = vec![1, 2, 3, 4];
let first_even = numbers.iter().find(|&&x| x % 2 == 0);
// Some(&2)

// position: Index of first match
let pos = numbers.iter().position(|&x| x == 3);
// Some(2)

// any: Check if any element matches
let has_even = numbers.iter().any(|&x| x % 2 == 0);
// true

// all: Check if all elements match
let all_positive = numbers.iter().all(|&x| x > 0);
// true

// nth: Get element at index
let third = numbers.iter().nth(2);
// Some(&3)

// last: Get last element
let last = numbers.iter().last();
// Some(&4)
```

#### Aggregating: Reducing to Single Values

```rust
// sum: Add all elements
let total: i32 = (1..=10).sum();
// 55

// product: Multiply all elements
let factorial: i32 = (1..=5).product();
// 120

// fold: Custom accumulation
let sum = (1..=5).fold(0, |acc, x| acc + x);
// 15

// fold for non-numeric types
let sentence = vec!["Hello", "world"];
let joined = sentence.into_iter().fold(String::new(), |mut acc, word| {
    if !acc.is_empty() {
        acc.push(' ');
    }
    acc.push_str(word);
    acc
});
// "Hello world"

// reduce: Like fold but uses first element as initial value
let max = vec![3, 1, 4, 1, 5].into_iter().reduce(|a, b| a.max(b));
// Some(5)

// max/min: Find extremes
let max = vec![3, 1, 4].into_iter().max();
// Some(4)

let min = vec![3, 1, 4].into_iter().min();
// Some(1)

// max_by/min_by: Custom comparison
let words = vec!["short", "longer", "longest"];
let longest = words.iter().max_by_key(|word| word.len());
// Some("longest")
```

#### Counting and Testing

```rust
// count: Number of elements
let count = (1..=100).filter(|x| x % 2 == 0).count();
// 50

// for_each: Side effects for each element
(1..=5).for_each(|x| println!("{}", x));
// Prints 1 through 5
```

### Advanced Patterns: Real-World Pipelines

Iterator combinators shine in data processing pipelines where you transform, filter, and aggregate data in a single pass:

```rust
// Example: Process log lines
let log_lines = vec![
    "ERROR: Database connection failed",
    "INFO: Server started",
    "ERROR: Null pointer exception",
    "WARN: High memory usage",
];

let error_count = log_lines.iter()
    .filter(|line| line.starts_with("ERROR"))
    .count();
// 2

// Example: Transform and collect user data
struct User {
    name: String,
    age: u32,
    active: bool,
}

let users = vec![
    User { name: "Alice".to_string(), age: 30, active: true },
    User { name: "Bob".to_string(), age: 25, active: false },
    User { name: "Charlie".to_string(), age: 35, active: true },
];

let active_names: Vec<String> = users.into_iter()
    .filter(|user| user.active)
    .map(|user| user.name)
    .collect();
// ["Alice", "Charlie"]

// Example: Nested iteration with flat_map
let teams = vec![
    vec!["Alice", "Bob"],
    vec!["Charlie", "Dave", "Eve"],
];

let all_members: Vec<_> = teams.iter()
    .flat_map(|team| team.iter())
    .copied()
    .collect();
// ["Alice", "Bob", "Charlie", "Dave", "Eve"]

// Example: Grouping with fold
use std::collections::HashMap;

let words = vec!["apple", "apricot", "banana", "blueberry"];
let grouped: HashMap<char, Vec<&str>> = words.into_iter()
    .fold(HashMap::new(), |mut map, word| {
        map.entry(word.chars().next().unwrap())
            .or_insert_with(Vec::new)
            .push(word);
        map
    });
// { 'a': ["apple", "apricot"], 'b': ["banana", "blueberry"] }
```

### Quick Reference Table

| Combinator      | Type     | Signature                               | Use Case                             |
|-----------------|----------|-----------------------------------------|--------------------------------------|
| `map`           | Adapter  | `map(f: T -> U) -> Iterator<U>`         | Transform each element               |
| `filter`        | Adapter  | `filter(f: T -> bool) -> Iterator<T>`   | Keep matching elements               |
| `filter_map`    | Adapter  | `filter_map(f: T -> Option<U>)`         | Map and filter simultaneously        |
| `flat_map`      | Adapter  | `flat_map(f: T -> Iterator<U>)`         | Map and flatten                      |
| `take`          | Adapter  | `take(n: usize) -> Iterator<T>`         | First N elements                     |
| `skip`          | Adapter  | `skip(n: usize) -> Iterator<T>`         | Skip first N elements                |
| `chain`         | Adapter  | `chain(other: Iterator) -> Iterator`    | Concatenate iterators                |
| `zip`           | Adapter  | `zip(other: Iterator<U>) -> (T, U)`     | Pair with another iterator           |
| `enumerate`     | Adapter  | `enumerate() -> (usize, T)`             | Add indices                          |
| `inspect`       | Adapter  | `inspect(f: &T -> ()) -> Iterator<T>`   | Debug/log without consuming          |
| `collect`       | Consumer | `collect() -> Collection`               | Build collection                     |
| `sum`           | Consumer | `sum() -> T`                            | Add all elements                     |
| `product`       | Consumer | `product() -> T`                        | Multiply all elements                |
| `fold`          | Consumer | `fold(init: B, f: (B, T) -> B) -> B`    | Custom accumulation                  |
| `reduce`        | Consumer | `reduce(f: (T, T) -> T) -> Option<T>`   | Accumulate without initial value     |
| `find`          | Consumer | `find(f: T -> bool) -> Option<T>`       | First matching element               |
| `any`           | Consumer | `any(f: T -> bool) -> bool`             | Check if any match                   |
| `all`           | Consumer | `all(f: T -> bool) -> bool`             | Check if all match                   |
| `count`         | Consumer | `count() -> usize`                      | Count elements                       |
| `max`/`min`     | Consumer | `max() -> Option<T>`                    | Find extreme values                  |
| `partition`     | Consumer | `partition(f: T -> bool) -> (C, C)`     | Split into two collections           |

---

## Collections: Vec, HashMap, HashSet, and More

Rust's standard library provides a rich set of collection types, each optimized for different access patterns. Understanding when to use each collection is crucial for writing efficient code. The core principle: **choose the collection whose performance characteristics match your usage pattern**.

### Vec: Contiguous Growable Array

`Vec<T>` is the workhorse collection—use it as your default choice unless you have specific requirements for other types. It provides O(1) indexed access and amortized O(1) push/pop at the end.

```rust
// Usage: Vec is the default collection; O(1) push/pop at end, O(1) index access
use std::vec::Vec;

// Creating vectors
let v1: Vec<i32> = Vec::new();
let v2 = vec![1, 2, 3];                    // vec! macro
let v3 = Vec::with_capacity(100);          // Pre-allocate
let v4 = vec![0; 5];                       // [0, 0, 0, 0, 0]

// Adding elements
let mut numbers = Vec::new();
numbers.push(1);                           // Add to end - O(1) amortized
numbers.extend([2, 3, 4]);                 // Add multiple
numbers.append(&mut vec![5, 6]);           // Move elements from another vec
numbers.insert(0, 0);                      // Insert at index - O(n)

// Accessing elements
let first = numbers[0];                    // Panics if out of bounds
let second = numbers.get(1);               // Returns Option<&T>
let last = numbers.last();                 // Option<&T>
let slice = &numbers[1..4];                // Borrow a slice

// Removing elements
let last = numbers.pop();                  // Option<T> - O(1)
let removed = numbers.remove(0);           // T - O(n), shifts elements
numbers.clear();                           // Empty the vec

// Iteration
for num in &numbers {                      // Immutable borrow
    println!("{}", num);
}

for num in &mut numbers {                  // Mutable borrow
    *num *= 2;
}

for num in numbers {                       // Consumes the vec
    println!("{}", num);
}

// Capacity management
let mut v = Vec::with_capacity(10);
println!("len: {}, capacity: {}", v.len(), v.capacity());
v.reserve(20);                             // Ensure at least 20 more slots
v.shrink_to_fit();                         // Release unused memory

// Deduplication and sorting
let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5];
data.sort();                               // Sort in place - O(n log n)
data.dedup();                              // Remove consecutive duplicates
// [1, 2, 3, 4, 5, 6, 9]

// Binary search (requires sorted data)
let sorted = vec![1, 2, 3, 4, 5];
match sorted.binary_search(&3) {
    Ok(index) => println!("Found at {}", index),
    Err(index) => println!("Not found, would insert at {}", index),
}
```

### VecDeque: Double-Ended Queue

Use `VecDeque<T>` when you need efficient insertion/removal at both ends. It's implemented as a ring buffer.

```rust
// Usage: VecDeque for O(1) push/pop at both ends (ring buffer)
use std::collections::VecDeque;

// Creating VecDeque
let mut deque = VecDeque::new();
let mut deque2 = VecDeque::from(vec![1, 2, 3]);

// Adding at both ends - O(1)
deque.push_back(1);                        // Add to back
deque.push_front(0);                       // Add to front
// [0, 1]

// Removing from both ends - O(1)
let back = deque.pop_back();               // Some(1)
let front = deque.pop_front();             // Some(0)

// Use cases
// Queue (FIFO)
let mut queue = VecDeque::new();
queue.push_back(1);
queue.push_back(2);
let first = queue.pop_front();             // FIFO order

// Stack (LIFO) - but Vec is better for this
let mut stack = VecDeque::new();
stack.push_back(1);
stack.push_back(2);
let last = stack.pop_back();               // LIFO order
```

### HashMap: Hash-Based Key-Value Store

`HashMap<K, V>` provides O(1) average-case insertion and lookup. Use it when you need fast key-based access and don't care about ordering.

```rust
// Usage: HashMap for O(1) avg key lookup; keys need Hash + Eq
use std::collections::HashMap;

// Creating HashMaps
let mut scores = HashMap::new();
let mut map: HashMap<String, i32> = HashMap::with_capacity(100);

// Inserting and updating
scores.insert("Alice".to_string(), 10);
scores.insert("Bob".to_string(), 20);

// Returns previous value if key existed
let old = scores.insert("Alice".to_string(), 15);  // Some(10)

// Accessing values
let alice_score = scores.get("Alice");             // Option<&i32>
let bob_score = scores["Bob"];                     // Panics if missing

// Safe indexing with unwrap_or
let charlie_score = scores.get("Charlie").unwrap_or(&0);

// Checking existence
if scores.contains_key("Alice") {
    println!("Alice has a score");
}

// Removing entries
let removed = scores.remove("Bob");                // Option<V>

// Entry API (powerful pattern)
// Insert if missing
scores.entry("Charlie".to_string()).or_insert(0);

// Modify existing or insert default
let alice = scores.entry("Alice".to_string()).or_insert(0);
*alice += 5;

// Complex logic with entry
let word_counts = vec!["apple", "banana", "apple"];
let mut counts = HashMap::new();
for word in word_counts {
    let count = counts.entry(word).or_insert(0);
    *count += 1;
}
// {"apple": 2, "banana": 1}

// Iteration
for (key, value) in &scores {
    println!("{}: {}", key, value);
}

for key in scores.keys() {
    println!("{}", key);
}

for value in scores.values() {
    println!("{}", value);
}

// Convert from/to other types
let pairs = vec![("a", 1), ("b", 2)];
let map: HashMap<_, _> = pairs.into_iter().collect();

let vec_pairs: Vec<_> = map.into_iter().collect();
```

### HashSet: Hash-Based Set

`HashSet<T>` stores unique values with O(1) average-case insertion and membership testing. Use it for deduplication and set operations.

```rust
use std::collections::HashSet;

// Creating HashSets
let mut set = HashSet::new();
let set2: HashSet<i32> = [1, 2, 3].iter().cloned().collect();

// Adding and removing - O(1)
set.insert(1);                             // true if inserted
set.insert(1);                             // false (already exists)
set.remove(&1);                            // true if removed

// Checking membership
if set.contains(&1) {
    println!("Set contains 1");
}

// Set operations
let set1: HashSet<_> = [1, 2, 3].iter().cloned().collect();
let set2: HashSet<_> = [2, 3, 4].iter().cloned().collect();

// Union: all elements from both sets
let union: HashSet<_> = set1.union(&set2).cloned().collect();
// {1, 2, 3, 4}

// Intersection: elements in both sets
let intersection: HashSet<_> = set1.intersection(&set2).cloned().collect();
// {2, 3}

// Difference: elements in first but not second
let diff: HashSet<_> = set1.difference(&set2).cloned().collect();
// {1}

// Symmetric difference: elements in either but not both
let sym_diff: HashSet<_> = set1.symmetric_difference(&set2).cloned().collect();
// {1, 4}

// Subset and superset
let small = HashSet::from([1, 2]);
let large = HashSet::from([1, 2, 3]);
assert!(small.is_subset(&large));
assert!(large.is_superset(&small));

// Deduplication pattern
let numbers = vec![1, 2, 2, 3, 3, 3];
let unique: HashSet<_> = numbers.into_iter().collect();
let deduped: Vec<_> = unique.into_iter().collect();
```

### BTreeMap and BTreeSet: Ordered Collections

Use `BTreeMap<K, V>` and `BTreeSet<T>` when you need sorted keys. They provide O(log n) operations but maintain sorted order.

```rust
use std::collections::{BTreeMap, BTreeSet};

// BTreeMap: Sorted keys
let mut scores = BTreeMap::new();
scores.insert("Alice", 10);
scores.insert("Charlie", 30);
scores.insert("Bob", 20);

// Iteration is always sorted by key
for (name, score) in &scores {
    println!("{}: {}", name, score);
}
// Alice: 10
// Bob: 20
// Charlie: 30

// Range queries
let numbers: BTreeMap<i32, &str> = [
    (1, "one"),
    (5, "five"),
    (10, "ten"),
].iter().cloned().collect();

// Get all entries in range
for (key, value) in numbers.range(2..8) {
    println!("{}: {}", key, value);
}
// 5: five

// First and last entries
let first = scores.first_key_value();      // Option<(&K, &V)>
let last = scores.last_key_value();        // Option<(&K, &V)>

// BTreeSet: Sorted set
let mut set = BTreeSet::new();
set.insert(5);
set.insert(1);
set.insert(3);

// Always sorted
for num in &set {
    println!("{}", num);
}
// 1, 3, 5

// Range iteration
for num in set.range(2..=5) {
    println!("{}", num);
}
// 3, 5
```

### BinaryHeap: Priority Queue

`BinaryHeap<T>` is a max-heap that provides O(log n) insertion and O(log n) removal of the largest element.

```rust
use std::collections::BinaryHeap;

// Creating a BinaryHeap
let mut heap = BinaryHeap::new();

// Adding elements - O(log n)
heap.push(3);
heap.push(1);
heap.push(5);
heap.push(2);

// Peeking at largest - O(1)
let largest = heap.peek();                 // Some(&5)

// Removing largest - O(log n)
while let Some(max) = heap.pop() {
    println!("{}", max);
}
// 5, 3, 2, 1 (descending order)

// Min-heap using Reverse
use std::cmp::Reverse;
let mut min_heap = BinaryHeap::new();
min_heap.push(Reverse(3));
min_heap.push(Reverse(1));
min_heap.push(Reverse(5));

while let Some(Reverse(min)) = min_heap.pop() {
    println!("{}", min);
}
// 1, 3, 5 (ascending order)

// Priority queue use case
#[derive(Eq, PartialEq, Debug)]
struct Task {
    priority: u32,
    description: String,
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

let mut tasks = BinaryHeap::new();
tasks.push(Task { priority: 1, description: "Low".to_string() });
tasks.push(Task { priority: 5, description: "High".to_string() });
tasks.push(Task { priority: 3, description: "Medium".to_string() });

// Process tasks by priority
while let Some(task) = tasks.pop() {
    println!("Processing: {:?}", task.description);
}
// High, Medium, Low
```

### LinkedList: Doubly-Linked List

`LinkedList<T>` provides O(1) insertion/removal anywhere if you have a cursor, but poor cache locality. **Rarely the best choice**—use `Vec` or `VecDeque` instead in most cases.

```rust
use std::collections::LinkedList;

// Creating LinkedList
let mut list = LinkedList::new();
list.push_back(1);
list.push_back(2);
list.push_front(0);
// [0, 1, 2]

// Splitting and merging
let mut list1 = LinkedList::from([1, 2, 3]);
let mut list2 = LinkedList::from([4, 5, 6]);
list1.append(&mut list2);                  // Moves all elements from list2
// list1: [1, 2, 3, 4, 5, 6], list2: []

// Note: LinkedList is rarely optimal!
// Vec is usually better due to:
// - Better cache locality
// - Lower memory overhead
// - Better performance for most operations
```

### Collection Selection Guide

| Collection    | Order        | Key Access | Lookup  | Insert  | Remove  | Use When                                 |
|---------------|--------------|------------|---------|---------|---------|------------------------------------------|
| `Vec`         | Insertion    | Index      | O(1)    | O(1)*   | O(n)    | Default choice, indexed access           |
| `VecDeque`    | Insertion    | Index      | O(1)    | O(1)**  | O(1)**  | Queue, double-ended operations           |
| `HashMap`     | Unordered    | Key        | O(1)    | O(1)    | O(1)    | Fast key-value lookup, no order needed   |
| `HashSet`     | Unordered    | Value      | O(1)    | O(1)    | O(1)    | Deduplication, membership testing        |
| `BTreeMap`    | Sorted       | Key        | O(log n)| O(log n)| O(log n)| Sorted keys, range queries               |
| `BTreeSet`    | Sorted       | Value      | O(log n)| O(log n)| O(log n)| Sorted set, range queries                |
| `BinaryHeap`  | Heap order   | N/A        | N/A     | O(log n)| O(log n)| Priority queue                           |
| `LinkedList`  | Insertion    | N/A        | O(n)    | O(1)*** | O(1)*** | Rarely useful (prefer Vec/VecDeque)      |

\* Amortized O(1) at end
\** At ends only
\*** With cursor, O(n) to find position

---

## String and Text Processing

Rust distinguishes between owned strings (`String`) and borrowed string slices (`&str`). Understanding this distinction and the rich text processing capabilities is essential for working with text data.

### String vs &str

```rust
// &str: Borrowed string slice
let literal: &str = "Hello, world!";       // String literals are &str
let slice: &str = &String::from("hello")[0..2];  // Slice of String

// &str is:
// - Immutable
// - Fixed size (known at compile time or stored as fat pointer)
// - Doesn't own its data
// - Cheap to pass around

// String: Owned, growable
let mut owned = String::from("Hello");
let mut owned2 = "Hello".to_string();
let mut owned3 = String::new();

// String is:
// - Mutable
// - Heap-allocated
// - Growable
// - UTF-8 encoded

// Converting between
let s = String::from("hello");
let slice: &str = &s;                      // String -> &str (cheap)
let slice: &str = s.as_str();              // Explicit conversion

let owned: String = slice.to_string();     // &str -> String (allocates)
let owned: String = slice.to_owned();      // Same
let owned: String = String::from(slice);   // Same
```

### String Creation and Manipulation

```rust
// Creating strings
let s1 = String::new();
let s2 = String::from("hello");
let s3 = "hello".to_string();
let s4 = String::with_capacity(100);       // Pre-allocate

// Appending text
let mut s = String::from("Hello");
s.push_str(", world");                     // Append &str
s.push('!');                               // Append char
// "Hello, world!"

// Concatenation
let s1 = String::from("Hello");
let s2 = String::from(" world");

// Using + (takes ownership of left operand)
let s3 = s1 + &s2;                         // s1 moved, s2 borrowed
// s1 is now invalid!

// Using format! (doesn't take ownership)
let s1 = String::from("Hello");
let s2 = String::from(" world");
let s3 = format!("{}{}", s1, s2);          // Both still valid
let s4 = format!("{s1}{s2}");              // Shorter syntax

// Inserting and removing
let mut s = String::from("Hello world");
s.insert(5, ',');                          // Insert char at byte position
s.insert_str(6, " beautiful");             // Insert &str
// "Hello, beautiful world"

s.remove(5);                               // Remove char at byte position
s.truncate(5);                             // Cut off everything after index
// "Hello"

s.clear();                                 // Empty the string

// Replacing text
let s = "I like cats";
let s2 = s.replace("cats", "dogs");        // Returns new String
// "I like dogs"

let s = "aaabbbccc";
let s2 = s.replacen("a", "x", 2);          // Replace first n occurrences
// "xxabbbccc"
```

### String Inspection and Searching

```rust
let text = "Hello, world!";

// Basic properties
text.len();                                // 13 (byte length, not char count!)
text.is_empty();                           // false

// Checking contents
text.starts_with("Hello");                 // true
text.ends_with("!");                       // true
text.contains("world");                    // true

// Finding patterns
let pos = text.find("world");              // Some(7) - byte position
let pos = text.find('w');                  // Some(7)
let rpos = text.rfind('o');                // Some(8) - rightmost

// Checking predicates
let all_alpha = "hello".chars().all(|c| c.is_alphabetic());
let has_digit = "hello123".chars().any(|c| c.is_numeric());

// Splitting
let parts: Vec<&str> = "a,b,c,d".split(',').collect();
// ["a", "b", "c", "d"]

let parts: Vec<&str> = "a::b::c".split("::").collect();
// ["a", "b", "c"]

let parts: Vec<&str> = "  a  b  c  ".split_whitespace().collect();
// ["a", "b", "c"] - automatically trims

let (left, right) = "key=value".split_once('=').unwrap();
// ("key", "value")

let lines: Vec<&str> = "line1\nline2\nline3".lines().collect();
// ["line1", "line2", "line3"]

// Trimming whitespace
let trimmed = "  hello  ".trim();          // "hello"
let left = "  hello  ".trim_start();       // "hello  "
let right = "  hello  ".trim_end();        // "  hello"

let custom = "###hello###".trim_matches('#');  // "hello"
```

### Character Iteration

**Important**: Strings are UTF-8 encoded. Never index directly into a string! Use iteration instead.

```rust
let text = "Hello 世界";

// Iterate over chars
for c in text.chars() {
    println!("{}", c);
}
// H, e, l, l, o, , 世, 界

// Iterate over bytes
for b in text.bytes() {
    println!("{}", b);
}
// 72, 101, 108, 108, 111, 32, 228, 184, 150, 231, 149, 140

// Get char at position (expensive!)
let third_char = text.chars().nth(2);      // Some('l')

// Count characters (not bytes)
let char_count = text.chars().count();     // 8 chars
let byte_count = text.len();               // 13 bytes

// Character ranges
let chars: Vec<char> = ('a'..='z').collect();  // ['a', 'b', ..., 'z']
```

### String Slicing (Use with Caution!)

```rust
let s = "Hello, 世界";

// Slicing at valid UTF-8 boundaries
let slice = &s[0..5];                      // "Hello"

// DANGER: Slicing at invalid boundaries
// let bad = &s[0..8];                     // Panics! Not a char boundary

// Safe slicing with get
let safe = s.get(0..5);                    // Some("Hello")
let bad = s.get(0..8);                     // None (invalid boundary)

// Finding character boundaries
if s.is_char_boundary(5) {
    let slice = &s[..5];
}
```

### Parsing and Formatting

```rust
use std::fmt;

// Parsing from strings
let num: i32 = "42".parse().unwrap();
let num: Result<i32, _> = "not a number".parse();  // Err

let float: f64 = "3.14".parse().unwrap();
let boolean: bool = "true".parse().unwrap();

// Formatting with format!
let name = "Alice";
let age = 30;

let msg = format!("Name: {}, Age: {}", name, age);
let msg = format!("Name: {name}, Age: {age}");     // Named arguments

// Format specifiers
format!("{:>10}", "right");                // "     right" (right-align, width 10)
format!("{:<10}", "left");                 // "left      " (left-align)
format!("{:^10}", "center");               // "  center  " (center)
format!("{:0>5}", "42");                   // "00042" (pad with zeros)

format!("{:.2}", 3.14159);                 // "3.14" (2 decimal places)
format!("{:e}", 1000.0);                   // "1e3" (scientific notation)
format!("{:#x}", 255);                     // "0xff" (hex with prefix)
format!("{:#b}", 10);                      // "0b1010" (binary with prefix)

// Custom Display implementation
struct Point { x: i32, y: i32 }

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

let p = Point { x: 10, y: 20 };
let s = format!("{}", p);                  // "(10, 20)"
```

### Case Conversion

```rust
let s = "Hello, World!";

// Case conversion
let lower = s.to_lowercase();              // "hello, world!"
let upper = s.to_uppercase();              // "HELLO, WORLD!"

// Unicode-aware (handles special cases)
let german = "Straße";
let upper = german.to_uppercase();         // "STRASSE" (ß -> SS)

// Case checking
let all_lower = s.chars().all(|c| c.is_lowercase() || !c.is_alphabetic());
```

### Regular Expressions (regex crate)

```rust
// Requires: regex = "1.0" in Cargo.toml
use regex::Regex;

// Creating and matching
let re = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
let is_match = re.is_match("2024-01-15");  // true

// Capturing groups
let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
let text = "Date: 2024-01-15";

if let Some(caps) = re.captures(text) {
    let year = &caps[1];                   // "2024"
    let month = &caps[2];                  // "01"
    let day = &caps[3];                    // "15"
}

// Finding all matches
let re = Regex::new(r"\d+").unwrap();
for mat in re.find_iter("Numbers: 42, 100, 7") {
    println!("{}", mat.as_str());
}
// "42", "100", "7"

// Replacing text
let re = Regex::new(r"\d+").unwrap();
let result = re.replace_all("Id: 123, Code: 456", "XXX");
// "Id: XXX, Code: XXX"
```

### Common String Patterns

```rust
// Joining strings
let words = vec!["Hello", "world"];
let sentence = words.join(" ");            // "Hello world"

let numbers = vec![1, 2, 3];
let csv = numbers.iter()
    .map(|n| n.to_string())
    .collect::<Vec<_>>()
    .join(",");                            // "1,2,3"

// Repeating strings
let repeated = "abc".repeat(3);            // "abcabcabc"

// Escaping special chars
let with_newlines = "Line 1\nLine 2\nLine 3";
let escaped = with_newlines.escape_default().to_string();
// "Line 1\\nLine 2\\nLine 3"

// Building strings efficiently
let mut s = String::with_capacity(100);    // Pre-allocate if you know size
for i in 0..10 {
    s.push_str(&i.to_string());
    s.push(' ');
}
```

---

## Option and Result: Error Handling

Rust uses `Option<T>` for values that might be absent and `Result<T, E>` for operations that might fail. These types replace null pointers and exceptions, making error handling explicit and composable.

### Option<T>: Handling Optional Values

```rust
// Creating Option values
let some_value: Option<i32> = Some(42);
let no_value: Option<i32> = None;

// Pattern matching (most explicit)
match some_value {
    Some(x) => println!("Value: {}", x),
    None => println!("No value"),
}

// if let (single pattern)
if let Some(x) = some_value {
    println!("Value: {}", x);
}

// Unwrapping (use with caution!)
let value = some_value.unwrap();           // Panics if None
let value = some_value.expect("No value"); // Panics with custom message
let value = some_value.unwrap_or(0);       // Provides default
let value = some_value.unwrap_or_else(|| expensive_default());
let value = some_value.unwrap_or_default(); // Uses Default::default()

// Checking for presence
if some_value.is_some() {
    println!("Has value");
}

if no_value.is_none() {
    println!("No value");
}

// Transforming Option
let doubled = some_value.map(|x| x * 2);   // Some(84)
let none_doubled = no_value.map(|x| x * 2); // None

// and_then for chaining operations that return Option
fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 { None } else { Some(a / b) }
}

let result = Some(10)
    .and_then(|x| divide(x, 2))            // Some(5)
    .and_then(|x| divide(x, 0));           // None

// Filtering
let value = Some(42);
let filtered = value.filter(|&x| x > 50);  // None (failed predicate)
let kept = value.filter(|&x| x > 30);      // Some(42)

// Converting between types
let opt: Option<i32> = Some(42);
let result: Result<i32, &str> = opt.ok_or("No value");  // Ok(42)

let result: Result<i32, &str> = Err("Error");
let opt: Option<i32> = result.ok();        // None (discards error)

// Borrowing inner value
let value = Some(String::from("hello"));
let borrowed: Option<&str> = value.as_ref().map(|s| s.as_str());
let length: Option<usize> = value.as_ref().map(|s| s.len());

// Taking ownership
let mut value = Some(42);
let taken = value.take();                  // Some(42), value is now None

// Replacing value
let mut value = Some(42);
let old = value.replace(100);              // old is Some(42), value is Some(100)
```

### Result<T, E>: Handling Errors

```rust
use std::fs::File;
use std::io::{self, Read};

// Functions returning Result
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Pattern matching
match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}

// The ? operator
fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;      // Returns early if Err
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Unwrapping (use sparingly!)
let value = divide(10, 2).unwrap();        // Panics on Err
let value = divide(10, 2).expect("Math error");

// Providing defaults
let value = divide(10, 0).unwrap_or(0);
let value = divide(10, 0).unwrap_or_else(|_| expensive_default());
let value = divide(10, 0).unwrap_or_default();

// Transforming Results
let doubled = divide(10, 2).map(|x| x * 2);  // Ok(10)
let err_mapped = divide(10, 0)
    .map_err(|e| format!("Fatal: {}", e));   // Map error type

// Chaining operations
let result = divide(10, 2)
    .and_then(|x| divide(x, 2))            // Ok(2)
    .and_then(|x| divide(x, 0));           // Err

// Checking status
if divide(10, 2).is_ok() {
    println!("Success");
}

if divide(10, 0).is_err() {
    println!("Failed");
}

// Converting between Ok/Err and Option
let result: Result<i32, String> = Ok(42);
let opt: Option<i32> = result.ok();        // Some(42), discards error
let err_opt: Option<String> = result.err(); // None

// Combining multiple Results (all must succeed)
let r1: Result<i32, &str> = Ok(1);
let r2: Result<i32, &str> = Ok(2);
let combined: Result<Vec<i32>, &str> =
    vec![r1, r2].into_iter().collect();    // Ok(vec![1, 2])

// Early return on first error
fn process() -> Result<(), String> {
    divide(10, 2)?;                        // Continue if Ok
    divide(20, 4)?;                        // Continue if Ok
    divide(5, 0)?;                         // Returns Err immediately
    Ok(())                                 // Never reached
}
```

### Error Propagation Patterns

```rust
use std::fs::File;
use std::io::{self, Read};

// Manual error propagation (verbose)
fn read_username_v1() -> Result<String, io::Error> {
    let f = File::open("username.txt");
    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),
    };

    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),
    }
}

// With ? operator (idiomatic)
fn read_username_v2() -> Result<String, io::Error> {
    let mut f = File::open("username.txt")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}

// Chaining with ? (more concise)
fn read_username_v3() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("username.txt")?.read_to_string(&mut s)?;
    Ok(s)
}

// Converting error types with ? and From
use std::num::ParseIntError;

fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    let num: i32 = s.parse()?;             // ? converts ParseIntError
    Ok(num * 2)
}
```

### Custom Error Types

```rust
use std::fmt;
use std::error::Error;

// Simple error enum
#[derive(Debug)]
enum MathError {
    DivisionByZero,
    NegativeSquareRoot,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathError::DivisionByZero => write!(f, "Division by zero"),
            MathError::NegativeSquareRoot => write!(f, "Square root of negative number"),
        }
    }
}

impl Error for MathError {}

fn safe_divide(a: f64, b: f64) -> Result<f64, MathError> {
    if b == 0.0 {
        Err(MathError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

// Using thiserror crate (recommended)
// Cargo.toml: thiserror = "1.0"
use thiserror::Error;

#[derive(Error, Debug)]
enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("Invalid data at line {line}: {msg}")]
    Invalid { line: usize, msg: String },
}
```

### Combinators Reference

```rust
// Option combinators
let opt = Some(42);

opt.map(|x| x * 2);                        // Some(84)
opt.and_then(|x| Some(x * 2));             // Some(84)
opt.or(Some(0));                           // Some(42)
opt.filter(|&x| x > 50);                   // None
opt.zip(Some(10));                         // Some((42, 10))

// Result combinators
let res: Result<i32, &str> = Ok(42);

res.map(|x| x * 2);                        // Ok(84)
res.and_then(|x| Ok(x * 2));               // Ok(84)
res.or(Ok(0));                             // Ok(42)
res.map_err(|e| format!("Error: {}", e));  // Transform error

// Early returns with ?
fn compute() -> Result<i32, String> {
    let a = divide(10, 2)?;
    let b = divide(a, 2)?;
    Ok(b)
}
```

---

## Smart Pointers: Box, Rc, Arc, RefCell

Smart pointers provide ownership and borrowing patterns beyond simple references. They enable recursive data structures, shared ownership, and interior mutability while maintaining Rust's safety guarantees.

### Box<T>: Heap Allocation

`Box<T>` is the simplest smart pointer—it allocates data on the heap and provides unique ownership.

```rust
// Creating boxed values
let boxed_int = Box::new(42);
let boxed_string = Box::new(String::from("hello"));

// Use cases for Box

// 1. Recursive data structures (size must be known)
#[derive(Debug)]
enum List {
    Cons(i32, Box<List>),
    Nil,
}

use List::{Cons, Nil};

let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

// 2. Large values you want to avoid copying
struct LargeStruct {
    data: [u8; 1000000],
}

let large = Box::new(LargeStruct { data: [0; 1000000] });
// Only the Box pointer is copied, not the data

// 3. Trait objects (dynamic dispatch)
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

let animal: Box<dyn Animal> = Box::new(Dog);
animal.speak();

// Accessing boxed values
let boxed = Box::new(42);
let value = *boxed;                        // Dereference to get value
println!("{}", boxed);                     // Auto-deref for Display

// Box provides unique ownership
let boxed = Box::new(42);
let moved = boxed;                         // Ownership transferred
// boxed is now invalid

// Converting to raw pointer
let boxed = Box::new(42);
let raw = Box::into_raw(boxed);            // *mut i32
unsafe {
    println!("{}", *raw);
    let _ = Box::from_raw(raw);            // Must reconstruct to free
}
```

### Rc<T>: Reference Counted Shared Ownership

`Rc<T>` allows multiple owners of the same data through reference counting. **Single-threaded only**.

```rust
use std::rc::Rc;

// Creating Rc values
let rc1 = Rc::new(42);
let rc2 = Rc::clone(&rc1);                 // Increment ref count
let rc3 = rc1.clone();                     // Same as Rc::clone

// All point to same data
println!("{}, {}, {}", rc1, rc2, rc3);     // 42, 42, 42

// Checking reference count
println!("Count: {}", Rc::strong_count(&rc1));  // 3

// Drop decrements count, frees when 0
drop(rc2);
println!("Count: {}", Rc::strong_count(&rc1));  // 2

// Use case: Shared graph nodes
use std::rc::Rc;

struct Node {
    value: i32,
    children: Vec<Rc<Node>>,
}

let leaf = Rc::new(Node { value: 3, children: vec![] });
let node = Rc::new(Node {
    value: 5,
    children: vec![Rc::clone(&leaf)],
});
// leaf is shared between owners

// Weak references (break cycles)
use std::rc::{Rc, Weak};

struct Parent {
    children: Vec<Rc<Child>>,
}

struct Child {
    parent: Weak<Parent>,                  // Weak doesn't increment count
}

let parent = Rc::new(Parent { children: vec![] });
let child = Rc::new(Child {
    parent: Rc::downgrade(&parent),        // Create Weak from Rc
});

// Access weak reference
if let Some(parent_rc) = child.parent.upgrade() {
    // parent_rc is Rc<Parent>
}
```

### Arc<T>: Atomic Reference Counted (Thread-Safe)

`Arc<T>` is the thread-safe version of `Rc<T>`, using atomic operations for reference counting.

```rust
use std::sync::Arc;
use std::thread;

// Creating Arc values
let arc1 = Arc::new(42);
let arc2 = Arc::clone(&arc1);

// Sharing across threads
let data = Arc::new(vec![1, 2, 3, 4, 5]);

let handles: Vec<_> = (0..3)
    .map(|i| {
        let data_clone = Arc::clone(&data);
        thread::spawn(move || {
            println!("Thread {}: {:?}", i, data_clone);
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}

// Checking reference count
println!("Count: {}", Arc::strong_count(&arc1));

// Use case: Shared immutable state
use std::sync::Arc;
use std::thread;

struct Config {
    max_connections: usize,
    timeout_ms: u64,
}

let config = Arc::new(Config {
    max_connections: 100,
    timeout_ms: 5000,
});

let config_clone = Arc::clone(&config);
thread::spawn(move || {
    println!("Max connections: {}", config_clone.max_connections);
});

println!("Timeout: {}", config.timeout_ms);
```

### RefCell<T>: Interior Mutability

`RefCell<T>` provides interior mutability—allows mutation through shared references. **Checks borrowing rules at runtime instead of compile time**. Single-threaded only.

```rust
use std::cell::RefCell;

// Creating RefCell values
let cell = RefCell::new(42);

// Borrowing mutably through shared ref
{
    let mut borrow = cell.borrow_mut();    // Runtime borrow check
    *borrow += 1;
}

let value = cell.borrow();                 // Immutable borrow
println!("{}", *value);                    // 43

// DANGER: Runtime panics on borrow violations
let cell = RefCell::new(42);
let borrow1 = cell.borrow_mut();
// let borrow2 = cell.borrow();            // Panics! Already mutably borrowed

// Checking borrow state
if let Ok(value) = cell.try_borrow() {
    println!("{}", *value);
} else {
    println!("Already borrowed mutably");
}

// Use case: Multiple owners with mutation
use std::rc::Rc;
use std::cell::RefCell;

struct SharedData {
    value: RefCell<i32>,
}

let data = Rc::new(SharedData {
    value: RefCell::new(0),
});

let data_clone = Rc::clone(&data);

*data.value.borrow_mut() += 1;
*data_clone.value.borrow_mut() += 1;

println!("{}", data.value.borrow());       // 2
```

### Cell<T>: Simple Interior Mutability

`Cell<T>` provides interior mutability for `Copy` types without runtime borrow checking.

```rust
use std::cell::Cell;

// Creating Cell values
let cell = Cell::new(42);

// Getting and setting
let value = cell.get();                    // 42 (Copy types only)
cell.set(100);
let new_value = cell.get();                // 100

// Swapping and updating
let old = cell.replace(200);               // Returns old value
cell.update(|x| x * 2);                    // 400

// Use case: Counters and flags
struct Counter {
    count: Cell<u32>,
}

impl Counter {
    fn increment(&self) {                  // Takes &self, not &mut self!
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> u32 {
        self.count.get()
    }
}

let counter = Counter { count: Cell::new(0) };
counter.increment();
counter.increment();
println!("{}", counter.get());             // 2
```

### Mutex<T> and RwLock<T>: Thread-Safe Interior Mutability

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// Mutex: Exclusive access
let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter_clone = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter_clone.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Count: {}", *counter.lock().unwrap());  // 10

// RwLock: Multiple readers or one writer
let data = Arc::new(RwLock::new(vec![1, 2, 3]));

// Multiple readers
let data_clone1 = Arc::clone(&data);
let reader1 = thread::spawn(move || {
    let vec = data_clone1.read().unwrap();
    println!("{:?}", *vec);
});

let data_clone2 = Arc::clone(&data);
let reader2 = thread::spawn(move || {
    let vec = data_clone2.read().unwrap();
    println!("{:?}", *vec);
});

// One writer
let writer = thread::spawn(move || {
    let mut vec = data.write().unwrap();
    vec.push(4);
});

reader1.join().unwrap();
reader2.join().unwrap();
writer.join().unwrap();
```

### Smart Pointer Selection Guide

| Pointer     | Thread-Safe | Ownership      | Mutability         | Use When                                    |
|-------------|-------------|----------------|--------------------|--------------------------------------------|
| `Box<T>`    | Yes         | Single         | Through &mut       | Heap allocation, recursive types           |
| `Rc<T>`     | No          | Shared         | Immutable          | Multiple owners, single thread             |
| `Arc<T>`    | Yes         | Shared         | Immutable          | Multiple owners, multiple threads          |
| `RefCell<T>`| No          | Single         | Interior mutability| Mutation through shared ref, single thread |
| `Cell<T>`   | No          | Single         | Interior mutability| Copy types, simple updates                 |
| `Mutex<T>`  | Yes         | Shared         | Interior mutability| Shared mutable state across threads        |
| `RwLock<T>` | Yes         | Shared         | Interior mutability| Many readers, few writers, across threads  |

**Common Combinations:**
- `Rc<RefCell<T>>`: Multiple owners with mutation (single-threaded)
- `Arc<Mutex<T>>`: Multiple owners with mutation (multi-threaded)
- `Arc<RwLock<T>>`: Multiple readers, occasional writers (multi-threaded)

---

## Cargo Commands Reference

Cargo is Rust's build system and package manager, handling compilation, dependency management, testing, and publishing. Understanding Cargo's command structure transforms your development workflow from managing files to orchestrating projects.

Think of Cargo as your project lifecycle manager. It creates scaffolding, fetches dependencies, invokes the compiler, runs tests, generates documentation, and publishes crates. Every Rust project beyond a single-file script benefits from Cargo's conventions and automation.

### Project Initialization

Starting a new project with Cargo creates the standard structure that the entire ecosystem expects:

```bash
# Create binary (application) project
cargo new my_app
# Creates:
# my_app/
# ├── Cargo.toml       # Manifest file
# └── src/
#     └── main.rs      # Binary entry point with fn main()

# Create library project
cargo new my_lib --lib
# Creates:
# my_lib/
# ├── Cargo.toml
# └── src/
#     └── lib.rs       # Library root

# Initialize in existing directory
cd existing_project
cargo init
cargo init --lib  # For library

# Project naming conventions
cargo new snake_case_name      # Preferred: uses underscores
cargo new kebab-case-name      # Also works: converted to underscores
```

The `Cargo.toml` manifest is your project's metadata and dependency specification:

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2021"           # Rust edition (2015, 2018, 2021)
authors = ["Your Name <you@example.com>"]
license = "MIT"
description = "A brief description"
repository = "https://github.com/user/my_app"

[dependencies]
# Dependencies from crates.io
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

# Git dependencies
my_utils = { git = "https://github.com/user/my_utils" }

# Local path dependencies
shared = { path = "../shared" }

[dev-dependencies]
# Test-only dependencies
proptest = "1.0"
criterion = "0.5"

[build-dependencies]
# Build script dependencies
cc = "1.0"
```

### Building and Running

Cargo manages the compile-run cycle with sensible defaults for development and release builds:

```bash
# Check code for errors (fastest - no codegen)
cargo check
# Use this constantly during development
# Runs type checking and borrow checker without producing binaries

# Build in debug mode (default)
cargo build
# Produces unoptimized binary at target/debug/my_app
# Fast compilation, slow execution, includes debug symbols

# Build in release mode
cargo build --release
# Produces optimized binary at target/release/my_app
# Slow compilation, fast execution, strips debug info

# Run the binary (builds if needed)
cargo run
cargo run --release
cargo run -- arg1 arg2        # Pass arguments to binary
cargo run --bin other_binary  # Run specific binary

# Build and run examples
cargo run --example my_example

# Build for specific target
cargo build --target x86_64-unknown-linux-musl
```

**Understanding debug vs release:**

- **Debug builds** (`cargo build`): Fast compile, slow runtime, large binaries
  - Optimizations: Off (opt-level = 0)
  - Debug symbols: Included
  - Overflow checks: Enabled
  - Use during development

- **Release builds** (`cargo build --release`): Slow compile, fast runtime, small binaries
  - Optimizations: Full (opt-level = 3)
  - Debug symbols: Stripped
  - Overflow checks: Disabled
  - Use for production, benchmarks, performance testing

### Testing

Rust's testing is built into the language and integrated with Cargo:

```bash
# Run all tests
cargo test

# Run tests matching pattern
cargo test test_addition       # Runs tests with "test_addition" in name
cargo test integration::       # Runs tests in integration module

# Run tests in specific file
cargo test --test integration_tests

# Run doc tests only
cargo test --doc

# Run tests with output (show println!)
cargo test -- --nocapture

# Run tests single-threaded
cargo test -- --test-threads=1

# Run ignored tests
cargo test -- --ignored

# Run benchmarks
cargo bench
```

**Test organization:**

```rust
// Unit tests (in same file as code)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("Expected panic");
    }

    #[test]
    #[ignore]  // Skip unless --ignored specified
    fn expensive_test() {
        // ...
    }
}

// Integration tests (in tests/ directory)
#[test]
fn test_public_api() {
    use my_lib::public_function;
    assert!(public_function());
}
```

### Dependency Management

Cargo handles transitive dependencies, version resolution, and lock files automatically:

```bash
# Add dependency (modifies Cargo.toml)
cargo add serde
cargo add tokio --features full
cargo add serde_json --dev         # Dev dependency
cargo add cc --build               # Build dependency

# Update dependencies to latest compatible versions
cargo update
cargo update serde                 # Update specific dependency

# Remove dependency
cargo remove serde

# Display dependency tree
cargo tree
cargo tree -i serde               # Show inverse dependencies (what needs serde)

# Check for outdated dependencies
cargo outdated  # Requires cargo-outdated plugin
```

**Version specification syntax:**

```toml
[dependencies]
# Caret (default): Compatible updates
serde = "^1.2.3"    # >=1.2.3, <2.0.0
serde = "1.2.3"     # Same as above (^ is implicit)

# Tilde: Patch updates only
serde = "~1.2.3"    # >=1.2.3, <1.3.0

# Exact version
serde = "=1.2.3"    # Exactly 1.2.3

# Wildcard
serde = "1.2.*"     # >=1.2.0, <1.3.0

# Comparison operators
serde = ">1.2.0"
serde = ">=1.2.0, <2.0.0"
```

The `Cargo.lock` file pins exact versions for reproducible builds:
- **Checked into version control** for binaries (ensures same build everywhere)
- **Not checked in** for libraries (allows dependents to use newer compatible versions)

### Documentation

Cargo generates HTML documentation from your doc comments:

```bash
# Generate and open documentation
cargo doc --open

# Include private items
cargo doc --document-private-items

# Generate without dependencies
cargo doc --no-deps
```

**Writing doc comments:**

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// assert_eq!(my_lib::add(2, 3), 5);
/// ```
///
/// # Panics
///
/// Panics if the sum overflows.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Publishing and Packaging

Cargo integrates with crates.io for publishing libraries:

```bash
# Login to crates.io (one-time setup)
cargo login YOUR_API_TOKEN

# Package for distribution (creates .crate file)
cargo package

# Dry-run publish (check for errors)
cargo publish --dry-run

# Publish to crates.io
cargo publish

# Yank a version (prevents new projects from using it)
cargo yank --vers 0.1.0
cargo yank --vers 0.1.0 --undo  # Un-yank
```

### Workspaces: Multi-Crate Projects

Workspaces manage multiple related crates in a single repository:

```toml
# Workspace Cargo.toml (at repository root)
[workspace]
members = [
    "server",
    "client",
    "shared",
]

# Individual crates have their own Cargo.toml files
# server/Cargo.toml
[package]
name = "server"
version = "0.1.0"

[dependencies]
shared = { path = "../shared" }
```

```bash
# Build all workspace crates
cargo build --workspace

# Run specific workspace binary
cargo run -p server

# Test all workspace crates
cargo test --workspace
```

### Advanced Commands

```bash
# Clean build artifacts
cargo clean

# Format code
cargo fmt
cargo fmt -- --check  # Check formatting without modifying

# Lint with Clippy
cargo clippy
cargo clippy -- -D warnings  # Fail on warnings

# Expand macros
cargo expand         # Requires cargo-expand

# View assembly
cargo asm            # Requires cargo-asm

# Security audit
cargo audit          # Requires cargo-audit

# Show why a dependency is included
cargo tree -i dependency_name

# Fix compiler warnings automatically
cargo fix

# Vendoring dependencies (bundle all dependencies)
cargo vendor
```

### Quick Reference Table

| Command                     | Purpose                                   | Common Options                    |
|-----------------------------|-------------------------------------------|-----------------------------------|
| `cargo new <name>`          | Create new project                        | `--lib`, `--vcs none`             |
| `cargo init`                | Initialize in existing dir                | `--lib`                           |
| `cargo build`               | Compile project                           | `--release`, `--target`           |
| `cargo run`                 | Build and run binary                      | `--release`, `-- <args>`          |
| `cargo check`               | Check code without building               | Fast feedback during development  |
| `cargo test`                | Run tests                                 | `--test <name>`, `-- --nocapture` |
| `cargo bench`               | Run benchmarks                            | Requires `#[bench]` or criterion  |
| `cargo doc`                 | Generate documentation                    | `--open`, `--no-deps`             |
| `cargo add <crate>`         | Add dependency                            | `--dev`, `--build`, `--features`  |
| `cargo update`              | Update dependencies                       | Updates to latest compatible      |
| `cargo tree`                | Show dependency tree                      | `-i <crate>` for inverse deps     |
| `cargo clean`               | Remove build artifacts                    | Frees disk space                  |
| `cargo fmt`                 | Format code                               | `-- --check` for CI               |
| `cargo clippy`              | Lint code                                 | `-- -D warnings` to fail on warn  |
| `cargo publish`             | Publish to crates.io                      | `--dry-run` to test first         |
| `cargo search <query>`      | Search crates.io                          |                                   |
| `cargo install <crate>`     | Install binary crate globally             |                                   |

### Environment Variables and Configuration

Cargo respects environment variables for customization:

```bash
# Increase parallelism
CARGO_BUILD_JOBS=8 cargo build

# Use offline mode (don't fetch updates)
CARGO_NET_OFFLINE=true cargo build

# Custom registry
CARGO_REGISTRY_DEFAULT=my-registry cargo build

# Target directory (useful for CI caching)
CARGO_TARGET_DIR=/tmp/target cargo build
```

Configuration in `~/.cargo/config.toml`:

```toml
[build]
jobs = 8
target-dir = "/tmp/cargo-target"

[term]
color = "always"

[net]
git-fetch-with-cli = true

[alias]
b = "build"
r = "run"
t = "test"
```

---

### Conclusion

This quick reference captures the patterns you'll use daily in production Rust development. Type conversions establish clear boundaries between domains. Trait implementations make your types composable with the standard library. Iterator combinators transform data processing from imperative loops into declarative pipelines. Cargo commands orchestrate your entire project lifecycle.

Keep this appendix close as you write code. The patterns become muscle memory through repetition, but having a concise reference accelerates learning and prevents subtle mistakes. When you encounter unfamiliar territory, use these examples as starting points and refer back to the detailed chapters for deeper understanding.


