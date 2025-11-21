# Lifetime Patterns

[Named Lifetimes and Elision](#pattern-1-named-lifetimes-and-elision-rules)

- Problem: References need lifetime annotations; unclear when to write explicit lifetimes; elision rules obscure
- Solution: Elision rules infer common cases; explicit 'a for multiple inputs/outputs; 'static for program-duration
- Why It Matters: Prevents use-after-free; type system encodes validity duration; zero runtime cost; most cases elided
- Use Cases: Multiple reference parameters, struct with references, return reference from multiple sources, explicit lifetime relationships

[Lifetime Bounds and Where Clauses](#pattern-2-lifetime-bounds-and-where-clauses)

- Problem: Generic types with references need constraints; unclear which lifetime outlives which; complex relationships unreadable
- Solution: T: 'a bounds (T outlives 'a); where 'b: 'a ('b outlives 'a); implied bounds from context; where clauses for clarity
- Why It Matters: Ensures references in generics remain valid; establishes lifetime hierarchies; enables flexible generic APIs
- Use Cases: Generic structs with references, cache implementations, parser types, complex lifetime relationships, trait bounds

[Higher-Ranked Trait Bounds (HRTB)](#pattern-3-higher-ranked-trait-bounds-hrtb)

- Problem: Closures accepting references need "for any lifetime"; can't express Fn(&'??? str) → String; lifetime parameter scope unclear
- Solution: for<'a> Fn(&'a str) → String means "for all lifetimes 'a"; HRTB in trait bounds; closure lifetime polymorphism
- Why It Matters: Enables lifetime-generic closures; function traits work with any reference; essential for iterator adapters
- Use Cases: Closure parameters, function traits, iterator combinators, higher-order functions, generic callbacks

