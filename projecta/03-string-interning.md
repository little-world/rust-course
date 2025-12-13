# String Interning

### Problem Statement

Build a string interning system that stores unique strings once and reuses them. This demonstrates Clone-on-Write (Cow) patterns and zero-copy optimization.

---

## Understanding String Interning

### What is String Interning?

**String interning** is a memory optimization technique where only one copy of each distinct string value is stored in memory. When you need the same string multiple times, instead of creating duplicate copies, you reuse a reference to the single stored instance.

**Simple Example:**

```rust
// Without interning (memory waste)
let name1 = String::from("Alice");  // Allocation #1
let name2 = String::from("Alice");  // Allocation #2 (duplicate!)
let name3 = String::from("Alice");  // Allocation #3 (duplicate!)
// Memory used: 3 allocations, 15 bytes total (5 bytes × 3)

// With interning (memory efficient)
let mut interner = StringInterner::new();
let name1 = interner.intern("Alice");  // Allocation #1
let name2 = interner.intern("Alice");  // Reuse! No allocation
let name3 = interner.intern("Alice");  // Reuse! No allocation
// Memory used: 1 allocation, 5 bytes, plus 3 pointers (24 bytes total on 64-bit)
// But pointers are often stack-allocated, so effective memory = 5 bytes!
```

### Core Concepts

#### 1. **Single Storage**
Each unique string is stored exactly once in a central repository (the "intern pool").

```
Intern Pool:
┌─────────────┐
│ "Alice"     │ ← Stored once
│ "Bob"       │ ← Stored once
│ "Charlie"   │ ← Stored once
└─────────────┘

References:
name1 → "Alice"
name2 → "Alice"  (same pointer)
name3 → "Bob"
name4 → "Alice"  (same pointer)
```

#### 2. **Identity-Based Equality**
Since identical strings have the same memory address, equality checks become pointer comparisons.

```rust
// String comparison: O(n) - must compare each character
if "Alice" == "Alice" {  // Checks: A==A, l==l, i==i, c==c, e==e
    // 5 comparisons
}

// Interned string comparison: O(1) - just compare pointers
if ptr1 == ptr2 {  // Single pointer comparison
    // 1 comparison
}
```

#### 3. **Deduplication**
When you try to intern a string that already exists, the interner returns the existing instance.

```rust
let mut interner = StringInterner::new();

let s1 = interner.intern("hello");  // New: stores "hello"
let s2 = interner.intern("world");  // New: stores "world"
let s3 = interner.intern("hello");  // Duplicate: returns existing "hello"

assert!(s1.as_ptr() == s3.as_ptr());  // Same memory address!
```

---

### Real-World Examples

#### String Interning in Practice

**1. Java String Pool**
```java
String s1 = "hello";        // Interned automatically
String s2 = "hello";        // Reuses interned string
assert(s1 == s2);           // true (same object)

String s3 = new String("hello");  // Not interned
assert(s1 == s3);                 // false (different objects)
```

**2. Python String Interning**
```python
s1 = "hello"
s2 = "hello"
assert(s1 is s2)  # True - Python interns short strings automatically
```

**3. Rust Compiler (rustc)**
- Interns all identifiers, keywords, and string literals
- Symbol table uses interned strings for fast lookup
- Reduces memory usage by ~15-20% during compilation

**4. V8 JavaScript Engine**
- Interns property names for objects
- Enables fast property lookup (pointer comparison)
- Critical for performance in property-heavy code

---

## Rust Programming Concepts for This Project

This project requires understanding several Rust-specific concepts related to smart pointers, borrowing patterns, and type system features. These concepts enable building memory-efficient systems with zero-cost abstractions.

### Cow: Clone-on-Write Smart Pointer

**The Core Problem**: Many functions sometimes need to modify their input, sometimes don't. How do you avoid unnecessary allocations in the "no modification needed" case?

**Wrong Approaches**:

```rust
// Approach 1: Always allocate (wasteful)
fn process(input: &str) -> String {
    input.to_string()  // Allocates even if unchanged!
}

// Approach 2: Try to return reference (doesn't compile)
fn process(input: &str) -> &str {
    if needs_modification(input) {
        return modified_string;  // ❌ Where does modified_string live?
    }
    input
}
```

