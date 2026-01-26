//! Pattern 2: Slice Algorithms
//! Example: Binary Search on Sorted Data
//!
//! Run with: cargo run --example p2_binary_search

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
}

fn main() {
    println!("=== Binary Search on Sorted Data ===\n");

    // Basic binary search
    let sorted = vec![1, 3, 5, 7, 9, 11, 13, 15];
    println!("Sorted array: {:?}", sorted);

    for target in [5, 8, 15, 0] {
        match sorted.binary_search(&target) {
            Ok(idx) => println!("  Found {} at index {}", target, idx),
            Err(idx) => println!("  {} not found, would insert at index {}", target, idx),
        }
    }

    // Binary search by key
    println!("\n=== Binary Search by Key ===\n");

    let mut users = vec![
        User { id: 3, name: "Charlie".to_string() },
        User { id: 1, name: "Alice".to_string() },
        User { id: 4, name: "Diana".to_string() },
        User { id: 2, name: "Bob".to_string() },
    ];

    // Must sort first!
    users.sort_by_key(|u| u.id);
    println!("Sorted users by ID:");
    for user in &users {
        println!("  {:?}", user);
    }

    fn find_user_by_id(users: &[User], id: u64) -> Option<&User> {
        users.binary_search_by_key(&id, |u| u.id)
            .ok()
            .map(|idx| &users[idx])
    }

    println!("\nSearching for users:");
    for id in [1, 2, 5, 4] {
        match find_user_by_id(&users, id) {
            Some(user) => println!("  ID {}: Found {:?}", id, user),
            None => println!("  ID {}: Not found", id),
        }
    }

    // Performance comparison
    println!("\n=== Performance: Linear vs Binary Search ===\n");

    let large_sorted: Vec<i32> = (0..100_000).collect();
    let targets = [0, 50_000, 99_999, 100_000];

    println!("Searching in {} elements:", large_sorted.len());

    for &target in &targets {
        // Linear search
        let start = std::time::Instant::now();
        let linear_result = large_sorted.iter().position(|&x| x == target);
        let linear_time = start.elapsed();

        // Binary search
        let start = std::time::Instant::now();
        let binary_result = large_sorted.binary_search(&target).ok();
        let binary_time = start.elapsed();

        println!(
            "  Target {}: linear {:?}, binary {:?}",
            target, linear_time, binary_time
        );
    }

    // Binary search for insertion point
    println!("\n=== Using Binary Search for Sorted Insertion ===\n");

    let mut sorted = vec![1, 3, 5, 7, 9];
    println!("Initial: {:?}", sorted);

    for value in [4, 0, 10, 6] {
        let idx = match sorted.binary_search(&value) {
            Ok(idx) => idx,     // Already exists
            Err(idx) => idx,    // Insert position
        };
        sorted.insert(idx, value);
        println!("  Insert {}: {:?}", value, sorted);
    }

    // Binary search with custom comparator
    println!("\n=== Binary Search with Custom Comparator ===\n");

    #[derive(Debug)]
    struct Event {
        timestamp: u64,
        name: String,
    }

    let events = vec![
        Event { timestamp: 100, name: "Start".to_string() },
        Event { timestamp: 200, name: "Process".to_string() },
        Event { timestamp: 300, name: "Complete".to_string() },
    ];

    let target_time = 200;
    let result = events.binary_search_by(|e| e.timestamp.cmp(&target_time));

    match result {
        Ok(idx) => println!("Event at timestamp {}: {:?}", target_time, events[idx]),
        Err(_) => println!("No event at timestamp {}", target_time),
    }

    println!("\n=== Key Points ===");
    println!("1. binary_search requires sorted data");
    println!("2. Returns Ok(index) if found, Err(insertion_point) if not");
    println!("3. O(log n) vs O(n) for linear search");
    println!("4. Use binary_search_by_key for searching by struct field");
}
