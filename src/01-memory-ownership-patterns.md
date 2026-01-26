# Memory & Ownership Patterns

When you are coming from other programming languages like Java, Python, C++, or Go. 
you probably know how to implement a linked list, a graph, or a cache. In Rust, your first attempt won't compile—and that's the point.

Rust replaces garbage collection and manual memory management with a third approach: **ownership**. The compiler tracks who is the owner of the data, who's borrowing it, and when it's freed. This eliminates null pointer bugs, data races, and use-after-free vulnerabilities at compile time - that is a good thing. But on the other side, you have to rethink how you organize code. 

This chapter covers:
- **Why assignment in Rust behaves differently** (move semantics vs. the copy-everywhere model you're used to)
- **The borrowing rules** that replace your garbage collector—and why "fighting the borrow checker" is a rite of passage
- **Lifetimes**: how the compiler proves references stay valid, without runtime overhead
- **Smart pointers** (Box, Rc, Arc, RefCell): the building blocks for complex data structures
- **Patterns** for shared state, interior mutability, and resource cleanup


## Pattern 1: Stack vs Heap

**Problem**: Programs need to store data somewhere. Stack allocation is fast but limited in size and flexibility. Heap allocation is flexible but slower and requires careful management.

**Solution**: Understand when Rust uses each: stack for fixed-size, short-lived data; heap for dynamic or large data. Use `Box`, `Vec`, `String` for heap allocation.

**Why It Matters**: Choosing the right allocation strategy affects performance by 10-100x. Stack allocation is ~1 CPU cycle; heap allocation involves system calls, locks, and fragmentation. Understanding this helps you write efficient code and avoid stack overflows.

### Example: Stack vs Heap Allocation

Stack-allocated data has a fixed size known at compile time and lives in the function's stack frame.
Heap-allocated types like Vec and String store a small header on the stack (pointer, length, capacity) with actual data on the heap.
When the scope ends, stack memory is reclaimed instantly; heap memory is freed via the allocator automatically.

```rust
fn stack_vs_heap() {
    // Stack: size known at compile time
    let x: i32 = 42;                 // 4 bytes on stack
    let arr: [i32; 100] = [0; 100];  // 400 bytes on stack

    // Heap: size can be dynamic
    let v: Vec<i32> = vec![1, 2, 3]; // 24B stack (ptr,len,cap)
                                     // +12B heap (data)

    let b: Box<i32> = Box::new(42);  // 8B stack (pointer)
                                     // +4B heap (the i32)

    let s = String::from("hello");   // 24B stack, +5B heap
}
// All memory freed here automatically
```

### Example: When Stack Fails

Large stack allocations risk stack overflow since thread stacks are typically 1-8MB.
The commented code would crash; the solution moves data to the virtually unlimited heap.
Vec handles this transparently—you get safe, large allocations without manual memory management.

```rust
// This would overflow the stack!
// fn stack_overflow() {
//     let huge: [u8; 10_000_000] = [0; 10_000_000];
// }

// Solution: Use heap allocation
fn heap_solution() {
    let huge: Vec<u8> = vec![0; 10_000_000]; // 10MB heap
    println!("Allocated {} bytes", huge.len());
}
```

### Example: Size Must Be Known

Trait objects like `dyn Animal` have no fixed size—Dog might be 8 bytes, Cat might be 16 bytes.
The compiler rejects direct stack storage since it cannot reserve the correct amount of space at compile time.
Use references (`&dyn`) or Box (`Box<dyn>`) for indirection with known pointer size.

```rust
trait Animal {
    fn speak(&self);
}

struct Dog;
impl Animal for Dog {
    fn speak(&self) { println!("Woof!"); }
}

// Can't store trait object directly on stack - size unknown
// let animal: dyn Animal = Dog; // Error!

// Solutions: use references or Box
fn with_reference(animal: &dyn Animal) {
    animal.speak();
}

fn with_box(animal: Box<dyn Animal>) {
    animal.speak();
}

let dog = Dog;
with_reference(&dog);
with_box(Box::new(Dog));
```

## Pattern 2: Move and Copy Semantics

**Problem**: When you assign a value or pass it to a function, should the original remain usable? Languages handle this differently—some copy everything, some share references, leading to bugs and confusion.

**Solution**: Rust uses move semantics by default: an assignment transfers ownership, that invalidates the original. Types can opt into `Copy` for implicit bitwise copying. Use `Clone` for explicit deep copies.

**Why It Matters**: Move semantics prevent use-after-free and double-free bugs at compile time. Understanding when values move vs copy lets you write code that compiles.

### Example: Move Semantics

When s1 is assigned to s2, ownership transfers completely and s1 becomes invalid immediately after assignment.
This prevents two variables from trying to free the same heap memory, avoiding dangerous double-free bugs.
Function calls work the same way: passing a value moves it entirely into the function's scope.

```rust
fn move_semantics() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is MOVED to s2

    // println!("{}", s1);  // Error! s1 is no longer valid
    println!("{}", s2);     // OK: s2 owns the data

    // Same with function calls
    let s3 = String::from("world");
    take_ownership(s3);
    // println!("{}", s3);  // Error! s3 was moved into function
}

fn take_ownership(s: String) {
    println!("Got: {}", s);
} // s is dropped here
```

### Example: Copy Types

Types implementing Copy are bitwise copied on assignment—both variables remain valid and fully independent.
Only types with no heap resources can be Copy: integers, floats, bools, chars, and shared references.
This is efficient (just memcpy) and inherently safe because there's no shared mutable state or double-free risk.

```rust
fn copy_semantics() {
    let x: i32 = 42;
    let y = x;  // x is COPIED to y

    println!("x={}, y={}", x, y);  // Both valid!

    // Primitives are Copy: i32, f64, bool, char
    // Tuples of Copy types: (i32, bool)
    // Arrays of Copy types: [i32; 10]
    // References are Copy: &T (not &mut T)
}

// Make your own type Copy (only if all fields are Copy)
#[derive(Copy, Clone)]
struct Point {
    x: f64,
    y: f64,
}

fn copy_custom_type() {
    let p1 = Point { x: 1.0, y: 2.0 };
    let p2 = p1;  // Copied!
    println!("p1: ({}, {})", p1.x, p1.y);  // Both valid
    println!("p2: ({}, {})", p2.x, p2.y);
}
```

### Example: Clone for Explicit Copies

Clone performs a deep copy—for String, this allocates new heap memory and copies all bytes to the new location.
Unlike implicit Copy, Clone requires an explicit `.clone()` call, making the performance cost visible in your code.
Use Clone when you need independent copies of heap-allocated data that can be modified separately.

```rust
fn explicit_clone() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // Explicit deep copy

    println!("s1={}, s2={}", s1, s2);  // Both valid!

    // Clone is explicit - makes cost visible
    let v1 = vec![1, 2, 3, 4, 5];
    let v2 = v1.clone();  // O(n) operation - visible
}

// Derive Clone for custom types
#[derive(Clone)]
struct Document {
    title: String,
    content: String,
}

fn clone_custom() {
    let doc1 = Document {
        title: "Report".into(),
        content: "...".into(),
    };
    let doc2 = doc1.clone();  // Deep copy of both Strings
}
```

### Example: Returning Ownership

Functions can give ownership to callers by returning owned values, transferring cleanup responsibility to the caller.
The transfer pattern—take ownership, process, return—avoids unnecessary cloning by moving data through each transformation.
This enables zero-cost abstractions: data flows through your program without intermediate copying or allocation overhead.

```rust
// Give ownership to caller
fn create_string() -> String {
    let s = String::from("created");
    s  // Ownership moves to caller
}

// Take and return ownership (transfer pattern)
fn process_and_return(mut s: String) -> String {
    s.push_str(" - processed");
    s  // Return ownership
}

fn ownership_transfer() {
    let s1 = create_string();
    let s2 = process_and_return(s1);
    // s1 is invalid, s2 is valid
    println!("{}", s2);
}
```

## Pattern 3: Borrowing Patterns

**Problem**: Moving ownership everywhere is inconvenient and inefficient. Sometimes you just want to read data or temporarily modify it without taking permanent ownership.

**Solution**: Borrowing creates references (`&T` for shared, `&mut T` for exclusive) that provide access without ownership transfer. The borrow checker enforces safety rules at compile time.

**Why It Matters**: Borrowing enables efficient data access patterns—pass large structs by reference instead of copying. The compile-time checks prevent data races, dangling pointers, and iterator invalidation bugs that plague other languages.

### Example: Shared Borrows (&T)

Multiple shared borrows can coexist simultaneously because they only allow reading, not modification of the data.
The function receives a reference, uses the data, but ownership remains with the caller throughout the operation.
This is the most common pattern: share read access widely while maintaining single ownership.

```rust
fn shared_borrows() {
    let s = String::from("hello");

    // Multiple shared borrows are OK
    let r1 = &s;
    let r2 = &s;
    let r3 = &s;

    println!("{}, {}, {}", r1, r2, r3);  // All valid simultaneously

    // Shared borrow in function - doesn't take ownership
    print_length(&s);
    println!("Still have s: {}", s);  // s still valid!
}

fn print_length(s: &String) {
    println!("Length: {}", s.len());
}  // s out of scope, nothing dropped (we don't own it)
```

### Example: Exclusive Borrows (&mut T)

Only one mutable borrow can exist at a time—this prevents data races at compile time.
While r1 is active, no other references (mutable or immutable) to s can exist.
After r1's last use, we can create r2—the borrow checker tracks actual usage, not just scope.

```rust
fn exclusive_borrows() {
    let mut s = String::from("hello");

    // Only ONE mutable borrow at a time
    let r1 = &mut s;
    r1.push_str(" world");
    // let r2 = &mut s;  // Error! Can't have two &mut
    println!("{}", r1);

    // After r1 is done, we can borrow again
    let r2 = &mut s;
    r2.push_str("!");
    println!("{}", r2);
}

fn modify_string(s: &mut String) {
    s.push_str(" - modified");
}

fn mutable_borrow_function() {
    let mut s = String::from("data");
    modify_string(&mut s);
    println!("{}", s);  // "data - modified"
}
```

### Example: Non-Lexical Lifetimes

NLL (Non-Lexical Lifetimes) allows borrows to end at their last use, not at the scope end.
Here, `first` is last used at println!, so the borrow ends there—not at the closing brace.
This enables the subsequent push() that would have been rejected in older Rust versions before NLL.

```rust
fn nll_example() {
    let mut data = vec![1, 2, 3];

    let first = &data[0];  // Immutable borrow starts
    println!("First: {}", first);  // Last use of `first`
    // Borrow ends here (NLL) - not at end of scope!

    data.push(4);  // OK! Mutable borrow is fine now
    println!("{:?}", data);
}

// Before NLL (Rust 2015), this wouldn't compile:
fn before_nll() {
    let mut data = vec![1, 2, 3];

    let first = &data[0];
    println!("First: {}", first);

    // In old Rust, `first` lived until }, blocking this:
    data.push(4);  // Now OK thanks to NLL
}
```

### Example: Reborrowing

Reborrowing creates a new borrow from an existing one, temporarily "freezing" the original.
When you pass `&mut *r1` or just `r` to a function, it reborrows rather than moves.
This lets you call multiple functions with the same mutable reference sequentially.

```rust
fn reborrow() {
    let mut data = String::from("hello");
    let r1: &mut String = &mut data;

    // Reborrow: new borrow from existing one
    let r2: &mut String = &mut *r1;  // r1 temporarily frozen
    r2.push_str(" world");
    // r1 is unfrozen when r2 goes out of scope

    r1.push_str("!");
    println!("{}", r1);
}

// Reborrowing happens automatically in function calls
fn takes_ref(s: &mut String) { s.push_str("!"); }

fn auto_reborrow() {
    let mut s = String::from("hello");
    let r = &mut s;

    takes_ref(r);  // r is reborrowed, not moved
    takes_ref(r);  // Can use r again
    println!("{}", r);
}
```

### Example: Borrow Splitting

The borrow checker understands that struct fields occupy separate memory—borrowing one field doesn't affect others.
You can simultaneously hold mutable borrows of `player.health` and `player.name` because they're disjoint memory regions.
Slices support splitting via `split_at_mut()`, which divides one slice into two non-overlapping mutable slices.

```rust
struct Player {
    name: String,
    health: i32,
    position: (f32, f32),
}

fn borrow_splitting() {
    let mut player = Player {
        name: "Hero".into(),
        health: 100,
        position: (0.0, 0.0),
    };

    // Can borrow different fields mutably at same time
    let name = &player.name;          // Immutable borrow
    let health = &mut player.health;  // Mutable borrow

    *health -= 10;
    println!("{} has {} health", name, health);
}

// Works with slices too
fn slice_splitting() {
    let mut arr = [1, 2, 3, 4, 5];
    let (left, right) = arr.split_at_mut(2);

    left[0] = 10;   // Mutate left half
    right[0] = 30;  // Mutate right half simultaneously

    println!("{:?}", arr);  // [10, 2, 30, 4, 5]
}
```

## Pattern 4: Lifetime Patterns

**Problem**: References must not outlive the data they point to. The compiler needs to verify this, but tracking every reference manually would be tedious and error-prone.

**Solution**: Lifetimes are annotations that describe how long references are valid. The compiler infers most lifetimes automatically; you only annotate when relationships are ambiguous.

**Why It Matters**: Lifetimes eliminate dangling pointer bugs—a major source of security vulnerabilities in C/C++. Understanding lifetime elision rules means you rarely write explicit annotations, while the compiler still guarantees safety.

### Example: Lifetime Elision (The Common Case)

Elision rules cover ~90% of cases: single input lifetime flows to output, &self wins for methods.
Rule 1 gives each parameter its own lifetime; Rule 2 propagates single inputs to outputs.
You only need explicit annotations when the compiler can't determine which input lifetime applies.

```rust
// The compiler infers lifetimes in these common cases:

// Rule 1: Each reference parameter gets its own lifetime
fn print(s: &str) { println!("{}", s); }
// Compiler sees: fn print<'a>(s: &'a str)

// Rule 2: If one input lifetime, output gets that lifetime
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
// Compiler sees: fn first_word<'a>(s: &'a str) -> &'a str

// Rule 3: If &self or &mut self, output gets self's lifetime
struct Player { name: String }
impl Player {
    fn name(&self) -> &str {
        &self.name
    }
    // Compiler sees: fn name<'a>(&'a self) -> &'a str
}
```

### Example: When You Need Explicit Lifetimes

With two input references, the compiler can't guess which lifetime the output should have.
The annotation `'a` says: the returned reference lives as long as both inputs do.
This forces callers to ensure both inputs outlive any use of the returned reference.

```rust
// Multiple input references - compiler can't guess which to use
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn use_longest() {
    let s1 = String::from("long string");
    let result;
    {
        let s2 = String::from("short");
        result = longest(&s1, &s2);
        println!("Longest: {}", result);  // Must use here, while s2 valid
    }
    // println!("{}", result);  // Error if uncommented: s2 dropped
}

// Different lifetimes for different relationships
fn first_or_default<'a, 'b>(
    first: &'a str,
    default: &'b str
) -> &'a str {
    if !first.is_empty() { first }
    else { first }  // Can't return default - wrong lifetime
}
```

### Example: Struct Lifetimes

Structs holding references need lifetime parameters—the struct can't outlive the data it borrows from.
`Parser<'a>` means "this parser borrows something with lifetime 'a and must be dropped before that data is."
Methods can return references tied to the struct's lifetime, letting the compiler verify all references remain valid throughout their use.

```rust
// Struct that borrows data
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.position..]
    }

    fn advance(&mut self, n: usize) {
        self.position += n;
    }
}

fn use_parser() {
    let text = String::from("hello world");
    let mut parser = Parser::new(&text);

    println!("Remaining: {}", parser.remaining());
    parser.advance(6);
    println!("Remaining: {}", parser.remaining());
}
// parser must not outlive text
```

### Example: 'static Lifetime

`'static` means the reference is valid for the entire program—string literals qualify automatically.
Owned data like String satisfies `T: 'static` because it doesn't borrow anything non-static.
Thread spawning requires `'static` because the thread might outlive the spawning scope.

```rust
// 'static means "lives for entire program"

// String literals are 'static
fn static_literal() {
    let s: &'static str = "I live forever";
    println!("{}", s);
}

// Owned data can satisfy 'static (it's not borrowed)
fn needs_static<T: 'static>(value: T) {
    // T contains no non-'static references
}

fn static_examples() {
    needs_static(String::from("owned"));  // OK: owned data
    needs_static(42i32);                   // OK: Copy type
    needs_static(vec![1, 2, 3]);          // OK: owned Vec

    let local = String::from("local");
    // needs_static(&local);  // Error: &local is not 'static
}

// Common with threads - data must be 'static or moved
use std::thread;

fn thread_static() {
    let data = vec![1, 2, 3];

    // Move ownership into thread (data becomes 'static-like)
    thread::spawn(move || {
        println!("{:?}", data);
    }).join().unwrap();
}
```

## Pattern 5: Flexible APIs with Conversion Traits

**Problem**: You want functions that accept multiple types (`String`, `&str`, `PathBuf`, `&Path`) without writing overloads for each combination.

**Solution**: Use conversion traits: `AsRef<T>` for borrowing as T, `Into<T>` for consuming and converting, `Borrow<T>` for hash-compatible borrowing, plus Deref coercion for automatic reference conversion.

**Why It Matters**: These traits make APIs ergonomic—callers pass whatever type is convenient. Library authors write one function that works with many types. This is how `std::fs::read("path")` accepts both `&str` and `PathBuf`.

### Example: AsRef for Read-Only Access

`AsRef<Path>` means "anything that can be viewed as a Path without requiring ownership transfer or allocation."
The function works with &str, String, PathBuf, and &Path—all implement AsRef<Path> in the standard library.
No heap allocation occurs; each type simply provides a borrowed reference to its path-like content.

```rust
use std::path::Path;

// Accept anything that can be viewed as a Path
fn file_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

fn use_asref() {
    // All of these work:
    file_exists("config.txt");            // &str
    file_exists(String::from("log.txt")); // String
    file_exists(Path::new("data.bin"));   // &Path

    use std::path::PathBuf;
    file_exists(PathBuf::from("/tmp"));   // PathBuf
}

// For strings, use AsRef<str>
fn count_words(t: impl AsRef<str>) -> usize {
    t.as_ref().split_whitespace().count()
}

fn use_asref_str() {
    count_words("hello world");            // &str
    count_words(String::from("hi there")); // String
}
```

### Example: Into for Ownership Transfer

`Into<String>` means "anything that can be converted into an owned String" through the conversion trait.
For &str, this allocates a new String; for String, it's a no-op move with zero cost.
Use Into when you need to store owned data but want flexible input types for callers.

```rust
// Accept anything convertible to String
fn greet(name: impl Into<String>) {
    let name: String = name.into();
    println!("Hello, {}!", name);
}

fn use_into() {
    greet("Alice");              // &str -> String (allocates)
    greet(String::from("Bob"));  // String -> String (no-op)
    greet('X');                  // char -> String
}

// Builder pattern with Into
struct Request {
    url: String,
    method: String,
}

impl Request {
    fn new(url: impl Into<String>) -> Self {
        Request {
            url: url.into(),
            method: "GET".into(),
        }
    }

    fn method(mut self, m: impl Into<String>) -> Self {
        self.method = m.into();
        self
    }
}

fn builder_example() {
    let req = Request::new("https://example.com")
        .method("POST");
}
```

### Example: Borrow Trait for HashMap Keys

Borrow is stricter than AsRef—borrowed form must have same Hash/Eq as owned form.
This lets HashMap<String, V> be queried with &str without allocating a String key.
The constraint `String: Borrow<Q>` ensures Q can stand in for String in lookups.

```rust
use std::collections::HashMap;
use std::borrow::Borrow;

fn lookup<Q>(
    map: &HashMap<String, i32>,
    key: &Q
) -> Option<i32>
where
    String: Borrow<Q>,
    Q: Eq + std::hash::Hash + ?Sized,
{
    map.get(key).copied()
}

fn use_borrow() {
    let mut scores: HashMap<String, i32> = HashMap::new();
    scores.insert("Alice".into(), 100);
    scores.insert("Bob".into(), 85);

    // Can lookup with &str even though keys are String
    let alice_score = scores.get("Alice");  // Works!

    // Our generic function works with both
    assert_eq!(lookup(&scores, "Alice"), Some(100));
    let bob = String::from("Bob");
    assert_eq!(lookup(&scores, &bob), Some(85));
}
```

### Example: Deref Coercion

Deref coercion automatically converts &T to &U when T implements Deref<Target=U>, eliminating explicit conversions.
This is why you can pass &String to functions expecting &str—String implements Deref with Target=str.
Coercion chains work too: &Box<String> → &String → &str, with the compiler applying multiple steps automatically.

```rust
// Deref coercion: &T -> &U if T: Deref<Target=U>

fn print_str(s: &str) { println!("{}", s); }

fn deref_coercion() {
    let owned = String::from("hello");
    let boxed = Box::new(String::from("world"));

    // All automatically coerce to &str
    print_str(&owned);    // &String -> &str
    print_str(&boxed);    // &Box<String> -> &str
    print_str("literal"); // &str -> &str

    // Works with slices too
    fn sum(n: &[i32]) -> i32 { n.iter().sum() }

    let v = vec![1, 2, 3];
    let a = [4, 5, 6];

    sum(&v);  // &Vec<i32> -> &[i32]
    sum(&a);  // &[i32; 3] -> &[i32]
}
```

### Example: From/Into Implementation

Implement From<T> and you automatically get Into<T> for free via a blanket implementation in the standard library.
Custom From impls define exactly how your types convert from standard types like integers, strings, or other primitives.
This integrates your types seamlessly with the ecosystem—they work with all generic Into bounds without additional implementation effort.

```rust
struct UserId(u64);

// Implement From, get Into for free
impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        UserId(id)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        UserId(s.parse().unwrap_or(0))
    }
}

fn create_user(id: impl Into<UserId>) {
    let user_id: UserId = id.into();
    println!("User ID: {}", user_id.0);
}

fn use_from_into() {
    create_user(42u64);      // u64 -> UserId
    create_user("12345");    // &str -> UserId
}
```

## Pattern 6: Box for Heap Allocation

**Problem**: Stack allocation has limits: recursive types have infinite size, large structs risk stack overflow, and trait objects have unknown size at compile time.

**Solution**: `Box<T>` allocates data on the heap, storing only an 8-byte pointer on the stack. The data is automatically freed when the Box goes out of scope.

**Why It Matters**: Box is the simplest smart pointer—single ownership with heap allocation. It enables recursive data structures, avoids stack overflow for large data, and provides the indirection needed for trait objects.

### Example: Recursive Types Require Box

Without Box, List would contain List which contains List—infinite size at compile time, which is impossible.
Box provides indirection: each Cons holds a pointer (8 bytes) to the next node on the heap.
The compiler can now calculate List's size: max of Cons (T + 8 bytes) or Nil plus tag.

```rust
#[derive(Debug)]
enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn new() -> Self {
        List::Nil
    }

    fn prepend(self, elem: T) -> Self {
        List::Cons(elem, Box::new(self))
    }
}

// Usage: Build a linked list
let list = List::new().prepend(3).prepend(2).prepend(1);
println!("{:?}", list); // Cons(1, Cons(2, Cons(3, Nil)))
```

### Example: Trait Objects with Box

Different types implementing Drawable have different sizes—Circle is 8 bytes, Rectangle is 16 bytes in memory.
Box<dyn Drawable> erases the concrete type, storing a fat pointer containing both data pointer and vtable.
This enables heterogeneous collections where each element can be a different concrete type at runtime.

```rust
trait Drawable {
    fn draw(&self);
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Drawable for Circle {
    fn draw(&self) { println!("Circle r={}", self.radius); }
}

impl Drawable for Rectangle {
    fn draw(&self) {
        println!("Rect {}x{}", self.width, self.height);
    }
}

// Store different types in one collection
let shapes: Vec<Box<dyn Drawable>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rectangle { width: 10.0, height: 20.0 }),
];
for s in &shapes { s.draw(); }
```

## Pattern 7: Rc and Arc for Shared Ownership

**Problem**: Rust's ownership model allows only one owner per value, but some data structures (graphs, shared configuration, caches) need multiple owners pointing to the same data.

**Solution**: `Rc<T>` (Reference Counted) and `Arc<T>` (Atomic Reference Counted) allow multiple owners. Each clone increments a counter; when the last owner drops, the data is freed.

**Why It Matters**: Shared ownership enables data structures impossible with single ownership: graphs with cycles, multiple components sharing configuration, subscriber lists. Rc is for single-threaded use; Arc adds thread safety.

### Example: Shared Configuration with Rc

Rc::clone() increments the reference count (cheap: just a counter bump, not a deep copy of data).
Multiple components hold Rc pointers to the same Config allocation stored on the heap memory.
When all Rcs are dropped, the count reaches zero and the Config is deallocated automatically.

```rust
use std::rc::Rc;

struct Config {
    database_url: String,
    max_connections: usize,
}

struct DatabasePool { config: Rc<Config> }
struct CacheService { config: Rc<Config> }

// Share config across components
let cfg = Rc::new(Config {
    database_url: "postgres://localhost/db".into(),
    max_connections: 100,
});

println!("Refs: {}", Rc::strong_count(&cfg)); // 1

let db = DatabasePool { config: Rc::clone(&cfg) };
let cache = CacheService { config: Rc::clone(&cfg) };

println!("Refs: {}", Rc::strong_count(&cfg)); // 3
```

### Example: Arc for Thread-Safe Sharing

Rc uses non-atomic operations—fast but unsafe across threads due to potential data races on the counter.
Arc uses atomic operations for its reference count, making it safe for concurrent access across threads.
Each thread gets its own Arc handle; they all point to the same heap-allocated Vec safely.

```rust
use std::sync::Arc;
use std::thread;

let data = Arc::new(vec![1, 2, 3, 4, 5]);
let mut handles = vec![];

for i in 0..3 {
    let d = Arc::clone(&data);
    handles.push(thread::spawn(move || {
        let sum: i32 = d.iter().sum();
        println!("Thread {}: sum={}", i, sum);
    }));
}

for handle in handles {
    handle.join().unwrap();
}
```

## Pattern 8: Interior Mutability

**Problem**: Rust requires `&mut self` for mutation, but some patterns need mutation through shared references (`&self`): caching computed values, counters, graph node updates.

**Solution**: Interior mutability types move borrow checking from compile-time to runtime. Cell for Copy types (get/set), RefCell for any type (borrow/borrow_mut), Mutex/RwLock for thread safety.

**Why It Matters**: Some data structures are impossible without interior mutability: caches that compute on first access, observer patterns, graph algorithms. These types let you have shared ownership + mutation safely.

### Example: Cell for Copy Types

Cell allows mutation through &self by never giving out references to the inner data it contains.
You can only get() copies out or set() new values in—no borrowing the contents is ever allowed.
Zero runtime overhead: just get and set operations, no borrow tracking needed for Copy types.

```rust
use std::cell::Cell;

struct Counter {
    count: Cell<usize>,
}

impl Counter {
    fn new() -> Self { Counter { count: Cell::new(0) } }

    fn increment(&self) {  // &self, not &mut self!
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> usize { self.count.get() }
}

let counter = Counter::new();
counter.increment();
counter.increment();
println!("Count: {}", counter.get()); // 2
```

### Example: RefCell for Complex Types

RefCell tracks borrows at runtime rather than compile time: borrow() creates shared access, borrow_mut() creates exclusive access.
Violating borrow rules (two simultaneous mutable borrows) causes a panic instead of a compile error.
Use RefCell for complex types that can't be copied, when you need interior mutation through &self.

```rust
use std::cell::RefCell;
use std::collections::HashMap;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Self {
        Cache { data: RefCell::new(HashMap::new()) }
    }

    fn get_or_compute(
        &self,
        key: &str,
        compute: impl FnOnce() -> String
    ) -> String {
        if let Some(v) = self.data.borrow().get(key) {
            return v.clone();
        }
        let v = compute();
        self.data.borrow_mut().insert(key.into(), v.clone());
        v
    }
}
```

### Example: Rc<RefCell<T>> for Shared Mutable Data

Rc provides shared ownership allowing multiple references to the same data; RefCell provides interior mutability—combined, they enable shared mutation.
Multiple Rc handles pointing to the same RefCell can call borrow_mut() to modify the shared data.
This pattern enables graph structures where nodes need to modify neighbors without a single owner coordinating mutations.

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    value: i32,
    neighbors: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(Node {
            value,
            neighbors: RefCell::new(Vec::new()),
        })
    }

    fn add_neighbor(&self, neighbor: Rc<Node>) {
        self.neighbors.borrow_mut().push(neighbor);
    }
}