**The Solution: `Cow<'a, B>`**

```rust
pub enum Cow<'a, B: ?Sized + 'a>
where
    B: ToOwned,
{
    Borrowed(&'a B),           // Zero-copy: points to existing data
    Owned(<B as ToOwned>::Owned),  // Allocated: owns the data
}
```

For strings, this becomes:
- `Cow::Borrowed(&str)` - Zero-copy reference to string
- `Cow::Owned(String)` - Heap-allocated string

**Key Characteristics**:

1. **Lifetime Parameter `'a`**: The borrowed variant holds a reference, so we need to track its lifetime
   ```rust
   fn process<'a>(input: &'a str) -> Cow<'a, str> {
       Cow::Borrowed(input)  // Lifetime of output tied to input
   }
   ```

2. **Trait Bound `B: ToOwned`**: The borrowed type must be convertible to an owned type
   ```rust
   // For str:
   impl ToOwned for str {
       type Owned = String;
       fn to_owned(&self) -> String { /* ... */ }
   }
   ```

3. **Smart Deref**: `Cow<str>` dereferences to `&str`, so you can use it like a string
   ```rust
   let cow: Cow<str> = Cow::Borrowed("hello");
   println!("{}", cow.len());  // Works! Derefs to &str
   assert_eq!(&*cow, "hello"); // Explicit deref
   ```

4. **Lazy Allocation**: Only allocate when mutation is needed
   ```rust
   let cow = Cow::Borrowed("test");
   let owned = cow.into_owned();  // Allocates String only now
   ```

**Common Cow Methods**:

```rust
impl<'a, B> Cow<'a, B> where B: ToOwned {
    // Convert to owned variant (may allocate)
    pub fn into_owned(self) -> <B as ToOwned>::Owned;

    // Get mutable reference (may allocate)
    pub fn to_mut(&mut self) -> &mut <B as ToOwned>::Owned;

    // Check if borrowed
    pub fn is_borrowed(&self) -> bool;

    // Check if owned
    pub fn is_owned(&self) -> bool;
}
```

**Pattern Matching on Cow**:

```rust
match cow {
    Cow::Borrowed(s) => println!("Zero-copy: {}", s),
    Cow::Owned(s) => println!("Allocated: {}", s),
}

// Or use if let
if let Cow::Owned(ref s) = cow {
    println!("This was allocated: {}", s);
}
```

**Performance Impact**:

```rust
// Without Cow (always allocate)
fn normalize(s: &str) -> String {
    s.trim().to_string()  // 100% allocation rate
}

// With Cow (allocate only when needed)
fn normalize(s: &str) -> Cow<str> {
    let trimmed = s.trim();
    if trimmed.len() == s.len() {
        Cow::Borrowed(s)  // 0% allocation for clean input
    } else {
        Cow::Owned(trimmed.to_string())  // Allocate only when trimmed
    }
}
```

If 90% of inputs are already clean:
- Without Cow: 100% allocations (10,000 inputs = 10,000 allocations)
- With Cow: 10% allocations (10,000 inputs = 1,000 allocations)
- **Result: 10x fewer allocations**

---

### Box<str> vs String: Choosing the Right String Type

Rust has multiple string types. Understanding when to use each is crucial for this project.

**The Three Main String Types**:

| Type | Owned? | Mutable? | Size on Stack | Use Case |
|------|--------|----------|---------------|----------|
| `&str` | No (borrowed) | No | 16 bytes (ptr + len) | Temporary references, function params |
| `String` | Yes | Yes | 24 bytes (ptr + len + capacity) | Growing strings, builder pattern |
| `Box<str>` | Yes | No | 16 bytes (ptr + len) | Fixed strings, interning |

**Why Box<str> for Interning?**

```rust
// String has 3 fields
struct String {
    ptr: *mut u8,      // 8 bytes
    len: usize,        // 8 bytes
    capacity: usize,   // 8 bytes - WASTED for interning!
}

// Box<str> has 2 fields (same as &str)
struct BoxStr {
    ptr: *const u8,    // 8 bytes
    len: usize,        // 8 bytes
}
```

**Key Insight**: Interned strings **never grow**, so capacity field is waste. `Box<str>` saves 8 bytes per string!

