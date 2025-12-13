// src/bin/complete_06_zero_copy.rs

pub struct ParserContext<'ctx> {
    keywords: &'ctx [&'ctx str],
    operators: &'ctx [char],
}

impl<'ctx> ParserContext<'ctx> {
    pub fn new(keywords: &'ctx [&'ctx str], operators: &'ctx [char]) -> Self {
        Self { keywords, operators }
    }

    pub fn is_keyword(&self, word: &str) -> bool {
        self.keywords.contains(&word)
    }

    pub fn is_operator(&self, ch: char) -> bool {
        self.operators.contains(&ch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token<'a> {
    Identifier(&'a str),
    Number(&'a str),
    String(&'a str),
    Symbol(char),
    Eof,
}

impl<'a> Token<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Token::Identifier(s) | Token::Number(s) | Token::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_keyword(&self, word: &str) -> bool {
        matches!(self, Token::Identifier(s) if *s == word)
    }

    pub fn is_eof(&self) -> bool {
        matches!(self, Token::Eof)
    }
}

pub struct Parser<'input, 'ctx> {
    input: &'input str,
    position: usize,
    context: &'ctx ParserContext<'ctx>,
}

impl<'input, 'ctx> Parser<'input, 'ctx> {
    pub fn new(input: &'input str, context: &'ctx ParserContext<'ctx>) -> Self {
        Self { input, position: 0, context }
    }

    // Helper to skip whitespace
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input.as_bytes()[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
    }

    // This is the core tokenization logic, replacing previous peek/advance
    pub fn next_token(&mut self) -> Token<'input> {
        self.skip_whitespace();

        if self.position == self.input.len() {
            return Token::Eof;
        }

        let start = self.position;
        let current_char = self.input[start..].chars().next().unwrap();

        // Handle string literals
        if current_char == '"' {
            let mut end = start + 1; // Skip opening quote
            while end < self.input.len() {
                if self.input.as_bytes()[end] == b'"' {
                    self.position = end + 1; // Move past closing quote
                    return Token::String(&self.input[start + 1..end]);
                }
                end += 1;
            }
            // If unterminated string, advance past the opening quote and treat it as a symbol for now
            self.position += 1;
            return Token::Symbol('"');
        }

        // Handle numbers
        if current_char.is_ascii_digit() || (current_char == '-' && self.input.as_bytes().get(start + 1).map_or(false, |&c| (c as char).is_ascii_digit())) {
            let mut end = start;
            // Check for hexadecimal prefix
            if current_char == '0' && self.input.as_bytes().get(start + 1).map_or(false, |&c| (c == b'x' || c == b'X')) {
                end += 2; // Skip "0x" or "0X"
                while end < self.input.len() && (self.input.as_bytes()[end] as char).is_ascii_hexdigit() {
                    end += 1;
                }
            } else {
                // Decimal or integer part
                while end < self.input.len() && (self.input.as_bytes()[end] as char).is_ascii_digit() {
                    end += 1;
                }
                // Fractional part
                if end < self.input.len() && self.input.as_bytes()[end] == b'.'{ 
                    end += 1;
                    while end < self.input.len() && (self.input.as_bytes()[end] as char).is_ascii_digit() {
                        end += 1;
                    }
                }
            }

            if end > start {
                self.position = end;
                return Token::Number(&self.input[start..end]);
            }
        }

        // Handle single-char symbols using context
        if self.context.is_operator(current_char) {
            self.position += current_char.len_utf8();
            return Token::Symbol(current_char);
        }

        // Handle identifiers (alphanumeric + underscore)
        if current_char.is_alphabetic() || current_char == '_' {
            let mut end = start;
            while end < self.input.len() {
                let ch = self.input[end..].chars().next().unwrap();
                if ch.is_alphanumeric() || ch == '_' {
                    end += ch.len_utf8();
                } else {
                    break;
                }
            }
            self.position = end;
            return Token::Identifier(&self.input[start..end]);
        }

        // Fallback for unhandled characters
        self.position += current_char.len_utf8();
        Token::Symbol(current_char) // Treat as symbol for now
    }

    // Returns unparsed portion of input
    pub fn remaining(&self) -> &'input str {
        &self.input[self.position..]
    }

    // Check if remaining input is empty after skipping whitespace
    pub fn is_empty(&self) -> bool {
        let mut temp_pos = self.position;
        while temp_pos < self.input.len() && self.input.as_bytes()[temp_pos].is_ascii_whitespace() {
            temp_pos += 1;
        }
        temp_pos == self.input.len()
    }

    // Returns reference with 'ctx lifetime
    pub fn get_keywords(&self) -> &'ctx [&'ctx str] {
        self.context.keywords
    }

    // Find next token that is an Identifier and is a keyword
    pub fn next_keyword_token(&mut self) -> Option<&'input str> {
        loop {
            match self.next_token() {
                Token::Identifier(s) => {
                    if self.context.is_keyword(s) {
                        return Some(s);
                    }
                }
                Token::Eof => return None,
                _ => {}
            }
        }
    }
}

