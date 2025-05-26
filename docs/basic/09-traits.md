

## Trait

A **trait** in Rust defines **shared behavior** — like an interface in other languages.

> 🧠 If a type implements a trait, it guarantees it provides the specified behavior.


### Define a Trait:

```rust
trait Greet {
    fn greet(&self); // method signature only
}
```

### Implement for a Struct:

```rust
struct Person {
    name: String,
}

impl Greet for Person {
    fn greet(&self) {
        println!("Hello, my name is {}!", self.name);
    }
}

fn main() {
    let user = Person { name: String::from("Alice") };
    user.greet(); // Trait method call
}
```


## Default Methods

You can provide **default behavior** in the trait:

```rust
trait Greet {
    fn greet(&self) {
        println!("Hello!");
    }
}
```

Types can **override or use the default**:

```rust
impl Greet for Person {} // uses default
```

---



## Using Traits

### Trait Bound (Generic Syntax):

```rust
fn make_greet<T: Greet>(thing: T) {
    thing.greet();
}
```

### Trait Object (Dynamic Dispatch):

```rust
fn greet_dyn(g: &dyn Greet) {
    g.greet();
}
```


## Deriving 

Rust includes built-in traits like `Debug`, `Clone`, `PartialEq`, etc.

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
```

```rust
fn main() {
    let p1 = Point { x: 1, y: 2 };
    println!("{:?}", p1);
}
```


## Summary 

| Trait Concept   | Example                       |
| --------------- | ----------------------------- |
| Define trait    | `trait Greet { fn greet(); }` |
| Implement trait | `impl Greet for Person`       |
| Default method  | Inside `trait` block          |
| Use in function | `fn f<T: Trait>(x: T)`        |
| Use dynamically | `fn f(x: &dyn Trait)`         |
| Built-in derive | `#[derive(Debug, Clone)]`     |