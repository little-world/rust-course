use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        let Value::Bool(b) = self else { return None };
        Some(*b)
    }

    pub fn as_number(&self) -> Option<f64> {
        let Value::Number(n) = self else { return None };
        Some(*n)
    }

    pub fn as_string(&self) -> Option<&str> {
        let Value::String(s) = self else { return None };
        Some(s)
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        let Value::Array(a) = self else { return None };
        Some(a)
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        let Value::Object(o) = self else { return None };
        Some(o)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object()?.get(key)
    }

    pub fn get_index(&self, index: usize) -> Option<&Value> {
        self.as_array()?.get(index)
    }

    pub fn get_path(&self, path: &str) -> Option<&Value> {
        let segments = parse_path(path);
        let mut current_value = self;
        for segment in segments {
            match (current_value, &segment) {
                (Value::Object(obj), PathSegment::Field(name)) => {
                    current_value = obj.get(name)?;
                }
                (Value::Array(arr), PathSegment::Index(idx)) => {
                    current_value = arr.get(*idx)?;
                }
                _ => return None,
            }
        }
        Some(current_value)
    }

    pub fn query(&self, path: &str) -> Vec<&Value> {
        let segments = parse_path(path);
        let mut results = Vec::new();
        query_recursive(self, &segments, &mut results);
        results
    }

    pub fn extract<'a>(&'a self, fields: &[&str]) -> Option<Vec<&'a Value>> {
        let obj = self.as_object()?;
        let mut extracted_values = Vec::with_capacity(fields.len());
        for field in fields {
            extracted_values.push(obj.get(*field)?);
        }
        Some(extracted_values)
    }

    pub fn as_tuple2(&self) -> Option<(&Value, &Value)> {
        let arr = self.as_array()?;
        if arr.len() == 2 {
            Some((&arr[0], &arr[1]))
        } else {
            None
        }
    }

    pub fn as_tuple3(&self) -> Option<(&Value, &Value, &Value)> {
        let arr = self.as_array()?;
        if arr.len() == 3 {
            Some((&arr[0], &arr[1], &arr[2]))
        } else {
            None
        }
    }

    pub fn split_first(&self) -> Option<(&Value, &[Value])> {
        let arr = self.as_array()?;
        if arr.is_empty() {
            None
        } else {
            Some((&arr[0], &arr[1..]))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace, RightBrace, LeftBracket, RightBracket,
    Colon, Comma,
    String(String),
    Number(f64),
    True, False, Null,
    Eof
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer { input, position: 0 }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.input[self.position..]
            .chars()
            .nth(offset)
    }

    fn advance(&mut self) {
        if self.position < self.input.len() {
            self.position += self.current_char().map_or(1, |c| c.len_utf8());
        }
    }

    fn expect_char(&mut self, expected: char) -> Result<(), ParseError> {
        if self.current_char() == Some(expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected '{}'", expected),
                position: self.position,
            })
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_string(&mut self) -> Result<Token, ParseError> {
        self.expect_char('"')?;
        let start_pos = self.position;
        let mut value = String::new();

        while let Some(c) = self.current_char() {
            if c == '"' {
                self.advance();
                return Ok(Token::String(value));
            } else if c == '\\' {
                self.advance();
                match self.current_char() {
                    Some('"') => { value.push('"'); self.advance(); }
                    Some('\\') => { value.push('\\'); self.advance(); }
                    Some('/') => { value.push('/'); self.advance(); }
                    Some('b') => { value.push('\x08'); self.advance(); }
                    Some('f') => { value.push('\x0C'); self.advance(); }
                    Some('n') => { value.push('\n'); self.advance(); }
                    Some('r') => { value.push('\r'); self.advance(); }
                    Some('t') => { value.push('\t'); self.advance(); }
                    Some('u') => {
                        self.advance();
                        let hex_start = self.position;
                        let mut hex_chars = self.input[hex_start..].chars().take(4);
                        let hex_str: String = hex_chars.collect();

                        if hex_str.len() == 4 {
                            if let Ok(code_point) = u16::from_str_radix(&hex_str, 16) {
                                if let Some(ch) = char::from_u32(code_point as u32) {
                                    value.push(ch);
                                    self.position += 4;
                                } else {
                                    return Err(ParseError {
                                        message: "Invalid Unicode code point".to_string(),
                                        position: hex_start,
                                    });
                                }
                            } else {
                                return Err(ParseError {
                                    message: "Invalid hex digits for Unicode escape".to_string(),
                                    position: hex_start,
                                });
                            }
                        } else {
                            return Err(ParseError {
                                message: "Incomplete Unicode escape sequence".to_string(),
                                position: hex_start,
                            });
                        }
                    }
                    _ => return Err(ParseError {
                        message: "Invalid escape sequence".to_string(),
                        position: self.position,
                    }),
                }
            } else {
                value.push(c);
                self.advance();
            }
        }
        Err(ParseError {
            message: "Unterminated string".to_string(),
            position: start_pos,
        })
    }

    fn parse_number(&mut self) -> Result<Token, ParseError> {
        let start = self.position;
        let mut end = self.position;

        // Optional minus sign
        if self.current_char() == Some('-') {
            self.advance();
            end = self.position;
        }

        // Integer part
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                self.advance();
                end = self.position;
            } else {
                break;
            }
        }

        // Fractional part
        if self.current_char() == Some('.') {
            self.advance();
            end = self.position;
            while let Some(c) = self.current_char() {
                if c.is_ascii_digit() {
                    self.advance();
                    end = self.position;
                } else {
                    break;
                }
            }
        }

        // Exponent part
        if self.current_char() == Some('e') || self.current_char() == Some('E') {
            self.advance();
            end = self.position;
            if self.current_char() == Some('-') || self.current_char() == Some('+') {
                self.advance();
                end = self.position;
            }
            while let Some(c) = self.current_char() {
                if c.is_ascii_digit() {
                    self.advance();
                    end = self.position;
                } else {
                    break;
                }
            }
        }

        let num_str = &self.input[start..end];
        if num_str.is_empty() {
            return Err(ParseError {
                message: "Expected number".to_string(),
                position: start,
            });
        }
        num_str.parse::<f64>().map(Token::Number).map_err(|_| ParseError {
            message: "Invalid number format".to_string(),
            position: start,
        })
    }


    fn parse_keyword(&mut self, expected: &str, token: Token) -> Result<Token, ParseError> {
        let start = self.position;
        let end_pos = self.position + expected.len();
        if end_pos <= self.input.len() && &self.input[self.position..end_pos] == expected {
            self.position = end_pos;
            Ok(token)
        } else {
            Err(ParseError {
                message: format!("Expected '{}'", expected),
                position: start,
            })
        }
    }

    pub fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace();

        let current_pos = self.position;
        let Some(c) = self.current_char() else { return Ok(Token::Eof); };

        match c {
            '{' => { self.advance(); Ok(Token::LeftBrace) }
            '}' => { self.advance(); Ok(Token::RightBrace) }
            '[' => { self.advance(); Ok(Token::LeftBracket) }
            ']' => { self.advance(); Ok(Token::RightBracket) }
            ':' => { self.advance(); Ok(Token::Colon) }
            ',' => { self.advance(); Ok(Token::Comma) }
            '"' => self.parse_string(),
            '-' | '0'..='9' => self.parse_number(),
            't' => self.parse_keyword("true", Token::True),
            'f' => self.parse_keyword("false", Token::False),
            'n' => self.parse_keyword("null", Token::Null),
            _ => Err(ParseError {
                message: format!("Unexpected character: '{}'", c),
                position: current_pos,
            }),
        }
    }
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

