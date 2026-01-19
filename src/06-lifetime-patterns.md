# Lifetime Patterns

This chapter explores essential lifetime patterns in Rust, covering how the compiler infers lifetimes through elision, how to use lifetime bounds for generic constraints, how higher-ranked trait bounds enable lifetime polymorphism for closures, the role of variance in subtyping, and how `Pin` makes self-referential structures safe.

## Pattern 1: Named Lifetimes and Elision

**Problem**: The compiler needs to know that references are valid, but annotating every single reference with a lifetime would be extremely verbose. It's not always obvious when lifetimes are required versus when they are inferred, and how to specify the relationship between multiple references.

**Solution**: Rust uses **lifetime elision rules** to automatically infer lifetimes in common, unambiguous cases. For more complex scenarios, you use explicit lifetime annotations like `'a` to tell the compiler how the lifetimes of different references relate to each other.

**Why it matters**: Lifetimes are a zero-cost, compile-time feature that prevents an entire class of memory safety bugs, like use-after-free errors. The elision rules make this powerful feature ergonomic, covering over 90% of use cases so you only need to write explicit lifetimes when the relationships are ambiguous.

**Use Cases**:
-   Functions that take multiple references (e.g., finding the longest of two strings).
-   Structs that hold references to data.
-   Functions that return a reference derived from one of its inputs.
-   Methods on structs that return references to the struct's data.

### Example: Why Lifetimes Exist

In languages like C, it's easy to accidentally return a pointer to memory that has been deallocated, leading to crashes.

```c
// C code - compiles but crashes!
char* get_string() {
    char buffer[100];
    strcpy(buffer, "Hello");
    return buffer; // Returns pointer to invalid stack memory!
}
```

The function returns a pointer to stack memory that's immediately deallocated. Using this pointer is undefined behavior. Lifetimes prevent this entire class of bugs:
### Example: Rust Prevents Dangling References

Rust's lifetime system catches dangling reference errors at compile time. When you try to return a reference to a local variable, the compiler sees that the data will be dropped when the function ends. This compile-time check eliminates use-after-free bugs that plague C and C++ programs.

```rust
// This Rust code will not compile.
// fn get_string() -> &str {
//     let s = String::from("Hello");
//     &s // Error: cannot return reference to local variable `s`
// }
```

The compiler sees that the returned reference doesn't outlive the function and rejects the code. Lifetimes encode "how long is this reference valid?" in the type system.

When a function takes multiple references and returns one, you must explicitly tell the compiler how the lifetimes are related. Here, `'a` ensures that the returned reference is valid for as long as the shorter of the two input references.
### Example: Basic Lifetime Annotation

When a function takes multiple references and returns one, you must explicitly tell the compiler how the lifetimes are related. Here, `'a` ensures that the returned reference is valid for as long as the shorter of the two input references.

```rust
// Explicit lifetime 'a
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// Usage: 'a ties return lifetime to both inputs; result valid while both exist.
let string1 = String::from("long string");
let string2 = String::from("short");
let result = longest(&string1, &string2);
println!("Longest: {}", result);
```
### Example: Lifetime Elision Rules
To avoid boilerplate, the compiler applies three rules to infer lifetimes automatically. You only need to write annotations when these rules are not sufficient.

- **Rule 1**: Each elided lifetime in a function's parameters gets its own distinct lifetime parameter.
- **Rule 2**: If there is exactly one input lifetime, that lifetime is assigned to all output lifetimes.
- **Rule 3**: If one of the parameters is `&self` or `&mut self`, its lifetime is assigned to all output lifetimes.

```rust
// Elision Rule 2 applies here.
// Compiler infers output lifetime equals input lifetime.
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

struct MyString<'a> {
    text: &'a str,
}

// Elision Rule 3 applies here.
// The output lifetime is tied to the lifetime of `&self`.
impl<'a> MyString<'a> {
    fn get_text(&self) -> &str {
        self.text
    }
}
// Usage: Elision infers lifetimes; no explicit annotations needed here.
let word = first_word("hello world"); // "hello"
let ms = MyString { text: "example" };
println!("{}", ms.get_text());
```

