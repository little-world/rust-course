# Project 1: Generic Data Structure Library with Const Generics

## Problem Statement

Build a **collection of generic data structures** that leverage const generics for compile-time size guarantees and zero-cost abstractions. The library demonstrates how Rust's generics enable writing reusable containers that compile to optimal machine code for each concrete type through monomorphization.

**Use Cases**:
- Embedded systems requiring fixed-size, stack-allocated collections
- High-performance applications needing predictable memory layout
- Real-time systems where dynamic allocation is prohibited
- Generic algorithm libraries working across multiple types
- Teaching compile-time guarantees and zero-cost abstractions

## Why It Matters

Generic data structures with const generics demonstrate **Rust's type system strengths**:

**Compile-Time Size Guarantees**:
- **Without const generics**: `Vec<T>` uses heap allocation, dynamic size, runtime overhead
- **With const generics**: `Stack<T, 1024>` uses stack allocation, fixed size, zero runtime cost
- **Memory layout**: Known at compile time, enables better optimization and cache locality

**Zero-Cost Abstractions**:
- Generic code monomorphizes to specialized machine code per type
- `Stack<i32, 100>` and `Stack<String, 100>` are completely different compiled types
- No vtable lookups, no boxing, no dynamic dispatch
- Performance identical to hand-written specialized code

**Type Safety**:
- Bounds checking at compile time (const generics)
- Trait bounds enforce required operations (Ord, Clone, Default)
- Invalid states prevented by the type system (e.g., pushing to full stack)

**Performance Impact**:
- **Dynamic allocation (Vec)**: 50-100ns per allocation, unpredictable latency
- **Fixed-size (Stack<T, N>)**: 0ns allocation (stack), predictable latency
- **Cache efficiency**: Contiguous stack allocation vs scattered heap pointers
- **Optimization**: Compiler can inline and optimize with known sizes

---

## Understanding Generic Programming in Rust

Before implementing the data structures, let's understand the powerful concepts that make Rust's generics unique and performant.

### What are Generics?

**Generics** allow you to write code that works with multiple types without knowing the specific type at the time of writing. Instead of writing separate implementations for `Stack<i32>`, `Stack<String>`, `Stack<MyStruct>`, you write one generic implementation `Stack<T>` that works for **any** type `T`.

**The Problem Without Generics**:
```rust
// Have to write separate implementations for each type!
struct StackInt {
    data: Vec<i32>,
}

struct StackString {
    data: Vec<String>,
}

struct StackPoint {
    data: Vec<Point>,
}

// And repeat all methods for each type... horrible code duplication!
```

**With Generics**:
```rust
// Write once, use with any type!
struct Stack<T> {
    data: Vec<T>,
}

// Same implementation works for all types
let int_stack: Stack<i32> = Stack::new();
let string_stack: Stack<String> = Stack::new();
let point_stack: Stack<Point> = Stack::new();
```

---

### Monomorphization: Zero-Cost Abstractions

**Monomorphization** is Rust's technique for achieving **zero-cost abstractions** with generics. The compiler generates specialized machine code for each concrete type you use.

**How It Works**:
```rust
// You write this once:
struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    fn push(&mut self, value: T) { ... }
    fn pop(&mut self) -> Option<T> { ... }
}

// You use it with different types:
let int_stack: Stack<i32> = Stack::new();
let string_stack: Stack<String> = Stack::new();

// Compiler generates TWO separate implementations:
struct Stack_i32 {        struct Stack_String {
    data: Vec<i32>,           data: Vec<String>,
}                         }

impl Stack_i32 {          impl Stack_String {
    fn push(&mut self, value: i32) { ... }
    fn pop(&mut self) -> Option<i32> { ... }
}                         }
```

**Key Insight**: At runtime, there is **no** generic `Stack<T>`. The compiler has generated completely specialized versions like `Stack_i32` and `Stack_String`. Each has optimal machine code for that specific type.

**Performance Implications**:
```
Generic code in Rust:
- NO vtable lookups (unlike trait objects)
- NO boxing/unboxing
- NO type checking at runtime
- SAME performance as hand-written specialized code
- Compiler can inline aggressively

Example:
stack.push(42);  // For Stack<i32>, compiles to direct memory write
                 // No indirection, no dynamic dispatch, just raw speed
```

**Trade-off**:
- **Pro**: Maximum runtime performance, zero overhead
- **Con**: Larger binary size (more code for each type)
- **Con**: Longer compile times (compiler generates more code)

**Comparison with Other Languages**:
```
C++ Templates:  Similar monomorphization approach
Java Generics:  Type erasure - generics removed at runtime, uses Object
C# Generics:    Hybrid - value types specialized, reference types shared
Go:            Just added generics (Go 1.18+), uses dictionary passing
```

---

### Const Generics: Compile-Time Constants

**Const generics** allow you to parameterize types by **values** (not just types). This enables compile-time size guarantees.

**Without Const Generics** (old approach):
```rust
// Had to use associated constants or separate types
struct Stack<T> {
    data: Vec<T>,  // Heap allocated, dynamic size
}

// Or use macros to generate fixed-size variants
array_stack!(ArrayStack10, 10);
array_stack!(ArrayStack20, 20);
// Ugly and limited!
```

**With Const Generics**:
```rust
struct Stack<T, const N: usize> {
    data: [T; N],  // Fixed size, known at compile time!
}

// Can use any size!
let small: Stack<i32, 10> = Stack::new();
let big: Stack<i32, 1000> = Stack::new();
let huge: Stack<i32, 100000> = Stack::new();
```

**Benefits**:

1. **Compile-Time Size Guarantees**:
   ```rust
   let stack: Stack<i32, 100>;
   // Compiler KNOWS this is exactly 400 bytes (100 * 4 bytes)
   // Can allocate on stack instead of heap
   // Can optimize memory layout
   ```

2. **Zero Runtime Cost**:
   ```rust
   // Size is known at compile time
   impl<T, const N: usize> Stack<T, N> {
       fn capacity(&self) -> usize {
           N  // This is a compile-time constant!
              // No memory load, just immediate value
       }
   }
   ```

3. **Type Safety with Sizes**:
   ```rust
   fn process_exactly_10<T>(stack: Stack<T, 10>) {
       // Can ONLY call with 10-element stack
       // Compile error if you pass Stack<T, 5> or Stack<T, 20>
   }
   ```

