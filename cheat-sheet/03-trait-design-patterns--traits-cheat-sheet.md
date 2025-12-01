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