[Variance and Subtyping](#pattern-4-variance-and-subtyping)

- Problem: Lifetime subtyping unclear; when can &'long be used where &'short expected? Covariant/contravariant/invariant confusing
- Solution: Covariant: T<'a> accepts longer lifetimes; invariant: exactly 'a required; contravariant: rare, function inputs
- Why It Matters: Enables flexible lifetime assignments; longer lifetime usable where shorter expected; variance determines safety
- Use Cases: Reference wrappers, iterator chains, function pointers, trait objects, smart pointers with references

[Self-Referential Structs and Pin](#pattern-5-self-referential-structs-and-pin)

- Problem: Struct referencing its own data causes lifetime issues; movable self-referential structs unsound; async needs self-references
- Solution: Pin<T> makes values unmovable; self-referential via unsafe + Pin; async runtime uses Pin for futures
- Why It Matters: Enables async/await; safe self-referential types possible; intrusive data structures; zero-cost futures
- Use Cases: Async futures, generators, intrusive linked lists, self-referential iterators, arena-allocated graphs


## Overview
This chapter explores lifetime patterns: elision rules for inference, bounds for generic constraints, higher-ranked bounds for closure lifetime polymorphism, variance for subtyping, and Pin for self-referential structures.

## Pattern 1: Named Lifetimes and Elision Rules

**Problem**: References need lifetime annotations to ensure validity—compiler must know how long reference lives. Unclear when explicit lifetimes required vs inferred. Elision rules not obvious (3 rules). Multiple reference parameters ambiguous—which lifetime to return? Struct with references needs lifetime parameters. Returning reference from one of multiple sources unclear. Reading code with lifetimes difficult. 'static overused when not needed.

**Solution**: Elision rules handle common cases automatically: (1) each elided lifetime gets distinct parameter, (2) single input lifetime → all output lifetimes, (3) &self lifetime → all output lifetimes. Explicit `'a` notation for multiple inputs/outputs: `fn longest<'a>(x: &'a str, y: &'a str) → &'a str`. Multiple distinct lifetimes `'a, 'b` when independent. 'static for program-duration data (string literals, constants). Compiler errors guide when annotations needed.

**Why It Matters**: Prevents use-after-free bugs—C's classic "returning pointer to stack" impossible in Rust. Type system encodes "how long is this valid"—references can't outlive data. Zero runtime cost: lifetimes are compile-time only, erased after checking. Most cases elided: don't write annotations unless needed. Elision rules cover 90%+ of cases. Errors are clear: "lifetime mismatch" with suggestions. Self-documenting: `&'a str` shows relationship between inputs/outputs.

**Use Cases**: Multiple reference parameters (longest string function), structs with references (Parser<'a> with input reference), returning reference from multiple sources (choose first or second), explicit lifetime relationships (Context with multiple lifetimes), methods returning references tied to struct lifetime, generic functions with reference parameters, cache/parser types holding borrowed data.

### Why Lifetimes Exist

```c
// C code - compiles but crashes!
char* get_string() {
    char buffer[100];
    strcpy(buffer, "Hello");
    return buffer; // Returns pointer to stack memory!
}
```

The function returns a pointer to stack memory that's immediately deallocated. Using this pointer is undefined behavior. Lifetimes prevent this entire class of bugs:

```rust
// Rust code - won't compile!
fn get_string() -> &str {
    let s = String::from("Hello");
    &s // Error: cannot return reference to local variable
}
```

The compiler sees that the returned reference doesn't outlive the function and rejects the code. Lifetimes encode "how long is this reference valid?" in the type system.

### Basic Lifetime Annotations

Lifetime annotations use apostrophe syntax:

```rust
// Explicit lifetime 'a
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn example() {
    let string1 = String::from("long string");
    let string2 = String::from("short");

    let result = longest(&string1, &string2);
    println!("Longest: {}", result);
}
```

The annotation `'a` is a lifetime parameter. It says: "the returned reference is valid for the same scope as the shorter of the two input references." The compiler enforces this:

```rust
fn example_invalid() {
    let string1 = String::from("long string");
    let result;

    {
        let string2 = String::from("short");
        result = longest(&string1, &string2);
    } // string2 dropped here

    // Error! result might point to dropped data
    // println!("{}", result);
}
```

### Lifetime Elision Rules

Writing lifetimes everywhere would be tedious. The compiler applies elision rules to infer lifetimes in common cases:

**Rule 1: Each elided lifetime gets a distinct parameter**

```rust
// What you write:
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap()
}

// What the compiler sees:
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap()
}
```

**Rule 2: If there's exactly one input lifetime, it's assigned to all output lifetimes**

```rust
// What you write:
fn get_first(vec: &Vec<i32>) -> &i32 {
    &vec[0]
}

// What the compiler sees:
fn get_first<'a>(vec: &'a Vec<i32>) -> &'a i32 {
    &vec[0]
}
```

**Rule 3: If there's a `&self` or `&mut self`, that lifetime is assigned to all output lifetimes**

```rust
impl<'a> Container<'a> {
    // What you write:
    fn get_data(&self) -> &str {
        self.data
    }

    // What the compiler sees:
    fn get_data(&'a self) -> &'a str {
        self.data
    }
}
```

These rules cover the vast majority of cases. When they don't apply, you need explicit annotations.

### When Elision Doesn't Work

If the compiler can't determine the lifetime, you must annotate explicitly:

```rust
// Multiple inputs, no &self - must annotate
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Different lifetimes for different parameters
fn mix<'a, 'b>(x: &'a str, y: &'b str) -> (&'a str, &'b str) {
    (x, y)
}

// Returning one of several references
fn choose<'a>(x: &'a str, y: &'a str, first: bool) -> &'a str {
    if first { x } else { y }
}
```

The annotations communicate to the compiler (and readers) how the lifetimes relate.

### Multiple Lifetime Parameters

Sometimes you need multiple distinct lifetimes:

```rust
struct Context<'a, 'b> {
    user: &'a User,
    session: &'b Session,
}

impl<'a, 'b> Context<'a, 'b> {
    fn new(user: &'a User, session: &'b Session) -> Self {
        Context { user, session }
    }

    // Return reference with lifetime 'a
    fn get_user(&self) -> &'a User {
        self.user
    }

    // Return reference with lifetime 'b
    fn get_session(&self) -> &'b Session {
        self.session
    }
}

struct User {
    name: String,
}

struct Session {
    token: String,
}

fn example() {
    let user = User { name: "Alice".to_string() };

    {
        let session = Session { token: "abc123".to_string() };
        let ctx = Context::new(&user, &session);

        let user_ref = ctx.get_user();  // Lives as long as user
        let session_ref = ctx.get_session(); // Lives as long as session
    }

    // user still valid here, session is not
}
```

Multiple lifetimes give you fine-grained control over which references outlive which others.

### Static Lifetime

The `'static` lifetime is special—it means "lives for the entire program":

```rust
// String literals have 'static lifetime
let s: &'static str = "Hello, world!";

// Constants have 'static lifetime
const MAX_SIZE: &'static str = "100";

// Leaked allocations become 'static
let leaked: &'static str = Box::leak(Box::new(String::from("leaked")));
```

Be careful with `'static`—it's often not what you want:

```rust
// Common mistake
fn get_string() -> &'static str {
    // Error! Cannot return 'static reference to non-static data
    // let s = String::from("Hello");
    // &s
}

// Correct: return owned String instead
fn get_string() -> String {
    String::from("Hello")
}
```

Only use `'static` when the data truly lives for the entire program duration.

## Pattern 2: Lifetime Bounds and Where Clauses

**Problem**: Generic types with references need lifetime constraints—`Wrapper<T>` with `&'a T` must ensure T outlives 'a. Unclear which lifetime outlives which in complex scenarios. Multiple lifetimes need ordering ('b outlives 'a). Complex lifetime relationships unreadable in function signature. Trait with associated types needs lifetime bounds. Generic functions returning references need T: 'a but unclear why. Before Rust 2018 required explicit T: 'a everywhere.

**Solution**: Lifetime bounds `T: 'a` mean "T must outlive lifetime 'a". Outlives constraint `'b: 'a` means "'b must outlive 'a". Modern Rust infers `T: 'a` from context (using `&'a T` implies bound). Where clauses for complex relationships: `where 'b: 'a, T: Debug + 'a`. Trait bounds combined with lifetime bounds. Associated types with lifetime constraints. Lifetime hierarchy expressed via bounds.

**Why It Matters**: Ensures references in generic types remain valid—can't have dangling reference in generic wrapper. Establishes lifetime hierarchies: 'long ⊇ 'medium ⊇ 'short. Enables flexible generic APIs: cache holding references, parser with borrowed input. Compiler infers most bounds (since Rust 2018): less annotation noise. Type system prevents lifetime bugs in generic code. Where clauses improve readability for complex constraints.

**Use Cases**: Generic structs with references (Wrapper<'a, T>), cache implementations (storing borrowed values), parser types (Parser<'a> with input), complex lifetime relationships (multiple independent lifetimes), trait definitions with lifetime constraints, generic functions returning references, collection types with borrowed content, builder patterns with references.

```rust
// T must outlive 'a
struct Wrapper<'a, T: 'a> {
    value: &'a T,
}

// Modern syntax (implicit bound)
struct Wrapper<'a, T> {
    value: &'a T,
}
// The bound T: 'a is implied when we use &'a T
```

Before Rust 2018, you had to write `T: 'a` explicitly. Now the compiler infers it from context.

### Where Clauses for Lifetimes

Complex lifetime constraints use where clauses:

```rust
fn complex_function<'a, 'b, T>(x: &'a T, y: &'b T) -> &'a T
where
    'b: 'a, // 'b outlives 'a
    T: std::fmt::Debug,
{
    println!("x: {:?}, y: {:?}", x, y);
    x
}
```

The bound `'b: 'a` means "'b lives at least as long as 'a." This is the lifetime equivalent of a trait bound.

### Lifetime Bounds in Trait Definitions

Traits can require lifetime bounds:

```rust
trait Cache<'a> {
    type Item: 'a; // Associated type must outlive 'a

    fn get(&self, key: &str) -> Option<&'a Self::Item>;
    fn insert(&mut self, key: String, value: Self::Item);
}

struct SimpleCache<'a, T: 'a> {
    items: std::collections::HashMap<String, &'a T>,
}

impl<'a, T: 'a> Cache<'a> for SimpleCache<'a, T> {
    type Item = T;

    fn get(&self, key: &str) -> Option<&'a T> {
        self.items.get(key).copied()
    }

    fn insert(&mut self, key: String, value: &'a T) {
        self.items.insert(key, value);
    }
}
```

The bound `T: 'a` ensures cached references remain valid for the cache's lifetime.

### Combining Lifetime and Trait Bounds

Real-world code often combines both:

```rust
fn process<'a, T>(items: &'a [T]) -> Vec<&'a T>
where
    T: PartialOrd + std::fmt::Debug + 'a,
{
    items
        .iter()
        .filter(|item| {
            println!("Processing: {:?}", item);
            true
        })
        .collect()
}
```

The `'a` bound on `T` allows returning references to `T` with lifetime `'a`.

### Lifetime Bounds in Structs

Structs with references need careful lifetime management:

```rust
struct Parser<'a, T: 'a> {
    input: &'a str,
    state: T,
}

impl<'a, T: 'a> Parser<'a, T> {
    fn new(input: &'a str, state: T) -> Self {
        Parser { input, state }
    }

    fn parse(&mut self) -> Option<&'a str>
    where
        T: Default,
    {
        // Parse logic using self.input and self.state
        Some(self.input)
    }
}
```

### Multiple Lifetime Bounds

You can express complex lifetime relationships:

```rust
struct MultiRef<'a, 'b, 'c> {
    short: &'a str,
    medium: &'b str,
    long: &'c str,
}

impl<'a, 'b, 'c> MultiRef<'a, 'b, 'c>
where
    'c: 'b,  // 'c outlives 'b
    'b: 'a,  // 'b outlives 'a
{
    fn new(long: &'c str, medium: &'b str, short: &'a str) -> Self {
        MultiRef { short, medium, long }
    }

    // Can return long reference as medium or short
    fn long_as_medium(&self) -> &'b str {
        self.long // OK because 'c: 'b
    }

    fn long_as_short(&self) -> &'a str {
        self.long // OK because 'c: 'b: 'a
    }
}
```

The bounds establish a lifetime hierarchy: `'c` ⊇ `'b` ⊇ `'a`.

## Pattern 3: Higher-Ranked Trait Bounds (HRTB)

**Problem**: Closures accepting references need "for any lifetime"—can't express `Fn(&'??? str) → String` where lifetime varies per call. Lifetime parameter scope unclear: does 'a apply to closure or call site? Function traits (Fn, FnMut, FnOnce) with reference parameters need polymorphism. Iterator adapters like `map` take closures with borrowed input. Generic function accepting closure with any reference lifetime. Can't write trait bound for "closure works with any lifetime".

**Solution**: `for<'a>` syntax means "for all lifetimes 'a"—higher-ranked trait bound. `F: for<'a> Fn(&'a str) → String` means F works with any lifetime. HRTB in trait bounds enables lifetime-polymorphic closures. Closure can be called with references of any lifetime. Iterator combinators use HRTB for flexibility. Function trait bounds become universally quantified. Compiler infers HRTB in common closure cases.

**Why It Matters**: Enables lifetime-generic closures: same closure works with short or long references. Function traits work with any reference: no single lifetime restriction. Essential for iterator adapters: `map(|x: &str| x.len())` works with any iterator lifetime. Higher-order functions become flexible: one function parameter type works with all lifetimes. Rust's functional programming patterns rely on HRTB. Without HRTB, closures overly restrictive.

**Use Cases**: Closure parameters (accepting functions with reference inputs), function trait bounds (generic over Fn/FnMut/FnOnce), iterator combinators (map/filter/flat_map with borrowed data), higher-order functions (functions returning closures), generic callbacks (event handlers with reference payloads), parser combinators (parsers consuming borrowed input), stream processing (transformations on borrowed items).

```rust
// This doesn't work!
fn apply_to_string<F>(f: F) -> String
where
    F: Fn(&str) -> String,
{
    f("hello")
}
```

What's the lifetime of the `&str` parameter? The closure should work with *any* lifetime, not just a specific one. This is where HRTBs come in:

```rust
// Correct: works for all lifetimes
fn apply_to_string<F>(f: F) -> String
where
    F: for<'a> Fn(&'a str) -> String,
{
    f("hello")
}

fn example() {
    let result = apply_to_string(|s| s.to_uppercase());
    println!("{}", result); // "HELLO"
}
```

The `for<'a>` syntax means "for any lifetime 'a, F implements Fn(&'a str) -> String." The closure must work with any possible lifetime.

### Understanding for<'a> Syntax

The `for<'a>` bound is a universal quantifier:

```rust
// Without HRTB (doesn't work for closures)
fn call_with_ref<'a, F>(f: F)
where
    F: Fn(&'a str) -> usize,
{
    // Specific lifetime 'a
}

// With HRTB (works for closures)
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> usize,
{
    // Any lifetime 'a
}
```

The second version is more flexible—it accepts closures that work with any lifetime.

### Common HRTB Patterns

HRTBs appear frequently with function traits:

```rust
// Function that accepts a closure working with any string reference
fn process_strings<F>(items: Vec<String>, f: F) -> Vec<usize>
where
    F: for<'a> Fn(&'a str) -> usize,
{
    items.iter().map(|s| f(s)).collect()
}

fn example() {
    let strings = vec!["hello".to_string(), "world".to_string()];

    // Closure works with any lifetime
    let lengths = process_strings(strings, |s| s.len());

    println!("{:?}", lengths); // [5, 5]
}
```

### HRTBs with Multiple Lifetimes

You can quantify over multiple lifetimes:

```rust
trait Comparator {
    fn compare<'a, 'b>(&self, x: &'a str, y: &'b str) -> std::cmp::Ordering;
}

fn sort_with<C>(items: &mut [String], comparator: C)
where
    C: for<'a, 'b> Fn(&'a str, &'b str) -> std::cmp::Ordering,
{
    items.sort_by(|a, b| comparator(a, b));
}

fn example() {
    let mut items = vec!["zebra".to_string(), "apple".to_string(), "mango".to_string()];

    sort_with(&mut items, |a, b| a.cmp(b));

    println!("{:?}", items); // ["apple", "mango", "zebra"]
}
```

### When You Need HRTBs

You need HRTBs when:

1. **Working with function traits and closures**

```rust
fn map_strs<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> String,
{
    // ...
}
```

2. **Implementing traits with lifetime parameters**

```rust
trait Parser {
    fn parse<'a>(&self, input: &'a str) -> Option<&'a str>;
}

fn use_parser<P>(parser: P)
where
    P: for<'a> Parser<Input<'a> = &'a str>,
{
    // ...
}
```

3. **Building flexible APIs**

```rust
struct EventHandler<F> {
    handler: F,
}

impl<F> EventHandler<F>
where
    F: for<'a> Fn(&'a Event) -> (),
{
    fn handle(&self, event: &Event) {
        (self.handler)(event);
    }
}
```

### Implicit HRTBs

The compiler often infers HRTBs for function traits:

```rust
// These are equivalent:
fn example1<F: Fn(&str) -> String>(f: F) { }
fn example2<F: for<'a> Fn(&'a str) -> String>(f: F) { }

// Compiler adds the HRTB automatically
```

However, making it explicit can improve clarity and error messages.

## Pattern 4: Self-Referential Structs and Pin

**Problem**: Struct referencing its own data impossible—can't create reference to field before field exists. Moving self-referential struct invalidates internal pointers. Borrow checker rejects self-references. Async futures need self-referential state (holding reference to local). Intrusive data structures (node references within node) unsound if movable. Generator state machines self-referential. Arena-allocated graphs need self-edges.

**Solution**: Pin<T> makes value unmovable—pinned value can't be moved, enabling safe self-references. Self-referential types via unsafe + Pin guarantees. Async runtime uses Pin<Box<Future>> for self-referential futures. Pin projection for accessing fields safely. Alternative: indices instead of references (Vec<Node> with usize indices). Alternative: arena allocation with stable addresses. Pin::new_unchecked for unsafe pinning. Unpin marker trait for normally-movable types.

**Why It Matters**: Enables async/await—futures self-referential across await points, Pin makes it safe. Zero-cost futures: self-referential without heap per await. Intrusive data structures possible: linked lists with pointers within nodes. Generators work: yield suspends with references to locals. Pin API is sound: compiler enforces pinning invariants. Alternative patterns (indices, arena) trade ergonomics for safety. Understanding Pin crucial for async Rust.

**Use Cases**: Async futures (Pin<Box<dyn Future>>), generators (self-referential state across yields), intrusive linked lists (node contains next pointer), self-referential iterators (iterator holding reference to own buffer), arena-allocated graphs (nodes reference each other), streaming protocols (buffer references within protocol state), zero-copy parsers (parser holds reference to own memory).

```rust
struct SelfReferential {
    data: String,
    reference: &str, // Error! Missing lifetime
}

// Even with lifetimes, this doesn't work:
struct SelfReferential<'a> {
    data: String,
    reference: &'a str,
}

// Can't implement this safely!
impl<'a> SelfReferential<'a> {
    fn new(s: String) -> Self {
        SelfReferential {
            data: s,
            reference: &s, // Error! s moved into data
        }
    }
}
```

The problem: you can't create a reference to `data` before `data` exists, but you can't create `data` after creating the reference.

### Why Self-References Are Dangerous

Moving a self-referential struct breaks the references:

```rust
// Hypothetical self-referential struct
let mut s = SelfReferential::new(String::from("hello"));
let s2 = s; // Move!

// s.reference now points to moved data
// This would be use-after-free!
```

Rust's ownership model prevents this entire class of bugs by disallowing self-references.

### Solution 1: Indices Instead of References

Use indices into a Vec instead of references:

```rust
struct Node {
    data: String,
    next: Option<usize>, // Index, not reference
}

struct LinkedList {
    nodes: Vec<Node>,
    head: Option<usize>,
}

impl LinkedList {
    fn new() -> Self {
        LinkedList {
            nodes: Vec::new(),
            head: None,
        }
    }

    fn push(&mut self, data: String) {
        let index = self.nodes.len();
        let node = Node {
            data,
            next: self.head,
        };
        self.nodes.push(node);
        self.head = Some(index);
    }
}
```

This is safe because indices remain valid when the Vec is moved.

### Solution 2: Pin and Unsafe

For truly self-referential structures, use `Pin`:

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    // Raw pointer instead of reference
    reference: *const String,
    _pin: PhantomPinned,
}

