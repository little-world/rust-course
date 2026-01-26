//! Pattern 4: Zero-Copy Slicing
//! Example: Complete Zero-Copy HTTP Parser
//!
//! Run with: cargo run --example p4_http_parser

#[derive(Debug)]
enum ParseError {
    InvalidUtf8,
    NoBodySeparator,
    Empty,
    NoMethod,
    NoPath,
}

#[derive(Debug)]
struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    version: &'a str,
    headers: Vec<(&'a str, &'a str)>,
    body: &'a [u8],
}

impl<'a> HttpRequest<'a> {
    fn parse(data: &'a [u8]) -> Result<Self, ParseError> {
        let data_str = std::str::from_utf8(data)
            .map_err(|_| ParseError::InvalidUtf8)?;

        let (head, body) = data_str.split_once("\r\n\r\n")
            .ok_or(ParseError::NoBodySeparator)?;

        let mut lines = head.lines();
        let request_line = lines.next().ok_or(ParseError::Empty)?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next().ok_or(ParseError::NoMethod)?;
        let path = parts.next().ok_or(ParseError::NoPath)?;
        let version = parts.next().unwrap_or("HTTP/1.0");

        let headers: Vec<_> = lines
            .filter_map(|line| line.split_once(": "))
            .collect();

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            body: body.as_bytes(),
        })
    }

    fn get_header(&self, name: &str) -> Option<&str> {
        self.headers.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| *v)
    }

    fn content_length(&self) -> Option<usize> {
        self.get_header("Content-Length")
            .and_then(|v| v.parse().ok())
    }
}

fn main() {
    println!("=== Zero-Copy HTTP Parser ===\n");

    // Parse a GET request
    let get_request = b"GET /api/users HTTP/1.1\r\n\
        Host: example.com\r\n\
        Accept: application/json\r\n\
        User-Agent: rust/1.0\r\n\
        \r\n";

    println!("=== GET Request ===\n");
    println!("Raw request ({} bytes):", get_request.len());

    match HttpRequest::parse(get_request) {
        Ok(req) => {
            println!("  Method:  {}", req.method);
            println!("  Path:    {}", req.path);
            println!("  Version: {}", req.version);
            println!("  Headers:");
            for (name, value) in &req.headers {
                println!("    {}: {}", name, value);
            }
            println!("  Body:    {} bytes", req.body.len());
        }
        Err(e) => println!("Parse error: {:?}", e),
    }

    // Parse a POST request with body
    let post_request = b"POST /api/users HTTP/1.1\r\n\
        Host: example.com\r\n\
        Content-Type: application/json\r\n\
        Content-Length: 27\r\n\
        \r\n\
        {\"name\":\"Alice\",\"age\":30}";

    println!("\n=== POST Request ===\n");

    match HttpRequest::parse(post_request) {
        Ok(req) => {
            println!("  Method:  {}", req.method);
            println!("  Path:    {}", req.path);
            println!("  Version: {}", req.version);
            println!("  Headers:");
            for (name, value) in &req.headers {
                println!("    {}: {}", name, value);
            }
            println!("  Content-Length: {:?}", req.content_length());
            println!("  Body:    '{}'", String::from_utf8_lossy(req.body));
        }
        Err(e) => println!("Parse error: {:?}", e),
    }

    // Demonstrate zero-copy nature
    println!("\n=== Zero-Copy Verification ===\n");

    let request_data = b"GET /test HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let request = HttpRequest::parse(request_data).unwrap();

    // Check that slices point into original data
    let data_ptr = request_data.as_ptr();
    let data_end = unsafe { data_ptr.add(request_data.len()) };

    let method_ptr = request.method.as_ptr();
    let path_ptr = request.path.as_ptr();

    let method_in_range = method_ptr >= data_ptr && method_ptr < data_end;
    let path_in_range = path_ptr >= data_ptr && path_ptr < data_end;

    println!("Original data range: {:?} - {:?}", data_ptr, data_end);
    println!("Method ptr: {:?} (in range: {})", method_ptr, method_in_range);
    println!("Path ptr:   {:?} (in range: {})", path_ptr, path_in_range);
    println!("\nAll parsed fields point into original buffer!");

    // Memory usage comparison
    println!("\n=== Memory Usage Comparison ===\n");

    // Zero-copy version
    let request_bytes = 1000; // Typical request size
    let headers_count = 10;
    let zero_copy_overhead = std::mem::size_of::<HttpRequest>()
        + headers_count * std::mem::size_of::<(&str, &str)>();

    // Allocating version (hypothetical)
    let allocating_overhead = request_bytes  // method, path, version strings
        + headers_count * 100;  // header name + value strings

    println!("For a {} byte request with {} headers:", request_bytes, headers_count);
    println!("  Zero-copy overhead:  ~{} bytes (just pointers)", zero_copy_overhead);
    println!("  Allocating overhead: ~{} bytes (copied strings)", allocating_overhead);
    println!("  Memory savings:      ~{}x", allocating_overhead / zero_copy_overhead);

    println!("\n=== Key Points ===");
    println!("1. All string fields are slices into original buffer");
    println!("2. Only the headers Vec allocates (not its contents)");
    println!("3. Lifetime 'a ties parsed request to input data");
    println!("4. 10-100x less memory than allocating approach");
    println!("5. Pattern works for any text/binary protocol");
}
