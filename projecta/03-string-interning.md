## Project 3: Custom String Interning with Cow Patterns

### Problem Statement

Build a string interning system that stores unique strings once and reuses them. This demonstrates Clone-on-Write (Cow) patterns and zero-copy optimization.

### Use Cases
1. **Compilers/Interpreters**: Variable names, function names, keywords, string literals
2. **Configuration systems**: Keys in config files (often repeated)
3. **Web frameworks**: Route paths, template variable names, header field names
4. **Databases**: Table/column names, SQL keywords, username strings
5. **Game engines**: Asset names, entity tags, component type names
6. **Logging systems**: Log levels, logger names, common message patterns


### Why It Matters

**Real-World Impact**: String duplication wastes massive amounts of memory in real programs:

**The String Duplication Problem**:
- Compiler parsing 100K LOC: identifier "count" appears 5,000 times
- Without interning: 5,000 allocations × 6 bytes = **30KB** for one identifier
- With interning: 1 allocation × 6 bytes = **6 bytes**, 5,000 pointers (8 bytes each) = **40KB total**
- But: pointers are often stack-allocated or in structs, actual savings = **29.9KB per repeated identifier**
- Across thousands of identifiers: **Megabytes of savings**

**Performance Benefits**:
1. **Memory**: 10-40% reduction in string memory for identifier-heavy workloads
2. **Comparison**: `O(1)` pointer equality vs `O(n)` string comparison
3. **Hashing**: Hash once, reuse hash value (important for HashMaps)
4. **Cache**: Fewer unique strings = better cache locality

**Cow Pattern Benefits**:
- **Zero-copy**: If string already interned, return borrowed reference (no allocation)
- **Lazy allocation**: Only allocate when necessary
- **API clarity**: Caller knows if allocation happened by checking `Cow` variant


**String Interning is Critical When**:
- Many duplicate strings (identifiers in code, repeated log messages)
- String comparison is frequent (symbol table lookups)
- Memory is constrained (embedded systems, large-scale deployments)

**Cow Pattern is Critical When**:
- Processing user input (may or may not need normalization)
- Path manipulation (may or may not need conversion)
- HTML escaping (most strings don't need escaping)

### Learning Goals

- Understand `Cow<'_, T>` and when to use it for zero-copy
- Implement string interning for memory optimization
- Build generational indices for safe handles (no lifetime issues)
- Measure memory savings and performance improvements
- Learn trade-offs: when interning helps vs hurts

---

### Milestone 1: Understand Cow Basics

**Goal**: Learn how `Cow` (Clone-on-Write) works through hands-on examples that demonstrate zero-copy optimization.

**Why This Milestone Matters**:

`Cow<'_, T>` is one of Rust's most elegant patterns for performance optimization. It solves a common dilemma: **"Should my function return a borrowed reference or an owned value?"**

The answer is often: **"It depends on the input!"**

**The Problem `Cow` Solves**:

Imagine writing a function that normalizes whitespace in text. Sometimes the input is already normalized (no work needed), sometimes it needs modification. What should the function signature be?

**Option 1: Always return `String` (always allocate)**
```rust
fn normalize(text: &str) -> String {
    text.replace("  ", " ")  // Always allocates, even if no changes!
}
```
❌ **Problem**: Wastes memory and time when input is already clean (90% of cases)

**Option 2: Return `&str` (never allocate)**
```rust
fn normalize(text: &str) -> &str {
    text  // Can't modify!
}
```
❌ **Problem**: Can't handle cases that need modification

**Option 3: Return `Cow<str>` (allocate only when needed)**
```rust
fn normalize(text: &str) -> Cow<str> {
    if text.contains("  ") {
        Cow::Owned(text.replace("  ", " "))  // Allocate when needed
    } else {
        Cow::Borrowed(text)  // Zero-copy when clean
    }
}
```
✅ **Perfect**: Zero overhead for clean input, handles modifications when needed!

**What is `Cow`?**

`Cow` stands for **Clone-on-Write** (or **Copy-on-Write**). It's an enum with two variants:

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),  // Borrowed reference (zero-copy)
    Owned(<B as ToOwned>::Owned),  // Owned value (allocated)
}
```

For strings:
- `Cow::Borrowed(&str)` - Points to existing string data
- `Cow::Owned(String)` - Owns heap-allocated string data

**Key Insights**:

1. **Caller's perspective**: `Cow<str>` acts like a string—you can read it, compare it, print it
2. **Zero-copy path**: When no modification needed, return `Cow::Borrowed` (no allocation!)
3. **Allocation path**: When modification needed, return `Cow::Owned` (allocate once)
4. **API clarity**: The type signature tells callers "might allocate, might not"

**Real-World Performance Impact**:

Consider processing 10,000 log lines, normalizing whitespace:
- 9,000 lines already clean (90%)
- 1,000 lines need normalization (10%)

**With `String` return (always allocate)**:
- 10,000 allocations
- ~75ns each = **750,000ns = 0.75ms**

**With `Cow<str>` return (allocate only when needed)**:
- 1,000 allocations (only for dirty lines)
- ~75ns each = **75,000ns = 0.075ms**
- **10x faster!**

**Common `Cow` Use Cases**:

1. **Text processing**: Escaping, normalization, case conversion
    - Most strings don't need escaping → return borrowed
    - HTML special chars found → return owned (escaped version)

2. **Path manipulation**: Canonicalization, directory separators
    - Already canonical → return borrowed
    - Needs normalization → return owned

3. **Configuration loading**: Environment variable expansion
    - No variables like `${FOO}` → return borrowed
    - Variables present → return owned (expanded version)

4. **Data validation**: Trimming, sanitization
    - Already valid → return borrowed
    - Needs fixes → return owned

**Key Concepts**:

1. **`normalize_whitespace(text: &str) -> Cow<str>`**
    - Checks for double spaces or tabs
    - If found: replace with single spaces (allocate)
    - If not found: return original (zero-copy)

2. **`maybe_escape_html(text: &str) -> Cow<str>`**
    - Checks for `<`, `>`, `&` characters
    - If found: escape to `&lt;`, `&gt;`, `&amp;` (allocate)
    - If not found: return original (zero-copy)


**Checkpoint Tests**:
```rust
#[test]
fn test_normalize_no_change() {
    let result = normalize_whitespace("hello world");
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(result, "hello world");
}