### Example: The `'static` Lifetime

The `'static` lifetime indicates that a reference is valid for the entire duration of the program. String literals are the most common example. Be cautious with `'static`, as it is rarely what you need for function inputs or outputs unless you are dealing with truly global data.

```rust
// `s` references data hardcoded into the program's binary.
let s: &'static str = "I have a static lifetime.";

// You can also create static data with `const`.
const STATIC_STRING: &'static str = "Also a static string.";
// Usage: 'static references are valid for the entire program duration.
println!("{} and {}", s, STATIC_STRING);
```

## Pattern 2: Lifetime Bounds

**Problem**: When a generic type holds a reference, the compiler needs to ensure that the referenced data lives at least as long as the generic type itself. For example, in a `struct Wrapper<'a, T>`, how do we ensure `T` doesn't contain a reference that dies before `'a`?

**Solution**: Use **lifetime bounds**. The syntax `T: 'a` means that the type `T` must "outlive" the lifetime `'a`.

**Why it matters**: Lifetime bounds are crucial for the safety of generic types that contain references. They ensure that you cannot create a generic struct holding a reference to data that might be destroyed while the struct is still in use.

**Use Cases**:
-   Generic structs that hold references, like caches or parsers.
-   Generic functions that work with borrowed data.
-   Traits that involve references in their method signatures or associated types.

### Example: Lifetime Bound on a Generic Struct

A struct containing a generic type with a reference needs a lifetime bound. Here, `T: 'a` ensures that whatever type `T` is, it does not contain any references that live for a shorter time than `'a`.

```rust
// `T: 'a` means `T` must outlive `'a`.
// In modern Rust, this bound is inferred from the `&'a T` field.
struct Wrapper<'a, T: 'a> {
    value: &'a T,
}
// Usage: T: 'a ensures wrapped reference doesn't outlive the data.
let num = 42;
let wrapped = Wrapper { value: &num };
```

### Example: `where` Clauses for Complex Bounds

For complex combinations of lifetime and trait bounds, a `where` clause can make the function signature much more readable.

```rust
fn process_and_debug<'a, T>(items: &'a [T])
where
    T: std::fmt::Debug + 'a, // T must outlive 'a
{
    for item in items {
        println!("Item: {:?}", item);
    }
}
// Usage: where clause combines trait bound (Debug) with lifetime bound ('a).
process_and_debug(&[1, 2, 3]);
```

2. **Implementing traits with lifetime parameters**

```rust
trait Parser {
    fn parse<'a>(&self, input: &'a str) -> Option<&'a str>;
}
```

## Pattern 3: Higher-Ranked Trait Bounds (for Lifetimes)

**Problem**: You need to write a function that accepts a closure, but the closure must work for *any* lifetime, not just one specific lifetime that you can name. This is common for iterator adapters or any function that calls a closure with locally created references.

**Solution**: Use a **Higher-Ranked Trait Bound (HRTB)** with the `for<'a>` syntax. The bound `F: for<'a> Fn(&'a str)` means that the closure `F` must work for a reference `&str` of *any* possible lifetime `'a`.

**Why it matters**: HRTBs are the key to Rust's powerful and flexible functional programming patterns. They allow you to write generic, higher-order functions that accept closures operating on borrowed data, without needing to tie the closure to a single, specific lifetime.

**Use Cases**:
-   Iterator adapters like `map`, `filter`, and `for_each` when working with references.
-   Functions that accept callbacks or event handlers.
-   Parser combinator libraries.

### Example: A Function Accepting a Lifetime-Generic Closure

The `call_on_hello` function creates a local string and calls a closure on a reference to it. The closure must be able to handle this local, temporary lifetime. The `for<'a>` bound ensures this.

```rust
// HRTB `for<'a> Fn(&'a str)` ensures `f` works for any lifetime.
fn call_on_hello<F>(f: F)
where
    F: for<'a> Fn(&'a str),
{
    let s = String::from("hello");
    f(&s); // Closure called with reference local to this function.
}

// Usage: for<'a> means closure handles any lifetime the function provides.
let print_it = |s: &str| println!("{}", s);
call_on_hello(print_it);
```