const MAX_NESTING_DEPTH: usize = 100;

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

    fn expect(&mut self, expected_token: Token) -> Result<(), ParseError> {
        if self.current_token == expected_token {
            self.advance()?;
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected_token, self.current_token),
                position: self.lexer.position,
            })
        }
    }

    pub fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.parse_value_with_depth(0)
    }

    fn parse_value_with_depth(&mut self, depth: usize) -> Result<Value, ParseError> {
        if depth > MAX_NESTING_DEPTH {
            return Err(ParseError {
                message: format!("Exceeded maximum nesting depth of {}", MAX_NESTING_DEPTH),
                position: self.lexer.position,
            });
        }

        let value = match self.current_token.clone() {
            Token::Null => {
                self.advance()?;
                Value::Null
            }
            Token::True => {
                self.advance()?;
                Value::Bool(true)
            }
            Token::False => {
                self.advance()?;
                Value::Bool(false)
            }
            Token::Number(n) => {
                self.advance()?;
                Value::Number(n)
            }
            Token::String(s) => {
                self.advance()?;
                Value::String(s)
            }
            Token::LeftBracket => self.parse_array(depth + 1)?,
            Token::LeftBrace => self.parse_object(depth + 1)?,
            _ => {
                return Err(ParseError {
                    message: format!("Unexpected token: {:?}", self.current_token),
                    position: self.lexer.position,
                });
            }
        };
        Ok(value)
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.expect(Token::LeftBracket)?;
        let mut elements = Vec::new();

        if self.current_token == Token::RightBracket {
            self.advance()?;
            return Ok(Value::Array(elements));
        }

        loop {
            elements.push(self.parse_value_with_depth(depth)?);

            if self.current_token == Token::RightBracket {
                self.advance()?;
                break;
            }

            self.expect(Token::Comma)?;
            if self.current_token == Token::RightBracket {
                self.advance()?;
                break;
            }
        }
        Ok(Value::Array(elements))
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, ParseError> {
        self.expect(Token::LeftBrace)?;
        let mut properties = HashMap::new();

        if self.current_token == Token::RightBrace {
            self.advance()?;
            return Ok(Value::Object(properties));
        }

        loop {
            let key = match self.current_token.clone() {
                Token::String(s) => {
                    self.advance()?;
                    s
                }
                _ => {
                    return Err(ParseError {
                        message: format!("Expected string key, found {:?}", self.current_token),
                        position: self.lexer.position,
                    });
                }
            };

            self.expect(Token::Colon)?;
            let value = self.parse_value_with_depth(depth)?;
            properties.insert(key, value);

            if self.current_token == Token::RightBrace {
                self.advance()?;
                break;
            }

            self.expect(Token::Comma)?;
            if self.current_token == Token::RightBrace {
                self.advance()?;
                break;
            }
        }
        Ok(Value::Object(properties))
    }
}

pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(input)?;
    let value = parser.parse_value()?;
    if parser.current_token != Token::Eof {
        return Err(ParseError {
            message: format!("Unexpected token at end of input: {:?}", parser.current_token),
            position: parser.lexer.position,
        });
    }
    Ok(value)
}


#[derive(Debug, Clone, PartialEq)]
enum PathSegment {
    Field(String),
    Index(usize),
    Wildcard,
}

fn parse_path(path: &str) -> Vec<PathSegment> {
    let mut segments = Vec::new();
    let mut current_segment = String::new();
    let mut in_bracket = false;

    let chars: Vec<char> = path.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '.' if !in_bracket => {
                if !current_segment.is_empty() {
                    segments.push(PathSegment::Field(current_segment.clone()));
                    current_segment.clear();
                }
                i += 1;
            }
            '[' => {
                if !current_segment.is_empty() {
                    segments.push(PathSegment::Field(current_segment.clone()));
                    current_segment.clear();
                }
                in_bracket = true;
                i += 1;
            }
            ']' => {
                if in_bracket {
                    if current_segment == "*" {
                        segments.push(PathSegment::Wildcard);
                    } else if let Ok(idx) = current_segment.parse::<usize>() {
                        segments.push(PathSegment::Index(idx));
                    } else {
                        segments.push(PathSegment::Field(current_segment.clone()));
                    }
                    current_segment.clear();
                    in_bracket = false;
                }
                i += 1;
            }
            _ => {
                current_segment.push(chars[i]);
                i += 1;
            }
        }
    }

    if !current_segment.is_empty() {
        segments.push(PathSegment::Field(current_segment));
    }

    segments
}

