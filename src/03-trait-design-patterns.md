# Trait Design Patterns
This chapter explores advanced trait patterns: inheritance and bounds for capabilities, associated types vs generics for API design, trait objects for dynamic dispatch, extension traits for extending external types, and sealed traits for controlled implementation.

## Pattern 1: Trait Inheritance and Bounds

*   **Problem**: Expressing complex capability requirements is unclear—a trait needs `Display` but can't require it directly. Combining multiple capabilities is verbose (`T: Clone + Debug + Display`).
*   **Solution**: Use supertrait relationships (`trait Loggable: Debug`) to express requirements. Use trait bounds in generics (`fn process<T: Clone>`), and `where` clauses for readability.
*   **Why It Matters**: Supertraits create clear capability requirements. Trait bounds allow for powerful composition of abstractions from simple components.

### Example: Super Traits

```rust
//==================================================
// Supertrait relationship: Printable requires Debug
//==================================================
trait Printable: std::fmt::Debug {
    fn print(&self) {
        println!("{:?}", self);
    }
}

//==========================================================
// Any type implementing Printable must also implement Debug
//==========================================================
#[derive(Debug)]
struct Document {
    title: String,
    content: String,
}

impl Printable for Document {}

fn example() {
    let doc = Document {
        title: "Rust Guide".to_string(),
        content: "...".to_string(),
    };
    doc.print(); // Uses Debug implementation
}
```

The supertrait relationship expresses a requirement: "To be Printable, you must first be Debug." This is similar to inheritance in object-oriented languages, but more flexible.

### Example: Multiple Supertraits

Traits can require multiple supertraits, combining different capabilities:

```rust
use std::fmt::{Debug, Display};

//================================
// Requires both Debug and Display
//================================
trait Loggable: Debug + Display {
    fn log(&self) {
        println!("[DEBUG] {:?}", self);
        println!("[INFO] {}", self);
    }
}

#[derive(Debug)]
struct User {
    name: String,
    id: u32,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "User {} (ID: {})", self.name, self.id)
    }
}

impl Loggable for User {}

fn use_loggable<T: Loggable>(item: &T) {
    item.log();
}
```

This pattern is useful when your abstraction needs multiple orthogonal capabilities. The `Loggable` trait doesn't need to know *how* to debug or display items—it just requires that the capability exists.

### Example: Trait Bounds in Generic Functions

Trait bounds specify what capabilities a generic type must have:

```rust
//=============
// Simple bound
//=============
fn print_item<T: std::fmt::Display>(item: T) {
    println!("{}", item);
}

//================
// Multiple bounds
//================
fn process<T: Clone + std::fmt::Debug>(item: T) {
    let copy = item.clone();
    println!("Processing: {:?}", copy);
}

//=============================
// Where clause for readability
//=============================
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: std::fmt::Debug + Clone,
    U: std::fmt::Display + Default,
{
    format!("{:?} and {}", t, u)
}
```

The `where` clause improves readability when you have many bounds or complex constraints. It's especially useful in traits and impl blocks:

```rust
trait DataProcessor {
    fn process<T>(&self, data: T) -> String
    where
        T: serde::Serialize + std::fmt::Debug;
}
```

### Example: Conditional Implementation with Trait Bounds

You can implement traits conditionally based on what traits the type parameters implement:

```rust
struct Wrapper<T>(T);

//===================================
// Only implement Clone if T is Clone
//===================================
impl<T: Clone> Clone for Wrapper<T> {
    fn clone(&self) -> Self {
        Wrapper(self.0.clone())
    }
}

//===================================
// Only implement Debug if T is Debug
//===================================
impl<T: std::fmt::Debug> std::fmt::Debug for Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrapper({:?})", self.0)
    }
}
```

