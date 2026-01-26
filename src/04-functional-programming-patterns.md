# Functional Programming Patterns

Rust's functional programming features are deeply integrated with its ownership system, creating unique patterns you won't find in other languages. This chapter explores how closures interact with the borrow checker, why every closure has a unique type, how the compiler optimizes functional code to match imperative performance, and the subtle differences between `fn`, `Fn`, `FnMut`, and `FnOnce`.

## Pattern 1: Functions and the Type System

*   **Problem**: Rust functions have behaviors that surprise programmers coming from other languages: every function has a unique type, generic functions don't have a single address, and the return type affects control flow in unexpected ways.
*   **Solution**: Understand function item types vs function pointer types, how monomorphization affects function identity, and how the never type (`!`) enables powerful patterns.
*   **Why It Matters**: These details explain why generic functions can't be stored in arrays, why `panic!` works in any match arm, and why function items are zero-cost. Mastering these concepts unlocks advanced patterns like type-level dispatch tables and compile-time function composition.

### Example: Function Item Types vs Function Pointers

Every function has a unique zero-sized type that differs from its function pointer type. Function items like `add` carry no runtime data—the address is baked in at compile time. Coerce to `fn(args) -> ret` pointer type when you need to store different functions in a collection.

```rust
// Every function has a unique, zero-sized type (function item type)
fn add(a: i32, b: i32) -> i32 { a + b }
fn sub(a: i32, b: i32) -> i32 { a - b }

fn demonstrate_function_types() {
    // These have DIFFERENT types, even though signatures match
    let f1 = add;  // type: fn add(i32, i32) -> i32 (zero-sized)
    let f2 = sub;  // type: fn sub(i32, i32) -> i32 (zero-sized)

    // This won't compile - different types!
    // let funcs = [add, sub];

    // Coerce to function pointer type to unify
    let funcs: [fn(i32, i32) -> i32; 2] = [add, sub];

    // Or coerce explicitly
    let f1: fn(i32, i32) -> i32 = add;
    let f2: fn(i32, i32) -> i32 = sub;

    // Size difference:
    // 0 bytes (function item is zero-sized)
    println!("Size of add (item): {}", std::mem::size_of_val(&add));
    // 8 bytes (pointer-sized)
    println!("Fn pointer: {}", std::mem::size_of::<fn() -> i32>());

    // Function items are zero-sized, so this is free:
    struct Callback<F> {
        f: F,  // Zero bytes if F is a function item
    }
}

// Store different functions by coercing to fn pointer type.
// Function items are zero-sized; fn pointers are 8 bytes.
let funcs: [fn(i32, i32) -> i32; 2] = [add, sub];
println!("{}", funcs[0](5, 3)); // 8
println!("{}", funcs[1](5, 3)); // 2
```

**Why zero-sized?** Function item types contain no runtime data—the function address is known at compile time and embedded directly at call sites. This enables perfect inlining and zero overhead.

### Example: Monomorphization and Function Identity

Generic functions don't exist at runtime—only their monomorphized versions do. `identity::<i32>` and `identity::<String>` are separate functions with different addresses. You cannot create a pointer to a generic function, only to specific instantiations.

```rust
fn identity<T>(x: T) -> T { x }

fn monomorphization_surprise() {
    // Each monomorphization is a DIFFERENT function
    let int_id: fn(i32) -> i32 = identity;
    let str_id: fn(&str) -> &str = identity;

    // They have different addresses!
    println!("identity::<i32> at {:p}", int_id as *const ());
    println!("identity::<&str> at {:p}", str_id as *const ());

    // This is why you can't do this:
    // let generic_ptr: fn<T>(T) -> T = identity;  // No such type!

    // Generic fns don't exist at runtime—only monomorphizations
}

// Implications for function pointer tables:
trait Handler {
    fn handle(&self, input: &str) -> String;
}

// Can't store generic functions, but can store trait object methods
fn build_dispatch_table() -> Vec<fn(&str) -> String> {
    // Must use concrete types
    fn handler_a(s: &str) -> String { format!("A: {}", s) }
    fn handler_b(s: &str) -> String { format!("B: {}", s) }
    vec![handler_a, handler_b]
}

// Each monomorphization is a separate fn with its own address.
// Can't take pointer to generic fn—only to instantiations.
let int_id: fn(i32) -> i32 = identity;
let u8_id: fn(u8) -> u8 = identity;
println!("{}, {}", int_id(42), u8_id(255)); // 42, 255
```

### Example: The Never Type and Control Flow

The never type `!` represents computations that don't return—`panic!`, `return`, `break`, infinite loops. Since `!` has no values, it can coerce to any type. This is why match arms with `panic!` work alongside arms returning concrete types.

```rust
// The never type `!` is the bottom type - it has no values
// Any expression of type `!` can coerce to any other type

fn the_never_type() {
    // All of these expressions have type `!`:
    // - panic!()
    // - return
    // - break
    // - continue
    // - loop {} (infinite loop without break)
    // - std::process::exit()

    // This is why match arms can have different "types":
    let value: i32 = match Some(42) {
        Some(n) => n,           // type: i32
        None => panic!("oops"), // type: !, coerces to i32
    };

    // And why this compiles:
    let result: String = if true {
        "hello".to_string()
    } else {
        return;  // type: !, coerces to String
    };
}

// Practical use: functions that handle "impossible" cases
fn unwrap_infallible<T>(
    result: Result<T, std::convert::Infallible>,
) -> T {
    match result {
        Ok(value) => value,
        Err(never) => match never {}, // Empty match on uninhabited
    }
}

// The `!` type in return position signals "never returns normally"
fn diverging_function() -> ! {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// Useful for builder patterns with required fields
struct Builder {
    required: Option<String>,
}

impl Builder {
    fn required(mut self, value: &str) -> Self {
        self.required = Some(value.to_string());
        self
    }

    fn build(self) -> Result<Config, &'static str> {
        Ok(Config {
            required: self.required.ok_or("required field")?,
        })
    }
}

struct Config {
    required: String,
}

// The never type (!) coerces to any type, enabling mixed match.
// Empty match on uninhabited types proves a case is impossible.
let value: i32 = match Some(42) {
    Some(n) => n,
    None => panic!("oops"), // ! coerces to i32
};
```

### Example: Const Functions and Compile-Time Evaluation

`const fn` functions execute at compile time when called in const contexts. Results are embedded directly in the binary with zero runtime cost. Use them for lookup tables, configuration values, and compile-time validation.

