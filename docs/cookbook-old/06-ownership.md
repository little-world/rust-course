# Ownership, Borrowing, and Lifetimes

> A comprehensive guide to Rust's memory management system for experienced programmers

## Table of Contents

1. [Understanding the Problem](#understanding-the-problem)
2. [Ownership Fundamentals](#ownership-fundamentals)
3. [Borrowing and References](#borrowing-and-references)
4. [Scope and Drop](#scope-and-drop)
5. [Lifetimes](#lifetimes)
6. [Function Calling Patterns](#function-calling-patterns)
7. [Advanced Lifetime Scenarios](#advanced-lifetime-scenarios)
8. [Common Patterns and Idioms](#common-patterns-and-idioms)
9. [Comparison with Other Languages](#comparison-with-other-languages)
10. [Practical Guidelines](#practical-guidelines)

---

## Understanding the Problem

### The Memory Management Trilemma

Most languages choose one of these approaches:

**Manual Memory Management (C, C++)**:
- Full control over allocation/deallocation
- Risk: Use-after-free, double-free, memory leaks, dangling pointers
- No runtime overhead

**Garbage Collection (Java, Python, Go, JavaScript)**:
- Automatic memory reclamation
- Risk: Unpredictable pauses, runtime overhead
- Easy to use

**Rust's Approach: Ownership System**:
- Compile-time memory safety guarantees
- Zero runtime overhead
- Learning curve: You must understand the rules

### What Rust Guarantees

Rust's ownership system prevents at compile time:

1. **Use-after-free**: Cannot use memory after it's been freed
2. **Double-free**: Cannot free the same memory twice
3. **Dangling pointers**: All references are guaranteed valid
4. **Data races**: Cannot have concurrent mutable access

The trade-off is that you must satisfy the borrow checker at compile time.

---

## Ownership Fundamentals

### The Three Rules

1. **Each value has a single owner** (the variable that holds it)
2. **When the owner goes out of scope, the value is dropped**
3. **Ownership can be transferred (moved) but not duplicated** (unless explicitly cloned)

### Stack vs Heap

Understanding ownership requires understanding where data lives:

**Stack** (fixed-size, known at compile time):
- Integers, floats, booleans, chars
- Fixed-size arrays: `[i32; 5]`
- Tuples of stack types: `(i32, f64)`
- These implement `Copy` trait - assignment copies the value

**Heap** (dynamic-size, known at runtime):
- `String` (not `&str`)
- `Vec<T>`
- `Box<T>`
- Custom structs with heap data
- These use move semantics - assignment transfers ownership

### Copy vs Move

```rust
// Copy types: Assignment duplicates the value
fn copy_example() {
    let x = 5;        // i32 is Copy
    let y = x;        // x is copied to y
    println!("{} {}", x, y);  // ✅ Both valid
}

// Move types: Assignment transfers ownership
fn move_example() {
    let s1 = String::from("hello");  // String is NOT Copy
    let s2 = s1;                     // s1 moves to s2
    // println!("{}", s1);            // ❌ Compile error! s1 is invalid
    println!("{}", s2);               // ✅ Only s2 is valid
}
```

**Why this matters**:
- In C++, assignment copies by default (unless using move semantics explicitly)
- In Java/Python, assignment copies references (both point to same object)
- In Rust, assignment behavior depends on the type

### Ownership Transfer (Move)

```rust
fn ownership_transfer() {
    let s = String::from("hello");

    // Method 1: Assignment
    let s2 = s;  // s is now invalid

    // Method 2: Function call
    fn take_ownership(string: String) {
        println!("{}", string);
    }  // string is dropped here

    let s = String::from("world");
    take_ownership(s);  // s is now invalid
    // println!("{}", s);  // ❌ Error

    // Method 3: Return from function
    fn give_ownership() -> String {
        String::from("yours")
    }  // ownership transferred to caller

    let s3 = give_ownership();  // s3 is now owner
}
```

**Analogy from other languages**:
- C++: Like `std::move()` but automatic
- Java: Imagine if assignment invalidated the source variable
- Python: Like if `b = a` made `a` undefined

### Cloning (Deep Copy)

When you actually want a duplicate:

```rust
fn cloning() {
    let s1 = String::from("hello");
    let s2 = s1.clone();  // Explicit deep copy

    println!("{} {}", s1, s2);  // ✅ Both valid
}
```

**Rule of thumb**:
- Use clone when you need an independent copy
- Be aware: clone can be expensive (allocates and copies heap data)
- Prefer borrowing when you just need to read

### Types that Implement Copy

```rust
// These are Copy (assignment duplicates):
let a: i32 = 5;
let b: bool = true;
let c: char = 'x';
let d: f64 = 3.14;
let e: (i32, i32) = (1, 2);
let f: &str = "string slice";  // Reference itself is Copy
let g: [i32; 3] = [1, 2, 3];   // Fixed-size array of Copy types

// These are NOT Copy (assignment moves):
let s: String = String::from("hello");
let v: Vec<i32> = vec![1, 2, 3];
let b: Box<i32> = Box::new(5);

// Custom types
#[derive(Copy, Clone)]
struct Point {
    x: i32,
    y: i32,
}  // ✅ Can be Copy (all fields are Copy)

struct Person {
    name: String,  // String is not Copy
}  // ❌ Cannot be Copy
```

---

## Borrowing and References

Borrowing is Rust's way of temporarily accessing data without taking ownership.

### Immutable References (&T)

```rust
fn immutable_borrowing() {
    let s = String::from("hello");

    // Create a reference
    let r1 = &s;
    let r2 = &s;
    let r3 = &s;

    println!("{} {} {} {}", s, r1, r2, r3);  // ✅ All valid

    // Original owner still valid
    println!("{}", s);  // ✅
}

fn print_length(s: &String) {
    println!("Length: {}", s.len());
    // s is automatically "returned" (reference ends)
}

fn main() {
    let my_string = String::from("hello");
    print_length(&my_string);
    println!("{}", my_string);  // ✅ Still owns it
}
```

**Comparison**:
- C++: Like `const T&` references
- Java: Like read-only access (but enforced at compile time)
- Python: Like passing an object but cannot modify it

### Mutable References (&mut T)

```rust
fn mutable_borrowing() {
    let mut s = String::from("hello");

    // Create a mutable reference
    let r = &mut s;
    r.push_str(" world");
    println!("{}", r);

    // r goes out of scope here, so we can use s again
    println!("{}", s);  // ✅
}

fn add_exclamation(s: &mut String) {
    s.push_str("!");
}

fn main() {
    let mut msg = String::from("Hello");
    add_exclamation(&mut msg);
    println!("{}", msg);  // "Hello!"
}
```

**Comparison**:
- C++: Like `T&` non-const reference
- Java: Like having exclusive write access
- Python: Like modifying an object in place

### The Borrowing Rules

**Rule 1: Multiple immutable XOR one mutable**

At any given time, you can have **EITHER**:
- Any number of immutable references (`&T`)
- **OR** exactly one mutable reference (`&mut T`)

But **NOT BOTH** simultaneously.

```rust
fn borrowing_rules() {
    let mut s = String::from("hello");

    // ✅ Multiple immutable borrows OK
    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);

    // ✅ Mutable borrow OK (after immutable borrows end)
    let r3 = &mut s;
    r3.push_str(" world");
    println!("{}", r3);

    // ❌ WRONG: Multiple mutable borrows
    // let r4 = &mut s;
    // let r5 = &mut s;
    // println!("{} {}", r4, r5);  // Error!

    // ❌ WRONG: Immutable and mutable simultaneously
    // let r6 = &s;
    // let r7 = &mut s;
    // println!("{} {}", r6, r7);  // Error!
}
```

**Why?** This prevents data races at compile time:
- Multiple readers: Safe (no one is writing)
- One writer: Safe (no one else reading or writing)
- Multiple writers or readers+writers: Unsafe (data race)

### Reference Scope (Non-Lexical Lifetimes)

Since Rust 2018, reference lifetimes are determined by **last use**, not scope:

```rust
fn non_lexical_lifetimes() {
    let mut s = String::from("hello");

    let r1 = &s;
    let r2 = &s;
    println!("{} {}", r1, r2);
    // r1 and r2 are no longer used after this point

    let r3 = &mut s;  // ✅ OK! r1 and r2 lifetimes ended
    r3.push_str(" world");
    println!("{}", r3);
}
```

**Before Rust 2018**, this would have been an error because r1 and r2 were still "in scope."

### Dereferencing

```rust
fn dereferencing() {
    let x = 5;
    let y = &x;

    // Must dereference to get the value
    assert_eq!(5, *y);

    // Auto-dereferencing for method calls
    let s = String::from("hello");
    let r = &s;
    println!("{}", r.len());  // Auto-deref: (*r).len()

    // Mutable dereferencing
    let mut x = 5;
    let y = &mut x;
    *y += 1;
    println!("{}", x);  // 6
}
```

### Dangling References Prevention

Rust prevents dangling references at compile time:

```rust
// ❌ This won't compile
fn dangle() -> &String {
    let s = String::from("hello");
    &s  // Error: s is dropped when function ends
}

// ✅ Return ownership instead
fn no_dangle() -> String {
    let s = String::from("hello");
    s  // Ownership transferred to caller
}

// ✅ Or borrow from caller
fn borrow_from_caller(s: &String) -> &String {
    s  // OK: reference came from caller, will outlive function
}
```

---

## Scope and Drop

### Scope Rules

A variable's scope is the region where it's valid:

```rust
fn scope_example() {
    // s is not valid here

    {
        let s = String::from("hello");  // s is valid from this point
        println!("{}", s);
    }  // s goes out of scope, is dropped here

    // s is not valid here
}
```

### The Drop Trait

When a value goes out of scope, Rust calls its `drop` method:

```rust
struct CustomDrop {
    name: String,
}

impl Drop for CustomDrop {
    fn drop(&mut self) {
        println!("{} is being dropped!", self.name);
    }
}

fn main() {
    let x = CustomDrop { name: String::from("x") };
    let y = CustomDrop { name: String::from("y") };
    println!("End of main");
}
// Output:
// End of main
// y is being dropped!
// x is being dropped!
```

**Note**: Values are dropped in reverse order of creation (like C++ destructors).

### Early Drop

```rust
fn early_drop() {
    let s = String::from("hello");

    // Drop explicitly
    drop(s);  // s is dropped here

    // println!("{}", s);  // ❌ Error: s already dropped
}
```

### Ownership in Data Structures

```rust
struct Person {
    name: String,      // Person owns the String
    age: u32,
}

fn struct_ownership() {
    let person = Person {
        name: String::from("Alice"),
        age: 30,
    };

    // person owns name
    // When person is dropped, name is also dropped
}  // person and person.name both dropped here
```

---

## Lifetimes

Lifetimes are Rust's way of ensuring references are always valid. They're a form of generic that describes how long references live.

### The Problem Lifetimes Solve

```rust
// Without lifetimes, this is ambiguous:
// Does the returned reference point to x or y?
// How long is it valid?

fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
// ❌ Error: missing lifetime specifier
```

### Lifetime Syntax

Lifetimes are named with `'a`, `'b`, etc. (apostrophe + name):

```rust
// Read as: "For some lifetime 'a, this function takes two references
// that both live at least as long as 'a, and returns a reference
// that lives at least as long as 'a"

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

### Understanding Lifetime Annotations

**Important**: Lifetimes don't change how long references live. They describe the relationships between reference lifetimes.

```rust
fn lifetime_annotations() {
    let string1 = String::from("long string");
    let result;

    {
        let string2 = String::from("short");
        result = longest(&string1, &string2);
        println!("{}", result);  // ✅ OK here
    }

    // println!("{}", result);  // ❌ Error: string2 doesn't live long enough
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

**What `'a` means**:
- Both input references must be valid for at least the same lifetime `'a`
- The return reference will be valid for that lifetime `'a`
- The actual lifetime is the **smaller** of x and y's lifetimes

### Lifetime Elision Rules

In many cases, Rust can infer lifetimes automatically:

**Rule 1**: Each input reference gets its own lifetime:
```rust
fn foo(x: &i32, y: &i32)  // becomes:
fn foo<'a, 'b>(x: &'a i32, y: &'b i32)
```

**Rule 2**: If there's one input lifetime, it's assigned to all outputs:
```rust
fn foo(x: &i32) -> &i32  // becomes:
fn foo<'a>(x: &'a i32) -> &'a i32
```

**Rule 3**: If there's a `&self` or `&mut self`, its lifetime is assigned to all outputs:
```rust
impl MyStruct {
    fn get_name(&self) -> &str  // becomes:
    fn get_name<'a>(&'a self) -> &'a str
}
```

### Examples Where Lifetimes Are Needed

```rust
// ✅ One input: lifetime elided
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

// ❌ Multiple inputs, ambiguous output: need explicit lifetimes
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// ✅ Return is independent of input: no lifetime needed
fn make_string(s: &str) -> String {
    s.to_string()  // Returns owned String, not reference
}

// ✅ Return depends on only one input: can specify
fn first<'a>(x: &'a str, _y: &str) -> &'a str {
    x  // Return is tied to x, not y
}
```

### Lifetime in Structs

When structs hold references, you must specify lifetimes:

```rust
// This struct holds a reference
struct ImportantExcerpt<'a> {
    part: &'a str,
}

fn struct_lifetimes() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();

    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };

    println!("{}", excerpt.part);
}  // excerpt dropped, then novel

// This won't compile:
fn wrong_struct_lifetime() {
    let excerpt;
    {
        let novel = String::from("...");
        let first = novel.split('.').next().unwrap();
        excerpt = ImportantExcerpt { part: first };
    }  // ❌ novel dropped here

    // println!("{}", excerpt.part);  // Dangling reference!
}
```

**What `'a` means here**: The struct can't outlive the reference it holds.

### The 'static Lifetime

`'static` means the reference lives for the entire program:

```rust
// String literals have 'static lifetime
let s: &'static str = "I live forever";

// Not all strings are 'static
fn not_static() {
    let s = String::from("temporary");
    let r: &str = &s;  // This is NOT 'static
}  // s dropped here
```

Use cases for `'static`:
- String literals
- Global constants
- Leaked memory: `Box::leak()`

**Warning**: Don't use `'static` unless you really need it. It's a very strong constraint.

---

## Function Calling Patterns

Understanding how ownership and borrowing work with functions is crucial.

### Pattern 1: Taking Ownership

```rust
fn consume(s: String) {
    println!("{}", s);
}  // s is dropped here

fn main() {
    let s = String::from("hello");
    consume(s);
    // println!("{}", s);  // ❌ Error: s was moved
}
```

**When to use**:
- Function needs to own the data
- Data won't be used after the function call
- Building values (builder pattern)

### Pattern 2: Borrowing Immutably

```rust
fn read_only(s: &String) {
    println!("{}", s);
}  // s reference ends, original owner still valid

fn main() {
    let s = String::from("hello");
    read_only(&s);
    println!("{}", s);  // ✅ Still valid
}
```

**When to use**:
- Just need to read/inspect data
- Original owner needs to keep using it
- Most common pattern

### Pattern 3: Borrowing Mutably

```rust
fn modify(s: &mut String) {
    s.push_str(" world");
}

fn main() {
    let mut s = String::from("hello");
    modify(&mut s);
    println!("{}", s);  // "hello world"
}
```

**When to use**:
- Need to modify data
- Original owner needs to keep ownership
- Building/updating in place

### Pattern 4: Taking and Returning Ownership

```rust
fn process(s: String) -> String {
    format!("{} processed", s)
}

fn main() {
    let s = String::from("data");
    let s = process(s);  // Ownership moved and returned
    println!("{}", s);
}
```

**When to use**:
- Transform and return
- Chaining operations
- Avoiding intermediate clones

### Pattern 5: Multiple References

```rust
fn compare(s1: &String, s2: &String) -> bool {
    s1.len() > s2.len()
}

fn main() {
    let a = String::from("hello");
    let b = String::from("hi");

    if compare(&a, &b) {
        println!("a is longer");
    }

    // Both still valid
    println!("{} {}", a, b);
}
```

### Pattern 6: Mixing Owned and Borrowed

```rust
fn create_and_append(base: &str, suffix: String) -> String {
    format!("{}{}", base, suffix)
}

fn main() {
    let base = "Hello";  // &str (borrowed from binary)
    let suffix = String::from(" world");  // Owned

    let result = create_and_append(base, suffix);
    println!("{}", result);

    println!("{}", base);   // ✅ Still valid (was borrowed)
    // println!("{}", suffix);  // ❌ Error (was moved)
}
```

### Pattern 7: Generic Functions with Lifetimes

```rust
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

fn main() {
    let sentence = String::from("Hello world");
    let word = first_word(&sentence);

    println!("{}", word);     // "Hello"
    println!("{}", sentence); // "Hello world"
}
```

### Pattern 8: Multiple Lifetimes

```rust
// x and y can have different lifetimes
// Return is tied to x only
fn choose_first<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    println!("Ignoring: {}", y);
    x
}

fn main() {
    let string1 = String::from("long");
    let result;

    {
        let string2 = String::from("short");
        result = choose_first(&string1, &string2);
    }  // string2 dropped, but that's OK

    println!("{}", result);  // ✅ result only depends on string1
}
```

### Pattern 9: Returning References from Functions

```rust
// ❌ Can't return reference to local
fn wrong() -> &String {
    let s = String::from("hello");
    &s  // Error: s dropped at end of function
}

// ✅ Return owned value
fn correct1() -> String {
    String::from("hello")
}

// ✅ Return reference to input
fn correct2(s: &String) -> &String {
    s
}

// ✅ Return reference to static
fn correct3() -> &'static str {
    "hello"
}

// ✅ Return part of input
fn correct4(s: &str) -> &str {
    &s[0..5]
}
```

### Pattern 10: Method Calling

```rust
struct Data {
    value: String,
}

impl Data {
    // Takes ownership of self
    fn consume(self) {
        println!("{}", self.value);
    }

    // Borrows self immutably
    fn read(&self) {
        println!("{}", self.value);
    }

    // Borrows self mutably
    fn modify(&mut self) {
        self.value.push_str("!");
    }
}

fn main() {
    let mut data = Data { value: String::from("hello") };

    data.read();        // Borrow immutably
    data.modify();      // Borrow mutably
    data.read();        // Borrow immutably again
    data.consume();     // Takes ownership
    // data.read();     // ❌ Error: data was moved
}
```

---

## Advanced Lifetime Scenarios

### Lifetime Bounds

```rust
use std::fmt::Display;

// T must outlive 'a
fn print_ref<'a, T>(value: &'a T)
where
    T: Display + 'a,
{
    println!("{}", value);
}

// Equivalent shorter syntax
fn print_ref2<'a, T: Display + 'a>(value: &'a T) {
    println!("{}", value);
}
```

### Lifetime in Trait Implementations

```rust
trait Parser {
    fn parse<'a>(&self, input: &'a str) -> &'a str;
}

struct SimpleParser;

impl Parser for SimpleParser {
    fn parse<'a>(&self, input: &'a str) -> &'a str {
        input.trim()
    }
}
```

### Higher-Rank Trait Bounds (HRTB)

```rust
// A function that works for ANY lifetime
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let s = String::from("hello");
    let result = f(&s);
    println!("{}", result);
}

fn main() {
    call_with_ref(|s| s);
}
```

### Multiple Struct Lifetimes

```rust
struct Context<'a, 'b> {
    prefix: &'a str,
    suffix: &'b str,
}

impl<'a, 'b> Context<'a, 'b> {
    fn format(&self, middle: &str) -> String {
        format!("{}{}{}", self.prefix, middle, self.suffix)
    }
}

fn main() {
    let prefix = String::from("Hello ");
    let result;

    {
        let suffix = String::from("!");
        let ctx = Context {
            prefix: &prefix,
            suffix: &suffix,
        };
        result = ctx.format("world");
    }  // suffix dropped, but result is owned String, so OK

    println!("{}", result);
}
```

### Anonymous Lifetimes

```rust
// Rust 2018+: use '_ for anonymous lifetimes
struct Wrapper<'_> {
    data: &'_ str,
}

// Equivalent to:
// struct Wrapper<'a> {
//     data: &'a str,
// }
```

---

## Common Patterns and Idioms

### Builder Pattern with Ownership

```rust
struct Config {
    host: String,
    port: u16,
}

struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
}

impl ConfigBuilder {
    fn new() -> Self {
        ConfigBuilder {
            host: None,
            port: None,
        }
    }

    // Takes self, returns self (ownership transferred)
    fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn build(self) -> Config {
        Config {
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            port: self.port.unwrap_or(8080),
        }
    }
}

fn main() {
    let config = ConfigBuilder::new()
        .host("example.com".to_string())
        .port(3000)
        .build();
}
```

### Option and Result with Ownership

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Alice"))  // Transfers ownership
    } else {
        None
    }
}

fn main() {
    match find_user(1) {
        Some(name) => println!("{}", name),  // name owns the String
        None => println!("Not found"),
    }
}
```

### Iterators and Ownership

```rust
fn iterator_ownership() {
    let v = vec![1, 2, 3];

    // into_iter() - takes ownership, yields owned values
    for x in v {  // Equivalent to: v.into_iter()
        println!("{}", x);
    }
    // println!("{:?}", v);  // ❌ Error: v was moved

    let v = vec![1, 2, 3];

    // iter() - borrows, yields references
    for x in &v {  // Equivalent to: v.iter()
        println!("{}", x);
    }
    println!("{:?}", v);  // ✅ Still valid

    let mut v = vec![1, 2, 3];

    // iter_mut() - borrows mutably, yields mutable references
    for x in &mut v {  // Equivalent to: v.iter_mut()
        *x += 1;
    }
    println!("{:?}", v);  // [2, 3, 4]
}
```

### Interior Mutability Pattern

Sometimes you need to mutate through a shared reference:

```rust
use std::cell::RefCell;

struct Database {
    cache: RefCell<Vec<String>>,
}

impl Database {
    fn query(&self, id: u32) -> String {
        // &self is immutable, but we can mutate cache
        let mut cache = self.cache.borrow_mut();
        cache.push(format!("Query {}", id));
        format!("Result {}", id)
    }
}
```

**Warning**: This bypasses some compile-time checks, moving them to runtime.

### Cow (Clone on Write)

Useful when you might need to modify borrowed data:

```rust
use std::borrow::Cow;

fn process<'a>(input: &'a str, uppercase: bool) -> Cow<'a, str> {
    if uppercase {
        Cow::Owned(input.to_uppercase())  // Need to allocate
    } else {
        Cow::Borrowed(input)  // Can just borrow
    }
}

fn main() {
    let s = "hello";
    let result1 = process(s, false);  // Borrowed
    let result2 = process(s, true);   // Owned
}
```

---

## Comparison with Other Languages

### vs C++

| Rust | C++ | Notes |
|------|-----|-------|
| `String` | `std::string` | Move by default in Rust |
| `&T` | `const T&` | Immutable reference |
| `&mut T` | `T&` | Mutable reference |
| Move | `std::move()` | Automatic in Rust |
| `Box<T>` | `std::unique_ptr<T>` | Exclusive ownership |
| `Rc<T>` | `std::shared_ptr<T>` | Reference counting (single-threaded) |
| `Arc<T>` | `std::shared_ptr<T>` | Reference counting (thread-safe) |
| Lifetime `'a` | No equivalent | Compile-time reference validation |

**Key Difference**: Rust enforces ownership at compile time; C++ relies on programmer discipline.

### vs Java

| Rust | Java | Notes |
|------|-----|-------|
| `String` | `String` | Rust: heap-allocated, Java: immutable |
| `&String` | `String` (read-only use) | Rust: explicit borrow |
| Move semantics | N/A | Java copies references, not values |
| Drop | `finalize()` / try-with-resources | Rust: deterministic, Java: GC decides |
| Lifetime | N/A | Java: GC handles this |

**Key Difference**: Java uses garbage collection; Rust uses compile-time ownership tracking.

### vs Python

| Rust | Python | Notes |
|------|-----|-------|
| `String` | `str` | Rust: explicit ownership |
| `&str` | `str` | Rust distinguishes borrowed vs owned |
| Move | N/A | Python: everything is a reference |
| Clone | `copy.deepcopy()` | Rust: explicit, Python: explicit |
| Drop | `__del__` | Rust: deterministic, Python: GC decides |

**Key Difference**: Python uses reference counting + GC; Rust uses ownership system.

### vs Go

| Rust | Go | Notes |
|------|-----|-------|
| `String` | `string` | Go: immutable |
| `&T` | `*T` | Rust: compile-time safety |
| Move | N/A | Go: copies pointers |
| Drop | `defer` / GC | Rust: automatic, Go: manual or GC |
| Lifetime | N/A | Go: escape analysis + GC |

**Key Difference**: Go uses garbage collection with escape analysis; Rust uses ownership.

---

## Practical Guidelines

### When to Use Each Pattern

**Use ownership (T)** when:
- Function is the final consumer
- Building/transforming values
- Implementing builder patterns
- Examples: `fn process(data: String) -> ProcessedData`

**Use immutable borrow (&T)** when:
- Just reading data
- Function doesn't need ownership
- Most common case
- Examples: `fn print(data: &str)`, `fn validate(&self) -> bool`

**Use mutable borrow (&mut T)** when:
- Modifying existing data in place
- Avoiding unnecessary allocations
- Examples: `fn sort(&mut self)`, `fn update(&mut self, value: i32)`

**Use clone()** when:
- Need independent copies
- Ownership rules make borrowing difficult
- Performance impact is acceptable
- Examples: Thread spawning, caching

### Common Mistakes

**Mistake 1: Fighting the borrow checker**

```rust
// ❌ Fighting
fn bad_example(v: &mut Vec<i32>) {
    for i in 0..v.len() {
        v.push(v[i]);  // Error: can't borrow as mutable while iterating
    }
}

// ✅ Working with it
fn good_example(v: &mut Vec<i32>) {
    let len = v.len();
    for i in 0..len {
        v.push(v[i]);
    }
}
```

**Mistake 2: Unnecessary cloning**

```rust
// ❌ Cloning unnecessarily
fn process_items(items: Vec<String>) {
    for item in items.clone() {  // Unnecessary clone!
        println!("{}", item);
    }
}

// ✅ Use reference
fn process_items(items: &[String]) {
    for item in items {
        println!("{}", item);
    }
}
```

**Mistake 3: Lifetime over-specification**

```rust
// ❌ Too specific
fn first_word<'a, 'b>(s: &'a str, _other: &'b str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

// ✅ Simpler (elision works)
fn first_word(s: &str, _other: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

### Performance Tips

1. **Prefer borrowing over cloning** - Avoid heap allocations
2. **Use string slices (&str) over String** - More flexible
3. **Pass large structs by reference** - Avoid copies
4. **Use `Vec::with_capacity()`** - Pre-allocate when size is known
5. **Return ownership from builders** - Enable method chaining

### Debugging Ownership Issues

**Error: "value borrowed here after move"**
- Solution: Use reference instead of moving
- Or: Clone if you need multiple owners
- Or: Restructure to avoid the move

**Error: "cannot borrow as mutable more than once"**
- Solution: Drop earlier mutable borrows before creating new ones
- Or: Use scope to limit borrow lifetimes
- Or: Split data structure to borrow different parts

**Error: "lifetime may not live long enough"**
- Solution: Ensure reference outlives its use
- Or: Return owned data instead of reference
- Or: Add explicit lifetime annotations

### Best Practices

1. **Start with references** - Use ownership only when needed
2. **Make functions accept &str, not &String** - More flexible
3. **Return owned values** - Let caller decide how to use them
4. **Use meaningful lifetime names** - `'input`, `'output` instead of `'a`, `'b`
5. **Leverage elision** - Don't add lifetimes unless required
6. **Design for ownership** - Think about who owns what
7. **Document ownership transfer** - Make it clear in comments

---

## Summary

### Core Concepts

1. **Ownership**: Each value has exactly one owner
2. **Move**: Ownership transfer invalidates the source
3. **Borrow**: Temporary access without ownership
4. **Lifetimes**: Compile-time validation of reference validity
5. **Drop**: Automatic cleanup when owner goes out of scope

### The Rules

1. Each value has one owner
2. When owner goes out of scope, value is dropped
3. Either multiple `&T` or one `&mut T`, not both
4. References must always be valid

### Mental Model

Think of Rust ownership like:
- **Ownership**: Who is responsible for cleanup?
- **Borrowing**: Who can use this temporarily?
- **Lifetimes**: How long is this valid?

### The Payoff

In exchange for learning these rules, you get:
- Memory safety without garbage collection
- Thread safety without data races
- Zero-cost abstractions
- Fearless concurrency

The borrow checker is strict, but it catches bugs at compile time that would be runtime crashes in other languages. Once your code compiles, you can be confident it's memory-safe.