This pattern allows `Wrapper<String>` to be `Clone` and `Debug`, while `Wrapper<Rc<RefCell<i32>>>` is only `Clone` (because `RefCell` isn't `Debug` in a useful way). The compiler automatically determines which implementations apply.

### Example: Trait Bound Patterns

Several common patterns emerge when working with trait bounds:

```rust
//==================================
// Builder pattern with trait bounds
//==================================
struct Query<T> {
    data: T,
}

impl<T> Query<T> {
    fn new(data: T) -> Self {
        Query { data }
    }
}

impl<T: Clone> Query<T> {
    // Only available if T is Clone
    fn duplicate(&self) -> Self {
        Query {
            data: self.data.clone(),
        }
    }
}

impl<T: serde::Serialize> Query<T> {
    // Only available if T is Serialize
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.data)
    }
}

//=============================================
// Higher-rank trait bounds (for all lifetimes)
//=============================================
fn process_with_lifetime<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let result = f("hello");
    println!("{}", result);
}
```

The builder pattern becomes particularly powerful with conditional trait implementations, as methods only appear when the type parameter supports them.

## Pattern 2: Associated Types vs Generics

*   **Problem**: A generic trait like `Parser<Output>` allows a single type to have multiple implementations (e.g., for different `Output` types), which can be confusing. Call sites become verbose (`parser.parse::<serde_json::Value>()`), and it's unclear if a type parameter is an "input" or an "output".
*   **Solution**: Use **associated types** when an implementing type determines a single, specific "output" type (`trait Parser { type Output; }`). Use **generics** when the caller chooses an "input" type and multiple implementations are desirable (`trait From<T>`).
*   **Why It Matters**: Associated types lead to more ergonomic APIs, as the compiler can infer the output type (`parser.parse()` is clean). This prevents ambiguity and simplifies trait bounds.

### Example: Generics
```rust
//=================================================
// With generics: Multiple implementations possible
//=================================================
trait Parser<Output> {
    fn parse(&self, input: &str) -> Result<Output, String>;
}

struct JsonParser;

impl Parser<serde_json::Value> for JsonParser {
    fn parse(&self, input: &str) -> Result<serde_json::Value, String> {
        serde_json::from_str(input).map_err(|e| e.to_string())
    }
}

//=========================================================
// Could also implement Parser<MyCustomType> for JsonParser
//=========================================================
```

With generics, a single type can implement the trait multiple times with different type parameters. Sometimes this is exactly what you want, but often it's confusing.

### Example: Associated Types: One Implementation

Associated types express "there is one specific type for this implementation":

```rust
//========================================================
// With associated types: Only one implementation possible
//========================================================
trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Result<Self::Output, String>;
}

struct JsonParser;

impl Parser for JsonParser {
    type Output = serde_json::Value;

    fn parse(&self, input: &str) -> Result<Self::Output, String> {
        serde_json::from_str(input).map_err(|e| e.to_string())
    }
}

//===================================================================
// Cannot implement Parser again for JsonParser with different Output
//===================================================================
```

Now `JsonParser` has exactly one `Output` type. Users don't need to specify it—the compiler infers it.

### Example: Ergonomics: Associated Types Win for Consumers

Associated types lead to cleaner call sites:

```rust
//=======================
// With generic parameter
//=======================
fn use_generic_parser<T, P: Parser<T>>(parser: P, input: &str) -> T {
    parser.parse(input).unwrap()
}

//======================
// Caller must specify T
//======================
let value: serde_json::Value = use_generic_parser::<serde_json::Value, _>(JsonParser, "{}");

//=====================
// With associated type
//=====================
fn use_associated_parser<P: Parser>(parser: P, input: &str) -> P::Output {
    parser.parse(input).unwrap()
}

//=======================
// Compiler infers Output
//=======================
let value = use_associated_parser(JsonParser, "{}");
```

The associated type version is cleaner because the output type is determined by the parser, not by the caller.

### Example: When to Use Each

**Use generics when:**
- A type might implement the trait multiple times with different type parameters
- The type parameter is an input to the behavior
- You want flexibility at the call site

```rust
//========================================
// Generic: Different conversions possible
//========================================
trait From<T> {
    fn from(value: T) -> Self;
}

//=======================================================
// String can be created from &str, String, Vec<u8>, etc.
//=======================================================
impl From<&str> for String { /* ... */ }
impl From<Vec<u8>> for String { /* ... */ }
```

**Use associated types when:**
- Only one implementation makes sense for a given type
- The associated type is an output of the behavior
- You want simpler API for consumers

```rust
//==================================================
// Associated type: One iterator type per collection
//==================================================
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

//====================================================
// Vec<i32>'s iterator produces i32, not anything else
//====================================================
```

### Example: Combining Both

Sometimes you want both generics and associated types:

```rust
trait Converter<Input> {
    type Output;
    type Error;

    fn convert(&self, input: Input) -> Result<Self::Output, Self::Error>;
}

struct TemperatureConverter;

impl Converter<f64> for TemperatureConverter {
    type Output = f64;
    type Error = String;

    fn convert(&self, celsius: f64) -> Result<f64, String> {
        Ok(celsius * 9.0 / 5.0 + 32.0)
    }
}

//======================================================================
// Could also implement Converter<i32> with different Output/Error types
//======================================================================
```

This pattern gives you flexibility where you need it (the input type can vary) while maintaining clarity where you don't (each `Converter<Input>` implementation has one output type).

### Example: Associated Types with Bounds

Associated types can have trait bounds:

```rust
trait Graph {
    type Node: std::fmt::Display;
    type Edge: Clone;

    fn nodes(&self) -> Vec<Self::Node>;
    fn edges(&self) -> Vec<Self::Edge>;
}

//=======================================
// Implementation must satisfy the bounds
//=======================================
struct SimpleGraph;

impl Graph for SimpleGraph {
    type Node = String; // String implements Display ✓
    type Edge = (usize, usize); // Tuple implements Clone ✓

    fn nodes(&self) -> Vec<String> {
        vec!["A".to_string(), "B".to_string()]
    }

    fn edges(&self) -> Vec<(usize, usize)> {
        vec![(0, 1)]
    }
}
```

This pattern ensures that associated types have the capabilities you need to use them correctly.

### Example: The Iterator Pattern Deep Dive

`Iterator` is the canonical example of associated types done right:

```rust
pub trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;

    // Many provided methods using Self::Item
    fn count(self) -> usize where Self: Sized { /* ... */ }
    fn map<B, F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item) -> B,
    { /* ... */ }
}
```

Why associated type instead of generic?
1. Each iterator produces one type of item
2. The item type is determined by the collection, not chosen by the caller
3. Simpler APIs: `Vec<i32>::iter()` returns iterator of `&i32`, not iterator of some generic `T`

## Pattern 3: Trait Objects and Dynamic Dispatch

*   **Problem**: Static dispatch via generics (`fn foo<T: Trait>`) creates a copy of the function for each concrete type, leading to code bloat. It's also impossible to create a collection of different types that share a behavior, like `Vec<[Circle, Rectangle]>`.
*   **Solution**: Use trait objects (`&dyn Trait`) for dynamic dispatch. This creates a single version of the function that accepts any type implementing the trait, looking up the correct method at runtime via a vtable.
*   **Why It Matters**: Dynamic dispatch results in smaller binary sizes and allows for runtime polymorphism (e.g., plugin systems). This flexibility comes at the small cost of a vtable pointer lookup for each method call, which prevents inlining.

### Example: static dispatch

```rust
fn process<T: Display>(item: T) {
    println!("{}", item);
}

// Compiler generates:
// fn process_i32(item: i32) { println!("{}", item); }
// fn process_String(item: String) { println!("{}", item); }
```

Each call site gets optimized code for that specific type. Fast, but increases binary size (code bloat).

### Example: Dynamic dispatch (trait objects):

```rust
fn process(item: &dyn Display) {
    println!("{}", item);
}

// Single function generated
// Uses vtable lookup to find the right Display implementation at runtime
```

One function handles all types. Smaller binary, but slight runtime cost for the vtable lookup.

### Example: Creating Trait Objects

Trait objects must be behind a pointer (reference, `Box`, `Rc`, `Arc`):

```rust
trait Drawable {
    fn draw(&self);
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

fn draw_all(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();
    }
}

fn example() {
    let shapes: Vec<Box<dyn Drawable>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 10.0, height: 20.0 }),
    ];

    draw_all(&shapes);
}
```

This pattern is powerful: `shapes` can contain different types, all treated uniformly through the `Drawable` interface.

### Example: Object Safety Requirements

Not all traits can be used as trait objects. A trait is "object safe" if:

1. **No generic methods**: Methods cannot have type parameters

```rust
trait NotObjectSafe {
    fn generic_method<T>(&self, value: T); // ✗ Generic method
}

//=================================
// Cannot create &dyn NotObjectSafe
//=================================
```

2. **No `Self: Sized` bound**: The trait can't require `Self` to be sized

```rust
trait NotObjectSafe {
    fn returns_self(self) -> Self; // ✗ Takes self by value, requires Sized
}
```

3. **No associated functions**: Methods must have a `self` receiver

```rust
trait NotObjectSafe {
    fn new() -> Self; // ✗ No self parameter
}
```

The reasoning: when calling a method on a trait object, the compiler doesn't know the concrete type. Generic methods and associated functions need to know the type at compile time.

### Example: Making Traits Object-Safe

You can make traits object-safe with careful design:

```rust
//================
// Not object-safe
//================
trait Repository {
    fn create<T: Serialize>(&self, item: T) -> Result<(), Error>;
}

//====================
// Object-safe version
//====================
trait Repository {
    fn create(&self, item: &dyn Serialize) -> Result<(), Error>;
}

//=========================
// Or split into two traits
//=========================
trait Repository {
    fn create(&self, item: Box<dyn Item>) -> Result<(), Error>;
}

trait Item: Serialize {
    // Specific item methods
}
```

This pattern—accepting trait objects instead of generics—makes the trait object-safe while maintaining flexibility.

### Example: Downcasting Trait Objects

Sometimes you need to convert a trait object back to a concrete type:

```rust
use std::any::Any;

trait Shape: Any {
    fn area(&self) -> f64;

    // Provided method for downcasting
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

fn try_as_circle(shape: &dyn Shape) -> Option<&Circle> {
    shape.as_any().downcast_ref::<Circle>()
}

fn example() {
    let circle = Circle { radius: 5.0 };
    let shape: &dyn Shape = &circle;

    if let Some(circle) = try_as_circle(shape) {
        println!("It's a circle with radius {}", circle.radius);
    }
}
```

This pattern is useful but breaks abstraction—use it sparingly, only when you truly need concrete type information.

### Example: Trait Objects with Lifetime Bounds

Trait objects can have lifetime bounds:

```rust
trait Processor {
    fn process(&self, data: &str) -> String;
}

//===========================
// Trait object with lifetime
//===========================
fn process_data<'a>(processor: &'a dyn Processor, data: &'a str) -> String {
    processor.process(data)
}

//=================================
// Boxed trait object with lifetime
//=================================
struct Handler<'a> {
    processor: Box<dyn Processor + 'a>,
}
```

The `+ 'a` syntax means "the trait object must live at least as long as `'a`". This ensures references in the trait implementation remain valid.

### Example: Costs of Dynamic Dispatch

Dynamic dispatch has small but real costs:

```rust
//================
// Static dispatch
//================
fn static_dispatch<T: Display>(items: &[T]) {
    for item in items {
        println!("{}", item); // Direct function call, inlinable
    }
}

//=================
// Dynamic dispatch
//=================
fn dynamic_dispatch(items: &[&dyn Display]) {
    for item in items {
        println!("{}", item); // Indirect call through vtable
    }
}
```

Benchmarking typical code shows:
- Static dispatch: ~1-2ns per call
- Dynamic dispatch: ~3-5ns per call

The difference is tiny for I/O-bound operations but can matter for tight inner loops. Profile before optimizing.

## Pattern 4: Extension Traits

*   **Problem**: You can't add methods to types from other crates (the "orphan rule"). You want to extend standard types like `Vec` or `String` with domain-specific helpers, but can't modify their source code.
*   **Solution**: Define a new trait (an "extension trait") with the desired methods. Then, implement that trait for the external type.
*   **Why It Matters**: This pattern allows you to extend any type you don't own in a modular, opt-in way. It solves the orphan rule problem cleanly.


### Example: Basic Extension Trait
The orphan rule prevents implementing a foreign trait on a foreign type. However, you can implement your *own* trait on a foreign type. This is the core of the extension trait pattern. Here, we can't add a `sum` method to `Vec` directly, but we can define our own `SumExt` trait and implement it for `Vec<i32>`.

```rust
trait SumExt {
    fn sum_ext(&self) -> i32;
}

impl SumExt for Vec<i32> {
    fn sum_ext(&self) -> i32 {
        self.iter().sum()
    }
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    // By bringing `SumExt` into scope, we "extend" Vec<i32>
    println!("Sum: {}", numbers.sum_ext());
}
```

### Example: Blanket Iterator Extensions
This pattern is extremely powerful. By defining an extension trait `IteratorExt` and providing a blanket implementation for *all* types that implement `Iterator`, we can add new functionality to every iterator in our program.

```rust
use std::collections::HashMap;

trait IteratorExt: Iterator {
    // Count occurrences of each item in an iterator.
    fn counts(self) -> HashMap<Self::Item, usize>
    where
        Self: Sized,
        Self::Item: Eq + std::hash::Hash,
    {
        let mut map = HashMap::new();
        for item in self {
            *map.entry(item).or_insert(0) += 1;
        }
        map
    }
}

// Blanket implementation: this applies to any type that is an Iterator.
impl<I: Iterator> IteratorExt for I {}

fn main() {
    let words = vec!["apple", "banana", "apple", "cherry", "banana", "apple"];
    let counts = words.into_iter().counts();
    println!("{:?}", counts); // {"apple": 3, "banana": 2, "cherry": 1}
}
```

### Example: Ergonomic Error Handling
Extension traits can make error handling more ergonomic by adding context or logging capabilities to the standard `Result` type. This `ResultExt` provides a `log_err` method that logs the error and its context before passing it up the call stack.

```rust
trait ResultExt<T> {
    fn log_err(self, context: &str) -> Self;
}

impl<T, E: std::error::Error> ResultExt<T> for Result<T, E> {
    fn log_err(self, context: &str) -> Self {
        self.map_err(|e| {
            eprintln!("[ERROR] {}: {}", context, e);
            e
        })
    }
}

fn main() {
    let _ = std::fs::read_to_string("non_existent_file.txt")
        .log_err("Failed to read config");
}
```

### Example: Extending Standard Types
You can add domain-specific helper methods to standard library types like `String` and `str` to make your code more expressive.

```rust
trait StringExt {
    fn truncate_to(&self, max_len: usize) -> String;
}

impl<T: AsRef<str>> StringExt for T {
    fn truncate_to(&self, max_len: usize) -> String {
        let s = self.as_ref();
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}

fn main() {
    let long_string = "This is a very long string that needs truncation".to_string();
    let truncated = long_string.truncate_to(20);
    println!("{}", truncated); // "This is a very lo..."
}
```

### Example: Conditional Extensions
An extension can be made conditional on the capabilities of the type it's extending. This `DebugExt` trait is implemented for any type `T` as long as `T` implements `Debug`, giving all debuggable types a handy `debug_print` method.

```rust
trait DebugExt {
    fn debug_print(&self);
}

impl<T: std::fmt::Debug> DebugExt for T {
    fn debug_print(&self) {
        println!("{:?}", self);
    }
}

fn main() {
    let numbers = vec![1, 2, 3];
    numbers.debug_print(); // Works because Vec<i32> implements Debug.

    let name = "Alice";
    name.debug_print(); // Works because &str implements Debug.
}
```

This pattern is incredibly powerful—one implementation provides functionality to infinite types.

## Pattern 5: Sealed Traits

*   **Problem**: As a library author, you want to publish a trait that users can depend on, but you want to prevent them from implementing it themselves. This allows you to add new methods to the trait later without it being a breaking change.
*   **Solution**: Create a private `sealed` module with a public but un-implementable `Sealed` trait. Make your public trait a supertrait of `sealed::Sealed`.
*   **Why It Matters**: This pattern gives you the freedom to evolve your API (e.g., add methods with default implementations to the trait) without breaking downstream users. It's a crucial tool for library authors who need to maintain long-term stability.

### Example: Basic Sealed Trait

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn my_method(&self);

    // Can add new methods without breaking external code
    fn new_method(&self) {
        println!("Default implementation");
    }
}