```rust
// const fn: functions that can run at compile time
const fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

// Computed at compile time, embedded in binary
const FACTORIAL_10: u64 = factorial(10);

// const fn restrictions (relaxed over time):
// - No heap allocation (no Vec, String, Box)
// - Limited control flow (loops added in 1.46, match in 1.46)
// - No trait bounds (some added in recent versions)
// - No floating point (stabilized in 1.82)

const fn const_example() -> usize {
    let mut sum = 0;
    let mut i = 0;
    while i < 10 {
        sum += i;
        i += 1;
    }
    sum
}

// Powerful pattern: compile-time lookup tables
const fn generate_lookup_table() -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = (i as u8).count_ones() as u8;
        i += 1;
    }
    table
}

static POPCOUNT_TABLE: [u8; 256] = generate_lookup_table();

fn count_bits(byte: u8) -> u8 {
    POPCOUNT_TABLE[byte as usize]
}

// const fn runs at compile time when called in const contexts.
// The result is embedded directly in the binary with zero cost.
const FACT_5: u64 = factorial(5); // Computed at compile time
const TABLE: [u8; 256] = generate_lookup_table(); // Static table
```

## Pattern 2: Closure Internals

*   **Problem**: Closures seem magical—they capture variables, have anonymous types, and somehow implement traits. Understanding their true nature is essential for advanced usage.
*   **Solution**: Recognize that closures are compiler-generated structs that implement `Fn` traits. Each closure literal creates a unique anonymous type containing captured variables as fields.
*   **Why It Matters**: This explains closure sizes, why you can't name closure types, how capture modes work, and when closures can be `Copy` or `Clone`.

### Example: What Closures Really Are

Closures are compiler-generated structs containing captured variables as fields. The struct implements `Fn`, `FnMut`, or `FnOnce` depending on how captures are used. This desugaring explains closure sizes and why each closure has a unique anonymous type.

```rust
fn closure_desugaring() {
    let x = 10;
    let y = String::from("hello");

    // This closure:
    let closure = |a: i32| a + x;

    // Is roughly equivalent to this struct:
    // struct __closure_1<'a> {
    //     x: &'a i32,  // captured by reference
    // }
    // impl<'a> Fn(i32) -> i32 for __closure_1<'a> {
    //     fn call(&self, a: i32) -> i32 {
    //         a + *self.x
    //     }
    // }

    // Proof - check the size (8 bytes = one reference)
    println!("Closure size: {}", std::mem::size_of_val(&closure));

    // A closure capturing more (32 bytes = String + &i32)
    let closure2 = |a: i32| format!("{}: {}", y, a + x);
    println!("Closure2 size: {}", std::mem::size_of_val(&closure2));

    // move closure: captures by value
    let closure3 = move |a: i32| a + x;
    // struct __closure_3 { x: i32 } // captured by value (copied)
    // 4 bytes = i32 size
    println!("Move closure: {}", std::mem::size_of_val(&closure3));
}

// Closure size reflects captures—refs are 8 bytes, values vary
// By-reference closure: 8 bytes; move closure with i32: 4 bytes
let by_ref = |a: i32| a + x; // Captures &x (8 bytes)
let by_val = move |a: i32| a + x; // Captures x by value (4 bytes)
```

### Example: Each Closure Has a Unique Type

Two closures with identical code have different types—each closure literal creates a new anonymous type. To store multiple closures together, use `Box<dyn Fn>`, function pointers (non-capturing only), or an enum wrapper. This uniqueness enables monomorphization and zero-cost abstraction.

```rust
fn unique_closure_types() {
    let a = |x: i32| x + 1;
    let b = |x: i32| x + 1;  // Identical code, but DIFFERENT TYPE

    // This won't compile:
    // let closures = [a, b];  // Error: mismatched types

    // Even with no captures, they're different:
    fn takes_closure<F: Fn(i32) -> i32>(f: F) {
        println!("Result: {}", f(10));
    }

    takes_closure(a);  // Monomorphized for type of `a`
    takes_closure(b);  // Monomorphized AGAIN for type of `b`

    // To store multiple closures, you need:
    // 1. Box<dyn Fn> - heap allocation + vtable
    let boxed: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|x| x + 1),
        Box::new(|x| x * 2),
    ];

    // 2. Function pointers (only for non-capturing closures)
    let fn_ptrs: Vec<fn(i32) -> i32> = vec![
        |x| x + 1,  // Coerces because no captures
        |x| x * 2,
    ];

    // 3. Enum wrapper
    enum Op {
        Add(i32),
        Mul(i32),
    }
    impl Op {
        fn apply(&self, x: i32) -> i32 {
            match self {
                Op::Add(n) => x + n,
                Op::Mul(n) => x * n,
            }
        }
    }
}

// Use Box<dyn Fn> to store different closures together.
// Non-capturing closures can coerce to fn pointers for storage.
let ops: Vec<Box<dyn Fn(i32) -> i32>> =
    vec![Box::new(|x| x + 1), Box::new(|x| x * 2)];
let fn_ptrs: Vec<fn(i32) -> i32> = vec![|x| x + 1, |x| x * 2];
```

### Example: When Closures Are Copy and Clone

A closure is `Copy` if it captures only by reference, or captures by value where all values are `Copy`. Non-`Copy` captures like `String` make the closure non-`Copy`. Clone follows similar rules—the closure is `Clone` if all its captures are `Clone`.

```rust
fn closure_copy_clone() {
    // A closure is Copy if:
    // 1. It captures by reference only, OR
    // 2. It captures by value AND all captured values are Copy

    let x = 10;  // i32 is Copy

    // Captures &i32 - closure is Copy
    let by_ref = || x + 1;
    let copy1 = by_ref;
    let copy2 = by_ref;  // OK, closure was copied

    // move + Copy capture - closure is Copy
    let by_val = move || x + 1;
    let copy3 = by_val;
    let copy4 = by_val;  // OK, x was copied, closure is Copy

    // Non-Copy capture - closure is NOT Copy
    let s = String::from("hello");
    let not_copy = move || s.len();  // s moved in
    let moved = not_copy;
    // let error = not_copy;  // Error: not_copy was moved

    // Clone works if all captures are Clone
    let s = String::from("hello");
    let cloneable = move || s.clone();
    // let c1 = cloneable.clone();  // Works if closure is Clone

    // Practical implication: thread spawning
    let data = vec![1, 2, 3];  // Vec is not Copy
    let handle = std::thread::spawn(move || {
        // `data` moved here - can't use original
        data.iter().sum::<i32>()
    });
}

// Closures capturing only Copy types (or refs) are Copy themselves.
// A closure capturing String by move is not Copy—it gets moved.
let x = 10;
let by_ref = || x + 1; // Copy: captures &i32
let c1 = by_ref;
let c2 = by_ref; // OK, closure was copied
```

