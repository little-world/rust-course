### Ownership and Borrowing Cheat Sheet
```rust
// ===== OWNERSHIP BASICS =====
// Move semantics (default for non-Copy types)
let s1 = String::from("hello");
let s2 = s1;                                        // s1 moved to s2, s1 invalid
// println!("{}", s1);                              // ERROR: s1 moved

let x = 5;
let y = x;                                          // Copied (i32 is Copy)
println!("{}", x);                                  // OK: x still valid

// Clone for deep copy
let s1 = String::from("hello");
let s2 = s1.clone();                                // Deep copy
println!("{} {}", s1, s2);                         // Both valid

// ===== OWNERSHIP WITH FUNCTIONS =====
// Passing ownership to function
fn takes_ownership(s: String) {                    // s owns the String
println!("{}", s);
}                                                   // s dropped here

let s = String::from("hello");
takes_ownership(s);                                 // s moved
// println!("{}", s);                               // ERROR: s moved

// Return ownership from function
fn gives_ownership() -> String {
String::from("hello")                           // Returns ownership
}

let s = gives_ownership();                          // s owns returned String

// Taking and returning ownership
fn takes_and_gives(s: String) -> String {
s                                               // Return ownership
}

// ===== BORROWING (REFERENCES) =====
// Immutable borrowing
let s1 = String::from("hello");
let len = calculate_length(&s1);                    // Borrow s1
println!("{} {}", s1, len);                        // s1 still valid

fn calculate_length(s: &String) -> usize {         // Borrows String
s.len()
}                                                   // s goes out of scope, nothing dropped

// Mutable borrowing
let mut s = String::from("hello");
change(&mut s);                                     // Mutable borrow
println!("{}", s);

fn change(s: &mut String) {
s.push_str(", world");
}

// ===== BORROWING RULES =====
// Rule 1: Multiple immutable borrows OK
let s = String::from("hello");
let r1 = &s;                                        // OK
let r2 = &s;                                        // OK
println!("{} {}", r1, r2);                         // OK

// Rule 2: Only ONE mutable borrow at a time
let mut s = String::from("hello");
let r1 = &mut s;                                    // OK
// let r2 = &mut s;                                 // ERROR: already borrowed
println!("{}", r1);

// Rule 3: Cannot mix mutable and immutable borrows
let mut s = String::from("hello");
let r1 = &s;                                        // OK
let r2 = &s;                                        // OK
// let r3 = &mut s;                                 // ERROR: immutable borrows exist
println!("{} {}", r1, r2);

// Non-lexical lifetimes (NLL) - borrows end at last use
let mut s = String::from("hello");
let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2);                         // Last use of r1, r2
let r3 = &mut s;                                    // OK: r1, r2 no longer used
println!("{}", r3);

// ===== REFERENCE SCOPE =====
// Reference must be valid
let reference_to_nothing;
{
let x = 5;
// reference_to_nothing = &x;                   // ERROR: x doesn't live long enough
}
// println!("{}", reference_to_nothing);

// Valid reference
let x = 5;
let r = &x;                                         // OK: x outlives r
println!("{}", r);

// ===== DANGLING REFERENCES =====
// Compiler prevents dangling references
fn dangle() -> &String {                            // ERROR: missing lifetime
let s = String::from("hello");
// &s                                           // ERROR: returns reference to local
}

// Fix: return ownership
fn no_dangle() -> String {
let s = String::from("hello");
s                                               // Move ownership out
}

// ===== SLICES (SPECIAL BORROWING) =====
// String slices
let s = String::from("hello world");
let hello = &s[0..5];                               // Immutable borrow of part
let world = &s[6..11];                              // Another immutable borrow
let slice = &s[..];                                 // Entire string

// Array slices
let a = [1, 2, 3, 4, 5];
let slice = &a[1..3];                               // &[i32] type

// Mutable slices
let mut a = [1, 2, 3, 4, 5];
let slice = &mut a[1..3];                           // &mut [i32]
slice[0] = 10;

// ===== COPY TRAIT =====
// Types implementing Copy don't move
let x = 5;                                          // i32 implements Copy
let y = x;                                          // x copied, not moved
println!("{} {}", x, y);                           // Both valid

// Copy types: all integers, bool, char, floats, tuples of Copy types
let tuple = (5, 'a', true);                        // Implements Copy
let tuple2 = tuple;                                 // Copied
println!("{:?} {:?}", tuple, tuple2);

// Non-Copy types: String, Vec, Box, etc.
let v1 = vec![1, 2, 3];                            // Vec doesn't implement Copy
let v2 = v1;                                        // Moved
// println!("{:?}", v1);                            // ERROR

// ===== CLONE TRAIT =====
// Explicit deep copy
let v1 = vec![1, 2, 3];
let v2 = v1.clone();                                // Deep copy
println!("{:?} {:?}", v1, v2);                     // Both valid

// Clone vs Copy
// Copy is implicit, cheap (bitwise)
// Clone is explicit, may be expensive

// ===== DROP TRAIT =====
// Automatic cleanup
{
let s = String::from("hello");                  // s owns String
}                                                   // s.drop() called automatically

// Manual drop
let s = String::from("hello");
drop(s);                                            // Explicitly drop
// println!("{}", s);                               // ERROR: s dropped

// Drop order: reverse of creation
let x = Box::new(5);
let y = Box::new(10);
// Dropped in order: y, then x

// ===== OWNERSHIP PATTERNS =====
// Pattern 1: Multiple owners with Rc
use std::rc::Rc;
let s = Rc::new(String::from("hello"));
let s1 = Rc::clone(&s);                            // Increment ref count
let s2 = Rc::clone(&s);                            // Another reference
println!("{} {} {}", s, s1, s2);                   // All valid

// Pattern 2: Interior mutability with RefCell
use std::cell::RefCell;
let data = RefCell::new(5);
*data.borrow_mut() += 1;                           // Mutable borrow at runtime
println!("{}", data.borrow());                     // Immutable borrow

// Pattern 3: Thread-safe sharing with Arc
use std::sync::Arc;
let data = Arc::new(vec![1, 2, 3]);
let data_clone = Arc::clone(&data);
std::thread::spawn(move || {
println!("{:?}", data_clone);
});

// Pattern 4: Combining Rc and RefCell
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data_clone = Rc::clone(&data);
data.borrow_mut().push(4);
println!("{:?}", data_clone.borrow());

// ===== LIFETIMES (EXPLICIT BORROWING) =====
// Lifetime annotations
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
if x.len() > y.len() { x } else { y }
}

let s1 = String::from("long string");
let result;
{
let s2 = String::from("short");
result = longest(&s1, &s2);                     // result borrows from s1 or s2
println!("{}", result);                        // OK: s2 still valid
}
// println!("{}", result);                          // ERROR: s2 dropped

// Multiple lifetimes
fn first_word<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
x
}

// Lifetime in struct
struct ImportantExcerpt<'a> {
part: &'a str,
}

let novel = String::from("Call me Ishmael. Some years ago...");
let first_sentence = novel.split('.').next().unwrap();
let excerpt = ImportantExcerpt {
part: first_sentence,
};                                                  // excerpt borrows from novel

// Lifetime elision rules
fn first_word(s: &str) -> &str {                   // Lifetimes inferred
&s[..1]
}

// Static lifetime
let s: &'static str = "I have a static lifetime";  // Lives entire program

// ===== REBORROWING =====
// Reborrow immutable reference
fn print_ref(s: &String) {
println!("{}", s);
}

let s = String::from("hello");
let r = &s;
print_ref(r);                                       // r reborrowed
print_ref(r);                                       // Can reborrow again

// Reborrow mutable reference
fn modify(s: &mut String) {
s.push_str(" world");
}

let mut s = String::from("hello");
let r = &mut s;
modify(r);                                          // r reborrowed mutably
// Can't use r after this without reborrowing

// ===== PARTIAL MOVES =====
// Struct field moves
struct Person {
name: String,
age: u32,
}

let person = Person {
name: String::from("Alice"),
age: 30,
};

let name = person.name;                             // name moved
// println!("{}", person.name);                     // ERROR: name moved
println!("{}", person.age);                        // OK: age copied (u32 is Copy)

// Tuple element moves
let tuple = (String::from("hello"), 5);
let (s, n) = tuple;                                 // s moved, n copied
// println!("{}", tuple.0);                         // ERROR: moved
println!("{}", tuple.1);                           // ERROR in older Rust, may work in newer

// ===== BORROWING WITH METHODS =====
impl String {
// Borrows self immutably
fn custom_len(&self) -> usize {
self.len()
}

    // Borrows self mutably
    fn custom_push(&mut self, s: &str) {
        self.push_str(s);
    }
    
    // Takes ownership of self
    fn into_bytes_custom(self) -> Vec<u8> {
        self.into_bytes()
    }
}

// ===== COMMON OWNERSHIP MISTAKES =====
// Mistake 1: Using after move
let s = String::from("hello");
let s2 = s;
// println!("{}", s);                               // ERROR: s moved

// Mistake 2: Multiple mutable borrows
let mut s = String::from("hello");
let r1 = &mut s;
// let r2 = &mut s;                                 // ERROR: already borrowed
println!("{}", r1);

// Mistake 3: Returning reference to local
fn bad() -> &String {                               // ERROR: missing lifetime
let s = String::from("hello");
// &s                                            // ERROR: returns reference to local
}

// Mistake 4: Modifying through immutable reference
let s = String::from("hello");
let r = &s;
// r.push_str(" world");                            // ERROR: can't mutate through &T

// ===== ADVANCED PATTERNS =====
// Splitting borrows
let mut v = vec![1, 2, 3, 4, 5];
let (left, right) = v.split_at_mut(2);             // Split into two mutable slices
left[0] = 10;
right[0] = 20;

// Temporary lifetime extension
let x = &mut String::from("hello");                // Temporary extended
x.push_str(" world");

// Reference in Option
let s = Some(String::from("hello"));
let r = s.as_ref();                                 // Option<&String>
match r {
Some(s) => println!("{}", s),                  // Borrows, doesn't move
None => {},
}
println!("{:?}", s);                               // s still valid

// Ownership with iterators
let v = vec![1, 2, 3];
for x in &v {                                       // Borrow elements
println!("{}", x);
}
println!("{:?}", v);                               // v still valid

let v = vec![1, 2, 3];
for x in v {                                        // Take ownership
println!("{}", x);
}
// println!("{:?}", v);                             // ERROR: v moved

// Mutable iteration
let mut v = vec![1, 2, 3];
for x in &mut v {                                   // Mutable borrow
*x += 1;
}
println!("{:?}", v);
```
