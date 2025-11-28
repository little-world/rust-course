# Project 4: JSON Parser with Pattern-Based Validation

## Problem Statement

Build a JSON parser and schema validator that uses Rust's pattern matching to parse JSON strings into an AST and validate against type schemas. You'll implement recursive pattern matching for nested structures, exhaustive enum matching for all JSON types, pattern guards for validation rules, and deep destructuring for complex document traversal.

## Use Cases

**When you need this pattern**:
1. **API validation**: Validate request/response payloads against schemas
2. **Configuration files**: Parse and validate app configuration with type safety
3. **Data serialization**: Type-safe JSON encoding/decoding
4. **Schema validation**: OpenAPI, JSON Schema enforcement
5. **Data transformation**: Map JSON to domain models with validation
6. **Contract testing**: Ensure API contracts are maintained

## Why It Matters

**Real-World Impact**: JSON parsing and validation is fundamental to modern applications:

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

**Real-World Tools Using These Patterns**:
- `serde_json`: Rust's standard JSON library
- API gateways: Request validation (Kong, Tyk)
- GraphQL servers: Schema validation
- OpenAPI validators: Contract enforcement
- Database ORMs: Type-safe query results

## Learning Goals

By completing this project, you will:

1. **Master exhaustive enum matching**: Handle all JSON types safely
2. **Use recursive patterns**: Parse nested structures
3. **Apply pattern guards**: Express validation rules clearly
4. **Deep destructuring**: Extract nested values type-safely
5. **Or-patterns**: Accept multiple valid types
6. **Let-else patterns**: Handle validation errors gracefully
7. **matches! macro**: Quick type checking

---

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

// TODO: Define Value enum for all JSON types
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    // TODO: Add variants for JSON types
    // Hint: Null
    // Hint: Bool(bool)
    // Hint: Number(f64)
    // Hint: String(String)
    // Hint: Array(Vec<Value>)
    // Hint: Object(HashMap<String, Value>)
}

impl Value {
    // TODO: Type checking using matches! macro
    pub fn is_null(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Null)
        todo!()
    }

    pub fn is_bool(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Bool(_))
        todo!()
    }

    pub fn is_number(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Number(_))
        todo!()
    }

    pub fn is_string(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::String(_))
        todo!()
    }

    pub fn is_array(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Array(_))
        todo!()
    }

    pub fn is_object(&self) -> bool {
        // Pseudocode:
        // matches!(self, Value::Object(_))
        todo!()
    }

    // TODO: Safe value extraction using let-else patterns
    pub fn as_bool(&self) -> Option<bool> {
        // Pseudocode:
        // let Value::Bool(b) = self else { return None };
        // Some(*b)
        todo!()
    }

    pub fn as_number(&self) -> Option<f64> {
        // Pseudocode:
        // let Value::Number(n) = self else { return None };
        // Some(*n)
        todo!()
    }

    pub fn as_string(&self) -> Option<&str> {
        // Pseudocode:
        // let Value::String(s) = self else { return None };
        // Some(s.as_str())
        todo!()
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        // Pseudocode:
        // let Value::Array(arr) = self else { return None };
        // Some(arr)
        todo!()
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        // Pseudocode:
        // let Value::Object(obj) = self else { return None };
        // Some(obj)
        todo!()
    }

    // TODO: Get nested field using pattern matching
    pub fn get(&self, key: &str) -> Option<&Value> {
        // Pseudocode:
        // match self:
        //     Value::Object(map) => map.get(key)
        //     _ => None
        todo!()
    }

    // TODO: Get array element using pattern matching
    pub fn get_index(&self, index: usize) -> Option<&Value> {
        // Pseudocode:
        // match self:
        //     Value::Array(arr) if index < arr.len() => Some(&arr[index])
        //     _ => None
        todo!()
    }
}

