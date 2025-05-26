
## Match


Rust uses pattern matching primarily through the `match` keyword, but also with `if let`, `while let`, and destructuring.



### Basic Syntax

```rust
fn main() {
    let number = 3;

    match number {
        1 => println!("One"),
        2 => println!("Two"),
        3 => println!("Three"),
        _ => println!("Something else"),
    }
}
```

* `match` is **exhaustive**: you must handle every possible case.
* `_` is a **catch-all** pattern.


### Multiple Values

```rust
match x {
    1 | 2 => println!("One or Two"),
    3..=5 => println!("Between 3 and 5"),
    _ => println!("Something else"),
}
```

* `|` = **OR**
* `..=` = inclusive range


## Destructuring

### Tuples

```rust
let point = (0, 7);

match point {
    (0, y) => println!("On Y axis at {}", y),
    (x, 0) => println!("On X axis at {}", x),
    (x, y) => println!("At ({}, {})", x, y),
}
```

### Structs

```rust
struct Person {
    name: String,
    age: u8,
}

let user = Person {
    name: String::from("Alice"),
    age: 30,
};

match user {
    Person { name, age: 30 } => println!("30-year-old named {}", name),
    Person { name, age } => println!("{} is {} years old", name, age),
}
```


### Destructuring Enums

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Say(String),
}

let msg = Message::Move { x: 1, y: 2 };

match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Say(text) => println!("Say: {}", text),
}
```


## If Let 

When you only care about one pattern:

```rust
let some_number = Some(5);

if let Some(x) = some_number {
    println!("Got: {}", x);
}
```

You can combine with `else`:

```rust
if let Some(x) = some_number {
    println!("Value: {}", x);
} else {
    println!("No value");
}
```


### while let Looping 

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("Top: {}", top);
}
```

---

##  Guards 
### if in match arm

```rust
let x = Some(4);

match x {
    Some(n) if n % 2 == 0 => println!("Even number: {}", n),
    Some(n) => println!("Odd number: {}", n),
    None => println!("No number"),
}
```

---

## Summary

| Pattern Type        | Example                         |    
|---------------------|---------------------------------| 
| Literal             | `match x { 1 => ... }`          |    
| Multiple values     | `1 \| 2 => ...`                 |
| Range               | `1..=5 => ...`                  |    
| Struct destructure  | `Person { name, age }`          |    
| Enum variant match  | `Message::Move { x, y } => ...` | 
| Conditional (guard) | `Some(n) if n > 0 => ...`       |    
| `if let`            | `if let Some(x) = ...`          |    
| `while let`         | `while let Some(x) = ...`       |    