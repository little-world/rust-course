//! Pattern 2: String Builder Pattern
//! Domain-Specific HTML Builder
//!
//! Run with: cargo run --example p2_html_builder

fn main() {
    println!("=== HTML Builder Pattern ===\n");

    // HTML building
    let mut html = HtmlBuilder::new();
    html.open_tag("html")
        .open_tag("body")
        .open_tag("h1")
        .content("Welcome")
        .close_tag("h1")
        .open_tag("p")
        .content("This is a paragraph")
        .close_tag("p")
        .close_tag("body")
        .close_tag("html");

    println!("Generated HTML:\n{}", html.build());

    println!("=== Key Points ===");
    println!("1. Domain-specific builders wrap generic StringBuilder");
    println!("2. Automatic indentation for readability");
    println!("3. Fluent API with method chaining");
}

// Internal StringBuilder
struct StringBuilder {
    buffer: String,
}

impl StringBuilder {
    fn with_capacity(capacity: usize) -> Self {
        StringBuilder {
            buffer: String::with_capacity(capacity),
        }
    }

    fn append(&mut self, s: &str) -> &mut Self {
        self.buffer.push_str(s);
        self
    }

    fn build(self) -> String {
        self.buffer
    }
}

// HTML Builder wrapping StringBuilder
struct HtmlBuilder {
    builder: StringBuilder,
    indent: usize,
}

impl HtmlBuilder {
    fn new() -> Self {
        HtmlBuilder {
            builder: StringBuilder::with_capacity(1024),
            indent: 0,
        }
    }

    fn open_tag(&mut self, tag: &str) -> &mut Self {
        self.write_indent();
        self.builder.append("<").append(tag).append(">\n");
        self.indent += 2;
        self
    }

    fn close_tag(&mut self, tag: &str) -> &mut Self {
        self.indent -= 2;
        self.write_indent();
        self.builder.append("</").append(tag).append(">\n");
        self
    }

    fn content(&mut self, text: &str) -> &mut Self {
        self.write_indent();
        self.builder.append(text).append("\n");
        self
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.builder.append(" ");
        }
    }

    fn build(self) -> String {
        self.builder.build()
    }
}
