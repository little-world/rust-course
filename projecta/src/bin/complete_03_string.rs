// Complete String Interning Implementation
// Implements all 6 milestones from the project specification

use std::borrow::Cow;
use std::collections::HashSet;
use std::time::Instant;

// ============================================================================
// Milestone 1: Understand Cow Basics
// ============================================================================

/// Normalize whitespace: convert double spaces and tabs to single spaces
pub fn normalize_whitespace(text: &str) -> Cow<'_, str> {
    if text.contains("  ") || text.contains('\t') {
        // Need modification - split and rejoin
        let normalized: String = text.split_whitespace().collect::<Vec<&str>>().join(" ");
        Cow::Owned(normalized)
    } else {
        // Already clean - return borrowed
        Cow::Borrowed(text)
    }
}

/// Escape HTML special characters if present
pub fn maybe_escape_html(text: &str) -> Cow<'_, str> {
    if text.contains('<') || text.contains('>') || text.contains('&') {
        // Need escaping - allocate
        let escaped = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        Cow::Owned(escaped)
    } else {
        // No special characters - return borrowed
        Cow::Borrowed(text)
    }
}

// ============================================================================
// Milestone 2, 3, 4: Basic String Interner with Stats
// ============================================================================

#[derive(Debug, Default, PartialEq, Clone)]
pub struct InternerStats {
    pub total_strings: usize,
    pub total_bytes: usize,
    pub allocations: usize,
    pub lookups: usize,
}

impl InternerStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.allocations + self.lookups;
        if total == 0 {
            0.0
        } else {
            self.lookups as f64 / total as f64
        }
    }

    pub fn average_string_length(&self) -> f64 {
        if self.total_strings == 0 {
            0.0
        } else {
            self.total_bytes as f64 / self.total_strings as f64
        }
    }
}

pub struct StringInterner {
    strings: HashSet<Box<str>>,
    stats: InternerStats,
}

impl StringInterner {
    pub fn new() -> Self {
        StringInterner {
            strings: HashSet::new(),
            stats: InternerStats::default(),
        }
    }

    /// Intern a string, returning a reference to the stored version
    pub fn intern(&mut self, s: &str) -> &str {
        if !self.strings.contains(s) {
            // New string - allocate and store
            let s_boxed: Box<str> = Box::from(s);
            self.stats.total_strings += 1;
            self.stats.total_bytes += s.len();
            self.stats.allocations += 1;
            self.strings.insert(s_boxed);
        } else {
            // Already interned - just lookup
            self.stats.lookups += 1;
        }
        // Return reference to stored string
        self.strings.get(s).unwrap().as_ref()
    }

    /// Milestone 3: Get or intern with Cow return type
    pub fn get_or_intern(&mut self, s: &str) -> Cow<'_, str> {
        if self.strings.contains(s) {
            self.stats.lookups += 1;
            Cow::Borrowed(self.strings.get(s).unwrap().as_ref())
        } else {
            let s_boxed: Box<str> = Box::from(s);
            self.stats.total_strings += 1;
            self.stats.total_bytes += s.len();
            self.stats.allocations += 1;
            self.strings.insert(s_boxed);
            Cow::Borrowed(self.strings.get(s).unwrap().as_ref())
        }
    }

    pub fn contains(&self, s: &str) -> bool {
        self.strings.contains(s)
    }

    pub fn len(&self) -> usize {
        self.strings.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    pub fn total_bytes(&self) -> usize {
        self.stats.total_bytes
    }

    /// Milestone 4: Get statistics
    pub fn statistics(&self) -> &InternerStats {
        &self.stats
    }

    pub fn clear(&mut self) {
        self.strings.clear();
        self.stats = InternerStats::default();
    }
}

// ============================================================================
// Milestone 5: Symbol-Based Access with Generational Indices
// ============================================================================

/// A symbol is a handle to an interned string that doesn't carry lifetimes
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub index: usize,
    pub generation: u32,
}

struct Slot {
    string: Option<Box<str>>,
    generation: u32,
}

pub struct SymbolInterner {
    slots: Vec<Slot>,
    free_list: Vec<usize>,
}

