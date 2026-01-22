# Reference & Iterator Patterns

This chapter explores the type-level mechanics of Rust's reference system. We cover reference binding in patterns, the `Deref` trait hierarchy, auto-referencing rules, method resolution order, and iterator protocols—focusing on how the type system enforces borrowing semantics and enables ergonomic abstractions.

## Pattern 1: Reference Binding and Type Inference

* **Problem**: Pattern matching on owned values moves non-`Copy` fields, but matching on references requires understanding how binding modes propagate through nested structures. Incorrect assumptions lead to unexpected moves or type mismatches.
* **Solution**: Leverage Rust 2018's binding mode inference—when matching on `&T`, bindings automatically become references. Use explicit `ref`/`ref mut` only when matching on owned values or overriding the inferred mode.

### Example: Binding Mode Inference

Rust 2018 introduced *binding mode inference*: when matching on a reference, the binding mode propagates inward.

```rust
#[derive(Clone)]
struct Pair<T>(T, T);

fn binding_modes<T: std::fmt::Debug>(pair: &Pair<T>) {
    // Matching on &Pair<T>: binding mode is "ref" by default
    let Pair(a, b) = pair;
    // a: &T, b: &T (inferred from matching on reference)

    // Explicit ref is redundant here but clarifies intent
    let Pair(ref x, ref y) = *pair;
    // x: &T, y: &T
}

fn explicit_move<T: Clone>(pair: &Pair<T>) {
    // To override binding mode and clone:
    let Pair(a, b) = pair.clone();
    // a: T, b: T (moved from cloned Pair)
}

// Usage:
// let p = Pair(1, 2);
// binding_modes(&p);  // a and b are &i32
// explicit_move(&p);  // a and b are i32 (cloned)
```

The binding mode rules:
1. Match on `T` → bindings move (unless `Copy`)
2. Match on `&T` → bindings are `&` (ref mode)
3. Match on `&mut T` → bindings are `&mut` (ref mut mode)
4. Explicit `ref`/`ref mut` overrides the inherited mode

### Example: Mixed Binding Modes

Copy fields are copied even when matching on a reference, while non-Copy fields become references. Use explicit `ref` to force reference binding for Copy types when you need a reference rather than a copy.

```rust
struct Record {
    id: u64,           // Copy
    data: Vec<u8>,     // !Copy
    name: String,      // !Copy
}

fn mixed_bindings(record: &Record) {
    // id copies, data and name are references
    let Record { id, data, name } = record;
    // id: u64 (copied), data: &Vec<u8>, name: &String

    // This works because u64: Copy, so it's copied rather than moved
}

fn force_ref_for_copy(record: &Record) {
    // Force reference even for Copy types
    let Record { ref id, .. } = *record;
    // id: &u64
}

// Usage:
// let r = Record { id: 42, data: vec![1, 2], name: "test".into() };
// mixed_bindings(&r);  // id copies, data/name are references
```

### Example: Pattern Matching with Enums and Nested References

Matching on `&self` in enum methods propagates reference mode through the pattern. Explicit `ref mut` is needed when matching on `&mut self` to clarify intent and avoid moving inner values.

```rust
enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

impl<T> Tree<T> {
    fn left(&self) -> Option<&Tree<T>> {
        match self {
            Tree::Node(left, _) => Some(left),
            // left: &Box<Tree<T>>, deref coercion gives &Tree<T>
            Tree::Leaf(_) => None,
        }
    }

    fn left_mut(&mut self) -> Option<&mut Tree<T>> {
        match self {
            Tree::Node(ref mut left, _) => Some(left),
            // Explicit ref mut needed when matching on &mut self
            // to clarify we want &mut Box, not to move Box
            Tree::Leaf(_) => None,
        }
    }
}

// Usage:
// let tree = Tree::Node(Box::new(Tree::Leaf(1)), Box::new(Tree::Leaf(2)));
// if let Some(left) = tree.left() { println!("has left subtree"); }
```

## Pattern 2: Deref Trait Mechanics and Type Resolution

* **Problem**: The `*` operator's behavior isn't obvious—`deref()` returns `&Target`, yet `*x` produces `Target`, not `&Target`. Misunderstanding this leads to confusion about when deref coercion applies.
* **Solution**: Recognize that `*x` desugars to `*Deref::deref(&x)`, adding an implicit dereference of the returned reference. Implement `Deref` only for smart pointer semantics (container → contained), never for general type conversion.
* **Why It Matters**: `Deref` enables the ergonomics of `Box<T>`, `Rc<T>`, `Arc<T>`, and `String`. Understanding the type-level mechanics is essential for building custom smart pointers and predicting coercion behavior.

### Example: The Deref Hierarchy

