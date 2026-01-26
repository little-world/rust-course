//! Pattern 7: Higher-Ranked Trait Bounds (HRTBs)
//! Example: Callback Storage with HRTB
//!
//! Run with: cargo run --example p7_callback_storage

use std::sync::atomic::{AtomicUsize, Ordering};

// Callback type with HRTB - works with events of any lifetime
type Callback = Box<dyn for<'a> Fn(&'a str)>;

struct EventEmitter {
    callbacks: Vec<Callback>,
}

impl EventEmitter {
    fn new() -> Self {
        EventEmitter {
            callbacks: Vec::new(),
        }
    }

    fn on<F>(&mut self, callback: F)
    where
        F: for<'a> Fn(&'a str) + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    fn emit(&self, event: &str) {
        for callback in &self.callbacks {
            callback(event);
        }
    }
}

// Parser combinator example with HRTB
struct BoxedParser<Output> {
    parser: Box<dyn for<'a> Fn(&'a str) -> Option<(Output, &'a str)>>,
}

impl<Output: 'static> BoxedParser<Output> {
    fn new<F>(f: F) -> Self
    where
        F: for<'a> Fn(&'a str) -> Option<(Output, &'a str)> + 'static,
    {
        BoxedParser {
            parser: Box::new(f),
        }
    }

    fn parse<'a>(&self, input: &'a str) -> Option<(Output, &'a str)> {
        (self.parser)(input)
    }
}

// Iterator extension with HRTB
trait IteratorExt: Iterator {
    fn for_each_ref<F>(self, f: F)
    where
        Self: Sized,
        F: for<'a> FnMut(&'a Self::Item);
}

impl<I: Iterator> IteratorExt for I {
    fn for_each_ref<F>(self, mut f: F)
    where
        F: for<'a> FnMut(&'a Self::Item),
    {
        for item in self {
            f(&item);
        }
    }
}

fn main() {
    println!("=== Event Emitter with HRTB Callbacks ===");
    let mut emitter = EventEmitter::new();

    // Register callbacks
    emitter.on(|event| println!("Callback 1: {}", event));
    emitter.on(|event| println!("Callback 2: {} (length: {})", event, event.len()));

    // Callback that captures state
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    emitter.on(|_event| {
        let count = COUNTER.fetch_add(1, Ordering::SeqCst);
        println!("Callback 3: Event #{}", count);
    });

    println!("Emitting 'click':");
    emitter.emit("click");

    println!("\nEmitting 'submit':");
    emitter.emit("submit");

    println!("\n=== Parser Combinator with HRTB ===");
    // Parser that extracts a digit
    let digit_parser = BoxedParser::new(|s: &str| {
        s.chars()
            .next()
            .filter(|c| c.is_ascii_digit())
            .map(|c| (c, &s[1..]))
    });

    println!("Parsing '123abc':");
    if let Some((ch, rest)) = digit_parser.parse("123abc") {
        println!("  Got digit '{}', remaining: \"{}\"", ch, rest);
    }

    println!("Parsing 'abc123':");
    match digit_parser.parse("abc123") {
        Some((ch, rest)) => println!("  Got digit '{}', remaining: \"{}\"", ch, rest),
        None => println!("  No digit at start"),
    }

    println!("\n=== Iterator Extension with HRTB ===");
    let mut sum = 0;
    vec![1, 2, 3, 4, 5].into_iter().for_each_ref(|x| {
        sum += x;
        println!("Processing: {}, running sum: {}", x, sum);
    });
    println!("Final sum: {}", sum);

    println!("\n=== Why HRTB for Stored Callbacks? ===");
    println!("Without HRTB, you'd need to parameterize the event lifetime:");
    println!("  struct EventEmitter<'event> {{ callbacks: Vec<Box<dyn Fn(&'event str)>> }}");
    println!();
    println!("This ties the emitter to ONE specific lifetime!");
    println!("With HRTB, callbacks work with events of ANY lifetime.");
}
