# Chapter 2: Type System Pattern

Pattern 1: Newtype Pattern for Type Safety

- Problem: Type aliases provide no safety; unit confusion bugs have
  crashed spacecraft
- Solution: Single-field structs create distinct types at compile time
- Why It Matters: Zero-cost compile-time bug prevention for domain types
- Use Cases: IDs, units, validated strings, API boundaries

Pattern 2: Phantom Types and Zero-Cost State Machines

- Problem: Runtime state machines allow invalid transitions
- Solution: Zero-sized type parameters encode states; transitions consume
  self
- Why It Matters: Security vulnerabilities and crashes become impossible
- Use Cases: Protocols, resource lifecycle, builders, permission systems

Pattern 3: Generic Associated Types (GATs)

- Problem: Can't express lending iterators or async traits with lifetimes
- Solution: Associated types with generic parameters (lifetimes)
- Why It Matters: Unlocks zero-allocation patterns previously impossible
- Use Cases: Lending iterators, database rows, async traits, pointer
  families

Pattern 4: Type-Level Programming with Const Generics

- Problem: Can't be generic over array sizes; dimension errors at runtime
- Solution: Type parameters for constant values enable compile-time sizes
- Why It Matters: Type-safe stack allocation for embedded/real-time
  systems
- Use Cases: Fixed buffers, matrices, SIMD, cryptography, protocols

Pattern 5: Trait Object Optimization

- Problem: Static dispatch bloats binaries; dynamic dispatch adds overhead
- Solution: Strategic use of trait objects with enum dispatch optimization
- Why It Matters: 1-10ns per call compounds to 20% overhead in hot paths
- Use Cases: Plugins, GUI, game engines, serialization, middleware

Pattern 6: Associated Types vs Generic Parameters

- Problem: Wrong choice makes APIs painful to use
- Solution: Associated types for single outputs, generics for multiple
  impls
- Why It Matters: Decision shapes entire API surface and ergonomics
- Use Cases: Iterators (associated), conversions (generic), mixed
  approaches

## Overview

Rust's type system is one of the most sophisticated in any mainstream programming language. It combines the expressiveness of ML-family languages with zero-cost abstractions, enabling you to encode invariants at the type level that would otherwise require runtime checks or extensive documentation.

This chapter explores advanced type system patterns that experienced programmers can leverage to write safer, more maintainable code. The key insight is that Rust's type system allows you to move validation from runtime to compile-time, catching entire classes of bugs before your code ever runs.

The patterns we'll explore include:
- Using newtypes to prevent mixing incompatible values
- Encoding state machines in the type system
- Generic Associated Types (GATs) for higher-kinded abstractions
- Type-level programming with const generics
- Optimizing trait objects for dynamic dispatch

## Type System Foundation

```rust
// Core type system concepts
struct Point<T> { x: T, y: T }           // Generic structs
enum Option<T> { Some(T), None }         // Generic enums
trait Display { fn fmt(&self); }         // Traits (interfaces)
impl Display for Point<i32> { }          // Trait implementation

// Advanced features
type Meters = f64;                       // Type alias (no safety)
struct Meters(f64);                      // Newtype (type safety)
struct State<S> { _marker: PhantomData<S> }  // Phantom types
trait Container { type Item; }           // Associated types
trait Lending { type Item<'a>; }         // GATs (Generic Associated Types)

// Trait objects and dynamic dispatch
Box<dyn Display>                         // Heap-allocated trait object
&dyn Display                             // Reference to trait object
```

## Pattern 1: Newtype Pattern for Type Safety

**Problem**: Type aliases (`type UserId = u64`) provide no type safety—you can accidentally pass a `ProductId` where a `UserId` is expected, and the compiler won't catch the error. Primitive types like `f64` can represent meters, feet, or seconds, leading to unit confusion bugs (remember the Mars Climate Orbiter crash?). Email addresses and URLs are just strings, but invalid ones can bypass validation.

**Solution**: Wrap values in single-field structs (newtypes). `struct UserId(u64)` and `struct ProductId(u64)` are distinct types despite identical representation. The compiler prevents mixing them. Make fields private and provide validated constructors to enforce invariants.

**Why It Matters**: Domain-specific types catch bugs at compile time that would otherwise manifest as runtime errors or silent data corruption. Unit confusion has caused spacecraft failures and medical dosing errors. Email validation bugs enable SQL injection. Newtypes eliminate these entire categories of bugs with zero runtime cost—the compiler erases the wrapper.

**Use Cases**: Domain identifiers (UserId, OrderId, SessionToken), units of measurement (Meters, Kilograms, Celsius), validated strings (Email, PhoneNumber, URL), API boundaries requiring opaque handles, preventing index confusion between different collections.

