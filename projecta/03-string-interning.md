# String Interning

### Problem Statement

Build a string interning system that stores unique strings once and reuses them. This demonstrates Clone-on-Write (Cow) patterns, zero-copy optimization, and a critical Rust pattern: escaping the borrow checker with raw pointers.

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

// s1 and s3 point to the same memory!
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

### The Borrow Checker Problem with String Interning

Before we dive into the implementation, let's understand a critical challenge that makes string interning tricky in Rust.

**The Naive Approach (Doesn't Work):**

```rust
impl StringInterner {
    fn intern(&mut self, s: &str) -> &str {
        // Returns a reference tied to &mut self
        self.strings.get(s).unwrap()
    }
}

fn main() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello");  // Borrows interner mutably
    let s2 = interner.intern("hello");  // ❌ ERROR: interner already borrowed!

    // Can't even compare them!
    if s1 == s2 { }  // Still borrowing
}
```

**Why Does This Fail?**

The signature `fn intern(&mut self, s: &str) -> &str` means:
- The returned `&str` borrows from `self`
- As long as `s1` exists, `interner` is borrowed
- You cannot call `intern()` again while `s1` is alive

This is the borrow checker doing its job—preventing data races and dangling references. But it makes string interning practically useless!

**The Solution: Raw Pointers**

```rust
let s1 = interner.intern("hello") as *const str;
let s2 = interner.intern("hello") as *const str;

// Now both calls work!
assert!(std::ptr::eq(s1, s2));  // Same pointer!
```

By casting to `*const str`, we:
1. Convert the borrowed `&str` to a raw pointer
2. The borrow ends at the end of the statement
3. We can call `intern()` again immediately

---

### Understanding `*const str`: Raw Pointers to String Slices

A raw pointer `*const str` is Rust's "escape hatch" from the borrow checker. Let's understand what it is and why we need it.

**What is `*const str`?**

```rust
// &str is a "fat pointer": pointer + length
// *const str is also a "fat pointer": pointer + length, but without borrow tracking

let s: &str = "hello";           // Borrowed reference with lifetime
let ptr: *const str = s;         // Raw pointer, no lifetime tracking

// Or explicitly cast:
let ptr: *const str = s as *const str;
```

**Memory Layout:**

```
&str (borrowed reference):
┌──────────────────────┬──────────────┐
│ pointer (8 bytes)    │ len (8 bytes)│ + lifetime tracking by compiler
└──────────────────────┴──────────────┘

*const str (raw pointer):
┌──────────────────────┬──────────────┐
│ pointer (8 bytes)    │ len (8 bytes)│  NO lifetime tracking
└──────────────────────┴──────────────┘
```

**Key Properties of `*const str`:**

| Property | `&str` | `*const str` |
|----------|--------|--------------|
| Lifetime tracking | Yes (compile-time) | No |
| Borrow checking | Yes | No |
| Copy | Yes | Yes |
| Null possible | No | Yes |
| Dereferencing | Safe | Unsafe |
| Size | 16 bytes | 16 bytes |

**Why `*const str` Solves Our Problem:**

```rust
fn intern(&mut self, s: &str) -> &str {
    // Returns &str tied to &mut self
    self.strings.get(s).unwrap()
}

// Without casting - DOESN'T COMPILE:
let s1 = interner.intern("hello");     // Borrow starts
let s2 = interner.intern("hello");     // ❌ Can't borrow again!
//       ^^^^^^^^ still borrowed here

// With casting - WORKS:
let s1 = interner.intern("hello") as *const str;  // Borrow ends at semicolon
let s2 = interner.intern("hello") as *const str;  // New borrow, no conflict
// Both s1 and s2 are now *const str - the borrow checker ignores them
```

**The Conversion Flow:**

```rust
interner.intern("hello")     // Step 1: Returns &str (borrows interner)
    as *const str            // Step 2: Convert to raw pointer
;                            // Step 3: Borrow ends here!

// Next line: interner is no longer borrowed
```

---

### Safety Considerations for `*const str`

Converting to `*const str` is **not unsafe** (the conversion itself is safe). However, **using** the raw pointer requires care:

**Safe Operations (no `unsafe` needed):**

```rust
let s1 = interner.intern("hello") as *const str;
let s2 = interner.intern("hello") as *const str;

// Pointer comparison - SAFE
assert!(std::ptr::eq(s1, s2));

// Checking for null - SAFE
assert!(!s1.is_null());

// Storing in a Vec - SAFE
let mut pointers: Vec<*const str> = vec![s1, s2];
```

**Unsafe Operations (require `unsafe` block):**

```rust
let ptr = interner.intern("hello") as *const str;

// Dereferencing - UNSAFE
unsafe {
    let s: &str = &*ptr;  // Convert back to reference
    println!("{}", s);
}
```

**When is Dereferencing Safe?**

The pointer is valid as long as:
1. The interner hasn't been dropped
2. The interner hasn't reallocated (for Vec-based storage)
3. The string hasn't been removed from the interner

```rust
// SAFE: Interner still exists, no reallocation
let ptr = interner.intern("hello") as *const str;
unsafe { println!("{}", &*ptr); }  // ✅ OK

// DANGEROUS: Interner dropped
let ptr = interner.intern("hello") as *const str;
drop(interner);
unsafe { println!("{}", &*ptr); }  // ❌ UNDEFINED BEHAVIOR!

// DANGEROUS: Potential reallocation (for Vec-based storage)
let ptr = interner.intern("hello") as *const str;
for i in 0..1000 {
    interner.intern(&format!("string{}", i));  // May reallocate!
}
unsafe { println!("{}", &*ptr); }  // ❌ MIGHT BE DANGLING!
```

**HashSet-Based Storage is Safer:**

Using `HashSet<Box<str>>` instead of `Vec<String>` is safer because:
- Each string is in its own heap allocation (`Box<str>`)
- Strings don't move when the HashSet grows
- Only the bucket pointers move, not the strings themselves

```rust
// HashSet<Box<str>> - strings are stable
let ptr = interner.intern("hello") as *const str;
interner.intern("world");  // HashSet grows, but "hello" doesn't move
unsafe { println!("{}", &*ptr); }  // ✅ Still safe!
```

---

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

---

### Generational Indices: Safe Handles Without Lifetimes

**The Problem**: Raw pointers (`*const str`) work, but they're inherently unsafe. References (`&str`) carry lifetime annotations that infect everything:

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

**Example**:

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

**Benefits Over Raw Pointers**:

| Aspect | `*const str` | `Symbol` |
|--------|--------------|----------|
| **Safety** | Unsafe to dereference | Safe (generation check) |
| **Dangling detection** | None (UB risk) | Returns `None` for stale |
| **Lifetime annotations** | None | None |
| **Serialization** | Impossible | Easy (just two numbers) |
| **Threading** | Dangerous | Simple (Send + Sync) |
| **Speed** | Direct pointer (~1ns) | Indirect lookup (~3ns) |
| **Size** | 16 bytes | 12 bytes |

---

## Build the Project

In this project, you're building a **string interning system**:

1. **Intrinsic State**: The string content itself (shared)
2. **Extrinsic State**: Where the string is used (not stored in interner)
3. **Factory**: The `StringInterner` struct manages the string pool
4. **Optimization**: Cow pattern enables zero-copy when strings are already interned

**What You'll Build:**
```rust
pub struct StringInterner {
    pool: HashSet<Box<str>>,  // The flyweight pool for strings
}

impl StringInterner {
    pub fn intern(&mut self, s: &str) -> &str {
        // Store if new, return reference to stored string
    }
}

// Usage with raw pointers to escape borrow checker:
let s1 = interner.intern("hello") as *const str;
let s2 = interner.intern("hello") as *const str;
assert!(std::ptr::eq(s1, s2));  // Same pointer!
```


**Performance Benefits**:
1. **Memory**: 10-40% reduction in string memory for identifier-heavy workloads
2. **Comparison**: `O(1)` pointer equality vs `O(n)` string comparison
3. **Hashing**: Hash once, reuse hash value (important for HashMaps)
4. **Cache**: Fewer unique strings = better cache locality

---


### Milestone 1: Understand Cow Basics

 Learn how `Cow` (Clone-on-Write) works through hands-on examples that demonstrate zero-copy optimization.


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

### Milestone 2: Basic String Interner with Raw Pointers

Implement a string interner that stores each unique string once. Because `intern()` returns `&str` tied to `&mut self`, we must convert to `*const str` to use multiple interned strings together.

**The Core Problem: Borrow Checker vs Practical Usage**

```rust
// This is what we want to write:
let s1 = interner.intern("hello");
let s2 = interner.intern("hello");
assert!(std::ptr::eq(s1, s2));  // Compare pointers

// But this DOESN'T COMPILE because:
// - intern() takes &mut self
// - Returns &str tied to that borrow
// - Can't call intern() again while s1 exists!
```

**The Solution: Cast to `*const str`**

```rust
// This WORKS:
let s1 = interner.intern("hello") as *const str;
let s2 = interner.intern("hello") as *const str;
assert!(std::ptr::eq(s1, s2));  // ✅ Compiles and works!
```

**Why Does This Work?**

```rust
interner.intern("hello")  // Returns &str, borrows &mut self
    as *const str         // Converts to raw pointer (Copy, no borrow tracking)
;                         // Borrow of interner ENDS here

// Now interner is free to be borrowed again!
interner.intern("hello") as *const str;  // New borrow, no conflict
```

The key insight: `as *const str` "forgets" the borrow. The raw pointer is `Copy` and has no lifetime parameter, so the compiler stops tracking it.

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

**Using Raw Pointers Safely**:

```rust
let mut interner = StringInterner::new();

// Convert to *const str immediately to escape the borrow
let s1 = interner.intern("hello") as *const str;
let s2 = interner.intern("hello") as *const str;

// Pointer comparison is SAFE (no dereferencing)
assert!(std::ptr::eq(s1, s2));

// To actually USE the string, you need unsafe:
unsafe {
    let str1: &str = &*s1;  // Dereference raw pointer
    println!("{}", str1);
}
```

**When is the Raw Pointer Valid?**

The `*const str` is valid as long as:
1. The `StringInterner` hasn't been dropped
2. The string hasn't been removed from the interner
3. (With HashSet<Box<str>>, individual strings don't move even if HashSet grows)

```rust
// ✅ SAFE: Interner still exists
let ptr = interner.intern("hello") as *const str;
interner.intern("world");  // HashSet may grow, but "hello" doesn't move
unsafe { println!("{}", &*ptr); }  // Still valid!

// ❌ UNSAFE: Interner dropped
let ptr = interner.intern("hello") as *const str;
drop(interner);
unsafe { println!("{}", &*ptr); }  // UNDEFINED BEHAVIOR!
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
        // TODO: Check if string already in set
        // TODO: First time seeing this string - allocate and store
        // TODO: Return reference to the string in the set
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
        todo!()
    }
}
```


**Checkpoint Tests**:
```rust
#[test]
fn test_intern_basic() {
    let mut interner = StringInterner::new();

    // Must cast to *const str to escape borrow checker!
    let s1 = interner.intern("hello") as *const str;
    let s2 = interner.intern("hello") as *const str;

    // Should be same pointer (no second allocation)
    assert!(std::ptr::eq(s1, s2));
    assert_eq!(interner.len(), 1);
}

#[test]
fn test_intern_different() {
    let mut interner = StringInterner::new();

    let s1 = interner.intern("hello") as *const str;
    let s2 = interner.intern("world") as *const str;

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

#[test]
fn test_pointer_stability() {
    let mut interner = StringInterner::new();

    // Get pointer to first string
    let ptr1 = interner.intern("first") as *const str;

    // Add many more strings (may cause HashSet to resize)
    for i in 0..100 {
        interner.intern(&format!("string{}", i));
    }

    // Original pointer should still be valid (Box<str> doesn't move)
    let ptr1_again = interner.intern("first") as *const str;
    assert!(std::ptr::eq(ptr1, ptr1_again));
}
```

**Check Your Understanding**:
- Why do we use `Box<str>` instead of `String`?
- Why must we cast to `*const str` after calling `intern()`?
- What makes the pointers equal for the same string?
- When is it safe to dereference the raw pointer?

---

### Milestone 3: Add Statistics Tracking

Add comprehensive statistics to measure interner effectiveness and understand allocation patterns.

Since we're working with raw pointers, we can't easily communicate "did this allocate?" through the return type. Instead, we track **aggregate statistics**:

- **Is the interner effective?** High hit rate (lookups/total) = good reuse!
- **Should we use interning?** If allocation rate is too low, overhead might not be worth it
- **Memory saved**: Compare `total_bytes` vs. `(allocations + lookups) × average_length`

**Architecture**:

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

**Hit Rate Calculation**:

```rust
impl InternerStats {
    fn hit_rate(&self) -> f64 {
        let total = self.allocations + self.lookups;
        if total == 0 { 0.0 } else { self.lookups as f64 / total as f64 }
    }
}
```

A hit rate of 0.9 (90%) means 90% of `intern()` calls found an existing string—great reuse!

**Performance Cost of Statistics**:

Adding stats is cheap:
- Increment counters: ~1ns each (just memory writes)
- String length: already computed for HashSet
- No allocations, no complex computation

The cost is negligible compared to the HashSet lookup (~50ns)

**Starter Code**:
```rust
#[derive(Debug, Default, PartialEq)]
struct InternerStats {
    total_strings: usize,
    total_bytes: usize,
    allocations: usize,  // How many times we allocated
    lookups: usize,      // How many times we just returned existing
}

struct StringInterner {
    strings: HashSet<Box<str>>,
    stats: InternerStats,
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

#[test]
fn test_hit_rate() {
    let mut interner = StringInterner::new();

    interner.intern("a");  // alloc
    interner.intern("a");  // lookup
    interner.intern("a");  // lookup
    interner.intern("b");  // alloc

    let stats = interner.statistics();
    // 2 lookups / 4 total = 50% hit rate
    let hit_rate = stats.lookups as f64 / (stats.allocations + stats.lookups) as f64;
    assert!((hit_rate - 0.5).abs() < 0.001);
}
```

**Check Your Understanding**:
- Why track both allocations and lookups?
- How does hit rate help evaluate interner effectiveness?

---

### Why Raw Pointers Aren't Enough

Our interner with raw pointers has some critical limitations:

**1. Unsafe to Dereference**:
```rust
let ptr = interner.intern("hello") as *const str;

// Every time you want to USE the string:
let s: &str = unsafe { &*ptr };  // Unsafe block required!
```

**2. No Stale Detection**:
```rust
let ptr = interner.intern("hello") as *const str;
drop(interner);  // Interner gone!

// ptr is now dangling - no way to detect this!
unsafe { println!("{}", &*ptr); }  // UNDEFINED BEHAVIOR
```

**3. Lifetime Not Tracked**:
```rust
// Compiler can't help you catch bugs:
fn get_interned() -> *const str {
    let interner = StringInterner::new();
    interner.intern("hello") as *const str
    // interner dropped here! Pointer is dangling!
}
```

**The Solution: Generational Indices**

Instead of raw pointers, use a `Symbol` handle with an index and generation:

```rust
#[derive(Copy, Clone, PartialEq)]
struct Symbol {
    index: usize,
    generation: u32,
}
```

Benefits:
- **Safe API**: No `unsafe` needed for normal usage
- **Stale detection**: Generation mismatch → returns `None`
- **Serializable**: Just two numbers, can save to disk
- **Thread-safe**: `Copy`, no lifetime complexity

---

### Milestone 4: Symbol-Based Access with Generational Indices

Replace raw pointers with safe `Symbol` handles that detect stale references at runtime.

**Architecture**:

Instead of returning `&str` (which we cast to `*const str`), return a `Symbol` handle (no lifetime):

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
2. **Resolve**: Look up `slots[index]`, check generation matches, return `Option<&str>`
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

**Comparison: Raw Pointers vs Symbols**:

| Aspect | `*const str` | `Symbol` |
|--------|--------------|----------|
| **Safety** | Unsafe to dereference | Safe (returns `Option`) |
| **Stale detection** | None (UB risk) | Generation check → `None` |
| **Borrow checker** | Escaped | No borrows needed |
| **Serialization** | Impossible | Easy (two numbers) |
| **Resolve speed** | ~1ns (direct) | ~3ns (lookup) |
| **Size** | 16 bytes | 12 bytes |

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
        // TODO: If matches:
        //   - Set string to None
        //   - Increment generation
        //   - Push index to free_list
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
fn test_symbol_is_copy() {
    let mut interner = SymbolInterner::new();
    let sym = interner.intern("test");

    // Symbol is Copy - can use it multiple times without moving
    let sym_copy = sym;
    assert_eq!(interner.resolve(sym), Some("test"));
    assert_eq!(interner.resolve(sym_copy), Some("test"));
}
```

**Check Your Understanding**:
- Why use Symbols instead of raw pointers?
- How do generational indices detect stale references?
- What's the advantage of reusing slots with free_list?
- Why is Symbol Copy but still safe?

---

### Milestone 5: Performance Comparison

**Goal**: Measure the benefit of interning vs raw allocation.

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
    println!("Hit rate: {:.1}%",
        stats.lookups as f64 / (stats.allocations + stats.lookups) as f64 * 100.0);
}

fn benchmark_without_interner() {
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];
    let mut strings = Vec::new();

    let start = Instant::now();
    for _ in 0..100000 {
        for word in &words {
            strings.push(word.to_string());  // Always allocate
        }
        strings.clear();  // Clear to avoid OOM
    }
    let duration = start.elapsed();

    println!("Without interner: {:?}", duration);
    println!("Allocations: {}", 100_000 * words.len());
}
```

**Expected Results**:
- Interner: ~10-50ms, 4 allocations, 596 lookups (99.3% hit rate)
- Without: ~200-500ms, 600,000 allocations

**When Does Interning Help Most?**
- High duplication rate (many lookups, few allocations)
- Long strings (allocation cost dominates)
- Frequent equality comparisons (pointer compare vs string compare)

**When Might Interning Hurt?**
- All unique strings (100% allocation rate + hash overhead)
- Very short-lived strings (lookup cost exceeds benefit)
- Single-use strings that are never compared

**Check Your Understanding**:
- When does interning help most?
- When might interning hurt performance?
- What's the memory trade-off?