fn query_recursive<'a>(
    value: &'a Value,
    segments: &[PathSegment],
    results: &mut Vec<&'a Value>,
) {
    if segments.is_empty() {
        results.push(value);
        return;
    }

    let current_segment = &segments[0];
    let remaining_segments = &segments[1..];

    match (value, current_segment) {
        (Value::Object(obj), PathSegment::Field(name)) => {
            if let Some(field_value) = obj.get(name) {
                query_recursive(field_value, remaining_segments, results);
            }
        }
        (Value::Object(obj), PathSegment::Wildcard) => {
            for field_value in obj.values() {
                query_recursive(field_value, remaining_segments, results);
            }
        }
        (Value::Array(arr), PathSegment::Index(idx)) => {
            if let Some(item_value) = arr.get(*idx) {
                query_recursive(item_value, remaining_segments, results);
            }
        }
        (Value::Array(arr), PathSegment::Wildcard) => {
            for item_value in arr {
                query_recursive(item_value, remaining_segments, results);
            }
        }
        _ => { /* Path does not match value type, do nothing */ }
    }
}


pub trait Validator: fmt::Debug {
    fn validate(&self, value: &Value) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct EmailValidator;

impl Validator for EmailValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let s = value.as_string().ok_or("Expected string for email validation".to_string())?;
        if s.contains('@') && s.contains('.') {
            Ok(())
        } else {
            Err(format!("'{}' is not a valid email address", s))
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlValidator;

impl Validator for UrlValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let s = value.as_string().ok_or("Expected string for URL validation".to_string())?;
        if s.starts_with("http://") || s.starts_with("https://") {
            Ok(())
        } else {
            Err(format!("'{}' is not a valid URL", s))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateValidator;

impl Validator for DateValidator {
    fn validate(&self, value: &Value) -> Result<(), String> {
        let s = value.as_string().ok_or("Expected string for date validation".to_string())?;
        if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') &&
           s.chars().take(4).all(|c| c.is_ascii_digit()) &&
           s.chars().skip(5).take(2).all(|c| c.is_ascii_digit()) &&
           s.chars().skip(8).take(2).all(|c| c.is_ascii_digit())
        {
            Ok(())
        } else {
            Err(format!("'{}' is not a valid YYYY-MM-DD date format", s))
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct PropertySchema {
    pub schema: Schema,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error at {}: {}", self.path, self.message)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Schema {
    Null,
    Bool,
    Number { min: Option<f64>, max: Option<f64>, integer_only: bool },
    String { min_length: Option<usize>, max_length: Option<usize>, pattern: Option<String> },
    Array { items: Box<Schema>, min_items: Option<usize>, max_items: Option<usize> },
    Object { properties: HashMap<String, PropertySchema>, required: Vec<String>, additional_properties: bool },
    Any,
    OneOf(Vec<Schema>),
    Custom(Box<dyn Validator + 'static>),
    Const(Value),
}

impl Schema {
    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        self.validate_at_path(value, "$")
    }

    fn validate_at_path(&self, value: &Value, path: &str) -> Result<(), ValidationError> {
        match self {
            Schema::Null => {
                if !value.is_null() {
                    return Err(ValidationError {
                        path: path.to_string(),
                        message: "Expected null".to_string(),
                    });
                }
            }
            Schema::Bool => {
                if !value.is_bool() {
                    return Err(ValidationError {
                        path: path.to_string(),
                        message: "Expected boolean".to_string(),
                    });
                }
            }
            Schema::Number { min, max, integer_only } => {
                let Some(n) = value.as_number() else { return Err(ValidationError { path: path.to_string(), message: "Expected number".to_string() }); };
                if let Some(min_val) = min {
                    if n < *min_val {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("Value {} is below minimum {}", n, min_val),
                        });
                    }
                }
                if let Some(max_val) = max {
                    if n > *max_val {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("Value {} is above maximum {}", n, max_val),
                        });
                    }
                }
                if *integer_only && n.fract() != 0.0 {
                    return Err(ValidationError {
                        path: path.to_string(),
                        message: format!("Value {} is not an integer", n),
                    });
                }
            }
            Schema::String { min_length, max_length, pattern } => {
                let Some(s) = value.as_string() else { return Err(ValidationError { path: path.to_string(), message: "Expected string".to_string() }); };
                if let Some(min_len) = min_length {
                    if s.len() < *min_len {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("String length {} is below minimum length {}", s.len(), min_len),
                        });
                    }
                }
                if let Some(max_len) = max_length {
                    if s.len() > *max_len {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("String length {} is above maximum length {}", s.len(), max_len),
                        });
                    }
                }
                if let Some(regex_pattern) = pattern {
                    if !s.contains(regex_pattern) {
                         return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("String '{}' does not match pattern '{}'", s, regex_pattern),
                        });
                    }
                }
            }
            Schema::Array { items, min_items, max_items } => {
                let Some(arr) = value.as_array() else { return Err(ValidationError { path: path.to_string(), message: "Expected array".to_string() }); };
                if let Some(min_i) = min_items {
                    if arr.len() < *min_i {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("Array has {} items, minimum is {}", arr.len(), min_i),
                        });
                    }
                }
                if let Some(max_i) = max_items {
                    if arr.len() > *max_i {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("Array has {} items, maximum is {}", arr.len(), max_i),
                        });
                    }
                }
                for (i, item_value) in arr.iter().enumerate() {
                    let item_path = format!("{}[{}]", path, i);
                    items.validate_at_path(item_value, &item_path)?;
                }
            }
            Schema::Object { properties, required, additional_properties } => {
                let Some(obj) = value.as_object() else { return Err(ValidationError { path: path.to_string(), message: "Expected object".to_string() }); };

                for key in required {
                    if !obj.contains_key(key) {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("Missing required property '{}'", key),
                        });
                    }
                }

                for (key, prop_value) in obj {
                    if let Some(prop_schema) = properties.get(key) {
                        let prop_path = format!("{}.{}", path, key);
                        prop_schema.schema.validate_at_path(prop_value, &prop_path)?;
                    } else if !additional_properties {
                        return Err(ValidationError {
                            path: path.to_string(),
                            message: format!("Unexpected property '{}'", key),
                        });
                    }
                }
            }
            Schema::Any => { /* always valid */ }
            Schema::OneOf(schemas) => {
                let mut one_of_errors = Vec::new();
                for s in schemas {
                    if s.validate_at_path(value, path).is_ok() { return Ok(()); }
                    else {
                        if let Err(e) = s.validate_at_path(value, path) { one_of_errors.push(e); }
                    }
                }
                return Err(ValidationError {
                    path: path.to_string(),
                    message: format!("Value does not match any of the provided schemas. Individual errors: {:?}", one_of_errors),
                });
            }
            Schema::Custom(validator) => {
                validator.validate(value).map_err(|msg| ValidationError {
                    path: path.to_string(),
                    message: msg,
                })?;
            }
            Schema::Const(expected_value) => {
                if value != expected_value {
                    return Err(ValidationError {
                        path: path.to_string(),
                        message: format!("Expected {:?}, found {:?}", expected_value, value),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn string() -> Self {
        Schema::String {
            min_length: None,
            max_length: None,
            pattern: None,
        }
    }

    pub fn number() -> Self {
        Schema::Number { min: None, max: None, integer_only: false }
    }

    pub fn integer() -> Self {
        Schema::Number { min: None, max: None, integer_only: true }
    }

    pub fn array(items: Schema) -> Self {
        Schema::Array {
            items: Box::new(items),
            min_items: None,
            max_items: None,
        }
    }

    pub fn object() -> ObjectSchemaBuilder {
        ObjectSchemaBuilder {
            properties: HashMap::new(),
            required: Vec::new(),
            additional_properties: false,
        }
    }

    pub fn any() -> Self {
        Schema::Any
    }

    pub fn one_of(schemas: Vec<Schema>) -> Self {
        Schema::OneOf(schemas)
    }

    pub fn const_string(s: impl Into<String>) -> Self {
        Schema::Const(Value::String(s.into()))
    }
    
    pub fn custom(validator: impl Validator + 'static) -> Self {
        Schema::Custom(Box::new(validator))
    }

    pub fn min(mut self, min: f64) -> Self {
        match &mut self {
            Schema::Number { min: min_field, .. } => *min_field = Some(min),
            _ => panic!("min() only valid for Number schema")
        }
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        match &mut self {
            Schema::Number { max: max_field, .. } => *max_field = Some(max),
            _ => panic!("max() only valid for Number schema")
        }
        self
    }

    pub fn min_length(mut self, len: usize) -> Self {
        match &mut self {
            Schema::String { min_length: len_field, .. } => *len_field = Some(len),
            _ => panic!("min_length() only valid for String schema")
        }
        self
    }

    pub fn max_length(mut self, len: usize) -> Self {
        match &mut self {
            Schema::String { max_length: len_field, .. } => *len_field = Some(len),
            _ => panic!("max_length() only valid for String schema")
        }
        self
    }

    pub fn pattern(mut self, regex_pattern: impl Into<String>) -> Self {
        match &mut self {
            Schema::String { pattern: pattern_field, .. } => *pattern_field = Some(regex_pattern.into()),
            _ => panic!("pattern() only valid for String schema")
        }
        self
    }

    pub fn min_items(mut self, min: usize) -> Self {
        match &mut self {
            Schema::Array { min_items: min_field, .. } => *min_field = Some(min),
            _ => panic!("min_items() only valid for Array schema")
        }
        self
    }

    pub fn max_items(mut self, max: usize) -> Self {
        match &mut self {
            Schema::Array { max_items: max_field, .. } => *max_field = Some(max),
            _ => panic!("max_items() only valid for Array schema")
        }
        self
    }
}

pub struct ObjectSchemaBuilder {
    properties: HashMap<String, PropertySchema>,
    required: Vec<String>,
    additional_properties: bool,
}

impl ObjectSchemaBuilder {
    pub fn property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        self.properties.insert(name.into(), PropertySchema { schema, required: false });
        self
    }

    pub fn required_property(mut self, name: impl Into<String>, schema: Schema) -> Self {
        let name_str = name.into();
        self.properties.insert(name_str.clone(), PropertySchema { schema, required: true });
        self.required.push(name_str);
        self
    }

    pub fn allow_additional(mut self) -> Self {
        self.additional_properties = true;
        self
    }

    pub fn build(self) -> Schema {
        Schema::Object {
            properties: self.properties,
            required: self.required,
            additional_properties: self.additional_properties,
        }
    }
}

pub struct ValidationContext {
    root_value: Value,
    current_path: String,
    errors: Vec<ValidationError>,
    strict_mode: bool,
}

impl ValidationContext {
    pub fn new(root: Value, strict: bool) -> Self {
        ValidationContext {
            root_value: root,
            current_path: "$".to_string(),
            errors: Vec::new(),
            strict_mode: strict,
        }
    }

    pub fn add_error(&mut self, message: String) {
        self.errors.push(ValidationError {
            path: self.current_path.clone(),
            message,
        });
    }

    pub fn validate_with_context(
        &mut self,
        schema: &Schema,
        value: &Value,
        path: &str,
    ) -> Result<(), ValidationError> {
        let prev_path = self.current_path.clone();
        self.current_path = path.to_string();

        let result = schema.validate_at_path(value, path);

        self.current_path = prev_path;

        if let Err(e) = result {
            self.add_error(e.message);
            if self.strict_mode {
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn finish(self) -> Result<(), Vec<ValidationError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }
}

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
                .min(13.0)
                .max(120.0))
            .property("website", Schema::custom(UrlValidator))
            .build()
    }

    pub fn api_request_schema() -> Schema {
        Schema::object()
            .required_property("method", Schema::one_of(vec![
                Schema::const_string("GET"),
                Schema::const_string("POST"),
                Schema::const_string("PUT"),
                Schema::const_string("DELETE"),
            ]))
            .required_property("path", Schema::string())
            .property("headers", Schema::object()
                .allow_additional()
                .build())
            .property("body", Schema::any())
            .build()
    }

    pub fn config_schema() -> Schema {
        Schema::object()
            .required_property("server", Schema::object()
                .required_property("host", Schema::string())
                .required_property("port", Schema::integer()
                    .min(1.0)
                    .max(65535.0))
                .build())
            .required_property("database", Schema::object()
                .required_property("url", Schema::custom(UrlValidator))
                .property("pool_size", Schema::integer()
                    .min(1.0)
                    .max(100.0))
                .build())
            .property("logging", Schema::object()
                .property("level", Schema::one_of(vec![
                    Schema::const_string("debug"),
                    Schema::const_string("info"),
                    Schema::const_string("warn"),
                    Schema::const_string("error"),
                ]))
                .build())
            .build()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone)]
    struct MockValidator {
        is_valid: bool,
    }

    impl Validator for MockValidator {
        fn validate(&self, _value: &Value) -> Result<(), String> {
            if self.is_valid {
                Ok(())
            } else {
                Err("Mock validation failed".to_string())
            }
        }
    }


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
        
        let value_exp = parse("1.2e+3").unwrap();
        assert_eq!(value_exp, Value::Number(1200.0));
    }

    #[test]
    fn test_parse_string() {
        let value = parse(r#"hello"#).unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
        assert_eq!(value.as_string(), Some("hello"));
    }

    #[test]
    fn test_parse_string_with_escapes() {
        let value = parse(r#"hello\nworld"#).unwrap();
        assert_eq!(value, Value::String("hello\nworld".to_string()));

        let value = parse(r#"quote: \"test\""#).unwrap();
        assert_eq!(value, Value::String(r#"quote: "test""#.to_string()));

        let value = parse(r#"slash: \/"#).unwrap();
        assert_eq!(value, Value::String("/".to_string()));

        let value = parse(r#"backsl: \\"#).unwrap();
        assert_eq!(value, Value::String("\\".to_string()));
        
        let value = parse(r#"unicode: \u0041"#).unwrap();
        assert_eq!(value, Value::String("unicode: A".to_string()));
    }

    #[test]
    fn test_parse_error() {
        let result = parse("invalid");
        assert!(result.is_err());

        let result = parse("tru"); // Incomplete true
        assert!(result.is_err());

        let result = parse(r#"unclosed string"#);
        assert!(result.is_err());

        let result = parse(r#"{"key": unexp}"#);
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
        let json = r#"{
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ],
            "count": 2
        }"#;

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

        let a = value.get("a").unwrap();
        let b = a.get("b").unwrap();
        let c = b.get("c").unwrap();
        let d = c.get("d").unwrap();
        let e = d.get("e").unwrap();

        assert_eq!(e, &Value::Number(5.0));
    }

    #[test]
    fn test_prevent_stack_overflow() {
        let mut json = String::new();
        for _ in 0..200 {
            json.push('[');
        }
        for _ in 0..200 {
            json.push(']');
        }

        let result = parse(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Exceeded maximum nesting depth"));
    }

    #[test]
    fn test_malformed_array() {
        assert!(parse("[1, 2,").is_err());
        assert!(parse("[1 2]").is_err());
        assert!(parse("[1, 2, ]").is_err());
        assert!(parse("[,1,2]").is_err());
    }

    #[test]
    fn test_malformed_object() {
        assert!(parse(r#"{"key": "value""#).is_err());
        assert!(parse(r#"{"key" "value"}"#).is_err());
        assert!(parse(r#"{key: "value"}"#).is_err());
        assert!(parse(r#"{"key": "value",}"#).is_err());
    }

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
        assert!(err.message.contains("below minimum"));

        let err = schema.validate(&Value::Number(150.0)).unwrap_err();
        assert!(err.message.contains("above maximum"));
    }

    #[test]
    fn test_validate_integer_only() {
        let schema = Schema::integer();

        assert!(schema.validate(&Value::Number(42.0)).is_ok());
        assert!(schema.validate(&Value::Number(3.14)).is_err());
        assert!(schema.validate(&Value::Bool(true)).is_err());
    }

    #[test]
    fn test_validate_string_length() {
        let schema = Schema::string().min_length(3).max_length(10);

        assert!(schema.validate(&Value::String("hello".to_string())).is_ok());

        let err = schema.validate(&Value::String("ab".to_string())).unwrap_err();
        assert!(err.message.contains("below minimum length"));

        let err = schema.validate(&Value::String("this is too long".to_string())).unwrap_err();
        assert!(err.message.contains("above maximum length"));
    }

    #[test]
    fn test_validate_string_pattern() {
        let schema = Schema::string().pattern("abc");
        assert!(schema.validate(&Value::String("xabcy".to_string())).is_ok());
        let err = schema.validate(&Value::String("xyz".to_string())).unwrap_err();
        assert!(err.message.contains("does not match pattern"));
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
        assert!(err.message.contains("minimum is"));

        let err = schema.validate(&Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(6.0),
        ])).unwrap_err();
        assert!(err.message.contains("maximum is"));
    }

    #[test]
    fn test_validate_object() {
        let schema = Schema::object()
            .required_property("name", Schema::string())
            .required_property("age", Schema::integer().min(0.0))
            .property("email", Schema::string())
            .build();

        let mut valid_map = HashMap::new();
        valid_map.insert("name".to_string(), Value::String("Alice".to_string()));
        valid_map.insert("age".to_string(), Value::Number(30.0));
        valid_map.insert("email".to_string(), Value::String("alice@example.com".to_string()));

        assert!(schema.validate(&Value::Object(valid_map)).is_ok());

        // Missing required field
        let mut invalid_map = HashMap::new();
        invalid_map.insert("name".to_string(), Value::String("Bob".to_string()));
        let err = schema.validate(&Value::Object(invalid_map)).unwrap_err();
        assert!(err.message.contains("Missing required property 'age'"));
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

        let json = r#"{
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        }"#;

        let value = parse(json).unwrap();
        assert!(schema.validate(&value).is_ok());

        // Invalid: user missing required field
        let json_invalid = r#"{
            "users": [
                {"id": 1}
            ]
        }"#;

        let value = parse(json_invalid).unwrap();
        let err = schema.validate(&value).unwrap_err();
        assert!(err.path.contains("users[0]"));
        assert!(err.message.contains("Missing required property 'name'"));
    }

    #[test]
    fn test_validate_one_of() {
        let schema = Schema::one_of(vec![
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
        assert!(err.message.contains("below minimum"));
    }
    
    #[test]
    fn test_get_path_simple() {
        let json = r#