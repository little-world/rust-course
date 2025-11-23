# Generics & Polymorphism


[Pattern 1: Type-Safe Generic Functions](#pattern-1-type-safe-generic-functions)

- Problem: Writing duplicate code for operations that differ only by type
- Solution: Use generic type parameters with trait bounds for type-safe abstraction
- Why It Matters: Monomorphization produces specialized code with zero runtime overhead
- Use Cases: Container methods, sorting, serialization, mathematical operations

[Pattern 2: Generic Structs and Enums](#pattern-2-generic-structs-and-enums)

- Problem: Data structures should work with any type, not just specific ones
- Solution: Parameterize structs and enums over types with `<T>`
- Why It Matters: Enables reusable, type-safe containers like `Vec<T>`, `Option<T>`
- Use Cases: Collections, result wrappers, configuration, state machines

[Pattern 3: Trait Bounds and Constraints](#pattern-3-trait-bounds-and-constraints)

- Problem: Generic code needs specific operations but `T` could be anything
- Solution: Constrain generic types with trait bounds using `where` clauses
- Why It Matters: Provides compile-time guarantees about type capabilities
- Use Cases: Serialization, comparison, hashing, formatting, cloning

[Pattern 4: Associated Types vs Generic Parameters](#pattern-4-associated-types-vs-generic-parameters)

- Problem: Deciding between `trait Foo<T>` and `trait Foo { type T; }`
- Solution: Use associated types for "output" types, generics for "input" types
- Why It Matters: Associated types simplify APIs and prevent trait implementation ambiguity
- Use Cases: Iterator::Item, collection element types, conversion traits

[Pattern 5: Blanket Implementations](#pattern-5-blanket-implementations)

- Problem: Providing default implementations for all types meeting certain criteria
- Solution: Use `impl<T> Trait for T where T: OtherTrait` patterns
- Why It Matters: Enables powerful trait composition without manual impl for each type
- Use Cases: `ToString` for `Display`, `From`/`Into` conversions, iterator adapters

[Pattern 6: Phantom Types and Type-Level State](#pattern-6-phantom-types-and-type-level-state)

- Problem: Encoding state or constraints that exist only at compile time
- Solution: Use `PhantomData<T>` to add type parameters without runtime cost
- Why It Matters: Enables type-state pattern for compile-time correctness guarantees
- Use Cases: Units of measure, authentication state, builder validation, FFI markers

[Pattern 7: Higher-Ranked Trait Bounds (HRTBs)](#pattern-7-higher-ranked-trait-bounds-hrtbs)

- Problem: Functions that work with closures taking references of any lifetime
- Solution: Use `for<'a>` syntax to express "for all lifetimes"
- Why It Matters: Essential for closure-heavy APIs and higher-order functions
- Use Cases: Iterator adapters, callback registries, parser combinators

[Pattern 8: Const Generics](#pattern-8-const-generics)

- Problem: Parameterizing types by compile-time constants, not just types
- Solution: Use `<const N: usize>` syntax to parameterize by values
- Why It Matters: Enables fixed-size arrays, matrices, and compile-time computations
- Use Cases: Fixed-size buffers, matrices, protocol frames, SIMD vectors

[Generics Cheat Sheet](#generics-cheat-sheet)
- Comprehensive reference for generic syntax and patterns

### Overview

Generics are Rust's mechanism for writing code that works across multiple types while maintaining full type safety. Unlike dynamic languages that achieve polymorphism through runtime type checking, Rust's generics are resolved entirely at compile time through **monomorphization**—the compiler generates specialized versions of generic code for each concrete type used.

This approach delivers three critical benefits:
1. **Zero runtime overhead**: Generic code runs as fast as hand-written specialized code
2. **Type safety**: Errors caught at compile time, not runtime
3. **Code reuse**: Write once, use with any type that satisfies constraints

Rust's generics go beyond simple type parameters. Combined with the trait system, they enable:
- **Bounded polymorphism**: Require types to implement specific capabilities
- **Associated types**: Simplify trait APIs by fixing "output" types
- **Const generics**: Parameterize by compile-time values, not just types
- **Higher-kinded patterns**: Express relationships between types

For experienced programmers, mastering generics means understanding when to use them, how to constrain them effectively, and recognizing patterns that leverage the type system for correctness guarantees.

## Type System Foundation

```rust
// Basic generic syntax
fn foo<T>(x: T) -> T { x }                          // Generic function
struct Pair<T, U> { first: T, second: U }           // Generic struct
enum Result<T, E> { Ok(T), Err(E) }                 // Generic enum
impl<T> Pair<T, T> { }                              // Generic impl

// Trait bounds (constraints)
fn print<T: Display>(x: T) { }                      // Single bound
fn both<T: Debug + Clone>(x: T) { }                 // Multiple bounds
fn complex<T>(x: T) where T: Debug + Clone { }      // Where clause
fn lifetime<'a, T: 'a>(x: &'a T) { }               // Lifetime bounds

// Associated types
trait Iterator { type Item; }                       // Type determined by impl
trait Graph { type Node; type Edge; }               // Multiple associated types

// Const generics
struct Array<T, const N: usize>([T; N]);           // Parameterized by value
fn fixed<const N: usize>() -> [u8; N] { [0; N] }   // Const in return type

// Higher-ranked trait bounds
fn hrtb<F>(f: F) where F: for<'a> Fn(&'a str) { }  // Works for any lifetime

// Phantom types
use std::marker::PhantomData;
struct Tagged<T, Tag> { value: T, _tag: PhantomData<Tag> }
```

## Pattern 1: Type-Safe Generic Functions

**Problem**: You need to write functions that perform the same operation on different types. Without generics, you'd either duplicate code for each type (error-prone, unmaintainable) or use dynamic typing with runtime casts (unsafe, slow).

**Solution**: Define functions with type parameters `<T>` that can be instantiated with any concrete type. Add trait bounds to constrain `T` to types that support required operations. The compiler generates specialized machine code for each type used.

**Why It Matters**: Monomorphization means generic code is as fast as hand-written specialized code—there's no vtable lookup, no boxing, no dynamic dispatch. The `Vec::push` you call on `Vec<i32>` is different compiled code than `Vec::push` on `Vec<String>`, each optimized for its type.

**Use Cases**: Container operations, sorting algorithms, serialization/deserialization, mathematical computations, comparison utilities, formatting helpers.

### Examples
```rust
use std::fmt::Display;
use std::cmp::Ordering;

//======================================================
// Pattern: Basic generic function with type inference
//======================================================
fn identity<T>(x: T) -> T {
    x
}

let num = identity(42);           // T inferred as i32
let text = identity("hello");     // T inferred as &str

//============================================================
// Pattern: Generic function with trait bound for operations
//============================================================
fn largest<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut largest = list.first()?;
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    Some(largest)
}

let numbers = vec![34, 50, 25, 100, 65];
assert_eq!(largest(&numbers), Some(&100));

let chars = vec!['y', 'm', 'a', 'q'];
assert_eq!(largest(&chars), Some(&'y'));

//=============================================================
// Pattern: Multiple trait bounds for complex requirements
//=============================================================
fn print_sorted<T>(mut items: Vec<T>)
where
    T: Ord + Display,    // Sortable AND printable
{
    items.sort();
    for item in items {
        println!("{}", item);
    }
}

//=====================================================
// Pattern: Generic function returning owned vs borrowed
//=====================================================
// Returns owned value - caller gets ownership
fn create_default<T: Default>() -> T {
    T::default()
}

// Returns reference - borrows from input
fn first<T>(slice: &[T]) -> Option<&T> {
    slice.first()
}

// Returns reference with explicit lifetime
fn longest<'a, T>(x: &'a [T], y: &'a [T]) -> &'a [T] {
    if x.len() > y.len() { x } else { y }
}

//=========================================================
// Pattern: Turbofish syntax for explicit type specification
//=========================================================
let parsed = "42".parse::<i32>().unwrap();      // Turbofish ::<i32>
let default = create_default::<String>();        // Explicit type
let collected: Vec<i32> = (0..10).collect();    // Type annotation alternative

//============================================
// Pattern: Generic comparison with borrowing
//============================================
fn compare<T: Ord>(a: &T, b: &T) -> Ordering {
    a.cmp(b)
}

fn min_ref<'a, T: Ord>(a: &'a T, b: &'a T) -> &'a T {
    if a <= b { a } else { b }
}

//===============================================
// Pattern: Generic function with default values
//===============================================
fn get_or_default<T: Default>(opt: Option<T>) -> T {
    opt.unwrap_or_default()
}

fn get_or<T>(opt: Option<T>, default: T) -> T {
    opt.unwrap_or(default)
}

fn get_or_else<T, F: FnOnce() -> T>(opt: Option<T>, f: F) -> T {
    opt.unwrap_or_else(f)
}
```

**When to use generic functions:**
- Operations that apply uniformly across types
- When you need zero-cost abstraction
- When trait bounds express the required capabilities
- When type inference reduces boilerplate

**Performance characteristics:**
- Monomorphization: separate code generated per type
- Zero runtime overhead vs hand-written specialized code
- Increased compile time and binary size (trade-off)
- Inlining opportunities for small generic functions

## Pattern 2: Generic Structs and Enums

**Problem**: Data structures need to store values of various types. Hard-coding types limits reusability—you'd need `IntStack`, `StringStack`, `FloatStack`, etc. Using `Box<dyn Any>` loses type safety and incurs runtime overhead.

**Solution**: Parameterize structs and enums over type parameters. `struct Stack<T>` can hold any type while remaining fully type-safe. Each instantiation creates a specialized type: `Stack<i32>` and `Stack<String>` are distinct types.

**Why It Matters**: Generic data structures are the foundation of Rust's standard library. `Vec<T>`, `HashMap<K, V>`, `Option<T>`, `Result<T, E>`—these are all generic types. Understanding how to design generic structures enables building reusable, composable abstractions.

**Use Cases**: Collections (vectors, maps, sets), option/result wrappers, configuration containers, state machines, protocol messages, tree structures.

### Examples
```rust
//=====================================
// Pattern: Basic generic struct
//=====================================
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Methods requiring specific traits
impl<T: Copy + std::ops::Add<Output = T>> Point<T> {
    fn add(&self, other: &Point<T>) -> Point<T> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

let int_point = Point::new(5, 10);
let float_point = Point::new(1.0, 4.0);

//==========================================
// Pattern: Multiple type parameters
//==========================================
struct Pair<T, U> {
    first: T,
    second: U,
}

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self {
        Pair { first, second }
    }

    // Method mixing type parameters
    fn swap(self) -> Pair<U, T> {
        Pair {
            first: self.second,
            second: self.first,
        }
    }
}

// Method with different generics than struct
impl<T, U> Pair<T, U> {
    fn mix<V, W>(self, other: Pair<V, W>) -> Pair<T, W> {
        Pair {
            first: self.first,
            second: other.second,
        }
    }
}

//======================================
// Pattern: Generic enum with variants
//======================================
enum BinaryTree<T> {
    Empty,
    Node {
        value: T,
        left: Box<BinaryTree<T>>,
        right: Box<BinaryTree<T>>,
    },
}

impl<T: Ord> BinaryTree<T> {
    fn new() -> Self {
        BinaryTree::Empty
    }

    fn insert(&mut self, new_value: T) {
        match self {
            BinaryTree::Empty => {
                *self = BinaryTree::Node {
                    value: new_value,
                    left: Box::new(BinaryTree::Empty),
                    right: Box::new(BinaryTree::Empty),
                };
            }
            BinaryTree::Node { value, left, right } => {
                if new_value <= *value {
                    left.insert(new_value);
                } else {
                    right.insert(new_value);
                }
            }
        }
    }

    fn contains(&self, target: &T) -> bool {
        match self {
            BinaryTree::Empty => false,
            BinaryTree::Node { value, left, right } => {
                match target.cmp(value) {
                    std::cmp::Ordering::Equal => true,
                    std::cmp::Ordering::Less => left.contains(target),
                    std::cmp::Ordering::Greater => right.contains(target),
                }
            }
        }
    }
}

//===============================================
// Pattern: Generic wrapper with transformation
//===============================================
struct Wrapper<T> {
    value: T,
}

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper { value }
    }

    fn into_inner(self) -> T {
        self.value
    }

    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Wrapper<U> {
        Wrapper { value: f(self.value) }
    }

    fn as_ref(&self) -> Wrapper<&T> {
        Wrapper { value: &self.value }
    }
}

//============================================
// Pattern: Specialized impl for specific types
//============================================
impl<T> Point<T> {
    fn get_x(&self) -> &T {
        &self.x
    }
}

// Only available for f64
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

// Only available for numeric types
impl<T: std::ops::Mul<Output = T> + Copy> Point<T> {
    fn scale(&self, factor: T) -> Point<T> {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

//===========================================
// Pattern: Generic struct with lifetime
//===========================================
struct Ref<'a, T> {
    value: &'a T,
}

impl<'a, T> Ref<'a, T> {
    fn new(value: &'a T) -> Self {
        Ref { value }
    }
}

struct MutRef<'a, T> {
    value: &'a mut T,
}

//==================================================
// Pattern: Default bounds for common functionality
//==================================================
#[derive(Debug, Clone, PartialEq)]
struct Container<T> {
    items: Vec<T>,
}

impl<T> Default for Container<T> {
    fn default() -> Self {
        Container { items: Vec::new() }
    }
}

impl<T> Container<T> {
    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
}
```

**Design guidelines:**
- Prefer single type parameter when elements are homogeneous
- Use multiple parameters for relationships (key-value, input-output)
- Add trait bounds at impl level, not struct definition
- Consider `Default` implementation for generic containers

**Common patterns:**
- `Wrapper<T>`: Add behavior to any type
- `Container<T>`: Store multiple values
- `Tree<T>`, `Graph<T>`: Recursive structures
- `Result<T, E>`: Success or error
- `State<S, T>`: Type-state machines

## Pattern 3: Trait Bounds and Constraints

**Problem**: Generic functions need to perform operations on their type parameters, but `T` could be any type—how do you call `.clone()` on `T` if `T` might not implement `Clone`? Without constraints, you can only move, drop, or pass the value around.

**Solution**: Use trait bounds to constrain type parameters to types implementing specific traits. Bounds can be inline (`fn foo<T: Clone>`) or in where clauses (`where T: Clone + Debug`). Multiple bounds create intersection requirements.

**Why It Matters**: Trait bounds are the contract between generic code and its callers. They specify exactly what capabilities are required, enabling the compiler to verify correctness. Well-chosen bounds make APIs flexible yet safe—accept the minimum needed, not more.

**Use Cases**: Requiring comparison (`Ord`, `PartialOrd`), hashing (`Hash`), serialization (`Serialize`), formatting (`Debug`, `Display`), cloning (`Clone`), default construction (`Default`).

### Examples
```rust
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::collections::HashMap;

//=================================
// Pattern: Single trait bound
//=================================
fn print_debug<T: Debug>(value: T) {
    println!("{:?}", value);
}

fn clone_it<T: Clone>(value: &T) -> T {
    value.clone()
}

//=================================
// Pattern: Multiple trait bounds
//=================================
// Using + syntax
fn compare_and_show<T: PartialOrd + Display>(a: T, b: T) {
    if a < b {
        println!("{} < {}", a, b);
    }
}

// Using where clause (cleaner for complex bounds)
fn complex_operation<T, U>(t: T, u: U)
where
    T: Clone + Debug + Default,
    U: AsRef<str> + Display,
{
    println!("T: {:?}, U: {}", t.clone(), u);
}

//=============================================
// Pattern: Bounds on associated types
//=============================================
fn sum_iterator<I>(iter: I) -> i32
where
    I: Iterator<Item = i32>,
{
    iter.sum()
}

fn print_all<I>(iter: I)
where
    I: Iterator,
    I::Item: Display,    // Bound on associated type
{
    for item in iter {
        println!("{}", item);
    }
}

//==========================================
// Pattern: Lifetime bounds on type params
//==========================================
// T must live at least as long as 'a
fn store_ref<'a, T: 'a>(storage: &mut Option<&'a T>, value: &'a T) {
    *storage = Some(value);
}

// T must be 'static (no borrows or only 'static borrows)
fn spawn_task<T: Send + 'static>(value: T) {
    std::thread::spawn(move || {
        drop(value);
    });
}

//==============================================
// Pattern: Conditional method implementation
//==============================================
struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Wrapper(value)
    }

    fn into_inner(self) -> T {
        self.0
    }
}

// Only when T: Display
impl<T: Display> Wrapper<T> {
    fn display(&self) {
        println!("{}", self.0);
    }
}

// Only when T: Clone
impl<T: Clone> Wrapper<T> {
    fn duplicate(&self) -> Self {
        Wrapper(self.0.clone())
    }
}

// Only when T: Default
impl<T: Default> Default for Wrapper<T> {
    fn default() -> Self {
        Wrapper(T::default())
    }
}

//============================================
// Pattern: Trait bounds for hashable keys
//============================================
fn count_occurrences<T>(items: &[T]) -> HashMap<&T, usize>
where
    T: Hash + Eq,
{
    let mut counts = HashMap::new();
    for item in items {
        *counts.entry(item).or_insert(0) += 1;
    }
    counts
}

//===========================================
// Pattern: Sized and ?Sized bounds
//===========================================
// T must be Sized (default, known size at compile time)
fn takes_sized<T>(value: T) {
    drop(value);
}

// T can be unsized (like str, [u8], dyn Trait)
fn takes_unsized<T: ?Sized>(value: &T) {
    // Can only use through reference
    let _ = std::mem::size_of_val(value);
}

// Works with both String and str
fn print_str<T: AsRef<str> + ?Sized>(s: &T) {
    println!("{}", s.as_ref());
}

//===============================================
// Pattern: Supertraits (trait inheritance)
//===============================================
trait Printable: Display + Debug {
    fn print(&self) {
        println!("Display: {}, Debug: {:?}", self, self);
    }
}

// Implementing Printable requires Display + Debug
impl Printable for i32 {}
impl Printable for String {}

//============================================
// Pattern: Generic bounds with Self
//============================================
trait Duplicable: Sized {
    fn duplicate(&self) -> Self;
}

impl<T: Clone> Duplicable for T {
    fn duplicate(&self) -> Self {
        self.clone()
    }
}

//=================================================
// Pattern: Combining bounds with impl Trait
//=================================================
fn make_iterator() -> impl Iterator<Item = i32> {
    (0..10).filter(|x| x % 2 == 0)
}

fn transform<T>(items: impl IntoIterator<Item = T>) -> Vec<T>
where
    T: Clone,
{
    items.into_iter().collect()
}
```

**Bound selection guidelines:**
- Start with minimal bounds, add as compiler requires
- Prefer `AsRef<T>` over concrete types for flexibility
- Use `?Sized` when working through references only
- Consider `Send + Sync + 'static` for thread-spawning

**Common bound combinations:**
- `Clone + Debug`: Development/testing
- `Hash + Eq`: HashMap keys
- `Ord`: Sortable, BTreeMap keys
- `Serialize + DeserializeOwned`: Data serialization
- `Send + Sync + 'static`: Thread safety

## Pattern 4: Associated Types vs Generic Parameters

**Problem**: When designing a trait, should you use `trait Foo<T>` (generic parameter) or `trait Foo { type T; }` (associated type)? Both allow types to vary by implementation, but they have different semantics and ergonomics.

**Solution**: Use **associated types** for "output" types where each implementation specifies exactly one type (Iterator::Item). Use **generic parameters** for "input" types where the same type can implement the trait multiple times with different parameters (From<T>).

**Why It Matters**: Associated types simplify APIs dramatically. With `Iterator<Item = i32>`, you know the item type. With `Iterator<T>` (if it existed), you'd have multiple impls per type, ambiguity everywhere, and verbose bounds. Choosing correctly makes APIs intuitive and prevents implementation conflicts.

**Use Cases**: Iterator::Item, Deref::Target, collection traits, conversion traits (associated) vs From<T>, Add<Rhs>, comparison traits (generic).

### Examples
```rust
use std::ops::Add;

//====================================================
// Pattern: Associated type - one impl per type
//====================================================
trait Container {
    type Item;                              // Associated type

    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn len(&self) -> usize;
}

impl<T> Container for Vec<T> {
    type Item = T;                          // Specified by impl

    fn get(&self, index: usize) -> Option<&T> {
        <[T]>::get(self, index)
    }

    fn len(&self) -> usize {
        self.len()
    }
}

// Usage is clean - no extra type parameters
fn first_item<C: Container>(c: &C) -> Option<&C::Item> {
    c.get(0)
}

//====================================================
// Pattern: Generic parameter - multiple impls per type
//====================================================
trait Convertible<T> {
    fn convert(&self) -> T;
}

// Same type can implement for multiple targets
impl Convertible<String> for i32 {
    fn convert(&self) -> String {
        self.to_string()
    }
}

impl Convertible<f64> for i32 {
    fn convert(&self) -> f64 {
        *self as f64
    }
}

// Must specify which conversion
let n: i32 = 42;
let s: String = Convertible::<String>::convert(&n);
let f: f64 = Convertible::<f64>::convert(&n);

//======================================================
// Pattern: Combining associated types and generics
//======================================================
trait Graph {
    type Node;
    type Edge;

    fn nodes(&self) -> Vec<&Self::Node>;
    fn edges(&self) -> Vec<&Self::Edge>;
}

// Generic over graph type, but Node/Edge are associated
fn count_nodes<G: Graph>(graph: &G) -> usize {
    graph.nodes().len()
}

//========================================================
// Pattern: Associated type with bounds
//========================================================
trait Summable {
    type Item: Add<Output = Self::Item> + Default + Copy;

    fn items(&self) -> &[Self::Item];

    fn sum(&self) -> Self::Item {
        self.items()
            .iter()
            .copied()
            .fold(Self::Item::default(), |acc, x| acc + x)
    }
}

//===============================================
// Pattern: Generic Associated Types (GAT)
//===============================================
trait LendingIterator {
    type Item<'a> where Self: 'a;           // GAT with lifetime

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// Allows returning references to self
struct WindowsMut<'a, T> {
    slice: &'a mut [T],
    pos: usize,
}

impl<'a, T> LendingIterator for WindowsMut<'a, T> {
    type Item<'b> = &'b mut [T] where Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.pos + 2 <= self.slice.len() {
            let window = &mut self.slice[self.pos..self.pos + 2];
            self.pos += 1;
            Some(window)
        } else {
            None
        }
    }
}

//=======================================================
// Pattern: Type families with associated types
//=======================================================
trait Family {
    type Member;
}

struct IntFamily;
impl Family for IntFamily {
    type Member = i32;
}

struct StringFamily;
impl Family for StringFamily {
    type Member = String;
}

fn create_member<F: Family>() -> F::Member
where
    F::Member: Default,
{
    F::Member::default()
}

//=========================================================
// Pattern: Associated type vs generic - API comparison
//=========================================================
// Associated type approach (cleaner)
trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

// If it were generic (worse - ambiguous)
trait DerefGeneric<Target: ?Sized> {
    fn deref(&self) -> &Target;
}

// Usage difference:
// Associated: impl Deref for MyPtr { type Target = i32; ... }
//             Only ONE target type per pointer type
//
// Generic:    impl DerefGeneric<i32> for MyPtr { ... }
//             impl DerefGeneric<String> for MyPtr { ... }
//             Ambiguous! Which deref to call?
```

**Decision guide:**

| Use Associated Types When | Use Generic Parameters When |
|---------------------------|------------------------------|
| One implementation per type | Multiple implementations per type |
| "Output" or "result" type | "Input" or "parameter" type |
| User shouldn't choose type | User chooses type at call site |
| Example: Iterator::Item | Example: From<T>, Into<T> |
| Example: Deref::Target | Example: Add<Rhs> |

**Key insight**: Associated types answer "what type does this produce?" Generic parameters answer "what type should this accept?"

## Pattern 5: Blanket Implementations

**Problem**: You want to provide trait implementations for all types meeting certain criteria, not just specific types. For example, any type implementing `Display` should automatically get `ToString`. Implementing manually for every type is impossible and unmaintainable.

**Solution**: Use blanket implementations: `impl<T: Bound> Trait for T`. This implements `Trait` for all types `T` that satisfy `Bound`. The standard library uses this extensively—`impl<T: Display> ToString for T` is a blanket impl.

**Why It Matters**: Blanket implementations enable powerful trait composition. You define the relationship once, and it applies to all qualifying types—past, present, and future. This is how Rust achieves "if it compiles, it works" ergonomics while maintaining zero-cost abstractions.

**Use Cases**: `ToString` for `Display`, `Into<U>` from `From<U>`, `Iterator` adapters, automatic trait derivation, extension traits.

### Examples
```rust
use std::fmt::{Debug, Display};

//============================================================
// Pattern: Blanket impl for all types implementing a trait
//============================================================
trait Printable {
    fn print(&self);
}

// Any type that implements Display gets Printable for free
impl<T: Display> Printable for T {
    fn print(&self) {
        println!("{}", self);
    }
}

// Now i32, String, etc. all have .print()
42.print();
"hello".print();

//=====================================================
// Pattern: Extension trait with blanket impl
//=====================================================
trait IteratorExt: Iterator {
    fn count_where<P>(self, predicate: P) -> usize
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool;
}

// Blanket impl for all iterators
impl<I: Iterator> IteratorExt for I {
    fn count_where<P>(self, mut predicate: P) -> usize
    where
        P: FnMut(&Self::Item) -> bool,
    {
        self.filter(|item| predicate(item)).count()
    }
}

// Now any iterator has count_where
let evens = (0..10).count_where(|x| x % 2 == 0);

//========================================
// Pattern: Reflexive implementation
//========================================
trait AsRefExt<T: ?Sized> {
    fn as_ref_ext(&self) -> &T;
}

// Every T can be viewed as &T
impl<T> AsRefExt<T> for T {
    fn as_ref_ext(&self) -> &T {
        self
    }
}

//================================================
// Pattern: Into from From (std library pattern)
//================================================
// This is how std implements Into<U> for all T: From<U>
trait MyInto<T> {
    fn my_into(self) -> T;
}

trait MyFrom<T> {
    fn my_from(value: T) -> Self;
}

// Blanket impl: if you can convert FROM T, you can convert INTO T
impl<T, U> MyInto<U> for T
where
    U: MyFrom<T>,
{
    fn my_into(self) -> U {
        U::my_from(self)
    }
}

//==========================================
// Pattern: Conditional blanket impl
//==========================================
trait Summarizable {
    fn summary(&self) -> String;
}

// Only for types that are both Debug and Clone
impl<T> Summarizable for T
where
    T: Debug + Clone,
{
    fn summary(&self) -> String {
        format!("{:?} (cloneable)", self)
    }
}

//=========================================
// Pattern: Blanket impl with references
//=========================================
trait Process {
    fn process(&self) -> String;
}

impl Process for i32 {
    fn process(&self) -> String {
        format!("processing {}", self)
    }
}

// Blanket impl: if T: Process, then &T: Process
impl<T: Process> Process for &T {
    fn process(&self) -> String {
        (*self).process()
    }
}

// Blanket impl: if T: Process, then Box<T>: Process
impl<T: Process> Process for Box<T> {
    fn process(&self) -> String {
        (**self).process()
    }
}

//=============================================
// Pattern: Negative reasoning (orphan rules)
//=============================================
// You cannot do blanket impl for foreign trait on foreign type
// This prevents conflicting impls across crates

// ALLOWED: Your trait, blanket impl
trait MyTrait {
    fn my_method(&self);
}
impl<T: Debug> MyTrait for T {
    fn my_method(&self) {
        println!("{:?}", self);
    }
}

// ALLOWED: Foreign trait, your type
struct MyType;
impl Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyType")
    }
}

// NOT ALLOWED: Foreign trait, foreign type
// impl Display for Vec<i32> { ... }  // ERROR: orphan rule
```

**Blanket impl patterns:**
- Extension traits: Add methods to foreign types
- Adapter patterns: Auto-convert between traits
- Forwarding: Implement for references/smart pointers
- Conditional behavior: Based on bounds

**Coherence and orphan rules:**
- One impl per type-trait combination globally
- Blanket impls must be in trait's crate or type's crate
- Can't impl foreign trait for foreign type

## Pattern 6: Phantom Types and Type-Level State

**Problem**: You want to track state or constraints at compile time without runtime overhead. For example, a file handle should only allow reading if opened in read mode, or a builder should require certain fields before building. Runtime checks waste cycles and can be forgotten.

**Solution**: Use phantom types—type parameters that exist only in the type signature, not in the data layout. `PhantomData<T>` is a zero-sized type marker that tells the compiler "pretend this struct contains a T." Combined with zero-sized state marker types, this enables the type-state pattern.

**Why It Matters**: Phantom types move invariants from runtime to compile time. Invalid states become unrepresentable—you literally cannot call `.read()` on a write-only handle because the method doesn't exist for that type. This is "making illegal states unrepresentable" in action.

**Use Cases**: Units of measure, authentication/authorization state, protocol state machines, builder pattern validation, FFI ownership markers, capability tokens.

### Examples
```rust
use std::marker::PhantomData;

//==========================================
// Pattern: Type-state for protocol states
//==========================================
struct Disconnected;
struct Connected;
struct Authenticated;

struct Connection<State> {
    socket: String,  // Simplified; would be real socket
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new() -> Self {
        Connection {
            socket: String::new(),
            _state: PhantomData,
        }
    }

    fn connect(self, addr: &str) -> Connection<Connected> {
        Connection {
            socket: addr.to_string(),
            _state: PhantomData,
        }
    }
}

impl Connection<Connected> {
    fn authenticate(self, _credentials: &str) -> Connection<Authenticated> {
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }

    fn disconnect(self) -> Connection<Disconnected> {
        Connection {
            socket: String::new(),
            _state: PhantomData,
        }
    }
}

impl Connection<Authenticated> {
    fn send(&mut self, _data: &[u8]) {
        // Only authenticated connections can send
    }

    fn logout(self) -> Connection<Connected> {
        Connection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

// Usage - compile-time state enforcement
let conn = Connection::<Disconnected>::new();
let conn = conn.connect("localhost:8080");
// conn.send(&[1,2,3]);  // ERROR: no method `send` on Connected
let mut conn = conn.authenticate("secret");
conn.send(&[1, 2, 3]);  // OK: Authenticated has send

//==========================================
// Pattern: Units of measure
//==========================================
struct Meters;
struct Feet;
struct Seconds;

struct Quantity<T, Unit> {
    value: T,
    _unit: PhantomData<Unit>,
}

impl<T, Unit> Quantity<T, Unit> {
    fn new(value: T) -> Self {
        Quantity { value, _unit: PhantomData }
    }

    fn value(&self) -> &T {
        &self.value
    }
}

// Can only add same units
impl<T: std::ops::Add<Output = T>, Unit> std::ops::Add for Quantity<T, Unit> {
    type Output = Quantity<T, Unit>;

    fn add(self, other: Self) -> Self::Output {
        Quantity::new(self.value + other.value)
    }
}

let m1: Quantity<f64, Meters> = Quantity::new(10.0);
let m2: Quantity<f64, Meters> = Quantity::new(5.0);
let m3 = m1 + m2;  // OK: both Meters

let f1: Quantity<f64, Feet> = Quantity::new(3.0);
// let bad = m3 + f1;  // ERROR: can't add Meters and Feet

//=============================================
// Pattern: Builder with required fields
//=============================================
struct NoName;
struct HasName;
struct NoEmail;
struct HasEmail;

struct UserBuilder<Name, Email> {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
    _name_state: PhantomData<Name>,
    _email_state: PhantomData<Email>,
}

impl UserBuilder<NoName, NoEmail> {
    fn new() -> Self {
        UserBuilder {
            name: None,
            email: None,
            age: None,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<Email> UserBuilder<NoName, Email> {
    fn name(self, name: &str) -> UserBuilder<HasName, Email> {
        UserBuilder {
            name: Some(name.to_string()),
            email: self.email,
            age: self.age,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<Name> UserBuilder<Name, NoEmail> {
    fn email(self, email: &str) -> UserBuilder<Name, HasEmail> {
        UserBuilder {
            name: self.name,
            email: Some(email.to_string()),
            age: self.age,
            _name_state: PhantomData,
            _email_state: PhantomData,
        }
    }
}

impl<Name, Email> UserBuilder<Name, Email> {
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
}

struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

// build() only available when both Name and Email are set
impl UserBuilder<HasName, HasEmail> {
    fn build(self) -> User {
        User {
            name: self.name.unwrap(),
            email: self.email.unwrap(),
            age: self.age,
        }
    }
}

// Must provide name and email
let user = UserBuilder::new()
    .name("Alice")
    .email("alice@example.com")
    .age(30)
    .build();

// let incomplete = UserBuilder::new().name("Bob").build();  // ERROR

//==========================================
// Pattern: FFI ownership marker
//==========================================
struct Owned;
struct Borrowed;

struct CString<Ownership> {
    ptr: *mut i8,
    _ownership: PhantomData<Ownership>,
}

impl CString<Owned> {
    fn new(s: &str) -> Self {
        // Allocate and copy string
        let ptr = Box::into_raw(s.to_string().into_boxed_str()) as *mut i8;
        CString { ptr, _ownership: PhantomData }
    }
}

impl Drop for CString<Owned> {
    fn drop(&mut self) {
        // Only owned strings need to be freed
        unsafe {
            drop(Box::from_raw(self.ptr));
        }
    }
}

impl CString<Borrowed> {
    unsafe fn from_ptr(ptr: *mut i8) -> Self {
        CString { ptr, _ownership: PhantomData }
    }
    // No Drop impl - we don't own it
}
```

**Phantom type benefits:**
- Zero runtime cost (ZST optimized away)
- Compile-time state verification
- Self-documenting API constraints
- Impossible to misuse

**Common phantom patterns:**
- State machines: Protocol, connection, auth states
- Units: Meters, seconds, currencies (can't mix)
- Permissions: ReadOnly, WriteOnly, ReadWrite
- Ownership: Owned, Borrowed, Shared

## Pattern 7: Higher-Ranked Trait Bounds (HRTBs)

**Problem**: You want to accept a closure that works with references of any lifetime, not a specific one. For example, a function that calls a closure with temporary references created inside the function. Normal lifetime parameters can't express "works for any lifetime."

**Solution**: Use higher-ranked trait bounds with `for<'a>` syntax: `F: for<'a> Fn(&'a str) -> &'a str`. This means "F implements Fn for all possible lifetimes 'a." The closure must work regardless of what lifetime the references have.

**Why It Matters**: HRTBs are essential for closure-heavy APIs. Without them, you couldn't write functions like `Vec::sort_by` that accept comparison closures operating on temporary references. HRTBs enable higher-order functions that are fundamental to Rust's functional programming style.

**Use Cases**: Iterator adapters (map, filter with borrowing), callback registries, parser combinators, sorting with custom comparators, visitor patterns.

### Examples
```rust
//=================================================
// Pattern: Basic HRTB for closure with references
//=================================================
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> usize,
{
    let local = String::from("hello");
    let result = f(&local);  // 'a is the lifetime of local
    println!("Length: {}", result);
}

call_with_ref(|s| s.len());  // Closure works for any lifetime

//=======================================================
// Pattern: HRTB vs regular lifetime parameter
//=======================================================
// Regular lifetime: caller chooses lifetime
fn with_lifetime<'a, F>(s: &'a str, f: F) -> &'a str
where
    F: Fn(&'a str) -> &'a str,
{
    f(s)
}

// HRTB: function chooses lifetime, closure must handle any
fn with_hrtb<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let local = String::from("hello world");
    f(&local).to_string()  // f must work with local's lifetime
}

//===========================================
// Pattern: Fn trait bounds are sugar for HRTB
//===========================================
// These are equivalent:
fn takes_fn_sugar<F: Fn(&str)>(f: F) { f("hi"); }
fn takes_fn_explicit<F>(f: F) where F: for<'a> Fn(&'a str) { f("hi"); }

// The sugar version automatically uses HRTB for reference parameters

//===========================================
// Pattern: HRTB with multiple lifetimes
//===========================================
fn call_with_two<F>(f: F)
where
    F: for<'a, 'b> Fn(&'a str, &'b str) -> bool,
{
    let s1 = String::from("hello");
    let s2 = String::from("world");
    let result = f(&s1, &s2);
    println!("Equal: {}", result);
}

//==========================================
// Pattern: HRTB in trait definitions
//==========================================
trait Parser<Output> {
    fn parse<'a>(&self, input: &'a str) -> Option<(Output, &'a str)>;
}

// Store parser that works for any input lifetime
struct BoxedParser<Output> {
    parser: Box<dyn for<'a> Fn(&'a str) -> Option<(Output, &'a str)>>,
}

impl<Output> BoxedParser<Output> {
    fn new<F>(f: F) -> Self
    where
        F: for<'a> Fn(&'a str) -> Option<(Output, &'a str)> + 'static,
    {
        BoxedParser { parser: Box::new(f) }
    }

    fn parse<'a>(&self, input: &'a str) -> Option<(Output, &'a str)> {
        (self.parser)(input)
    }
}

//================================================
// Pattern: HRTB for iterator adapters
//================================================
trait IteratorExt: Iterator {
    fn for_each_ref<F>(self, f: F)
    where
        Self: Sized,
        F: for<'a> FnMut(&'a Self::Item);
}

impl<I: Iterator> IteratorExt for I {
    fn for_each_ref<F>(self, mut f: F)
    where
        F: for<'a> FnMut(&'a Self::Item),
    {
        for item in self {
            f(&item);
        }
    }
}

//============================================
// Pattern: Callback storage with HRTB
//============================================
type Callback = Box<dyn for<'a> Fn(&'a str)>;

struct EventEmitter {
    callbacks: Vec<Callback>,
}

impl EventEmitter {
    fn new() -> Self {
        EventEmitter { callbacks: Vec::new() }
    }

    fn on<F>(&mut self, callback: F)
    where
        F: for<'a> Fn(&'a str) + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    fn emit(&self, event: &str) {
        for callback in &self.callbacks {
            callback(event);
        }
    }
}

//===================================================
// Pattern: Where HRTB is NOT needed (common mistake)
//===================================================
// DON'T need HRTB: lifetime comes from parameter
fn map_ref<'a, T, U, F>(value: &'a T, f: F) -> U
where
    F: Fn(&'a T) -> U,  // No HRTB needed, 'a from parameter
{
    f(value)
}

// DO need HRTB: lifetime created inside function
fn create_and_process<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,  // HRTB needed
{
    let local = String::from("temp");
    let _ = f(&local);  // local's lifetime unknown to caller
}
```

**When to use HRTB:**
- Closure works with references created inside your function
- Storing callbacks that will be called with various lifetimes
- Parser combinators and similar patterns
- When compiler suggests it

**When NOT to use HRTB:**
- Lifetime comes from function parameter
- Closure returns owned value
- No references involved

## Pattern 8: Const Generics

**Problem**: You need to parameterize types by compile-time constant values, not just types. Arrays in Rust have their size in the type: `[i32; 5]` is different from `[i32; 10]`. Without const generics, you'd need separate impls for each size or use runtime-sized `Vec`.

**Solution**: Use const generics: `struct Array<T, const N: usize>`. The `N` is a compile-time constant that becomes part of the type. Rust currently supports const generics for integers, bools, and chars. This enables truly generic fixed-size types.

**Why It Matters**: Const generics enable zero-cost fixed-size abstractions. Matrix multiplication `Matrix<3, 4>` * `Matrix<4, 5>` = `Matrix<3, 5>` is checked at compile time. Buffer sizes, protocol frame lengths, and SIMD vector widths can all be type-level constants, catching dimension mismatches before runtime.

**Use Cases**: Fixed-size arrays, matrices with compile-time dimension checking, network protocol frames, ring buffers, SIMD vectors, lookup tables.

### Examples
```rust
//=====================================
// Pattern: Basic const generic array
//=====================================
struct Array<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> Array<T, N> {
    fn new() -> Self {
        Array { data: [T::default(); N] }
    }
}

impl<T, const N: usize> Array<T, N> {
    fn len(&self) -> usize {
        N  // Known at compile time
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index < N {
            Some(&self.data[index])
        } else {
            None
        }
    }
}

let arr: Array<i32, 5> = Array::new();
assert_eq!(arr.len(), 5);

//===========================================
// Pattern: Compile-time size validation
//===========================================
struct NonEmpty<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> NonEmpty<T, N> {
    fn new(data: [T; N]) -> Self {
        // Compile-time check that N > 0
        const { assert!(N > 0, "NonEmpty requires N > 0") }
        NonEmpty { data }
    }

    fn first(&self) -> &T {
        &self.data[0]  // Always safe, N > 0
    }
}

let valid: NonEmpty<i32, 3> = NonEmpty::new([1, 2, 3]);
// let invalid: NonEmpty<i32, 0> = NonEmpty::new([]);  // Compile error

//==========================================
// Pattern: Matrix with dimension checking
//==========================================
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

impl<T: Default + Copy, const R: usize, const C: usize> Matrix<T, R, C> {
    fn new() -> Self {
        Matrix { data: [[T::default(); C]; R] }
    }

    fn rows(&self) -> usize { R }
    fn cols(&self) -> usize { C }
}

impl<T, const R: usize, const C: usize> Matrix<T, R, C> {
    fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.data.get(row)?.get(col)
    }

    fn transpose(self) -> Matrix<T, C, R>
    where
        T: Default + Copy,
    {
        let mut result = Matrix::<T, C, R>::new();
        for i in 0..R {
            for j in 0..C {
                result.data[j][i] = self.data[i][j];
            }
        }
        result
    }
}

// Matrix multiplication with compile-time dimension checking
impl<T, const M: usize, const N: usize> Matrix<T, M, N>
where
    T: Default + Copy + std::ops::Add<Output = T> + std::ops::Mul<Output = T>,
{
    fn multiply<const P: usize>(
        &self,
        other: &Matrix<T, N, P>,
    ) -> Matrix<T, M, P> {
        let mut result = Matrix::<T, M, P>::new();
        for i in 0..M {
            for j in 0..P {
                let mut sum = T::default();
                for k in 0..N {
                    sum = sum + self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        result
    }
}

let a: Matrix<i32, 2, 3> = Matrix::new();
let b: Matrix<i32, 3, 4> = Matrix::new();
let c: Matrix<i32, 2, 4> = a.multiply(&b);  // Compiles!

// let bad = a.multiply(&a);  // ERROR: 2x3 * 2x3 dimension mismatch

//=============================================
// Pattern: Fixed-size buffer / ring buffer
//=============================================
struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    fn new() -> Self
    where
        T: Copy,
    {
        RingBuffer {
            buffer: [None; N],
            head: 0,
            tail: 0,
            len: 0,
        }
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len == N {
            Err(value)  // Buffer full
        } else {
            self.buffer[self.tail] = Some(value);
            self.tail = (self.tail + 1) % N;
            self.len += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            let value = self.buffer[self.head].take();
            self.head = (self.head + 1) % N;
            self.len -= 1;
            value
        }
    }

    fn capacity(&self) -> usize { N }
    fn len(&self) -> usize { self.len }
}

//=============================================
// Pattern: Const generics with expressions
//=============================================
fn double_array<T: Copy + Default, const N: usize>(
    arr: [T; N],
) -> [T; N * 2] {
    let mut result = [T::default(); N * 2];
    result[..N].copy_from_slice(&arr);
    result[N..].copy_from_slice(&arr);
    result
}

//===============================================
// Pattern: Const generic for protocol frames
//===============================================
struct Frame<const SIZE: usize> {
    header: [u8; 4],
    payload: [u8; SIZE],
    checksum: u32,
}

impl<const SIZE: usize> Frame<SIZE> {
    fn new(payload: [u8; SIZE]) -> Self {
        Frame {
            header: [0; 4],
            payload,
            checksum: 0,
        }
    }

    fn total_size(&self) -> usize {
        4 + SIZE + 4  // header + payload + checksum
    }
}

// Different frame types
type SmallFrame = Frame<64>;
type LargeFrame = Frame<1024>;
type JumboFrame = Frame<9000>;

//==========================================
// Pattern: Const bounds and where clauses
//==========================================
fn requires_at_least_two<T, const N: usize>(arr: [T; N]) -> (T, T)
where
    [(); N - 2]:,  // Assert N >= 2
{
    let mut iter = arr.into_iter();
    (iter.next().unwrap(), iter.next().unwrap())
}

fn split_array<T, const N: usize, const M: usize>(
    arr: [T; N],
) -> ([T; M], [T; N - M])
where
    [(); N - M]:,  // Assert N >= M
{
    todo!()  // Implementation omitted for brevity
}
```

**Const generic benefits:**
- Compile-time size checking
- Zero runtime overhead
- Type-safe fixed-size containers
- Dimension-aware math operations

**Current limitations:**
- Only primitive types (integers, bool, char)
- Complex const expressions still unstable
- No const generic associated types yet
- Const bounds syntax is awkward

## Performance Summary

| Feature | Compile Overhead | Runtime Overhead | Binary Size |
|---------|------------------|------------------|-------------|
| Monomorphization | High | None | Increases |
| Trait bounds | Low | None | None |
| Associated types | Low | None | None |
| PhantomData | None | None | None |
| Const generics | Medium | None | Varies |
| HRTBs | Low | None | None |

## Quick Reference: Choosing Generic Patterns

```rust
// Need same operation on different types?
fn operation<T: Trait>(x: T) { }              // Generic function

// Need reusable data structure?
struct Container<T> { data: T }               // Generic struct

// Need to constrain type capabilities?
fn foo<T: Clone + Debug>(x: T) { }            // Trait bounds

// One implementation per type?
trait Iter { type Item; }                     // Associated type

// Multiple implementations per type?
trait From<T> { fn from(t: T) -> Self; }      // Generic parameter

// Implement for all types with property?
impl<T: Display> ToString for T { }           // Blanket impl

// Compile-time state tracking?
struct State<S> { _marker: PhantomData<S> }   // Phantom type

// Closure works with any lifetime?
F: for<'a> Fn(&'a str)                        // HRTB

// Parameterize by constant value?
struct Array<T, const N: usize>               // Const generic
```

## Common Anti-Patterns

```rust
// ❌ Over-constraining: bounds not needed by implementation
fn foo<T: Clone + Debug + Display + Default>(x: T) {
    println!("{:?}", x);  // Only uses Debug!
}

// ✓ Minimal bounds
fn foo<T: Debug>(x: T) {
    println!("{:?}", x);
}

// ❌ Generic when concrete works
fn always_i32<T>(x: T) -> i32 where T: Into<i32> {
    x.into()
}

// ✓ Just accept i32
fn always_i32(x: impl Into<i32>) -> i32 {
    x.into()
}

// ❌ Associated type when generic needed
trait Converter {
    type Output;  // Can only convert to ONE type!
    fn convert(&self) -> Self::Output;
}

// ✓ Generic parameter for multiple conversions
trait Converter<T> {
    fn convert(&self) -> T;
}

// ❌ Phantom type without purpose
struct Wrapper<T, Phantom> {
    value: T,
    _p: PhantomData<Phantom>,  // Does nothing!
}

// ✓ Phantom type with state meaning
struct Connection<State> {
    socket: Socket,
    _state: PhantomData<State>,  // Enforces protocol
}

// ❌ Missing HRTB - won't compile
fn broken<F>(f: F) where F: Fn(&str) -> &str {
    let local = String::from("temp");
    let _ = f(&local);  // ERROR: lifetime mismatch
}

// ✓ HRTB when creating references inside
fn works<F>(f: F) where F: for<'a> Fn(&'a str) -> &'a str {
    let local = String::from("temp");
    let _ = f(&local);  // OK
}
```

### Key Takeaways

1. **Monomorphization is zero-cost**: Generic code compiles to specialized machine code
2. **Trait bounds are contracts**: Specify exactly what capabilities you need
3. **Associated types simplify APIs**: Use for "output" types in traits
4. **Blanket impls enable composition**: Implement once, apply everywhere
5. **Phantom types are free**: Zero runtime cost for compile-time guarantees
6. **HRTBs unlock closures**: Essential for callback-heavy APIs
7. **Const generics enable value-level types**: Compile-time dimension checking
8. **Start minimal**: Add bounds only when the compiler requires them

Generics are the backbone of Rust's zero-cost abstractions. Mastering them means understanding not just syntax, but the trade-offs between compile-time safety, API ergonomics, and implementation flexibility.

### Generics Cheat Sheet
```rust
// ===== BASIC GENERIC SYNTAX =====
// Generic function
fn identity<T>(x: T) -> T {
    x
}

let num = identity(5);                              // T inferred as i32
let text = identity("hello");                       // T inferred as &str
let explicit = identity::<f64>(3.14);              // Explicit type (turbofish)

// Multiple type parameters
fn pair<T, U>(first: T, second: U) -> (T, U) {
    (first, second)
}

// Generic struct
struct Point<T> {
    x: T,
    y: T,
}

let int_point = Point { x: 5, y: 10 };
let float_point = Point { x: 1.0, y: 4.0 };

// Generic enum
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}

// ===== TRAIT BOUNDS =====
// Single bound
fn print_debug<T: std::fmt::Debug>(value: T) {
    println!("{:?}", value);
}

// Multiple bounds with +
fn compare_print<T: PartialOrd + std::fmt::Display>(a: T, b: T) {
    if a < b {
        println!("{} < {}", a, b);
    }
}

// Where clause (preferred for complex bounds)
fn complex<T, U>(t: T, u: U) -> i32
where
    T: Clone + std::fmt::Debug,
    U: Copy + Default,
{
    42
}

// Bound on return type
fn largest<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0];
    for &item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

// ===== GENERIC IMPLEMENTATIONS =====
// Basic impl
impl<T> Point<T> {
    fn new(x: T, y: T) -> Self {
        Point { x, y }
    }

    fn x(&self) -> &T {
        &self.x
    }
}

// Impl with bounds (methods only for types with these traits)
impl<T: Copy + std::ops::Add<Output = T>> Point<T> {
    fn add(&self, other: &Point<T>) -> Point<T> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Impl for specific type
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

// ===== GENERIC TRAITS =====
// Generic trait
trait Container<T> {
    fn get(&self) -> &T;
    fn set(&mut self, value: T);
}

// Implementing generic trait
struct Box<T> {
    value: T,
}

impl<T> Container<T> for Box<T> {
    fn get(&self) -> &T {
        &self.value
    }

    fn set(&mut self, value: T) {
        self.value = value;
    }
}

// ===== ASSOCIATED TYPES =====
// Instead of generic parameter on trait
trait Iterator {
    type Item;                                      // Associated type
    fn next(&mut self) -> Option<Self::Item>;
}

// Implementing with associated type
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;                                // Specify associated type

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        Some(self.count)
    }
}

// Using associated type in bounds
fn sum_all<I: Iterator<Item = i32>>(iter: I) -> i32 {
    let mut sum = 0;
    // ...
    sum
}

// ===== DEFAULT TYPE PARAMETERS =====
// Default generic type
struct Wrapper<T = i32> {
    value: T,
}

let default_wrapper = Wrapper { value: 5 };        // T defaults to i32
let string_wrapper = Wrapper { value: "hello" };   // T is &str

// Operator overloading with default
use std::ops::Add;

impl<T: Add<Output = T>> Add for Point<T> {
    type Output = Point<T>;

    fn add(self, other: Point<T>) -> Point<T> {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// ===== CONST GENERICS =====
// Array with const generic size
struct Array<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> Array<T, N> {
    fn new() -> Self {
        Array { data: [T::default(); N] }
    }

    fn len(&self) -> usize {
        N
    }
}

let arr: Array<i32, 5> = Array::new();
let arr10: Array<i32, 10> = Array::new();

// Function with const generic
fn create_array<T: Default + Copy, const N: usize>() -> [T; N] {
    [T::default(); N]
}

let zeros: [i32; 3] = create_array();

// ===== PHANTOM DATA =====
use std::marker::PhantomData;

// PhantomData for unused type parameter
struct Tagged<T, Tag> {
    value: T,
    _marker: PhantomData<Tag>,                      // Tag used at type level only
}

struct Meters;
struct Feet;

type Distance<T> = Tagged<f64, T>;

let in_meters: Distance<Meters> = Tagged { value: 100.0, _marker: PhantomData };
let in_feet: Distance<Feet> = Tagged { value: 328.0, _marker: PhantomData };

// Type-state pattern
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        Door { _state: PhantomData }
    }
}

// ===== HIGHER-RANKED TRAIT BOUNDS (HRTB) =====
// for<'a> - "for any lifetime 'a"
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let s = String::from("hello");
    let result = f(&s);
    println!("{}", result);
}

// Common in closure bounds
fn apply<F>(f: F, s: &str) -> String
where
    F: Fn(&str) -> String,                          // Desugars to for<'a> Fn(&'a str)
{
    f(s)
}

// ===== BLANKET IMPLEMENTATIONS =====
// Impl for all types matching bounds
trait Printable {
    fn print(&self);
}

impl<T: std::fmt::Display> Printable for T {
    fn print(&self) {
        println!("{}", self);
    }
}

// Now any Display type has print()
5.print();
"hello".print();

// Standard library example: Into from From
// impl<T, U> Into<U> for T where U: From<T>

// ===== GENERIC LIFETIMES =====
// Lifetime parameter
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Generic type with lifetime
struct ImportantExcerpt<'a> {
    part: &'a str,
}

// Multiple lifetimes
fn complex_ref<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x
}

// Lifetime bounds
fn ref_to_static<T: 'static>(t: T) {
    // T doesn't contain non-static references
}

// ===== IMPL TRAIT =====
// Return type impl Trait (existential type)
fn make_iter() -> impl Iterator<Item = i32> {
    vec![1, 2, 3].into_iter()
}

// Argument position impl Trait
fn print_iter(iter: impl Iterator<Item = i32>) {
    for i in iter {
        println!("{}", i);
    }
}

// Equivalent to generic but simpler
fn print_iter_generic<I: Iterator<Item = i32>>(iter: I) {
    for i in iter {
        println!("{}", i);
    }
}

// ===== TURBOFISH SYNTAX =====
// Explicit type specification
let parsed = "5".parse::<i32>().unwrap();
let collected: Vec<i32> = (0..10).collect();
let collected2 = (0..10).collect::<Vec<i32>>();

// Method with type parameter
struct Container<T>(T);

impl<T> Container<T> {
    fn convert<U: From<T>>(self) -> Container<U> {
        Container(U::from(self.0))
    }
}

let c = Container(5i32);
let c2 = c.convert::<i64>();                        // Turbofish for method type param

// ===== COMMON TRAIT BOUNDS =====
// Clone - explicit copy
fn duplicate<T: Clone>(value: &T) -> T {
    value.clone()
}

// Copy - implicit copy (bitwise)
fn copy_it<T: Copy>(value: T) -> (T, T) {
    (value, value)
}

// Default - has default value
fn with_default<T: Default>() -> T {
    T::default()
}

// Debug - {:?} formatting
fn debug_print<T: std::fmt::Debug>(value: &T) {
    println!("{:?}", value);
}

// Display - {} formatting
fn display_print<T: std::fmt::Display>(value: &T) {
    println!("{}", value);
}

// PartialEq - equality comparison
fn equals<T: PartialEq>(a: &T, b: &T) -> bool {
    a == b
}

// Ord - total ordering
fn sort_vec<T: Ord>(vec: &mut Vec<T>) {
    vec.sort();
}

// Hash - can be hashed
use std::collections::HashMap;
fn as_key<K: std::hash::Hash + Eq, V>(map: &HashMap<K, V>, key: &K) -> Option<&V> {
    map.get(key)
}

// Send - safe to send between threads
fn spawn_with<T: Send + 'static>(value: T) {
    std::thread::spawn(move || {
        let _ = value;
    });
}

// Sync - safe to share references between threads
fn share<T: Sync>(value: &T) {
    // Can be safely referenced from multiple threads
}

// Sized - has known size at compile time (default bound)
fn sized_only<T: Sized>(value: T) {}

// ?Sized - may be unsized (like str, [T], dyn Trait)
fn maybe_unsized<T: ?Sized>(value: &T) {}

// ===== GENERIC PATTERNS =====
// Builder pattern with generics
struct RequestBuilder<State> {
    url: String,
    method: String,
    _state: PhantomData<State>,
}

struct NoUrl;
struct HasUrl;

impl RequestBuilder<NoUrl> {
    fn new() -> Self {
        RequestBuilder {
            url: String::new(),
            method: String::from("GET"),
            _state: PhantomData,
        }
    }

    fn url(self, url: &str) -> RequestBuilder<HasUrl> {
        RequestBuilder {
            url: url.to_string(),
            method: self.method,
            _state: PhantomData,
        }
    }
}

impl RequestBuilder<HasUrl> {
    fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    fn build(self) -> Request {
        Request {
            url: self.url,
            method: self.method,
        }
    }
}

struct Request {
    url: String,
    method: String,
}

// Newtype pattern
struct Meters(f64);
struct Kilometers(f64);

impl From<Kilometers> for Meters {
    fn from(km: Kilometers) -> Self {
        Meters(km.0 * 1000.0)
    }
}

// ===== TYPE INFERENCE TIPS =====
// Let compiler infer when possible
let v = vec![1, 2, 3];                             // Vec<i32> inferred
let s: String = "hello".into();                    // Into<String> inferred

// Sometimes annotation needed
let parsed: i32 = "5".parse().unwrap();           // parse() needs return type
let collected: Vec<_> = (0..10).collect();        // collect() needs container type

// Use _ for partial inference
let map: HashMap<_, _> = vec![(1, "a"), (2, "b")].into_iter().collect();

// ===== COMMON GOTCHAS =====
// Can't use T without bounds
fn broken<T>(x: T) {
    // println!("{}", x);                          // ERROR: T may not impl Display
    // x.clone();                                  // ERROR: T may not impl Clone
}

// Fixed with bounds
fn fixed<T: std::fmt::Display + Clone>(x: T) {
    println!("{}", x);
    let _ = x.clone();
}

// Trait objects vs generics
fn generic<T: std::fmt::Debug>(x: T) {
    // Monomorphized - separate code for each T
    println!("{:?}", x);
}

fn trait_object(x: &dyn std::fmt::Debug) {
    // Dynamic dispatch - single code, vtable lookup
    println!("{:?}", x);
}

// Associated type vs generic
trait WithAssociated {
    type Output;                                    // One output type per impl
    fn produce(&self) -> Self::Output;
}

trait WithGeneric<T> {
    fn produce(&self) -> T;                        // Multiple possible for same impl
}
```