let a = Node::new(1);
let b = Node::new(2);
a.add_neighbor(Rc::clone(&b));
b.add_neighbor(Rc::clone(&a)); // Cycle is allowed
```

### Example: Arc<Mutex<T>> for Thread-Safe Mutation

Mutex provides exclusive access to shared data: lock() blocks the current thread until available, then returns a guard.
The guard automatically releases the lock when dropped—you cannot forget to unlock because the compiler handles it.
Arc<Mutex<T>> is the standard pattern for sharing mutable state safely across multiple threads in concurrent programs.

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    handles.push(thread::spawn(move || {
        for _ in 0..100 {
            let mut num = counter.lock().unwrap();
            *num += 1;
        }
    }));
}

for handle in handles {
    handle.join().unwrap();
}

println!("Final: {}", *counter.lock().unwrap());
```

## Pattern 9: Breaking Cycles with Weak

**Problem**: When A holds `Rc<B>` and B holds `Rc<A>`, reference counts never reach zero—memory leak! The cycle keeps both alive forever.

**Solution**: `Weak<T>` is a non-owning reference that doesn't increment the strong count. Use Rc for ownership relationships (parent→child), Weak for back-references (child→parent).

**Why It Matters**: Cycles are common in graphs, trees with parent pointers, observer patterns, and caches. Weak references let you express "I reference this but don't own it," breaking cycles while maintaining safe access.