#[test]
fn test_normalize_with_change() {
    let result = normalize_whitespace("hello  world");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "hello world");
}

#[test]
fn test_escape_no_html() {
    let result = maybe_escape_html("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn test_escape_with_html() {
    let result = maybe_escape_html("<div>");
    assert!(matches!(result, Cow::Owned(_)));
    assert_eq!(result, "&lt;div&gt;");
}
```

**Exercises**:
```rust
use std::borrow::Cow;

// Exercise 1: Function that sometimes modifies input
fn normalize_whitespace(text: &str) -> Cow<str> {
    if text.contains("  ") || text.contains('\t') {
        // Need to modify - return Owned
    } else {
        // No modification needed - return Borrowed
    }
}

// Exercise 2: Function that might escape HTML
// replace: '&' -> &amp;  '<' -> &lt;  '>' -> &gt;
fn maybe_escape_html(text: &str) -> Cow<str> {
    if text.contains('<') || text.contains('>') || text.contains('&') {
        // TODO replace
    } else {
        // TODO
    }
}
```

**Check Your Understanding**:
- When should you return `Cow::Borrowed` vs `Cow::Owned`?
- What's the benefit of returning `Cow` vs always returning `String`?
- How can the caller use a `Cow<str>`?

---

### Why Milestone 1 Isn't Enough → Moving to Milestone 2

**Limitation**: `Cow` shows us *when* to avoid allocations, but doesn't actually *store* strings for reuse. Each call still checks/modifies independently.

**The Real Problem**: Consider processing 1 million log messages, many containing "ERROR: Connection timeout". Without interning:
- Each occurrence: Parse, check, maybe allocate
- No sharing between occurrences
- Memory: Thousands of copies of "ERROR: Connection timeout"

**What we're adding**: **String Interning** - global string pool:
- **HashSet<Box<str>>** - stores unique strings once
- **References** - return `&str` pointing into the set
- **Deduplication** - automatic string reuse

**Improvements**:
- **Memory**: One allocation per unique string (not per occurrence)
- **Comparison**: `ptr::eq()` for equality (vs strcmp)
- **Lifetime**: Strings live as long as interner exists
- **Cost**: HashSet lookup + occasional allocation

**Performance Numbers**:
- Without interning: 1M strings × 25 bytes average = **25MB**
- With interning (10K unique): 10K × 25 bytes = **250KB** (100x savings!)
- Lookup overhead: ~50ns per intern call (hash + comparison)
- Win when: duplicates > ~2x per unique string

---

### Milestone 2: Basic String Interner

**Goal**: Implement a string interner that stores each unique string once and returns references to deduplicated storage.

**Why This Milestone Matters**:

Now that we understand `Cow` for conditional allocation, let's tackle a bigger problem: **string duplication across your entire program**. String interning is a powerful technique that trades lookup time for dramatic memory savings.

**What is String Interning?**

String interning is a technique where:
1. **Unique strings stored once**: First occurrence allocates and stores
2. **Duplicates return references**: Subsequent occurrences return pointer to existing storage
3. **Pointer equality works**: Can compare strings with `ptr::eq()` instead of `strcmp()`

**The Core Design**:

```rust
struct StringInterner {
    strings: HashSet<Box<str>>,  // Set of unique strings
}
```

**The `intern()` Algorithm**:

```rust
fn intern(&mut self, s: &str) -> &str {
    // 1. Check if string already in set
    if !self.strings.contains(s) {
        // 2. First time seeing this string - allocate and store
        self.strings.insert(Box::from(s));
    }
    // 3. Return reference to the string in the set
    self.strings.get(s).unwrap()
}
```

**Key Insight**: `HashSet::get()` returns a reference to the **stored value**, not the input! This is how we return `&str` with a longer lifetime.

**Lifetime Magic**:

Notice the signature: `fn intern(&mut self, s: &str) -> &str`

The returned `&str` is **not** tied to the input `s`—it's tied to `&mut self`! The string lives in the `HashSet`, so it lives as long as the interner.

```rust
let interner = StringInterner::new();
let interned: &str = interner.intern("hello");
// `interned` lives as long as `interner`, not the string literal
```

**Pointer Equality Optimization**:

With interning, you can compare strings by pointer:

```rust
let s1 = interner.intern("hello");
let s2 = interner.intern("hello");

// Fast pointer comparison (1 CPU cycle)
assert!(std::ptr::eq(s1, s2));

// Slow string comparison (N cycles, where N = string length)
// assert_eq!(s1, s2);  // Not needed anymore!
```

**The Methods You'll Implement**:

1. **`new() -> Self`**: Create empty interner with empty HashSet
2. **`intern(&mut self, s: &str) -> &str`**: Add string to set if new, return reference
3. **`contains(&self, s: &str) -> bool`**: Check if string is interned
4. **`len(&self) -> usize`**: Number of unique strings stored
5. **`total_bytes(&self) -> usize`**: Total bytes used by all strings



**Checkpoint Tests**:
```rust
#[test]
fn test_intern_basic() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("hello");

    // Should be same pointer (no second allocation)
    assert!(std::ptr::eq(s1, s2));
    assert_eq!(interner.len(), 1);
}

