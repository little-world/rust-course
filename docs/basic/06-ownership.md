

## Ownership

Every value in Rust has a **single owner** — a variable responsible for cleaning it up. When ownership moves, the original variable can no longer be used.

### Example:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1; // ownership moves to s2

    // println!("{}", s1); // ❌ Error! s1 is no longer valid
    println!("{}", s2); // ✅
}
```

🧠 **Why?** Rust prevents double-free errors by enforcing single ownership.


### Cloning 

You can **clone** the data if you want two valid owners:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1.clone(); // deep copy

    println!("s1: {}, s2: {}", s1, s2); // ✅ Both valid
}
```


## Borrowing 

Borrowing lets you **use a value without taking ownership**, via a reference (`&`).

### Immutable Borrow:

```rust
fn print_length(s: &String) {
    println!("Length: {}", s.len());
}

fn main() {
    let s = String::from("hello");
    print_length(&s); // Borrowed, not moved
    println!("Still valid: {}", s); // ✅
}
```

### Mutable Borrow:

```rust
fn add_exclamation(s: &mut String) {
    s.push_str("!");
}

fn main() {
    let mut msg = String::from("Hello");
    add_exclamation(&mut msg);
    println!("{}", msg); // Hello!
}
```

🛑 **Rules**:

* At any time, **either** one mutable reference **or** any number of immutable references.
* References must always be **valid**.



### Dangling References Are Not Allowed

Rust prevents returning a reference to data that will be dropped:

```rust
// fn dangle() -> &String {
//     let s = String::from("hello");
//     &s // ❌ Error: s goes out of scope!
// }
```

## Function 

### Takes ownership:

```rust
fn take(s: String) {
    println!("Took: {}", s);
}
```

### Returns ownership:

```rust
fn give() -> String {
    String::from("hi")
}
```

### Borrowing:

```rust
fn peek(s: &String) {
    println!("Peek: {}", s);
}
```

### Example combining them:

```rust
fn main() {
    let s = String::from("Rust");
    peek(&s); // Borrowed
    let s = modify(s); // Ownership moved and returned
    println!("Now: {}", s);
}

fn modify(mut s: String) -> String {
    s.push_str(" is great!");
    s
}
```

---

## Summary Table

| Concept          | Description                     | Example                     |
| ---------------- | ------------------------------- | --------------------------- |
| Ownership        | Variable responsible for value  | `let s2 = s1;`              |
| Move             | Ownership transferred           | `s1` becomes invalid        |
| Clone            | Deep copy of data               | `let s2 = s1.clone();`      |
| Immutable Borrow | Use without changing            | `fn peek(&s: &String)`      |
| Mutable Borrow   | Change without taking ownership | `fn modify(s: &mut String)` |
| Return Ownership | Return values from functions    | `fn give() -> String`       |