### Example: Closure Type Inference Pitfalls

Closure parameter and return types are inferred from the first use—once fixed, they can't change. This means `|x| x` called with `i32` becomes locked to `i32`. Add explicit type annotations when inference causes issues or when you need clarity.

```rust
fn inference_pitfalls() {
    // Closure types are inferred from FIRST USE
    let closure = |x| x;  // Type not yet determined

    let _: i32 = closure(5);  // Now it's fn(i32) -> i32
    // let _: &str = closure("hello");  // Error! Type already fixed

    // This is why you sometimes need annotations:
    let explicit = |x: &str| -> String { x.to_uppercase() };

    // Or the turbofish when collecting:
    let strings: Vec<_> = ["a", "b", "c"]
        .iter()
        .map(|s| s.to_uppercase())
        .collect();  // collect::<Vec<String>>() also works

    // Generic closures don't exist - but you can fake it:
    fn make_identity<T>() -> impl Fn(T) -> T {
        |x| x
    }
    let int_id = make_identity::<i32>();
    let str_id = make_identity::<String>();
}

// Higher-ranked trait bounds for truly generic closures:
fn apply_to_ref<F>(f: F, s: &str) -> usize
where
    F: for<'a> Fn(&'a str) -> usize,  // Works with ANY lifetime
{
    f(s)
}

// Closure types inferred from first use—once fixed, can't change.
// Use a factory function to create closures for each type.
let closure = |x| x;
let _: i32 = closure(42); // Type locked to fn(i32) -> i32
fn make_identity<T>() -> impl Fn(T) -> T { |x| x }
let int_id = make_identity::<i32>();
```

## Pattern 3: Capture Semantics Deep Dive

*   **Problem**: The rules for how closures capture variables are subtle. Partial captures, disjoint field borrows, and the interaction with `move` create edge cases.
*   **Solution**: Understand that Rust 2021 captures individual fields, not whole structs. Learn when captures are promoted from `&T` to `&mut T` to `T`.
*   **Why It Matters**: Mastering capture semantics prevents borrow checker battles and enables more precise, efficient closures.

### Example: Field-Level Capture (Rust 2021)

Rust 2021 captures individual struct fields rather than the entire struct. If a closure only uses `data.name`, the other fields remain accessible outside. This enables more precise borrows and fewer borrow checker conflicts.

```rust
fn field_capture() {
    struct Data {
        name: String,
        value: i32,
        items: Vec<u8>,
    }

    let data = Data {
        name: "test".into(),
        value: 42,
        items: vec![1, 2, 3],
    };

    // Rust 2021: captures only `data.name`, not entire `data`
    let closure = || println!("{}", data.name);

    // So this still works:
    println!("Value: {}", data.value);  // `data.value` not captured
    println!("Items: {:?}", data.items); // not captured

    // But this closure captures all used fields:
    let closure2 = || {
        println!("{}: {}", data.name, data.value);
    };
    // data.name and data.value are borrowed, but data.items is free

    // Rust 2018 would capture entire `data` struct
    // You can force this with a dummy:
    let closure_whole = || {
        let _ = &data;  // Force capture of entire struct
        println!("{}", data.name);
    };
}

// Rust 2021 captures individual fields, not the whole struct.
// A closure using data.name leaves data.value accessible outside.
let get_name = || data.name.len(); // Only captures data.name
println!("{}", data.value); // Still accessible—not captured
```

### Example: Capture Mode Inference

The compiler infers the least restrictive capture mode: `&T` if only read, `&mut T` if mutated, or `T` (move) if consumed. This determines whether the closure is `Fn`, `FnMut`, or `FnOnce`. The `move` keyword forces by-value capture but doesn't change the trait implemented.

```rust
fn capture_mode_inference() {
    let mut x = 5;
    let s = String::from("hello");

    // 1. Just reading: capture by &T
    let read_only = || println!("{} {}", x, s);
    // Equivalent to: captures (&x, &s)

    // 2. Mutating: capture by &mut T
    let mut mutating = || {
        x += 1;  // Need &mut x
        println!("{}", x);
    };
    mutating();
    // x is mutably borrowed until `mutating` is dropped

    // 3. Moving out: capture by T
    let consuming = || {
        drop(s);  // Takes ownership of s
    };
    // s is moved into the closure
    // consuming();  // Would consume s
    // println!("{}", s);  // Error: s moved

    // The compiler picks the LEAST restrictive mode that works:
    // - Prefer &T over &mut T
    // - Prefer &mut T over T
    // - Unless `move` keyword is used

    // Common gotcha: calling a method might require &mut
    let mut vec = vec![1, 2, 3];
    let pushing = || vec.push(4);  // Captures &mut vec
    // pushing();
    // println!("{:?}", vec);  // OK after `pushing` is done
}

// Compiler infers capture: &T for reads, &mut T for mut, T for move
// Read-only = Fn; mutating = FnMut; consuming = FnOnce
let read = || x + 1; // Fn: captures &x
let mut inc = || { count += 1; count }; // FnMut: captures &mut
let consume = || drop(s); // FnOnce: takes ownership of s
```

### Example: Move Closures and Partial Moves

The `move` keyword forces all captures by value—non-`Copy` types are moved, `Copy` types are copied. In Rust 2021, you can partially move struct fields into closures. Destructure first (`let Pair { a, b } = pair;`) to control which fields move.

```rust
fn move_semantics() {
    let s1 = String::from("hello");
    let s2 = String::from("world");

    // `move` forces ALL captures by value
    let move_all = move || {
        println!("{} {}", s1, s2);
    };
    // Both s1 and s2 are moved into closure
    // println!("{}", s1);  // Error: moved

    // But with Copy types, `move` copies:
    let x = 42;
    let move_copy = move || x + 1;
    println!("{}", x);  // OK! x was copied, not moved

    // Partial move in closure (Rust 2021):
    struct Pair {
        a: String,
        b: String,
    }
    let pair = Pair {
        a: "hello".into(),
        b: "world".into(),
    };

    // Only moves pair.a
    let partial = || drop(pair.a);
    // pair.a is moved, but pair.b is still accessible... kind of
    // println!("{}", pair.b);  // Depends on edition

    // To move specific fields:
    let Pair { a, b } = pair;
    let uses_a = move || println!("{}", a);
    println!("{}", b);  // b is separate
}

// Usage: `move` copies Copy types and moves non-Copy types.
// Destructure before closure to control which fields move into it.
let x = 42;
let move_copy = move || x + 1;
println!("{}", x); // OK: x was copied, not moved
```