#[test]
fn test_intern_different() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello");
    let s2 = interner.intern("world");

    assert!(!std::ptr::eq(s1, s2));
    assert_eq!(interner.len(), 2);
}

#[test]
fn test_contains() {
    let mut interner = StringInterner::new();
    interner.intern("hello");

    assert!(interner.contains("hello"));
    assert!(!interner.contains("world"));
}

#[test]
fn test_total_bytes() {
    let mut interner = StringInterner::new();
    interner.intern("hi");     // 2 bytes
    interner.intern("hello");  // 5 bytes

    assert_eq!(interner.total_bytes(), 7);
}
```
**Starter Code**:
```rust
use std::collections::HashSet;

struct StringInterner {
    strings: HashSet<Box<str>>,
}

impl StringInterner {
    fn new() -> Self {
        // TODO: Create new StringInterner with empty HashSet
        todo!()
    }

    fn intern(&mut self, s: &str) -> &str {
        // TODO: Check if string already interned using contains()
        if todo!("Check if !self.strings.contains(s)") {
            // TODO: Insert Box::from(s) into self.strings
            todo!();
        }

        // TODO: Get reference to the interned string from HashSet
        // Hint: self.strings.get(s).unwrap()
        // This works because we just inserted it if it wasn't there
        todo!()
    }

    fn contains(&self, s: &str) -> bool {
        // TODO: Check if strings HashSet contains s
        todo!()
    }

    fn len(&self) -> usize {
        // TODO: Return length of strings HashSet
        todo!()
    }