### Example: Tree with Parent Pointers

Children are owned by parents (Rc in children vec), but parent references are weak (non-owning).
Rc::downgrade() creates a Weak from an Rc without incrementing the strong reference count.
upgrade() returns Option<Rc>—None if the referenced data was already dropped, Some if still alive.

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct TreeNode {
    value: i32,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc<Self> {
        Rc::new(TreeNode {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
        })
    }

    fn add_child(
        parent: &Rc<TreeNode>,
        value: i32
    ) -> Rc<TreeNode> {
        let child = TreeNode::new(value);
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(Rc::clone(&child));
        child
    }
}

let root = TreeNode::new(1);
let child = TreeNode::add_child(&root, 2);

if let Some(parent) = child.parent.borrow().upgrade() {
    println!("Parent value: {}", parent.value);
}
```

## Pattern 10: Clone-on-Write with Cow

**Problem**: Functions that may or may not modify input face a dilemma: always clone (wasteful if no changes needed) or require mutable reference (inflexible API, forces callers to own data).

**Solution**: `Cow<T>` (Clone on Write) holds either borrowed (`Cow::Borrowed`) or owned (`Cow::Owned`) data. Read access works for both; mutation clones borrowed data first.

**Why It Matters**: Cow provides zero-allocation fast paths when no modification is needed, while still supporting modification when required. Used extensively in parsing, text processing, and configuration systems.

### Example: Conditional String Processing

If input needs no changes, return Cow::Borrowed—zero allocation overhead, simply wraps and returns the original input reference.
If modifications are required, create an owned String with the changes and return Cow::Owned to transfer ownership.
Callers can use the result uniformly regardless of which variant it is; Cow implements Deref to &str for seamless reading access.

```rust
use std::borrow::Cow;