### Example: Trait with a Higher-Ranked Method

You can use HRTBs in traits to define methods that are generic over lifetimes. This is common in "streaming iterator" or "lending iterator" patterns.

```rust
trait Processor {
    // This method must work for any input lifetime 'a.
    fn process<'a>(&self, data: &'a str) -> &'a str;
}

struct Trimmer;

impl Processor for Trimmer {
    fn process<'a>(&self, data: &'a str) -> &'a str {
        data.trim()
    }
}
// Usage: Method is generic over 'a; works with any input lifetime.
let trimmer = Trimmer;
let result = trimmer.process("  hello  "); // "hello"
```

## Pattern 4: Self-Referential Structs and `Pin`

**Problem**: It is normally impossible to create a struct that holds a reference to one of its own fields. The borrow checker forbids this because if the struct were moved, the internal reference would become invalid (dangling).

**Solution**: Use `Pin<T>`. A `Pin` "pins" a value to its location in memory, guaranteeing that it will not be moved.

**Why it matters**: `Pin` is the cornerstone that makes async/await in Rust work safely and efficiently. Futures in async Rust are often self-referential, and `Pin` ensures that they can be polled without their internal references being invalidated.

**Use Cases**:
-   Async `Future`s, which store state across `.await` points.
-   Generators and other coroutines.
-   Intrusive data structures like linked lists where nodes are embedded within other objects.

### Example: The Problem with Self-Reference

This code demonstrates why safe Rust disallows self-referential structs. You cannot create a reference to a field before the struct is fully constructed and moved into its final memory location.

```rust
// This will not compile.
// struct SelfReferential<'a> {
//     data: String,
//     // This reference is supposed to point to `data`.
//     reference: &'a str,
// }
```

### Example: A Safe Alternative Using Indices

Instead of direct references, you can use indices into a collection. This avoids the self-reference problem because indices remain valid even if the collection is moved.

```rust
// Graph where nodes reference each other via indices.
struct Node {
    name: String,
    edges: Vec<usize>, // Indices into graph's `nodes` vector.
}

struct Graph {
    nodes: Vec<Node>,
}
// Usage: Indices stay valid even if vec reallocates; avoids self-reference.
let mut graph = Graph {
    nodes: vec![Node { name: "A".into(), edges: vec![1] }]
};
```

### Example: A Pinned, Self-Referential Struct (Unsafe)

This example shows how `Pin` and `unsafe` can be used to create a truly self-referential struct. This is an advanced technique and should be used with great care.

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct Unmovable {
    data: String,
    slice: *const str, // A raw pointer, not a reference
    _pin: PhantomPinned,
}

impl Unmovable {
    fn new(data: String) -> Pin<Box<Self>> {
        let res = Unmovable {
            data,
            // Can't init `slice` yet, `data` is not pinned.
            slice: std::ptr::null(),
            _pin: PhantomPinned,
        };
        let mut boxed = Box::pin(res);

        // Now that data is pinned, we can create a pointer to it.
        let slice = &boxed.data[..] as *const str;
        // And update the `slice` field with the correct pointer.
        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).slice = slice;
        }

        boxed
    }

    fn data(&self) -> &str {
        &self.data
    }

    fn reference(&self) -> &str {
        unsafe { &*self.reference }
    }
}
```

`Pin` guarantees the struct won't move, making the self-reference safe. This is advanced and requires `unsafe`.

### Solution 3: Rental Crates

Libraries like `ouroboros` provide safe abstractions for self-referential structs without requiring `unsafe` code. These crates use procedural macros to generate the necessary boilerplate for pinning and initialization. The `#[self_referencing]` attribute transforms your struct definition into a safe, usable self-referential type.

