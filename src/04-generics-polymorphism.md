# Generics & Polymorphism
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

### Type System Foundation

```rust
// Basic generic syntax
fn foo<T>(x: T) -> T { x }                // Generic function
struct Pair<T, U> { first: T, second: U } // Generic struct
enum Result<T, E> { Ok(T), Err(E) }       // Generic enum
impl<T> Pair<T, T> { }                    // Generic impl

// Trait bounds (constraints)
fn print<T: Display>(x: T) { }            // Single bound
fn both<T: Debug + Clone>(x: T) { }       // Multiple bounds
fn complex<T>(x: T) where T: Debug + Clone { }
fn lifetime<'a, T: 'a>(x: &'a T) { }      // Lifetime bounds

// Associated types
trait Iterator { type Item; }             // By impl
trait Graph { type Node; type Edge; }     // Multiple

// Const generics
struct Array<T, const N: usize>([T; N]);  // By value
fn fixed<const N: usize>() -> [u8; N] { [0; N] }

// Higher-ranked trait bounds
fn hrtb<F>(f: F) where F: for<'a> Fn(&'a str) { }

// Phantom types
use std::marker::PhantomData;
struct Tagged<T, Tag> {
    value: T,
    _tag: PhantomData<Tag>,
}
```

## Pattern 1: Type-Safe Generic Functions

*   **Problem**: You need to write functions that perform the same operation on different types. Without generics, you'd either duplicate code for each type (error-prone, unmaintainable) or use dynamic typing with runtime casts (unsafe, slow).
*   **Solution**: Define functions with type parameters `<T>` that can be instantiated with any concrete type. Add trait bounds to constrain `T` to types that support required operations. Monomorphization means generic code is as fast as hand-written specialized code—there's no vtable lookup, no boxing, no dynamic dispatch. The `Vec::push` you call on `Vec<i32>` is different compiled code than `Vec::push` on `Vec<String>`, each optimized for its type.

### Example: Basic Generic Function with Type Inference

The compiler infers type parameter `T` from usage, so `identity(42)` becomes `identity::<i32>`. Each call site can use a different concrete type. The function body works identically regardless of the actual type.

```rust
fn identity<T>(x: T) -> T {
    x
}

// Usage: Compiler infers T from argument; works with any type.
let x = identity(42); // Returns 42, type i32
let s = identity("hello"); // Returns "hello", type &str
```

### Example: Generic Function with Trait Bound

Add `T: PartialOrd` to require comparison capability. Without the bound, `item > largest` wouldn't compile since not all types support `>`. The bound is checked at compile time for each concrete type used.

```rust
fn largest<T: PartialOrd>(list: &[T]) -> Option<&T> {
    let mut largest = list.first()?;
    for item in list {
        if item > largest { largest = item; }
    }
    Some(largest)
}

// Usage: Works on any slice of comparable items.
let max_int = largest(&[34, 50, 25, 100, 65]);
let max_str = largest(&["apple", "zebra", "mango"]);
```

### Example: Multiple Trait Bounds

Use `+` to combine bounds: `T: Ord + Display` requires both sorting and printing. The `where` clause is cleaner for complex bounds. Each bound unlocks specific operations on the type parameter.

```rust
use std::fmt::Display;

fn print_sorted<T: Ord + Display>(mut items: Vec<T>) {
    items.sort();
    for item in items { println!("{}", item); }
}

// Usage: Bounds enable sorting (Ord) and printing (Display).
print_sorted(vec![3, 1, 4, 1, 5]);
print_sorted(vec!["banana", "apple", "cherry"]);
```

### Example: Returning Owned vs Borrowed

Generic functions can return owned values (`T`), borrowed references (`&T`), or references with explicit lifetimes. `create_default` returns ownership; `first` borrows from input. Lifetimes connect input and output reference validity.

```rust
fn create_default<T: Default>() -> T { T::default() }
fn first<T>(slice: &[T]) -> Option<&T> { slice.first() }
fn longest<'a, T>(x: &'a [T], y: &'a [T]) -> &'a [T] {
    if x.len() > y.len() { x } else { y }
}

// Usage: Generic return types inferred from variable annotation.
let s: String = create_default(); // Empty string via Default
let f = first(&[1, 2, 3]); // Some(&1)
let longer = longest(&[1, 2], &[3, 4, 5]); // &[3, 4, 5]
```

### Example: Turbofish for Explicit Type

When inference can't determine the type, use turbofish `::<Type>` syntax. Common with `parse()`, `collect()`, and functions returning `Default`. You can also annotate the variable instead: `let x: i32 = ...`.

```rust
fn create_default<T: Default>() -> T { T::default() }

// Usage: Turbofish specifies type when inference isn't enough.
let parsed = "42".parse::<i32>().unwrap();
let default = create_default::<String>();
let collected: Vec<i32> = (0..5).collect();
```

### Example: Generic Comparison with Borrowing

Comparison functions typically borrow both arguments to avoid ownership transfer. Returning a reference requires lifetime annotations connecting output to inputs. The `Ord` bound provides `cmp()` for total ordering.