impl SymbolInterner {
    pub fn new() -> Self {
        SymbolInterner {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Intern a string and return a symbol handle
    pub fn intern(&mut self, s: &str) -> Symbol {
        // Check if string already exists
        for (index, slot) in self.slots.iter().enumerate() {
            if let Some(existing) = &slot.string {
                if existing.as_ref() == s {
                    return Symbol {
                        index,
                        generation: slot.generation,
                    };
                }
            }
        }

        // Not found - allocate new slot
        let (index, generation) = if let Some(index) = self.free_list.pop() {
            // Reuse freed slot
            let slot = &mut self.slots[index];
            slot.generation = slot.generation.wrapping_add(1);
            slot.string = Some(Box::from(s));
            (index, slot.generation)
        } else {
            // Allocate new slot
            let index = self.slots.len();
            self.slots.push(Slot {
                string: Some(Box::from(s)),
                generation: 0,
            });
            (index, 0)
        };

        Symbol { index, generation }
    }

    /// Resolve a symbol to get the string, or None if stale
    pub fn resolve(&self, symbol: Symbol) -> Option<&str> {
        self.slots.get(symbol.index).and_then(|slot| {
            if slot.generation == symbol.generation {
                slot.string.as_deref()
            } else {
                None
            }
        })
    }

    /// Remove a string from the interner
    pub fn remove(&mut self, symbol: Symbol) {
        if let Some(slot) = self.slots.get_mut(symbol.index) {
            if slot.generation == symbol.generation && slot.string.is_some() {
                slot.string = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(symbol.index);
            }
        }
    }

    /// Clear all strings from the interner
    pub fn clear(&mut self) {
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if slot.string.is_some() {
                slot.string = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free_list.push(index);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.string.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ============================================================================
// Milestone 6: Performance Comparison
// ============================================================================

pub fn run_benchmarks() {
    println!("\n=== String Interning Performance Benchmarks ===");
    benchmark_with_interner();
    benchmark_without_interner();
    println!("===============================================\n");
}

fn benchmark_with_interner() {
    let mut interner = StringInterner::new();
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];

    let start = Instant::now();
    for _ in 0..100_000 {
        for word in &words {
            let _ = interner.intern(word);
        }
    }
    let duration = start.elapsed();

    let stats = interner.statistics();
    println!("With interner:");
    println!("  Duration: {:?}", duration);
    println!("  Unique strings: {}", stats.total_strings);
    println!("  Total bytes: {}", stats.total_bytes);
    println!("  Allocations: {}", stats.allocations);
    println!("  Lookups: {}", stats.lookups);
    println!("  Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    println!(
        "  Avg string length: {:.2} bytes",
        stats.average_string_length()
    );
}

fn benchmark_without_interner() {
    let words = vec!["hello", "world", "foo", "bar", "hello", "world"];
    let mut strings = Vec::new();

    let start = Instant::now();
    for _ in 0..100_000 {
        for word in &words {
            strings.push(word.to_string());
        }
        strings.clear(); // Clear to avoid unbounded memory growth
    }
    let duration = start.elapsed();

    println!("\nWithout interner (alloc + clear):");
    println!("  Duration: {:?}", duration);
    println!("  Total allocations: {}", 100_000 * words.len());
}

// ============================================================================
// Main Function - Demonstrates All Milestones
// ============================================================================

fn main() {
    println!("=== String Interning Project ===\n");

    // Milestone 1: Cow Basics
    println!("--- Milestone 1: Cow Basics ---");
    let clean = "hello world";
    let dirty = "hello  world\ttab";

    let result1 = normalize_whitespace(clean);
    println!("Normalize '{}': {:?}", clean, result1);
    println!("  Is borrowed: {}", matches!(result1, Cow::Borrowed(_)));

    let result2 = normalize_whitespace(dirty);
    println!("Normalize '{}': {:?}", dirty, result2);
    println!("  Is owned: {}", matches!(result2, Cow::Owned(_)));

    let html = "<div>content</div>";
    let result3 = maybe_escape_html(html);
    println!("Escape HTML '{}': {}", html, result3);
    println!("  Is owned: {}", matches!(result3, Cow::Owned(_)));

    // Milestone 2: Basic String Interner
    println!("\n--- Milestone 2: Basic String Interner ---");
    let mut interner = StringInterner::new();

    interner.intern("hello");
    interner.intern("world");
    interner.intern("hello");

    println!("Interned 'hello' twice and 'world' once");
    println!("  Unique strings: {}", interner.len());
    println!("  Total bytes: {}", interner.total_bytes());

    // Test pointer equality separately
    let s1_ptr = interner.intern("test") as *const str;
    let s2_ptr = interner.intern("test") as *const str;
    println!("  Same string returns same pointer: {}", std::ptr::eq(s1_ptr, s2_ptr));

    // Milestone 3: Cow-based API
    println!("\n--- Milestone 3: Cow-based API ---");
    let mut interner2 = StringInterner::new();

    let cow1 = interner2.get_or_intern("test");
    println!("First intern: {:?}", cow1);
    println!("  Is borrowed: {}", matches!(cow1, Cow::Borrowed(_)));

    let cow2 = interner2.get_or_intern("test");
    println!("Second intern: {:?}", cow2);
    println!("  Is borrowed: {}", matches!(cow2, Cow::Borrowed(_)));

    // Milestone 4: Statistics Tracking
    println!("\n--- Milestone 4: Statistics Tracking ---");
    let mut interner3 = StringInterner::new();

    for word in &["foo", "bar", "foo", "baz", "foo", "bar"] {
        interner3.intern(word);
    }

    let stats = interner3.statistics();
    println!("After interning 6 strings (3 unique):");
    println!("  Total strings: {}", stats.total_strings);
    println!("  Total bytes: {}", stats.total_bytes);
    println!("  Allocations: {}", stats.allocations);
    println!("  Lookups: {}", stats.lookups);
    println!("  Hit rate: {:.1}%", stats.hit_rate() * 100.0);

    // Milestone 5: Symbol-Based Access
    println!("\n--- Milestone 5: Symbol-Based Access ---");
    let mut sym_interner = SymbolInterner::new();

    let sym1 = sym_interner.intern("alpha");
    let sym2 = sym_interner.intern("beta");
    let sym3 = sym_interner.intern("alpha");

    println!("Symbol 1: {:?}", sym1);
    println!("Symbol 2: {:?}", sym2);
    println!("Symbol 3: {:?}", sym3);
    println!("  sym1 == sym3: {}", sym1 == sym3);

    println!("Resolve sym1: {:?}", sym_interner.resolve(sym1));
    println!("Resolve sym2: {:?}", sym_interner.resolve(sym2));

    // Test stale detection
    sym_interner.remove(sym1);
    println!("After removing sym1:");
    println!("  Resolve sym1: {:?}", sym_interner.resolve(sym1));
    println!("  Resolve sym3: {:?}", sym_interner.resolve(sym3));

    let sym4 = sym_interner.intern("alpha");
    println!("New symbol for 'alpha': {:?}", sym4);
    println!("  Same index: {}", sym1.index == sym4.index);
    println!("  Different generation: {}", sym1.generation != sym4.generation);

    // Milestone 6: Performance Comparison
    println!("\n--- Milestone 6: Performance Comparison ---");
    run_benchmarks();

    println!("=== All Milestones Complete! ===");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Milestone 1 Tests
    #[test]
    fn test_normalize_no_change() {
        let result = normalize_whitespace("hello world");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_normalize_with_change() {
        let result = normalize_whitespace("hello  world\tagain");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "hello world again");
    }

    #[test]
    fn test_normalize_tabs() {
        let result = normalize_whitespace("hello\tworld");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_escape_no_html() {
        let result = maybe_escape_html("hello");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_escape_with_html() {
        let result = maybe_escape_html("<div>");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "&lt;div&gt;");
    }

    #[test]
    fn test_escape_ampersand() {
        let result = maybe_escape_html("a & b");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "a &amp; b");
    }

    #[test]
    fn test_escape_complex() {
        let result = maybe_escape_html("<script>alert('&')</script>");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(
            result,
            "&lt;script&gt;alert('&amp;')&lt;/script&gt;"
        );
    }

    // Milestone 2 Tests
    #[test]
    fn test_intern_basic() {
        let mut interner = StringInterner::new();

        let s1_ptr = interner.intern("hello") as *const str;
        let s2_ptr = interner.intern("hello") as *const str;

        assert!(std::ptr::eq(s1_ptr, s2_ptr));
        assert_eq!(interner.len(), 1);
    }

    #[test]
    fn test_intern_different() {
        let mut interner = StringInterner::new();

        let s1_ptr = interner.intern("hello") as *const str;
        let s2_ptr = interner.intern("world") as *const str;

        assert!(!std::ptr::eq(s1_ptr, s2_ptr));
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_contains() {
        let mut interner = StringInterner::new();
        interner.intern("hello");

        assert!(interner.contains("hello"));
        assert!(!interner.contains("world"));
    }

    #[test]
    fn test_total_bytes() {
        let mut interner = StringInterner::new();
        interner.intern("hi"); // 2 bytes
        interner.intern("hello"); // 5 bytes

        assert_eq!(interner.total_bytes(), 7);
    }

    // Milestone 3 Tests
    #[test]
    fn test_cow_already_interned() {
        let mut interner = StringInterner::new();
        interner.intern("hello");

        let result = interner.get_or_intern("hello");
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn test_cow_new_string() {
        let mut interner = StringInterner::new();

        let result = interner.get_or_intern("hello");
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(interner.len(), 1);
    }

    // Milestone 4 Tests
    #[test]
    fn test_stats() {
        let mut interner = StringInterner::new();

        interner.intern("hello"); // allocation
        interner.intern("world"); // allocation
        interner.intern("hello"); // lookup

        let stats = interner.statistics();
        assert_eq!(stats.total_strings, 2);
        assert_eq!(stats.total_bytes, 10); // 5 + 5
        assert_eq!(stats.allocations, 2);
        assert_eq!(stats.lookups, 1);
    }

    #[test]
    fn test_stats_empty() {
        let interner = StringInterner::new();
        let stats = interner.statistics();
        assert_eq!(stats.total_strings, 0);
        assert_eq!(stats.allocations, 0);
        assert_eq!(stats.lookups, 0);
    }

    #[test]
    fn test_hit_rate() {
        let mut interner = StringInterner::new();

        interner.intern("a"); // alloc
        interner.intern("a"); // lookup
        interner.intern("a"); // lookup
        interner.intern("b"); // alloc

        let stats = interner.statistics();
        assert_eq!(stats.hit_rate(), 0.5); // 2 lookups out of 4 total
    }

    // Milestone 5 Tests
    #[test]
    fn test_symbol_intern() {
        let mut interner = SymbolInterner::new();

        let sym1 = interner.intern("hello");
        let sym2 = interner.intern("hello");

        assert_eq!(sym1, sym2);
        assert_eq!(interner.resolve(sym1), Some("hello"));
    }

    #[test]
    fn test_symbol_resolve() {
        let mut interner = SymbolInterner::new();

        let sym = interner.intern("test");
        assert_eq!(interner.resolve(sym), Some("test"));
    }

    #[test]
    fn test_stale_symbol() {
        let mut interner = SymbolInterner::new();

        let sym1 = interner.intern("test");
        interner.clear();

        assert_eq!(interner.resolve(sym1), None);
    }

    #[test]
    fn test_generation_reuse() {
        let mut interner = SymbolInterner::new();

        let sym1 = interner.intern("test");
        let index1 = sym1.index;
        let gen1 = sym1.generation;

        interner.remove(sym1);

        let sym2 = interner.intern("test");
        assert_eq!(sym2.index, index1); // Same slot
        assert_ne!(sym2.generation, gen1); // Different generation
    }

    #[test]
    fn test_symbol_lifetime_safety() {
        let mut interner = SymbolInterner::new();
        let sym = interner.intern("test");

        // Symbol can outlive the borrow of interner
        drop(interner);

        // This is safe - we just can't resolve it anymore
        let _copy = sym; // Symbol is Copy
    }

    #[test]
    fn test_symbol_remove() {
        let mut interner = SymbolInterner::new();

        let sym1 = interner.intern("hello");
        let sym2 = interner.intern("world");

        assert_eq!(interner.len(), 2);

        interner.remove(sym1);
        assert_eq!(interner.len(), 1);
        assert_eq!(interner.resolve(sym1), None);
        assert_eq!(interner.resolve(sym2), Some("world"));
    }

    #[test]
    fn test_symbol_different_strings() {
        let mut interner = SymbolInterner::new();

        let sym1 = interner.intern("hello");
        let sym2 = interner.intern("world");

        assert_ne!(sym1, sym2);
        assert_eq!(interner.resolve(sym1), Some("hello"));
        assert_eq!(interner.resolve(sym2), Some("world"));
    }

    #[test]
    fn test_multiple_removes_and_interns() {
        let mut interner = SymbolInterner::new();

        let sym1 = interner.intern("a");
        let sym2 = interner.intern("b");

        interner.remove(sym1);
        interner.remove(sym2);

        let sym3 = interner.intern("c");
        let sym4 = interner.intern("d");

        // Should reuse slots
        assert_eq!(interner.resolve(sym3), Some("c"));
        assert_eq!(interner.resolve(sym4), Some("d"));
        assert_eq!(interner.resolve(sym1), None);
        assert_eq!(interner.resolve(sym2), None);
    }
}