fn normalize_whitespace(text: &str) -> Cow<'_, str> {
    if text.contains("  ") || text.contains('\t') {
        Cow::Owned(text.replace("  ", " ").replace('\t', " "))
    } else {
        Cow::Borrowed(text)
    }
}

let clean = normalize_whitespace("hello world");   // Borrowed
let fixed = normalize_whitespace("hello  world");  // Owned
```

### Example: Configuration with Defaults

Static defaults are borrowed directly from string literals embedded in the binary (no heap allocation, &'static str lifetime).
User-provided overrides become owned Strings stored in the Cow::Owned variant, allocated only when customization is actually needed.
This pattern provides zero-cost defaults for the common case while fully supporting runtime customization without any API complexity for callers.

```rust
use std::borrow::Cow;

struct Config<'a> {
    host: Cow<'a, str>,
    database: Cow<'a, str>,
}

impl<'a> Config<'a> {
    fn new(host: &'a str) -> Self {
        Config {
            host: Cow::Borrowed(host),
            database: Cow::Borrowed("default_db"),
        }
    }

    fn with_database(mut self, db: String) -> Self {
        self.database = Cow::Owned(db);
        self
    }
}
```

## Pattern 11: Drop Guards (RAII)

**Problem**: Manual resource cleanup is error-prone—forgetting to close files, release locks, restore state, or rollback transactions causes leaks, deadlocks, and corruption.

**Solution**: Implement the `Drop` trait to tie cleanup to scope. Create guard types that acquire resources on construction and release them automatically when dropped.

**Why It Matters**: RAII (Resource Acquisition Is Initialization) makes resource leaks impossible. You can't forget to unlock a Mutex—MutexGuard's Drop does it. This pattern eliminates entire categories of bugs.

### Example: Scope Guard for Rollback

ScopeGuard holds a cleanup closure in an Option that executes automatically when the guard is dropped, unless disarmed.
If the operation fails or panics, the cleanup closure runs automatically—ensuring the transaction gets rolled back properly.
On success, call disarm() to consume the closure without running it—the transaction remains committed without rollback.

```rust
struct ScopeGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    fn new(cleanup: F) -> Self {
        ScopeGuard { cleanup: Some(cleanup) }
    }

    fn disarm(mut self) {
        self.cleanup = None;
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

// Usage: Rollback runs unless disarmed
fn transaction() {
    let guard = ScopeGuard::new(|| println!("Rollback"));
    // ... do work ...
    guard.disarm(); // Success!
}
```

### Example: Timing Guard

Timer records the start time on construction; Drop calculates and prints the elapsed time automatically.
No matter how the scope exits (return, panic, break, or normal completion), timing is always recorded.
The underscore prefix `_timer` tells Rust "I know this appears unused, but it's intentional."

```rust
use std::time::Instant;

struct Timer<'a> {
    name: &'a str,
    start: Instant,
}

