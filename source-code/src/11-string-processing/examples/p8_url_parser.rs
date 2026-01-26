//! Pattern 8: URL Parser State Machine
//! Parse URLs into Components
//!
//! Run with: cargo run --example p8_url_parser

fn main() {
    println!("=== URL Parser State Machine ===\n");

    let urls = [
        "https://example.com/path/to/page?key=value#section",
        "http://user:pass@host:8080/path",
        "file:///home/user/file.txt",
        "mailto:user@example.com",
    ];

    for url_str in &urls {
        let mut parser = UrlParser::new(url_str);
        match parser.parse() {
            Ok(url) => {
                println!("URL: {}", url_str);
                println!("  Scheme:    {}", url.scheme);
                println!("  Authority: {:?}", url.authority);
                println!("  Path:      {}", url.path);
                println!("  Query:     {:?}", url.query);
                println!("  Fragment:  {:?}", url.fragment);
                println!();
            }
            Err(e) => println!("Parse error for '{}': {}\n", url_str, e),
        }
    }

    // Error case
    println!("=== Error Case ===\n");
    let mut parser = UrlParser::new(":invalid");
    match parser.parse() {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("\n=== Key Points ===");
    println!("1. State transitions on delimiter characters");
    println!("2. Lookahead for // after scheme");
    println!("3. Optional components with Option<String>");
    println!("4. Single-pass O(N) parsing");
}

#[derive(Debug, PartialEq)]
struct Url {
    scheme: String,
    authority: Option<String>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

#[derive(Debug)]
enum ParseState {
    Scheme,
    AfterScheme,
    Authority,
    Path,
    Query,
    Fragment,
}

struct UrlParser {
    input: Vec<char>,
    pos: usize,
    state: ParseState,
}

impl UrlParser {
    fn new(url: &str) -> Self {
        UrlParser {
            input: url.chars().collect(),
            pos: 0,
            state: ParseState::Scheme,
        }
    }

    fn parse(&mut self) -> Result<Url, String> {
        let mut scheme = String::new();
        let mut authority = None;
        let mut path = String::new();
        let mut query = None;
        let mut fragment = None;

        while self.pos < self.input.len() {
            let ch = self.input[self.pos];

            match self.state {
                ParseState::Scheme => {
                    if ch == ':' {
                        if scheme.is_empty() {
                            return Err("Empty scheme".to_string());
                        }
                        self.state = ParseState::AfterScheme;
                        self.pos += 1;
                    } else if ch.is_alphanumeric()
                        || ch == '+' || ch == '-' || ch == '.' {
                        scheme.push(ch);
                        self.pos += 1;
                    } else {
                        return Err(format!("Invalid scheme character: {}", ch));
                    }
                }

                ParseState::AfterScheme => {
                    if self.pos + 1 < self.input.len()
                        && self.input[self.pos] == '/'
                        && self.input[self.pos + 1] == '/'
                    {
                        self.state = ParseState::Authority;
                        self.pos += 2;
                    } else {
                        self.state = ParseState::Path;
                    }
                }

                ParseState::Authority => {
                    if ch == '/' {
                        self.state = ParseState::Path;
                    } else if ch == '?' {
                        self.state = ParseState::Query;
                        self.pos += 1;
                    } else if ch == '#' {
                        self.state = ParseState::Fragment;
                        self.pos += 1;
                    } else {
                        if authority.is_none() {
                            authority = Some(String::new());
                        }
                        authority.as_mut().unwrap().push(ch);
                        self.pos += 1;
                    }
                }

                ParseState::Path => {
                    if ch == '?' {
                        self.state = ParseState::Query;
                        self.pos += 1;
                    } else if ch == '#' {
                        self.state = ParseState::Fragment;
                        self.pos += 1;
                    } else {
                        path.push(ch);
                        self.pos += 1;
                    }
                }

                ParseState::Query => {
                    if ch == '#' {
                        self.state = ParseState::Fragment;
                        self.pos += 1;
                    } else {
                        if query.is_none() {
                            query = Some(String::new());
                        }
                        query.as_mut().unwrap().push(ch);
                        self.pos += 1;
                    }
                }

                ParseState::Fragment => {
                    if fragment.is_none() {
                        fragment = Some(String::new());
                    }
                    fragment.as_mut().unwrap().push(ch);
                    self.pos += 1;
                }
            }
        }

        Ok(Url {
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }
}