struct MyType;

impl sealed::Sealed for MyType {}
impl MyTrait for MyType {
    fn my_method(&self) {
        println!("Internal implementation");
    }
}

// External crates can USE MyTrait but cannot IMPLEMENT it
// because they can't access sealed::Sealed
```

This pattern ensures you can evolve the trait API without semver-major version bumps.

### Example: Dependency Injection with Traits

Use traits for testable, flexible architectures:

```rust
trait Database {
    fn get_user(&self, id: i32) -> Option<User>;
    fn save_user(&self, user: &User) -> Result<(), Error>;
}

trait EmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Error>;
}

struct UserService<D, E> {
    database: D,
    email: E,
}

impl<D: Database, E: EmailService> UserService<D, E> {
    fn new(database: D, email: E) -> Self {
        UserService { database, email }
    }

    fn register_user(&self, name: &str, email: &str) -> Result<User, Error> {
        let user = User {
            id: generate_id(),
            name: name.to_string(),
            email: email.to_string(),
        };

        self.database.save_user(&user)?;
        self.email.send_email(email, "Welcome!", "Thanks for signing up")?;

        Ok(user)
    }
}

// Production uses real implementations
// Tests use mocks
```

This pattern makes code testable and modular.

> **See Also**: For blanket implementations (`impl<T: Trait> OtherTrait for T`), see **Chapter 4: Pattern 5 (Blanket Implementations)**. For typestate builder patterns, see **Chapter 5: Pattern 2 (Typestate Pattern)**.

### Summary

This chapter covered trait design patterns for flexible, expressive Rust APIs:

1. **Trait Inheritance and Bounds**: Supertrait relationships, multiple bounds, where clauses, conditional implementations
2. **Associated Types vs Generics**: When to use each, ergonomics vs flexibility, Iterator pattern
3. **Trait Objects and Dynamic Dispatch**: &dyn Trait, object safety, heterogeneous collections
4. **Extension Traits**: Extending external types, modular opt-in design
5. **Sealed Traits**: Preventing external impls, API evolution, safety guarantees

**Key Takeaways**:
- Trait inheritance expresses capabilities: "to be A must be B" is declarative and composable
- Associated types = one impl per type, inferred; generics = multiple impls, explicit choice
- Dynamic dispatch = smaller binary, ~2-3ns overhead; static dispatch = optimized per-type
- Extension traits extend types you don't own via trait + impl
- Sealed traits prevent external impls via private supertrait

**Design Guidelines**:
- Supertraits for capability requirements: `trait Loggable: Debug + Display`
- Associated types when output determined by type, generics when chosen by caller
- Trait objects for heterogeneous collections, generics for performance
- Extension traits for opt-in functionality on external types
- Sealed traits when evolution/safety requires controlled implementations

**Object Safety Rules** (for `&dyn Trait`):
- No generic methods (needs concrete type at compile-time)
- No Self: Sized bound (trait objects are !Sized)
- Must have &self/&mut self receiver (needs object to call)
- No associated functions without self (can't call without type)

**Common Patterns**:
```rust
// Trait inheritance
trait Loggable: Debug + Display {
    fn log(&self) { println!("{:?}", self); }
}