impl<'a> Timer<'a> {
    fn new(name: &'a str) -> Self {
        Timer { name, start: Instant::now() }
    }
}

impl Drop for Timer<'_> {
    fn drop(&mut self) {
        println!("{}: {:?}", self.name, self.start.elapsed());
    }
}

// Usage: Automatically prints elapsed time
fn do_work() {
    let _timer = Timer::new("do_work");
    // ... expensive operation ...
} // Prints "do_work: 123ms" when scope ends
```

## Pattern 12: Arena Allocation

**Problem**: Many small allocations are slow. Each `Box::new()` or `Vec::push()` involves the system allocator—locks, fragmentation, syscalls. Allocating thousands of small objects (AST nodes, game entities, request handlers) becomes a bottleneck.

**Solution**: Pre-allocate a large memory chunk, then bump-allocate within it. Allocation becomes a pointer increment. Deallocation happens all at once when the arena is dropped.

**Why It Matters**: Arena allocation is 10-100x faster than individual allocations. Java added arenas in JDK 21 (JEP 454). Game engines use per-frame arenas. Compilers allocate entire ASTs in arenas. This pattern appears wherever allocation performance matters.

### How It Works

**System allocator** (malloc/jemalloc):
```
allocate(32 bytes):
  1. Acquire lock (thread safety)
  2. Search free lists for suitable block
  3. Split block if too large
  4. Update bookkeeping metadata
  5. Release lock
  6. Return pointer
