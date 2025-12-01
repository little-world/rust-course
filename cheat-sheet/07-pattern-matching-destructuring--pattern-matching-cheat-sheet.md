

### Pattern Matching Cheat Sheet

```rust 
// ===== BASIC MATCH =====
// Simple match expression
let number = 5;
match number {
    1 => println!("One"),
    2 => println!("Two"),
    3 | 4 | 5 => println!("Three, four, or five"),    // Multiple patterns
    _ => println!("Something else"),                   // Catch-all
}

// Match returns a value
let description = match number {
    1 => "one",
    2 => "two",
    _ => "other",
};

// Match with blocks
let result = match number {
    1 => {
        println!("Processing one");
        "one"
    }
    2 => {
        println!("Processing two");
        "two"
    }
    _ => "other",
};

// ===== MATCHING ENUMS =====
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

let msg = Message::Write(String::from("hello"));

match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(text) => println!("Write: {}", text),
    Message::ChangeColor(r, g, b) => println!("Color: ({}, {}, {})", r, g, b),
}

// ===== MATCHING OPTION =====
let some_value = Some(5);

match some_value {
    Some(x) => println!("Got: {}", x),
    None => println!("Nothing"),
}

// Matching nested Option
let nested = Some(Some(5));
match nested {
    Some(Some(x)) => println!("Nested value: {}", x),
    Some(None) => println!("Outer Some, inner None"),
    None => println!("Outer None"),
}

// ===== MATCHING RESULT =====
let result: Result<i32, &str> = Ok(42);

match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => println!("Error: {}", e),
}

// ===== IF LET =====
// Concise match for single pattern
let some_value = Some(5);

if let Some(x) = some_value {
    println!("Got: {}", x);
}

// if let with else
if let Some(x) = some_value {
    println!("Got: {}", x);
} else {
    println!("Nothing");
}

// if let else if let else
let value = Some(3);
if let Some(x) = value {
    if x < 5 {
        println!("Small: {}", x);
    }
} else if let None = value {
    println!("Nothing");
} else {
    println!("Something else");
}

// ===== WHILE LET =====
// Loop while pattern matches
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}

// ===== LET ELSE =====
// Pattern match or early return
fn process(input: Option<i32>) -> i32 {
    let Some(value) = input else {
        return 0;                                      // Early return if None
    };
    value * 2
}

// Multiple let else
fn parse_config(data: &str) -> Result<Config, Error> {
    let Some(first_line) = data.lines().next() else {
        return Err(Error::Empty);
    };
    
    let Ok(value) = first_line.parse::<i32>() else {
        return Err(Error::ParseError);
    };
    
    Ok(Config { value })
}

// ===== DESTRUCTURING =====
// Destructure tuples
let tuple = (1, "hello", 3.14);
let (x, y, z) = tuple;
println!("{} {} {}", x, y, z);

// Destructure structs
struct Point {
    x: i32,
    y: i32,
}

let point = Point { x: 10, y: 20 };
let Point { x, y } = point;
println!("x: {}, y: {}", x, y);

// Destructure with different names
let Point { x: a, y: b } = point;
println!("a: {}, b: {}", a, b);

// Destructure nested structs
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

let rect = Rectangle {
    top_left: Point { x: 0, y: 10 },
    bottom_right: Point { x: 10, y: 0 },
};

let Rectangle {
    top_left: Point { x: x1, y: y1 },
    bottom_right: Point { x: x2, y: y2 },
} = rect;

// ===== MATCH GUARDS =====
// Additional conditions in match arms
let num = Some(4);

match num {
    Some(x) if x < 5 => println!("Less than 5: {}", x),
    Some(x) => println!("Greater or equal to 5: {}", x),
    None => println!("None"),
}

// Multiple patterns with guard
let x = 4;
let y = false;

match x {
    4 | 5 | 6 if y => println!("yes"),              // Guard applies to all patterns
    _ => println!("no"),
}

// ===== RANGE PATTERNS =====
// Match ranges (inclusive)
let x = 5;

match x {
    1..=5 => println!("One through five"),          // Inclusive range
    6..=10 => println!("Six through ten"),
    _ => println!("Something else"),
}

// Character ranges
let c = 'c';

match c {
    'a'..='j' => println!("Early letter"),
    'k'..='z' => println!("Late letter"),
    _ => println!("Not a letter"),
}

// ===== @ BINDINGS =====
// Bind value while testing pattern
let msg = Message::Move { x: 10, y: 20 };

match msg {
    Message::Move { x: x @ 10..=20, y } => {
        println!("x is in range 10-20: {}, y: {}", x, y);
    }
    Message::Move { x, y } => {
        println!("x: {}, y: {}", x, y);
    }
    _ => {}
}

// @ with enum variants
enum Status {
    Active(u32),
    Inactive,
}

let status = Status::Active(5);

match status {
    Status::Active(code @ 0..=10) => {
        println!("Low code: {}", code);
    }
    Status::Active(code) => {
        println!("High code: {}", code);
    }
    Status::Inactive => println!("Inactive"),
}

// ===== IGNORING VALUES =====
// Ignore entire value with _
let value = Some(5);
match value {
    Some(_) => println!("Got some value"),
    None => println!("Got none"),
}

// Ignore parts of value
let tuple = (1, 2, 3, 4);
let (first, _, third, _) = tuple;

// Ignore remaining parts with ..
let numbers = (1, 2, 3, 4, 5);
let (first, .., last) = numbers;                   // first = 1, last = 5

// Ignore multiple struct fields
struct Point3D { x: i32, y: i32, z: i32 }
let point = Point3D { x: 1, y: 2, z: 3 };
let Point3D { x, .. } = point;                     // Only extract x

// ===== REF AND REF MUT =====
// Borrow instead of move in pattern
let value = Some(String::from("hello"));

match value {
    Some(ref s) => println!("Got: {}", s),          // Borrow s
    None => println!("None"),
}
println!("{:?}", value);                            // value still valid

// Mutable reference
let mut value = Some(String::from("hello"));

match value {
    Some(ref mut s) => s.push_str(" world"),        // Mutable borrow
    None => {}
}

// ===== MATCHING SLICES =====
// Match array/slice patterns
let arr = [1, 2, 3];

match arr {
    [a, b, c] => println!("{}, {}, {}", a, b, c),
}

// Match with rest pattern
let slice = &[1, 2, 3, 4, 5][..];

match slice {
    [first, second, ..] => println!("First: {}, Second: {}", first, second),
    [] => println!("Empty"),
}

match slice {
    [first, .., last] => println!("First: {}, Last: {}", first, last),
    [] => println!("Empty"),
}

match slice {
    [first, middle @ .., last] => {
        println!("First: {}, Middle: {:?}, Last: {}", first, middle, last);
    }
    [] => println!("Empty"),
}

// ===== MATCHES! MACRO =====
// Check if pattern matches
let value = Some(5);
let is_some = matches!(value, Some(_));             // true

let number = 4;
let in_range = matches!(number, 1..=5);            // true

// More complex pattern
enum Status {
    Active { code: u32 },
    Inactive,
}

let status = Status::Active { code: 200 };
let is_ok = matches!(status, Status::Active { code: 200..=299 });

// ===== NESTED PATTERNS =====
// Complex nested matching
enum Color {
    Rgb(u8, u8, u8),
    Hsv(u8, u8, u8),
}

enum Message2 {
    Quit,
    Move { x: i32, y: i32 },
    ChangeColor(Color),
}

let msg = Message2::ChangeColor(Color::Rgb(0, 160, 255));

match msg {
    Message2::ChangeColor(Color::Rgb(r, g, b)) => {
        println!("RGB: {}, {}, {}", r, g, b);
    }
    Message2::ChangeColor(Color::Hsv(h, s, v)) => {
        println!("HSV: {}, {}, {}", h, s, v);
    }
    Message2::Move { x, y } => {
        println!("Move to ({}, {})", x, y);
    }
    _ => {}
}

// ===== OR PATTERNS =====
// Multiple patterns in one arm
let x = 1;

match x {
    1 | 2 => println!("One or two"),
    3 | 4 | 5 => println!("Three, four, or five"),
    _ => println!("Something else"),
}

// Or patterns in if let
if let Some(1) | Some(2) = Some(1) {
    println!("One or two");
}

// Or patterns with binding
match x {
    1 | 2 => println!("Small"),
    n @ 3..=10 => println!("Medium: {}", n),
    n => println!("Large: {}", n),
}

// ===== MATCH ERGONOMICS =====
// Automatic dereferencing and borrowing
let value = &Some(5);

match value {
    Some(x) => println!("{}", x),                   // x is &i32, automatic deref
    None => println!("None"),
}

// Works with nested references
let nested = &&Some(5);
match nested {
    Some(x) => println!("{}", x),                   // Automatic double deref
    None => println!("None"),
}

// ===== IRREFUTABLE PATTERNS =====
// Patterns that always match (used in let, for, function params)
let (x, y, z) = (1, 2, 3);                         // Always matches

for (key, value) in map.iter() {                   // Always matches
    println!("{}: {}", key, value);
}

fn print_point((x, y): (i32, i32)) {               // Always matches
    println!("({}, {})", x, y);
}

// ===== REFUTABLE PATTERNS =====
// Patterns that might not match (used in if let, while let, match)
if let Some(x) = some_value {                      // Might not match
    println!("{}", x);
}

// ===== COMMON PATTERNS =====
// Pattern 1: Matching with side effects
let result = match compute() {
    Ok(value) => {
        log("Success");
        value
    }
    Err(e) => {
        log_error(&e);
        default_value()
    }
};

// Pattern 2: State machine
enum State {
    Idle,
    Processing { progress: u32 },
    Done { result: String },
}

fn transition(state: State, event: Event) -> State {
    match (state, event) {
        (State::Idle, Event::Start) => State::Processing { progress: 0 },
        (State::Processing { progress }, Event::Update) => {
            State::Processing { progress: progress + 10 }
        }
        (State::Processing { .. }, Event::Complete(result)) => {
            State::Done { result }
        }
        (s, _) => s,                                // No transition
    }
}

// Pattern 3: Exhaustive enum handling
#[derive(Debug)]
enum Action {
    Read,
    Write,
    Delete,
}

fn handle(action: Action) {
    match action {
        Action::Read => read(),
        Action::Write => write(),
        Action::Delete => delete(),
        // Compiler ensures all variants are handled
    }
}

// Pattern 4: Option unwrapping with default
let value = some_option.unwrap_or_else(|| {
    match calculate_default() {
        Some(x) => x,
        None => 0,
    }
});

// Pattern 5: Nested Result/Option handling
let result: Result<Option<i32>, Error> = get_data();

match result {
    Ok(Some(value)) => println!("Got: {}", value),
    Ok(None) => println!("No data"),
    Err(e) => println!("Error: {}", e),
}

// Pattern 6: Match as expression in chain
let result = load_data()
    .and_then(|data| match validate(data) {
        Ok(d) => Ok(d),
        Err(e) => Err(e),
    });

// Pattern 7: Pattern matching in closures
let vec = vec![Some(1), None, Some(3)];
let filtered: Vec<_> = vec.into_iter()
    .filter_map(|x| match x {
        Some(n) if n > 1 => Some(n * 2),
        _ => None,
    })
    .collect();

// Pattern 8: Struct update with matching
let point = Point { x: 10, y: 20 };
let new_point = match point {
    Point { x, .. } if x > 5 => Point { x: x * 2, ..point },
    _ => point,
};

// Pattern 9: Guard with multiple conditions
let pair = (2, -3);
match pair {
    (x, y) if x > 0 && y > 0 => println!("Both positive"),
    (x, y) if x < 0 && y < 0 => println!("Both negative"),
    (x, y) if x * y < 0 => println!("Different signs"),
    _ => println!("At least one is zero"),
}

// Pattern 10: Matching borrowed vs owned
fn process(data: &Option<String>) {
    match data {
        Some(s) => println!("Got: {}", s),          // s is &String
        None => println!("None"),
    }
}

fn process_owned(data: Option<String>) {
    match data {
        Some(s) => println!("Got: {}", s),          // s is String (moved)
        None => println!("None"),
    }
}
```