// Associated type (one impl per type)
trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Self::Output;
}

// Generic (multiple impls possible)
trait From<T> {
    fn from(value: T) -> Self;
}

// Trait object (heterogeneous collection)
let shapes: Vec<Box<dyn Drawable>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rectangle { width: 10.0, height: 20.0 }),
];

// Extension trait
trait StringExt {
    fn truncate_to(&self, max_len: usize) -> String;
}
impl StringExt for String { /* ... */ }

// Sealed trait (prevent external impl)
mod sealed { pub trait Sealed {} }
pub trait MyTrait: sealed::Sealed { /* ... */ }
```

> **See Also**: For blanket implementations and generics patterns, see **Chapter 4: Generics & Polymorphism**. For typestate builders, see **Chapter 5: Builder & API Design**.


### Traits Cheat Sheet
```rust
// ===== BASIC TRAITS =====
// Define a trait
trait Summary {
    fn summarize(&self) -> String;
}

// Implement trait for a type
struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

// Use trait
fn print_summary(item: &impl Summary) {
    println!("{}", item.summarize());
}

// ===== DEFAULT IMPLEMENTATIONS =====
trait Greeting {
    fn greet(&self) -> String {
        String::from("Hello!")                               // Default implementation
    }
    
    fn farewell(&self) -> String {
        String::from("Goodbye!")
    }
}