```rust
//========================================
// Problem: Type aliases provide no safety
//========================================
type UserId = u64;
type ProductId = u64;

fn get_user(id: UserId) -> User { /* ... */ User }
fn get_product(id: ProductId) -> Product { /* ... */ Product }

// Compiles but wrong!
let user_id: UserId = 42;
let product = get_product(user_id);  // Type error not caught!

//==========================
// Solution: Newtype pattern
//==========================
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UserId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ProductId(u64);

fn get_user_safe(id: UserId) -> User { /* ... */ User }
fn get_product_safe(id: ProductId) -> Product { /* ... */ Product }

// Now this won't compile!
let user_id = UserId(42);
// let product = get_product_safe(user_id);  // Compile error!

//==============================================
// Pattern: Implement Deref for ergonomic access
//==============================================
use std::ops::Deref;

impl Deref for UserId {
    type Target = u64;
    fn deref(&self) -> &u64 {
        &self.0
    }
}

//====================================================
// Pattern: Validated construction with private fields
//====================================================
pub struct Email(String);

impl Email {
    pub fn new(s: String) -> Result<Self, &'static str> {
        if s.contains('@') && s.contains('.') {
            Ok(Email(s))
        } else {
            Err("Invalid email format")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Cannot construct invalid Email
// let invalid = Email("not-an-email".to_string());  // Private field!
let valid = Email::new("user@example.com".to_string()).unwrap();

//============================================
// Pattern: Unit newtypes for different scales
//============================================
#[derive(Debug, Clone, Copy)]
struct Meters(f64);

#[derive(Debug, Clone, Copy)]
struct Feet(f64);

impl Meters {
    fn to_feet(self) -> Feet {
        Feet(self.0 * 3.28084)
    }
}

impl Feet {
    fn to_meters(self) -> Meters {
        Meters(self.0 / 3.28084)
    }
}

fn calculate_area(width: Meters, height: Meters) -> f64 {
    width.0 * height.0
}

// Won't compile: prevents mixing units
// let area = calculate_area(Meters(10.0), Feet(5.0));

//============================================
// Pattern: Opaque newtypes for API boundaries
//============================================
pub struct Token(String);

impl Token {
    pub(crate) fn new(s: String) -> Self {
        Token(s)
    }

    // No way to extract the string from outside the crate
}

pub fn authenticate(username: &str, password: &str) -> Option<Token> {
    // Validation logic
    if username.len() > 3 && password.len() > 8 {
        Some(Token::new(format!("{}:{}", username, password)))
    } else {
        None
    }
}

pub fn authorize(token: &Token) -> bool {
    // Can only be called with a properly constructed Token
    !token.0.is_empty()
}

//======================================================================
// Pattern: Newtype for index types (prevents indexing wrong collection)
//======================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StudentId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CourseId(usize);

struct Database {
    students: Vec<Student>,
    courses: Vec<Course>,
}

impl Database {
    fn get_student(&self, id: StudentId) -> Option<&Student> {
        self.students.get(id.0)
    }

    fn get_course(&self, id: CourseId) -> Option<&Course> {
        self.courses.get(id.0)
    }
}

struct Student;
struct Course;
struct User;
struct Product;
```

**When to use newtypes:**
- Domain modeling with distinct types (UserId, ProductId, Email)
- Units of measure (Meters, Seconds, Celsius)
- Validated strings (Email, URL, PhoneNumber)
- API boundaries where you want to prevent misuse
- Preventing index confusion between collections

**Performance characteristics:**
- Zero runtime cost (optimized away by compiler)
- Same memory layout as wrapped type
- No vtable or indirection
- Perfect for performance-critical code

## Pattern 2: Phantom Types and Zero-Cost State Machines

**Problem**: State machines implemented with enums or flags allow invalid state transitions. You can call `door.open()` when it's locked, `connection.send()` before authentication, or `builder.build()` with missing required fields. Runtime checks add overhead and can still be bypassed. Documentation of valid state transitions is ignored.

**Solution**: Encode states as zero-sized types (phantom types) and make transitions consume `self`, returning new state types. `Door<Locked>` has `unlock() -> Door<Unlocked>`, but no `open()` method. Invalid transitions don't compile. Use `PhantomData<State>` to embed type-level state without runtime storage.

**Why It Matters**: Invalid state transitions cause security vulnerabilities (sending data over unencrypted connections), data corruption (modifying committed transactions), and crashes (using closed file handles). The typestate pattern makes these impossible—if it compiles, the state machine is valid. This is zero-cost abstraction perfected: state tracking with no runtime overhead.

**Use Cases**: Protocol implementations (TCP state machine, TLS handshake), resource lifecycle (file handles, database transactions), builder patterns with required fields, permission systems (authenticated vs unauthenticated users), dimensional analysis in physics calculations.