```rust
// Using ouroboros crate
use ouroboros::self_referencing;

#[self_referencing]
struct SelfRef {
    data: String,
    #[borrows(data)]
    reference: &'this str,
}

fn example() {
    let s = SelfRefBuilder {
        data: "hello".to_string(),
        reference_builder: |data| &data[..],
    }.build();

    s.with_reference(|r| println!("{}", r));
}
```

The `ouroboros` crate uses macros and `Pin` internally to provide a safe interface.

### Solution 4: Restructure the Design

Often, the need for self-references signals a design that can be restructured. Instead of a struct owning data and holding a reference to it, separate the owner from the borrower into two distinct types. This approach is idiomatic Rust and works naturally with the borrow checker.

```rust
// Instead of self-referential:
struct BadDesign<'a> {
    data: String,
    view: &'a str,
}

// Use two separate types:
struct Data {
    content: String,
}

struct View<'a> {
    data: &'a Data,
    window: &'a str,
}

impl Data {
    fn view(&self) -> View {
        View {
            data: self,
            window: &self.content[..],
        }
    }
}
// Usage: Separate owner (Data) from borrower (View) to avoid self-reference.
let data = Data { content: "hello".into() };
let view = data.view(); // View borrows from Data
```

This separates ownership from borrowing, eliminating the self-reference.

### When Self-References Are Actually Needed

Rare cases that truly need self-references:

1. **Async runtimes**: Futures containing references to their own data
2. **Parsers**: Holding both input buffer and views into it
3. **Game engines**: Scene graphs with parent-child relationships

For these, use `Pin`, arena allocation, or specialized crates.

## Pattern 5: Variance and Subtyping

**Problem**: Lifetime subtyping rules unclear—when can `&'long` be used where `&'short` expected? Covariant vs contravariant vs invariant confusing.

**Solution**: Covariant types accept longer lifetimes: `&'a T` is covariant in 'a, so `&'long T` usable where `&'short T` expected. Invariant types require exact lifetime: `&mut 'a T` is invariant—can't substitute.

**Why It Matters**: Enables flexible lifetime assignments—longer lifetime works where shorter needed. Immutable reference covariance allows ergonomic code: can pass long-lived reference to function expecting short.

**Use Cases**: Reference wrappers (determining if Wrapper<'a> covariant), iterator chains (covariant iterators compose naturally), function pointers (contravariant arguments, covariant returns), trait objects (variance of dyn Trait<'a>), smart pointers with references (Arc<&'a T> variance), custom pointer types (controlling subtyping behavior with PhantomData), phantom data usage (adding variance markers to generic types).

```rust
// Usage: Longer lifetime ('static) substitutes for shorter; subtyping in action.
let outer: &'static str = "hello";
let inner: &str = outer; // OK: 'static subtype of shorter
```

If `'long: 'short` (read: "'long outlives 'short"), then `'long` is a subtype of `'short`. You can use a longer lifetime where a shorter one is expected.

### Variance Categories

Types have variance with respect to their lifetime and type parameters, determining how subtyping relationships propagate. Covariance means a longer lifetime can substitute for a shorter one. Invariance means no substitution is allowed, while contravariance (rare) reverses the direction.

**Covariant**: Subtyping flows in the same direction

```rust
// &'a T is covariant over 'a
// If 'a: 'b, then &'a T <: &'b T

// Usage: Covariant &'a T allows longer 'static where shorter 'a expected.
let long: &'static str = "hello";
let short: &str = long; // OK
```

**Invariant**: No subtyping allowed

```rust
// &'a mut T is invariant over 'a
// Cannot substitute different lifetimes