// TODO: Define Token enum for lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // TODO: Add token types
    // Hint: LeftBrace, RightBrace, LeftBracket, RightBracket
    // Hint: Colon, Comma
    // Hint: String(String)
    // Hint: Number(f64)
    // Hint: True, False, Null
    // Hint: Eof
}

// TODO: Define ParseError for detailed error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    // TODO: Add fields
    // Hint: message: String
    // Hint: position: usize
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format error with position
        // Pseudocode:
        // write!(f, "Parse error at position {}: {}", self.position, self.message)
        todo!()
    }
}

impl std::error::Error for ParseError {}

// TODO: Tokenizer/Lexer
pub struct Lexer<'a> {
    // TODO: Add fields
    // Hint: input: &'a str
    // Hint: position: usize
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        // Pseudocode:
        // Self { input, position: 0 }
        todo!()
    }

    // TODO: Get next token using pattern matching on characters
    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        // Pseudocode:
        // Skip whitespace
        // Match on current character:
        //     '{' => Token::LeftBrace
        //     '}' => Token::RightBrace
        //     '[' => Token::LeftBracket
        //     ']' => Token::RightBracket
        //     ':' => Token::Colon
        //     ',' => Token::Comma
        //     '"' => parse_string()
        //     't' => parse_true()
        //     'f' => parse_false()
        //     'n' => parse_null()
        //     '0'..='9' | '-' => parse_number()
        //     _ => Err(ParseError)
        todo!()
    }

    fn skip_whitespace(&mut self) {
        // Pseudocode:
        // while position < input.len():
        //     match input[position]:
        //         ' ' | '\t' | '\n' | '\r' => position += 1
        //         _ => break
        todo!()
    }

    fn parse_string(&mut self) -> Result<Token, ParseError> {
        // TODO: Parse string with escape sequences
        // Pseudocode:
        // Skip opening quote
        // Collect characters until closing quote
        // Handle escape sequences: \", \\, \n, \t, \u
        // Return Token::String(s)
        todo!()
    }

    fn parse_number(&mut self) -> Result<Token, ParseError> {
        // TODO: Parse number (integer or float)
        // Pseudocode:
        // Collect digits, optional minus, optional decimal point
        // Parse as f64
        // Return Token::Number(n)
        todo!()
    }

    fn parse_true(&mut self) -> Result<Token, ParseError> {
        // TODO: Match "true" exactly
        // Pseudocode:
        // if input[position..].starts_with("true"):
        //     position += 4
        //     return Ok(Token::True)
        // else:
        //     return Err(ParseError)
        todo!()
    }

    fn parse_false(&mut self) -> Result<Token, ParseError> {
        // TODO: Match "false" exactly
        todo!()
    }

    fn parse_null(&mut self) -> Result<Token, ParseError> {
        // TODO: Match "null" exactly
        todo!()
    }

    fn current_char(&self) -> Option<char> {
        // Pseudocode:
        // self.input.chars().nth(self.position)
        todo!()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        // Pseudocode:
        // self.input.chars().nth(self.position + offset)
        todo!()
    }
}

