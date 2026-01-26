//! Pattern 3: Trait Objects and Dynamic Dispatch
//! Example: Trait Objects with Lifetime Bounds
//!
//! Run with: cargo run --example p3_lifetime_bounds

trait Processor {
    fn process(&self, data: &str) -> String;
}

// Simple processor - owns all its data
struct UpperCaseProcessor;

impl Processor for UpperCaseProcessor {
    fn process(&self, data: &str) -> String {
        data.to_uppercase()
    }
}

// Processor that borrows data
struct PrefixProcessor<'a> {
    prefix: &'a str,
}

impl<'a> Processor for PrefixProcessor<'a> {
    fn process(&self, data: &str) -> String {
        format!("{}{}", self.prefix, data)
    }
}

// Processor with owned prefix
struct OwnedPrefixProcessor {
    prefix: String,
}

impl Processor for OwnedPrefixProcessor {
    fn process(&self, data: &str) -> String {
        format!("{}{}", self.prefix, data)
    }
}

// Function taking trait object with lifetime
fn process_data<'a>(processor: &'a dyn Processor, data: &'a str) -> String {
    processor.process(data)
}

// Struct holding boxed trait object with lifetime
struct Handler<'a> {
    processor: Box<dyn Processor + 'a>,
}

impl<'a> Handler<'a> {
    fn new(processor: Box<dyn Processor + 'a>) -> Self {
        Handler { processor }
    }

    fn handle(&self, data: &str) -> String {
        self.processor.process(data)
    }
}

fn main() {
    // Usage: Lifetime bounds ensure trait object outlives borrowed data.
    println!("=== Simple Processors ===");
    let upper = UpperCaseProcessor;
    let result = process_data(&upper, "hello world");
    println!("Uppercase: {}", result);

    println!("\n=== Processor with Borrowed Data ===");
    let prefix = String::from(">>> ");
    let prefixer = PrefixProcessor { prefix: &prefix };
    let result = process_data(&prefixer, "message");
    println!("Prefixed: {}", result);

    println!("\n=== Handler with Boxed Trait Object ===");
    // Handler with 'static processor
    let handler1 = Handler::new(Box::new(UpperCaseProcessor));
    println!("Handler1: {}", handler1.handle("test"));

    // Handler with owned processor
    let handler2 = Handler::new(Box::new(OwnedPrefixProcessor {
        prefix: "==> ".to_string(),
    }));
    println!("Handler2: {}", handler2.handle("test"));

    // Handler with borrowed processor (needs lifetime)
    {
        let my_prefix = String::from("### ");
        let prefixer = PrefixProcessor { prefix: &my_prefix };
        let handler3 = Handler::new(Box::new(prefixer));
        println!("Handler3: {}", handler3.handle("test"));
        // handler3 must not outlive my_prefix
    }

    println!("\n=== Multiple Processors in Collection ===");
    let owned_prefix = OwnedPrefixProcessor {
        prefix: "[INFO] ".to_string(),
    };

    // Collection of processors with 'static lifetime
    let processors: Vec<Box<dyn Processor>> = vec![
        Box::new(UpperCaseProcessor),
        Box::new(OwnedPrefixProcessor {
            prefix: "LOG: ".to_string(),
        }),
    ];

    for (i, p) in processors.iter().enumerate() {
        println!("Processor {}: {}", i, p.process("data"));
    }

    // Reference to non-'static processor
    let result = owned_prefix.process("message");
    println!("\nOwned prefix result: {}", result);
}
