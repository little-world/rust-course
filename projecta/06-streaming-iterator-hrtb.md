# Streaming Iterator with HRTB

## Problem Statement

Build a **streaming iterator** that yields borrowed elements with complex lifetime requirements using Higher-Ranked Trait Bounds (HRTB) and Generic Associated Types (GATs). Unlike standard `Iterator` which returns owned items, streaming iterators return items that borrow from the iterator itself, enabling zero-copy iteration over streaming data.


**Performance Impact**:
- **Standard Iterator**: Must allocate/copy for each item (50-100ns per item)
- **Streaming Iterator**: Zero-copy borrowing (1-2ns per item)
- **10-100x faster** for large datasets
- **Memory efficiency**: No heap allocation, better cache locality

---

## Understanding Streaming Iterators and Advanced Lifetimes

Before implementing streaming iterators, let's understand the fundamental concepts of Generic Associated Types (GATs), Higher-Ranked Trait Bounds (HRTBs), and advanced lifetime patterns that make zero-copy iteration possible.

### Why Standard Iterator Is Limited

The standard `Iterator` trait has a fundamental limitation: items cannot borrow from the iterator.

**Standard Iterator Trait**:
```rust
trait Iterator {
    type Item;  // Associated type, NO lifetime parameter
    fn next(&mut self) -> Option<Self::Item>;
}
```

**The Problem**:
```rust
struct WindowIterator {
    data: Vec<i32>,
    position: usize,
}

// ❌ Can't implement Iterator to return &[i32]
impl Iterator for WindowIterator {
    type Item = &[i32];  // ERROR: Missing lifetime parameter!
    //          ↑ What lifetime? Can't refer to 'self lifetime here!

    fn next(&mut self) -> Option<Self::Item> {
        // Want to return slice borrowing from self.data
        // But Item can't have a lifetime parameter!
    }
}
```

**Why This Fails**:
```rust
// Let's try adding a lifetime:
impl<'a> Iterator for WindowIterator<'a> {
    type Item = &'a [i32];  // 'a is from the struct

    fn next(&mut self) -> Option<Self::Item> {
        // Problem: 'a is FIXED when WindowIterator is created
        // But next() is called MULTIPLE times with DIFFERENT &mut self lifetimes!
        // Each call has a fresh lifetime that doesn't relate to 'a
        Some(&self.data[self.position..])  // ERROR: lifetime mismatch
    }
}

// Each next() call has its own lifetime:
let mut iter = WindowIterator { data: vec![1,2,3], position: 0 };
let window1 = iter.next(); // Lifetime 'call1
let window2 = iter.next(); // Lifetime 'call2 (DIFFERENT!)

// 'a can't be both 'call1 and 'call2 - it's fixed!
```

**The Core Issue**: `Item` is an associated **type**, not a **type constructor**. It can't be parameterized by the lifetime of each `next()` call.

---

### Generic Associated Types (GATs): The Solution

**Generic Associated Types** (GATs) allow associated types to have generic parameters, including lifetimes.

**Enabled in Rust 1.65+**, GATs are one of the most powerful type system features.

**Streaming Iterator with GAT**:
```rust
trait StreamingIterator {
    type Item<'a> where Self: 'a;  // ← GAT! Item is now a type constructor
    //          ↑ Takes a lifetime parameter

    fn next(&mut self) -> Option<Self::Item<'_>>;
    //                                      ↑ Anonymous lifetime from &mut self
}
```

**How This Works**:
```rust
impl StreamingIterator for WindowIterator {
    type Item<'a> = &'a [i32] where Self: 'a;
    //       ↑ Type constructor: given lifetime, produces type

    fn next(&mut self) -> Option<Self::Item<'_>> {
        //      ↑ &'a mut self
        // Returns Option<&'a [i32]> - borrows from THIS call's lifetime
        Some(&self.data[self.position..])  // ✓ OK!
    }
}

// Now each call can have its own lifetime:
let mut iter = WindowIterator { data: vec![1,2,3], position: 0 };
let window1 = iter.next(); // Returns Option<&'1 [i32]>
drop(window1);
let window2 = iter.next(); // Returns Option<&'2 [i32]> (fresh lifetime!)
```

**Key Insight**: `Item<'a>` is a **type constructor** (also called **type-level function**). Given a lifetime `'a`, it produces a type `&'a [i32]`.

**The `where Self: 'a` Clause**:
```rust
type Item<'a> where Self: 'a;
```

This says: "The lifetime `'a` cannot outlive `Self`". It's necessary because:
```rust
// Without where clause:
type Item<'a>; // Could try to return &'a [i32] even if 'a > Self lifetime

// With where clause:
type Item<'a> where Self: 'a;
// Guarantees: Item<'a> can only borrow for as long as Self lives
// Prevents returning references that outlive the iterator
```

---

### Higher-Ranked Trait Bounds (HRTB)

**HRTBs** allow you to express that a trait must hold **for all possible lifetimes**, not just one specific lifetime.

**The Problem Without HRTB**:
```rust
fn process_stream<'a, I, F>(iter: I, f: F)
where
    I: StreamingIterator,
    F: FnMut(I::Item<'a>),  // F works with ONE specific lifetime 'a
{
    while let Some(item) = iter.next() {
        //                 ↑ returns Item<'call>
        // But F expects Item<'a>!
        // These lifetimes don't match!
        f(item);  // ❌ ERROR
    }
}
```

**Why This Fails**: Each call to `next()` produces an item with a **fresh lifetime**, but `F` is bound to ONE specific lifetime `'a` chosen when the function is called.

