### Struct Cheat Sheet
```rust
// ===== BASIC STRUCTS =====
// Named field struct
struct User {
    username: String,
    email: String,
    age: u32,
    active: bool,
}

// Create instance
fn basic_struct_example() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    // Access fields
    println!("Username: {}", user.username);
    println!("Email: {}", user.email);
}

// Mutable struct
fn mutable_struct() {
    let mut user = User {
        username: String::from("bob"),
        email: String::from("bob@example.com"),
        age: 25,
        active: true,
    };
    
    // Modify fields
    user.email = String::from("bob_new@example.com");
    user.age += 1;
}

// ===== TUPLE STRUCTS =====
// Tuple struct (fields without names)
struct Color(u8, u8, u8);
struct Point(i32, i32, i32);

fn tuple_struct_example() {
    let black = Color(0, 0, 0);
    let origin = Point(0, 0, 0);
    
    // Access by index
    println!("Red: {}", black.0);
    println!("X: {}", origin.0);
    
    // Pattern matching
    let Color(r, g, b) = black;
    println!("RGB: ({}, {}, {})", r, g, b);
}

// ===== UNIT-LIKE STRUCTS =====
// Struct with no fields
struct Marker;
struct AlwaysEqual;

fn unit_struct_example() {
    let marker = Marker;
    let equal1 = AlwaysEqual;
    let equal2 = AlwaysEqual;
}

// ===== FIELD INIT SHORTHAND =====
fn build_user(email: String, username: String) -> User {
    User {
        username,                                            // Shorthand when variable name matches field
        email,
        age: 0,
        active: true,
    }
}

// ===== STRUCT UPDATE SYNTAX =====
fn struct_update_example() {
    let user1 = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    // Create new struct using fields from another
    let user2 = User {
        email: String::from("alice2@example.com"),
        ..user1                                              // Copy remaining fields
    };
    
    // Note: user1.username and user1.email are moved to user2
    // Can still use user1.age and user1.active (Copy types)
}

// ===== STRUCT METHODS =====
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    // Method (takes &self)
    fn area(&self) -> u32 {
        self.width * self.height
    }
    
    // Method with mutable self
    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }
    
    // Method that takes ownership
    fn into_square(self) -> Square {
        Square {
            side: self.width.min(self.height),
        }
    }
    
    // Method with parameters
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

struct Square {
    side: u32,
}

fn method_example() {
    let mut rect = Rectangle {
        width: 30,
        height: 50,
    };
    
    println!("Area: {}", rect.area());
    
    rect.scale(2);
    println!("New width: {}", rect.width);
    
    let square = rect.into_square();
    // rect is now invalid (moved)
}

// ===== ASSOCIATED FUNCTIONS =====
impl Rectangle {
    // Associated function (no self parameter)
    fn new(width: u32, height: u32) -> Rectangle {
        Rectangle { width, height }
    }
    
    // Another constructor
    fn square(size: u32) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }
}

fn associated_function_example() {
    let rect = Rectangle::new(30, 50);
    let square = Rectangle::square(10);
}

// ===== MULTIPLE IMPL BLOCKS =====
impl Rectangle {
    fn perimeter(&self) -> u32 {
        2 * (self.width + self.height)
    }
}

impl Rectangle {
    fn is_square(&self) -> bool {
        self.width == self.height
    }
}

// ===== GENERIC STRUCTS =====
struct Point2D<T> {
    x: T,
    y: T,
}

impl<T> Point2D<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// Implementation for specific type
impl Point2D<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

// Multiple generic parameters
struct Point3D<T, U> {
    x: T,
    y: T,
    z: U,
}

impl<T, U> Point3D<T, U> {
    fn mixup<V, W>(self, other: Point3D<V, W>) -> Point3D<T, W> {
        Point3D {
            x: self.x,
            y: self.y,
            z: other.z,
        }
    }
}

fn generic_struct_example() {
    let integer_point = Point2D { x: 5, y: 10 };
    let float_point = Point2D { x: 1.0, y: 4.0 };
    
    let p1 = Point3D { x: 5, y: 10, z: 1.5 };
    let p2 = Point3D { x: "Hello", y: "World", z: 'c' };
    
    let p3 = p1.mixup(p2);
    // p3 has type Point3D<i32, char>
}

// ===== STRUCT WITH LIFETIMES =====
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

fn lifetime_struct_example() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().expect("Could not find a '.'");
    
    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };
}

// ===== PRIVATE AND PUBLIC FIELDS =====
mod my_module {
    pub struct PublicStruct {
        pub public_field: String,
        private_field: i32,                              // Private by default
    }
    
    impl PublicStruct {
        pub fn new(public_field: String) -> PublicStruct {
            PublicStruct {
                public_field,
                private_field: 0,
            }
        }
        
        pub fn get_private(&self) -> i32 {
            self.private_field
        }
    }
}

fn visibility_example() {
    let s = my_module::PublicStruct::new(String::from("hello"));
    println!("{}", s.public_field);
    // println!("{}", s.private_field);                 // ERROR: private field
}

// ===== DESTRUCTURING =====
fn destructuring_example() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    // Destructure all fields
    let User { username, email, age, active } = user;
    
    // Destructure some fields
    let User { username, email, .. } = user;
    
    // Rename fields during destructuring
    let User { username: name, email: mail, .. } = user;
}

// ===== DERIVE MACROS =====
#[derive(Debug)]                                         // Auto-implement Debug
struct DebugStruct {
    field: i32,
}

#[derive(Clone)]                                         // Auto-implement Clone
struct CloneStruct {
    data: Vec<i32>,
}

#[derive(Copy, Clone)]                                   // Copy requires Clone
struct CopyStruct {
    value: i32,
}

#[derive(PartialEq, Eq)]                                // Equality comparison
struct EqStruct {
    id: i32,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]               // Ordering
struct OrdStruct {
    priority: i32,
}

#[derive(Default)]                                       // Default values
struct DefaultStruct {
    count: i32,
    name: String,
}

#[derive(Hash)]                                          // Hashing
struct HashStruct {
    id: i32,
}

// Multiple derives
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MultiDeriveStruct {
    id: i32,
    name: String,
}

fn derive_example() {
    let s = DebugStruct { field: 42 };
    println!("{:?}", s);                                 // Uses Debug
    
    let default: DefaultStruct = Default::default();
    println!("{}", default.count);                       // 0
}

// ===== PATTERN MATCHING WITH STRUCTS =====
fn pattern_matching() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };
    
    match user {
        User { active: true, age, .. } => {
            println!("Active user, age: {}", age);
        }
        User { active: false, .. } => {
            println!("Inactive user");
        }
    }
    
    // If let pattern
    if let User { username, .. } = user {
        println!("Username: {}", username);
    }
}

// ===== BUILDER PATTERN =====
#[derive(Debug)]
struct Config {
    host: String,
    port: u16,
    timeout: u64,
    retries: u32,
}

struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u64>,
    retries: Option<u32>,
}

impl ConfigBuilder {
    fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
            timeout: None,
            retries: None,
        }
    }
    
    fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }
    
    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
    
    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
    
    fn retries(mut self, retries: u32) -> Self {
        self.retries = Some(retries);
        self
    }
    
    fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
            retries: self.retries.unwrap_or(3),
        })
    }
}

fn builder_example() {
    let config = ConfigBuilder::new()
        .host("localhost")
        .port(3000)
        .timeout(60)
        .build()
        .unwrap();
    
    println!("{:?}", config);
}

// ===== NEWTYPE PATTERN =====
struct Meters(f64);
struct Kilometers(f64);

impl Meters {
    fn to_kilometers(&self) -> Kilometers {
        Kilometers(self.0 / 1000.0)
    }
}

impl Kilometers {
    fn to_meters(&self) -> Meters {
        Meters(self.0 * 1000.0)
    }
}

fn newtype_example() {
    let distance = Meters(5000.0);
    let km = distance.to_kilometers();
    
    // Cannot accidentally mix Meters and Kilometers
    // let sum = Meters(100.0) + Kilometers(1.0);       // ERROR
}

// ===== INTERIOR MUTABILITY =====
use std::cell::{Cell, RefCell};

struct Counter {
    count: Cell<i32>,
}

impl Counter {
    fn new() -> Self {
        Counter {
            count: Cell::new(0),
        }
    }
    
    fn increment(&self) {                                // Takes &self, not &mut self
        self.count.set(self.count.get() + 1);
    }
    
    fn get(&self) -> i32 {
        self.count.get()
    }
}

struct Container {
    data: RefCell<Vec<i32>>,
}

impl Container {
    fn add(&self, value: i32) {                          // Takes &self
        self.data.borrow_mut().push(value);
    }
    
    fn get(&self, index: usize) -> Option<i32> {
        self.data.borrow().get(index).copied()
    }
}

// ===== ZERO-SIZED TYPES =====
struct ZeroSized;

fn zero_sized_example() {
    let zst = ZeroSized;
    println!("Size: {}", std::mem::size_of::<ZeroSized>());  // 0
    
    // Useful as markers or tokens
    let vec: Vec<ZeroSized> = vec![ZeroSized; 1000];
    // Takes no heap memory!
}

// ===== PHANTOM DATA =====
use std::marker::PhantomData;

struct PhantomStruct<T> {
    data: i32,
    _marker: PhantomData<T>,                             // Zero-sized
}

impl<T> PhantomStruct<T> {
    fn new(data: i32) -> Self {
        PhantomStruct {
            data,
            _marker: PhantomData,
        }
    }
}

// Different types even with same data
fn phantom_example() {
    let p1: PhantomStruct<i32> = PhantomStruct::new(42);
    let p2: PhantomStruct<String> = PhantomStruct::new(42);
    
    // p1 and p2 are different types
}

// ===== STRUCT WITH ARRAYS =====
struct Grid {
    cells: [[i32; 10]; 10],
}

impl Grid {
    fn new() -> Self {
        Grid {
            cells: [[0; 10]; 10],
        }
    }
    
    fn get(&self, x: usize, y: usize) -> i32 {
        self.cells[x][y]
    }
    
    fn set(&mut self, x: usize, y: usize, value: i32) {
        self.cells[x][y] = value;
    }
}

// ===== STRUCT WITH FUNCTIONS =====
struct Operation {
    name: String,
    func: fn(i32, i32) -> i32,
}

impl Operation {
    fn execute(&self, a: i32, b: i32) -> i32 {
        (self.func)(a, b)
    }
}

fn struct_with_function_example() {
    let add_op = Operation {
        name: String::from("add"),
        func: |a, b| a + b,
    };
    
    println!("Result: {}", add_op.execute(5, 3));
}

// ===== CONST GENERICS =====
struct FixedArray<T, const N: usize> {
    data: [T; N],
}

impl<T: Default + Copy, const N: usize> FixedArray<T, N> {
    fn new() -> Self {
        FixedArray {
            data: [T::default(); N],
        }
    }
}

fn const_generic_example() {
    let arr: FixedArray<i32, 5> = FixedArray::new();
    let arr2: FixedArray<i32, 10> = FixedArray::new();
    
    // arr and arr2 are different types
}

// ===== SELF-REFERENTIAL STRUCTS =====
use std::pin::Pin;

// Simple self-referential (unsafe)
struct SelfRef {
    data: String,
    ptr: *const String,                                  // Points to data
}

impl SelfRef {
    fn new(s: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfRef {
            data: s,
            ptr: std::ptr::null(),
        });
        
        let ptr = &boxed.data as *const String;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).ptr = ptr;
        }
        
        boxed
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: State machine using structs
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        Door { _state: PhantomData }
    }
    
    fn unlock(self) -> Door<Unlocked> {
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }
    
    fn open(&self) {
        println!("Door opened");
    }
}

fn state_machine_example() {
    let door = Door::<Locked>::new();
    let door = door.unlock();
    door.open();
    let door = door.lock();
    // door.open();                                     // ERROR: door is locked
}

// Pattern 2: Wrapper for external types
struct Wrapper(Vec<i32>);

impl Wrapper {
    fn sum(&self) -> i32 {
        self.0.iter().sum()
    }
}

// Pattern 3: Type-safe IDs
struct UserId(u32);
struct PostId(u32);

fn get_user(id: UserId) -> String {
    format!("User {}", id.0)
}

fn id_example() {
    let user_id = UserId(1);
    let post_id = PostId(1);
    
    get_user(user_id);
    // get_user(post_id);                               // ERROR: type mismatch
}

// Pattern 4: Cached computation
struct Cached<T> {
    value: Option<T>,
    computation: fn() -> T,
}

impl<T> Cached<T> {
    fn new(computation: fn() -> T) -> Self {
        Cached {
            value: None,
            computation,
        }
    }
    
    fn get(&mut self) -> &T {
        if self.value.is_none() {
            self.value = Some((self.computation)());
        }
        self.value.as_ref().unwrap()
    }
}

// Pattern 5: Struct with validation
struct Email {
    address: String,
}

impl Email {
    fn new(address: String) -> Result<Self, String> {
        if address.contains('@') {
            Ok(Email { address })
        } else {
            Err("Invalid email format".to_string())
        }
    }
    
    fn as_str(&self) -> &str {
        &self.address
    }
}

// Pattern 6: Composed structs
struct Address {
    street: String,
    city: String,
    country: String,
}

struct Person {
    name: String,
    address: Address,
}

// Pattern 7: Optional fields with builder
#[derive(Default)]
struct Options {
    verbose: bool,
    debug: bool,
    output: Option<String>,
}

impl Options {
    fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
    
    fn debug(mut self) -> Self {
        self.debug = true;
        self
    }
    
    fn output(mut self, path: String) -> Self {
        self.output = Some(path);
        self
    }
}

fn options_example() {
    let opts = Options::default()
        .verbose()
        .debug()
        .output(String::from("output.txt"));
}

// Pattern 8: Tagged union (enum alternative)
struct Tagged<T> {
    tag: String,
    data: T,
}

impl<T> Tagged<T> {
    fn new(tag: &str, data: T) -> Self {
        Tagged {
            tag: tag.to_string(),
            data,
        }
    }
}
```