```rust
use std::cmp::Ordering;

fn compare<T: Ord>(a: &T, b: &T) -> Ordering { a.cmp(b) }
fn min_ref<'a, T: Ord>(a: &'a T, b: &'a T) -> &'a T {
    if a <= b { a } else { b }
}

// Usage: Borrow arguments for comparison without taking ownership.
let ord = compare(&5, &10); // Ordering::Less
let min = min_ref(&"apple", &"banana"); // &"apple"
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

*   **Problem**: Data structures need to store values of various types. Hard-coding types limits reusability—you'd need `IntStack`, `StringStack`, `FloatStack`, etc.
*   **Solution**: Parameterize structs and enums over type parameters. `struct Stack<T>` can hold any type while remaining fully type-safe.
*   **Why It Matters**: Generic data structures are the foundation of Rust's standard library. `Vec<T>`, `HashMap<K, V>`, `Option<T>`, `Result<T, E>`—these are all generic types.

### Example: Basic Generic Struct

The `impl<T>` block introduces type parameter for methods on the struct. Methods can add their own trait bounds beyond the struct definition. The same struct type works with integers, floats, strings, or custom types.

```rust
#[derive(Debug, PartialEq)]
struct Point<T> { x: T, y: T }

impl<T> Point<T> {
    fn new(x: T, y: T) -> Self { Point { x, y } }
}

impl<T: Copy + std::ops::Add<Output = T>> Point<T> {
    fn add(&self, other: &Point<T>) -> Point<T> {
        Point { x: self.x + other.x, y: self.y + other.y }
    }
}

// Usage: Same struct works with i32, f64, or any addable type.
let p1 = Point::new(1, 2);
let p2 = Point::new(3, 4);
let sum = p1.add(&p2); // Point { x: 4, y: 6 }
```

### Example: Multiple Type Parameters

Use multiple type parameters `<T, U>` when fields have different types. Methods can swap, mix, or transform type parameters. Method generics `<V, W>` can introduce parameters beyond the struct's own.

```rust
struct Pair<T, U> { first: T, second: U }

impl<T, U> Pair<T, U> {
    fn new(first: T, second: U) -> Self { Pair { first, second } }
    fn swap(self) -> Pair<U, T> {
        Pair { first: self.second, second: self.first }
    }
    fn mix<V, W>(self, other: Pair<V, W>) -> Pair<T, W> {
        Pair { first: self.first, second: other.second }
    }
}

// Usage: Different type parameters allow heterogeneous pairs.
let p = Pair::new(42, "hello"); // Pair<i32, &str>
let swapped = p.swap(); // Pair<&str, i32>
```

### Example: Generic Enum with Variants

Generic enums like `Option<T>` and `Result<T, E>` carry different data per variant. Recursive structures need `Box` to break the infinite size. Trait bounds on `impl` enable operations like comparison.

```rust
enum BinaryTree<T> {
    Empty,
    Node {
        value: T,
        left: Box<BinaryTree<T>>,
        right: Box<BinaryTree<T>>,
    },
}

impl<T: Ord> BinaryTree<T> {
    fn new() -> Self { BinaryTree::Empty }

