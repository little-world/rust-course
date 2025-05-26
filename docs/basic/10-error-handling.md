
## Error Handling

* `panic!` (unrecoverable errors)
* `Result<T, E>` (recoverable errors)
* `Option<T>` (presence/absence)
* `?` operator
* Custom error types


### Unrecoverable Errors: panic!

Use `panic!()` when your program encounters a **critical failure** and **must stop**.

```rust
fn main() {
    panic!("Something went terribly wrong!");
}
```

### Example: Index out of bounds

```rust
let v = vec![1, 2, 3];
println!("{}", v[99]); // panics!
```

---

## Result

Use `Result` when errors **can and should be handled** gracefully.

### Basic Syntax

```rust
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err(String::from("Cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}

fn main() {
    match divide(10.0, 2.0) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
}
```


## The ? Operator 
### Quick Propagation

Instead of writing nested `match`, use `?` to **bubble up** the error.

```rust
use std::fs::File;
use std::io::{self, Read};

fn read_file() -> Result<String, io::Error> {
    let mut file = File::open("myfile.txt")?; // returns early if error
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
```

> The `?` operator only works in functions that return `Result`.


##  Option

Used when a value **might not exist** (e.g., indexing, searching).

```rust
fn get_item(index: usize) -> Option<&'static str> {
    let items = ["apple", "banana"];
    items.get(index).copied()
}

fn main() {
    match get_item(1) {
        Some(item) => println!("Got: {}", item),
        None => println!("Item not found"),
    }
}
```



## Combining

You can convert between them:

```rust
let maybe = Some("data");
let result: Result<_, &str> = maybe.ok_or("No data"); // converts Option -> Result
```

---

## Custom Error

Use an `enum` and implement `std::fmt::Display` and `std::error::Error`.

```rust
use std::fmt;

#[derive(Debug)]
enum MyError {
    NotFound,
    Invalid,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for MyError {}

fn do_something() -> Result<(), MyError> {
    Err(MyError::NotFound)
}
```


## Summary 

| Concept         | Type           | Use Case                     |
| --------------- | -------------- | ---------------------------- |
| Crash now       | `panic!()`     | Serious, unrecoverable error |
| Handle result   | `Result<T, E>` | Errors that can be handled   |
| Missing value   | `Option<T>`    | Value may be absent          |
| Propagate error | `?` operator   | Early return on error        |
| Custom errors   | `enum + impl`  | Define your own error logic  |