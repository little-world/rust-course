//! Pattern 1: The Entry API
//! Word Frequency Counter and Grouping Items
//!
//! Run with: cargo run --example p1_entry_api

use std::collections::HashMap;

fn main() {
    println!("=== The Entry API ===\n");

    // Word frequency counter
    println!("=== Word Frequency Counter ===\n");
    word_frequency_counter();

    // Grouping items by key
    println!("\n=== Grouping Items by Key ===\n");
    group_sales_by_category();

    println!("\n=== Key Points ===");
    println!("1. Entry API reduces hash lookups from 2+ to 1");
    println!("2. or_insert(default) for simple defaults");
    println!("3. or_insert_with(|| ...) for lazy initialization");
    println!("4. Perfect for counting, grouping, and aggregation");
}

fn word_frequency_counter() {
    let text = "the quick brown fox jumps over the lazy dog";
    let mut counts: HashMap<String, usize> = HashMap::new();

    for word in text.split_whitespace() {
        // Get the entry for the word, insert 0 if it's vacant,
        // and then get a mutable reference to the value to increment it.
        *counts.entry(word.to_string()).or_insert(0) += 1;
    }

    println!("Text: \"{}\"", text);
    println!("Word counts: {:?}", counts);

    // Top word is "the" with a count of 2.
    assert_eq!(counts.get("the"), Some(&2));
    println!("'the' appears {} times", counts.get("the").unwrap());
}

#[derive(Debug)]
struct Sale {
    category: String,
    amount: f64,
}

fn group_sales_by_category() {
    let sales = vec![
        Sale { category: "Electronics".to_string(), amount: 1200.0 },
        Sale { category: "Furniture".to_string(), amount: 450.0 },
        Sale { category: "Electronics".to_string(), amount: 25.0 },
    ];

    let mut sales_by_category: HashMap<String, Vec<Sale>> = HashMap::new();

    for sale in sales {
        // Find the vector for the category, creating a new one if it doesn't exist,
        // and then push the sale into it.
        sales_by_category
            .entry(sale.category.clone())
            .or_insert_with(Vec::new)
            .push(sale);
    }

    println!("Sales grouped by category:");
    for (category, sales) in &sales_by_category {
        let total: f64 = sales.iter().map(|s| s.amount).sum();
        println!("  {}: {} sales, total ${:.2}", category, sales.len(), total);
    }

    assert_eq!(sales_by_category.get("Electronics").unwrap().len(), 2);
}