    fn insert(&mut self, item: T) {
        match self {
            BinaryTree::Empty => {
                *self = BinaryTree::Node {
                    value: item,
                    left: Box::new(BinaryTree::Empty),
                    right: Box::new(BinaryTree::Empty),
                };
            }
            BinaryTree::Node { value, left, right } => {
                if item < *value { left.insert(item); }
                else { right.insert(item); }
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

// Usage: Generic tree works with any Ord type.
let mut tree = BinaryTree::new();
tree.insert(5);
tree.insert(3);
let found = tree.contains(&5); // true
```

### Example: Generic Wrapper with Transformation

Wrapper types add behavior to any inner type. The `map` method transforms inner type via closure. `into_inner` extracts with ownership; `as_ref` borrows without consuming.

```rust
struct Wrapper<T> { value: T }

impl<T> Wrapper<T> {
    fn new(value: T) -> Self { Wrapper { value } }
    fn into_inner(self) -> T { self.value }
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Wrapper<U> {
        Wrapper { value: f(self.value) }
    }
    fn as_ref(&self) -> Wrapper<&T> { Wrapper { value: &self.value } }
}

// Usage: map transforms inner value; type changes if closure returns different type.
let w = Wrapper::new(5);
let doubled = w.map(|x| x * 2); // Wrapper { value: 10 }
```

### Example: Specialized Impl for Specific Types

Add `impl Point<f64>` for methods only available on that concrete type. Generic impls with bounds like `T: Mul` work for all matching types. This enables both generic and specialized behavior on the same struct.

```rust
#[derive(Debug, PartialEq)]
struct Point<T> { x: T, y: T }

impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

impl<T: std::ops::Mul<Output = T> + Copy> Point<T> {
    fn scale(&self, factor: T) -> Point<T> {
        Point { x: self.x * factor, y: self.y * factor }
    }
}

// Usage: f64-specific method only available on Point<f64>.
let p = Point { x: 3.0_f64, y: 4.0_f64 };
let dist = p.distance_from_origin(); // 5.0 (Pythagorean theorem)
```

### Example: Generic Struct with Lifetime

Structs holding references need lifetime parameters `<'a>`. The lifetime connects to the referenced data's validity. Combine with type parameters as `<'a, T>` for generic reference wrappers.

```rust
struct Ref<'a, T> { value: &'a T }

impl<'a, T> Ref<'a, T> {
    fn new(value: &'a T) -> Self { Ref { value } }
    fn get(&self) -> &T { self.value }
}

// Usage: Lifetime ensures Ref doesn't outlive referenced data.
let num = 42;
let r = Ref::new(&num);
let value = r.get(); // &42
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

*   **Problem**: Generic functions need to perform operations on their type parameters, but `T` could be any type—how do you call `.clone()` on `T` if `T` might not implement `Clone`? Without constraints, you can only move, drop, or pass the value around.
*   **Solution**: Use trait bounds to constrain type parameters to types implementing specific traits. Bounds can be inline (`fn foo<T: Clone>`) or in where clauses (`where T: Clone + Debug`).
*   **Why It Matters**: Trait bounds are the contract between generic code and its callers. They specify exactly what capabilities are required, enabling the compiler to verify correctness.

### Example: Single Trait Bound

Add `T: Debug` to require debug formatting capability. The bound is part of the function signature and checked at compile time. Without it, you couldn't call `println!("{:?}", value)` on arbitrary `T`.

```rust
use std::fmt::Debug;

fn print_debug<T: Debug>(value: T) { println!("{:?}", value); }
fn clone_it<T: Clone>(value: &T) -> T { value.clone() }

// Usage: Bounds unlock specific operations on generic types.
print_debug(vec![1, 2, 3]); // Prints "[1, 2, 3]"
let cloned = clone_it(&vec![1, 2, 3]); // Returns cloned Vec
```

### Example: Multiple Trait Bounds

Use `+` to combine bounds inline, or `where` clause for complex signatures. Each bound adds capabilities you can use in the function body. The `where` form is preferred when bounds span multiple lines.

```rust
use std::fmt::{Debug, Display};

fn compare_and_show<T: PartialOrd + Display>(a: T, b: T) {
    if a < b { println!("{} < {}", a, b); }
}

fn complex_operation<T, U>(t: T, u: U)
where
    T: Clone + Debug + Default,
    U: AsRef<str> + Display,
{
    println!("T: {:?}, U: {}", t.clone(), u);
}

// Usage: Multiple bounds combined with + or where clause.
compare_and_show(1, 2);
complex_operation(42, "hello");
```

### Example: Bounds on Associated Types

Constrain iterator item types with `I: Iterator<Item = i32>` or `I::Item: Display`. The `Item` is an associated type determined by the iterator implementation. This enables generic functions over any iterator producing specific types.

```rust
use std::fmt::Display;

fn sum_iterator<I: Iterator<Item = i32>>(iter: I) -> i32 {
    iter.sum()
}

fn print_all<I>(iter: I) where I: Iterator, I::Item: Display {
    for item in iter { println!("{}", item); }
}

// Usage: Constrain iterator's Item type for specific operations.
let sum = sum_iterator(vec![1, 2, 3].into_iter()); // 6
print_all(vec!["a", "b", "c"]); // Prints each item
```

### Example: Lifetime Bounds on Type Parameters

`T: 'a` means `T` contains no references shorter than `'a`. `T: 'static` means `T` has no non-static borrows—required for thread spawning. These bounds constrain what references a type may contain.

```rust
fn store_ref<'a, T: 'a>(storage: &mut Option<&'a T>, value: &'a T) {
    *storage = Some(value);
}

fn spawn_task<T: Send + 'static>(value: T) {
    std::thread::spawn(move || { drop(value); });
}

// Usage: T: 'static required for spawning threads with owned data.
let num = 42;
let mut storage: Option<&i32> = None;
store_ref(&mut storage, &num);
```

### Example: Conditional Method Implementation

Add methods only when `T` satisfies certain bounds. `impl<T: Display> Wrapper<T>` adds `display()` only for displayable types. This enables progressive capability based on the inner type.

```rust
use std::fmt::Display;

#[derive(Debug, PartialEq)]
struct Wrapper<T>(T);

impl<T> Wrapper<T> {
    fn new(value: T) -> Self { Wrapper(value) }
}

impl<T: Display> Wrapper<T> {
    fn display(&self) { println!("{}", self.0); }
}

impl<T: Clone> Wrapper<T> {
    fn duplicate(&self) -> Self { Wrapper(self.0.clone()) }
}

// Usage: Methods appear only when inner type has required traits.
let w = Wrapper::new(42);
w.display(); // Works because i32: Display
let dup = w.duplicate(); // Works because i32: Clone
```

### Example: Sized and ?Sized Bounds

By default, `T` must be `Sized` (known size at compile time). Add `?Sized` to accept unsized types like `str`, `[u8]`, or `dyn Trait`. Unsized types can only be used through references.

```rust
fn takes_sized<T>(value: T) { drop(value); }

fn takes_unsized<T: ?Sized>(value: &T) -> usize {
    std::mem::size_of_val(value)
}

fn print_str<T: AsRef<str> + ?Sized>(s: &T) {
    println!("{}", s.as_ref());
}

// Usage: ?Sized accepts dynamically-sized types via reference.
takes_sized(42); // i32 has known size
let size = takes_unsized("hello"); // 5 bytes (str is unsized)
```

### Example: Supertraits (Trait Inheritance)

A supertrait bound like `Printable: Display + Debug` requires implementors to also implement the parent traits. The trait can use methods from its supertraits in default implementations. This creates trait hierarchies.

```rust
use std::fmt::{Debug, Display};

trait Printable: Display + Debug {
    fn print(&self) {
        println!("Display: {}, Debug: {:?}", self, self);
    }
}

impl Printable for i32 {}
impl Printable for String {}

// Usage: Supertrait methods available via the subtrait.
let num: i32 = 42;
num.print(); // Prints "Display: 42, Debug: 42"
```

### Example: Combining Bounds with impl Trait

`impl Trait` in return position hides the concrete type while exposing trait capabilities. In argument position, it's sugar for generics. Combine with `where` clauses for additional constraints.

```rust
fn make_iterator() -> impl Iterator<Item = i32> {
    (0..10).filter(|x| x % 2 == 0)
}

fn transform<T: Clone>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    items.into_iter().collect()
}

// Usage: impl Trait hides concrete type while exposing capabilities.
let evens: Vec<_> = make_iterator().collect(); // [0, 2, 4, 6, 8]
let v = transform(vec![1, 2, 3]); // Works with Vec
let v2 = transform([4, 5, 6]); // Also works with array
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

*   **Problem**: When designing a trait, should you use `trait Foo<T>` (generic parameter) or `trait Foo { type T; }` (associated type)? Both allow types to vary by implementation, but they have different semantics and ergonomics.
*   **Solution**: Use **associated types** for "output" types where each implementation specifies exactly one type (Iterator::Item). Use **generic parameters** for "input" types where the same type can implement the trait multiple times with different parameters (From<T>).
*   **Why It Matters**: Associated types simplify APIs dramatically. With `Iterator<Item = i32>`, you know the item type.

### Example: Associated Type - One Impl Per Type

Associated types declare `type Item` inside the trait; implementors specify the concrete type. Users access via `C::Item` without extra generic parameters. This is the pattern for `Iterator::Item`, `Deref::Target`.

```rust
trait Container {
    type Item;
    fn get(&self, index: usize) -> Option<&Self::Item>;
}

impl<T> Container for Vec<T> {
    type Item = T;
    fn get(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }
}

fn first_item<C: Container>(c: &C) -> Option<&C::Item> { c.get(0) }

// Usage: Associated type inferred from container; no turbofish needed.
let v = vec![1, 2, 3];
let first = first_item(&v); // Some(&1)
```

### Example: Generic Parameter - Multiple Impls Per Type

Generic parameters `trait Convertible<T>` allow one type to implement the trait multiple times. Use turbofish `Convertible::<String>::convert` to disambiguate. This is the pattern for `From<T>`, `Into<T>`, `Add<Rhs>`.

```rust
trait Convertible<T> { fn convert(&self) -> T; }

impl Convertible<String> for i32 {
    fn convert(&self) -> String { self.to_string() }
}
impl Convertible<f64> for i32 {
    fn convert(&self) -> f64 { *self as f64 }
}

// Usage: Same type converts to multiple targets; turbofish selects which.
let n: i32 = 42;
let s: String = Convertible::<String>::convert(&n); // "42"
let f: f64 = Convertible::<f64>::convert(&n); // 42.0
```

### Example: Associated Type with Bounds

Associated types can have bounds: `type Item: Add + Default`. This constrains what concrete types implementors can choose. The trait can use those bounds in default method implementations.

```rust
use std::ops::Add;

trait Summable {
    type Item: Add<Output = Self::Item> + Default + Copy;
    fn items(&self) -> &[Self::Item];

    fn sum(&self) -> Self::Item {
        self.items().iter().copied()
            .fold(Self::Item::default(), |acc, x| acc + x)
    }
}

struct Numbers(Vec<i32>);
impl Summable for Numbers {
    type Item = i32;
    fn items(&self) -> &[i32] { &self.0 }
}

// Usage: Bounded associated type enables default sum() implementation.
let nums = Numbers(vec![1, 2, 3, 4, 5]);
let total = nums.sum(); // 15
```

### Example: Generic Associated Types (GAT)

GATs add generic parameters to associated types: `type Item<'a>`. This enables returning references tied to the borrowing call. Essential for lending iterators and self-referential patterns.

```rust
trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

struct WindowsMut<'a, T> { slice: &'a mut [T], pos: usize }

impl<'a, T> LendingIterator for WindowsMut<'a, T> {
    type Item<'b> = &'b mut [T] where Self: 'b;
    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.pos + 2 <= self.slice.len() {
            let window = &mut self.slice[self.pos..self.pos + 2];
            self.pos += 1;
            Some(window)
        } else { None }
    }
}

// Usage: GAT enables lending iterator returning borrowed windows.
let mut data = vec![1, 2, 3, 4];
let mut windows = WindowsMut { slice: &mut data, pos: 0 };
if let Some(w) = windows.next() { /* w is &mut [1, 2] */ }
```

### Example: Type Families with Associated Types

Use associated types to create type-level mappings. A "family" trait maps a marker type to its member type. Functions generic over families can create family-specific values.

```rust
trait Family { type Member; }

struct IntFamily;
impl Family for IntFamily { type Member = i32; }

struct StringFamily;
impl Family for StringFamily { type Member = String; }

fn create_member<F: Family>() -> F::Member where F::Member: Default {
    F::Member::default()
}

// Usage: Type family maps marker type to its associated member type.
let int_val: i32 = create_member::<IntFamily>(); // 0 (default)
let str_val: String = create_member::<StringFamily>(); // "" (empty)
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

*   **Problem**: You want to provide trait implementations for all types meeting certain criteria, not just specific types. For example, any type implementing `Display` should automatically get `ToString`.
*   **Solution**: Use blanket implementations: `impl<T: Bound> Trait for T`. This implements `Trait` for all types `T` that satisfy `Bound`.
*   **Why It Matters**: Blanket implementations enable powerful trait composition. You define the relationship once, and it applies to all qualifying types—past, present, and future.

### Example: Blanket Impl for All Types Implementing a Trait

`impl<T: Display> Printable for T` gives `Printable` to every `Display` type automatically. This includes `i32`, `String`, and any future type implementing `Display`. The stdlib uses this for `ToString`: any `Display` type gets `to_string()`.

```rust
use std::fmt::Display;

trait Printable { fn print(&self); }

impl<T: Display> Printable for T {
    fn print(&self) { println!("{}", self); }
}

// Usage: Any Display type automatically gets print() method.
42.print(); // Works via blanket impl
"hello".print(); // Same blanket impl applies
```

### Example: Extension Trait with Blanket Impl

Define a trait extending another (`IteratorExt: Iterator`) and blanket-impl for all iterators. This adds methods to all iterators without modifying the `Iterator` trait. Libraries like `itertools` use this pattern extensively.

```rust
trait IteratorExt: Iterator {
    fn count_where<P: FnMut(&Self::Item) -> bool>(self, p: P) -> usize
    where Self: Sized;
}

impl<I: Iterator> IteratorExt for I {
    fn count_where<P: FnMut(&Self::Item) -> bool>(self, mut p: P) -> usize
    where Self: Sized {
        self.filter(|item| p(item)).count()
    }
}

// Usage: Extension adds count_where() to all iterators.
let evens = (0..10).count_where(|x| x % 2 == 0);
let pos = vec![-2, -1, 0, 1, 2].into_iter()
    .count_where(|x| *x > 0);
```

### Example: Into from From (Std Library Pattern)

The stdlib implements `Into<U>` for all `T where U: From<T>`. You only need to implement `From`, and `Into` comes free. This shows how blanket impls create trait relationships.

```rust
trait MyInto<T> { fn my_into(self) -> T; }
trait MyFrom<T> { fn my_from(value: T) -> Self; }

impl<T, U: MyFrom<T>> MyInto<U> for T {
    fn my_into(self) -> U { U::my_from(self) }
}

struct Meters(f64);
impl MyFrom<f64> for Meters {
    fn my_from(value: f64) -> Self { Meters(value) }
}

// Usage: Implement From, get Into free via blanket impl.
let m: Meters = 5.0.my_into(); // Uses blanket impl
```

### Example: Blanket Impl with References

If `T: Process`, implement `Process` for `&T` and `Box<T>` too. This enables calling trait methods through references and smart pointers. The impl delegates to the underlying type's implementation.

```rust
trait Process { fn process(&self) -> String; }

impl Process for i32 {
    fn process(&self) -> String { format!("processing {}", self) }
}

impl<T: Process> Process for &T {
    fn process(&self) -> String { (*self).process() }
}

impl<T: Process> Process for Box<T> {
    fn process(&self) -> String { (**self).process() }
}

// Usage: Trait works on value, reference, and Box transparently.
let num = 42;
num.process(); // Direct call
(&num).process(); // Through reference
Box::new(42).process(); // Through Box
```

### Example: Orphan Rules for Blanket Impls

You can blanket-impl your own trait for foreign types, or implement foreign traits on your types. But you cannot impl foreign traits on foreign types—this prevents conflicting impls across crates.

```rust
use std::fmt::{Debug, Display};

// ALLOWED: Your trait with blanket impl
trait MyTrait { fn my_method(&self); }
impl<T: Debug> MyTrait for T {
    fn my_method(&self) { println!("{:?}", self); }
}

// ALLOWED: Foreign trait on your type
struct MyType;
impl Display for MyType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MyType")
    }
}

// NOT ALLOWED: impl Display for Vec<i32> { ... }

// Usage: Your trait on foreign types OK; foreign trait on yours OK.
42.my_method(); // Your trait via blanket impl
let mt = MyType;
format!("{}", mt); // Display (foreign) on MyType (yours)
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

*   **Problem**: You want to track state or constraints at compile time without runtime overhead. For example, a file handle should only allow reading if opened in read mode, or a builder should require certain fields before building.
*   **Solution**: Use phantom types—type parameters that exist only in the type signature, not in the data layout. `PhantomData<T>` is a zero-sized type marker that tells the compiler "pretend this struct contains a T." Combined with zero-sized state marker types, this enables the type-state pattern.
*   **Why It Matters**: Phantom types move invariants from runtime to compile time. Invalid states become unrepresentable—you literally cannot call `.read()` on a write-only handle because the method doesn't exist for that type.

### Example: Type-State for Protocol States

Zero-sized marker types (`struct Connected;`) represent states. Methods consume `self` and return a new state type. Only valid transitions compile—`send()` exists only on `Authenticated`, not `Connected`.

```rust
use std::marker::PhantomData;

struct Disconnected;
struct Connected;
struct Authenticated;

struct Connection<State> {
    socket: String,
    _state: PhantomData<State>,
}

impl Connection<Disconnected> {
    fn new() -> Self {
        Connection { socket: String::new(), _state: PhantomData }
    }
    fn connect(self, addr: &str) -> Connection<Connected> {
        Connection { socket: addr.to_string(), _state: PhantomData }
    }
}

impl Connection<Connected> {
    fn authenticate(self, _creds: &str) -> Connection<Authenticated> {
        Connection { socket: self.socket, _state: PhantomData }
    }
}

impl Connection<Authenticated> {
    fn send(&mut self, _data: &[u8]) { /* only auth'd can send */ }
}

// Usage: Type state ensures send() only callable after authentication.
let conn = Connection::<Disconnected>::new();
let conn = conn.connect("localhost:8080"); // → Connected
let mut conn = conn.authenticate("secret"); // → Authenticated
conn.send(b"hello"); // Only works when authenticated
```

### Example: Units of Measure

Phantom type parameters prevent mixing incompatible units. `Quantity<f64, Meters>` and `Quantity<f64, Feet>` are distinct types. Addition is only defined for matching units—mismatched units won't compile.

```rust
use std::marker::PhantomData;
use std::ops::Add;

struct Meters;
struct Feet;

struct Quantity<T, Unit> { value: T, _unit: PhantomData<Unit> }

impl<T, Unit> Quantity<T, Unit> {
    fn new(value: T) -> Self { Quantity { value, _unit: PhantomData } }
}

impl<T: Add<Output = T>, Unit> Add for Quantity<T, Unit> {
    type Output = Quantity<T, Unit>;
    fn add(self, other: Self) -> Self::Output {
        Quantity::new(self.value + other.value)
    }
}

// Usage: Phantom unit type prevents adding incompatible quantities.
let m1: Quantity<f64, Meters> = Quantity::new(10.0);
let m2: Quantity<f64, Meters> = Quantity::new(5.0);
let m3 = m1 + m2; // 15.0 meters
// m3 + feet; // ERROR: can't add Meters and Feet
```

### Example: Builder with Required Fields

Multiple phantom parameters track which fields are set. `build()` only exists on `UserBuilder<HasName, HasEmail>`. Forgetting required fields is a compile error, not a runtime panic.

```rust
use std::marker::PhantomData;

struct NoName; struct HasName;
struct NoEmail; struct HasEmail;

struct UserBuilder<Name, Email> {
    name: Option<String>,
    email: Option<String>,
    _state: PhantomData<(Name, Email)>,
}

impl UserBuilder<NoName, NoEmail> {
    fn new() -> Self {
        UserBuilder { name: None, email: None, _state: PhantomData }
    }
}

impl<E> UserBuilder<NoName, E> {
    fn name(self, n: &str) -> UserBuilder<HasName, E> {
        UserBuilder {
            name: Some(n.into()), email: self.email, _state: PhantomData
        }
    }
}

impl<N> UserBuilder<N, NoEmail> {
    fn email(self, e: &str) -> UserBuilder<N, HasEmail> {
        UserBuilder {
            name: self.name, email: Some(e.into()), _state: PhantomData
        }
    }
}

impl UserBuilder<HasName, HasEmail> {
    fn build(self) -> (String, String) {
        (self.name.unwrap(), self.email.unwrap())
    }
}

// Usage: build() only available when all required fields are set.
let (name, email) = UserBuilder::new()
    .name("Alice")
    .email("a@b.com")
    .build(); // Only compiles with both fields set
```

### Example: FFI Ownership Marker

Phantom types distinguish owned vs borrowed pointers at the type level. Only `OwnedBuffer<Owned>` implements `Drop` to free memory. `OwnedBuffer<Borrowed>` has no drop—we don't own the data.

```rust
use std::marker::PhantomData;

struct Owned;
struct Borrowed;

struct OwnedBuffer<Ownership> {
    ptr: *mut u8,
    len: usize,
    _ownership: PhantomData<Ownership>,
}

impl OwnedBuffer<Owned> {
    fn new(data: &[u8]) -> Self {
        let boxed = data.to_vec().into_boxed_slice();
        let len = boxed.len();
        let ptr = Box::into_raw(boxed) as *mut u8;
        OwnedBuffer { ptr, len, _ownership: PhantomData }
    }
}

impl Drop for OwnedBuffer<Owned> {
    fn drop(&mut self) {
        unsafe {
            let slice = std::slice::from_raw_parts_mut(self.ptr, self.len);
            drop(Box::from_raw(slice));
        }
    }
}

impl OwnedBuffer<Borrowed> {
    unsafe fn from_ptr(ptr: *mut u8, len: usize) -> Self {
        OwnedBuffer { ptr, len, _ownership: PhantomData }
    }
    // No Drop - we don't own it
}

// Usage: Owned has Drop to free memory; Borrowed doesn't.
let owned = OwnedBuffer::<Owned>::new(b"hello"); // Drop frees memory
// OwnedBuffer<Borrowed> has no Drop—we don't own the data
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

*   **Problem**: You want to accept a closure that works with references of any lifetime, not a specific one. For example, a function that calls a closure with temporary references created inside the function.
*   **Solution**: Use higher-ranked trait bounds with `for<'a>` syntax: `F: for<'a> Fn(&'a str) -> &'a str`. This means "F implements Fn for all possible lifetimes 'a." The closure must work regardless of what lifetime the references have.
*   **Why It Matters**: HRTBs are essential for closure-heavy APIs. Without them, you couldn't write functions like `Vec::sort_by` that accept comparison closures operating on temporary references.

### Example: Basic HRTB for Closure with References

The `for<'a>` syntax means the closure must work for any
lifetime, not a specific one chosen by the caller. This is
essential when the function creates local values and passes
references to the closure. The closure cannot assume any
particular lifetime for those references.

```rust
fn call_with_ref<F>(f: F) -> usize
where
    F: for<'a> Fn(&'a str) -> usize,
{
    let local = String::from("hello");
    f(&local)  // 'a is lifetime of local
}

// Usage: Closure must handle any lifetime the function provides.
let len = call_with_ref(|s| s.len()); // 5
let count = call_with_ref(|s| s.chars().count()); // 5
```

### Example: HRTB vs Regular Lifetime Parameter

With regular lifetimes, the caller chooses the lifetime and
the function must accept it. With HRTB, the function chooses
the lifetime and the closure must handle whatever lifetime
is given.

```rust
// Regular: caller chooses lifetime
fn with_lifetime<'a, F>(s: &'a str, f: F) -> &'a str
where
    F: Fn(&'a str) -> &'a str,
{
    f(s)
}

// HRTB: function chooses, closure must handle any
fn with_hrtb<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let local = String::from("hello world");
    f(&local).to_string()  // f must work with local
}

// Usage: Regular lifetime from caller; HRTB for internal temporaries.
let s = "test";
let r = with_lifetime(s, |x| x); // Caller's lifetime
let result = with_hrtb(|x| x); // Function's internal lifetime
```

### Example: Fn Trait Bounds Are Sugar for HRTB

When you write `Fn(&str)` as a bound, Rust automatically
desugars it to `for<'a> Fn(&'a str)`. This makes common
closure bounds ergonomic while still being fully general.

```rust
use std::cell::Cell;

// These are equivalent:
fn takes_fn_sugar<F: Fn(&str)>(f: F) { f("hi"); }
fn takes_fn_explicit<F>(f: F)
where
    F: for<'a> Fn(&'a str),
{
    f("hi");
}

// Usage: Fn(&str) is sugar for for<'a> Fn(&'a str).
let count = Cell::new(0);
takes_fn_sugar(|_| count.set(count.get() + 1));
takes_fn_explicit(|_| count.set(count.get() + 1)); // Same as above
```

### Example: HRTB with Multiple Lifetimes

You can quantify over multiple lifetimes when the closure
takes multiple reference parameters. Each lifetime is
independent—`'a` and `'b` can be completely different.

```rust
fn call_with_two<F>(f: F) -> bool
where
    F: for<'a, 'b> Fn(&'a str, &'b str) -> bool,
{
    let s1 = String::from("hello");
    let s2 = String::from("world");
    f(&s1, &s2)  // Different lifetimes
}

// Usage: Multiple independent lifetimes in closure parameters.
let equal = call_with_two(|a, b| a == b); // false ("hello" != "world")
let longer = call_with_two(|a, b| a.len() > b.len()); // false
```

### Example: HRTB in Trait Definitions

When storing closures that process references, you need
HRTB to say "this closure works for any input lifetime."
This is common in parser combinators where the input
string's lifetime varies per call.

```rust
struct BoxedParser<Output> {
    parser: Box<
        dyn for<'a> Fn(&'a str) -> Option<(Output, &'a str)>
    >,
}

impl<Output> BoxedParser<Output> {
    fn new<F>(f: F) -> Self
    where
        F: for<'a> Fn(&'a str) -> Option<(Output, &'a str)>,
        F: 'static,
    {
        BoxedParser { parser: Box::new(f) }
    }

    fn parse<'a>(&self, input: &'a str)
        -> Option<(Output, &'a str)>
    {
        (self.parser)(input)
    }
}

// Usage: HRTB in stored closure handles varying input lifetimes.
let digit = BoxedParser::new(|s: &str| {
    s.chars().next()
        .filter(|c| c.is_ascii_digit())
        .map(|c| (c, &s[1..]))
});
digit.parse("123"); // Some(('1', "23"))
```

### Example: HRTB for Iterator Adapters

Iterator extension methods often take closures that work
with references to items. HRTB ensures the closure can
handle items with any lifetime, not just one specific one.

```rust
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

// Usage: HRTB lets closure borrow items with any lifetime.
let mut sum = 0;
vec![1, 2, 3].into_iter().for_each_ref(|x| sum += x); // sum = 6
```

### Example: Callback Storage with HRTB

Event emitters store callbacks that will be called later
with event data. Using HRTB in the callback type means
callbacks work with events of any lifetime.

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

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

// Usage: Stored callbacks work with events of any lifetime.
let mut emitter = EventEmitter::new();
emitter.on(|event| println!("Got: {}", event));
emitter.emit("click"); // Calls all registered callbacks
```

### Example: When HRTB Is Not Needed

Don't use HRTB when the lifetime comes from a function
parameter—the caller already chose it. Only use HRTB when
the function creates values internally and the closure must
handle those unknown lifetimes.

```rust
// DON'T need HRTB: lifetime comes from parameter
fn map_ref<'a, T, U, F>(value: &'a T, f: F) -> U
where
    F: Fn(&'a T) -> U,  // 'a from parameter
{
    f(value)
}

// DO need HRTB: lifetime created inside function
fn create_and_process<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,  // HRTB needed
{
    let local = String::from("temp");
    let _ = f(&local);  // local's lifetime unknown
}

// Usage: HRTB only needed when function creates internal temporaries.
let v = 42;
let doubled = map_ref(&v, |x| x * 2); // No HRTB needed
create_and_process(|s| s); // HRTB needed
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

*   **Problem**: You need to parameterize types by compile-time constant values, not just types. Arrays in Rust have their size in the type: `[i32; 5]` is different from `[i32; 10]`.
*   **Solution**: Use const generics: `struct Array<T, const N: usize>`. The `N` is a compile-time constant that becomes part of the type.
*   **Why It Matters**: Const generics enable zero-cost fixed-size abstractions. Matrix multiplication `Matrix<3, 4>` * `Matrix<4, 5>` = `Matrix<3, 5>` is checked at compile time.

### Example: Basic Const Generic Array

Const generics let you parameterize types by compile-time
values like `const N: usize`. The size becomes part of the
type, so `Array<i32, 5>` and `Array<i32, 10>` are distinct.
Methods can use `N` directly since it's known at compile time.

```rust
struct Array<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> Array<T, N> {
    fn new() -> Self {
        Array { data: [T::default(); N] }
    }
}

impl<T, const N: usize> Array<T, N> {
    fn len(&self) -> usize { N }  // Known at compile time

    fn get(&self, index: usize) -> Option<&T> {
        if index < N { Some(&self.data[index]) } else { None }
    }
}

// Usage: Size N is part of type; Array<i32, 5> differs from Array<i32, 10>.
let arr: Array<i32, 5> = Array::new();
let len = arr.len(); // 5 (compile-time constant)
let first = arr.get(0); // Some(&0)
```

### Example: Compile-Time Size Validation

You can use const blocks to assert properties of const
generics at compile time. This `NonEmpty` type guarantees
N > 0, making `first()` always safe without runtime checks.

```rust
struct NonEmpty<T, const N: usize> {
    data: [T; N],
}

impl<T, const N: usize> NonEmpty<T, N> {
    fn new(data: [T; N]) -> Self {
        const { assert!(N > 0, "NonEmpty requires N > 0") }
        NonEmpty { data }
    }

    fn first(&self) -> &T {
        &self.data[0]  // Always safe, N > 0
    }
}

// Usage: Const assertion prevents N=0 at compile time.
let valid: NonEmpty<i32, 3> = NonEmpty::new([1, 2, 3]);
let first = valid.first(); // &1, always safe
// NonEmpty::<i32, 0>::new([]); // Compile error!
```

### Example: Matrix with Dimension Checking

Matrices can use multiple const generics for rows and cols.
Operations like transpose and multiply are dimension-checked
at compile time—mismatched dimensions won't compile.

```rust
use std::ops::{Add, Mul};

struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

impl<T: Default + Copy, const R: usize, const C: usize>
    Matrix<T, R, C>
{
    fn new() -> Self {
        Matrix { data: [[T::default(); C]; R] }
    }

    fn transpose(self) -> Matrix<T, C, R> {
        let mut result = Matrix::<T, C, R>::new();
        for i in 0..R {
            for j in 0..C {
                result.data[j][i] = self.data[i][j];
            }
        }
        result
    }
}

// Dimension-checked multiplication
impl<T, const M: usize, const N: usize> Matrix<T, M, N>
where
    T: Default + Copy + Add<Output = T> + Mul<Output = T>,
{
    fn multiply<const P: usize>(
        &self, other: &Matrix<T, N, P>,
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

// Usage: Dimension mismatch is compile error; result dimensions inferred.
let a: Matrix<i32, 2, 3> = Matrix::new();
let b: Matrix<i32, 3, 4> = Matrix::new();
let c: Matrix<i32, 2, 4> = a.multiply(&b);
```

### Example: Fixed-Size Ring Buffer

Const generics are ideal for fixed-capacity data structures.
This ring buffer has its capacity baked into the type, so
no heap allocation needed and capacity is known statically.

```rust
struct RingBuffer<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
    len: usize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
    fn new() -> Self {
        RingBuffer {
            buffer: [None; N], head: 0, tail: 0, len: 0,
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
        if self.len == 0 { return None; }
        let value = self.buffer[self.head].take();
        self.head = (self.head + 1) % N;
        self.len -= 1;
        value
    }

    fn capacity(&self) -> usize { N }
}

// Usage: Capacity is part of the type; no heap allocation needed.
let mut buf = RingBuffer::<i32, 3>::new();
buf.push(1); // Ok
buf.push(2); // Ok
buf.push(3); // Ok
buf.push(4); // Err—buffer full
```

### Example: Const Generics with Expressions

You can use const expressions in return types. Here the
return type `[T; N * 2]` doubles the input array size.
The compiler computes `N * 2` at compile time.

```rust
fn double_array<T: Copy + Default, const N: usize>(
    arr: [T; N],
) -> [T; N * 2] {
    let mut result = [T::default(); N * 2];
    result[..N].copy_from_slice(&arr);
    result[N..].copy_from_slice(&arr);
    result
}

// Usage: Output size N*2 computed at compile time from input size N.
let doubled = double_array([1, 2, 3]); // [1, 2, 3, 1, 2, 3]
// Array size 6 determined at compile time
```

### Example: Const Generic Protocol Frames

Network protocols often have fixed-size frames. Using const
generics, different frame sizes become distinct types. Type
aliases make common sizes convenient to use.

```rust
struct Frame<const SIZE: usize> {
    header: [u8; 4],
    payload: [u8; SIZE],
    checksum: u32,
}

impl<const SIZE: usize> Frame<SIZE> {
    fn new(payload: [u8; SIZE]) -> Self {
        Frame { header: [0; 4], payload, checksum: 0 }
    }

    fn total_size(&self) -> usize {
        4 + SIZE + 4  // header + payload + checksum
    }
}

// Different frame types
type SmallFrame = Frame<64>;
type LargeFrame = Frame<1024>;
type JumboFrame = Frame<9000>;

// Usage: Type aliases make common frame sizes convenient.
let small = SmallFrame::new([0u8; 64]);
let size = small.total_size(); // 4 + 64 + 4 = 72
```

### Example: Const Bounds and Where Clauses

You can add constraints on const generics using where
clauses with array types. `[(); N - 2]:` asserts N >= 2
at compile time—smaller N causes a compile error.

```rust
fn requires_at_least_two<T, const N: usize>(
    arr: [T; N],
) -> (T, T)
where
    [(); N - 2]:,  // Assert N >= 2 at compile time
{
    let mut iter = arr.into_iter();
    (iter.next().unwrap(), iter.next().unwrap())
}

// Usage: Const bound [(); N - 2] asserts N >= 2 at compile time.
let (a, b) = requires_at_least_two([1, 2, 3]); // (1, 2)
// requires_at_least_two([1]); // Compile error: N < 2
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

### Performance Summary

| Feature | Compile Overhead | Runtime Overhead | Binary Size |
|---------|------------------|------------------|-------------|
| Monomorphization | High | None | Increases |
| Trait bounds | Low | None | None |
| Associated types | Low | None | None |
| PhantomData | None | None | None |
| Const generics | Medium | None | Varies |
| HRTBs | Low | None | None |

### Quick Reference: Choosing Generic Patterns

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

### Common Anti-Patterns

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

- Monomorphization is zero-cost: Generic code compiles to specialized machine code
- Trait bounds are contracts: Specify exactly what capabilities you need
- Associated types simplify APIs: Use for "output" types in traits
- Blanket impls enable composition: Implement once, apply everywhere
- Phantom types are free: Zero runtime cost for compile-time guarantees
- HRTBs unlock closures: Essential for callback-heavy APIs
- Const generics enable value-level types: Compile-time dimension checking
- Start minimal: Add bounds only when the compiler requires them

