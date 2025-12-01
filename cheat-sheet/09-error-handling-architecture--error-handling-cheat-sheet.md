### Error Handling Cheat Sheet
```rust
// ===== RESULT TYPE =====
// Result enum definition
enum Result<T, E> {
    Ok(T),
    Err(E),
}

// Basic Result usage
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(String::from("Division by zero"))
    } else {
        Ok(a / b)
    }
}

// Handling Result with match
match divide(10, 2) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}

// ===== RESULT METHODS =====
let result: Result<i32, &str> = Ok(42);

result.is_ok()                                      // Check if Ok
result.is_err()                                     // Check if Err
result.ok()                                         // Convert to Option<T>, discards error
result.err()                                        // Convert to Option<E>, discards value

result.unwrap()                                     // Get value or panic
result.expect("Custom panic message")               // Unwrap with custom message
result.unwrap_or(0)                                 // Get value or default
result.unwrap_or_else(|e| 0)                       // Get value or compute default
result.unwrap_or_default()                         // Get value or T::default()

result.map(|x| x * 2)                              // Transform Ok value
result.map_err(|e| format!("Error: {}", e))        // Transform Err value
result.and_then(|x| Ok(x * 2))                     // Chain operations (flatMap)
result.or_else(|e| Ok(0))                          // Provide alternative

// ===== QUESTION MARK OPERATOR (?) =====
// Propagate errors automatically
fn read_username() -> Result<String, io::Error> {
    let mut file = File::open("user.txt")?;        // Returns Err if fails
    let mut username = String::new();
    file.read_to_string(&mut username)?;           // Returns Err if fails
    Ok(username)
}

// Without ?: verbose
fn read_username_verbose() -> Result<String, io::Error> {
    let file = File::open("user.txt");
    let mut file = match file {
        Ok(f) => f,
        Err(e) => return Err(e),
    };
    let mut username = String::new();
    match file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
}

// ? with Option
fn last_char(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()            // Propagate None
}

// ===== CUSTOM ERROR TYPES =====
// Simple custom error
#[derive(Debug)]
struct MyError {
    message: String,
}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MyError {}

// Using custom error
fn my_function() -> Result<i32, MyError> {
    Err(MyError {
        message: String::from("Something went wrong"),
    })
}

// Enum for multiple error types
#[derive(Debug)]
enum AppError {
    IoError(io::Error),
    ParseError(std::num::ParseIntError),
    CustomError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "IO error: {}", e),
            AppError::ParseError(e) => write!(f, "Parse error: {}", e),
            AppError::CustomError(s) => write!(f, "Custom error: {}", s),
        }
    }
}

impl std::error::Error for AppError {}

// Automatic conversion with From trait
impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::ParseError(err)
    }
}

// Now ? works with automatic conversion
fn process_file() -> Result<i32, AppError> {
    let mut file = File::open("data.txt")?;        // Converts io::Error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;           // Converts io::Error
    let number = contents.trim().parse::<i32>()?;  // Converts ParseIntError
    Ok(number)
}

// ===== THISERROR CRATE =====
use thiserror::Error;

#[derive(Error, Debug)]
enum DataError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Invalid data at line {line}: {message}")]
    InvalidData { line: usize, message: String },
}

// ===== ANYHOW CRATE =====
use anyhow::{Context, Result, anyhow, bail};

// Simple error handling
fn load_config() -> Result<Config> {
    let file = File::open("config.toml")
        .context("Failed to open config file")?;   // Add context
    
    let config: Config = toml::from_reader(file)
        .context("Failed to parse config")?;
    
    Ok(config)
}

// Create ad-hoc errors
fn validate(value: i32) -> Result<()> {
    if value < 0 {
        bail!("Value must be non-negative");        // Early return with error
    }
    if value > 100 {
        return Err(anyhow!("Value {} too large", value)); // Create error
    }
    Ok(())
}

// ===== OPTION TYPE =====
// Option enum definition
enum Option<T> {
    Some(T),
    None,
}

// Basic Option usage
fn find_user(id: u32) -> Option<User> {
    if id == 1 {
        Some(User { name: "Alice".into() })
    } else {
        None
    }
}

// ===== OPTION METHODS =====
let opt: Option<i32> = Some(42);

opt.is_some()                                       // Check if Some
opt.is_none()                                       // Check if None
opt.unwrap()                                        // Get value or panic
opt.expect("No value present")                     // Unwrap with message
opt.unwrap_or(0)                                   // Get value or default
opt.unwrap_or_else(|| 0)                          // Get value or compute
opt.unwrap_or_default()                           // Get value or T::default()

opt.map(|x| x * 2)                                // Transform Some value
opt.and_then(|x| Some(x * 2))                     // Chain operations (flatMap)
opt.or(Some(0))                                    // Provide alternative
opt.or_else(|| Some(0))                           // Compute alternative

opt.filter(|x| *x > 10)                           // Filter value
opt.take()                                         // Take value, leave None
opt.replace(10)                                    // Replace value, return old

// Convert between Option and Result
opt.ok_or("Error message")                         // Option -> Result
opt.ok_or_else(|| "Error")                        // With closure
result.ok()                                        // Result -> Option

// ===== COMBINING OPTIONS =====
let a = Some(5);
let b = Some(10);

// Zip options
a.zip(b)                                           // Some((5, 10))
a.zip(b).map(|(x, y)| x + y)                      // Some(15)

// Transpose Option<Result> to Result<Option>
let opt_result: Option<Result<i32, &str>> = Some(Ok(5));
let result_opt: Result<Option<i32>, &str> = opt_result.transpose();

// ===== PANIC =====
// Unrecoverable errors
panic!("Something went wrong");                     // Panic with message
panic!("Error: {}", error_message);                // Formatted panic

// Assert macros
assert!(condition);                                 // Panic if false
assert!(2 + 2 == 4);
assert_eq!(left, right);                           // Panic if not equal
assert_eq!(2 + 2, 4);
assert_ne!(left, right);                           // Panic if equal
assert_ne!(2 + 2, 5);

// Debug assertions (only in debug builds)
debug_assert!(expensive_check());
debug_assert_eq!(a, b);
debug_assert_ne!(a, b);

// Unreachable code
match value {
    1 => "one",
    2 => "two",
    _ => unreachable!("Value should be 1 or 2"),
}

// Unimplemented
fn future_feature() {
    unimplemented!("This feature is not yet implemented");
}

// Todo
fn work_in_progress() {
    todo!("Implement this later");
}

// ===== CATCH UNWIND =====
use std::panic;

// Catch panics
let result = panic::catch_unwind(|| {
    panic!("Oops!");
});

match result {
    Ok(_) => println!("No panic"),
    Err(_) => println!("Caught panic"),
}

// Set panic hook
panic::set_hook(Box::new(|panic_info| {
    eprintln!("Custom panic handler: {:?}", panic_info);
}));

// ===== COMMON PATTERNS =====
// Pattern 1: Early return with ?
fn process() -> Result<String, io::Error> {
    let file = File::open("data.txt")?;
    let reader = BufReader::new(file);
    let first_line = reader.lines().next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Empty file"))??;
    Ok(first_line)
}

// Pattern 2: Map error types
fn parse_config(path: &str) -> Result<Config, AppError> {
    let contents = fs::read_to_string(path)
        .map_err(|e| AppError::IoError(e))?;
    
    let config = toml::from_str(&contents)
        .map_err(|e| AppError::ParseError(format!("{}", e)))?;
    
    Ok(config)
}

// Pattern 3: Collect Results
let results: Vec<Result<i32, _>> = vec!["1", "2", "three"]
    .iter()
    .map(|s| s.parse::<i32>())
    .collect();

// Short-circuit on first error
let numbers: Result<Vec<i32>, _> = vec!["1", "2", "3"]
    .iter()
    .map(|s| s.parse::<i32>())
    .collect();

// Pattern 4: Fallback chain
let value = option1
    .or(option2)
    .or(option3)
    .unwrap_or(default);

// Pattern 5: Transform and validate
fn validate_age(age_str: &str) -> Result<u32, String> {
    let age: u32 = age_str.parse()
        .map_err(|_| "Invalid number".to_string())?;
    
    if age < 18 {
        return Err("Too young".to_string());
    }
    if age > 120 {
        return Err("Invalid age".to_string());
    }
    
    Ok(age)
}

// Pattern 6: Logging errors
fn process_with_logging() -> Result<(), AppError> {
    let result = risky_operation();
    
    if let Err(ref e) = result {
        eprintln!("Error occurred: {}", e);
        // Log or handle error
    }
    
    result
}

// Pattern 7: Retry on error
fn retry<F, T, E>(mut f: F, max_attempts: u32) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut attempts = 0;
    loop {
        match f() {
            Ok(val) => return Ok(val),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

// Pattern 8: Multiple error handling
fn process_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("data.txt")?;            // io::Error
    let data: Data = serde_json::from_reader(file)?; // serde error
    let number = data.value.parse::<i32>()?;       // ParseIntError
    Ok(())
}

// Pattern 9: Contextual errors with anyhow
use anyhow::Context;

fn load_data(path: &str) -> anyhow::Result<Data> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open {}", path))?;
    
    let data = serde_json::from_reader(file)
        .context("Failed to parse JSON")?;
    
    Ok(data)
}

// Pattern 10: Option combinators
let result = get_user(id)
    .and_then(|user| get_profile(user.id))
    .and_then(|profile| get_settings(profile.id))
    .map(|settings| settings.theme)
    .unwrap_or_default();

// Pattern 11: Error from string
fn custom_error() -> Result<(), Box<dyn std::error::Error>> {
    Err("Something went wrong".into())             // String -> Box<Error>
}

// Pattern 12: Propagate with context
fn read_config() -> anyhow::Result<Config> {
    let path = std::env::var("CONFIG_PATH")
        .context("CONFIG_PATH not set")?;
    
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path))?;
    
    toml::from_str(&contents)
        .context("Failed to parse TOML")
}

// Pattern 13: Partition results
let (successes, failures): (Vec<_>, Vec<_>) = results
    .into_iter()
    .partition(Result::is_ok);

let successes: Vec<_> = successes.into_iter()
    .map(Result::unwrap)
    .collect();

// Pattern 14: Early exit from main
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;
    let data = fetch_data(&config)?;
    process(data)?;
    Ok(())
}

// Pattern 15: Custom Result type alias
type AppResult<T> = Result<T, AppError>;

fn operation() -> AppResult<String> {
    // ...
}
```