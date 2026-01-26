# Appendix B: Anti-Patterns
**Common Pitfalls:**

- [Excessive Cloning](#anti-pattern-excessive-cloning)
- [Overusing Rc/Arc Without Need](#anti-pattern-overusing-rcarc-without-need)
- [Ignoring Iterator Combinators](#anti-pattern-ignoring-iterator-combinators)
- [Deref Coercion Abuse](#anti-pattern-deref-coercion-abuse)
- [String vs &str Confusion](#anti-pattern-string-vs-str-confusion)

**Performance Anti-Patterns:**

- [Collecting Iterators Unnecessarily](#anti-pattern-collecting-iterators-unnecessarily)
- [Vec<T> When [] Suffices](#anti-pattern-vect-when-t-n-suffices)
- [HashMap for Small Key Sets](#anti-pattern-hashmap-for-small-key-sets)
- [Premature String Allocation](#anti-pattern-premature-string-allocation)
- [Boxed Trait Objects Everywhere](#anti-pattern-boxed-trait-objects-everywhere)

**Safety Anti-Patterns:**

- [Unsafe for Convenience](#anti-pattern-unsafe-for-convenience)
- [Unwrap() in Production Code](#anti-pattern-unwrap-in-production-code)
- [RefCell/Mutex Without Consideration](#anti-pattern-refcellmutex-without-consideration)
- [Ignoring Send/Sync Implications](#anti-pattern-ignoring-sendsync-implications)

**API Design Mistakes:**

- [Stringly-Typed APIs](#anti-pattern-stringly-typed-apis)
- [Boolean Parameter Trap](#anti-pattern-boolean-parameter-trap)
- [Leaky Abstractions](#anti-pattern-leaky-abstractions)
- [Returning Owned When Borrowed Suffices](#anti-pattern-returning-owned-when-borrowed-suffices)
- [Overengineered Generic APIs](#anti-pattern-overengineered-generic-apis)

### Overview

Anti-patterns are common solutions to recurring problems that initially seem reasonable but ultimately create more issues than they solve. Unlike design patterns, which represent best practices, anti-patterns represent pitfalls—seductive shortcuts that lead to bugs, performance degradation, or unmaintainable code.

In Rust, anti-patterns often emerge when developers apply patterns from other languages without adapting to Rust's ownership model, or when they fight the compiler rather than understanding what it's trying to prevent. The borrow checker is not your enemy—it's preventing real-time bugs. When code feels like a battle against the type system, you're usually approaching the problem incorrectly.

This catalog identifies common anti-patterns in four categories:

**Common pitfalls** are mistakes developers make when learning Rust, often from misunderstanding ownership, borrowing, or lifetime rules. These lead to unnecessary clones, reference counting where simple borrowing would suffice, or awkward code structure.

**Performance anti-patterns** sacrifice efficiency for convenience without understanding the cost. These include excessive allocations, missed optimization opportunities, and patterns that prevent the compiler from generating optimal code.

**Safety anti-patterns** undermine Rust's guarantees, often through misuse of `unsafe`, inappropriate uses of interior mutability, or patterns that make code fragile and error-prone.

**API design mistakes** create poor interfaces—difficult to use correctly, easy to misuse, or inconsistent with Rust ecosystem conventions. These frustrate users and lead to adoption problems.

Each anti-pattern includes:
- **The pattern**: What it looks like in code
- **Why it's problematic**: The issues it causes
- **The solution**: The correct approach
- **Real-world impact**: Consequences in production code

Learning to recognize and avoid these anti-patterns is as important as knowing design patterns. They represent the accumulated wisdom of the Rust community—lessons learned through bugs, performance issues, and maintenance headaches.

---

## Common Pitfalls

These anti-patterns emerge from misunderstanding Rust's fundamental concepts. They're especially common among developers transitioning from garbage-collected languages or C++.

### Anti-Pattern: Excessive Cloning

**The Pattern**: Cloning data unnecessarily to satisfy the borrow checker rather than understanding borrowing rules.

```rust
//  ANTI-PATTERN: clone() to avoid borrow checker - 3 clones = 3x memory
fn process_data(data: Vec<String>) {
    let copy1 = data.clone();  // Unnecessary
    print_data(copy1);

    let copy2 = data.clone();  // Unnecessary
    transform_data(copy2);

    let copy3 = data.clone();  // Unnecessary
    save_data(copy3);
}

fn print_data(data: Vec<String>) { for item in &data { println!("{}", item); } }
fn transform_data(data: Vec<String>) -> Vec<String> { data.iter().map(|s| s.to_uppercase()).collect() }
fn save_data(data: Vec<String>) { /* Save to database */ }
```

**Why It's Problematic**:
- Performance degradation: Each clone allocates and copies all data
- Memory waste: Multiple copies of the same data
- Indicates misunderstanding of ownership and borrowing
- In large datasets, this can cause significant slowdowns

**The Solution**: Use references for read-only access, mutable references for modifications, or move ownership when appropriate.

```rust
// CORRECT: Use borrowing - &data for reads, move when done
fn process_data(data: Vec<String>) {
    // Borrow for read-only access
    print_data(&data);

    // Clone only when you need to modify and keep original
    let transformed = transform_data(&data);

    // Move ownership when done with original
    save_data(data);
}

fn print_data(data: &[String]) { for item in data { println!("{}", item); } }
fn transform_data(data: &[String]) -> Vec<String> { data.iter().map(|s| s.to_uppercase()).collect() }
fn save_data(data: Vec<String>) { /* Takes ownership */ }
```

**Real-World Impact**: A web service cloning request data at every processing step saw 3x memory usage and 40% latency increase. Switching to borrowing reduced memory by 66% and improved response times significantly.

---

### Anti-Pattern: Overusing Rc/Arc Without Need

**The Pattern**: Using reference counting for shared ownership when simple borrowing or restructuring would work.

```rust
use std::rc::Rc;
//  ANTI-PATTERN: Rc adds heap + refcount overhead for single-owner data
struct DataProcessor {
    config: Rc<Config>,
    logger: Rc<Logger>,
    cache: Rc<Cache>,
}

impl DataProcessor {
    fn new(config: Config, logger: Logger, cache: Cache) -> Self {
        Self {
            config: Rc::new(config),
            logger: Rc::new(logger),
            cache: Rc::new(cache),
        }
    }

    fn process(&self, data: &str) { self.logger.log("Processing..."); }  // Rc overhead for no reason
}
```

**Why It's Problematic**:
- Runtime overhead: Reference counting adds CPU cost
- Heap allocation: Rc requires heap allocation
- Complexity: Rc<RefCell<T>> for mutation is unnecessarily complex
- Hides ownership structure: Makes data flow unclear
- Thread safety issues: Rc isn't Send/Sync

**The Solution**: Use references with explicit lifetimes, restructure ownership, or only use Rc/Arc when genuinely needed for shared ownership.

```rust
// CORRECT: Use &'a references for borrowed data
struct DataProcessor<'a> {
    config: &'a Config,
    logger: &'a Logger,
    cache: &'a Cache,
}

impl<'a> DataProcessor<'a> {
    fn new(config: &'a Config, logger: &'a Logger, cache: &'a Cache) -> Self { Self { config, logger, cache } }
    fn process(&self, data: &str) { self.logger.log("Processing..."); }
}

// Or restructure to owned data when sharing isn't needed
struct DataProcessor {
    config: Config,  // Small configs can be copied/cloned
}

// Only use Rc/Arc when truly sharing across owners
use std::sync::Arc;
use std::thread;

fn multiple_threads_need_shared_data() {
    let config = Arc::new(Config::load());

    let config1 = Arc::clone(&config);
    let handle1 = thread::spawn(move || process_with_config(&config1));

    let config2 = Arc::clone(&config);
    let handle2 = thread::spawn(move || process_with_config(&config2));

    // Genuine shared ownership across threads
}
```

**Real-World Impact**: A server using Rc<T> throughout for "convenience" had 15% CPU overhead from reference counting. Refactoring to use references where possible eliminated the overhead.

---

### Anti-Pattern: Ignoring Iterator Combinators

**The Pattern**: Using manual loops and mutable accumulators instead of iterator methods.

```rust
//  ANTI-PATTERN: Manual loops instead of iterators - harder to read/parallelize
fn process_numbers(numbers: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    for &num in numbers {
        if num % 2 == 0 {
            result.push(num * 2);
        }
    }
    result
}

fn find_first_large(numbers: &[i32]) -> Option<i32> {
    for &num in numbers {
        if num > 100 {
            return Some(num);
        }
    }
    None
}

fn sum_squares(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for &num in numbers {
        sum += num * num;
    }
    sum
}
```

**Why It's Problematic**:
- More verbose and harder to read
- Mutable state management is error-prone
- Misses optimization opportunities (compiler can optimize iterator chains better)
- Doesn't compose well
- Harder to parallelize (rayon works with iterators)

**The Solution**: Use iterator combinators for declarative, composable data processing.

```rust
// CORRECT: .filter().map().sum() - declarative, easy to parallelize
fn process_numbers(numbers: &[i32]) -> Vec<i32> {
    numbers.iter()
        .filter(|&&num| num % 2 == 0)
        .map(|&num| num * 2)
        .collect()
}

fn find_first_large(numbers: &[i32]) -> Option<i32> { numbers.iter().find(|&&num| num > 100).copied() }
fn sum_squares(numbers: &[i32]) -> i32 { numbers.iter().map(|&num| num * num).sum() }

use rayon::prelude::*;
fn parallel_sum_squares(numbers: &[i32]) -> i32 {
    numbers.par_iter().map(|&num| num * num).sum()  // Just iter() -> par_iter()
}
```

**Real-World Impact**: Iterator chains often compile to the same or better assembly than manual loops, while being more maintainable. A data processing pipeline converted to iterators saw identical performance but 40% less code.

---

### Anti-Pattern: Deref Coercion Abuse

**The Pattern**: Relying on Deref coercion for API design, making code implicit and confusing.

```rust
use std::ops::Deref;
//  ANTI-PATTERN: Abusing Deref for inheritance - use explicit delegation
struct Employee {
    name: String,
    id: u32,
}

struct Manager {
    employee: Employee,
    team_size: usize,
}

impl Deref for Manager {
    type Target = Employee;
    fn deref(&self) -> &Self::Target { &self.employee }
}

fn print_employee_info(emp: &Employee) { println!("{}: {}", emp.id, emp.name); }

let manager = Manager {
    employee: Employee { name: "Alice".to_string(), id: 1 },
    team_size: 5,
};

// Works due to Deref, but confusing
print_employee_info(&manager);
```

**Why It's Problematic**:
- Violates principle of least surprise: implicit conversions hide intent
- Not true inheritance: doesn't work with trait objects
- Maintenance issues: changing Deref breaks code in non-obvious ways
- Against Rust guidelines: Deref is for smart pointers, not emulating inheritance

**The Solution**: Use explicit methods or trait implementations for delegation.

```rust
// CORRECT: manager.employee() or manager.name() makes relationship explicit
struct Manager {
    employee: Employee,
    team_size: usize,
}

impl Manager {
    fn employee(&self) -> &Employee { &self.employee }
    fn name(&self) -> &str { &self.employee.name }  // Explicit delegation
    fn id(&self) -> u32 { self.employee.id }
}

// Explicit conversion
fn print_employee_info(emp: &Employee) {
    println!("{}: {}", emp.id, emp.name);
}

let manager = Manager {
    employee: Employee { name: "Alice".to_string(), id: 1 },
    team_size: 5,
};

// Clear and explicit
print_employee_info(manager.employee());
```

**Real-World Impact**: A library using Deref for pseudo-inheritance confused users and broke when internal structure changed. Switching to explicit methods improved API clarity.

---

### Anti-Pattern: String vs &str Confusion

**The Pattern**: Unnecessary string allocations and confusing conversions between String and &str.

```rust
//  ANTI-PATTERN: fn greet(name: String) forces callers to allocate
fn greet(name: String) -> String { format!("Hello, {}", name) }

fn process_names(names: Vec<&str>) {
    for name in names {
        let owned = name.to_string();  // Unnecessary allocation
        greet(owned);
    }
}

// Forces callers to own Strings
let name = "Alice".to_string();  // Allocation just to call function
greet(name);
```

**Why It's Problematic**:
- Forces unnecessary allocations on callers
- Less flexible API (can't use string literals without converting)
- Performance cost from heap allocations
- Doesn't follow Rust idioms

**The Solution**: Accept &str in function parameters, return String when ownership is transferred.

```rust
// CORRECT: Accept &str - greet("Alice") or greet(&owned) both work
fn greet(name: &str) -> String { format!("Hello, {}", name) }

fn process_names(names: &[&str]) {
    for &name in names {
        let greeting = greet(name);  // No unnecessary allocation
        println!("{}", greeting);
    }
}

// Flexible: works with literals and owned strings
greet("Alice");              // No allocation needed
let owned = String::from("Bob");
greet(&owned);               // Also works

fn greet_generic<S: AsRef<str>>(name: S) -> String { format!("Hello, {}", name.as_ref()) }  // Works with &str, String, Cow

// Works with &str, String, Cow<str>, etc.
greet_generic("Alice");
greet_generic(String::from("Bob"));
```

**Real-World Impact**: A web service accepting `String` parameters forced clients to allocate for every request. Changing to `&str` reduced allocations by 80% in typical workloads.

---

## Performance Anti-Patterns

These patterns sacrifice performance without good reason, often from not understanding the cost model or missing optimization opportunities.

### Anti-Pattern: Collecting Iterators Unnecessarily

**The Pattern**: Calling `.collect()` in the middle of iterator chains when the final consumer can work with iterators.

```rust
//  ANTI-PATTERN: .collect() between steps allocates intermediate Vecs
fn process_data(numbers: &[i32]) -> i32 {
    let evens: Vec<i32> = numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .copied()
        .collect();  // Unnecessary allocation

    let doubled: Vec<i32> = evens.iter()
        .map(|&x| x * 2)
        .collect();  // Another unnecessary allocation

    doubled.iter().sum()
}
```

**Why It's Problematic**:
- Multiple heap allocations
- Cache unfriendly: data copied multiple times
- Breaks iterator fusion: compiler can't optimize across collect boundaries
- Memory overhead from intermediate vectors

**The Solution**: Chain iterators without intermediate collections.

```rust
// CORRECT: .filter().map().sum() in one chain; zero allocations
fn process_data(numbers: &[i32]) -> i32 {
    numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .sum()
    // Zero allocations, single pass through data
}

// Only collect when you need to reuse the result
fn process_data_reusable(numbers: &[i32]) -> Vec<i32> {
    numbers.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .collect()  // Now justified: we return the collection
}
```

**Benchmarking shows**: Single iterator chain can be 10-100x faster than multiple collect calls for large datasets.

---

### Anti-Pattern: Vec<T> When rray Suffices

**The Pattern**: Using heap-allocated `Vec<T>` for fixed-size, small collections when stack-allocated arrays would work.

```rust
//  ANTI-PATTERN: vec![r, g, b] allocates heap for 3 bytes; [u8; 3] is stack
fn get_rgb_channels(pixel: u32) -> Vec<u8> {
    vec![
        ((pixel >> 16) & 0xFF) as u8,
        ((pixel >> 8) & 0xFF) as u8,
        (pixel & 0xFF) as u8,
    ]
}

fn multiply_3x3(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    // Matrix multiplication with heap-allocated matrices
    // Multiple allocations for 9 numbers!
    unimplemented!()
}
```

**Why It's Problematic**:
- Heap allocation overhead for small data
- Pointer indirection reduces cache locality
- Runtime size checks instead of compile-time guarantees
- Can't take advantage of SIMD optimizations

**The Solution**: Use arrays for fixed-size data.

```rust
// CORRECT: [u8; 3] stack-allocated; [[f64; 3]; 3] for matrix
fn get_rgb_channels(pixel: u32) -> [u8; 3] {
    [
        ((pixel >> 16) & 0xFF) as u8,
        ((pixel >> 8) & 0xFF) as u8,
        (pixel & 0xFF) as u8,
    ]
}

fn multiply_3x3(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut result = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    result
    // All on stack, no allocations
}

// Use Vec only when size is truly dynamic
fn get_pixels(count: usize) -> Vec<[u8; 3]> {
    vec![[0, 0, 0]; count]
}
```

**Real-World Impact**: Graphics code using `Vec<u8>` for RGB triplets spent 40% of time in allocator. Switching to `[u8; 3]` eliminated allocations and doubled throughput.

---

### Anti-Pattern: HashMap for Small Key Sets

**The Pattern**: Using `HashMap` for collections with few items when linear search would be faster.

```rust
use std::collections::HashMap;
//  ANTI-PATTERN: HashMap for 3 items - match is 10x faster for small sets
fn get_status_code(status: &str) -> u16 {
    let mut codes = HashMap::new();
    codes.insert("ok", 200);
    codes.insert("not_found", 404);
    codes.insert("error", 500);

    *codes.get(status).unwrap_or(&500)
}

// Recreating HashMap on every call!
```

**Why It's Problematic**:
- Hash computation overhead for small collections
- HashMap allocation and initialization cost
- Linear search is faster for <10 items
- Poor cache locality from hashing

**The Solution**: Use arrays or match statements for small, known key sets.

```rust
// CORRECT: match compiles to jump table for small known sets
fn get_status_code(status: &str) -> u16 {
    match status {
        "ok" => 200,
        "not_found" => 404,
        "error" => 500,
        _ => 500,
    }
    // Compiles to jump table or if-chain, no allocation
}

// Or array of tuples for linear search
fn get_status_code_array(status: &str) -> u16 {
    const CODES: &[(&str, u16)] = &[
        ("ok", 200),
        ("not_found", 404),
        ("error", 500),
    ];

    CODES.iter()
        .find(|(key, _)| *key == status)
        .map(|(_, code)| *code)
        .unwrap_or(500)
}

// Use HashMap only for larger, dynamic collections
use std::collections::HashMap;
use std::sync::LazyLock;

static LARGE_CODES: LazyLock<HashMap<&'static str, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    // Populate with many items
    for i in 0..1000 {
        // ...
    }
    map
});
```

**Benchmarking**: For 3-5 items, match is 10x faster than HashMap. HashMap becomes faster around 10-20 items.

---

### Anti-Pattern: Premature String Allocation

**The Pattern**: Converting to String early when working with string data, causing unnecessary allocations.

```rust
//  ANTI-PATTERN: line.to_string() before checking - 90% may be filtered
fn process_log_line(line: &str) -> Option<String> {
    let owned = line.to_string();  // Allocate immediately

    if !owned.starts_with("ERROR") {
        return None;  // Wasted allocation
    }

    Some(owned.to_uppercase())  // Another allocation
}

fn extract_field(data: &str, field: &str) -> String {
    let owned = data.to_string();  // Unnecessary
    owned.split(',')
        .find(|s| s.starts_with(field))
        .unwrap_or("")
        .to_string()  // Only this allocation needed
}
```

**Why It's Problematic**:
- Allocates even when not needed (early returns)
- Multiple allocations when one suffices
- Doesn't leverage string slice efficiency

**The Solution**: Work with &str as long as possible, allocate only when necessary.

```rust
// CORRECT: Check first, allocate only if needed
fn process_log_line(line: &str) -> Option<String> {
    if !line.starts_with("ERROR") {
        return None;  // No allocation for filtered lines
    }

    Some(line.to_uppercase())  // Single allocation only when needed
}

fn extract_field<'a>(data: &'a str, field: &str) -> &'a str {
    data.split(',').find(|s| s.starts_with(field)).unwrap_or("")  // No allocations
}

// If ownership needed, use Cow for conditional allocation
use std::borrow::Cow;

fn normalize<'a>(s: &'a str) -> Cow<'a, str> {
    if s.chars().any(|c| c.is_uppercase()) {
        Cow::Owned(s.to_lowercase())  // Allocate only if needed
    } else {
        Cow::Borrowed(s)  // Zero-cost
    }
}
```

**Real-World Impact**: Log processing service allocating Strings for every line processed 10M strings/sec. 90% were filtered. Using &str until needed reduced allocations by 90% and improved throughput 3x.

---

### Anti-Pattern: Boxed Trait Objects Everywhere

**The Pattern**: Using `Box<dyn Trait>` when static dispatch with generics would work.

```rust
//  ANTI-PATTERN: Box<dyn Trait> when types known at compile time; 2-10x slower
trait Processor {
    fn process(&self, data: &str) -> String;
}

fn pipeline(processors: Vec<Box<dyn Processor>>, data: &str) -> String {
    let mut result = data.to_string();
    for processor in processors {
        result = processor.process(&result);  // Virtual call overhead
    }
    result
}
```

**Why It's Problematic**:
- Heap allocation for each trait object
- Virtual dispatch prevents inlining
- No specialization possible
- Dynamic dispatch has 2-10x overhead vs static

**The Solution**: Use generics for static dispatch when types are known at compile-time.

```rust
// CORRECT: Generics for static dispatch - inlined, no heap
fn pipeline<P1, P2, P3>(p1: P1, p2: P2, p3: P3, data: &str) -> String
where
    P1: Processor,
    P2: Processor,
    P3: Processor,
{
    let result = p1.process(data);
    let result = p2.process(&result);
    p3.process(&result)
    // All calls inlined, no heap allocations
}

// Or use impl Trait for flexibility
fn process_twice(data: &str, processor: impl Processor) -> String {
    let once = processor.process(data);
    processor.process(&once)
}

// Only use dyn when you truly need runtime polymorphism
fn dynamic_pipeline(processors: Vec<Box<dyn Processor>>, data: &str) -> String {
    // Justified: processors unknown at compile time
    let mut result = data.to_string();
    for processor in processors {
        result = processor.process(&result);
    }
    result
}
```

**Benchmarking**: Static dispatch via generics can be 5-10x faster than dyn trait objects for simple operations due to inlining and specialization.

---

## Safety Anti-Patterns

These patterns undermine Rust's safety guarantees, creating opportunities for bugs, undefined behavior, or security vulnerabilities.

### Anti-Pattern: Unsafe for Convenience

**The Pattern**: Using `unsafe` to bypass borrow checker restrictions without genuine need or proper justification.

```rust
//  ANTI-PATTERN: unsafe to bypass borrow checker; if i==j, aliased &mut = UB
struct Cache {
    data: Vec<String>,
}

impl Cache {
    fn get_mut_two(&mut self, i: usize, j: usize) -> (&mut String, &mut String) {
        // "I know what I'm doing" famous last words
        unsafe {
            let ptr = self.data.as_mut_ptr();
            (&mut *ptr.add(i), &mut *ptr.add(j))
        }
        // What if i == j? Undefined behavior!
    }
}
```

**Why It's Problematic**:
- Creates undefined behavior (aliased mutable references)
- No bounds checking (can access out of bounds)
- Defeats Rust's safety guarantees
- Hard to audit and maintain
- Usually indicates misunderstanding of safe alternatives

**The Solution**: Use safe alternatives or properly validate unsafe code.

```rust
// CORRECT: Use split_at_mut() for disjoint mutable borrows
impl Cache {
    fn get_mut_two(&mut self, i: usize, j: usize) -> Option<(&mut String, &mut String)> {
        if i == j {
            return None;  // Can't return two mutable refs to same element
        }

        // Safe split_at_mut
        if i < j {
            let (left, right) = self.data.split_at_mut(j);
            Some((&mut left[i], &mut right[0]))
        } else {
            let (left, right) = self.data.split_at_mut(i);
            Some((&mut right[0], &mut left[j]))
        }
    }
}

// Or use indexing if you're sure indices are valid
impl Cache {
    fn get_mut_two_unchecked(&mut self, i: usize, j: usize) -> (&mut String, &mut String) {
        assert!(i != j);
        assert!(i < self.data.len());
        assert!(j < self.data.len());

        // Still use safe split_at_mut
        if i < j {
            let (left, right) = self.data.split_at_mut(j);
            (&mut left[i], &mut right[0])
        } else {
            let (left, right) = self.data.split_at_mut(i);
            (&mut right[0], &mut left[j])
        }
    }
}
```

**Real-World Impact**: A library using `unsafe` for "convenience" had a bug where indices could be equal, causing memory corruption. The safe solution using `split_at_mut` would have prevented this.

---

### Anti-Pattern: Unwrap() in Production Code

**The Pattern**: Using `.unwrap()` or `.expect()` liberally without proper error handling.

```rust
//  ANTI-PATTERN: .unwrap() crashes on errors; use Result/? for production
fn load_config(path: &str) -> Config {
    let contents = std::fs::read_to_string(path).unwrap();  // Panics if file missing
    let config: Config = serde_json::from_str(&contents).unwrap();  // Panics if invalid JSON
    config
}

fn get_user_age(users: &HashMap<String, User>, id: &str) -> u32 {
    users.get(id).unwrap().age  // Panics if user not found
}

fn divide(a: i32, b: i32) -> i32 {
    a.checked_div(b).unwrap()  // Panics on division by zero
}
```

**Why It's Problematic**:
- Crashes on unexpected input
- Poor user experience (panic messages are cryptic)
- No recovery opportunity
- Makes code fragile
- Acceptable in examples, not production

**The Solution**: Handle errors properly with Result/Option or document why unwrap is safe.

```rust
// CORRECT: Return Result<T, E> and use ?; caller handles errors
use std::io;
use serde_json;

#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    Parse(serde_json::Error),
}

impl From<io::Error> for ConfigError { fn from(err: io::Error) -> Self { ConfigError::Io(err) } }
impl From<serde_json::Error> for ConfigError { fn from(err: serde_json::Error) -> Self { ConfigError::Parse(err) } }

fn load_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}

fn get_user_age(users: &HashMap<String, User>, id: &str) -> Option<u32> { users.get(id).map(|user| user.age) }
fn divide(a: i32, b: i32) -> Option<i32> { a.checked_div(b) }

// If unwrap is genuinely safe, document why
fn get_first_line(text: &str) -> &str {
    text.lines()
        .next()
        .unwrap()  // Safe: lines() always yields at least one line (possibly empty)
}
```

**Real-World Impact**: A web service using unwrap() crashed on malformed requests. Proper error handling returned 400 errors instead of crashing.

---

### Anti-Pattern: RefCell/Mutex Without Consideration

**The Pattern**: Using interior mutability as a first resort rather than last resort.

```rust
use std::cell::RefCell;
//  ANTI-PATTERN: RefCell everywhere adds runtime borrow checks; use &mut self
struct Application {
    state: RefCell<AppState>,
    config: RefCell<Config>,
    users: RefCell<Vec<User>>,
}

impl Application {
    fn process(&self) {
        let mut state = self.state.borrow_mut();
        let config = self.config.borrow();
        let mut users = self.users.borrow_mut();

        // Runtime borrow checking overhead
        // Can panic if borrowing rules violated
        // Hidden mutation through shared reference
    }
}
```

**Why It's Problematic**:
- Runtime cost of borrow checking
- Can panic if borrowing rules violated
- Hides mutability in API (& self but mutates)
- Indicates fighting the type system
- Makes code harder to reason about

**The Solution**: Design with proper ownership and mutability; use RefCell only when genuinely needed.

```rust
// CORRECT: fn process(&mut self) is honest about mutation; compile-time checked
struct Application {
    state: AppState,
    config: Config,
    users: Vec<User>,
}

impl Application {
    fn process(&mut self) { /* Compile-time checked */ }  // Honest about mutation
    fn read_config(&self) -> &Config { &self.config }  // No runtime overhead
}

// Use RefCell only when necessary (e.g., graph structures, caching)
use std::cell::RefCell;

struct Node {
    value: i32,
    children: RefCell<Vec<Node>>,  // Justified: allows mutation during traversal
}

// For shared ownership with mutation, use Arc<Mutex<T>>
use std::sync::{Arc, Mutex};

fn concurrent_modification() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));

    let data_clone = Arc::clone(&data);
    std::thread::spawn(move || {
        data_clone.lock().unwrap().push(4);
    });
}
```

**Real-World Impact**: Code using RefCell throughout had runtime panics from double borrows and 10% overhead from runtime checks. Restructuring with proper &mut eliminated both issues.

---

### Anti-Pattern: Ignoring Send/Sync Implications

**The Pattern**: Using thread-unsafe types across threads without understanding Send/Sync bounds.

```rust
use std::rc::Rc;
use std::cell::RefCell;
use std::thread;
//  ANTI-PATTERN: Rc/RefCell aren't Send; unsafe sharing causes data races
fn share_across_threads() {
    let data = Rc::new(RefCell::new(vec![1, 2, 3]));
    let data_clone = Rc::clone(&data);

    // Compile error: Rc and RefCell aren't Send/Sync
    // thread::spawn(move || {
    //     data_clone.borrow_mut().push(4);
    // });

    // "Fix" with unsafe (WRONG!)
    let ptr = Rc::into_raw(data_clone);
    thread::spawn(move || {
        unsafe {
            let rc = Rc::from_raw(ptr);
            rc.borrow_mut().push(4);
            // Data race! Undefined behavior!
        }
    });
}
```

**Why It's Problematic**:
- Data races cause undefined behavior
- Rc uses non-atomic reference counting (race condition)
- RefCell uses non-atomic borrow tracking (race condition)
- Circumventing Send/Sync with unsafe defeats safety

**The Solution**: Use thread-safe alternatives (Arc, Mutex) or restructure to avoid sharing.

```rust
use std::sync::{Arc, Mutex};
use std::thread;
// CORRECT: Arc<Mutex<T>> for shared mutable state; or channels
fn share_across_threads_safely() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);

    let handle = thread::spawn(move || {
        data_clone.lock().unwrap().push(4);
    });

    handle.join().unwrap();

    let final_data = data.lock().unwrap();
    println!("{:?}", *final_data);
}

// Or use message passing (often better)
use std::sync::mpsc;

fn message_passing() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        tx.send(vec![1, 2, 3, 4]).unwrap();
    });

    let data = rx.recv().unwrap();
    println!("{:?}", data);
}
```

**Real-World Impact**: Code using unsafe to share Rc across threads had intermittent crashes from data races. Switching to Arc<Mutex<T>> eliminated the undefined behavior.

---

## API Design Mistakes

These anti-patterns create poor interfaces that are hard to use correctly or inconsistent with Rust ecosystem conventions.

### Anti-Pattern: Stringly-Typed APIs

**The Pattern**: Using strings to represent structured data that should have proper types.

```rust
//  ANTI-PATTERN: set_log_level("degub") typo compiles; enum catches at compile time
fn set_log_level(level: &str) {
    match level {
        "debug" | "info" | "warn" | "error" => { /* ... */ }
        _ => panic!("Invalid log level"),
    }
}

fn parse_color(color: &str) -> (u8, u8, u8) {
    match color {
        "red" => (255, 0, 0),
        "green" => (0, 255, 0),
        "blue" => (0, 0, 255),
        _ => panic!("Unknown color"),
    }
}

// Caller must know valid strings
set_log_level("degub");  // Typo! Runtime panic
```

**Why It's Problematic**:
- No compile-time validation
- Easy to make typos
- Runtime errors for invalid values
- No IDE autocomplete
- Poor discoverability

**The Solution**: Use enums for fixed sets of values.

```rust
// CORRECT: set_log_level(LogLevel::Debug) - IDE autocomplete, compile-time check
#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

fn set_log_level(level: LogLevel) {
    match level {
        LogLevel::Debug => { /* ... */ }
        LogLevel::Info => { /* ... */ }
        LogLevel::Warn => { /* ... */ }
        LogLevel::Error => { /* ... */ }
    }
}

#[derive(Debug, Clone, Copy)]
enum Color {
    Red,
    Green,
    Blue,
    Rgb(u8, u8, u8),
}

impl Color {
    fn to_rgb(self) -> (u8, u8, u8) {
        match self { Color::Red => (255, 0, 0), Color::Green => (0, 255, 0), Color::Blue => (0, 0, 255), Color::Rgb(r, g, b) => (r, g, b) }
    }
}

// Compile-time checked, IDE autocomplete
set_log_level(LogLevel::Debug);
// set_log_level(LogLevel::Degub);  // Compile error!

let color = Color::Red;
let custom = Color::Rgb(128, 128, 128);
```

**Real-World Impact**: A database library using string-based query types had 30% of user issues from typos. Switching to enums eliminated these errors.

---

### Anti-Pattern: Boolean Parameter Trap

**The Pattern**: Using boolean parameters where the meaning is unclear at call sites.

```rust
//  ANTI-PATTERN: connect("host", true, false, true) - what do booleans mean?
fn connect(host: &str, encrypted: bool, persistent: bool, verbose: bool) {
    // ...
}

// What do these booleans mean?
connect("localhost", true, false, true);  // Unclear!
connect("localhost", false, true, false);  // What?
```

**Why It's Problematic**:
- Unclear intent at call sites
- Easy to swap arguments
- Hard to extend (what if you need 4 modes, not 2?)
- Poor readability

**The Solution**: Use enums or builder pattern for clarity.

```rust
// CORRECT: connect(host, Encryption::Encrypted, Connection::Transient) - self-documenting
enum Encryption {
    Encrypted,
    Plaintext,
}

enum Connection {
    Persistent,
    Transient,
}

enum Verbosity {
    Verbose,
    Quiet,
}

fn connect(
    host: &str,
    encryption: Encryption,
    connection: Connection,
    verbosity: Verbosity,
) {
    // ...
}

// Clear intent at call site
connect(
    "localhost",
    Encryption::Encrypted,
    Connection::Transient,
    Verbosity::Verbose,
);

// Or use builder pattern
struct ConnectionBuilder {
    host: String,
    encrypted: bool,
    persistent: bool,
    verbose: bool,
}

impl ConnectionBuilder {
    fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            encrypted: false,
            persistent: false,
            verbose: false,
        }
    }

    fn encrypted(mut self) -> Self { self.encrypted = true; self }
    fn persistent(mut self) -> Self { self.persistent = true; self }
    fn verbose(mut self) -> Self { self.verbose = true; self }
    fn connect(self) -> Connection { unimplemented!() }
}

// Clear and fluent
let conn = ConnectionBuilder::new("localhost")
    .encrypted()
    .verbose()
    .connect();
```

**Real-World Impact**: A configuration API with 5 boolean parameters had numerous bugs from swapped arguments. Builder pattern eliminated the errors.

---

### Anti-Pattern: Leaky Abstractions

**The Pattern**: Exposing implementation details in public APIs.

```rust
//  ANTI-PATTERN: pub connection_pool exposes internals; can't change later
pub struct Database {
    pub connection_pool: Vec<Connection>,  // Internal detail exposed
    pub cache: HashMap<String, Vec<u8>>,   // Implementation leaked
}

impl Database {
    pub fn get_connection(&mut self) -> &mut Connection {
        &mut self.connection_pool[0]  // Caller can manipulate pool
    }

    pub fn query(&self, sql: &str) -> Vec<u8> {
        self.cache.get(sql).cloned().unwrap_or_else(|| {
            // Actually query database
            vec![]
        })
    }
}
```

**Why It's Problematic**:
- Can't change implementation without breaking users
- Exposes internal invariants that must be maintained
- No encapsulation
- Hard to evolve API

**The Solution**: Hide implementation details, expose only necessary interfaces.

```rust
// CORRECT: Private fields with public methods; hides implementation
pub struct Database {
    connection_pool: Vec<Connection>,  // Private
    cache: HashMap<String, Vec<u8>>,   // Private
}

impl Database {
    pub fn new(connection_string: &str) -> Result<Self, Error> {
        Ok(Self {
            connection_pool: vec![Connection::new(connection_string)?],
            cache: HashMap::new(),
        })
    }

    pub fn query(&mut self, sql: &str) -> Result<Vec<Row>, Error> {
        // Returns proper type, not implementation detail (Vec<u8>)
        if let Some(cached) = self.cache.get(sql) {
            return Ok(deserialize_rows(cached));
        }

        let conn = self.get_connection_internal()?;
        let result = conn.execute(sql)?;
        let serialized = serialize_rows(&result);
        self.cache.insert(sql.to_string(), serialized);
        Ok(result)
    }

    fn get_connection_internal(&mut self) -> Result<&mut Connection, Error> {
        self.connection_pool.first_mut().ok_or(Error::NoConnections)
    }
}

struct Row {
    // Proper abstraction
}

fn serialize_rows(rows: &[Row]) -> Vec<u8> {
    unimplemented!()
}

fn deserialize_rows(data: &[u8]) -> Vec<Row> {
    unimplemented!()
}
```

**Real-World Impact**: A library exposing internal Vec was locked into using Vec even when a different structure would be better. Proper encapsulation would have allowed evolution.

---

### Anti-Pattern: Returning Owned When Borrowed Suffices

**The Pattern**: Returning owned `String` or `Vec<T>` when a borrowed reference would work.

```rust
//  ANTI-PATTERN: fn get_name(&self) -> String clones on every call; use &str
struct User {
    name: String,
    email: String,
}

impl User {
    fn get_name(&self) -> String { self.name.clone() }   // Allocates every call
    fn get_email(&self) -> String { self.email.clone() }  // Unnecessary clone
}

fn format_user(user: &User) -> String {
    format!("{} <{}>", user.get_name(), user.get_email())
    // Two allocations just to borrow the data
}
```

**Why It's Problematic**:
- Forces allocation on callers
- Wasteful when data is just read
- Less flexible (can't pattern match on borrowed data efficiently)

**The Solution**: Return references for data you already own.

```rust
// CORRECT: fn name(&self) -> &str borrows; zero allocation
impl User {
    fn name(&self) -> &str { &self.name }
    fn email(&self) -> &str { &self.email }
}
fn format_user(user: &User) -> String { format!("{} <{}>", user.name(), user.email()) }  // No extra allocs

impl User {  // Return owned only when creating new data
    fn display_name(&self) -> String { format!("{} ({})", self.name, self.email) }
}
```

**Real-World Impact**: An ORM returning cloned Strings for every field access had 10x memory overhead. Switching to references eliminated the waste.

---

### Anti-Pattern: Overengineered Generic APIs

**The Pattern**: Using complex generic bounds when concrete types would suffice.

```rust
//  ANTI-PATTERN: 6 trait bounds for println!(); &[impl Display] suffices
fn print_items<I, T>(items: I)
where
    I: IntoIterator<Item = T>,
    T: std::fmt::Display + std::fmt::Debug + Clone + Send + Sync + 'static,
{
    for item in items {
        println!("{}", item);
    }
}

// Confusing signature for simple functionality
```

**Why It's Problematic**:
- Harder to understand
- Confusing error messages
- Unnecessary complexity
- Most callers use concrete types anyway

**The Solution**: Use concrete types or simple generics that provide real value.

```rust
// CORRECT: fn print_items(items: &[impl Display]) is simpler and clearer
fn print_items(items: &[impl std::fmt::Display]) { for item in items { println!("{}", item); } }
fn print_items_generic<T: std::fmt::Display>(items: &[T]) { for item in items { println!("{}", item); } }

// Reserve complex bounds for when truly needed
use std::hash::Hash;
use std::collections::HashMap;

fn count_occurrences<T>(items: &[T]) -> HashMap<&T, usize>
where
    T: Hash + Eq,  // Actually needed for HashMap
{
    let mut counts = HashMap::new();
    for item in items {
        *counts.entry(item).or_insert(0) += 1;
    }
    counts
}
```

**Real-World Impact**: A library with overly generic APIs confused users and had poor compile times. Simplifying to concrete types where appropriate improved both.

---

### Summary

Anti-patterns represent accumulated wisdom from mistakes made across the Rust ecosystem. Recognizing these patterns helps you write better code from the start and refactor problematic code when you encounter it.

### Key Principles to Avoid Anti-Patterns

**Ownership and borrowing**:
- Clone only when semantically necessary, not to satisfy the borrow checker
- Use references with lifetimes; resort to Rc/Arc only when genuinely sharing ownership
- Understand why the borrow checker complains before "fixing" with workarounds

**Performance**:
- Chain iterators without intermediate collections
- Use arrays for fixed-size data, Vec for dynamic
- Prefer static dispatch (generics) over dynamic (trait objects) when types known
- Work with &str until you need ownership

**Safety**:
- Use unsafe only when necessary and document invariants
- Handle errors with Result/Option, not unwrap()
- Use proper mutability (&mut) rather than interior mutability by default
- Respect Send/Sync bounds; don't circumvent with unsafe

**API design**:
- Use types (enums) instead of strings/booleans for clarity
- Hide implementation details, expose minimal necessary interface
- Return references when data already owned, own when creating new data
- Keep generics simple unless complexity provides clear value

### Learning from Anti-Patterns

When you find yourself:
- **Adding .clone() to fix errors**: Understand borrowing first
- **Wrapping everything in Rc/Arc**: Reconsider ownership structure
- **Fighting the borrow checker**: The design might be wrong
- **Using unsafe to "fix" safety errors**: Safe solution usually exists
- **Creating complex generic APIs**: Simplicity often better

The Rust compiler is your ally. When it rejects code, it's usually protecting you from real bugs. Instead of fighting it with workarounds, understand what it's preventing and design accordingly. The patterns that feel natural in Rust—ownership, borrowing, iterators, enums—exist because they align with the language's guarantees.

Master these anti-patterns not to never make mistakes, but to recognize and fix them quickly when they appear in your code.