`*x` desugars to `*Deref::deref(&x)`—the deref method returns `&Target`, then `*` dereferences that. This two-step process means `*wrapper` yields `Target`, not `&Target`, enabling transparent access to wrapped values.

```rust
use std::ops::{Deref, DerefMut};

// Deref defines: fn deref(&self) -> &Self::Target
// DerefMut defines: fn deref_mut(&mut self) -> &mut Self::Target

// The relationship between * and deref:
// *x where x: T is equivalent to *Deref::deref(&x)
// This means *x: Self::Target, not &Self::Target

struct Wrapper<T>(T);

impl<T> Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.0 }
}

impl<T> DerefMut for Wrapper<T> {
    fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

fn deref_typing() {
    let w: Wrapper<String> = Wrapper(String::from("hello"));

    // Type of expressions:
    let _: &String = &*w;           // explicit deref then ref
    let _: &String = w.deref();     // method call
    let _: &str = &*w;              // deref coercion: &String -> &str

    // The * operator dereferences the return value of deref()
    // *w is sugar for *(w.deref()), which is *(&self.0), which is self.0
}

// Usage:
// let w = Wrapper(42);
// println!("{}", *w);  // prints 42 via Deref
```

### Example: Deref Coercion Rules

Deref coercion applies in specific contexts:

```rust
fn takes_str(s: &str) {}
fn takes_mut_slice(s: &mut [u8]) {}

fn coercion_contexts() {
    let s = String::from("hello");
    let boxed = Box::new(String::from("boxed"));

    // Coercion sites:
    // 1. Function/method arguments
    takes_str(&s);          // &String -> &str
    takes_str(&boxed);      // &Box<String> -> &String -> &str

    // 2. Let bindings with explicit type
    let _: &str = &s;       // coerced
    let _: &str = &boxed;   // coerced through two Derefs

    // 3. Struct field initialization (if field type is explicit)
    struct Holder<'a> { s: &'a str }
    let _ = Holder { s: &s };

    // 4. Return position (if return type is explicit)
    fn return_coercion(s: &String) -> &str { s }
}

// Coercion rules:
// &T      -> &U       where T: Deref<Target=U>
// &mut T  -> &mut U   where T: DerefMut<Target=U>
// &mut T  -> &U       where T: Deref<Target=U>  (mut to shared OK)
// &T      -> &mut U   NEVER (shared to mut forbidden)

// Usage:
// let s = String::from("hello");
// takes_str(&s);  // automatic coercion from &String to &str
```

### Example: DerefMut and Interior Mutability Interaction

Combines `DerefMut` with `RefCell` for tracking mutations—the write counter uses interior mutability while `DerefMut` provides mutable access to the wrapped value. This pattern is valid because metadata mutation is separate from the value's mutability.

```rust
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

struct TrackedMut<T> {
    value: T,
    write_count: RefCell<usize>,
}

impl<T> Deref for TrackedMut<T> {
    type Target = T;
    fn deref(&self) -> &T { &self.value }
}

impl<T> DerefMut for TrackedMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        *self.write_count.borrow_mut() += 1;
        &mut self.value
    }
}

// Key insight: DerefMut requires &mut self, but the RefCell
// allows mutation through &self. This is a valid pattern because
// we're not mutating through DerefMut, we're using interior mutability
// for metadata while DerefMut gives mutable access to the wrapped value.

// Usage:
// let mut t = TrackedMut { value: String::new(), write_count: RefCell::new(0) };
// t.push_str("hello");  // increments write_count via DerefMut
// println!("writes: {}", t.write_count.borrow());  // prints 1
```

## Pattern 3: Type System Rules for Auto-Referencing

* **Problem**: Method calls like `x.method()` work regardless of whether `x` is `T`, `&T`, `&mut T`, or `Box<T>`. The implicit conversions are convenient but can cause subtle bugs when the wrong method is selected.
* **Solution**: Understand the method resolution algorithm: Rust tries receivers in a specific order (value → ref → mut ref), then derefs and repeats. Use fully qualified syntax (`Trait::method(&x)`) when disambiguation is needed.
* **Why It Matters**: Auto-referencing enables ergonomic APIs but can select unexpected methods. Knowing the resolution order helps predict behavior and debug "method not found" or ambiguity errors.

### Example: Method Resolution Order

Rust tries method receivers in order: by value, by ref, by mut ref—first for inherent methods, then trait methods. If no match, it derefs and repeats. This order determines which method is called when multiple candidates exist.