fn invariant_example<'a, 'b>(x: &'a mut i32, y: &'b mut i32)
where
    'a: 'b,
{
    // Cannot assign x to y even though 'a: 'b
    // let z: &'b mut i32 = x; // Error!
}
```

**Contravariant**: Subtyping flows in opposite direction (rare in Rust)

```rust
// Function arguments are contravariant over lifetimes
// fn(&'a T) is contravariant over 'a
```

### Why Variance Matters

Variance determines when types are compatible and when lifetime substitutions are safe. Covariance allows flexibility—a `&'static str` can be used anywhere a shorter-lived `&'a str` is expected. Invariance prevents unsound substitutions that could create dangling references through mutable aliasing.

```rust
// Covariance allows this:
fn take_short(x: &str) {}

// Usage: Covariance lets 'static str pass to function expecting &str.
let s: &'static str = "hello";
take_short(s); // OK: 'static works where shorter expected

// Invariance prevents this:
fn swap<'a, 'b>(x: &'a mut &'static str, y: &'b mut &'a str) {
    // std::mem::swap(x, y); // Error! Invariance prevents swapping
}
```

### Variance in Practice

Most standard library types are covariant over their type parameters, enabling flexible composition. Mutable references and interior mutability types are invariant because they could otherwise be exploited to create unsound aliasing. Understanding these rules helps predict which type substitutions the compiler will accept.

```rust
// Covariant:
// &'a T
// *const T
// fn() -> T
// Vec<T>, Box<T>, etc.

// Invariant:
// &'a mut T
// *mut T
// Cell<T>, UnsafeCell<T>

// Contravariant (rare):
// fn(T) -> ()
```

Example showing practical impact:

```rust
struct Producer<T> {
    produce: fn() -> T, // Covariant over T
}

struct Consumer<T> {
    consume: fn(T), // Contravariant over T
}

// Usage: Producer covariant (longer → shorter OK); Consumer contravariant.
let p: Producer<&'static str> = Producer { produce: || "hello" };
let _p2: Producer<&str> = p; // OK: covariant
let c: Consumer<&str> = Consumer { consume: |_s| {} };
// let _c2: Consumer<&'static str> = c; // Would error: contravariant
```

### Interior Mutability and Invariance

Types with interior mutability like `Cell<T>` and `RefCell<T>` are invariant over their contents. If they were covariant, you could store a short-lived reference and later read it as a longer-lived one—a soundness hole. This invariance is the price of safe interior mutation.

```rust
use std::cell::Cell;

fn example() {
    let cell: Cell<&'static str> = Cell::new("hello");

    // Cannot do this even though 'static: 'a
    // fn take_cell<'a>(c: Cell<&'a str>) {}
    // take_cell(cell); // Error! Cell is invariant
}
```

Invariance prevents creating references with incorrect lifetimes through interior mutability.

### PhantomData and Variance

`PhantomData<T>` lets you control a type's variance without actually storing `T`. The marker inherits the variance of whichever type you use—`PhantomData<T>` is covariant, `PhantomData<fn(T)>` is contravariant, and `PhantomData<Cell<T>>` is invariant. This is essential for unsafe code that logically owns or references data without a physical field.

```rust
use std::marker::PhantomData;

// Covariant over T
struct Covariant<T> {
    _marker: PhantomData<T>,
}

// Invariant over T
struct Invariant<T> {
    _marker: PhantomData<Cell<T>>,
}

// Contravariant over T (rare)
struct Contravariant<T> {
    _marker: PhantomData<fn(T)>,
}
// Usage: PhantomData controls variance without storing T.
let cov: Covariant<&'static str> = Covariant {
    _marker: PhantomData
};
```

Use `PhantomData` when you need to control variance without storing the actual type.

### Subtyping and Higher-Rank Trait Bounds

HRTBs and variance work together to enable flexible generic code. A closure satisfying `for<'a> Fn(&'a str)` must handle any lifetime, which works because covariance lets longer lifetimes substitute for shorter ones. This combination powers iterator adapters and callback-based APIs.

```rust
// Works because of variance
fn accepts_any_lifetime<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> String,
{
    f("hello");
}

