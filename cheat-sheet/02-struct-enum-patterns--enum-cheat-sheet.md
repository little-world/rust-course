

### Enum Cheat Sheet

```rust
// ===== BASIC ENUMS =====
// Simple enum
enum Direction {
    North,
    South,
    East,
    West,
}

fn basic_enum_example() {
    let dir = Direction::North;
    
    match dir {
        Direction::North => println!("Going north"),
        Direction::South => println!("Going south"),
        Direction::East => println!("Going east"),
        Direction::West => println!("Going west"),
    }
}

// ===== ENUMS WITH DATA =====
// Each variant can hold different types and amounts of data
enum Message {
    Quit,                                                // No data
    Move { x: i32, y: i32 },                            // Named fields
    Write(String),                                       // Single value
    ChangeColor(u8, u8, u8),                            // Multiple values
}

fn enum_with_data_example() {
    let msg1 = Message::Quit;
    let msg2 = Message::Move { x: 10, y: 20 };
    let msg3 = Message::Write(String::from("Hello"));
    let msg4 = Message::ChangeColor(255, 0, 0);
}

// ===== PATTERN MATCHING =====
fn process_message(msg: Message) {
    match msg {
        Message::Quit => {
            println!("Quitting");
        }
        Message::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        Message::Write(text) => {
            println!("Writing: {}", text);
        }
        Message::ChangeColor(r, g, b) => {
            println!("Changing color to RGB({}, {}, {})", r, g, b);
        }
    }
}

// Match with guards
fn match_with_guard(msg: Message) {
    match msg {
        Message::Move { x, y } if x > 0 && y > 0 => {
            println!("Moving to positive quadrant");
        }
        Message::Move { x, y } => {
            println!("Moving to ({}, {})", x, y);
        }
        _ => {}
    }
}

// ===== OPTION ENUM =====
// Option<T> is defined in standard library as:
// enum Option<T> {
//     Some(T),
//     None,
// }

fn option_example() {
    let some_number = Some(5);
    let some_string = Some("Hello");
    let absent_number: Option<i32> = None;
    
    // Pattern matching
    match some_number {
        Some(n) => println!("Number: {}", n),
        None => println!("No number"),
    }
    
    // if let
    if let Some(n) = some_number {
        println!("Got number: {}", n);
    }
    
    // Unwrap (panics if None)
    let n = some_number.unwrap();
    
    // Unwrap with default
    let n = absent_number.unwrap_or(0);
    
    // Unwrap with closure
    let n = absent_number.unwrap_or_else(|| 10);
    
    // Map
    let doubled = some_number.map(|n| n * 2);
    
    // And then (flatMap)
    let result = some_number.and_then(|n| Some(n * 2));
}

// ===== RESULT ENUM =====
// Result<T, E> is defined as:
// enum Result<T, E> {
//     Ok(T),
//     Err(E),
// }

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

fn result_example() {
    let result = divide(10, 2);
    
    // Pattern matching
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(e) => println!("Error: {}", e),
    }
    
    // Unwrap (panics on Err)
    let value = divide(10, 2).unwrap();
    
    // Expect (custom panic message)
    let value = divide(10, 2).expect("Division failed");
    
    // Unwrap or default
    let value = divide(10, 0).unwrap_or(0);
    
    // Question mark operator
    fn propagate_error() -> Result<i32, String> {
        let result = divide(10, 2)?;                     // Returns Err early
        Ok(result * 2)
    }
    
    // Map
    let doubled = divide(10, 2).map(|n| n * 2);
    
    // Map error
    let result = divide(10, 0).map_err(|e| format!("Error: {}", e));
}

// ===== ENUM METHODS =====
impl Message {
    fn call(&self) {
        match self {
            Message::Quit => println!("Quit called"),
            Message::Move { x, y } => println!("Move to ({}, {})", x, y),
            Message::Write(text) => println!("Write: {}", text),
            Message::ChangeColor(r, g, b) => println!("Color: RGB({}, {}, {})", r, g, b),
        }
    }
    
    fn is_quit(&self) -> bool {
        matches!(self, Message::Quit)
    }
}

fn enum_methods_example() {
    let msg = Message::Write(String::from("Hello"));
    msg.call();
    println!("Is quit: {}", msg.is_quit());
}

// ===== GENERIC ENUMS =====
enum Result2<T, E> {
    Ok(T),
    Err(E),
}

enum Option2<T> {
    Some(T),
    None,
}

// Multiple generic parameters
enum Either<L, R> {
    Left(L),
    Right(R),
}

fn generic_enum_example() {
    let left: Either<i32, String> = Either::Left(42);
    let right: Either<i32, String> = Either::Right(String::from("Hello"));
    
    match left {
        Either::Left(n) => println!("Number: {}", n),
        Either::Right(s) => println!("String: {}", s),
    }
}

// ===== IF LET =====
fn if_let_example() {
    let some_value = Some(3);
    
    // Instead of match
    if let Some(n) = some_value {
        println!("Number: {}", n);
    }
    
    // With else
    if let Some(n) = some_value {
        println!("Number: {}", n);
    } else {
        println!("No number");
    }
    
    // Multiple if let
    let msg = Message::Move { x: 10, y: 20 };
    
    if let Message::Move { x, y } = msg {
        println!("Move to ({}, {})", x, y);
    } else if let Message::Write(text) = msg {
        println!("Write: {}", text);
    } else {
        println!("Other message");
    }
}

// ===== WHILE LET =====
fn while_let_example() {
    let mut stack = vec![1, 2, 3, 4, 5];
    
    while let Some(top) = stack.pop() {
        println!("{}", top);
    }
}

// ===== MATCHES! MACRO =====
fn matches_example() {
    let msg = Message::Write(String::from("Hello"));
    
    // Check if matches pattern
    if matches!(msg, Message::Write(_)) {
        println!("It's a Write message");
    }
    
    // With guard
    let num = Some(4);
    if matches!(num, Some(x) if x < 5) {
        println!("Number is less than 5");
    }
}

// ===== ENUM DISCRIMINANTS =====
#[repr(u8)]
enum Status {
    Active = 1,
    Inactive = 2,
    Pending = 3,
}

fn discriminant_example() {
    let status = Status::Active;
    let value = status as u8;
    println!("Status value: {}", value);
}

// ===== C-LIKE ENUMS =====
#[repr(C)]
enum Color {
    Red = 0xFF0000,
    Green = 0x00FF00,
    Blue = 0x0000FF,
}

#[repr(i32)]
enum ErrorCode {
    Success = 0,
    NotFound = -1,
    PermissionDenied = -2,
    InternalError = -500,
}

// ===== DERIVE MACROS FOR ENUMS =====
#[derive(Debug)]
enum LogLevel {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Copy)]
enum Direction2 {
    North,
    South,
    East,
    West,
}

#[derive(PartialEq, Eq)]
enum State {
    Active,
    Inactive,
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Priority {
    Low,
    Medium,
    High,
}

fn derive_example() {
    let level = LogLevel::Info;
    println!("{:?}", level);
    
    let state1 = State::Active;
    let state2 = State::Active;
    println!("Equal: {}", state1 == state2);
    
    let p1 = Priority::Low;
    let p2 = Priority::High;
    println!("p1 < p2: {}", p1 < p2);
}

// ===== ENUMS WITH LIFETIMES =====
enum Cow<'a> {
    Borrowed(&'a str),
    Owned(String),
}

fn lifetime_enum_example() {
    let borrowed = Cow::Borrowed("Hello");
    let owned = Cow::Owned(String::from("World"));
}

// ===== RECURSIVE ENUMS =====
// Requires Box for indirection
enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn new() -> Self {
        List::Nil
    }
    
    fn prepend(self, elem: i32) -> Self {
        List::Cons(elem, Box::new(self))
    }
    
    fn len(&self) -> usize {
        match self {
            List::Cons(_, tail) => 1 + tail.len(),
            List::Nil => 0,
        }
    }
}

fn recursive_enum_example() {
    let list = List::new()
        .prepend(1)
        .prepend(2)
        .prepend(3);
    
    println!("Length: {}", list.len());
}

// Binary tree
enum Tree<T> {
    Leaf(T),
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

// ===== ENUM AS STATE MACHINE =====
enum ConnectionState {
    Disconnected,
    Connecting { retry_count: u32 },
    Connected { session_id: String },
    Error { message: String },
}

impl ConnectionState {
    fn connect(self) -> Self {
        match self {
            ConnectionState::Disconnected => {
                ConnectionState::Connecting { retry_count: 0 }
            }
            _ => self,
        }
    }
    
    fn establish(self, session_id: String) -> Self {
        match self {
            ConnectionState::Connecting { .. } => {
                ConnectionState::Connected { session_id }
            }
            _ => self,
        }
    }
    
    fn disconnect(self) -> Self {
        ConnectionState::Disconnected
    }
}

// ===== ENUM WITH ASSOCIATED DATA PATTERNS =====
// Linked list
enum LinkedList<T> {
    Empty,
    Node(T, Box<LinkedList<T>>),
}

// JSON-like value
#[derive(Debug)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(std::collections::HashMap<String, JsonValue>),
}

fn json_example() {
    use std::collections::HashMap;
    
    let mut obj = HashMap::new();
    obj.insert(String::from("name"), JsonValue::String(String::from("Alice")));
    obj.insert(String::from("age"), JsonValue::Number(30.0));
    obj.insert(String::from("active"), JsonValue::Bool(true));
    
    let json = JsonValue::Object(obj);
    println!("{:?}", json);
}

// ===== NESTED ENUMS =====
enum OuterMessage {
    Inner(InnerMessage),
    Other,
}

enum InnerMessage {
    Data(i32),
    Text(String),
}

fn nested_enum_example() {
    let msg = OuterMessage::Inner(InnerMessage::Data(42));
    
    match msg {
        OuterMessage::Inner(InnerMessage::Data(n)) => {
            println!("Data: {}", n);
        }
        OuterMessage::Inner(InnerMessage::Text(s)) => {
            println!("Text: {}", s);
        }
        OuterMessage::Other => {
            println!("Other");
        }
    }
}

// ===== ENUM SIZE AND LAYOUT =====
fn enum_size() {
    println!("Option<i32> size: {}", std::mem::size_of::<Option<i32>>());
    println!("Result<i32, String> size: {}", std::mem::size_of::<Result<i32, String>>());
    
    // Enum size is size of largest variant + discriminant
    enum Large {
        Small(u8),
        Large([u8; 100]),
    }
    
    println!("Large enum size: {}", std::mem::size_of::<Large>());
}

// ===== ENUM DESTRUCTURING =====
fn destructure_enum() {
    let msg = Message::ChangeColor(255, 0, 0);
    
    // Destructure in match
    match msg {
        Message::ChangeColor(r, g, b) => {
            println!("R: {}, G: {}, B: {}", r, g, b);
        }
        _ => {}
    }
    
    // Destructure in let
    if let Message::ChangeColor(r, g, b) = msg {
        println!("R: {}, G: {}, B: {}", r, g, b);
    }
    
    // Destructure struct variant
    let msg = Message::Move { x: 10, y: 20 };
    if let Message::Move { x, y } = msg {
        println!("X: {}, Y: {}", x, y);
    }
}

// ===== ENUM CONVERSIONS =====
// From/Into implementations
impl From<bool> for Status {
    fn from(active: bool) -> Self {
        if active {
            Status::Active
        } else {
            Status::Inactive
        }
    }
}

fn conversion_example() {
    let status: Status = true.into();
}

// TryFrom for fallible conversions
use std::convert::TryFrom;

impl TryFrom<i32> for Status {
    type Error = String;
    
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Status::Active),
            2 => Ok(Status::Inactive),
            3 => Ok(Status::Pending),
            _ => Err(format!("Invalid status code: {}", value)),
        }
    }
}

// ===== NON-EXHAUSTIVE ENUMS =====
#[non_exhaustive]
pub enum ApiError {
    NotFound,
    Unauthorized,
    InternalError,
}

// Users must use _ pattern to handle future variants
fn handle_error(error: ApiError) {
    match error {
        ApiError::NotFound => println!("Not found"),
        ApiError::Unauthorized => println!("Unauthorized"),
        ApiError::InternalError => println!("Internal error"),
        _ => println!("Unknown error"),                 // Required for non_exhaustive
    }
}

// ===== COMMON PATTERNS =====

// Pattern 1: Option chaining
fn option_chaining() {
    let value = Some(5)
        .map(|x| x * 2)
        .and_then(|x| if x > 5 { Some(x) } else { None })
        .unwrap_or(0);
    
    println!("Value: {}", value);
}

// Pattern 2: Result chaining
fn result_chaining() -> Result<i32, String> {
    divide(10, 2)?
        .checked_mul(3)
        .ok_or_else(|| "Overflow".to_string())
}

// Pattern 3: Custom error enum
#[derive(Debug)]
enum AppError {
    IoError(std::io::Error),
    ParseError(std::num::ParseIntError),
    CustomError(String),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(error: std::num::ParseIntError) -> Self {
        AppError::ParseError(error)
    }
}

fn app_function() -> Result<i32, AppError> {
    let contents = std::fs::read_to_string("file.txt")?;
    let number = contents.trim().parse::<i32>()?;
    Ok(number)
}

// Pattern 4: Event system
#[derive(Debug)]
enum Event {
    KeyPress(char),
    MouseClick { x: i32, y: i32 },
    Resize { width: u32, height: u32 },
    Quit,
}

fn handle_event(event: Event) {
    match event {
        Event::KeyPress(c) => println!("Key pressed: {}", c),
        Event::MouseClick { x, y } => println!("Mouse clicked at ({}, {})", x, y),
        Event::Resize { width, height } => println!("Resized to {}x{}", width, height),
        Event::Quit => println!("Quitting"),
    }
}

// Pattern 5: Command pattern
enum Command {
    Create { name: String },
    Update { id: u32, name: String },
    Delete { id: u32 },
    List,
}

impl Command {
    fn execute(&self) {
        match self {
            Command::Create { name } => println!("Creating: {}", name),
            Command::Update { id, name } => println!("Updating {} to {}", id, name),
            Command::Delete { id } => println!("Deleting {}", id),
            Command::List => println!("Listing all"),
        }
    }
}

// Pattern 6: Parser result
enum ParseResult<T> {
    Success(T, usize),                                   // value, bytes consumed
    Incomplete(usize),                                   // bytes needed
    Error(String),
}

// Pattern 7: Cow-like enum
enum MaybeOwned<'a> {
    Borrowed(&'a str),
    Owned(String),
}

impl<'a> MaybeOwned<'a> {
    fn as_str(&self) -> &str {
        match self {
            MaybeOwned::Borrowed(s) => s,
            MaybeOwned::Owned(s) => s.as_str(),
        }
    }
}

// Pattern 8: Validation result
enum Validation<T, E> {
    Valid(T),
    Invalid(Vec<E>),
}

impl<T, E> Validation<T, E> {
    fn is_valid(&self) -> bool {
        matches!(self, Validation::Valid(_))
    }
    
    fn errors(&self) -> Option<&Vec<E>> {
        match self {
            Validation::Invalid(errors) => Some(errors),
            _ => None,
        }
    }
}

// Pattern 9: Notification system
#[derive(Debug)]
enum Notification {
    Email { to: String, subject: String, body: String },
    Sms { to: String, message: String },
    Push { device_id: String, title: String, body: String },
}

impl Notification {
    fn send(&self) {
        match self {
            Notification::Email { to, subject, .. } => {
                println!("Sending email to {} with subject: {}", to, subject);
            }
            Notification::Sms { to, message } => {
                println!("Sending SMS to {}: {}", to, message);
            }
            Notification::Push { device_id, title, .. } => {
                println!("Sending push to {}: {}", device_id, title);
            }
        }
    }
}

// Pattern 10: AST (Abstract Syntax Tree)
enum Expr {
    Number(i32),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
}

impl Expr {
    fn eval(&self) -> Result<i32, String> {
        match self {
            Expr::Number(n) => Ok(*n),
            Expr::Add(left, right) => Ok(left.eval()? + right.eval()?),
            Expr::Subtract(left, right) => Ok(left.eval()? - right.eval()?),
            Expr::Multiply(left, right) => Ok(left.eval()? * right.eval()?),
            Expr::Divide(left, right) => {
                let r = right.eval()?;
                if r == 0 {
                    Err("Division by zero".to_string())
                } else {
                    Ok(left.eval()? / r)
                }
            }
        }
    }
}

fn ast_example() {
    // (2 + 3) * 4
    let expr = Expr::Multiply(
        Box::new(Expr::Add(
            Box::new(Expr::Number(2)),
            Box::new(Expr::Number(3)),
        )),
        Box::new(Expr::Number(4)),
    );
    
    println!("Result: {}", expr.eval().unwrap());
}
```
