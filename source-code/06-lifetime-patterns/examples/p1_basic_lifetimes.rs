//! Pattern 1: Named Lifetimes and Elision
//! Example: Basic Lifetime Annotations and Elision Rules
//!
//! Run with: cargo run --example p1_basic_lifetimes

// Explicit lifetime 'a ties return lifetime to both inputs.
// The returned reference is valid as long as both inputs are valid.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// Elision Rule 2: Single input lifetime → all output lifetimes.
// Compiler infers: fn first_word<'a>(s: &'a str) -> &'a str
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or(s)
}

// Struct holding a reference needs a lifetime parameter.
struct MyString<'a> {
    text: &'a str,
}

// Elision Rule 3: &self lifetime → all output lifetimes.
// Compiler infers: fn get_text(&'a self) -> &'a str
impl<'a> MyString<'a> {
    fn get_text(&self) -> &str {
        self.text
    }

    fn get_prefix(&self, len: usize) -> &str {
        if len <= self.text.len() {
            &self.text[..len]
        } else {
            self.text
        }
    }
}

// Multiple lifetime parameters when inputs have independent lifetimes.
fn pick_first<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str {
    x // Return is tied to 'a only, not 'b
}

fn main() {
    println!("=== Basic Lifetime Annotation ===");
    // Usage: 'a ties return lifetime to both inputs; result valid while both exist.
    let string1 = String::from("long string is long");
    let string2 = String::from("short");
    let result = longest(&string1, &string2);
    println!("Longest: {}", result);

    println!("\n=== Lifetime Elision Rule 2 ===");
    // Single input lifetime automatically assigned to output.
    let sentence = "hello world from rust";
    let word = first_word(sentence);
    println!("First word of '{}': '{}'", sentence, word);

    println!("\n=== Struct with Lifetime ===");
    let text = "example text here";
    let my_string = MyString { text };
    println!("Full text: {}", my_string.get_text());
    println!("Prefix(7): {}", my_string.get_prefix(7));

    println!("\n=== Elision Rule 3 (&self) ===");
    // Method return lifetime tied to &self automatically.
    let ms = MyString { text: "hello world" };
    let retrieved = ms.get_text();
    println!("Retrieved: {}", retrieved);

    println!("\n=== Independent Lifetimes ===");
    let a = String::from("first");
    let b = String::from("second");
    let picked = pick_first(&a, &b);
    println!("Picked: {}", picked);

    println!("\n=== Lifetime Scope Demonstration ===");
    let outer = String::from("outer string");
    {
        let inner = String::from("inner");
        let result = longest(&outer, &inner);
        println!("Inside block: {}", result);
        // `result` is valid here because both outer and inner are alive
    }
    // `inner` is dropped, but `outer` is still valid
    println!("After block: {}", outer);

    println!("\n=== Elision Rules Summary ===");
    println!("Rule 1: Each elided input lifetime gets its own parameter");
    println!("Rule 2: Single input lifetime → all output lifetimes");
    println!("Rule 3: &self lifetime → all output lifetimes");
    println!("\nThese rules cover 90%+ of cases automatically!");
}
