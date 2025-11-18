# Trait Design Patterns

Trait Inheritance and Bounds

- Problem: Expressing capability requirements; combining traits; conditional implementations unclear; complex constraints unreadable
- Solution: Supertrait relationships (trait A: B); multiple bounds (T: A + B); where clauses; conditional impl based on bounds
- Why It Matters: Expresses "to be A, must be B"; enables capability composition; conditional impl avoids bloat; where improves readability
- Use Cases: API design, generic constraints, blanket implementations, builder patterns, capability requirements

Associated Types vs Generics

- Problem: Generic traits enable multiple implementations confusing API; type parameters verbose at call sites; unclear output types
- Solution: Associated types for output determined by implementor; generics for input chosen by caller; combine both when needed
- Why It Matters: Associated types = simpler API, one implementation per type; generics = multiple implementations possible; ergonomics vs flexibility
- Use Cases: Iterator pattern, parser output types, conversions, data transformations, type families

Trait Objects and Dynamic Dispatch

- Problem: Static dispatch causes code bloat; can't have heterogeneous collections; vtable overhead; object safety constraints unclear
- Solution: &dyn Trait for dynamic dispatch; Box/Rc/Arc<dyn Trait> for owned; understand object safety; downcast with Any when needed
- Why It Matters: Dynamic dispatch = one implementation, smaller binary, ~2-3ns overhead; enables heterogeneous collections; object safety prevents generics
- Use Cases: Plugin systems, heterogeneous collections, GUI frameworks, callbacks, polymorphic APIs, reduced binary size

Extension Traits

- Problem: Can't add methods to external types; want to extend standard library; need modular opt-in functionality
- Solution: Define trait with methods; impl for external type; users import trait to get methods; blanket impl for all types
- Why It Matters: Extends types you don't own; modular opt-in design; enables ecosystem interop; doesn't break coherence rules
- Use Cases: Iterator extensions, Result/Option helpers, string utilities, collection extensions, error handling, type conversions

Sealed Traits

- Problem: Public trait shouldn't be implemented by users; want to extend trait without breaking compatibility; prevent external impls
- Solution: Private supertrait in private module; only crate can satisfy; prevents external implementations; enables safe evolution
- Why It Matters: Control trait implementations; prevents misuse; allows non-breaking trait changes; maintains API guarantees
- Use Cases: Internal abstractions, public trait with limited impls, future-proof APIs, unsafe traits, protocol enforcement


This chapter explores advanced trait patterns: inheritance and bounds for capabilities, associated types vs generics for API design, trait objects for dynamic dispatch, extension traits for extending external types, and sealed traits for controlled implementation.

## Pattern 1: Trait Inheritance and Bounds

**Problem**: Expressing complex capability requirements is unclear—trait needs Display but can't require it directly. Combining multiple capabilities verbose (T: Clone + Debug + Display). Conditional implementations confusing: "only implement MyTrait if T implements Clone" not obvious syntax. Complex generic constraints become unreadable (long type parameter lists). No way to say "this type needs these capabilities". Blanket implementations need bounds but syntax unclear. Builder pattern methods should appear only when type supports them.

**Solution**: Use supertrait relationships `trait Loggable: Debug + Display` to express "to be Loggable, must be Debug and Display". Use trait bounds in generics `fn process<T: Clone + Debug>(item: T)`. Use where clauses for readability when multiple constraints. Conditional implementations: `impl<T: Clone> MyTrait for Wrapper<T>` implements only when T is Clone. Blanket implementations: `impl<T: Iterator> MyExt for T` extends all iterators. Higher-rank trait bounds `for<'a> Fn(&'a str)` for lifetime-generic closures.

**Why It Matters**: Supertrait expresses capability requirements clearly—"Printable needs Debug" is declarative. Combining traits enables rich abstractions from simple components. Conditional implementations prevent bloat: Wrapper<String> gets Clone, Wrapper<NonClone> doesn't, all automatic. Where clauses improve readability: complex constraints don't clutter function signature. Blanket implementations powerful: one impl extends entire category of types. Type system enforces requirements at compile-time, no runtime checks. Capability-based design: types declare what they can do, functions declare what they need.