// TODO: Parser for simple values
pub struct Parser<'a> {
    // TODO: Add fields
    // Hint: lexer: Lexer<'a>
    // Hint: current_token: Token
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        // Pseudocode:
        // let mut lexer = Lexer::new(input)
        // let current_token = lexer.next_token()?
        // Ok(Self { lexer, current_token })
        todo!()
    }

    // TODO: Parse simple value using exhaustive pattern matching
    pub fn parse_value(&mut self) -> Result<Value, ParseError> {
        // Pseudocode:
        // match &self.current_token:
        //     Token::Null => {
        //         self.advance()?;
        //         Ok(Value::Null)
        //     }
        //     Token::True => {
        //         self.advance()?;
        //         Ok(Value::Bool(true))
        //     }
        //     Token::False => {
        //         self.advance()?;
        //         Ok(Value::Bool(false))
        //     }
        //     Token::Number(n) => {
        //         let num = *n;
        //         self.advance()?;
        //         Ok(Value::Number(num))
        //     }
        //     Token::String(s) => {
        //         let string = s.clone();
        //         self.advance()?;
        //         Ok(Value::String(string))
        //     }
        //     Token::LeftBracket => self.parse_array()
        //     Token::LeftBrace => self.parse_object()
        //     _ => Err(ParseError::unexpected_token())
        todo!()
    }

    fn advance(&mut self) -> Result<(), ParseError> {
        // Pseudocode:
        // self.current_token = self.lexer.next_token()?;
        // Ok(())
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

// TODO: Convenience function to parse JSON string
pub fn parse(input: &str) -> Result<Value, ParseError> {
    // Pseudocode:
    // let mut parser = Parser::new(input)?;
    // parser.parse_value()
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
    // TODO: Parse array using recursive pattern matching
    fn parse_array(&mut self) -> Result<Value, ParseError> {
        // Pseudocode:
        // Expect LeftBracket token
        // self.advance()?;
        //
        // let mut elements = Vec::new();
        //
        // Loop:
        //     match &self.current_token:
        //         Token::RightBracket =>
        //             self.advance()?;
        //             return Ok(Value::Array(elements))
        //
        //         _ =>
        //             // Parse element recursively
        //             let value = self.parse_value()?;
        //             elements.push(value);
        //
        //             // Handle comma or closing bracket
        //             match &self.current_token:
        //                 Token::Comma =>
        //                     self.advance()?;
        //                     continue
        //                 Token::RightBracket =>
        //                     self.advance()?;
        //                     return Ok(Value::Array(elements))
        //                 _ =>
        //                     return Err(ParseError::expected(",", "]"))
        todo!()
    }

    // TODO: Parse object using recursive pattern matching
    fn parse_object(&mut self) -> Result<Value, ParseError> {
        // Pseudocode:
        // Expect LeftBrace token
        // self.advance()?;
        //
        // let mut map = HashMap::new();
        //
        // Loop:
        //     match &self.current_token:
        //         Token::RightBrace =>
        //             self.advance()?;
        //             return Ok(Value::Object(map))
        //
        //         Token::String(key) =>
        //             let key = key.clone();
        //             self.advance()?;
        //
        //             // Expect colon
        //             match &self.current_token:
        //                 Token::Colon =>
        //                     self.advance()?;
        //                 _ =>
        //                     return Err(ParseError::expected(":"))
        //
        //             // Parse value recursively
        //             let value = self.parse_value()?;
        //             map.insert(key, value);
        //
        //             // Handle comma or closing brace
        //             match &self.current_token:
        //                 Token::Comma =>
        //                     self.advance()?;
        //                     continue
        //                 Token::RightBrace =>
        //                     self.advance()?;
        //                     return Ok(Value::Object(map))
        //                 _ =>
        //                     return Err(ParseError::expected(",", "}"))
        //
        //         _ =>
        //             return Err(ParseError::expected("string key"))
        todo!()
    }

    // TODO: Track nesting depth to prevent stack overflow
    fn parse_value_with_depth(&mut self, depth: usize) -> Result<Value, ParseError> {
        // Pseudocode:
        // const MAX_DEPTH: usize = 100;
        // if depth > MAX_DEPTH:
        //     return Err(ParseError::too_deeply_nested())
        //
        // match &self.current_token:
        //     Token::LeftBracket => self.parse_array_with_depth(depth + 1)
        //     Token::LeftBrace => self.parse_object_with_depth(depth + 1)
        //     _ => self.parse_simple_value()
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
// TODO: Define Schema enum for type definitions
#[derive(Debug, Clone, PartialEq)]
pub enum Schema {
    // TODO: Add schema variants
    // Hint: Null
    // Hint: Bool
    // Hint: Number { min: Option<f64>, max: Option<f64>, integer_only: bool }
    // Hint: String { min_length: Option<usize>, max_length: Option<usize>, pattern: Option<String> }
    // Hint: Array { items: Box<Schema>, min_items: Option<usize>, max_items: Option<usize> }
    // Hint: Object { properties: HashMap<String, PropertySchema>, required: Vec<String>, additional_properties: bool }
    // Hint: Any - accepts any value
    // Hint: OneOf(Vec<Schema>) - value must match one of the schemas
}

// TODO: Define PropertySchema for object properties
#[derive(Debug, Clone, PartialEq)]
pub struct PropertySchema {
    // TODO: Add fields
    // Hint: schema: Schema
    // Hint: required: bool
    // Hint: default: Option<Value>
}

// TODO: Define ValidationError for detailed error reporting
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    // TODO: Add fields
    // Hint: path: String - JSON path where error occurred
    // Hint: message: String
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format error with path
        // Pseudocode:
        // write!(f, "Validation error at '{}': {}", self.path, self.message)
        todo!()
    }
}

impl std::error::Error for ValidationError {}

impl Schema {
    // TODO: Validate value against schema using exhaustive pattern matching
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        // Pseudocode:
        // self.validate_at_path(value, "$")
        todo!()
    }

    fn validate_at_path(&self, value: &Value, path: &str) -> Result<(), ValidationError> {
        // Pseudocode:
        // match (self, value):
        //     (Schema::Null, Value::Null) => Ok(())
        //     (Schema::Null, _) => Err(ValidationError::type_mismatch(path, "null"))
        //
        //     (Schema::Bool, Value::Bool(_)) => Ok(())
        //     (Schema::Bool, _) => Err(ValidationError::type_mismatch(path, "bool"))
        //
        //     (Schema::Number { min, max, integer_only }, Value::Number(n)) =>
        //         // Use pattern guards for constraints
        //         if *integer_only && n.fract() != 0.0:
        //             return Err(ValidationError::not_integer(path))
        //         if let Some(min_val) = min:
        //             if n < min_val:
        //                 return Err(ValidationError::below_minimum(path, *min_val))
        //         if let Some(max_val) = max:
        //             if n > max_val:
        //                 return Err(ValidationError::above_maximum(path, *max_val))
        //         Ok(())
        //     (Schema::Number { .. }, _) => Err(ValidationError::type_mismatch(path, "number"))
        //
        //     (Schema::String { min_length, max_length, pattern }, Value::String(s)) =>
        //         if let Some(min) = min_length:
        //             if s.len() < *min:
        //                 return Err(ValidationError::too_short(path, *min))
        //         if let Some(max) = max_length:
        //             if s.len() > *max:
        //                 return Err(ValidationError::too_long(path, *max))
        //         // TODO: Check regex pattern if present
        //         Ok(())
        //     (Schema::String { .. }, _) => Err(ValidationError::type_mismatch(path, "string"))
        //
        //     (Schema::Array { items, min_items, max_items }, Value::Array(arr)) =>
        //         // Validate array constraints
        //         if let Some(min) = min_items:
        //             if arr.len() < *min:
        //                 return Err(ValidationError::too_few_items(path, *min))
        //         if let Some(max) = max_items:
        //             if arr.len() > *max:
        //                 return Err(ValidationError::too_many_items(path, *max))
        //         // Validate each item recursively
        //         for (i, item) in arr.iter().enumerate():
        //             let item_path = format!("{}[{}]", path, i);
        //             items.validate_at_path(item, &item_path)?;
        //         Ok(())
        //     (Schema::Array { .. }, _) => Err(ValidationError::type_mismatch(path, "array"))
        //
        //     (Schema::Object { properties, required, additional_properties }, Value::Object(obj)) =>
        //         // Check required fields
        //         for req_field in required:
        //             if !obj.contains_key(req_field):
        //                 return Err(ValidationError::missing_field(path, req_field))
        //
        //         // Validate each property
        //         for (key, value) in obj:
        //             let prop_path = format!("{}.{}", path, key);
        //             if let Some(prop_schema) = properties.get(key):
        //                 prop_schema.schema.validate_at_path(value, &prop_path)?;
        //             else if !additional_properties:
        //                 return Err(ValidationError::additional_property(path, key))
        //         Ok(())
        //     (Schema::Object { .. }, _) => Err(ValidationError::type_mismatch(path, "object"))
        //
        //     (Schema::Any, _) => Ok(())
        //
        //     (Schema::OneOf(schemas), value) =>
        //         // Try each schema, succeed if any matches
        //         for schema in schemas:
        //             if schema.validate_at_path(value, path).is_ok():
        //                 return Ok(())
        //         Err(ValidationError::no_schema_matched(path))
        todo!()
    }

    // TODO: Builder methods for common schemas
    pub fn string() -> Self {
        // Pseudocode:
        // Schema::String {
        //     min_length: None,
        //     max_length: None,
        //     pattern: None,
        // }
        todo!()
    }

    pub fn number() -> Self {
        // Pseudocode:
        // Schema::Number {
        //     min: None,
        //     max: None,
        //     integer_only: false,
        // }
        todo!()
    }

    pub fn integer() -> Self {
        // Pseudocode:
        // Schema::Number {
        //     min: None,
        //     max: None,
        //     integer_only: true,
        // }
        todo!()
    }

    pub fn array(items: Schema) -> Self {
        // Pseudocode:
        // Schema::Array {
        //     items: Box::new(items),
        //     min_items: None,
        //     max_items: None,
        // }
        todo!()
    }

    pub fn object() -> ObjectSchemaBuilder {
        // Return builder for fluent API
        todo!()
    }

    // TODO: Chainable constraint methods
    pub fn min(mut self, min: f64) -> Self {
        // Pseudocode:
        // match &mut self:
        //     Schema::Number { min: min_field, .. } =>
        //         *min_field = Some(min);
        //     _ => panic!("min() only valid for Number schema")
        // self
        todo!()
    }

    pub fn max(mut self, max: f64) -> Self {
        // Similar to min()
        todo!()
    }

    pub fn min_length(mut self, len: usize) -> Self {
        // For String schemas
        todo!()
    }

    pub fn max_length(mut self, len: usize) -> Self {
        // For String schemas
        todo!()
    }
}

