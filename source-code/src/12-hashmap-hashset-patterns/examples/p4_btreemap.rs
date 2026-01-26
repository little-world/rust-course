//! Pattern 4: Alternative Map Implementations
//! BTreeMap for Ordered Operations
//!
//! Run with: cargo run --example p4_btreemap

use std::collections::BTreeMap;

fn main() {
    println!("=== BTreeMap for Ordered Operations ===\n");

    leaderboard();

    // Time-series example
    println!("\n=== Time-Series Data ===\n");
    time_series();

    println!("\n=== Key Points ===");
    println!("1. BTreeMap keeps keys sorted automatically");
    println!("2. O(log n) operations vs O(1) for HashMap");
    println!("3. Supports efficient range queries");
    println!("4. Deterministic iteration order");
}

fn leaderboard() {
    // Scores are the keys, so they are kept sorted.
    let mut scores = BTreeMap::new();
    scores.insert(1500, "Alice".to_string());
    scores.insert(2200, "David".to_string());
    scores.insert(1800, "Charlie".to_string());
    scores.insert(1950, "Bob".to_string());
    scores.insert(2100, "Eve".to_string());

    // iter() returns sorted order. rev() gives descending order.
    println!("Leaderboard (Top 5):");
    for (rank, (score, name)) in scores.iter().rev().enumerate() {
        println!("  {}. {}: {} points", rank + 1, name, score);
    }

    // BTreeMap also supports efficient range queries.
    println!("\nPlayers with scores between 1500 and 2000:");
    for (score, name) in scores.range(1500..=2000) {
        println!("  {}: {} points", name, score);
    }

    // First and last
    println!("\nHighest score: {:?}", scores.last_key_value());
    println!("Lowest score: {:?}", scores.first_key_value());
}

fn time_series() {
    let mut events: BTreeMap<u64, String> = BTreeMap::new();

    // Timestamps as keys (auto-sorted)
    events.insert(1000, "Server started".to_string());
    events.insert(1005, "First request received".to_string());
    events.insert(1010, "Cache warmed up".to_string());
    events.insert(1025, "Peak load reached".to_string());
    events.insert(1050, "Load normalized".to_string());

    println!("All events (chronological):");
    for (timestamp, event) in &events {
        println!("  [{}] {}", timestamp, event);
    }

    // Range query: events between t=1005 and t=1030
    println!("\nEvents between t=1005 and t=1030:");
    for (timestamp, event) in events.range(1005..=1030) {
        println!("  [{}] {}", timestamp, event);
    }

    // Events after t=1020
    println!("\nEvents after t=1020:");
    for (timestamp, event) in events.range(1021..) {
        println!("  [{}] {}", timestamp, event);
    }
}
