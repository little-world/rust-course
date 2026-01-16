# Trait Design Patterns
This chapter explores advanced trait patterns: inheritance and bounds for capabilities, associated types vs generics for API design, trait objects for dynamic dispatch, extension traits for extending external types, and sealed traits for controlled implementation.

## Pattern 1: Trait Inheritance and Bounds

*   **Problem**: Expressing complex capability requirements is unclear—a trait needs `Display` but can't require it directly. Combining multiple capabilities is verbose (`T: Clone + Debug + Display`).
*   **Solution**: Use supertrait relationships (`trait Loggable: Debug`) to express requirements. Use trait bounds in generics (`fn process<T: Clone>`), and `where` clauses for readability.

### Example: Super Traits

A supertrait is a trait that another trait depends on. When you declare `trait Printable: Debug`, any type implementing `Printable` must also implement `Debug`. This creates a clear dependency chain and guarantees capabilities.

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

// Usage: Implementing Printable requires implementing Debug (supertrait).
// The print() method uses Debug internally via println!("{:?}", self).
let doc = Document { title: "Rust Guide".to_string(), content: "Learning Rust".to_string() };
doc.print(); // Uses Debug formatting
```


### Example: Multiple Supertraits

Traits can require multiple supertraits using `+` syntax, combining different capabilities. A type must implement all supertraits before it can implement the subtrait. This composes behaviors without code duplication.

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

// Usage: User implements Debug (derived) and Display (manual), so it can implement Loggable.
// The log() method uses both formats; use_loggable() requires the combined bound.
let user = User { name: "Alice".to_string(), id: 42 };
user.log(); // Prints both [DEBUG] and [INFO] lines
use_loggable(&user);
```

This pattern is useful when your abstraction needs multiple orthogonal capabilities. The `Loggable` trait doesn't need to know *how* to debug or display items—it just requires that the capability exists.

### Example: Trait Bounds in Generic Functions

Trait bounds specify what capabilities a generic type must have using `T: Trait` syntax. Multiple bounds use `+` (e.g., `T: Clone + Debug`). For complex bounds, `where` clauses improve readability.

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

// Usage: print_item requires Display, process requires Clone + Debug.
// complex_function uses a where clause for readability with multiple bounds.
print_item("hello"); // &str implements Display
process(vec![1, 2, 3]); // Vec implements Clone + Debug
let result = complex_function(vec![1, 2], String::new()); // Where clause in action
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

You can implement traits conditionally based on what traits the type parameters implement. `impl<T: Clone> Clone for Wrapper<T>` means Wrapper is Clone only when T is Clone. The compiler automatically determines which implementations apply for each concrete type.

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

// Usage: Wrapper<T> is Clone only when T is Clone, Debug only when T is Debug.
// The compiler automatically selects which trait impls apply for each concrete type.
let w = Wrapper("hello".to_string());
let w_clone = w.clone(); // Works because String is Clone
println!("{:?}", w); // Works because String is Debug
```

This pattern allows `Wrapper<String>` to be `Clone` and `Debug`, while `Wrapper<Rc<RefCell<i32>>>` is only `Clone` (because `RefCell` isn't `Debug` in a useful way). The compiler automatically determines which implementations apply.

### Example: Trait Bound Patterns

Several common patterns emerge when working with trait bounds. Methods can be conditionally available based on type parameter bounds. Higher-rank trait bounds (`for<'a>`) express constraints that must hold for all lifetimes.

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

