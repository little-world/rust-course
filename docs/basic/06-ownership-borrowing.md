### Ownership and Borrowing in Rust — A Practical Chapter for Experienced Programmers

#### Who this is for
If you already code in languages like C/C++, Java, C#, Go, Python, or Kotlin, this chapter will map Rust’s ownership and borrowing model to concepts you know, then go deeper into how functions interact with values, references, and lifetimes. We’ll focus on how to design and call functions correctly and ergonomically.

---

### Core Mental Model

- Every value in Rust has exactly one owner at a time.
- Moving transfers ownership; the previous binding becomes invalid for use.
- Borrowing uses references to access a value without taking ownership.
- The compiler enforces rules at compile time (no GC, no runtime borrow checks).

For performance and safety: values are dropped (destructors run) when their owner goes out of scope — deterministically (like C++ RAII).

---

### Copy vs Move

- Types that are small and trivially copyable (e.g., integers, bools, char, most `Copy` structs) are implicitly copied on assignment or argument passing.
- Non-`Copy` types (e.g., `String`, `Vec<T>`, `Box<T>`, most user-defined types) move by default on assignment or argument passing.

```rust
let x = 5;        // i32 is Copy
let y = x;        // copy, x is still valid

let s = String::from("hi");
let t = s;        // move, s is now invalid
// println!("{}", s); // ❌ use of moved value
```

Use `.clone()` when you need a deep copy (explicit and potentially expensive):

```rust
let s1 = String::from("data");
let s2 = s1.clone(); // deep copy
```

---

### References and Borrowing

- Immutable borrow: `&T` (read-only view)
- Mutable borrow: `&mut T` (exclusive, read-write view)

Borrowing rules:
- Any number of immutable borrows OR exactly one mutable borrow in a given region.
- Borrows must not outlive the owner (no dangling references).

```rust
fn len_of(s: &String) -> usize { s.len() }
fn push_excl(s: &mut String) { s.push('!') }

let mut msg = String::from("Hello");
let n = len_of(&msg);           // borrow immutably
push_excl(&mut msg);            // borrow mutably
```

Note: For function APIs, prefer `&str` over `&String` when you only need string view access. Similarly, prefer `&[T]` over `&Vec<T>` for slice views.

---

### Function Parameters: Choosing Ownership Semantics

Rust communicates intent through parameter types:

- `fn consume(v: String)`
    - Takes ownership. The caller loses use of the variable unless it was cloned or the function returns it (or part of it) back.
    - Use when you must keep/return the value, mutate and retain it, or when moving into threads/async tasks.

- `fn borrow(v: &String)` or `fn borrow(v: &str)`
    - Immutable borrow. Use for read-only access with minimal cost.

- `fn mutate(v: &mut String)`
    - Mutable borrow. Use when you need to modify the caller’s value in place.

- `fn by_copy(x: i32)`
    - For `Copy` types, passing by value is effectively by-copy and the caller still owns a copy.

Examples:

```rust
fn consume(s: String) { /* takes ownership */ }
fn inspect(s: &str) { println!("size={} bytes", s.len()); }
fn tweak(s: &mut String) { s.make_ascii_uppercase(); }

let mut s = String::from("hello");
inspect(&s);        // borrow immutably

tweak(&mut s);      // borrow mutably; caller keeps ownership

let t = s;          // move to t, s is now invalid here
consume(t);         // move into function
```

Design guideline:
- Prefer borrowing (`&T`, `&mut T`) for library functions unless ownership is explicitly required by the semantics (store it, spawn a task, or return a transformed value without wanting to leave the source valid).

---

### Return Values and Ownership

Return by value moves ownership to the caller. This is idiomatic and cheap due to NRVO and move semantics — especially for heap-backed containers.

```rust
fn to_upper_owned(s: &str) -> String {
    s.to_uppercase() // create and return new owned String
}

let u = to_upper_owned("hello"); // caller owns `u`
```

Mutating in place vs returning new:
- In-place: `fn normalize(name: &mut String)`
- New value: `fn normalized(name: &str) -> String`

Pick based on API clarity and performance tradeoffs.

---

### Method Receivers: `self`, `&self`, `&mut self`

- `fn into_x(self) -> X` takes ownership (consumes the instance) and often returns a transformed type.
- `fn get(&self) -> &T` borrows immutably (read-only method).
- `fn set(&mut self, v: T)` borrows mutably (mutating method).

```rust
impl Buffer {
    fn len(&self) -> usize { self.data.len() }
    fn clear(&mut self) { self.data.clear() }
    fn into_vec(self) -> Vec<u8> { self.data }
}
```