// TODO: Builder for object schemas
pub struct ObjectSchemaBuilder {
    // TODO: Add fields
    // Hint: properties: HashMap<String, PropertySchema>
    // Hint: required: Vec<String>
    // Hint: additional_properties: bool
}

impl ObjectSchemaBuilder {
    pub fn property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        // Pseudocode:
        // self.properties.insert(name.into(), PropertySchema {
        //     schema,
        //     required: false,
        //     default: None,
        // });
        // self
        todo!()
    }

    pub fn required_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        // Pseudocode:
        // let name_str = name.into();
        // self.properties.insert(name_str.clone(), PropertySchema { ... });
        // self.required.push(name_str);
        // self
        todo!()
    }

    pub fn allow_additional(mut self) -> Self {
        // Pseudocode:
        // self.additional_properties = true;
        // self
        todo!()
    }

    pub fn build(self) -> Schema {
        // Pseudocode:
        // Schema::Object {
        //     properties: self.properties,
        //     required: self.required,
        //     additional_properties: self.additional_properties,
        // }
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
    // TODO: Get value at path using pattern matching
    pub fn get_path(&self, path: &str) -> Option<&Value> {
        // Pseudocode:
        // Parse path into segments
        // Start with self
        // For each segment:
        //     match current value:
        //         Value::Object(map) if segment is field name =>
        //             current = map.get(segment)?
        //         Value::Array(arr) if segment is index =>
        //             current = arr.get(index)?
        //         _ => return None
        // Some(current)
        todo!()
    }

    // TODO: Get multiple values matching path pattern
    pub fn query(&self, path: &str) -> Vec<&Value> {
        // Pseudocode:
        // Parse path into segments
        // Handle wildcards: $.users[*].name
        // Recursively match and collect results
        todo!()
    }

    // TODO: Deep destructuring with pattern matching
    pub fn extract<'a>(&'a self, fields: &[&str]) -> Option<Vec<&'a Value>> {
        // Pseudocode:
        // Extract multiple fields from object
        // match self:
        //     Value::Object(map) =>
        //         let mut values = Vec::new();
        //         for field in fields:
        //             values.push(map.get(*field)?);
        //         Some(values)
        //     _ => None
        todo!()
    }

    // TODO: Array destructuring with pattern matching
    pub fn as_tuple2(&self) -> Option<(&Value, &Value)> {
        // Pseudocode:
        // match self:
        //     Value::Array(arr) if arr.len() == 2 =>
        //         Some((&arr[0], &arr[1]))
        //     _ => None
        todo!()
    }

    pub fn as_tuple3(&self) -> Option<(&Value, &Value, &Value)> {
        // Similar to as_tuple2
        todo!()
    }

    // TODO: Destructure first element and rest
    pub fn split_first(&self) -> Option<(&Value, &[Value])> {
        // Pseudocode:
        // match self:
        //     Value::Array(arr) if !arr.is_empty() =>
        //         Some((&arr[0], &arr[1..]))
        //     _ => None
        todo!()
    }
}