impl SelfReferential {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfReferential {
            data: s,
            reference: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // Safe because we pinned first
        let self_ptr: *const String = &boxed.data;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).reference = self_ptr;
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

Libraries like `ouroboros` provide safe abstractions:

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

Often, self-references indicate a design problem. Consider alternatives:

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
```

This separates ownership from borrowing, eliminating the self-reference.

### When Self-References Are Actually Needed

Rare cases that truly need self-references:

1. **Async runtimes**: Futures containing references to their own data
2. **Parsers**: Holding both input buffer and views into it
3. **Game engines**: Scene graphs with parent-child relationships

For these, use `Pin`, arena allocation, or specialized crates.

## Pattern 5: Variance and Subtyping

**Problem**: Lifetime subtyping rules unclear—when can `&'long` be used where `&'short` expected? Covariant vs contravariant vs invariant confusing. Can't tell if `Wrapper<'a>` accepts longer lifetimes. Function pointer lifetimes behave differently. Mutable references invariant but immutable covariant. PhantomData variance unclear. Generic type variance determined by fields. Soundness holes if variance wrong.

**Solution**: Covariant types accept longer lifetimes: `&'a T` is covariant in 'a, so `&'long T` usable where `&'short T` expected. Invariant types require exact lifetime: `&mut 'a T` is invariant—can't substitute. Contravariant rare: function argument positions. Variance rules: `&'a T` covariant, `&mut 'a T` invariant, `fn(&'a T)` contravariant in 'a, `Cell<&'a T>` invariant. PhantomData<&'a T> adds covariance. Compiler infers variance from structure. Longer lifetime ('static) is subtype of shorter.

**Why It Matters**: Enables flexible lifetime assignments—longer lifetime works where shorter needed. Immutable reference covariance allows ergonomic code: can pass long-lived reference to function expecting short. Mutable reference invariance prevents soundness holes: can't substitute lifetimes arbitrarily to break safety. Variance determines what lifetime conversions are safe. Understanding variance explains compiler errors about lifetime mismatches. PhantomData controls generic type variance for custom pointer types.

**Use Cases**: Reference wrappers (determining if Wrapper<'a> covariant), iterator chains (covariant iterators compose naturally), function pointers (contravariant arguments, covariant returns), trait objects (variance of dyn Trait<'a>), smart pointers with references (Arc<&'a T> variance), custom pointer types (controlling subtyping behavior with PhantomData), phantom data usage (adding variance markers to generic types).

```rust
fn example() {
    let outer: &'static str = "hello";

    {
        let inner: &str = outer; // OK: 'static is subtype of shorter lifetime
    }
}
```

If `'long: 'short` (read: "'long outlives 'short"), then `'long` is a subtype of `'short`. You can use a longer lifetime where a shorter one is expected.

### Variance Categories

Types have variance with respect to their lifetime and type parameters:

**Covariant**: Subtyping flows in the same direction

```rust
// &'a T is covariant over 'a
// If 'a: 'b, then &'a T <: &'b T

fn covariant_example() {
    let long: &'static str = "hello";
    let short: &str = long; // OK
}
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

Variance determines when types are compatible:

```rust
// Covariance allows this:
fn take_short(x: &str) {}

fn example() {
    let s: &'static str = "hello";
    take_short(s); // OK: can pass 'static where shorter lifetime expected
}

// Invariance prevents this:
fn swap<'a, 'b>(x: &'a mut &'static str, y: &'b mut &'a str) {
    // std::mem::swap(x, y); // Error! Invariance prevents swapping
}
```

### Variance in Practice

Common types and their variance:

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

fn example() {
    // Can use Producer<&'static str> where Producer<&'a str> expected
    let p: Producer<&'static str> = Producer { produce: || "hello" };
    let _p2: Producer<&str> = p; // OK

    // Contravariance with Consumer
    let c: Consumer<&str> = Consumer { consume: |_s| {} };
    // let _c2: Consumer<&'static str> = c; // Would be error if uncommented
}
```

### Interior Mutability and Invariance

Interior mutability types are invariant:

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

Control variance explicitly with `PhantomData`:

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
```

Use `PhantomData` when you need to control variance without storing the actual type.

### Subtyping and Higher-Rank Trait Bounds

HRTBs interact with variance:

```rust
// Works because of variance
fn accepts_any_lifetime<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> String,
{
    f("hello");
}

fn example() {
    // This closure works with any lifetime
    accepts_any_lifetime(|s: &str| s.to_uppercase());
}
```

The HRTB ensures the function works with any lifetime, leveraging variance.

### Lifetime Subtyping in APIs

Design APIs to work with variance:

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

// Cannot substitute different lifetimes
```

Prefer immutable references for flexibility unless mutation is necessary.

## Pattern 6: Advanced Lifetime Patterns

Let's explore some sophisticated patterns that combine these concepts.

### Lifetime Bounds with Closures

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
```

The closure receives references tied to the input lifetimes.

### Streaming Iterators

Iterator items that borrow from the iterator itself:

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
            let window = &self.data[self.position..self.position + self.window_size];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}
```

This pattern allows iterators to yield references tied to the iterator's lifetime.

### Lifetime Elision in Impl Blocks

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
```

### Anonymous Lifetimes

Use `'_` for clarity without naming:

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
```

Anonymous lifetimes improve readability when the specific lifetime name doesn't matter.

## Summary

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


## Lifetime Cheat Sheet

```rust
// ===== BASIC LIFETIMES =====
// Lifetime annotations tell the compiler how long references are valid

// Function with lifetime annotation
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn basic_lifetime_example() {
    let string1 = String::from("long string");
    let string2 = String::from("short");
    
    let result = longest(&string1, &string2);
    println!("Longest: {}", result);
}

// Multiple lifetime parameters
fn first_word<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x.split_whitespace().next().unwrap_or("")
}

// ===== LIFETIME IN STRUCTS =====
// Struct that holds references
struct ImportantExcerpt<'a> {
    part: &'a str,
}