    fn total_bytes(&self) -> usize {
        // TODO: Sum up the length of all strings
        // Hint: self.strings.iter().map(|s| s.len()).sum()
        todo!()
    }
}
```

**Check Your Understanding**:
- Why do we use `Box<str>` instead of `String`?
- Why can we return `&str` from intern even though it takes `&mut self`?
- What makes the pointers equal for the same string?

---

### Milestone 3: Add Cow-based API

**Goal**: Combine the `Cow` pattern from Milestone 1 with the interner from Milestone 2 to create an API that communicates allocation status.

**Why This Milestone Matters**:

In Milestone 1, we learned that `Cow` communicates **"did we allocate or not?"** to the caller. In Milestone 2, we built an interner but lost that information—`intern()` always returns `&str`, hiding whether allocation happened.

Let's bring these concepts together!

**The Problem with `intern()`**:

```rust
let s1 = interner.intern("hello");  // First time - allocates
let s2 = interner.intern("hello");  // Already there - no allocation
```

Both calls return `&str`, so the caller can't tell which one allocated. 

**The Solution: `get_or_intern()`**:

```rust
fn get_or_intern(&mut self, s: &str) -> Cow<str> {
    if self.contains(s) {
        Cow::Borrowed(self.strings.get(s).unwrap())  // Already there
    } else {
        self.strings.insert(Box::from(s));
        Cow::Borrowed(self.strings.get(s).unwrap())  // Just inserted
    }
}
```

**Wait, why always `Cow::Borrowed`?**

Good question! You might expect:
```rust
// Intuitive but WRONG approach
fn get_or_intern(&mut self, s: &str) -> Cow<str> {
    if self.contains(s) {
        Cow::Borrowed(self.strings.get(s).unwrap())
    } else {
        Cow::Owned(s.to_string())  // ❌ Wrong!
    }
}
```

**Why this is wrong**: The interner's job is to **store and return references to stored strings**. If we return `Cow::Owned(String)`, the string isn't in the interner—it's owned by the caller! That defeats the purpose.

**The Correct Pattern**:

Actually, for a string interner, `get_or_intern()` should **always** return `Cow::Borrowed` because:
1. Already interned → borrow from HashSet
2. Not interned → insert, then borrow from HashSet

The `Cow` variant isn't the right way to communicate allocation here (it's always `Borrowed`). In the next milestone, we'll add explicit statistics tracking instead.


**Key Takeaway**:

This milestone illustrates that **`Cow` isn't always the right tool**. It's perfect for "maybe modify the input" but awkward for "maybe store the input." This prepares you for Milestone 4's better solution: explicit statistics tracking.



**Checkpoint Tests**:
```rust
#[test]
fn test_cow_already_interned() {
    let mut interner = StringInterner::new();
    interner.intern("hello");

    let result = interner.get_or_intern("hello");
    assert!(matches!(result, Cow::Borrowed(_)));
}

