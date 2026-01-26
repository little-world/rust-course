// Pattern 4: Zero-Cost Abstractions & Branch Prediction
// Demonstrates branch prediction and branchless code techniques.

use std::time::Instant;

// ============================================================================
// Example: Understanding Branch Misprediction
// ============================================================================

fn with_unpredictable_branch(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        // Unpredictable - depends on data
        if x % 2 == 0 {
            sum += x;
        }
    }
    sum
}

fn without_branch(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        // Branchless - uses arithmetic
        sum += x * (x % 2 == 0) as i32;
    }
    sum
}

// ============================================================================
// Example: Sorting for Branch Prediction
// ============================================================================

fn sum_if_positive_unsorted(data: &[i32]) -> i32 {
    let mut sum = 0;
    for &x in data {
        if x > 0 {
            sum += x;
        }
    }
    sum
}

fn sum_if_positive_sorted(data: &mut [i32]) -> i32 {
    data.sort_unstable();

    let mut sum = 0;
    for &x in data.iter() {
        if x > 0 {
            sum += x;
        }
    }
    sum
}

// ============================================================================
// Example: Branch-Free Code with Bitwise Operations
// ============================================================================

// With branch
fn max_with_branch(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

// Branchless
fn max_branchless(a: i32, b: i32) -> i32 {
    let diff = a.wrapping_sub(b);
    let sign = diff >> 31;  // -1 if negative, 0 if positive
    a.wrapping_sub(diff & sign)
}

// ============================================================================
// Example: Pattern Matching Optimization
// ============================================================================

#[derive(Debug)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

// Common case first for better branch prediction
fn process_message(msg: &Message) -> String {
    match msg {
        Message::Write(s) => s.clone(),  // Assume this is most common
        Message::Move { x, y } => format!("move {} {}", x, y),
        Message::ChangeColor(r, g, b) => format!("color {} {} {}", r, g, b),
        Message::Quit => "quit".to_string(),
    }
}

// ============================================================================
// Example: Likely/Unlikely pattern (stable version)
// ============================================================================

#[inline(never)]
#[cold]
fn handle_error(msg: &str) {
    eprintln!("Error: {}", msg);
}

fn process_value(x: i32) -> i32 {
    if x < 0 {
        handle_error("negative value");
        return 0;
    }
    x * 2
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_vs_branchless() {
        let data: Vec<i32> = (0..100).collect();
        let r1 = with_unpredictable_branch(&data);
        let r2 = without_branch(&data);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_max_implementations() {
        assert_eq!(max_with_branch(5, 3), 5);
        assert_eq!(max_with_branch(3, 5), 5);
        assert_eq!(max_with_branch(4, 4), 4);

        assert_eq!(max_branchless(5, 3), 5);
        assert_eq!(max_branchless(3, 5), 5);
        assert_eq!(max_branchless(4, 4), 4);

        // Test common cases (branchless has overflow issues with extreme values)
        assert_eq!(max_branchless(100, -100), 100);
        assert_eq!(max_branchless(-50, -25), -25);
    }

    #[test]
    fn test_sorted_vs_unsorted() {
        let data: Vec<i32> = vec![-5, 3, -2, 7, -1, 4, 0, 2];
        let mut sorted_data = data.clone();

        let sum1 = sum_if_positive_unsorted(&data);
        let sum2 = sum_if_positive_sorted(&mut sorted_data);

        assert_eq!(sum1, sum2);
        assert_eq!(sum1, 3 + 7 + 4 + 2); // 16
    }

    #[test]
    fn test_process_message() {
        let msg = Message::Write("hello".to_string());
        assert_eq!(process_message(&msg), "hello");

        let msg = Message::Move { x: 10, y: 20 };
        assert_eq!(process_message(&msg), "move 10 20");

        let msg = Message::Quit;
        assert_eq!(process_message(&msg), "quit");
    }

    #[test]
    fn test_process_value() {
        assert_eq!(process_value(5), 10);
        assert_eq!(process_value(-5), 0);
        assert_eq!(process_value(0), 0);
    }
}

fn main() {
    println!("Pattern 4: Zero-Cost Abstractions & Branch Prediction");
    println!("======================================================\n");

    // Generate random data for branch misprediction demo
    let random_data: Vec<i32> = (0..1_000_000)
        .map(|_| rand::random::<i32>() % 100)
        .collect();

    println!("Branch prediction benchmark (random data):");
    let start = Instant::now();
    let sum1 = with_unpredictable_branch(&random_data);
    println!("  With branch:   {:?} (sum={})", start.elapsed(), sum1);

    let start = Instant::now();
    let sum2 = without_branch(&random_data);
    println!("  Branchless:    {:?} (sum={})", start.elapsed(), sum2);

    // Sorted vs unsorted
    println!("\nSorted vs unsorted data:");
    let mut mixed_data: Vec<i32> = (-500_000..500_000).collect();
    use rand::seq::SliceRandom;
    mixed_data.shuffle(&mut rand::thread_rng());

    let start = Instant::now();
    let _ = sum_if_positive_unsorted(&mixed_data);
    println!("  Unsorted: {:?}", start.elapsed());

    let mut sorted_copy = mixed_data.clone();
    let start = Instant::now();
    let _ = sum_if_positive_sorted(&mut sorted_copy);
    println!("  Sorted:   {:?} (includes sort time)", start.elapsed());

    // Max implementations
    println!("\nMax implementations:");
    println!("  max_with_branch(10, 5) = {}", max_with_branch(10, 5));
    println!("  max_branchless(10, 5) = {}", max_branchless(10, 5));

    // Message processing
    println!("\nMessage processing:");
    let messages = vec![
        Message::Write("hello".to_string()),
        Message::Move { x: 10, y: 20 },
        Message::ChangeColor(255, 128, 0),
        Message::Quit,
    ];
    for msg in &messages {
        println!("  {:?} -> {}", msg, process_message(msg));
    }
}