struct Person {
    name: String,
}

impl Greeting for Person {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name)                // Override default
    }
    // farewell() uses default implementation
}

// ===== TRAIT BOUNDS =====
// Trait bound syntax
fn notify<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// impl Trait syntax (sugar for above)
fn notify_impl(item: &impl Summary) {
    println!("{}", item.summarize());
}

// Multiple trait bounds
fn notify_multiple<T: Summary + Clone>(item: &T) {
    println!("{}", item.summarize());
}

// impl Trait with multiple bounds
fn process(item: &impl Summary + Clone) {
    println!("{}", item.summarize());
}

// Where clauses (cleaner for complex bounds)
fn complex<T, U>(t: &T, u: &U) -> String
where
    T: Summary + Clone,
    U: Summary + std::fmt::Debug,
{
    format!("{} - {:?}", t.summarize(), u)
}

// ===== RETURNING TRAITS =====
// Return impl Trait
fn create_summary() -> impl Summary {
    Tweet {
        username: String::from("user"),
        content: String::from("content"),
    }
}

// Cannot return different types with impl Trait
// This won't compile:
// fn create_item(flag: bool) -> impl Summary {
//     if flag {
//         Article { ... }
//     } else {
//         Tweet { ... }  // ERROR: different types
//     }
// }

// ===== TRAIT OBJECTS =====
// Box<dyn Trait> for dynamic dispatch
fn create_boxed(flag: bool) -> Box<dyn Summary> {
    if flag {
        Box::new(Article {
            title: String::from("Title"),
            author: String::from("Author"),
            content: String::from("Content"),
        })
    } else {
        Box::new(Tweet {
            username: String::from("user"),
            content: String::from("tweet"),
        })
    }
}