// Iterator implementation for Parser
impl<'input, 'ctx> Iterator for Parser<'input, 'ctx> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.is_eof() {
            None
        } else {
            Some(token)
        }
    }
}

// IntoIterator implementation for Parser
impl<'input, 'ctx> IntoIterator for Parser<'input, 'ctx> {
    type Item = Token<'input>;
    type IntoIter = Self;

    fn into_iter(self) -> Self::IntoIter {
        self
    }
}

// Iterator adapter methods for Parser
impl<'input, 'ctx> Parser<'input, 'ctx> {
    // Collect all identifiers
    pub fn identifiers(&mut self) -> Vec<&'input str> {
        self.filter_map(|token| match token {
            Token::Identifier(s) => Some(s),
            _ => None,
        }).collect()
    }

    // Check if keyword exists in input
    pub fn has_keyword(&mut self, keyword: &str) -> bool {
        // Use self.into_iter() to consume the parser and check all tokens
        let mut temp_parser = Parser::new(self.input, self.context); // Create a temporary parser
        temp_parser.position = self.position; // Start from current position
        temp_parser.any(|token| token.is_keyword(keyword))
    }

    // Count tokens of a specific type
    pub fn count_numbers(&mut self) -> usize {
        self.filter(|token| matches!(token, Token::Number(_))).count()
    }

    // Collect all identifier/number pairs
    pub fn extract_assignments(&mut self) -> Vec<(&'input str, &'input str)> {
        let mut result = Vec::new();
        // Use an internal parser instance to avoid consuming the original `self`
        let temp_input = self.remaining();
        let mut temp_parser = Parser::new(temp_input, self.context);

        let tokens: Vec<Token> = temp_parser.into_iter().collect();

        // Use windows to find patterns like "name = value"
        for window in tokens.windows(3) {
            if let [Token::Identifier(name), Token::Symbol('='), Token::Number(value)] = window {
                result.push((*name, *value));
            }
        }
        result
    }

    // Group consecutive identifiers
    pub fn group_identifiers(&mut self) -> Vec<Vec<&'input str>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();

        // Use an internal parser instance to avoid consuming the original `self`
        let temp_input = self.remaining();
        let mut temp_parser = Parser::new(temp_input, self.context);

        for token in temp_parser {
            match token {
                Token::Identifier(s) => current_group.push(s),
                _ => {
                    if !current_group.is_empty() {
                        groups.push(current_group);
                        current_group = Vec::new();
                    }
                }
            }
        }

        if !current_group.is_empty() {
            groups.push(current_group);
        }

        groups
    }
}

// Allocating parser for comparison
pub struct AllocatingParser {
    input: String,
    position: usize,
}

impl AllocatingParser {
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    // Helper to skip whitespace
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input.as_bytes()[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
    }