**Use Cases**: API design (trait requirements for public APIs), generic function constraints (specify needed capabilities), blanket implementations (extend all types matching pattern), builder patterns (methods appear only when supported), capability requirements (Serializable needs Debug for errors), trait object preparation (ensure object safety), library design (compose small traits into larger abstractions), conditional functionality (additional methods when T: Clone).

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

### Multiple Supertraits

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

### Trait Bounds in Generic Functions

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

### Conditional Implementation with Trait Bounds

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

### Trait Bound Patterns

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

**Problem**: Generic trait `Parser<Output>` allows multiple implementations of Parser for same type with different Output—confusing which to use. Type parameters verbose at call sites: `use_parser::<serde_json::Value, JsonParser>` requires specifying both types. Output type unclear: is it input chosen by caller or output determined by parser? Multiple implementations possible when only one makes sense (Vec's iterator produces &T, not arbitrary type). API consumers forced to specify types compiler could infer. Trait with many type parameters unreadable.

**Solution**: Use associated types when output determined by implementor: `trait Parser { type Output; }`. Use generics when input chosen by caller: `trait From<T>`. Associated types: one implementation per type, simpler call sites (compiler infers). Generics: multiple implementations possible, explicit at call site. Combine both: `trait Converter<Input> { type Output; }` for flexibility where needed, clarity where not. Associated types with bounds: `type Node: Display` ensures capabilities. Iterator pattern canonical example: `type Item` because collection determines item type, not caller.

**Why It Matters**: Associated types = ergonomic API: `parser.parse()` infers output type, no turbofish needed. Generics = flexible API: String can be From<&str>, From<Vec<u8>>, From<String> (multiple impls). One implementation constraint prevents confusion: JsonParser has one Output, not ambiguous. Call site simplicity: associated types eliminate `use_parser::<ComplexType, _>` turbofish. Type families: related types grouped (Graph has Node and Edge associated types). Compiler inference better with associated types. Documentation clearer: "this parser outputs JSON" vs "this parser outputs T".

**Use Cases**: Iterator pattern (item type determined by collection), parser output types (parser determines what it produces), conversion traits with multiple sources (From<T> for String), data transformation pipelines (associated Output/Error types), graph algorithms (Node/Edge types associated with graph), database query builders (associated Row type), serialization (associated Output format), type families (group related types together).

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
// This means JsonParser could parse to multiple different types
```

With generics, a single type can implement the trait multiple times with different type parameters. Sometimes this is exactly what you want, but often it's confusing.

### Associated Types: One Implementation

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

### Ergonomics: Associated Types Win for Consumers

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

### When to Use Each

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

### Combining Both

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

### Associated Types with Bounds

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

### The Iterator Pattern Deep Dive

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

**Problem**: Static dispatch (generics) generates separate function copy per type—code bloat for large generic functions. Can't have heterogeneous collections (Vec can't hold Circle and Rectangle together). Binary size explosion with many instantiations. Vtable indirection overhead when needed. Object safety constraints unclear: which traits work as &dyn Trait? Can't downcast trait objects to concrete types. Lifetime management complex with boxed trait objects. Generic methods prevent trait objects.

**Solution**: Use trait objects `&dyn Trait` for dynamic dispatch—one implementation, vtable lookup at runtime. Box/Rc/Arc<dyn Trait> for owned trait objects. Understand object safety rules: no generic methods, no Self: Sized bound, must have &self/&mut self receiver. Heterogeneous collections: `Vec<Box<dyn Drawable>>` holds different types. Downcast with Any trait when concrete type needed. Lifetime bounds on trait objects: `Box<dyn Trait + 'a>`. Choose based on needs: static for performance, dynamic for flexibility/binary size.

**Why It Matters**: Dynamic dispatch = smaller binary: one implementation not N instantiations. Heterogeneous collections possible: plugin systems, GUI widgets, game entities. Runtime polymorphism: choose implementation at runtime, not compile-time. Performance trade-off: vtable lookup adds ~2-3ns per call vs direct call, but prevents inlining. Binary size matters: embedded systems, WebAssembly. Object safety prevents issues: generic methods need concrete type at compile-time, incompatible with runtime polymorphism. Downcasting enables type recovery when needed but breaks abstraction.

**Use Cases**: Plugin systems (load implementations at runtime), heterogeneous collections (Vec<Box<dyn Widget>>), GUI frameworks (different widget types), game engines (entity components), callback systems (store different closures), event handlers (different event types), middleware (chain of handlers), reduced binary size (embedded/WASM), polymorphic APIs (database drivers, serialization formats).

```rust
fn process<T: Display>(item: T) {
    println!("{}", item);
}

//====================
// Compiler generates:
//====================
// fn process_i32(item: i32) { println!("{}", item); }
//==========================================================
// fn process_String(item: String) { println!("{}", item); }
//==========================================================
```

Each call site gets optimized code for that specific type. Fast, but increases binary size (code bloat).

Dynamic dispatch (trait objects):

```rust
fn process(item: &dyn Display) {
    println!("{}", item);
}

//==========================
// Single function generated
//==========================
// Uses vtable lookup to find the right Display implementation at runtime
```

One function handles all types. Smaller binary, but slight runtime cost for the vtable lookup.

### Creating Trait Objects

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

### Object Safety Requirements

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

### Making Traits Object-Safe

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

### Downcasting Trait Objects

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

### Trait Objects with Lifetime Bounds

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

### Costs of Dynamic Dispatch

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

**Problem**: Can't add methods to types from other crates (orphan rule prevents impl Vec { ... }). Want to extend standard library types (String, Vec, Option) with domain-specific methods. Need opt-in functionality (users choose to import extension). Ecosystem interop: multiple crates want to extend same type. Can't modify external library to add methods. Want modular design where extensions are separate. Coherence rules prevent implementing external traits on external types.

**Solution**: Define extension trait with desired methods. Implement trait for external type (allowed by coherence). Users import trait to get methods on type. Blanket implementation extends all types matching pattern: `impl<T: Iterator> IterExt for T`. Name clearly (ResultExt, StringExt) to indicate extension. Provide trait in separate module for opt-in import. Use where Self: Sized for object-safe base with non-object-safe extensions.

**Why It Matters**: Extends types you don't own—add methods to Vec, String, Result without modifying std. Modular design: functionality is opt-in via import, doesn't pollute namespace. Ecosystem interop: multiple crates extend same type without conflicts. Coherence satisfied: your crate defines trait, implements for external type (one of trait/type is yours). Enables rich APIs: standard types get domain-specific methods. Backward compatible: adding extension trait doesn't break existing code. Composition: import multiple extension traits for combined functionality.

**Use Cases**: Iterator extensions (custom collection methods: counts, unique, chunks), Result/Option helpers (log_err, unwrap_or_log, context), string utilities (truncate_to, is_valid_email), collection extensions (HashMap: get_or_compute), error handling (add context to errors), type conversions (TryInto extensions), async utilities (timeout, retry on futures), parsing helpers (FromStr extensions).

```rust
//================================
// Cannot do this! Vec is from std
//================================
impl Vec<i32> {
    fn sum(&self) -> i32 {
        self.iter().sum()
    }
}
```

Extension traits solve this elegantly:

```rust
trait SumExt {
    fn sum_ext(&self) -> i32;
}

impl SumExt for Vec<i32> {
    fn sum_ext(&self) -> i32 {
        self.iter().sum()
    }
}

fn example() {
    let numbers = vec![1, 2, 3, 4, 5];
    println!("Sum: {}", numbers.sum_ext()); // Works!
}
```

Now any code that imports `SumExt` gets the `sum_ext` method on `Vec<i32>`.

### Iterator Extension Traits

The `Iterator` trait demonstrates extension traits beautifully:

```rust
use std::collections::HashMap;

trait IteratorExt: Iterator {
    // Convert iterator of tuples into HashMap
    fn collect_hashmap<K, V>(self) -> HashMap<K, V>
    where
        Self: Sized + Iterator<Item = (K, V)>,
        K: Eq + std::hash::Hash,
    {
        self.collect()
    }

    // Count occurrences of each item
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

//============================================
// Automatically implemented for all iterators
//============================================
impl<I: Iterator> IteratorExt for I {}

fn example() {
    let words = vec!["apple", "banana", "apple", "cherry", "banana", "apple"];
    let counts = words.into_iter().counts();

    println!("{:?}", counts); // {"apple": 3, "banana": 2, "cherry": 1}
}
```

This pattern extends all iterators with new functionality while keeping the extension modular and opt-in.

### Extension Traits for Error Handling

Extension traits can make error handling more ergonomic:

```rust
trait ResultExt<T> {
    fn log_err(self, context: &str) -> Result<T, Box<dyn std::error::Error>>;
}

impl<T, E: std::error::Error + 'static> ResultExt<T> for Result<T, E> {
    fn log_err(self, context: &str) -> Result<T, Box<dyn std::error::Error>> {
        self.map_err(|e| {
            eprintln!("[ERROR] {}: {}", context, e);
            Box::new(e) as Box<dyn std::error::Error>
        })
    }
}

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::read_to_string("config.toml")
        .log_err("Failed to read config")?;

    Ok(())
}
```

This pattern centralizes error logging and context addition.

### Type-Specific Extensions

You can extend specific types with convenient methods:

```rust
trait StringExt {
    fn truncate_to(&self, max_len: usize) -> String;
    fn remove_whitespace(&self) -> String;
}

impl StringExt for String {
    fn truncate_to(&self, max_len: usize) -> String {
        if self.len() <= max_len {
            self.clone()
        } else {
            format!("{}...", &self[..max_len.saturating_sub(3)])
        }
    }

    fn remove_whitespace(&self) -> String {
        self.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

impl StringExt for str {
    fn truncate_to(&self, max_len: usize) -> String {
        if self.len() <= max_len {
            self.to_string()
        } else {
            format!("{}...", &self[..max_len.saturating_sub(3)])
        }
    }

    fn remove_whitespace(&self) -> String {
        self.chars().filter(|c| !c.is_whitespace()).collect()
    }
}

fn example() {
    let long_string = "This is a very long string that needs truncation".to_string();
    let truncated = long_string.truncate_to(20);

    let spaced = "Hello   World   !";
    let compact = spaced.remove_whitespace();

    println!("{}", truncated); // "This is a very lo..."
    println!("{}", compact);   // "HelloWorld!"
}
```

### Conditional Extension Traits

Extensions can be conditional based on trait bounds:

```rust
trait DebugExt {
    fn debug_print(&self) -> String;
}

impl<T: std::fmt::Debug> DebugExt for T {
    fn debug_print(&self) -> String {
        format!("{:?}", self)
    }
}

//====================================
// Now all Debug types get debug_print
//====================================
fn example() {
    let numbers = vec![1, 2, 3];
    println!("{}", numbers.debug_print()); // Works because Vec<i32> is Debug
}
```

This pattern is incredibly powerful—one implementation provides functionality to infinite types.

## Pattern 5: Sealed Traits

**Problem**: Public trait shouldn't be implemented by external users—want to reserve right to add methods without breaking changes. Can't evolve trait (add methods) if external impls exist—breaking change. Unsafe trait needs controlled implementations (only crate can verify safety invariants). Want public interface but private implementation set. Protocol enforcement: only specific types should implement trait. Future compatibility: need to change trait internals without affecting users.

**Solution**: Create private `sealed` module with private `Sealed` trait. Make public trait require Sealed as supertrait: `pub trait MyTrait: sealed::Sealed`. Only your crate can implement sealed::Sealed (it's private). External crates can use MyTrait but can't implement it. Allows adding methods to MyTrait without breaking compatibility—external impls impossible so no break. Document that trait is sealed to set expectations.

**Why It Matters**: Control trait implementations: prevent misuse, maintain invariants only your crate can verify. Non-breaking evolution: add methods to sealed trait without semver major version bump. Safety guarantees: unsafe traits sealed ensure only verified implementations. Future-proof APIs: reserve implementation rights while providing public interface. Documentation: sealed trait signals "use but don't implement". Protocol enforcement: only sanctioned types implement trait. Prevents incorrect implementations that violate trait's contract.

**Use Cases**: Internal abstractions (pub trait, private impls), public traits with limited implementations (only std types), future-proof APIs (reserve right to add methods), unsafe traits (verify safety invariants internally), protocol enforcement (only certain types valid), standard library patterns (Iterator is open, but some traits sealed), marker traits with meaning (Send/Sync are special, user impls wrong), library evolution (add functionality without breaking).

```rust
trait ToString {
    fn to_string(&self) -> String;
}

//======================================================================
// Blanket implementation: ToString for all types that implement Display
//======================================================================
impl<T: std::fmt::Display> ToString for T {
    fn to_string(&self) -> String {
        format!("{}", self)
    }
}

//=================================================================
// Now any type that implements Display automatically gets ToString
//=================================================================
```

This is why you can call `.to_string()` on integers, floats, and any other `Display` type—the blanket implementation provides it.

### The From/Into Pattern

The standard library's `From` and `Into` traits showcase blanket implementations:

```rust
trait From<T> {
    fn from(value: T) -> Self;
}

trait Into<T> {
    fn into(self) -> T;
}

//=======================================================================================
// Blanket implementation: Into is automatically implemented for all From implementations
//=======================================================================================
impl<T, U> Into<U> for T
where
    U: From<T>,
{
    fn into(self) -> U {
        U::from(self)
    }
}

//================================
// You only need to implement From
//================================
impl From<i32> for f64 {
    fn from(value: i32) -> f64 {
        value as f64
    }
}

//================================
// Into is automatically available
//================================
fn example() {
    let x: i32 = 42;
    let y: f64 = x.into(); // Works because of blanket impl!
}
```

This pattern reduces boilerplate—implement `From`, get `Into` for free.

### Trait Composition with Blanket Impls

Blanket implementations can compose multiple traits:

```rust
trait Serialize {
    fn serialize(&self) -> String;
}

trait Deserialize {
    fn deserialize(data: &str) -> Result<Self, String>
    where
        Self: Sized;
}

//==============================================================
// Blanket impl: all Serialize + Deserialize types get RoundTrip
//==============================================================
trait RoundTrip: Serialize + Deserialize {
    fn round_trip(&self) -> Result<Self, String>
    where
        Self: Sized,
    {
        let serialized = self.serialize();
        Self::deserialize(&serialized)
    }
}

//=====================================================
// Automatically implemented for types with both traits
//=====================================================
impl<T> RoundTrip for T where T: Serialize + Deserialize {}

#[derive(Debug, PartialEq)]
struct Data {
    value: i32,
}

impl Serialize for Data {
    fn serialize(&self) -> String {
        self.value.to_string()
    }
}

impl Deserialize for Data {
    fn deserialize(data: &str) -> Result<Self, String> {
        Ok(Data {
            value: data.parse().map_err(|e| format!("{}", e))?,
        })
    }
}

fn example() {
    let data = Data { value: 42 };
    let round_tripped = data.round_trip().unwrap();
    assert_eq!(data, round_tripped);
}
```

The `RoundTrip` trait is automatically implemented for any type with both `Serialize` and `Deserialize`.

### Blanket Implementations for Smart Pointers

Blanket implementations work excellently with smart pointers:

```rust
trait Process {
    fn process(&self) -> String;
}

//==========================================================
// Blanket impl: Process for all references to Process types
//==========================================================
impl<T: Process + ?Sized> Process for &T {
    fn process(&self) -> String {
        (**self).process()
    }
}

//===========================================
// Blanket impl: Process for all Box<Process>
//===========================================
impl<T: Process + ?Sized> Process for Box<T> {
    fn process(&self) -> String {
        (**self).process()
    }
}

//==========================
// Similar for Arc, Rc, etc.
//==========================

struct Data {
    value: String,
}

impl Process for Data {
    fn process(&self) -> String {
        format!("Processing: {}", self.value)
    }
}

fn use_processor<T: Process>(processor: T) -> String {
    processor.process()
}

fn example() {
    let data = Data { value: "test".to_string() };

    // All of these work thanks to blanket impls
    use_processor(&data);
    use_processor(Box::new(data.clone()));
    use_processor(std::sync::Arc::new(data.clone()));
}
```

This pattern makes smart pointers transparent—you can use `Box<T>` wherever you'd use `T`.

### Marker Traits with Blanket Impls

Marker traits (traits with no methods) can use blanket implementations to automatically mark types:

```rust
//===============================================================
// Marker trait for types that can be safely sent between threads
//===============================================================
trait ThreadSafe {}

//===================================================
// Blanket impl: all Send + Sync types are ThreadSafe
//===================================================
impl<T: Send + Sync> ThreadSafe for T {}

fn requires_thread_safe<T: ThreadSafe>(_value: T) {
    println!("Value is thread-safe");
}

fn example() {
    requires_thread_safe(42); // i32 is Send + Sync, so it's ThreadSafe
    requires_thread_safe(vec![1, 2, 3]); // Vec<i32> too
}
```

This pattern creates semantic groupings of types based on existing traits.

### Negative Trait Bounds (Unstable)

While not stable, negative trait bounds would allow conditional implementations based on what traits a type *doesn't* implement:

```rust
//===============================
// Future syntax (not yet stable)
//===============================
impl<T: !Copy> Clone for Wrapper<T> {
    fn clone(&self) -> Self {
        // Special clone logic for non-Copy types
    }
}

//==================================================
// Workaround: Use specialization or different types
//==================================================
```

For now, work around this limitation with different types or specialization.

### Coherence and Orphan Rules

Blanket implementations must respect Rust's coherence rules:

```rust
//===============================
// You can do this in your crate:
//===============================
trait MyTrait {
    fn my_method(&self);
}

impl<T: std::fmt::Display> MyTrait for T {
    fn my_method(&self) {
        println!("{}", self);
    }
}

//======================================
// But you cannot do this (orphan rule):
//======================================
// impl<T> std::fmt::Display for Vec<T> {
//===============================================================
//     // Error! Neither Display nor Vec is defined in this crate
//===============================================================
// }
```

The orphan rule prevents conflicts: either the trait or the type must be defined in your crate.

### Real-World Example: AsRef Pattern

The standard library's `AsRef` trait is a masterclass in blanket implementations:

```rust
trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

//=============================
// String implements AsRef<str>
//=============================
impl AsRef<str> for String {
    fn as_ref(&self) -> &str {
        self
    }
}

//===========================
// &str implements AsRef<str>
//===========================
impl AsRef<str> for str {
    fn as_ref(&self) -> &str {
        self
    }
}

//===========================================
// Functions can accept either String or &str
//===========================================
fn print_string<S: AsRef<str>>(s: S) {
    println!("{}", s.as_ref());
}

fn example() {
    print_string("hello");              // &str
    print_string("hello".to_string());  // String
}
```

This pattern makes APIs flexible without sacrificing type safety.

## Pattern 6: Advanced Trait Patterns

Let's explore some sophisticated patterns that combine these concepts.

### Builder Traits

Use traits to create type-safe builder patterns:

```rust
//===============================
// Type-state pattern with traits
//===============================
trait Buildable {
    type Built;
    fn build(self) -> Self::Built;
}

struct UserBuilder<Stage> {
    name: Option<String>,
    email: Option<String>,
    _stage: std::marker::PhantomData<Stage>,
}

struct Initial;
struct WithName;
struct WithEmail;

impl UserBuilder<Initial> {
    fn new() -> Self {
        UserBuilder {
            name: None,
            email: None,
            _stage: std::marker::PhantomData,
        }
    }

    fn with_name(self, name: String) -> UserBuilder<WithName> {
        UserBuilder {
            name: Some(name),
            email: None,
            _stage: std::marker::PhantomData,
        }
    }
}

impl UserBuilder<WithName> {
    fn with_email(self, email: String) -> UserBuilder<WithEmail> {
        UserBuilder {
            name: self.name,
            email: Some(email),
            _stage: std::marker::PhantomData,
        }
    }
}

impl Buildable for UserBuilder<WithEmail> {
    type Built = User;

    fn build(self) -> User {
        User {
            name: self.name.unwrap(),
            email: self.email.unwrap(),
        }
    }
}

struct User {
    name: String,
    email: String,
}

fn example() {
    let user = UserBuilder::new()
        .with_name("Alice".to_string())
        .with_email("alice@example.com".to_string())
        .build();

    // This won't compile (missing email):
    // UserBuilder::new()
    //     .with_name("Bob".to_string())
    //     .build();
}
```

The type system enforces that you can only build when all required fields are set.

### Sealed Traits

Prevent external implementations of your traits:

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn my_method(&self);
}

struct MyType;

impl sealed::Sealed for MyType {}
impl MyTrait for MyType {
    fn my_method(&self) {
        println!("Internal implementation");
    }
}

//=====================================================================================
// External crates cannot implement MyTrait because they can't implement sealed::Sealed
//=====================================================================================
```

This pattern ensures you can add methods to the trait without breaking compatibility.

### Dependency Injection with Traits

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

//=====================================
// Production uses real implementations
//=====================================
// Tests use mocks
```

This pattern makes code testable and modular.

## Summary

This chapter covered advanced trait design patterns for flexible, expressive Rust APIs:

1. **Trait Inheritance and Bounds**: Supertrait relationships, multiple bounds, where clauses, conditional implementations
2. **Associated Types vs Generics**: When to use each, ergonomics vs flexibility, Iterator pattern, type families
3. **Trait Objects and Dynamic Dispatch**: &dyn Trait, object safety, heterogeneous collections, vtable overhead
4. **Extension Traits**: Extending external types, blanket implementations, modular design, ecosystem interop
5. **Sealed Traits**: Preventing external impls, API evolution, safety guarantees, protocol enforcement

**Key Takeaways**:
- Trait inheritance expresses capabilities: "to be A must be B" is declarative and composable
- Associated types = one impl per type, inferred; generics = multiple impls, explicit choice
- Dynamic dispatch = smaller binary, ~2-3ns overhead; static dispatch = optimized per-type, code bloat
- Extension traits extend types you don't own via trait + impl, coherence allows (one of trait/type is yours)
- Sealed traits prevent external impls via private supertrait, enables non-breaking trait evolution

**Design Guidelines**:
- Supertraits for capability requirements: trait Loggable: Debug + Display
- Associated types when output determined by type, generics when chosen by caller
- Trait objects for heterogeneous collections, generics for performance
- Extension traits for opt-in functionality on external types
- Sealed traits when evolution/safety requires controlled implementations

**Performance Considerations**:
- Static dispatch: optimized per-type, inlined, zero cost, but code bloat
- Dynamic dispatch: vtable lookup ~2-3ns, not inlinable, one implementation
- Blanket implementations: zero runtime cost, compile-time feature provision
- Associated types vs generics: no performance difference, ergonomics differ
- Trait bounds: purely compile-time, zero runtime cost

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
mod sealed {
    pub trait Sealed {}
}
pub trait MyTrait: sealed::Sealed { /* ... */ }
impl sealed::Sealed for MyType {}
impl MyTrait for MyType { /* ... */ }

// Blanket implementation (extend all types)
impl<T: Iterator> IteratorExt for T {
    fn counts(self) -> HashMap<Self::Item, usize> { /* ... */ }
}
```

**Object Safety Rules** (for `&dyn Trait`):
- No generic methods (needs concrete type at compile-time)
- No Self: Sized bound (trait objects are !Sized)
- Must have &self/&mut self receiver (needs object to call)
- No associated functions without self (can't call without type)

**API Design Checklist**:
- Use supertraits to express capability requirements (clear dependencies)
- Choose associated types for determined outputs (better inference)
- Use generics for flexible inputs (multiple implementations)
- Provide trait objects when heterogeneity needed (dyn Trait)
- Create extension traits for external types (modular opt-in)
- Seal traits that need controlled evolution (future-proof)
- Document trait expectations (pre/post-conditions, invariants)
- Use where clauses for complex bounds (readability)

**When to Use What**:
- Supertraits: Capability composition, trait requirements
- Associated types: One implementation, determined by type
- Generics: Multiple implementations, chosen by caller
- Trait objects: Runtime polymorphism, heterogeneous collections
- Extension traits: Add methods to external types
- Sealed traits: Control implementations, API evolution
- Blanket impls: Provide functionality to all matching types