// Collection of different types
fn mixed_collection() {
    let items: Vec<Box<dyn Summary>> = vec![
        Box::new(Article {
            title: String::from("News"),
            author: String::from("John"),
            content: String::from("..."),
        }),
        Box::new(Tweet {
            username: String::from("alice"),
            content: String::from("Hello"),
        }),
    ];
    
    for item in items {
        println!("{}", item.summarize());
    }
}

// Reference to trait object
fn use_trait_object(item: &dyn Summary) {
    println!("{}", item.summarize());
}

// ===== ASSOCIATED TYPES =====
trait Iterator {
    type Item;                                               // Associated type
    
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < 6 {
            Some(self.count)
        } else {
            None
        }
    }
}

// Generic trait (alternative to associated types)
trait GenericIterator<T> {
    fn next(&mut self) -> Option<T>;
}

// ===== TRAIT INHERITANCE (SUPERTRAITS) =====
trait Printable {
    fn print(&self);
}

trait DisplayWithColor: Printable {                          // Requires Printable
    fn print_colored(&self);
}

struct ColoredText {
    text: String,
    color: String,
}

impl Printable for ColoredText {
    fn print(&self) {
        println!("{}", self.text);
    }
}

impl DisplayWithColor for ColoredText {
    fn print_colored(&self) {
        println!("\x1b[{}m{}\x1b[0m", self.color, self.text);
    }
}

// Multiple supertraits
trait Advanced: Printable + Clone + std::fmt::Debug {
    fn advanced_operation(&self);
}

