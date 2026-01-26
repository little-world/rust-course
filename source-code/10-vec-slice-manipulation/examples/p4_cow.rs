//! Pattern 4: Zero-Copy Slicing
//! Example: Copy-on-Write with Cow
//!
//! Run with: cargo run --example p4_cow

use std::borrow::Cow;

fn main() {
    println!("=== Copy-on-Write with Cow ===\n");

    // Basic Cow usage
    println!("=== Basic Cow Usage ===\n");

    fn decode_field(field: &[u8]) -> Cow<str> {
        match std::str::from_utf8(field) {
            Ok(s) => Cow::Borrowed(s),
            Err(_) => {
                // Only allocate when we need to fix encoding
                Cow::Owned(String::from_utf8_lossy(field).into_owned())
            }
        }
    }

    let valid_utf8 = b"hello world";
    let invalid_utf8 = b"hello \xFF world";

    let result1 = decode_field(valid_utf8);
    let result2 = decode_field(invalid_utf8);

    println!("Valid UTF-8:   '{}'", result1);
    println!("  Is borrowed: {}", matches!(result1, Cow::Borrowed(_)));

    println!("Invalid UTF-8: '{}'", result2);
    println!("  Is borrowed: {}", matches!(result2, Cow::Borrowed(_)));

    // Conditional modification
    println!("\n=== Conditional Modification ===\n");

    fn normalize_path(path: &str) -> Cow<str> {
        if path.contains("//") || path.ends_with('/') {
            // Need to modify - allocate
            let mut result = path.replace("//", "/");
            if result.ends_with('/') && result.len() > 1 {
                result.pop();
            }
            Cow::Owned(result)
        } else {
            // No modification needed - borrow
            Cow::Borrowed(path)
        }
    }

    let paths = [
        "/home/user/documents",
        "/home//user//documents",
        "/home/user/",
        "//double//slashes//",
    ];

    for path in paths {
        let normalized = normalize_path(path);
        let status = if matches!(normalized, Cow::Borrowed(_)) {
            "borrowed"
        } else {
            "owned"
        };
        println!("  '{}' -> '{}' ({})", path, normalized, status);
    }

    // Cow in structs
    println!("\n=== Cow in Structs ===\n");

    #[derive(Debug)]
    struct Config<'a> {
        name: Cow<'a, str>,
        value: Cow<'a, str>,
    }

    impl<'a> Config<'a> {
        fn from_str(s: &'a str) -> Option<Self> {
            let (name, value) = s.split_once('=')?;
            Some(Config {
                name: Cow::Borrowed(name.trim()),
                value: Cow::Borrowed(value.trim()),
            })
        }

        fn with_default(name: &'a str, default: &'a str) -> Self {
            Config {
                name: Cow::Borrowed(name),
                value: Cow::Borrowed(default),
            }
        }

        fn override_value(&mut self, new_value: String) {
            self.value = Cow::Owned(new_value);
        }
    }

    let line = "database_url = postgres://localhost/db";
    let mut config = Config::from_str(line).unwrap();
    println!("Parsed config: {:?}", config);
    println!("  Name borrowed: {}", matches!(config.name, Cow::Borrowed(_)));

    config.override_value("postgres://production/db".to_string());
    println!("After override: {:?}", config);
    println!("  Value borrowed: {}", matches!(config.value, Cow::Borrowed(_)));

    // Cow for efficient APIs
    println!("\n=== Cow for Efficient APIs ===\n");

    fn escape_html(input: &str) -> Cow<str> {
        if input.contains(['<', '>', '&', '"', '\'']) {
            let mut result = String::with_capacity(input.len() + 10);
            for c in input.chars() {
                match c {
                    '<' => result.push_str("&lt;"),
                    '>' => result.push_str("&gt;"),
                    '&' => result.push_str("&amp;"),
                    '"' => result.push_str("&quot;"),
                    '\'' => result.push_str("&#39;"),
                    _ => result.push(c),
                }
            }
            Cow::Owned(result)
        } else {
            Cow::Borrowed(input)
        }
    }

    let texts = [
        "Hello, World!",
        "5 > 3 && 2 < 4",
        "Say \"Hello\"",
        "Plain text without special chars",
    ];

    for text in texts {
        let escaped = escape_html(text);
        let status = if matches!(escaped, Cow::Borrowed(_)) { "borrowed" } else { "owned" };
        println!("  '{}' -> '{}' ({})", text, escaped, status);
    }

    // to_mut for lazy cloning
    println!("\n=== Lazy Cloning with to_mut ===\n");

    fn ensure_trailing_slash(path: &str) -> Cow<str> {
        let mut cow = Cow::Borrowed(path);
        if !path.ends_with('/') {
            cow.to_mut().push('/');
        }
        cow
    }

    let paths = ["/home/user", "/home/user/"];
    for path in paths {
        let result = ensure_trailing_slash(path);
        let status = if matches!(result, Cow::Borrowed(_)) { "borrowed" } else { "owned" };
        println!("  '{}' -> '{}' ({})", path, result, status);
    }

    println!("\n=== Key Points ===");
    println!("1. Cow<'a, T> is either Borrowed(&'a T) or Owned(T::Owned)");
    println!("2. Only allocates when modification is needed");
    println!("3. Use for APIs that usually return input unchanged");
    println!("4. to_mut() converts to owned only when needed");
    println!("5. Great for escaping, normalization, optional transforms");
}