```rust
struct S;

impl S {
    fn by_value(self) {}
    fn by_ref(&self) {}
    fn by_mut(&mut self) {}
}

fn resolution_order() {
    let mut s = S;

    // For s.method(), Rust tries in order:
    // 1. S::method(s)           - inherent, by value
    // 2. S::method(&s)          - inherent, by ref
    // 3. S::method(&mut s)      - inherent, by mut ref
    // 4. <S as Trait>::method(s)    - trait, by value
    // 5. <S as Trait>::method(&s)   - trait, by ref
    // 6. <S as Trait>::method(&mut s) - trait, by mut ref
    // 7. Deref to U, repeat 1-6 with U
    // 8. Unsized coercion, repeat

    // This order means by_ref is NOT called via auto-ref when
    // by_value exists, unless by_value's receiver doesn't match.
}

// Usage:
// let mut s = S;
// s.by_ref();   // auto-ref: (&s).by_ref()
// s.by_mut();   // auto-ref: (&mut s).by_mut()
// s.by_value(); // consumes s
```

### Example: Deref Chains in Method Resolution

Method calls on `Rc<Outer>` automatically traverse the deref chain: `Rc<Outer>` → `Outer` → `Inner`. Rust finds `inner_method` on `Inner` and applies auto-ref to call it through the entire wrapper chain.

```rust
use std::rc::Rc;

struct Inner;
impl Inner {
    fn inner_method(&self) -> &'static str { "inner" }
}

struct Outer(Inner);
impl std::ops::Deref for Outer {
    type Target = Inner;
    fn deref(&self) -> &Inner { &self.0 }
}

fn deref_chain_resolution() {
    let rc: Rc<Outer> = Rc::new(Outer(Inner));

    // rc.inner_method() resolution:
    // 1. Rc<Outer> doesn't have inner_method
    // 2. Deref Rc<Outer> -> Outer, try Outer::inner_method
    // 3. Outer doesn't have inner_method
    // 4. Deref Outer -> Inner, try Inner::inner_method
    // 5. Found: Inner::inner_method(&self)
    // 6. Apply auto-ref: (&*(*rc)).inner_method()

    let _: &str = rc.inner_method();
}

// Usage:
// let rc = Rc::new(Outer(Inner));
// rc.inner_method();  // resolves through Rc -> Outer -> Inner
```

### Example: Ambiguity and Explicit Disambiguation

When multiple traits define the same method name, calling `s.method()` is ambiguous. Use fully qualified syntax `A::method(&s)` or `<S as A>::method(&s)` to specify which trait implementation to call.

```rust
trait A { fn method(&self) -> i32; }
trait B { fn method(&self) -> i32; }

struct S;
impl A for S { fn method(&self) -> i32 { 1 } }
impl B for S { fn method(&self) -> i32 { 2 } }

fn disambiguation() {
    let s = S;

    // s.method(); // Error: ambiguous

    // Fully qualified syntax:
    let _: i32 = A::method(&s);        // calls A::method
    let _: i32 = B::method(&s);        // calls B::method
    let _: i32 = <S as A>::method(&s); // explicit trait
}

// Usage:
// let s = S;
// assert_eq!(A::method(&s), 1);
// assert_eq!(B::method(&s), 2);
```

## Pattern 4: Reference Variance and Subtyping

* **Problem**: Lifetime errors in generic code often stem from variance mismatches. A function accepting `&'a mut Vec<&'a str>` has different constraints than one accepting `&'a Vec<&'a str>`, but the reason isn't immediately obvious.
* **Solution**: Learn variance rules: `&'a T` is covariant in both `'a` and `T`; `&'a mut T` is covariant in `'a` but invariant in `T`. Use `PhantomData` to control variance in custom types.
* **Why It Matters**: Variance determines when one type can substitute for another. Incorrect variance in generic types causes confusing lifetime errors or unsoundness. Understanding variance is essential for library authors.

### Example: Variance of Reference Types

Shared references `&'a T` are covariant in both lifetime and type—longer-lived refs can substitute for shorter. Mutable references `&'a mut T` are invariant in `T` to prevent writing shorter-lived values or reading as longer-lived.

```rust
// Variance rules for references:
// &'a T     is covariant in 'a and covariant in T
// &'a mut T is covariant in 'a and invariant in T

fn covariance_demo<'long, 'short>(
    long_ref: &'long str,
    short_ref: &'short str,
) where 'long: 'short {
    // 'long: 'short means 'long outlives 'short

    // &'long T can be used where &'short T is expected
    let _: &'short str = long_ref;  // OK: covariant in lifetime

    // Cannot go the other way
    // let _: &'long str = short_ref;  // Error
}

fn invariance_demo<'a>(
    mut_ref: &'a mut Vec<&'a str>,
) {
    // &mut T is invariant in T
    // This prevents:
    // 1. Inserting shorter-lived references
    // 2. Extracting longer-lived references

    // If T = Vec<&'a str>, we cannot treat it as Vec<&'static str>
    // even though &'static str: 'a
}

// Usage:
// covariance_demo("static", &String::from("temp"));  // 'static outlives temp
```

