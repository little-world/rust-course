//! Pattern 1: Capacity Management
//! Example: Shrink Long-Lived Data Structures
//!
//! Run with: cargo run --example p1_shrink_to_fit

fn main() {
    println!("=== Shrink Long-Lived Data Structures ===\n");

    // Demonstrate over-allocation and shrinking
    #[derive(Debug, Clone)]
    struct Entry {
        id: usize,
        value: String,
        active: bool,
    }

    impl Entry {
        fn should_index(&self) -> bool {
            self.active
        }
    }

    #[derive(Debug)]
    struct IndexEntry {
        id: usize,
        value: String,
    }

    impl From<&Entry> for IndexEntry {
        fn from(entry: &Entry) -> Self {
            IndexEntry {
                id: entry.id,
                value: entry.value.clone(),
            }
        }
    }

    fn build_lookup_table(entries: &[Entry]) -> Vec<IndexEntry> {
        // Over-estimate capacity (not all entries will be indexed)
        let mut table = Vec::with_capacity(entries.len() * 2);

        println!("  Initial capacity: {}", table.capacity());

        for entry in entries {
            if entry.should_index() {
                table.push(IndexEntry::from(entry));
            }
        }

        println!("  After filtering: len={}, capacity={}", table.len(), table.capacity());

        // Reclaim unused space for long-lived data
        table.shrink_to_fit();

        println!("  After shrink_to_fit: len={}, capacity={}", table.len(), table.capacity());

        table
    }

    // Create entries where only some are active
    let entries: Vec<Entry> = (0..100)
        .map(|i| Entry {
            id: i,
            value: format!("entry_{}", i),
            active: i % 3 == 0, // Only 1/3 are active
        })
        .collect();

    println!("Building lookup table from {} entries:\n", entries.len());
    let table = build_lookup_table(&entries);
    println!("\nFinal table size: {} entries", table.len());

    // Demonstrate shrink_to
    println!("\n=== shrink_to vs shrink_to_fit ===\n");

    let mut vec: Vec<i32> = Vec::with_capacity(100);
    vec.extend(0..20);

    println!("Initial: len={}, capacity={}", vec.len(), vec.capacity());

    // shrink_to allows keeping some extra capacity
    vec.shrink_to(30);
    println!("After shrink_to(30): capacity={}", vec.capacity());

    // shrink_to_fit removes all excess
    vec.shrink_to_fit();
    println!("After shrink_to_fit: capacity={}", vec.capacity());

    // When NOT to shrink
    println!("\n=== When NOT to Shrink ===\n");

    println!("Avoid shrink_to_fit on:");
    println!("  - Vectors that will grow again soon");
    println!("  - Temporary buffers in loops");
    println!("  - Small vectors (overhead not worth it)");

    println!("\nUse shrink_to_fit for:");
    println!("  - Long-lived lookup tables");
    println!("  - Cached data that won't change");
    println!("  - Memory-constrained environments");

    // Memory estimation
    println!("\n=== Memory Savings ===\n");

    let original_capacity = 1000;
    let actual_size = 100;
    let element_size = std::mem::size_of::<i32>();

    let wasted_before = (original_capacity - actual_size) * element_size;
    let wasted_after = 0;

    println!("Vector with {} elements, capacity {}:", actual_size, original_capacity);
    println!("  Element size: {} bytes", element_size);
    println!("  Wasted before shrink: {} bytes", wasted_before);
    println!("  Wasted after shrink: {} bytes", wasted_after);
    println!("  Memory saved: {} bytes", wasted_before - wasted_after);

    println!("\n=== Key Points ===");
    println!("1. shrink_to_fit() reclaims unused capacity");
    println!("2. Use for long-lived data structures after construction");
    println!("3. Don't use on frequently modified vectors");
    println!("4. shrink_to(n) keeps minimum capacity of n");
}
