//! Pattern 3: if let, while let, and let-else
//! Example: while let for Iteration
//!
//! Run with: cargo run --example p3_while_let

use std::collections::VecDeque;

// Basic while-let with a queue
fn drain_queue(queue: &mut VecDeque<String>) {
    // Loop continues as long as pop_front returns Some(task)
    while let Some(task) = queue.pop_front() {
        println!("  Processing: {}", task);
    }
    println!("  Queue is now empty");
}

// while-let with iterator
fn process_iter() {
    let numbers = vec![1, 2, 3, 4, 5];
    let mut iter = numbers.into_iter();

    while let Some(n) = iter.next() {
        println!("  Got: {}", n);
    }
}

// while-let with Result for parsing
fn parse_numbers(input: &str) {
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            // Collect all consecutive digits
            let mut num_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Ok(n) = num_str.parse::<i32>() {
                println!("  Found number: {}", n);
            }
        } else {
            chars.next(); // Skip non-digit
        }
    }
}

// Stack processing with while-let
fn process_stack() {
    let mut stack = vec!['a', 'b', 'c', 'd'];

    println!("  Popping from stack:");
    while let Some(item) = stack.pop() {
        println!("    {}", item);
    }
}

// Nested while-let patterns
fn process_pairs(pairs: &mut VecDeque<(String, i32)>) {
    while let Some((name, value)) = pairs.pop_front() {
        println!("  {} = {}", name, value);
    }
}

// while-let with mutable binding
fn increment_all(queue: &mut VecDeque<i32>) -> Vec<i32> {
    let mut results = Vec::new();

    while let Some(mut n) = queue.pop_front() {
        n += 1; // Mutate the bound value
        results.push(n);
    }

    results
}

// Compare: while-let vs loop with match
fn while_let_style(opt: &mut Option<i32>) {
    while let Some(n) = opt.take() {
        println!("  while-let got: {}", n);
        if n > 1 {
            *opt = Some(n - 1);
        }
    }
}

fn loop_match_style(opt: &mut Option<i32>) {
    loop {
        match opt.take() {
            Some(n) => {
                println!("  loop-match got: {}", n);
                if n > 1 {
                    *opt = Some(n - 1);
                }
            }
            None => break,
        }
    }
}

fn main() {
    println!("=== while-let: Draining a Queue ===");
    // Usage: drain a queue until it's empty
    let mut queue = VecDeque::from([
        "task1".to_string(),
        "task2".to_string(),
        "task3".to_string(),
    ]);
    drain_queue(&mut queue);

    println!("\n=== while-let: Iterator ===");
    process_iter();

    println!("\n=== while-let: Stack (LIFO) ===");
    process_stack();

    println!("\n=== while-let: Parsing Numbers ===");
    parse_numbers("abc123def456xyz789");

    println!("\n=== while-let: Destructuring Tuples ===");
    let mut pairs = VecDeque::from([
        ("x".to_string(), 10),
        ("y".to_string(), 20),
        ("z".to_string(), 30),
    ]);
    process_pairs(&mut pairs);

    println!("\n=== while-let: Mutable Binding ===");
    let mut numbers = VecDeque::from([1, 2, 3, 4, 5]);
    let incremented = increment_all(&mut numbers);
    println!("  Incremented: {:?}", incremented);

    println!("\n=== Comparison: while-let vs loop-match ===");
    let mut opt1 = Some(3);
    println!("while-let style:");
    while_let_style(&mut opt1);

    let mut opt2 = Some(3);
    println!("loop-match style:");
    loop_match_style(&mut opt2);

    println!("\n=== while-let Syntax ===");
    println!("while let pattern = expression {{");
    println!("    // loop body executes while pattern matches");
    println!("}}");

    println!("\n=== Common Use Cases ===");
    println!("1. Draining collections (VecDeque, Vec)");
    println!("2. Consuming iterators manually");
    println!("3. Processing streams/channels");
    println!("4. Parsing with peekable iterators");
    println!("5. Stack/queue-based algorithms");
}