```rust
use std::marker::PhantomData;

//==============================================
// Pattern: Typestate pattern for state machines
//==============================================
struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        println!("Door created in locked state");
        Door { _state: PhantomData }
    }

    fn unlock(self) -> Door<Unlocked> {
        println!("Door unlocked");
        Door { _state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn lock(self) -> Door<Locked> {
        println!("Door locked");
        Door { _state: PhantomData }
    }

    fn open(&self) {
        println!("Door opened");
    }
}

//================================================
// Usage: Invalid state transitions don't compile!
//================================================
fn door_example() {
    let door = Door::<Locked>::new();
    // door.open();  // Compile error: method not available
    let door = door.unlock();
    door.open();  // OK
    let door = door.lock();
    // door.open();  // Compile error again
}

//==============================================
// Pattern: Builder with compile-time validation
//==============================================
struct Unset;
struct Set<T>(T);

struct HttpRequestBuilder<Url, Method, Body> {
    url: Url,
    method: Method,
    body: Body,
}

impl HttpRequestBuilder<Unset, Unset, Unset> {
    fn new() -> Self {
        HttpRequestBuilder {
            url: Unset,
            method: Unset,
            body: Unset,
        }
    }
}

impl<Method, Body> HttpRequestBuilder<Unset, Method, Body> {
    fn url(self, url: String) -> HttpRequestBuilder<Set<String>, Method, Body> {
        HttpRequestBuilder {
            url: Set(url),
            method: self.method,
            body: self.body,
        }
    }
}

impl<Url, Body> HttpRequestBuilder<Url, Unset, Body> {
    fn method(self, method: String) -> HttpRequestBuilder<Url, Set<String>, Body> {
        HttpRequestBuilder {
            url: self.url,
            method: Set(method),
            body: self.body,
        }
    }
}

impl<Url, Method> HttpRequestBuilder<Url, Method, Unset> {
    fn body(self, body: String) -> HttpRequestBuilder<Url, Method, Set<String>> {
        HttpRequestBuilder {
            url: self.url,
            method: self.method,
            body: Set(body),
        }
    }
}

// Only valid when all fields are set
impl HttpRequestBuilder<Set<String>, Set<String>, Set<String>> {
    fn build(self) -> HttpRequest {
        HttpRequest {
            url: self.url.0,
            method: self.method.0,
            body: self.body.0,
        }
    }
}

struct HttpRequest {
    url: String,
    method: String,
    body: String,
}

//===============================================
// Usage: Cannot build without setting all fields
//===============================================
fn request_example() {
    let request = HttpRequestBuilder::new()
        .url("https://api.example.com".to_string())
        .method("POST".to_string())
        .body("{}".to_string())
        .build();  // Only compiles when all fields set

    // Won't compile:
    // let incomplete = HttpRequestBuilder::new()
    //     .url("https://api.example.com".to_string())
    //     .build();  // Error: method not found
}

//============================================
// Pattern: Units and dimensions at type level
//============================================
struct Dimension<const MASS: i32, const LENGTH: i32, const TIME: i32>;

type Scalar = Dimension<0, 0, 0>;
type Length = Dimension<0, 1, 0>;
type Time = Dimension<0, 0, 1>;
type Velocity = Dimension<0, 1, -1>;  // Length / Time
type Acceleration = Dimension<0, 1, -2>;  // Length / Time^2

struct Quantity<T, D> {
    value: T,
    _dimension: PhantomData<D>,
}

impl<T, D> Quantity<T, D> {
    fn new(value: T) -> Self {
        Quantity { value, _dimension: PhantomData }
    }
}

// Addition only for same dimensions
impl<T: std::ops::Add<Output = T>, D> std::ops::Add for Quantity<T, D> {
    type Output = Quantity<T, D>;

    fn add(self, rhs: Self) -> Self::Output {
        Quantity::new(self.value + rhs.value)
    }
}

// Multiplication combines dimensions
impl<T, const M1: i32, const L1: i32, const T1: i32, const M2: i32, const L2: i32, const T2: i32>
    std::ops::Mul<Quantity<T, Dimension<M2, L2, T2>>>
    for Quantity<T, Dimension<M1, L1, T1>>
where
    T: std::ops::Mul<Output = T>
{
    type Output = Quantity<T, Dimension<{M1 + M2}, {L1 + L2}, {T1 + T2}>>;

    fn mul(self, rhs: Quantity<T, Dimension<M2, L2, T2>>) -> Self::Output {
        Quantity::new(self.value * rhs.value)
    }
}

fn physics_example() {
    let distance = Quantity::<f64, Length>::new(100.0);
    let time = Quantity::<f64, Time>::new(10.0);

    // This compiles: same dimensions
    let total_distance = distance + Quantity::<f64, Length>::new(50.0);

    // This won't compile: different dimensions
    // let invalid = distance + time;

    // Division would give velocity (if we implemented Div)
}

//================================
// Pattern: Protocol state machine
//================================
struct Disconnected;
struct Connected;
struct Authenticated;

struct TcpConnection<State> {
    socket: String,  // Simplified
    _state: PhantomData<State>,
}

impl TcpConnection<Disconnected> {
    fn new(address: String) -> Self {
        TcpConnection {
            socket: address,
            _state: PhantomData,
        }
    }

    fn connect(self) -> Result<TcpConnection<Connected>, std::io::Error> {
        println!("Connecting to {}", self.socket);
        Ok(TcpConnection {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl TcpConnection<Connected> {
    fn authenticate(self, _password: &str) -> Result<TcpConnection<Authenticated>, &'static str> {
        println!("Authenticating...");
        Ok(TcpConnection {
            socket: self.socket,
            _state: PhantomData,
        })
    }

    fn disconnect(self) -> TcpConnection<Disconnected> {
        TcpConnection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}

impl TcpConnection<Authenticated> {
    fn send_data(&self, data: &[u8]) {
        println!("Sending {} bytes", data.len());
    }

    fn disconnect(self) -> TcpConnection<Disconnected> {
        TcpConnection {
            socket: self.socket,
            _state: PhantomData,
        }
    }
}
```

