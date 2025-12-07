
## Project 9: Parser Combinator Library with Associated Types

### Problem Statement

Build a parser combinator library using associated types for ergonomic composition. You'll start with a generic parser trait, refactor to use associated types for better API design, then build a complete expression parser using combinators.

**Generic vs Associated Type APIs**:
```rust
// With generic type parameter - verbose!
trait Parser<Output> {
    fn parse(&self, input: &str) -> Result<Output, ParseError>;
}

fn use_parser<O, P: Parser<O>>(parser: P, input: &str) -> O {
    parser.parse(input).unwrap()
}

// Caller must specify Output type:
let result: i32 = use_parser::<i32, _>(number_parser, "42");
//                             ^^^  Annoying turbofish!
```

```rust
// With associated type - ergonomic!
trait Parser {
    type Output;
    fn parse(&self, input: &str) -> Result<Self::Output, ParseError>;
}

fn use_parser<P: Parser>(parser: P, input: &str) -> P::Output {
    parser.parse(input).unwrap()
}

// Compiler infers Output:
let result = use_parser(number_parser, "42");  // Output inferred!
```

**Performance Benefits**:
- **Zero overhead**: Both compile to same code
- **Composition**: Combinators build complex parsers from simple ones
- **Type inference**: Associated types reduce annotation burden
- **Compile-time parsing**: Grammar encoded in types


**Associated Types are Critical When**:
- Output type determined by parser implementation
- One implementation per type makes sense
- API ergonomics important (avoid turbofish)
- Composing many parsers together



### Milestone 1: Basic Parser Trait with Generics

**Goal**: Define a parser trait using generic type parameters.


**Starter Code**:
```rust
#[derive(Debug, Clone, PartialEq)]
struct ParseError {
    message: String,
    position: usize,
}

impl ParseError {
    fn new(message: String, position: usize) -> Self {
        // TODO: Create ParseError
        todo!()
    }
}

// Generic parser trait - Output is a type parameter
trait ParserGeneric<Output> {
    fn parse(&self, input: &str) -> Result<(Output, &str), ParseError>;
}

// Parser that matches a specific character
struct CharParser {
    expected: char,
}

impl ParserGeneric<char> for CharParser {
    fn parse(&self, input: &str) -> Result<(char, &str), ParseError> {
        // TODO: Check if input starts with expected char
        todo!()
    }
}

// Parser that matches any digit and returns as u32
struct DigitParser;

impl ParserGeneric<u32> for DigitParser {
    fn parse(&self, input: &str) -> Result<(u32, &str), ParseError> {
        // TODO: Check if first char is digit
        // Parse digit and return with remaining input
        todo!()
    }
}

// Helper function (note the verbose type parameters!)
fn run_parser_generic<Output, P: ParserGeneric<Output>>(
    parser: P,
    input: &str,
) -> Result<Output, ParseError> {
    // TODO: Call parser.parse and return just the Output (discard remaining input)
    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_char_parser() {
    let parser = CharParser { expected: 'a' };

    let result = parser.parse("abc");
    assert_eq!(result, Ok(('a', "bc")));

    let result = parser.parse("xyz");
    assert!(result.is_err());
}

#[test]
fn test_digit_parser() {
    let parser = DigitParser;

    let result = parser.parse("5 apples");
    assert_eq!(result, Ok((5, " apples")));

    let result = parser.parse("abc");
    assert!(result.is_err());
}

#[test]
fn test_generic_verbose() {
    let parser = CharParser { expected: 'x' };

    // Must specify types explicitly - annoying!
    let result: char = run_parser_generic::<char, _>(parser, "xyz").unwrap();
    assert_eq!(result, 'x');
}
```


**Check Your Understanding**:
- Why do we need to specify `<char, _>` when calling `run_parser_generic`?
- Can `CharParser` implement `ParserGeneric<String>` too? What would that mean?
- What's the downside of having multiple possible implementations?

---

### 🔄 Why Milestone 1 Isn't Enough

**Limitations with Generics**:
1. **Verbose call sites**: Must specify types with turbofish `::<>`
2. **Ambiguity**: `CharParser` could implement `ParserGeneric<char>` and `ParserGeneric<String>`
3. **Type inference fails**: Compiler can't always deduce Output from usage
4. **Documentation confusion**: Which Output type should I use?

**What we're adding**: **Associated Types** - Output type determined by parser:
- `type Output` in trait definition
- One implementation per type (no ambiguity)
- Compiler infers Output from parser type
- Cleaner API with no turbofish needed