impl<'a> ImportantExcerpt<'a> {
    fn level(&self) -> i32 {
        3
    }
    
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention: {}", announcement);
        self.part
    }
}

fn struct_lifetime_example() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().expect("Could not find '.'");
    
    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };
    
    println!("Excerpt: {}", excerpt.part);
}

// Multiple lifetimes in struct
struct Context<'a, 'b> {
    name: &'a str,
    config: &'b Config,
}

struct Config {
    timeout: u64,
}

// ===== LIFETIME ELISION RULES =====
// Compiler can infer lifetimes in certain cases

// Rule 1: Each parameter gets its own lifetime
fn simple(s: &str) -> &str {
    // Equivalent to: fn simple<'a>(s: &'a str) -> &'a str
    s
}

// Rule 2: If one input lifetime, it's assigned to all outputs
fn first_char(s: &str) -> &str {
    &s[0..1]
}

// Rule 3: If &self or &mut self, lifetime of self is assigned to outputs
impl<'a> ImportantExcerpt<'a> {
    fn get_part(&self) -> &str {
        // Return type gets lifetime of &self
        self.part
    }
}

// ===== STATIC LIFETIME =====
// 'static means the reference lives for the entire program

fn static_lifetime_example() {
    let s: &'static str = "I have a static lifetime";
    
    // String literals always have 'static lifetime
    let literal = "Hello, world!";
    
    // Leaked memory also has 'static lifetime
    let leaked: &'static str = Box::leak(Box::new(String::from("leaked")));
}