### Example: The `move` + Borrow Pattern

Threads require `'static` data, but you often want to keep using data after spawning. Clone data for independent copies, or use `Arc` for shared ownership across threads. Convert borrowed data to owned (`reference.to_string()`) before moving into thread closures.

```rust
use std::thread;

fn move_and_borrow() {
    let data = vec![1, 2, 3, 4, 5];

    // Problem: thread needs owned data, but we want data after
    // let handle = thread::spawn(|| {
    //     data.iter().sum::<i32>()  // Error: lifetime
    // });

    // Solution 1: Clone
    let data_clone = data.clone();
    let handle = thread::spawn(move || {
        data_clone.iter().sum::<i32>()
    });
    println!("Original: {:?}", data);  // OK

    // Solution 2: Arc for shared ownership
    use std::sync::Arc;
    let shared = Arc::new(data);
    let shared_clone = Arc::clone(&shared);
    let handle2 = thread::spawn(move || {
        shared_clone.iter().sum::<i32>()
    });
    println!("Shared: {:?}", shared);

    // Common pattern: move reference into closure
    let local = String::from("local");
    let reference = &local;
    // Can't move `reference` to thread - it's a borrow
    // But can move owned data derived from it:
    let owned = reference.to_string();
    let handle3 = thread::spawn(move || {
        println!("{}", owned);
    });

    handle.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
}

// Clone data before spawning threads to keep using the original.
// Use Arc for shared ownership across threads without full cloning.
let data_clone = data.clone();
let handle = thread::spawn(move || data_clone.iter().sum::<i32>());
println!("{:?}", data); // Original still accessible
```

## Pattern 4: Function Pointers vs Fn Traits

*   **Problem**: The distinction between `fn(Args) -> Ret` and `Fn(Args) -> Ret` is confusing. When should you use each? What are the performance implications?
*   **Solution**: Use `fn` for C FFI and when you specifically need a function pointer. Use `impl Fn`/`&dyn Fn` for everything else. Understand that `fn` is a subset of `Fn`.
*   **Why It Matters**: Wrong choice leads to unnecessary heap allocations, inability to use closures, or loss of inlining optimization.

### Example: The Complete Picture

Function items are zero-sized and unique; function pointers are 8 bytes and unify different functions. Generic `impl Fn` gives monomorphization and inlining; `dyn Fn` gives runtime polymorphism via vtable. Choose based on whether you need static dispatch or dynamic dispatch.

```rust
fn fn_landscape() {
    fn regular(x: i32) -> i32 { x + 1 }

    // 1. Function item type (unique, zero-sized)
    let item = regular;  // type: fn regular(i32) -> i32
    println!("Item size: {}", std::mem::size_of_val(&item));  // 0

    // 2. Function pointer (concrete type, 8 bytes)
    let ptr: fn(i32) -> i32 = regular;
    println!("Ptr size: {}", std::mem::size_of_val(&ptr));  // 8

    // 3. Generic over Fn trait (monomorphized, zero-cost)
    fn take_generic<F: Fn(i32) -> i32>(f: F) -> i32 { f(10) }
    take_generic(regular);  // F = fn regular type, size 0
    take_generic(ptr);      // F = fn(i32) -> i32, size 8
    take_generic(|x| x + 1); // F = closure type, varies

    // 4. Trait object (dynamic dispatch, pointer + vtable)
    let dyn_ref: &dyn Fn(i32) -> i32 = &regular;
    println!("&dyn Fn: {}", std::mem::size_of_val(&dyn_ref)); // 16

    // 5. Boxed trait object (heap allocated)
    let boxed: Box<dyn Fn(i32) -> i32> = Box::new(regular);
    println!("Boxed: {}", std::mem::size_of_val(&boxed)); // 16
}

// Performance hierarchy (best to worst):
// 1. Generic (impl Fn / F: Fn) - inlined, zero-cost
// 2. Function pointer (fn) - indirect call, no inlining
// 3. &dyn Fn - vtable lookup + indirect call
// 4. Box<dyn Fn> - heap allocation + vtable lookup

// Function items = 0 bytes, fn pointers = 8 bytes, &dyn Fn = 16.
// All callable types work similarly but have different performance.
let item = regular; // Zero-sized function item
let ptr: fn(i32) -> i32 = regular; // 8-byte pointer
let dyn_ref: &dyn Fn(i32) -> i32 = &regular; // 16-byte fat pointer
```

### Example: When Each Type Is Appropriate

Use `fn` pointers for C FFI callbacks and static dispatch tables with non-capturing functions. Use `impl Fn` generics for best performance when callers provide the function. Use `Box<dyn Fn>` when storing closures with captures or building event emitters.

```rust
// Use `fn` for: C FFI, dispatch tables, no captures needed

// C FFI callback
extern "C" {
    fn register_callback(cb: extern "C" fn(i32) -> i32);
}

// Dispatch table
static HANDLERS: [fn(&str) -> String; 3] = [
    |s| s.to_uppercase(),  // Coerces because no capture
    |s| s.to_lowercase(),
    |s| s.chars().rev().collect(),
];

// Use generic `impl Fn` for: best performance, caller decides type

fn process_all<F: Fn(i32) -> i32>(items: &[i32], f: F) -> Vec<i32> {
    items.iter().map(|&x| f(x)).collect()
}

// Use `&dyn Fn` for: heterogeneous callbacks, type erasure no heap

fn with_callbacks(callbacks: &[&dyn Fn(i32)]) {
    for (i, cb) in callbacks.iter().enumerate() {
        cb(i as i32);
    }
}

// Use `Box<dyn Fn>` for: storing closures with captures, returns

struct EventEmitter {
    listeners: Vec<Box<dyn Fn(&str)>>,
}

impl EventEmitter {
    fn on(&mut self, listener: impl Fn(&str) + 'static) {
        self.listeners.push(Box::new(listener));
    }

    fn emit(&self, event: &str) {
        for listener in &self.listeners {
            listener(event);
        }
    }
}

// fn ptrs for dispatch; impl Fn for perf; Box<dyn Fn> for storage
static HANDLERS: [fn(i32) -> i32; 2] = [|x| x + 1, |x| x * 2];
fn process<F: Fn(i32) -> i32>(items: &[i32], f: F) -> Vec<i32> {
    items.iter().map(|&x| f(x)).collect()
}
```

### Example: Coercion Rules

Function items and non-capturing closures coerce to `fn` pointers automatically. Capturing closures cannot become `fn` pointers—they need `impl Fn` or `dyn Fn`. Function pointers implement all `Fn` traits, so they work wherever closures are accepted.