// TODO: Path query parser
#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    // TODO: Add segment types
    // Hint: Field(String) - object field access
    // Hint: Index(usize) - array index access
    // Hint: Wildcard - match all elements
}

fn parse_path(path: &str) -> Vec<PathSegment> {
    // TODO: Parse path string into segments
    // Pseudocode:
    // Split by '.' and parse each segment
    // Handle [index] and [*] syntax
    // Return Vec<PathSegment>
    todo!()
}

// TODO: Query implementation using pattern matching
fn query_recursive<'a>(
    value: &'a Value,
    segments: &[PathSegment],
    results: &mut Vec<&'a Value>,
) {
    // Pseudocode:
    // if segments.is_empty():
    //     results.push(value);
    //     return
    //
    // match (&segments[0], value):
    //     (PathSegment::Field(name), Value::Object(map)) =>
    //         if let Some(next_value) = map.get(name):
    //             query_recursive(next_value, &segments[1..], results)
    //
    //     (PathSegment::Index(i), Value::Array(arr)) =>
    //         if let Some(next_value) = arr.get(*i):
    //             query_recursive(next_value, &segments[1..], results)
    //
    //     (PathSegment::Wildcard, Value::Array(arr)) =>
    //         for item in arr:
    //             query_recursive(item, &segments[1..], results)
    //
    //     (PathSegment::Wildcard, Value::Object(map)) =>
    //         for value in map.values():
    //             query_recursive(value, &segments[1..], results)
    //
    //     _ => // Path doesn't match structure
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
// TODO: Custom validator trait
pub trait Validator: fmt::Debug {
    fn validate(&self, value: &Value) -> Result<(), String>;
}

// TODO: Email validator
#[derive(Debug, Clone)]
pub struct EmailValidator;

impl Validator for EmailValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // Pseudocode:
        // let Value::String(s) = value else {
        //     return Err("not a string".to_string())
        // };
        // if !s.contains('@'):
        //     return Err("invalid email format".to_string())
        // // More thorough regex check
        // Ok(())
        todo!()
    }
}

