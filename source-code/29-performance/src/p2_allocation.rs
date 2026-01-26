// Pattern 2: Allocation Reduction
// Demonstrates techniques for reducing heap allocations.

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Instant;
use smallvec::SmallVec;
use typed_arena::Arena;

// ============================================================================
// Example: Allocation vs Stack Allocation
// ============================================================================

fn allocation_benchmark() {
    let iterations = 1_000_000;

    // Heap allocating
    let start = Instant::now();
    for _ in 0..iterations {
        let _v: Vec<i32> = vec![1, 2, 3, 4, 5];
    }
    println!("Heap allocating: {:?}", start.elapsed());

    // Stack only
    let start = Instant::now();
    for _ in 0..iterations {
        let _arr = [1, 2, 3, 4, 5];
    }
    println!("Stack allocation: {:?}", start.elapsed());
}

// ============================================================================
// Example: Reusing Allocations
// ============================================================================

// Bad: Allocates every iteration
fn process_bad(items: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    for item in items {
        let mut buffer = String::new();  // Allocates!
        buffer.push_str("processed_");
        buffer.push_str(item);
        results.push(buffer);
    }
    results
}

// Good: Reuses buffer
fn process_good(items: &[String]) -> Vec<String> {
    let mut results = Vec::new();
    let mut buffer = String::new();  // Allocate once
    for item in items {
        buffer.clear();  // Reuse allocation
        buffer.push_str("processed_");
        buffer.push_str(item);
        results.push(buffer.clone());
    }
    results
}

// Better: Pre-allocate
fn process_better(items: &[String]) -> Vec<String> {
    let mut results = Vec::with_capacity(items.len());  // Pre-allocate
    for item in items {
        let processed = format!("processed_{}", item);
        results.push(processed);
    }
    results
}

// Best: Use iterators
fn process_best(items: &[String]) -> Vec<String> {
    items
        .iter()
        .map(|item| format!("processed_{}", item))
        .collect()
}

// ============================================================================
// Example: SmallVec - Stack-Allocated Small Collections
// ============================================================================

type SmallVec4<T> = SmallVec<[T; 4]>;

fn process_with_smallvec(items: &[i32]) -> SmallVec4<i32> {
    let mut result = SmallVec4::new();
    for &item in items {
        if item % 2 == 0 {
            result.push(item);
        }
        if result.len() >= 4 {
            break;
        }
    }
    result
}

// ============================================================================
// Example: Cow - Clone-On-Write
// ============================================================================

fn process_string(input: &str) -> Cow<str> {
    if input.contains("bad") {
        // Must modify - allocates
        Cow::Owned(input.replace("bad", "good"))
    } else {
        // No modification needed - no allocation
        Cow::Borrowed(input)
    }
}

// ============================================================================
// Example: Arena Allocation
// ============================================================================

struct Node<'a> {
    value: i32,
    children: Vec<&'a Node<'a>>,
}

fn build_tree<'a>(arena: &'a Arena<Node<'a>>) -> &'a Node<'a> {
    let child1 = arena.alloc(Node {
        value: 1,
        children: vec![],
    });

    let child2 = arena.alloc(Node {
        value: 2,
        children: vec![],
    });

    arena.alloc(Node {
        value: 0,
        children: vec![child1, child2],
    })
}

// ============================================================================
// Example: String Interning
// ============================================================================

struct StringInterner {
    strings: HashMap<String, usize>,
    reverse: Vec<String>,
}

impl StringInterner {
    fn new() -> Self {
        StringInterner {
            strings: HashMap::new(),
            reverse: Vec::new(),
        }
    }

    fn intern(&mut self, s: &str) -> usize {
        if let Some(&id) = self.strings.get(s) {
            id
        } else {
            let id = self.reverse.len();
            self.reverse.push(s.to_string());
            self.strings.insert(s.to_string(), id);
            id
        }
    }

    fn get(&self, id: usize) -> &str {
        &self.reverse[id]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_methods_equivalent() {
        let items: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let r1 = process_bad(&items);
        let r2 = process_good(&items);
        let r3 = process_better(&items);
        let r4 = process_best(&items);

        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
        assert_eq!(r3, r4);
    }

    #[test]
    fn test_smallvec_no_heap_for_small() {
        let items = [2, 4, 6, 8, 10];
        let result = process_with_smallvec(&items);
        assert_eq!(result.len(), 4);
        assert!(!result.spilled()); // Still on stack
    }

    #[test]
    fn test_cow_no_allocation_when_unchanged() {
        let good_text = "good text";
        let result = process_string(good_text);
        assert!(matches!(result, Cow::Borrowed(_)));

        let bad_text = "bad text";
        let result = process_string(bad_text);
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "good text");
    }

    #[test]
    fn test_arena_allocation() {
        let arena = Arena::new();
        let tree = build_tree(&arena);
        assert_eq!(tree.value, 0);
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].value, 1);
        assert_eq!(tree.children[1].value, 2);
    }

    #[test]
    fn test_string_interning() {
        let mut interner = StringInterner::new();

        let id1 = interner.intern("hello");
        let id2 = interner.intern("hello");
        let id3 = interner.intern("world");

        assert_eq!(id1, id2);  // Same string, same ID
        assert_ne!(id1, id3);  // Different string, different ID
        assert_eq!(interner.get(id1), "hello");
        assert_eq!(interner.get(id3), "world");
    }
}

fn main() {
    println!("Pattern 2: Allocation Reduction");
    println!("================================\n");

    println!("Allocation benchmark:");
    allocation_benchmark();

    println!("\nSmallVec demo:");
    let items = [1, 2, 3, 4, 5, 6, 7, 8];
    let result = process_with_smallvec(&items);
    println!("  Result: {:?}, spilled: {}", result.as_slice(), result.spilled());

    println!("\nCow demo:");
    println!("  process_string(\"good text\"): {}", process_string("good text"));
    println!("  process_string(\"bad text\"): {}", process_string("bad text"));

    println!("\nArena demo:");
    let arena = Arena::new();
    let tree = build_tree(&arena);
    println!("  Tree root value: {}, children: {}", tree.value, tree.children.len());

    println!("\nString interning demo:");
    let mut interner = StringInterner::new();
    let id1 = interner.intern("hello");
    let id2 = interner.intern("hello");
    println!("  'hello' interned twice: id1={}, id2={}, equal={}", id1, id2, id1 == id2);
}