    pub fn next_token(&mut self) -> Option<String> {
        self.skip_whitespace();

        if self.position == self.input.len() {
            return None;
        }

        let start = self.position;
        let current_char = self.input[start..].chars().next().unwrap();

        // Handle string literals
        if current_char == '"' {
            let mut end = start + 1;
            while end < self.input.len() {
                if self.input.as_bytes()[end] == b'"' {
                    let s = self.input[start + 1..end].to_string();
                    self.position = end + 1;
                    return Some(s);
                }
                end += 1;
            }
            self.position += 1; // Consume the opening quote even if unterminated
            return Some("\"".to_string());
        }

        // Handle numbers
        if current_char.is_ascii_digit() || (current_char == '-' && self.input.as_bytes().get(start + 1).map_or(false, |&c| (c as char).is_ascii_digit())) {
            let mut end = start;
            if current_char == '0' && self.input.as_bytes().get(start + 1).map_or(false, |&c| (c == b'x' || c == b'X')) {
                end += 2;
                while end < self.input.len() && (self.input.as_bytes()[end] as char).is_ascii_hexdigit() {
                    end += 1;
                }
            } else {
                while end < self.input.len() && (self.input.as_bytes()[end] as char).is_ascii_digit() {
                    end += 1;
                }
                if end < self.input.len() && self.input.as_bytes()[end] == b'.'{ 
                    end += 1;
                    while end < self.input.len() && (self.input.as_bytes()[end] as char).is_ascii_digit() {
                        end += 1;
                    }
                }
            }
            if end > start {
                let s = self.input[start..end].to_string();
                self.position = end;
                return Some(s);
            }
        }

        // Handle symbols or identifiers
        let mut end = start;
        while end < self.input.len() {
            let ch = self.input[end..].chars().next().unwrap();
            // This is a simplification; a real parser would need a context or a predefined set of symbols
            // For now, any non-alphanumeric, non-underscore, non-whitespace char is a symbol
            if ch.is_alphanumeric() || ch == '_' || "+-*/()[]{{}}<>;=!,&".contains(ch) {
                end += ch.len_utf8();
            } else {
                break;
            }
        }
        if end > start {
            let s = self.input[start..end].to_string();
            self.position = end;
            return Some(s);
        }

        // Fallback for unhandled characters
        let ch = self.input[self.position..].chars().next().unwrap();
        self.position += ch.len_utf8();
        Some(ch.to_string())
    }
}

// Performance comparison function
pub fn benchmark_parsers(input: &str, iterations: usize) {
    use std::time::Instant;

    let default_context = ParserContext::new(&[], &[]);

    // Zero-copy parser
    let start = Instant::now();
    for _ in 0..iterations {
        let parser = Parser::new(input, &default_context);
        let _tokens: Vec<Token> = parser.into_iter().collect();
    }
    let zero_copy_time = start.elapsed();

    // Allocating parser
    let start = Instant::now();
    for _ in 0..iterations {
        let mut parser = AllocatingParser::new(input.to_string());
        let mut tokens = Vec::new();
        while let Some(token) = parser.next_token() {
            tokens.push(token);
        }
    }
    let allocating_time = start.elapsed();

    println!("Zero-copy Parser Time: {:?}", zero_copy_time);
    println!("Allocating Parser Time: {:?}", allocating_time);
    if zero_copy_time.as_nanos() > 0 {
        println!("Speedup: {:.2}x", allocating_time.as_secs_f64() / zero_copy_time.as_secs_f64());
    }
}