// TODO: URL validator
#[derive(Debug, Clone)]
pub struct UrlValidator;

impl Validator for UrlValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // Check for valid URL format
        todo!()
    }
}

// TODO: Date validator (ISO 8601)
#[derive(Debug, Clone)]
pub struct DateValidator;

impl Validator for DateValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        // Pseudocode:
        // let Value::String(s) = value else { return Err(...) };
        // Check format: YYYY-MM-DD
        todo!()
    }
}

// TODO: Enhanced Schema with custom validators
#[derive(Debug, Clone)]
pub enum EnhancedSchema {
    // All previous schema types plus:
    // Hint: Custom { validator: Box<dyn Validator>, fallback: Schema }
}

// TODO: Validation context for cross-field validation
pub struct ValidationContext {
    // TODO: Add fields
    // Hint: root: Value - the root document
    // Hint: current_path: String
    // Hint: errors: Vec<ValidationError>
    // Hint: strict: bool
}

impl ValidationContext {
    pub fn new(root: Value, strict: bool) -> Self {
        // Pseudocode:
        // Self {
        //     root,
        //     current_path: "$".to_string(),
        //     errors: Vec::new(),
        //     strict,
        // }
        todo!()
    }

    // TODO: Add error without stopping validation
    pub fn add_error(&mut self, message: String) {
        // Pseudocode:
        // self.errors.push(ValidationError {
        //     path: self.current_path.clone(),
        //     message,
        // });
        todo!()
    }