// Usage: HRTB + covariance lets closure work with any provided lifetime.
accepts_any_lifetime(|s: &str| s.to_uppercase());
```

The HRTB ensures the function works with any lifetime, leveraging variance.

### Lifetime Subtyping in APIs

When designing APIs, prefer covariant types (immutable references) for maximum flexibility. A `GoodReader<'static>` can be used wherever `GoodReader<'a>` is expected because covariance allows longer lifetimes to substitute. Use mutable references only when mutation is truly required, as their invariance restricts how callers can use your API.

```rust
// Good: covariant, flexible
struct GoodReader<'a> {
    data: &'a [u8],
}

impl<'a> GoodReader<'a> {
    fn read(&self) -> &'a [u8] {
        self.data
    }
}

// Can use GoodReader<'static> where GoodReader<'a> expected

// Bad: invariant, inflexible
struct BadReader<'a> {
    data: &'a mut [u8],
}

// Usage: Covariant GoodReader accepts longer lifetimes; more flexible API.
let reader = GoodReader { data: b"hello" };
let bytes = reader.read();
```

Prefer immutable references for flexibility unless mutation is necessary.

## Pattern 6: Advanced Lifetime Patterns

Let's explore some sophisticated patterns that combine these concepts.

### Lifetime Bounds with Closures

When a closure parameter needs to work with references tied to other inputs, you combine lifetime bounds with generic constraints. The closure receives references that share the same lifetime as the data passed to the function. This pattern is common in APIs that process data with user-provided callbacks.

```rust
fn process_with_context<'a, F, T>(
    data: &'a str,
    context: &'a T,
    f: F,
) -> String
where
    F: Fn(&'a str, &'a T) -> String,
{
    f(data, context)
}
// Usage: Closure receives refs with same lifetime as function inputs.
let ctx = "prefix";
let result = process_with_context(
    "data", &ctx, |d, c| format!("{}: {}", c, d));
```

The closure receives references tied to the input lifetimes.

### Streaming Iterators

Standard iterators own their items, but streaming iterators yield references tied to the iterator's own lifetime. This pattern uses Generic Associated Types (GATs) to express that each yielded item borrows from the iterator. It enables memory-efficient iteration over sliding windows or parsed chunks without copying data.

```rust
trait StreamingIterator {
    type Item<'a> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

struct WindowIter<'a> {
    data: &'a [i32],
    window_size: usize,
    position: usize,
}

impl<'a> StreamingIterator for WindowIter<'a> {
    type Item<'b> = &'b [i32] where Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position + self.window_size <= self.data.len() {
            let start = self.position;
            let end = self.position + self.window_size;
            let window = &self.data[start..end];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}
// Usage: GAT enables iterator that yields refs tied to iterator's lifetime.
let data = [1, 2, 3, 4];
let mut iter = WindowIter {
    data: &data, window_size: 2, position: 0
};
while let Some(window) = iter.next() { println!("{:?}", window); }
```

This pattern allows iterators to yield references tied to the iterator's lifetime.

### Lifetime Elision in Impl Blocks

Lifetime elision rules also apply within impl blocks, reducing annotation noise. Methods returning references to struct fields typically have their lifetimes inferred from `&self`. When you need different lifetimes for different parameters, you can introduce additional lifetime parameters on individual methods.

```rust
struct Parser<'a> {
    input: &'a str,
}

impl<'a> Parser<'a> {
    // Elided: fn parse(&self) -> Option<&str>
    // Actual: fn parse(&'a self) -> Option<&'a str>
    fn parse(&self) -> Option<&str> {
        Some(self.input)
    }

    // Multiple lifetimes when needed
    fn parse_with<'b>(&self, other: &'b str) -> (&'a str, &'b str) {
        (self.input, other)
    }
}
// Usage: Elision ties method return lifetime to &self automatically.
let parser = Parser { input: "hello world" };
let parsed = parser.parse(); // Some("hello world")
```

### Anonymous Lifetimes

The `'_` placeholder indicates "infer this lifetime" without requiring a name. It improves readability when the specific lifetime doesn't matter—the compiler still enforces correctness. Use `'_` in function signatures and impl blocks where elision rules would apply but you want to be explicit.

