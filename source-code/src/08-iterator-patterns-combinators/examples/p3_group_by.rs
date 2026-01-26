//! Pattern 3: Advanced Iterator Composition
//! Example: Group by Key
//!
//! Run with: cargo run --example p3_group_by

use std::collections::HashMap;

/// Group elements by an arbitrary key function in a single pass.
fn group_by<K, V, F>(items: Vec<V>, key_fn: F) -> HashMap<K, Vec<V>>
where
    K: Eq + std::hash::Hash,
    F: Fn(&V) -> K,
{
    items.into_iter().fold(HashMap::new(), |mut map, item| {
        map.entry(key_fn(&item)).or_default().push(item);
        map
    })
}

/// Group by with transformation of values.
fn group_by_map<K, V, U, F, M>(items: Vec<V>, key_fn: F, map_fn: M) -> HashMap<K, Vec<U>>
where
    K: Eq + std::hash::Hash,
    F: Fn(&V) -> K,
    M: Fn(V) -> U,
{
    items.into_iter().fold(HashMap::new(), |mut map, item| {
        let key = key_fn(&item);
        map.entry(key).or_default().push(map_fn(item));
        map
    })
}

/// Group and count (like SQL GROUP BY ... COUNT(*))
fn group_count<K, V, F>(items: Vec<V>, key_fn: F) -> HashMap<K, usize>
where
    K: Eq + std::hash::Hash,
    F: Fn(&V) -> K,
{
    items.into_iter().fold(HashMap::new(), |mut map, item| {
        *map.entry(key_fn(&item)).or_insert(0) += 1;
        map
    })
}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    department: String,
    salary: u32,
}

fn main() {
    println!("=== Group by Key ===\n");

    // Usage: group numbers by even/odd
    let grouped = group_by(vec![1, 2, 3, 4, 5, 6], |x| x % 2);
    println!("Group [1,2,3,4,5,6] by even(0)/odd(1):");
    println!("  Even: {:?}", grouped.get(&0));
    println!("  Odd: {:?}", grouped.get(&1));

    println!("\n=== Group Strings by Length ===");
    let words = vec!["a", "bb", "ccc", "dd", "e", "fff", "gggg"];
    let by_len = group_by(words, |s| s.len());
    println!("Words grouped by length:");
    let mut keys: Vec<_> = by_len.keys().collect();
    keys.sort();
    for len in keys {
        println!("  len {}: {:?}", len, by_len.get(len));
    }

    println!("\n=== Group by First Character ===");
    let names = vec!["Alice", "Bob", "Anna", "Charlie", "Ben", "Carol"];
    let by_initial = group_by(names, |s| s.chars().next().unwrap());
    println!("Names grouped by initial:");
    let mut initials: Vec<_> = by_initial.keys().collect();
    initials.sort();
    for initial in initials {
        println!("  '{}': {:?}", initial, by_initial.get(initial));
    }

    println!("\n=== Complex Example: Employees by Department ===");
    let employees = vec![
        Person { name: "Alice".into(), department: "Engineering".into(), salary: 100000 },
        Person { name: "Bob".into(), department: "Sales".into(), salary: 80000 },
        Person { name: "Carol".into(), department: "Engineering".into(), salary: 95000 },
        Person { name: "Dave".into(), department: "Sales".into(), salary: 85000 },
        Person { name: "Eve".into(), department: "HR".into(), salary: 70000 },
    ];

    // Group by department, extracting just names
    let by_dept = group_by_map(
        employees.clone(),
        |p| p.department.clone(),
        |p| p.name,
    );
    println!("Employees by department (names only):");
    for (dept, names) in &by_dept {
        println!("  {}: {:?}", dept, names);
    }

    println!("\n=== Count per Department ===");
    let counts = group_count(employees, |p| p.department.clone());
    println!("Employee count by department:");
    for (dept, count) in &counts {
        println!("  {}: {}", dept, count);
    }

    println!("\n=== Key Points ===");
    println!("1. Single pass with fold and HashMap");
    println!("2. or_default() ensures each key starts with empty Vec");
    println!("3. Key function can extract any hashable field");
    println!("4. Common pattern for bucketing/categorizing data");
}