// Usage: Query::new works for any T, but duplicate() only appears when T: Clone.
// Higher-rank bounds (for<'a>) express constraints that must hold for all lifetimes.
let q = Query::new(vec![1, 2, 3]);
let q_dup = q.duplicate(); // Only available because Vec is Clone
process_with_lifetime(|s| s); // Closure works with higher-rank bounds
```

The builder pattern becomes particularly powerful with conditional trait implementations, as methods only appear when the type parameter supports them.

## Pattern 2: Associated Types vs Generics

*   **Problem**: A generic trait like `Parser<Output>` allows a single type to have multiple implementations (e.g., for different `Output` types), which can be confusing. Call sites become verbose (`parser.parse::<serde_json::Value>()`), and it's unclear if a type parameter is an "input" or an "output".
*   **Solution**: Use **associated types** when an implementing type determines a single, specific "output" type (`trait Parser { type Output; }`). Use **generics** when the caller chooses an "input" type and multiple implementations are desirable (`trait From<T>`).

#### Example: Generics

With generic type parameters, a single type can implement the same trait multiple times for different type arguments. This allows flexibility but requires the caller to specify which implementation to use. The syntax becomes verbose with turbofish (`Parser::<i32>::parse`).

```rust
//=================================================
// With generics: Multiple implementations possible
//=================================================
trait Parser<Output> {
    fn parse(&self, input: &str) -> Result<Output, String>;
}

struct JsonParser;

impl Parser<serde_json::Value> for JsonParser {
    fn parse(
        &self,
        input: &str,
    ) -> Result<serde_json::Value, String> {
        serde_json::from_str(input).map_err(|e| e.to_string())
    }
}

//=========================================================
// Could also implement Parser<MyCustomType> for JsonParser
//=========================================================

// Demo without serde dependency
struct SimpleParser;

impl Parser<i32> for SimpleParser {
    fn parse(&self, input: &str) -> Result<i32, String> {
        input.parse().map_err(|e| format!("{}", e))
    }
}

impl Parser<bool> for SimpleParser {
    fn parse(&self, input: &str) -> Result<bool, String> {
        match input {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err("invalid bool".to_string()),
        }
    }
}

// Usage: SimpleParser implements Parser<i32> and Parser<bool> separately.
// Caller must specify which impl to use with turbofish syntax.
let parser = SimpleParser;
let num: Result<i32, _> = Parser::<i32>::parse(&parser, "42");
let b: Result<bool, _> = Parser::<bool>::parse(&parser, "true");
```

#### Example: Associated Types: One Implementation

Associated types express "there is one specific type for this implementation." Each implementor defines exactly one concrete type for the associated type. The compiler infers the output type from the implementor, eliminating the need for turbofish syntax.

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

//=======================================================
// Cannot impl Parser again for JsonParser (one Output!)
//=======================================================
```

#### Example: Ergonomics: Associated Types Win for Consumers

Associated types lead to cleaner call sites because the output type is determined by the implementor. With generics, functions need extra type parameters that callers must specify. With associated types, the compiler infers everything from the concrete type.

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
let value: serde_json::Value =
    use_generic_parser::<serde_json::Value, _>(JsonParser, "{}");

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

#### Example: When to Use Each

The choice between generics and associated types depends on whether the type parameter is an "input" or "output." Generics let callers choose the type; associated types let implementors fix it. Here are the guidelines:

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

#### Example: Combining Both

Sometimes you want both generics and associated types in the same trait. Use generics for inputs that callers choose, and associated types for outputs determined by the implementation. This gives flexibility where needed while keeping the API clean.

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

//=======================================================
// Could also impl Converter<i32> with different types
//=======================================================
```

#### Example: Associated Types with Bounds

Associated types can have trait bounds that constrain what types implementors can use. This ensures the associated type has capabilities needed by the trait's methods. Implementors must choose types that satisfy these bounds.

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

#### Example: The Iterator Pattern Deep Dive

`Iterator` is the canonical example of associated types done right. Each collection has exactly one item type—`Vec<i32>` yields `i32`, not something the caller chooses. This makes iterator chains like `.map().filter().collect()` ergonomic and type-safe.

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
*   **Solution**: Use trait objects (`&dyn Trait`) for dynamic dispatch. This creates a single version of the function that accepts any type implementing the trait, looking up the correct method at runtime via a vtable. Dynamic dispatch results in smaller binary sizes and allows for runtime polymorphism (e.g., plugin systems).

### Example: static dispatch

With generics, the compiler generates a specialized copy of the function for each concrete type used. This is called monomorphization—`process::<i32>` and `process::<String>` become separate functions. The result is fast code but larger binaries.

```rust
fn process<T: Display>(item: T) {
    println!("{}", item);
}


