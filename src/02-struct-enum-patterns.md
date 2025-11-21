# Struct & Enum Patterns

[Struct Design Patterns](#pattern-1-struct-design-patterns)

- Problem: When to use named vs tuple vs unit structs; lack of semantic clarity; repetitive struct definitions
- Solution: Named for clarity (User with fields), tuple for position (Point3D), unit for markers (Authenticated state)
- Why It Matters: Named fields self-document; tuple structs create distinct types; unit structs enable typestate at zero cost
- Use Cases: Named for data models; tuple for newtypes/coordinates; unit for phantom types, markers, typestate

[Newtype and Wrapper Patterns](#pattern-2-newtype-and-wrapper-patterns)

- Problem: Mixing incompatible types (UserId vs OrderId both u64); no invariant enforcement; raw primitives lack domain meaning
- Solution: Newtype pattern wraps primitives (struct UserId(u64)); validated wrappers enforce invariants; Deref for transparency
- Why It Matters: Type safety prevents mixing IDs; invariants encoded in types; eliminates defensive checks; zero runtime cost
- Use Cases: Domain-specific IDs, validated types (PositiveInteger), units (Kilometers), semantic clarity, orphan rule workaround

[Zero-Sized Types and Markers](#pattern-3-zero-sized-types-and-markers)

- Problem: Runtime state checks expensive; want compile-time guarantees; capabilities unclear; phantom type parameters unused warning
- Solution: Zero-sized types (unit structs, PhantomData) occupy 0 bytes; marker traits tag capabilities; typestate pattern enforces valid transitions
- Why It Matters: Zero runtime cost; compile-time state checking; impossible states unrepresentable; builder safety; API misuse prevented
- Use Cases: Typestate (Authenticated/Unauthenticated), capabilities (ReadPermission), builder patterns, phantom types, generic markers

[Enum Design Patterns](#pattern-4-enum-design-patterns)

- Problem: Multiple related types; optional data; state machines; error types; discriminated unions needed
- Solution: Enums for variants with pattern matching; Option/Result built-in; state machines as enums; exhaustive matching enforced
- Why It Matters: Exhaustive matching catches all cases; impossible states unrepresentable; zero-cost abstraction; clear intent
- Use Cases: State machines, error types, optional values, message types, command patterns, ASTs, protocol parsing

[Advanced Enum Techniques](#pattern-5-advanced-enum-techniques)

- Problem: Large enum variants waste memory; recursive enums; need interior mutability; want to extend enum behavior
- Solution: Box large variants; Box for recursion; impl methods on enums; match expressions; From/TryFrom for conversions
- Why It Matters: Memory efficiency (size of largest variant); recursion possible; ergonomic APIs; conversion patterns
- Use Cases: AST nodes (Box recursion), large variant optimization, state transitions, protocol handlers, command dispatch

[Struct Cheat Sheet](#struct-cheat-sheet)
- A comprehensive guide on the major aspects of structs in Rust

[Enum Cheat Sheet](#enum-cheat-sheet)
- A comprehensive guide on the major aspects of enums in Rust



## Overview 
This chapter explores struct and enum patterns for type-safe data modeling: choosing struct types, newtype wrappers for domain types, zero-sized types for compile-time guarantees, enums for variants, and advanced techniques for memory efficiency and recursion.

## Pattern 1: Struct Design Patterns

**Problem**: Confusion about when to use named field structs vs tuple structs vs unit structs. Named fields verbose for simple types (Point needs x, y, z names). Tuple structs unclear which field is which (Point3D(1.0, 2.0, 3.0)—which is x?). No semantic distinction between similar types (both u64 but UserId vs OrderId). Zero-sized marker types need PhantomData but compiler warns about unused type parameters. Want compile-time state tracking without runtime cost.

**Solution**: Use named field structs for data models where field names add clarity (`struct User { id: u64, username: String }`). Use tuple structs for simple types where position conveys meaning (`struct Point3D(f64, f64, f64)`) and newtype pattern (`struct UserId(u64)`). Use unit structs for zero-sized markers in typestate pattern (`struct Authenticated;`). PhantomData for phantom type parameters (`PhantomData<State>`). Choose based on: need for field names, distinctness requirements, zero-size marker needs.

**Why It Matters**: Named fields self-document: `user.email` is clear, `user.2` is not. Tuple structs create distinct types: UserId(1) and OrderId(1) are different types despite both wrapping u64. Unit structs enable typestate pattern at zero runtime cost: `Database<Authenticated>` vs `Database<Unauthenticated>` enforced at compile-time. Field reordering: named fields can reorder without breaking code, tuple fields can't. Memory layout: all three have same efficiency, choice is semantic not performance. Clarity vs brevity trade-off: named for complex data, tuple for simple wrappers.

**Use Cases**: Named field structs for data models (User, Config, Request/Response), domain entities, API types, database models, complex state. Tuple structs for newtype pattern (UserId, Kilometers), coordinates (Point3D, Color RGB), simple wrappers, creating distinct types from primitives. Unit structs for typestate markers (Authenticated, Open/Closed), phantom type parameters, zero-cost compile-time tags, capability markers (ReadPermission), builder pattern states.

### Example: Named Field Structs

```rust
#[derive(Debug, Clone)]
struct User {
    id: u64,
    username: String,
    email: String,
    active: bool,
}

impl User {
    fn new(id: u64, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            active: true,
        }
    }

    fn deactivate(&mut self) {
        self.active = false;
    }
}

// Usage
let user = User::new(1, "alice".to_string(), "alice@example.com".to_string());
println!("User {} is active: {}", user.username, user.active);
```

**Why this matters:** Named fields provide self-documenting code. When you see `user.email`, the intent is clear. They also allow field reordering without breaking code.

### Example: Tuple Structs

Tuple structs are useful when field names would be redundant or when you want to create distinct types:

```rust
// Coordinates where position matters more than names
struct Point3D(f64, f64, f64);

// Type-safe wrappers (newtype pattern)
struct Kilometers(f64);
struct Miles(f64);

impl Point3D {
    fn origin() -> Self {
        Point3D(0.0, 0.0, 0.0)
    }

    fn distance_from_origin(&self) -> f64 {
        (self.0.powi(2) + self.1.powi(2) + self.2.powi(2)).sqrt()
    }
}

// Usage
let point = Point3D(3.0, 4.0, 0.0);
println!("Distance: {}", point.distance_from_origin());

// Type safety prevents mixing units
let distance_km = Kilometers(100.0);
let distance_mi = Miles(62.0);
// let total = distance_km.0 + distance_mi.0; // Compiles but semantically wrong!
```

**The pattern:** Use tuple structs when the structure itself conveys meaning more than field names would. They're particularly powerful for the newtype pattern.

### Example: Unit Structs

Unit structs carry no data but can implement traits and provide type-level information:

```rust
// Marker types for type-level programming
struct Authenticated;
struct Unauthenticated;

// Zero-sized types for phantom data
struct Database<State> {
    connection_string: String,
    _state: std::marker::PhantomData<State>,
}

impl Database<Unauthenticated> {
    fn new(connection_string: String) -> Self {
        Database {
            connection_string,
            _state: std::marker::PhantomData,
        }
    }

    fn authenticate(self, password: &str) -> Result<Database<Authenticated>, String> {
        if password == "secret" {
            Ok(Database {
                connection_string: self.connection_string,
                _state: std::marker::PhantomData,
            })
        } else {
            Err("Invalid password".to_string())
        }
    }
}

impl Database<Authenticated> {
    fn query(&self, sql: &str) -> Vec<String> {
        println!("Executing: {}", sql);
        vec!["result1".to_string(), "result2".to_string()]
    }
}

// Usage
let db = Database::new("postgres://localhost".to_string());
// db.query("SELECT *"); // Error! Can't query unauthenticated database
let db = db.authenticate("secret").unwrap();
let results = db.query("SELECT * FROM users"); // Now this works
```

**The insight:** Unit structs enable compile-time state tracking without runtime overhead. This is the typestate pattern in action.

## Pattern 2: Newtype and Wrapper Patterns

**Problem**: Mixing incompatible types causes bugs—UserId(42) and OrderId(42) both u64, accidentally pass OrderId to get_user(). No invariant enforcement: PositiveInteger is just i32, negative values slip through. Raw primitives lack domain meaning: is this u64 a UserId, timestamp, or count? Can't implement external traits on external types (orphan rule): want `impl Display for Vec<T>` but can't. Defensive validation everywhere: every function checks if number is positive. Type aliases don't create new types: `type UserId = u64` doesn't prevent mixing.

**Solution**: Newtype pattern: `struct UserId(u64)` creates distinct type wrapping u64. Validated wrappers enforce invariants: `PositiveInteger::new()` returns Result, guarantees positivity. Smart constructors prevent invalid construction. Deref trait for transparent access: `impl Deref for Validated<T>` allows calling T's methods. Derive traits to propagate functionality (Debug, Clone, PartialEq). Accessor methods (`.get()`) when direct field access undesired. Workaround orphan rule: wrap external type to impl external trait.

**Why It Matters**: Type safety prevents bugs: compiler rejects `get_user(order_id)`, catches at compile-time not runtime. Invariants in types: once you have PositiveInteger, it's guaranteed positive—no defensive checks needed. Zero runtime cost: newtype compiles to same representation as wrapped type. Self-documenting code: UserId vs u64 shows intent. Orphan rule workaround: `struct Wrapper(Vec<T>)` lets you `impl Display for Wrapper`. API clarity: domain types vs primitives makes interfaces clearer. Eliminates entire bug classes: no mixing IDs, no invalid states.

**Use Cases**: Domain-specific IDs (UserId, OrderId, ProductId—prevent mixing), units (Kilometers, Miles, Seconds—prevent unit confusion), validated types (PositiveInteger, NonEmptyString, Email—enforce invariants), semantic wrappers (ConnectionString, Password—hide internals), orphan rule workaround (wrap external type to impl trait), database handles (ConnectionId, SessionId), newtype index pattern (prevent indexing wrong Vec), sensitive data (Password type hides value in Debug).

### Example: Newtype
```rust
use std::fmt;

// Newtype for semantic clarity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UserId(u64);

#[derive(Debug, Clone, Copy)]
struct OrderId(u64);

// Prevent accidentally mixing IDs
fn get_user(id: UserId) -> User {
    println!("Fetching user {}", id.0);
    // ... fetch user
    unimplemented!()
}

// This won't compile:
// let order_id = OrderId(123);
// get_user(order_id); // Type error!

// Wrapper for adding functionality
struct PositiveInteger(i32);

impl PositiveInteger {
    fn new(value: i32) -> Result<Self, String> {
        if value > 0 {
            Ok(PositiveInteger(value))
        } else {
            Err(format!("{} is not positive", value))
        }
    }

    fn get(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for PositiveInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Usage prevents invalid states
let num = PositiveInteger::new(42).unwrap();
// let invalid = PositiveInteger::new(-5); // Returns Err
```

**Why wrappers matter:** They encode invariants in the type system. Once you have a `PositiveInteger`, you know it's valid. This eliminates defensive checks throughout your codebase.

### Example: Transparent Wrappers with Deref

For ergonomic access to the wrapped type:

```rust
use std::ops::Deref;

struct Validated<T> {
    value: T,
    validated_at: std::time::Instant,
}

impl<T> Validated<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            validated_at: std::time::Instant::now(),
        }
    }

    fn age(&self) -> std::time::Duration {
        self.validated_at.elapsed()
    }
}

impl<T> Deref for Validated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// Usage
let validated_string = Validated::new("hello".to_string());
println!("Length: {}", validated_string.len()); // Deref to String
println!("Age: {:?}", validated_string.age());  // Validated method
```

## Pattern 3: Zero-Sized Types and Marker Traits

**Problem**: Runtime state checks are expensive (if authenticated { query() } else { panic!() }). Want compile-time guarantees about state. Capabilities unclear: does this File have ReadPermission? PhantomData needed for unused type parameters but compiler warns. Need zero-cost type-level information. Builder pattern allows calling build() before all fields set. No way to encode "this operation only valid in this state" at type level.

**Solution**: Zero-sized types (ZSTs) occupy 0 bytes: unit structs, PhantomData<T>. Use for marker types in typestate pattern: `Database<Unauthenticated>` vs `Database<Authenticated>`. Marker traits tag capabilities without methods: `trait ReadPermission {}`. PhantomData for phantom type parameters: `_state: PhantomData<State>` satisfies compiler. Typestate pattern: consume self, return new state type. Builder pattern with typestate: `Builder<NoFields>` → `Builder<WithName>` → build() only when complete.

**Why It Matters**: Zero runtime cost: ZSTs are 0 bytes, purely compile-time. State checking at compile-time: can't call query() on unauthenticated database, compiler error not runtime panic. Impossible states unrepresentable: builder can't be in invalid state. API misuse prevented: wrong order of operations = compile error. No overhead: typestate compiles to same code as untyped version. Type-level programming: encode constraints in type system. Self-documenting: `File<ReadWrite>` vs `File<ReadOnly>` shows capabilities.

**Use Cases**: Typestate pattern (Database<Authenticated>, File<Open>/File<Closed>), builder safety (prevent build() before ready), capability system (ReadPermission, WritePermission, AdminPermission), phantom type parameters (generic types unused at runtime), lifetime markers (variance control), protocol states (Connection<Handshake>/Connection<Established>), resource lifecycle (Handle<Acquired>/Handle<Released>), initialization tracking (Initialized/Uninitialized).

### Example: Struct Update Syntax and Partial Moves

Rust's struct update syntax enables elegant immutable updates while understanding partial moves is crucial for ownership:

```rust
#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
    retry_count: u32,
}

impl Config {
    fn with_port(self, port: u16) -> Self {
        Config { port, ..self }
    }

    fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}

// Builder-style updates
let config = Config {
    host: "localhost".to_string(),
    port: 8080,
    timeout_ms: 5000,
    retry_count: 3,
};

let new_config = Config {
    port: 9090,
    ..config // Moves non-Copy fields!
};

// config is now partially moved - can't use it anymore
// println!("{:?}", config); // Error!

// Safe pattern: clone when needed
let config = Config {
    host: "localhost".to_string(),
    port: 8080,
    timeout_ms: 5000,
    retry_count: 3,
};

let new_config = Config {
    host: "production.example.com".to_string(),
    ..config.clone()
};

// Both configs are usable
println!("Old: {:?}", config);
println!("New: {:?}", new_config);
```

**Understanding partial moves:**

```rust
struct Data {
    copyable: i32,      // Implements Copy
    moveable: String,   // Does not implement Copy
}

let data = Data {
    copyable: 42,
    moveable: "hello".to_string(),
};

// Partial move
let s = data.moveable;  // Moves String out
let n = data.copyable;  // Copies i32

// data.moveable is moved, but data.copyable is still accessible
println!("Copyable: {}", data.copyable); // OK
// println!("{}", data.moveable); // Error: value borrowed after move
```

**The pattern:** When building fluent APIs or config builders, be mindful of moves. Consider consuming self and returning Self, or use `&mut self` for in-place updates.

## Pattern 4: Enum Design Patterns

**Problem**: Multiple related types without relationship (Circle, Rectangle, Triangle all separate). Optional data represented as separate Option fields messy. State machines unclear: is connection open or closed? Error types need context but String loses structure. No way to represent "one of several types". Exhaustive handling not enforced (forgot to handle variant). Multiple return types require Result<Box<dyn Trait>, Error>.

**Solution**: Enums for variants: `enum Shape { Circle(f64), Rectangle(f64, f64), Triangle(f64, f64, f64) }`. Pattern matching enforces exhaustiveness—compiler ensures all variants handled. Option<T>/Result<T, E> built-in enums. State machines as enums: `enum ConnectionState { Connecting, Connected, Disconnected }`. Custom error types as enums with context. Methods on enums via impl blocks. Exhaustive match prevents forgetting cases.

**Why It Matters**: Exhaustive matching catches all cases: adding enum variant causes compile errors in incomplete matches. Impossible states unrepresentable: can't have both Ok and Err simultaneously. Zero-cost abstraction: enum memory = size of largest variant + discriminant (usually 1 byte). Clear intent: enum shows all possibilities. Type-safe state machines: state transitions enforced. Error handling with context: custom error enum better than String. Pattern matching provides compile-time guarantees.

**Use Cases**: State machines (ConnectionState, HttpRequestState), error types (custom Error enums with variants), optional values (Option<T> replacement), message types (WebSocket messages, RPC calls), command patterns (Command enum with variants), AST nodes (expression trees, parse trees), protocol parsing (packet types), event handling (Event enum), sum types (Either, Result).

### Example: Zero-sized Types
```rust
// Zero-sized types
struct Empty;                    // Unit struct: 0 bytes
struct Marker<T> {
    _phantom: std::marker::PhantomData<T>  // 0 bytes
}
struct TwoMarkers {
    a: (),                      // Unit type: 0 bytes
    b: std::marker::PhantomData<i32>  // 0 bytes
}

// Verify they're actually zero-sized
fn check_sizes() {
    assert_eq!(std::mem::size_of::<Empty>(), 0);
    assert_eq!(std::mem::size_of::<Marker<String>>(), 0);
    assert_eq!(std::mem::size_of::<TwoMarkers>(), 0);
}
```

**Why ZSTs matter:**
- Compile-time information with zero runtime cost
- Enable marker-based type system programming
- Foundation for phantom types and type states
- Used extensively in Rust's standard library

### Example: Marker Traits

Marker traits are traits with no methods that tag types with capabilities or properties:

```rust
//=========================================
// Standard library marker traits (auto traits)
//=========================================

// Send: Can be transferred across thread boundaries
// Sync: Can be shared between threads (via &T)
// Copy: Can be copied bitwise
// Sized: Has a known size at compile time
// Unpin: Can be safely moved after being pinned

//=========================================
// Custom marker traits
//=========================================

// Marker for types that have been validated
trait Validated {}

// Marker for types that are safe to serialize
trait SafeSerialize {}

// Marker for types representing capabilities
trait ReadPermission {}
trait WritePermission {}
trait AdminPermission {}

//=========================================
// Pattern: Capability-based design
//=========================================

struct File<Permissions> {
    path: String,
    _permissions: PhantomData<Permissions>,
}

// Only files with ReadPermission can be read
impl<P: ReadPermission> File<P> {
    fn read(&self) -> String {
        format!("Reading from {}", self.path)
    }
}

// Only files with WritePermission can be written
impl<P: WritePermission> File<P> {
    fn write(&mut self, data: &str) {
        println!("Writing '{}' to {}", data, self.path);
    }
}

// Only files with AdminPermission can be deleted
impl<P: AdminPermission> File<P> {
    fn delete(self) {
        println!("Deleting {}", self.path);
    }
}

// Permission types (all ZSTs)
struct ReadOnly;
struct ReadWrite;
struct Admin;

impl ReadPermission for ReadOnly {}
impl ReadPermission for ReadWrite {}
impl ReadPermission for Admin {}

impl WritePermission for ReadWrite {}
impl WritePermission for Admin {}

impl AdminPermission for Admin {}

// Usage: Type system enforces permissions
fn capability_example() {
    let readonly_file = File::<ReadOnly> {
        path: "data.txt".to_string(),
        _permissions: PhantomData,
    };

    let _data = readonly_file.read();  // ✓ OK
    // readonly_file.write("data");    // ✗ Error: no write permission

    let mut readwrite_file = File::<ReadWrite> {
        path: "config.txt".to_string(),
        _permissions: PhantomData,
    };

    let _data = readwrite_file.read();      // ✓ OK
    readwrite_file.write("new config");     // ✓ OK
    // readwrite_file.delete();             // ✗ Error: no admin permission

    let admin_file = File::<Admin> {
        path: "system.txt".to_string(),
        _permissions: PhantomData,
    };

    admin_file.delete();  // ✓ OK: Admin has all permissions
}
```

### Example: Type-Level Witnesses

Use ZSTs as compile-time witnesses to prove properties:

```rust
//=========================================
// Pattern: Type-level proof witnesses
//=========================================

// Witness that a value is non-empty
struct NonEmpty;

// Witness that a value is sorted
struct Sorted;

struct Vec<T, State = ()> {
    data: std::vec::Vec<T>,
    _state: PhantomData<State>,
}

impl<T> Vec<T, ()> {
    fn new(data: std::vec::Vec<T>) -> Self {
        Vec {
            data,
            _state: PhantomData,
        }
    }

    // Prove non-empty at construction
    fn non_empty(data: std::vec::Vec<T>) -> Option<Vec<T, NonEmpty>> {
        if data.is_empty() {
            None
        } else {
            Some(Vec {
                data,
                _state: PhantomData,
            })
        }
    }
}

impl<T: Ord> Vec<T, ()> {
    // Prove sorted by sorting
    fn sorted(mut data: std::vec::Vec<T>) -> Vec<T, Sorted> {
        data.sort();
        Vec {
            data,
            _state: PhantomData,
        }
    }
}

// Only non-empty vectors can return first element
impl<T: Clone> Vec<T, NonEmpty> {
    fn first(&self) -> T {
        // Safe: NonEmpty witness guarantees data is non-empty
        self.data[0].clone()
    }
}

// Only sorted vectors can do binary search
impl<T: Ord> Vec<T, Sorted> {
    fn binary_search(&self, item: &T) -> Result<usize, usize> {
        self.data.binary_search(item)
    }
}

fn witness_example() {
    let empty = Vec::new(vec![]);
    // empty.first();  // ✗ Error: no witness for NonEmpty

    let non_empty = Vec::non_empty(vec![1, 2, 3]).unwrap();
    let first = non_empty.first();  // ✓ OK: has NonEmpty witness

    let sorted = Vec::sorted(vec![3, 1, 2]);
    let _pos = sorted.binary_search(&2);  // ✓ OK: has Sorted witness
}
```

### Example: Branded Types Pattern

Use ZSTs to create "branded" types that cannot be forged:

```rust
use std::marker::PhantomData;

//=========================================
// Pattern: Branded types for security
//=========================================

// Brand marker (not exported)
struct SanitizedBrand;

// Branded type ensures values are sanitized
pub struct Sanitized<T> {
    value: T,
    _brand: PhantomData<SanitizedBrand>,
}

impl Sanitized<String> {
    // Only way to create is through validation
    pub fn new(input: String) -> Self {
        let sanitized = input
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("&", "&amp;");

        Sanitized {
            value: sanitized,
            _brand: PhantomData,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }
}

// Safe database queries only accept sanitized input
pub struct Database;

impl Database {
    // Cannot be called with unsanitized strings
    pub fn query(&self, sql: Sanitized<String>) {
        println!("Executing: {}", sql.as_str());
        // Safe: sql is guaranteed sanitized
    }
}

fn branded_example() {
    let db = Database;

    // Must sanitize before querying
    let user_input = String::from("Robert'); DROP TABLE users;--");
    let sanitized = Sanitized::new(user_input);

    db.query(sanitized);  // ✓ Safe

    // Cannot forge a Sanitized<String>
    // let fake = Sanitized { value: "malicious".to_string(), _brand: PhantomData };
    // ✗ Error: SanitizedBrand is private
}

//=========================================
// Pattern: Multiple brands for different properties
//=========================================

struct Encrypted;
struct Compressed;

struct Data<Brands> {
    bytes: Vec<u8>,
    _brands: PhantomData<Brands>,
}

impl Data<()> {
    fn new(bytes: Vec<u8>) -> Self {
        Data {
            bytes,
            _brands: PhantomData,
        }
    }

    fn encrypt(self) -> Data<Encrypted> {
        // ... encryption logic ...
        Data {
            bytes: self.bytes,
            _brands: PhantomData,
        }
    }
}

impl Data<Encrypted> {
    fn compress(self) -> Data<(Encrypted, Compressed)> {
        // ... compression logic ...
        Data {
            bytes: self.bytes,
            _brands: PhantomData,
        }
    }
}

// Only encrypted and compressed data can be transmitted
fn transmit(data: Data<(Encrypted, Compressed)>) {
    println!("Transmitting {} bytes", data.bytes.len());
}
```

### Example: Performance and Best Practices

**Zero-cost guarantees:**
```rust
struct WithZst {
    data: String,
    marker: PhantomData<i32>,
}

struct WithoutZst {
    data: String,
}

fn size_check() {
    // Both have the same size - ZST adds no overhead
    assert_eq!(
        std::mem::size_of::<WithZst>(),
        std::mem::size_of::<WithoutZst>()
    );

    // Arrays of ZSTs are also zero-sized
    assert_eq!(std::mem::size_of::<[(); 1000]>(), 0);
}
```

**Best practices:**
1. Use `PhantomData<T>` instead of storing unused `T` values
2. Prefer marker traits over runtime booleans for properties
3. Use sealed traits to control trait implementations
4. Brand sensitive types to prevent forgery
5. Document why each ZST exists (not obvious from code)

**When to use ZSTs:**
- ✓ Capability-based security (permissions)
- ✓ Compile-time witnesses (sorted, validated, non-empty)
- ✓ Branded types (sanitized, encrypted)
- ✓ Sealed trait hierarchies
- ✓ Type-level programming foundations

**When NOT to use ZSTs:**
- ✗ When runtime checks are simpler and sufficient
- ✗ For dynamic properties that change at runtime
- ✗ When the type complexity outweighs benefits

## Pattern 5: Advanced Enum Techniques

**Problem**: Large enum variants waste memory—enum size = largest variant + discriminant, small variants waste space. Recursive enums (AST nodes with children) have infinite size—compiler error. Need interior mutability in enum. Want to extend enum behavior without modifying definition. Conversions between related types verbose. Enum size unpredictable affecting performance. Can't determine variant without matching.

**Solution**: Box large variants to reduce enum size: `enum Message { Small(u8), Large(Box<HugeStruct>) }` makes enum smaller. Box for recursive enums: `enum Node { Leaf(i32), Branch(Box<Node>, Box<Node>) }` breaks infinite size. Implement methods on enums with impl blocks. Match expressions for transformation. From/TryFrom traits for conversions. Use #[repr(u8)] for explicit discriminant. mem::size_of to check enum size. `matches!` macro for simple checks.

**Why It Matters**: Memory efficiency: boxing large variants reduces enum from size of largest to size of pointer. Recursion enabled: Box breaks infinite size allowing AST nodes, linked lists. Method dispatch via match: same interface different behavior per variant. Ergonomic APIs: methods on enums cleaner than separate functions. Conversion patterns: From/TryFrom standardize conversions. Performance: smaller enums = better cache locality. Discriminant control: #[repr] ensures layout for FFI. Variant checking: matches! avoids full match.

**Use Cases**: AST nodes (Box for child nodes), large variant optimization (Box rarely-used variants), state machines (methods for transitions), protocol handlers (dispatch via match), command dispatch (Command enum with execute() method), Option/Result extensions (custom enums), recursive data structures (trees, lists), error conversion (From for error types), enum-based visitor pattern.

### Example: Enum as Contract

```rust
// Model HTTP responses precisely
enum HttpResponse {
    Ok { body: String, headers: Vec<(String, String)> },
    Created { id: u64, location: String },
    NoContent,
    BadRequest { error: String },
    Unauthorized,
    NotFound,
    ServerError { message: String, details: Option<String> },
}

impl HttpResponse {
    fn status_code(&self) -> u16 {
        match self {
            HttpResponse::Ok { .. } => 200,
            HttpResponse::Created { .. } => 201,
            HttpResponse::NoContent => 204,
            HttpResponse::BadRequest { .. } => 400,
            HttpResponse::Unauthorized => 401,
            HttpResponse::NotFound => 404,
            HttpResponse::ServerError { .. } => 500,
        }
    }

    fn format(&self) -> String {
        match self {
            HttpResponse::Ok { body, .. } => body.clone(),
            HttpResponse::Created { id, location } => {
                format!("Created resource {} at {}", id, location)
            }
            HttpResponse::NoContent => String::new(),
            HttpResponse::BadRequest { error } => {
                format!("Bad request: {}", error)
            }
            HttpResponse::Unauthorized => "Unauthorized".to_string(),
            HttpResponse::NotFound => "Not found".to_string(),
            HttpResponse::ServerError { message, details } => {
                if let Some(d) = details {
                    format!("Error: {} ({})", message, d)
                } else {
                    format!("Error: {}", message)
                }
            }
        }
    }
}

// Usage
fn handle_request(path: &str) -> HttpResponse {
    match path {
        "/users" => HttpResponse::Ok {
            body: "[{\"id\": 1, \"name\": \"Alice\"}]".to_string(),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        },
        "/users/create" => HttpResponse::Created {
            id: 123,
            location: "/users/123".to_string(),
        },
        _ => HttpResponse::NotFound,
    }
}
```

**The power:** Each variant can carry exactly the data it needs. There's no `null` or `undefined` - if a variant needs an ID, it has one. If it doesn't, it can't have one.

### Example: Enum Variants with Rich Data

Enums shine when modeling state machines or complex workflows:

```rust
enum OrderStatus {
    Pending {
        items: Vec<String>,
        customer_id: u64,
    },
    Processing {
        order_id: u64,
        started_at: std::time::Instant,
    },
    Shipped {
        order_id: u64,
        tracking_number: String,
        carrier: String,
    },
    Delivered {
        order_id: u64,
        delivered_at: std::time::SystemTime,
        signature: Option<String>,
    },
    Cancelled {
        order_id: u64,
        reason: String,
    },
}

impl OrderStatus {
    fn process(self) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Pending { items, customer_id } => {
                if items.is_empty() {
                    return Err("Cannot process empty order".to_string());
                }
                Ok(OrderStatus::Processing {
                    order_id: 12345, // Generated
                    started_at: std::time::Instant::now(),
                })
            }
            _ => Err("Order is not in pending state".to_string()),
        }
    }

    fn ship(self, tracking_number: String, carrier: String) -> Result<OrderStatus, String> {
        match self {
            OrderStatus::Processing { order_id, .. } => {
                Ok(OrderStatus::Shipped {
                    order_id,
                    tracking_number,
                    carrier,
                })
            }
            _ => Err("Can only ship processing orders".to_string()),
        }
    }

    fn can_cancel(&self) -> bool {
        matches!(self, OrderStatus::Pending { .. } | OrderStatus::Processing { .. })
    }
}
```

**The benefit:** Invalid state transitions become impossible. You can't ship a cancelled order because the types don't align.

## Pattern 6: Visitor Pattern with Enums

The visitor pattern in Rust leverages enums for traversing complex structures:

```rust
// AST for a simple expression language
enum Expr {
    Number(f64),
    Variable(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnOp,
        expr: Box<Expr>,
    },
}

enum BinOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

enum UnOp {
    Negate,
    Abs,
}

// Visitor trait
trait ExprVisitor {
    type Output;

    fn visit(&mut self, expr: &Expr) -> Self::Output {
        match expr {
            Expr::Number(n) => self.visit_number(*n),
            Expr::Variable(name) => self.visit_variable(name),
            Expr::BinaryOp { op, left, right } => {
                self.visit_binary_op(op, left, right)
            }
            Expr::UnaryOp { op, expr } => {
                self.visit_unary_op(op, expr)
            }
        }
    }

    fn visit_number(&mut self, n: f64) -> Self::Output;
    fn visit_variable(&mut self, name: &str) -> Self::Output;
    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Self::Output;
    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> Self::Output;
}

// Pretty printer visitor
struct PrettyPrinter {
    indent: usize,
}

impl ExprVisitor for PrettyPrinter {
    type Output = String;

    fn visit_number(&mut self, n: f64) -> String {
        n.to_string()
    }

    fn visit_variable(&mut self, name: &str) -> String {
        name.to_string()
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> String {
        let op_str = match op {
            BinOp::Add => "+",
            BinOp::Subtract => "-",
            BinOp::Multiply => "*",
            BinOp::Divide => "/",
        };

        format!("({} {} {})",
            self.visit(left),
            op_str,
            self.visit(right))
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> String {
        let op_str = match op {
            UnOp::Negate => "-",
            UnOp::Abs => "abs",
        };

        format!("{}({})", op_str, self.visit(expr))
    }
}

// Evaluator visitor
struct Evaluator {
    variables: std::collections::HashMap<String, f64>,
}

impl ExprVisitor for Evaluator {
    type Output = Result<f64, String>;

    fn visit_number(&mut self, n: f64) -> Self::Output {
        Ok(n)
    }

    fn visit_variable(&mut self, name: &str) -> Self::Output {
        self.variables.get(name)
            .copied()
            .ok_or_else(|| format!("Undefined variable: {}", name))
    }

    fn visit_binary_op(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> Self::Output {
        let left_val = self.visit(left)?;
        let right_val = self.visit(right)?;

        match op {
            BinOp::Add => Ok(left_val + right_val),
            BinOp::Subtract => Ok(left_val - right_val),
            BinOp::Multiply => Ok(left_val * right_val),
            BinOp::Divide => {
                if right_val == 0.0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left_val / right_val)
                }
            }
        }
    }

    fn visit_unary_op(&mut self, op: &UnOp, expr: &Expr) -> Self::Output {
        let val = self.visit(expr)?;
        match op {
            UnOp::Negate => Ok(-val),
            UnOp::Abs => Ok(val.abs()),
        }
    }
}

// Usage
fn demo_visitor() {
    // (3 + 4) * 2
    let expr = Expr::BinaryOp {
        op: BinOp::Multiply,
        left: Box::new(Expr::BinaryOp {
            op: BinOp::Add,
            left: Box::new(Expr::Number(3.0)),
            right: Box::new(Expr::Number(4.0)),
        }),
        right: Box::new(Expr::Number(2.0)),
    };

    let mut printer = PrettyPrinter { indent: 0 };
    println!("Expression: {}", printer.visit(&expr));

    let mut evaluator = Evaluator {
        variables: std::collections::HashMap::new(),
    };
    match evaluator.visit(&expr) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```

**The pattern:** Visitors separate traversal logic from data structure. You can add new operations without modifying the enum definition.

## Pattern 8: Type-Safe State Machines with Enums

State machines prevent invalid states and transitions at compile time:

```rust
// Simple state machine: Door
struct Open;
struct Closed;
struct Locked;

struct Door<State> {
    _state: std::marker::PhantomData<State>,
}

impl Door<Closed> {
    fn new() -> Self {
        println!("Door created in closed state");
        Door { _state: std::marker::PhantomData }
    }

    fn open(self) -> Door<Open> {
        println!("Opening door");
        Door { _state: std::marker::PhantomData }
    }

    fn lock(self) -> Door<Locked> {
        println!("Locking door");
        Door { _state: std::marker::PhantomData }
    }
}

impl Door<Open> {
    fn close(self) -> Door<Closed> {
        println!("Closing door");
        Door { _state: std::marker::PhantomData }
    }
}

impl Door<Locked> {
    fn unlock(self) -> Door<Closed> {
        println!("Unlocking door");
        Door { _state: std::marker::PhantomData }
    }
}

// Complex state machine with enum states
#[derive(Debug)]
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Authenticated { session_id: String, user_id: u64 },
}

struct Connection {
    state: ConnectionState,
    max_retries: u32,
}

impl Connection {
    fn new() -> Self {
        Connection {
            state: ConnectionState::Disconnected,
            max_retries: 3,
        }
    }

    fn connect(&mut self) -> Result<(), String> {
        match &self.state {
            ConnectionState::Disconnected => {
                self.state = ConnectionState::Connecting { attempt: 1 };
                Ok(())
            }
            ConnectionState::Connecting { attempt } if *attempt < self.max_retries => {
                self.state = ConnectionState::Connecting { attempt: attempt + 1 };
                Ok(())
            }
            _ => Err("Cannot connect in current state".to_string()),
        }
    }

    fn establish(&mut self, session_id: String) -> Result<(), String> {
        match &self.state {
            ConnectionState::Connecting { .. } => {
                self.state = ConnectionState::Connected { session_id };
                Ok(())
            }
            _ => Err("Not in connecting state".to_string()),
        }
    }

    fn authenticate(&mut self, user_id: u64) -> Result<(), String> {
        match &self.state {
            ConnectionState::Connected { session_id } => {
                self.state = ConnectionState::Authenticated {
                    session_id: session_id.clone(),
                    user_id,
                };
                Ok(())
            }
            _ => Err("Must be connected to authenticate".to_string()),
        }
    }

    fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    fn is_authenticated(&self) -> bool {
        matches!(self.state, ConnectionState::Authenticated { .. })
    }
}

fn demo_state_machine() {
    // Type-state door
    let door = Door::<Closed>::new();
    let door = door.open();
    let door = door.close();
    let door = door.lock();
    // door.open(); // Compile error! Can't open a locked door
    let door = door.unlock();
    let _door = door.open();

    // Enum-based connection
    let mut conn = Connection::new();
    conn.connect().unwrap();
    conn.establish("session-123".to_string()).unwrap();
    conn.authenticate(42).unwrap();
    assert!(conn.is_authenticated());
    println!("Connection state: {:?}", conn.state);
}
```

**Why this works:** The type system enforces valid state transitions. You can't accidentally call `unlock()` on an open door because that method simply doesn't exist for `Door<Open>`.

### Combining Enums with Type States

For maximum safety, combine both approaches:

```rust
// Payment processing state machine
struct Pending;
struct Authorized;
struct Captured;
struct Refunded;

struct Payment<State> {
    id: String,
    amount: u64,
    state_data: State,
}

// Each state has its own data
impl Payment<Pending> {
    fn new(amount: u64) -> Self {
        Payment {
            id: format!("pay_{}", uuid::Uuid::new_v4()),
            amount,
            state_data: Pending,
        }
    }

    fn authorize(self, auth_code: String) -> Payment<Authorized> {
        Payment {
            id: self.id,
            amount: self.amount,
            state_data: Authorized { auth_code },
        }
    }

    fn cancel(self) -> PaymentResult {
        PaymentResult::Cancelled { payment_id: self.id }
    }
}

struct Authorized {
    auth_code: String,
}

impl Payment<Authorized> {
    fn capture(self) -> Payment<Captured> {
        Payment {
            id: self.id,
            amount: self.amount,
            state_data: Captured {
                auth_code: self.state_data.auth_code,
                captured_at: std::time::SystemTime::now(),
            },
        }
    }

    fn void(self) -> PaymentResult {
        PaymentResult::Voided {
            payment_id: self.id,
            auth_code: self.state_data.auth_code,
        }
    }
}

struct Captured {
    auth_code: String,
    captured_at: std::time::SystemTime,
}

impl Payment<Captured> {
    fn refund(self, reason: String) -> Payment<Refunded> {
        Payment {
            id: self.id,
            amount: self.amount,
            state_data: Refunded {
                auth_code: self.state_data.auth_code,
                captured_at: self.state_data.captured_at,
                refunded_at: std::time::SystemTime::now(),
                reason,
            },
        }
    }
}

struct Refunded {
    auth_code: String,
    captured_at: std::time::SystemTime,
    refunded_at: std::time::SystemTime,
    reason: String,
}

enum PaymentResult {
    Cancelled { payment_id: String },
    Voided { payment_id: String, auth_code: String },
}

// Usage demonstrates compile-time safety
fn process_payment() {
    let payment = Payment::<Pending>::new(10000);
    // payment.capture(); // Compile error! Can't capture pending payment

    let payment = payment.authorize("AUTH123".to_string());
    let payment = payment.capture();
    // payment.authorize(...); // Compile error! Already captured

    let _result = payment.refund("Customer requested".to_string());
}
```

**The architecture:** Each state transition consumes the old state and returns a new one. Invalid transitions don't exist in the type system.

## Summary

This chapter covered struct and enum patterns for type-safe data modeling:

1. **Struct Design Patterns**: Named fields for clarity, tuple for newtypes/position, unit for zero-cost markers
2. **Newtype and Wrapper Patterns**: Domain IDs, validated types, invariant enforcement, orphan rule workaround
3. **Zero-Sized Types and Markers**: Typestate pattern, capability markers, PhantomData, compile-time guarantees
4. **Enum Design Patterns**: Variants for related types, exhaustive matching, state machines, error types
5. **Advanced Enum Techniques**: Box for large/recursive variants, methods on enums, conversions, memory optimization

**Key Takeaways**:
- Struct choice is semantic: named for data models, tuple for wrappers, unit for markers
- Newtype pattern: UserId(u64) vs OrderId(u64) prevents mixing at zero cost
- ZSTs are 0 bytes: typestate and markers provide compile-time guarantees without overhead
- Enums enforce exhaustiveness: adding variant causes compile errors in incomplete matches
- Impossible states unrepresentable: Payment<Pending> can't capture(), only authorize()

**Design Principles**:
- Use named fields when clarity matters, tuple when type itself is meaningful
- Wrap primitives in domain types (UserId not u64) for type safety
- Encode invariants in types (PositiveInteger guaranteed positive)
- Use typestate for state machines (Database<Authenticated> vs Database<Unauthenticated>)
- Enums for "one of" types, structs for "all of" types
- Box large/recursive enum variants for memory efficiency

**Performance Characteristics**:
- Newtype: zero runtime cost, same representation as wrapped type
- ZSTs: 0 bytes, purely compile-time, no runtime overhead
- Typestate: compiles to same code as untyped version
- Enum size: largest variant + discriminant (usually 1 byte)
- Boxing: reduces enum to pointer size, adds indirection

**Memory Layout**:
- Named struct: fields in declaration order (subject to alignment)
- Tuple struct: same as tuple with same types
- Unit struct: 0 bytes
- Enum: size_of(largest variant) + discriminant
- Box<T>: size_of pointer (8 bytes on 64-bit)

**Pattern Decision Matrix**:
- **Multiple types, all fields present**: Named struct
- **Simple wrapper, distinct type**: Tuple struct (newtype)
- **No data, marker only**: Unit struct
- **Compile-time state**: Unit struct + PhantomData (typestate)
- **One of several types**: Enum
- **Recursive structure**: Enum with Box
- **Validated type**: Newtype with smart constructor
- **Domain-specific ID**: Newtype (struct UserId(u64))

**Anti-Patterns to Avoid**:
- Using u64 for IDs instead of newtypes (loses type safety)
- Runtime state checks instead of typestate (loses compile-time safety)
- Multiple Option fields instead of enum (unclear which combinations valid)
- Large enum variants without Box (wastes memory)
- Missing exhaustive match (non-exhaustive pattern use `_`)
- Type aliases for distinct types (`type UserId = u64` doesn't prevent mixing)

## Struct Cheat Sheet
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



## Enum Cheat Sheet

```rust
// ===== BASIC ENUMS =====
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

fn basic_enum_example() {
    let dir = Direction::North;
    
    match dir {
        Direction::North => println!("Going north"),
        Direction::South => println!("Going south"),
        Direction::East => println!("Going east"),
        Direction::West => println!("Going west"),
    }
}

// ===== ENUMS WITH DATA =====
// Each variant can hold different types and amounts of data
enum Message {
    Quit,                                                // No data
    Move { x: i32, y: i32 },                            // Named fields
    Write(String),                                       // Single value
    ChangeColor(u8, u8, u8),                            // Multiple values
}

fn enum_with_data_example() {
    let msg1 = Message::Quit;
    let msg2 = Message::Move { x: 10, y: 20 };
    let msg3 = Message::Write(String::from("Hello"));
    let msg4 = Message::ChangeColor(255, 0, 0);
}

// ===== PATTERN MATCHING =====
fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quitting");
        }
        Message::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        Message::Write(text) => {
            println!("Writing: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println!("Changing color to RGB({}, {}, {})", r, g, b);
        }
    }
}

// Match with guards
fn match_with_guard(msg: Message) {
    match msg {
        Message::Move { x, y } if x > 0 && y > 0 => {
            println!("Moving to positive quadrant");
        }
        Message::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        _ => {}
    }
}

// ===== OPTION ENUM =====
// Option<T> is defined in standard library as:
// enum Option<T> {
//     Some(T),
//     None,
// }

fn option_example() {
    let some_number = Some(5);
    let some_string = Some("Hello");
    let absent_number: Option<i32> = None;
    
    // Pattern matching
    match some_number {
        Some(n) => println!("Number: {}", n),
        None => println!("No number"),
    }
    
    // if let
    if let Some(n) = some_number {
        println!("Got number: {}", n);
    }
    
    // Unwrap (panics if None)
    let n = some_number.unwrap();
    
    // Unwrap with default
    let n = absent_number.unwrap_or(0);
    
    // Unwrap with closure
    let n = absent_number.unwrap_or_else(|| 10);
    
    // Map
    let doubled = some_number.map(|n| n * 2);
    
    // And then (flatMap)
    let result = some_number.and_then(|n| Some(n * 2));
}

// ===== RESULT ENUM =====
// Result<T, E> is defined as:
// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

fn result_example() {
    let result = divide(10, 2);
    
    // Pattern matching
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(e) => println!("Error: {}", e),
    }
    
    // Unwrap (panics on Err)
    let value = divide(10, 2).unwrap();
    
    // Expect (custom panic message)
    let value = divide(10, 2).expect("Division failed");
    
    // Unwrap or default
    let value = divide(10, 0).unwrap_or(0);
    
    // Question mark operator
    fn propagate_error() -> Result<i32, String> {
        let result = divide(10, 2)?;                     // Returns Err early
        Ok(result * 2)
    }
    
    // Map
    let doubled = divide(10, 2).map(|n| n * 2);
    
    // Map error
    let result = divide(10, 0).map_err(|e| format!("Error: {}", e));
}

// ===== ENUM METHODS =====
impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit called"),
            Message::Move { x, y } => println!("Move to ({}, {})", x, y),
            Message::Write(text) => println!("Write: {}", text),
            Message::ChangeColor(r, g, b) => println!("Color: RGB({}, {}, {})", r, g, b),
        }
    }
    
    fn is_quit(&self) -> bool {
        matches!(self, Message::Quit)
    }
}

fn enum_methods_example() {
    let msg = Message::Write(String::from("Hello"));
    msg.call();
    println!("Is quit: {}", msg.is_quit());
}

// ===== GENERIC ENUMS =====
enum Result2<T, E> {
    Ok(T),
    Err(E),
}

enum Option2<T> {
    Some(T),
    None,
}

// Multiple generic parameters
enum Either<L, R> {
    Left(L),
    Right(R),
}

fn generic_enum_example() {
    let left: Either<i32, String> = Either::Left(42);
    let right: Either<i32, String> = Either::Right(String::from("Hello"));
    
    match left {
        Either::Left(n) => println!("Number: {}", n),
        Either::Right(s) => println!("String: {}", s),
    }
}

// ===== IF LET =====
fn if_let_example() {
    let some_value = Some(3);
    
    // Instead of match
    if let Some(n) = some_value {
        println!("Number: {}", n);
    }
    
    // With else
    if let Some(n) = some_value {
        println!("Number: {}", n);
    } else {
        println!("No number");
    }
    
    // Multiple if let
    let msg = Message::Move { x: 10, y: 20 };
    
    if let Message::Move { x, y } = msg {
        println!("Move to ({}, {})", x, y);
    } else if let Message::Write(text) = msg {
        println!("Write: {}", text);
    } else {
        println!("Other message");
    }
}

// ===== WHILE LET =====
fn while_let_example() {
    let mut stack = vec![1, 2, 3, 4, 5];
    
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
}

// ===== MATCHES! MACRO =====
fn matches_example() {
    let msg = Message::Write(String::from("Hello"));
    
    // Check if matches pattern
    if matches!(msg, Message::Write(_)) {
        println!("It's a Write message");
    }
    
    // With guard
    let num = Some(4);
    if matches!(num, Some(x) if x < 5) {
        println!("Number is less than 5");
    }
}

// ===== ENUM DISCRIMINANTS =====
#[repr(u8)]
enum Status {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}

fn discriminant_example() {
    let status = Status::Active;
    let value = status as u8;
    println!("Status value: {}", value);
}

// ===== C-LIKE ENUMS =====
#[repr(C)]
enum Color {
    Red = 0xFF0000,
    Green = 0x00FF00,
    Blue = 0x0000FF,
}

#[repr(i32)]
enum ErrorCode {
    Success = 0,
    NotFound = -1,
    PermissionDenied = -2,
    InternalError = -500,
}

// ===== DERIVE MACROS FOR ENUMS =====
#[derive(Debug)]
enum LogLevel {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Copy)]
enum Direction2 {
    North,
    South,
    East,
    West,
}

#[derive(PartialEq, Eq)]
enum State {
    Active,
    Inactive,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Priority {
    Low,
    Medium,
    High,
}

fn derive_example() {
    let level = LogLevel::Info;
    println!("{:?}", level);
    
    let state1 = State::Active;
    let state2 = State::Active;
    println!("Equal: {}", state1 == state2);
    
    let p1 = Priority::Low;
    let p2 = Priority::High;
    println!("p1 < p2: {}", p1 < p2);
}

// ===== ENUMS WITH LIFETIMES =====
enum Cow<'a> {
    Borrowed(&'a str),
    Owned(String),
}

fn lifetime_enum_example() {
    let borrowed = Cow::Borrowed("Hello");
    let owned = Cow::Owned(String::from("World"));
}

// ===== RECURSIVE ENUMS =====
// Requires Box for indirection
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> Self {
        List::Nil
    }
    
    fn prepend(self, elem: i32) -> Self {
        List::Cons(elem, Box::new(self))
    }
    
    fn len(&self) -> usize {
        match self {
            List::Cons(_, tail) => 1 + tail.len(),
            List::Nil => 0,
        }
    }
}

fn recursive_enum_example() {
    let list = List::new()
        .prepend(1)
        .prepend(2)
        .prepend(3);
    
    println!("Length: {}", list.len());
}

// Binary tree
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

// ===== ENUM AS STATE MACHINE =====
enum ConnectionState {
    Disconnected,
    Connecting { retry_count: u32 },
    Connected { session_id: String },
    Error { message: String },
}

impl ConnectionState {
    fn connect(self) -> Self {
        match self {
            ConnectionState::Disconnected => {
                ConnectionState::Connecting { retry_count: 0 }
            }
            _ => self,
        }
    }
    
    fn establish(self, session_id: String) -> Self {
        match self {
            ConnectionState::Connecting { .. } => {
                ConnectionState::Connected { session_id }
            }
            _ => self,
        }
    }
    
    fn disconnect(self) -> Self {
        ConnectionState::Disconnected
    }
}

// ===== ENUM WITH ASSOCIATED DATA PATTERNS =====
// Linked list
enum LinkedList<T> {
    Empty,
    Node(T, Box<LinkedList<T>>),
}

// JSON-like value
#[derive(Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::HashMap<String, JsonValue>),
}

fn json_example() {
    use std::collections::HashMap;
    
    let mut obj = HashMap::new();
    obj.insert(String::from("name"), JsonValue::String(String::from("Alice")));
    obj.insert(String::from("age"), JsonValue::Number(30.0));
    obj.insert(String::from("active"), JsonValue::Bool(true));
    
    let json = JsonValue::Object(obj);
    println!("{:?}", json);
}

// ===== NESTED ENUMS =====
enum OuterMessage {
    Inner(InnerMessage),
    Other,
}

enum InnerMessage {
    Data(i32),
    Text(String),
}

fn nested_enum_example() {
    let msg = OuterMessage::Inner(InnerMessage::Data(42));
    
    match msg {
        OuterMessage::Inner(InnerMessage::Data(n)) => {
            println!("Data: {}", n);
        }
        OuterMessage::Inner(InnerMessage::Text(s)) => {
            println!("Text: {}", s);
        }
        OuterMessage::Other => {
            println!("Other");
        }
    }
}

// ===== ENUM SIZE AND LAYOUT =====
fn enum_size() {
    println!("Option<i32> size: {}", std::mem::size_of::<Option<i32>>());
    println!("Result<i32, String> size: {}", std::mem::size_of::<Result<i32, String>>());
    
    // Enum size is size of largest variant + discriminant
    enum Large {
        Small(u8),
        Large([u8; 100]),
    }
    
    println!("Large enum size: {}", std::mem::size_of::<Large>());
}

// ===== ENUM DESTRUCTURING =====
fn destructure_enum() {
    let msg = Message::ChangeColor(255, 0, 0);
    
    // Destructure in match
    match msg {
        Message::ChangeColor(r, g, b) => {
            println!("R: {}, G: {}, B: {}", r, g, b);
        }
        _ => {}
    }
    
    // Destructure in let
    if let Message::ChangeColor(r, g, b) = msg {
        println!("R: {}, G: {}, B: {}", r, g, b);
    }
    
    // Destructure struct variant
    let msg = Message::Move { x: 10, y: 20 };
    if let Message::Move { x, y } = msg {
        println!("X: {}, Y: {}", x, y);
    }
}

// ===== ENUM CONVERSIONS =====
// From/Into implementations
impl From<bool> for Status {
    fn from(active: bool) -> Self {
        if active {
            Status::Active
        } else {
            Status::Inactive
        }
    }
}

fn conversion_example() {
    let status: Status = true.into();
}

// TryFrom for fallible conversions
use std::convert::TryFrom;

impl TryFrom<i32> for Status {
    type Error = String;
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Status::Active),
            2 => Ok(Status::Inactive),
            3 => Ok(Status::Pending),
            _ => Err(format!("Invalid status code: {}", value)),
        }
    }
}

// ===== NON-EXHAUSTIVE ENUMS =====
#[non_exhaustive]
pub enum ApiError {
    NotFound,
    Unauthorized,
    InternalError,
}

// Users must use _ pattern to handle future variants
fn handle_error(error: ApiError) {
    match error {
        ApiError::NotFound => println!("Not found"),
        ApiError::Unauthorized => println!("Unauthorized"),
        ApiError::InternalError => println!("Internal error"),
        _ => println!("Unknown error"),                 // Required for non_exhaustive
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: Option chaining
fn option_chaining() {
    let value = Some(5)
        .map(|x| x * 2)
        .and_then(|x| if x > 5 { Some(x) } else { None })
        .unwrap_or(0);
    
    println!("Value: {}", value);
}

// Pattern 2: Result chaining
fn result_chaining() -> Result<i32, String> {
    divide(10, 2)?
        .checked_mul(3)
        .ok_or_else(|| "Overflow".to_string())
}

// Pattern 3: Custom error enum
#[derive(Debug)]
enum AppError {
    IoError(std::io::Error),
    ParseError(std::num::ParseIntError),
    CustomError(String),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(error: std::num::ParseIntError) -> Self {
        AppError::ParseError(error)
    }
}

fn app_function() -> Result<i32, AppError> {
    let contents = std::fs::read_to_string("file.txt")?;
    let number = contents.trim().parse::<i32>()?;
    Ok(number)
}

// Pattern 4: Event system
#[derive(Debug)]
enum Event {
    KeyPress(char),
    MouseClick { x: i32, y: i32 },
    Resize { width: u32, height: u32 },
    Quit,
}

fn handle_event(event: Event) {
    match event {
        Event::KeyPress(c) => println!("Key pressed: {}", c),
        Event::MouseClick { x, y } => println!("Mouse clicked at ({}, {})", x, y),
        Event::Resize { width, height } => println!("Resized to {}x{}", width, height),
        Event::Quit => println!("Quitting"),
    }
}

// Pattern 5: Command pattern
enum Command {
    Create { name: String },
    Update { id: u32, name: String },
    Delete { id: u32 },
    List,
}

impl Command {
    fn execute(&self) {
        match self {
            Command::Create { name } => println!("Creating: {}", name),
            Command::Update { id, name } => println!("Updating {} to {}", id, name),
            Command::Delete { id } => println!("Deleting {}", id),
            Command::List => println!("Listing all"),
        }
    }
}

// Pattern 6: Parser result
enum ParseResult<T> {
    Success(T, usize),                                   // value, bytes consumed
    Incomplete(usize),                                   // bytes needed
    Error(String),
}

// Pattern 7: Cow-like enum
enum MaybeOwned<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> MaybeOwned<'a> {
    fn as_str(&self) -> &str {
        match self {
            MaybeOwned::Borrowed(s) => s,
            MaybeOwned::Owned(s) => s.as_str(),
        }
    }
}

// Pattern 8: Validation result
enum Validation<T, E> {
    Valid(T),
    Invalid(Vec<E>),
}

impl<T, E> Validation<T, E> {
    fn is_valid(&self) -> bool {
        matches!(self, Validation::Valid(_))
    }
    
    fn errors(&self) -> Option<&Vec<E>> {
        match self {
            Validation::Invalid(errors) => Some(errors),
            _ => None,
        }
    }
}

// Pattern 9: Notification system
#[derive(Debug)]
enum Notification {
    Email { to: String, subject: String, body: String },
    Sms { to: String, message: String },
    Push { device_id: String, title: String, body: String },
}

impl Notification {
    fn send(&self) {
        match self {
            Notification::Email { to, subject, .. } => {
                println!("Sending email to {} with subject: {}", to, subject);
            }
            Notification::Sms { to, message } => {
                println!("Sending SMS to {}: {}", to, message);
            }
            Notification::Push { device_id, title, .. } => {
                println!("Sending push to {}: {}", device_id, title);
            }
        }
    }
}

// Pattern 10: AST (Abstract Syntax Tree)
enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> Result<i32, String> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::Add(left, right) => Ok(left.eval()? + right.eval()?),
            Expr::Subtract(left, right) => Ok(left.eval()? - right.eval()?),
            Expr::Multiply(left, right) => Ok(left.eval()? * right.eval()?),
            Expr::Divide(left, right) => {
                let r = right.eval()?;
                if r == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left.eval()? / r)
                }
            }
        }
    }
}

fn ast_example() {
    // (2 + 3) * 4
    let expr = Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Number(2)),
            Box::new(Expr::Number(3)),
        )),
        Box::new(Expr::Number(4)),
    );
    
    println!("Result: {}", expr.eval().unwrap());
}
```