```

**Arena allocator** (bump allocation):
```
allocate(32 bytes):
  1. pointer = current_position
  2. current_position += 32
  3. Return pointer
```

That's it. No locks, no searching, no metadata. Just increment a pointer.

### Trade-offs

| Aspect | Arena | System Allocator |
|--------|-------|------------------|
| Allocation speed | O(1) bump | O(log n) or worse |
| Individual dealloc | Not possible | Yes |
| Memory overhead | Minimal | Per-alloc metadata |
| Fragmentation | None in arena | Can fragment |
| Thread safety | One per thread | Built-in (locks) |
| Best for | Many short-lived | Long-lived objects |

**When to use arenas:**
- Compilers: AST nodes, type info, symbol tables
- Games: Per-frame entities, particle systems
- Servers: Per-request allocations
- Parsers: Token streams, parse trees

**When NOT to use arenas:**
- Objects with vastly different lifetimes
- When you need to free individual objects
- Long-running processes where arena would grow unbounded

### Example: Bump Allocator

Allocate by simply incrementing a position pointer within a pre-allocated chunk of memory.
When the current chunk fills up, allocate a new chunk and save the old one for later cleanup.
All memory is freed when the arena drops—no individual deallocations are needed throughout its lifetime.

```rust
struct Arena {
    chunks: Vec<Vec<u8>>,
    current: Vec<u8>,
    position: usize,
}