### Example: PhantomData for Variance Control

`PhantomData<T>` makes types covariant, `PhantomData<fn(T)>` makes them contravariant, `PhantomData<fn(T) -> T>` makes them invariant. Use these markers to control subtyping behavior in generic types that don't directly store `T`.

```rust
use std::marker::PhantomData;

// Covariant in T (default, like storing T)
struct Covariant<T>(PhantomData<T>);

// Contravariant in T (like taking T as input)
struct Contravariant<T>(PhantomData<fn(T)>);

// Invariant in T (like storing &mut T)
struct Invariant<T>(PhantomData<fn(T) -> T>);

// Practical example: a handle that conceptually "owns" T
struct Handle<T> {
    id: u64,
    _marker: PhantomData<T>,  // Covariant: Handle<Cat> can be Handle<Animal>
}

// A handle that can produce T values
struct Producer<T> {
    id: u64,
    _marker: PhantomData<fn() -> T>,  // Covariant in T
}

// A handle that consumes T values
struct Consumer<T> {
    id: u64,
    _marker: PhantomData<fn(T)>,  // Contravariant in T
}

// Usage:
// let h: Handle<String> = Handle { id: 1, _marker: PhantomData };
// // Handle<String> can be used where Handle<&str> might be expected (covariant)
```

### Example: Lifetime Bounds and Higher-Ranked Trait Bounds

Regular lifetime parameters fix the lifetime at the call site. Higher-ranked bounds (`for<'a>`) require the function to work with any lifetime, enabling patterns like `sort_by` where the comparison function must handle arbitrary borrowed elements.

```rust
// Regular lifetime parameter
fn regular<'a, F>(f: F)
where
    F: Fn(&'a str) -> &'a str
{
    // F works with one specific lifetime 'a
}

// Higher-ranked: F works for ANY lifetime
fn higher_ranked<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str
{
    // F must work for any lifetime the caller provides
    let s = String::from("hello");
    let result = f(&s);  // Works with the local lifetime of s
}

// Practical example: comparison functions
fn sort_by<T, F>(slice: &mut [T], compare: F)
where
    F: for<'a> Fn(&'a T, &'a T) -> std::cmp::Ordering
{
    // compare must work with any borrowed elements
    slice.sort_by(|a, b| compare(a, b));
}

// Usage:
// higher_ranked(|s| s);  // identity fn works for any lifetime
// sort_by(&mut vec![3, 1, 2], |a, b| a.cmp(b));
```

## Pattern 5: Reborrowing and Borrow Splitting

* **Problem**: Mutable references are move-only (`&mut T` is not `Copy`), yet passing `&mut` to a function doesn't always consume it. When does Rust implicitly reborrow, and when can you hold multiple `&mut` to different parts of a structure?
* **Solution**: Reborrowing (`&mut *r`) creates a temporary borrow from an existing `&mut`, freezing the original. Borrow splitting allows simultaneous `&mut` to disjoint fields or slice regions through methods like `split_at_mut`.
* **Why It Matters**: Reborrowing enables ergonomic `&mut` passing without explicit lifetime management. Borrow splitting is essential for algorithms that need mutable access to multiple parts of a data structure simultaneously.

### Example: Reborrowing Mechanics

Passing `&mut` to a function doesn't move it—Rust implicitly reborrows (`&mut *r`), creating a temporary borrow that freezes the original. After the function returns, the original `&mut` is usable again, enabling multiple sequential uses.

```rust
fn takes_mut(s: &mut String) {
    s.push_str(" world");
}

fn reborrow_demo() {
    let mut s = String::from("hello");
    let r: &mut String = &mut s;

    // This works via reborrowing:
    takes_mut(r);  // Implicitly: takes_mut(&mut *r)
    takes_mut(r);  // r is NOT moved, it's reborrowed

    // Reborrowing creates a new &mut that borrows from r
    // Original r is temporarily "frozen" during the reborrow

    r.push_str("!");  // r is usable again after takes_mut returns
}

fn explicit_reborrow() {
    let mut s = String::from("hello");
    let r = &mut s;

    // Explicit reborrow syntax - use block to scope reborrow
    {
        let r2: &mut String = &mut *r;
        r2.push_str(" world");
        // r is frozen while r2 exists
    }  // r2's lifetime ends here

    r.push_str("!");  // r usable again
}

// Usage:
// reborrow_demo();     // prints "hello world world!"
// explicit_reborrow(); // prints "hello world!"
```

### Example: Borrow Splitting

Rust allows simultaneous `&mut` to disjoint struct fields or non-overlapping slice regions. Methods like `split_at_mut` return two mutable slices safely; manual splitting requires unsafe but is sound when slices don't overlap.