// Example: CSV parser
pub fn parse_csv_line<'a>(line: &'a str) -> Vec<&'a str> {
    line.split(',').map(|s| s.trim()).collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of_val;

    // Default context for tests that don't need specific keywords/operators
    fn default_test_context<'a>() -> ParserContext<'a> {
        ParserContext::new(&[], &[])
    }

    #[test]
    fn test_parser_new() {
        let input = "hello world";
        let context = default_test_context();
        let parser = Parser::new(input, &context);
        assert_eq!(parser.remaining(), "hello world");
    }

    #[test]
    fn test_is_empty() {
        let context = default_test_context();
        let mut parser = Parser::new("  ", &context);
        assert!(parser.is_empty());
        let mut parser = Parser::new("hello", &context);
        assert!(!parser.is_empty());
    }

    #[test]
    fn test_token_identifier() {
        let context = default_test_context();
        let mut parser = Parser::new("foo bar_123", &context);

        assert_eq!(parser.next_token(), Token::Identifier("foo"));
        assert_eq!(parser.next_token(), Token::Identifier("bar_123"));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_token_number() {
        let context = default_test_context();
        let mut parser = Parser::new("42 3.14 0xFF", &context);

        assert_eq!(parser.next_token(), Token::Number("42"));
        assert_eq!(parser.next_token(), Token::Number("3.14"));
        assert_eq!(parser.next_token(), Token::Number("0xFF"));
    }

    #[test]
    fn test_token_string() {
        let context = default_test_context();
        let mut parser = Parser::new(r#""hello" "world""#, &context);

        assert_eq!(parser.next_token(), Token::String("hello"));
        assert_eq!(parser.next_token(), Token::String("world"));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_token_symbols() {
        let operators = &['+', '-', '*', '/', '(', ')'];
        let context = ParserContext::new(&[], operators);
        let mut parser = Parser::new("+ - * / ( )", &context);

        assert_eq!(parser.next_token(), Token::Symbol('+'));
        assert_eq!(parser.next_token(), Token::Symbol('-'));
        assert_eq!(parser.next_token(), Token::Symbol('*'));
        assert_eq!(parser.next_token(), Token::Symbol('/'));
        assert_eq!(parser.next_token(), Token::Symbol('('));
        assert_eq!(parser.next_token(), Token::Symbol(')'));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_mixed_tokens() {
        let operators = &['=', '+'];
        let context = ParserContext::new(&[], operators);
        let mut parser = Parser::new(r###"let x = 42 + "test""###, &context);

        assert_eq!(parser.next_token(), Token::Identifier("let"));
        assert_eq!(parser.next_token(), Token::Identifier("x"));
        assert_eq!(parser.next_token(), Token::Symbol('='));
        assert_eq!(parser.next_token(), Token::Number("42"));
        assert_eq!(parser.next_token(), Token::Symbol('+'));
        assert_eq!(parser.next_token(), Token::String("test"));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_is_keyword() {
        let token = Token::Identifier("if");
        assert!(token.is_keyword("if"));
        assert!(!token.is_keyword("else"));

        let token = Token::Number("42");
        assert!(!token.is_keyword("if"));
    }

    #[test]
    fn test_token_lifetime_tied_to_input() {
        let input = String::from("test");
        let context = default_test_context();
        let mut parser = Parser::new(&input, &context);
        let token = parser.next_token();

        // Token is valid as long as input is valid
        assert_eq!(token, Token::Identifier("test"));

        // This should NOT compile (uncomment to verify):
        // drop(input);
        // println!("{:?}", token); // ERROR: token can't outlive input
    }

    #[test]
    fn test_iterator() {
        let context = default_test_context();
        let parser = Parser::new("a b c", &context);
        let tokens: Vec<Token> = parser.into_iter().collect();

        assert_eq!(tokens.len(), 3);
        assert!(tokens.iter().all(|t| matches!(t, Token::Identifier(_))));
    }

    #[test]
    fn test_iterator_chaining() {
        let operators = &['='];
        let context = ParserContext::new(&[], operators);
        let parser = Parser::new("let x = 42", &context);
        let identifiers: Vec<&str> = parser
            .into_iter()
            .filter_map(|token| match token {
                Token::Identifier(s) => Some(s),
                _ => None,
            })
            .collect();

        assert_eq!(identifiers, vec!["let", "x"]);
    }

    #[test]
    fn test_identifiers_method() {
        let context = default_test_context();
        let mut parser = Parser::new("fn foo(x: i32) -> bool", &context);
        let ids = parser.identifiers();

        assert_eq!(ids, vec!["fn", "foo", "x", "i32", "bool"]);
    }

    #[test]
    fn test_has_keyword() {
        let keywords = &["if", "else", "while"];
        let context = ParserContext::new(keywords, &[]);
        let mut parser = Parser::new("if x > 0 { return true }", &context);

        assert!(parser.has_keyword("if"));

        // Parser state is not consumed by has_keyword due to internal temporary parser
        assert!(!parser.has_keyword("return")); // 'return' is not in our keywords
    }

    #[test]
    fn test_count_numbers() {
        let operators = &['+', '-', '*'];
        let context = ParserContext::new(&[], operators);
        let mut parser = Parser::new("1 + 2 * 3 - 4", &context);
        assert_eq!(parser.count_numbers(), 4);
    }

    #[test]
    fn test_iterator_zero_copy() {
        let input = String::from("foo bar baz");
        let context = default_test_context();
        let parser = Parser::new(&input, &context);

        // Collect tokens (no allocation for token content)
        let tokens: Vec<Token> = parser.into_iter().collect();

        // All identifiers should point into original input
        for token in tokens {
            if let Token::Identifier(s) = token {
                // Verify s is a slice of input (same pointer range)
                let input_ptr = input.as_ptr() as usize;
                let s_ptr = s.as_ptr() as usize;
                assert!(s_ptr >= input_ptr);
                assert!(s_ptr < input_ptr + input.len());
            }
        }
    }

    #[test]
    fn test_parser_with_context() {
        let keywords = &["if", "else", "fn", "let"];
        let operators = &['+', '-', '*', '/'];
        let context = ParserContext::new(keywords, operators);

        let input = "if x + 1";
        let mut parser = Parser::new(input, &context);

        assert_eq!(parser.next_token(), Token::Identifier("if"));
        assert_eq!(parser.next_token(), Token::Identifier("x"));
        assert_eq!(parser.next_token(), Token::Symbol('+'));
        assert_eq!(parser.next_token(), Token::Number("1"));
        assert_eq!(parser.next_token(), Token::Eof);
    }

    #[test]
    fn test_keyword_detection_in_context() {
        let keywords = &["if", "else", "while"];
        let context = ParserContext::new(keywords, &[]);

        assert!(context.is_keyword("if"));
        assert!(!context.is_keyword("foo"));
    }

    #[test]
    fn test_operator_detection_in_context() {
        let operators = &['+', '-'];
        let context = ParserContext::new(&[], operators);
        assert!(context.is_operator('+'));
        assert!(!context.is_operator('*'));
    }


    #[test]
    fn test_next_keyword_token() {
        let keywords = &["if", "fn"];
        let operators = &['+'];
        let context = ParserContext::new(keywords, operators);

        let mut parser = Parser::new("foo if bar fn baz", &context);

        assert_eq!(parser.next_keyword_token(), Some("if"));
        assert_eq!(parser.next_keyword_token(), Some("fn"));
        assert_eq!(parser.next_keyword_token(), None);
    }

    #[test]
    fn test_multiple_lifetime_bounds() {
        let keywords_vec = vec!["let".to_string(), "mut".to_string()];
        let keyword_refs: Vec<&str> = keywords_vec.iter().map(|s| s.as_str()).collect();

        let context = ParserContext::new(&keyword_refs, &[]);
        let input = String::from("let mut x = 42");

        let mut parser = Parser::new(&input, &context);

        // Both input and context must outlive parser
        let token = parser.next_token();
        assert_eq!(token, Token::Identifier("let"));

        // This demonstrates two independent lifetimes
        let kw = parser.get_keywords();
        assert_eq!(kw.len(), 2);
    }

    #[test]
    fn test_lifetime_independence() {
        // Context lives longer than input
        let keywords = &["fn"];
        let context = ParserContext::new(keywords, &[]);

        {
            let input = String::from("fn foo");
            let mut parser = Parser::new(&input, &context);

            assert_eq!(parser.next_keyword_token(), Some("fn"));
            // input dropped here
        }

        // context still valid here
        assert!(context.is_keyword("fn"));
    }

    #[test]
    fn test_extract_assignments() {
        let operators = &['='];
        let context = ParserContext::new(&[], operators);
        let mut parser = Parser::new("x = 42 y = 100", &context);

        let assignments = parser.extract_assignments();
        assert_eq!(assignments, vec![("x", "42"), ("y", "100")]);
    }

    #[test]
    fn test_group_identifiers() {
        let operators = &['+'];
        let context = ParserContext::new(&[], operators);
        let mut parser = Parser::new("foo bar + baz qux", &context);

        let groups = parser.group_identifiers();
        assert_eq!(groups, vec![vec!["foo", "bar"], vec!["baz", "qux"]]);
    }

    #[test]
    fn test_csv_zero_copy() {
        let line = "alice,bob,charlie";
        let fields = parse_csv_line(line);

        assert_eq!(fields, vec!["alice", "bob", "charlie"]);

        // Verify zero-copy: fields point into original line
        for field in fields {
            let line_ptr = line.as_ptr() as usize;
            let field_ptr = field.as_ptr() as usize;
            assert!(field_ptr >= line_ptr);
            assert!(field_ptr < line_ptr + line.len());
        }
    }

    #[test]
    fn test_benchmark_not_crash() {
        // Large input with many tokens
        let input = (0..100) // Reduced iterations for quicker test run
            .map(|i| format!("token{} ", i))
            .collect::<String>();

        // Run benchmark (just verify it doesn't crash)
        benchmark_parsers(&input, 10); // Reduced iterations for quicker test run
    }

    #[test]
    fn test_memory_efficiency() {
        let input = "test data here";
        let context = default_test_context();
        let parser = Parser::new(input, &context);
        let tokens: Vec<Token> = parser.into_iter().collect();

        // Each &str token is 2 * usize (pointer + length)
        // String would be 3 * usize (pointer + length + capacity) + heap allocation

        let token_size: usize = tokens.iter().map(|t| size_of_val(t)).sum();

        println!("Token vector size: {} bytes", token_size);
        println!("Number of tokens: {}", tokens.len());
        if tokens.len() > 0 {
            println!("Avg bytes per token: {}", token_size / tokens.len());
        }
        // This test mainly demonstrates the principle and prints memory info.
        // Exact sizes can vary by architecture, but the zero-copy nature should be evident.
    }
}