**Solution with HRTB**:
```rust
fn process_stream<I, F>(mut iter: I, mut f: F)
where
    I: StreamingIterator,
    F: for<'a> FnMut(I::Item<'a>),  // ← HRTB!
    //  ↑ "for ALL lifetimes 'a"
{
    while let Some(item) = iter.next() {
        // item has lifetime 'call
        // F must work for ANY lifetime, including 'call
        f(item);  // ✓ OK! F works for 'call because it works for ALL lifetimes
    }
}
```

**Reading `for<'a>`**: "for all lifetimes `'a`" - the trait bound must hold **universally**, not just for one specific lifetime.

**Analogy**:
```rust
// Without HRTB: Pick ONE integer
fn takes_one_int<N: Integer>(f: impl Fn(N)) { }
// f works with ONE specific integer type (i32, i64, etc.)

// With HRTB: Works with ALL integers
fn takes_all_ints(f: impl for<N: Integer> Fn(N)) { }
// f works with EVERY integer type
```

**Why Needed**:
```rust
// Each next() call has different lifetime:
let mut iter = windows(&data, 3);

// Call 1:
let item1 = iter.next(); // item1: Option<&'1 [i32]>
process(item1);

// Call 2:
let item2 = iter.next(); // item2: Option<&'2 [i32]> (different!)
process(item2);

// Closure must work with BOTH '1 and '2 (and all other lifetimes)
// Hence: for<'a> FnMut(Item<'a>)
```

---

### Lifetime Variance

**Variance** describes how subtyping works with generic parameters.

**Three Kinds of Variance**:
1. **Covariant**: If `'long: 'short`, then `F<'long>: F<'short>` (longer lifetime subtypes shorter)
2. **Contravariant**: If `'long: 'short`, then `F<'short>: F<'long>` (reversed!)
3. **Invariant**: No subtyping relationship

**Shared References are Covariant**:
```rust
// &'a T is covariant in both 'a and T

fn covariance_example<'long: 'short, 'short>(long_ref: &'long str) -> &'short str {
    long_ref  // ✓ OK: Can use &'long where &'short expected
    // Because 'long: 'short (long outlives short)
    // And &'a T is covariant in 'a
}

// Real example:
let long_lived = String::from("data");
{
    let short_ref: &str = &long_lived;  // &'long assigned to &'short
    // Covariance allows this!
}
```

**Why Covariance is Safe for &T**:
- Shared references are read-only
- Returning a longer-lived reference where a shorter-lived one is expected is always safe
- The data lives at least as long as required

**Mutable References are Invariant**:
```rust
// &'a mut T is invariant in 'a (but covariant in T)

fn invariance_example<'long: 'short, 'short>(
    long_ref: &'long mut str
) -> &'short mut str {
    long_ref  // ❌ ERROR: Can't do this!
    // &'a mut T is invariant in 'a
}
```

**Why Invariance is Necessary for &mut T**:
```rust
// If &mut was covariant, this would be unsound:
fn unsound_if_covariant() {
    let mut long_lived = String::from("long");
    {
        let short_lived = String::from("short");
        let short_ref: &mut String = &mut short_lived;

        // If covariant (IT'S NOT!), could do:
        // let long_ref: &mut String = &mut long_lived;
        // *short_ref = long_ref;  // Replace short with long

        // short_ref would now point to long_lived
    } // short_lived dropped, but short_ref still exists!
    // Use-after-free!
}
```

**Variance Table**:
```
Type              Variance in 'a    Variance in T
&'a T             Covariant         Covariant
&'a mut T         Invariant         Covariant
fn(T) -> U        -                 Contravariant in T, Covariant in U
Cell<T>           -                 Invariant
PhantomData<T>    -                 Covariant
PhantomData<fn(T)> -                Contravariant
```

---

### Zero-Copy Iteration

**The Allocation Problem**:
```rust
// Standard iterator approach:
struct WindowIter {
    data: Vec<i32>,
    window_size: usize,
    position: usize,
}

impl Iterator for WindowIter {
    type Item = Vec<i32>;  // Must return OWNED Vec

    fn next(&mut self) -> Option<Vec<i32>> {
        if self.position + self.window_size <= self.data.len() {
            // ALLOCATION: Copy data into new Vec
            let window = self.data[self.position..self.position + self.window_size]
                .to_vec();
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}

// For 1000 windows:
// - 1000 heap allocations (~50-100ns each = 50-100µs)
// - 1000 memcpy operations
// - 1000 deallocations
// Total overhead: ~150µs for small windows
```

**Streaming Iterator Approach**:
```rust
impl StreamingIterator for WindowIter {
    type Item<'a> = &'a [i32] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position + self.window_size <= self.data.len() {
            // ZERO-COPY: Just create slice (pointer + length)
            let window = &self.data[self.position..self.position + self.window_size];
            self.position += 1;
            Some(window)  // ~1-2ns (just pointer arithmetic)
        } else {
            None
        }
    }
}

// For 1000 windows:
// - 0 heap allocations
// - 0 memcpy operations
// - Just pointer arithmetic
// Total time: ~2µs
// 75x faster!
```

**Memory Comparison**:
```
Standard Iterator Window:
┌───────────────────────────┐
│ Vec<i32>                  │
├───────────────────────────┤
│ ptr: *mut i32             │ 8 bytes
│ len: usize                │ 8 bytes
│ cap: usize                │ 8 bytes
└───────────────────────────┘
        ↓ points to
┌───────────────────────────┐
│ Heap allocation           │
│ [i32; window_size]        │ window_size * 4 bytes
└───────────────────────────┘

Total: 24 bytes + heap allocation + fragmentation

Streaming Iterator Window:
┌───────────────────────────┐
│ &[i32]                    │
├───────────────────────────┤
│ ptr: *const i32           │ 8 bytes
│ len: usize                │ 8 bytes
└───────────────────────────┘
        ↓ points into existing data
┌───────────────────────────┐
│ Original Vec<i32>         │
│ (no additional allocation)│
└───────────────────────────┘

Total: 16 bytes (no heap allocation)
```