**When to use phantom types:**
- State machines where invalid transitions should be impossible
- Protocol implementations with sequential states
- Type-level tracking of capabilities or permissions
- Units and dimensional analysis
- Builders requiring all fields before construction

**Performance characteristics:**
- Absolute zero runtime cost
- PhantomData is zero-sized
- All checks are compile-time only
- No vtables, no runtime checks, no overhead

## Pattern 3: Generic Associated Types (GATs)

**Problem**: Standard `Iterator` requires `Item` to have a `'static` lifetime or outlive the iterator, making lending iterators (that return references into `self`) impossible. Database libraries can't return borrowed rows. Async traits with lifetime parameters were inexpressible before GATs. Generic containers couldn't abstract over borrowed vs owned access patterns.

**Solution**: Generic Associated Types allow associated types to have their own generic parameters, typically lifetimes. `type Item<'a> where Self: 'a` lets iterators return references tied to `&mut self`'s lifetime. This enables lending iterators, async traits with borrowing, and database abstractions that avoid allocations.

**Why It Matters**: Without GATs, iterators over windows (`&[T]` slices) were impossible—you'd need allocation or unsafe code. Database queries had to clone every row instead of borrowing. Async functions in traits required the `async-trait` crate with heap allocations. GATs unlock zero-allocation patterns that were previously language limitations.

**Use Cases**: Lending iterators (windows, streaming parsers returning borrowed chunks), database libraries with borrowed rows, async traits (now stabilized), generic pointer families (abstraction over Box/Rc/Arc), effect systems and monad-like patterns.

```rust
//============================================================
// Pattern: Lending iterator (Iterator that borrows from self)
//============================================================
trait LendingIterator {
    type Item<'a> where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

//==========================================================
// Example: Windows iterator that returns overlapping slices
//==========================================================
struct Windows<'data, T> {
    slice: &'data [T],
    size: usize,
    position: usize,
}

impl<'data, T> LendingIterator for Windows<'data, T> {
    type Item<'a> = &'a [T] where Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.position + self.size <= self.slice.len() {
            let window = &self.slice[self.position..self.position + self.size];
            self.position += 1;
            Some(window)
        } else {
            None
        }
    }
}

//================================================
// Pattern: Generic container with borrowed access
//================================================
trait Container {
    type Item<'a> where Self: 'a;

    fn get(&self, index: usize) -> Option<Self::Item<'_>>;
}

struct VecContainer<T> {
    data: Vec<T>,
}

impl<T> Container for VecContainer<T> {
    type Item<'a> = &'a T where Self: 'a;

    fn get(&self, index: usize) -> Option<Self::Item<'_>> {
        self.data.get(index)
    }
}

//=================================================================
// Pattern: Async trait simulation (before async traits stabilized)
//=================================================================
trait AsyncRead {
    type ReadFuture<'a>: std::future::Future<Output = std::io::Result<usize>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a>;
}

//=================================
// Pattern: Family of related types
//=================================
trait Database {
    type Row<'a> where Self: 'a;
    type Transaction<'a> where Self: 'a;

    fn query<'a>(&'a self, sql: &str) -> Vec<Self::Row<'a>>;
    fn transaction<'a>(&'a mut self) -> Self::Transaction<'a>;
}

struct PostgresDb;
struct PostgresRow<'a> {
    data: &'a str,
}
struct PostgresTransaction<'a> {
    db: &'a mut PostgresDb,
}

impl Database for PostgresDb {
    type Row<'a> = PostgresRow<'a>;
    type Transaction<'a> = PostgresTransaction<'a>;

    fn query<'a>(&'a self, _sql: &str) -> Vec<Self::Row<'a>> {
        vec![PostgresRow { data: "row1" }]
    }

    fn transaction<'a>(&'a mut self) -> Self::Transaction<'a> {
        PostgresTransaction { db: self }
    }
}

//======================================
// Pattern: Pointer-like trait with GATs
//======================================
trait PointerFamily {
    type Pointer<T>: std::ops::Deref<Target = T>;

    fn new<T>(value: T) -> Self::Pointer<T>;
}

struct BoxFamily;

impl PointerFamily for BoxFamily {
    type Pointer<T> = Box<T>;

    fn new<T>(value: T) -> Self::Pointer<T> {
        Box::new(value)
    }
}

struct RcFamily;

impl PointerFamily for RcFamily {
    type Pointer<T> = std::rc::Rc<T>;

    fn new<T>(value: T) -> Self::Pointer<T> {
        std::rc::Rc::new(value)
    }
}

// Generic code over pointer types
fn create_container<P: PointerFamily>(value: i32) -> P::Pointer<i32> {
    P::new(value)
}

//==================================
// Pattern: Effect system simulation
//==================================
trait Effect {
    type Output<T>;

    fn pure<T>(value: T) -> Self::Output<T>;
    fn bind<A, B, F>(effect: Self::Output<A>, f: F) -> Self::Output<B>
    where
        F: FnOnce(A) -> Self::Output<B>;
}

struct OptionEffect;

impl Effect for OptionEffect {
    type Output<T> = Option<T>;

    fn pure<T>(value: T) -> Self::Output<T> {
        Some(value)
    }

    fn bind<A, B, F>(effect: Self::Output<A>, f: F) -> Self::Output<B>
    where
        F: FnOnce(A) -> Self::Output<B>
    {
        effect.and_then(f)
    }
}

struct ResultEffect<E> {
    _error: PhantomData<E>,
}

impl<E> Effect for ResultEffect<E> {
    type Output<T> = Result<T, E>;

    fn pure<T>(value: T) -> Self::Output<T> {
        Ok(value)
    }

    fn bind<A, B, F>(effect: Self::Output<A>, f: F) -> Self::Output<B>
    where
        F: FnOnce(A) -> Self::Output<B>
    {
        effect.and_then(f)
    }
}
```