#[test]
fn test_cow_new_string() {
    let mut interner = StringInterner::new();

    let result = interner.get_or_intern("hello");
    // First time still returns Borrowed after interning
    assert!(matches!(result, Cow::Borrowed(_)));
    assert_eq!(interner.len(), 1);
}
```
**Code**:
```rust
impl StringInterner {
    fn get_or_intern(&mut self, s: &str) -> Cow<str> {
        // TODO: Check if string is already interned
        if todo!("self.contains(s)") {
            // TODO: Return Cow::Borrowed with reference from HashSet
            // Hint: Cow::Borrowed(self.strings.get(s).unwrap())
            todo!()
        } else {
            // TODO: Insert the string into HashSet
            todo!();
            // TODO: Return Cow::Borrowed with reference to newly inserted string
            todo!()
        }
    }
}
```

**Check Your Understanding**:
- Why does `get_or_intern` always return `Cow::Borrowed`?
- When would it return `Cow::Owned`?
- How does this API communicate whether allocation happened?

---

### Milestone 4: Add Statistics Tracking

**Goal**: Add comprehensive statistics to measure interner effectiveness and understand allocation patterns.

**Why This Milestone Matters**:

As we learned in Milestone 3, `Cow` isn't the ideal way to track string interner performance. What we really need is **aggregate statistics** that answer questions like:

- **Is the interner effective?** High hit rate (lookups/total) = good reuse!
- **Should we use interning?** If allocation rate is too low, overhead might not be worth it
- **Memory saved**: Compare `total_bytes` vs. `(allocations + lookups) × average_length`
- **Performance tuning**: Identify which strings are duplicated most

**The Problem Without Statistics**:

```rust
let mut interner = StringInterner::new();
// ... 10,000 intern calls later ...
// ❓ How many were duplicates?
// ❓ How much memory did we save?
// ❓ Is the interner actually helping?
```

Without stats, you're flying blind. You don't know if the interner is paying for itself!

**What We're Adding: `InternerStats` struct**:

```rust
struct InternerStats {
    total_strings: usize,   // Unique strings currently stored
    total_bytes: usize,     // Total bytes used by strings
    allocations: usize,     // How many new strings added
    lookups: usize,         // How many duplicate strings found
}
```

**Key Metrics**:

1. **Hit Rate**: `lookups / (allocations + lookups)`
    - High hit rate (>50%) = interner is valuable
    - Low hit rate (<10%) = mostly unique strings, overhead may not be worth it

2. **Memory Efficiency**: Compare actual memory vs. without interning
    - Without: `(allocations + lookups) × average_string_length`
    - With: `total_bytes` (only unique strings)
    - Savings: `(without - with) / without × 100%`

3. **Allocation Ratio**: `allocations / total_calls`
    - Low ratio = lots of reuse (good for interning)
    - High ratio = mostly unique (bad for interning)

**Real-World Example: Web Server Logs**:

Imagine processing 100,000 HTTP log entries:
```
GET /api/users 200
GET /api/users 200
GET /api/posts 404
GET /api/users 200
...
```

**Expected pattern**:
- 10,000 unique strings (paths, status codes, methods)
- 100,000 total strings
- Hit rate: 90% (90,000 lookups, 10,000 allocations)

**Statistics would show**:
```rust
InternerStats {
    total_strings: 10_000,     // 10K unique
    total_bytes: 250_000,      // ~25 bytes average
    allocations: 10_000,       // 10K new strings
    lookups: 90_000,           // 90K duplicates found!
}
```

**Analysis**:
- **Hit rate**: 90,000 / 100,000 = 90% ✅ Excellent!
- **Memory without interning**: 100,000 × 25 = 2.5MB
- **Memory with interning**: 250KB
- **Savings**: (2.5MB - 250KB) / 2.5MB = **90% memory saved!** 🎉

**When Statistics Show Interning Is NOT Worth It**:

```rust
// Processing unique user comments (no duplicates)
InternerStats {
    total_strings: 100_000,
    total_bytes: 5_000_000,    // 50 bytes average
    allocations: 100_000,      // Every string is new
    lookups: 0,                // No hits!
}
```

**Analysis**:
- **Hit rate**: 0% ❌ Terrible!
- **Overhead**: Hash computation + HashSet storage + lookup time
- **Verdict**: Remove the interner, just use `String` directly

**Implementing Statistics**:

The stats need to be updated in `intern()`:

```rust
fn intern(&mut self, s: &str) -> &str {
    if !self.strings.contains(s) {
        // New string - record allocation
        self.strings.insert(Box::from(s));
        self.stats.total_strings += 1;
        self.stats.total_bytes += s.len();
        self.stats.allocations += 1;
    } else {
        // Duplicate - record lookup
        self.stats.lookups += 1;
    }
    self.strings.get(s).unwrap()
}
```

**Production Monitoring**:

In real systems, you'd export these stats to monitoring:

```rust
// Export to Prometheus
gauge!("interner.total_strings", interner.stats.total_strings as f64);
gauge!("interner.total_bytes", interner.stats.total_bytes as f64);
counter!("interner.allocations", interner.stats.allocations as u64);
counter!("interner.lookups", interner.stats.lookups as u64);
```

This lets you graph hit rate over time, alert on low efficiency, etc.

**Why Track Both `total_strings` AND `allocations`?**

Good question! They're usually equal, but might differ if you add a `clear()` or `remove()` method:

```rust
interner.intern("hello");  // allocations=1, total_strings=1
interner.clear();          // allocations=1, total_strings=0 (cleared!)
interner.intern("world");  // allocations=2, total_strings=1
```

`allocations` is the **lifetime total**, `total_strings` is the **current count**.

**Performance Cost of Statistics**:

Adding stats is cheap:
- Increment counters: ~1ns each (just memory writes)
- String length: already computed for HashSet
- No allocations, no complex computation

The cost is negligible compared to the HashSet lookup (~50ns).

**Alternative: Separate `StatsInterner` Type?**

Some designs use generics to make statistics optional:

```rust
struct StringInterner<S = NoStats> {
    strings: HashSet<Box<str>>,
    stats: S,
}
```

This avoids overhead for users who don't need stats, but adds API complexity. For learning, we'll just always include stats (overhead is tiny anyway).


**Checkpoint Tests**:
```rust
#[test]
fn test_stats() {
    let mut interner = StringInterner::new();

    interner.intern("hello");  // allocation
    interner.intern("world");  // allocation
    interner.intern("hello");  // lookup

    let stats = interner.statistics();
    assert_eq!(stats.total_strings, 2);
    assert_eq!(stats.total_bytes, 10);  // 5 + 5
    assert_eq!(stats.allocations, 2);
    assert_eq!(stats.lookups, 1);
}

#[test]
fn test_stats_empty() {
    let interner = StringInterner::new();
    let stats = interner.statistics();
    assert_eq!(stats.total_strings, 0);
    assert_eq!(stats.allocations, 0);
}
```
**Starter Code**:
```rust
#[derive(Debug, PartialEq)]
struct InternerStats {
    total_strings: usize,
    total_bytes: usize,
    allocations: usize,  // How many times we allocated
    lookups: usize,      // How many times we just returned existing
}

impl StringInterner {
    fn new() -> Self {
        // TODO: Create StringInterner with empty HashSet and zero stats
        todo!()
    }

    fn intern(&mut self, s: &str) -> &str {
        // TODO: Check if string is not already interned
        if todo!("!self.strings.contains(s)") {
            // TODO: Insert string into HashSet
            todo!();
            // TODO: Update statistics: increment total_strings, add to total_bytes, increment allocations
            todo!();
        } else {
            // TODO: Increment lookups count (string was already interned)
            todo!();
        }

        // TODO: Return reference to interned string
        todo!()
    }