```rust
fn coercion_rules() {
    fn named(x: i32) -> i32 { x }

    // Function item -> function pointer: always OK
    let ptr: fn(i32) -> i32 = named;

    // Non-capturing closure -> function pointer: OK
    let closure_ptr: fn(i32) -> i32 = |x| x + 1;

    // Capturing closure -> function pointer: ERROR
    let y = 10;
    // let bad: fn(i32) -> i32 = |x| x + y;  // Error!

    // Any callable -> impl Fn: OK (via generics)
    fn take_fn(f: impl Fn(i32) -> i32) {}
    take_fn(named);
    take_fn(|x| x + 1);
    take_fn(|x| x + y);  // Capturing closure: OK here

    // Any callable -> &dyn Fn: OK (with borrow)
    let dyn_ref: &dyn Fn(i32) -> i32 = &named;
    let dyn_ref2: &dyn Fn(i32) -> i32 = &|x| x + y;

    // Function pointer -> Fn: automatically via Fn impl
    fn take_fn_trait<F: Fn()>(f: F) { f() }
    let ptr: fn() = || println!("hi");
    take_fn_trait(ptr);  // fn implements Fn
}

// Non-capturing closures coerce to fn ptrs; capturing need impl Fn
// Function pointers implement all Fn traits, work where closures do
let ptr: fn(i32) -> i32 = named; // Item -> fn pointer: OK
let closure_ptr: fn(i32) -> i32 = |x| x * 2; // Non-capturing: OK
```

## Pattern 5: Higher-Order Functions and Type Inference

*   **Problem**: Writing functions that take or return functions requires careful handling of generic bounds, lifetimes, and type inference.
*   **Solution**: Use `impl Fn` in argument position for simplicity, explicit generics for flexibility, and understand how return position `impl Trait` works.
*   **Why It Matters**: Higher-order functions are the backbone of functional APIs. Getting the types right enables composable, reusable code.

### Example: Accepting Functions

Three approaches: `impl Fn` for simplicity, generics `<F: Fn>` for flexibility, and `&dyn Fn` for dynamic dispatch. Use generics when the same function type appears multiple times in a signature. Use trait objects for recursive or self-referential function types.

```rust
// Three ways to accept a function:

// 1. impl Trait (simplest, one concrete type)
fn apply_v1(value: i32, f: impl Fn(i32) -> i32) -> i32 {
    f(value)
}

// 2. Generic (most flexible, can specify bounds)
fn apply_v2<F>(value: i32, f: F) -> i32
where
    F: Fn(i32) -> i32,
{
    f(value)
}

// 3. Trait object (dynamic dispatch)
fn apply_v3(value: i32, f: &dyn Fn(i32) -> i32) -> i32 {
    f(value)
}

// When does it matter?

// Multiple uses of same type - need generic:
fn apply_both<F: Fn(i32) -> i32>(
    a: i32, b: i32, f: F
) -> (i32, i32) {
    (f(a), f(b)) // Same F used twice
}

// Different fn types - need separate generics or trait objects
fn apply_two<F, G>(value: i32, f: F, g: G) -> i32
where
    F: Fn(i32) -> i32,
    G: Fn(i32) -> i32,
{
    g(f(value))
}

// Recursive or self-referential - need trait object
// Complex nested fn types are easier with trait objects

// Use impl Fn for simple cases; generics when type appears 2x
// Generics enable composition: apply_both(a, b, f) reuses f
fn apply1(v: i32, f: impl Fn(i32) -> i32) -> i32 { f(v) }
fn apply2<F: Fn(i32) -> i32>(a: i32, b: i32, f: F) -> (i32, i32) {
    (f(a), f(b))
}
```

### Example: Returning Functions

Use `impl Fn` to return closures—the return type is opaque but fixed to one concrete type. For conditional returns of different closure types, use `Box<dyn Fn>` or an enum wrapper. The enum approach avoids heap allocation but requires manual dispatch.

```rust
// Return closure with impl Trait
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

// Return type is opaque but FIXED - can't return different closures
// fn broken(condition: bool) -> impl Fn(i32) -> i32 {
//     if condition {
//         |x| x + 1  // Type A
//     } else {
//         |x| x * 2  // Type B - ERROR: different types!
//     }
// }

// Solutions for conditional returns:

// 1. Box<dyn Fn> - heap allocation
fn make_op_boxed(multiply: bool) -> Box<dyn Fn(i32) -> i32> {
    if multiply {
        Box::new(|x| x * 2)
    } else {
        Box::new(|x| x + 2)
    }
}

// 2. Enum wrapper - no allocation
enum MathOp {
    Add(i32),
    Mul(i32),
}

impl MathOp {
    fn apply(&self, x: i32) -> i32 {
        match self {
            MathOp::Add(n) => x + n,
            MathOp::Mul(n) => x * n,
        }
    }
}

fn make_op_enum(multiply: bool, n: i32) -> MathOp {
    if multiply {
        MathOp::Mul(n)
    } else {
        MathOp::Add(n)
    }
}

// 3. Same closure with different captures
fn make_op_same(multiply: bool, n: i32) -> impl Fn(i32) -> i32 {
    move |x| if multiply { x * n } else { x + n }
}

// Return impl Fn for single closure; Box<dyn Fn> for conditionals.
// One closure with conditional logic avoids Box for differing types
fn make_adder(n: i32) -> impl Fn(i32) -> i32 { move |x| x + n }
let add_5 = make_adder(5);
println!("{}", add_5(10)); // 15
```

### Example: Higher-Order Lifetimes

Closures that borrow must include the lifetime in the return type: `impl Fn() + 'a`. Higher-ranked trait bounds (`for<'a>`) let functions accept closures that work with any lifetime. Use HRTB when the closure receives references created inside your function.

```rust
// Functions returning references need careful lifetime handling

// Simple case: input lifetime flows to output
fn first_that<'a, F>(items: &'a [i32], pred: F) -> Option<&'a i32>
where
    F: Fn(&i32) -> bool,
{
    items.iter().find(|x| pred(x))
}

// Returning closure that borrows - needs explicit lifetime
fn make_checker<'a>(t: &'a i32) -> impl Fn(i32) -> bool + 'a {
    move |x| x > *t
}

// Higher-ranked trait bounds (HRTB) for generic lifetimes
fn for_each_ref<F>(items: &[String], f: F)
where
    F: for<'a> Fn(&'a str),  // F works with ANY lifetime
{
    for item in items {
        f(item);  // Each call might have different lifetime
    }
}

// Without HRTB, this wouldn't work:
fn hrtb_example() {
    let items = vec!["hello".to_string(), "world".to_string()];

    // The closure must work with references of ANY lifetime
    for_each_ref(&items, |s| println!("{}", s));

    // Why HRTB? Because each `s` has a different lifetime
    // bound to each iteration of the loop
}

// Add lifetime bounds to returned closures: impl Fn() + 'a.
// Use HRTB (for<'a>) when closure must work with any lifetime.
fn make_check<'a>(t: &'a i32) -> impl Fn(i32) -> bool + 'a {
    move |x| x > *t
}
```

