//! Pattern 3: Zero-Copy String Operations
//! Line Parser and CSV Parser with Zero Allocations
//!
//! Run with: cargo run --example p3_zero_copy_parsing

fn main() {
    println!("=== Zero-Copy String Operations ===\n");

    // Zero-copy line parsing
    println!("=== Zero-Copy Line Parser ===\n");

    let data = "line one\nline two\nline three";
    let parser = LineParser::new(data);

    println!("Lines:");
    for line in parser.lines() {
        println!("  '{}'", line);
    }

    // Zero-copy CSV parsing
    println!("\n=== Zero-Copy CSV Parser ===\n");

    let csv_data = "name,age,city\nAlice,30,NYC\nBob,25,LA";
    let csv_parser = CsvParser::new(csv_data);

    let rows = csv_parser.parse();
    println!("Parsed CSV:");
    for row in &rows {
        println!("  {:?}", row);
    }

    // Process without intermediate allocations
    println!("\nProcessing CSV fields directly:");
    csv_parser.process(|fields| {
        if fields.len() >= 2 {
            println!("  Name: {}, Age: {}", fields[0], fields[1]);
        }
    });

    // String view with UTF-8 boundary checking
    println!("\n=== String View with UTF-8 Safety ===\n");

    let text = "Hello, World!";
    if let Some(view) = StringView::new(text, 0, 5) {
        println!("View (0, 5): '{}'", view.as_str());
    }

    if let Some(view) = StringView::new(text, 7, 5) {
        println!("View (7, 5): '{}'", view.as_str());
    }

    // UTF-8 boundary checking
    let utf8_text = "Héllo";
    println!("\nUTF-8 boundary check on '{}':", utf8_text);
    println!("  Boundary at 0: {}", utf8_text.is_char_boundary(0));
    println!("  Boundary at 1: {}", utf8_text.is_char_boundary(1));
    println!("  Boundary at 2: {}", utf8_text.is_char_boundary(2)); // Middle of 'é'
    println!("  Boundary at 3: {}", utf8_text.is_char_boundary(3));

    println!("\n=== Key Points ===");
    println!("1. Return slices (&str) instead of owned strings");
    println!("2. Use split(), lines() for zero-copy iteration");
    println!("3. Always check is_char_boundary() before slicing");
    println!("4. Zero-copy = zero allocations = 10-100x faster");
}

struct LineParser<'a> {
    data: &'a str,
}

impl<'a> LineParser<'a> {
    fn new(data: &'a str) -> Self {
        LineParser { data }
    }

    // Returns iterator over lines without allocation
    fn lines(&self) -> impl Iterator<Item = &'a str> {
        self.data.lines()
    }

    // Split by delimiter without allocation
    fn split(&self, delimiter: &'a str) -> impl Iterator<Item = &'a str> {
        self.data.split(delimiter)
    }

    // Extract field by index
    fn field(&self, line: &'a str, index: usize) -> Option<&'a str> {
        line.split(',').nth(index)
    }
}

struct CsvParser<'a> {
    data: &'a str,
}

impl<'a> CsvParser<'a> {
    fn new(data: &'a str) -> Self {
        CsvParser { data }
    }

    fn parse(&self) -> Vec<Vec<&'a str>> {
        self.data
            .lines()
            .map(|line| line.split(',').map(|field| field.trim()).collect())
            .collect()
    }

    // Process without intermediate allocations
    fn process<F>(&self, mut f: F)
    where
        F: FnMut(&[&str]),
    {
        for line in self.data.lines() {
            let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();
            f(&fields);
        }
    }
}

struct StringView<'a> {
    data: &'a str,
    start: usize,
    len: usize,
}

impl<'a> StringView<'a> {
    fn new(data: &'a str, start: usize, len: usize) -> Option<Self> {
        if start + len <= data.len() && data.is_char_boundary(start) {
            if start + len == data.len() || data.is_char_boundary(start + len) {
                return Some(StringView { data, start, len });
            }
        }
        None
    }

    fn as_str(&self) -> &'a str {
        &self.data[self.start..self.start + self.len]
    }

    fn slice(&self, start: usize, len: usize) -> Option<StringView<'a>> {
        if start + len <= self.len {
            StringView::new(self.data, self.start + start, len)
        } else {
            None
        }
    }
}