    fn statistics(&self) -> &InternerStats {
        // TODO: Return reference to stats
        todo!()
    }
}
```

**Check Your Understanding**:
- Why track both allocations and lookups?
- How does this help evaluate interner effectiveness?

---

### 🔄 Why Milestone 4 Isn't Enough → Moving to Milestone 5

Our interner from Milestone 4 has a critical flaw: **lifetime hell**. Every interned string reference is tied to the interner's lifetime, making it nearly impossible to use in real applications.

**The Lifetime Problem**:

```rust
struct Compiler<'intern> {
    identifiers: Vec<&'intern str>,  // ❌ Lifetime everywhere!
    interner: &'intern StringInterner,  // ❌ Must hold reference
}

// Can't return identifiers without dragging 'intern lifetime along
fn parse<'intern>(source: &str, interner: &'intern mut StringInterner)
    -> Result<Vec<&'intern str>, Error> {  // ❌ Lifetime infected return type!
    // ...
}
```

**The pain gets worse**:
- Can't store identifiers in one struct and interner in another
- Can't serialize/deserialize (references can't be saved to disk)
- Can't send between threads easily (lifetimes don't cross thread boundaries cleanly)
- Can't build self-referential structures (compiler forbids them)

**What we're adding**: **Generational Indices** (AKA "slot map" pattern):
- **Symbol** handle: `{index: usize, generation: u32}` - Copy, 'static
- **Indirection**: Symbol → lookup in Vec → get string
- **Stale detection**: Generation mismatch = invalid symbol

**Improvements**:
- **Lifetime freedom**: Symbols are `Copy` + `'static`, store anywhere
- **Safety**: Stale symbols return `None` (not dangling pointers)
- **Memory reuse**: Freed slots recycled with incremented generation
- **Cost**: Extra indirection (Vec lookup) ~2-3ns

**Comparison**:
- `&'a str` approach: Zero runtime cost, lifetime complexity
- `Symbol` approach: Small runtime cost, no lifetime complexity
- **Choose Symbol when**: Need flexibility, store in multiple places, serialize/deserialize

**Memory layout**:
- Symbol: 12 bytes (8 byte index + 4 byte generation)
- Reference: 8 bytes (just pointer)
- Trade-off: 50% more memory per handle, but no lifetime constraints

---

### Milestone 5: Symbol-Based Access with Generational Indices

**Goal**: Replace lifetime-bound references with `Copy` handles that work anywhere, using the generational index pattern to detect stale handles safely.


**Handles Instead of References**:

Instead of returning `&str` (with lifetime), return a `Symbol` handle (no lifetime):

```rust
#[derive(Copy, Clone, PartialEq)]
struct Symbol {
    index: usize,      // Which slot in the interner?
    generation: u32,   // Which version of that slot?
}
```

Now your code looks like:

```rust
struct Compiler {
    identifiers: Vec<Symbol>,  // ✅ No lifetime!
    interner: SymbolInterner,  // ✅ Can own it
}

fn parse(source: &str, interner: &mut SymbolInterner) -> Result<Vec<Symbol>, Error> {
    // ✅ No lifetimes in return type!
}
```

**What Are Generational Indices?**

Generational indices (also called "slot maps" or "generational arena") solve two problems:

1. **Stable handles**: Index stays valid even if other items are removed
2. **Dangling detection**: Generation number catches stale references

**The Core Idea**:

```rust
struct Slot {
    string: Option<Box<str>>,  // None = slot is free
    generation: u32,           // Incremented each time slot is reused
}

struct SymbolInterner {
    slots: Vec<Slot>,           // All slots (some filled, some free)
    free_list: Vec<usize>,      // Indices of free slots to reuse
}
```

**How It Works**:

1. **Allocate**: Find free slot (or create new one), store string, return `Symbol{index, generation}`
2. **Resolve**: Look up `slots[index]`, check generation matches, return `&str` or `None`
3. **Remove**: Set `slots[index].string = None`, increment generation, add index to free list
4. **Reuse**: Next allocation reuses freed slot with new generation number

**Example Walkthrough**:

```rust
let mut interner = SymbolInterner::new();

// 1. Intern "hello" → creates slot 0
let sym1 = interner.intern("hello");  // Symbol{index: 0, generation: 0}
assert_eq!(interner.resolve(sym1), Some("hello"));

// 2. Remove "hello" → frees slot 0, increments generation
interner.remove(sym1);
// slots[0] = Slot{string: None, generation: 1}
// free_list = [0]

// 3. Try to resolve old symbol → generation mismatch!
assert_eq!(interner.resolve(sym1), None);  // sym1 has gen=0, slot has gen=1

// 4. Intern "world" → reuses slot 0 with new generation
let sym2 = interner.intern("world");  // Symbol{index: 0, generation: 1}
assert_eq!(interner.resolve(sym2), Some("world"));

// 5. Old symbol still doesn't work
assert_eq!(interner.resolve(sym1), None);  // Still stale!
```