impl Arena {
    fn new() -> Self {
        Arena {
            chunks: Vec::new(),
            current: vec![0; 4096],
            position: 0,
        }
    }

    fn alloc<T>(&mut self, value: T) -> &mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        // Align position
        let pad = (align - (self.position % align)) % align;
        self.position += pad;

        // New chunk if needed
        if self.position + size > self.current.len() {
            let old = std::mem::replace(
                &mut self.current, vec![0; 4096]
            );
            self.chunks.push(old);
            self.position = 0;
        }

        let ptr = self.current[self.position..].as_mut_ptr();
        let ptr = ptr as *mut T;
        self.position += size;

        unsafe {
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }
}

// Usage: Fast allocation for many small objects
let mut arena = Arena::new();

// Allocate each value (borrow checker requires sequential use)
let a = arena.alloc(42i32);
println!("Allocated: {}", a);

let b = arena.alloc(String::from("hello"));
println!("Allocated: {}", b);
// All memory freed when arena drops
```

### Example: AST Arena

Compilers build abstract syntax trees with thousands of interconnected nodes that reference each other in complex patterns.
Arena allocation eliminates per-node allocation overhead and enables dramatically simpler lifetime management compared to individual heap allocations.
All nodes live exactly as long as the arena does—no reference counting, garbage collection, or complex ownership tracking required for safe memory management.

```rust
enum Expr<'a> {
    Number(i64),
    Add(&'a Expr<'a>, &'a Expr<'a>),
    Mul(&'a Expr<'a>, &'a Expr<'a>),
}

struct AstArena {
    arena: Arena,
}

impl AstArena {
    fn new() -> Self {
        AstArena { arena: Arena::new() }
    }

    fn number(&mut self, n: i64) -> &Expr {
        self.arena.alloc(Expr::Number(n))
    }

    fn add<'a>(
        &'a mut self,
        l: &'a Expr<'a>,
        r: &'a Expr<'a>
    ) -> &'a Expr<'a> {
        self.arena.alloc(Expr::Add(l, r))
    }
}