```rust
struct Data {
    left: Vec<i32>,
    right: Vec<i32>,
}

impl Data {
    // Returns mutable references to both fields simultaneously
    fn split(&mut self) -> (&mut Vec<i32>, &mut Vec<i32>) {
        (&mut self.left, &mut self.right)
    }
}

// Slice splitting
fn slice_split() {
    let mut arr = [1, 2, 3, 4, 5];

    // split_at_mut returns two non-overlapping mutable slices
    let (left, right) = arr.split_at_mut(2);
    // left: &mut [1, 2], right: &mut [3, 4, 5]

    left[0] = 10;
    right[0] = 30;
    // Both mutations are valid because slices don't overlap
}

// Manual split with unsafe (when safe API isn't available)
fn manual_split<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    assert!(mid <= slice.len());

    let ptr = slice.as_mut_ptr();
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), slice.len() - mid),
        )
    }
}

// Usage:
// let mut data = Data { left: vec![1], right: vec![2] };
// let (l, r) = data.split();  // borrow both fields mutably
// l.push(10); r.push(20);     // modify both simultaneously
```

## Pattern 6: Method Receivers and Self Types

* **Problem**: Choosing between `self`, `&self`, `&mut self`, `Box<Self>`, and `Pin<&mut Self>` affects API ergonomics, ownership semantics, and trait object compatibility. The wrong choice leads to awkward APIs or prevents desired use cases.
* **Solution**: Use `&self` for read-only access, `&mut self` for mutation, `self` for consuming operations. Use `Box<Self>`, `Rc<Self>`, or `Arc<Self>` receivers for smart pointer methods. Use `Pin<&mut Self>` for self-referential types.
* **Why It Matters**: The receiver type is part of the method's contract. It determines whether callers retain ownership, whether the method can be called on trait objects, and how the method interacts with smart pointers and pinning.

### Example: Receiver Type Inference

Method receivers (`self`, `&self`, `&mut self`, `Box<Self>`, `Rc<Self>`, `Pin<&mut Self>`) define ownership semantics. Smart pointer receivers enable calling methods directly on wrapped types; `Pin` receivers enforce immovability for self-referential types.

```rust
use std::rc::Rc;
use std::sync::Arc;
use std::pin::Pin;

struct Widget { name: String }

impl Widget {
    // Each receiver type has specific semantics
    fn ref_method(&self) -> &str { &self.name }
    fn mut_method(&mut self) { self.name.push_str("!"); }
    fn owned_method(self) -> String { self.name }

    // Arbitrary self types (requires #![feature(arbitrary_self_types)]
    // for custom types, but these work in stable):
    fn box_method(self: Box<Self>) -> String { self.name }
    fn rc_method(self: Rc<Self>) -> Rc<Self> { self }
    fn arc_method(self: Arc<Self>) -> Arc<Self> { self }
    fn pin_method(self: Pin<&Self>) -> &str { &self.get_ref().name }
    fn pin_mut_method(self: Pin<&mut Self>) {
        // Must maintain pin invariants
        self.get_mut().name.push_str("!");
    }
}

fn receiver_types() {
    // Each receiver type determines how the method is called
    let w = Widget { name: "w".into() };
    let _ = w.ref_method();      // auto-ref to &Widget

    let boxed = Box::new(Widget { name: "boxed".into() });
    let _ = boxed.box_method();  // consumes Box<Widget>

    let rc = Rc::new(Widget { name: "rc".into() });
    let rc2 = rc.clone();
    let _ = rc.rc_method();      // consumes one Rc handle
    let _ = rc2.ref_method();    // Deref through Rc to call &self method
}

// Usage:
// let w = Widget { name: "test".into() };
// println!("{}", w.ref_method());  // "test"
// let name = w.owned_method();     // consumes w, returns "test"
```

### Example: Trait Object Receivers and Object Safety

Object-safe traits can use `&self`, `&mut self`, or `Box<Self>` receivers. Methods with `self` by value, generic parameters, or returning `Self` break object safety because the vtable can't encode size or monomorphize at runtime.

```rust
trait ObjectSafe {
    // These are allowed in trait objects:
    fn by_ref(&self);
    fn by_mut(&mut self);
    fn by_box(self: Box<Self>);
}

trait NotObjectSafe {
    // NOT allowed in trait objects:
    fn by_value(self);  // Requires knowing size at compile time
    fn generic<T>(&self, t: T);  // Generic methods can't use vtable
    fn returns_self(&self) -> Self;  // Size of Self unknown
}

struct MyType;
impl ObjectSafe for MyType {
    fn by_ref(&self) { println!("by_ref"); }
    fn by_mut(&mut self) { println!("by_mut"); }
    fn by_box(self: Box<Self>) { println!("by_box"); }
}

// Usage:
// let obj: Box<dyn ObjectSafe> = Box::new(MyType);
// obj.by_ref();  // vtable dispatch
// obj.by_box();  // consumes the Box
```

