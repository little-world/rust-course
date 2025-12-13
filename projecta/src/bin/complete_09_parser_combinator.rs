use std::fmt;

// =============================================================================
// Milestone 1: Generic parser trait
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
struct ParseError {
    message: String,
    position: usize,
}

impl ParseError {
    fn new(message: String, position: usize) -> Self {
        Self { message, position }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at position {}", self.message, self.position)
    }
}

type ParseResult<'a, T> = Result<(T, &'a str), ParseError>;

trait ParserGeneric<Output> {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Output>;
}

struct CharParser {
    expected: char,
}

impl CharParser {
    fn parse_internal<'a>(&self, input: &'a str) -> ParseResult<'a, char> {
        let mut chars = input.char_indices();
        match chars.next() {
            Some((idx, ch)) if ch == self.expected => {
                let len = ch.len_utf8();
                Ok((ch, &input[idx + len..]))
            }
            Some((_, ch)) => Err(ParseError::new(
                format!("Expected '{}' but found '{}'", self.expected, ch),
                0,
            )),
            None => Err(ParseError::new(
                format!("Expected '{}' but reached end of input", self.expected),
                0,
            )),
        }
    }
}

impl ParserGeneric<char> for CharParser {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, char> {
        self.parse_internal(input)
    }
}

struct DigitParser;

impl DigitParser {
    fn parse_internal<'a>(&self, input: &'a str) -> ParseResult<'a, u32> {
        let mut chars = input.char_indices();
        match chars.next() {
            Some((idx, ch)) if ch.is_ascii_digit() => {
                let len = ch.len_utf8();
                let value = ch.to_digit(10).unwrap();
                Ok((value, &input[idx + len..]))
            }
            Some((_, ch)) => Err(ParseError::new(
                format!("Expected digit but found '{}'", ch),
                0,
            )),
            None => Err(ParseError::new("Expected digit but input was empty".to_string(), 0)),
        }
    }
}

impl ParserGeneric<u32> for DigitParser {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, u32> {
        self.parse_internal(input)
    }
}

fn run_parser_generic<Output, P: ParserGeneric<Output>>(
    parser: P,
    input: &str,
) -> Result<Output, ParseError> {
    parser.parse(input).map(|(value, _)| value)
}

// =============================================================================
// Milestone 2: Associated type parser trait
// =============================================================================

trait Parser {
    type Output;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output>;
}

impl Parser for CharParser {
    type Output = char;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, char> {
        self.parse_internal(input)
    }
}

impl Parser for DigitParser {
    type Output = u32;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, u32> {
        self.parse_internal(input)
    }
}

fn run_parser<P: Parser>(parser: P, input: &str) -> Result<P::Output, ParseError> {
    parser.parse(input).map(|(value, _)| value)
}

struct StringParser {
    expected: String,
}

impl Parser for StringParser {
    type Output = String;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, String> {
        if input.starts_with(&self.expected) {
            Ok((self.expected.clone(), &input[self.expected.len()..]))
        } else {
            Err(ParseError::new(
                format!("Expected \"{}\"", self.expected),
                0,
            ))
        }
    }
}

// =============================================================================
// Milestone 3: Parser combinators
// =============================================================================

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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, NewOutput> {
        let (value, remaining) = self.parser.parse(input)?;
        let mapped = (self.mapper)(value);
        Ok((mapped, remaining))
    }
}

trait ParserExt: Parser + Sized {
    fn map<F, NewOutput>(self, mapper: F) -> MapParser<Self, F>
    where
        F: Fn(Self::Output) -> NewOutput,
    {
        MapParser {
            parser: self,
            mapper,
        }
    }

    fn and_then<P2>(self, other: P2) -> AndThenParser<Self, P2>
    where
        P2: Parser,
    {
        AndThenParser {
            first: self,
            second: other,
        }
    }
}

impl<P: Parser> ParserExt for P {}

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

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, Self::Output> {
        let (first_value, remaining) = self.first.parse(input)?;
        let (second_value, final_remaining) = self.second.parse(remaining)?;
        Ok(((first_value, second_value), final_remaining))
    }
}

struct NumberParser;

impl Parser for NumberParser {
    type Output = u32;

    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, u32> {
        let mut end = 0;
        for (idx, ch) in input.char_indices() {
            if ch.is_ascii_digit() {
                end = idx + ch.len_utf8();
            } else {
                break;
            }
        }

        if end == 0 {
            return Err(ParseError::new(
                "Expected one or more digits".to_string(),
                0,
            ));
        }

        let digits = &input[..end];
        let remaining = &input[end..];
        match digits.parse::<u32>() {
            Ok(value) => Ok((value, remaining)),
            Err(_) => Err(ParseError::new(
                "Failed to parse integer".to_string(),
                0,
            )),
        }
    }
}

fn parse_addition(input: &str) -> Result<u32, ParseError> {
    let parser = NumberParser
        .and_then(CharParser { expected: '+' })
        .and_then(NumberParser)
        .map(|((left, _plus), right)| left + right);

    run_parser(parser, input)
}

fn main() {
    match parse_addition("12+30") {
        Ok(value) => println!("12+30 = {}", value),
        Err(err) => eprintln!("Failed to parse expression: {}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_parser() {
        let parser = CharParser { expected: 'a' };

        let result = ParserGeneric::parse(&parser, "abc");
        assert_eq!(result, Ok(('a', "bc")));

        let result = ParserGeneric::parse(&parser, "xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_digit_parser() {
        let parser = DigitParser;

        let result = ParserGeneric::parse(&parser, "5 apples");
        assert_eq!(result, Ok((5, " apples")));

        let result = ParserGeneric::parse(&parser, "abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_generic_verbose() {
        let parser = CharParser { expected: 'x' };

        let result: char = run_parser_generic::<char, _>(parser, "xyz").unwrap();
        assert_eq!(result, 'x');
    }

    #[test]
    fn test_associated_type_inference() {
        let parser = CharParser { expected: 'x' };

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

        let c = run_parser(char_parser, "abc").unwrap();
        let n = run_parser(digit_parser, "123").unwrap();

        assert_eq!(c, 'a');
        assert_eq!(n, 1);
    }

    #[test]
    fn test_map_combinator() {
        let parser = DigitParser.map(|d| format!("Digit: {}", d));

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

        assert!(parser.parse("abc").is_err());
    }

    #[test]
    fn test_parse_addition() {
        assert_eq!(parse_addition("5+3"), Ok(8));
        assert_eq!(parse_addition("100+200"), Ok(300));
        assert!(parse_addition("abc").is_err());
    }

    #[test]
    fn test_combinator_composition() {
        let parser = CharParser { expected: 'x' }
            .and_then(DigitParser)
            .map(|(c, d)| format!("{}{}", c, d));

        let result = run_parser(parser, "x5 items").unwrap();
        assert_eq!(result, "x5");
    }
}