// Function requiring static lifetime
fn needs_static(s: &'static str) {
    println!("{}", s);
}

// ===== LIFETIME BOUNDS =====
// Constrain type parameter lifetimes

// T must live at least as long as 'a
fn print_ref<'a, T>(value: &'a T)
where
    T: std::fmt::Display + 'a,
{
    println!("{}", value);
}

// Struct with lifetime bound
struct Ref<'a, T: 'a> {
    value: &'a T,
}

// Generic lifetime bound
impl<'a, T> Ref<'a, T>
where
    T: 'a,
{
    fn new(value: &'a T) -> Self {
        Ref { value }
    }
}

// ===== MULTIPLE LIFETIMES =====
// Different parameters can have different lifetimes

struct MultiLife<'a, 'b> {
    first: &'a str,
    second: &'b str,
}

fn multiple_lifetimes<'a, 'b>(x: &'a str, y: &'b str) -> (&'a str, &'b str) {
    (x, y)
}

fn multi_example() {
    let s1 = String::from("first");
    {
        let s2 = String::from("second");
        let (r1, r2) = multiple_lifetimes(&s1, &s2);
        println!("{} {}", r1, r2);
    }
    // r2 is out of scope here
}

// ===== LIFETIME SUBTYPING =====
// 'a: 'b means 'a lives at least as long as 'b