## Pattern 7: Iterator Type Signatures

* **Problem**: Iterator adapters return complex nested types like `Filter<Map<Iter<'a, T>, F1>, F2>`. Understanding the ownership model (`Item = T` vs `Item = &T` vs `Item = &mut T`) is crucial but often unclear from usage.
* **Solution**: Know the three `IntoIterator` implementations for collections: `Vec<T>` yields `T`, `&Vec<T>` yields `&T`, `&mut Vec<T>` yields `&mut T`. Use `impl Iterator<Item = T>` to hide complex return types.
* **Why It Matters**: Iterator types encode ownership at the type level. Misunderstanding them causes "cannot move out of borrowed content" errors. The `impl Trait` feature simplifies signatures but requires understanding lifetime bounds.

### Example: Iterator Trait Family

Collections implement `IntoIterator` three ways: `vec.into_iter()` yields owned `T`, `(&vec).into_iter()` yields `&T`, `(&mut vec).into_iter()` yields `&mut T`. The `Item` type encodes ownership at compile time.

```rust
// The core iterator traits (simplified):
trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}

trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// For Vec<T>, three implementations:
// impl IntoIterator for Vec<T>       -> Item = T
// impl IntoIterator for &Vec<T>      -> Item = &T
// impl IntoIterator for &mut Vec<T>  -> Item = &mut T

fn type_signatures() {
    let vec: Vec<String> = vec!["a".into(), "b".into()];

    // Type of iterator and items:
    let iter: std::vec::IntoIter<String> = vec.into_iter();
    // iter.next() returns Option<String>

    let vec: Vec<String> = vec!["a".into(), "b".into()];
    let iter: std::slice::Iter<'_, String> = vec.iter();
    // iter.next() returns Option<&String>

    let mut vec: Vec<String> = vec!["a".into(), "b".into()];
    let iter: std::slice::IterMut<'_, String> = vec.iter_mut();
    // iter.next() returns Option<&mut String>
}

// Usage:
// for s in vec.iter() { println!("{}", s); }      // borrows
// for s in vec.into_iter() { println!("{}", s); } // consumes
```

### Example: Lifetime Bounds in Iterators

Standard iterators have `Item` fixed at type definition—the item's lifetime is tied to the iterator's lifetime parameter. GAT-based lending iterators allow items with lifetimes tied to each `next()` call, enabling overlapping window iteration.

```rust
// Lending iterator pattern (GAT-based)
trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// Windows iterator: yields overlapping slices
struct Windows<'a, T> {
    slice: &'a [T],
    size: usize,
    pos: usize,
}

impl<'a, T> Iterator for Windows<'a, T> {
    type Item = &'a [T];  // Item lifetime tied to original slice

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + self.size <= self.slice.len() {
            let window = &self.slice[self.pos..self.pos + self.size];
            self.pos += 1;
            Some(window)
        } else {
            None
        }
    }
}

// Usage:
// let data = [1, 2, 3, 4, 5];
// let windows = Windows { slice: &data, size: 3, pos: 0 };
// for w in windows { println!("{:?}", w); }  // [1,2,3], [2,3,4], [3,4,5]
```

### Example: Returning Iterators with Type Inference

`impl Iterator<Item = T>` hides complex adapter types like `Filter<Map<...>>` from the signature. Add lifetime bounds (`+ 'a`) when the iterator borrows from inputs. Named iterator types are needed when implementing traits.

```rust
// Return position impl Trait
fn even_numbers(limit: i32) -> impl Iterator<Item = i32> {
    (0..limit).filter(|n| n % 2 == 0)
}

// With lifetime
fn filter_prefix<'a>(
    strings: &'a [String],
    prefix: &'a str
) -> impl Iterator<Item = &'a String> + 'a {
    strings.iter().filter(move |s| s.starts_with(prefix))
}

// Named iterator type for trait implementations
struct Evens {
    current: i32,
    limit: i32,
}

impl Iterator for Evens {
    type Item = i32;
    fn next(&mut self) -> Option<i32> {
        if self.current < self.limit {
            let val = self.current;
            self.current += 2;
            Some(val)
        } else {
            None
        }
    }
}

// Usage:
// for n in even_numbers(10) { print!("{} ", n); }  // 0 2 4 6 8
// let evens = Evens { current: 0, limit: 6 };
// let v: Vec<_> = evens.collect();  // [0, 2, 4]
```

## Pattern 8: Reference Conversion Trait Hierarchy