**When to use GATs:**
- Lending iterators (iterators that borrow from self)
- Async traits with lifetimes
- Database libraries with borrowed rows
- Effect systems and monadic patterns
- Generic abstractions over pointer types

**Complexity considerations:**
- GATs are complex and can make code harder to understand
- Error messages can be cryptic
- Use only when simpler patterns don't suffice
- Document thoroughly

## Pattern 4: Type-Level Programming with Const Generics

**Problem**: Fixed-size arrays (`[T; N]`) couldn't be generic over size before const generics—every size needed separate code. Matrix operations with incompatible dimensions (multiplying 2x3 by 5x7) are runtime errors instead of compile errors. Embedded systems need stack allocation but `Vec<T>` requires heap. Ring buffers and circular queues have to use `Vec` even when size is known at compile time.

**Solution**: Const generics let types be parameterized by constant values: `struct Buffer<const N: usize>`. Matrix multiplication enforces dimensional compatibility at compile time: `Matrix<M, N> * Matrix<N, P> -> Matrix<M, P>`. Arrays of any size work with generic code, enabling stack allocation with compile-time size checking.

**Why It Matters**: Dynamic allocation is forbidden in many embedded systems, real-time systems, and kernels. Dimension errors in linear algebra cause subtle bugs (machine learning models, graphics, physics simulations). Const generics enable type-safe stack allocation with zero runtime overhead—array bounds and dimensions checked at compile time, not runtime.

**Use Cases**: Embedded systems with fixed buffers, linear algebra (matrices, vectors with compile-time dimensions), SIMD operations (Vec3, Vec4), cryptography (fixed-size hashes, keys), network protocol buffers with fixed fields, ring buffers in real-time systems.

