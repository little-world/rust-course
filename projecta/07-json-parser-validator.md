# JSON Parser

## Problem Statement

Build a JSON parser and schema validator that uses Rust's pattern matching to parse JSON strings into an AST and validate against type schemas. You'll implement recursive pattern matching for nested structures, exhaustive enum matching for all JSON types, pattern guards for validation rules, and deep destructuring for complex document traversal.

---

## Understanding JSON Syntax

Before diving into parsing, let's understand JSON (JavaScript Object Notation) - a lightweight data interchange format that's easy for humans to read and write, and easy for machines to parse and generate.

### JSON Data Types

JSON supports exactly **six data types**:

#### 1. **Null**
Represents an empty or non-existent value.

```json
null
```

**Properties:**
- Only one possible value: `null`
- Often used to indicate "no value" or "unknown"
- Case-sensitive (must be lowercase)

#### 2. **Boolean**
Logical true or false values.

```json
true
false
```

**Properties:**
- Only two possible values: `true` or `false`
- Case-sensitive (must be lowercase)
- Not quoted (not strings)

#### 3. **Number**
Numeric values including integers and floating-point numbers.

```json
42
-17
3.14159
2.5e10
-1.23e-4
```

**Properties:**
- No distinction between integer and float in JSON spec
- Can be negative (prefix with `-`)
- Can use scientific notation (`e` or `E`)
- No octal or hexadecimal notation
- No leading zeros (except for `0.something`)
- No `NaN` or `Infinity` (not valid JSON)

**Examples:**
```json
0           // Valid
-42         // Valid
3.14        // Valid
1.5e3       // Valid (1500)
-2.5e-2     // Valid (-0.025)
```

#### 4. **String**
Sequence of Unicode characters wrapped in double quotes.

```json
"hello"
"Hello, World!"
"Line 1\nLine 2"
"Unicode: \u0048\u0065\u006C\u006C\u006F"
```

**Properties:**
- **Must** use double quotes (not single quotes)
- Can contain escape sequences:
  - `\"` - double quote
  - `\\` - backslash
  - `\/` - forward slash
  - `\b` - backspace
  - `\f` - form feed
  - `\n` - newline
  - `\r` - carriage return
  - `\t` - tab
  - `\uXXXX` - Unicode character (4 hex digits)
- Cannot contain unescaped control characters

**Examples:**
```json
"simple"                    // Valid
"with \"quotes\""           // Valid (escaped quotes)
"path\\to\\file"            // Valid (escaped backslashes)
"tab\there"                 // Valid (tab character)
"unicode: \u0041"           // Valid (produces "unicode: A")
'single quotes'             // INVALID (must use double quotes)
"unescaped
newline"                    // INVALID (newlines must be escaped)
```

#### 5. **Array**
Ordered list of values (can be of mixed types).

```json
[1, 2, 3]
["apple", "banana", "cherry"]
[true, 42, "mixed", null]
[
  [1, 2],
  [3, 4]
]
[]
```

**Properties:**
- Enclosed in square brackets `[ ]`
- Values separated by commas `,`
- Can contain any JSON value type
- Can be nested (arrays within arrays)
- Can be empty
- **Trailing commas not allowed**

**Examples:**
```json
[1, 2, 3]                  // Valid
[]                         // Valid (empty array)
[1, "two", true, null]     // Valid (mixed types)
[[1, 2], [3, 4]]          // Valid (nested)
[1, 2, 3,]                // INVALID (trailing comma)
```

#### 6. **Object**
Unordered collection of key-value pairs.

```json
{
  "name": "John",
  "age": 30,
  "isStudent": false
}
```

**Properties:**
- Enclosed in curly braces `{ }`
- Keys **must** be strings (in double quotes)
- Key-value pairs separated by colons `:`
- Pairs separated by commas `,`
- Keys must be unique within an object
- Can contain any JSON value type as values
- Can be nested
- **Trailing commas not allowed**

**Examples:**
```json
// Simple object
{
  "name": "Alice",
  "age": 25
}

// Nested objects
{
  "person": {
    "name": "Bob",
    "address": {
      "city": "NYC",
      "zip": "10001"
    }
  }
}

// Mixed values
{
  "id": 123,
  "active": true,
  "tags": ["rust", "programming"],
  "metadata": null
}

// Empty object
{}

// INVALID examples:
{ name: "value" }          // Keys must be quoted
{ "key": "value", }        // Trailing comma not allowed
{ "a": 1, "a": 2 }        // Duplicate keys (undefined behavior)
```

---

### JSON Grammar Rules

#### Whitespace
JSON ignores whitespace between tokens:
- Space ` `
- Tab `\t`
- Newline `\n`
- Carriage return `\r`

```json
// These are equivalent:
{"name":"value"}
{
  "name": "value"
}
{ "name" : "value" }
```

#### Valid JSON Documents

A JSON document **must** have exactly one root value:

```json
// Valid - single object
{ "key": "value" }

// Valid - single array
[1, 2, 3]

// Valid - single string
"hello"

// INVALID - multiple root values
{ "a": 1 } { "b": 2 }

// INVALID - just a comma
,
```

---

### JSON Examples in Context of This Project

#### Example 1: Simple User Object
```json
{
  "id": 42,
  "username": "alice",
  "email": "alice@example.com",
  "isActive": true,
  "lastLogin": null
}
```

**Structure breakdown:**
- Root: Object with 5 key-value pairs
- Keys: All strings
- Values: Number, String, String, Boolean, Null

#### Example 2: Nested Configuration
```json
{
  "server": {
    "host": "localhost",
    "port": 8080,
    "tls": {
      "enabled": true,
      "cert": "/path/to/cert.pem"
    }
  },
  "features": ["logging", "metrics", "auth"]
}
```

**Structure breakdown:**
- Root: Object
- Nested: 2 levels deep (server → tls)
- Array: Contains strings
- Mixed types: Objects, Strings, Numbers, Booleans, Arrays

#### Example 3: Array of Objects
```json
[
  {
    "name": "Task 1",
    "completed": true,
    "priority": 1
  },
  {
    "name": "Task 2",
    "completed": false,
    "priority": 2
  }
]
```

**Structure breakdown:**
- Root: Array containing objects
- Each object: Same structure (homogeneous)
- Useful for: Lists of records, database results, API responses

#### Example 4: Complex Nested Structure
```json
{
  "users": [
    {
      "id": 1,
      "profile": {
        "name": "Alice",
        "settings": {
          "theme": "dark",
          "notifications": true
        }
      },
      "posts": [
        { "title": "First Post", "likes": 10 },
        { "title": "Second Post", "likes": 25 }
      ]
    }
  ],
  "metadata": {
    "version": "1.0",
    "timestamp": 1234567890
  }
}
```

**Structure breakdown:**
- Root: Object
- Maximum nesting depth: 4 levels
- Mixed types throughout: Objects, Arrays, Numbers, Strings, Booleans

---

### Common JSON Pitfalls

1. **Single Quotes**: ❌ `{'key': 'value'}` → ✅ `{"key": "value"}`
2. **Unquoted Keys**: ❌ `{key: "value"}` → ✅ `{"key": "value"}`
3. **Trailing Commas**: ❌ `[1, 2, 3,]` → ✅ `[1, 2, 3]`
4. **Comments**: ❌ `{"key": "value" /* comment */}` → JSON has no comments
5. **Undefined**: ❌ `{"key": undefined}` → ✅ `{"key": null}`
6. **Multiple Roots**: ❌ `{}{}` → ✅ `[{}, {}]` or just one `{}`

---


## Key Concepts Explained

This project teaches advanced Rust pattern matching techniques through JSON parsing and validation. These concepts are essential for building type-safe, maintainable parsers and data processors.

### 1. Exhaustive Enum Matching

Rust's `match` expressions **must handle all cases** of an enum:

```rust
enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

fn type_name(value: &Value) -> &str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        // Compiler error if we miss a case!
    }
}
```

**Why it matters**: The compiler catches all missing cases at compile time. If you add a new enum variant, every `match` expression that doesn't handle it becomes a compile error, forcing you to update all affected code.

**vs C/C++ switch**:
```c
// C code - compiles even with missing cases
switch (type) {
    case NULL_TYPE:   return "null";
    case BOOL_TYPE:   return "boolean";
    // Forgot to handle NUMBER_TYPE - no error, undefined behavior at runtime!
}
```

### 2. Recursive Types

Enums can contain themselves, enabling tree structures:

```rust
pub enum Value {
    Array(Vec<Value>),      // Array contains Values
    Object(HashMap<String, Value>),  // Object contains Values
    // ...
}
```

**Why it matters**: JSON is inherently recursive (arrays contain values, values can be arrays). Rust's type system directly models this:

```json
{
  "users": [
    {"name": "Alice", "tags": ["admin", "dev"]},
    {"name": "Bob", "tags": ["user"]}
  ]
}
```

This nests 4 levels: Object → Array → Object → Array. The `Value` enum naturally represents any depth.

**Memory layout**:
```rust
// Value enum is 32 bytes (on 64-bit):
// - 8 bytes: discriminant (which variant)
// - 24 bytes: largest variant data (Vec or HashMap)

// All variants fit in same size:
Value::Null              // 8 bytes used
Value::Bool(true)        // 9 bytes used
Value::Number(3.14)      // 16 bytes used
Value::Array(vec)        // 32 bytes used (Vec = ptr + len + cap)
```

