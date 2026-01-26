// Pattern 3: Iterator Combinators Reference
// Demonstrates map, filter, filter_map, flat_map, take, skip, chain, zip, etc.

use std::collections::HashMap;

// ============================================================================
// Example: Creating Iterators
// ============================================================================

fn creating_iterators() {
    let data = vec![1, 2, 3];

    // iter() borrows elements immutably
    let sum: i32 = data.iter().sum();
    println!("Sum via iter(): {}", sum);
    // data is still valid

    // iter_mut() borrows elements mutably
    let mut data = vec![1, 2, 3];
    for item in data.iter_mut() {
        *item *= 2;
    }
    println!("After iter_mut(): {:?}", data);

    // into_iter() takes ownership
    let data = vec![1, 2, 3];
    let _collected: Vec<i32> = data.into_iter().collect();
    // data is now invalid (moved)
}

fn manual_iterator_creation() {
    // Range iterators
    let range: Vec<i32> = (0..5).collect();
    println!("Range 0..5: {:?}", range);

    let inclusive: Vec<i32> = (0..=5).collect();
    println!("Range 0..=5: {:?}", inclusive);

    // Infinite iterator with take
    let first_five: Vec<i32> = (0..).take(5).collect();
    println!("First 5 from infinite: {:?}", first_five);

    // From functions
    let single: Vec<i32> = std::iter::once(42).collect();
    println!("Single element: {:?}", single);

    let repeated: Vec<i32> = std::iter::repeat(7).take(5).collect();
    println!("Repeated 5 times: {:?}", repeated);

    // Empty iterator
    let empty: Vec<i32> = std::iter::empty().collect();
    println!("Empty: {:?}", empty);
}

// ============================================================================
// Example: Mapping - Transforming Elements
// ============================================================================

fn mapping_examples() {
    // map: Transform each element
    let numbers = vec![1, 2, 3];
    let doubled: Vec<_> = numbers.iter().map(|x| x * 2).collect();
    println!("Doubled: {:?}", doubled);

    // map with complex transformations
    let users = vec!["Alice", "Bob"];
    let greetings: Vec<_> = users
        .iter()
        .map(|name| format!("Hello, {}!", name))
        .collect();
    println!("Greetings: {:?}", greetings);

    // filter_map: Map and filter in one pass
    let inputs = vec!["42", "abc", "100"];
    let numbers: Vec<i32> = inputs.iter().filter_map(|s| s.parse().ok()).collect();
    println!("Parsed numbers: {:?}", numbers);

    // flat_map: Map and flatten
    let words = vec!["hello", "world"];
    let chars: Vec<_> = words.iter().flat_map(|word| word.chars()).collect();
    println!("Flattened chars: {:?}", chars);
}

// ============================================================================
// Example: Filtering - Selecting Elements
// ============================================================================

fn filtering_examples() {
    let numbers = vec![1, 2, 3, 4, 5, 6];

    // filter: Keep elements matching predicate
    let evens: Vec<_> = numbers.iter().filter(|&&x| x % 2 == 0).copied().collect();
    println!("Evens: {:?}", evens);

    // take: First N elements
    let first_three: Vec<_> = (1..=100).take(3).collect();
    println!("First 3: {:?}", first_three);

    // take_while: Elements until predicate fails
    let less_than_five: Vec<_> = (1..=10).take_while(|&x| x < 5).collect();
    println!("Less than 5: {:?}", less_than_five);

    // skip: Skip first N elements
    let skip_first: Vec<_> = vec![1, 2, 3, 4, 5].into_iter().skip(2).collect();
    println!("Skip first 2: {:?}", skip_first);

    // skip_while: Skip until predicate fails
    let skip_small: Vec<_> = vec![1, 2, 3, 4, 5]
        .into_iter()
        .skip_while(|&x| x < 3)
        .collect();
    println!("Skip while < 3: {:?}", skip_small);
}

// ============================================================================
// Example: Combining - Multiple Iterators
// ============================================================================

fn combining_examples() {
    // chain: Concatenate iterators
    let a = vec![1, 2];
    let b = vec![3, 4];
    let combined: Vec<_> = a.iter().chain(b.iter()).copied().collect();
    println!("Chained: {:?}", combined);

    // zip: Pair elements from two iterators
    let names = vec!["Alice", "Bob"];
    let ages = vec![30, 25];
    let people: Vec<_> = names.iter().zip(ages.iter()).collect();
    println!("Zipped: {:?}", people);

    // Zip stops at shortest
    let short = vec![1, 2];
    let long = vec![10, 20, 30, 40];
    let pairs: Vec<_> = short.iter().zip(long.iter()).collect();
    println!("Zipped (shortest): {:?}", pairs);

    // enumerate: Add indices
    let letters = vec!['a', 'b', 'c'];
    let indexed: Vec<_> = letters.iter().enumerate().collect();
    println!("Enumerated: {:?}", indexed);
}