```rust
//===================================================
// Pattern: Fixed-size arrays without heap allocation
//===================================================
struct FixedBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> FixedBuffer<N> {
    fn new() -> Self {
        FixedBuffer {
            data: [0; N],
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) -> Result<(), &'static str> {
        if self.len < N {
            self.data[self.len] = byte;
            self.len += 1;
            Ok(())
        } else {
            Err("Buffer full")
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

// Different sizes are different types
fn buffer_example() {
    let mut small: FixedBuffer<16> = FixedBuffer::new();
    let mut large: FixedBuffer<4096> = FixedBuffer::new();

    // Cannot assign: different types!
    // small = large;
}

//=============================================
// Pattern: Matrix with compile-time dimensions
//=============================================
#[derive(Debug)]
struct Matrix<T, const ROWS: usize, const COLS: usize> {
    data: [[T; COLS]; ROWS],
}

impl<T: Default + Copy, const ROWS: usize, const COLS: usize> Matrix<T, ROWS, COLS> {
    fn new() -> Self {
        Matrix {
            data: [[T::default(); COLS]; ROWS],
        }
    }

    fn get(&self, row: usize, col: usize) -> Option<&T> {
        self.data.get(row)?.get(col)
    }

    fn set(&mut self, row: usize, col: usize, value: T) {
        if row < ROWS && col < COLS {
            self.data[row][col] = value;
        }
    }
}

// Matrix multiplication with compile-time dimension checking
impl<T, const M: usize, const N: usize, const P: usize> Matrix<T, M, N>
where
    T: Default + Copy + std::ops::Add<Output = T> + std::ops::Mul<Output = T>
{
    fn multiply(&self, other: &Matrix<T, N, P>) -> Matrix<T, M, P> {
        let mut result = Matrix::new();

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

fn matrix_example() {
    let a: Matrix<i32, 2, 3> = Matrix::new();
    let b: Matrix<i32, 3, 4> = Matrix::new();
    let c: Matrix<i32, 2, 4> = a.multiply(&b);  // Types enforce valid multiplication

    // Won't compile: dimension mismatch
    // let invalid = b.multiply(&a);
}

//====================================================
// Pattern: Type-level numbers for small integer types
//====================================================
struct SmallInt<const N: u8>(u8);

impl<const N: u8> SmallInt<N> {
    fn new(value: u8) -> Result<Self, &'static str> {
        if value < N {
            Ok(SmallInt(value))
        } else {
            Err("Value exceeds maximum")
        }
    }

    fn get(&self) -> u8 {
        self.0
    }
}

// Different maxima are different types
type Percentage = SmallInt<101>;  // 0-100
type DayOfMonth = SmallInt<32>;   // 1-31

//======================================================
// Pattern: SIMD-like operations with compile-time sizes
//======================================================
#[derive(Debug, Clone, Copy)]
struct Vec3<T>([T; 3]);

#[derive(Debug, Clone, Copy)]
struct Vec4<T>([T; 4]);

trait VecOps<T, const N: usize> {
    fn dot(&self, other: &Self) -> T;
    fn magnitude_squared(&self) -> T;
}

impl<T> VecOps<T, 3> for Vec3<T>
where
    T: Copy + std::ops::Mul<Output = T> + std::ops::Add<Output = T> + Default
{
    fn dot(&self, other: &Self) -> T {
        self.0[0] * other.0[0] +
        self.0[1] * other.0[1] +
        self.0[2] * other.0[2]
    }

    fn magnitude_squared(&self) -> T {
        self.dot(self)
    }
}

//============================================
// Pattern: Ring buffer with compile-time size
//============================================
struct RingBuffer<T, const N: usize> {
    data: [Option<T>; N],
    read: usize,
    write: usize,
    count: usize,
}

impl<T, const N: usize> RingBuffer<T, N>
where
    T: Copy + Default
{
    fn new() -> Self {
        RingBuffer {
            data: [None; N],
            read: 0,
            write: 0,
            count: 0,
        }
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.count == N {
            Err(value)
        } else {
            self.data[self.write] = Some(value);
            self.write = (self.write + 1) % N;
            self.count += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let value = self.data[self.read].take();
            self.read = (self.read + 1) % N;
            self.count -= 1;
            value
        }
    }
}

//=======================================
// Pattern: Compile-time array operations
//=======================================
fn sum_array<const N: usize>(arr: [i32; N]) -> i32 {
    arr.iter().sum()
}

// Works with any size
fn array_operations() {
    let small = sum_array([1, 2, 3]);
    let large = sum_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}
```

**When to use const generics:**
- Fixed-size collections (stack-allocated)
- Linear algebra with compile-time dimensions
- Embedded systems with size constraints
- SIMD-like operations
- Protocol buffers with fixed-size fields
- Ring buffers and circular queues

**Performance characteristics:**
- Zero heap allocation
- Size known at compile time
- Optimizes to same code as hand-written arrays
- No dynamic bounds checking overhead

## Pattern 5: Trait Object Optimization

**Problem**: Monomorphization (static dispatch with generics) generates specialized code for every concrete type, bloating binary size. Plugin systems need to load code at runtime, impossible with static dispatch. Heterogeneous collections of different types implementing the same trait can't use `Vec<T>` with a single `T`. But trait objects (`dyn Trait`) add overhead: vtable indirection prevents inlining and can cause cache misses.

**Solution**: Use trait objects (`Box<dyn Trait>`, `&dyn Trait`) for dynamic dispatch when needed, but optimize strategically. Use enum dispatch when types are known and closed. Cache trait method results. Minimize trait object passing in hot loops. Consider function pointers for simple cases. Make traits object-safe by avoiding generic methods and `Self` returns.

**Why It Matters**: Trait objects enable plugin systems, heterogeneous collections, and runtime polymorphism, but add 1-10ns overhead per call from vtable lookup and prevent inlining. In hot paths processing millions of items, this compounds to seconds of overhead. Understanding when to use static vs dynamic dispatch determines whether your abstraction costs 0% or 20% performance.

**Use Cases**: Plugin systems and dynamic loading, GUI frameworks with heterogeneous widgets, game engines with component systems, serialization frameworks, middleware/handler chains in web frameworks, configuration-driven behavior.