**Why Generations?**

Without generations, you'd have a classic "dangling pointer" bug:

```rust
// Without generations (BAD!)
let sym1 = interner.intern("hello");  // Symbol{index: 0}
interner.remove(sym1);
let sym2 = interner.intern("world");  // Reuses slot 0

// BUG: sym1 resolves to "world" instead of None!
assert_eq!(interner.resolve(sym1), Some("world"));  // ❌ Wrong string!
```

With generations, stale symbols return `None` safely:

```rust
// With generations (GOOD!)
let sym1 = interner.intern("hello");  // Symbol{index: 0, gen: 0}
interner.remove(sym1);                 // Slot becomes {None, gen: 1}
let sym2 = interner.intern("world");  // Symbol{index: 0, gen: 1}

assert_eq!(interner.resolve(sym1), None);       // ✅ Detects stale!
assert_eq!(interner.resolve(sym2), Some("world"));  // ✅ Correct!
```

**Memory Layout**:

```
SymbolInterner:
  slots: [
    Slot{string: Some("hello"), generation: 0},   // index 0
    Slot{string: None, generation: 3},            // index 1 (freed 3 times)
    Slot{string: Some("world"), generation: 0},   // index 2
  ]
  free_list: [1]  // Slot 1 is available for reuse
```

**Performance Trade-Offs**:

| Aspect | `&str` Approach | `Symbol` Approach |
|--------|----------------|-------------------|
| **Resolve speed** | Direct pointer dereference (~1ns) | Vec lookup + generation check (~3ns) |
| **Handle size** | 8 bytes (pointer) | 12 bytes (index + generation) |
| **Lifetime complexity** | High (infects everything) | Zero (Copy, 'static) |
| **Safety** | Compiler enforced | Runtime checks |
| **Serialization** | Impossible | Easy (just two numbers) |
| **Thread safety** | Complex (lifetime bounds) | Simple (Copy, Send, Sync) |

**When to Use Which**:

✅ **Use `Symbol` (generational index) when**:
- Need to store in multiple places
- Need to serialize/deserialize
- Want to avoid lifetime annotations everywhere
- Building complex data structures (graphs, trees)
- Working with concurrent code

✅ **Use `&str` (reference) when**:
- Short-lived, local usage only
- Performance-critical tight loop (avoid indirection)
- Simple codebase where lifetimes aren't a burden

**Real-World Examples**:

1. **Rust compiler**: Uses `Symbol` for identifiers (from `rustc_span::symbol`)
    - 100,000s of identifiers across compilation
    - Stored in AST nodes, type tables, name resolution tables
    - Serialized to incremental compilation cache

2. **Game engines (Bevy, Amethyst)**: Entity IDs are generational indices
    - Entities can be despawned and IDs reused
    - Systems store entity references without lifetimes
    - Generation catches "use after despawn" bugs

3. **GUI frameworks (Druid, Iced)**: Widget IDs
    - Widgets destroyed and recreated frequently
    - Event handlers store widget IDs across frames
    - Stale IDs safely ignored

**Implementation Strategy**:

1. **`intern(s: &str) -> Symbol`**:
    - Check if string already exists (linear search through slots)
    - If found: return Symbol with that index/generation
    - If not found:
        - Try `free_list.pop()` for reusable slot
        - Otherwise push new slot
        - Return Symbol

2. **`resolve(symbol: Symbol) -> Option<&str>`**:
    - Look up `slots[symbol.index]`
    - Check `slot.generation == symbol.generation`
    - If match: return `Some(&string)`
    - If mismatch: return `None` (stale)

3. **`remove(symbol: Symbol)`**:
    - Check generation matches
    - Set `slot.string = None`
    - Increment `slot.generation`
    - Push index to `free_list`



**Checkpoint Tests**:
```rust
#[test]
fn test_symbol_intern() {
    let mut interner = SymbolInterner::new();

    let sym1 = interner.intern("hello");
    let sym2 = interner.intern("hello");

    // Same string should have same symbol
    assert_eq!(sym1, sym2);
    assert_eq!(interner.resolve(sym1), Some("hello"));
}

#[test]
fn test_symbol_resolve() {
    let mut interner = SymbolInterner::new();

    let sym = interner.intern("test");
    assert_eq!(interner.resolve(sym), Some("test"));
}

#[test]
fn test_stale_symbol() {
    let mut interner = SymbolInterner::new();

    let sym1 = interner.intern("test");
    interner.clear();

    // sym1 is now stale
    assert_eq!(interner.resolve(sym1), None);
}

#[test]
fn test_generation_reuse() {
    let mut interner = SymbolInterner::new();

    let sym1 = interner.intern("test");
    let index1 = sym1.index;
    let gen1 = sym1.generation;

    interner.remove(sym1);

    // Interning again should reuse slot but increment generation
    let sym2 = interner.intern("test");
    assert_eq!(sym2.index, index1);  // Same slot
    assert_ne!(sym2.generation, gen1);  // Different generation
}

#[test]
fn test_symbol_lifetime_safety() {
    let mut interner = SymbolInterner::new();
    let sym = interner.intern("test");

    // Symbol can outlive the borrow of interner
    drop(interner);

    // This is safe - we just can't resolve it anymore
    let _copy = sym;  // Symbol is Copy
}
```


**Starter Code**:
```rust
#[derive(Debug, Copy, Clone, PartialEq)]
struct Symbol {
    index: usize,
    generation: u32,
}

struct Slot {
    string: Option<Box<str>>,
    generation: u32,
}

struct SymbolInterner {
    slots: Vec<Slot>,
    free_list: Vec<usize>,
}

impl SymbolInterner {
    fn new() -> Self {
        // TODO: Create SymbolInterner with empty slots and free_list
        todo!()
    }

    fn intern(&mut self, s: &str) -> Symbol {
        // TODO: Check if string already exists in slots
        // Hint: Loop through slots, check if slot.string matches s
        for (index, slot) in self.slots.iter().enumerate() {
            if let Some(existing) = &slot.string {
                if existing.as_ref() == s {
                    // TODO: Return Symbol with this index and generation
                    todo!()
                }
            }
        }

        // Not found - allocate new slot
        // TODO: Check if there's a free slot to reuse
        if let Some(index) = self.free_list.pop() {
            // TODO: Reuse freed slot
            // - Get mutable reference to slot at index
            // - Increment generation
            // - Set string to Some(Box::from(s))
            // - Return Symbol with index and new generation
            todo!()
        } else {
            // TODO: Allocate new slot at end of Vec
            // - Get index (current slots.len())
            // - Push new Slot with string and generation 0
            // - Return Symbol with index and generation 0
            todo!()
        }
    }

    fn resolve(&self, symbol: Symbol) -> Option<&str> {
        // TODO: Get slot at symbol.index
        // TODO: Check if generation matches
        // TODO: If matches, return string as Option<&str>, else None
        // Hint: self.slots.get(symbol.index).and_then(|slot| ...)
        todo!()
    }

    fn remove(&mut self, symbol: Symbol) {
        // TODO: Get mutable reference to slot at symbol.index
        // TODO: Check if generation matches
        // TODO: If matches, set string to None and push index to free_list
        todo!()
    }

    fn clear(&mut self) {
        // TODO: Iterate through all slots
        // TODO: For each slot with a string:
        //   - Set string to None
        //   - Increment generation
        //   - Push index to free_list
        todo!()
    }
}
```

**Check Your Understanding**:
- Why use symbols instead of direct string references?
- How do generational indices detect stale references?
- What's the advantage of reusing slots with free_list?
- Why is Symbol Copy but still safe?

---

### Milestone 6: Performance Comparison

**Goal**: Measure the benefit of interning.

**Benchmark Code**:
```rust
use std::time::Instant;

fn benchmark_with_interner() {
    let mut interner = StringInterner::new();
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];

    let start = Instant::now();
    for _ in 0..100000 {
        for word in &words {
            let _ = interner.intern(word);
        }
    }
    let duration = start.elapsed();

    let stats = interner.statistics();
    println!("With interner: {:?}", duration);
    println!("Stats: {:?}", stats);
}

fn benchmark_without_interner() {
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];
    let mut strings = Vec::new();

    let start = Instant::now();
    for _ in 0..100000 {
        for word in &words {
            strings.push(word.to_string());  // Always allocate
        }
    }
    let duration = start.elapsed();

    println!("Without interner: {:?}", duration);
    println!("Allocations: {}", strings.len());
}
```

**Expected Results**: Interner should be much faster for duplicate-heavy workloads and use significantly less memory.

**Check Your Understanding**:
- When does interning help most?
- When might interning hurt performance?
- What's the memory trade-off?

---

### Complete Project Summary

**What You Built**:
1. Understanding of `Cow<T>` for zero-copy patterns
2. Basic string interner with HashSet
3. Statistics tracking for allocations
4. Symbol-based access with generational indices
5. Performance comparisons

**Key Concepts Practiced**:
- Clone-on-Write patterns
- String interning benefits
- Generational indices for safe handles
- Trade-offs between copying and interning

---