// ============================================================================
// Example: Inspection
// ============================================================================

fn inspection_examples() {
    // inspect: Peek at elements without consuming
    let sum: i32 = (1..=3)
        .inspect(|x| print!("Processing {} -> ", x))
        .map(|x| x * 2)
        .inspect(|x| print!("{}, ", x))
        .sum();
    println!("\nSum: {}", sum);
}

// ============================================================================
// Example: Collecting - Building Collections
// ============================================================================

fn collecting_examples() {
    use std::collections::{HashMap, HashSet};

    // collect into Vec
    let vec: Vec<i32> = (1..=5).collect();
    println!("Vec: {:?}", vec);

    // collect into HashSet (deduplicates)
    let set: HashSet<_> = vec![1, 2, 2, 3].into_iter().collect();
    println!("HashSet: {:?}", set);

    // collect into HashMap from tuples
    let map: HashMap<_, _> = vec![("a", 1), ("b", 2)].into_iter().collect();
    println!("HashMap: {:?}", map);

    // collect into String
    let chars = vec!['h', 'e', 'l', 'l', 'o'];
    let word: String = chars.into_iter().collect();
    println!("String: {}", word);

    // partition: Split into two collections
    let numbers = vec![1, 2, 3, 4, 5];
    let (evens, odds): (Vec<_>, Vec<_>) = numbers.into_iter().partition(|&x| x % 2 == 0);
    println!("Evens: {:?}, Odds: {:?}", evens, odds);
}

// ============================================================================
// Example: Searching - Finding Elements
// ============================================================================

fn searching_examples() {
    let numbers = vec![1, 2, 3, 4];

    // find: First element matching predicate
    let first_even = numbers.iter().find(|&&x| x % 2 == 0);
    println!("First even: {:?}", first_even);

    // position: Index of first match
    let pos = numbers.iter().position(|&x| x == 3);
    println!("Position of 3: {:?}", pos);

    // any: Check if any element matches
    let has_even = numbers.iter().any(|&x| x % 2 == 0);
    println!("Has even: {}", has_even);

    // all: Check if all elements match
    let all_positive = numbers.iter().all(|&x| x > 0);
    println!("All positive: {}", all_positive);

    // nth: Get element at index
    let third = numbers.iter().nth(2);
    println!("Third element: {:?}", third);

    // last: Get last element
    let last = numbers.iter().last();
    println!("Last element: {:?}", last);
}

// ============================================================================
// Example: Aggregating - Reducing to Single Values
// ============================================================================

fn aggregating_examples() {
    // sum
    let total: i32 = (1..=10).sum();
    println!("Sum 1..=10: {}", total);

    // product
    let factorial: i32 = (1..=5).product();
    println!("5! = {}", factorial);

    // fold: Custom accumulation
    let sum = (1..=5).fold(0, |acc, x| acc + x);
    println!("Fold sum: {}", sum);

    // fold for string joining
    let sentence = vec!["Hello", "world"];
    let joined = sentence.into_iter().fold(String::new(), |mut acc, word| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(word);
        acc
    });
    println!("Joined: {}", joined);

    // reduce: Like fold but uses first element
    let max = vec![3, 1, 4, 1, 5].into_iter().reduce(|a, b| a.max(b));
    println!("Max via reduce: {:?}", max);

    // max/min
    let max = vec![3, 1, 4].into_iter().max();
    let min = vec![3, 1, 4].into_iter().min();
    println!("Max: {:?}, Min: {:?}", max, min);

    // max_by_key
    let words = vec!["short", "longer", "longest"];
    let longest = words.iter().max_by_key(|word| word.len());
    println!("Longest word: {:?}", longest);
}

// ============================================================================
// Example: Counting and Testing
// ============================================================================

fn counting_examples() {
    // count
    let count = (1..=100).filter(|x| x % 2 == 0).count();
    println!("Even numbers 1..=100: {}", count);

    // for_each: Side effects
    print!("for_each: ");
    (1..=5).for_each(|x| print!("{} ", x));
    println!();
}

// ============================================================================
// Example: Real-World Pipelines
// ============================================================================

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