// ===== MARKER TRAITS =====
// Empty traits used for type constraints
trait Marker {}

struct MyType;
impl Marker for MyType {}

fn requires_marker<T: Marker>(item: T) {
    // Function only accepts types that implement Marker
}

// Standard marker traits:
// Send - can be transferred across thread boundaries
// Sync - can be referenced from multiple threads
// Copy - bitwise copyable
// Sized - has known size at compile time
// Unpin - can be moved after pinning

// ===== OPERATOR OVERLOADING =====
use std::ops::{Add, Sub, Mul, Div, Neg, Index};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

// Addition
impl Add for Point {
    type Output = Point;
    
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

// Subtraction
impl Sub for Point {
    type Output = Point;
    
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

// Negation
impl Neg for Point {
    type Output = Point;
    
    fn neg(self) -> Point {
        Point {
            x: -self.x,
            y: -self.y,
        }
    }
}

fn operator_example() {
    let p1 = Point { x: 1, y: 2 };
    let p2 = Point { x: 3, y: 4 };
    
    let sum = p1 + p2;                                       // Point { x: 4, y: 6 }
    let diff = p1 - p2;                                      // Point { x: -2, y: -2 }
    let neg = -p1;                                           // Point { x: -1, y: -2 }
}

// ===== CONVERSION TRAITS =====
// From trait
impl From<(i32, i32)> for Point {
    fn from(tuple: (i32, i32)) -> Self {
        Point { x: tuple.0, y: tuple.1 }
    }
}

// Into is automatically implemented when From is implemented
fn conversion_example() {
    let p: Point = (1, 2).into();                            // Using Into
    let p = Point::from((3, 4));                             // Using From
}

// TryFrom for fallible conversions
use std::convert::TryFrom;

impl TryFrom<(i32, i32)> for Point {
    type Error = String;
    
    fn try_from(tuple: (i32, i32)) -> Result<Self, Self::Error> {
        if tuple.0 >= 0 && tuple.1 >= 0 {
            Ok(Point { x: tuple.0, y: tuple.1 })
        } else {
            Err("Coordinates must be non-negative".to_string())
        }
    }
}

// AsRef and AsMut
impl AsRef<[i32]> for Point {
    fn as_ref(&self) -> &[i32] {
        std::slice::from_ref(&self.x)
    }
}

// ===== DISPLAY AND DEBUG TRAITS =====
use std::fmt;

// Display for user-facing output
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Debug for programmer-facing output (often derived)
impl fmt::Debug for Article {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Article")
            .field("title", &self.title)
            .field("author", &self.author)
            .finish()
    }
}

// ===== CLONE AND COPY =====
// Clone trait
#[derive(Clone)]
struct ExpensiveData {
    data: Vec<i32>,
}

impl Clone for ExpensiveData {
    fn clone(&self) -> Self {
        println!("Cloning expensive data");
        ExpensiveData {
            data: self.data.clone(),
        }
    }
}

// Copy trait (requires Clone)
#[derive(Clone, Copy)]
struct Lightweight {
    value: i32,
}

// ===== DROP TRAIT =====
struct CustomDrop {
    data: String,
}

impl Drop for CustomDrop {
    fn drop(&mut self) {
        println!("Dropping CustomDrop with data: {}", self.data);
    }
}

// ===== DEFAULT TRAIT =====
#[derive(Default)]
struct Config {
    timeout: u32,
    retries: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            timeout: 30,
            retries: 3,
        }
    }
}

fn default_example() {
    let config = Config::default();
    let config: Config = Default::default();
}

// ===== DEREF AND DEREFMUT =====
use std::ops::{Deref, DerefMut};

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn deref_example() {
    let x = MyBox(5);
    let y = *x;                                              // Deref coercion
}

// ===== PARTIAL AND TOTAL ORDERING =====
use std::cmp::{PartialOrd, Ord, Ordering};

#[derive(PartialEq, Eq)]
struct User {
    name: String,
    age: u32,
}

impl PartialOrd for User {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for User {
    fn cmp(&self, other: &Self) -> Ordering {
        self.age.cmp(&other.age)
            .then_with(|| self.name.cmp(&other.name))
    }
}

// ===== HASH TRAIT =====
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(PartialEq, Eq)]
struct Coordinate {
    x: i32,
    y: i32,
}

impl Hash for Coordinate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

// ===== INDEX TRAIT =====
struct Matrix {
    data: Vec<Vec<i32>>,
}

impl Index<(usize, usize)> for Matrix {
    type Output = i32;
    
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0][index.1]
    }
}

fn index_example() {
    let matrix = Matrix {
        data: vec![vec![1, 2], vec![3, 4]],
    };
    
    let value = matrix[(0, 1)];                              // Uses Index trait
}

