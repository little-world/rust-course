


## Defining a Function

In Rust, functions are defined using the `fn` keyword:

```rust
fn main() {
    greet(); // calling a function
}

fn greet() {
    println!("Hello from Rust!");
}
```

* The `main` function is the program’s entry point.
* Functions must be defined **before or after** `main`.



## Parameters

### With Parameters:

```rust
fn greet(name: &str) {
    println!("Hello, {}!", name);
}
```

### With Return Value:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b // no semicolon = return value
}

fn main() {
    let result = add(3, 4);
    println!("Sum: {}", result);
}
```

* The arrow `->` defines the return type.
* Rust returns the **last expression** if there’s no semicolon.
* Use `return` if you want to return early:

```rust
return a * b;
```


### Return Tuple

```rust
fn stats(numbers: &[i32]) -> (i32, i32) {
    let min = *numbers.iter().min().unwrap();
    let max = *numbers.iter().max().unwrap();
    (min, max)
}
```


## Ownership 

### Takes Ownership:

```rust
fn consume(s: String) {
    println!("Consumed: {}", s);
    // s is dropped here
}
```

### Borrowing (no ownership transfer):

```rust
fn borrow(s: &String) {
    println!("Borrowed: {}", s);
}

fn main() {
    let name = String::from("Alice");
    borrow(&name); // name still valid
}
```

---

## Closures 

Closures are like lambdas:

```rust
fn main() {
    let add = |x: i32, y: i32| -> i32 {
        x + y
    };

    println!("Sum: {}", add(2, 3));
}
```

You can omit types and the return type if they are clear:

```rust
let square = |x| x * x;
```



## Best Practices

* Keep functions small and focused on one task.
* Use expressive names.
* Leverage Rust’s type system for safety and clarity.
* Use `&` for references to avoid unnecessary moves.


### Summary Table

| Concept          | Example                       | 
| ---------------- |-------------------------------| 
| Basic function   | `fn greet() { ... }`          | 
| With parameters  | `fn add(a: i32, b: i32)`      | 
| Return value     | `fn square(x: i32) -> i32`    | 
| Tuple return     | `(i32, i32)`                  | 
| Ownership passed | `fn consume(s: String)`       | 
| Borrowing        | `fn borrow(s: &String)`       | 
| Closure          | `let add = \| x, y \| x + y;` |