### 3. Pattern Guards

Add conditions to match arms with `if`:

```rust
fn validate_number(value: &Value, min: f64, max: f64) -> bool {
    match value {
        Value::Number(n) if *n >= min && *n <= max => true,
        Value::Number(_) => false,  // Number outside range
        _ => false,  // Not a number
    }
}
```

**Why it matters**: Combine type checking with constraint validation in one expression:

```rust
// Without guards - verbose
match value {
    Value::Number(n) => {
        if *n >= 0.0 && *n <= 100.0 {
            println!("Valid percentage");
        } else {
            println!("Out of range");
        }
    }
    _ => println!("Not a number"),
}

// With guards - concise
match value {
    Value::Number(n) if *n >= 0.0 && *n <= 100.0 => println!("Valid percentage"),
    Value::Number(_) => println!("Out of range"),
    _ => println!("Not a number"),
}
```

### 4. Let-Else Pattern

Extract values or early return:

```rust
pub fn as_string(&self) -> Option<&str> {
    let Value::String(s) = self else {
        return None;
    };
    Some(s)
}
```

**Why it matters**: Cleaner than nested if-let or match:

```rust
// Without let-else
pub fn as_string(&self) -> Option<&str> {
    if let Value::String(s) = self {
        Some(s)
    } else {
        None
    }
}

// With let-else - more direct
pub fn as_string(&self) -> Option<&str> {
    let Value::String(s) = self else { return None };
    Some(s)
}
```

**Pattern**: "Extract this shape or bail out" - common in parsers and validators.

### 5. Deep Destructuring

Extract nested data in one pattern:

```rust
// Extract from nested JSON:
// {"user": {"address": {"city": "NYC"}}}

match value {
    Value::Object(map) => {
        match map.get("user") {
            Some(Value::Object(user)) => {
                match user.get("address") {
                    Some(Value::Object(addr)) => {
                        match addr.get("city") {
                            Some(Value::String(city)) => println!("City: {}", city),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    _ => {}
}

// Same thing with method chaining
if let Some(Value::String(city)) = value
    .get("user")
    .and_then(|v| v.get("address"))
    .and_then(|v| v.get("city"))
{
    println!("City: {}", city);
}
```

**Why it matters**: JSON often nests deeply. Pattern matching or chaining provides safe navigation without null pointer errors.

### 6. Or-Patterns

Match multiple patterns in one arm:

```rust
match token {
    Token::True | Token::False => {
        // Handle both boolean tokens
        Value::Bool(matches!(token, Token::True))
    }
    Token::LeftBrace | Token::LeftBracket => {
        // Both start compound structures
        parse_compound()
    }
    _ => parse_simple(),
}
```

**Why it matters**: Avoid code duplication when multiple cases have identical handling:

```rust
// Without or-patterns - repeated code
match schema {
    Schema::Null => check_null(value),
    Schema::Bool => check_bool(value),
    Schema::Number { .. } => check_number(value, constraints),
    Schema::String { .. } => check_string(value, constraints),
    Schema::Array { .. } => check_array(value, constraints),
    Schema::Object { .. } => check_object(value, constraints),
}

// With or-patterns - group simple types
match schema {
    Schema::Null | Schema::Bool => check_simple(value, schema),
    Schema::Number { .. } | Schema::String { .. } => check_constrained(value, schema),
    Schema::Array { .. } | Schema::Object { .. } => check_recursive(value, schema),
}
```

### 7. matches! Macro

Check if a value matches a pattern without extracting:

```rust
// Without matches!
fn is_number(value: &Value) -> bool {
    match value {
        Value::Number(_) => true,
        _ => false,
    }
}

// With matches! - one line
fn is_number(value: &Value) -> bool {
    matches!(value, Value::Number(_))
}

// Even shorter with method
impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }
}
```

**Why it matters**: Concise type checking in validators and filters:

```rust
// Filter to only numbers
let numbers: Vec<_> = values.iter()
    .filter(|v| matches!(v, Value::Number(_)))
    .collect();

// Validate all array elements are objects
let all_objects = array.iter().all(|v| matches!(v, Value::Object(_)));
```

### 8. Recursive Descent Parsing

Parse nested structures by calling parsing functions recursively:

```rust
fn parse_value(&mut self) -> Result<Value, Error> {
    match self.current_token {
        Token::LeftBracket => self.parse_array(),  // Recursive
        Token::LeftBrace => self.parse_object(),    // Recursive
        Token::String(s) => Ok(Value::String(s)),   // Base case
        Token::Number(n) => Ok(Value::Number(n)),   // Base case
        _ => Err(Error::UnexpectedToken),
    }
}

fn parse_array(&mut self) -> Result<Value, Error> {
    let mut elements = Vec::new();
    loop {
        elements.push(self.parse_value()?);  // Recursion!
        // Handle comma/closing bracket...
    }
    Ok(Value::Array(elements))
}
```

**Why it matters**: The parser structure mirrors the data structure:
- JSON arrays → `parse_array()` recursively calls `parse_value()`
- JSON objects → `parse_object()` recursively calls `parse_value()`
- Simple values → base case, no recursion

**Stack usage**: Deep nesting can cause stack overflow:
```json
[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]
```

**Solution**: Track depth and error on excessive nesting:
```rust
fn parse_value_with_depth(&mut self, depth: usize) -> Result<Value, Error> {
    if depth > MAX_DEPTH {
        return Err(Error::TooDeep);
    }
    // Recursive calls pass depth + 1
}
```

### 9. Error Handling with Result

Propagate errors up the call stack:

```rust
pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(input)?;  // ? operator propagates errors
    parser.parse_value()?
}

fn parse_object(&mut self) -> Result<Value, ParseError> {
    self.expect(Token::LeftBrace)?;  // Propagate if wrong token
    let mut map = HashMap::new();

    while !self.check(Token::RightBrace) {
        let key = self.parse_string()?;  // Propagate if not string
        self.expect(Token::Colon)?;       // Propagate if no colon
        let value = self.parse_value()?;  // Propagate if invalid value
        map.insert(key, value);
    }

    Ok(Value::Object(map))
}
```

**Why it matters**: Errors flow naturally without explicit checks at every level:

```c
// C-style error handling (verbose)
Value* parse_object(Parser* p) {
    if (!expect(p, LEFT_BRACE)) return NULL;
    Map* map = map_new();
    if (!map) return NULL;

    while (!check(p, RIGHT_BRACE)) {
        char* key = parse_string(p);
        if (!key) { map_free(map); return NULL; }

        if (!expect(p, COLON)) { free(key); map_free(map); return NULL; }

        Value* val = parse_value(p);
        if (!val) { free(key); map_free(map); return NULL; }

        map_insert(map, key, val);
    }
    return value_object(map);
}
```

Rust's `?` operator handles cleanup automatically via `Drop`.

### 10. Builder Pattern for Complex Types

Construct objects incrementally:

```rust
let schema = Schema::object()
    .required_property("name", Schema::string().min_length(1))
    .required_property("age", Schema::integer().min(0.0).max(120.0))
    .property("email", Schema::string())
    .allow_additional()
    .build();
```

**Why it matters**: Readable, type-safe construction of complex configurations:

```rust
// Without builder - verbose struct construction
let schema = Schema::Object {
    properties: {
        let mut props = HashMap::new();
        props.insert("name".to_string(), PropertySchema {
            schema: Schema::String {
                min_length: Some(1),
                max_length: None,
                pattern: None,
            },
            required: true,
        });
        props.insert("age".to_string(), PropertySchema {
            schema: Schema::Number {
                min: Some(0.0),
                max: Some(120.0),
                integer_only: true,
            },
            required: true,
        });
        props
    },
    required: vec!["name".to_string(), "age".to_string()],
    additional_properties: true,
};

// With builder - clear and concise
let schema = Schema::object()
    .required_property("name", Schema::string().min_length(1))
    .required_property("age", Schema::integer().min(0.0).max(120.0))
    .allow_additional()
    .build();
```

### 11. Type State Pattern

Use types to enforce correct usage order:

```rust
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token()?;
        Ok(Parser { lexer, current_token })
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_token = self.lexer.next_token()?;
        Ok(())
    }
}
```

**Why it matters**: Parser always has a current token after construction. No "uninitialized state" possible:

```rust
// Can't create parser without initializing current_token
let parser = Parser { lexer, current_token: ??? };  // ❌ Can't compile

// Must use constructor
let parser = Parser::new(input)?;  // ✅ Always valid
```

### 12. Trait Objects for Extensibility

Allow custom validators:

```rust
pub trait Validator: fmt::Debug {
    fn validate(&self, value: &Value) -> Result<(), String>;
}

#[derive(Debug)]
pub struct EmailValidator;

impl Validator for EmailValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let Value::String(s) = value else {
            return Err("expected string".to_string());
        };
        if s.contains('@') {
            Ok(())
        } else {
            Err("invalid email".to_string())
        }
    }
}

// Schema can hold any validator
pub enum Schema {
    Custom(Box<dyn Validator>),
    // ...
}
```

**Why it matters**: Users can add custom validation without modifying the schema enum:

```rust
// Built-in validators
let email_schema = Schema::Custom(Box::new(EmailValidator));
let url_schema = Schema::Custom(Box::new(UrlValidator));

// User-defined validator
struct ApiKeyValidator;
impl Validator for ApiKeyValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // Custom logic
    }
}
let api_key_schema = Schema::Custom(Box::new(ApiKeyValidator));
```

---

## Connection to This Project

Here's how each concept maps to the milestones you'll implement:

### Milestone 1: JSON Value Representation and Basic Parsing

**Concepts applied**:
- **Exhaustive enum matching**: Every `Token` matched in `parse_value()`
- **Recursive types**: `Value` enum contains itself via `Array(Vec<Value>)`
- **Let-else pattern**: Extract token data safely
- **matches! macro**: Implement type-checking methods like `is_null()`
- **Error propagation**: `Result<Value, ParseError>` throughout

**Why this matters**: Type-safe representation prevents bugs:

```rust
// ❌ Without enums - runtime errors
struct Value {
    type_tag: i32,
    data: *mut void,  // Unsafe! What type is this?
}

// ✅ With enums - compile-time safety
enum Value {
    Number(f64),      // Can only be f64
    String(String),   // Can only be String
    // Impossible to misuse
}
```

**Real-world impact**: A production API gateway parsing JSON:
- **Without type safety**: Invalid JSON accepted, crashes at runtime when accessing wrong type
- **With exhaustive matching**: Invalid JSON rejected immediately with clear error

**Performance**: Enum dispatch via `match` compiles to **jump table** (O(1) lookup), same speed as C switch but type-safe.

---

### Milestone 2: Recursive Parsing for Arrays and Objects

**Concepts applied**:
- **Recursive descent parsing**: `parse_array()` calls `parse_value()` recursively
- **Pattern matching on tokens**: Match `[` for arrays, `{` for objects
- **Depth tracking**: Prevent stack overflow from malicious input
- **Error handling**: Propagate parse errors with context

**Why this matters**: Safe handling of nested structures:

```rust
// Parse arbitrarily nested JSON
fn parse_array(&mut self) -> Result<Value, ParseError> {
    let mut items = Vec::new();
    loop {
        items.push(self.parse_value()?);  // Recursive call
        match self.current_token {
            Token::Comma => self.advance()?,
            Token::RightBracket => break,
            _ => return Err(ParseError::Expected("comma or ]")),
        }
    }
    Ok(Value::Array(items))
}
```

**Real-world impact**: Parsing configuration files with nested objects:

```json
{
  "server": {
    "endpoints": [
      {"path": "/api", "handlers": [{"method": "GET", "fn": "handle_get"}]}
    ]
  }
}
```

**Without recursion**: Would need manual stack management (complex, error-prone)
**With recursion**: Parser structure mirrors JSON structure (simple, correct)

**Stack safety**:
- Naive parser: 100-level nesting = 100 stack frames → ~8KB stack
- Protected parser: Max depth limit prevents stack overflow attacks

**Benchmarks** (1000 nested arrays):
- Without depth limit: Stack overflow crash
- With depth limit: Graceful error in ~1μs

---

### Milestone 3: Schema Definition and Type Validation

**Concepts applied**:
- **Pattern guards**: Validate constraints like `Value::Number(n) if *n >= min`
- **Recursive validation**: Validate nested arrays/objects
- **Builder pattern**: Construct complex schemas fluently
- **Method chaining**: `Schema::number().min(0).max(100)`

**Why this matters**: Runtime type checking with clear errors:

```rust
// Schema defines expectations
let schema = Schema::object()
    .required_property("age", Schema::integer().min(0).max(120))
    .build();

// Validation catches violations
let json = r#"{"age": -5}"#;
let value = parse(json)?;

match schema.validate(&value) {
    Ok(()) => process(value),
    Err(e) => {
        // Error: "Validation error at $.age: value -5 below minimum 0"
        eprintln!("{}", e);
    }
}
```

**Real-world impact**: API request validation before database insertion:

```rust
// User registration endpoint
let schema = Schema::object()
    .required_property("email", Schema::custom(EmailValidator))
    .required_property("password", Schema::string().min_length(8))
    .required_property("age", Schema::integer().min(13))
    .build();

// Validate before saving
schema.validate(&request_body)?;
database.insert_user(request_body)?;
```

**Without validation**:
- Invalid email → DB insert fails (late detection)
- Short password → Security vulnerability
- Negative age → Data corruption

**With validation**:
- Invalid data rejected at API boundary (early detection)
- Clear error messages guide users
- Database always receives valid data

**Performance comparison** (validating 10,000 objects):
- No validation: 0ms (no checking)
- Runtime validation: ~50ms (type checks + constraints)
- **Cost**: 5μs per object
- **Benefit**: Prevents 100% of type errors, 90%+ of constraint violations

---

### Milestone 4: Deep Destructuring and Path Queries

**Concepts applied**:
- **Deep destructuring**: Navigate nested JSON with pattern matching
- **Option chaining**: Safe navigation with `?` operator
- **Path queries**: JSONPath-like querying with wildcards
- **Recursive search**: Find all matching values in tree

**Why this matters**: Extract data from complex JSON without brittleness:

```rust
// Manual navigation - fragile
let city = if let Value::Object(root) = &json {
    if let Some(Value::Object(user)) = root.get("user") {
        if let Some(Value::Object(addr)) = user.get("address") {
            if let Some(Value::String(city)) = addr.get("city") {
                Some(city.as_str())
            } else { None }
        } else { None }
    } else { None }
} else { None };

// Path query - resilient
let city = json.get_path("user.address.city")
    .and_then(|v| v.as_string());
```

**Real-world impact**: Extract all user IDs from paginated API response:

```json
{
  "pages": [
    {"users": [{"id": 1}, {"id": 2}]},
    {"users": [{"id": 3}, {"id": 4}]}
  ]
}
```

```rust
// Get all user IDs across all pages
let ids = json.query("pages[*].users[*].id");
// Returns: [Number(1), Number(2), Number(3), Number(4)]
```

**Performance**: Wildcard query on 1000-element array:
- Naive: Parse entire document, traverse manually → ~10ms
- Optimized path query: Single pass with pattern matching → ~0.5ms (**20x faster**)

---

### Milestone 5: Complete Validation Framework

**Concepts applied**:
- **Trait objects**: `Box<dyn Validator>` for custom validators
- **Error aggregation**: Collect all validation errors, not just first
- **Builder pattern**: Construct complex schemas ergonomically
- **Cross-field validation**: Access entire document during validation

**Why this matters**: Production-grade validation with extensibility:

```rust
// Built-in validators + custom validators
let schema = Schema::object()
    .required_property("email", Schema::custom(EmailValidator))
    .required_property("url", Schema::custom(UrlValidator))
    .required_property("date", Schema::custom(DateValidator))
    .build();

// Multiple errors collected
let json = r#"{
    "email": "invalid",
    "url": "not-a-url",
    "date": "99-99-9999"
}"#;

let errors = schema.validate_all(&json)?;
// Returns all 3 errors, not just "email invalid"
```

**Real-world impact**: User registration form validation:

```rust
let schema = user_registration_schema();

match schema.validate(&form_data) {
    Ok(()) => create_account(form_data),
    Err(errors) => {
        // Show all errors to user:
        // - Username too short (min 3 chars)
        // - Email invalid format
        // - Password too weak (min 8 chars)
        // - Age below minimum (13+)
        return json_response(errors);
    }
}
```

**UX benefit**: Show all form errors at once, not one-by-one:
- **Stop-at-first**: User fixes email → resubmit → username too short → resubmit → password weak → frustrating!
- **Collect-all**: User sees all 3 errors immediately → fix all → submit once → smooth!

**Extensibility**: Add validators without modifying core code:

```rust
// Library provides EmailValidator, UrlValidator

// User adds custom validator
struct ApiKeyValidator;
impl Validator for ApiKeyValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let key = value.as_string()?;
        if api_keys::is_valid(key) {
            Ok(())
        } else {
            Err("Invalid API key".to_string())
        }
    }
}

// Use it in schema
let schema = Schema::custom(ApiKeyValidator);
```

**Performance** (10,000 validations):
- Stop-at-first: ~10ms (some fail fast)
- Collect-all: ~12ms (**20% slower but better UX**)
- Custom validators: +2ms per validator

---

### Project-Wide Benefits

By implementing all five milestones, you master:

**1. Type Safety**:
- Exhaustive matching prevents missing cases
- Recursive types model data structure accurately
- Pattern guards validate constraints at type level

**2. Correctness**:
- Compiler catches logic errors (missed enum cases)
- Impossible to access wrong variant data
- No null pointer dereferences or type confusion

**3. Maintainability**:
- Add enum variant → compiler finds all match sites
- Pattern matching makes intent clear
- Error types document failure modes

**4. Performance**:
- Match compiles to jump tables (O(1) dispatch)
- No runtime type checking overhead (types erased)
- Inlining makes patterns zero-cost

**5. Extensibility**:
- Trait objects for custom validators
- Builder pattern for flexible schemas
- Path queries for generic data extraction

**Concrete comparison** - JSON parsing 10,000 objects:

| Metric | No Validation | Basic Validation | Full Validation | Improvement |
|--------|---------------|------------------|-----------------|-------------|
| Parse time | 5ms | 5ms | 5ms | Same |
| Validation | 0ms | 15ms | 25ms | Type-safe checks |
| Type errors caught | 0% | 80% | 98% | Early detection |
| Clear error messages | No | No | Yes | Developer experience |
| Custom validators | No | No | Yes | Extensibility |

**Real-world validation**:
- **serde_json**: Similar enum-based design, 100M+ downloads
- **valico**: JSON schema validator using trait objects
- **jsonschema**: Exhaustive validation with pattern matching

This project teaches the patterns used in production Rust libraries that power thousands of web services, APIs, and data pipelines.

---

**The Naive Approach Problem**:
```rust
// Unsafe: No type checking, runtime panics
fn naive_parse(json: &str) -> HashMap<String, String> {
    // Problems:
    // - Assumes all values are strings (runtime panic on numbers)
    // - No validation (accepts invalid JSON)
    // - No nested object support
    // - No schema enforcement
    // - No helpful error messages
    serde_json::from_str(json).unwrap() // Panics on error!
}
```

**Pattern-Based Validation Benefits**:
- **Compile-time safety**: Exhaustive matching catches all cases
- **Type guarantees**: Schema validation at parse time
- **Clear errors**: Detailed validation failure messages
- **Extensibility**: Easy to add new validation rules
- **Performance**: No runtime type checking overhead


## Milestone 1: JSON Value Representation and Basic Parsing

**Goal**: Define JSON value types using enums and parse simple JSON into an AST.

**Implementation Steps**:

1. **Define JSON value enum**:
   - Create `Value` enum with all JSON types
   - Use recursive types for arrays and objects
   - Implement `Debug` and `PartialEq` for testing

2. **Implement tokenizer**:
   - Scan JSON string into tokens
   - Handle strings, numbers, booleans, null, punctuation
   - Skip whitespace and track positions for errors

3. **Parse simple values**:
   - Parse strings, numbers, booleans, null
   - Use pattern matching on token types
   - Return detailed parse errors with positions

4. **Test on simple JSON**:
   - Parse `"hello"`, `42`, `true`, `null`
   - Validate error messages
   - Handle malformed input

**Starter Code**:

```rust
use std::collections::HashMap;
use std::fmt;

// TODO: Create an enum to represent any JSON value
// Refer to the "JSON Data Types" section above for the six types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // TODO: Add variants for all six JSON types described in the JSON syntax section
    // Hint: Some variants need to contain data (e.g., boolean values, numbers)
    // Hint: Arrays and objects require recursive structures
}

impl Value {
    // TODO: Implement type-checking methods that return true if this value is of the specified type
    pub fn is_null(&self) -> bool {
        // TODO: Check if this value is the Null variant
       todo!()
    }

    pub fn is_bool(&self) -> bool {
       // TODO: Check if this value is the Bool variant
        todo!()
    }

    pub fn is_number(&self) -> bool {
       // TODO: Check if this value is the Number variant
        todo!()
    }

    pub fn is_string(&self) -> bool {
       // TODO: Check if this value is the String variant
        todo!()
    }

    pub fn is_array(&self) -> bool {
       // TODO: Check if this value is the Array variant
        todo!()
    }

    pub fn is_object(&self) -> bool {
       // TODO: Check if this value is the Object variant
        todo!()
    }

    // TODO: Extract the inner value safely, returning None if the type doesn't match
    pub fn as_bool(&self) -> Option<bool> {
        let Value::Bool(b) = self else { return None };
        Some(*b)
    }

    pub fn as_number(&self) -> Option<f64> {
        // TODO: Extract the number if this is a Number variant, otherwise return None
        // Hint: Follow the same pattern as as_bool above
        todo!()
    }

    pub fn as_string(&self) -> Option<&str> {
       // TODO: Extract a string reference if this is a String variant
       // Hint: Similar to as_bool, but return a reference to the string data
        todo!()
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
       // TODO: Extract the array reference if this is an Array variant
        todo!()
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
       // TODO: Extract the object reference if this is an Object variant
        todo!()
    }

    // TODO: Get a field value from an object by key name
    pub fn get(&self, key: &str) -> Option<&Value> {
       // TODO: If this is an object, look up the key and return the value
       // Hint: First extract the object, then look up the key
        todo!()
    }

    // TODO: Get an element from an array by index
    pub fn get_index(&self, index: usize) -> Option<&Value> {
       // TODO: If this is an array, get the element at the given position
        todo!()
    }
}

// TODO: Create an enum representing the different tokens the lexer can recognize
// These are already defined for you as a reference
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace, RightBrace, LeftBracket, RightBracket,
    Colon, Comma,
    String(String),
    Number(f64),
    True, False, Null,
    Eof
}

// TODO: Create a structure to hold error information when parsing fails
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    // TODO: Store the error description and location where the error occurred
    // Hint: You'll need at least a message and some way to track position
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly error message that includes the position
        // Hint: Use the write! macro to format the output
        todo!()
    }
}

impl std::error::Error for ParseError {}

// TODO: Create a tokenizer that breaks JSON text into meaningful pieces
pub struct Lexer<'a> {
    // TODO: Store the input text and track the current reading position
    // Hint: Keep a reference to the input string and an index
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
       // TODO: Initialize the lexer with the input text at position zero
        todo!()
    }

    // TODO: Read and return the next meaningful token from the input
    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        // TODO: First, skip any whitespace characters
        // TODO: Look at the current character and decide what token it starts:
        //   - Opening/closing braces and brackets create punctuation tokens
        //   - Quotes begin a string value
        //   - Letters 't', 'f', 'n' start keywords (true, false, null)
        //   - Digits or minus sign start a number
        //   - Anything else is an unexpected character error
        // Hint: Use pattern matching on the current character
        todo!()
    }

    fn skip_whitespace(&mut self) {
        // TODO: Advance position while the current character is whitespace
        // Hint: Spaces, tabs, newlines, and carriage returns are whitespace
        todo!()
    }

    fn parse_string(&mut self) -> Result<Token, ParseError> {
        // TODO: Parse a JSON string value
        // TODO: Skip the opening quote, then collect characters until the closing quote
        // TODO: Handle escape sequences like \", \\, \n, \t, and \uXXXX
        // Hint: Build up the string character by character, watching for backslashes
        todo!()
    }

    fn parse_number(&mut self) -> Result<Token, ParseError> {
        // TODO: Parse a numeric value (can be integer or decimal)
        // TODO: Collect all digits, handling optional minus sign and decimal point
        // TODO: Convert the collected characters into a number
        // Hint: Build a string of numeric characters, then parse it
        todo!()
    }

    fn parse_true(&mut self) -> Result<Token, ParseError> {
        // TODO: Verify that the next characters spell "true" exactly
        // Hint: Check each expected character in sequence
        todo!()
    }

    fn parse_false(&mut self) -> Result<Token, ParseError> {
        // TODO: Verify that the next characters spell "false" exactly
        todo!()
    }

    fn parse_null(&mut self) -> Result<Token, ParseError> {
        // TODO: Verify that the next characters spell "null" exactly
        todo!()
    }

    fn current_char(&self) -> Option<char> {
        // TODO: Return the character at the current position, or None if at end
        todo!()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
       // TODO: Look ahead at a character without advancing position
       // Hint: Add the offset to current position and get that character
       todo!()
    }
}

// TODO: Create a parser that converts tokens into JSON values
pub struct Parser<'a> {
    // TODO: Store the lexer and keep track of the current token being examined
    // Hint: You'll need the lexer to get tokens, and need to remember the current token
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        // TODO: Create a new parser and read the first token
        // Hint: Create the lexer, then immediately fetch the first token
        todo!()
    }

    // TODO: Convert the current token into a JSON Value
    pub fn parse_value(&mut self) -> Result<Value, ParseError> {
        // TODO: Look at what type of token you have and create the matching Value
        // Hint: Use pattern matching to handle each token type
        // Refer to the Token enum and Value enum definitions
        todo!()
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        // TODO: Move to the next token by asking the lexer for another token
        todo!()
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        // Placeholder for Milestone 2
        todo!()
    }

    fn parse_object(&mut self) -> Result<Value, ParseError> {
        // Placeholder for Milestone 2
        todo!()
    }
}

// TODO: Main entry point for parsing JSON text into a Value
pub fn parse(input: &str) -> Result<Value, ParseError> {
    // TODO: Create a parser and use it to parse the input
    // Hint: Create a Parser, then call parse_value on it
    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_parse_null() {
    let value = parse("null").unwrap();
    assert_eq!(value, Value::Null);
    assert!(value.is_null());
}

#[test]
fn test_parse_bool() {
    let value_true = parse("true").unwrap();
    assert_eq!(value_true, Value::Bool(true));
    assert_eq!(value_true.as_bool(), Some(true));

    let value_false = parse("false").unwrap();
    assert_eq!(value_false, Value::Bool(false));
    assert_eq!(value_false.as_bool(), Some(false));
}

#[test]
fn test_parse_number() {
    let value = parse("42").unwrap();
    assert_eq!(value, Value::Number(42.0));
    assert_eq!(value.as_number(), Some(42.0));

    let value_float = parse("3.14").unwrap();
    assert_eq!(value_float, Value::Number(3.14));

    let value_negative = parse("-10").unwrap();
    assert_eq!(value_negative, Value::Number(-10.0));
}

#[test]
fn test_parse_string() {
    let value = parse(r#""hello""#).unwrap();
    assert_eq!(value, Value::String("hello".to_string()));
    assert_eq!(value.as_string(), Some("hello"));
}

#[test]
fn test_parse_string_with_escapes() {
    let value = parse(r#""hello\nworld""#).unwrap();
    assert_eq!(value, Value::String("hello\nworld".to_string()));

    let value = parse(r#""quote: \"test\"""#).unwrap();
    assert_eq!(value, Value::String(r#"quote: "test""#.to_string()));
}

#[test]
fn test_parse_error() {
    let result = parse("invalid");
    assert!(result.is_err());

    let result = parse("tru"); // Incomplete true
    assert!(result.is_err());

    let result = parse(r#""unclosed string"#);
    assert!(result.is_err());
}

#[test]
fn test_type_checking() {
    let null = Value::Null;
    assert!(null.is_null());
    assert!(!null.is_bool());
    assert!(!null.is_number());

    let num = Value::Number(42.0);
    assert!(num.is_number());
    assert!(!num.is_string());
}
```