struct Parser<'c, 's> {
    context: &'c str,
    source: &'s str,
}

impl<'c, 's> Parser<'c, 's> {
    fn parse(&self) -> Result<(), &'s str> {
        Ok(())
    }
}

// Lifetime subtyping example
fn subtyping<'a, 'b>(x: &'a str, y: &'b str) -> &'a str
where
    'b: 'a,  // 'b outlives 'a
{
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// ===== HIGHER-RANKED TRAIT BOUNDS (HRTB) =====
// for<'a> syntax for lifetime bounds that work for any lifetime

trait DoSomething {
    fn do_it(&self, data: &str);
}

// Function that works for any lifetime
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str),
{
    let s = String::from("temporary");
    f(&s);
}

fn hrtb_example() {
    call_with_ref(|s| {
        println!("Got: {}", s);
    });
}

// ===== LIFETIME IN CLOSURES =====
fn closure_lifetime_example() {
    let s = String::from("hello");
    
    // Closure borrowing
    let print = || println!("{}", s);
    print();
    println!("{}", s); // s still valid
    
    // Closure taking ownership
    let consume = move || println!("{}", s);
    consume();
    // println!("{}", s); // ERROR: s moved
}

// Function returning closure with lifetime
fn make_adder<'a>(x: &'a i32) -> impl Fn(i32) -> i32 + 'a {
    move |y| x + y
}