```rust
impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'_ str> {
        // '_ = 'a in this context
        Some(self.input)
    }
}

// Generic context
fn get_first<T>(vec: &Vec<T>) -> Option<&'_ T> {
    vec.first()
}
// Usage: '_ placeholder lets compiler infer lifetime without naming it.
let v = vec![1, 2, 3];
let first = get_first(&v); // Some(&1)
```

Anonymous lifetimes improve readability when the specific lifetime name doesn't matter.

### Summary

This chapter covered lifetime patterns for ensuring reference validity:

1. **Named Lifetimes and Elision**: Three elision rules infer common cases, explicit 'a for complex relationships
2. **Lifetime Bounds and Where Clauses**: T: 'a (T outlives 'a), 'b: 'a ('b outlives 'a), implied bounds
3. **Higher-Ranked Trait Bounds**: for<'a> Fn(&'a str) for lifetime-polymorphic closures
4. **Self-Referential Structs and Pin**: Pin<T> enables safe self-references, essential for async
5. **Variance and Subtyping**: Covariant (&'a T), invariant (&mut 'a T), determines lifetime substitution

**Key Takeaways**:
- Elision rules cover 90%+ cases: let compiler infer when possible
- Lifetimes prevent use-after-free: references can't outlive data
- Zero runtime cost: lifetimes compile-time only, erased after checking
- HRTBs enable flexible closures: for<'a> means "for all lifetimes"
- Pin makes async possible: self-referential futures safe when pinned

**Lifetime Elision Rules**:
1. Each elided lifetime gets distinct parameter: `fn foo(x: &i32)` → `fn foo<'a>(x: &'a i32)`
2. Single input lifetime → all output lifetimes: `fn foo(x: &str) → &str` → `fn foo<'a>(x: &'a str) → &'a str`
3. &self lifetime → all output lifetimes: `fn get(&self) → &T` → `fn get(&'a self) → &'a T`

**Variance Rules**:
- Covariant: `&'a T` accepts longer lifetimes ('long usable where 'short expected)
- Invariant: `&mut 'a T` requires exact lifetime (no substitution)
- Contravariant: function arguments (rare, opposite direction)

**Common Patterns**:
```rust
// Explicit lifetime for multiple parameters
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Struct with lifetime
struct Parser<'a> {
    input: &'a str,
}

// Lifetime bounds
fn process<'a, T: 'a>(data: &'a T) -> &'a T { data }

// Higher-ranked trait bound
fn apply<F>(f: F) where F: for<'a> Fn(&'a str) -> String {
    f("hello");
}

// Variance example
let long: &'static str = "hello";
let short: &str = long; // OK: 'static is subtype of shorter
```


### Summary
**When to Use What**:
- Elision: Most function signatures (let compiler infer)
- Explicit 'a: Multiple references, unclear relationships
- Multiple lifetimes: Independent lifetimes (Context<'user, 'session>)
- 'static: Program-duration data (string literals, leaked allocations)
- T: 'a bounds: Generic types with references
- for<'a>: Closures accepting references
- Pin: Self-referential types, async futures

**Debugging Lifetime Errors**:
- Read error carefully: compiler explains what outlives what
- Draw lifetime diagram: visualize scope of each reference
- Simplify: remove complexity to isolate issue
- Check elision: explicit annotations when elision fails
- Use '_ placeholder: anonymous lifetime for clarity
- Compiler suggestions: often provide exact fix

**Performance**:
- Lifetimes: zero runtime cost, compile-time only
- Bounds checking: compile-time, no runtime impact
- HRTB: monomorphization, no runtime overhead
- Pin: zero-cost abstraction when used correctly
- Variance: compile-time subtyping rules, no cost

**Anti-Patterns**:
- Overusing 'static (rarely needed, only for program-duration)
- Fighting the borrow checker (restructure instead)
- Cloning to avoid lifetimes (understand issue first)
- Multiple unnecessary lifetimes (use one when possible)
- Ignoring compiler suggestions (they're usually correct)