---

### Type Constructors vs Concrete Types

**Concrete Type**: A type you can create values of
```rust
i32                    // Concrete type
Vec<String>            // Concrete type
&'static str           // Concrete type
```

**Type Constructor**: A "function" that takes type/lifetime parameters and produces a concrete type
```rust
Vec<T>                 // Type constructor (needs T)
&'a T                  // Type constructor (needs 'a and T)
Option<T>              // Type constructor (needs T)
```

**GATs Introduce Type Constructors in Traits**:
```rust
trait StreamingIterator {
    type Item<'a> where Self: 'a;  // Type constructor!
    //       ↑ Takes lifetime, returns type
}

// Concrete instance:
impl StreamingIterator for WindowIter {
    type Item<'a> = &'a [i32] where Self: 'a;
    // Item is a type constructor:
    // Item<'call> = &'call [i32]
}
```

**Using Type Constructors**:
```rust
fn process<I: StreamingIterator>(mut iter: I) {
    // Item<'_> uses anonymous lifetime from next()
    let first: Option<I::Item<'_>> = iter.next();
    //                         ↑ Apply constructor to fresh lifetime

    // Different call, different lifetime:
    let second: Option<I::Item<'_>> = iter.next();
    //                          ↑ Fresh lifetime again
}
```

---

### The Borrowing Constraint

**Standard Iterator - Items Can Be Held**:
```rust
let mut iter = vec![1, 2, 3].into_iter();
let first = iter.next();   // first: Option<i32>
let second = iter.next();  // second: Option<i32>

// Both can exist simultaneously:
if let (Some(a), Some(b)) = (first, second) {
    println!("{} {}", a, b);  // ✓ OK
}
```

**Streaming Iterator - Items Borrow from Iterator**:
```rust
let data = vec![1, 2, 3];
let mut iter = Iter::new(&data);

let first = iter.next();   // first: Option<&'a i32> where 'a = borrow of iter
// let second = iter.next();  // ❌ ERROR: Can't borrow iter mutably again!

// Can't hold two items at once:
// if let (Some(a), Some(b)) = (first, second) {  // Won't compile
//     println!("{} {}", a, b);
// }

// Must drop first before getting second:
if let Some(a) = first {
    println!("{}", a);
}
// first dropped here

let second = iter.next();  // ✓ OK now
```

**Why This Limitation**:
```rust
// Item borrows from iterator:
type Item<'a> = &'a T where Self: 'a;
//               ↑ This 'a is the lifetime of &mut self in next()

// When you call next():
fn next(&mut self) -> Option<Self::Item<'_>>;
//      ↑ Borrows self mutably
//                              ↑ Item borrows from this &mut self

// Holding the item = holding the borrow of self
// Can't borrow self again until item is dropped
```

---

### Connection to This Project

In this project, you'll implement all these concepts:

1. **Milestone 1**: StreamingIterator trait with GATs
   - Define `type Item<'a> where Self: 'a`
   - Understand why standard Iterator can't borrow from self
   - Implement simple streaming iterator

2. **Milestone 2**: Windows iterator with zero-copy
   - Return slices borrowing from data
   - Demonstrate 10-100x speedup over allocating
   - Build step_by adapter

3. **Milestone 3**: Higher-Ranked Trait Bounds
   - Generic functions with `for<'a>` bounds
   - Universal quantification over lifetimes
   - Implement for_each, fold, all with HRTB

4. **Milestone 4**: GroupBy with lifetime variance
   - Demonstrate covariance of &'a T
   - Show why &'a mut T is invariant
   - Lifetime subtyping in practice

5. **Milestone 5**: Performance comparison
   - Benchmark streaming vs standard iterators
   - Understand trade-offs
   - Document use cases

**Key Learning Points**:
- GATs enable self-borrowing iterators
- HRTBs express universal lifetime constraints
- Zero-copy iteration eliminates allocation overhead
- Variance determines lifetime subtyping behavior
- Streaming iterators trade flexibility for performance

**Real-World Applications**:
- Log file parsing (zero-copy line iteration)
- Network packet inspection (borrow packet data)
- Database result sets (cursor-style iteration)
- Video/audio processing (frame/sample windows)
- Compression algorithms (sliding window decompression)

---

## Milestone 1: Basic StreamingIterator Trait with GATs

**Goal**: Define the `StreamingIterator` trait using Generic Associated Types (GATs) to enable items that borrow from the iterator.

**Concepts**:
- Generic Associated Types (GATs): `type Item<'a>`
- Lifetime parameters in associated types
- Higher-kinded types (type constructors)
- Self-borrowing return types
- GAT where clauses: `where Self: 'a`

**Implementation Steps**:

1. **Define `StreamingIterator` trait**:
   - Associated type `Item<'a>` parameterized by lifetime
   - Where clause `where Self: 'a` (item can borrow from self)
   - Method `next(&mut self) -> Option<Self::Item<'_>>`
   - Method `size_hint(&self) -> (usize, Option<usize>)`

2. **Implement simple streaming iterator**:
   - `SliceIter<T>` that yields `&[T]` windows
   - Each call to `next()` returns a slice borrowing from internal data
   - Demonstrate why standard `Iterator` can't do this

3. **Compare with standard Iterator**:
   - Show compilation error when trying to use `Iterator`
   - Explain why `Item` can't depend on `&self` lifetime
   - Demonstrate GAT enables self-borrowing

**Starter Code**:

```rust
// Standard Iterator for comparison
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Streaming Iterator with GATs
pub trait StreamingIterator {
    // TODO: Define Item<'a> as associated type
    // Hint: Generic Associated Type with lifetime parameter
    // Must have where clause: where Self: 'a
    type Item<'a> where Self: 'a;

    // TODO: Define next() method
    // Returns Option<Self::Item<'_>> (borrows from &mut self)
    fn next(&mut self) -> Option<Self::Item<'_>>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

// Simple implementation: iterate over references
pub struct Iter<'data, T> {
    data: &'data [T],
    position: usize,
}

impl<'data, T> Iter<'data, T> {
    pub fn new(data: &'data [T]) -> Self {
        Self { data, position: 0 }
    }
}

impl<'data, T> StreamingIterator for Iter<'data, T> {
    // TODO: Set Item<'a> to &'a T
    // This means: when you call next(&'a mut self), you get &'a T
    type Item<'a> = &'a T where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        // TODO: Check if position < data.len()
        // Return Some(&data[position]) and increment position
        // Otherwise return None

        if self.position < self.data.len() {
            let item = &self.data[self.position];
            self.position += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.data.len() - self.position;
        (remaining, Some(remaining))
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_iter_basic() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = Iter::new(&data);

        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&5));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_size_hint() {
        let data = vec![10, 20, 30];
        let mut iter = Iter::new(&data);

        assert_eq!(iter.size_hint(), (3, Some(3)));

        iter.next();
        assert_eq!(iter.size_hint(), (2, Some(2)));

        iter.next();
        assert_eq!(iter.size_hint(), (1, Some(1)));
    }

    #[test]
    fn test_lifetime_borrowing() {
        let data = vec![1, 2, 3];
        let mut iter = Iter::new(&data);

        // Item borrows from iter, which borrows from data
        let first = iter.next().unwrap();
        assert_eq!(*first, 1);

        // Can't move iter while first is borrowed
        // This should NOT compile (uncomment to verify):
        // let second = iter.next(); // ERROR: iter is borrowed
        // println!("{}", first);
    }

    // This demonstrates why standard Iterator won't work
    // #[test]
    // fn test_standard_iterator_fails() {
    //     struct BrokenIter<'a, T> {
    //         data: &'a [T],
    //         pos: usize,
    //     }
    //
    //     // This won't compile: Item can't depend on &self lifetime
    //     impl<'a, T> Iterator for BrokenIter<'a, T> {
    //         type Item = &'a T;  // ERROR: 'a is not related to next()'s lifetime
    //         fn next(&mut self) -> Option<Self::Item> {
    //             // ...
    //         }
    //     }
    // }
}
```

**Check Your Understanding**:
1. What does `type Item<'a> where Self: 'a` mean?
2. Why can't standard `Iterator` have `type Item<'a>`?
3. How does the lifetime in `next(&mut self)` relate to `Item<'_>`?

---

## Milestone 2: Window Iterator with Slice Borrowing

**Goal**: Implement `Windows<'data, T>` that yields overlapping slices of fixed size, demonstrating true streaming iteration.

**Concepts**:
- Yielding slices that borrow from data
- Window size as const generic
- Sliding window algorithm
- Item lifetime tied to method call lifetime
- Cannot collect into Vec (items borrow from iterator)

**Implementation Steps**:

1. **Define `Windows<'data, T>` struct**:
   - Field: `data: &'data [T]` (source data)
   - Field: `window_size: usize` (size of each window)
   - Field: `position: usize` (current start position)

2. **Implement `StreamingIterator` for `Windows`**:
   - `Item<'a> = &'a [T]` (slice of window_size elements)
   - `next()` returns slice from `data[position..position+window_size]`
   - Increment position by 1 (overlapping windows)
   - Return None when window exceeds data bounds

3. **Add helper methods**:
   - `windows(data, size)` constructor
   - `step_by(n)` adapter for non-overlapping windows
   - Demonstrate why this requires streaming iteration

**Starter Code**:

```rust
pub struct Windows<'data, T> {
    data: &'data [T],
    window_size: usize,
    position: usize,
}

impl<'data, T> Windows<'data, T> {
    pub fn new(data: &'data [T], window_size: usize) -> Self {
        assert!(window_size > 0, "Window size must be > 0");
        Self {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<'data, T> StreamingIterator for Windows<'data, T> {
    // TODO: Set Item<'a> to &'a [T]
    // This means each window is a borrowed slice
    type Item<'a> = &'a [T] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        // TODO: Check if we can create a window at current position
        // Need: position + window_size <= data.len()
        // TODO: If yes, get slice &data[position..position+window_size]
        // TODO: Increment position by 1 (overlapping)
        // TODO: Return Some(window)

        if self.position + self.window_size <= self.data.len() {
            let window = &self.data[self.position..self.position + self.window_size];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO: Calculate remaining windows
        let remaining = if self.position + self.window_size <= self.data.len() {
            self.data.len() - self.position - self.window_size + 1
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

// Helper function to create windows
pub fn windows<T>(data: &[T], size: usize) -> Windows<'_, T> {
    Windows::new(data, size)
}

// Adapter for step-by iteration (non-overlapping)
pub struct StepBy<I> {
    iter: I,
    step: usize,
}

impl<I: StreamingIterator> StreamingIterator for StepBy<I> {
    type Item<'a> = I::Item<'a> where Self: 'a, I: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        // TODO: Get next item
        let item = self.iter.next()?;

        // TODO: Skip (step - 1) items
        for _ in 0..(self.step - 1) {
            self.iter.next();
        }

        Some(item)
    }
}

// Extension trait for adapters
pub trait StreamingIteratorExt: StreamingIterator {
    fn step_by(self, step: usize) -> StepBy<Self>
    where
        Self: Sized,
    {
        assert!(step > 0);
        StepBy { iter: self, step }
    }
}

impl<I: StreamingIterator> StreamingIteratorExt for I {}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_overlapping() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = windows(&data, 3);

        assert_eq!(iter.next(), Some(&[1, 2, 3][..]));
        assert_eq!(iter.next(), Some(&[2, 3, 4][..]));
        assert_eq!(iter.next(), Some(&[3, 4, 5][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_windows_size_2() {
        let data = vec!['a', 'b', 'c', 'd'];
        let mut iter = windows(&data, 2);

        assert_eq!(iter.next(), Some(&['a', 'b'][..]));
        assert_eq!(iter.next(), Some(&['b', 'c'][..]));
        assert_eq!(iter.next(), Some(&['c', 'd'][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_windows_exact_size() {
        let data = vec![10, 20, 30];
        let mut iter = windows(&data, 3);

        assert_eq!(iter.next(), Some(&[10, 20, 30][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_windows_too_large() {
        let data = vec![1, 2];
        let mut iter = windows(&data, 5);

        assert_eq!(iter.next(), None); // Window larger than data
    }

    #[test]
    fn test_step_by() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let mut iter = windows(&data, 2).step_by(2);

        assert_eq!(iter.next(), Some(&[1, 2][..]));
        assert_eq!(iter.next(), Some(&[3, 4][..]));
        assert_eq!(iter.next(), Some(&[5, 6][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_cannot_collect() {
        let data = vec![1, 2, 3, 4];
        let mut iter = windows(&data, 2);

        // Can consume and process
        while let Some(window) = iter.next() {
            println!("{:?}", window); // OK: process immediately
        }

        // Cannot collect because Item<'a> borrows from iter
        // This won't compile (no collect() for StreamingIterator):
        // let collected: Vec<_> = iter.collect(); // ERROR
    }

    #[test]
    fn test_lifetime_constraint() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = windows(&data, 3);

        let first = iter.next().unwrap();
        // first borrows from iter, which borrows from data

        // Can't get second window while first is borrowed
        // let second = iter.next(); // ERROR: iter is mutably borrowed

        println!("{:?}", first);
        // first dropped here, can continue iteration
    }
}
```