// ===== ITERATOR TRAIT =====
struct Fibonacci {
    curr: u32,
    next: u32,
}

impl Iterator for Fibonacci {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        self.curr = self.next;
        self.next = current + self.next;
        Some(current)
    }
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}

fn iterator_example() {
    let fib = Fibonacci::new();
    for num in fib.take(10) {
        println!("{}", num);
    }
}

// ===== CUSTOM TRAITS =====
// Trait with multiple methods
trait Drawable {
    fn draw(&self);
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
    
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    
    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

// Trait with associated constants
trait MathConstants {
    const PI: f64 = 3.14159265359;
    const E: f64 = 2.71828182846;
}

// ===== EXTENSION TRAITS =====
// Add methods to existing types
trait StringExt {
    fn is_palindrome(&self) -> bool;
}

impl StringExt for String {
    fn is_palindrome(&self) -> bool {
        let chars: Vec<char> = self.chars().collect();
        chars.iter().eq(chars.iter().rev())
    }
}

impl StringExt for str {
    fn is_palindrome(&self) -> bool {
        let chars: Vec<char> = self.chars().collect();
        chars.iter().eq(chars.iter().rev())
    }
}

fn extension_example() {
    let s = String::from("racecar");
    println!("Is palindrome: {}", s.is_palindrome());
}

// ===== GENERIC TRAITS =====
trait Container<T> {
    fn add(&mut self, item: T);
    fn get(&self, index: usize) -> Option<&T>;
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T> Container<T> for Stack<T> {
    fn add(&mut self, item: T) {
        self.items.push(item);
    }
    
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index)
    }
}

// ===== TRAIT OBJECTS AND OBJECT SAFETY =====
// Object-safe trait (can be used as trait object)
trait ObjectSafe {
    fn method(&self);
}

// Not object-safe (generic method)
// trait NotObjectSafe {
//     fn generic_method<T>(&self, item: T);
// }

// Not object-safe (returns Self)
// trait AlsoNotObjectSafe {
//     fn returns_self(&self) -> Self;
// }

// ===== BLANKET IMPLEMENTATIONS =====
// Implement trait for all types that satisfy bounds
trait Stringify {
    fn to_string_custom(&self) -> String;
}

impl<T: std::fmt::Display> Stringify for T {
    fn to_string_custom(&self) -> String {
        format!("{}", self)
    }
}

// ===== CONDITIONAL TRAIT IMPLEMENTATION =====
use std::fmt::Debug;

struct Wrapper<T> {
    value: T,
}

// Implement only if T implements Debug
impl<T: Debug> Debug for Wrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wrapper({:?})", self.value)
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: Builder pattern with traits
trait Builder {
    type Output;
    fn build(self) -> Self::Output;
}

struct UserBuilder {
    name: Option<String>,
    age: Option<u32>,
}

impl Builder for UserBuilder {
    type Output = Result<User, String>;
    
    fn build(self) -> Self::Output {
        Ok(User {
            name: self.name.ok_or("Name required")?,
            age: self.age.ok_or("Age required")?,
        })
    }
}

// Pattern 2: Strategy pattern
trait SortStrategy {
    fn sort(&self, data: &mut [i32]);
}

struct BubbleSort;
impl SortStrategy for BubbleSort {
    fn sort(&self, data: &mut [i32]) {
        // Bubble sort implementation
    }
}

struct QuickSort;
impl SortStrategy for QuickSort {
    fn sort(&self, data: &mut [i32]) {
        // Quick sort implementation
    }
}

fn sort_data(data: &mut [i32], strategy: &dyn SortStrategy) {
    strategy.sort(data);
}

// Pattern 3: Trait as capability
trait Read {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

trait Write {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>;
}

// Type that can both read and write
struct File;
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        Ok(0)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }
}

fn copy<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> std::io::Result<()> {
    let mut buffer = [0u8; 1024];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        writer.write(&buffer[..n])?;
    }
    Ok(())
}

// Pattern 4: Newtype pattern with traits
struct Meters(f64);
struct Kilometers(f64);

impl Add for Meters {
    type Output = Meters;
    
    fn add(self, other: Meters) -> Meters {
        Meters(self.0 + other.0)
    }
}

// Cannot accidentally add Meters and Kilometers

// Pattern 5: Trait aliases (nightly feature)
// #![feature(trait_alias)]
// trait Service = Clone + Send + Sync;

// Workaround for stable Rust:
trait Service: Clone + Send + Sync {}
impl<T: Clone + Send + Sync> Service for T {}

// Pattern 6: Sealed traits (prevent external implementation)
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn method(&self);
}

impl sealed::Sealed for MyType {}
impl MyTrait for MyType {
    fn method(&self) {
        // Implementation
    }
}

// Users cannot implement MyTrait for their own types
```
