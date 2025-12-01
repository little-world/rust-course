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
