## Project 3: Custom String Interning with Cow Patterns

### Problem Statement

Build a string interning system that stores unique strings once and reuses them. This demonstrates Clone-on-Write (Cow) patterns and zero-copy optimization.

### Why It Matters

**Real-World Impact**: String duplication wastes massive amounts of memory in real programs:

**The String Duplication Problem**:
- Compiler parsing 100K LOC: identifier "count" appears 5,000 times
- Without interning: 5,000 allocations × 6 bytes = **30KB** for one identifier
- With interning: 1 allocation × 6 bytes = **6 bytes**, 5,000 pointers (8 bytes each) = **40KB total**
- But: pointers are often stack-allocated or in structs, actual savings = **29.9KB per repeated identifier**
- Across thousands of identifiers: **Megabytes of savings**

**Real Production Examples**:
- **Rust compiler**: `Symbol` interning saves 40% memory on large codebases
- **Python**: All string literals interned, identifiers interned automatically
- **Java JVM**: String pool for literals, manual `intern()` for runtime strings
- **JavaScript V8**: Symbol table interning for property names
- **Databases**: Column names, table names, SQL keywords interned

**Performance Benefits**:
1. **Memory**: 10-40% reduction in string memory for identifier-heavy workloads
2. **Comparison**: `O(1)` pointer equality vs `O(n)` string comparison
3. **Hashing**: Hash once, reuse hash value (important for HashMaps)
4. **Cache**: Fewer unique strings = better cache locality

**Cow Pattern Benefits**:
- **Zero-copy**: If string already interned, return borrowed reference (no allocation)
- **Lazy allocation**: Only allocate when necessary
- **API clarity**: Caller knows if allocation happened by checking `Cow` variant

### Use Cases

**When you need this pattern**:
1. **Compilers/Interpreters**: Variable names, function names, keywords, string literals
2. **Configuration systems**: Keys in config files (often repeated)
3. **Web frameworks**: Route paths, template variable names, header field names
4. **Databases**: Table/column names, SQL keywords, username strings
5. **Game engines**: Asset names, entity tags, component type names
6. **Logging systems**: Log levels, logger names, common message patterns

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

**Goal**: Learn how `Cow` works with simple examples.

**Exercises**:
```rust
use std::borrow::Cow;

// Exercise 1: Function that sometimes modifies input
fn normalize_whitespace(text: &str) -> Cow<str> {
    if text.contains("  ") || text.contains('\t') {
        // Need to modify - return Owned
        let normalized = text.replace("  ", " ").replace('\t', " ");
        Cow::Owned(normalized)
    } else {
        // No modification needed - return Borrowed
        Cow::Borrowed(text)
    }
}

// Exercise 2: Function that might escape HTML
fn maybe_escape_html(text: &str) -> Cow<str> {
    if text.contains('<') || text.contains('>') || text.contains('&') {
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        Cow::Owned(escaped)
    } else {
        Cow::Borrowed(text)
    }
}
```

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

**Check Your Understanding**:
- When should you return `Cow::Borrowed` vs `Cow::Owned`?
- What's the benefit of returning `Cow` vs always returning `String`?
- How can the caller use a `Cow<str>`?

---

### 🔄 Why Milestone 1 Isn't Enough → Moving to Milestone 2

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

**Goal**: Store unique strings and return references.

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

**Check Your Understanding**:
- Why do we use `Box<str>` instead of `String`?
- Why can we return `&str` from intern even though it takes `&mut self`?
- What makes the pointers equal for the same string?

---

### Milestone 3: Add Cow-based API

**Goal**: Return `Cow` to show whether allocation happened.

**Add Method**:
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

**Check Your Understanding**:
- Why does `get_or_intern` always return `Cow::Borrowed`?
- When would it return `Cow::Owned`?
- How does this API communicate whether allocation happened?

---

### Milestone 4: Add Statistics Tracking

**Goal**: Track allocations and memory usage.

**Add Struct**:
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

**Check Your Understanding**:
- Why track both allocations and lookups?
- How does this help evaluate interner effectiveness?

---

### 🔄 Why Milestone 4 Isn't Enough → Moving to Milestone 5

**Critical Limitation**: Lifetimes! Returning `&str` from intern ties all references to the interner's lifetime. This causes problems:

```rust
let s: &str;
{
    let mut interner = StringInterner::new();
    s = interner.intern("hello");  // ❌ Error: s outlives interner
}
println!("{}", s);  // Dangling reference!
```

**Real-world pain points**:
- Can't store interned strings in long-lived structs without holding interner reference
- Compiler fights you with lifetime errors
- `&'static str` doesn't work for runtime strings

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

**Real-world usage**:
- Game engines (entity IDs)
- GUI frameworks (widget handles)
- Compilers (symbol table indices)
- Databases (row IDs with generation for MVCC)

**Memory layout**:
- Symbol: 12 bytes (8 byte index + 4 byte generation)
- Reference: 8 bytes (just pointer)
- Trade-off: 50% more memory per handle, but no lifetime constraints

---

### Milestone 5: Symbol-Based Access with Generational Indices

**Goal**: Use handles instead of direct references to avoid lifetime issues.

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

## Final Review Questions

After completing all three projects, review these concepts:

### Memory & Ownership Patterns

1. **Interior Mutability**:
   - When would you use `Cell` vs `RefCell` vs `Mutex` vs `RwLock`?
   - What's the runtime cost of each?
   - Why can interior mutability panic?

2. **Arena Allocation**:
   - What workloads benefit most from arena allocation?
   - What's the trade-off of arena vs individual allocations?
   - When does arena allocation hurt performance?

3. **Cow Patterns**:
   - When should a function return `Cow<T>` vs `T` vs `&T`?
   - How does `Cow` enable zero-copy optimization?
   - What's the caller's responsibility when receiving `Cow`?

4. **Lifetimes**:
   - Why do arena-allocated objects need lifetime annotations?
   - How do generational indices avoid lifetime issues?
   - When are lifetimes better than indices?

### Design Patterns

1. **When to use what**:
   - Single-threaded mutation: `RefCell`
   - Multi-threaded mutation: `Mutex` or `RwLock`
   - Bulk allocation/deallocation: `Arena`
   - Avoiding duplicate allocations: `Cow` or string interning
   - Stable handles: generational indices

2. **Performance Characteristics**:
   - `Cell::get/set`: zero cost
   - `RefCell`: runtime check overhead
   - `Mutex`: OS lock overhead + contention
   - `RwLock`: higher overhead than `Mutex`, but allows concurrent reads
   - Arena: extremely fast allocation, but can't free individual items

### Common Pitfalls

1. **Don't hold RefCell borrows across function calls** - causes panics
2. **Don't use `Arc<Mutex<T>>` for single-threaded code** - unnecessary overhead
3. **Don't intern everything** - has its own costs
4. **Don't ignore lock scope** - minimizing critical sections is important
5. **Don't assume arena is always faster** - measure for your workload

---

## Next Steps

- Implement the additional challenges from each project
- Read the corresponding chapter sections again
- Try combining patterns (e.g., thread-safe arena allocator)
- Profile your implementations to understand costs
- Explore real-world codebases using these patterns (Rust compiler, game engines, etc.)