* **Problem**: `AsRef`, `AsMut`, `Borrow`, and `BorrowMut` all provide reference conversions, but using the wrong one causes subtle bugs—especially with `HashMap` lookups where hash consistency matters.
* **Solution**: Use `AsRef<T>` for cheap view conversions (e.g., `String` → `Path`). Use `Borrow<T>` when semantic equivalence is required (same `Hash`, `Eq`, `Ord`). Use `Cow` to defer the owned-vs-borrowed decision.
* **Why It Matters**: `Borrow` is required for `HashMap::get` to work with `&str` keys on `HashMap<String, V>`. `AsRef` is more permissive but doesn't guarantee hash equivalence. Choosing correctly enables flexible APIs without runtime overhead.

### Example: AsRef, AsMut, Borrow, BorrowMut

`AsRef` allows broad type conversions (String→Path, String→[u8]). `Borrow` guarantees semantic equivalence—hash, equality, and ordering must match. Use `Borrow` for HashMap keys to enable `&str` lookups on `HashMap<String, V>`.

```rust
use std::borrow::Borrow;

// AsRef: cheap reference conversion
// Use when you want to accept anything that can be viewed as &T
fn print_path<P: AsRef<std::path::Path>>(path: P) {
    println!("{:?}", path.as_ref());
}

// Borrow: semantic equivalence (same Hash, Eq, Ord)
// Use for lookup keys in collections
fn lookup<'a, K, V, Q>(map: &'a std::collections::HashMap<K, V>, key: &Q) -> Option<&'a V>
where
    K: Borrow<Q> + std::hash::Hash + Eq,
    Q: std::hash::Hash + Eq + ?Sized,
{
    map.get(key)
}

// Key distinction:
// - AsRef is for type conversion (String -> Path is valid)
// - Borrow requires semantic equivalence (String::borrow() -> &str has same hash)

fn trait_differences() {
    let s = String::from("hello");

    // AsRef: many conversions
    let _: &str = s.as_ref();
    let _: &[u8] = s.as_ref();
    let _: &std::path::Path = s.as_ref();
    let _: &std::ffi::OsStr = s.as_ref();

    // Borrow: semantic equivalence only
    let _: &str = s.borrow();
    // String doesn't impl Borrow<[u8]> because hash would differ
}

// Usage:
// print_path("file.txt");  // &str -> &Path via AsRef
// let map: HashMap<String, i32> = [("key".into(), 1)].into();
// map.get("key");  // &str lookup on String keys via Borrow
```

### Example: ToOwned and Cow

`ToOwned` creates owned types from borrowed (`&str` → `String`, `&[T]` → `Vec<T>`). `Cow` (clone-on-write) defers allocation—returns borrowed data when no modification needed, allocates only when mutation is required.

```rust
use std::borrow::Cow;

// ToOwned: create owned from borrowed
// Generalization of Clone for borrowed types
fn to_owned_demo() {
    let s: &str = "hello";
    let owned: String = s.to_owned();  // &str -> String

    let slice: &[i32] = &[1, 2, 3];
    let owned: Vec<i32> = slice.to_owned();  // &[T] -> Vec<T>
}

// Cow: clone on write, avoids allocation when possible
fn process_name(name: &str) -> Cow<'_, str> {
    if name.contains(' ') {
        // Must allocate to modify
        Cow::Owned(name.replace(' ', "_"))
    } else {
        // No allocation needed
        Cow::Borrowed(name)
    }
}

fn cow_in_structs() {
    #[derive(Debug)]
    struct Config<'a> {
        name: Cow<'a, str>,
        data: Cow<'a, [u8]>,
    }

    // Can own or borrow flexibly
    let config1 = Config {
        name: Cow::Borrowed("static"),
        data: Cow::Borrowed(&[1, 2, 3]),
    };

    let config2 = Config {
        name: Cow::Owned(String::from("dynamic")),
        data: Cow::Owned(vec![4, 5, 6]),
    };

    // Both have the same type: Config<'_>
}

// Usage:
// let name = process_name("John");       // Cow::Borrowed (no alloc)
// let name = process_name("Jane Doe");   // Cow::Owned (allocates)
```

## Pattern 9: Conversion Naming Conventions

* **Problem**: Inconsistent method naming across libraries makes APIs unpredictable. When does a method clone vs move? When does it allocate?
* **Solution**: Follow Rust's `as_`/`to_`/`into_` convention: `as_` returns a reference (O(1), no allocation), `to_` returns owned (may clone), `into_` consumes self (transfers ownership). Apply this consistently in your own APIs.
* **Why It Matters**: Naming conventions communicate ownership semantics without reading documentation. `into_inner()` clearly consumes the wrapper; `as_slice()` clearly borrows. Consistent naming reduces cognitive load and bugs.