**Check Your Understanding**:
1. Why can't you collect `Windows` into a `Vec` like standard iterators?
2. How does the window lifetime relate to the data lifetime?
3. What happens if you try to call `next()` while holding a previous window?

---

## Milestone 3: Higher-Ranked Trait Bounds for Generic Functions

**Goal**: Implement generic functions that work with any `StreamingIterator`, using HRTB to abstract over all lifetimes.

**Concepts**:
- Higher-Ranked Trait Bounds: `for<'a>`
- Universal quantification over lifetimes
- Generic functions over streaming iterators
- Lifetime polymorphism
- `for<'a> Fn(&'a T)` pattern

**Implementation Steps**:

1. **Implement `for_each` with HRTB**:
   - Function that processes each item
   - Works with any `StreamingIterator`
   - Closure must work for any lifetime: `for<'a> F: FnMut(&'a T)`

2. **Implement `all` predicate with HRTB**:
   - Check if all items satisfy predicate
   - Predicate: `for<'a> F: Fn(&'a T) -> bool`

3. **Implement `fold` with HRTB**:
   - Accumulate value from streaming iterator
   - Closure: `for<'a> F: FnMut(B, &'a T) -> B`

4. **Explain why HRTB is necessary**:
   - Without `for<'a>`: closure tied to single lifetime
   - With `for<'a>`: closure works for all call lifetimes

**Starter Code**:

```rust
// Generic function using HRTB
pub fn for_each<I, F>(mut iter: I, mut f: F)
where
    I: StreamingIterator,
    // HRTB: F must work for ANY lifetime 'a
    F: for<'a> FnMut(I::Item<'a>),
{
    // TODO: Call f on each item from iter
    while let Some(item) = iter.next() {
        f(item);
    }
}

// Check if all items satisfy predicate
pub fn all<I, F>(mut iter: I, mut predicate: F) -> bool
where
    I: StreamingIterator,
    // TODO: Add HRTB for predicate
    // Hint: for<'a> Fn(I::Item<'a>) -> bool
    F: for<'a> FnMut(I::Item<'a>) -> bool,
{
    // TODO: Return false if any item fails predicate
    while let Some(item) = iter.next() {
        if !predicate(item) {
            return false;
        }
    }
    true
}

// Fold/reduce with accumulator
pub fn fold<I, B, F>(mut iter: I, init: B, mut f: F) -> B
where
    I: StreamingIterator,
    // TODO: Add HRTB for fold function
    // Hint: for<'a> FnMut(B, I::Item<'a>) -> B
    F: for<'a> FnMut(B, I::Item<'a>) -> B,
{
    // TODO: Fold items into accumulator
    let mut acc = init;
    while let Some(item) = iter.next() {
        acc = f(acc, item);
    }
    acc
}

// Count items
pub fn count<I>(mut iter: I) -> usize
where
    I: StreamingIterator,
{
    // TODO: Count all items
    let mut count = 0;
    while iter.next().is_some() {
        count += 1;
    }
    count
}

// Find first item matching predicate
pub fn find<I, F>(mut iter: I, mut predicate: F) -> bool
where
    I: StreamingIterator,
    // TODO: Add HRTB
    F: for<'a> FnMut(I::Item<'a>) -> bool,
{
    // TODO: Return true if any item matches
    while let Some(item) = iter.next() {
        if predicate(item) {
            return true;
        }
    }
    false
}

// Why HRTB is necessary - comparison:

// WITHOUT HRTB (doesn't work):
// fn broken_for_each<'a, I, F>(mut iter: I, mut f: F)
// where
//     I: StreamingIterator,
//     F: FnMut(I::Item<'a>),  // ERROR: 'a must outlive function
// {
//     while let Some(item) = iter.next() {
//         f(item);  // ERROR: item has different lifetime each call
//     }
// }

// WITH HRTB (works):
// for<'a> means "for ALL lifetimes 'a"
// Function works regardless of what lifetime next() returns
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_each() {
        let data = vec![1, 2, 3, 4, 5];
        let iter = windows(&data, 2);

        let mut sum = 0;
        for_each(iter, |window| {
            sum += window[0] + window[1];
        });

        // Windows: [1,2], [2,3], [3,4], [4,5]
        // Sum: 3 + 5 + 7 + 9 = 24
        assert_eq!(sum, 24);
    }

    #[test]
    fn test_all() {
        let data = vec![2, 4, 6, 8];
        let iter = Iter::new(&data);

        let all_even = all(iter, |&x| x % 2 == 0);
        assert!(all_even);

        let data = vec![2, 4, 5, 8];
        let iter = Iter::new(&data);
        let all_even = all(iter, |&x| x % 2 == 0);
        assert!(!all_even);
    }

    #[test]
    fn test_fold() {
        let data = vec![1, 2, 3, 4, 5];
        let iter = Iter::new(&data);

        let sum = fold(iter, 0, |acc, &x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_fold_windows() {
        let data = vec![1, 2, 3, 4];
        let iter = windows(&data, 2);

        // Concatenate all windows
        let result = fold(iter, Vec::new(), |mut acc, window| {
            acc.extend_from_slice(window);
            acc
        });

        // Windows: [1,2], [2,3], [3,4]
        assert_eq!(result, vec![1, 2, 2, 3, 3, 4]);
    }

    #[test]
    fn test_count() {
        let data = vec![10, 20, 30, 40, 50];
        let iter = windows(&data, 3);

        assert_eq!(count(iter), 3); // 3 windows of size 3
    }

    #[test]
    fn test_find() {
        let data = vec![1, 2, 3, 4, 5];
        let iter = windows(&data, 2);

        // Find window where first element is 3
        let has_window_starting_3 = find(iter, |window| window[0] == 3);
        assert!(has_window_starting_3);
    }

    #[test]
    fn test_hrtb_necessity() {
        let data = vec![1, 2, 3];
        let iter = windows(&data, 2);

        // Closure must work for ANY lifetime
        // Each call to next() returns item with different lifetime
        for_each(iter, |window| {
            // window: &'a [i32] where 'a is different each iteration
            println!("{:?}", window);
        });

        // Without for<'a>, we'd need to name the lifetime upfront
        // But we don't know it until next() is called!
    }
}
```