fn real_world_pipelines() {
    // Process log lines
    let log_lines = vec![
        "ERROR: Database connection failed",
        "INFO: Server started",
        "ERROR: Null pointer exception",
        "WARN: High memory usage",
    ];

    let error_count = log_lines
        .iter()
        .filter(|line| line.starts_with("ERROR"))
        .count();
    println!("Error count: {}", error_count);

    // Transform and collect user data
    let users = vec![
        User {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        },
        User {
            name: "Bob".to_string(),
            age: 25,
            active: false,
        },
        User {
            name: "Charlie".to_string(),
            age: 35,
            active: true,
        },
    ];

    let active_names: Vec<String> = users
        .into_iter()
        .filter(|user| user.active)
        .map(|user| user.name)
        .collect();
    println!("Active users: {:?}", active_names);

    // Nested iteration with flat_map
    let teams = vec![vec!["Alice", "Bob"], vec!["Charlie", "Dave", "Eve"]];

    let all_members: Vec<_> = teams
        .iter()
        .flat_map(|team| team.iter())
        .copied()
        .collect();
    println!("All team members: {:?}", all_members);

    // Grouping with fold
    let words = vec!["apple", "apricot", "banana", "blueberry"];
    let grouped: HashMap<char, Vec<&str>> =
        words
            .into_iter()
            .fold(HashMap::new(), |mut map, word| {
                map.entry(word.chars().next().unwrap())
                    .or_insert_with(Vec::new)
                    .push(word);
                map
            });
    println!("Grouped by first letter: {:?}", grouped);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map() {
        let doubled: Vec<i32> = vec![1, 2, 3].iter().map(|x| x * 2).collect();
        assert_eq!(doubled, vec![2, 4, 6]);
    }

    #[test]
    fn test_filter() {
        let evens: Vec<i32> = (1..=10).filter(|&x| x % 2 == 0).collect();
        assert_eq!(evens, vec![2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_filter_map() {
        let inputs = vec!["42", "abc", "100"];
        let numbers: Vec<i32> = inputs.iter().filter_map(|s| s.parse().ok()).collect();
        assert_eq!(numbers, vec![42, 100]);
    }

    #[test]
    fn test_flat_map() {
        let words = vec!["ab", "cd"];
        let chars: Vec<char> = words.iter().flat_map(|w| w.chars()).collect();
        assert_eq!(chars, vec!['a', 'b', 'c', 'd']);
    }

    #[test]
    fn test_take() {
        let first: Vec<i32> = (1..=100).take(3).collect();
        assert_eq!(first, vec![1, 2, 3]);
    }

    #[test]
    fn test_skip() {
        let rest: Vec<i32> = vec![1, 2, 3, 4, 5].into_iter().skip(2).collect();
        assert_eq!(rest, vec![3, 4, 5]);
    }

    #[test]
    fn test_chain() {
        let combined: Vec<i32> = vec![1, 2].into_iter().chain(vec![3, 4]).collect();
        assert_eq!(combined, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_zip() {
        let pairs: Vec<_> = vec![1, 2].into_iter().zip(vec!['a', 'b']).collect();
        assert_eq!(pairs, vec![(1, 'a'), (2, 'b')]);
    }

    #[test]
    fn test_enumerate() {
        let indexed: Vec<_> = vec!['a', 'b'].into_iter().enumerate().collect();
        assert_eq!(indexed, vec![(0, 'a'), (1, 'b')]);
    }

    #[test]
    fn test_sum() {
        let total: i32 = (1..=10).sum();
        assert_eq!(total, 55);
    }

    #[test]
    fn test_product() {
        let factorial: i32 = (1..=5).product();
        assert_eq!(factorial, 120);
    }

    #[test]
    fn test_fold() {
        let sum = (1..=5).fold(0, |acc, x| acc + x);
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_reduce() {
        let max = vec![3, 1, 4, 1, 5].into_iter().reduce(|a, b| a.max(b));
        assert_eq!(max, Some(5));
    }

    #[test]
    fn test_find() {
        let first_even = (1..=10).find(|&x| x % 2 == 0);
        assert_eq!(first_even, Some(2));
    }

    #[test]
    fn test_any_all() {
        let numbers = vec![2, 4, 6];
        assert!(numbers.iter().all(|&x| x % 2 == 0));
        assert!(numbers.iter().any(|&x| x > 4));
    }

    #[test]
    fn test_partition() {
        let (evens, odds): (Vec<i32>, Vec<i32>) =
            vec![1, 2, 3, 4, 5].into_iter().partition(|&x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
        assert_eq!(odds, vec![1, 3, 5]);
    }

    #[test]
    fn test_collect_to_string() {
        let word: String = vec!['h', 'e', 'l', 'l', 'o'].into_iter().collect();
        assert_eq!(word, "hello");
    }
}

fn main() {
    println!("Pattern 3: Iterator Combinators Reference");
    println!("==========================================\n");

    println!("Creating Iterators:");
    creating_iterators();
    println!();

    println!("Manual Iterator Creation:");
    manual_iterator_creation();
    println!();

    println!("Mapping Examples:");
    mapping_examples();
    println!();

    println!("Filtering Examples:");
    filtering_examples();
    println!();

    println!("Combining Examples:");
    combining_examples();
    println!();

    println!("Inspection Example:");
    inspection_examples();
    println!();

    println!("Collecting Examples:");
    collecting_examples();
    println!();

    println!("Searching Examples:");
    searching_examples();
    println!();

    println!("Aggregating Examples:");
    aggregating_examples();
    println!();

    println!("Counting Examples:");
    counting_examples();
    println!();

    println!("Real-World Pipelines:");
    real_world_pipelines();
}