## Pattern 6: Iterator Internals and Optimization

*   **Problem**: Iterator chains look elegant but seem like they'd be slow—creating intermediate objects, calling methods repeatedly. How can they be zero-cost?
*   **Solution**: Understand that iterators are lazy state machines that get inlined and optimized away. The compiler often produces code identical to hand-written loops.
*   **Why It Matters**: Knowing iterators are zero-cost encourages their use. Understanding when they're NOT (e.g., with `dyn`) helps avoid performance pitfalls.

### Example: Iterator State Machines

Iterator chains create nested types like `Filter<Map<Iter<...>>>` but allocate nothing until collected. Each iterator is a tiny state machine whose `next()` calls chain together. LLVM inlines everything into a single loop—iterator code compiles to identical assembly as hand-written loops.

```rust
fn iterator_internals() {
    let v = vec![1, 2, 3, 4, 5];

    // This chain:
    let result: Vec<i32> = v.iter()
        .map(|x| x * 2)
        .filter(|x| x > &4)
        .collect();

    // Creates a nested type like:
    // Filter<Map<Iter<'_, i32>, [closure]>, [closure]>
    // But it's ZERO allocation until .collect()

    // Each iterator is a small state machine:
    // struct Map<I, F> { iter: I, f: F }
    // struct Filter<I, P> { iter: I, predicate: P }

    // .next() calls chain together:
    // filter.next() calls map.next() calls iter.next()

    // But LLVM inlines all of this into a single loop!
}

// Proof: check the assembly
fn sum_even_doubled(data: &[i32]) -> i32 {
    data.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .sum()
}

fn sum_even_doubled_imperative(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        if x % 2 == 0 {
            sum += x * 2;
        }
    }
    sum
}
// These compile to IDENTICAL assembly with optimizations on!

// Iterator chains compile to same assembly as hand-written loops.
// Filter-map-sum becomes a single optimized loop, no allocations.
let res: Vec<i32> =
    v.iter().map(|x| x * 2).filter(|x| *x > 4).collect();
let sum: i32 =
    data.iter().filter(|&&x| x % 2 == 0).map(|&x| x * 2).sum();
```

### Example: When Iterators Have Overhead

Trait object iterators (`Box<dyn Iterator>`) prevent inlining—each `next()` becomes a virtual call. Collecting to intermediate `Vec`s creates unnecessary allocations. Store generic iterators `I: Iterator` in structs instead of `Box<dyn Iterator>` for performance.

```rust
fn iterator_overhead() {
    // 1. Trait objects prevent inlining
    let data = vec![1, 2, 3, 4, 5];

    // This CAN'T be fully inlined:
    let iter: Box<dyn Iterator<Item = i32>> =
        Box::new(data.into_iter());
    let doubled: Box<dyn Iterator<Item = i32>> =
        Box::new(iter.map(|x| x * 2));
    // Each .next() is a virtual call

    // 2. Collect to intermediate collections
    let data = vec![1, 2, 3, 4, 5];
    let bad: i32 = data.iter()
        .map(|x| x * 2)
        .collect::<Vec<_>>()  // Unnecessary allocation!
        .iter()
        .sum();

    // Better: keep it lazy
    let good: i32 = data.iter()
        .map(|x| x * 2)
        .sum();

    // 3. Iterator in a struct (if using dyn)
    struct BadStreamer {
        source: Box<dyn Iterator<Item = i32>>,  // Virtual dispatch
    }

    struct GoodStreamer<I: Iterator<Item = i32>> {
        source: I,  // Monomorphized, inline-able
    }
}

// Keep iterator chains lazy—avoid collect() to intermediate Vec.
// Use generic iterators in structs instead of Box<dyn Iterator>.
let good: i32 = data.iter().map(|x| x * 2).sum(); // Lazy, efficient
// Avoid: .collect::<Vec<_>>().iter().sum() // Extra allocation
```

### Example: Custom Iterator for Efficiency

Implement `Iterator` for custom traversal patterns that stdlib doesn't provide efficiently. Add `#[inline]` hints on `next()` for optimization. Implement `size_hint()` and `ExactSizeIterator` to enable `collect()` pre-allocation optimizations.

```rust
// When stdlib iterators aren't enough

struct Windows<'a, T> {
    data: &'a [T],
    size: usize,
    pos: usize,
}

impl<'a, T> Iterator for Windows<'a, T> {
    type Item = &'a [T];

    #[inline]  // Hint for inlining
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + self.size <= self.data.len() {
            let window = &self.data[self.pos..self.pos + self.size];
            self.pos += 1;
            Some(window)
        } else {
            None
        }
    }

    // Implement size_hint for collect optimization
    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.data.len()
            .saturating_sub(self.pos + self.size - 1);
        (rem, Some(rem))
    }
}

// ExactSizeIterator enables optimizations
impl<'a, T> ExactSizeIterator for Windows<'a, T> {}

// Parallel iteration with rayon
fn parallel_example() {
    use rayon::prelude::*;  // If rayon is available

    let data: Vec<i32> = (0..1_000_000).collect();

    // Sequential
    let sum: i32 = data.iter().map(|x| x * 2).sum();

    // Parallel - just change iter() to par_iter()
    // let sum: i32 = data.par_iter().map(|x| x * 2).sum();
}

// Implement Iterator for custom traversal patterns.
// Add size_hint() for collect() optimization; #[inline] on next().
let windows = Windows { data: &data, size: 3, pos: 0 };
let result: Vec<_> = windows.collect(); // [[1,2,3], [2,3,4],...]
```

## Pattern 7: Closure Patterns for APIs

*   **Problem**: Designing APIs that accept callbacks or actions requires choosing between generics, trait objects, and function pointers.
*   **Solution**: Use generics for performance-critical hot paths, trait objects for flexibility and heterogeneous collections, and function pointers for FFI.
*   **Why It Matters**: The right choice affects API ergonomics, performance, and compatibility.

### Example: Builder with Callbacks

Builder patterns with optional callbacks use `Option<Box<dyn Fn>>` for stored closures. Add `Send + Sync` bounds if the builder will be used across threads. The builder consumes and returns `self` for method chaining.