**Improvements**:
- **Ergonomics**: `parser.parse(input)` - compiler infers output type
- **Clarity**: Each parser has exactly one output type
- **Type inference**: Better inference with associated types
- **Documentation**: "This parser produces X" vs "This parser produces T"

**Trade-offs**:
- **Flexibility**: Can't have multiple Output types for same parser
- **Usually correct**: Most parsers produce one logical output type

---

### Milestone 2: Refactor to Associated Types

**Goal**: Change the trait to use associated types for better ergonomics.


**Starter Code**:
```rust
// Parser trait with associated type
trait Parser {
    type Output;

    fn parse(&self, input: &str) -> Result<(Self::Output, &str), ParseError>;
}

// CharParser now has one clear Output type
impl Parser for CharParser {
    type Output = char;

    fn parse(&self, input: &str) -> Result<(char, &str), ParseError> {
        // TODO: Same implementation as before
        todo!()
    }
}

impl Parser for DigitParser {
    type Output = u32;

    fn parse(&self, input: &str) -> Result<(u32, &str), ParseError> {
        // TODO: Same implementation as before
        todo!()
    }
}

// Much cleaner helper function!
fn run_parser<P: Parser>(parser: P, input: &str) -> Result<P::Output, ParseError> {
    // TODO: Parse and return Output
    // Note: P::Output is the associated type
    todo!()
}

// String parser - matches multiple characters
struct StringParser {
    expected: String,
}

impl Parser for StringParser {
    type Output = String;

    fn parse(&self, input: &str) -> Result<(String, &str), ParseError> {
        // TODO: Check if input starts with expected string
        // Return matched string and remaining input
        todo!()
    }
}
```
**Checkpoint Tests**:
```rust
#[test]
fn test_associated_type_inference() {
    let parser = CharParser { expected: 'x' };

    // No turbofish needed! Compiler infers Output = char
    let result = run_parser(parser, "xyz").unwrap();
    assert_eq!(result, 'x');
}

#[test]
fn test_string_parser() {
    let parser = StringParser {
        expected: "hello".to_string(),
    };

    let result = parser.parse("hello world");
    assert_eq!(result, Ok(("hello".to_string(), " world")));

    let result = parser.parse("goodbye");
    assert!(result.is_err());
}

#[test]
fn test_output_type_inference() {
    let char_parser = CharParser { expected: 'a' };
    let digit_parser = DigitParser;

    // Types inferred from parser!
    let c = run_parser(char_parser, "abc").unwrap();
    let n = run_parser(digit_parser, "123").unwrap();

    assert_eq!(c, 'a');
    assert_eq!(n, 1);
}
```



**Check Your Understanding**:
- Why can't you call `run_parser` without specifying types in Milestone 1?
- Why does it work in Milestone 2?
- Can you implement `Parser` twice for `CharParser` with different `Output`? Why not?
- When would you want multiple implementations?

---

### Why Milestone 2 Isn't Enough → Moving to Milestone 3

**Missing Functionality**:
1. **No composition**: Can't combine parsers (e.g., parse char then digit)
2. **No transformation**: Can't map parser output (e.g., digit to string)
3. **No alternatives**: Can't try multiple parsers (e.g., digit or letter)
4. **Boilerplate**: Creating new parsers for combinations is tedious

**What we're adding**: **Parser Combinators** - functions that combine parsers:
- `and_then`: Parse A then B, return (A, B)
- `map`: Parse A, transform output with function
- `or_else`: Try A, if fails try B
- `many`: Parse repeatedly until failure

**Improvements**:
- **Composability**: Build complex parsers from simple ones
- **Reusability**: Combinators work with any parser
- **Type-safe**: Compiler checks combinator composition
- **Declarative**: Grammar reads like BNF notation

---

### Milestone 3: Parser Combinators and Composition

**Goal**: Implement combinator functions that compose parsers.


