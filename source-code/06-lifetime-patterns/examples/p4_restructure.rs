//! Pattern 4: Self-Referential Structs and Pin
//! Example: Restructuring Design to Avoid Self-Reference
//!
//! Run with: cargo run --example p4_restructure

// PROBLEM: This self-referential design doesn't work in Rust
// struct BadDesign<'a> {
//     data: String,
//     view: &'a str, // Can't reference our own data field!
// }

// SOLUTION: Separate owner from borrower into two types

// The owner holds the data
#[derive(Debug)]
struct Data {
    content: String,
}

// The view borrows from the owner
struct View<'a> {
    data: &'a Data,
    window: &'a str,
}

impl Data {
    fn new(content: impl Into<String>) -> Self {
        Data {
            content: content.into(),
        }
    }

    fn view(&self) -> View {
        View {
            data: self,
            window: &self.content[..],
        }
    }

    fn view_range(&self, start: usize, end: usize) -> Option<View> {
        if end <= self.content.len() && start <= end {
            Some(View {
                data: self,
                window: &self.content[start..end],
            })
        } else {
            None
        }
    }
}

impl<'a> View<'a> {
    fn window(&self) -> &str {
        self.window
    }

    fn full_content(&self) -> &str {
        &self.data.content
    }
}

// Another example: Parser that doesn't own its input

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

struct ParseResult<'a> {
    token: &'a str,
    remaining: &'a str,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.position..]
    }

    fn next_word(&mut self) -> Option<ParseResult<'a>> {
        let remaining = self.remaining().trim_start();
        if remaining.is_empty() {
            return None;
        }

        let end = remaining
            .find(char::is_whitespace)
            .unwrap_or(remaining.len());

        let token = &remaining[..end];
        let new_remaining = &remaining[end..];

        // Update position to match what we've consumed
        self.position = self.input.len() - new_remaining.len();

        Some(ParseResult {
            token,
            remaining: new_remaining,
        })
    }
}

// Buffer and Cursor pattern: separate data from iteration state

struct Buffer {
    data: Vec<u8>,
}

struct Cursor<'a> {
    buffer: &'a Buffer,
    position: usize,
}

impl Buffer {
    fn new(data: Vec<u8>) -> Self {
        Buffer { data }
    }

    fn cursor(&self) -> Cursor {
        Cursor {
            buffer: self,
            position: 0,
        }
    }
}

impl<'a> Cursor<'a> {
    fn read(&mut self, n: usize) -> &'a [u8] {
        let start = self.position;
        let end = (start + n).min(self.buffer.data.len());
        self.position = end;
        &self.buffer.data[start..end]
    }

    fn remaining(&self) -> &'a [u8] {
        &self.buffer.data[self.position..]
    }

    fn position(&self) -> usize {
        self.position
    }
}

fn main() {
    println!("=== Restructured Design: Data + View ===");
    // Usage: Separate owner (Data) from borrower (View) to avoid self-reference.
    let data = Data::new("hello world from rust");
    let view = data.view();

    println!("Full content: {}", view.full_content());
    println!("Window: {}", view.window());

    // Create multiple views of the same data
    if let Some(partial) = data.view_range(0, 5) {
        println!("Partial view (0..5): {}", partial.window());
    }
    if let Some(partial) = data.view_range(6, 11) {
        println!("Partial view (6..11): {}", partial.window());
    }

    println!("\n=== Parser Pattern ===");
    let input = "hello world from rust programming";
    let mut parser = Parser::new(input);

    println!("Parsing: '{}'", input);
    while let Some(result) = parser.next_word() {
        println!("  Token: '{}', Remaining: '{}'", result.token, result.remaining.trim());
    }

    println!("\n=== Buffer + Cursor Pattern ===");
    let buffer = Buffer::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    let mut cursor = buffer.cursor();

    println!("Reading from buffer:");
    println!("  Read 3 bytes: {:?}", cursor.read(3));
    println!("  Position: {}", cursor.position());
    println!("  Read 4 bytes: {:?}", cursor.read(4));
    println!("  Remaining: {:?}", cursor.remaining());

    println!("\n=== Why This Works ===");
    println!("1. Data owns the content (no references)");
    println!("2. View/Cursor borrows from Data");
    println!("3. Lifetimes are explicit and checkable");
    println!("4. No self-reference needed!");

    println!("\n=== Design Principles ===");
    println!("- Separate ownership from borrowing");
    println!("- Owner is a simple, movable struct");
    println!("- Views/Cursors borrow with explicit lifetimes");
    println!("- Multiple views can coexist (shared borrows)");

    println!("\n=== When to Restructure ===");
    println!("Most 'self-referential' needs can be restructured:");
    println!("  - String + view -> String owner + &str view");
    println!("  - Buffer + position -> Buffer owner + Cursor view");
    println!("  - Data + computed slice -> separate types");
    println!("\nOnly use Pin/unsafe for truly necessary cases like:");
    println!("  - Async futures (handled by the language)");
    println!("  - Intrusive data structures");
    println!("  - FFI requirements");
}