**Memory Comparison**:

```rust
// With String (24 bytes each + string data)
let strings: Vec<String> = vec![
    String::from("hello"),  // 24 + 5 = 29 bytes
    String::from("world"),  // 24 + 5 = 29 bytes
];
// Total: 58 bytes + Vec overhead

// With Box<str> (16 bytes each + string data)
let strings: Vec<Box<str>> = vec![
    Box::from("hello"),  // 16 + 5 = 21 bytes
    Box::from("world"),  // 16 + 5 = 21 bytes
];
// Total: 42 bytes + Vec overhead
// Savings: 16 bytes (27% less!)
```

For 10,000 interned strings averaging 20 bytes each:
- With `String`: 10,000 × (24 + 20) = 440 KB
- With `Box<str>`: 10,000 × (16 + 20) = 360 KB
- **Savings: 80 KB (18% reduction)**

**Converting Between Types**:

```rust
// str → String
let s: String = "hello".to_string();
let s: String = "hello".to_owned();
let s: String = String::from("hello");

// str → Box<str>
let b: Box<str> = Box::from("hello");
let b: Box<str> = "hello".into();

// String → Box<str> (drops capacity)
let s = String::from("hello");
let b: Box<str> = s.into_boxed_str();

// Box<str> → String (realloc with capacity)
let b: Box<str> = Box::from("hello");
let s: String = b.into();
```

**When to Use Each**:

✅ **Use `&str` when**:
- Temporary references
- Function parameters
- Slicing existing strings

✅ **Use `String` when**:
- Building strings incrementally (`push`, `push_str`)
- Modifying strings (`replace`, `insert`)
- Unknown final size

✅ **Use `Box<str>` when**:
- Fixed, immutable strings
- String interning (this project!)
- Minimizing memory footprint
- Storage in collections where size matters

---

### HashSet and Hashing: Fast Lookup Data Structure

**What is a HashSet?**

A `HashSet<T>` is a collection that:
- Stores unique values (no duplicates)
- Provides O(1) average-case lookup, insert, remove
- Uses hashing to achieve speed

**How Hashing Works**:

```
1. Hash the value: "hello" → hash("hello") → 5863208
2. Index into buckets: 5863208 % bucket_count → bucket 24
3. Store/lookup in bucket 24
```

**Visual Example**:

```
HashSet<&str> with 8 buckets:
┌─────────┬──────────────┐
│ Bucket 0│              │
│ Bucket 1│ "world" ───┐ │
│ Bucket 2│              │
│ Bucket 3│ "hello" ───┤ │  (hash("hello") % 8 = 3)
│ Bucket 4│              │
│ Bucket 5│ "foo" ─────┤ │
│ Bucket 6│              │
│ Bucket 7│ "bar" ─────┘ │
└─────────┴──────────────┘
```

**Key HashSet Operations**:

```rust
use std::collections::HashSet;

let mut set = HashSet::new();

// Insert (returns true if new, false if duplicate)
assert!(set.insert("hello"));   // true - new
assert!(!set.insert("hello"));  // false - duplicate

// Contains (O(1) average)
assert!(set.contains("hello"));

// Get (returns reference to stored value)
let stored: &str = set.get("hello").unwrap();

// Remove
set.remove("hello");

// Iterate
for s in &set {
    println!("{}", s);
}
```

**The Magic of `HashSet::get()`**:

This is crucial for string interning!

```rust
let mut set: HashSet<Box<str>> = HashSet::new();
set.insert(Box::from("hello"));

// Input: &str reference (temporary)
let input: &str = "hello";

// get() returns: reference to the Box<str> INSIDE the set!
let stored: &Box<str> = set.get(input).unwrap();

// We can then return a &str pointing INTO that Box
let interned: &str = stored.as_ref();
```

**Why This Works for Interning**:

```rust
fn intern(&mut self, s: &str) -> &str {
    // 1. Check if already stored
    if !self.strings.contains(s) {
        self.strings.insert(Box::from(s));  // Allocate and store
    }

    // 2. Return reference to stored value (not input!)
    // This reference lives as long as the HashSet
    self.strings.get(s).unwrap().as_ref()
}
```