// ===== LIFETIME IN ITERATORS =====
struct WordIterator<'a> {
    text: &'a str,
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }
        
        match self.text.find(' ') {
            Some(pos) => {
                let word = &self.text[..pos];
                self.text = &self.text[pos + 1..];
                Some(word)
            }
            None => {
                let word = self.text;
                self.text = "";
                Some(word)
            }
        }
    }
}

fn iterator_lifetime_example() {
    let text = String::from("hello world rust");
    let iter = WordIterator { text: &text };
    
    for word in iter {
        println!("{}", word);
    }
}

// ===== SELF-REFERENTIAL STRUCTS =====
// Cannot directly create self-referential structs in safe Rust

// This won't work:
// struct SelfRef<'a> {
//     data: String,
//     reference: &'a str, // Cannot reference 'data' field
// }

// Solution 1: Use indices instead of references
struct SafeSelfRef {
    data: String,
    start: usize,
    end: usize,
}

impl SafeSelfRef {
    fn new(data: String, start: usize, end: usize) -> Self {
        SafeSelfRef { data, start, end }
    }
    
    fn get_slice(&self) -> &str {
        &self.data[self.start..self.end]
    }
}

// Solution 2: Use Pin for self-referential structs (advanced)
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfReferential {
            data,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });
        
        let ptr = &boxed.data as *const String;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).ptr = ptr;
        }
        
        boxed
    }
    
    fn get_data(&self) -> &str {
        unsafe { &*self.ptr }
    }
}

// ===== COMMON LIFETIME PATTERNS =====

// Pattern 1: Returning references from struct methods
struct Container<'a> {
    data: &'a [i32],
}

impl<'a> Container<'a> {
    fn first(&self) -> Option<&i32> {
        self.data.first()
    }
    
    fn last(&self) -> Option<&i32> {
        self.data.last()
    }
}

// Pattern 2: Splitting references
fn split_at<'a>(s: &'a str, mid: usize) -> (&'a str, &'a str) {
    (&s[..mid], &s[mid..])
}

// Pattern 3: Caching/memoization with lifetimes
struct Cache<'a, T> {
    value: Option<T>,
    generator: &'a dyn Fn() -> T,
}

impl<'a, T> Cache<'a, T> {
    fn new(generator: &'a dyn Fn() -> T) -> Self {
        Cache {
            value: None,
            generator,
        }
    }
    
    fn get(&mut self) -> &T {
        if self.value.is_none() {
            self.value = Some((self.generator)());
        }
        self.value.as_ref().unwrap()
    }
}

// Pattern 4: Builder with references
struct RequestBuilder<'a> {
    method: &'a str,
    url: &'a str,
    headers: Vec<(&'a str, &'a str)>,
}

impl<'a> RequestBuilder<'a> {
    fn new(method: &'a str, url: &'a str) -> Self {
        RequestBuilder {
            method,
            url,
            headers: Vec::new(),
        }
    }
    
    fn header(mut self, key: &'a str, value: &'a str) -> Self {
        self.headers.push((key, value));
        self
    }
}