// Usage: Each call generates a specialized function (monomorphization).
// process::<i32>, process::<&str>, and process::<f64> become separate functions.
process(42i32);   // Generates process::<i32>
process("hello"); // Generates process::<&str>     
```

Each call site gets optimized code for that specific type. Fast, but increases binary size (code bloat).

### Example: Dynamic dispatch (trait objects):

With trait objects (`&dyn Trait`), a single function handles all types at runtime. The compiler generates one function that uses a vtable to look up the correct method. This trades a small runtime cost for smaller binary size.

```rust
use std::fmt::Display;

fn process(item: &dyn Display) {
    println!("{}", item);
}

// Usage: One process() function handles all types at runtime via vtable lookup.
// Trait objects enable heterogeneous collections like Vec<&dyn Display>.
let num: i32 = 42;
let text: &str = "hello";
process(&num); // Same function, different types
let items: Vec<&dyn Display> = vec![&num, &text]; // Different types in one Vec
```

One function handles all types. Smaller binary, but slight runtime cost for the vtable lookup.

### Example: Creating Trait Objects

Trait objects must be behind a pointer (`&dyn`, `Box<dyn>`, `Rc<dyn>`, `Arc<dyn>`) because their size is unknown at compile time. This enables heterogeneous collections—a `Vec<Box<dyn Drawable>>` can hold circles, rectangles, and any other drawable type. Each element is accessed through the trait interface.

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

// Usage: Box<dyn Drawable> enables heterogeneous collections of different shape types.
// All shapes are accessed through the same trait interface via draw_all().
let shapes: Vec<Box<dyn Drawable>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rectangle { width: 10.0, height: 20.0 }),
];
draw_all(&shapes); // Each shape drawn via trait method
```

### Example: Object Safety Requirements

Not all traits can be made into trait objects—only "object-safe" traits qualify. The compiler must be able to call methods without knowing the concrete type at compile time. These rules ensure the vtable can dispatch all methods correctly.

A trait is "object safe" if:

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
    fn returns_self(self) -> Self; // ✗ requires Sized
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

You can make traits object-safe with careful design by replacing generics with trait objects. Instead of `fn create<T: Serialize>`, use `fn create(&self, item: &dyn Serialize)`. Another approach is splitting functionality into separate traits.

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

Sometimes you need to convert a trait object back to a concrete type using `std::any::Any`. By requiring `Any` as a supertrait and providing an `as_any()` method, you can use `downcast_ref::<T>()`. Use this sparingly—it breaks abstraction.

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

// Usage: Use as_any() and downcast_ref::<T>() to recover concrete types from trait objects.
// Downcasting succeeds for matching types, returns None for mismatches.
let circle = Circle { radius: 5.0 };
let shape: &dyn Shape = &circle;
if let Some(c) = try_as_circle(shape) {
    println!("Radius: {}", c.radius); // Access concrete type fields
}
```

This pattern is useful but breaks abstraction—use it sparingly, only when you truly need concrete type information.

### Example: Trait Objects with Lifetime Bounds

Trait objects can have lifetime bounds using `dyn Trait + 'a` syntax. This specifies how long the concrete type behind the trait object must live. It's essential when the trait object contains or references borrowed data.

```rust
trait Processor {
    fn process(&self, data: &str) -> String;
}

//===========================
// Trait object with lifetime
//===========================
fn process_data<'a>(
    processor: &'a dyn Processor,
    data: &'a str,
) -> String {
    processor.process(data)
}

//=================================
// Boxed trait object with lifetime
//=================================
struct Handler<'a> {
    processor: Box<dyn Processor + 'a>,
}

struct UpperCaseProcessor;

impl Processor for UpperCaseProcessor {
    fn process(&self, data: &str) -> String {
        data.to_uppercase()
    }
}

struct PrefixProcessor<'a> {
    prefix: &'a str,
}

impl<'a> Processor for PrefixProcessor<'a> {
    fn process(&self, data: &str) -> String {
        format!("{}{}", self.prefix, data)
    }
}

// Usage: Box<dyn Processor + 'a> specifies the trait object lives at least as long as 'a.
// PrefixProcessor borrows data, so its lifetime must be tracked.
let prefix = String::from(">>> ");
let prefixer = PrefixProcessor { prefix: &prefix };
let result = process_data(&prefixer, "message"); // Returns ">>> message"
```