    // TODO: Validate with path context
    pub fn validate_with_context(
        &mut self,
        schema: &Schema,
        value: &Value,
        path: &str,
    ) {
        // Pseudocode:
        // Save current path
        // Set current_path to path
        // Validate and catch errors
        // Restore previous path
        todo!()
    }

    // TODO: Get all errors or Ok
    pub fn finish(self) -> Result<(), Vec<ValidationError>> {
        // Pseudocode:
        // if self.errors.is_empty():
        //     Ok(())
        // else:
        //     Err(self.errors)
        todo!()
    }
}

// TODO: Example schemas
pub mod schemas {
    use super::*;

    pub fn user_registration_schema() -> Schema {
        // TODO: Define comprehensive user schema
        // Pseudocode:
        // Schema::object()
        //     .required_property("username", Schema::string()
        //         .min_length(3)
        //         .max_length(20)
        //         .pattern("^[a-zA-Z0-9_]+$"))
        //     .required_property("email", Schema::custom(EmailValidator))
        //     .required_property("password", Schema::string()
        //         .min_length(8))
        //     .required_property("age", Schema::integer()
        //         .min(13)
        //         .max(120))
        //     .property("website", Schema::custom(UrlValidator))
        //     .build()
        todo!()
    }

    pub fn api_request_schema() -> Schema {
        // TODO: Define API request schema
        // Pseudocode:
        // Schema::object()
        //     .required_property("method", Schema::OneOf(vec![
        //         Schema::const_string("GET"),
        //         Schema::const_string("POST"),
        //         Schema::const_string("PUT"),
        //         Schema::const_string("DELETE"),
        //     ]))
        //     .required_property("path", Schema::string())
        //     .property("headers", Schema::object()
        //         .allow_additional()
        //         .build())
        //     .property("body", Schema::Any)
        //     .build()
        todo!()
    }

    pub fn config_schema() -> Schema {
        // TODO: Define application configuration schema
        // Pseudocode:
        // Schema::object()
        //     .required_property("server", Schema::object()
        //         .required_property("host", Schema::string())
        //         .required_property("port", Schema::integer()
        //             .min(1)
        //             .max(65535))
        //         .build())
        //     .required_property("database", Schema::object()
        //         .required_property("url", Schema::custom(UrlValidator))
        //         .property("pool_size", Schema::integer()
        //             .min(1)
        //             .max(100))
        //         .build())
        //     .property("logging", Schema::object()
        //         .property("level", Schema::OneOf(vec![
        //             Schema::const_string("debug"),
        //             Schema::const_string("info"),
        //             Schema::const_string("warn"),
        //             Schema::const_string("error"),
        //         ]))
        //         .build())
        //     .build()
        todo!()
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