**Starter Code**:
```rust
// Combinator: Map parser output using function
struct MapParser<P, F> {
    parser: P,
    mapper: F,
}

impl<P, F, NewOutput> Parser for MapParser<P, F>
where
    P: Parser,
    F: Fn(P::Output) -> NewOutput,
{
    type Output = NewOutput;

    fn parse(&self, input: &str) -> Result<(NewOutput, &str), ParseError> {
        // TODO: Parse using self.parser
        // TODO: Apply self.mapper to output
        // TODO: Return mapped output with remaining input
        todo!()
    }
}

// Extension trait for ergonomic combinators
trait ParserExt: Parser + Sized {
    fn map<F, NewOutput>(self, mapper: F) -> MapParser<Self, F>
    where
        F: Fn(Self::Output) -> NewOutput,
    {
        // TODO: Create MapParser wrapping self and mapper
        todo!()
    }

    fn and_then<P2>(self, other: P2) -> AndThenParser<Self, P2>
    where
        P2: Parser,
    {
        // TODO: Create AndThenParser (define below)
        todo!()
    }
}

// Implement for all Parsers
impl<P: Parser> ParserExt for P {}

// Combinator: Parse A then B
struct AndThenParser<P1, P2> {
    first: P1,
    second: P2,
}

impl<P1, P2> Parser for AndThenParser<P1, P2>
where
    P1: Parser,
    P2: Parser,
{
    type Output = (P1::Output, P2::Output);

    fn parse(&self, input: &str) -> Result<(Self::Output, &str), ParseError> {
        // TODO: Parse with first parser
        // TODO: Parse remaining input with second parser
        // TODO: Return tuple of both outputs with final remaining input
        todo!()
    }
}

// Number parser: parses multiple digits
struct NumberParser;

impl Parser for NumberParser {
    type Output = u32;

    fn parse(&self, input: &str) -> Result<(u32, &str), ParseError> {
        // TODO: Parse as many digits as possible
        // Parse the string slice as u32
        todo!()
    }
}

// Helper: Parse arithmetic expression "5+3"
fn parse_addition(input: &str) -> Result<u32, ParseError> {
    // TODO: Use NumberParser, CharParser('+'), NumberParser
    // Combine with and_then, map to compute sum
    todo!()
}
```

**Checkpoint Tests**:
```rust
#[test]
fn test_map_combinator() {
    let parser = DigitParser
        .map(|d| format!("Digit: {}", d));

    let result = run_parser(parser, "5 apples").unwrap();
    assert_eq!(result, "Digit: 5");
}

#[test]
fn test_and_then_combinator() {
    let parser = CharParser { expected: 'a' }
        .and_then(CharParser { expected: 'b' });

    let result = parser.parse("abc");
    assert_eq!(result, Ok((('a', 'b'), "c")));

    let result = parser.parse("axc");
    assert!(result.is_err());
}

#[test]
fn test_number_parser() {
    let parser = NumberParser;

    let result = parser.parse("42 answer");
    assert_eq!(result, Ok((42, " answer")));

    let result = parser.parse("0");
    assert_eq!(result, Ok((0, "")));
}

#[test]
fn test_parse_addition() {
    assert_eq!(parse_addition("5+3"), Ok(8));
    assert_eq!(parse_addition("100+200"), Ok(300));
    assert!(parse_addition("abc").is_err());
}

#[test]
fn test_combinator_composition() {
    // Parse "x5" -> (char, u32)
    let parser = CharParser { expected: 'x' }
        .and_then(DigitParser)
        .map(|(c, d)| format!("{}{}", c, d));

    let result = run_parser(parser, "x5 items").unwrap();
    assert_eq!(result, "x5");
}
```

**Check Your Understanding**:
- Why does `map` return `MapParser<Self, F>` instead of changing Self?
- How does `ParserExt` add methods to all Parser types?
- What's the type of `parser.and_then(parser2).map(f)`?
- Could you implement `or_else` combinator? How would Output type work?

---

### Complete Project Summary

**What You Built**:
1. Parser trait with generic type parameters
2. Refactored to associated types for better API
3. Parser combinators for composition and transformation
4. Complete expression parser using combinators

**Key Concepts Practiced**:
- Associated types vs generic type parameters
- Type-driven API design
- Parser combinator patterns
- Extension traits for adding methods
- Higher-order functions with parsers

**API Comparison**:

| Aspect | Generic `<Output>` | Associated `type Output` |
|--------|-------------------|-------------------------|
| **Call site** | `use::<Type, _>(p)` | `use(p)` - inferred |
| **Multiple impls** | Possible | One per type |
| **Type inference** | Often fails | Usually works |
| **Flexibility** | High | Lower |
| **Ergonomics** | Poor | Excellent |
| **Use case** | Input to trait | Output from trait |

**Real-World Applications**:
- nom parser combinator library
- Compiler frontends (Rust, Swift)
- Protocol parsers (HTTP, DNS)
- Data extraction tools

