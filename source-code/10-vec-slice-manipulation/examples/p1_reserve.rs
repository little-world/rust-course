//! Pattern 1: Capacity Management
//! Example: Reserve Before Iterative Construction
//!
//! Run with: cargo run --example p1_reserve

fn main() {
    println!("=== Reserve Before Bulk Operations ===\n");

    // Simulate building results from multiple queries
    struct Query {
        name: &'static str,
        estimated_count: usize,
    }

    impl Query {
        fn estimated_results(&self) -> usize {
            self.estimated_count
        }
    }

    fn execute_query(query: &Query) -> Vec<i32> {
        // Simulate query results
        (0..query.estimated_count as i32).collect()
    }

    fn build_result_set(queries: &[Query]) -> Vec<i32> {
        let mut results = Vec::new();

        // Estimate total size to avoid multiple reallocations
        let estimated_total: usize = queries.iter()
            .map(|q| q.estimated_results())
            .sum();

        println!("  Estimated total results: {}", estimated_total);
        results.reserve(estimated_total);
        println!("  Reserved capacity: {}", results.capacity());

        for query in queries {
            for result in execute_query(query) {
                results.push(result);
            }
        }

        results
    }

    let queries = vec![
        Query { name: "query1", estimated_count: 100 },
        Query { name: "query2", estimated_count: 200 },
        Query { name: "query3", estimated_count: 150 },
    ];

    let results = build_result_set(&queries);
    println!("  Final length: {}", results.len());
    println!("  Final capacity: {}", results.capacity());

    // Demonstrate reserve vs reserve_exact
    println!("\n=== reserve vs reserve_exact ===\n");

    let mut vec1: Vec<i32> = Vec::new();
    vec1.reserve(100);
    println!("reserve(100):");
    println!("  Capacity: {} (may be >= 100)", vec1.capacity());

    let mut vec2: Vec<i32> = Vec::new();
    vec2.reserve_exact(100);
    println!("\nreserve_exact(100):");
    println!("  Capacity: {} (exactly 100)", vec2.capacity());

    // Demonstrate additional reserve
    println!("\n=== Additional Reserve ===\n");

    let mut data: Vec<i32> = Vec::with_capacity(10);
    data.extend(0..10);
    println!("Initial: len={}, capacity={}", data.len(), data.capacity());

    // Need to add 20 more elements
    data.reserve(20);
    println!("After reserve(20): capacity={}", data.capacity());

    data.extend(10..30);
    println!("After adding 20 more: len={}, capacity={}", data.len(), data.capacity());

    println!("\n=== Key Points ===");
    println!("1. reserve(n) ensures space for at least n more elements");
    println!("2. reserve_exact(n) allocates exactly n more (no extra)");
    println!("3. Call reserve before bulk operations to minimize allocations");
    println!("4. Estimate sizes when exact count is unknown");
}