---

### Slices, String Slices, and Iterators

For collection-like APIs:
- Prefer `&[T]` instead of `&Vec<T>`; prefer `&str` instead of `&String`.
- Iteration triplet:
    - `iter()` → `&T` (borrowing iterator, non-consuming)
    - `iter_mut()` → `&mut T` (borrowing iterator, mutating)
    - `into_iter()` → `T` (consuming iterator, takes ownership)

```rust
let v = vec![1, 2, 3];
for x in v.iter() { /* &i32 */ }
for x in v.into_iter() { /* i32, v moved */ }
```

---

### Reborrowing and Shortening Borrows

Rust can temporarily reborrow a mutable reference as an immutable one, and borrows often end earlier than their lexical block when possible (non-lexical lifetimes):

```rust
let mut s = String::from("abc");
let r = &mut s;            // mutable borrow
let len = r.len();         // immutably reborrow inside method
r.push('d');               // still usable here
```

Another example:

```rust
let mut v = vec![1, 2, 3];
let first = &v[0];                 // immutable borrow of first element
v.push(4);                         // ❌ cannot push while `first` is borrowed
println!("{}", first);
```

The borrow checker prevents aliasing + mutation that could invalidate references.

---

### Pattern Matching and Moves

Binding by value in a `let` or `match` moves values unless the type is `Copy` or you explicitly borrow.

```rust
struct Data(String);
let d = Data(String::from("x"));
let Data(inner) = d;       // move `inner` out; `d` is now partially moved and unusable

// Prefer borrowing patterns when needed:
let d = Data(String::from("x"));
let Data(ref inner_ref) = d; // borrow `&String`; `d` stays usable
```

Field moves also apply in struct updates and pattern matches; borrow instead when you need to keep using the original.

---

### Common Function-Call Pitfalls and Fixes

1) Passing ownership accidentally

```rust
fn takes(s: String) { /* ... */ }
let s = String::from("hi");
// takes(s);          // moves, `s` invalid afterward
// println!("{}", s);  // ❌

// Fix: borrow if you don’t need ownership
fn takes_ref(s: &str) { /* ... */ }
takes_ref(&s);
println!("{}", s); // ✅
```

2) Borrow checker complaints about simultaneous borrows

```rust
let mut a = String::from("a");
let r1 = &a;               // immutable
let r2 = &a;               // immutable
// let r3 = &mut a;        // ❌ cannot borrow `a` as mutable because it is also borrowed as immutable
println!("{}, {}", r1, r2);
// r1, r2 end here (last use)
let r3 = &mut a;           // now ok
r3.push('!');
```

3) Iterating while mutating container size

```rust
let mut v = vec![1,2,3];
for x in &v {             // immutable iteration
    // v.push(4);         // ❌ cannot mutate length while immutably borrowed
}
```

Use indexing or collect changes then apply, or use `retain`, `drain_filter` (nightly), etc.

---

### Lifetimes (Function-Level Intuition)

Most lifetimes are inferred. You rarely write them for straightforward functions. When you do, they describe that the output reference can’t outlive the input reference(s).

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}
```

Rules of thumb:
- If your function returns a reference, it must come from an input reference or a global.
- Returned references cannot outlive the owners of the data they point to.
- Methods often don’t need explicit lifetimes thanks to elision rules.

---

### `Clone`, `ToOwned`, and `Cow`

- `.clone()` produces a deep copy. Be explicit and deliberate.
- `.to_owned()` creates an owned value from a borrowed value (e.g., `&str` → `String`).
- `Cow<'a, T>` can be borrowed or owned, enabling APIs that usually borrow but clone on write or when needed.

```rust
use std::borrow::Cow;
fn normalize<'a>(s: &'a str) -> Cow<'a, str> {
    if s.is_ascii() { Cow::Borrowed(s) } else { Cow::Owned(s.to_uppercase()) }
}
```

---

### Interior Mutability (Advanced)

Types like `Cell<T>`, `RefCell<T>`, `Mutex<T>`, and `RwLock<T>` allow mutation behind an immutable reference, by enforcing rules at runtime (single-threaded for `RefCell`, multi-threaded for `Mutex`/`RwLock`). Use sparingly—prefer standard borrowing when possible.

```rust
use std::cell::RefCell;
struct C { x: RefCell<i32> }
let c = C { x: RefCell::new(0) };
*c.x.borrow_mut() = 42; // runtime-checked borrow
```

---

### Closures and Captures