```rust
struct HttpClient {
    timeout: u64,
    on_request: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_response: Option<Box<dyn Fn(u16) + Send + Sync>>,
}

impl HttpClient {
    fn builder() -> HttpClientBuilder {
        HttpClientBuilder::default()
    }
}

#[derive(Default)]
struct HttpClientBuilder {
    timeout: u64,
    on_request: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_response: Option<Box<dyn Fn(u16) + Send + Sync>>,
}

impl HttpClientBuilder {
    fn timeout(mut self, ms: u64) -> Self {
        self.timeout = ms;
        self
    }

    // Accept any Fn, box it internally
    fn on_request<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_request = Some(Box::new(callback));
        self
    }

    fn on_response<F>(mut self, callback: F) -> Self
    where
        F: Fn(u16) + Send + Sync + 'static,
    {
        self.on_response = Some(Box::new(callback));
        self
    }

    fn build(self) -> HttpClient {
        HttpClient {
            timeout: self.timeout,
            on_request: self.on_request,
            on_response: self.on_response,
        }
    }
}

// Usage
fn builder_usage() {
    let client = HttpClient::builder()
        .timeout(5000)
        .on_request(|url| println!("Requesting: {}", url))
        .on_response(|status| println!("Got status: {}", status))
        .build();
}

// Store callbacks as Option<Box<dyn Fn>> with Send + Sync
// Accept any Fn via generics and box it internally in builder.
let client = HttpClient::builder()
    .timeout(5000)
    .on_request(|url| println!("Requesting: {}", url))
    .build();
```

### Example: Scoped Callbacks (No Allocation)

When a callback's lifetime is limited to a function call, use generics instead of `Box<dyn Fn>` to avoid heap allocation. The `with_*` pattern (like `with_transaction`) ensures cleanup happens via RAII even if the closure panics. This is Rust's answer to try-with-resources or context managers.

```rust
// When closure lifetime is limited to a scope, avoid Box

fn with_transaction<F, T>(f: F) -> Result<T, &'static str>
where
    F: FnOnce(&mut Transaction) -> T,
{
    let mut tx = Transaction::begin();
    let result = f(&mut tx);
    tx.commit()?;
    Ok(result)
}

struct Transaction {
    // ...
}

impl Transaction {
    fn begin() -> Self { Transaction {} }
    fn execute(&mut self, _sql: &str) {}
    fn commit(self) -> Result<(), &'static str> { Ok(()) }
}

fn transaction_usage() {
    let result = with_transaction(|tx| {
        tx.execute("INSERT INTO users ...");
        tx.execute("UPDATE accounts ...");
        42  // Return value
    });
}

// Resource management with closures
fn with_file<F, T>(path: &str, f: F) -> std::io::Result<T>
where
    F: FnOnce(&mut std::fs::File) -> T,
{
    let mut file = std::fs::File::open(path)?;
    Ok(f(&mut file))
}

// This pattern ensures cleanup even if closure panics
fn with_cleanup<T, F, C>(value: T, f: F, cleanup: C)
where
    F: FnOnce(&T),
    C: FnOnce(T),
{
    struct Guard<T, C: FnOnce(T)> {
        value: Option<T>,
        cleanup: Option<C>,
    }

    impl<T, C: FnOnce(T)> Drop for Guard<T, C> {
        fn drop(&mut self) {
            let pair = (self.value.take(), self.cleanup.take());
            if let (Some(v), Some(c)) = pair {
                c(v);
            }
        }
    }

    let guard = Guard {
        value: Some(value),
        cleanup: Some(cleanup),
    };
    f(guard.value.as_ref().unwrap());
    // cleanup runs when guard drops
}

// Use with_* pattern for scoped resource management—no Box.
// Cleanup happens via RAII even if the closure panics.
let result = with_transaction(|tx| {
    tx.execute("INSERT INTO users ...");
    42
});
```

### Example: Plugin Systems

Plugin registries store closures in a `HashMap<String, Box<dyn Fn>>` for runtime lookup by name. For richer plugins, define a `Plugin` trait with multiple methods and store `Box<dyn Plugin>`. Both approaches require `'static` lifetime for stored closures.

```rust
use std::collections::HashMap;

// Type-erased plugin registry
struct PluginRegistry {
    processors: HashMap<String, Box<dyn Fn(&str) -> String + Send>>,
}

impl PluginRegistry {
    fn new() -> Self {
        PluginRegistry {
            processors: HashMap::new(),
        }
    }

    fn register<F>(&mut self, name: &str, processor: F)
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.processors.insert(name.into(), Box::new(processor));
    }

    fn process(&self, name: &str, input: &str) -> Option<String> {
        self.processors.get(name).map(|f| f(input))
    }
}

// Alternative: trait-based plugins (more flexible)
trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, input: &str) -> String;
}

struct TraitRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl TraitRegistry {
    fn register(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    fn process(&self, name: &str, input: &str) -> Option<String> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.process(input))
    }
}

// Store closures in HashMap<String, Box<dyn Fn>> for lookup.
// Closures must be 'static; use trait objects for richer APIs.
let mut registry = PluginRegistry::new();
registry.register("upper", |s| s.to_uppercase());
registry.process("upper", "hello"); // Some("HELLO")
```

## Pattern 8: Advanced Composition Patterns

*   **Problem**: Complex applications need sophisticated function composition: middleware chains, validation pipelines, and transformation sequences.
*   **Solution**: Build composable abstractions using closures, leverage `Option`/`Result` combinators, and create custom combinator APIs.
*   **Why It Matters**: These patterns enable clean, maintainable code for complex data transformations and control flow.

### Example: Middleware Chains

Middleware chains process requests through a sequence of transformations. Each middleware receives the value and a `next` function to call the rest of the chain. The recursive structure `Fn(T, &dyn Fn(T) -> T) -> T` allows short-circuiting or modification at any step.