The lifetime magic:
- Input `s: &str` has a short lifetime
- Returned `&str` has the lifetime of `&self`
- The string lives in the `HashSet`, so it outlives the input

**Hash Trait Requirements**:

For `HashSet<T>`, type `T` must implement:
- `Hash` - can be hashed to a number
- `Eq` - can be compared for equality

```rust
// String types implement these automatically:
impl Hash for str { /* ... */ }
impl Hash for String { /* ... */ }
impl<T: Hash> Hash for Box<T> { /* ... */ }

// Your custom types need manual impl or derive:
#[derive(Hash, Eq, PartialEq)]
struct Symbol {
    index: usize,
    generation: u32,
}
```

**HashSet Performance**:

| Operation | Average Case | Worst Case |
|-----------|--------------|------------|
| Insert | O(1) | O(n) |
| Lookup | O(1) | O(n) |
| Remove | O(1) | O(n) |

Worst case happens with hash collisions (rare with good hash function).

**HashSet vs Vec for String Interning**:

```rust
// Vec approach (linear search - O(n))
fn intern_vec(&mut self, s: &str) -> &str {
    for existing in &self.strings {
        if existing.as_ref() == s {
            return existing;  // Found! But took O(n) time
        }
    }
    // Not found, add it
    self.strings.push(Box::from(s));
    self.strings.last().unwrap()
}

// HashSet approach (hash lookup - O(1))
fn intern_hashset(&mut self, s: &str) -> &str {
    if !self.strings.contains(s) {  // O(1)
        self.strings.insert(Box::from(s));
    }
    self.strings.get(s).unwrap()  // O(1)
}
```

For 1,000 strings:
- Vec: Average 500 comparisons per lookup = **500n string comparisons**
- HashSet: 1 hash + ~1 comparison = **~n operations total**
- **Speedup: 500x faster!**

---

### Generational Indices: Safe Handles Without Lifetimes

**The Problem**: References (`&str`) carry lifetime annotations that infect everything:

```rust
struct Compiler<'intern> {
    identifiers: Vec<&'intern str>,  // ❌ Lifetime everywhere
    interner: &'intern StringInterner,
}
```