**Check Your Understanding**:
- Why use exhaustive matching on `Token` enum instead of if-else chains?
- How does the let-else pattern simplify value extraction?
- What errors can occur during tokenization vs parsing?
- Why separate tokenization (lexing) from parsing?

---

## Milestone 2: Recursive Parsing for Arrays and Objects

**Goal**: Parse nested JSON structures using recursive pattern matching.

**Implementation Steps**:

1. **Implement array parsing**:
   - Match on `[` token
   - Recursively parse values
   - Handle commas and closing `]`
   - Support empty arrays and trailing commas

2. **Implement object parsing**:
   - Match on `{` token
   - Parse key-value pairs (key must be string)
   - Recursively parse values
   - Handle commas and closing `}`
   - Support empty objects

3. **Handle deep nesting**:
   - Arrays containing objects
   - Objects containing arrays
   - Deeply nested structures
   - Prevent stack overflow on excessive nesting

4. **Test complex JSON**:
   - Parse nested objects
   - Parse arrays of arrays
   - Parse mixed structures
   - Handle malformed input

**Starter Code Extension**:

```rust
impl<'a> Parser<'a> {
    // TODO: Parse a JSON array by recursively parsing each element
    fn parse_array(&mut self) -> Result<Value, ParseError> {
        // TODO: Verify the current token is an opening bracket
        // TODO: Create a vector to collect array elements
        // TODO: Loop until you find the closing bracket:
        //   - Parse each value recursively (arrays can contain anything!)
        //   - Look for commas between elements
        //   - Stop when you see the closing bracket
        // TODO: Return the collected values as an Array variant
        // Hint: Handle the empty array case (immediate closing bracket)
            todo!()
    }

    // TODO: Parse a JSON object by recursively parsing each key-value pair
    fn parse_object(&mut self) -> Result<Value, ParseError> {
       // TODO: Verify the current token is an opening brace
       // TODO: Create a map to store key-value pairs
       // TODO: Loop until you find the closing brace:
       //   - Keys must be strings (check the token type)
       //   - After each key, expect a colon
       //   - Parse the value recursively (objects can contain anything!)
       //   - Look for commas between pairs
       //   - Stop when you see the closing brace
       // TODO: Return the map as an Object variant
       // Hint: Handle the empty object case
        todo!()
    }

    // TODO: Parse a value while tracking how deeply nested we are
    fn parse_value_with_depth(&mut self, depth: usize) -> Result<Value, ParseError> {
        // TODO: Define a maximum allowed nesting depth (e.g., 100 levels)
        // TODO: Return an error if we've nested too deeply
        // TODO: Check what kind of token we have:
        //   - Opening bracket means an array (recurse with depth + 1)
        //   - Opening brace means an object (recurse with depth + 1)
        //   - Other tokens are simple values (no recursion needed)
        // Hint: This prevents stack overflow from maliciously deeply nested input
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_parse_empty_array() {
    let value = parse("[]").unwrap();
    assert_eq!(value, Value::Array(vec![]));
    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 0);
}

#[test]
fn test_parse_array_of_numbers() {
    let value = parse("[1, 2, 3, 4, 5]").unwrap();
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0], Value::Number(1.0));
    assert_eq!(arr[4], Value::Number(5.0));
}

#[test]
fn test_parse_mixed_array() {
    let value = parse(r#"[1, "hello", true, null]"#).unwrap();
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 4);
    assert_eq!(arr[0], Value::Number(1.0));
    assert_eq!(arr[1], Value::String("hello".to_string()));
    assert_eq!(arr[2], Value::Bool(true));
    assert_eq!(arr[3], Value::Null);
}

#[test]
fn test_parse_nested_arrays() {
    let value = parse("[[1, 2], [3, 4], [5, 6]]").unwrap();
    let arr = value.as_array().unwrap();
    assert_eq!(arr.len(), 3);

    let first = arr[0].as_array().unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(first[0], Value::Number(1.0));
}

#[test]
fn test_parse_empty_object() {
    let value = parse("{}").unwrap();
    assert_eq!(value, Value::Object(HashMap::new()));
    assert!(value.is_object());
}

#[test]
fn test_parse_simple_object() {
    let value = parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj.get("name"), Some(&Value::String("Alice".to_string())));
    assert_eq!(obj.get("age"), Some(&Value::Number(30.0)));
}

#[test]
fn test_parse_nested_object() {
    let value = parse(r#"{"user": {"name": "Bob", "admin": true}}"#).unwrap();
    let obj = value.as_object().unwrap();

    let user = obj.get("user").unwrap().as_object().unwrap();
    assert_eq!(user.get("name"), Some(&Value::String("Bob".to_string())));
    assert_eq!(user.get("admin"), Some(&Value::Bool(true)));
}

#[test]
fn test_parse_complex_structure() {
    let json = r#"
    {
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ],
        "count": 2
    }
    "#;

    let value = parse(json).unwrap();
    let obj = value.as_object().unwrap();

    let users = obj.get("users").unwrap().as_array().unwrap();
    assert_eq!(users.len(), 2);

    let first_user = users[0].as_object().unwrap();
    assert_eq!(first_user.get("id"), Some(&Value::Number(1.0)));
}

#[test]
fn test_deeply_nested_structure() {
    let json = r#"{"a": {"b": {"c": {"d": {"e": 5}}}}}"#;
    let value = parse(json).unwrap();

    // Navigate deep nesting using pattern matching
    let a = value.get("a").unwrap();
    let b = a.get("b").unwrap();
    let c = b.get("c").unwrap();
    let d = c.get("d").unwrap();
    let e = d.get("e").unwrap();

    assert_eq!(e, &Value::Number(5.0));
}

#[test]
fn test_prevent_stack_overflow() {
    // Create extremely nested structure
    let mut json = String::new();
    for _ in 0..200 {
        json.push('[');
    }
    for _ in 0..200 {
        json.push(']');
    }

    // Should return error instead of stack overflow
    let result = parse(&json);
    assert!(result.is_err());
}

#[test]
fn test_malformed_array() {
    assert!(parse("[1, 2,").is_err()); // Missing closing bracket
    assert!(parse("[1 2]").is_err()); // Missing comma
    assert!(parse("[,1,2]").is_err()); // Leading comma
}

#[test]
fn test_malformed_object() {
    assert!(parse(r#"{"key": "value""#).is_err()); // Missing closing brace
    assert!(parse(r#"{"key" "value"}"#).is_err()); // Missing colon
    assert!(parse(r#"{key: "value"}"#).is_err()); // Unquoted key
}
```

**Check Your Understanding**:
- Why is recursive descent parsing natural for JSON?
- How does pattern matching on tokens simplify parsing logic?
- What's the risk of unbounded recursion in parsing?
- How would you improve error messages for nested structures?

---

## Milestone 3: Schema Definition and Type Validation

**Goal**: Define JSON schemas and validate values using pattern guards and exhaustive matching.

**Implementation Steps**:

1. **Define Schema enum**:
   - Schemas for all JSON types
   - Support for optional fields
   - Support for array item schemas
   - Support for object property schemas

2. **Implement validation**:
   - Match value type against schema type
   - Use pattern guards for constraints
   - Validate nested structures recursively
   - Return detailed validation errors

3. **Add constraints**:
   - String: min/max length, regex patterns
   - Number: min/max, integer-only
   - Array: min/max items, unique items
   - Object: required fields, additional properties

4. **Test validation**:
   - Valid data passes
   - Invalid data fails with clear errors
   - Nested validation works correctly

**Starter Code**:

```rust
// TODO: Create an enum that defines validation rules for JSON values
// This is already partially defined to show you the structure
#[derive(Debug, Clone, PartialEq)]
pub enum Schema {
    Null,
    Bool,
    Number { min: Option<f64>, max: Option<f64>, integer_only: bool },
    String { min_length: Option<usize>, max_length: Option<usize>, pattern: Option<String> },
    Array { items: Box<Schema>, min_items: Option<usize>, max_items: Option<usize> },
    Object { properties: HashMap<String, PropertySchema>, required: Vec<String>, additional_properties: bool },
    Any, // accepts any value
    OneOf(Vec<Schema>) // value must match one of the schemas
}

// TODO: Create a structure describing an object property's validation rules
#[derive(Debug, Clone, PartialEq)]
pub struct PropertySchema {
    // TODO: Store the validation schema for this property and whether it's required
    // Hint: You might also want to store a default value
}

// TODO: Create a structure to hold validation failure information
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    // TODO: Store the path to the invalid value and a description of what's wrong
    // Hint: Path helps users find the exact location of the error in nested structures
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a helpful error message showing the path and what went wrong
        // Hint: Something like "Validation error at $.user.age: value too small"
        todo!()
    }
}

impl std::error::Error for ValidationError {}

impl Schema {
    // TODO: Check if a value conforms to this schema
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
         self.validate_at_path(value, "$");
        todo!()
    }

    fn validate_at_path(&self, value: &Value, path: &str) -> Result<(), ValidationError> {
        // TODO: Match the schema type against the value type
        // TODO: For Null schema: verify the value is null
        // TODO: For Bool schema: verify the value is a boolean
        // TODO: For Number schema:
        //   - Verify the value is a number
        //   - Check if it meets minimum/maximum constraints
        //   - Check if integer_only is true and value has no decimal part
        // TODO: For String schema:
        //   - Verify the value is a string
        //   - Check length constraints (min_length, max_length)
        //   - Optionally check against regex pattern
        // TODO: For Array schema:
        //   - Verify the value is an array
        //   - Check size constraints (min_items, max_items)
        //   - Recursively validate each element against the items schema
        // TODO: For Object schema:
        //   - Verify the value is an object
        //   - Check that all required fields are present
        //   - Validate each property against its schema
        //   - If additional_properties is false, reject unexpected fields
        // TODO: For Any schema: accept any value
        // TODO: For OneOf schema: try each option, succeed if at least one matches
        // Hint: Use pattern matching to handle each combination of schema and value type
           todo!()
    }

    // TODO: Convenience functions to create common schema types
    pub fn string() -> Self {
        Schema::String {
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    pub fn number() -> Self {
        // TODO: Create a Number schema with no constraints
        // Hint: Set all optional fields to None, integer_only to false
        todo!()
    }

    pub fn integer() -> Self {
       // TODO: Create a Number schema that only accepts integers
       // Hint: Similar to number(), but set integer_only to true
        todo!()
    }

    pub fn array(items: Schema) -> Self {
       // TODO: Create an Array schema that validates each item
       // Hint: Set the items schema, no size constraints
        todo!()
    }

    pub fn object() -> ObjectSchemaBuilder {
        // TODO: Return a builder to construct object schemas fluently
        todo!()
    }

    // TODO: Methods to add constraints to schemas (these modify and return self for chaining)
    pub fn min(mut self, min: f64) -> Self {
        match &mut self {
            Schema::Number { min: min_field, .. } => *min_field = Some(min);
            _ => panic!("min() only valid for Number schema")
        }
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        // TODO: Set maximum value constraint for Number schemas
        // Hint: Similar pattern to min() above
        todo!()
    }

    pub fn min_length(mut self, len: usize) -> Self {
        // TODO: Set minimum length constraint for String schemas
        todo!()
    }

    pub fn max_length(mut self, len: usize) -> Self {
        // TODO: Set maximum length constraint for String schemas
        todo!()
    }
}

// TODO: Builder pattern for constructing object schemas
pub struct ObjectSchemaBuilder {
    // TODO: Store the properties being defined, required field names, and whether to allow extra fields
    // Hint: properties maps field names to their schemas
    // Hint: required is a list of field names that must be present
    // Hint: additional_properties controls if undeclared fields are allowed
}

impl ObjectSchemaBuilder {
    pub fn property(mut self, name: impl Into<String>, schema: Schema) -> Self {
       // TODO: Add an optional property to the schema
       // Hint: Add to properties but not to required
        todo!()
    }

    pub fn required_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
       // TODO: Add a required property to the schema
       // Hint: Add to both properties and required
        todo!()
    }

    pub fn allow_additional(mut self) -> Self {
        // TODO: Allow the object to have fields not listed in properties
        // Hint: Set additional_properties to true
        todo!()
    }

    pub fn build(self) -> Schema {
        // TODO: Construct the final Object schema from the builder
        todo!()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_validate_null() {
    let schema = Schema::Null;
    assert!(schema.validate(&Value::Null).is_ok());
    assert!(schema.validate(&Value::Bool(true)).is_err());
}

#[test]
fn test_validate_number_constraints() {
    let schema = Schema::number().min(0.0).max(100.0);

    assert!(schema.validate(&Value::Number(50.0)).is_ok());
    assert!(schema.validate(&Value::Number(0.0)).is_ok());
    assert!(schema.validate(&Value::Number(100.0)).is_ok());

    let err = schema.validate(&Value::Number(-10.0)).unwrap_err();
    assert!(err.message.contains("minimum"));

    let err = schema.validate(&Value::Number(150.0)).unwrap_err();
    assert!(err.message.contains("maximum"));
}

#[test]
fn test_validate_integer_only() {
    let schema = Schema::integer();

    assert!(schema.validate(&Value::Number(42.0)).is_ok());
    assert!(schema.validate(&Value::Number(3.14)).is_err());
}

#[test]
fn test_validate_string_length() {
    let schema = Schema::string().min_length(3).max_length(10);

    assert!(schema.validate(&Value::String("hello".to_string())).is_ok());

    let err = schema.validate(&Value::String("ab".to_string())).unwrap_err();
    assert!(err.message.contains("short"));

    let err = schema.validate(&Value::String("this is too long".to_string())).unwrap_err();
    assert!(err.message.contains("long"));
}

#[test]
fn test_validate_array() {
    let schema = Schema::array(Schema::number());

    let valid = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    assert!(schema.validate(&valid).is_ok());

    let invalid = Value::Array(vec![
        Value::Number(1.0),
        Value::String("not a number".to_string()),
    ]);
    assert!(schema.validate(&invalid).is_err());
}

#[test]
fn test_validate_array_constraints() {
    let schema = Schema::array(Schema::number())
        .min_items(2)
        .max_items(5);

    assert!(schema.validate(&Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
    ])).is_ok());

    let err = schema.validate(&Value::Array(vec![Value::Number(1.0)])).unwrap_err();
    assert!(err.message.contains("few"));

    let err = schema.validate(&Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
        Value::Number(6.0),
    ])).unwrap_err();
    assert!(err.message.contains("many"));
}

#[test]
fn test_validate_object() {
    let schema = Schema::object()
        .required_property("name", Schema::string())
        .required_property("age", Schema::integer().min(0.0))
        .property("email", Schema::string())
        .build();

    let mut valid = HashMap::new();
    valid.insert("name".to_string(), Value::String("Alice".to_string()));
    valid.insert("age".to_string(), Value::Number(30.0));
    valid.insert("email".to_string(), Value::String("alice@example.com".to_string()));

    assert!(schema.validate(&Value::Object(valid)).is_ok());

    // Missing required field
    let mut invalid = HashMap::new();
    invalid.insert("name".to_string(), Value::String("Bob".to_string()));
    let err = schema.validate(&Value::Object(invalid)).unwrap_err();
    assert!(err.message.contains("required") || err.message.contains("missing"));
}

#[test]
fn test_validate_nested_object() {
    let user_schema = Schema::object()
        .required_property("id", Schema::integer())
        .required_property("name", Schema::string())
        .build();

    let schema = Schema::object()
        .required_property("users", Schema::array(user_schema))
        .build();

    let json = r#"
    {
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    }
    "#;

    let value = parse(json).unwrap();
    assert!(schema.validate(&value).is_ok());

    // Invalid: user missing required field
    let json_invalid = r#"
    {
        "users": [
            {"id": 1}
        ]
    }
    "#;

    let value = parse(json_invalid).unwrap();
    let err = schema.validate(&value).unwrap_err();
    assert!(err.path.contains("users[0]"));
}

#[test]
fn test_validate_one_of() {
    let schema = Schema::OneOf(vec![
        Schema::string(),
        Schema::number(),
    ]);

    assert!(schema.validate(&Value::String("hello".to_string())).is_ok());
    assert!(schema.validate(&Value::Number(42.0)).is_ok());
    assert!(schema.validate(&Value::Bool(true)).is_err());
}

#[test]
fn test_validation_error_path() {
    let schema = Schema::object()
        .required_property("user", Schema::object()
            .required_property("age", Schema::integer().min(0.0))
            .build())
        .build();

    let json = r#"{"user": {"age": -5}}"#;
    let value = parse(json).unwrap();

    let err = schema.validate(&value).unwrap_err();
    assert_eq!(err.path, "$.user.age");
    assert!(err.message.contains("minimum"));
}
```

**Check Your Understanding**:
- How do pattern guards enable complex validation rules?
- Why validate recursively instead of iteratively?
- How does the path tracking help with error reporting?
- What's the benefit of OneOf schema over checking multiple types?

---

## Milestone 4: Deep Destructuring and Path Queries

**Goal**: Extract values from nested JSON using deep destructuring and implement JSONPath-like queries.

**Implementation Steps**:

1. **Implement deep field access**:
   - Navigate nested objects with dot notation
   - Access array elements by index
   - Handle missing fields gracefully
   - Return Option<&Value> for safety

2. **Add path query language**:
   - Support `$.field` for object fields
   - Support `$[0]` for array indices
   - Support `$.nested.field` for deep access
   - Support `$.array[*]` for all array elements

3. **Use pattern matching for queries**:
   - Parse path expressions
   - Match on path segments
   - Recursively navigate structure
   - Collect results for wildcard queries

4. **Test complex queries**:
   - Query nested structures
   - Handle missing paths
   - Wildcard queries return multiple results
   - Edge cases (empty arrays, null values)

**Starter Code**:

```rust
impl Value {
    // TODO: Navigate to a value using a path string like "user.address.city"
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        // TODO: Break the path into individual steps (field names or array indices)
        // TODO: Start at the current value
        // TODO: For each step in the path:
        //   - If current value is an object and step is a field name, move into that field
        //   - If current value is an array and step is an index, move to that element
        //   - If step doesn't match value type, return None
        // TODO: Return the final value reached, or None if path doesn't exist
        todo!()
    }

    // TODO: Find all values that match a path pattern (supports wildcards)
    pub fn query(&self, path: &str) -> Vec<&Value> {
        // TODO: Parse the path into segments (fields, indices, or wildcards)
        // TODO: Use recursion to handle wildcards like "users[*].name"
        // TODO: Collect all matching values into a vector
        // Hint: A wildcard means "all children" whether in an array or object
        todo!()
    }

    // TODO: Extract multiple fields from an object into a vector
    pub fn extract<'a>(&'a self, fields: &[&str]) -> Option<Vec<&'a Value>> {
        // TODO: Verify this is an object
        // TODO: Look up each field name and collect the values
        // TODO: Return None if any field is missing
        todo!()
    }

    // TODO: Convert a 2-element array into a tuple
    pub fn as_tuple2(&self) -> Option<(&Value, &Value)> {
        // TODO: Verify this is an array with exactly 2 elements
        // TODO: Return references to the first and second elements
        todo!()
    }

    pub fn as_tuple3(&self) -> Option<(&Value, &Value, &Value)> {
        // TODO: Verify this is an array with exactly 3 elements
        // TODO: Return references to the three elements
        todo!()
    }

    // TODO: Split an array into its first element and the remaining elements
    pub fn split_first(&self) -> Option<(&Value, &[Value])> {
        // TODO: Verify this is an array with at least one element
        // TODO: Return the first element and a slice of the rest
        todo!()
    }
}

// TODO: Represent one step in a path (field name, array index, or wildcard)
#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    // TODO: Define the different types of path steps
    // Hint: Field(String) for accessing object properties
    // Hint: Index(usize) for accessing array elements
    // Hint: Wildcard for matching all children
}

fn parse_path(path: &str) -> Vec<PathSegment> {
    // TODO: Convert a path string like "users[0].name" into a sequence of segments
    // TODO: Split on dots to find field names
    // TODO: Recognize [number] as array indices and [*] as wildcards
    // TODO: Return the list of segments
    todo!()
}

// TODO: Recursively find all values matching a path pattern
fn query_recursive<'a>(
    value: &'a Value,
    segments: &[PathSegment],
    results: &mut Vec<&'a Value>,
) {
    // TODO: If no more segments to process, we've reached a match - add it to results
    // TODO: Look at the first segment and the current value type:
    //   - If segment is a field name and value is an object: navigate into that field
    //   - If segment is an index and value is an array: navigate to that element
    //   - If segment is a wildcard and value is an array: recursively process ALL elements
    //   - If segment is a wildcard and value is an object: recursively process ALL values
    //   - If segment doesn't match value type: this path doesn't work, stop
    // TODO: For each successful navigation, recurse with the remaining segments
    // Hint: Use pattern matching on (segment_type, value_type) pairs
    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_get_path_simple() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let value = parse(json).unwrap();

    assert_eq!(
        value.get_path("name"),
        Some(&Value::String("Alice".to_string()))
    );
    assert_eq!(value.get_path("age"), Some(&Value::Number(30.0)));
    assert_eq!(value.get_path("missing"), None);
}

#[test]
fn test_get_path_nested() {
    let json = r#"{"user": {"name": "Bob", "address": {"city": "NYC"}}}"#;
    let value = parse(json).unwrap();

    assert_eq!(
        value.get_path("user.name"),
        Some(&Value::String("Bob".to_string()))
    );
    assert_eq!(
        value.get_path("user.address.city"),
        Some(&Value::String("NYC".to_string()))
    );
}

#[test]
fn test_get_path_array() {
    let json = r#"{"items": [1, 2, 3, 4, 5]}"#;
    let value = parse(json).unwrap();

    assert_eq!(value.get_path("items[0]"), Some(&Value::Number(1.0)));
    assert_eq!(value.get_path("items[2]"), Some(&Value::Number(3.0)));
    assert_eq!(value.get_path("items[10]"), None);
}

#[test]
fn test_query_wildcard() {
    let json = r#"{"users": [{"name": "Alice"}, {"name": "Bob"}, {"name": "Charlie"}]}"#;
    let value = parse(json).unwrap();

    let names = value.query("users[*].name");
    assert_eq!(names.len(), 3);
    assert_eq!(names[0], &Value::String("Alice".to_string()));
    assert_eq!(names[1], &Value::String("Bob".to_string()));
    assert_eq!(names[2], &Value::String("Charlie".to_string()));
}

#[test]
fn test_extract_fields() {
    let json = r#"{"id": 1, "name": "Alice", "email": "alice@example.com"}"#;
    let value = parse(json).unwrap();

    let fields = value.extract(&["id", "name"]).unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0], &Value::Number(1.0));
    assert_eq!(fields[1], &Value::String("Alice".to_string()));
}

#[test]
fn test_array_destructuring() {
    let json = r#"[1, 2]"#;
    let value = parse(json).unwrap();

    let (first, second) = value.as_tuple2().unwrap();
    assert_eq!(first, &Value::Number(1.0));
    assert_eq!(second, &Value::Number(2.0));
}

#[test]
fn test_split_first() {
    let json = r#"[1, 2, 3, 4, 5]"#;
    let value = parse(json).unwrap();

    let (first, rest) = value.split_first().unwrap();
    assert_eq!(first, &Value::Number(1.0));
    assert_eq!(rest.len(), 4);
    assert_eq!(rest[0], Value::Number(2.0));
}

#[test]
fn test_complex_query() {
    let json = r#"
    {
        "company": {
            "departments": [
                {
                    "name": "Engineering",
                    "employees": [
                        {"id": 1, "name": "Alice"},
                        {"id": 2, "name": "Bob"}
                    ]
                },
                {
                    "name": "Sales",
                    "employees": [
                        {"id": 3, "name": "Charlie"}
                    ]
                }
            ]
        }
    }
    "#;

    let value = parse(json).unwrap();

    // Get all employee names across all departments
    let names = value.query("company.departments[*].employees[*].name");
    assert_eq!(names.len(), 3);
}
```

**Check Your Understanding**:
- How does pattern matching simplify path navigation?
- Why return Option instead of panicking on missing paths?
- What's the complexity of path queries with wildcards?
- How would you optimize wildcard queries for large documents?

---

## Milestone 5: Complete Validation Framework with Examples

**Goal**: Build a complete validation framework with real-world examples and comprehensive error reporting.

**Implementation Steps**:

1. **Add custom validators**:
   - Email validation
   - URL validation
   - Date/time validation
   - Custom regex patterns
   - Cross-field validation

2. **Implement error aggregation**:
   - Collect all validation errors
   - Don't stop at first error
   - Return all errors with paths
   - Support strict/lenient modes

3. **Create real-world schemas**:
   - User registration schema
   - API request schema
   - Configuration file schema
   - Database record schema

4. **Add schema serialization**:
   - Serialize schemas to JSON
   - Load schemas from JSON
   - Share schemas between systems
   - Version schema definitions

**Complete Implementation**:

```rust
// TODO: Define a trait for custom validation logic
pub trait Validator: fmt::Debug {
    fn validate(&self, value: &Value) -> Result<(), String>;
}

// TODO: A validator that checks if a string looks like an email address
#[derive(Debug, Clone)]
pub struct EmailValidator;

impl Validator for EmailValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
       // TODO: Verify the value is a string
       // TODO: Check if it contains an @ symbol (simple check)
       // TODO: Optionally check for a dot after the @
       // Hint: Real email validation is complex, this is a simplified version
        todo!()
    }
}

// TODO: A validator that checks if a string is a valid URL
#[derive(Debug, Clone)]
pub struct UrlValidator;

impl Validator for UrlValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // TODO: Verify the value is a string
        // TODO: Check if it starts with http:// or https://
        // Hint: Real URL validation is complex, this is a simplified version
        todo!()
    }
}

// TODO: A validator that checks if a string is a valid date in YYYY-MM-DD format
#[derive(Debug, Clone)]
pub struct DateValidator;

impl Validator for DateValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // TODO: Extract the string value
        // TODO: Check the format matches YYYY-MM-DD (10 characters, dashes in right places)
        // TODO: Optionally validate the numbers are in valid ranges
        // Hint: Full date validation should check if the date actually exists
        todo!()
    }
}

// TODO: Extended schema type that supports custom validators
#[derive(Debug, Clone)]
pub enum EnhancedSchema {
    // TODO: Include all the basic schema types from before
    // TODO: Add a Custom variant that holds a validator
    // Hint: You might need Box<dyn Validator> to store any validator type
}

// TODO: A context object that tracks validation state and collects all errors
pub struct ValidationContext {
    // TODO: Store the entire JSON document (needed for cross-field validation)
    // TODO: Track the current path being validated (for error messages)
    // TODO: Collect all validation errors found so far
    // TODO: Track whether we're in strict mode (fail fast vs collect all errors)
}

impl ValidationContext {
    pub fn new(root: Value, strict: bool) -> Self {
        // TODO: Initialize the context with the root document
        // TODO: Start the path at "$" (JSON path notation for root)
        // TODO: Create an empty error list
        todo!()
    }

    // TODO: Record a validation error without stopping
    pub fn add_error(&mut self, message: String) {
        // TODO: Create a ValidationError with the current path and message
        // TODO: Add it to the errors list
        // Hint: This allows collecting multiple errors in one validation pass
        todo!()
    }

    // TODO: Validate a value while tracking where we are in the document
    pub fn validate_with_context(
        &mut self,
        schema: &Schema,
        value: &Value,
        path: &str,
    ) {
        // TODO: Remember the previous path
        // TODO: Update current_path to the new path
        // TODO: Run validation, catching any errors
        // TODO: Restore the previous path when done
        // Hint: This lets error messages show exact locations like "$.users[0].email"
        todo!()
    }

    // TODO: Finish validation and return the result
    pub fn finish(self) -> Result<(), Vec<ValidationError>> {
        // TODO: If there are no errors, return Ok
        // TODO: If there are errors, return them all
        todo!()
    }
}

// TODO: Example schemas
pub mod schemas {
    use super::*;

    pub fn user_registration_schema() -> Schema {
        Schema::object()
            .required_property("username", Schema::string()
                .min_length(3)
                .max_length(20)
                .pattern("^[a-zA-Z0-9_]+$"))
            .required_property("email", Schema::custom(EmailValidator))
            .required_property("password", Schema::string()
                .min_length(8))
            .required_property("age", Schema::integer()
                .min(13)
                .max(120))
            .property("website", Schema::custom(UrlValidator))
            .build()
    }

    pub fn api_request_schema() -> Schema {
        // Define API request schema
        Schema::object()
            .required_property("method", Schema::OneOf(vec![
                Schema::const_string("GET"),
                Schema::const_string("POST"),
                Schema::const_string("PUT"),
                Schema::const_string("DELETE"),
            ]))
            .required_property("path", Schema::string())
            .property("headers", Schema::object()
                .allow_additional()
                .build())
            .property("body", Schema::Any)
            .build()
    }

    pub fn config_schema() -> Schema {
        //Define application configuration schema
        Schema::object()
            .required_property("server", Schema::object()
                .required_property("host", Schema::string())
                .required_property("port", Schema::integer()
                    .min(1)
                    .max(65535))
                .build())
            .required_property("database", Schema::object()
                .required_property("url", Schema::custom(UrlValidator))
                .property("pool_size", Schema::integer()
                    .min(1)
                    .max(100))
                .build())
            .property("logging", Schema::object()
                .property("level", Schema::OneOf(vec![
                    Schema::const_string("debug"),
                    Schema::const_string("info"),
                    Schema::const_string("warn"),
                    Schema::const_string("error"),
                ]))
                .build())
            .build()
    }
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_user_registration_valid() {
    use schemas::user_registration_schema;

    let json = r#"
    {
        "username": "alice123",
        "email": "alice@example.com",
        "password": "SecurePass123",
        "age": 25
    }
    "#;

    let value = parse(json).unwrap();
    let schema = user_registration_schema();

    assert!(schema.validate(&value).is_ok());
}

#[test]
fn test_user_registration_invalid_email() {
    use schemas::user_registration_schema;

    let json = r#"
    {
        "username": "alice123",
        "email": "not-an-email",
        "password": "SecurePass123",
        "age": 25
    }
    "#;

    let value = parse(json).unwrap();
    let schema = user_registration_schema();

    let err = schema.validate(&value).unwrap_err();
    assert!(err.path.contains("email"));
}

#[test]
fn test_multiple_validation_errors() {
    use schemas::user_registration_schema;

    let json = r#"
    {
        "username": "ab",
        "email": "invalid",
        "password": "short",
        "age": 5
    }
    "#;

    let value = parse(json).unwrap();
    let schema = user_registration_schema();

    let mut ctx = ValidationContext::new(value.clone(), false);
    ctx.validate_with_context(&schema, &value, "$");

    let errors = ctx.finish().unwrap_err();
    assert!(errors.len() >= 3); // username too short, email invalid, password too short, age too low
}

#[test]
fn test_config_validation() {
    use schemas::config_schema;

    let json = r#"
    {
        "server": {
            "host": "localhost",
            "port": 8080
        },
        "database": {
            "url": "postgres://localhost/mydb",
            "pool_size": 10
        },
        "logging": {
            "level": "info"
        }
    }
    "#;

    let value = parse(json).unwrap();
    let schema = config_schema();

    assert!(schema.validate(&value).is_ok());
}

#[test]
fn test_api_request_validation() {
    use schemas::api_request_schema;

    let json = r#"
    {
        "method": "POST",
        "path": "/api/users",
        "headers": {
            "Content-Type": "application/json",
            "Authorization": "Bearer token123"
        },
        "body": {
            "name": "Alice",
            "email": "alice@example.com"
        }
    }
    "#;

    let value = parse(json).unwrap();
    let schema = api_request_schema();

    assert!(schema.validate(&value).is_ok());
}

#[test]
fn test_invalid_http_method() {
    use schemas::api_request_schema;

    let json = r#"
    {
        "method": "INVALID",
        "path": "/api/users"
    }
    "#;

    let value = parse(json).unwrap();
    let schema = api_request_schema();

    let err = schema.validate(&value).unwrap_err();
    assert!(err.path.contains("method"));
}

#[test]
fn test_schema_serialization() {
    let schema = Schema::object()
        .required_property("name", Schema::string())
        .required_property("age", Schema::integer())
        .build();

    // Serialize schema to JSON
    let schema_json = serialize_schema(&schema);

    // Deserialize back
    let schema_restored = deserialize_schema(&schema_json).unwrap();

    assert_eq!(schema, schema_restored);
}

#[test]
fn test_comprehensive_validation() {
    let json = r#"
    {
        "users": [
            {
                "id": 1,
                "username": "alice",
                "email": "alice@example.com",
                "profile": {
                    "bio": "Software engineer",
                    "website": "https://alice.dev"
                }
            },
            {
                "id": 2,
                "username": "bob",
                "email": "bob@example.com",
                "profile": {
                    "bio": "Product manager"
                }
            }
        ],
        "total": 2
    }
    "#;

    let value = parse(json).unwrap();

    // Navigate and validate
    let users = value.get_path("users").unwrap().as_array().unwrap();
    assert_eq!(users.len(), 2);

    let first_user = &users[0];
    assert_eq!(
        first_user.get("username").unwrap(),
        &Value::String("alice".to_string())
    );

    // Extract fields
    let (id, username) = first_user
        .extract(&["id", "username"])
        .map(|v| (v[0], v[1]))
        .unwrap();

    assert_eq!(id, &Value::Number(1.0));
    assert_eq!(username, &Value::String("alice".to_string()));
}
```

**Check Your Understanding**:
- How do custom validators extend the validation framework?
- Why collect all errors instead of stopping at the first?
- How does cross-field validation work with the context pattern?
- What are the trade-offs of strict vs lenient validation?

---

## Complete Project Summary

**What You Built**:
1. Complete JSON tokenizer and recursive descent parser
2. Exhaustive pattern matching for all JSON types
3. Comprehensive schema validation framework
4. Deep destructuring and path query system
5. Custom validators and error aggregation
6. Real-world validation examples

**Key Concepts Practiced**:
- Exhaustive enum matching (Value, Token, Schema)
- Recursive pattern matching (parse_array, parse_object)
- Pattern guards (validation constraints)
- Deep destructuring (path queries, field extraction)
- Or-patterns (OneOf schemas)
- Let-else patterns (safe value extraction)
- matches! macro (type checking)

**Real-World Applications**:
- API request/response validation
- Configuration file parsing
- Data serialization/deserialization
- Schema enforcement (OpenAPI, JSON Schema)
- Type-safe data transformation

**Extension Ideas**:
1. **Performance**: Optimize parser with zero-copy strings
2. **JSONPath**: Full JSONPath query language support
3. **Schema composition**: Extend, merge, compose schemas
4. **Code generation**: Generate Rust structs from schemas
5. **Async validation**: Async custom validators (DB lookups)
6. **Schema evolution**: Handle schema versioning
7. **Pretty errors**: Color-coded error messages with snippets
8. **Benchmarks**: Compare with serde_json performance
9. **WASM**: Compile to WebAssembly for browser use
10. **CLI tool**: Command-line JSON validator

This project demonstrates how Rust's pattern matching creates type-safe, maintainable parsing and validation code!
