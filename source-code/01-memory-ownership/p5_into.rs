// Pattern 5: Into for Ownership Transfer

// Accept anything convertible to String
fn greet(name: impl Into<String>) {
    let name: String = name.into();
    println!("Hello, {}!", name);
}

fn use_into() {
    greet("Alice");              // &str -> String (allocates)
    greet(String::from("Bob"));  // String -> String (no-op)
    greet('X');                  // char -> String
}

// Builder pattern with Into
struct Request {
    url: String,
    method: String,
}

impl Request {
    fn new(url: impl Into<String>) -> Self {
        Request {
            url: url.into(),
            method: "GET".into(),
        }
    }

    fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }
}

fn builder_example() {
    let req = Request::new("https://example.com")
        .method("POST");
    println!("Request to {} with method {}", req.url, req.method);
}

fn main() {
    use_into();
    builder_example();
    println!("Into example completed");
}
