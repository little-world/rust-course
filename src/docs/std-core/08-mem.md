Here's a **cookbook-style tutorial for `std::mem`**, a Rust standard module for **memory-related operations**, such as swapping values, replacing them, moving them, and inspecting their layout. It's low-level and powerful — great for systems programming and advanced data manipulation.

---

## Rust std::mem Cookbook

> 📦 Module: [`std::mem`](https://doc.rust-lang.org/std/mem/)

Contains functions like:

* `size_of`, `align_of`
* `replace`, `take`, `swap`
* `forget`, `drop`
* `transmute` (⚠️ unsafe)

---

## Recipes and Examples

---

### Get Size of a Type

```rust
use std::mem;

fn main() {
    println!("Size of i32: {}", mem::size_of::<i32>()); // 4
    println!("Size of &str: {}", mem::size_of::<&str>()); // 8 on 64-bit
}
```

📘 Use to understand memory usage of types.

---

### Get Alignment of a Type

```rust
use std::mem;

fn main() {
    println!("Align of f64: {}", mem::align_of::<f64>());
}
```

📘 Useful for layout-sensitive operations (FFI, custom allocators).

---

### Replace a Value Without Moving Out

```rust
use std::mem;

fn main() {
    let mut s = String::from("hello");
    let old = mem::replace(&mut s, String::from("world"));

    println!("old = {}, new = {}", old, s);
}
```

📘 Replaces in-place and returns the old value.

---

### Take a Value and Replace with Default

```rust
use std::mem;

fn main() {
    let mut name = Some(String::from("Alice"));
    let taken = mem::take(&mut name); // replaces with None

    println!("Taken = {:?}, Remaining = {:?}", taken, name);
}
```

📘 Equivalent to `std::mem::replace(&mut x, Default::default())`.

---

### Swap Two Values

```rust
use std::mem;

fn main() {
    let mut a = 1;
    let mut b = 2;
    mem::swap(&mut a, &mut b);

    println!("a = {}, b = {}", a, b); // a = 2, b = 1
}
```

📘 In-place, no copying or temporary variable needed.

---

### Drop a Value Explicitly

```rust
use std::mem;

fn main() {
    let s = String::from("important");
    mem::drop(s); // frees memory early
    // println!("{}", s); // ❌ use-after-move error
}
```

📘 Use to release memory or trigger `Drop::drop()` early.

---

### Prevent a Value from Being Dropped (forget)

```rust
use std::mem;

fn main() {
    let s = String::from("leaked");
    mem::forget(s); // memory leak — destructor not run
}
```

📘 Useful in FFI or unsafe code. **⚠️ Dangerous if misused.**

---

### Move a Value Out of a Field

You can’t directly move from borrowed data, but you can use `take()` or `replace()`:

```rust
#[derive(Debug)]
struct Wrapper {
    name: Option<String>,
}

fn main() {
    let mut w = Wrapper { name: Some("Rust".into()) };
    let moved = w.name.take(); // safe move out

    println!("{:?}", moved);
}
```

---

### Zero-Initialize a Type with MaybeUninit (safe alternative)

```rust
use std::mem::MaybeUninit;

fn main() {
    let x = MaybeUninit::<i32>::uninit(); // uninitialized, unsafe to read
    println!("Uninit created"); // value not accessed
}
```

📘 Used in `unsafe` code or performance-sensitive buffers.

---

### Unsafe: Transmute One Type to Another

```rust
use std::mem;

fn main() {
    let bytes: [u8; 4] = [0, 0, 0, 42];
    let num: u32 = unsafe { mem::transmute(bytes) };
    println!("Transmuted: {}", num);
}
```

📘 ⚠️ Highly unsafe. Only use when layout and size are guaranteed to match.

---

## Summary Table

| Function          | Purpose                                 |
| ----------------- | --------------------------------------- |
| `size_of::<T>()`  | Get memory size of a type               |
| `align_of::<T>()` | Get memory alignment of a type          |
| `replace()`       | Replace value in-place, return old one  |
| `take()`          | Replace value with `Default::default()` |
| `swap()`          | Swap two variables in-place             |
| `drop()`          | Manually drop a value                   |
| `forget()`        | Prevent drop from running               |
| `transmute()`     | Reinterpret memory as another type (⚠️) |
| `MaybeUninit<T>`  | Unsafe, uninitialized memory            |

---

### When to Use std::mem

* **Performance**: avoid copies, pre-allocate uninitialized memory
* **FFI**: raw memory layouts and alignment
* **Unsafe code**: manually controlling lifetimes or memory safety
* **Generic programming**: default replacement, buffer swaps