This makes code complex and limits flexibility (can't easily serialize, send between threads, etc.).

**The Solution**: **Generational Indices** (also called "slot map pattern")

Replace references with `Copy` handles that contain an index and a generation:

```rust
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct Symbol {
    index: usize,     // Which slot?
    generation: u32,  // Which version of that slot?
}
```

**How It Works**:

```
Storage:
  slots: Vec<Slot>
  free_list: Vec<usize>

Slot:
  data: Option<T>     // None = free slot
  generation: u32     // Increments on reuse

Symbol:
  index: usize        // Points to slot
  generation: u32     // Must match slot's generation
```

**Example Lifecycle**:

```rust
let mut interner = SymbolInterner::new();

// 1. Allocate "hello" → slot 0
let sym1 = interner.intern("hello");
// sym1 = Symbol{index: 0, generation: 0}
// slots[0] = Slot{data: Some("hello"), generation: 0}

// 2. Remove "hello"
interner.remove(sym1);
// slots[0] = Slot{data: None, generation: 1}  ← generation incremented!
// free_list = [0]

// 3. Try to use old symbol → returns None!
assert_eq!(interner.resolve(sym1), None);
// sym1 has gen=0, but slot has gen=1 → mismatch!

// 4. Allocate "world" → reuses slot 0
let sym2 = interner.intern("world");
// sym2 = Symbol{index: 0, generation: 1}
// slots[0] = Slot{data: Some("world"), generation: 1}

// 5. Old symbol still doesn't work
assert_eq!(interner.resolve(sym1), None);  // Still gen=0
assert_eq!(interner.resolve(sym2), Some("world"));  // gen=1 matches
```

**Why Generations Prevent Bugs**:

Without generations:
```rust
let sym1 = interner.intern("hello");  // Symbol{index: 0}
interner.remove(sym1);
let sym2 = interner.intern("world");  // Reuses index 0

// BUG: sym1 now resolves to "world"!
assert_eq!(interner.resolve(sym1), Some("world"));  // ❌ WRONG!
```

With generations:
```rust
let sym1 = interner.intern("hello");  // Symbol{index: 0, gen: 0}
interner.remove(sym1);                 // Slot becomes gen 1
let sym2 = interner.intern("world");  // Symbol{index: 0, gen: 1}

// sym1 is stale → safely returns None
assert_eq!(interner.resolve(sym1), None);  // ✅ Detected stale!
```

**Benefits Over References**:

| Aspect | `&'a str` | `Symbol` |
|--------|-----------|----------|
| **Lifetime annotations** | Required everywhere | None (Copy, 'static) |
| **Struct storage** | Infects struct with `'a` | Clean, no lifetimes |
| **Serialization** | Impossible | Easy (just two numbers) |
| **Threading** | Complex lifetime bounds | Simple (Send + Sync) |
| **Flexibility** | Limited by borrow checker | Store anywhere |
| **Safety** | Compile-time | Runtime (generation check) |
| **Speed** | Direct pointer (~1ns) | Indirect lookup (~3ns) |
| **Size** | 8 bytes | 12 bytes |

**When to Use Generational Indices**:

✅ **Use generational indices when**:
- Building complex data structures (graphs, trees)
- Need to store handles in multiple places
- Want to serialize/deserialize
- Working with concurrent code
- Lifetimes become too complex

❌ **Use references when**:
- Simple, short-lived usage
- Performance-critical tight loops
- Borrow checker isn't a burden

**Implementation Pattern**:

```rust
struct Slot<T> {
    data: Option<T>,
    generation: u32,
}

struct Arena<T> {
    slots: Vec<Slot<T>>,
    free_list: Vec<usize>,
}

impl<T> Arena<T> {
    fn insert(&mut self, value: T) -> Handle {
        let index = if let Some(index) = self.free_list.pop() {
            let slot = &mut self.slots[index];
            slot.generation += 1;  // Increment on reuse
            slot.data = Some(value);
            index
        } else {
            let index = self.slots.len();
            self.slots.push(Slot {
                data: Some(value),
                generation: 0,
            });
            index
        };

        Handle {
            index,
            generation: self.slots[index].generation,
        }
    }

    fn get(&self, handle: Handle) -> Option<&T> {
        self.slots.get(handle.index).and_then(|slot| {
            if slot.generation == handle.generation {
                slot.data.as_ref()
            } else {
                None  // Stale handle
            }
        })
    }
}
```

---

### Option Type: Safe Null Handling

Rust doesn't have null. Instead, it uses `Option<T>` to represent "might not have a value."

**The Option Enum**:

```rust
pub enum Option<T> {
    Some(T),  // Has a value
    None,     // No value
}
```

**Why No Null?**

In languages with null:
```java
String s = getString();  // Might return null
int len = s.length();    // ❌ NullPointerException!
```

In Rust:
```rust
let s: Option<String> = get_string();  // Explicit!
let len = s.length();  // ❌ Compile error: Option has no length method

// Must explicitly handle None case:
let len = s.unwrap().len();  // Panics if None
let len = s.expect("no string").len();  // Panics with message
let len = s?.len();  // Returns early if None
match s {
    Some(str) => str.len(),
    None => 0,  // Handle explicitly
}
```

**Common Option Methods**:

```rust
let opt: Option<i32> = Some(42);

// Extract value (panics if None)
opt.unwrap();  // 42

// Extract or provide default
opt.unwrap_or(0);  // 42
opt.unwrap_or_else(|| expensive_default());

// Check if Some or None
opt.is_some();  // true
opt.is_none();  // false

// Transform the value inside
opt.map(|x| x * 2);  // Some(84)

// Chain operations
opt.and_then(|x| Some(x * 2));  // Some(84)

// Pattern matching
match opt {
    Some(val) => println!("{}", val),
    None => println!("nothing"),
}
```

**Using Option in String Interning**:

```rust
fn resolve(&self, symbol: Symbol) -> Option<&str> {
    // Get the slot (might not exist)
    let slot = self.slots.get(symbol.index)?;  // Return None if out of bounds

    // Check generation matches
    if slot.generation != symbol.generation {
        return None;  // Stale symbol
    }

    // Return the string (might be None if slot freed)
    slot.string.as_ref().map(|s| s.as_ref())
}
```

**Option Combinators for Chaining**:

```rust
// Without combinators (verbose)
fn resolve(&self, symbol: Symbol) -> Option<&str> {
    if let Some(slot) = self.slots.get(symbol.index) {
        if slot.generation == symbol.generation {
            if let Some(string) = &slot.string {
                return Some(string.as_ref());
            }
        }
    }
    None
}

// With combinators (concise)
fn resolve(&self, symbol: Symbol) -> Option<&str> {
    self.slots
        .get(symbol.index)
        .filter(|slot| slot.generation == symbol.generation)
        .and_then(|slot| slot.string.as_ref().map(|s| s.as_ref()))
}
```

---

### Copy Trait: Understanding Value Semantics

**The Copy Trait**:

```rust
pub trait Copy: Clone { }
```

Types that implement `Copy` are duplicated on assignment:

```rust
// i32 is Copy
let x = 42;
let y = x;  // x is copied to y
println!("{}", x);  // ✅ x still valid

// String is NOT Copy
let s1 = String::from("hello");
let s2 = s1;  // s1 is moved to s2
println!("{}", s1);  // ❌ Error: s1 was moved
```

**Copy Requirements**:

A type can be `Copy` only if:
1. All fields are `Copy`
2. No custom `Drop` implementation (no cleanup needed)
3. It's safe to duplicate by just copying bits

**Copy Types in This Project**:

```rust
// Symbol is Copy (all fields are Copy)
#[derive(Copy, Clone)]
struct Symbol {
    index: usize,     // Copy
    generation: u32,  // Copy
}

// Can freely duplicate
let sym1 = Symbol { index: 0, generation: 0 };
let sym2 = sym1;  // Copied
let sym3 = sym1;  // Can use sym1 again!
```

**Why Symbol Needs Copy**:

```rust
// With Copy
let sym = interner.intern("hello");
identifiers.push(sym);  // Copied
keywords.insert(sym);   // Can still use sym!
write_to_file(sym);     // Can still use sym!

// Without Copy (if Symbol contained String)
let sym = interner.intern("hello");
identifiers.push(sym);  // Moved!
// Can't use sym anymore ❌
```

**Copy vs Clone**:

- **Copy**: Implicit, done on assignment, cheap (just copy bits)
- **Clone**: Explicit, must call `.clone()`, might be expensive

```rust
// Copy (implicit)
let x = 42;
let y = x;  // Automatic copy

// Clone (explicit)
let s1 = String::from("hello");
let s2 = s1.clone();  // Must explicitly clone
```

---

### 'static Lifetime: Global and Owned Data

The `'static` lifetime is special—it means "lives for the entire program duration."

**Two Meanings of 'static**:

1. **String literals and constants**:
   ```rust
   let s: &'static str = "hello";  // Lives in program binary
   static MAX: i32 = 100;          // Lives forever
   ```

2. **No borrowed data** (owned or Copy types):
   ```rust
   // Symbol is 'static because it's Copy (no references)
   fn store(sym: Symbol) {
       GLOBAL.lock().unwrap().push(sym);  // OK!
   }

   // &str is NOT 'static (has lifetime 'a)
   fn store<'a>(s: &'a str) {
       GLOBAL.lock().unwrap().push(s);  // ❌ Won't compile
   }
   ```

**Why Symbol is 'static**:

```rust
#[derive(Copy)]
struct Symbol {
    index: usize,
    generation: u32,
}
// No references = no lifetime = automatically 'static
```

This means Symbol can:
- Be stored in global variables
- Be sent across threads freely
- Outlive any particular scope
- Be returned from any function without lifetime annotations

**Contrast with References**:

```rust
// Reference approach
fn intern<'a>(&'a mut self, s: &str) -> &'a str;
// Returns reference with lifetime 'a

// Symbol approach
fn intern(&mut self, s: &str) -> Symbol;
// Returns Copy value with 'static lifetime
```

---

### Connection to This Project

In this project, you're building a **string interning system**,

1. **Intrinsic State**: The string content itself (shared)
2. **Extrinsic State**: Where the string is used (not stored in interner)
3. **Factory**: The `StringInterner` struct manages the string pool
4. **Optimization**: Cow pattern enables zero-copy when strings are already interned

**What You'll Build:**
```rust
pub struct StringInterner {
    pool: HashSet<String>,  // The flyweight pool for strings
}

impl StringInterner {
    pub fn intern<'a>(&'a mut self, s: &str) -> Cow<'a, str> {
        // Flyweight factory pattern:
        // - Check if string exists (lookup intrinsic state)
        // - If yes: return reference (reuse flyweight)
        // - If no: store and return reference (create flyweight)
    }
}
```


**Performance Benefits**:
1. **Memory**: 10-40% reduction in string memory for identifier-heavy workloads
2. **Comparison**: `O(1)` pointer equality vs `O(n)` string comparison
3. **Hashing**: Hash once, reuse hash value (important for HashMaps)
4. **Cache**: Fewer unique strings = better cache locality

**Cow Pattern Benefits**:
- **Zero-copy**: If string already interned, return borrowed reference (no allocation)
- **Lazy allocation**: Only allocate when necessary
- **API clarity**: Caller knows if allocation happened by checking `Cow` variant

---

### Milestone 1: Understand Cow Basics

 Learn how `Cow` (Clone-on-Write) works through hands-on examples that demonstrate zero-copy optimization.


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


**Architecture**:

**functions**:
- `normalize_whitespace(text: &str) -> Cow<str>`
    - Checks for double spaces or tabs
    - If found: replace with single spaces (allocate)
    - If not found: return original (zero-copy)

- `maybe_escape_html(text: &str) -> Cow<str>`
    - Checks for `<`, `>`, `&` characters
    - If found: escape to `&lt;`, `&gt;`, `&amp;` (allocate)
    - If not found: return original (zero-copy)



**Starter Code**:
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

#### Why Milestone 1 Isn't Enough

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

Implement a string interner that stores each unique string once and returns references to deduplicated storage.


Now that we understand `Cow` for conditional allocation, let's tackle a bigger problem: **string duplication across your entire program**. String interning is a powerful technique that trades lookup time for dramatic memory savings.

**What is String Interning?**

String interning is a technique where:
1. **Unique strings stored once**: First occurrence allocates and stores
2. **Duplicates return references**: Subsequent occurrences return pointer to existing storage
3. **Pointer equality works**: Can compare strings with `ptr::eq()` instead of `strcmp()`

**Architecture**:

**struct**:
```rust
struct StringInterner {
    strings: HashSet<Box<str>>,  // Set of unique strings
}
```
**functions**:
- `new() -> Self` - Create empty interner with empty HashSet
- `intern(&mut self, s: &str) -> &str` - Add string to set if new, return reference
- `contains(&self, s: &str) -> bool` - Check if string is interned
- `len(&self) -> usize` - Number of unique strings stored
- `total_bytes(&self) -> usize` - Total bytes used by all strings



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

 Combine the `Cow` pattern from Milestone 1 with the interner from Milestone 2 to create an API that communicates allocation status.


In Milestone 1, we learned that `Cow` communicates **"did we allocate or not?"** to the caller. In Milestone 2, we built an interner but lost that information—`intern()` always returns `&str`, hiding whether allocation happened.


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

**Why always `Cow::Borrowed`?**

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



**Starter Code**:
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
 Add comprehensive statistics to measure interner effectiveness and understand allocation patterns.

As we learned in Milestone 3, `Cow` isn't the ideal way to track string interner performance. What we really need is **aggregate statistics** that answer questions like:

- **Is the interner effective?** High hit rate (lookups/total) = good reuse!
- **Should we use interning?** If allocation rate is too low, overhead might not be worth it
- **Memory saved**: Compare `total_bytes` vs. `(allocations + lookups) × average_length`
- **Performance tuning**: Identify which strings are duplicated most

**Architecture**:

Without stats, you're flying blind. You don't know if the interner is paying for itself!

**struct**:
- `InternerStats` 

```rust
struct InternerStats {
    total_strings: usize,   // Unique strings currently stored
    total_bytes: usize,     // Total bytes used by strings
    allocations: usize,     // How many new strings added
    lookups: usize,         // How many duplicate strings found
}
```
**functions**:
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

#### Why Milestone 4 Isn't Enough 

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

Replace lifetime-bound references with `Copy` handles that work anywhere, using the generational index pattern to detect stale handles safely.


**Architecture**:

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


**functions**:

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
## Complete Working Example