**Check Your Understanding**:
1. What does `for<'a> FnMut(I::Item<'a>)` mean in plain English?
2. Why can't we use a regular lifetime parameter instead of HRTB?
3. How does the compiler verify the HRTB constraint is satisfied?

---

## Milestone 4: GroupBy Iterator with Lifetime Variance

**Goal**: Implement `GroupBy` that yields consecutive equal elements, demonstrating covariance and lifetime relationships.

**Concepts**:
- Lifetime variance: covariant, contravariant, invariant
- Subtyping with lifetimes (`'long: 'short`)
- GroupBy algorithm with comparison
- Multiple borrows from same data
- Variance in `&'a T` (covariant in both 'a and T)

**Implementation Steps**:

1. **Define `GroupBy<'data, T>` iterator**:
   - Groups consecutive equal elements
   - Returns slices of equal elements
   - `Item<'a> = &'a [T]` where `T: PartialEq`

2. **Implement comparison logic**:
   - Scan forward while elements are equal
   - Return slice of all equal consecutive elements
   - Demonstrate lifetime covariance

3. **Explain variance**:
   - `&'a T` is covariant in 'a: if 'long: 'short, then &'long T: &'short T
   - `&'a mut T` is invariant in 'a: no subtyping
   - Function pointers contravariant in argument types

**Starter Code**:

```rust
pub struct GroupBy<'data, T> {
    data: &'data [T],
    position: usize,
}

impl<'data, T> GroupBy<'data, T> {
    pub fn new(data: &'data [T]) -> Self {
        Self { data, position: 0 }
    }
}

impl<'data, T: PartialEq> StreamingIterator for GroupBy<'data, T> {
    type Item<'a> = &'a [T] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        // TODO: Check if at end
        if self.position >= self.data.len() {
            return None;
        }

        // TODO: Find end of group (while elements equal)
        let start = self.position;
        let first = &self.data[start];
        let mut end = start + 1;

        while end < self.data.len() && &self.data[end] == first {
            end += 1;
        }

        // TODO: Update position
        self.position = end;

        // TODO: Return slice of group
        Some(&self.data[start..end])
    }
}

// Demonstrate variance
pub fn demonstrate_variance() {
    let data = vec![1, 1, 2, 2, 2, 3];

    // 'data is the lifetime of data
    let group_iter = GroupBy::new(&data);

    // Each group borrows from data
    // &'data [i32] is covariant in 'data:
    // If we had 'long: 'short, we could use &'long in place of &'short

    // This is safe because & is read-only
}

// Variance comparison:
struct CovariantExample<'a, T> {
    reference: &'a T,  // Covariant in both 'a and T
}

struct InvariantExample<'a, T> {
    mutable: &'a mut T,  // Invariant in 'a, covariant in T
}

// Covariance example
fn covariance_works<'long: 'short, 'short>(long_ref: &'long str) -> &'short str {
    // Can return &'long as &'short because 'long: 'short
    // &'a T is covariant in 'a
    long_ref
}

// Invariance example
// fn invariance_fails<'long: 'short, 'short>(
//     long_ref: &'long mut str
// ) -> &'short mut str {
//     // ERROR: Can't return &'long mut as &'short mut
//     // &'a mut T is invariant in 'a
//     long_ref
// }
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_by_consecutive() {
        let data = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&[1, 1, 1][..]));
        assert_eq!(iter.next(), Some(&[2, 2][..]));
        assert_eq!(iter.next(), Some(&[3, 3, 3, 3][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_group_by_single_elements() {
        let data = vec![1, 2, 3, 4];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&[1][..]));
        assert_eq!(iter.next(), Some(&[2][..]));
        assert_eq!(iter.next(), Some(&[3][..]));
        assert_eq!(iter.next(), Some(&[4][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_group_by_all_equal() {
        let data = vec!['a', 'a', 'a', 'a'];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&['a', 'a', 'a', 'a'][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_group_by_strings() {
        let data = vec!["a", "a", "b", "c", "c", "c"];
        let mut iter = GroupBy::new(&data);

        assert_eq!(iter.next(), Some(&["a", "a"][..]));
        assert_eq!(iter.next(), Some(&["b"][..]));
        assert_eq!(iter.next(), Some(&["c", "c", "c"][..]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_variance_covariant() {
        fn takes_short<'short>(_: &'short str) {}

        let long_lived = String::from("long");
        {
            // long_lived: 'long
            let long_ref: &str = &long_lived;

            // Can pass &'long to function expecting &'short
            // because 'long: 'short (long outlives short)
            takes_short(long_ref); // Covariance!
        }
    }

    #[test]
    fn test_lifetime_subtyping() {
        let data = vec![1, 2, 3];

        // 'data lifetime
        let iter = GroupBy::new(&data);

        // Each group has lifetime tied to 'data
        // Groups can't outlive data (enforced by borrow checker)

        // This won't compile:
        // let group;
        // {
        //     let data2 = vec![4, 5, 6];
        //     let mut iter2 = GroupBy::new(&data2);
        //     group = iter2.next(); // ERROR: data2 doesn't live long enough
        // }
        // println!("{:?}", group);
    }
}
```

**Check Your Understanding**:
1. What does it mean for `&'a T` to be covariant in `'a`?
2. Why is `&'a mut T` invariant in `'a` but `&'a T` is covariant?
3. How does variance affect lifetime subtyping in `GroupBy`?

---

## Milestone 5: Comparison with Standard Iterator and Performance

**Goal**: Compare streaming iterator with standard iterator, benchmark performance, and demonstrate when each is appropriate.

**Concepts**:
- Standard vs streaming iterator trade-offs
- Zero-copy vs allocation performance
- When to use each pattern
- Limitations of streaming iterators
- Performance benchmarking

**Implementation Steps**:

1. **Create allocating window iterator**:
   - Standard `Iterator` that returns `Vec<T>` (owned)
   - Compare with streaming `Windows` (borrowed)

2. **Benchmark both approaches**:
   - Measure allocations and time
   - Demonstrate 10-100x difference

3. **Document limitations**:
   - Can't collect streaming iterators
   - Can't hold multiple items simultaneously
   - More complex API

4. **Show when streaming is appropriate**:
   - Large datasets (log files, network streams)
   - Performance-critical code
   - Zero-copy requirements

**Starter Code**:

```rust
// Standard Iterator version (allocates)
pub struct AllocatingWindows<T: Clone> {
    data: Vec<T>,
    window_size: usize,
    position: usize,
}

impl<T: Clone> AllocatingWindows<T> {
    pub fn new(data: Vec<T>, window_size: usize) -> Self {
        Self {
            data,
            window_size,
            position: 0,
        }
    }
}

impl<T: Clone> Iterator for AllocatingWindows<T> {
    type Item = Vec<T>;  // Owned, allocated

    fn next(&mut self) -> Option<Self::Item> {
        if self.position + self.window_size <= self.data.len() {
            // ALLOCATION: Copy window into new Vec
            let window = self.data[self.position..self.position + self.window_size]
                .to_vec();
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}

// Benchmark comparison
pub fn benchmark_windows(data_size: usize, window_size: usize, iterations: usize) {
    use std::time::Instant;

    let data: Vec<i32> = (0..data_size as i32).collect();

    // Streaming iterator (zero-copy)
    let start = Instant::now();
    for _ in 0..iterations {
        let mut iter = windows(&data, window_size);
        let mut sum = 0;
        while let Some(window) = iter.next() {
            sum += window[0]; // Just access, no allocation
        }
        std::hint::black_box(sum); // Prevent optimization
    }
    let streaming_time = start.elapsed();

    // Standard iterator (allocating)
    let start = Instant::now();
    for _ in 0..iterations {
        let iter = AllocatingWindows::new(data.clone(), window_size);
        let mut sum = 0;
        for window in iter {
            sum += window[0]; // Each window allocated
        }
        std::hint::black_box(sum);
    }
    let allocating_time = start.elapsed();

    println!("Data size: {}, Window: {}", data_size, window_size);
    println!("Streaming:   {:?}", streaming_time);
    println!("Allocating:  {:?}", allocating_time);
    println!(
        "Speedup: {:.2}x",
        allocating_time.as_secs_f64() / streaming_time.as_secs_f64()
    );

    let num_windows = data_size - window_size + 1;
    println!("Allocations saved: {}", num_windows * iterations);
}

// Trade-offs comparison
pub fn compare_iterators() {
    println!("=== Standard Iterator ===");
    println!("Pros:");
    println!("  - Can collect() into Vec, HashMap, etc.");
    println!("  - Can hold multiple items simultaneously");
    println!("  - Familiar API (map, filter, fold)");
    println!("  - Can be cloned if Item: Clone");
    println!("\nCons:");
    println!("  - Must allocate/copy data");
    println!("  - Higher memory usage");
    println!("  - Slower for large items");

    println!("\n=== Streaming Iterator ===");
    println!("Pros:");
    println!("  - Zero-copy (items borrow from iterator)");
    println!("  - 10-100x faster for large items");
    println!("  - Minimal memory footprint");
    println!("  - Can iterate over infinite streams");
    println!("\nCons:");
    println!("  - Can't collect() (no owned items)");
    println!("  - Can only hold one item at a time");
    println!("  - More complex lifetime requirements");
    println!("  - Requires GATs (Rust 1.65+)");
}

// When to use streaming iterators
pub fn use_cases() {
    println!("=== Use Streaming Iterator When: ===");
    println!("  - Processing large files (logs, CSVs)");
    println!("  - Network packet inspection");
    println!("  - Database result sets");
    println!("  - Sliding window algorithms");
    println!("  - Performance-critical hot paths");
    println!("  - Zero-copy parsing");

    println!("\n=== Use Standard Iterator When: ===");
    println!("  - Need to collect results");
    println!("  - Items are small (integers, chars)");
    println!("  - Need to process items multiple times");
    println!("  - Want familiar API (map, filter, etc.)");
    println!("  - Allocation cost is negligible");
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocating_windows() {
        let data = vec![1, 2, 3, 4, 5];
        let mut iter = AllocatingWindows::new(data, 3);

        assert_eq!(iter.next(), Some(vec![1, 2, 3]));
        assert_eq!(iter.next(), Some(vec![2, 3, 4]));
        assert_eq!(iter.next(), Some(vec![3, 4, 5]));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_allocating_can_collect() {
        let data = vec![1, 2, 3, 4];
        let iter = AllocatingWindows::new(data, 2);

        // CAN collect with standard Iterator
        let windows: Vec<Vec<i32>> = iter.collect();
        assert_eq!(windows, vec![vec![1, 2], vec![2, 3], vec![3, 4]]);
    }

    #[test]
    fn test_streaming_cannot_collect() {
        let data = vec![1, 2, 3, 4];
        let _iter = windows(&data, 2);

        // CANNOT collect with StreamingIterator
        // No collect() method available!
        // let windows: Vec<_> = iter.collect(); // ERROR

        // Must process immediately:
        // for window in iter {
        //     process(window);
        // }
    }

    #[test]
    fn test_benchmark_small() {
        // Small benchmark
        benchmark_windows(1000, 10, 100);
    }

    #[test]
    fn test_memory_comparison() {
        use std::mem::size_of_val;

        let data: Vec<i32> = (0..1000).collect();

        // Streaming: just slice metadata
        let mut streaming = windows(&data, 10);
        if let Some(window) = streaming.next() {
            println!("Streaming window size: {} bytes", size_of_val(window));
            // Just 16 bytes (pointer + length)
        }

        // Allocating: full Vec allocation
        let mut allocating = AllocatingWindows::new(data.clone(), 10);
        if let Some(window) = allocating.next() {
            println!("Allocating window size: {} bytes", size_of_val(&window));
            // 24 bytes (pointer + length + capacity) + heap allocation
            println!("  + {} bytes on heap", window.capacity() * 4);
        }
    }

    #[test]
    fn test_use_case_demonstration() {
        // Use case: Processing log file (streaming appropriate)
        let log_data = "ERROR: file not found\nWARN: deprecated API\nINFO: startup complete";

        // Streaming: zero-copy line processing
        let lines: Vec<&str> = log_data.lines().collect();
        for line in &lines {
            // Process without allocation
            if line.starts_with("ERROR") {
                println!("Found error: {}", line);
            }
        }

        // Use case: Building data structure (standard iterator appropriate)
        let numbers = vec![1, 2, 3, 4, 5];
        let doubled: Vec<i32> = numbers.iter().map(|&x| x * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
        // Need collect() here, so standard Iterator is better
    }
}
```

**Check Your Understanding**:
1. Why can't you `collect()` a `StreamingIterator` into a `Vec`?
2. When would the allocation overhead of standard `Iterator` be acceptable?
3. How do the lifetime requirements differ between the two iterator types?

---

## Summary

You've built a **complete streaming iterator library** with:

1. **Basic StreamingIterator Trait with GATs** - `type Item<'a>` for self-borrowing items
2. **Window Iterator** - Overlapping slices with zero-copy
3. **Higher-Ranked Trait Bounds** - `for<'a>` for lifetime polymorphism
4. **GroupBy Iterator with Variance** - Demonstrating lifetime covariance
5. **Performance Comparison** - 10-100x faster than allocating iterators

**Key Patterns Learned**:
- **GATs (Generic Associated Types)**: `type Item<'a> where Self: 'a`
- **HRTB**: `for<'a> FnMut(I::Item<'a>)` for lifetime polymorphism
- **Self-borrowing**: Items that borrow from the iterator
- **Lifetime variance**: Covariant vs invariant positions
- **Zero-copy iteration**: Avoiding allocation in hot paths
- **Trade-offs**: When to use streaming vs standard iterators

**Performance Characteristics**:
- **Zero-copy**: 1-2ns per item (pointer arithmetic only)
- **Allocating**: 50-100ns per item (malloc + memcpy)
- **10-100x faster** for large items or datasets
- **Memory**: No heap allocation, better cache locality
- **Real-world**: Essential for high-performance streaming data

**Real-World Applications**:
- Log file parsing (zero-copy line processing)
- Network packet inspection (borrowing packet headers)
- Database cursors (streaming result sets)
- Compression/decompression (buffer windowing)
- Video/audio streaming (frame borrowing)

**When to Use Streaming Iterators**:
- ✅ Large files or datasets
- ✅ Performance-critical code
- ✅ Zero-copy requirements
- ✅ Sliding window algorithms
- ❌ Need to collect into Vec/HashMap
- ❌ Items are small (copy is cheap)
- ❌ Need to hold multiple items

**Next Steps**:
- Implement more adapters (map, filter for StreamingIterator)
- Add async streaming iterators (AsyncStreamingIterator)
- Build real parser using streaming iteration
- Compare with `streaming-iterator` crate
- Explore LendingIterator (alternative name)