```rust
//=============================
// Pattern: Trait object basics
//=============================
trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

// Dynamic dispatch with trait objects
fn draw_shapes(shapes: &[Box<dyn Drawable>]) {
    for shape in shapes {
        shape.draw();  // Virtual function call
    }
}

//=======================================================
// Pattern: Minimize trait object size with thin pointers
//=======================================================
// Bad: Wide trait objects (multiple vtable pointers)
trait BadTrait: std::fmt::Debug + Clone + Send {}

// Good: Single trait, compose at usage site
trait GoodTrait: Send {}

fn process<T: GoodTrait + std::fmt::Debug>(value: T) {
    // Use trait bounds instead of multi-trait objects
}

//===============================================
// Pattern: Object-safe vs non-object-safe traits
//===============================================
// Object-safe: Can be made into trait object
trait ObjectSafe {
    fn method(&self);  // Takes &self
}

// Not object-safe: Generic methods
trait NotObjectSafe {
    fn generic<T>(&self, value: T);  // Can't be called on dyn NotObjectSafe
}

// Not object-safe: Returns Self
trait AlsoNotObjectSafe {
    fn clone(&self) -> Self;  // Self size unknown in trait object
}

//===================================
// Pattern: Making traits object-safe
//===================================
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

impl<T: Clone + 'static> Cloneable for T {
    fn clone_box(&self) -> Box<dyn Cloneable> {
        Box::new(self.clone())
    }
}

//================================================================
// Pattern: Enum dispatch instead of trait objects (when possible)
//================================================================
enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

impl Shape {
    fn draw(&self) {
        match self {
            Shape::Circle(c) => c.draw(),
            Shape::Rectangle(r) => r.draw(),
        }
    }
}

// Enum dispatch is faster: no vtable lookup
fn draw_shapes_fast(shapes: &[Shape]) {
    for shape in shapes {
        shape.draw();  // Direct call, compiler can inline
    }
}

//=====================================================
// Pattern: Small vector optimization for trait objects
//=====================================================
use std::mem;

enum SmallVec<T> {
    Inline([T; 3], usize),
    Heap(Vec<T>),
}

impl<T: Default + Copy> SmallVec<T> {
    fn new() -> Self {
        SmallVec::Inline([T::default(); 3], 0)
    }

    fn push(&mut self, value: T) {
        match self {
            SmallVec::Inline(arr, len) if *len < 3 => {
                arr[*len] = value;
                *len += 1;
            }
            SmallVec::Inline(arr, len) => {
                let mut vec = arr[..*len].to_vec();
                vec.push(value);
                *self = SmallVec::Heap(vec);
            }
            SmallVec::Heap(vec) => {
                vec.push(value);
            }
        }
    }
}

//===================================================
// Pattern: Trait object with static dispatch wrapper
//===================================================
trait Operation {
    fn execute(&self) -> i32;
}

struct Add(i32, i32);
impl Operation for Add {
    fn execute(&self) -> i32 { self.0 + self.1 }
}

struct Multiply(i32, i32);
impl Operation for Multiply {
    fn execute(&self) -> i32 { self.0 * self.1 }
}

// Static dispatch: monomorphization
fn execute_static<T: Operation>(op: &T) -> i32 {
    op.execute()  // Inlined
}

// Dynamic dispatch: trait object
fn execute_dynamic(op: &dyn Operation) -> i32 {
    op.execute()  // Virtual call
}

//================================
// Pattern: Downcast trait objects
//================================
use std::any::Any;

trait Component: Any {
    fn update(&mut self);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

struct Position { x: f32, y: f32 }

impl Component for Position {
    fn update(&mut self) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn get_position(component: &dyn Component) -> Option<&Position> {
    component.as_any().downcast_ref::<Position>()
}

//====================================================
// Pattern: Function pointers instead of trait objects
//====================================================
type DrawFn = fn(&str);

fn draw_circle(name: &str) {
    println!("Drawing circle: {}", name);
}

fn draw_rectangle(name: &str) {
    println!("Drawing rectangle: {}", name);
}

struct ShapeWithFn {
    name: String,
    draw: DrawFn,
}

// Slightly faster than trait objects: no vtable lookup
fn draw_with_fn(shapes: &[ShapeWithFn]) {
    for shape in shapes {
        (shape.draw)(&shape.name);
    }
}
```

**Trait object performance characteristics:**
- Dynamic dispatch: 1 indirect call (vtable lookup)
- Prevents inlining
- Cache misses on vtable access
- Larger binary size (no monomorphization duplication)

**Optimization strategies:**
- Use enum dispatch when variants are known
- Minimize trait object passing in hot paths
- Cache trait method results
- Use static dispatch in generic inner functions
- Consider function pointers for simple cases

**When to use trait objects:**
- Heterogeneous collections
- Plugin systems
- Dynamic loading
- When compile time polymorphism is impractical
- When binary size matters more than performance

## Pattern 6: Associated Types vs Generic Parameters

**Problem**: Trait design choices affect API ergonomics dramatically. Generic parameters (`trait Parser<Output>`) require explicit type annotations everywhere: `Vec<dyn Parser<String>>` is impossible (infinitely many possible `Output` types). Associated types seem simpler but can't express multiple implementations of a trait for the same type with different outputs.

**Solution**: Use associated types (`type Output`) when there's exactly one natural output type per implementation—like `Iterator::Item` is determined by the iterator type. Use generic parameters when a type should implement the trait multiple times with different inputs—like `From<i32>` and `From<&str>` both for `String`. Mix both for maximum flexibility: output as associated type, inputs as generic parameters.

**Why It Matters**: Wrong choice makes APIs painful. `Iterator` with a generic parameter would require `Vec<Box<dyn Iterator<Item = i32>>>` everywhere instead of clean `Box<dyn Iterator<Item = i32>>`. But `From` with associated types would allow only one `From` impl per type, breaking the conversion ecosystem. This decision shapes your entire API surface.

