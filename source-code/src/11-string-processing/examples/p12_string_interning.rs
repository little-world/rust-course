//! Pattern 12: String Interning
//! Deduplicate Repeated Strings for Memory Savings
//!
//! Run with: cargo run --example p12_string_interning

use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("=== String Interning ===\n");

    let mut interner = StringInterner::new();

    // Basic interning
    println!("=== Basic Interning ===\n");

    let s1 = interner.intern("hello");
    let s2 = interner.intern("world");
    let s3 = interner.intern("hello");  // Returns same Arc

    println!("s1 = intern(\"hello\"): '{}'", s1.as_str());
    println!("s2 = intern(\"world\"): '{}'", s2.as_str());
    println!("s3 = intern(\"hello\"): '{}'", s3.as_str());
    println!("\ns1 == s3: {}", s1 == s3);
    println!("Unique strings: {}", interner.len());

    // Memory savings demonstration
    println!("\n=== Memory Savings ===\n");

    let tags = vec!["rust", "programming", "rust", "tutorial", "rust",
                    "programming", "rust", "guide", "rust"];

    println!("Input tags: {:?}", tags);
    println!("Total strings: {}", tags.len());

    let interned_tags: Vec<_> = tags.iter()
        .map(|&s| interner.intern(s))
        .collect();

    println!("Unique strings after interning: {}", interner.len());
    println!("Memory used: {} bytes", interner.memory_usage());

    // Without interning
    let without_interning: usize = tags.iter().map(|s| s.len()).sum();
    println!("Without interning would use: {} bytes (+ overhead per string)", without_interning);

    // Comparison efficiency
    println!("\n=== Comparison Efficiency ===\n");

    let a = interner.intern("a_long_identifier_name");
    let b = interner.intern("a_long_identifier_name");
    let c = interner.intern("different_identifier");

    // Interned comparison is O(1) - just pointer comparison
    println!("a == b (same interned): {}", a == b);
    println!("a == c (different): {}", a == c);
    println!("Arc pointer comparison is O(1), not O(N)!");

    // Use case: Symbol table
    println!("\n=== Use Case: Symbol Table ===\n");

    let mut symbols = StringInterner::new();

    // Simulate parsing code with repeated identifiers
    let identifiers = [
        "main", "args", "result", "main", "println", "result",
        "main", "args", "temp", "result", "println", "main",
    ];

    for ident in &identifiers {
        symbols.intern(ident);
    }

    println!("Parsed {} identifier occurrences", identifiers.len());
    println!("Unique symbols: {}", symbols.len());
    println!("Memory for symbols: {} bytes", symbols.memory_usage());

    println!("\n=== Key Points ===");
    println!("1. Arc<str> for thread-safe reference counting");
    println!("2. O(1) comparison via pointer equality");
    println!("3. Memory savings: N copies -> 1 copy");
    println!("4. Ideal for compilers, configs, logging");
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct InternedString(Arc<str>);

impl InternedString {
    fn as_str(&self) -> &str {
        &self.0
    }
}

struct StringInterner {
    map: HashMap<Arc<str>, InternedString>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            map: HashMap::new(),
        }
    }

    fn intern(&mut self, s: &str) -> InternedString {
        if let Some(interned) = self.map.get(s) {
            return interned.clone();
        }

        let arc: Arc<str> = Arc::from(s);
        let interned = InternedString(arc.clone());
        self.map.insert(arc, interned.clone());
        interned
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn memory_usage(&self) -> usize {
        self.map.iter()
            .map(|(k, _)| k.len())
            .sum()
    }
}