**Real-World Example**:
```rust
// Graphics programming with fixed-size vectors
struct Vec3<T, const N: usize> {
    coords: [T; N],
}

type Vec2D = Vec3<f32, 2>;  // 2D vector
type Vec3D = Vec3<f32, 3>;  // 3D vector
type Vec4D = Vec3<f32, 4>;  // 4D vector (homogeneous coordinates)

// Compiler generates specialized code for each dimension
// No runtime checks, perfect optimization
```

---

### MaybeUninit<T>: Safe Uninitialized Memory

**The Problem**: Fixed-size arrays `[T; N]` require all elements to be initialized immediately. But for a data structure like Stack, we want to allocate space without initializing all elements.

**Unsafe Approach (Don't do this)**:
```rust
struct Stack<T, const N: usize> {
    data: [T; N],  // ERROR: Can't create this without N values of T!
    len: usize,
}

// How to create empty stack? Can't!
// Would need N default values, but T might not have Default
```

**MaybeUninit<T> Solution**:
```rust
use std::mem::MaybeUninit;

struct Stack<T, const N: usize> {
    data: [MaybeUninit<T>; N],  // ✓ Can create without initializing!
    len: usize,                  // Track how many are initialized
}
```

**What is MaybeUninit<T>?**

`MaybeUninit<T>` is a wrapper that can hold either:
- An initialized value of type `T`
- Uninitialized memory (garbage bytes)

It's the **safe** way to work with uninitialized memory in Rust.

**Key Operations**:

```rust
// 1. Create uninitialized memory
let uninit: MaybeUninit<i32> = MaybeUninit::uninit();
// This is just raw memory, contains garbage

// 2. Write a value into it
let mut uninit = MaybeUninit::uninit();
uninit.write(42);  // Now contains 42

// 3. Read the value (UNSAFE - you must know it's initialized!)
let value: i32 = unsafe { uninit.assume_init_read() };

// 4. Get a reference (UNSAFE - you must know it's initialized!)
let reference: &i32 = unsafe { uninit.assume_init_ref() };

// 5. Drop an initialized value (UNSAFE - you must know it's initialized!)
unsafe { uninit.assume_init_drop() };
```

**Why Unsafe?**

Reading uninitialized memory is undefined behavior:
```rust
let uninit: MaybeUninit<i32> = MaybeUninit::uninit();
// Contains random garbage bytes, could be anything!

let value = unsafe { uninit.assume_init() };
// UNDEFINED BEHAVIOR! Reading garbage as an i32
// Could crash, could produce random numbers, could summon demons
```

**Safe Pattern in Stack**:
```rust
impl<T, const N: usize> Stack<T, N> {
    fn push(&mut self, value: T) {
        // Only write to uninitialized slots
        if self.len < N {
            self.data[self.len].write(value);  // Safe: writing to uninit
            self.len += 1;
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len > 0 {
            self.len -= 1;
            // Safe: we KNOW data[len] is initialized (len tracks this!)
            Some(unsafe { self.data[self.len].assume_init_read() })
        } else {
            None
        }
    }
}
```

**The Safety Invariant**: We maintain `len` to track which elements are initialized. Elements `0..len` are initialized, `len..N` are uninitialized.

**Memory Layout Example**:
```
Stack<i32, 5> with len=3 after pushing 10, 20, 30:

[  10  |  20  |  30  |  ??  |  ??  ]
  ↑      ↑      ↑      ↑      ↑
data[0] data[1] data[2] data[3] data[4]
                 len=3
   <--initialized--> <--uninitialized-->

Safe to read: data[0], data[1], data[2]
UNSAFE to read: data[3], data[4] (garbage!)
```

**Why Not Use Option<T>?**

```rust
// Why not this?
struct Stack<T, const N: usize> {
    data: [Option<T>; N],  // Each element is Option
}
```

**Answer**: Memory waste!
```
Option<i32> = 8 bytes (4 for i32, 4 for discriminant)
MaybeUninit<i32> = 4 bytes (just the i32)

For Stack<i32, 100>:
- With Option: 100 * 8 = 800 bytes
- With MaybeUninit: 100 * 4 = 400 bytes + 8 bytes for len = 408 bytes

Nearly 2x more memory with Option!
```

Plus, `Option<T>` requires `T` to be valid, so you still can't have "truly" uninitialized memory.

---

### Trait Bounds: Constraining Generic Types

**Trait bounds** specify what operations a generic type must support. They're like contracts: "To use `T` in this code, `T` must implement these traits."

**The Problem Without Bounds**:
```rust
struct BinaryHeap<T> {
    data: Vec<T>,
}

impl<T> BinaryHeap<T> {
    fn push(&mut self, value: T) {
        self.data.push(value);
        // ERROR: How do we compare values to maintain heap property?
        // if self.data[i] > self.data[parent]  // ← Can't do this!
        //    T might not have comparison!
    }
}
```

**Solution: Add Trait Bounds**:
```rust
struct BinaryHeap<T: Ord> {  // ← Bound: T must implement Ord
    data: Vec<T>,
}

impl<T: Ord> BinaryHeap<T> {
    fn push(&mut self, value: T) {
        self.data.push(value);
        // ✓ Now we can compare!
        if self.data[i] > self.data[parent] {
            // Works because T: Ord guarantees > operator
        }
    }
}
```

**Common Trait Bounds**:

```rust
// 1. Ord - Total ordering (can compare any two values)
fn sort<T: Ord>(items: &mut [T]) {
    // Can use <, >, ==, etc.
}

// 2. Clone - Can be duplicated
fn duplicate<T: Clone>(value: &T) -> T {
    value.clone()
}

// 3. Copy - Can be bitwise copied (implicit clone)
fn double<T: Copy>(value: T) -> (T, T) {
    (value, value)  // Both use same value, but T is Copy
}

// 4. Default - Has a default value
fn create_with_default<T: Default>() -> T {
    T::default()
}

// 5. Debug - Can be formatted with {:?}
fn print_debug<T: Debug>(value: &T) {
    println!("{:?}", value);
}

// 6. Display - Can be formatted with {}
fn print<T: Display>(value: &T) {
    println!("{}", value);
}
```

**Multiple Bounds**:
```rust
// Require BOTH Ord and Copy
fn find_max<T: Ord + Copy>(items: &[T]) -> Option<T> {
    items.iter().copied().max()
}

// Alternative syntax (where clause)
fn find_max<T>(items: &[T]) -> Option<T>
where
    T: Ord + Copy,
{
    items.iter().copied().max()
}
```

**Why Ord vs PartialOrd?**

```rust
// PartialOrd: Partial ordering (some values can't be compared)
// Example: f32, f64 (NaN is not comparable to anything)
assert!(f32::NAN < 1.0);  // false
assert!(f32::NAN > 1.0);  // false
assert!(f32::NAN == f32::NAN);  // false

// Ord: Total ordering (ALL values can be compared)
// Example: i32, String, custom types
// For BinaryHeap, we NEED total ordering
// Can't have heap property if some elements are incomparable!

impl<T: Ord> BinaryHeap<T> {  // Must be Ord, not PartialOrd
    // Heap needs to compare ANY two elements
    // If using PartialOrd, what if comparison returns None?
}
```

**Conditional Implementation**:
```rust
// Implement Debug ONLY when T implements Debug
impl<T: Debug, const N: usize> Debug for Stack<T, N> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_list()
            .entries(/* iterate and format T values */)
            .finish()
    }
}

// This means:
let stack_i32: Stack<i32, 10> = Stack::new();
println!("{:?}", stack_i32);  // ✓ Works, i32: Debug

let stack_fn: Stack<fn(), 10> = Stack::new();
println!("{:?}", stack_fn);  // ✗ ERROR: fn() doesn't implement Debug
```

---

### Associated Types vs Generic Parameters

**Generic Type Parameters** (`<T>`):
```rust
trait Iterator<T> {  // T is generic parameter
    fn next(&mut self) -> Option<T>;
}

// Problem: Could implement Iterator multiple times!
impl Iterator<i32> for MyType { ... }
impl Iterator<String> for MyType { ... }

// Which one to call?
let mut iter = MyType::new();
iter.next();  // Returns i32 or String???
```

**Associated Types**:
```rust
trait Iterator {
    type Item;  // Associated type, not generic parameter
    fn next(&mut self) -> Option<Self::Item>;
}

// Can ONLY implement Iterator once per type
impl Iterator for MyType {
    type Item = i32;  // Pick specific type
    fn next(&mut self) -> Option<i32> { ... }
}

// Clear: MyType yields i32
let mut iter = MyType::new();
let item: Option<i32> = iter.next();
```

**When to Use Each**:

**Use Generic Parameters** when:
- Multiple implementations possible for the same type
- Caller chooses the type
```rust
trait From<T> {  // Can implement From<i32>, From<String>, etc.
    fn from(value: T) -> Self;
}
```

**Use Associated Types** when:
- Only one implementation makes sense
- Implementation chooses the type
```rust
trait Iterator {
    type Item;  // Iterator determines what it yields
}

trait Container {
    type Item;  // Container determines what it stores
}
```

---

### Generic Associated Types (GATs)

**Generic Associated Types** allow associated types to themselves be generic. This is advanced but powerful.

**The Problem Without GATs**:
```rust
trait Container<T> {
    type Iter: Iterator<Item = T>;  // Error: lifetime issue

    fn iter(&self) -> Self::Iter;  // How long does iterator live?
}
```

**With GATs**:
```rust
trait Container<T> {
    type Iter<'a>: Iterator<Item = &'a T>  // GAT: generic over lifetime!
    where
        Self: 'a,
        T: 'a;

    fn iter(&self) -> Self::Iter<'_>;  // Returns iterator borrowing self
}

impl<T, const N: usize> Container<T> for Stack<T, N> {
    type Iter<'a> = StackIter<'a, T, N>  // Concrete type for this lifetime
    where
        T: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        StackIter { stack: self, index: 0 }
    }
}
```

**Why This Matters**: Allows iterators that borrow from the container, which is how Rust's standard library works.

---

### Type-State Pattern with Zero-Sized Types

**Type-state pattern** uses the type system to encode state machine states, preventing invalid transitions at compile time.

**Zero-Sized Types (ZSTs)** are types that compile to nothing:
```rust
struct Empty;  // No fields
struct Ready;  // No fields

// These have ZERO size at runtime!
assert_eq!(std::mem::size_of::<Empty>(), 0);
assert_eq!(std::mem::size_of::<Ready>(), 0);
```

**Using ZSTs for Type-State**:
```rust
struct Builder<T, State> {
    value: Option<T>,
    _state: PhantomData<State>,  // ZST marker, zero runtime cost
}

impl<T> Builder<T, Empty> {
    fn new() -> Self {
        Builder { value: None, _state: PhantomData }
    }

    fn with_value(self, value: T) -> Builder<T, Ready> {
        Builder { value: Some(value), _state: PhantomData }
    }
}

impl<T> Builder<T, Ready> {
    fn build(self) -> T {  // Only Ready state can build!
        self.value.unwrap()
    }
}

// Usage:
let builder = Builder::new();  // Builder<T, Empty>
// builder.build();  // ✗ COMPILE ERROR: build() not on Empty

let builder = builder.with_value(42);  // Builder<i32, Ready>
let value = builder.build();  // ✓ OK: build() available on Ready
```

**Benefits**:
- **Compile-time safety**: Invalid state transitions impossible
- **Zero runtime cost**: State marker compiles away completely
- **Self-documenting**: API makes valid usage clear

**Real-World Example**:
```rust
// File builder with type-states
let file = FileBuilder::new()  // Builder<Unopened>
    .path("/tmp/data.txt")     // Builder<Unopened>
    .open()                     // Builder<Opened> - state change!
    .write(b"data")            // Builder<Opened>
    .close();                   // File - final state

// Can't write before opening (compile error)
// Can't open twice (compile error)
// Can't close before opening (compile error)
```

---

### Connection to This Project

In this project, you'll use all these concepts:

1. **Generics**: `Stack<T, N>`, `RingBuffer<T, N>`, `BinaryHeap<T, N>`
2. **Monomorphization**: Compiler generates specialized code for each type
3. **Const Generics**: `const N: usize` for compile-time sizes
4. **MaybeUninit<T>**: Safe uninitialized memory management
5. **Trait Bounds**: `T: Ord` for heap, `T: Clone` for builder
6. **Associated Types**: `Container::Iter<'a>` for iterators
7. **GATs**: Lifetime-parameterized associated types
8. **Type-State**: Builder pattern with `Empty`, `Configured`, `Ready` states
9. **ZSTs**: State markers with zero runtime cost

**Key Learning Points**:
- Generic code achieves zero-cost abstraction through monomorphization
- Const generics enable compile-time size guarantees
- MaybeUninit provides safe low-level memory control
- Trait bounds make generic code type-safe
- Type-state pattern prevents invalid operations at compile time

---

## Milestone 1: Generic Fixed-Size Stack with Const Generics

**Goal**: Implement a generic stack `Stack<T, const N: usize>` backed by a fixed-size array.

**Concepts**:
- Const generic parameters (`const N: usize`)
- `MaybeUninit<T>` for uninitialized memory
- Generic type parameter `<T>` with trait bounds
- Monomorphization and zero-cost abstractions
- Compile-time capacity checks

**Implementation Steps**:

1. **Define the `Stack<T, const N: usize>` struct**:
   - Use `[MaybeUninit<T>; N]` for storage (allows uninitialized elements)
   - Add `len: usize` field to track the number of elements
   - Derive `Debug` only when `T: Debug`

2. **Implement `new()` constructor**:
   - Initialize storage with `MaybeUninit::uninit_array()`
   - Set `len` to 0
   - Return `Self { storage, len }`

3. **Implement `push(&mut self, value: T) -> Result<(), T>`**:
   - Check if `self.len < N` (capacity check)
   - If full, return `Err(value)` (ownership back to caller)
   - Use `MaybeUninit::write()` to initialize the slot at `self.len`
   - Increment `self.len`
   - Return `Ok(())`

4. **Implement `pop(&mut self) -> Option<T>`**:
   - Check if `self.len > 0`
   - If empty, return `None`
   - Decrement `self.len`
   - Use `MaybeUninit::assume_init_read()` to read the value
   - Return `Some(value)`

5. **Implement `peek(&self) -> Option<&T>`**:
   - Check if `self.len > 0`
   - If empty, return `None`
   - Get reference to `self.storage[self.len - 1]`
   - Use `MaybeUninit::assume_init_ref()` to get `&T`
   - Return `Some(&value)`

6. **Implement `Drop` for cleanup**:
   - Loop from `0..self.len`
   - Call `MaybeUninit::assume_init_drop()` on each element
   - This ensures `T`'s destructor runs for all pushed elements

**Starter Code**:

```rust
use std::mem::MaybeUninit;

pub struct Stack<T, const N: usize> {
    // TODO: Add storage field using MaybeUninit<T>
    // Hint: [MaybeUninit<T>; N]

    // TODO: Add len field to track number of elements
}

impl<T, const N: usize> Stack<T, N> {
    pub fn new() -> Self {
        // TODO: Initialize storage with MaybeUninit::uninit_array()
        // TODO: Set len to 0

        todo!()
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        // TODO: Check if stack is full (len >= N)
        // If full, return Err(value)

        // TODO: Write value to storage[len] using MaybeUninit::write()

        // TODO: Increment len

        // TODO: Return Ok(())

        todo!()
    }

    pub fn pop(&mut self) -> Option<T> {
        // TODO: Check if stack is empty (len == 0)
        // If empty, return None

        // TODO: Decrement len

        // TODO: Read value from storage[len] using assume_init_read()

        // TODO: Return Some(value)

        todo!()
    }

    pub fn peek(&self) -> Option<&T> {
        // TODO: Check if stack is empty
        // If empty, return None

        // TODO: Get reference to storage[len - 1]
        // Use assume_init_ref() to get &T

        // TODO: Return Some(&value)

        todo!()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Drop for Stack<T, N> {
    fn drop(&mut self) {
        // TODO: Loop from 0..self.len
        // For each index, call assume_init_drop() on storage[i]
        // This ensures T's destructor runs

        todo!()
    }
}

// Only implement Debug when T implements Debug
impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for Stack<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries((0..self.len).map(|i| unsafe {
                self.storage[i].assume_init_ref()
            }))
            .finish()
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stack_is_empty() {
        let stack: Stack<i32, 10> = Stack::new();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
        assert_eq!(stack.capacity(), 10);
    }

    #[test]
    fn test_push_and_pop() {
        let mut stack: Stack<i32, 5> = Stack::new();

        assert_eq!(stack.push(1), Ok(()));
        assert_eq!(stack.push(2), Ok(()));
        assert_eq!(stack.push(3), Ok(()));
        assert_eq!(stack.len(), 3);

        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_push_when_full() {
        let mut stack: Stack<i32, 3> = Stack::new();

        assert_eq!(stack.push(1), Ok(()));
        assert_eq!(stack.push(2), Ok(()));
        assert_eq!(stack.push(3), Ok(()));
        assert!(stack.is_full());

        // Should return the value back since stack is full
        assert_eq!(stack.push(4), Err(4));
    }

    #[test]
    fn test_peek() {
        let mut stack: Stack<String, 5> = Stack::new();

        assert_eq!(stack.peek(), None);

        stack.push("hello".to_string()).unwrap();
        stack.push("world".to_string()).unwrap();

        assert_eq!(stack.peek(), Some(&"world".to_string()));
        assert_eq!(stack.len(), 2); // peek doesn't modify
    }

    #[test]
    fn test_drop_calls_destructor() {
        use std::sync::Arc;

        let value = Arc::new(42);
        assert_eq!(Arc::strong_count(&value), 1);

        {
            let mut stack: Stack<Arc<i32>, 5> = Stack::new();
            stack.push(Arc::clone(&value)).unwrap();
            stack.push(Arc::clone(&value)).unwrap();
            assert_eq!(Arc::strong_count(&value), 3);
        } // stack dropped here

        // Should be back to 1 (destructors ran)
        assert_eq!(Arc::strong_count(&value), 1);
    }

    #[test]
    fn test_different_types() {
        // Test with i32
        let mut stack_i32: Stack<i32, 10> = Stack::new();
        stack_i32.push(42).unwrap();
        assert_eq!(stack_i32.pop(), Some(42));

        // Test with String
        let mut stack_str: Stack<String, 10> = Stack::new();
        stack_str.push("test".to_string()).unwrap();
        assert_eq!(stack_str.pop(), Some("test".to_string()));

        // Test with custom struct
        #[derive(Debug, PartialEq)]
        struct Point { x: i32, y: i32 }

        let mut stack_point: Stack<Point, 10> = Stack::new();
        stack_point.push(Point { x: 1, y: 2 }).unwrap();
        assert_eq!(stack_point.pop(), Some(Point { x: 1, y: 2 }));
    }
}
```

**Check Your Understanding**:
1. Why do we use `MaybeUninit<T>` instead of `Option<T>` for storage?
2. What would happen if we didn't implement `Drop` for types that allocate?
3. How does the compiler generate different code for `Stack<i32, 100>` vs `Stack<String, 100>`?

---

## Milestone 2: Generic Ring Buffer with Circular Queuing

**Goal**: Implement a generic ring buffer `RingBuffer<T, const N: usize>` with circular indexing and FIFO behavior.

**Concepts**:
- Circular buffer algorithm with modulo arithmetic
- Distinguishing full vs empty states
- Generic constraints with `Default` trait
- Iterator implementation for generic types
- Compile-time capacity validation

**Implementation Steps**:

1. **Define the `RingBuffer<T, const N: usize>` struct**:
   - Use `[MaybeUninit<T>; N]` for storage
   - Add `head: usize` (read position)
   - Add `tail: usize` (write position)
   - Add `len: usize` (number of elements, distinguishes full from empty)

2. **Implement `new()` and `push(&mut self, value: T) -> Result<(), T>`**:
   - In `push`: write to `storage[tail]`, increment `tail` with `(tail + 1) % N`
   - If buffer is full (`len == N`), decide: overwrite oldest or return error
   - Implementation choice: overwrite and advance `head` (circular behavior)

3. **Implement `pop(&mut self) -> Option<T>`**:
   - Check if empty (`len == 0`)
   - Read from `storage[head]`, increment `head` with `(head + 1) % N`
   - Decrement `len`

4. **Implement `IntoIterator` for consuming iteration**:
   - Create iterator struct `RingBufferIter<T, const N: usize>`
   - Implement `Iterator` trait with `next()` calling `pop()`

5. **Handle edge cases**:
   - Empty buffer: `head == tail && len == 0`
   - Full buffer: `head == tail && len == N`
   - Wraparound: indices wrap using modulo

**Starter Code**:

```rust
use std::mem::MaybeUninit;

pub struct RingBuffer<T, const N: usize> {
    storage: [MaybeUninit<T>; N],
    head: usize,    // Read position
    tail: usize,    // Write position
    len: usize,     // Number of elements
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        // TODO: Initialize with MaybeUninit::uninit_array()
        // Set head, tail, len to 0

        todo!()
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        // TODO: Check if buffer is full (len == N)

        // If full, overwrite oldest element:
        // - Write to storage[tail]
        // - Advance tail: (tail + 1) % N
        // - Advance head: (head + 1) % N (to skip overwritten element)
        // - len stays at N

        // If not full:
        // - Write to storage[tail]
        // - Advance tail: (tail + 1) % N
        // - Increment len

        todo!()
    }

    pub fn pop(&mut self) -> Option<T> {
        // TODO: Check if empty (len == 0)
        // If empty, return None

        // TODO: Read from storage[head] using assume_init_read()

        // TODO: Advance head: (head + 1) % N

        // TODO: Decrement len

        // TODO: Return Some(value)

        todo!()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == N
    }

    pub fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Drop for RingBuffer<T, N> {
    fn drop(&mut self) {
        // TODO: Pop all remaining elements to run destructors
        while self.pop().is_some() {}
    }
}

// Iterator support
pub struct RingBufferIter<T, const N: usize> {
    buffer: RingBuffer<T, N>,
}

impl<T, const N: usize> Iterator for RingBufferIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Call pop() on the buffer
        self.buffer.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.buffer.len();
        (len, Some(len))
    }
}

impl<T, const N: usize> IntoIterator for RingBuffer<T, N> {
    type Item = T;
    type IntoIter = RingBufferIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        RingBufferIter { buffer: self }
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push_pop() {
        let mut buf: RingBuffer<i32, 4> = RingBuffer::new();

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        assert_eq!(buf.pop(), Some(1));
        assert_eq!(buf.pop(), Some(2));

        buf.push(4).unwrap();
        buf.push(5).unwrap();

        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_ring_buffer_wraparound() {
        let mut buf: RingBuffer<i32, 3> = RingBuffer::new();

        // Fill buffer
        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        // Now full, next push should overwrite
        buf.push(4).unwrap(); // Overwrites 1
        buf.push(5).unwrap(); // Overwrites 2

        assert_eq!(buf.pop(), Some(3));
        assert_eq!(buf.pop(), Some(4));
        assert_eq!(buf.pop(), Some(5));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn test_ring_buffer_iterator() {
        let mut buf: RingBuffer<i32, 5> = RingBuffer::new();

        buf.push(10).unwrap();
        buf.push(20).unwrap();
        buf.push(30).unwrap();

        let collected: Vec<i32> = buf.into_iter().collect();
        assert_eq!(collected, vec![10, 20, 30]);
    }

    #[test]
    fn test_ring_buffer_fifo_order() {
        let mut buf: RingBuffer<char, 4> = RingBuffer::new();

        buf.push('a').unwrap();
        buf.push('b').unwrap();
        buf.push('c').unwrap();

        assert_eq!(buf.pop(), Some('a')); // FIFO: first in, first out

        buf.push('d').unwrap();
        buf.push('e').unwrap();

        assert_eq!(buf.pop(), Some('b'));
        assert_eq!(buf.pop(), Some('c'));
        assert_eq!(buf.pop(), Some('d'));
    }

    #[test]
    fn test_ring_buffer_size_hint() {
        let mut buf: RingBuffer<i32, 5> = RingBuffer::new();

        buf.push(1).unwrap();
        buf.push(2).unwrap();
        buf.push(3).unwrap();

        let iter = buf.into_iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
    }
}
```

**Check Your Understanding**:
1. How does the ring buffer distinguish between full and empty states when `head == tail`?
2. Why do we use modulo arithmetic instead of checking bounds explicitly?
3. What are the trade-offs between overwriting old data vs returning an error when full?

---

## Milestone 3: Generic Binary Heap with Ordering

**Goal**: Implement a generic min-heap or max-heap `BinaryHeap<T, const N: usize>` with trait bounds for ordering.

**Concepts**:
- Heap property: parent-child ordering relationships
- Generic trait bounds: `T: Ord` for comparison
- Heap algorithms: `heapify_up`, `heapify_down`
- Parent/child index calculations: `parent = (i - 1) / 2`, `left = 2*i + 1`
- Comparison abstraction with trait bounds

**Implementation Steps**:

1. **Define `BinaryHeap<T, const N: usize>` struct**:
   - Storage: `[MaybeUninit<T>; N]`
   - Track `len: usize`
   - Constraint: `T: Ord` (required for comparison)

2. **Implement `push(&mut self, value: T) -> Result<(), T>`**:
   - Check capacity (`len < N`)
   - Insert at `storage[len]`
   - Call `heapify_up(len)` to restore heap property
   - Increment `len`

3. **Implement `heapify_up(&mut self, index: usize)`**:
   - While `index > 0`:
     - Calculate `parent = (index - 1) / 2`
     - Compare `storage[index]` with `storage[parent]`
     - If heap property violated, swap and continue with parent
     - Otherwise, break

4. **Implement `pop(&mut self) -> Option<T>`**:
   - Check if empty
   - Save root (`storage[0]`)
   - Move last element to root
   - Decrement `len`
   - Call `heapify_down(0)` to restore heap property
   - Return saved root

5. **Implement `heapify_down(&mut self, index: usize)`**:
   - While `index` has children:
     - Calculate `left = 2 * index + 1`, `right = 2 * index + 2`
     - Find smallest/largest child
     - If heap property violated, swap and continue with child
     - Otherwise, break

6. **Add `peek(&self) -> Option<&T>` to view root without removing**

**Starter Code**:

```rust
use std::mem::MaybeUninit;

pub struct BinaryHeap<T: Ord, const N: usize> {
    storage: [MaybeUninit<T>; N],
    len: usize,
}

impl<T: Ord, const N: usize> BinaryHeap<T, N> {
    pub fn new() -> Self {
        Self {
            storage: MaybeUninit::uninit_array(),
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        // TODO: Check if full
        if self.len >= N {
            return Err(value);
        }

        // TODO: Insert at end
        self.storage[self.len].write(value);

        // TODO: Heapify up from len
        self.heapify_up(self.len);

        self.len += 1;
        Ok(())
    }

    fn heapify_up(&mut self, mut index: usize) {
        // TODO: While index > 0
        while index > 0 {
            // TODO: Calculate parent index
            let parent = (index - 1) / 2;

            // TODO: Compare child with parent
            // Hint: Use assume_init_ref() to get &T for comparison
            let child_ref = unsafe { self.storage[index].assume_init_ref() };
            let parent_ref = unsafe { self.storage[parent].assume_init_ref() };

            // For max-heap: if child > parent, swap
            // For min-heap: if child < parent, swap
            // TODO: Implement max-heap (largest at root)
            if child_ref > parent_ref {
                self.storage.swap(index, parent);
                index = parent;
            } else {
                break;
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        // TODO: Check if empty
        if self.len == 0 {
            return None;
        }

        // TODO: Save root element
        self.len -= 1;

        // Swap root with last element
        self.storage.swap(0, self.len);

        // Read the (now last) element
        let value = unsafe { self.storage[self.len].assume_init_read() };

        // TODO: Heapify down from root (if not empty)
        if self.len > 0 {
            self.heapify_down(0);
        }

        Some(value)
    }

    fn heapify_down(&mut self, mut index: usize) {
        loop {
            // TODO: Calculate left and right child indices
            let left = 2 * index + 1;
            let right = 2 * index + 2;

            // TODO: Find largest among parent, left, right
            let mut largest = index;

            if left < self.len {
                let left_ref = unsafe { self.storage[left].assume_init_ref() };
                let largest_ref = unsafe { self.storage[largest].assume_init_ref() };
                if left_ref > largest_ref {
                    largest = left;
                }
            }

            if right < self.len {
                let right_ref = unsafe { self.storage[right].assume_init_ref() };
                let largest_ref = unsafe { self.storage[largest].assume_init_ref() };
                if right_ref > largest_ref {
                    largest = right;
                }
            }

            // TODO: If largest is not parent, swap and continue
            if largest != index {
                self.storage.swap(index, largest);
                index = largest;
            } else {
                break;
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        // TODO: Return reference to root if not empty
        if self.len > 0 {
            Some(unsafe { self.storage[0].assume_init_ref() })
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Ord, const N: usize> Drop for BinaryHeap<T, N> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe {
                self.storage[i].assume_init_drop();
            }
        }
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heap_push_pop_order() {
        let mut heap: BinaryHeap<i32, 10> = BinaryHeap::new();

        heap.push(5).unwrap();
        heap.push(3).unwrap();
        heap.push(7).unwrap();
        heap.push(1).unwrap();
        heap.push(9).unwrap();

        // Max-heap should pop in descending order
        assert_eq!(heap.pop(), Some(9));
        assert_eq!(heap.pop(), Some(7));
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(3));
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn test_heap_peek() {
        let mut heap: BinaryHeap<i32, 5> = BinaryHeap::new();

        assert_eq!(heap.peek(), None);

        heap.push(10).unwrap();
        heap.push(20).unwrap();
        heap.push(5).unwrap();

        assert_eq!(heap.peek(), Some(&20)); // Max element
        assert_eq!(heap.len(), 3); // Peek doesn't remove
    }

    #[test]
    fn test_heap_with_strings() {
        let mut heap: BinaryHeap<String, 5> = BinaryHeap::new();

        heap.push("apple".to_string()).unwrap();
        heap.push("zebra".to_string()).unwrap();
        heap.push("banana".to_string()).unwrap();

        // Lexicographic order
        assert_eq!(heap.pop(), Some("zebra".to_string()));
        assert_eq!(heap.pop(), Some("banana".to_string()));
        assert_eq!(heap.pop(), Some("apple".to_string()));
    }

    #[test]
    fn test_heap_capacity() {
        let mut heap: BinaryHeap<i32, 3> = BinaryHeap::new();

        assert_eq!(heap.push(1), Ok(()));
        assert_eq!(heap.push(2), Ok(()));
        assert_eq!(heap.push(3), Ok(()));

        // Should fail when full
        assert_eq!(heap.push(4), Err(4));
    }

    #[test]
    fn test_heap_property_maintained() {
        let mut heap: BinaryHeap<i32, 10> = BinaryHeap::new();

        for i in 0..10 {
            heap.push(i).unwrap();
        }

        let mut prev = heap.pop().unwrap();
        while let Some(current) = heap.pop() {
            assert!(prev >= current, "Heap property violated");
            prev = current;
        }
    }
}
```

**Check Your Understanding**:
1. How does the heap property differ between min-heap and max-heap?
2. Why is `T: Ord` required instead of `T: PartialOrd` for the heap?
3. What is the time complexity of `push` and `pop` operations?

---

## Milestone 4: Generic Container Trait with Associated Types

**Goal**: Define a generic `Container<T>` trait with associated types for iterators, and implement it for all data structures.

**Concepts**:
- Trait definition with associated types
- Generic trait implementations
- Iterator associated types
- Trait bounds for implementations
- Code reuse through trait abstractions

**Implementation Steps**:

1. **Define `Container<T>` trait**:
   - Associated type `Iter` for iterator
   - Required methods: `len()`, `is_empty()`, `clear()`, `iter()`
   - Optional methods with default implementations

2. **Implement `Container<T>` for `Stack<T, N>`**:
   - Define `StackIter<'a, T, N>` iterator struct
   - Implement `Iterator` trait for `StackIter`
   - Connect via associated type `Iter = StackIter<'a, T, N>`

3. **Implement `Container<T>` for `RingBuffer<T, N>` and `BinaryHeap<T, N>`**:
   - Create corresponding iterator types
   - Implement trait methods

4. **Add generic function using `Container<T>` trait bound**:
   - Example: `fn print_all<C: Container<T>, T: Display>(container: &C)`
   - Demonstrates polymorphism through trait bounds

**Starter Code**:

```rust
use std::fmt::Display;

pub trait Container<T> {
    type Iter<'a>: Iterator<Item = &'a T>
    where
        T: 'a,
        Self: 'a;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn iter(&self) -> Self::Iter<'_>;

    // Optional: mutable clear
    fn clear(&mut self);
}

// Implement for Stack
impl<T, const N: usize> Container<T> for Stack<T, N> {
    type Iter<'a> = StackIter<'a, T, N>
    where
        T: 'a;

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> Self::Iter<'_> {
        StackIter {
            stack: self,
            index: 0,
        }
    }

    fn clear(&mut self) {
        // TODO: Pop all elements
        while self.pop().is_some() {}
    }
}

pub struct StackIter<'a, T, const N: usize> {
    stack: &'a Stack<T, N>,
    index: usize,
}

impl<'a, T, const N: usize> Iterator for StackIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Iterate from bottom to top (0 to len)
        if self.index < self.stack.len {
            let item = unsafe { self.stack.storage[self.index].assume_init_ref() };
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

// Generic function using Container trait
pub fn print_all<T, C>(container: &C)
where
    T: Display,
    C: Container<T>,
{
    for item in container.iter() {
        println!("{}", item);
    }
}

// Generic function to count elements matching predicate
pub fn count_matching<T, C, F>(container: &C, predicate: F) -> usize
where
    C: Container<T>,
    F: Fn(&T) -> bool,
{
    // TODO: Use iterator and filter with predicate
    container.iter().filter(|item| predicate(item)).count()
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_container_trait() {
        let mut stack: Stack<i32, 10> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();

        assert_eq!(stack.len(), 3);
        assert!(!stack.is_empty());

        let collected: Vec<&i32> = stack.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
    }

    #[test]
    fn test_container_clear() {
        let mut stack: Stack<i32, 5> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();

        stack.clear();
        assert_eq!(stack.len(), 0);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_generic_print_all() {
        let mut stack: Stack<i32, 5> = Stack::new();
        stack.push(10).unwrap();
        stack.push(20).unwrap();

        // Should compile and run without panic
        print_all(&stack);
    }

    #[test]
    fn test_count_matching() {
        let mut stack: Stack<i32, 10> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();
        stack.push(4).unwrap();
        stack.push(5).unwrap();

        let even_count = count_matching(&stack, |&x| x % 2 == 0);
        assert_eq!(even_count, 2); // 2 and 4

        let greater_than_3 = count_matching(&stack, |&x| x > 3);
        assert_eq!(greater_than_3, 2); // 4 and 5
    }

    #[test]
    fn test_container_polymorphism() {
        fn sum_all<T, C>(container: &C) -> T
        where
            T: std::ops::Add<Output = T> + Default + Copy,
            C: Container<T>,
        {
            container.iter().copied().fold(T::default(), |acc, x| acc + x)
        }

        let mut stack: Stack<i32, 5> = Stack::new();
        stack.push(1).unwrap();
        stack.push(2).unwrap();
        stack.push(3).unwrap();

        assert_eq!(sum_all(&stack), 6);
    }
}
```

**Check Your Understanding**:
1. Why use associated types (`type Iter`) instead of generic type parameters?
2. How does the `Container` trait enable polymorphism across different data structures?
3. What are the benefits of GATs (Generic Associated Types) in the `Iter` definition?

---

## Milestone 5: Builder Pattern with Generic Constraints

**Goal**: Implement a builder pattern for creating containers with custom configuration using generics.

**Concepts**:
- Builder pattern with type-state
- Generic constraints with `Default`, `Clone` traits
- Method chaining
- Consuming builders
- Zero-sized types (ZSTs) for state

**Implementation Steps**:

1. **Define `ContainerBuilder<T, const N: usize, State>` struct**:
   - Use phantom type `State` for type-state pattern
   - States: `Empty`, `Configured`, `Ready`
   - Each state is a zero-sized type (ZST)

2. **Implement builder methods with state transitions**:
   - `new() -> ContainerBuilder<T, N, Empty>`
   - `with_default(value: T) -> ContainerBuilder<T, N, Configured>` (requires `T: Clone`)
   - `fill() -> ContainerBuilder<T, N, Ready>` (requires `T: Default`)
   - `build() -> Stack<T, N>` (only on `Ready` state)

3. **Add compile-time state guarantees**:
   - Cannot call `build()` on `Empty` state (compile error)
   - Cannot call `with_default()` twice
   - Type system enforces valid construction sequences

4. **Demonstrate zero-cost abstraction**:
   - All builder types are ZSTs (size 0 at runtime)
   - State transitions happen at compile time only
   - Final `build()` produces actual container with no runtime overhead

**Starter Code**:

```rust
use std::marker::PhantomData;

// State marker types (ZSTs)
pub struct Empty;
pub struct Configured;
pub struct Ready;

pub struct ContainerBuilder<T, const N: usize, State> {
    config: Option<T>,
    _state: PhantomData<State>,
}

impl<T, const N: usize> ContainerBuilder<T, N, Empty> {
    pub fn new() -> Self {
        Self {
            config: None,
            _state: PhantomData,
        }
    }

    // Transition to Configured state
    pub fn with_default(self, value: T) -> ContainerBuilder<T, N, Configured>
    where
        T: Clone,
    {
        // TODO: Create Configured builder with value
        ContainerBuilder {
            config: Some(value),
            _state: PhantomData,
        }
    }

    // Transition to Ready state (for types with Default)
    pub fn with_defaults(self) -> ContainerBuilder<T, N, Ready>
    where
        T: Default,
    {
        // TODO: Create Ready builder
        ContainerBuilder {
            config: None,
            _state: PhantomData,
        }
    }
}

impl<T, const N: usize> ContainerBuilder<T, N, Configured> {
    // Transition to Ready state
    pub fn ready(self) -> ContainerBuilder<T, N, Ready> {
        ContainerBuilder {
            config: self.config,
            _state: PhantomData,
        }
    }
}

impl<T, const N: usize> ContainerBuilder<T, N, Ready> {
    // Only Ready state can build
    pub fn build(self) -> Stack<T, N>
    where
        T: Clone,
    {
        let mut stack = Stack::new();

        // TODO: If config has a default value, fill the stack
        if let Some(default) = self.config {
            for _ in 0..N {
                // Don't fail if full, just stop
                if stack.push(default.clone()).is_err() {
                    break;
                }
            }
        }

        stack
    }

    // Build with custom initializer
    pub fn build_with<F>(self, mut init: F) -> Stack<T, N>
    where
        F: FnMut(usize) -> T,
    {
        let mut stack = Stack::new();

        // TODO: Initialize each element with init function
        for i in 0..N {
            let value = init(i);
            if stack.push(value).is_err() {
                break;
            }
        }

        stack
    }
}
```

**Checkpoint Tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_with_default() {
        let stack: Stack<i32, 5> = ContainerBuilder::new()
            .with_default(42)
            .ready()
            .build();

        assert_eq!(stack.len(), 5);
        assert_eq!(stack.peek(), Some(&42));
    }

    #[test]
    fn test_builder_with_defaults() {
        let stack: Stack<i32, 3> = ContainerBuilder::new()
            .with_defaults()
            .build();

        // Should be empty (Default for i32 is 0, but we don't auto-fill)
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_builder_with_initializer() {
        let stack: Stack<i32, 5> = ContainerBuilder::new()
            .with_defaults()
            .build_with(|i| (i * 2) as i32);

        assert_eq!(stack.len(), 5);
        assert_eq!(stack.pop(), Some(8)); // i=4 -> 4*2=8
        assert_eq!(stack.pop(), Some(6)); // i=3 -> 3*2=6
    }

    // This should NOT compile (uncomment to verify):
    // #[test]
    // fn test_builder_invalid_state() {
    //     let stack = ContainerBuilder::<i32, 5, Empty>::new()
    //         .build(); // ERROR: build() not available on Empty state
    // }

    #[test]
    fn test_builder_zero_size() {
        use std::mem::size_of;

        // All builder states should be zero-sized
        assert_eq!(size_of::<ContainerBuilder<i32, 100, Empty>>(), size_of::<Option<i32>>());
        assert_eq!(size_of::<ContainerBuilder<i32, 100, Configured>>(), size_of::<Option<i32>>());
        assert_eq!(size_of::<ContainerBuilder<i32, 100, Ready>>(), size_of::<Option<i32>>());

        // State markers are ZSTs
        assert_eq!(size_of::<Empty>(), 0);
        assert_eq!(size_of::<Configured>(), 0);
        assert_eq!(size_of::<Ready>(), 0);
    }

    #[test]
    fn test_builder_method_chaining() {
        let stack = ContainerBuilder::new()
            .with_default(String::from("test"))
            .ready()
            .build();

        assert_eq!(stack.len(), 5);
    }
}
```

**Check Your Understanding**:
1. How do zero-sized types (ZSTs) enable compile-time state checking with no runtime cost?
2. Why does the builder pattern prevent calling `build()` on the wrong state?
3. What are the trade-offs between builder pattern and direct construction?

---

## Summary

You've built a **complete generic data structure library** with:

1. **Generic Fixed-Size Stack** with `MaybeUninit<T>` and const generics
2. **Generic Ring Buffer** with circular indexing and FIFO behavior
3. **Generic Binary Heap** with heap property and `T: Ord` constraint
4. **Generic Container Trait** with associated types for abstraction
5. **Builder Pattern** with type-state and zero-sized types (ZSTs)

**Key Patterns Learned**:
- **Const generics** for compile-time size guarantees
- **MaybeUninit<T>** for uninitialized memory safety
- **Trait bounds** (`Ord`, `Clone`, `Default`) for generic constraints
- **Monomorphization** and zero-cost abstractions
- **Associated types** vs generic type parameters
- **Type-state pattern** with phantom types
- **Drop** implementation for RAII and destructors

**Performance Characteristics**:
- **Stack allocation**: 0ns allocation cost vs 50-100ns heap
- **Monomorphization**: Specialized machine code per type (no vtable overhead)
- **Cache locality**: Contiguous memory layout improves performance
- **Const generics**: Compiler knows sizes, enables better optimization
- **Zero-cost abstractions**: Generic code compiles to same ASM as hand-written

**Real-World Applications**:
- Embedded systems (fixed-size, no heap)
- Real-time systems (predictable performance)
- High-frequency trading (low latency)
- Game engines (cache-friendly data structures)
- Database indexes (B-trees, heaps)

**Next Steps**:
- Add support for custom allocators (e.g., arena allocators)
- Implement more complex structures (B-tree, skip list)
- Add const generic expressions when stabilized
- Benchmark against std library equivalents
- Explore SIMD optimizations for numeric types