**Use Cases**: Associated types for natural outputs (Iterator::Item, Future::Output, Graph::Node), generic parameters for conversions (From, Into, TryFrom), mixed approach for flexible abstractions (generic container with natural element type).

```rust
//=============================================
// Pattern: Associated types for "output" types
//=============================================
trait Iterator {
    type Item;  // Output type

    fn next(&mut self) -> Option<Self::Item>;
}

// Cleaner than generic parameter:
// trait Iterator<Item> { fn next(&mut self) -> Option<Item>; }
// Because Item is always determined by the iterator type

//==============================================
// Pattern: Generic parameters for "input" types
//==============================================
trait From<T> {
    fn from(value: T) -> Self;
}

// Multiple From implementations for same type
impl From<i32> for String {
    fn from(n: i32) -> String {
        n.to_string()
    }
}

impl From<&str> for String {
    fn from(s: &str) -> String {
        s.to_string()
    }
}

//===========================================
// Pattern: Mix associated types and generics
//===========================================
trait Converter {
    type Output;

    fn convert<T: Into<Self::Output>>(&self, value: T) -> Self::Output;
}

//==================================
// Pattern: Associated type families
//==================================
trait Graph {
    type Node;
    type Edge;

    fn nodes(&self) -> Vec<Self::Node>;
    fn edges(&self) -> Vec<Self::Edge>;
    fn neighbors(&self, node: &Self::Node) -> Vec<Self::Node>;
}

struct AdjacencyList {
    adjacency: Vec<Vec<usize>>,
}

impl Graph for AdjacencyList {
    type Node = usize;
    type Edge = (usize, usize);

    fn nodes(&self) -> Vec<Self::Node> {
        (0..self.adjacency.len()).collect()
    }

    fn edges(&self) -> Vec<Self::Edge> {
        let mut edges = Vec::new();
        for (from, neighbors) in self.adjacency.iter().enumerate() {
            for &to in neighbors {
                edges.push((from, to));
            }
        }
        edges
    }

    fn neighbors(&self, node: &Self::Node) -> Vec<Self::Node> {
        self.adjacency.get(*node).cloned().unwrap_or_default()
    }
}
```

**Decision matrix:**
- **Associated type**: One natural output per implementation
- **Generic parameter**: Multiple implementations per type
- **Both**: Output types with input flexibility

## Performance Comparison

| Pattern | Compile Time | Runtime | Binary Size | Flexibility |
|---------|--------------|---------|-------------|-------------|
| Newtype | ✓ Fast | ✓ Zero cost | ✓ Small | Medium |
| Phantom types | ✓ Fast | ✓ Zero cost | ✓ Small | Low |
| GATs | ✗ Slow | ✓ Zero cost | Medium | High |
| Const generics | Medium | ✓ Zero cost | Medium | Medium |
| Trait objects | ✓ Fast | ✗ Dynamic dispatch | ✓ Small | High |
| Enum dispatch | ✓ Fast | ✓ Fast | Medium | Low |

## Quick Reference

```rust
// Type safety without runtime cost
struct UserId(u64);  // Newtype

// State machines in types
struct Connection<State> { _s: PhantomData<State> }

// Higher-kinded types
trait LendingIterator { type Item<'a> where Self: 'a; }

// Compile-time sizes
struct Matrix<T, const N: usize> { data: [[T; N]; N] }

// Dynamic dispatch
Box<dyn Trait>  // Heap-allocated trait object
&dyn Trait      // Borrowed trait object

// Static dispatch
fn generic<T: Trait>(x: T) { }  // Monomorphization
```

## Common Anti-Patterns

```rust
// ❌ Trait object when enum suffices
Box<dyn Operation>  // Slow

// ✓ Enum for closed set of types
enum Operation { Add, Multiply }  // Fast

// ❌ Generic parameter for single output type
trait Parser<Output> { fn parse(&self) -> Output; }

// ✓ Associated type for single output
trait Parser { type Output; fn parse(&self) -> Self::Output; }

// ❌ Type alias for domain types
type UserId = u64;  // No safety

// ✓ Newtype for domain types
struct UserId(u64);  // Type safe

// ❌ Over-engineering with phantom types
struct SimpleCounter<State> { count: usize, _s: PhantomData<State> }

// ✓ Simple when state machine not needed
struct SimpleCounter { count: usize }
```

## Key Takeaways

1. **Newtypes are free**: Use them liberally for domain modeling
2. **Phantom types enable compile-time state machines**: Zero runtime cost
3. **GATs unlock higher-kinded patterns**: Use when simpler patterns don't work
4. **Const generics for compile-time sizes**: Stack allocation without heap
5. **Trait objects have cost**: Profile before using in hot paths
6. **Enum dispatch often faster**: Closed set of types? Use enum
7. **Associated types for single outputs**: Generic parameters for inputs
8. **Type system is your friend**: Move validation to compile time

Rust's type system enables you to encode invariants that would be runtime checks (or bugs) in other languages. The patterns in this chapter show how to leverage the type system to write code that's simultaneously safer and faster than traditional approaches.
