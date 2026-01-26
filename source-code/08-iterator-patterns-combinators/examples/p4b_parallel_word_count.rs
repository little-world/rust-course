//! Pattern 4b: Parallel Iteration with Rayon
//! Example: Parallel Word Count (Fold with Combiner)
//!
//! Run with: cargo run --example p4b_parallel_word_count

use rayon::prelude::*;

/// Count words across lines in parallel.
/// Each thread counts locally, then results are summed.
fn parallel_word_count(lines: &[String]) -> usize {
    lines
        .par_iter()
        .map(|line| line.split_whitespace().count())
        .sum()
}

/// Count specific word occurrences.
fn parallel_count_word(lines: &[String], target: &str) -> usize {
    lines
        .par_iter()
        .map(|line| {
            line.split_whitespace()
                .filter(|&word| word == target)
                .count()
        })
        .sum()
}

/// Find longest line.
fn parallel_longest_line(lines: &[String]) -> Option<&String> {
    lines.par_iter().max_by_key(|s| s.len())
}

/// Average word length.
fn parallel_avg_word_length(lines: &[String]) -> f64 {
    let (total_len, total_count): (usize, usize) = lines
        .par_iter()
        .map(|line| {
            line.split_whitespace()
                .map(|w| (w.len(), 1))
                .fold((0, 0), |(len, cnt), (l, c)| (len + l, cnt + c))
        })
        .reduce(
            || (0, 0),
            |(len1, cnt1), (len2, cnt2)| (len1 + len2, cnt1 + cnt2),
        );

    if total_count == 0 {
        0.0
    } else {
        total_len as f64 / total_count as f64
    }
}

fn main() {
    println!("=== Parallel Word Count with Fold/Reduce ===\n");

    // Usage: count words across lines in parallel
    let lines: Vec<String> = vec![
        "hello world".into(),
        "foo bar baz".into(),
        "rust is awesome".into(),
    ];

    let count = parallel_word_count(&lines);
    println!("Lines: {:?}", lines);
    println!("Total word count: {}", count);
    // 2 + 3 + 3 = 8

    // Larger example
    println!("\n=== Larger Dataset ===");
    let large_lines: Vec<String> = (0..1000)
        .map(|i| format!("line {} with some words here", i))
        .collect();

    let total = parallel_word_count(&large_lines);
    println!("1000 lines, {} total words", total);

    // Count specific word
    println!("\n=== Count Specific Word ===");
    let rust_count = parallel_count_word(&large_lines, "with");
    println!("Occurrences of 'with': {}", rust_count);

    // Longest line
    println!("\n=== Find Longest Line ===");
    let sample_lines: Vec<String> = vec![
        "short".into(),
        "a medium length line".into(),
        "this is the longest line in our sample data set".into(),
        "tiny".into(),
    ];
    let longest = parallel_longest_line(&sample_lines);
    println!("Longest line: {:?}", longest);

    // Average word length
    println!("\n=== Average Word Length ===");
    let text_lines: Vec<String> = vec![
        "The quick brown fox".into(),
        "jumps over".into(),
        "the lazy dog".into(),
    ];
    let avg_len = parallel_avg_word_length(&text_lines);
    println!("Average word length: {:.2}", avg_len);

    println!("\n=== How Parallel Sum Works ===");
    println!("1. Split data across threads");
    println!("2. Each thread computes local sum");
    println!("3. sum() combines partial sums");
    println!("");
    println!("No locks needed - each thread works independently!");

    println!("\n=== Fold + Reduce Pattern ===");
    println!("fold: creates initial value per thread");
    println!("reduce: combines results from all threads");
    println!("");
    println!("Example for average:");
    println!("  fold: (0, 0) -> each thread sums (total_len, count)");
    println!("  reduce: combines all (len, count) pairs");

    println!("\n=== Key Points ===");
    println!("1. map().sum() for simple parallel aggregation");
    println!("2. fold()+reduce() for complex aggregations");
    println!("3. No synchronization - threads work independently");
    println!("4. Final combination is efficient tree reduction");
}