The `+ 'a` syntax means "the trait object must live at least as long as `'a`". This ensures references in the trait implementation remain valid.

## Pattern 4: Extension Traits

*   **Problem**: You can't add methods to types from other crates (the "orphan rule"). You want to extend standard types like `Vec` or `String` with domain-specific helpers, but can't modify their source code.
*   **Solution**: Define a new trait (an "extension trait") with the desired methods. Then, implement that trait for the external type.


### Example: Basic Extension Trait

The orphan rule prevents implementing a foreign trait on a foreign type. However, you can implement your own trait on a foreign type—this is the extension trait pattern. Here we define `SumExt` and implement it for `Vec<i32>` to add a `sum_ext` method.

```rust
trait SumExt {
    fn sum_ext(&self) -> i32;
}

impl SumExt for Vec<i32> {
    fn sum_ext(&self) -> i32 {
        self.iter().sum()
    }
}

// Extend Vec<f64> too
impl SumExt for Vec<f64> {
    fn sum_ext(&self) -> i32 {
        self.iter().sum::<f64>() as i32
    }
}

// Usage: The extension trait adds sum_ext() to Vec<i32> and Vec<f64>.
// It works like a native method once the trait is in scope.
let numbers = vec![1, 2, 3, 4, 5];
let sum = numbers.sum_ext(); // Returns 15
```

### Example: Blanket Iterator Extensions

Define a trait with a supertrait bound (`: Iterator`) and provide a blanket impl for all `I: Iterator`. This adds your methods to every iterator in the program without touching any iterator types. The `Sized` bound on individual methods allows trait object compatibility.

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

// Blanket impl: applies to any type that is an Iterator.
impl<I: Iterator> IteratorExt for I {}

// Usage: The blanket impl adds counts() to every Iterator automatically.
// Works on Vec, array, range, and string iterators.
let words = vec!["apple", "banana", "apple"];
let counts = words.into_iter().counts(); // HashMap: {"apple": 2, "banana": 1}
```

### Example: Ergonomic Error Handling

Extension traits can add context or logging to the standard `Result` type. Here `ResultExt` provides a `log_err` method that logs errors before passing them up the call stack. This pattern is used extensively in libraries like `anyhow` for error context chaining.

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

// Add context method for more info
trait ResultContextExt<T, E> {
    fn with_context(self, msg: &str) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> ResultContextExt<T, E> for Result<T, E> {
    fn with_context(self, msg: &str) -> Result<T, String> {
        self.map_err(|e| format!("{}: {}", msg, e))
    }
}

// Usage: log_err() logs errors while preserving them; with_context() adds context to error messages.
// Both work seamlessly with the standard Result type.
let result: Result<i32, &str> = Err("not found");
let contexted = result.with_context("Loading user"); // "Loading user: not found"
```

### Example: Extending Standard Types

You can add domain-specific helper methods to standard library types like `String` and `str`. Using generic bounds like `T: AsRef<str>` makes the extension work for multiple string types. The trait must be in scope to use the extended methods.

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

// Usage: truncate_to() works on &str, String, and Cow<str> via AsRef<str> bound.
// Strings shorter than max_len are returned unchanged.
let s = "This is a long string that needs truncation";
let truncated = s.truncate_to(20); // "This is a long st..."
```

### Example: Conditional Extensions

An extension can be conditional on the capabilities of the type being extended. This `DebugExt` trait is implemented for any `T: Debug`, giving all debuggable types a `debug_print` method. The blanket impl `impl<T: Debug> DebugExt for T` automatically covers thousands of types.

```rust
trait DebugExt {
    fn debug_print(&self);
}

