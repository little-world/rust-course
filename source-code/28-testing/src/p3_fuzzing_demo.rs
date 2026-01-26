// Pattern 3: Coverage-Guided Fuzzing (Demonstration)
// NOTE: Real fuzzing requires cargo-fuzz and libfuzzer. This file demonstrates
// the patterns that would be used in fuzz targets, with regular tests to verify.

// ============================================================================
// Example: Parser for fuzzing demonstration
// ============================================================================

/// A simple expression parser that would be a good fuzz target
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Parse expression from bytes - a typical fuzz target
    pub fn parse_from_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.is_empty() {
            return Err("empty input");
        }

        let s = std::str::from_utf8(data).map_err(|_| "invalid utf8")?;
        Self::parse_str(s.trim())
    }

    fn parse_str(s: &str) -> Result<Self, &'static str> {
        if s.is_empty() {
            return Err("empty expression");
        }

        // Simple parser: look for + or - operators
        if let Some(pos) = s.rfind('+') {
            let left = Self::parse_str(s[..pos].trim())?;
            let right = Self::parse_str(s[pos + 1..].trim())?;
            return Ok(Expr::Add(Box::new(left), Box::new(right)));
        }

        if let Some(pos) = s.rfind('-') {
            if pos > 0 {
                let left = Self::parse_str(s[..pos].trim())?;
                let right = Self::parse_str(s[pos + 1..].trim())?;
                return Ok(Expr::Sub(Box::new(left), Box::new(right)));
            }
        }

        // Try parsing as number
        s.parse::<i64>()
            .map(Expr::Number)
            .map_err(|_| "invalid number")
    }

    /// Serialize back to bytes - for round-trip testing
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    fn to_string(&self) -> String {
        match self {
            Expr::Number(n) => n.to_string(),
            Expr::Add(l, r) => format!("{} + {}", l.to_string(), r.to_string()),
            Expr::Sub(l, r) => format!("{} - {}", l.to_string(), r.to_string()),
        }
    }

    /// Evaluate the expression
    pub fn eval(&self) -> i64 {
        match self {
            Expr::Number(n) => *n,
            Expr::Add(l, r) => l.eval().saturating_add(r.eval()),
            Expr::Sub(l, r) => l.eval().saturating_sub(r.eval()),
        }
    }
}

// ============================================================================
// Example: Structured input for fuzzing (simulating arbitrary crate)
// ============================================================================

/// A request structure that would use #[derive(Arbitrary)] for fuzzing
#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub body: Vec<u8>,
}

impl Request {
    /// Validate the request - a typical fuzz target
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.method.is_empty() {
            return Err("empty method");
        }
        if !["GET", "POST", "PUT", "DELETE"].contains(&self.method.as_str()) {
            return Err("invalid method");
        }
        if self.path.is_empty() || !self.path.starts_with('/') {
            return Err("invalid path");
        }
        Ok(())
    }

    /// Process the request
    pub fn process(&self) -> Result<String, &'static str> {
        self.validate()?;
        Ok(format!("{} {} ({} bytes)", self.method, self.path, self.body.len()))
    }
}

// ============================================================================
// Tests simulating what a fuzzer would do
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Simulating fuzz_target!(|data: &[u8]| { ... })
    fn fuzz_expr_parser(data: &[u8]) {
        if let Ok(expr) = Expr::parse_from_bytes(data) {
            // Round trip: serialize then parse again
            let encoded = expr.to_bytes();
            if let Ok(decoded) = Expr::parse_from_bytes(&encoded) {
                // Verify semantic equivalence
                assert_eq!(expr.eval(), decoded.eval());
            }
        }
    }

    #[test]
    fn test_fuzz_valid_inputs() {
        // Test cases a fuzzer might generate
        let test_cases: &[&[u8]] = &[
            b"42",
            b"1 + 2",
            b"10 - 5",
            b"1 + 2 + 3",
            b"100 - 50 + 25",
            b"-42",
            b"0",
        ];

        for data in test_cases {
            fuzz_expr_parser(data);
        }
    }

    #[test]
    fn test_fuzz_invalid_inputs() {
        // Invalid inputs shouldn't panic
        let test_cases: &[&[u8]] = &[
            b"",
            b"   ",
            b"abc",
            b"1 + ",
            b"+ 1",
            &[0xFF, 0xFE], // Invalid UTF-8
        ];

        for data in test_cases {
            fuzz_expr_parser(data);
        }
    }

    #[test]
    fn test_request_validation() {
        let valid = Request {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            body: vec![],
        };
        assert!(valid.validate().is_ok());

        let invalid_method = Request {
            method: "INVALID".to_string(),
            path: "/api".to_string(),
            body: vec![],
        };
        assert!(invalid_method.validate().is_err());

        let invalid_path = Request {
            method: "GET".to_string(),
            path: "no-slash".to_string(),
            body: vec![],
        };
        assert!(invalid_path.validate().is_err());
    }
}

fn main() {
    println!("Pattern 3: Fuzzing Demonstration");
    println!("================================");
    println!();
    println!("This demonstrates patterns used in fuzz targets.");
    println!("Real fuzzing requires: cargo install cargo-fuzz");
    println!();

    // Demo expression parsing
    let inputs = ["42", "1 + 2", "10 - 5 + 3"];
    for input in inputs {
        match Expr::parse_from_bytes(input.as_bytes()) {
            Ok(expr) => println!("'{}' -> {:?} = {}", input, expr, expr.eval()),
            Err(e) => println!("'{}' -> Error: {}", input, e),
        }
    }

    println!();
    println!("Run tests with: cargo test --bin p3_fuzzing_demo");
}