| Prefix | Self | Returns | Cost | Example |
|--------|------|---------|------|---------|
| `as_` | `&self` | `&U` | O(1) | `as_str()`, `as_bytes()` |
| `to_` | `&self` | `U` | O(n) | `to_string()`, `to_vec()` |
| `into_` | `self` | `U` | O(1)* | `into_inner()`, `into_bytes()` |

*`into_` may have O(n) cost for type conversions, but avoids cloning.

```rust
struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    // as_: returns reference, no allocation
    fn as_slice(&self) -> &[u8] { &self.data }
    fn as_mut_slice(&mut self) -> &mut [u8] { &mut self.data }

    // to_: returns owned, may allocate/clone
    fn to_vec(&self) -> Vec<u8> { self.data.clone() }
    fn to_hex(&self) -> String {
        self.data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    // into_: consumes self, returns owned
    fn into_vec(self) -> Vec<u8> { self.data }
    fn into_boxed_slice(self) -> Box<[u8]> { self.data.into_boxed_slice() }
}

// Usage:
// let buf = Buffer { data: vec![1, 2, 3] };
// let slice: &[u8] = buf.as_slice();   // borrows
// let owned: Vec<u8> = buf.into_vec(); // consumes buf
```

### Example: Raw Pointer Conversions

`into_raw` transfers ownership to a raw pointer (caller must eventually reclaim it). `from_raw` reclaims ownership from a raw pointer. `as_ptr`/`as_mut_ptr` borrow as raw pointers without ownership transfer—the original owner remains responsible for cleanup.

```rust
struct Handle {
    ptr: *mut u8,
    len: usize,
}

impl Handle {
    // into_raw: leak memory, caller takes ownership
    fn into_raw(self) -> *mut u8 {
        let ptr = self.ptr;
        std::mem::forget(self);  // Prevent destructor
        ptr
    }

    // from_raw: reclaim ownership from raw pointer
    unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        Handle { ptr, len }
    }

    // as_ptr: borrow as raw pointer (no ownership transfer)
    fn as_ptr(&self) -> *const u8 { self.ptr }
    fn as_mut_ptr(&mut self) -> *mut u8 { self.ptr }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                self.ptr,
                std::alloc::Layout::from_size_align_unchecked(self.len, 1)
            );
        }
    }
}

// Usage:
// let raw = handle.into_raw();           // transfer ownership out
// let handle = Handle::from_raw(raw, len); // reclaim ownership
// let ptr = handle.as_ptr();             // borrow as raw ptr (no transfer)
```

## Pattern 10: Pin and Self-Referential Types

* **Problem**: Rust's move semantics invalidate internal pointers. A struct containing a pointer to its own field becomes unsound if moved. This prevents self-referential types and complicates async/await implementation.
* **Solution**: Use `Pin<P>` to guarantee the pointee won't move. Implement `!Unpin` (via `PhantomPinned`) to opt out of the default move-safe assumption. Use pin projection to safely access fields of pinned types.
* **Why It Matters**: `Pin` is foundational for async Rust—`Future`s often contain self-references across await points. Understanding `Pin` is essential for writing custom futures, implementing intrusive data structures, and working with FFI that requires stable addresses.

```rust
use std::pin::Pin;
use std::marker::PhantomPinned;

struct SelfReferential {
    data: String,
    // This would point into data - requires Pin
    ptr: *const String,
    _pin: PhantomPinned,  // Opts out of Unpin
}

impl SelfReferential {
    fn new(data: String) -> Pin<Box<Self>> {
        let mut boxed = Box::new(SelfReferential {
            data,
            ptr: std::ptr::null(),
            _pin: PhantomPinned,
        });

        // Safe because we're setting up the self-reference
        // before returning the Pin
        let ptr: *const String = &boxed.data;
        boxed.ptr = ptr;

        // SAFETY: we never move the data after this
        unsafe { Pin::new_unchecked(boxed) }
    }

    fn data(self: Pin<&Self>) -> &str {
        &self.get_ref().data
    }
}

// Pin projections for field access
impl SelfReferential {
    // Pin projection: Pin<&Self> -> Pin<&Field>
    // Only valid if Field: Unpin, otherwise must be unsafe
    fn project_data(self: Pin<&Self>) -> &String {
        // String: Unpin, so this is safe
        &self.get_ref().data
    }
}

// Usage:
// let pinned = SelfReferential::new("hello".into());
// println!("{}", pinned.as_ref().data());  // access through Pin
// // let moved = *pinned;  // ERROR: can't move out of Pin<!Unpin>
```

These patterns form the foundation of Rust's zero-cost abstractions around references. The type system enforces borrowing rules at compile time, while traits like `Deref` and `AsRef` enable ergonomic APIs without runtime overhead.