impl<T: std::fmt::Debug> DebugExt for T {
    fn debug_print(&self) {
        println!("{:?}", self);
    }
}

// Usage: The blanket impl adds debug_print() to any type implementing Debug.
// Works on Vec, tuples, Option, and custom #[derive(Debug)] structs.
let numbers = vec![1, 2, 3];
numbers.debug_print(); // Prints "[1, 2, 3]"
```



## Pattern 5: Sealed Traits

*   **Problem**: As a library author, you want to publish a trait that users can depend on, but you want to prevent them from implementing it themselves. This allows you to add new methods to the trait later without it being a breaking change.
*   **Solution**: Create a private `sealed` module with a public but un-implementable `Sealed` trait. Make your public trait a supertrait of `sealed::Sealed`.

### Example: Basic Sealed Trait

A sealed trait uses a private supertrait to prevent external implementations. The `sealed::Sealed` trait is public but in a private module, so external crates can't access it. This lets library authors add methods without breaking changes.

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

struct MyType {
    value: i32
}
impl MyTrait for MyType {
    fn my_method(&self) {
        println!("Value: {}", self.value);
    }
}

struct AnotherType;

impl sealed::Sealed for AnotherType {}
impl MyTrait for AnotherType {
    fn my_method(&self) {
        println!("AnotherType impl");
    }
}

// External crates can USE MyTrait but cannot IMPLEMENT it 

fn use_trait<T: MyTrait>(item: &T) {
    item.my_method();
    item.new_method();
}

// Usage: External crates can use MyTrait but cannot implement it.
// Library authors can add methods like new_method() without breaking changes.
let my = MyType { value: 42 };
my.my_method(); // Prints "Value: 42"
my.new_method(); // Default implementation works
```

### Example: Dependency Injection with Traits

Use traits to define interfaces for external services like databases and email. Your code depends on trait bounds, not concrete types. This enables easy mocking in tests and swapping implementations in production.

```rust
trait Database {
    fn get_user(&self, id: i32) -> Option<User>;
    fn save_user(&self, user: &User) -> Result<(), Error>;
}

trait EmailService {
    fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), Error>;
}

struct UserService<D, E> {
    database: D,
    email: E,
}

impl<D: Database, E: EmailService> UserService<D, E> {
    fn new(database: D, email: E) -> Self {
        UserService { database, email }
    }

    fn register_user(
        &self,
        name: &str,
        email: &str,
    ) -> Result<User, Error> {
        let user = User {
            id: generate_id(),
            name: name.to_string(),
            email: email.to_string(),
        };

        self.database.save_user(&user)?;
        self.email.send_email(email, "Welcome!", "Thanks")?;

        Ok(user)
    }
}

// Supporting types
#[derive(Clone, Debug, PartialEq)]
struct User { id: i32, name: String, email: String }
#[derive(Debug)]
struct Error;
fn generate_id() -> i32 { 1 }

// Mock implementations for testing
use std::cell::RefCell;

struct MockDb { users: RefCell<Vec<User>> }
struct MockEmail { sent: RefCell<Vec<String>> }

impl Database for MockDb {
    fn get_user(&self, id: i32) -> Option<User> {
        self.users.borrow().iter()
            .find(|u| u.id == id).cloned()
    }
    fn save_user(&self, user: &User) -> Result<(), Error> {
        self.users.borrow_mut().push(user.clone());
        Ok(())
    }
}

impl EmailService for MockEmail {
    fn send_email(
        &self, to: &str, _: &str, _: &str
    ) -> Result<(), Error> {
        self.sent.borrow_mut().push(to.to_string());
        Ok(())
    }
}

// Usage: UserService accepts any D: Database and E: EmailService.
// In tests, inject MockDb and MockEmail; in production, use real implementations.
let db = MockDb { users: RefCell::new(vec![]) };
let email = MockEmail { sent: RefCell::new(vec![]) };
let service = UserService::new(db, email);
let user = service.register_user("Alice", "a@b.com"); // Uses mock implementations
```


### Summary

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
