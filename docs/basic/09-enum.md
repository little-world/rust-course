

## Enum

An `enum` (**enumeration**) in Rust is a type that can be **one of several possible variants**. It's like `enum` in other languages — but **way more powerful**.


### Enum Definition

```rust
enum Direction {
    North,
    South,
    East,
    West,
}

fn main() {
    let dir = Direction::North;
}
```

You can use pattern matching to act on it:

```rust
match dir {
    Direction::North => println!("Going up"),
    Direction::South => println!("Going down"),
    _ => println!("Other direction"),
}
```


## with Data

Variants can carry data — like different shapes of a message.

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Text(String),
    ChangeColor(u8, u8, u8),
}
```

### Use:

```rust
let m1 = Message::Quit;
let m2 = Message::Move { x: 10, y: 20 };
let m3 = Message::Text(String::from("Hello"));
let m4 = Message::ChangeColor(255, 0, 0);
```

---

## Pattern Matching

```rust
fn process(msg: Message) {
    match msg {
        Message::Quit => println!("Quit"),
        Message::Move { x, y } => println!("Move to ({}, {})", x, y),
        Message::Text(s) => println!("Text: {}", s),
        Message::ChangeColor(r, g, b) => println!("Color: {},{},{}", r, g, b),
    }
}
```



## with Methods

You can implement methods on enums using `impl`:

```rust
impl Direction {
    fn turn(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }
}
```

---

## Standard Enums

### Option: Some or None

```rust
let name: Option<String> = Some(String::from("Alice"));
let nothing: Option<i32> = None;

if let Some(n) = name {
    println!("Name: {}", n);
}
```

### Result: Ok or Err

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Cannot divide by zero".into())
    } else {
        Ok(a / b)
    }
}
```


## with Macros

Add traits like `Debug`, `Clone`, `PartialEq`:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Status {
    Success,
    Failure(String),
}
```



## Summary

| Feature              | Example                             |
| -------------------- | ----------------------------------- |
| Define enum          | `enum Color { Red, Green, Blue }`   |
| Variant with data    | `Text(String)`, `Move { x: i32 }`   |
| Match enum           | `match value { Variant => ... }`    |
| Method on enum       | `impl Enum { fn method(&self) {} }` |
| Pattern match `Some` | `if let Some(x) = ...`              |