```rust
type Middleware<T> = Box<dyn Fn(T, &dyn Fn(T) -> T) -> T>;

struct Pipeline<T> {
    middlewares: Vec<Middleware<T>>,
}

impl<T: 'static> Pipeline<T> {
    fn new() -> Self {
        Pipeline { middlewares: Vec::new() }
    }

    fn use_middleware<F>(&mut self, f: F)
    where
        F: Fn(T, &dyn Fn(T) -> T) -> T + 'static,
    {
        self.middlewares.push(Box::new(f));
    }

    fn execute(&self, initial: T) -> T {
        self.middlewares.iter().rev().fold(
            Box::new(|x| x) as Box<dyn Fn(T) -> T>,
            |next, middleware| {
                Box::new(move |x| middleware(x, &*next))
            }
        )(initial)
    }
}

// Simpler approach with impl Fn
fn compose<T, F, G>(outer: F, inner: G) -> impl Fn(T) -> T
where
    F: Fn(T) -> T,
    G: Fn(T) -> T,
{
    move |x| outer(inner(x))
}

fn middleware_example() {
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;
    let square = |x: i32| x * x;

    // Compose: square(double(add_one(x)))
    let pipeline = compose(square, compose(double, add_one));
    println!("Result: {}", pipeline(5));  // ((5+1)*2)^2 = 144
}

// Compose functions with nested calls or fold over transforms.
// compose(sq, compose(dbl, add_one)) = sq(dbl(add_one(x)))
let pipeline = compose(square, compose(double, add_one));
println!("{}", pipeline(5)); // ((5+1)*2)^2 = 144
```

### Example: Railway-Oriented Programming

Chain fallible operations with `and_then()` for "happy path" flow that short-circuits on errors. The `?` operator is often clearer than combinator chains for sequential validation. For collecting all errors instead of failing fast, use applicative style with explicit matching.

```rust
// Chain operations that might fail, with early exit on error

fn validate_age(age: i32) -> Result<i32, String> {
    if age < 0 {
        Err("Age cannot be negative".into())
    } else if age > 150 {
        Err("Age seems unrealistic".into())
    } else {
        Ok(age)
    }
}

fn validate_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        Err("Name cannot be empty".into())
    } else if name.len() > 100 {
        Err("Name too long".into())
    } else {
        Ok(name.to_string())
    }
}

struct User {
    name: String,
    age: i32,
}

// Railway style with and_then
fn create_user(name: &str, age: i32) -> Result<User, String> {
    validate_name(name)
        .and_then(|valid_name| {
            validate_age(age).map(|valid_age| User {
                name: valid_name,
                age: valid_age,
            })
        })
}

// Or with ? operator (often clearer)
fn create_user_v2(name: &str, age: i32) -> Result<User, String> {
    let valid_name = validate_name(name)?;
    let valid_age = validate_age(age)?;
    Ok(User { name: valid_name, age: valid_age })
}

// Collecting all errors (applicative style)
fn validate_all(name: &str, age: i32) -> Result<User, Vec<String>> {
    let name_result = validate_name(name);
    let age_result = validate_age(age);

    match (name_result, age_result) {
        (Ok(n), Ok(a)) => Ok(User { name: n, age: a }),
        (Err(e1), Err(e2)) => Err(vec![e1, e2]),
        (Err(e), _) | (_, Err(e)) => Err(vec![e]),
    }
}

// Chain fallible operations with and_then() or the ? operator.
// The ? operator is often clearer for sequential validation.
let result = validate_name(name).and_then(|n| {
    validate_age(age).map(|a| User { name: n, age: a })
});
// Or: let n = validate_name(name)?; let a = validate_age(age)?;
```

### Example: Lazy Evaluation with Closures

Lazy evaluation defers computation until the value is actually needed. Store a `FnOnce` closure that produces the value on first access, then cache the result. The stdlib provides `OnceCell` and `LazyCell` for production use of this pattern.

```rust
// Thunks: delayed computation
struct Lazy<T, F: FnOnce() -> T> {
    compute: Option<F>,
    value: Option<T>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {
    fn new(f: F) -> Self {
        Lazy { compute: Some(f), value: None }
    }

    fn force(&mut self) -> &T {
        if self.value.is_none() {
            let f = self.compute.take().unwrap();
            self.value = Some(f());
        }
        self.value.as_ref().unwrap()
    }
}

// Once cell pattern (stdlib has OnceCell/LazyCell)
use std::cell::OnceCell;

struct Config {
    database_url: OnceCell<String>,
}

impl Config {
    fn database_url(&self) -> &str {
        self.database_url.get_or_init(|| {
            // Expensive initialization, only done once
            std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "localhost".into())
        })
    }
}

// Memoization
use std::collections::HashMap;
use std::cell::RefCell;

fn memoize<A, R, F>(f: F) -> impl FnMut(A) -> R
where
    A: std::hash::Hash + Eq + Clone,
    R: Clone,
    F: Fn(A) -> R,
{
    let cache = RefCell::new(HashMap::new());
    move |arg: A| {
        if let Some(result) = cache.borrow().get(&arg) {
            return result.clone();
        }
        let result = f(arg.clone());
        cache.borrow_mut().insert(arg, result.clone());
        result
    }
}

// Defer expensive computation with OnceCell::get_or_init().
// Closure runs once on first access; later calls return cached val
let cell: OnceCell<String> = OnceCell::new();
let url = cell.get_or_init(|| {
    std::env::var("DATABASE_URL").unwrap_or("localhost".into())
});
```

### Summary

This chapter covered functional programming patterns unique to Rust:

1. **Function Type System**: Function items vs pointers, monomorphization, never type
2. **Closure Internals**: Compiler-generated structs, unique types, Copy/Clone rules
3. **Capture Semantics**: Field-level capture, mode inference, move patterns
4. **fn vs Fn Traits**: When to use each, coercion rules, performance implications
5. **Higher-Order Functions**: Accepting and returning functions, lifetime handling
6. **Iterator Optimization**: Zero-cost abstraction, when overhead occurs
7. **API Design**: Callbacks, builders, plugin systems
8. **Composition Patterns**: Middleware, railway-oriented, lazy evaluation

**Key Takeaways**:
- Every closure is a unique struct implementing Fn traits
- Function items are zero-sized; function pointers are 8 bytes
- Rust 2021 captures individual fields, not whole structs
- `impl Fn` in arguments = monomorphization = inlining
- `dyn Fn` = virtual dispatch = no inlining
- Iterator chains compile to the same code as manual loops
- `move` copies Copy types, moves non-Copy types

**Performance Mental Model**:
```
Fastest                                    Slowest
   |                                           |
   v                                           v
Generic    fn ptr    &dyn Fn    Box<dyn Fn>
(inlined)  (call)    (vtable)   (heap+vtable)
```

**Closure Size Formula**:
- Non-move: sum of pointer sizes for captured references
- Move: sum of sizes of captured values
- No capture: zero-sized (can coerce to fn pointer)

**Design Guidelines**:
- Accept `impl Fn` for maximum performance
- Return `impl Fn` when single type, `Box<dyn Fn>` for multiple
- Use `fn` only for FFI or dispatch tables
- Prefer iterator chains over manual loops
- Use `move` for threads and 'static requirements
- Box callbacks in long-lived structs, use generics for scoped callbacks
