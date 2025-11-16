# Comprehensive Guide to Advanced Pattern Matching in Rust

Pattern matching is one of Rust's most powerful features, enabling concise and safe code. This guide covers advanced techniques beyond basic `match` statements.

## Table of Contents

1. [Pattern Matching Fundamentals](#pattern-matching-fundamentals)
2. [Match Expressions](#match-expressions)
3. [Pattern Matching Locations](#pattern-matching-locations)
4. [Destructuring Patterns](#destructuring-patterns)
5. [Advanced Pattern Syntax](#advanced-pattern-syntax)
6. [Pattern Guards](#pattern-guards)
7. [@ Bindings](#-bindings)
8. [Reference Patterns](#reference-patterns)
9. [Slice Patterns](#slice-patterns)
10. [Nested Patterns](#nested-patterns)
11. [Refutability](#refutability)
12. [Binding Modes](#binding-modes)
13. [Advanced Enum Patterns](#advanced-enum-patterns)
14. [Real-World Patterns](#real-world-patterns)
15. [Performance Considerations](#performance-considerations)
16. [Common Pitfalls](#common-pitfalls)
17. [Quick Reference](#quick-reference)

---

## Pattern Matching Fundamentals

### What is a Pattern?

A pattern is a special syntax in Rust used to match against the structure of types. Patterns can:
- Match literal values
- Destructure structs, enums, tuples, and arrays
- Create variable bindings
- Ignore values
- Test conditions with guards

```rust
fn main() {
    let x = 5;

    // All of these are patterns:
    let y = x;                    // Irrefutable pattern (always matches)
    let (a, b) = (1, 2);         // Tuple destructuring pattern
    let Some(value) = Some(3);   // Refutable pattern (may fail)

    match x {
        1 => println!("one"),     // Literal pattern
        2..=5 => println!("2-5"), // Range pattern
        _ => println!("other"),   // Wildcard pattern
    }
}
```

---

## Match Expressions

### Exhaustiveness

Match expressions must be exhaustive - all possible values must be handled.

```rust
enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {
    let status = Status::Active;

    // ✅ Exhaustive - all variants covered
    match status {
        Status::Active => println!("Active"),
        Status::Inactive => println!("Inactive"),
        Status::Pending => println!("Pending"),
    }

    // ✅ Also exhaustive - wildcard covers remaining
    match status {
        Status::Active => println!("Active"),
        _ => println!("Not active"),
    }
}
```

---

### Match as an Expression

Match is an expression that returns a value.

```rust
fn main() {
    let number = 7;

    let description = match number {
        1 => "one",
        2 | 3 | 5 | 7 | 11 => "prime (small)",
        n if n % 2 == 0 => "even",
        _ => "other",
    };

    println!("Number {} is {}", number, description);

    // Use in complex expressions
    let result = match compute_value() {
        Ok(val) => val * 2,
        Err(_) => 0,
    };
}

fn compute_value() -> Result<i32, ()> {
    Ok(42)
}
```

---

### Multiple Patterns with OR (|)

```rust
fn main() {
    let x = 2;

    match x {
        1 | 2 => println!("one or two"),
        3 | 4 | 5 => println!("three through five"),
        _ => println!("something else"),
    }

    // Works with more complex patterns
    enum Message {
        Hello,
        Goodbye,
        Data(i32),
    }

    let msg = Message::Hello;

    match msg {
        Message::Hello | Message::Goodbye => println!("greeting"),
        Message::Data(_) => println!("data"),
    }
}
```

---

### Range Patterns

```rust
fn main() {
    let x = 5;

    match x {
        1..=5 => println!("between 1 and 5 inclusive"),
        6..=10 => println!("between 6 and 10"),
        _ => println!("something else"),
    }

    // Character ranges
    let c = 'k';
    match c {
        'a'..='j' => println!("first part of alphabet"),
        'k'..='z' => println!("second part of alphabet"),
        _ => println!("not a lowercase letter"),
    }

    // Can use in guards too
    let age = 25;
    match age {
        0..=17 => println!("minor"),
        18..=64 => println!("adult"),
        65.. => println!("senior"),
        _ => unreachable!(),
    }
}
```

---

## Pattern Matching Locations

### In let Statements

```rust
fn main() {
    // Simple binding
    let x = 5;

    // Tuple destructuring
    let (x, y, z) = (1, 2, 3);

    // Struct destructuring
    struct Point { x: i32, y: i32 }
    let Point { x, y } = Point { x: 10, y: 20 };

    // Array destructuring
    let [first, second, third] = [1, 2, 3];

    // Ignoring values
    let (a, _, c) = (1, 2, 3);
    let Point { x: my_x, y: _ } = Point { x: 5, y: 10 };
}
```

---

### In Function Parameters

```rust
// Tuple parameters
fn print_coordinates(&(x, y): &(i32, i32)) {
    println!("x: {}, y: {}", x, y);
}

// Struct parameters
struct Point { x: i32, y: i32 }

fn print_point(&Point { x, y }: &Point) {
    println!("Point at ({}, {})", x, y);
}

// Enum parameters
enum Message {
    Move { x: i32, y: i32 },
    Write(String),
}

fn handle_message(msg: Message) {
    match msg {
        Message::Move { x, y } => println!("Move to ({}, {})", x, y),
        Message::Write(text) => println!("Write: {}", text),
    }
}

fn main() {
    print_coordinates(&(5, 10));
    print_point(&Point { x: 3, y: 7 });
}
```

---

### In for Loops

```rust
fn main() {
    let pairs = vec![(1, 2), (3, 4), (5, 6)];

    // Destructure in for loop
    for (x, y) in pairs {
        println!("x: {}, y: {}", x, y);
    }

    // With enumerate
    let items = vec!["a", "b", "c"];
    for (index, value) in items.iter().enumerate() {
        println!("{}: {}", index, value);
    }

    // Nested destructuring
    let nested = vec![((1, 2), (3, 4)), ((5, 6), (7, 8))];
    for ((a, b), (c, d)) in nested {
        println!("a:{} b:{} c:{} d:{}", a, b, c, d);
    }
}
```

---

### if let Expressions

```rust
fn main() {
    let some_value = Some(42);

    // Basic if let
    if let Some(x) = some_value {
        println!("Got value: {}", x);
    }

    // With else
    if let Some(x) = some_value {
        println!("Value: {}", x);
    } else {
        println!("No value");
    }

    // Multiple if let
    let config = Some("production");

    if let Some("production") = config {
        println!("Running in production");
    } else if let Some("development") = config {
        println!("Running in development");
    } else {
        println!("Unknown environment");
    }

    // Combining with regular if
    let number = Some(7);
    if let Some(n) = number {
        if n > 5 {
            println!("Big number: {}", n);
        }
    }
}
```

---

### while let Loops

```rust
fn main() {
    let mut stack = vec![1, 2, 3, 4, 5];

    // Pop until empty
    while let Some(top) = stack.pop() {
        println!("Popped: {}", top);
    }

    // With complex patterns
    let mut data = vec![Some(1), Some(2), None, Some(3)];
    let mut index = 0;

    while let Some(Some(value)) = data.get(index) {
        println!("Value at {}: {}", index, value);
        index += 1;
    }

    // Pattern matching in iterator
    let mut iter = vec![Ok(1), Ok(2), Err("error"), Ok(3)].into_iter();

    while let Some(Ok(value)) = iter.next() {
        println!("Success: {}", value);
        // Stops at first Err
    }
}
```

---

## Destructuring Patterns

### Tuple Destructuring

```rust
fn main() {
    let tuple = (1, "hello", true, 3.14);

    // Full destructuring
    let (a, b, c, d) = tuple;

    // Partial destructuring with wildcards
    let (first, _, _, last) = tuple;

    // Nested tuples
    let nested = ((1, 2), (3, 4));
    let ((a, b), (c, d)) = nested;

    // In match
    let pair = (0, -2);
    match pair {
        (0, y) => println!("On Y axis at {}", y),
        (x, 0) => println!("On X axis at {}", x),
        (x, y) => println!("At ({}, {})", x, y),
    }
}
```

---

### Struct Destructuring

```rust
struct User {
    username: String,
    email: String,
    age: u32,
    active: bool,
}

fn main() {
    let user = User {
        username: String::from("alice"),
        email: String::from("alice@example.com"),
        age: 30,
        active: true,
    };

    // Full destructuring
    let User { username, email, age, active } = user;

    // Renaming fields
    let User {
        username: name,
        email: contact,
        age: years,
        active: is_active,
    } = User {
        username: String::from("bob"),
        email: String::from("bob@example.com"),
        age: 25,
        active: false,
    };

    // Partial destructuring
    let User { username, age, .. } = User {
        username: String::from("charlie"),
        email: String::from("charlie@example.com"),
        age: 35,
        active: true,
    };

    // In match
    match user_status() {
        User { active: true, age: 18..=64, .. } => println!("Active adult"),
        User { active: true, .. } => println!("Active"),
        User { active: false, .. } => println!("Inactive"),
    }
}

fn user_status() -> User {
    User {
        username: String::from("test"),
        email: String::from("test@example.com"),
        age: 30,
        active: true,
    }
}
```

---

### Enum Destructuring

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn main() {
    let msg = Message::ChangeColor(255, 0, 128);

    match msg {
        Message::Quit => println!("Quit"),
        Message::Move { x, y } => println!("Move to ({}, {})", x, y),
        Message::Write(text) => println!("Text: {}", text),
        Message::ChangeColor(r, g, b) => {
            println!("Change color to RGB({}, {}, {})", r, g, b)
        }
    }

    // Nested enums
    enum Color {
        Rgb(u8, u8, u8),
        Hsv(u8, u8, u8),
    }

    enum Action {
        SetColor(Color),
        Move { x: i32, y: i32 },
    }

    let action = Action::SetColor(Color::Rgb(255, 0, 0));

    match action {
        Action::SetColor(Color::Rgb(r, g, b)) => {
            println!("RGB: {}, {}, {}", r, g, b)
        }
        Action::SetColor(Color::Hsv(h, s, v)) => {
            println!("HSV: {}, {}, {}", h, s, v)
        }
        Action::Move { x, y } => println!("Move: {}, {}", x, y),
    }
}
```

---

## Advanced Pattern Syntax

### Ignoring Values with _

```rust
fn main() {
    // Ignore entire value
    let _ = expensive_computation();

    // Ignore parts of tuple
    let (x, _, z) = (1, 2, 3);

    // Ignore struct fields
    struct Point3D { x: i32, y: i32, z: i32 }
    let Point3D { x, .. } = Point3D { x: 1, y: 2, z: 3 };

    // Ignore function parameters
    fn do_something(_: i32, value: i32) -> i32 {
        value * 2
    }

    // In match
    match get_result() {
        Ok(value) => println!("Success: {}", value),
        Err(_) => println!("Error occurred"), // Ignore error details
    }
}

fn expensive_computation() -> i32 {
    42
}

fn get_result() -> Result<i32, String> {
    Ok(10)
}
```

---

### Ignoring Remaining Parts with ..

```rust
fn main() {
    // In tuples
    let tuple = (1, 2, 3, 4, 5);
    let (first, .., last) = tuple;
    println!("first: {}, last: {}", first, last); // 1, 5

    let (first, second, ..) = tuple;
    println!("first two: {}, {}", first, second); // 1, 2

    // In structs
    struct Config {
        host: String,
        port: u16,
        timeout: u64,
        retries: u32,
    }

    let config = Config {
        host: "localhost".to_string(),
        port: 8080,
        timeout: 30,
        retries: 3,
    };

    let Config { host, port, .. } = config;

    // In arrays/slices
    let arr = [1, 2, 3, 4, 5];
    let [first, .., last] = arr;
    println!("first: {}, last: {}", first, last);
}
```

---

### Multiple Binding Names with |

```rust
fn main() {
    let x = 1;

    match x {
        1 | 2 => println!("one or two"),
        3 | 4 | 5 => println!("three, four, or five"),
        _ => println!("something else"),
    }

    // With more complex patterns
    enum Direction {
        North,
        South,
        East,
        West,
    }

    let dir = Direction::North;

    match dir {
        Direction::North | Direction::South => println!("vertical"),
        Direction::East | Direction::West => println!("horizontal"),
    }

    // Cannot bind variables differently in OR patterns
    // This won't work:
    // match some_value {
    //     Some(x) | None(y) => ... // ERROR
    // }
}
```

---

## Pattern Guards

Pattern guards add extra conditions to match arms using `if`.

### Basic Guards

```rust
fn main() {
    let number = Some(4);

    match number {
        Some(x) if x < 5 => println!("Less than 5: {}", x),
        Some(x) => println!("5 or more: {}", x),
        None => println!("No number"),
    }

    // Multiple conditions
    let pair = (2, 3);
    match pair {
        (x, y) if x + y == 5 => println!("Sum is 5"),
        (x, y) if x * y == 6 => println!("Product is 6"),
        _ => println!("Neither condition met"),
    }
}
```

---

### Guards with OR Patterns

```rust
fn main() {
    let x = 4;
    let y = false;

    match x {
        // Guard applies to all patterns in the OR
        4 | 5 | 6 if y => println!("yes"),
        _ => println!("no"),
    }

    // More complex example
    enum Status {
        Active(u32),
        Inactive(u32),
    }

    let status = Status::Active(5);
    let threshold = 10;

    match status {
        Status::Active(level) | Status::Inactive(level) if level > threshold => {
            println!("High level: {}", level)
        }
        Status::Active(level) => println!("Active: {}", level),
        Status::Inactive(level) => println!("Inactive: {}", level),
    }
}
```

---

### Complex Guard Expressions

```rust
fn main() {
    struct User {
        name: String,
        age: u32,
        active: bool,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
        active: true,
    };

    match user {
        User { age, active: true, .. } if age >= 18 && age < 65 => {
            println!("Active adult user")
        }
        User { age, active: true, .. } if age >= 65 => {
            println!("Active senior user")
        }
        User { active: false, .. } => {
            println!("Inactive user")
        }
        _ => println!("Other"),
    }

    // Guards can call functions
    fn is_valid(x: i32) -> bool {
        x > 0 && x < 100
    }

    let number = 42;
    match number {
        x if is_valid(x) => println!("Valid number: {}", x),
        _ => println!("Invalid number"),
    }
}
```

---

## @ Bindings

The `@` operator lets you bind a value to a variable while also testing it against a pattern.

### Basic @ Bindings

```rust
fn main() {
    let number = 5;

    match number {
        n @ 1..=10 => println!("Number {} is between 1 and 10", n),
        n @ 11..=20 => println!("Number {} is between 11 and 20", n),
        _ => println!("Number is outside range"),
    }

    // Useful with Option
    let some_value = Some(42);

    match some_value {
        Some(n @ 0..=10) => println!("Small number: {}", n),
        Some(n @ 11..=100) => println!("Medium number: {}", n),
        Some(n) => println!("Large number: {}", n),
        None => println!("No number"),
    }
}
```

---

### @ with Struct Fields

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let point = Point { x: 5, y: 10 };

    match point {
        Point { x: x_val @ 0..=5, y: y_val @ 0..=10 } => {
            println!("Point in bounds: x={}, y={}", x_val, y_val)
        }
        Point { x, y } => {
            println!("Point out of bounds: x={}, y={}", x, y)
        }
    }

    // With nested patterns
    enum Message {
        Hello { id: u32 },
        Goodbye,
    }

    let msg = Message::Hello { id: 7 };

    match msg {
        Message::Hello { id: id_value @ 3..=10 } => {
            println!("ID in range: {}", id_value)
        }
        Message::Hello { id } => {
            println!("ID out of range: {}", id)
        }
        Message::Goodbye => println!("Goodbye"),
    }
}
```

---

### @ with Complex Patterns

```rust
fn main() {
    enum Color {
        Rgb(u8, u8, u8),
        Hsv(u8, u8, u8),
    }

    let color = Color::Rgb(200, 100, 50);

    match color {
        // Bind the entire RGB while also checking red component
        Color::Rgb(r @ 200..=255, g, b) => {
            println!("High red ({}) color: RGB({}, {}, {})", r, r, g, b)
        }
        Color::Rgb(r, g, b) => {
            println!("Normal RGB: ({}, {}, {})", r, g, b)
        }
        Color::Hsv(h, s, v) => {
            println!("HSV: ({}, {}, {})", h, s, v)
        }
    }

    // Multiple @ bindings
    let pair = (5, 10);
    match pair {
        (x @ 1..=5, y @ 6..=10) => {
            println!("Both in range: x={}, y={}", x, y)
        }
        _ => println!("Out of range"),
    }
}
```

---

## Reference Patterns

### ref and ref mut

When pattern matching, you can use `ref` to create a reference instead of moving/copying.

```rust
fn main() {
    let value = Some(String::from("hello"));

    // Without ref - moves the String
    // match value {
    //     Some(s) => println!("{}", s), // s owns the String
    //     None => println!("none"),
    // }
    // println!("{:?}", value); // ERROR: value was moved

    // With ref - creates a reference
    match value {
        Some(ref s) => println!("{}", s), // s is &String
        None => println!("none"),
    }
    println!("{:?}", value); // OK: value still exists

    // ref mut for mutable references
    let mut value = Some(String::from("hello"));
    match value {
        Some(ref mut s) => {
            s.push_str(" world");
            println!("{}", s);
        }
        None => println!("none"),
    }
    println!("{:?}", value); // Some("hello world")
}
```

---

### Matching References

```rust
fn main() {
    let x = 5;
    let reference = &x;

    // Match a reference
    match reference {
        &val => println!("Got value: {}", val),
    }

    // Or use ref
    match reference {
        ref r => println!("Got reference to: {}", r),
    }

    // With Some
    let num = Some(42);
    let num_ref = &num;

    match num_ref {
        Some(value) => println!("Value: {}", value),
        None => println!("None"),
    }

    // Dereferencing in patterns
    let mut numbers = vec![1, 2, 3];

    for value in &mut numbers {
        // value is &mut i32
        *value *= 2;
    }

    println!("{:?}", numbers); // [2, 4, 6]
}
```

---

### Pattern Matching with Box

```rust
fn main() {
    let boxed = Box::new(5);

    match boxed {
        box val => println!("Boxed value: {}", val),
    }

    // With nested structures
    enum List {
        Cons(i32, Box<List>),
        Nil,
    }

    use List::{Cons, Nil};

    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));

    match list {
        Cons(first, box Cons(second, _)) => {
            println!("First two: {}, {}", first, second)
        }
        Cons(first, _) => {
            println!("First: {}", first)
        }
        Nil => println!("Empty list"),
    }
}
```

---

## Slice Patterns

Slice patterns allow matching on arrays and slices.

### Fixed-Size Array Patterns

```rust
fn main() {
    let arr = [1, 2, 3];

    match arr {
        [1, 2, 3] => println!("Exact match"),
        _ => println!("No match"),
    }

    // With wildcards
    match arr {
        [1, _, 3] => println!("First is 1, last is 3"),
        _ => println!("Different pattern"),
    }

    // Destructure array
    let [first, middle, last] = arr;
    println!("{}, {}, {}", first, middle, last);
}
```

---

### Slice Patterns with Rest

```rust
fn main() {
    let slice: &[i32] = &[1, 2, 3, 4, 5];

    match slice {
        [] => println!("Empty"),
        [first] => println!("One element: {}", first),
        [first, second] => println!("Two elements: {}, {}", first, second),
        [first, .., last] => println!("First: {}, Last: {}", first, last),
    }

    // More complex patterns
    match slice {
        [1, rest @ ..] => println!("Starts with 1, rest: {:?}", rest),
        [.., 5] => println!("Ends with 5"),
        [1, .., 5] => println!("Starts with 1 and ends with 5"),
        _ => println!("Other"),
    }

    // Multiple elements with rest
    match slice {
        [first, second, rest @ ..] => {
            println!("First: {}, Second: {}, Rest: {:?}", first, second, rest)
        }
        _ => println!("Not enough elements"),
    }
}
```

---

### Matching Byte Strings

```rust
fn main() {
    let data: &[u8] = b"hello";

    match data {
        b"hello" => println!("Greeting"),
        b"goodbye" => println!("Farewell"),
        [b'h', rest @ ..] => println!("Starts with h, rest: {:?}", rest),
        _ => println!("Other"),
    }

    // Useful for parsing
    fn parse_header(header: &[u8]) -> &str {
        match header {
            [b'P', b'N', b'G', ..] => "PNG image",
            [b'G', b'I', b'F', ..] => "GIF image",
            [b'J', b'P', b'E', b'G' | b'G', ..] => "JPEG image",
            _ => "Unknown format",
        }
    }

    println!("{}", parse_header(b"PNG..."));
}
```

---

## Nested Patterns

### Deeply Nested Structures

```rust
struct Point {
    x: i32,
    y: i32,
}

enum Shape {
    Circle { center: Point, radius: f64 },
    Rectangle { top_left: Point, bottom_right: Point },
}

fn main() {
    let shape = Shape::Circle {
        center: Point { x: 0, y: 0 },
        radius: 5.0,
    };

    match shape {
        Shape::Circle {
            center: Point { x: 0, y: 0 },
            radius,
        } => {
            println!("Circle at origin with radius {}", radius)
        }
        Shape::Circle { center, radius } => {
            println!("Circle at ({}, {}) with radius {}", center.x, center.y, radius)
        }
        Shape::Rectangle { top_left, bottom_right } => {
            println!(
                "Rectangle from ({}, {}) to ({}, {})",
                top_left.x, top_left.y, bottom_right.x, bottom_right.y
            )
        }
    }
}
```

---

### Nested Enums

```rust
enum Color {
    Rgb(u8, u8, u8),
    Hsv(u8, u8, u8),
}

enum Message {
    Quit,
    ChangeColor(Color),
    Move { x: i32, y: i32 },
}

fn main() {
    let msg = Message::ChangeColor(Color::Rgb(255, 0, 0));

    match msg {
        Message::ChangeColor(Color::Rgb(r, 0, 0)) => {
            println!("Pure red: {}", r)
        }
        Message::ChangeColor(Color::Rgb(r, g, b)) => {
            println!("RGB: {}, {}, {}", r, g, b)
        }
        Message::ChangeColor(Color::Hsv(h, s, v)) => {
            println!("HSV: {}, {}, {}", h, s, v)
        }
        Message::Move { x, y } => {
            println!("Move to ({}, {})", x, y)
        }
        Message::Quit => println!("Quit"),
    }
}
```

---

### Complex Nested Patterns

```rust
fn main() {
    enum Action {
        Say(String),
        MoveTo(i32, i32),
        ChangeColor(u8, u8, u8),
    }

    let actions = vec![
        Some(Action::Say("Hello".to_string())),
        None,
        Some(Action::MoveTo(10, 20)),
        Some(Action::ChangeColor(255, 0, 0)),
    ];

    for action in &actions {
        match action {
            Some(Action::Say(msg)) if msg.len() > 0 => {
                println!("Say: {}", msg)
            }
            Some(Action::MoveTo(x, y)) if *x >= 0 && *y >= 0 => {
                println!("Move to positive coords: ({}, {})", x, y)
            }
            Some(Action::ChangeColor(r, g, b)) => {
                println!("Change to RGB({}, {}, {})", r, g, b)
            }
            None => println!("No action"),
            _ => println!("Other action"),
        }
    }
}
```

---

## Refutability

Patterns come in two forms: refutable and irrefutable.

### Irrefutable Patterns

Patterns that always match. Used in `let`, function parameters, and `for` loops.

```rust
fn main() {
    // ✅ Irrefutable - always matches
    let x = 5;
    let (a, b) = (1, 2);
    let Point { x, y } = Point { x: 3, y: 4 };

    struct Point { x: i32, y: i32 }

    // Function parameters (irrefutable)
    fn print_tuple((x, y): (i32, i32)) {
        println!("{}, {}", x, y);
    }

    // for loops (irrefutable)
    for (index, value) in vec![1, 2, 3].iter().enumerate() {
        println!("{}: {}", index, value);
    }
}
```

---

### Refutable Patterns

Patterns that can fail to match. Must be used with `if let`, `while let`, or `match`.

```rust
fn main() {
    let some_value: Option<i32> = Some(5);

    // ❌ ERROR: refutable pattern in let
    // let Some(x) = some_value;

    // ✅ Correct: use if let for refutable patterns
    if let Some(x) = some_value {
        println!("Got value: {}", x);
    }

    // ✅ Or use match
    match some_value {
        Some(x) => println!("Value: {}", x),
        None => println!("No value"),
    }

    // while let with refutable pattern
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
}
```

---

### Mixing Refutability

```rust
fn main() {
    // Can use irrefutable patterns in if let (though unnecessary)
    if let x = 5 {
        println!("{}", x); // Always executes
    }

    // More practical: use if let when you only care about one case
    let config = Some("production");

    if let Some("production") = config {
        println!("Running in production mode");
    }

    // Better than:
    match config {
        Some("production") => println!("Running in production mode"),
        _ => {} // Empty catch-all
    }
}
```

---

## Binding Modes

Rust automatically adjusts binding modes when pattern matching references.

### Automatic Dereferencing

```rust
fn main() {
    let reference = &Some(5);

    // Rust automatically dereferences
    match reference {
        Some(value) => println!("Value: {}", value),
        None => println!("None"),
    }

    // Equivalent to:
    match reference {
        &Some(ref value) => println!("Value: {}", value),
        &None => println!("None"),
    }

    // With multiple levels
    let double_ref = &&Some(10);
    match double_ref {
        Some(value) => println!("Value: {}", value), // Still works!
        None => println!("None"),
    }
}
```

---

### ref vs & Patterns

```rust
fn main() {
    let value = Some(String::from("hello"));

    // Using & pattern - matches a reference
    let reference = &value;
    match reference {
        &Some(ref s) => println!("String: {}", s),
        &None => println!("None"),
    }

    // Using ref - creates a reference
    match value {
        Some(ref s) => println!("String: {}", s), // s is &String
        None => println!("None"),
    }
    // value is still valid here

    // Without ref - moves value
    // match value {
    //     Some(s) => println!("String: {}", s), // s owns the String
    //     None => println!("None"),
    // }
    // println!("{:?}", value); // ERROR: value was moved
}
```

---

## Advanced Enum Patterns

### Matching Enum Variants with Data

```rust
#[derive(Debug)]
enum WebEvent {
    PageLoad,
    PageUnload,
    KeyPress(char),
    Paste(String),
    Click { x: i64, y: i64 },
}

fn inspect(event: WebEvent) {
    match event {
        WebEvent::PageLoad => println!("page loaded"),
        WebEvent::PageUnload => println!("page unloaded"),
        WebEvent::KeyPress(c) => println!("pressed '{}'", c),
        WebEvent::Paste(s) => println!("pasted \"{}\"", s),
        WebEvent::Click { x, y } => {
            println!("clicked at x={}, y={}", x, y);
        }
    }
}

fn main() {
    let event = WebEvent::Click { x: 20, y: 80 };
    inspect(event);

    let events = vec![
        WebEvent::PageLoad,
        WebEvent::KeyPress('x'),
        WebEvent::Paste("hello".to_string()),
        WebEvent::Click { x: 10, y: 20 },
    ];

    for event in events {
        inspect(event);
    }
}
```

---

### Enum Patterns with Result and Option

```rust
fn main() {
    // Option patterns
    let values = vec![Some(1), None, Some(2), Some(3)];

    for value in values {
        match value {
            Some(n) if n > 1 => println!("Big number: {}", n),
            Some(n) => println!("Small number: {}", n),
            None => println!("No value"),
        }
    }

    // Result patterns
    fn divide(a: f64, b: f64) -> Result<f64, String> {
        if b == 0.0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    let results = vec![
        divide(10.0, 2.0),
        divide(10.0, 0.0),
        divide(15.0, 3.0),
    ];

    for result in results {
        match result {
            Ok(value) if value > 5.0 => println!("Large result: {}", value),
            Ok(value) => println!("Result: {}", value),
            Err(e) => println!("Error: {}", e),
        }
    }
}
```

---

### Custom Enum Patterns

```rust
#[derive(Debug)]
enum Task {
    Todo { description: String, priority: u8 },
    InProgress { description: String, progress: u8 },
    Done { description: String },
}

fn main() {
    let tasks = vec![
        Task::Todo {
            description: "Write docs".to_string(),
            priority: 1,
        },
        Task::InProgress {
            description: "Review PR".to_string(),
            progress: 50,
        },
        Task::Done {
            description: "Fix bug".to_string(),
        },
    ];

    for task in tasks {
        match task {
            // High priority todos
            Task::Todo { priority: 1, description } => {
                println!("URGENT: {}", description)
            }
            // Regular todos
            Task::Todo { description, .. } => {
                println!("TODO: {}", description)
            }
            // Nearly complete
            Task::InProgress { progress: 90..=100, description } => {
                println!("Almost done: {}", description)
            }
            // In progress
            Task::InProgress { description, progress } => {
                println!("Working on: {} ({}%)", description, progress)
            }
            // Completed
            Task::Done { description } => {
                println!("Completed: {}", description)
            }
        }
    }
}
```

---

## Real-World Patterns

### Parsing Command-Line Arguments

```rust
fn main() {
    let args = vec!["program", "run", "--verbose", "file.txt"];

    let mut verbose = false;
    let mut file = None;
    let mut command = None;

    let mut iter = args.iter().skip(1);

    while let Some(arg) = iter.next() {
        match *arg {
            "run" | "build" | "test" => command = Some(*arg),
            "--verbose" | "-v" => verbose = true,
            filename if !filename.starts_with("-") => file = Some(filename),
            _ => println!("Unknown argument: {}", arg),
        }
    }

    match (command, file) {
        (Some(cmd), Some(f)) => {
            println!("Command: {}, File: {}, Verbose: {}", cmd, f, verbose)
        }
        (Some(cmd), None) => {
            println!("Command: {}, Verbose: {}", cmd, verbose)
        }
        _ => println!("Invalid arguments"),
    }
}
```

---

### State Machine Pattern

```rust
enum State {
    Idle,
    Running { progress: u8 },
    Paused { progress: u8 },
    Completed,
    Failed { error: String },
}

struct Machine {
    state: State,
}

impl Machine {
    fn new() -> Self {
        Machine { state: State::Idle }
    }

    fn start(&mut self) {
        match self.state {
            State::Idle => {
                self.state = State::Running { progress: 0 };
                println!("Started");
            }
            State::Paused { progress } => {
                self.state = State::Running { progress };
                println!("Resumed from {}%", progress);
            }
            _ => println!("Cannot start from current state"),
        }
    }

    fn pause(&mut self) {
        match self.state {
            State::Running { progress } => {
                self.state = State::Paused { progress };
                println!("Paused at {}%", progress);
            }
            _ => println!("Can only pause when running"),
        }
    }

    fn update(&mut self, amount: u8) {
        match &mut self.state {
            State::Running { progress } if *progress + amount >= 100 => {
                self.state = State::Completed;
                println!("Completed!");
            }
            State::Running { progress } => {
                *progress += amount;
                println!("Progress: {}%", progress);
            }
            _ => println!("Not running"),
        }
    }
}

fn main() {
    let mut machine = Machine::new();
    machine.start();
    machine.update(30);
    machine.pause();
    machine.start();
    machine.update(80);
}
```

---

### JSON-like Data Structure

```rust
#[derive(Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

fn print_json(value: &JsonValue, indent: usize) {
    let spaces = " ".repeat(indent);

    match value {
        JsonValue::Null => print!("null"),
        JsonValue::Bool(b) => print!("{}", b),
        JsonValue::Number(n) => print!("{}", n),
        JsonValue::String(s) => print!("\"{}\"", s),
        JsonValue::Array(arr) => {
            println!("[");
            for (i, item) in arr.iter().enumerate() {
                print!("{}", " ".repeat(indent + 2));
                print_json(item, indent + 2);
                if i < arr.len() - 1 {
                    println!(",");
                } else {
                    println!();
                }
            }
            print!("{}]", spaces);
        }
        JsonValue::Object(obj) => {
            println!("{{");
            for (i, (key, val)) in obj.iter().enumerate() {
                print!("{}\"{}\": ", " ".repeat(indent + 2), key);
                print_json(val, indent + 2);
                if i < obj.len() - 1 {
                    println!(",");
                } else {
                    println!();
                }
            }
            print!("{}}}", spaces);
        }
    }
}

fn main() {
    let data = JsonValue::Object(vec![
        ("name".to_string(), JsonValue::String("Alice".to_string())),
        ("age".to_string(), JsonValue::Number(30.0)),
        ("active".to_string(), JsonValue::Bool(true)),
        (
            "tags".to_string(),
            JsonValue::Array(vec![
                JsonValue::String("rust".to_string()),
                JsonValue::String("programming".to_string()),
            ]),
        ),
    ]);

    print_json(&data, 0);
    println!();
}
```

---

### Error Handling Pattern

```rust
#[derive(Debug)]
enum AppError {
    IoError(String),
    ParseError(String),
    ValidationError { field: String, message: String },
    NotFound,
}

fn process_file(path: &str) -> Result<String, AppError> {
    // Simulate file reading
    if path.is_empty() {
        return Err(AppError::ValidationError {
            field: "path".to_string(),
            message: "Path cannot be empty".to_string(),
        });
    }

    if path == "missing" {
        return Err(AppError::NotFound);
    }

    Ok("file contents".to_string())
}

fn main() {
    let paths = vec!["file.txt", "", "missing", "data.json"];

    for path in paths {
        match process_file(path) {
            Ok(content) => {
                println!("Read from {}: {}", path, content)
            }
            Err(AppError::ValidationError { field, message }) => {
                println!("Validation error in '{}': {}", field, message)
            }
            Err(AppError::NotFound) => {
                println!("File not found: {}", path)
            }
            Err(AppError::IoError(msg)) => {
                println!("IO error: {}", msg)
            }
            Err(AppError::ParseError(msg)) => {
                println!("Parse error: {}", msg)
            }
        }
    }
}
```

---

## Performance Considerations

### Match vs if-else Chains

```rust
fn main() {
    let value = 42;

    // ✅ Match - compiler can optimize better
    let result = match value {
        0 => "zero",
        1..=10 => "small",
        11..=100 => "medium",
        _ => "large",
    };

    // ❌ if-else chain - less optimal
    let result = if value == 0 {
        "zero"
    } else if value >= 1 && value <= 10 {
        "small"
    } else if value >= 11 && value <= 100 {
        "medium"
    } else {
        "large"
    };
}
```

---

### Pattern Matching is Zero-Cost

Pattern matching compiles to efficient machine code, often as efficient as manual pointer manipulation.

```rust
enum Operation {
    Add(i32, i32),
    Subtract(i32, i32),
    Multiply(i32, i32),
}

fn execute(op: Operation) -> i32 {
    // Compiles to efficient jump table or branch
    match op {
        Operation::Add(a, b) => a + b,
        Operation::Subtract(a, b) => a - b,
        Operation::Multiply(a, b) => a * b,
    }
}

fn main() {
    let result = execute(Operation::Add(5, 3));
    println!("Result: {}", result);
}
```

---

### Avoid Repeated Matching

```rust
fn main() {
    let value = Some(42);

    // ❌ Less efficient - matches twice
    if let Some(x) = value {
        if x > 10 {
            println!("Large: {}", x);
        }
    }

    // ✅ Better - match once with guard
    match value {
        Some(x) if x > 10 => println!("Large: {}", x),
        _ => {}
    }

    // ❌ Inefficient
    let data = vec![Some(1), Some(2), None, Some(3)];
    for item in &data {
        if item.is_some() {
            let value = item.unwrap();
            println!("{}", value);
        }
    }

    // ✅ Efficient
    for item in &data {
        if let Some(value) = item {
            println!("{}", value);
        }
    }
}
```

---

## Common Pitfalls

### Pitfall 1: Unreachable Patterns

```rust
fn main() {
    let x = 5;

    match x {
        _ => println!("anything"),
        // ❌ WARNING: unreachable pattern
        5 => println!("five"),
    }

    // ✅ Correct - specific patterns first
    match x {
        5 => println!("five"),
        _ => println!("anything"),
    }
}
```

---

### Pitfall 2: Forgetting to Handle All Cases

```rust
enum Status {
    Active,
    Inactive,
    Pending,
}

fn main() {
    let status = Status::Active;

    // ❌ ERROR: non-exhaustive patterns
    // match status {
    //     Status::Active => println!("active"),
    //     Status::Inactive => println!("inactive"),
    //     // Missing: Pending
    // }

    // ✅ Correct
    match status {
        Status::Active => println!("active"),
        Status::Inactive => println!("inactive"),
        Status::Pending => println!("pending"),
    }
}
```

---

### Pitfall 3: Moving Values Unintentionally

```rust
fn main() {
    let s = Some(String::from("hello"));

    // ❌ Moves the String
    match s {
        Some(text) => println!("{}", text),
        None => println!("none"),
    }
    // println!("{:?}", s); // ERROR: s was moved

    let s = Some(String::from("hello"));

    // ✅ Use ref to avoid moving
    match s {
        Some(ref text) => println!("{}", text),
        None => println!("none"),
    }
    println!("{:?}", s); // OK

    // ✅ Or use as_ref()
    let s = Some(String::from("hello"));
    match s.as_ref() {
        Some(text) => println!("{}", text),
        None => println!("none"),
    }
    println!("{:?}", s); // OK
}
```

---

### Pitfall 4: Overlapping Ranges

```rust
fn main() {
    let x = 5;

    // ❌ Confusing - which pattern matches 5?
    match x {
        1..=5 => println!("1 to 5"),   // This matches first
        3..=7 => println!("3 to 7"),   // Unreachable for 3-5
        _ => println!("other"),
    }

    // ✅ Better - non-overlapping ranges
    match x {
        1..=2 => println!("1 to 2"),
        3..=5 => println!("3 to 5"),
        6..=7 => println!("6 to 7"),
        _ => println!("other"),
    }
}
```

---

### Pitfall 5: Incorrect Guard Scope

```rust
fn main() {
    let x = 4;
    let y = false;

    // Guard applies to ALL patterns in the OR
    match x {
        4 | 5 | 6 if y => println!("yes"), // Doesn't match (y is false)
        _ => println!("no"),
    }

    // This is NOT the same as:
    // match x {
    //     (4 if y) | 5 | 6 => println!("yes"), // Doesn't work!
    //     _ => println!("no"),
    // }
}
```

---

### Pitfall 6: Using == Instead of Patterns

```rust
fn main() {
    let value = Some(42);

    // ❌ Less idiomatic
    if value == Some(42) {
        println!("Found 42");
    }

    // ✅ More idiomatic - use pattern matching
    if let Some(42) = value {
        println!("Found 42");
    }

    // Or with match
    match value {
        Some(42) => println!("Found 42"),
        _ => {}
    }
}
```

---

### Pitfall 7: Mutability in Patterns

```rust
fn main() {
    let mut value = Some(10);

    // ❌ This doesn't make the binding mutable
    match value {
        Some(x) => {
            // x += 1; // ERROR: cannot assign to immutable
        }
        None => {}
    }

    // ✅ Use ref mut
    match value {
        Some(ref mut x) => {
            *x += 1; // OK
        }
        None => {}
    }

    println!("{:?}", value); // Some(11)
}
```

---

### Pitfall 8: Confusing if let with Match

```rust
fn main() {
    let number = Some(7);

    // if let only matches one pattern
    if let Some(x) = number {
        println!("Got: {}", x);
    } else {
        println!("Nothing");
    }

    // ❌ Cannot do multiple if let in one statement
    // if let Some(x) = number | None = number {
    //     ...
    // }

    // ✅ Use match for multiple patterns
    match number {
        Some(x) => println!("Got: {}", x),
        None => println!("Nothing"),
    }
}
```

---

## Quick Reference

### Pattern Types

| Pattern | Example | Description |
|---------|---------|-------------|
| Literal | `42`, `"hello"` | Match exact value |
| Variable | `x`, `name` | Bind to variable |
| Wildcard | `_` | Ignore value |
| Rest | `..` | Ignore remaining items |
| Range | `1..=10`, `'a'..='z'` | Match range of values |
| OR | `1 \| 2 \| 3` | Match any of several patterns |
| Tuple | `(x, y)`, `(1, _)` | Destructure tuple |
| Array | `[a, b, c]` | Destructure array |
| Slice | `[first, .., last]` | Match slice pattern |
| Struct | `Point { x, y }` | Destructure struct |
| Enum | `Some(x)`, `Ok(val)` | Match enum variant |
| Reference | `&val`, `&mut val` | Match reference |
| Ref binding | `ref x`, `ref mut x` | Create reference |
| Box | `box val` | Match boxed value |
| @ binding | `x @ 1..=10` | Bind while matching |
| Guard | `x if x > 5` | Additional condition |

---

### Pattern Locations

| Location | Refutable? | Example |
|----------|------------|---------|
| `let` | No | `let (x, y) = pair;` |
| Function params | No | `fn f((x, y): (i32, i32))` |
| `for` loops | No | `for (k, v) in map` |
| `match` arms | Yes | `match x { Some(n) => n }` |
| `if let` | Yes | `if let Some(x) = opt` |
| `while let` | Yes | `while let Some(x) = iter.next()` |

---

### Common Patterns by Use Case

**Extracting Option values:**
```rust
if let Some(value) = option { ... }
match option {
    Some(value) => ...,
    None => ...,
}
```

**Handling Results:**
```rust
match result {
    Ok(value) => ...,
    Err(error) => ...,
}
```

**Destructuring structs:**
```rust
let Point { x, y } = point;
match point {
    Point { x: 0, y } => ...,
    Point { x, y } => ...,
}
```

**Iterating with patterns:**
```rust
for (key, value) in map { ... }
for (index, item) in vec.iter().enumerate() { ... }
```

**Guards for additional conditions:**
```rust
match value {
    Some(x) if x > 10 => ...,
    Some(x) => ...,
    None => ...,
}
```

---

## Summary

Pattern matching in Rust provides:

* ✅ **Exhaustiveness checking** - Compiler ensures all cases are handled
* ✅ **Type safety** - Patterns are checked at compile time
* ✅ **Destructuring** - Extract values from complex types elegantly
* ✅ **Zero cost** - Compiles to efficient machine code
* ✅ **Expressiveness** - Clear, concise code
* ✅ **Safety** - Prevents common bugs like null pointer dereferences

Master pattern matching to write safer, more elegant Rust code!