// ===== LIFETIME TROUBLESHOOTING =====

// Problem: Returning reference to local variable
// fn dangling_reference() -> &str {
//     let s = String::from("hello");
//     &s // ERROR: s doesn't live long enough
// }

// Solution: Return owned value
fn no_dangling() -> String {
    let s = String::from("hello");
    s
}

// Problem: Mutable and immutable borrows
fn borrow_problem() {
    let mut s = String::from("hello");
    
    // let r1 = &s;
    // let r2 = &mut s; // ERROR: cannot borrow as mutable
    // println!("{}", r1);
    
    // Solution: End immutable borrow before mutable
    {
        let r1 = &s;
        println!("{}", r1);
    } // r1 goes out of scope
    
    let r2 = &mut s;
    r2.push_str(" world");
}

// Problem: Lifetime too restrictive
fn restrictive<'a>(x: &'a str, y: &str) -> &'a str {
    // Cannot return y because it doesn't have lifetime 'a
    x
}

// Solution: Add second lifetime parameter
fn flexible<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x
}

// ===== ADVANCED LIFETIME SCENARIOS =====

// Lifetime variance
struct Covariant<'a, T> {
    value: &'a T,
}

struct Contravariant<'a, T> {
    callback: fn(&'a T),
}

struct Invariant<'a, T> {
    value: &'a mut T,
}

// Lifetime with trait objects
trait Speak {
    fn speak(&self);
}

fn speak_twice<'a>(speaker: &'a dyn Speak) {
    speaker.speak();
    speaker.speak();
}

// Lifetime with associated types
trait Producer {
    type Item;
    fn produce(&self) -> Self::Item;
}

struct StringProducer<'a> {
    template: &'a str,
}

impl<'a> Producer for StringProducer<'a> {
    type Item = &'a str;
    
    fn produce(&self) -> Self::Item {
        self.template
    }
}

// ===== LIFETIME WITH GENERIC TYPES =====
struct Holder<'a, T>
where
    T: 'a,
{
    value: &'a T,
}

impl<'a, T> Holder<'a, T>
where
    T: 'a,
{
    fn new(value: &'a T) -> Self {
        Holder { value }
    }
    
    fn get(&self) -> &T {
        self.value
    }
}

// ===== LIFETIME IN ASYNC =====
// Lifetimes in async functions
async fn async_lifetime<'a>(s: &'a str) -> &'a str {
    // Simulate async work
    s
}

// Struct with async method and lifetime
struct AsyncContext<'a> {
    data: &'a str,
}

impl<'a> AsyncContext<'a> {
    async fn process(&self) -> String {
        format!("Processing: {}", self.data)
    }
}

// ===== LIFETIME ANNOTATIONS IN PRACTICE =====

// Real-world example: Parser
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }
    
    fn parse_word(&mut self) -> Option<&'a str> {
        let start = self.pos;
        
        while self.pos < self.input.len() {
            if self.input.as_bytes()[self.pos].is_ascii_whitespace() {
                break;
            }
            self.pos += 1;
        }
        
        if start == self.pos {
            None
        } else {
            let word = &self.input[start..self.pos];
            self.pos += 1; // Skip whitespace
            Some(word)
        }
    }
    
    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }
}

fn parser_example() {
    let text = "hello world rust";
    let mut parser = Parser::new(text);
    
    while let Some(word) = parser.parse_word() {
        println!("Word: {}", word);
    }
}

// Real-world example: String pool
struct StringPool {
    pool: Vec<String>,
}

impl StringPool {
    fn new() -> Self {
        StringPool { pool: Vec::new() }
    }
    
    fn intern(&mut self, s: String) -> &str {
        if !self.pool.iter().any(|existing| existing == &s) {
            self.pool.push(s);
        }
        self.pool.iter().find(|existing| *existing == &s).unwrap()
    }
    
    fn get(&self, index: usize) -> Option<&str> {
        self.pool.get(index).map(|s| s.as_str())
    }
}

// ===== COMMON MISTAKES AND SOLUTIONS =====

// Mistake 1: Trying to return reference to temporary
// fn wrong() -> &str {
//     let temp = String::from("temp");
//     &temp // ERROR
// }

// Solution: Return owned value or use static
fn correct() -> String {
    String::from("temp")
}

// Mistake 2: Lifetime conflicts in method chains
struct Builder<'a> {
    parts: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    fn new() -> Self {
        Builder { parts: Vec::new() }
    }
    
    fn add(mut self, part: &'a str) -> Self {
        self.parts.push(part);
        self
    }
    
    fn build(self) -> String {
        self.parts.join("")
    }
}

fn builder_lifetime_example() {
    let part1 = String::from("hello");
    let part2 = String::from(" world");
    
    let result = Builder::new()
        .add(&part1)
        .add(&part2)
        .build();
    
    println!("{}", result);
}

// Mistake 3: Conflating lifetimes unnecessarily
// Better to be specific when different lifetimes are needed
fn specific_lifetimes<'a, 'b>(
    long_lived: &'a str,
    short_lived: &'b str,
) -> &'a str {
    long_lived
}
```

