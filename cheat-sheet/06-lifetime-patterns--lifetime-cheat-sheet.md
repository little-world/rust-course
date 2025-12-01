### Lifetime Cheat Sheet

```rust
// ===== BASIC LIFETIMES =====
// Lifetime annotations tell the compiler how long references are valid

// Function with lifetime annotation
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn basic_lifetime_example() {
    let string1 = String::from("long string");
    let string2 = String::from("short");
    
    let result = longest(&string1, &string2);
    println!("Longest: {}", result);
}

// Multiple lifetime parameters
fn first_word<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x.split_whitespace().next().unwrap_or("")
}

// ===== LIFETIME IN STRUCTS =====
// Struct that holds references
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

fn struct_lifetime_example() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().expect("Could not find '.'");
    
    let excerpt = ImportantExcerpt {
        part: first_sentence,
    };
    
    println!("Excerpt: {}", excerpt.part);
}

// Multiple lifetimes in struct
struct Context<'a, 'b> {
    name: &'a str,
    config: &'b Config,
}

struct Config {
    timeout: u64,
}

// ===== LIFETIME ELISION RULES =====
// Compiler can infer lifetimes in certain cases

// Rule 1: Each parameter gets its own lifetime
fn simple(s: &str) -> &str {
    // Equivalent to: fn simple<'a>(s: &'a str) -> &'a str
    s
}

// Rule 2: If one input lifetime, it's assigned to all outputs
fn first_char(s: &str) -> &str {
    &s[0..1]
}

// Rule 3: If &self or &mut self, lifetime of self is assigned to outputs
impl<'a> ImportantExcerpt<'a> {
    fn get_part(&self) -> &str {
        // Return type gets lifetime of &self
        self.part
    }
}

// ===== STATIC LIFETIME =====
// 'static means the reference lives for the entire program

fn static_lifetime_example() {
    let s: &'static str = "I have a static lifetime";
    
    // String literals always have 'static lifetime
    let literal = "Hello, world!";
    
    // Leaked memory also has 'static lifetime
    let leaked: &'static str = Box::leak(Box::new(String::from("leaked")));
}

// Function requiring static lifetime
fn needs_static(s: &'static str) {
    println!("{}", s);
}

// ===== LIFETIME BOUNDS =====
// Constrain type parameter lifetimes

// T must live at least as long as 'a
fn print_ref<'a, T>(value: &'a T)
where
    T: std::fmt::Display + 'a,
{
    println!("{}", value);
}

// Struct with lifetime bound
struct Ref<'a, T: 'a> {
    value: &'a T,
}

// Generic lifetime bound
impl<'a, T> Ref<'a, T>
where
    T: 'a,
{
    fn new(value: &'a T) -> Self {
        Ref { value }
    }
}

// ===== MULTIPLE LIFETIMES =====
// Different parameters can have different lifetimes

struct MultiLife<'a, 'b> {
    first: &'a str,
    second: &'b str,
}

fn multiple_lifetimes<'a, 'b>(x: &'a str, y: &'b str) -> (&'a str, &'b str) {
    (x, y)
}

fn multi_example() {
    let s1 = String::from("first");
    {
        let s2 = String::from("second");
        let (r1, r2) = multiple_lifetimes(&s1, &s2);
        println!("{} {}", r1, r2);
    }
    // r2 is out of scope here
}

// ===== LIFETIME SUBTYPING =====
// 'a: 'b means 'a lives at least as long as 'b

struct Parser<'c, 's> {
    context: &'c str,
    source: &'s str,
}

impl<'c, 's> Parser<'c, 's> {
    fn parse(&self) -> Result<(), &'s str> {
        Ok(())
    }
}

// Lifetime subtyping example
fn subtyping<'a, 'b>(x: &'a str, y: &'b str) -> &'a str
where
    'b: 'a,  // 'b outlives 'a
{
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// ===== HIGHER-RANKED TRAIT BOUNDS (HRTB) =====
// for<'a> syntax for lifetime bounds that work for any lifetime

trait DoSomething {
    fn do_it(&self, data: &str);
}

// Function that works for any lifetime
fn call_with_ref<F>(f: F)
where
    F: for<'a> Fn(&'a str),
{
    let s = String::from("temporary");
    f(&s);
}

fn hrtb_example() {
    call_with_ref(|s| {
        println!("Got: {}", s);
    });
}

// ===== LIFETIME IN CLOSURES =====
fn closure_lifetime_example() {
    let s = String::from("hello");
    
    // Closure borrowing
    let print = || println!("{}", s);
    print();
    println!("{}", s); // s still valid
    
    // Closure taking ownership
    let consume = move || println!("{}", s);
    consume();
    // println!("{}", s); // ERROR: s moved
}

// Function returning closure with lifetime
fn make_adder<'a>(x: &'a i32) -> impl Fn(i32) -> i32 + 'a {
    move |y| x + y
}

// ===== LIFETIME IN ITERATORS =====
struct WordIterator<'a> {
    text: &'a str,
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }
        
        match self.text.find(' ') {
            Some(pos) => {
                let word = &self.text[..pos];
                self.text = &self.text[pos + 1..];
                Some(word)
            }
            None => {
                let word = self.text;
                self.text = "";
                Some(word)
            }
        }
    }
}

fn iterator_lifetime_example() {
    let text = String::from("hello world rust");
    let iter = WordIterator { text: &text };
    
    for word in iter {
        println!("{}", word);
    }
}

// ===== SELF-REFERENTIAL STRUCTS =====
// Cannot directly create self-referential structs in safe Rust

// This won't work:
// struct SelfRef<'a> {
//     data: String,
//     reference: &'a str, // Cannot reference 'data' field
// }

// Solution 1: Use indices instead of references
struct SafeSelfRef {
    data: String,
    start: usize,
    end: usize,
}

impl SafeSelfRef {
    fn new(data: String, start: usize, end: usize) -> Self {
        SafeSelfRef { data, start, end }
    }
    
    fn get_slice(&self) -> &str {
        &self.data[self.start..self.end]
    }
}

// Solution 2: Use Pin for self-referential structs (advanced)
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    ptr: *const String,
    _pin: PhantomPinned,
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::pin(SelfReferential {
            data,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });
        
        let ptr = &boxed.data as *const String;
        unsafe {
            let mut_ref = Pin::as_mut(&mut boxed);
            Pin::get_unchecked_mut(mut_ref).ptr = ptr;
        }
        
        boxed
    }
    
    fn get_data(&self) -> &str {
        unsafe { &*self.ptr }
    }
}

// ===== COMMON LIFETIME PATTERNS =====

// Pattern 1: Returning references from struct methods
struct Container<'a> {
    data: &'a [i32],
}

impl<'a> Container<'a> {
    fn first(&self) -> Option<&i32> {
        self.data.first()
    }
    
    fn last(&self) -> Option<&i32> {
        self.data.last()
    }
}

// Pattern 2: Splitting references
fn split_at<'a>(s: &'a str, mid: usize) -> (&'a str, &'a str) {
    (&s[..mid], &s[mid..])
}

// Pattern 3: Caching/memoization with lifetimes
struct Cache<'a, T> {
    value: Option<T>,
    generator: &'a dyn Fn() -> T,
}

impl<'a, T> Cache<'a, T> {
    fn new(generator: &'a dyn Fn() -> T) -> Self {
        Cache {
            value: None,
            generator,
        }
    }
    
    fn get(&mut self) -> &T {
        if self.value.is_none() {
            self.value = Some((self.generator)());
        }
        self.value.as_ref().unwrap()
    }
}

// Pattern 4: Builder with references
struct RequestBuilder<'a> {
    method: &'a str,
    url: &'a str,
    headers: Vec<(&'a str, &'a str)>,
}

impl<'a> RequestBuilder<'a> {
    fn new(method: &'a str, url: &'a str) -> Self {
        RequestBuilder {
            method,
            url,
            headers: Vec::new(),
        }
    }
    
    fn header(mut self, key: &'a str, value: &'a str) -> Self {
        self.headers.push((key, value));
        self
    }
}

// ===== LIFETIME TROUBLESHOOTING =====

// Problem: Returning reference to local variable
// fn dangling_reference() -> &str {
//     let s = String::from("hello");
//     &s // ERROR: s doesn't live long enough
// }

// Solution: Return owned value
fn no_dangling() -> String {
    let s = String::from("hello");
    s
}

// Problem: Mutable and immutable borrows
fn borrow_problem() {
    let mut s = String::from("hello");
    
    // let r1 = &s;
    // let r2 = &mut s; // ERROR: cannot borrow as mutable
    // println!("{}", r1);
    
    // Solution: End immutable borrow before mutable
    {
        let r1 = &s;
        println!("{}", r1);
    } // r1 goes out of scope
    
    let r2 = &mut s;
    r2.push_str(" world");
}

// Problem: Lifetime too restrictive
fn restrictive<'a>(x: &'a str, y: &str) -> &'a str {
    // Cannot return y because it doesn't have lifetime 'a
    x
}

// Solution: Add second lifetime parameter
fn flexible<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x
}

// ===== ADVANCED LIFETIME SCENARIOS =====

// Lifetime variance
struct Covariant<'a, T> {
    value: &'a T,
}

struct Contravariant<'a, T> {
    callback: fn(&'a T),
}

struct Invariant<'a, T> {
    value: &'a mut T,
}

// Lifetime with trait objects
trait Speak {
    fn speak(&self);
}

fn speak_twice<'a>(speaker: &'a dyn Speak) {
    speaker.speak();
    speaker.speak();
}

// Lifetime with associated types
trait Producer {
    type Item;
    fn produce(&self) -> Self::Item;
}

struct StringProducer<'a> {
    template: &'a str,
}

impl<'a> Producer for StringProducer<'a> {
    type Item = &'a str;
    
    fn produce(&self) -> Self::Item {
        self.template
    }
}

// ===== LIFETIME WITH GENERIC TYPES =====
struct Holder<'a, T>
where
    T: 'a,
{
    value: &'a T,
}

impl<'a, T> Holder<'a, T>
where
    T: 'a,
{
    fn new(value: &'a T) -> Self {
        Holder { value }
    }
    
    fn get(&self) -> &T {
        self.value
    }
}

// ===== LIFETIME IN ASYNC =====
// Lifetimes in async functions
async fn async_lifetime<'a>(s: &'a str) -> &'a str {
    // Simulate async work
    s
}

// Struct with async method and lifetime
struct AsyncContext<'a> {
    data: &'a str,
}

impl<'a> AsyncContext<'a> {
    async fn process(&self) -> String {
        format!("Processing: {}", self.data)
    }
}

// ===== LIFETIME ANNOTATIONS IN PRACTICE =====

// Real-world example: Parser
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, pos: 0 }
    }
    
    fn parse_word(&mut self) -> Option<&'a str> {
        let start = self.pos;
        
        while self.pos < self.input.len() {
            if self.input.as_bytes()[self.pos].is_ascii_whitespace() {
                break;
            }
            self.pos += 1;
        }
        
        if start == self.pos {
            None
        } else {
            let word = &self.input[start..self.pos];
            self.pos += 1; // Skip whitespace
            Some(word)
        }
    }
    
    fn remaining(&self) -> &'a str {
        &self.input[self.pos..]
    }
}

fn parser_example() {
    let text = "hello world rust";
    let mut parser = Parser::new(text);
    
    while let Some(word) = parser.parse_word() {
        println!("Word: {}", word);
    }
}

// Real-world example: String pool
struct StringPool {
    pool: Vec<String>,
}

impl StringPool {
    fn new() -> Self {
        StringPool { pool: Vec::new() }
    }
    
    fn intern(&mut self, s: String) -> &str {
        if !self.pool.iter().any(|existing| existing == &s) {
            self.pool.push(s);
        }
        self.pool.iter().find(|existing| *existing == &s).unwrap()
    }
    
    fn get(&self, index: usize) -> Option<&str> {
        self.pool.get(index).map(|s| s.as_str())
    }
}

// ===== COMMON MISTAKES AND SOLUTIONS =====

// Mistake 1: Trying to return reference to temporary
// fn wrong() -> &str {
//     let temp = String::from("temp");
//     &temp // ERROR
// }

// Solution: Return owned value or use static
fn correct() -> String {
    String::from("temp")
}

// Mistake 2: Lifetime conflicts in method chains
struct Builder<'a> {
    parts: Vec<&'a str>,
}

impl<'a> Builder<'a> {
    fn new() -> Self {
        Builder { parts: Vec::new() }
    }
    
    fn add(mut self, part: &'a str) -> Self {
        self.parts.push(part);
        self
    }
    
    fn build(self) -> String {
        self.parts.join("")
    }
}

fn builder_lifetime_example() {
    let part1 = String::from("hello");
    let part2 = String::from(" world");
    
    let result = Builder::new()
        .add(&part1)
        .add(&part2)
        .build();
    
    println!("{}", result);
}

// Mistake 3: Conflating lifetimes unnecessarily
// Better to be specific when different lifetimes are needed
fn specific_lifetimes<'a, 'b>(
    long_lived: &'a str,
    short_lived: &'b str,
) -> &'a str {
    long_lived
}
```