// Usage: Build AST with fast allocation
// For production, use typed-arena or bumpalo crates
```

### Example: Per-Request Arena (Web Server Pattern)

Web servers handle requests independently—each request allocates temporary data, processes it, then frees everything when the response is sent.
An arena per request means zero memory fragmentation between requests and instant cleanup when each request completes (just drop the arena).
This pattern is common in high-throughput servers handling thousands of requests per second and in game engines that reset per-frame allocations.

```rust
struct RequestArena {
    arena: Arena,
}

impl RequestArena {
    fn new() -> Self {
        RequestArena { arena: Arena::new() }
    }

    fn alloc_str(&mut self, s: &str) -> &str {
        let bytes = self.arena.alloc_slice(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

impl Arena {
    fn alloc_slice<T: Copy>(&mut self, slice: &[T]) -> &mut [T] {
        let size = std::mem::size_of::<T>() * slice.len();
        let align = std::mem::align_of::<T>();

        let pad = (align - (self.position % align)) % align;
        self.position += pad;

        if self.position + size > self.current.len() {
            let new_sz = (size + 4095) & !4095;
            let old = std::mem::replace(
                &mut self.current, vec![0; new_sz.max(4096)]
            );
            self.chunks.push(old);
            self.position = 0;
        }

        let ptr = self.current[self.position..].as_mut_ptr();
        let ptr = ptr as *mut T;
        self.position += size;

        unsafe {
            std::ptr::copy_nonoverlapping(
                slice.as_ptr(), ptr, slice.len()
            );
            std::slice::from_raw_parts_mut(ptr, slice.len())
        }
    }
}

// Usage: Each request gets its own arena
fn handle_request(data: &str) {
    let mut arena = RequestArena::new();
    let parsed = arena.alloc_str(data);
    // ... process request using arena for all allocations ...
} // All request memory freed instantly here
```

### Lifetimes and Arenas

Arena-allocated references are tied to the arena's lifetime. This is both a feature and a constraint:

```rust
struct Arena { /* ... */ }

// All references share the arena's lifetime
fn build_tree<'a>(arena: &'a Arena) -> &'a Node<'a> {
    let left = arena.alloc(Node::Leaf(1));   // &'a Node
    let right = arena.alloc(Node::Leaf(2));  // &'a Node
    arena.alloc(Node::Branch(left, right))   // &'a Node
}

// The arena owns everything; references are just views
let arena = Arena::new();
let tree = build_tree(&arena);
// tree is valid as long as arena exists
drop(arena);  // All nodes freed, tree is now invalid
```

This lifetime coupling eliminates the need for `Rc` or reference counting—the arena guarantees all allocations live together.

### Production Crates

Don't roll your own arena for production code. Use battle-tested crates:

| Crate | Best For | Notes |
|-------|----------|-------|
| `bumpalo` | General | Popular, `#![no_std]` |
| `typed-arena` | Single type | Simpler, type-safe |
| `toolshed` | Multi-type | Arena + interning |
| `id-arena` | Index access | Returns indices |

```rust
// Using bumpalo (recommended)
use bumpalo::Bump;

let bump = Bump::new();
let x = bump.alloc(42);
let s = bump.alloc_str("hello");
let v = bump.alloc_slice_copy(&[1, 2, 3]);

// Using typed-arena
use typed_arena::Arena;

let arena: Arena<Node> = Arena::new();
let node1 = arena.alloc(Node::new(1));
let node2 = arena.alloc(Node::new(2));
```

### Memory Layout Visualization

```
┌─────────────────────────────────────────────────────┐
│                    Arena                            │
├─────────────────────────────────────────────────────┤
│  Chunk 0 (full)     │  Chunk 1 (full)    │ Current  │
│  ┌───┬───┬───┬───┐  │  ┌───┬───┬───┐     │ Chunk    │
│  │ A │ B │ C │ D │  │  │ E │ F │ G │     │ ┌───┬──┐ │
│  └───┴───┴───┴───┘  │  └───┴───┴───┘     │ │ H │░░│ │
│                     │                    │ └───┴──┘ │
│                     │                    │     ↑    │
│                     │                    │  position│
└─────────────────────────────────────────────────────┘

Allocation: just move position pointer right
Deallocation: drop entire arena (all chunks freed at once)
```

---

## Quick Reference

### Ownership Cheat Sheet

| Situation | Pattern |
|-----------|---------|
| Need heap allocation | `Box<T>` |
| Share data, single-threaded | `Rc<T>` |
| Share data, multi-threaded | `Arc<T>` |
| Mutate through `&self`, Copy type | `Cell<T>` |
| Mutate through `&self`, any type | `RefCell<T>` |
| Mutate shared data, multi-threaded | `Arc<Mutex<T>>` |
| Break reference cycles | `Weak<T>` |
| Maybe clone, maybe borrow | `Cow<T>` |
| Cleanup on scope exit | `Drop` trait |

### Borrowing Rules

```rust
// OK: Multiple immutable borrows
let r1 = &data;
let r2 = &data;

// OK: One mutable borrow
let r1 = &mut data;

// ERROR: Can't mix
let r1 = &data;
let r2 = &mut data;  // Error!

// OK: Borrows of different fields
let r1 = &mut data.field1;
let r2 = &mut data.field2;
```

### Common Mistakes

```rust
// ❌ Using Arc when single-threaded
let data = Arc::new(Mutex::new(vec![])); // Overhead

// ✓ Use Rc<RefCell> for single-threaded
let data = Rc::new(RefCell::new(vec![]));

// ❌ Holding borrow across potential panic
let borrowed = data.borrow();
might_panic();  // If panics, borrow not dropped

// ✓ Scope borrows tightly
{
    let borrowed = data.borrow();
    use_borrowed(&borrowed);
}
might_panic();

// ❌ Returning reference to local
fn bad() -> &str {
    let s = String::from("local");
    &s  // Error! s dropped at end of function
}

// ✓ Return owned data
fn good() -> String {
    String::from("owned")
}
```