Closures capture by reference, mutable reference, or by move depending on usage:

```rust
let mut s = String::from("hi");
let print = || println!("{}", s); // borrows `&s`

let mut up = || s.make_ascii_uppercase(); // borrows `&mut s`

let moved = move || drop(s); // takes ownership into the closure
```

Use `move` when spawning threads or tasks that outlive the current scope:

```rust
std::thread::spawn(move || {
    // owns captured values
});
```

---

### FFI and Pinning (Brief Notes)

- FFI to C often uses raw pointers; you must manage lifetimes manually and avoid dangling references.
- `Pin<T>` ensures objects won’t move in memory (relevant for self-referential types and async state machines). This is tangential but related to movement semantics.

---

### API Design Checklist (Functions)

- Do you need to keep the argument? Use `T` (ownership).
- Only need to read? Use `&T` or more general `&U` (e.g., `&str`, `&[T]`).
- Need to mutate in place? Use `&mut T`.
- Want to return a new value while keeping input valid? Take `&T`/`&U`, return `Owned`.
- Prefer trait-based generics (`AsRef<str>`, `Into<String>`) only where ergonomics justify it.
- Avoid overusing `.clone()`; measure and justify copies.

---

### Practical Patterns

1) Builder that consumes self to enforce single-use transitions
```rust
struct ReqBuilder { url: String, body: String }
impl ReqBuilder {
    fn new(url: impl Into<String>) -> Self { Self { url: url.into(), body: String::new() } }
    fn body(mut self, b: impl Into<String>) -> Self { self.body = b.into(); self }
    fn send(self) -> Response { /* consumes builder */ unimplemented!() }
}
```

2) Parser with zero-copy borrowing
```rust
struct View<'a> { head: &'a str, tail: &'a str }
fn split_once(s: &str) -> View<'_> {
    if let Some(i) = s.find(':') { View { head: &s[..i], tail: &s[i+1..] } } else { View { head: s, tail: "" } }
}
```

3) Mutate-in-place utilities
```rust
fn trim_in_place(s: &mut String) {
    let t = s.trim();
    if t.len() != s.len() { s.replace_range(.., t); }
}
```

---

### Reading Compiler Errors

- “value moved here” → You passed ownership. Borrow instead or clone deliberately.
- “cannot borrow `x` as mutable because it is also borrowed as immutable” → ensure earlier immutable borrows are no longer used before creating a mutable borrow; restructure code to shorten borrow lifetimes.
- “borrowed value does not live long enough” → a reference may outlive its owner; ensure the owner lives at least as long as any references.

---

### Quick Reference

- Own: `T` (move semantics)
- Borrow immutably: `&T`
- Borrow mutably: `&mut T`
- Copy types: passed and returned by value cheaply (`Copy`)
- Non-Copy: move by default; use `.clone()` for deep copy
- String views and slices: `&str`, `&[T]` for APIs
- Iteration: `iter()` = `&T`, `iter_mut()` = `&mut T`, `into_iter()` = `T`
- Receivers: `&self` read-only, `&mut self` mutating, `self` consuming

---

### From Other Languages: Mental Map

- C++: Like move semantics + RAII, but enforced borrowing rules prevent UB by aliasing + mutation. Think unique_ptr by default; shared ownership needs `Rc`/`Arc`.
- Java/C#: No GC here; ownership is static. Borrowing is like passing references, but compiler enforces aliasing and lifetime safety.
- Go: No GC; think of explicit ownership passing and borrowing preventing data races even in single-threaded code. For concurrency, use `Arc<Mutex<T>>` for shared ownership + mutation.
- Python: Far fewer implicit copies; you’ll often pass `&T` instead of duplicating.

---

### Exercises (Short)

1) Write `fn first<'a>(a: &'a str, b: &'a str) -> &'a str` returning `a`.
2) Write `fn append_excl(s: &mut String)` that appends `!` without returning.
3) Design both APIs for normalization: `fn normalize_in_place(&mut String)` and `fn normalized(&str) -> String`.

---

### Final Takeaways

- Function signatures in Rust are your contract for ownership.
- Prefer borrowing in APIs; consume only when necessary.
- Cloning is explicit – measure and avoid accidental copies.
- Let the compiler guide you: fix moves/borrows by clarifying who owns what and when.

For more foundational examples, see your project’s existing docs:
- `docs/basic/06-ownership.md` — concise intro and function interactions
- `docs/basic/05-functions.md` — parameters, returns, and ownership basics
- `docs/cookbook/13-smart-pointers.md` — advanced ownership via smart pointers