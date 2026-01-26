//! Pattern 2: Zero-Allocation Iteration
//! Example: `fold` for Custom Reductions
//!
//! Run with: cargo run --example p2_fold

/// Count strings longer than 10 characters without creating intermediate collections.
fn count_long_strings(strings: &[&str]) -> usize {
    strings
        .iter()
        .fold(0, |n, &s| if s.len() > 10 { n + 1 } else { n })
}

/// Compute both sum and count in a single pass.
fn sum_and_count(numbers: &[i32]) -> (i32, usize) {
    numbers
        .iter()
        .fold((0, 0), |(sum, count), &x| (sum + x, count + 1))
}

/// Find min and max in one pass.
fn min_max(numbers: &[i32]) -> Option<(i32, i32)> {
    numbers
        .iter()
        .fold(None, |acc, &x| {
            match acc {
                None => Some((x, x)),
                Some((min, max)) => Some((min.min(x), max.max(x))),
            }
        })
}

/// Build a string by concatenating with separator.
fn join_with_separator(items: &[&str], sep: &str) -> String {
    items
        .iter()
        .fold(String::new(), |mut acc, &s| {
            if !acc.is_empty() {
                acc.push_str(sep);
            }
            acc.push_str(s);
            acc
        })
}

/// Count occurrences of each character.
fn char_frequency(text: &str) -> std::collections::HashMap<char, usize> {
    text.chars()
        .fold(std::collections::HashMap::new(), |mut map, c| {
            *map.entry(c).or_insert(0) += 1;
            map
        })
}

fn main() {
    println!("=== Custom Reductions with fold ===\n");

    // Usage: count strings longer than 10 characters
    let count = count_long_strings(&["short", "a long string here", "tiny"]);
    println!("Long strings count: {}", count);

    println!("\n=== Multiple Aggregates in One Pass ===");
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let (sum, count) = sum_and_count(&numbers);
    println!("Numbers: {:?}", numbers);
    println!("Sum: {}, Count: {}, Average: {:.2}", sum, count, sum as f64 / count as f64);

    println!("\n=== Min and Max in One Pass ===");
    let values = [5, 2, 8, 1, 9, 3, 7];
    let result = min_max(&values);
    println!("Values: {:?}", values);
    println!("Min/Max: {:?}", result);

    println!("\n=== String Joining ===");
    let words = ["hello", "world", "from", "rust"];
    let joined = join_with_separator(&words, ", ");
    println!("Words: {:?}", words);
    println!("Joined: {}", joined);

    println!("\n=== Character Frequency ===");
    let text = "hello world";
    let freq = char_frequency(text);
    println!("Text: '{}'", text);
    println!("Character frequencies:");
    let mut entries: Vec<_> = freq.iter().collect();
    entries.sort_by_key(|&(c, _)| c);
    for (c, count) in entries {
        if !c.is_whitespace() {
            println!("  '{}': {}", c, count);
        }
    }

    println!("\n=== fold vs filter().count() ===");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Less efficient: creates an iterator, then counts
    let count1 = data.iter().filter(|&&x| x > 5).count();

    // More efficient with fold: single pass, no intermediate state
    let count2 = data.iter().fold(0, |n, &x| if x > 5 { n + 1 } else { n });

    println!("Count with filter().count(): {}", count1);
    println!("Count with fold: {}", count2);

    println!("\n=== Key Points ===");
    println!("1. fold takes initial accumulator and combining closure");
    println!("2. More flexible than sum(), count(), etc.");
    println!("3. Can compute multiple aggregates in one pass");
    println!("4. Accumulator can be any type (tuples, HashMaps, etc.)");
}
